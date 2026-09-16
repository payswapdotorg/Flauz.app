# Codex Universal GUI Integration Work Orders

**Status:** IN EXECUTION — GUI-001 (PR #1), GUI-003 (PR #2), GUI-002 (PR #3) MERGED; GUI-004 COMPLETE (`docs/codex-universal/reports/GUI-004-report.md`, branch `gui-004/unified-ux`)
**Repository:** `payswapdotorg/Flauz.app`
**Authority:** `docs/codex-universal/GUI-INTEGRATION-ARCHITECTURE.md`
**Universal source of truth:** `payswapdotorg/codex`
**Maximum concurrent workers:** 3

## Graph

```text
GUI-001 Foundation + Boundary Lock
        |
        +----------------------+
        |                      |
        v                      v
GUI-002 Codex Desktop     GUI-003 Universal Workflow
       Parity Closure          Client Integration
        |                      |
        +----------+-----------+
                   v
          GUI-004 Unified UX
                   |
          +--------+---------+
          |                  |
          v                  v
 GUI-005 Multi-Environment   PACK-UX-001 Pack-aware
          UX                Workflow/System Views
          |                  |
          +--------+---------+
                   v
          GUI-006 Distribution + Release
                   |
                   v
          GUI-007 Parity + Product Validation
```

The Universal Pack contract foundation in `payswapdotorg/codex` is a parallel architectural track. It must not block GUI-002 or GUI-003.

## GUI-001 — Foundation + Boundary Lock

**Depends on:** none

Establish Flauz.app as the official GUI client repository for Codex Universal and lock both the Workflow and Pack client boundaries.

Required:

- inspect the current codexRS fork and identify all retained subsystems;
- pin the codexRS compatibility baseline already used by the fork;
- inspect the current Universal Workflow and Pack contracts in `payswapdotorg/codex`;
- define the client/app-server boundary for Universal workflow and Pack operations;
- define transport/version/capability negotiation;
- document ownership of runtime state, UI state, workflow state, Pack state and credentials;
- add integration test scaffolding with isolated `CODEX_HOME`;
- preserve codexRS safety/boundedness rules;
- ensure Pack state is requested through typed Universal contracts and never becomes GUI-owned durable authority.

Acceptance:

- no duplicate runtime/engine is introduced;
- the GUI can connect to the existing Codex runtime;
- typed Universal Workflow and Pack client boundaries exist;
- Pack identity/revision and WorkflowVersion references have explicit provenance boundaries;
- the Tech Lead can dispatch GUI-002, GUI-003 and the staged Pack contract work independently.

## GUI-002 — Codex Desktop Parity Closure

**Status:** COMPLETE (release-critical selection) — see `docs/codex-universal/reports/GUI-002-report.md`
**Depends on:** GUI-001

Use the codexRS parity matrix as the behavioral baseline.

Close release-critical parity gaps required for the target platform matrix, prioritizing:

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
- accessibility and restart/reconnect behavior.

Do not spend effort reproducing explicitly unavailable proprietary behavior unless a public contract exists and the feature is required by our product acceptance criteria.

Acceptance:

- parity matrix rows selected as release-critical are green;
- same-state visual/interaction tests cover them;
- no upstream proprietary runtime dependency is introduced.

## GUI-003 — Universal Workflow Client Integration

**Status:** COMPLETE — see `docs/codex-universal/reports/GUI-003-report.md`
**Depends on:** GUI-001

Build the first complete Universal workflow vertical slice in the native GUI.

Required user path:

```text
New Workflow
 -> choose DEMONSTRATE / INSTRUCT / HYBRID
 -> teach
 -> observe live trajectory/evidence
 -> compile
 -> review candidate
 -> approve/publish immutable version
 -> run
 -> observe execution/evidence
 -> close/reopen
 -> inspect durable state
```

The GUI must use the existing Universal workflow control plane and contracts.

Acceptance:

- the entire path works without terminal fallback;
- published versions are immutable and control-plane authoritative;
- execution and evidence are the actual Universal runtime path;
- restart/reconnect is tested.

## GUI-004 — Unified Agent + Workflow UX

**Status:** COMPLETE — see `docs/codex-universal/reports/GUI-004-report.md`
**Depends on:** GUI-002, GUI-003

Make workflows a first-class peer to chats/projects instead of a separate tool while reserving room for Pack-aware system views.

Required:

- unified navigation;
- workflow/project linking;
- workflow-aware agent sessions;
- run history;
- evidence inspector;
- capability/resource/dependency panels;
- workflow Git lifecycle;
- fork/review/merge/release surfaces;
- clear distinction between chat instructions and durable workflow semantics;
- no third top-level product surface for Packs.

Acceptance:

A normal user can move from idea -> agent -> teaching -> workflow -> run -> evidence -> version without opening a terminal, and Pack state can be introduced later without restructuring the top-level navigation.

## GUI-005 — Multi-Environment UX

**Depends on:** GUI-004

Expose the Universal execution plane without inventing modality-specific semantics.

Required first-class experiences:

- Browser;
- Computer/Desktop;
- Terminal;
- API/Tool/MCP;
- Human gates;
- mixed-environment workflow timeline.

Show capability/resource bindings, approvals, takeover/recovery, and environment changes as explicit product state.

Acceptance:

- one workflow can visibly cross environment boundaries;
- the GUI shows the resulting evidence and binding history;
- environment failures fail closed and are actionable.

## PACK-UX-001 — Pack-aware Workflow/System Views

**Depends on:** GUI-001 and the Codex Pack contracts `PACK-001` through `PACK-004` when available.

Keep the desktop product as two primary surfaces while adding Pack information inside the Workflow/System Experience.

Required first-stage views:

- Pack identity and immutable revision;
- Mission / Value / Context;
- Pack Constitution / policy summary;
- system-state summary;
- referenced WorkflowVersions;
- capability/dependency graph;
- evaluation/evidence summary;
- assurance/determinism policy;
- candidate vs promoted state;
- provenance and rollback checkpoint display when available.

Acceptance:

- Pack state is readable without exposing internal implementation details unnecessarily;
- workflow semantics remain visibly distinct from Pack system state;
- GUI never becomes Pack durable authority;
- no Pack generator/evolution engine is required for this work order.

## GUI-006 — Distribution + Release

**Depends on:** GUI-004

Produce a real desktop application release.

Required:

- Windows and Linux primary support according to the fork's actual platform capabilities;
- signed/verified release strategy where available;
- checksums;
- installer/portable option;
- desktop integration;
- update procedure;
- uninstall procedure;
- bundled/runtime dependency accounting;
- compatible Codex CLI/app-server version policy;
- Universal workflow compatibility version;
- Pack compatibility version once Pack contracts are present.

Acceptance:

Fresh machine -> download -> install -> launch -> sign in/configure -> create workflow -> teach -> publish -> run -> restart succeeds without source checkout.

## GUI-007 — Parity + Product Validation

**Depends on:** GUI-005, GUI-006

Validate three dimensions separately:

### Parity

Codex Desktop reference behaviors selected as release-critical are reproduced through public/runtime-owned interfaces.

### Universal value

The GUI provides the workflow lifecycle and multi-environment product capabilities that distinguish Codex Universal from an ordinary coding-agent desktop client.

### Pack readiness

The GUI consumes Pack contracts without becoming a second authority, and Pack-aware views remain compatible with future generation/evolution features.

Required evidence:

- native UI tests;
- same-state screenshots for release-critical views;
- fresh-machine validation;
- workflow teaching/execution validation;
- restart/recovery validation;
- cross-environment validation;
- Pack contract/provenance validation where implemented;
- security/secret scan of release evidence.

Final status must explicitly separate:

```text
Desktop parity
Universal workflow UX
Universal execution UX
Pack contract / UX readiness
Distribution readiness
Known limitations
```
