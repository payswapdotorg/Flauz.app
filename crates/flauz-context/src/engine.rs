//! The context engine: compilation from durable task state (Wave 3, work
//! order **ORCH-002**).
//!
//! [`compile_from_durable_state`] is the F6 engine step promised by the
//! architecture: given a task's **durable state** — its artifacts,
//! observations, evidence, recent events and memory items — it produces a
//! [`ContextSnapshot`]: the projection, never a transcript.
//!
//! # Inputs as data (the flauz-cap pattern)
//!
//! The engine takes [`DurableStateInputs`] — plain call-side data. It does
//! NOT import `flauz-world` or `flauz-exec`: foreign entities cross the
//! seam exclusively as frozen-format canonical-ID references (validated
//! local newtypes such as [`ArtifactRef`] and the locally defined
//! [`EvidenceRef`]) plus bounded summary text. No entity type of another
//! crate is re-defined here.
//!
//! # The rebuild law (no replay)
//!
//! Context is **rebuildable without replaying any model conversation**
//! (Wave-3 kernel addendum §2): two compilations from the same durable
//! state with no memory-item mutation yield **equivalent projections**
//! ([`projections_equivalent`]). The engine's only input is durable data —
//! there is no conversation, model output or transcript type anywhere in
//! its API, so reconstruction structurally cannot replay one. Each
//! compilation is a fresh immutable snapshot with its own `ctxsnap_` ID;
//! equivalence is projection-content equality, not snapshot-identity
//! equality.
//!
//! # Never mutates (the F2 law, engine level)
//!
//! Compilation takes `&DurableStateInputs` and `&ModelContextProfile` by
//! shared reference: compiling for model B cannot mutate any input. The
//! engine may consult, but never requires, prior snapshots; model/context
//! switching never mutates canonical task state.
//!
//! # Projection rules
//!
//! - **Memory items**: task-scoped and workspace-scoped items in the HOT
//!   and WARM tiers (COLD is retrieved just in time and therefore never
//!   auto-included) whose authorization class admits model context
//!   (Secret never compiles — kernel §7). Each projected item carries the
//!   memory item's own tier, content and authorization class, attributed
//!   to the memory item.
//! - **Observations** project at the HOT tier (current resource state)
//!   and **recent events, artifacts and evidence** at the WARM tier
//!   (recent conversation, recent artifacts — architecture §2).
//! - **Evidence** is attributed to its **verification event** (kernel §7:
//!   every evidence record references the event that verified it). The
//!   frozen ten-kind provenance vocabulary has no evidence kind, so the
//!   projection attributes the evidence item to the durable verification
//!   event it depends on — the honest mapping within the frozen
//!   vocabulary.
//! - Ordering is fully deterministic: memory items first (canonical
//!   memory-item ID order), then observations, recent events, artifacts
//!   and evidence (each in canonical entity-ID order), so the projection
//!   is independent of the order the caller lists records in.
//! - The projection is bounded by [`MAX_CONTEXT_ITEMS`](crate::
//!   MAX_CONTEXT_ITEMS): a durable state whose eligible projection
//!   exceeds the canonical item bound is rejected honestly, never
//!   silently truncated.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ids::{
    ArtifactRef, ContextSnapshotId, EventRef, IdError, ObservationRef, SessionRef, TaskRef,
};
use crate::memory::{MemoryContent, MemoryItem, MemoryTier};
use crate::profile::ModelContextProfile;
use crate::provenance::{AuthorizationClass, ContextProvenance, ContextSource};
use crate::refs::ActorRef;
use crate::snapshot::{ContextItem, ContextItemContent, ContextSnapshot};
use crate::time::Timestamp;
use crate::{ContextError, ContextVersion, MAX_STATEMENT_BYTES, ensure_non_empty};

/// A validated reference to a foreign Evidence entity (`evd_<ULID>`,
/// owned by flauz-world; kernel §2 registry).
///
/// The engine's durable-state inputs carry evidence records, so the
/// `evd_` kind is referenced through this local validating newtype — the
/// same frozen-format discipline as the other foreign references. The
/// Evidence entity itself is never defined here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EvidenceRef(String);

impl EvidenceRef {
    /// The frozen kind prefix of this identifier kind.
    pub const PREFIX: &'static str = "evd";

    /// Returns the canonical identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parses and validates a canonical `evd_<ULID>` identifier.
    ///
    /// # Errors
    ///
    /// Returns [`IdError`] when the identifier violates the frozen
    /// canonical-ID grammar or is not an `evd_` kind.
    pub fn parse(id: &str) -> Result<Self, IdError> {
        let kind = crate::ids::validate(id)?;
        if kind.prefix() != Self::PREFIX {
            return Err(IdError::KindMismatch {
                expected: Self::PREFIX,
                found: kind.prefix(),
            });
        }
        Ok(Self(id.to_owned()))
    }
}

impl fmt::Display for EvidenceRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for EvidenceRef {
    type Err = IdError;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::parse(id)
    }
}

impl Serialize for EvidenceRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for EvidenceRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// One durable artifact of a task, as call-side data: the canonical
/// artifact reference plus the bounded summary the projection inlines.
/// The flauz-world `Artifact` entity is never re-defined here — this is
/// the frozen-format projection input, the flauz-cap pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRecord {
    /// The canonical artifact reference (`art_<ULID>`).
    pub artifact_id: ArtifactRef,
    /// The bounded summary of the artifact's durable state.
    pub summary: String,
}

impl ArtifactRecord {
    /// Builds an artifact record, validating the summary bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary is empty or
    /// exceeds the canonical statement bound.
    pub fn new(artifact_id: ArtifactRef, summary: impl Into<String>) -> Result<Self, ContextError> {
        let summary = summary.into();
        ensure_summary("artifact summary", &summary)?;
        Ok(Self {
            artifact_id,
            summary,
        })
    }

    /// Validates the record.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary bounds fail.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.artifact_id.clone(), self.summary.clone())?;
        Ok(())
    }
}

/// One durable resource observation of a task, as call-side data: the
/// canonical observation reference plus its bounded summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationRecord {
    /// The canonical observation reference (`obs_<ULID>`).
    pub observation_id: ObservationRef,
    /// The bounded summary of the observation's durable state.
    pub summary: String,
}

impl ObservationRecord {
    /// Builds an observation record, validating the summary bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary is empty or
    /// exceeds the canonical statement bound.
    pub fn new(
        observation_id: ObservationRef,
        summary: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let summary = summary.into();
        ensure_summary("observation summary", &summary)?;
        Ok(Self {
            observation_id,
            summary,
        })
    }

    /// Validates the record.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary bounds fail.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.observation_id.clone(), self.summary.clone())?;
        Ok(())
    }
}

/// One verified evidence record of a task, as call-side data: the
/// canonical evidence reference, the verification event that made it
/// verified (kernel §7 — every evidence record references one), and the
/// bounded summary of what was verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    /// The canonical evidence reference (`evd_<ULID>`).
    pub evidence_id: EvidenceRef,
    /// The canonical verification-event reference (`ev_<ULID>`) — the
    /// durable event that verified the evidence.
    pub verification_event_id: EventRef,
    /// The bounded summary of the verified finding.
    pub summary: String,
}

impl EvidenceRecord {
    /// Builds an evidence record, validating the summary bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary is empty or
    /// exceeds the canonical statement bound.
    pub fn new(
        evidence_id: EvidenceRef,
        verification_event_id: EventRef,
        summary: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let summary = summary.into();
        ensure_summary("evidence summary", &summary)?;
        Ok(Self {
            evidence_id,
            verification_event_id,
            summary,
        })
    }

    /// Validates the record.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary bounds fail.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.evidence_id.clone(),
            self.verification_event_id.clone(),
            self.summary.clone(),
        )?;
        Ok(())
    }
}

/// One recent event from the task's event stream, as call-side data: the
/// canonical event reference plus its bounded summary. Events are
/// append-only and immutable (kernel §3); the engine projects the recent
/// tail the caller supplies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskEventRecord {
    /// The canonical event reference (`ev_<ULID>`).
    pub event_id: EventRef,
    /// The bounded summary of the event.
    pub summary: String,
}

impl TaskEventRecord {
    /// Builds a task-event record, validating the summary bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary is empty or
    /// exceeds the canonical statement bound.
    pub fn new(event_id: EventRef, summary: impl Into<String>) -> Result<Self, ContextError> {
        let summary = summary.into();
        ensure_summary("event summary", &summary)?;
        Ok(Self { event_id, summary })
    }

    /// Validates the record.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary bounds fail.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.event_id.clone(), self.summary.clone())?;
        Ok(())
    }
}

/// The durable state of one task, as data — the engine's sole input
/// family (the flauz-cap pattern: inputs, never entity imports).
///
/// The task is identified solely by its canonical `task_` ID (kernel §6);
/// the session is an optional reference a compilation happened in
/// (Session ≠ Context). Memory items may carry other tasks' scope — the
/// engine filters them out defensively rather than trusting the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurableStateInputs {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// The task whose durable state this is.
    pub task_id: TaskRef,
    /// The session the compilation happens in, if any.
    pub session_id: Option<SessionRef>,
    /// The task's durable artifacts, as data.
    pub artifacts: Vec<ArtifactRecord>,
    /// The task's durable resource observations, as data.
    pub observations: Vec<ObservationRecord>,
    /// The task's verified evidence, as data.
    pub evidence: Vec<EvidenceRecord>,
    /// The task's recent events, as data.
    pub recent_events: Vec<TaskEventRecord>,
    /// The memory items in scope (task-scoped and workspace-scoped).
    pub memory_items: Vec<MemoryItem>,
}

impl DurableStateInputs {
    /// Builds the durable-state inputs, validating every record, the
    /// per-family ID uniqueness (duplicates are rejected, never silently
    /// merged) and every memory item.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any record fails canonical
    /// validation or two records in one family share a canonical ID.
    pub fn new(
        task_id: TaskRef,
        session_id: Option<SessionRef>,
        artifacts: Vec<ArtifactRecord>,
        observations: Vec<ObservationRecord>,
        evidence: Vec<EvidenceRecord>,
        recent_events: Vec<TaskEventRecord>,
        memory_items: Vec<MemoryItem>,
    ) -> Result<Self, ContextError> {
        for record in &artifacts {
            record.validate()?;
        }
        for record in &observations {
            record.validate()?;
        }
        for record in &evidence {
            record.validate()?;
        }
        for record in &recent_events {
            record.validate()?;
        }
        for item in &memory_items {
            item.validate()?;
        }
        ensure_unique(
            "artifact",
            artifacts.iter().map(|record| record.artifact_id.as_str()),
        )?;
        ensure_unique(
            "observation",
            observations
                .iter()
                .map(|record| record.observation_id.as_str()),
        )?;
        ensure_unique(
            "evidence",
            evidence.iter().map(|record| record.evidence_id.as_str()),
        )?;
        ensure_unique(
            "event",
            recent_events.iter().map(|record| record.event_id.as_str()),
        )?;
        ensure_unique(
            "memory item",
            memory_items.iter().map(|item| item.id.as_str()),
        )?;
        Ok(Self {
            v: ContextVersion,
            task_id,
            session_id,
            artifacts,
            observations,
            evidence,
            recent_events,
            memory_items,
        })
    }

    /// Validates the durable-state inputs.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.task_id.clone(),
            self.session_id.clone(),
            self.artifacts.clone(),
            self.observations.clone(),
            self.evidence.clone(),
            self.recent_events.clone(),
            self.memory_items.clone(),
        )?;
        Ok(())
    }
}

/// Compiles a context snapshot from a task's durable state: the
/// projection, never a transcript (the rebuild law — see the module
/// docs).
///
/// The snapshot targets the profile's model (model identity is execution
/// state, never task identity — kernel §6), carries the caller-supplied
/// compiling actor and timestamp, and receives a freshly generated
/// `ctxsnap_` ID. Inputs are taken by shared reference and are never
/// mutated.
///
/// # Errors
///
/// Returns [`ContextError::Invalid`] when the inputs or the profile fail
/// canonical validation, or when the eligible projection exceeds the
/// canonical context-item bound (rejected honestly, never silently
/// truncated).
pub fn compile_from_durable_state(
    inputs: &DurableStateInputs,
    profile: &ModelContextProfile,
    compiled_by: ActorRef,
    compiled_at: Timestamp,
) -> Result<ContextSnapshot, ContextError> {
    inputs.validate()?;
    profile.validate()?;
    let items = project_items(inputs)?;
    ContextSnapshot::new(
        ContextSnapshotId::generate(),
        inputs.task_id.clone(),
        inputs.session_id.clone(),
        profile.model_id.clone(),
        compiled_by,
        compiled_at,
        items,
    )
}

/// The rebuild law's equivalence relation: two snapshots are equivalent
/// projections when they agree on the task, the session, the target
/// model, the snapshot version and every item — everything except the
/// compilation metadata (the snapshot ID, the compiling actor and the
/// compilation timestamp, each of which records a fresh durable
/// compilation event).
///
/// Two compilations from the same durable state with no memory-item
/// mutation yield equivalent projections; a compilation with a different
/// target model, task or items does not.
#[must_use]
pub fn projections_equivalent(a: &ContextSnapshot, b: &ContextSnapshot) -> bool {
    a.task_id == b.task_id
        && a.session_id == b.session_id
        && a.model_id == b.model_id
        && a.version == b.version
        && a.items == b.items
}

/// Projects the eligible items from the durable state (see the module
/// docs for the selection and ordering rules).
fn project_items(inputs: &DurableStateInputs) -> Result<Vec<ContextItem>, ContextError> {
    let mut items = Vec::new();

    // Memory items: task-scoped + workspace-scoped, HOT/WARM (COLD is
    // jit-only), authorization-eligible (Secret never compiles), in
    // canonical memory-item ID order.
    let mut memory: Vec<&MemoryItem> = inputs
        .memory_items
        .iter()
        .filter(|item| {
            (item.task_id.as_ref() == Some(&inputs.task_id) || item.task_id.is_none())
                && item.tier.eligible_on_reset()
                && item.eligible_for_model_context()
        })
        .collect();
    memory.sort_by(|a, b| a.id.cmp(&b.id));
    for item in memory {
        items.push(memory_item_context(item)?);
    }

    // Observations: the current resource state — HOT tier.
    let mut observations: Vec<&ObservationRecord> = inputs.observations.iter().collect();
    observations.sort_by(|a, b| a.observation_id.cmp(&b.observation_id));
    for record in observations {
        items.push(ContextItem::new(
            ContextItemContent::Text {
                text: record.summary.clone(),
            },
            MemoryTier::Hot,
            ContextProvenance::new(
                ContextSource::ResourceObservation {
                    observation_id: record.observation_id.clone(),
                },
                AuthorizationClass::Task,
            )?,
        )?);
    }

    // Recent events: WARM tier (recent conversation), session-event
    // provenance.
    let mut events: Vec<&TaskEventRecord> = inputs.recent_events.iter().collect();
    events.sort_by(|a, b| a.event_id.cmp(&b.event_id));
    for record in events {
        items.push(ContextItem::new(
            ContextItemContent::Text {
                text: record.summary.clone(),
            },
            MemoryTier::Warm,
            ContextProvenance::new(
                ContextSource::SessionEvent {
                    event_id: record.event_id.clone(),
                },
                AuthorizationClass::Task,
            )?,
        )?);
    }

    // Artifacts: WARM tier (recent artifacts).
    let mut artifacts: Vec<&ArtifactRecord> = inputs.artifacts.iter().collect();
    artifacts.sort_by(|a, b| a.artifact_id.cmp(&b.artifact_id));
    for record in artifacts {
        items.push(ContextItem::new(
            ContextItemContent::Text {
                text: record.summary.clone(),
            },
            MemoryTier::Warm,
            ContextProvenance::new(
                ContextSource::Artifact {
                    artifact_id: record.artifact_id.clone(),
                },
                AuthorizationClass::Task,
            )?,
        )?);
    }

    // Evidence: WARM tier, attributed to the verification event that made
    // it verified (kernel §7) — the honest mapping within the frozen
    // ten-kind provenance vocabulary.
    let mut evidence: Vec<&EvidenceRecord> = inputs.evidence.iter().collect();
    evidence.sort_by(|a, b| a.evidence_id.cmp(&b.evidence_id));
    for record in evidence {
        items.push(ContextItem::new(
            ContextItemContent::Text {
                text: record.summary.clone(),
            },
            MemoryTier::Warm,
            ContextProvenance::new(
                ContextSource::SessionEvent {
                    event_id: record.verification_event_id.clone(),
                },
                AuthorizationClass::Task,
            )?,
        )?);
    }

    Ok(items)
}

/// Projects one memory item as a context item: the item's own content,
/// tier and authorization class, attributed to the memory item.
fn memory_item_context(item: &MemoryItem) -> Result<ContextItem, ContextError> {
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
}

/// Validates a record summary: non-empty and within the canonical
/// statement bound.
fn ensure_summary(field: &'static str, summary: &str) -> Result<(), ContextError> {
    ensure_non_empty(field, summary)?;
    if summary.len() > MAX_STATEMENT_BYTES {
        return Err(ContextError::invalid(format!(
            "{field} exceeds {MAX_STATEMENT_BYTES} bytes"
        )));
    }
    Ok(())
}

/// Rejects duplicate canonical IDs within one record family — duplicates
/// are caller errors, never silently merged (kernel §7's no-silent-dedup
/// discipline).
fn ensure_unique<'a>(
    family: &'static str,
    ids: impl Iterator<Item = &'a str>,
) -> Result<(), ContextError> {
    let mut seen: Vec<&str> = ids.collect();
    seen.sort_unstable();
    for pair in seen.windows(2) {
        if pair[0] == pair[1] {
            return Err(ContextError::invalid(format!(
                "duplicate {family} record for canonical id {}",
                pair[0]
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelRef;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn evidence_refs_parse_the_frozen_evd_kind() {
        let reference = ok(EvidenceRef::parse("evd_01J8ZQ5V8K3T2B7N6X4R9DQPK9"));
        assert_eq!(reference.as_str(), "evd_01J8ZQ5V8K3T2B7N6X4R9DQPK9");
        // Wrong kinds and malformed IDs are rejected.
        assert!(EvidenceRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1").is_err());
        assert!(EvidenceRef::parse("evd_01J8ZQ5V8K3T2B7N6X4R9DQPK").is_err());
        assert!(EvidenceRef::parse("evd_01J8ZQ5V8K3T2B7N6X4R9DQPKO").is_err());
        // Serialization is the plain canonical string.
        let serialized = ok(serde_json::to_string(&reference));
        assert_eq!(serialized, "\"evd_01J8ZQ5V8K3T2B7N6X4R9DQPK9\"");
        let reloaded: EvidenceRef = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, reference);
    }

    #[test]
    fn durable_state_round_trips_and_rejects_unknown_fields() {
        let inputs = ok(DurableStateInputs::new(
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            Some(ok(SessionRef::parse("sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6"))),
            vec![ok(ArtifactRecord::new(
                ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPH7")),
                "the CRM ticket export: 3 open tickets",
            ))],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));
        let serialized = ok(serde_json::to_string(&inputs));
        let reloaded: DurableStateInputs = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, inputs);
        assert!(
            serde_json::from_str::<DurableStateInputs>(&serialized.replace(
                "\"task_id\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\"",
                "\"task_id\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\",\"totally_unknown\":true"
            ))
            .is_err(),
            "unknown fields must be rejected, not ignored"
        );
    }

    #[test]
    fn duplicate_records_within_a_family_are_rejected() {
        let record = ok(TaskEventRecord::new(
            ok(EventRef::parse("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
            "the reconciliation started",
        ));
        assert!(
            DurableStateInputs::new(
                ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![record.clone(), record],
                Vec::new(),
            )
            .is_err(),
            "duplicate event records are caller errors, never silently merged"
        );
    }

    #[test]
    fn minimal_durable_state_compiles_an_empty_projection() {
        let inputs = ok(DurableStateInputs::new(
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));
        let profile = ok(ModelContextProfile::new(
            ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE")),
            8_192,
            crate::profile::MultimodalBehavior::TextOnly,
            crate::profile::ToolSchemaHandling::InlineFullSchemas,
            ok(ActorRef::system("flauz-fake")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        ));
        let snapshot = ok(compile_from_durable_state(
            &inputs,
            &profile,
            ok(ActorRef::system("flauz-context-engine")),
            ok(Timestamp::parse("2026-09-21T13:46:00Z")),
        ));
        assert!(snapshot.items.is_empty());
        ok(snapshot.validate());
    }
}
