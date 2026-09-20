#!/bin/bash
# rg-a11y-03b.sh — WO-UX-001 re-run of the precondition-correct legs that
# a03 could not test: the shared /tmp/rg-a11y-data sqlite had persisted
# a02's final route, so a03 booted INTO Settings and legs A-E ran on the
# wrong surface (A1=A2 byte-identical to a01's settings frame; the swap
# "changes" were the E3 navigation away from Settings).
#
# This run uses a FRESH data dir (donor sqlite + the seeded rollout copied
# in) so the boot lands on the default route, then runs ONLY the legs that
# need the correct precondition (a03's F/G evidence stands):
#   A  LIVE CHAT (boot-default composer focus): type -> ctrl+Return.
#   B  FIND IN THREAD (live task): ctrl+f -> query -> Enter -> Escape.
#   C  TERMINAL PTY (live task): ctrl+` -> echo -> close.
#   D  ATTENTION (with selection): mark -> jump -> clear-all.
#   E  CHAT SWAP (live + seeded): next -> next (boundary no-op) -> prev.
#
# Usage: rg-a11y-03b.sh <binary-path> [evidir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/battery-rg/a11y-03b}"
DISP=":127"
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

# FRESH data dir: donor sqlite + the a01 seeded long chat (NO persisted route)
DATA=/tmp/rg-a11y3b-data
rm -rf "$DATA"; mkdir -p "$DATA/codex-home/sessions"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
[ -n "$DONOR" ] || { echo "FATAL: no donor"; exit 1; }
cp "$DONOR" "$DATA/state.sqlite3"
mkdir -p "$DATA/codex-home/sessions/2026/09/20"
cp /tmp/rg-a11y-data/codex-home/sessions/2026/09/20/rollout-*.jsonl \
   "$DATA/codex-home/sessions/2026/09/20/"

pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb "$DISP" -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 2

HOME_B=$(mktemp -d /tmp/rg-a11y3b-home.XXXXXX); mkdir -p "$HOME_B/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_B" XDG_RUNTIME_DIR="$HOME_B/xdg" \
  CODEX_RS_DATA_DIR="$DATA" CODEX_HOME="$DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!
say "boot APP_PID=$APP_PID (fresh data dir — default route)"

rendered=0
for i in $(seq 1 40); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { say "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true   # window nudge only
  cap "$OUT/_probe.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
  [ "${SZ:-0}" -gt 30000 ] && { say "UI rendered after $((i*5))s"; rendered=1; break; }
done
rm -f "$OUT/_probe.png"
WID=$(xwininfo -display "$DISP" -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+" | head -1)
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
for i in $(seq 1 20); do CPID=$(pgrep -P $APP_PID -f "runtime/codex" | head -1); [ -n "$CPID" ] && break; sleep 2; done
sleep 8

say "LEG-A LIVE CHAT (boot-default composer focus, default route)"
cap "$OUT/a03-00-entry.png"; say "A0 $(md5 "$OUT/a03-00-entry.png")"
type_ "a03b live chat one"; sleep 1.2
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
type_ "echo a03b-pty-ok"; sleep 0.8
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

say "LEG-E CHAT SWAP (live at index 0, seeded after)"
key ctrl+shift+bracketright; sleep 2.5
cap "$OUT/a03-13-swap-next.png"; say "E1 $(md5 "$OUT/a03-13-swap-next.png")"
key ctrl+shift+bracketright; sleep 2.5
cap "$OUT/a03-14-swap-next2.png"; say "E2 $(md5 "$OUT/a03-14-swap-next2.png")"
key ctrl+shift+bracketleft; sleep 2.5
cap "$OUT/a03-15-swap-prev.png"; say "E3 $(md5 "$OUT/a03-15-swap-prev.png")"

say "RG-A11Y-03B COMPLETE — 16 frames"
