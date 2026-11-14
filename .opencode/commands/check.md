---
description: Verify the current work and report evidence, without publishing
---

Read AGENTS.md and run `sh scripts/doctor.sh` and `sh scripts/test-doctor.sh`. If Cargo.toml exists, load rust-development and run the documented formatting, clippy and test checks after any required permission approvals. If it does not exist, explicitly report that application checks are not yet applicable. Inspect the diff if this is a git repository. Do not edit project files, install dependencies, start native UI, commit, or publish. The diagnostic regression test may create and remove its own temporary fixtures. Summarize actual results and blockers. Additional focus: $ARGUMENTS
