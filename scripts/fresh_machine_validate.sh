#!/usr/bin/env bash
# Fresh-machine validation for a released Flauz.app archive (GUI-006).
#
# Simulates a fresh machine: isolated HOME, XDG dirs, and CODEX_HOME;
# no source checkout; only the downloaded release artifacts plus the
# separately installed Codex runtime.
#
# Usage:
#   RELEASE_TAG=v0.1.0-rc.13 RUNTIME_BIN=/path/to/codex \
#     bash scripts/fresh_machine_validate.sh
#
# On a stock desktop the documented runtime dependencies are present.
# On a minimal headless sandbox, provide the missing client libraries
# via EXTRA_LIBS (an LD_LIBRARY_PATH entry) extracted from stock
# distribution packages; nothing on the host is modified.
set -euo pipefail

RELEASE_TAG="${RELEASE_TAG:-v0.1.0-rc.13}"
RUNTIME_BIN="${RUNTIME_BIN:-/home/z/runtime/codex}"
EXTRA_LIBS="${EXTRA_LIBS:-}"
STAGE="$(mktemp -d)"
trap 'rm -rf -- "$STAGE"' EXIT

run_app() {
  if [[ -n "$EXTRA_LIBS" ]]; then
    LD_LIBRARY_PATH="$EXTRA_LIBS${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" "$@"
  else
    "$@"
  fi
}

echo "== [1] download =="
cd "$STAGE"
base="https://github.com/payswapdotorg/Flauz.app/releases/download/${RELEASE_TAG}"
curl -fSL --retry 3 -o SHA256SUMS.txt "${base}/SHA256SUMS.txt"
asset="codexrs-${RELEASE_TAG}-linux-x86_64.tar.gz"
curl -fSL --retry 3 -o "$asset" "${base}/${asset}"

echo "== [2] checksum verify =="
expected="$(grep -E "[ /]\\.?/?${asset}\$" SHA256SUMS.txt | awk '{print $1}')"
actual="$(sha256sum "$asset" | awk '{print $1}')"
test -n "$expected"
test "$expected" = "$actual"
echo "sha256 OK: $actual"

echo "== [3] install (extract portable archive) =="
install -d fresh-root
tar -xzf "$asset" -C fresh-root
app_dir="fresh-root/codexrs-${RELEASE_TAG}-linux-x86_64"
test -x "${app_dir}/codexrs"

echo "== [4] bounded runtime information =="
run_app "${app_dir}/codexrs" info

echo "== [5] fresh HOME + XDG + CODEX_HOME; desktop integration =="
export HOME="${STAGE}/home"
export XDG_DATA_HOME="${STAGE}/data"
export XDG_RUNTIME_DIR="${STAGE}/runtime"
export CODEX_HOME="${STAGE}/codex"
install -d -m 0700 "$HOME" "$XDG_DATA_HOME" "$XDG_RUNTIME_DIR" "$CODEX_HOME"
run_app "${app_dir}/codexrs" --install-desktop-entry
entry="${XDG_DATA_HOME}/applications/com.codexrs.CodexRS.desktop"
test -f "$entry" || entry="${HOME}/.local/share/applications/com.codexrs.CodexRS.desktop"
test -f "$entry"
grep -q "Exec=" "$entry"
echo "desktop entry OK: $entry"

echo "== [6] runtime probe (fresh machine, Universal runtime) =="
run_app "${app_dir}/codexrs" probe --codex-bin "$RUNTIME_BIN" --codex-home "$CODEX_HOME"

echo "== [7] restart: second probe on the same fresh home =="
run_app "${app_dir}/codexrs" probe --codex-bin "$RUNTIME_BIN" --codex-home "$CODEX_HOME"

echo "FRESH_MACHINE_VALIDATION_OK"
