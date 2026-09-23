//! Frozen reference formats re-validated locally (the
//! flauz-cap/flauz-collab pattern): canonical entity IDs and actor
//! references cross the seam as frozen format strings, never as cargo
//! dependencies.
//!
//! The grammar is owned by `flauz-world` (kernel §2); this crate
//! re-validates the same frozen formats locally so a reference serialized
//! by either crate has byte-identical meaning on the wire:
//!
//! - a canonical entity ID is `<kind>_<ULID>` — 26 characters of
//!   Crockford Base32 (uppercase; no `I`, `L`, `O` or `U`) after the
//!   frozen kind prefix. Leases reference the kinds they need: `res`
//!   (the resource a lease scopes), `lease` (a lease being renewed or
//!   released) and `task` (the task a waiting requester belongs to, as
//!   call-side attribution data);
//! - an actor reference is `{"kind": "user|agent|system|provider",
//!   "id": "<canonical entity id or principal string>"}` — the frozen
//!   actor shape that `flauz-world` owns, carried as data.
//!
//! No external `ulid` crate and no generation: this crate generates no
//! identifiers (kernel §7 — the manager is a pure function of its
//! inputs); it only validates. The granted lease id is caller-minted
//! through the request.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{LeaseError, MAX_ACTOR_ID_BYTES, ensure_non_empty, ensure_str_bound};

/// Length of the ULID part of a canonical identifier (kernel §2).
const ULID_LEN: usize = 26;

/// The Crockford Base32 alphabet of canonical identifiers (kernel §2):
/// uppercase, no `I`, `L`, `O` or `U`.
const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Validates a canonical identifier string `<kind>_<ULID>` against one
/// expected frozen kind prefix.
fn validate_canonical_id(field: &'static str, kind: &str, value: &str) -> Result<(), LeaseError> {
    let (found_kind, rest) = value.split_once('_').ok_or_else(|| {
        LeaseError::invalid(format!("{field} {value:?} is missing the `_` separator"))
    })?;
    if found_kind != kind {
        return Err(LeaseError::invalid(format!(
            "{field} {value:?} must have the {kind:?} kind prefix"
        )));
    }
    if rest.len() != ULID_LEN {
        return Err(LeaseError::invalid(format!(
            "{field} {value:?} ULID part must be {ULID_LEN} characters, found {}",
            rest.len()
        )));
    }
    for (position, character) in rest.char_indices() {
        if !character.is_ascii() || !CROCKFORD_ALPHABET.contains(&(character as u8)) {
            return Err(LeaseError::invalid(format!(
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
            /// Returns [`LeaseError`] when the value is not a valid
            /// `<prefix>_<ULID>` canonical reference.
            pub fn parse(value: &str) -> Result<Self, LeaseError> {
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
            /// Returns [`LeaseError`] when the reference violates the
            /// frozen grammar.
            pub fn validate(&self) -> Result<(), LeaseError> {
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
    /// A canonical Resource reference (`res_<ULID>`) — the resource a
    /// lease scopes (kernel §7: a resource ID is stable when surfaces
    /// change; the lease manager only ever sees the stable id).
    ResourceRef,
    "res"
);

canonical_ref!(
    /// A canonical ResourceLease reference (`lease_<ULID>`) — a lease
    /// being named, renewed or released. The durable lease entity is
    /// owned by the world store; this reference crosses the seam as the
    /// frozen format string.
    LeaseRef,
    "lease"
);

canonical_ref!(
    /// A canonical Task reference (`task_<ULID>`) — call-side
    /// attribution for the task a waiting requester belongs to. Task
    /// identity is the ID (kernel §6); waiting never forks a task.
    TaskRef,
    "task"
);

/// The kind of an actor attribution (the frozen actor grammar, kernel
/// §2). Lease holders and requesters are attributed through exactly
/// this shape; the id is a canonical entity ID or a principal string,
/// never credential material.
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

/// Attribution to the actor that holds or requests a lease — the frozen
/// actor-reference grammar `{"kind": "user|agent|system|provider",
/// "id": "…"}` re-validated locally (the `flauz-world` wire format,
/// carried as data). Every lease conflict names its actors through this
/// shape (the conflict-honesty law).
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
    /// Returns [`LeaseError`] when the id is empty or exceeds
    /// [`MAX_ACTOR_ID_BYTES`].
    pub fn new(kind: ActorKind, id: &str) -> Result<Self, LeaseError> {
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
    /// Returns [`LeaseError`] when the id is empty or over the bound.
    pub fn user(id: &str) -> Result<Self, LeaseError> {
        Self::new(ActorKind::User, id)
    }

    /// Builds an `agent` actor reference.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the id is empty or over the bound.
    pub fn agent(id: &str) -> Result<Self, LeaseError> {
        Self::new(ActorKind::Agent, id)
    }

    /// Builds a `system` actor reference.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the id is empty or over the bound.
    pub fn system(id: &str) -> Result<Self, LeaseError> {
        Self::new(ActorKind::System, id)
    }

    /// Builds a `provider` actor reference.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the id is empty or over the bound.
    pub fn provider(id: &str) -> Result<Self, LeaseError> {
        Self::new(ActorKind::Provider, id)
    }

    /// Validates the actor reference bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the reference violates the frozen
    /// grammar bounds.
    pub fn validate(&self) -> Result<(), LeaseError> {
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
        // The frozen §2 vectors (the res_ one verbatim; lease_/task_
        // carry the same ULID grammar).
        assert!(ResourceRef::parse("res_01J8ZQ5V8K3T2B7N6X4R9DQPC2").is_ok());
        assert!(
            ok(LeaseRef::parse("lease_01J8ZQ5V8K3T2B7N6X4R9DQPG6"))
                .validate()
                .is_ok()
        );
        assert!(
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"))
                .validate()
                .is_ok()
        );

        let invalid = [
            "resX_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "RES_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "res_01j8zq5v8k3t2b7n6x4r9dqpa0",
            "res_01J8ZQ5V8K3T2B7N6X4R9DQPA",
            "res_01J8ZQ5V8K3T2B7N6X4R9DQPI0",
            "res_01J8ZQ5V8K3T2B7N6X4R9DQPU0",
            "res_01J8ZQ5V8K3T2B7N6X4R9DQPL0",
            "res_01J8ZQ5V8K3T2B7N6X4R9DQPO0",
            "res",
            "res_",
            "",
            "01J8ZQ5V8K3T2B7N6X4R9DQPA0",
        ];
        for value in invalid {
            assert!(
                ResourceRef::parse(value).is_err(),
                "{value:?} must be rejected"
            );
            assert!(
                LeaseRef::parse(value).is_err(),
                "{value:?} must be rejected"
            );
        }
        // Kind prefixes must match: a res_ id is not a lease id.
        assert!(LeaseRef::parse("res_01J8ZQ5V8K3T2B7N6X4R9DQPC2").is_err());
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
    }
}
