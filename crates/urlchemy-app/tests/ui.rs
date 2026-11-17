use gpui_kit::test::{TestEventExt, TestInput, TestQueryExt, TestWindowExt};
use gpui_kit::{
    AnyWindowHandle, App, AppContext, Bounds, ClipboardItem, ElementInputHandler, Entity,
    TestAppContext, Window, WindowBounds, WindowOptions, px, size,
};
use std::{cell::RefCell, path::Path, rc::Rc};
use urlchemy_app::{Launcher, PaletteEffect, PaletteEvent, PaletteView};
use urlchemy_core::config::load_bytes;

struct Harness {
    window: AnyWindowHandle,
    view: Entity<PaletteView>,
    effects: Rc<RefCell<Vec<PaletteEffect>>>,
}
fn setup(cx: &mut TestAppContext, file: &str, prefix: &str) -> Harness {
    cx.update(gpui_kit::init);
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/superpowers/specs/yaml-url-contract")
        .join(file);
    let bytes = format!("{prefix}{}", std::fs::read_to_string(&path).unwrap());
    let catalog = Rc::new(load_bytes(&path, bytes.as_bytes()).unwrap());
    let (window, view) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    Default::default(),
                    size(px(640.), px(480.)),
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| PaletteView::new(Launcher::new(catalog, 1), window, cx)),
        )
        .unwrap()
    });
    let effects = Rc::new(RefCell::new(Vec::new()));
    let seen = effects.clone();
    cx.update(|cx| {
        cx.subscribe(&view, move |_, event: &PaletteEvent, _| {
            if let PaletteEvent::Effect(effect) = event {
                seen.borrow_mut().push(effect.clone());
            }
        })
        .detach()
    });
    let h = Harness {
        window,
        view,
        effects,
    };
    settle(cx, &h);
    h
}
fn settle(cx: &mut TestAppContext, h: &Harness) {
    cx.run_until_parked();
    cx.update_window(h.window, |_, w, cx| w.render_frame(cx))
        .unwrap();
}
fn ui(cx: &mut TestAppContext, h: &Harness, f: impl FnOnce(&mut Window, &mut App)) {
    cx.update_window(h.window, |_, w, cx| {
        w.render_frame(cx);
        f(w, cx);
    })
    .unwrap();
    settle(cx, h);
}
fn key(cx: &mut TestAppContext, h: &Harness, key: &str) {
    ui(cx, h, |w, cx| w.press(key, cx));
}
fn text(cx: &mut TestAppContext, h: &Harness, text: &str) {
    ui(cx, h, |w, cx| w.input(text, cx));
}
fn value(cx: &mut TestAppContext, h: &Harness, label: &str) -> String {
    cx.update_window(h.window, |_, w, _| {
        w.find_by_label(label).value().unwrap().to_owned()
    })
    .unwrap()
}
fn click_label(cx: &mut TestAppContext, h: &Harness, label: &str) {
    ui(cx, h, |w, cx| {
        let position = w.find_by_label(label).bounds().center();
        w.pointer_down(position, gpui_kit::MouseButton::Left, cx);
        w.pointer_up(position, gpui_kit::MouseButton::Left, cx);
    });
}
fn opened(h: &Harness) -> String {
    let effects = h.effects.borrow();
    assert_eq!(effects.len(), 1);
    let PaletteEffect::Open { url, .. } = &effects[0] else {
        panic!("No URL request")
    };
    url.as_str().into()
}

#[gpui_kit::test]
fn linear_journey_uses_the_real_view(cx: &mut TestAppContext) {
    let h = setup(cx, "linear.yaml", "");
    key(cx, &h, "enter");
    click_label(cx, &h, "Jobs");
    assert_eq!(value(cx, &h, "Section"), "");
    ui(cx, &h, |w, cx| w.click("primary", cx));
    text(cx, &h, "accounts");
    key(cx, &h, "enter");
    assert_eq!(
        opened(&h),
        "https://idp.example.com/company/engineering/application/accounts/jobs?owner=accounts&owner=backup"
    );
}
#[gpui_kit::test]
fn visible_suggestion_does_not_replace_literal_text(cx: &mut TestAppContext) {
    let h = setup(cx, "linear.yaml", "");
    key(cx, &h, "enter");
    text(cx, &h, "Jobs");
    key(cx, &h, "enter");
    text(cx, &h, "accounts");
    key(cx, &h, "enter");
    assert_eq!(
        opened(&h),
        "https://idp.example.com/company/engineering/application/accounts/Jobs?owner=accounts&owner=backup"
    );
}
#[gpui_kit::test]
fn custom_launch_replaces_enter(cx: &mut TestAppContext) {
    let h = setup(cx, "linear.yaml", "shortcuts: {launch: cmd-enter}\n");
    key(cx, &h, "enter");
    assert_eq!(value(cx, &h, "Open a flow"), "");
    key(cx, &h, "cmd-enter");
    text(cx, &h, "jobs");
    key(cx, &h, "enter");
    assert_eq!(value(cx, &h, "Section"), "jobs");
    key(cx, &h, "cmd-enter");
    assert_eq!(value(cx, &h, "Application"), "");
    ui(cx, &h, |w, cx| w.key_down("cmd-enter", true, cx));
    assert!(h.effects.borrow().is_empty());
    assert_eq!(value(cx, &h, "Application"), "");
}
#[gpui_kit::test]
fn paste_preserves_invalid_controls_until_confirmation(cx: &mut TestAppContext) {
    let h = setup(cx, "linear.yaml", "");
    key(cx, &h, "enter");
    for pasted in ["jobs\narchive", "jobs\rarchive"] {
        key(cx, &h, "cmd-a");
        cx.update(|cx| cx.write_to_clipboard(ClipboardItem::new_string(pasted.into())));
        key(cx, &h, "cmd-v");
        assert_eq!(value(cx, &h, "Section"), pasted);
        key(cx, &h, "enter");
        ui(cx, &h, |w, _| {
            assert_eq!(w.find("error").label(), Some("Remove control characters."))
        });
        assert!(h.effects.borrow().is_empty());
    }
}
#[gpui_kit::test]
fn composition_commit_does_not_confirm(cx: &mut TestAppContext) {
    let h = setup(cx, "linear.yaml", "");
    key(cx, &h, "enter");
    ui(cx, &h, |w, cx| {
        let editor = h.view.read(cx).editor();
        let bounds = w.find("launcher-field").bounds();
        let mut input = TestInput::new(ElementInputHandler::new(bounds, editor), w);
        input.compose_text(None, "jo", Some(2..2), cx);
    });
    key(cx, &h, "enter");
    assert_eq!(value(cx, &h, "Section"), "jo");
    assert!(h.effects.borrow().is_empty());
    key(cx, &h, "enter");
    assert_eq!(value(cx, &h, "Application"), "");
}
#[gpui_kit::test]
fn old_editor_events_do_not_replay_alias_text(cx: &mut TestAppContext) {
    let h = setup(cx, "branching.yaml", "");
    text(cx, &h, "open-dd");
    assert_eq!(value(cx, &h, "Dashboard"), "");
    assert!(h.effects.borrow().is_empty());
}
#[gpui_kit::test]
fn back_escape_arrows_and_geometry(cx: &mut TestAppContext) {
    let h = setup(cx, "linear.yaml", "");
    key(cx, &h, "enter");
    key(cx, &h, "down");
    key(cx, &h, "up");
    key(cx, &h, "enter");
    assert_eq!(value(cx, &h, "Application"), "");
    text(cx, &h, "unfinished");
    key(cx, &h, "cmd-[");
    assert_eq!(value(cx, &h, "Section"), "jobs");
    text(cx, &h, "new");
    assert_eq!(value(cx, &h, "Section"), "new");
    for font in [14., 20.] {
        cx.update(|cx| gpui_kit::component::Theme::update(cx, |theme| theme.font_size = px(font)));
        settle(cx, &h);
        ui(cx, &h, |w, _| {
            let field = w.find("launcher-field").bounds();
            let footer = w.find("palette-footer").bounds();
            assert!(field.size.height > px(0.));
            assert!(footer.top() >= field.bottom());
            assert!(footer.bottom() <= w.viewport_size().height);
            assert_eq!(field.left(), footer.left());
        });
    }
    cx.simulate_window_resize(h.window, size(px(360.), px(380.)));
    settle(cx, &h);
    ui(cx, &h, |w, _| {
        assert!(w.find("primary").bounds().right() <= w.viewport_size().width)
    });
    key(cx, &h, "escape");
    assert!(matches!(
        &h.effects.borrow()[0],
        PaletteEffect::Close { .. }
    ));
}
