//! Public in-memory fakes: the deterministic conformance surface for
//! the takeover fabric (kernel §7).
//!
//! Everything here is a pure function of its arguments — no I/O, no
//! wall-clock reads, no randomness, no generated identifiers. The
//! canonical fake family covers the shapes the J-17 needs-you
//! surface, the conformance tests and the Wave-5 gate harness drive:
//!
//! - [`fake_actors`] — the two-actor family (Ana the human, Dev's
//!   agent whose turn she takes over);
//! - [`env_switch_gate`] — the canonical approval gate (the work
//!   order's exact example: "approve the environment switch", with
//!   the consequence of each side);
//! - [`ana_takes_over_research`] — the canonical takeover record
//!   (the human takes over the research step, the agent's notes
//!   preserved);
//! - [`research_graph`] / [`finished_dependent_graph`] — the
//!   canonical propagation graphs (the research → analysis → report
//!   chain; all-running and finished-dependent variants);
//! - [`cancel_leaf_node`] / [`cancel_mid_run_node`] /
//!   [`cancel_root_node`] / [`cancel_run`] — the propagation
//!   matrix's four cancellation records;
//! - [`propagation_matrix`] — the **complete** fake propagation
//!   space: every cancellation kind × every graph variant, so the
//!   propagation matrix, the dependent-naming law and the
//!   determinism law are proven by exhaustive deterministic
//!   enumeration rather than sampling.

use crate::TakeoverError;
use crate::approval::ApprovalGate;
use crate::cancel::{CancellationRecord, CancelledWhat, GraphNode, GraphShape};
use crate::refs::{ActorRef, AgentRef, ArtifactRef, NodeName, TaskRef};
use crate::takeover::{PreservedArtifact, TakeoverRecord, TakeoverScope};
use crate::time::Timestamp;

/// The fake task whose stream the canonical family's records land on
/// (the kernel §2 frozen `task_` vector).
pub const FAKE_TASK_ID: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";
/// A second fake task (stream isolation).
pub const FAKE_OTHER_TASK_ID: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPF5";
/// Dev's agent — the agent whose turn the human takes over (the
/// kernel §2 frozen `agent_` vector).
pub const FAKE_AGENT_ID: &str = "agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6";
/// Ana's user principal (the human in the loop).
pub const FAKE_HUMAN_ID: &str = "user_ana";
/// The deterministic moment the canonical family decides at.
pub const FAKE_NOW: &str = "2026-09-23T15:04:00Z";

/// The canonical fake artifact ids, in slot order: 0 the agent's
/// research notes (preserved by the takeover; kept by the root
/// cancellation) · 1 the analysis draft (kept by the
/// finished-dependent graph) · 2 the finished report (kept by the
/// finished-dependent graph) · 3 the handback's human-added evidence
/// slot.
pub const FAKE_ARTIFACT_ID_STRINGS: [&str; 4] = [
    "art_01J8ZQ5V8K3T2B7N6X4R9DQPH7",
    "art_01J8ZQ5V8K3T2B7N6X4R9DQPJ8",
    "art_01J8ZQ5V8K3T2B7N6X4R9DQPK9",
    "art_01J8ZQ5V8K3T2B7N6X4R9DQPMA",
];

/// The canonical node names, in the graph's declared order.
pub const FAKE_NODE_NAMES: [&str; 3] = ["research", "analysis", "report"];

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("the fake takeover family is canonically valid: {error}"),
    }
}

/// The two-actor takeover family: Ana (the human) and Dev's agent
/// (whose turn she takes over).
#[derive(Debug, Clone)]
pub struct FakeActors {
    /// Ana — a `user` principal; the human in the loop.
    pub ana: ActorRef,
    /// Dev's agent — the `agent_` actor whose turn is taken over.
    pub dev_agent: AgentRef,
}

impl FakeActors {
    /// Ana's actor reference with the AGENT kind — the negative-space
    /// shape for the attribution-law tests (the same principal, the
    /// wrong kind: a takeover or a decision by it must be refused).
    #[must_use]
    pub fn ana_as_agent(&self) -> ActorRef {
        ok(ActorRef::agent(FAKE_HUMAN_ID))
    }
}

/// The two-actor takeover family.
#[must_use]
pub fn fake_actors() -> FakeActors {
    FakeActors {
        ana: ok(ActorRef::user(FAKE_HUMAN_ID)),
        dev_agent: ok(AgentRef::parse(FAKE_AGENT_ID)),
    }
}

/// The fake task whose stream the canonical family's records land on.
#[must_use]
pub fn fake_task() -> TaskRef {
    ok(TaskRef::parse(FAKE_TASK_ID))
}

/// A second fake task (stream isolation).
#[must_use]
pub fn fake_other_task() -> TaskRef {
    ok(TaskRef::parse(FAKE_OTHER_TASK_ID))
}

/// The canonical fake artifact ids, in slot order (see
/// [`FAKE_ARTIFACT_ID_STRINGS`]).
#[must_use]
pub fn fake_artifact_ids() -> [ArtifactRef; 4] {
    [
        ok(ArtifactRef::parse(FAKE_ARTIFACT_ID_STRINGS[0])),
        ok(ArtifactRef::parse(FAKE_ARTIFACT_ID_STRINGS[1])),
        ok(ArtifactRef::parse(FAKE_ARTIFACT_ID_STRINGS[2])),
        ok(ArtifactRef::parse(FAKE_ARTIFACT_ID_STRINGS[3])),
    ]
}

/// The deterministic moment the canonical family decides at.
#[must_use]
pub fn now() -> Timestamp {
    ok(Timestamp::parse(FAKE_NOW))
}

/// A fake node name from the canonical family.
#[must_use]
pub fn fake_node(slot: usize) -> NodeName {
    ok(NodeName::parse(FAKE_NODE_NAMES[slot]))
}

/// The canonical approval gate (the work order's exact example): the
/// environment-setup step blocks with the named need — "approve the
/// environment switch" — with the consequence of each side stated.
pub fn env_switch_gate() -> Result<ApprovalGate, TakeoverError> {
    ApprovalGate::new(
        fake_task(),
        ok(NodeName::parse("environment-setup")),
        "approve the environment switch",
        "moving to the remote sandbox will re-run the setup steps",
        "this step stops with your decision recorded as the reason",
        fake_actors().dev_agent,
        now(),
    )
}

/// The canonical takeover record: Ana takes over the research step
/// from Dev's agent, with the agent's research notes preserved
/// verbatim (the projection law).
#[must_use]
pub fn ana_takes_over_research() -> TakeoverRecord {
    ok(TakeoverRecord::new(
        fake_task(),
        TakeoverScope::Node { node: fake_node(0) },
        fake_actors().dev_agent,
        fake_actors().ana,
        now(),
        "the research step needs a human eye",
        vec![ok(PreservedArtifact::new(
            fake_artifact_ids()[0].clone(),
            "research notes",
        ))],
    ))
}

/// The canonical propagation graph: the research → analysis → report
/// chain, research running with its notes already attributed, nothing
/// finished.
pub fn research_graph() -> Result<GraphShape, TakeoverError> {
    GraphShape::new(vec![
        GraphNode {
            node: fake_node(0),
            depends_on: Vec::new(),
            finished: false,
            artifacts: vec![fake_artifact_ids()[0].clone()],
        },
        GraphNode {
            node: fake_node(1),
            depends_on: vec![fake_node(0)],
            finished: false,
            artifacts: Vec::new(),
        },
        GraphNode {
            node: fake_node(2),
            depends_on: vec![fake_node(1)],
            finished: false,
            artifacts: Vec::new(),
        },
    ])
}

/// The finished-dependent propagation graph: analysis and report
/// finished (their waits had cleared through research's kept notes;
/// their own work is attributed) before the research step was
/// cancelled — the blocked-resolved family.
pub fn finished_dependent_graph() -> Result<GraphShape, TakeoverError> {
    GraphShape::new(vec![
        GraphNode {
            node: fake_node(0),
            depends_on: Vec::new(),
            finished: false,
            artifacts: vec![fake_artifact_ids()[0].clone()],
        },
        GraphNode {
            node: fake_node(1),
            depends_on: vec![fake_node(0)],
            finished: true,
            artifacts: vec![fake_artifact_ids()[1].clone()],
        },
        GraphNode {
            node: fake_node(2),
            depends_on: vec![fake_node(1)],
            finished: true,
            artifacts: vec![fake_artifact_ids()[2].clone()],
        },
    ])
}

fn node_cancellation(slot: usize, reason: &str) -> Result<CancellationRecord, TakeoverError> {
    CancellationRecord::new(
        fake_task(),
        CancelledWhat::Node {
            node: fake_node(slot),
        },
        reason,
        fake_actors().ana,
        now(),
    )
}

/// The leaf cancellation of the propagation matrix: the report step
/// (nothing waits on it).
pub fn cancel_leaf_node() -> Result<CancellationRecord, TakeoverError> {
    node_cancellation(2, "the report is not needed anymore")
}

/// The mid-node cancellation of the propagation matrix: the analysis
/// step (the report waits on it).
pub fn cancel_mid_run_node() -> Result<CancellationRecord, TakeoverError> {
    node_cancellation(1, "the analysis is going the wrong way")
}

/// The root cancellation of the propagation matrix: the research
/// step (analysis and report wait on it, transitively).
pub fn cancel_root_node() -> Result<CancellationRecord, TakeoverError> {
    node_cancellation(0, "the research direction changed")
}

/// The run cancellation of the propagation matrix: the whole run
/// (every non-finished node is named).
pub fn cancel_run() -> Result<CancellationRecord, TakeoverError> {
    CancellationRecord::new(
        fake_task(),
        CancelledWhat::Run,
        "stopping here — the plan changed",
        fake_actors().ana,
        now(),
    )
}

/// One case of the fake propagation space: a cancellation record and
/// the graph it propagates over.
#[derive(Debug, Clone)]
pub struct PropagationCase {
    /// The cancellation.
    pub record: CancellationRecord,
    /// The graph it propagates over.
    pub graph: GraphShape,
}

/// The **complete** fake propagation space: every cancellation kind
/// (leaf / mid / root / run) × every graph variant (the running
/// chain, the finished-dependent chain) — 4 × 2 = 8 cases, each
/// deterministic. Exhaustive deterministic enumeration — the
/// propagation matrix, the dependent-naming law and the
/// determinism law are proven over the whole space, not sampled.
#[must_use]
pub fn propagation_matrix() -> Vec<PropagationCase> {
    let graphs = [research_graph(), finished_dependent_graph()];
    let mut cases = Vec::new();
    for graph in graphs {
        let graph = ok(graph);
        for record in [
            cancel_leaf_node(),
            cancel_mid_run_node(),
            cancel_root_node(),
            cancel_run(),
        ] {
            let record = ok(record);
            cases.push(PropagationCase {
                record,
                graph: graph.clone(),
            });
        }
    }
    cases
}
