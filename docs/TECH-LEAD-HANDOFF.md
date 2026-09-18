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
6. `docs/parity-matrix.md`

## Immediate instruction

Finish the remaining Codex Desktop parity backlog first.

Do not start Web, Mobile, broad remote-provider integrations, or multi-user
collaboration before parity closure. Only narrow scaffolding required to avoid
future architectural coupling is allowed.

Workflow/Pack development remains frozen.

## Then execute

```
F2 canonical contracts
→ F3 environment/provider fabric
→ F4 model/provider + agent runtimes
→ F5 capability/skill resolver
→ F6 multi-model/multi-agent orchestration
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
