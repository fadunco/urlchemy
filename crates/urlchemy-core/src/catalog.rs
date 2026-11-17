use crate::{
    Shortcuts,
    diagnostic::Origin,
    url::{CompiledTemplate, FixedBase},
    value::{CaptureName, CapturedValue},
};

macro_rules! identifier {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            pub(crate) owner: u64,
            pub(crate) index: u32,
        }
    };
}
identifier!(EntryId);
identifier!(FlowId);
identifier!(StepId);
identifier!(ChoiceId);
identifier!(SuggestionId);

#[derive(Debug)]
pub struct Catalog {
    pub(crate) owner: u64,
    pub(crate) roots: Vec<Entry>,
    pub(crate) flows: Vec<Flow>,
    pub(crate) shortcuts: Shortcuts,
}
#[derive(Debug)]
pub struct Entry {
    pub(crate) id: EntryId,
    pub(crate) presentation: Presentation,
    pub(crate) kind: EntryKind,
}
#[derive(Debug)]
pub(crate) enum EntryKind {
    Folder(Vec<Entry>),
    Flow(FlowId),
}
#[derive(Debug)]
pub(crate) struct Presentation {
    pub label: String,
    pub emoji: Option<String>,
    pub alias: Option<String>,
}
#[derive(Debug)]
pub(crate) struct Flow {
    pub id: FlowId,
    pub key: String,
    pub direct_alias: Option<String>,
    pub base: FixedBase,
    pub start: Step,
}
#[derive(Debug)]
pub(crate) struct Step {
    pub id: StepId,
    pub origin: Origin,
    pub kind: StepKind,
}
#[derive(Debug)]
pub(crate) enum StepKind {
    Input {
        label: String,
        capture: CaptureName,
        suggestions: Vec<Suggestion>,
        next: Box<Step>,
    },
    Select {
        label: String,
        selection: Selection,
    },
    Destination(CompiledTemplate),
}
#[derive(Debug)]
pub(crate) enum Selection {
    Common {
        choices: Vec<Choice>,
        next: Box<Step>,
    },
    Branched(Vec<BranchChoice>),
}
#[derive(Debug)]
pub(crate) struct Choice {
    pub id: ChoiceId,
    pub label: String,
    pub capture: Option<(CaptureName, CapturedValue)>,
}
#[derive(Debug)]
pub(crate) struct BranchChoice {
    pub choice: Choice,
    pub next: Box<Step>,
}
#[derive(Debug)]
pub(crate) struct Suggestion {
    pub id: SuggestionId,
    pub label: String,
    pub value: CapturedValue,
}

impl Entry {
    pub fn alias(&self) -> Option<&str> {
        self.presentation.alias.as_deref()
    }
    pub fn id(&self) -> EntryId {
        self.id
    }
    pub fn label(&self) -> &str {
        &self.presentation.label
    }
    pub fn emoji(&self) -> Option<&str> {
        self.presentation.emoji.as_deref()
    }
    pub fn children(&self) -> Option<&[Entry]> {
        if let EntryKind::Folder(entries) = &self.kind {
            Some(entries)
        } else {
            None
        }
    }
    pub fn flow(&self) -> Option<FlowId> {
        if let EntryKind::Flow(id) = self.kind {
            Some(id)
        } else {
            None
        }
    }
}
impl Catalog {
    /// Returns configured bindings, not registered system shortcuts.
    pub fn shortcuts(&self) -> &Shortcuts {
        &self.shortcuts
    }
    pub fn roots(&self) -> &[Entry] {
        &self.roots
    }
    pub fn flow_by_key(&self, key: &str) -> Option<FlowId> {
        self.flows.iter().find(|f| f.key == key).map(|f| f.id)
    }
    pub(crate) fn flow(&self, id: FlowId) -> Option<&Flow> {
        self.flows.iter().find(|f| f.id == id)
    }
}
impl Selection {
    pub fn choices(&self) -> Vec<&Choice> {
        match self {
            Self::Common { choices, .. } => choices.iter().collect(),
            Self::Branched(choices) => choices.iter().map(|b| &b.choice).collect(),
        }
    }
    pub fn next(&self, id: ChoiceId) -> Option<&Step> {
        match self {
            Self::Common { choices, next } => {
                choices.iter().any(|c| c.id == id).then_some(next.as_ref())
            }
            Self::Branched(choices) => choices
                .iter()
                .find(|b| b.choice.id == id)
                .map(|b| b.next.as_ref()),
        }
    }
}
