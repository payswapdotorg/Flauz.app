//! The cancellation record and the propagation projection (TAKE-001
//! §cancel) — cancellation honesty, made structural (addendum §4).
//!
//! A [`CancellationRecord`] names what was cancelled (one node, or
//! the run), the honest reason, the attributed actor and the moment.
//! Cancellation is a DISTINCT record family from failure: a node
//! that was cancelled is never recorded as merely failed, and the
//! distinction rides every wire form (the frozen `flauz-orch` law —
//! the six-state vocabulary gains no `canceled` variant; the
//! CANCELLATION RECORD distinguishes them).
//!
//! [`propagate_cancellation`] is the **propagation projection**: a
//! pure function over a [`GraphShape`] (the graph as plain data —
//! nodes, their dependency edges, their finished/attributed state)
//! plus a [`CancellationRecord`]. It names EVERY dependent's
//! terminal state — never a silently-stuck `Blocked`:
//!
//! - a dependent that can no longer run (a node it transitively
//!   waited on was cancelled) is **cancelled**, with the reason and
//!   the cancelled step it waited on named;
//! - a dependent whose wait had already cleared through the
//!   cancelled node's kept work, and whose own work finished before
//!   the cancellation, is **blocked-resolved** with the reason: its
//!   state stays terminal-success and its attributed work is kept;
//! - already-attributed work is KEPT everywhere — the cancelled
//!   node's own artifacts ride the projection's `kept_work` despite
//!   the cancellation;
//! - the run-level record stays at most ONE per sequence (the frozen
//!   `flauz-orch` law): a run cancellation produces exactly one
//!   record and names every non-finished node as a node-level
//!   dependent resolution.
//!
//! Determinism (kernel §7): the projection is a pure function of its
//! inputs — dependents are named in the graph's node order, the
//! transitive closure is computed to a bounded fixpoint, and
//! identical inputs produce byte-identical output.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::TakeoverError;
use crate::TakeoverVersion;
use crate::refs::{ActorRef, ArtifactRef, NodeName, TaskRef};
use crate::time::Timestamp;
use crate::{MAX_DEPENDENCIES, MAX_GRAPH_NODES, ensure_explanation, ensure_list_bound};

/// What was cancelled: one node, or the run as a whole.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum CancelledWhat {
    /// One node of the task's graph was cancelled.
    Node {
        /// The cancelled node.
        node: NodeName,
    },
    /// The run as a whole was cancelled.
    Run,
}

impl CancelledWhat {
    /// The node this cancellation names, when it is a node
    /// cancellation.
    #[must_use]
    pub const fn node(&self) -> Option<&NodeName> {
        match self {
            Self::Node { node } => Some(node),
            Self::Run => None,
        }
    }

    /// Whether this is the run-level cancellation (at most one per
    /// sequence — the frozen law).
    #[must_use]
    pub const fn is_run(&self) -> bool {
        matches!(self, Self::Run)
    }
}

/// The cancellation record: what was cancelled, the honest reason,
/// the attributed actor and the moment. A distinct family from
/// failure — cancellation honesty (addendum §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancellationRecord {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The task whose event stream this record lands on.
    pub task: TaskRef,
    /// What was cancelled (the node, or the run).
    pub what: CancelledWhat,
    /// The honest cancellation reason.
    pub reason: String,
    /// The actor who cancelled (a human deciding, or the system on
    /// shutdown — attributed either way).
    pub actor: ActorRef,
    /// When the cancellation happened.
    pub cancelled_at: Timestamp,
}

impl CancellationRecord {
    /// Builds a cancellation record.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the reason is empty or over the
    /// bound, or a reference fails validation.
    pub fn new(
        task: TaskRef,
        what: CancelledWhat,
        reason: &str,
        actor: ActorRef,
        cancelled_at: Timestamp,
    ) -> Result<Self, TakeoverError> {
        task.validate()?;
        if let Some(node) = what.node() {
            node.validate()?;
        }
        ensure_explanation("cancellation reason", reason)?;
        actor.validate()?;
        Ok(Self {
            v: TakeoverVersion,
            task,
            what,
            reason: reason.to_owned(),
            actor,
            cancelled_at,
        })
    }

    /// Validates the record against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), TakeoverError> {
        Self::new(
            self.task.clone(),
            self.what.clone(),
            &self.reason,
            self.actor.clone(),
            self.cancelled_at,
        )?;
        Ok(())
    }

    /// The task stream this record lands on.
    #[must_use]
    pub const fn stream_task(&self) -> &TaskRef {
        &self.task
    }
}

/// One node of the propagation graph, as plain data: its name, the
/// nodes it depends on, whether its work already finished
/// (terminal-success — its attributed work is kept), and the
/// artifacts already attributed to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNode {
    /// The node's name.
    pub node: NodeName,
    /// The nodes this node depends on (its dependency edges — a node
    /// runs only when every dependency's work it waits on has
    /// landed). Unique, bounded, every name known to the graph.
    pub depends_on: Vec<NodeName>,
    /// Whether the node's work already finished (terminal-success)
    /// when the cancellation landed — its attributed work is kept.
    pub finished: bool,
    /// The artifacts already attributed to this node (kept despite
    /// any cancellation — the honesty law).
    pub artifacts: Vec<ArtifactRef>,
}

/// The graph the propagation projects over — plain call-side data
/// (NOT the frozen `flauz-orch` graph; the caller snapshots whatever
/// it knows). Bounded, duplicate-free, with every dependency edge
/// pointing at a known node and no self-dependencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphShape {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The nodes, in the caller's declared order (the projection
    /// names dependents in this order — deterministic by input).
    pub nodes: Vec<GraphNode>,
}

impl GraphShape {
    /// Builds a graph shape, validating the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the node list is over the
    /// bound, a node name repeats, a dependency is unknown, a node
    /// depends on itself, or a dependency list is over the bound or
    /// contains duplicates.
    pub fn new(nodes: Vec<GraphNode>) -> Result<Self, TakeoverError> {
        ensure_list_bound("graph nodes", nodes.len(), MAX_GRAPH_NODES)?;
        let mut names = BTreeSet::new();
        for entry in &nodes {
            entry.node.validate()?;
            ensure_list_bound(
                "graph node dependencies",
                entry.depends_on.len(),
                MAX_DEPENDENCIES,
            )?;
            if !names.insert(entry.node.as_str()) {
                return Err(TakeoverError::invalid(format!(
                    "the graph contains the node {} more than once",
                    entry.node
                )));
            }
        }
        for entry in &nodes {
            let mut seen = BTreeSet::new();
            for dependency in &entry.depends_on {
                if dependency == &entry.node {
                    return Err(TakeoverError::invalid(format!(
                        "the node {} depends on itself",
                        entry.node
                    )));
                }
                if !names.contains(dependency.as_str()) {
                    return Err(TakeoverError::invalid(format!(
                        "the node {} depends on the unknown node {}",
                        entry.node, dependency
                    )));
                }
                if !seen.insert(dependency.as_str()) {
                    return Err(TakeoverError::invalid(format!(
                        "the node {} depends on {} more than once",
                        entry.node, dependency
                    )));
                }
            }
            let mut artifacts = BTreeSet::new();
            for artifact in &entry.artifacts {
                artifact.validate()?;
                if !artifacts.insert(artifact.as_str()) {
                    return Err(TakeoverError::invalid(format!(
                        "the node {} attributes {} more than once",
                        entry.node, artifact
                    )));
                }
            }
        }
        Ok(Self {
            v: TakeoverVersion,
            nodes,
        })
    }

    /// Validates the graph against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), TakeoverError> {
        Self::new(self.nodes.clone())?;
        Ok(())
    }
}

/// The terminal state one dependent lands in (addendum §4 — the two
/// named states; never a silently-stuck `Blocked`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum DependentTerminal {
    /// The dependent can no longer run: a node it (transitively)
    /// waited on was cancelled. Terminal, with the named reason.
    Cancelled {
        /// The honest reason (the original cancellation's reason —
        /// the truth the dependent is told).
        reason: String,
    },
    /// The dependent's wait had already cleared through the cancelled
    /// node's kept work and its own work finished before the
    /// cancellation: its state stays terminal-success, the resolution
    /// is named with the reason, and its attributed work is kept.
    BlockedResolved {
        /// The honest reason.
        reason: String,
    },
}

/// One dependent's named resolution in the propagation output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependentResolution {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The dependent node.
    pub node: NodeName,
    /// The cancelled node it (directly) waited on, when a NODE
    /// cancellation sealed its fate ("the research step it waited on
    /// was cancelled"); `None` for run-level dependents — the run
    /// itself is the cause, and the record's `what` carries it.
    pub waited_on: Option<NodeName>,
    /// The terminal state it lands in.
    pub terminal: DependentTerminal,
    /// The dependent's already-attributed work, kept (empty for a
    /// dependent that never produced work).
    pub kept: Vec<ArtifactRef>,
}

/// The propagation projection's output: the cancellation record plus
/// every dependent's named terminal state, in the graph's node
/// order, plus the already-attributed work kept despite the
/// cancellation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Propagation {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The cancellation this projection propagates (exactly one
    /// record; for a run cancellation, the one run-level record of
    /// the sequence — the frozen law).
    pub cancellation: CancellationRecord,
    /// Every dependent's named terminal state, in the graph's node
    /// order.
    pub dependents: Vec<DependentResolution>,
    /// The already-attributed work kept despite the cancellation:
    /// the cancelled node's own artifacts (a node cancellation), or
    /// every finished node's artifacts (a run cancellation).
    pub kept_work: Vec<ArtifactRef>,
}

impl Propagation {
    /// Whether this projection carries the run-level record (at most
    /// one per sequence — the frozen law; a node-level propagation
    /// carries none).
    #[must_use]
    pub const fn carries_run_record(&self) -> bool {
        self.cancellation.what.is_run()
    }

    /// Whether every dependent is named terminal with a NAMED reason
    /// — none left silently `Blocked` (the terminal kind is structural;
    /// the honesty check is the reason).
    #[must_use]
    pub fn every_dependent_terminal(&self) -> bool {
        self.dependents
            .iter()
            .all(|dependent| match &dependent.terminal {
                DependentTerminal::Cancelled { reason }
                | DependentTerminal::BlockedResolved { reason } => !reason.is_empty(),
            })
    }
}

/// The propagation projection (addendum §4): given the graph as data
/// and the cancellation record, name every dependent's terminal
/// state — never a silently-stuck `Blocked`.
///
/// - **A node cancellation**: the cancelled node must exist in the
///   graph. The blast radius is the transitive closure over
///   `depends_on`. Each dependent in radius, in graph order:
///   `cancelled` with the reason (it can no longer run) — unless its
///   work already finished, in which case `blocked-resolved` with
///   the reason and its attributed work kept. The cancelled node's
///   own artifacts are kept (`kept_work`) despite the cancellation.
/// - **A run cancellation**: every non-finished node is named
///   `cancelled` with the reason (it waited on the run); the
///   finished nodes' attributed work is kept (`kept_work`).
///
/// Deterministic: a pure function of its inputs; dependents in the
/// graph's node order; byte-identical output for identical inputs.
///
/// # Errors
///
/// Returns [`TakeoverError`] when the graph or the record fails
/// validation, or a node cancellation names a node the graph does
/// not know.
pub fn propagate_cancellation(
    record: &CancellationRecord,
    graph: &GraphShape,
) -> Result<Propagation, TakeoverError> {
    record.validate()?;
    graph.validate()?;
    let reason = record.reason.clone();
    match &record.what {
        CancelledWhat::Node { node } => {
            let cancelled_index = graph
                .nodes
                .iter()
                .position(|entry| &entry.node == node)
                .ok_or_else(|| {
                    TakeoverError::invalid(format!("the cancelled node {node} is not in the graph"))
                })?;
            // The blast radius: the transitive closure over depends_on
            // (a node is in the radius when it depends on the cancelled
            // node or on a node in the radius), computed to a bounded
            // fixpoint. The cancelled node itself is NOT a dependent.
            let mut radius: BTreeSet<usize> = BTreeSet::new();
            let mut changed = true;
            while changed {
                changed = false;
                for (index, entry) in graph.nodes.iter().enumerate() {
                    if index == cancelled_index || radius.contains(&index) {
                        continue;
                    }
                    let touches_radius = entry.depends_on.iter().any(|dependency| {
                        dependency == node
                            || graph
                                .nodes
                                .iter()
                                .position(|other| &other.node == dependency)
                                .is_some_and(|dependency_index| radius.contains(&dependency_index))
                    });
                    if touches_radius {
                        radius.insert(index);
                        changed = true;
                    }
                }
            }
            let mut dependents = Vec::new();
            for index in &radius {
                let entry = &graph.nodes[*index];
                // The cancelled step this dependent directly waited
                // on: the cancelled node, or the in-radius node it
                // depends on.
                let waited_on = entry
                    .depends_on
                    .iter()
                    .find(|dependency| {
                        *dependency == node
                            || graph
                                .nodes
                                .iter()
                                .position(|other| &other.node == *dependency)
                                .is_some_and(|dependency_index| radius.contains(&dependency_index))
                    })
                    .cloned()
                    .ok_or_else(|| {
                        TakeoverError::invalid(format!(
                            "the dependent {} does not name the cancelled path it waited on",
                            entry.node
                        ))
                    })?;
                let (terminal, kept) = if entry.finished {
                    (
                        DependentTerminal::BlockedResolved {
                            reason: reason.clone(),
                        },
                        entry.artifacts.clone(),
                    )
                } else {
                    (
                        DependentTerminal::Cancelled {
                            reason: reason.clone(),
                        },
                        Vec::new(),
                    )
                };
                dependents.push(DependentResolution {
                    v: TakeoverVersion,
                    node: entry.node.clone(),
                    waited_on: Some(waited_on),
                    terminal,
                    kept,
                });
            }
            Ok(Propagation {
                v: TakeoverVersion,
                cancellation: record.clone(),
                dependents,
                kept_work: graph.nodes[cancelled_index].artifacts.clone(),
            })
        }
        CancelledWhat::Run => {
            // The run cancellation: every non-finished node is named
            // cancelled (the run itself is the cause — `waited_on` is
            // `None`, the record's `what` carries the truth); the
            // finished nodes' attributed work is kept.
            let mut dependents = Vec::new();
            let mut kept_work = Vec::new();
            for entry in &graph.nodes {
                if entry.finished {
                    kept_work.extend(entry.artifacts.iter().cloned());
                    continue;
                }
                dependents.push(DependentResolution {
                    v: TakeoverVersion,
                    node: entry.node.clone(),
                    waited_on: None,
                    terminal: DependentTerminal::Cancelled {
                        reason: reason.clone(),
                    },
                    kept: Vec::new(),
                });
            }
            Ok(Propagation {
                v: TakeoverVersion,
                cancellation: record.clone(),
                dependents,
                kept_work,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{
        cancel_mid_run_node, fake_actors, finished_dependent_graph, now, research_graph,
    };

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn cancellation_records_are_distinct_from_failures() {
        let record = ok(cancel_mid_run_node());
        assert!(record.what.node().is_some());
        assert!(!record.what.is_run());
        let serialized = ok(serde_json::to_string(&record));
        assert!(serialized.contains("\"kind\":\"node\""));
        assert!(serialized.contains("\"reason\":\""));
        let reloaded: CancellationRecord = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, record);
        assert!(
            serde_json::from_str::<CancellationRecord>(
                &serialized.replace("\"reason\":", "\"why\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
        // An empty reason is refused — the truth is always named.
        assert!(
            CancellationRecord::new(
                crate::fakes::fake_task(),
                CancelledWhat::Run,
                "",
                fake_actors().ana,
                now(),
            )
            .is_err()
        );
    }

    #[test]
    fn the_graph_shape_is_validated_as_data() {
        assert!(ok(research_graph()).validate().is_ok());
        assert!(ok(finished_dependent_graph()).validate().is_ok());
        // Unknown dependency.
        let bad = GraphShape::new(vec![GraphNode {
            node: ok(NodeName::parse("report")),
            depends_on: vec![ok(NodeName::parse("ghost"))],
            finished: false,
            artifacts: Vec::new(),
        }]);
        assert!(bad.is_err());
        // Self-dependency.
        let bad = GraphShape::new(vec![GraphNode {
            node: ok(NodeName::parse("report")),
            depends_on: vec![ok(NodeName::parse("report"))],
            finished: false,
            artifacts: Vec::new(),
        }]);
        assert!(bad.is_err());
        // Duplicate nodes.
        let node = ok(NodeName::parse("report"));
        let bad = GraphShape::new(vec![
            GraphNode {
                node: node.clone(),
                depends_on: Vec::new(),
                finished: false,
                artifacts: Vec::new(),
            },
            GraphNode {
                node,
                depends_on: Vec::new(),
                finished: false,
                artifacts: Vec::new(),
            },
        ]);
        assert!(bad.is_err());
    }

    #[test]
    fn cancelling_a_node_names_every_dependent_in_the_graph_order() {
        let graph = ok(research_graph());
        let record = ok(crate::fakes::cancel_root_node());
        let propagation = ok(propagate_cancellation(&record, &graph));
        // The blast radius of cancelling research: analysis (waits on
        // research) and report (waits on analysis) — in graph order.
        assert_eq!(propagation.dependents.len(), 2);
        assert_eq!(propagation.dependents[0].node.as_str(), "analysis");
        assert_eq!(
            propagation.dependents[0]
                .waited_on
                .as_ref()
                .map(NodeName::as_str),
            Some("research")
        );
        assert!(matches!(
            propagation.dependents[0].terminal,
            DependentTerminal::Cancelled { .. }
        ));
        assert_eq!(propagation.dependents[1].node.as_str(), "report");
        assert_eq!(
            propagation.dependents[1]
                .waited_on
                .as_ref()
                .map(NodeName::as_str),
            Some("analysis")
        );
        assert!(matches!(
            propagation.dependents[1].terminal,
            DependentTerminal::Cancelled { .. }
        ));
        assert!(propagation.every_dependent_terminal());
        assert!(!propagation.carries_run_record());
        // The cancelled node's own already-attributed work is kept.
        assert_eq!(propagation.kept_work.len(), 1);
        assert_eq!(
            propagation.kept_work[0],
            crate::fakes::fake_artifact_ids()[0]
        );
        // A dependent is never named twice.
        let mut seen = BTreeSet::new();
        for dependent in &propagation.dependents {
            assert!(seen.insert(dependent.node.as_str()));
        }
    }

    #[test]
    fn a_finished_dependent_is_blocked_resolved_and_its_work_kept() {
        let graph = ok(finished_dependent_graph());
        let record = ok(crate::fakes::cancel_root_node());
        let propagation = ok(propagate_cancellation(&record, &graph));
        // In the finished-dependent graph, analysis and report
        // finished before the cancellation: both are blocked-resolved
        // with the reason, their work kept.
        for dependent in &propagation.dependents {
            match &dependent.terminal {
                DependentTerminal::BlockedResolved { reason } => {
                    assert_eq!(reason, &record.reason);
                }
                DependentTerminal::Cancelled { .. } => {
                    panic!(
                        "the finished dependent {} must be blocked-resolved, not cancelled",
                        dependent.node
                    );
                }
            }
            assert!(!dependent.kept.is_empty());
        }
    }

    #[test]
    fn a_run_cancellation_carries_exactly_one_run_record() {
        let graph = ok(research_graph());
        let record = ok(crate::fakes::cancel_run());
        let propagation = ok(propagate_cancellation(&record, &graph));
        assert!(propagation.carries_run_record());
        // Every non-finished node is named as a node-level dependent,
        // with the run (not a node) as the cause.
        assert_eq!(propagation.dependents.len(), 3);
        for dependent in &propagation.dependents {
            assert!(matches!(
                dependent.terminal,
                DependentTerminal::Cancelled { .. }
            ));
            assert!(
                dependent.waited_on.is_none(),
                "a run-level dependent waited on the run, not a node"
            );
        }
        // Exactly one run-level record rides the output: the embedded
        // record. Every dependent is node-level.
        let serialized = ok(serde_json::to_string(&propagation));
        assert_eq!(serialized.matches("\"what\":{\"kind\":\"run\"}").count(), 1);
        assert!(propagation.every_dependent_terminal());

        // The node-level propagation carries no run record at all.
        let node_propagation = ok(propagate_cancellation(
            &ok(crate::fakes::cancel_mid_run_node()),
            &graph,
        ));
        assert!(!node_propagation.carries_run_record());
        let serialized = ok(serde_json::to_string(&node_propagation));
        assert_eq!(serialized.matches("\"kind\":\"run\"").count(), 0);
    }

    #[test]
    fn a_leaf_cancellation_has_no_dependents() {
        let graph = ok(research_graph());
        let record = ok(crate::fakes::cancel_leaf_node());
        let propagation = ok(propagate_cancellation(&record, &graph));
        assert!(propagation.dependents.is_empty());
        assert!(propagation.kept_work.is_empty());
    }

    #[test]
    fn cancelling_an_unknown_node_is_an_error() {
        let graph = ok(research_graph());
        let record = ok(CancellationRecord::new(
            crate::fakes::fake_task(),
            CancelledWhat::Node {
                node: ok(NodeName::parse("ghost")),
            },
            "not needed anymore",
            fake_actors().ana,
            now(),
        ));
        assert!(propagate_cancellation(&record, &graph).is_err());
    }
}
