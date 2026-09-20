#!/bin/bash
# D20 — WO-P2-020 GUI verification (honest status for Ctrl+P without a workspace).
#
# OFFLINE-HONEST SCOPE: F-A4 is exactly the no-workspace state — this scene
# runs with a FRESH EMPTY state DB (no recent_workspace row), unlike the
# donor-seeded D17/D18 scenes. The reference Files palette behavior with a
# workspace open is already evidenced (D13, WO-P2-010).
#
#   01 entry baseline — fresh no-workspace state
#   02 Ctrl+P → the Files palette attempt with NO workspace: honest status
#      "Select a workspace before searching files." (NOT a silent no-op)
#   03 settle frame (status still visible)
#   04 Ctrl+K — the Unified palette still works everywhere (regression guard)
#
# PASS: md5 frames + VLM read of 02 shows the honest status text.
# Usage: d20-files-palette-status.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d20}"
DISP=":110"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

# FRESH EMPTY STATE — deliberately NO donor (the no-workspace precondition)
D20DATA=/tmp/d20-data
rm -rf "$D20DATA"; mkdir -p "$D20DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D20=$(mktemp -d /tmp/d20-home.XXXXXX)
mkdir -p "$HOME_D20/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D20" XDG_RUNTIME_DIR="$HOME_D20/xdg" \
  CODEX_RS_DATA_DIR="$D20DATA" \
  CODEX_HOME="$D20DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }

rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { echo "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
  cap "$OUT/_probe.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
  if [ "${SZ:-0}" -gt 30000 ]; then
    echo "UI rendered after $((i*5))s (frame $SZ bytes)"
    rendered=1
    break
  fi
done
[ "$rendered" = "1" ] || echo "WARN — no >30KB frame in 150s; running scene anyway"
sleep 5

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

echo "--- [01] entry baseline (no workspace)"
key Escape; sleep 2
cap "$OUT/d20-01-entry.png"; echo "B01 $(md5 "$OUT/d20-01-entry.png")"

echo "--- [02] Ctrl+P with NO workspace → honest status"
key ctrl+p
sleep 3
cap "$OUT/d20-02-ctrlp-status.png"; echo "B02 $(md5 "$OUT/d20-02-ctrlp-status.png")"

echo "--- [03] settle (status persists / state stable)"
sleep 3
cap "$OUT/d20-03-settle.png"; echo "B03 $(md5 "$OUT/d20-03-settle.png")"

echo "--- [04] Ctrl+K still opens the Unified palette (regression guard)"
key ctrl+k
sleep 3
cap "$OUT/d20-04-unified-open.png"; echo "B04 $(md5 "$OUT/d20-04-unified-open.png")"

echo "--- teardown"
kill $APP_PID 2>/dev/null; pkill -P $APP_PID 2>/dev/null
sleep 1
pkill -x picom 2>/dev/null
kill $XPID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
echo "d20 done (artifacts in $OUT)"
