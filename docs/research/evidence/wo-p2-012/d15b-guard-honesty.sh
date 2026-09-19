#!/bin/bash
# D15b — WO-P2-012 GUI verification, corrected re-run (guard honesty).
#
# D15 findings driving this re-run:
#   (a) the entry surface shows a persistent "Sign in to get started" hero
#       card; Ctrl+K reaches the app through it (palette opened at D15-05)
#       but the archive/pin/rename CHORDS produced byte-identical frames —
#       the card's focus scope is the prime suspect. D15b deflects focus
#       (neutral click on the main heading) before each chord and captures
#       at TWO timings (1.2s + 2.5s) to catch transient statuses.
#   (b) F-D1 VERIFIED in D15 (the "/review" thread row + context card) —
#       re-run with a later capture (+10s) for a clean row frame.
#   (c) F-D2 was probed 5.5s after thread creation (runtime already
#       loaded) and the /compact submit never left the composer (the
#       command-autocomplete pill was staging). D15b submits /compact
#       IMMEDIATELY after the thread-creating message (no ladder, no
#       sleeps) to land inside the runtime-loading window.
#   (d) F-A6 stays NOT RUN offline (pending-PR state not constructible;
#       unit-covered on CI) — no palette probe needed (D15-05 already
#       recorded the honest "No matches" absence for "commit").
#
# Usage: d15b-guard-honesty.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/wo-p2-012/d15b}"
DISP=":110"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/parity-lab/lib:${LD_LIBRARY_PATH:-}"

D15DATA=/tmp/d15b-data
rm -rf "$D15DATA"; mkdir -p "$D15DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D15DATA/state.sqlite3"
mkdir -p "$D15DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D15=$(mktemp -d /tmp/d15b-home.XXXXXX)
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
neutral() { DISPLAY=$DISP xdotool mousemove 720 300 click 1; sleep 0.5; }

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

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 2

echo "--- [01] entry baseline (sign-in hero card expected)"
sleep 3
cap "$OUT/d15b-01-entry.png"; B1=$(md5 "$OUT/d15b-01-entry.png")

echo "--- [01b] neutral click on main heading (deflect card focus)"
neutral
cap "$OUT/d15b-01b-neutral.png"; B1b=$(md5 "$OUT/d15b-01b-neutral.png")

echo "--- [02] F-A1: ctrl+shift+a after neutral click (expect 'Select a chat before archiving it.')"
key ctrl+shift+a; sleep 1.2
cap "$OUT/d15b-02-early.png"; B2e=$(md5 "$OUT/d15b-02-early.png")
sleep 1.5
cap "$OUT/d15b-02-late.png"; B2l=$(md5 "$OUT/d15b-02-late.png")

echo "--- [03] F-A2: ctrl+alt+p (expect 'Select a chat before pinning or unpinning it.')"
sleep 3; neutral
key ctrl+alt+p; sleep 1.2
cap "$OUT/d15b-03-early.png"; B3e=$(md5 "$OUT/d15b-03-early.png")
sleep 1.5
cap "$OUT/d15b-03-late.png"; B3l=$(md5 "$OUT/d15b-03-late.png")

echo "--- [04] F-A3: ctrl+alt+r (expect 'Select a chat before renaming it.'; no dialog)"
sleep 3; neutral
key ctrl+alt+r; sleep 1.2
cap "$OUT/d15b-04-early.png"; B4e=$(md5 "$OUT/d15b-04-early.png")
sleep 1.5
cap "$OUT/d15b-04-late.png"; B4l=$(md5 "$OUT/d15b-04-late.png")
key Escape; sleep 1

echo "--- [05] F-D1: '/review' visible submission (fall-through; row at +10s)"
ladder; sleep 0.5
type_text "/review"; sleep 1.0
key ctrl+Return; sleep 10
cap "$OUT/d15b-05-review-thread.png"; B5=$(md5 "$OUT/d15b-05-review-thread.png")

echo "--- [06] F-D2: '/compact' IMMEDIATELY after thread creation (runtime-loading window)"
ladder; sleep 0.3
type_text "d15b probe"; sleep 0.8
key ctrl+Return
# no sleep, no ladder — the composer keeps focus after submit; type immediately
type_text --clearmodifiers "/compact" 2>/dev/null || type_text "/compact"
sleep 0.4
key ctrl+Return; sleep 1.5
cap "$OUT/d15b-06-compact-early.png"; B6e=$(md5 "$OUT/d15b-06-compact-early.png")
sleep 2.5
cap "$OUT/d15b-06-compact-late.png"; B6l=$(md5 "$OUT/d15b-06-compact-late.png")

echo "--- [07] settle frame"
sleep 5
cap "$OUT/d15b-07-settle.png"; B7=$(md5 "$OUT/d15b-07-settle.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null

{
  echo "d15b-01-entry:        $B1"
  echo "d15b-01b-neutral:     $B1b   (focus deflection: expect ~entry or minor hover delta)"
  echo "d15b-02-early:        $B2e   (F-A1 @1.2s)"
  echo "d15b-02-late:         $B2l   (F-A1 @2.7s — status may fade)"
  echo "d15b-03-early:        $B3e   (F-A2 @1.2s)"
  echo "d15b-03-late:         $B3l   (F-A2 @2.7s)"
  echo "d15b-04-early:        $B4e   (F-A3 @1.2s; no rename dialog must open)"
  echo "d15b-04-late:         $B4l   (F-A3 @2.7s)"
  echo "d15b-05-review:       $B5   (F-D1: thread row + literal '/review' at +10s)"
  echo "d15b-06-early:        $B6e   (F-D2 @1.5s: composer error guidance?)"
  echo "d15b-06-late:         $B6l   (F-D2 @4s)"
  echo "d15b-07-settle:       $B7"
} | tee "$OUT/d15b-md5.txt"

echo "SCENE COMPLETE — VLM adjudication pending on 01b/02/03/04/05/06"
