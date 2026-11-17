use crate::Launcher;
use urlchemy_core::{Entry, PrimaryActionHint as Hint, RowId, SessionSnapshot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PaletteSurface {
    Navigation,
    Input,
    Selection,
    DestinationError,
    OpeningPending,
    OpeningFailed,
    Inactive,
}

pub(super) struct PalettePresentation {
    surface: PaletteSurface,
    context: Vec<String>,
    field_label: Option<String>,
    primary_label: Option<String>,
    primary_enabled: bool,
    body_message: Option<String>,
    error_message: Option<String>,
    commitment_hint: Option<String>,
    can_go_back: bool,
}
impl PalettePresentation {
    pub(super) fn derive(launcher: &Launcher) -> Self {
        let metadata = launcher.presentation().ok();
        let mut p = Self {
            surface: PaletteSurface::Inactive,
            context: metadata
                .as_ref()
                .map(|p| p.context().to_vec())
                .unwrap_or_default(),
            field_label: None,
            primary_label: None,
            primary_enabled: false,
            body_message: None,
            error_message: launcher.error().map(|error| error.message().into()),
            commitment_hint: None,
            can_go_back: metadata.as_ref().is_some_and(|p| p.can_go_back()),
        };
        if launcher.is_opening() {
            p.surface = PaletteSurface::OpeningPending;
            p.body_message = Some("Opening in your default browser.".into());
            p.can_go_back = false;
            return p;
        }
        if let Some(error) = launcher.opening_error() {
            p.surface = PaletteSurface::OpeningFailed;
            p.error_message = Some(error.to_string());
            p.primary_label = Some("Retry".into());
            p.primary_enabled = true;
            p.can_go_back = false;
            return p;
        }
        let Ok(snapshot) = launcher.snapshot() else {
            p.body_message = Some("Could not display the launcher.".into());
            return p;
        };
        let hint = metadata
            .map(|p| p.primary_action())
            .unwrap_or(Hint::Unavailable);
        match snapshot {
            SessionSnapshot::Browsing { folders, .. } => {
                p.surface = PaletteSurface::Navigation;
                p.field_label = Some("Open a flow".into());
                p.primary_enabled = launcher.active().is_some();
                p.primary_label = Some(
                    match hint {
                        Hint::OpenFolder => "Open folder",
                        Hint::StartFlow => "Start flow",
                        Hint::OpenUrl => "Open URL",
                        _ => "Continue",
                    }
                    .into(),
                );
                if launcher.rows().is_empty() {
                    p.body_message = Some(
                        if launcher.query().is_empty()
                            && folders.is_empty()
                            && launcher.catalog().roots().is_empty()
                        {
                            p.commitment_hint =
                                Some("Edit the configuration file. Restart URLchemy.".into());
                            "No flows configured"
                        } else if launcher.query().is_empty() && !folders.is_empty() {
                            "No entries in this folder"
                        } else {
                            "No matches"
                        }
                        .into(),
                    );
                }
            }
            SessionSnapshot::Selection { label, .. } => {
                p.surface = PaletteSurface::Selection;
                p.field_label = Some(label);
                p.primary_enabled = launcher.active().is_some();
                p.primary_label = Some(
                    if hint == Hint::OpenUrl {
                        "Open URL"
                    } else {
                        "Continue"
                    }
                    .into(),
                );
                if launcher.rows().is_empty() {
                    p.body_message = Some("No matches".into());
                }
            }
            SessionSnapshot::Input {
                label, selected, ..
            } => {
                p.surface = PaletteSurface::Input;
                p.field_label = Some(label);
                p.primary_enabled = true;
                let selected_label = selected
                    .and_then(|id| {
                        launcher
                            .rows()
                            .iter()
                            .find(|row| row.id() == RowId::Suggestion(id))
                    })
                    .map(|row| row.label());
                p.primary_label = Some(if hint == Hint::OpenUrl {
                    p.commitment_hint = Some(
                        selected_label
                            .map(|label| format!("Using {label}"))
                            .unwrap_or_else(|| "Opens in your default browser.".into()),
                    );
                    "Open URL".into()
                } else if let Some(label) = selected_label {
                    format!("Use {label}")
                } else if launcher.query().is_empty() {
                    "Continue".into()
                } else {
                    format!("Use “{}”", launcher.query())
                });
                if launcher.rows().is_empty() {
                    p.body_message = Some("No suggestions. You can use the text you enter.".into());
                }
            }
            SessionSnapshot::DestinationError { .. } => {
                p.surface = PaletteSurface::DestinationError
            }
            _ => (),
        }
        p
    }
    pub(super) fn surface(&self) -> PaletteSurface {
        self.surface
    }
    pub(super) fn context(&self) -> &[String] {
        &self.context
    }
    pub(super) fn field_label(&self) -> Option<&str> {
        self.field_label.as_deref()
    }
    pub(super) fn primary_label(&self) -> Option<&str> {
        self.primary_label.as_deref()
    }
    pub(super) fn primary_enabled(&self) -> bool {
        self.primary_enabled
    }
    pub(super) fn body_message(&self) -> Option<&str> {
        self.body_message.as_deref()
    }
    pub(super) fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
    pub(super) fn commitment_hint(&self) -> Option<&str> {
        self.commitment_hint.as_deref()
    }
    pub(super) fn can_go_back(&self) -> bool {
        self.can_go_back
    }
}

#[derive(Clone)]
pub(super) struct RowDecoration {
    emoji: Option<String>,
    alias: Option<String>,
    is_folder: bool,
}
impl RowDecoration {
    pub(super) fn from_entry(entry: &Entry) -> Self {
        Self {
            emoji: entry.emoji().map(str::to_owned),
            alias: entry.alias().map(str::to_owned),
            is_folder: entry.children().is_some(),
        }
    }
    pub(super) fn emoji(&self) -> Option<&str> {
        self.emoji.as_deref()
    }
    pub(super) fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }
    pub(super) fn is_folder(&self) -> bool {
        self.is_folder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DesktopError, Launcher, PaletteCommand as Cmd, PaletteEffect};
    use std::{path::Path, rc::Rc};
    use urlchemy_core::{RowId, config::load_bytes};

    fn source(source: &str) -> Launcher {
        Launcher::new(
            Rc::new(load_bytes(Path::new("fixture"), source.as_bytes()).unwrap()),
            1,
        )
    }
    fn fixture(name: &str) -> Launcher {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/superpowers/specs/yaml-url-contract")
            .join(name);
        source(&std::fs::read_to_string(path).unwrap())
    }
    fn input() -> Launcher {
        let mut launcher = fixture("branching.yaml");
        launcher.dispatch(Cmd::Edited("open-dd".into()));
        launcher.dispatch(Cmd::Confirm);
        launcher
    }

    #[test]
    fn empty_and_filtered_feedback_is_mode_specific() {
        let launcher = source("navigation: []");
        let p = PalettePresentation::derive(&launcher);
        assert_eq!(p.surface(), PaletteSurface::Navigation);
        assert_eq!(p.body_message(), Some("No flows configured"));
        assert_eq!(
            p.commitment_hint(),
            Some("Edit the configuration file. Restart URLchemy.")
        );
        assert!(!p.primary_enabled());
        let mut launcher =
            source("navigation:\n  - folder:\n      label: Empty\n      children: []");
        launcher.dispatch(Cmd::Confirm);
        assert_eq!(
            PalettePresentation::derive(&launcher).body_message(),
            Some("No entries in this folder")
        );
        let mut launcher = fixture("fixed.yaml");
        launcher.dispatch(Cmd::Edited("absent".into()));
        assert_eq!(
            PalettePresentation::derive(&launcher).body_message(),
            Some("No matches")
        );
        let mut launcher = fixture("common.yaml");
        launcher.dispatch(Cmd::Confirm);
        assert_eq!(
            PalettePresentation::derive(&launcher).surface(),
            PaletteSurface::Selection
        );
        launcher.dispatch(Cmd::Edited("absent".into()));
        assert!(!PalettePresentation::derive(&launcher).primary_enabled());
        let mut launcher = input();
        launcher.dispatch(Cmd::Edited("unmatched".into()));
        let p = PalettePresentation::derive(&launcher);
        assert_eq!(p.surface(), PaletteSurface::Input);
        assert_eq!(p.field_label(), Some("Source application"));
        assert_eq!(
            p.body_message(),
            Some("No suggestions. You can use the text you enter.")
        );
        assert!(p.primary_enabled());
        launcher.dispatch(Cmd::Edited("\n".into()));
        launcher.dispatch(Cmd::Confirm);
        assert_eq!(
            PalettePresentation::derive(&launcher).error_message(),
            Some("Remove control characters.")
        );
    }

    #[test]
    fn opening_state_overrides_completed_snapshot() {
        let mut launcher = fixture("fixed.yaml");
        launcher.dispatch(Cmd::Edited("service-status".into()));
        // The exact alias resolves and its effect is returned by this dispatch.
        launcher.dispatch(Cmd::StartOver);
        let Some(PaletteEffect::Open {
            invocation,
            operation,
            ..
        }) = launcher.dispatch(Cmd::Edited("service-status".into()))
        else {
            panic!("No opening effect");
        };
        let p = PalettePresentation::derive(&launcher);
        assert_eq!(p.surface(), PaletteSurface::OpeningPending);
        assert_eq!(p.field_label(), None);
        assert!(!p.primary_enabled());
        launcher.dispatch(Cmd::OpenFinished {
            invocation,
            operation,
            result: Err(DesktopError::OpenFailed),
        });
        let p = PalettePresentation::derive(&launcher);
        assert_eq!(p.surface(), PaletteSurface::OpeningFailed);
        assert_eq!(p.primary_label(), Some("Retry"));
        assert!(p.primary_enabled());
        assert_eq!(p.error_message(), Some("Could not open the browser."));
        let mut launcher = fixture("linear.yaml");
        launcher.dispatch(Cmd::Confirm);
        launcher.dispatch(Cmd::Edited("jobs".into()));
        launcher.dispatch(Cmd::Confirm);
        launcher.dispatch(Cmd::Edited("..".into()));
        launcher.dispatch(Cmd::Confirm);
        let p = PalettePresentation::derive(&launcher);
        assert_eq!(p.surface(), PaletteSurface::DestinationError);
        assert_eq!(p.field_label(), None);
        assert_eq!(p.primary_label(), None);
        assert!(p.can_go_back());
        launcher.dispatch(Cmd::Dismiss(crate::CloseReason::Explicit));
        assert_eq!(
            PalettePresentation::derive(&launcher).surface(),
            PaletteSurface::Inactive
        );
    }

    #[test]
    fn commitment_labels_follow_explicit_selection() {
        let mut launcher = input();
        assert_eq!(
            PalettePresentation::derive(&launcher).primary_label(),
            Some("Continue")
        );
        launcher.dispatch(Cmd::Edited("acc".into()));
        assert_eq!(
            PalettePresentation::derive(&launcher).primary_label(),
            Some("Use “acc”")
        );
        launcher.dispatch(Cmd::Activate(launcher.rows()[0].id()));
        let p = PalettePresentation::derive(&launcher);
        assert_eq!(p.primary_label(), Some("Use Accounts"));
        assert_eq!(launcher.query(), "acc");
        launcher.dispatch(Cmd::Confirm);
        let p = PalettePresentation::derive(&launcher);
        assert_eq!(p.primary_label(), Some("Open URL"));
        assert!(!p.context().iter().any(|label| label.contains("accounts")));
        let mut launcher = source(
            "navigation:\n  - flow:\n      id: one\n      label: One\n      baseURL: https://example.com\n      start:\n        input:\n          label: Account\n          capture: account\n          suggestions:\n            - label: Accounts\n              value: internal-value\n          next:\n            destination:\n              template: '/{account}'",
        );
        launcher.dispatch(Cmd::Confirm);
        launcher.dispatch(Cmd::Activate(launcher.rows()[0].id()));
        let p = PalettePresentation::derive(&launcher);
        assert_eq!(p.primary_label(), Some("Open URL"));
        assert_eq!(p.commitment_hint(), Some("Using Accounts"));
    }

    #[test]
    fn navigation_metadata_is_current_context_only() {
        let mut launcher = fixture("fixed.yaml");
        let RowId::Entry(root) = launcher.rows()[0].id() else {
            panic!()
        };
        let other = fixture("fixed.yaml");
        let RowId::Entry(foreign) = other.rows()[0].id() else {
            panic!()
        };
        assert!(launcher.navigation_entry(foreign).is_none());
        let entry = launcher.navigation_entry(root).unwrap();
        let d = RowDecoration::from_entry(entry);
        assert_eq!(d.emoji(), Some("🧰"));
        assert_eq!(d.alias(), Some("tools"));
        assert!(d.is_folder());
        launcher.dispatch(Cmd::Confirm);
        assert!(launcher.navigation_entry(root).is_none());
        let RowId::Entry(flow) = launcher.rows()[0].id() else {
            panic!()
        };
        assert!(!RowDecoration::from_entry(launcher.navigation_entry(flow).unwrap()).is_folder());
        launcher.dispatch(Cmd::Confirm);
        assert!(launcher.navigation_entry(flow).is_none());
    }
}
