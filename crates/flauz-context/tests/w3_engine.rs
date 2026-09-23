//! The context-engine laws (Wave 3, ORCH-002): the rebuild law (two
//! compilations from the same durable state — no model-conversation
//! replay — yield equivalent projections), the never-mutates law at the
//! engine level (compiling for model B leaves every input unmutated),
//! and the projection's selection and determinism laws.

use std::fmt;

use flauz_context::fakes::{
    fake_minimal_durable_state, fake_tool_catalog, fake_typical_durable_state,
};
use flauz_context::profile::{ModelContextProfile, MultimodalBehavior, ToolSchemaHandling};
use flauz_context::provenance::ContextSource;
use flauz_context::{
    ActorRef, AuthorizationClass, ContextSnapshot, DurableStateInputs, MemoryItemId, MemoryTier,
    ModelRef, Timestamp, compile_from_durable_state, projections_equivalent,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

const MODEL_A: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPRE";
const MODEL_B: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPSF";
const TASK: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";

fn profile_for(model: &str, capacity: u64) -> ModelContextProfile {
    test_ok(ModelContextProfile::new(
        test_ok(ModelRef::parse(model)),
        capacity,
        MultimodalBehavior::TextOnly,
        ToolSchemaHandling::LazyPerTool,
        test_ok(ActorRef::system("flauz-fake")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ))
}

fn engine_actor() -> ActorRef {
    test_ok(ActorRef::system("flauz-context-engine"))
}

fn compile(inputs: &DurableStateInputs, model: &str, at: &str) -> ContextSnapshot {
    test_ok(compile_from_durable_state(
        inputs,
        &profile_for(model, 200_000),
        engine_actor(),
        test_ok(Timestamp::parse(at)),
    ))
}

/// THE REBUILD LAW (acceptance criterion 2): two compilations from the
/// same durable state with no memory-item mutation yield equivalent
/// projections — the context reconstructs from durable state WITHOUT
/// replaying any model conversation.
///
/// The second compilation runs the process-equivalent fresh path: the
/// durable state is serialized to canonical JSON, everything in memory
/// is dropped, the state is reloaded from the document, and a fresh
/// compilation runs — exactly what a reconnecting or model-switching
/// process does. There is no conversation to replay: the engine's only
/// input is the durable state, and no model output can reach the
/// projection (the frozen provenance vocabulary attributes every item
/// to a durable source kind).
#[test]
fn rebuild_law_two_compilations_yield_equivalent_projections() {
    let inputs = test_ok(fake_typical_durable_state());
    let first = compile(&inputs, MODEL_A, "2026-09-21T13:46:00Z");

    // The process-equivalent fresh path: serialize → drop → reload.
    let serialized = test_ok(serde_json::to_string(&inputs));
    drop(inputs);
    let reloaded: DurableStateInputs = test_ok(serde_json::from_str(&serialized));

    let second = compile(&reloaded, MODEL_A, "2026-09-22T09:03:00Z");

    // Equivalent projections: identical content, fresh compilation
    // metadata (new snapshot ID, new timestamp).
    assert!(
        projections_equivalent(&first, &second),
        "two compilations from the same durable state must yield equivalent projections"
    );
    assert_eq!(first.items, second.items);
    assert_ne!(first.id, second.id);
    assert_ne!(first.compiled_at, second.compiled_at);
    test_ok(first.validate());
    test_ok(second.validate());

    // No model conversation anywhere: every item is attributed to a
    // durable source kind (memory item, observation, session event or
    // artifact). The frozen provenance vocabulary has no model-output
    // kind, so replay is structurally impossible — asserted here so the
    // law stays visible at the contract surface.
    let durable_kinds = second
        .items
        .iter()
        .filter(|item| {
            matches!(
                &item.provenance.source,
                ContextSource::MemoryItem { .. }
                    | ContextSource::ResourceObservation { .. }
                    | ContextSource::SessionEvent { .. }
                    | ContextSource::Artifact { .. }
            )
        })
        .count();
    assert_eq!(
        durable_kinds,
        second.items.len(),
        "every projected item must attribute to a durable source"
    );
}

/// Compiling for model B must not mutate ANY input (acceptance
/// criterion 5, the F2 law at the engine level): the inputs are taken by
/// shared reference, and their canonical serialization is byte-identical
/// before and after compiling for models A and B.
#[test]
fn compile_for_model_b_leaves_every_input_unmutated() {
    let inputs = test_ok(fake_typical_durable_state());
    let before = test_ok(serde_json::to_string(&inputs));
    let inputs_copy = inputs.clone();

    let for_a = compile(&inputs, MODEL_A, "2026-09-21T13:46:00Z");
    let for_b = compile(&inputs, MODEL_B, "2026-09-21T13:47:00Z");

    let after = test_ok(serde_json::to_string(&inputs));
    assert_eq!(before, after, "compilation must never mutate the inputs");
    assert_eq!(inputs, inputs_copy);

    // The same logical task, two model-targeted projections.
    assert_eq!(for_a.task_id, for_b.task_id);
    assert_eq!(for_a.task_id.as_str(), TASK);
    assert_eq!(for_a.model_id.as_str(), MODEL_A);
    assert_eq!(for_b.model_id.as_str(), MODEL_B);
    // The engine's item selection is model-independent; model-awareness
    // enters through the per-profile compaction budget and tool
    // exposure.
    assert_eq!(for_a.items, for_b.items);
    // Different target models are NOT equivalent projections.
    assert!(!projections_equivalent(&for_a, &for_b));
    test_ok(for_a.validate());
    test_ok(for_b.validate());
}

/// The projection is the authorized, reset-eligible subset of the
/// durable state (the engine's selection law): HOT + WARM,
/// task-scoped + workspace-scoped, never COLD (jit-only), never Secret,
/// never another task's memory; world records project at their frozen
/// tiers with their canonical provenance; the ordering is canonical.
#[test]
fn engine_projection_is_the_authorized_reset_eligible_subset() {
    let inputs = test_ok(fake_typical_durable_state());
    let snapshot = compile(&inputs, MODEL_A, "2026-09-21T13:46:00Z");

    // 3 memory items (HOT plan, WARM decision, WARM workspace knowledge)
    // + 2 observations + 2 events + 2 artifacts + 1 evidence = 10 items.
    assert_eq!(
        snapshot.items.len(),
        10,
        "the typical state projects 10 items"
    );

    // Every projected memory item is one of the three in-scope items.
    let memory_ids: Vec<&str> = snapshot
        .items
        .iter()
        .filter_map(|item| match &item.provenance.source {
            ContextSource::MemoryItem { memory_item_id } => Some(memory_item_id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        memory_ids,
        vec![
            "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5",
            "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF6",
            "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF7",
        ],
        "COLD (jit-only), Secret and other-task memory never project"
    );

    // The tier ordering of the projection: memory first (F5 < F6 < F7),
    // then observations (J8 < J9), events (D3 < D4), artifacts
    // (H7 < H8), evidence — canonical entity-ID order per family.
    let tiers: Vec<MemoryTier> = snapshot.items.iter().map(|item| item.tier).collect();
    assert_eq!(
        tiers,
        vec![
            MemoryTier::Hot,  // mem F5 (its own tier)
            MemoryTier::Warm, // mem F6
            MemoryTier::Warm, // mem F7
            MemoryTier::Hot,  // obs J8
            MemoryTier::Hot,  // obs J9
            MemoryTier::Warm, // ev D3
            MemoryTier::Warm, // ev D4
            MemoryTier::Warm, // art H7
            MemoryTier::Warm, // art H8
            MemoryTier::Warm, // evidence
        ]
    );

    // Provenance maps every world record to its canonical entity.
    let sources: Vec<String> = snapshot
        .items
        .iter()
        .map(|item| {
            test_ok(item.provenance.validate());
            match &item.provenance.source {
                ContextSource::MemoryItem { memory_item_id } => memory_item_id.to_string(),
                ContextSource::ResourceObservation { observation_id } => observation_id.to_string(),
                ContextSource::SessionEvent { event_id } => event_id.to_string(),
                ContextSource::Artifact { artifact_id } => artifact_id.to_string(),
                other => panic!("unexpected source {other:?} in the typical projection"),
            }
        })
        .collect();
    assert_eq!(
        sources,
        vec![
            "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5",
            "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF6",
            "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF7",
            "obs_01J8ZQ5V8K3T2B7N6X4R9DQPJ8",
            "obs_01J8ZQ5V8K3T2B7N6X4R9DQPJ9",
            "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
            "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD4",
            "art_01J8ZQ5V8K3T2B7N6X4R9DQPH7",
            "art_01J8ZQ5V8K3T2B7N6X4R9DQPH8",
            // The evidence item attributes to its verification event.
            "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD4",
        ]
    );

    // Every item is authorization-eligible.
    assert!(
        snapshot
            .items
            .iter()
            .all(|item| item.provenance.eligible_for_model_context())
    );
    // Secret-authorized records stay durable but never project.
    assert!(
        inputs
            .memory_items
            .iter()
            .any(|item| item.authorization == AuthorizationClass::Secret)
    );

    // The compiled view of the projection filters nothing further.
    let profile = profile_for(MODEL_A, 200_000);
    let context = test_ok(flauz_context::Context::compile_from_snapshot(
        &snapshot,
        &profile,
        test_ok(Timestamp::parse("2026-09-21T13:48:00Z")),
    ));
    assert_eq!(context.items.len(), snapshot.items.len());
}

/// A memory-item mutation changes the projection (the rebuild law's
/// other side): promoting a COLD archive item to WARM makes it eligible,
/// and the next compilation includes it.
#[test]
fn projection_tracks_memory_mutations() {
    let inputs = test_ok(fake_typical_durable_state());
    let before = compile(&inputs, MODEL_A, "2026-09-21T13:46:00Z");
    assert_eq!(before.items.len(), 10);

    // Promote the COLD archive item into WARM (referenced → WARM holds
    // it). Persisting would version-bump it; the engine only reads.
    let promoted = test_ok(flauz_context::promote_memory_item(
        &inputs.memory_items,
        &test_ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF8")),
    ));
    let mut mutated = inputs.clone();
    mutated.memory_items = promoted;

    let after = compile(&mutated, MODEL_A, "2026-09-21T13:50:00Z");
    assert_eq!(after.items.len(), 11, "the promoted archive now projects");
    assert!(!projections_equivalent(&before, &after));
    // The original inputs were not mutated by the promotion planner.
    assert_eq!(inputs.memory_items[3].tier, MemoryTier::Cold);
}

/// The engine is deterministic and order-independent: reordering every
/// record family in the inputs yields the identical projection (the
/// canonical entity-ID sort), and the minimal (fresh task) state
/// compiles an empty valid projection.
#[test]
fn engine_compilation_is_deterministic_and_order_independent() {
    let inputs = test_ok(fake_typical_durable_state());
    let first = compile(&inputs, MODEL_A, "2026-09-21T13:46:00Z");

    // Reverse every record family.
    let mut reversed = inputs.clone();
    reversed.artifacts.reverse();
    reversed.observations.reverse();
    reversed.evidence.reverse();
    reversed.recent_events.reverse();
    reversed.memory_items.reverse();

    let second = compile(&reversed, MODEL_A, "2026-09-21T13:46:00Z");
    assert_eq!(
        first.items, second.items,
        "the projection is independent of the caller's record order"
    );

    let minimal = test_ok(fake_minimal_durable_state());
    let empty = compile(&minimal, MODEL_A, "2026-09-21T13:46:00Z");
    assert!(empty.items.is_empty());
    test_ok(empty.validate());

    // The tool-catalog fake family stays deterministic too.
    assert_eq!(test_ok(fake_tool_catalog()), test_ok(fake_tool_catalog()));
}

/// The engine rejects invalid inputs honestly: duplicate records within
/// one family and empty or oversized summaries are errors, never silent
/// drops or silent merges.
#[test]
fn engine_rejects_invalid_inputs_honestly() {
    let inputs = test_ok(fake_typical_durable_state());
    let try_compile = |inputs: &DurableStateInputs| {
        compile_from_durable_state(
            inputs,
            &profile_for(MODEL_A, 200_000),
            engine_actor(),
            test_ok(Timestamp::parse("2026-09-21T13:46:00Z")),
        )
    };

    // Duplicate artifact records.
    let mut duplicated = inputs.clone();
    let artifact = duplicated.artifacts[0].clone();
    duplicated.artifacts.push(artifact);
    assert!(try_compile(&duplicated).is_err());

    // An empty summary.
    let mut empty_summary = inputs.clone();
    empty_summary.observations[0].summary = String::new();
    assert!(try_compile(&empty_summary).is_err());

    // An oversized summary.
    let mut oversized = inputs.clone();
    oversized.artifacts[0].summary = "x".repeat(9 * 1024);
    assert!(try_compile(&oversized).is_err());

    // A profile with an out-of-bounds capacity is rejected before any
    // projection runs (the first profile constructor below is invalid,
    // so the compile never happens; the valid-capacity variant compiles).
    assert!(
        ModelContextProfile::new(
            test_ok(ModelRef::parse(MODEL_A)),
            100_000_001,
            MultimodalBehavior::TextOnly,
            ToolSchemaHandling::LazyPerTool,
            test_ok(ActorRef::system("flauz-fake")),
            test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        )
        .is_err()
    );
}
