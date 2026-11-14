#!/bin/sh
# Read-only environment checks. Exit nonzero while prerequisites are missing.
set -u
missing=0

check() {
  label=$1
  shift
  if "$@"; then
    printf 'OK: %s\n' "$label"
  else
    printf 'MISSING: %s\n' "$label"
    missing=1
  fi
}

printf 'URLchemy environment (no installation or authentication performed)\n'
check 'git' git --version
check 'GitHub CLI' gh --version
check 'GitHub sign-in (user action if missing)' gh auth status

if command -v cargo >/dev/null 2>&1; then
  cargo=cargo
  rustc=rustc
else
  cargo="$HOME/.cargo/bin/cargo"
  rustc="$HOME/.cargo/bin/rustc"
fi
check 'Rust compiler' "$rustc" --version
check 'Cargo' "$cargo" --version
check 'rustfmt' "$cargo" fmt --version
check 'clippy' "$cargo" clippy --version

printf '\nSelected developer directory:\n'
check 'Xcode developer directory selection' xcode-select -p
check 'full Xcode (not only Command Line Tools)' xcodebuild -version
# Xcode can expose a metal shim before the optional toolchain is installed.
# Finding the path alone is not evidence that the compiler can execute.
check 'Metal compiler execution' xcrun metal --version

opencode=${OPENCODE_BIN:-opencode}
printf '\nOpenCode (project configuration requires V2):\n'
check 'OpenCode CLI' "$opencode" --version
if command -v "$opencode" >/dev/null 2>&1; then
  case "$("$opencode" --version 2>/dev/null)" in
    'opencode v2.'*|2.*) ;;
    *) printf 'MISSING: selected CLI is not V2; set OPENCODE_BIN to the app-bundled CLI (see docs/harness.md)\n'; missing=1 ;;
  esac
fi

printf '\nProject assets:\n'
check 'GPUI Kit skill' test -f .opencode/skills/gpui-kit/SKILL.md
check 'GPUI Kit design skill' test -f .opencode/skills/gpui-kit-design-guides/SKILL.md
if test -f Cargo.toml; then
  printf 'Cargo workspace present; use /check for application verification.\n'
else
  printf 'NOT APPLICABLE: no Cargo workspace yet; application checks have not run.\n'
fi
exit "$missing"
