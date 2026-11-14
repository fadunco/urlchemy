# GPUI Kit agent-testing probe

## Scope and result

On 2026-11-12, a disposable package outside the URLchemy repository verified that
an agent can exercise real GPUI Kit components without opening a visible window.
No application workspace, native automation plugin, MCP server, OS accessibility
permission, commit or push was needed for this experiment.

Environment: macOS ARM64, Xcode 27.2, working Metal Toolchain, Rust 1.100.0,
`gpui-kit = "=0.9.1"`, default component/assets features. The dev dependency used
the identical package version with `features = ["test-support"]`. A generated
Cargo.lock retained the resolved graph. These are verified probe versions, not
yet the application's selected dependencies.

**Result:** three UI integration tests and a separate real-Metal offscreen
rendering executable passed. The fixture passed rustfmt and clippy with warnings
denied. No URLchemy application tests exist yet.

## What the fixture exercises

One retained `CommandState` renders a stock `Command`: its actual input and list,
with Calendar, disabled Calculator, Catalog and Notes entries. An ordinary
"Outside palette" Button provides a second focus target. The fixture owner
records confirmation index paths and cancel counts; no URL or shell command is
executed. Presentation uses theme colors and rem-scale spacing.

| Check | Evidence |
| --- | --- |
| Focus entry and return | Establish the initial Tab stop with GPUI's traversal primitive, then dispatch Tab and Shift-Tab. Assert native focus snapshots for the Button and input. |
| Real Unicode editing | Type `José 中文 🦀` at current focus; assert the input value, then Backspace removes the emoji. This is not an OS IME test. |
| Filtering | Type `ca`; assert three matches, absence of Notes, and retained original item coordinates. |
| Arrow navigation | Down skips disabled Calculator, Up returns, and input retains focus. |
| Enter | Confirm Catalog through the key event and assert exactly one deferred owner callback with its original index path. |
| No matches | Enter with no selected match produces no confirmation. |
| Disabled pointer interaction | Dispatch a click at Calculator; assert no confirmation and no changed selection. |
| Escape | With text, clear the query without cancelling; with an empty query, invoke cancellation once without confirmation. The fixture records cancellation; it does not dismiss a real launcher. |
| Geometry | Assert positive row dimensions, rows below the input, correct ordering and containment in the fixed test viewport. |
| Actual pixels | Use the native offscreen Metal renderer to save `initial.png` and `filtered.png`; assert matching dimensions and changed pixel buffers, then inspect both PNGs. Two captures of the unfocused static initial state were identical on this machine. |

## Practical findings for agents

1. **Events have scheduling boundaries.** Typing updates the input before its
   subscription has finished filtering the command list. Leave the borrowed
   window update, call `TestAppContext::run_until_parked()`, then render/query a
   new frame. Likewise, observe confirmation callbacks after deferred work runs.
   An immediate assertion inside the typing update incorrectly saw the old list.
2. **Do not assume a pointer click establishes keyboard focus.** A click on the
   fixture Button did not focus it. Establish a known focus target explicitly,
   assert it, and then test real keyboard traversal.
3. **Accessibility snapshots are incomplete.** In published GPUI Kit 0.9.1,
   `Command`'s disabled row returned `None` for `disabled()`. Source inspection
   confirmed its row builder does not set `aria_disabled`, although it suppresses
   handlers and keyboard selection. Prove behavior with events; do not interpret
   an absent flag as enabled or disabled. This needs attention before claiming
   accessibility completeness or choosing this control unchanged for production.
4. **Semantics and pixels complement each other.** A label snapshot is an
   accessible name, not proof of rendered text. Offscreen capture supplies actual
   pixels and can be inspected with the agent's image-read tool.
5. **Use two test contexts deliberately.** `#[gpui_kit::test]` supplies
   `TestAppContext` for deterministic interaction tests. The rendering executable
   uses `HeadlessAppContext::with_platform`, the platform text system, bundled
   Assets and `platform::current_headless_renderer`. It opens a `show: false`
   window and calls `capture_screenshot`; it does not substitute a fake renderer.

## Commands used in the disposable package

```sh
cargo generate-lockfile
cargo test --locked --test ui -j 4 -- --nocapture
cargo test --locked --test rendering -j 4
cargo test --locked --tests -j 4
cargo fmt --all -- --check
cargo clippy --locked --all-targets -j 4 -- -D warnings
```

The package is temporary; these are not commands to run from URLchemy's root.
The renderer target uses `[[test]] name = "rendering", harness = false`. Artifacts
stay in that package's `artifacts/` directory, not in the application repository.
When application code exists, import its production view into tests instead of
shipping a separate lookalike test implementation. Do not depend on temporary
paths or cache directories in the application's build/test setup.

## What remains unverified

No global shortcuts, native activation/repeated summon, focus restoration to
another app, overlay dismissal/focus restoration, Spaces, fullscreen, multiple
monitors, OS IME candidate windows, VoiceOver or packaged-app behavior were tested.
Those still require targeted native coverage. No browser was launched.

The pixel check proves the fixture renders and responds visually; it is not a
golden-image regression suite or a performance benchmark. Cross-machine pixel
equality requires controlled fonts, theme, display scale, animations and renderer
versions. Focused caret blinking also makes naive exact-image assertions brittle.

## Sources inspected

- Versioned project skill: `.opencode/skills/gpui-kit/references/gpui/test.md`.
- Published GPUI Kit 0.9.1 source: `src/test.rs`, `tests/ui.rs`,
  `tests/rendering.rs` and the integration focus tests.
- Published gpui-component 0.9.1 source: `src/command/command.rs`,
  `src/command/state.rs`, `src/command/item.rs` and `src/input/input.rs`.

The installed release, not the latest website or a remembered API, was the source
of truth for this probe. No additional plugin or skill pack was necessary.
