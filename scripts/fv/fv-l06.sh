#!/bin/bash
# FV-L06 — J-05/J-06 guards (the entry-surface honest statuses; Linux
# desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - terminal: Ctrl+` — KeyBinding::new("ctrl-`",
#       ToggleTerminalShortcut) at crates/codex-app/src/ui.rs:5777
#     - browser: Ctrl+T — the registry row "openBrowserTab" (OpenBrowserTab
#       shortcut "Ctrl+T" at ui.rs:4215) resolved through the keystroke
#       observer (ui.rs:12137-12161)
#     - environments rail: Ctrl+Alt+Shift+3 — KeyBinding::new(
#       &shortcut("alt-shift-3"), FlauzTaskEnvironmentsShortcut) at
#       ui.rs:5793-5797
#   Copy (the guarded-chord honest statuses — bottom-banner statuses,
#   bounded + dismissible):
#     - "Select a task before opening a terminal." —
#       crates/codex-core/src/lib.rs (the reducer's guard; surfaced via
#       state.status_message; pinned by ui.rs:53123)
#     - "Open a chat before opening the Browser." —
#       crates/codex-core/src/lib.rs:9360/15168/15196 (pinned by
#       ui.rs:53130)
#     - the environments rail honest empty: "No environments attached" +
#       "Environments are where work actually happens — browser sites,
#       terminals, …" — crates/codex-app/src/ui/flauz_shell/mod.rs:305
#       and :324 (empty_title/empty_body of TaskRailSection::Environments)
#   The scene runs from the ENTRY surface (no chat selected) so every
#   guarded chord fires its guard — "zero silent frames" is the law
#   (every guarded chord changes the frame).
#
# Moments (the catalog FV-L06 row):
#   01 entry baseline (no chat selected)
#   02 Ctrl+` — honest terminal status ("Select a task before opening a
#      terminal.")
#   03 status dismisses (Escape) — frame returns
#   04 Ctrl+T — honest browser status ("Open a chat before opening the
#      Browser.")
#   05 status dismisses (Escape) — frame returns
#   06 Ctrl+Alt+Shift+3 — the environments rail's honest empty state
#      ("No environments attached" + the body)
#   07 Escape — the rail section closes
#
# PASS (Lead-adjudicated, VLM reads):
#   - the two guarded-chord statuses verbatim as bottom-banner statuses
#     (bounded, dismissible);
#   - "No environments attached" + "Environments are where work actually
#     happens…" verbatim;
#   - zero silent frames — every guarded chord changes the frame (md5
#     deltas between the pre/post frames).
#
# Usage: scripts/fv/fv-l06.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l06 "J-05/J-06" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline (no chat selected — the entry surface)"
key Escape; sleep 1
cap fvl06-01-entry
BASE_MD5="$(frame_md5 fvl06-01-entry.png)"

moment 02 "Ctrl+\` — honest terminal status ('Select a task before opening a terminal.')"
key ctrl+grave; sleep 2.5
cap fvl06-02-terminal-status
T1="$(frame_md5 fvl06-02-terminal-status.png)"
say "frame delta vs baseline: $([ "$BASE_MD5" = "$T1" ] && echo "SILENT — FAIL candidate" || echo "changed")"

moment 03 "Escape — the status dismisses; the frame returns"
key Escape; sleep 2
cap fvl06-03-terminal-dismissed

moment 04 "Ctrl+T — honest browser status ('Open a chat before opening the Browser.')"
key ctrl+t; sleep 2.5
cap fvl06-04-browser-status
T2="$(frame_md5 fvl06-04-browser-status.png)"
say "frame delta vs baseline: $([ "$BASE_MD5" = "$T2" ] && echo "SILENT — FAIL candidate" || echo "changed")"

moment 05 "Escape — the status dismisses; the frame returns"
key Escape; sleep 2
cap fvl06-05-browser-dismissed

moment 06 "Ctrl+Alt+Shift+3 — the environments rail's honest empty state"
key ctrl+alt+shift+3; sleep 2.5
cap fvl06-06-environments-empty
sleep 1.5
cap fvl06-06b-environments-detail
T3="$(frame_md5 fvl06-06-environments-empty.png)"
say "frame delta vs baseline: $([ "$BASE_MD5" = "$T3" ] && echo "SILENT — FAIL candidate" || echo "changed")"

moment 07 "Escape — the rail section closes"
key Escape; sleep 2
cap fvl06-07-environments-closed

fv_end
