#![forbid(unsafe_code)]
mod desktop;
mod hotkey_bridge;
mod launcher;
pub use desktop::*;
pub use hotkey_bridge::{HotkeyBridge, HotkeyLatch};
pub use launcher::*;
mod palette;
pub use palette::{PaletteEvent, PaletteView};
mod shell;
pub use shell::{AppShell, OpenLauncher, Quit, StartupFailure, StartupState, palette_bounds};
#[cfg(target_os = "macos")]
pub mod native;
