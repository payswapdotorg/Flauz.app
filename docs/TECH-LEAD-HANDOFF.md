# Flauz Tech Lead Handoff

**Start from:** `main`

## Mission

Continue Flauz using this repository as the sole implementation source of truth.
Do not depend on prior chat history.

Read, in order:

1. `AGENTS.md`
2. `docs/FLAUZ-SOURCE-OF-TRUTH.md`
3. `docs/IMPLEMENTATION-ROADMAP.md`
4. `docs/WORK-ORDER-TEMPLATE.md`
5. `docs/architecture.md`
6. `docs/CONTEXT-HARNESS-ARCHITECTURE.md`
7. `docs/PRODUCT-UX-JOURNEYS.md`
8. `docs/parity-matrix.md`

## Immediate instruction

F1 (Codex Desktop parity) is CLOSED at v0.1.0-rc.14
(`docs/research/evidence/f1-sweep/F1-CLOSURE-RECORD.md`; user review received
2026-09-21). Do not reopen F1 merely because full-reference parity remains
non-green outside the closed release scope — the bounded/platform/proprietary
classifications stand.

Execute F2 — canonical contracts + discoverability shell — as the first
vertical product slice wave:

1. Freeze the shared contract kernel (`docs/F2-CONTRACT-KERNEL.md`) before any
   worker branches.
2. Dispatch Wave 1 in parallel: ARCH-001 (world/resource/evidence contracts),
   ARCH-002 (execution/provider/model/agent contracts), ORCH-001 + UX-001
   (context contracts + discoverable platform shell). Work orders live in
   `docs/research/F2-WORK-ORDERS.md`.
3. Close the F2 gate only when the fake-provider round-trip
   (create task → attach resource/environment/agent → compile context →
   produce artifact → observe → evidence → verify → persist → reload, with
   task identity preserved across a model switch) passes end-to-end with no
   external service.

Every F2+ capability must be implemented as a vertical product slice: visible
primary entry + contextual affordance + search/command-palette fallback +
useful empty state + success/next-step state + keyboard-accessible path +
truthful unavailable/failure state. The command palette is never the only
discovery mechanism.

Do not start Web/Mobile clients, broad remote-provider integrations, or
multi-user collaboration before F2–F6 close. Fake providers/runtimes satisfy
the contracts first; real integrations follow their own phases.

Workflow/Pack development remains frozen.

## Then execute

```
F2 canonical contracts + Context/Resource/Execution/Evidence/Procedure contracts
→ F3 environment/provider fabric
→ F4 model/provider + agent runtimes
→ F5 capability/skill resolver
→ F6 context-aware harness + multi-model/multi-agent orchestration
→ F7 user-owned providers + free-tier routing
→ F8 provider-neutral parity lab
→ F9 collaboration
→ F10 macOS
→ F11 Web
→ F12 Mobile
→ F13 production
```

## Required final architecture

Flauz must support:

- Linux, Windows, macOS, Web and Mobile client adapters;
- desktop clients controlling local and remote environments;
- user-owned provider accounts and quotas;
- E2B plus additional interchangeable execution providers;
- multiple providers/environments concurrently;
- Codex, Copilot, direct-model and future agent runtimes;
- Gemini/Anthropic/OpenAI/local/custom model providers;
- skills with explicit capability requirements;
- capability-gap diagnostics and unlock paths;
- multiple independent models cooperating on one skill;
- multi-agent and multi-environment orchestration;
- free-tier-first routing over user-owned accounts;
- shared workspaces/sessions/presence;
- collaborative documents/sheets;
- collaborative code with isolated worktrees and explicit shared-filesystem mode;
- provider-neutral parity labs.

## Non-negotiables

- Official Codex live state stays behind the official app-server.
- Native desktop remains free of Electron/Tauri/Wry/WebView/Node runtime
  dependencies.
- E2B is not the canonical environment.
- Codex/OpenAI is not the universal model registry.
- Skills remain provider/model neutral.
- Secrets never enter the repository or model context by default.
- Every change has a work-order ID and acceptance evidence.
- Workers do not alter architecture without a repository amendment.

## Completion definition

The roadmap is complete when one logical workspace/session model works across
desktop/web/mobile, compatible models and environments can be composed
dynamically, multiple providers can run concurrently, users can consume their
own provider quotas, skill capability gaps can be unlocked, and multiple people
can collaborate across clients/environments.

Compiling each client independently is not sufficient.
