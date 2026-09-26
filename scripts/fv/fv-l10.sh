#!/bin/bash
# FV-L10 — J-09 Understand what happened (the Evidence section + Find +
# Copy as Markdown; Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - Evidence rail section: Ctrl+Alt+Shift+4 — KeyBinding::new(
#       &shortcut("alt-shift-4"), FlauzTaskEvidenceShortcut) at
#       crates/codex-app/src/ui.rs:5798
#     - Find: Ctrl+F — registry row "findInThread" (CmdOrCtrl+F; the
#       KeyBinding shortcut("f") at ui.rs:5772)
#     - find advance: Ctrl+G (the find-bar next tooltip "Next result ·
#       Ctrl+G", ui.rs:14836) and F3 (FIXED_KEYBOARD_SHORTCUTS
#       "Find Next" F3, ui.rs:49216-49217)
#     - Copy as Markdown: the registry row "copyConversationMarkdown"
#       (title "Copy as Markdown", ui.rs:3258-3263 — no default chord;
#       driven through the palette), executing
#       copy_selected_conversation_as_markdown (ui.rs:12298)
#   Copy:
#     - Evidence honest empty: "No evidence collected yet" —
#       crates/codex-app/src/ui/flauz_shell/mod.rs:306 (empty_title);
#       the claims-vs-verified body (empty_body, mod.rs:3xx —
#       "Observations, claims, verification and provenance" rail tooltip
#       mod.rs:273)
#     - the Find counter: "{n} / {m}{+} results" — ui.rs:14777-14781
#       ("0 results" :14770; the "+" suffix when capped/truncated
#       :14772-14776); highlighted matches + advance (the non-color
#       distinction — C-17 — decided in the find-bar design)
#     - the seeded chat timeline: donor state (wo-p2-008 pattern) so the
#       Find positive path has real matches and the markdown export
#       carries a real turn
#
# Moments (the catalog FV-L10 row):
#   01 entry baseline (seeded donor state)
#   02 select the seeded chat — the timeline visible
#   03 Ctrl+Alt+Shift+4 — the rail Evidence section (honest empty)
#   04 Escape — the section closes
#   05 Ctrl+F — the Find bar opens; type a term present in the seeded
#      timeline
#   06 the counter ("N / M+ results") + highlighted matches
#   07 advance (Ctrl+G / F3)
#   08 Escape — the Find bar closes
#   09 palette "Copy as Markdown" + Return — the export (the clipboard
#      holds the seeded turn; the capture + the app log carry the
#      bounded-export behavior)
#
# PASS (Lead-adjudicated, VLM reads):
#   - the evidence empty state verbatim ("No evidence collected yet" +
#     the claims-vs-verified body);
#   - the Find bar + counter "N / M+ results" + highlighted matches +
#     advance (the non-color distinction — C-17);
#   - the markdown export contains the seeded turn (the clipboard
#     content is read at the Lead station via the app's bounded export —
#     xclip/xsel if present, else the app log + the palette-success
#     frame; honest bound named if the station lacks a clipboard probe).
#
# Usage: scripts/fv/fv-l10.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime
fv_need_donor

fv_begin fv-l10 "J-09" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline (seeded donor state)"
cap fvl10-01-entry

moment 02 "select the seeded chat — the timeline visible"
key Tab; key Tab; key Tab; sleep 1
key Down; sleep 1
key Return; sleep 2.5
cap fvl10-02-seeded-timeline

moment 03 "Ctrl+Alt+Shift+4 — the rail Evidence section (the honest empty)"
key ctrl+alt+shift+4; sleep 2.5
cap fvl10-03-evidence-empty
sleep 1.5
cap fvl10-03b-evidence-detail

moment 04 "Escape — the section closes"
key Escape; sleep 2
cap fvl10-04-evidence-closed

moment 05 "Ctrl+F — the Find bar opens; type a term present in the seeded timeline"
key ctrl+f; sleep 2
cap fvl10-05-find-open
type_ "a"; sleep 2
cap fvl10-06-find-counter
say "the counter must read 'N / M+ results' (ui.rs:14777-14781) with highlighted matches (the non-color distinction, C-17)"

moment 07 "advance (Ctrl+G — the find-bar next tooltip chord)"
key ctrl+g; sleep 1.5
cap fvl10-07-find-advance
key F3; sleep 1.5
cap fvl10-07b-find-advance-f3

moment 08 "Escape — the Find bar closes"
key Escape; sleep 2
cap fvl10-08-find-closed

moment 09 "palette 'Copy as Markdown' + Return — the export"
key ctrl+k; sleep 2.5
type_ "Copy as Markdown"; sleep 2
cap fvl10-09-palette-copy-md
key Return; sleep 2
cap fvl10-09b-export-done
if command -v xclip >/dev/null 2>&1; then
  xclip -selection clipboard -o > "$FV_OUT/export-markdown.txt" 2>/dev/null || true
  say "clipboard probe: export archived to export-markdown.txt (xclip)"
elif command -v xsel >/dev/null 2>&1; then
  xsel --clipboard --output > "$FV_OUT/export-markdown.txt" 2>/dev/null || true
  say "clipboard probe: export archived to export-markdown.txt (xsel)"
else
  say "HONEST BOUND: no clipboard probe (xclip/xsel) at this station — the Lead reads the exported markdown at the gate station (the export itself is the calibrated ui.rs:12298 path)"
fi

fv_end
