use std::{path::Path, rc::Rc};
use urlchemy_app::{CloseReason, DesktopError, Launcher, PaletteCommand as Cmd, PaletteEffect};
use urlchemy_core::{Catalog, RowId, SessionSnapshot, config::load_bytes};

fn catalog(name: &str) -> Rc<Catalog> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/superpowers/specs/yaml-url-contract")
        .join(name);
    Rc::new(load_bytes(&path, &std::fs::read(&path).unwrap()).unwrap())
}
fn input() -> Launcher {
    let mut launcher = Launcher::new(catalog("linear.yaml"), 1);
    assert!(launcher.dispatch(Cmd::Confirm).is_none());
    launcher
}
fn last_input(launcher: &mut Launcher) {
    launcher.dispatch(Cmd::Edited("jobs".into()));
    launcher.dispatch(Cmd::Confirm);
    launcher.dispatch(Cmd::Edited("accounts".into()));
}
fn opened(effect: Option<PaletteEffect>) -> (u64, u64, String) {
    let Some(PaletteEffect::Open {
        invocation,
        operation,
        url,
    }) = effect
    else {
        panic!("Expected an opening request")
    };
    (invocation, operation, url.as_str().into())
}

#[test]
fn literal_and_suggestion_paths_emit_exact_urls() {
    for (select, section) in [(false, "Jobs"), (true, "jobs")] {
        let mut launcher = input();
        launcher.dispatch(Cmd::Edited("Jobs".into()));
        if select {
            let suggestion = launcher.rows()[0].id();
            assert!(matches!(suggestion, RowId::Suggestion(_)));
            assert!(launcher.dispatch(Cmd::Activate(suggestion)).is_none());
        }
        launcher.dispatch(Cmd::Confirm);
        launcher.dispatch(Cmd::Edited("accounts".into()));
        let (_, _, url) = opened(launcher.dispatch(Cmd::Confirm));
        assert_eq!(
            url,
            format!(
                "https://idp.example.com/company/engineering/application/accounts/{section}?owner=accounts&owner=backup"
            )
        );
        assert_eq!(launcher.snapshot().unwrap(), SessionSnapshot::Completed);
    }
}

#[test]
fn back_and_dismiss_preserve_only_committed_history() {
    let mut launcher = input();
    launcher.dispatch(Cmd::Edited("jobs".into()));
    launcher.dispatch(Cmd::Confirm);
    launcher.dispatch(Cmd::Edited("unfinished".into()));
    launcher.dispatch(Cmd::Back);
    assert!(
        matches!(launcher.snapshot().unwrap(),SessionSnapshot::Input {text,..} if text=="jobs")
    );
    launcher.dispatch(Cmd::Confirm);
    assert!(
        matches!(launcher.snapshot().unwrap(),SessionSnapshot::Input {text,..} if text.is_empty())
    );
    assert!(matches!(
        launcher.dispatch(Cmd::Dismiss(CloseReason::Explicit)),
        Some(PaletteEffect::Close {
            invocation: 1,
            reason: CloseReason::Explicit
        })
    ));
    assert!(launcher.invoke(2));
    assert!(!launcher.invoke(1));
    assert!(!launcher.invoke(0));
    assert_eq!(launcher.invocation(), 2);
    assert!(
        matches!(launcher.snapshot().unwrap(),SessionSnapshot::Browsing {folders,..} if folders.is_empty())
    );
    assert!(launcher.query().is_empty());
}

#[test]
fn opening_retry_is_not_core_confirmation() {
    let mut launcher = input();
    last_input(&mut launcher);
    let (invocation, operation, url) = opened(launcher.dispatch(Cmd::Confirm));
    assert!(launcher.is_opening());
    assert!(launcher.dispatch(Cmd::Confirm).is_none());
    assert!(launcher.dispatch(Cmd::Retry).is_none());
    launcher.dispatch(Cmd::OpenFinished {
        invocation,
        operation,
        result: Err(DesktopError::OpenFailed),
    });
    assert!(launcher.opening_error().is_some());
    let (invocation2, operation2, url2) = opened(launcher.dispatch(Cmd::Retry));
    assert_eq!(invocation2, invocation);
    assert!(operation2 > operation);
    assert_eq!(url2, url);
    assert_eq!(launcher.snapshot().unwrap(), SessionSnapshot::Completed);
    assert!(matches!(
        launcher.dispatch(Cmd::OpenFinished {
            invocation: invocation2,
            operation: operation2,
            result: Ok(())
        }),
        Some(PaletteEffect::Close {
            reason: CloseReason::Opened,
            ..
        })
    ));
    assert!(!launcher.is_opening());
    assert!(launcher.dispatch(Cmd::Retry).is_none());
}

#[test]
fn stale_results_cannot_close_a_new_invocation() {
    let mut launcher = input();
    last_input(&mut launcher);
    let (invocation, operation, _) = opened(launcher.dispatch(Cmd::Confirm));
    launcher.dispatch(Cmd::OpenFinished {
        invocation,
        operation,
        result: Err(DesktopError::OpenFailed),
    });
    let (_, retry, _) = opened(launcher.dispatch(Cmd::Retry));
    assert!(
        launcher
            .dispatch(Cmd::OpenFinished {
                invocation,
                operation,
                result: Ok(())
            })
            .is_none()
    );
    assert!(launcher.is_opening());
    assert!(launcher.invoke(2));
    assert!(
        launcher
            .dispatch(Cmd::OpenFinished {
                invocation,
                operation: retry,
                result: Ok(())
            })
            .is_none()
    );
    let mut replacement = Launcher::new(catalog("linear.yaml"), 2);
    assert!(
        replacement
            .dispatch(Cmd::OpenFinished {
                invocation,
                operation: retry,
                result: Ok(())
            })
            .is_none()
    );
    assert!(matches!(
        replacement.snapshot().unwrap(),
        SessionSnapshot::Browsing { .. }
    ));
    assert!(launcher.dispatch(Cmd::Retry).is_none());
}

#[test]
fn row_selection_and_filtering_are_explicit() {
    let mut launcher = Launcher::new(catalog("linear.yaml"), 1);
    assert!(launcher.active().is_some());
    launcher.dispatch(Cmd::Edited("NO MATCH".into()));
    assert!(launcher.rows().is_empty());
    assert!(launcher.active().is_none());
    assert!(launcher.dispatch(Cmd::Confirm).is_none());
    launcher.dispatch(Cmd::Edited("APPLICATION".into()));
    assert_eq!(launcher.rows().len(), 1);
    launcher.dispatch(Cmd::Confirm);
    assert!(launcher.active().is_none());
    launcher.dispatch(Cmd::Move(1));
    let first = launcher.active();
    launcher.dispatch(Cmd::Move(-1));
    assert_eq!(launcher.active(), first);
    launcher.dispatch(Cmd::Edited("data".into()));
    assert!(launcher.active().is_none());
    assert_eq!(launcher.rows()[0].label(), "Databases");
    launcher.dispatch(Cmd::Move(-1));
    assert!(launcher.active().is_some());
}

#[test]
fn common_selection_uses_the_core_continuation() {
    let mut launcher = Launcher::new(catalog("common.yaml"), 1);
    launcher.dispatch(Cmd::Confirm);
    let choice = launcher.rows()[0].id();
    launcher.dispatch(Cmd::Activate(choice));
    launcher.dispatch(Cmd::Edited("accounts".into()));
    assert_eq!(
        opened(launcher.dispatch(Cmd::Confirm)).2,
        "https://idp.example.com/company/staging/application/accounts"
    );
    launcher.dispatch(Cmd::StartOver);
    assert!(matches!(
        launcher.snapshot().unwrap(),
        SessionSnapshot::Browsing { .. }
    ));
    assert!(!launcher.is_opening());
}

#[test]
fn resolution_error_recovers_through_back() {
    let mut launcher = input();
    launcher.dispatch(Cmd::Edited("jobs".into()));
    launcher.dispatch(Cmd::Confirm);
    launcher.dispatch(Cmd::Edited("..".into()));
    assert!(launcher.dispatch(Cmd::Confirm).is_none());
    assert_eq!(launcher.error().unwrap().code(), "E_RESOLVE_DOT_SEGMENT");
    launcher.dispatch(Cmd::Back);
    assert!(launcher.error().is_none());
    launcher.dispatch(Cmd::Edited("accounts".into()));
    assert!(launcher.dispatch(Cmd::Confirm).is_some());
}

#[test]
fn resolution_failure_clears_stale_rows_and_preserves_back() {
    let source = b"navigation:
  - flow:
      id: invalid-suggestion
      label: Invalid suggestion
      baseURL: https://example.com
      start:
        input:
          label: Segment
          capture: segment
          suggestions:
            - label: Parent
              value: '..'
          next:
            destination:
              template: '/{segment}'
";
    let catalog = Rc::new(load_bytes(Path::new("fixture.yaml"), source).unwrap());
    let mut launcher = Launcher::new(catalog, 1);
    launcher.dispatch(Cmd::Confirm);
    let suggestion = launcher.rows()[0].id();
    launcher.dispatch(Cmd::Activate(suggestion));
    assert!(launcher.dispatch(Cmd::Confirm).is_none());
    assert_eq!(launcher.error().unwrap().code(), "E_RESOLVE_DOT_SEGMENT");
    assert!(matches!(
        launcher.snapshot().unwrap(),
        SessionSnapshot::DestinationError { .. }
    ));
    assert!(launcher.rows().is_empty());
    assert_eq!(launcher.active(), None);
    assert!(launcher.presentation().unwrap().can_go_back());
    launcher.dispatch(Cmd::Back);
    assert!(matches!(
        launcher.snapshot().unwrap(),
        SessionSnapshot::Input { .. }
    ));
}
