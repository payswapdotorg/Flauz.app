#!/bin/bash
# D25 — Wave-3 Gate B: the J-03 recovery surface (ORCH-003), the J-07
# agents view (ORCH-004), the J-10 save-flow panel (ORCH-004), and the
# J-11 workflows-list regression (the F2 shell, now ORCH-004's list
# states) at the merged Wave-3 binary (eff1322).
#
# CALIBRATION (2026-09-23, verified against the merged source at eff1322):
#   Recovery (ORCH-003): Ctrl+Alt+Shift+R chord (letter family — shift
#     reported truthfully, no symbol companion needed). With a task
#     selected and nothing recoverable (the gate binary has no live
#     harness wiring), the chord surfaces the honest guidance through
#     the command-status line: "This task is up to date — there's
#     nothing to pick up right now." — and NO banner renders (never a
#     permanent banner). Palette row "Resume this task where it left
#     off" / "Pick up an interrupted task with everything it kept"
#     (WorkspaceShell group). Scoped Escape ("FlauzRecovery") for the
#     banner path.
#   Agents view (ORCH-004): Ctrl+Alt+Shift+7 chord + the alt-&
#     shifted-symbol companion (the N6 gate-fix family — xdotool's
#     ctrl+alt+shift+7 produces the live "&" keystroke) — opens the
#     task rail's Agents section. Panel heading "Agents on this task";
#     empty title "No extra agents on this task yet" + the honest
#     wiring state; palette row "See who is working on this task" /
#     "Watch each agent's role, progress and what they're waiting for".
#     Dismissed through the rail's FlauzTaskRail scoped Escape.
#   Save flow (ORCH-004): Ctrl+Alt+Shift+S chord (letter family).
#     Panel heading "Save as a reusable workflow" + entry affordance +
#     the honest not-wired state ("Saving isn't wired to live tasks
#     yet" + what it will hold + the next step); palette row "Save this
#     task as a reusable workflow" / "Remember the successful steps of
#     this task so you can run them again". Scoped Escape
#     ("FlauzSaveWorkflow").
#   Workflows list (J-11 regression, the F2 shell surface enhanced by
#     ORCH-004): Ctrl+Alt+2 — list title "Your reusable workflows" +
#     the honest empty state "No reusable workflows yet" (+ what they
#     are + how to save one).
#   Palette: Ctrl+K (Unified, d20-proven). Anchor task: Ctrl+N + type +
#     ctrl+Return (d23/d24-proven).
#
#   01 entry baseline (cold start)
#   02 anchor task — the live task surface (the recovery/agents/save
#      affordances all ride the task context)
#   03 J-03 Ctrl+Alt+Shift+R — the honest nothing-to-pick-up guidance
#      (command-status line; NO permanent banner)
#   04 Escape settle — nothing traps; the task surface stands
#   05 palette "Resume this task" — the recovery row (layer 3) +
#      Return lands (the guidance fires again — B05b)
#   06 J-07 Ctrl+Alt+Shift+7 — the agents panel (heading + empty +
#      honest wiring state)
#   07 scoped Escape (rail) — the panel closes, task surface returns
#   08 palette "See who is working" — the agents row + Return lands
#   09 J-10 Ctrl+Alt+Shift+S — the save panel (heading + entry
#      affordance + honest not-wired state)
#   10 scoped Escape — the save panel closes
#   11 palette "Save this task" — the save row + Return lands
#   12 J-11 Ctrl+Alt+2 — the workflows list (honest empty state)
#   13 settle frame
#
# PASS: VLM reads show (a) the nothing-to-pick-up guidance at 03 (and
# no banner on the task surface), (b) the recovery palette row at 05,
# (c) the agents panel at 06 with "Agents on this task" + the honest
# not-yet state, (d) the agents palette row at 08, (e) the save panel
# at 09 with "Save as a reusable workflow" + the honest not-wired
# state, (f) the save palette row at 11, (g) the workflows empty state
# at 12. PASS/FAIL is adjudicated by the Lead from the frames + VLM
# reads.
# Usage: d25-w3-surfaces.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d25}"
DISP=":126"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D25DATA=/tmp/d25-data
rm -rf "$D25DATA"; mkdir -p "$D25DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D25DATA/state.sqlite3"
mkdir -p "$D25DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D25=$(mktemp -d /tmp/d25-home.XXXXXX)
mkdir -p "$HOME_D25/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D25" XDG_RUNTIME_DIR="$HOME_D25/xdg" \
  CODEX_RS_DATA_DIR="$D25DATA" \
  CODEX_HOME="$D25DATA/codex-home" \
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
cap "$OUT/d25-01-entry.png"; echo "B01 $(md5 "$OUT/d25-01-entry.png")"

echo "--- [02] anchor task — the live task surface"
key ctrl+n; sleep 2.5
type_text "Wave 3 gate anchor task"; sleep 1
key ctrl+Return; sleep 4
cap "$OUT/d25-02-task-surface.png"; echo "B02 $(md5 "$OUT/d25-02-task-surface.png")"

echo "--- [03] J-03 Ctrl+Alt+Shift+R — honest nothing-to-pick-up guidance (status line; no banner)"
key ctrl+alt+shift+r; sleep 2
cap "$OUT/d25-03-recovery-guidance.png"; echo "B03 $(md5 "$OUT/d25-03-recovery-guidance.png")"
sleep 2
cap "$OUT/d25-03b-recovery-guidance-settle.png"; echo "B03b $(md5 "$OUT/d25-03b-recovery-guidance-settle.png")"

echo "--- [04] Escape settle — nothing traps; the task surface stands"
key Escape; sleep 2
cap "$OUT/d25-04-task-stands.png"; echo "B04 $(md5 "$OUT/d25-04-task-stands.png")"

echo "--- [05] palette: 'Resume this task' row + Return lands (guidance fires again)"
key ctrl+k; sleep 2
type_text "Resume this task"; sleep 2
cap "$OUT/d25-05-palette-recovery.png"; echo "B05 $(md5 "$OUT/d25-05-palette-recovery.png")"
key Return; sleep 2.5
cap "$OUT/d25-05b-palette-recovery-landed.png"; echo "B05b $(md5 "$OUT/d25-05b-palette-recovery-landed.png")"
key Escape; sleep 1.5

echo "--- [06] J-07 Ctrl+Alt+Shift+7 — the agents panel (heading + empty + honest wiring state)"
key ctrl+alt+shift+7; sleep 2.5
cap "$OUT/d25-06-agents-open.png"; echo "B06 $(md5 "$OUT/d25-06-agents-open.png")"
sleep 1.5
cap "$OUT/d25-07-agents-detail.png"; echo "B07 $(md5 "$OUT/d25-07-agents-detail.png")"

echo "--- [08] scoped Escape (rail) — the panel closes, task surface returns"
key Escape; sleep 2
cap "$OUT/d25-08-agents-closed.png"; echo "B08 $(md5 "$OUT/d25-08-agents-closed.png")"

echo "--- [09] palette: 'See who is working' row + Return lands"
key ctrl+k; sleep 2
type_text "See who is working"; sleep 2
cap "$OUT/d25-09-palette-agents.png"; echo "B09 $(md5 "$OUT/d25-09-palette-agents.png")"
key Return; sleep 2.5
cap "$OUT/d25-09b-palette-agents-landed.png"; echo "B09b $(md5 "$OUT/d25-09b-palette-agents-landed.png")"
key Escape; sleep 1.5

echo "--- [10] J-10 Ctrl+Alt+Shift+S — the save panel (heading + entry + honest not-wired state)"
key ctrl+alt+shift+s; sleep 2.5
cap "$OUT/d25-10-save-open.png"; echo "B10 $(md5 "$OUT/d25-10-save-open.png")"
sleep 1.5
cap "$OUT/d25-11-save-detail.png"; echo "B11 $(md5 "$OUT/d25-11-save-detail.png")"

echo "--- [12] scoped Escape — the save panel closes"
key Escape; sleep 2
cap "$OUT/d25-12-save-closed.png"; echo "B12 $(md5 "$OUT/d25-12-save-closed.png")"

echo "--- [13] palette: 'Save this task' row + Return lands"
key ctrl+k; sleep 2
type_text "Save this task"; sleep 2
cap "$OUT/d25-13-palette-save.png"; echo "B13 $(md5 "$OUT/d25-13-palette-save.png")"
key Return; sleep 2.5
cap "$OUT/d25-13b-palette-save-landed.png"; echo "B13b $(md5 "$OUT/d25-13b-palette-save-landed.png")"
key Escape; sleep 1.5

echo "--- [14] J-11 Ctrl+Alt+2 — the workflows list (honest empty state)"
key ctrl+alt+2; sleep 2.5
cap "$OUT/d25-14-workflows-list.png"; echo "B14 $(md5 "$OUT/d25-14-workflows-list.png")"

echo "--- [15] settle frame"
sleep 2
cap "$OUT/d25-15-settle.png"; echo "B15 $(md5 "$OUT/d25-15-settle.png")"

kill $APP_PID 2>/dev/null; kill $XPID 2>/dev/null; pkill -f "picom" 2>/dev/null
echo "D25 SCENE COMPLETE — evidence in $OUT"
