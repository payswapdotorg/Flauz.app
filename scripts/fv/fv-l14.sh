#!/bin/bash
# FV-L14 — J-13 Collaborate on one task (the members panel + the sharing
# surface; Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - members panel: Ctrl+Alt+Shift+U — KeyBinding::new(
#       &shortcut("alt-shift-u"), FlauzMembersShortcut) at
#       crates/codex-app/src/ui.rs:5862 (the d26-verified chord)
#     - palette: Ctrl+K (ui.rs:5739)
#     - scoped escapes: KeyBinding::new("escape", Escape,
#       Some("FlauzMembers")) at ui.rs:5923; the sharing surface closes
#       through the palette-landed panel's Escape (the d26 B12b/B14
#       moments)
#   Copy (crates/codex-app/src/ui/flauz_members.rs):
#     - PANEL_HEADING "Who is on this workspace" :109; SOLO_TITLE "Just
#       you" :114; the solo body (what inviting adds) — the d26 VLM read
#       recorded it verbatim ("Right now this workspace is just you.
#       Inviting someone lets them see the same projects and tasks, work
#       on them with you, and show up here with what they're doing.")
#     - INVITE_NOT_WIRED_TITLE "Inviting isn't connected yet" :120 (the
#       d26 VLM body: "Invitations arrive when Flauz connects workspaces
#       across devices — nothing is sent today, and no one else can see
#       your work.")
#     - ROLES_NOT_WIRED_TITLE "Changing roles isn't connected yet" :127
#     - the sharing surface: FILES_HEADING "Files" :154 +
#       FILES_ISOLATED_LINE "Isolated copy — your files stay separate"
#       :156; NOTES_HEADING "Remembered notes" :163 +
#       NOTES_ONLY_ME_LINE "Only me — remembered notes stay visible only
#       to you" :165; WORK_PRODUCTS_HEADING "Work products" :170;
#       CHANGE_SHARING_LABEL "Change sharing" :178
#     - palette row "Change a task's sharing" (the d26 B12 palette row —
#       filtered with "Change a task" in the d26 drive :199)
#
# Moments (the catalog FV-L14 row):
#   01 entry baseline
#   02 Ctrl+Alt+Shift+U — the members panel ("Just you" + both
#      not-wired states)
#   03 settle/detail frame
#   04 scoped Escape — the members panel closes
#   05 palette "Change a task's sharing" + Return lands the sharing
#      surface (Files/Remembered notes/Work products + the honest sync
#      note)
#   06 settle/detail frame
#   07 Escape — both panels close on one Escape each (final settle)
#
# PASS (Lead-adjudicated, VLM reads):
#   - "Who is on this workspace" + "Just you" + "Inviting isn't
#     connected yet … nothing is sent today" + "Changing roles isn't
#     connected yet" verbatim (02/03);
#   - the sharing surface's three sections + "Change sharing" (05/06);
#   - both panels close on one Escape (04/07).
#
# Usage: scripts/fv/fv-l14.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l14 "J-13" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl14-01-entry

moment 02 "Ctrl+Alt+Shift+U — the members panel ('Just you' + both not-wired states)"
key ctrl+alt+shift+u; sleep 2.5
cap fvl14-02-members-panel

moment 03 "settle/detail frame"
sleep 1.5
cap fvl14-03-members-detail

moment 04 "scoped Escape — the members panel closes"
key Escape; sleep 2
cap fvl14-04-members-closed

moment 05 "palette 'Change a task's sharing' + Return lands the sharing surface"
key ctrl+k; sleep 2.5
type_ "Change a task"; sleep 2
cap fvl14-05-palette-sharing
key Return; sleep 2.5
cap fvl14-05b-sharing-surface

moment 06 "settle/detail frame (the three sections + the honest sync note readable)"
sleep 1.5
cap fvl14-06-sharing-detail

moment 07 "Escape — the sharing surface closes; final settle"
key Escape; sleep 2
cap fvl14-07-settle

fv_end
