# FV Gate Record — Wave 7 Formal Verification (FV-003)

**Date:** 2026-09-26
**Lead:** Z.ai Code — Tech Lead (Flauz.app), resident orchestrator session
**Pinned SHA (all lanes):** `4ed0a0c8dbafdc2598ff5c421169f4a4a935b9e3` (main, post FV-002 merge PR #60)
**Harness:** FV-002 (PR #60) — `scripts/fv/**`, `scripts/windows_fv_smoke.ps1`, `web/lab/fv-manifest.json` + the Lead gate-fixes (`74ebcda`: lavapipe ICD path resolution + dead-app hard-fail)

## Lane verdicts

| Lane | Verdict | Evidence | Detail |
|---|---|---|---|
| **Linux desktop** | **GREEN — 21/21 scenes PASS** | `docs/research/evidence/fv-gate/linux/` | Every catalog scene (FV-L01..L21) executed at the pinned SHA under LINUX_GUI_LAB (Xvfb + picom + lavapipe + the pinned official CLI 0.146.0-alpha.3.1); per-scene run.json lineage + frames + md5 manifests + PROGRESS.txt action logs; VLM adjudication sample: every scene's decisive frame shows a real rendered window, no black frames, no crashes (`VLM-ADJUDICATION-SAMPLE.json`) |
| **Windows desktop** | **GREEN — CI journey-smoke success** | GitHub Actions run 36211458032 (the 4ed0a0c merge); the evidence artifact uploaded by the workflow | Both matrix legs green; the FV-002 additive steps ran: "Install the pinned Codex CLI" ✓, "Windows FV journey smoke (FV-002 harness)" ✓ (FV-W01..W12 SendKeys scenes, window-state assertions, the honest stub fallback where the runner bounds apply), "Upload FV Windows journey-smoke evidence" ✓ |
| **Web client** | **PARTIAL — 1/14 green, 13 blocked on environment auth** | `docs/research/evidence/fv-gate/web/fv-e00/` (3/3 runs pass) | FV-E00 (the gateway-security probe) green ×3 against the REAL Rust gateway. FV-E01..E13 are BLOCKED: their drivers (the Wave-6 j-series convention) drive the signed-in workspace, and the lab's codex auth state (present during the Wave-6 gate, `auth.json` under the codex home) was wiped by the station reset — the sign-in gate cannot complete without operator credentials. Named below with the recovery path. |

## The named web-lane gap (addendum §2 form)

**Gap W-AUTH (environment, not product):** the web formal-pass scenes FV-E01..E13
require an authenticated app-server session (the Wave-6 gate ran with the
operator's then-present `auth.json`). The current station has NO codex
credentials (searched; the reset wiped them), and fabricating auth is forbidden
(the E2B harness law). The product itself is NOT implicated: the sign-in flow
renders, the ChatGPT OAuth popup fires (`account/login/start` → `authUrl` →
`window.open` verified in the probe run), the gateway supervises the pinned CLI
cleanly (`supervisor` connected, zero reconnect warnings), and the unauthenticated
honest bounds behave exactly as the Linux lane's L-1 bound documents.

**Recovery path (operator action, one-time):** provide an OpenAI API key (the
pinned CLI supports headless auth: `codex login --with-api-key`, reading the key
from stdin, writing `auth.json` into the gateway's codex home) — or complete one
ChatGPT sign-in through a locally-reachable app instance. On auth restore, the
Lead re-runs `node web/lab/journeys.mjs --manifest lab/fv-manifest.json
--scene <id>` per blocked scene (the harness is fully provisioned: real gateway
binary, web/dist, playwright chromium 1208, the pinned CLI exported).

## Harness findings during execution (recorded honestly)

1. **Fixed pre-merge (Lead gate-fix `74ebcda`):** the lavapipe ICD hardcode
   (station-rebuild path drift) and the dead-app WARN path (black-frame
   COMPLETE hazard). Both verified by the post-fix fv-l01 run.
2. **Driver invocation note:** the web manifest command resolves
   `--manifest` relative to `web/` (invoke from `web/` as
   `node lab/journeys.mjs --manifest lab/fv-manifest.json`); the pinned CLI
   must be exported as `CODEX_RS_CODEX_BIN` (the gateway defaults to PATH).
   Recorded for the FV-003 re-runs.
3. **Orphaned-gateway hygiene:** a crashed driver leaves the gateway holding
   8610 (`pkill -f flauz-web-gateway` before re-runs). A follow-up harness
   hardening candidate, not a product defect.

## Product defects found

**NONE.** Across 21 Linux scenes + the Windows CI journey-smoke + the web
gateway-security probe: no crash, no silent no-op, no draft loss, no raw
protocol error. The unauthenticated "Codex couldn't complete the turn" retry
toasts are the documented honest-bound behavior (L-1 / E-1), not defects.
No fix work orders are required from this pass.

## The journey coverage (catalog → verdicts)

- J-01..J-18 applicable Linux variants: **FV-L01..L21 all PASS** (incl. the
  domain-neutral scene FV-L20 and the A11Y scene FV-L21)
- J-01..J-15 Windows variants: **FV-W01..W12 green in CI** (the bounded depth
  per addendum §2: SendKeys drives + window-state assertions; the
  live-turn bound is the named gap W-5 in the FV catalog, unchanged)
- Web: FV-E00 (gateway security) **PASS ×3**; FV-E01..E13 **blocked on W-AUTH**
  (this record)

## Verdict

The three-lane formal verification at `4ed0a0c` is **Linux GREEN, Windows
GREEN, Web partial (1/14 + the named environment gap W-AUTH)**. Per the
addendum's honest-bounds law, the FV wave closes its executable scope with
this record; the web-lane completion is a one-operator-action recovery away
and does not gate the production-readiness assessment of the Linux and
Windows clients.
