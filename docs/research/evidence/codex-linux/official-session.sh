#!/usr/bin/env bash
# official-session.sh — WO-LAB-001: run the OFFICIAL ChatGPT/Codex Linux preview
# app (26.908.70816, selective userspace extract) in the LINUX_GUI_LAB, capture
# baseline surfaces for parity evidence. Unauthenticated by design (no
# credentials in lab); account-powered surfaces stay bound.
#
# Usage: official-session.sh <workspace-dir> <display> <scene-script>
set -u
WS=$1; DISP=$2; SCENE=$3
LABTOOLS=/home/z/parity-lab/tools
export LD_LIBRARY_PATH="$LABTOOLS/rootfs/usr/lib/x86_64-linux-gnu"
RT="$LABTOOLS/rootfs/usr/bin"
APPDIR=/home/z/parity-lab/official-app/usr/lib/chatgpt
export DISPLAY="$DISP"

mkdir -p "$WS/home" "$WS/data" "$WS/config" "$WS/cache" "$WS/runtime" "$WS/shots"
chmod 700 "$WS/runtime" 2>/dev/null || true

pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb "$DISP" -screen 0 1600x1000x24 -nolisten tcp -noreset >"$WS/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY="$DISP" LD_LIBRARY_PATH="$LD_LIBRARY_PATH" "$RT/picom" --backend xrender >"$WS/picom.log" 2>&1 &
PICOMPID=$!
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up (see $WS/picom.log)"

# Electron on a bare Xvfb: no sandbox helper binary shipped -> --no-sandbox;
# software GL via swiftshader; English UI; X11 explicitly (no Wayland attempt).
env -u WAYLAND_DISPLAY \
  HOME="$WS/home" XDG_DATA_HOME="$WS/data" XDG_CONFIG_HOME="$WS/config" \
  XDG_CACHE_HOME="$WS/cache" XDG_RUNTIME_DIR="$WS/runtime" \
  LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 \
  VK_ICD_FILENAMES="/home/z/parity-lab/tools/rootfs/usr/share/vulkan/icd.d/lvp_icd.json" \
  DISPLAY="$DISP" \
  "$APPDIR/ChatGPT" --no-sandbox --disable-gpu --disable-dev-shm-usage \
    --ozone-platform=x11 --lang=en-US >"$WS/app.log" 2>&1 &
APID=$!
echo "official-session: app pid $APID on $DISP (workspace $WS)"

source "$LABTOOLS/scene_helpers.sh"
rendered=0
for i in $(seq 1 36); do
  sleep 5
  kill -0 "$APID" 2>/dev/null || { echo "official-session: app exited early (see $WS/app.log)"; break; }
  XDO mousemove 800 500 click 1 2>/dev/null || true
  CAPTURE _render_probe >/dev/null 2>&1
  SZ=$(stat -c%s "$WS/shots/_render_probe.png" 2>/dev/null || echo 0)
  if [ "${SZ:-0}" -gt 30000 ]; then
    echo "official-session: UI rendered after $((i*5))s (frame $SZ bytes)"
    rendered=1
    break
  fi
done
[ "$rendered" = "1" ] || echo "official-session: WARN — no >30KB frame in 180s; running scene anyway"

WS="$WS" APID="$APID" bash "$SCENE"

echo "official-session: tearing down"
kill "$APID" 2>/dev/null
pkill -P "$APID" 2>/dev/null
pkill -f "chatgpt/codex-launcher" 2>/dev/null
pkill -f "$APPDIR/ChatGPT" 2>/dev/null
kill "$PICOMPID" 2>/dev/null; pkill -x picom 2>/dev/null
kill "$XPID" 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
echo "official-session: done (artifacts in $WS/shots, logs: app.log xvfb.log picom.log)"
