use crate::{HotkeySignal, HotkeyWait};
use gpui_kit::{App, Task};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub struct HotkeyLatch {
    id: u32,
    down: bool,
}
impl HotkeyLatch {
    pub fn new(id: u32) -> Self {
        Self { id, down: false }
    }
    pub fn accept(&mut self, signal: HotkeySignal) -> bool {
        if signal.id() != self.id {
            return false;
        }
        let pressed = signal.is_pressed() && !self.down;
        self.down = signal.is_pressed();
        pressed
    }
}
pub struct HotkeyBridge {
    cancel: Arc<AtomicBool>,
    _task: Task<()>,
}
impl HotkeyBridge {
    pub fn start(
        wait: HotkeyWait,
        mut on_signal: impl FnMut(HotkeySignal, &mut App) + 'static,
        cx: &App,
    ) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let stopped = cancel.clone();
        let background = cx.background_executor().clone();
        let task = cx.spawn(async move |cx| {
            while !stopped.load(Ordering::Acquire) {
                let wait = wait.clone();
                let stop = stopped.clone();
                let signal = background
                    .spawn(async move {
                        if stop.load(Ordering::Acquire) {
                            None
                        } else {
                            wait(Duration::from_millis(100))
                        }
                    })
                    .await;
                if stopped.load(Ordering::Acquire) {
                    break;
                }
                if let Some(signal) = signal {
                    cx.update(|cx| on_signal(signal, cx));
                } else {
                    background.timer(Duration::from_millis(100)).await;
                }
            }
        });
        Self {
            cancel,
            _task: task,
        }
    }
}
impl Drop for HotkeyBridge {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
    }
}
