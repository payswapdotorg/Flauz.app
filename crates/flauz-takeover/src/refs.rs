//! Frozen reference formats re-validated locally (the
//! flauz-cap/flauz-lease pattern): canonical entity IDs, node names
//! and actor references cross the seam as frozen format strings, never
//! as cargo dependencies.
//!
//! The grammars are owned by the contract crates (kernel §2 /
//! flauz-orch's name bound); this crate re-validates the same frozen
//! formats locally so a reference serialized by either crate has
//! byte-identical meaning on the wire:
//!
//! - a canonical entity ID is `<kind>_<ULID>` — 26 characters of
//!   Crockford Base32 (uppercase; no `I`, `L`, `O` or `U`) after the
//!   frozen kind prefix. Takeover records reference the kinds they
//!   need: `task` (the task whose stream every record lands on — the
//!   same-stream law's anchor), `agent` (the agent whose turn the
//!   human takes over) and `art` (already-attributed artifacts kept
//!   by a takeover or a cancellation);
//! - a node name is a bounded non-empty string (the flauz-orch name
//!   grammar — node identities are NAMES, not ULIDs);
//! - an actor reference is `{"kind": "user|agent|system|provider",
//!   "id": "<canonical entity id or principal string>"}` — the frozen
//!   actor shape that `flauz-world` owns, carried as data.
//!
//! No external `ulid` crate and no generation: this crate generates no
//! identifiers (kernel §7 — every record is a pure function of its
//! inputs); it only validates.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{TakeoverError, MAX_ACTOR_ID_BYTES, ensure_name, ensure_non_empty, ensure_str_bound};

/// Length of the ULID part of a canonical identifier (kernel §2).
const ULID_LEN: usize = 26;

/// The Crockford Base32 alphabet of canonical identifiers (kernel §2):
/// uppercase, no `I`, `L`, `O` or `U`.
const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Validates a canonical identifier string `<kind>_<ULID>` against one
/// expected frozen kind prefix.
fn validate_canonical_id(
    field: &'static str,
    kind: &str,
    value: &str,
) -> Result<(), TakeoverError> {
    let (found_kind, rest) = value.split_once('_').ok_or_else(|| {
        TakeoverError::invalid(format!("{field} {value:?} is missing the `_` separator"))
    })?;
    if found_kind != kind {
        return Err(TakeoverError::invalid(format!(
            "{field} {value:?} must have the {kind:?} kind prefix"
        )));
    }
    if rest.len() != ULID_LEN {
        return Err(TakeoverError::invalid(format!(
            "{field} {value:?} ULID part must be {ULID_LEN} characters, found {}",
            rest.len()
        )));
    }
    for (position, character) in rest.char_indices() {
        if !character.is_ascii() || !CROCKFORD_ALPHABET.contains(&(character as u8)) {
            return Err(TakeoverError::invalid(format!(
                "{field} {value:?} contains {character:?} at position {position}, outside the \
                 Crockford Base32 alphabet"
            )));
        }
    }
    Ok(())
}

macro_rules! canonical_ref {
    ($(#[$doc:meta])* $name:ident, $prefix:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Parses and validates a canonical reference of this kind
            /// against the frozen grammar.
            ///
            /// # Errors
            ///
            /// Returns [`TakeoverError`] when the value is not a valid
            /// `<prefix>_<ULID>` canonical reference.
            pub fn parse(value: &str) -> Result<Self, TakeoverError> {
                validate_canonical_id(stringify!($name), $prefix, value)?;
                Ok(Self(value.to_owned()))
            }

            /// Returns the canonical reference string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Validates the reference against the frozen grammar.
            ///
            /// # Errors
            ///
            /// Returns [`TakeoverError`] when the reference violates the
            /// frozen grammar.
            pub fn validate(&self) -> Result<(), TakeoverError> {
                validate_canonical_id(stringify!($name), $prefix, &self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::parse(&text).map_err(serde::de::Error::custom)
            }
        }
    };
}

canonical_ref!(
    /// A canonical Task reference (`task_<ULID>`) — the task whose
    /// event stream every takeover/approval/cancellation record lands
    /// on (the same-stream law). Task identity is the ID (kernel §6);
    /// a takeover never forks a task.
    TaskRef,
    "task"
);

canonical_ref!(
    /// A canonical Agent reference (`agent_<ULID>`) — the agent whose
    /// turn the human takes over, or the agent requesting an approval.
    /// The durable agent entity is owned by the exec-side contracts;
    /// this reference crosses the seam as the frozen format string.
    AgentRef,
    "agent"
);

canonical_ref!(
    /// A canonical Artifact reference (`art_<ULID>`) — already-
    /// attributed work kept verbatim by a takeover (the projection
    /// law) or kept despite a cancellation (the honesty law).
    ArtifactRef,
    "art"
);

/// A node's name — the flauz-orch name grammar as data (node
/// identities are bounded names, not ULIDs). The graph the
/// propagation projects over references its nodes through this shape.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeName(String);

impl NodeName {
    /// Parses and validates a node name (non-empty, bounded).
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the name is empty or exceeds
    /// the bound.
    pub fn parse(value: &str) -> Result<Self, TakeoverError> {
        ensure_name("NodeName", value)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the node name string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the node name against the frozen bound.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the name is empty or over the
    /// bound.
    pub fn validate(&self) -> Result<(), TakeoverError> {
        ensure_name("NodeName", &self.0)
    }
}

impl fmt::Display for NodeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl serde::Serialize for NodeName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for NodeName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// The kind of an actor attribution (the frozen actor grammar, kernel
/// §2). Takeovers and approval decisions are attributed through
/// exactly this shape; the id is a canonical entity ID or a principal
/// string, never credential material.
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

/// Attribution to the acting human, agent or system — the frozen
/// actor-reference grammar `{"kind": "user|agent|system|provider",
/// "id": "…"}` re-validated locally (the `flauz-world` wire format,
/// carried as data). Every takeover and every approval decision names
/// its actor through this shape (the takeover law).
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
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the id is empty or exceeds
    /// [`MAX_ACTOR_ID_BYTES`].
    pub fn new(kind: ActorKind, id: &str) -> Result<Self, TakeoverError> {
        ensure_non_empty("actor id", id)?;
        ensure_str_bound("actor id", id, MAX_ACTOR_ID_BYTES)?;
        Ok(Self {
            kind,
            id: id.to_owned(),
        })
    }

    /// Builds a `user` actor reference.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the id is empty or over the bound.
    pub fn user(id: &str) -> Result<Self, TakeoverError> {
        Self::new(ActorKind::User, id)
    }

    /// Builds an `agent` actor reference.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the id is empty or over the bound.
    pub fn agent(id: &str) -> Result<Self, TakeoverError> {
        Self::new(ActorKind::Agent, id)
    }

    /// Builds a `system` actor reference.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the id is empty or over the bound.
    pub fn system(id: &str) -> Result<Self, TakeoverError> {
        Self::new(ActorKind::System, id)
    }

    /// Whether this actor is a human user principal (the takeover
    /// law's WHO: only a human takes over, only a human decides an
    /// approval — `HumanApproval != AgentDecision`).
    #[must_use]
    pub const fn is_human(&self) -> bool {
        matches!(self.kind, ActorKind::User)
    }

    /// Validates the actor reference bounds.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the reference violates the frozen
    /// grammar bounds.
    pub fn validate(&self) -> Result<(), TakeoverError> {
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
    fn canonical_refs_accept_the_kernel_vectors_and_reject_bad_ones() {
        // The frozen §2 vectors (task_ verbatim; agent_/art_ carry the
        // same ULID grammar).
        assert!(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1").is_ok());
        assert!(
            ok(AgentRef::parse("agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6"))
                .validate()
                .is_ok()
        );
        assert!(
            ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPH7"))
                .validate()
                .is_ok()
        );

        let invalid = [
            "taskX_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "TASK_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "task_01j8zq5v8k3t2b7n6x4r9dqpa0",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPA",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPI0",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPU0",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPL0",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPO0",
            "task",
            "task_",
            "",
            "01J8ZQ5V8K3T2B7N6X4R9DQPA0",
        ];
        for value in invalid {
            assert!(TaskRef::parse(value).is_err(), "{value:?} must be rejected");
            assert!(AgentRef::parse(value).is_err(), "{value:?} must be rejected");
            assert!(
                ArtifactRef::parse(value).is_err(),
                "{value:?} must be rejected"
            );
        }
        // Kind prefixes must match: a task_ id is not an agent id.
        assert!(AgentRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1").is_err());
    }

    #[test]
    fn node_names_follow_the_bounded_name_grammar() {
        assert!(NodeName::parse("research").is_ok());
        assert!(NodeName::parse("environment-setup").is_ok());
        assert!(NodeName::parse("").is_err(), "empty names are rejected");
        let long = "x".repeat(MAX_NODE_NAME_BYTES + 1);
        assert!(NodeName::parse(&long).is_err(), "over-long names are rejected");
        let bounded = "x".repeat(MAX_NODE_NAME_BYTES);
        assert!(NodeName::parse(&bounded).is_ok());
    }

    #[test]
    fn actor_refs_serialize_to_the_frozen_shape() {
        let actor = ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6"));
        let serialized = ok(serde_json::to_string(&actor));
        assert_eq!(
            serialized,
            "{\"kind\":\"agent\",\"id\":\"agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6\"}"
        );
        let reloaded: ActorRef = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, actor);
        assert!(
            serde_json::from_str::<ActorRef>(
                "{\"kind\":\"agent\",\"id\":\"agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6\",\"extra\":1}"
            )
            .is_err(),
            "unknown fields are rejected"
        );
        assert!(ActorRef::user("").is_err(), "empty actor ids are rejected");
        let human = ok(ActorRef::user("user_ana"));
        assert!(human.is_human());
        assert!(!actor.is_human());
    }
}
