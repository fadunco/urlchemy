# URLchemy

A lightweight, keyboard-driven URL launcher for macOS. Navigate complex web applications by interactively building URLs instead of clicking through menus, using customizable local configuration files.

**Status:** development-harness preparation only; no application implementation yet.

Initial UI direction: Rust + GPUI + GPUI Kit. This is an open-source project under the MIT license, with third-party material under its own licenses.

The working [product definition](docs/product.md) records agreed functional decisions and open questions.
See the [roadmap](ROADMAP.md) for the proposed sequence from domain design to a verified MVP.

## Agent development setup

Read [the harness guide](docs/harness.md), then run:

```sh
sh scripts/doctor.sh
```

The project includes OpenCode V2 configuration, project instructions, skills and `/check`, `/review`, `/prepare-pr` commands. General Rust/GitHub skills can also be installed globally using `sh scripts/install-global-skills.sh`.

Framework skills are checked in as a reviewed snapshot. Use `scripts/install-gpui-skills.sh` only when bootstrapping a copy missing those skills, not to update an existing installation.

Repository: [fadunco/urlchemy](https://github.com/fadunco/urlchemy). Commits and remote mutations require explicit maintainer approval; see `AGENTS.md`.
