# Flauz Release Roadmap

**Current operational state:** [docs/ACTIVE-EXECUTION-STATE.md](docs/ACTIVE-EXECUTION-STATE.md)  
**Autonomous execution contract:** [docs/AUTONOMOUS-EXECUTION.md](docs/AUTONOMOUS-EXECUTION.md)  
**Frozen architecture/dependency plan:** [docs/IMPLEMENTATION-ROADMAP.md](docs/IMPLEMENTATION-ROADMAP.md)

> The implementation roadmap is a frozen historical/dependency plan. Current phase status, ordering decisions, blockers, and next actions are governed by ACTIVE-EXECUTION-STATE.md.

## Current release position — 2026-09-27

```
F0 Governance                         ✅
F1 Codex parity                       ✅
F2 Canonical contracts + shell        ✅
Wave 2 / F3-F5 fabric                 ✅
Wave 3 / F6 orchestration             ✅
Wave 4 / F7-F9                         ✅
Wave 5 / F6 depth                      ✅
Wave 6 / Web                           ✅
Wave 7 / formal verification          ✅ executed; Web W-AUTH gap remains
Wave 8 / production hardening         ▶ active
Wave 8b / release follow-through      ▶ active
Production release                   ⬜
macOS / Mobile                       ⏸ after production
```

## Active production path

```
OBS-001
   ├──> merge + gate
   │
COMP-001 ──> merge + protocol gate
   │
UPD-001 ──> merge + cadence/offline gate
   │
W-AUTH ──> authenticated Web FV scenes
   │
current-main verification reconciliation
   │
production readiness audit
   │
production release
```

The exact order may vary only where work orders are pairwise-disjoint and their dependencies permit parallel execution.

## Release gates

A release is not complete until:
- active production work orders are merged and accepted;
- authenticated Web formal scenes are rerun;
- post-FV changes have had affected verification reconciled against current main;
- release, migration, security, observability, compatibility, and recovery evidence is current;
- all remaining external/operator dependencies are explicitly named.

## Platform scope

First production clients:
- Linux
- Windows
- Web

macOS and Mobile remain deferred until after production verification.

## Historical roadmap

Detailed dependency history remains in [docs/IMPLEMENTATION-ROADMAP.md](docs/IMPLEMENTATION-ROADMAP.md). It must not be used as the current execution queue when it conflicts with ACTIVE-EXECUTION-STATE.md.
