# ORCH-004 — Completion Report & Lead Gate Record (Wave 3)

**Status: MERGED (PR #50)** — the execution graph (the flauz-orch
crate: attributed agents, dependency waits, the structural independence
rule, the merge point, cancellation propagation) + the two surfaces:
the who/role/blocked/verified Agents view (J-07/J-09) and the
"Save as a reusable workflow" slice (J-10/J-11 — the F6 phasing).

## Lead gate record

- **Delivery**: worker session de83860a (agents tab, GLM-5.3,
  Full-Stack), branch `feat/orch-004-execution-graph`, single clean
  commit `63fd9e0` on base `19d16d5ab416` (exact — the worker
  SURFACED the dispatched 39-char SHA discrepancy for the Lead to
  confirm rather than silently resolving it; the resolution verified
  correct). Bundle harvested from the worker pod
  (`leads-harvest/de83860a`, 70,551 bytes) — `git bundle verify` PASS.
- **Turn-death recovery**: one turn death mid-work; one continuation
  nudge resumed it — the worker completed the full ladder server-side
  (30 files, +6978/−27) and emitted the full report (all 11 fields +
  the seven layers on BOTH surfaces line-by-line).
- **Gate 0 (bundle)**: PASS — requires exactly 19d16d5ab416; one
  commit; 30 files +6978/−27, all within the owned set.
- **Gate 1 (source review)**: PASS — flauz-orch is the flauz-cap
  pattern (serde-only, zero contract-crate imports, grammar
  re-pinned); **the no-chat-relay law is STRUCTURAL**: `WaitOn` carries
  exactly the two shared-state kinds (artifact_ready | resource_ready)
  — a context-snapshot/transcript wait cannot even be parsed; the
  independence rule enforced at graph validation (a verifier's inputs
  ARE the artifacts it verifies; self-verification forbidden);
  attribution on every product (producing-while-blocked rejected);
  the merge point distinguishes verified from merely done (J-09 at
  the task level); cancellation propagation additive (no seventh
  state); **run-again refuses a fork** (RunAgainError::ForkAttempt —
  tested); user language only ("reusable workflow", never "Procedure"
  — tested); the two shell mounts additive (all other sections
  byte-unchanged); Cargo.lock deliberately excluded (the owned-files
  boundary — the Lead adds it).
- **Gate 2 (Lead-local)**: fmt clean; clippy `-p flauz-orch
  --all-targets` clean; `cargo test -p flauz-orch` **45/45** (29 lib +
  8 conformance + 8 graph — matches the worker's claim; the worker
  re-ran its gates from a fresh bundle clone); sibling crates green
  (flauz-cap 31, flauz-world 77); **codex-app `cargo check` EXIT 0**
  (the Lead independent compile of both UI modules + the 14 ui.rs
  seams + the two shell mounts). Worker's UI verification: the
  scratch-crate pattern (full-module check/clippy against the real
  APIs, 0 errors; zero-dependency pure-logic tests 15/15 against the
  REAL ui.rs/shell copies; the explicit grep pass).
- **Lead gate-fix (`b662c4f`)**: the flauz-orch Cargo.lock entry (the
  mechanical package entry the members line implies; the CAP-001
  precedent — the worker's owned-files boundary excluded the lock, the
  Lead adds it so `--locked` builds stay consistent).
- **Contract deviations: NONE** (worker's report — with the base-SHA
  discrepancy honestly surfaced as dispatch metadata, resolution
  confirmed by the Lead).

## Worker's COMPLETION REPORT (verbatim, as recovered)

```text
=== ORCH-004 COMPLETION REPORT ===
- Work Order ID: ORCH-004
- base branch + base SHA: main @ 19d16d5a749849e3e48aef0c9b45db7a4d8e4a2c2 — **the literal SHA as dispatched does not exist in the repository** (`git cat-file` fails; verified across all refs). The unique commit matching the order's own abbreviated form "main @ 19d16d5" is **19d16d5ab41669197a63bb22809b6dada402313b** ("docs(research): Wave-3 base pinned to e9bee81 — … the workers' single clone base carries the Wave-3 orders") — its message names it the Wave-3 worker clone base, and the ORCH-004 ledger criterion itself says "at base 19d16d5". Based on it; flagging the long-SHA discrepancy for the Lead to re-pin.
- BRANCH: feat/orch-004-execution-graph | COMMITS: 63fd9e04eb9a2953c9edb7e3660cc17f1c98a615 (single clean commit, parent = the base above)
- changed files/surfaces: 30 files, +6978/−27:
  - `Cargo.toml` — the workspace members line (one line, `"crates/flauz-orch"`, alphabetical).
  - `crates/flauz-orch/**` (NEW, 25 files) — Cargo.toml (the flauz-cap pattern: serde-only, serde_json dev-only, workspace lints), README.md, src/{lib,refs,node,graph,evaluator,fakes}.rs, tests/{conformance,graph}.rs, fixtures under tests/fixtures/w3/ (agent-assignment, execution-graph, graph-event-sequence, graph-evaluation — typical/minimal/invalid per family).
  - `crates/codex-app/src/ui/flauz_agents_view.rs` (NEW) — the task rail's Agents panel upgrade.
  - `crates/codex-app/src/ui/flauz_save_workflow.rs` (NEW) — the save-as-reusable-workflow slice.
  - `crates/codex-app/src/ui.rs` — ONLY the named seams, each tagged ORCH-004 (module declarations + action imports; PaletteCommand::{SeeWhoIsWorking, SaveReusableWorkflow} with enum/ALL 82→84/title/description/shortcut/icon/group/requires_selected_chat arms; palette dispatch arms; KeyBindings alt-shift-7 + alt-& companion + alt-shift-s; scoped Escape `Some("FlauzSaveWorkflow")`; WorkspaceView state fields + construction; the dispatch navigation-close seam; the task-surface render mount; the two workspace-root on_action handlers). Distinct from MOD-001's/CAP-001's/ORCH-003's seams.
  - `crates/codex-app/src/ui/flauz_shell/mod.rs` — the two additive render mounts only (the Agents rail-section panel body delegates to flauz_agents_view; the Reusable-workflows surface body delegates to flauz_save_workflow; plus their two `use super::…` import lines). All other sections/surfaces byte-unchanged.
  - Cargo.lock deliberately NOT included (the members line mechanically updates it; the lock is outside the owned-files boundary and the Lead's sequential merge regenerates it — the CAP-001 precedent owns only the members line).
- implementation summary (graph crate + the two surfaces):
  - **flauz-orch** (the flauz-cap pattern: serde-only, inputs as data, zero contract-crate imports, grammar re-pinning): `refs.rs` re-validates the frozen canonical-ID grammar (`<kind>_<ULID>`, Crockford — kernel vectors re-pinned) for task_/art_/evd_/res_ plus the frozen actor-ref grammar; `WaitOn` carries exactly the two shared-state kinds (`artifact_ready` | `resource_ready`) — a context-snapshot/trans transcript wait cannot even be parsed (addendum §4's no-chat-relay law, made structural). `node.rs`: AgentAssignment (who/role/inputs/state/verifier scope; harness states as plain opaque strings — the ORCH-003 seam) with the six frozen states. `graph.rs`: validation — unique nodes, known targets, DAG (Kahn), **the independence rule** (a verifier's inputs ARE the artifacts it verifies — it must wait on an artifact from every verified node; self-verification forbidden; the rejection names "never a context snapshot"), blocked honesty. `evaluator.rs`: `advance(graph, events)` — a pure deterministic fold: unblock when waits clear (a blocked node always names what it waits for), attribute every artifact/evidence to exactly one running node (producing while blocked/blocked-state is an error), the derived Verified overlay (done ∧ a finished verifier in scope → verified), **the merge point** (all-done → the `task.verification_merged` descriptor distinguishing verified from merely done — J-09 at the task level), **cancellation propagation** (the `run_canceled` event: non-terminal nodes fail with the cancellation record keeping from-states/settled work/kept attribution; at most one per sequence; the six-state vocabulary gains no `canceled` variant). `fakes.rs`: the deterministic 3-node research/analysis/review run + the blocked-resolution case (+ resolution tail). Registered event vocabulary: `task.verification_merged`.
  - **flauz_agents_view.rs** (J-07/J-09): the who/role/state-chips list in user language ("Waiting for the research notes" — never "blocked-on-artifact-id"), per-node dependency disclosure ("Blocked until: the analysis notes (from Analysis)"), verification badges distinguishing **independently verified** from merely **Finished** (claimed) with the independence explanation, the parallel-progress view, the merge summary with the J-10 cross-link, the honest empty state (what agents add + how to add one) + honest wiring state. View-model pattern (no flauz-orch import possible — codex-app's Cargo.toml is not owned); `set_detail` is the wiring seam. The panel has no focus state of its own — it renders inside the shell's rail panel (FlauzTaskRail scoped Escape; never trapped).
  - **flauz_save_workflow.rs** (J-10/J-11, the F6 phasing — product, not infrastructure): the "Save as a reusable workflow" affordance on the task surface; the save flow (name → the minimal ProcedureVersion re-pinned as view-model data: steps from the ACTUAL run shape, inputs, resource bindings — never aspirational) recording `procedure.created` through the world-store seam; the Workspace nav "Reusable workflows" upgrade (honest empty state → the list with **Run again**); Run again prepares a brand-new task via the app's new-task path seeded from the workflow (a NEW task, never a fork — BeginNewChat clears any previous selection), and the seam-level `run_again` records `task.created` on the new task **and refuses a "new" id equal to the source task (RunAgainError::ForkAttempt)**; the deviation hint ("no silent drift"). User language ONLY — "reusable workflow", never "Procedure" (tested).
- tests/commands and exact results:
  - `cargo fmt --all --check` → **CLEAN** (re-verified from the bundle clone).
  - `cargo clippy -p flauz-orch --all-targets` → **CLEAN, 0 warnings** (re-verified from the bundle clone).
  - `cargo test -p flauz-orch` → **45 passed / 0 failed** (29 lib + 8 conformance + 8 graph; re-run from the bundle clone: 45/45).
  - `cargo test -p flauz-cap -p flauz-world` → GREEN (31 + 77) — sibling crates unaffected by the members line.
  - UI modules (the GUI crate **cannot LINK in this sandbox** — the pre-existing libpipewire gap under codex-app → codex-platform → xcat → pipewire, no root to install; verified at base and matching MOD-001's recorded evidence — stated plainly): verified via the CAP-001 scratch-crate pattern. (1) Scratch CHECK crate (the FULL unchanged modules + the REAL flauz_shell + a minimal stand-in ui.rs parent, against real gpui =0.2.2 + the patched gpui-component + codex-core): `cargo check --all-targets` → **0 errors**; `cargo clippy --all-targets` → **0 real warnings** (the 10 remaining are stand-in artifacts — each grep-verified used by the real ui.rs). It caught and fixed two real bugs (needless `mut` ×2) plus unused imports, an `expect()` (house idiom), and missing dead-code allows. (2) Scratch RUN crate (zero-dependency stripped pure-logic copies with include_str! pointing at REAL copies of the edited ui.rs + flauz_shell/mod.rs): `cargo test` → **15/15 GREEN** (8 agents + 7 save) — the seam assertions verify the real files. (3) Explicit grep pass: all 14 ui.rs seam strings + both shell mounts present; all 18 pre-existing shell/gap/picker test assertion strings intact; the palette ALL-iterating tests unaffected.
  - Bundle verification: `git bundle verify` OK; fresh-repo fetch from the bundle → commit 63fd9e0 with tree identical to the worker tree (6a6592dc9161…); from the bundle clone the flauz-orch gates re-ran green as above.
- kernel compliance checklist (Wave-3 addendum §1–§7, each item):
  1. **Existing signatures frozen** — COMPLIANT: no trait touched, no signature changed; flauz-orch imports no contract crate at all; harness states referenced as plain data (the ORCH-003 seam); the Procedure contract consumed as re-pinned view-model shapes, never redefined.
  2. **Context is a projection, never a transcript** — COMPLIANT: the graph's coordination grammar has no transcript/snapshot kind (structurally unparsable, pinned by test); verifier inputs are artifacts (the independence rule).
  3. **The harness is an explicit state machine** — N/A for this order (ORCH-003's scope): the graph treats harness states as opaque bounded strings only.
  4. **Orchestration is a graph of attributed agents, not a chat relay** — COMPLIANT (this crate's core law): who/role/dependencies/state per node; blocked nodes name what they wait for; attribution on every product; independence structural; no chat-relay abstraction anywhere.
  5. **Procedure slice is product, not infrastructure** — COMPLIANT: "Save as a reusable workflow" persists the minimal version via the world-store seam (procedure.created), discoverable from the Workspace nav, seven layers, user language only; NO library/search/versions UI (F10 not built).
  6. **Credentials stay references; canonical JSON; task identity sacred** — COMPLIANT: no credential material anywhere (the conformance scan proves it over all fixtures + fake outputs); "v":1 / snake_case / deny_unknown_fields / no floats / strict reads / canonical round-trips (the v2 document is rejected); the graph belongs to one task and run-again creates a NEW task (the fork attempt is refused and tested).
  7. **GUI slices follow the seven-layer rule** — COMPLIANT: seven layers on both surfaces (below); the palette is never the only mechanism; no technical recovery language (N/A here); user language tested.
- GUI discoverability layers covered (the seven, one line each, both surfaces):
  - **Agents view**: (1) primary entry — the task rail's labeled "Agents" control (always visible while a task is selected) opening the upgraded panel; (2) contextual affordance — per-node dependency disclosures appear exactly when a node waits, plus the parallel-progress hint when more than one agent is active; (3) palette fallback — the "See who is working on this task" row (WorkspaceShell group; natural queries tested); (4) stateful empty state — "No extra agents on this task yet" + what parallel agents add + how to add one (ask to split the work) + the honest wiring note; (5) success-state continuation — the merge summary ("N independently verified · M finished, not yet independently checked") cross-linking "Save as a reusable workflow"; (6) keyboard path — Ctrl+Alt+Shift+7 (+ the alt-& shifted-symbol companion, the N6 gate-fix family) opens it; the rail panel's scoped Escape closes it; tab-navigable; (7) honest unavailable state — until the graph wiring lands, "Live agent progress is on its way" explains what will appear; no agent is invented.
  - **Save workflow**: (1) primary entry — the labeled "Save as a reusable workflow" control on the task surface (the next-step neighborhood); (2) contextual affordance — the save-flow panel (affordance/chord/palette) and the Run again flow + deviation hint from the Workspace nav; (3) palette fallback — the "Save this task as a reusable workflow" row plus the existing "Reusable workflows" navigation row (J-11's multiple routes); (4) stateful empty state — the nav's "No reusable workflows yet" (what they are + how to save one) and the panel's honest not-wired state (no invented steps); (5) success-state continuation — after save "Saved — find it any time under Reusable workflows"; after run-again a new task prepared from the workflow with the original untouched; (6) keyboard path — Ctrl+Alt+Shift+S opens the flow; the scoped FlauzSaveWorkflow Escape closes it; tab-navigable; (7) honest unavailable state — "Saving isn't wired to live tasks yet" says exactly what will appear and what to do today; nothing aspirational.
- acceptance-criteria evidence (map each of the 5 bullets):
  1. **Single clean commit at the base, owned files only** — 63fd9e0 on feat/orch-004-execution-graph, parent 19d16d5ab41… (the unique 19d16d5 commit; the dispatched long SHA does not exist — see the base field); 30 files, all within the owned set; Cargo.lock excluded per the boundary; working tree clean and complete; NOT pushed.
  2. **The graph: dependencies/blocked/attribution/independence/merge — deterministic tests, no chat relay** — tests/graph.rs + in-crate tests: `blocked_nodes_name_their_waits_and_unblock_exactly_on_clear` (dependencies/blocked, partial + unrelated artifacts never clear a wait), `every_product_is_attributed_to_its_node` (+ producing-while-blocked and duplicate-artifact rejection), `the_independence_rule_is_structural_not_behavioral` (grammar + shape + derived overlay), `the_merge_point_fires_only_when_all_nodes_reach_terminal_success` (+ no-merge cases), `cancellation_propagation_is_additive_and_honest`; `orchestration_is_a_graph_of_attributed_agents_not_a_chat_relay` + the conformance context-snapshot-wait rejection prove the no-chat-relay law structurally.
  3. **The two surfaces: seven layers each; user language only (tested); badges distinguish verified vs claimed** — the seven layers above; the forbidden-term scans run over every copy string + the run-again seed text (never "Procedure", never internal type names/work-order ids); `verification_badges_distinguish_verified_from_claimed` asserts the J-09 law (badge/chip presence + the merge summary's split).
  4. **The save→discover→run-again loop with identity discipline** — `the_save_discover_run_again_loop_works_end_to_end`: set_run_shape → begin_save (the run's EXACT steps staged — never aspirational) → confirm_save (procedure.created, seq 1, source task) → the catalog lists it (the nav renders from it) → run_again(new id) (task.created, seq 2, on the NEW task) → run_again(source id) REFUSED (ForkAttempt); the UI path prepares a brand-new chat seeded from the workflow (never a fork, by the app's own new-task path).
  5. **Non-empty Contract deviations blocks closure** — see below.
- known limitations:
  - The GUI crate cannot link in this sandbox (the pre-existing libpipewire gap) — the two UI modules were verified by the scratch-crate pattern (full-module check/clippy against the real APIs; zero-dependency pure-logic tests against the REAL ui.rs/shell copies; the grep pass); the Lead independently compiles and gates, as the order anticipates.
  - The surfaces render view-models until the wiring lands (codex-app cannot depend on flauz-orch within this order's owned files); the panels show honest wiring states and invent no data; `set_detail` / `set_run_shape` / `run_again` are the documented wiring points.
  - The UI Run again prepares a new chat seeded from the workflow; the durable task.created record lands via the world-store seam when the run wiring exists (the seam-level path is fully tested today); the run-again status copy is static (the app's status channel takes `&'static str`).
  - flauz-orch's v1 records carry no timestamps (determinism comes from the caller's event order); the merge descriptor is a payload — the world store assigns seq/timestamp at append time.
  - Cancellation lands nodes in Failed + the cancellation record (no seventh state — the six-state vocabulary is the addendum's frozen enumeration); failures are local (no cascade) by design.
- contract deviations: NONE. (Kernel-contract-wise: no frozen signature, schema, boundary, or vocabulary was deviated from. The base-SHA discrepancy noted in the base field is dispatch metadata — the dispatched long SHA does not exist in the repository; the base used is the unique commit matching the order's own abbreviated "19d16d5", whose message designates it the Wave-3 worker clone base. Surfaced for the Lead to confirm rather than silently resolved.)
- follow-up work:
  - One owned-files work order to wire the surfaces: add flauz-orch to codex-app's dependencies, populate AgentsViewDetail from `ExecutionGraph::advance` evaluations, populate RunShape from the harness/graph run (the ORCH-003 seam), and record the merge event + run-again task.created through the real world store.
  - The F10 Procedure library/search/versions/sharing + real deviation detection (the deviation hint is honest copy today, by design).
  - The Lead's lab scenes (J-07/J-10/J-11 at the merged binary) and the F6 domain-neutral E2E scenario at the Wave-3 integration gate (research task → parallel research/analysis/review with an independent verifier → attributable, independently verified results → save → run-again as a NEW task).
=== END REPORT ===
```


## Post-merge notes

- The surfaces render view-models until the wiring order lands (one
  owned-files order: add flauz-orch to codex-app's dependencies,
  populate AgentsViewDetail from `ExecutionGraph::advance`, RunShape
  from the harness/graph run, record the merge + run-again events
  through the real world store).
- The Wave-3 integration gate drives the graph end-to-end at the
  merged binary: the F6 domain-neutral E2E scenario (research task →
  parallel research/analysis/review with an independent verifier →
  attributable, independently verified results → save → run-again as
  a NEW task).
- F10 owns the full Procedure library (search/versions/sharing +
  real deviation detection — the deviation hint is honest copy today).
