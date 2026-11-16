# Framework selection and shortcut configuration

Status: approved by the maintainer on 2026-11-15, including the framework version,
optional shortcut block, and detailed validation policy.

## Scope and selected approaches

This amendment extends the [YAML contract](2026-11-14-yaml-url-contract-design.md)
and [interaction contract](2026-11-15-interaction-contract-design.md).
It selects dependencies for later implementation, not for installation now.
It adds no application code, native registration, or permission request.

| Decision | Selected approach | Alternative and tradeoff |
| --- | --- | --- |
| Framework source | Published `gpui-kit = "=0.9.1"` | A Git revision requires new compatibility evidence without a demonstrated need |
| Missing shortcut settings | Use documented defaults | Requiring an invocation binding would invalidate existing fixtures |
| Configuration location | Optional `shortcuts` mapping in the existing file | A second file adds precedence and lifecycle rules |

The maintainer selected these approaches. Keep Rust `1.100.0`.
GPUI Kit 0.9.1 depends on `gpui-pre = "=0.3.12"` and re-exports GPUI.
The future application uses GPUI Kit's default component/assets features.
Its test dependency uses the same version with `test-support`.
Keep the UI-independent core free of GPUI dependencies.
Retain the resolved dependency graph in `Cargo.lock` during implementation.

## Approved configuration shape and behavior

```yaml
shortcuts:
  invoke: "ctrl-alt-space"
  launch: "enter"

navigation: []
```

`navigation` remains required. `shortcuts` is optional.
The defaults are `ctrl-alt-space` for invocation and `enter` for confirmation.
The launch binding replaces Enter's palette confirmation role rather than adding
a second binding. It applies to navigation activation, closed choices, and input
confirmation. It does not replace native button activation or IME composition handling.
Immediate alias activation and terminal completion require no additional confirmation.
Footer hints must reflect the configured binding.

Invocation is system-wide. Launch confirmation applies only within the palette.
The application reads both settings once at startup from the existing configuration
file. Changes require restarting the application.
The application must not silently change a configured or default shortcut.

## Approved validation policy

Use the existing strict YAML rules for duplicate keys, scalar types, limits,
unknown fields, and explicit nulls. Within `shortcuts`, allow only `invoke` and `launch`.
Omitted members receive their individual defaults. An empty mapping uses both defaults.
Reject null, sequences, non-string bindings, empty strings, and whitespace around bindings.

Use this deliberately bounded syntax instead of accepting every GPUI parser extension:

- A binding contains one simultaneous key combination, not a sequence of combinations.
- Modifiers are lowercase `ctrl`, `alt`, `shift`, and `cmd`, separated by `-`.
- Each modifier occurs at most once. Modifier order has no semantic meaning.
- Keys are lowercase `a` through `z`, digits `0` through `9`, `enter`, `space`, or `f1` through `f12`.
- Invocation requires at least `ctrl` or `cmd`.
- Launch accepts bare `enter`. Other launch combinations require at least `ctrl` or `cmd`.
- Reject `fn`, modifier aliases, glyph spellings, physical keycodes, and test-event suffixes.

Thus `cmd-enter` is a valid alternative launch binding.
Do not trim, case-fold, or silently repair configuration text.
Compare parsed combinations rather than their modifier spelling order.

Reject these internal conflicts:

- Invocation and launch resolve to the same combination.
- Either binding equals a navigation ordinal, `cmd-1` through `cmd-9`.
- Either binding equals a protected editing or application command: `cmd-a`, `cmd-c`,
  `cmd-v`, `cmd-x`, `cmd-z`, `cmd-shift-z`, `cmd-q`, `cmd-w`, `cmd-h`,
  `cmd-alt-h`, or `cmd-m`.

The bounded key set excludes Escape, Back, arrow, Tab, and deletion bindings.
Shortcut conflict validation does not enable deferred alias-conflict validation.

Use `E_SHORTCUT_SYNTAX` for invalid combination syntax and
`E_SHORTCUT_CONFLICT` for the internal conflicts above.
Use existing schema diagnostics for wrong field types and unknown fields.
Attach each diagnostic to its configuration field and available source location.
Do not publish partially validated application settings or an executable catalog.

## Native boundary and remaining work

GPUI's `App::bind_keys` manages application keybindings, not system-wide registration.
Its dynamic `KeyBinding::load` returns a parse error instead of using the panicking
static constructor. Use validated settings at this boundary.
The native adapter must map the invocation combination to the chosen macOS registration API.

The OS can reject a syntactically valid invocation binding.
Do not claim that configuration validation proves system-wide availability.
A registration failure must identify the shortcut and explain correction and restart.
Provide a way to reach that diagnostic without the failed shortcut.
The native implementation plan must select and verify that access path and registration API.
Do not replace registration with an unapproved keyboard monitor or permission request.

Before implementation, test defaults, partial mappings, strict rejection,
modifier-order equivalence, internal conflicts, and unchanged legacy fixture behavior.
Native registration, keyboard layouts, activation, and Spaces require separate verification.

## Evidence and approval boundary

On 2026-11-15, the crates.io API identified 0.9.1 as the latest published,
non-yanked GPUI Kit release. Published manifests and source confirmed the versions,
features, and keybinding signatures above.
The [earlier probe](../../testing-probe.md) used GPUI Kit 0.9.1 with Rust 1.100.0.
It did not verify native invocation or the production view.

Sources: [release metadata](https://crates.io/api/v1/crates/gpui-kit/0.9.1),
[package source](https://docs.rs/crate/gpui-kit/0.9.1/source/Cargo.toml), and
[GPUI source](https://docs.rs/crate/gpui-pre/0.3.12/source/src/keymap/binding.rs).

The author performed the review in this session. No independent reviewer participated.
Approval of this amendment does not authorize installation, implementation,
native UI launch, commits, or publication.
