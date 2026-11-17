use super::{PalettePresentation, PaletteView};
use gpui_kit::{
    component::{ElementExt, tooltip::Tooltip},
    *,
};
use std::rc::Rc;

pub(super) fn context_suffix(
    labels: &[String],
    available: Pixels,
    mut measure: impl FnMut(&str) -> Pixels,
) -> String {
    if labels.is_empty() {
        return "URLchemy".into();
    }
    for start in 0..labels.len() {
        let candidate = if start == 0 {
            labels.join(" / ")
        } else {
            format!("… / {}", labels[start..].join(" / "))
        };
        if measure(&candidate) <= available {
            return candidate;
        }
    }
    labels.last().cloned().unwrap_or_default()
}

#[derive(Default)]
pub(super) struct ContextHelp {
    labels: Vec<String>,
    full: String,
    suffix: String,
    truncated: bool,
    tooltip: Option<AnyView>,
    generation: u64,
    focused: bool,
    keyboard_dismissed: bool,
    hovered: bool,
    hover_dismissed: bool,
}

impl PaletteView {
    pub(super) fn sync_context(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let labels = PalettePresentation::derive(&self.launcher)
            .context()
            .to_vec();
        if self.context_help.labels == labels && self.context_help.tooltip.is_some() {
            return;
        }
        let full = if labels.is_empty() {
            "URLchemy".into()
        } else {
            labels.join(" / ")
        };
        let text = full.clone();
        let tooltip = Tooltip::element(move |_, _| {
            div()
                .id("context-tooltip")
                .test_support()
                .aria_label(text.clone())
                .max_w(rems(24.))
                .child(text.clone())
        })
        .build(window, cx);
        self.context_help = ContextHelp {
            labels,
            suffix: full.clone(),
            full,
            tooltip: Some(tooltip),
            generation: self
                .context_help
                .generation
                .checked_add(1)
                .expect("context generation exhausted"),
            ..Default::default()
        };
    }

    pub(super) fn dismiss_context_help(&mut self, window: &Window, cx: &mut Context<Self>) -> bool {
        let help = &mut self.context_help;
        if help.truncated
            && ((self.back_focus.is_focused(window) && !help.keyboard_dismissed)
                || (help.hovered && !help.hover_dismissed))
        {
            help.keyboard_dismissed = true;
            help.hover_dismissed = true;
            cx.notify();
            return true;
        }
        false
    }

    pub(super) fn render_context(&self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let help = &self.context_help;
        let labels = help.labels.clone();
        let full = help.full.clone();
        let full_label = full.clone();
        let suffix = help.suffix.clone();
        let tooltip = help.tooltip.clone();
        let generation = help.generation;
        let was_focused = help.focused;
        let dismissed = help.keyboard_dismissed;
        let hovered = help.hovered && !help.hover_dismissed;
        let back = self.back_focus.clone();
        let owner = cx.weak_entity();
        div()
            .id("context-summary")
            .test_support()
            .aria_label(full_label)
            .flex_1()
            .min_w_0()
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.context_help.hovered != *hovered {
                    this.context_help.hovered = *hovered;
                    if !hovered {
                        this.context_help.hover_dismissed = false;
                    }
                    cx.notify();
                }
            }))
            .child(
                div()
                    .id("context-visible")
                    .test_support()
                    .aria_label(suffix.clone())
                    .truncate()
                    .child(suffix),
            )
            .on_prepaint(move |bounds, window, cx| {
                let style = window.text_style();
                let rem = window.rem_size();
                let viewport = window.viewport_size();
                let font_size = style.font_size.to_pixels(rem);
                let mut measure = |text: &str| {
                    window
                        .text_system()
                        .shape_line(
                            text.to_owned().into(),
                            font_size,
                            &[style.to_run(text.len())],
                            None,
                        )
                        .width()
                };
                let suffix = context_suffix(&labels, bounds.size.width, &mut measure);
                let truncated = suffix != full || measure(&suffix) > bounds.size.width;
                let focused = back.is_focused(window);
                let keyboard_dismissed = if focused != was_focused {
                    false
                } else {
                    dismissed
                };
                if truncated
                    && ((focused && !keyboard_dismissed) || hovered)
                    && let Some(tooltip) = tooltip.clone()
                {
                    let live = owner.clone();
                    window.set_tooltip(AnyTooltip {
                        view: tooltip,
                        mouse_position: point(bounds.left(), bounds.bottom()),
                        check_visible_and_update: Rc::new(move |_, window, cx| {
                            live.read_with(cx, |view, _| {
                                let help = &view.context_help;
                                help.generation == generation
                                    && help.truncated
                                    && ((view.back_focus.is_focused(window)
                                        && !help.keyboard_dismissed)
                                        || (help.hovered && !help.hover_dismissed))
                            })
                            .unwrap_or(false)
                        }),
                    });
                }
                let handle = window.window_handle();
                let owner = owner.clone();
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_, window, cx| {
                        if window.rem_size() != rem || window.viewport_size() != viewport {
                            return;
                        }
                        let _ = owner.update(cx, |view, cx| {
                            let help = &mut view.context_help;
                            if help.generation != generation {
                                return;
                            }
                            if help.suffix != suffix
                                || help.truncated != truncated
                                || help.focused != focused
                            {
                                help.suffix = suffix;
                                help.truncated = truncated;
                                if help.focused != focused {
                                    help.keyboard_dismissed = false;
                                }
                                help.focused = focused;
                                cx.notify();
                            }
                        });
                    });
                });
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::context_suffix;
    use gpui_kit::px;

    #[test]
    fn context_suffix_drops_old_ancestors_before_the_current_object() {
        let labels = vec!["Workspace".into(), "Tools".into(), "Current".into()];
        let measure = |text: &str| px(text.chars().count() as f32);
        assert_eq!(
            context_suffix(&labels, px(40.), measure),
            "Workspace / Tools / Current"
        );
        assert_eq!(
            context_suffix(&labels, px(20.), measure),
            "… / Tools / Current"
        );
        assert_eq!(context_suffix(&labels, px(12.), measure), "… / Current");
        assert_eq!(context_suffix(&labels, px(3.), measure), "Current");
    }
}
