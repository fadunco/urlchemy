#!/bin/sh
# Install reviewed upstream skill snapshots. No package-manager code execution.
# Optional first argument: an existing checkout at the exact approved revision.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
revision=6653d1c024b5461461143c90b005d80089cc085f
checkout=${1:-}
if test -z "$checkout"; then
  checkout=$(mktemp -d "${TMPDIR:-/tmp}/urlchemy-gpui-skills.XXXXXX")
  printf 'Fetching reviewed source into %s\n' "$checkout"
  git -C "$checkout" init -q
  git -C "$checkout" remote add origin https://github.com/longbridge/gpui-kit.git
  git -C "$checkout" fetch --depth 1 origin "$revision"
  git -C "$checkout" checkout --detach FETCH_HEAD
fi
actual=$(git -C "$checkout" rev-parse HEAD)
if test "$actual" != "$revision"; then
  printf 'Expected %s, got %s; refusing install.\n' "$revision" "$actual" >&2
  exit 1
fi
# Preflight all destinations before copying anything.
for skill in gpui-kit gpui-kit-design-guides; do
  test -f "$checkout/skills/$skill/SKILL.md"
  if test -e "$root/.opencode/skills/$skill"; then
    printf 'Already exists: %s; review changes manually, not an automatic update.\n' "$skill" >&2
    exit 1
  fi
done
mkdir -p "$root/.opencode/skills" "$root/third-party/gpui-kit"
for skill in gpui-kit gpui-kit-design-guides; do
  cp -R "$checkout/skills/$skill" "$root/.opencode/skills/$skill"
done
cp "$checkout/LICENSE-APACHE" "$root/third-party/gpui-kit/LICENSE-APACHE"
cp "$checkout/LICENSE-DOCS.md" "$root/third-party/gpui-kit/LICENSE-DOCS.md"
printf 'Installed GPUI Kit skills at %s. Preserve THIRD_PARTY_NOTICES.md and license files.\n' "$revision"
