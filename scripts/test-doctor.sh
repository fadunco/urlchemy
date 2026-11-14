#!/bin/sh
# Deterministic regression tests; no real installations, credentials or network.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/urlchemy-doctor-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
mkdir -p "$tmp/bin"
cat > "$tmp/bin/stub" <<'STUB'
#!/bin/sh
case "${0##*/}" in
  git) printf 'git version test\n' ;;
  gh) printf 'github.com: test account authenticated\n' ;;
  cargo|rustc) printf 'Rust tool test version\n' ;;
  opencode) printf '%s\n' "opencode v${DOCTOR_TEST_OPENCODE_VERSION:-2.2.7}" ;;
  xcode-select) printf '/Applications/Xcode.app/Contents/Developer\n' ;;
  xcodebuild) printf 'Xcode test version\n' ;;
  xcrun)
    case "$*" in
      '--find metal') printf '/mock/Xcode/usr/bin/metal\n' ;;
      'metal --version')
        if test "${DOCTOR_TEST_METAL:-available}" = missing; then
          printf 'cannot execute metal: missing Metal Toolchain\n' >&2
          exit 1
        fi
        printf 'Apple metal test version\n'
        ;;
      *) printf 'Unexpected xcrun arguments: %s\n' "$*" >&2; exit 1 ;;
    esac
    ;;
  *) exit 1 ;;
esac
STUB
chmod +x "$tmp/bin/stub"
for command in git gh cargo rustc opencode xcode-select xcodebuild xcrun; do
  ln -s stub "$tmp/bin/$command"
done

run_case() {
  name=$1
  expected=$2
  metal=$3
  version=$4
  status=0
  (cd "$root" && PATH="$tmp/bin:/usr/bin:/bin" OPENCODE_BIN=opencode \
    DOCTOR_TEST_METAL="$metal" DOCTOR_TEST_OPENCODE_VERSION="$version" \
    sh scripts/doctor.sh) > "$tmp/$name.log" 2>&1 || status=$?
  if test "$status" -ne "$expected"; then
    cat "$tmp/$name.log" >&2
    printf 'FAIL: %s (expected exit %s, got %s)\n' "$name" "$expected" "$status" >&2
    exit 1
  fi
  printf 'PASS: %s\n' "$name"
}

run_case healthy 0 available 2.2.7
# Regression: xcrun --find metal succeeds but its executable is only a stub.
run_case missing-metal-toolchain 1 missing 2.2.7
grep -q 'MISSING: Metal compiler execution' "$tmp/missing-metal-toolchain.log"
run_case legacy-opencode 1 available 1.19.5
grep -q 'selected CLI is not V2' "$tmp/legacy-opencode.log"
