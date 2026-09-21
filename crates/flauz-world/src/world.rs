//! Core world entities: [`Workspace`], [`Session`], [`Task`] and
//! [`Artifact`].
//!
//! A [`Task`] is identified solely by its canonical `task_` ID (kernel §6):
//! model switches, context resets, environment switches, agent handoffs and
//! session restarts are events on the task stream, never identity changes.
//! A [`Session`] references at most one task and may expire freely. An
//! [`Artifact`] is a produced work product; byte payloads are kept behind
//! bounded references rather than inlined bulk.

use serde::{Deserialize, Serialize};

use crate::ids::{ArtifactId, SessionId, TaskId, WorkspaceId};
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContractVersion, MAX_NAME_BYTES, MAX_REFERENCE_BYTES, MAX_STATEMENT_BYTES, MAX_TEXT_BYTES,
    WorldError, ensure_non_empty, ensure_str_bound,
};

/// The collaboration root (kernel: FLAUZ-SOURCE-OF-TRUTH).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical workspace ID (`ws_<ULID>`).
    pub id: WorkspaceId,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// Human-readable workspace name.
    pub name: String,
    /// The actor that created the workspace.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Workspace {
    /// Builds a new workspace at version 1.
    pub fn new(
        id: WorkspaceId,
        name: &str,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, WorldError> {
        ensure_non_empty("workspace name", name)?;
        ensure_str_bound("workspace name", name, MAX_NAME_BYTES)?;
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            name: name.to_owned(),
            created_by,
            created_at,
        })
    }

    /// Validates the workspace against canonical bounds and version rules.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(
            self.id.clone(),
            &self.name,
            self.created_by.clone(),
            self.created_at,
        )?;
        self.validate_version()
    }

    fn validate_version(&self) -> Result<(), WorldError> {
        if self.version == 0 {
            Err(WorldError::invalid("workspace version must be at least 1"))
        } else {
            Ok(())
        }
    }
}

/// A session: an attachment context that references at most one task. A
/// session is not the task's identity; sessions may be created and may
/// expire freely (kernel §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical session ID (`sess_<ULID>`).
    pub id: SessionId,
    /// Durable entity version, starting at 1.
    pub version: u64,
    /// The workspace this session belongs to.
    pub workspace_id: WorkspaceId,
    /// The task this session references, if any (at most one).
    pub task_id: Option<TaskId>,
    /// The actor that created the session.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Session {
    /// Builds a new session at version 1.
    pub fn new(
        id: SessionId,
        workspace_id: WorkspaceId,
        task_id: Option<TaskId>,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, WorldError> {
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            workspace_id,
            task_id,
            created_by,
            created_at,
        })
    }

    /// Validates the session.
    pub fn validate(&self) -> Result<(), WorldError> {
        if self.version == 0 {
            return Err(WorldError::invalid("session version must be at least 1"));
        }
        Ok(())
    }
}

/// A unit of durable work. The task is identified solely by its canonical
/// ID; everything else about it (objective, artifacts, evidence, resources,
/// environment bindings, agent assignments, approvals) hangs off the task
/// via its event stream and snapshots and is fully reconstructible from
/// them without any model context (kernel §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical task ID (`task_<ULID>`) — the only task identity.
    pub id: TaskId,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// The workspace this task belongs to.
    pub workspace_id: WorkspaceId,
    /// The task objective.
    pub objective: String,
    /// The actor that created the task.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Task {
    /// Builds a new task at version 1.
    pub fn new(
        id: TaskId,
        workspace_id: WorkspaceId,
        objective: &str,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, WorldError> {
        ensure_non_empty("task objective", objective)?;
        ensure_str_bound("task objective", objective, MAX_STATEMENT_BYTES)?;
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            workspace_id,
            objective: objective.to_owned(),
            created_by,
            created_at,
        })
    }

    /// Validates the task.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(
            self.id.clone(),
            self.workspace_id.clone(),
            &self.objective,
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(WorldError::invalid("task version must be at least 1"));
        }
        Ok(())
    }
}

/// The content of an artifact: either bounded inline text or a bounded
/// reference. Bulk bytes are never inlined (kernel §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactContent {
    /// Bounded inline text content (base64 for binary payloads).
    Text {
        /// The text content.
        text: String,
    },
    /// A bounded reference to content stored elsewhere (an artifact/event
    /// reference, URI or digest).
    Reference {
        /// The reference string.
        reference: String,
    },
}

impl ArtifactContent {
    /// Validates the content bounds.
    pub fn validate(&self) -> Result<(), WorldError> {
        match self {
            Self::Text { text } => {
                ensure_non_empty("artifact text", text)?;
                ensure_str_bound("artifact text", text, MAX_TEXT_BYTES)
            }
            Self::Reference { reference } => {
                ensure_non_empty("artifact reference", reference)?;
                ensure_str_bound("artifact reference", reference, MAX_REFERENCE_BYTES)
            }
        }
    }
}

impl Serialize for ArtifactContent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            Self::Text { text } => {
                map.serialize_entry("kind", "text")?;
                map.serialize_entry("text", text)?;
            }
            Self::Reference { reference } => {
                map.serialize_entry("kind", "reference")?;
                map.serialize_entry("reference", reference)?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for ArtifactContent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ArtifactContentVisitor;

        impl<'de> serde::de::Visitor<'de> for ArtifactContentVisitor {
            type Value = ArtifactContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("artifact content tagged with `kind`")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                use serde::de::Error;

                let mut kind: Option<String> = None;
                let mut text: Option<String> = None;
                let mut reference: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "kind" => {
                            if kind.replace(map.next_value()?).is_some() {
                                return Err(A::Error::duplicate_field("kind"));
                            }
                        }
                        "text" => {
                            if text.replace(map.next_value()?).is_some() {
                                return Err(A::Error::duplicate_field("text"));
                            }
                        }
                        "reference" => {
                            if reference.replace(map.next_value()?).is_some() {
                                return Err(A::Error::duplicate_field("reference"));
                            }
                        }
                        other => {
                            return Err(A::Error::unknown_field(
                                other,
                                &["kind", "text", "reference"],
                            ));
                        }
                    }
                }

                match kind.as_deref() {
                    Some("text") => {
                        let text = text.ok_or_else(|| A::Error::missing_field("text"))?;
                        if reference.is_some() {
                            return Err(A::Error::custom(
                                "artifact text content must not carry a reference field",
                            ));
                        }
                        Ok(ArtifactContent::Text { text })
                    }
                    Some("reference") => {
                        let reference =
                            reference.ok_or_else(|| A::Error::missing_field("reference"))?;
                        if text.is_some() {
                            return Err(A::Error::custom(
                                "artifact reference content must not carry a text field",
                            ));
                        }
                        Ok(ArtifactContent::Reference { reference })
                    }
                    Some(other) => Err(A::Error::unknown_variant(other, &["text", "reference"])),
                    None => Err(A::Error::missing_field("kind")),
                }
            }
        }

        deserializer.deserialize_map(ArtifactContentVisitor)
    }
}

/// A produced work product attached to a task. An artifact is a durable
/// output, not a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical artifact ID (`art_<ULID>`).
    pub id: ArtifactId,
    /// Durable entity version, starting at 1.
    pub version: u64,
    /// The task that produced this artifact.
    pub task_id: TaskId,
    /// Human-readable artifact title.
    pub title: String,
    /// The artifact content: bounded inline text or a bounded reference.
    pub content: ArtifactContent,
    /// The actor that produced the artifact.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Artifact {
    /// Builds a new artifact at version 1.
    pub fn new(
        id: ArtifactId,
        task_id: TaskId,
        title: &str,
        content: ArtifactContent,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, WorldError> {
        ensure_non_empty("artifact title", title)?;
        ensure_str_bound("artifact title", title, MAX_NAME_BYTES)?;
        content.validate()?;
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            task_id,
            title: title.to_owned(),
            content,
            created_by,
            created_at,
        })
    }

    /// Validates the artifact.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(
            self.id.clone(),
            self.task_id.clone(),
            &self.title,
            self.content.clone(),
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(WorldError::invalid("artifact version must be at least 1"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refs::ActorRef;
    use crate::time::Timestamp;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn artifact_content_is_strict_and_tagged() {
        let content = ArtifactContent::Text {
            text: "release notes".to_owned(),
        };
        let serialized = ok(serde_json::to_string(&content));
        assert_eq!(serialized, "{\"kind\":\"text\",\"text\":\"release notes\"}");
        let parsed: ArtifactContent = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, content);

        assert!(
            serde_json::from_str::<ArtifactContent>(
                "{\"kind\":\"text\",\"text\":\"x\",\"extra\":1}"
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<ArtifactContent>(
                "{\"kind\":\"text\",\"text\":\"x\",\"reference\":\"y\"}"
            )
            .is_err()
        );
        assert!(serde_json::from_str::<ArtifactContent>("{\"kind\":\"text\"}").is_err());
        assert!(serde_json::from_str::<ArtifactContent>("{\"kind\":\"blob\"}").is_err());
    }

    #[test]
    fn task_rejects_empty_and_oversized_objectives() {
        let id = TaskId::generate();
        let actor = ok(ActorRef::user("alice"));
        let ts = ok(Timestamp::parse("2026-09-21T13:45:00Z"));
        assert!(Task::new(id.clone(), WorkspaceId::generate(), "", actor.clone(), ts).is_err());
        let oversized = "x".repeat(MAX_STATEMENT_BYTES + 1);
        assert!(Task::new(id, WorkspaceId::generate(), &oversized, actor, ts).is_err());
    }
}
