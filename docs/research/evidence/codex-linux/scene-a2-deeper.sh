#!/usr/bin/env bash
# a2-deeper.sh — WO-LAB-001 scene 2: deeper unauthenticated surfaces of the
# official Linux preview app: full shortcuts overlay (scrolled), API-key entry
# surface, mode-switcher (Alt+N) probes, palette filtering, auth-flow shape
# (click "Continue to sign in" — capture the redirect surface, NEVER sign in).
set -u
source /home/z/parity-lab/tools/scene_helpers.sh

sleep 8
CAPTURE a2-00-baseline

# --- 1. Full shortcuts overlay: open + scroll through it ---
XDO key ctrl+slash
sleep 2
CAPTURE a2-01-shortcuts-top
XDO key Page_Down
sleep 1
CAPTURE a2-02-shortcuts-pgdn
XDO key Page_Down
sleep 1
CAPTURE a2-03-shortcuts-pgdn2
XDO key Escape
sleep 1

# --- 2. Palette: type queries, observe filtering at login surface ---
XDO key ctrl+k
sleep 2
XDO type --delay 60 "settings"
sleep 1
CAPTURE a2-04-palette-settings
XDO key ctrl+a
XDO type --delay 60 "terminal"
sleep 1
CAPTURE a2-05-palette-terminal
XDO key ctrl+a
XDO type --delay 60 "import"
sleep 1
CAPTURE a2-06-palette-import
XDO key Escape
sleep 1

# --- 3. Mode switcher probes (Alt+1/2/3 per "Switch to Work: Alt+2") ---
XDO key alt+1
sleep 2
CAPTURE a2-07-alt1
XDO key alt+2
sleep 2
CAPTURE a2-08-alt2
XDO key alt+3
sleep 2
CAPTURE a2-09-alt3
XDO key Escape
sleep 1

# --- 4. API-key sign-in surface ---
# window is 1090x760 at +255+120; center column: approx x=800, y=470 for the
# secondary button (per VLM read: below the dark primary button), y~520 for
# Sign up link. Click secondary button "Sign in with an API key".
XDO mousemove 800 470 click 1
sleep 3
CAPTURE a2-10-apikey-surface
XDO key Escape
sleep 1
XDO key ctrl+k
XDO key Escape
sleep 1

# --- 5. Auth-flow shape: click "Continue to sign in" (primary, y~430) ---
XDO mousemove 800 430 click 1
sleep 5
CAPTURE a2-11-auth-flow
sleep 5
CAPTURE a2-12-auth-flow-settled
XWIN -root -tree > "$WS/shots/a2-window-tree.txt" 2>&1

# close any auth window with Escape
XDO key Escape
sleep 1
CAPTURE a2-13-final
echo "a2-deeper: scene complete"
