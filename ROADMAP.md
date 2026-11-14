# URLchemy Roadmap

This roadmap proposes the sequence from the agreed [MVP product definition](docs/product.md)
to a usable macOS application. It is not an executable implementation plan or
authorization to scaffold, install dependencies, commit, publish, or delegate.

**Current position:** MVP functional scope and acceptance criteria agreed; an
illustrative YAML representation exists. No Cargo workspace or production
application exists yet. The next milestone is domain and execution design.

## How to use this roadmap

- Keep product behavior in `docs/product.md`, technical designs in
  `docs/superpowers/specs/`, and actionable implementation plans in
  `docs/superpowers/plans/`.
- Agree on each milestone's design, acceptance, and execution mode before
  implementing it. Plan the next bounded increment, not the entire application
  in speculative detail.
- Existing product decisions are inputs, not questions to reopen by default.
  Record any necessary product changes explicitly with the maintainer.
- Milestones are delivery checkpoints, not a requirement for one crate, module,
  commit, or session per milestone. Concrete Rust boundaries follow the design.

## 1. Define the domain and execution model

**Goal:** explain the engine independently of YAML examples and GPUI.

Deliver a technical design covering:

- Configuration data as read from files, the validated executable definition,
  and the mutable state of one launcher session. Explain whether separate Rust
  representations are useful rather than assuming identical YAML/runtime types.
- Proposed Rust structs/enums for navigation entries, flows, steps, choices,
  continuations, captured values, and destinations; ownership and identifiers
  explained in plain language.
- Invariants: finite trees, fixed base per flow, required inputs, named captures,
  valid continuations, and leaf references available on each traversal.
- Session actions and transitions: enter a folder or flow, confirm an answer,
  choose a branch, go back, invalidate later answers, dismiss, and resolve a URL.
- The boundary between engine results and external effects. The engine returns
  state, diagnostics, or a resolved URL; it does not open browsers or depend on
  GPUI or macOS APIs.

**Exit criterion:** the maintainer can trace a branching example from its
definition in memory through session transitions to a resolved destination, and
approves the model without needing to infer it from YAML nesting.

## 2. Specify the YAML contract and URL semantics

**Goal:** connect the domain to an unambiguous, editable configuration format.

- Build on the accepted YAML example; do not redesign the domain from scratch.
- Define serialization and validation for the agreed model, including selections
  with a common continuation and selections with branch-specific continuations.
- Provide complete fixtures for a fixed destination, a linear flow, and a
  branching flow, plus representative invalid configurations.
- Specify base/path joining, preserving base path prefixes; context-specific
  encoding; empty or invalid input; and malformed template handling.
- Define the single file's location, missing-file behavior, and initial setup.
  Do not silently introduce config generation or an editor.
- Specify useful diagnostics and their file locations. Keep automatic alias
  conflict validation deferred; document alias restrictions.

**Exit criterion:** each fixture maps to a defined model or diagnostic, and each
valid traversal has an exact expected URL. The schema and examples are approved.

## 3. Implement and test the UI-independent core

**Goal:** prove configuration and execution behavior without native UI.

- Prepare a bounded implementation plan from milestones 1–2.
- Bootstrap the minimal Cargo structure when explicitly authorized. Pin Rust,
  retain `Cargo.lock`, and justify dependencies; do not add GPUI to the core.
- Implement loading, validation, navigation/session transitions, and URL
  resolution with test-first increments.
- Test fixed, linear, common-continuation, and branching traversals; labels versus
  values; out-of-URL-order capture; corrections; cancellation/reset; invalid
  configuration; and reserved/Unicode characters in URL values.
- Test UI-independent filtering and navigation actions at the appropriate core
  boundary. Use test harnesses rather than building a second launcher product.

**Exit criterion:** the fixtures execute deterministically in tests, the agreed
invalid configurations fail with diagnostics, and the core has no GUI/native
dependency or real browser-opening effects.

## 4. Define the interaction contract and low-fidelity UX

**Goal:** make the proven model understandable and fast to operate.

- Load the project's GPUI and design guidance before making UI decisions.
- Sketch root and nested navigation, free text, suggestions, closed selections,
  errors, and back navigation using the reference fixtures.
- Resolve focus, back/dismiss bindings, filtering, ordinal shortcuts, exact alias
  activation, and the transition from navigation typing to parameter entry.
- Preserve immediate completion, no final confirmation screen, automatic closing,
  and a fresh root on every invocation.
- Keep visual exploration lightweight. No browser companion, native UI launch,
  or OS permission request without approval.

**Exit criterion:** representative keyboard and mixed-input journeys are agreed,
including correction and cancellation, without requiring polished visuals.

This interaction design may overlap core development once the domain and YAML
contract are approved. Feed genuine findings back into the design; do not attempt
to freeze every UX detail before hands-on use.

## 5. Build an early native vertical slice

**Goal:** validate real macOS integration before completing the interface.

- Select a compatible GPUI + GPUI Kit release/revision and verify its actual APIs.
  A skills snapshot is not an application dependency selection.
- Connect the production view to the core; do not duplicate flow logic in GPUI.
- Deliver one small end-to-end fixture: global shortcut → root → simple flow →
  input → resolved URL → default browser → overlay closes.
- Validate activation, keyboard focus, dismissal, and repeat invocation on macOS
  with permission and temporary fixture configuration.
- Keep browser opening behind a native boundary so automated tests can observe
  the requested URL without actually launching it. Handle opening failures
  explicitly rather than assuming the OS call succeeds.

**Exit criterion:** the maintainer can perform the small real journey, and native
limitations are recorded before expanding the full UI.

## 6. Complete the agreed MVP journeys

**Goal:** expose the rest of the tested core through the production interface.

- Add multilevel folders, folder/flow emojis, scoped aliases and direct access,
  ordinal navigation, suggestions, closed selections, and branches.
- Complete correction/invalidation, startup diagnostics, and all remaining
  interactions from the approved design.
- Read `docs/testing-probe.md` before implementing GPUI interaction or screenshot
  tests. Test the production view and respect deferred-event boundaries.

**Exit criterion:** the fixed, linear, and branching fixtures work through the
real interface and satisfy the product acceptance criteria, not just core tests.

## 7. Verify and document the first usable MVP

**Goal:** establish what actually works and how to use it.

- Check every acceptance criterion in `docs/product.md` against evidence.
- After the Cargo workspace exists, run:

  ```sh
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  ```

- Distinguish core tests, headless UI tests, and authorized native verification.
  Record what was and was not checked for activation, Spaces/fullscreen, IME,
  and VoiceOver; a headless pass is not evidence for those behaviors.
- Document setup, configuration location, valid examples, aliases, restart-to-
  reload behavior, and known limitations. Keep examples fictitious and public-safe.
- Review the maintainer's hands-on experience and fix failures within scope.
  Packaging/distribution decisions, commits, and publication require their own
  agreement; this milestone does not authorize a public release.

**Exit criterion:** evidence supports the agreed MVP criteria and remaining
limitations are explicit. Untested behavior is not reported as passing.

## After the MVP

Reassess based on actual use, not speculative extensibility. Possible later work
includes multiple configuration files (perhaps one per flow), reload without a
restart, optional inputs, and automatic alias-conflict validation. Cross-flow
invocation or shared definitions require a demonstrated need and a new design.
External data fetching remains excluded by the product definition.
