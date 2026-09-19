#!/bin/bash
# D11b — WO-P2-008 GUI verification, EMPTY-SURFACE scope (honest revision).
#
# The original d11-unread-attention.sh assumed composer chat creation works
# in the lab — it does NOT: the desktop app spawns the official codex CLI as
# its runtime (resolve_codex_binary -> PATH `codex` / CODEX_RS_CODEX_BIN),
# no codex CLI exists in this lab, and fabricating a mock runtime is
# forbidden (no-fabrication doctrine). Without a runtime the sidebar is the
# authenticated-less entry surface with zero threads, so dot-on-row and
# jump-between-chats are NOT exercisable here — they are covered by the
# codex-core state-machine tests (7) + codex-app binding/jump tests (6, CI).
#
# What this scene DOES verify (all four bindings resolve visibly on the real
# binary — the WO-P2-007 input-quality doctrine: never a silent no-op):
#   01 entry baseline
#   02 Ctrl+Shift+U  -> "Select a chat before marking it unread."
#   03 Ctrl+Alt+A    -> "No chats need attention."
#   04 Ctrl+Alt+U    -> "Activity view is not available yet. ..."
#   05 Shift+Escape  -> "No unread chats"
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d11b}"
DISP=":104"

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D11DATA=/tmp/d11b-data
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

HOME_D=$(mktemp -d /tmp/d11b-home.XXXXXX)
mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$D11DATA" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }

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
cap "$OUT/d11b-01-entry.png"; B1=$(md5 "$OUT/d11b-01-entry.png")

echo "--- [02] Ctrl+Shift+U (mark unread, no chat selected)"
sleep 1
key ctrl+shift+u
sleep 2.5
cap "$OUT/d11b-02-mark-unread.png"; B2=$(md5 "$OUT/d11b-02-mark-unread.png")

echo "--- [03] Ctrl+Alt+A (next unread chat, none exist)"
sleep 1
key ctrl+alt+a
sleep 2.5
cap "$OUT/d11b-03-next-unread.png"; B3=$(md5 "$OUT/d11b-03-next-unread.png")

echo "--- [04] Ctrl+Alt+U (activity view guidance)"
sleep 1
key ctrl+alt+u
sleep 2.5
cap "$OUT/d11b-04-activity-view.png"; B4=$(md5 "$OUT/d11b-04-activity-view.png")

echo "--- [05] Shift+Escape (clear all unread, none flagged)"
sleep 1
key shift+Escape
sleep 2.5
cap "$OUT/d11b-05-clear-all.png"; B5=$(md5 "$OUT/d11b-05-clear-all.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null

{
  echo "d11b-01-entry:        $B1"
  echo "d11b-02-mark-unread:  $B2"
  echo "d11b-03-next-unread:  $B3"
  echo "d11b-04-activity-view: $B4"
  echo "d11b-05-clear-all:    $B5"
} > "$OUT/d11b-md5.txt"
cat "$OUT/d11b-md5.txt"
echo "--- app.log tail ---"
tail -5 "$OUT/app.log"
echo "D11B SCENE COMPLETE — VLM-read frames 02-05 for the four honest statuses"
