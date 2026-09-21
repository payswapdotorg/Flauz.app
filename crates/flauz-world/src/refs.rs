//! Actor, entity and stream references (kernel §2, §5).
//!
//! Actors are `{"kind": "user|agent|system|provider", "id": "..."}` where the
//! id is a canonical entity ID or a principal string. Entity references name
//! a subject by its registered kind plus canonical ID. Event streams are
//! workspace, task or session streams.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::WorldError;
use crate::ids::{EntityKind, EventId, IdError, SessionId, TaskId, WorkspaceId, validate};

/// The kind of an actor attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    /// A human user principal.
    User,
    /// An agent (referenced by its `agent_` canonical ID).
    Agent,
    /// A Flauz system component.
    System,
    /// A provider (referenced by its canonical ID or principal string).
    Provider,
}

/// Attribution to the actor that produced a record.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorRef {
    /// The actor kind.
    pub kind: ActorKind,
    /// The canonical entity ID or principal string of the actor.
    pub id: String,
}

impl ActorRef {
    /// Builds an actor reference, validating that the id is a non-empty,
    /// bounded string.
    pub fn new(kind: ActorKind, id: &str) -> Result<Self, WorldError> {
        crate::ensure_non_empty("actor id", id)?;
        crate::ensure_str_bound("actor id", id, crate::MAX_ACTOR_ID_BYTES)?;
        Ok(Self {
            kind,
            id: id.to_owned(),
        })
    }

    /// Builds a `user` actor reference.
    pub fn user(id: &str) -> Result<Self, WorldError> {
        Self::new(ActorKind::User, id)
    }

    /// Builds an `agent` actor reference.
    pub fn agent(id: &str) -> Result<Self, WorldError> {
        Self::new(ActorKind::Agent, id)
    }

    /// Builds a `system` actor reference.
    pub fn system(id: &str) -> Result<Self, WorldError> {
        Self::new(ActorKind::System, id)
    }

    /// Builds a `provider` actor reference.
    pub fn provider(id: &str) -> Result<Self, WorldError> {
        Self::new(ActorKind::Provider, id)
    }

    /// Validates the actor reference bounds.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(self.kind, &self.id)?;
        Ok(())
    }
}

/// A reference to an entity by its registered kind and canonical ID. The ID
/// prefix must match the kind. Foreign entities (for example `env_` or
/// `ctxsnap_`) can be referenced without owning their types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityRef {
    /// The registered entity kind.
    pub entity_kind: EntityKind,
    /// The canonical entity ID.
    pub id: String,
}

impl EntityRef {
    /// Builds an entity reference from a kind and a full canonical ID
    /// string, validating the ID and its kind consistency.
    pub fn new(entity_kind: EntityKind, id: &str) -> Result<Self, WorldError> {
        let parsed = validate(id)?;
        if parsed != entity_kind {
            return Err(WorldError::Id(IdError::KindMismatch {
                expected: entity_kind.prefix(),
                found: parsed.prefix(),
            }));
        }
        Ok(Self {
            entity_kind,
            id: id.to_owned(),
        })
    }

    /// Builds a workspace entity reference.
    #[must_use]
    pub fn workspace(id: &WorkspaceId) -> Self {
        Self {
            entity_kind: EntityKind::Workspace,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds a session entity reference.
    #[must_use]
    pub fn session(id: &SessionId) -> Self {
        Self {
            entity_kind: EntityKind::Session,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds a task entity reference.
    #[must_use]
    pub fn task(id: &TaskId) -> Self {
        Self {
            entity_kind: EntityKind::Task,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds an artifact entity reference.
    #[must_use]
    pub fn artifact(id: &crate::ids::ArtifactId) -> Self {
        Self {
            entity_kind: EntityKind::Artifact,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds a resource entity reference.
    #[must_use]
    pub fn resource(id: &crate::ids::ResourceId) -> Self {
        Self {
            entity_kind: EntityKind::Resource,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds an event entity reference.
    #[must_use]
    pub fn event(id: &EventId) -> Self {
        Self {
            entity_kind: EntityKind::Event,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds an observation entity reference.
    #[must_use]
    pub fn observation(id: &crate::ids::ObservationId) -> Self {
        Self {
            entity_kind: EntityKind::Observation,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds a claim entity reference.
    #[must_use]
    pub fn claim(id: &crate::ids::ClaimId) -> Self {
        Self {
            entity_kind: EntityKind::Claim,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds an evidence entity reference.
    #[must_use]
    pub fn evidence(id: &crate::ids::EvidenceId) -> Self {
        Self {
            entity_kind: EntityKind::Evidence,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds a lease entity reference.
    #[must_use]
    pub fn lease(id: &crate::ids::LeaseId) -> Self {
        Self {
            entity_kind: EntityKind::ResourceLease,
            id: id.as_str().to_owned(),
        }
    }

    /// Builds a procedure entity reference.
    #[must_use]
    pub fn procedure(id: &crate::ids::ProcedureId) -> Self {
        Self {
            entity_kind: EntityKind::Procedure,
            id: id.as_str().to_owned(),
        }
    }

    /// Validates the reference.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(self.entity_kind, &self.id)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntityRef {
    entity_kind: EntityKind,
    id: String,
}

impl Serialize for EntityRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        RawEntityRef {
            entity_kind: self.entity_kind,
            id: self.id.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EntityRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawEntityRef::deserialize(deserializer)?;
        Self::new(raw.entity_kind, &raw.id).map_err(serde::de::Error::custom)
    }
}

/// The kind of an event stream (kernel §3, §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamKind {
    /// A workspace stream.
    Workspace,
    /// A task stream.
    Task,
    /// A session stream.
    Session,
}

/// A reference to an event stream: `{"kind": "workspace|task|session",
/// "id": "<canonical id>"}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StreamRef {
    /// A workspace stream.
    Workspace(WorkspaceId),
    /// A task stream.
    Task(TaskId),
    /// A session stream.
    Session(SessionId),
}

impl StreamRef {
    /// Builds a workspace stream reference.
    #[must_use]
    pub fn workspace(id: &WorkspaceId) -> Self {
        Self::Workspace(id.clone())
    }

    /// Builds a task stream reference.
    #[must_use]
    pub fn task(id: &TaskId) -> Self {
        Self::Task(id.clone())
    }

    /// Builds a session stream reference.
    #[must_use]
    pub fn session(id: &SessionId) -> Self {
        Self::Session(id.clone())
    }

    /// Returns the stream kind.
    #[must_use]
    pub const fn kind(&self) -> StreamKind {
        match self {
            Self::Workspace(_) => StreamKind::Workspace,
            Self::Task(_) => StreamKind::Task,
            Self::Session(_) => StreamKind::Session,
        }
    }

    /// Returns the canonical stream ID.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Workspace(id) => id.as_str(),
            Self::Task(id) => id.as_str(),
            Self::Session(id) => id.as_str(),
        }
    }
}

impl fmt::Display for StreamRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} stream {}",
            match self.kind() {
                StreamKind::Workspace => "workspace",
                StreamKind::Task => "task",
                StreamKind::Session => "session",
            },
            self.id()
        )
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStreamRef {
    kind: StreamKind,
    id: String,
}

impl Serialize for StreamRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        RawStreamRef {
            kind: self.kind(),
            id: self.id().to_owned(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StreamRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawStreamRef::deserialize(deserializer)?;
        let kind = validate(&raw.id).map_err(serde::de::Error::custom)?;
        let mismatch = |expected: &'static str| IdError::KindMismatch {
            expected,
            found: kind.prefix(),
        };
        match raw.kind {
            StreamKind::Workspace => {
                if kind != EntityKind::Workspace {
                    return Err(serde::de::Error::custom(mismatch("ws")));
                }
                WorkspaceId::parse(&raw.id)
                    .map(Self::Workspace)
                    .map_err(serde::de::Error::custom)
            }
            StreamKind::Task => {
                if kind != EntityKind::Task {
                    return Err(serde::de::Error::custom(mismatch("task")));
                }
                TaskId::parse(&raw.id)
                    .map(Self::Task)
                    .map_err(serde::de::Error::custom)
            }
            StreamKind::Session => {
                if kind != EntityKind::Session {
                    return Err(serde::de::Error::custom(mismatch("sess")));
                }
                SessionId::parse(&raw.id)
                    .map(Self::Session)
                    .map_err(serde::de::Error::custom)
            }
        }
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
    fn actor_ref_serializes_with_kind_field() {
        let actor = ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6"));
        let serialized = ok(serde_json::to_string(&actor));
        assert_eq!(
            serialized,
            "{\"kind\":\"agent\",\"id\":\"agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6\"}"
        );
        assert!(ActorRef::new(ActorKind::User, "").is_err());
        assert!(serde_json::from_str::<ActorRef>("{\"kind\":\"robot\",\"id\":\"x\"}").is_err());
        assert!(
            serde_json::from_str::<ActorRef>("{\"kind\":\"user\",\"id\":\"x\",\"x\":1}").is_err()
        );
    }

    #[test]
    fn entity_ref_serializes_with_entity_kind_and_validates_prefix() {
        let subject = ok(EntityRef::new(
            EntityKind::Task,
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1",
        ));
        let serialized = ok(serde_json::to_string(&subject));
        assert_eq!(
            serialized,
            "{\"entity_kind\":\"task\",\"id\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\"}"
        );
        let parsed: EntityRef = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, subject);
        assert!(EntityRef::new(EntityKind::Task, "ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0").is_err());
        // Foreign kinds are referenceable by their frozen prefixes.
        assert!(EntityRef::new(EntityKind::Environment, "env_01J8ZQ5V8K3T2B7N6X4R9DQPE4").is_ok());
        assert!(
            serde_json::from_str::<EntityRef>("{\"entity_kind\":\"task\",\"id\":\"x\"}").is_err()
        );
    }

    #[test]
    fn stream_ref_serializes_like_the_kernel_envelope() {
        let stream = StreamRef::workspace(&ok(WorkspaceId::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0")));
        let serialized = ok(serde_json::to_string(&stream));
        assert_eq!(
            serialized,
            "{\"kind\":\"workspace\",\"id\":\"ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0\"}"
        );
        let parsed: StreamRef = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, stream);
        // kind/id inconsistency is rejected
        let swapped = "{\"kind\":\"task\",\"id\":\"ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0\"}";
        assert!(serde_json::from_str::<StreamRef>(swapped).is_err());
        assert!(
            serde_json::from_str::<StreamRef>(
                "{\"kind\":\"task\",\"id\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\",\"extra\":true}"
            )
            .is_err()
        );
    }
}
