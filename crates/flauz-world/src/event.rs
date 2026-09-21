//! Events and the v1 event envelope (kernel §3, §5, §6).
//!
//! Events are append-only and immutable: `event_id` is unique, `seq` is
//! strictly increasing within a stream (workspace / task / session), and
//! events are never mutated or deleted — corrections are new events. The
//! [`EventEnvelope`] shape is frozen field-for-field by kernel §5.
//!
//! Task-identity events (`task.model_changed`, `task.context_reset`, ...) are
//! recorded on the task stream; they never change the task ID and never fork
//! the logical task.

use serde::{Deserialize, Serialize};

use crate::ids::EventId;
use crate::refs::EntityRef;
use crate::refs::{ActorRef, StreamRef};
use crate::time::Timestamp;
use crate::value::Payload;
use crate::{ContractVersion, MAX_NAME_BYTES, MAX_REFERENCE_BYTES, WorldError, ensure_str_bound};

/// The registered event-type vocabulary of this crate (kernel §5). Other
/// crates register their own `<entity>.<verb_past>` types.
pub mod event_types {
    /// A workspace was created.
    pub const WORKSPACE_CREATED: &str = "workspace.created";
    /// A session was created.
    pub const SESSION_CREATED: &str = "session.created";
    /// A task was created.
    pub const TASK_CREATED: &str = "task.created";
    /// The model executing a task changed. The task identity is preserved.
    pub const TASK_MODEL_CHANGED: &str = "task.model_changed";
    /// The task context was reset or compacted. The task identity is
    /// preserved.
    pub const TASK_CONTEXT_RESET: &str = "task.context_reset";
    /// The environment executing a task changed. The task identity is
    /// preserved.
    pub const TASK_ENVIRONMENT_CHANGED: &str = "task.environment_changed";
    /// The task was handed off to another agent. The task identity is
    /// preserved.
    pub const TASK_AGENT_HANDOFF: &str = "task.agent_handoff";
    /// The session executing a task restarted. The task identity is
    /// preserved.
    pub const TASK_SESSION_RESTARTED: &str = "task.session_restarted";
    /// An artifact was produced.
    pub const ARTIFACT_PRODUCED: &str = "artifact.produced";
    /// A resource was created.
    pub const RESOURCE_CREATED: &str = "resource.created";
    /// The access surfaces of a resource changed. The resource identity is
    /// preserved.
    pub const RESOURCE_SURFACES_CHANGED: &str = "resource.surfaces_changed";
    /// A resource was observed through an access surface.
    pub const RESOURCE_OBSERVED: &str = "resource.observed";
    /// A claim was made.
    pub const CLAIM_MADE: &str = "claim.made";
    /// A claim was verified, producing evidence.
    pub const EVIDENCE_VERIFIED: &str = "evidence.verified";
    /// A lease was granted.
    pub const LEASE_GRANTED: &str = "lease.granted";
    /// A lease was released.
    pub const LEASE_RELEASED: &str = "lease.released";
    /// A procedure was created.
    pub const PROCEDURE_CREATED: &str = "procedure.created";
    /// A new procedure version was recorded.
    pub const PROCEDURE_VERSION_ADDED: &str = "procedure.version_added";
}

/// A validated `<entity>.<verb_past>` event type name (kernel §5): exactly
/// one dot, both segments `[a-z][a-z0-9_]*`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventTypeName(String);

impl EventTypeName {
    /// Parses and validates an event type name.
    pub fn parse(value: &str) -> Result<Self, WorldError> {
        let Some((entity, verb)) = value.split_once('.') else {
            return Err(WorldError::invalid(format!(
                "event type {value:?} must have the form `<entity>.<verb_past>`"
            )));
        };
        if verb.contains('.') {
            return Err(WorldError::invalid(format!(
                "event type {value:?} must contain exactly one `.`"
            )));
        }
        if !valid_segment(entity) || !valid_segment(verb) {
            return Err(WorldError::invalid(format!(
                "event type {value:?} segments must be `[a-z][a-z0-9_]*`"
            )));
        }
        ensure_str_bound("event type", value, MAX_NAME_BYTES)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the event type name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the event type name.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

fn valid_segment(segment: &str) -> bool {
    let mut characters = segment.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}

impl std::fmt::Display for EventTypeName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::str::FromStr for EventTypeName {
    type Err = WorldError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for EventTypeName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for EventTypeName {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// The semantic content of an event: what happened, when, who did it, what
/// it is about. The store assigns identity and sequencing when appending an
/// [`EventEnvelope`] to a stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The `<entity>.<verb_past>` event type.
    pub event_type: EventTypeName,
    /// When the event happened (caller-supplied).
    pub ts: Timestamp,
    /// The actor that caused the event.
    pub actor: ActorRef,
    /// The event that caused this one, if any.
    pub causation_id: Option<EventId>,
    /// An opaque correlation identifier linking related events, if any.
    pub correlation_id: Option<String>,
    /// The entity this event is about.
    pub subject: EntityRef,
    /// The canonical event payload.
    pub payload: Payload,
}

impl Event {
    /// Builds a new event, validating the event type name.
    pub fn new(
        event_type: &str,
        ts: Timestamp,
        actor: ActorRef,
        subject: EntityRef,
        payload: Payload,
    ) -> Result<Self, WorldError> {
        Ok(Self {
            v: ContractVersion,
            event_type: EventTypeName::parse(event_type)?,
            ts,
            actor,
            causation_id: None,
            correlation_id: None,
            subject,
            payload,
        })
    }

    /// Sets the causation event.
    #[must_use]
    pub fn with_causation(mut self, causation_id: EventId) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    /// Sets the correlation identifier.
    #[must_use]
    pub fn with_correlation(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_owned());
        self
    }

    /// Validates the event.
    pub fn validate(&self) -> Result<(), WorldError> {
        self.event_type.validate()?;
        if let Some(correlation_id) = &self.correlation_id {
            ensure_str_bound("correlation id", correlation_id, MAX_REFERENCE_BYTES)?;
        }
        self.actor.validate()?;
        self.subject.validate()?;
        self.payload.validate()?;
        Ok(())
    }
}

/// The frozen kind tag of the v1 event envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventEnvelopeKind {
    /// The `flauz.event` envelope kind.
    #[serde(rename = "flauz.event")]
    FlauzEvent,
}

/// The v1 event envelope (kernel §5), frozen field-for-field:
///
/// ```json
/// {
///   "v": 1,
///   "kind": "flauz.event",
///   "event_id": "ev_01J8...",
///   "seq": 42,
///   "stream": { "kind": "workspace|task|session", "id": "ws_01J8..." },
///   "event_type": "task.created",
///   "ts": "2026-09-21T13:45:00Z",
///   "actor": { "kind": "agent", "id": "agent_01J8..." },
///   "causation_id": null,
///   "correlation_id": null,
///   "subject": { "entity_kind": "task", "id": "task_01J8..." },
///   "payload": { }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventEnvelope {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The frozen envelope kind tag, `"flauz.event"`.
    pub kind: EventEnvelopeKind,
    /// Canonical event ID (`ev_<ULID>`), unique across all streams.
    pub event_id: EventId,
    /// The strictly increasing per-stream sequence number.
    pub seq: u64,
    /// The stream this event belongs to (workspace / task / session).
    pub stream: StreamRef,
    /// The `<entity>.<verb_past>` event type.
    pub event_type: EventTypeName,
    /// When the event happened (caller-supplied).
    pub ts: Timestamp,
    /// The actor that caused the event.
    pub actor: ActorRef,
    /// The event that caused this one, if any.
    pub causation_id: Option<EventId>,
    /// An opaque correlation identifier, if any.
    pub correlation_id: Option<String>,
    /// The entity this event is about.
    pub subject: EntityRef,
    /// The canonical event payload.
    pub payload: Payload,
}

impl EventEnvelope {
    /// Assembles an envelope from an event plus its assigned identity,
    /// sequence and stream.
    #[must_use]
    pub fn new(event_id: EventId, seq: u64, stream: StreamRef, event: Event) -> Self {
        let Event {
            v,
            event_type,
            ts,
            actor,
            causation_id,
            correlation_id,
            subject,
            payload,
        } = event;
        Self {
            v,
            kind: EventEnvelopeKind::FlauzEvent,
            event_id,
            seq,
            stream,
            event_type,
            ts,
            actor,
            causation_id,
            correlation_id,
            subject,
            payload,
        }
    }

    /// Validates the envelope.
    pub fn validate(&self) -> Result<(), WorldError> {
        if self.seq == 0 {
            return Err(WorldError::invalid("event seq must be at least 1"));
        }
        self.event_type.validate()?;
        if let Some(correlation_id) = &self.correlation_id {
            ensure_str_bound("correlation id", correlation_id, MAX_REFERENCE_BYTES)?;
        }
        self.actor.validate()?;
        self.subject.validate()?;
        self.payload.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_grammar_is_enforced() {
        assert!(EventTypeName::parse("task.created").is_ok());
        assert!(EventTypeName::parse("task.model_changed").is_ok());
        assert!(EventTypeName::parse("evidence.verified").is_ok());
        assert!(EventTypeName::parse("flauz.event.fired").is_err());
        assert!(EventTypeName::parse("task").is_err());
        assert!(EventTypeName::parse("task.").is_err());
        assert!(EventTypeName::parse(".created").is_err());
        assert!(EventTypeName::parse("Task.created").is_err());
        assert!(EventTypeName::parse("task.Created").is_err());
        assert!(serde_json::from_str::<EventTypeName>("\"task.created\"").is_ok());
        assert!(serde_json::from_str::<EventTypeName>("\"task created\"").is_err());
    }

    #[test]
    fn registered_event_types_parse() {
        for name in [
            event_types::WORKSPACE_CREATED,
            event_types::SESSION_CREATED,
            event_types::TASK_CREATED,
            event_types::TASK_MODEL_CHANGED,
            event_types::TASK_CONTEXT_RESET,
            event_types::TASK_ENVIRONMENT_CHANGED,
            event_types::TASK_AGENT_HANDOFF,
            event_types::TASK_SESSION_RESTARTED,
            event_types::ARTIFACT_PRODUCED,
            event_types::RESOURCE_CREATED,
            event_types::RESOURCE_SURFACES_CHANGED,
            event_types::RESOURCE_OBSERVED,
            event_types::CLAIM_MADE,
            event_types::EVIDENCE_VERIFIED,
            event_types::LEASE_GRANTED,
            event_types::LEASE_RELEASED,
            event_types::PROCEDURE_CREATED,
            event_types::PROCEDURE_VERSION_ADDED,
        ] {
            assert!(EventTypeName::parse(name).is_ok(), "{name} must parse");
        }
    }
}
