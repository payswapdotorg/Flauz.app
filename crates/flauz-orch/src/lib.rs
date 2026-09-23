//! The execution graph (F2 Wave 3, work order **ORCH-004**): attributed
//! multi-agent orchestration — a graph of agents, never a chat relay.
//!
//! One task may hold several agents cooperating with roles, dependencies
//! and blocked states: Agent A (research, browser) + Agent B (analysis,
//! terminal) + Agent C (independent review) running in parallel where B
//! waits on A's artifact and C verifies both. This crate owns the pure
//! data + evaluator of that graph:
//!
//! - [`AgentAssignment`] — who (an actor reference), role label,
//!   dependencies (artifact-wait / resource-wait edges), state
//!   (ready / running / blocked / done / failed / verified) and — for
//!   verifier nodes — the independence scope;
//! - [`ExecutionGraph`] — the attributed graph for one task (validated:
//!   acyclic, known nodes, unique node ids, waits resolvable);
//! - the evaluator ([`ExecutionGraph::advance`]) — unblock nodes when
//!   their waits clear, attribute every produced artifact/evidence to
//!   its node, derive the independently-verified overlay, fire the merge
//!   point (all-done → the task-level verification event) and propagate
//!   cancellation ([`ExecutionGraph::cancel`]);
//! - [`GraphEvent`] — the run's happenings as plain data: the graph takes
//!   events as inputs and never calls a runtime, a harness or a model.
//!
//! # Orchestration is a graph of attributed agents, not a chat relay
//! (Wave-3 addendum §4)
//!
//! Agent-to-agent coordination happens through **shared task state +
//! artifacts / evidence**: a node waits for an ARTIFACT or a RESOURCE —
//! never for "the transcript of node X". There is no chat-relay
//! abstraction anywhere in this crate: the dependency grammar
//! ([`WaitOn`]) has exactly two kinds, `artifact_ready` and
//! `resource_ready`, and the serialized grammar rejects every other kind
//! on read (the conformance test pins a `context_snapshot` wait being
//! rejected — structurally impossible to express, not merely
//! discouraged).
//!
//! # The independence rule (addendum §4)
//!
//! A verifier node's inputs are the **artifacts it verifies** —
//! structurally never a context snapshot of the verified node. A node
//! declares its verifier scope ([`VerifierScope`]); validation then
//! REQUIRES the verifier to wait on at least one artifact from every
//! node it verifies, and forbids self-verification. Independence is
//! structural, not behavioral: the verifier's only route to the
//! verified work is the artifact.
//!
//! # Inputs only — the frozen formats cross the seam (the flauz-cap
//! pattern)
//!
//! The crate does NOT import `flauz-world`, `flauz-exec`,
//! `flauz-context` or `flauz-cap`: foreign entities cross the seam
//! exclusively as frozen format strings. [`refs`] re-validates the
//! frozen canonical-ID grammar (`<kind>_<ULID>`, Crockford Base32) for
//! the kinds the graph references (`task`, `art`, `evd`, `res`), plus
//! the frozen actor-reference grammar
//! `{"kind": "user|agent|system|provider", "id": "…"}` — byte-identical
//! in meaning to `flauz-world`'s types on the wire. Harness states (the
//! ORCH-003 seam) are referenced as plain data: a node may carry a
//! bounded `harness_state` string, opaque to this crate.
//!
//! # Registered event types (kernel §5)
//!
//! This crate registers the `event_type` vocabulary it produces:
//!
//! | Constant | `event_type` |
//! |---|---|
//! | [`event_types::TASK_VERIFICATION_MERGED`] | `task.verification_merged` |
//!
//! The merge point fires when every node has reached a terminal-success
//! state: the descriptor (in [`MergePoint`]) is the task-level
//! verification event a later slice appends to the task's stream
//! through the world store.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields
//! (`deny_unknown_fields`), contains no floats, and tags enums
//! internally with `"kind"` (fieldless enums serialize as their
//! lowercase snake_case names). Timestamps do not occur in the graph's
//! v1 records: node states are ordered by the caller's event sequence,
//! not by wall-clock time.
//!
//! # Determinism (kernel §7)
//!
//! Contract code in this crate never reads wall-clock time or
//! randomness and generates no identifiers: the evaluator is a pure
//! function of (graph, events), so identical inputs produce identical
//! evaluations — byte-identical serialized records. The in-memory
//! [`fakes`] are fully deterministic.

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod evaluator;
pub mod fakes;
pub mod graph;
pub mod node;
pub mod refs;

pub use crate::evaluator::{
    Attribution, CanceledNode, GraphCancellation, GraphEvaluation, GraphEvent, GraphEventSequence,
    MergePoint, NodeEvaluation, Product, event_types,
};
pub use crate::graph::ExecutionGraph;
pub use crate::node::{AgentAssignment, NodeId, NodeInputs, NodeState, RoleLabel, VerifierScope};
pub use crate::refs::{
    ActorKind, ActorRef, ArtifactRef, EvidenceRef, ResourceRef, TaskRef, WaitOn,
};

/// Maximum length of short human-readable names (node ids, role labels,
/// artifact/evidence labels, wait labels — matches the flauz-world bound).
pub const MAX_NAME_BYTES: usize = 256;

/// Maximum length of honest failure/cancellation reasons and other
/// bounded prose fields.
pub const MAX_EXPLANATION_BYTES: usize = 512;

/// Maximum number of nodes in one graph (matches the flauz-world
/// related-refs bound).
pub const MAX_NODES: usize = 64;

/// Maximum number of dependencies one node may declare.
pub const MAX_DEPENDENCIES: usize = 64;

/// Maximum number of nodes one verifier may independently verify.
pub const MAX_VERIFIES: usize = 64;

/// Maximum number of graph events evaluated in one `advance` batch.
pub const MAX_EVENTS: usize = 512;

/// Contract-level validation error for values that fail canonical rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchError {
    /// A value violates a canonical rule (bounds, ordering, grammar,
    /// graph consistency, illegal transition).
    Invalid(String),
}

impl OrchError {
    /// Builds an [`OrchError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }

    /// Returns the error's reason text.
    #[must_use]
    pub fn reason(&self) -> &str {
        match self {
            Self::Invalid(reason) => reason,
        }
    }
}

impl fmt::Display for OrchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
        }
    }
}

impl Error for OrchError {}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrchVersion;

impl serde::Serialize for OrchVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for OrchVersion {
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

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), OrchError> {
    if value.is_empty() {
        Err(OrchError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), OrchError> {
    if value.len() > max_bytes {
        Err(OrchError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_name(field: &'static str, value: &str) -> Result<(), OrchError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_NAME_BYTES)
}

pub(crate) fn ensure_explanation(field: &'static str, value: &str) -> Result<(), OrchError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_EXPLANATION_BYTES)
}

pub(crate) fn ensure_list_bound(
    field: &'static str,
    list_len: usize,
    max_entries: usize,
) -> Result<(), OrchError> {
    if list_len > max_entries {
        Err(OrchError::invalid(format!(
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
    fn the_schema_version_marker_round_trips_and_rejects_other_versions() {
        assert_eq!(ok(serde_json::to_string(&OrchVersion)), "1");
        let reloaded: OrchVersion = ok(serde_json::from_str("1"));
        assert_eq!(reloaded, OrchVersion);
        assert!(
            serde_json::from_str::<OrchVersion>("2").is_err(),
            "a future schema version must fail the canonical read, never silently misread"
        );
    }

    #[test]
    fn reasons_are_displayable_for_every_error_shape() {
        let error = OrchError::invalid("node ids must be unique");
        assert_eq!(error.to_string(), "node ids must be unique");
        assert_eq!(error.reason(), "node ids must be unique");
    }
}
