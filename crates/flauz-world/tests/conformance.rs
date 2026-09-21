//! Kernel conformance tests (F2-CONTRACT-KERNEL §9): the frozen ID vectors,
//! the v1 event envelope, canonical fixture round-trips, and the
//! no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_world::event::event_types;
use flauz_world::fakes::FakeWorldStore;
use flauz_world::ids::{
    self, ClaimId, EntityKind, EventId, IdError, ResourceId, TaskId, WorkspaceId,
};
use flauz_world::procedure::ProcedureInput;
use flauz_world::refs::{ActorRef, EntityRef, StreamRef};
use flauz_world::resource::{AccessSurface, ResourceState, SurfaceKind};
use flauz_world::time::Timestamp;
use flauz_world::value::{CanonicalValue, Payload};
use flauz_world::{AccessMode, ConflictPolicy, LeaseId, Procedure, ProcedureVersion, Resource};
use flauz_world::{
    Artifact, ArtifactContent, Claim, Event, EventEnvelope, Evidence, Observation, ResourceLease,
    Session, Task, VerificationStatus, Workspace, WorldStore,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn test_payload(entries: &[(&str, CanonicalValue)]) -> Payload {
    let mut payload = Payload::empty();
    for (key, value) in entries {
        payload = test_ok(payload.with(key, value.clone()));
    }
    payload
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

/// The frozen valid ID vectors (kernel §2), asserted through the generic
/// canonical-ID validator and, for world-owned kinds, the typed parsers.
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

    // World-owned kinds also parse through their typed newtypes, and
    // serialization is the plain canonical string.
    let task_id = test_ok(TaskId::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"));
    assert_eq!(
        test_ok(serde_json::to_string(&task_id)),
        "\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\""
    );
    let reloaded: TaskId = test_ok(serde_json::from_str("\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\""));
    assert_eq!(reloaded, task_id);
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
        assert!(TaskId::parse(&vector.id).is_err());
    }
}

#[derive(serde::Deserialize)]
struct InvalidVector {
    id: String,
    reason: String,
}

/// The v1 event envelope round-trips with every field populated, in the
/// frozen kernel §5 shape.
#[test]
fn envelope_roundtrip_all_fields() {
    let event = test_ok(Event::new(
        event_types::TASK_MODEL_CHANGED,
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        test_ok(EntityRef::new(
            EntityKind::Task,
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1",
        )),
        test_payload(&[
            (
                "from_model",
                CanonicalValue::from("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE"),
            ),
            (
                "to_model",
                CanonicalValue::from("model_01J8ZQ5V8K3T2B7N6X4R9DQPSF"),
            ),
        ]),
    ))
    .with_causation(test_ok(EventId::parse("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")))
    .with_correlation("reconciliation-42");

    let envelope = EventEnvelope::new(
        test_ok(EventId::parse("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
        42,
        StreamRef::workspace(&test_ok(WorkspaceId::parse(
            "ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
        ))),
        event,
    );

    let serialized = test_ok(serde_json::to_string(&envelope));
    let value: serde_json::Value = test_ok(serde_json::from_str(&serialized));
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("envelope must serialize to an object"));
    let expected_fields = [
        "v",
        "kind",
        "event_id",
        "seq",
        "stream",
        "event_type",
        "ts",
        "actor",
        "causation_id",
        "correlation_id",
        "subject",
        "payload",
    ];
    assert_eq!(object.len(), expected_fields.len());
    for field in expected_fields {
        assert!(object.contains_key(field), "envelope is missing {field}");
    }
    assert_eq!(object["v"], serde_json::json!(1));
    assert_eq!(object["kind"], serde_json::json!("flauz.event"));
    assert_eq!(object["seq"], serde_json::json!(42));
    assert_eq!(
        object["event_type"],
        serde_json::json!("task.model_changed")
    );
    assert_eq!(object["ts"], serde_json::json!("2026-09-21T13:45:00Z"));
    assert_eq!(
        object["causation_id"],
        serde_json::json!("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")
    );
    assert_eq!(
        object["correlation_id"],
        serde_json::json!("reconciliation-42")
    );
    assert_eq!(
        object["stream"],
        serde_json::json!({"kind": "workspace", "id": "ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0"})
    );
    assert_eq!(
        object["actor"],
        serde_json::json!({"kind": "agent", "id": "agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD"})
    );
    assert_eq!(
        object["subject"],
        serde_json::json!({"entity_kind": "task", "id": "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"})
    );

    let reloaded: EventEnvelope = test_ok(serde_json::from_str(&serialized));
    assert_eq!(reloaded, envelope);
    let round_trip = test_ok(serde_json::to_string(&reloaded));
    assert_eq!(round_trip, serialized);
}

/// An envelope carrying an unknown field is rejected by canonical reads.
#[test]
fn envelope_unknown_field_rejected() {
    let content = read_fixture("event-envelope/typical.json");
    let mut value: serde_json::Value = test_ok(serde_json::from_str(&content));
    let object = value
        .as_object_mut()
        .unwrap_or_else(|| panic!("fixture must be an object"));
    object.insert("totally_unknown".to_owned(), serde_json::Value::Bool(true));
    let poisoned = test_ok(serde_json::to_string(&object));
    let error = serde_json::from_str::<EventEnvelope>(&poisoned);
    assert!(
        error.is_err(),
        "unknown fields must be rejected, not ignored"
    );
    // The committed invalid fixture exercises the same rule.
    let invalid = read_fixture("event-envelope/invalid-unknown-field.json");
    assert!(serde_json::from_str::<EventEnvelope>(&invalid).is_err());
}

/// Every conformance fixture is canonical JSON: it parses strictly and
/// re-serializes to exactly the committed bytes.
#[test]
fn fixtures_roundtrip_canonical() {
    let checks: &[(&str, FixtureKind)] = &[
        ("workspace/minimal.json", FixtureKind::Workspace),
        ("workspace/typical.json", FixtureKind::Workspace),
        ("session/typical.json", FixtureKind::Session),
        ("task/minimal.json", FixtureKind::Task),
        ("task/typical.json", FixtureKind::Task),
        ("artifact/typical.json", FixtureKind::Artifact),
        ("resource/typical.json", FixtureKind::Resource),
        ("resource/boundary.json", FixtureKind::Resource),
        ("resource-state/typical.json", FixtureKind::ResourceState),
        ("event-envelope/typical.json", FixtureKind::Envelope),
        ("observation/typical.json", FixtureKind::Observation),
        ("claim/typical.json", FixtureKind::Claim),
        ("evidence/typical.json", FixtureKind::Evidence),
        ("lease/typical.json", FixtureKind::Lease),
        ("lease/boundary.json", FixtureKind::Lease),
        ("procedure/minimal.json", FixtureKind::Procedure),
        ("procedure/typical.json", FixtureKind::Procedure),
    ];

    for (relative, kind) in checks {
        let content = read_fixture(relative);
        let serialized = match kind {
            FixtureKind::Workspace => round_trip::<Workspace>(&content),
            FixtureKind::Session => round_trip::<Session>(&content),
            FixtureKind::Task => round_trip::<Task>(&content),
            FixtureKind::Artifact => round_trip::<Artifact>(&content),
            FixtureKind::Resource => round_trip::<Resource>(&content),
            FixtureKind::ResourceState => round_trip::<ResourceState>(&content),
            FixtureKind::Envelope => round_trip::<EventEnvelope>(&content),
            FixtureKind::Observation => round_trip::<Observation>(&content),
            FixtureKind::Claim => round_trip::<Claim>(&content),
            FixtureKind::Evidence => round_trip::<Evidence>(&content),
            FixtureKind::Lease => round_trip::<ResourceLease>(&content),
            FixtureKind::Procedure => round_trip::<Procedure>(&content),
        };
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }

    // Invalid fixtures must fail strict parses.
    let malformed_task = read_fixture("task/invalid-malformed-id.json");
    assert!(
        serde_json::from_str::<Task>(&malformed_task).is_err(),
        "a malformed canonical ID must fail the strict parse"
    );
}

enum FixtureKind {
    Workspace,
    Session,
    Task,
    Artifact,
    Resource,
    ResourceState,
    Envelope,
    Observation,
    Claim,
    Evidence,
    Lease,
    Procedure,
}

fn round_trip<T>(content: &str) -> String
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = test_ok(serde_json::from_str(content));
    test_ok(serde_json::to_string_pretty(&parsed))
}

/// No contract type, fixture, or serialized state contains credential
/// material: only references are ever stored (kernel §7).
#[test]
fn no_credential_material_in_serialized_state() {
    let store = test_world_with_everything();
    let snapshot = store.snapshot();
    let serialized = test_ok(serde_json::to_string(&snapshot));
    assert_credential_free(&serialized);

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

/// Builds a world exercising every entity kind, mirroring the F2 gate flow.
fn test_world_with_everything() -> FakeWorldStore {
    let created = test_ok(Timestamp::parse("2026-09-21T13:45:00Z"));
    let ws_id = test_ok(WorkspaceId::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0"));
    let task_id = test_ok(TaskId::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"));
    let resource_id = test_ok(ResourceId::parse("res_01J8ZQ5V8K3T2B7N6X4R9DQPC2"));
    let claim_id = test_ok(ClaimId::parse("claim_01J8ZQ5V8K3T2B7N6X4R9DQPK9"));
    let user = test_ok(ActorRef::user("user-alice"));
    let agent = test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD"));

    let mut store = FakeWorldStore::new();
    test_ok(store.create_workspace(test_ok(Workspace::new(
        ws_id.clone(),
        "Research workspace",
        user.clone(),
        created,
    ))));
    test_ok(store.create_session(test_ok(Session::new(
        test_ok(flauz_world::SessionId::parse(
            "sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6",
        )),
        ws_id.clone(),
        Some(task_id.clone()),
        user.clone(),
        created,
    ))));
    test_ok(store.create_task(test_ok(Task::new(
        task_id.clone(),
        ws_id,
        "Reconcile the ticket counts",
        agent.clone(),
        created,
    ))));
    test_ok(store.create_artifact(test_ok(Artifact::new(
        test_ok(flauz_world::ArtifactId::parse(
            "art_01J8ZQ5V8K3T2B7N6X4R9DQPH7",
        )),
        task_id.clone(),
        "Report",
        ArtifactContent::Reference {
            reference: "art_01J8ZQ5V8K3T2B7N6X4R9DQPH7/content.txt".to_owned(),
        },
        agent.clone(),
        created,
    ))));
    test_ok(store.create_resource(test_ok(Resource::new(
        resource_id.clone(),
        "Acme support portal",
        vec![AccessSurface::new(
            SurfaceKind::Browser,
            resource_id.clone(),
        )],
        user,
        created,
    ))));
    test_ok(store.put_resource_state(ResourceState {
        v: flauz_world::ContractVersion,
        resource_id: resource_id.clone(),
        resource_version: 1,
        permissions: vec!["dashboard:read".to_owned()],
        last_verified_observation: None,
        authoritative_surface: Some(SurfaceKind::Api),
        active_lease_ids: Vec::new(),
        pending_mutation_ids: Vec::new(),
        related_artifact_ids: Vec::new(),
        captured_by: agent.clone(),
        captured_at: created,
    }));
    test_ok(store.append_event(
        StreamRef::task(&task_id),
        test_ok(Event::new(
            event_types::TASK_MODEL_CHANGED,
            created,
            agent.clone(),
            EntityRef::task(&task_id),
            test_payload(&[(
                "to_model",
                CanonicalValue::from("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE"),
            )]),
        )),
    ));
    test_ok(store.record_observation(test_ok(Observation::new(
        test_ok(flauz_world::ObservationId::parse(
            "obs_01J8ZQ5V8K3T2B7N6X4R9DQPJ8",
        )),
        resource_id.clone(),
        SurfaceKind::Browser,
        agent.clone(),
        created,
        "The dashboard shows 3 open tickets",
    ))));
    test_ok(store.record_claim(test_ok(Claim::new(
        claim_id.clone(),
        resource_id,
        Some(SurfaceKind::Api),
        agent.clone(),
        created,
        "The ticket count matches the CRM export",
    ))));
    let claim = test_ok(store.claim(&claim_id));
    let Some(claim) = claim else {
        panic!("claim missing");
    };
    let verification = test_ok(store.verify_claim(
        claim,
        test_ok(ActorRef::user("user-alice")),
        created,
        StreamRef::task(&task_id),
        Payload::empty(),
    ));
    assert_eq!(
        verification.claim.verification,
        VerificationStatus::Verified
    );
    test_ok(store.grant_lease(test_ok(ResourceLease::new(
        test_ok(LeaseId::parse("lease_01J8ZQ5V8K3T2B7N6X4R9DQPNB")),
        verification.claim.resource_id.clone(),
        AccessMode::Write,
        agent.clone(),
        created,
        test_ok(Timestamp::parse("2026-09-21T14:45:00Z")),
        ConflictPolicy::Queue,
    ))));
    test_ok(store.create_procedure(test_ok(Procedure::new(
        test_ok(flauz_world::ProcedureId::parse(
            "proc_01J8ZQ5V8K3T2B7N6X4R9DQPPC",
        )),
        "Ticket reconciliation",
        test_procedure_version(None, 1, 0),
        agent,
        created,
    ))));
    store
}

fn test_procedure_version(
    predecessor: Option<flauz_world::SemanticVersion>,
    major: u32,
    minor: u32,
) -> ProcedureVersion {
    ProcedureVersion {
        major,
        minor,
        predecessor,
        objective: "Reconcile the weekly ticket counts".to_owned(),
        preconditions: vec!["Portal access is connected".to_owned()],
        inputs: vec![ProcedureInput {
            name: "week".to_owned(),
            kind: Some("text".to_owned()),
            description: None,
        }],
        steps: vec![],
        required_capabilities: vec![test_ok(flauz_world::CapabilityKey::parse("browser.input"))],
        resource_bindings: vec![],
        validation_rules: vec![],
        recovery_rules: vec![],
        evidence: vec![],
        created_by: test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created_at: test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    }
}
