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

### Journey-first increment

The implementation branch adds headless coverage for fixed destinations, nested navigation, aliases, both dashboard branches, and shared continuations.
It also covers committed-answer review, suffix invalidation, exact URL escaping, mixed input, focus traversal, and context help.
Navigation rows show configured emojis and aliases.
Destination errors and browser-request states no longer display an editable flow.
Long action labels leave space for footer feedback at smaller window sizes.

The journey-first verification passed 98 tests, including 24 production-view UI tests and 16 shell tests.
The core and native adapter dependencies did not change.
The initial journey-first gate compiled the updated native harness without executing it.
The earlier manual evidence applies to the original vertical slice.
Later manual checks appear in the native follow-up section below.

### Viewport-numbered navigation

The ordinal increment adds Command+1 through Command+9 for navigation entries.
Only the first nine fully visible rows receive hints.
Filtering, scrolling, resizing, and base-font changes recompute the numbers.
Partial and offscreen rows receive no hint.
Options and suggestions do not receive navigation shortcuts.

The list now owns a viewport that fits its body region.
This prevents rows from extending beneath the footer in smaller windows.
Headless regressions also verify that scrolling can reach the final row.

The workspace suite passes 115 tests, including 35 production-view UI tests and 18 shell tests.
Four pure tests cover ordinal geometry and stale mappings.
Eleven UI tests cover ordinal presentation, activation, event boundaries, and the native scroll fixture.
Painted-text records verify shortcut content without screenshots.
These records do not establish pixel appearance or VoiceOver behavior.

Verification commands:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p urlchemy-core --locked
cargo test -p urlchemy-app --features native-acceptance --test native_acceptance --no-run --locked
sh scripts/doctor.sh
sh scripts/test-doctor.sh
```

The independent core run passes 39 tests.
The native harness compiles without execution.
The diagnostic regressions pass all three cases.

### Native follow-up and reverse traversal

The maintainer authorized fixture launches after the ordinal implementation.
One run showed a shortcut registration error and a root without an ordinal.
The subsequent process check found no earlier matching process.
Two headless shell regressions verify ordinals after compact resizing and registration failure.
Both pass, but neither reproduces the initial native observation.
The owner of the shortcut conflict remains unknown.

During a controlled run, the maintainer confirmed root ordinal activation, Back, dismissal, and fresh root invocation.
The maintainer then reported that Shift+Tab did not move from the input to Close.
An expanded headless test reproduced this failure.
The input container and editor registered the same focus handle twice.
Removing the container's duplicate registration makes the complete forward and reverse cycles pass.
The workspace suite, formatting, and Clippy pass after this correction.
The maintainer then confirmed native traversal cycles in both directions with the corrected build.
The maintainer also confirmed browser opening and focus behavior with a URL that matched the entered values.
With the 30-folder fixture, the maintainer confirmed native ordinal renumbering after scrolling and filtering.
Partial rows showed no shortcut, and Command+1 entered the indicated fully visible folder.
Filtering for `Folder 2` assigned Command+1 to `Folder 20`.
The maintainer also confirmed that scrolling could reach `Folder 30`.
These are manual observations, not automated OS acceptance results.
VoiceOver remains unverified.

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

### Scroll-only native fixture

Set `URLCHEMY_NATIVE_FIXTURE=scroll` to use 30 empty folders instead of the linear fixture.
This variant contains no flows or URLs.
Ordinal activation enters the indicated folder without opening a browser.
The default fixture remains unchanged when the variable is absent.
Unknown values stop the harness before application startup.

After permission, close previous URLchemy instances and run:

```sh
URLCHEMY_NATIVE_FIXTURE=scroll cargo test -p urlchemy-app --features native-acceptance --test native_acceptance --locked
```

Check numbering after partial-row scrolling and filtering.
Compare the folder entered through an ordinal with the row that displayed that shortcut.
Use Back to return to the root.
Verify that the final folder remains reachable by scrolling.

The compiled harness is not an automated OS acceptance suite.
Record observed results separately from headless results.

## Remaining limits

- Command+1…9 applies only to navigation entries. Options and suggestions do not have ordinal activation.
- Non-ordinal journeys have headless coverage. The full native branch-correction matrix remains incomplete.
- The maintainer confirmed global invocation, typing, Escape dismissal, focus restoration, Back, and a fresh root after dismissal and invocation.
- Invocation while a session remains open, other focus scenarios, shortcut conflicts, Spaces, fullscreen, and OS IME behavior remain unverified.
- VoiceOver completeness, screenshot regression baselines, packaging, and signing remain later work.
- A registration-diagnostic banner uses a bounded window instead of automatic compact-height sizing.
- The hotkey bridge can add up to 100ms of idle backoff after an empty wait.
- Headless accessibility queries do not prove complete native accessibility or visual rendering.

The [testing probe](testing-probe.md) describes the verified testing APIs and their limits.
