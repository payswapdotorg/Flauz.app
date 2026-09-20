# Flauz Master Implementation Roadmap

**Status:** FROZEN PLAN — base 2026-09-18; approved context/harness/UX extension 2026-09-19  
**Execution:** Tech Lead + up to 3 concurrent workers

The frozen architecture is in
[FLAUZ-SOURCE-OF-TRUTH.md](FLAUZ-SOURCE-OF-TRUTH.md). The approved context/harness
extension is in [CONTEXT-HARNESS-ARCHITECTURE.md](CONTEXT-HARNESS-ARCHITECTURE.md).
Required product journeys and GUI discoverability are in
[PRODUCT-UX-JOURNEYS.md](PRODUCT-UX-JOURNEYS.md). Codex-specific parity is in
[parity-matrix.md](parity-matrix.md).

## Graph

```
F0 Governance
  ↓
F1 Codex parity
  ↓
F2 Canonical contracts
  ├──────────────┬──────────────┐
  ↓              ↓              ↓
F3 Environments F4 Models      F5 Capabilities
  └──────────────┼──────────────┘
                 ↓
             F6 Orchestration
                 ↓
       ┌─────────┼─────────┐
       ↓         ↓         ↓
     F7 BYOP    F8 Labs   F9 Collaboration
       └─────────┼─────────┘
                 ↓
         F10 macOS client
                 ↓
          F11 Web client
                 ↓
        F12 Mobile client
                 ↓
          F13 Production
```

## F0 — Governance

- ✅ source-of-truth constitution
- ✅ master roadmap
- ✅ work-order protocol
- ✅ Tech Lead handoff
- ✅ current native architecture contract
- ✅ Codex parity matrix

## F1 — Codex Desktop parity

Already merged:
- ✅ Terminal discoverability
- ✅ Browser discoverability
- ✅ Multi-folder projects
- ✅ Settings command-palette coverage
- ✅ stable slash-command subset
- ✅ side chats
- ✅ Linux Computer Use platform-bound contract
- ✅ Linux/official-Linux parity lab evidence
- ✅ WO-P2-007 Ctrl+P file-search silent-no-op fix
- ✅ WO-P2-008 per-chat unread-attention state and attention bindings
- ✅ WO-P2-009 persistent browsing history / revisit / Settings management
- ✅ WO-P2-010 evidenced command-palette rows and availability guards
- ✅ WO-P2-011 context-scoped browser reload/force-reload/copy-URL chords
- ✅ WO-P2-012 guard honesty (six evidenced silent no-op states + executor fall-through)
- ✅ WO-P2-013 Activity view surface (J-17)
- ✅ WO-P2-018 visible-entry + label parity (title-bar palette entry, Activity bell, archived-deletion label, unread-dot tooltip)
- ✅ WO-P2-020 F-A4 honest status for Ctrl+P without a workspace
- ✅ RWO-020/RWO-021/RWO-022 reference/review/adversarial verification waves
- ✅ Release-critical parity scope is green for the selected public/runtime-owned rows;
  bounded proprietary/platform rows remain explicitly classified.

Remaining:
- ⬜ WO-P2-017 palette/overlay close focus restoration (r1 rejected — deterministic
  palette-close panic, D17 evidence; fix round in flight, rebases on the merged head)
- ⬜ WO-P2-019 confirmation-modal focus-trap completion (worker in flight)
- ⬜ remaining full-reference P2/P3 parity differences
- ⬜ final parity journeys across the remaining partial/platform rows
- ⬜ accessibility/keyboard-only/reduced-motion/screen-reader/contrast pass
- ⬜ release-candidate soak/reconnect/performance gates

**Gate:** every remaining full-reference difference is explicitly classified;
release-critical journeys remain green; and the final parity/accessibility/release
journeys pass.

## F2 — Canonical contracts

- ⬜ Workspace
- ⬜ Session
- ⬜ Task
- ⬜ Environment
- ⬜ ExecutionProvider
- ⬜ ModelProvider
- ⬜ Model
- ⬜ AgentRuntime
- ⬜ Agent
- ⬜ Skill
- ⬜ Capability
- ⬜ Artifact
- ⬜ ProviderConnection
- ⬜ Context
- ⬜ MemoryItem / durable task memory
- ⬜ Resource
- ⬜ ResourceState
- ⬜ ExecutionGraph / ExecutionNode
- ⬜ Event / Observation / Claim / Evidence
- ⬜ ResourceLease
- ⬜ Procedure / ProcedureVersion
- ⬜ ContextSnapshot / ContextProvenance
- ⬜ ModelContextProfile
- ⬜ versioned session/event transport
- ⬜ conformance fixtures

**Gate:** fake model/runtime/environment/provider implementations pass the
contracts without real external services; context, resource, execution,
evidence and procedure fixtures round-trip without external services.

## F3 — Environment fabric

- ⬜ LocalEnvironmentProvider wrapping current terminal/browser/Computer Use/Git
- ⬜ environment lifecycle
- ⬜ capability advertisement
- ⬜ remote attach/detach
- ⬜ cancellation/heartbeat/reconnect
- ⬜ artifact transfer
- ⬜ provider-neutral environment UI

**Gate:** desktop uses local and fake-remote environments through one contract.

## F4 — Model/runtime fabric

- ⬜ Flauz model registry
- ⬜ model-provider registry
- ⬜ model discovery/capability metadata
- ⬜ secure credential references
- ⬜ Codex runtime adapter
- ⬜ OpenAI API
- ⬜ Gemini
- ⬜ Anthropic
- ⬜ GitHub Copilot SDK/runtime
- ⬜ Ollama
- ⬜ LM Studio
- ⬜ OpenAI-compatible custom endpoint
- ⬜ custom agent-runtime adapter

**Gate:** a non-Codex model/provider can run a bounded test task without changing
the Codex app-server integration.

## F5 — Capability/skill fabric

- ⬜ capability registry
- ⬜ skill requirement schema
- ⬜ model/environment discovery
- ⬜ policy/permission filtering
- ⬜ resolver
- ⬜ skill dependency validation
- ⬜ capability-gap diagnostics
- ⬜ skill-unlock actions
- ⬜ Computer Use compound capability model

**Gate:** an unavailable skill explains the missing capability and offers a valid
unlock route when one exists.

## F6 — Orchestration

- ⬜ reactive execution graph
- ⬜ context compilation/retrieval
- ⬜ tiered memory
- ⬜ structured compaction/reset
- ⬜ dynamic tool discovery/schema loading
- ⬜ model-aware context compilation
- ⬜ agent roles
- ⬜ serial delegation
- ⬜ parallel agents
- ⬜ model-as-tool
- ⬜ resource-aware routing
- ⬜ leases/conflict resolution
- ⬜ shared artifacts/context
- ⬜ independent evaluators/verifiers
- ⬜ claims/observations/evidence
- ⬜ fallback/rerouting
- ⬜ cancellation/dependency propagation
- ⬜ human takeover/approval/handoff
- ⬜ execution provenance
- ⬜ reusable Procedure capture/execution/improvement
- ⬜ harness telemetry and replay

**Gate:** two independent model/runtime adapters cooperate on one task across
multiple execution surfaces and produce attributable, independently verifiable
results without a shared giant transcript.

## F7 — User-owned providers / free-tier routing

- ⬜ provider connection UI
- ⬜ OAuth/API-key flows
- ⬜ secure credential vault/references
- ⬜ multiple accounts/provider
- ⬜ quota/usage
- ⬜ free-tier metadata
- ⬜ user routing policies
- ⬜ free-tier-first scheduling
- ⬜ concurrency/session/spend limits

Initial execution providers:
- ⬜ E2B
- ⬜ Daytona
- ⬜ Azure
- ⬜ GitHub Actions
- ⬜ Codemagic
- ⬜ Vercel Sandbox
- ⬜ Cloudflare Sandbox

**Gate:** a user can consume their own provider quota without requiring a
Flauz-owned global account.

## F8 — Provider-neutral parity lab

- ✅ local Linux GUI lab foundation
- ⬜ GitHub Windows
- ⬜ Azure Windows GUI
- ⬜ GitHub macOS
- ⬜ Codemagic macOS
- ⬜ E2B
- ⬜ Daytona
- ⬜ evidence schema
- ⬜ screenshots/action/accessibility evidence
- ⬜ reference/candidate comparator
- ⬜ provider-independent journeys

**Gate:** one journey runs unchanged against at least two providers.

## F9 — Collaboration

- ⬜ Workspace membership/permissions
- ⬜ presence
- ⬜ shared sessions
- ⬜ shared environments
- ⬜ collaborative documents
- ⬜ collaborative sheets
- ⬜ collaborative code
- ⬜ isolated worktrees
- ⬜ explicit shared-filesystem mode
- ⬜ activity/history/comments

**Gate:** two independent clients can concurrently use one workspace/session
without losing authorized state.

## F10 — macOS client

- ⬜ native client adapter
- ⬜ local Mac environment
- ⬜ local Computer Use/terminal/browser/Git
- ⬜ Codex runtime
- ⬜ remote environment selection
- ⬜ parity journeys
- ⬜ packaging

## F11 — Web client

- ⬜ Web shell
- ⬜ authenticated workspace/session connection
- ⬜ remote environment control
- ⬜ model/provider selection
- ⬜ skill-unlock UI
- ⬜ collaboration/presence
- ⬜ artifacts/documents/sheets
- ⬜ accessibility/responsive pass

## F12 — Mobile client

- ⬜ mobile shell
- ⬜ workspace/session
- ⬜ remote control
- ⬜ approvals/notifications
- ⬜ handoff/resume
- ⬜ collaboration/presence
- ⬜ appropriate model/provider/skill controls
- ⬜ mobile security/storage boundary

## F13 — Production

- ⬜ signed desktop releases
- ⬜ Linux distribution strategy
- ⬜ web/mobile release channels
- ⬜ automatic updates
- ⬜ migrations
- ⬜ observability
- ⬜ recovery
- ⬜ security review
- ⬜ provider outage behavior
- ⬜ protocol compatibility guarantees


## Cross-cutting GUI/UX discoverability gate

Every major capability introduced from F2 onward must map to one or more
journeys in [PRODUCT-UX-JOURNEYS.md](PRODUCT-UX-JOURNEYS.md).

A work order is not complete until its GUI/lab evidence demonstrates, where
applicable:

- a visible primary entry point;
- a contextual affordance when the capability is relevant;
- command-palette/search discovery;
- a useful first-run empty state;
- a success/next-step state;
- keyboard-accessible discovery and action paths;
- truthful state, provenance and failure feedback.

For Procedures specifically, the journey must cover learning, saving,
discovering later, running, detecting deviations, and intentional improvement.

The product architecture is domain-neutral. Journey validation must include
at least one non-code example once the corresponding runtime capabilities
exist; coding is not the universal acceptance scenario.

## Stable work-order prefixes

```
PAR-*   Codex parity
ARCH-*  canonical architecture/contracts
ENV-*   environments
MOD-*   model providers
RT-*    agent runtimes
CAP-*   capabilities/skills
ORCH-*  orchestration, context, harness, task graphs and verification
PROV-*  execution providers
LAB-*   parity labs
COL-*   collaboration
UX-*    cross-cutting GUI journeys and discoverability
CLI-*   clients
SEC-*   security/credentials
REL-*   release
```

Only the Tech Lead changes roadmap status. A worker report never closes a phase.
A phase closes only after merged implementation plus acceptance evidence.
