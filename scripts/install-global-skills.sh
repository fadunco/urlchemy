#!/bin/sh
# Install only our general-purpose skills, without overwriting different files.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
destination="${XDG_CONFIG_HOME:-$HOME/.config}/opencode/skills"
for skill in rust-development github-workflow; do
  source="$root/.opencode/skills/$skill/SKILL.md"
  target="$destination/$skill/SKILL.md"
  if test -e "$target" && ! cmp -s "$source" "$target"; then
    printf 'Refusing to overwrite different skill: %s\n' "$target" >&2
    exit 1
  fi
done
for skill in rust-development github-workflow; do
  mkdir -p "$destination/$skill"
  cp "$root/.opencode/skills/$skill/SKILL.md" "$destination/$skill/SKILL.md"
  printf 'Installed %s\n' "$destination/$skill/SKILL.md"
done
