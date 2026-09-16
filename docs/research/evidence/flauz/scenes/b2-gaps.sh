#!/usr/bin/env bash
# Scene B2-gaps — Worker B2 gap-filling (Task 41-B2):
# terminal/browser toggles on the CHAT view (Worker B's b2 ran them on the
# Workflows/settings view where they are no-ops), composer slash/@ mention menus,
# unauthenticated send, Settings > Import detection (fabricated Claude Code
# session already present in $HOME/.claude), and the mutation phase for the
# restart-persistence check (Appearance theme + window resize).
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
CAPTURE b2g-01-boot-restored

# --- New chat (Ctrl+N): reach the chat view with composer ---
XDO key ctrl+n; sleep 2; CAPTURE b2g-02-new-chat

# --- Terminal dock on chat view (Ctrl+`) ---
XDO key ctrl+grave; sleep 2; CAPTURE b2g-03-terminal-dock-open
XDO type --delay 60 "echo FLAUZ-LAB-TERMINAL-OK"; sleep 0.4; XDO key Return; sleep 1.5; CAPTURE b2g-04-terminal-echo
XDO key ctrl+grave; sleep 1.2; CAPTURE b2g-05-terminal-hidden

# --- Browser tab on chat view (Ctrl+T / Ctrl+L / Ctrl+Shift+B) ---
XDO key ctrl+t; sleep 3; CAPTURE b2g-06-browser-tab-open
XDO key ctrl+l; sleep 1; XDO type --delay 50 "data:text/plain,FLAUZ-LAB-BROWSER-OK"; sleep 0.5; XDO key Return; sleep 2.5; CAPTURE b2g-07-browser-navigated
XDO key ctrl+shift+b; sleep 1.5; CAPTURE b2g-08-browser-panel-toggled

# --- Computer Use inspector via palette (on chat view this time) ---
XDO key ctrl+k; sleep 1.2; XDO type --delay 60 "computer use"; sleep 1; XDO key Return; sleep 2.5; CAPTURE b2g-09-computer-use-inspector
XDO key Escape; sleep 0.5

# --- Composer: slash menu, @ mention menu, unauthenticated send ---
XDO key ctrl+n; sleep 1.5
XDO mousemove 800 820 click 1; sleep 0.6
XDO type --delay 50 "/"; sleep 1.5; CAPTURE b2g-10-slash-menu
XDO key BackSpace; sleep 0.4
XDO type --delay 50 "@"; sleep 1.5; CAPTURE b2g-11-mention-menu
XDO key ctrl+a; XDO key Delete; sleep 0.4
XDO type --delay 50 "hello"; sleep 0.5; XDO key Return; sleep 4; CAPTURE b2g-12-send-unauthenticated

# --- Settings > Import (detection of $HOME/.claude session) ---
XDO mousemove 300 856 click 1; sleep 2; CAPTURE b2g-13-settings-open
XDO mousemove 573 165 click 1; sleep 0.5
XDO type --delay 50 "import"; sleep 1.2; CAPTURE b2g-14-settings-search-import
XDO mousemove 573 240 click 1; sleep 2.5; CAPTURE b2g-15-import-page

# --- Mutation for restart persistence: Appearance theme -> Light + resize ---
XDO key ctrl+k; sleep 1.2; XDO type --delay 60 "appearance"; sleep 1; XDO key Return; sleep 2
CAPTURE b2g-16-appearance-before
XDO mousemove 950 250 click 1; sleep 1.5; CAPTURE b2g-17-appearance-light-clicked
WIN=$(WINID); echo "winid=$WIN"
XDO windowsize "$WIN" 1100 900; sleep 1.5; CAPTURE b2g-18-window-resized
