//! Actor references (kernel §2).
//!
//! Actors are `{"kind": "user|agent|system|provider", "id": "<canonical
//! entity id or principal string>"}` — the frozen cross-crate actor format.
//! This is a local validating type over the frozen format, not a dependency
//! on the world crate: the serialized shape is identical on the wire.

use serde::{Deserialize, Serialize};

use crate::{ExecError, MAX_ACTOR_ID_BYTES};

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

/// Attribution to the actor that produced a record. Every remote execution
/// surface in this crate stays attributable through actor references.
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
    pub fn new(kind: ActorKind, id: &str) -> Result<Self, ExecError> {
        crate::ensure_non_empty("actor id", id)?;
        crate::ensure_str_bound("actor id", id, MAX_ACTOR_ID_BYTES)?;
        Ok(Self {
            kind,
            id: id.to_owned(),
        })
    }

    /// Builds a `user` actor reference.
    pub fn user(id: &str) -> Result<Self, ExecError> {
        Self::new(ActorKind::User, id)
    }

    /// Builds an `agent` actor reference.
    pub fn agent(id: &str) -> Result<Self, ExecError> {
        Self::new(ActorKind::Agent, id)
    }

    /// Builds a `system` actor reference.
    pub fn system(id: &str) -> Result<Self, ExecError> {
        Self::new(ActorKind::System, id)
    }

    /// Builds a `provider` actor reference.
    pub fn provider(id: &str) -> Result<Self, ExecError> {
        Self::new(ActorKind::Provider, id)
    }

    /// Validates the actor reference bounds.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(self.kind, &self.id)?;
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
    fn actor_ref_serializes_with_kind_field() {
        let actor = ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6"));
        let serialized = ok(serde_json::to_string(&actor));
        assert_eq!(
            serialized,
            "{\"kind\":\"agent\",\"id\":\"agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6\"}"
        );
        assert!(ActorRef::new(ActorKind::User, "").is_err());
        assert!(serde_json::from_str::<ActorRef>("{\"kind\":\"robot\",\"id\":\"x\"}").is_err());
        assert!(
            serde_json::from_str::<ActorRef>("{\"kind\":\"user\",\"id\":\"x\",\"x\":1}").is_err()
        );
    }
}
