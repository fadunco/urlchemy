use gpui_kit::{App, Bounds, Pixels, Task};
use std::{fmt, path::PathBuf, sync::Arc, time::Duration};
use urlchemy_core::{ResolvedUrl, Shortcut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopError {
    HomeUnavailable,
    DisplayUnavailable,
    RegistrationFailed,
    NativeUrlRejected,
    OpenFailed,
}
impl fmt::Display for DesktopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::HomeUnavailable => "Could not locate the configuration directory.",
            Self::DisplayUnavailable => "No display is available.",
            Self::RegistrationFailed => "Could not register the invocation shortcut.",
            Self::NativeUrlRejected | Self::OpenFailed => "Could not open the browser.",
        })
    }
}
impl std::error::Error for DesktopError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotkeySignal {
    id: u32,
    pressed: bool,
}
impl HotkeySignal {
    pub fn new(id: u32, pressed: bool) -> Self {
        Self { id, pressed }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }
}
pub type HotkeyWait = Arc<dyn Fn(Duration) -> Option<HotkeySignal> + Send + Sync>;
pub struct RegisteredHotkey {
    id: u32,
    wait: HotkeyWait,
}
impl RegisteredHotkey {
    pub fn new(id: u32, wait: HotkeyWait) -> Self {
        Self { id, wait }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn wait(&self) -> HotkeyWait {
        self.wait.clone()
    }
}

/// Desktop effects are injected so automated tests never invoke native services.
pub trait DesktopServices {
    fn configuration_path(&self) -> Result<PathBuf, DesktopError>;
    fn register_invocation(&self, shortcut: &Shortcut) -> Result<RegisteredHotkey, DesktopError>;
    fn open_url(&self, url: &ResolvedUrl, cx: &App) -> Task<Result<(), DesktopError>>;
    fn remember_frontmost(&self, invocation: u64);
    fn restore_frontmost(&self, invocation: u64);
    fn discard_frontmost(&self, invocation: u64);
    fn is_launcher_frontmost(&self) -> bool;
    fn work_area(&self, cx: &App) -> Result<Bounds<Pixels>, DesktopError>;
    fn shutdown(&self);
}
