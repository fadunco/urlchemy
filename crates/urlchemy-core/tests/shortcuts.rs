mod support;

use std::path::Path;
use support::{drive, fixture, fixture_path};
use urlchemy_core::{
    Catalog, Diagnostics, Outcome, SessionSnapshot, SourceLocation, config::load_bytes,
};

fn load(text: &str) -> Result<Catalog, Diagnostics> {
    load_bytes(Path::new("shortcuts.yaml"), text.as_bytes())
}

#[test]
fn shortcut_defaults_and_partial_mappings() {
    for (text, invoke, launch) in [
        ("navigation: []", "ctrl-alt-space", "enter"),
        ("shortcuts: {}\nnavigation: []", "ctrl-alt-space", "enter"),
        (
            "shortcuts: {invoke: ctrl-k}\nnavigation: []",
            "ctrl-k",
            "enter",
        ),
        (
            "shortcuts: {launch: cmd-enter}\nnavigation: []",
            "ctrl-alt-space",
            "cmd-enter",
        ),
        (
            "shortcuts: {invoke: alt-ctrl-k, launch: shift-cmd-enter}\nnavigation: []",
            "ctrl-alt-k",
            "shift-cmd-enter",
        ),
    ] {
        let catalog = load(text).unwrap();
        assert_eq!(catalog.shortcuts().invoke().binding(), invoke);
        assert_eq!(catalog.shortcuts().launch().binding(), launch);
    }
}

#[test]
fn strict_shortcut_schema_and_locations() {
    for (settings, code, pointer) in [
        ("shortcuts: null", "E_SCHEMA_TYPE", "/shortcuts"),
        ("shortcuts: []", "E_SCHEMA_TYPE", "/shortcuts"),
        ("shortcuts: x", "E_SCHEMA_TYPE", "/shortcuts"),
        (
            "shortcuts: {launch: null}",
            "E_SCHEMA_TYPE",
            "/shortcuts/launch",
        ),
        (
            "shortcuts: {invoke: 12}",
            "E_SCHEMA_TYPE",
            "/shortcuts/invoke",
        ),
        (
            "shortcuts: {invoke: true}",
            "E_SCHEMA_TYPE",
            "/shortcuts/invoke",
        ),
        (
            "shortcuts: {launch: [enter]}",
            "E_SCHEMA_TYPE",
            "/shortcuts/launch",
        ),
        (
            "shortcuts: {confirm: enter}",
            "E_SCHEMA_UNKNOWN_FIELD",
            "/shortcuts/confirm",
        ),
        (
            "shortcuts: {launch: enter, launch: x}",
            "E_YAML_DUPLICATE_KEY",
            "/shortcuts/launch",
        ),
        (
            "shortcuts: {}\nshortcuts: {}",
            "E_YAML_DUPLICATE_KEY",
            "/shortcuts",
        ),
    ] {
        let errors = load(&format!("{settings}\nnavigation: []\n")).unwrap_err();
        let error = &errors.items()[0];
        assert_eq!(error.code(), code, "{settings}");
        assert_eq!(error.pointer(), pointer, "{settings}");
        assert_eq!(error.source(), Path::new("shortcuts.yaml"));
        assert!(error.location().is_some(), "{settings}");
    }
    let errors = load("shortcuts: {12: enter}\nnavigation: []").unwrap_err();
    assert_eq!(errors.items()[0].code(), "E_SCHEMA_TYPE");
    assert!(errors.items()[0].location().is_some());
    let errors = load("shortcuts: {}").unwrap_err();
    assert_eq!(errors.items()[0].code(), "E_SCHEMA_REQUIRED");
    assert_eq!(errors.items()[0].pointer(), "/navigation");
}

#[test]
fn shortcut_validation_is_atomic_and_source_aware() {
    let errors = load("shortcuts:\n  launch: cmd-a\nnavigation: []\n").unwrap_err();
    let error = &errors.items()[0];
    assert_eq!(error.code(), "E_SHORTCUT_CONFLICT");
    assert_eq!(error.pointer(), "/shortcuts/launch");
    assert_eq!(
        error.location(),
        Some(SourceLocation {
            line: 2,
            column: 11
        })
    );
    assert_ne!(
        error.message(),
        "The configuration or action violates the required contract."
    );

    let errors = load("shortcuts: {launch: alt-ctrl-space}\nnavigation: []").unwrap_err();
    assert_eq!(errors.items()[0].code(), "E_SHORTCUT_CONFLICT");
    assert_eq!(errors.items()[0].pointer(), "/shortcuts/launch");
    assert!(errors.items()[0].related_locations().is_empty());

    let errors =
        load("shortcuts:\n  invoke: ctrl-alt-k\n  launch: alt-ctrl-k\nnavigation: []").unwrap_err();
    let error = &errors.items()[0];
    assert_eq!(error.code(), "E_SHORTCUT_CONFLICT");
    assert_eq!(error.pointer(), "/shortcuts/launch");
    assert_eq!(
        error.location(),
        Some(SourceLocation {
            line: 3,
            column: 11
        })
    );
    assert_eq!(
        error.related_locations(),
        &[SourceLocation {
            line: 2,
            column: 11
        }]
    );

    for field in ["invoke", "launch"] {
        for text in ["cmd-1", "cmd-9", "alt-cmd-h", "shift-cmd-z"] {
            let errors =
                load(&format!("shortcuts: {{{field}: {text}}}\nnavigation: []")).unwrap_err();
            assert_eq!(errors.items()[0].code(), "E_SHORTCUT_CONFLICT");
            assert_eq!(errors.items()[0].pointer(), format!("/shortcuts/{field}"));
        }
    }
    let valid_navigation = "navigation:\n  - flow:\n      id: status\n      label: Status\n      baseURL: https://example.com\n      start: {destination: {template: \"\"}}\n";
    assert!(load(valid_navigation).is_ok());
    let errors = load(&format!(
        "shortcuts: {{invoke: ctrl-f13}}\n{valid_navigation}"
    ))
    .unwrap_err();
    assert_eq!(errors.items()[0].code(), "E_SHORTCUT_SYNTAX");
    let errors = load(&format!(
        "shortcuts: {{invoke: ctrl-f13}}\n{}",
        valid_navigation.replace("https://", "file://")
    ))
    .unwrap_err();
    assert!(
        errors
            .items()
            .iter()
            .any(|error| error.code() == "E_SHORTCUT_SYNTAX")
    );
    assert!(
        errors
            .items()
            .iter()
            .any(|error| error.code() == "E_BASE_URL")
    );

    for field in ["invoke", "launch"] {
        for text in [
            "",
            " enter",
            "enter ",
            "CMD-enter",
            "ctrl-ctrl-k",
            "ctrl-k ctrl-l",
            "cmd",
            "cmd-",
            "fn-enter",
            "super-enter",
            "cmd-k->x",
            "ctrl-é",
            "ctrl-f0",
            "ctrl-f13",
            "ctrl-f01",
            "cmd-escape",
            "cmd-[",
            "space",
            "a",
            "f1",
            "shift-enter",
            "alt-enter",
            "ctrl--k",
            "k-ctrl",
            "ctrl-enter\n",
            "ctrl-\tenter",
            "ctrl-ENTER",
            "ctrl-k-alt",
        ] {
            let quoted = serde_json::to_string(text).unwrap();
            let errors =
                load(&format!("shortcuts: {{{field}: {quoted}}}\nnavigation: []")).unwrap_err();
            assert_eq!(
                errors.items()[0].code(),
                "E_SHORTCUT_SYNTAX",
                "{field}: {text:?}"
            );
            assert_eq!(errors.items()[0].pointer(), format!("/shortcuts/{field}"));
            assert!(errors.items()[0].location().is_some());
        }
    }
}

#[test]
fn custom_shortcuts_preserve_existing_journeys() {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fixture("expectations.json")).unwrap();
    for journey in manifest["journeys"].as_array().unwrap() {
        let file = journey["file"].as_str().unwrap();
        let original = fixture(file);
        let defaults = load_bytes(&fixture_path(file), &original).unwrap();
        assert_eq!(defaults.shortcuts().invoke().binding(), "ctrl-alt-space");
        assert_eq!(defaults.shortcuts().launch().binding(), "enter");
        let mut configured = b"shortcuts: {invoke: ctrl-k, launch: cmd-enter}\n".to_vec();
        configured.extend_from_slice(&original);
        let catalog = load_bytes(&fixture_path(file), &configured).unwrap();
        assert_eq!(catalog.shortcuts().invoke().binding(), "ctrl-k");
        assert_eq!(catalog.shortcuts().launch().binding(), "cmd-enter");
        for catalog in [&defaults, &catalog] {
            let (session, outcome) = drive(
                catalog,
                journey["flow"].as_str().unwrap(),
                journey["answers"].as_array().unwrap(),
            );
            let Outcome::Resolved(url) = outcome else {
                panic!("Journey failed: {}: {outcome:?}", journey["name"]);
            };
            assert_eq!(url.as_str(), journey["url"].as_str().unwrap());
            assert_eq!(
                session.snapshot(catalog).unwrap(),
                SessionSnapshot::Completed
            );
        }
    }
}

#[test]
fn shortcut_diagnostics_do_not_echo_invalid_values() {
    let private = "private-configuration-value";
    let errors = load(&format!("shortcuts: {{invoke: {private}}}\nnavigation: []")).unwrap_err();
    assert_eq!(errors.items()[0].code(), "E_SHORTCUT_SYNTAX");
    assert!(!format!("{errors:?}").contains(private));
    assert!(!errors.items()[0].message().contains(private));
}
