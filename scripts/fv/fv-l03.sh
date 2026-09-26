#!/bin/bash
# FV-L03 — J-03 Recover/continue (the honest nothing-to-pick-up guidance;
# Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - recovery: Ctrl+Alt+Shift+R — KeyBinding::new(
#       &shortcut("alt-shift-r"), FlauzRecoveryShortcut) at
#       crates/codex-app/src/ui.rs:5815
#     - palette: Ctrl+K (ui.rs:5739)
#   Copy:
#     - NOTHING_TO_RECOVER_GUIDANCE "This task is up to date — there's
#       nothing to pick up right now." —
#       crates/codex-app/src/ui/flauz_recovery.rs:131-132, dispatched
#       through the command-status line (dispatch_command_status,
#       flauz_recovery.rs:424-426 — the [Dismiss] affordance rides the
#       status strip, ui.rs "Dismiss" labels)
#     - PALETTE_ROW_TITLE "Resume this task where it left off" —
#       flauz_recovery.rs:136 (description :138-139)
#     - the task-surface frame is banner-free when nothing is recoverable:
#       open_recovery_surface returns early with guidance only
#       (flauz_recovery.rs:415-427) — no banner is set (set_view None,
#       flauz_recovery.rs:383-392)
#
# Moments (the catalog FV-L03 row):
#   01 entry baseline
#   02 anchor task created (up to date — nothing to pick up)
#   03 Ctrl+Alt+Shift+R on the up-to-date task — the honest
#      nothing-to-pick-up guidance via the command-status line; NO banner
#   04 the task-surface frame is banner-free (frame-compare vs 02)
#   05 palette row "Resume this task where it left off" filters
#   06 Return lands the same guidance (frame-compare vs 03)
#
# PASS (Lead-adjudicated):
#   - "This task is up to date — there's nothing to pick up right now."
#     + [Dismiss] via the command-status line at 03/06;
#   - the task-surface frame at 04 is banner-free (md5-adjacent compare
#     with 02's settled surface);
#   - the palette row filters and lands the identical guidance.
#
# Usage: scripts/fv/fv-l03.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l03 "J-03" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl03-01-entry

moment 02 "anchor task created (an up-to-date task — nothing to pick up)"
key ctrl+n; sleep 2.5
type_ "FV-L03 up-to-date anchor task"; sleep 1.5
key ctrl+Return; sleep 6
cap fvl03-02-task-surface
sleep 3
cap fvl03-02b-task-surface-settled

moment 03 "Ctrl+Alt+Shift+R on the up-to-date task — the honest nothing-to-pick-up guidance (command-status line), NO banner"
key ctrl+alt+shift+r; sleep 2.5
cap fvl03-03-recovery-guidance

moment 04 "the task-surface frame is banner-free (compare vs the 02b settled surface)"
sleep 2
cap fvl03-04-banner-free

moment 05 "palette: 'Resume this task where it left off' filters"
key ctrl+k; sleep 2.5
type_ "Resume this task"; sleep 2
cap fvl03-05-palette-recovery-row

moment 06 "Return lands the same guidance (frame-compare vs 03)"
key Return; sleep 2.5
cap fvl03-06-palette-landed-same-guidance
key Escape; sleep 1.5

moment 07 "Escape settle — final frame"
key Escape; sleep 2
cap fvl03-07-settle

fv_end
