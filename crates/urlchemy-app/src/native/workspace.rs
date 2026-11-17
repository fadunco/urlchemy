use super::native_hotkey;
use crate::{DesktopError, DesktopServices, HotkeySignal, RegisteredHotkey};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gpui_kit::{App, Bounds, Pixels, Task, point, px, size};
use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::{
    NSApplicationActivationOptions, NSEvent, NSRunningApplication, NSScreen, NSWorkspace,
};
use objc2_foundation::{NSHomeDirectory, NSString, NSURL};
use std::{cell::RefCell, path::PathBuf, sync::Arc};
use urlchemy_core::{ResolvedUrl, Shortcut};

struct FocusLease<T> {
    entry: Option<(u64, T)>,
}
impl<T> Default for FocusLease<T> {
    fn default() -> Self {
        Self { entry: None }
    }
}
impl<T> FocusLease<T> {
    fn remember(&mut self, invocation: u64, external: Option<T>) {
        if let Some(target) = external {
            self.entry = Some((invocation, target));
        } else if let Some((id, _)) = &mut self.entry {
            *id = invocation;
        }
    }
    fn take(&mut self, invocation: u64, launcher_frontmost: bool) -> Option<T> {
        if self.entry.as_ref().is_some_and(|(id, _)| *id == invocation) {
            let target = self.entry.take().map(|(_, target)| target);
            if launcher_frontmost {
                return target;
            }
        }
        None
    }
    fn clear(&mut self, invocation: u64) {
        self.take(invocation, false);
    }
}

fn screen_bounds(primary_top: f64, rect: [f64; 4]) -> Bounds<Pixels> {
    // Cocoa uses bottom-left screen coordinates. GPUI uses top-left points.
    Bounds::new(
        point(
            px(rect[0] as f32),
            px((primary_top - rect[1] - rect[3]) as f32),
        ),
        size(px(rect[2] as f32), px(rect[3] as f32)),
    )
}
#[derive(Default)]
pub struct MacDesktopServices {
    manager: RefCell<Option<GlobalHotKeyManager>>,
    focus: RefCell<FocusLease<Retained<NSRunningApplication>>>,
}
impl MacDesktopServices {
    pub fn new() -> Self {
        Self::default()
    }
}
impl DesktopServices for MacDesktopServices {
    fn configuration_path(&self) -> Result<PathBuf, DesktopError> {
        let home = NSHomeDirectory().to_string();
        if home.is_empty() {
            return Err(DesktopError::HomeUnavailable);
        }
        Ok(urlchemy_core::config::configuration_path(
            std::path::Path::new(&home),
        ))
    }
    fn register_invocation(&self, shortcut: &Shortcut) -> Result<RegisteredHotkey, DesktopError> {
        if MainThreadMarker::new().is_none() || self.manager.borrow().is_some() {
            return Err(DesktopError::RegistrationFailed);
        }
        let hotkey = native_hotkey(shortcut)?;
        let manager = GlobalHotKeyManager::new().map_err(|_| DesktopError::RegistrationFailed)?;
        manager
            .register(hotkey)
            .map_err(|_| DesktopError::RegistrationFailed)?;
        *self.manager.borrow_mut() = Some(manager);
        let receiver = GlobalHotKeyEvent::receiver().clone();
        Ok(RegisteredHotkey::new(
            hotkey.id(),
            Arc::new(move |timeout| {
                receiver.recv_timeout(timeout).ok().map(|event| {
                    HotkeySignal::new(event.id(), event.state() == HotKeyState::Pressed)
                })
            }),
        ))
    }
    fn open_url(&self, url: &ResolvedUrl, _: &App) -> Task<Result<(), DesktopError>> {
        let result = NSURL::URLWithString(&NSString::from_str(url.as_str()))
            .ok_or(DesktopError::NativeUrlRejected)
            .and_then(|url| {
                if NSWorkspace::sharedWorkspace().openURL(&url) {
                    Ok(())
                } else {
                    Err(DesktopError::OpenFailed)
                }
            });
        Task::ready(result)
    }
    fn remember_frontmost(&self, invocation: u64) {
        let Some(application) = NSWorkspace::sharedWorkspace().frontmostApplication() else {
            self.focus.borrow_mut().entry = None;
            return;
        };
        let external =
            (application.processIdentifier() as u32 != std::process::id()).then_some(application);
        self.focus.borrow_mut().remember(invocation, external);
    }
    fn restore_frontmost(&self, invocation: u64) {
        if let Some(application) = self
            .focus
            .borrow_mut()
            .take(invocation, self.is_launcher_frontmost())
            && !application.isTerminated()
        {
            application.activateWithOptions(NSApplicationActivationOptions::empty());
        }
    }
    fn discard_frontmost(&self, invocation: u64) {
        self.focus.borrow_mut().clear(invocation);
    }
    fn is_launcher_frontmost(&self) -> bool {
        NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .is_some_and(|app| app.processIdentifier() as u32 == std::process::id())
    }
    fn work_area(&self, cx: &App) -> Result<Bounds<Pixels>, DesktopError> {
        if let Some(mtm) = MainThreadMarker::new() {
            let screens = NSScreen::screens(mtm);
            if let Some(primary) = screens.firstObject() {
                let frame = primary.frame();
                let top = frame.origin.y + frame.size.height;
                let mouse = NSEvent::mouseLocation();
                let screen = screens
                    .iter()
                    .find(|screen| {
                        let frame = screen.frame();
                        mouse.x >= frame.origin.x
                            && mouse.x < frame.origin.x + frame.size.width
                            && mouse.y >= frame.origin.y
                            && mouse.y < frame.origin.y + frame.size.height
                    })
                    .or_else(|| NSScreen::mainScreen(mtm));
                if let Some(screen) = screen {
                    let rect = screen.visibleFrame();
                    return Ok(screen_bounds(
                        top,
                        [
                            rect.origin.x,
                            rect.origin.y,
                            rect.size.width,
                            rect.size.height,
                        ],
                    ));
                }
            }
        }
        cx.primary_display()
            .map(|display| display.visible_bounds())
            .ok_or(DesktopError::DisplayUnavailable)
    }
    fn shutdown(&self) {
        self.manager.borrow_mut().take();
        self.focus.borrow_mut().entry = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_lease_preserves_the_external_target() {
        let mut lease = FocusLease::default();
        lease.remember(1, Some(42));
        lease.remember(2, None);
        assert_eq!(lease.take(1, true), None);
        lease.clear(1);
        assert_eq!(lease.take(2, true), Some(42));
        assert_eq!(lease.take(2, true), None);
        lease.remember(3, Some(9));
        assert_eq!(lease.take(3, false), None);
        assert_eq!(lease.take(3, true), None);
    }
    #[test]
    fn screen_conversion_preserves_negative_origins_and_dock_clearance() {
        let bounds = screen_bounds(900., [-1200., 900., 1200., 800.]);
        assert_eq!(
            bounds.origin,
            gpui_kit::point(gpui_kit::px(-1200.), gpui_kit::px(-800.))
        );
        let bounds = screen_bounds(900., [0., 60., 1440., 816.]);
        assert_eq!(
            bounds.origin,
            gpui_kit::point(gpui_kit::px(0.), gpui_kit::px(24.))
        );
        assert_eq!(
            bounds.size,
            gpui_kit::size(gpui_kit::px(1440.), gpui_kit::px(816.))
        );
    }
}
