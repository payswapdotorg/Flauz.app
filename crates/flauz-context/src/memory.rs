//! Tiered memory items (kernel §2 registry: `mem_<ULID>`; context-harness
//! architecture §2 memory tiers).
//!
//! A [`MemoryItem`] is a durable, bounded piece of rememberable state. Its
//! tier records how it participates in context compilation:
//!
//! - **HOT** — current turn, current tool result, active plan, current
//!   errors, current environment state. Eligible by default.
//! - **WARM** — recent conversation, task summary, decisions, unresolved
//!   questions, important discoveries, recent artifacts. Selectively
//!   included.
//! - **COLD** — complete history, archived tool results, workspace
//!   knowledge, documents, old executions, reusable procedures. Retrieved
//!   just in time.
//!
//! Memory ≠ Context (the frozen invariants): a memory item is durable state
//! that a snapshot may *reference*; it is never itself "the context".

use serde::{Deserialize, Serialize};

use crate::ids::{MemoryItemId, TaskRef};
use crate::provenance::AuthorizationClass;
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContextError, ContextVersion, MAX_MEMORY_CONTENT_BYTES, MAX_STATEMENT_BYTES,
    ensure_non_empty, ensure_str_bound,
};

/// How a memory item participates in context compilation (context-harness
/// architecture §2 memory tiers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryTier {
    /// Current turn, current tool result, active plan, current errors,
    /// current environment state. Eligible by default.
    Hot,
    /// Recent conversation, task summary, decisions, unresolved questions,
    /// important discoveries, recent artifacts. Selectively included.
    Warm,
    /// Complete history, archived tool results, workspace knowledge,
    /// documents, old executions, reusable procedures. Retrieved just in
    /// time.
    Cold,
}

impl MemoryTier {
    /// Whether items in this tier are eligible for inclusion in a freshly
    /// reconstructed context without just-in-time retrieval. HOT is eligible
    /// by default and WARM is selectively included; COLD is retrieved just
    /// in time and therefore never auto-included by a fresh reset.
    #[must_use]
    pub const fn eligible_on_reset(self) -> bool {
        matches!(self, Self::Hot | Self::Warm)
    }
}

/// The content of a memory item: either bounded inline text or a bounded
/// reference to durable storage (an artifact, event or other canonical
/// reference). Bulk bytes are never inlined (kernel §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryContent {
    /// Bounded inline text content.
    Text {
        /// The text content.
        text: String,
    },
    /// A bounded reference to content stored elsewhere.
    Reference {
        /// The reference string (a canonical entity reference or digest).
        reference: String,
    },
}

impl MemoryContent {
    /// Validates the content bounds.
    pub fn validate(&self) -> Result<(), ContextError> {
        match self {
            Self::Text { text } => {
                ensure_non_empty("memory text", text)?;
                ensure_str_bound("memory text", text, MAX_MEMORY_CONTENT_BYTES)
            }
            Self::Reference { reference } => {
                ensure_non_empty("memory reference", reference)?;
                ensure_str_bound("memory reference", reference, MAX_STATEMENT_BYTES)
            }
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Text { .. } => "text",
            Self::Reference { .. } => "reference",
        }
    }
}

impl Serialize for MemoryContent {
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

impl<'de> Deserialize<'de> for MemoryContent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MemoryContentVisitor;

        impl<'de> serde::de::Visitor<'de> for MemoryContentVisitor {
            type Value = MemoryContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("memory content tagged with `kind`")
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
                                "memory text content must not carry a reference field",
                            ));
                        }
                        Ok(MemoryContent::Text { text })
                    }
                    Some("reference") => {
                        let reference =
                            reference.ok_or_else(|| A::Error::missing_field("reference"))?;
                        if text.is_some() {
                            return Err(A::Error::custom(
                                "memory reference content must not carry a text field",
                            ));
                        }
                        Ok(MemoryContent::Reference { reference })
                    }
                    Some(other) => Err(A::Error::unknown_variant(other, &["text", "reference"])),
                    None => Err(A::Error::missing_field("kind")),
                }
            }
        }

        deserializer.deserialize_map(MemoryContentVisitor)
    }
}

/// A durable, tiered piece of rememberable state (kernel §2 registry
/// `mem_<ULID>`). Memory is durable state that context snapshots reference;
/// it is never itself "the context" (Context ≠ Memory). Each item carries
/// the authorization class that governs its eligibility as model context
/// (kernel §7): a `secret`-class memory item is authorized for retrieval by
/// its owner but never becomes model context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryItem {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// Canonical memory-item ID (`mem_<ULID>`).
    pub id: MemoryItemId,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// The task this memory belongs to, or `None` for workspace-scoped
    /// memory (for example workspace knowledge or documents).
    pub task_id: Option<TaskRef>,
    /// The memory tier (HOT / WARM / COLD).
    pub tier: MemoryTier,
    /// The authorization class governing the item's eligibility as model
    /// context (kernel §7).
    pub authorization: AuthorizationClass,
    /// The memory content: bounded inline text or a bounded reference.
    pub content: MemoryContent,
    /// The actor that recorded the memory.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied; contract code never reads the
    /// wall clock).
    pub created_at: Timestamp,
}

impl MemoryItem {
    /// Builds a new memory item at version 1.
    pub fn new(
        id: MemoryItemId,
        task_id: Option<TaskRef>,
        tier: MemoryTier,
        authorization: AuthorizationClass,
        content: MemoryContent,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ContextError> {
        content.validate()?;
        Ok(Self {
            v: ContextVersion,
            id,
            version: 1,
            task_id,
            tier,
            authorization,
            content,
            created_by,
            created_at,
        })
    }

    /// Validates the memory item against canonical bounds and version
    /// rules.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.id.clone(),
            self.task_id.clone(),
            self.tier,
            self.authorization,
            self.content.clone(),
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(ContextError::invalid(
                "memory item version must be at least 1",
            ));
        }
        Ok(())
    }

    /// Whether this memory item may be included in compiled model context
    /// (authorization-aware filtering, kernel §7).
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
    fn memory_content_is_strict_and_tagged() {
        let content = MemoryContent::Text {
            text: "the dashboard shows 3 open tickets".to_owned(),
        };
        let serialized = ok(serde_json::to_string(&content));
        assert_eq!(
            serialized,
            "{\"kind\":\"text\",\"text\":\"the dashboard shows 3 open tickets\"}"
        );
        let parsed: MemoryContent = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, content);
        assert_eq!(content.kind(), "text");

        assert!(
            serde_json::from_str::<MemoryContent>(
                "{\"kind\":\"text\",\"text\":\"x\",\"extra\":1}"
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<MemoryContent>(
                "{\"kind\":\"text\",\"text\":\"x\",\"reference\":\"y\"}"
            )
            .is_err()
        );
        assert!(serde_json::from_str::<MemoryContent>("{\"kind\":\"text\"}").is_err());
        assert!(serde_json::from_str::<MemoryContent>("{\"kind\":\"blob\"}").is_err());
    }

    #[test]
    fn memory_item_round_trips_and_rejects_empty_content() {
        let item = ok(MemoryItem::new(
            ok(MemoryItemId::parse("mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5")),
            Some(ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"))),
            MemoryTier::Warm,
            crate::provenance::AuthorizationClass::Task,
            MemoryContent::Text {
                text: "decision: use the CRM export as source of truth".to_owned(),
            },
            ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        ));
        let serialized = ok(serde_json::to_string(&item));
        let reloaded: MemoryItem = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, item);
        assert_eq!(item.version, 1);
        assert!(item.eligible_for_model_context());

        let empty = MemoryItem::new(
            item.id.clone(),
            None,
            MemoryTier::Cold,
            crate::provenance::AuthorizationClass::Workspace,
            MemoryContent::Text {
                text: String::new(),
            },
            item.created_by.clone(),
            item.created_at,
        );
        assert!(empty.is_err());
    }

    #[test]
    fn only_hot_and_warm_are_eligible_on_reset() {
        assert!(MemoryTier::Hot.eligible_on_reset());
        assert!(MemoryTier::Warm.eligible_on_reset());
        assert!(!MemoryTier::Cold.eligible_on_reset());
    }
}
