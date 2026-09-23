//! Graph nodes: agent assignments — who, role, dependencies, state, and
//! the verifier's independence scope (addendum §4).

use serde::{Deserialize, Serialize};

use crate::refs::{ActorRef, WaitOn};
use crate::{
    MAX_DEPENDENCIES, MAX_VERIFIES, OrchError, OrchVersion, ensure_list_bound, ensure_name,
};

/// A stable node identifier within one graph (for example `research`,
/// `analysis`, `review`). Node ids are local to the graph; they are not
/// canonical entity IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    /// Parses and validates a node identifier (non-empty, bounded).
    pub fn parse(value: &str) -> Result<Self, OrchError> {
        ensure_name("node id", value)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the node identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the node identifier.
    pub fn validate(&self) -> Result<(), OrchError> {
        ensure_name("node id", &self.0)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A node's role label (for example `Research`, `Analysis`,
/// `Independent review`) — user-facing vocabulary, bounded prose.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoleLabel(String);

impl RoleLabel {
    /// Parses and validates a role label (non-empty, bounded).
    pub fn parse(value: &str) -> Result<Self, OrchError> {
        ensure_name("role label", value)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the role label string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the role label.
    pub fn validate(&self) -> Result<(), OrchError> {
        ensure_name("role label", &self.0)
    }
}

impl std::fmt::Display for RoleLabel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A node's state in the orchestration graph (addendum §4 — the frozen
/// six-state vocabulary):
///
/// - `Ready` — all waits cleared, runnable now;
/// - `Running` — executing;
/// - `Blocked` — waiting for at least one dependency (a blocked node
///   names WHAT it waits for);
/// - `Done` — finished its work;
/// - `Failed` — failed (a cancellation also lands here, distinguished
///   by the cancellation record, never silently);
/// - `Verified` — done AND independently verified by a verifier node
///   (the derived overlay; the J-09 law: verified is distinct from
///   merely claimed/done).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    /// All waits cleared; runnable now.
    Ready,
    /// Executing.
    Running,
    /// Waiting for at least one dependency.
    Blocked,
    /// Finished its work.
    Done,
    /// Failed (or canceled — distinguished by the cancellation record).
    Failed,
    /// Done and independently verified by a verifier node.
    Verified,
}

impl NodeState {
    /// Whether the state is a terminal-success state (the merge point
    /// requires every node terminal-success).
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Done | Self::Verified)
    }

    /// Whether the state is terminal (no further transitions).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Verified | Self::Failed)
    }
}

/// A verifier node's independence scope (addendum §4): the nodes this
/// node independently verifies.
///
/// The independence rule is STRUCTURAL: a verifier's inputs are the
/// ARTIFACTS it verifies — never a context snapshot of the verified
/// node. [`ExecutionGraph`]'s validation enforces that a node with a
/// non-empty scope waits on at least one artifact from every node it
/// verifies and never verifies itself; the wait grammar itself has no
/// snapshot kind ([`crate::refs::WaitOn`]), so the only route from the
/// verified work to the verifier is the artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifierScope {
    /// The node ids this verifier independently verifies (non-empty,
    /// unique, sorted; none may be the verifier itself).
    pub verifies: Vec<String>,
}

impl VerifierScope {
    /// Builds a verifier scope, validating bounds, uniqueness and
    /// ordering.
    pub fn new(verifies: Vec<String>) -> Result<Self, OrchError> {
        ensure_list_bound("verifier scope", verifies.len(), MAX_VERIFIES)?;
        if verifies.is_empty() {
            return Err(OrchError::invalid(
                "a verifier scope must name at least one verified node",
            ));
        }
        for node in &verifies {
            ensure_name("verified node id", node)?;
        }
        let sorted = verifies.windows(2).all(|pair| pair[0] < pair[1]);
        if !sorted {
            return Err(OrchError::invalid(
                "verifier scope node ids must be sorted and deduplicated",
            ));
        }
        Ok(Self { verifies })
    }

    /// Validates the scope's bounds.
    pub fn validate(&self) -> Result<(), OrchError> {
        Self::new(self.verifies.clone())?;
        Ok(())
    }
}

/// A node's declared inputs (addendum §4): the waits it must clear
/// before running. Workers wait for the artifacts/resources their work
/// consumes; verifiers wait for the artifacts they verify (the
/// structural independence rule — the graph's validation cross-checks
/// the two lists).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeInputs {
    /// The artifact-wait / resource-wait edges this node depends on
    /// (unique, canonically ordered).
    pub waits: Vec<WaitOn>,
}

impl NodeInputs {
    /// Builds node inputs, validating bounds and wait shapes.
    pub fn new(waits: Vec<WaitOn>) -> Result<Self, OrchError> {
        ensure_list_bound("node dependencies", waits.len(), MAX_DEPENDENCIES)?;
        for wait in &waits {
            wait.validate()?;
        }
        Ok(Self { waits })
    }

    /// Validates the inputs' bounds.
    pub fn validate(&self) -> Result<(), OrchError> {
        Self::new(self.waits.clone())?;
        Ok(())
    }
}

/// One node of the orchestration graph: an agent assignment (addendum
/// §4). Who is assigned (an actor reference), the role label, the
/// dependencies (artifact-wait / resource-wait edges), the current
/// state, and — for a verifier node — the independence scope. Everything
/// the node produces is attributed to it by the evaluator's ledger;
/// the assignment itself never carries products.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentAssignment {
    /// Contract schema version (`"v": 1`).
    pub v: OrchVersion,
    /// The stable node id within the graph.
    pub node_id: NodeId,
    /// Who is assigned: the actor reference (the frozen actor grammar).
    pub who: ActorRef,
    /// The role label (`Research`, `Analysis`, `Independent review`, …).
    pub role: RoleLabel,
    /// The node's declared inputs: the artifact/resource waits it must
    /// clear before running.
    pub inputs: NodeInputs,
    /// The node's current state.
    pub state: NodeState,
    /// The verifier's independence scope; `None` for a worker node.
    /// When present, the node is an independent verifier: its inputs
    /// are the artifacts it verifies — structurally never a context
    /// snapshot of the verified nodes.
    pub verifier: Option<VerifierScope>,
    /// The harness state backing this node, as plain data (the
    /// ORCH-003 seam): an opaque, bounded string the graph never
    /// interprets — for example a serialized `prepare → execute → …`
    /// phase label. `None` when no harness is attached.
    pub harness_state: Option<String>,
}

impl AgentAssignment {
    /// Builds a worker assignment (no verifier scope).
    #[allow(clippy::too_many_arguments)]
    pub fn worker(
        node_id: NodeId,
        who: ActorRef,
        role: RoleLabel,
        inputs: NodeInputs,
        state: NodeState,
        harness_state: Option<String>,
    ) -> Result<Self, OrchError> {
        Self::build(node_id, who, role, inputs, state, None, harness_state)
    }

    /// Builds a verifier assignment — an independent reviewer whose
    /// inputs are the artifacts it verifies.
    #[allow(clippy::too_many_arguments)]
    pub fn verifier(
        node_id: NodeId,
        who: ActorRef,
        role: RoleLabel,
        inputs: NodeInputs,
        state: NodeState,
        verifier: VerifierScope,
        harness_state: Option<String>,
    ) -> Result<Self, OrchError> {
        Self::build(
            node_id,
            who,
            role,
            inputs,
            state,
            Some(verifier),
            harness_state,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        node_id: NodeId,
        who: ActorRef,
        role: RoleLabel,
        inputs: NodeInputs,
        state: NodeState,
        verifier: Option<VerifierScope>,
        harness_state: Option<String>,
    ) -> Result<Self, OrchError> {
        let assignment = Self {
            v: OrchVersion,
            node_id,
            who,
            role,
            inputs,
            state,
            verifier,
            harness_state,
        };
        assignment.validate()?;
        Ok(assignment)
    }

    /// Validates the assignment against the canonical rules (bounds,
    /// grammar, shape).
    pub fn validate(&self) -> Result<(), OrchError> {
        self.node_id.validate()?;
        self.who.validate()?;
        self.role.validate()?;
        self.inputs.validate()?;
        if let Some(verifier) = &self.verifier {
            verifier.validate()?;
        }
        if let Some(harness_state) = &self.harness_state {
            ensure_name("harness state", harness_state)?;
        }
        Ok(())
    }

    /// The node's uncleared waits, in declaration order (what a blocked
    /// node names: "Waiting for the research notes").
    #[must_use]
    pub fn waits(&self) -> &[WaitOn] {
        &self.inputs.waits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MAX_EXPLANATION_BYTES, MAX_NAME_BYTES, ensure_explanation};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn worker_fixture() -> AgentAssignment {
        ok(AgentAssignment::worker(
            ok(NodeId::parse("research")),
            ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4")),
            ok(RoleLabel::parse("Research")),
            ok(NodeInputs::new(vec![])),
            NodeState::Ready,
            None,
        ))
    }

    #[test]
    fn node_states_serialize_to_their_snake_case_names() {
        for (state, name) in [
            (NodeState::Ready, "ready"),
            (NodeState::Running, "running"),
            (NodeState::Blocked, "blocked"),
            (NodeState::Done, "done"),
            (NodeState::Failed, "failed"),
            (NodeState::Verified, "verified"),
        ] {
            assert_eq!(ok(serde_json::to_string(&state)), format!("\"{name}\""));
            let reloaded: NodeState = ok(serde_json::from_str(&format!("\"{name}\"")));
            assert_eq!(reloaded, state);
        }
        assert!(
            serde_json::from_str::<NodeState>("\"canceled\"").is_err(),
            "the state vocabulary is the frozen six — cancellation is a record, not a state"
        );
        assert!(NodeState::Verified.is_success());
        assert!(NodeState::Done.is_success());
        assert!(!NodeState::Failed.is_success());
        assert!(NodeState::Failed.is_terminal());
        assert!(!NodeState::Running.is_terminal());
    }

    #[test]
    fn assignments_carry_who_role_and_waits_canonically() {
        let assignment = worker_fixture();
        assert_eq!(assignment.node_id.as_str(), "research");
        assert_eq!(assignment.role.as_str(), "Research");
        assert!(assignment.waits().is_empty());
        let serialized = ok(serde_json::to_string(&assignment));
        assert!(serialized.contains("\"node_id\":\"research\""));
        assert!(serialized.contains("\"role\":\"Research\""));
        assert!(serialized.contains("\"who\":{\"kind\":\"agent\""));
        let reloaded: AgentAssignment = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, assignment);
        assert!(
            serde_json::from_str::<AgentAssignment>(
                &serialized.replace("\"node_id\":", "\"node\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn verifier_scopes_are_bounded_unique_and_sorted() {
        assert!(VerifierScope::new(vec![]).is_err());
        assert!(VerifierScope::new(vec!["research".to_owned()]).is_ok());
        // Unsorted / duplicated scopes are rejected.
        assert!(
            VerifierScope::new(vec!["research".to_owned(), "analysis".to_owned()]).is_err(),
            "scope node ids must be sorted"
        );
        assert!(
            VerifierScope::new(vec!["research".to_owned(), "research".to_owned()]).is_err(),
            "scope node ids must be deduplicated"
        );
        assert!(VerifierScope::new(vec!["x".repeat(MAX_NAME_BYTES + 1)]).is_err());
    }

    #[test]
    fn harness_states_are_opaque_bounded_strings() {
        let mut assignment = worker_fixture();
        assignment.harness_state = Some("executing".to_owned());
        ok(assignment.validate());
        assignment.harness_state = Some("x".repeat(MAX_NAME_BYTES + 1));
        assert!(assignment.validate().is_err());
        assignment.harness_state = Some("".to_owned());
        assert!(assignment.validate().is_err());
    }

    #[test]
    fn explanations_are_bounded_and_non_empty() {
        assert!(ensure_explanation("reason", "").is_err());
        assert!(ensure_explanation("reason", &"x".repeat(MAX_EXPLANATION_BYTES + 1)).is_err());
        assert!(ensure_explanation("reason", "the shared browser was busy").is_ok());
    }
}
