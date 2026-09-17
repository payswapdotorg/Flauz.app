#!/usr/bin/env bash
# Scene D7b — WO-P2-004 supplement: palette top scrolled to reveal the full
# Settings group listing (wheel scroll; the unscrolled fold cuts after
# "Configuration" — Hooks and Git sit below it).
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
XDO key ctrl+k; sleep 1.2
XDO mousemove 800 380
for _ in 1 2 3 4 5 6; do XDO click 5; sleep 0.2; done
sleep 0.8
CAPTURE d7b-001-palette-top-scrolled-settings-group
XDO key Escape; sleep 0.5
