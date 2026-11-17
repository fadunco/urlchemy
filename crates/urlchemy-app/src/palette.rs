use crate::{CloseReason, Launcher, PaletteCommand as Cmd, PaletteEffect};
use gpui_kit::component::{
    ActiveTheme, Disableable, IndexPath,
    button::{Button, ButtonVariants},
    input::{InputEditsEvent, Textarea, TextareaState},
    list::{List, ListDelegate, ListItem, ListState},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::{collections::BTreeSet, rc::Rc};
use urlchemy_core::{EntryId, PrimaryActionHint, Row, SessionSnapshot, StepId};

#[derive(Clone, Debug)]
pub enum PaletteEvent {
    Effect(PaletteEffect),
    ContentHeight { invocation: u64, height: Pixels },
}
#[derive(Clone, PartialEq, Eq)]
enum Scope {
    Browse(Vec<EntryId>),
    Input(StepId),
    Choice(StepId),
    Other,
}
fn scope(snapshot: &SessionSnapshot) -> Scope {
    match snapshot {
        SessionSnapshot::Browsing { folders, .. } => Scope::Browse(folders.clone()),
        SessionSnapshot::Input { step, .. } => Scope::Input(*step),
        SessionSnapshot::Selection { step, .. } => Scope::Choice(*step),
        _ => Scope::Other,
    }
}
#[derive(Default)]
struct PaletteBindings(BTreeSet<String>);
impl Global for PaletteBindings {}

pub struct PaletteView {
    launcher: Launcher,
    editor: Entity<TextareaState>,
    _edits: Subscription,
    list: Entity<ListState<PaletteListDelegate>>,
    scope: Scope,
    generation: u64,
    revision: u64,
    launch: Option<KeybindingKeystroke>,
    launch_label: String,
    primary_focus: FocusHandle,
    back_focus: FocusHandle,
    close_focus: FocusHandle,
    measured_height: Option<Pixels>,
}
impl EventEmitter<PaletteEvent> for PaletteView {}
impl PaletteView {
    fn make_editor(
        text: &str,
        generation: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (Entity<TextareaState>, Subscription) {
        let input = cx.new(|cx| {
            let mut input = TextareaState::new(window, cx).rows(1).submit_on_enter(true);
            input.set_value(text, window, cx);
            input
        });
        let edits = cx.subscribe_in(
            &input,
            window,
            move |this, _, event: &InputEditsEvent, window, cx| {
                if this.generation == generation
                    && let Some((_, text)) = event.edits().last()
                {
                    this.dispatch(Cmd::Edited(text.to_string()), window, cx);
                }
            },
        );
        (input, edits)
    }
    pub fn new(launcher: Launcher, window: &mut Window, cx: &mut Context<Self>) -> Self {
        if !cx.has_global::<PaletteBindings>() {
            cx.set_global(PaletteBindings::default());
        }
        let predicate = Rc::new(
            KeyBindingContextPredicate::parse("URLchemyPalette > Input")
                .expect("static input context"),
        );
        let launch_label = launcher.catalog().shortcuts().launch().binding().to_owned();
        let loaded = KeyBinding::load(
            &launch_label,
            Box::new(NoAction),
            Some(predicate.clone()),
            false,
            None,
            cx,
        )
        .ok();
        let launch = loaded
            .as_ref()
            .and_then(|binding| binding.keystrokes().first())
            .cloned();
        // Scoped suppression leaves raw events available with repeat/composition metadata.
        for key in ["enter", "up", "down", "escape", "cmd-["]
            .into_iter()
            .chain(std::iter::once(launch_label.as_str()))
        {
            if cx
                .global_mut::<PaletteBindings>()
                .0
                .insert(format!("input:{key}"))
                && let Ok(binding) = KeyBinding::load(
                    key,
                    Box::new(NoAction),
                    Some(predicate.clone()),
                    false,
                    None,
                    cx,
                )
            {
                cx.bind_keys([binding]);
            }
        }
        let palette = Rc::new(
            KeyBindingContextPredicate::parse("URLchemyPalette").expect("static palette context"),
        );
        for key in ["escape", "cmd-["] {
            if cx
                .global_mut::<PaletteBindings>()
                .0
                .insert(format!("palette:{key}"))
                && let Ok(binding) = KeyBinding::load(
                    key,
                    Box::new(NoAction),
                    Some(palette.clone()),
                    false,
                    None,
                    cx,
                )
            {
                cx.bind_keys([binding]);
            }
        }
        let (editor, edits) = Self::make_editor(launcher.query(), 1, window, cx);
        editor.update(cx, |input, cx| input.focus(window, cx));
        let rows = launcher.rows().to_vec();
        let selected = launcher
            .active()
            .and_then(|id| rows.iter().position(|row| row.id() == id))
            .map(IndexPath::new);
        let owner = cx.weak_entity();
        let list = cx.new(|cx| {
            let mut list = ListState::new(
                PaletteListDelegate {
                    owner,
                    rows,
                    selected,
                    revision: 0,
                },
                window,
                cx,
            )
            .searchable(false);
            list.set_selected_index(selected, window, cx);
            list
        });
        let current = launcher
            .snapshot()
            .map(|snapshot| scope(&snapshot))
            .unwrap_or(Scope::Other);
        Self {
            launcher,
            editor,
            _edits: edits,
            list,
            scope: current,
            generation: 1,
            revision: 0,
            launch,
            launch_label,
            primary_focus: cx.focus_handle().tab_index(1),
            back_focus: cx.focus_handle().tab_index(2),
            close_focus: cx.focus_handle().tab_index(3),
            measured_height: None,
        }
    }
    /// The actual editor entity, shared with focus and input-handler integrations.
    pub fn editor(&self) -> Entity<TextareaState> {
        self.editor.clone()
    }
    pub fn invocation(&self) -> u64 {
        self.launcher.invocation()
    }
    pub fn dispatch(&mut self, command: Cmd, window: &mut Window, cx: &mut Context<Self>) {
        let back = matches!(command, Cmd::Back);
        let refocus = !matches!(command, Cmd::OpenFinished { .. } | Cmd::Dismiss(_));
        let effect = self.launcher.dispatch(command);
        let current = self
            .launcher
            .snapshot()
            .map(|snapshot| scope(&snapshot))
            .unwrap_or(Scope::Other);
        if current != self.scope {
            self.scope = current;
            self.generation = self
                .generation
                .checked_add(1)
                .expect("editor generation exhausted");
            let (editor, edits) =
                Self::make_editor(self.launcher.query(), self.generation, window, cx);
            self.editor = editor;
            self._edits = edits;
            if refocus {
                self.editor.update(cx, |input, cx| {
                    input.focus(window, cx);
                    if back {
                        input.select_all(window, cx);
                    }
                });
            }
        } else if self.editor.read(cx).value().as_str() != self.launcher.query() {
            self.editor.update(cx, |input, cx| {
                input.set_value(self.launcher.query(), window, cx)
            });
        }
        self.revision = self.revision.wrapping_add(1);
        let rows = self.launcher.rows().to_vec();
        let selected = self
            .launcher
            .active()
            .and_then(|id| rows.iter().position(|row| row.id() == id))
            .map(IndexPath::new);
        self.list.update(cx, |list, cx| {
            list.delegate_mut().rows = rows;
            list.delegate_mut().revision = self.revision;
            list.set_selected_index(selected, window, cx);
            if selected.is_some() {
                list.scroll_to_selected_item(window, cx);
            }
            cx.notify();
        });
        if self.launcher.opening_error().is_some() {
            self.primary_focus.focus(window, cx);
        } else if matches!(
            self.launcher.snapshot(),
            Ok(SessionSnapshot::DestinationError { .. })
        ) {
            self.back_focus.focus(window, cx);
        } else if refocus
            && matches!(
                self.scope,
                Scope::Input(_) | Scope::Browse(_) | Scope::Choice(_)
            )
        {
            self.editor.update(cx, |input, cx| input.focus(window, cx));
        }
        cx.notify();
        if let Some(effect) = effect {
            cx.emit(PaletteEvent::Effect(effect));
        }
    }
    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = &event.keystroke;
        let plain = key.modifiers == Modifiers::default();
        let editor_focused = self.editor.read(cx).focus_handle(cx).is_focused(window);
        let launch = editor_focused
            && self
                .launch
                .as_ref()
                .is_some_and(|target| key.should_match(target));
        let escape = plain && key.key == "escape";
        let back = key.key == "["
            && key.modifiers.platform
            && !key.modifiers.control
            && !key.modifiers.alt
            && !key.modifiers.shift;
        let arrow = editor_focused && plain && matches!(key.key.as_str(), "up" | "down");
        if !launch && !escape && !back && !arrow && !(editor_focused && plain && key.key == "enter")
        {
            return;
        }
        cx.stop_propagation();
        if event.is_held && (launch || escape || back) {
            return;
        }
        let composing = self.editor.update(cx, |input, cx| {
            EntityInputHandler::marked_text_range(input, window, cx).is_some()
        });
        if composing {
            if escape {
                self.editor.update(cx, |input, cx| {
                    input.set_value(self.launcher.query(), window, cx)
                });
            } else if launch {
                self.editor.update(cx, |input, cx| {
                    EntityInputHandler::unmark_text(input, window, cx)
                });
            }
            return;
        }
        if escape {
            self.dispatch(Cmd::Dismiss(CloseReason::Explicit), window, cx);
        } else if back {
            self.dispatch(Cmd::Back, window, cx);
        } else if arrow {
            self.dispatch(
                Cmd::Move(if key.key == "down" { 1 } else { -1 }),
                window,
                cx,
            );
        } else if launch {
            let text = self.editor.read(cx).value().to_string();
            let generation = self.generation;
            if text != self.launcher.query() {
                self.dispatch(Cmd::Edited(text), window, cx);
            }
            if generation == self.generation {
                self.dispatch(Cmd::Confirm, window, cx);
            }
        }
    }
}

struct PaletteListDelegate {
    owner: WeakEntity<PaletteView>,
    rows: Vec<Row>,
    selected: Option<IndexPath>,
    revision: u64,
}
impl ListDelegate for PaletteListDelegate {
    type Item = ListItem;
    fn items_count(&self, _: usize, _: &App) -> usize {
        self.rows.len()
    }
    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<ListItem> {
        let row = self.rows.get(ix.row)?;
        Some(
            ListItem::new(SharedString::from(format!("row:{:?}", row.id())))
                .accessibility_label(row.label().to_owned())
                .child(row.label().to_owned()),
        )
    }
    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) {
        self.selected = ix;
    }
    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<ListState<Self>>) {
        let Some(row) = self.selected.and_then(|ix| self.rows.get(ix.row)) else {
            return;
        };
        let id = row.id();
        let owner = self.owner.clone();
        let revision = self.revision;
        let handle = window.window_handle();
        cx.defer(move |cx| {
            let _ = handle.update(cx, |_, window, cx| {
                let _ = owner.update(cx, |view, cx| {
                    if revision == view.revision {
                        view.dispatch(Cmd::Activate(id), window, cx)
                    }
                });
            });
        });
    }
    fn render_empty(
        &mut self,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> impl IntoElement {
        div().child("No matches")
    }
}

impl Render for PaletteView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.launcher.snapshot().ok();
        let presentation = self.launcher.presentation().ok();
        let label = match &snapshot {
            Some(SessionSnapshot::Input { label, .. })
            | Some(SessionSnapshot::Selection { label, .. }) => label.clone(),
            _ => "Open a flow".into(),
        };
        let context = presentation
            .as_ref()
            .map(|p| p.context().join(" / "))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "URLchemy".into());
        let can_back = presentation.as_ref().is_some_and(|p| p.can_go_back());
        let hint = presentation
            .as_ref()
            .map(|p| p.primary_action())
            .unwrap_or(PrimaryActionHint::Unavailable);
        let opening_error = self.launcher.opening_error().copied();
        let primary = if opening_error.is_some() {
            "Retry"
        } else {
            match hint {
                PrimaryActionHint::OpenFolder => "Open folder",
                PrimaryActionHint::StartFlow => "Start flow",
                PrimaryActionHint::OpenUrl => "Open URL",
                _ => "Continue",
            }
        };
        let enabled = opening_error.is_some()
            || (!self.launcher.is_opening()
                && !matches!(
                    hint,
                    PrimaryActionHint::Unavailable | PrimaryActionHint::ChooseOption
                ));
        let error = self
            .launcher
            .error()
            .map(|e| e.message().to_owned())
            .or_else(|| opening_error.map(|e| e.to_string()));
        let editor_focus = self.editor.read(cx).focus_handle(cx);
        let owner = cx.weak_entity();
        let list = self.list.clone();
        div()
            .on_children_prepainted(move |bounds, window, cx| {
                use gpui_kit::base::ScrollbarHandle;
                if bounds.len() < 5 {
                    return;
                }
                let rem = window.rem_size();
                let state = list.read(cx);
                let body = if state.delegate().rows.is_empty() {
                    bounds[3].size.height
                } else {
                    state.scroll_handle().content_size().height.min(rem * 24.)
                };
                let fixed = bounds
                    .iter()
                    .enumerate()
                    .filter(|(ix, _)| *ix != 3)
                    .map(|(_, bounds)| bounds.size.height)
                    .fold(Pixels::ZERO, |sum, height| sum + height);
                // p_4 is 1rem on each edge. gap_3 is 0.75rem.
                let height = fixed + body + rem * (2. + 0.75 * (bounds.len() - 1) as f32);
                let owner = owner.clone();
                cx.defer(move |cx| {
                    let _ = owner.update(cx, |view, cx| {
                        if view.measured_height != Some(height) {
                            view.measured_height = Some(height);
                            cx.emit(PaletteEvent::ContentHeight {
                                invocation: view.invocation(),
                                height,
                            });
                        }
                    });
                });
            })
            .id("palette")
            .key_context("URLchemyPalette")
            .capture_key_down(cx.listener(Self::key_down))
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(can_back, |this| {
                        this.child(
                            Button::new("back")
                                .ghost()
                                .label("Back")
                                .track_focus(&self.back_focus)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.dispatch(Cmd::Back, window, cx)
                                })),
                        )
                    })
                    .child(div().flex_1().min_w_0().truncate().child(context))
                    .child(
                        Button::new("close")
                            .ghost()
                            .label("Close")
                            .track_focus(&self.close_focus)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.dispatch(Cmd::Dismiss(CloseReason::Explicit), window, cx)
                            })),
                    ),
            )
            .child(div().id("field-label").test_support().child(label.clone()))
            .child(
                div()
                    .id("launcher-field")
                    .test_support()
                    .track_focus(&editor_focus)
                    .w_full()
                    .child(Textarea::new(&self.editor).aria_label(label).tab_index(0)),
            )
            .child(
                div()
                    .min_h_0()
                    .flex_shrink_1()
                    .max_h(rems(24.))
                    .h_auto()
                    .child(
                        List::new(&self.list)
                            .scroll_bounce(false)
                            .max_h(rems(24.))
                            .h_auto(),
                    ),
            )
            .when_some(error, |this, error| {
                this.child(
                    div()
                        .id("error")
                        .role(Role::Alert)
                        .test_support()
                        .aria_label(error.clone())
                        .text_color(cx.theme().danger)
                        .child(error),
                )
            })
            .child(
                div()
                    .id("palette-footer")
                    .test_support()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} · Confirm", self.launch_label)),
                    )
                    .when(opening_error.is_some(), |this| {
                        this.child(Button::new("start-over").label("Start over").on_click(
                            cx.listener(|this, _, window, cx| {
                                this.dispatch(Cmd::StartOver, window, cx)
                            }),
                        ))
                    })
                    .child(
                        Button::new(if opening_error.is_some() {
                            "retry"
                        } else {
                            "primary"
                        })
                        .primary()
                        .label(primary)
                        .track_focus(&self.primary_focus)
                        .disabled(!enabled)
                        .on_click(cx.listener(|this, _, window, cx| {
                            let command = if this.launcher.opening_error().is_some() {
                                Cmd::Retry
                            } else {
                                Cmd::Confirm
                            };
                            this.dispatch(command, window, cx);
                        })),
                    ),
            )
    }
}
