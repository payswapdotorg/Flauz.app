//! The graph evaluator: `advance` — unblock when waits clear, attribute
//! every produced artifact/evidence to its node, derive the independent
//! verification overlay, fire the merge point, propagate cancellation
//! (addendum §4).
//!
//! [`ExecutionGraph::advance`] is a pure function of `(graph, events)`:
//! it takes the run's happenings as plain data ([`GraphEventSequence`])
//! and produces a [`GraphEvaluation`] — the resulting node states (with
//! what every blocked node still waits for), the attribution ledger
//! (every artifact/evidence attributed to exactly one node), an
//! optional cancellation record, and the merge point when every node
//! has reached a terminal-success state.
//!
//! # Determinism (kernel §7)
//!
//! No wall-clock reads, no randomness, no generated identifiers:
//! identical `(graph, events)` inputs produce byte-identical
//! evaluations. Illegal transitions and dishonest events (producing
//! while blocked, producing the same artifact twice, starting a
//! non-ready node) are ERRORS, never silently accepted.
//!
//! # Registered event types (kernel §5)
//!
//! The merge point produces the task-level verification event
//! [`event_types::TASK_VERIFICATION_MERGED`] (`task.verification_merged`)
//! — the descriptor a later slice appends to the task's stream through
//! the world store. The per-event vocabulary of the evaluator itself
//! (`node.started`, …) is internal to this crate's input grammar; only
//! the merge event crosses into the world's event vocabulary.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::graph::ExecutionGraph;
use crate::node::{NodeId, NodeState};
use crate::refs::{ArtifactRef, EvidenceRef, ResourceRef, TaskRef, WaitOn};
use crate::{
    MAX_EVENTS, OrchError, OrchVersion, ensure_explanation, ensure_list_bound, ensure_name,
};

/// The `event_type` vocabulary this crate registers (kernel §5).
pub mod event_types {
    /// The merge point: every node reached a terminal-success state —
    /// the task-level verification event. The descriptor is
    /// [`crate::MergePoint`]; a later slice appends it to the task's
    /// stream through the world store.
    pub const TASK_VERIFICATION_MERGED: &str = "task.verification_merged";
}

/// One happening in the run, as plain data (inputs as data — the
/// evaluator never calls a runtime, harness or model; a node's
/// execution is whatever drives it, and only its products and state
/// changes cross into the graph).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum GraphEvent {
    /// A node began running (Ready → Running).
    NodeStarted {
        /// The node that started.
        node: String,
    },
    /// A node produced an artifact: attributed to the node, and clears
    /// every artifact wait on `(from_node, label)` — shared task state,
    /// never a transcript.
    ArtifactProduced {
        /// The producing node.
        by_node: String,
        /// The artifact.
        artifact: ArtifactRef,
        /// The artifact's label (the same label the waits carry).
        label: String,
    },
    /// A node recorded verification evidence: attributed to the node.
    EvidenceRecorded {
        /// The recording node.
        by_node: String,
        /// The evidence record.
        evidence: EvidenceRef,
        /// The evidence's label (for example "the review verdict").
        label: String,
    },
    /// A node finished its work (Running → Done). When the node is a
    /// verifier, the nodes in its scope transition to the derived
    /// `Verified` overlay.
    NodeFinished {
        /// The node that finished.
        node: String,
    },
    /// A node failed with an honest reason (Ready/Running/Blocked →
    /// Failed, terminal). Failures are local: they do not cascade.
    NodeFailed {
        /// The node that failed.
        node: String,
        /// The honest failure reason.
        reason: String,
    },
    /// A resource became available: clears every resource wait on it.
    ResourceAvailable {
        /// The resource.
        resource: ResourceRef,
    },
    /// The run was canceled: every non-terminal node fails with the
    /// cancellation recorded (the six-state vocabulary gains no
    /// `canceled` variant — the cancellation record distinguishes
    /// cancellation from failure, and work already attributed is kept).
    /// At most one per sequence.
    RunCanceled {
        /// The honest cancellation reason.
        reason: String,
    },
}

impl GraphEvent {
    /// Validates the event's shape and bounds (the graph-level legality
    /// — known nodes, legal transitions — is checked by `advance`).
    pub fn validate(&self) -> Result<(), OrchError> {
        match self {
            Self::NodeStarted { node } | Self::NodeFinished { node } => {
                ensure_name("event node", node)?;
            }
            Self::ArtifactProduced {
                by_node,
                artifact,
                label,
            } => {
                ensure_name("event node", by_node)?;
                artifact.validate()?;
                ensure_name("artifact label", label)?;
            }
            Self::EvidenceRecorded {
                by_node,
                evidence,
                label,
            } => {
                ensure_name("event node", by_node)?;
                evidence.validate()?;
                ensure_name("evidence label", label)?;
            }
            Self::NodeFailed { node, reason } => {
                ensure_name("event node", node)?;
                ensure_explanation("failure reason", reason)?;
            }
            Self::ResourceAvailable { resource } => {
                resource.validate()?;
            }
            Self::RunCanceled { reason } => {
                ensure_explanation("cancellation reason", reason)?;
            }
        }
        Ok(())
    }

    /// The node the event is about, for attribution-bearing events;
    /// `None` for run-level events.
    #[must_use]
    pub fn node(&self) -> Option<&str> {
        match self {
            Self::NodeStarted { node }
            | Self::NodeFinished { node }
            | Self::NodeFailed { node, .. } => Some(node),
            Self::ArtifactProduced { by_node, .. } | Self::EvidenceRecorded { by_node, .. } => {
                Some(by_node)
            }
            Self::ResourceAvailable { .. } | Self::RunCanceled { .. } => None,
        }
    }
}

/// A canonically versioned sequence of graph events — the evaluator's
/// input document (the run's happenings as plain data).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEventSequence {
    /// Contract schema version (`"v": 1`).
    pub v: OrchVersion,
    /// The events, in happening order.
    pub events: Vec<GraphEvent>,
}

impl GraphEventSequence {
    /// Builds an event sequence, validating every event's shape and the
    /// sequence bound.
    pub fn new(events: Vec<GraphEvent>) -> Result<Self, OrchError> {
        ensure_list_bound("event sequence", events.len(), MAX_EVENTS)?;
        for event in &events {
            event.validate()?;
        }
        Ok(Self {
            v: OrchVersion,
            events,
        })
    }

    /// Validates the sequence's shape and bounds.
    pub fn validate(&self) -> Result<(), OrchError> {
        Self::new(self.events.clone())?;
        Ok(())
    }
}

/// A product attributed to its producing node: an artifact or an
/// evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum Product {
    /// An artifact produced by the node.
    Artifact {
        /// The artifact.
        artifact: ArtifactRef,
        /// The artifact's label.
        label: String,
    },
    /// Verification evidence recorded by the node.
    Evidence {
        /// The evidence record.
        evidence: EvidenceRef,
        /// The evidence's label.
        label: String,
    },
}

/// One attribution ledger entry: the node that produced the product.
/// Every produced artifact/evidence is attributed to exactly one node
/// (addendum §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attribution {
    /// The producing node.
    pub node_id: NodeId,
    /// What it produced.
    pub product: Product,
}

/// One node's outcome in an evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeEvaluation {
    /// The node's id.
    pub node_id: NodeId,
    /// The resulting state (the derived `Verified` overlay included).
    pub state: NodeState,
    /// Whether this evaluation changed the node's state from the
    /// declared graph state.
    pub changed: bool,
    /// The node's UNCLEARED waits, in declaration order — what a
    /// blocked node names ("Waiting for the research notes").
    pub waiting_on: Vec<WaitOn>,
}

/// A canceled node: what it was doing when the run was canceled, and
/// the work it had already produced (kept — honest: attributed work is
/// never erased).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanceledNode {
    /// The canceled node.
    pub node_id: NodeId,
    /// The state it was canceled from.
    pub from_state: NodeState,
    /// The products it had already produced (kept in the attribution
    /// ledger).
    pub products_kept: Vec<Product>,
}

/// The cancellation record: why the run was canceled, which nodes were
/// canceled (from which states), and which nodes had already settled
/// (their terminal states stand). The resulting state of every canceled
/// node is `Failed` — the six-state vocabulary gains no `canceled`
/// variant; THIS record distinguishes cancellation from failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphCancellation {
    /// Contract schema version (`"v": 1`).
    pub v: OrchVersion,
    /// The canceled run's task.
    pub task: TaskRef,
    /// The honest cancellation reason.
    pub reason: String,
    /// The canceled nodes, sorted by node id.
    pub canceled: Vec<CanceledNode>,
    /// The nodes that had already settled (terminal states stand; their
    /// attributed work is kept), sorted by node id.
    pub settled: Vec<String>,
}

/// The merge point (addendum §4): every node reached a terminal-success
/// state — the task-level verification event. `verified` names the
/// independently verified nodes (the derived overlay) and `done` the
/// merely finished ones (claimed, not independently verified — J-09's
/// law holds at the task level too).
///
/// This is a DESCRIPTOR: a later slice appends it to the task's event
/// stream through the world store (sequence numbers belong to the
/// store, kernel §3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergePoint {
    /// Contract schema version (`"v": 1`).
    pub v: OrchVersion,
    /// The registered event type
    /// ([`event_types::TASK_VERIFICATION_MERGED`]).
    pub event_type: String,
    /// The task whose work merged.
    pub task: TaskRef,
    /// The independently verified node ids, sorted.
    pub verified: Vec<String>,
    /// The merely done node ids (no independent verification), sorted.
    pub done: Vec<String>,
    /// Every attributed artifact, in attribution order.
    pub artifacts: Vec<ArtifactRef>,
    /// Every attributed evidence record, in attribution order.
    pub evidence: Vec<EvidenceRef>,
}

impl MergePoint {
    /// Validates the merge descriptor (the registered event type, the
    /// verified/done split).
    pub fn validate(&self) -> Result<(), OrchError> {
        if self.event_type != event_types::TASK_VERIFICATION_MERGED {
            return Err(OrchError::invalid(format!(
                "merge point event type must be {:?}, found {:?}",
                event_types::TASK_VERIFICATION_MERGED,
                self.event_type
            )));
        }
        self.task.validate()?;
        for list in [&self.verified, &self.done] {
            ensure_list_bound("merge node list", list.len(), crate::MAX_NODES)?;
            for node in list {
                ensure_name("merge node id", node)?;
            }
        }
        Ok(())
    }
}

/// The evaluator's output: the resulting node states, the attribution
/// ledger, an optional cancellation record, and the merge point when
/// every node reached a terminal-success state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEvaluation {
    /// Contract schema version (`"v": 1`).
    pub v: OrchVersion,
    /// The evaluated task.
    pub task: TaskRef,
    /// Every node's outcome, sorted by node id (canonical order).
    pub nodes: Vec<NodeEvaluation>,
    /// The attribution ledger: every produced artifact/evidence
    /// attributed to its node, in production order.
    pub attribution: Vec<Attribution>,
    /// The cancellation record, when the run was canceled.
    pub cancellation: Option<GraphCancellation>,
    /// The merge point, when every node reached a terminal-success
    /// state.
    pub merge: Option<MergePoint>,
}

impl ExecutionGraph {
    /// Evaluates the graph over an event sequence: unblock nodes when
    /// their waits clear, attribute every produced artifact/evidence to
    /// its node, derive the independent verification overlay, record a
    /// cancellation, and fire the merge point when every node has
    /// reached a terminal-success state.
    ///
    /// Pure and deterministic — identical inputs produce identical
    /// evaluations. The graph is never mutated.
    ///
    /// # Errors
    ///
    /// Returns [`OrchError`] when the sequence is invalid (shape,
    /// bound, unknown node) or an event is illegal against the current
    /// state (starting a non-ready node, producing while not running,
    /// producing the same artifact/evidence twice, finishing a
    /// non-running node, failing a terminal node, canceling twice) —
    /// never silently accepted.
    pub fn advance(&self, sequence: &GraphEventSequence) -> Result<GraphEvaluation, OrchError> {
        self.validate()?;
        sequence.validate()?;

        // The fold state.
        let mut states: BTreeMap<String, NodeState> = self
            .nodes
            .iter()
            .map(|node| (node.node_id.as_str().to_owned(), node.state))
            .collect();
        // Cleared waits: (node id, wait index).
        let mut cleared: BTreeSet<(String, usize)> = BTreeSet::new();
        let mut attribution: Vec<Attribution> = Vec::new();
        let mut produced_artifacts: BTreeSet<String> = BTreeSet::new();
        let mut produced_evidence: BTreeSet<String> = BTreeSet::new();
        let mut produced_labels: BTreeSet<(String, String)> = BTreeSet::new();
        let mut cancellation: Option<GraphCancellation> = None;

        for event in &sequence.events {
            match event {
                GraphEvent::NodeStarted { node } => {
                    let state = self.checked_state(&states, node)?;
                    if state != NodeState::Ready {
                        return Err(OrchError::invalid(format!(
                            "node {node:?} cannot start from {state:?} — only a ready node can \
                             start"
                        )));
                    }
                    states.insert(node.clone(), NodeState::Running);
                }
                GraphEvent::ArtifactProduced {
                    by_node,
                    artifact,
                    label,
                } => {
                    let state = self.checked_state(&states, by_node)?;
                    if state != NodeState::Running {
                        return Err(OrchError::invalid(format!(
                            "node {by_node:?} cannot produce an artifact while {state:?} — \
                             attribution requires a running node"
                        )));
                    }
                    if !produced_artifacts.insert(artifact.as_str().to_owned()) {
                        return Err(OrchError::invalid(format!(
                            "artifact {:?} was already produced — every artifact is produced \
                             exactly once",
                            artifact.as_str()
                        )));
                    }
                    if !produced_labels.insert((by_node.clone(), label.clone())) {
                        return Err(OrchError::invalid(format!(
                            "node {by_node:?} already produced an artifact labeled {label:?} — \
                             wait labels must be unambiguous"
                        )));
                    }
                    attribution.push(Attribution {
                        node_id: self.checked_node(by_node)?.node_id.clone(),
                        product: Product::Artifact {
                            artifact: artifact.clone(),
                            label: label.clone(),
                        },
                    });
                    self.clear_artifact_waits(by_node, label, &mut cleared);
                    self.unblock_ready(&mut states, &cleared);
                }
                GraphEvent::EvidenceRecorded {
                    by_node,
                    evidence,
                    label,
                } => {
                    let state = self.checked_state(&states, by_node)?;
                    if state != NodeState::Running {
                        return Err(OrchError::invalid(format!(
                            "node {by_node:?} cannot record evidence while {state:?} — \
                             attribution requires a running node"
                        )));
                    }
                    if !produced_evidence.insert(evidence.as_str().to_owned()) {
                        return Err(OrchError::invalid(format!(
                            "evidence {:?} was already recorded — every evidence record is \
                             produced exactly once",
                            evidence.as_str()
                        )));
                    }
                    if !produced_labels.insert((by_node.clone(), label.clone())) {
                        return Err(OrchError::invalid(format!(
                            "node {by_node:?} already produced a product labeled {label:?} — \
                             labels must be unambiguous"
                        )));
                    }
                    attribution.push(Attribution {
                        node_id: self.checked_node(by_node)?.node_id.clone(),
                        product: Product::Evidence {
                            evidence: evidence.clone(),
                            label: label.clone(),
                        },
                    });
                }
                GraphEvent::NodeFinished { node } => {
                    let state = self.checked_state(&states, node)?;
                    if state != NodeState::Running {
                        return Err(OrchError::invalid(format!(
                            "node {node:?} cannot finish from {state:?} — only a running node \
                             can finish"
                        )));
                    }
                    states.insert(node.clone(), NodeState::Done);
                }
                GraphEvent::NodeFailed { node, reason } => {
                    let state = self.checked_state(&states, node)?;
                    if state.is_terminal() {
                        return Err(OrchError::invalid(format!(
                            "node {node:?} is already {state:?} — a terminal node cannot fail"
                        )));
                    }
                    let _ = reason;
                    states.insert(node.clone(), NodeState::Failed);
                }
                GraphEvent::ResourceAvailable { resource } => {
                    self.clear_resource_waits(resource, &mut cleared);
                    self.unblock_ready(&mut states, &cleared);
                }
                GraphEvent::RunCanceled { reason } => {
                    if cancellation.is_some() {
                        return Err(OrchError::invalid(
                            "a run is canceled at most once per evaluation",
                        ));
                    }
                    let mut canceled = Vec::new();
                    let mut settled = Vec::new();
                    for node in &self.nodes {
                        let id = node.node_id.as_str();
                        let state = states[id];
                        if state.is_terminal() {
                            settled.push(id.to_owned());
                        } else {
                            let products_kept = attribution
                                .iter()
                                .filter(|entry| entry.node_id.as_str() == id)
                                .map(|entry| entry.product.clone())
                                .collect();
                            canceled.push(CanceledNode {
                                node_id: node.node_id.clone(),
                                from_state: state,
                                products_kept,
                            });
                            states.insert(id.to_owned(), NodeState::Failed);
                        }
                    }
                    cancellation = Some(GraphCancellation {
                        v: OrchVersion,
                        task: self.task.clone(),
                        reason: reason.clone(),
                        canceled,
                        settled,
                    });
                }
            }
        }

        // The derived verification overlay: a Done node whose verifier
        // (a node with a scope naming it) has itself reached a
        // terminal-success BASE state is Verified. The verifier's own
        // derived state never cascades — independence is one step deep
        // by construction (a verifier verifies WORK, not verifiers).
        let verified_by: BTreeSet<&str> = self
            .nodes
            .iter()
            .filter(|verifier| {
                let base = states[verifier.node_id.as_str()];
                base.is_success()
                    && verifier
                        .verifier
                        .as_ref()
                        .is_some_and(|scope| !scope.verifies.is_empty())
            })
            .flat_map(|verifier| {
                verifier
                    .verifier
                    .as_ref()
                    .map(|scope| scope.verifies.iter().map(String::as_str))
                    .into_iter()
                    .flatten()
            })
            .collect();

        let mut nodes = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let id = node.node_id.as_str();
            let base = states[id];
            let state = if base == NodeState::Done && verified_by.contains(id) {
                NodeState::Verified
            } else {
                base
            };
            let waiting_on = node
                .inputs
                .waits
                .iter()
                .enumerate()
                .filter(|(index, _)| !cleared.contains(&(id.to_owned(), *index)))
                .map(|(_, wait)| wait.clone())
                .collect();
            nodes.push(NodeEvaluation {
                node_id: node.node_id.clone(),
                state,
                changed: state != node.state,
                waiting_on,
            });
        }

        let all_success = nodes.iter().all(|node| node.state.is_success());
        let merge = if all_success {
            let verified: Vec<String> = nodes
                .iter()
                .filter(|node| node.state == NodeState::Verified)
                .map(|node| node.node_id.as_str().to_owned())
                .collect();
            let done: Vec<String> = nodes
                .iter()
                .filter(|node| node.state == NodeState::Done)
                .map(|node| node.node_id.as_str().to_owned())
                .collect();
            let artifacts = attribution
                .iter()
                .filter_map(|entry| match &entry.product {
                    Product::Artifact { artifact, .. } => Some(artifact.clone()),
                    Product::Evidence { .. } => None,
                })
                .collect();
            let evidence = attribution
                .iter()
                .filter_map(|entry| match &entry.product {
                    Product::Artifact { .. } => None,
                    Product::Evidence { evidence, .. } => Some(evidence.clone()),
                })
                .collect();
            Some(MergePoint {
                v: OrchVersion,
                event_type: event_types::TASK_VERIFICATION_MERGED.to_owned(),
                task: self.task.clone(),
                verified,
                done,
                artifacts,
                evidence,
            })
        } else {
            None
        };

        let evaluation = GraphEvaluation {
            v: OrchVersion,
            task: self.task.clone(),
            nodes,
            attribution,
            cancellation,
            merge,
        };
        Ok(evaluation)
    }

    /// The current state of `node`, erroring on unknown nodes.
    fn checked_state(
        &self,
        states: &BTreeMap<String, NodeState>,
        node: &str,
    ) -> Result<NodeState, OrchError> {
        states
            .get(node)
            .copied()
            .ok_or_else(|| OrchError::invalid(format!("unknown node {node:?}")))
    }

    /// The assignment of `node`, erroring on unknown nodes.
    fn checked_node(&self, node: &str) -> Result<&crate::node::AgentAssignment, OrchError> {
        self.node(node)
            .ok_or_else(|| OrchError::invalid(format!("unknown node {node:?}")))
    }

    /// Clears every artifact wait matching `(from_node, label)`.
    fn clear_artifact_waits(
        &self,
        from_node: &str,
        label: &str,
        cleared: &mut BTreeSet<(String, usize)>,
    ) {
        for node in &self.nodes {
            for (index, wait) in node.inputs.waits.iter().enumerate() {
                if let WaitOn::ArtifactReady {
                    from_node: wait_node,
                    label: wait_label,
                } = wait
                    && wait_node == from_node
                    && wait_label == label
                {
                    cleared.insert((node.node_id.as_str().to_owned(), index));
                }
            }
        }
    }

    /// Clears every resource wait on `resource`.
    fn clear_resource_waits(
        &self,
        resource: &ResourceRef,
        cleared: &mut BTreeSet<(String, usize)>,
    ) {
        for node in &self.nodes {
            for (index, wait) in node.inputs.waits.iter().enumerate() {
                if let WaitOn::ResourceReady {
                    resource: wait_resource,
                    ..
                } = wait
                    && wait_resource == resource
                {
                    cleared.insert((node.node_id.as_str().to_owned(), index));
                }
            }
        }
    }

    /// Moves every Blocked node whose waits have ALL cleared to Ready.
    fn unblock_ready(
        &self,
        states: &mut BTreeMap<String, NodeState>,
        cleared: &BTreeSet<(String, usize)>,
    ) {
        for node in &self.nodes {
            if states[node.node_id.as_str()] != NodeState::Blocked {
                continue;
            }
            let all_cleared = node
                .inputs
                .waits
                .iter()
                .enumerate()
                .all(|(index, _)| cleared.contains(&(node.node_id.as_str().to_owned(), index)));
            if all_cleared && !node.inputs.waits.is_empty() {
                states.insert(node.node_id.as_str().to_owned(), NodeState::Ready);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes;
    use crate::node::{NodeInputs, RoleLabel, VerifierScope};
    use crate::refs::{ActorRef, WaitOn};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn event_kind(event: &GraphEvent) -> &'static str {
        match event {
            GraphEvent::NodeStarted { .. } => "node_started",
            GraphEvent::ArtifactProduced { .. } => "artifact_produced",
            GraphEvent::EvidenceRecorded { .. } => "evidence_recorded",
            GraphEvent::NodeFinished { .. } => "node_finished",
            GraphEvent::NodeFailed { .. } => "node_failed",
            GraphEvent::ResourceAvailable { .. } => "resource_available",
            GraphEvent::RunCanceled { .. } => "run_canceled",
        }
    }

    #[test]
    fn events_serialize_internally_tagged_and_reject_unknown_kinds() {
        let started = GraphEvent::NodeStarted {
            node: "research".to_owned(),
        };
        assert_eq!(
            ok(serde_json::to_string(&started)),
            "{\"kind\":\"node_started\",\"node\":\"research\"}"
        );
        let produced = GraphEvent::ArtifactProduced {
            by_node: "research".to_owned(),
            artifact: ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPA0")),
            label: "the research notes".to_owned(),
        };
        let serialized = ok(serde_json::to_string(&produced));
        assert_eq!(
            serialized,
            concat!(
                "{\"kind\":\"artifact_produced\",",
                "\"by_node\":\"research\",",
                "\"artifact\":\"art_01J8ZQ5V8K3T2B7N6X4R9DQPA0\",",
                "\"label\":\"the research notes\"}"
            )
        );
        let reloaded: GraphEvent = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, produced);
        assert!(
            serde_json::from_str::<GraphEvent>(
                "{\"kind\":\"transcript_relayed\",\"node\":\"research\"}"
            )
            .is_err(),
            "unknown event kinds are rejected"
        );
        assert_eq!(event_kind(&started), "node_started");
        assert_eq!(started.node(), Some("research"));
        assert!(
            GraphEvent::RunCanceled {
                reason: "the user asked to stop".to_owned()
            }
            .node()
            .is_none()
        );
    }

    #[test]
    fn sequences_are_bounded_and_canonical() {
        assert!(GraphEventSequence::new(vec![]).is_ok());
        let too_many: Vec<GraphEvent> = (0..MAX_EVENTS + 1)
            .map(|_| GraphEvent::ResourceAvailable {
                resource: ok(ResourceRef::parse("res_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
            })
            .collect();
        assert!(GraphEventSequence::new(too_many).is_err());
        let sequence = ok(GraphEventSequence::new(vec![
            GraphEvent::NodeStarted {
                node: "research".to_owned(),
            },
            GraphEvent::NodeStarted {
                node: "research".to_owned(),
            },
        ]));
        let serialized = ok(serde_json::to_string(&sequence));
        assert!(serialized.starts_with("{\"v\":1,\"events\":["));
        let reloaded: GraphEventSequence = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, sequence);
        assert!(
            GraphEventSequence::new(vec![GraphEvent::NodeStarted {
                node: "".to_owned()
            }])
            .is_err()
        );
    }

    #[test]
    fn the_fake_happy_path_merges_with_independent_verification() {
        let (graph, sequence) = fakes::fake_research_analysis_review_run();
        let evaluation = ok(graph.advance(&sequence));
        assert!(evaluation.cancellation.is_none());
        let merge = evaluation
            .merge
            .as_ref()
            .unwrap_or_else(|| panic!("the happy path merges"));
        ok(merge.validate());
        assert_eq!(merge.event_type, event_types::TASK_VERIFICATION_MERGED);
        assert_eq!(
            merge.verified,
            vec!["analysis".to_owned(), "research".to_owned()],
            "the reviewer independently verified both workers"
        );
        assert_eq!(merge.done, vec!["review".to_owned()]);
        assert_eq!(merge.artifacts.len(), 2);
        assert_eq!(merge.evidence.len(), 1);
        // The J-09 law at the node level: verified is distinct from done.
        let state = |id: &str| {
            evaluation
                .nodes
                .iter()
                .find(|node| node.node_id.as_str() == id)
                .map(|node| node.state)
                .unwrap_or_else(|| panic!("known node {id}"))
        };
        assert_eq!(state("research"), NodeState::Verified);
        assert_eq!(state("analysis"), NodeState::Verified);
        assert_eq!(state("review"), NodeState::Done);
        // Every node's outcome is present, canonically ordered.
        let ids: Vec<&str> = evaluation
            .nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect();
        assert_eq!(ids, vec!["analysis", "research", "review"]);
        // No blocked waits remain.
        assert!(
            evaluation
                .nodes
                .iter()
                .all(|node| node.waiting_on.is_empty())
        );
    }

    #[test]
    fn the_fake_blocked_case_unblocks_only_when_the_wait_clears() {
        let (graph, prefix) = fakes::fake_blocked_resolution_run();
        // Mid-run: research done, analysis unblocked to ready, the
        // reviewer still blocked — NAMING the analysis notes it waits
        // for (a blocked node names what it waits for).
        let evaluation = ok(graph.advance(&prefix));
        let state = |id: &str| {
            evaluation
                .nodes
                .iter()
                .find(|node| node.node_id.as_str() == id)
                .map(|node| (node.state, node.waiting_on.clone()))
                .unwrap_or_else(|| panic!("known node {id}"))
        };
        assert_eq!(state("research").0, NodeState::Done);
        assert_eq!(state("analysis").0, NodeState::Ready);
        let (review_state, review_waits) = state("review");
        assert_eq!(review_state, NodeState::Blocked);
        assert_eq!(review_waits.len(), 1);
        assert_eq!(review_waits[0].label(), fakes::FAKE_ANALYSIS_NOTES_LABEL);
        assert!(
            evaluation.merge.is_none(),
            "a blocked reviewer means no merge yet"
        );

        // The resolution: the tail clears the last wait, the reviewer
        // runs and finishes, and the run merges.
        let mut events = prefix.events.clone();
        events.extend(fakes::fake_blocked_resolution_tail().events);
        let resolved = ok(graph.advance(&ok(GraphEventSequence::new(events))));
        let merge = resolved
            .merge
            .as_ref()
            .unwrap_or_else(|| panic!("the resolved run merges"));
        assert_eq!(
            merge.verified,
            vec!["analysis".to_owned(), "research".to_owned()],
            "both workers are independently verified after the reviewer finishes"
        );
        assert_eq!(merge.done, vec!["review".to_owned()]);
        assert!(
            resolved.nodes.iter().all(|node| node.waiting_on.is_empty()),
            "no blocked waits remain after the resolution"
        );
    }

    #[test]
    fn producing_while_blocked_is_rejected_not_silently_accepted() {
        let graph = one_node_graph();
        let invalid = ok(GraphEventSequence::new(vec![
            GraphEvent::ArtifactProduced {
                by_node: "research".to_owned(),
                artifact: ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPA0")),
                label: "the research notes".to_owned(),
            },
        ]));
        assert!(
            graph.advance(&invalid).is_err(),
            "a ready (not running) node cannot produce — attribution requires a running node"
        );
    }

    #[test]
    fn the_same_artifact_cannot_be_produced_twice() {
        let graph = one_node_graph();
        let duplicate = ok(GraphEventSequence::new(vec![
            GraphEvent::NodeStarted {
                node: "research".to_owned(),
            },
            GraphEvent::ArtifactProduced {
                by_node: "research".to_owned(),
                artifact: ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPA0")),
                label: "the research notes".to_owned(),
            },
            GraphEvent::ArtifactProduced {
                by_node: "research".to_owned(),
                artifact: ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPA0")),
                label: "the revised notes".to_owned(),
            },
        ]));
        assert!(
            graph.advance(&duplicate).is_err(),
            "every artifact is produced exactly once"
        );
    }

    #[test]
    fn cancellation_propagates_and_keeps_settled_work() {
        let (graph, _) = fakes::fake_research_analysis_review_run();
        // Cancel after research has finished but before analysis runs.
        let partial = ok(GraphEventSequence::new(vec![
            GraphEvent::NodeStarted {
                node: "research".to_owned(),
            },
            GraphEvent::ArtifactProduced {
                by_node: "research".to_owned(),
                artifact: ok(ArtifactRef::parse("art_01J8ZQ5V8K3T2B7N6X4R9DQPA0")),
                label: "the research notes".to_owned(),
            },
            GraphEvent::NodeFinished {
                node: "research".to_owned(),
            },
            GraphEvent::RunCanceled {
                reason: "the user asked to stop".to_owned(),
            },
        ]));
        let evaluation = ok(graph.advance(&partial));
        assert!(evaluation.merge.is_none(), "a canceled run never merges");
        let cancellation = evaluation
            .cancellation
            .as_ref()
            .unwrap_or_else(|| panic!("recorded"));
        assert_eq!(cancellation.reason, "the user asked to stop");
        // research settled (done); analysis and review were canceled.
        assert_eq!(cancellation.settled, vec!["research".to_owned()]);
        let canceled_ids: Vec<&str> = cancellation
            .canceled
            .iter()
            .map(|node| node.node_id.as_str())
            .collect();
        assert_eq!(canceled_ids, vec!["analysis", "review"]);
        // analysis was canceled from Ready (the research notes unblocked
        // it); review was canceled from Blocked (the analysis notes never
        // arrived) — a blocked node names what it waits for, even canceled.
        assert_eq!(cancellation.canceled[0].from_state, NodeState::Ready);
        assert_eq!(cancellation.canceled[1].from_state, NodeState::Blocked);
        // The canceled nodes land in Failed; their kept products survive.
        let state = |id: &str| {
            evaluation
                .nodes
                .iter()
                .find(|node| node.node_id.as_str() == id)
                .map(|node| node.state)
                .unwrap_or_else(|| panic!("known node {id}"))
        };
        assert_eq!(state("analysis"), NodeState::Failed);
        assert_eq!(state("review"), NodeState::Failed);
        assert_eq!(state("research"), NodeState::Done);
        assert_eq!(evaluation.attribution.len(), 1, "kept work is kept");

        // A second cancellation in the same sequence is rejected.
        let twice = ok(GraphEventSequence::new(vec![
            GraphEvent::RunCanceled {
                reason: "first".to_owned(),
            },
            GraphEvent::RunCanceled {
                reason: "second".to_owned(),
            },
        ]));
        assert!(
            graph.advance(&twice).is_err(),
            "a run is canceled at most once per evaluation"
        );
    }

    #[test]
    fn evaluations_are_deterministic_functions_of_their_inputs() {
        let (graph, sequence) = fakes::fake_research_analysis_review_run();
        let first = ok(graph.advance(&sequence));
        let second = ok(graph.advance(&sequence));
        assert_eq!(
            ok(serde_json::to_string_pretty(&first)),
            ok(serde_json::to_string_pretty(&second)),
            "identical inputs produce byte-identical evaluations"
        );
        // And the graph itself is never mutated by evaluation.
        assert_eq!(graph.nodes[0].state, NodeState::Blocked);
    }

    #[test]
    fn failures_are_local_and_honest() {
        let (graph, _) = fakes::fake_research_analysis_review_run();
        let failed = ok(GraphEventSequence::new(vec![GraphEvent::NodeFailed {
            node: "research".to_owned(),
            reason: "the browser session expired".to_owned(),
        }]));
        let evaluation = ok(graph.advance(&failed));
        assert!(evaluation.merge.is_none());
        let analysis = evaluation
            .nodes
            .iter()
            .find(|node| node.node_id.as_str() == "analysis")
            .unwrap_or_else(|| panic!("known node"));
        assert_eq!(
            analysis.state,
            NodeState::Blocked,
            "a sibling failure does not cascade — the blocked node still names its wait"
        );
        assert_eq!(analysis.waiting_on.len(), 1);
        assert_eq!(analysis.waiting_on[0].label(), "the research notes");
        // Failing a terminal node is rejected.
        let twice = ok(GraphEventSequence::new(vec![
            GraphEvent::NodeFailed {
                node: "research".to_owned(),
                reason: "first".to_owned(),
            },
            GraphEvent::NodeFailed {
                node: "research".to_owned(),
                reason: "second".to_owned(),
            },
        ]));
        assert!(graph.advance(&twice).is_err());
    }

    #[test]
    fn merge_descriptors_validate_their_event_type() {
        let (graph, sequence) = fakes::fake_research_analysis_review_run();
        let evaluation = ok(graph.advance(&sequence));
        let mut merge = evaluation
            .merge
            .unwrap_or_else(|| panic!("the happy path merges"));
        ok(merge.validate());
        merge.event_type = "task.done".to_owned();
        assert!(merge.validate().is_err());
    }

    #[test]
    fn resource_waits_clear_on_availability() {
        let browser = ok(ResourceRef::parse("res_01J8ZQ5V8K3T2B7N6X4R9DQPD3"));
        let graph = ok(ExecutionGraph::new(
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            vec![ok(crate::node::AgentAssignment::worker(
                ok(crate::node::NodeId::parse("research")),
                ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4")),
                ok(RoleLabel::parse("Research")),
                ok(NodeInputs::new(vec![ok(WaitOn::resource(
                    browser.clone(),
                    "the shared browser",
                ))])),
                NodeState::Blocked,
                None,
            ))],
            None,
        ));
        // Without the resource: still blocked, naming the wait.
        let waiting = ok(graph.advance(&ok(GraphEventSequence::new(vec![]))));
        assert_eq!(waiting.nodes[0].state, NodeState::Blocked);
        assert_eq!(waiting.nodes[0].waiting_on.len(), 1);
        assert_eq!(waiting.nodes[0].waiting_on[0].label(), "the shared browser");

        // With the resource: unblocked to ready.
        let unblocked = ok(graph.advance(&ok(GraphEventSequence::new(vec![
            GraphEvent::ResourceAvailable {
                resource: browser.clone(),
            },
        ]))));
        assert_eq!(unblocked.nodes[0].state, NodeState::Ready);
        assert!(unblocked.nodes[0].waiting_on.is_empty());
        let _ = VerifierScope::new(vec!["research".to_owned()]);
    }

    fn one_node_graph() -> ExecutionGraph {
        ok(ExecutionGraph::new(
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")),
            vec![ok(crate::node::AgentAssignment::worker(
                ok(crate::node::NodeId::parse("research")),
                ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4")),
                ok(RoleLabel::parse("Research")),
                ok(NodeInputs::new(vec![])),
                NodeState::Ready,
                None,
            ))],
            None,
        ))
    }
}
