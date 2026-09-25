# Active Execution State

**Updated:** 2026-09-25
**Current main:** 3546201 (the Wave-5 merge — LEASE-001 PR #55 + TAKE-001 PR #56)

This file is the repository's operational handoff for the active Tech Leads. It does not override the architecture constitution or work-order contracts; it records the current execution state and sequencing decisions.

## Current product state

F0 Governance                         ✅
F1 Codex Desktop parity              ✅ CLOSED
F2 Canonical contracts + shell       ✅ CLOSED
Wave 2 / F3-F5 fabric foundation     ✅ DELIVERED
Wave 3 / F6 orchestration            ✅ CLOSED
Wave 4 / F7-F9                       ✅ CLOSED 2026-09-23 (gate record: docs/research/evidence/w4-gate/WAVE4-GATE-RECORD.md)
Wave 5 / F6 depth (leases + takeover) ✅ CLOSED 2026-09-25 (gate record: docs/research/evidence/w5-gate/WAVE5-GATE-RECORD.md)
Wave 6 / F11 Web client              ▶ NEXT (per the sequencing below)
Web / Windows / Linux formal verify  ▶ REQUIRED BEFORE PRODUCTION
Production                           ⬜
macOS                                ⏸ DEFERRED UNTIL AFTER PRODUCTION
Mobile                               ⏸ DEFERRED UNTIL AFTER PRODUCTION

Wave 4 closed at the merged gate: PROV-001 (PR #53), LAB-001 (PR #52),
COL-001 (PR #54) — 3/3 merged, deviations NONE ×3, the 19-step harness
green (the three-fabric scenario), the d26 lab scenes VLM-adjudicated,
CI green both platforms. The next active work is the remaining F6
orchestration depth (leases/conflict handling + human takeover), then
the Web client.

## Release sequencing decision

The first production release is deliberately restricted to the three clients that must be formally verified first:

Wave 4  ✅ CLOSED
  ↓
F7 + F8 + F9 gate  ✅ GREEN 2026-09-23
  ↓
remaining F6 depth needed  ▶ NEXT
  ↓
┌────────────┼────────────┐
↓            ↓            ↓
Linux        Windows      Web
└────────────┼────────────┘
             ↓
FORMAL PRODUCTION VERIFY
             ↓
PRODUCTION
             ↓
┌────────────┴────────────┐
↓                         ↓
macOS                     Mobile
DEFERRED                  DEFERRED

macOS and Mobile are not prerequisites for the first production release. They become post-production expansion clients and receive their own formal verification after production.

## Tech Lead #1 — product/integration authority

TL #1 owns:
- architecture and contract integrity;
- current main;
- Wave-4 integration;
- remaining orchestration depth;
- provider/model/runtime integration;
- Web client;
- production hardening;
- formal release gates for Linux, Windows and Web;
- integration of product fixes discovered by TL #2.

TL #1 must not treat a worker report as phase completion. Closure requires merged implementation plus acceptance evidence.

## Tech Lead #2 — Linux/E2B verification authority

TL #2 owns:
- real Linux GUI dogfooding;
- E2B Desktop provisioning and GUI interaction;
- J-01..J-18 current-main verification;
- Linux-specific regression detection;
- GUI/accessibility/discoverability evidence;
- E2B verification tooling;
- focused product-fix PRs when a reproducible defect belongs in Flauz.

TL #2 must test the exact current main SHA, not a long-lived stale product branch.

Historical Linux-lane branches and evidence remain valuable investigation history, but are not alternative product truth.

## Synchronization law

current main
  ↓
fresh E2B Desktop
  ↓
real GUI interaction
  ↓
defect / verification result
  ↓
focused PR if product fix is required
  ↓
merge to main
  ↓
fresh E2B environment
  ↓
rerun affected journey

A Linux defect is closed only after the fix is merged to main and verified again in a fresh E2B Desktop environment.

## Current Linux verification expectations

Reconcile the historical Linux evidence with current main.

Known historical findings include the GPUI/X11/lavapipe rendering issue (L-002), runtime/environment incompatibilities, silent no-ops, draft loss, raw protocol errors, toast persistence and other UX defects. Each must be reclassified against current main rather than assumed still open or fixed.

The E2B harness must never embed credentials in clone URLs, logs, actions or evidence.

## Shared acceptance law

Every major capability must be:
discoverable
→ usable
→ observable
→ recoverable
→ truthful on failure
→ evidenced

The command palette is a fallback, never the sole discovery mechanism.

Major waves must include the applicable J-01..J-18 journeys and at least one domain-neutral non-software-development scenario.

## Immediate execution order

1. ✅ Merge/gate PROV-001 + LAB-001 + COL-001 (3/3 merged 2026-09-23)
2. ✅ Run the Wave-4 three-fabric integration gate (Gates A/B/C green)
3. ✅ Complete the remaining F6 orchestration depth (Wave 5 CLOSED: LEASE-001 + TAKE-001)
4. ▶ Finish Web client (Wave 6: WEB-001 foundation → WEB-002 capability surfaces)
5. Bring Linux + Windows + Web to formal verification readiness
6. Run formal production verification across all three
7. Close production hardening/release gates
8. Ship production release
9. Start macOS and Mobile expansion

When uncertainty exists, inspect current main, the authoritative roadmap, the active work order, and the latest gate record before continuing. Never use an old chat handoff as the source of truth.
