//! Canonical world-state contracts for Flauz (F2 contract wave, work order
//! ARCH-001).
//!
//! This crate is the single owner of the platform's durable world-state
//! types: [`Workspace`], [`Session`], [`Task`], [`Artifact`], [`Resource`],
//! [`ResourceState`], [`Event`] + [`EventEnvelope`], [`Observation`],
//! [`Claim`], [`Evidence`], [`ResourceLease`], [`Procedure`] and
//! [`ProcedureVersion`]. Every cross-crate rule implemented here comes from
//! the frozen [F2 contract kernel]
//! (`docs/F2-CONTRACT-KERNEL.md`): canonical IDs, entity versions, canonical
//! JSON serialization, the v1 event envelope, task identity semantics, the
//! claim/evidence distinction, and lease semantics.
//!
//! # Registered event types (kernel §5)
//!
//! This crate registers the following `event_type` vocabulary. Other crates
//! register their own; the grammar `<entity>.<verb_past>` is enforced by
//! [`EventTypeName`] for all of them.
//!
//! | Constant | `event_type` |
//! |---|---|
//! | [`event_types::WORKSPACE_CREATED`] | `workspace.created` |
//! | [`event_types::SESSION_CREATED`] | `session.created` |
//! | [`event_types::TASK_CREATED`] | `task.created` |
//! | [`event_types::TASK_MODEL_CHANGED`] | `task.model_changed` |
//! | [`event_types::TASK_CONTEXT_RESET`] | `task.context_reset` |
//! | [`event_types::TASK_ENVIRONMENT_CHANGED`] | `task.environment_changed` |
//! | [`event_types::TASK_AGENT_HANDOFF`] | `task.agent_handoff` |
//! | [`event_types::TASK_SESSION_RESTARTED`] | `task.session_restarted` |
//! | [`event_types::ARTIFACT_PRODUCED`] | `artifact.produced` |
//! | [`event_types::RESOURCE_CREATED`] | `resource.created` |
//! | [`event_types::RESOURCE_SURFACES_CHANGED`] | `resource.surfaces_changed` |
//! | [`event_types::RESOURCE_OBSERVED`] | `resource.observed` |
//! | [`event_types::CLAIM_MADE`] | `claim.made` |
//! | [`event_types::EVIDENCE_VERIFIED`] | `evidence.verified` |
//! | [`event_types::LEASE_GRANTED`] | `lease.granted` |
//! | [`event_types::LEASE_RELEASED`] | `lease.released` |
//! | [`event_types::PROCEDURE_CREATED`] | `procedure.created` |
//! | [`event_types::PROCEDURE_VERSION_ADDED`] | `procedure.version_added` |
//!
//! Per the kernel, `task.model_changed`, `task.context_reset`,
//! `task.environment_changed`, `task.agent_handoff` and
//! `task.session_restarted` are events on the task stream: they never change
//! the [`Task`] ID and never fork the logical task.
//!
//! # Determinism (kernel §7)
//!
//! Contract code in this crate never reads wall-clock time or randomness
//! directly: every timestamp is passed in by callers. The only entropy
//! source is ID generation, confined to a private `ulid` module (no external
//! ULID crate). The in-memory [`fakes::FakeWorldStore`] is fully
//! deterministic apart from generated IDs.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields, contains no floats (and
//! no null payload values), emits timestamps as RFC 3339 UTC
//! `YYYY-MM-DDTHH:MM:SSZ`, and keeps byte payloads out of inline state via
//! bounded references. Durations do not occur in the v1 contracts; when they
//! are introduced they must be integer milliseconds.

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod event;
pub mod evidence;
pub mod fakes;
pub mod ids;
pub mod lease;
pub mod procedure;
pub mod refs;
pub mod resource;
pub mod store;
pub mod time;
pub mod value;
pub mod world;

mod ulid;

pub use crate::event::{Event, EventEnvelope, EventEnvelopeKind, EventTypeName, event_types};
pub use crate::evidence::{Claim, Evidence, Observation, VerificationStatus};
pub use crate::ids::{
    ArtifactId, ClaimId, EntityKind, EventId, EvidenceId, IdError, LeaseId, ObservationId,
    ProcedureId, ResourceId, SessionId, TaskId, WorkspaceId,
};
pub use crate::lease::{AccessMode, ConflictPolicy, ResourceLease};
pub use crate::procedure::{
    CapabilityKey, Procedure, ProcedureInput, ProcedureStep, ProcedureVersion, ResourceBinding,
    SemanticVersion,
};
pub use crate::refs::{ActorKind, ActorRef, EntityRef, StreamKind, StreamRef};
pub use crate::resource::{AccessSurface, Resource, ResourceState, SurfaceKind};
pub use crate::store::{Verification, WorldSnapshot, WorldStore, WorldStoreError};
pub use crate::time::Timestamp;
pub use crate::value::{CanonicalValue, Payload};
pub use crate::world::{Artifact, ArtifactContent, Session, Task, Workspace};

/// Maximum length of short human-readable names (workspaces, resources,
/// procedures, artifact titles, event types).
pub const MAX_NAME_BYTES: usize = 256;

/// Maximum length of statements, objectives and other bounded prose fields.
pub const MAX_STATEMENT_BYTES: usize = 8 * 1024;

/// Maximum length of inline artifact text content.
pub const MAX_TEXT_BYTES: usize = 64 * 1024;

/// Maximum length of bounded reference strings (artifact content references,
/// correlation identifiers).
pub const MAX_REFERENCE_BYTES: usize = 2048;

/// Maximum length of an actor identifier string.
pub const MAX_ACTOR_ID_BYTES: usize = 256;

/// Maximum number of fields in a canonical payload object.
pub const MAX_PAYLOAD_FIELDS: usize = 64;

/// Maximum length of a string value (or key) inside a canonical payload.
pub const MAX_PAYLOAD_STRING_BYTES: usize = 8 * 1024;

/// Maximum number of items in a canonical payload array or object.
pub const MAX_PAYLOAD_ITEMS: usize = 64;

/// Maximum number of access surfaces on one resource.
pub const MAX_SURFACES_PER_RESOURCE: usize = 16;

/// Maximum number of entries in bounded reference lists (supporting
/// artifacts, resource-state lists, permissions, procedure lists).
pub const MAX_RELATED_REFS: usize = 64;

/// Maximum number of versions recorded in one procedure.
pub const MAX_PROCEDURE_VERSIONS: usize = 64;

/// Contract-level validation error for values that fail canonical rules.
///
/// Store operations surface these through
/// [`WorldStoreError::Invalid`](store::WorldStoreError).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldError {
    /// A canonical identifier is malformed (see [`IdError`] for the frozen
    /// failure reasons).
    Id(IdError),
    /// A value violates a canonical rule (bounds, ordering, grammar).
    Invalid(String),
}

impl WorldError {
    /// Builds an [`WorldError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(error) => error.fmt(formatter),
            Self::Invalid(reason) => write!(formatter, "invalid world state: {reason}"),
        }
    }
}

impl Error for WorldError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Id(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<IdError> for WorldError {
    fn from(error: IdError) -> Self {
        Self::Id(error)
    }
}

/// The F2 contract schema version marker. Serializes as `"v": 1` and rejects
/// any other value, so a document written by a different schema version
/// fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContractVersion;

impl serde::Serialize for ContractVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for ContractVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = u32::deserialize(deserializer)?;
        if version == 1 {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported contract schema version {version}; this build reads v1"
            )))
        }
    }
}

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), WorldError> {
    if value.is_empty() {
        Err(WorldError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), WorldError> {
    if value.len() > max_bytes {
        Err(WorldError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_list_bound<T>(
    field: &'static str,
    list: &[T],
    max_items: usize,
) -> Result<(), WorldError> {
    if list.len() > max_items {
        Err(WorldError::invalid(format!(
            "{field} exceeds {max_items} entries"
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn contract_version_serializes_as_one_and_rejects_others() {
        let serialized = ok(serde_json::to_string(&ContractVersion));
        assert_eq!(serialized, "1");
        assert!(serde_json::from_str::<ContractVersion>("1").is_ok());
        assert!(serde_json::from_str::<ContractVersion>("2").is_err());
        assert!(serde_json::from_str::<ContractVersion>("0").is_err());
    }
}
