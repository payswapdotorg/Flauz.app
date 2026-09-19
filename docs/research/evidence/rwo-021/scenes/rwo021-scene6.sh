#!/bin/bash
# RWO-021 scene 6 — SSIM-guided: ladder to leave Settings, then all remaining legs.
set -u
BIN="/home/z/flauz/target/release/codexrs"
OUT=/home/z/rwo021-evidence
DISP=":104"
PREFIX=/home/z/sysroot/prefix
export PATH="$PREFIX/usr/bin:$PATH"
export LD_LIBRARY_PATH="$PREFIX/usr/lib/x86_64-linux-gnu:$PREFIX/usr/lib/llvm-19/lib:${LD_LIBRARY_PATH:-}"
DATA=/tmp/rwo021-data
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1600x1000x24 -nolisten tcp -noreset >"$OUT/xvfb6.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom6.log" 2>&1 &
sleep 3
HOME_D=$(mktemp -d /tmp/rwo021-home6.XXXXXX)
mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="$PREFIX/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$DATA" \
  "$BIN" >"$OUT/app6.log" 2>&1 &
APP_PID=$!
cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1600x1000 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 55 "$1"; }
click() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }
# ssim of file vs reference -> prints number
ssim_of() { ffmpeg -i "$1" -i "$2" -filter_complex "[0][1]ssim" -f null - 2>&1 | grep -oE "All:[0-9.]+([eE]-?[0-9]+)?" | cut -d: -f2; }

rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { echo "app exited early"; break; }
  click 900 500 2>/dev/null || true
  cap "$OUT/_probe6.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe6.png" 2>/dev/null || echo 0)
  [ "${SZ:-0}" -gt 30000 ] && { echo "rendered after $((i*5))s"; rendered=1; break; }
done
WID=$(DISPLAY=$DISP xwininfo -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 2

cap "$OUT/f00-restored.png"
echo "f00: $(md5 "$OUT/f00-restored.png")"

# --- Ladder: click Back-to-app candidates until we are on the main surface ---
MAIN_REF="$OUT/a01-ws-baseline.png"      # main surface (composer + fixture project)
SET_REF="$OUT/e01-main-surface.png"      # settings General page
on_main=0
ladder_y="205 185 165 150 135 120 105"
for y in $ladder_y; do
  click 300 $y; sleep 2
  cap "$OUT/_ladder.png"
  S_MAIN=$(ssim_of "$OUT/_ladder.png" "$MAIN_REF")
  S_SET=$(ssim_of "$OUT/_ladder.png" "$SET_REF")
  echo "ladder y=$y  main_ssim=$S_MAIN  settings_ssim=$S_SET"
  if awk -v m="$S_MAIN" -v s="$S_SET" 'BEGIN{exit !(m > s + 0.05)}'; then
    echo "MAIN SURFACE reached at ladder y=$y"
    cp "$OUT/_ladder.png" "$OUT/f01-main-surface.png"
    on_main=1
    break
  fi
done
[ "$on_main" = "1" ] || { echo "FATAL: never reached main surface"; kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP"; exit 1; }

echo "--- [f02/f03] composer /feedback (FBK-C5 slash leg)"
click 880 870; sleep 1
type_ "/feedback"; sleep 1.5
cap "$OUT/f02-composer-feedback-typed.png"
key Return; sleep 2.5
cap "$OUT/f03-feedback-dialog-slash.png"
key Escape; sleep 1.5
echo "--- [f04] sidebar Pull requests (300,222)"
click 300 222; sleep 3
cap "$OUT/f04-sidebar-pullrequests.png"
echo "--- [f05] sidebar Plugins (300,258)"
click 300 258; sleep 3
cap "$OUT/f05-sidebar-plugins.png"
echo "--- [f06] settings; [f07] search 'import' (SET-C3)"
key ctrl+comma; sleep 3
cap "$OUT/f06-settings.png"
click 300 203; sleep 0.8
type_ "import"; sleep 1.8
cap "$OUT/f07-settings-search-import.png"
key ctrl+a; key Delete; sleep 0.8
echo "--- [f08] Keyboard shortcuts row (300,382); [f09] filter 'search' (KSR-C4)"
click 300 382; sleep 2.5
cap "$OUT/f08-settings-kbshortcuts.png"
click 700 240; sleep 0.8
type_ "search"; sleep 1.8
cap "$OUT/f09-kbshortcuts-filtered.png"

kill -0 $APP_PID 2>/dev/null && echo "APP ALIVE AT SCENE-6 END" || echo "APP EXITED"
kill $APP_PID 2>/dev/null; sleep 1; pkill -f "Xvfb $DISP" 2>/dev/null
for f in "$OUT"/f*.png; do echo "$(basename "$f"): $(md5 "$f")"; done > "$OUT/scene6-md5.txt"
cat "$OUT/scene6-md5.txt"
echo "SCENE6_COMPLETE"
