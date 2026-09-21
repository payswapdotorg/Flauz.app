#!/bin/bash
# D19 phase C — the trap-probe leg (Lead-driven, semi-interactive).
# Boots the app on the EXISTING /tmp/d19-data (5 seeded browsing_history rows),
# dismisses the fresh-profile what's-new promo, navigates to Settings Browser,
# and parks at the settled frame. The Lead then VLM-verifies the Clear button
# position and drives the probes via d19-probes.sh.
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d19}"
DISP=":110"
RUNTIME=/home/z/parity-lab/runtime/codex

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D19DATA=/tmp/d19-data
mkdir -p "$OUT"

pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!; echo "$XPID" > "$OUT/xpid"
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3

HOME_D=$(mktemp -d /tmp/d19c-home.XXXXXX)
mkdir -p "$HOME_D/xdg"
echo "$HOME_D" > "$OUT/home_d"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$D19DATA" \
  CODEX_HOME="$D19DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!
echo "$APP_PID" > "$OUT/app_pid"
echo "$DISP" > "$OUT/disp"

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 60 "$1"; }
clickat() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }

rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { echo "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 1200 100 click 1 2>/dev/null || true
  cap "$OUT/_probe.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
  if [ "${SZ:-0}" -gt 30000 ]; then rendered=1; break; fi
done
[ "$rendered" = "1" ] || echo "WARN — no >30KB frame in 150s"
sleep 4

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

echo "--- [C0] boot state (promo check)"
cap "$OUT/d19-c0-boot.png"; echo "C0 $(md5 "$OUT/d19-c0-boot.png")"

echo "--- dismiss promo if present (click the primary button area)"
clickat 580 619; sleep 2
cap "$OUT/d19-c1-after-promo.png"; echo "C1 $(md5 "$OUT/d19-c1-after-promo.png")"

echo "--- [C2] palette -> browser settings"
key ctrl+k; sleep 2
type_ "browser settings"; sleep 2
key Return; sleep 2.5
cap "$OUT/d19-c2-settings-browser.png"; echo "C2 $(md5 "$OUT/d19-c2-settings-browser.png")"
sleep 1.5
cap "$OUT/d19-c3-settings-settled.png"; echo "C3 $(md5 "$OUT/d19-c3-settings-settled.png")"
echo "PHASE C PARKED — app running (APP_PID=$APP_PID). Lead: VLM d19-c3 for Clear coords, then d19-probes.sh"
