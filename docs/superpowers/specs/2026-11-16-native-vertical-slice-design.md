# Early native vertical slice

Status: approved by the maintainer on 2026-11-16, including the proposed dependencies
and a new feature branch from updated `main`.
The executable implementation plan still requires approval.

## Intent and scope

Deliver the first production macOS window connected to the real core.
Prove invocation, a simple configured journey, URL opening, dismissal, and fresh invocation.
Do not redesign the [interaction contract](2026-11-15-interaction-contract-design.md)
or the [shortcut settings](2026-11-15-shortcuts-framework-design.md).

Start with the existing linear fixture and its exact expected URLs.
Read the actual single configuration file once at startup in production.
Tests inject fixture bytes and native service doubles, never private launcher data.
Native acceptance uses a temporary test home containing the same relative configuration path.
Do not add a production configuration search path or a second configuration file.

Milestone 5 proves the native boundary before completing milestone 6's full journey matrix.
It is not a claim of complete branch, suggestion, accessibility, or Spaces support.
The UI must identify any currently unsupported session mode instead of confirming a hidden choice.
Native test results must distinguish implemented behavior from unverified behavior.

## Technical approaches

| Approach | Benefit | Cost | Recommendation |
| --- | --- | --- | --- |
| GPUI Kit with a small `global-hotkey` adapter and AppKit services | Reuses reviewed registration and safe native bindings | Adds native dependencies and their lifecycle rules | Recommended |
| GPUI Kit with application-owned Carbon and Objective-C FFI | Full control over native integration | Adds unsafe code and ownership obligations without a demonstrated need | Not recommended |

The maintainer approved the recommended approach with this design.
Do not use a global keyboard monitor, AppleScript, or Accessibility permissions as a shortcut-registration fallback.

## Dependencies and ownership

Keep `urlchemy-core` independent of GPUI and macOS.
Add one `urlchemy-app` package with a library for production views and a thin executable.
The library lets UI integration tests instantiate the actual production view.

Proposed direct dependencies for the app:

- `gpui-kit = "=0.9.1"`, with default component/assets features.
- `global-hotkey = "=0.10.0"`, restricted to the macOS adapter.
- `objc2 = "=0.6.5"`, for retained native object ownership.
- `objc2-app-kit = "=0.3.2"`, with the narrow workspace and running-application features.
- `objc2-foundation = "=0.3.2"`, for native strings and URLs.
- The local `urlchemy-core` package.

Use the same GPUI Kit version with `test-support` for tests.
Do not enable the alternate `gpui-fast` backend, speech, or editor extensions.
Retain Rust `1.100.0`. Commit the resolved lockfile only after the maintainer authorizes publication.
Adding a selected version to Cargo still requires approval of the execution plan.

The app shell owns the catalog or startup diagnostics, global registration,
window lifecycle, and the previously active application handle.
The palette owns `Session`, its input entity, transient query, active row IDs,
focus handles, subscriptions, and opening/retry presentation.
Core answers and traversal logic must not move into the view.

## Window and recovery access

Use a normal macOS application with a Dock entry for this early slice.
Do not add a menu-bar extra, login item, launch agent, or background service.
Provide `Open launcher` and `Quit` commands in the application menu.
Dock reopen invokes the same fresh-root operation as the global shortcut.
This preserves access to diagnostics when the global shortcut cannot register.

Create at most one visible palette window. Invocation replaces any unfinished session with a fresh root.
Keep the application running when the palette closes so the global shortcut remains available.
Register the invocation binding once, on the main event-loop thread.
Retain its manager until shutdown and release the registration on shutdown.
Do not re-register on each invocation.

The palette is a focused floating utility window using the approved compact structure.
Use the active work area's bounds to clamp its size and position.
Exact cross-Space and fullscreen behavior requires native verification, not an API-name assumption.

Capture the previously active application before activating URLchemy.
On Escape or Close, request restoration only for that invocation's still-running application.
Outside clicks and application switches follow the user's new target instead.
Successful browser opening must not restore the prior application over the browser.
Ignore callbacks associated with an older invocation or a closed palette.

## Global shortcut adapter

Map the validated core key and modifiers to `HotKey::new` directly.
Do not pass the YAML string to the crate's more permissive parser.
Use only the core's letters, digits, Enter, Space, and F1 through F12.
These keys use `RegisterEventHotKey` in the inspected macOS implementation.
They do not enter its separate media-key event-tap path.

Handle both pressed and released events for the registered ID.
Invoke only on a transition to pressed. Ignore repeats until release.
Deliver events through a retained, cancellable bridge into GPUI's main context.
Do not mutate GPUI entities from a blocking channel thread.

Registration failure produces a visible diagnostic with the combination and restart guidance.
It must not silently substitute the default combination.
Keep Dock/menu invocation available, including when configuration validation fails.
An invalid configuration cannot execute flows or register an unvalidated invocation binding.

The crate maps letters to physical US-layout key positions.
The default `ctrl-alt-space` avoids letter-position differences.
Document this early-slice limitation for custom letter bindings.
Do not claim logical-character matching on non-QWERTY layouts without another verified mapping.

## Production palette and core presentation

Use retained GPUI Kit Input and list components with stable core row IDs.
Planning source inspection found that single-line Input removes pasted line breaks.
Use the same editing engine through a one-row Textarea to preserve raw text until core validation.
This component substitution preserves the approved text policy and compact field structure.
Do not adopt stock Command behavior unchanged: its query-clearing Escape differs from the approved dismissal contract.
Separate committed navigation text from input drafts and preserve exact accepted text.
Route pointer activation, configured confirmation, and the footer through the same controller operations.
Install dynamic bindings through the fallible GPUI `KeyBinding::load` path.
Do not treat Enter as an extra confirmation binding when configuration selects another launch combination.

The implementation plan must verify input composition, paste behavior, selection defaults,
and row navigation against GPUI Kit 0.9.1 source before fixing concrete view APIs.
Never intercept composition Enter as an answer confirmation.
No native IME claim follows from ordinary Unicode typing tests.

Add narrow readonly core presentation metadata for context, alias hints, and the next confirmation action.
Do not expose captured answers to the view merely to reconstruct branch context.
Keep IDs private and derive context through the core's existing traversal.

## Browser opening

Do not use GPUI's fire-and-forget `App::open_url` for a recoverable opening operation.
Use `NSWorkspace::openURL`, which returns whether the opening request succeeded.
Convert only the core's validated resolved HTTP(S) URL to a native URL.
Do not interpolate configuration into shell commands or log the completed URL.

A successful opening request closes the palette. It does not prove that a page loaded.
A failed request preserves only the completed URL for Retry, Start over, and Close.
Retry repeats the opening request, not core confirmation.
Dismissal and fresh invocation discard that retained URL.
Prevent duplicate submissions and ignore stale operation results.

## Acceptance and verification

1. Load existing core fixtures and run the complete core suite without GUI dependencies in the core package.
2. Exercise the production view headlessly: root activation, exact text, custom launch binding, Back, dismissal, and reinvocation.
3. Complete the linear fixture and assert its exact URL through an injected opener, without launching a browser.
4. Simulate opening failure, retry, stale completion, and Start over without resuming old answers.
5. Simulate registration failure and prove Dock/menu invocation still reaches the diagnostic state.
6. Verify key/modifier translation and pressed/released deduplication without registering an OS shortcut in automated tests.
7. After separate launch permission, use temporary fixtures for real global invocation, focus, dismissal, browser opening, and repeated invocation.

Read [the testing probe](../../testing-probe.md) before adding UI integration or screenshot tests.
Follow deferred-event boundaries and do not substitute a test-only palette.
Keep screenshot, headless interaction, and native acceptance results separate.
Run workspace format, Clippy with warnings denied, tests, and diagnostic-script regressions.

## Evidence and approval boundary

Source inspection on 2026-11-16 confirmed the global registration API,
main-thread requirement, exclusive registration errors, and physical key mapping in
[global-hotkey 0.10.0](https://docs.rs/crate/global-hotkey/0.10.0/source/src/platform_impl/macos/mod.rs).
The package declares Rust 1.77 and MIT/Apache-2.0 licensing.
Published AppKit bindings expose workspace URL opening and application activation through safe methods.
GPUI source confirms that `App::open_url` returns no error result.
This inspection did not exercise native behavior.

This is author self-review. No independent reviewer participated.
Approval should cover this design, proposed dependencies, and a new feature branch from updated `main`.
An executable task plan and its execution mode must still receive approval before product code changes.
Visible native launch, browser opening, and OS permission requests require separate permission.
