//! Tier dynamics and the structured compaction planner (Wave 3, work
//! order **ORCH-002**).
//!
//! # Tier dynamics
//!
//! Memory tiers are not static (architecture §2): items are **promoted**
//! and **demoted** by deterministic rules, as pure state transitions over
//! [`MemoryItem`] values:
//!
//! - **HOT stays bounded** ([`MAX_HOT_MEMORY_ITEMS`]): promoting into HOT
//!   beyond the bound demotes the *oldest* other HOT item (by
//!   `created_at`, ties by canonical ID) to WARM
//!   ([`promote_memory_item`], [`enforce_hot_memory_bound`]).
//! - **WARM holds referenced items** ([`age_out_unreferenced_warm`]):
//!   WARM items that are no longer referenced demote to COLD.
//! - **COLD is jit-only**: COLD items are never auto-included in a fresh
//!   compilation (enforced by
//!   [`MemoryTier::eligible_on_reset`](crate::MemoryTier) and the engine);
//!   referencing a COLD item promotes it to WARM first.
//! - **Secret never compiles**: tier transitions never change an item's
//!   authorization class — a Secret item promoted all the way to HOT is
//!   still excluded from every compiled projection (kernel §7).
//!
//! The transitions are *planners*: they return new item values with the
//! tier changed and the **version untouched** — the durable version bump
//! (+1 per mutation, kernel §3) happens when the caller persists the
//! returned items through the context store's optimistic-concurrency
//! update path. Inputs are taken by shared reference and are never
//! mutated.
//!
//! # Structured compaction (architecture §12)
//!
//! COMPACTION **preserves continuity while shrinking** — it is distinct
//! from RESET (which reconstructs a fresh context from durable task
//! state). [`plan_compaction`] compacts a snapshot within a budget
//! ([`CompactionBudget`], derived per
//! [`ModelContextProfile`](crate::ModelContextProfile)):
//!
//! - every **retained** item keeps its provenance, verbatim;
//! - every **summarized** item is **NAMED** in the compaction record
//!   ([`CompactionRecord`]) — its provenance, its tier and a bounded
//!   deterministic summary. Compaction never silently drops: retained +
//!   summarized always partitions the input snapshot's items;
//! - the compacted snapshot is a **new** immutable `ctxsnap_` entity; the
//!   superseded snapshot is retained unmutated (non-destructive — the
//!   named items stay fully readable there);
//! - retention is priority-ordered (HOT before WARM before COLD — the
//!   structured state of architecture §12 survives first) and the
//!   retained set preserves the snapshot's original item order;
//! - the token budget is enforced with a deterministic integer estimate
//!   ([`estimate_tokens`]) — no floats (kernel §4), no model calls.
//!
//! The compaction record is the compaction counterpart of
//! [`ContextReset`](crate::ContextReset): the Wave-3 kernel addendum §1
//! rule keeps the frozen `ResetReason` vocabulary untouched (its kinds —
//! manual, pressure-as-reset, model/environment changed, recovery —
//! describe RESET, not COMPACTION), so compaction introduces its **own**
//! record family and reason kinds ([`CompactionReason`]) as new types,
//! never a mutation of the existing ones.
//!
//! # Determinism (kernel §7)
//!
//! Everything here is a pure function of its inputs: no wall clock, no
//! randomness (the only generated value is the compacted snapshot's
//! `ctxsnap_` ID, from the crate's private ulid module), no I/O.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ids::{ContextSnapshotId, MemoryItemId, TaskRef};
use crate::memory::{MemoryItem, MemoryTier};
use crate::profile::ModelContextProfile;
use crate::provenance::ContextProvenance;
use crate::refs::ActorRef;
use crate::snapshot::{ContextItem, ContextItemContent, ContextSnapshot};
use crate::time::Timestamp;
use crate::{
    ContextError, ContextVersion, MAX_CONTEXT_ITEMS, MAX_MODEL_TOKENS, MIN_MODEL_TOKENS,
    ensure_non_empty,
};

/// Maximum number of HOT-tier memory items the tier dynamics retain:
/// promoting beyond this bound demotes the oldest HOT items to WARM.
pub const MAX_HOT_MEMORY_ITEMS: usize = 16;

/// Maximum length of a compaction summary, in characters. The
/// deterministic summarizer truncates to this bound.
pub const MAX_SUMMARY_CHARS: usize = 120;

/// Bytes per estimated token for inline text content (a conservative
/// ASCII heuristic; integer math only — no floats, kernel §4).
pub const TEXT_BYTES_PER_TOKEN: u64 = 4;

/// Flat estimated token cost of one reference-content item.
pub const REFERENCE_TOKEN_COST: u64 = 8;

/// Why a compaction happened: the compaction record's reason kinds (NEW
/// kind types per the Wave-3 kernel addendum §1 rule — the frozen
/// `ResetReason` vocabulary stays untouched, so compaction carries its
/// own reason family).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompactionReason {
    /// Context pressure rose; the snapshot was compacted within the
    /// model profile's budget while preserving continuity.
    Pressure,
    /// A participant asked for a compacted context.
    Manual,
}

/// The compaction budget: the bounds a compacted snapshot must respect.
/// Derived per [`ModelContextProfile`] ([`CompactionBudget::from_profile`])
/// or constructed directly for tighter caller budgets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactionBudget {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// Maximum number of retained items.
    pub max_items: u64,
    /// Maximum estimated tokens across retained items.
    pub max_tokens: u64,
}

impl CompactionBudget {
    /// Builds a budget, validating both bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when `max_items` is outside
    /// `1..=MAX_CONTEXT_ITEMS` or `max_tokens` is outside the canonical
    /// model-token bounds.
    pub fn new(max_items: u64, max_tokens: u64) -> Result<Self, ContextError> {
        if max_items == 0 || max_items > MAX_CONTEXT_ITEMS as u64 {
            return Err(ContextError::invalid(format!(
                "compaction budget max_items {max_items} is outside 1..={MAX_CONTEXT_ITEMS}"
            )));
        }
        if !(MIN_MODEL_TOKENS..=MAX_MODEL_TOKENS).contains(&max_tokens) {
            return Err(ContextError::invalid(format!(
                "compaction budget max_tokens {max_tokens} is outside the canonical model token \
                 bounds {MIN_MODEL_TOKENS}..={MAX_MODEL_TOKENS}"
            )));
        }
        Ok(Self {
            v: ContextVersion,
            max_items,
            max_tokens,
        })
    }

    /// Derives the budget from a model context profile: the profile's
    /// context capacity in tokens and the canonical context-item bound.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the profile fails canonical
    /// validation.
    pub fn from_profile(profile: &ModelContextProfile) -> Result<Self, ContextError> {
        profile.validate()?;
        Self::new(MAX_CONTEXT_ITEMS as u64, profile.context_capacity_tokens)
    }

    /// Validates the budget's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when either bound is outside its
    /// canonical range.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.max_items, self.max_tokens)?;
        Ok(())
    }
}

/// One summarized (not retained) item, **NAMED**: the provenance and tier
/// it carried in the superseded snapshot plus its bounded deterministic
/// summary. The item itself remains fully readable in the superseded
/// snapshot — compaction is non-destructive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummarizedItem {
    /// The provenance of the summarized item — attribution is retained
    /// even for summarized content (compaction never silently drops).
    pub provenance: ContextProvenance,
    /// The tier the item was in.
    pub tier: MemoryTier,
    /// The bounded deterministic summary of the item's content.
    pub summary: String,
}

impl SummarizedItem {
    /// Builds one summarized-item record, validating the summary bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the summary is empty or
    /// exceeds [`MAX_SUMMARY_CHARS`] characters.
    pub fn new(
        provenance: ContextProvenance,
        tier: MemoryTier,
        summary: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let summary = summary.into();
        provenance.validate()?;
        ensure_non_empty("compaction summary", &summary)?;
        if summary.chars().count() > MAX_SUMMARY_CHARS {
            return Err(ContextError::invalid(format!(
                "compaction summary exceeds {MAX_SUMMARY_CHARS} characters"
            )));
        }
        Ok(Self {
            provenance,
            tier,
            summary,
        })
    }

    /// Validates the record.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the provenance or the
    /// summary bounds fail.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.provenance.clone(), self.tier, self.summary.clone())?;
        Ok(())
    }
}

/// The structured compaction record (architecture §12): the compaction
/// counterpart of [`ContextReset`](crate::ContextReset) — a NEW kind
/// type, so the frozen `ResetReason` vocabulary stays untouched. Durable
/// and attributable like every other context decision.
///
/// The record names every summarized item, the budget that drove the
/// compaction, and the snapshot it supersedes. It does not name the
/// compacted snapshot: like a reset, the caller receives both the record
/// and the fresh snapshot together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactionRecord {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// The task whose context was compacted (identified solely by its
    /// canonical `task_` ID — kernel §6).
    pub task_id: TaskRef,
    /// Why the context was compacted.
    pub reason: CompactionReason,
    /// The participant or component that performed the compaction.
    pub performed_by: ActorRef,
    /// When the compaction happened (caller-supplied).
    pub performed_at: Timestamp,
    /// The snapshot the compacted snapshot supersedes. The superseded
    /// snapshot is retained unmutated.
    pub superseded_snapshot_id: ContextSnapshotId,
    /// The budget that drove the compaction.
    pub budget: CompactionBudget,
    /// Every summarized item, NAMED — never silently dropped.
    pub summarized: Vec<SummarizedItem>,
}

impl CompactionRecord {
    /// Builds a compaction record, validating every field and record.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any summarized record or
    /// the budget fails canonical validation.
    pub fn new(
        task_id: TaskRef,
        reason: CompactionReason,
        performed_by: ActorRef,
        performed_at: Timestamp,
        superseded_snapshot_id: ContextSnapshotId,
        budget: CompactionBudget,
        summarized: Vec<SummarizedItem>,
    ) -> Result<Self, ContextError> {
        budget.validate()?;
        for item in &summarized {
            item.validate()?;
        }
        Ok(Self {
            v: ContextVersion,
            task_id,
            reason,
            performed_by,
            performed_at,
            superseded_snapshot_id,
            budget,
            summarized,
        })
    }

    /// Validates the compaction record.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.task_id.clone(),
            self.reason,
            self.performed_by.clone(),
            self.performed_at,
            self.superseded_snapshot_id.clone(),
            self.budget.clone(),
            self.summarized.clone(),
        )?;
        Ok(())
    }
}

/// The result of a compaction: the compacted snapshot (every retained
/// item keeps its provenance) and the compaction record naming every
/// summarized item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compaction {
    /// The compacted snapshot: retained items only, in the superseded
    /// snapshot's original order. A new `ctxsnap_` entity; the superseded
    /// snapshot is never mutated.
    pub snapshot: ContextSnapshot,
    /// The compaction record naming every summarized item.
    pub record: CompactionRecord,
}

/// Promotes one memory item one tier (COLD → WARM, WARM → HOT), keeping
/// HOT bounded: when HOT would exceed [`MAX_HOT_MEMORY_ITEMS`], the
/// oldest *other* HOT items (by `created_at`, ties by canonical ID) demote
/// to WARM — the item the caller just promoted is protected.
///
/// The returned items preserve the input order; only tiers change — the
/// version bump belongs to the store's persist path (kernel §3). Inputs
/// are never mutated.
///
/// # Errors
///
/// Returns [`ContextError::Invalid`] when the item is not found or is
/// already HOT (the ladder has no step above HOT).
pub fn promote_memory_item(
    items: &[MemoryItem],
    memory_item_id: &MemoryItemId,
) -> Result<Vec<MemoryItem>, ContextError> {
    let mut result = items.to_vec();
    let Some(item) = result.iter_mut().find(|item| &item.id == memory_item_id) else {
        return Err(ContextError::invalid(format!(
            "memory item {memory_item_id} not found"
        )));
    };
    match item.tier {
        MemoryTier::Hot => {
            return Err(ContextError::invalid(format!(
                "memory item {memory_item_id} is already HOT"
            )));
        }
        MemoryTier::Warm => item.tier = MemoryTier::Hot,
        MemoryTier::Cold => item.tier = MemoryTier::Warm,
    }
    demote_oldest_hot(&mut result, Some(memory_item_id))?;
    Ok(result)
}

/// Demotes one memory item one tier (HOT → WARM, WARM → COLD; COLD is
/// the end of the ladder). The returned items preserve the input order;
/// only the tier changes; inputs are never mutated.
///
/// # Errors
///
/// Returns [`ContextError::Invalid`] when the item is not found or is
/// already COLD.
pub fn demote_memory_item(
    items: &[MemoryItem],
    memory_item_id: &MemoryItemId,
) -> Result<Vec<MemoryItem>, ContextError> {
    let mut result = items.to_vec();
    let Some(item) = result.iter_mut().find(|item| &item.id == memory_item_id) else {
        return Err(ContextError::invalid(format!(
            "memory item {memory_item_id} not found"
        )));
    };
    match item.tier {
        MemoryTier::Hot => item.tier = MemoryTier::Warm,
        MemoryTier::Warm => item.tier = MemoryTier::Cold,
        MemoryTier::Cold => {
            return Err(ContextError::invalid(format!(
                "memory item {memory_item_id} is already COLD"
            )));
        }
    }
    Ok(result)
}

/// Enforces the HOT bound over a memory set: demotes the oldest HOT
/// items to WARM until at most [`MAX_HOT_MEMORY_ITEMS`] remain. The
/// returned items preserve the input order; inputs are never mutated.
///
/// # Errors
///
/// Returns [`ContextError::Invalid`] when the bound cannot be enforced
/// (structurally unreachable with a positive bound and no protected
/// item).
pub fn enforce_hot_memory_bound(items: &[MemoryItem]) -> Result<Vec<MemoryItem>, ContextError> {
    let mut result = items.to_vec();
    demote_oldest_hot(&mut result, None)?;
    Ok(result)
}

/// WARM holds referenced items: WARM items whose IDs are not in
/// `referenced` demote to COLD (just-in-time only). HOT and COLD items
/// are untouched. The returned items preserve the input order; inputs
/// are never mutated.
///
/// # Errors
///
/// Currently cannot fail; the `Result` keeps the family signature
/// honest for future canonical checks.
pub fn age_out_unreferenced_warm(
    items: &[MemoryItem],
    referenced: &BTreeSet<MemoryItemId>,
) -> Result<Vec<MemoryItem>, ContextError> {
    let mut result = items.to_vec();
    for item in &mut result {
        if item.tier == MemoryTier::Warm && !referenced.contains(&item.id) {
            item.tier = MemoryTier::Cold;
        }
    }
    Ok(result)
}

/// A deterministic, dependency-free token estimate for one context item:
/// inline text at [`TEXT_BYTES_PER_TOKEN`] bytes per token (plus one),
/// reference content at a flat [`REFERENCE_TOKEN_COST`]. Integer math
/// only — no floats (kernel §4), no model calls.
#[must_use]
pub fn estimate_tokens(item: &ContextItem) -> u64 {
    match &item.content {
        ContextItemContent::Text { text } => text.len() as u64 / TEXT_BYTES_PER_TOKEN + 1,
        ContextItemContent::Reference { reference } => {
            REFERENCE_TOKEN_COST + reference.len() as u64 / (TEXT_BYTES_PER_TOKEN * 2)
        }
    }
}

/// Plans the structured compaction of a snapshot within a budget: the
/// compacted snapshot (retained items, each keeping its provenance, in
/// the original order) and the compaction record naming every summarized
/// item.
///
/// Retention is priority-ordered — HOT before WARM before COLD (within a
/// tier, the snapshot's own order) — and the walk stops at the first
/// item that does not fit the remaining budget: a deterministic,
/// explainable greedy. A budget so tight that even the highest-priority
/// item does not fit summarizes everything and retains nothing — the
/// record honestly names it all, and the superseded snapshot (retained
/// unmutated) keeps the full content.
///
/// # Errors
///
/// Returns [`ContextError::Invalid`] when the snapshot or the budget
/// fails canonical validation.
pub fn plan_compaction(
    snapshot: &ContextSnapshot,
    budget: &CompactionBudget,
    reason: CompactionReason,
    performed_by: ActorRef,
    performed_at: Timestamp,
) -> Result<Compaction, ContextError> {
    snapshot.validate()?;
    budget.validate()?;

    // Rank by tier priority, stable within a tier.
    let mut ranked: Vec<usize> = (0..snapshot.items.len()).collect();
    ranked.sort_by_key(|&index| (tier_priority(snapshot.items[index].tier), index));

    // Priority-ordered greedy retention; stop at the first item that
    // does not fit.
    let mut retain = vec![false; snapshot.items.len()];
    let mut retained_count = 0usize;
    let mut retained_tokens = 0u64;
    for &index in &ranked {
        let estimate = estimate_tokens(&snapshot.items[index]);
        let next_tokens = retained_tokens.saturating_add(estimate);
        if retained_count < budget.max_items as usize && next_tokens <= budget.max_tokens {
            retain[index] = true;
            retained_count += 1;
            retained_tokens = next_tokens;
        } else {
            break;
        }
    }

    // The retained set preserves the snapshot's original item order.
    let retained: Vec<ContextItem> = snapshot
        .items
        .iter()
        .zip(&retain)
        .filter(|(_, retain)| **retain)
        .map(|(item, _)| item.clone())
        .collect();

    // Every summarized item is NAMED: provenance + tier + a bounded
    // deterministic summary.
    let summarized: Vec<SummarizedItem> = snapshot
        .items
        .iter()
        .zip(&retain)
        .filter(|(_, retain)| !**retain)
        .map(|(item, _)| {
            SummarizedItem::new(item.provenance.clone(), item.tier, summarize_item(item))
        })
        .collect::<Result<_, _>>()?;

    let compacted = ContextSnapshot::new(
        ContextSnapshotId::generate(),
        snapshot.task_id.clone(),
        snapshot.session_id.clone(),
        snapshot.model_id.clone(),
        performed_by.clone(),
        performed_at,
        retained,
    )?;
    let record = CompactionRecord::new(
        snapshot.task_id.clone(),
        reason,
        performed_by,
        performed_at,
        snapshot.id.clone(),
        budget.clone(),
        summarized,
    )?;
    Ok(Compaction {
        snapshot: compacted,
        record,
    })
}

/// The retention priority of one tier: HOT first, then WARM, then COLD.
const fn tier_priority(tier: MemoryTier) -> u8 {
    match tier {
        MemoryTier::Hot => 0,
        MemoryTier::Warm => 1,
        MemoryTier::Cold => 2,
    }
}

/// Demotes the oldest HOT items to WARM until at most `bound` remain,
/// skipping the protected item (the one a promotion just touched).
fn demote_oldest_hot(
    items: &mut [MemoryItem],
    protected: Option<&MemoryItemId>,
) -> Result<(), ContextError> {
    loop {
        let hot_count = items
            .iter()
            .filter(|item| item.tier == MemoryTier::Hot)
            .count();
        if hot_count <= MAX_HOT_MEMORY_ITEMS {
            return Ok(());
        }
        let candidate = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.tier == MemoryTier::Hot && protected.is_none_or(|id| &item.id != id)
            })
            .min_by(|(_, a), (_, b)| {
                (a.created_at, a.id.as_str()).cmp(&(b.created_at, b.id.as_str()))
            })
            .map(|(index, _)| index);
        match candidate {
            Some(index) => items[index].tier = MemoryTier::Warm,
            None => {
                return Err(ContextError::invalid(
                    "the HOT bound cannot be enforced: every HOT item is protected",
                ));
            }
        }
    }
}

/// Produces the bounded deterministic summary for one summarized item:
/// the first line of text content (or the reference string), truncated to
/// [`MAX_SUMMARY_CHARS`] characters. No model calls, no entropy.
fn summarize_item(item: &ContextItem) -> String {
    let source = match &item.content {
        ContextItemContent::Text { text } => text.lines().next().unwrap_or_default(),
        ContextItemContent::Reference { reference } => reference.as_str(),
    };
    let summary: String = source.chars().take(MAX_SUMMARY_CHARS).collect();
    if summary.trim().is_empty() {
        "(no textual summary)".to_owned()
    } else {
        summary
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
    fn budgets_validate_bounds() {
        ok(CompactionBudget::new(256, 512));
        assert!(CompactionBudget::new(0, 512).is_err());
        assert!(CompactionBudget::new(257, 512).is_err());
        assert!(CompactionBudget::new(256, 0).is_err());
        let serialized = ok(serde_json::to_string(&ok(CompactionBudget::new(256, 512))));
        assert_eq!(serialized, "{\"v\":1,\"max_items\":256,\"max_tokens\":512}");
        let reloaded: CompactionBudget = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, ok(CompactionBudget::new(256, 512)));
    }

    #[test]
    fn compaction_reasons_serialize_snake_case() {
        assert_eq!(
            ok(serde_json::to_string(&CompactionReason::Pressure)),
            "\"pressure\""
        );
        assert_eq!(
            ok(serde_json::to_string(&CompactionReason::Manual)),
            "\"manual\""
        );
        assert!(
            serde_json::from_str::<CompactionReason>("\"reset\"").is_err(),
            "compaction reasons are a NEW kind family, never ResetReason mutations"
        );
    }

    #[test]
    fn estimates_are_integer_and_deterministic() {
        let text = ok(ContextItem::new(
            crate::snapshot::ContextItemContent::Text {
                text: "x".repeat(100),
            },
            MemoryTier::Hot,
            ok(crate::provenance::ContextProvenance::new(
                crate::provenance::ContextSource::UserInput,
                crate::provenance::AuthorizationClass::Task,
            )),
        ));
        assert_eq!(estimate_tokens(&text), 26);
        let reference = ok(ContextItem::new(
            crate::snapshot::ContextItemContent::Reference {
                reference: "mem_01J8ZQ5V8K3T2B7N6X4R9DQPF5".to_owned(),
            },
            MemoryTier::Warm,
            ok(crate::provenance::ContextProvenance::new(
                crate::provenance::ContextSource::UserInput,
                crate::provenance::AuthorizationClass::Task,
            )),
        ));
        assert_eq!(estimate_tokens(&reference), 11);
    }

    #[test]
    fn summarized_summaries_are_bounded_deterministic_digests() {
        let item = ok(ContextItem::new(
            crate::snapshot::ContextItemContent::Text {
                text: "first line of a much longer story\nsecond line".to_owned(),
            },
            MemoryTier::Warm,
            ok(crate::provenance::ContextProvenance::new(
                crate::provenance::ContextSource::UserInput,
                crate::provenance::AuthorizationClass::Task,
            )),
        ));
        let long = ok(ContextItem::new(
            crate::snapshot::ContextItemContent::Text {
                text: "y".repeat(500),
            },
            MemoryTier::Warm,
            ok(crate::provenance::ContextProvenance::new(
                crate::provenance::ContextSource::UserInput,
                crate::provenance::AuthorizationClass::Task,
            )),
        ));
        assert_eq!(summarize_item(&item), "first line of a much longer story");
        assert_eq!(summarize_item(&long).chars().count(), MAX_SUMMARY_CHARS);
        assert_eq!(summarize_item(&long), "y".repeat(MAX_SUMMARY_CHARS));
        // Summaries longer than the bound are invalid records.
        assert!(
            SummarizedItem::new(
                ok(crate::provenance::ContextProvenance::new(
                    crate::provenance::ContextSource::UserInput,
                    crate::provenance::AuthorizationClass::Task,
                )),
                MemoryTier::Warm,
                "z".repeat(MAX_SUMMARY_CHARS + 1),
            )
            .is_err()
        );
    }
}
