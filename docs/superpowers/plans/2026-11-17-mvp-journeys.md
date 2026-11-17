# MVP Journeys Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan inline, task by task. Steps use checkbox syntax.

**Goal:** Complete the approved non-ordinal MVP journeys and their presentation through the production palette.

**Architecture:** Retain `Session` as the answer/history owner and `Launcher` as the command/effect boundary.
Add private derived presentation policy to the palette, not a second session model.
Extend real-view tests and replace generic rendering with state-specific surfaces and named measurement regions.

**Tech Stack:** Rust `1.100.0`, GPUI Kit `=0.9.1`, and the existing locked workspace dependencies.

**Spec:** [Approved journey design](../specs/2026-11-17-mvp-journeys-design.md).
The [interaction contract](../specs/2026-11-15-interaction-contract-design.md) and
[testing probe](../../testing-probe.md) remain binding inputs.

Status: approved by the maintainer on 2026-11-17.
Merge this documentation separately before implementation on a new branch.

## Global Constraints

- Create `feat/mvp-journeys-implementation` from `main` after the design/plan PR merges.
- The product-code baseline is merge commit `ec9f682412e2434ba0e4ebc5ff29faf974dec7ea`. Record the later documentation merge as the execution base.
- Preserve the existing `ROADMAP.md` update and approved design.
- Execute inline with TDD and author self-review. Do not delegate.
- Do not create a worktree, commit, publish, or merge without new authorization.
- Keep Rust `1.100.0`, GPUI Kit `=0.9.1`, and `Cargo.lock`. Add no dependencies.
- Do not change YAML fields, URL semantics, or approved fixture files.
- Keep the core independent of GPUI and native services.
- Do not add ordinal activation or hints. That is the next separate increment.
- Preserve exact text, immutable committed-edit events, editor generations, stable row IDs, and stale-effect guards.
- Do not expose stored suggestion values through labels or copy captured answers into context.
- Use semantic Kit components, theme tokens, and rem-based layout. Add no product colors or pixel-based spacing.
- Use fixture catalogs and injected desktop services in tests. Launch no native UI or browser under this plan's automated checks.
- Previous native-launch and browser permissions do not authorize new native acceptance runs.
- Do not log drafts, resolved URLs, or private configuration. Keep execution artifacts in ignored `.superpowers/`.

## Review Focus

1. Changing a shared-continuation choice after committing later inputs must discard those later answers. Task 1 tests an extended common fixture.
2. Queued row activation and editor events must not apply to a newly restored branch. Tasks 1 and 4 cover rapid context changes.
3. Hiding the editor for errors or opening states must preserve a valid keyboard focus path. Tasks 3 and 4 test recovery by actual key events.
4. A wrapped diagnostic or longer action label must not move Close or the footer outside a small, enlarged-text viewport. Tasks 3 and 4 test geometry.
5. Full context help must exclude answers and dismiss before the palette when keyboard focus owns the tooltip. Task 4 tests this explicitly.

---

## File map and shared interfaces

| Location | Responsibility |
| --- | --- |
| `crates/urlchemy-app/src/launcher.rs` | Existing session controller and a readonly current-entry lookup |
| `crates/urlchemy-app/src/palette.rs` | Retained editor/list state, events, focus, and state-specific region composition |
| `crates/urlchemy-app/src/palette/presentation.rs` (new) | Private derived surface, feedback, commitment labels, and navigation decoration |
| `crates/urlchemy-app/src/palette/layout.rs` (new) | Private preferred-height policy for named measured regions |
| `crates/urlchemy-app/src/palette/context.rs` (new) | Context elision and keyboard/hover context help using verified public APIs |
| `crates/urlchemy-app/tests/ui.rs` | Existing headless harness, source-based setup, and original regressions |
| `crates/urlchemy-app/tests/support/journeys.rs` (new) | Additional tests compiled as a submodule of the `ui` target |
| `crates/urlchemy-app/tests/fixtures/*.yaml` (new) | Test-only nested navigation and extended review journeys |
| `crates/urlchemy-app/tests/shell.rs` | Real shell with injected pending/failing opening and window lifecycle |
| `crates/urlchemy-app/tests/support/mod.rs` | Existing deterministic desktop service double |
| `docs/native-development.md`, `ROADMAP.md` | Actual evidence, increment status, and remaining limitations |

Do not move existing modules or replace the palette implementation wholesale.
Extract the render helpers this plan names when introducing state-specific composition.
Keep helper types private to the app or palette module.
Paths beginning with `src/` or `tests/` below are relative to `crates/urlchemy-app/`.

### Test harness seam

Keep the existing `Harness`, `setup`, `settle`, `ui`, `key`, `text`, `value`, `click_label`, and `opened` helpers.
Add `setup_source(cx: &mut TestAppContext, source: &str) -> Harness`.
Make existing `setup` load a fixture and delegate construction to that helper.
Keep the real `PaletteView::new(Launcher::new(catalog, 1), window, cx)` constructor.
Subscribe only to `PaletteEvent::Effect` for effect assertions.

In `tests/ui.rs`, include `#[path = "support/journeys.rs"] mod journeys;`.
The child module uses the parent's private helpers. Do not add test-only methods to production types.
Use accessible labels for rows and stable IDs for controls.
Inspect fresh frames after deferred work. Never infer enabled or selected behavior solely from missing accessibility flags.

### Test-first policy for existing behavior

Several journeys may already pass. Record these as characterization coverage.
Do not delete or temporarily damage working code to manufacture RED evidence.
When a new assertion exposes a bug, keep its observed failure and make the smallest production fix.
New presentation or interaction behavior must have an observed failing test before implementation.

## Task 1: Complete journey and correction coverage

**Files:** `tests/ui.rs`, `tests/support/journeys.rs`, `tests/fixtures/nested_navigation.yaml`,
`tests/fixtures/four_inputs.yaml`, `tests/fixtures/common_review.yaml`.
Modify `src/launcher.rs` or `src/palette.rs` only if a real journey regression requires it.

**Consumes:** Existing `PaletteView`, `PaletteCommand`, and fixture catalogs.
**Produces:** The source-based harness seam above and executable journey coverage for later render changes.

- [ ] Add the harness seam and these test-only fixture definitions:
  - `nested_navigation.yaml`: Workspace (`🗂`, alias `ws`) → Tools (`🧰`, alias `tools`) → Service status (`🟢`, alias `status`, direct alias `service-status`).
    Its fixed base is `https://status.example.com/company/` and destination is `/overview?view=summary`.
  - `four_inputs.yaml`: one flow with input labels/captures `X/x`, `Y/y`, `Z/z`, `W/w`.
    Base `https://review.example.com`, destination `/x/{x}/y/{y}/z/{z}/w/{w}`.
  - `common_review.yaml`: Environment choice with Staging/staging and Production/production, captured as `environment`.
    Its shared continuation asks Application/application, then Review/review.
    Base `https://review.example.com`, destination `/{environment}/application/{application}?review={review}`.
- [ ] Add `fixed_destinations_complete_without_an_input_screen`:
  keyboard navigation through Tools → Service status emits exactly one request for
  `https://status.example.com/company/overview?view=summary`.
  Use a separate harness for pointer activation of each terminal option in `fixed.yaml`.
  Assert `/company/overview` and `/company/incidents` respectively, with no additional answer confirmation.
  The fixed-destination assertion is:
  ```rust
  assert_eq!(opened(&h), "https://status.example.com/company/overview?view=summary");
  ```
- [ ] Add `nested_and_direct_aliases_keep_their_scope`:
  navigate `ws`, `tools`, `status` without Enter.
  Separately invoke `service-status` at root and assert the same URL.
  Typing scoped `status` at root does not open anything.
  Retain the existing alias-handoff test for an empty Dashboard field.
  At a free-text input, typing `open-dd` remains a draft and emits no opening request.
- [ ] Add `branching_journeys_use_labels_without_substituting_literals`:
  run HTTP Traffic once with typed `acc`, an explicit Accounts click, footer confirmation, and Destination `billing`.
  Assert `https://app.datadog.com/company/8c21-http?from=accounts&to=billing`.
  Repeat without suggestion selection and assert `https://app.datadog.com/company/8c21-http?from=acc&to=billing`.
  Run User Activity/alice and assert `https://app.datadog.com/company/9d32-users?userID=alice`.
  Assert the other branch's input labels are absent at each step.
- [ ] Add `back_and_branch_change_discard_only_invalidated_answers`:
  confirm Accounts, type unconfirmed `billing`, then Back.
  The Source application field contains `accounts`, selected for replacement.
  Confirming it returns to an empty Destination application.
  Back to Dashboard and choose User Activity. Its User identifier field is empty.
- [ ] Add `unchanged_answers_preserve_the_committed_suffix` using `four_inputs.yaml`:
  confirm `one`, `two`, `three`, and stop at W.
  Back twice to Y. Confirm unchanged `two` and observe Z restored as `three`.
  In a separate run, replace Y with `changed`. Z and W must both require fresh text.
  Complete with `new-three` and `four`, asserting
  `https://review.example.com/x/one/y/changed/z/new-three/w/four`.
- [ ] Add `common_choice_changes_invalidate_the_shared_suffix`:
  confirm Staging and `accounts`, stop at Review, then Back to Environment.
  The restored choice remains visible even after the previous filter matched only Staging.
  Reconfirming Staging restores Application `accounts`.
  Changing to Production instead restores an empty Application.
  Complete `billing` and `checked`, asserting
  `https://review.example.com/production/application/billing?review=checked`.
- [ ] Add `view_preserves_context_specific_url_text` with literal expected URLs from the approved manifest:
  cover `café/ops & SRE`, ` accounts `, and `%2F` at Application.
  Expect the encoded path fragments `caf%C3%A9%2Fops%20%26%20SRE`, `%20accounts%20`, and `%252F`.
  Assert the complete URL, including the query and `owner=backup`, not just those fragments.
- [ ] Run `cargo test -p urlchemy-app --locked --test ui journeys`.
  Expected: characterization passes or named behavioral failures. Record the actual result before changing production code.
- [ ] Fix only observed command, history, or event-routing failures. Keep the core as the answer owner.
  If a core defect appears, add its focused core regression before changing it and record the expanded file scope.
- [ ] Run `cargo test --workspace --locked`. Expected: all original and new tests pass.
  Do not commit.

## Task 2: Derived presentation, row metadata, and commitment labels

**Files:** `src/launcher.rs`, `src/palette.rs`, new `src/palette/presentation.rs`,
and `tests/support/journeys.rs`.

**Consumes:** `Launcher::snapshot`, `rows`, `active`, `query`, `presentation`,
`is_opening`, `opening_error`, and existing readonly catalog metadata.
**Produces:**

```rust
// launcher.rs: crate-private, valid only for current browsing rows.
pub(crate) fn navigation_entry(&self, id: urlchemy_core::EntryId)
    -> Option<&urlchemy_core::Entry>;

// palette/presentation.rs: visible only to its parent palette module.
pub(super) enum PaletteSurface {
    Navigation, Input, Selection, DestinationError,
    OpeningPending, OpeningFailed, Inactive,
}
pub(super) struct PalettePresentation; // private fields
impl PalettePresentation {
    pub(super) fn derive(launcher: &Launcher) -> Self;
    pub(super) fn surface(&self) -> PaletteSurface;
    pub(super) fn context(&self) -> &[String];
    pub(super) fn field_label(&self) -> Option<&str>;
    pub(super) fn primary_label(&self) -> Option<&str>;
    pub(super) fn primary_enabled(&self) -> bool;
    pub(super) fn body_message(&self) -> Option<&str>;
    pub(super) fn error_message(&self) -> Option<&str>;
    pub(super) fn commitment_hint(&self) -> Option<&str>;
}
```

The derived type is not persistent session state. Do not derive URL/draft-bearing Debug output.
Use an internal `RowDecoration` with private fields and readers for emoji, scoped alias, and folder/flow kind.
Construct it from the current navigation entry, not a recursively searched unrelated catalog entry.

- [ ] Add `empty_and_filtered_feedback_is_mode_specific` and `opening_state_overrides_completed_snapshot`
  in the new module before defining the policy implementation.
  Test empty catalog, empty folder, filtered navigation, selection, input, destination error, pending open, and failed open.
  Drive real controller commands rather than constructing impossible snapshots.
  Expect feedback:
  `No flows configured`, `No entries in this folder`, `No matches`,
  and `No suggestions. You can use the text you enter.` as applicable.
  Empty-catalog help also says `Edit the configuration file. Restart URLchemy.`
- [ ] Test intermediate input commitment:
  name the test `commitment_labels_follow_explicit_selection`.
  Literal `acc` produces `Use “acc”`.
  Explicit Accounts selection produces `Use Accounts`.
  A blank intermediate draft uses `Continue` and remains confirmable for validation.
  Final input uses `Open URL`.
  Explicit selection supplies supporting text `Using Accounts`.
  Confirmed input errors use the existing safe diagnostic message.
  Include these assertions after driving a real intermediate input with `acc` and Accounts selection:
  ```rust
  let presentation = PalettePresentation::derive(&launcher);
  assert_eq!(presentation.primary_label(), Some("Use Accounts"));
  assert_eq!(launcher.query(), "acc");
  ```
- [ ] Add `navigation_metadata_is_current_context_only`.
  Test that `navigation_entry` rejects stale or foreign-catalog IDs and returns no entry outside browsing.
  Reuse its current-path lookup inside entry activation rather than maintaining two traversal implementations.
- [ ] Run `cargo test -p urlchemy-app --locked --lib palette::presentation`.
  Expected: RED for the missing derived policy.
- [ ] Implement the private policy. Opening state takes precedence over the core's Completed snapshot.
  Derive context from `SessionPresentation`, never from draft text or captured values.
  An unavailable snapshot produces safe inactive feedback and Close, not a panic.
- [ ] Add `navigation_rows_show_configured_metadata` through the production view:
  assert visible emoji and alias observations for Workspace, Tools, and Service status.
  Preserve each row's accessible name as its configured label.
  Verify icon and label alignment when adjacent entries lack metadata.
  Expected before renderer changes: RED for missing metadata.
- [ ] Render standard `ListItem` rows with a fixed rem-based icon slot and a trailing alias lane.
  Use configured emoji, otherwise GPUI Kit's Folder/File icons.
  Preserve domain-derived row IDs, configuration order, explicit selection, and deferred activation guards.
  Do not add ordinal hints.
- [ ] Add `footer_tracks_explicit_selection_and_literal_draft`:
  show `Use Accounts` after explicit selection while the actual input remains `acc`.
  Editing clears selection and changes the label to the literal commitment.
  Pointer and configured-key confirmation must still emit the expected exact URL once.
  Run it before applying the derived footer labels, then make it pass.
- [ ] Run `cargo test --workspace --locked`. Expected: all tests pass without native effects.
  Do not commit.

## Task 3: State-specific surfaces and measured window sizing

**Files:** `src/palette.rs`, new `src/palette/layout.rs`,
`tests/support/journeys.rs`, `tests/shell.rs`, and `tests/support/mod.rs` only if the service double needs extension.

**Consumes:** Task 2 `PalettePresentation` and the existing `PaletteEvent::ContentHeight`.
**Produces:**

```rust
// palette/layout.rs: all dimensions come from actual component measurements.
pub(super) fn preferred_height(
    header: Pixels,
    question: Option<Pixels>,
    body: Pixels,
    footer: Pixels,
    rem: Pixels,
) -> Pixels;
```

Use named header, optional question, body, and footer regions.
The question region includes its label and editor.
Keep `render_header`, `render_question`, `render_body`, and `render_footer` as private PaletteView methods returning `AnyElement`.
They consume the derived presentation, current Window, and Context.
Do not change the public command, effect, or desktop service signatures.

- [ ] Add `empty_feedback_distinguishes_catalog_folder_and_filter`:
  empty catalog shows configuration/restart help, an empty folder offers Back, and unmatched navigation Enter emits no effect.
  No-suggestion input still accepts literal confirmation.
  Run the test before changing the generic `No matches` renderer. Expected: RED.
- [ ] Add `destination_error_has_back_without_an_editable_field`:
  use a test-only single-input flow with a suggestion labelled Parent and stored value `..`.
  After selecting and confirming it, expect `A rendered path segment must not be a dot segment.`
  Assert no `launcher-field`, no rows, visible/focused Back, no opening effect, and Enter on Back restores the input.
  Expected before mode-specific rendering: RED.
- [ ] Add `pending_opening_blocks_duplicate_submission` to shell tests:
  complete the real flow with the existing injected unresolved completion.
  Observe `Opening in your default browser.`, no editor, and available Close.
  Extra Enter and pointer attempts cause no second service request.
  Reinvoke before resolving the old completion. The new root remains open.
  Expected before mode-specific rendering: RED.
- [ ] Extend `open_failure_retry_and_start_over`:
  failure shows only the recovery surface, with Retry focused.
  Enter requests the same URL again exactly once.
  Start over restores root without the completed answers.
  Close and Escape preserve the existing close-reason focus policy.
- [ ] Add `named_regions_determine_preferred_height` before `preferred_height`.
  Use outer `p_4` and inter-region `gap_3`, with body height capped at `24rem`.
  Count only present regions. Include questionless errors, wrapped body feedback, and larger rem.
  Include:
  ```rust
  assert_eq!(preferred_height(px(30.), Some(px(60.)), px(400.), px(30.), px(10.)), px(402.5));
  assert_eq!(preferred_height(px(30.), None, px(400.), px(30.), px(10.)), px(335.));
  assert_eq!(preferred_height(px(30.), Some(px(60.)), px(400.), px(30.), px(20.)), px(605.));
  ```
  Expected: RED for the missing sizing function.
- [ ] Render each PaletteSurface with applicable controls:
  - Navigation, input, and selection retain their current editor path.
  - Destination error shows Back recovery.
  - Pending open shows waiting feedback and Close.
  - Failed open shows Retry, Start over, and Close.
  Keep Escape/Back repeat suppression and composition precedence.
  Do not focus a hidden editor or retain a stale input subscription in a non-input surface.
  Make the retained edit subscription optional if needed for these transitions.
  Preserve the public `editor()` reader and create fresh edit generations when returning to an editable surface.
- [ ] Replace the root `bounds[3]` assumption with named-region measurements.
  Use `ListState::scroll_handle().content_size()` for natural list-body height and measured feedback height for non-list bodies.
  Include revision, width, surface, and rem in any retained measurement validity.
  Emit ContentHeight only for coherent changed measurements.
  Release the entity and window leases before delivery.
  Keep the shell's existing top-anchor and remaining-work-area clamp.
- [ ] Run `cargo test -p urlchemy-app --locked --test ui` and `cargo test -p urlchemy-app --locked --test shell`.
  Expected: all tests pass, including original deferred-event, native-policy, and sizing regressions.
- [ ] Run `cargo test --workspace --locked`. Expected: all tests pass.
  Do not commit.

## Task 4: Context, focus, event boundaries, and final evidence

**Files:** `src/palette.rs`, new `src/palette/context.rs`,
`tests/support/journeys.rs`, `tests/shell.rs`, `docs/native-development.md`, and `ROADMAP.md`.

**Consumes:** Tasks 1–3 tests and private presentation/surface/measurement seams.
**Produces:** Complete non-ordinal acceptance evidence and documented remaining limits.

- [ ] Add `tab_order_follows_the_current_surface`:
  normal flow order is field → primary → Back → Close.
  Reverse traversal reverses that order.
  A destination error begins at Back, a failed opening begins at Retry, and no hidden editor receives focus.
  Typing or Tab never accepts a suggestion.
  Run before making any necessary focus fix. Preserve any already-passing behavior.
- [ ] Add `queued_activation_does_not_cross_a_restored_context`:
  queue an actual row click, then perform Back or context replacement before its deferred owner callback.
  Assert no row from the former context becomes an answer and no unintended opening effect occurs.
  Keep the existing old-editor alias test and stale-operation shell tests.
  If this already passes, record characterization evidence rather than changing working guards.
- [ ] Add `context_keeps_the_current_object_and_excludes_answers`:
  create long nested labels and navigate into HTTP Traffic.
  At narrow width, retain the current context object while eliding older ancestors first.
  Observe the full configured context through accessible text.
  Enter `private-answer-sentinel` and assert it never appears in context help.
  Expected before context implementation: RED for missing accessible full context and elision behavior.
- [ ] Add a private context module with:
  `context_suffix(labels: &[String], available: Pixels, measure: impl FnMut(&str) -> Pixels) -> String`.
  Drop oldest ancestor labels until the suffix fits. Use `… / ` when ancestors were omitted.
  If the current object alone is too wide, let standard text truncation apply while preserving the full accessible text.
  Measure actual framework text with the active font. Invalidate layout on width, labels, or rem changes.
- [ ] Provide hover and keyboard context help without adding another Tab stop.
  When context is truncated, focusing the existing Back button exposes the full context tooltip.
  Escape dismisses that tooltip first and does not also dismiss the palette.
  Blur, context replacement, or invocation replacement hides it.
  Preserve the ordinary Back button activation and complete context accessibility label.
- [ ] Use verified GPUI Kit `component::tooltip::Tooltip::new(...).build(window, cx)` content.
  Kit 0.9.1's managed Button tooltip is hover-only. Do not assume `.tooltip(...)` adds keyboard behavior.
  For keyboard deployment, retain the tooltip content outside render and use GPUI's public `AnyTooltip` with `Window::set_tooltip` in prepaint.
  Its visibility callback must check the live focus/context generation.
  Do not call `set_tooltip` from an input handler, duplicate Root, or install another component overlay provider.
  Test show, first-Escape dismissal, subsequent palette dismissal, blur, and stale context behavior.
- [ ] Add `state_layout_remains_usable_with_long_content`:
  exercise long labels, empty results, inline errors, destination errors, pending open, and failed open.
  Use base fonts 14 and 20, a 360-point viewport, and a larger list.
  Assert positive controls, shared leading edges, contained footer/Close, and body-only scrolling.
  Verify suggestion selection survives a frame while plain hover does not replace it.
  Keep the existing 24-point long-list work-area clamp test.
- [ ] Update the run guide and roadmap with actual results.
  State that ordinals remain unimplemented and that new headless coverage does not extend native verification.
  Preserve the maintainer's earlier native evidence and distinguish it from this increment.
- [ ] Run `sh scripts/doctor.sh` and `sh scripts/test-doctor.sh`.
  Expected: inspect actual prerequisites and all three diagnostic regressions.
  Do not install tools or initiate authentication if a prerequisite is absent.
- [ ] Run `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked`, and `cargo test -p urlchemy-core --locked`.
  Expected: all checks pass.
- [ ] Compile, but do not execute, the native harness:
  `cargo test -p urlchemy-app --features native-acceptance --test native_acceptance --no-run --locked`.
  Expected: compilation succeeds without native effects.
- [ ] Check changed English prose, local links, the complete diff, and dependency stability.
  Expected: no dependency changes, private paths, secrets, fixture-contract mutations, or unrelated edits.
- [ ] Perform author self-review against the spec, this plan, and the five Review Focus risks.
  Record findings and test evidence in this plan's ignored execution ledger.
  Report any unresolved issue honestly. Do not request an independent subagent review without permission.

## Self-review and handoff

Task 1 establishes the complete journeys and suffix behavior before presentation changes.
Task 2 adds derived policy and navigation decoration without changing session semantics.
Task 3 owns state-specific surfaces, keyboard recovery, and their window measurements.
Task 4 closes focus, context, queued-event, geometry, documentation, and verification requirements.

Named tests in the owning tasks cover all five Review Focus risks.
The core remains the only answer/history owner. The current plan requires no new core presentation API.
The author checked interfaces and spec coverage. No independent reviewer participated.

The approved execution method is inline with TDD and author self-review, without subagents.
The maintainer approved this plan and requested a separate documentation merge before implementation.
Committing, publishing, native testing, and the later ordinal increment still require their own authorization.
