use super::{Answer, Draft, Run};
use crate::{
    Catalog, Diagnostic,
    catalog::{Step, StepKind},
    url::{ResolvedUrl, resolve},
    value::{CaptureName, CapturedValue},
};
use std::collections::BTreeMap;

pub(super) fn trace<'a>(
    catalog: &'a Catalog,
    run: &Run,
    mut visit: impl FnMut(&Step, &Answer),
) -> Option<&'a Step> {
    let mut step = &catalog.flow(run.flow)?.start;
    for record in run.answers.get(..run.cursor)? {
        if record.step != step.id {
            return None;
        }
        visit(step, &record.answer);
        step = match (&step.kind, &record.answer) {
            (StepKind::Input { next, .. }, Answer::Input(_)) => next,
            (StepKind::Select { selection, .. }, Answer::Choice(id)) => selection.next(*id)?,
            _ => return None,
        };
    }
    Some(step)
}
pub(super) fn current<'a>(catalog: &'a Catalog, run: &Run) -> Option<&'a Step> {
    trace(catalog, run, |_, _| ())
}
fn captures(catalog: &Catalog, run: &Run) -> Option<BTreeMap<CaptureName, CapturedValue>> {
    let mut values = BTreeMap::new();
    trace(catalog, run, |step, answer| match (&step.kind, answer) {
        (StepKind::Input { capture, .. }, Answer::Input(value)) => {
            values.insert(capture.clone(), value.clone());
        }
        (StepKind::Select { selection, .. }, Answer::Choice(id)) => {
            if let Some((name, value)) = selection
                .choices()
                .into_iter()
                .find(|c| c.id == *id)
                .and_then(|c| c.capture.as_ref())
            {
                values.insert(name.clone(), value.clone());
            }
        }
        _ => (),
    })?;
    Some(values)
}
pub(super) fn visit(catalog: &Catalog, run: &mut Run) -> Result<Option<ResolvedUrl>, Diagnostic> {
    let step = current(catalog, run).ok_or_else(|| super::session_diagnostic("E_ACTION_TARGET"))?;
    let recorded = run
        .answers
        .get(run.cursor)
        .filter(|r| r.step == step.id)
        .map(|r| &r.answer);
    run.draft = match &step.kind {
        StepKind::Input { .. } => Draft::Input {
            text: match recorded {
                Some(Answer::Input(v)) => v.0.clone(),
                _ => String::new(),
            },
            selected: None,
        },
        StepKind::Select { .. } => Draft::Selection {
            selected: match recorded {
                Some(Answer::Choice(id)) => Some(*id),
                _ => None,
            },
        },
        StepKind::Destination(template) => {
            run.draft = Draft::DestinationError;
            return resolve(
                &catalog
                    .flow(run.flow)
                    .ok_or_else(|| super::session_diagnostic("E_ACTION_TARGET"))?
                    .base,
                template,
                &captures(catalog, run)
                    .ok_or_else(|| super::session_diagnostic("E_ACTION_TARGET"))?,
            )
            .map(Some);
        }
    };
    Ok(None)
}
