#!/usr/bin/env bash
# a3-overlay.sh — WO-LAB-001 scene 3: complete the Keyboard shortcuts overlay
# inventory (wheel-scroll), plus quick binding probes at the login surface.
set -u
source /home/z/parity-lab/tools/scene_helpers.sh

sleep 8
XDO mousemove 800 500 click 1
sleep 1

# --- shortcuts overlay: open + wheel-scroll to the bottom ---
XDO key ctrl+slash
sleep 2
CAPTURE a3-01-overlay-open
XDO mousemove 800 400
for i in 1 2 3 4 5 6 7 8; do
  XDO click 5; XDO click 5; XDO click 5
  sleep 0.6
done
CAPTURE a3-02-overlay-scrolled
XDO mousemove 800 700
for i in 1 2 3; do XDO click 5; XDO click 5; sleep 0.5; done
CAPTURE a3-03-overlay-scrolled2
XDO key Escape
sleep 1

# --- binding probes at login surface ---
XDO key ctrl+shift+o
sleep 2
CAPTURE a3-04-ctrl-shift-o
XDO key Escape
XDO key ctrl+f
sleep 2
CAPTURE a3-05-ctrl-f
XDO key Escape
XDO key ctrl+alt+o
sleep 2
CAPTURE a3-06-ctrl-alt-o
XDO key Escape
sleep 1
CAPTURE a3-07-final
echo "a3-overlay: scene complete"
