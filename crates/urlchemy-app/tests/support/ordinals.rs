use super::*;
use gpui_kit::{InputEvent, KeyDownEvent, Keystroke, Pixels, ScrollEvent, point};

fn raw_key(w: &mut Window, cx: &mut App, key: &'static str) {
    w.dispatch_event(
        KeyDownEvent {
            keystroke: Keystroke::parse(key).unwrap(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
}

fn flows() -> String {
    let mut source = "navigation:\n".to_owned();
    for ix in 0..40 {
        source.push_str(&format!("  - flow:\n      id: flow-{ix}\n      label: Flow {ix:02}\n      alias: f{ix}\n      baseURL: https://example.com\n      start:\n        destination:\n          template: /{ix}\n"));
    }
    source
}

fn check_hints(cx: &mut TestAppContext, h: &Harness) {
    ui(cx, h, |w, _| {
        let viewport = w.find("palette-body").bounds();
        let rows = w.find_all("ordinal-row");
        let mut eligible = 0;
        for row in rows {
            let bounds = row.bounds();
            let full = bounds.size.height > px(0.) && bounds.intersect(&viewport) == bounds;
            let hints: Vec<_> = w
                .find_all("ordinal-hint")
                .into_iter()
                .filter(|hint| hint.bounds().intersect(&bounds) == hint.bounds())
                .collect();
            if full && eligible < 9 {
                eligible += 1;
                assert_eq!(hints.len(), 1, "missing hint for {bounds:?}");
                assert_eq!(hints[0].label(), Some(format!("⌘{eligible}").as_str()));
                assert!(
                    gpui_kit::base::test_support::texts(w)
                        .iter()
                        .any(|text| text.text() == format!("⌘{eligible}")),
                    "hint metadata must match painted text"
                );
            } else {
                assert!(
                    hints.is_empty(),
                    "hint on partial or excess row: bounds={bounds:?}, viewport={viewport:?}, eligible={eligible}, hints={hints:?}"
                );
            }
        }
        assert!(eligible > 0, "no complete rows");
    });
}

#[gpui_kit::test]
fn navigation_hints_follow_complete_rows(cx: &mut TestAppContext) {
    let h = setup_source(cx, &flows());
    check_hints(cx, &h);
}

#[gpui_kit::test]
fn native_scroll_fixture_activates_the_indicated_folder(cx: &mut TestAppContext) {
    let h = setup_source(cx, include_str!("../fixtures/native_scroll.yaml"));
    check_hints(cx, &h);
    text(cx, &h, "Folder 2");
    check_hints(cx, &h);
    key(cx, &h, "cmd-1");
    ui(cx, &h, |w, _| {
        assert_eq!(w.find("context-summary").label(), Some("Folder 20"));
        assert_eq!(
            w.find("body-message").label(),
            Some("No entries in this folder")
        );
        assert!(w.find_all("ordinal-hint").is_empty());
    });
    assert!(h.effects.borrow().is_empty());
    key(cx, &h, "cmd-[");
    check_hints(cx, &h);
}

#[gpui_kit::test]
fn hints_renumber_after_filter_scroll_and_resize(cx: &mut TestAppContext) {
    let h = setup_source(cx, &flows());
    text(cx, &h, "Flow 1");
    check_hints(cx, &h);
    ui(cx, &h, |w, cx| {
        w.scroll(
            "palette-body",
            gpui_kit::ScrollDelta::Pixels(point(px(0.), px(-37.))),
            cx,
        )
    });
    check_hints(cx, &h);
    cx.simulate_window_resize(h.window, size(px(360.), px(300.)));
    settle(cx, &h);
    check_hints(cx, &h);
}

#[gpui_kit::test]
fn shortcut_lane_preserves_row_alignment(cx: &mut TestAppContext) {
    for font in [14., 20.] {
        for width in [360., 640.] {
            let h = setup_source(cx, &flows());
            cx.update(|cx| {
                gpui_kit::component::Theme::update(cx, |theme| theme.font_size = px(font))
            });
            cx.simulate_window_resize(h.window, size(px(width), px(480.)));
            settle(cx, &h);
            check_hints(cx, &h);
            ui(cx, &h, |w, _| {
                for id in ["row-icon", "row-label", "row-alias", "ordinal-slot"] {
                    let elements = w.find_all(id);
                    assert!(elements.len() > 1);
                    let x = elements[0].bounds().left();
                    assert!(elements.iter().all(|e| e.bounds().left() == x), "{id}");
                    assert!(
                        elements
                            .iter()
                            .all(|e| e.bounds().size.width > Pixels::ZERO),
                        "{id}"
                    );
                }
            });
        }
    }
}

#[gpui_kit::test]
fn scrolling_can_reach_the_last_complete_row(cx: &mut TestAppContext) {
    let h = setup_source(cx, &flows());
    cx.simulate_window_resize(h.window, size(px(360.), px(300.)));
    settle(cx, &h);
    ui(cx, &h, |w, cx| {
        w.scroll(
            "palette-body",
            gpui_kit::ScrollDelta::Pixels(point(px(0.), px(-10000.))),
            cx,
        )
    });
    ui(cx, &h, |w, _| {
        let last = w.find_by_label("Flow 39").bounds();
        let viewport = w.find("palette-body").bounds();
        assert_eq!(last.intersect(&viewport), last);
    });
    check_hints(cx, &h);
}

fn hinted_flow(cx: &mut TestAppContext, h: &Harness, number: usize) -> usize {
    cx.update_window(h.window, |_, w, _| {
        let hint = w.find_by_label(&format!("⌘{number}")).bounds();
        (0..40)
            .find(|ix| {
                w.try_find_by_label(&format!("Flow {ix:02}"))
                    .is_some_and(|row| row.bounds().intersect(&hint) == hint)
            })
            .unwrap()
    })
    .unwrap()
}

#[gpui_kit::test]
fn ordinal_activation_matches_the_displayed_hint(cx: &mut TestAppContext) {
    for number in 1..=9 {
        let h = setup_source(cx, &flows());
        cx.simulate_window_resize(h.window, size(px(640.), px(900.)));
        settle(cx, &h);
        let expected = hinted_flow(cx, &h, number);
        key(cx, &h, &format!("cmd-{number}"));
        assert_eq!(opened(&h), format!("https://example.com/{expected}"));
    }
    let h = setup_source(cx, &flows());
    text(cx, &h, "Flow 2");
    key(cx, &h, "cmd-1");
    assert_eq!(opened(&h), "https://example.com/20");
    let h = setup_source(cx, include_str!("../fixtures/nested_navigation.yaml"));
    for _ in 0..3 {
        key(cx, &h, "cmd-1");
    }
    assert_eq!(
        opened(&h),
        "https://status.example.com/company/overview?view=summary"
    );
}

#[gpui_kit::test]
fn missing_hint_never_activates(cx: &mut TestAppContext) {
    let h = setup_source(cx, &flows());
    cx.simulate_window_resize(h.window, size(px(360.), px(300.)));
    settle(cx, &h);
    ui(cx, &h, |w, _| assert!(w.try_find_by_label("⌘9").is_none()));
    key(cx, &h, "cmd-9");
    assert!(h.effects.borrow().is_empty());
    text(cx, &h, "Flow 39");
    key(cx, &h, "cmd-2");
    assert!(h.effects.borrow().is_empty());
    text(cx, &h, "absent");
    key(cx, &h, "cmd-1");
    assert!(h.effects.borrow().is_empty());
}

#[gpui_kit::test]
fn scroll_then_shortcut_rejects_the_previous_viewport(cx: &mut TestAppContext) {
    let h = setup_source(cx, &flows());
    ui(cx, &h, |w, cx| {
        let position = w.find("palette-body").bounds().center();
        w.pointer_move(position, None, cx);
        w.dispatch_event(
            ScrollEvent {
                position,
                delta: gpui_kit::ScrollDelta::Pixels(point(px(0.), px(-150.))),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        raw_key(w, cx, "cmd-1");
    });
    // GPUI redraws dirty content before dispatching a key. The accepted ID
    // must belong to the new viewport, never the previously numbered Flow 00.
    let url = opened(&h);
    assert_ne!(url, "https://example.com/0");
    assert_eq!(url, "https://example.com/5");
}

#[gpui_kit::test]
fn queued_query_or_context_change_rejects_old_ordinals(cx: &mut TestAppContext) {
    let h = setup_source(cx, &flows());
    ui(cx, &h, |w, cx| {
        w.input("Flow 2", cx);
        raw_key(w, cx, "cmd-1");
    });
    assert!(h.effects.borrow().is_empty());
    key(cx, &h, "cmd-1");
    assert_eq!(opened(&h), "https://example.com/20");
    let h = setup_source(cx, include_str!("../fixtures/nested_navigation.yaml"));
    ui(cx, &h, |w, cx| {
        raw_key(w, cx, "enter");
        raw_key(w, cx, "cmd-1");
    });
    assert_eq!(value(cx, &h, "Open a flow"), "");
    assert!(h.effects.borrow().is_empty());
    ui(cx, &h, |w, _| {
        assert!(w.find_by_label("Service status").visible())
    });
    ui(cx, &h, |w, cx| {
        raw_key(w, cx, "cmd-[");
        raw_key(w, cx, "cmd-1");
    });
    assert!(h.effects.borrow().is_empty());
    ui(cx, &h, |w, _| {
        assert!(w.find_by_label("Service status").visible())
    });
    key(cx, &h, "cmd-1");
    assert_eq!(
        opened(&h),
        "https://status.example.com/company/overview?view=summary"
    );
}

#[gpui_kit::test]
fn keyboard_scroll_and_rem_changes_refresh_ordinals(cx: &mut TestAppContext) {
    let h = setup_source(cx, &flows());
    for _ in 0..20 {
        key(cx, &h, "down");
    }
    check_hints(cx, &h);
    assert!(hinted_flow(cx, &h, 1) > 0);
    cx.update(|cx| gpui_kit::component::Theme::update(cx, |theme| theme.font_size = px(20.)));
    settle(cx, &h);
    check_hints(cx, &h);
    let expected = hinted_flow(cx, &h, 1);
    key(cx, &h, "cmd-1");
    assert_eq!(opened(&h), format!("https://example.com/{expected}"));
}

#[gpui_kit::test]
fn ordinal_keys_do_not_cross_input_or_composition_boundaries(cx: &mut TestAppContext) {
    let h = setup_source(cx, &flows());
    text(cx, &h, "1");
    assert_eq!(value(cx, &h, "Open a flow"), "1");
    for chord in ["cmd-shift-1", "cmd-alt-1", "ctrl-cmd-1"] {
        key(cx, &h, chord);
    }
    assert!(h.effects.borrow().is_empty());
    let h = setup_source(cx, include_str!("../fixtures/nested_navigation.yaml"));
    key(cx, &h, "cmd-1");
    ui(cx, &h, |w, cx| w.key_down("cmd-1", true, cx));
    assert!(h.effects.borrow().is_empty());
    key(cx, &h, "cmd-1");
    assert!(h.effects.borrow().is_empty());
    key(cx, &h, "cmd-1");
    assert_eq!(
        opened(&h),
        "https://status.example.com/company/overview?view=summary"
    );
    let h = setup_source(cx, &flows());
    ui(cx, &h, |w, cx| {
        let editor = h.view.read(cx).editor();
        let bounds = w.find("launcher-field").bounds();
        TestInput::new(ElementInputHandler::new(bounds, editor), w).compose_text(
            None,
            "Flow",
            Some(4..4),
            cx,
        );
    });
    key(cx, &h, "cmd-1");
    assert!(h.effects.borrow().is_empty());
    for file in ["linear.yaml", "branching.yaml"] {
        let h = setup(cx, file, "");
        key(cx, &h, "enter");
        if file == "branching.yaml" {
            key(cx, &h, "enter");
        }
        ui(cx, &h, |w, _| {
            assert!(w.find_all("ordinal-hint").is_empty())
        });
        key(cx, &h, "cmd-1");
        assert!(h.effects.borrow().is_empty());
        let label = if file == "linear.yaml" {
            "Section"
        } else {
            "Dashboard"
        };
        assert_eq!(value(cx, &h, label), "");
    }
}
