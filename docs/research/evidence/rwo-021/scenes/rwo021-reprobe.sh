#!/bin/bash
# RWO-021 re-probe — targeted legs that missed focus in the main scene:
#   FBK-C5 slash leg (composer /feedback), SET-C3 (settings search "import"),
#   KSR-C4 (shortcuts page filter), NTR-C5 (sidebar PR/Plugins pages)
set -u
BIN="/home/z/flauz/target/release/codexrs"
OUT=/home/z/rwo021-evidence
DISP=":104"
PREFIX=/home/z/sysroot/prefix
export PATH="$PREFIX/usr/bin:$PATH"
export LD_LIBRARY_PATH="$PREFIX/usr/lib/x86_64-linux-gnu:$PREFIX/usr/lib/llvm-19/lib:${LD_LIBRARY_PATH:-}"

DATA=/tmp/rwo021-data   # seeded earlier (recent_workspaces -> /tmp/rwo021-fixture)
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1600x1000x24 -nolisten tcp -noreset >"$OUT/xvfb2.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom2.log" 2>&1 &
sleep 3

HOME_D=$(mktemp -d /tmp/rwo021-home2.XXXXXX)
mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="$PREFIX/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$DATA" \
  "$BIN" >"$OUT/app2.log" 2>&1 &
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
  click 800 500 2>/dev/null || true
  cap "$OUT/_probe2.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe2.png" 2>/dev/null || echo 0)
  [ "${SZ:-0}" -gt 30000 ] && { echo "UI rendered after $((i*5))s"; rendered=1; break; }
done
[ "$rendered" = "1" ] || echo "WARN — render probe never crossed 30KB"

WID=$(DISPLAY=$DISP xwininfo -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 2

echo "--- [b01] composer focus + typed /feedback (FBK-C5 slash leg)"
click 700 770; sleep 1
type_ "/feedback"; sleep 1.5
cap "$OUT/b01-composer-feedback-typed.png"
key Return; sleep 2.5
cap "$OUT/b02-feedback-dialog-slash.png"
key Escape; sleep 1.5

echo "--- [b03] settings + search import (SET-C3)"
key ctrl+comma; sleep 3
cap "$OUT/b03-settings.png"
click 420 370; sleep 0.8
type_ "import"; sleep 1.8
cap "$OUT/b04-settings-search-import.png"
key ctrl+a; key Delete; sleep 0.8

echo "--- [b05] shortcuts page + filter (KSR-C4)"
key ctrl+k; sleep 1.5; type_ "keyboard"; sleep 1.2; key Return; sleep 2.5
cap "$OUT/b05-settings-kbshortcuts.png"
click 800 450; sleep 0.8
type_ "search"; sleep 1.8
cap "$OUT/b06-kbshortcuts-filtered.png"

echo "--- [b07] back to app + sidebar PR page (NTR-C5)"
click 420 260; sleep 2.5
cap "$OUT/b07-back-to-app.png"
click 300 222; sleep 3
cap "$OUT/b08-sidebar-pullrequests.png"
echo "--- [b09] sidebar Plugins page (NTR-C5)"
click 300 258; sleep 3
cap "$OUT/b09-sidebar-plugins.png"

kill -0 $APP_PID 2>/dev/null && echo "APP ALIVE AT RE-PROBE END" || echo "APP EXITED"
kill $APP_PID 2>/dev/null; sleep 1; pkill -f "Xvfb $DISP" 2>/dev/null

for f in "$OUT"/b*.png; do echo "$(basename "$f"): $(md5 "$f")"; done > "$OUT/reprobe-md5.txt"
cat "$OUT/reprobe-md5.txt"
echo "REPROBE_COMPLETE"
