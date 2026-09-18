#!/usr/bin/env bash
# Scene D9 — WO-P2-006 side-chats verification on the FIXED binary (006 branch).
# Calibrated input recipe (D8c lessons): windowfocus + click ladder; composer
# zone y749-829 at x802; submits are CTRL+RETURN. Window 1278x818+163+91 →
# right-docked 360px side panel spans root x~1081-1441, side composer ~ (1261,789).
#
# PASS criteria (VLM-read): (a) ctrl+alt+s opens a side panel with the MAIN
# chat still selected; (b) the side composer accepts text; (c) after a side
# submit the main chat remains selected with its draft intact; (d) the side
# panel closes without affecting the main chat; (e) /side reopens it.
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
WID=$(WINID)
echo "WINID=$WID"

# input recipe
XDO windowfocus --sync "$WID" 2>/dev/null || XDO windowfocus "$WID"
sleep 0.5
for Y in 849 829 809 789; do
  XDO mousemove 802 $Y click 1
  sleep 0.3
done

# 1. create + select the main chat
XDO type --delay 40 "side chat fixture main"
sleep 0.8
CAPTURE d9-001-typed
XDO key ctrl+Return
sleep 12
CAPTURE d9-002-main-created

# 2. settle, dismiss retry banner
XDO key Escape
sleep 2.5
CAPTURE d9-003-settled

# 3. ENTRY PATH 1: ctrl+alt+s opens the side chat
XDO key ctrl+alt+s
sleep 2.0
CAPTURE d9-004-side-open

# 4. type in the side composer (right-docked panel)
XDO mousemove 1261 789 click 1
sleep 0.6
XDO type --delay 40 "quick aside question"
sleep 1.0
CAPTURE d9-005-side-typed

# 5. submit the side message
XDO key ctrl+Return
sleep 8
CAPTURE d9-006-side-submitted

# 6. main chat intact? capture the sidebar + main composer state
sleep 4
CAPTURE d9-007-aftermath

# 7. close the side panel (Escape), main chat must remain selected
XDO key Escape
sleep 2.0
CAPTURE d9-008-side-closed

# 8. ENTRY PATH 2: /side reopens it
XDO mousemove 802 789 click 1
sleep 0.5
XDO type --delay 60 "/side"
sleep 1.5
CAPTURE d9-009-slash-row
XDO key ctrl+Return
sleep 2.5
CAPTURE d9-010-side-reopened
echo "scene complete"
