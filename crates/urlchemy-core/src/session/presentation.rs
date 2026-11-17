use super::{Answer, Draft, RowId, Session, SessionSnapshot, State, history, navigation};
use crate::{
    Catalog, Diagnostic, Entry, FlowId,
    catalog::{Step, StepKind},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimaryActionHint {
    OpenFolder,
    StartFlow,
    Continue,
    OpenUrl,
    ChooseOption,
    Unavailable,
}

/// Readonly labels and action hints without captured input values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionPresentation {
    context: Vec<String>,
    primary_action: PrimaryActionHint,
    can_go_back: bool,
}
impl SessionPresentation {
    pub fn context(&self) -> &[String] {
        &self.context
    }
    pub fn primary_action(&self) -> PrimaryActionHint {
        self.primary_action
    }
    pub fn can_go_back(&self) -> bool {
        self.can_go_back
    }
}
fn flow_context(entries: &[Entry], target: FlowId) -> Option<Vec<String>> {
    for entry in entries {
        if entry.flow() == Some(target) {
            return Some(vec![entry.label().into()]);
        }
        if let Some(children) = entry.children()
            && let Some(mut labels) = flow_context(children, target)
        {
            labels.insert(0, entry.label().into());
            return Some(labels);
        }
    }
    None
}
fn continuation(step: &Step) -> PrimaryActionHint {
    if matches!(step.kind, StepKind::Destination(_)) {
        PrimaryActionHint::OpenUrl
    } else {
        PrimaryActionHint::Continue
    }
}
impl Session {
    pub fn presentation(
        &self,
        catalog: &Catalog,
        active: Option<RowId>,
    ) -> Result<SessionPresentation, Diagnostic> {
        let snapshot = self.snapshot(catalog)?;
        let rows = match &snapshot {
            SessionSnapshot::Browsing { rows, .. }
            | SessionSnapshot::Input { rows, .. }
            | SessionSnapshot::Selection { rows, .. } => rows.as_slice(),
            _ => &[],
        };
        let invalid = || self.error(catalog, "E_ACTION_TARGET");
        if active.is_some_and(|id| !rows.iter().any(|row| row.id() == id)) {
            return Err(invalid());
        }
        let mut result = SessionPresentation {
            context: Vec::new(),
            primary_action: PrimaryActionHint::Unavailable,
            can_go_back: false,
        };
        match &self.state {
            State::Browsing(path) => {
                for (ix, id) in path.iter().enumerate() {
                    let entry = navigation::entries(catalog, &path[..ix])
                        .and_then(|entries| entries.iter().find(|entry| entry.id() == *id))
                        .ok_or_else(invalid)?;
                    result.context.push(entry.label().into());
                }
                result.can_go_back = !path.is_empty();
                if let Some(RowId::Entry(id)) = active {
                    let entry = navigation::entries(catalog, path)
                        .and_then(|entries| entries.iter().find(|entry| entry.id() == id))
                        .ok_or_else(invalid)?;
                    result.primary_action = if entry.children().is_some() {
                        PrimaryActionHint::OpenFolder
                    } else {
                        let flow = entry
                            .flow()
                            .and_then(|id| catalog.flow(id))
                            .ok_or_else(invalid)?;
                        if matches!(flow.start.kind, StepKind::Destination(_)) {
                            PrimaryActionHint::OpenUrl
                        } else {
                            PrimaryActionHint::StartFlow
                        }
                    };
                }
            }
            State::Running(run) => {
                result.context = flow_context(catalog.roots(), run.flow).ok_or_else(invalid)?;
                let step = history::trace(catalog, run, |step, answer| {
                    if let (StepKind::Select { selection, .. }, Answer::Choice(id)) =
                        (&step.kind, answer)
                        && let Some(choice) = selection
                            .choices()
                            .into_iter()
                            .find(|choice| choice.id == *id)
                    {
                        result.context.push(choice.label.clone());
                    }
                })
                .ok_or_else(invalid)?;
                result.can_go_back = true;
                result.primary_action = match &step.kind {
                    StepKind::Input { next, .. } => continuation(next),
                    StepKind::Select { selection, .. } => {
                        let selected = if let Some(RowId::Choice(id)) = active {
                            Some(id)
                        } else if let Draft::Selection { selected } = &run.draft {
                            *selected
                        } else {
                            None
                        };
                        selected
                            .map(|id| selection.next(id).map(continuation).ok_or_else(invalid))
                            .transpose()?
                            .unwrap_or(PrimaryActionHint::ChooseOption)
                    }
                    StepKind::Destination(_) => PrimaryActionHint::Unavailable,
                };
            }
            _ => (),
        }
        Ok(result)
    }
}
