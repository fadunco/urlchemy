# UI-independent Core Implementation Plan

> **For agentic workers:** Use `executing-plans` for approved inline execution, or `subagent-driven-development` only after explicit delegation approval. Track steps with the checkboxes below. Do not implement this draft before approval.

**Goal:** Execute the approved configuration fixtures through a tested Rust engine without GUI dependencies or browser effects.

**Architecture:** Use one library crate with separate file data, validated definitions, and mutable session state. A bounded YAML adapter preserves source locations before schema conversion. Catalog compilation establishes invariants, and the session returns resolved URLs as data.

**Tech Stack:** Rust 1.100.0, edition 2024. Proposed production dependencies are `yaml-rust2 = "=0.13.0"` with default features disabled and `url = "=2.5.8"`. Use `serde_json = "=1.0.154"` only in tests.

**Spec:** [Approved domain model](../specs/2026-11-14-domain-execution-model-design.md) and [approved YAML/URL contract](../specs/2026-11-14-yaml-url-contract-design.md). Read both, plus the [fixture manifest](../specs/yaml-url-contract/expectations.json), before execution.

Status: implementation and verification completed on 2026-11-14 after approval
for inline execution. The original checklist remains below as the approved plan.
The execution record at the end tracks delivered tasks.
Preserve the approved specifications and fixtures. Do not commit, relocate, or delete them as preparation.

## Global Constraints

- Configuration data, validated definitions, and session state remain separate.
- Resource limits are 1,048,576 file bytes, 128 nested collections, and 50,000 syntax nodes.
- At most 50 diagnostics appear, with a separate indication that more diagnostics exist.
- The configuration path is `~/Library/Application Support/URLchemy/config.yaml`.
- There is no file generation, include, live reload, or configuration search fallback.
- Parse YAML 1.2 Core scalars without implicit string conversion.
- Reject duplicate keys, directives, explicit tags, anchors, aliases, and merge keys.
- Every flow has one fixed HTTP(S) base. Preserve its path prefix.
- Encode captured UTF-8 bytes using uppercase percent escapes and the ASCII unreserved set.
- Query spaces use `%20`, not `+`. Captured percent signs remain data.
- Reject complete dot segments before any URL parser can normalize them.
- Changing an answer discards every later answer. An unchanged answer preserves them.
- Every invocation starts at root. Dismissal discards the current session.
- Reaching a valid destination resolves immediately, without another confirmation.
- Do not open URLs, perform DNS/HTTP requests, use shell interpolation, or add GUI/native dependencies.
- Do not log captured values or resolved URLs. Keep diagnostic wording independent of private contents.
- Follow RED-GREEN-REFACTOR for production behavior. Record each observed failure and successful check.
- Approval of this plan must explicitly authorize Cargo bootstrap and dependency downloads before execution.
- No automatic commits, publication, branch changes, worktrees, or subagents.

## Review Focus

1. Numeric-looking text can overflow primitive parsers. Classify YAML scalars lexically rather than treating overflow as a string. Task 1 tests this.
2. Parser markers can differ from original source positions. Preserve quoted starts, Unicode columns, CRLF, BOM, and anchor locations. Tasks 1 and 8 test this.
3. URL parsers can repair malformed authorities or remove encoded dot segments. Check raw syntax before parsing and compare rendered structure afterward. Task 3 tests this.
4. Two options can capture identical text but select different branches. Compare choice identity, and reject identifiers from other catalogs. Tasks 4 and 5 test this.
5. An invalid file or interrupted read must not expose a partial catalog or modify existing data. Task 7 tests this.

---

## File structure and dependency choices

Create this minimal workspace only after execution approval:

```text
Cargo.toml                       workspace, resolver 3, shared versions and metadata
Cargo.lock                       resolved dependency graph, retained in the repository
rust-toolchain.toml              channel 1.100.0, minimal profile, rustfmt and clippy
crates/urlchemy-core/
  Cargo.toml                     library package urlchemy-core, no binary target
  src/
    lib.rs                       public exports, forbid unsafe code
    diagnostic.rs                codes, locations, diagnostic sets, redacted messages
    value.rs                     names, required text checks, presentation checks
    catalog.rs                   owned domain types, guarded identifiers, read-only access
    config/
      mod.rs                     load_bytes orchestration
      source.rs                  original text and scalar/byte/line location mapping
      yaml.rs                    bounded scanner/event adapter and located syntax tree
      scalar.rs                  YAML 1.2 Core scalar classification
      raw.rs                     file representation with field origins
      schema.rs                  strict syntax-tree conversion into RawConfig
      compile.rs                 atomic RawConfig-to-Catalog validation
      file.rs                    bounded reads and explicit-path loading
    url/
      mod.rs                     FixedBase, CompiledTemplate, ResolvedUrl
      base.rs                    raw base checks and WHATWG parsing
      template.rs                literal/capture token compilation and source mapping
      resolve.rs                 context encoding, prefix append, final structure checks
    session/
      mod.rs                     Session, Action, Outcome and read-only snapshots
      history.rs                 replay, comparison, suffix invalidation
      navigation.rs              visible targets and scoped exact alias lookup
    filter.rs                    order-preserving label filtering primitive
  tests/
    support/mod.rs               fixture reading and action translation, not a second engine
    support/scratch.rs           isolated temporary test files with scoped cleanup
    contract.rs                  execute the JSON manifest against production code
    lifecycle.rs                 file loading and startup result tests
README.md                        document core commands and scope after implementation
```

Keep fixtures in their approved location under `docs/superpowers/specs/yaml-url-contract/`.
Do not create a second copy under the crate.
Place focused unit tests in the source module that owns the behavior.
Integration tests use only public APIs and the approved fixtures.

### Dependency rationale and checked APIs

| Dependency | Role | Checked published API and constraints |
| --- | --- | --- |
| `yaml-rust2` 0.13.0 | YAML tokenization and parse events, not its high-level loader | `Scanner::new`, token variants, `Scanner::get_error`, `Parser::new_from_str`, `Parser::next_token`, event markers. Published Rust floor 1.85.0 |
| `url` 2.5.8 | WHATWG base parsing and final structure validation | `Url::parse`, `scheme`, `host_str`, `port`, `username`, `password`, `path`, `query`, `fragment`, and `Position`. Published Rust floor 1.63 |
| `serde_json` 1.0.154, dev-only | Read the existing expectation manifest as `Value` | `from_str`, `Value` accessors. No production JSON or serialization dependency |

All three publish MIT OR Apache-2.0 licensing.
Disabling the YAML encoding feature avoids accepting encodings outside UTF-8.
Do not add `serde_yaml`, regex, async, HTTP, CLI, logging, or error-derive dependencies.
Use small standard-library validators and a byte encoder for the specified grammar.
`url` includes host-processing dependencies. Do not replace it with an ad hoc URL parser
to reduce that dependency graph.

Planning included published documentation and package metadata inspection, not compilation.
Task 1 must verify the resolved graph on the pinned toolchain.
If a capability check fails, stop and revise the dependency choice explicitly.
Do not weaken the approved contract or silently select another version.

Sources:

- [YAML parser API](https://docs.rs/yaml-rust2/0.13.0/yaml_rust2/parser/struct.Parser.html)
- [YAML token variants](https://docs.rs/yaml-rust2/0.13.0/yaml_rust2/scanner/enum.TokenType.html)
- [YAML scanner source](https://docs.rs/yaml-rust2/0.13.0/src/yaml_rust2/scanner.rs.html)
- [YAML package metadata](https://docs.rs/crate/yaml-rust2/0.13.0/source/Cargo.toml)
- [URL API](https://docs.rs/url/2.5.8/url/)
- [URL package metadata](https://docs.rs/crate/url/2.5.8/source/Cargo.toml)
- [JSON API](https://docs.rs/serde_json/1.0.154/serde_json/)

## Shared interface contract

These signatures are implementation targets, not code to create during planning.
Items marked internal use `pub(crate)` or narrower visibility.
Keep constructors for validated definitions and identifiers private.

```rust
// Public configuration boundary in config/mod.rs and config/file.rs.
pub fn load_bytes(source: &Path, bytes: &[u8]) -> Result<Catalog, Diagnostics>;
pub fn load_file(path: &Path) -> Result<Catalog, Diagnostics>;
pub fn configuration_path(home: &Path) -> PathBuf;

// Internal pipeline.
fn parse_yaml(source: &Path, bytes: &[u8]) -> Result<YamlDocument, Diagnostics>;
fn deserialize(document: YamlDocument) -> Result<RawConfig, Diagnostics>;
fn compile(raw: RawConfig) -> Result<Catalog, Diagnostics>;
fn check_text(text: &str) -> Result<(), TextError>;

// Internal URL boundary. Origin carries source path, pointer, and span.
fn parse_base(text: &str, origin: &Origin) -> Result<FixedBase, Diagnostic>;
fn compile_template(text: &str, origin: &Origin)
    -> Result<CompiledTemplate, Diagnostic>;
fn resolve(base: &FixedBase, template: &CompiledTemplate,
    captures: &BTreeMap<CaptureName, CapturedValue>)
    -> Result<ResolvedUrl, ResolutionDiagnostic>;

// Public read-only access and execution.
impl Diagnostics {
    pub fn items(&self) -> &[Diagnostic];
    pub fn truncated(&self) -> bool;
}
impl Diagnostic {
    pub fn code(&self) -> &'static str;
    pub fn pointer(&self) -> &str;
    pub fn location(&self) -> Option<SourceLocation>;
    pub fn message(&self) -> &str;
    pub fn related_locations(&self) -> &[SourceLocation];
}
pub struct SourceLocation { pub line: usize, pub column: usize }
impl Catalog {
    pub fn flow_by_key(&self, key: &str) -> Option<FlowId>;
    pub fn roots(&self) -> &[Entry];
}
impl Session {
    pub fn new() -> Self; // Inactive.
    pub fn apply(&mut self, catalog: &Catalog, action: Action) -> Outcome;
    pub fn snapshot(&self, catalog: &Catalog) -> Result<SessionSnapshot, Diagnostic>;
}
impl ResolvedUrl { pub fn as_str(&self) -> &str; }

// Exact variants and semantics come from the approved domain design.
// Action: Invoke, EnterFolder, StartFlow, SetInputText, SelectSuggestion,
// SelectChoice, Confirm, Back, Dismiss.
// Outcome: StateChanged, Rejected, Resolved, ResolutionFailed.
```

`Origin`, `YamlDocument`, `RawConfig`, `FixedBase`, `CompiledTemplate`,
`CaptureName`, `CapturedValue`, `TextError`, and `ResolutionDiagnostic` use the
responsibilities defined above and in the approved designs.
`TextError` distinguishes `Required` from `Control`.
URL modules return located diagnostics without formatting user input into them.
Only `ResolvedUrl::as_str` intentionally exposes the finished URL to its caller.

`Entry` exposes `id()`, `label()`, `emoji()`, `children()`, and `flow()` read-only getters.
`children()` returns `Option<&[Entry]>`, with `None` for a flow entry.
`flow()` returns `Option<FlowId>`, with `None` for a folder entry.
Navigation presentation remains independent from the flow execution tree.
Use `Diagnostic` as the underlying representation of `ActionDiagnostic` and
`ResolutionDiagnostic`, with the same read-only accessors.
Invalid action context, target, or catalog use `E_ACTION_CONTEXT`,
`E_ACTION_TARGET`, or `E_ACTION_CATALOG`, respectively.

`SessionSnapshot` is an owned read model with these variants:

```rust
pub enum SessionSnapshot {
    Inactive,
    Browsing { folders: Vec<EntryId>, rows: Vec<Row> },
    Input { step: StepId, label: String, text: String,
        selected: Option<SuggestionId>, rows: Vec<Row>, cursor: usize,
        committed_answers: usize },
    Selection { step: StepId, label: String, selected: Option<ChoiceId>,
        rows: Vec<Row>, cursor: usize, committed_answers: usize },
    DestinationError { step: StepId, committed_answers: usize },
    Completed,
}
pub struct Row { id: RowId, label: String, emoji: Option<String> }
pub enum RowId { Entry(EntryId), Choice(ChoiceId), Suggestion(SuggestionId) }
```

Expose rows through read-only getters `id()`, `label()`, and `emoji()`.
Snapshot fields shown above are public read data, not mutable session internals.
Snapshot text is for the UI controller, not diagnostic output.
Use redacted `Debug` output for input-bearing actions, answers, drafts, snapshots,
and resolved URLs. Do not derive a formatter that prints those contents.

Identifiers carry a private catalog identity as well as a node index.
A checked process-local identity allocator distinguishes separate loaded catalogs.
This makes foreign identifiers rejectable even when their numeric indices match.
Sessions bind to the catalog identity on their first `Invoke`.
An action using another catalog rejects without changing state, except `Invoke`,
which resets and binds to the supplied catalog.
Snapshot access with the wrong catalog returns `E_ACTION_CATALOG`, not foreign rows.
This refines the illustrative integer IDs without changing their approved role.

## Task 1: Bootstrap and bounded located YAML parsing

**Files:** Create workspace manifests, lockfile, toolchain file, `src/lib.rs`,
`src/diagnostic.rs`, `src/config/mod.rs`, `src/config/source.rs`,
`src/config/yaml.rs`, and `src/config/scalar.rs` under the crate above.
Tests live in those source modules.

**Interfaces:** Produce `parse_yaml`, `YamlDocument`, `Origin`, `Diagnostic`,
`Diagnostics`, `SourceLocation`, and internal
`classify_scalar(text: &str, style: yaml_rust2::scanner::TScalarStyle) -> ScalarKind`.
`ScalarKind` distinguishes String, Null, Boolean, Integer, and Float without numeric conversion.
`YamlDocument` owns the original source map and an optional located root node.
Its mappings retain ordered key/value nodes, not a map that drops duplicate keys.

- [ ] **Step 1:** After explicit bootstrap approval, create the minimal package metadata and empty module declarations. Use workspace member `crates/urlchemy-core`, resolver `3`, package version `0.1.0`, license `MIT`, and `publish = false`. Pin Rust 1.100.0 and the versions above. Set `#![forbid(unsafe_code)]`. Generate `Cargo.lock` with `cargo generate-lockfile`. Do not create a binary or product implementation yet.
- [ ] **Step 2:** Write `config::yaml::tests::located_yaml_gate` with assertions that parse errors expose the specified code and location. Include the following representative assertions:

```rust
let e = parse_yaml(Path::new("fixture.yaml"), b"navigation: &entries []\n").unwrap_err();
assert_eq!(e.items()[0].code(), "E_YAML_FEATURE");
assert_eq!(e.items()[0].location(), Some(SourceLocation { line: 1, column: 13 }));
let e = parse_yaml(Path::new("fixture.yaml"), b"a: x\na: y\n").unwrap_err();
assert_eq!(e.items()[0].code(), "E_YAML_DUPLICATE_KEY");
assert_eq!(e.items()[0].location(), Some(SourceLocation { line: 2, column: 1 }));
```

Add table cases for explicit tags, aliases, merge keys, directives, multiple
documents, invalid UTF-8, and malformed YAML. Test known and unknown directives.
Test literal text containing `&`, `%`, or `!` inside quotes. Quoted feature-like text must parse.
Test byte lengths 1,048,576 and 1,048,577, collection depths 128 and 129, and
syntax node counts 50,000 and 50,001. Only the upper case of each pair fails
with `E_CONFIG_LIMIT`. Use a comment for the byte-boundary input, nested flow
sequences for depth, and one sequence plus scalar children for node count.
An empty or comment-only document parses with no root for later schema rejection.

- [ ] **Step 3:** Write `config::scalar::tests::core_scalar_types` as a table. Plain `123`, a 500-digit decimal, `0o17`, and `0xFF` classify as Integer. Plain `1e3`, `.5`, and `.NaN` classify as Float. Plain `true` and `FALSE` classify as Boolean. Plain `null`, `~`, and an implicit empty scalar classify as Null. Plain `yes`, `on`, `0b101`, `1_000`, and `2026-11-14` classify as String. Quoted `123`, `true`, and an empty quoted scalar classify as String.
- [ ] **Step 4:** Run `cargo test -p urlchemy-core --locked config::`. Expect missing implementation errors, not a green stub. Record the failure.
- [ ] **Step 5:** Implement the parser adapter using a bounded token pass and an explicit-stack event pass. Reject forbidden tokens without expanding aliases. Check `Scanner::get_error` after token iteration. Do not interpret early iterator termination as success. Use pull-style `Parser::next_token` so limits can stop parsing. Classify plain scalars using the YAML Core lexical grammar, not `parse::<i64>()` success.

Retain scanner starts for quotes, anchors, tags, and block scalars where parser
events lose those positions. Build scalar-index-to-byte mappings from original
text. Normalize one leading BOM and CRLF for location reporting without losing
original byte offsets. The scanner's published line counter starts at one and
column counter at zero. Do not blindly add one to both.

During the bounded lexical pass, retain the earliest forbidden-feature or scan
failure candidate. During the event pass, detect earlier syntax and duplicate-key
failures. Report the first failure by source position, as the contract requires.
Enforce collection depth during scanning as well as event construction.
Do not scan an arbitrarily deep stream fully before applying the bound.
Count actual syntax nodes in the event pass, including mapping keys and implicit nulls.

- [ ] **Step 6:** Run `cargo test -p urlchemy-core --locked config::` and `cargo tree --locked`. Expect all Task 1 tests to pass, no GUI/HTTP dependency, and no YAML `encoding_rs` feature dependency. Retain the lockfile. Stop if the adapter cannot preserve required starts or enforce bounded parsing through the reviewed API.

## Task 2: Strict schema conversion and shared text validation

**Files:** Create `src/value.rs`, `src/config/raw.rs`, and `src/config/schema.rs`.
Update module declarations and diagnostic codes as needed.

**Interfaces:** Consume Task 1 syntax nodes and source mapping.
Produce `deserialize(document) -> Result<RawConfig, Diagnostics>` and
`check_text(&str) -> Result<(), TextError>`.
Raw types match the approved domain sketch, with origins retained for every
field, key, and omitted-field parent. They remain independent of session types.

- [ ] **Step 1:** Write `config::schema::tests::strict_schema` using the approved fixtures. Assert that all four valid files deserialize. Assert exact codes, pointers, and locations for unknown fields, numeric values, null, and empty options from the manifest. Assert missing input `next` gives `E_SCHEMA_REQUIRED`. Assert unknown or multiple wrapper alternatives give `E_SCHEMA_VARIANT`. Empty root gives `E_SCHEMA_TYPE`, while `navigation: []` succeeds.
- [ ] **Step 2:** Write `value::tests::required_text_contract` with exact results:

```rust
assert_eq!(check_text(""), Err(TextError::Required));
assert_eq!(check_text(" \u{00a0} "), Err(TextError::Required));
assert_eq!(check_text("jobs\narchive"), Err(TextError::Control));
assert_eq!(check_text("\t"), Err(TextError::Control));
assert_eq!(check_text(" accounts "), Ok(()));
assert_eq!(check_text("café/ops & SRE"), Ok(()));
```

Test every listed White_Space and forbidden-control range from the contract.
Assert schema strings preserve bytes, quoted numeric values remain strings,
unknown fields never disappear, and omitted suggestions become an empty list.
Use field-appropriate diagnostics for invalid labels, emojis, capture names,
flow identifiers, alias characters, and configured values.

- [ ] **Step 3:** Run `cargo test -p urlchemy-core --locked strict_schema` and `cargo test -p urlchemy-core --locked required_text_contract`. Expect RED failures.
- [ ] **Step 4:** Implement explicit schema conversion from located syntax nodes. Apply the complete key tables and omitted-versus-null rules. Collect independent structural diagnostics in source order, capped at 50 with a truncation flag. Preserve raw continuation combinations for domain checks in Task 4. Do not add alias-conflict validation.
- [ ] **Step 5:** Run the commands from Step 3 and `cargo test -p urlchemy-core --locked config::`. Expect all schema, text, and prior parsing tests to pass.

## Task 3: URL compilation and resolution without effects

**Files:** Create `src/url/mod.rs`, `src/url/base.rs`, `src/url/template.rs`,
and `src/url/resolve.rs`. Add `CaptureName` and `CapturedValue` in `src/value.rs`.

**Interfaces:** Produce the shared `parse_base`, `compile_template`, and `resolve`
signatures. `CompiledTemplate` exposes an internal iterator over capture references
with their source origins for Task 4. Keep path and query tokens distinct.

- [ ] **Step 1:** Write `url::base::tests::strict_bases`. Assert canonical serialization of `HTTPS://EXAMPLE.COM:443` is `https://example.com/`. Reject credentials, missing authority, `https:///example.com`, queries/fragments including empty markers, port zero, port 65536, malformed escapes, non-ASCII bases, backslashes, and literal/encoded dot segments before normalization. Accept `https://[::1]:8443/company/` and preserve its prefix.
- [ ] **Step 2:** Write `url::template::tests::template_grammar`. Assert empty and query-only templates compile. Assert partial-segment placeholders, duplicate query names, literal `%7B`, and empty static query values compile. Assert malformed braces/escapes, `//host`, relative paths, fragments, raw Unicode, placeholders in query names, bare query names, extra raw `=`, and trailing `&` fail with `E_TEMPLATE_SYNTAX`.
- [ ] **Step 3:** Write `url::resolve::tests::encoding_and_prefix`. Turn all six prefix-table rows in the approved URL contract into exact assertions. Add these encoder cases using an internal `encode_capture(&str) -> String` helper:

```rust
assert_eq!(encode_capture("A+B & café/ops"), "A%2BB%20%26%20caf%C3%A9%2Fops");
assert_eq!(encode_capture("%2F"), "%252F");
assert_eq!(encode_capture("~._-"), "~._-");
```

Resolve complete templates, not just encoder outputs. Assert a query value `..`
succeeds, a complete path segment `..` fails with `E_RESOLVE_DOT_SEGMENT`,
and literals plus captures that jointly render `..` also fail.
Assert encoded static dot spellings reject at compilation, before `Url::parse`.
Literal `%7B` stays `%7B`, while captured `{x}` becomes `%7Bx%7D`.
Assert repeated query names and literal escape casing remain unchanged.

- [ ] **Step 4:** Run `cargo test -p urlchemy-core --locked url::`. Expect RED failures.
- [ ] **Step 5:** Implement raw base checks before `Url::parse`. Compile the restricted template grammar into typed tokens. Encode capture bytes without `query_pairs_mut`, form encoding, `Url::join`, or path setters that reinterpret existing escapes. Append the prefix explicitly, parse the final string once, and compare its scheme, authority, path, and query with the constructed components. Preserve the constructed URL bytes in `ResolvedUrl` only after those checks succeed.
- [ ] **Step 6:** Run `cargo test -p urlchemy-core --locked url::`. Expect all URL tests to pass. Confirm by inspection that the module has no file, process, network, or browser-opening calls.

## Task 4: Atomic validated catalogs and guarded identities

**Files:** Create `src/catalog.rs`, `src/config/compile.rs`, and complete
`src/config/mod.rs` orchestration.

**Interfaces:** Consume RawConfig and Task 3 compilation functions.
Produce `compile`, `load_bytes`, the approved owned catalog tree, and its
read-only entry/flow accessors. Implement catalog, step, choice, and suggestion
identities with private owner tokens. Allocation failures must return diagnostics,
not wrap indices or panic. Keep traversal lookup simple before adding indexes.

- [ ] **Step 1:** Write `config::compile::tests::catalog_invariants`. Assert `branching.yaml` compiles with two distinct continuations. Assert `common.yaml` owns one common continuation. Assert duplicate flow keys, repeated path captures, missing/mixed continuations, unused non-capturing values, and unavailable leaf references reject. Match manifest codes, pointers, and locations for each existing invalid fixture.
- [ ] **Step 2:** Add path-sensitive cases. A value captured only in one sibling branch must not validate a destination in another branch. The same capture name in two separate branches must compile. Two choices with identical stored text must retain different `ChoiceId` values and continuations. An unused capture is valid. Empty folders and the empty root are valid.
- [ ] **Step 3:** Add `config::compile::tests::atomic_catalog`. Combine a valid flow followed by an invalid flow. Assert `load_bytes` returns only diagnostics and exposes no catalog. Create 51 independent domain errors and assert 50 ordered diagnostics plus `truncated() == true`. Assert an invalid capture name does not create a secondary unavailable-reference diagnostic.
- [ ] **Step 4:** Run `cargo test -p urlchemy-core --locked config::compile::`. Expect RED failures.
- [ ] **Step 5:** Implement recursive validation with independent capture sets per branch. Enforce continuation and capture-value mode rules before producing typed Selection variants. Issue identifiers only for the candidate catalog and publish it only on complete success. Validate presentation, names, and values before dependent reference checks. Keep source ordering deterministic.
- [ ] **Step 6:** Run `cargo test -p urlchemy-core --locked config::` and `cargo test -p urlchemy-core --locked url::`. Expect all earlier parsing/schema/URL tests and catalog tests to pass.

## Task 5: Session transitions, review history, and immediate resolution

**Files:** Create `src/session/mod.rs`, `src/session/history.rs`, and initial
`src/session/navigation.rs`. Export Session, Action, Outcome, and snapshot types.

**Interfaces:** Consume the immutable Catalog and URL resolver.
Produce `Session::new`, `Session::apply`, and `Session::snapshot` with the exact
action/outcome semantics above. Store answers and cursor separately from the draft.
Derive captures by replaying the committed active path, never from a stale cache.

- [ ] **Step 1:** Write `session::tests::immediate_completion`. Use `fixed.yaml` to assert `StartFlow` resolves the input-free destination immediately after `Invoke`. Assert a Completed snapshot and rejection of another Confirm without a URL. Use both Datadog branches to assert resolution happens on the final Confirm, not on another Enter.
- [ ] **Step 2:** Write `session::history::tests::review_and_invalidation` using a four-input test configuration. Confirm X=`x`, Y=`y`, Z=`z`, then stop at W. Back twice restores Y with three committed answers. Confirm unchanged Y and assert Z remains recorded and restored. Back once, change Y to `new-y`, and assert only X and Y remain. For common-path and equal-valued-choice cases, use inline definitions with enough continuation inputs to review an earlier committed suffix before completion. Changing either choice must clear that suffix, even if its captured value or next step stays the same.
- [ ] **Step 3:** Write `session::tests::drafts_and_rejections`. Editing a draft then returning without confirmation preserves the old committed answer and suffix. Empty/control input preserves draft, cursor, and history with the specified error code. Text edits clear an explicit suggestion selection. Literal `Jobs` confirms `Jobs`, while selecting its suggestion confirms `jobs`. A suggestion and literal with identical effective text preserve the same suffix.
- [ ] **Step 4:** Write `session::tests::reset_and_identity`. Invoke or Dismiss clears answers and folders. Back at the first step returns to the navigation origin and discards the run. Direct access records root as origin. Invalid actions and foreign/stale step choices leave snapshots unchanged. Load the same bytes into two catalogs and assert IDs from one reject in the other's session even when local indices match. Snapshot mismatch against another catalog must not expose that catalog's rows or private state.
- [ ] **Step 5:** Write `session::tests::resolution_error`. Confirm both answers of the manifest's dot-segment rejection case and assert DestinationError retains two committed answers without a URL. Back restores the final answer. Correcting it permits completion. For a direct-destination resolution failure, use an internal injected resolver failure rather than admitting an invalid public catalog. Assert Back leaves its empty run for the origin.
- [ ] **Step 6:** Run `cargo test -p urlchemy-core --locked session::`. Expect RED failures.
- [ ] **Step 7:** Implement semantic transitions and one-position-at-a-time review advancement. Restore committed text as literal draft text, with no implicit suggestion selection. Reject unsupported actions before mutation. On resolver failure retain valid committed history at DestinationError. On success clear answers and emit exactly one Resolved outcome. Implement redacted Debug formatting and no external effects.
- [ ] **Step 8:** Run `cargo test -p urlchemy-core --locked session::`. Expect all session tests to pass. Inspect the production call path to confirm no browser launch and no extra final confirmation.

## Task 6: Pure filtering and scoped navigation lookup

**Files:** Create `src/filter.rs` and complete `src/session/navigation.rs`.

**Interfaces:** Produce these policy-explicit helpers:

```rust
pub enum MatchMode { CaseSensitiveSubstring, CaseInsensitiveSubstring }
pub fn filter_rows(rows: &[Row], query: &str, mode: MatchMode) -> Vec<RowId>;
impl Session {
    pub fn exact_alias(&self, catalog: &Catalog, text: &str) -> Option<Action>;
}
```

Filtering matches row labels and preserves configuration order and row identity.
Case-insensitive mode uses Unicode lowercase, without accent removal or normalization.
Neither mode has an implicit default. The interaction design chooses the UI policy.
This helper is not a ranking, fuzzy-matching, ordinal-binding, or focus system.
Its two explicit modes are technical proposals for plan approval, not reopened product decisions.

- [ ] **Step 1:** Write `filter::tests::ordered_label_filter`. Given rows labeled `Accounts`, `Billing`, and `accounts API`, assert an empty query returns all IDs in order. Sensitive query `accounts` returns only the third ID. Insensitive query `accounts` returns the first and third IDs. No match returns an empty list. Values not present in labels do not match. Assert queries do not mutate selection or confirm suggestions.
- [ ] **Step 2:** Write `session::navigation::tests::alias_context`. At root, `open-dd` resolves to the nested Datadog flow. Inside Datadog, `dashboards` resolves to it and `open-dd` does not. Prefix `open` never activates. While entering a value, all navigation aliases return None. A folder alias produces EnterFolder, and a visible flow alias produces StartFlow. Do not test or implement automatic conflict detection.
- [ ] **Step 3:** Run `cargo test -p urlchemy-core --locked filter::` and `cargo test -p urlchemy-core --locked alias_context`. Expect RED failures.
- [ ] **Step 4:** Implement only the helpers above. Return semantic actions from exact alias lookup, leaving immediate dispatch to the caller. Keep ordinal positions and physical keybindings out of the engine. Define behavior only for configurations satisfying the documented alias-conflict restriction.
- [ ] **Step 5:** Run both commands from Step 3 and all session tests. Expect all to pass.

## Task 7: Explicit-path loading without configuration side effects

**Files:** Create `src/config/file.rs`, `tests/lifecycle.rs`, and
`tests/support/scratch.rs`. Re-export the public configuration boundary.

**Interfaces:** Produce `configuration_path` and `load_file` above, plus internal
`load_reader(source: &Path, reader: impl Read) -> Result<Catalog, Diagnostics>`.
The native adapter supplies the actual home directory later.
The core neither discovers home through environment variables nor requests OS permissions.

- [ ] **Step 1:** Write `configuration_path_is_explicit` with this assertion:

```rust
assert_eq!(configuration_path(Path::new("/fixture-home")),
    PathBuf::from("/fixture-home/Library/Application Support/URLchemy/config.yaml"));
```

Write integration test `missing_and_unreadable_files_are_diagnostics` to assert a missing path
gives `E_CONFIG_MISSING` and a directory gives `E_CONFIG_READ`, both without
invented source coordinates. In `src/config/file.rs`, add unit test
`read_failures_are_atomic`. An injected reader failing after a valid YAML prefix
must return `E_CONFIG_READ` rather than compile the prefix. Simulate PermissionDenied
through injected I/O errors instead of relying on the test account's permissions.

- [ ] **Step 2:** Write `bounded_read_and_no_mutation`. Pad `navigation: []` with a comment to create a valid 1,048,576-byte file, then add one byte for the oversized case. Read at most the byte limit plus one byte. Assert the exact-boundary file loads and oversize gives `E_CONFIG_LIMIT`. Compare fixture bytes before and after success/failure and assert no new file or directory appears. A loaded catalog must remain unchanged after editing the test file until a new explicit load occurs.
- [ ] **Step 3:** Run `cargo test -p urlchemy-core --locked --test lifecycle` and `cargo test -p urlchemy-core --locked config::file::`. Expect RED failures.
- [ ] **Step 4:** Implement bounded reads with standard-library I/O. An open failure with NotFound maps to `E_CONFIG_MISSING`. All other open failures and all reader failures map to `E_CONFIG_READ`. Preserve atomic publication through `load_bytes`. Use isolated test paths created with collision-safe creation. Cleanup removes only files created by that test and then its directory. Never access the maintainer's real launcher file.
- [ ] **Step 5:** Run both Step 3 commands and `cargo test -p urlchemy-core --locked config::`. Expect all lifecycle and prior configuration tests to pass.

## Task 8: Production fixture harness, regression coverage, and handoff

**Files:** Create `tests/support/mod.rs` and `tests/contract.rs`.
Modify `README.md` only to describe the new library, fixture checks, and commands.

**Interfaces:** Test-only `fixture(name: &str) -> Vec<u8>` reads approved fixtures.
Test-only `drive(catalog: &Catalog, flow_key: &str, answers: &[serde_json::Value])
-> (Session, Outcome)` translates the manifest into public production actions.
It invokes root, locates the flow through its navigation entries, then applies
the listed confirmations using current snapshot row IDs.
It must not construct URLs, validate YAML, resolve captures, or emulate history.
Do not bypass nested navigation by calling an inaccessible StartFlow from root.

- [ ] **Step 1:** Write `all_contract_examples_execute` to read `expectations.json`. Assert every valid YAML file loads and every journey produces its exact URL plus Completed state. Assert every runtime rejection has the specified code, no URL, and the described preserved state. Assert every invalid file returns its exact code, pointer, line, and column. Assert every fixture file appears in the manifest.
- [ ] **Step 2:** Write `source_coordinates_and_privacy`. Add inline configurations with BOM, CRLF, quoted scalars, Unicode before the failing node, escaped duplicate keys, and missing fields. Assert original-character columns and related duplicate origins. Use sentinel input `private-session-value` and assert it never appears in error messages or Debug formatting for actions, snapshots, outcomes, or URLs. Also assert invalid schema diagnostics suppress template execution entirely.
- [ ] **Step 3:** Run `cargo test -p urlchemy-core --locked --test contract`. Expect RED failures for missing harness coverage or production defects. Do not alter approved expected URLs or diagnostics just to make implementation pass. If an approved expectation contradicts the selected parser's behavior, investigate whether the adapter needs correction before requesting a contract change.
- [ ] **Step 4:** Implement only test-driver translation and fix uncovered production defects with focused RED-GREEN tests. Use real production views of state, not a test-only launcher model. Document library-only scope and manual configuration setup without claiming native behavior exists.
- [ ] **Step 5:** Run the complete verification suite from repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo tree --locked
sh scripts/doctor.sh
sh scripts/test-doctor.sh
git diff --check
```

Expected: all Rust and existing harness checks pass. The dependency graph contains
no GPUI, HTTP client, browser launcher, or native integration dependency.
The doctor now recognizes Cargo and no longer reports application checks as inapplicable.
Record command output and any skipped checks honestly.

- [ ] **Step 6:** Review the focused diff and untracked files without staging or committing. Verify Cargo.lock and the pinned toolchain exist, all acceptance cases map to actual tests, and no private configuration or build output entered the change set. Report unit/integration results separately from native testing, which did not run.

## Execution and review gates

Recommended mode: inline execution in the current checkout, without subagents.
The tasks share several interfaces, so sequential work avoids repeated context
and reduces coordination cost. Self-review is not independent review.
An independent reviewer requires separate explicit delegation approval.

Before execution, obtain approval of this plan and its two filtering-helper modes.
Obtain authorization to create Cargo files and download/build dependencies.
Confirm the execution mode. Do not create a worktree or move the existing untracked work.
Agree on any branch change before making it.

After each task, perform a focused self-review and record the RED/GREEN evidence
in ignored `.superpowers/` execution notes. Do not place private input there.
Commits are optional user-authorized checkpoints, not automatic task steps.
Publication and native verification remain separately gated.

## Plan self-review

- Spec coverage: Tasks 1–4 cover startup syntax, schema, domain, and URL rules.
- Session coverage: Task 5 covers navigation, confirmation, correction, reset, and completion.
- Core helpers: Task 6 tests filtering without fixing the later UI interaction policy.
- File lifecycle: Task 7 preserves explicit paths, bounded reads, and atomic loading.
- Contract evidence: Task 8 executes the approved fixtures through production APIs.
- Type consistency: consumers use the shared interfaces above. Tests need no production test-only constructor.
- Remaining boundary: actual home lookup, startup UI, keybindings, global shortcut, overlay, and browser opening belong to later milestones.
- Verification performed while planning: prerequisite diagnostics and published API/source inspection only. No Rust compilation or application test ran.

## Execution record

The maintainer approved Cargo bootstrap, dependency downloads, and execution in
the current session without subagents. The maintainer separately authorized
commit, push, and PR integration on 2026-11-14.

| Task | Delivered result |
| --- | --- |
| 1 | Pinned workspace and bounded, located YAML parsing |
| 2 | Strict file schema and shared text validation |
| 3 | Fixed-base/template compilation and context-safe URL resolution |
| 4 | Atomic validated catalog publication and guarded identifiers |
| 5 | Semantic sessions, history review, suffix invalidation, and immediate completion |
| 6 | Policy-explicit filtering and scoped exact alias lookup |
| 7 | Explicit-path bounded file loading without configuration side effects |
| 8 | Production contract harness, source/privacy regressions, and core documentation |

Task verification used complete unit/integration runs in addition to focused
regression runs. Initial missing-API compilation failures supplied RED evidence.
Supplemental boundary tests passed existing behavior and have no separate RED evidence.
The author performed the final self-review. No independent review or native verification ran.
Detailed commands and results remain in the ignored execution ledger.

Final verification on 2026-11-14: 28 Rust tests passed, including the complete
approved fixture manifest. Formatting, Clippy with warnings denied, the dependency
inspection, and both harness diagnostic scripts passed.
Native behavior remains untested and outside this milestone.
