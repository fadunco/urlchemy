# Development harness

## Chosen baseline

- **OpenCode V2**, with the existing provider/model selection inherited. No credentials in the repository.
- **Rust + GPUI + GPUI Kit** as the initial desktop UI direction, not yet an application dependency selection.
- Official `gpui-kit` and `gpui-kit-design-guides` skills installed project-locally at the revision in `THIRD_PARTY_NOTICES.md`.
- Project-owned `rust-development` and `github-workflow` skills, also available for optional global installation.
- No MCP servers, third-party executable plugins, or automatic multi-agent orchestration.
- The initial harness setup did not scaffold the application or publish anything. The maintainer subsequently approved initializing Git and publishing the harness to the public `fadunco/urlchemy` repository under MIT.

The two general skills are versioned in this project, so contributors do not need the global copies. The global installer copies only these two skills and refuses to replace different existing files. Project skills take precedence in V2. Global OpenCode configuration and provider credentials are unchanged.

## Why these choices

GPUI is the UI framework behind Zed and has a longer real-application track record. GPUI Kit adds components and first-party-maintained agent guidance. Both remain evolving APIs: compile against a chosen compatible version, rather than relying on generic GPUI examples. The future application's Rust version and dependencies still need to be selected and pinned.

Rune is a credible alternative, particularly for agents: its current documentation provides PNG snapshots, element/accessibility trees, scripted keyboard input, global-shortcut simulation and launcher window support. It is explicitly pre-release and requires a pinned revision and matching patches. We have not built or benchmarked either framework here. Prefer GPUI initially for the maintainer's maturity criterion, not because Rune lacks agent tooling.

Sources checked during setup:

- https://github.com/zed-industries/zed/tree/main/crates/gpui
- https://github.com/longbridge/gpui-kit/tree/6653d1c024b5461461143c90b005d80089cc085f/skills
- https://github.com/anomalyco/rune
- https://github.com/anomalyco/rune/blob/dev/crates/rune/README.md
- https://opencode.ai/docs/config
- https://opencode.ai/docs/skills
- https://opencode.ai/docs/permissions
- https://opencode.ai/docs/commands

Superpowers was evaluated (https://github.com/obra/superpowers). It offers a comprehensive methodology, but introduces mandatory workflows/bootstrap injection and optional subagent-driven execution. It is intentionally not installed: a small set of explicit skills is a better starting baseline here. General frontend/web design skills and Playwright were not installed because this UI is native, not a browser DOM.

## Machine prerequisites

Run `sh scripts/doctor.sh` from the repository root. A nonzero exit identifies outstanding prerequisites; it is not a failed application test.

### OpenCode CLI mismatch

At setup, `/Applications/OpenCode.app` and its bundled CLI are **2.2.7**, while `opencode` on PATH is **1.19.5**. The global config contains only the schema declaration. The existing global package.json references the V1 plugin SDK; leave it alone unless a migration is explicitly requested. No plugin is being installed here.

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

The initial machine has Command Line Tools selected, not full Xcode. GPUI's documented macOS setup requires Xcode and Metal tooling. Install Xcode from Apple, launch it and complete the macOS component/license setup. Then select the actual installation, for example:

```sh
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
xcodebuild -version
xcrun --find metal
```

These are manual instructions, not commands the agent should run without approval. Command Line Tools alone are not proof that GPUI can compile.

### GitHub

GitHub CLI is installed through Homebrew. Authentication is user-driven:

```sh
gh auth login --web
gh auth status
gh api user --jq .login
```

Do not share tokens in chat. The authenticated owner is `fadunco`; the maintainer approved the public `fadunco/urlchemy` repository under MIT, including Git initialization, the initial harness commit and push. Future commits and remote mutations still require explicit approval. The initial commit uses a repository-local Git identity with GitHub's noreply address rather than publishing a personal email.

## Working loop

1. Agree on a small task and acceptance criteria.
2. Load relevant skills; verify the selected framework API against source.
3. Add focused tests and implement a small change.
4. Run `/check`; use `/review` for a non-editing review. Commands stay in the current session, with no automatic subagents.
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

## Updating and removal

- Review upstream changes before updating the framework skill revision and notices. The installer refuses existing skill directories rather than overwriting work.
- Global copies are independent snapshots. Updates require review if files differ; deleting or renaming project files does not silently change global behavior.
- To undo global skills, inspect and remove only `~/.config/opencode/skills/rust-development/` and `github-workflow/` if they still contain the copies installed here. Do not remove unrelated skills, SDK packages, OpenCode state or credentials.

## Setup verification (2026-11-12)

- Installed Rust stable **1.100.0**, rustfmt and clippy via rustup, without editing PATH or shell profiles.
- Installed GitHub CLI **2.104.0** through Homebrew. GitHub is not authenticated yet.
- V2 CLI `debug config` successfully discovered and parsed the project configuration, including expanded rustfmt path and permission rules.
- The running OpenCode service reports **2.2.7**. All four installed skills appeared in this session's skill catalog.
- `sh -n` passed for the three setup/diagnostic scripts. Both installed upstream skill directories matched the reviewed checkout exactly (`diff -rq`).
- Doctor verified the Rust tools and skill files. Outstanding prerequisites: GitHub sign-in and full Xcode/Metal. Default PATH still selects the V1 CLI; the bundled V2 CLI works explicitly.
- No application build, Rust tests or native UI checks ran: there is no Cargo workspace yet. No git repository, commits or remote resources were created.
