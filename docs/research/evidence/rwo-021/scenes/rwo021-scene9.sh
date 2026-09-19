#!/bin/bash
set -u
BIN="/home/z/flauz/target/release/codexrs"
OUT=/home/z/rwo021-evidence
DISP=":104"
PREFIX=/home/z/sysroot/prefix
export PATH="$PREFIX/usr/bin:$PATH"
export LD_LIBRARY_PATH="$PREFIX/usr/lib/x86_64-linux-gnu:$PREFIX/usr/lib/llvm-19/lib:${LD_LIBRARY_PATH:-}"
DATA=/tmp/rwo021-data
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1600x1000x24 -nolisten tcp -noreset >"$OUT/xvfb9.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom9.log" 2>&1 &
sleep 3
HOME_D=$(mktemp -d /tmp/rwo021-home9.XXXXXX)
mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="$PREFIX/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$DATA" \
  "$BIN" >"$OUT/app9.log" 2>&1 &
APP_PID=$!
cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1600x1000 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 55 "$1"; }
click() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }
ssim_of() { ffmpeg -i "$1" -i "$2" -filter_complex "[0][1]ssim" -f null - 2>&1 | grep -oE "All:[0-9.]+([eE]-?[0-9]+)?" | cut -d: -f2; }
rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || break
  click 900 500 2>/dev/null || true
  cap "$OUT/_probe9.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe9.png" 2>/dev/null || echo 0)
  [ "${SZ:-0}" -gt 30000 ] && { echo "rendered after $((i*5))s"; rendered=1; break; }
done
WID=$(DISPLAY=$DISP xwininfo -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 2
MAIN_REF="$OUT/a01-ws-baseline.png"; SET_REF="$OUT/e01-main-surface.png"
click 300 165; sleep 2.5
cap "$OUT/_i00.png"
S_MAIN=$(ssim_of "$OUT/_i00.png" "$MAIN_REF"); S_SET=$(ssim_of "$OUT/_i00.png" "$SET_REF")
echo "post-ladder: main=$S_MAIN set=$S_SET"
awk -v m="$S_MAIN" -v s="$S_SET" 'BEGIN{exit !(m > s + 0.05)}' || { echo "FATAL not on main"; kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP"; exit 1; }

echo "--- [i01/i02] composer /feedback + alt+Return"
click 748 841; sleep 1
type_ "/feedback"; sleep 1.2
cap "$OUT/i01-composer-feedback-typed.png"
key alt+Return; sleep 2.5
cap "$OUT/i02-feedback-dialog-alt-enter.png"
key Escape; sleep 1.5
echo "--- [i03] overlay filtered query (matcher GUI evidence)"
key ctrl+slash; sleep 2.5
type_ "search"; sleep 1.8
cap "$OUT/i03-overlay-filtered-search.png"
key Escape; sleep 1
kill -0 $APP_PID 2>/dev/null && echo "APP ALIVE AT SCENE-9 END" || echo "APP EXITED"
kill $APP_PID 2>/dev/null; sleep 1; pkill -f "Xvfb $DISP" 2>/dev/null
for f in "$OUT"/i*.png; do echo "$(basename "$f"): $(md5 "$f")"; done
echo "SCENE9_COMPLETE"
