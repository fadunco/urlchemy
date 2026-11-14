# URLchemy

A lightweight, keyboard-driven URL launcher for macOS. Navigate complex web applications by interactively building URLs instead of clicking through menus, using customizable local configuration files.

**Status:** a UI-independent Rust core implements configuration validation, session transitions, and URL resolution. The native macOS application is not implemented yet.

Initial UI direction: Rust + GPUI + GPUI Kit. This is an open-source project under the MIT license, with third-party material under its own licenses.

The working [product definition](docs/product.md) records agreed functional decisions and open questions.
See the [roadmap](ROADMAP.md) for the proposed sequence from domain design to a verified MVP.

## Core development

The workspace contains the `urlchemy-core` library, with Rust pinned in
`rust-toolchain.toml` and resolved dependencies retained in `Cargo.lock`.
The core has no GPUI dependency and does not open a browser or fetch external data.

Run these checks from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The tests execute the [approved YAML fixtures and URL expectations](docs/superpowers/specs/yaml-url-contract/expectations.json)
through the production engine. They also cover configuration limits, diagnostics,
corrections, cancellation, filtering, and scoped alias lookup.
These checks do not establish native shortcut, focus, overlay, or browser behavior.

The [configuration contract](docs/superpowers/specs/2026-11-14-yaml-url-contract-design.md)
defines one manually edited file at
`~/Library/Application Support/URLchemy/config.yaml` for the future application.
The library reads only the path or bytes supplied by its caller.
It does not create that directory, generate configuration, or read personal data during tests.
There is no launcher binary yet.

## Agent development setup

Read [the harness guide](docs/harness.md), then run:

```sh
sh scripts/doctor.sh
sh scripts/test-doctor.sh
```

The project includes OpenCode V2 configuration, project instructions, skills and `/check`, `/review`, `/prepare-pr` commands. General Rust/GitHub skills can also be installed globally using `sh scripts/install-global-skills.sh`.

The maintainer also uses global **Superpowers** and **find-skills**. See [the toolkit guide](docs/agent-toolkit.md) for installation, pinned versions, update ownership and project-specific approval rules. A clone does not install these global tools automatically.

Framework skills are checked in as a reviewed snapshot. Use `scripts/install-gpui-skills.sh` only when bootstrapping a copy missing those skills, not to update an existing installation.

A temporary keyboard/focus and Metal screenshot experiment established a working agent-testing path; see [the testing probe report](docs/testing-probe.md). The experiment did not scaffold the application.

Repository: [fadunco/urlchemy](https://github.com/fadunco/urlchemy). Commits and remote mutations require explicit maintainer approval; see `AGENTS.md`.
