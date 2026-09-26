#!/bin/bash
# FV-L18 — J-17 Review activity needing the human (the unread flow + the
# Activity view; Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords (all registry rows, resolved through the keystroke observer —
#   ui.rs:12137-12161):
#     - mark unread: Ctrl+Shift+U — the "toggleThreadUnread" row
#       ("Mark chat unread", CmdOrCtrl+Shift+U) at ui.rs:3251-3256
#     - jump to next unread: Ctrl+Alt+A — the "nextUnreadChat" row
#       ("Next chat needing attention", CmdOrCtrl+Alt+A) at
#       ui.rs:3334-3340 (visit clears the dot — select_next_chat_needing_
#       attention, ui.rs:12317)
#     - clear all: Shift+Escape — the "clearAllUnread" row ("Clear all
#       unread indicators") at ui.rs:3341-3347
#     - the Activity view: Ctrl+Alt+U — the "toggleActivityView" row
#       ("Toggle Activity view", CmdOrCtrl+Alt+U) at ui.rs:3327-3333;
#       the Activity surface: Ctrl+Alt+4 (FlauzActivityShortcut,
#       ui.rs:5790)
#     - the Activity view rows are keyboard-navigable: the scoped
#       bindings ActivityViewSelectPrevious/Next/Confirm/Escape
#       (up/down/enter/escape) at ui.rs:5882-5885
#   Copy:
#     - "Chat marked unread" — the toggle's status message (pinned by
#       ui.rs:54931)
#     - "Cleared unread indicators for N chats" — the clear-all status
#       (pinned by ui.rs:54885; "Cleared unread indicators for 1 chat"
#       with one flagged chat)
#     - the unread dot: UNREAD_ATTENTION_DOT_TOOLTIP "Unread activity"
#       (ui.rs:1895); the row-count banner "{row_count} chats need
#       attention" (ui.rs:1901)
#     - the Activity surface: heading "Activity" + description
#       "Everything that needs you, in one place" + honest empty
#       "Nothing needs your attention" (flauz_shell/mod.rs:99, 124, 154)
#     - the needs-you extension rows: the takeover panel's escalation
#       rows (flauz_takeover.rs — the FV-L09 calibration) show their row
#       shape when seeded (bounded: the quiet state renders without
#       live escalations — X-3)
#
# Moments (the catalog FV-L18 row):
#   01 entry baseline (seeded donor state — chats present)
#   02 select the first seeded chat
#   03 Ctrl+Shift+U — mark unread ("Chat marked unread" + the dot + the
#      label on the row)
#   04 Ctrl+Alt+A — the jump lands selection on the flagged chat with
#      the dot cleared (visit clears)
#   05 Ctrl+Shift+U (re-flag) then Shift+Escape — clear-all honest
#      count ("Cleared unread indicators for 1 chat")
#   06 Ctrl+Alt+U — the Activity view (rows keyboard-navigable)
#   07 the Activity view keyboard probes (up/down/enter/escape)
#   08 Ctrl+Alt+4 — the Activity surface (the workspace surface)
#   09 Escape settle — final frame
#
# PASS (Lead-adjudicated, VLM reads):
#   - "Chat marked unread" → the dot on the row → the jump lands
#     selection on the flagged chat with the dot cleared → "Cleared
#     unread indicators for 1 chat";
#   - the Activity view renders with keyboard navigation
#     (up/down/enter/escape — the ActivityView rows);
#   - a needs-you-kind row shape when seeded (bounded: the quiet state
#     otherwise — named).
#
# Usage: scripts/fv/fv-l18.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime
fv_need_donor

fv_begin fv-l18 "J-17" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline (seeded donor state)"
cap fvl18-01-entry

moment 02 "select the first seeded chat"
key Tab; key Tab; key Tab; sleep 1
key Down; sleep 1
key Return; sleep 2.5
cap fvl18-02-chat-selected

moment 03 "Ctrl+Shift+U — mark unread ('Chat marked unread' + the dot + the label)"
key ctrl+shift+u; sleep 2.5
cap fvl18-03-marked-unread

moment 04 "Ctrl+Alt+A — the jump lands selection on the flagged chat with the dot cleared (visit clears)"
key ctrl+alt+a; sleep 2.5
cap fvl18-04-jump-cleared

moment 05 "re-flag then Shift+Escape — clear-all honest count ('Cleared unread indicators for 1 chat')"
key ctrl+shift+u; sleep 2
cap fvl18-05a-reflagged
key shift+Escape; sleep 2.5
cap fvl18-05b-clear-all

moment 06 "Ctrl+Alt+U — the Activity view"
key ctrl+alt+u; sleep 2.5
cap fvl18-06-activity-view

moment 07 "the Activity view keyboard probes (up/down/enter/escape)"
key Up; sleep 1
cap fvl18-07a-activity-up
key Down; sleep 1
cap fvl18-07b-activity-down
key Return; sleep 1.5
cap fvl18-07c-activity-enter
key Escape; sleep 1.5
cap fvl18-07d-activity-escape

moment 08 "Ctrl+Alt+4 — the Activity surface (the workspace surface)"
key ctrl+alt+4; sleep 2.5
cap fvl18-08-activity-surface

moment 09 "Escape settle — final frame"
key Escape; sleep 2
cap fvl18-09-settle

say "RUN NOTE: the needs-you extension rows render their row shape when seeded; with no live escalations the honest quiet state renders (catalog gap X-3 — named; never invented)"

fv_end
