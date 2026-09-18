#!/usr/bin/env bash
# Scene D8c2 — WO-P2-005-FIX verification of /worktree on the FIXED binary.
# Calibrated 2026-09-17: windowfocus + click-ladder + proven composer point
# (802,789; composer zone y 749..829 at x 802, window 1278x818+163+91).
#
# The regression gate: after Enter #1 opens the fork-destination picker we
# WAIT 2.5s and capture d8c4-006-picker-still-open — PASS = picker still
# rendered (the fix); FAIL = bare command row (the defect: set_value ->
# InputEvent::Change -> composer_fork_picker_open=false for any != "/fork").
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
WID=$(WINID)
echo "WINID=$WID"

# 0. explicit focus + click ladder (proven input recipe)
XDO windowfocus --sync "$WID" 2>/dev/null || XDO windowfocus "$WID"
sleep 0.5
for Y in 849 829 809 789; do
  XDO mousemove 802 $Y click 1
  sleep 0.3
done

# 1. create + select a task (unauthenticated; the turn retries by design —
#    the selected-task precondition is what matters)
XDO type --delay 40 "worktree fixture fix"
sleep 0.8
CAPTURE d8c4-001-typed
XDO key ctrl+Return
sleep 12
CAPTURE d8c4-002-task-created

# 2. dismiss the retry banner, settle
XDO key Escape
sleep 2.5
CAPTURE d8c4-003-settled

# 3. type /worktree — the slash menu row shows
XDO mousemove 802 789 click 1
sleep 0.5
for i in $(seq 1 30); do XDO key BackSpace; done
sleep 0.5
XDO type --delay 60 "/worktree"
sleep 1.5
CAPTURE d8c4-004-slash-row

# 4. Enter #1: execute the slash command -> picker opens
XDO key ctrl+Return
sleep 1.2
CAPTURE d8c4-005-picker-open

# 5. SETTLE PROBE (the fix's regression gate): 2.5s past the event cycle.
sleep 2.5
CAPTURE d8c4-006-picker-still-open

# 6. Enter #2: on the still-open picker -> fork the task into a worktree
XDO key ctrl+Return
sleep 2.5
CAPTURE d8c4-007-fork-dispatched

# 7. aftermath
sleep 8
CAPTURE d8c4-008-aftermath
echo "scene complete"
