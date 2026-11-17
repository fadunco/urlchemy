use super::*;
use gpui_kit::Focusable;

const NESTED: &str = include_str!("../fixtures/nested_navigation.yaml");
const FOUR: &str = include_str!("../fixtures/four_inputs.yaml");
const COMMON: &str = include_str!("../fixtures/common_review.yaml");

#[gpui_kit::test]
fn fixed_destinations_complete_without_an_input_screen(cx: &mut TestAppContext) {
    let h = setup(cx, "fixed.yaml", "");
    key(cx, &h, "enter");
    key(cx, &h, "enter");
    assert_eq!(
        opened(&h),
        "https://status.example.com/company/overview?view=summary"
    );
    for (option, expected) in [
        ("Overview", "https://status.example.com/company/overview"),
        ("Incidents", "https://status.example.com/company/incidents"),
    ] {
        let h = setup(cx, "fixed.yaml", "");
        click_label(cx, &h, "Choose status page");
        click_label(cx, &h, option);
        assert_eq!(opened(&h), expected);
    }
}

#[gpui_kit::test]
fn nested_and_direct_aliases_keep_their_scope(cx: &mut TestAppContext) {
    let h = setup_source(cx, NESTED);
    text(cx, &h, "status");
    assert!(h.effects.borrow().is_empty());
    assert_eq!(value(cx, &h, "Open a flow"), "status");
    key(cx, &h, "cmd-a");
    text(cx, &h, "ws");
    assert_eq!(value(cx, &h, "Open a flow"), "");
    text(cx, &h, "tools");
    assert_eq!(value(cx, &h, "Open a flow"), "");
    text(cx, &h, "status");
    assert_eq!(
        opened(&h),
        "https://status.example.com/company/overview?view=summary"
    );
    let h = setup_source(cx, NESTED);
    text(cx, &h, "service-status");
    assert_eq!(
        opened(&h),
        "https://status.example.com/company/overview?view=summary"
    );
    let h = setup(cx, "linear.yaml", "");
    key(cx, &h, "enter");
    text(cx, &h, "open-dd");
    assert_eq!(value(cx, &h, "Section"), "open-dd");
    assert!(h.effects.borrow().is_empty());
}

#[gpui_kit::test]
fn branching_journeys_use_labels_without_substituting_literals(cx: &mut TestAppContext) {
    for (select, expected) in [
        (
            true,
            "https://app.datadog.com/company/8c21-http?from=accounts&to=billing",
        ),
        (
            false,
            "https://app.datadog.com/company/8c21-http?from=acc&to=billing",
        ),
    ] {
        let h = setup(cx, "branching.yaml", "");
        text(cx, &h, "open-dd");
        click_label(cx, &h, "HTTP Traffic");
        ui(cx, &h, |w, _| {
            assert!(w.try_find_by_label("User identifier").is_none())
        });
        text(cx, &h, "acc");
        if select {
            click_label(cx, &h, "Accounts");
            assert_eq!(value(cx, &h, "Source application"), "acc");
        }
        ui(cx, &h, |w, cx| w.click("primary", cx));
        text(cx, &h, "billing");
        key(cx, &h, "enter");
        assert_eq!(opened(&h), expected);
    }
    let h = setup(cx, "branching.yaml", "");
    text(cx, &h, "open-dd");
    key(cx, &h, "down");
    key(cx, &h, "enter");
    ui(cx, &h, |w, _| {
        assert!(w.try_find_by_label("Source application").is_none());
        assert!(w.try_find_by_label("Destination application").is_none());
    });
    text(cx, &h, "alice");
    key(cx, &h, "enter");
    assert_eq!(
        opened(&h),
        "https://app.datadog.com/company/9d32-users?userID=alice"
    );
}

#[gpui_kit::test]
fn back_and_branch_change_discard_only_invalidated_answers(cx: &mut TestAppContext) {
    let h = setup(cx, "branching.yaml", "");
    text(cx, &h, "open-dd");
    click_label(cx, &h, "HTTP Traffic");
    click_label(cx, &h, "Accounts");
    key(cx, &h, "enter");
    text(cx, &h, "billing");
    key(cx, &h, "cmd-[");
    assert_eq!(value(cx, &h, "Source application"), "accounts");
    text(cx, &h, "accounts");
    assert_eq!(value(cx, &h, "Source application"), "accounts");
    key(cx, &h, "enter");
    assert_eq!(value(cx, &h, "Destination application"), "");
    key(cx, &h, "cmd-[");
    key(cx, &h, "cmd-[");
    click_label(cx, &h, "User Activity");
    assert_eq!(value(cx, &h, "User identifier"), "");
    text(cx, &h, "alice");
    key(cx, &h, "enter");
    assert_eq!(
        opened(&h),
        "https://app.datadog.com/company/9d32-users?userID=alice"
    );
}

#[gpui_kit::test]
fn unchanged_answers_preserve_the_committed_suffix(cx: &mut TestAppContext) {
    for changed in [false, true] {
        let h = setup_source(cx, FOUR);
        key(cx, &h, "enter");
        for answer in ["one", "two", "three"] {
            text(cx, &h, answer);
            key(cx, &h, "enter");
        }
        text(cx, &h, "unfinished");
        key(cx, &h, "cmd-[");
        key(cx, &h, "cmd-[");
        assert_eq!(value(cx, &h, "Y"), "two");
        if changed {
            text(cx, &h, "changed");
        }
        key(cx, &h, "enter");
        assert_eq!(value(cx, &h, "Z"), if changed { "" } else { "three" });
        if changed {
            text(cx, &h, "new-three");
        }
        key(cx, &h, "enter");
        assert_eq!(value(cx, &h, "W"), "");
        text(cx, &h, "four");
        key(cx, &h, "enter");
        assert_eq!(
            opened(&h),
            if changed {
                "https://review.example.com/x/one/y/changed/z/new-three/w/four"
            } else {
                "https://review.example.com/x/one/y/two/z/three/w/four"
            }
        );
    }
}

#[gpui_kit::test]
fn common_choice_changes_invalidate_the_shared_suffix(cx: &mut TestAppContext) {
    for changed in [false, true] {
        let h = setup_source(cx, COMMON);
        key(cx, &h, "enter");
        text(cx, &h, "Stag");
        key(cx, &h, "enter");
        text(cx, &h, "accounts");
        key(cx, &h, "enter");
        text(cx, &h, "unfinished");
        key(cx, &h, "cmd-[");
        key(cx, &h, "cmd-[");
        assert_eq!(value(cx, &h, "Environment"), "");
        ui(cx, &h, |w, _| {
            assert!(w.find_by_label("Staging").visible());
            assert!(w.find_by_label("Production").visible());
        });
        if changed {
            click_label(cx, &h, "Production");
        } else {
            key(cx, &h, "enter");
        }
        assert_eq!(
            value(cx, &h, "Application"),
            if changed { "" } else { "accounts" }
        );
        if changed {
            text(cx, &h, "billing");
        }
        key(cx, &h, "enter");
        assert_eq!(value(cx, &h, "Review"), "");
        text(cx, &h, "checked");
        key(cx, &h, "enter");
        assert_eq!(
            opened(&h),
            if changed {
                "https://review.example.com/production/application/billing?review=checked"
            } else {
                "https://review.example.com/staging/application/accounts?review=checked"
            }
        );
    }
}

#[gpui_kit::test]
fn view_preserves_context_specific_url_text(cx: &mut TestAppContext) {
    for (answer, expected) in [
        (
            "café/ops & SRE",
            "https://idp.example.com/company/engineering/application/caf%C3%A9%2Fops%20%26%20SRE/jobs?owner=caf%C3%A9%2Fops%20%26%20SRE&owner=backup",
        ),
        (
            " accounts ",
            "https://idp.example.com/company/engineering/application/%20accounts%20/jobs?owner=%20accounts%20&owner=backup",
        ),
        (
            "%2F",
            "https://idp.example.com/company/engineering/application/%252F/jobs?owner=%252F&owner=backup",
        ),
    ] {
        let h = setup(cx, "linear.yaml", "");
        key(cx, &h, "enter");
        text(cx, &h, "jobs");
        key(cx, &h, "enter");
        text(cx, &h, answer);
        key(cx, &h, "enter");
        assert_eq!(opened(&h), expected);
    }
}

#[gpui_kit::test]
fn navigation_rows_show_configured_metadata(cx: &mut TestAppContext) {
    let source = format!(
        "{NESTED}\n  - flow:\n      id: home\n      label: Home\n      baseURL: https://example.com\n      start:\n        destination:\n          template: ''\n"
    );
    let h = setup_source(cx, &source);
    ui(cx, &h, |w, _| {
        assert!(w.find_by_label("🗂").visible());
        assert!(w.find_by_label("Alias: ws").visible());
        let icons = w.find_all("row-icon");
        let labels = w.find_all("row-label");
        assert_eq!(icons.len(), 2);
        assert_eq!(labels.len(), 2);
        assert_eq!(icons[0].bounds().size.width, icons[1].bounds().size.width);
        assert_eq!(labels[0].bounds().left(), labels[1].bounds().left());
        assert!(labels[0].bounds().size.width > px(0.));
    });
    click_label(cx, &h, "Workspace");
    ui(cx, &h, |w, _| {
        assert!(w.find_by_label("🧰").visible());
        assert!(w.find_by_label("Alias: tools").visible());
    });
    click_label(cx, &h, "Tools");
    ui(cx, &h, |w, _| {
        assert!(w.find_by_label("🟢").visible());
        assert!(w.find_by_label("Alias: status").visible());
    });
    click_label(cx, &h, "Service status");
    assert_eq!(
        opened(&h),
        "https://status.example.com/company/overview?view=summary"
    );
}

#[gpui_kit::test]
fn footer_tracks_explicit_selection_and_literal_draft(cx: &mut TestAppContext) {
    for binding in ["enter", "cmd-enter"] {
        let h = setup(
            cx,
            "branching.yaml",
            &format!("shortcuts: {{launch: {binding}}}\n"),
        );
        text(cx, &h, "open-dd");
        click_label(cx, &h, "HTTP Traffic");
        text(cx, &h, "acc");
        click_label(cx, &h, "Accounts");
        assert_eq!(value(cx, &h, "Source application"), "acc");
        ui(cx, &h, |w, _| {
            assert_eq!(w.find("primary").label(), Some("Use Accounts"))
        });
        key(cx, &h, "cmd-right");
        text(cx, &h, "x");
        ui(cx, &h, |w, _| {
            assert_eq!(w.find("primary").label(), Some("Use “accx”"))
        });
        key(cx, &h, "cmd-a");
        text(cx, &h, "acc");
        click_label(cx, &h, "Accounts");
        if binding == "enter" {
            ui(cx, &h, |w, cx| w.click("primary", cx));
        } else {
            key(cx, &h, binding);
        }
        text(cx, &h, "billing");
        key(cx, &h, binding);
        assert_eq!(
            opened(&h),
            "https://app.datadog.com/company/8c21-http?from=accounts&to=billing"
        );
    }
}

#[gpui_kit::test]
fn empty_feedback_distinguishes_catalog_folder_and_filter(cx: &mut TestAppContext) {
    let h = setup_source(cx, "navigation: []");
    ui(cx, &h, |w, _| {
        assert_eq!(w.find("body-message").label(), Some("No flows configured"));
        assert_eq!(
            w.find("commitment-hint").label(),
            Some("Edit the configuration file. Restart URLchemy.")
        );
        assert!(w.find("close").visible());
    });
    let h = setup_source(
        cx,
        "navigation:\n  - folder:\n      label: Empty\n      children: []",
    );
    key(cx, &h, "enter");
    ui(cx, &h, |w, _| {
        assert_eq!(
            w.find("body-message").label(),
            Some("No entries in this folder")
        );
        assert!(w.find("back").visible());
    });
    let h = setup(cx, "linear.yaml", "");
    text(cx, &h, "absent");
    key(cx, &h, "enter");
    assert!(h.effects.borrow().is_empty());
    ui(cx, &h, |w, _| {
        assert_eq!(w.find("body-message").label(), Some("No matches"))
    });
    key(cx, &h, "cmd-a");
    key(cx, &h, "backspace");
    key(cx, &h, "enter");
    text(cx, &h, "unmatched");
    ui(cx, &h, |w, _| {
        assert_eq!(
            w.find("body-message").label(),
            Some("No suggestions. You can use the text you enter.")
        )
    });
    key(cx, &h, "enter");
    assert_eq!(value(cx, &h, "Application"), "");
}

#[gpui_kit::test]
fn destination_error_has_back_without_an_editable_field(cx: &mut TestAppContext) {
    let h = setup_source(cx, include_str!("../fixtures/dot_error.yaml"));
    key(cx, &h, "enter");
    click_label(cx, &h, "Parent");
    key(cx, &h, "enter");
    ui(cx, &h, |w, _| {
        assert!(w.try_find("launcher-field").is_none());
        assert!(w.find_all("row-label").is_empty());
        assert_eq!(
            w.find("error").label(),
            Some("A rendered path segment must not be a dot segment.")
        );
        assert!(w.find("back").visible());
        assert_eq!(w.find("back").focused(), Some(true));
    });
    assert!(h.effects.borrow().is_empty());
    key(cx, &h, "enter");
    assert_eq!(value(cx, &h, "Segment"), "..");
}

#[gpui_kit::test]
fn tab_order_follows_the_current_surface(cx: &mut TestAppContext) {
    let h = setup(cx, "branching.yaml", "");
    text(cx, &h, "open-dd");
    click_label(cx, &h, "HTTP Traffic");
    text(cx, &h, "acc");
    ui(cx, &h, |w, cx| {
        assert!(
            h.view
                .read(cx)
                .editor()
                .read(cx)
                .focus_handle(cx)
                .is_focused(w)
        )
    });
    for id in ["primary", "back", "close", "launcher-field"] {
        key(cx, &h, "tab");
        ui(cx, &h, |w, cx| {
            if id == "launcher-field" {
                assert!(
                    h.view
                        .read(cx)
                        .editor()
                        .read(cx)
                        .focus_handle(cx)
                        .is_focused(w)
                );
            } else {
                assert_eq!(w.find(id).focused(), Some(true), "{id}");
            }
        });
    }
    for id in ["close", "back", "primary", "launcher-field"] {
        key(cx, &h, "shift-tab");
        ui(cx, &h, |w, cx| {
            if id == "launcher-field" {
                assert!(
                    h.view
                        .read(cx)
                        .editor()
                        .read(cx)
                        .focus_handle(cx)
                        .is_focused(w)
                );
            } else {
                assert_eq!(w.find(id).focused(), Some(true), "{id}");
            }
        });
    }
    assert_eq!(value(cx, &h, "Source application"), "acc");
    assert!(h.effects.borrow().is_empty());
    key(cx, &h, "enter");
    text(cx, &h, "billing");
    key(cx, &h, "enter");
    assert_eq!(
        opened(&h),
        "https://app.datadog.com/company/8c21-http?from=acc&to=billing"
    );
}

#[gpui_kit::test]
fn queued_activation_does_not_cross_a_restored_context(cx: &mut TestAppContext) {
    let h = setup(cx, "branching.yaml", "");
    text(cx, &h, "open-dd");
    ui(cx, &h, |w, cx| {
        let position = w.find_by_label("HTTP Traffic").bounds().center();
        w.pointer_down(position, gpui_kit::MouseButton::Left, cx);
        w.pointer_up(position, gpui_kit::MouseButton::Left, cx);
        h.view.update(cx, |view, cx| {
            view.dispatch(urlchemy_app::PaletteCommand::Back, w, cx)
        });
    });
    assert_eq!(value(cx, &h, "Open a flow"), "");
    assert!(h.effects.borrow().is_empty());
}

#[gpui_kit::test]
fn context_keeps_the_current_object_and_excludes_answers(cx: &mut TestAppContext) {
    let root = "A very long configured Workspace ancestor";
    let flow = "A very long configured dashboard flow";
    let source =
        include_str!("../../../../docs/superpowers/specs/yaml-url-contract/branching.yaml")
            .replace("label: Datadog", &format!("label: {root}"))
            .replace("label: Open dashboard", &format!("label: {flow}"));
    let h = setup_source(cx, &source);
    text(cx, &h, "open-dd");
    click_label(cx, &h, "HTTP Traffic");
    cx.simulate_window_resize(h.window, size(px(360.), px(480.)));
    settle(cx, &h);
    text(cx, &h, "private-answer-sentinel");
    key(cx, &h, "enter");
    let full = format!("{root} / {flow} / HTTP Traffic");
    ui(cx, &h, |w, _| {
        assert_eq!(w.find("context-summary").label(), Some(full.as_str()));
        assert_eq!(w.find("context-visible").label(), Some("… / HTTP Traffic"));
        assert!(
            !w.find("context-summary")
                .label()
                .unwrap()
                .contains("private-answer-sentinel")
        );
    });
    key(cx, &h, "tab");
    key(cx, &h, "tab");
    ui(cx, &h, |w, _| {
        assert_eq!(w.find("back").focused(), Some(true));
        assert_eq!(w.find("context-tooltip").label(), Some(full.as_str()));
    });
    key(cx, &h, "tab");
    ui(cx, &h, |w, _| {
        assert!(w.try_find("context-tooltip").is_none())
    });
    key(cx, &h, "shift-tab");
    ui(cx, &h, |w, _| assert!(w.find("context-tooltip").visible()));
    key(cx, &h, "escape");
    ui(cx, &h, |w, _| {
        assert!(w.try_find("context-tooltip").is_none());
        assert!(w.find("back").visible());
    });
    assert!(h.effects.borrow().is_empty());
    key(cx, &h, "enter");
    assert_eq!(
        value(cx, &h, "Source application"),
        "private-answer-sentinel"
    );
    key(cx, &h, "cmd-[");
    ui(cx, &h, |w, _| {
        assert!(w.try_find("context-tooltip").is_none());
        assert!(
            !w.find("context-summary")
                .label()
                .unwrap()
                .contains("HTTP Traffic")
        );
    });
    key(cx, &h, "escape");
    assert!(matches!(
        h.effects.borrow().last(),
        Some(PaletteEffect::Close { .. })
    ));
}

fn assert_layout(cx: &mut TestAppContext, h: &Harness) {
    ui(cx, h, |w, _| {
        let footer = w.find("palette-footer").bounds();
        let close = w.find("close").bounds();
        assert!(footer.size.height > px(0.));
        assert!(
            footer.bottom() <= w.viewport_size().height,
            "footer={footer:?}, field={:?}, hint={:?}, primary={:?}, viewport={:?}",
            w.try_find("launcher-field").map(|field| field.bounds()),
            w.find("commitment-hint").bounds(),
            w.try_find("primary").map(|button| button.bounds()),
            w.viewport_size()
        );
        assert!(footer.right() <= w.viewport_size().width);
        assert!(close.right() <= w.viewport_size().width);
        assert!(close.top() >= px(0.));
        if let Some(field) = w.try_find("launcher-field") {
            assert!(field.bounds().size.height > px(0.));
            assert_eq!(field.bounds().left(), footer.left());
        }
        for id in ["primary", "retry", "start-over"] {
            if let Some(button) = w.try_find(id) {
                assert!(button.bounds().right() <= w.viewport_size().width, "{id}");
                assert!(button.bounds().bottom() <= w.viewport_size().height, "{id}");
            }
        }
    });
}

#[gpui_kit::test]
fn state_layout_remains_usable_with_long_content(cx: &mut TestAppContext) {
    for font in [14., 20.] {
        let h = setup(cx, "linear.yaml", "");
        cx.update(|cx| gpui_kit::component::Theme::update(cx, |theme| theme.font_size = px(font)));
        cx.simulate_window_resize(h.window, size(px(360.), px(420.)));
        settle(cx, &h);
        assert_layout(cx, &h);
        text(cx, &h, "no-match");
        assert_layout(cx, &h);
        key(cx, &h, "cmd-a");
        key(cx, &h, "backspace");
        key(cx, &h, "enter");
        key(cx, &h, "enter");
        assert_layout(cx, &h);
        text(cx, &h, &"long-answer".repeat(16));
        assert_layout(cx, &h);
        let h = setup_source(cx, include_str!("../fixtures/dot_error.yaml"));
        cx.simulate_window_resize(h.window, size(px(360.), px(420.)));
        key(cx, &h, "enter");
        click_label(cx, &h, "Parent");
        key(cx, &h, "enter");
        assert_layout(cx, &h);
        let h = setup(cx, "fixed.yaml", "");
        cx.simulate_window_resize(h.window, size(px(360.), px(420.)));
        text(cx, &h, "service-status");
        assert_layout(cx, &h);
        let (invocation, operation) = match &h.effects.borrow()[0] {
            PaletteEffect::Open {
                invocation,
                operation,
                ..
            } => (*invocation, *operation),
            _ => panic!("No opening effect"),
        };
        ui(cx, &h, |w, cx| {
            h.view.update(cx, |view, cx| {
                view.dispatch(
                    urlchemy_app::PaletteCommand::OpenFinished {
                        invocation,
                        operation,
                        result: Err(urlchemy_app::DesktopError::OpenFailed),
                    },
                    w,
                    cx,
                )
            })
        });
        assert_layout(cx, &h);
    }
    let mut source = "navigation:\n".to_owned();
    for ix in 0..40 {
        source.push_str(&format!("  - flow:\n      id: flow-{ix}\n      label: Flow {ix}\n      baseURL: https://example.com\n      start:\n        destination:\n          template: /fixed\n"));
    }
    let h = setup_source(cx, &source);
    cx.simulate_window_resize(h.window, size(px(360.), px(420.)));
    settle(cx, &h);
    let (field, footer) = cx
        .update_window(h.window, |_, w, cx| {
            w.render_frame(cx);
            (
                w.find("launcher-field").bounds(),
                w.find("palette-footer").bounds(),
            )
        })
        .unwrap();
    ui(cx, &h, |w, cx| {
        w.scroll(
            "palette-body",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-300.))),
            cx,
        )
    });
    ui(cx, &h, |w, _| {
        assert_eq!(w.find("launcher-field").bounds(), field);
        assert_eq!(w.find("palette-footer").bounds(), footer);
        assert!(
            w.try_find_by_label("Flow 0")
                .is_none_or(|row| !row.visible())
        );
    });
    assert_layout(cx, &h);
}

#[gpui_kit::test]
fn hover_does_not_replace_an_explicit_suggestion(cx: &mut TestAppContext) {
    let h = setup(cx, "branching.yaml", "");
    text(cx, &h, "open-dd");
    click_label(cx, &h, "HTTP Traffic");
    click_label(cx, &h, "Accounts");
    ui(cx, &h, |w, cx| {
        let position = w.find_by_label("Billing").bounds().center();
        w.pointer_move(position, None, cx);
    });
    settle(cx, &h);
    ui(cx, &h, |w, _| {
        assert_eq!(w.find("primary").label(), Some("Use Accounts"))
    });
    key(cx, &h, "enter");
    text(cx, &h, "billing");
    key(cx, &h, "enter");
    assert_eq!(
        opened(&h),
        "https://app.datadog.com/company/8c21-http?from=accounts&to=billing"
    );
}

#[gpui_kit::test]
fn context_help_dismisses_before_the_palette(cx: &mut TestAppContext) {
    let source =
        include_str!("../../../../docs/superpowers/specs/yaml-url-contract/branching.yaml")
            .replace(
                "label: Datadog",
                "label: A long configured ancestor for context help",
            )
            .replace(
                "label: Open dashboard",
                "label: Another long configured ancestor for context help",
            );
    let h = setup_source(cx, &source);
    text(cx, &h, "open-dd");
    click_label(cx, &h, "HTTP Traffic");
    cx.simulate_window_resize(h.window, size(px(360.), px(420.)));
    settle(cx, &h);
    ui(cx, &h, |w, cx| w.hover("context-summary", cx));
    ui(cx, &h, |w, _| assert!(w.find("context-tooltip").visible()));
    ui(cx, &h, |w, cx| {
        let position = w.find("launcher-field").bounds().center();
        w.pointer_move(position, None, cx);
    });
    ui(cx, &h, |w, _| {
        assert!(w.try_find("context-tooltip").is_none())
    });
    key(cx, &h, "tab");
    key(cx, &h, "tab");
    ui(cx, &h, |w, _| assert!(w.find("context-tooltip").visible()));
    key(cx, &h, "escape");
    assert!(h.effects.borrow().is_empty());
    ui(cx, &h, |w, _| {
        assert!(w.try_find("context-tooltip").is_none())
    });
    key(cx, &h, "escape");
    assert!(matches!(
        h.effects.borrow().last(),
        Some(PaletteEffect::Close { .. })
    ));
}
