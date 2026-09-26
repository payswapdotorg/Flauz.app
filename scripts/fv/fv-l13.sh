#!/bin/bash
# FV-L13 — J-12 Reuse/improve a Procedure (the honest-state slice; Linux
# desktop lane, FV-002 / Wave 7). NAMED BOUND: the full
# run/deviation/improve loop is N/A — not wired on any client (catalog
# §4 J-12 row; cross-lane gap X-2).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - save panel: Ctrl+Alt+Shift+S (ui.rs:5855 — the FV-L11
#       calibration)
#     - Reusable workflows surface: Ctrl+Alt+2 (ui.rs:5788 — the FV-L12
#       calibration)
#   Copy (crates/codex-app/src/ui/flauz_save_workflow.rs):
#     - the deviation promise: DEVIATION_HINT "If a run takes different
#       steps than the workflow, the run will say so — no silent drift."
#       :164-165 (the save panel's deviation promise the catalog names;
#       the full library is F10 — :162-164 comment)
#     - the list's empty state: WORKFLOWS_EMPTY_TITLE "No reusable
#       workflows yet" :149; WORKFLOWS_EMPTY_BODY "A reusable workflow
#       remembers the successful steps of a task — exactly what ran — so
#       you can run them again on a new task any time." :151-152;
#       WORKFLOWS_EMPTY_NEXT_STEP :154-155
#     - RUN_AGAIN_LABEL "Run again" :157 (the run path — present in the
#       copy registry; live run wiring is future, X-2)
#     - the not-wired copy names that deviations/improvement arrive with
#       the wiring: NOT_WIRED_TITLE :115 + NOT_WIRED_NEXT_STEP "…the
#       run's real steps appear here once the run wiring lands." :121-122
#
# Moments (the catalog FV-L13 row):
#   01 entry baseline
#   02 anchor task
#   03 Ctrl+Alt+Shift+S — the save panel's deviation promise (the
#      not-wired state naming that deviations/improvement arrive with
#      the wiring)
#   04 Escape — the panel closes
#   05 Ctrl+Alt+2 — the list's empty state (the workflows surface)
#   06 Escape settle — final frame
#
# PASS (Lead-adjudicated, VLM reads):
#   - the not-wired copy naming that deviations/improvement arrive with
#     the wiring (03: the not-wired body + the deviation hint);
#   - the list's empty state verbatim (05);
#   - the run notes name the N/A reason (cross-lane gap X-2: the full
#     run/deviation/improve loop is not wired on any client — the
#     honest not-wired state is the current truth; no scene invents
#     deviation data).
#
# Usage: scripts/fv/fv-l13.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l13 "J-12" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl13-01-entry

moment 02 "anchor task"
key ctrl+n; sleep 2.5
type_ "FV-L13 anchor task"; sleep 1.5
key ctrl+Return; sleep 6
cap fvl13-02-task-surface

moment 03 "Ctrl+Alt+Shift+S — the save panel (the deviation promise inside the not-wired state)"
key ctrl+alt+shift+s; sleep 2.5
cap fvl13-03-save-panel-deviation-promise
sleep 1.5
cap fvl13-03b-save-panel-detail

moment 04 "Escape — the panel closes"
key Escape; sleep 2
cap fvl13-04-panel-closed

moment 05 "Ctrl+Alt+2 — the workflows list's empty state"
key ctrl+alt+2; sleep 2.5
cap fvl13-05-workflows-empty

moment 06 "Escape settle — final frame"
key Escape; sleep 2
cap fvl13-06-settle

say "RUN NOTE (catalog §4 J-12 / gap X-2): the full run/deviation/improve loop is N/A — not wired on any client at this base; this scene verifies the honest not-wired states (the deviation promise + the empty list) exactly as the catalog's honest-state slice defines"

fv_end
