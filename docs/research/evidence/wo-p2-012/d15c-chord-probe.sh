#!/bin/bash
# D15c — discriminating probe: known-good chord vs r3 guard chords.
# TEST 1: ctrl+shift+u (toggleThreadUnread, D11b-proven on the 008 build)
#         -> if its status renders on the r3 binary, delivery+render are
#            fine and the archive/pin/rename silence is path-specific.
#         -> if it does NOT render, the entry-surface status rendering
#            regressed somewhere between 00a3392 and r3 (real bug).
# TEST 2: ctrl+shift+a / ctrl+alt+p / ctrl+alt+r (r3 guards) after it.
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/wo-p2-012/d15c}"
DISP=":111"
RUNTIME=/home/z/parity-lab/runtime/codex
[ -x "$RUNTIME" ] || { echo "FATAL: no runtime"; exit 1; }
export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/parity-lab/lib:${LD_LIBRARY_PATH:-}"

D=/tmp/d15c-data; rm -rf "$D"; mkdir -p "$D/codex-home"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
[ -z "$DONOR" ] && { echo "FATAL: no donor"; exit 1; }
cp "$DONOR" "$D/state.sqlite3"
mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
HOME_H=$(mktemp -d /tmp/d15c-home.XXXXXX); mkdir -p "$HOME_H/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_H" XDG_RUNTIME_DIR="$HOME_H/xdg" \
  CODEX_RS_DATA_DIR="$D" CODEX_HOME="$D/codex-home" CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!
cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }

rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || break
  cap "$OUT/_probe.png" 2>/dev/null
  [ "$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)" -gt 30000 ] && { rendered=1; echo "rendered @ $((i*5))s"; break; }
done
WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 2

echo "--- [01] entry"
sleep 2; cap "$OUT/c01-entry.png"; B1=$(md5 "$OUT/c01-entry.png")

echo "--- [02] ctrl+shift+u (D11b-proven: 'Select a chat before marking it unread.')"
key ctrl+shift+u; sleep 2.5
cap "$OUT/c02-unread.png"; B2=$(md5 "$OUT/c02-unread.png")

echo "--- [03] settle back (wait for status fade)"
sleep 8; cap "$OUT/c03-settle.png"; B3=$(md5 "$OUT/c03-settle.png")

echo "--- [04] ctrl+shift+a (F-A1)"
key ctrl+shift+a; sleep 2.5
cap "$OUT/c04-archive.png"; B4=$(md5 "$OUT/c04-archive.png")

echo "--- [05] ctrl+alt+p (F-A2)"
sleep 3; key ctrl+alt+p; sleep 2.5
cap "$OUT/c05-pin.png"; B5=$(md5 "$OUT/c05-pin.png")

echo "--- [06] ctrl+alt+r (F-A3)"
sleep 3; key ctrl+alt+r; sleep 2.5
cap "$OUT/c06-rename.png"; B6=$(md5 "$OUT/c06-rename.png")

echo "--- [07] shift+escape (clearAllUnread, D11b-proven: 'No unread chats')"
sleep 3; key shift+Escape; sleep 2.5
cap "$OUT/c07-clearall.png"; B7=$(md5 "$OUT/c07-clearall.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
{
  echo "c01-entry:     $B1"
  echo "c02-unread:    $B2   (KNOWN-GOOD: expect delta vs c01 + status text)"
  echo "c03-settle:    $B3   (status faded?)"
  echo "c04-archive:   $B4   (F-A1)"
  echo "c05-pin:       $B5   (F-A2)"
  echo "c06-rename:    $B6   (F-A3)"
  echo "c07-clearall:  $B7   (KNOWN-GOOD #2: 'No unread chats')"
} | tee "$OUT/d15c-md5.txt"
echo "D15C COMPLETE"
