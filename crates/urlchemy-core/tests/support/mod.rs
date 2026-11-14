use std::path::{Path, PathBuf};
pub fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/superpowers/specs/yaml-url-contract")
        .join(name)
}
pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).unwrap()
}
use urlchemy_core::{
    Action, Catalog, Entry, EntryId, FlowId, Outcome, RowId, Session, SessionSnapshot,
};

fn route(entries: &[Entry], flow: FlowId, path: &mut Vec<EntryId>) -> bool {
    for entry in entries {
        if entry.flow() == Some(flow) {
            return true;
        }
        if let Some(children) = entry.children() {
            path.push(entry.id());
            if route(children, flow, path) {
                return true;
            }
            path.pop();
        }
    }
    false
}

pub fn drive(catalog: &Catalog, key: &str, answers: &[serde_json::Value]) -> (Session, Outcome) {
    let mut session = Session::new();
    session.apply(catalog, Action::Invoke);
    let flow = catalog.flow_by_key(key).unwrap();
    let mut path = Vec::new();
    assert!(route(catalog.roots(), flow, &mut path));
    for folder in path {
        assert!(matches!(
            session.apply(catalog, Action::EnterFolder(folder)),
            Outcome::StateChanged
        ));
    }
    let mut out = session.apply(catalog, Action::StartFlow(flow));
    for answer in answers {
        assert!(matches!(out, Outcome::StateChanged));
        let action = if let Some(text) = answer.get("text") {
            Action::SetInputText(text.as_str().unwrap().into())
        } else if let Some(index) = answer.get("choice") {
            let SessionSnapshot::Selection { rows, .. } = session.snapshot(catalog).unwrap() else {
                panic!("Expected selection");
            };
            let RowId::Choice(id) = rows[index.as_u64().unwrap() as usize].id() else {
                panic!();
            };
            Action::SelectChoice(id)
        } else {
            let SessionSnapshot::Input { rows, .. } = session.snapshot(catalog).unwrap() else {
                panic!("Expected input");
            };
            let RowId::Suggestion(id) = rows[answer["suggestion"].as_u64().unwrap() as usize].id()
            else {
                panic!();
            };
            Action::SelectSuggestion(Some(id))
        };
        assert!(matches!(
            session.apply(catalog, action),
            Outcome::StateChanged
        ));
        out = session.apply(catalog, Action::Confirm);
    }
    (session, out)
}
