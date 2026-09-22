//! In-memory fakes: the deterministic conformance surface (kernel §7).
//!
//! [`FakeContextStore`] implements [`ContextStore`] with plain in-memory
//! maps, no I/O, and no wall-clock reads: every timestamp is
//! caller-supplied and the only generated values are canonical IDs (the
//! crate's single entropy source, produced by the private `ulid` module).
//! Use it for tests, the F2 integration gate, and as the reference
//! semantics for real stores.

use std::collections::BTreeMap;

use crate::MAX_CONTEXT_ITEMS;
use crate::context::{Context, ContextReset, ResetReason};
use crate::ids::{ContextSnapshotId, MemoryItemId, ModelRef, TaskRef};
use crate::memory::{MemoryContent, MemoryItem};
use crate::profile::ModelContextProfile;
use crate::provenance::{ContextProvenance, ContextSource};
use crate::refs::ActorRef;
use crate::snapshot::{ContextItem, ContextItemContent, ContextSnapshot};
use crate::store::{ContextReconstruction, ContextStateSnapshot, ContextStore, ContextStoreError};
use crate::time::Timestamp;

/// The in-memory fake context store. Deterministic: identical operation
/// sequences produce identical logical state (generated IDs aside).
#[derive(Debug, Default, Clone)]
pub struct FakeContextStore {
    memory_items: BTreeMap<MemoryItemId, MemoryItem>,
    snapshots: BTreeMap<ContextSnapshotId, ContextSnapshot>,
    profiles: BTreeMap<ModelRef, ModelContextProfile>,
    resets: Vec<ContextReset>,
}

impl FakeContextStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a memory-backed context item for one memory item: the item's
    /// content becomes the projected content, and the provenance attributes
    /// it to the memory item while carrying the item's own authorization
    /// class. This is the frozen representation of "a memory item included
    /// as context" — the F6 engine owns selection/ranking.
    fn memory_context_item(item: &MemoryItem) -> Result<ContextItem, ContextStoreError> {
        let content = match &item.content {
            MemoryContent::Text { text } => ContextItemContent::Text { text: text.clone() },
            MemoryContent::Reference { reference } => ContextItemContent::Reference {
                reference: reference.clone(),
            },
        };
        ContextItem::new(
            content,
            item.tier,
            ContextProvenance::new(
                ContextSource::MemoryItem {
                    memory_item_id: item.id.clone(),
                },
                item.authorization,
            )?,
        )
        .map_err(ContextStoreError::from)
    }

    /// Collects the reset-eligible, authorized memory items for a task:
    /// task-scoped items plus workspace-scoped memory, HOT and WARM only
    /// (COLD is retrieved just in time — architecture §2), each carrying
    /// its own recorded authorization class, bounded to the canonical item
    /// limit. Deterministic order: canonical memory-item ID order.
    fn reset_eligible_items(
        &self,
        task_id: &TaskRef,
    ) -> Result<Vec<ContextItem>, ContextStoreError> {
        let mut items = Vec::new();
        for item in self.memory_items.values() {
            if items.len() == MAX_CONTEXT_ITEMS {
                break;
            }
            let task_scoped = item.task_id.as_ref() == Some(task_id);
            let workspace_scoped = item.task_id.is_none();
            if (task_scoped || workspace_scoped)
                && item.tier.eligible_on_reset()
                && item.authorization.eligible_for_model_context()
            {
                items.push(Self::memory_context_item(item)?);
            }
        }
        Ok(items)
    }
}

impl ContextStore for FakeContextStore {
    fn create_memory_item(&mut self, item: MemoryItem) -> Result<MemoryItem, ContextStoreError> {
        item.validate()?;
        if let Some(existing) = self.memory_items.get(&item.id) {
            return Err(ContextStoreError::Duplicate {
                id: existing.id.as_str().to_owned(),
            });
        }
        if item.version != 1 {
            return Err(ContextStoreError::Invalid {
                reason: format!("memory item {} must be created at version 1", item.id),
            });
        }
        self.memory_items.insert(item.id.clone(), item.clone());
        Ok(item)
    }

    fn memory_item(&self, id: &MemoryItemId) -> Result<Option<MemoryItem>, ContextStoreError> {
        Ok(self.memory_items.get(id).cloned())
    }

    fn update_memory_item(&mut self, item: MemoryItem) -> Result<MemoryItem, ContextStoreError> {
        item.validate()?;
        let Some(stored) = self.memory_items.get(&item.id) else {
            return Err(ContextStoreError::NotFound {
                id: item.id.as_str().to_owned(),
            });
        };
        if stored.version != item.version {
            return Err(ContextStoreError::VersionConflict {
                id: item.id.as_str().to_owned(),
                expected_version: item.version,
                actual_version: stored.version,
            });
        }
        let mut updated = item;
        updated.version = stored.version + 1;
        self.memory_items
            .insert(updated.id.clone(), updated.clone());
        Ok(updated)
    }

    fn put_snapshot(&mut self, snapshot: ContextSnapshot) -> Result<(), ContextStoreError> {
        snapshot.validate()?;
        if self.snapshots.contains_key(&snapshot.id) {
            return Err(ContextStoreError::Duplicate {
                id: snapshot.id.as_str().to_owned(),
            });
        }
        self.snapshots.insert(snapshot.id.clone(), snapshot);
        Ok(())
    }

    fn snapshot(
        &self,
        id: &ContextSnapshotId,
    ) -> Result<Option<ContextSnapshot>, ContextStoreError> {
        Ok(self.snapshots.get(id).cloned())
    }

    fn create_profile(
        &mut self,
        profile: ModelContextProfile,
    ) -> Result<ModelContextProfile, ContextStoreError> {
        profile.validate()?;
        if let Some(existing) = self.profiles.get(&profile.model_id) {
            return Err(ContextStoreError::Duplicate {
                id: existing.model_id.as_str().to_owned(),
            });
        }
        if profile.version != 1 {
            return Err(ContextStoreError::Invalid {
                reason: format!(
                    "model context profile for {} must be created at version 1",
                    profile.model_id
                ),
            });
        }
        self.profiles
            .insert(profile.model_id.clone(), profile.clone());
        Ok(profile)
    }

    fn profile(&self, model: &ModelRef) -> Result<Option<ModelContextProfile>, ContextStoreError> {
        Ok(self.profiles.get(model).cloned())
    }

    fn update_profile(
        &mut self,
        profile: ModelContextProfile,
    ) -> Result<ModelContextProfile, ContextStoreError> {
        profile.validate()?;
        let Some(stored) = self.profiles.get(&profile.model_id) else {
            return Err(ContextStoreError::NotFound {
                id: profile.model_id.as_str().to_owned(),
            });
        };
        if stored.version != profile.version {
            return Err(ContextStoreError::VersionConflict {
                id: profile.model_id.as_str().to_owned(),
                expected_version: profile.version,
                actual_version: stored.version,
            });
        }
        let mut updated = profile;
        updated.version = stored.version + 1;
        self.profiles
            .insert(updated.model_id.clone(), updated.clone());
        Ok(updated)
    }

    fn reset_context(
        &mut self,
        task_id: TaskRef,
        model: ModelRef,
        reason: ResetReason,
        performed_by: ActorRef,
        performed_at: Timestamp,
        superseded_snapshot: ContextSnapshotId,
    ) -> Result<ContextReconstruction, ContextStoreError> {
        let Some(superseded) = self.snapshots.get(&superseded_snapshot).cloned() else {
            return Err(ContextStoreError::NotFound {
                id: superseded_snapshot.as_str().to_owned(),
            });
        };
        if superseded.task_id != task_id {
            return Err(ContextStoreError::Invalid {
                reason: format!(
                    "superseded snapshot {} belongs to task {}, not {task_id}",
                    superseded.id, superseded.task_id
                ),
            });
        }
        let Some(profile) = self.profiles.get(&model).cloned() else {
            return Err(ContextStoreError::NotFound {
                id: model.as_str().to_owned(),
            });
        };

        // Reconstruct the fresh durable snapshot from durable task state:
        // the reset-eligible, authorized memory items. The superseded
        // snapshot is never read for content — a reset is not a compaction
        // (architecture §12).
        let items = self.reset_eligible_items(&task_id)?;
        let fresh = ContextSnapshot::new(
            ContextSnapshotId::generate(),
            task_id.clone(),
            superseded.session_id.clone(),
            model.clone(),
            performed_by.clone(),
            performed_at,
            items,
        )?;
        self.put_snapshot(fresh.clone())?;

        let reset = ContextReset::new(
            task_id,
            reason,
            performed_by,
            performed_at,
            superseded_snapshot,
        )?;
        self.resets.push(reset.clone());

        let context = Context::compile_from_snapshot(&fresh, &profile, performed_at)?;
        Ok(ContextReconstruction {
            reset,
            snapshot: fresh,
            context,
        })
    }

    fn state(&self) -> ContextStateSnapshot {
        ContextStateSnapshot {
            v: crate::ContextVersion,
            memory_items: self.memory_items.clone(),
            snapshots: self.snapshots.clone(),
            profiles: self.profiles.clone(),
            resets: self.resets.clone(),
        }
    }

    fn restore(state: ContextStateSnapshot) -> Result<Self, ContextStoreError> {
        state.validate()?;
        Ok(Self {
            memory_items: state.memory_items,
            snapshots: state.snapshots,
            profiles: state.profiles,
            resets: state.resets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthorizationClass;
    use crate::MemoryTier;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn fake_store_round_trips_its_entire_state() {
        let mut store = FakeContextStore::new();
        ok(store.create_memory_item(ok(MemoryItem::new(
            MemoryItemId::generate(),
            None,
            MemoryTier::Warm,
            AuthorizationClass::Workspace,
            MemoryContent::Text {
                text: "decision: use the CRM export".to_owned(),
            },
            ok(ActorRef::user("alice")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        ))));
        let snapshot = ok(ContextSnapshot::new(
            ContextSnapshotId::generate(),
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            None,
            ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE")),
            ok(ActorRef::system("flauz-context-engine")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
            Vec::new(),
        ));
        ok(store.put_snapshot(snapshot));

        let serialized = ok(serde_json::to_string(&store.state()));
        let reloaded: ContextStateSnapshot = ok(serde_json::from_str(&serialized));
        let restored = ok(FakeContextStore::restore(reloaded));
        assert_eq!(restored.state(), store.state());
    }
}
