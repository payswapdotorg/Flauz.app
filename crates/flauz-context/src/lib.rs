//! Canonical context, memory and model-profile contracts for Flauz
//! (F2 contract wave, work order **ORCH-001**).
//!
//! This crate is the single owner of the platform's context-side types:
//! [`Context`], [`MemoryItem`], [`ContextSnapshot`],
//! [`ContextProvenance`] and [`ModelContextProfile`], plus the reset
//! representation [`ContextReset`] (architecture §12). Every cross-crate
//! rule implemented here comes from the frozen
//! [F2 contract kernel](../../docs/F2-CONTRACT-KERNEL.md) and the approved
//! [context/harness architecture](../../docs/CONTEXT-HARNESS-ARCHITECTURE.md).
//!
//! # Context is a projection, not a transcript
//!
//! A [`ContextSnapshot`] is a durable, serializable **projection**: it
//! references durable entities by their canonical opaque ID strings
//! (kernel §2 format) through validated local newtypes — [`TaskRef`],
//! [`SessionRef`], [`EventRef`], [`ArtifactRef`], [`ObservationRef`],
//! [`EnvironmentRef`], [`ModelRef`] — and never re-defines the owning
//! crates' entity types. Foreign crates are referenced by frozen formats,
//! not by cargo dependencies: this crate is self-contained (no dependency
//! on flauz-world or flauz-exec) and builds alone.
//!
//! Every included context item carries [`ContextProvenance`]: the source it
//! is attributable to (one of the ten frozen source kinds — user input,
//! durable session event, memory item, artifact, resource observation, tool
//! result, retrieved document, skill, environment state, collaboration
//! event) plus the authorization class that governs its eligibility as
//! model context. Authorization-aware filtering is representable: an item
//! whose provenance carries [`AuthorizationClass::Secret`] is authorized
//! for retrieval by its owner but is **never** included in compiled context
//! — secrets never become context merely because they are technically
//! retrievable (kernel §7).
//!
//! # Session ≠ Context, Context ≠ Memory, TaskState ≠ AgentContext
//!
//! A [`Session`](crate::ids::SessionRef) references at most one task; a
//! snapshot *may* record the session a compilation happened in, but the
//! session is never the context. [`MemoryItem`]s are durable rememberable
//! state tiered HOT / WARM / COLD (HOT: current turn, current tool result,
//! active plan, current errors, current environment state; WARM: recent
//! conversation, task summary, decisions, unresolved questions,
//! discoveries, recent artifacts; COLD: complete history, archived tool
//! results, workspace knowledge, documents, old executions, reusable
//! procedures). The active [`Context`] is the compiled view presented to
//! one model at one inference step.
//!
//! # Model-aware compilation (representation only)
//!
//! [`ModelContextProfile`] records the per-model compilation profile —
//! context capacity, multimodal behavior, tool-schema handling — keyed by
//! the [`ModelRef`] it profiles. [`Context::compile_from_snapshot`] is the
//! frozen *representation* of compilation: authorization-aware filtering
//! with every durable reference preserved untouched. The compilation
//! engine (retrieval, ranking, budgets) is F6, not this crate. Model
//! switches never mutate task references and never fork the logical task
//! (kernel §6).
//!
//! # Snapshots and resets (architecture §12)
//!
//! Snapshots are durable, serializable and reconstructible from their
//! references ([`ContextSnapshot::durable_references`]); they are
//! immutable once compiled — a new compilation is a new snapshot.
//! [`ContextReset`] represents the RESET operation: a fresh context
//! reconstructed from durable task state, superseding the previous
//! snapshot. RESET is distinct from COMPACTION (preserve continuity while
//! shrinking); compaction is represented by the Wave-3 [`tiers`] module
//! (ORCH-002) through its own record family.
//!
//! # The context engine (Wave 3, ORCH-002)
//!
//! The [`engine`] module compiles context snapshots from a task's
//! **durable state** — artifacts, observations, evidence, recent events
//! and memory items, taken as data (the flauz-cap pattern: inputs, never
//! entity imports). The rebuild law holds: two compilations from the
//! same durable state with no memory-item mutation yield equivalent
//! projections — context reconstructs without replaying any model
//! conversation. The [`tiers`] module adds tier dynamics (promotion and
//! demotion with a bounded HOT tier) and the structured compaction
//! planner (provenance retained on every retained item, every summarized
//! item named in a compaction record, budgets per model profile). The
//! [`tools`] module computes dynamic tool exposure per model profile:
//! excluded tools are named, never silently dropped.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields (`deny_unknown_fields`),
//! contains no floats, emits timestamps as RFC 3339 UTC
//! `YYYY-MM-DDTHH:MM:SSZ`, and keeps byte payloads behind bounded
//! references. Enums are internally tagged with `"kind"`.
//!
//! # Determinism (kernel §7)
//!
//! Contract code in this crate never reads wall-clock time or randomness
//! directly: every timestamp is passed in by callers. The only entropy
//! source is ID generation, confined to a private `ulid` module (no
//! external ULID crate). The in-memory [`fakes::FakeContextStore`] is
//! fully deterministic apart from generated IDs.
//!
//! # No credential material
//!
//! No contract type, fixture, or serialized state in this crate contains
//! credential material: secret-authorized items carry references and
//! authorization classes only, and are never compiled into model context
//! (kernel §7).

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod context;
pub mod engine;
pub mod fakes;
pub mod ids;
pub mod memory;
pub mod profile;
pub mod provenance;
pub mod refs;
pub mod snapshot;
pub mod store;
pub mod tiers;
pub mod time;
pub mod tools;

mod ulid;

pub use crate::context::{Context, ContextReset, ResetReason};
pub use crate::engine::{
    ArtifactRecord, DurableStateInputs, EvidenceRecord, EvidenceRef, ObservationRecord,
    TaskEventRecord, compile_from_durable_state, projections_equivalent,
};
pub use crate::ids::{
    ArtifactRef, ContextSnapshotId, EntityKind, EnvironmentRef, EventRef, IdError, MemoryItemId,
    ModelRef, ObservationRef, SessionRef, TaskRef, validate,
};
pub use crate::memory::{MemoryContent, MemoryItem, MemoryTier};
pub use crate::profile::{ModelContextProfile, MultimodalBehavior, ToolSchemaHandling};
pub use crate::provenance::{AuthorizationClass, ContextProvenance, ContextSource};
pub use crate::refs::{ActorKind, ActorRef, SkillRef};
pub use crate::snapshot::{ContextItem, ContextItemContent, ContextSnapshot};
pub use crate::store::{
    ContextReconstruction, ContextStateSnapshot, ContextStore, ContextStoreError,
};
pub use crate::tiers::{
    Compaction, CompactionBudget, CompactionReason, CompactionRecord, SummarizedItem,
    age_out_unreferenced_warm, demote_memory_item, enforce_hot_memory_bound, estimate_tokens,
    plan_compaction, promote_memory_item,
};
pub use crate::time::Timestamp;
pub use crate::tools::{
    AdmissionGap, CapabilityAdmission, ExcludedTool, ExposedTool, ToolDefinition, ToolExposure,
    ToolPresentation, compute_tool_exposure, validate_capability_key,
};

/// Maximum length of short human-readable names (skill keys).
pub const MAX_NAME_BYTES: usize = 256;

/// Maximum length of statements, objectives and other bounded prose fields.
pub const MAX_STATEMENT_BYTES: usize = 8 * 1024;

/// Maximum length of bounded inline memory content.
pub const MAX_MEMORY_CONTENT_BYTES: usize = 64 * 1024;

/// Maximum length of bounded inline context-item content.
pub const MAX_ITEM_CONTENT_BYTES: usize = 64 * 1024;

/// Maximum length of an actor identifier string.
pub const MAX_ACTOR_ID_BYTES: usize = 256;

/// Maximum number of items in one context snapshot (and in one compiled
/// context view).
pub const MAX_CONTEXT_ITEMS: usize = 256;

/// Smallest representable model context capacity, in tokens.
pub const MIN_MODEL_TOKENS: u64 = 1;

/// Largest representable model context capacity, in tokens.
pub const MAX_MODEL_TOKENS: u64 = 100_000_000;

/// Contract-level validation error for values that fail canonical rules.
///
/// Store operations surface these through
/// [`ContextStoreError::Invalid`](store::ContextStoreError).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextError {
    /// A canonical identifier is malformed (see [`IdError`] for the frozen
    /// failure reasons).
    Id(IdError),
    /// A value violates a canonical rule (bounds, ordering, grammar).
    Invalid(String),
}

impl ContextError {
    /// Builds a [`ContextError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl fmt::Display for ContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(error) => error.fmt(formatter),
            Self::Invalid(reason) => write!(formatter, "invalid context state: {reason}"),
        }
    }
}

impl Error for ContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Id(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<IdError> for ContextError {
    fn from(error: IdError) -> Self {
        Self::Id(error)
    }
}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextVersion;

impl serde::Serialize for ContextVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for ContextVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = u32::deserialize(deserializer)?;
        if version == 1 {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported contract schema version {version}; this build reads v1"
            )))
        }
    }
}

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), ContextError> {
    if value.is_empty() {
        Err(ContextError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), ContextError> {
    if value.len() > max_bytes {
        Err(ContextError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
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
    fn contract_version_serializes_as_one_and_rejects_others() {
        let serialized = ok(serde_json::to_string(&ContextVersion));
        assert_eq!(serialized, "1");
        assert!(serde_json::from_str::<ContextVersion>("1").is_ok());
        assert!(serde_json::from_str::<ContextVersion>("2").is_err());
        assert!(serde_json::from_str::<ContextVersion>("0").is_err());
    }
}
