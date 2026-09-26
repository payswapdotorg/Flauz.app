#!/bin/bash
# FV-L11 — J-10 Save as reusable workflow (the save panel's honest
# not-wired state; Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - save panel: Ctrl+Alt+Shift+S — KeyBinding::new(
#       &shortcut("alt-shift-s"), FlauzSaveWorkflowShortcut) at
#       crates/codex-app/src/ui.rs:5855
#     - scoped Escape: the save panel owns a scoped escape binding
#       (KeyBinding::new("escape", Escape, Some("FlauzSaveWorkflow")) at
#       ui.rs:5904)
#   Copy (crates/codex-app/src/ui/flauz_save_workflow.rs):
#     - the task-surface affordance: ENTRY_LABEL "Save as a reusable
#       workflow" :103 (PANEL_HEADING :108); ENTRY_TOOLTIP "Remember the
#       successful steps of this task so you can run them again
#       (Ctrl+Alt+Shift+S)" :105-106; PANEL_DESCRIPTION "Flauz can
#       remember the successful steps so you can run them again" :110
#     - the honest not-wired state: NOT_WIRED_TITLE "Saving isn't wired
#       to live tasks yet" :115; NOT_WIRED_BODY "When a task finishes
#       successfully, the steps that actually ran — exactly what
#       happened, nothing aspirational — can be saved here under a name
#       you choose, together with the task's inputs and the resources it
#       used." :117-119; NOT_WIRED_NEXT_STEP "Finish a task successfully,
#       then open this panel again — the run's real steps appear here
#       once the run wiring lands." :121-122
#     - ESCAPE_HINT "Escape closes this panel" :145
#     - PALETTE_ROW_TITLE "Save this task as a reusable workflow" :167
#
# Moments (the catalog FV-L11 row):
#   01 entry baseline
#   02 anchor task (the affordance rides the task surface)
#   03 the task-surface affordance "Save as a reusable workflow" (+ its
#      description) — visible in the frame; VLM read
#   04 Ctrl+Alt+Shift+S — the save panel (the honest not-wired state)
#   05 settle/detail frame (the not-wired body + next-step + escape hint
#      readable)
#   06 scoped Escape — the panel closes (the task surface stands)
#
# PASS (Lead-adjudicated, VLM reads):
#   - the affordance + its description ("Flauz can remember the
#     successful steps…") at 03;
#   - "Saving isn't wired to live tasks yet" + "the steps that actually
#     ran — exactly what happened, nothing aspirational" + the
#     next-step + "Escape closes this panel" verbatim at 04/05;
#   - one Escape closes the panel (06).
#
# Usage: scripts/fv/fv-l11.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l11 "J-10" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl11-01-entry

moment 02 "anchor task (the affordance rides the task surface)"
key ctrl+n; sleep 2.5
type_ "FV-L11 anchor task"; sleep 1.5
key ctrl+Return; sleep 6
cap fvl11-02-task-surface-affordance

moment 03 "the task-surface affordance visible (settle frame for the VLM read of the affordance + description)"
sleep 3
cap fvl11-03-affordance-settle

moment 04 "Ctrl+Alt+Shift+S — the save panel (the honest not-wired state)"
key ctrl+alt+shift+s; sleep 2.5
cap fvl11-04-save-panel-not-wired

moment 05 "settle/detail frame (the not-wired body + next-step + escape hint readable)"
sleep 1.5
cap fvl11-05-save-panel-detail

moment 06 "scoped Escape — the panel closes (the task surface stands)"
key Escape; sleep 2
cap fvl11-06-panel-closed

moment 07 "Escape settle — final frame"
key Escape; sleep 2
cap fvl11-07-settle

fv_end
