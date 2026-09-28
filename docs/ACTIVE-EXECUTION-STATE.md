# Active Execution State

**Updated:** 2026-09-28  
**Last product-code main baseline:** `5cbfbd838c11` (COMP-001 merged; PR #65)  
**Current HEAD rule:** verify with `git rev-parse HEAD`; subsequent documentation-only reconciliation commits may advance HEAD without changing the product-code baseline.  
**Role:** This file is the current operational state for autonomous execution. It supersedes stale historical handoffs, old chat summaries, and pre-gate roadmap snapshots.

## Current product state

F0 Governance                         ✅  
F1 Codex Desktop parity              ✅ CLOSED  
F2 Canonical contracts + shell       ✅ CLOSED  
Wave 2 / F3-F5 fabric foundation     ✅ DELIVERED  
Wave 3 / F6 orchestration            ✅ CLOSED  
Wave 4 / F7-F9                       ✅ CLOSED 2026-09-23  
Wave 5 / F6 depth                    ✅ CLOSED 2026-09-25  
Wave 6 / F11 Web client              ✅ CLOSED 2026-09-25  
Wave 7 / formal verification         ✅ EXECUTED 2026-09-26, Web remains W-AUTH-bound  
Wave 8 / production hardening        ▶ ACTIVE  
Wave 8b / production follow-through  ▶ ACTIVE  
Production release                   ⬜  
macOS                                ⏸ DEFERRED UNTIL AFTER PRODUCTION  
Mobile                               ⏸ DEFERRED UNTIL AFTER PRODUCTION  

## Verified current-main facts

- Current `main` is `5cbfbd838c11`.
- REL-001 is merged (PR #62).
- SEC-001 is merged (PR #63).
- MIG-002 is merged (PR #64).
- COMP-001 is merged (PR #65): `docs/PROTOCOL-COMPATIBILITY.md` is the compat law; the initialize-with-capabilities freeze test is green on main.
- OBS-001 is not present as a merged mainline change yet.
- No merged UPD-001 branch is present.
- Wave-7 gate record at `4ed0a0c` records:
  - Linux: 21/21 GREEN.
  - Windows: journey-smoke GREEN within CI bounds.
  - Web: 1/14 GREEN; E01-E13 blocked by named environment gap W-AUTH.
- Any production-gate decision must account for changes made after the FV pin. Do not silently treat the older FV pin as equivalent to current main; rerun affected evidence when a change can invalidate it.

## Production sequence

```
Wave 7 formal verification
  ↓
Wave 8 production hardening
  ↓
Wave 8b follow-through
  ↓
recover W-AUTH
  ↓
complete authenticated Web FV scenes
  ↓
reconcile current-main verification after post-FV changes
  ↓
production release audit
  ↓
production release
  ↓
post-production macOS
  ↓
post-production Mobile
```

## Active work orders

### OBS-001 — observability
Expected output:
- credential-scrubbed diagnostics export;
- palette + CLI path;
- allowlist-based export;
- provider-outage behavior record;
- unit tests and station evidence.

Acceptance:
- no secret leakage;
- named fields only;
- palette and CLI both work;
- outage states documented against evidence;
- CI green.

### UPD-001 — update check
Expected output:
- notify-only release check;
- once-per-day cadence;
- settings switch;
- truthful update / current / failed states;
- tests.

Acceptance:
- never auto-download/install;
- offline failure is truthful;
- disabled setting prevents fetch;
- CI green.

### WEB-REL — web release channel
Blocked until W-AUTH is recovered and the authenticated web formal-pass scenes are rerun.

## Autonomous dispatch rules

1. Read this file plus the relevant work-order file before acting.
2. Confirm `git rev-parse HEAD` equals the repository's current `main` before beginning a new work order.
3. Never infer completion from a worker message alone.
4. Closure requires:
   - implementation merged to `main`;
   - work-order acceptance tests green;
   - required evidence archived;
   - current-state file updated with the new SHA.
5. Pairwise-disjoint work orders may run concurrently, subject to actual worker/platform capacity.
6. A blocked platform operation must not cause speculative repository changes.
7. Keep all secrets out of commits, URLs, logs, reports, and evidence.
8. When a work order discovers a product defect, create a focused fix order rather than silently changing scope.
9. After a product-source change that can affect a formal/production-verification surface:
   - identify the affected evidence;
   - rerun it against the new main SHA;
   - update the gate record.
10. When no active work order remains, stop dispatching and produce a release-readiness reconciliation rather than inventing another wave.

## Worker model

Use up to three concurrent workers where the work is genuinely disjoint:

- Worker A: production/release/pipeline track.
- Worker B: diagnostics/observability/product support track.
- Worker C: security/compatibility/release-audit track.

The Lead owns:
- work-order selection;
- source-of-truth validation;
- integration;
- merge/gate decisions;
- production readiness;
- current-state updates.

## External station/platform blockers

These are not repository facts and must be rechecked at execution time:
- worker capacity/service availability;
- W-AUTH credentials/session;
- signing-key availability;
- deployed release feed/infrastructure;
- production deployment credentials.

When one of these is unavailable, record the named gap and recovery action in the repo; do not fabricate success.

## Stop conditions

Stop autonomous execution when:
- all active production work orders are merged and gated;
- W-AUTH is recovered and authenticated Web FV completes;
- current-main production verification has been reconciled;
- production release checklist is green or has named operator-owned gaps;
- macOS/Mobile are still intentionally deferred.

At that point TL #1 becomes a maintenance/release-review role rather than a continuously running implementation lead.
