//! The active context: the deliberate projection presented to a model at
//! one inference step (architecture §2), plus the structured reset
//! representation (architecture §12).
//!
//! A [`Context`] is a compiled *view*: it is derived from a
//! [`ContextSnapshot`](crate::ContextSnapshot) for one
//! [`ModelContextProfile`](crate::ModelContextProfile), never stored as a
//! transcript, and never mutates the durable references it projects
//! (`context_compilation_does_not_mutate_task_refs`, kernel §9). The
//! compilation *engine* (ranking, retrieval, budgets) is F6; what is frozen
//! here is the representation: authorization-aware filtering
//! (Secret-authorized items are excluded — kernel §7) and tier eligibility
//! on reset (HOT eligible, WARM selectively included, COLD retrieved just
//! in time).
//!
//! [`ContextReset`] represents the RESET operation of architecture §12: a
//! fresh context reconstructed from durable task state, superseding the
//! previous snapshot. RESET is distinct from COMPACTION (which shrinks
//! while preserving continuity); representing compaction is later work.

use serde::{Deserialize, Serialize};

use crate::ids::{ContextSnapshotId, ModelRef, TaskRef};
use crate::profile::ModelContextProfile;
use crate::provenance::ContextProvenance;
use crate::refs::ActorRef;
use crate::snapshot::{ContextItem, ContextSnapshot};
use crate::time::Timestamp;
use crate::{ContextError, ContextVersion, MAX_CONTEXT_ITEMS};

/// The reason a context was reset (architecture §12): a reset reconstructs
/// a fresh context from durable task state; the reason records what
/// triggered the reconstruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetReason {
    /// A participant asked for a fresh context.
    Manual,
    /// Context pressure rose and structured state was preserved by
    /// reconstructing from durable task state.
    Pressure,
    /// The model changed; durable task state survives the switch
    /// (kernel §6 — a model switch never forks the task).
    ModelChanged,
    /// The execution environment changed; durable task state survives the
    /// switch (kernel §6).
    EnvironmentChanged,
    /// Recovery after an interruption or reconnect.
    Recovery,
}

/// The representation of a context RESET (architecture §12): a fresh
/// context reconstructed from durable task state, superseding the previous
/// snapshot. Durable and attributable like every other context decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextReset {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// The task whose context was reset (identified solely by its
    /// canonical `task_` ID — kernel §6).
    pub task_id: TaskRef,
    /// Why the context was reset.
    pub reason: ResetReason,
    /// The participant or component that performed the reset.
    pub performed_by: ActorRef,
    /// When the reset happened (caller-supplied).
    pub performed_at: Timestamp,
    /// The snapshot the fresh context supersedes.
    pub superseded_snapshot_id: ContextSnapshotId,
}

impl ContextReset {
    /// Builds a reset record.
    pub fn new(
        task_id: TaskRef,
        reason: ResetReason,
        performed_by: ActorRef,
        performed_at: Timestamp,
        superseded_snapshot_id: ContextSnapshotId,
    ) -> Result<Self, ContextError> {
        Ok(Self {
            v: ContextVersion,
            task_id,
            reason,
            performed_by,
            performed_at,
            superseded_snapshot_id,
        })
    }

    /// Validates the reset record.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.task_id.clone(),
            self.reason,
            self.performed_by.clone(),
            self.performed_at,
            self.superseded_snapshot_id.clone(),
        )?;
        Ok(())
    }
}

/// The active model context for one task at one inference step: a compiled
/// projection of a snapshot for one model profile. Not durable state — the
/// durable projection is the snapshot; this view is what a harness presents
/// to the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// The task this context projects (identified solely by its canonical
    /// `task_` ID — kernel §6).
    pub task_id: TaskRef,
    /// The model profile this context was compiled for.
    pub model_id: ModelRef,
    /// The snapshot this context was compiled from.
    pub snapshot_id: ContextSnapshotId,
    /// When the compilation happened (caller-supplied).
    pub compiled_at: Timestamp,
    /// The included items: the authorized, eligible subset of the
    /// snapshot's items.
    pub items: Vec<ContextItem>,
}

impl Context {
    /// Compiles a context view from a snapshot for a model profile.
    ///
    /// This is the frozen *representation* of compilation, not the F6
    /// engine: it applies authorization-aware filtering (items whose
    /// provenance carries the `secret` authorization class are never
    /// included — kernel §7) and preserves every durable reference
    /// untouched. The snapshot is taken by reference and is never mutated.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the profile does not match
    /// the snapshot's model.
    pub fn compile_from_snapshot(
        snapshot: &ContextSnapshot,
        profile: &ModelContextProfile,
        compiled_at: Timestamp,
    ) -> Result<Self, ContextError> {
        if snapshot.model_id != profile.model_id {
            return Err(ContextError::invalid(format!(
                "model context profile {} does not match snapshot model {}",
                profile.model_id, snapshot.model_id
            )));
        }
        let items = snapshot
            .items
            .iter()
            .filter(|item| item.provenance.eligible_for_model_context())
            .cloned()
            .collect();
        Ok(Self {
            v: ContextVersion,
            task_id: snapshot.task_id.clone(),
            model_id: snapshot.model_id.clone(),
            snapshot_id: snapshot.id.clone(),
            compiled_at,
            items,
        })
    }

    /// Validates the compiled view.
    pub fn validate(&self) -> Result<(), ContextError> {
        if self.items.len() > MAX_CONTEXT_ITEMS {
            return Err(ContextError::invalid(format!(
                "compiled context exceeds {MAX_CONTEXT_ITEMS} items"
            )));
        }
        for item in &self.items {
            item.validate()?;
            if !item.provenance.eligible_for_model_context() {
                return Err(ContextError::invalid(
                    "compiled context must not include ineligible items",
                ));
            }
        }
        Ok(())
    }

    /// Lists the provenance of every included item. Provenance covers every
    /// item by construction; this is the accessor the context inspector
    /// surfaces use.
    #[must_use]
    pub fn provenance(&self) -> Vec<&ContextProvenance> {
        self.items.iter().map(|item| &item.provenance).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::SessionRef;
    use crate::memory::MemoryTier;
    use crate::profile::{MultimodalBehavior, ToolSchemaHandling};
    use crate::provenance::{AuthorizationClass, ContextSource};
    use crate::snapshot::{ContextItem, ContextItemContent};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn profile_for(model: &str) -> ModelContextProfile {
        ok(ModelContextProfile::new(
            ok(ModelRef::parse(model)),
            200_000,
            MultimodalBehavior::TextOnly,
            ToolSchemaHandling::LazyPerTool,
            ok(ActorRef::system("flauz-fake")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        ))
    }

    fn snapshot_with_items(model: &str, items: Vec<ContextItem>) -> ContextSnapshot {
        ok(ContextSnapshot::new(
            ok(ContextSnapshotId::parse(
                "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5",
            )),
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            Some(ok(SessionRef::parse("sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6"))),
            ok(ModelRef::parse(model)),
            ok(ActorRef::system("flauz-context-engine")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
            items,
        ))
    }

    fn item(authorization: AuthorizationClass) -> ContextItem {
        ok(ContextItem::new(
            ContextItemContent::Text {
                text: "the dashboard shows 3 open tickets".to_owned(),
            },
            MemoryTier::Hot,
            ok(ContextProvenance::new(
                ContextSource::UserInput,
                authorization,
            )),
        ))
    }

    #[test]
    fn compile_filters_secret_items_and_preserves_references() {
        let snapshot = snapshot_with_items(
            "model_01J8ZQ5V8K3T2B7N6X4R9DQPRE",
            vec![
                item(AuthorizationClass::Task),
                item(AuthorizationClass::Secret),
                item(AuthorizationClass::Participant),
            ],
        );
        let task_before = snapshot.task_id.clone();
        let context = ok(Context::compile_from_snapshot(
            &snapshot,
            &profile_for("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE"),
            ok(Timestamp::parse("2026-09-21T13:46:00Z")),
        ));
        assert_eq!(context.items.len(), 2, "secret items are never included");
        assert_eq!(context.task_id, task_before);
        assert_eq!(context.snapshot_id, snapshot.id);
        ok(context.validate());

        // The snapshot was not mutated by compilation.
        assert_eq!(snapshot.task_id, task_before);
        assert_eq!(snapshot.items.len(), 3);
        ok(snapshot.validate());
    }

    #[test]
    fn compile_rejects_a_profile_for_another_model() {
        let snapshot = snapshot_with_items("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE", Vec::new());
        let other_profile = profile_for("model_01J8ZQ5V8K3T2B7N6X4R9DQPSF");
        assert!(
            Context::compile_from_snapshot(
                &snapshot,
                &other_profile,
                ok(Timestamp::parse("2026-09-21T13:46:00Z")),
            )
            .is_err()
        );
    }

    #[test]
    fn reset_record_round_trips() {
        let reset = ok(ContextReset::new(
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            ResetReason::Pressure,
            ok(ActorRef::user("alice")),
            ok(Timestamp::parse("2026-09-21T14:00:00Z")),
            ok(ContextSnapshotId::parse(
                "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5",
            )),
        ));
        let serialized = ok(serde_json::to_string(&reset));
        let reloaded: ContextReset = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, reset);
        assert!(
            serde_json::from_str::<ContextReset>(&serialized.replace("\"v\":1", "\"v\":2"))
                .is_err()
        );
    }
}
