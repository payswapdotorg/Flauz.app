#!/usr/bin/env bash
# Scene B3 (Worker B2 rerun) — no-runtime error state: session.sh is invoked with
# CODEX_RS_CODEX_BIN=/bin/false. Captures the app's runtime-failure surface,
# then opportunistically attempts Settings > Import navigation (multi-position
# blind clicks; the Import page content is the remaining uncaptured surface).
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 3
CAPTURE b3-01-error-state
sleep 3
CAPTURE b3-02-error-state-later
XDO mousemove 291 195 click 1; sleep 0.5
XDO type --delay 50 "import"; sleep 1.2; CAPTURE b3-03-settings-search-import
XDO mousemove 291 575 click 1; sleep 2; CAPTURE b3-04-import-attempt-a
XDO mousemove 406 575 click 1; sleep 2; CAPTURE b3-05-import-attempt-b
XDO mousemove 291 335 click 1; sleep 2; CAPTURE b3-06-import-attempt-c
