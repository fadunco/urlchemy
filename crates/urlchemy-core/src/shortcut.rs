/// A key in a validated configuration shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutKey {
    Character(char),
    Enter,
    Space,
    Function(u8),
}

/// The modifiers in a shortcut, independent of any native registration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShortcutModifiers {
    control: bool,
    alt: bool,
    shift: bool,
    command: bool,
}

impl ShortcutModifiers {
    pub fn is_control(&self) -> bool {
        self.control
    }
    pub fn is_alt(&self) -> bool {
        self.alt
    }
    pub fn is_shift(&self) -> bool {
        self.shift
    }
    pub fn is_command(&self) -> bool {
        self.command
    }
}

/// One validated key combination, exposed as immutable catalog metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    modifiers: ShortcutModifiers,
    key: ShortcutKey,
    binding: String,
}

/// Validated invocation and confirmation bindings stored with the catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcuts {
    invoke: Shortcut,
    launch: Shortcut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShortcutRole {
    Invoke,
    Launch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShortcutError {
    Syntax,
    Conflict,
}

impl Shortcut {
    pub fn modifiers(&self) -> ShortcutModifiers {
        self.modifiers
    }
    pub fn key(&self) -> ShortcutKey {
        self.key
    }
    /// Returns the binding with modifiers ordered as `ctrl-alt-shift-cmd`.
    /// This value does not establish system-wide availability.
    pub fn binding(&self) -> &str {
        &self.binding
    }

    pub(crate) fn parse(text: &str, role: ShortcutRole) -> Result<Self, ShortcutError> {
        let mut parts = text.split('-').peekable();
        let mut modifiers = ShortcutModifiers::default();
        let mut key_text = "";
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                key_text = part;
                break;
            }
            let flag = match part {
                "ctrl" => &mut modifiers.control,
                "alt" => &mut modifiers.alt,
                "shift" => &mut modifiers.shift,
                "cmd" => &mut modifiers.command,
                _ => return Err(ShortcutError::Syntax),
            };
            if std::mem::replace(flag, true) {
                return Err(ShortcutError::Syntax);
            }
        }
        let key = match key_text {
            "enter" => ShortcutKey::Enter,
            "space" => ShortcutKey::Space,
            s if s.len() == 1 && s.as_bytes()[0].is_ascii_lowercase() => {
                ShortcutKey::Character(s.as_bytes()[0] as char)
            }
            s if s.len() == 1 && s.as_bytes()[0].is_ascii_digit() => {
                ShortcutKey::Character(s.as_bytes()[0] as char)
            }
            s => {
                let number = s.strip_prefix('f').ok_or(ShortcutError::Syntax)?;
                let number = number.parse::<u8>().map_err(|_| ShortcutError::Syntax)?;
                if !(1..=12).contains(&number) || s != format!("f{number}") {
                    return Err(ShortcutError::Syntax);
                }
                ShortcutKey::Function(number)
            }
        };
        if !modifiers.control
            && !modifiers.command
            && !(role == ShortcutRole::Launch
                && key == ShortcutKey::Enter
                && modifiers == ShortcutModifiers::default())
        {
            return Err(ShortcutError::Syntax);
        }
        let mut binding = String::new();
        for (enabled, name) in [
            (modifiers.control, "ctrl"),
            (modifiers.alt, "alt"),
            (modifiers.shift, "shift"),
            (modifiers.command, "cmd"),
        ] {
            if enabled {
                binding.push_str(name);
                binding.push('-');
            }
        }
        binding.push_str(key_text);
        if matches!(
            binding.as_str(),
            "cmd-a"
                | "cmd-c"
                | "cmd-v"
                | "cmd-x"
                | "cmd-z"
                | "shift-cmd-z"
                | "cmd-q"
                | "cmd-w"
                | "cmd-h"
                | "alt-cmd-h"
                | "cmd-m"
                | "cmd-1"
                | "cmd-2"
                | "cmd-3"
                | "cmd-4"
                | "cmd-5"
                | "cmd-6"
                | "cmd-7"
                | "cmd-8"
                | "cmd-9"
        ) {
            return Err(ShortcutError::Conflict);
        }
        Ok(Self {
            modifiers,
            key,
            binding,
        })
    }
}

impl Shortcuts {
    pub fn invoke(&self) -> &Shortcut {
        &self.invoke
    }
    pub fn launch(&self) -> &Shortcut {
        &self.launch
    }
    pub(crate) fn new(invoke: Shortcut, launch: Shortcut) -> Result<Self, ShortcutError> {
        if invoke == launch {
            return Err(ShortcutError::Conflict);
        }
        Ok(Self { invoke, launch })
    }
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            invoke: Shortcut {
                modifiers: ShortcutModifiers {
                    control: true,
                    alt: true,
                    ..Default::default()
                },
                key: ShortcutKey::Space,
                binding: "ctrl-alt-space".into(),
            },
            launch: Shortcut {
                modifiers: ShortcutModifiers::default(),
                key: ShortcutKey::Enter,
                binding: "enter".into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_canonical_bindings() {
        let defaults = Shortcuts::default();
        assert_eq!(defaults.invoke().binding(), "ctrl-alt-space");
        assert_eq!(defaults.launch().binding(), "enter");
        assert_eq!(defaults.invoke().key(), ShortcutKey::Space);
        assert!(defaults.invoke().modifiers().is_control());
        assert!(defaults.invoke().modifiers().is_alt());
        assert!(!defaults.invoke().modifiers().is_shift());
        assert!(!defaults.invoke().modifiers().is_command());
        assert_eq!(
            Shortcut::parse("alt-ctrl-space", ShortcutRole::Invoke).unwrap(),
            *defaults.invoke()
        );
        let binding = Shortcut::parse("cmd-shift-alt-ctrl-k", ShortcutRole::Invoke).unwrap();
        assert_eq!(binding.binding(), "ctrl-alt-shift-cmd-k");
        assert_eq!(binding.key(), ShortcutKey::Character('k'));
        let modifiers = binding.modifiers();
        assert!(modifiers.is_control());
        assert!(modifiers.is_alt());
        assert!(modifiers.is_shift());
        assert!(modifiers.is_command());
        assert_eq!(
            Shortcut::parse("cmd-enter", ShortcutRole::Launch)
                .unwrap()
                .key(),
            ShortcutKey::Enter
        );
        assert_eq!(
            Shortcut::parse("ctrl-f12", ShortcutRole::Invoke)
                .unwrap()
                .key(),
            ShortcutKey::Function(12)
        );
        assert_eq!(
            Shortcuts::new(
                Shortcut::parse("ctrl-alt-k", ShortcutRole::Invoke).unwrap(),
                Shortcut::parse("alt-ctrl-k", ShortcutRole::Launch).unwrap()
            ),
            Err(ShortcutError::Conflict)
        );
        assert!(
            Shortcuts::new(
                defaults.invoke().clone(),
                Shortcut::parse("cmd-enter", ShortcutRole::Launch).unwrap()
            )
            .is_ok()
        );
    }

    #[test]
    fn bounded_shortcut_syntax() {
        for role in [ShortcutRole::Invoke, ShortcutRole::Launch] {
            for text in [
                "",
                " enter",
                "enter ",
                "CMD-enter",
                "ctrl-ctrl-k",
                "ctrl-k ctrl-l",
                "cmd",
                "cmd-",
                "fn-enter",
                "super-enter",
                "cmd-k->x",
                "ctrl-é",
                "ctrl-f0",
                "ctrl-f13",
                "ctrl-f01",
                "cmd-escape",
                "cmd-[",
                "space",
                "a",
                "f1",
                "shift-enter",
                "alt-enter",
                "ctrl--k",
                "k-ctrl",
                "ctrl-enter\n",
                "ctrl-\tenter",
                "ctrl-ENTER",
                "ctrl-k-alt",
            ] {
                assert_eq!(
                    Shortcut::parse(text, role),
                    Err(ShortcutError::Syntax),
                    "{text:?}"
                );
            }
            for key in ('a'..='z')
                .chain('0'..='9')
                .map(|c| c.to_string())
                .chain(["enter".to_owned(), "space".to_owned()])
                .chain((1..=12).map(|n| format!("f{n}")))
            {
                assert!(
                    Shortcut::parse(&format!("ctrl-{key}"), role).is_ok(),
                    "{key}"
                );
            }
        }
        assert_eq!(
            Shortcut::parse("enter", ShortcutRole::Invoke),
            Err(ShortcutError::Syntax)
        );
        assert!(Shortcut::parse("enter", ShortcutRole::Launch).is_ok());
    }

    #[test]
    fn reserved_shortcut_combinations() {
        for role in [ShortcutRole::Invoke, ShortcutRole::Launch] {
            for text in [
                "cmd-a",
                "cmd-c",
                "cmd-v",
                "cmd-x",
                "cmd-z",
                "cmd-shift-z",
                "cmd-q",
                "cmd-w",
                "cmd-h",
                "cmd-alt-h",
                "cmd-m",
                "alt-cmd-h",
                "shift-cmd-z",
            ]
            .into_iter()
            .map(str::to_owned)
            .chain((1..=9).map(|n| format!("cmd-{n}")))
            {
                assert_eq!(
                    Shortcut::parse(&text, role),
                    Err(ShortcutError::Conflict),
                    "{text}"
                );
            }
            for text in ["cmd-0", "cmd-shift-1", "ctrl-f12", "cmd-enter"] {
                assert!(Shortcut::parse(text, role).is_ok(), "{text}");
            }
        }
    }
}
