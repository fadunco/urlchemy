# Development harness

## Chosen baseline

- **OpenCode V2**, with the existing provider/model selection inherited. No credentials in the repository.
- **Rust + GPUI + GPUI Kit** as the initial desktop UI direction, not yet an application dependency selection.
- Official `gpui-kit` and `gpui-kit-design-guides` skills installed project-locally at the revision in `THIRD_PARTY_NOTICES.md`.
- Project-owned `rust-development` and `github-workflow` skills, also available for optional global installation.
- No MCP servers. The maintainer subsequently requested global Superpowers and find-skills; see [the toolkit integration guide](agent-toolkit.md). Multi-agent execution still requires approval for the task/plan.
- The initial harness setup did not scaffold the application or publish anything. The maintainer subsequently approved initializing Git and publishing the harness to the public `fadunco/urlchemy` repository under MIT.

The two general skills are versioned in this project, so contributors do not need the global copies. The global installer copies only these two skills and refuses to replace different existing files. Project skills take precedence in V2. Global OpenCode configuration now includes the explicitly requested Superpowers plugin and global workflow instructions; provider settings and credentials are unchanged.

## Why these choices

GPUI is the UI framework behind Zed and has a longer real-application track record. GPUI Kit adds components and first-party-maintained agent guidance. Both remain evolving APIs: compile against a chosen compatible version, rather than relying on generic GPUI examples. The future application's Rust version and dependencies still need to be selected and pinned.

Rune is a credible alternative, particularly for agents: its current documentation provides PNG snapshots, element/accessibility trees, scripted keyboard input, global-shortcut simulation and launcher window support. It is explicitly pre-release and requires a pinned revision and matching patches. We have built and manually checked a minimal GPUI Kit window, but have not built Rune or benchmarked either framework. Prefer GPUI initially for the maintainer's maturity criterion, not because Rune lacks agent tooling.

Sources checked during setup:

- https://github.com/zed-industries/zed/tree/main/crates/gpui
- https://github.com/longbridge/gpui-kit/tree/6653d1c024b5461461143c90b005d80089cc085f/skills
- https://github.com/anomalyco/rune
- https://github.com/anomalyco/rune/blob/dev/crates/rune/README.md
- https://opencode.ai/docs/config
- https://opencode.ai/docs/skills
- https://opencode.ai/docs/permissions
- https://opencode.ai/docs/commands

The initial lean setup did not install Superpowers. The maintainer later explicitly requested it as the general process toolkit, plus Vercel's Skills CLI and find-skills. They are now installed globally at reviewed revisions, with approval/preservation exceptions in AGENTS.md. They complement rather than replace the GPUI/Rust technical skills. The maintainer also requested global ASD-STE100 writing guidance, documented in the toolkit guide. General frontend/web design skills and Playwright remain unnecessary for this native UI.

## Machine prerequisites

Run `sh scripts/doctor.sh` from the repository root. A nonzero exit identifies outstanding prerequisites; it is not a failed application test. Metal is checked by executing `xcrun metal --version`, not merely finding its path: Xcode can expose a shim even when the optional toolchain is missing.

Run `sh scripts/test-doctor.sh` to exercise the diagnostic against local command stubs. It checks a healthy environment, a discoverable-but-unusable Metal compiler, and a legacy OpenCode CLI. These tests do not use real credentials, install software or access the network.

### OpenCode CLI mismatch

At setup, `/Applications/OpenCode.app` and its bundled CLI are **2.2.7**, while `opencode` on PATH is **1.19.5**. The global config initially contained only the schema declaration; it now also pins Superpowers under V2's `plugins` field. The existing global package.json references the V1 plugin SDK; it was not modified and should be left alone unless a migration is explicitly requested.

Use the V2 CLI without replacing the old binary:

```sh
/Applications/OpenCode.app/Contents/Resources/opencode-cli --version
/Applications/OpenCode.app/Contents/Resources/opencode-cli .
```

This path is machine-specific guidance, not a shared runtime config entry. Config and skill changes reload automatically in V2; use `/reload` if needed, not a service restart. Do not use the V1 CLI to interpret this project's V2 config. Aligning the default PATH CLI remains an explicit follow-up, not an authorized upgrade.

To run the doctor against the bundled CLI:

```sh
OPENCODE_BIN=/Applications/OpenCode.app/Contents/Resources/opencode-cli sh scripts/doctor.sh
```

### Rust

Install stable Rust with rustup, minimal profile plus rustfmt and clippy. The installer used for this setup comes from https://sh.rustup.rs. It was run with `--no-modify-path`: shell startup files are not changed.

If needed in a terminal session:

```sh
. "$HOME/.cargo/env"
cargo --version
```

Agents may use `$HOME/.cargo/bin/cargo` directly. The project's rustfmt command uses the rustup path so formatting does not depend on a GUI client's PATH. The configured edition is 2024; confirm it when creating Cargo.toml. LSP is not configured: compiler/clippy/test output is the initial feedback loop; rust-analyzer can be evaluated later.

### Xcode / Metal

The initial machine had only Command Line Tools selected. Xcode 27.2 and the Metal Toolchain are now installed and selected. For another contributor's machine, GPUI's documented macOS setup requires Xcode and Metal tooling. Install Xcode from Apple, launch it and complete the macOS component/license setup. Then select the actual installation, for example:

```sh
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
xcodebuild -version
xcrun metal --version
```

If the compiler reports a missing Metal Toolchain, install the component explicitly:

```sh
xcodebuild -downloadComponent MetalToolchain
```

These are manual instructions, not commands the agent should run without approval. Command Line Tools alone, or a successful `xcrun --find metal`, are not proof that the compiler can execute. Setup verification also compiled a minimal shader to AIR and linked it into a Metal library.

### GitHub

GitHub CLI is installed through Homebrew. Authentication is user-driven:

```sh
gh auth login --web
gh auth status
gh api user --jq .login
```

Do not share tokens in chat. The authenticated owner is `fadunco`; the maintainer approved the public `fadunco/urlchemy` repository under MIT, including Git initialization, the initial harness commit and push. Future commits and remote mutations still require explicit approval. The initial commit uses a repository-local Git identity with GitHub's noreply address rather than publishing a personal email.

## Working loop

1. Agree on a small task, acceptance criteria, proportionate design/plan and execution mode using the relevant Superpowers process skills.
2. Load domain skills and verify the selected framework API against source. Use find-skills only when another capability is genuinely missing; further installations need approval.
3. Follow test-first for production changes and implement a small change; keep disposable exploration separate from product implementation.
4. Run `/check`; use `/review` for a non-editing, in-session review. An independent reviewer or subagent-driven plan needs explicit delegation approval; do not claim self-review is independent.
5. Use `/prepare-pr` to draft a PR description without publishing.
6. The user approves commits and remote mutations explicitly.

Shell defaults to confirmation, with a small read-only git allowlist. This intentionally also asks before builds/tests (which execute build scripts/code), installations, native launches, and gh operations. These rules are safeguards, not an OS sandbox; broader global/agent rules or saved approvals can change effective access. Instructions require approval even where another configuration allows a tool.

## Verification once the application exists

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Do not create a fake CI workflow or green application-test claim while Cargo.toml is absent. Add CI after a real workspace and lockfile exist; it should exercise the same checks on macOS.

For the native launcher, eventually verify activation/repeated summon, focus restoration, global shortcuts and conflicts, Escape/Enter/arrow routing, keyboard layouts/IME, multiple displays, Spaces/fullscreen and VoiceOver. Headless tests cannot prove OS integration. Opening URLs must be mockable; tests should not launch real browser sessions or access private configuration.

### Verified agent-testing path

A separate temporary GPUI Kit 0.9.1 probe passed three UI integration tests for input, filtering, keyboard/focus and negative cases, plus a real-Metal offscreen screenshot check. See [the testing probe report](testing-probe.md) for evidence, exact scope, scheduling rules and an accessibility limitation discovered in the published Command component. The fixture and PNGs remain outside this repository; these are framework experiments, not URLchemy application tests.

## Updating and removal

- Review upstream changes before updating the framework skill revision and notices. The installer refuses existing skill directories rather than overwriting work.
- Global copies are independent snapshots. Updates require review if files differ; deleting or renaming project files does not silently change global behavior.
- To undo global skills, inspect and remove only `~/.config/opencode/skills/rust-development/` and `github-workflow/` if they still contain the copies installed here. Do not remove unrelated skills, SDK packages, OpenCode state or credentials.

## Initial setup and subsequent verification (2026-11-12)

- Installed Rust stable **1.100.0**, rustfmt and clippy via rustup, without editing PATH or shell profiles.
- Installed GitHub CLI **2.104.0** through Homebrew. The user subsequently authenticated as `fadunco`.
- V2 CLI `debug config` successfully discovered and parsed the project configuration, including expanded rustfmt path and permission rules.
- The running OpenCode service reports **2.2.7**. All four installed skills appeared in this session's skill catalog.
- `sh -n` passed for the three setup/diagnostic scripts. Both installed upstream skill directories matched the reviewed checkout exactly (`diff -rq`).
- Doctor now passes with the bundled V2 CLI: Rust tools, GitHub sign-in, Xcode **27.2**, actual Metal compiler execution, and skill files. Default PATH still selects the V1 CLI; the bundled V2 CLI works explicitly.
- A separate temporary project with **GPUI Kit 0.9.1**, its default component/assets features, and a generated Cargo.lock passed `cargo build --locked`, rustfmt and clippy with warnings denied. It produced an ARM64 executable. This did not create a Cargo workspace in URLchemy.
- The user approved launching that temporary native window, confirmed the rendered text/button, and closed it using the button. The process exited without an error. This verifies a minimal native surface and click/quit path, not global shortcuts or launcher behavior.
- Git was subsequently initialized and the initial harness commit published to public `fadunco/urlchemy` with explicit approval. Later preparation edits remain subject to separate commit/push approval.
- The doctor regression test first reproduced the Metal false-positive, then passed all three stubbed scenarios after the execution check was fixed.
