# ORCH-002 — Completion Report & Lead Gate Record (Wave 3)

**Status: MERGED (PR #49, merge cd59421)** — the context engine:
compile-from-durable-state (the projection, never a transcript; the
rebuild law), tiered memory dynamics + structured compaction (provenance
verbatim on every retained item; every summarized item named; the
superseded snapshot unmutated), and model-aware dynamic tool exposure
(exclusions named; per-profile schema handling).

## Lead gate record

- **Delivery**: worker session fa4f4ba4 (agents tab, GLM-5.3,
  Full-Stack), branch `feat/orch-002-context-engine`, single clean
  commit `a9401da` on base `19d16d5ab416` (exact — the worker RESOLVED
  a 39-char SHA typo in the Lead's prompt to the true base by short
  prefix + commit message + content, documented in its report; the
  resolution verified correct by the Lead). Bundle harvested from the
  worker pod (`leads-harvest/fa4f4ba4`, 54,912 bytes) — `git bundle
  verify` PASS.
- **Turn-death recovery**: the first turn died after the toolchain
  check (2 commands); one continuation nudge resumed it — the worker
  then completed the full ladder server-side (26 files, +5141/−1) and
  emitted the full report (all 11 fields).
- **Gate 0 (bundle)**: PASS — requires exactly 19d16d5ab416; one
  commit; 26 files +5141/−1, all within the owned set
  (crates/flauz-context/**).
- **Gate 1 (source review)**: PASS — additive-only behind the frozen
  signatures (ContextStore/compile_context_snapshot public types
  untouched; all 14 pre-existing f2 tests green); the engine takes
  inputs as DATA (the flauz-cap pattern — zero flauz-world/flauz-exec
  imports; the CapabilityResolution record crosses as frozen format
  strings with the grammar re-pinned in tests); the rebuild law is
  structural (the engine API carries no conversation/model-output
  type); compaction keeps the superseded snapshot unmutated + names
  every summarized item; the ONE removed line in lib.rs is a stale doc
  sentence ("representing compaction is later work") now corrected —
  no code removals; no other crate depends on flauz-context (the
  change is fully contained).
- **Gate 2 (Lead-local + CI)**: fmt clean; clippy `-p flauz-context
  --all-targets` clean; `cargo test -p flauz-context` **83/83** (42 lib
  + 8 f2 conformance + 6 f2 dynamics + 5 w3 conformance + 6 w3 engine +
  11 w3 tiers + 5 w3 tools — matches the worker's claim exactly; the
  worker also independently re-verified from its own delivered bundle
  via a fresh clone). **CI GREEN both platforms** at a9401da (PR #49).
- **Contract deviations: NONE** (worker's report, confirmed by the
  Lead's review).

## Worker's COMPLETION REPORT (verbatim, as recovered)

```text
=== ORCH-002 COMPLETION REPORT ===
- Work Order ID: ORCH-002
- base branch + base SHA: main @ 19d16d5a749849e3e48aef0c9b45db7a4d8e4a2c2 — **note**: the order's literal SHA is 39 hex chars and names no git object (typo). Resolved to the actual commit **19d16d5ab41669197a63bb22809b6dada402313b** ("docs(research): Wave-3 base pinned to e9bee81 … the workers' single clone base carries the Wave-3 orders"), whose short form, message, and Wave-3-order content confirm it is the intended base. The bundle range uses the resolved full SHA.
- BRANCH: feat/orch-002-context-engine | COMMITS: a9401daa0df8e9cba4edd611ccab7a963de376db (single clean commit; 26 files, +5141/−1)
- changed files/surfaces:
  - NEW: `crates/flauz-context/src/engine.rs` (739 lines), `crates/flauz-context/src/tiers.rs` (701), `crates/flauz-context/src/tools.rs` (880)
  - ADDITIVE: `crates/flauz-context/src/fakes.rs` (+443: 7 new deterministic fakes, no existing line changed), `crates/flauz-context/src/lib.rs` (module lines + re-exports + one stale doc sentence corrected: "representing compaction is later work" → points at the tiers module)
  - NEW tests: `tests/w3_engine.rs` (6), `tests/w3_tiers.rs` (11), `tests/w3_tools.rs` (5), `tests/w3_conformance.rs` (5)
  - NEW fixtures: `tests/fixtures/w3/**` — 5 families × typical/minimal/invalid = 17 files (durable-state, tool-definitions, capability-admissions, tool-exposure, compaction-record)
  - `Cargo.toml`: NOT touched (module lines live in lib.rs; no new deps). Nothing outside the owned surface was touched.
- implementation summary (engine + tiers + tools design):
  - **engine.rs** — `compile_from_durable_state(&DurableStateInputs, &ModelContextProfile, ActorRef, Timestamp) -> ContextSnapshot`: the projection, never a transcript. Inputs are pure data (the flauz-cap pattern — no flauz-world/flauz-exec imports): task + optional session + artifact/observation/evidence/recent-event records (frozen-format canonical-ID references + bounded summaries) + memory items. `EvidenceRef` is a local validating newtype over `evd_<ULID>` (kernel §2 registry kind). Selection: HOT+WARM, task-scoped + workspace-scoped, never COLD (jit-only), never Secret, never another task's memory; world records project at frozen tiers (observations HOT; events/artifacts/evidence WARM; evidence attributes to its verification event per kernel §7 — the honest mapping within the frozen ten-kind `ContextSource` vocabulary, which was NOT extended). Ordering is deterministic (memory first, then observations/events/artifacts/evidence, each in canonical entity-ID order — order-independent of caller input). Duplicate records rejected; the `MAX_CONTEXT_ITEMS` bound rejects honestly. `projections_equivalent` is the rebuild law's equivalence relation (task/session/model/version/items equality; the snapshot ID, actor and timestamp are compilation metadata).
  - **tiers.rs** — Tier dynamics as pure planners: `promote_memory_item` (one-step ladder, HOT stays bounded at `MAX_HOT_MEMORY_ITEMS`=16 — the oldest other HOT item demotes, the just-promoted item is protected), `demote_memory_item` (honest edge errors at both ladder ends), `enforce_hot_memory_bound` (oldest-first), `age_out_unreferenced_warm` (WARM holds referenced items). Planners change tiers only — versions stay untouched; the +1 belongs to the store's optimistic-concurrency persist path (kernel §3). Compaction (architecture §12, distinct from RESET): `plan_compaction(snapshot, budget, reason, …) -> Compaction{snapshot, record}` — priority-ordered retention (HOT→WARM→COLD, walk stops at the first non-fitting item), retained set preserves the original order, **every retained item keeps its provenance verbatim**, **every summarized item is NAMED** in the `CompactionRecord` (provenance + tier + bounded deterministic digest); retained+summarized partitions the input exactly; the superseded snapshot is retained unmutated; the compacted snapshot is a new `ctxsnap_` entity. `CompactionBudget::from_profile` maps the profile's capacity (budgets per profile; custom budgets via `new`); `estimate_tokens` is integer-only (no floats, no model calls). The compaction record is a **NEW kind type family** — `CompactionReason {pressure, manual}` — because the frozen `ResetReason` has no compaction variant and the addendum §1 rule + my owned-files boundary forbid mutating it (existing variants untouched).
  - **tools.rs** — `compute_tool_exposure(profile, tools, admissions)`: exposed tools presented per the profile's `ToolSchemaHandling` (Full/Summary/Lazy — hand-written strict serde in the `MemoryContent` pattern so `deny_unknown_fields` holds), **exclusions NAMED** (gap-blocked tools carry the joined named-gap reasons "dimension: reason; …"; tools with no resolution recorded carry the explicit "no capability resolution is recorded for <capability>" reason); exposed + excluded covers every input tool exactly once, sorted by name (deterministic, order-independent). The `CapabilityResolution` record crosses the seam as DATA (no flauz-cap import): the capability-key grammar is re-validated locally (`validate_capability_key`) and the five frozen dimension names re-pinned (`CAPABILITY_DIMENSIONS`), with grammar vectors re-pinned in tests (the CAP-001 pattern).
  - **fakes** — `fake_typical/minimal/pressure_durable_state`, `fake_small/large_budget_profile` (40 vs 200 000 tokens), `fake_tool_catalog` (2 admitted, 1 gap-blocked, 1 deliberately unresolved) + `fake_capability_admissions` — fixed canonical IDs, fixed timestamps, zero entropy.
- tests/commands and exact results (run on the pinned toolchain — rustup **1.97.1 + clippy + rustfmt installed in the sandbox**, so no static-verification fallback was needed):
  - `cargo fmt --all --check` → **clean**
  - `cargo clippy -p flauz-context --all-targets` → **0 warnings, 0 errors**
  - `cargo test -p flauz-context` → **83 passed / 0 failed** (42 lib incl. 12 new unit tests, 8 f2 conformance, 6 f2 dynamics — all pre-existing tests still green; 6 w3_engine, 11 w3_tiers, 5 w3_tools, 5 w3_conformance)
  - `git bundle verify /home/z/my-project/ORCH-002-delivery.bundle` → okay (contains a9401da on feat/orch-002-context-engine; requires base 19d16d5)
  - Independent re-verification from the delivered bundle: fresh clone → fetch bundle → checkout branch → fmt/clippy/test all green again (83/83).
- kernel compliance checklist (Wave-3 addendum §1–§7, each item):
  - **§1 Existing signatures frozen**: ✔ no trait or public-signature changes; `ContextStore`, `compile_context_snapshot`'s public types (`Context::compile_from_snapshot`, `ContextSnapshot`, …) untouched; lib.rs/fakes.rs changes purely additive; no fake conformance implementation broke (all 14 pre-existing f2 tests green).
  - **§2 Context is a projection, never a transcript / rebuild without replay**: ✔ proven by `rebuild_law_two_compilations_yield_equivalent_projections` (serialize → drop → reload → compile → `projections_equivalent`), plus the structural no-replay proof (the engine API carries no conversation/model-output type; every projected item attributes to a durable source kind). The engine never requires prior snapshots and never mutates canonical task state (model-B test: byte-identical inputs, same task).
  - **§3 Harness is an explicit state machine / reset-compaction visibly non-destructive**: ✔ (context-side portion) — compaction is non-destructive (superseded snapshot retained unmutated; every summarized item named; within-budget compaction retains everything in order); ORCH-003 owns the machine itself (non-goal here).
  - **§4 Orchestration graph / verifier independence**: n/a for this order (ORCH-004) — no chat-relay abstraction introduced anywhere; evidence attribution goes through durable events, never transcripts.
  - **§5 Procedure slice is product**: n/a (no Procedure work in scope).
  - **§6 Credentials stay references; canonical JSON; task identity sacred**: ✔ no credential material (test walks every w3 fixture); references only (`flauz-secrets://…` locations, never material); canonical JSON throughout ("v":1, snake_case, `deny_unknown_fields`, no floats, RFC3339-Z, kind-tagged enums; fixtures round-trip byte-identical); task identity never forked (every test asserts the single `task_` ID).
  - **§7 Seven-layer GUI rule**: n/a (contracts only — recovery UX is ORCH-003; the order's discovery fields are n/a).
- acceptance-criteria evidence (map each of the 6 bullets):
  1. **Single clean commit, owned files only** — a9401da on feat/orch-002-context-engine at base 19d16d5 (full: 19d16d5ab41669197a63bb22809b6dada402313b; see the SHA note above); 26 files, all within the owned surface; working tree clean.
  2. **The rebuild law proven** — `rebuild_law_two_compilations_yield_equivalent_projections`: same durable state, the process-equivalent fresh path (serialize → drop → reload → compile), equivalent projections, NO model-conversation replay (structurally impossible: durable-source attribution asserted on every item).
  3. **Compaction never silently drops** — `compaction_retains_provenance_and_names_every_summarized_item`: provenance kept verbatim on every retained item; every summarized item NAMED (provenance + tier + bounded summary); retained+summarized = 6 = partition of the input items; budget respected (Σ estimate_tokens ≤ max_tokens, count ≤ max_items, per profile via `CompactionBudget::from_profile`); priority + nothing-fits edge (everything named) + within-budget no-op all tested.
  4. **Tool exposure names its exclusions; per-profile handling honored** — `exposure_names_its_exclusions_never_silently_drops` (2 exposed + 2 excluded = 4 input tools, honest gap reasons and the explicit no-resolution reason) and `per_profile_schema_handling_is_honored` (Full/Summary/Lazy under the three frozen handlings; full schemas not inlined in summary mode).
  5. **Compile-for-model-B leaves every input unmutated** — `compile_for_model_b_leaves_every_input_unmutated`: byte-identical canonical serialization before/after compiling for models A and B, clone equality, same task, different model targets, both snapshots valid (reinforced structurally by `&`-only inputs).
  6. **Contract deviations** — NONE (see below).
- known limitations:
  - The bundle command in the order used a 39-hex-char SHA that names no object; I used the resolved base 19d16d5ab41669197a63bb22809b6dada402313b (documented above) so the range covers exactly the one commit.
  - `estimate_tokens` is a deterministic byte/4 heuristic, not a model tokenizer (budgets are planner-level, honest, integer-only; a real tokenizer can later refine the estimator behind the same seam).
  - The compaction walk stops at the first non-fitting item (deterministic, explainable); a budget tighter than one item summarizes everything — honestly named, and the superseded snapshot retains the full content — but the compacted snapshot is then empty (escalation is ORCH-003's explicit state).
  - Summaries are bounded deterministic first-line digests, not model-generated summaries (no model calls is a non-goal boundary); provenance keeps every summarized item fully traceable.
  - The engine does not consult prior snapshots at all (permitted — "may consult, never require"); retrieval/embedding/ranking remain later waves, so the projection is the eligible durable state, not a relevance-ranked subset.
  - Evidence items attribute to their verification event because the frozen ten-kind provenance vocabulary has no evidence kind; extending that vocabulary would be a kernel change (deliberately not taken).
- contract deviations: NONE
- follow-up work:
  - ORCH-003: wire the engine's `compile_from_durable_state` and the compaction record into the harness prepare/recovery states and the "what was kept / what was summarized" recovery UX (J-03), and persist `CompactionRecord`s through the context store seam (an append-only audit list mirroring `resets`).
  - Wave-3 gate (Lead): the durable-state → engine → model switch → recompile → same-logical-task step at the merged binary.
  - Later waves: retrieval/JIT COLD loading, ranking/reranking, a real tokenizer-backed estimator, tool-overlap detection (§11), and a possible kernel amendment adding an `evidence` provenance kind.
=== END REPORT ===
```


## Post-merge notes

- The engine is what ORCH-003's harness prepare-state and the J-03
  recovery UX ("what was kept / what was summarized") wire against; the
  CompactionRecord persistence (append-only audit list mirroring
  `resets`) is an ORCH-003/follow-up seam.
- The Wave-3 integration gate step: durable-state → engine → model
  switch → recompile → same logical task (the worker's rebuild-law test
  is the unit-level proof; the gate runs it end-to-end at the merged
  binary).
- Later waves (recorded by the worker): retrieval/JIT COLD loading,
  ranking/reranking, a real tokenizer-backed estimator,
  tool-overlap detection, and a possible kernel amendment adding an
  `evidence` provenance kind (the engine honestly maps evidence to its
  verification event within the frozen ten-kind vocabulary today).
