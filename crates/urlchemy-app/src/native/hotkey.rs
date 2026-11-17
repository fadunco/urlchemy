use crate::DesktopError;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use urlchemy_core::{Shortcut, ShortcutKey};

pub fn native_hotkey(shortcut: &Shortcut) -> Result<HotKey, DesktopError> {
    let modifiers = shortcut.modifiers();
    let mut mods = Modifiers::empty();
    for (enabled, flag) in [
        (modifiers.is_control(), Modifiers::CONTROL),
        (modifiers.is_alt(), Modifiers::ALT),
        (modifiers.is_shift(), Modifiers::SHIFT),
        (modifiers.is_command(), Modifiers::META),
    ] {
        if enabled {
            mods |= flag;
        }
    }
    let key = match shortcut.key() {
        ShortcutKey::Enter => Code::Enter,
        ShortcutKey::Space => Code::Space,
        ShortcutKey::Character(c) if c.is_ascii_lowercase() => [
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
        ][c as usize - 'a' as usize],
        ShortcutKey::Character(c) if c.is_ascii_digit() => [
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
        ][c as usize - '0' as usize],
        ShortcutKey::Function(n) => *[
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
        ]
        .get(usize::from(n).wrapping_sub(1))
        .ok_or(DesktopError::RegistrationFailed)?,
        _ => return Err(DesktopError::RegistrationFailed),
    };
    Ok(HotKey::new(Some(mods), key))
}
