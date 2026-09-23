//! Frozen reference formats re-validated locally (the flauz-cap
//! pattern): canonical entity IDs and actor references cross the seam as
//! frozen format strings, never as cargo dependencies.
//!
//! The grammar is owned by `flauz-world` (kernel §2); this crate
//! re-validates the same frozen formats locally so a reference
//! serialized by either crate has byte-identical meaning on the wire:
//!
//! - a canonical entity ID is `<kind>_<ULID>` — 26 characters of
//!   Crockford Base32 (uppercase; no `I`, `L`, `O` or `U`) after the
//!   frozen kind prefix. The graph references the kinds it needs:
//!   `task` (the graph's task), `art` (artifacts), `evd` (evidence) and
//!   `res` (resources);
//! - an actor reference is `{"kind": "user|agent|system|provider",
//!   "id": "<canonical entity id or principal string>"}`.
//!
//! No external `ulid` crate and no generation: this crate generates no
//! identifiers (kernel §7 — the evaluator is a pure function of its
//! inputs); it only validates.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{OrchError, ensure_name};

/// Length of the ULID part of a canonical identifier (kernel §2).
const ULID_LEN: usize = 26;

/// The Crockford Base32 alphabet of canonical identifiers (kernel §2):
/// uppercase, no `I`, `L`, `O` or `U`.
const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Validates a canonical identifier string `<kind>_<ULID>` against one
/// expected frozen kind prefix.
fn validate_canonical_id(field: &'static str, kind: &str, value: &str) -> Result<(), OrchError> {
    let (found_kind, rest) = value.split_once('_').ok_or_else(|| {
        OrchError::invalid(format!("{field} {value:?} is missing the `_` separator"))
    })?;
    if found_kind != kind {
        return Err(OrchError::invalid(format!(
            "{field} {value:?} must have the {kind:?} kind prefix"
        )));
    }
    if rest.len() != ULID_LEN {
        return Err(OrchError::invalid(format!(
            "{field} {value:?} ULID part must be {ULID_LEN} characters, found {}",
            rest.len()
        )));
    }
    for (position, character) in rest.char_indices() {
        if !character.is_ascii() || !CROCKFORD_ALPHABET.contains(&(character as u8)) {
            return Err(OrchError::invalid(format!(
                "{field} {value:?} contains {character:?} at position {position}, outside the \
                 Crockford Base32 alphabet"
            )));
        }
    }
    Ok(())
}

macro_rules! canonical_ref {
    ($(#[$doc:meta])* $name:ident, $prefix:literal, $serialized_name:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Parses and validates a canonical reference of this kind
            /// against the frozen grammar.
            pub fn parse(value: &str) -> Result<Self, OrchError> {
                validate_canonical_id(stringify!($name), $prefix, value)?;
                Ok(Self(value.to_owned()))
            }

            /// Returns the canonical reference string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Validates the reference against the frozen grammar.
            pub fn validate(&self) -> Result<(), OrchError> {
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
    /// A canonical Task reference (`task_<ULID>`) — the graph's task.
    /// Task identity is the ID (kernel §6): run-again creates a NEW task
    /// reference, never a fork.
    TaskRef,
    "task",
    "task_ref"
);
canonical_ref!(
    /// A canonical Artifact reference (`art_<ULID>`) — the durable work
    /// products nodes produce and wait for.
    ArtifactRef,
    "art",
    "artifact_ref"
);
canonical_ref!(
    /// A canonical Evidence reference (`evd_<ULID>`) — verification
    /// records, attributed to the verifier node that recorded them.
    EvidenceRef,
    "evd",
    "evidence_ref"
);
canonical_ref!(
    /// A canonical Resource reference (`res_<ULID>`) — the resources a
    /// node may wait on.
    ResourceRef,
    "res",
    "resource_ref"
);

/// The kind of an actor attribution (the frozen actor grammar, kernel
/// §2).
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

/// Attribution to the actor behind a graph node — the frozen actor
/// reference grammar `{"kind": "user|agent|system|provider", "id":
/// "…"}` re-validated locally (the `flauz-world` wire format).
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
    pub fn new(kind: ActorKind, id: &str) -> Result<Self, OrchError> {
        ensure_name("actor id", id)?;
        Ok(Self {
            kind,
            id: id.to_owned(),
        })
    }

    /// Builds a `user` actor reference.
    pub fn user(id: &str) -> Result<Self, OrchError> {
        Self::new(ActorKind::User, id)
    }

    /// Builds an `agent` actor reference.
    pub fn agent(id: &str) -> Result<Self, OrchError> {
        Self::new(ActorKind::Agent, id)
    }

    /// Builds a `system` actor reference.
    pub fn system(id: &str) -> Result<Self, OrchError> {
        Self::new(ActorKind::System, id)
    }

    /// Builds a `provider` actor reference.
    pub fn provider(id: &str) -> Result<Self, OrchError> {
        Self::new(ActorKind::Provider, id)
    }

    /// Validates the actor reference bounds.
    pub fn validate(&self) -> Result<(), OrchError> {
        Self::new(self.kind, &self.id)?;
        Ok(())
    }
}

impl fmt::Display for ActorRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?} {}", self.kind, self.id)
    }
}

/// What one node waits for before it can run (addendum §4): an
/// ARTIFACT produced by another node, or a RESOURCE becoming available.
///
/// The graph-relay law made structural: these two kinds are the ONLY
/// kinds — the serialized grammar (`"kind": "artifact_ready" |
/// "resource_ready"`) rejects every other wait kind on read, so a
/// context-snapshot wait, a transcript wait or any other chat-relay
/// edge is impossible to express, not merely discouraged.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum WaitOn {
    /// Wait for an artifact with `label` produced by `from_node` —
    /// shared task state, never a transcript.
    ArtifactReady {
        /// The node that will produce the artifact.
        from_node: String,
        /// The artifact's label (for example "the research notes"); the
        /// producing event carries the same label.
        label: String,
    },
    /// Wait for a resource to become available.
    ResourceReady {
        /// The resource.
        resource: ResourceRef,
        /// The resource's label (for example "the shared browser").
        label: String,
    },
}

impl WaitOn {
    /// Builds an artifact wait, validating bounds.
    pub fn artifact(from_node: &str, label: &str) -> Result<Self, OrchError> {
        ensure_name("wait node", from_node)?;
        ensure_name("wait label", label)?;
        Ok(Self::ArtifactReady {
            from_node: from_node.to_owned(),
            label: label.to_owned(),
        })
    }

    /// Builds a resource wait, validating bounds.
    pub fn resource(resource: ResourceRef, label: &str) -> Result<Self, OrchError> {
        ensure_name("wait label", label)?;
        Ok(Self::ResourceReady {
            resource,
            label: label.to_owned(),
        })
    }

    /// Validates the wait's bounds.
    pub fn validate(&self) -> Result<(), OrchError> {
        match self {
            Self::ArtifactReady { from_node, label } => {
                ensure_name("wait node", from_node)?;
                ensure_name("wait label", label)?;
            }
            Self::ResourceReady { resource, label } => {
                resource.validate()?;
                ensure_name("wait label", label)?;
            }
        }
        Ok(())
    }

    /// The user-facing label of what is awaited ("the research notes"),
    /// whatever the kind.
    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::ArtifactReady { label, .. } | Self::ResourceReady { label, .. } => label,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MAX_NAME_BYTES, ensure_str_bound};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn canonical_references_follow_the_frozen_grammar() {
        // The frozen kernel vectors (F2-CONTRACT-KERNEL §2), re-pinned
        // for the kinds this crate references: every valid vector parses
        // with its own kind; each invalid vector is rejected.
        assert!(
            TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1").is_ok(),
            "the frozen task vector parses"
        );
        assert!(
            ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPA0").is_ok(),
            "the frozen artifact vector parses"
        );
        assert!(
            EvidenceRef::parse("evd_01J8ZQ5V8K3T2B7N6X4R9DQPC2").is_ok(),
            "the frozen evidence vector parses"
        );
        assert!(
            ResourceRef::parse("res_01J8ZQ5V8K3T2B7N6X4R9DQPD3").is_ok(),
            "the frozen resource vector parses"
        );
        assert_eq!(
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")).as_str(),
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"
        );
        // Wrong kind prefix.
        assert!(TaskRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPA0").is_err());
        // Uppercase kind.
        assert!(TaskRef::parse("TASK_01J8ZQ5V8K3T2B7N6X4R9DQPA0").is_err());
        // Lowercase ULID.
        assert!(TaskRef::parse("task_01j8zq5v8k3t2b7n6x4r9dqpa0").is_err());
        // 25-char ULID.
        assert!(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPA").is_err());
        // Crockford exclusions.
        for bad in ['I', 'L', 'O', 'U'] {
            let id = format!("task_01J8ZQ5V8K3T2B7N6X4R9DQ{}0", bad);
            assert!(TaskRef::parse(&id).is_err(), "{bad} is not Crockford");
        }
        // No separator / empty.
        assert!(TaskRef::parse("task").is_err());
        assert!(TaskRef::parse("task_").is_err());
        assert!(TaskRef::parse("").is_err());
        // Missing kind.
        assert!(TaskRef::parse("01J8ZQ5V8K3T2B7N6X4R9DQPA0").is_err());
    }

    #[test]
    fn actor_references_follow_the_frozen_grammar() {
        let agent = ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4"));
        let serialized = ok(serde_json::to_string(&agent));
        assert_eq!(
            serialized,
            "{\"kind\":\"agent\",\"id\":\"agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4\"}"
        );
        let reloaded: ActorRef = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, agent);

        let user = ok(ActorRef::user("alice"));
        assert_eq!(
            ok(serde_json::to_string(&user)),
            "{\"kind\":\"user\",\"id\":\"alice\"}"
        );
        // Principal strings are valid actor ids (the frozen semantics).
        assert!(ActorRef::user("alice").is_ok());
        assert!(ActorRef::provider("conn_01J8ZQ5V8K3T2B7N6X4R9DQPF5").is_ok());
        // Empty and oversized ids are rejected.
        assert!(ActorRef::user("").is_err());
        assert!(ActorRef::agent(&"a".repeat(MAX_NAME_BYTES + 1)).is_err());
        // Unknown actor kinds and unknown fields are rejected on read.
        assert!(
            serde_json::from_str::<ActorRef>("{\"kind\":\"robot\",\"id\":\"x\"}").is_err(),
            "unknown actor kinds are rejected"
        );
        assert!(
            serde_json::from_str::<ActorRef>(
                "{\"kind\":\"user\",\"id\":\"alice\",\"name\":\"Alice\"}"
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn waits_carry_exactly_the_two_shared_state_kinds() {
        let artifact = ok(WaitOn::artifact("research", "the research notes"));
        assert_eq!(
            ok(serde_json::to_string(&artifact)),
            concat!(
                "{\"kind\":\"artifact_ready\",",
                "\"from_node\":\"research\",",
                "\"label\":\"the research notes\"}"
            )
        );
        let resource = ok(WaitOn::resource(
            ok(ResourceRef::parse("res_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
            "the shared browser",
        ));
        assert_eq!(
            ok(serde_json::to_string(&resource)),
            concat!(
                "{\"kind\":\"resource_ready\",",
                "\"resource\":\"res_01J8ZQ5V8K3T2B7N6X4R9DQPD3\",",
                "\"label\":\"the shared browser\"}"
            )
        );
        assert_eq!(artifact.label(), "the research notes");
        assert_eq!(resource.label(), "the shared browser");
        assert!(WaitOn::artifact("", "label").is_err());
        assert!(WaitOn::artifact("node", "").is_err());
        assert!(
            WaitOn::resource(ok(ResourceRef::parse("res_01J8ZQ5V8K3T2B7N6X4R9DQPD3")), "",)
                .is_err()
        );
    }

    #[test]
    fn the_wait_grammar_has_no_transcript_or_snapshot_kind() {
        // THE structural independence proof: a context-snapshot or
        // transcript wait cannot even be parsed — the relay abstraction
        // does not exist in the grammar.
        for forbidden in [
            "{\"kind\":\"context_snapshot\",\"from_node\":\"research\",\"label\":\"notes\"}",
            "{\"kind\":\"transcript_ready\",\"from_node\":\"research\",\"label\":\"notes\"}",
            "{\"kind\":\"chat_relay\",\"from_node\":\"research\",\"label\":\"notes\"}",
        ] {
            assert!(
                serde_json::from_str::<WaitOn>(forbidden).is_err(),
                "the wait grammar must reject {forbidden}"
            );
        }
    }

    #[test]
    fn oversized_actor_ids_are_bounded() {
        let long = "a".repeat(MAX_NAME_BYTES + 1);
        assert!(ActorRef::system(&long).is_err());
        assert!(ensure_str_bound("field", &long, MAX_NAME_BYTES).is_err());
    }
}
