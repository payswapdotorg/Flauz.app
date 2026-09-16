# Codex Universal GUI — Tech Lead Handoff

## Mission

Turn `payswapdotorg/Flauz.app` from the forked codexRS desktop client into the native desktop GUI for Codex Universal.

Do **not** rebuild the GUI. The repository is intentionally a fork of codexRS because codexRS already contains the native GPUI application, app-server supervision, protocol layer, platform layer, Git/worktree support, terminal, Browser, Computer Use, settings, persistence, and release foundation required for Desktop parity.

The existing `payswapdotorg/codex` repository remains the Universal runtime/workflow/Pack source of truth.

## Read first

1. `AGENTS.md`
2. `docs/architecture.md`
3. `docs/parity-matrix.md`
4. `ROADMAP.md`
5. `docs/codex-universal/GUI-INTEGRATION-ARCHITECTURE.md`
6. `docs/codex-universal/WORK-ORDERS.md`
7. In `payswapdotorg/codex`:
   - `ARCHITECT_START_HERE.md`
   - `TECH_LEAD_START_HERE.md`
   - `AGENTS.md`
   - `docs/architecture/CODEX-UNIVERSAL-ARCHITECTURE.md`
   - `docs/architecture/CODEX-UNIVERSAL-LOCK.md`
   - `docs/architecture/CODEX-PACK-ARCHITECTURE.md`
   - `docs/architecture/PACK-WORK-ORDERS.md`
   - `docs/architecture/ARCHITECTURE-CHANGE-REQUEST-002.md`

## Source-of-truth model

```text
Flauz.app
  = GUI/client implementation

payswapdotorg/codex
  = Universal runtime + Workflow Platform + Pack/System-State Platform + contracts

Codex app-server
  = live Codex runtime/application protocol authority
```

The GUI owns presentation, interaction state, and its own bounded UI metadata. It does not own durable workflow or Pack semantics, Pack evolution authority, or live Codex persistence.

## Mandatory architectural rules

- Reuse the forked codexRS implementation; do not replace GPUI with another GUI framework.
- Do not introduce Electron, Tauri, Wry, WebView, Node.js, or another browser runtime for the desktop shell.
- Do not create a second agent runtime.
- Do not create a second workflow engine.
- Do not create a second skills/plugins runtime.
- Do not create a Pack generator, Pack optimizer, or Pack evolution engine inside the GUI.
- Do not directly read/write live `CODEX_HOME` SQLite, JSONL, auth or logs.
- Keep Codex runtime access through supervised official app-server boundaries.
- Keep Universal workflow and Pack semantics in `payswapdotorg/codex` and expose them through explicit client contracts.
- Keep Pack identity, mission, system-state revision, and provenance durable in the Universal control plane rather than GUI state.
- Keep credentials out of workflow/Pack source, ordinary UI state, prompts and evidence.
- Keep all frames, events, queues, pages and retained UI state bounded.
- Preserve codexRS single-writer storage rules.

## Execution order

```text
                         GUI-001
                            |
             +--------------+--------------+
             |                             |
             v                             v
          GUI-002                        GUI-003
             |                             |
             +--------------+--------------+
                            v
                         GUI-004
                            |
                  +---------+---------+
                  |                   |
                  v                   v
               GUI-005           PACK-UX-001
                  |                   |
                  +---------+---------+
                            v
                         GUI-006
                            |
                            v
                         GUI-007
```

GUI-002 and GUI-003 are the intended two-worker parallelization point after GUI-001. The Codex Universal Pack contract work (`PACK-001` through `PACK-004`) may proceed in parallel in the Universal repository but must not block the desktop parity or first Workflow vertical slice.

## GUI-001

Establish the client boundary and integration scaffolding.

Do not begin by rebuilding screens. First determine:

- the exact Universal Workflow and Pack client protocols needed by the GUI;
- which existing Universal APIs/contracts already satisfy them;
- whether an existing app-server extension is sufficient;
- where Universal services should live/process-wise;
- capability negotiation/versioning;
- reconnect/restart behavior;
- authorization and resource binding behavior;
- Pack ownership versus GUI-owned presentation state.

The deliverable must leave a minimal end-to-end connectivity slice and typed contracts for both Workflow and Pack state.

## GUI-002

Treat the fork's `docs/parity-matrix.md` as the Desktop parity baseline.

Prioritize release-critical gaps rather than implementing every conceivable proprietary feature. Preserve the fork's public-contract-only approach.

## GUI-003

Implement one complete Universal workflow vertical slice using the native GUI:

```text
create workflow
  -> choose teaching mode
  -> teach
  -> review trajectory/evidence
  -> compile
  -> review candidate
  -> approve/publish
  -> run
  -> inspect live execution/evidence
  -> close/reopen
```

Teaching modes:

- DEMONSTRATE
- INSTRUCT
- HYBRID

The path must not require terminal commands.

## GUI-004

Unify agent and workflow experiences so the product feels like one desktop application rather than codexRS plus a disconnected workflow tool.

The normal user path should support:

```text
idea -> chat/agent -> teach -> workflow -> run -> evidence -> version -> fork/improve
```

Do not introduce Packs as a third top-level product surface. Reserve the Workflow/System experience for Pack mission, system-state, policy and evaluation views.

## GUI-005

Expose Browser, Computer/Desktop, Terminal, API/Tool/MCP and Human execution as first-class workflow execution surfaces while retaining one workflow semantic model.

Show environment changes, capability/resource bindings, approvals, takeover and recovery explicitly.

## PACK-UX-001

Add Pack-aware views inside the Workflow/System experience after the client boundary exists and the first Pack contracts are available.

The first stage is read/inspect oriented:

```text
Pack identity / revision
Mission / Value / Context
Pack Constitution / policy
System State
WorkflowVersion dependencies
Capabilities / dependencies
Evaluation / evidence
Assurance / determinism policy
Candidate vs promoted revision
Provenance / rollback checkpoint
```

Do not implement automatic Pack generation, architecture search, experimentation, promotion, or self-evolution in the GUI as part of this work order.

## GUI-006

Package the actual desktop application. Reuse the fork's release tooling where possible and add Universal compatibility metadata, including Pack compatibility once the contract exists.

## GUI-007

Validate:

1. codexRS Desktop parity for release-critical flows;
2. Universal workflow lifecycle;
3. mixed-environment workflow execution;
4. Pack contract/provenance consumption where implemented;
5. recovery/restart;
6. fresh-machine installation and use;
7. security/evidence hygiene.

## Worker output requirements

Each worker must produce:

- implementation commit(s);
- focused tests;
- updated relevant docs/contracts;
- explicit acceptance checklist;
- exact verification commands/results;
- known limitations;
- no unrelated refactors.

## Final release gate

Do not declare the GUI complete merely because the application window opens.

The final application must demonstrate:

- real Codex agent interaction;
- Desktop-grade core UX;
- GUI workflow teaching in all three modes;
- immutable workflow publication;
- GUI workflow execution;
- durable run history/evidence;
- at least one mixed-environment workflow path;
- Pack contract consumption without duplicate authority;
- restart/reconnect;
- fresh-machine installation;
- release artifact verification.

The final report must separately state:

```text
CODEX_DESKTOP_PARITY: <status>
UNIVERSAL_WORKFLOW_GUI: <status>
MULTI_ENVIRONMENT_GUI: <status>
PACK_CONTRACT_UX: <status>
DISTRIBUTION: <status>
OVERALL: <status>
```
