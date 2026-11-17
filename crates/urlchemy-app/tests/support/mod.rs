use gpui_kit::{App, Bounds, Task, point, px, size};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};
use urlchemy_app::{DesktopError, DesktopServices, HotkeySignal, RegisteredHotkey};
use urlchemy_core::{Catalog, ResolvedUrl, Shortcut, config::load_bytes};

pub fn catalog() -> Rc<Catalog> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/superpowers/specs/yaml-url-contract/linear.yaml");
    Rc::new(load_bytes(&path, &std::fs::read(&path).unwrap()).unwrap())
}

#[derive(Default)]
pub struct Completion {
    result: Cell<Option<Result<(), DesktopError>>>,
    waker: RefCell<Option<Waker>>,
}
impl Completion {
    pub fn finish(&self, result: Result<(), DesktopError>) {
        self.result.set(Some(result));
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }
}
#[derive(Default)]
pub struct FakeDesktop {
    pub registration_fails: Cell<bool>,
    pub display_fails: Cell<bool>,
    pub frontmost: Cell<bool>,
    pub paths: Cell<usize>,
    pub configuration: RefCell<Option<PathBuf>>,
    pub registrations: RefCell<Vec<String>>,
    pub opened: RefCell<Vec<String>>,
    pub pending: RefCell<Vec<Rc<Completion>>>,
    pub remembered: RefCell<Vec<u64>>,
    pub restored: RefCell<Vec<u64>>,
    pub discarded: RefCell<Vec<u64>>,
    pub shutdowns: Cell<usize>,
    pub signals: Arc<Mutex<VecDeque<HotkeySignal>>>,
}
impl DesktopServices for FakeDesktop {
    fn configuration_path(&self) -> Result<PathBuf, DesktopError> {
        self.paths.set(self.paths.get() + 1);
        self.configuration
            .borrow()
            .clone()
            .ok_or(DesktopError::HomeUnavailable)
    }
    fn register_invocation(&self, shortcut: &Shortcut) -> Result<RegisteredHotkey, DesktopError> {
        self.registrations
            .borrow_mut()
            .push(shortcut.binding().into());
        if self.registration_fails.get() {
            return Err(DesktopError::RegistrationFailed);
        }
        let signals = self.signals.clone();
        Ok(RegisteredHotkey::new(
            7,
            Arc::new(move |_| signals.lock().unwrap().pop_front()),
        ))
    }
    fn open_url(&self, url: &ResolvedUrl, cx: &App) -> Task<Result<(), DesktopError>> {
        self.opened.borrow_mut().push(url.as_str().into());
        let completion = Rc::new(Completion::default());
        self.pending.borrow_mut().push(completion.clone());
        cx.spawn(async move |_| {
            std::future::poll_fn(move |cx| {
                if let Some(result) = completion.result.take() {
                    Poll::Ready(result)
                } else {
                    *completion.waker.borrow_mut() = Some(cx.waker().clone());
                    Poll::Pending
                }
            })
            .await
        })
    }
    fn remember_frontmost(&self, invocation: u64) {
        self.remembered.borrow_mut().push(invocation);
    }
    fn restore_frontmost(&self, invocation: u64) {
        self.restored.borrow_mut().push(invocation);
    }
    fn discard_frontmost(&self, invocation: u64) {
        self.discarded.borrow_mut().push(invocation);
    }
    fn is_launcher_frontmost(&self) -> bool {
        self.frontmost.get()
    }
    fn work_area(&self, _: &App) -> Result<Bounds<gpui_kit::Pixels>, DesktopError> {
        if self.display_fails.get() {
            Err(DesktopError::DisplayUnavailable)
        } else {
            Ok(Bounds::new(
                point(px(-100.), px(20.)),
                size(px(1200.), px(800.)),
            ))
        }
    }
    fn shutdown(&self) {
        self.shutdowns.set(self.shutdowns.get() + 1);
    }
}
