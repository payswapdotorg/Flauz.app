#!/usr/bin/env bash
# d9b-close-probe.sh — find the side panel close button + prove the close path.
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
WID=$(WINID)
XDO windowfocus --sync "$WID" 2>/dev/null || XDO windowfocus "$WID"
sleep 0.5
for Y in 849 829 809 789; do XDO mousemove 802 $Y click 1; sleep 0.3; done

# create + select the main chat
XDO type --delay 40 "close probe main"
sleep 0.6
XDO key ctrl+Return
sleep 11
XDO key Escape
sleep 2

# open the side chat
XDO key ctrl+alt+s
sleep 2
CAPTURE d9b-01-side-open

# candidate close-button positions (panel header right edge, top of panel)
# window root span x163-1441, y91-909. Panel right-docked ~x1081-1441.
for POS in "1418 160" "1418 185" "1418 140" "1400 160"; do
  set -- $POS
  XDO mousemove $1 $2 click 1
  sleep 1.2
  CAPTURE "d9b-click-$1-$2"
done
echo "probe complete"
