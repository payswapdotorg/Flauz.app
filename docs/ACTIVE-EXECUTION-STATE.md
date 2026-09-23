# Active Execution State

**Updated:** 2026-09-23
**Current main:** 83796f59805948a503760d03c5d06d8d872fd758

This file is the repository's operational handoff for the active Tech Leads. It does not override the architecture constitution or work-order contracts; it records the current execution state and sequencing decisions.

## Current product state

F0 Governance                         ✅
F1 Codex Desktop parity              ✅ CLOSED
F2 Canonical contracts + shell       ✅ CLOSED
Wave 2 / F3-F5 fabric foundation     ✅ DELIVERED
Wave 3 / F6 orchestration            ✅ CLOSED
Wave 4 / F7-F9                       ▶ ACTIVE
Web / Windows / Linux formal verify  ▶ REQUIRED BEFORE PRODUCTION
Production                           ⬜
macOS                                ⏸ DEFERRED UNTIL AFTER PRODUCTION
Mobile                               ⏸ DEFERRED UNTIL AFTER PRODUCTION

Wave 3 is closed at the merged gate. The current active work orders are:
- PROV-001 — user-owned provider accounts, quota attribution, routing policy and free-tier-first scheduling.
- LAB-001 — provider-neutral journey/evidence/comparator fabric.
- COL-001 — collaboration membership, permissions, presence, private/shared visibility and two-actor simulation.

## Release sequencing decision

The first production release is deliberately restricted to the three clients that must be formally verified first:

Wave 4
  ↓
F7 + F8 + F9 gate
  ↓
remaining F6 depth needed
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

1. Merge/gate PROV-001 + LAB-001 + COL-001
2. Run the Wave-4 three-fabric integration gate
3. Complete the remaining F6 orchestration depth required for the production scenario
4. Finish Web client
5. Bring Linux + Windows + Web to formal verification readiness
6. Run formal production verification across all three
7. Close production hardening/release gates
8. Ship production release
9. Start macOS and Mobile expansion

When uncertainty exists, inspect current main, the authoritative roadmap, the active work order, and the latest gate record before continuing. Never use an old chat handoff as the source of truth.
