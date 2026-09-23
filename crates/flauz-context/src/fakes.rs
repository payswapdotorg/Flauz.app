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
use crate::engine::{
    ArtifactRecord, DurableStateInputs, EvidenceRecord, ObservationRecord, TaskEventRecord,
};
use crate::ids::{ContextSnapshotId, MemoryItemId, ModelRef, TaskRef};
use crate::memory::{MemoryContent, MemoryItem};
use crate::profile::ModelContextProfile;
use crate::provenance::{ContextProvenance, ContextSource};
use crate::refs::ActorRef;
use crate::snapshot::{ContextItem, ContextItemContent, ContextSnapshot};
use crate::store::{ContextReconstruction, ContextStateSnapshot, ContextStore, ContextStoreError};
use crate::time::Timestamp;
use crate::tools::{CapabilityAdmission, ToolDefinition};

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

// ---------------------------------------------------------------
// The Wave-3 engine fakes (ORCH-002): the deterministic durable-state,
// compaction-budget and tool-exposure fixture families. Everything here
// is a pure function of nothing — fixed canonical IDs (frozen-format
// strings), fixed caller-supplied timestamps, no I/O, no wall clock, no
// randomness — so downstream waves (ORCH-003's harness, the Wave-3
// integration gate) can rely on byte-stable reference inputs.
// ---------------------------------------------------------------

/// The task-shaped canonical IDs the durable-state fakes are built from
/// (frozen-format strings, fixed across runs).
mod durable_ids {
    use crate::engine::EvidenceRef;
    use crate::ids::{ArtifactRef, EventRef, MemoryItemId, ObservationRef, SessionRef, TaskRef};

    /// The task under compilation.
    pub(crate) const TASK: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";
    /// Another task (its memory must never leak into this task's
    /// projection).
    pub(crate) const OTHER_TASK: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQZ9T";
    /// The session the compilation happens in.
    pub(crate) const SESSION: &str = "sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6";
    /// The first artifact.
    pub(crate) const ARTIFACT_A: &str = "art_01J8ZQ5V8K3T2B7N6X4R9DQPH7";
    /// The second artifact.
    pub(crate) const ARTIFACT_B: &str = "art_01J8ZQ5V8K3T2B7N6X4R9DQPH8";
    /// The first observation.
    pub(crate) const OBSERVATION_A: &str = "obs_01J8ZQ5V8K3T2B7N6X4R9DQPJ8";
    /// The second observation.
    pub(crate) const OBSERVATION_B: &str = "obs_01J8ZQ5V8K3T2B7N6X4R9DQPJ9";
    /// The evidence record.
    pub(crate) const EVIDENCE: &str = "evd_01J8ZQ5V8K3T2B7N6X4R9DQPK9";
    /// The task-started event.
    pub(crate) const EVENT_A: &str = "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3";
    /// The verification event.
    pub(crate) const EVENT_B: &str = "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD4";
    /// HOT current-plan memory.
    pub(crate) const MEMORY_PLAN: &str = "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5";
    /// WARM decision memory.
    pub(crate) const MEMORY_DECISION: &str = "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF6";
    /// WARM workspace-scoped knowledge.
    pub(crate) const MEMORY_WORKSPACE: &str = "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF7";
    /// COLD archive (jit-only).
    pub(crate) const MEMORY_ARCHIVE: &str = "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF8";
    /// HOT secret reference (storable, never compiled).
    pub(crate) const MEMORY_SECRET: &str = "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF9";
    /// HOT memory of the OTHER task.
    pub(crate) const MEMORY_OTHER_TASK: &str = "mem_01J8ZQ5V8K3T2B7N6X4R9DQPGA";
    /// The model the compilation targets.
    pub(crate) const MODEL: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPRE";

    /// Parses a task reference (fixed vector).
    pub(crate) fn task(value: &str) -> Result<TaskRef, crate::ContextError> {
        Ok(TaskRef::parse(value)?)
    }

    /// Parses a session reference (fixed vector).
    pub(crate) fn session() -> Result<SessionRef, crate::ContextError> {
        Ok(SessionRef::parse(SESSION)?)
    }

    /// Parses an artifact reference (fixed vector).
    pub(crate) fn artifact(value: &str) -> Result<ArtifactRef, crate::ContextError> {
        Ok(ArtifactRef::parse(value)?)
    }

    /// Parses an observation reference (fixed vector).
    pub(crate) fn observation(value: &str) -> Result<ObservationRef, crate::ContextError> {
        Ok(ObservationRef::parse(value)?)
    }

    /// Parses the evidence reference (fixed vector).
    pub(crate) fn evidence() -> Result<EvidenceRef, crate::ContextError> {
        Ok(EvidenceRef::parse(EVIDENCE)?)
    }

    /// Parses an event reference (fixed vector).
    pub(crate) fn event(value: &str) -> Result<EventRef, crate::ContextError> {
        Ok(EventRef::parse(value)?)
    }

    /// Parses a memory-item ID (fixed vector).
    pub(crate) fn memory(value: &str) -> Result<MemoryItemId, crate::ContextError> {
        Ok(MemoryItemId::parse(value)?)
    }
}

/// The exact byte length of every pressure-fake text (sized so the token
/// estimate is 16 per item: a 40-token budget retains the two HOT items
/// and summarizes the four WARM items).
const PRESSURE_TEXT_BYTES: usize = 60;

/// Pads one seed line to the exact pressure text length (all-ASCII, so
/// byte length and char count agree).
fn pressure_text(seed: &str) -> String {
    let mut text = seed.to_owned();
    while text.len() < PRESSURE_TEXT_BYTES {
        text.push('x');
    }
    text
}

/// Builds one memory item of the durable-state fakes.
fn durable_memory(
    id: &str,
    task: Option<&str>,
    tier: crate::MemoryTier,
    authorization: crate::AuthorizationClass,
    content: MemoryContent,
    created_at: &str,
) -> Result<MemoryItem, crate::ContextError> {
    MemoryItem::new(
        durable_ids::memory(id)?,
        task.map(durable_ids::task).transpose()?,
        tier,
        authorization,
        content,
        ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD")?,
        Timestamp::parse(created_at)?,
    )
}

/// The typical task-shaped durable state (the reference input of the
/// engine's rebuild law): two artifacts, two observations, one verified
/// evidence record, two recent events, and a full memory family — HOT
/// current plan, WARM decision, WARM workspace knowledge, COLD archive
/// (jit-only), a HOT secret reference (storable, never compiled), and
/// another task's HOT memory (never in scope).
///
/// Deterministic: fixed canonical IDs, fixed caller-supplied timestamps,
/// no generated values.
///
/// # Errors
///
/// Returns [`ContextStoreError::Invalid`] if any fixed record fails
/// canonical validation (a bug in the fakes, not in caller data).
pub fn fake_typical_durable_state() -> Result<DurableStateInputs, crate::ContextError> {
    DurableStateInputs::new(
        durable_ids::task(durable_ids::TASK)?,
        Some(durable_ids::session()?),
        vec![
            ArtifactRecord::new(
                durable_ids::artifact(durable_ids::ARTIFACT_A)?,
                "the CRM ticket export: 3 open tickets, reconciled against the dashboard",
            )?,
            ArtifactRecord::new(
                durable_ids::artifact(durable_ids::ARTIFACT_B)?,
                "the reconciliation report draft: dashboard and CRM counts now match",
            )?,
        ],
        vec![
            ObservationRecord::new(
                durable_ids::observation(durable_ids::OBSERVATION_A)?,
                "the dashboard currently shows 3 open tickets",
            )?,
            ObservationRecord::new(
                durable_ids::observation(durable_ids::OBSERVATION_B)?,
                "the CRM export lists 3 open tickets as of the last sync",
            )?,
        ],
        vec![EvidenceRecord::new(
            durable_ids::evidence()?,
            durable_ids::event(durable_ids::EVENT_B)?,
            "the ticket counts were independently verified: 3 open tickets",
        )?],
        vec![
            TaskEventRecord::new(
                durable_ids::event(durable_ids::EVENT_A)?,
                "the reconciliation started: both sources were queried",
            )?,
            TaskEventRecord::new(
                durable_ids::event(durable_ids::EVENT_B)?,
                "the ticket counts were verified against both sources",
            )?,
        ],
        vec![
            durable_memory(
                durable_ids::MEMORY_PLAN,
                Some(durable_ids::TASK),
                crate::MemoryTier::Hot,
                crate::AuthorizationClass::Task,
                MemoryContent::Text {
                    text: "current plan: reconcile the ticket counts between the dashboard and \
                           the CRM export"
                        .to_owned(),
                },
                "2026-09-21T13:45:00Z",
            )?,
            durable_memory(
                durable_ids::MEMORY_DECISION,
                Some(durable_ids::TASK),
                crate::MemoryTier::Warm,
                crate::AuthorizationClass::Task,
                MemoryContent::Text {
                    text: "decision: the CRM export is the source of truth for ticket counts"
                        .to_owned(),
                },
                "2026-09-21T13:44:00Z",
            )?,
            durable_memory(
                durable_ids::MEMORY_WORKSPACE,
                None,
                crate::MemoryTier::Warm,
                crate::AuthorizationClass::Workspace,
                MemoryContent::Text {
                    text: "workspace knowledge: the staging portal is read-only".to_owned(),
                },
                "2026-09-21T13:43:00Z",
            )?,
            durable_memory(
                durable_ids::MEMORY_ARCHIVE,
                Some(durable_ids::TASK),
                crate::MemoryTier::Cold,
                crate::AuthorizationClass::Task,
                MemoryContent::Reference {
                    reference: "flauz-archive://runs/2026-09-20/full-ticket-export".to_owned(),
                },
                "2026-09-21T13:42:00Z",
            )?,
            durable_memory(
                durable_ids::MEMORY_SECRET,
                Some(durable_ids::TASK),
                crate::MemoryTier::Hot,
                crate::AuthorizationClass::Secret,
                MemoryContent::Reference {
                    reference: "flauz-secrets://providers/example/api-key".to_owned(),
                },
                "2026-09-21T13:41:00Z",
            )?,
            durable_memory(
                durable_ids::MEMORY_OTHER_TASK,
                Some(durable_ids::OTHER_TASK),
                crate::MemoryTier::Hot,
                crate::AuthorizationClass::Task,
                MemoryContent::Text {
                    text: "current plan for the other task: ship the onboarding guide".to_owned(),
                },
                "2026-09-21T13:40:00Z",
            )?,
        ],
    )
}

/// The minimal (fresh task) durable state: a task, a session, and no
/// records or memory — the honest empty state a first compilation sees.
///
/// # Errors
///
/// Returns [`ContextStoreError::Invalid`] only on a fake-construction
/// bug.
pub fn fake_minimal_durable_state() -> Result<DurableStateInputs, crate::ContextError> {
    DurableStateInputs::new(
        durable_ids::task(durable_ids::TASK)?,
        None,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

/// The pressure-shaped durable state: six 60-byte memory items (two HOT,
/// four WARM), sized so the small-budget profile's 40-token compaction
/// budget retains exactly the two HOT items and summarizes the four WARM
/// items — the reference input of the compaction tests.
///
/// # Errors
///
/// Returns [`ContextStoreError::Invalid`] only on a fake-construction
/// bug.
pub fn fake_pressure_durable_state() -> Result<DurableStateInputs, crate::ContextError> {
    let hot = |id: &str, seed: &str| {
        durable_memory(
            id,
            Some(durable_ids::TASK),
            crate::MemoryTier::Hot,
            crate::AuthorizationClass::Task,
            MemoryContent::Text {
                text: pressure_text(seed),
            },
            "2026-09-21T13:45:00Z",
        )
    };
    let warm = |id: &str, seed: &str| {
        durable_memory(
            id,
            Some(durable_ids::TASK),
            crate::MemoryTier::Warm,
            crate::AuthorizationClass::Task,
            MemoryContent::Text {
                text: pressure_text(seed),
            },
            "2026-09-21T13:44:00Z",
        )
    };
    DurableStateInputs::new(
        durable_ids::task(durable_ids::TASK)?,
        Some(durable_ids::session()?),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            hot(
                durable_ids::MEMORY_PLAN,
                "current plan: reconcile the ticket counts",
            )?,
            hot(
                durable_ids::MEMORY_DECISION,
                "current error: the dashboard query timed out",
            )?,
            warm(
                durable_ids::MEMORY_WORKSPACE,
                "discovery: the CRM export needs the staging token",
            )?,
            warm(
                durable_ids::MEMORY_ARCHIVE,
                "question: which sync window does the export cover",
            )?,
            warm(
                durable_ids::MEMORY_SECRET,
                "decision: re-run the export after the nightly sync",
            )?,
            warm(
                durable_ids::MEMORY_OTHER_TASK,
                "discovery: the dashboard API paginates at fifty rows",
            )?,
        ],
    )
}

/// The small-budget model profile: a 40-token context capacity — the
/// compacting profile whose derived budget summarizes the pressure
/// state's WARM items.
///
/// # Errors
///
/// Returns [`ContextStoreError::Invalid`] only on a fake-construction
/// bug.
pub fn fake_small_budget_profile() -> Result<ModelContextProfile, crate::ContextError> {
    ModelContextProfile::new(
        ModelRef::parse(durable_ids::MODEL)?,
        40,
        crate::profile::MultimodalBehavior::TextOnly,
        crate::profile::ToolSchemaHandling::LazyPerTool,
        ActorRef::system("flauz-fake")?,
        Timestamp::parse("2026-09-21T13:45:00Z")?,
    )
}

/// The large-budget model profile: a 200_000-token context capacity — no
/// compaction is needed under its derived budget.
///
/// # Errors
///
/// Returns [`ContextStoreError::Invalid`] only on a fake-construction
/// bug.
pub fn fake_large_budget_profile() -> Result<ModelContextProfile, crate::ContextError> {
    ModelContextProfile::new(
        ModelRef::parse(durable_ids::MODEL)?,
        200_000,
        crate::profile::MultimodalBehavior::ImageInput,
        crate::profile::ToolSchemaHandling::SummariesWithLazySchemas,
        ActorRef::system("flauz-fake")?,
        Timestamp::parse("2026-09-21T13:45:00Z")?,
    )
}

/// The deterministic tool catalog: four tools (read files, run commands,
/// browse the web, view the screen) over three distinct capability
/// shapes — two admitted, one gap-blocked, one with no resolution
/// recorded at all (the named-exclusion case).
///
/// # Errors
///
/// Returns [`ContextStoreError::Invalid`] only on a fake-construction
/// bug.
pub fn fake_tool_catalog() -> Result<Vec<ToolDefinition>, crate::ContextError> {
    Ok(vec![
        ToolDefinition::new(
            "read_file",
            "filesystem.read",
            "{\"type\":\"object\",\"properties\":{\"path\":{\"type\":\"string\"}},\"required\":\
             [\"path\"]}",
            "reads one file from the workspace",
        )?,
        ToolDefinition::new(
            "run_command",
            "terminal",
            "{\"type\":\"object\",\"properties\":{\"command\":{\"type\":\"string\"}},\"required\":\
             [\"command\"]}",
            "runs one shell command in the task environment",
        )?,
        ToolDefinition::new(
            "browse_web",
            "browser.navigation",
            "{\"type\":\"object\",\"properties\":{\"url\":{\"type\":\"string\"}},\"required\":\
             [\"url\"]}",
            "opens one page in the task browser",
        )?,
        ToolDefinition::new(
            "view_screen",
            "computer.screen",
            "{\"type\":\"object\",\"properties\":{\"display\":{\"type\":\"integer\"}},\"required\":\
             [\"display\"]}",
            "captures the current screen of the task environment",
        )?,
    ])
}

/// The deterministic capability admissions for the fake tool catalog:
/// `filesystem.read` and `terminal` admitted; `browser.navigation`
/// unavailable with two named gaps (model and environment);
/// `computer.screen` deliberately unresolved — the exposure must name
/// its exclusion honestly instead of dropping it silently.
///
/// # Errors
///
/// Returns [`ContextStoreError::Invalid`] only on a fake-construction
/// bug.
pub fn fake_capability_admissions() -> Result<Vec<CapabilityAdmission>, crate::ContextError> {
    Ok(vec![
        CapabilityAdmission::new("filesystem.read", true, Vec::new())?,
        CapabilityAdmission::new("terminal", true, Vec::new())?,
        CapabilityAdmission::new(
            "browser.navigation",
            false,
            vec![
                crate::tools::AdmissionGap::new("model", "the model in use does not offer it")?,
                crate::tools::AdmissionGap::new(
                    "environment",
                    "the attached environment does not offer this capability",
                )?,
            ],
        )?,
    ])
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
