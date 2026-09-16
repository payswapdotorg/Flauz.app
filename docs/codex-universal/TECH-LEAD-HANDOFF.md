# Codex Universal GUI — Tech Lead Handoff

## Mission

Turn `payswapdotorg/Flauz.app` from the forked codexRS desktop client into the native desktop GUI for Codex Universal.

Do **not** rebuild the GUI. The repository is intentionally a fork of codexRS because codexRS already contains the native GPUI application, app-server supervision, protocol layer, platform layer, Git/worktree support, terminal, Browser, Computer Use, settings, persistence, and release foundation required for Desktop parity.

The existing `payswapdotorg/codex` repository remains the Universal runtime/workflow/Pack/client-contract source of truth.

## Final architecture status

Universal architecture: **0.4.0 FROZEN**.

Current approved extensions:

- Pack / governed System-State architecture: Architecture Change Request 002.
- Universal client/adaptor portability: Architecture Change Request 003.

The final frozen model is:

```text
                    CODEX UNIVERSAL AUTHORITY
                              |
                  Versioned client/app protocol
                              |
       +----------+-----------+-----------+-----------+
       |          |           |           |           |
    Desktop      Web       Mobile   Browser Ext   VS Code/IDE
   Flauz.app
       |          |           |           |           |
       +----------+-----------+-----------+-----------+
                              |
                    SAME SEMANTICS
                              |
              +---------------+---------------+
              |               |               |
          Workflows         Packs          Evidence
              |               |               |
              +---------------+---------------+
                              |
                     Universal Runtime
```

A new client surface must be a bounded adapter, not a second Codex runtime.

## Read first

### Flauz.app

1. `AGENTS.md`
2. `docs/architecture.md`
3. `docs/parity-matrix.md`
4. `ROADMAP.md`
5. `docs/codex-universal/GUI-INTEGRATION-ARCHITECTURE.md`
6. `docs/codex-universal/WORK-ORDERS.md`
7. this handoff

### `payswapdotorg/codex`

1. `ARCHITECT_START_HERE.md`
2. `TECH_LEAD_START_HERE.md`
3. `AGENTS.md`
4. `docs/architecture/CODEX-UNIVERSAL-ARCHITECTURE.md`
5. `docs/architecture/CODEX-UNIVERSAL-LOCK.md`
6. `docs/architecture/CODEX-PACK-ARCHITECTURE.md`
7. `docs/architecture/PACK-WORK-ORDERS.md`
8. `docs/architecture/CLIENT-ADAPTER-ARCHITECTURE.md`
9. `docs/architecture/ARCHITECTURE-CHANGE-REQUEST-002.md`
10. `docs/architecture/ARCHITECTURE-CHANGE-REQUEST-003.md`

## Source-of-truth model

```text
Flauz.app
  = native desktop client implementation

payswapdotorg/codex
  = Universal runtime + Model Plane + Workflow Platform + Pack/System-State Platform + client/application contracts

Codex app-server
  = live Codex runtime/application protocol authority
```

The GUI owns presentation, interaction state, and bounded UI metadata. It does not own durable Workflow semantics, Pack semantics, Pack evolution authority, Evidence authority, credential authority, or live Codex persistence.

## Mandatory architectural rules

- Reuse the forked codexRS implementation; do not replace GPUI with another GUI framework.
- Do not introduce Electron, Tauri, Wry, WebView, Node.js, or another browser runtime for the desktop shell.
- Do not create a second agent runtime.
- Do not create a second workflow engine.
- Do not create a second skills/plugins runtime.
- Do not create a Pack generator, Pack optimizer, Pack architecture-search engine, or Pack evolution engine inside the GUI.
- Do not directly read/write live `CODEX_HOME` SQLite, JSONL, auth or logs.
- Keep Codex runtime access through supervised official app-server boundaries.
- Keep Universal workflow and Pack semantics in `payswapdotorg/codex` and expose them through explicit client/application contracts.
- Treat client adapters and execution adapters as separate concepts even when one surface implements both.
- Keep Pack identity, mission, system-state revision, WorkflowVersion references, and provenance durable in the Universal control plane rather than GUI state.
- Keep credentials out of workflow/Pack source, ordinary UI state, prompts and evidence.
- Keep all frames, events, queues, pages and retained UI state bounded.
- Preserve codexRS single-writer storage rules.
- Prefer existing Universal contracts before inventing GUI-specific abstractions.

## Client portability rule

Future clients must be cheap because they consume shared semantics:

```text
Desktop / Web / Mobile / Browser Extension / VS Code / IDE / SDK
                |
                v
       Versioned Universal Contracts
                |
       Workflow / Pack / Evidence
                |
        Universal Runtime + Control Planes
```

A client adapter primarily adds:

- presentation and interaction;
- host/platform integration;
- authentication/session handling;
- protocol bindings;
- capability negotiation;
- streaming/cancellation/reconnect;
- packaging/distribution.

It must not reimplement durable semantic authority.

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

### Parallelization

After GUI-001:

- GUI-002 and GUI-003 may run concurrently.
- Universal `PACK-001` through `PACK-004` may run in parallel in `payswapdotorg/codex`.
- Pack contract work must not block GUI-002 or GUI-003.
- PACK-005 onward is deferred and must not block the first working desktop Workflow experience.

## GUI-001 — Foundation + Boundary Lock

Establish the shared client boundary before significant UI restructuring.

Determine and implement:

- exact Universal Workflow and Pack client contracts required by the GUI;
- existing contracts that already satisfy them;
- whether app-server/public application protocol extensions are sufficient;
- process ownership of Universal services;
- protocol/version compatibility;
- capability negotiation;
- authentication/authorization and resource binding semantics;
- streamed events and correlation IDs;
- bounded frames/queues;
- cancellation;
- reconnect/resume;
- error taxonomy;
- redaction/secret handling;
- Pack state ownership versus GUI-owned presentation state.

Acceptance:

- minimal real connectivity to the Universal runtime exists;
- typed Workflow and Pack boundaries exist;
- GUI remains presentation/client authority only;
- restart/reconnect behavior is defined and tested;
- no duplicate runtime or semantic engine is introduced.

## GUI-002 — Codex Desktop Parity Closure

Use the fork's `docs/parity-matrix.md` as the behavioral baseline.

Prioritize release-critical gaps rather than implementing every conceivable proprietary feature.

Cover as applicable:

- project/chat shell;
- task lifecycle;
- composer and command behavior;
- streaming timeline;
- settings/account/configuration;
- Git/worktrees/diff review;
- terminal/files/outputs;
- Browser;
- Computer Use;
- Skills/plugins/MCP Apps;
- notifications/background completion;
- accessibility;
- restart/reconnect.

Do not reproduce proprietary internals when no public/runtime-owned contract is required for the product.

## GUI-003 — Universal Workflow Vertical Slice

Implement the first complete Workflow path in the native GUI:

```text
create workflow
  -> DEMONSTRATE / INSTRUCT / HYBRID
  -> teach
  -> review trajectory/evidence
  -> compile
  -> review candidate
  -> approve/publish immutable version
  -> run
  -> inspect live execution/evidence
  -> close/reopen
```

No terminal fallback for the normal path.

Workflow authority remains entirely in the Universal control plane.

## GUI-004 — Unified Agent + Workflow/System UX

Make the application feel like one product rather than codexRS plus a disconnected workflow tool.

Normal user journey:

```text
idea -> chat/agent -> teach -> workflow -> run -> evidence -> version -> fork/improve
```

Maintain exactly two primary desktop surfaces:

```text
AGENT EXPERIENCE
WORKFLOW / SYSTEM EXPERIENCE
```

Do **not** create a third top-level Pack product surface.

## GUI-005 — Multi-Environment UX

Expose the Universal execution plane with one semantic model:

- Browser;
- Computer/Desktop;
- Terminal;
- API/Tool/MCP;
- Human gates;
- mixed-environment workflows.

Show:

- environment changes;
- capability/resource bindings;
- approvals;
- takeover;
- recovery;
- evidence and binding history.

## PACK-UX-001 — Pack-Aware Workflow/System Views

Implement Pack presentation inside the Workflow/System experience after GUI-001 and the initial Pack contracts are available.

First-stage views should include:

```text
Pack identity / immutable revision
Mission / Value / Context
Pack Constitution / policy summary
System State
WorkflowVersion references
Capability / dependency graph
Evaluation / evidence summary
Assurance / determinism policy
Candidate vs promoted revision
Provenance / rollback checkpoint
```

This work order is inspect/governance-oriented.

Do not implement:

- Pack generation;
- architecture search;
- automatic experimentation;
- automatic promotion;
- self-evolution;
- Pack optimizer;

inside Flauz.app as part of the current release.

## GUI-006 — Distribution + Release

Produce a real desktop release using the fork's release machinery where possible.

Required:

- Windows/Linux according to verified fork capabilities;
- installer/portable option;
- checksums and verification;
- desktop integration;
- update/uninstall procedure;
- runtime dependency accounting;
- compatible Codex CLI/app-server policy;
- Universal Workflow compatibility;
- Pack compatibility metadata;
- release artifact provenance.

## GUI-007 — Final Validation

Validate separately:

1. Desktop parity;
2. Universal Workflow UX;
3. mixed-environment execution UX;
4. Pack contract/provenance consumption;
5. client boundary/reconnect behavior;
6. fresh-machine installation;
7. security and evidence hygiene.

Required evidence:

- native UI tests;
- same-state screenshots for release-critical views;
- Workflow teaching/execution validation;
- restart/recovery validation;
- mixed-environment validation;
- Pack contract/provenance validation;
- fresh-machine validation;
- release artifact verification;
- secret/security scan.

## Worker output requirements

Every worker must produce:

- implementation commit(s);
- focused tests;
- relevant docs/contracts updated;
- explicit acceptance checklist;
- exact verification commands/results;
- known limitations;
- no unrelated refactors.

## Absolute non-goals for this handoff

Do not:

- rebuild the desktop UI from scratch;
- replace GPUI;
- build a second runtime;
- build a second Workflow engine;
- build a Pack runtime in the client;
- create client-specific Workflow/Pack semantics;
- introduce a webview-based desktop shortcut;
- expand into full Pack self-evolution before the first Workflow desktop vertical slice is proven.

## Final release gate

Do not declare the GUI complete merely because the application window opens.

The completed release must demonstrate:

- real Codex agent interaction;
- Desktop-grade core UX;
- GUI Workflow teaching in all three modes;
- immutable Workflow publication;
- GUI Workflow execution;
- durable run history/evidence;
- at least one mixed-environment Workflow path;
- Pack contract consumption without duplicate authority;
- restart/reconnect;
- fresh-machine installation;
- release artifact verification.

The final report must explicitly state:

```text
CODEX_DESKTOP_PARITY: <status>
UNIVERSAL_WORKFLOW_GUI: <status>
MULTI_ENVIRONMENT_GUI: <status>
PACK_CONTRACT_UX: <status>
CLIENT_PORTABILITY_BOUNDARY: <status>
DISTRIBUTION: <status>
OVERALL: <status>
```
