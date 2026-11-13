---
name: rust-development
description: Implement, debug or review Rust code with version-correct APIs, small tests, and explicit verification. Use for Cargo, ownership, error handling, async work or Rust dependency changes.
---

# Rust development

1. Read the project's AGENTS.md, Cargo.toml, Cargo.lock and rust-toolchain.toml if present. If there is no Cargo workspace, report that rather than creating one without a request.
2. Determine the selected toolchain and actual dependency versions. Prefer official documentation and the source of that version. Never invent APIs or update dependencies just to make an example compile.
3. Agree on observable behavior. Add a focused regression test for a bug; demonstrate the failure when feasible, then implement the smallest fix. Do not remove user code because it predates a test.
4. Keep pure domain logic separate from UI and native effects. Use typed errors at recoverable boundaries; avoid panics/unwrap for user input, file contents and normal I/O failure. Do not add unsafe code or runtime/framework dependencies without explaining why.
5. Avoid blocking UI threads. Make state ownership, subscriptions, cancellation and task lifetime explicit. Check the chosen UI framework's current context rules.
6. Use rustfmt and clippy. Once a workspace exists, run:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --locked
   ```

   Use the repository's narrower documented commands if the full workspace needs unavailable system dependencies. Report exactly what was skipped and why. These commands may compile and execute dependency build scripts/tests; honor shell approval rules.
7. Inspect the diff for unrelated edits and lockfile churn. Report checks actually run, not expected results.

For a maintainer new to Rust, explain ownership or dependency tradeoffs in plain language when they materially affect the design. Do not flood routine changes with a Rust tutorial.

If cargo is not on PATH but rustup is installed, use `$HOME/.cargo/bin/cargo`. Do not modify shell profiles or install toolchains without approval.
