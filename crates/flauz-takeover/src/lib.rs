//! The human in the loop: takeover/approval/handoff records with
//! attribution, approval gates with named needs and consequences, and
//! cancellation/dependency propagation with named terminal states
//! (F2 Wave 5, work order **TAKE-001**).
//!
//! The harness can ESCALATE (the frozen `flauz-exec` state machine's
//! `Escalated` state — read-only reference), but escalation alone
//! models nothing that happens NEXT. This crate is the fabric that
//! does: the human's work as a first-class attributed turn
//! ([`takeover`]), the explicit handback, the approval gates that
//! block with a named need ([`approval`]), and the honest
//! cancellation of a node or the run with every dependent told the
//! truth ([`cancel`]).
//!
//! # The takeover law (J-17, addendum §3)
//!
//! A takeover is an EXPLICIT, ATTRIBUTED handoff: who took over (the
//! human actor), from what (the agent/node/run), when, and why. The
//! law is STRUCTURAL — a record without full attribution cannot be
//! constructed:
//!
//! - a [`takeover::TakeoverRecord`] carries the human actor (kind
//!   `user`, validated at construction), the agent whose turn it was
//!   ([`refs::AgentRef`], the canonical `agent_<ULID>` grammar), the
//!   node/run scope, the moment and the reason;
//! - the **same-stream law**: the human's turn is a TASK event — every
//!   record carries its [`refs::TaskRef`] and lands on that task's
//!   event stream through the caller's world-store seam (the
//!   [`events`] vocabulary); there is no second store, and this crate
//!   records nothing itself;
//! - the **projection law**: the agent's prior work is preserved
//!   verbatim — the takeover record carries the preserved artifacts,
//!   and the [`takeover::HandbackRecord`] can only be constructed
//!   against its own takeover, handing back the same preserved work;
//! - the **handback** is explicit and attributed: what the human did,
//!   and whether the agent resumes or the run completes
//!   ([`takeover::HandbackOutcome`]).
//!
//! An approval gate BLOCKS with a named need ([`approval::ApprovalGate`]):
//! what must be approved, the requesting node, and the consequence of
//! EACH side. Approval and denial are attributed decisions
//! ([`approval::ApprovalDecision`]); denial carries the named
//! consequence — the node fails honestly with the denial as its
//! reason, never a silent proceed. An approval gate is DATA the
//! evaluator/caller consults, not a thread block.
//!
//! # Cancellation honesty (addendum §4 — the honest-failure family)
//!
//! Cancellation is distinguished from failure everywhere. A
//! [`cancel::CancellationRecord`] names what was cancelled (node or
//! run), the reason and the actor. The propagation projection
//! ([`cancel::propagate_cancellation`]) takes a graph as data plus the
//! cancelled node and names EVERY dependent's terminal state:
//! `cancelled`, or `blocked-resolved` with the reason — never a
//! silently-stuck `Blocked`. Already-attributed work is kept. The
//! run-level record stays at most one `RunCanceled` per sequence (the
//! frozen `flauz-orch` law); the node-level propagation is the
//! projection's output, recorded as new world event types.
//!
//! # Inputs only — the frozen formats cross the seam (the flauz-cap law)
//!
//! This crate does NOT import `flauz-world`, `flauz-exec`,
//! `flauz-orch` or any other contract crate: foreign entities cross
//! the seam exclusively as frozen format strings — [`refs::TaskRef`]
//! re-validates the `task_<ULID>` grammar, [`refs::AgentRef`] the
//! `agent_<ULID>` grammar, [`refs::ArtifactRef`] the `art_<ULID>`
//! grammar, [`refs::NodeName`] the bounded node-name grammar, and
//! [`refs::ActorRef`] the frozen actor-reference shape that
//! `flauz-world` owns — never as re-defined entity types. The graph
//! the propagation projects over is [`cancel::GraphShape`] — plain
//! call-side data, not the frozen `flauz-orch` graph.
//!
//! # Determinism (kernel §7, addendum §7)
//!
//! Contract code in this crate never reads wall-clock time or
//! randomness and generates no identifiers: every timestamp is
//! caller-supplied, and every record is a pure function of its
//! constructor arguments. Identical inputs always produce
//! byte-identical records and projections (proven by the conformance
//! tests). The in-memory [`fakes`] are fully deterministic.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields
//! (`deny_unknown_fields`), contains no floats, and tags enums
//! internally with `"kind"` (fieldless enums serialize as their
//! lowercase names). Timestamps are RFC 3339 UTC seconds precision
//! (`YYYY-MM-DDTHH:MM:SSZ`); durations are integer milliseconds.
//!
//! # The world-stream event vocabulary
//!
//! The crate defines the `task.*` takeover vocabulary as DATA
//! ([`events`]): `task.takeover_started` / `task.takeover_handback` /
//! `task.approval_requested` / `task.approval_decided` /
//! `task.cancelled` / `task.dependent_cancelled` — canonical payload
//! shapes the CALLER records through the existing world-store seam
//! (the frozen `<entity>.<verb_past>` grammar). This crate records
//! nothing itself; the world store keeps owning the streams.

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod approval;
pub mod cancel;
pub mod events;
pub mod fakes;
pub mod refs;
pub mod takeover;
pub mod time;

pub use crate::approval::{
    ApprovalDecision, ApprovalGate, DecisionEffect, DENIAL_FAILURE_PREFIX,
};
pub use crate::cancel::{
    CancelledWhat, CancellationRecord, DependentResolution, DependentTerminal, GraphNode,
    GraphShape, Propagation, propagate_cancellation,
};
pub use crate::events::{
    ApprovalDecidedPayload, ApprovalRequestedPayload, CancelledPayload, DependentCancelledPayload,
    EventKind, TakeoverHandbackPayload, TakeoverStartedPayload,
};
pub use crate::refs::{ActorKind, ActorRef, AgentRef, ArtifactRef, NodeName, TaskRef};
pub use crate::takeover::{
    HandbackOutcome, HandbackRecord, PreservedArtifact, TakeoverRecord, TakeoverScope,
};
pub use crate::time::Timestamp;

/// Maximum length of an actor id (matches the flauz-world bound).
pub const MAX_ACTOR_ID_BYTES: usize = 256;

/// Maximum length of a node name (matches the flauz-orch bound).
pub const MAX_NODE_NAME_BYTES: usize = 256;

/// Maximum length of honest reasons, needs and consequences (matches
/// the flauz-orch explanation bound).
pub const MAX_EXPLANATION_BYTES: usize = 512;

/// Maximum number of nodes in one propagation graph (matches the
/// flauz-orch bound — bounded inputs).
pub const MAX_GRAPH_NODES: usize = 64;

/// Maximum number of dependencies one node may declare (matches the
/// flauz-orch bound).
pub const MAX_DEPENDENCIES: usize = 64;

/// Maximum number of preserved artifacts one takeover may carry
/// (bounded inputs — the projection law's list is bounded).
pub const MAX_PRESERVED_ARTIFACTS: usize = 64;

/// Contract-level validation error for values that fail canonical rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TakeoverError {
    /// A value violates a canonical rule (bounds, ordering, grammar,
    /// attribution, record consistency).
    Invalid(String),
}

impl TakeoverError {
    /// Builds a [`TakeoverError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl fmt::Display for TakeoverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid takeover state: {reason}"),
        }
    }
}

impl Error for TakeoverError {}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TakeoverVersion;

impl serde::Serialize for TakeoverVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for TakeoverVersion {
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

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), TakeoverError> {
    if value.is_empty() {
        Err(TakeoverError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), TakeoverError> {
    if value.len() > max_bytes {
        Err(TakeoverError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_explanation(field: &'static str, value: &str) -> Result<(), TakeoverError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_EXPLANATION_BYTES)
}

pub(crate) fn ensure_name(field: &'static str, value: &str) -> Result<(), TakeoverError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_NODE_NAME_BYTES)
}

pub(crate) fn ensure_list_bound(
    field: &'static str,
    list_len: usize,
    max_entries: usize,
) -> Result<(), TakeoverError> {
    if list_len > max_entries {
        Err(TakeoverError::invalid(format!(
            "{field} exceeds {max_entries} entries"
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
        let serialized = ok(serde_json::to_string(&TakeoverVersion));
        assert_eq!(serialized, "1");
        assert!(serde_json::from_str::<TakeoverVersion>("1").is_ok());
        assert!(serde_json::from_str::<TakeoverVersion>("2").is_err());
        assert!(serde_json::from_str::<TakeoverVersion>("0").is_err());
    }
}
