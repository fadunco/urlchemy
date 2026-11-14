# URLchemy — Product Definition

Status: MVP product scope and acceptance criteria agreed. This document records
functional decisions; it is not an approved implementation plan or a finalized
configuration schema. Remaining design questions are explicit below.

## Purpose and first user

URLchemy is an open-source, macOS-native, keyboard-driven URL launcher and
builder, inspired by Spotlight and Raycast interaction patterns.

It replaces repetitive navigation through web application interfaces with local,
configurable URL construction. The initial target user is the maintainer, a
technical user who edits their own configuration.

Representative use cases:

- IDP: open an application's jobs, databases, dependencies, or other sections
  without navigating multiple menus and application listings.
- Jira: open a known ticket by entering its identifier, or select a dashboard.
- Datadog: select a dashboard and supply its particular filters, such as source
  and destination applications or a user identifier.

## Principles and boundaries

- macOS-only initially; fast, lightweight, keyboard-first interaction.
- Local configuration only. URLchemy does not fetch options, suggestions, or
  other construction data from external HTTP services or APIs.
- Opening the completed URL in the default browser is the intended outcome;
  local-only construction does not mean the destination must be offline.
- No cloud service, remote account, or synchronization requirement.
- Configuration describes arbitrary services; IDP, Jira, dashboards, and
  applications are examples, not built-in domain concepts.
- Initial technical direction is Rust + GPUI + GPUI Kit. Product definition does
  not authorize application scaffolding or implementation.

## Domain model

### Folders

Folders organize access to flows. They may contain both folders and flow entries,
allowing multilevel hierarchies. Root entries appear when the launcher is invoked.

Folders do not capture parameters or contribute URL fragments, and do not impose
a shared base URL on their contents.

Folders and flows support a configurable emoji displayed alongside their label.
Emoji support is part of the MVP and does not apply to parameters or input steps.

For example, IDP may contain Applications, Routing, and Machine Learning folders,
each containing relevant flows or further folders.

### Flows

A flow defines a fixed `baseURL` and a finite tree of construction steps.

- The base may include a path prefix, such as
  `https://app.datadog.com/company`.
- All destinations within a flow share its base; branches cannot change domains.
- Configuration defines the possible step sequences. User selections determine
  which branch is traversed.
- There is no prescribed number of conceptual layers.
- Flows do not invoke other flows. Cycles and shared-definition references are
  not part of the initial scope.

The selected model replaces the earlier, provisional fixed-sequence-only model.
Independent linear flows organized in folders remain valid instances of it.
Branching was selected to support choices that change subsequent required inputs,
without introducing cross-flow invocation and its value-sharing contracts.

### Steps and values

Steps may collect:

- Free text.
- A selection from a closed, locally configured list.
- Free text with locally configured suggestions; suggestions do not restrict the
  values that may be entered.

A selection may continue along a common path or select a different continuation.
It may also capture a value, such as a dashboard identifier.

Options distinguish their visible label from the value used in a URL. For
example, “HTTP Traffic” may insert a UUID. The same distinction is available for
suggestions.

Values are identified by name. The configured capture order is independent of
their positions in the URL. A flow may request `app-section` before `app-name`
even when its destination path is:

```text
/engineering/application/{app-name}/{app-section}
```

Every input requested along the chosen branch is mandatory in the MVP. Optional
or skippable inputs are deferred.

### Destinations

A destination is a leaf of the construction tree. Each leaf declares its complete
path-and-query template relative to the flow's fixed base.

Templates may insert captured values into both path components and query
parameter values. Steps collect values and select a destination; they do not
incrementally concatenate URL fragments.

Captured values are plain text, not URL syntax or template fragments. URLchemy
encodes them for their insertion context (path component or query value), so
reserved characters in a value do not change the URL structure. For example,
`&` inside a query value is encoded as `%26`, rather than introducing another
parameter. Users do not need to pre-encode their input.

Illustrative destination, not configuration syntax:

```text
/{dashboard-id}?param_1={p1-value}&param_2={p2-value}
```

Reaching a destination with all required values captured opens the completed URL
immediately in the default browser, with no extra confirmation or review screen.
For a final input, this happens when its value is confirmed using Enter or another
configurable launch binding. Flows without inputs are allowed: activating a flow
that leads directly to a destination, or choosing a terminal option, opens the
URL immediately without an additional confirmation.
After opening the URL, the launcher overlay closes automatically. The next
invocation starts at the root, as with every invocation.

## Navigation and interaction

- A global keyboard shortcut invokes a lightweight overlay above the current app.
- Clicks, arrow keys with Enter, ordinal number shortcuts, and aliases can be
  mixed within the same navigation session.
- Folder aliases operate within the current level.
- Flow direct-access aliases are available from the root, bypassing folders.
- An exact alias match activates immediately, without Enter, leaving the next
  step ready for input.
- Aliases in the same context must not duplicate one another or be prefixes of
  one another. Initially this is a documented configuration restriction;
  automatic conflict validation is deferred.
- During value entry, navigation aliases and ordinal shortcuts are not
  interpreted as navigation commands.
- Typing in a free-text field filters its suggestions. Enter confirms literal
  text unless the user explicitly selected a suggestion using arrows or a click.
- Users can return to earlier steps and correct their answers. Changing any
  answer discards all subsequent answers, even if the branch stays the same;
  changing a branch decision likewise discards the old continuation's answers.
  Returning without modifying an answer preserves it and subsequent answers.
  Construction has a configured direction: after changing Y in X → Y → Z,
  Z must be completed again. This direction is independent of URL value order.
- Every invocation starts at the root. Dismissing the launcher abandons the
  current traversal and its answers; it never resumes a partially completed
  operation. This is intentional product behavior, not just an MVP limitation:
  stale intermediate state is confusing when returning later.

Fine interaction details will be refined through development and hands-on use.

## Configuration lifecycle

- One local YAML file contains the initial configuration: folders, flows,
  aliases, steps, options, and destination templates.
- The application reads it on startup. Applying edits requires restarting the
  application in the MVP.
- If configuration loading or validation fails, no flows are loaded partially.
  The launcher shows a diagnostic identifying the problem and its location in
  the configuration. It does not modify the user's file. This covers the
  validation implemented for the MVP; automatic alias-conflict validation
  remains deferred.
- Minimum MVP validation rejects invalid YAML, incomplete configuration
  structure, invalid `baseURL` values, steps or options without a valid
  continuation, and destination templates referencing values not captured on
  their traversal. A terminal destination is a valid endpoint and does not
  require a continuation.
- Multiple configuration files and live reload are deferred.
- The example below is accepted as a satisfactory first representation of the
  domain model. It is not yet a complete, implementation-ready schema.
- Splitting configuration into smaller files, possibly one per flow, may improve
  readability later; the MVP still uses one file.
- The configuration file location has not yet been defined.

### Accepted domain example

This example shows one flow with two branches requiring different inputs. Labels
are distinct from captured values, and each leaf supplies its own template.
The key names illustrate the model rather than establish a finalized schema.

```yaml
navigation:
  - folder:
      label: Datadog
      alias: dd
      children:
        - flow:
            id: open-dashboard
            label: Open dashboard
            alias: dashboards
            direct_alias: open-dd
            baseURL: https://app.datadog.com/company
            start:
              select:
                label: Dashboard
                capture: dashboard_id
                options:
                  - label: HTTP Traffic
                    value: 8c21-http
                    next:
                      input:
                        label: Source application
                        capture: source_app
                        suggestions:
                          - label: Accounts
                            value: accounts
                          - label: Billing
                            value: billing
                        next:
                          input:
                            label: Destination application
                            capture: destination_app
                            next:
                              destination:
                                template: "/{dashboard_id}?from={source_app}&to={destination_app}"
                  - label: User Activity
                    value: 9d32-users
                    next:
                      input:
                        label: User identifier
                        capture: user_id
                        next:
                          destination:
                            template: "/{dashboard_id}?userID={user_id}"
```

Choosing HTTP Traffic, then entering `accounts` and `billing`, resolves to:

```text
https://app.datadog.com/company/8c21-http?from=accounts&to=billing
```

Choosing User Activity and entering `alice` resolves to:

```text
https://app.datadog.com/company/9d32-users?userID=alice
```

A selection must also support a common continuation for all its options, without
duplicating subsequent steps. That syntax is not illustrated here yet.

## MVP acceptance criteria

1. **Configuration:** read one local YAML file at startup. Configuration changes
   require a restart. The agreed validation failures produce a diagnostic with
   the problem location, without partially loading flows or modifying the file.
2. **Invocation:** a global keyboard shortcut opens the launcher at the root on
   every invocation.
3. **Navigation:** support multilevel folders, configurable folder and flow
   emojis, and mixed navigation through clicks, arrows with Enter, ordinal
   shortcuts, and immediately activated exact aliases with the agreed scopes.
4. **Construction:** support free text, local suggestions, closed selections,
   common continuations, and branches with different required inputs. Visible
   option labels may differ from captured values.
5. **URLs:** preserve the flow's fixed base, including its path prefix, and use
   the reached leaf's path-and-query template. Encode captured values for their
   insertion context, independently of capture order.
6. **Correction:** allow returning to earlier steps; changing an answer discards
   all subsequent answers.
7. **Completion:** reaching a fully resolved destination opens the default
   browser and closes the overlay, without an additional confirmation. This
   includes destinations requiring no input.
8. **Cancellation and privacy:** dismissal abandons the traversal and answers.
   URL construction does not consult external services.

Representative verification scenarios must include a fixed destination, a
linear flow, and a branching flow with different inputs per branch, as well as
the invalid configurations described above. These are acceptance targets, not
claims of tests already run.

Native macOS verification is required for the global shortcut, overlay behavior,
and browser opening. Logic or headless UI tests alone cannot establish those
criteria; native verification requires the maintainer's permission.

## Remaining design questions before implementation

- Complete the YAML schema, including selections with a common continuation,
  using the accepted example as a starting point.
- Remaining URL composition rules: base/path joining and invalid input handling.
- Back-navigation bindings and dismissal controls.
- Configuration location, initial setup, global shortcut configuration, and
  minimum native macOS behavior required for the MVP.

## Deferred capabilities

- Optional or skippable inputs.
- Automatic alias-conflict validation.
- Multiple configuration files and reload without restarting.
- Cross-flow invocation and shared-definition reuse.

External data fetching is an explicit product exclusion, not a planned extension.
