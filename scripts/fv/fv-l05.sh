#!/bin/bash
# FV-L05 — J-04 Discover a capability gap (the capability panel; Linux
# desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - capability panel: Ctrl+Alt+Shift+6 — KeyBinding::new(
#       &shortcut("alt-shift-6"), FlauzCapabilityGapShortcut) at
#       crates/codex-app/src/ui.rs:5809 (the shifted-symbol companion
#       alt-^ at ui.rs:5846)
#     - palette: Ctrl+K (ui.rs:5739)
#     - scoped Escape: KeyBinding::new("escape", Escape,
#       Some("FlauzCapabilityGap")) at ui.rs:5897
#   Copy (crates/codex-app/src/ui/flauz_capability_gap.rs):
#     - PANEL_HEADING "Capabilities" :58; PANEL_DESCRIPTION "What a task
#       can do — and what's missing when it can't" :60 (the subtitle the
#       catalog names)
#     - the pre-wiring copy naming the five dimensions: EMPTY_TITLE
#       "Capability resolution is on its way" :84; EMPTY_BODY "When a
#       capability is unavailable, this panel will say why — naming
#       exactly what is missing: the model, the runtime, the environment,
#       permissions, or workspace policy. No capability quietly
#       disappears." :86-88 (the five dimensions: model / runtime /
#       environment / permissions / workspace policy — the CapabilityDim-
#       ension enum :111-122)
#     - PALETTE_ROW_TITLE "Why is a capability unavailable?" :101
#       (description :103-104)
#     - ESCAPE_HINT "Escape closes this panel" :98
#
# Moments (the catalog FV-L05 row):
#   01 entry baseline
#   02 anchor task (the panel rides the task context)
#   03 Ctrl+Alt+Shift+6 — the capability panel (subtitle + the honest
#      pre-wiring copy naming the five dimensions)
#   04 settle/detail frame
#   05 scoped Escape — the panel closes (frame returns to the task surface)
#   06 palette row "Why is a capability unavailable?" + Return lands the
#      SAME panel
#   07 the chord frame and the palette-row frame are pixel-identical
#      (frame-compare: 06b vs 03)
#
# PASS (Lead-adjudicated, VLM reads + md5 compare):
#   - the panel subtitle "What a task can do — and what's missing when it
#     can't" + the no-silent-disappearance copy at 03;
#   - the chord frame (03) and the palette-row frame (06b) are
#     pixel-identical (md5 equal or visually identical);
#   - Escape returns to the task surface (05).
#
# Usage: scripts/fv/fv-l05.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l05 "J-04" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl05-01-entry

moment 02 "anchor task (the capability affordance rides the task context)"
key ctrl+n; sleep 2.5
type_ "FV-L05 anchor task"; sleep 1.5
key ctrl+Return; sleep 6
cap fvl05-02-task-surface

moment 03 "Ctrl+Alt+Shift+6 — the capability panel (subtitle + the five-dimension pre-wiring copy)"
key ctrl+alt+shift+6; sleep 2.5
cap fvl05-03-capability-panel
CHORD_FRAME="$(frame_md5 fvl05-03-capability-panel.png)"
say "chord frame md5: $CHORD_FRAME"
sleep 1.5
cap fvl05-04-capability-detail

moment 05 "scoped Escape — the panel closes (the task surface stands)"
key Escape; sleep 2
cap fvl05-05-panel-closed

moment 06 "palette row 'Why is a capability unavailable?' + Return lands the SAME panel"
key ctrl+k; sleep 2.5
type_ "Why is a capability unavailable"; sleep 2
cap fvl05-06-palette-row
key Return; sleep 2.5
cap fvl05-06b-palette-landed
PALETTE_FRAME="$(frame_md5 fvl05-06b-palette-landed.png)"
say "palette-landed frame md5: $PALETTE_FRAME"

moment 07 "frame-compare: the chord frame vs the palette-row frame (expect pixel-identical)"
if [ "$CHORD_FRAME" = "$PALETTE_FRAME" ] && [ -n "$CHORD_FRAME" ]; then
  say "FRAME-COMPARE: identical (md5 match)"
else
  say "FRAME-COMPARE: md5 differs ($CHORD_FRAME vs $PALETTE_FRAME) — the Lead adjudicates pixel-identity (settle jitter is the known d-series cause; see the w4-gate d26 record)"
fi
key Escape; sleep 1.5

moment 08 "Escape settle — final frame"
key Escape; sleep 2
cap fvl05-08-settle

fv_end
