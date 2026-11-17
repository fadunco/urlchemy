#![forbid(unsafe_code)]

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{
        path::PathBuf,
        rc::Rc,
        time::{SystemTime, UNIX_EPOCH},
    };
    use urlchemy_app::{AppShell, OpenLauncher, StartupState, native::MacDesktopServices};

    struct FixtureHome(PathBuf);
    impl Drop for FixtureHome {
        fn drop(&mut self) {
            // This directory belongs only to this process's acceptance fixture.
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let home = std::env::temp_dir().join(format!("urlchemy-native-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&home)?;
    let fixture = FixtureHome(home);
    let path = urlchemy_core::config::configuration_path(&fixture.0);
    std::fs::create_dir_all(path.parent().ok_or("Missing fixture parent directory")?)?;
    std::fs::write(
        &path,
        include_bytes!("../../../docs/superpowers/specs/yaml-url-contract/linear.yaml"),
    )?;
    let catalog = urlchemy_core::config::load_file(&path)
        .map_err(|_| "Could not load the acceptance fixture.")?;

    let application = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    application.on_reopen(|cx| cx.dispatch_action(&OpenLauncher));
    application.run(move |cx| {
        gpui_kit::init(cx);
        AppShell::install(
            StartupState::Ready(Rc::new(catalog)),
            Rc::new(MacDesktopServices::new()),
            cx,
        );
    });
    drop(fixture);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("Native acceptance requires macOS.");
}
