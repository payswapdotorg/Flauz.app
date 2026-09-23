#!/bin/bash
# D25-PROBE — discriminate: does the recovery chord (Ctrl+Alt+Shift+R)
# fire, with the anchor task's retry loop given time to exhaust?
# Baseline → chord → captures at +1.5s and +4s.
set -u
BIN="${1:?binary}"
OUT="${2:-/home/z/parity-lab/evidence/d25-probe}"
DISP=":127"
RUNTIME=/home/z/parity-lab/runtime/codex
export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D=/tmp/d25p-data
rm -rf "$D"; mkdir -p "$D/codex-home"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
cp "$DONOR" "$D/state.sqlite3"
mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
H=$(mktemp -d /tmp/d25p-home.XXXXXX); mkdir -p "$H/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$H" XDG_RUNTIME_DIR="$H/xdg" \
  CODEX_RS_DATA_DIR="$D" CODEX_HOME="$D/codex-home" CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
refocus() { W=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+"); [ -n "$W" ] && DISPLAY=$DISP xdotool windowfocus --sync "$W" 2>/dev/null || true; }
key() { refocus; DISPLAY=$DISP xdotool key "$1"; }
type_text() { refocus; DISPLAY=$DISP xdotool type --delay 60 "$1"; }

for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP 2>/dev/null || break
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
  cap "$OUT/_probe.png" 2>/dev/null
  [ "$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)" -gt 30000 ] && { echo "rendered ($((i*5))s)"; break; }
done
sleep 3
DISPLAY=$DISP xdotool mousemove 580 619 click 1; sleep 2
DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true; sleep 1

key Escape; sleep 1
key ctrl+n; sleep 2.5
type_text "Recovery chord probe"; sleep 1
key ctrl+Return; sleep 3
echo "--- anchor done; settling 30s for the retry loop to space out"
sleep 30
cap "$OUT/p01-baseline.png"; echo "P01 $(md5 "$OUT/p01-baseline.png")"
echo "--- chord ctrl+alt+shift+r"
key ctrl+alt+shift+r; sleep 1.5
cap "$OUT/p02-chord-15.png"; echo "P02 $(md5 "$OUT/p02-chord-15.png")"
sleep 2.5
cap "$OUT/p03-chord-40.png"; echo "P03 $(md5 "$OUT/p03-chord-40.png")"
echo "--- chord again (repeat-fire check)"
key ctrl+alt+shift+r; sleep 1.5
cap "$OUT/p04-chord2.png"; echo "P04 $(md5 "$OUT/p04-chord2.png")"
kill $APP 2>/dev/null; kill $XPID 2>/dev/null; pkill -f "picom" 2>/dev/null
echo "PROBE COMPLETE"
