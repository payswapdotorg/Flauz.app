#!/bin/bash
# FV-L19 — DOMAIN-NEUTRAL (the J-01/J-14/J-09 composite, non-code; Linux
# desktop lane, FV-002 / Wave 7). A garden-research task: no code.
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - new task: Ctrl+N (ui.rs:5743 — the FV-L01 calibration)
#     - composer submit: Ctrl+Return (the D11r FW-5 calibration)
#     - model picker: Ctrl+Alt+Shift+M (ui.rs:5806 — the FV-L15
#       calibration; the /model slash command also opens it, ui.rs:9446)
#     - the plan/status surfaces: the task surface's own status family
#       (the composer status panel via /status, ui.rs:9407 — the FV-L02
#       calibration)
#     - Evidence rail: Ctrl+Alt+Shift+4 (ui.rs:5798 — the FV-L10
#       calibration)
#     - save affordance: the task-surface entry "Save as a reusable
#       workflow" (flauz_save_workflow.rs:103 — the FV-L11 calibration;
#       visible post-success-path)
#   Copy: the domain-neutral law — every visited surface answers in
#   domain-neutral language (no repo/branch/test vocabulary on these
#   surfaces — PRODUCT-UX-JOURNEYS §6, the catalog's PJ §6 reference).
#   The honest states: the model picker's empty (flauz_model_picker.rs:
#   124-137), the Evidence empty (flauz_shell/mod.rs:306), the save
#   affordance + its description (flauz_save_workflow.rs:103, :110).
#   The objective text "Research seasonal planting calendars for the
#   community garden" is the catalog's own non-code objective (the
#   shared web-lab j-domain-neutral objective — web/lab/journeys.mjs
#   :650).
#
# Moments (the catalog FV-L19 row):
#   01 entry baseline
#   02 new task: "Research seasonal planting calendars for the community
#      garden" (no code)
#   03 the objective echoed on the task surface
#   04 the model picker visit (the honest empty, domain-neutral)
#   05 the status surface visit (/status panel)
#   06 the artifacts/evidence honest states (Ctrl+Alt+Shift+3 /
#      Ctrl+Alt+Shift+4 rails)
#   07 the save affordance visible post-success-path
#   08 Escape settle — final frame
#
# PASS (Lead-adjudicated, VLM reads):
#   - the non-code objective echoed on the task surface;
#   - every visited surface answers in domain-neutral language (no
#     repo/branch/test vocabulary on these surfaces — PJ §6);
#   - the honest states verbatim.
#
# Usage: scripts/fv/fv-l19.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l19 "DOMAIN-NEUTRAL" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl19-01-entry

moment 02 "new task: the non-code objective typed (no code)"
key ctrl+n; sleep 2.5
type_ "Research seasonal planting calendars for the community garden"; sleep 1.5
cap fvl19-02-objective-typed

moment 03 "the objective echoed on the task surface (Ctrl+Return submit)"
key ctrl+Return; sleep 6
cap fvl19-03-task-surface
sleep 3
cap fvl19-03b-task-surface-settled

moment 04 "the model picker visit (the honest empty, domain-neutral)"
key ctrl+alt+shift+m; sleep 2.5
cap fvl19-04-model-picker
key Escape; sleep 1.5

moment 05 "the status surface visit (the /status panel)"
type_ "/status"; sleep 1.5
key Return; sleep 3
cap fvl19-05-status-panel
key Escape; sleep 2

moment 06 "the artifacts/evidence honest states (the rails)"
key ctrl+alt+shift+3; sleep 2.5
cap fvl19-06a-environments-honest
key Escape; sleep 1.5
key ctrl+alt+shift+4; sleep 2.5
cap fvl19-06b-evidence-honest
key Escape; sleep 1.5

moment 07 "the save affordance visible post-success-path (the task-surface entry)"
key Escape; sleep 1
sleep 2
cap fvl19-07-save-affordance-visible

moment 08 "Escape settle — final frame"
key Escape; sleep 2
cap fvl19-08-settle

say "DOMAIN-NEUTRAL LAW (PJ §6): every visited surface must answer in domain-neutral language — no repo/branch/test vocabulary on these surfaces; the VLM read checks each frame"

fv_end
