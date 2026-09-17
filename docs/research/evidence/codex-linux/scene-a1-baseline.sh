#!/usr/bin/env bash
# a1-baseline.sh — WO-LAB-001 scene: official Linux preview app baseline
# surfaces, unauthenticated. Captures (all 1600x1000, same geometry as Flauz
# sessions): boot/entry surface, settled state, window controls probing,
# official affordance shapes for terminal/browser/palette/shortcut surfaces.
# NOTE: every capture is prefixed a1- for evidence curation.
set -u
source /home/z/parity-lab/tools/scene_helpers.sh

sleep 6
CAPTURE a1-01-entry
sleep 8
CAPTURE a1-02-settled

# window tree truth (what surfaces exist, window title)
XWIN -root -tree > "$WS/shots/a1-window-tree.txt" 2>&1
XDPY > "$WS/shots/a1-display-info.txt" 2>&1

# Try to focus/activate the main window (click center like Flauz scenes)
XDO mousemove 800 500 click 1
sleep 2
CAPTURE a1-03-focused

# Official reference-matrix affordance probes (unauthenticated surfaces):
# 1. Command palette (Ctrl+K per official docs)
XDO key ctrl+k
sleep 2
CAPTURE a1-04-ctrl-k
XDO key Escape
sleep 1

# 2. Command palette alternate (Ctrl+Shift+P)
XDO key ctrl+shift+p
sleep 2
CAPTURE a1-05-ctrl-shift-p
XDO key Escape
sleep 1

# 3. Browser tab (Ctrl+T — official "Open browser tab")
XDO key ctrl+t
sleep 3
CAPTURE a1-06-ctrl-t
XDO key Escape
sleep 1

# 4. Browser panel toggle (Ctrl+Shift+B — official "Toggle browser panel")
XDO key ctrl+shift+b
sleep 3
CAPTURE a1-07-ctrl-shift-b
XDO key Escape
sleep 1

# 5. Terminal (Ctrl+` — official "Toggle terminal", current-docs binding)
XDO key ctrl+grave
sleep 3
CAPTURE a1-08-ctrl-grave
XDO key Escape
sleep 1

# 6. Settings surface (Ctrl+, — common Electron settings binding)
XDO key ctrl+comma
sleep 3
CAPTURE a1-09-ctrl-comma
XDO key Escape
sleep 1

# 7. New chat / new task surface (Ctrl+N common; Cmd+N official macOS analog)
XDO key ctrl+n
sleep 2
CAPTURE a1-10-ctrl-n
XDO key Escape
sleep 1

# 8. Fullscreen (F11) — window chrome behavior
XDO key F11
sleep 2
CAPTURE a1-11-fullscreen
XDO key F11
sleep 1

# 9. Keyboard shortcuts overlay (Ctrl+/ — commonly the shortcuts list)
XDO key ctrl+slash
sleep 2
CAPTURE a1-12-ctrl-slash
XDO key Escape
sleep 1

# 10. Menu bar probe (Alt — opens menu in some Electron apps)
XDO key alt
sleep 1
XDO key alt+F
sleep 2
CAPTURE a1-13-menu-probe
XDO key Escape
sleep 1

# settled final
CAPTURE a1-14-final
echo "a1-baseline: scene complete"
