# MVP journeys before viewport ordinals

Status: approved by the maintainer on 2026-11-17.
The maintainer selected journey completion before viewport ordinals and approved inline execution without subagents.
The maintainer also approved the implementation plan and requested a separate documentation merge before product-code changes.

## Intent

Complete the agreed journeys through the production palette before adding viewport-numbered shortcuts.
The native vertical slice established a working linear journey.
The next increment must expose the remaining core behavior without duplicating its answers or traversal rules.

Sources:

- [Product acceptance criteria](../../product.md#MVP-acceptance-criteria)
- [Approved interaction contract](2026-11-15-interaction-contract-design.md)
- [YAML fixtures and exact URL expectations](yaml-url-contract/expectations.json)
- [Native evidence and limitations](../../native-development.md)
- [Verified testing APIs and deferred-event boundaries](../../testing-probe.md)

## Approaches and selected order

| Approach | Benefit | Cost | Decision |
| --- | --- | --- | --- |
| Complete journeys and their presentation first | Establishes correction, branches, and mixed-input behavior before viewport mapping | Numbered shortcuts remain unavailable during this increment | Selected by the maintainer |
| Implement viewport ordinals first | Adds a navigation shortcut sooner | Leaves incomplete journey coverage behind a new activation path | Next increment, after journey completion |

Within the selected approach, extend the existing production view and controller.
A second palette or a new session model would duplicate working behavior without solving the coverage gaps.

## Current implementation

`Session` owns answers, history, validation, branch traversal, and URL resolution.
`Launcher` translates palette commands into core actions and emits desktop effects.
`PaletteView` owns editor generations, list presentation, focus, and input routing.
`AppShell` owns startup, native registration, window lifecycle, and desktop effect handling.

Current UI integration tests cover linear input, explicit suggestions, literal text, custom confirmation, raw control-character paste, composition, alias handoff, and Back.
Controller tests exercise one common continuation and resolution-error recovery.
The production view does not yet prove the complete fixed, branching, and correction journeys.

The current row renderer displays labels but omits configured emojis and alias hints.
Its empty message does not distinguish an empty catalog from an unmatched suggestion filter.
Its normal editor surface remains visible during destination errors and browser-request states.
These are gaps against the approved contract, not requests for a new visual design.

## Goals

1. Verify fixed, linear, common-continuation, and branching journeys through real palette interactions.
2. Verify correction, suffix preservation, branch invalidation, cancellation, and fresh invocation.
3. Complete navigation row metadata and mode-specific feedback.
4. Keep keyboard, pointer, and footer actions on the same semantic command path.
5. Preserve exact URLs, text, focus, and privacy behavior.

## Non-goals

- Viewport-numbered Command+1…9 activation or hints.
- New YAML fields, new dependencies, or changes to URL semantics.
- Fuzzy search, remote suggestions, answer summaries, or a final confirmation screen.
- New native services, configuration reload, packaging, signing, or release publication.
- A complete native accessibility, IME, Spaces, or fullscreen claim.

## Journey behavior

### Navigation and direct destinations

Folders and flows retain their stable core IDs.
Each navigation row shows its configured emoji or an appropriate framework fallback icon.
The icon and alias lanes retain their alignment when either value is absent.
Show the current entry's scoped alias when configured.
Root direct aliases remain executable without duplicating their nested flow as a root row.

Keep exact aliases case-sensitive and label filtering case-insensitive.
Aliases operate only during navigation.
One activation enters one folder or flow without copying navigation text into a parameter field.

A fixed destination emits one opening request immediately after activation.
It requires no input screen or additional confirmation.
An empty result set has no active row or activation.

### Inputs, suggestions, and choices

Keep the existing one-row Textarea and immutable committed-edit subscription.
Do not trim, normalize, or substitute accepted input text.
Suggestion labels remain distinct from their stored values.
Visible suggestions do not imply selection.
Typing clears explicit selection. Hover does not change confirmation.

Keep suggestion selection in the core and active-row presentation in the controller.
A suggestion click selects without confirming.
A choice click selects and confirms.
Arrows retain editor focus and stop at list boundaries.
The configured confirmation shortcut and footer action produce the same result.

The footer must describe the actual commitment.
For an intermediate input, identify the selected suggestion or literal draft without displaying its hidden stored value.
For a final input, retain the `Open URL` action and identify explicit suggestion selection in supporting text when necessary.
Do not log the displayed draft or use it in diagnostic messages.

### Correction and committed suffixes

Back restores the committed answer, not an unfinished later draft.
An input answer is selected for replacement.
A restored choice remains selected and visible.
Transient filters do not hide that restored choice.

Confirming an unchanged answer preserves its committed suffix.
Changing an answer discards every later answer.
Changing a branch choice invalidates the former continuation, even when captured strings happen to match.
The same invalidation rule applies to a shared continuation.

Use test-only extended journeys when a public fixture resolves too soon to inspect a committed suffix.
Do not change the approved contract fixtures merely to create a longer history.
The four-input `X → Y → Z → W` journey proves unchanged-suffix preservation and changed-answer invalidation.

### State-specific feedback and recovery

| State | Required surface |
| --- | --- |
| Empty catalog | Explain that no flows are configured and that file changes require a restart. Keep Close available |
| Empty folder | Show `No entries in this folder`. Keep Back and Close available |
| No navigation or choice matches | Show `No matches`. Keep filtering available. Confirmation activates nothing |
| No suggestion matches | Explain that literal text remains valid. Do not disable confirmation solely because suggestions are absent |
| Invalid confirmed input | Keep the field, draft, and focus. Show the existing safe diagnostic beside the field |
| Destination resolution error | Show the diagnostic and Back recovery, not an editable input or stale result rows. Focus Back |
| Browser request pending | Show waiting feedback only during a real pending request. Prevent duplicate opening requests. Keep dismissal available |
| Browser request failed | Show Retry, Start over, and Close. Focus Retry. Do not display a completed session as an editable flow |

Editing clears stale input errors.
Retry repeats the retained URL request, not core confirmation.
Start over creates a fresh root within the current invocation.
Dismissal and fresh invocation discard transient UI state and invalidate old callbacks.

## Presentation and architecture

Retain the approved compact structure, theme tokens, rem-based scale, and semantic Kit components.
The context line must retain the current object when older ancestors need elision.
Provide the full context as accessible text and through a keyboard-accessible tooltip when truncation requires explanation.
Keep captured answers out of that context.

Only the body scrolls. Keep the context, field when applicable, and current recovery action visible.
Retain measured compact sizing, a stable top anchor, and work-area clearance at larger text sizes.
State-specific layouts must not depend on the existing fixed child index used by height measurement.
Adapt that measurement boundary deliberately when the surface structure changes.

Reuse readonly `Row::emoji`, `Entry::alias`, and `SessionPresentation` metadata.
If the controller needs a derived presentation value, derive it from the current catalog, snapshot, and stable IDs.
Do not parse configuration again or introduce a second answer cache.
Add a narrow core presentation seam only if the existing public data cannot express approved behavior.

## Acceptance evidence

| Area | Required automated evidence |
| --- | --- |
| Fixed destinations | Activate nested destinations by keyboard and pointer. Assert the exact approved URL and one opening effect |
| Aliases | Exercise scoped folder/flow aliases and root direct access. Assert an empty next-step field and no alias activation during capture |
| Branches | Complete HTTP Traffic with an explicit suggestion and with literal text. Complete User Activity without displaying the traffic branch's fields |
| Common continuation | Complete Staging and Production journeys. Use an extended fixture to prove later committed answers disappear after a choice change |
| Correction | Abandon an unconfirmed draft, preserve an unchanged committed suffix, and invalidate a changed suffix through the real view |
| Branch change | Return to Dashboard, select the other branch, and assert that former branch answers do not reappear |
| Exact text | Exercise Unicode, reserved characters, surrounding spaces, and percent signs through the view. Assert exact expected URLs |
| Mixed input | Mix row clicks, arrows, configured confirmation, and footer buttons without duplicate effects |
| Feedback | Exercise empty catalog, empty folder, unmatched filters, literal input without suggestions, invalid input, and destination error recovery |
| Opening states | Use injected pending, failed, and successful requests. Verify waiting feedback, Retry, Start over, dismissal, and stale-result rejection |
| Focus and geometry | Verify applicable Tab order, selected-text replacement, restored-choice visibility, footer containment, and long rows at larger fonts |

Use the production view, real core, fixture catalogs, and injected desktop services.
Keep helpers and extended fixtures in test support rather than production classes.
Wait across deferred-event boundaries before observing outcomes.
Treat accessibility properties as partial evidence and pair them with actual interactions.

Run formatting, Clippy with warnings denied, the workspace suite, and the independent core suite.
Native launches and browser opening need separate permission for this increment.
No additional native acceptance follows automatically from the previous linear-fixture permission.

## Execution and approval boundary

Publish this design, its plan, and the roadmap update from `feat/mvp-journeys`.
Implement on a separate branch from the resulting `main`.
Proposed execution: inline, test-first, with author self-review and no subagents.
The previous publication request does not authorize automatic commits, publication, or merges for this new increment.

The author reviewed this design against the existing implementation and approved interaction contract.
No independent reviewer participated.
The author's builder check found no unresolved product decisions that prevent an implementation plan.
The approved implementation plan defines the bounded execution steps.
Viewport ordinals remain the next separate increment.
