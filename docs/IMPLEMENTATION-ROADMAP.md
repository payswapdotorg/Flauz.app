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

## F1 — Codex Desktop parity — **CLOSED 2026-09-21**

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
- ✅ WO-UX-002 begin_new_chat composer focus (keyboard-only chat creation, RG-A11Y N1)
- ✅ WO-P2-019 confirmation-modal focus-trap completion (four modals; runtime probes for the flow-gated four at the RC binding pass)
- ✅ WO-P2-017 palette/overlay close focus restoration (r2 merged via PR #41 → `8dcfcb9`; the r1 deterministic palette-close panic eliminated by parameter-passed close context — compile-time signature pin, CI double matrix, D17 evidence wo-p2-017/)
- ✅ RWO-020/RWO-021/RWO-022 reference/review/adversarial verification waves
- ✅ Release-critical parity scope is green for the selected public/runtime-owned rows;
  bounded proprietary/platform rows remain explicitly classified.

Remaining:
- ✅ remaining full-reference P2/P3 parity differences — explicitly classified
  (WO-F1-SWEEP-001 inventory, reconciled at `7d1d61d`: 5 closed / 17 bounded /
  11 P2 / 8 P3; the re-sliced current-target items → the F2 annex)
- ✅ final parity journeys across the remaining partial/platform rows
  (FR-1/FR-3 baselines at `8a68c9a` + binding PASS at the published rc.14 —
  `FR-BINDING-VERDICT.md`)
- ✅ accessibility/keyboard-only/reduced-motion/screen-reader/contrast pass
  (WO-UX-001 baseline + binding GREEN at the published rc.14 —
  `A11Y-BINDING-VERDICT.md`: N1 closed via UX-002, 017 focus contract verified,
  Find/attention positive paths, 019 trap ladder green; N5/N6 bounded notes →
  F2 a11y)
- ✅ release-candidate soak/reconnect/performance gates (RG-SOAK CLOSED;
  RG-RECONNECT adjudicated PASS; FR-4 fresh-machine rc.14 PASSED)

**Gate:** every remaining full-reference difference is explicitly classified;
release-critical journeys remain green; and the final parity/accessibility/release
journeys pass. — **SATISFIED per the F1 closure record**
(`docs/research/evidence/f1-sweep/F1-CLOSURE-RECORD.md`, 2026-09-21).
**CLOSED**: the user reviewed the record on 2026-09-21 and approved the F1→F2
transition. F1 is complete at v0.1.0-rc.14; the bounded/platform/proprietary
classifications stand and F1 is not reopened because full-reference parity
remains non-green outside the closed release scope.

## F2 — Canonical contracts — **ACTIVE (Wave 1 dispatched 2026-09-21)**

Shared-contract authority: [F2-CONTRACT-KERNEL.md](F2-CONTRACT-KERNEL.md)
(frozen by the Tech Lead before any worker branched: entity identity rules,
ID/version rules, serialization format, event envelope, task identity
semantics, ownership boundaries, file ownership, trait/module boundaries,
fixture naming, test vocabulary).

Wave-1 ownership split (parallel, non-overlapping; ledger:
[research/F2-WORK-ORDERS.md](research/F2-WORK-ORDERS.md)):

- **ARCH-001** — Workspace/Session/Task/Artifact/Resource/ResourceState/
  Event/Observation/Claim/Evidence/ResourceLease + Procedure/ProcedureVersion
  contracts (`crates/flauz-world`).
- **ARCH-002** — Environment/ExecutionProvider/ModelProvider/Model/
  AgentRuntime/Agent/Skill/Capability/ProviderConnection contracts + versioned
  session/event transport (`crates/flauz-exec`).
- **ORCH-001 + UX-001** — Context/MemoryItem/ContextSnapshot/ContextProvenance/
  ModelContextProfile contracts (`crates/flauz-context`) + the first
  discoverable platform GUI shell per PRODUCT-UX-JOURNEYS §2.1 (Workspace
  navigation: Projects/Tasks, Procedures, Artifacts, Activity; Task rail:
  Context, Agents, Environments, Evidence, More/Inspect) with honest empty
  states (`crates/codex-app`).
- **UX-003** (queued) — F2 accessibility carry-overs: N5 PTY focus transfer,
  N6 bracket-swap positive re-evidence, fresh-profile first-run modal
  keyboard-swallowing (promo modal).

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
- ⬜ GUI discovery shell (Workspace navigation + Task rail per
  PRODUCT-UX-JOURNEYS §2.1)

**Gate:** fake model/runtime/environment/provider implementations pass the
contracts without real external services; context, resource, execution,
evidence and procedure fixtures round-trip without external services.
Integration verification (Lead-run after the Wave-1 merges):

```
create task → attach resource → attach environment → attach agent →
compile context → produce artifact → record observation → attach evidence →
verify → persist → reload
```

with the same logical Task preserved through model/context changes. Every
F2 capability ships as a vertical product slice (primary entry, contextual
affordance, palette fallback, empty state, success state, keyboard path,
truthful failure state).

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

Procedure phasing (adjusted 2026-09-21): F2 delivers the Procedure
**contract**; F6 delivers the **first-class Save/Run/Deviation vertical slice**
— the first successful repeatable task must be able to surface "Save as a
reusable workflow", persist a minimal Procedure object, and be discoverable
again from the Workspace; F10 delivers the full library (sharing, search,
versions, improvement UX). Do not wait until F10 before exposing the concept.

**Gate:** two independent model/runtime adapters cooperate on one task across
multiple execution surfaces and produce attributable, independently verifiable
results without a shared giant transcript. Before F6 is declared complete, run
at least one domain-neutral scenario that is not software development
(recommended first scenario: research task → web/browser resources → parallel
research agents → sandbox analysis → evidence collection → human review →
final report artifact → Save as reusable workflow).

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
- ⬜ full Procedure library: sharing, search, versions, improvement UX
  (completes the Procedure lifecycle begun in F2/F6)

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

At every major implementation wave, re-run the applicable J-01..J-18 journeys
from a cold start and verify for each: primary visible path, contextual path,
search/palette fallback, empty/error state, success/next action, keyboard
path, and recovery/reconnect.

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
