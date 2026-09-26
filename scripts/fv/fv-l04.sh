#!/bin/bash
# FV-L04 — J-03 the reconnect/retry cadence (kill the supervised
# app-server child; Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   The footer retry cadence lives in render_sidebar_footer —
#   crates/codex-app/src/ui.rs:16265-16317:
#     - ConnectionStatus::Recovering, attempt 0 → "Reconnecting…"
#       (ui.rs:16288)
#     - attempt N with retry_in_ms → format!("Retry {attempt} in {}s", …)
#       (ui.rs:16289-16291) — "Retry 1 in 1s" … "Retry N in 20s" (the
#       cap holds; the backoff schedule is app-server-owned)
#     - ConnectionStatus::Online → "App-server online" (ui.rs:16277)
#     - a NEW loss starts at attempt 1 again (the Recovering state resets)
#   The supervised app-server child is spawned with CODEX_RS_CODEX_BIN
#   (the sealed recipe) and is found as a child of the app pid matching
#   "runtime/codex" — the rg-a11y-01.sh:116 precedent
#   (pgrep -P $APP_PID -f "runtime/codex").
#
# Moments (the catalog FV-L04 row):
#   01 baseline footer (App-server online)
#   02 kill the supervised app-server child mid-session
#   03 footer retry cadence frames: "Reconnecting…" → "Retry 1 in 1s" →
#      successive attempts up to the cap ("Retry N in 20s")
#   04 let a retry succeed → auto-return online ("App-server online")
#   05 kill again → the timer restarts at attempt 1
#   06 log extract archived (app.log tail — bounded, 16 KiB)
#
# PASS (Lead-adjudicated, frames + log):
#   - the cadence frames show each named state;
#   - the cap holds ("Retry N in 20s" does not exceed 20s);
#   - "App-server online" after a successful retry;
#   - the NEXT loss starts at attempt 1 / 1 s.
#   Run notes: the seeded-session bound is named (G-6): the retry cadence
#   is observable without an authenticated session; the supervised child
#   respawns from the pinned isolated CODEX_RS_CODEX_BIN.
#
# Usage: scripts/fv/fv-l04.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l04 "J-03" "${1:?binary path required}" "${2:-}"

moment 01 "baseline footer (App-server online)"
sleep 6
cap fvl04-01-online
say "footer baseline captured (expect 'App-server online' — ui.rs:16277)"

moment 02 "kill the supervised app-server child mid-session"
CPID=""
for i in $(seq 1 20); do
  CPID="$(pgrep -P "$FV_APP_PID" -f 'runtime/codex' | head -1)"
  [ -n "$CPID" ] && break
  sleep 2
done
if [ -z "$CPID" ]; then
  say "WARN: supervised app-server child not found by pgrep -P $FV_APP_PID -f runtime/codex — trying any child codex process"
  CPID="$(pgrep -P "$FV_APP_PID" | head -1)"
fi
[ -n "$CPID" ] || fv_fail "no supervised app-server child found under pid $FV_APP_PID after 40s — the sealed recipe expects CODEX_RS_CODEX_BIN=$FV_RUNTIME to spawn a child; check $FV_OUT/app.log"
say "supervised child pid: $CPID"
kill "$CPID" 2>/dev/null || true
say "child killed"

moment 03 "footer retry cadence frames (Reconnecting… → Retry 1 in 1s → … cap)"
sleep 1.5
cap fvl04-03a-reconnecting
sleep 2
cap fvl04-03b-retry-1
sleep 4
cap fvl04-03c-retry-2
sleep 8
cap fvl04-03d-retry-n
sleep 12
cap fvl04-03e-cap-holds

moment 04 "let a retry succeed — auto-return online"
say "waiting for the retry to succeed (the app respawns the supervised child from the pinned runtime)"
ONLINE_SEEN=0
for i in $(seq 1 24); do
  NEWCPID="$(pgrep -P "$FV_APP_PID" -f 'runtime/codex' | head -1)"
  if [ -n "$NEWCPID" ] && [ "$NEWCPID" != "$CPID" ]; then
    say "respawned child pid: $NEWCPID (after ~$((i*5))s)"
    ONLINE_SEEN=1
    break
  fi
  sleep 5
done
[ "$ONLINE_SEEN" = "1" ] || say "WARN: no respawned child observed in 120s — capturing the honest footer state anyway (bounded; the Lead adjudicates)"
sleep 8
cap fvl04-04-online-again

moment 05 "kill again — the timer restarts at attempt 1"
CPID2="$(pgrep -P "$FV_APP_PID" -f 'runtime/codex' | head -1)"
if [ -n "$CPID2" ]; then
  kill "$CPID2" 2>/dev/null || true
  say "second kill: child pid $CPID2"
  sleep 2
  cap fvl04-05a-reconnecting-again
  sleep 3
  cap fvl04-05b-retry-1-again
else
  say "WARN: no supervised child for the second kill — the first-loss frames carry the cadence truth (honest bound named in run notes)"
  cap fvl04-05-no-child
fi

moment 06 "log extract archived (bounded app.log tail)"
tail -c 16384 "$FV_OUT/app.log" > "$FV_OUT/app-tail.log" 2>/dev/null || true
say "app.log tail (16 KiB bound) archived as app-tail.log"

fv_end
