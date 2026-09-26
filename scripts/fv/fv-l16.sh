#!/bin/bash
# FV-L16 — J-15 Switch execution environment (the worktree fork flow;
# Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords/anchors:
#     - the composer's work-location affordance: the "Work locally"
#       button (ui.rs:26327-26338, tooltip "Work on {current_branch}"
#       :26329) opens the dropdown with the "Continue in" header
#       (:26334) — "Work locally" checked + disabled (:26337-26339) and
#       "New worktree" + "Create a copy of your local project to work in
#       parallel" (:26347, :26355) whose click dispatches
#       Action::ForkSelectedTaskIntoWorktree (:26361-26367)
#     - the footer "Continue in" family: the composer's fork picker
#       renders "Continue in new chat" / "Continue in new worktree" +
#       "Create a copy of your local project to work in parallel"
#       (render_fork_slash_picker, ui.rs:24257-24298 — the "/fork"
#       slash picker + the footer affordance the catalog names)
#     - environments rail honest state: Ctrl+Alt+Shift+3 (ui.rs:5793) →
#       "No environments attached" + the body (flauz_shell/mod.rs:305,
#       :324 — the FV-L06 calibration)
#   The seeded git chat: donor state (wo-p2-008 pattern) — a chat bound
#   to a local git workspace, so the fork flow has a real repository to
#   fork. The post-fork task surface continues in the fork (the task
#   continues); the fork-state transfer is recorded in the run notes
#   (bounded: the 20 MiB transfer contract is unit-anchored).
#
# Moments (the catalog FV-L16 row):
#   01 entry baseline (seeded donor state)
#   02 select the seeded git chat
#   03 the composer "Work locally" affordance (settle frame — VLM read
#      of the footer)
#   04 open the dropdown — the picker with "Work locally" checked +
#      "New worktree" + "Create a copy of your local project to work in
#      parallel" (keyboard: Tab to the affordance + Enter/Down opens)
#   05 select "New worktree" — the fork explanation stands; the
#      worktree is created (the task continues in the fork)
#   06 the post-fork task surface (settle frame)
#   07 Ctrl+Alt+Shift+3 — the environments rail honest state
#   08 Escape settle — final frame
#
# PASS (Lead-adjudicated, VLM reads):
#   - the picker with "Work locally" checked + "New worktree" + "Create
#     a copy of your local project to work in parallel" (04);
#   - the post-fork task surface (05/06);
#   - the run notes record the fork state transfer (bounded: the 20 MiB
#     transfer contract is unit-anchored — the catalog's named bound);
#   - the environments rail honest state (07).
#
# Usage: scripts/fv/fv-l16.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime
fv_need_donor

fv_begin fv-l16 "J-15" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline (seeded donor state)"
cap fvl16-01-entry

moment 02 "select the seeded git chat"
key Tab; key Tab; key Tab; sleep 1
key Down; sleep 1
key Return; sleep 2.5
cap fvl16-02-seeded-git-chat

moment 03 "the composer 'Work locally' affordance (settle frame — the footer)"
sleep 2
cap fvl16-03-work-locally-footer

moment 04 "open the work-location dropdown (keyboard: Tab reaches the affordance; Enter opens the menu)"
key Tab; key Tab; sleep 1
key Return; sleep 2
cap fvl16-04-work-location-picker
say "the picker must show 'Continue in' + 'Work locally' (checked) + 'New worktree' + 'Create a copy of your local project to work in parallel' (ui.rs:26333-26356)"

moment 05 "select 'New worktree' (arrow-down + Enter) — the fork explanation stands; the worktree is created"
key Down; sleep 1
key Return; sleep 8
cap fvl16-05-worktree-forked
sleep 3
cap fvl16-05b-fork-settled

moment 06 "the post-fork task surface (settle frame)"
sleep 2
cap fvl16-06-post-fork-task-surface

moment 07 "Ctrl+Alt+Shift+3 — the environments rail honest state"
key ctrl+alt+shift+3; sleep 2.5
cap fvl16-07-environments-honest
key Escape; sleep 1.5

moment 08 "Escape settle — final frame"
key Escape; sleep 2
cap fvl16-08-settle

say "RUN NOTE: the fork state transfer is recorded in the frames 05-06 (the task continues in the fork); the 20 MiB transfer contract is unit-anchored at this base (the catalog's named bound — no live size probe is authored; the Lead reads the fork evidence from the frames + app.log)"

fv_end
