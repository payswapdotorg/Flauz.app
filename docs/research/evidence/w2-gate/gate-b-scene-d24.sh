#!/bin/bash
# D24 — Wave-2 Gate B: the J-13 model-picker scene (MOD-001) + the J-04
# capability-gap scene (CAP-001) + the J-05/J-06 environment-rail
# regression (ENV-001) at the merged Wave-2 binary.
#
# CALIBRATION (2026-09-23, from the merged registrations):
#   Model picker (MOD-001): Ctrl+Alt+Shift+M chord; palette row
#     "Choose a model…" (WorkspaceShell group, requires a selected chat).
#   Capability gap (CAP-001): Ctrl+Alt+Shift+6 chord + the alt-^
#     shifted-symbol companion (the N6 gate-fix family); palette row
#     "Why is a capability unavailable?".
#   Environment rail (the F2 shell): Ctrl+Alt+Shift+3.
#   Palette: Ctrl+K (Unified, d20-proven). Anchor task: Ctrl+N + type +
#     ctrl+Return (d23-proven).
#
#   01 entry baseline (cold start)
#   02 anchor task — the live task surface (the picker control + the
#      Capabilities affordance both mount on the task surface — VLM
#      adjudicates their labels)
#   03 Ctrl+Alt+Shift+M — the picker panel opens (layer 4: keyboard
#      path); honest EMPTY state (no models configured: what a model
#      provides + the connection next-step; never a fake list)
#   04 picker close (scoped Escape) — the task surface returns (focus
#      not trapped, the d19 discipline)
#   05 palette Ctrl+K + "Choose a model" — the picker palette row
#      visible (layer 3) + Return lands on the panel
#   06 Ctrl+Alt+Shift+6 — the gap panel opens (layer 4); honest EMPTY
#      state (what the panel will do + the no-quietly-disappears promise)
#   07 gap panel — the Why?/What's-missing/Ways-to-unlock chain area
#      (the disclosure is fully implemented; the empty state is the
#      honest pre-wiring copy)
# 08 gap close (scoped Escape)
#   09 palette Ctrl+K + "Why is a capability" — the gap palette row
#      (layer 3) + Return lands
#   10 Ctrl+Alt+Shift+3 — the environment rail (the ENV-001 regression:
#      the honest empty state still renders at the merged binary; the
#      Wave-2 provider fabric is contracts-only — no UI change)
#   11 settle frame
#
# PASS: VLM reads show (a) the picker control labeled on the task
# surface at 02, (b) the picker panel at 03 with the honest empty state
# (no model list — cold start), (c) the palette row at 05 with the user
# title, (d) the Capabilities affordance at 02 + the gap panel at 06-07
# + its palette row at 09, (e) the environment rail empty state at 10.
# PASS/FAIL is adjudicated by the Lead from the frames + VLM reads.
# Usage: d24-w2-surfaces.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d24}"
DISP=":126"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D24DATA=/tmp/d24-data
rm -rf "$D24DATA"; mkdir -p "$D24DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D24DATA/state.sqlite3"
mkdir -p "$D24DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D24=$(mktemp -d /tmp/d24-home.XXXXXX)
mkdir -p "$HOME_D24/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D24" XDG_RUNTIME_DIR="$HOME_D24/xdg" \
  CODEX_RS_DATA_DIR="$D24DATA" \
  CODEX_HOME="$D24DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
refocus() {
  W=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
  [ -n "$W" ] && DISPLAY=$DISP xdotool windowfocus --sync "$W" 2>/dev/null || true
}
key() { refocus; DISPLAY=$DISP xdotool key "$1"; }
type_text() { refocus; DISPLAY=$DISP xdotool type --delay 60 "$1"; }

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

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

# D19 lesson: fresh-profile promo modal defensive dismissal (no-op absent).
DISPLAY=$DISP xdotool mousemove 580 619 click 1
sleep 2
DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
sleep 1

echo "--- [01] entry baseline (cold start)"
key Escape; sleep 2
cap "$OUT/d24-01-entry.png"; echo "B01 $(md5 "$OUT/d24-01-entry.png")"

echo "--- [02] anchor task — the live task surface (picker control + Capabilities affordance)"
key ctrl+n; sleep 2.5
type_text "Wave 2 gate anchor task"; sleep 1
key ctrl+Return; sleep 4
cap "$OUT/d24-02-task-surface.png"; echo "B02 $(md5 "$OUT/d24-02-task-surface.png")"

echo "--- [03] Ctrl+Alt+Shift+M — the picker panel (keyboard path; honest empty state)"
key ctrl+alt+shift+m; sleep 2.5
cap "$OUT/d24-03-picker-open.png"; echo "B03 $(md5 "$OUT/d24-03-picker-open.png")"

echo "--- [04] scoped Escape — picker closes, task surface returns (focus not trapped)"
key Escape; sleep 2
cap "$OUT/d24-04-picker-closed.png"; echo "B04 $(md5 "$OUT/d24-04-picker-closed.png")"

echo "--- [05] palette: 'Choose a model' row (palette discovery) + Return lands"
key ctrl+k; sleep 2
type_text "Choose a model"; sleep 2
cap "$OUT/d24-05-palette-model.png"; echo "B05 $(md5 "$OUT/d24-05-palette-model.png")"
key Return; sleep 2.5
cap "$OUT/d24-05b-palette-model-landed.png"; echo "B05b $(md5 "$OUT/d24-05b-palette-model-landed.png")"
key Escape; sleep 1.5

echo "--- [06] Ctrl+Alt+Shift+6 — the capability-gap panel (keyboard path; honest empty state)"
key ctrl+alt+shift+6; sleep 2.5
cap "$OUT/d24-06-gap-open.png"; echo "B06 $(md5 "$OUT/d24-06-gap-open.png")"

echo "--- [07] gap panel detail — the Why/What-missing/Ways-to-unlock chain area"
sleep 1.5
cap "$OUT/d24-07-gap-detail.png"; echo "B07 $(md5 "$OUT/d24-07-gap-detail.png")"

echo "--- [08] scoped Escape — gap closes"
key Escape; sleep 2
cap "$OUT/d24-08-gap-closed.png"; echo "B08 $(md5 "$OUT/d24-08-gap-closed.png")"

echo "--- [09] palette: 'Why is a capability' row (palette discovery) + Return lands"
key ctrl+k; sleep 2
type_text "Why is a capability"; sleep 2
cap "$OUT/d24-09-palette-gap.png"; echo "B09 $(md5 "$OUT/d24-09-palette-gap.png")"
key Return; sleep 2.5
cap "$OUT/d24-09b-palette-gap-landed.png"; echo "B09b $(md5 "$OUT/d24-09b-palette-gap-landed.png")"
key Escape; sleep 1.5

echo "--- [10] Ctrl+Alt+Shift+3 — the environment rail (the ENV-001 regression check)"
key ctrl+alt+shift+3; sleep 2.5
cap "$OUT/d24-10-env-rail.png"; echo "B10 $(md5 "$OUT/d24-10-env-rail.png")"

echo "--- [11] settle frame"
sleep 2
cap "$OUT/d24-11-settle.png"; echo "B11 $(md5 "$OUT/d24-11-settle.png")"

kill $APP_PID 2>/dev/null; kill $XPID 2>/dev/null; pkill -f "picom" 2>/dev/null
echo "D24 SCENE COMPLETE — evidence in $OUT"
