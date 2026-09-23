//! The tier-dynamics and compaction laws (Wave 3, ORCH-002): HOT stays
//! bounded, WARM holds referenced items, COLD is jit-only, Secret never
//! compiles; compaction never silently drops (provenance retained,
//! summaries named, budgets per model profile).

use std::collections::BTreeSet;
use std::fmt;

use flauz_context::fakes::{
    fake_large_budget_profile, fake_pressure_durable_state, fake_small_budget_profile,
    fake_typical_durable_state,
};
use flauz_context::provenance::{AuthorizationClass, ContextProvenance, ContextSource};
use flauz_context::{
    ActorRef, CompactionBudget, CompactionReason, ContextItem, ContextItemContent, ContextSnapshot,
    ContextSnapshotId, DurableStateInputs, MemoryContent, MemoryItem, MemoryItemId, MemoryTier,
    ModelContextProfile, ModelRef, SessionRef, TaskRef, Timestamp, age_out_unreferenced_warm,
    compile_from_durable_state, demote_memory_item, enforce_hot_memory_bound, estimate_tokens,
    plan_compaction, promote_memory_item,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

const TASK: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";
const SESSION: &str = "sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6";
const MODEL_A: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPRE";
const SNAPSHOT: &str = "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5";

/// Valid two-character ULID tail codes for fixed test memory IDs
/// (Crockford Base32, no `I`, `L`, `O`, `U`).
const MEM_CODES: [&str; 24] = [
    "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "AB", "AC", "AD", "AE", "AF", "AG",
    "AH", "AJ", "AK", "AM", "AN", "AP", "AQ", "AR",
];

fn mem_id(n: usize) -> MemoryItemId {
    test_ok(MemoryItemId::parse(&format!(
        "mem_01J8ZQ5V8K3T2B7N6X4R9DQP{}",
        MEM_CODES[n]
    )))
}

fn hot_memory(n: usize, created_at: &str) -> MemoryItem {
    memory(n, MemoryTier::Hot, created_at)
}

fn warm_memory(n: usize, created_at: &str) -> MemoryItem {
    memory(n, MemoryTier::Warm, created_at)
}

fn cold_memory(n: usize, created_at: &str) -> MemoryItem {
    memory(n, MemoryTier::Cold, created_at)
}

fn memory(n: usize, tier: MemoryTier, created_at: &str) -> MemoryItem {
    test_ok(MemoryItem::new(
        mem_id(n),
        Some(test_ok(TaskRef::parse(TASK))),
        tier,
        AuthorizationClass::Task,
        MemoryContent::Text {
            text: format!("memory item {n}: the ticket reconciliation state"),
        },
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        test_ok(Timestamp::parse(created_at)),
    ))
}

/// Projects one memory item as the engine does (content + tier +
/// provenance), for building fixed-ID snapshots by hand.
fn memory_item_context(item: &MemoryItem) -> ContextItem {
    let content = match &item.content {
        MemoryContent::Text { text } => ContextItemContent::Text { text: text.clone() },
        MemoryContent::Reference { reference } => ContextItemContent::Reference {
            reference: reference.clone(),
        },
    };
    test_ok(ContextItem::new(
        content,
        item.tier,
        test_ok(ContextProvenance::new(
            ContextSource::MemoryItem {
                memory_item_id: item.id.clone(),
            },
            item.authorization,
        )),
    ))
}

/// The pressure snapshot: a fixed-`ctxsnap` snapshot of the pressure
/// durable state (two HOT items, four WARM items, 16 estimated tokens
/// each), so compaction outcomes are fully deterministic.
fn pressure_snapshot() -> ContextSnapshot {
    let items = test_ok(fake_pressure_durable_state())
        .memory_items
        .iter()
        .map(memory_item_context)
        .collect();
    test_ok(ContextSnapshot::new(
        test_ok(ContextSnapshotId::parse(SNAPSHOT)),
        test_ok(TaskRef::parse(TASK)),
        Some(test_ok(SessionRef::parse(SESSION))),
        test_ok(ModelRef::parse(MODEL_A)),
        test_ok(ActorRef::system("flauz-context-engine")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        items,
    ))
}

fn small_budget() -> CompactionBudget {
    test_ok(CompactionBudget::from_profile(&test_ok(
        fake_small_budget_profile(),
    )))
}

/// HOT stays bounded: promoting a WARM item into a full HOT tier demotes
/// the OLDEST other HOT item to WARM — never the item just promoted. The
/// planner changes only tiers (versions belong to the store's persist
/// path) and never mutates its inputs.
#[test]
fn promotion_keeps_hot_bounded() {
    // Sixteen HOT items, oldest first.
    let mut items = Vec::new();
    for n in 0..16 {
        items.push(hot_memory(n, "2026-09-21T13:00:00Z"));
    }
    // One WARM item, the promotion target.
    items.push(warm_memory(16, "2026-09-21T13:30:00Z"));
    let before = items.clone();

    let result = test_ok(promote_memory_item(&items, &mem_id(16)));

    // The promoted item is HOT.
    assert_eq!(result[16].tier, MemoryTier::Hot);
    // The oldest HOT item (index 0) demoted to WARM.
    assert_eq!(result[0].tier, MemoryTier::Warm);
    // HOT count is exactly the bound.
    assert_eq!(
        result
            .iter()
            .filter(|item| item.tier == MemoryTier::Hot)
            .count(),
        flauz_context::tiers::MAX_HOT_MEMORY_ITEMS
    );
    // Versions are untouched by the planner (the store bumps them on
    // persist, kernel §3) and the inputs are unmutated.
    for (before_item, after_item) in before.iter().zip(&result) {
        assert_eq!(before_item.version, after_item.version);
    }
    assert_eq!(items, before);
}

/// Promoting a COLD item lands it in WARM (WARM holds referenced items);
/// a second promotion raises it to HOT.
#[test]
fn promotion_from_cold_lands_in_warm() {
    let items = vec![cold_memory(0, "2026-09-21T13:00:00Z")];
    let warmed = test_ok(promote_memory_item(&items, &mem_id(0)));
    assert_eq!(warmed[0].tier, MemoryTier::Warm);
    let heated = test_ok(promote_memory_item(&warmed, &mem_id(0)));
    assert_eq!(heated[0].tier, MemoryTier::Hot);
}

/// The tier ladder is honest at its edges: promoting a HOT item and
/// demoting a COLD item are errors, as is addressing an unknown item.
#[test]
fn ladder_edges_are_rejected_honestly() {
    let items = vec![
        hot_memory(0, "2026-09-21T13:00:00Z"),
        cold_memory(1, "2026-09-21T13:01:00Z"),
    ];
    assert!(promote_memory_item(&items, &mem_id(0)).is_err());
    assert!(demote_memory_item(&items, &mem_id(1)).is_err());
    assert!(demote_memory_item(&items, &mem_id(9)).is_err());
    // HOT → WARM and WARM → COLD demote one step each.
    let demoted = test_ok(demote_memory_item(&items, &mem_id(0)));
    assert_eq!(demoted[0].tier, MemoryTier::Warm);
    let archived = test_ok(demote_memory_item(&demoted, &mem_id(0)));
    assert_eq!(archived[0].tier, MemoryTier::Cold);
}

/// `enforce_hot_memory_bound` demotes the oldest HOT items first until
/// the bound holds, preserving order and versions.
#[test]
fn enforce_hot_bound_demotes_oldest_first() {
    let mut items = Vec::new();
    for n in 0..20 {
        items.push(hot_memory(n, "2026-09-21T13:00:00Z"));
    }
    let before = items.clone();
    let result = test_ok(enforce_hot_memory_bound(&items));
    let hot = result
        .iter()
        .filter(|item| item.tier == MemoryTier::Hot)
        .count();
    assert_eq!(hot, flauz_context::tiers::MAX_HOT_MEMORY_ITEMS);
    // The four oldest (indices 0..=3) are the demoted ones.
    for (index, item) in result.iter().enumerate() {
        let expected = if index < 4 {
            MemoryTier::Warm
        } else {
            MemoryTier::Hot
        };
        assert_eq!(item.tier, expected);
    }
    assert_eq!(items, before);
}

/// WARM holds referenced items: WARM items outside the referenced set
/// age out to COLD; HOT and COLD items are untouched; inputs unmutated.
#[test]
fn warm_holds_referenced_items() {
    let items = vec![
        hot_memory(0, "2026-09-21T13:00:00Z"),
        warm_memory(1, "2026-09-21T13:01:00Z"),
        warm_memory(2, "2026-09-21T13:02:00Z"),
        cold_memory(3, "2026-09-21T13:03:00Z"),
    ];
    let before = items.clone();
    let referenced: BTreeSet<MemoryItemId> = [mem_id(1)].into_iter().collect();
    let result = test_ok(age_out_unreferenced_warm(&items, &referenced));
    assert_eq!(result[0].tier, MemoryTier::Hot);
    assert_eq!(result[1].tier, MemoryTier::Warm, "referenced WARM stays");
    assert_eq!(
        result[2].tier,
        MemoryTier::Cold,
        "unreferenced WARM ages out"
    );
    assert_eq!(result[3].tier, MemoryTier::Cold);
    assert_eq!(items, before);
}

/// COLD is jit-only and Secret never compiles — the tier laws at the
/// engine level: a COLD item joins a projection only after a promotion
/// to WARM, and a Secret item never joins one, even promoted all the way
/// to HOT (tier transitions never change authorization).
#[test]
fn cold_is_jit_only_and_secret_never_compiles() {
    let typical = test_ok(fake_typical_durable_state());
    let profile = test_ok(ModelContextProfile::new(
        test_ok(ModelRef::parse(MODEL_A)),
        200_000,
        flauz_context::MultimodalBehavior::TextOnly,
        flauz_context::ToolSchemaHandling::LazyPerTool,
        test_ok(ActorRef::system("flauz-fake")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ));
    let compile = |inputs: &DurableStateInputs| {
        compile_from_durable_state(
            inputs,
            &profile,
            test_ok(ActorRef::system("flauz-context-engine")),
            test_ok(Timestamp::parse("2026-09-21T13:46:00Z")),
        )
    };

    // COLD and Secret are excluded from the projection.
    let base = test_ok(compile(&typical));
    assert_eq!(base.items.len(), 10);

    // Promote the COLD archive to WARM: it now projects (jit-eligible).
    let promoted = test_ok(promote_memory_item(
        &typical.memory_items,
        &test_ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF8")),
    ));
    let mut warmed = typical.clone();
    warmed.memory_items = promoted;
    let after = test_ok(compile(&warmed));
    assert_eq!(after.items.len(), 11);

    // The Secret item is already HOT in the typical state and still
    // never projects; tier dynamics cannot change its authorization.
    let secret_id = test_ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF9"));
    let secret = warmed
        .memory_items
        .iter()
        .find(|item| item.id == secret_id)
        .unwrap_or_else(|| panic!("the secret memory item exists"));
    assert_eq!(secret.tier, MemoryTier::Hot);
    assert_eq!(secret.authorization, AuthorizationClass::Secret);
    let serialized = test_ok(serde_json::to_string(&after));
    assert!(
        !serialized.contains("flauz-secrets://"),
        "secret references never compile into model context"
    );
}

/// THE COMPACTION LAW (acceptance criterion 3): compaction never
/// silently drops — every retained item keeps its provenance verbatim,
/// every summarized item is NAMED in the compaction record (provenance +
/// tier + bounded summary), retained + summarized partitions the input
/// items exactly, the budget is respected, and the superseded snapshot
/// is retained unmutated.
#[test]
fn compaction_retains_provenance_and_names_every_summarized_item() {
    let snapshot = pressure_snapshot();
    let snapshot_before = snapshot.clone();
    let budget = small_budget();

    let compaction = test_ok(plan_compaction(
        &snapshot,
        &budget,
        CompactionReason::Pressure,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
    ));

    let compacted = &compaction.snapshot;
    let record = &compaction.record;

    // Retained + summarized partitions the input items exactly.
    assert_eq!(compacted.items.len() + record.summarized.len(), 6);
    assert_eq!(record.summarized.len(), 4, "the four WARM items summarize");

    // Retained: the two HOT items, in the snapshot's original order,
    // each with its provenance kept verbatim.
    assert_eq!(compacted.items.len(), 2);
    for (retained, original) in compacted.items.iter().zip(snapshot.items.iter()) {
        assert_eq!(retained.provenance, original.provenance);
        assert_eq!(retained.tier, MemoryTier::Hot);
        assert_eq!(retained.content, original.content);
    }

    // Summarized: every entry is NAMED — provenance kept, tier kept,
    // non-empty bounded summary.
    for (summarized, original) in record.summarized.iter().zip(snapshot.items.iter().skip(2)) {
        test_ok(summarized.validate());
        assert_eq!(summarized.provenance, original.provenance);
        assert_eq!(summarized.tier, MemoryTier::Warm);
        assert!(!summarized.summary.is_empty());
        assert!(summarized.summary.chars().count() <= flauz_context::tiers::MAX_SUMMARY_CHARS);
    }

    // The budget is respected.
    let tokens: u64 = compacted.items.iter().map(estimate_tokens).sum();
    assert!(tokens <= budget.max_tokens);
    assert!(compacted.items.len() as u64 <= budget.max_items);

    // The compacted snapshot is a new immutable entity for the same
    // logical task; the superseded snapshot is unmutated.
    assert_ne!(compacted.id, snapshot.id);
    assert_eq!(compacted.task_id, snapshot.task_id);
    assert_eq!(compacted.session_id, snapshot.session_id);
    assert_eq!(compacted.model_id, snapshot.model_id);
    test_ok(compacted.validate());
    assert_eq!(snapshot, snapshot_before);

    // The record is durable, attributable and round-trips canonically.
    assert_eq!(record.task_id.as_str(), TASK);
    assert_eq!(record.reason, CompactionReason::Pressure);
    assert_eq!(record.superseded_snapshot_id.as_str(), SNAPSHOT);
    assert_eq!(record.budget, budget);
    let serialized = test_ok(serde_json::to_string(record));
    let reloaded: flauz_context::CompactionRecord = test_ok(serde_json::from_str(&serialized));
    assert_eq!(reloaded, *record);
}

/// Retention priority is HOT before WARM (architecture §12: structured
/// state survives first), and the walk stops at the first item that does
/// not fit — including the honest edge where nothing fits at all.
#[test]
fn compaction_priority_preserves_hot_before_warm() {
    let snapshot = pressure_snapshot();

    // A 20-token budget: the first HOT item (16 tokens) fits, the second
    // does not — one HOT item retained, everything else named.
    let tight = test_ok(CompactionBudget::new(256, 20));
    let compaction = test_ok(plan_compaction(
        &snapshot,
        &tight,
        CompactionReason::Pressure,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
    ));
    assert_eq!(compaction.snapshot.items.len(), 1);
    assert_eq!(compaction.snapshot.items[0].tier, MemoryTier::Hot);
    assert_eq!(compaction.record.summarized.len(), 5);

    // A 10-token budget: even the highest-priority item does not fit —
    // everything is summarized and NAMED, nothing silently dropped, and
    // the superseded snapshot (retained unmutated) still holds the full
    // content.
    let impossible = test_ok(CompactionBudget::new(256, 10));
    let empty = test_ok(plan_compaction(
        &snapshot,
        &impossible,
        CompactionReason::Pressure,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
    ));
    assert!(empty.snapshot.items.is_empty());
    assert_eq!(empty.record.summarized.len(), 6, "every item is named");
    test_ok(empty.snapshot.validate());
}

/// A snapshot already within budget compacts to itself: everything
/// retained in the original order, nothing summarized — continuity
/// preserved, visibly.
#[test]
fn compaction_within_budget_retains_everything_in_order() {
    let snapshot = pressure_snapshot();
    let budget = test_ok(CompactionBudget::from_profile(&test_ok(
        fake_large_budget_profile(),
    )));

    let compaction = test_ok(plan_compaction(
        &snapshot,
        &budget,
        CompactionReason::Manual,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
    ));

    assert_eq!(compaction.snapshot.items, snapshot.items);
    assert!(compaction.record.summarized.is_empty());
    assert_eq!(compaction.record.reason, CompactionReason::Manual);
    test_ok(compaction.record.validate());
}

/// Budgets derive per model profile: the profile's context capacity
/// becomes the token budget; the canonical item bound bounds the items.
#[test]
fn compaction_budgets_derive_from_the_model_profile() {
    let small = test_ok(CompactionBudget::from_profile(&test_ok(
        fake_small_budget_profile(),
    )));
    assert_eq!(small.max_tokens, 40);
    assert_eq!(small.max_items, 256);
    let large = test_ok(CompactionBudget::from_profile(&test_ok(
        fake_large_budget_profile(),
    )));
    assert_eq!(large.max_tokens, 200_000);
    // Deriving from an invalid profile fails honestly: a profile whose
    // durable version is zero fails canonical validation.
    let mut broken = test_ok(fake_small_budget_profile());
    broken.version = 0;
    assert!(CompactionBudget::from_profile(&broken).is_err());
}

/// Every tier-dynamics planner is pure: same inputs, same outputs, and
/// the inputs are never mutated (the F2 law, tier level).
#[test]
fn tier_dynamics_are_pure_and_deterministic() {
    let typical = test_ok(fake_typical_durable_state());
    let items = typical.memory_items.clone();
    // The WARM decision item (promotable) and the COLD archive item.
    let warm = test_ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF6"));
    let cold = test_ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF8"));
    let referenced: BTreeSet<MemoryItemId> = [warm.clone()].into_iter().collect();

    let promote_first = test_ok(promote_memory_item(&items, &warm));
    let promote_second = test_ok(promote_memory_item(&items, &warm));
    assert_eq!(promote_first, promote_second);

    let demote_first = test_ok(demote_memory_item(&items, &warm));
    let demote_second = test_ok(demote_memory_item(&items, &warm));
    assert_eq!(demote_first, demote_second);

    let cold_promote_first = test_ok(promote_memory_item(&items, &cold));
    let cold_promote_second = test_ok(promote_memory_item(&items, &cold));
    assert_eq!(cold_promote_first, cold_promote_second);

    let bound_first = test_ok(enforce_hot_memory_bound(&items));
    let bound_second = test_ok(enforce_hot_memory_bound(&items));
    assert_eq!(bound_first, bound_second);

    let aged_first = test_ok(age_out_unreferenced_warm(&items, &referenced));
    let aged_second = test_ok(age_out_unreferenced_warm(&items, &referenced));
    assert_eq!(aged_first, aged_second);

    assert_eq!(items, typical.memory_items);

    // The summarized-item records are pure data with stable summaries.
    let snapshot = pressure_snapshot();
    let one = test_ok(plan_compaction(
        &snapshot,
        &small_budget(),
        CompactionReason::Pressure,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
    ));
    let two = test_ok(plan_compaction(
        &snapshot,
        &small_budget(),
        CompactionReason::Pressure,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
    ));
    // Same inputs → same records; the compacted snapshots differ only in
    // their generated IDs.
    assert_eq!(one.record, two.record);
    assert_eq!(one.snapshot.items, two.snapshot.items);
    assert_ne!(one.snapshot.id, two.snapshot.id);
    let summaries: Vec<&str> = one
        .record
        .summarized
        .iter()
        .map(|item| item.summary.as_str())
        .collect();
    assert!(summaries.iter().all(|summary| !summary.is_empty()));
}
