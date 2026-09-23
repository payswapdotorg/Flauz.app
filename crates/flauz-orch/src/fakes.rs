//! Public in-memory fakes: the deterministic conformance surface for the
//! execution graph (kernel §7).
//!
//! Everything here is a pure function of its arguments — no I/O, no
//! wall-clock reads, no randomness, no generated identifiers. The two
//! canonical fake runs cover the shapes the wave gate and the two
//! surfaces drive:
//!
//! - [`fake_research_analysis_review_run`] — the deterministic 3-node
//!   parallel graph (research → analysis, both independently verified
//!   by review) and its full happy-path event sequence, ending in the
//!   merge point;
//! - [`fake_blocked_resolution_run`] — the blocked-resolution case: a
//!   mid-run snapshot where analysis is still blocked waiting for the
//!   research notes, plus the events that resolve the block.
//!
//! Downstream waves (the F6 wave gate, the agents panel wiring, the
//! save-workflow slice) use these fakes as the reference semantics for
//! real graph inputs.

use crate::evaluator::{GraphEvent, GraphEventSequence};
use crate::graph::ExecutionGraph;
use crate::node::{AgentAssignment, NodeId, NodeInputs, NodeState, RoleLabel, VerifierScope};
use crate::refs::{ActorRef, ArtifactRef, EvidenceRef, TaskRef, WaitOn};

/// The fake task the canonical fake runs orchestrate.
pub const FAKE_TASK_ID: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";

/// The fake agents behind the three nodes (deterministic handles).
pub const FAKE_RESEARCH_AGENT: &str = "agent_01J8ZQ5V8K3T2B7N6X4R9DQPE4";
/// The analysis agent's handle.
pub const FAKE_ANALYSIS_AGENT: &str = "agent_01J8ZQ5V8K3T2B7N6X4R9DQPF5";
/// The independent reviewer's handle.
pub const FAKE_REVIEW_AGENT: &str = "agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6";

/// The fake artifacts and evidence of the canonical runs (deterministic
/// canonical IDs — the fakes generate nothing).
pub const FAKE_RESEARCH_NOTES: &str = "art_01J8ZQ5V8K3T2B7N6X4R9DQPA0";
/// The analysis notes artifact.
pub const FAKE_ANALYSIS_NOTES: &str = "art_01J8ZQ5V8K3T2B7N6X4R9DQPB1";
/// The reviewer's verification evidence.
pub const FAKE_REVIEW_EVIDENCE: &str = "evd_01J8ZQ5V8K3T2B7N6X4R9DQPC2";

/// The user-language wait labels of the canonical runs (the same labels
/// the surfaces render).
pub const FAKE_RESEARCH_NOTES_LABEL: &str = "the research notes";
/// The analysis notes' label.
pub const FAKE_ANALYSIS_NOTES_LABEL: &str = "the analysis notes";
/// The review evidence's label.
pub const FAKE_REVIEW_EVIDENCE_LABEL: &str = "the review verdict";

/// The objective of the canonical fake runs (domain-neutral research —
/// the F6 gate's recommended scenario shape).
pub const FAKE_OBJECTIVE: &str = "Research, analyze and independently review the weekly brief";

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("the fake graph family is canonically valid: {error}"),
    }
}

fn node_id(value: &str) -> NodeId {
    ok(NodeId::parse(value))
}

fn actor(handle: &str) -> ActorRef {
    ok(ActorRef::agent(handle))
}

fn role(value: &str) -> RoleLabel {
    ok(RoleLabel::parse(value))
}

fn task() -> TaskRef {
    ok(TaskRef::parse(FAKE_TASK_ID))
}

fn artifact(value: &str) -> ArtifactRef {
    ok(ArtifactRef::parse(value))
}

fn evidence(value: &str) -> EvidenceRef {
    ok(EvidenceRef::parse(value))
}

fn waits(list: Vec<WaitOn>) -> NodeInputs {
    ok(NodeInputs::new(list))
}

fn artifact_wait(from: &str, label: &str) -> WaitOn {
    ok(WaitOn::artifact(from, label))
}

/// Builds the deterministic 3-node research/analysis/review graph: the
/// research agent runs in parallel with nothing, the analysis agent
/// waits for the research notes, and an independent reviewer waits for
/// both artifacts and verifies both producers (the structural
/// independence rule: the reviewer's inputs ARE the artifacts it
/// verifies).
///
/// The declared states are the at-rest start: research `Ready`,
/// analysis and review `Blocked` (they wait for artifacts).
///
/// # Panics
///
/// Panics if the canonical family fails canonical validation (a bug in
/// the fakes, not in caller data).
#[must_use]
pub fn fake_research_analysis_review_graph() -> ExecutionGraph {
    let research = ok(AgentAssignment::worker(
        node_id("research"),
        actor(FAKE_RESEARCH_AGENT),
        role("Research"),
        waits(vec![]),
        NodeState::Ready,
        None,
    ));
    let analysis = ok(AgentAssignment::worker(
        node_id("analysis"),
        actor(FAKE_ANALYSIS_AGENT),
        role("Analysis"),
        waits(vec![artifact_wait("research", FAKE_RESEARCH_NOTES_LABEL)]),
        NodeState::Blocked,
        None,
    ));
    let review = ok(AgentAssignment::verifier(
        node_id("review"),
        actor(FAKE_REVIEW_AGENT),
        role("Independent review"),
        waits(vec![
            artifact_wait("research", FAKE_RESEARCH_NOTES_LABEL),
            artifact_wait("analysis", FAKE_ANALYSIS_NOTES_LABEL),
        ]),
        NodeState::Blocked,
        ok(VerifierScope::new(vec![
            "analysis".to_owned(),
            "research".to_owned(),
        ])),
        None,
    ));
    ok(ExecutionGraph::new(
        task(),
        vec![research, analysis, review],
        Some(FAKE_OBJECTIVE),
    ))
}

/// The full happy-path event sequence of the canonical fake run:
/// research starts, produces its notes and finishes; analysis unblocks,
/// starts, produces its notes and finishes; the reviewer unblocks,
/// starts, records its verification evidence and finishes — the
/// evaluation ends at the merge point with both workers independently
/// verified.
///
/// # Panics
///
/// Panics if the canonical sequence fails canonical validation (a bug
/// in the fakes, not in caller data).
#[must_use]
pub fn fake_research_analysis_review_events() -> GraphEventSequence {
    ok(GraphEventSequence::new(vec![
        GraphEvent::NodeStarted {
            node: "research".to_owned(),
        },
        GraphEvent::ArtifactProduced {
            by_node: "research".to_owned(),
            artifact: artifact(FAKE_RESEARCH_NOTES),
            label: FAKE_RESEARCH_NOTES_LABEL.to_owned(),
        },
        GraphEvent::NodeFinished {
            node: "research".to_owned(),
        },
        GraphEvent::NodeStarted {
            node: "analysis".to_owned(),
        },
        GraphEvent::ArtifactProduced {
            by_node: "analysis".to_owned(),
            artifact: artifact(FAKE_ANALYSIS_NOTES),
            label: FAKE_ANALYSIS_NOTES_LABEL.to_owned(),
        },
        GraphEvent::NodeFinished {
            node: "analysis".to_owned(),
        },
        GraphEvent::NodeStarted {
            node: "review".to_owned(),
        },
        GraphEvent::EvidenceRecorded {
            by_node: "review".to_owned(),
            evidence: evidence(FAKE_REVIEW_EVIDENCE),
            label: FAKE_REVIEW_EVIDENCE_LABEL.to_owned(),
        },
        GraphEvent::NodeFinished {
            node: "review".to_owned(),
        },
    ]))
}

/// The canonical fake run: the 3-node graph and its full happy-path
/// event sequence (the F6 gate's domain-neutral research scenario at
/// the fake level).
///
/// # Panics
///
/// Panics if the canonical family fails canonical validation (a bug in
/// the fakes, not in caller data).
#[must_use]
pub fn fake_research_analysis_review_run() -> (ExecutionGraph, GraphEventSequence) {
    (
        fake_research_analysis_review_graph(),
        fake_research_analysis_review_events(),
    )
}

/// The blocked-resolution case: the mid-run event prefix where research
/// has produced its notes and finished while analysis has NOT yet
/// started (it is unblocked to `Ready` by the notes) and the reviewer
/// is still `Blocked`, naming the analysis notes it waits for.
/// Evaluating the graph over this prefix yields the blocked shape the
/// Agents panel renders (one node done, one ready, one waiting);
/// appending the tail of
/// [`fake_research_analysis_review_events`] resolves the block and
/// reaches the merge point.
///
/// # Panics
///
/// Panics if the canonical family fails canonical validation (a bug in
/// the fakes, not in caller data).
#[must_use]
pub fn fake_blocked_resolution_run() -> (ExecutionGraph, GraphEventSequence) {
    let graph = fake_research_analysis_review_graph();
    let sequence = ok(GraphEventSequence::new(vec![
        GraphEvent::NodeStarted {
            node: "research".to_owned(),
        },
        GraphEvent::ArtifactProduced {
            by_node: "research".to_owned(),
            artifact: artifact(FAKE_RESEARCH_NOTES),
            label: FAKE_RESEARCH_NOTES_LABEL.to_owned(),
        },
        GraphEvent::NodeFinished {
            node: "research".to_owned(),
        },
    ]));
    (graph, sequence)
}

/// The events that resolve the blocked case into the full happy path:
/// analysis runs, produces its notes and finishes (clearing the
/// reviewer's last wait), then the reviewer runs, records its
/// verification evidence and finishes (the merge point).
///
/// # Panics
///
/// Panics if the canonical family fails canonical validation (a bug in
/// the fakes, not in caller data).
#[must_use]
pub fn fake_blocked_resolution_tail() -> GraphEventSequence {
    ok(GraphEventSequence::new(vec![
        GraphEvent::NodeStarted {
            node: "analysis".to_owned(),
        },
        GraphEvent::ArtifactProduced {
            by_node: "analysis".to_owned(),
            artifact: artifact(FAKE_ANALYSIS_NOTES),
            label: FAKE_ANALYSIS_NOTES_LABEL.to_owned(),
        },
        GraphEvent::NodeFinished {
            node: "analysis".to_owned(),
        },
        GraphEvent::NodeStarted {
            node: "review".to_owned(),
        },
        GraphEvent::EvidenceRecorded {
            by_node: "review".to_owned(),
            evidence: evidence(FAKE_REVIEW_EVIDENCE),
            label: FAKE_REVIEW_EVIDENCE_LABEL.to_owned(),
        },
        GraphEvent::NodeFinished {
            node: "review".to_owned(),
        },
    ]))
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
    fn the_fake_graph_is_deterministic_and_canonical() {
        let first = fake_research_analysis_review_graph();
        let second = fake_research_analysis_review_graph();
        assert_eq!(
            ok(serde_json::to_string_pretty(&first)),
            ok(serde_json::to_string_pretty(&second)),
            "the fakes generate nothing — identical on every call"
        );
        ok(first.validate());
        assert_eq!(first.nodes.len(), 3);
        assert_eq!(first.task.as_str(), FAKE_TASK_ID);
        assert_eq!(first.objective.as_deref(), Some(FAKE_OBJECTIVE));
    }

    #[test]
    fn the_fake_event_sequences_are_deterministic() {
        assert_eq!(
            fake_research_analysis_review_events(),
            fake_research_analysis_review_events()
        );
        assert_eq!(
            fake_research_analysis_review_events().events.len(),
            9,
            "3 starts + 2 artifact productions + 1 evidence record + 3 finishes"
        );
    }
}
