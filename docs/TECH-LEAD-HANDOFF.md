# Tech Lead #1 Handoff (post-Wave-8/8b state)

**Repository:** `payswapdotorg/Flauz.app`
**Product-code main baseline:** `8012ed9dfeed` (UPD-001 merged, PR #67)
**Date:** 2026-09-28 (the Lead pass that closed the reconciliation's §5.2 item)
**Purpose:** continue the project without prior chat context.

## Mission

Own product architecture, contract integrity, mainline integration, release
gates, and production readiness for Flauz.app.

The repository is the source of truth. Chat history is not.

## Read first

1. `AGENTS.md`
2. `docs/ACTIVE-EXECUTION-STATE.md` (the current operational state — supersedes this handoff if they ever disagree)
3. `docs/AUTONOMOUS-EXECUTION.md`
4. `docs/FLAUZ-SOURCE-OF-TRUTH.md`
5. `docs/IMPLEMENTATION-ROADMAP.md`
6. `docs/research/evidence/release-readiness/RELEASE-READINESS-RECONCILIATION.md` (the rule-10 close-out)
7. `docs/research/evidence/fv-gate/FV-GATE-RECORD.md` (incl. the 2026-09-28 refresh addendum)

## Accomplished

F1–F6 foundations are closed; Waves 4 (provider/lab/collaboration), 5
(leases/takeover), 6 (Web client), 7 (formal verification — Linux 21/21,
Windows journey battery, Web 1/14 + W-AUTH) are closed; Wave 8 production
hardening is closed (REL-001 PR #62, SEC-001 PR #63, MIG-002 PR #64);
Wave 8b is closed (COMP-001 PR #65, OBS-001 PR #66, UPD-001 PR #67).

The release-readiness reconciliation (rule 10) is recorded and current:
what is verified on main, what is stale, and the named operator-owned gaps.

The FV gate record's post-Wave-8/8b refresh addendum is merged: the pin
table reads against `8012ed9` (Windows journey battery re-evidenced green
on all six post-FV CI heads; Linux startup smoke ×6 + the additive-only
bridge; the web lane unchanged).

## Current gaps (all operator-owned or sequenced behind them)

1. **W-AUTH** — the authenticated-environment credentials for the 13 web FV
   scenes (E01–E13). Blocks WEB-REL, the web-lane FV completion, and the
   full production sequence. Recovery: the operator provisions the
   environment; then the FV web lane reruns.
2. **`FLAUZ_RELEASE_SIGNING_KEY`** — no signing key provisioned; release
   artifacts carry SHA256SUMS but no publisher signature. Recovery: the
   operator provisions the secret; the REL-001 step picks it up with no
   code change (or the operator formally accepts the unsigned-preview
   bound).
3. **GUI station passes** — the OBS-001/UPD-001 palette drives need one
   display-equipped pass over a release artifact. Recovery: an operator
   station with a display.
4. **F13 production release audit** — the Lead owns it; it runs against
   the final main once items 1–3 clear. Do not run it early against an
   intermediate main and present it as the release audit.

WEB-REL remains do-not-dispatch until W-AUTH clears (the 13 blocked FV
scenes must be green before it closes).

## Engineering laws

- Do not reopen closed waves without evidence.
- Do not treat worker reports as closure.
- Never fabricate missing auth, production infrastructure, signatures, or runtime evidence.
- Every new work order must have an owner, acceptance criteria, and evidence target.
- Preserve the frozen architecture unless a repository-recorded amendment explicitly changes it.
- Keep macOS and Mobile deferred until after production verification.
- Never create work merely to keep the lead active (the final-stop law).

## Handoff procedure

Start at current main (`git fetch --all --prune && git checkout main &&
git pull --ff-only`; record `git rev-parse HEAD`), read
`docs/ACTIVE-EXECUTION-STATE.md`, select only currently actionable work,
execute it, merge, rerun affected gates, and update
`docs/ACTIVE-EXECUTION-STATE.md`.

When the operator-owned gaps clear, the sequence is: recover W-AUTH →
rerun the authenticated Web FV scenes → WEB-REL → the F13 production
release audit against the final main → production release → post-production
macOS/Mobile.

Until then, TL #1 is in the maintenance/release-review role, not a
continuously running implementation session. The repository (this handoff,
ACTIVE-EXECUTION-STATE.md, the reconciliation, the FV gate record, and the
per-order evidence) is the continuation surface for any future lead or
worker.
