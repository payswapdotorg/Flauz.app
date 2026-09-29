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

---

# Refresh Addendum — the post-Wave-8/8b pin reconciliation (Lead pass)

**Date:** 2026-09-28
**Lead:** TL #1 (the Lead pass named by the release-readiness
reconciliation §3/§5.2)
**Product-code baseline refreshed to:** `8012ed9dfeed` (the UPD-001 merge,
PR #67 — the last product-code change on main)
**Docs head at authoring:** `6cfe0e0` (documentation-only commits may
advance HEAD past the product baseline without a new pin)

## Why this refresh exists

The Wave-8/8b production-hardening set landed six product-source work
orders AFTER the FV pin `4ed0a0c`. Per dispatch rule 9 and the
reconciliation §3, the affected formal-verification evidence was rerun
against every post-FV merge head by CI, and this record's pin table is now
refreshed to cite that evidence. No new FV scene execution was needed: the
change set is additive-only (audited below), and the rerun-at-head evidence
is the CI-embedded harness.

## The post-FV change set (the complete product delta `4ed0a0c..8012ed9`)

| Order | PR | Merge | CI head (evidence) | Actions run |
| --- | --- | --- | --- | --- |
| REL-001 signed-release scaffold | #62 | `354449c` | `f0b459c` | 36219462920 |
| SEC-001 production security review | #63 | `7b434ca` | `a05f40e` | 36219474671 |
| MIG-002 migration guarantee | #64 | `fb5c862` | `5f81eb3` | 36258757061 |
| COMP-001 protocol compatibility | #65 | `5cbfbd8` | `9553de6` | 36384231078 |
| OBS-001 diagnostics export | #66 | `84b9c64` | `48bf288` | 36396045909 |
| UPD-001 update check | #67 | `8012ed9` | `1cd4684` | 36399789011 |

Product surface touched (11 files, +3642/−19): the release workflow + the
verify script (REL-001); the security review docs (SEC-001, docs-only in
product terms); the migration fixtures/tests (MIG-002, test-only); the
protocol version-field semantics + the freeze test (COMP-001); the
diagnostics collector + CLI export flag (OBS-001); the update-check module,
palette/footer/settings affordances (UPD-001).

## The rerun-at-head evidence (verified via the GitHub check-runs/jobs API on 2026-09-28)

Every one of the six CI heads above shows BOTH matrix legs
(`ubuntu-24.04`, `windows-latest`) `success`, with the full battery green
(fmt, clippy `--locked --workspace --all-targets -D warnings`, the
workspace test suite, the release build) and:

- **"Windows FV journey smoke (FV-002 harness)" — `success` on all six
  windows legs.** This CI step IS the FV-W01..W12 journey battery (per the
  Wave-7 record above), so the Windows lane is fully re-evidenced at
  `8012ed9`.
- **"Linux desktop startup smoke" — `success` on all six ubuntu legs.**
  The startup smoke (real app launch under the pinned CLI) is the
  CI-embeddable Linux rerun; the full 21-scene LINUX_GUI_LAB battery's
  execution evidence remains pinned at `4ed0a0c`, bridged to current main
  by the additive-only audit below (no verified surface altered, so the
  scene-level verdicts transfer).
- Zero new failures anywhere in the six runs: every step on both legs
  ended `success` or the leg-appropriate `skipped`.

## The additive-surface audit (Lead judgment, recorded in the reconciliation §3)

- Palette commands: 93 → 95 (`Export diagnostics` [OBS-001], `Check for
  updates` [UPD-001]); the `PaletteCommand` enum carries 95 variants on
  current main (verified by count at the `8012ed9` tree).
- New UI surfaces: the footer update-status row and the updates settings
  card (UPD-001); no existing row, card, or command altered.
- Backend: two new commands (diagnostics export, update check) +
  read-only storage preference accessors; `codex-protocol`'s change is the
  version-field semantics with the frozen-surface table ENFORCED by the
  COMP-001 freeze tests (additive by construction).
- No FV scene asserts the palette command count or the footer row set;
  the in-app palette registry/coverage tests were updated with the changes
  and are green in the workspace battery at `8012ed9` (357 codex-app
  tests, per the reconciliation §2).

## Refreshed pin table

| Lane | Verdict | Pinned SHA | Basis |
|---|---|---|---|
| **Linux desktop** | **GREEN** — 21/21 scenes at the original pin; startup smoke re-evidenced at every post-FV head | scenes `4ed0a0c` → bridged to `8012ed9` (additive-only audit; startup smoke green ×6) | original run.json lineage + the six CI runs |
| **Windows desktop** | **GREEN** — the FV-002 journey battery rerun green at every post-FV head | `4ed0a0c` → `8012ed9` (the CI step is the harness; green ×6) | Actions runs listed above |
| **Web client** | **PARTIAL — 1/14 green; FV-E01..E13 blocked on W-AUTH** | unchanged (`4ed0a0c` execution; the environment gap is not a code state) | the named gap below stands |

## What this refresh does NOT claim

- No new FV scene execution occurred in this pass (none was required — the
  change set is additive-only and the CI-embedded harness reran green).
- The GUI station passes (OBS-001 palette-export drive, UPD-001
  palette/footer drive) remain the operator-owned gap named in the
  reconciliation §4.3; CI smokes + the unit suites cover the code paths.
- W-AUTH remains unrecovered; WEB-REL remains do-not-dispatch; the F13
  production release audit remains sequenced behind the operator-owned
  gaps per the reconciliation §5.

## Refresh verdict

The formal-verification gate now reads against `8012ed9`: **Linux GREEN
(original 21/21 + additive-only bridge + startup-smoke re-evidence),
Windows GREEN (journey battery re-evidenced ×6 at head), Web PARTIAL
(1/14 + W-AUTH, unchanged)**. This closes the reconciliation's §5.2 item
(the FV gate record refresh); the remaining release-readiness items are
the three operator-owned gaps of reconciliation §4.
