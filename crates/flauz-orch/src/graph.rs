//! The execution graph: the attributed multi-agent graph for one task
//! (addendum §4) — validated shape, no engine coupling.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::node::{AgentAssignment, NodeState};
use crate::refs::{TaskRef, WaitOn};
use crate::{MAX_NODES, OrchError, OrchVersion, ensure_list_bound, ensure_str_bound};

/// The orchestration graph of ONE task: its agent assignments (who,
/// role, dependencies, state, verifier scopes). Pure data + validation
/// — the evaluator ([`ExecutionGraph::advance`]) computes over it and
/// never mutates it; every produced artifact/evidence is attributed to
/// its node by the evaluation's ledger, not by the graph.
///
/// Validation enforces the graph laws:
///
/// - **unique nodes** — node ids are unique;
/// - **known targets** — every dependency names a node in the graph;
/// - **acyclic** — dependency edges form a DAG (an artifact wait on
///   `from_node` is an edge from that node; a resource wait is not a
///   node edge);
/// - **the independence rule** — a verifier node waits on at least one
///   artifact from EVERY node it verifies, and never verifies itself;
/// - **blocked honesty** — a node with uncleared waits at rest carries
///   `Blocked` (or a terminal state), never `Ready`/`Running`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionGraph {
    /// Contract schema version (`"v": 1`).
    pub v: OrchVersion,
    /// The task this graph orchestrates. Task identity is the canonical
    /// ID (kernel §6): a run-again from a saved workflow seeds a NEW
    /// task — a new graph — never a fork of this one.
    pub task: TaskRef,
    /// The graph's nodes, sorted by node id (canonical order).
    pub nodes: Vec<AgentAssignment>,
    /// An honest, bounded note about what the graph is orchestrating
    /// (the task's objective, for example), or `None`.
    pub objective: Option<String>,
}

impl ExecutionGraph {
    /// Builds a graph from assignments, validating every graph law.
    /// Nodes are canonically ordered by node id on construction.
    pub fn new(
        task: TaskRef,
        mut nodes: Vec<AgentAssignment>,
        objective: Option<&str>,
    ) -> Result<Self, OrchError> {
        task.validate()?;
        ensure_list_bound("graph nodes", nodes.len(), MAX_NODES)?;
        if nodes.is_empty() {
            return Err(OrchError::invalid("a graph carries at least one node"));
        }
        if let Some(objective) = objective {
            crate::ensure_non_empty("graph objective", objective)?;
            ensure_str_bound("graph objective", objective, crate::MAX_EXPLANATION_BYTES)?;
        }
        for node in &nodes {
            node.validate()?;
        }
        // Canonical order + uniqueness.
        nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        let ids: Vec<&str> = nodes.iter().map(|node| node.node_id.as_str()).collect();
        let unique: BTreeSet<&str> = ids.iter().copied().collect();
        if unique.len() != ids.len() {
            return Err(OrchError::invalid("node ids must be unique within a graph"));
        }
        let graph = Self {
            v: OrchVersion,
            task,
            nodes,
            objective: objective.map(str::to_owned),
        };
        graph.validate()?;
        Ok(graph)
    }

    /// Validates the graph against the canonical rules and the graph
    /// laws (unique nodes, known targets, acyclic, independence,
    /// blocked honesty).
    pub fn validate(&self) -> Result<(), OrchError> {
        self.task.validate()?;
        ensure_list_bound("graph nodes", self.nodes.len(), MAX_NODES)?;
        if self.nodes.is_empty() {
            return Err(OrchError::invalid("a graph carries at least one node"));
        }
        if let Some(objective) = &self.objective {
            crate::ensure_non_empty("graph objective", objective)?;
            ensure_str_bound("graph objective", objective, crate::MAX_EXPLANATION_BYTES)?;
        }
        let known: BTreeSet<&str> = self
            .nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect();
        if known.len() != self.nodes.len() {
            return Err(OrchError::invalid("node ids must be unique within a graph"));
        }
        // Canonical ordering.
        let sorted = self
            .nodes
            .windows(2)
            .all(|pair| pair[0].node_id < pair[1].node_id);
        if !sorted {
            return Err(OrchError::invalid("graph nodes are sorted by node id"));
        }
        for node in &self.nodes {
            node.validate()?;
            // Known targets.
            for wait in &node.inputs.waits {
                if let WaitOn::ArtifactReady { from_node, .. } = wait
                    && !known.contains(from_node.as_str())
                {
                    return Err(OrchError::invalid(format!(
                        "node {:?} waits on unknown node {from_node:?}",
                        node.node_id.as_str()
                    )));
                }
            }
            // The independence rule.
            if let Some(verifier) = &node.verifier {
                if verifier
                    .verifies
                    .contains(&node.node_id.as_str().to_owned())
                {
                    return Err(OrchError::invalid(format!(
                        "node {:?} cannot verify itself — independence is structural",
                        node.node_id.as_str()
                    )));
                }
                for verified in &verifier.verifies {
                    if !known.contains(verified.as_str()) {
                        return Err(OrchError::invalid(format!(
                            "verifier {:?} verifies unknown node {verified:?}",
                            node.node_id.as_str()
                        )));
                    }
                    let waits_on_artifact = node.inputs.waits.iter().any(|wait| match wait {
                        WaitOn::ArtifactReady { from_node, .. } => from_node == verified,
                        WaitOn::ResourceReady { .. } => false,
                    });
                    if !waits_on_artifact {
                        return Err(OrchError::invalid(format!(
                            "verifier {:?} must wait on an artifact from {verified:?} — a \
                             verifier's inputs are the artifacts it verifies, never a context \
                             snapshot",
                            node.node_id.as_str()
                        )));
                    }
                }
            }
        }
        // Acyclic (Kahn's algorithm over artifact-wait edges).
        self.assert_acyclic()?;
        // Blocked honesty: a node whose waits reference other nodes'
        // artifacts cannot rest Ready/Running (it must be Blocked or
        // terminal). Initial states are validated against the
        // DECLARED waits, because the artifact ledger starts empty.
        for node in &self.nodes {
            let waits_on_others = node
                .inputs
                .waits
                .iter()
                .any(|wait| matches!(wait, WaitOn::ArtifactReady { .. }));
            if waits_on_others && matches!(node.state, NodeState::Ready | NodeState::Running) {
                return Err(OrchError::invalid(format!(
                    "node {:?} waits on other nodes' artifacts, so it cannot start {} — it is \
                     blocked until they are produced",
                    node.node_id.as_str(),
                    match node.state {
                        NodeState::Ready => "ready",
                        NodeState::Running => "running",
                        _ => unreachable!("the match guards ready/running"),
                    }
                )));
            }
        }
        Ok(())
    }

    /// The node with `node_id`, if any.
    #[must_use]
    pub fn node(&self, node_id: &str) -> Option<&AgentAssignment> {
        self.nodes
            .iter()
            .find(|node| node.node_id.as_str() == node_id)
    }

    /// The verifier nodes that independently verify `node_id`.
    #[must_use]
    pub fn verifiers_of(&self, node_id: &str) -> Vec<&AgentAssignment> {
        self.nodes
            .iter()
            .filter(|node| {
                node.verifier
                    .as_ref()
                    .is_some_and(|scope| scope.verifies.iter().any(|verified| verified == node_id))
            })
            .collect()
    }

    /// Kahn's-algorithm acyclicity check over the artifact-wait edges.
    fn assert_acyclic(&self) -> Result<(), OrchError> {
        let mut indegree = std::collections::BTreeMap::<&str, usize>::new();
        let mut edges = std::collections::BTreeMap::<&str, Vec<&str>>::new();
        for node in &self.nodes {
            indegree.entry(node.node_id.as_str()).or_insert(0);
        }
        for node in &self.nodes {
            for wait in &node.inputs.waits {
                if let WaitOn::ArtifactReady { from_node, .. } = wait {
                    edges
                        .entry(from_node.as_str())
                        .or_default()
                        .push(node.node_id.as_str());
                    *indegree.entry(node.node_id.as_str()).or_insert(0) += 1;
                }
            }
        }
        let mut queue: Vec<&str> = indegree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(node, _)| *node)
            .collect();
        let mut visited = 0usize;
        while let Some(current) = queue.pop() {
            visited += 1;
            if let Some(neighbors) = edges.get(current) {
                for neighbor in neighbors {
                    if let Some(degree) = indegree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor);
                        }
                    }
                }
            }
        }
        if visited != self.nodes.len() {
            return Err(OrchError::invalid(
                "dependency edges must form a DAG — a cycle was found",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeId, NodeInputs, RoleLabel, VerifierScope};
    use crate::refs::{ActorRef, WaitOn};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn id(value: &str) -> NodeId {
        ok(NodeId::parse(value))
    }

    fn actor(handle: &str) -> ActorRef {
        ok(ActorRef::agent(handle))
    }

    fn role(value: &str) -> RoleLabel {
        ok(RoleLabel::parse(value))
    }

    fn task() -> TaskRef {
        ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"))
    }

    fn waits(list: Vec<WaitOn>) -> NodeInputs {
        ok(NodeInputs::new(list))
    }

    fn artifact_wait(from: &str, label: &str) -> WaitOn {
        ok(WaitOn::artifact(from, label))
    }

    #[test]
    fn graphs_are_canonically_ordered_and_round_trip() {
        let graph = ok(ExecutionGraph::new(
            task(),
            vec![
                ok(AgentAssignment::worker(
                    id("analysis"),
                    actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4"),
                    role("Analysis"),
                    waits(vec![artifact_wait("research", "the research notes")]),
                    NodeState::Blocked,
                    None,
                )),
                ok(AgentAssignment::worker(
                    id("research"),
                    actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPF5"),
                    role("Research"),
                    waits(vec![]),
                    NodeState::Ready,
                    None,
                )),
            ],
            Some("Produce the weekly market brief"),
        ));
        assert_eq!(graph.nodes.len(), 2);
        // Nodes are canonically ordered by node id on construction.
        assert_eq!(graph.nodes[0].node_id.as_str(), "analysis");
        assert_eq!(graph.nodes[1].node_id.as_str(), "research");
        let serialized = ok(serde_json::to_string(&graph));
        assert!(serialized.contains("\"task\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\""));
        assert!(serialized.contains("\"v\":1"));
        let reloaded: ExecutionGraph = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, graph);
        assert!(
            serde_json::from_str::<ExecutionGraph>(
                &serialized.replace("\"nodes\":", "\"agents\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn graphs_reject_unknown_targets_duplicates_and_cycles() {
        // Unknown wait target.
        assert!(
            ExecutionGraph::new(
                task(),
                vec![ok(AgentAssignment::worker(
                    id("analysis"),
                    actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4"),
                    role("Analysis"),
                    waits(vec![artifact_wait("ghost", "the research notes")]),
                    NodeState::Blocked,
                    None,
                ))],
                None,
            )
            .is_err(),
            "waiting on an unknown node is rejected"
        );

        // A dependency cycle: research waits on analysis's notes while
        // analysis waits on research's notes.
        let cyclical = vec![
            ok(AgentAssignment::worker(
                id("analysis"),
                actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4"),
                role("Analysis"),
                waits(vec![artifact_wait("research", "the research notes")]),
                NodeState::Blocked,
                None,
            )),
            ok(AgentAssignment::worker(
                id("research"),
                actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPF5"),
                role("Research"),
                waits(vec![artifact_wait("analysis", "the analysis notes")]),
                NodeState::Blocked,
                None,
            )),
        ];
        assert!(
            ExecutionGraph::new(task(), cyclical, None).is_err(),
            "dependency cycles are rejected"
        );

        // Duplicate node ids.
        let duplicated = vec![
            ok(AgentAssignment::worker(
                id("research"),
                actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPF5"),
                role("Research"),
                waits(vec![]),
                NodeState::Ready,
                None,
            )),
            ok(AgentAssignment::worker(
                id("research"),
                actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4"),
                role("Research (again)"),
                waits(vec![]),
                NodeState::Ready,
                None,
            )),
        ];
        assert!(
            ExecutionGraph::new(task(), duplicated, None).is_err(),
            "duplicate node ids are rejected"
        );

        // Empty graph.
        assert!(ExecutionGraph::new(task(), vec![], None).is_err());
        // Oversized graph.
        let too_many: Vec<AgentAssignment> = (0..MAX_NODES + 1)
            .map(|index| {
                ok(AgentAssignment::worker(
                    id(&format!("node-{index:03}")),
                    actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4"),
                    role("Research"),
                    waits(vec![]),
                    NodeState::Ready,
                    None,
                ))
            })
            .collect();
        assert!(ExecutionGraph::new(task(), too_many, None).is_err());
    }

    #[test]
    fn the_independence_rule_is_structural() {
        // A verifier that does NOT wait on the verified node's artifact
        // is rejected — its inputs must BE the artifacts it verifies.
        let snapshot_ish = ok(AgentAssignment::verifier(
            id("review"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6"),
            role("Independent review"),
            waits(vec![]),
            NodeState::Ready,
            ok(VerifierScope::new(vec!["research".to_owned()])),
            None,
        ));
        let research = ok(AgentAssignment::worker(
            id("research"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPF5"),
            role("Research"),
            waits(vec![]),
            NodeState::Ready,
            None,
        ));
        let error = ExecutionGraph::new(task(), vec![research, snapshot_ish], None)
            .err()
            .unwrap_or_else(|| {
                panic!("a verifier without artifact inputs is the context-snapshot shape")
            });
        assert!(
            error.reason().contains("never a context snapshot"),
            "the rejection names the independence rule: {error}"
        );

        // Self-verification is rejected (independence is not behavioral).
        let self_verifier = ok(AgentAssignment::verifier(
            id("review"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6"),
            role("Independent review"),
            waits(vec![artifact_wait("research", "the research notes")]),
            NodeState::Blocked,
            ok(VerifierScope::new(vec!["review".to_owned()])),
            None,
        ));
        let research = ok(AgentAssignment::worker(
            id("research"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPF5"),
            role("Research"),
            waits(vec![]),
            NodeState::Ready,
            None,
        ));
        assert!(
            ExecutionGraph::new(task(), vec![research, self_verifier], None).is_err(),
            "a node cannot verify itself"
        );

        // The honest verifier: waits on the verified nodes' artifacts.
        let verifier = ok(AgentAssignment::verifier(
            id("review"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6"),
            role("Independent review"),
            waits(vec![
                artifact_wait("analysis", "the analysis notes"),
                artifact_wait("research", "the research notes"),
            ]),
            NodeState::Blocked,
            ok(VerifierScope::new(vec![
                "analysis".to_owned(),
                "research".to_owned(),
            ])),
            None,
        ));
        let research = ok(AgentAssignment::worker(
            id("research"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPF5"),
            role("Research"),
            waits(vec![]),
            NodeState::Ready,
            None,
        ));
        let analysis = ok(AgentAssignment::worker(
            id("analysis"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4"),
            role("Analysis"),
            waits(vec![artifact_wait("research", "the research notes")]),
            NodeState::Blocked,
            None,
        ));
        let graph = ok(ExecutionGraph::new(
            task(),
            vec![research, analysis, verifier],
            None,
        ));
        assert_eq!(graph.verifiers_of("research").len(), 1);
        assert_eq!(graph.verifiers_of("analysis").len(), 1);
        assert!(graph.verifiers_of("review").is_empty());
    }

    #[test]
    fn blocked_honesty_rejects_running_waiters() {
        let lying = ok(AgentAssignment::worker(
            id("analysis"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4"),
            role("Analysis"),
            waits(vec![artifact_wait("research", "the research notes")]),
            NodeState::Running,
            None,
        ));
        let research = ok(AgentAssignment::worker(
            id("research"),
            actor("agent_01J8ZQ5V8K3T2B7N6X4R9DQPF5"),
            role("Research"),
            waits(vec![]),
            NodeState::Done,
            None,
        ));
        assert!(
            ExecutionGraph::new(task(), vec![research, lying], None).is_err(),
            "a node waiting on others' artifacts cannot start running in the declared shape"
        );
    }
}
