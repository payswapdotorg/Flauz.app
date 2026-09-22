//! The context-state store contract, the store error surface, and the
//! canonical context-state document used for serialize → drop → reload
//! round-trips.
//!
//! Mutations follow the kernel §3 version rules: entities are created at
//! version 1, every durable mutation increments by exactly 1, and updates
//! carry the caller's expected version — a mismatch is a
//! [`ContextStoreError::VersionConflict`]; silent overwrite is forbidden.
//!
//! [`ContextSnapshot`]s are immutable once compiled: they enter the store
//! through `put_snapshot` and are never updated (a new compilation is a new
//! snapshot). [`MemoryItem`]s and [`ModelContextProfile`]s are mutable and
//! follow the full version rules. [`ContextReset`] records are append-only
//! audit entries.
//!
//! `reset_context` is the represented RESET operation (architecture §12):
//! it reconstructs a fresh [`Context`] for a task from durable state —
//! the task's authorized, reset-eligible memory items — and records the
//! reset. The full compilation engine (retrieval, ranking, budgets) is F6,
//! not this wave.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::context::{Context, ContextReset, ResetReason};
use crate::ids::{ContextSnapshotId, MemoryItemId, ModelRef, TaskRef};
use crate::memory::MemoryItem;
use crate::profile::ModelContextProfile;
use crate::refs::ActorRef;
use crate::snapshot::ContextSnapshot;
use crate::time::Timestamp;
use crate::ContextVersion;

/// Errors returned by context-state store operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextStoreError {
    /// An entity with the same canonical ID (or profile key) already
    /// exists.
    Duplicate {
        /// The conflicting canonical ID or key.
        id: String,
    },
    /// No entity exists with the given canonical ID (or profile key).
    NotFound {
        /// The missing canonical ID or key.
        id: String,
    },
    /// The caller's expected version did not match the stored version
    /// (kernel §3: optimistic concurrency; silent overwrite is forbidden).
    VersionConflict {
        /// The conflicting canonical ID.
        id: String,
        /// The version the caller expected.
        expected_version: u64,
        /// The version actually stored.
        actual_version: u64,
    },
    /// The operation violates a canonical rule (bounds, ordering, state).
    Invalid {
        /// The reason the operation was rejected.
        reason: String,
    },
}

impl fmt::Display for ContextStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate { id } => write!(formatter, "entity {id} already exists"),
            Self::NotFound { id } => write!(formatter, "entity {id} does not exist"),
            Self::VersionConflict {
                id,
                expected_version,
                actual_version,
            } => write!(
                formatter,
                "version conflict on {id}: expected {expected_version}, stored {actual_version}"
            ),
            Self::Invalid { reason } => write!(formatter, "invalid operation: {reason}"),
        }
    }
}

impl Error for ContextStoreError {}

impl From<crate::ContextError> for ContextStoreError {
    fn from(error: crate::ContextError) -> Self {
        Self::Invalid {
            reason: error.to_string(),
        }
    }
}

/// The result of a represented context RESET: the recorded reset decision,
/// the fresh durable snapshot reconstructed from durable task state, and
/// the compiled view of that snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextReconstruction {
    /// The append-only reset record.
    pub reset: ContextReset,
    /// The fresh durable snapshot the reset produced (a new `ctxsnap_`
    /// entity; the superseded snapshot is unchanged).
    pub snapshot: ContextSnapshot,
    /// The fresh context: the compiled view of the fresh snapshot.
    pub context: Context,
}

/// The canonical serialization of the entire context state: memory items,
/// snapshots, model profiles and reset records. Used by fakes and real
/// stores alike for serialize → drop → reload round-trips; the state
/// survives with equality (kernel §4 round-trip law).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextStateSnapshot {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// All memory items, keyed by canonical ID.
    pub memory_items: BTreeMap<MemoryItemId, MemoryItem>,
    /// All context snapshots, keyed by canonical ID. Immutable once
    /// compiled.
    pub snapshots: BTreeMap<ContextSnapshotId, ContextSnapshot>,
    /// All model context profiles, keyed by the model they profile.
    pub profiles: BTreeMap<ModelRef, ModelContextProfile>,
    /// All reset records, in canonical ID order of the superseded
    /// snapshots (append-only audit trail).
    pub resets: Vec<ContextReset>,
}

impl ContextStateSnapshot {
    /// The empty state snapshot.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            v: ContextVersion,
            memory_items: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            profiles: BTreeMap::new(),
            resets: Vec::new(),
        }
    }

    /// Validates the state snapshot's structural invariants: map keys match
    /// the entity IDs (or profile keys) they store, and every entity passes
    /// canonical validation.
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::Invalid`] when an invariant is broken.
    pub fn validate(&self) -> Result<(), ContextStoreError> {
        for (id, item) in &self.memory_items {
            check_key(id.as_str(), item.id.as_str(), item.validate())?;
        }
        for (id, snapshot) in &self.snapshots {
            check_key(id.as_str(), snapshot.id.as_str(), snapshot.validate())?;
        }
        for (model, profile) in &self.profiles {
            check_key(model.as_str(), profile.model_id.as_str(), profile.validate())?;
        }
        for reset in &self.resets {
            if let Err(error) = reset.validate() {
                return Err(ContextStoreError::Invalid {
                    reason: format!("reset record for {} is invalid: {error}", reset.task_id),
                });
            }
        }
        Ok(())
    }
}

fn check_key(
    key: &str,
    entity_id: &str,
    validation: Result<(), crate::ContextError>,
) -> Result<(), ContextStoreError> {
    if key != entity_id {
        return Err(ContextStoreError::Invalid {
            reason: format!("map key {key} does not match entity id {entity_id}"),
        });
    }
    validation.map_err(ContextStoreError::from)
}

/// The context-state store contract.
///
/// All timestamps are supplied by callers; the store never reads the wall
/// clock. Snapshots are immutable; memory items and profiles follow the
/// kernel §3 optimistic-concurrency version rules.
pub trait ContextStore {
    /// Stores a new memory item (created at version 1).
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::Duplicate`] when the ID already exists
    /// and [`ContextStoreError::Invalid`] when the item fails validation.
    fn create_memory_item(&mut self, item: MemoryItem) -> Result<MemoryItem, ContextStoreError>;

    /// Looks up a memory item.
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::Invalid`] only on store misuse; a
    /// missing item yields `Ok(None)`.
    fn memory_item(&self, id: &MemoryItemId) -> Result<Option<MemoryItem>, ContextStoreError>;

    /// Updates a memory item (optimistic concurrency on the passed
    /// version; kernel §3).
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::VersionConflict`] on an expected-version
    /// mismatch.
    fn update_memory_item(&mut self, item: MemoryItem) -> Result<MemoryItem, ContextStoreError>;

    /// Stores an immutable context snapshot. A new compilation is a new
    /// snapshot; snapshots are never updated.
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::Duplicate`] when the ID already exists.
    fn put_snapshot(&mut self, snapshot: ContextSnapshot) -> Result<(), ContextStoreError>;

    /// Looks up a context snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::Invalid`] only on store misuse; a
    /// missing snapshot yields `Ok(None)`.
    fn snapshot(&self, id: &ContextSnapshotId)
        -> Result<Option<ContextSnapshot>, ContextStoreError>;

    /// Stores a new model context profile (created at version 1, keyed by
    /// the model it profiles).
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::Duplicate`] when a profile for the
    /// model already exists.
    fn create_profile(
        &mut self,
        profile: ModelContextProfile,
    ) -> Result<ModelContextProfile, ContextStoreError>;

    /// Looks up the profile for a model.
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::Invalid`] only on store misuse; a
    /// missing profile yields `Ok(None)`.
    fn profile(&self, model: &ModelRef) -> Result<Option<ModelContextProfile>, ContextStoreError>;

    /// Updates a model context profile (optimistic concurrency on the
    /// passed version; kernel §3).
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::VersionConflict`] on an
    /// expected-version mismatch.
    fn update_profile(
        &mut self,
        profile: ModelContextProfile,
    ) -> Result<ModelContextProfile, ContextStoreError>;

    /// The represented RESET operation (architecture §12): records the
    /// reset decision, reconstructs a fresh durable snapshot for the task
    /// from durable state — its authorized, reset-eligible memory items
    /// (HOT eligible by default, WARM selectively included, COLD retrieved
    /// just in time and therefore excluded) — and returns the compiled
    /// view of the fresh snapshot for `model`. The superseded snapshot must
    /// exist and is never mutated; the fresh snapshot is a new `ctxsnap_`
    /// entity.
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::NotFound`] when the superseded snapshot
    /// or the model profile does not exist.
    fn reset_context(
        &mut self,
        task_id: TaskRef,
        model: ModelRef,
        reason: ResetReason,
        performed_by: ActorRef,
        performed_at: Timestamp,
        superseded_snapshot: ContextSnapshotId,
    ) -> Result<ContextReconstruction, ContextStoreError>;

    /// Captures the entire context state as a canonical document.
    fn state(&self) -> ContextStateSnapshot;

    /// Rebuilds a store from a state snapshot (serialize → drop → reload).
    ///
    /// # Errors
    ///
    /// Returns [`ContextStoreError::Invalid`] when the snapshot fails the
    /// canonical invariants.
    fn restore(state: ContextStateSnapshot) -> Result<Self, ContextStoreError>
    where
        Self: Sized;
}
