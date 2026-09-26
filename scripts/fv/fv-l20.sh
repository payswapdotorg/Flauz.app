#!/bin/bash
# FV-L20 — A11Y (the RG-A11Y-01/02 re-run at current main; Linux desktop
# lane, FV-002 / Wave 7): keyboard-only across the six F1 surfaces, the
# focus probes, the guarded-chord ladder.
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   The six F1 surfaces + their chords (the rg-a11y-01 precedent legs,
#   recalibrated at this base):
#     - PALETTE: Ctrl+K (ui.rs:5739). The palette close must restore
#       focus so a typed probe lands in the composer (the
#       no-swallowed-keyboard contract; the WO-P2-018/UX-003 test
#       regions pin it, ui.rs:45576/55832).
#     - SETTINGS: Ctrl+, (KeyBinding shortcut(","), OpenSettingsShortcut,
#       ui.rs:5778) with its own Ctrl+F search (the settings search
#       input, ui.rs:12327-12331).
#     - TERMINAL: Ctrl+` (ui.rs:5777). The PTY focus transfer (ui.rs:426
#       comment + the pending transfer :6635-6643): typed keys land in
#       the PTY, not the composer (the N5 contract).
#     - BROWSER: Ctrl+Shift+B (the "toggleBrowserPanel" registry row,
#       ui.rs:4216; the FV-L07 calibration).
#     - HISTORY: the settings search path to the Browser section (the
#       rg-a11y-01 LEG-5 precedent: Ctrl+, then Ctrl+F "browsing").
#     - ATTENTION: the unread family (ui.rs:3251-3256 / 3334-3340 /
#       3341-3347 — the FV-L18 calibration) + the Activity view
#       (Ctrl+Alt+U, ui.rs:3327-3333).
#   The shortcuts-overlay two-Escape contract: Ctrl+/ (ShowKeyboardShort-
#   cuts, ui.rs:5779) opens the overlay; with a query typed, the FIRST
#   Escape clears the query and the SECOND closes the overlay
#   (keyboard_shortcuts_escape_closes_overlay, ui.rs:11817-11828 +
#   49164-49206).
#   Ctrl+P no-workspace honest status: "Select a workspace before
#   searching files." (files_palette_command_status, ui.rs:48202-48209;
#   the F-A4 guard pinned at ui.rs:55628-55652).
#   The guarded-chord ladder (every advertised chord answers — zero
#   silent frames): the panel family M/6/R/P/S/U/L/Y/7 =
#   Ctrl+Alt+Shift+{m,6,r,p,s,u,l,y,7} (ui.rs:5806, 5809, 5815, 5822,
#   5855, 5862, 5868, 5874, 5853); the rail family 1..5 =
#   Ctrl+Alt+Shift+1..5 (ui.rs:5791-5799); the nav family 1..4 =
#   Ctrl+Alt+1..4 (ui.rs:5787-5790).
#   The reachable modal traps: the modal family owns tab/shift-tab
#   scoped bindings (ui.rs:5944-6041) and closes on Escape (the 019
#   request-once discipline — never trapped).
#
# Moments (the catalog FV-L20 row):
#   00 entry baseline
#   1  PALETTE: open → filter → Escape close → typed probe lands in the
#      composer (the palette close restore)
#   2  SETTINGS: Ctrl+, → Ctrl+F search → Escape return
#   3  TERMINAL: Ctrl+` → typed echo (PTY receives) → close
#   4  BROWSER: Ctrl+Shift+B → Escape → close
#   5  HISTORY: Ctrl+, → Ctrl+F "browsing" → the Browser section
#   6  ATTENTION: mark → jump → clear-all (the unread family)
#   7  OVERLAY: Ctrl+/ → typed query → Escape 1 (clears the query) →
#      Escape 2 (closes) — the two-Escape contract
#   8  Ctrl+P no-workspace honest status ("Select a workspace before
#      searching files.")
#   9  the guarded-chord ladder: M/6/R/P/S/U/L/Y/7 + rail 1..5 + nav
#      1..4 (every chord → capture; zero silent frames)
#
# PASS (Lead-adjudicated, frames + typed-probe reads):
#   - each surface opens/closes keyboard-only;
#   - the typed probe lands with the caret in the composer after
#     palette/overlay close;
#   - first Escape clears the overlay query, second closes;
#   - PTY receives typed keys on open;
#   - each guarded chord produces its honest status (zero silent frames
#     — the A-3 verdict re-proven);
#   - the chord ladder covers the Flauz panel family (M/6/R/P/S/U/L/Y/7
#     + rail 1..5 + nav 1..4).
#
# Usage: scripts/fv/fv-l20.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l20 "A11Y" "${1:?binary path required}" "${2:-}"

moment 00 "entry baseline"
key Escape; sleep 1
cap fvl20-00-entry
LADDER_BASE="$(frame_md5 fvl20-00-entry.png)"

moment 1 "PALETTE: Ctrl+K open → filter → Escape close → typed probe lands in the composer"
key ctrl+k; sleep 2.5
cap fvl20-01a-palette-open
type_ "settings"; sleep 2
cap fvl20-01b-palette-filtered
key Escape; sleep 1.5
cap fvl20-01c-palette-closed
type_ "a11y probe one"; sleep 1
cap fvl20-01d-typed-probe-lands
say "typed probe after palette close must show in the composer (the no-swallowed-keyboard contract)"
key ctrl+a; key Delete; sleep 1

moment 2 "SETTINGS: Ctrl+, → Ctrl+F search → Escape return"
key ctrl+comma; sleep 2.5
cap fvl20-02a-settings-open
key ctrl+f; sleep 1
type_ "appearance"; sleep 2
cap fvl20-02b-settings-search
key Escape; sleep 1
key Escape; sleep 2
cap fvl20-02c-settings-return

moment 3 "TERMINAL: Ctrl+\` → typed echo (PTY receives) → close"
key ctrl+grave; sleep 3
cap fvl20-03a-terminal-open
type_ "echo a11y-fv-l20-pty-focus"; sleep 0.8
key Return; sleep 2.5
cap fvl20-03b-terminal-echo
say "the PTY (not the composer) must show the typed keys + output (the N5 focus-transfer contract)"
key ctrl+grave; sleep 2
cap fvl20-03c-terminal-closed

moment 4 "BROWSER: Ctrl+Shift+B → Escape → close"
key ctrl+shift+b; sleep 3
cap fvl20-04a-browser-open
key Escape; sleep 1.5
key ctrl+shift+b; sleep 2
cap fvl20-04b-browser-closed

moment 5 "HISTORY: Ctrl+, → Ctrl+F 'browsing' → the Browser section"
key ctrl+comma; sleep 2.5
key ctrl+f; sleep 1
type_ "browsing"; sleep 2
cap fvl20-05a-history-nav
key Escape; sleep 1
key Escape; sleep 2
cap fvl20-05b-history-return

moment 6 "ATTENTION: mark → jump → clear-all (the unread family)"
key ctrl+n; sleep 2.5
type_ "a11y ladder attention chat"; sleep 1
key ctrl+Return; sleep 8
cap fvl20-06a-chat-created
key ctrl+shift+u; sleep 2
cap fvl20-06b-marked-unread
key ctrl+alt+a; sleep 2
cap fvl20-06c-jump-cleared
key shift+Escape; sleep 2
cap fvl20-06d-clear-all

moment 7 "OVERLAY: Ctrl+/ → typed query → Escape 1 clears the query → Escape 2 closes (the two-Escape contract)"
key ctrl+slash; sleep 2.5
cap fvl20-07a-overlay-open
type_ "palette"; sleep 1.5
cap fvl20-07b-overlay-query
key Escape; sleep 1.5
cap fvl20-07c-overlay-esc1-query-cleared
key Escape; sleep 1.5
cap fvl20-07d-overlay-esc2-closed
type_ "a11y probe two"; sleep 1
cap fvl20-07e-typed-probe-lands
say "typed probe after overlay close must show in the composer"
key ctrl+a; key Delete; sleep 1

moment 8 "Ctrl+P no-workspace honest status ('Select a workspace before searching files.')"
key ctrl+p; sleep 2.5
cap fvl20-08-ctrl-p-no-workspace-status
key Escape; sleep 1.5

moment 9 "the guarded-chord ladder — every advertised chord answers (zero silent frames)"
say "the panel family (M/6/R/P/S/U/L/Y/7) + the rail family (1..5) + the nav family (1..4)"
for chord in ctrl+alt+shift+m ctrl+alt+shift+6 ctrl+alt+shift+r ctrl+alt+shift+p ctrl+alt+shift+s ctrl+alt+shift+u ctrl+alt+shift+l ctrl+alt+shift+y ctrl+alt+shift+7 ctrl+alt+shift+1 ctrl+alt+shift+2 ctrl+alt+shift+3 ctrl+alt+shift+4 ctrl+alt+shift+5 ctrl+alt+1 ctrl+alt+2 ctrl+alt+3 ctrl+alt+4; do
  tag="$(echo "$chord" | tr '+' '-')"
  key "$chord"; sleep 2
  cap "fvl20-09-ladder-$tag"
  F="$(frame_md5 "fvl20-09-ladder-$tag.png")"
  if [ "$F" = "$LADDER_BASE" ] && [ -n "$F" ]; then
    say "LADDER $chord: frame SILENT (identical to the baseline) — FAIL candidate (the A-3 law: every advertised chord answers)"
  else
    say "LADDER $chord: frame changed (answered)"
  fi
  key Escape; sleep 1.5
  key Escape; sleep 1
done
cap fvl20-09z-ladder-final

say "FV-L20 COMPLETE — the ladder covered the Flauz panel family (M/6/R/P/S/U/L/Y/7 + rail 1..5 + nav 1..4); the Lead adjudicates zero-silent-frames + the typed-probe reads"

fv_end
