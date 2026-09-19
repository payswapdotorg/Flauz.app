#!/bin/bash
# D15d — WO-P2-012 chord probes, modal-cleared + sidebar-defocused.
#
# D15-series forensics so far:
#   - d15c c01-c06: the "Introducing GPT-5.6-Sol" NUX MODAL was up the whole
#     time (c07's shift+Escape dismissed it — that frame delta was the modal
#     clearing, not a status). All chord probes were modal-absorbed.
#   - d15b [01b]: the "neutral" click at (720,300) landed ON the modal.
#   - d15 [02-04]: NUX modal dismissed by an early Escape; card-only state;
#     chords still no-op — but no defocus was attempted (the card/buttons may
#     hold focus; GPUI focused-element keydown may consume multi-modifier
#     chords before the workspace interceptor).
#
# D15d: (1) dismiss the NUX modal (Escape x2, verify by capture), (2) defocus
# via a sidebar empty-area click (x200,y600 — outside card/modal/composer),
# (3) fire the four guarded chords with dual-timing captures, including the
# D11b-proven ctrl+shift+u as the known-good cross-check.
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/wo-p2-012/d15d}"
DISP=":112"
RUNTIME=/home/z/parity-lab/runtime/codex
[ -x "$RUNTIME" ] || { echo "FATAL: no runtime"; exit 1; }
export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/parity-lab/lib:${LD_LIBRARY_PATH:-}"

D=/tmp/d15d-data; rm -rf "$D"; mkdir -p "$D/codex-home"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
[ -z "$DONOR" ] && { echo "FATAL: no donor"; exit 1; }
cp "$DONOR" "$D/state.sqlite3"
mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
HOME_H=$(mktemp -d /tmp/d15d-home.XXXXXX); mkdir -p "$HOME_H/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_H" XDG_RUNTIME_DIR="$HOME_H/xdg" \
  CODEX_RS_DATA_DIR="$D" CODEX_HOME="$D/codex-home" CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!
cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
click() { DISPLAY=$DISP xdotool mousemove $1 $2 click 1; sleep 0.5; }

for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || break
  cap "$OUT/_probe.png" 2>/dev/null
  [ "$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)" -gt 30000 ] && { echo "rendered @ $((i*5))s"; break; }
done
WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 2

echo "--- [01] entry (NUX modal may be up)"
cap "$OUT/d01-entry.png"; B1=$(md5 "$OUT/d01-entry.png")

echo "--- [02] Escape x2 (dismiss NUX modal), settle"
key Escape; sleep 1.5; key Escape; sleep 2
cap "$OUT/d02-modal-cleared.png"; B2=$(md5 "$OUT/d02-modal-cleared.png")

echo "--- [03] sidebar defocus click (x200 y600)"
click 200 600
cap "$OUT/d03-defocused.png"; B3=$(md5 "$OUT/d03-defocused.png")

echo "--- [04] F-A1 ctrl+shift+a (expect 'Select a chat before archiving it.')"
key ctrl+shift+a; sleep 1.2
cap "$OUT/d04a-early.png"; B4e=$(md5 "$OUT/d04a-early.png")
sleep 1.5
cap "$OUT/d04b-late.png"; B4l=$(md5 "$OUT/d04b-late.png")

echo "--- [05] F-A2 ctrl+alt+p (expect 'Select a chat before pinning or unpinning it.')"
sleep 3; click 200 600
key ctrl+alt+p; sleep 1.2
cap "$OUT/d05a-early.png"; B5e=$(md5 "$OUT/d05a-early.png")
sleep 1.5
cap "$OUT/d05b-late.png"; B5l=$(md5 "$OUT/d05b-late.png")

echo "--- [06] F-A3 ctrl+alt+r (expect 'Select a chat before renaming it.')"
sleep 3; click 200 600
key ctrl+alt+r; sleep 1.2
cap "$OUT/d06a-early.png"; B6e=$(md5 "$OUT/d06a-early.png")
sleep 1.5
cap "$OUT/d06b-late.png"; B6l=$(md5 "$OUT/d06b-late.png")

echo "--- [07] cross-check ctrl+shift+u (D11b-proven guard family)"
sleep 3; click 200 600
key ctrl+shift+u; sleep 1.2
cap "$OUT/d07a-early.png"; B7e=$(md5 "$OUT/d07a-early.png")
sleep 1.5
cap "$OUT/d07b-late.png"; B7l=$(md5 "$OUT/d07b-late.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
{
  echo "d01-entry:       $B1   (NUX modal state)"
  echo "d02-modalclear:  $B2   (expect != d01)"
  echo "d03-defocused:   $B3"
  echo "d04-early:       $B4e   (F-A1 @1.2s)"
  echo "d04-late:        $B4l   (F-A1 @2.7s)"
  echo "d05-early:       $B5e   (F-A2 @1.2s)"
  echo "d05-late:        $B5l   (F-A2 @2.7s)"
  echo "d06-early:       $B6e   (F-A3 @1.2s)"
  echo "d06-late:        $B6l   (F-A3 @2.7s)"
  echo "d07-early:       $B7e   (cross-check @1.2s)"
  echo "d07-late:        $B7l   (cross-check @2.7s)"
} | tee "$OUT/d15d-md5.txt"
echo "D15D COMPLETE"
