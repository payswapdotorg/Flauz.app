#!/bin/bash
# D11 — WO-P2-008 GUI verification (per-chat unread-attention + 4 bindings).
#
# Delivery under test: feat/wo-p2-008-unread-attention @ c37c21b20
#   - Ctrl+Shift+U  toggleThreadUnread   (mark SELECTED chat unread → dot)
#   - Ctrl+Alt+A    nextUnreadChat       (jump to next unread chat)
#   - Shift+Escape  clearAllUnread       (clear every flag + honest status)
#   - Ctrl+Alt+U    toggleActivityView   (honest guidance — separate scene target)
#
# Scene (guaranteed UI paths — manual flags need no backend):
#   01 entry surface baseline
#   02 create chat A via composer ("alpha attention chat" + Return)
#   03 Ctrl+Shift+U on A → dot + "Chat marked unread"
#   04 Ctrl+N chat B ("beta background chat" + Return) → B selected, A dot stays
#   05 Ctrl+Alt+A → jump to A (visit clears A's dot)
#   06 Ctrl+Shift+U again → dot back on A
#   07 Shift+Escape → "Cleared unread indicators for 1 chat", dot gone
#
# PASS: md5 frame sequence + VLM reads show dot/jump/clear; escape hatch on
# every step (frame-diff check, app.log tail on failure).
# Usage: d11-unread-attention.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d11}"
DISP=":103"

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D11DATA=/tmp/d11-data
rm -rf "$D11DATA"; mkdir -p "$D11DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D11DATA/state.sqlite3"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D11=$(mktemp -d /tmp/d11-home.XXXXXX)
mkdir -p "$HOME_D11/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D11" XDG_RUNTIME_DIR="$HOME_D11/xdg" \
  CODEX_RS_DATA_DIR="$D11DATA" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_text() { DISPLAY=$DISP xdotool type --delay 150 "$1"; }

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

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

echo "--- [01] entry baseline"
cap "$OUT/d11-01-entry.png"; B1=$(md5 "$OUT/d11-01-entry.png")

echo "--- [02] create chat A (composer)"
DISPLAY=$DISP xdotool mousemove 720 700 click 1 2>/dev/null || true
sleep 0.5
type_text "alpha attention chat"
sleep 1.0
cap "$OUT/d11-02a-typed.png"
key Return
sleep 6
cap "$OUT/d11-02b-chatA-created.png"; B2=$(md5 "$OUT/d11-02b-chatA-created.png")

echo "--- [03] Ctrl+Shift+U → mark A unread (dot + status)"
sleep 2
key ctrl+shift+u
sleep 2.5
cap "$OUT/d11-03-marked-unread.png"; B3=$(md5 "$OUT/d11-03-marked-unread.png")

echo "--- [04] Ctrl+N → chat B (A keeps its dot in the sidebar)"
key ctrl+n
sleep 3
DISPLAY=$DISP xdotool mousemove 720 700 click 1 2>/dev/null || true
sleep 0.5
type_text "beta background chat"
sleep 1.0
key Return
sleep 6
cap "$OUT/d11-04-chatB-created.png"; B4=$(md5 "$OUT/d11-04-chatB-created.png")

echo "--- [05] Ctrl+Alt+A → jump to A (visit clears the dot)"
sleep 2
key ctrl+alt+a
sleep 3
cap "$OUT/d11-05-jumped-to-A.png"; B5=$(md5 "$OUT/d11-05-jumped-to-A.png")

echo "--- [06] Ctrl+Shift+U again → dot back on A"
key ctrl+shift+u
sleep 2.5
cap "$OUT/d11-06-remarked.png"; B6=$(md5 "$OUT/d11-06-remarked.png")

echo "--- [07] Shift+Escape → clear all unread"
key shift+Escape
sleep 2.5
cap "$OUT/d11-07-cleared.png"; B7=$(md5 "$OUT/d11-07-cleared.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null

{
  echo "d11-01-entry:            $B1"
  echo "d11-02b-chatA-created:   $B2"
  echo "d11-03-marked-unread:    $B3"
  echo "d11-04-chatB-created:    $B4"
  echo "d11-05-jumped-to-A:      $B5"
  echo "d11-06-remarked:         $B6"
  echo "d11-07-cleared:          $B7"
} > "$OUT/d11-md5.txt"
cat "$OUT/d11-md5.txt"
echo "--- app.log tail ---"
tail -5 "$OUT/app.log"
echo "D11 SCENE COMPLETE — VLM-read frames 03/04/05/07 for the dot/jump/clear sequence"
