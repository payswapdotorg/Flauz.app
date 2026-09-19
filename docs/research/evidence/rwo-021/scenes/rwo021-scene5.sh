#!/bin/bash
# RWO-021 scene 5 — FINAL, window-offset-calibrated (window 1278x818 at screen +163,+91)
set -u
BIN="/home/z/flauz/target/release/codexrs"
OUT=/home/z/rwo021-evidence
DISP=":104"
PREFIX=/home/z/sysroot/prefix
export PATH="$PREFIX/usr/bin:$PATH"
export LD_LIBRARY_PATH="$PREFIX/usr/lib/x86_64-linux-gnu:$PREFIX/usr/lib/llvm-19/lib:${LD_LIBRARY_PATH:-}"
DATA=/tmp/rwo021-data
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1600x1000x24 -nolisten tcp -noreset >"$OUT/xvfb5.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom5.log" 2>&1 &
sleep 3
HOME_D=$(mktemp -d /tmp/rwo021-home5.XXXXXX)
mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="$PREFIX/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$DATA" \
  "$BIN" >"$OUT/app5.log" 2>&1 &
APP_PID=$!
cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1600x1000 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 55 "$1"; }
click() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }
rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { echo "app exited early"; break; }
  click 900 500 2>/dev/null || true
  cap "$OUT/_probe5.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe5.png" 2>/dev/null || echo 0)
  [ "${SZ:-0}" -gt 30000 ] && { echo "rendered after $((i*5))s"; rendered=1; break; }
done
WID=$(DISPLAY=$DISP xwininfo -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 2

echo "--- [e00] restored state"
cap "$OUT/e00-restored.png"
echo "--- [e01] Back to app (195,180)"
click 195 180; sleep 2.5
cap "$OUT/e01-main-surface.png"
echo "--- [e02/e03] composer /feedback (FBK-C5 slash leg)"
click 880 870; sleep 1
type_ "/feedback"; sleep 1.5
cap "$OUT/e02-composer-feedback-typed.png"
key Return; sleep 2.5
cap "$OUT/e03-feedback-dialog-slash.png"
key Escape; sleep 1.5
echo "--- [e04] sidebar Pull requests (300,222)"
click 300 222; sleep 3
cap "$OUT/e04-sidebar-pullrequests.png"
echo "--- [e05] sidebar Plugins (300,258)"
click 300 258; sleep 3
cap "$OUT/e05-sidebar-plugins.png"
echo "--- [e06] settings; [e07] search import (SET-C3)"
key ctrl+comma; sleep 3
cap "$OUT/e06-settings.png"
click 300 235; sleep 0.8
type_ "import"; sleep 1.8
cap "$OUT/e07-settings-search-import.png"
key ctrl+a; key Delete; sleep 0.8
echo "--- [e08] Keyboard shortcuts nav row (300,475); [e09] filter 'search' (KSR-C4)"
click 300 475; sleep 2.5
cap "$OUT/e08-settings-kbshortcuts.png"
click 700 240; sleep 0.8
type_ "search"; sleep 1.8
cap "$OUT/e09-kbshortcuts-filtered.png"

kill -0 $APP_PID 2>/dev/null && echo "APP ALIVE AT SCENE-5 END" || echo "APP EXITED"
kill $APP_PID 2>/dev/null; sleep 1; pkill -f "Xvfb $DISP" 2>/dev/null
for f in "$OUT"/e*.png; do echo "$(basename "$f"): $(md5 "$f")"; done > "$OUT/scene5-md5.txt"
cat "$OUT/scene5-md5.txt"
echo "SCENE5_COMPLETE"
