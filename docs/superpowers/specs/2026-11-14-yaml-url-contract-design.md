# YAML contract and URL semantics

Status: approved by the maintainer on 2026-11-14 for roadmap milestone 2.
Approval covers the strict contract, detailed policies, and accompanying fixtures.

## 1. Scope and decisions

This specification completes the file and URL boundaries of the
[approved domain model](2026-11-14-domain-execution-model-design.md).
It preserves [the product behavior](../../product.md) and implements no application code.
The deliverables are this contract and the complete
[fixture set](yaml-url-contract/expectations.json).

| Decision | Alternatives | Selected approach and reason |
| --- | --- | --- |
| YAML validation | Strict rejection or implicit conversion | Strict rejection, selected by the maintainer. Detect typing errors before execution |
| Continuations | Shared `select.next` or duplicated option trees | Shared `next` for common continuations. Option `next` for branches, preserving the accepted example |
| Configuration location | Native macOS directory or `~/.config/urlchemy/` | Native macOS directory. Avoid two search paths and environment-dependent precedence |
| Input text | Preserve, trim, or normalize | Preserve nonblank text exactly. Reject blank values instead of changing their meaning |
| URL composition | Relative URL resolution or explicit prefix append | Explicit append. A destination must never remove the flow's base prefix |
| Query spaces | `%20` or form-style `+` | `%20`. Use one explicit percent-encoding rule and preserve literal plus signs |
| URL schemes | HTTP(S) or arbitrary protocols | HTTP(S) initially. Other protocols have different parsing and opening behavior |

The maintainer approved all choices above as part of this milestone.
No Cargo files, dependencies, UI bindings, native integration, or generated user
configuration belong to this work.

## 2. File lifecycle and initial setup

The single location is:

```text
~/Library/Application Support/URLchemy/config.yaml
```

Here `~` denotes the current user's home directory, obtained through the native
filesystem boundary. It is not shell expansion performed on configuration data.
There is no working-directory search, environment override, second filename,
fallback file, include, or automatic example installation in the MVP.

The application reads this file once at startup. It publishes the complete
validated catalog or a startup diagnostic state, never a partially loaded catalog.
An existing diagnostic state still permits launcher invocation to display the problem.
It cannot execute flows. Correcting the file requires restarting the application.

| Condition | Result |
| --- | --- |
| File or containing directory does not exist | `E_CONFIG_MISSING`, with the expected location |
| File cannot be read, or the path is a directory | `E_CONFIG_READ`, without file contents |
| Empty or comment-only file | `E_SCHEMA_TYPE`, expected root mapping |
| Valid `navigation: []` | Empty catalog, with no executable entries |
| Loading or validation failure | No catalog. Do not modify the file |

Initial setup is manual:

1. Create the `URLchemy` directory in the location above.
2. Create `config.yaml` with a text editor.
3. Use a complete fixture as a starting point and replace its example services.
4. Start or restart URLchemy.

These are future user instructions, not actions performed during this design.
The application does not create directories, overwrite files, or open an editor.
The repository fixtures contain public-safe example data, not private launcher data.

## 3. YAML subset and structural rules

The file contains exactly one UTF-8 YAML document, using
[YAML 1.2 Core scalar types](https://yaml.org/spec/1.2.2/#103-core-schema).
The reader accepts one leading UTF-8 byte-order mark. Mapping keys must be strings.
The parser accepts comments, block and flow collections, and ordinary quoted or plain scalars.
That document may contain one start marker and one end marker.

Reject these features before constructing a domain tree:

- Duplicate mapping keys, including keys with equivalent decoded strings.
- YAML directives, explicit tags, anchors, aliases, and merge keys.
- More than one document, including an additional empty document.
- Invalid UTF-8, invalid YAML syntax, or unsupported scalar types at a field.

No scalar conversion occurs. A string field rejects integers, floats, booleans,
sequences, mappings, and null. For example, `value: 123` fails while `value: "123"` succeeds.
The YAML 1.2 words `yes`, `no`, `on`, and `off` are strings, not booleans.
Quoted empty strings remain strings but may fail field validation.
Optional means omitted, not null. Explicit `null` never means an omitted field.

Every mapping accepts only the keys listed in this specification.
Field names are case-sensitive. List order determines navigation order, option
order, and suggestion order. Mapping key order has no execution meaning.

Resource limits are 1,048,576 file bytes, 128 nested collections, and
50,000 syntax nodes. Keys and values both count as nodes.
The root collection has depth one. A nested sequence or mapping adds one level.
Apply the byte limit while reading and the other limits while parsing.
Reject a limit breach with `E_CONFIG_LIMIT`, without publishing any definitions.
These are safety limits, not a fixed number of conceptual flow layers.

The eventual YAML library must meet these rules without object construction or
implicit coercion. Selecting that dependency belongs to the implementation plan.

## 4. Complete schema

All fields below are strings unless a table specifies another type.
No serialization version field exists in this initial contract.

### Root and navigation entries

The approved [shortcut amendment](2026-11-15-shortcuts-framework-design.md)
adds an optional `shortcuts` root mapping with default values.
The original schema below records the milestone 2 contract.

| Mapping | Required fields | Optional fields |
| --- | --- | --- |
| Root | `navigation`: list of entries | None |
| Entry wrapper | Exactly one of `folder` or `flow`: mapping | None |
| Folder | `label`, `children`: list of entries | `emoji`, `alias` |
| Flow | `id`, `label`, `baseURL`, `start`: step wrapper | `emoji`, `alias`, `direct_alias` |

Root and folder entry lists may be empty. Every flow has exactly one navigation entry.
Flow `id` values are globally unique, case-sensitive configuration identifiers.
Folders have no `id`, base, capture, or construction step.

### Steps

| Mapping | Required fields | Optional fields |
| --- | --- | --- |
| Step wrapper | Exactly one of `input`, `select`, or `destination`: mapping | None |
| Input | `label`, `capture`, `next`: step wrapper | `suggestions`: list of suggestions |
| Suggestion | `label`, `value` | None |
| Select | `label`, `options`: nonempty list of options | `capture`, `next`: step wrapper |
| Option | `label` | `value`, `next`: step wrapper, subject to the rules below |
| Destination | `template` | None |

An omitted `suggestions` field means an empty list.
An explicit empty suggestions list has the same meaning.
No other field receives a default value.
An entry or step wrapper with anything other than one supported key produces
`E_SCHEMA_VARIANT`. Unknown fields inside a recognized variant produce
`E_SCHEMA_UNKNOWN_FIELD` instead.

**Common continuation:** `select.next` exists and every option omits `next`.
**Branching continuation:** `select.next` is absent and every option has `next`.
Reject mixed modes and missing continuations with `E_CONTINUATION`.
A destination is already a complete continuation. It never has a `next` field.

**Capturing selection:** `select.capture` exists and every option has `value`.
**Non-capturing selection:** `select.capture` is absent and every option omits `value`.
Reject incomplete or unused option values with `E_CAPTURE_VALUE`.
This prevents a configured value from being silently ignored.

### Strings, names, and presentation

- Flow identifiers and capture names match `[A-Za-z_][A-Za-z0-9_-]*`.
- A capture name occurs at most once on a traversal. Separate branches may reuse it.
- Every template reference must exist on that destination's traversal.
- Captured values and suggestion values follow the input rules in section 7.
- Labels must contain a non-whitespace character and no forbidden control character.
- `emoji` follows the label rule. It is presentation text, not a Unicode-classification check.
- Only folders and flows accept `emoji`. Omitting it means no configured emoji.
- An alias matches `[a-z0-9][a-z0-9_-]*`. Alias matching is exact and case-sensitive.
- Labels may repeat. Option and suggestion values may repeat.

Ordinal positions and internal identifiers are not file fields.
Options with identical values remain different choices, even if their labels also match.
They can lead to different continuations, as established in the approved session model.

Folder and flow `alias` values apply at their containing navigation level.
Flow `direct_alias` values apply at the root, including flows nested in folders.
At the root, direct aliases and visible entry aliases share one context.
Aliases in a context must neither duplicate nor prefix one another.
Automatic conflict detection remains deferred, as agreed in the product definition.
Schema validation of alias characters does not imply conflict validation.

## 5. Fixed bases

The MVP accepts absolute `http` and `https` URLs only.
The configured string must explicitly contain a scheme, `://`, and a nonempty host.
Reject credentials, query markers, fragment markers, and placeholders in the base.
Reject raw spaces, non-ASCII characters, control characters, backslashes, and malformed percent escapes.
Non-ASCII paths can use their UTF-8 percent-encoded representation.
International hostnames must use their ASCII representation in this initial contract.
Reject percent escapes in the authority, including the host and port.

Parse the remaining base using the
[WHATWG URL Standard](https://url.spec.whatwg.org/).
Reject parse failures and ports outside 1 through 65535.
No network request or DNS lookup participates in validation.
Use the parser's serialized scheme, authority, and path as the fixed base.
Standard host casing, default-port removal, and root-path serialization therefore apply.
For example, `HTTPS://EXAMPLE.COM:443` becomes `https://example.com/`.
This rule does not select a Rust dependency or version.

Before URL parsing, reject a base path segment whose percent-decoded bytes equal
`.` or `..`. Percent decoding here occurs once and is case-insensitive for hex digits.
The check includes literal, escaped, and mixed dot spellings, such as `.%2e`.
This prevents a parser from silently removing configured path segments.
The base may otherwise contain repeated or trailing slashes.
Do not remove empty internal path segments.

Validation fixes the base. No captured value can replace its scheme,
host, port, or path prefix. A base's optional path prefix is not a folder URL.

## 6. Destination grammar and URL construction

### Grammar

A destination contains a path, an optional query, or both.
The empty template is valid and resolves to the serialized fixed base.
A query-only template, such as `?view=summary`, preserves the base path unchanged.
Every nonempty path begins with exactly one `/`.
Reject relative paths, authority-like `//` prefixes, absolute URLs, and fragments.
The resolver accepts and preserves internal or trailing path slashes.

Path segments and query values contain sequences of literal atoms and
`{capture-name}` placeholders. Partial-segment substitutions such as
`/ticket/APP-{ticket_number}` are valid.
Capture names use the grammar from section 4 and match case-sensitively.
There is no recursive substitution, expression syntax, environment expansion,
default value, or placeholder in a query name.

A literal atom is an ASCII unreserved character or a `%HH` escape.
The unreserved set is `A-Z`, `a-z`, `0-9`, `-`, `.`, `_`, and `~`.
Hex digits in literal escapes may use either case. The resolver preserves their spelling.
Encode other literal characters before writing them into the template.
For example, `%7B` and `%7D` represent literal braces.
Double braces are not an escape mechanism.
Raw Unicode, spaces, backslashes, unmatched braces, and malformed escapes fail validation.

A query contains one or more `name=value` pairs separated by `&`.
Names are nonempty literal atoms only. Values may be empty or contain placeholders.
Keep pair order and duplicate names exactly as configured.
Reject empty pairs, bare names, multiple raw `=` characters per pair, and a trailing `&`.
The query marker cannot appear without a pair.
Literal reserved characters inside a name or value require percent encoding.

Static path segments must not decode once to `.` or `..`.
Malformed templates produce `E_TEMPLATE_SYNTAX`.
Unavailable capture references produce `E_CAPTURE_UNAVAILABLE` instead.

### Value encoding

Captured values contain decoded Unicode text, never pre-encoded URL text.
Encode each capture's UTF-8 bytes before insertion.
Preserve only ASCII unreserved bytes and encode every other byte as uppercase `%HH`.
This rule applies separately to path-segment and query-value tokens.
It uses the unreserved set defined by
[RFC 3986, section 2.3](https://www.rfc-editor.org/rfc/rfc3986#section-2.3).

Consequently, a captured `/` becomes `%2F`, `&` becomes `%26`, `+` becomes `%2B`,
and a space becomes `%20`. A captured `%2F` becomes `%252F`, not a slash.
Capture tokens never introduce query separators, path separators, or new placeholders.
Static literal escapes remain escapes and are not encoded again.

After rendering each complete path segment, reject it if its once-decoded bytes
equal `.` or `..`. This includes combinations of literals and captures.
Return `E_RESOLVE_DOT_SEGMENT` and no URL. Do not let a URL parser normalize that segment.
The same values remain valid inside query values or longer path segments.

Encoding preserves URL structure, not a destination application's interpretation.
A server may interpret encoded slashes or decode values again.
URLchemy cannot promise routing isolation inside an external service.

### Prefix append algorithm

Let `B` be the serialized base path and `P` the rendered destination path.
Both are strings whose separators already have their structural meaning.

1. If `P` is empty, keep `B` unchanged.
2. Otherwise, remove exactly one trailing `/` from `B`, if present.
3. Append `P` to the resulting base path.
4. Append the configured rendered query, if present.
5. Combine the fixed scheme and authority with that path and query.

This is explicit composition, not general relative-URL resolution.
Never normalize dot segments, decode substitutions, or remove internal slashes afterward.
The result must parse as an HTTP(S) URL with the fixed authority and composed path.
An unexpected structure change produces `E_RESOLVE_STRUCTURE`, never a browser request.

| Base | Template | Exact resolved URL |
| --- | --- | --- |
| `https://example.com/company` | `/dashboard` | `https://example.com/company/dashboard` |
| `https://example.com/company/` | `/dashboard` | `https://example.com/company/dashboard` |
| `https://example.com/company` | `/` | `https://example.com/company/` |
| `https://example.com/company/` | `""` | `https://example.com/company/` |
| `https://example.com/company` | `?view=summary` | `https://example.com/company?view=summary` |
| `https://example.com/company//` | `/dashboard` | `https://example.com/company//dashboard` |

## 7. Confirmed values and invalid input

The mandatory-value rule rejects an empty string or a string containing
only Unicode White_Space characters. Do not trim or normalize accepted values.
Preserve case, leading and trailing spaces, and Unicode composition exactly.

For this contract, White_Space is this explicit set:

```text
U+0009–U+000D, U+0020, U+0085, U+00A0, U+1680,
U+2000–U+200A, U+2028, U+2029, U+202F, U+205F, U+3000
```

Forbidden controls are U+0000–U+001F, U+007F–U+009F, U+2028, and U+2029.
This also rejects tabs and newlines inside otherwise nonblank values.
Check empty text first, forbidden controls second, and whitespace-only text third.
There is no additional user-value length limit in this contract.
Do not truncate text or a generated URL silently.

At input confirmation, invalid text produces `E_INPUT_REQUIRED` or `E_INPUT_CONTROL`.
Keep the draft, cursor, and all committed answers unchanged.
The user can correct the text or dismiss the session.
An explicitly selected suggestion confirms its stored value, not its label.
With no explicit suggestion selection, Enter confirms literal text.

Configured choice and suggestion values use the same text rules at startup.
Failures produce `E_VALUE_REQUIRED` or `E_VALUE_CONTROL` and reject the catalog.
Dot-only text is not globally invalid: its insertion context determines validity.
For example, `..` in a query value is data, while a complete `..` path segment fails resolution.
Resolution failure follows the approved `DestinationError` session transition.

## 8. Diagnostics and source locations

Every configuration diagnostic contains a stable code, a short message, a file
location, and a configuration pointer when the reader can identify the field.
Pointers use zero-based list indices and
[JSON Pointer escaping](https://www.rfc-editor.org/rfc/rfc6901).
An empty pointer identifies the root.

Locations use one-based lines and Unicode-scalar columns in the original text.
CRLF counts as one line break. Tabs count as one column, not visual tab stops.
The leading byte-order mark does not advance the first column.
Keep source spans before deserialization or normalization loses their positions.

For an invalid field, identify its scalar or collection start.
For an unknown field, identify its key. For a continuation conflict, identify
the offending option's `next` key. For a missing branch continuation, point to
that option's absent `next` field and identify its parent mapping.
For a missing field, identify its parent mapping and point to the absent field's path.
For a duplicate key, identify the second key and relate it to the first occurrence.
Syntax errors use the parser position, including EOF when appropriate.
Read failures identify the file without inventing a line or column.
Runtime errors identify the active step and relevant template origin when available.
They never echo captured values or the completed URL into diagnostics or logs.

| Code | Condition |
| --- | --- |
| `E_CONFIG_MISSING`, `E_CONFIG_READ`, `E_CONFIG_LIMIT` | File access or resource limit failure |
| `E_YAML_ENCODING`, `E_YAML_SYNTAX`, `E_YAML_DOCUMENT` | Invalid encoding, syntax, or document count |
| `E_YAML_FEATURE`, `E_YAML_DUPLICATE_KEY` | Forbidden YAML feature or duplicate key |
| `E_SCHEMA_UNKNOWN_FIELD`, `E_SCHEMA_TYPE`, `E_SCHEMA_REQUIRED` | Unknown field, wrong type, or absent field |
| `E_SCHEMA_VARIANT`, `E_SCHEMA_EMPTY_OPTIONS` | Invalid wrapper or empty option list |
| `E_TEXT_REQUIRED`, `E_TEXT_CONTROL`, `E_NAME`, `E_ALIAS` | Invalid presentation text, identifier, or alias characters |
| `E_FLOW_DUPLICATE`, `E_CAPTURE_DUPLICATE` | Repeated flow identifier or path capture |
| `E_CAPTURE_VALUE`, `E_CONTINUATION` | Inconsistent selection values or continuation mode |
| `E_BASE_URL`, `E_TEMPLATE_SYNTAX`, `E_CAPTURE_UNAVAILABLE` | Invalid base, template, or unavailable reference |
| `E_VALUE_REQUIRED`, `E_VALUE_CONTROL` | Invalid configured captured value |
| `E_INPUT_REQUIRED`, `E_INPUT_CONTROL` | Invalid input confirmation |
| `E_RESOLVE_DOT_SEGMENT`, `E_RESOLVE_STRUCTURE` | Unsafe or inconsistent rendered destination |

Stop after an encoding, syntax, duplicate-key, or forbidden-feature failure.
Report the first such failure in source order.
Otherwise collect independent structural diagnostics in source order.
Perform domain validation only after structural validation succeeds.
Suppress dependent diagnostics, such as a missing reference caused by an invalid capture name.
Domain diagnostics also follow source order. At most 50 diagnostics appear,
with a separate indication that more diagnostics exist.
Automatic alias-conflict diagnostics are not part of this list.

Representative messages use fixed wording without private contents:

```text
E_SCHEMA_TYPE: Expected a string.
E_CONTINUATION: Define either one common continuation or a continuation for every option.
E_CAPTURE_UNAVAILABLE: This destination references a capture unavailable on its path.
E_INPUT_REQUIRED: Enter a nonblank value.
E_RESOLVE_DOT_SEGMENT: A rendered path segment must not be a dot segment.
```

## 9. Fixtures and observable results

Every YAML file in `yaml-url-contract/` is a complete standalone configuration,
including invalid examples. None relies on another file or an implicit fragment.
The adjacent `expectations.json` records semantic journeys and exact results.
Choice and suggestion indices in that manifest are zero-based file positions,
not proposed user-facing shortcuts or internal runtime identifiers.

| Valid fixture | Coverage |
| --- | --- |
| `fixed.yaml` | Emoji, input-free completion, prefix joining, and base-only destinations |
| `linear.yaml` | Suggestions, literal confirmation, Unicode, reserved text, and capture order differing from URL order |
| `common.yaml` | A capturing selection with one shared continuation |
| `branching.yaml` | The complete accepted Datadog example, unchanged |

The manifest also contains runtime rejection cases and complete invalid-file expectations.
Expected codes and pointers are contract assertions. Parser-specific prose is not an assertion.
The manifest also specifies exact diagnostic lines and columns for these files.
Line and column preservation still needs tests against the selected production parser.
Fixtures alone do not demonstrate that an implementation satisfies this contract.

## 10. Approval boundary and verification targets

The maintainer approved these policies on 2026-11-14:

1. One native macOS configuration path, manual setup, and diagnostic-only failure states.
2. The strict YAML subset, field grammar, resource limits, and selection syntax.
3. HTTP(S)-only ASCII bases without credentials, base queries, or fragments.
4. The literal-template grammar, explicit prefix append, and percent-encoding rules.
5. Rejecting blank/control text while preserving all other text, including surrounding spaces.
6. Rejecting complete dot segments instead of silently changing their URL meaning.

This design changes no navigation, branch correction, immediate completion, or session reset behavior.
The fixtures preserve both exact URLs from the product definition.

Future implementation tests must cover the fixture manifest and these additional boundaries:

- YAML 1.2 scalar resolution, duplicate decoded keys, aliases, tags, and document limits.
- Exact limit boundaries, invalid UTF-8, and deeply nested input without stack exhaustion.
- Missing files, read failures, empty catalogs, and atomic configuration rejection.
- Base normalization, IPv6 and ports, malformed percent escapes, and encoded dot spellings.
- Mixed literal/capture dot segments, encoded literal braces, and empty static query values.
- Repeated query names, internal empty path segments, and every prefix-boundary case above.
- Missing fields, unexpected nulls, non-capturing selections, and terminal options.
- Character columns, CRLF, missing-field origins, and privacy of diagnostic messages.
- Unchanged-answer preservation and changed-answer invalidation from the approved session model.

Review method: in-session self-review against the product and approved domain design.
No subagent or independent reviewer participates in this work.
Approval does not authorize Cargo setup, implementation, commits, or publication.
