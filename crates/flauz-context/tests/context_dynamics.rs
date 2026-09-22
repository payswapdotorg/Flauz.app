//! Context dynamics tests: the behavior-level laws of the context contracts
//! (the world_dynamics / exec_dynamics pattern): reset reconstruction,
//! authorization-aware filtering, tier eligibility, task-identity
//! preservation across model switches, and the session/context separation.

use std::fmt;

use flauz_context::fakes::FakeContextStore;
use flauz_context::profile::{ModelContextProfile, MultimodalBehavior, ToolSchemaHandling};
use flauz_context::provenance::{AuthorizationClass, ContextProvenance, ContextSource};
use flauz_context::{
    ActorRef, Context, ContextItem, ContextItemContent, ContextSnapshot, ContextSnapshotId,
    ContextStore, ContextStoreError, MemoryContent, MemoryItemId, MemoryItem, MemoryTier,
    ModelRef, ResetReason, SessionRef, TaskRef, Timestamp,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

const TASK: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";
const OTHER_TASK: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQZ9";
const MODEL: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQRE";
const OTHER_MODEL: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPSF";
const MEMORY: &str = "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5";
const SNAPSHOT: &str = "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5";

fn task_ref() -> TaskRef {
    test_ok(TaskRef::parse(TASK))
}

fn profile_for(model: &str) -> ModelContextProfile {
    test_ok(ModelContextProfile::new(
        test_ok(ModelRef::parse(model)),
        200_000,
        MultimodalBehavior::TextOnly,
        ToolSchemaHandling::LazyPerTool,
        test_ok(ActorRef::system("flauz-fake")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ))
}

fn memory(
    id: &str,
    task: Option<&str>,
    tier: MemoryTier,
    authorization: AuthorizationClass,
    text: &str,
) -> MemoryItem {
    test_ok(MemoryItem::new(
        test_ok(MemoryItemId::parse(id)),
        task.map(|task| test_ok(TaskRef::parse(task))),
        tier,
        authorization,
        MemoryContent::Text {
            text: text.to_owned(),
        },
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ))
}

fn snapshot(id: &str, task: &str, model: &str, items: Vec<ContextItem>) -> ContextSnapshot {
    test_ok(ContextSnapshot::new(
        test_ok(ContextSnapshotId::parse(id)),
        test_ok(TaskRef::parse(task)),
        Some(test_ok(SessionRef::parse("sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6"))),
        test_ok(ModelRef::parse(model)),
        test_ok(ActorRef::system("flauz-context-engine")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        items,
    ))
}

/// A reset reconstructs a fresh context from durable task state: the
/// task-scoped and workspace-scoped, authorized, reset-eligible memory items
/// — and nothing else. COLD memory is retrieved just in time and is
/// therefore excluded; secret-authorized memory is never included.
#[test]
fn reset_reconstructs_from_durable_task_state() {
    let mut store = FakeContextStore::new();
    test_ok(store.create_profile(profile_for(MODEL)));
    test_ok(store.create_memory_item(memory(
        MEMORY,
        Some(TASK),
        MemoryTier::Hot,
        AuthorizationClass::Task,
        "current plan: reconcile the ticket counts",
    )));
    test_ok(store.create_memory_item(memory(
        "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF6",
        Some(TASK),
        MemoryTier::Warm,
        AuthorizationClass::Workspace,
        "decision: the CRM export is the source of truth",
    )));
    test_ok(store.create_memory_item(memory(
        "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF7",
        None,
        MemoryTier::Warm,
        AuthorizationClass::Workspace,
        "workspace knowledge: the staging portal is read-only",
    )));
    test_ok(store.create_memory_item(memory(
        "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF8",
        Some(TASK),
        MemoryTier::Cold,
        AuthorizationClass::Task,
        "archived tool result: the full ticket export from last quarter",
    )));
    test_ok(store.create_memory_item(memory(
        "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF9",
        Some(TASK),
        MemoryTier::Hot,
        AuthorizationClass::Secret,
        "secret reference: the portal credentials location",
    )));
    test_ok(store.create_memory_item(memory(
        "mem_01J8ZQ5V8K3T2B7N6X4R9DQPGA",
        Some(OTHER_TASK),
        MemoryTier::Hot,
        AuthorizationClass::Task,
        "another task's memory",
    )));
    test_ok(store.put_snapshot(snapshot(
        SNAPSHOT,
        TASK,
        MODEL,
        Vec::new(),
    )));

    let reconstruction = test_ok(store.reset_context(
        task_ref(),
        test_ok(ModelRef::parse(MODEL)),
        ResetReason::Pressure,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
        test_ok(ContextSnapshotId::parse(SNAPSHOT)),
    ));

    // HOT + WARM, task-scoped + workspace-scoped, authorized only.
    assert_eq!(reconstruction.context.items.len(), 3);
    let texts: Vec<&str> = reconstruction
        .context
        .items
        .iter()
        .map(|item| match &item.content {
            ContextItemContent::Text { text } => text.as_str(),
            ContextItemContent::Reference { reference } => reference.as_str(),
        })
        .collect();
    assert!(texts.contains(&"current plan: reconcile the ticket counts"));
    assert!(texts.contains(&"decision: the CRM export is the source of truth"));
    assert!(texts.contains(&"workspace knowledge: the staging portal is read-only"));
    assert!(!texts.iter().any(|text| text.contains("last quarter")));
    assert!(!texts.iter().any(|text| text.contains("credentials")));
    assert!(!texts.iter().any(|text| text.contains("another task")));

    // Every reconstructed item is attributed to its memory item.
    for item in &reconstruction.context.items {
        let ContextSource::MemoryItem { memory_item_id } = &item.provenance.source else {
            panic!("reset items must be attributed to memory items");
        };
        test_ok(MemoryItemId::parse(memory_item_id.as_str()));
    }

    // The reset is recorded, and the fresh snapshot is durable.
    assert_eq!(reconstruction.reset.reason, ResetReason::Pressure);
    assert_eq!(reconstruction.reset.superseded_snapshot_id.as_str(), SNAPSHOT);
    let fresh = stored(&store, reconstruction.snapshot.id.as_str());
    assert_eq!(fresh.items.len(), 3);
}

/// Compiling the same durable task state for a different model produces a
/// different projection under that model's profile, while the task
/// reference — the task's only identity — never changes (kernel §6).
#[test]
fn task_identity_survives_model_switch() {
    let mut store = FakeContextStore::new();
    test_ok(store.create_profile(profile_for(MODEL)));
    test_ok(store.create_profile(profile_for(OTHER_MODEL)));
    let items = vec![test_ok(ContextItem::new(
        ContextItemContent::Text {
            text: "reconcile the ticket counts".to_owned(),
        },
        MemoryTier::Hot,
        test_ok(ContextProvenance::new(
            ContextSource::UserInput,
            AuthorizationClass::Task,
        )),
    ))];
    test_ok(store.put_snapshot(snapshot(SNAPSHOT, TASK, MODEL, items)));

    // Compile for the first model...
    let first = stored(&store, SNAPSHOT);
    let first_context = test_ok(Context::compile_from_snapshot(
        &first,
        &profile_for(MODEL),
        test_ok(Timestamp::parse("2026-09-21T13:46:00Z")),
    ));

    // ...then switch the model: a reset reconstructs from the same durable
    // task state under the other model's profile.
    let reconstruction = test_ok(store.reset_context(
        task_ref(),
        test_ok(ModelRef::parse(OTHER_MODEL)),
        ResetReason::ModelChanged,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
        first.id.clone(),
    ));

    assert_eq!(first_context.task_id, task_ref());
    assert_eq!(reconstruction.context.task_id, task_ref());
    assert_eq!(reconstruction.context.model_id.as_str(), OTHER_MODEL);
    assert_eq!(
        reconstruction.reset.reason,
        ResetReason::ModelChanged,
        "a model switch is represented as a reset event, never a task fork"
    );
    // Both projections coexist as distinct durable snapshots of one task.
    let state = store.state();
    assert!(state.snapshots.contains_key(&first.id));
    assert!(state
        .snapshots
        .contains_key(&reconstruction.snapshot.id));
    assert_eq!(state.resets.len(), 1);
}

/// A snapshot for one task cannot be superseded by a reset for another
/// task: the store rejects the mismatch honestly.
#[test]
fn reset_rejects_a_snapshot_from_another_task() {
    let mut store = FakeContextStore::new();
    test_ok(store.create_profile(profile_for(MODEL)));
    test_ok(store.put_snapshot(snapshot(SNAPSHOT, TASK, MODEL, Vec::new())));

    let error = store.reset_context(
        test_ok(TaskRef::parse(OTHER_TASK)),
        test_ok(ModelRef::parse(MODEL)),
        ResetReason::Manual,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
        test_ok(ContextSnapshotId::parse(SNAPSHOT)),
    );
    assert_eq!(
        error.err(),
        Some(ContextStoreError::Invalid {
            reason: format!(
                "superseded snapshot {SNAPSHOT} belongs to task {TASK}, not {OTHER_TASK}"
            )
        })
    );
}

/// Authorization-aware filtering is representable end to end: a snapshot
/// may carry secret-authorized items (they are durable, attributable
/// records), but no compiled view ever includes them.
#[test]
fn secret_items_are_storable_but_never_compiled() {
    let secret_item = test_ok(ContextItem::new(
        ContextItemContent::Reference {
            reference: "flausec://portal/credentials".to_owned(),
        },
        MemoryTier::Hot,
        test_ok(ContextProvenance::new(
            ContextSource::ToolResult {
                reference: "art_01J8ZQ5V8K3T2B7N6X4R9DQPH7/secret-lookup".to_owned(),
            },
            AuthorizationClass::Secret,
        )),
    ));
    let open_item = test_ok(ContextItem::new(
        ContextItemContent::Text {
            text: "the dashboard shows 3 open tickets".to_owned(),
        },
        MemoryTier::Hot,
        test_ok(ContextProvenance::new(
            ContextSource::UserInput,
            AuthorizationClass::Task,
        )),
    ));
    let mut store = FakeContextStore::new();
    test_ok(store.create_profile(profile_for(MODEL)));
    test_ok(store.put_snapshot(snapshot(
        SNAPSHOT,
        TASK,
        MODEL,
        vec![open_item.clone(), secret_item],
    )));

    let snapshot = stored(&store, SNAPSHOT);
    assert_eq!(snapshot.items.len(), 2, "the snapshot keeps both records");
    let context = test_ok(Context::compile_from_snapshot(
        &snapshot,
        &profile_for(MODEL),
        test_ok(Timestamp::parse("2026-09-21T13:46:00Z")),
    ));
    assert_eq!(context.items, vec![open_item]);
    test_ok(context.validate());

    // A compiled view that somehow carried an ineligible item would fail
    // canonical validation.
    let mut poisoned = context.clone();
    poisoned.items.push(test_ok(ContextItem::new(
        ContextItemContent::Text {
            text: "should not be here".to_owned(),
        },
        MemoryTier::Hot,
        test_ok(ContextProvenance::new(
            ContextSource::UserInput,
            AuthorizationClass::Secret,
        )),
    )));
    assert!(poisoned.validate().is_err());
}

/// Session ≠ Context: the session a compilation happened in is a reference
/// on the snapshot, never the context's identity; snapshots with and
/// without a session coexist for the same task.
#[test]
fn session_is_a_reference_not_the_context() {
    let with_session = snapshot(SNAPSHOT, TASK, MODEL, Vec::new());
    let mut without_session = snapshot(
        "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPGB",
        TASK,
        MODEL,
        Vec::new(),
    );
    without_session.session_id = None;
    assert_eq!(with_session.task_id, without_session.task_id);
    assert!(with_session.session_id.is_some());
    assert!(without_session.session_id.is_none());

    let context = test_ok(Context::compile_from_snapshot(
        &with_session,
        &profile_for(MODEL),
        test_ok(Timestamp::parse("2026-09-21T13:46:00Z")),
    ));
    // The compiled view is keyed by the task and the model — the session is
    // not part of the compiled projection.
    let serialized = test_ok(serde_json::to_string(&context));
    assert!(
        !serialized.contains("sess_"),
        "the compiled context carries no session identity"
    );
    assert!(serialized.contains(TASK));
}

/// The whole context state survives a serialize → drop → reload round-trip
/// with equality, including reset records and profiles.
#[test]
fn context_state_survives_serialize_drop_reload() {
    let mut store = FakeContextStore::new();
    test_ok(store.create_profile(profile_for(MODEL)));
    test_ok(store.create_memory_item(memory(
        MEMORY,
        Some(TASK),
        MemoryTier::Warm,
        AuthorizationClass::Task,
        "decision: use the CRM export as source of truth",
    )));
    test_ok(store.put_snapshot(snapshot(SNAPSHOT, TASK, MODEL, Vec::new())));
    test_ok(store.reset_context(
        task_ref(),
        test_ok(ModelRef::parse(MODEL)),
        ResetReason::Manual,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
        test_ok(ContextSnapshotId::parse(SNAPSHOT)),
    ));

    let serialized = test_ok(serde_json::to_string(&store.state()));
    let reloaded: flauz_context::ContextStateSnapshot = test_ok(serde_json::from_str(&serialized));
    let restored = test_ok(FakeContextStore::restore(reloaded));
    assert_eq!(restored.state(), store.state());
    assert_eq!(restored.state().resets.len(), 1);
    assert_eq!(restored.state().snapshots.len(), 2);
}

fn stored(store: &FakeContextStore, id: &str) -> ContextSnapshot {
    let snapshot_id = test_ok(ContextSnapshotId::parse(id));
    let Some(snapshot) = test_ok(store.snapshot(&snapshot_id)) else {
        panic!("snapshot {id} missing");
    };
    snapshot
}
