---
description: Review changes for correctness, safety, keyboard UX and missing tests
---

Read AGENTS.md and inspect the current diff, or the requested target: $ARGUMENTS. This command requests an in-session self-review, not an independent review. Review without editing files, delegating, committing or publishing; this narrow scope takes precedence over broader workflow defaults. For Rust changes load rust-development; for GPUI changes load gpui-kit and gpui-kit-design-guides; for native launcher behavior consult docs/harness.md. Prioritize correctness, API/version drift, URL escaping, config validation, UI-thread blocking, focus/key routing and missing tests. Report actionable findings with file locations and severity; distinguish verified defects from uncertainties. If a plan requires an independent review gate, say that it still requires separate approval. If there is no git repository yet, say so rather than initializing one.
