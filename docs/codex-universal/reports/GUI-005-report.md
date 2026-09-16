# GUI-005 — Multi-Environment UX

**Status:** COMPLETE
**Base:** `48d33d4` (merged GUI-004)
**Branch:** `gui-005/multi-environment-ux`
**Work order:** `docs/codex-universal/WORK-ORDERS.md` § GUI-005

## What was delivered

The Universal execution plane is exposed as explicit product state on
the workflow surface, without inventing modality-specific semantics:
every environment label, binding record, and recovery action is either
relayed verbatim from the control plane or honestly absent.

### 1. Environment classification (`workflow_evidence_environment`)

- Control-plane evidence kinds map onto the Universal environment
  classes the GUI labels explicitly: **Browser**, **Computer**,
  **Terminal**, **API / tool / MCP**, **Human gate**, **Binding**,
  **Trace**.
- Unknown kinds return `None` and render as "Unrecognized kind" —
  the GUI never guesses an environment. Focused test pins the
  known-kinds-only behavior (including empty and nonsense inputs).

### 2. Mixed-environment timeline (instance detail)

- Evidence references render as a numbered **Environment timeline**:
  each entry carries its environment class, the verbatim control-plane
  kind, locator, and digest.
- Boundary crossings between two *known* environment classes are
  explicit: `── environment change: Terminal → Browser ──` marker
  lines. Crossings are never claimed for or through unrecognized
  kinds.
- A per-instance summary line lists the distinct environments present
  in evidence ("environments in evidence: Browser, Terminal, Human
  gate"), with an honest placeholder when none are recorded.

### 3. Binding history (published versions)

- `WorkflowBindingResolutionCard` (approved digest → executable
  digest) is captured **verbatim** from
  `WorkflowPublishResponse.binding_resolution` and remembered in the
  bounded, deduplicated published-version bookkeeping.
- The Published versions pane renders it per version:
  `binding resolution approved <digest> → executable <digest>`.
- Fork and improvement releases do not carry a binding-resolution
  audit record in the current protocol; the field stays honestly
  absent rather than fabricated.
- The publish-path reducer test now asserts the audit record survives
  publication remembering.

### 4. Takeover / recovery (instance resume + cancel)

- **codex-core**: `WorkflowInstanceResume` / `WorkflowInstanceCancel`
  actions (cancel requires a non-empty reason so the durable record
  stays actionable), single-flight `instance_resume_pending` /
  `instance_cancel_pending`, and `workflow_refresh_instance`, which
  refreshes the record in the bounded instance list and the open
  detail view while preserving previously loaded evidence (resume and
  cancel responses carry the record but not the evidence list).
- **backend**: typed `workflow_instance_resume` /
  `workflow_instance_cancel` dispatch arms relaying the control-plane
  responses verbatim.
- **UI**: per-instance **Resume** (enabled only while `Paused`) and
  **Cancel** (enabled while `Pending` / `Running` / `Paused`) actions
  with pending labels and disabled states.
- Environment failures fail closed: request failures clear the pending
  flag and surface the control-plane error; the record keeps its
  settled status until the control plane says otherwise.

## Acceptance checklist (vs the work order)

| Requirement | Result |
| --- | --- |
| Browser, Computer/Desktop, Terminal, API/Tool/MCP, Human gates as first-class product state | ✅ environment classes label every evidence entry and the per-instance summary; the native Browser/Computer/Terminal surfaces come from the codexRS parity base (GUI-002) |
| Mixed-environment workflow timeline | ✅ numbered Environment timeline with explicit boundary-crossing markers |
| Capability/resource bindings shown | ✅ compiler binding proposals (GUI-004 candidate review) + publication binding-resolution audit record (this PR) |
| Approvals shown | ✅ approval decisions and approver identity in the governed improve chain (GUI-004); human-gate evidence classified in the timeline |
| Takeover/recovery as explicit product state | ✅ Resume/Cancel lifecycle with single-flight pendings and honest failure paths |
| Environment changes as explicit product state | ✅ boundary-crossing markers + Binding evidence class |
| One workflow can visibly cross environment boundaries | ✅ the timeline renders crossings between consecutive, differently-classified evidence entries |
| GUI shows the resulting evidence and binding history | ✅ verbatim evidence references (kind, locator, digest) + binding-resolution line on published versions |
| Environment failures fail closed and are actionable | ✅ failures clear pendings, surface control-plane errors, and leave settled records untouched; Resume/Cancel gated to valid statuses |

## Verification

Commands (workspace root, `scripts/dev-env.sh` sourced):

```
cargo fmt --all                                          # clean
cargo clippy -p codex-core -p codex-app --all-targets    # 0 errors, 0 warnings
cargo test -p codex-core                                 # 210 passed (208 baseline + 2 new, count asserted)
cargo test -p codex-protocol                             # 60 passed
cargo test -p codex-platform                             # 112 passed + 2 real-runtime integration passed
python3 scripts/check_dependency_policy.py               # 561 packages, passed
```

New tests:

- `workflow_instance_resume_and_cancel_are_single_flight_and_refresh_state`
- `workflow_evidence_environment_classifies_known_kinds_only`
- publish-path test extended to assert binding-resolution retention

## Known limitations

- Environment classification is a presentation-layer mapping of
  control-plane evidence kinds. Kinds the control plane has not
  defined render verbatim without an environment label (by design —
  the GUI does not guess).
- Boundary markers show environment changes *between consecutive
  evidence entries*. Correlating timeline nodes to individual
  evidence references is not exposed by the current protocol; the GUI
  does not infer it.
- Fork/improvement publish responses carry no binding-resolution
  audit record today; the version card shows the field only when the
  control plane reported one.
- Screenshot evidence remains gated on GUI-007 (headless sandbox
  limitation recorded in GUI-003).
