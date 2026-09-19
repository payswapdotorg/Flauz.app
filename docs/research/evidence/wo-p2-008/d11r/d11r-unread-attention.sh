#!/bin/bash
# D11r — WO-P2-008 full-flow scene, CORRECTED RE-RUN (RWO-022 FW-5 + FW-7).
#
# The original D11 (2026-09-19 05:00 UTC) could not exercise chat creation for
# TWO stacked causes (adjudicated by RWO-022 T4 + EQ-1):
#   (a) environment gap  — the lab's pinned Codex CLI runtime (lost in the
#       post-Task-49 cleanups) was absent, so the app-server never connected
#       ("Couldn't connect to the Codex app-server" / "No chats");
#   (b) script key defect — the scene submitted with plain `Return`, which is
#       NOT the composer submit key (D8c/D9 calibration: submits are
#       CTRL+RETURN) — chat creation would have failed even with a runtime.
#
# This re-run fixes BOTH: the runtime is restored + pinned
# (npm @openai/codex@0.146.0-alpha.3.1-linux-x64 — the exact oracle pin from
# docs/platform-support.md — at /home/z/parity-lab/runtime/codex, sha256
# ae77c5e73db36d15c131381c5d620278abed999650867a7490b13384e1f5842d) and is
# injected via CODEX_RS_CODEX_BIN (resolve_codex_binary's explicit-override
# path); every composer submit is ctrl+Return.
#
# Binary under test: current main (3c812a7 = WO-P2-008..011 all merged).
#
# Scene (D9-proven unauthenticated pattern: thread creation succeeds, turn
# execution fails with a retry toast — the toast does not block the sidebar):
#   01 entry baseline (footer MUST read "App-server online" — EQ-1 record)
#   02 create chat A via composer (ctrl+Return) → sidebar row + retry toast;
#      Escape settles the toast
#   03 Ctrl+Shift+U on A → dot on A's row + "Chat marked unread"
#   04 Ctrl+N chat B ("beta background chat" + ctrl+Return) → B selected, A dot stays
#   05 Ctrl+Alt+A → jump to A (visit clears A's dot)
#   06 Ctrl+Shift+U again → dot back on A
#   07 Shift+Escape → "Cleared unread indicators for 1 chat", dot gone
#
# PASS: md5 frame sequence + VLM reads show dot/jump/clear on REAL thread rows.
# Usage: d11r-unread-attention.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d11r}"
DISP=":103"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D11DATA=/tmp/d11r-data
rm -rf "$D11DATA"; mkdir -p "$D11DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D11DATA/state.sqlite3"
# CodexHome::resolve (codex-platform app_server.rs) requires the dir to EXIST;
# the CLI hard-exits on a missing CODEX_HOME. Pre-wipe session.sh set CODEX_HOME
# for exactly this reason (D9's "App-server online").
mkdir -p "$D11DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D11=$(mktemp -d /tmp/d11r-home.XXXXXX)
mkdir -p "$HOME_D11/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D11" XDG_RUNTIME_DIR="$HOME_D11/xdg" \
  CODEX_RS_DATA_DIR="$D11DATA" \
  CODEX_HOME="$D11DATA/codex-home" \
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

# EQ-1 record: the runtime connection state must be visible in the log/footer
sleep 5
grep -iE 'app.?server|connect' "$OUT/app.log" | head -5 || echo "(no connection lines in app.log yet)"

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

echo "--- [01] entry baseline (expect footer 'App-server online')"
sleep 3
cap "$OUT/d11r-01-entry.png"; B1=$(md5 "$OUT/d11r-01-entry.png")

echo "--- [02] create chat A (composer, ctrl+Return submit)"
# dismiss the model-NUX modal if present (Escape), then focus the composer via
# the D9-calibrated click ladder (zone y749-829 at x802 — a single click at
# y700 lands on the central first-run sign-in card and the typing is eaten)
key Escape; sleep 2
cap "$OUT/d11r-01b-nux-cleared.png"; BN=$(md5 "$OUT/d11r-01b-nux-cleared.png")
ladder
sleep 0.5
type_text "alpha attention chat"
sleep 1.0
cap "$OUT/d11r-02a-typed.png"; BA=$(md5 "$OUT/d11r-02a-typed.png")
key ctrl+Return
sleep 12
cap "$OUT/d11r-02b-chatA-created.png"; B2=$(md5 "$OUT/d11r-02b-chatA-created.png")
# settle the unauthenticated retry toast (D9 step-2 recipe)
key Escape
sleep 2

echo "--- [03] Ctrl+Shift+U → mark A unread (dot + status)"
sleep 1
key ctrl+shift+u
sleep 2.5
cap "$OUT/d11r-03-marked-unread.png"; B3=$(md5 "$OUT/d11r-03-marked-unread.png")

echo "--- [04] Ctrl+N → chat B (A keeps its dot in the sidebar)"
key ctrl+n
sleep 3
ladder
sleep 0.5
type_text "beta background chat"
sleep 1.0
key ctrl+Return
sleep 12
cap "$OUT/d11r-04-chatB-created.png"; B4=$(md5 "$OUT/d11r-04-chatB-created.png")
key Escape
sleep 2

echo "--- [05] Ctrl+Alt+A → jump to A (visit clears the dot)"
sleep 1
key ctrl+alt+a
sleep 3
cap "$OUT/d11r-05-jumped-to-A.png"; B5=$(md5 "$OUT/d11r-05-jumped-to-A.png")

echo "--- [06] Ctrl+Shift+U again → dot back on A"
key ctrl+shift+u
sleep 2.5
cap "$OUT/d11r-06-remarked.png"; B6=$(md5 "$OUT/d11r-06-remarked.png")

echo "--- [07] Shift+Escape → clear all unread"
key shift+Escape
sleep 2.5
cap "$OUT/d11r-07-cleared.png"; B7=$(md5 "$OUT/d11r-07-cleared.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null

{
  echo "d11r-01-entry:            $B1   (VLM: footer 'App-server online')"
  echo "d11r-01b-nux-cleared:     $BN"
  echo "d11r-02a-typed:           $BA"
  echo "d11r-02b-chatA-created:   $B2   (VLM: sidebar row 'alpha attention chat')"
  echo "d11r-03-marked-unread:    $B3   (VLM: dot on A + 'Chat marked unread')"
  echo "d11r-04-chatB-created:    $B4   (VLM: B selected, A dot stays)"
  echo "d11r-05-jumped-to-A:      $B5   (VLM: A selected, dot cleared)"
  echo "d11r-06-remarked:         $B6   (VLM: dot back on A)"
  echo "d11r-07-cleared:          $B7   (VLM: 'Cleared unread indicators for 1 chat', dot gone)"
} > "$OUT/d11r-md5.txt"
cat "$OUT/d11r-md5.txt"
echo "--- app.log tail ---"
tail -8 "$OUT/app.log"
echo "D11r SCENE COMPLETE — VLM-read frames 01/02b/03/04/05/06/07"
