#!/bin/bash
# FV-L17 — J-16 Inspect resource conflicts (the conflicts panel; Linux
# desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - conflicts panel: Ctrl+Alt+Shift+L — KeyBinding::new(
#       &shortcut("alt-shift-l"), FlauzConflictsShortcut) at
#       crates/codex-app/src/ui.rs:5868 (the LEASE-001 chord; the
#       title-bar conflicts button rides the same surface, ui.rs:15591)
#     - scoped Escape: KeyBinding::new("escape", Escape,
#       Some("FlauzConflicts")) at ui.rs:5929
#   Copy (crates/codex-app/src/ui/flauz_conflicts.rs):
#     - PANEL_HEADING "Who is using what" :97; PANEL_DESCRIPTION
#       "Every resource's state — who holds it, who is waiting, and what
#       happens next" :99-100
#     - the view-model states (unit-pinned; the render's
#       ResourceStateView): FREE_STATE "Free" :102 +
#       FREE_STATE_BODY :104-105; HELD_LINE "{holder} is using {resource}
#       — {mode} until {until}" :108; QUEUE_LINE_YOURS_NEXT "{count}
#       waiting — yours is next" :111 ("2 tasks waiting — yours is next"
#       in the catalog's wording — the module's tests pin the exact
#       shapes at :1216/:1281); the escalation card
#       ESCALATION_HEADLINE :120-121 with the decision affordances
#       GRANT_TO_WAITER_LABEL :123 + GRANT_TO_WAITER_CONSEQUENCE :125-126
#       and KEEP_HOLDER_LABEL :128 + KEEP_HOLDER_CONSEQUENCE :130-131
#       (each stating its consequence)
#     - the honest empty + not-wired note: EMPTY_TITLE "Nothing is being
#       shared yet" :151; EMPTY_BODY :153-155; NOT_WIRED_NOTE "This
#       picture is the panel's own state right now — it connects to live
#       tasks once resource scheduling is wired into the run." :148-149
#       (the live lease state is future wiring — the catalog's named
#       not-wired note)
#     - ESCAPE_HINT "Escape closes this panel" :165; PALETTE_ROW_TITLE
#       "See who is using what" :168
#
# Moments (the catalog FV-L17 row):
#   01 entry baseline
#   02 anchor task
#   03 Ctrl+Alt+Shift+L — the conflicts panel (the heading + the honest
#      empty + the not-wired note)
#   04 settle/detail frame
#   05 scoped Escape — the panel closes
#   06 palette row "See who is using what" + Return lands the same panel
#   07 Escape settle — final frame
#
# PASS (Lead-adjudicated, VLM reads):
#   - the panel title + the honest empty + the not-wired note verbatim
#     (the live lease state is future wiring — named);
#   - the view-model states (Free / held-with-expiry / queue position /
#     the escalation card with consequences) are unit-pinned at this
#     base (the module's copy tests, flauz_conflicts.rs:1265-1362) —
#     the run notes name the bound (catalog gap X-3); the panel never
#     invents contention data;
#   - one Escape closes the panel.
#
# Usage: scripts/fv/fv-l17.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l17 "J-16" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl17-01-entry

moment 02 "anchor task"
key ctrl+n; sleep 2.5
type_ "FV-L17 anchor task"; sleep 1.5
key ctrl+Return; sleep 6
cap fvl17-02-task-surface

moment 03 "Ctrl+Alt+Shift+L — the conflicts panel (the honest empty + the not-wired note)"
key ctrl+alt+shift+l; sleep 2.5
cap fvl17-03-conflicts-panel

moment 04 "settle/detail frame"
sleep 1.5
cap fvl17-04-conflicts-detail

moment 05 "scoped Escape — the panel closes"
key Escape; sleep 2
cap fvl17-05-panel-closed

moment 06 "palette row 'See who is using what' + Return lands the same panel"
key ctrl+k; sleep 2.5
type_ "See who is using what"; sleep 2
cap fvl17-06-palette-row
key Return; sleep 2.5
cap fvl17-06b-palette-landed
key Escape; sleep 1.5

moment 07 "Escape settle — final frame"
key Escape; sleep 2
cap fvl17-07-settle

say "RUN NOTE (catalog gap X-3): the conflicts panel renders its honest view-models — the Free/held/queue/escalation states are unit-pinned copy (flauz_conflicts.rs module tests at this base); live lease wiring is future work; this scene verifies the honest panel state and never invents contention data"

fv_end
