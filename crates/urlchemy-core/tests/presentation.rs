use std::path::Path;
use urlchemy_core::{
    Action, Catalog, Outcome, PrimaryActionHint as Hint, RowId, Session, SessionSnapshot,
    config::load_bytes,
};

fn fixture(name: &str) -> Catalog {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/superpowers/specs/yaml-url-contract")
        .join(name);
    load_bytes(&path, &std::fs::read(&path).unwrap()).unwrap()
}
fn first(session: &Session, catalog: &Catalog) -> RowId {
    match session.snapshot(catalog).unwrap() {
        SessionSnapshot::Browsing { rows, .. }
        | SessionSnapshot::Input { rows, .. }
        | SessionSnapshot::Selection { rows, .. } => rows[0].id(),
        _ => panic!("No rows"),
    }
}
fn start(session: &mut Session, catalog: &Catalog, key: &str) {
    session.apply(catalog, Action::Invoke);
    session.apply(
        catalog,
        Action::StartFlow(catalog.flow_by_key(key).unwrap()),
    );
}

#[test]
fn presentation_tracks_context_without_answers() {
    let catalog = fixture("branching.yaml");
    let mut session = Session::new();
    session.apply(&catalog, Action::Invoke);
    assert!(!session.presentation(&catalog, None).unwrap().can_go_back());
    assert_eq!(catalog.roots()[0].alias(), Some("dd"));
    let action = session.exact_alias(&catalog, "dd").unwrap();
    session.apply(&catalog, action);
    assert!(session.presentation(&catalog, None).unwrap().can_go_back());
    session.apply(
        &catalog,
        session.exact_alias(&catalog, "dashboards").unwrap(),
    );
    let RowId::Choice(choice) = first(&session, &catalog) else {
        panic!()
    };
    session.apply(&catalog, Action::SelectChoice(choice));
    session.apply(&catalog, Action::Confirm);
    session.apply(&catalog, Action::SetInputText("accounts".into()));
    session.apply(&catalog, Action::Confirm);
    let before = session.snapshot(&catalog).unwrap();
    let presentation = session.presentation(&catalog, None).unwrap();
    assert_eq!(
        presentation.context().last().map(String::as_str),
        Some("HTTP Traffic")
    );
    assert!(
        !presentation
            .context()
            .iter()
            .any(|label| label.contains("accounts"))
    );
    assert!(presentation.can_go_back());
    assert_eq!(before, session.snapshot(&catalog).unwrap());
}

#[test]
fn confirmation_hint_follows_the_current_definition() {
    let catalog = fixture("branching.yaml");
    let mut session = Session::new();
    session.apply(&catalog, Action::Invoke);
    assert_eq!(
        session
            .presentation(&catalog, Some(first(&session, &catalog)))
            .unwrap()
            .primary_action(),
        Hint::OpenFolder
    );
    session.apply(&catalog, session.exact_alias(&catalog, "dd").unwrap());
    assert_eq!(
        session
            .presentation(&catalog, Some(first(&session, &catalog)))
            .unwrap()
            .primary_action(),
        Hint::StartFlow
    );
    let linear = fixture("linear.yaml");
    start(&mut session, &linear, "open-application");
    assert_eq!(
        session
            .presentation(&linear, None)
            .unwrap()
            .primary_action(),
        Hint::Continue
    );
    session.apply(&linear, Action::SetInputText("jobs".into()));
    session.apply(&linear, Action::Confirm);
    assert_eq!(
        session
            .presentation(&linear, None)
            .unwrap()
            .primary_action(),
        Hint::OpenUrl
    );
    session.apply(&linear, Action::SetInputText("..".into()));
    assert!(matches!(
        session.apply(&linear, Action::Confirm),
        Outcome::ResolutionFailed(_)
    ));
    assert_eq!(
        session
            .presentation(&linear, None)
            .unwrap()
            .primary_action(),
        Hint::Unavailable
    );
    assert!(session.presentation(&linear, None).unwrap().can_go_back());
    let common = fixture("common.yaml");
    start(&mut session, &common, "environment-application");
    assert_eq!(
        session
            .presentation(&common, None)
            .unwrap()
            .primary_action(),
        Hint::ChooseOption
    );
    assert_eq!(
        session
            .presentation(&common, Some(first(&session, &common)))
            .unwrap()
            .primary_action(),
        Hint::Continue
    );
    let fixed = fixture("fixed.yaml");
    session.apply(&fixed, Action::Invoke);
    let flow = fixed.flow_by_key("fixed").unwrap();
    let entry = fixed
        .roots()
        .iter()
        .flat_map(|entry| entry.children().unwrap_or(&[]))
        .find(|entry| entry.flow() == Some(flow))
        .unwrap();
    let folder = fixed
        .roots()
        .iter()
        .find(|e| {
            e.children()
                .is_some_and(|es| es.iter().any(|e| e.id() == entry.id()))
        })
        .unwrap();
    session.apply(&fixed, Action::EnterFolder(folder.id()));
    assert_eq!(
        session
            .presentation(&fixed, Some(RowId::Entry(entry.id())))
            .unwrap()
            .primary_action(),
        Hint::OpenUrl
    );
}

#[test]
fn presentation_rejects_stale_targets() {
    let catalog = fixture("branching.yaml");
    let other = fixture("branching.yaml");
    let mut session = Session::new();
    session.apply(&catalog, Action::Invoke);
    let stale = first(&session, &catalog);
    assert_eq!(
        session.presentation(&other, None).unwrap_err().code(),
        "E_ACTION_CATALOG"
    );
    session.apply(&catalog, session.exact_alias(&catalog, "dd").unwrap());
    assert_eq!(
        session
            .presentation(&catalog, Some(stale))
            .unwrap_err()
            .code(),
        "E_ACTION_TARGET"
    );
}
