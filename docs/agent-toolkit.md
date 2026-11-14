# Agent toolkit: Skills CLI and Superpowers

## Installed scope and ownership

The maintainer explicitly requested these additions after the initial minimal
harness setup. They are installed **globally for this user's OpenCode**, not as
application dependencies or project-vendored skills. Cloning URLchemy does not
install or execute them on a contributor's machine.

| Component | Reviewed version/revision | Managed by |
| --- | --- | --- |
| Node.js | 26.13.1, installed through Homebrew | Homebrew; runtime for npm/npx |
| Vercel Skills CLI | `skills@1.8.1` | npm global installation |
| `find-skills` | `vercel-labs/skills` at `b716f9dc2ba36b6aa8f0080615e5a3a6caeac63b` | Skills CLI, global lock |
| `asd-ste100` | version 0.4.0 at `d5caf54905eac776e9bb3a247be84153701ff4f8` | Skills CLI, global lock |
| Superpowers | package version 7.0.2 at `22f0b463e7a5e579904dd6211a520321035526f7` | OpenCode V2 plugin manager |
| GPUI Kit skills | Revision in `THIRD_PARTY_NOTICES.md` | Project installer and vendored copies |
| Rust/GitHub project skills | Project-owned Markdown | This repository; optional independent global copies |

`npx skills update` is an **update command**, not a skill or OpenCode plugin.
`find-skills` is a skill that teaches discovery and installation through that CLI.
Superpowers is an executable OpenCode plugin plus its own skill library. Do not
also install Superpowers through `skills add`: that duplicates registration and
creates competing update mechanisms.

Both upstream projects use MIT; no upstream material from these two projects is
vendored here. The existing GPUI Kit licenses/notices remain unchanged.

## What Superpowers adds

The reviewed OpenCode adapter has no npm runtime dependencies. It registers its
15 skills and injects the `using-superpowers` bootstrap into transient model
context for controller sessions. In V2 it hooks context, compaction and generate
requests and suppresses controller bootstrap for delegated child sessions.
It does not itself create commits, launch subagents or run tests: its instructions
direct the agent to do those things, subject to project rules and tool permissions.

Its process covers brainstorming/design, plans, TDD, systematic debugging,
verification, reviews, worktrees and branch completion. Its optional browser
visual companion is for mockups, not native macOS UI verification. The plugin
executes inside OpenCode; this source review is not a complete security audit of
every bundled helper script.

## Integration contract and conflicts resolved

Project instructions explicitly specialize upstream workflows; the installed
upstream plugin and its skills are left unchanged.

| Existing rule or upstream default | Resolution for URLchemy |
| --- | --- |
| Initial decision not to install plugins | Superseded only for this explicitly requested toolkit. No MCP was added. |
| General workflow versus GPUI-specific rules | Superpowers sets process; Rust, GPUI Kit/design and GitHub skills supply technical constraints. Do not replace native tests with browser/web assumptions. |
| Subagent-driven execution and final-review dispatch | Obtain explicit approval for the task/plan's execution mode and delegation. This includes the final reviewer in the inline workflow. Without approval, work inline and label self-review; do not claim independent review. Keep `subagent: ask` permissions. |
| Worktree creation and setup | Ask first, prefer OpenCode-native worktree tools, verify baseline and respect dirty work. Do not migrate or auto-commit the existing checkout's uncommitted changes. |
| Frequent commits, committed specs and commit steps in plans | These are checkpoints, not authorization. Ask before local commits; approval to implement a plan alone does not authorize them. Push/PR/merge remain separately gated. |
| TDD instruction to delete code written before tests | Preserve existing work. Follow test-first for new production behavior; never delete/reset user work or rewrite existing code merely to satisfy a workflow. Report missing RED evidence honestly. |
| Configuration/docs installation work and disposable probes | Validate parser/loading, command behavior and existing regressions. Do not manufacture production-code RED cycles or treat probes as application features. |
| Specs/plans and execution scratch | Shareable docs belong in `docs/superpowers/specs/` and `plans/`; `.superpowers/` and `.worktrees/` are ignored. No automatic cleanup/deletion of workspaces or ledgers. |
| Tool/model recommendations | Use actual V2 tools. Do not invent a todo tool or model ID. Do not switch models or use a higher-cost reviewer without the task's approval. |
| `find-skills` popularity criteria and install examples | Install counts/stars are discovery signals, not a security guarantee. Review source, scripts, license and version compatibility; ask before additional installations or updates. |
| `/check`, `/review`, `/prepare-pr` | Keep their narrow scopes: diagnostics, in-session read-only review, and draft-only PR preparation. They do not trigger automatic implementation, delegation or publication. |

The global `~/.config/opencode/AGENTS.md` records the general approval,
preservation and skill-management preferences. Project AGENTS.md keeps the
URLchemy-specific contract. Global instructions are not an OS sandbox or a
replacement for permission rules.

## Reproducing the global installation

Run only after opting into these user-wide changes. Node must satisfy the Skills
CLI's `>=22.20.0` requirement. On this Mac, Node/npm/npx were not previously
installed, so Node was installed with Homebrew. No shell profiles were modified.

```sh
brew install node
npm install --global skills@1.8.1 --ignore-scripts --no-audit --no-fund
DISABLE_TELEMETRY=1 npx --no-install skills add \
  vercel-labs/skills@b716f9dc2ba36b6aa8f0080615e5a3a6caeac63b \
  --skill find-skills --global --agent opencode --yes
/Applications/OpenCode.app/Contents/Resources/opencode-cli plugin add \
  'superpowers@git+https://github.com/obra/superpowers.git#22f0b463e7a5e579904dd6211a520321035526f7'
```

Use an equivalent **V2** binary on other machines. The default PATH `opencode`
on the maintainer's machine remains V1; it was not upgraded or replaced. The
global config now contains a `plugins` array with the exact Git revision. No
providers, models, credentials, V1 SDK dependencies or project permissions changed.

The Skills CLI installed `find-skills` in `~/.agents/skills/find-skills/`, which
OpenCode discovers natively, with its entry in `~/.agents/.skill-lock.json`.
The installed skill matched the reviewed source byte-for-byte. It is not copied
into `.opencode/skills/`, and our own global skills remain untracked by this CLI.

## Daily usage and updates

```sh
DISABLE_TELEMETRY=1 npx --no-install skills find gpui --owner longbridge
DISABLE_TELEMETRY=1 npx --no-install skills list --global --agent opencode
DISABLE_TELEMETRY=1 npx --no-install skills update find-skills --global --yes
/Applications/OpenCode.app/Contents/Resources/opencode-cli plugin list
```

The scoped update command was tested and reported no updates. Because find-skills
was installed from a full commit ref, it stays on that reviewed source; accepting
a newer upstream version means reviewing a new commit and explicitly reinstalling
from that ref. An unscoped `npx skills update` can select a broader scope and is
not routine project setup. In the tested CLI, an unqualified `skills check`
checked project scope even when global find-skills was installed; it is not
evidence that global skills are current. Use the explicitly scoped command above.

Superpowers updates are separate. Review a new revision, then replace the single
configured plugin revision with the approved one through the V2 plugin manager
or configuration; do not append duplicate installations. Exact Git revisions are
skipped by automatic plugin updates. The Vercel CLI does not manage this plugin
or the project's manually vendored GPUI skills.

The Skills CLI collects telemetry by default; our agent-issued commands use
`DISABLE_TELEMETRY=1`, which upstream documents as disabling both usage telemetry
and audit requests. This is per invocation, not a machine-wide environment change.
Popularity checks do not replace source review. If the optional Superpowers visual
companion is approved later, launch it with `SUPERPOWERS_DISABLE_TELEMETRY=1` to
disable its documented remote logo/version request; it was not launched here.

V2 watches configuration and reloads plugins/skills. No restart was needed for
this installation; use `/reload` if a client needs an explicit refresh. Do not
follow older upstream restart/cache-deletion advice unless diagnosing a real
failure. Do not delete the OpenCode cache, state or credentials to update a plugin.

## Optional writing guidance

The maintainer also requested the MIT-licensed `danyuchn/asd-ste100-skill` as a global skill.
Its reviewed revision is `d5caf54905eac776e9bb3a247be84153701ff4f8`.
Install it only after approving this user-wide change:

```sh
DISABLE_TELEMETRY=1 npx --no-install skills add \
  danyuchn/asd-ste100-skill@d5caf54905eac776e9bb3a247be84153701ff4f8 \
  --skill asd-ste100 --global --agent opencode --yes
```

The installed files matched the reviewed source. OpenCode loaded the skill through its native tool.
The skill does not include the full official dictionary or certify ASD-STE100 compliance.
Review updates before installing a different revision.

The maintainer's global `AGENTS.md` defines these communication preferences:

1. Use plain language.
2. State the conclusion first.
3. Use numbered steps for procedures.
4. Preserve precision and uncertainty.
5. Omit empty introductions, repetition and closing filler.

English repository prose follows the skill more rigorously than conversational chat.
Procedures use Strict mode. Explanatory documentation uses STE-flavored mode.
Spanish chat follows the clarity principles flexibly, without English vocabulary restrictions or rigid sentence limits.
These are global preferences, not files that a clone installs automatically.

## Verification performed

- OpenCode V2 `plugin list` identifies Superpowers at the exact reviewed commit;
  the server plugin API reports it `active`.
- All 15 Superpowers skills appeared in the current session's native catalog;
  `verification-before-completion` loaded from the installed package.
- The upstream bootstrap-caching test passed against the reviewed adapter,
  including V2 tool mapping and hook injection with its mock host. This does not
  claim an end-to-end LLM workflow evaluation.
- Skills CLI 1.8.1 runs through `npx --no-install`; its global list reports
  find-skills as tracked for OpenCode, and the native skill tool loaded it.
- A read-only CLI search for GPUI under `longbridge` returned the official Kit
  skills. The scoped global find-skills update command also completed successfully.
- The project/global merged configuration parses with the V2 CLI and preserves
  the project's shell and subagent approval rules.

## Sources

- https://github.com/vercel-labs/skills
- https://skills.sh/vercel-labs/skills/find-skills
- https://github.com/obra/superpowers
- https://github.com/danyuchn/asd-ste100-skill/tree/d5caf54905eac776e9bb3a247be84153701ff4f8
- https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/.opencode/plugins/superpowers.js
- https://opencode.ai/docs/plugins
- https://opencode.ai/docs/skills
- https://opencode.ai/docs/instructions
