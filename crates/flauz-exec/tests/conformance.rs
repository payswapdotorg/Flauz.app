//! Kernel conformance tests (F2-CONTRACT-KERNEL §9, exec side): the frozen
//! ID vectors, the versioned session/event transport, the shared runtime
//! and environment contracts over BOTH fakes, model/runtime separation,
//! secret references only, version monotonicity, and the
//! no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use flauz_exec::capability::CapabilityId;
use flauz_exec::fakes::{
    FakeCodexRuntime, FakeExecStore, FakeExecutionProvider, FakeLocalEnvironment,
    FakeModelProvider, FakeNonCodexRuntime, FakeRemoteEnvironment, fake_model_ids,
    fake_provider_connection,
};
use flauz_exec::ids::{self, EntityKind, EnvironmentId, IdError, ModelId};
use flauz_exec::refs::ActorRef;
use flauz_exec::runtime::{RuntimeRequest, RuntimeStatus};
use flauz_exec::time::Timestamp;
use flauz_exec::transport::SessionEventTransport;
use flauz_exec::{
    Agent, AgentRuntime, Environment, EnvironmentDescriptor, EnvironmentSpec, ExecStore,
    ExecStoreError, ExecutionProvider, Locality, Model, ModelProvider, ProviderConnection,
    SecretRef, Skill, SkillId,
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

fn test_timestamp() -> Timestamp {
    test_ok(Timestamp::parse("2026-09-21T13:45:00Z"))
}

fn test_actor() -> ActorRef {
    test_ok(ActorRef::user("alice"))
}

fn parse_capability(key: &str) -> CapabilityId {
    test_ok(CapabilityId::parse(key))
}

/// The frozen valid ID vectors (kernel §2), asserted through the generic
/// canonical-ID validator and, for exec-owned kinds, the typed parsers.
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

    // Exec-owned kinds also parse through their typed newtypes, and
    // serialization is the plain canonical string.
    let environment_id = test_ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4"));
    assert_eq!(
        test_ok(serde_json::to_string(&environment_id)),
        "\"env_01J8ZQ5V8K3T2B7N6X4R9DQPE4\""
    );
    let reloaded: EnvironmentId =
        test_ok(serde_json::from_str("\"env_01J8ZQ5V8K3T2B7N6X4R9DQPE4\""));
    assert_eq!(reloaded, environment_id);
    // Generated identifiers carry the exec kind prefixes.
    assert!(EnvironmentId::generate().as_str().starts_with("env_"));
    assert!(ModelId::generate().as_str().starts_with("model_"));
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
        assert!(EnvironmentId::parse(&vector.id).is_err());
        assert!(ModelId::parse(&vector.id).is_err());
    }
}

#[derive(serde::Deserialize)]
struct InvalidVector {
    id: String,
    reason: String,
}

/// A test-local stand-in for the world crate's `EventEnvelope` (kernel §5
/// shape, reduced to its scalar bones): the transport must treat it — and
/// any other `E` — as an opaque value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TestEventEnvelope {
    v: u32,
    kind: String,
    event_id: String,
    seq: u64,
    event_type: String,
    ts: String,
    actor_id: String,
    payload: std::collections::BTreeMap<String, String>,
}

fn test_event_envelope(seq: u64, event_type: &str) -> TestEventEnvelope {
    let mut payload = std::collections::BTreeMap::new();
    payload.insert("note".to_owned(), format!("turn {seq}"));
    TestEventEnvelope {
        v: 1,
        kind: "flauz.event".to_owned(),
        event_id: format!("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD{seq:02}"),
        seq,
        event_type: event_type.to_owned(),
        ts: "2026-09-21T13:45:00Z".to_owned(),
        actor_id: "agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6".to_owned(),
        payload,
    }
}

/// The transport round-trips envelope VALUES: serialize → frame → parse →
/// deserialize, with value equality and byte-stable envelope serialization
/// (kernel §5; acceptance 8).
#[test]
fn transport_roundtrip_preserves_envelope_value() {
    let transport = SessionEventTransport::<TestEventEnvelope>::new();

    let envelopes = vec![
        test_event_envelope(1, "environment.attached"),
        test_event_envelope(2, "task.model_changed"),
        test_event_envelope(42, "evidence.verified"),
    ];

    for envelope in &envelopes {
        // The envelope's own serialization before framing.
        let before_bytes = test_ok(serde_json::to_string(envelope));

        // serialize → frame → parse → deserialize.
        let frame_bytes = test_ok(transport.encode(envelope));
        let line: &[u8] = frame_bytes.strip_suffix(b"\n").unwrap_or(&frame_bytes);
        let restored = test_ok(transport.decode(line));

        // Value equality.
        assert_eq!(&restored, envelope);
        // Byte-stable envelope serialization after the round-trip.
        let after_bytes = test_ok(serde_json::to_string(&restored));
        assert_eq!(
            before_bytes, after_bytes,
            "envelope serialization must be byte-stable across the transport"
        );
    }

    // The same law holds over a stream of frames, in order, including a
    // tolerated trailing blank line.
    let mut stream: Vec<u8> = Vec::new();
    for envelope in &envelopes {
        test_ok(transport.write(envelope, &mut stream));
    }
    stream.push(b'\n');
    let mut reader = std::io::Cursor::new(&stream);
    let restored = test_ok(transport.read_up_to(&mut reader, 64));
    assert_eq!(restored, envelopes);

    // The transport is generic over ANY serde value — proving envelope
    // opacity with arbitrary JSON (this is how the world crate's
    // EventEnvelope will flow through at integration, with no exec-crate
    // trait implemented on the foreign type).
    let json_transport = SessionEventTransport::<serde_json::Value>::new();
    let opaque = serde_json::json!({
        "event_id": "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
        "seq": 42,
        "nested": {"deep": [true, 7, "x", null]}
    });
    let frame_bytes = test_ok(json_transport.encode(&opaque));
    let line: &[u8] = frame_bytes.strip_suffix(b"\n").unwrap_or(&frame_bytes);
    let restored = test_ok(json_transport.decode(line));
    assert_eq!(restored, opaque);

    // The frame carries its own version field; a future framing version is
    // rejected without touching the envelope.
    let future = "{\"v\":2,\"kind\":\"flauz.transport.frame\",\"envelope\":{}}";
    assert!(transport.decode(future.as_bytes()).is_err());
}

/// The shared [`AgentRuntime`] conformance, parameterized over BOTH fake
/// runtimes: the fake Codex app-server runtime AND the fake non-Codex
/// direct-model runtime satisfy the SAME contract (kernel §9; acceptance 3).
#[test]
fn fake_runtimes_satisfy_agent_runtime_contract() {
    let codex = FakeCodexRuntime::new();
    let direct = FakeNonCodexRuntime::new();
    let runtimes: [&dyn AgentRuntime; 2] = [&codex, &direct];
    assert_ne!(
        runtimes[0].runtime_kind(),
        runtimes[1].runtime_kind(),
        "the two fakes must represent different runtimes"
    );

    for runtime in runtimes {
        let kind = runtime.runtime_kind();
        assert!(!kind.is_empty(), "runtime kind must not be empty");
        assert!(
            kind.chars()
                .next()
                .is_some_and(|first| first.is_ascii_lowercase()),
            "runtime kind {kind:?} must be a lowercase token"
        );

        // Capability advertisement: valid, sorted, deduplicated, non-empty.
        let capabilities = runtime.capabilities();
        assert!(
            !capabilities.is_empty(),
            "{kind} must advertise capabilities"
        );
        for capability in capabilities {
            test_ok(capability.validate());
        }
        assert!(
            capabilities.windows(2).all(|pair| pair[0] < pair[1]),
            "{kind} capabilities must be sorted and deduplicated"
        );

        let model_id = test_ok(ModelId::parse(fake_model_ids::TEXT));

        // A satisfiable request completes and echoes the model, the
        // environment and the runtime kind.
        let required: Vec<CapabilityId> = capabilities.iter().take(2).cloned().collect();
        let request = test_ok(RuntimeRequest::new(
            model_id.clone(),
            Some(test_ok(EnvironmentId::parse(
                "env_01J8ZQ5V8K3T2B7N6X4R9DQPE4",
            ))),
            required.clone(),
            "Run the reconciliation step",
        ));
        let outcome = test_ok(runtime.execute(&request));
        assert_eq!(outcome.status, RuntimeStatus::Completed, "{kind}");
        assert_eq!(outcome.runtime_kind, kind);
        assert_eq!(outcome.model_id, model_id);
        assert_eq!(outcome.environment_id, request.environment_id);
        assert!(outcome.missing_capabilities.is_empty());
        test_ok(outcome.validate());
        // Outcomes are canonical serialized state: they round-trip.
        let serialized = test_ok(serde_json::to_string(&outcome));
        let reloaded: flauz_exec::RuntimeOutcome = test_ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, outcome);

        // An unsatisfiable request produces exactly the capability gap —
        // the actionable diagnosis surface.
        let gap_request = test_ok(RuntimeRequest::new(
            model_id.clone(),
            None,
            vec![parse_capability("mcp")],
            "Talk to the MCP server",
        ));
        let gap = test_ok(runtime.execute(&gap_request));
        assert_eq!(gap.status, RuntimeStatus::CapabilityGap, "{kind}");
        assert_eq!(gap.missing_capabilities, vec![parse_capability("mcp")]);
        assert_eq!(gap.model_id, model_id);
        test_ok(gap.validate());

        // A structurally invalid request is a contract error, not an
        // outcome.
        let invalid = RuntimeRequest {
            v: flauz_exec::ContractVersion,
            model_id: model_id.clone(),
            environment_id: None,
            required_capabilities: Vec::new(),
            instruction: String::new(),
        };
        assert!(runtime.execute(&invalid).is_err(), "{kind}");
    }
}

/// The shared [`Environment`] conformance, parameterized over BOTH fake
/// environments: the local AND the remote fake satisfy the SAME contract
/// (kernel §9; acceptance 4).
#[test]
fn environment_contract_same_for_local_and_remote_fakes() {
    let local = test_ok(FakeLocalEnvironment::with_default_id());
    let remote = test_ok(FakeRemoteEnvironment::with_default_id());
    let environments: [&dyn Environment; 2] = [&local, &remote];

    for environment in environments {
        // Identity: a canonical env_ ID, opaque, kind-verified.
        let id = environment.environment_id();
        assert_eq!(test_ok(ids::validate(id.as_str())), EntityKind::Environment);

        // The live surface and the durable descriptor agree.
        let descriptor = environment.descriptor();
        assert_eq!(descriptor.id, *id);
        assert_eq!(descriptor.locality, environment.locality());
        assert_eq!(descriptor.status, environment.status());
        assert_eq!(
            descriptor.capabilities.as_slice(),
            environment.capabilities()
        );
        test_ok(descriptor.validate());

        // Capability advertisement: valid, sorted, deduplicated, non-empty.
        let capabilities = environment.capabilities();
        assert!(
            !capabilities.is_empty(),
            "environment {id} must advertise capabilities"
        );
        for capability in capabilities {
            test_ok(capability.validate());
        }
        assert!(
            capabilities.windows(2).all(|pair| pair[0] < pair[1]),
            "environment {id} capabilities must be sorted and deduplicated"
        );

        // The descriptor is canonical serialized state: it round-trips.
        let serialized = test_ok(serde_json::to_string(&descriptor));
        let reloaded: EnvironmentDescriptor = test_ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, descriptor);
    }

    // The two fakes differ exactly where the concept differs: locality.
    assert_eq!(local.locality(), Locality::Local);
    assert_eq!(remote.locality(), Locality::Remote);
}

/// A Model is intelligence only: no provider-specific environment
/// behavior, and no environment determines a Model (structural separation,
/// tested — acceptance 5).
#[test]
fn model_has_no_provider_environment_behavior() {
    // (a) Models are constructed with NO environment, runtime or
    // connection in scope: Model::new has no such parameters at all.
    let provider = test_ok(FakeModelProvider::new());
    let models = provider.fake_models().to_vec();
    assert_eq!(models.len(), 2);
    assert_ne!(
        models[0].capabilities, models[1].capabilities,
        "the fake models must offer different capability sets"
    );

    for model in &models {
        // (b) The serialized field set is exactly the model's own fields —
        // no environment, locality, runtime, connection or execution state.
        let serialized = test_ok(serde_json::to_string(model));
        let value: serde_json::Value = test_ok(serde_json::from_str(&serialized));
        let object = value
            .as_object()
            .unwrap_or_else(|| panic!("model must serialize to an object"));
        let expected_fields = [
            "v",
            "id",
            "version",
            "name",
            "provider_kind",
            "capabilities",
            "description",
            "created_by",
            "created_at",
        ];
        assert_eq!(object.len(), expected_fields.len());
        for field in expected_fields {
            assert!(object.contains_key(field), "model is missing {field}");
        }
        for forbidden in [
            "environment",
            "environment_id",
            "locality",
            "runtime",
            "runtime_kind",
            "connection",
            "connection_id",
            "execution",
        ] {
            assert!(
                !object.contains_key(forbidden),
                "a model must never carry {forbidden} state"
            );
        }

        // (c) Canonical round-trip.
        let reloaded: Model = test_ok(serde_json::from_str(&serialized));
        assert_eq!(&reloaded, model);
    }

    // (d) Environments never determine models: switching the bound
    // environment of a runtime turn changes the model VALUE not at all —
    // the model is passed by value and comes back unchanged.
    let text_model = models[0].clone();
    let runtime = FakeCodexRuntime::new();
    let local = test_ok(FakeLocalEnvironment::with_default_id());
    let remote = test_ok(FakeRemoteEnvironment::with_default_id());
    for environment in [&local as &dyn Environment, &remote] {
        let request = test_ok(RuntimeRequest::new(
            text_model.id.clone(),
            Some(environment.environment_id().clone()),
            vec![parse_capability("terminal")],
            "Run the tests",
        ));
        let outcome = test_ok(runtime.execute(&request));
        assert_eq!(
            outcome.environment_id,
            Some(environment.environment_id().clone())
        );
        // The model value is untouched by the environment it ran in.
        assert_eq!(provider.model(&text_model.id), Some(text_model.clone()));
    }

    // (e) The environment record carries no model state either — the
    // separation is mutual and structural.
    for descriptor in [local.descriptor(), remote.descriptor()] {
        let serialized = test_ok(serde_json::to_string(&descriptor));
        for forbidden in ["model", "model_id", "models", "runtime", "agent"] {
            assert!(
                !serialized.contains(&format!("\"{forbidden}\"")),
                "an environment record must never carry {forbidden} state"
            );
        }
    }

    // (f) And an execution provider sources environments without any
    // model existing in the process at all.
    let mut execution_provider = FakeExecutionProvider::new();
    let spec = test_ok(EnvironmentSpec::new(
        "Isolated runner",
        Locality::Remote,
        vec![parse_capability("ports")],
        test_actor(),
        test_timestamp(),
    ));
    let sourced = test_ok(execution_provider.source_environment(&spec));
    test_ok(sourced.validate());
}

/// Credentials are represented ONLY as opaque `flausec_` references
/// (kernel §7; acceptance 7).
#[test]
fn provider_connection_holds_only_secret_references() {
    // The fake connection and the fixture serialize exactly the reference
    // fields — nothing else can even appear.
    let connection = test_ok(fake_provider_connection());
    let serialized = test_ok(serde_json::to_string(&connection));
    let value: serde_json::Value = test_ok(serde_json::from_str(&serialized));
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("connection must serialize to an object"));
    let expected_fields = [
        "v",
        "id",
        "version",
        "provider_kind",
        "account_label",
        "secret_ref",
        "created_by",
        "created_at",
    ];
    assert_eq!(object.len(), expected_fields.len());
    for field in expected_fields {
        assert!(object.contains_key(field), "connection is missing {field}");
    }
    assert!(
        object["secret_ref"]
            .as_str()
            .is_some_and(|reference| reference.starts_with(SecretRef::PREFIX)),
        "the secret reference must be an opaque flausec_ token"
    );

    // The committed fixture carries the same shape.
    let fixture = read_fixture("connection/typical.json");
    let parsed: ProviderConnection = test_ok(serde_json::from_str(&fixture));
    test_ok(parsed.validate());
    assert!(parsed.secret_ref.as_str().starts_with("flausec_"));

    // Raw credential material cannot be smuggled into a secret reference:
    // every classic shape is rejected at parse time.
    for raw_credential in [
        "sk-proj-abcdefgh123456",
        "Bearer abcdef123456",
        "ghp_0123456789abcdef",
        "xoxb-1234567890",
        "flausec_sk-proj-abcdefgh123456",
        "flausec_Bearer token",
        "flausec_api_keyvalue",
        "",
    ] {
        assert!(
            SecretRef::parse(raw_credential).is_err(),
            "{raw_credential:?} must never parse as a secret reference"
        );
    }

    // The committed invalid fixture (a reference without the flausec_
    // prefix, chosen to contain no credential markers itself) fails the
    // strict parse.
    let invalid = read_fixture("connection/invalid-secret-ref.json");
    assert!(
        serde_json::from_str::<ProviderConnection>(&invalid).is_err(),
        "a non-flausec secret reference must fail the canonical parse"
    );
}

/// Durable entities start at version 1, increment by exactly one per
/// mutation, and stale expected versions are rejected with a
/// `VersionConflict` — never silently overwritten (kernel §3; acceptance
/// tested for all four exec entity kinds).
#[test]
fn version_monotonic_and_version_conflict_rejected() {
    let mut store = FakeExecStore::new();

    // Environments.
    let environment = test_ok(EnvironmentDescriptor::new(
        test_ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4")),
        "Local Linux workstation",
        Locality::Local,
        Some("local"),
        vec![parse_capability("terminal")],
        flauz_exec::EnvironmentStatus::Available,
        None,
        test_actor(),
        test_timestamp(),
    ));
    let created = test_ok(store.create_environment(environment.clone()));
    assert_eq!(created.version, 1, "entities are created at version 1");
    let mut renamed = created.clone();
    renamed.name = "Local Linux workstation 2".to_owned();
    let updated = test_ok(store.update_environment(renamed));
    assert_eq!(updated.version, 2, "+1 per durable mutation");
    let conflict = store
        .update_environment(created)
        .err()
        .unwrap_or_else(|| panic!("a stale expected version must conflict"));
    assert!(matches!(conflict, ExecStoreError::VersionConflict { .. }));

    // Models.
    let model = test_ok(Model::new(
        test_ok(ModelId::parse(fake_model_ids::TEXT)),
        "Fake Text Model",
        "flauz-fake",
        Vec::new(),
        None,
        test_actor(),
        test_timestamp(),
    ));
    let created = test_ok(store.create_model(model.clone()));
    assert_eq!(created.version, 1);
    let mut redescribed = created.clone();
    redescribed.description = Some("Updated description".to_owned());
    let updated = test_ok(store.update_model(redescribed));
    assert_eq!(updated.version, 2);
    let conflict = store
        .update_model(created)
        .err()
        .unwrap_or_else(|| panic!("a stale expected version must conflict"));
    assert!(matches!(conflict, ExecStoreError::VersionConflict { .. }));

    // Agents — including the model switch, which bumps the version and
    // never changes the identity.
    let agent = test_ok(Agent::new(
        test_ok(flauz_exec::AgentId::parse(
            "agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6",
        )),
        "Reconciliation agent",
        Some("codex-app-server"),
        Some(test_ok(ModelId::parse(fake_model_ids::TEXT))),
        test_actor(),
        test_timestamp(),
    ));
    let created = test_ok(store.create_agent(agent.clone()));
    assert_eq!(created.version, 1);
    let mut switched = created.clone();
    switched.model_id = Some(test_ok(ModelId::parse(fake_model_ids::VISION)));
    let updated = test_ok(store.update_agent(switched));
    assert_eq!(updated.version, 2);
    assert_eq!(
        updated.id, agent.id,
        "a model switch never changes identity"
    );
    let conflict = store
        .update_agent(created)
        .err()
        .unwrap_or_else(|| panic!("a stale expected version must conflict"));
    assert!(matches!(conflict, ExecStoreError::VersionConflict { .. }));

    // Provider connections.
    let connection = test_ok(fake_provider_connection());
    let created = test_ok(store.create_connection(connection.clone()));
    assert_eq!(created.version, 1);
    let mut relabeled = created.clone();
    relabeled.account_label = "Personal sandbox account".to_owned();
    let updated = test_ok(store.update_connection(relabeled));
    assert_eq!(updated.version, 2);
    let conflict = store
        .update_connection(created)
        .err()
        .unwrap_or_else(|| panic!("a stale expected version must conflict"));
    assert!(
        matches!(conflict, ExecStoreError::VersionConflict { .. }),
        "silent overwrite is forbidden"
    );

    // Creation at a non-1 version is rejected outright.
    let mut poisoned = updated.clone();
    poisoned.version = 7;
    let rejected = store
        .update_connection(poisoned)
        .err()
        .unwrap_or_else(|| panic!("this update must fail"));
    assert!(matches!(rejected, ExecStoreError::VersionConflict { .. }));
}

/// No contract type, fixture, or serialized state contains credential
/// material: only opaque references are ever stored (kernel §7).
#[test]
fn no_credential_material_in_serialized_state() {
    let store = test_exec_state_with_everything();
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

/// Skill semantics are provider- and model-neutral (acceptance 6): the
/// serialized skill carries requirement data only.
#[test]
fn skill_semantics_are_provider_and_model_neutral() {
    let skill = test_ok(Skill::new(
        test_ok(SkillId::parse("flauz.research.collect")),
        flauz_exec::SemanticVersion::FIRST,
        "Collect research",
        Some("Collect and reconcile evidence from web sources"),
        vec![
            parse_capability("browser.input"),
            parse_capability("web.search"),
        ],
        test_actor(),
        test_timestamp(),
    ));
    test_ok(skill.validate());

    let serialized = test_ok(serde_json::to_string(&skill));
    let value: serde_json::Value = test_ok(serde_json::from_str(&serialized));
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("skill must serialize to an object"));
    let expected_fields = [
        "v",
        "id",
        "semantic_version",
        "name",
        "description",
        "required_capabilities",
        "created_by",
        "created_at",
    ];
    assert_eq!(object.len(), expected_fields.len());
    for field in expected_fields {
        assert!(object.contains_key(field), "skill is missing {field}");
    }
    for forbidden in [
        "provider",
        "provider_kind",
        "model",
        "model_id",
        "runtime",
        "runtime_kind",
        "environment",
        "environment_id",
        "connection",
    ] {
        assert!(
            !object.contains_key(forbidden),
            "a skill must never carry {forbidden} state"
        );
    }

    // The same neutrality holds for the committed fixtures.
    for relative in ["skill/minimal.json", "skill/typical.json"] {
        let content = read_fixture(relative);
        let parsed: Skill = test_ok(serde_json::from_str(&content));
        test_ok(parsed.validate());
        for forbidden in ["provider", "model", "runtime", "environment"] {
            assert!(
                !content.contains(&format!("\"{forbidden}")),
                "skill fixture {relative} must not carry {forbidden} state"
            );
        }
    }
}

/// Every conformance fixture is canonical JSON: it parses strictly and
/// re-serializes to exactly the committed bytes (kernel §4, §8).
#[test]
fn fixtures_roundtrip_canonical() {
    let checks: &[(&str, FixtureKind)] = &[
        ("capability/typical.json", FixtureKind::Capability),
        ("environment/minimal.json", FixtureKind::Environment),
        ("environment/typical.json", FixtureKind::Environment),
        ("model/typical.json", FixtureKind::Model),
        ("agent/typical.json", FixtureKind::Agent),
        ("agent/minimal.json", FixtureKind::Agent),
        ("skill/minimal.json", FixtureKind::Skill),
        ("skill/typical.json", FixtureKind::Skill),
        ("connection/typical.json", FixtureKind::Connection),
        ("runtime-request/typical.json", FixtureKind::RuntimeRequest),
        ("runtime-outcome/typical.json", FixtureKind::RuntimeOutcome),
        ("runtime-outcome/boundary.json", FixtureKind::RuntimeOutcome),
        ("transport-frame/typical.json", FixtureKind::TransportFrame),
    ];

    for (relative, kind) in checks {
        let content = read_fixture(relative);
        let serialized = match kind {
            FixtureKind::Capability => round_trip::<flauz_exec::Capability>(&content),
            FixtureKind::Environment => round_trip::<EnvironmentDescriptor>(&content),
            FixtureKind::Model => round_trip::<Model>(&content),
            FixtureKind::Agent => round_trip::<Agent>(&content),
            FixtureKind::Skill => round_trip::<Skill>(&content),
            FixtureKind::Connection => round_trip::<ProviderConnection>(&content),
            FixtureKind::RuntimeRequest => round_trip::<RuntimeRequest>(&content),
            FixtureKind::RuntimeOutcome => round_trip::<flauz_exec::RuntimeOutcome>(&content),
            FixtureKind::TransportFrame => {
                round_trip::<flauz_exec::TransportFrame<serde_json::Value>>(&content)
            }
        };
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }

    // Invalid fixtures must fail strict parses.
    let invalid_connection = read_fixture("connection/invalid-secret-ref.json");
    assert!(
        serde_json::from_str::<ProviderConnection>(&invalid_connection).is_err(),
        "a non-flausec secret reference must fail the strict parse"
    );
}

enum FixtureKind {
    Capability,
    Environment,
    Model,
    Agent,
    Skill,
    Connection,
    RuntimeRequest,
    RuntimeOutcome,
    TransportFrame,
}

fn round_trip<T>(content: &str) -> String
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = test_ok(serde_json::from_str(content));
    test_ok(serde_json::to_string_pretty(&parsed))
}

/// Builds an execution state exercising every entity kind, mirroring the F2
/// gate flow.
fn test_exec_state_with_everything() -> FakeExecStore {
    let created = test_timestamp();
    let mut store = FakeExecStore::new();

    // Both environment fakes, through their descriptors.
    let local = test_ok(FakeLocalEnvironment::with_default_id());
    let remote = test_ok(FakeRemoteEnvironment::with_default_id());
    test_ok(store.create_environment(local.descriptor()));
    test_ok(store.create_environment(remote.descriptor()));

    // A sourced environment from the fake execution provider.
    let mut provider = FakeExecutionProvider::new();
    let spec = test_ok(EnvironmentSpec::new(
        "CI runner",
        Locality::Remote,
        vec![parse_capability("ports")],
        test_actor(),
        created,
    ));
    test_ok(store.create_environment(test_ok(provider.source_environment(&spec))));

    // Both fake models.
    let model_provider = test_ok(FakeModelProvider::new());
    for model in model_provider.models() {
        test_ok(store.create_model(model.clone()));
    }

    // Agents on both runtimes — the Codex and the non-Codex one.
    test_ok(store.create_agent(test_ok(Agent::new(
        test_ok(flauz_exec::AgentId::parse(
            "agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6",
        )),
        "Codex agent",
        Some(FakeCodexRuntime::new().runtime_kind()),
        Some(test_ok(ModelId::parse(fake_model_ids::TEXT))),
        test_actor(),
        created,
    ))));
    test_ok(store.create_agent(test_ok(Agent::new(
        test_ok(flauz_exec::AgentId::parse(
            "agent_01J8ZQ5V8K3T2B7N6X4R9DQPZ2",
        )),
        "Direct agent",
        Some(FakeNonCodexRuntime::new().runtime_kind()),
        Some(test_ok(ModelId::parse(fake_model_ids::VISION))),
        test_actor(),
        created,
    ))));

    // A provider connection with an opaque secret reference.
    test_ok(store.create_connection(test_ok(fake_provider_connection())));

    store
}
