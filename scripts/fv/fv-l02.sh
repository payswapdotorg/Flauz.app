#!/bin/bash
# FV-L02 — J-02 Understand what the agent knows (Linux desktop lane,
# FV-002 / Wave 7). The Context-section honest state + the /status panel.
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - task-rail Context: Ctrl+Alt+Shift+1 — KeyBinding::new(
#       &shortcut("alt-shift-1"), FlauzTaskContextShortcut) at
#       crates/codex-app/src/ui.rs:5791
#     - composer submit: Ctrl+Return (the D11r FW-5 calibration)
#   Copy:
#     - Context rail honest state "The context view is on its way" —
#       crates/codex-app/src/ui/flauz_shell/mod.rs:303 (empty_title),
#       body "The context view will show what the agent currently knows —
#       the sources it is drawing from and why each one is included — and
#       let you pin or remove items where policy allows." mod.rs:314-318
#     - the /status slash command opens the composer status panel —
#       ui.rs:9407-9410 (command == "/status" → open_composer_status);
#       the panel's rows: "Session:" (ui.rs:24490) and "Context:" with
#       "{remaining}% left ({} used / {total})" (ui.rs:24511-24513,
#       24526) — the Session row + live context remaining % the catalog
#       names. The seeded chat carries a timeline so the Context row has
#       a real usage value (donor state, the wo-p2-008 pattern).
#     - Escape closes the status panel (close_composer_status,
#       ui.rs:9727-9734) and the rail section (the shell's Escape,
#       ui.rs:5890 FlauzWorkspaceSurface / 5891 FlauzTaskRail scopes).
#
# Moments (the catalog FV-L02 row):
#   01 entry baseline (seeded donor state — chats present)
#   02 select the seeded chat (keyboard: click-free selection via
#      Ctrl+Alt+A's attention jump is not guaranteed on a read chat; the
#      scene uses Down/Tab focus walk + Enter — the honest bounded path)
#   03 Ctrl+Alt+Shift+1 — the task-rail Context section (honest
#      "on its way" state)
#   04 Escape — the section closes (frame returns to the task surface)
#   05 type "/status" + Enter on the seeded chat — the status panel
#   06 settle frame (Session row + context-usage row readable)
#   07 Escape — the status panel closes cleanly
#
# PASS (Lead-adjudicated, VLM reads):
#   - "The context view is on its way" + its body verbatim at 03;
#   - the status panel's Session row and the context-usage row at 05/06;
#   - Escape closes both cleanly (07 returns to the task surface).
#
# Usage: scripts/fv/fv-l02.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime
fv_need_donor

fv_begin fv-l02 "J-02" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline (seeded donor state — chats present)"
cap fvl02-01-entry

moment 02 "select the seeded chat (keyboard focus walk: Tab into the chat list, Enter selects)"
key Tab; key Tab; key Tab; sleep 1
key Down; sleep 1
key Return; sleep 2.5
cap fvl02-02-seeded-chat-selected

moment 03 "Ctrl+Alt+Shift+1 — the task-rail Context section (the honest 'on its way' state)"
key ctrl+alt+shift+1; sleep 2.5
cap fvl02-03-context-section
sleep 1.5
cap fvl02-03b-context-detail

moment 04 "Escape — the Context section closes (frame returns to the task surface)"
key Escape; sleep 2
cap fvl02-04-context-closed

moment 05 "type /status + Enter on the seeded chat — the composer status panel"
type_ "/status"; sleep 1.5
cap fvl02-05-status-typed
key Return; sleep 3
cap fvl02-05b-status-panel

moment 06 "settle frame (the Session row + the context-usage row readable)"
sleep 2
cap fvl02-06-status-settled

moment 07 "Escape — the status panel closes cleanly (frame returns to the task surface)"
key Escape; sleep 2
cap fvl02-07-status-closed

fv_end
