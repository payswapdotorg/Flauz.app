//! Context provenance: every included context item is attributable to a
//! source, and carries the authorization class that governs its eligibility
//! as model context (context-harness architecture §2, kernel §7).
//!
//! The ten source kinds are the frozen vocabulary from the architecture:
//! user input, durable session event, memory item, artifact, resource
//! observation, tool result, retrieved document, skill, environment state,
//! and collaboration event.
//!
//! Authorization-aware filtering is *representable* here: an item whose
//! provenance carries [`AuthorizationClass::Secret`] is authorized for
//! retrieval by its owner but is never eligible as model context — secrets
//! and provider credentials never become context merely because they are
//! technically retrievable (kernel §7).

use serde::{Deserialize, Serialize};

use crate::ContextError;
use crate::ids::{ArtifactRef, EnvironmentRef, EventRef, MemoryItemId, ObservationRef};
use crate::refs::SkillRef;
use crate::{MAX_STATEMENT_BYTES, ensure_str_bound};

/// The authorization class of an included context item. Governs whether the
/// item may be presented to a model as context (kernel §7 authorization-aware
/// filtering).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthorizationClass {
    /// No restriction beyond the workspace policy.
    Public,
    /// Visible to workspace participants.
    Workspace,
    /// Visible to participants of the task the context belongs to.
    Task,
    /// Private to one participant; shared task state never includes it.
    Participant,
    /// Authorized for retrieval by its owner, but **never** eligible as
    /// model context. Secrets and provider credentials never become context
    /// merely because they are retrievable (kernel §7).
    Secret,
}

impl AuthorizationClass {
    /// Whether an item with this authorization class may be included in
    /// compiled model context. [`Self::Secret`] is never eligible.
    #[must_use]
    pub const fn eligible_for_model_context(self) -> bool {
        !matches!(self, Self::Secret)
    }
}

/// The source a context item is attributable to: one of the ten frozen
/// source kinds of the context architecture. Serialized as an internally
/// `kind`-tagged enum (kernel §4).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContextSource {
    /// The current turn's user input (the input itself is the content).
    UserInput,
    /// A durable session event (`ev_<ULID>`, owned by flauz-world).
    SessionEvent {
        /// The canonical event reference.
        event_id: EventRef,
    },
    /// A durable memory item (`mem_<ULID>`, owned by this crate).
    MemoryItem {
        /// The canonical memory-item reference.
        memory_item_id: MemoryItemId,
    },
    /// A durable artifact (`art_<ULID>`, owned by flauz-world).
    Artifact {
        /// The canonical artifact reference.
        artifact_id: ArtifactRef,
    },
    /// A durable resource observation (`obs_<ULID>`, owned by flauz-world).
    ResourceObservation {
        /// The canonical observation reference.
        observation_id: ObservationRef,
    },
    /// A tool result, kept as a bounded reference to the archived payload
    /// (never inlined bulk — kernel §4 / architecture §11).
    ToolResult {
        /// The bounded reference to the archived tool result.
        reference: String,
    },
    /// A document retrieved just in time, kept as a bounded reference.
    RetrievedDocument {
        /// The bounded reference to the retrieved document.
        reference: String,
    },
    /// A skill (`namespaced.key`, owned by flauz-exec) whose behavior
    /// package the item draws on.
    Skill {
        /// The namespaced skill key.
        skill_id: SkillRef,
    },
    /// The state of an execution environment (`env_<ULID>`, owned by
    /// flauz-exec).
    EnvironmentState {
        /// The canonical environment reference.
        environment_id: EnvironmentRef,
    },
    /// A collaboration event (`ev_<ULID>`, owned by flauz-world).
    CollaborationEvent {
        /// The canonical event reference.
        event_id: EventRef,
    },
}

impl ContextSource {
    /// Validates the source's reference bounds.
    pub fn validate(&self) -> Result<(), ContextError> {
        match self {
            Self::ToolResult { reference } | Self::RetrievedDocument { reference } => {
                ensure_str_bound("context source reference", reference, MAX_STATEMENT_BYTES)
            }
            _ => Ok(()),
        }
    }
}

/// Provenance for one included context item: the source it is attributable
/// to plus the authorization class that governs its eligibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextProvenance {
    /// The source this item is attributable to.
    pub source: ContextSource,
    /// The authorization class of the item.
    pub authorization: AuthorizationClass,
}

impl ContextProvenance {
    /// Builds a provenance record, validating the source bounds.
    pub fn new(
        source: ContextSource,
        authorization: AuthorizationClass,
    ) -> Result<Self, ContextError> {
        source.validate()?;
        Ok(Self {
            source,
            authorization,
        })
    }

    /// Validates the provenance record.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.source.clone(), self.authorization)?;
        Ok(())
    }

    /// Whether an item with this provenance may be included in compiled
    /// model context (authorization-aware filtering, kernel §7).
    #[must_use]
    pub const fn eligible_for_model_context(&self) -> bool {
        self.authorization.eligible_for_model_context()
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
    fn provenance_serializes_kind_tagged_with_authorization() {
        let provenance = ok(ContextProvenance::new(
            ContextSource::SessionEvent {
                event_id: ok(EventRef::parse("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
            },
            AuthorizationClass::Task,
        ));
        let serialized = ok(serde_json::to_string(&provenance));
        let source = "{\"source\":{\"kind\":\"session_event\",\
                      \"event_id\":\"ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3\"}";
        assert_eq!(serialized, format!("{source},\"authorization\":\"task\"}}"));
        let reloaded: ContextProvenance = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, provenance);
        let with_unknown = format!("{source},\"authorization\":\"task\",\"extra\":true}}");
        assert!(serde_json::from_str::<ContextProvenance>(&with_unknown).is_err());
        let dream = "{\"kind\":\"dream\",\"reference\":\"x\"}";
        assert!(serde_json::from_str::<ContextSource>(dream).is_err());
    }

    #[test]
    fn secret_authorization_is_never_model_context() {
        let secret = ok(ContextProvenance::new(
            ContextSource::ToolResult {
                reference: "flausec://archived/tool/42".to_owned(),
            },
            AuthorizationClass::Secret,
        ));
        assert!(!secret.eligible_for_model_context());
        let open = ok(ContextProvenance::new(
            ContextSource::UserInput,
            AuthorizationClass::Task,
        ));
        assert!(open.eligible_for_model_context());
        for class in [
            AuthorizationClass::Public,
            AuthorizationClass::Workspace,
            AuthorizationClass::Task,
            AuthorizationClass::Participant,
        ] {
            assert!(class.eligible_for_model_context());
        }
    }

    #[test]
    fn every_frozen_source_kind_round_trips() {
        let sources = [
            ContextSource::UserInput,
            ContextSource::SessionEvent {
                event_id: ok(EventRef::parse("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
            },
            ContextSource::MemoryItem {
                memory_item_id: ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5")),
            },
            ContextSource::Artifact {
                artifact_id: ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPH7")),
            },
            ContextSource::ResourceObservation {
                observation_id: ok(ObservationRef::parse("obs_01J8ZQ5V8K3T2B7N6X4R9DQPJ8")),
            },
            ContextSource::ToolResult {
                reference: "art_01J8ZQ5V8K3T2B7N6X4R9DQPH7/tool-output.json".to_owned(),
            },
            ContextSource::RetrievedDocument {
                reference: "doc://handbook/onboarding".to_owned(),
            },
            ContextSource::Skill {
                skill_id: ok(SkillRef::parse("flauz.research.collect")),
            },
            ContextSource::EnvironmentState {
                environment_id: ok(EnvironmentRef::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4")),
            },
            ContextSource::CollaborationEvent {
                event_id: ok(EventRef::parse("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
            },
        ];
        assert_eq!(
            sources.len(),
            10,
            "the frozen source vocabulary has ten kinds"
        );
        for source in &sources {
            let provenance = ok(ContextProvenance::new(
                source.clone(),
                AuthorizationClass::Task,
            ));
            let serialized = ok(serde_json::to_string(&provenance));
            let reloaded: ContextProvenance = ok(serde_json::from_str(&serialized));
            assert_eq!(reloaded, provenance, "source {source:?} must round-trip");
        }
    }
}
