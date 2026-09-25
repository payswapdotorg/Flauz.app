//! The world-stream event vocabulary (TAKE-001): the `task.*`
//! takeover/approval/cancellation event types as **data** — canonical
//! payload shapes the CALLER records through the existing world-store
//! seam (the frozen `<entity>.<verb_past>` grammar; addendum §1).
//!
//! This crate records nothing itself: the world store keeps owning
//! the durable entities and their streams. Every payload carries
//! full attribution — the takeover law rides the wire:
//!
//! | event type | recorded when | payload |
//! |---|---|---|
//! | `task.takeover_started` | a human takes over the agent's turn | [`TakeoverStartedPayload`] |
//! | `task.takeover_handback` | the human hands the turn back | [`TakeoverHandbackPayload`] |
//! | `task.approval_requested` | a gate blocks a node with a named need | [`ApprovalRequestedPayload`] |
//! | `task.approval_decided` | the human approves or declines | [`ApprovalDecidedPayload`] |
//! | `task.cancelled` | a node or the run is cancelled | [`CancelledPayload`] |
//! | `task.dependent_cancelled` | the propagation names one dependent | [`DependentCancelledPayload`] |
//!
//! The **same-stream law** (addendum §3): every payload names its
//! task — the human's turn is a task event on the task's own stream,
//! never a second store.

use serde::{Deserialize, Serialize};

use crate::TakeoverError;
use crate::TakeoverVersion;
use crate::approval::{ApprovalDecision, ApprovalGate};
use crate::cancel::{CancellationRecord, DependentResolution};
use crate::refs::TaskRef;
use crate::takeover::{HandbackRecord, TakeoverRecord};

/// The registered event-type vocabulary of this crate (kernel §5).
pub mod event_types {
    /// A human took over the agent's turn.
    pub const TASK_TAKEOVER_STARTED: &str = "task.takeover_started";
    /// The human handed the turn back (explicit, attributed).
    pub const TASK_TAKEOVER_HANDBACK: &str = "task.takeover_handback";
    /// A gate blocked a node with a named need.
    pub const TASK_APPROVAL_REQUESTED: &str = "task.approval_requested";
    /// The human approved or declined a gate.
    pub const TASK_APPROVAL_DECIDED: &str = "task.approval_decided";
    /// A node or the run was cancelled.
    pub const TASK_CANCELLED: &str = "task.cancelled";
    /// The propagation named one dependent's terminal state.
    pub const TASK_DEPENDENT_CANCELLED: &str = "task.dependent_cancelled";
}

/// One event kind of the takeover vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// A human took over the agent's turn.
    TakeoverStarted,
    /// The human handed the turn back.
    TakeoverHandback,
    /// A gate blocked a node with a named need.
    ApprovalRequested,
    /// The human approved or declined a gate.
    ApprovalDecided,
    /// A node or the run was cancelled.
    Cancelled,
    /// The propagation named one dependent's terminal state.
    DependentCancelled,
}

impl EventKind {
    /// Every event kind, in vocabulary order.
    pub const ALL: [Self; 6] = [
        Self::TakeoverStarted,
        Self::TakeoverHandback,
        Self::ApprovalRequested,
        Self::ApprovalDecided,
        Self::Cancelled,
        Self::DependentCancelled,
    ];

    /// The frozen `<entity>.<verb_past>` event-type string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TakeoverStarted => event_types::TASK_TAKEOVER_STARTED,
            Self::TakeoverHandback => event_types::TASK_TAKEOVER_HANDBACK,
            Self::ApprovalRequested => event_types::TASK_APPROVAL_REQUESTED,
            Self::ApprovalDecided => event_types::TASK_APPROVAL_DECIDED,
            Self::Cancelled => event_types::TASK_CANCELLED,
            Self::DependentCancelled => event_types::TASK_DEPENDENT_CANCELLED,
        }
    }
}

/// The payload of a `task.takeover_started` event: the takeover
/// record, in full — the human's turn begins on the task's stream,
/// attributed to the human.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TakeoverStartedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The takeover record (who took over, from what, when, why, the
    /// preserved work).
    pub takeover: TakeoverRecord,
}

impl TakeoverStartedPayload {
    /// Builds the payload from a takeover record.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the record fails validation.
    pub fn new(takeover: TakeoverRecord) -> Result<Self, TakeoverError> {
        takeover.validate()?;
        Ok(Self {
            v: TakeoverVersion,
            takeover,
        })
    }
}

/// The payload of a `task.takeover_handback` event: the handback
/// record, in full — the explicit, attributed return.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TakeoverHandbackPayload {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The handback record (the takeover embedded, what the human
    /// did, whether the agent resumes or the run completes).
    pub handback: HandbackRecord,
}

impl TakeoverHandbackPayload {
    /// Builds the payload from a handback record.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the record fails validation.
    pub fn new(handback: HandbackRecord) -> Result<Self, TakeoverError> {
        handback.validate()?;
        Ok(Self {
            v: TakeoverVersion,
            handback,
        })
    }
}

/// The payload of a `task.approval_requested` event: the gate, in
/// full — the named need with the consequence of each side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalRequestedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The approval gate (the named need that blocks the node).
    pub gate: ApprovalGate,
}

impl ApprovalRequestedPayload {
    /// Builds the payload from a gate.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the gate fails validation.
    pub fn new(gate: ApprovalGate) -> Result<Self, TakeoverError> {
        gate.validate()?;
        Ok(Self {
            v: TakeoverVersion,
            gate,
        })
    }
}

/// The payload of a `task.approval_decided` event: the decision, in
/// full — approve or deny, the deciding human, the attributed effect
/// (a denial's effect is the honest failure with the named
/// consequence).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalDecidedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The approval decision.
    pub decision: ApprovalDecision,
}

impl ApprovalDecidedPayload {
    /// Builds the payload from a decision.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the decision fails validation.
    pub fn new(decision: ApprovalDecision) -> Result<Self, TakeoverError> {
        decision.validate()?;
        Ok(Self {
            v: TakeoverVersion,
            decision,
        })
    }
}

/// The payload of a `task.cancelled` event: the cancellation record,
/// in full — what was cancelled (node or run), the reason, the actor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancelledPayload {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The cancellation record.
    pub cancellation: CancellationRecord,
}

impl CancelledPayload {
    /// Builds the payload from a cancellation record.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the record fails validation.
    pub fn new(cancellation: CancellationRecord) -> Result<Self, TakeoverError> {
        cancellation.validate()?;
        Ok(Self {
            v: TakeoverVersion,
            cancellation,
        })
    }
}

/// The payload of a `task.dependent_cancelled` event: one dependent's
/// named terminal state from the propagation — the node-level truth
/// the projection outputs (addendum §4: the node-level propagation is
/// the projection's output, recorded as new world event types).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependentCancelledPayload {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The task whose stream the event lands on.
    pub task: TaskRef,
    /// The dependent's named resolution.
    pub dependent: DependentResolution,
}

impl DependentCancelledPayload {
    /// Builds the payload from a task and one dependent resolution.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the task reference or the
    /// resolution's references fail validation.
    pub fn new(task: TaskRef, dependent: DependentResolution) -> Result<Self, TakeoverError> {
        task.validate()?;
        dependent.node.validate()?;
        if let Some(waited_on) = dependent.waited_on.as_ref() {
            waited_on.validate()?;
        }
        Ok(Self {
            v: TakeoverVersion,
            task,
            dependent,
        })
    }
}

#[cfg(test)]
mod tests {
use super::*;
use crate::cancel::DependentTerminal;
use crate::fakes::{
    ana_takes_over_research, cancel_mid_run_node, env_switch_gate, fake_task, now, research_graph,
};
use crate::propagate_cancellation;
use crate::refs::ActorRef;
    use crate::refs::ActorRef;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn the_vocabulary_is_the_frozen_dotted_grammar() {
        for (kind, text) in [
            (EventKind::TakeoverStarted, "task.takeover_started"),
            (EventKind::TakeoverHandback, "task.takeover_handback"),
            (EventKind::ApprovalRequested, "task.approval_requested"),
            (EventKind::ApprovalDecided, "task.approval_decided"),
            (EventKind::Cancelled, "task.cancelled"),
            (EventKind::DependentCancelled, "task.dependent_cancelled"),
        ] {
            assert_eq!(kind.as_str(), text);
        }
        assert_eq!(EventKind::ALL.len(), 6);
        // Every event type parses under the frozen world grammar
        // (`<entity>.<verb_past>`, both segments `[a-z][a-z0-9_]*`)
        // — pinned locally so a rename cannot drift out of the
        // world-store seam's acceptance.
        for kind in EventKind::ALL {
            let text = kind.as_str();
            let (entity, verb) = ok(text.split_once('.').ok_or("missing the dot"));
            assert!(!entity.is_empty());
            assert!(!verb.contains('.'));
            assert!(entity.chars().next().is_some_and(|c| c.is_ascii_lowercase()));
            assert!(verb.chars().next().is_some_and(|c| c.is_ascii_lowercase()));
            assert!(entity
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
            assert!(verb
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        }
    }

    #[test]
    fn payloads_round_trip_canonically_and_carry_attribution() {
        // Takeover started.
        let takeover = ana_takes_over_research();
        let started = ok(TakeoverStartedPayload::new(takeover.clone()));
        let serialized = ok(serde_json::to_string(&started));
        let reloaded: TakeoverStartedPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, started);
        assert!(serialized.contains("\"human\":{"));
        assert!(serialized.contains("\"preserved\":["));

        // Handback.
        let handback = ok(crate::takeover::HandbackRecord::for_takeover(
            takeover,
            ok(crate::Timestamp::parse("2026-09-23T15:26:00Z")),
            "reviewed the sandbox credentials and approved the switch",
            crate::takeover::HandbackOutcome::AgentResumes,
        ));
        let payload = ok(TakeoverHandbackPayload::new(handback));
        let serialized = ok(serde_json::to_string(&payload));
        let reloaded: TakeoverHandbackPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, payload);
        assert!(serialized.contains("\"outcome\":\"agent_resumes\""));

        // Approval requested.
        let gate = ok(env_switch_gate());
        let requested = ok(ApprovalRequestedPayload::new(gate.clone()));
        let serialized = ok(serde_json::to_string(&requested));
        let reloaded: ApprovalRequestedPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, requested);
        assert!(serialized.contains("\"approve_consequence\":"));
        assert!(serialized.contains("\"deny_consequence\":"));

        // Approval decided (both sides).
        let approved = ok(ApprovalDecision::approve(
            gate.clone(),
            ok(ActorRef::user("user_ana")),
            now(),
        ));
        let payload = ok(ApprovalDecidedPayload::new(approved));
        let serialized = ok(serde_json::to_string(&payload));
        assert!(serialized.contains("\"kind\":\"approved\""));
        assert!(serde_json::from_str::<ApprovalDecidedPayload>(&serialized).is_ok());
        let denied = ok(ApprovalDecision::deny(
            gate,
            ok(ActorRef::user("user_ana")),
            now(),
        ));
        let payload = ok(ApprovalDecidedPayload::new(denied));
        let serialized = ok(serde_json::to_string(&payload));
        assert!(serialized.contains("\"kind\":\"denied\""));
        assert!(serialized.contains("\"kind\":\"node_failed\""));

        // Cancelled + dependent cancelled.
        let cancellation = ok(cancel_mid_run_node());
        let payload = ok(CancelledPayload::new(cancellation.clone()));
        let serialized = ok(serde_json::to_string(&payload));
        let reloaded: CancelledPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, payload);
        let propagation = ok(propagate_cancellation(
            &cancellation,
            &ok(research_graph()),
        ));
        let dependent = propagation.dependents[0].clone();
        let payload = ok(DependentCancelledPayload::new(fake_task(), dependent));
        let serialized = ok(serde_json::to_string(&payload));
        let reloaded: DependentCancelledPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, payload);
        assert!(serialized.contains("\"kind\":\"cancelled\""));

        // Unknown fields are rejected on every payload family.
        assert!(
            serde_json::from_str::<TakeoverStartedPayload>(
                &ok(serde_json::to_string(&ok(TakeoverStartedPayload::new(
                    ana_takes_over_research()
                ))))
                .replace("\"takeover\":", "\"grab\":")
            )
            .is_err()
        );
    }

    #[test]
    fn the_same_stream_law_rides_every_payload() {
        // Every payload family names its task — the human's turn is a
        // task event, never a second store.
        let takeover = ana_takes_over_research();
        assert_eq!(
            ok(TakeoverStartedPayload::new(takeover.clone()))
                .takeover
                .stream_task(),
            &takeover.task
        );
        let handback = ok(crate::takeover::HandbackRecord::for_takeover(
            takeover.clone(),
            ok(crate::Timestamp::parse("2026-09-23T15:26:00Z")),
            "reviewed the sandbox credentials and approved the switch",
            crate::takeover::HandbackOutcome::AgentResumes,
        ));
        assert_eq!(
            ok(TakeoverHandbackPayload::new(handback))
                .handback
                .stream_task(),
            &takeover.task
        );
        let gate = ok(env_switch_gate());
        assert_eq!(
            ok(ApprovalRequestedPayload::new(gate.clone()))
                .gate
                .stream_task(),
            &gate.task
        );
        let cancellation = ok(cancel_mid_run_node());
        assert_eq!(
            ok(CancelledPayload::new(cancellation.clone()))
                .cancellation
                .stream_task(),
            &cancellation.task
        );
        let dependent = crate::cancel::DependentResolution {
            v: crate::TakeoverVersion,
            node: ok(crate::refs::NodeName::parse("analysis")),
            waited_on: Some(ok(crate::refs::NodeName::parse("research"))),
            terminal: DependentTerminal::Cancelled {
                reason: cancellation.reason.clone(),
            },
            kept: Vec::new(),
        };
        let payload = ok(DependentCancelledPayload::new(cancellation.task, dependent));
        let serialized = ok(serde_json::to_string(&payload));
        assert!(serialized.contains("\"task\":\"task_"));
    }
}
