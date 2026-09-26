#!/bin/bash
# FV-L12 — J-11 Discover a learned Procedure (the Reusable workflows
# surface; Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - Reusable workflows surface: Ctrl+Alt+2 — KeyBinding::new(
#       &shortcut("alt-2"), FlauzReusableWorkflowsShortcut) at
#       crates/codex-app/src/ui.rs:5788
#     - palette: Ctrl+K (ui.rs:5739)
#   Copy (crates/codex-app/src/ui/flauz_shell/mod.rs):
#     - nav label "Reusable workflows" :97; description "Successful
#       steps, saved so you can run them again" :122
#     - honest empty: "No reusable workflows yet" :152; empty body
#       "Flauz can remember the successful steps of a task so you can
#       run them again. Saved workflows appear here with their history,
#       and are suggested on similar future tasks." :166-169
#     - next step: "Finish a task successfully, then choose 'Save as a
#       reusable workflow' — it will appear here and be suggested on
#       similar tasks." :191-193 (the what-to-do-next)
#     - palette row: palette_title "Reusable workflows" :132;
#       palette_description "Browse and run the workflows you saved" :143
#     - the sidebar's Workspace section highlights the surface (the
#       WorkspaceNavSurface highlight state — the surface's id
#       "reusable-workflows" :219)
#
# Moments (the catalog FV-L12 row):
#   01 entry baseline
#   02 Ctrl+Alt+2 — the Reusable workflows surface (heading + honest
#      empty + body + what-to-do-next)
#   03 settle/detail frame (the sidebar's Workspace section highlight
#      visible)
#   04 palette row "Reusable workflows" filters
#   05 Escape settle — final frame
#
# PASS (Lead-adjudicated, VLM reads):
#   - "Reusable workflows" + "Successful steps, saved so you can run
#     them again" + "No reusable workflows yet" + the body + the
#     what-to-do-next verbatim;
#   - the sidebar's Workspace section highlights the surface (02/03).
#
# Usage: scripts/fv/fv-l12.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l12 "J-11" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl12-01-entry

moment 02 "Ctrl+Alt+2 — the Reusable workflows surface (the honest empty)"
key ctrl+alt+2; sleep 2.5
cap fvl12-02-workflows-surface

moment 03 "settle/detail frame (the sidebar's Workspace section highlight visible)"
sleep 1.5
cap fvl12-03-sidebar-highlight

moment 04 "palette row 'Reusable workflows' filters"
key ctrl+k; sleep 2.5
type_ "Reusable workflows"; sleep 2
cap fvl12-04-palette-row
key Escape; sleep 1.5

moment 05 "Escape settle — final frame (the surface stands)"
key Escape; sleep 2
cap fvl12-05-settle

fv_end
