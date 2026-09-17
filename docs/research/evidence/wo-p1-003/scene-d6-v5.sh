#!/usr/bin/env bash
# Scene D6 v5 — WO-P1-003 full flow (pixel-calibrated):
# Edit project surface -> Make primary swap -> Done -> persistence + cwd follow.
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
# Open the Edit project surface via the row's gear affordance.
XDO mousemove 138 380; sleep 0.6
XDO mousemove 382 380 click 1; sleep 1.5
CAPTURE d6v5-001-edit-surface
# Make primary on the related (beta) row: star button.
XDO mousemove 1015 544 click 1; sleep 1.5
CAPTURE d6v5-002-primary-swapped
# Done.
XDO mousemove 1023 631 click 1; sleep 1.2
CAPTURE d6v5-003-edit-closed-path-follows-primary
# Reopen: the swapped state must persist (storage round-trip).
XDO mousemove 382 380 click 1; sleep 1.5
CAPTURE d6v5-004-edit-reopened-persisted
