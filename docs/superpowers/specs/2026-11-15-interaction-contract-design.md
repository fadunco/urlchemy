# Compact palette interaction contract

Status: approved interaction contract for roadmap milestone 4 on 2026-11-15.
The maintainer approved the compact palette and the interaction rules below.
Native measurements remain provisional. Shortcut serialization remains a separate design decision.

## 1. Intent and boundaries

URLchemy helps a technical user reach a configured destination without navigating
web application menus. The overlay should expose one current decision, accept
mixed keyboard and pointer input, and then disappear after successful completion.

Inputs are the [product definition](../../product.md),
[approved domain model](2026-11-14-domain-execution-model-design.md),
[approved YAML/URL contract](2026-11-14-yaml-url-contract-design.md), and the
production core delivered in [milestone 3](../plans/2026-11-14-ui-independent-core.md).

This milestone specifies interaction and low-fidelity structure, not a native
implementation or a polished visual theme. It does not select a GPUI dependency
version, register a shortcut, request permissions, or open destination URLs.
No core behavior or YAML schema changes accompany this document.

Preserve these agreed rules:

- Every invocation starts at root, without previous queries, answers, or errors.
- Folders organize navigation. Flows and their branches collect mandatory values.
- Exact aliases activate without Enter in their agreed navigation contexts.
- Navigation aliases and ordinals do not execute during value capture.
- Free text remains unrestricted by local suggestions.
- Enter confirms literal text unless the user explicitly selected a suggestion.
- A confirmed answer change discards every later answer.
- Returning without changing an answer preserves it and its committed suffix.
- A reached destination resolves immediately. There is no final review screen.
- Dismissal discards the session. A new invocation never resumes a partial flow.

## 2. Approaches and selected shell

| Approach | Benefit | Cost | Decision |
| --- | --- | --- | --- |
| Compact palette with one contextual line | Keeps the current question prominent and the field position stable | Previous answers require back navigation for inspection | Selected by the maintainer |
| Palette with a persistent answer summary | Exposes previous answers while entering the next value | Consumes height and moves the field as answers accumulate | Not selected |

Both alternatives used the same Datadog journey and preserved immediate completion.
The rejected summary was not a final confirmation screen or an editable form.

The selected shell has four regions:

1. A quiet context line, with Back when applicable and a visible Close command.
2. The current field label and text field at a stable position.
3. A single scrolling region for entries, choices, suggestions, or current feedback.
4. A compact footer with the current primary action and relevant keyboard hints.

The footer action is a real button in the future native interface, not decorative text.
It dispatches the same semantic action as Enter, so pointer users can confirm text.
There is no sidebar, persistent answer table, stepper, favorites section, or recent-history surface.
The palette never displays future questions from branches the user did not choose.

### Low-fidelity structure

```text
ROOT NAVIGATION
┌───────────────────────────────────────────────────────┐
│ URLchemy                                      Close   │
│ Open a flow                                           │
│ [ Search or type an alias                           ] │
│                                                       │
│   Datadog                                  dd    ⌘1   │
│                                                       │
│ ↑ ↓ Choose                         Open folder ↵       │
└───────────────────────────────────────────────────────┘

CLOSED SELECTION
┌───────────────────────────────────────────────────────┐
│ ‹ Back   Datadog / Open dashboard              Close   │
│ Dashboard                                             │
│ [ Filter dashboards                                 ] │
│                                                       │
│ ▸ HTTP Traffic                                        │
│   User Activity                                       │
│                                                       │
│ ↑ ↓ Choose                            Continue ↵      │
└───────────────────────────────────────────────────────┘

FREE TEXT WITH SUGGESTIONS
┌───────────────────────────────────────────────────────┐
│ ‹ Back   Datadog / HTTP Traffic                Close   │
│ Source application                                    │
│ [ acc|                                              ] │
│                                                       │
│   Accounts                  visible, not selected     │
│                                                       │
│ ↓ Choose suggestion                    Use “acc” ↵    │
└───────────────────────────────────────────────────────┘

FINAL INPUT
┌───────────────────────────────────────────────────────┐
│ ‹ Back   Datadog / HTTP Traffic                Close   │
│ Destination application                               │
│ [ billing|                                          ] │
│                                                       │
│ Opens the dashboard in your default browser.           │
│                                                       │
│                                         Open URL ↵    │
└───────────────────────────────────────────────────────┘
```

The selected row and caret markers above explain state, not literal decoration.
The final input example follows an explicit selection of Accounts at the previous step.
These sketches specify no rendered pixel values or verified GPUI behavior.

### Presentation constraints

- Keep one leading alignment spine for labels, fields, rows, errors, and empty states.
- Use platform typography, semantic theme colors, rem-based spacing, and component sizes.
- Distinguish keyboard focus, active row, explicit suggestion selection, hover, and validation.
- Use selected row surfaces rather than a leading-edge selection bar.
- Preserve icon and shortcut lanes when an entry lacks an emoji or alias.
- Show configured folder/flow emojis. Use one consistent framework icon family for fallback icons.
- Show option and suggestion labels, not hidden UUIDs or URL-encoded values.
- Keep the context line to one visual line. Elide older ancestors before the current object.
- Expose the full context through accessible text and a focus-accessible tooltip when truncation needs explanation.
- Let only the result body scroll. Keep the field, context, and primary action available.
- Fit the panel to the work area and larger text. Do not enlarge it to show empty result slots.
- Use medium control density initially. A list can show up to nine ordinary rows before scrolling when space permits.
- Propose a 40rem preferred panel width, a 32rem minimum target, and a 48rem maximum.
- Clamp width to the available work area with surrounding clearance. The minimum must never force the panel offscreen.
- Limit the result body to 24rem or the remaining work-area height, whichever is smaller.
- Shrink the body for short lists. Keep the panel's top anchor stable when its content height changes.
- Preserve focus rings, long labels, and readable error messages at smaller work-area sizes.
- Avoid simulated loading, success toasts, animations, or badges for immediate local transitions.

Exact size bounds and resolved alignment require the real components and native
work-area measurements in the next milestone. They are not validated by the HTML mockups.

## 3. State and action ownership

The core remains authoritative for the catalog, answers, history cursor, branch
continuations, validation, and URL resolution. The native adapter owns the window,
text editing component, focus, row highlight, filter query, scrolling, and external effects.

Use the production `Session` and `SessionSnapshot` instead of reconstructing a
launcher model inside GPUI. All activation paths converge on semantic core actions.
Clicks, keyboard confirmation, and footer buttons must not implement different transitions.

Keep navigation/filter text separate from captured input text:

- Browsing owns a temporary navigation query. It never becomes an answer.
- Closed selection owns a temporary option filter. It never becomes a captured value.
- An input field mirrors the core's current draft and explicit suggestion selection.
- Entering another folder or step clears its transient filter and row highlight policy.
- Back restores the core's committed answer rather than a UI-maintained answer cache.
- Back abandons an unconfirmed draft. Preserving answers does not mean preserving unfinished text at a later step.
- Dismissal and invocation clear all transient view state.

Use stable `RowId` values for selection and element identity.
Never store a filtered-list index as a selected option or suggestion identity.
Programmatic field restoration must not emit a user-edit event back into the engine.
Pass committed input text to the core without application trimming or normalization.
Do not silently flatten pasted line breaks before the core can reject control characters.

### Core integration gaps to plan, not silently work around

The current input snapshots expose the current step, draft, rows, cursor, and
answer count. They do not expose the selected branch's presentation context.
The context line therefore needs a narrow readonly presentation seam from the core.
The native view must not parse YAML again or duplicate answer/branch traversal logic.

Similarly, alias hints need readonly navigation presentation metadata.
Root direct aliases remain executable through `Session::exact_alias` even when
their nested target is not a visible root row. They need no duplicate navigation entry.

The UI also needs to distinguish an input whose confirmation reaches a leaf
from an input that continues. The footer must not guess this from answer count.
Plan a core-provided action hint derived from the current definition and draft.
If a selection has no active option yet, use neutral `Choose an option` feedback.

These are presentation/API requirements for later implementation, not core changes
made in this design milestone. Verify the selected GPUI Kit version before choosing
specific methods or assuming component selection defaults.

## 4. Keyboard, focus, and pointer policy

### Approved defaults

| Input | Browsing | Closed selection | Free-text input |
| --- | --- | --- | --- |
| Text entry | Update query, check exact alias, then filter | Filter configured choice labels | Update literal draft and filter suggestion labels |
| Up / Down | Move active row | Move active option | Explicitly select a visible suggestion, if available |
| Enter | Activate active entry | Confirm active option | Confirm explicit suggestion or literal draft |
| Command+1…9 | Activate numbered result entry | No navigation action | No navigation action |
| Command+[ | Back one navigation level | Return to the preceding step or launch origin | Return to the preceding step or launch origin |
| Escape | Dismiss launcher | Dismiss launcher | Dismiss launcher |
| Backspace / Left / Right | Native text editing only | Native text editing only | Native text editing only |
| Tab / Shift+Tab | Move among field and applicable command buttons | Same | Same |

The maintainer approved the defaults in this table.
Do not bind empty-field Backspace to Back. Do not use Escape to clear a query before dismissal.
Back stays visible and keyboard-reachable. At root, omit or disable it without misleading hover feedback.

This document does not select the global invocation binding.
It must not assume that Spotlight's or another launcher's shortcut is available.
The eventual configurable launch binding dispatches the same action as Enter.

### Focus and active rows

Invocation places focus in the root text field.
Entering a folder, flow, or new step places focus in its field without requiring a click.
Back to an input restores its committed text and selects that text for replacement.
Back to a selection restores the committed choice and scrolls it into view.

Browsing and closed selection initially highlight the first available row.
This is not answer confirmation. Enter or row activation confirms the target.
Up and Down stop at the result-list boundaries rather than wrapping.
An empty result list has no active row or Enter activation.
Modified arrows and native caret commands remain text-editing operations.

An input with suggestions initially has no explicitly selected suggestion.
Down selects the first suggestion. Without an explicit selection, Up selects the last suggestion.
Further arrows move within the list without wrapping.
Typing or pasting changes the literal draft and clears explicit selection.
Hover does not select a suggestion or change Enter's meaning.

Tab does not accept a suggestion or complete a value.
The text field is the initial task focus, followed by the primary footer action,
Back when applicable, and Close. Shift+Tab reverses that order.
Rows remain available through their composite keyboard and accessibility behavior,
not one additional Tab stop per result.
After Escape or Close, restore focus to the invoking application where the platform permits.
An outside click or deliberate application switch follows the user's chosen target instead.
Successful browser opening must not reactivate the previously focused application.
Transient IME windows and owned component surfaces must not trigger accidental dismissal.

### Pointer actions

- One click on a folder or flow activates it immediately.
- One click on a closed option selects and confirms it immediately.
- One click on a suggestion explicitly selects it but does not confirm the answer.
- The footer button confirms the selected suggestion or literal text.
- Back dispatches the same Back action as Command+[.
- Close dispatches Dismiss and closes the overlay.
- An outside click dismisses the overlay. It does not preserve a draft for later invocation.

A selected suggestion remains visually distinct from hover and from an unselected match.
The footer names the actual commitment, such as `Use Accounts` or `Use “acc”`.
The native text field retains the typed query during explicit suggestion selection.
Do not silently replace the draft with its stored value during arrow navigation.

### IME and event precedence

An input method editor (IME) can compose text before committing it.
Composition text must not activate aliases, rows, shortcuts, or URL completion.
An Enter that commits composition must not also confirm a flow answer.
Escape first cancels an active composition or topmost transient surface.
Only a subsequent unconsumed Escape dismisses the launcher.

For each input event, apply this order:

1. Let active text composition or the topmost transient component consume it.
2. Handle the focused command button's activation, when applicable.
3. Handle the current palette mode's applicable command.
4. Let ordinary text editing update committed field text.
5. Evaluate exact navigation aliases only for committed navigation text.

One event can advance at most one construction step or navigation level.
A completion event can additionally request one browser opening.
Never replay the characters that activated an alias into the next input field.
Ignore held-key repeats for activation, confirmation, dismissal, and Back.
Arrow movement and ordinary text editing can repeat normally.
Do not restore focus through a closing overlay during rapid invoke/dismiss sequences.

## 5. Filtering, aliases, and ordinals

Use the existing `CaseInsensitiveSubstring` label filter as the approved default.
It lowercases Unicode text, without accent removal, normalization, fuzzy ranking,
sorting, or remote suggestions. Preserve configuration order.
An empty query displays all rows for the current context.

On a changed navigation or option filter, retain the active row only if its ID
remains in the results. Otherwise highlight the first result or clear the highlight.
On a changed input draft, clear explicit suggestion selection even if its row still matches.

Exact aliases remain case-sensitive and follow the approved contexts.
Check the whole committed navigation query against aliases before applying label filtering.
Do not require an alias target to match its label filter.
Do not split a pasted string into an alias plus answers for later steps.
Automatic alias-conflict validation remains deferred.

Ordinals refer to the first nine fully visible navigation entries after filtering,
not their original configuration positions. Show each applicable shortcut on its row.
Recompute the displayed numbers when filtering or scrolling changes that visible set.
Never activate an offscreen entry through an ordinal whose hint is absent.
Use the current coherent visible-row IDs when dispatching, not an older filter or viewport index.
If presentation is between result sets, suppress activation until its mapping is coherent.
Reset ordinal mapping on every navigation context or query change.
The maintainer approved this viewport-numbering policy for the visible-entry shortcuts.

## 6. Validation, empty states, and external effects

| Condition | Surface and recovery |
| --- | --- |
| Valid empty catalog | `No flows configured`. Explain manual configuration and restart. Keep Close available |
| No navigation/choice matches | `No matches`. Keep the field editable. Enter activates nothing |
| No suggestion matches | Explain that literal text is still valid. Do not block confirmation for lack of a suggestion |
| Invalid confirmed text | Inline error beside the field. Preserve draft, cursor, committed history, and field focus |
| Destination resolution failure | Compact diagnostic state with Back as the recovery action. Do not emit a browser request |
| Startup configuration failure | Diagnostic surface instead of flow rows. Show problem, file, pointer, and available line/column |
| Browser request pending | Prevent duplicate submission. Show waiting feedback only while a real operation is pending |
| Browser request failed | Keep overlay open with Retry, Start over, and Close. Do not resurrect a completed session |

Input errors appear after a rejected confirmation, not on every intermediate keystroke.
Editing clears the stale error presentation. The next confirmation validates again.
Apply the approved empty/control rules without trimming accepted text.

For a destination error, the core is at `DestinationError`, not an editable input.
Back restores the preceding answer. It can take more than one Back to reach
the answer that needs correction. A direct destination failure returns to its launch origin.
Confirm is not a retry action at this state.
Focus Back for resolution recovery and Retry after an opening failure.
Their Enter activation follows the focused button, not the engine's Confirm action.
For startup diagnostics, focus the diagnostic region, with Close as the next command stop.

```text
INVALID INPUT                         RESOLUTION FAILURE
Source application                    Could not build the URL
[     |                    ]          A path segment cannot be a dot segment.
Enter a nonblank value.                Go back and edit an answer.
                         Continue ↵                   Back to answer
```

The startup diagnostic surface does not generate files or open an editor.
It explains that configuration corrections require restarting URLchemy.
Diagnostics and accessible error descriptions must not echo captured values or completed URLs.
Do not use a toast as the only explanation of a failure.

### Browser-opening boundary

On `Outcome::Resolved`, the engine has already cleared answers and entered Completed.
The native adapter retains only the resolved URL and an operation identity while
requesting default-browser opening. This is not a preview or confirmation screen.

On success, close the overlay and discard the temporary URL.
On failure, Retry repeats only that opening request. Start over invokes a fresh root.
Dismissal or a new invocation clears the pending/failure UI and invalidates old completion callbacks.
A stale completion must not close a newly invoked palette.
Do not log the URL or keep it as recent history.
Dismissal cannot undo a browser request that the OS already accepted.

```text
BROWSER OPENING FAILURE
Could not open the browser.
[ Retry ]  [ Start over ]  [ Close ]
```

The actual opener API and native error behavior require milestone 5 verification.
These sketches do not prove activation, IME, focus restoration, Spaces, or VoiceOver behavior.

## 7. Representative journeys and acceptance

1. **Fixed destination:** invoke root, enter Tools, activate Service status with its displayed ordinal. Resolve and open immediately, without an answer screen.
2. **Nested navigation:** invoke root, type `dd`, and enter Datadog without Enter. Type `dashboards` to start its flow. No alias text enters the next field.
3. **Direct access:** invoke root and type `open-dd`. Start the same flow without visiting its folder.
4. **Branching:** select HTTP Traffic, type `acc`, explicitly choose Accounts, confirm it, and type `billing`. The last confirmation resolves the agreed traffic URL.
5. **Literal text:** at Source application, type `acc` and confirm without selecting Accounts. Capture literal `acc`, not `accounts`.
6. **Alternative branch:** choose User Activity and confirm `alice`. Resolve the agreed user-activity URL without the other branch's fields.
7. **Correction:** at Destination application, type `billing` without confirming, then return to Source application. Back abandons the unconfirmed `billing` draft. Confirming `accounts` again returns to an empty destination draft, because that destination had no committed answer.
8. **Branch correction:** return to Dashboard and choose another option. Drop all answers from the former continuation, even if captured strings match.
9. **Common continuation:** choose Staging in the common fixture. Changing it to Production later clears the common continuation's later answers too.
10. **Cancellation:** dismiss while typing or reviewing a prior answer. Invoke again and see root with no previous query or answers.
11. **Mixed input:** enter a folder by ordinal. Click a flow and a closed option. Type a value, click a suggestion, and use the footer to confirm.
12. **Input composition:** commit IME text with Enter without activating an alias or confirming an answer on that same event.
13. **Opening failure:** complete a flow, observe a simulated native opener failure, then retry or start over without resuming old answers.

For committed-suffix review, use the core's four-input test journey `X → Y → Z → W`.
Confirm X, Y, and Z, then stop at W. Return to Y and confirm the same value to preserve Z.
Changing Y removes Z and requires it again. The unfinished W draft is not a committed answer.

Future tests must exercise the production view and the real core together.
Use pure tests for the presentation policy where possible, GPUI integration tests
for focus and input routing, and separately authorized native tests for platform behavior.
Read `docs/testing-probe.md` before designing interaction or screenshot tests.

## 8. Configuration seam and next milestone

The maintainer subsequently approved the
[shortcut and framework amendment](2026-11-15-shortcuts-framework-design.md).
It selects GPUI Kit `=0.9.1` and defines the optional YAML settings and validation policy.
The configuration-only implementation does not add GPUI dependencies or register native shortcuts.
The paragraphs below record the prerequisites identified during interaction design.

The approved YAML schema has no field for global or launch keybindings.
The product still requires a configurable launch binding and leaves invocation configuration open.
Do not invent an undocumented root key or load a second configuration file.

Before native implementation, agree on a small, explicit configuration amendment
for the global shortcut and launch binding in the same file.
Specify its grammar against the selected framework version, add invalid/conflict
diagnostics as appropriate, and preserve existing fixtures without reinterpretation.
Shortcut conflict checks must not introduce the deferred automatic alias-conflict validation.
This document defines approved interaction defaults, not that serialization amendment.
The missing settings seam blocks a complete configurable native implementation,
not the present compact-palette layout decision.

Other milestone 5 prerequisites are a compatible GPUI Kit revision, verified component
capabilities, the narrow readonly context/action-hint API, and an approved execution plan.
Actual native launch and OS permission requests still require explicit permission.

## 9. Review and approval boundary

Confirmed by the maintainer:

- Compact palette instead of a persistent answer summary.
- Command+1 through Command+9 for navigation ordinals, without consuming ordinary digits.
- No navigation ordinal execution during value capture.

Additional interaction rules approved on 2026-11-15:

- Escape dismisses, Command+[ returns, and Backspace only edits text.
- The focus, active-row, suggestion-click, filtering, and ordinal mapping rules above.
- Inline errors, direct resolution-error recovery, and the browser-failure retry surface.
- One primary action button per current decision, matching Enter without a final review screen.

The author reviewed this draft against the GPUI Kit Design Guides and Coding Guides.
The mockups establish structure only. Contrast, measurements, component behavior,
large text, native focus, and accessibility still need actual rendering and testing.
No independent review or subagent participated. No application code changed.
Approval of this document alone does not authorize native implementation or publication.
