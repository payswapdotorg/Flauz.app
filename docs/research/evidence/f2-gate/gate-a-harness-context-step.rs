//! The context-snapshot step of the kernel §10 round-trip — Gate A (real).
//!
//! Wires the merged `flauz-context` contracts into the F2 integration gate.
//! The canonical world/exec entity IDs cross the crate seam as frozen-format
//! strings (kernel §2 — the shared ID grammar), a durable
//! [`ContextSnapshot`](flauz_context::ContextSnapshot) is compiled as a
//! projection (never a transcript), provenance covers every included item,
//! authorization-aware filtering excludes secrets, compilation never mutates
//! the snapshot or its task references, the context state round-trips
//! through canonical JSON, and a model-switch RESET reconstructs a fresh
//! context from durable task state without forking the logical task.

use flauz_context::fakes::FakeContextStore;
use flauz_context::store::ContextStore;
use flauz_context::{
    ArtifactRef, AuthorizationClass, Context, ContextItem, ContextItemContent, ContextProvenance,
    ContextSnapshot, ContextSnapshotId, ContextSource, EnvironmentRef, EventRef, MemoryContent,
    MemoryItemId, MemoryItem, MemoryTier, ModelContextProfile, ModelRef, MultimodalBehavior,
    ObservationRef, ResetReason, SessionRef, SkillRef, TaskRef, Timestamp as ContextTimestamp,
    ToolSchemaHandling,
};
use flauz_context::{ActorRef as ContextActorRef, ContextStateSnapshot};

/// The step status (printed by the harness banner).
pub const STATUS: &str = "REAL (flauz-context merged — ORCH-001)";

/// The frozen-format canonical IDs this step consumes from the world and
/// exec stores, crossing the crate seam as strings (kernel §2).
pub struct ContextStepInput<'a> {
    /// Canonical task ID (`task_<ULID>`, owned by flauz-world).
    pub task_id: &'a str,
    /// Canonical session ID (`sess_<ULID>`, owned by flauz-world).
    pub session_id: &'a str,
    /// Canonical artifact ID (`art_<ULID>`, owned by flauz-world).
    pub artifact_id: &'a str,
    /// Canonical observation ID (`obs_<ULID>`, owned by flauz-world).
    pub observation_id: &'a str,
    /// Canonical event ID (`ev_<ULID>`, owned by flauz-world).
    pub event_id: &'a str,
    /// Canonical environment ID (`env_<ULID>`, owned by flauz-exec).
    pub environment_id: &'a str,
    /// Canonical model A ID (`model_<ULID>`, owned by flauz-exec).
    pub model_a: &'a str,
    /// Canonical model B ID (`model_<ULID>`, owned by flauz-exec).
    pub model_b: &'a str,
}

fn ok<T, E: std::fmt::Debug>(result: Result<T, E>, what: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("F2-GATE FAIL at context step {what}: {error:?}"),
    }
}

fn ctxt(value: &str) -> ContextTimestamp {
    ok(ContextTimestamp::parse(value), "context timestamp parse")
}

fn text_item(
    text: &str,
    tier: MemoryTier,
    source: ContextSource,
    authorization: AuthorizationClass,
) -> ContextItem {
    ok(
        ContextItem::new(
            ContextItemContent::Text {
                text: text.to_owned(),
            },
            tier,
            ok(
                ContextProvenance::new(source, authorization),
                "provenance ctor",
            ),
        ),
        "text item ctor",
    )
}

/// Runs the real context-snapshot step. Panics (→ non-zero exit) on any
/// contract violation; every assertion is a kernel §10 gate assertion.
pub fn compile_context_snapshot(input: ContextStepInput<'_>) {
    let t0 = ctxt("2026-09-21T12:16:00Z");
    let t1 = ctxt("2026-09-21T12:17:00Z");
    let t2 = ctxt("2026-09-21T12:18:00Z");
    let t3 = ctxt("2026-09-21T12:19:00Z");
    let lead = ok(ContextActorRef::user("user_f2gate_lead"), "context actor");
    let engine = ok(ContextActorRef::system("flauz-context-engine"), "engine actor");

    // ------------------------------------------------ 9a. the frozen-format
    // seam: world/exec canonical IDs parse as validated context refs.
    let task_ref = ok(TaskRef::parse(input.task_id), "task ref parse");
    let session_ref = ok(SessionRef::parse(input.session_id), "session ref parse");
    let artifact_ref = ok(ArtifactRef::parse(input.artifact_id), "artifact ref parse");
    let observation_ref =
        ok(ObservationRef::parse(input.observation_id), "observation ref parse");
    let event_ref = ok(EventRef::parse(input.event_id), "event ref parse");
    let environment_ref =
        ok(EnvironmentRef::parse(input.environment_id), "environment ref parse");
    let model_a = ok(ModelRef::parse(input.model_a), "model A ref parse");
    let model_b = ok(ModelRef::parse(input.model_b), "model B ref parse");
    let skill_ref = ok(SkillRef::parse("flauz.f2gate"), "skill ref parse");
    println!("[09a] frozen-format seam: world/exec canonical IDs parse as context refs (task/session/artifact/obs/event/env/model)");

    // ------------------------------------------------------- 9b. durable
    // memory: HOT current turn, WARM summary, COLD archive (JIT-only), and
    // a SECRET credential reference (storable, never compiled).
    let mut store = FakeContextStore::new();
    let hot_id = MemoryItemId::generate();
    ok(
        store.create_memory_item(ok(
            MemoryItem::new(
                hot_id.clone(),
                Some(task_ref.clone()),
                MemoryTier::Hot,
                AuthorizationClass::Task,
                MemoryContent::Text {
                    text: "current turn: run the F2 gate verification".to_owned(),
                },
                lead.clone(),
                t0,
            ),
            "hot memory ctor",
        )),
        "hot memory item",
    );
    let warm_id = MemoryItemId::generate();
    ok(
        store.create_memory_item(ok(
            MemoryItem::new(
                warm_id.clone(),
                Some(task_ref.clone()),
                MemoryTier::Warm,
                AuthorizationClass::Task,
                MemoryContent::Text {
                    text: "task summary: produce and verify the F2 gate artifact".to_owned(),
                },
                lead.clone(),
                t0,
            ),
            "warm memory ctor",
        )),
        "warm memory item",
    );
    let cold_id = MemoryItemId::generate();
    ok(
        store.create_memory_item(ok(
            MemoryItem::new(
                cold_id.clone(),
                Some(task_ref.clone()),
                MemoryTier::Cold,
                AuthorizationClass::Workspace,
                MemoryContent::Reference {
                    reference: "flauz-archive://runs/2026-09-20/full-transcript".to_owned(),
                },
                lead.clone(),
                t0,
            ),
            "cold memory ctor",
        )),
        "cold memory item",
    );
    let secret_id = MemoryItemId::generate();
    ok(
        store.create_memory_item(ok(
            MemoryItem::new(
                secret_id.clone(),
                Some(task_ref.clone()),
                MemoryTier::Hot,
                AuthorizationClass::Secret,
                MemoryContent::Reference {
                    // A reference to the secret store — never the credential
                    // material itself (kernel §7).
                    reference: "flauz-secrets://providers/example/api-key".to_owned(),
                },
                lead.clone(),
                t0,
            ),
            "secret memory ctor",
        )),
        "secret memory item",
    );
    println!("[09b] memory recorded: HOT + WARM + COLD (jit-only) + Secret (reference-only, storable)");

    // ------------------------------------------- 9c. model context profiles.
    let profile_a = ok(
        ModelContextProfile::new(
            model_a.clone(),
            128_000,
            MultimodalBehavior::TextOnly,
            ToolSchemaHandling::SummariesWithLazySchemas,
            lead.clone(),
            t0,
        ),
        "profile A ctor",
    );
    let profile_b = ok(
        ModelContextProfile::new(
            model_b.clone(),
            200_000,
            MultimodalBehavior::ImageInput,
            ToolSchemaHandling::LazyPerTool,
            lead.clone(),
            t0,
        ),
        "profile B ctor",
    );
    ok(store.create_profile(profile_a.clone()), "profile A");
    ok(store.create_profile(profile_b.clone()), "profile B");
    println!("[09c] model context profiles recorded (text-only 128k + vision 200k)");

    // --------------------------------- 9d. the durable context snapshot: a
    // projection of the durable task state for model A. Every item carries
    // provenance (structural); the snapshot references durable entities by
    // canonical ID and never redefines them.
    let snapshot_id = ContextSnapshotId::generate();
    let items = vec![
        text_item(
            "Run the F2 gate verification and report the round-trip result",
            MemoryTier::Hot,
            ContextSource::UserInput,
            AuthorizationClass::Workspace,
        ),
        ok(
            ContextItem::new(
                ContextItemContent::Text {
                    text: "current turn: run the F2 gate verification".to_owned(),
                },
                MemoryTier::Hot,
                ok(
                    ContextProvenance::new(
                        ContextSource::MemoryItem {
                            memory_item_id: hot_id.clone(),
                        },
                        AuthorizationClass::Task,
                    ),
                    "hot provenance",
                ),
            ),
            "hot item",
        ),
        ok(
            ContextItem::new(
                ContextItemContent::Reference {
                    reference: format!("artifact-summary:{}", input.artifact_id),
                },
                MemoryTier::Warm,
                ok(
                    ContextProvenance::new(
                        ContextSource::Artifact {
                            artifact_id: artifact_ref.clone(),
                        },
                        AuthorizationClass::Task,
                    ),
                    "artifact provenance",
                ),
            ),
            "artifact item",
        ),
        ok(
            ContextItem::new(
                ContextItemContent::Reference {
                    reference: format!("observation:{}", input.observation_id),
                },
                MemoryTier::Hot,
                ok(
                    ContextProvenance::new(
                        ContextSource::ResourceObservation {
                            observation_id: observation_ref.clone(),
                        },
                        AuthorizationClass::Task,
                    ),
                    "observation provenance",
                ),
            ),
            "observation item",
        ),
        ok(
            ContextItem::new(
                ContextItemContent::Reference {
                    reference: format!("event:{}", input.event_id),
                },
                MemoryTier::Warm,
                ok(
                    ContextProvenance::new(
                        ContextSource::SessionEvent {
                            event_id: event_ref.clone(),
                        },
                        AuthorizationClass::Task,
                    ),
                    "event provenance",
                ),
            ),
            "event item",
        ),
        ok(
            ContextItem::new(
                ContextItemContent::Reference {
                    reference: format!("environment-state:{}", input.environment_id),
                },
                MemoryTier::Hot,
                ok(
                    ContextProvenance::new(
                        ContextSource::EnvironmentState {
                            environment_id: environment_ref.clone(),
                        },
                        AuthorizationClass::Task,
                    ),
                    "environment provenance",
                ),
            ),
            "environment item",
        ),
        text_item(
            "skill package: the F2 gate verification procedure",
            MemoryTier::Warm,
            ContextSource::Skill {
                skill_id: skill_ref.clone(),
            },
            AuthorizationClass::Workspace,
        ),
        // The secret-authorized item: present in the durable snapshot (it
        // is storable state), excluded from every compiled view (9e).
        ok(
            ContextItem::new(
                ContextItemContent::Reference {
                    reference: "flauz-secrets://providers/example/api-key".to_owned(),
                },
                MemoryTier::Hot,
                ok(
                    ContextProvenance::new(
                        ContextSource::MemoryItem {
                            memory_item_id: secret_id.clone(),
                        },
                        AuthorizationClass::Secret,
                    ),
                    "secret provenance",
                ),
            ),
            "secret item",
        ),
    ];
    let snapshot = ok(
        ContextSnapshot::new(
            snapshot_id.clone(),
            task_ref.clone(),
            Some(session_ref.clone()),
            model_a.clone(),
            engine.clone(),
            t1,
            items,
        ),
        "snapshot ctor",
    );
    assert_eq!(snapshot.items.len(), 8, "the snapshot carries 8 items");
    for item in &snapshot.items {
        assert!(
            item.provenance.validate().is_ok(),
            "provenance covers every item (structural)"
        );
    }
    ok(store.put_snapshot(snapshot.clone()), "put snapshot");
    println!("[09d] durable snapshot compiled: 8 items, provenance on every item (projection, not transcript)");

    // --------------------------------------- 9e. compile for model A: the
    // authorization-aware view. Secrets are excluded; the snapshot is not
    // mutated; task references are preserved untouched.
    let before = ok(store.snapshot(&snapshot_id), "snapshot before compile")
        .expect("snapshot stored");
    let context = ok(
        Context::compile_from_snapshot(&snapshot, &profile_a, t2),
        "compile for model A",
    );
    let after = ok(store.snapshot(&snapshot_id), "snapshot after compile")
        .expect("snapshot still stored");
    assert_eq!(before, after, "compilation never mutates the snapshot");
    assert_eq!(
        context.items.len(),
        7,
        "the secret-authorized item is never compiled"
    );
    assert!(
        context.items.iter().all(|item| item
            .provenance
            .eligible_for_model_context()),
        "every compiled item is authorization-eligible"
    );
    assert_eq!(
        context.task_id, task_ref,
        "compilation preserves the task reference untouched"
    );
    assert_eq!(context.model_id, model_a, "the view targets model A");
    ok(context.validate(), "compiled view validates");
    println!("[09e] compiled for model A: 7/8 items (Secret excluded), snapshot unmutated, task ref preserved");

    // ------------------------- 9f. reconstructibility from references alone.
    let references = snapshot.durable_references();
    for expected in [
        task_ref.as_str(),
        session_ref.as_str(),
        model_a.as_str(),
        artifact_ref.as_str(),
        observation_ref.as_str(),
        event_ref.as_str(),
        environment_ref.as_str(),
        hot_id.as_str(),
        secret_id.as_str(),
    ] {
        assert!(
            references.contains(&expected.to_owned()),
            "durable_references() must cover {expected}"
        );
    }
    assert!(
        !references.contains(&format!("artifact-summary:{}", input.artifact_id)),
        "durable references are canonical IDs, never content strings"
    );
    println!("[09f] reconstructible from references: task/session/model/artifact/obs/event/env/memory all named");

    // ----------------- 9g. canonical JSON: serialize → drop → reload →
    // equality; no credential material anywhere in the serialized state.
    let state = store.state();
    let serialized = ok(serde_json::to_string(&state), "context state JSON");
    assert!(
        serialized.contains("\"v\":1"),
        "canonical JSON carries the v:1 schema marker"
    );
    assert!(
        !serialized.contains("ghp_") && !serialized.contains("sk-") && !serialized.contains("api-key-value"),
        "no credential material in serialized context state"
    );
    assert!(
        serialized.contains("flauz-secrets://providers/example/api-key"),
        "the secret reference (not material) is durable state"
    );
    drop(store);
    let parsed: ContextStateSnapshot = ok(
        serde_json::from_str(&serialized),
        "context state parse",
    );
    let mut reloaded = ok(FakeContextStore::restore(parsed), "context restore");
    assert_eq!(
        reloaded.state(),
        state,
        "context state equality after serialize → drop → reload"
    );
    println!("[09g] canonical JSON round-trip: v:1 marker, no credential material, state equality after reload");

    // ------------------------- 9h. the model-switch RESET: a fresh context
    // reconstructed from durable task state for model B. The superseded
    // snapshot is unchanged; COLD is JIT-only (dropped); the Secret item is
    // never compiled; the logical task is not forked.
    let reconstruction = ok(
        reloaded.reset_context(
            task_ref.clone(),
            model_b.clone(),
            ResetReason::ModelChanged,
            engine.clone(),
            t3,
            snapshot_id.clone(),
        ),
        "model-switch reset",
    );
    assert_ne!(
        reconstruction.snapshot.id, snapshot_id,
        "the fresh snapshot is a new ctxsnap entity"
    );
    assert_eq!(
        reconstruction.reset.superseded_snapshot_id, snapshot_id,
        "the reset record names the superseded snapshot"
    );
    assert_eq!(
        reconstruction.snapshot.task_id, task_ref,
        "RESET keeps the same logical task (no fork)"
    );
    assert_eq!(
        reconstruction.context.model_id, model_b,
        "the fresh view targets model B"
    );
    assert!(
        reconstruction.snapshot.items.iter().all(|item| item.tier != MemoryTier::Cold),
        "COLD memory is JIT-only: never auto-included by a reset"
    );
    assert!(
        reconstruction
            .context
            .items
            .iter()
            .all(|item| item.provenance.eligible_for_model_context()),
        "secrets never compile, even after reset"
    );
    assert!(
        !reconstruction.context.items.iter().any(|item| match &item.provenance.source {
            ContextSource::MemoryItem { memory_item_id } => *memory_item_id == secret_id,
            _ => false,
        }),
        "the secret memory item is not in the fresh compiled view"
    );
    let superseded_still = ok(
        reloaded.snapshot(&snapshot_id),
        "superseded snapshot lookup",
    )
    .expect("the superseded snapshot is retained");
    assert_eq!(
        superseded_still, snapshot,
        "the superseded snapshot is never mutated by the reset"
    );
    println!("[09h] model-switch RESET: fresh snapshot from durable task state (HOT/WARM retained, COLD jit-only, Secret excluded), same logical task");

    println!("[09] context snapshot step: ALL CONTEXT-CONTRACT ASSERTIONS PASS");
}
