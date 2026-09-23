# Wave 5 Work Orders — F6 Depth: Leases/Conflicts + Human Takeover

> **Status: DISPATCHED 2026-09-23** (LEASE-001 Worker A, TAKE-001 Worker
> B — all base = main @ ddbf772; the dispatch prompts pin the full SHA).
> Shared-contract authority:
> [F2-CONTRACT-KERNEL.md](F2-CONTRACT-KERNEL.md) (frozen) + the Wave-2
> addendum in [WAVE2-WORK-ORDERS.md](WAVE2-WORK-ORDERS.md) (frozen) + the
> Wave-3 addendum in [WAVE3-WORK-ORDERS.md](WAVE3-WORK-ORDERS.md) (frozen)
> + the Wave-4 addendum in [WAVE4-WORK-ORDERS.md](WAVE4-WORK-ORDERS.md)
> (frozen) + the **Wave-5 kernel addendum** below + the MERGED contract
> crates (flauz-world / flauz-exec / flauz-context / flauz-cap /
> flauz-orch / flauz-prov / flauz-lab / flauz-collab). Every work order
> follows [WORK-ORDER-TEMPLATE.md](../WORK-ORDER-TEMPLATE.md). Non-empty
> `Contract deviations` in a worker report blocks closure. Workers deliver
> via `git bundle` on a single clean commit branch; the Lead gates, fixes,
> merges.
>
> **The phase this wave serves** (ACTIVE-EXECUTION-STATE.md immediate
> order item 3 — the remaining F6 orchestration depth required for the
> production scenario; ROADMAP item 6's "leases/conflict handling and
> human-takeover UX"): **LEASE-001** closes J-16 (inspect resource
> conflicts) with the lease-manager semantics over the frozen world
> lease contract (queue/escalate/expiry/renewal) + resource-aware
> scheduling data; **TAKE-001** closes the takeover/approval/handoff
> contracts + cancellation/dependency propagation with the J-17
> review surface. Real multi-client transport, marketplace and editors
> remain F10+ (out of scope).

## Wave-5 kernel addendum (frozen by the Tech Lead before dispatch)

1. **Existing signatures are frozen — and the graph/evaluator is now
   load-bearing.** No trait signature changes; no `GraphEvent`,
   `NodeState`, `HarnessStateKind`/`HarnessTransition`, `WaitOn`,
   `ResourceLease`/`ConflictPolicy` or world-store shape changes.
   Additive behavior lands as NEW record families in new crates (inputs
   as data) + NEW world event types through the existing world-store
   seam (the frozen dotted grammar; `task.`/`resource.`-namespaced) +
   additive UI seams. If an evaluator/harness integration genuinely
   requires an additive enum variant or transition, mark it a
   deviation with the exact match-sites updated — the Lead gates it
   case by case.
2. **The conflict-honesty law (J-16).** Every lease conflict is NAMED:
   who holds, who waits, which policy decided (reject / queue /
   escalate), and what happens next. A silently-queued request, a
   silently-dropped waiter, or an ambiguous conflict is forbidden. The
   queue is FIFO and deterministic (waiters advance in request order);
   expiry is honest (a lease not held at the moment is not held — no
   implicit renewal; renewal is an EXPLICIT new request); escalation
   produces a named "needs the human" record with the decision the
   human must make, never an auto-resolution of a contested resource.
3. **The takeover law (J-17).** A takeover is an EXPLICIT, ATTRIBUTED
   handoff: who took over (the human actor), from what (the agent/
   node/run), when, and why. The human's turn lands on the SAME task
   event stream (never a second store); the agent's prior work is
   preserved verbatim (the projection law); the handback is explicit
   and attributed. An approval gate BLOCKS with a named need (what
   must be approved, on whose authority, with what consequence);
   approval and denial are attributed decisions; denial carries the
   named consequence (the node fails honestly — it never silently
   proceeds).
4. **Cancellation honesty (the honest-failure family).** Cancellation
   is distinguished from failure everywhere. Cancelling a node leaves
   dependents in NAMED states (cancelled, or blocked-resolved with the
   reason), never silently stuck `Blocked` forever; already-attributed
   work (artifacts/evidence) is kept; the run-level record stays at
   most one `RunCanceled` per sequence (the frozen law) with the
   node-level propagation as new world event types.
5. **Leases are world contracts; the manager is data.** The
   lease-manager takes lease snapshots + requests + policy as DATA (no
   live counters, no clock reads — caller-supplied `now`, the kernel §7
   law); its decisions are deterministic, canonical-JSON records. The
   manager does NOT import flauz-world (the flauz-cap law: the frozen
   formats cross the seam as strings), and the world store keeps
   owning the durable lease entities.
6. **GUI slices follow the seven-layer rule + the d25 listener law.**
   The palette is never the only discovery mechanism; every chord has
   its `on_action` listener REGISTERED AND SEAM-TESTED; conflicts and
   takeovers are in user language ("Ana is using the browser right now
   — your task is next in line (2nd)", "Needs you: approve the
   environment switch", never "lease ref / conflict policy enum").
7. **The standing laws carry.** Canonical JSON (`"v":1`, snake_case,
   `deny_unknown_fields`, no floats, RFC3339-Z, integer-ms);
   determinism (no wall clock, no entropy); credentials are references
   forever (no material anywhere); serde/chrono-family deps only for
   the new crates; proportional engineering.

---

## LEASE-001 — The lease/conflict fabric: the manager, resource-aware scheduling data, the J-16 surface

```
ID: LEASE-001
Title: Resource leases with honest conflicts — the lease manager
  (reject/queue/escalate, expiry sweeps, explicit renewal), the
  resource-aware scheduling data plane, and the inspect-resource-
  conflicts surface (J-16)
Phase: Wave 5 (F6 depth — leases/conflict handling)
Owner: Worker A (agents-tab session)
Dependencies: flauz-world (the FROZEN ResourceLease + ConflictPolicy +
  AccessMode + the world-store seam — you touch NO flauz-world file);
  flauz-orch (the graph's WaitOn::ResourceReady + ResourceAvailable
  semantics — read-only reference); the shell-family UI patterns
Contract(s): F2-CONTRACT-KERNEL.md; Wave-2/3/4 addenda; this addendum
  §1, §2, §5, §6, §7
Problem: the lease CONTRACT exists (grant/expire, is_held_at, the
  reject/queue/escalate policy enum) but nothing implements the
  SEMANTICS: a conflicting request is neither queued nor escalated
  anywhere; the graph's resource waits clear on availability but no
  manager decides who gets a contended resource or names the conflict;
  J-16 (inspect resource conflicts) has no surface — a user cannot see
  who holds what, who is waiting, or make the call on an escalated
  conflict.
User-visible outcome: a user sees every resource's state ("Free",
  "Ana is using the browser — exclusive until 3:40 PM", "2 tasks
  waiting — yours is next"), sees their own place in a queue with the
  honest consequence, and when a conflict escalates makes the call
  with the full named picture (who holds, who waits, what each needs);
  the graph's waits carry named queue positions.
Scope:
  - crates/flauz-lease/** (NEW self-contained crate, the flauz-cap
    pattern: serde/chrono-family only, inputs as data, NO
    contract-crate imports):
    - request.rs: LeaseRequest (the resource ref + access mode + the
      requester actor + the requested window + the policy as data) +
      LeaseDecision — Granted (the lease record as data) / Rejected
      (the named holder + the policy that rejected) / Queued (the
      named queue: position, who is ahead, the projected grant moment
      given the holders' deadlines) / Escalated (the named conflict:
      holder + requester + modes + the decision the human must make).
      Every variant carries full attribution — the conflict-honesty
      law is structural (a decision without the named cause cannot be
      constructed).
    - manager.rs: the deterministic lease manager — decide(request,
      lease_snapshots, wait_queue_snapshots, now) → LeaseDecision
      (pure function over data); the expiry sweep (given snapshots +
      now → which leases released + which waiters advance, IN REQUEST
      ORDER, with the advanced positions named); the explicit renewal
      (a renewal is a new request that names the lease it extends —
      never implicit).
    - fakes.rs + fixtures per family (typical/minimal/invalid) +
      conformance tests: the full conflict space (exclusive×exclusive,
      exclusive×shared, shared×shared), all three policies, expiry
      mid-queue, renewal success/conflict, determinism (same inputs →
      byte-identical decisions), canonical JSON round-trips.
  - The world-stream event vocabulary (as DATA in the crate — the
    registered new event types the caller records through the existing
    world-store seam, the frozen-format discipline): lease.requested /
    lease.granted / lease.rejected / lease.queued / lease.escalated /
    lease.released / lease.expired / lease.renewed — canonical payload
    shapes the crate defines and tests.
  - crates/codex-app/src/ui/flauz_conflicts.rs (NEW, the shell-family
    pattern): the resource-conflicts surface — the resource list with
    live state in user words (holder + mode + deadline), the queue
    view ("2 tasks waiting — yours is next"), the escalation card
    ("Needs you: two tasks want the browser exclusively — Ana's task
    holds it until 3:40 PM; Dev's task has waited 12 minutes") with
    the decision affordances (grant to waiter / keep holder — each
    with the consequence stated), the honest empty state (no
    contention → what the surface adds), the task-surface affordance
    when THIS task waits or holds ("Waiting for the browser — 2nd in
    line").
  - ui.rs: ONLY the named seams (module + palette rows + chord +
    registrations + the on_action LISTENER) — distinct from every
    prior wave's seams.
Non-goals: NO flauz-world/flauz-orch file modifications (the manager
  is data; the wiring is the caller's through the existing seams), NO
  GraphEvent/NodeState/WaitOn changes, NO real-time lock managers, NO
  distributed leasing, NO automatic conflict resolution (escalation is
  the human's call — §2).
Files/subsystems owned: crates/flauz-lease/**; the workspace Cargo.toml
  members line (one line); crates/codex-app/src/ui/flauz_conflicts.rs;
  ui.rs NAMED SEAMS ONLY
Inputs: flauz-world's lease.rs (the frozen contract — snapshot shapes
  as data), the ORCH-004 graph's resource-wait semantics (read-only),
  the d24/d25/d26 scene calibration patterns
Outputs/artifacts: the flauz-lease crate + the event vocabulary +
  fixtures + the conflicts surface + tests
Tests: the conflict-honesty law (no decision without the named cause;
  every variant attributed); the full conflict matrix; the queue law
  (FIFO, deterministic advancement, honest positions); the expiry law
  (honest release + in-order advancement, no implicit renewal);
  renewal-explicit; escalation (named conflict + the human decision,
  never auto-resolved); determinism + canonical JSON + fixtures; the
  credential scan over the new families; the UI module tests (copy
  rules — user words with consequences, tested; seven layers; the
  chord with the listener SEAM-TESTED — the d25 law)
GUI/lab evidence: Lead gate — lab scenes at the merged binary (the
  conflicts surface: the free state, the held state with the holder's
  words, the queue with positions, the escalation card with the
  decision affordances)
UX journey IDs: J-16 (inspect resource conflicts — the full surface
  slice), J-17 (the escalation card is an attention surface the
  Activity rows can attribute)
Primary discovery surface: the resource-conflicts panel
  (Workspace-level)
Contextual discovery surface: the task-surface affordance when this
  task waits or holds
Search/palette discovery: palette rows ("See who is using what",
  "Resolve a resource conflict")
Empty/success-state behavior: no contention → the honest free state +
  what the surface adds; held → the holder + deadline + queue; escalated
  → the decision card with consequences
Keyboard path: a letter-family chord (Ctrl+Alt+Shift+L suggested —
  verify no conflict at implementation; the palette family is at 89
  rows, ALL → 91) + scoped Escape; tab-navigable flow
Acceptance criteria:
  1. Single clean commit on feat/lease-001-conflicts at the pinned
     base; owned files only.
  2. The conflict-honesty law proven: no LeaseDecision without the
     named cause + attribution; escalation never auto-resolves.
  3. The queue law proven: FIFO advancement, honest positions, honest
     expiry, renewal-explicit — all deterministic.
  4. The surface: seven layers; user language with consequences
     (tested); the chord listener SEAM-TESTED (the d25 law).
  5. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive crate + modules + seams; revert the commit.
Integration notes: the manager is pure data (no imports); the event
  vocabulary is the wiring contract the Lead's wave-gate harness
  drives end-to-end (a contended graph run → named queue events on
  the world stream → the escalation card → the human decision → the
  waiter granted). The UI's live state is the module's fake-seam state
  this wave (the picker/providers precedent) — the honest not-wired
  note says so plainly.
Status: DISPATCHED 2026-09-23
```

---

## TAKE-001 — The human takeover/approval/handoff fabric + cancellation propagation, the J-17 surface

```
ID: TAKE-001
Title: The human in the loop — takeover/approval/handoff records with
  attribution, approval gates with named needs and consequences,
  cancellation/dependency propagation with named terminal states, and
  the needs-you review surface (J-17)
Phase: Wave 5 (F6 depth — human takeover/approval/handoff)
Owner: Worker B (agents-tab session)
Dependencies: flauz-exec (the FROZEN harness state machine — the
  Escalated state + prepare/execute/observe/verify/persist/continue/
  recover/escalate transitions, read-only reference); flauz-orch (the
  graph's NodeFailed/RunCanceled semantics — read-only); flauz-world
  (the event stream + ActorRef); the attention/activity surfaces
  (WO-P2-008/P2-013 — additive attribution only); the shell-family UI
  patterns
Contract(s): F2-CONTRACT-KERNEL.md; Wave-2/3/4 addenda; this addendum
  §1, §3, §4, §6, §7
Problem: the harness can ESCALATE ("the human is needed") but nothing
  models what happens next: the human's work is not a first-class
  attributed turn, the handback to the agent is undefined, approval
  gates do not exist (a node that needs approval has no way to block
  with a named need), and cancellation is a run-level event with no
  dependency propagation (dependents of a cancelled node would sit
  silently Blocked forever) — the honest-failure family is incomplete.
User-visible outcome: when a run needs the human, the user sees the
  named need ("Needs you: approve the environment switch — moving to
  the remote sandbox will re-run the setup steps"), decides with the
  consequence stated, can TAKE OVER the agent's turn (their work lands
  on the same task, attributed to them; the agent's prior work
  preserved), hands back explicitly, and can cancel a node or the run
  with every dependent told the truth ("Cancelled — the research step
  it waited on was cancelled").
Scope:
  - crates/flauz-takeover/** (NEW self-contained crate, the flauz-cap
    pattern: serde/chrono-family only, inputs as data, NO
    contract-crate imports):
    - takeover.rs: TakeoverRecord (the human actor ref + what they took
      over — the node/run ref + the agent ref — + when + why) +
      HandbackRecord (the explicit return + what the human did +
      whether the agent resumes or the run completes); the takeover
      law is structural (a record without full attribution cannot be
      constructed).
    - approval.rs: ApprovalGate (the named need: what must be approved,
      the requesting node, the consequence of each side) +
      ApprovalDecision (approve/deny + the deciding human + the
      timestamp + the attributed effect); denial carries the named
      consequence (the node fails honestly with the denial as its
      reason — never a silent proceed); an approval gate is data the
      evaluator/caller consults, not a thread block.
    - cancel.rs: CancellationRecord (what was cancelled — node or run —
      + the reason + the actor) + the propagation projection (given a
      graph as data + the cancelled node → every dependent's named
      terminal state: cancelled, or blocked-resolved with the reason;
      already-attributed work kept); at most one run-level record per
      sequence (the frozen law), the node-level propagation as the
      projection's output.
    - fakes.rs + fixtures per family + conformance tests: the takeover
      round-trip (take over → work attributed to the human → handback
      → the agent resumes; the agent's artifacts preserved verbatim);
      the approval space (approve/deny × consequences, the named
      needs); the propagation matrix (cancel a leaf / a mid node / the
      root; dependents named; attributed work kept; one run record);
      determinism; canonical JSON.
  - The world-stream event vocabulary (as DATA in the crate — the
    registered new event types the caller records through the existing
    world-store seam): task.takeover_started / task.takeover_handback /
    task.approval_requested / task.approval_decided / task.cancelled /
    task.dependent_cancelled — canonical payload shapes the crate
    defines and tests.
  - crates/codex-app/src/ui/flauz_takeover.rs (NEW, the shell-family
    pattern): the needs-you review surface — the approval card (the
    named need + the consequence of each side + Approve/Decline), the
    takeover affordance ("Take over this step" — the agent's state
    preserved note + the handback path), the cancellation affordance
    (node + run, each with the consequence stated: what gets cancelled
    downstream), the honest empty state (nothing needs you), the
    wiring honesty (the live agent wiring is later — say so plainly).
  - The attention-model EXTENSION seams (additive, the COL-001
    precedent): the needs-you rows on the EXISTING attention/activity
    surfaces carry the kind (approval gate / escalation / takeover
    opportunity) — additive data + additive render lines only; NO
    second store (the seam test pins it).
  - ui.rs: ONLY the named seams — distinct from every prior wave's.
Non-goals: NO harness state-machine changes (the takeover rides the
  EXISTING Escalated + continue/recover transitions — the records are
  data the caller drives; if a genuinely-required additive transition
  surfaces, mark it a deviation with the match-sites — the Lead gates
  it), NO GraphEvent/NodeState changes, NO real concurrent-human
  sessions, NO mobile/web surfaces (the Web client is its own wave).
Files/subsystems owned: crates/flauz-takeover/**; the workspace
  Cargo.toml members line (one line; the Lead resolves the two-way
  one-liner at merge); crates/codex-app/src/ui/flauz_takeover.rs; the
  EXISTING attention modules' additive kind seams (named files in the
  report); ui.rs NAMED SEAMS ONLY
Inputs: flauz-exec's harness.rs (the frozen state machine), the
  ORCH-003 recovery surface (the Escalate-to-me precedent), the
  ORCH-004 graph semantics, the COL-001 attention-attribution seams
Outputs/artifacts: the flauz-takeover crate + the event vocabulary +
  fixtures + the needs-you surface + the attention kind seams + tests
Tests: the takeover law (attribution structural; the same-stream law —
  the human's turn is a task event, never a second store; the
  projection law — the agent's work preserved); the approval space
  (named needs, attributed decisions, denial consequence); the
  propagation matrix (every dependent named; attributed work kept; one
  run record); determinism + canonical JSON + fixtures; the credential
  scan; the UI module tests (copy rules with consequences; seven
  layers; the not-wired honesty; the chord listener SEAM-TESTED); the
  attention-extension seam test (additive-only, source-inspected)
GUI/lab evidence: Lead gate — lab scenes at the merged binary (the
  needs-you surface: the approval card with consequences, the takeover
  affordance, the cancel path with the downstream truth, the honest
  empty state)
UX journey IDs: J-17 (review activity that needs the human — the
  approve/decline/take-over flow), J-14/J-15's takeover dimension
  (the human steps in without losing the agent's work)
Primary discovery surface: the needs-you review panel + the attention
  rows' kind attribution
Contextual discovery surface: the task-surface affordance when THIS
  task needs a decision
Search/palette discovery: palette rows ("See what needs you",
  "Take over a running step")
Empty/success-state behavior: nothing needs you → the honest quiet
  state + what lands here; a gate → the card with both consequences;
  after deciding → the attributed outcome + the resume state
Keyboard path: a letter-family chord (Ctrl+Alt+Shift+Y suggested —
  verify no conflict at implementation; the palette family is at 89
  rows, ALL → 91) + scoped Escape; tab-navigable flow
Acceptance criteria:
  1. Single clean commit on feat/take-001-human-loop at the pinned
     base; owned files only.
  2. The takeover law proven: attribution structural, same-stream,
     projection-preserved; the handback explicit.
  3. The approval + cancellation spaces proven: named needs, attributed
     decisions, denial consequences, every dependent named, one run
     record — all deterministic.
  4. The surface: seven layers; user language with consequences
     (tested); the chord listener SEAM-TESTED (the d25 law); the
     attention extension additive-only + seam-tested.
  5. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive crate + modules + seams; revert the commit.
Integration notes: the records are pure data (no imports); the event
  vocabulary is the wiring contract the Lead's wave-gate harness
  drives end-to-end (an approval-gated graph run → the named gate →
  the human decision on the stream → the resume/cancel with named
  dependents). The UI's live state is the module's fake-seam state
  this wave — the honest not-wired note says so plainly.
Status: DISPATCHED 2026-09-23
```

---

## Reporting contract (all Wave-5 workers)

The 11-field completion report (exact headers), delivered in the worker's
final message AND via the delivery bundle branch: WO ID(s); base branch +
SHA; branch/commits; changed files/surfaces; implementation summary;
tests/commands + exact results (state plainly if the sandbox lacks the
Rust toolchain — static reasoning is acceptable, the Lead independently
compiles and gates); kernel-compliance checklist (this addendum's §1-§7,
each item); GUI discoverability layers covered (UI-bearing WOs);
acceptance-criteria evidence (map each bullet); known limitations;
contract deviations (NONE if none); follow-up work.

## Wave-5 integration gate (Lead, after both merge)

Extend the F2/Wave-2/3/4 harness (lead-tools/f2-integration-harness):
the human-in-the-loop scenario — a contended graph run (two nodes,
one exclusive resource, the LEASE-001 manager deciding: one granted,
one queued with its named position, the expiry sweep advancing it),
an approval-gated node (TAKE-001: the named gate on the stream → the
human approves with attribution → the node resumes), a takeover
mid-run (the human's turn attributed on the same stream → the handback
→ the agent's artifacts preserved), and a cancellation (the node
cancelled → every dependent named on the stream → the attributed work
kept → one run record). Plus the lab scenes (the conflicts surface
with the escalation card, the needs-you surface with the approval
card and the takeover affordance) at the merged binary. The record
lands at evidence/w5-gate/WAVE5-GATE-RECORD.md; the F6 depth closes
only then.
