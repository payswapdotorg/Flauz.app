# Roadmap

The product roadmap is now governed by [docs/FLAUZ-SOURCE-OF-TRUTH.md](docs/FLAUZ-SOURCE-OF-TRUTH.md) and [docs/IMPLEMENTATION-ROADMAP.md](docs/IMPLEMENTATION-ROADMAP.md). This file remains a concise release-facing index.

## Current state

**Current phase: Wave 4 (the parallel wave: F7 BYOP / F8 labs / F9
collaboration).**
F1 (Codex Desktop parity) CLOSED 2026-09-21 at v0.1.0-rc.14 per the closure
record (`docs/research/evidence/f1-sweep/F1-CLOSURE-RECORD.md`; user review
received). F2 (canonical contracts + discoverability shell) CLOSED
2026-09-22 at 9977302 per
`docs/research/evidence/f2-gate/F2-GATE-RECORD.md`. **Wave 2 (the fabric)
DELIVERED 2026-09-22** — 3/3 merged, the 17-step integration harness green
(`docs/research/evidence/w2-gate/WAVE2-GATE-RECORD.md`). **Wave 3 (F6
orchestration: context engine / harness / execution graph) CLOSED
2026-09-23** — 3/3 merged (cd59421 / ef43a0e / 82ae6af + the Lead Gate-B
gate-fix db79499), deviations NONE ×3, the 18-step integration harness
green including the F6 domain-neutral non-software-development scenario;
gate record:
`docs/research/evidence/w3-gate/WAVE3-GATE-RECORD.md`. Wave-4 work orders
(PROV-001 / LAB-001 / COL-001) dispatched per
`docs/research/WAVE4-WORK-ORDERS.md`.

- [x] Native Rust/GPUI Codex-compatible desktop foundation.
- [x] Windows and Linux desktop targets.
- [x] Official Codex app-server supervision boundary.
- [x] Native terminal, Browser, Computer Use, Git, Skills, plugins, MCP Apps, and Marketplace foundations.
- [x] P1/P2 parity work orders completed to the current merged work-order set, including P2-007 through P2-011.
- [x] RWO-020/RWO-021/RWO-022 reference, verification, and adversarial review waves.
- [x] Linux parity lab and official Linux reference+1 lab.
- [x] Release-critical parity scope closed with remaining proprietary/platform and full-reference deltas explicitly bounded.
- [x] Full-reference Codex parity backlog and release-candidate gates closed at
  v0.1.0-rc.14 — F1 CLOSED 2026-09-21 (closure record:
  `docs/research/evidence/f1-sweep/F1-CLOSURE-RECORD.md`).

## Approved platform direction

After parity closure, implement in this order:

1. [x] Canonical Workspace/Session/Task/Environment/Resource/Artifact/Evidence/Procedure/Context/Provider/Model/Agent/Skill/Capability contracts. — **F2 CLOSED 2026-09-22** (flauz-world + flauz-exec + flauz-context + the discoverable shell; frozen `docs/F2-CONTRACT-KERNEL.md`; gate record: `docs/research/evidence/f2-gate/F2-GATE-RECORD.md`).
2. [ ] Context Engine and model-aware Harness contracts: retrieval, tiered memory, compaction/reset, dynamic tool exposure, provenance, telemetry. — **F6 foundation merged 2026-09-23** (ORCH-002: compile-from-durable-state + the rebuild law + tier dynamics + compaction + tool exposure; ORCH-003: the harness state machine with continue/recover/escalate first-class + telemetry + the recovery UX; full retrieval is later waves).
3. [ ] Environment/provider fabric with local and remote execution through one contract, including browser/site/terminal/sandbox topology. — **Wave-2 foundation merged 2026-09-22** (ENV-001: LocalEnvironmentProvider wrapping the four F1 surfaces + FakeRemoteEnvironmentProvider, the fake-consistency remote every real remote provider copies; cross-environment identity preserved).
4. [ ] Model-provider and agent-runtime fabric, including BYOK and non-Codex models. — **Wave-2 foundation merged 2026-09-22** (MOD-001+RT-001: the model/provider registry with SecretRef-only connections, the Codex adapter boundary + the first non-Codex runtime, the model picker; BYOK flows → Wave 4 PROV-001).
5. [ ] Capability resolver, skill dependency checking, and skill-unlock UX. — **Wave-2 foundation merged 2026-09-22** (CAP-001: the five-dimension resolver with named gaps, no silent fall-through, + the gap UX; skill dependency checking/unlock flows are later waves).
6. [ ] Reactive execution graph, multi-model/multi-agent orchestration, verification, resource leases/conflict handling, and human takeover. — **F6 core merged 2026-09-23** (ORCH-004: the attributed graph + the Save-as-a-reusable-workflow slice; ORCH-002/003: the engine + harness behind it; leases/conflict handling and human-takeover UX are later waves).
7. [ ] User-owned provider connections, quotas, cost policies, and free-tier-first routing. — **Wave 4 DISPATCHED 2026-09-23** (PROV-001: the connection flow on fakes, the quota ledger with visible attribution, the routing policy as data).
8. [ ] Provider-neutral parity labs and cross-environment execution validation. — **Wave 4 DISPATCHED 2026-09-23** (LAB-001: journey specs, the normalized evidence schema, the comparator, local + fake-remote adapters).
9. [ ] Collaboration: shared task state, presence, shared sessions/environments, documents/sheets, collaborative code, private-vs-shared context. — **Wave 4 DISPATCHED 2026-09-23** (COL-001: membership/permissions with named denial, presence, private-vs-shared visibility, the two-actor simulation + the members surface; editors and real transport are F10+).
10. [ ] Procedure library and learn → save → discover → run → deviation → improve lifecycle.
11. [ ] macOS client adapter.
12. [ ] Web client adapter.
13. [ ] Mobile client adapter.
14. [ ] Production hardening, signed releases, migrations, observability, recovery, and compatibility guarantees.

## Frozen architectural decisions

- Desktop clients can control local **and** remote environments.
- Web and mobile are clients of the same Workspace/Session model, not separate products.
- Codex app-server is a first-class agent runtime, not the universal Flauz model registry.
- Skills are reusable, model-agnostic artifacts with explicit capability requirements.
- Multiple models/agents may cooperate on one skill.
- E2B is an execution-provider adapter, not the canonical environment.
- GitHub Actions, Codemagic, Azure, E2B, Daytona, Vercel Sandbox, Cloudflare Sandbox, local machines, and future providers are interchangeable behind provider contracts.
- Users may connect their own provider accounts and consume their own free/paid quotas.
- Workflow/Pack development remains frozen and is not the new execution architecture.

See [docs/IMPLEMENTATION-ROADMAP.md](docs/IMPLEMENTATION-ROADMAP.md) for the full dependency graph and work-order IDs.

Feature proposals and architecture changes require a repository-recorded amendment before implementation.

Major product capabilities also require GUI journeys and discoverability evidence;
see `docs/PRODUCT-UX-JOURNEYS.md`.
