#![forbid(unsafe_code)]

mod catalog;
pub mod filter;
pub mod session;
pub use session::{Action, Outcome, Row, RowId, Session, SessionSnapshot};
pub mod config;
mod diagnostic;
mod shortcut;
pub use shortcut::{Shortcut, ShortcutKey, ShortcutModifiers, Shortcuts};
mod url;
mod value;
pub use catalog::{Catalog, ChoiceId, Entry, EntryId, FlowId, StepId, SuggestionId};
pub use diagnostic::{Diagnostic, Diagnostics, SourceLocation};
pub use url::ResolvedUrl;
