#!/bin/bash
# FV-L09 — J-08 Human takeover / handoff (the needs-you panel; Linux
# desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - needs-you panel: Ctrl+Alt+Shift+Y — KeyBinding::new(
#       &shortcut("alt-shift-y"), FlauzTakeoverShortcut) at
#       crates/codex-app/src/ui.rs:5874 (the title-bar needs-you button
#       rides the same surface, ui.rs:15596)
#     - scoped Escape: KeyBinding::new("escape", Escape,
#       Some("FlauzTakeover")) at ui.rs:5935 — the 017 restore contract
#   Copy (crates/codex-app/src/ui/flauz_takeover.rs):
#     - PANEL_HEADING "What needs you" :117; PANEL_DESCRIPTION :119-120
#     - the honest quiet state: EMPTY_TITLE "Nothing needs you right
#       now" :178; EMPTY_BODY "When a step needs your approval, a
#       conflict needs your call, or you want to take a step over
#       yourself, it lands here — with what happens on each side."
#       :180-181 (the what-lands-here copy)
#     - the approval card's BOTH consequences: APPROVAL_HEADLINE
#       "Needs you: {need} — {approve_consequence}" :123; APPROVE_SIDE
#       "If you approve: {approve_consequence}" :127; DECLINE_SIDE "If
#       you decline: {deny_consequence}" :132 — the consequence pair
#       (unit-pinned copy; live approval data is future wiring — catalog
#       gap X-3, named in the run notes)
#     - the takeover affordance: TAKEOVER_LABEL "Take over this step"
#       :140 + TAKEOVER_PRESERVED_NOTE "Your work joins the task — the
#       agent's work so far is kept exactly as it was" :143-144
#     - the handback copy: HANDBACK_PATH_NOTE "Hand back when you're
#       done — the agent picks up from where you left it" :147-148
#     - the cancellation affordances with downstream truth:
#       CANCEL_STEP_LABEL :158, CANCEL_RUN_LABEL :160, CANCEL_CONSEQU-
#       ENCE "What gets cancelled with it: {downstream}" :163,
#       DEPENDENT_LINE :166
#     - NOT_WIRED_NOTE :184-185; ESCAPE_HINT "Escape closes this panel"
#       :196; PALETTE_ROW_TITLE "See what needs you" :198
#
# Moments (the catalog FV-L09 row):
#   01 entry baseline
#   02 anchor task
#   03 Ctrl+Alt+Shift+Y — the needs-you panel (the honest quiet state +
#      the what-lands-here copy)
#   04 settle/detail frame (the not-wired note readable)
#   05 scoped Escape — the panel closes (one Escape, the 017 restore
#      contract)
#   06 the title-bar needs-you entry (visible in chrome; VLM read)
#
# PASS (Lead-adjudicated, VLM reads):
#   - "Nothing needs you right now" + the what-lands-here copy verbatim;
#   - the approval card's consequence pair + "Take over this step" + the
#     state-preserved note + "Hand back when you're done" verbatim
#     (rendered when the panel carries view-model content; with no live
#     approvals the quiet state renders and the consequence-pair copy is
#     unit-pinned at this base — the X-3 bound is named in the run
#     notes; never invented);
#   - one Escape closes the panel (05).
#
# Usage: scripts/fv/fv-l09.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l09 "J-08" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl09-01-entry

moment 02 "anchor task"
key ctrl+n; sleep 2.5
type_ "FV-L09 anchor task"; sleep 1.5
key ctrl+Return; sleep 6
cap fvl09-02-task-surface

moment 03 "Ctrl+Alt+Shift+Y — the needs-you panel (the honest quiet state + what-lands-here)"
key ctrl+alt+shift+y; sleep 2.5
cap fvl09-03-needs-you-panel
sleep 1.5
cap fvl09-04-needs-you-detail

moment 05 "scoped Escape — the panel closes (one Escape, the 017 restore contract)"
key Escape; sleep 2
cap fvl09-05-panel-closed

moment 06 "the title-bar needs-you entry (visible in chrome — the WO-P2-018 layer-1 entry)"
key ctrl+alt+shift+y; sleep 2.5
cap fvl09-06-titlebar-entry-visible
key Escape; sleep 1.5

say "RUN NOTE (catalog gap X-3): the needs-you panel renders its honest view-model — live approval/takeover data is future wiring; the approval-card consequence pair, the takeover/handback copy, and the cancellation affordances are unit-pinned at this base (flauz_takeover.rs copy constants + the module's tests); this scene never invents approval data"

fv_end
