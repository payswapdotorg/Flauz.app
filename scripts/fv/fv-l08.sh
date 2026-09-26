#!/bin/bash
# FV-L08 — J-07 Parallelize work (the agents view; Linux desktop lane,
# FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - agents view: Ctrl+Alt+Shift+7 — KeyBinding::new(
#       &shortcut("alt-shift-7"), FlauzAgentsViewShortcut) at
#       crates/codex-app/src/ui.rs:5853 (shifted-symbol companion alt-&
#       at ui.rs:5854)
#     - palette: Ctrl+K (ui.rs:5739)
#     - scoped Escape: KeyBinding::new("escape", Escape,
#       Some("FlauzAgentsView"))… the agents view rides the rail family
#       escape (the d26 members/conflicts precedent; the scoped binding
#       set lives at ui.rs:5882-5935)
#   Copy (crates/codex-app/src/ui/flauz_agents_view.rs):
#     - PANEL_HEADING "Agents on this task" :66
#     - EMPTY_TITLE "No extra agents on this task yet" :75
#     - EMPTY_BODY "Extra agents let one task work in parallel: one agent
#       researches while another analyzes, and an independent reviewer
#       checks the results. Everything each one produces stays separate
#       and attributed to its maker." :77-79 (the delegation body)
#     - WIRED_TITLE "Live agent progress is on its way" :86; WIRED_BODY
#       "…No agent is invented here before that wiring lands." :87-90
#       (the not-wired line the catalog names)
#     - PALETTE_ROW_TITLE "See who is working on this task" :119
#       (description :121-122)
#     - ESCAPE_HINT "Escape closes this panel" :117
#
# Moments (the catalog FV-L08 row):
#   01 entry baseline
#   02 anchor task (the view rides the task context)
#   03 Ctrl+Alt+Shift+7 — the agents view (heading + honest empty +
#      parallel-work body + the not-wired line)
#   04 settle/detail frame
#   05 scoped Escape — the view closes
#   06 palette row "See who is working on this task" + Return lands the
#      same panel
#   07 the chord frame == the palette frame (pixel-identical:
#      frame-compare 06b vs 03)
#
# PASS (Lead-adjudicated, VLM reads + md5 compare):
#   - "Agents on this task" + "No extra agents on this task yet" + the
#     delegation body + "Live agent progress is on its way … No agent is
#     invented here…" verbatim;
#   - the chord frame and the palette frame are pixel-identical;
#   - one Escape returns to the task surface.
#
# Usage: scripts/fv/fv-l08.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l08 "J-07" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl08-01-entry

moment 02 "anchor task"
key ctrl+n; sleep 2.5
type_ "FV-L08 anchor task"; sleep 1.5
key ctrl+Return; sleep 6
cap fvl08-02-task-surface

moment 03 "Ctrl+Alt+Shift+7 — the agents view (honest empty + delegation body + the not-wired line)"
key ctrl+alt+shift+7; sleep 2.5
cap fvl08-03-agents-view
CHORD_FRAME="$(frame_md5 fvl08-03-agents-view.png)"
say "chord frame md5: $CHORD_FRAME"
sleep 1.5
cap fvl08-04-agents-detail

moment 05 "scoped Escape — the view closes; the task surface stands"
key Escape; sleep 2
cap fvl08-05-agents-closed

moment 06 "palette row 'See who is working on this task' + Return lands the same panel"
key ctrl+k; sleep 2.5
type_ "See who is working on this task"; sleep 2
cap fvl08-06-palette-row
key Return; sleep 2.5
cap fvl08-06b-palette-landed
PALETTE_FRAME="$(frame_md5 fvl08-06b-palette-landed.png)"
say "palette-landed frame md5: $PALETTE_FRAME"

moment 07 "frame-compare: the chord frame vs the palette frame (expect pixel-identical)"
if [ "$CHORD_FRAME" = "$PALETTE_FRAME" ] && [ -n "$CHORD_FRAME" ]; then
  say "FRAME-COMPARE: identical (md5 match)"
else
  say "FRAME-COMPARE: md5 differs ($CHORD_FRAME vs $PALETTE_FRAME) — the Lead adjudicates pixel-identity (settle jitter is the known d-series cause)"
fi
key Escape; sleep 1.5

moment 08 "Escape settle — final frame"
key Escape; sleep 2
cap fvl08-08-settle

fv_end
