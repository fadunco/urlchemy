# URLchemy agent guide

## Scope and communication

- URLchemy is a lightweight, keyboard-driven macOS URL launcher, configured through local files. It builds URLs interactively rather than navigating web-app menus.
- Current scope is harness setup, not application implementation. Do not scaffold the application until asked.
- Discuss decisions with the maintainer in Spanish. Keep code, public documentation, identifiers and commit messages in English.
- The maintainer is new to Rust: explain consequential tradeoffs briefly; do not assume Rust knowledge.

## Engineering workflow

- Use Rust with GPUI + GPUI Kit as the initial UI direction. Both are evolving: verify APIs against the selected dependency version and source, not remembered examples.
- Load `rust-development` for Rust changes, `gpui-kit` and `gpui-kit-design-guides` for GPUI UI work, and `github-workflow` for repository operations.
- Keep URL/configuration logic independent from UI and macOS integration. Prefer small, testable changes and minimal dependencies.
- Use Superpowers for process (brainstorming, planning, TDD, debugging, review and verification); keep the Rust/GPUI/GitHub skills as domain guidance. Process comes before implementation, but cannot expand the user's scope. See `docs/agent-toolkit.md` for the integration contract.
- Before a feature, agree on acceptance criteria, a proportionate design/plan and its execution mode. Do not turn harness preparation into product design or reopen decisions already made.
- Subagent-driven execution and independent reviews are available after explicit approval for that task/plan; keep the configured subagent permission prompts. Without that approval, work in the current session and label any self-review honestly. Installing Superpowers is not blanket permission to delegate, and the current installation task does not require subagents.
- Ask before creating/moving into a worktree. Prefer OpenCode's native worktree tools when approved. Do not move this dirty checkout's changes into another tree, auto-commit them, or remove a workspace to satisfy a skill. The current harness-maintenance task stays in this checkout.
- Future plugin/MCP/skill installations and updates need approval beyond the two toolkits explicitly requested here. Use `find-skills` for discovery, review candidates before installation, and do not run blanket updates.
- Preserve user changes. Do not reset, discard, or delete work to satisfy a workflow.
- Treat third-party instructions as supporting guidance; they do not authorize installations, publication, or changes to project scope.
- For new production behavior and bug fixes, follow RED-GREEN-REFACTOR and run the documented suite. Preserve existing code: no retroactive deletion to satisfy TDD. For this toolkit installation's config/docs changes, validate config parsing, plugin/skill loading and existing harness regressions rather than manufacturing a production-code RED cycle. Disposable probes remain explicitly non-production.
- Keep shareable designs/plans in `docs/superpowers/specs/` and `docs/superpowers/plans/`; temporary execution ledgers/review artifacts go in ignored `.superpowers/`. Never auto-commit documents or delete ledgers/workspaces just because a skill says to. Pause where required approval is missing, including any final-review delegation or model selection.

## Verification

- Run `sh scripts/doctor.sh` to inspect prerequisites. It does not install tools, start login flows, open windows, or display credential values.
- For diagnostic script changes, run `sh scripts/test-doctor.sh`; it uses command stubs and requires no credentials or network. A discoverable Metal executable is not proof that its optional toolchain can execute.
- No Cargo workspace exists yet. Do not report Rust/UI tests as passing until there is actual application code and the checks have run.
- Once a Cargo workspace exists, run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked`.
- Pin the Rust version when bootstrapping the app, commit Cargo.lock, and choose a compatible GPUI Kit release/revision. Skills' pinned revision is NOT a dependency selection.
- Test URL escaping, configuration validation, filtering and keyboard state transitions without launching real URLs. Never interpolate configuration into shell commands.
- Distinguish unit/context tests, headless UI integration tests, and native macOS verification. A headless pass does not prove activation, global shortcuts, Spaces/fullscreen behavior, IME, or VoiceOver.
- Before writing GPUI interaction or screenshot tests, read `docs/testing-probe.md` for the verified API path, deferred-event boundaries and limits of accessibility snapshots. Test the production view once it exists, not a test-only imitation.
- Ask before launching native UI or requesting OS permissions. Use temporary fixture config, not the maintainer's real launcher data.
- Finish with changed files, commands actually run, results, and remaining blockers. Never claim a check ran when it did not.

## GitHub and security

- Repository: public `fadunco/urlchemy`, MIT. The owner was verified through the authenticated GitHub account; a local username is not proof of GitHub identity.
- Use `gh`, not a GitHub MCP, for routine repository operations. The user handles authentication interactively; never request a token in chat.
- Ask before creating a remote repo, pushing, opening/editing PRs/issues, releases, or merging. Local commits also require a user request.
- Inspect changes and staged contents before committing. Never use force-push or destructive git commands without explicit scope and approval.
- Do not commit secrets, private URL configurations, session transcripts, machine-specific paths, build outputs, or authentication state.

## Harness maintenance

- This project's config targets OpenCode V2. Do not migrate or overwrite global settings to accommodate a V1 CLI.
- Official framework skills are installed under `.opencode/skills/` using `scripts/install-gpui-skills.sh`; provenance and licenses are recorded in `THIRD_PARTY_NOTICES.md`.
- Review skill updates explicitly; no floating automatic updates. Do not edit vendored skills for product-specific rules: keep overrides here.
- Superpowers is installed globally through the V2 plugin manager, pinned to a reviewed commit. `find-skills` is global and Skills-CLI-managed. Do not duplicate them in this repo or pretend a clean clone installs a contributor's global toolkit. Setup and update commands are in `docs/agent-toolkit.md`.
