mod support;
use gpui_kit::test::{TestQueryExt, TestWindowExt};
use gpui_kit::{
    AnyWindowHandle, App, AppContext, Bounds, Entity, TestAppContext, VisualTestContext, Window,
    point, px, size,
};
use std::{path::Path, rc::Rc};
use support::{FakeDesktop, catalog};
use urlchemy_app::{
    AppShell, DesktopError, HotkeySignal, OpenLauncher, StartupFailure, StartupState,
    palette_bounds,
};

fn setup(
    cx: &mut TestAppContext,
    initial: StartupState,
    services: Rc<FakeDesktop>,
) -> Entity<AppShell> {
    cx.update(gpui_kit::init);
    let shell = cx.update(|cx| AppShell::install(initial, services, cx));
    cx.run_until_parked();
    shell
}
fn menu(cx: &mut TestAppContext) {
    // The test platform retains a closed active handle. Clear activation before
    // testing the application menu without a focused window.
    if let Some(handle) = cx.update(|cx| cx.active_window()) {
        VisualTestContext::from_window(handle, cx).deactivate_window();
    }
    cx.update(|cx| cx.dispatch_action(&OpenLauncher));
    cx.run_until_parked();
}
fn window(cx: &TestAppContext) -> AnyWindowHandle {
    let windows = cx.windows();
    assert_eq!(windows.len(), 1);
    windows[0]
}
fn ui(cx: &mut TestAppContext, f: impl FnOnce(&mut Window, &mut App)) {
    cx.update_window(window(cx), |_, w, cx| {
        w.render_frame(cx);
        f(w, cx);
    })
    .unwrap();
    cx.run_until_parked();
}
fn root(cx: &mut TestAppContext) {
    ui(cx, |w, _| {
        assert_eq!(w.find_by_label("Open a flow").value(), Some(""))
    });
}
fn finish_flow(cx: &mut TestAppContext) {
    ui(cx, |w, cx| w.press("enter", cx));
    ui(cx, |w, cx| w.input("jobs", cx));
    ui(cx, |w, cx| w.press("enter", cx));
    ui(cx, |w, cx| w.input("accounts", cx));
    ui(cx, |w, cx| w.press("enter", cx));
}
#[gpui_kit::test]
fn registration_failure_keeps_menu_recovery(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    services.registration_fails.set(true);
    let _shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    ui(cx, |w, cx| w.click("close", cx));
    assert!(cx.windows().is_empty());
    menu(cx);
    ui(cx, |w, _| {
        let message = w.find("registration-error").label().unwrap().to_owned();
        assert!(message.contains("ctrl-alt-space"));
        assert!(message.contains("Restart"));
        assert!(w.find("close").visible());
    });
    finish_flow(cx);
    assert_eq!(services.opened.borrow().len(), 1);
    assert_eq!(
        services.registrations.borrow().as_slice(),
        ["ctrl-alt-space"]
    );
    assert_eq!(services.paths.get(), 0);
}
#[gpui_kit::test]
fn compact_root_keeps_its_ordinal_after_resize(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let _shell = setup(cx, StartupState::Ready(catalog()), services);
    root(cx);
    ui(cx, |w, cx| {
        assert!(w.find_by_label("⌘1").visible());
        w.press("cmd-1", cx);
    });
    ui(cx, |w, _| assert!(w.find_by_label("Section").visible()));
}

#[gpui_kit::test]
fn registration_failure_keeps_navigation_ordinals(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    services.registration_fails.set(true);
    let _shell = setup(cx, StartupState::Ready(catalog()), services);
    ui(cx, |w, cx| {
        assert!(w.find("registration-error").visible());
        assert!(w.find_by_label("⌘1").visible());
        w.press("cmd-1", cx);
    });
    ui(cx, |w, _| assert!(w.find_by_label("Section").visible()));
}
#[gpui_kit::test]
fn invalid_config_remains_reachable_without_a_binding(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let diagnostics =
        urlchemy_core::config::load_bytes(Path::new("fixture.yaml"), b"navigation: false")
            .unwrap_err();
    let _shell = setup(
        cx,
        StartupState::Failed(StartupFailure::Configuration(diagnostics)),
        services.clone(),
    );
    ui(cx, |w, cx| w.click("close", cx));
    menu(cx);
    ui(cx, |w, _| {
        let diagnostic = w.find("startup-status").label().unwrap().to_owned();
        assert!(diagnostic.contains("fixture.yaml"));
        assert!(diagnostic.contains("1:13"));
        assert!(diagnostic.contains("/navigation"));
    });
    assert!(services.registrations.borrow().is_empty());
    assert!(services.opened.borrow().is_empty());
    assert_eq!(services.paths.get(), 0);
}
#[gpui_kit::test]
fn reopen_and_hotkey_share_fresh_invocation(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    ui(cx, |w, cx| w.press("enter", cx));
    ui(cx, |w, cx| w.input("partial", cx));
    cx.update(|cx| shell.update(cx, |shell, cx| shell.invoke(cx)));
    cx.run_until_parked();
    root(cx);
    services
        .signals
        .lock()
        .unwrap()
        .push_back(HotkeySignal::new(7, true));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(250));
    cx.run_until_parked();
    root(cx);
    assert_eq!(services.remembered.borrow().as_slice(), [1, 2, 3]);
    assert_eq!(services.registrations.borrow().len(), 1);
}
#[gpui_kit::test]
fn open_failure_retry_and_start_over(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let _shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    finish_flow(cx);
    services.pending.borrow()[0].finish(Err(DesktopError::OpenFailed));
    cx.run_until_parked();
    ui(cx, |w, _| {
        assert!(w.try_find("launcher-field").is_none());
        assert_eq!(w.find("retry").focused(), Some(true));
    });
    ui(cx, |w, cx| w.press("enter", cx));
    assert_eq!(
        services.opened.borrow().as_slice(),
        [
            "https://idp.example.com/company/engineering/application/accounts/jobs?owner=accounts&owner=backup",
            "https://idp.example.com/company/engineering/application/accounts/jobs?owner=accounts&owner=backup",
        ]
    );
    services.pending.borrow()[1].finish(Err(DesktopError::OpenFailed));
    cx.run_until_parked();
    ui(cx, |w, cx| w.click("start-over", cx));
    root(cx);
}
#[gpui_kit::test]
fn old_completions_do_not_close_new_windows(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let _shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    finish_flow(cx);
    menu(cx);
    services.pending.borrow()[0].finish(Ok(()));
    cx.run_until_parked();
    root(cx);
    assert!(services.restored.borrow().is_empty());
}
#[gpui_kit::test]
fn focus_restore_depends_on_close_reason(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    services.frontmost.set(true);
    let _shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    ui(cx, |w, cx| w.press("escape", cx));
    assert!(cx.windows().is_empty());
    assert_eq!(services.restored.borrow().as_slice(), [1]);
    menu(cx);
    finish_flow(cx);
    services.pending.borrow()[0].finish(Ok(()));
    cx.run_until_parked();
    assert!(cx.windows().is_empty());
    assert_eq!(services.restored.borrow().as_slice(), [1]);
    assert!(services.discarded.borrow().contains(&2));
    menu(cx);
    services.frontmost.set(false);
    VisualTestContext::from_window(window(cx), cx).deactivate_window();
    cx.run_until_parked();
    assert!(cx.windows().is_empty());
    assert_eq!(services.restored.borrow().as_slice(), [1]);
}
#[gpui_kit::test]
fn missing_display_can_recover_on_later_invocation(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    services.display_fails.set(true);
    let _shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    assert!(cx.windows().is_empty());
    services.display_fails.set(false);
    menu(cx);
    root(cx);
    assert_eq!(services.registrations.borrow().len(), 1);
    assert_eq!(services.paths.get(), 0);
}
#[gpui_kit::test]
fn shutdown_releases_registration_once(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    cx.update(|cx| {
        shell.update(cx, |shell, cx| {
            shell.shutdown(cx);
            shell.shutdown(cx);
        })
    });
    services
        .signals
        .lock()
        .unwrap()
        .push_back(HotkeySignal::new(7, true));
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    cx.run_until_parked();
    menu(cx);
    assert!(cx.windows().is_empty());
    assert_eq!(services.shutdowns.get(), 1);
}
#[test]
fn panel_bounds_fit_small_work_areas_and_large_fonts() {
    let area = Bounds::new(point(px(-400.), px(40.)), size(px(320.), px(500.)));
    for rem in [14., 20.] {
        let bounds = palette_bounds(area, px(rem), px(900.));
        assert!(bounds.left() >= area.left() + px(rem));
        assert!(bounds.right() <= area.right() - px(rem));
        assert!(bounds.top() >= area.top() + px(rem));
        assert!(bounds.bottom() <= area.bottom() - px(rem));
        assert!(bounds.size.width > px(0.));
    }
}

#[gpui_kit::test]
fn menu_from_a_focused_window_replaces_it(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    services.frontmost.set(true);
    let _shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    root(cx);
    let old = window(cx);
    cx.update(|cx| cx.dispatch_action(&OpenLauncher));
    cx.run_until_parked();
    root(cx);
    assert_ne!(old, window(cx));
    assert_eq!(services.remembered.borrow().as_slice(), [1, 2]);
}

#[gpui_kit::test]
fn owned_deactivation_does_not_dismiss_the_palette(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    services.frontmost.set(true);
    let _shell = setup(cx, StartupState::Ready(catalog()), services);
    VisualTestContext::from_window(window(cx), cx).deactivate_window();
    cx.run_until_parked();
    root(cx);
}

#[gpui_kit::test]
fn loading_failure_is_published_without_registering(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let _shell = setup(cx, StartupState::Loading, services.clone());
    ui(cx, |w, _| {
        assert!(
            w.find("startup-status")
                .label()
                .unwrap()
                .contains("configuration directory")
        )
    });
    assert_eq!(services.paths.get(), 1);
    assert!(services.registrations.borrow().is_empty());
}

#[gpui_kit::test]
fn short_content_resizes_without_moving_the_top_anchor(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let _shell = setup(cx, StartupState::Ready(catalog()), services);
    root(cx);
    let initial = cx.update_window(window(cx), |_, w, _| w.bounds()).unwrap();
    assert!(initial.size.height < px(350.));
    ui(cx, |w, cx| w.press("enter", cx));
    ui(cx, |w, cx| w.input("no-matching-suggestion", cx));
    ui(cx, |w, _| {
        assert_eq!(w.bounds().top(), initial.top());
        assert!(w.bounds().size.height < initial.size.height);
        assert!(w.find("palette-footer").bounds().bottom() <= w.viewport_size().height);
    });
}

#[gpui_kit::test]
fn loading_reads_the_supplied_file_once(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let path = std::env::temp_dir().join(format!("urlchemy-startup-{}.yaml", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    use std::io::Write;
    file.write_all(b"navigation: []\n").unwrap();
    *services.configuration.borrow_mut() = Some(path.clone());
    let _shell = setup(cx, StartupState::Loading, services.clone());
    root(cx);
    std::fs::remove_file(path).unwrap();
    menu(cx);
    root(cx);
    assert_eq!(services.paths.get(), 1);
    assert_eq!(services.registrations.borrow().len(), 1);
}

#[gpui_kit::test]
fn growing_content_stays_inside_the_work_area(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| gpui_kit::component::Theme::update(cx, |theme| theme.font_size = px(24.)));
    let services = Rc::new(FakeDesktop::default());
    let mut source = String::from("navigation:\n");
    for ix in 0..60 {
        source.push_str(&format!("  - flow:\n      id: flow-{ix}\n      label: Flow {ix}\n      baseURL: https://example.com\n      start:\n        destination:\n          template: /fixed\n"));
    }
    let catalog = Rc::new(
        urlchemy_core::config::load_bytes(Path::new("fixture.yaml"), source.as_bytes()).unwrap(),
    );
    let _shell = cx.update(|cx| AppShell::install(StartupState::Ready(catalog), services, cx));
    cx.run_until_parked();
    for _ in 0..3 {
        ui(cx, |_, _| {});
    }
    ui(cx, |w, _| {
        assert!(w.bounds().bottom() <= px(820.) - px(24.));
        assert!(w.find("palette-footer").bounds().bottom() <= w.viewport_size().height);
    });
}

#[gpui_kit::test]
fn pending_opening_blocks_duplicate_submission(cx: &mut TestAppContext) {
    let services = Rc::new(FakeDesktop::default());
    let _shell = setup(cx, StartupState::Ready(catalog()), services.clone());
    finish_flow(cx);
    ui(cx, |w, cx| {
        assert_eq!(
            w.find("body-message").label(),
            Some("Opening in your default browser.")
        );
        assert!(w.try_find("launcher-field").is_none());
        assert!(w.try_find("primary").is_none());
        assert!(w.find("close").visible());
        w.click("body-message", cx);
        w.press("enter", cx);
    });
    assert_eq!(services.opened.borrow().len(), 1);
    menu(cx);
    services.pending.borrow()[0].finish(Ok(()));
    cx.run_until_parked();
    root(cx);
}
