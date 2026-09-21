//! The world-state store contract, the store error surface, and the
//! canonical world snapshot used for serialize → drop → reload round-trips.
//!
//! Mutations follow the kernel §3 version rules: entities are created at
//! version 1, every durable mutation increments by exactly 1, and updates
//! carry the caller's expected version — a mismatch is a
//! [`WorldStoreError::VersionConflict`]; silent overwrite is forbidden.
//! Events are append-only with strictly increasing per-stream `seq`.
//!
//! Leases are read with a caller-supplied "now": expiry is enforced on read
//! and an expired lease is not held (kernel §7). Evidence enters the store
//! only through [`WorldStore::verify_claim`], which atomically produces the
//! evidence record, its `evidence.verified` event, and the claim update —
//! a Claim can never become Evidence any other way.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::event::{Event, EventEnvelope};
use crate::evidence::{Claim, Evidence, Observation};
use crate::ids::{
    ArtifactId, ClaimId, EventId, EvidenceId, LeaseId, ObservationId, ProcedureId, ResourceId,
    SessionId, TaskId, WorkspaceId,
};
use crate::lease::ResourceLease;
use crate::procedure::Procedure;
use crate::refs::{ActorRef, StreamRef};
use crate::resource::{Resource, ResourceState};
use crate::time::Timestamp;
use crate::value::Payload;
use crate::world::{Artifact, Session, Task, Workspace};
use crate::{ContractVersion, WorldError};

/// Errors returned by world-state store operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldStoreError {
    /// An entity with the same canonical ID already exists.
    Duplicate {
        /// The conflicting canonical ID.
        id: String,
    },
    /// No entity exists with the given canonical ID.
    NotFound {
        /// The missing canonical ID.
        id: String,
    },
    /// The caller's expected version did not match the stored version
    /// (kernel §3: optimistic concurrency; silent overwrite is forbidden).
    VersionConflict {
        /// The conflicting canonical ID.
        id: String,
        /// The version the caller expected.
        expected_version: u64,
        /// The version actually stored.
        actual_version: u64,
    },
    /// The operation violates a canonical rule (bounds, ordering, state).
    Invalid {
        /// The reason the operation was rejected.
        reason: String,
    },
}

impl fmt::Display for WorldStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate { id } => write!(formatter, "entity {id} already exists"),
            Self::NotFound { id } => write!(formatter, "entity {id} does not exist"),
            Self::VersionConflict {
                id,
                expected_version,
                actual_version,
            } => write!(
                formatter,
                "version conflict on {id}: expected {expected_version}, stored {actual_version}"
            ),
            Self::Invalid { reason } => write!(formatter, "invalid operation: {reason}"),
        }
    }
}

impl Error for WorldStoreError {}

impl From<WorldError> for WorldStoreError {
    fn from(error: WorldError) -> Self {
        Self::Invalid {
            reason: error.to_string(),
        }
    }
}

/// The result of a successful claim verification: the updated claim (now
/// verified and linked to the evidence), the evidence record, and the
/// `evidence.verified` event envelope that anchors it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    /// The updated claim (version bumped, verification status `verified`).
    pub claim: Claim,
    /// The evidence record.
    pub evidence: Evidence,
    /// The `evidence.verified` event envelope.
    pub event: EventEnvelope,
}

/// The canonical serialization of an entire world state. Used by fakes and
/// real stores alike for serialize → drop → reload round-trips: the state
/// survives with equality (kernel round-trip law).
///
/// Events are stored as a flat, deterministically ordered list (sorted by
/// stream, then `seq`); per-stream sequence counters are reconstructed on
/// reload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldSnapshot {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// All workspaces, keyed by canonical ID.
    pub workspaces: BTreeMap<WorkspaceId, Workspace>,
    /// All sessions, keyed by canonical ID.
    pub sessions: BTreeMap<SessionId, Session>,
    /// All tasks, keyed by canonical ID.
    pub tasks: BTreeMap<TaskId, Task>,
    /// All artifacts, keyed by canonical ID.
    pub artifacts: BTreeMap<ArtifactId, Artifact>,
    /// All resources, keyed by canonical ID.
    pub resources: BTreeMap<ResourceId, Resource>,
    /// Resource states, keyed by the resource they describe.
    pub resource_states: BTreeMap<ResourceId, ResourceState>,
    /// All event envelopes, in canonical (stream, seq) order.
    pub events: Vec<EventEnvelope>,
    /// All observations, keyed by canonical ID.
    pub observations: BTreeMap<ObservationId, Observation>,
    /// All claims, keyed by canonical ID.
    pub claims: BTreeMap<ClaimId, Claim>,
    /// All evidence, keyed by canonical ID.
    pub evidence: BTreeMap<EvidenceId, Evidence>,
    /// All leases, keyed by canonical ID.
    pub leases: BTreeMap<LeaseId, ResourceLease>,
    /// All procedures, keyed by canonical ID.
    pub procedures: BTreeMap<ProcedureId, Procedure>,
}

impl WorldSnapshot {
    /// The empty snapshot.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            v: ContractVersion,
            workspaces: BTreeMap::new(),
            sessions: BTreeMap::new(),
            tasks: BTreeMap::new(),
            artifacts: BTreeMap::new(),
            resources: BTreeMap::new(),
            resource_states: BTreeMap::new(),
            events: Vec::new(),
            observations: BTreeMap::new(),
            claims: BTreeMap::new(),
            evidence: BTreeMap::new(),
            leases: BTreeMap::new(),
            procedures: BTreeMap::new(),
        }
    }

    /// Validates the snapshot's structural invariants: map keys match the
    /// entity IDs they store, per-stream event sequences are strictly
    /// increasing, event IDs are unique, and every entity passes canonical
    /// validation.
    pub fn validate(&self) -> Result<(), WorldStoreError> {
        for (id, workspace) in &self.workspaces {
            check_key(id.as_str(), workspace.id.as_str(), workspace.validate())?;
        }
        for (id, session) in &self.sessions {
            check_key(id.as_str(), session.id.as_str(), session.validate())?;
        }
        for (id, task) in &self.tasks {
            check_key(id.as_str(), task.id.as_str(), task.validate())?;
        }
        for (id, artifact) in &self.artifacts {
            check_key(id.as_str(), artifact.id.as_str(), artifact.validate())?;
        }
        for (id, resource) in &self.resources {
            check_key(id.as_str(), resource.id.as_str(), resource.validate())?;
        }
        for (id, state) in &self.resource_states {
            if id != &state.resource_id {
                return Err(WorldStoreError::Invalid {
                    reason: format!(
                        "resource state map key {id} does not match its resource {}",
                        state.resource_id
                    ),
                });
            }
            state.validate().map_err(WorldStoreError::from)?;
        }
        for (id, observation) in &self.observations {
            check_key(id.as_str(), observation.id.as_str(), observation.validate())?;
        }
        for (id, claim) in &self.claims {
            check_key(id.as_str(), claim.id.as_str(), claim.validate())?;
        }
        for (id, evidence) in &self.evidence {
            check_key(id.as_str(), evidence.id.as_str(), evidence.validate())?;
        }
        for (id, lease) in &self.leases {
            check_key(id.as_str(), lease.id.as_str(), lease.validate())?;
        }
        for (id, procedure) in &self.procedures {
            check_key(id.as_str(), procedure.id.as_str(), procedure.validate())?;
        }

        let mut seen_event_ids = std::collections::BTreeSet::new();
        let mut last_seq_by_stream: BTreeMap<StreamRef, u64> = BTreeMap::new();
        for envelope in &self.events {
            envelope.validate().map_err(WorldStoreError::from)?;
            if !seen_event_ids.insert(envelope.event_id.clone()) {
                return Err(WorldStoreError::Invalid {
                    reason: format!("duplicate event id {}", envelope.event_id),
                });
            }
            match last_seq_by_stream.entry(envelope.stream.clone()) {
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    if envelope.seq <= *entry.get() {
                        return Err(WorldStoreError::Invalid {
                            reason: format!(
                                "event seq {} does not increase on stream {}",
                                envelope.seq, envelope.stream
                            ),
                        });
                    }
                    *entry.get_mut() = envelope.seq;
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(envelope.seq);
                }
            }
        }
        Ok(())
    }
}

fn check_key(
    key: &str,
    entity_id: &str,
    validation: Result<(), WorldError>,
) -> Result<(), WorldStoreError> {
    if key != entity_id {
        return Err(WorldStoreError::Invalid {
            reason: format!("map key {key} does not match entity id {entity_id}"),
        });
    }
    validation.map_err(WorldStoreError::from)
}

/// The world-state store contract.
///
/// All timestamps are supplied by callers; the store never reads the
/// wall clock. Reads are bounded: event queries take an explicit limit.
pub trait WorldStore {
    /// Stores a new workspace (created at version 1).
    fn create_workspace(&mut self, workspace: Workspace) -> Result<Workspace, WorldStoreError>;
    /// Looks up a workspace.
    fn workspace(&self, id: &WorkspaceId) -> Result<Option<Workspace>, WorldStoreError>;
    /// Updates a workspace; the passed `version` is the expected version,
    /// and the stored entity is returned with its version incremented.
    fn update_workspace(&mut self, workspace: Workspace) -> Result<Workspace, WorldStoreError>;

    /// Stores a new session (created at version 1).
    fn create_session(&mut self, session: Session) -> Result<Session, WorldStoreError>;
    /// Looks up a session.
    fn session(&self, id: &SessionId) -> Result<Option<Session>, WorldStoreError>;
    /// Updates a session (optimistic concurrency on the passed version).
    fn update_session(&mut self, session: Session) -> Result<Session, WorldStoreError>;

    /// Stores a new task (created at version 1).
    fn create_task(&mut self, task: Task) -> Result<Task, WorldStoreError>;
    /// Looks up a task.
    fn task(&self, id: &TaskId) -> Result<Option<Task>, WorldStoreError>;
    /// Updates a task (optimistic concurrency on the passed version).
    fn update_task(&mut self, task: Task) -> Result<Task, WorldStoreError>;

    /// Stores a new artifact (created at version 1).
    fn create_artifact(&mut self, artifact: Artifact) -> Result<Artifact, WorldStoreError>;
    /// Looks up an artifact.
    fn artifact(&self, id: &ArtifactId) -> Result<Option<Artifact>, WorldStoreError>;
    /// Updates an artifact (optimistic concurrency on the passed version).
    fn update_artifact(&mut self, artifact: Artifact) -> Result<Artifact, WorldStoreError>;

    /// Stores a new resource (created at version 1).
    fn create_resource(&mut self, resource: Resource) -> Result<Resource, WorldStoreError>;
    /// Looks up a resource.
    fn resource(&self, id: &ResourceId) -> Result<Option<Resource>, WorldStoreError>;
    /// Updates a resource; surface changes are durable mutations that
    /// preserve the resource identity (optimistic concurrency on the
    /// passed version).
    fn update_resource(&mut self, resource: Resource) -> Result<Resource, WorldStoreError>;

    /// Stores or replaces the state snapshot of a resource. The resource
    /// must exist.
    fn put_resource_state(
        &mut self,
        state: ResourceState,
    ) -> Result<ResourceState, WorldStoreError>;
    /// Looks up the state snapshot of a resource.
    fn resource_state(
        &self,
        resource: &ResourceId,
    ) -> Result<Option<ResourceState>, WorldStoreError>;

    /// Appends an event to a stream. The store assigns the unique event ID
    /// and the strictly increasing per-stream `seq` and returns the full
    /// envelope.
    fn append_event(
        &mut self,
        stream: StreamRef,
        event: Event,
    ) -> Result<EventEnvelope, WorldStoreError>;
    /// Looks up an event envelope by its canonical ID.
    fn event(&self, id: &EventId) -> Result<Option<EventEnvelope>, WorldStoreError>;
    /// Reads at most `limit` events from a stream with `seq` greater than
    /// `after_seq`, in `seq` order (bounded read).
    fn events(
        &self,
        stream: &StreamRef,
        after_seq: u64,
        limit: usize,
    ) -> Result<Vec<EventEnvelope>, WorldStoreError>;

    /// Records a new observation (created at version 1, verification
    /// status `observed`).
    fn record_observation(
        &mut self,
        observation: Observation,
    ) -> Result<Observation, WorldStoreError>;
    /// Looks up an observation.
    fn observation(&self, id: &ObservationId) -> Result<Option<Observation>, WorldStoreError>;
    /// Updates an observation (optimistic concurrency on the passed
    /// version).
    fn update_observation(
        &mut self,
        observation: Observation,
    ) -> Result<Observation, WorldStoreError>;

    /// Records a new claim (created at version 1, verification status
    /// `claimed`).
    fn record_claim(&mut self, claim: Claim) -> Result<Claim, WorldStoreError>;
    /// Looks up a claim.
    fn claim(&self, id: &ClaimId) -> Result<Option<Claim>, WorldStoreError>;
    /// Updates a claim (optimistic concurrency on the passed version).
    fn update_claim(&mut self, claim: Claim) -> Result<Claim, WorldStoreError>;

    /// Verifies a claim: atomically appends an `evidence.verified` event to
    /// `stream`, creates the evidence record referencing that event, and
    /// updates the claim to verification status `verified` with the
    /// evidence link (the passed claim's `version` is the expected
    /// version). This is the only way evidence enters the store.
    fn verify_claim(
        &mut self,
        claim: Claim,
        verifier: ActorRef,
        verified_at: Timestamp,
        stream: StreamRef,
        payload: Payload,
    ) -> Result<Verification, WorldStoreError>;

    /// Looks up evidence.
    fn evidence(&self, id: &EvidenceId) -> Result<Option<Evidence>, WorldStoreError>;

    /// Grants (stores) a new lease (created at version 1, not yet
    /// released). The lease must be bounded: `expires_at` strictly after
    /// `granted_at`.
    fn grant_lease(&mut self, lease: ResourceLease) -> Result<ResourceLease, WorldStoreError>;
    /// Looks up a lease **as held at `now`**: an expired or released lease
    /// is not held and returns `None` (kernel §7: expiry is enforced on
    /// read; no implicit renewal).
    fn lease(
        &self,
        id: &LeaseId,
        now: &Timestamp,
    ) -> Result<Option<ResourceLease>, WorldStoreError>;
    /// Looks up the raw lease record regardless of held-ness (for
    /// provenance).
    fn lease_record(&self, id: &LeaseId) -> Result<Option<ResourceLease>, WorldStoreError>;
    /// Lists the leases on a resource that are held at `now`, in canonical
    /// ID order.
    fn active_leases(
        &self,
        resource: &ResourceId,
        now: &Timestamp,
    ) -> Result<Vec<ResourceLease>, WorldStoreError>;
    /// Releases a held lease at `at` (the lease must be held at that
    /// timestamp). Returns the updated lease record with `released_at` set
    /// and its version incremented.
    fn release_lease(
        &mut self,
        id: &LeaseId,
        at: Timestamp,
    ) -> Result<ResourceLease, WorldStoreError>;

    /// Stores a new procedure (created at version 1, first semantic version
    /// `1.0`).
    fn create_procedure(&mut self, procedure: Procedure) -> Result<Procedure, WorldStoreError>;
    /// Looks up a procedure.
    fn procedure(&self, id: &ProcedureId) -> Result<Option<Procedure>, WorldStoreError>;
    /// Updates a procedure; recording an improved version is a durable
    /// mutation (optimistic concurrency on the passed version).
    fn update_procedure(&mut self, procedure: Procedure) -> Result<Procedure, WorldStoreError>;

    /// Captures the entire world state as a canonical snapshot.
    fn snapshot(&self) -> WorldSnapshot;
    /// Rebuilds a store from a snapshot (serialize → drop → reload). The
    /// snapshot must satisfy the canonical invariants (map keys match
    /// entity IDs, strictly increasing per-stream event sequences, unique
    /// event IDs).
    fn restore(snapshot: WorldSnapshot) -> Result<Self, WorldStoreError>
    where
        Self: Sized;
}
