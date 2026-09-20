#!/bin/bash
# rg-a11y-02.sh — WO-UX-001 scene RG-A11Y-02: focus-order probes for the
# source-unverifiable findings (accessibility-inventory §a/§d).
#
#   P1 PALETTE-CLOSE FOCUS LANDING (the WO-P2-017 axis): Ctrl+K -> Escape ->
#      immediately type a probe string; where does the text land? (composer =
#      focus restored to the composer surface; nowhere visible = unmanaged
#      landing — the pre-fix state this probe documents at the current tree).
#      Same probe for the keyboard-shortcuts overlay close (Ctrl+/).
#   P2 FIND ACTIVE OCCURRENCE (non-color distinction): seeded multi-match
#      chat -> Ctrl+F -> query "turn" -> first occurrence emphasized ->
#      Enter/arrow to advance -> capture pair (VLM adjudicates the active
#      occurrence moving = a non-color/position channel beyond color alone).
#   P3 TERMINAL PTY FOCUS TRANSFER: Ctrl+` -> immediately type (no click) ->
#      output proves the open path transferred focus into the PTY.
#   P4 CONFIRMATION-MODAL TRAPS (the WO-P2-019 axis): the Settings-reachable
#      destructive modals; Tab x3 inside; captures (focus must stay confined
#      — Cancel/confirm highlight moves within the modal). Documents the
#      current-tree state; the binding verification is 019's own D-scene at
#      the merged tree.
#
# Usage: rg-a11y-02.sh <binary-path> [evidir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/battery-rg/a11y-02}"
DISP=":125"
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

# reuse the a11y-01 seeded DATA if present (same long chat), else re-seed
DATA=/tmp/rg-a11y-data
if [ ! -d "$DATA/codex-home/sessions" ]; then
  echo "FATAL: run rg-a11y-01.sh first (seeded state required)"; exit 1
fi

pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb "$DISP" -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 2

HOME_P=$(mktemp -d /tmp/rg-a11yp-home.XXXXXX); mkdir -p "$HOME_P/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_P" XDG_RUNTIME_DIR="$HOME_P/xdg" \
  CODEX_RS_DATA_DIR="$DATA" CODEX_HOME="$DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!
say "boot APP_PID=$APP_PID (focus probes)"

rendered=0
for i in $(seq 1 40); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { say "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
  cap "$OUT/_probe.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
  [ "${SZ:-0}" -gt 30000 ] && { say "UI rendered after $((i*5))s"; rendered=1; break; }
done
rm -f "$OUT/_probe.png"
WID=$(xwininfo -display "$DISP" -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+" | head -1)
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
for i in $(seq 1 20); do CPID=$(pgrep -P $APP_PID -f "runtime/codex" | head -1); [ -n "$CPID" ] && break; sleep 2; done
sleep 8

say "P1a PALETTE-CLOSE focus landing"
key Escape; sleep 1
key ctrl+k; sleep 2.5
cap "$OUT/a02-01-palette-open.png"
key Escape; sleep 1.2
type_ "focusprobe-after-palette-close"; sleep 1.2
cap "$OUT/a02-02-palette-close-landing.png"; say "P1a $(md5 "$OUT/a02-02-palette-close-landing.png")"
key ctrl+a; sleep 0.3; key Delete   # clean the composer for the next probe
sleep 0.5

say "P1b OVERLAY-CLOSE focus landing"
key ctrl+slash; sleep 2.5
cap "$OUT/a02-03-overlay-open.png"
key Escape; sleep 1.2
key Escape; sleep 1
type_ "focusprobe-after-overlay-close"; sleep 1.2
cap "$OUT/a02-04-overlay-close-landing.png"; say "P1b $(md5 "$OUT/a02-04-overlay-close-landing.png")"
key ctrl+a; sleep 0.3; key Delete
sleep 0.5

say "P2 FIND active occurrence (seeded multi-match chat selected)"
DISPLAY=$DISP xdotool mousemove 130 413 click 1 2>/dev/null || true   # select the seeded long chat (calibrated row Y=413)
sleep 2.5
key ctrl+f; sleep 1.5
type_ "turn"; sleep 2
cap "$OUT/a02-05-find-first.png"; say "P2a $(md5 "$OUT/a02-05-find-first.png")"
key Return; sleep 1.5
cap "$OUT/a02-06-find-next.png"; say "P2b $(md5 "$OUT/a02-06-find-next.png")"
key Return; sleep 1.5
cap "$OUT/a02-07-find-next2.png"; say "P2c $(md5 "$OUT/a02-07-find-next2.png")"
key Escape; sleep 1.5

say "P3 TERMINAL PTY focus transfer (type immediately, no click)"
key ctrl+grave; sleep 2.5
type_ "echo ptyfocus-probe-3"; sleep 0.8
key Return; sleep 2.5
cap "$OUT/a02-08-pty-focus.png"; say "P3 $(md5 "$OUT/a02-08-pty-focus.png")"
key ctrl+grave; sleep 2

say "P4 CONFIRMATION-MODAL traps (Settings-reachable, current-tree record)"
key ctrl+comma; sleep 2.5
cap "$OUT/a02-09-settings.png"
# reach a destructive confirm: Settings search "reset" (Reset keyboard
# shortcuts / Reset memories are the evidenced-trap family; the probe records
# the trap behavior on the reachable ones at this tree)
key ctrl+f; sleep 1
type_ "reset"; sleep 2
cap "$OUT/a02-10-settings-reset.png"; say "P4a $(md5 "$OUT/a02-10-settings-reset.png")"
key Escape; sleep 1; key Escape; sleep 2
say "P4 NOTE: the four WO-P2-019 modals (remote pairing / remote confirmation / account logout / plugin install) are flow-gated; their trap verification is 019's D-scene at the merged tree."

say "RG-A11Y-02 COMPLETE"
