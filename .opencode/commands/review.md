---
description: Review changes for correctness, safety, keyboard UX and missing tests
---

Read AGENTS.md and inspect the current diff, or the requested target: $ARGUMENTS. Review without editing files, delegating, committing or publishing. For Rust changes load rust-development; for GPUI changes load gpui-kit and gpui-kit-design-guides; for native launcher behavior consult docs/harness.md. Prioritize correctness, API/version drift, URL escaping, config validation, UI-thread blocking, focus/key routing and missing tests. Report actionable findings with file locations and severity; distinguish verified defects from uncertainties. If there is no git repository yet, say so rather than initializing one.
