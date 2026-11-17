# Viewport ordinals design

Status: the maintainer approved the approach and this written design on 2026-11-17.

## Intent and scope

Add fast keyboard activation for navigation entries that the palette currently shows.
Preserve the approved [interaction contract](2026-11-15-interaction-contract-design.md), especially sections 4 and 5.
Build on the completed, uncommitted [journey-first increment](2026-11-17-mvp-journeys-design.md).

Command+1 through Command+9 activate the first nine fully visible navigation entries.
Number entries in filtered configuration order, starting at 1 for the current viewport.
Do not number closed options, suggestions, diagnostics, or browser-request surfaces.
Ordinary digits remain text input.
Do not change aliases, filtering, confirmation, Back, or focus traversal.

## Approaches

The maintainer approved measured visibility with the existing List component.
Keep framework-owned virtualization, selection, and scrolling.
Measure actual row bounds against the effective clipped viewport.
Use one coherent mapping for shortcut hints and keyboard activation.

Two alternatives do not fit this increment:

- Index arithmetic depends on assumed row geometry and can include partial rows.
- Replacing List duplicates existing behavior and expands the regression surface.

GPUI Kit 0.9.1 renders partial rows and additional content outside the visible area.
Its virtualized render range therefore does not prove full visibility.
The scroll handle exposes viewport bounds and offset.
GPUI also exposes the current content mask during prepaint.
Implementation must verify hook ordering against the selected source.

## Presentation

Retain the current icon, label, and alias columns.
Reserve a trailing shortcut column so hint changes do not move labels or aliases.
Use the existing theme and rem-based sizing.
Show the complete shortcut, not an unexplained digit.
Do not add focus stops or pointer controls.

```text
🗂  Tools                         tools    ⌘1
🟢  Service status                status   ⌘2
```

Partially clipped rows receive no hint.
The tenth fully visible entry receives no hint.
Keep each row's accessible name and expose its applicable shortcut through accessible metadata or labeled hint content.
Headless metadata checks do not establish VoiceOver behavior.

## Ownership and coherence

Keep ordinal state private to the palette.
The core and Launcher continue to own navigation and activation semantics.
Resolve ordinals to stable EntryId values, never retained result indexes.
Dispatch the existing Activate command after validation.
Do not add a second selection model.

Collect row geometry for one presentation generation.
Include the complete row frame, not only its label or content frame.
Intersect the list viewport with applicable ancestor clipping before checking full containment.
Order eligible rows by their current result positions and retain at most nine IDs.

The displayed hints and the activation map must describe the same presentation.
A newly measured map is not actionable before its hints reach presentation.
Reject stale deferred geometry instead of applying it to a later result set.
Do not update the map incrementally from individual row callbacks.

Invalidate ordinal activation when any relevant input changes:

- navigation context, query, or result IDs
- scroll offset or pending scroll-to-item request
- viewport bounds, window size, or rem size
- row geometry or presentation generation.

Clear the map on context or query changes, even when some result IDs remain.
After scrolling or resizing, recompute hints from measured full visibility.
Suppress activation while measurement and displayed hints disagree.
Validate current geometry inputs again when handling a shortcut.
Never dispatch an old ID merely because it remains somewhere in the result set.
Conservative suppression during a transition is preferable to activating the wrong entry.

## Keyboard behavior

Recognize Command+1 through Command+9 only in navigation.
Require Command without additional Control, Option, or Shift modifiers.
Apply the existing composition guard before activation.
Ignore held-key repeats.
One key event activates at most one entry.
A fixed destination can then produce its normal browser-request effect.
Do not replay the event into a new folder or input.

Keep this behavior on the palette's existing keyboard path.
Do not introduce global ordinal bindings.
Preserve ordinary text editing and existing scoped key suppression.

## Acceptance and verification

Use pure tests for full containment, ordering, the nine-entry limit, and stale-generation rejection.
Use the production PaletteView for headless interaction tests.
Follow the deferred-event boundaries in the [testing probe](../../testing-probe.md).

Cover these observable cases:

1. Root and nested navigation show correct hints and activate the indicated entries.
2. Filtering restarts numbering without preserving configuration positions.
3. Scrolling excludes partial rows and renumbers fully visible rows.
4. Keyboard-driven scrolling and resizing also refresh the mapping.
5. A missing hint never activates an entry, including an offscreen entry.
6. A queued shortcut cannot cross a query, context, or viewport change.
7. Ordinary digits, extra modifiers, held repeats, and marked text do not trigger ordinal activation.
8. Inputs and closed selections have no navigation hints or ordinal activation.
9. Existing mixed-input journeys, aliases, Tab traversal, and fixed destinations still work.
10. Shortcut columns remain aligned at smaller widths and larger base fonts.

Run formatting, Clippy with warnings denied, and the locked workspace suite.
Run the documented prerequisite diagnostics.
Report pure tests, headless UI tests, and native evidence separately.

## Execution boundaries

Prepare a bounded implementation plan after approval of this document.
Keep execution inline with TDD and author self-review.
Do not use subagents or create a worktree.
Preserve the uncommitted journey-first work in the current checkout.
Do not commit, publish, or merge either increment without separate authorization.
Do not change dependencies, native services, configuration schema, or core production behavior.
Native launches, browser opening, screenshots, and OS permission requests require separate permission.
