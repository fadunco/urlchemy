# Native development

## Status and scope

The native vertical slice connects the production GPUI Kit palette to the tested core.
The app uses Rust 1.100.0, GPUI Kit 0.9.1, and macOS desktop services.
The core has no GPUI or macOS dependency.

Headless tests exercise the production view and shell with injected desktop services.
These tests do not register OS shortcuts, activate native windows, or open a browser.
Native acceptance is partial. Compilation does not prove native behavior.

Verification on 2026-11-16:

- All 74 workspace tests passed, including 15 shell tests and seven production-view UI tests.
- All 39 core tests passed independently.
- Formatting and Clippy with warnings denied passed.
- The prerequisite check and all three diagnostic-script regressions passed.
- The native acceptance target compiled with `--no-run` during automated verification.

The author performed self-review. No independent reviewer participated.

### Manual native evidence

After explicit launch permission, the agent launched the native harness with the temporary linear fixture.
The maintainer confirmed this sequence:

1. Control+Option+Space from another app displayed the palette.
2. The palette accepted typed text.
3. Escape dismissed the palette and returned focus to the previous app.

This is maintainer-reported evidence, not an automated OS test.
The maintainer also confirmed this sequence:

1. The flow accepted `jobs` in Section.
2. Command+[ from an unfinished Application input returned to Section with `jobs` selected for replacement.
3. Escape followed by invocation displayed a fresh root without the unfinished draft.

On 2026-11-17, after separate browser-opening permission, the maintainer confirmed the completed linear flow.
The browser received the fixture URL. The palette closed without URLchemy regaining focus.
This confirms the observed opening behavior, not that the destination page loaded.
The harness process then completed without an error in its reported output.
Shortcut conflicts and the other native scenarios below still require manual checks.

## Configure and launch the app

The following procedure launches a visible native application.
Agents must obtain permission before they run it.

1. Run `sh scripts/doctor.sh`.
2. Create `~/Library/Application Support/URLchemy/config.yaml`.
3. Use the [YAML contract](superpowers/specs/2026-11-14-yaml-url-contract-design.md) to define that single file.
4. Run `cargo run -p urlchemy-app --locked`.

The app reads that path once at startup. It does not search the current directory.
Restart URLchemy after you change the configuration or resolve a shortcut conflict.
Invalid configuration produces source-aware diagnostics instead of a partial catalog.

The defaults are Control+Option+Space for invocation and Enter for confirmation.
An optional `shortcuts` mapping replaces either default:

```yaml
shortcuts:
  invoke: ctrl-alt-space
  launch: cmd-enter
```

With this example, Enter does not confirm an answer. Command+Enter confirms it.
Custom global letter shortcuts use physical US-layout key positions.
The default Space shortcut avoids this letter-layout limitation.

Use the Dock or **Open launcher…** in the application menu if registration fails.
The diagnostic identifies the configured combination. The app does not register a fallback combination.
Valid flows remain available through the palette.

Each invocation discards the unfinished session and opens a fresh root.
Escape or Close dismisses the palette without quitting URLchemy.
Explicit dismissal requests restoration of the previous application if URLchemy remains frontmost.
Outside dismissal and successful browser opening do not request that restoration.
**Quit URLchemy** or Command+Q releases the registration and quits the app.

A failed browser request offers Retry, Start over, and Close.
Retry repeats the completed URL request. It does not confirm the core session again.
A successful request means macOS accepted the request, not that the page loaded.

## Automated verification

Run these commands without launching the native executable:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p urlchemy-core --locked
sh scripts/test-doctor.sh
```

If Cargo is absent from PATH, use `$HOME/.cargo/bin/cargo`.
The app tests cover controller transitions, desktop mapping, production-view interaction, and shell lifecycle.
The diagnostic script tests use command stubs and require no credentials or network.

## Gated native acceptance

Compilation alone has no visible native effects:

```sh
cargo test -p urlchemy-app --features native-acceptance --test native_acceptance --no-run --locked
```

The following command **launches the native app and registers a real global shortcut**.
Run it only after explicit permission:

```sh
cargo test -p urlchemy-app --features native-acceptance --test native_acceptance --locked
```

This separate executable creates a temporary fixture home with the production configuration path suffix.
It loads the public linear fixture and passes its catalog to the production shell.
It never reads or changes the user's launcher configuration.
Normal workspace tests exclude this target.
The harness removes the fixture directory after a normal exit.
Forced termination can leave it in the OS temporary directory.

After permission, check the following with the maintainer:

1. Invoke from another app with Control+Option+Space.
2. Check initial text focus, Back, Escape, Close, and fresh repeated invocation.
3. Complete the Section and Application inputs.
4. Obtain separate permission before confirming the fixture's browser destination.
5. Check browser-request failure and recovery where reproducible.
6. Test a shortcut conflict with a temporary fixture variant.
7. Check Dock and application-menu recovery without changing OS shortcuts.
8. Check another Space, fullscreen, multiple monitors, and an IME session with the maintainer.

The supplied native fixture uses default shortcuts.
Custom confirmation and controlled conflict scenarios need a temporary fixture variant, not a change to private configuration.
The compiled harness is not an automated OS acceptance suite.
Record observed results separately from headless results.

## Remaining limits

- Viewport-numbered Command+1…9 activation is not implemented. The palette displays no ordinal hints.
- The full milestone 6 branch-correction and journey matrix remains incomplete.
- The maintainer confirmed global invocation, typing, Escape dismissal, focus restoration, Back, and a fresh root after dismissal and invocation.
- Invocation while a session remains open, other focus scenarios, shortcut conflicts, Spaces, fullscreen, and OS IME behavior remain unverified.
- VoiceOver completeness, screenshot regression baselines, packaging, and signing remain later work.
- A registration-diagnostic banner uses a bounded window instead of automatic compact-height sizing.
- The hotkey bridge can add up to 100ms of idle backoff after an empty wait.
- Headless accessibility queries do not prove complete native accessibility or visual rendering.

The [testing probe](testing-probe.md) describes the verified testing APIs and their limits.
