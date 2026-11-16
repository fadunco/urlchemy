# Shortcut Configuration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` for inline execution or `subagent-driven-development` after explicit delegation approval. Steps use checkboxes for tracking.

**Goal:** Load and validate the approved shortcut settings without adding GUI or native dependencies.

**Architecture:** Keep source-marked strings in the existing raw configuration layer.
Compile them into immutable shortcut values stored with the validated catalog.
Sessions continue to consume semantic actions and never interpret physical keys.

**Tech Stack:** Rust `1.100.0`, existing YAML parser, standard library, existing test dependencies. No new dependencies.

**Spec:** [Framework and shortcut amendment](../specs/2026-11-15-shortcuts-framework-design.md),
[YAML contract](../specs/2026-11-14-yaml-url-contract-design.md), and
[interaction contract](../specs/2026-11-15-interaction-contract-design.md).

## Global Constraints

- Keep the UI-independent core free of GPUI dependencies.
- The defaults are `ctrl-alt-space` for invocation and `enter` for confirmation.
- The launch binding replaces Enter's palette confirmation role rather than adding a second binding.
- Modifier order has no semantic meaning.
- Do not trim, case-fold, or silently repair configuration text.
- Do not publish partially validated application settings or an executable catalog.
- Preserve all existing fixture files and expected URL outcomes.
- Keep Rust `1.100.0`, dependency pins, and `Cargo.lock` unchanged.
- Use TDD for production behavior. Preserve existing code and user changes.
- Use English for code and repository prose, and Spanish for maintainer discussions.
- This plan does not authorize commits, publication, worktree creation, or delegation.
- Execution mode and the plan require approval before implementation.

## Review Focus

1. Modifier permutations must not bypass duplicate-binding or reserved-command checks. Cover in Tasks 1–2.
2. A partial mapping must validate its explicit binding against the other binding's default. Cover in Task 2.
3. YAML wrong types and duplicate keys must retain existing source-aware diagnostics. Cover in Task 2.
4. Invalid shortcuts must prevent a valid navigation tree from escaping as a usable catalog. Cover in Task 2.
5. Existing sessions and fixture URLs must remain unchanged with either default or custom shortcuts. Cover in Task 3.

---

## Boundary and file map

This is the first independently testable increment before the native vertical slice.
The maintainer selected GPUI Kit `=0.9.1`, but this plan does not add it to Cargo.

| File | Responsibility |
| --- | --- |
| `crates/urlchemy-core/src/shortcut.rs` (new) | Immutable values, bounded parser, and conflict rules |
| `crates/urlchemy-core/src/lib.rs` | Re-export validated public value types |
| `crates/urlchemy-core/src/config/raw.rs` | Optional source-marked shortcut fields |
| `crates/urlchemy-core/src/config/schema.rs` | Strict mapping and scalar conversion |
| `crates/urlchemy-core/src/config/compile.rs` | Defaults, pair validation, and diagnostic origins |
| `crates/urlchemy-core/src/catalog.rs` | Store and expose validated shortcut settings |
| `crates/urlchemy-core/src/diagnostic.rs` | Specific shortcut diagnostic messages |
| `crates/urlchemy-core/tests/shortcuts.rs` (new) | Public loader, privacy, and traversal regression tests |
| Existing YAML and interaction specifications | Link the approved amendment without rewriting historical decisions |

Keep `load_bytes`, `load_file`, and session signatures unchanged.
`Catalog::shortcuts()` exposes immutable metadata for the future application adapter.
This avoids a loader migration or a second configuration parse for two settings.
It does not make sessions responsible for keyboard behavior.

## Task 1: Validated shortcut values

**Files:** Create `src/shortcut.rs` and modify `src/lib.rs` within `crates/urlchemy-core/`.
Tests live in the new module.

**Interfaces:** Produce these public types with private struct fields:

```rust
// All types derive Debug, Clone, PartialEq, Eq. Small value types also derive Copy.
pub enum ShortcutKey { Character(char), Enter, Space, Function(u8) }
pub struct ShortcutModifiers { /* private booleans */ }
pub struct Shortcut { /* private modifiers, key, canonical binding */ }
pub struct Shortcuts { /* private invoke and launch */ }

impl ShortcutModifiers {
    pub fn is_control(&self) -> bool;
    pub fn is_alt(&self) -> bool;
    pub fn is_shift(&self) -> bool;
    pub fn is_command(&self) -> bool;
}
impl Shortcut {
    pub fn modifiers(&self) -> ShortcutModifiers;
    pub fn key(&self) -> ShortcutKey;
    pub fn binding(&self) -> &str;
}
impl Shortcuts {
    pub fn invoke(&self) -> &Shortcut;
    pub fn launch(&self) -> &Shortcut;
}
// Shortcuts also implements Default with the approved defaults.
```

Internal interfaces in `shortcut.rs`:

```rust
pub(crate) enum ShortcutRole { Invoke, Launch }
pub(crate) enum ShortcutError { Syntax, Conflict }
impl Shortcut {
    pub(crate) fn parse(text: &str, role: ShortcutRole) -> Result<Self, ShortcutError>;
}
impl Shortcuts {
    pub(crate) fn new(invoke: Shortcut, launch: Shortcut) -> Result<Self, ShortcutError>;
}
```

Canonical output uses modifier order `ctrl-alt-shift-cmd`, followed by the key.
Canonicalization changes the value's representation, not the configuration file.
Accept any input modifier order, but reject repeated modifiers and noncanonical key spellings.
Check syntax before individual reserved-command conflicts. Pair construction rejects equal combinations.

- [x] Write unit tests `defaults_and_canonical_bindings`, `bounded_shortcut_syntax`, and `reserved_shortcut_combinations`.

Use table-driven assertions with these exact cases:

```rust
assert_eq!(Shortcuts::default().invoke().binding(), "ctrl-alt-space");
assert_eq!(Shortcuts::default().launch().binding(), "enter");
assert_eq!(
    Shortcut::parse("alt-ctrl-space", ShortcutRole::Invoke).unwrap().binding(),
    "ctrl-alt-space"
);
assert!(matches!(
    Shortcut::parse("cmd-enter", ShortcutRole::Launch).unwrap().key(),
    ShortcutKey::Enter
));
```

For syntax, reject `""`, `" enter"`, `"enter "`, `"CMD-enter"`, `"ctrl-ctrl-k"`,
`"ctrl-k ctrl-l"`, `"cmd"`, `"cmd-"`, `"fn-enter"`, `"super-enter"`, `"cmd-k->x"`,
`"ctrl-é"`, `"ctrl-f0"`, `"ctrl-f13"`, `"ctrl-f01"`, `"cmd-escape"`, and `"cmd-["`.
Reject bare `space`, `a`, `f1`, and `shift-enter` in both roles.
Accept bare `enter` only for Launch.
Accept each allowed key with `ctrl` in both roles.
Assert modifier readers for `ctrl-alt-shift-cmd-k`.
For both roles, reject every protected combination and every `cmd-1` through `cmd-9`.
Include reordered `alt-cmd-h` and `shift-cmd-z` as conflicts.
Accept `cmd-0`, `cmd-shift-1`, `ctrl-f12`, and `cmd-enter` as distinct valid combinations.
Assert equal reordered combinations fail `Shortcuts::new`.

- [x] Run `$HOME/.cargo/bin/cargo test -p urlchemy-core --locked shortcut::tests` and record the expected RED result for the missing API.
- [x] Implement the types, parser, defaults, and conflict checks using only the standard library.
- [x] Run the same command and require all new tests to pass.
- [x] Self-review exact grammar coverage, public field privacy, and absence of logging or native effects.

## Task 2: Atomic YAML integration and diagnostics

**Files:** Modify `config/raw.rs`, `config/schema.rs`, `config/compile.rs`,
`catalog.rs`, and `diagnostic.rs`. Create `tests/shortcuts.rs`.

**Consumes:** Task 1's parser, defaults, and pair constructor.
**Produces:** `pub fn Catalog::shortcuts(&self) -> &Shortcuts`.
Existing `config::load_bytes(source: &Path, bytes: &[u8]) -> Result<Catalog, Diagnostics>` remains the entry point.

Add `RawConfig.shortcuts: Option<RawShortcuts>` and
`RawShortcuts { invoke: Option<Field<String>>, launch: Option<Field<String>> }`.
The schema accepts only `navigation` and `shortcuts` at root.
Within the optional mapping, accept only `invoke` and `launch` as optional strings.
Retain `Field` origins for explicit values.

Store `pub(crate) shortcuts: Shortcuts` in `Catalog`.
Compile settings alongside navigation and return no catalog when either fails.
Do not return early on shortcut compilation errors if navigation compilation can provide more diagnostics.
Keep the existing diagnostic sorting and 50-item limit.

- [x] Write `shortcut_defaults_and_partial_mappings` against the public loader with these cases:

```text
navigation: []                                      -> ctrl-alt-space / enter
shortcuts: {}\nnavigation: []                       -> ctrl-alt-space / enter
shortcuts: {invoke: ctrl-k}\nnavigation: []          -> ctrl-k / enter
shortcuts: {launch: cmd-enter}\nnavigation: []       -> ctrl-alt-space / cmd-enter
```

Assert the exact `binding()` values from `catalog.shortcuts()`.

- [x] Write `strict_shortcut_schema_and_locations` with a diagnostic code and pointer for each case:

```text
shortcuts: null                         E_SCHEMA_TYPE          /shortcuts
shortcuts: []                           E_SCHEMA_TYPE          /shortcuts
shortcuts: {launch: null}               E_SCHEMA_TYPE          /shortcuts/launch
shortcuts: {invoke: 12}                 E_SCHEMA_TYPE          /shortcuts/invoke
shortcuts: {launch: [enter]}            E_SCHEMA_TYPE          /shortcuts/launch
shortcuts: {confirm: enter}             E_SCHEMA_UNKNOWN_FIELD /shortcuts/confirm
shortcuts: {launch: enter, launch: x}   E_YAML_DUPLICATE_KEY    /shortcuts/launch
```

Add `navigation: []` to each document. Assert a source location exists.
Also reject duplicate root `shortcuts`, scalar `shortcuts: x`, and a non-string member key with existing YAML/schema diagnostics.
Assert `shortcuts: {}` without `navigation` still reports `E_SCHEMA_REQUIRED` at `/navigation`.

- [x] Write `shortcut_validation_is_atomic_and_source_aware` for syntax and conflict errors.

For `"shortcuts:\n  launch: cmd-a\nnavigation: []\n"`, assert `E_SHORTCUT_CONFLICT`,
pointer `/shortcuts/launch`, line 2, and column 11.
For `launch: alt-ctrl-space` with omitted invoke, assert a conflict at `/shortcuts/launch`.
For two explicit reordered identical bindings, attach the conflict to launch and relate the explicit invocation location.
For an individually forbidden invocation, attach the conflict to `/shortcuts/invoke`.
For an invalid invocation and valid flow, assert the whole load returns `Err`.
For invalid settings and invalid navigation, assert both diagnostic classes occur.
Copy Task 1's malformed-string table through the YAML loader using quoted strings.
Assert `E_SHORTCUT_SYNTAX` rather than a panic or a repaired binding.

- [x] Run `$HOME/.cargo/bin/cargo test -p urlchemy-core --locked --test shortcuts` and record RED evidence.
- [x] Implement strict conversion, defaults, semantic compilation, catalog storage, and the reader.

For default resolution, replace only omitted members. Do not replace invalid explicit values.
Only compare the pair after both individual bindings pass validation.
Use `Diagnostic::related` for the explicit invocation when a pair conflicts.
Add these exact message mappings:

```text
E_SHORTCUT_SYNTAX: Define one supported key combination with the required modifiers.
E_SHORTCUT_CONFLICT: Choose a shortcut that does not conflict with another binding or a protected command.
```

- [x] Run the shortcut integration target and module tests. Require all tests to pass.
- [x] Self-review diagnostic privacy: neither messages nor Debug diagnostics include rejected shortcut strings or captured URL values.

## Task 3: Existing-journey regression coverage and documentation

**Files:** Extend `tests/shortcuts.rs`. Use existing `tests/support/mod.rs` helpers
without modifying approved YAML fixtures or their expectations manifest.
Add amendment links to the YAML and interaction specifications.

**Consumes:** Task 2's validated catalog and unchanged session API.
**Produces:** Regression evidence and linked configuration documentation.

- [x] Add `custom_shortcuts_preserve_existing_journeys`.

For each successful journey in `yaml-url-contract/expectations.json`, prepend
`shortcuts: {invoke: ctrl-k, launch: cmd-enter}` to its existing fixture bytes.
Load the result through `load_bytes`, use the existing `support::drive`, and assert
the exact expected URL plus `SessionSnapshot::Completed`.
This proves semantic session actions do not depend on a physical Enter binding.
Assert the original fixture bytes load with default shortcuts too.
Add a privacy assertion that formatted loader diagnostics omit an invalid supplied string.

- [x] Run `$HOME/.cargo/bin/cargo test -p urlchemy-core --locked --test shortcuts`.
This regression can already pass after Task 2. Record it as coverage, not missing RED evidence for a new behavior.
- [x] Link the approved amendment from the original root-schema and interaction prerequisite sections.
State that the maintainer selected GPUI Kit but did not install it. OS registration remains outside this increment.
- [x] Run `sh scripts/doctor.sh` without starting login flows or installing tools.
- [x] Run `$HOME/.cargo/bin/cargo fmt --all -- --check`.
- [x] Run `$HOME/.cargo/bin/cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [x] Run `$HOME/.cargo/bin/cargo test --workspace --locked`.
- [x] Run `sh scripts/test-doctor.sh` for the documented harness regressions.
- [x] Check changed English prose with the installed `asd-ste100` linter.
- [x] Inspect the final diff and confirm no dependency, toolchain, lockfile, native, or unrelated change.
- [x] Report changed files, actual commands and results, and the remaining native work. Do not commit without approval.

## Self-review and handoff

Self-review completed for this plan: types and readers agree across tasks, and
the five review risks map to explicit tests. No subagent review occurred.

Coverage deliberately stops at validated configuration metadata.
The native adapter, dynamic GPUI binding installation, configured footer hints,
global registration failures, and diagnostic access without a working shortcut remain unimplemented.
The next native plan must select the registration API and fallback access path.
It must verify opener failure reporting and use the production view with the real core.
This plan makes no claim of native shortcut or accessibility verification.

Recommended execution: inline in the current session, with honest self-review.
The two production tasks share a small API and add no dependencies.
Delegation and independent final review remain optional and require explicit approval.
Keep current documentation work intact. Ask before changing branches or creating a worktree.

## Execution record — 2026-11-16

The maintainer approved inline execution. Work remained in the existing checkout.
No commit, worktree change, dependency installation, or delegated review occurred.

- Task 1: the initial RED run failed for missing shortcut types. The three parser tests then passed.
- Task 2: the initial RED run failed for the missing catalog reader. With that reader, three behavioral tests failed on the unsupported YAML field.
  After integration, all three loader tests passed without warnings.
- Task 3: the two additional regression tests passed immediately after Task 2.
  They add coverage, not a new production behavior or a claimed RED cycle.
- Full workspace suite: 36 tests passed, with no failures.
- Format check and Clippy with warnings denied passed.
- Environment diagnostics passed, including Metal execution. All three diagnostic-script scenarios passed.

The author reviewed the tracked diff and both new Rust files against the plan.
This was self-review, not independent review. Native registration, GPUI binding
installation, and native UI verification remain outside this increment.
