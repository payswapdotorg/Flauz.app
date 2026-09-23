//! The harness telemetry projection (ORCH-003, F6 Wave 3): a bounded,
//! canonical-JSON serializable trace of a harness run.
//!
//! The trace is a **projection** of the machine's own history (the
//! [`HarnessEventRecord`] stream) — never of model output. It answers
//! the harness-observability questions the architecture records
//! (CONTEXT-HARNESS-ARCHITECTURE §15): how long each phase held, which
//! turns completed and which reported capability gaps, how recoveries
//! and escalations went, and how the run ended.
//!
//! # Determinism
//!
//! [`HarnessTrace::project`] reads only the records: timestamps are the
//! caller-supplied ones the machine already recorded, durations are
//! integer milliseconds (kernel §4 — no floats), and no clock or
//! randomness is consulted. Projecting the same history twice yields
//! the same trace.
//!
//! # Replayable into a display model
//!
//! The trace round-trips through canonical JSON (serialize → drop →
//! reload → equality, the round-trip law), so an activity feed or a
//! run inspector can rebuild the display model from the reloaded trace
//! alone (J-17: the harness events surface in the activity feed).
//!
//! # Canonical JSON (kernel §4)
//!
//! Every serialized type carries `"v": 1`, uses snake_case fields,
//! rejects unknown fields, contains no floats, and emits timestamps as
//! RFC 3339 UTC `YYYY-MM-DDTHH:MM:SSZ`.

use serde::{Deserialize, Serialize};

use crate::ContractVersion;
use crate::harness::{
    HarnessError, HarnessEventDetail, HarnessEventRecord, HarnessStateKind, replay_history,
};
use crate::time::Timestamp;

/// The maximum number of spans one trace may carry. A longer history is
/// rejected: the caller splits runs (a task's full history lives on the
/// event stream; the trace is a bounded projection of one run).
pub const MAX_HARNESS_TRACE_SPANS: usize = 256;

/// The outcome classification of one trace span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessSpanOutcome {
    /// A phase transition within a running task.
    Phase,
    /// A turn completed.
    TurnCompleted,
    /// A turn reported a capability gap (the named missing keys are on
    /// the record).
    TurnCapabilityGap,
    /// The run resumed from its own history (recovery).
    Recovered,
    /// The run needs the human (escalation).
    Escalated,
    /// The run completed.
    Completed,
    /// The run failed.
    Failed,
}

impl HarnessSpanOutcome {
    /// Every outcome, in declaration order.
    pub const ALL: [Self; 7] = [
        Self::Phase,
        Self::TurnCompleted,
        Self::TurnCapabilityGap,
        Self::Recovered,
        Self::Escalated,
        Self::Completed,
        Self::Failed,
    ];

    /// The canonical (serialized) name of this outcome.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Phase => "phase",
            Self::TurnCompleted => "turn_completed",
            Self::TurnCapabilityGap => "turn_capability_gap",
            Self::Recovered => "recovered",
            Self::Escalated => "escalated",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// One span of the trace: a transition, when it happened, how long the
/// machine held the resulting state, and the outcome classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessTraceSpan {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The record's sequence on the harness stream.
    pub seq: u64,
    /// The registered `event_type` of the transition.
    pub event_type: String,
    /// The state the transition moved the machine into.
    pub to_state: HarnessStateKind,
    /// When the transition happened (caller-supplied, from the record).
    pub occurred_at: Timestamp,
    /// How long the machine held `to_state`, in integer milliseconds
    /// (until the next transition; `0` for the last span).
    pub duration_ms: u64,
    /// The span's outcome classification.
    pub outcome: HarnessSpanOutcome,
}

impl HarnessTraceSpan {
    /// Validates the span.
    pub fn validate(&self) -> Result<(), HarnessError> {
        if self.seq == 0 {
            return Err(HarnessError::invalid("span sequences are 1-based"));
        }
        if self.event_type != self.to_state.entry_event_type() {
            return Err(HarnessError::invalid(format!(
                "span {} event type {:?} does not match its state",
                self.seq, self.event_type
            )));
        }
        Ok(())
    }
}

/// The bounded trace of one harness run: the transition spans with
/// their durations and outcomes, projected from the machine's own
/// history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessTrace {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The task the run drove (canonical `task_` id).
    pub task_id: String,
    /// The transition spans, in stream order.
    pub spans: Vec<HarnessTraceSpan>,
}

impl HarnessTrace {
    /// Projects a run's history into a bounded trace. The history must
    /// be a legal machine stream (validated by the replay fold — the
    /// same law recovery applies) and belong to the named task.
    ///
    /// # Errors
    ///
    /// Returns [`HarnessError::Invalid`] when the records are not a
    /// legal history, do not belong to `task_id`, or exceed the span
    /// bound.
    pub fn project(task_id: &str, records: &[HarnessEventRecord]) -> Result<Self, HarnessError> {
        let replayed = replay_history(records)?;
        if replayed.task_id != task_id {
            return Err(HarnessError::invalid(format!(
                "the history belongs to task {} but the trace is for task {task_id}",
                replayed.task_id
            )));
        }
        if records.len() > MAX_HARNESS_TRACE_SPANS {
            return Err(HarnessError::invalid(format!(
                "a trace carries at most {MAX_HARNESS_TRACE_SPANS} spans; split the run"
            )));
        }
        let mut spans = Vec::with_capacity(records.len());
        for (index, record) in records.iter().enumerate() {
            let duration_ms = records
                .get(index + 1)
                .map_or(0, |next| span_duration_ms(record, next));
            spans.push(HarnessTraceSpan {
                v: ContractVersion,
                seq: record.seq,
                event_type: record.event_type.clone(),
                to_state: record.to_state,
                occurred_at: record.occurred_at,
                duration_ms,
                outcome: span_outcome(record),
            });
        }
        Ok(Self {
            v: ContractVersion,
            task_id: task_id.to_owned(),
            spans,
        })
    }

    /// The total wall-time of the traced run, in integer milliseconds
    /// (from the first transition to the last).
    #[must_use]
    pub fn total_duration_ms(&self) -> u64 {
        self.spans
            .iter()
            .map(|span| span.duration_ms)
            .fold(0u64, u64::saturating_add)
    }

    /// The observability summary: span count, total duration and the
    /// outcome counts (turns, gaps, recoveries, escalations, endings).
    #[must_use]
    pub fn summary(&self) -> HarnessTraceSummary {
        let mut summary = HarnessTraceSummary {
            spans: self.spans.len() as u64,
            total_duration_ms: self.total_duration_ms(),
            ..HarnessTraceSummary::default()
        };
        for span in &self.spans {
            match span.outcome {
                HarnessSpanOutcome::TurnCompleted => summary.turns_completed += 1,
                HarnessSpanOutcome::TurnCapabilityGap => summary.turns_with_capability_gaps += 1,
                HarnessSpanOutcome::Recovered => summary.recoveries += 1,
                HarnessSpanOutcome::Escalated => summary.escalations += 1,
                HarnessSpanOutcome::Completed => summary.completed_runs += 1,
                HarnessSpanOutcome::Failed => summary.failed_runs += 1,
                HarnessSpanOutcome::Phase => {}
            }
        }
        summary
    }

    /// Validates the trace.
    pub fn validate(&self) -> Result<(), HarnessError> {
        if self.spans.len() > MAX_HARNESS_TRACE_SPANS {
            return Err(HarnessError::invalid(format!(
                "a trace carries at most {MAX_HARNESS_TRACE_SPANS} spans"
            )));
        }
        for span in &self.spans {
            span.validate()?;
        }
        if self.spans.windows(2).any(|pair| pair[0].seq >= pair[1].seq) {
            return Err(HarnessError::invalid(
                "span sequences must be strictly increasing",
            ));
        }
        Ok(())
    }
}

/// The counts a run's trace answers the observability questions with
/// (CONTEXT-HARNESS-ARCHITECTURE §15: recovery success, task
/// completion, verification outcomes, human escalation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessTraceSummary {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// How many spans the trace carries.
    pub spans: u64,
    /// The total wall-time of the run, in integer milliseconds.
    pub total_duration_ms: u64,
    /// How many turns completed.
    pub turns_completed: u64,
    /// How many turns reported a capability gap.
    pub turns_with_capability_gaps: u64,
    /// How many times the run resumed from its own history.
    pub recoveries: u64,
    /// How many times the run escalated to the human.
    pub escalations: u64,
    /// Whether the run completed (at most once: `Done` is terminal).
    pub completed_runs: u64,
    /// Whether the run failed (at most once: `Failed` is terminal).
    pub failed_runs: u64,
}

impl Default for HarnessTraceSummary {
    fn default() -> Self {
        Self::new()
    }
}

impl HarnessTraceSummary {
    /// Builds an empty summary marked with the contract version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            v: ContractVersion,
            spans: 0,
            total_duration_ms: 0,
            turns_completed: 0,
            turns_with_capability_gaps: 0,
            recoveries: 0,
            escalations: 0,
            completed_runs: 0,
            failed_runs: 0,
        }
    }

    /// Validates the summary's structural rules: the version marker is
    /// implicit; a run ends at most once.
    pub fn validate(&self) -> Result<(), HarnessError> {
        if self.completed_runs > 1 || self.failed_runs > 1 {
            return Err(HarnessError::invalid(
                "a run ends at most once (completed and failed are terminal)",
            ));
        }
        if self.completed_runs + self.failed_runs > 1 {
            return Err(HarnessError::invalid(
                "a run cannot be both completed and failed",
            ));
        }
        Ok(())
    }
}

/// The duration between two consecutive records, in integer
/// milliseconds. The replay fold already guarantees non-decreasing
/// timestamps; the subtraction is checked regardless.
fn span_duration_ms(from: &HarnessEventRecord, to: &HarnessEventRecord) -> u64 {
    let delta_seconds = to.occurred_at.to_unix_seconds() - from.occurred_at.to_unix_seconds();
    if delta_seconds <= 0 {
        return 0;
    }
    u64::try_from(delta_seconds.saturating_mul(1000)).unwrap_or(u64::MAX)
}

/// Classifies one record into a span outcome.
fn span_outcome(record: &HarnessEventRecord) -> HarnessSpanOutcome {
    match (&record.detail, record.to_state) {
        (
            HarnessEventDetail::Execute {
                outcome_status: crate::runtime::RuntimeStatus::Completed,
                ..
            },
            HarnessStateKind::Executing,
        ) => HarnessSpanOutcome::TurnCompleted,
        (
            HarnessEventDetail::Execute {
                outcome_status: crate::runtime::RuntimeStatus::CapabilityGap,
                ..
            },
            HarnessStateKind::Executing,
        ) => HarnessSpanOutcome::TurnCapabilityGap,
        (_, HarnessStateKind::Recovering) => HarnessSpanOutcome::Recovered,
        (_, HarnessStateKind::Escalated) => HarnessSpanOutcome::Escalated,
        (_, HarnessStateKind::Done) => HarnessSpanOutcome::Completed,
        (_, HarnessStateKind::Failed) => HarnessSpanOutcome::Failed,
        (_, _) => HarnessSpanOutcome::Phase,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_outcomes_have_distinct_canonical_names() {
        let mut names: Vec<&str> = HarnessSpanOutcome::ALL
            .iter()
            .map(|outcome| outcome.as_str())
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), HarnessSpanOutcome::ALL.len());
        assert_eq!(
            HarnessSpanOutcome::TurnCapabilityGap.as_str(),
            "turn_capability_gap"
        );
    }
}
