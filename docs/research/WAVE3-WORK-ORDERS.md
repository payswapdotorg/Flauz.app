# Wave 3 Work Orders — F6 Orchestration: Context Engine / Harness / Execution Graph

> **Status: 2 of 3 MERGED** (ORCH-002 Worker A — PR #49, merge
> cd59421; ORCH-004 Worker C — PR #50, merge ef43a0e; both CI both
> platforms, deviations NONE; evidence:
> [w3-orch-002](evidence/w3-orch-002/ORCH-002-COMPLETION-REPORT.md) +
> [w3-orch-004](evidence/w3-orch-004/ORCH-004-COMPLETION-REPORT.md)).
> ORCH-003 Worker B: delivered + gated (PR #51, CI running on the
> gate-fix head de4b792). Shared-contract authority:
> [F2-CONTRACT-KERNEL.md](F2-CONTRACT-KERNEL.md) (frozen) + the Wave-2
> addendum in [WAVE2-WORK-ORDERS.md](WAVE2-WORK-ORDERS.md) (frozen) + the
> **Wave-3 kernel addendum** below + the MERGED contract crates themselves
> (flauz-world / flauz-exec / flauz-context / flauz-cap at `e9bee81` — the
> crates are the implementation truth). Every work order follows
> [WORK-ORDER-TEMPLATE.md](../WORK-ORDER-TEMPLATE.md). Non-empty
> `Contract deviations` in a worker report blocks closure. Workers deliver
> via `git bundle` on a single clean commit branch; the Lead gates, fixes,
> merges.
>
> **The F6 gate (IMPLEMENTATION-ROADMAP, F6)**: two independent
> model/runtime adapters cooperate on one task across multiple execution
> surfaces and produce attributable, independently verifiable results
> without a shared giant transcript. Before F6 is declared complete, run
> at least one domain-neutral scenario that is not software development
> (recommended: research task → web/browser resources → parallel research
> agents → sandbox analysis → evidence collection → human review → final
> report artifact → Save as a reusable workflow). The wave integration
> gate (Lead) proves this at the merged binary.

## Wave-3 kernel addendum (frozen by the Tech Lead before dispatch)

1. **Existing signatures are frozen.** Wave-3 builds behind the merged
   Wave-1/Wave-2 contracts (`WorldStore`, `AgentRuntime`,
   `ExecutionProvider`, `ModelProvider`, `Environment`,
   `resolve_capability`, `compile_context_snapshot`'s public types). No
   trait signature changes. Additive methods require every existing fake
   conformance implementation to keep compiling — if an addition is
   unavoidable, mark it a deviation and justify.
2. **Context is a projection, never a transcript** (the F2 law, now
   load-bearing): a context snapshot must be REBUILDABLE from durable
   task state (world store: tasks/artifacts/observations/evidence/events
   + the context store's memory items) WITHOUT replaying any model
   conversation. The engine may consult, but never require, prior
   snapshots. Model/context switching must never mutate canonical task
   state (the Gate-A law, extended to the engine).
3. **The harness is an explicit state machine, not a loop.**
   `prepare → execute → observe → verify → persist` with the three
   continuations `continue / recover / escalate` as FIRST-CLASS states.
   Every transition is event-sourced (the world event stream). Recovery
   is reconstruction from durable state — never "start over". Reset or
   compaction must be visibly non-destructive: the user's task is never
   silently replaced by a blank one.
4. **Orchestration is a graph of attributed agents, not a chat relay.**
   Agent-to-agent coordination happens through shared task state +
   artifacts/claims/observations/evidence/decisions — never "Agent A →
   chat transcript → Agent B" as the coordination abstraction. Every
   node (agent assignment) carries: who (AgentRef), role, dependencies
   (artifact/resource waits), state (ready/running/blocked/done/failed/
   verified), and attribution on everything it produces. A blocked node
   names WHAT it waits for. An independent verifier node NEVER shares a
   context snapshot with the nodes it verifies (independence is
   structural, not behavioral).
5. **Procedure slice is product, not infrastructure** (the F6 phasing):
   the first successful repeatable task must surface "Save as a reusable
   workflow" (user language; the internal object is a `Procedure`),
   persist a minimal procedure version, and be discoverable again from
   the Workspace nav. No library/search/versions UI (F10). The seven
   layers apply.
6. **Credentials stay references; canonical JSON discipline; task
   identity is sacred** (Wave-2 addendum §3/§5/§6 — unchanged, enforced).
7. **GUI slices follow the seven-layer rule** (PRODUCT-UX-JOURNEYS §1).
   The palette is never the only discovery mechanism. Recovery language
   is non-technical (never "context window", "token budget",
   "rehydration"; say "picking up where we left off", "what we kept",
   "what was summarized").

---

## ORCH-002 — Context Engine: rebuild-without-replay, tiered memory, compaction

```
ID: ORCH-002
Title: The context engine — compile from durable task state, tiered
  memory dynamics, structured compaction/reset with provenance, and
  model-aware compilation with dynamic tool exposure
Phase: Wave 3 (F6 orchestration — context)
Owner: Worker A (agents-tab session)
Dependencies: flauz-context merged at e9bee81 (Context/MemoryItem/
  ContextSnapshot/ContextProvenance/ModelContextProfile + the compile
  step); flauz-world (WorldStore reads: tasks/artifacts/observations/
  evidence/events); the F2 gate-A harness pattern
Contract(s): F2-CONTRACT-KERNEL.md; Wave-2 addendum; this addendum §1-§3,
  §6-§7
Problem: snapshots can be compiled from memory items (F2), but nothing
  rebuilds a context from the durable WORLD state of a task; memory tiers
  are static (nothing promotes/demotes); compaction/reset exist as record
  types with no engine semantics; tool schemas are static per profile.
User-visible outcome: after a disconnect/model switch/long absence, the
  task's context is reconstructed from what the task DURABLY holds — the
  same logical task continues with its evidence and artifacts intact
  (J-03's backend); compacted context keeps provenance on every item.
Scope:
  - crates/flauz-context/src/engine.rs (NEW): compile_from_durable_state
    — given a TaskRef + a durable-state view (the task's artifacts,
    observations, evidence, recent events, and memory items — an inputs
    struct, the flauz-cap pattern: the engine takes data, does not
    import flauz-world), produce a ContextSnapshot: the projection. The
    rebuild law: TWO compilations from the same durable state with no
    memory-item mutation must yield equivalent projections (the no-replay
    proof); compiling for model B must not mutate any input.
  - crates/flauz-context/src/tiers.rs (NEW): tier dynamics — promotion/
    demotion rules (HOT stays bounded; WARM holds referenced; COLD is
    jit-only; Secret never compiles), the compaction planner: given a
    snapshot + a budget (per ModelContextProfile), produce a compacted
    snapshot where every retained item keeps its provenance and every
    summarized item is NAMED in a compaction record (a ContextReset with
    reason kind "compaction" — additive enum variants are a deviation-
    free zone ONLY if the existing variants stay; otherwise new kind
    types).
  - crates/flauz-context/src/tools.rs (NEW): dynamic tool exposure —
    given a ModelContextProfile (tool-schema handling) + a capability
    admission list (the flauz-cap resolution record as DATA), compute the
    tool schemas a model sees: excluded tools are NAMED (never silently
    dropped), schema-format conversion per profile.
  - public fakes extended (durable-state fixture families); fixtures per
    entity family (typical/minimal/invalid); conformance tests.
Non-goals: NO UI (the J-02/J-03 surfaces exist; ORCH-003 wires the
  recovery UX), NO retrieval/embedding/vector store (later waves), NO
  flauz-world imports (inputs as data — the flauz-cap pattern), NO trait
  signature changes, NO real model calls.
Files/subsystems owned: crates/flauz-context/src/{engine,tiers,tools}.rs;
  crates/flauz-context/src/{fakes.rs,lib.rs} (additive only);
  crates/flauz-context/tests/** (conformance + fixtures under
  tests/fixtures/w3/); Cargo.toml only if module lines are needed.
Inputs: the merged flauz-context crate; the F2 gate-A harness context
  step (compile_context_snapshot) as the pattern; flauz-cap's
  CapabilityResolution record shape (as frozen format strings — grammar
  re-pinned in tests, the CAP-001 pattern)
Outputs/artifacts: the engine modules + conformance tests + fixtures
Tests: the rebuild law (two compilations, no replay, equivalence);
  compile-for-model-B never mutates inputs; tier promotion/demotion
  bounds; compaction: provenance retained, summarized items named,
  budget respected per profile; tool exposure: excluded tools named,
  per-profile schema handling; canonical JSON + fixtures; no credential
  material; determinism
GUI/lab evidence: Lead gate — the Wave-3 harness step: durable-state →
  engine → snapshot → model switch → recompile → same logical task,
  provenance everywhere
UX journey IDs: J-03 (backend), J-02 (context view stays honest)
Primary discovery surface: n/a (contracts; the recovery UX is ORCH-003)
Contextual discovery surface: n/a
Search/palette discovery: n/a
Empty/success-state behavior: n/a
Acceptance criteria:
  1. Single clean commit on feat/orch-002-context-engine at base e9bee81;
     owned files only.
  2. The rebuild law proven: context reconstructs from durable state
     with NO model-conversation replay (test: same durable state, fresh
     process-equivalent path → equivalent projection).
  3. Compaction never silently drops: provenance retained, summaries
     named, budgets per profile.
  4. Tool exposure names its exclusions; per-profile handling honored.
  5. Compile-for-model-B leaves every input unmutated (the F2 law at the
     engine level).
  6. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive modules; revert the commit.
Integration notes: the engine is what ORCH-003's harness prepare-state
  calls (through the existing public types OR the new engine API —
  ORCH-003 freezes against the e9bee81 surface + this order's public
  API). Keep the engine deterministic; the no-flauz-world-import rule
  holds (inputs as data).
Status: MERGED (2026-09-23) — PR #49 (merge cd59421): worker commit a9401da
  (bundle-verified, base 19d16d5ab416 exact — the worker resolved the
  prompt's 39-char SHA typo to the true base, documented in its report).
  Gates: fmt/clippy clean; flauz-context 83/83; CI GREEN both platforms.
  Deviations: NONE. Evidence:
  evidence/w3-orch-002/ORCH-002-COMPLETION-REPORT.md
```

---

## ORCH-003 — The harness: state machine, telemetry, and the recovery UX

```
ID: ORCH-003
Title: The execution harness — prepare/execute/observe/verify/persist
  with continue/recover/escalate as first-class states, event-sourced
  telemetry, and the non-technical recovery surface (J-03)
Phase: Wave 3 (F6 orchestration — harness/recovery)
Owner: Worker B (agents-tab session)
Dependencies: flauz-exec merged at e9bee81 (AgentRuntime, fakes, store);
  flauz-context at e9bee81 (compile_context_snapshot + public types);
  flauz-cap (resolver, inputs as data); the shell (flauz_shell patterns)
Contract(s): F2-CONTRACT-KERNEL.md; Wave-2 addendum; this addendum §1-§3,
  §6-§7
Problem: execution today is implicit (whoever drives a runtime drives
  it); there is no explicit harness state machine, no recovery state, no
  telemetry — and when context is compacted or a session reconnects, the
  product has nothing to show.
User-visible outcome: a task that hits interruption/pressure shows
  "Picking up where we left off" with what was kept/summarized (non-
  technical language), resumes from durable state, and escalation to the
  human is an explicit visible state (J-03); the harness records every
  transition on the task's event stream.
Scope:
  - crates/flauz-exec/src/harness.rs (NEW): the harness state machine —
    HarnessState {Preparing, Executing, Observing, Verifying, Persisting,
    Continuing, Recovering, Escalated, Done, Failed} + transitions:
    prepare (attach model/runtime/environment via the EXISTING public
    types; compile context via the EXISTING flauz-context compile step),
    execute (a turn through the Box<dyn AgentRuntime> the caller
    supplies), observe (record an observation), verify (claim →
    evidence, the world pattern), persist (the store seam), and the
    three continuations. Every transition emits an Event to the task
    stream (existing Event types; new type names need the frozen-format
    discipline). Recovery = reconstruct HarnessState from the event
    stream (replay of TRANSITIONS — the machine's own history — never of
    model output).
  - crates/flauz-exec/src/harness_telemetry.rs (NEW): the telemetry
    projection — a bounded, canonical-JSON serializable trace of a
    harness run (transitions + durations + outcomes), replayable into a
    display model.
  - crates/codex-app/src/ui/flauz_recovery.rs (NEW): the recovery
    surface (shell-family): a task-surface banner when HarnessState is
    Recovering/Escalated — "Picking up where we left off" / what was
    kept (durable artifacts, evidence, memory) / what was summarized
    (the compaction record) / "Review and continue" + "Escalate to me"
    actions; the escalation state is explicit and visible (J-08's
    backend marker); honest empty state; keyboard path.
  - ui.rs: ONLY the named seams (declaration + registrations) — distinct
    from MOD-001's and CAP-001's seams.
  - F1 a11y findings carried: the recovery banner must NOT trap keyboard
    focus (the d19 modal-trap lesson: scoped Escape, focus returns to
    the task surface); the PTY focus-transfer discipline (d21) applies
    when the recovery path lands on a terminal surface.
Non-goals: NO context-engine internals (call the EXISTING compile step;
  ORCH-002's engine wires in later behind the same seam), NO multi-agent
  graph (ORCH-004), NO real provider integrations, NO trait signature
  changes, NO Procedure work.
Files/subsystems owned: crates/flauz-exec/src/{harness,harness_telemetry}
  .rs; fakes.rs (additive only — a FakeHarnessObserver or similar);
  lib.rs module lines; crates/flauz-exec/tests/fixtures/w3/**; crates/
  codex-app/src/ui/flauz_recovery.rs; ui.rs NAMED SEAMS ONLY.
Inputs: the F2 gate-A harness (the Lead's pattern); flauz_shell/mod.rs
  (the shell-family discipline); the d19 modal-trap + d21 PTY-focus
  lessons (docs/research/evidence/f1-sweep)
Outputs/artifacts: the harness modules + telemetry + the recovery
  surface + tests + fixtures
Tests: the state machine (legal/illegal transitions, every continuation
  reachable); recovery-from-event-stream (reconstruct state after
  serialize → drop → reload; the machine's OWN history replays, model
  output does not); telemetry round-trip; the UI module tests (copy
  rules — no technical terms per addendum §7, banner states, keyboard
  chord, palette row, honest empty state, focus-not-trapped)
GUI/lab evidence: Lead gate — lab scenes at the merged binary: J-03
  recovery banner (the non-technical copy), keyboard path, focus return;
  escalation visible
UX journey IDs: J-03 (recover/continue — the wave's core UX), J-08
  (takeover/handoff — the escalation marker), J-17 (activity — the
  harness events surface in the activity feed)
Primary discovery surface: the recovery banner on the task surface
  (state-driven, always visible while Recovering/Escalated)
Contextual discovery surface: the "what was kept / what was summarized"
  disclosure panel
Search/palette discovery: a palette row ("Resume this task where it left
  off") when the task is in a recoverable state
Empty/success-state behavior: nothing to recover → no banner (the task
  surface is unchanged — never a permanent banner); recovered → "Resumed
  from <n> events · everything kept except <the compaction record>" with
  the next step; escalated → explicit "Needs you" state with the action
Keyboard path: Ctrl+Alt+Shift+R (the recovery chord) + scoped Escape;
  focus returns to the task surface on close (never trapped)
Acceptance criteria:
  1. Single clean commit on feat/orch-003-harness at base e9bee81;
     owned files only.
  2. The state machine: every transition event-sourced; recover
     reconstructs from the machine's own history; no model-output
     replay.
  3. The recovery UX: seven layers; non-technical copy (tested);
     keyboard never trapped (the d19 discipline).
  4. Canonical JSON + fixtures; no credential material; deterministic.
  5. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive modules + seams; revert the commit.
Integration notes: ORCH-002's engine later replaces the compile seam's
  internals (same public types). The harness is what ORCH-004's graph
  nodes drive (a node's execution = a harness run); keep the HarnessState
  exportable as plain data for ORCH-004's view-model.
Status: DISPATCHED (2026-09-23)
```

---

## ORCH-004 — Execution graph: multi-agent orchestration + the Procedure slice

```
ID: ORCH-004
Title: The reactive execution graph — agent roles/dependencies/blocked
  states, parallel agents with shared artifacts + independent
  verification, the who/role/dependencies/blocked/verified task view
  (J-07), and the first-class "Save as a reusable workflow" slice (J-10/
  J-11)
Phase: Wave 3 (F6 orchestration — the graph + the product slice)
Owner: Worker C (agents-tab session)
Dependencies: flauz-world at e9bee81 (Task/Artifact/Evidence/Procedure/
  Event + WorldStore); flauz-exec at e9bee81 (Agent, AgentRuntime,
  fakes); the shell (the task rail's Agents panel + the Workspace nav)
Contract(s): F2-CONTRACT-KERNEL.md; Wave-2 addendum; this addendum §1-§7
Problem: nothing models multiple agents cooperating on one task with
  roles, dependencies, blocked states, and independent verification —
  and the successful multi-agent task cannot be saved as reusable work
  (the Procedure contract exists but no surface reaches it).
User-visible outcome: a task can hold Agent A (research/browser) +
  Agent B (analysis/terminal) + Agent C (independent reviewer) running
  in parallel where B waits on A's artifact and C verifies both — the
  user sees who is doing what, what is blocked on what, and what has
  been independently verified, WITHOUT reading any transcript (J-07);
  after success, "Save as a reusable workflow" persists the shape and it
  is discoverable from the Workspace nav (J-10/J-11 — the F6 phasing).
Scope:
  - crates/flauz-orch/** (NEW self-contained crate, the flauz-cap
    pattern: serde-only, inputs as data, no contract-crate imports):
    the orchestration graph — AgentAssignment (who: an actor ref string
    + role label, dependencies: artifact-wait/resource-wait edges,
    state: ready/running/blocked/done/failed/verified), the graph
    evaluator (advance: unblock when waits clear; attribute every
    produced artifact/evidence to its node), the independence rule (a
    verifier node's inputs are the ARTIFACTS/EVIDENCE it verifies —
    structurally never a context snapshot of the verified node), the
    merge point (all-done → task-level verification event). Canonical
    JSON + fixtures per family; public fakes (a deterministic 3-node
    research/analysis/review graph + a blocked-resolution case).
  - crates/codex-app/src/ui/flauz_agents_view.rs (NEW): the task rail's
    Agents panel upgrade (the shell-family pattern): the node list with
    who/role/state chips (ready/running/blocked/done/verified — user
    language: "Waiting for the research notes"), per-node dependency
    disclosure ("blocked until: Analysis notes artifact"), the
    verification badges (independently verified vs claimed), the
    parallel-progress view (J-07: "watch parallel progress"); honest
    empty state (no agents → what agents add + how to add one);
    keyboard path.
  - crates/codex-app/src/ui/flauz_save_workflow.rs (NEW): the Procedure
    slice — the "Save as a reusable workflow" affordance on a successful
    task (the task surface's next-step area after Done+verified), the
    save flow (name → the minimal ProcedureVersion record via the
    world-store seam: steps from the harness/graph shape, inputs, the
    resource bindings), the Workspace nav's "Reusable workflows" surface
    upgrade: from empty state to listing saved workflows with "Run
    again" (a NEW task seeded from the procedure — same identity
    discipline: a new task, never a fork of the old one), and the
    deviation hint (a run that diverges says so — the full library is
    F10). User language ONLY ("reusable workflow", never "Procedure").
  - ui.rs: ONLY the named seams — distinct from MOD-001/CAP-001/
    ORCH-003's seams.
Non-goals: NO Procedure library/search/versions/sharing UI (F10), NO
  real provider/model calls (fakes drive the graph), NO harness
  internals (the graph references harness states as plain data — the
  ORCH-003 seam), NO permission/policy engine, NO trait changes.
Files/subsystems owned: crates/flauz-orch/**; the workspace Cargo.toml
  members line (one line); crates/codex-app/src/ui/{flauz_agents_view,
  flauz_save_workflow}.rs; ui.rs NAMED SEAMS ONLY; the shell's agents-
  panel + reusable-workflows surface mounts (flauz_shell/mod.rs — the
  existing render seams ONLY, additive)
Inputs: flauz-world's Procedure contract; the shell module patterns;
  PRODUCT-UX-JOURNEYS J-07/J-10/J-11; the F6 phasing note
  (IMPLEMENTATION-ROADMAP)
Outputs/artifacts: the graph crate + the two surfaces + tests + fixtures
Tests: the graph evaluator (unblock on wait-clear; attribution on every
  product; the independence rule structurally enforced; the merge point;
  cancellation propagation — additive); canonical JSON + fixtures; the
  UI module tests (copy rules — user language tested; state chips;
  dependency disclosure; verification badges distinguish verified vs
  claimed; seven layers on both surfaces; keyboard chords)
GUI/lab evidence: Lead gate — lab scenes at the merged binary: J-07 (the
  3-node graph view: who/role/blocked/verified), J-10 (save a successful
  task → the Workspace nav lists it), J-11 (find it later); the F6
  domain-neutral E2E scenario runs at the wave integration gate
UX journey IDs: J-07 (parallelize), J-09 (understand what happened —
  the verification badges), J-10 (save as reusable workflow), J-11
  (discover it later)
Primary discovery surface: the Agents panel on the task rail (upgraded);
  the "Save as a reusable workflow" affordance on the successful task
Contextual discovery surface: per-node dependency disclosure; the
  run-again flow from the Workspace nav
Search/palette discovery: palette rows ("See who is working on this
  task", "Save this task as a reusable workflow")
Empty/success-state behavior: no agents → honest copy + next step;
  running → the live graph; done → the merge + verification summary;
  the workflows nav: empty → what reusable workflows are + how to save
  one; with items → the list with run-again
Keyboard path: Ctrl+Alt+Shift+7 (agents view) + the save-flow chord;
  scoped Escape everywhere; tab-navigable panels
Acceptance criteria:
  1. Single clean commit on feat/orch-004-execution-graph at base
     e9bee81; owned files only.
  2. The graph: dependencies/blocked/attribution/independence/
     merge — all proven by deterministic tests; no chat-relay
     abstraction anywhere.
  3. The two surfaces: seven layers each; user language only (tested);
     verification badges distinguish verified vs claimed (J-09's law).
  4. The save→discover→run-again loop works end-to-end with identity
     discipline (run-again creates a NEW task, never forks the old).
  5. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive crate + modules + seams; revert the commit.
Integration notes: the graph is pure data + evaluator (no engine
  coupling); the F6 E2E scenario (Lead's wave gate) drives it through
  the ORCH-003 harness with fake runtimes at the merged binary. The
  Procedure record is the F10 library's foundation — keep the minimal
  version honest (steps = what actually ran, never aspirational).
Status: DISPATCHED (2026-09-23)
```

---

## Reporting contract (all Wave-3 workers)

The 11-field completion report (exact headers), delivered in the worker's
final message AND via the delivery bundle branch: WO ID(s); base branch +
SHA; branch/commits; changed files/surfaces; implementation summary;
tests/commands + exact results (state plainly if the sandbox lacks the
Rust toolchain — static reasoning is acceptable, the Lead independently
compiles and gates); kernel-compliance checklist (this addendum's §1-§7,
each item); GUI discoverability layers covered (UI-bearing WOs);
acceptance-criteria evidence (map each bullet); known limitations;
contract deviations (NONE if none); follow-up work.

## Wave-3 integration gate (Lead, after all three merge)

Extend the F2/Wave-2 harness (lead-tools/f2-integration-harness): the
F6 domain-neutral scenario end-to-end on fakes — create a research task
→ attach the fake browser environment + fake runtimes → ORCH-004 graph
with research/analysis/review nodes (independent verifier) → ORCH-003
harness drives each node (prepare/execute/observe/verify/persist) →
ORCH-002 engine rebuilds context from durable state mid-run (model
switch) → same logical task → artifacts/evidence attributed and
independently verified → save as a reusable workflow → run-again as a
NEW task → both tasks' identities intact. Plus the lab scenes (J-03/J-07/
J-10/J-11) at the merged binary. The record lands at
evidence/w3-gate/WAVE3-GATE-RECORD.md; F6 closes only then.
