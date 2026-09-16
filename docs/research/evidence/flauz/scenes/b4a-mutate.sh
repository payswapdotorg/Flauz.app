#!/usr/bin/env bash
# Scene B4a — mutate state for the restart-persistence test:
# switch Appearance theme (Light) + resize window, so session B4b can check restoration.
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
# Open Appearance settings via palette
XDO key ctrl+k; sleep 1.2; XDO type --delay 60 "appearance"; sleep 1; XDO key Return; sleep 1.5
CAPTURE b4a-01-appearance-before
# Appearance page content: theme selector near the top of the content pane (screen x ~ 730..1419)
XDO mousemove 860 220 click 1; sleep 1; CAPTURE b4a-02-appearance-clicked
# resize the window so b4b can check window placement restore
WIN=$(WINID); echo "winid=$WIN"
XDO windowsize "$WIN" 1100 900; sleep 1
CAPTURE b4a-03-window-resized
