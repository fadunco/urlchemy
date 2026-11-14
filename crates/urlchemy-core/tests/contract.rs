mod support;
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};
use support::{drive, fixture, fixture_path};
use urlchemy_core::{Action, Outcome, SessionSnapshot, config::load_bytes};

fn manifest() -> Value {
    serde_json::from_slice(&fixture("expectations.json")).unwrap()
}

#[test]
fn all_contract_examples_execute() {
    let m = manifest();
    for case in m["journeys"].as_array().unwrap() {
        let name = case["file"].as_str().unwrap();
        let c = load_bytes(&fixture_path(name), &fixture(name)).unwrap();
        let (s, out) = drive(
            &c,
            case["flow"].as_str().unwrap(),
            case["answers"].as_array().unwrap(),
        );
        let Outcome::Resolved(url) = out else {
            panic!("Journey failed: {}: {out:?}", case["name"]);
        };
        assert_eq!(
            url.as_str(),
            case["url"].as_str().unwrap(),
            "{}",
            case["name"]
        );
        assert_eq!(s.snapshot(&c).unwrap(), SessionSnapshot::Completed);
    }
    for case in m["runtime_rejections"].as_array().unwrap() {
        let name = case["file"].as_str().unwrap();
        let c = load_bytes(&fixture_path(name), &fixture(name)).unwrap();
        let (s, out) = drive(
            &c,
            case["flow"].as_str().unwrap(),
            case["answers"].as_array().unwrap(),
        );
        let error = match out {
            Outcome::Rejected(e) | Outcome::ResolutionFailed(e) => e,
            _ => panic!("Expected runtime rejection"),
        };
        assert_eq!(error.code(), case["code"].as_str().unwrap());
        if error.code() == "E_RESOLVE_DOT_SEGMENT" {
            assert!(matches!(
                s.snapshot(&c).unwrap(),
                SessionSnapshot::DestinationError {
                    committed_answers: 2,
                    ..
                }
            ));
        } else {
            assert!(
                matches!(s.snapshot(&c).unwrap(), SessionSnapshot::Input {text, cursor:0, committed_answers:0,..} if text == case["answers"][0]["text"].as_str().unwrap())
            );
        }
    }
    for case in m["invalid_files"].as_array().unwrap() {
        let name = case["file"].as_str().unwrap();
        let e = load_bytes(&fixture_path(name), &fixture(name)).unwrap_err();
        let e = &e.items()[0];
        assert_eq!(e.code(), case["code"].as_str().unwrap(), "{name}");
        assert_eq!(e.pointer(), case["pointer"].as_str().unwrap(), "{name}");
        let location = e.location().unwrap();
        assert_eq!(
            location.line as u64,
            case["line"].as_u64().unwrap(),
            "{name}"
        );
        assert_eq!(
            location.column as u64,
            case["column"].as_u64().unwrap(),
            "{name}"
        );
    }
    let listed: BTreeSet<_> = m["valid_files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap().to_string())
        .chain(
            m["invalid_files"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c["file"].as_str().unwrap().to_string()),
        )
        .collect();
    let found: BTreeSet<_> = std::fs::read_dir(fixture_path("."))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|s| s.ends_with(".yaml"))
        .collect();
    assert_eq!(found, listed);
}

#[test]
fn source_coordinates_and_privacy() {
    for (text, code, line, column) in [
        (
            "\u{feff}navigation: []\r\nunknown: x\r\n",
            "E_SCHEMA_UNKNOWN_FIELD",
            2,
            1,
        ),
        (
            "navigation: [{folder: {label: \"😀\", children: [], unknown: x}}]",
            "E_SCHEMA_UNKNOWN_FIELD",
            1,
            50,
        ),
        (
            "navigation: []\n\"na\\u0076igation\": []",
            "E_YAML_DUPLICATE_KEY",
            2,
            1,
        ),
        (
            "navigation:\n  - flow:\n      id: test\n      baseURL: https://example.com\n      start:\n        destination:\n          template: /",
            "E_SCHEMA_REQUIRED",
            3,
            7,
        ),
    ] {
        let errors = load_bytes(Path::new("f"), text.as_bytes()).unwrap_err();
        let e = &errors.items()[0];
        assert_eq!(e.code(), code);
        assert_eq!(e.location().unwrap().line, line);
        assert_eq!(e.location().unwrap().column, column);
        if code == "E_YAML_DUPLICATE_KEY" {
            assert_eq!(e.related_locations()[0].line, 1);
        }
    }
    let sentinel = "private-session-value";
    let c = load_bytes(&fixture_path("linear.yaml"), &fixture("linear.yaml")).unwrap();
    let (mut s, out) = drive(
        &c,
        "open-application",
        &[serde_json::json!({"text": format!("{sentinel}\n")})],
    );
    let Outcome::Rejected(e) = &out else {
        panic!();
    };
    assert!(!e.message().contains(sentinel));
    assert!(!format!("{out:?}").contains(sentinel));
    assert!(!format!("{:?}", s.snapshot(&c).unwrap()).contains(sentinel));
    assert!(!format!("{:?}", Action::SetInputText(sentinel.into())).contains(sentinel));
    s.apply(&c, Action::SetInputText("jobs".into()));
    s.apply(&c, Action::Confirm);
    s.apply(&c, Action::SetInputText(sentinel.into()));
    let out = s.apply(&c, Action::Confirm);
    let Outcome::Resolved(url) = &out else {
        panic!();
    };
    assert!(url.as_str().contains(sentinel));
    assert!(!format!("{out:?}").contains(sentinel));
}

#[test]
fn raw_controls_cannot_hide_trailing_configuration() {
    for text in [
        "navigation: []\0ignored: [",
        "navigation: [] #\u{1}invalid",
        "navigation: [] #\u{7f}invalid",
        "\u{feff}\u{feff}navigation: []",
    ] {
        let result = load_bytes(Path::new("f"), text.as_bytes());
        assert!(
            result.is_err(),
            "Raw YAML controls must not hide trailing content"
        );
        assert_eq!(result.unwrap_err().items()[0].code(), "E_YAML_SYNTAX");
    }
}

#[test]
fn schema_diagnostics_identify_the_expected_type() {
    let e = load_bytes(
        &fixture_path("invalid-string-type.yaml"),
        &fixture("invalid-string-type.yaml"),
    )
    .unwrap_err();
    assert_eq!(e.items()[0].message(), "Expected a string.");
    let e = load_bytes(Path::new("f"), b"navigation: true").unwrap_err();
    assert_eq!(e.items()[0].message(), "Expected a sequence.");
}
