#!/bin/bash
# FV-L07 — J-06 chat-scoped slice (the terminal dock + the browser panel
# honest state; Linux desktop lane, FV-002 / Wave 7). BOUNDED: no live
# turn (the auth wall — catalog gap L-1; named in the run notes).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - terminal dock: Ctrl+` — ui.rs:5777 (ToggleTerminalShortcut). The
#       dock opens with a live tab and the PTY focus transfer: "opening
#       the dock must transfer keyboard focus to the terminal dock's PTY
#       input" (ui.rs:426 comment; the pending-PTY-focus transfer armed
#       at ui.rs:6635-6643). Typed keys land in the PTY, not the composer
#       (the N5 contract observable — the rg-a11y-01 LEG-3 precedent:
#       type 'echo …' + Return → the echo frame).
#     - browser panel toggle: Ctrl+Shift+B — registry row
#       "toggleBrowserPanel" (shortcut "Ctrl+Shift+B" at ui.rs:4216;
#       tooltip ui.rs:509)
#     - composer submit: Ctrl+Return (D11r FW-5)
#   Copy: the browser panel's honest state with no page open —
#   "Open a page before reloading the Browser." is the reload guard
#   (codex-core lib.rs:15270, pinned ui.rs:55442); the empty browser
#   pane carries its own empty copy (bounded, no fabricated content).
#   The seeded chat is selected (donor state) so the docks are
#   chat-scoped; the timeline is visible on the task surface.
#
# Moments (the catalog FV-L07 row):
#   01 entry baseline (seeded donor state)
#   02 select the seeded chat — the timeline visible
#   03 Ctrl+\` — the dock opens with a live tab (+ a prompt)
#   04 the PTY focus transfer: typed keys land in the PTY (echo probe +
#      Return) — the N5 contract observable
#   05 Ctrl+\` — the dock closes
#   06 Ctrl+Shift+B — the browser panel opens (its honest state)
#   07 Escape — the browser panel closes
#
# PASS (Lead-adjudicated, VLM reads):
#   - the dock with a tab + a prompt at 03;
#   - a typed echo frame at 04 (the echo output visible — the PTY
#     received the keys, the composer did not);
#   - the browser panel's honest state at 06;
#   - the timeline visible throughout (the seeded chat's).
#   Run notes name the auth bound: no live turn runs in this scene
#   (gap L-1 — the honest-state slice is the verifiable truth).
#
# Usage: scripts/fv/fv-l07.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime
fv_need_donor

fv_begin fv-l07 "J-06" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline (seeded donor state)"
cap fvl07-01-entry

moment 02 "select the seeded chat — the timeline visible"
key Tab; key Tab; key Tab; sleep 1
key Down; sleep 1
key Return; sleep 2.5
cap fvl07-02-seeded-chat-timeline

moment 03 "Ctrl+\` — the dock opens with a live tab (+ a prompt)"
key ctrl+grave; sleep 3
cap fvl07-03-terminal-dock-open

moment 04 "the PTY focus transfer: typed keys land in the PTY (the N5 contract observable)"
type_ "echo fv-l07-pty-focus-probe"; sleep 0.8
cap fvl07-04a-pty-typed
key Return; sleep 2.5
cap fvl07-04b-pty-echo
say "the echo probe: the PTY (not the composer) must show the typed keys and their output"

moment 05 "Ctrl+\` — the dock closes"
key ctrl+grave; sleep 2
cap fvl07-05-terminal-dock-closed

moment 06 "Ctrl+Shift+B — the browser panel opens (its honest state)"
key ctrl+shift+b; sleep 3
cap fvl07-06-browser-panel-honest
sleep 1.5
cap fvl07-06b-browser-detail

moment 07 "Escape — the browser panel closes"
key Escape; sleep 1.5
key ctrl+shift+b; sleep 2
cap fvl07-07-browser-closed

say "RUN NOTE (the auth bound, catalog gap L-1): no live turn runs in this scene — the honest-state slice (dock + PTY + browser honest state + timeline) is the verifiable truth at this base"

fv_end
