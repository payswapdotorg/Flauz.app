#!/usr/bin/env bash
# Scene B5a — Import the fabricated Claude session (unauthenticated task-context unlock):
# navigate to Settings > Import, capture detection state.
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
CAPTURE b5a-01-boot
# Palette -> "import" -> Enter (Import settings page)
XDO key ctrl+k; sleep 1.2; XDO type --delay 60 "import"; sleep 1; XDO key Return
sleep 3; CAPTURE b5a-02-import-detect
