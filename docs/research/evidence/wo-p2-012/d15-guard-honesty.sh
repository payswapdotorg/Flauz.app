#!/bin/bash
# D15 — WO-P2-012 GUI verification (guard honesty, six evidenced states), r3 branch.
#
# OFFLINE-HONEST SCOPE NOTE (D12/D14 doctrine): F-A6 (commit-or-push with a
# pending PR -> "A Git workflow is already running.") requires arranging
# state.git.pending_pull_request — not constructible in the offline lab —
# recorded NOT RUN here, covered deterministically by the focused CI unit
# tests on the r3 branch. Everything else is probed on the REAL binary:
#
#   01 entry baseline (no chat selected; footer "App-server online")
#   02 F-A1  Ctrl+Shift+A (no chat)  -> status "Select a chat before archiving it."
#   03 F-A2  Ctrl+Alt+P   (no chat)  -> status "Select a chat before pinning or unpinning it."
#   04 F-A3  Ctrl+Alt+R   (no chat)  -> status "Select a chat before renaming it."
#   05 F-A6  palette "commit" query frame (record; pending-PR guard = unit-covered, NOT RUN)
#   06 F-D1  composer "/review" + ctrl+Return (no chat) -> VISIBLE submission:
#            new thread row + transcript shows the literal "/review" text
#            (fall-through — input is NOT swallowed, no silent no-op)
#   07 F-D2  new chat via composer, then "/compact" + ctrl+Return DURING the
#            thread-runtime load -> composer error
#            "Wait for the chat to finish loading before compacting context."
#   08 settle frame (post-state record)
#
# PASS: md5 frame deltas + VLM reads show the honest statuses on the real UI.
# Usage: d15-guard-honesty.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d15}"
DISP=":109"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/parity-lab/lib:${LD_LIBRARY_PATH:-}"

D15DATA=/tmp/d15-data
rm -rf "$D15DATA"; mkdir -p "$D15DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D15DATA/state.sqlite3"
# CodexHome::resolve requires the dir to EXIST (D11r run-3 negative control).
mkdir -p "$D15DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D15=$(mktemp -d /tmp/d15-home.XXXXXX)
mkdir -p "$HOME_D15/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D15" XDG_RUNTIME_DIR="$HOME_D15/xdg" \
  CODEX_RS_DATA_DIR="$D15DATA" \
  CODEX_HOME="$D15DATA/codex-home" \
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
    rendered=1; break
  fi
done
[ "$rendered" = "1" ] || echo "WARN — no >30KB frame in 150s; running scene anyway"

sleep 5
grep -iE 'app.?server|connect' "$OUT/app.log" | head -5 || echo "(no connection lines in app.log yet)"

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1
# first-run NUX modal settle (D11r pattern)
key Escape; sleep 1.5

echo "--- [01] entry baseline (expect footer 'App-server online'; no chat selected)"
sleep 3
cap "$OUT/d15-01-entry.png"; B1=$(md5 "$OUT/d15-01-entry.png")

echo "--- [02] F-A1: Ctrl+Shift+A, no chat selected (expect status 'Select a chat before archiving it.')"
cap "$OUT/d15-02a-pre.png"; P2=$(md5 "$OUT/d15-02a-pre.png")
key ctrl+shift+a; sleep 2.5
cap "$OUT/d15-02b-status.png"; B2=$(md5 "$OUT/d15-02b-status.png")

echo "--- [03] F-A2: Ctrl+Alt+P, no chat selected (expect 'Select a chat before pinning or unpinning it.')"
sleep 3
key ctrl+alt+p; sleep 2.5
cap "$OUT/d15-03-status.png"; B3=$(md5 "$OUT/d15-03-status.png")

echo "--- [04] F-A3: Ctrl+Alt+R, no chat selected (expect 'Select a chat before renaming it.')"
sleep 3
key ctrl+alt+r; sleep 2.5
cap "$OUT/d15-04-status.png"; B4=$(md5 "$OUT/d15-04-status.png")
# dismiss any rename dialog that could have (wrongly) opened
key Escape; sleep 1.5

echo "--- [05] F-A6 record: palette 'commit' query (pending-PR guard = unit-covered, NOT RUN offline)"
key ctrl+k; sleep 2.5
type_text "commit"; sleep 2
cap "$OUT/d15-05-palette-commit.png"; B5=$(md5 "$OUT/d15-05-palette-commit.png")
key Escape; sleep 1.5

echo "--- [06] F-D1: composer '/review' + ctrl+Return at entry (expect VISIBLE submission: thread row + literal '/review' in transcript)"
ladder; sleep 0.5
type_text "/review"; sleep 1.0
key ctrl+Return; sleep 6
cap "$OUT/d15-06-review-fallthrough.png"; B6=$(md5 "$OUT/d15-06-review-fallthrough.png")
# settle any retry toast (unauthenticated turn execution fails — non-blocking)
key Escape; sleep 2

echo "--- [07] F-D2: new chat then '/compact' during runtime load (expect 'Wait for the chat to finish loading before compacting context.')"
ladder; sleep 0.5
type_text "d15 compact probe"; sleep 1.0
key ctrl+Return; sleep 1.5
ladder; sleep 0.3
type_text "/compact"; sleep 0.8
key ctrl+Return; sleep 3
cap "$OUT/d15-07-compact-loading-guard.png"; B7=$(md5 "$OUT/d15-07-compact-loading-guard.png")

echo "--- [08] settle frame (post-state record)"
sleep 5
cap "$OUT/d15-08-settle.png"; B8=$(md5 "$OUT/d15-08-settle.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null

{
  echo "d15-01-entry:              $B1"
  echo "d15-02a-pre:               $P2"
  echo "d15-02b-ctrl+shift+a:      $B2   (F-A1: expect status bar change vs pre + VLM 'archiving' text)"
  echo "d15-03-ctrl+alt+p:         $B3   (F-A2: expect 'pinning or unpinning' status)"
  echo "d15-04-ctrl+alt+r:         $B4   (F-A3: expect 'renaming' status; NO dialog must open)"
  echo "d15-05-palette-commit:     $B5   (F-A6 record; guard unit-covered — NOT RUN offline)"
  echo "d15-06-review-fallthrough: $B6   (F-D1: expect thread row + literal '/review' in transcript)"
  echo "d15-07-compact-guard:      $B7   (F-D2: expect 'Wait for the chat to finish loading...' composer error)"
  echo "d15-08-settle:             $B8   (post-state record)"
} | tee "$OUT/d15-md5.txt"

echo "SCENE COMPLETE — VLM adjudication pending on frames 02b/03/04/06/07"
