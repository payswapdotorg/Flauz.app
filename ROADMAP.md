# Roadmap

The product roadmap is now governed by [docs/FLAUZ-SOURCE-OF-TRUTH.md](docs/FLAUZ-SOURCE-OF-TRUTH.md) and [docs/IMPLEMENTATION-ROADMAP.md](docs/IMPLEMENTATION-ROADMAP.md). This file remains a concise release-facing index.

## Current state

- [x] Native Rust/GPUI Codex-compatible desktop foundation.
- [x] Windows and Linux desktop targets.
- [x] Official Codex app-server supervision boundary.
- [x] Native terminal, Browser, Computer Use, Git, Skills, plugins, MCP Apps, and Marketplace foundations.
- [x] P1/P2 parity work orders completed to the current merged work-order set.
- [x] Linux parity lab and official Linux reference+1 lab.
- [ ] Complete the remaining Codex parity backlog and release-candidate gates.

## Approved platform direction

After parity closure, implement in this order:

1. [ ] Canonical Workspace/Session/Environment/Provider/Model/Agent/Skill/Capability contracts.
2. [ ] Environment/provider fabric with local and remote execution through one contract.
3. [ ] Model-provider and agent-runtime fabric, including BYOK and non-Codex models.
4. [ ] Capability resolver, skill dependency checking, and skill-unlock UX.
5. [ ] Multi-model, multi-agent, and multi-environment orchestration.
6. [ ] User-owned provider connections, quotas, cost policies, and free-tier-first routing.
7. [ ] Provider-neutral Windows/macOS/Linux parity labs and execution providers.
8. [ ] Multi-user workspaces, presence, shared sessions, collaborative documents/sheets, and collaborative code.
9. [ ] macOS client adapter.
10. [ ] Web client adapter.
11. [ ] Mobile client adapter.
12. [ ] Production hardening, signed releases, migrations, observability, and compatibility guarantees.

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
