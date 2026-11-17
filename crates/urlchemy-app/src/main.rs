#![forbid(unsafe_code)]

#[cfg(target_os = "macos")]
fn main() {
    use std::rc::Rc;
    use urlchemy_app::{AppShell, OpenLauncher, StartupState, native::MacDesktopServices};
    let application = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    application.on_reopen(|cx| cx.dispatch_action(&OpenLauncher));
    application.run(|cx| {
        gpui_kit::init(cx);
        AppShell::install(
            StartupState::Loading,
            Rc::new(MacDesktopServices::new()),
            cx,
        );
    });
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("URLchemy requires macOS.");
}
