#!/usr/bin/env bash
# Scene B2 — composer affordances + panels (terminal, browser, computer use) + shortcuts overlay
# Composer bottom row approx: + (482,869), model (1125,869), effort (1240,869), speed (1330,869), send (1393,869).
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
CAPTURE b2-01-initial

# --- Terminal (Ctrl+`) ---
XDO key ctrl+grave; sleep 2; CAPTURE b2-02-terminal-open
# type into the PTY
XDO type --delay 60 "echo FLAUZ-LAB-TERMINAL-OK"; sleep 0.5; XDO key Return; sleep 1.5; CAPTURE b2-03-terminal-echo
XDO key ctrl+grave; sleep 1; CAPTURE b2-04-terminal-hidden

# --- Browser panel (Ctrl+T open browser tab) ---
XDO key ctrl+t; sleep 2.5; CAPTURE b2-05-browser-tab
# Focus address bar (Ctrl+L) and type
XDO key ctrl+l; sleep 1; XDO type --delay 60 "about:blank"; sleep 0.5; XDO key Return; sleep 2; CAPTURE b2-06-browser-address-bar
XDO key ctrl+shift+b; sleep 1; CAPTURE b2-07-browser-toggled

# --- Computer Use via palette ---
XDO key ctrl+k; sleep 1.2; XDO type --delay 60 "computer"; sleep 1; XDO key Return; sleep 2; CAPTURE b2-08-computer-use-inspector

# --- Keyboard shortcuts overlay (Ctrl+/) ---
XDO key ctrl+slash; sleep 1.5; CAPTURE b2-09-shortcuts-overlay
XDO key Escape; sleep 0.5

# --- Composer pickers ---
XDO key ctrl+shift+m; sleep 1.2; CAPTURE b2-10-model-picker
XDO key Escape; sleep 0.5
XDO mousemove 1240 869 click 1; sleep 1.2; CAPTURE b2-11-effort-picker
XDO key Escape; sleep 0.5
XDO mousemove 1330 869 click 1; sleep 1.2; CAPTURE b2-12-speed-picker
XDO key Escape; sleep 0.5
XDO mousemove 482 869 click 1; sleep 1.2; CAPTURE b2-13-composer-add-menu
XDO key Escape; sleep 0.5

# --- Slash commands in composer ---
XDO mousemove 800 800 click 1; sleep 0.5
XDO type --delay 60 "/"; sleep 1.5; CAPTURE b2-14-composer-slash
XDO key BackSpace; sleep 0.3

# --- @ mention file search in composer ---
XDO type --delay 60 "@"; sleep 1.5; CAPTURE b2-15-composer-mention
XDO key ctrl+a; XDO key Delete; sleep 0.5

# --- Send without auth (sign-in wall journey evidence) ---
XDO type --delay 60 "hello"; sleep 0.5; XDO key Return; sleep 3; CAPTURE b2-16-send-unauthenticated

# --- Sidebar toggle (Ctrl+B) ---
XDO key ctrl+b; sleep 1; CAPTURE b2-17-sidebar-toggled
XDO key ctrl+b; sleep 0.5

# --- Responsive: narrow window (sidebar collapse) ---
WIN=$(WINID); echo "winid=$WIN"
XDO windowsize "$WIN" 640 700; sleep 1.5; CAPTURE b2-18-narrow-window
XDO windowsize "$WIN" 1278 818; sleep 1
