# Domain and execution model

Status: approved by the maintainer on 2026-11-14 for roadmap milestone 1.
Approval covers the three-representation approach and the detailed model below.

## 1. Purpose and scope

Define the URLchemy engine independently of YAML nesting, GPUI, and macOS APIs.
The maintainer must be able to trace a configured branch through memory and
session transitions to a resolved URL.

Inputs are [the product definition](../../product.md) and
[roadmap milestone 1](../../../ROADMAP.md). Their contents match the
`docs/product-definition` branch at the time of this design.

This document preserves the agreed product behavior:

- One local configuration loads at startup, with no partial publication.
- Navigation folders organize flows but never contribute URL components.
- A flow owns one fixed base and a finite construction tree.
- Steps collect mandatory values or select continuations.
- Destinations contain complete templates relative to the fixed base.
- Captured text never becomes URL syntax or another template.
- An actual answer change discards every later answer.
- Completion is immediate, with no review screen.
- Dismissal discards the session. Every invocation starts at the root.

Non-goals are application implementation, Cargo setup, dependency selection,
the complete YAML contract, detailed URL syntax, and UI or native design.
Milestone 2 must settle the configuration and URL policies identified below.
No external data source, flow invocation, optional step, or shared-definition
reference enters this design.

## 2. Approaches and decision

| Approach | Benefit | Cost | Decision |
| --- | --- | --- | --- |
| Use one recursive type for files and execution | Fewer types and conversions | Runtime code must repeatedly handle invalid definitions and file-format details | Not selected |
| Separate file data, validated definitions, and session state | Validation establishes engine guarantees. Mutable answers cannot change definitions | Requires an explicit conversion and source mapping | Selected by the maintainer |
| Compile directly into indexed instruction tables | Cheap identifier lookup and compact execution records | Adds table consistency rules and obscures a small tree | Not selected for the initial model |

The selected domain remains an owned tree. Internal identifiers locate nodes
without turning their ownership into a graph. This is not an instruction-table
interpreter or a design for reusable subflows.

## 3. Boundaries and ownership

```text
local YAML bytes
    | parse / deserialize
    v
RawConfig + source locations
    | validate / compile
    +-----------------------> configuration diagnostics, no catalog
    v
Catalog (immutable, owned by the application)
    | borrowed by engine operations
    v
Session (mutable, owned by the launcher controller)
    | apply semantic actions
    +-----------------------> next state / action diagnostic
    +-----------------------> ResolvedUrl + completed session
                                     |
                              native adapter, outside engine
                              browser request / overlay lifecycle
```

The application owns the catalog for its entire startup configuration lifetime.
Each engine call borrows the catalog rather than copying it into the session.
The session owns answers, draft text, and navigation history.
It stores identifiers, not Rust references into the tree.
This avoids session lifetime parameters and self-referential structures.

This model needs no `Rc`, `Arc`, locks, asynchronous runtime, or unsafe code.
Later UI ownership may require a shared catalog handle outside the core.
That choice does not change the engine contract.

All Rust declarations in this document are illustrative type sketches.
They are not compiled code, dependency APIs, or a committed public API.
Fields that establish invariants remain private in the eventual implementation.

## 4. File representation: data without execution guarantees

The file representation preserves names and author-written strings.
It does not claim that a base URL, continuation, or capture reference is valid.
The YAML adapter also retains a configuration path and available source span.

```rust
struct RawConfig {
    navigation: Vec<Located<RawEntry>>,
}

enum RawEntry {
    Folder(RawFolder),
    Flow(RawFlow),
}

struct RawFolder {
    label: String,
    emoji: Option<String>,
    alias: Option<String>,
    children: Vec<Located<RawEntry>>,
}

struct RawFlow {
    id: String,
    label: String,
    emoji: Option<String>,
    alias: Option<String>,
    direct_alias: Option<String>,
    base_url: String,
    start: Box<Located<RawStep>>,
}

enum RawStep {
    Input(RawInput),
    Select(RawSelect),
    Destination { template: String },
}

struct RawInput {
    label: String,
    capture: String,
    suggestions: Vec<RawSuggestion>,
    next: Option<Box<Located<RawStep>>>,
}

struct RawSelect {
    label: String,
    capture: Option<String>,
    options: Vec<Located<RawChoice>>,
    next: Option<Box<Located<RawStep>>>,
}

struct RawChoice {
    label: String,
    value: Option<String>,
    next: Option<Box<Located<RawStep>>>,
}

struct RawSuggestion { label: String, value: String }
struct Located<T> { value: T, origin: Origin }
struct Origin { config_path: String, span: Option<SourceSpan> }
struct SourceSpan { start_byte: usize, end_byte: usize }
```

`RawStep` represents the conceptual alternatives after structural parsing.
Malformed alternatives, unknown fields, and missing required fields can fail
before a complete `RawConfig` exists. They still produce located diagnostics.
The adapter maps YAML `baseURL` to Rust `base_url`.
These sketches do not select a YAML library or serialization attributes.

The proposed common-continuation representation uses `RawSelect.next`.
Branch-specific continuations use each `RawChoice.next`, as in the accepted
example. Validation rejects mixed common and branch-specific continuations.
This proposal does not finalize their YAML spelling in milestone 2.

File-level YAML aliases or anchors must not bypass tree validation.
Milestone 2 must define their acceptance and parser resource limits.
They do not introduce domain references, cycles, or shared nodes.

## 5. Validated domain: executable definitions

```rust
struct EntryId(u32);
struct FlowId(u32);
struct StepId(u32);
struct ChoiceId(u32);
struct SuggestionId(u32);
struct FlowKey(String);       // Author-written flow id.
struct CaptureName(String);   // Validated parameter name.
struct CapturedValue(String); // Plain text, never pre-encoded.

struct Catalog {
    roots: Vec<Entry>,
    flows: Vec<Flow>,
    origins: OriginIndex,
}

struct Entry { id: EntryId, kind: EntryKind }
enum EntryKind {
    Folder { presentation: Presentation, children: Vec<Entry> },
    Flow { presentation: Presentation, flow: FlowId },
}
struct Presentation {
    label: String,
    emoji: Option<String>,
    alias: Option<String>,
}

struct Flow {
    id: FlowId,
    key: FlowKey,
    direct_alias: Option<String>,
    base: FixedBase,
    start: Step,
}

struct Step { id: StepId, kind: StepKind }
enum StepKind {
    Input {
        label: String,
        capture: CaptureName,
        suggestions: Vec<Suggestion>,
        next: Box<Step>,
    },
    Select {
        label: String,
        selection: Selection,
    },
    Destination(Destination),
}

enum Selection {
    Common { options: NonEmpty<Choice>, next: Box<Step> },
    Branched { options: NonEmpty<BranchChoice> },
}
struct Choice { id: ChoiceId, label: String, capture: Option<Capture> }
struct BranchChoice { choice: Choice, next: Box<Step> }
struct Capture { name: CaptureName, value: CapturedValue }
struct Suggestion { id: SuggestionId, label: String, value: CapturedValue }

struct Destination { template: CompiledTemplate }
```

`NonEmpty<T>`, `FixedBase`, `CompiledTemplate`, and `OriginIndex` are conceptual
private types. They do not imply new dependencies.
Recursive edges use `Box` because Rust must know each struct's finite size.
Vectors already store their elements outside the containing struct.

Each navigation entry owns its presentation and appears once in the tree.
Each flow entry refers to one catalog-owned flow.
This separates navigation from execution, without allowing one flow to invoke
another. A direct alias locates the same flow through its original entry.

`Selection` makes invalid continuation combinations unrepresentable.
A common continuation exists once, not once per option.
For a capturing selection, validation produces the same capture name on every
choice. A non-capturing selection only chooses a continuation.
An input without suggestions is ordinary free text.
An input with suggestions still accepts literal text.

Internal identifiers are unique within one catalog and never reused during its
lifetime. Constructors are private. They are not serialized or persisted.
The user-written `FlowKey` remains separate and must be unique across flows.
An implementation can locate step identifiers by walking the small owned tree.
An optional immutable lookup index is an optimization, not a second definition.

### Validation invariants

Compilation is atomic: it returns either the complete catalog or diagnostics.
It never publishes valid flows from an otherwise invalid configuration.

```rust
struct ConfigDiagnostic {
    code: ConfigErrorCode,
    origin: Origin,
    message: String,
}
// compile(RawConfig) -> Result<Catalog, Vec<ConfigDiagnostic>>
```

The adapter retains origins for fields as well as nodes.
`OriginIndex` relates compiled identifiers and template tokens to those origins.
Diagnostics identify a configuration path and the available line/column location
derived from the source span. They must not include entered session values.

1. Navigation and construction definitions are finite owned trees.
2. Each flow has one valid fixed base. Children cannot replace its authority or base prefix.
3. Each input has one named capture and exactly one continuation.
4. Each selection has at least one option and exactly one continuation mode.
5. Every branch option has one continuation. A common selection has one shared continuation.
6. A capturing selection supplies a value on every option.
7. A destination is a leaf. It needs no continuation and collects no answer.
8. Every reachable destination references only captures available on its path.
9. A capture name occurs at most once on any single traversal.
10. Every confirmed input is mandatory. There is no skip transition.
11. Templates contain only supported structural syntax and value substitutions.

Rules 9 and unique flow keys are technical proposals, not previous product
decisions. Rejecting duplicate captures avoids implicit replacement semantics.
Sibling branches may reuse a capture name because they cannot execute together.
Validation permits unused captures: a choice may select a destination without needing
its captured value in that destination's template.

Validation walks each flow with a set of available capture names.
An input adds its name before visiting its continuation.
A selection adds its capture name before visiting each continuation.
Sibling branches receive independent sets. Their captures never merge.
At a destination, every placeholder must exist in that path's set.
Duplicate capture definitions on that path produce a diagnostic.

Alias conflicts remain a documented configuration restriction.
This model does not add automatic duplicate or prefix-conflict validation.
The deterministic alias activation contract applies to configurations that obey
the product's restriction.

## 6. Templates and URL resolution boundary

The validated template distinguishes URL structure from captured values.
Conceptually it contains path segments and ordered query pairs.
Each segment or query value contains literal tokens and capture tokens.
Query names are structural literals, not captured values in the initial model.
Capture tokens retain their insertion context.

```text
FixedBase: https://app.datadog.com/company
Path:      [ [Capture(dashboard_id)] ]
Query:     [ ("from", [Capture(source_app)]),
             ("to",   [Capture(destination_app)]) ]
```

At a leaf, the resolver derives a capture map from the active committed path.
It substitutes values as data, encodes each value for its context, and combines
the template with the fixed base. It returns `ResolvedUrl` or a typed diagnostic.
It never performs recursive substitution or expands user-written fragments.

The accepted example's leading `/` means a path below the flow's base prefix.
It must not remove `/company` through ordinary absolute-path URL joining.
All destinations preserve the base authority and prefix.

Milestone 2 must specify the exact joining and encoding algorithms.
It must also settle base queries/fragments, trailing slashes, empty text,
whitespace, literal escapes, malformed placeholders, and dot-segment values.
Relative resolution must never let values such as `..` escape the base prefix.
This document does not approve a generic URL-library `join` call.
It assumes no user-input trimming, normalization, or rejection policy.

Required means that the user must confirm an answer.
Whether an empty confirmed string is valid remains a milestone 2 policy.
The engine does not accept that policy implicitly through a default string value.

## 7. Mutable session and semantic actions

```rust
enum Session {
    Inactive,
    Browsing(NavigationState),
    Running(FlowRun),
    Completed, // No answers retained. Await adapter lifecycle handling.
}

struct NavigationState {
    folders: Vec<EntryId>, // Root is the empty path.
}

struct FlowRun {
    flow: FlowId,
    origin: NavigationState,
    answers: Vec<AnswerRecord>,
    cursor: usize,
    draft: Draft,
}
struct AnswerRecord { step: StepId, answer: Answer }
enum Answer {
    Input(CapturedValue),
    Choice(ChoiceId),
}
enum Draft {
    Input { text: String, selected_suggestion: Option<SuggestionId> },
    Selection { selected: Option<ChoiceId> },
    DestinationError { step: StepId },
}

enum Action {
    Invoke,
    EnterFolder(EntryId),
    StartFlow(FlowId),
    SetInputText(String),
    SelectSuggestion(Option<SuggestionId>),
    SelectChoice(ChoiceId),
    Confirm,
    Back,
    Dismiss,
}
```

The input adapter translates clicks, aliases, ordinals, and keybindings into
semantic actions. The engine never receives macOS key codes.
Visible-row filtering and binding design remain later concerns.
Navigation aliases and ordinals are not interpreted while a flow collects values.

Navigation actions must target an entry in the current folder.
`StartFlow` accepts a visible flow or a direct-access target from the root.
The engine rejects inaccessible or foreign identifiers without changing state.
A direct-access start records the root as its navigation origin.
Selection and suggestion identifiers must belong to the current step.

Selecting a suggestion explicitly changes the draft selection.
Editing text clears that explicit selection.
`Confirm` uses an explicitly selected suggestion's value. Otherwise it uses literal text.
Filtering suggestions alone never selects one.
Selecting a closed-list choice changes its draft selection.
A click that activates an option translates to selection followed by confirmation.

`answers` stores one committed record for each visited input or selection.
Destinations never occupy history slots.
`cursor` is an answer position, not a URL-component position.
It ranges from zero through `answers.len()`.
At the end, it identifies the next unanswered step or a destination error.
Before the end, it identifies an earlier committed step under review.
The current step follows from replaying the records before the cursor.
No separately mutable capture map or branch pointer can disagree with that replay.

Only committed answers supply captured values.
Unconfirmed draft edits do not invalidate history.
`Back` abandons the current draft and restores the preceding committed answer.
Returning without confirmation therefore changes no committed answer.

### Transition contract

| State and action | Result |
| --- | --- |
| Any state, `Invoke` | Discard the old session and browse root |
| Browsing, `EnterFolder` | Append the visible folder to the navigation path |
| Browsing, `Back` | Remove the last folder, or remain at root |
| Browsing, `StartFlow` | Create an empty run and visit its start node |
| Running, draft action | Update the current draft only |
| Running, `Confirm` with invalid or missing answer | Keep history and current step. Return an action diagnostic |
| Running, `Confirm` with a valid answer | Commit or compare the answer, then visit its continuation |
| Running, `Back` with cursor above zero | Decrement cursor and restore that step's committed answer |
| Running, `Back` at cursor zero | Return to the navigation origin and abandon this run |
| Any state, `Dismiss` | Become inactive and discard all session data |
| Visit a destination | Resolve immediately. Return the URL and become completed |

Actions without an applicable transition return an action diagnostic and preserve
the session. The destination row describes successful resolution.
Resolution failures follow the explicit error transition in section 8.

Returning from the first flow step to its navigation origin is a proposed engine
boundary. The later interaction design chooses its binding, not different
capture semantics. There is no resume after leaving the flow.

### Confirmation and invalidation

For a new answer, append its record and advance.
For an earlier answer, compare its effective answer with the stored record:

1. If both answers match, preserve every later record and advance one position.
2. If it changed, remove every later record, replace this record, and advance.
3. Restore the next recorded answer if one remains. Otherwise create a fresh draft.
4. If the continuation is a destination, resolve it immediately.

For text input, equality compares captured text, not suggestion presentation.
A literal and an explicitly selected suggestion with identical values are equal.
For a closed selection, equality compares `ChoiceId`, not the captured value.
Different choices can capture identical text but lead to different continuations.

An unchanged answer never automatically skips the remaining review positions.
The user advances through them with confirmation, retaining their answers.
The already entered suffix remains stored but cannot influence URL resolution
until traversal reaches its destination again.

For `X → Y → Z`, changing Y removes Z even when Y selects the same branch.
Returning to Y and confirming the same answer preserves Z.
Editing a draft and then returning without confirming preserves the old Y and Z.
Once the user confirms a final answer, immediate completion closes the journey.
There is no completed-flow editor in the next invocation.

## 8. Engine results and external effects

```rust
struct EngineUpdate {
    outcome: Outcome,
}
enum Outcome {
    StateChanged,
    Rejected(ActionDiagnostic),
    Resolved(ResolvedUrl),
    ResolutionFailed(ResolutionDiagnostic),
}

// Conceptual boundary, not a selected public signature:
// apply(&Catalog, &mut Session, Action) -> EngineUpdate
```

The engine mutates only session state and returns data.
Configuration errors belong to startup loading, not action processing.
Stale identifiers, invalid actions, and resolution failures use typed diagnostics.
They do not panic or partially commit an invalid transition.

A rejected confirmation retains the prior committed history and draft.
If destination resolution fails, the engine retains the valid committed answers
and returns no URL. It does not enter `Completed`.
The cursor follows the final answer, and the draft becomes `DestinationError`.
A direct destination failure retains an empty run with that error position.
`Back` restores the final answer, or leaves an empty run for its navigation origin.
`Confirm` at an error position rejects the action without retrying native effects.

A successful resolution atomically clears answers and enters `Completed`.
Later actions cannot emit the URL again, except a new invocation and traversal.
The native adapter requests browser opening using the returned URL.
After a successful opening request, it closes the overlay.
The adapter handles opening failures explicitly. The engine does not assume
that producing a URL proves browser opening or close the overlay itself.
Failure presentation and retry controls remain native-integration work.
They must not restore a partially completed flow on the next invocation.

## 9. Accepted YAML example through memory

### Deserialized configuration

The accepted YAML becomes the following conceptual file data.
This illustration omits origins only for readability.

```text
RawConfig.navigation
└─ RawFolder(label="Datadog", alias="dd")
   └─ RawFlow(id="open-dashboard", alias="dashboards", direct_alias="open-dd")
      base_url = "https://app.datadog.com/company"
      start = RawSelect(label="Dashboard", capture="dashboard_id", next=None)
      ├─ RawChoice(label="HTTP Traffic", value="8c21-http")
      │  next = RawInput(capture="source_app")
      │         suggestions = [("Accounts", "accounts"), ("Billing", "billing")]
      │         next = RawInput(capture="destination_app")
      │                next = RawDestination
      │                template = "/{dashboard_id}?from={source_app}&to={destination_app}"
      └─ RawChoice(label="User Activity", value="9d32-users")
         next = RawInput(capture="user_id")
                next = RawDestination
                template = "/{dashboard_id}?userID={user_id}"
```

### Validated catalog

Illustrative identifiers describe this catalog only. They are not file syntax.

```text
Catalog.roots
└─ Entry E0: Folder("Datadog", alias="dd")
   └─ Entry E1: Flow("Open dashboard", alias="dashboards", flow=F0)

Catalog.flows
└─ F0: key="open-dashboard", direct_alias="open-dd"
   base=FixedBase("https://app.datadog.com/company")
   S0: Select::Branched("Dashboard")
   ├─ C0: "HTTP Traffic", capture=(dashboard_id, "8c21-http")
   │  S1: Input(source_app, suggestions=[G0: accounts, G1: billing])
   │  └─ S2: Input(destination_app, suggestions=[])
   │     └─ S3: Destination(path=[dashboard_id], query=[from:source_app, to:destination_app])
   └─ C1: "User Activity", capture=(dashboard_id, "9d32-users")
      S4: Input(user_id, suggestions=[])
      └─ S5: Destination(path=[dashboard_id], query=[userID:user_id])
```

At S3 the available names are `dashboard_id`, `source_app`, and `destination_app`.
At S5 they are `dashboard_id` and `user_id`.
Referencing `source_app` at S5 would reject the entire configuration.
The validator cannot borrow captures from the other branch.

### HTTP Traffic traversal

| Action | Session position and committed history |
| --- | --- |
| `Invoke` | Browsing root, no answers |
| `EnterFolder(E0)` | Browsing Datadog |
| `StartFlow(F0)` | Running at S0, history `[]`, origin Datadog |
| `SelectChoice(C0)`, `Confirm` | At S1, history `[S0: Choice(C0)]` |
| `SetInputText("accounts")`, `Confirm` | At S2, history `[S0: Choice(C0), S1: Input("accounts")]` |
| `SetInputText("billing")`, `Confirm` | Visit S3 and resolve immediately |

The last confirmation temporarily supplies the complete active capture map:

```text
dashboard_id    = "8c21-http"
source_app      = "accounts"
destination_app = "billing"
```

S3 supplies structure, while these three strings supply only values.
The resolver preserves `/company` and returns:

```text
https://app.datadog.com/company/8c21-http?from=accounts&to=billing
```

The session becomes `Completed` without retaining answers.
The native adapter receives that URL, requests browser opening, and closes the
overlay after a successful opening request.

### Correction before completion

At S2, before confirming its final answer, `Back` restores S1 with `accounts`.
Confirming `accounts` again preserves the history and returns to S2.
Changing S1 to `billing` replaces that answer and clears its later records.
S2 requires confirmation again. Its previous unconfirmed draft is not an answer.

From S2, two `Back` actions restore S0 with C0 selected.
Choosing C1 replaces the S0 record and removes the S1 record.
The next step is S4 with a fresh draft. No source-application answer survives.
Confirming `alice` supplies:

```text
dashboard_id = "9d32-users"
user_id      = "alice"
```

S5 then resolves immediately to:

```text
https://app.datadog.com/company/9d32-users?userID=alice
```

An independent invocation can also start F0 through its root direct alias.
It produces the same execution, with root as its navigation origin.

### Context-specific encoding

With C0, `source_app="accounts & billing"`, and `destination_app="billing"`,
the source value cannot introduce another query pair.
Its ampersand must become `%26`. Its space representation depends on the
query encoding policy selected in milestone 2.
Neither the input label nor a suggestion label appears in the URL.

## 10. Other required shapes

**Fixed destination:** `Flow.start` is a destination.
`StartFlow` resolves it immediately without creating an answer record.

**Linear flow:** an input points to another input or a destination.
The engine uses the same history rules without selection records.

**Common continuation:** a selection owns two or more choices and one `next`.
For example, C2 captures `environment="staging"`, while C3 captures
`environment="production"`. Both continue to the same application input.
Changing C2 to C3 discards every later answer despite the unchanged continuation.

**Terminal option:** a branch choice points directly to a destination.
Confirming the choice resolves it immediately, without another Enter.

**Capture order differs from URL order:** input `app-section` precedes
`app-name`, but the destination uses
`/engineering/application/{app-name}/{app-section}`.
The history follows construction order. Named substitution follows template order.

## 11. Proposed verification targets

These are future tests, not execution evidence:

- Deserialize the accepted example and compile the expected branch structure.
- Resolve both exact Datadog URLs shown above.
- Resolve fixed, linear, terminal-option, and common-continuation definitions.
- Keep labels separate from values and suggestions separate from restrictions.
- Confirm literal text unless the user explicitly selects a suggestion.
- Preserve later answers after unchanged confirmation.
- Invalidate every later answer after any effective answer change.
- Distinguish choices with identical values and different continuations.
- Reject missing continuations, mixed modes, empty selections, and missing values.
- Reject duplicate captures on a path and unavailable destination references.
- Allow repeated capture names in separate branches.
- Reject an invalid catalog atomically, with source locations.
- Reject stale or inaccessible identifiers without state mutation.
- Reset folders, drafts, and answers on dismissal and every invocation.
- Emit a resolved URL once per successful completion, without browser effects.
- Verify encoding, prefix preservation, Unicode, and hostile values after milestone 2.

## 12. Approval and next boundary

The maintainer approved these technical decisions on 2026-11-14:

1. An owned validated tree with private catalog-local identifiers.
2. Mutually exclusive common and branch-specific continuations.
3. Unique flow keys and no repeated capture name on a traversal.
4. Committed history plus a review cursor, with effective-answer comparison.
5. Returning from the first step abandons the run and restores its navigation origin.
6. Engine completion returns a URL and clears answers, without native effects.

No contradiction with the agreed product behavior emerged during this design.
Detailed YAML, URL policies, and configuration setup remain milestone 2 work.
Approval of this document does not authorize scaffolding or implementation.

Review method: in-session self-review against the product and roadmap.
This review used no independent reviewer or subagent.
