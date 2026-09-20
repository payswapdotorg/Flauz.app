#!/bin/bash
# rg-a11y-03.sh — WO-UX-001 follow-up scene RG-A11Y-03: the precondition-
# correct re-probes for the legs a01/a02 could not evidence because their
# scripted preconditions hit the app's own guards:
#
#   - a01 LEG-3/4 ran ctrl+` / ctrl+shift+b ON the Settings page (the
#     escape ladder never exits Settings — a01-06 evidence) where those
#     chords are not bound (requires_selected_chat / requires_task_workspace
#     guards in ui.rs).
#   - a02 P2/P3 ran Find + PTY probes against the SEEDED chat (synthetic
#     rollout = no live task workspace) — FindInThread and the terminal PTY
#     are task-workspace-guarded, so the probes hit honest empty states.
#   - a01 LEG-6/7: begin_new_chat does not focus the composer (ui.rs
#     begin_new_chat vs begin_new_chat_with_prompt), so the typed message
#     never submitted -> no task -> no selection -> mark/swap hit guards.
#
# This scene re-runs those legs with the CORRECT preconditions, keyboard-only:
#   A  LIVE CHAT (boot-default composer focus): type -> ctrl+Return ->
#      the live chat exists, is selected, has a task workspace.
#   B  FIND IN THREAD (live task): ctrl+f -> bar opens -> query -> Enter
#      advances the active occurrence -> Escape closes.
#   C  TERMINAL PTY (live task): ctrl+` -> dock opens WITH a PTY -> type
#      echo -> output renders -> ctrl+` closes.
#   D  ATTENTION (with selection): ctrl+shift+u marks -> ctrl+alt+a jumps ->
#      shift+esc clears all.
#   E  CHAT SWAP (2 chats): ctrl+shift+] / ctrl+shift+[ moves selection.
#   F  ctrl+n FOCUS GAP (documents the finding): ctrl+n -> type probe ->
#      where does the text land?
#   G  SETTINGS EXCURSION (documents the exit-path truth): ctrl+, ->
#      Escape x3 (still Settings?) -> ctrl+n exits -> type probe (focus
#      state after the excursion).
#
# Usage: rg-a11y-03.sh <binary-path> [evidir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/battery-rg/a11y-03}"
DISP=":126"
RUNTIME=/home/z/parity-lab/runtime/codex

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/bin/../lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing"; exit 1; }
mkdir -p "$OUT"

cleanup() {
  kill "${APP_PID:-0}" 2>/dev/null; pkill -P "${APP_PID:-0}" 2>/dev/null
  pkill -x picom 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
}
trap cleanup EXIT
say() { echo "[$(date -u +%H:%M:%S)] $*" | tee -a "$OUT/PROGRESS.txt"; }
cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i "$DISP" -frames:v 1 "$1" 2>/dev/null; }
md5() { md5sum "$1" 2>/dev/null | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 100 "$1"; }

DATA=/tmp/rg-a11y-data
[ -f "$DATA/state.sqlite3" ] || { echo "FATAL: seeded state missing (run rg-a11y-01.sh first)"; exit 1; }

pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb "$DISP" -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 2

HOME_A=$(mktemp -d /tmp/rg-a11y3-home.XXXXXX); mkdir -p "$HOME_A/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_A" XDG_RUNTIME_DIR="$HOME_A/xdg" \
  CODEX_RS_DATA_DIR="$DATA" CODEX_HOME="$DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!
say "boot APP_PID=$APP_PID (precondition-correct re-probes)"

rendered=0
for i in $(seq 1 40); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { say "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true   # window nudge only (pre-journey)
  cap "$OUT/_probe.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
  [ "${SZ:-0}" -gt 30000 ] && { say "UI rendered after $((i*5))s"; rendered=1; break; }
done
rm -f "$OUT/_probe.png"
WID=$(xwininfo -display "$DISP" -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+" | head -1)
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
for i in $(seq 1 20); do CPID=$(pgrep -P $APP_PID -f "runtime/codex" | head -1); [ -n "$CPID" ] && break; sleep 2; done
sleep 8

say "LEG-A LIVE CHAT (boot-default composer focus)"
type_ "a03 live chat one"; sleep 1.2
cap "$OUT/a03-01-composer-typed.png"; say "A1 $(md5 "$OUT/a03-01-composer-typed.png")"
key ctrl+Return; sleep 15
cap "$OUT/a03-02-chat-submitted.png"; say "A2 $(md5 "$OUT/a03-02-chat-submitted.png")"

say "LEG-B FIND IN THREAD (live task workspace)"
key ctrl+f; sleep 2
cap "$OUT/a03-03-find-open.png"; say "B1 $(md5 "$OUT/a03-03-find-open.png")"
type_ "one"; sleep 1.5
cap "$OUT/a03-04-find-query.png"; say "B2 $(md5 "$OUT/a03-04-find-query.png")"
key Return; sleep 1.5
cap "$OUT/a03-05-find-next.png"; say "B3 $(md5 "$OUT/a03-05-find-next.png")"
key Escape; sleep 1.5
cap "$OUT/a03-06-find-closed.png"; say "B4 $(md5 "$OUT/a03-06-find-closed.png")"

say "LEG-C TERMINAL PTY (live task workspace)"
key ctrl+grave; sleep 2.5
cap "$OUT/a03-07-terminal-open.png"; say "C1 $(md5 "$OUT/a03-07-terminal-open.png")"
type_ "echo a03-pty-ok"; sleep 0.8
key Return; sleep 2.5
cap "$OUT/a03-08-terminal-echo.png"; say "C2 $(md5 "$OUT/a03-08-terminal-echo.png")"
key ctrl+grave; sleep 2
cap "$OUT/a03-09-terminal-closed.png"; say "C3 $(md5 "$OUT/a03-09-terminal-closed.png")"

say "LEG-D ATTENTION (with selection)"
key ctrl+shift+u; sleep 2.5
cap "$OUT/a03-10-marked-unread.png"; say "D1 $(md5 "$OUT/a03-10-marked-unread.png")"
key ctrl+alt+a; sleep 2.5
cap "$OUT/a03-11-jump.png"; say "D2 $(md5 "$OUT/a03-11-jump.png")"
key shift+Escape; sleep 2.5
cap "$OUT/a03-12-clear-all.png"; say "D3 $(md5 "$OUT/a03-12-clear-all.png")"

say "LEG-E CHAT SWAP (2 chats: live + seeded)"
key ctrl+shift+bracketright; sleep 2.5
cap "$OUT/a03-13-swap-next.png"; say "E1 $(md5 "$OUT/a03-13-swap-next.png")"
key ctrl+shift+bracketright; sleep 2.5
cap "$OUT/a03-14-swap-next2.png"; say "E2 $(md5 "$OUT/a03-14-swap-next2.png")"
key ctrl+shift+bracketleft; sleep 2.5
cap "$OUT/a03-15-swap-prev.png"; say "E3 $(md5 "$OUT/a03-15-swap-prev.png")"

say "LEG-F ctrl+n FOCUS GAP (finding documentation)"
key ctrl+a; sleep 0.3; key Delete; sleep 0.5
key ctrl+n; sleep 2.5
type_ "a03-newchat-probe"; sleep 1.2
cap "$OUT/a03-16-newchat-focus.png"; say "F1 $(md5 "$OUT/a03-16-newchat-focus.png")"
key ctrl+a; sleep 0.3; key Delete; sleep 0.5

say "LEG-G SETTINGS EXCURSION (exit-path truth)"
key ctrl+comma; sleep 2.5
cap "$OUT/a03-17-settings-open.png"; say "G1 $(md5 "$OUT/a03-17-settings-open.png")"
key Escape; sleep 1; key Escape; sleep 1; key Escape; sleep 2
cap "$OUT/a03-18-settings-esc3.png"; say "G2 $(md5 "$OUT/a03-18-settings-esc3.png")"
key ctrl+n; sleep 2.5
type_ "a03-after-settings-probe"; sleep 1.2
cap "$OUT/a03-19-post-excursion-focus.png"; say "G3 $(md5 "$OUT/a03-19-post-excursion-focus.png")"

say "RG-A11Y-03 COMPLETE — 19 frames, keyboard-only after window placement"
