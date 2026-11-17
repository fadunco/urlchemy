use crate::DesktopError;
use std::rc::Rc;
use urlchemy_core::{
    Action, Catalog, Diagnostic, Outcome, ResolvedUrl, Row, RowId, Session, SessionPresentation,
    SessionSnapshot,
    filter::{MatchMode, filter_rows},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseReason {
    Explicit,
    Outside,
    Opened,
}
pub enum PaletteCommand {
    Edited(String),
    Activate(RowId),
    Confirm,
    Move(i32),
    Back,
    Dismiss(CloseReason),
    Retry,
    StartOver,
    OpenFinished {
        invocation: u64,
        operation: u64,
        result: Result<(), DesktopError>,
    },
}
#[derive(Clone, Debug)]
pub enum PaletteEffect {
    Open {
        invocation: u64,
        operation: u64,
        url: ResolvedUrl,
    },
    Close {
        invocation: u64,
        reason: CloseReason,
    },
}
struct Opening {
    url: ResolvedUrl,
    operation: u64,
    pending: bool,
    error: Option<DesktopError>,
}
pub struct Launcher {
    catalog: Rc<Catalog>,
    session: Session,
    invocation: u64,
    operation: u64,
    rows: Vec<Row>,
    active: Option<RowId>,
    query: String,
    error: Option<Diagnostic>,
    opening: Option<Opening>,
}
impl Launcher {
    pub fn new(catalog: Rc<Catalog>, invocation: u64) -> Self {
        let mut result = Self {
            catalog,
            session: Session::new(),
            invocation: 0,
            operation: 0,
            rows: Vec::new(),
            active: None,
            query: String::new(),
            error: None,
            opening: None,
        };
        result.invoke(invocation);
        result
    }
    pub fn invoke(&mut self, invocation: u64) -> bool {
        if invocation == 0 || invocation <= self.invocation {
            return false;
        }
        self.invocation = invocation;
        self.reset();
        true
    }
    fn reset(&mut self) {
        self.opening = None;
        self.query.clear();
        self.error = None;
        self.session.apply(&self.catalog, Action::Invoke);
        self.refresh(true);
    }
    pub fn snapshot(&self) -> Result<SessionSnapshot, Diagnostic> {
        self.session.snapshot(&self.catalog)
    }
    pub fn presentation(&self) -> Result<SessionPresentation, Diagnostic> {
        self.session.presentation(&self.catalog, self.active)
    }
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub fn active(&self) -> Option<RowId> {
        self.active
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn error(&self) -> Option<&Diagnostic> {
        self.error.as_ref()
    }
    pub fn opening_error(&self) -> Option<&DesktopError> {
        self.opening
            .as_ref()
            .and_then(|opening| opening.error.as_ref())
    }
    pub fn is_opening(&self) -> bool {
        self.opening.as_ref().is_some_and(|opening| opening.pending)
    }
    pub fn invocation(&self) -> u64 {
        self.invocation
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    fn refresh(&mut self, reset: bool) {
        let Ok(snapshot) = self.snapshot() else {
            return;
        };
        let (rows, selected, input) = match snapshot {
            SessionSnapshot::Browsing { rows, .. } => (rows, None, false),
            SessionSnapshot::Selection { rows, selected, .. } => {
                (rows, selected.map(RowId::Choice), false)
            }
            SessionSnapshot::Input {
                rows,
                selected,
                text,
                ..
            } => {
                self.query = text;
                (rows, selected.map(RowId::Suggestion), true)
            }
            _ => {
                self.rows.clear();
                self.active = None;
                return;
            }
        };
        let ids = filter_rows(&rows, &self.query, MatchMode::CaseInsensitiveSubstring);
        let mut matches = ids.iter().peekable();
        self.rows = rows
            .into_iter()
            .filter(|row| {
                if matches.peek().is_some_and(|id| **id == row.id()) {
                    matches.next();
                    true
                } else {
                    false
                }
            })
            .collect();
        self.active = if input {
            selected.filter(|id| ids.contains(id))
        } else {
            selected
                .filter(|id| ids.contains(id))
                .or_else(|| {
                    if reset {
                        None
                    } else {
                        self.active.filter(|id| ids.contains(id))
                    }
                })
                .or_else(|| ids.first().copied())
        };
    }
    fn apply(&mut self, action: Action, reset: bool) -> Option<PaletteEffect> {
        match self.session.apply(&self.catalog, action) {
            Outcome::Rejected(error) => self.error = Some(error),
            Outcome::ResolutionFailed(error) => {
                self.error = Some(error);
                self.query.clear();
                self.refresh(true);
            }
            Outcome::Resolved(url) => {
                self.error = None;
                self.refresh(true);
                return self.begin_open(url);
            }
            Outcome::StateChanged => {
                self.error = None;
                if reset {
                    self.query.clear();
                }
                self.refresh(reset);
            }
        }
        None
    }
    fn begin_open(&mut self, url: ResolvedUrl) -> Option<PaletteEffect> {
        self.operation = self.operation.checked_add(1)?;
        self.opening = Some(Opening {
            url: url.clone(),
            operation: self.operation,
            pending: true,
            error: None,
        });
        Some(PaletteEffect::Open {
            invocation: self.invocation,
            operation: self.operation,
            url,
        })
    }
    fn activate(&mut self, id: RowId) -> Option<PaletteEffect> {
        if !self.rows.iter().any(|row| row.id() == id) {
            return None;
        }
        match id {
            RowId::Entry(id) => {
                let SessionSnapshot::Browsing { folders, .. } = self.snapshot().ok()? else {
                    return None;
                };
                let mut entries = self.catalog.roots();
                for folder in folders {
                    entries = entries
                        .iter()
                        .find(|entry| entry.id() == folder)?
                        .children()?;
                }
                let entry = entries.iter().find(|entry| entry.id() == id)?;
                let action = entry
                    .flow()
                    .map(Action::StartFlow)
                    .unwrap_or(Action::EnterFolder(id));
                self.apply(action, true)
            }
            RowId::Choice(id) => {
                self.apply(Action::SelectChoice(id), false);
                self.apply(Action::Confirm, true)
            }
            RowId::Suggestion(id) => self.apply(Action::SelectSuggestion(Some(id)), false),
        }
    }
    pub fn dispatch(&mut self, command: PaletteCommand) -> Option<PaletteEffect> {
        match command {
            PaletteCommand::OpenFinished {
                invocation,
                operation,
                result,
            } => {
                if invocation != self.invocation {
                    return None;
                }
                let opening = self.opening.as_mut()?;
                if opening.operation != operation || !opening.pending {
                    return None;
                }
                match result {
                    Ok(()) => {
                        self.opening = None;
                        self.session.apply(&self.catalog, Action::Dismiss);
                        self.refresh(true);
                        return Some(PaletteEffect::Close {
                            invocation,
                            reason: CloseReason::Opened,
                        });
                    }
                    Err(error) => {
                        opening.pending = false;
                        opening.error = Some(error);
                    }
                }
            }
            PaletteCommand::Dismiss(reason) => {
                self.opening = None;
                self.query.clear();
                self.error = None;
                self.session.apply(&self.catalog, Action::Dismiss);
                self.refresh(true);
                return Some(PaletteEffect::Close {
                    invocation: self.invocation,
                    reason,
                });
            }
            PaletteCommand::StartOver => self.reset(),
            PaletteCommand::Retry => {
                if let Some(opening) = &self.opening
                    && !opening.pending
                {
                    return self.begin_open(opening.url.clone());
                }
            }
            _ if self.opening.is_some() => (),
            PaletteCommand::Edited(text) => match self.snapshot().ok()? {
                SessionSnapshot::Input { .. } => {
                    self.apply(Action::SetInputText(text), false);
                }
                SessionSnapshot::Browsing { .. } => {
                    self.query = text;
                    if let Some(action) = self.session.exact_alias(&self.catalog, &self.query) {
                        return self.apply(action, true);
                    }
                    self.refresh(false);
                }
                SessionSnapshot::Selection { .. } => {
                    self.query = text;
                    self.refresh(false);
                }
                _ => (),
            },
            PaletteCommand::Activate(id) => return self.activate(id),
            PaletteCommand::Confirm => match self.snapshot().ok()? {
                SessionSnapshot::Browsing { .. } | SessionSnapshot::Selection { .. } => {
                    if let Some(id) = self.active {
                        return self.activate(id);
                    }
                }
                SessionSnapshot::Input { .. } => return self.apply(Action::Confirm, true),
                _ => (),
            },
            PaletteCommand::Back => return self.apply(Action::Back, true),
            PaletteCommand::Move(delta) => {
                if self.rows.is_empty() || delta == 0 {
                    return None;
                }
                let ix = match self
                    .active
                    .and_then(|id| self.rows.iter().position(|row| row.id() == id))
                {
                    Some(ix) => {
                        (ix as i64 + i64::from(delta)).clamp(0, self.rows.len() as i64 - 1) as usize
                    }
                    None if delta > 0 => 0,
                    None => self.rows.len() - 1,
                };
                self.active = Some(self.rows[ix].id());
                if let Some(RowId::Suggestion(id)) = self.active {
                    return self.apply(Action::SelectSuggestion(Some(id)), false);
                }
            }
        }
        None
    }
}
