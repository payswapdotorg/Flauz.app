#!/bin/bash
# D17 — WO-P2-017 GUI verification (palette + shortcuts-overlay close restore focus).
#
# FOCUS PROBE METHOD: after closing the palette/overlay, type one character.
# If (and only if) focus was restored to the composer, the character lands in
# the composer input; a stranded/dropped focus shows nowhere (silent no-op)
# or in the wrong surface. Frame deltas + VLM reads carry the evidence; the
# deterministic restore semantics are covered by the two focused unit tests
# on the branch.
#
#   01 entry baseline (footer "App-server online")
#   02 composer focus (D9 click ladder) — pre-palette state
#   03 Ctrl+K — command palette OPENS
#   04 Escape — palette closes (restore fires)
#   05 TYPE "x" — the focus probe: lands in the composer iff restored
#   06 clear composer
#   07 Ctrl+/ — keyboard-shortcuts overlay OPENS
#   08 Escape (empty query) — overlay closes (restore fires)
#   09 TYPE "y" — second focus probe
#   10 settle frame
#
# PASS: md5 frame sequence + VLM read of 05/09 shows the probe char in the
# composer after each close.
# Usage: d17-palette-close-focus.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d17}"
DISP=":107"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D17DATA=/tmp/d17-data
rm -rf "$D17DATA"; mkdir -p "$D17DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D17DATA/state.sqlite3"
mkdir -p "$D17DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D17=$(mktemp -d /tmp/d17-home.XXXXXX)
mkdir -p "$HOME_D17/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D17" XDG_RUNTIME_DIR="$HOME_D17/xdg" \
  CODEX_RS_DATA_DIR="$D17DATA" \
  CODEX_HOME="$D17DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
# D19 lesson (2026-09-21): re-assert windowfocus before every keyboard burst —
# the page can drop focus between bursts under load and a lost key is a
# silent scene corruption. Frame sequence B01-B10 is UNCHANGED by this.
refocus() {
  W=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
  [ -n "$W" ] && DISPLAY=$DISP xdotool windowfocus --sync "$W" 2>/dev/null || true
}
key() { refocus; DISPLAY=$DISP xdotool key "$1"; }
type_text() { refocus; DISPLAY=$DISP xdotool type --delay 60 "$1"; }
ladder() { for Y in 849 829 809 789; do DISPLAY=$DISP xdotool mousemove 802 $Y click 1; sleep 0.3; done; }

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

# D19 lesson (2026-09-21): fresh-profile "What's New" promo modal swallows
# the keyboard until clicked. D17DATA is recreated fresh each run, so the
# promo CAN appear. Defensive dismissal (no-op when absent); the stray click
# lands harmlessly in the main surface. No numbered frame — B01-B10 contract
# unchanged.
DISPLAY=$DISP xdotool mousemove 580 619 click 1
sleep 2
DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
sleep 1

echo "--- [01] entry baseline"
key Escape; sleep 2
cap "$OUT/d17-01-entry.png"; echo "B01 $(md5 "$OUT/d17-01-entry.png")"

echo "--- [02] composer focus (ladder)"
ladder
sleep 1
cap "$OUT/d17-02-composer-focus.png"; echo "B02 $(md5 "$OUT/d17-02-composer-focus.png")"

echo "--- [03] Ctrl+K — palette OPEN"
key ctrl+k
sleep 3
cap "$OUT/d17-03-palette-open.png"; echo "B03 $(md5 "$OUT/d17-03-palette-open.png")"

echo "--- [04] Escape — palette CLOSE (restore fires)"
key Escape
sleep 2
cap "$OUT/d17-04-palette-closed.png"; echo "B04 $(md5 "$OUT/d17-04-palette-closed.png")"

echo "--- [05] TYPE x — focus probe (must land in composer)"
type_text "x"
sleep 1.5
cap "$OUT/d17-05-probe-x.png"; echo "B05 $(md5 "$OUT/d17-05-probe-x.png")"

echo "--- [06] clear composer"
key ctrl+a; sleep 0.5; key BackSpace
sleep 1
cap "$OUT/d17-06-cleared.png"; echo "B06 $(md5 "$OUT/d17-06-cleared.png")"

echo "--- [07] Ctrl+/ — shortcuts overlay OPEN"
key ctrl+slash
sleep 3
cap "$OUT/d17-07-overlay-open.png"; echo "B07 $(md5 "$OUT/d17-07-overlay-open.png")"

echo "--- [08] Escape (empty query) — overlay CLOSE"
key Escape
sleep 2
cap "$OUT/d17-08-overlay-closed.png"; echo "B08 $(md5 "$OUT/d17-08-overlay-closed.png")"

echo "--- [09] TYPE y — second focus probe"
type_text "y"
sleep 1.5
cap "$OUT/d17-09-probe-y.png"; echo "B09 $(md5 "$OUT/d17-09-probe-y.png")"

echo "--- [10] settle"
key ctrl+a; sleep 0.5; key BackSpace; sleep 1
cap "$OUT/d17-10-settle.png"; echo "B10 $(md5 "$OUT/d17-10-settle.png")"

echo "--- teardown"
kill $APP_PID 2>/dev/null; pkill -P $APP_PID 2>/dev/null
sleep 1
pkill -x picom 2>/dev/null
kill $XPID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
echo "d17 done (artifacts in $OUT)"
