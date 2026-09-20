#!/bin/bash
# D16 — WO-P2-013 Activity view scene (J-17 GUI evidence, Lead-owned).
#
# Binary under test: the WO-P2-013 delivery (feat/wo-p2-013-activity-view
# @ 6d08bf3 = worker 20a6018 + Lead fmt pass). D11r runtime pattern: the
# pinned Codex CLI runtime (npm @openai/codex@0.146.0-alpha.3.1-linux-x64,
# sha256 ae77c5e7...) injected via CODEX_RS_CODEX_BIN; CODEX_HOME pre-created;
# composer submits are CTRL+RETURN; NUX modal dismissed with Escape; composer
# focus via the D9 click ladder (x802, y849→789).
#
# Scene (unauthenticated D9 slice: thread creation OK, turn execution fails
# with a retry toast — non-blocking for the sidebar):
#   01 entry baseline (footer MUST read "App-server online")
#   02 create chat A via composer (ctrl+Return) → sidebar row + toast settle
#   03 Ctrl+Shift+U on A → dot on A + "Chat marked unread"
#   04 Ctrl+N → create chat B → B selected, A dot stays
#   05 Ctrl+Shift+U on B → dot on B (TWO chats flagged)
#   06 Ctrl+Alt+U → ACTIVITY VIEW OPENS: header "Activity", "2 chats need
#      attention", rows for both chats (first row selected)
#   07 Down → selection moves to the second row (keyboard nav)
#   08 Enter → JUMP: panel dismissed, selected chat open, ITS dot cleared,
#      the other chat's dot stays
#   09 Ctrl+Alt+U → reopens with "1 chat needs attention" (visit-clear composed)
#   10 Ctrl+Alt+U → TOGGLE-CLOSES (panel gone)
#   11 Shift+Escape → clear all unread flags
#   12 Ctrl+Alt+U → EMPTY STATE: "No chats need attention" + mark-unread guidance
#   13 Escape → closes (bare-Escape path)
#
# PASS: md5 frame sequence + VLM reads on 03/05/06/07/08/09/12.
# Usage: d16-activity-view.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d16}"
DISP=":104"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D16DATA=/tmp/d16-data
rm -rf "$D16DATA"; mkdir -p "$D16DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D16DATA/state.sqlite3"
mkdir -p "$D16DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D16=$(mktemp -d /tmp/d16-home.XXXXXX)
mkdir -p "$HOME_D16/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D16" XDG_RUNTIME_DIR="$HOME_D16/xdg" \
  CODEX_RS_DATA_DIR="$D16DATA" \
  CODEX_HOME="$D16DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_text() { DISPLAY=$DISP xdotool type --delay 60 "$1"; }
ladder() { for Y in 849 829 809 789; do DISPLAY=$DISP xdotool mousemove 802 $Y click 1; sleep 0.3; done; }

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

sleep 5
grep -iE 'app.?server|connect' "$OUT/app.log" | head -5 || echo "(no connection lines in app.log yet)"

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

echo "--- [01] entry baseline (expect footer 'App-server online')"
sleep 3
cap "$OUT/d16-01-entry.png"; B1=$(md5 "$OUT/d16-01-entry.png")

echo "--- [02] create chat A (composer, ctrl+Return submit)"
key Escape; sleep 2
cap "$OUT/d16-01b-nux-cleared.png"; BN=$(md5 "$OUT/d16-01b-nux-cleared.png")
ladder
sleep 0.5
type_text "alpha activity chat"
sleep 1.0
cap "$OUT/d16-02a-typed.png"; BA=$(md5 "$OUT/d16-02a-typed.png")
key ctrl+Return
sleep 12
cap "$OUT/d16-02b-chatA-created.png"; B2=$(md5 "$OUT/d16-02b-chatA-created.png")
key Escape
sleep 2

echo "--- [03] Ctrl+Shift+U on A → dot + status"
sleep 1
key ctrl+shift+u
sleep 2.5
cap "$OUT/d16-03-marked-unread.png"; B3=$(md5 "$OUT/d16-03-marked-unread.png")

echo "--- [04] Ctrl+N → chat B (A keeps its dot)"
key ctrl+n
sleep 3
ladder
sleep 0.5
type_text "beta activity chat"
sleep 1.0
key ctrl+Return
sleep 12
cap "$OUT/d16-04-chatB-created.png"; B4=$(md5 "$OUT/d16-04-chatB-created.png")
key Escape
sleep 2

echo "--- [05] Ctrl+Shift+U on B → dot on B (TWO flagged)"
key ctrl+shift+u
sleep 2.5
cap "$OUT/d16-05-both-flagged.png"; B5=$(md5 "$OUT/d16-05-both-flagged.png")

echo "--- [06] Ctrl+Alt+U → ACTIVITY VIEW OPENS (2 rows, first selected)"
key ctrl+alt+u
sleep 3
cap "$OUT/d16-06-activity-open.png"; B6=$(md5 "$OUT/d16-06-activity-open.png")

echo "--- [07] Down → selection to the second row (keyboard nav)"
key Down
sleep 2
cap "$OUT/d16-07-down-second-row.png"; B7=$(md5 "$OUT/d16-07-down-second-row.png")

echo "--- [08] Enter → JUMP (panel dismissed, selected chat's dot cleared)"
key Return
sleep 3.5
cap "$OUT/d16-08-enter-jumped.png"; B8=$(md5 "$OUT/d16-08-enter-jumped.png")

echo "--- [09] Ctrl+Alt+U → reopens with '1 chat needs attention'"
key ctrl+alt+u
sleep 3
cap "$OUT/d16-09-reopen-one-left.png"; B9=$(md5 "$OUT/d16-09-reopen-one-left.png")

echo "--- [10] Ctrl+Alt+U → TOGGLE-CLOSES"
key ctrl+alt+u
sleep 2.5
cap "$OUT/d16-10-toggle-closed.png"; B10=$(md5 "$OUT/d16-10-toggle-closed.png")

echo "--- [11] Shift+Escape → clear all unread"
key shift+Escape
sleep 2.5
cap "$OUT/d16-11-cleared-all.png"; B11=$(md5 "$OUT/d16-11-cleared-all.png")

echo "--- [12] Ctrl+Alt+U → EMPTY STATE"
key ctrl+alt+u
sleep 3
cap "$OUT/d16-12-empty-state.png"; B12=$(md5 "$OUT/d16-12-empty-state.png")

echo "--- [13] Escape → closes"
key Escape
sleep 2
cap "$OUT/d16-13-escape-closed.png"; B13=$(md5 "$OUT/d16-13-escape-closed.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null

{
  echo "d16-01-entry:            $B1   (VLM: footer 'App-server online')"
  echo "d16-01b-nux-cleared:     $BN"
  echo "d16-02a-typed:           $BA"
  echo "d16-02b-chatA-created:   $B2   (VLM: sidebar row 'alpha activity chat')"
  echo "d16-03-marked-unread:    $B3   (VLM: dot on A + 'Chat marked unread')"
  echo "d16-04-chatB-created:    $B4   (VLM: B selected, A dot stays)"
  echo "d16-05-both-flagged:     $B5   (VLM: dots on A AND B)"
  echo "d16-06-activity-open:    $B6   (VLM: 'Activity' panel, '2 chats need attention', rows A+B, first selected)"
  echo "d16-07-down-second-row:  $B7   (VLM: selection moved to second row)"
  echo "d16-08-enter-jumped:     $B8   (VLM: panel GONE, jumped chat open, its dot cleared, other dot stays)"
  echo "d16-09-reopen-one-left:  $B9   (VLM: '1 chat needs attention', one row)"
  echo "d16-10-toggle-closed:    $B10  (VLM: panel gone)"
  echo "d16-11-cleared-all:      $B11  (VLM: no dots)"
  echo "d16-12-empty-state:      $B12  (VLM: 'No chats need attention' + guidance)"
  echo "d16-13-escape-closed:    $B13  (VLM: panel gone)"
} > "$OUT/d16-md5.txt"
cat "$OUT/d16-md5.txt"
echo "--- app.log tail ---"
tail -8 "$OUT/app.log"
echo "D16 SCENE COMPLETE — VLM-read frames 03/05/06/07/08/09/12"
