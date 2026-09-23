//! Frozen reference formats re-validated locally (the flauz-cap/flauz-orch
//! pattern): canonical entity IDs and actor references cross the seam as
//! frozen format strings, never as cargo dependencies.
//!
//! The grammar is owned by `flauz-world` (kernel §2); this crate
//! re-validates the same frozen formats locally so a reference serialized
//! by either crate has byte-identical meaning on the wire:
//!
//! - a canonical entity ID is `<kind>_<ULID>` — 26 characters of
//!   Crockford Base32 (uppercase; no `I`, `L`, `O` or `U`) after the
//!   frozen kind prefix. Collaboration references the kinds it needs:
//!   `ws` (the workspace a roster belongs to) and `task` (the task a
//!   sharing posture belongs to);
//! - an actor reference is `{"kind": "user|agent|system|provider",
//!   "id": "<canonical entity id or principal string>"}` — the member
//!   identity ref of a [`WorkspaceMembership`](crate::WorkspaceMembership)
//!   is exactly this frozen shape, carried as data.
//!
//! No external `ulid` crate and no generation: this crate generates no
//! identifiers (kernel §7 — the evaluator and simulator are pure
//! functions of their inputs); it only validates.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{CollabError, ensure_name};

/// Length of the ULID part of a canonical identifier (kernel §2).
const ULID_LEN: usize = 26;

/// The Crockford Base32 alphabet of canonical identifiers (kernel §2):
/// uppercase, no `I`, `L`, `O` or `U`.
const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Validates a canonical identifier string `<kind>_<ULID>` against one
/// expected frozen kind prefix.
fn validate_canonical_id(field: &'static str, kind: &str, value: &str) -> Result<(), CollabError> {
    let (found_kind, rest) = value.split_once('_').ok_or_else(|| {
        CollabError::invalid(format!("{field} {value:?} is missing the `_` separator"))
    })?;
    if found_kind != kind {
        return Err(CollabError::invalid(format!(
            "{field} {value:?} must have the {kind:?} kind prefix"
        )));
    }
    if rest.len() != ULID_LEN {
        return Err(CollabError::invalid(format!(
            "{field} {value:?} ULID part must be {ULID_LEN} characters, found {}",
            rest.len()
        )));
    }
    for (position, character) in rest.char_indices() {
        if !character.is_ascii() || !CROCKFORD_ALPHABET.contains(&(character as u8)) {
            return Err(CollabError::invalid(format!(
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
            pub fn parse(value: &str) -> Result<Self, CollabError> {
                validate_canonical_id(stringify!($name), $prefix, value)?;
                Ok(Self(value.to_owned()))
            }

            /// Returns the canonical reference string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Validates the reference against the frozen grammar.
            pub fn validate(&self) -> Result<(), CollabError> {
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
    /// A canonical Workspace reference (`ws_<ULID>`) — the collaboration
    /// root a roster, grant or presence record belongs to. The workspace
    /// is the collaboration root for members, permissions, presence and
    /// policy (FLAUZ-SOURCE-OF-TRUTH).
    WorkspaceRef,
    "ws"
);

canonical_ref!(
    /// A canonical Task reference (`task_<ULID>`) — the task a sharing
    /// posture (worktree policy + context visibility) or a presence
    /// locator belongs to. Task identity is the ID (kernel §6): sharing a
    /// task never forks it.
    TaskRef,
    "task"
);

/// The kind of an actor attribution (the frozen actor grammar, kernel
/// §2). Workspace members are `user` principals; agents may join as
/// collaborators in later waves.
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

/// The member identity ref — the frozen actor-reference grammar
/// `{"kind": "user|agent|system|provider", "id": "…"}` re-validated
/// locally (the `flauz-world` wire format, carried as data). A
/// [`WorkspaceMembership`](crate::WorkspaceMembership) references its
/// member through exactly this shape; the id is a canonical entity ID or
/// a principal string, never credential material.
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
    /// bounded string (the flauz-world bound).
    pub fn new(kind: ActorKind, id: &str) -> Result<Self, CollabError> {
        ensure_name("actor id", id)?;
        Ok(Self {
            kind,
            id: id.to_owned(),
        })
    }

    /// Builds a `user` actor reference (the workspace member shape).
    pub fn user(id: &str) -> Result<Self, CollabError> {
        Self::new(ActorKind::User, id)
    }

    /// Builds an `agent` actor reference.
    pub fn agent(id: &str) -> Result<Self, CollabError> {
        Self::new(ActorKind::Agent, id)
    }

    /// Builds a `system` actor reference.
    pub fn system(id: &str) -> Result<Self, CollabError> {
        Self::new(ActorKind::System, id)
    }

    /// Builds a `provider` actor reference.
    pub fn provider(id: &str) -> Result<Self, CollabError> {
        Self::new(ActorKind::Provider, id)
    }

    /// Validates the actor reference bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        Self::new(self.kind, &self.id)?;
        Ok(())
    }
}

impl fmt::Display for ActorRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?} {}", self.kind, self.id)
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

    /// The frozen kernel §2 ID vectors for the kinds this crate
    /// references: the valid forms parse, and each invalid form is
    /// rejected for its frozen reason.
    #[test]
    fn kernel_id_valid_vectors() {
        assert!(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0").is_ok());
        assert!(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1").is_ok());
    }

    /// The frozen kernel §2 invalid vectors, each with its reason: bad
    /// separator, uppercase kind, lowercase ULID, wrong length,
    /// non-Crockford letters, missing parts, and wrong kind prefix.
    #[test]
    fn kernel_id_invalid_vectors() {
        for invalid in [
            "taskX_01J8ZQ5V8K3T2B7N6X4R9DQPA0", // bad separator
            "TASK_01J8ZQ5V8K3T2B7N6X4R9DQPA0",  // uppercase kind
            "task_01j8zq5v8k3t2b7n6x4r9dqpa0",  // lowercase ULID
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPA",   // 25-char ULID
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPI0",  // contains I
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPU0",  // contains U
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPL0",  // contains L
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPO0",  // contains O
            "task",                             // no separator
            "task_",                            // empty ULID
            "",                                 // empty string
            "01J8ZQ5V8K3T2B7N6X4R9DQPA0",       // missing kind
        ] {
            assert!(
                TaskRef::parse(invalid).is_err(),
                "{invalid:?} must be rejected by the frozen grammar"
            );
        }
        // Kind prefixes are frozen: a task ID is not a workspace ID.
        assert!(WorkspaceRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1").is_err());
    }

    /// References serialize as their bare canonical string and round-trip
    /// through strict parses.
    #[test]
    fn references_serialize_as_bare_canonical_strings() {
        let workspace = ok(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0"));
        assert_eq!(
            ok(serde_json::to_string(&workspace)),
            "\"ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0\""
        );
        let reloaded: WorkspaceRef = ok(serde_json::from_str("\"ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0\""));
        assert_eq!(reloaded, workspace);
        // Non-string JSON is rejected.
        assert!(serde_json::from_str::<WorkspaceRef>("42").is_err());
    }

    /// The actor reference serializes with the frozen `kind`/`id` shape
    /// (the flauz-world wire format) and rejects unknown fields and
    /// unknown kinds.
    #[test]
    fn actor_refs_serialize_with_the_frozen_grammar() {
        let member = ok(ActorRef::user("ana"));
        let serialized = ok(serde_json::to_string(&member));
        assert_eq!(serialized, "{\"kind\":\"user\",\"id\":\"ana\"}");
        let reloaded: ActorRef = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, member);
        assert!(ActorRef::new(ActorKind::User, "").is_err());
        assert!(
            serde_json::from_str::<ActorRef>("{\"kind\":\"robot\",\"id\":\"x\"}").is_err(),
            "unknown actor kinds are rejected"
        );
        assert!(
            serde_json::from_str::<ActorRef>("{\"kind\":\"user\",\"id\":\"x\",\"x\":1}").is_err(),
            "unknown fields are rejected"
        );
    }
}
