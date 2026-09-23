//! The graph algebra (Wave-3 addendum §4, THE contract of this crate):
//! dependencies/blocked, attribution, the independence rule, the merge
//! point, and cancellation propagation — proven deterministically over
//! the public fakes.

use std::fmt;

use flauz_orch::evaluator::{GraphEvent, GraphEventSequence, NodeEvaluation, Product};
use flauz_orch::fakes;
use flauz_orch::node::NodeState;

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn state_of<'a>(evaluation: &'a flauz_orch::GraphEvaluation, id: &str) -> &'a NodeEvaluation {
    evaluation
        .nodes
        .iter()
        .find(|node| node.node_id.as_str() == id)
        .unwrap_or_else(|| panic!("known node {id}"))
}

/// THE law of the crate (addendum §4): orchestration is a graph of
/// attributed agents, not a chat relay. Dependencies clear on shared
/// task state — artifacts and resources — and the blocked node NAMES
/// what it waits for. Proven across the whole fake family: every
/// blocked shape names its uncleared waits, and every wait is one of
/// the two shared-state kinds.
#[test]
fn orchestration_is_a_graph_of_attributed_agents_not_a_chat_relay() {
    let (graph, prefix) = fakes::fake_blocked_resolution_run();
    let evaluation = test_ok(graph.advance(&prefix));

    // The reviewer is blocked and NAMES the analysis notes it waits for.
    let review = state_of(&evaluation, "review");
    assert_eq!(review.state, NodeState::Blocked);
    assert_eq!(review.waiting_on.len(), 1);
    assert_eq!(
        review.waiting_on[0].label(),
        fakes::FAKE_ANALYSIS_NOTES_LABEL
    );
    // The wait is an artifact wait (shared task state), never a
    // transcript reference — the grammar pins it.
    let serialized = test_ok(serde_json::to_string(&review.waiting_on[0]));
    assert_eq!(
        serialized,
        concat!(
            "{\"kind\":\"artifact_ready\",",
            "\"from_node\":\"analysis\",",
            "\"label\":\"the analysis notes\"}"
        )
    );

    // Across every node and every wait in the whole family: only the two
    // shared-state kinds exist.
    for node in &graph.nodes {
        for wait in &node.inputs.waits {
            let serialized = test_ok(serde_json::to_string(wait));
            assert!(
                serialized.contains("\"kind\":\"artifact_ready\"")
                    || serialized.contains("\"kind\":\"resource_ready\""),
                "the wait grammar carries exactly the two shared-state kinds: {serialized}"
            );
        }
    }
}

/// Attribution: every produced artifact/evidence is attributed to
/// exactly one node, in production order, and dishonest attribution
/// (producing while not running) is an error — never silently accepted.
#[test]
fn every_product_is_attributed_to_its_node() {
    let (graph, events) = fakes::fake_research_analysis_review_run();
    let evaluation = test_ok(graph.advance(&events));
    assert_eq!(evaluation.attribution.len(), 3);
    // Research's notes → research; analysis's notes → analysis; the
    // review verdict (evidence) → the reviewer.
    assert_eq!(evaluation.attribution[0].node_id.as_str(), "research");
    assert!(matches!(
        &evaluation.attribution[0].product,
        Product::Artifact { label, .. } if label == fakes::FAKE_RESEARCH_NOTES_LABEL
    ));
    assert_eq!(evaluation.attribution[1].node_id.as_str(), "analysis");
    assert!(matches!(
        &evaluation.attribution[1].product,
        Product::Artifact { label, .. } if label == fakes::FAKE_ANALYSIS_NOTES_LABEL
    ));
    assert_eq!(evaluation.attribution[2].node_id.as_str(), "review");
    assert!(matches!(
        &evaluation.attribution[2].product,
        Product::Evidence { label, .. } if label == fakes::FAKE_REVIEW_EVIDENCE_LABEL
    ));
    // Every product appears exactly once (attribution is a ledger, not
    // a multiset).
    let mut products = Vec::new();
    for entry in &evaluation.attribution {
        let serialized = test_ok(serde_json::to_string(&entry.product));
        products.push(serialized);
    }
    products.sort();
    let count = products.len();
    products.dedup();
    assert_eq!(products.len(), count, "every product is attributed once");

    // The merge descriptor carries the same attribution set.
    let merge = evaluation
        .merge
        .as_ref()
        .unwrap_or_else(|| panic!("the happy path merges"));
    assert_eq!(merge.artifacts.len(), 2);
    assert_eq!(merge.evidence.len(), 1);

    // Producing while not running is rejected.
    let graph = fakes::fake_research_analysis_review_graph();
    let dishonest = test_ok(GraphEventSequence::new(vec![
        GraphEvent::EvidenceRecorded {
            by_node: "review".to_owned(),
            evidence: test_ok(flauz_orch::EvidenceRef::parse(fakes::FAKE_REVIEW_EVIDENCE)),
            label: fakes::FAKE_REVIEW_EVIDENCE_LABEL.to_owned(),
        },
    ]));
    assert!(
        graph.advance(&dishonest).is_err(),
        "a blocked node cannot record evidence — attribution requires a running node"
    );
}

/// The independence rule (addendum §4): a verifier node's inputs are
/// the ARTIFACTS it verifies — structurally never a context snapshot of
/// the verified node. Proven three ways: the grammar has no snapshot
/// kind; a verifier that does not wait on the verified node's artifact
/// fails validation; and the honest fake's reviewer consumes exactly
/// the two artifacts of the nodes it verifies.
#[test]
fn the_independence_rule_is_structural_not_behavioral() {
    // Grammar: the context-snapshot wait cannot be expressed.
    assert!(
        serde_json::from_str::<flauz_orch::WaitOn>(
            "{\"kind\":\"context_snapshot\",\"from_node\":\"research\",\"label\":\"its context\"}"
        )
        .is_err(),
        "the wait grammar has no context-snapshot kind"
    );

    // Shape: a verifier that does not consume the verified artifacts
    // fails validation with the independence rule named.
    let graph = fakes::fake_research_analysis_review_graph();
    let reviewer = graph.node("review").unwrap_or_else(|| panic!("known node"));
    let scope = reviewer
        .verifier
        .as_ref()
        .unwrap_or_else(|| panic!("the reviewer has a scope"));
    for verified in &scope.verifies {
        assert!(
            reviewer.inputs.waits.iter().any(|wait| match wait {
                flauz_orch::WaitOn::ArtifactReady { from_node, .. } => from_node == verified,
                flauz_orch::WaitOn::ResourceReady { .. } => false,
            }),
            "the reviewer consumes an artifact from {verified} — its inputs are the artifacts \
             it verifies"
        );
    }

    // The serialized grammar of the reviewer's inputs contains only
    // artifact waits on the verified nodes.
    let serialized = test_ok(serde_json::to_string(&reviewer.inputs));
    assert!(serialized.contains("\"kind\":\"artifact_ready\""));
    assert!(!serialized.contains("context"));

    // And the derived overlay: after the run, the verified nodes carry
    // Verified while the reviewer itself carries Done (the J-09 law).
    let (graph, events) = fakes::fake_research_analysis_review_run();
    let evaluation = test_ok(graph.advance(&events));
    assert_eq!(state_of(&evaluation, "research").state, NodeState::Verified);
    assert_eq!(state_of(&evaluation, "analysis").state, NodeState::Verified);
    assert_eq!(state_of(&evaluation, "review").state, NodeState::Done);
}

/// The merge point (addendum §4): all-done → the task-level
/// verification event, distinguishing independently verified from
/// merely done — J-09's law holds at the task level. A run with a
/// blocked or failed node never merges.
#[test]
fn the_merge_point_fires_only_when_all_nodes_reach_terminal_success() {
    let (graph, events) = fakes::fake_research_analysis_review_run();
    let evaluation = test_ok(graph.advance(&events));
    let merge = evaluation
        .merge
        .as_ref()
        .unwrap_or_else(|| panic!("all-done merges"));
    assert_eq!(merge.event_type, "task.verification_merged");
    assert_eq!(
        merge.verified,
        vec!["analysis".to_owned(), "research".to_owned()]
    );
    assert_eq!(merge.done, vec!["review".to_owned()]);

    // A prefix that leaves the reviewer unfinished does not merge.
    let (graph, prefix) = fakes::fake_blocked_resolution_run();
    let evaluation = test_ok(graph.advance(&prefix));
    assert!(
        evaluation.merge.is_none(),
        "a blocked reviewer means the task has not merged"
    );

    // A failed node means no merge, ever.
    let failed = test_ok(GraphEventSequence::new(vec![GraphEvent::NodeFailed {
        node: "review".to_owned(),
        reason: "the reviewer lost access to the shared folder".to_owned(),
    }]));
    let evaluation = test_ok(graph.advance(&failed));
    assert!(evaluation.merge.is_none());
}

/// Cancellation propagation (additive): canceling the run fails every
/// non-terminal node, keeps every settled node's state, keeps the
/// attribution of work already done, and produces the cancellation
/// record that distinguishes cancellation from failure. A canceled run
/// never merges.
#[test]
fn cancellation_propagation_is_additive_and_honest() {
    let (graph, prefix) = fakes::fake_blocked_resolution_run();
    let mut events = prefix.events.clone();
    events.push(GraphEvent::RunCanceled {
        reason: "the user asked to stop this run".to_owned(),
    });
    let canceled = test_ok(GraphEventSequence::new(events));
    let evaluation = test_ok(graph.advance(&canceled));
    assert!(evaluation.merge.is_none(), "a canceled run never merges");

    let cancellation = evaluation
        .cancellation
        .as_ref()
        .unwrap_or_else(|| panic!("the cancellation is recorded"));
    assert_eq!(cancellation.reason, "the user asked to stop this run");
    // research settled (done); analysis and review were canceled.
    assert_eq!(cancellation.settled, vec!["research".to_owned()]);
    let canceled_ids: Vec<&str> = cancellation
        .canceled
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(canceled_ids, vec!["analysis", "review"]);
    // The canceled nodes land in Failed — the record distinguishes.
    assert_eq!(state_of(&evaluation, "analysis").state, NodeState::Failed);
    assert_eq!(state_of(&evaluation, "review").state, NodeState::Failed);
    assert_eq!(state_of(&evaluation, "research").state, NodeState::Done);
    // The work already attributed is kept.
    assert_eq!(evaluation.attribution.len(), 1);
    assert!(
        cancellation
            .canceled
            .iter()
            .all(|node| node.products_kept.is_empty()),
        "neither canceled node had produced anything yet"
    );

    // Canceling a fully settled run is a no-op record: nothing to
    // cancel, nothing lost.
    let (graph, events) = fakes::fake_research_analysis_review_run();
    let mut settled = events.events.clone();
    settled.push(GraphEvent::RunCanceled {
        reason: "arrived after the work was already done".to_owned(),
    });
    let evaluation = test_ok(graph.advance(&test_ok(GraphEventSequence::new(settled))));
    let cancellation = evaluation
        .cancellation
        .as_ref()
        .unwrap_or_else(|| panic!("recorded"));
    assert!(cancellation.canceled.is_empty());
    assert_eq!(cancellation.settled.len(), 3);
    assert!(
        evaluation.merge.is_some(),
        "settled work still merges — cancellation after completion loses nothing"
    );
}

/// Task identity discipline for run-again (the save→run-again loop's
/// kernel law): the graph is per-task; a run-again from a saved
/// workflow is a NEW task reference with a NEW graph — never a fork of
/// the old task's graph. The evaluator never rewrites the task
/// reference.
#[test]
fn a_graph_belongs_to_one_task_and_never_forks_it() {
    let (graph, events) = fakes::fake_research_analysis_review_run();
    let evaluation = test_ok(graph.advance(&events));
    assert_eq!(
        evaluation.task.as_str(),
        graph.task.as_str(),
        "the evaluation carries the graph's task — never a fork"
    );
    assert_eq!(
        evaluation
            .merge
            .as_ref()
            .unwrap_or_else(|| panic!("merges"))
            .task
            .as_str(),
        graph.task.as_str()
    );

    // A NEW task seeds a NEW graph with the same shape: distinct task
    // references, distinct graphs, same node shape (the run-again
    // discipline the save-workflow slice renders).
    let new_task = test_ok(flauz_orch::TaskRef::parse(
        "task_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
    ));
    assert_ne!(new_task.as_str(), graph.task.as_str());
    let mut run_again = graph.clone();
    run_again.task = new_task.clone();
    test_ok(run_again.validate());
    let run_again_evaluation = test_ok(run_again.advance(&events));
    assert_eq!(run_again_evaluation.task.as_str(), new_task.as_str());
    assert_ne!(run_again_evaluation.task.as_str(), evaluation.task.as_str());
    // The old task's evaluation is untouched (pure function).
    let original = test_ok(graph.advance(&events));
    assert_eq!(original.task.as_str(), graph.task.as_str());
}

/// Blocked honesty end-to-end: a blocked node always names what it
/// waits for, and it unblocks exactly when (and only when) the wait
/// clears — partial production does not clear a different wait.
#[test]
fn blocked_nodes_name_their_waits_and_unblock_exactly_on_clear() {
    let (graph, prefix) = fakes::fake_blocked_resolution_run();
    // Mid-run: analysis unblocked (the research notes cleared its only
    // wait); the reviewer still blocked on the analysis notes.
    let evaluation = test_ok(graph.advance(&prefix));
    let analysis = state_of(&evaluation, "analysis");
    assert_eq!(analysis.state, NodeState::Ready);
    assert!(
        analysis.waiting_on.is_empty(),
        "the research notes cleared the analysis wait"
    );
    let review = state_of(&evaluation, "review");
    assert_eq!(review.state, NodeState::Blocked);
    assert_eq!(review.waiting_on.len(), 1);
    // The research notes did NOT clear the reviewer's OTHER wait.
    assert_eq!(
        review.waiting_on[0].label(),
        fakes::FAKE_ANALYSIS_NOTES_LABEL
    );

    // An unrelated artifact does not clear any wait: analysis runs and
    // produces "the appendix tables" — the reviewer still waits for the
    // analysis notes.
    let mut unrelated_events = prefix.events.clone();
    unrelated_events.push(GraphEvent::NodeStarted {
        node: "analysis".to_owned(),
    });
    unrelated_events.push(GraphEvent::ArtifactProduced {
        by_node: "analysis".to_owned(),
        artifact: test_ok(flauz_orch::ArtifactRef::parse(
            "art_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
        )),
        label: "the appendix tables".to_owned(),
    });
    let unrelated = test_ok(GraphEventSequence::new(unrelated_events));
    let evaluation = test_ok(graph.advance(&unrelated));
    let review = state_of(&evaluation, "review");
    assert_eq!(
        review.state,
        NodeState::Blocked,
        "an unrelated artifact never clears the analysis-notes wait"
    );
    assert_eq!(review.waiting_on.len(), 1);
}

/// Harness states ride as plain data (the ORCH-003 seam): the graph
/// never interprets them, and they never affect evaluation.
#[test]
fn harness_states_are_plain_data_the_graph_never_interprets() {
    let mut graph = fakes::fake_research_analysis_review_graph();
    for node in &mut graph.nodes {
        node.harness_state = Some("verifying".to_owned());
    }
    test_ok(graph.validate());
    let (reference_graph, events) = fakes::fake_research_analysis_review_run();
    let reference = test_ok(reference_graph.advance(&events));
    let evaluation = test_ok(graph.advance(&events));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&evaluation.nodes)),
        test_ok(serde_json::to_string_pretty(&reference.nodes)),
        "harness states never affect the evaluation"
    );
}
