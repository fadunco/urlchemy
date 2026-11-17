#![cfg(target_os = "macos")]
use global_hotkey::hotkey::{Code, Modifiers};
use gpui_kit::TestAppContext;
use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use urlchemy_app::{HotkeyBridge, HotkeyLatch, HotkeySignal, native::native_hotkey};
use urlchemy_core::config::load_bytes;

#[test]
fn native_mapping_matches_every_allowed_key() {
    let letters = [
        Code::KeyA,
        Code::KeyB,
        Code::KeyC,
        Code::KeyD,
        Code::KeyE,
        Code::KeyF,
        Code::KeyG,
        Code::KeyH,
        Code::KeyI,
        Code::KeyJ,
        Code::KeyK,
        Code::KeyL,
        Code::KeyM,
        Code::KeyN,
        Code::KeyO,
        Code::KeyP,
        Code::KeyQ,
        Code::KeyR,
        Code::KeyS,
        Code::KeyT,
        Code::KeyU,
        Code::KeyV,
        Code::KeyW,
        Code::KeyX,
        Code::KeyY,
        Code::KeyZ,
    ];
    let digits = [
        Code::Digit0,
        Code::Digit1,
        Code::Digit2,
        Code::Digit3,
        Code::Digit4,
        Code::Digit5,
        Code::Digit6,
        Code::Digit7,
        Code::Digit8,
        Code::Digit9,
    ];
    let functions = [
        Code::F1,
        Code::F2,
        Code::F3,
        Code::F4,
        Code::F5,
        Code::F6,
        Code::F7,
        Code::F8,
        Code::F9,
        Code::F10,
        Code::F11,
        Code::F12,
    ];
    let cases = ('a'..='z')
        .map(|c| c.to_string())
        .zip(letters)
        .chain(('0'..='9').map(|c| c.to_string()).zip(digits))
        .chain((1..=12).map(|n| format!("f{n}")).zip(functions))
        .chain([("enter".into(), Code::Enter), ("space".into(), Code::Space)]);
    for (key, code) in cases {
        let text = format!("shortcuts: {{invoke: ctrl-{key}}}\nnavigation: []");
        let catalog = load_bytes(std::path::Path::new("fixture"), text.as_bytes()).unwrap();
        let hotkey = native_hotkey(catalog.shortcuts().invoke()).unwrap();
        assert_eq!(hotkey.key, code);
        assert_eq!(hotkey.mods, Modifiers::CONTROL);
    }
    let catalog = load_bytes(
        std::path::Path::new("fixture"),
        b"shortcuts: {invoke: shift-cmd-alt-ctrl-space}\nnavigation: []",
    )
    .unwrap();
    let hotkey = native_hotkey(catalog.shortcuts().invoke()).unwrap();
    assert_eq!(
        hotkey.mods,
        Modifiers::SHIFT | Modifiers::META | Modifiers::ALT | Modifiers::CONTROL
    );
    assert_eq!(hotkey.key, Code::Space);
}

#[test]
fn press_release_latch_ignores_repeats() {
    let mut latch = HotkeyLatch::new(7);
    assert!(latch.accept(HotkeySignal::new(7, true)));
    assert!(!latch.accept(HotkeySignal::new(7, true)));
    assert!(!latch.accept(HotkeySignal::new(8, false)));
    assert!(!latch.accept(HotkeySignal::new(7, true)));
    assert!(!latch.accept(HotkeySignal::new(7, false)));
    assert!(latch.accept(HotkeySignal::new(7, true)));
}

#[gpui_kit::test]
fn bridge_does_not_deliver_after_drop(cx: &mut TestAppContext) {
    let count = Rc::new(Cell::new(0));
    let ready = Arc::new(AtomicBool::new(true));
    let wait: urlchemy_app::HotkeyWait = Arc::new(move |_| {
        ready
            .swap(false, Ordering::SeqCst)
            .then_some(HotkeySignal::new(7, true))
    });
    let seen = count.clone();
    let bridge =
        cx.update(|cx| HotkeyBridge::start(wait.clone(), move |_, _| seen.set(seen.get() + 1), cx));
    drop(bridge);
    cx.run_until_parked();
    assert_eq!(count.get(), 0);
    let ready = Arc::new(AtomicBool::new(true));
    let wait = Arc::new(move |_| {
        ready
            .swap(false, Ordering::SeqCst)
            .then_some(HotkeySignal::new(7, true))
    });
    let seen = count.clone();
    let bridge =
        cx.update(|cx| HotkeyBridge::start(wait, move |_, _| seen.set(seen.get() + 1), cx));
    cx.run_until_parked();
    assert_eq!(count.get(), 1);
    drop(bridge);
    let seen = count.clone();
    let wait = Arc::new(move |_| Some(HotkeySignal::new(7, true)));
    let bridge =
        cx.update(|cx| HotkeyBridge::start(wait, move |_, _| seen.set(seen.get() + 1), cx));
    drop(bridge);
    cx.run_until_parked();
    assert_eq!(count.get(), 1);
}
