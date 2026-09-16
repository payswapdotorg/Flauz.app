# GUI-003 — Universal Workflow Client Integration

**Status:** COMPLETE
**Base:** `e448ee5` (merged GUI-001)
**Branch:** `gui-003/workflow-vertical-slice`
**Work order:** `docs/codex-universal/WORK-ORDERS.md` § GUI-003
**Boundary:** `docs/codex-universal/CLIENT-BOUNDARY.md` (locked by GUI-001)

## What was delivered

The first complete Universal workflow vertical slice in the native GUI:
the whole lifecycle runs in the desktop app through the supervised
app-server boundary, with no terminal fallback and no second engine.

### codex-core — workflow surface state machine (additive, +~1400 lines)

- `MainRoute::Workflows` navigation route.
- Bounded workflow state: `WorkflowState` (teaching session, candidate,
  published versions, instances, instance detail, pending flags, error)
  with house-style caps — `MAX_WORKFLOW_NAME_BYTES` (512),
  `MAX_WORKFLOW_TEXT_BYTES` (8 KiB), `MAX_WORKFLOW_STEPS` (200),
  `MAX_WORKFLOW_FINDINGS` (64), `MAX_WORKFLOW_PUBLISHED_VERSIONS` (50),
  `MAX_WORKFLOW_INSTANCES` (200), `MAX_WORKFLOW_EVIDENCE_REFERENCES`
  (64), and friends.
- Lifecycle actions: `WorkflowTeachStart/Started`, `TeachInstruct`,
  `TeachDemonstrate/Recorded`, `TeachReconcile/Reconciled`,
  `Compile/Compiled`, `Review/Reviewed`, `Approve/Approved`,
  `Publish/Published`, `InstanceRun/RunCompleted`,
  `RefreshInstances/InstancesLoaded`, `InstanceGet/InstanceLoaded`,
  `RequestFailed`, `DismissError`.
- `Effect::WorkflowRequest(WorkflowRequest)` with typed variants for
  all 11 lifecycle requests; the reducer only relays control-plane
  responses into state — it never derives ids, statuses, counts, or
  digests.
- Honest lifecycle semantics: failure actions clear the matching
  pending flag and surface an actionable error; disconnect clears
  ephemeral teaching/candidate state and rehydrates only the durable
  instance list; route entry loads `workflow/instance/list`.

### codex-app backend — typed dispatcher (additive, +~520 lines)

- One backend dispatcher `run_workflow_request` mapping core view types
  to the GUI-001 protocol bindings (`AppServerConnection::workflow_*`)
  and control-plane responses back to completion actions.
- Runtime-unavailable path emits `WorkflowRequestFailed` with a
  dedicated message (no silent drops).
- `const fn` mappers between core/protocol enums; every semantic value
  in the mapped state comes from a response.

### codex-app UI — Workflows view (new module, minimal ui.rs wiring)

- New `crates/codex-app/src/ui/ui_workflow.rs` (~1270 lines):
  teaching pane (mode picker DEMONSTRATE / INSTRUCT / HYBRID, workflow
  name, instruction recording, demonstration recording with kind,
  live record count and session status, reconcile), candidate review
  pane (steps with origin, validation findings by severity, approve
  with approver + reference, publish with commit SHA + optional
  repository/semantic version), published versions list, instances
  pane (run a published version, list with status, instance detail
  with evidence references), and an error banner with dismissal for
  every failure path.
- `ui.rs` changes are wiring-only (+~107 lines): route mount
  (`MainRoute::Workflows => ui_workflow::render_workflows`), sidebar
  entry, command palette `OpenWorkflows`, input-state fields.

### Tests

Six focused reducer tests in codex-core (house style, no
expect/unwrap):

- teaching lifecycle walks the control-plane state machine;
- compile → review → approve → publish → run follow control-plane gates;
- request failures surface control-plane errors and clear pending state;
- lists stay bounded and deduplicated;
- disconnect clears ephemeral state and rehydrates durable state;
- the Workflows route loads the durable instance list.

Transport-level correctness (the same 18 methods against the real
runtime) is already pinned by the GUI-001 integration tests and is not
duplicated here.

## Acceptance checklist (work order)

| Requirement | Result |
| --- | --- |
| New Workflow → choose DEMONSTRATE / INSTRUCT / HYBRID → teach | PASS — mode picker + name + record/reconcile flow in the Workflows view |
| observe live trajectory | PASS — live record count, sequence, session status, demonstration/instruction recording |
| compile | PASS — `WorkflowCompile` action → candidate state with step count + validation |
| review candidate | PASS — steps with origin, findings by severity, epoch/status |
| approve/publish immutable version | PASS — approver + reference + commit SHA (+ optional repo/semver); published list records version identity |
| run | PASS — run a published version from the instances pane |
| observe execution/evidence | PASS — instance detail with evidence references (kind/locator/digest) |
| close/reopen | PASS — teaching close (reconcile) + session/candidate state cleared honestly on failure; app-level route persistence via existing storage |
| inspect durable state | PASS — instance list rehydrated on route entry and after disconnect |
| entire path works without terminal fallback | PASS — all steps are native GUI actions through the supervised boundary |
| published versions immutable and control-plane authoritative | PASS — GUI renders returned identity verbatim; no local mutation paths exist |
| execution/evidence are the actual Universal runtime path | PASS — every request is a real `workflow/*` app-server call |
| restart/reconnect tested | PASS — reducer test for disconnect/rehydrate; transport restart pinned by GUI-001 integration test |

## Verification commands and results

```text
. scripts/dev-env.sh
cargo fmt --all --check                                  → PASS (clean)
cargo clippy -p codex-core --all-targets -- -D warnings  → PASS
cargo clippy -p codex-app --all-targets -- -D warnings   → PASS
cargo test -p codex-core                                 → 205 passed (199 pre-existing + 6 new)
CODEX_RS_TEST_CODEX_BIN=<codex rust-v0.1.0> cargo test -p codex-platform
                                                         → 112 + 2 integration passed (boundary still green)
cargo check -p codex-app                                 → PASS (whole app incl. workflow UI)
python3 scripts/check_dependency_policy.py               → PASS (561 packages)
```

## Known limitations

1. Visual/screenshot validation of the Workflows view could not run in
   the headless sandbox (GPUI needs a GPU/Vulkan surface); the view is
   verified by compilation, the reducer state machine, and the
   transport-level integration tests. GUI-007 owns same-state
   screenshots on real display hardware.
2. Fork/improve surfaces (`workflow/fork`, `workflow/improve/*`) are
   bound at the protocol/boundary layer (GUI-001) but not yet exposed
   in the view; the work order's required path does not include them.
3. Full `cargo build --release -p codex-app` remains the CI-owned gate
   (sandbox RAM limit; see GUI-001 report).
4. Demonstration evidence attach in the UI records text-only events;
   attaching evidence references (label/locator/sha256) via the
   composer is left for GUI-004 polish.

## No unrelated refactors

Diff surface: `crates/codex-core/src/lib.rs` (additive workflow state
machine + tests), `crates/codex-app/src/backend.rs` (additive
dispatcher + mappers), `crates/codex-app/src/ui.rs` (wiring only:
imports, palette command, route mount, input fields),
`crates/codex-app/src/ui/ui_workflow.rs` (new), work-order status
flip, this report.
