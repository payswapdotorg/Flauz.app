#!/usr/bin/env bash
# Scene B1 — sidebar navigation sweep + settings pages (Worker B, Task 41-B)
# Window: 1278x818 centered on 1600x1000 → screen origin (161,91); titlebar 34px.
# Sidebar 275px wide (screen x 161..436). Sidebar buttons: New chat y≈150,
# Repository y≈186, Pull requests y≈222, Plugins y≈258, Workflows y≈294.
# Footer Settings row y≈856. Settings nav column screen x 436..710.
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
CAPTURE b1-01-initial-welcome

XDO mousemove 300 150 click 1; sleep 1.5; CAPTURE b1-02-sidebar-newchat
XDO mousemove 300 186 click 1; sleep 1.5; CAPTURE b1-03-sidebar-repository
XDO mousemove 300 222 click 1; sleep 2;   CAPTURE b1-04-sidebar-pullrequests
XDO mousemove 300 258 click 1; sleep 2.5; CAPTURE b1-05-sidebar-plugins
XDO mousemove 300 294 click 1; sleep 2.5; CAPTURE b1-06-sidebar-workflows
XDO mousemove 300 856 click 1; sleep 2;   CAPTURE b1-07-settings-general

# Command palette (Ctrl+K) — capture empty, then filtered, then navigate
XDO key ctrl+k; sleep 1.5; CAPTURE b1-08-command-palette
XDO type --delay 60 "appearance"; sleep 1; CAPTURE b1-09-palette-appearance
XDO key Return; sleep 1.5; CAPTURE b1-10-settings-appearance

XDO key ctrl+k; sleep 1; XDO type --delay 60 "keyboard"; sleep 1; XDO key Return; sleep 1.5; CAPTURE b1-11-settings-keyboard
XDO key ctrl+k; sleep 1; XDO type --delay 60 "usage"; sleep 1; XDO key Return; sleep 1.5; CAPTURE b1-12-settings-usage
XDO key ctrl+k; sleep 1; XDO type --delay 60 "connections"; sleep 1; XDO key Return; sleep 1.5; CAPTURE b1-13-settings-connections
XDO key ctrl+k; sleep 1; XDO type --delay 60 "archived"; sleep 1; XDO key Return; sleep 1.5; CAPTURE b1-14-settings-archived

# Settings search box (top of settings nav column) — type a query
XDO mousemove 573 165 click 1; sleep 0.5
XDO type --delay 50 "mcp"; sleep 1.5; CAPTURE b1-15-settings-search-mcp
XDO key Escape; sleep 0.5

# Back to chats via sidebar New chat
XDO mousemove 300 150 click 1; sleep 1.5; CAPTURE b1-16-back-to-tasks
