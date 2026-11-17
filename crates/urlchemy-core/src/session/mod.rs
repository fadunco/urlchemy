mod history;
mod navigation;
mod presentation;
pub use presentation::{PrimaryActionHint, SessionPresentation};

use crate::{
    Catalog, ChoiceId, Diagnostic, EntryId, FlowId, ResolvedUrl, StepId, SuggestionId,
    catalog::StepKind,
    diagnostic::Origin,
    value::{CapturedValue, TextError, check_text},
};
use std::{fmt, path::Path};

#[derive(Clone)]
pub enum Action {
    Invoke,
    EnterFolder(EntryId),
    StartFlow(FlowId),
    SetInputText(String),
    SelectSuggestion(Option<SuggestionId>),
    SelectChoice(ChoiceId),
    Confirm,
    Back,
    Dismiss,
}
#[derive(Debug)]
pub enum Outcome {
    StateChanged,
    Rejected(Diagnostic),
    Resolved(ResolvedUrl),
    ResolutionFailed(Diagnostic),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowId {
    Entry(EntryId),
    Choice(ChoiceId),
    Suggestion(SuggestionId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub(crate) id: RowId,
    pub(crate) label: String,
    pub(crate) emoji: Option<String>,
}
impl Row {
    pub fn id(&self) -> RowId {
        self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn emoji(&self) -> Option<&str> {
        self.emoji.as_deref()
    }
}
#[derive(Clone, PartialEq, Eq)]
pub enum SessionSnapshot {
    Inactive,
    Browsing {
        folders: Vec<EntryId>,
        rows: Vec<Row>,
    },
    Input {
        step: StepId,
        label: String,
        text: String,
        selected: Option<SuggestionId>,
        rows: Vec<Row>,
        cursor: usize,
        committed_answers: usize,
    },
    Selection {
        step: StepId,
        label: String,
        selected: Option<ChoiceId>,
        rows: Vec<Row>,
        cursor: usize,
        committed_answers: usize,
    },
    DestinationError {
        step: StepId,
        committed_answers: usize,
    },
    Completed,
}
#[derive(Clone, PartialEq, Eq)]
enum Answer {
    Input(CapturedValue),
    Choice(ChoiceId),
}
#[derive(Clone)]
struct Record {
    step: StepId,
    answer: Answer,
}
#[derive(Clone)]
enum Draft {
    Input {
        text: String,
        selected: Option<SuggestionId>,
    },
    Selection {
        selected: Option<ChoiceId>,
    },
    DestinationError,
}
#[derive(Clone)]
struct Run {
    flow: FlowId,
    origin: Vec<EntryId>,
    answers: Vec<Record>,
    cursor: usize,
    draft: Draft,
}
enum State {
    Inactive,
    Browsing(Vec<EntryId>),
    Running(Run),
    Completed,
}
pub struct Session {
    owner: Option<u64>,
    state: State,
}
macro_rules! redacted_debug {
    ($($ty:ty),+) => { $(impl fmt::Debug for $ty { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(concat!(stringify!($ty), "([redacted])")) } })+ };
}
redacted_debug!(Action, Session, SessionSnapshot);
impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
fn session_diagnostic(code: &'static str) -> Diagnostic {
    Diagnostic::new(code, &Origin::file(Path::new("<session>")))
}
impl Session {
    pub fn new() -> Self {
        Self {
            owner: None,
            state: State::Inactive,
        }
    }
    fn error(&self, catalog: &Catalog, code: &'static str) -> Diagnostic {
        if let State::Running(run) = &self.state
            && let Some(step) = history::current(catalog, run)
        {
            return Diagnostic::new(code, &step.origin);
        }
        session_diagnostic(code)
    }
    fn install(&mut self, catalog: &Catalog, mut run: Run) -> Outcome {
        match history::visit(catalog, &mut run) {
            Ok(Some(url)) => {
                self.state = State::Completed;
                Outcome::Resolved(url)
            }
            Ok(None) => {
                self.state = State::Running(run);
                Outcome::StateChanged
            }
            Err(e) => {
                self.state = State::Running(run);
                Outcome::ResolutionFailed(e)
            }
        }
    }
    pub fn apply(&mut self, catalog: &Catalog, action: Action) -> Outcome {
        if matches!(action, Action::Invoke) {
            self.owner = Some(catalog.owner);
            self.state = State::Browsing(Vec::new());
            return Outcome::StateChanged;
        }
        if self.owner.is_some_and(|o| o != catalog.owner) {
            return Outcome::Rejected(session_diagnostic("E_ACTION_CATALOG"));
        }
        let context_error = self.error(catalog, "E_ACTION_CONTEXT");
        let context = || context_error.clone();
        match action {
            Action::Invoke => unreachable!(),
            Action::Dismiss => {
                self.state = State::Inactive;
                self.owner = None;
                Outcome::StateChanged
            }
            Action::EnterFolder(id) => {
                let State::Browsing(path) = &self.state else {
                    return Outcome::Rejected(context());
                };
                if navigation::entries(catalog, path)
                    .is_none_or(|rows| !rows.iter().any(|e| e.id() == id && e.children().is_some()))
                {
                    return Outcome::Rejected(self.error(catalog, "E_ACTION_TARGET"));
                }
                let mut path = path.clone();
                path.push(id);
                self.state = State::Browsing(path);
                Outcome::StateChanged
            }
            Action::StartFlow(id) => {
                let State::Browsing(path) = &self.state else {
                    return Outcome::Rejected(context());
                };
                let Some(flow) = catalog.flow(id) else {
                    return Outcome::Rejected(self.error(catalog, "E_ACTION_TARGET"));
                };
                let visible = navigation::entries(catalog, path)
                    .is_some_and(|es| es.iter().any(|e| e.flow() == Some(id)));
                if !visible && !(path.is_empty() && flow.direct_alias.is_some()) {
                    return Outcome::Rejected(self.error(catalog, "E_ACTION_TARGET"));
                }
                self.install(
                    catalog,
                    Run {
                        flow: id,
                        origin: path.clone(),
                        answers: Vec::new(),
                        cursor: 0,
                        draft: Draft::DestinationError,
                    },
                )
            }
            Action::Back => match &mut self.state {
                State::Browsing(path) => {
                    path.pop();
                    Outcome::StateChanged
                }
                State::Running(run) if run.cursor == 0 => {
                    self.state = State::Browsing(run.origin.clone());
                    Outcome::StateChanged
                }
                State::Running(run) => {
                    let mut run = run.clone();
                    run.cursor -= 1;
                    self.install(catalog, run)
                }
                _ => Outcome::Rejected(context()),
            },
            Action::SetInputText(text) => {
                if let State::Running(Run {
                    draft:
                        Draft::Input {
                            text: current,
                            selected,
                        },
                    ..
                }) = &mut self.state
                {
                    *current = text;
                    *selected = None;
                    Outcome::StateChanged
                } else {
                    Outcome::Rejected(context())
                }
            }
            Action::SelectSuggestion(id) => {
                let State::Running(run) = &self.state else {
                    return Outcome::Rejected(context());
                };
                let Some(step) = history::current(catalog, run) else {
                    return Outcome::Rejected(context());
                };
                let StepKind::Input { suggestions, .. } = &step.kind else {
                    return Outcome::Rejected(context());
                };
                if id.is_some_and(|id| !suggestions.iter().any(|s| s.id == id)) {
                    return Outcome::Rejected(self.error(catalog, "E_ACTION_TARGET"));
                }
                if let State::Running(Run {
                    draft: Draft::Input { selected, .. },
                    ..
                }) = &mut self.state
                {
                    *selected = id;
                }
                Outcome::StateChanged
            }
            Action::SelectChoice(id) => {
                let State::Running(run) = &self.state else {
                    return Outcome::Rejected(context());
                };
                let Some(step) = history::current(catalog, run) else {
                    return Outcome::Rejected(context());
                };
                let StepKind::Select { selection, .. } = &step.kind else {
                    return Outcome::Rejected(context());
                };
                if !selection.choices().iter().any(|c| c.id == id) {
                    return Outcome::Rejected(self.error(catalog, "E_ACTION_TARGET"));
                }
                if let State::Running(Run {
                    draft: Draft::Selection { selected },
                    ..
                }) = &mut self.state
                {
                    *selected = Some(id);
                }
                Outcome::StateChanged
            }
            Action::Confirm => {
                let State::Running(run) = &self.state else {
                    return Outcome::Rejected(context());
                };
                let Some(step) = history::current(catalog, run) else {
                    return Outcome::Rejected(context());
                };
                let answer = match (&step.kind, &run.draft) {
                    (StepKind::Input { suggestions, .. }, Draft::Input { text, selected }) => {
                        let value = if let Some(id) = selected {
                            let Some(s) = suggestions.iter().find(|s| s.id == *id) else {
                                return Outcome::Rejected(self.error(catalog, "E_ACTION_TARGET"));
                            };
                            s.value.clone()
                        } else {
                            CapturedValue(text.clone())
                        };
                        if let Err(e) = check_text(&value.0) {
                            return Outcome::Rejected(Diagnostic::new(
                                match e {
                                    TextError::Required => "E_INPUT_REQUIRED",
                                    TextError::Control => "E_INPUT_CONTROL",
                                },
                                &step.origin,
                            ));
                        }
                        Answer::Input(value)
                    }
                    (StepKind::Select { .. }, Draft::Selection { selected: Some(id) }) => {
                        Answer::Choice(*id)
                    }
                    _ => return Outcome::Rejected(context()),
                };
                let mut candidate = run.clone();
                if candidate
                    .answers
                    .get(candidate.cursor)
                    .is_none_or(|r| r.answer != answer)
                {
                    candidate.answers.truncate(candidate.cursor);
                    candidate.answers.push(Record {
                        step: step.id,
                        answer,
                    });
                }
                candidate.cursor += 1;
                self.install(catalog, candidate)
            }
        }
    }
    pub fn snapshot(&self, catalog: &Catalog) -> Result<SessionSnapshot, Diagnostic> {
        if self.owner.is_some_and(|o| o != catalog.owner) {
            return Err(session_diagnostic("E_ACTION_CATALOG"));
        }
        Ok(match &self.state {
            State::Inactive => SessionSnapshot::Inactive,
            State::Completed => SessionSnapshot::Completed,
            State::Browsing(path) => SessionSnapshot::Browsing {
                folders: path.clone(),
                rows: navigation::entries(catalog, path)
                    .ok_or_else(|| self.error(catalog, "E_ACTION_TARGET"))?
                    .iter()
                    .map(|e| Row {
                        id: RowId::Entry(e.id()),
                        label: e.label().into(),
                        emoji: e.emoji().map(str::to_string),
                    })
                    .collect(),
            },
            State::Running(run) => {
                let step = history::current(catalog, run)
                    .ok_or_else(|| self.error(catalog, "E_ACTION_TARGET"))?;
                match (&step.kind, &run.draft) {
                    (
                        StepKind::Input {
                            label, suggestions, ..
                        },
                        Draft::Input { text, selected },
                    ) => SessionSnapshot::Input {
                        step: step.id,
                        label: label.clone(),
                        text: text.clone(),
                        selected: *selected,
                        rows: suggestions
                            .iter()
                            .map(|s| Row {
                                id: RowId::Suggestion(s.id),
                                label: s.label.clone(),
                                emoji: None,
                            })
                            .collect(),
                        cursor: run.cursor,
                        committed_answers: run.answers.len(),
                    },
                    (StepKind::Select { label, selection }, Draft::Selection { selected }) => {
                        SessionSnapshot::Selection {
                            step: step.id,
                            label: label.clone(),
                            selected: *selected,
                            rows: selection
                                .choices()
                                .iter()
                                .map(|c| Row {
                                    id: RowId::Choice(c.id),
                                    label: c.label.clone(),
                                    emoji: None,
                                })
                                .collect(),
                            cursor: run.cursor,
                            committed_answers: run.answers.len(),
                        }
                    }
                    (_, Draft::DestinationError) => SessionSnapshot::DestinationError {
                        step: step.id,
                        committed_answers: run.answers.len(),
                    },
                    _ => return Err(self.error(catalog, "E_ACTION_TARGET")),
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Catalog, config::load_bytes};
    use std::path::Path;
    pub(super) fn fixture(name: &str) -> Catalog {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/superpowers/specs/yaml-url-contract")
            .join(name);
        load_bytes(&p, &std::fs::read(&p).unwrap()).unwrap()
    }
    pub(super) fn start(c: &Catalog, key: &str) -> Session {
        let mut s = Session::new();
        s.apply(c, Action::Invoke);
        s.apply(c, Action::StartFlow(c.flow_by_key(key).unwrap()));
        s
    }
    pub(super) fn input(s: &mut Session, c: &Catalog, text: &str) -> Outcome {
        s.apply(c, Action::SetInputText(text.into()));
        s.apply(c, Action::Confirm)
    }
    pub(super) fn choose(s: &mut Session, c: &Catalog, index: usize) -> Outcome {
        let SessionSnapshot::Selection { rows, .. } = s.snapshot(c).unwrap() else {
            panic!("Expected selection");
        };
        let RowId::Choice(id) = rows[index].id() else {
            panic!("Expected choice");
        };
        s.apply(c, Action::SelectChoice(id));
        s.apply(c, Action::Confirm)
    }
    pub(super) fn four_inputs() -> Catalog {
        let mut step = "{destination: {template: '/{X}/{Y}/{Z}/{W}'}}".to_string();
        for name in ["W", "Z", "Y", "X"] {
            step = format!("{{input: {{label: {name}, capture: {name}, next: {step}}}}}");
        }
        load_bytes(Path::new("f"), format!("navigation: [{{flow: {{id: test, label: Test, baseURL: 'https://example.com', start: {step}}}}}]").as_bytes()).unwrap()
    }
    #[test]
    fn immediate_completion() {
        let c = fixture("fixed.yaml");
        let mut s = Session::new();
        s.apply(&c, Action::Invoke);
        let result = s.apply(&c, Action::StartFlow(c.flow_by_key("base-only").unwrap()));
        let Outcome::Resolved(url) = result else {
            panic!("Expected URL");
        };
        assert_eq!(url.as_str(), "https://example.com/");
        assert_eq!(s.snapshot(&c).unwrap(), SessionSnapshot::Completed);
        assert!(matches!(s.apply(&c, Action::Confirm), Outcome::Rejected(_)));
        let c = fixture("branching.yaml");
        for (choice, answers, expected) in [
            (
                0,
                vec!["accounts", "billing"],
                "https://app.datadog.com/company/8c21-http?from=accounts&to=billing",
            ),
            (
                1,
                vec!["alice"],
                "https://app.datadog.com/company/9d32-users?userID=alice",
            ),
        ] {
            let mut s = start(&c, "open-dashboard");
            choose(&mut s, &c, choice);
            let mut result = Outcome::StateChanged;
            for answer in answers {
                result = input(&mut s, &c, answer);
            }
            let Outcome::Resolved(url) = result else {
                panic!("Expected URL");
            };
            assert_eq!(url.as_str(), expected);
        }
    }
    #[test]
    fn review_and_invalidation() {
        let c = four_inputs();
        let mut s = start(&c, "test");
        for v in ["x", "y", "z"] {
            input(&mut s, &c, v);
        }
        s.apply(&c, Action::Back);
        s.apply(&c, Action::Back);
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input {text, cursor:1, committed_answers:3,..} if text == "y")
        );
        input(&mut s, &c, "y");
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input {text, cursor:2, committed_answers:3,..} if text == "z")
        );
        s.apply(&c, Action::Back);
        input(&mut s, &c, "new-y");
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input {text, cursor:2, committed_answers:2,..} if text.is_empty())
        );
    }
    #[test]
    fn drafts_and_rejections() {
        let c = four_inputs();
        let mut s = start(&c, "test");
        for v in ["x", "y", "z"] {
            input(&mut s, &c, v);
        }
        s.apply(&c, Action::Back);
        s.apply(&c, Action::SetInputText("unconfirmed".into()));
        s.apply(&c, Action::Back);
        input(&mut s, &c, "y");
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input {text, committed_answers:3,..} if text == "z")
        );
        let before = s.snapshot(&c).unwrap();
        let Outcome::Rejected(e) = input(&mut s, &c, "") else {
            panic!("Expected rejection");
        };
        assert_eq!(e.code(), "E_INPUT_REQUIRED");
        assert!(matches!(
            s.snapshot(&c).unwrap(),
            SessionSnapshot::Input {
                committed_answers: 3,
                cursor: 2,
                ..
            }
        ));
        assert_ne!(s.snapshot(&c).unwrap(), before);
        let c = fixture("linear.yaml");
        let mut s = start(&c, "open-application");
        let SessionSnapshot::Input { rows, .. } = s.snapshot(&c).unwrap() else {
            panic!();
        };
        let RowId::Suggestion(id) = rows[0].id() else {
            panic!();
        };
        s.apply(&c, Action::SelectSuggestion(Some(id)));
        s.apply(&c, Action::SetInputText("Jobs".into()));
        s.apply(&c, Action::Confirm);
        let Outcome::Resolved(url) = input(&mut s, &c, "accounts") else {
            panic!();
        };
        assert!(url.as_str().contains("/accounts/Jobs?"));
    }
    #[test]
    fn reset_and_identity() {
        let c = four_inputs();
        let other = four_inputs();
        let mut s = start(&c, "test");
        input(&mut s, &c, "x");
        let before = s.snapshot(&c).unwrap();
        assert!(
            matches!(s.apply(&other, Action::Confirm), Outcome::Rejected(e) if e.code() == "E_ACTION_CATALOG")
        );
        assert!(s.snapshot(&other).is_err());
        assert_eq!(s.snapshot(&c).unwrap(), before);
        assert!(matches!(
            s.apply(&c, Action::StartFlow(other.flow_by_key("test").unwrap())),
            Outcome::Rejected(_)
        ));
        s.apply(&c, Action::Dismiss);
        assert_eq!(s.snapshot(&c).unwrap(), SessionSnapshot::Inactive);
        s.apply(&c, Action::Invoke);
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Browsing {folders,..} if folders.is_empty())
        );
        s.apply(&c, Action::StartFlow(c.flow_by_key("test").unwrap()));
        s.apply(&c, Action::Back);
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Browsing {folders,..} if folders.is_empty())
        );
    }
    #[test]
    fn resolution_error() {
        let c = fixture("linear.yaml");
        let mut s = start(&c, "open-application");
        input(&mut s, &c, "jobs");
        assert!(
            matches!(input(&mut s, &c, ".."), Outcome::ResolutionFailed(e) if e.code() == "E_RESOLVE_DOT_SEGMENT")
        );
        assert!(matches!(
            s.snapshot(&c).unwrap(),
            SessionSnapshot::DestinationError {
                committed_answers: 2,
                ..
            }
        ));
        assert!(matches!(s.apply(&c, Action::Confirm), Outcome::Rejected(_)));
        s.apply(&c, Action::Back);
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input {text,..} if text == "..")
        );
        assert!(matches!(
            input(&mut s, &c, "accounts"),
            Outcome::Resolved(_)
        ));
    }

    #[test]
    fn choice_identity_and_common_suffix() {
        let mut suffix = "{destination: {template: '/{route}/{x}/{y}/{z}'}}".to_string();
        for name in ["z", "y", "x"] {
            suffix = format!("{{input: {{label: {name}, capture: {name}, next: {suffix}}}}}");
        }
        for common in [true, false] {
            let selection = if common {
                format!(
                    "{{label: Route, capture: route, options: [{{label: First, value: same}}, {{label: Second, value: same}}], next: {suffix}}}"
                )
            } else {
                format!(
                    "{{label: Route, capture: route, options: [{{label: First, value: same, next: {suffix}}}, {{label: Second, value: same, next: {suffix}}}]}}"
                )
            };
            let c = load_bytes(Path::new("f"), format!("navigation: [{{flow: {{id: test, label: Test, baseURL: 'https://example.com', start: {{select: {selection}}}}}}}]").as_bytes()).unwrap();
            let mut s = start(&c, "test");
            choose(&mut s, &c, 0);
            input(&mut s, &c, "x");
            input(&mut s, &c, "y");
            for _ in 0..3 {
                s.apply(&c, Action::Back);
            }
            choose(&mut s, &c, 0);
            assert!(
                matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input { text, committed_answers:3,..} if text == "x")
            );
            s.apply(&c, Action::Back);
            choose(&mut s, &c, 1);
            assert!(
                matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input { text, committed_answers:1,..} if text.is_empty())
            );
        }
    }

    #[test]
    fn suggestion_value_equality_preserves_suffix() {
        let mut step = serde_json::json!({"destination": {"template": "/{x}/{y}/{z}"}});
        for name in ["z", "y", "x"] {
            step = serde_json::json!({"input": {"label": name, "capture": name, "next": step}});
        }
        step["input"]["suggestions"] = serde_json::json!([{"label": "Example", "value": "same"}]);
        let text = serde_json::json!({"navigation": [{"flow": {"id": "test", "label": "Test", "baseURL": "https://example.com", "start": step}}]}).to_string();
        let c = load_bytes(Path::new("f"), text.as_bytes()).unwrap();
        let mut s = start(&c, "test");
        input(&mut s, &c, "same");
        input(&mut s, &c, "y");
        s.apply(&c, Action::Back);
        s.apply(&c, Action::Back);
        let SessionSnapshot::Input { rows, .. } = s.snapshot(&c).unwrap() else {
            panic!();
        };
        let RowId::Suggestion(id) = rows[0].id() else {
            panic!();
        };
        s.apply(&c, Action::SelectSuggestion(Some(id)));
        s.apply(&c, Action::Confirm);
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input {text, committed_answers:2,..} if text == "y")
        );
        s.apply(&c, Action::Invoke);
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Browsing {folders,..} if folders.is_empty())
        );
    }

    #[test]
    fn empty_run_failure_and_nested_origin() {
        let mut c = fixture("fixed.yaml");
        let id = c.flow_by_key("base-only").unwrap();
        let flow = c.flows.iter_mut().find(|f| f.id == id).unwrap();
        flow.start.kind = crate::catalog::StepKind::Destination(
            crate::url::compile_template("/{missing}", &flow.start.origin).unwrap(),
        );
        let mut s = start(&c, "base-only");
        assert!(matches!(
            s.snapshot(&c).unwrap(),
            SessionSnapshot::DestinationError {
                committed_answers: 0,
                ..
            }
        ));
        s.apply(&c, Action::Back);
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Browsing {folders,..} if folders.is_empty())
        );
        let c = fixture("branching.yaml");
        let mut s = Session::new();
        s.apply(&c, Action::Invoke);
        let folder = c.roots()[0].id();
        s.apply(&c, Action::EnterFolder(folder));
        s.apply(
            &c,
            Action::StartFlow(c.flow_by_key("open-dashboard").unwrap()),
        );
        s.apply(&c, Action::Back);
        assert!(
            matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Browsing {folders,..} if folders == vec![folder])
        );
    }
}
