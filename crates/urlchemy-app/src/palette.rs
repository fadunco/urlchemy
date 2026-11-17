use crate::{CloseReason, Launcher, PaletteCommand as Cmd, PaletteEffect};
use gpui_kit::component::{
    ActiveTheme, Disableable, ElementExt, IndexPath, Sizable,
    button::{Button, ButtonVariants},
    input::{InputEditsEvent, Textarea, TextareaState},
    list::{List, ListDelegate, ListItem, ListState},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};
use urlchemy_core::{EntryId, Row, RowId, SessionSnapshot, StepId};

mod context;
mod layout;
mod ordinal_row;
mod ordinals;
use ordinal_row::{OrdinalList, OrdinalPresentation, OrdinalRow};
mod presentation;
use presentation::{PalettePresentation, PaletteSurface, RowDecoration};

fn row_decorations(launcher: &Launcher) -> BTreeMap<EntryId, RowDecoration> {
    let visible: BTreeSet<_> = launcher
        .rows()
        .iter()
        .filter_map(|row| {
            let RowId::Entry(id) = row.id() else {
                return None;
            };
            Some(id)
        })
        .collect();
    launcher
        .navigation_entries()
        .unwrap_or_default()
        .iter()
        .filter(|entry| visible.contains(&entry.id()))
        .map(|entry| (entry.id(), RowDecoration::from_entry(entry)))
        .collect()
}

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
    _edits: Option<Subscription>,
    list: Entity<ListState<PaletteListDelegate>>,
    scope: Scope,
    generation: u64,
    revision: u64,
    launch: Option<KeybindingKeystroke>,
    launch_label: String,
    primary_focus: FocusHandle,
    back_focus: FocusHandle,
    close_focus: FocusHandle,
    start_over_focus: FocusHandle,
    status_focus: FocusHandle,
    context_help: context::ContextHelp,
    ordinals: Rc<RefCell<OrdinalPresentation>>,
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
        for key in ["enter", "up", "down", "escape", "cmd-[", "tab", "shift-tab"]
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
        for key in ["escape", "cmd-[", "tab", "shift-tab"] {
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
        let ordinals = Rc::new(RefCell::new(OrdinalPresentation::default()));
        let decorations = row_decorations(&launcher);
        let empty_message = PalettePresentation::derive(&launcher)
            .body_message()
            .unwrap_or("No matches")
            .to_owned();
        let list = cx.new(|cx| {
            let mut list = ListState::new(
                PaletteListDelegate {
                    owner,
                    rows,
                    selected,
                    revision: 0,
                    decorations,
                    empty_message,
                    ordinals: ordinals.clone(),
                },
                window,
                cx,
            )
            .searchable(false);
            list.set_selected_index(selected, window, cx);
            list
        });
        ordinals.borrow_mut().scroll = list.read(cx).scroll_handle().clone();
        let current = launcher
            .snapshot()
            .map(|snapshot| scope(&snapshot))
            .unwrap_or(Scope::Other);
        let mut view = Self {
            launcher,
            editor,
            _edits: Some(edits),
            list,
            scope: current,
            generation: 1,
            revision: 0,
            launch,
            launch_label,
            primary_focus: cx.focus_handle().tab_index(1),
            back_focus: cx.focus_handle().tab_index(2),
            close_focus: cx.focus_handle().tab_index(3),
            start_over_focus: cx.focus_handle().tab_index(2),
            status_focus: cx.focus_handle(),
            context_help: context::ContextHelp::default(),
            ordinals,
            measured_height: None,
        };
        view.sync_context(window, cx);
        view
    }
    /// The actual editor entity, shared with focus and input-handler integrations.
    pub fn editor(&self) -> Entity<TextareaState> {
        self.editor.clone()
    }
    pub fn invocation(&self) -> u64 {
        self.launcher.invocation()
    }
    pub fn dispatch(&mut self, command: Cmd, window: &mut Window, cx: &mut Context<Self>) {
        self.ordinals.borrow_mut().displayed = None;
        let back = matches!(command, Cmd::Back);
        let refocus = !matches!(command, Cmd::OpenFinished { .. } | Cmd::Dismiss(_));
        let effect = self.launcher.dispatch(command);
        let current = self
            .launcher
            .snapshot()
            .map(|snapshot| scope(&snapshot))
            .unwrap_or(Scope::Other);
        let editable = PalettePresentation::derive(&self.launcher)
            .field_label()
            .is_some();
        if !editable {
            self.scope = current;
            self.generation = self
                .generation
                .checked_add(1)
                .expect("editor generation exhausted");
            self._edits = None;
        } else if current != self.scope || self._edits.is_none() {
            self.scope = current;
            self.generation = self
                .generation
                .checked_add(1)
                .expect("editor generation exhausted");
            let (editor, edits) =
                Self::make_editor(self.launcher.query(), self.generation, window, cx);
            self.editor = editor;
            self._edits = Some(edits);
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
        self.ordinals.borrow_mut().revision = self.revision;
        let rows = self.launcher.rows().to_vec();
        let decorations = row_decorations(&self.launcher);
        let empty_message = PalettePresentation::derive(&self.launcher)
            .body_message()
            .unwrap_or("No matches")
            .to_owned();
        let selected = self
            .launcher
            .active()
            .and_then(|id| rows.iter().position(|row| row.id() == id))
            .map(IndexPath::new);
        self.list.update(cx, |list, cx| {
            list.delegate_mut().rows = rows;
            list.delegate_mut().revision = self.revision;
            list.delegate_mut().decorations = decorations;
            list.delegate_mut().empty_message = empty_message;
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
        } else if self.launcher.is_opening() {
            self.status_focus.focus(window, cx);
        } else if refocus
            && matches!(
                self.scope,
                Scope::Input(_) | Scope::Browse(_) | Scope::Choice(_)
            )
        {
            self.editor.update(cx, |input, cx| input.focus(window, cx));
        }
        self.sync_context(window, cx);
        cx.notify();
        if let Some(effect) = effect {
            cx.emit(PaletteEvent::Effect(effect));
        }
    }
    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = &event.keystroke;
        let plain = key.modifiers == Modifiers::default();
        let editor_focused =
            self._edits.is_some() && self.editor.read(cx).focus_handle(cx).is_focused(window);
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
        let tab = key.key == "tab"
            && !key.modifiers.control
            && !key.modifiers.alt
            && !key.modifiers.platform;
        let ordinal = if matches!(self.scope, Scope::Browse(_))
            && key.modifiers.platform
            && !key.modifiers.control
            && !key.modifiers.alt
            && !key.modifiers.shift
            && key.key.len() == 1
        {
            key.key
                .parse::<u8>()
                .ok()
                .filter(|number| (1..=9).contains(number))
        } else {
            None
        };
        if !launch
            && !escape
            && !back
            && !arrow
            && !tab
            && ordinal.is_none()
            && !(editor_focused && plain && key.key == "enter")
        {
            return;
        }
        cx.stop_propagation();
        if event.is_held && (launch || escape || back || ordinal.is_some()) {
            return;
        }
        if escape && self.dismiss_context_help(window, cx) {
            return;
        }
        let composing = editor_focused
            && self.editor.update(cx, |input, cx| {
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
        if tab {
            if key.modifiers.shift {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            return;
        }
        if let Some(number) = ordinal {
            if self.editor.read(cx).value().as_ref() != self.launcher.query() {
                return;
            }
            let id = {
                let p = self.ordinals.borrow();
                p.displayed
                    .as_ref()
                    .and_then(|map| map.resolve(number, &p.geometry(window)))
            };
            if let Some(id) = id
                && self
                    .launcher
                    .rows()
                    .iter()
                    .any(|row| row.id() == RowId::Entry(id))
            {
                self.dispatch(Cmd::Activate(RowId::Entry(id)), window, cx);
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
    decorations: BTreeMap<EntryId, RowDecoration>,
    empty_message: String,
    ordinals: Rc<RefCell<OrdinalPresentation>>,
}
impl ListDelegate for PaletteListDelegate {
    type Item = OrdinalRow;
    fn items_count(&self, _: usize, _: &App) -> usize {
        self.rows.len()
    }
    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<OrdinalRow> {
        let row = self.rows.get(ix.row)?;
        let decoration = match row.id() {
            RowId::Entry(id) => self.decorations.get(&id),
            _ => None,
        };
        let alias = decoration.and_then(|d| d.alias()).map(str::to_owned);
        let icon = div()
            .id("row-icon")
            .test_support()
            .w_5()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center();
        let icon = if let Some(emoji) = decoration.and_then(|d| d.emoji()).or(row.emoji()) {
            icon.aria_label(emoji.to_owned())
                .child(emoji.to_owned())
                .into_any_element()
        } else if let Some(decoration) = decoration {
            let name = if decoration.is_folder() {
                gpui_kit::assets::IconName::Folder
            } else {
                gpui_kit::assets::IconName::File
            };
            icon.child(gpui_kit::component::Icon::new(name).small())
                .into_any_element()
        } else {
            icon.into_any_element()
        };
        let sample = match row.id() {
            RowId::Entry(id) => Some((ix.row, id)),
            _ => None,
        };
        Some(OrdinalRow::new(
            ListItem::new(SharedString::from(format!("row:{:?}", row.id())))
                .accessibility_label(row.label().to_owned())
                .selected(self.selected == Some(ix))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .w_full()
                        .child(icon)
                        .child(
                            div()
                                .id("row-label")
                                .test_support()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .child(row.label().to_owned()),
                        )
                        .child(div().w_20().flex_shrink_0().text_sm().when_some(
                            alias,
                            |this, alias| {
                                this.child(
                                    div()
                                        .id("row-alias")
                                        .test_support()
                                        .aria_label(format!("Alias: {alias}"))
                                        .truncate()
                                        .child(alias),
                                )
                            },
                        )),
                ),
            sample,
            self.ordinals.clone(),
        ))
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
        div().child(self.empty_message.clone())
    }
}

impl PaletteView {
    fn render_header(
        &self,
        p: &PalettePresentation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .when(p.can_go_back(), |this| {
                this.child(
                    Button::new("back")
                        .ghost()
                        .label("Back")
                        .track_focus(&self.back_focus)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.dispatch(Cmd::Back, window, cx)),
                        ),
                )
            })
            .child(self.render_context(window, cx))
            .child(
                Button::new("close")
                    .ghost()
                    .label("Close")
                    .track_focus(&self.close_focus)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.dispatch(Cmd::Dismiss(CloseReason::Explicit), window, cx)
                    })),
            )
            .into_any_element()
    }

    fn render_question(
        &self,
        p: &PalettePresentation,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let label = p.field_label().unwrap_or_default().to_owned();
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().id("field-label").test_support().child(label.clone()))
            .child(
                div()
                    .id("launcher-field")
                    .test_support()
                    .w_full()
                    .child(Textarea::new(&self.editor).aria_label(label).tab_index(0)),
            )
            .when_some(p.error_message(), |this, error| {
                this.child(
                    div()
                        .id("error")
                        .role(Role::Alert)
                        .test_support()
                        .aria_label(error.to_owned())
                        .text_color(cx.theme().danger)
                        .child(error.to_owned()),
                )
            })
            .into_any_element()
    }

    fn render_body(
        &self,
        p: &PalettePresentation,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if p.field_label().is_some() && !self.launcher.rows().is_empty() {
            return OrdinalList::new(
                List::new(&self.list).scroll_bounce(false).h_full(),
                self.ordinals.clone(),
            )
            .into_any_element();
        }
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .when_some(p.body_message(), |this, message| {
                this.child(
                    div()
                        .id("body-message")
                        .role(Role::Status)
                        .test_support()
                        .aria_label(message.to_owned())
                        .child(message.to_owned()),
                )
            })
            .when(p.field_label().is_none(), |this| {
                this.when_some(p.error_message(), |this, error| {
                    this.child(
                        div()
                            .id("error")
                            .role(Role::Alert)
                            .test_support()
                            .aria_label(error.to_owned())
                            .text_color(cx.theme().danger)
                            .child(error.to_owned()),
                    )
                })
            })
            .into_any_element()
    }

    fn render_footer(
        &self,
        p: &PalettePresentation,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let failed = p.surface() == PaletteSurface::OpeningFailed;
        let hint = p.commitment_hint().map(str::to_owned).unwrap_or_else(|| {
            if p.field_label().is_some() {
                format!("{} · Confirm", self.launch_label)
            } else if p.can_go_back() {
                "Command+[ · Back".into()
            } else {
                "Escape · Close".into()
            }
        });
        div()
            .id("palette-footer")
            .test_support()
            .w_full()
            .min_h(rems(2.))
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .id("commitment-hint")
                    .role(Role::Status)
                    .test_support()
                    .aria_label(hint.clone())
                    .flex_1()
                    .min_w_0()
                    .text_color(cx.theme().muted_foreground)
                    .child(hint),
            )
            .when(failed, |this| {
                this.child(
                    Button::new("start-over")
                        .label("Start over")
                        .track_focus(&self.start_over_focus)
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.dispatch(Cmd::StartOver, window, cx)
                        })),
                )
            })
            .when_some(p.primary_label(), |this, label| {
                this.child(
                    Button::new(if failed { "retry" } else { "primary" })
                        .primary()
                        .label(label.to_owned())
                        .max_w(relative(0.6))
                        .track_focus(&self.primary_focus)
                        .disabled(!p.primary_enabled())
                        .on_click(cx.listener(|this, _, window, cx| {
                            let command = if this.launcher.opening_error().is_some() {
                                Cmd::Retry
                            } else {
                                Cmd::Confirm
                            };
                            this.dispatch(command, window, cx);
                        })),
                )
            })
            .into_any_element()
    }
}

impl Render for PaletteView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = PalettePresentation::derive(&self.launcher);
        let question = p.field_label().is_some();
        let heights = Rc::new(RefCell::new(layout::RegionHeights::default()));
        let header_heights = heights.clone();
        let question_heights = heights.clone();
        let body_heights = heights.clone();
        let footer_heights = heights.clone();
        let owner = cx.weak_entity();
        let list = self.list.clone();
        let ordinals = self.ordinals.clone();
        let revision = self.revision;
        div()
            .on_children_prepainted(move |_, window, cx| {
                let m = heights.borrow();
                let (Some(header), Some(mut body), Some(footer)) = (m.header, m.body, m.footer)
                else {
                    return;
                };
                let question_height = if question {
                    let Some(height) = m.question else { return };
                    Some(height)
                } else {
                    None
                };
                if question
                    && !list.read(cx).delegate().rows.is_empty()
                    && let Some(row_height) = ordinals.borrow().row_height
                {
                    body = row_height * list.read(cx).delegate().rows.len() as f32;
                }
                let rem = window.rem_size();
                let width = window.viewport_size().width;
                let height = layout::preferred_height(header, question_height, body, footer, rem);
                let handle = window.window_handle();
                let owner = owner.clone();
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_, window, cx| {
                        if window.rem_size() != rem || window.viewport_size().width != width {
                            return;
                        }
                        let _ = owner.update(cx, |view, cx| {
                            if view.revision == revision && view.measured_height != Some(height) {
                                view.measured_height = Some(height);
                                cx.emit(PaletteEvent::ContentHeight {
                                    invocation: view.invocation(),
                                    height,
                                });
                            }
                        });
                    });
                });
            })
            .id("palette")
            .key_context("URLchemyPalette")
            .track_focus(&self.status_focus)
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
                    .w_full()
                    .flex_shrink_0()
                    .child(self.render_header(&p, window, cx))
                    .on_prepaint(move |bounds, _, _| {
                        header_heights.borrow_mut().header = Some(bounds.size.height)
                    }),
            )
            .when(question, |this| {
                this.child(
                    div()
                        .w_full()
                        .flex_shrink_0()
                        .child(self.render_question(&p, window, cx))
                        .on_prepaint(move |bounds, _, _| {
                            question_heights.borrow_mut().question = Some(bounds.size.height)
                        }),
                )
            })
            .child(
                div()
                    .on_children_prepainted(move |bounds, _, _| {
                        body_heights.borrow_mut().body =
                            bounds.first().map(|bounds| bounds.size.height);
                    })
                    .id("palette-body")
                    .test_support()
                    .w_full()
                    .min_h_0()
                    .flex_shrink_1()
                    .max_h(rems(24.))
                    .h_auto()
                    .when(!question || self.launcher.rows().is_empty(), |this| {
                        this.overflow_y_scroll()
                    })
                    .when(question && !self.launcher.rows().is_empty(), |this| {
                        this.h(rems(24.)).overflow_hidden()
                    })
                    .child(self.render_body(&p, window, cx)),
            )
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .child(self.render_footer(&p, window, cx))
                    .on_prepaint(move |bounds, _, _| {
                        footer_heights.borrow_mut().footer = Some(bounds.size.height)
                    }),
            )
    }
}
