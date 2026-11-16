use crate::diagnostic::Origin;

#[derive(Clone, Debug)]
pub(crate) struct Field<T> {
    pub value: T,
    pub origin: Origin,
}
#[derive(Debug)]
pub(super) struct RawConfig {
    pub origin: Origin,
    pub navigation: Vec<RawEntry>,
    pub shortcuts: Option<RawShortcuts>,
}
#[derive(Debug)]
pub(super) struct RawShortcuts {
    pub invoke: Option<Field<String>>,
    pub launch: Option<Field<String>>,
}
#[derive(Debug)]
pub(super) enum RawEntry {
    Folder(Box<RawFolder>),
    Flow(Box<RawFlow>),
}
#[derive(Debug)]
pub(super) struct RawPresentation {
    pub label: Field<String>,
    pub emoji: Option<Field<String>>,
    pub alias: Option<Field<String>>,
}
#[derive(Debug)]
pub(super) struct RawFolder {
    pub presentation: RawPresentation,
    pub children: Vec<RawEntry>,
}
#[derive(Debug)]
pub(super) struct RawFlow {
    pub id: Field<String>,
    pub presentation: RawPresentation,
    pub direct_alias: Option<Field<String>>,
    pub base: Field<String>,
    pub start: RawStep,
}
#[derive(Debug)]
pub(super) struct RawStep {
    pub origin: Origin,
    pub kind: RawStepKind,
}
#[derive(Debug)]
pub(super) enum RawStepKind {
    Input {
        label: Field<String>,
        capture: Field<String>,
        suggestions: Vec<RawSuggestion>,
        next: Box<RawStep>,
    },
    Select {
        label: Field<String>,
        capture: Option<Field<String>>,
        options: Vec<RawChoice>,
        next: Option<Box<RawStep>>,
    },
    Destination {
        template: Field<String>,
    },
}
#[derive(Debug)]
pub(super) struct RawSuggestion {
    pub label: Field<String>,
    pub value: Field<String>,
}
#[derive(Debug)]
pub(super) struct RawChoice {
    pub origin: Origin,
    pub label: Field<String>,
    pub value: Option<Field<String>>,
    pub next: Option<Box<RawStep>>,
    pub next_key: Option<Origin>,
}
