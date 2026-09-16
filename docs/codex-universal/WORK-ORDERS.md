# Codex Universal GUI Integration Work Orders

**Status:** READY FOR TECH LEAD
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
                   v
          GUI-005 Multi-Environment UX
                   |
                   v
          GUI-006 Distribution + Release
                   |
                   v
          GUI-007 Parity + Product Validation
```

## GUI-001 — Foundation + Boundary Lock

**Depends on:** none

Establish Flauz.app as the official GUI client repository for Codex Universal.

Required:

- inspect the current codexRS fork and identify all retained subsystems;
- pin the codexRS compatibility baseline already used by the fork;
- inspect the current Universal contracts in `payswapdotorg/codex`;
- define the client/app-server boundary for Universal workflow operations;
- define transport/version/capability negotiation;
- document ownership of runtime state, UI state, workflow state and credentials;
- add integration test scaffolding with isolated `CODEX_HOME`;
- preserve codexRS safety/boundedness rules.

Acceptance:

- no duplicate runtime/engine is introduced;
- the GUI can connect to the existing Codex runtime;
- a typed Universal client boundary exists;
- the Tech Lead can dispatch GUI-002 and GUI-003 independently.

## GUI-002 — Codex Desktop Parity Closure

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

**Depends on:** GUI-002, GUI-003

Make workflows a first-class peer to chats/projects instead of a separate tool.

Required:

- unified navigation;
- workflow/project linking;
- workflow-aware agent sessions;
- run history;
- evidence inspector;
- capability/resource/dependency panels;
- workflow Git lifecycle;
- fork/review/merge/release surfaces;
- clear distinction between chat instructions and durable workflow semantics.

Acceptance:

A normal user can move from idea -> agent -> teaching -> workflow -> run -> evidence -> version without opening a terminal.

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
- Universal workflow compatibility version.

Acceptance:

Fresh machine -> download -> install -> launch -> sign in/configure -> create workflow -> teach -> publish -> run -> restart succeeds without source checkout.

## GUI-007 — Parity + Product Validation

**Depends on:** GUI-005, GUI-006

Validate two dimensions separately:

### Parity

Codex Desktop reference behaviors selected as release-critical are reproduced through public/runtime-owned interfaces.

### Universal value

The GUI provides the workflow lifecycle and multi-environment product capabilities that distinguish Codex Universal from an ordinary coding-agent desktop client.

Required evidence:

- native UI tests;
- same-state screenshots for release-critical views;
- fresh-machine validation;
- workflow teaching/execution validation;
- restart/recovery validation;
- cross-environment validation;
- security/secret scan of release evidence.

Final status must explicitly separate:

```text
Desktop parity
Universal workflow UX
Universal execution UX
Distribution readiness
Known limitations
```
