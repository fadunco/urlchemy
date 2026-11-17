# Native Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` for inline execution or `subagent-driven-development` after explicit delegation approval. Track steps with checkboxes.

**Goal:** Connect the production compact palette to the core, configurable macOS invocation, and recoverable browser opening.

**Architecture:** A new app library owns the controller, production view, and desktop service boundary.
The executable composes startup and the native adapters. The existing core remains the sole owner of answers and traversal.

**Tech Stack:** Rust `1.100.0`, GPUI Kit `=0.9.1`, global-hotkey `=0.10.0`, objc2 `=0.6.5`, AppKit/Foundation `=0.3.2`.

**Spec:** [Native design](../specs/2026-11-16-native-vertical-slice-design.md),
[interaction contract](../specs/2026-11-15-interaction-contract-design.md),
[shortcut amendment](../specs/2026-11-15-shortcuts-framework-design.md), and
[testing probe](../../testing-probe.md).

## Global Constraints

- Keep `urlchemy-core` independent of GPUI and macOS.
- Retain Rust `1.100.0` and the existing core dependency pins.
- Use only the approved dependency versions. Do not enable `gpui-fast`, speech, or editor extensions.
- No application-owned unsafe code, shell interpolation, URL logging, or private fixture data.
- Register the invocation binding once, on the main event-loop thread.
- Do not use a global keyboard monitor, AppleScript, or Accessibility permissions as a shortcut-registration fallback.
- Keep Dock/menu invocation available, including when configuration validation fails.
- Do not treat Enter as an extra confirmation binding when configuration selects another launch combination.
- Preserve exact text. Errors appear on confirmation rather than after every keystroke.
- Use the production view in tests. Wait across deferred-event boundaries.
- Native launch, real browser opening, and OS permissions require separate approval.
- No commits, push, merge, worktree creation, or delegated review without task-specific authorization.
- The approved branch is `feat/native-vertical-slice`, based on `origin/main` at `2dffc52747be817af558bda897270a5b046ed745`.

## Review Focus

1. Pasted CR/LF text must not become a valid value through silent editor normalization. Task 4 tests raw paste and rejection.
2. Queued edits and stale opening results must not modify or close a newer invocation. Tasks 2, 4, and 5 test generation boundaries.
3. Registration failure or invalid YAML must leave diagnostics reachable without the global shortcut. Task 5 tests both recovery entry points.
4. Repeated invocation while URLchemy is already active must not replace the original focus target with URLchemy itself. Tasks 3 and 5 cover focus policy.
5. Headless tests must not register real hotkeys or open browsers. Tasks 3–5 inject services and assert the exact boundary requests.

---

## Deliverable boundaries

This slice proves root navigation, simple input construction, correction, dismissal,
and native service integration. The existing linear fixture includes suggestions,
so the basic explicit-selection and literal-value paths belong to this slice too.
Render closed selections through the same rows/controller path and cover one common continuation.
Do not claim the full milestone 6 journey matrix from that single selection test.

Viewport-numbered Command+1…9 activation and its geometry tests remain milestone 6 work.
Do not display ordinal hints or activate offscreen rows before that mapping exists.
Full branch correction, screenshot regression baselines, packaging/signing, and complete
VoiceOver/Spaces/IME coverage remain later work. Report these limits in the run guide.
Do not hide an unsupported state behind a fake empty list or silently confirm its first choice.

## File map and shared interfaces

| Location | Responsibility |
| --- | --- |
| `crates/urlchemy-core/src/session/presentation.rs` (new) | Readonly context and confirmation hints |
| `crates/urlchemy-core/src/catalog.rs`, `src/session/mod.rs`, `src/session/history.rs`, `src/lib.rs` | Minimal presentation exports and internal traversal access |
| `crates/urlchemy-core/tests/presentation.rs` (new) | Core metadata behavior |
| `Cargo.toml`, `Cargo.lock`, `crates/urlchemy-app/Cargo.toml` (new) | App package and resolved dependencies |
| `crates/urlchemy-app/src/lib.rs`, `src/main.rs` (new) | Public app seam and thin native entry point |
| `crates/urlchemy-app/src/launcher.rs` (new) | UI-independent controller around the real session |
| `crates/urlchemy-app/src/desktop.rs` (new) | Injectable desktop boundary and safe error types |
| `crates/urlchemy-app/src/native/{mod,hotkey,workspace}.rs` (new) | macOS registration, opening, home directory, and focus adapters |
| `crates/urlchemy-app/src/hotkey_bridge.rs` (new) | Cancellable background wait and main-context delivery |
| `crates/urlchemy-app/src/palette.rs` (new) | Production GPUI view, editor routing, and list delegate |
| `crates/urlchemy-app/src/shell.rs` (new) | Startup, one-window lifecycle, menus, and effects |
| `crates/urlchemy-app/tests/{controller,desktop,ui,shell}.rs` (new) | Pure, boundary, and headless integration tests |
| `crates/urlchemy-app/tests/support/mod.rs` (new) | Fixtures and deterministic desktop doubles |
| `crates/urlchemy-app/tests/native_acceptance.rs` (new) | Explicitly gated harness using the production shell and view |
| `docs/native-development.md` (new) | Run instructions, permission boundary, and actual evidence |

All paths following a crate name are relative to that crate.
Public structs have private fields and constructors/readers. Keep native handles out of core types.
Application enums below describe new application APIs, not remembered GPUI APIs.

### Application command and effect contract

Define in `launcher.rs` and re-export from the app library:

```rust
pub enum CloseReason { Explicit, Outside, Opened }
pub enum PaletteCommand {
    Edited(String), Activate(RowId), Confirm, Move(i32), Back,
    Dismiss(CloseReason), Retry, StartOver,
    OpenFinished { invocation: u64, operation: u64, result: Result<(), DesktopError> },
}
pub enum PaletteEffect {
    Open { invocation: u64, operation: u64, url: ResolvedUrl },
    Close { invocation: u64, reason: CloseReason },
}
pub struct Launcher; // private retained state
impl Launcher {
    pub fn new(catalog: Rc<Catalog>, invocation: u64) -> Self;
    pub fn invoke(&mut self, invocation: u64) -> bool;
    pub fn dispatch(&mut self, command: PaletteCommand) -> Option<PaletteEffect>;
    pub fn snapshot(&self) -> Result<SessionSnapshot, Diagnostic>;
    pub fn presentation(&self) -> Result<SessionPresentation, Diagnostic>;
    pub fn rows(&self) -> &[Row];
    pub fn active(&self) -> Option<RowId>;
    pub fn query(&self) -> &str;
    pub fn error(&self) -> Option<&Diagnostic>;
    pub fn opening_error(&self) -> Option<&DesktopError>;
    pub fn is_opening(&self) -> bool;
    pub fn invocation(&self) -> u64;
}
```

`Edited` means committed editor text, not IME preedit.
Each dispatch updates one coherent state and emits at most one effect.
Store an opening URL only while pending or available for retry. Redact URL-bearing Debug output.
The shell issues monotonically increasing nonzero invocation IDs with checked increments.
Pass that ID into each new controller. A new window must not reset the sequence.
`invoke` accepts only a newer ID and returns false for a stale or zero ID.
Start over resets the session within its current invocation and preserves the operation counter.
The controller issues monotonically increasing operation IDs. Reject further effects on identifier exhaustion rather than reusing an ID.

## Task 1: Readonly core presentation metadata

**Files:** The core files listed above. Do not change loading or session action signatures.

**Interfaces:** Add these exports, using private fields for `SessionPresentation`:

```rust
pub enum PrimaryActionHint { OpenFolder, StartFlow, Continue, OpenUrl, ChooseOption, Unavailable }
pub struct SessionPresentation;
impl SessionPresentation {
    pub fn context(&self) -> &[String];
    pub fn primary_action(&self) -> PrimaryActionHint;
    pub fn can_go_back(&self) -> bool;
}
impl Session {
    pub fn presentation(&self, catalog: &Catalog, active: Option<RowId>)
        -> Result<SessionPresentation, Diagnostic>;
}
impl Entry { pub fn alias(&self) -> Option<&str>; }
```

Context contains configured folder names, the flow label, and committed branch-choice labels.
It contains no captured input values, URLs, internal IDs, or future questions.
For browsing, derive the hint from the supplied active entry. For a selection,
use the supplied option or the restored core choice. Without either, return `ChooseOption`.
An input whose next step is a destination returns `OpenUrl`, even before its draft is valid.
Destination errors return `Unavailable` with Back available.
Validate supplied IDs against the current rows and reject cross-catalog IDs.
Reuse the existing history trace, exposing it only within the session module if necessary.

- [ ] Write `presentation_tracks_context_without_answers`, `confirmation_hint_follows_the_current_definition`, and `presentation_rejects_stale_targets`.
  Use the branching, common, fixed, and linear fixtures through real session actions.
  Assert `OpenFolder` for Datadog, `StartFlow` for its dashboard flow, and context ending in `HTTP Traffic` after that choice.
  Assert `Continue` at Section and `OpenUrl` at Application in the linear fixture.
  Confirm `accounts`, then assert it never appears in returned context.
  Assert Back availability at root, nested browsing, and input states. Assert presentation queries do not change the session snapshot.
- [ ] Run `cargo test -p urlchemy-core --locked --test presentation`. Expected: RED for the missing presentation API.
- [ ] Implement the new module and exports without duplicating branch traversal or exposing the answer cache.
- [ ] Run the target again and `cargo test -p urlchemy-core --locked`. Expected: all tests pass.

## Task 2: App package, controller, and service ports

**Files:** Root manifest/lockfile, app manifest, `lib.rs`, `launcher.rs`, `desktop.rs`, and `tests/controller.rs`.
Create a minimal app library before its tests. Add the native executable only in Task 5.

**Consumes:** Task 1 metadata and the existing `Session`, filtering, shortcuts, and resolved URLs.
**Produces:** The controller contract above and these service APIs:

```rust
pub enum DesktopError { HomeUnavailable, DisplayUnavailable, RegistrationFailed, NativeUrlRejected, OpenFailed }
pub struct HotkeySignal; // new(id: u32, pressed: bool), id(), is_pressed()
pub type HotkeyWait = Arc<dyn Fn(Duration) -> Option<HotkeySignal> + Send + Sync>;
pub struct RegisteredHotkey; // new(id: u32, wait: HotkeyWait), id(), wait()
pub trait DesktopServices {
    fn configuration_path(&self) -> Result<PathBuf, DesktopError>;
    fn register_invocation(&self, shortcut: &Shortcut) -> Result<RegisteredHotkey, DesktopError>;
    fn open_url(&self, url: &ResolvedUrl, cx: &App) -> Task<Result<(), DesktopError>>;
    fn remember_frontmost(&self, invocation: u64);
    fn restore_frontmost(&self, invocation: u64);
    fn discard_frontmost(&self, invocation: u64);
    fn is_launcher_frontmost(&self) -> bool;
    fn work_area(&self, cx: &App) -> Result<Bounds<Pixels>, DesktopError>;
    fn shutdown(&self);
}
```

The native service retains the registration manager. `RegisteredHotkey` exposes event delivery metadata, not ownership of the manager.
The service is main-thread-owned, behind `Rc<dyn DesktopServices>`, not `Send`.
Only `HotkeyWait` crosses to a background worker. Use `RegisteredHotkey::wait() -> HotkeyWait` to clone its handle.
Signal and registration `id()` readers return `u32`. The signal's `is_pressed()` reader returns `bool`.
Implement Display for `DesktopError` with static messages and no native URL payload.
Use `Could not open the browser.` for both native URL rejection and opening failure.
Explain registration failure with the configured shortcut and restart guidance in the diagnostic view, not in a secret-bearing error object.

Add the app to workspace members. Use GPUI Kit defaults and the identical dev dependency with `test-support`.
Put global-hotkey and Objective-C dependencies under the macOS target table.
Disable AppKit/Foundation default features. Enable AppKit `std`, `NSWorkspace`, `NSRunningApplication`, `NSScreen`, and `NSEvent`.
Enable Foundation `std`, `NSString`, `NSURL`, `NSPathUtilities`, and `NSGeometry`.
Use `#![forbid(unsafe_code)]` in the app library and executable.
Resolve dependencies once with `cargo check -p urlchemy-app`, without a blanket update.
Use `--locked` for every subsequent verification command.

- [ ] Write controller tests before defining the controller body.
  `literal_and_suggestion_paths_emit_exact_urls` must resolve `Jobs` literally without selection and `jobs` after explicit Jobs selection.
  With Application `accounts`, assert the two exact URLs from the existing expectations manifest.
  `back_and_dismiss_preserve_only_committed_history` must discard an unconfirmed Application draft and reset to root on invocation.
  `opening_retry_is_not_core_confirmation` must repeat one failed URL request without calling Confirm on Completed.
  `stale_results_cannot_close_a_new_invocation` must ignore the old invocation and old retry operation IDs.
  Test controllers for two replacement windows with shell-issued IDs 1 and 2.
  An opening result from 1 must not close 2.
  `row_selection_and_filtering_are_explicit` must cover case-insensitive filtering, first navigation highlight, no initial suggestion selection, and nonwrapping movement.
  Add one common-continuation selection journey and a resolution-error Back recovery case.
- [ ] Run `cargo test -p urlchemy-app --locked --test controller`. Expected: RED for the missing controller behavior.
- [ ] Implement controller actions around the real session. Use stable `RowId`s rather than stored filtered indices.
  Folder/flow clicks activate immediately. Choice activation selects and confirms. Suggestion activation selects without confirmation.
  Editing clears explicit suggestion selection and stale input errors. Use core validation rather than app-side trimming.
  Keep one pending open operation, reject duplicate Confirm/Retry, and clear retained URLs on dismissal or fresh invocation.
- [ ] Run the controller target and core suite. Expected: all tests pass, with no new native effect in tests.

## Task 3: macOS adapters and cancellable hotkey bridge

**Files:** `native/mod.rs`, `native/hotkey.rs`, `native/workspace.rs`, `hotkey_bridge.rs`, and `tests/desktop.rs`.

**Interfaces:** `pub struct MacDesktopServices` implements `DesktopServices`.
Expose `MacDesktopServices::new() -> Self`. Retain native objects privately.
Add `pub fn native_hotkey(shortcut: &Shortcut) -> Result<global_hotkey::hotkey::HotKey, DesktopError>` for the verified mapping.
Add `pub struct HotkeyLatch` with `new(id: u32)` and `accept(&mut self, signal: HotkeySignal) -> bool`.
Add `HotkeyBridge::start(wait: HotkeyWait, on_signal: impl FnMut(HotkeySignal, &mut App) + 'static, cx: &App) -> Self`.
Retain its foreground Task and cancellation flag. Dropping it must stop delivery.

- [ ] Write `native_mapping_matches_every_allowed_key`, `press_release_latch_ignores_repeats`, and `bridge_does_not_deliver_after_drop`.
  Assert Command maps to META, Control to CONTROL, and that `ctrl-alt-space` maps to Space with both modifiers.
  Cover every letter, digit, Enter, Space, and F1–F12 using typed fields, not the crate string parser.
  For one registered ID, assert Pressed/Pressed/Released/Pressed produces true/false/false/true. Unrelated IDs must not reset the latch.
  Test the bridge with an injected wait closure and real GPUI context, not OS registration.
  Test focus-restoration policy with abstract application IDs: retain the external target when the current frontmost ID is URLchemy's ID.
- [ ] In `native/workspace.rs`, write unit tests for a private `FocusLease<T>` before its implementation.
  Give it `remember(invocation: u64, external: Option<T>)`, `take(invocation: u64, launcher_frontmost: bool) -> Option<T>`, and `clear(invocation: u64)`.
  `None` in remember means URLchemy is already frontmost: retain the prior target and retag it to the new invocation.
  Clear the lease separately when frontmost application information is unavailable.
  Assert stale take/clear operations leave the current lease intact, and outside dismissal never returns a restoration target.
- [ ] Run `cargo test -p urlchemy-app --locked --test desktop`. Expected: RED for missing mapping, lifecycle, and delivery behavior.
- [ ] Run `cargo test -p urlchemy-app --locked native::workspace::tests` for the FocusLease tests. Expected: RED before the policy implementation.
- [ ] Implement typed hotkey translation and registration on the main thread.
  Map letters to `Code::KeyA` through `Code::KeyZ` and digits to `Code::Digit0` through `Code::Digit9`.
  Construct `HotKey::new` directly and map registration errors to `RegistrationFailed`.
  Clone `GlobalHotKeyEvent::receiver()` into the wait closure. Do not install its process-global one-time callback.
  Wait through `recv_timeout` with a 100ms maximum in GPUI's background executor. Never block the UI executor.
  Await each background wait from the retained foreground task and invoke the callback only through the live app context.
  Check cancellation before and after each wait. Do not notify or render on timeout.
- [ ] Implement workspace services using the inspected safe APIs.
  Use `NSHomeDirectory` plus core `configuration_path`, not current-directory search or shell expansion.
  Use `NSWorkspace::openURL` and return `Task::ready(Ok(()))` or a typed failure. Never include the URL in error text.
  Retain the prior `NSRunningApplication` with its invocation ID. Restore only for a matching explicit dismissal while URLchemy remains frontmost.
  Discard stale focus tokens without activating anything. A successful activation request is not proof of focus restoration.
  Select the screen containing the mouse from `NSScreen::screens`, with `mainScreen` as fallback.
  Convert `visibleFrame` to GPUI's top-left coordinate system using the primary screen's frame. Preserve negative monitor coordinates.
  If native screen information is unavailable, use `cx.primary_display()` and the returned display's `visible_bounds()`.
  Return `DisplayUnavailable` when neither path supplies a display. Do not guess screen dimensions.
  Make service shutdown idempotent by taking and dropping the retained registration manager only once.
- [ ] Add pure conversion tests for a secondary screen above and left of the primary screen, and for a Dock-reduced visible frame.
- [ ] Run the desktop target, workspace policy unit tests, and app controller target. Expected: all tests pass.
  Do not call the real opener or registrar during these checks.

## Task 4: Production palette and editor event routing

**Files:** `palette.rs`, `tests/ui.rs`, and `tests/support/mod.rs`.

**Interfaces:** Export `PaletteView` and `PaletteEvent`.
Use `PaletteView::new(launcher: Launcher, window: &mut Window, cx: &mut Context<Self>) -> Self`.
Emit `PaletteEvent::Effect(PaletteEffect)` for the shell. Keep the retained controller private.
Expose `pub fn dispatch(&mut self, command: PaletteCommand, window: &mut Window, cx: &mut Context<Self>)` for shell responses and shared command paths.
Do not expose test-only confirmation methods or duplicate launcher state for assertions.

Use a one-row `TextareaState::new(window, cx).rows(1).submit_on_enter(true)` with `Textarea`.
Single-line Input strips CR/LF before the core can reject it. This substitution preserves the approved text policy.
Use `set_value` for programmatic restoration because the inspected method emits no Change event.
Subscribe to `InputEditsEvent` and consume the immutable text snapshot rather than reading a possibly later field value.
The inspected editing engine does not emit Change or immutable edit batches for marked preedit text.
Replace the editor entity and subscription on a navigation/step transition, not during render.
Tag subscriptions with that editor generation so queued old edits cannot enter the next step.
Use the public `EntityInputHandler::marked_text_range` path to check composition before routing palette commands.

Use `ListState<PaletteListDelegate>` with `searchable(false)` and semantic `ListItem` rows.
The delegate maps indices to current RowIds. The controller remains authoritative for selection and activation.
Keep focus in the editor during arrow navigation. Use row accessibility labels and stable domain-derived element IDs.
Do not enable List's asynchronous search path or show loading for local filtering.
Build the context, field, scroll body, and footer from theme tokens and rem-based helpers.
Use real Buttons for primary action, Back, Close, Retry, and Start over.

- [ ] Write UI integration tests using `gpui_kit::open_window`, the production view, and fixture catalogs.
  Render a frame, establish focus explicitly, dispatch native input, leave the window update, call `run_until_parked`, then inspect a new frame.
  Use `TestWindowExt` and accessible-label queries for the actual Textarea value.
  Add an observed `launcher-field` wrapper tracking the editor's real FocusHandle for keyboard targeting.
  Button IDs are `primary`, `back`, `close`, `retry`, and `start-over`.
  `linear_journey_uses_the_real_view` must select Jobs explicitly and emit the exact accounts/jobs URL.
  `visible_suggestion_does_not_replace_literal_text` must confirm literal `Jobs` and emit the case-preserving expected URL.
  `custom_launch_replaces_enter` must leave the answer unconfirmed after Enter with `cmd-enter` configured, then advance exactly once on Command+Enter.
  `paste_preserves_invalid_controls_until_confirmation` must paste `jobs\narchive` and `jobs\rarchive`, then observe a rejected confirmation and no opening effect.
  `composition_commit_does_not_confirm` must compose marked text with the actual editor InputHandler, commit it, and assert no traversal on that event.
  `old_editor_events_do_not_replay_alias_text` must activate a fixture alias without copying it into the first input.
  Test Back selection-for-replacement, Escape with nonempty text, no initial suggestion selection, and stopped arrow boundaries.
- [ ] Run `cargo test -p urlchemy-app --locked --test ui`. Expected: RED for missing production view behavior.
- [ ] Implement dynamic confirmation actions with fallible `KeyBinding::load` after `gpui_kit::init`.
  Register handlers on the same focus path as their key context. Do not use a second PressEnter confirmation subscription.
  Ignore held-key repeat for confirmation, Back, and dismissal. Preserve ordinary text editing and modified arrows.
  Capture composition state before child action dispatch. Consume a composition-cancelling Escape without also closing.
  Let owned input menus consume Escape before palette dismissal. Do not turn transient owned surfaces into outside-click dismissal.
- [ ] Run the UI target again. Expected: all tests pass. Add positive geometry checks for shared leading edges, field placement, and footer containment.
  Repeat geometry checks at base font sizes 14 and 20, and after a smaller viewport resize.
  Keep the field and footer visible while only the result body scrolls.

## Task 5: Startup, one-window shell, and recovery entry points

**Files:** `shell.rs`, `main.rs`, app manifest, and `tests/shell.rs`.

**Interfaces:** Export `StartupState { Loading, Ready(Rc<Catalog>), Failed(StartupFailure) }` and
`StartupFailure { Configuration(Diagnostics), Desktop(DesktopError) }`.
Export `AppShell::install(initial: StartupState, services: Rc<dyn DesktopServices>, cx: &mut App) -> Entity<AppShell>`.
Add `AppShell::invoke(&mut self, cx: &mut Context<Self>)` and `AppShell::shutdown(&mut self, cx: &mut Context<Self>)`.
Store a global shell entity for OpenLauncher/Quit actions. Keep its subscriptions and bridge alive until shutdown.

- [ ] Write shell integration tests before the lifecycle implementation.
  Inject fake services that record boundary requests and return configured results.
  `registration_failure_keeps_menu_recovery` must reject registration, dispatch OpenLauncher without a focused window, and expose a diagnostic plus Close.
  `invalid_config_remains_reachable_without_a_binding` must show source-aware startup diagnostics and make zero registration/opening requests.
  `reopen_and_hotkey_share_fresh_invocation` must route both through invoke, keep at most one window, and discard partial answers.
  `open_failure_retry_and_start_over` must issue the same URL twice on Retry and return to root on Start over.
  `old_completions_do_not_close_new_windows` must deliver a previous operation result after invoking again and retain the new root.
  `focus_restore_depends_on_close_reason` must request restoration only for Explicit, never Outside or Opened.
  `missing_display_can_recover_on_later_invocation` must create no window on display failure, then open root when a display becomes available.
  Assert that recovery does not reload configuration or register the shortcut twice.
  `shutdown_releases_registration_once` must drop the bridge and service registration without leaving another window or reader task.
- [ ] Run `cargo test -p urlchemy-app --locked --test shell`. Expected: RED for missing shell behavior.
- [ ] Implement startup and shell effect handling.
  Resolve the single path through desktop services, then run `config::load_file` once in a background task.
  Read only for initial Loading. A supplied Ready or Failed fixture state must not trigger a user configuration read.
  Maintain Loading until that real operation finishes. Publish either the complete catalog or a startup failure.
  Install one global registration only after successful validation. Do not register a fallback combination on invalid configuration.
  Keep the registration error visible through Dock/menu invocation without disabling valid pointer-driven flows.
  Initialize Kit once, set `ActivationPolicy::Regular`, bind keys before building native menus, and register the reopen callback.
  On initial application launch, invoke once. OpenLauncher and Dock reopen use the same operation thereafter.
  Opening a new invocation removes the previous palette window and invalidates its effects before creating the new production view.
  Increment the shell's invocation counter before replacement and pass that ID to `Launcher::new`.
  Closing the palette does not quit the app. Quit shuts down the services and then quits GPUI.
- [ ] Use `WindowKind::PopUp` for the floating launcher and `gpui_kit::open_window` for its Root ownership.
  Capture the external focus target before `App::activate(true)` and `Window::activate_window`.
  Observe activation changes through a retained `observe_window_activation` subscription.
  On deactivation, consult `is_launcher_frontmost`: owned menus/composition surfaces must not cause outside dismissal while URLchemy remains frontmost.
  Explicit Close/Escape restores focus only through the matching service token. Outside dismissal must not steal focus from the clicked app.
  On `DisplayUnavailable`, create no window and report a static error without configuration values.
  A later menu invocation may retry display discovery without reloading configuration or re-registering the shortcut.
- [ ] Implement pure `palette_bounds(work_area: Bounds<Pixels>, rem: Pixels, preferred_height: Pixels) -> Bounds<Pixels>` in `shell.rs`.
  Prefer 40rem width, constrain to 48rem maximum, and clamp to the work area with 1rem clearance.
  Treat 32rem as a minimum target, never an offscreen-forcing minimum.
  Center the initial panel in the chosen work area. Keep its top anchor stable when short-list height changes.
  Bound the result body to 24rem or remaining height. Use measured layout rather than a fixed empty-row count.
  Test a 320-point-wide work area and a larger base font, requiring all panel edges inside the work area.
- [ ] Run shell, UI, desktop, and controller targets. Expected: all tests pass without a real window, registration, or browser.

## Task 6: Gated native acceptance, run guide, and full verification

**Files:** `tests/native_acceptance.rs`, app manifest, `docs/native-development.md`, and this plan's execution record.

Add a non-default `native-acceptance` feature and a `harness = false` test target requiring it.
Its main function creates a temporary fixture directory and calls the production
shell/view with a fixture catalog and real native services. It must not read the user's launcher configuration.
Use the same core configuration path suffix under that temporary directory.
This harness is an explicit native entry point, not a production CLI override or a second UI implementation.
Do not include its feature in normal workspace tests.

- [ ] Document `cargo run -p urlchemy-app` as a native launch, not a verification command to run without permission.
  Document single-file setup, restart behavior, Dock recovery, quit behavior, and physical-letter shortcut limitations.
  State which milestone 6 capabilities remain unimplemented or unverified, including viewport ordinals.
- [ ] Run `sh scripts/doctor.sh` and `sh scripts/test-doctor.sh`. Expected: prerequisites available and all three harness scenarios pass.
- [ ] Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked`.
  Expected: all checks pass. Run `cargo test -p urlchemy-core --locked` separately to verify the core independently.
- [ ] Compile, but do not run, the native harness with `cargo test -p urlchemy-app --features native-acceptance --test native_acceptance --no-run --locked`.
  Expected: the target compiles without registering a hotkey or opening a window.
- [ ] Review changed prose, local documentation links, the dependency graph, lockfile changes, and the complete diff.
  No unrelated core dependency upgrades, secrets, fixture mutations, build outputs, or new private paths may enter the proposed commit.
- [ ] Request explicit permission before running the native harness or opening its fixture destination.
  Without permission, stop that verification path and report native acceptance as pending rather than claiming milestone 5 complete.
- [ ] If authorized, check global invocation from another app, text focus, custom confirmation, Back, dismissal, browser opening, and repeated invocation.
  Repeat with an occupied invocation shortcut, then use Dock/menu recovery. Restore no user configuration and change no OS shortcuts.
  Check another Space/fullscreen and an IME session only with the maintainer's participation. Record actual results, including failures and untested cases.
- [ ] Finish with changed files, executed commands, results, self-review limits, and remaining native blockers. Do not commit or publish without a new request.

## Self-review and execution handoff

The plan covers the native design through Tasks 1–6. The conditional native gate
remains explicit because automated evidence cannot establish activation or OS focus behavior.
The five review risks each map to a named test. The controller, view, shell,
and native adapters share the same command/effect and desktop-service contracts.

Two implementation details refine the approved design without changing product behavior.
A one-row Textarea preserves CR/LF. A cancellable 100ms background wait bridges
the existing hotkey receiver without adding a channel dependency.
Neither choice authorizes a different text policy or real native test execution.

This is author self-review. No independent reviewer or subagent participated.
The previous increment used inline execution. Confirm that mode for this new plan.
Recommend inline execution with TDD and honest self-review because the tasks share
one controller and shell contract. A delegated review remains available after explicit approval.
