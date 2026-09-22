//! The context snapshot: the durable, serializable projection of what a
//! model was told at one compilation (kernel §1 worker C scope;
//! architecture §2 — a session retains bounded durable history, the active
//! model context is a deliberate *projection* of that history and other
//! state).
//!
//! A snapshot is a **projection, not a transcript**: every
//! [`ContextItem`] carries [`ContextProvenance`] attributing it to a source,
//! and references durable entities by their canonical opaque ID strings
//! (validated local newtypes — never re-definitions of the owning crates'
//! entity types). Because items carry only bounded inline text or bounded
//! references, a snapshot is reconstructible from its references: the
//! durable-reference list ([`ContextSnapshot::durable_references`]) names
//! every canonical entity the projection depends on.
//!
//! Snapshots are immutable once compiled: a new compilation produces a new
//! snapshot with its own `ctxsnap_` ID. Durable mutations (which would
//! carry version bumps per kernel §3) belong to the memory items and model
//! profiles referenced, not to the snapshot itself; the snapshot's
//! `version` is therefore always `1` and enforced as such.

use serde::{Deserialize, Serialize};

use crate::ids::{ContextSnapshotId, ModelRef, SessionRef, TaskRef};
use crate::memory::MemoryTier;
use crate::provenance::ContextProvenance;
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContextError, ContextVersion, MAX_CONTEXT_ITEMS, MAX_ITEM_CONTENT_BYTES,
    MAX_STATEMENT_BYTES, ensure_non_empty, ensure_str_bound,
};

/// The content of one context item: either bounded inline text (for
/// projection-local content such as the current turn) or a bounded reference
/// to a durable payload. Bulk bytes are never inlined (kernel §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextItemContent {
    /// Bounded inline text content.
    Text {
        /// The text content.
        text: String,
    },
    /// A bounded reference to a durable payload (an artifact, event,
    /// archived tool result, retrieved document, ...).
    Reference {
        /// The reference string.
        reference: String,
    },
}

impl ContextItemContent {
    /// Validates the content bounds.
    pub fn validate(&self) -> Result<(), ContextError> {
        match self {
            Self::Text { text } => {
                ensure_non_empty("context item text", text)?;
                ensure_str_bound("context item text", text, MAX_ITEM_CONTENT_BYTES)
            }
            Self::Reference { reference } => {
                ensure_non_empty("context item reference", reference)?;
                ensure_str_bound("context item reference", reference, MAX_STATEMENT_BYTES)
            }
        }
    }
}

impl Serialize for ContextItemContent {
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

impl<'de> Deserialize<'de> for ContextItemContent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ContextItemContentVisitor;

        impl<'de> serde::de::Visitor<'de> for ContextItemContentVisitor {
            type Value = ContextItemContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("context item content tagged with `kind`")
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
                                "context item text content must not carry a reference field",
                            ));
                        }
                        Ok(ContextItemContent::Text { text })
                    }
                    Some("reference") => {
                        let reference =
                            reference.ok_or_else(|| A::Error::missing_field("reference"))?;
                        if text.is_some() {
                            return Err(A::Error::custom(
                                "context item reference content must not carry a text field",
                            ));
                        }
                        Ok(ContextItemContent::Reference { reference })
                    }
                    Some(other) => Err(A::Error::unknown_variant(other, &["text", "reference"])),
                    None => Err(A::Error::missing_field("kind")),
                }
            }
        }

        deserializer.deserialize_map(ContextItemContentVisitor)
    }
}

/// One included context item: bounded content plus its tier plus the
/// provenance that attributes it to a source and carries its authorization
/// class. Provenance is structural — an item cannot exist without it
/// (`context_provenance_covers_every_item`, kernel §9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextItem {
    /// Bounded inline text or a bounded durable reference.
    pub content: ContextItemContent,
    /// The tier the item was included from.
    pub tier: MemoryTier,
    /// Provenance: the source this item is attributable to, plus the
    /// authorization class governing its eligibility as model context.
    pub provenance: ContextProvenance,
}

impl ContextItem {
    /// Builds a context item, validating content, provenance and bounds.
    pub fn new(
        content: ContextItemContent,
        tier: MemoryTier,
        provenance: ContextProvenance,
    ) -> Result<Self, ContextError> {
        content.validate()?;
        provenance.validate()?;
        Ok(Self {
            content,
            tier,
            provenance,
        })
    }

    /// Validates the item.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.content.clone(), self.tier, self.provenance.clone())?;
        Ok(())
    }
}

/// A durable, serializable projection of the model context for one task at
/// one compilation, targeting one model profile.
///
/// The snapshot references durable entities by canonical ID — the task, the
/// optional session the compilation happened in, and the model it targeted —
/// and every item carries provenance. It never re-defines those foreign
/// entities; it is reconstructible from its references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextSnapshot {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// Canonical snapshot ID (`ctxsnap_<ULID>`).
    pub id: ContextSnapshotId,
    /// Durable entity version. Snapshots are immutable once compiled, so
    /// the version is always 1 (kernel §3 version rules apply to the
    /// mutable entities a snapshot references).
    pub version: u64,
    /// The task whose durable state this context projects (identified
    /// solely by its canonical `task_` ID — kernel §6).
    pub task_id: TaskRef,
    /// The session the compilation happened in, if any. A session is never
    /// the context (Session ≠ Context).
    pub session_id: Option<SessionRef>,
    /// The model the snapshot was compiled for (execution state, never
    /// task identity — kernel §6).
    pub model_id: ModelRef,
    /// The actor that compiled the snapshot (the Context Engine acting for
    /// a participant).
    pub compiled_by: ActorRef,
    /// Compilation timestamp (caller-supplied).
    pub compiled_at: Timestamp,
    /// The included items, each with provenance. Bounded.
    pub items: Vec<ContextItem>,
}

impl ContextSnapshot {
    /// Builds a new snapshot at version 1, validating every item and the
    /// item bound.
    pub fn new(
        id: ContextSnapshotId,
        task_id: TaskRef,
        session_id: Option<SessionRef>,
        model_id: ModelRef,
        compiled_by: ActorRef,
        compiled_at: Timestamp,
        items: Vec<ContextItem>,
    ) -> Result<Self, ContextError> {
        if items.len() > MAX_CONTEXT_ITEMS {
            return Err(ContextError::invalid(format!(
                "context snapshot exceeds {MAX_CONTEXT_ITEMS} items"
            )));
        }
        for item in &items {
            item.validate()?;
        }
        Ok(Self {
            v: ContextVersion,
            id,
            version: 1,
            task_id,
            session_id,
            model_id,
            compiled_by,
            compiled_at,
            items,
        })
    }

    /// Validates the snapshot.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.id.clone(),
            self.task_id.clone(),
            self.session_id.clone(),
            self.model_id.clone(),
            self.compiled_by.clone(),
            self.compiled_at,
            self.items.clone(),
        )?;
        if self.version != 1 {
            return Err(ContextError::invalid(
                "context snapshots are immutable and carry version 1",
            ));
        }
        Ok(())
    }

    /// Lists every canonical entity reference the projection depends on:
    /// the task, the session, the model, and every canonical ID reachable
    /// through item provenance. A fresh context can be reconstructed from
    /// these references plus the durable state they name (the compilation
    /// engine itself is F6; this is the frozen representation).
    #[must_use]
    pub fn durable_references(&self) -> Vec<String> {
        let mut references = vec![
            self.task_id.as_str().to_owned(),
            self.model_id.as_str().to_owned(),
        ];
        if let Some(session) = &self.session_id {
            references.push(session.as_str().to_owned());
        }
        for item in &self.items {
            match &item.provenance.source {
                crate::provenance::ContextSource::SessionEvent { event_id }
                | crate::provenance::ContextSource::CollaborationEvent { event_id } => {
                    references.push(event_id.as_str().to_owned());
                }
                crate::provenance::ContextSource::MemoryItem { memory_item_id } => {
                    references.push(memory_item_id.as_str().to_owned());
                }
                crate::provenance::ContextSource::Artifact { artifact_id } => {
                    references.push(artifact_id.as_str().to_owned());
                }
                crate::provenance::ContextSource::ResourceObservation { observation_id } => {
                    references.push(observation_id.as_str().to_owned());
                }
                crate::provenance::ContextSource::EnvironmentState { environment_id } => {
                    references.push(environment_id.as_str().to_owned());
                }
                crate::provenance::ContextSource::UserInput
                | crate::provenance::ContextSource::ToolResult { .. }
                | crate::provenance::ContextSource::RetrievedDocument { .. }
                | crate::provenance::ContextSource::Skill { .. } => {}
            }
        }
        references
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::MemoryItemId;
    use crate::provenance::{AuthorizationClass, ContextSource};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn sample_item(reference: &str) -> ContextItem {
        ok(ContextItem::new(
            ContextItemContent::Reference {
                reference: reference.to_owned(),
            },
            MemoryTier::Warm,
            ok(ContextProvenance::new(
                ContextSource::MemoryItem {
                    memory_item_id: ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5")),
                },
                AuthorizationClass::Task,
            )),
        ))
    }

    #[test]
    fn snapshot_item_content_is_strict_and_tagged() {
        let content = ContextItemContent::Text {
            text: "reconcile the ticket counts".to_owned(),
        };
        let serialized = ok(serde_json::to_string(&content));
        assert_eq!(
            serialized,
            "{\"kind\":\"text\",\"text\":\"reconcile the ticket counts\"}"
        );
        let parsed: ContextItemContent = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, content);
        assert!(
            serde_json::from_str::<ContextItemContent>(
                "{\"kind\":\"text\",\"text\":\"x\",\"reference\":\"y\"}"
            )
            .is_err()
        );
        assert!(serde_json::from_str::<ContextItemContent>("{\"kind\":\"text\"}").is_err());
        assert!(serde_json::from_str::<ContextItemContent>("{\"kind\":\"video\"}").is_err());
    }

    #[test]
    fn snapshot_round_trips_and_lists_durable_references() {
        let snapshot = ok(ContextSnapshot::new(
            ok(ContextSnapshotId::parse("ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5")),
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            Some(ok(SessionRef::parse("sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6"))),
            ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE")),
            ok(ActorRef::system("flauz-context-engine")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
            vec![sample_item("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5/decision")],
        ));
        let serialized = ok(serde_json::to_string(&snapshot));
        let reloaded: ContextSnapshot = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, snapshot);
        assert_eq!(
            snapshot.durable_references(),
            vec![
                "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1".to_owned(),
                "model_01J8ZQ5V8K3T2B7N6X4R9DQPRE".to_owned(),
                "sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6".to_owned(),
                "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5".to_owned(),
            ]
        );
    }

    #[test]
    fn snapshot_rejects_oversized_item_lists_and_mutable_versions() {
        let base = ok(ContextSnapshot::new(
            ContextSnapshotId::generate(),
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            None,
            ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE")),
            ActorRef::system("flauz-context-engine"),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
            Vec::new(),
        ));
        let mut mutated = base.clone();
        mutated.version = 2;
        assert!(mutated.validate().is_err());

        let oversized = vec![sample_item("r"); MAX_CONTEXT_ITEMS + 1];
        assert!(
            ContextSnapshot::new(
                ContextSnapshotId::generate(),
                base.task_id.clone(),
                None,
                base.model_id.clone(),
                base.compiled_by.clone(),
                base.compiled_at,
                oversized,
            )
            .is_err()
        );
    }
}
