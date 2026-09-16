#!/usr/bin/env bash
# Scene B4b (Worker B2 rerun) — after restart: verify state persistence
# (window placement 1100x900 + last route), then reach Settings > Import via the
# settings nav search box (correct nav geometry: column screen x ~161..426,
# search input center ~(291,195), first nav row ~(291,295)).
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 3
CAPTURE b4b-01-restarted-state
XDO mousemove 291 195 click 1; sleep 0.5
XDO type --delay 50 "import"; sleep 1.2; CAPTURE b4b-02-settings-search-import
XDO mousemove 291 295 click 1; sleep 2.5; CAPTURE b4b-03-import-page
XDO mousemove 291 135 click 1; sleep 2; CAPTURE b4b-04-back-to-app
