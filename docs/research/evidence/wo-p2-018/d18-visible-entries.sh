#!/bin/bash
# D18 — WO-P2-018 GUI verification (visible palette entry, Activity bell,
# label parity for icon-only controls).
#
#   01 entry baseline (footer "App-server online")
#   02 title-bar crop — the two new visible entries (Search + Bell icons)
#   03 CLICK the command-palette entry button → palette OPENS (Unified)
#   04 Escape — close
#   05 CLICK the Activity bell → Activity view OPENS (empty state:
#      "No chats need attention" guidance)
#   06 Escape — close
#   07 create chat A (composer ctrl+Return) + Ctrl+Shift+U → unread dot
#   08 HOVER the sidebar dot → tooltip "Unread activity"
#   09 Settings → Archived chats → the single deletion row action shows the
#      visible "Delete" label (was icon-only)
#   10 settle frame
#
# PASS: md5 frames + VLM reads of 02/03/05/08/09.
# NOTE: PX_SEARCH / PX_BELL are calibrated against the 1440x900 lab frame;
# the title bar is ~40px tall, entries sit left of the window controls.
# Usage: d18-visible-entries.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d18}"
DISP=":108"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D18DATA=/tmp/d18-data
rm -rf "$D18DATA"; mkdir -p "$D18DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D18DATA/state.sqlite3"
mkdir -p "$D18DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D18=$(mktemp -d /tmp/d18-home.XXXXXX)
mkdir -p "$HOME_D18/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D18" XDG_RUNTIME_DIR="$HOME_D18/xdg" \
  CODEX_RS_DATA_DIR="$D18DATA" \
  CODEX_HOME="$D18DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
capbar() { ffmpeg -y -loglevel error -f x11grab -video_size 500x60 -i "$DISP+940+0" -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_text() { DISPLAY=$DISP xdotool type --delay 60 "$1"; }
ladder() { for Y in 849 829 809 789; do DISPLAY=$DISP xdotool mousemove 802 $Y click 1; sleep 0.3; done; }
click() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }

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
# geometry-derived entry coordinates (the window is NOT at (0,0): 83,41 on this box)
GEO=$(xwininfo -display $DISP -id "$WID" 2>/dev/null)
WX=$(echo "$GEO" | rg "Absolute upper-left X" | rg -o "[0-9-]+$")
WY=$(echo "$GEO" | rg "Absolute upper-left Y" | rg -o "[0-9-]+$")
WW=$(echo "$GEO" | rg "^  Width" | rg -o "[0-9]+$")
PX_SEARCH=$((WX + WW - 206)); PX_BELL=$((WX + WW - 174)); PY_BAR=$((WY + 17))
echo "window ($WX,$WY) ${WW}px — search=($PX_SEARCH,$PY_BAR) bell=($PX_BELL,$PY_BAR)"

echo "--- [01] entry baseline"
key Escape; sleep 2
cap "$OUT/d18-01-entry.png"; echo "B01 $(md5 "$OUT/d18-01-entry.png")"

echo "--- [02] title-bar crop (Search + Bell entries visible)"
capbar "$OUT/d18-02-titlebar-crop.png"; echo "B02 $(md5 "$OUT/d18-02-titlebar-crop.png")"

echo "--- [03] CLICK command-palette entry → palette OPENS"
click "$PX_SEARCH" "$PY_BAR"
sleep 3
cap "$OUT/d18-03-palette-open.png"; echo "B03 $(md5 "$OUT/d18-03-palette-open.png")"

echo "--- [04] Escape close"
key Escape; sleep 2
cap "$OUT/d18-04-palette-closed.png"; echo "B04 $(md5 "$OUT/d18-04-palette-closed.png")"

echo "--- [05] CLICK Activity bell → Activity view OPENS"
click "$PX_BELL" "$PY_BAR"
sleep 3
cap "$OUT/d18-05-activity-open.png"; echo "B05 $(md5 "$OUT/d18-05-activity-open.png")"

echo "--- [06] Escape close"
key Escape; sleep 2
cap "$OUT/d18-06-activity-closed.png"; echo "B06 $(md5 "$OUT/d18-06-activity-closed.png")"

echo "--- [07] chat A + mark unread (dot on row)"
ladder; sleep 0.5
type_text "alpha visible entry chat"
sleep 1
key ctrl+Return
sleep 12
key ctrl+shift+u
sleep 2.5
cap "$OUT/d18-07-unread-dot.png"; echo "B07 $(md5 "$OUT/d18-07-unread-dot.png")"

echo "--- [08] HOVER the sidebar dot → tooltip"
# sidebar chat rows sit under the Projects/Chats header; hover the first chat row's left edge
DISPLAY=$DISP xdotool mousemove $((WX + 150)) $((WY + 330))
sleep 2.5
cap "$OUT/d18-08-dot-tooltip.png"; echo "B08 $(md5 "$OUT/d18-08-dot-tooltip.png")"

echo "--- [09] archive the chat (Ctrl+Shift+A) so the contextual section exists"
key ctrl+shift+a
sleep 3
cap "$OUT/d18-09a-archived.png"; echo "B09a $(md5 "$OUT/d18-09a-archived.png")"

echo "--- [10] Settings → Archived chats (visible Delete label)"
key ctrl+comma
sleep 3
cap "$OUT/d18-10a-settings.png"; echo "B10a $(md5 "$OUT/d18-10a-settings.png")"
# navigate to Archived chats in the settings nav (left column) — calibrated
DISPLAY=$DISP xdotool mousemove 150 400 click 1
sleep 2.5
cap "$OUT/d18-10b-archived.png"; echo "B10b $(md5 "$OUT/d18-10b-archived.png")"

echo "--- [11] settle"
key Escape; sleep 1.5; key Escape; sleep 1.5
cap "$OUT/d18-11-settle.png"; echo "B11 $(md5 "$OUT/d18-11-settle.png")"

echo "--- teardown"
kill $APP_PID 2>/dev/null; pkill -P $APP_PID 2>/dev/null
sleep 1
pkill -x picom 2>/dev/null
kill $XPID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
echo "d18 done (artifacts in $OUT)"
