#!/usr/bin/env bash
# Scene B3 — no-runtime error state (CODEX_RS_CODEX_BIN=/bin/false passed to session.sh)
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 3
CAPTURE b3-01-error-state
sleep 3
CAPTURE b3-02-error-state-later
