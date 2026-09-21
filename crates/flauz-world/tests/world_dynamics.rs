//! World dynamics conformance (F2-CONTRACT-KERNEL §9, world side):
//! versioning, task identity, the claim/evidence separation, conflicting
//! observations, leases, resource surface changes, procedure lineage, and
//! the serialize → drop → reload round-trip.

use std::fmt;

use flauz_world::SemanticVersion;
use flauz_world::event::event_types;
use flauz_world::fakes::FakeWorldStore;
use flauz_world::ids::{
    ArtifactId, ClaimId, EventId, EvidenceId, LeaseId, ObservationId, ResourceId, SessionId,
    TaskId, WorkspaceId,
};
use flauz_world::procedure::ProcedureInput;
use flauz_world::refs::{ActorRef, EntityRef, StreamRef};
use flauz_world::resource::{AccessSurface, ResourceState, SurfaceKind};
use flauz_world::time::Timestamp;
use flauz_world::value::{CanonicalValue, Payload};
use flauz_world::{
    AccessMode, Artifact, ArtifactContent, Claim, ConflictPolicy, Event, Evidence, Observation,
    Procedure, ProcedureVersion, Resource, ResourceLease, Session, Task, VerificationStatus,
    Workspace, WorldSnapshot, WorldStore, WorldStoreError,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn test_ts(value: &str) -> Timestamp {
    test_ok(Timestamp::parse(value))
}

fn test_payload(entries: &[(&str, CanonicalValue)]) -> Payload {
    let mut payload = Payload::empty();
    for (key, value) in entries {
        payload = test_ok(payload.with(key, value.clone()));
    }
    payload
}

/// Entity versions start at 1, increment by exactly one per durable
/// mutation, and stale updates are rejected with a version conflict.
#[test]
fn version_monotonic_and_version_conflict_rejected() {
    let created = test_ts("2026-09-21T13:45:00Z");
    let mut store = FakeWorldStore::new();
    let task_id = TaskId::generate();
    let task = test_ok(Task::new(
        task_id.clone(),
        WorkspaceId::generate(),
        "Reconcile the ticket counts",
        test_ok(ActorRef::user("alice")),
        created,
    ));
    let created_task = test_ok(store.create_task(task.clone()));
    assert_eq!(created_task.version, 1);

    // Monotonic: two durable mutations, +1 each.
    let mut updated = test_ok(store.update_task(created_task.clone()));
    assert_eq!(updated.version, 2);
    updated.objective = "Reconcile the ticket counts across all queues".to_owned();
    test_ok(updated.validate());
    updated = test_ok(store.update_task(updated));
    assert_eq!(updated.version, 3);

    // The stale copy (version 2) conflicts with the stored version 3:
    // silent overwrite is forbidden.
    let mut stale = updated.clone();
    stale.version = 2;
    let conflict = store.update_task(stale.clone());
    assert_eq!(
        conflict.err(),
        Some(WorldStoreError::VersionConflict {
            id: task_id.to_string(),
            expected_version: 2,
            actual_version: 3,
        })
    );
    let stored = test_ok(store.task(&task_id));
    let Some(stored) = stored else {
        panic!("task missing");
    };
    assert_eq!(stored.version, 3);
    assert_eq!(
        stored.objective,
        "Reconcile the ticket counts across all queues"
    );

    // Creating an entity twice is rejected as a duplicate; entities must
    // be created at version 1.
    assert_eq!(
        store.create_task(updated.clone()).err(),
        Some(WorldStoreError::Duplicate {
            id: task_id.to_string(),
        })
    );
    let mut not_fresh = test_ok(Task::new(
        TaskId::generate(),
        WorkspaceId::generate(),
        "Fresh",
        test_ok(ActorRef::user("alice")),
        created,
    ));
    not_fresh.version = 7;
    assert!(matches!(
        store.create_task(not_fresh).err(),
        Some(WorldStoreError::Invalid { .. })
    ));
}

/// A model switch is an event on the task stream: the task ID, entity and
/// event history are preserved across serialize → drop → reload, with no
/// fork.
#[test]
fn task_identity_survives_model_switch() {
    let created = test_ts("2026-09-21T13:45:00Z");
    let ws_id = WorkspaceId::generate();
    let task_id = TaskId::generate();
    let mut store = FakeWorldStore::new();
    test_ok(store.create_workspace(test_ok(Workspace::new(
        ws_id.clone(),
        "Research",
        test_ok(ActorRef::user("alice")),
        created,
    ))));
    let task = test_ok(store.create_task(test_ok(Task::new(
        task_id.clone(),
        ws_id,
        "Reconcile the ticket counts",
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
    ))));
    let before = task.clone();

    // The model switch: an event, never an identity change.
    let switch = test_ok(store.append_event(
        StreamRef::task(&task_id),
        test_ok(Event::new(
            event_types::TASK_MODEL_CHANGED,
            test_ts("2026-09-21T14:00:00Z"),
            test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
            EntityRef::task(&task_id),
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
        )),
    ));
    assert_eq!(switch.seq, 1);

    // Serialize, drop, reload.
    let json = test_ok(serde_json::to_string(&store.snapshot()));
    drop(store);
    let snapshot: WorldSnapshot = test_ok(serde_json::from_str(&json));
    let mut reloaded = test_ok(FakeWorldStore::restore(snapshot));

    let after = test_ok(reloaded.task(&task_id));
    let Some(after) = after else {
        panic!("task missing after reload");
    };
    assert_eq!(after.id, before.id, "task identity is the canonical ID");
    assert_eq!(after, before, "a model switch is an event, not a mutation");
    let events = test_ok(reloaded.events(&StreamRef::task(&task_id), 0, 10));
    assert_eq!(events.len(), 1, "no fork: exactly one task stream");
    assert_eq!(events[0].event_id, switch.event_id);
    assert_eq!(
        events[0].event_type.as_str(),
        event_types::TASK_MODEL_CHANGED
    );

    // Continuing after the switch appends to the same stream, and the
    // identity still holds.
    let continued = test_ok(reloaded.append_event(
        StreamRef::task(&task_id),
        test_ok(Event::new(
            event_types::TASK_CONTEXT_RESET,
            test_ts("2026-09-21T14:05:00Z"),
            test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
            EntityRef::task(&task_id),
            Payload::empty(),
        )),
    ));
    assert_eq!(continued.seq, 2, "sequence continues after the reload");
    let final_task = test_ok(reloaded.task(&task_id));
    let Some(final_task) = final_task else {
        panic!("task missing after continuation");
    };
    assert_eq!(final_task.id, after.id);
}

/// A Claim can never be constructed, serialized, or parsed as Evidence.
#[test]
fn claim_cannot_be_constructed_as_evidence() {
    let created = test_ts("2026-09-21T13:45:00Z");
    let resource_id = ResourceId::generate();
    let claim = test_ok(Claim::new(
        ClaimId::generate(),
        resource_id,
        Some(SurfaceKind::Api),
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
        "The ticket count matches the CRM export",
    ));

    // Serialization level: claim JSON never parses as evidence (missing
    // verifier/verification_event_id/verified_at, unknown claim fields).
    let claim_json = test_ok(serde_json::to_string(&claim));
    assert!(serde_json::from_str::<Evidence>(&claim_json).is_err());

    // Even a claim marked verified is still a claim: the verification
    // status is state, not evidence identity.
    let mut verified_claim = claim.clone();
    verified_claim.verification = VerificationStatus::Verified;
    let verified_json = test_ok(serde_json::to_string(&verified_claim));
    assert!(serde_json::from_str::<Evidence>(&verified_json).is_err());

    // The reverse direction holds too: evidence never parses as a claim.
    let evidence = test_ok(Evidence::new(
        EvidenceId::generate(),
        claim.id.clone(),
        test_ok(ActorRef::user("alice")),
        EventId::generate(),
        created,
    ));
    let evidence_json = test_ok(serde_json::to_string(&evidence));
    assert!(serde_json::from_str::<Claim>(&evidence_json).is_err());

    // Type level: constructing Evidence requires the three verification
    // anchors (verifier actor, verification event reference, timestamp) —
    // a Claim value carries none of them, and no conversion exists. The
    // evidence fixture and the store path (verify_claim is the only way
    // evidence enters a store) both enforce this.
    let mut store = FakeWorldStore::new();
    let stored = test_ok(store.record_claim(claim));
    let verification = test_ok(store.verify_claim(
        stored,
        test_ok(ActorRef::user("alice")),
        created,
        StreamRef::task(&TaskId::generate()),
        Payload::empty(),
    ));
    assert_eq!(
        verification.evidence.verifier,
        test_ok(ActorRef::user("alice"))
    );
    let reloaded_evidence = test_ok(store.evidence(&verification.evidence.id));
    let Some(reloaded_evidence) = reloaded_evidence else {
        panic!("evidence missing");
    };
    assert_eq!(reloaded_evidence, verification.evidence);
    let reloaded_claim = test_ok(store.claim(&verification.claim.id));
    let Some(reloaded_claim) = reloaded_claim else {
        panic!("claim missing");
    };
    assert_eq!(reloaded_claim.verification, VerificationStatus::Verified);
    assert_eq!(
        reloaded_claim.verifying_evidence_id,
        Some(verification.evidence.id)
    );
}

/// Conflicting observations about the same subject stay distinct records
/// with surface attribution, across reloads.
#[test]
fn conflicting_observations_stay_distinct() {
    let created = test_ts("2026-09-21T13:45:00Z");
    let resource_id = ResourceId::generate();
    let mut store = FakeWorldStore::new();

    let first = test_ok(store.record_observation(test_ok(Observation::new(
        ObservationId::generate(),
        resource_id.clone(),
        SurfaceKind::Browser,
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
        "The dashboard shows 3 open tickets",
    ))));
    let second = test_ok(store.record_observation(test_ok(Observation::new(
        ObservationId::generate(),
        resource_id.clone(),
        SurfaceKind::Api,
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPTG")),
        test_ts("2026-09-21T13:46:00Z"),
        "The API reports 5 open tickets",
    ))));

    assert_ne!(first.id, second.id, "no merging, no silent dedup");
    assert_ne!(first, second);
    assert_eq!(first.surface, SurfaceKind::Browser);
    assert_eq!(second.surface, SurfaceKind::Api);
    assert_eq!(first.statement, "The dashboard shows 3 open tickets");
    assert_eq!(second.statement, "The API reports 5 open tickets");

    // Both survive a serialize → drop → reload round-trip.
    let json = test_ok(serde_json::to_string(&store.snapshot()));
    drop(store);
    let snapshot: WorldSnapshot = test_ok(serde_json::from_str(&json));
    let reloaded = test_ok(FakeWorldStore::restore(snapshot));
    let observations = [
        test_ok(reloaded.observation(&first.id)).unwrap_or_else(|| panic!("first missing")),
        test_ok(reloaded.observation(&second.id)).unwrap_or_else(|| panic!("second missing")),
    ];
    assert_eq!(observations[0], first);
    assert_eq!(observations[1], second);
    assert_ne!(
        observations[0], observations[1],
        "still distinct after reload"
    );
}

/// Lease expiry is enforced on read: at and after the absolute deadline the
/// lease is not held, and there is no implicit renewal.
#[test]
fn lease_expiry_enforced_on_read() {
    let granted = test_ts("2026-09-21T13:45:00Z");
    let expires = test_ts("2026-09-21T14:45:00Z");
    let at_deadline = expires;
    let after_deadline = test_ts("2026-09-21T14:45:01Z");
    let resource_id = ResourceId::generate();
    let lease_id = LeaseId::generate();
    let mut store = FakeWorldStore::new();

    test_ok(store.grant_lease(test_ok(ResourceLease::new(
        lease_id.clone(),
        resource_id.clone(),
        AccessMode::Write,
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        granted,
        expires,
        ConflictPolicy::Queue,
    ))));

    let before_deadline = test_ts("2026-09-21T14:44:59Z");
    assert!(test_ok(store.lease(&lease_id, &before_deadline)).is_some());
    assert_eq!(
        test_ok(store.active_leases(&resource_id, &before_deadline)).len(),
        1
    );
    // Exactly at the deadline the lease is no longer held.
    assert!(
        test_ok(store.lease(&lease_id, &at_deadline)).is_none(),
        "an expired lease is not held (deadline inclusive)"
    );
    assert!(test_ok(store.lease(&lease_id, &after_deadline)).is_none());
    assert_eq!(
        test_ok(store.active_leases(&resource_id, &after_deadline)).len(),
        0
    );
    // The raw record survives for provenance.
    assert!(test_ok(store.lease_record(&lease_id)).is_some());

    // A released lease is not held either, even before its deadline.
    let second = LeaseId::generate();
    test_ok(store.grant_lease(test_ok(ResourceLease::new(
        second.clone(),
        resource_id.clone(),
        AccessMode::Read,
        test_ok(ActorRef::user("alice")),
        granted,
        expires,
        ConflictPolicy::Reject,
    ))));
    let released = test_ok(store.release_lease(&second, before_deadline));
    assert_eq!(released.released_at, Some(before_deadline));
    assert!(test_ok(store.lease(&second, &before_deadline)).is_none());
    // Releasing again fails: it is no longer held.
    assert!(store.release_lease(&second, before_deadline).is_err());
}

/// Every lease is bounded and attributable: absolute deadline, owner actor,
/// resource scope, access mode, conflict policy.
#[test]
fn lease_is_attributable_and_bounded() {
    let granted = test_ts("2026-09-21T13:45:00Z");
    let resource_id = ResourceId::generate();
    let owner = test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD"));
    let lease = test_ok(ResourceLease::new(
        LeaseId::generate(),
        resource_id.clone(),
        AccessMode::Commit,
        owner.clone(),
        granted,
        test_ts("2026-09-21T13:46:00Z"),
        ConflictPolicy::Escalate,
    ));

    assert_eq!(
        lease.owner, owner,
        "leases are attributable to an owner actor"
    );
    assert_eq!(lease.resource_id, resource_id);
    assert_eq!(lease.mode, AccessMode::Commit);
    assert_eq!(lease.conflict_policy, ConflictPolicy::Escalate);
    assert!(lease.expires_at > lease.granted_at, "absolute deadline");

    // Unbounded or inverted leases cannot be constructed or stored.
    assert!(
        ResourceLease::new(
            LeaseId::generate(),
            resource_id.clone(),
            AccessMode::Read,
            owner.clone(),
            granted,
            granted,
            ConflictPolicy::Queue
        )
        .is_err()
    );
    assert!(
        ResourceLease::new(
            LeaseId::generate(),
            resource_id.clone(),
            AccessMode::Read,
            owner.clone(),
            granted,
            test_ts("2026-09-21T13:44:59Z"),
            ConflictPolicy::Queue
        )
        .is_err()
    );

    let mut store = FakeWorldStore::new();
    let stored = test_ok(store.grant_lease(lease));
    let json = test_ok(serde_json::to_string(&store.snapshot()));
    drop(store);
    let snapshot: WorldSnapshot = test_ok(serde_json::from_str(&json));
    let reloaded = test_ok(FakeWorldStore::restore(snapshot));
    let record = test_ok(reloaded.lease_record(&stored.id));
    let Some(record) = record else {
        panic!("lease record missing after reload");
    };
    assert_eq!(record.owner, owner, "attribution survives the reload");
    assert_eq!(record.expires_at, stored.expires_at);
}

/// A resource's identity is stable when its access surfaces change; the
/// surface change is a durable mutation of the same entity.
#[test]
fn resource_identity_stable_across_surface_changes() {
    let created = test_ts("2026-09-21T13:45:00Z");
    let resource_id = ResourceId::generate();
    let mut store = FakeWorldStore::new();
    let resource = test_ok(store.create_resource(test_ok(Resource::new(
        resource_id.clone(),
        "Acme support portal",
        vec![AccessSurface::new(
            SurfaceKind::Browser,
            resource_id.clone(),
        )],
        test_ok(ActorRef::user("alice")),
        created,
    ))));
    assert_eq!(resource.version, 1);
    assert_eq!(resource.id, resource_id);
    assert_eq!(resource.surfaces.len(), 1);
    assert_eq!(resource.surfaces[0].kind, SurfaceKind::Browser);

    // The surface change: browser -> api + cli + mcp.
    let mut updated = resource.clone();
    updated.surfaces = vec![
        AccessSurface::new(SurfaceKind::Api, resource_id.clone()),
        AccessSurface::new(SurfaceKind::Cli, resource_id.clone()),
        AccessSurface::new(SurfaceKind::Mcp, resource_id.clone()),
    ];
    let updated = test_ok(store.update_resource(updated));
    assert_eq!(
        updated.id, resource_id,
        "identity is stable across surfaces"
    );
    assert_eq!(updated.version, 2, "a surface change is a durable mutation");
    assert_eq!(updated.surfaces.len(), 3);

    let json = test_ok(serde_json::to_string(&store.snapshot()));
    drop(store);
    let snapshot: WorldSnapshot = test_ok(serde_json::from_str(&json));
    let reloaded = test_ok(FakeWorldStore::restore(snapshot));
    let after = test_ok(reloaded.resource(&resource_id));
    let Some(after) = after else {
        panic!("resource missing after reload");
    };
    assert_eq!(after.id, resource_id);
    assert_eq!(after.version, 2);
    assert_eq!(after.surfaces, updated.surfaces);
    for surface in &after.surfaces {
        assert_eq!(surface.resource_id, resource_id);
    }
}

/// Procedures round-trip canonically and their version lineage (1.0 first,
/// each improvement referencing its predecessor) is preserved and enforced.
#[test]
fn procedure_roundtrip_and_version_lineage() {
    let created = test_ts("2026-09-21T13:45:00Z");
    let procedure_id = flauz_world::ProcedureId::generate();
    let mut store = FakeWorldStore::new();

    let first = test_procedure_version(None, 1, 0);
    let created_procedure = test_ok(store.create_procedure(test_ok(Procedure::new(
        procedure_id.clone(),
        "Ticket reconciliation",
        first,
        test_ok(ActorRef::user("alice")),
        created,
    ))));
    assert_eq!(created_procedure.version, 1);
    assert_eq!(created_procedure.versions.len(), 1);
    assert_eq!(
        created_procedure.current().map(|v| v.number()),
        Some(SemanticVersion::FIRST)
    );

    // A saved improvement creates a new ProcedureVersion referencing its
    // predecessor, and is a durable mutation of the procedure entity.
    let improved = test_procedure_version(Some(SemanticVersion::new(1, 0)), 1, 1);
    let mut updated = created_procedure.clone();
    test_ok(updated.add_version(improved.clone()));
    let updated = test_ok(store.update_procedure(updated));
    assert_eq!(updated.version, 2);
    assert_eq!(updated.versions.len(), 2);
    assert_eq!(
        updated.versions[1].predecessor,
        Some(SemanticVersion::new(1, 0))
    );
    assert_eq!(
        updated.current().map(|v| v.number()),
        Some(SemanticVersion::new(1, 1))
    );

    // Lineage rules are enforced: no orphan versions, no regressions.
    let mut broken = updated.clone();
    assert!(
        broken
            .add_version(test_procedure_version(None, 1, 2))
            .is_err()
    );
    let mut regressing = updated.clone();
    assert!(
        regressing
            .add_version(test_procedure_version(
                Some(SemanticVersion::new(1, 1)),
                1,
                0
            ))
            .is_err()
    );

    // Canonical round-trip of the procedure document itself.
    let json = test_ok(serde_json::to_string(&updated));
    let reloaded_procedure: Procedure = test_ok(serde_json::from_str(&json));
    assert_eq!(reloaded_procedure, updated, "procedure round-trip equality");

    // And through the whole-world snapshot.
    let snapshot_json = test_ok(serde_json::to_string(&store.snapshot()));
    drop(store);
    let snapshot: WorldSnapshot = test_ok(serde_json::from_str(&snapshot_json));
    let reloaded = test_ok(FakeWorldStore::restore(snapshot));
    let after = test_ok(reloaded.procedure(&procedure_id));
    let Some(after) = after else {
        panic!("procedure missing after reload");
    };
    assert_eq!(after, updated);
    assert_eq!(
        after.versions[1].predecessor,
        Some(SemanticVersion::new(1, 0))
    );
}

/// The whole world survives serialize → drop → reload with equality, and
/// the identity, evidence and lease semantics hold on the reloaded state.
#[test]
fn state_survives_serialize_drop_reload_roundtrip() {
    let created = test_ts("2026-09-21T13:45:00Z");
    let ws_id = WorkspaceId::generate();
    let task_id = TaskId::generate();
    let resource_id = ResourceId::generate();
    let claim_id = ClaimId::generate();
    let expired_lease_id = LeaseId::generate();
    let active_lease_id = LeaseId::generate();
    let mut store = FakeWorldStore::new();

    test_ok(store.create_workspace(test_ok(Workspace::new(
        ws_id.clone(),
        "Research",
        test_ok(ActorRef::user("alice")),
        created,
    ))));
    test_ok(store.create_session(test_ok(Session::new(
        SessionId::generate(),
        ws_id.clone(),
        Some(task_id.clone()),
        test_ok(ActorRef::user("alice")),
        created,
    ))));
    let task = test_ok(store.create_task(test_ok(Task::new(
        task_id.clone(),
        ws_id,
        "Reconcile the ticket counts",
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
    ))));
    test_ok(store.create_artifact(test_ok(Artifact::new(
        ArtifactId::generate(),
        task_id.clone(),
        "Report",
        ArtifactContent::Text {
            text: "Dashboard and CRM match".to_owned(),
        },
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
    ))));
    let resource = test_ok(store.create_resource(test_ok(Resource::new(
        resource_id.clone(),
        "Acme support portal",
        vec![AccessSurface::new(
            SurfaceKind::Browser,
            resource_id.clone(),
        )],
        test_ok(ActorRef::user("alice")),
        created,
    ))));
    test_ok(store.put_resource_state(ResourceState {
        v: flauz_world::ContractVersion,
        resource_id: resource_id.clone(),
        resource_version: resource.version,
        permissions: vec!["dashboard:read".to_owned()],
        last_verified_observation: None,
        authoritative_surface: Some(SurfaceKind::Api),
        active_lease_ids: vec![active_lease_id.clone()],
        pending_mutation_ids: Vec::new(),
        related_artifact_ids: Vec::new(),
        captured_by: test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        captured_at: created,
    }));
    let switch = test_ok(store.append_event(
        StreamRef::task(&task_id),
        test_ok(Event::new(
            event_types::TASK_MODEL_CHANGED,
            created,
            test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
            EntityRef::task(&task_id),
            test_payload(&[(
                "to_model",
                CanonicalValue::from("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE"),
            )]),
        )),
    ));
    test_ok(store.record_observation(test_ok(Observation::new(
        ObservationId::generate(),
        resource_id.clone(),
        SurfaceKind::Browser,
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
        "The dashboard shows 3 open tickets",
    ))));
    test_ok(store.record_claim(test_ok(Claim::new(
        claim_id.clone(),
        resource_id.clone(),
        Some(SurfaceKind::Api),
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
        "The ticket count matches the CRM export",
    ))));
    let claim = test_ok(store.claim(&claim_id));
    let Some(claim) = claim else {
        panic!("claim missing");
    };
    let verification = test_ok(store.verify_claim(
        claim,
        test_ok(ActorRef::user("alice")),
        created,
        StreamRef::task(&task_id),
        Payload::empty(),
    ));
    // One lease that will have expired, one that is still held.
    test_ok(store.grant_lease(test_ok(ResourceLease::new(
        expired_lease_id.clone(),
        resource_id.clone(),
        AccessMode::Read,
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
        test_ts("2026-09-21T14:00:00Z"),
        ConflictPolicy::Queue,
    ))));
    test_ok(store.grant_lease(test_ok(ResourceLease::new(
        active_lease_id.clone(),
        resource_id.clone(),
        AccessMode::Write,
        test_ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
        created,
        test_ts("2099-01-01T00:00:00Z"),
        ConflictPolicy::Queue,
    ))));
    test_ok(store.create_procedure(test_ok(Procedure::new(
        flauz_world::ProcedureId::generate(),
        "Ticket reconciliation",
        test_procedure_version(None, 1, 0),
        test_ok(ActorRef::user("alice")),
        created,
    ))));

    // Serialize everything, drop the in-memory state, reload.
    let snapshot = store.snapshot();
    let json = test_ok(serde_json::to_string(&snapshot));
    drop(store);
    let parsed: WorldSnapshot = test_ok(serde_json::from_str(&json));
    assert_eq!(parsed, snapshot, "snapshot round-trip with equality");
    let reloaded = test_ok(FakeWorldStore::restore(parsed));
    let snapshot_two = reloaded.snapshot();
    assert_eq!(
        snapshot_two, snapshot,
        "reloaded store reproduces the same world state"
    );

    // The same logical task identity.
    let reloaded_task = test_ok(reloaded.task(&task_id));
    let Some(reloaded_task) = reloaded_task else {
        panic!("task missing after reload");
    };
    assert_eq!(reloaded_task, task);
    // The task stream carries the model switch and the evidence.verified
    // event that verify_claim appended to the same stream.
    let events = test_ok(reloaded.events(&StreamRef::task(&task_id), 0, 10));
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_id, switch.event_id);
    assert_eq!(
        events[1].event_type.as_str(),
        flauz_world::event::event_types::EVIDENCE_VERIFIED
    );

    // Evidence is still verified.
    let reloaded_evidence = test_ok(reloaded.evidence(&verification.evidence.id));
    let Some(reloaded_evidence) = reloaded_evidence else {
        panic!("evidence missing after reload");
    };
    assert_eq!(reloaded_evidence, verification.evidence);
    let reloaded_claim = test_ok(reloaded.claim(&claim_id));
    let Some(reloaded_claim) = reloaded_claim else {
        panic!("claim missing after reload");
    };
    assert_eq!(reloaded_claim.verification, VerificationStatus::Verified);

    // The expired lease is not held; the active one still is.
    let now = test_ts("2026-09-21T14:30:00Z");
    assert!(test_ok(reloaded.lease(&expired_lease_id, &now)).is_none());
    assert!(test_ok(reloaded.lease(&active_lease_id, &now)).is_some());
}

fn test_procedure_version(
    predecessor: Option<SemanticVersion>,
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
        created_at: test_ts("2026-09-21T13:45:00Z"),
    }
}
