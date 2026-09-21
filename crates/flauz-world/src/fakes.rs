//! In-memory fakes: the deterministic conformance surface (kernel §7).
//!
//! [`FakeWorldStore`] implements [`WorldStore`] with plain in-memory maps,
//! no I/O, and no wall-clock reads: every timestamp is caller-supplied and
//! the only generated values are canonical IDs (the crate's single entropy
//! source, produced by the private `ulid` module). Use it for tests, the F2
//! integration gate, and as the reference semantics for real stores.

use std::collections::BTreeMap;

use crate::WorldError;
use crate::event::{Event, EventEnvelope, event_types};
use crate::evidence::{Claim, Evidence, Observation, VerificationStatus};
use crate::ids::{
    ArtifactId, ClaimId, EntityKind, EventId, EvidenceId, LeaseId, ObservationId, ProcedureId,
    ResourceId, SessionId, TaskId, WorkspaceId,
};
use crate::lease::ResourceLease;
use crate::procedure::Procedure;
use crate::refs::{ActorRef, EntityRef, StreamRef};
use crate::resource::{Resource, ResourceState};
use crate::store::{Verification, WorldSnapshot, WorldStore, WorldStoreError};
use crate::time::Timestamp;
use crate::value::Payload;
use crate::world::{Artifact, Session, Task, Workspace};

/// The in-memory fake world store. Deterministic: identical operation
/// sequences produce identical logical state (generated IDs aside).
#[derive(Debug, Default, Clone)]
pub struct FakeWorldStore {
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    sessions: BTreeMap<SessionId, Session>,
    tasks: BTreeMap<TaskId, Task>,
    artifacts: BTreeMap<ArtifactId, Artifact>,
    resources: BTreeMap<ResourceId, Resource>,
    resource_states: BTreeMap<ResourceId, ResourceState>,
    events: BTreeMap<StreamRef, Vec<EventEnvelope>>,
    next_seq: BTreeMap<StreamRef, u64>,
    event_streams: BTreeMap<EventId, StreamRef>,
    observations: BTreeMap<ObservationId, Observation>,
    claims: BTreeMap<ClaimId, Claim>,
    evidence: BTreeMap<EvidenceId, Evidence>,
    leases: BTreeMap<LeaseId, ResourceLease>,
    procedures: BTreeMap<ProcedureId, Procedure>,
}

/// Internal helpers shared by the fake's create/update paths.
mod entity {
    use super::*;

    /// The operations every durable entity supports in the fake.
    pub(super) trait WorldEntity {
        fn entity_id(&self) -> &str;
        fn entity_version(&self) -> u64;
        fn set_entity_version(&mut self, version: u64);
        fn validate_entity(&self) -> Result<(), WorldError>;
    }

    impl WorldEntity for Workspace {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for Session {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for Task {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for Artifact {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for Resource {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for Observation {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for Claim {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for Evidence {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for ResourceLease {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    impl WorldEntity for Procedure {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), WorldError> {
            self.validate()
        }
    }

    /// Creates an entity: must be valid, at version 1, and not present.
    pub(super) fn create<E: WorldEntity + Clone, K: Ord + Clone>(
        map: &mut BTreeMap<K, E>,
        key: K,
        entity: E,
    ) -> Result<E, WorldStoreError> {
        entity.validate_entity()?;
        if map.contains_key(&key) {
            return Err(WorldStoreError::Duplicate {
                id: entity.entity_id().to_owned(),
            });
        }
        if entity.entity_version() != 1 {
            return Err(WorldStoreError::Invalid {
                reason: format!("entity {} must be created at version 1", entity.entity_id()),
            });
        }
        map.insert(key, entity.clone());
        Ok(entity)
    }

    /// Updates an entity: the passed version is the expected version; on
    /// mismatch the store returns a version conflict (kernel §3).
    pub(super) fn update<E: WorldEntity + Clone, K: Ord + Clone>(
        map: &mut BTreeMap<K, E>,
        key: &K,
        mut entity: E,
    ) -> Result<E, WorldStoreError> {
        entity.validate_entity()?;
        let Some(stored) = map.get(key) else {
            return Err(WorldStoreError::NotFound {
                id: entity.entity_id().to_owned(),
            });
        };
        if stored.entity_version() != entity.entity_version() {
            return Err(WorldStoreError::VersionConflict {
                id: entity.entity_id().to_owned(),
                expected_version: entity.entity_version(),
                actual_version: stored.entity_version(),
            });
        }
        entity.set_entity_version(stored.entity_version() + 1);
        map.insert(key.clone(), entity.clone());
        Ok(entity)
    }
}

impl FakeWorldStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn append_envelope(
        &mut self,
        stream: &StreamRef,
        event_id: EventId,
        event: Event,
    ) -> Result<EventEnvelope, WorldStoreError> {
        let seq = {
            let counter = self.next_seq.entry(stream.clone()).or_insert(0);
            *counter += 1;
            *counter
        };
        let envelope = EventEnvelope::new(event_id, seq, stream.clone(), event);
        envelope.validate()?;
        self.events
            .entry(stream.clone())
            .or_default()
            .push(envelope.clone());
        self.event_streams
            .insert(envelope.event_id.clone(), stream.clone());
        Ok(envelope)
    }
}

impl WorldStore for FakeWorldStore {
    fn create_workspace(&mut self, workspace: Workspace) -> Result<Workspace, WorldStoreError> {
        entity::create(&mut self.workspaces, workspace.id.clone(), workspace)
    }

    fn workspace(&self, id: &WorkspaceId) -> Result<Option<Workspace>, WorldStoreError> {
        Ok(self.workspaces.get(id).cloned())
    }

    fn update_workspace(&mut self, workspace: Workspace) -> Result<Workspace, WorldStoreError> {
        entity::update(&mut self.workspaces, &workspace.id.clone(), workspace)
    }

    fn create_session(&mut self, session: Session) -> Result<Session, WorldStoreError> {
        entity::create(&mut self.sessions, session.id.clone(), session)
    }

    fn session(&self, id: &SessionId) -> Result<Option<Session>, WorldStoreError> {
        Ok(self.sessions.get(id).cloned())
    }

    fn update_session(&mut self, session: Session) -> Result<Session, WorldStoreError> {
        entity::update(&mut self.sessions, &session.id.clone(), session)
    }

    fn create_task(&mut self, task: Task) -> Result<Task, WorldStoreError> {
        entity::create(&mut self.tasks, task.id.clone(), task)
    }

    fn task(&self, id: &TaskId) -> Result<Option<Task>, WorldStoreError> {
        Ok(self.tasks.get(id).cloned())
    }

    fn update_task(&mut self, task: Task) -> Result<Task, WorldStoreError> {
        entity::update(&mut self.tasks, &task.id.clone(), task)
    }

    fn create_artifact(&mut self, artifact: Artifact) -> Result<Artifact, WorldStoreError> {
        entity::create(&mut self.artifacts, artifact.id.clone(), artifact)
    }

    fn artifact(&self, id: &ArtifactId) -> Result<Option<Artifact>, WorldStoreError> {
        Ok(self.artifacts.get(id).cloned())
    }

    fn update_artifact(&mut self, artifact: Artifact) -> Result<Artifact, WorldStoreError> {
        entity::update(&mut self.artifacts, &artifact.id.clone(), artifact)
    }

    fn create_resource(&mut self, resource: Resource) -> Result<Resource, WorldStoreError> {
        entity::create(&mut self.resources, resource.id.clone(), resource)
    }

    fn resource(&self, id: &ResourceId) -> Result<Option<Resource>, WorldStoreError> {
        Ok(self.resources.get(id).cloned())
    }

    fn update_resource(&mut self, resource: Resource) -> Result<Resource, WorldStoreError> {
        entity::update(&mut self.resources, &resource.id.clone(), resource)
    }

    fn put_resource_state(
        &mut self,
        state: ResourceState,
    ) -> Result<ResourceState, WorldStoreError> {
        state.validate()?;
        if !self.resources.contains_key(&state.resource_id) {
            return Err(WorldStoreError::NotFound {
                id: state.resource_id.as_str().to_owned(),
            });
        }
        self.resource_states
            .insert(state.resource_id.clone(), state.clone());
        Ok(state)
    }

    fn resource_state(
        &self,
        resource: &ResourceId,
    ) -> Result<Option<ResourceState>, WorldStoreError> {
        Ok(self.resource_states.get(resource).cloned())
    }

    fn append_event(
        &mut self,
        stream: StreamRef,
        event: Event,
    ) -> Result<EventEnvelope, WorldStoreError> {
        event.validate()?;
        self.append_envelope(&stream, EventId::generate(), event)
    }

    fn event(&self, id: &EventId) -> Result<Option<EventEnvelope>, WorldStoreError> {
        match self.event_streams.get(id) {
            Some(stream) => Ok(self.events.get(stream).and_then(|envelopes| {
                envelopes
                    .iter()
                    .find(|envelope| &envelope.event_id == id)
                    .cloned()
            })),
            None => Ok(None),
        }
    }

    fn events(
        &self,
        stream: &StreamRef,
        after_seq: u64,
        limit: usize,
    ) -> Result<Vec<EventEnvelope>, WorldStoreError> {
        Ok(self
            .events
            .get(stream)
            .map(|envelopes| {
                envelopes
                    .iter()
                    .filter(|envelope| envelope.seq > after_seq)
                    .take(limit)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    fn record_observation(
        &mut self,
        observation: Observation,
    ) -> Result<Observation, WorldStoreError> {
        entity::create(&mut self.observations, observation.id.clone(), observation)
    }

    fn observation(&self, id: &ObservationId) -> Result<Option<Observation>, WorldStoreError> {
        Ok(self.observations.get(id).cloned())
    }

    fn update_observation(
        &mut self,
        observation: Observation,
    ) -> Result<Observation, WorldStoreError> {
        entity::update(&mut self.observations, &observation.id.clone(), observation)
    }

    fn record_claim(&mut self, claim: Claim) -> Result<Claim, WorldStoreError> {
        entity::create(&mut self.claims, claim.id.clone(), claim)
    }

    fn claim(&self, id: &ClaimId) -> Result<Option<Claim>, WorldStoreError> {
        Ok(self.claims.get(id).cloned())
    }

    fn update_claim(&mut self, claim: Claim) -> Result<Claim, WorldStoreError> {
        entity::update(&mut self.claims, &claim.id.clone(), claim)
    }

    fn verify_claim(
        &mut self,
        claim: Claim,
        verifier: ActorRef,
        verified_at: Timestamp,
        stream: StreamRef,
        payload: Payload,
    ) -> Result<Verification, WorldStoreError> {
        claim.validate()?;
        if claim.verification == VerificationStatus::Verified {
            return Err(WorldStoreError::Invalid {
                reason: format!("claim {} is already verified", claim.id),
            });
        }
        let Some(stored) = self.claims.get(&claim.id) else {
            return Err(WorldStoreError::NotFound {
                id: claim.id.as_str().to_owned(),
            });
        };
        if stored.version != claim.version {
            return Err(WorldStoreError::VersionConflict {
                id: claim.id.as_str().to_owned(),
                expected_version: claim.version,
                actual_version: stored.version,
            });
        }

        // The evidence record and its verification event reference each
        // other, so both IDs are allocated up front and the store assembles
        // them atomically.
        let evidence_id = EvidenceId::generate();
        let event_id = EventId::generate();
        let event = Event::new(
            event_types::EVIDENCE_VERIFIED,
            verified_at,
            verifier.clone(),
            EntityRef {
                entity_kind: EntityKind::Evidence,
                id: evidence_id.as_str().to_owned(),
            },
            payload,
        )?;
        let envelope = self.append_envelope(&stream, event_id, event)?;
        let evidence = Evidence::new(
            evidence_id,
            claim.id.clone(),
            verifier,
            envelope.event_id.clone(),
            verified_at,
        )?;
        entity::create(&mut self.evidence, evidence.id.clone(), evidence.clone())?;

        let mut updated_claim = claim;
        updated_claim.verification = VerificationStatus::Verified;
        updated_claim.verifying_evidence_id = Some(evidence.id.clone());
        let claim_key = updated_claim.id.clone();
        let updated_claim = entity::update(&mut self.claims, &claim_key, updated_claim)?;

        Ok(Verification {
            claim: updated_claim,
            evidence,
            event: envelope,
        })
    }

    fn evidence(&self, id: &EvidenceId) -> Result<Option<Evidence>, WorldStoreError> {
        Ok(self.evidence.get(id).cloned())
    }

    fn grant_lease(&mut self, lease: ResourceLease) -> Result<ResourceLease, WorldStoreError> {
        entity::create(&mut self.leases, lease.id.clone(), lease)
    }

    fn lease(
        &self,
        id: &LeaseId,
        now: &Timestamp,
    ) -> Result<Option<ResourceLease>, WorldStoreError> {
        match self.leases.get(id) {
            Some(lease) if lease.is_held_at(now) => Ok(Some(lease.clone())),
            _ => Ok(None),
        }
    }

    fn lease_record(&self, id: &LeaseId) -> Result<Option<ResourceLease>, WorldStoreError> {
        Ok(self.leases.get(id).cloned())
    }

    fn active_leases(
        &self,
        resource: &ResourceId,
        now: &Timestamp,
    ) -> Result<Vec<ResourceLease>, WorldStoreError> {
        Ok(self
            .leases
            .values()
            .filter(|lease| &lease.resource_id == resource && lease.is_held_at(now))
            .cloned()
            .collect())
    }

    fn release_lease(
        &mut self,
        id: &LeaseId,
        at: Timestamp,
    ) -> Result<ResourceLease, WorldStoreError> {
        let Some(stored) = self.leases.get(id) else {
            return Err(WorldStoreError::NotFound {
                id: id.as_str().to_owned(),
            });
        };
        if !stored.is_held_at(&at) {
            return Err(WorldStoreError::Invalid {
                reason: format!("lease {id} is not held at {at}"),
            });
        }
        let mut released = stored.clone();
        released.released_at = Some(at);
        entity::update(&mut self.leases, id, released)
    }

    fn create_procedure(&mut self, procedure: Procedure) -> Result<Procedure, WorldStoreError> {
        entity::create(&mut self.procedures, procedure.id.clone(), procedure)
    }

    fn procedure(&self, id: &ProcedureId) -> Result<Option<Procedure>, WorldStoreError> {
        Ok(self.procedures.get(id).cloned())
    }

    fn update_procedure(&mut self, procedure: Procedure) -> Result<Procedure, WorldStoreError> {
        entity::update(&mut self.procedures, &procedure.id.clone(), procedure)
    }

    fn snapshot(&self) -> WorldSnapshot {
        let mut events: Vec<EventEnvelope> = self
            .events
            .values()
            .flat_map(|envelopes| envelopes.iter().cloned())
            .collect();
        events.sort_by(|left, right| (&left.stream, left.seq).cmp(&(&right.stream, right.seq)));
        WorldSnapshot {
            v: crate::ContractVersion,
            workspaces: self.workspaces.clone(),
            sessions: self.sessions.clone(),
            tasks: self.tasks.clone(),
            artifacts: self.artifacts.clone(),
            resources: self.resources.clone(),
            resource_states: self.resource_states.clone(),
            events,
            observations: self.observations.clone(),
            claims: self.claims.clone(),
            evidence: self.evidence.clone(),
            leases: self.leases.clone(),
            procedures: self.procedures.clone(),
        }
    }

    fn restore(snapshot: WorldSnapshot) -> Result<Self, WorldStoreError> {
        snapshot.validate()?;

        let mut events: BTreeMap<StreamRef, Vec<EventEnvelope>> = BTreeMap::new();
        let mut next_seq: BTreeMap<StreamRef, u64> = BTreeMap::new();
        let mut event_streams: BTreeMap<EventId, StreamRef> = BTreeMap::new();
        let mut sorted = snapshot.events;
        sorted.sort_by(|left, right| (&left.stream, left.seq).cmp(&(&right.stream, right.seq)));
        for envelope in sorted {
            let stream = envelope.stream.clone();
            next_seq.insert(stream.clone(), envelope.seq);
            event_streams.insert(envelope.event_id.clone(), stream.clone());
            events.entry(stream).or_default().push(envelope);
        }

        Ok(Self {
            workspaces: snapshot.workspaces,
            sessions: snapshot.sessions,
            tasks: snapshot.tasks,
            artifacts: snapshot.artifacts,
            resources: snapshot.resources,
            resource_states: snapshot.resource_states,
            events,
            next_seq,
            event_streams,
            observations: snapshot.observations,
            claims: snapshot.claims,
            evidence: snapshot.evidence,
            leases: snapshot.leases,
            procedures: snapshot.procedures,
        })
    }
}
