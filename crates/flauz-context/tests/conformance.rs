//! Kernel conformance tests (F2-CONTRACT-KERNEL §9, context side): the
//! frozen ID vectors, the context-specific canonical laws, canonical
//! fixture round-trips, and the no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_context::fakes::FakeContextStore;
use flauz_context::ids::{self, EntityKind, IdError};
use flauz_context::profile::{ModelContextProfile, MultimodalBehavior, ToolSchemaHandling};
use flauz_context::provenance::{AuthorizationClass, ContextProvenance, ContextSource};
use flauz_context::{
    ActorRef, ArtifactRef, Context, ContextItem, ContextItemContent, ContextReset,
    ContextSnapshot, ContextSnapshotId, ContextStore, ContextStoreError, EnvironmentRef, EventRef,
    MemoryContent, MemoryItemId, MemoryItem, MemoryTier, ModelRef, ObservationRef, ResetReason,
    SessionRef, SkillRef, TaskRef, Timestamp,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/f2")
        .join(relative)
}

fn read_fixture(relative: &str) -> String {
    let content = fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
    // Canonical fixtures are committed with LF endings; a Windows checkout
    // with autocrlf translates them to CRLF. Normalize before comparison so
    // the byte-for-byte round-trip law is tested against the CANONICAL form,
    // not the platform's line-ending translation.
    content.replace("\r\n", "\n").trim_end().to_owned()
}

fn stored_snapshot(store: &FakeContextStore, id: &str) -> ContextSnapshot {
    let snapshot_id = test_ok(ContextSnapshotId::parse(id));
    let Some(snapshot) = test_ok(store.snapshot(&snapshot_id)) else {
        panic!("snapshot {id} missing");
    };
    snapshot
}

/// The frozen valid ID vectors (kernel §2), asserted through the generic
/// canonical-ID validator and, for context-owned kinds, the typed parsers.
#[test]
fn kernel_id_valid_vectors() {
    let content = read_fixture("kernel/ids.valid.json");
    let vectors: Vec<String> = test_ok(serde_json::from_str(&content));

    let expected = [
        ("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0", EntityKind::Workspace),
        ("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1", EntityKind::Task),
        ("res_01J8ZQ5V8K3T2B7N6X4R9DQPC2", EntityKind::Resource),
        ("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3", EntityKind::Event),
        ("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4", EntityKind::Environment),
        (
            "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5",
            EntityKind::ContextSnapshot,
        ),
    ];
    assert_eq!(vectors.len(), expected.len(), "valid vector count changed");

    for ((vector, kind), fixture) in expected.iter().zip(&vectors) {
        assert_eq!(vector, fixture, "fixture must carry the frozen vector");
        let parsed = test_ok(ids::validate(vector));
        assert_eq!(parsed, *kind);
        assert_eq!(parsed.prefix(), kind.prefix());
    }

    // Context-owned kinds also parse through their typed newtypes, and
    // serialization is the plain canonical string. Foreign kinds parse
    // through their local validating reference newtypes.
    let snapshot_id = test_ok(ContextSnapshotId::parse("ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5"));
    assert_eq!(
        test_ok(serde_json::to_string(&snapshot_id)),
        "\"ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5\""
    );
    let reloaded: ContextSnapshotId =
        test_ok(serde_json::from_str("\"ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5\""));
    assert_eq!(reloaded, snapshot_id);
    let task_ref = test_ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"));
    assert_eq!(task_ref.as_str(), "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1");
}

/// The frozen invalid ID vectors (kernel §2), each with its reason, asserted
/// against the typed failure reasons of the validator.
#[test]
fn kernel_id_invalid_vectors() {
    let content = read_fixture("kernel/ids.invalid.json");
    let vectors: Vec<InvalidVector> = test_ok(serde_json::from_str(&content));
    assert_eq!(vectors.len(), 12, "invalid vector count changed");

    for vector in &vectors {
        let error = ids::validate(&vector.id)
            .err()
            .unwrap_or_else(|| panic!("{} must be invalid", vector.id));
        let matches = match (vector.reason.as_str(), &error) {
            ("bad separator", IdError::BadSeparator) => true,
            ("uppercase kind", IdError::WrongKindCase) => true,
            ("lowercase ULID", IdError::InvalidCharacter { character, .. }) => {
                character.is_lowercase()
            }
            ("25-char ULID", IdError::WrongLength { length: 25 }) => true,
            ("empty ULID", IdError::WrongLength { length: 0 }) => true,
            ("contains I", IdError::InvalidCharacter { character: 'I', .. }) => true,
            ("contains U", IdError::InvalidCharacter { character: 'U', .. }) => true,
            ("contains L", IdError::InvalidCharacter { character: 'L', .. }) => true,
            ("contains O", IdError::InvalidCharacter { character: 'O', .. }) => true,
            ("no separator", IdError::NoSeparator) => true,
            ("empty string", IdError::Empty) => true,
            ("missing kind", IdError::MissingKind) => true,
            _ => false,
        };
        assert!(
            matches,
            "vector {:?} (reason {:?}) produced unexpected error {error:?}",
            vector.id, vector.reason
        );
        // The typed parsers reject every invalid vector too.
        assert!(ContextSnapshotId::parse(&vector.id).is_err());
        assert!(MemoryItemId::parse(&vector.id).is_err());
        assert!(TaskRef::parse(&vector.id).is_err());
    }
}

#[derive(serde::Deserialize)]
struct InvalidVector {
    id: String,
    reason: String,
}

/// The context is a projection, not a transcript: a snapshot references
/// durable entities by canonical ID, serializes canonically, survives a
/// serialize → drop → reload round-trip, and is reconstructible from its
/// references (kernel §9 `context_snapshot_reconstructible_from_references`).
#[test]
fn context_snapshot_reconstructible_from_references() {
    let snapshot = test_snapshot_with_every_source_kind();
    let references = snapshot.durable_references();

    // Every reference is a valid canonical ID.
    for reference in &references {
        assert!(
            ids::validate(reference).is_ok(),
            "{reference} must be a canonical entity ID"
        );
    }

    // Canonical serialization round-trips with equality, and the reference
    // list survives the round-trip unchanged.
    let serialized = test_ok(serde_json::to_string(&snapshot));
    let reloaded: ContextSnapshot = test_ok(serde_json::from_str(&serialized));
    assert_eq!(reloaded, snapshot);
    assert_eq!(reloaded.durable_references(), references);
    let reserialized = test_ok(serde_json::to_string(&reloaded));
    assert_eq!(reserialized, serialized);

    // The serialized projection contains only references to durable
    // entities — no transcript of foreign entities, no bulk payloads.
    let document: serde_json::Value = test_ok(serde_json::from_str(&serialized));
    let object = document
        .as_object()
        .unwrap_or_else(|| panic!("snapshot must serialize to an object"));
    for field in [
        "v",
        "id",
        "version",
        "task_id",
        "session_id",
        "model_id",
        "compiled_by",
        "compiled_at",
        "items",
    ] {
        assert!(object.contains_key(field), "snapshot is missing {field}");
    }
    assert_eq!(
        object["task_id"],
        serde_json::json!("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")
    );
    assert_eq!(
        object["model_id"],
        serde_json::json!("model_01J8ZQ5V8K3T2B7N6X4R9DQRE")
    );
}

/// Every included context item is attributable to a source and carries an
/// authorization class (kernel §9 `context_provenance_covers_every_item`).
#[test]
fn context_provenance_covers_every_item() {
    let snapshot = test_snapshot_with_every_source_kind();
    assert!(
        !snapshot.items.is_empty(),
        "the test snapshot must carry items"
    );
    for item in &snapshot.items {
        let provenance = &item.provenance;
        test_ok(provenance.validate());
        // Structurally: provenance exists and names one of the ten frozen
        // source kinds, each attributable to a source.
        let source_kind = match &provenance.source {
            ContextSource::UserInput => "user_input",
            ContextSource::SessionEvent { .. } => "session_event",
            ContextSource::MemoryItem { .. } => "memory_item",
            ContextSource::Artifact { .. } => "artifact",
            ContextSource::ResourceObservation { .. } => "resource_observation",
            ContextSource::ToolResult { .. } => "tool_result",
            ContextSource::RetrievedDocument { .. } => "retrieved_document",
            ContextSource::Skill { .. } => "skill",
            ContextSource::EnvironmentState { .. } => "environment_state",
            ContextSource::CollaborationEvent { .. } => "collaboration_event",
        };
        assert!(!source_kind.is_empty());
        // Every item's authorization class is one of the frozen classes,
        // and eligibility follows it exactly.
        let eligible = provenance.eligible_for_model_context();
        assert_eq!(
            eligible,
            provenance.authorization != AuthorizationClass::Secret
        );
    }

    // A compiled view covers its items with the same provenance; an item
    // without provenance cannot be constructed (the type requires it).
    let profile = test_profile();
    let context = test_ok(Context::compile_from_snapshot(
        &snapshot,
        &profile,
        test_ok(Timestamp::parse("2026-09-21T13:46:00Z")),
    ));
    assert_eq!(context.provenance().len(), context.items.len());
}

/// Compiling (or switching) a context never mutates task references: the
/// snapshot is taken by reference, the compiled view carries the same task
/// reference, and a RESET reconstructs from durable task state without
/// mutating the superseded snapshot (kernel §9
/// `context_compilation_does_not_mutate_task_refs`).
#[test]
fn context_compilation_does_not_mutate_task_refs() {
    let mut store = test_store_with_everything();
    let snapshot = stored_snapshot(&store, "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5");

    let task_before = snapshot.task_id.clone();
    let items_before = snapshot.items.clone();
    let profile = test_profile();
    let compiled_at = test_ok(Timestamp::parse("2026-09-21T13:46:00Z"));

    // Compile twice; the snapshot is untouched and the views agree.
    let first = test_ok(Context::compile_from_snapshot(&snapshot, &profile, compiled_at));
    let second = test_ok(Context::compile_from_snapshot(&snapshot, &profile, compiled_at));
    assert_eq!(first, second);
    assert_eq!(first.task_id, task_before);
    assert_eq!(snapshot.task_id, task_before);
    assert_eq!(snapshot.items, items_before);
    test_ok(snapshot.validate());

    // And a RESET (a fresh context reconstructed from durable task state)
    // preserves the task reference too, superseding — never mutating — the
    // previous snapshot.
    let reconstruction = test_ok(store.reset_context(
        task_before.clone(),
        test_ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQRE")),
        ResetReason::ModelChanged,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
        snapshot.id.clone(),
    ));
    assert_eq!(reconstruction.context.task_id, task_before);
    assert_eq!(reconstruction.snapshot.task_id, task_before);
    assert_eq!(reconstruction.reset.task_id, task_before);
    assert_ne!(reconstruction.snapshot.id, snapshot.id);
    let superseded = stored_snapshot(&store, "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5");
    assert_eq!(
        superseded.items, items_before,
        "the superseded snapshot is never mutated"
    );
}

/// Kernel §3 version rules: memory items and model profiles start at
/// version 1, increment by exactly 1 per durable mutation, and reject
/// expected-version mismatches with `VersionConflict` — silent overwrite is
/// forbidden (kernel §9 `version_monotonic_and_version_conflict_rejected`).
#[test]
fn version_monotonic_and_version_conflict_rejected() {
    let mut store = FakeContextStore::new();

    // Memory items.
    let item = test_ok(MemoryItem::new(
        test_ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5")),
        Some(test_ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"))),
        MemoryTier::Warm,
        AuthorizationClass::Task,
        MemoryContent::Text {
            text: "decision: use the CRM export as source of truth".to_owned(),
        },
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ));
    let created = test_ok(store.create_memory_item(item));
    assert_eq!(created.version, 1);

    let mut updated = created.clone();
    updated.content = MemoryContent::Text {
        text: "decision: confirmed against the CRM export".to_owned(),
    };
    let next = test_ok(store.update_memory_item(updated.clone()));
    assert_eq!(next.version, 2);

    // A stale expected version is rejected, never silently overwritten.
    let stale = store.update_memory_item(updated);
    assert_eq!(
        stale.err(),
        Some(ContextStoreError::VersionConflict {
            id: "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5".to_owned(),
            expected_version: 1,
            actual_version: 2,
        })
    );
    let Some(stored) = test_ok(store.memory_item(&next.id)) else {
        panic!("memory item missing");
    };
    assert_eq!(stored.version, 2);
    assert_eq!(
        stored.content,
        MemoryContent::Text {
            text: "decision: confirmed against the CRM export".to_owned(),
        }
    );

    // Model profiles follow the same rules, keyed by their model.
    let profile = test_profile();
    let created_profile = test_ok(store.create_profile(profile));
    assert_eq!(created_profile.version, 1);
    let mut updated_profile = created_profile.clone();
    updated_profile.context_capacity_tokens = 400_000;
    let next_profile = test_ok(store.update_profile(updated_profile.clone()));
    assert_eq!(next_profile.version, 2);
    assert_eq!(
        store.update_profile(updated_profile).err(),
        Some(ContextStoreError::VersionConflict {
            id: "model_01J8ZQ5V8K3T2B7N6X4R9DQRE".to_owned(),
            expected_version: 1,
            actual_version: 2,
        })
    );

    // Snapshots are immutable: a version other than 1 fails validation.
    let mut snapshot = test_snapshot_with_every_source_kind();
    assert_eq!(snapshot.version, 1);
    snapshot.version = 2;
    assert!(snapshot.validate().is_err());
}

/// Every conformance fixture is canonical JSON: it parses strictly and
/// re-serializes to exactly the committed bytes.
#[test]
fn fixtures_roundtrip_canonical() {
    let checks: &[(&str, FixtureKind)] = &[
        ("memory-item/minimal.json", FixtureKind::MemoryItem),
        ("memory-item/typical.json", FixtureKind::MemoryItem),
        ("context-item/typical.json", FixtureKind::ContextItem),
        ("context-snapshot/typical.json", FixtureKind::Snapshot),
        ("context/minimal.json", FixtureKind::Context),
        ("context-reset/typical.json", FixtureKind::Reset),
        ("model-context-profile/typical.json", FixtureKind::Profile),
        ("model-context-profile/minimal.json", FixtureKind::Profile),
    ];

    for (relative, kind) in checks {
        let content = read_fixture(relative);
        let serialized = match kind {
            FixtureKind::MemoryItem => round_trip::<MemoryItem>(&content),
            FixtureKind::ContextItem => round_trip::<ContextItem>(&content),
            FixtureKind::Snapshot => round_trip::<ContextSnapshot>(&content),
            FixtureKind::Context => round_trip::<Context>(&content),
            FixtureKind::Reset => round_trip::<ContextReset>(&content),
            FixtureKind::Profile => round_trip::<ModelContextProfile>(&content),
        };
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }

    // Invalid fixtures must fail strict parses.
    let unknown_field = read_fixture("context-snapshot/invalid-unknown-field.json");
    assert!(
        serde_json::from_str::<ContextSnapshot>(&unknown_field).is_err(),
        "unknown fields must be rejected, not ignored"
    );
}

enum FixtureKind {
    MemoryItem,
    ContextItem,
    Snapshot,
    Context,
    Reset,
    Profile,
}

fn round_trip<T>(content: &str) -> String
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = test_ok(serde_json::from_str(content));
    test_ok(serde_json::to_string_pretty(&parsed))
}

/// No contract type, fixture, or serialized state contains credential
/// material: only references are ever stored, and secret-authorized items
/// are never compiled into model context (kernel §7).
#[test]
fn no_credential_material_in_serialized_state() {
    let mut store = test_store_with_everything();
    let snapshot = stored_snapshot(&store, "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5");
    let reconstruction = test_ok(store.reset_context(
        snapshot.task_id.clone(),
        test_ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQRE")),
        ResetReason::Pressure,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
        snapshot.id.clone(),
    ));
    let serialized = test_ok(serde_json::to_string(&store.state()));
    assert_credential_free(&serialized);
    let context_serialized = test_ok(serde_json::to_string(&reconstruction.context));
    assert_credential_free(&context_serialized);

    // The committed fixtures are data too: they must stay credential-free.
    let fixtures_root = fixture_path("");
    let mut stack = vec![fixtures_root];
    while let Some(directory) = stack.pop() {
        for entry in test_ok(fs::read_dir(&directory)) {
            let entry = test_ok(entry);
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let content = test_ok(fs::read_to_string(&path));
                assert_credential_free(&content);
            }
        }
    }
}

fn assert_credential_free(serialized: &str) {
    for marker in [
        "sk-",
        "Bearer ",
        "api_key",
        "apikey",
        "password",
        "passwd",
        "client_secret",
        "access_token",
        "refresh_token",
        "PRIVATE KEY",
        "BEGIN RSA",
        "xoxb-",
        "ghp_",
    ] {
        assert!(
            !serialized.contains(marker),
            "serialized state must never contain credential material ({marker:?})"
        );
    }
}

fn test_profile() -> ModelContextProfile {
    test_ok(ModelContextProfile::new(
        test_ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQRE")),
        200_000,
        MultimodalBehavior::ImageInput,
        ToolSchemaHandling::SummariesWithLazySchemas,
        test_ok(ActorRef::system("flauz-fake")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ))
}

fn test_item(source: ContextSource, authorization: AuthorizationClass) -> ContextItem {
    test_ok(ContextItem::new(
        ContextItemContent::Text {
            text: "the dashboard shows 3 open tickets".to_owned(),
        },
        MemoryTier::Hot,
        test_ok(ContextProvenance::new(source, authorization)),
    ))
}

/// A snapshot exercising every provenance source kind that carries a
/// canonical entity reference.
fn test_snapshot_with_every_source_kind() -> ContextSnapshot {
    test_ok(ContextSnapshot::new(
        test_ok(ContextSnapshotId::parse("ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5")),
        test_ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
        Some(test_ok(SessionRef::parse("sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6"))),
        test_ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQRE")),
        test_ok(ActorRef::system("flauz-context-engine")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        vec![
            test_item(ContextSource::UserInput, AuthorizationClass::Task),
            test_item(
                ContextSource::SessionEvent {
                    event_id: test_ok(EventRef::parse("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
                },
                AuthorizationClass::Workspace,
            ),
            test_item(
                ContextSource::MemoryItem {
                    memory_item_id: test_ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5")),
                },
                AuthorizationClass::Task,
            ),
            test_item(
                ContextSource::Artifact {
                    artifact_id: test_ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPH7")),
                },
                AuthorizationClass::Task,
            ),
            test_item(
                ContextSource::ResourceObservation {
                    observation_id: test_ok(ObservationRef::parse(
                        "obs_01J8ZQ5V8K3T2B7N6X4R9DQPJ8",
                    )),
                },
                AuthorizationClass::Public,
            ),
            test_item(
                ContextSource::ToolResult {
                    reference: "art_01J8ZQ5V8K3T2B7N6X4R9DQPH7/tool-output.json".to_owned(),
                },
                AuthorizationClass::Participant,
            ),
            test_item(
                ContextSource::RetrievedDocument {
                    reference: "doc://handbook/onboarding".to_owned(),
                },
                AuthorizationClass::Workspace,
            ),
            test_item(
                ContextSource::Skill {
                    skill_id: test_ok(SkillRef::parse("flauz.research.collect")),
                },
                AuthorizationClass::Public,
            ),
            test_item(
                ContextSource::EnvironmentState {
                    environment_id: test_ok(EnvironmentRef::parse(
                        "env_01J8ZQ5V8K3T2B7N6X4R9DQPE4",
                    )),
                },
                AuthorizationClass::Task,
            ),
            test_item(
                ContextSource::CollaborationEvent {
                    event_id: test_ok(EventRef::parse("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
                },
                AuthorizationClass::Workspace,
            ),
        ],
    ))
}

/// A store with one memory item (workspace-scoped, WARM), the model profile
/// and the every-source-kind snapshot, mirroring the F2 gate flow.
fn test_store_with_everything() -> FakeContextStore {
    let mut store = FakeContextStore::new();
    test_ok(store.create_profile(test_profile()));
    test_ok(store.create_memory_item(test_ok(MemoryItem::new(
        test_ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5")),
        None,
        MemoryTier::Warm,
        AuthorizationClass::Workspace,
        MemoryContent::Text {
            text: "decision: use the CRM export as source of truth".to_owned(),
        },
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ))));
    test_ok(store.put_snapshot(test_snapshot_with_every_source_kind()));
    store
}
