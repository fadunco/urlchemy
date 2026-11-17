# Viewport Ordinals Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan inline, task by task. Steps use checkbox syntax.

**Goal:** Activate only navigation entries with displayed Command+1 through Command+9 hints.

**Architecture:** Keep List virtualization and existing Launcher activation.
A private palette module owns measured visibility and coherent ordinal mappings.
A small row adapter measures the complete ListItem without replacing its behavior.

**Tech Stack:** Rust `1.100.0`, GPUI Kit `=0.9.1`, and the existing locked dependencies.

**Spec:** [Approved viewport design](../specs/2026-11-17-viewport-ordinals-design.md).
The [interaction contract](../specs/2026-11-15-interaction-contract-design.md) remains authoritative.

Status: the maintainer approved this plan on 2026-11-17.

## Global Constraints

- Keep execution inline with TDD and author self-review.
- Do not use subagents or create a worktree.
- Preserve the uncommitted journey-first work in the current checkout.
- Do not commit, publish, or merge either increment without separate authorization.
- Do not change dependencies, native services, configuration schema, or core production behavior.
- Native launches, browser opening, screenshots, and OS permission requests require separate permission.
- Number only the first nine fully visible navigation entries in filtered configuration order.
- Preserve ordinary digits, aliases, filtering, confirmation, Back, and focus traversal.
- Reject stale geometry and suppress activation whenever the displayed mapping is not coherent.
- Read `docs/testing-probe.md` before writing interaction tests.
- Keep temporary execution evidence under ignored `.superpowers/`.

## Review Focus

1. Scroll followed immediately by a shortcut must not activate the previous viewport's entry.
2. Repeated row measurement must not duplicate IDs or consume an ordinal.
3. Selection-driven scrolling and rem changes must invalidate geometry without a query change.
4. A pending editor subscription must not let a shortcut activate the previous filter.
5. Held keys and composition must not carry an activation into a new folder or flow.

## File Responsibilities

| File | Responsibility |
| --- | --- |
| `crates/urlchemy-app/src/palette/ordinals.rs` | Private geometry policy, frame collection, and activation validation |
| `crates/urlchemy-app/src/palette/ordinal_row.rs` | Complete-row measurement and fixed-width shortcut presentation |
| `crates/urlchemy-app/src/palette.rs` | Frame lifecycle, List delegate integration, and keyboard dispatch |
| `crates/urlchemy-app/tests/support/ordinals.rs` | Production-view ordinal interaction tests |
| `crates/urlchemy-app/tests/ui.rs` | Include the new child test module |
| `docs/native-development.md`, `ROADMAP.md` | Verification evidence and remaining native limits |

Do not change Launcher commands or public desktop interfaces.
Do not restructure the existing presentation, context, or layout modules.

## Shared Interfaces

Keep all new production interfaces private to the palette.
Use `pub(super)` only where sibling modules require access.

```rust
// ordinals.rs
#[derive(Clone, PartialEq)]
pub(super) struct OrdinalGeometry {
    // Private fields: revision, viewport, scroll offset, window size, rem.
}
impl OrdinalGeometry {
    pub(super) fn new(
        revision: u64,
        viewport: Bounds<Pixels>,
        offset: Point<Pixels>,
        window_size: Size<Pixels>,
        rem: Pixels,
    ) -> Self;
}
pub(super) struct OrdinalFrame;
impl OrdinalFrame {
    pub(super) fn new(geometry: OrdinalGeometry) -> Self;
    pub(super) fn record(
        &mut self, result_ix: usize, id: EntryId,
        bounds: Bounds<Pixels>, clip: Bounds<Pixels>,
    ) -> Option<u8>;
    pub(super) fn finish(self) -> Option<OrdinalMapping>;
}
pub(super) struct OrdinalMapping;
impl OrdinalMapping {
    pub(super) fn resolve(
        &self, number: u8, current: &OrdinalGeometry,
    ) -> Option<EntryId>;
}
```

`record` returns a number only for a fully contained, positive-size row.
Intersect viewport and effective clipping before checking containment.
Require ascending result positions during actual row prepaint.
Reject duplicate IDs or positions instead of counting them twice.
Reject an incoherent frame as a whole.
The implementation must distinguish preliminary measurement from actual presentation.

---

## Task 1: Pure visibility and mapping policy

**Files:** Create `src/palette/ordinals.rs`. Add its private module declaration in `src/palette.rs`.
Paths in task sections are relative to `crates/urlchemy-app/`.

**Consumes:** `EntryId`, GPUI geometry types, and current palette revision.
**Produces:** The shared interfaces above.

- [ ] Add `fully_visible_rows_receive_at_most_nine_ordinals`.
  Use viewport `(0, 10, 100, 100)` and twelve rows, each 10 units high.
  Row 0 starts at y=5 and receives no number.
  Rows 1–9 start at y=15 through y=95 and receive 1–9.
  Row 10 starts at y=105 and receives no number.
  Row 11 starts at y=115 and receives no number.
- [ ] Add `containment_uses_full_row_and_effective_clip`.
  Assert no number for vertical clipping, horizontal clipping, empty bounds, or zero-height viewport.
  Assert full containment at an exact viewport boundary remains eligible.
- [ ] Add `duplicate_or_reordered_samples_reject_the_frame`.
  Assert `finish()` returns None for duplicate IDs, duplicate positions, or decreasing positions.
- [ ] Add `mapping_rejects_changed_geometry`.
  Change each geometry field independently and assert `resolve(1, changed)` returns None.
  Assert numbers 0 and 10 never resolve.
- [ ] Run `$HOME/.cargo/bin/cargo test -p urlchemy-app --locked --lib palette::ordinals`.
  Record RED for the missing interfaces.
- [ ] Implement the shared interfaces without UI mutation or side effects.
  Retain at most nine eligible IDs and the geometry that produced them.
- [ ] Repeat the focused command and record GREEN.

## Task 2: Frame-coherent hints on the production list

**Files:** Create `src/palette/ordinal_row.rs` and `tests/support/ordinals.rs`.
Modify `src/palette.rs` and `tests/ui.rs`.

**Consumes:** Task 1 policy, `ListItem`, `ListDelegate`, and the retained list scroll handle.
**Produces:** A private selectable row adapter and a complete displayed mapping.

```rust
// ordinal_row.rs
pub(super) struct OrdinalRow;
impl OrdinalRow {
    pub(super) fn new(
        item: ListItem,
        sample: Option<(usize, EntryId)>,
        frame: Rc<RefCell<OrdinalFrame>>,
    ) -> Self;
}
```

Forward `Selectable` state to the inner ListItem.
The adapter must preserve its accessibility, pointer behavior, and stable domain identity.
Do not add a focus handle.

- [ ] Include `support/ordinals.rs` as a child module of the existing UI test target.
  Reuse `setup_source`, `ui`, `key`, and `settle`.
- [ ] Add `navigation_hints_follow_complete_rows`.
  Generate 40 uniquely labeled fixed flows with distinct destination paths.
  Compare row bounds against the clipped viewport.
  Assert only the first nine fully visible rows have numbered hints.
- [ ] Add `hints_renumber_after_filter_scroll_and_resize`.
  Filter by labels, scroll by a non-row-aligned amount, and resize the window.
  Assert numbering restarts at 1 for each resulting visible set.
- [ ] Add `shortcut_lane_preserves_row_alignment`.
  Check icon, label, alias, and shortcut columns at widths 360 and 640.
  Repeat at base-font values 14 and 20.
  Assert hint changes do not move the other columns.
- [ ] Run `$HOME/.cargo/bin/cargo test -p urlchemy-app --locked --test ui ordinals`.
  Record RED for missing hints.
- [ ] Implement the row adapter around the complete ListItem frame.
  Reserve a fixed rem-based shortcut lane beside the existing alias lane.
  Use muted theme text and show `⌘1` through `⌘9`.
- [ ] Collect samples only during actual row prepaint, not during render or size measurement.
  Read the effective content mask before entering the row's own clipping.
  Do not infer complete-row bounds from a child label.
- [ ] Supply all rows in one actual list presentation with the same frame collector.
  Verify ascending prepaint order against the selected framework source and tests.
  A narrow custom Element may forward layout and prepare the hint after row bounds become available.
  Keep ListItem interaction and styling unchanged.
- [ ] Finalize the map only after all row prepaint completes.
  Make hint text and accessible shortcut content derive from that same frame.
  Publish the completed map after presentation, outside borrowed entity/window leases.
  Reject delivery if revision or geometry changed.
- [ ] Notify only when the displayed mapping changes.
  Do not create a repeated render/measurement notification loop.
- [ ] Repeat the focused UI command and record GREEN.

If the selected APIs cannot maintain this phase ordering, stop and revise the plan.
Do not substitute previous-frame indexes or replace List.
This is an implementation feasibility gate, not permission to expand scope.

## Task 3: Guarded ordinal keyboard activation

**Files:** Modify `src/palette.rs`, `src/palette/ordinals.rs`, and `tests/support/ordinals.rs`.

**Consumes:** The displayed mapping, live scroll geometry, and `Launcher::query()`.
**Produces:** Navigation-only shortcuts through existing `Cmd::Activate(RowId::Entry(id))`.

- [ ] Add `ordinal_activation_matches_the_displayed_hint`.
  Press each applicable shortcut in a fresh harness.
  Assert exactly one opening effect with the destination from its indicated row.
  Include root, nested folder, and filtered cases.
- [ ] Add `missing_hint_never_activates`.
  Cover an offscreen row, fewer than nine rows, an empty result set, and a partial row.
  Assert no opening effect or navigation change for an unavailable number.
- [ ] Add `scroll_then_shortcut_rejects_the_previous_viewport`.
  Queue wheel input and a shortcut in one window update without an intermediate frame.
  Assert no activation through the previous map.
  Render the new frame, then assert its displayed shortcut works.
- [ ] Add `queued_query_or_context_change_rejects_old_ordinals`.
  Queue text input before its subscription completes, then send a shortcut.
  Repeat after Back and after a folder change.
  Assert no old row becomes an answer or opening request.
- [ ] Add `keyboard_scroll_and_rem_changes_refresh_ordinals`.
  Move selection beyond the viewport with arrows.
  Change the base font without changing query text.
  Assert hints match the new fully visible rows.
- [ ] Add `ordinal_keys_do_not_cross_input_or_composition_boundaries`.
  Cover ordinary digits, extra modifiers, marked text, held repeats, closed choices, and free-text inputs.
  Assert these cases produce no ordinal navigation.
  Assert a held Command+1 cannot descend through multiple folders.
- [ ] Run the focused UI command and record RED for missing activation and guards.
- [ ] Recognize exact Command+1 through Command+9 on the existing palette keyboard path.
  Preserve the existing composition check and suppress held activation repeats.
  Limit scoped suppression to applicable navigation shortcuts.
- [ ] Before activation, require the editor text to match the controller query.
  If it differs, suppress the ordinal and allow the normal edit subscription to complete.
  Do not flush an alias and activate a second entry through the same event.
- [ ] Reject pending scroll-to-item work and changed live geometry.
  Invalidate before palette commands or list synchronization can request programmatic scrolling.
  Restore eligibility only after a completed presentation measures the resulting viewport.
  GPUI Kit keeps pending scroll state private, so do not inspect framework internals.
  Resolve only against the displayed map and current navigation rows.
  Dispatch the existing Activate command once after all checks pass.
- [ ] Invalidate on context/query changes before replacing editor or list state.
  Preserve existing generation guards for deferred events.
- [ ] Repeat focused tests and record GREEN.
- [ ] Run `$HOME/.cargo/bin/cargo test --workspace --locked`.
  Require all existing journey, shell, core, and shortcut tests to pass.

## Task 4: Final evidence and author review

**Files:** Modify `docs/native-development.md` and `ROADMAP.md`.
Record temporary evidence under `.superpowers/`.

**Consumes:** Tasks 1–3 production behavior and test results.
**Produces:** Verification evidence and an explicit statement of remaining native limits.

- [ ] Review all changed and new files against the approved design.
  Review the ordinal delta separately from the preserved journey-first changes.
- [ ] Check each Review Focus item against its named regression test.
  Add a failing regression test before correcting any discovered defect.
- [ ] Run `sh scripts/doctor.sh`.
- [ ] Run `sh scripts/test-doctor.sh`.
- [ ] Run `$HOME/.cargo/bin/cargo fmt --all -- --check`.
- [ ] Run `$HOME/.cargo/bin/cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [ ] Run `$HOME/.cargo/bin/cargo test --workspace --locked`.
- [ ] Run `$HOME/.cargo/bin/cargo test -p urlchemy-core --locked`.
- [ ] Compile the native harness without executing it:
  `$HOME/.cargo/bin/cargo test -p urlchemy-app --features native-acceptance --test native_acceptance --no-run --locked`.
- [ ] Update documentation with actual test counts and commands.
  Distinguish headless results from the earlier native vertical-slice evidence.
  Do not claim new native, screenshot, or VoiceOver verification.
- [ ] Check changed prose, local links, whitespace, and accidental private paths.
- [ ] Record author self-review, remaining findings, and any execution deviation.
  Do not represent self-review as independent review.
- [ ] Report changed files, commands actually run, results, and remaining blockers.
  Leave product changes uncommitted and unpublished.

## Plan Self-Review

- Spec coverage: Tasks 1–3 cover eligibility, presentation, coherence, keyboard guards, and regression cases.
- Phase safety: Task 2 explicitly gates complete-row measurement and same-frame hint publication.
- Type consistency: Task 1 defines the mapping interfaces that Tasks 2 and 3 consume.
- Review Focus: Task 1 covers duplicate samples. Task 3 covers the other four failure modes.
- Scope: The plan excludes core, dependency, native service, installation, publication, and workspace-isolation changes.
- Execution: Retain inline TDD and author self-review after maintainer approval of this plan.
