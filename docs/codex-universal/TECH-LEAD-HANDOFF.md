# Codex Universal GUI — Tech Lead Handoff

## Mission

Turn `payswapdotorg/Flauz.app` from the forked codexRS desktop client into the native desktop GUI for Codex Universal.

Do **not** rebuild the GUI. The repository is intentionally a fork of codexRS because codexRS already contains the native GPUI application, app-server supervision, protocol layer, platform layer, Git/worktree support, terminal, Browser, Computer Use, settings, persistence, and release foundation required for Desktop parity.

The existing `payswapdotorg/codex` repository remains the Universal runtime/workflow source of truth.

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

## Source-of-truth model

```text
Flauz.app
  = GUI/client implementation

payswapdotorg/codex
  = Universal runtime + Workflow Platform + universal contracts

Codex app-server
  = live Codex runtime/application protocol authority
```

The GUI owns presentation, interaction state, and its own bounded UI metadata. It does not own durable workflow semantics or live Codex persistence.

## Mandatory architectural rules

- Reuse the forked codexRS implementation; do not replace GPUI with another GUI framework.
- Do not introduce Electron, Tauri, Wry, WebView, Node.js, or another browser runtime for the desktop shell.
- Do not create a second agent runtime.
- Do not create a second workflow engine.
- Do not create a second skills/plugins runtime.
- Do not directly read/write live `CODEX_HOME` SQLite, JSONL, auth or logs.
- Keep Codex runtime access through supervised official app-server boundaries.
- Keep Universal workflow semantics in `payswapdotorg/codex` and expose them through an explicit client contract.
- Keep credentials out of workflow source, ordinary UI state, prompts and evidence.
- Keep all frames, events, queues, pages and retained UI state bounded.
- Preserve codexRS single-writer storage rules.

## Execution order

```text
GUI-001
   |
   +----------------------+
   |                      |
   v                      v
GUI-002                 GUI-003
   |                      |
   +----------+-----------+
              v
           GUI-004
              |
              v
           GUI-005
              |
              v
           GUI-006
              |
              v
           GUI-007
```

GUI-002 and GUI-003 are the intended two-worker parallelization point after GUI-001.

## GUI-001

Establish the client boundary and integration scaffolding.

Do not begin by rebuilding screens. First determine:

- the exact Universal client protocol needed by the GUI;
- which existing Universal APIs/contracts already satisfy it;
- whether an existing app-server extension is sufficient;
- where Universal services should live/process-wise;
- capability negotiation/versioning;
- reconnect/restart behavior;
- authorization and resource binding behavior.

The deliverable must leave a minimal end-to-end connectivity slice and typed contract.

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

## GUI-005

Expose Browser, Computer/Desktop, Terminal, API/Tool/MCP and Human execution as first-class workflow execution surfaces while retaining one workflow semantic model.

Show environment changes, capability/resource bindings, approvals, takeover and recovery explicitly.

## GUI-006

Package the actual desktop application. Reuse the fork's release tooling where possible and add Universal compatibility metadata.

## GUI-007

Validate:

1. codexRS Desktop parity for release-critical flows;
2. Universal workflow lifecycle;
3. mixed-environment workflow execution;
4. recovery/restart;
5. fresh-machine installation and use;
6. security/evidence hygiene.

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
- restart/reconnect;
- fresh-machine installation;
- release artifact verification.

The final report must separately state:

```text
CODEX_DESKTOP_PARITY: <status>
UNIVERSAL_WORKFLOW_GUI: <status>
MULTI_ENVIRONMENT_GUI: <status>
DISTRIBUTION: <status>
OVERALL: <status>
```
