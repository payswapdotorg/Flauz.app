//! The takeover and handback records (TAKE-001 §takeover) — the
//! takeover law made STRUCTURAL (addendum §3).
//!
//! A [`TakeoverRecord`] is the explicit, attributed handoff: WHO took
//! over (a human actor — anything else is refused at construction),
//! from WHAT (the agent whose turn it was, plus the node/run scope),
//! WHEN, and WHY. It also carries the agent's prior work as
//! [`PreservedArtifact`] rows — the **projection law** as data: the
//! agent's artifacts are preserved verbatim, and every consumer of
//! the record sees the same list.
//!
//! A [`HandbackRecord`] is the explicit return: what the human did,
//! when, and whether the agent resumes or the run completes. The
//! handback can only be constructed **against its own takeover**
//! ([`HandbackRecord::for_takeover`]): it embeds the takeover record
//! in full (the same attribution pair), and the handback moment must
//! lie strictly after the takeover moment. The agent resumes with the
//! SAME preserved work — [`HandbackRecord::preserved`] hands back
//! exactly what the takeover preserved, verbatim.
//!
//! The **same-stream law** (addendum §3): both records carry the
//! task reference — the human's turn is a TASK event on that task's
//! stream (recorded by the caller through the [`crate::events`]
//! vocabulary), never a second store. This crate holds no store of
//! any kind.

use serde::{Deserialize, Serialize};

use crate::TakeoverError;
use crate::TakeoverVersion;
use crate::refs::{ActorRef, AgentRef, ArtifactRef, NodeName, TaskRef};
use crate::time::Timestamp;
use crate::{MAX_PRESERVED_ARTIFACTS, ensure_explanation, ensure_list_bound, ensure_name};

/// What the human took over: one node of the task's graph, or the
/// run as a whole.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum TakeoverScope {
    /// The human took over one node's turn.
    Node {
        /// The node whose turn the human took over.
        node: NodeName,
    },
    /// The human took over the run as a whole.
    Run,
}

impl TakeoverScope {
    /// The node this scope names, when it is a node scope.
    #[must_use]
    pub const fn node(&self) -> Option<&NodeName> {
        match self {
            Self::Node { node } => Some(node),
            Self::Run => None,
        }
    }
}

/// One piece of the agent's prior work, preserved verbatim by a
/// takeover (the projection law as data): the artifact reference and
/// the label it was produced under. The work stays attributed to the
/// agent — a takeover never rewrites history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreservedArtifact {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The preserved artifact's canonical reference.
    pub artifact: ArtifactRef,
    /// The artifact's label (the same label the graph's artifact waits
    /// carry).
    pub label: String,
}

impl PreservedArtifact {
    /// Builds one preserved-artifact row, validating the label.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the label is empty or over the
    /// name bound.
    pub fn new(artifact: ArtifactRef, label: &str) -> Result<Self, TakeoverError> {
        artifact.validate()?;
        ensure_name("preserved artifact label", label)?;
        Ok(Self {
            v: TakeoverVersion,
            artifact,
            label: label.to_owned(),
        })
    }
}

/// The explicit, attributed handoff of the agent's turn to a human
/// (addendum §3 — the takeover law). A record without full
/// attribution cannot be constructed:
///
/// - `human` must be a `user` actor (the WHO — an agent cannot take
///   over from itself; `HumanApproval != AgentDecision`);
/// - `from_agent` names the agent whose turn it was (the `agent_<ULID>`
///   grammar, validated);
/// - `task` anchors the record to the task whose stream it lands on
///   (the same-stream law);
/// - `taken_at` is the moment (caller-supplied — never a wall clock);
/// - `reason` is the honest why (bounded, non-empty);
/// - `preserved` is the agent's prior work, kept verbatim (bounded,
///   duplicate-free).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TakeoverRecord {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The task whose event stream this record lands on (the
    /// same-stream law).
    pub task: TaskRef,
    /// What was taken over (the node, or the run).
    pub scope: TakeoverScope,
    /// The agent whose turn the human took over.
    pub from_agent: AgentRef,
    /// The human who took over (kind `user`, validated).
    pub human: ActorRef,
    /// When the takeover happened.
    pub taken_at: Timestamp,
    /// The honest reason for the takeover.
    pub reason: String,
    /// The agent's prior work, preserved verbatim (the projection
    /// law).
    pub preserved: Vec<PreservedArtifact>,
}

impl TakeoverRecord {
    /// Builds a takeover record, enforcing the takeover law
    /// structurally.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the human actor is not a `user`
    /// principal, a reference fails validation, the reason is empty
    /// or over the bound, or the preserved list is over the bound or
    /// contains duplicate artifacts.
    pub fn new(
        task: TaskRef,
        scope: TakeoverScope,
        from_agent: AgentRef,
        human: ActorRef,
        taken_at: Timestamp,
        reason: &str,
        preserved: Vec<PreservedArtifact>,
    ) -> Result<Self, TakeoverError> {
        task.validate()?;
        if let Some(node) = scope.node() {
            node.validate()?;
        }
        from_agent.validate()?;
        human.validate()?;
        if !human.is_human() {
            return Err(TakeoverError::invalid(
                "a takeover is a HUMAN handoff: the human actor must have kind `user`",
            ));
        }
        ensure_explanation("takeover reason", reason)?;
        ensure_list_bound(
            "preserved artifacts",
            preserved.len(),
            MAX_PRESERVED_ARTIFACTS,
        )?;
        let mut seen = std::collections::BTreeSet::new();
        for artifact in &preserved {
            artifact.artifact.validate()?;
            if !seen.insert(artifact.artifact.as_str()) {
                return Err(TakeoverError::invalid(format!(
                    "preserved artifacts contain {} more than once",
                    artifact.artifact
                )));
            }
        }
        Ok(Self {
            v: TakeoverVersion,
            task,
            scope,
            from_agent,
            human,
            taken_at,
            reason: reason.to_owned(),
            preserved,
        })
    }

    /// Validates the record against the canonical rules (the
    /// constructor's law, re-checkable on read).
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), TakeoverError> {
        Self::new(
            self.task.clone(),
            self.scope.clone(),
            self.from_agent.clone(),
            self.human.clone(),
            self.taken_at,
            &self.reason,
            self.preserved.clone(),
        )?;
        Ok(())
    }

    /// The task stream this record lands on (the same-stream law's
    /// one-line accessor: every takeover record names its stream).
    #[must_use]
    pub const fn stream_task(&self) -> &TaskRef {
        &self.task
    }
}

/// Whether the agent resumes or the run completes when the human
/// hands the turn back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandbackOutcome {
    /// The agent resumes the turn from where the human left it.
    AgentResumes,
    /// The run completes with the human's work as its outcome.
    RunCompletes,
}

/// The explicit, attributed return of the turn to the agent (addendum
/// §3). Constructed only against its own takeover: the takeover
/// record is embedded in full, and the handback moment lies strictly
/// after the takeover moment. What the human did is named; whether
/// the agent resumes or the run completes is named; and the agent
/// resumes with exactly the preserved work ([`Self::preserved`] —
/// the projection law's handback side).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandbackRecord {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The takeover being returned from, in full (the same
    /// attribution pair — task, scope, agent, human, moment, reason,
    /// preserved work).
    pub takeover: TakeoverRecord,
    /// When the human handed the turn back (strictly after
    /// `takeover.taken_at`).
    pub handed_back_at: Timestamp,
    /// The honest summary of what the human did.
    pub human_work: String,
    /// Whether the agent resumes or the run completes.
    pub outcome: HandbackOutcome,
}

impl HandbackRecord {
    /// Builds the handback for one takeover — the only construction
    /// path, so a handback without its takeover cannot exist.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the takeover fails validation,
    /// the handback moment is not strictly after the takeover moment,
    /// or the human-work summary is empty or over the bound.
    pub fn for_takeover(
        takeover: TakeoverRecord,
        handed_back_at: Timestamp,
        human_work: &str,
        outcome: HandbackOutcome,
    ) -> Result<Self, TakeoverError> {
        takeover.validate()?;
        if !handed_back_at.is_after(&takeover.taken_at) {
            return Err(TakeoverError::invalid(
                "a handback must happen strictly after its takeover",
            ));
        }
        ensure_explanation("handback human work", human_work)?;
        Ok(Self {
            v: TakeoverVersion,
            takeover,
            handed_back_at,
            human_work: human_work.to_owned(),
            outcome,
        })
    }

    /// Validates the record against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), TakeoverError> {
        Self::for_takeover(
            self.takeover.clone(),
            self.handed_back_at,
            &self.human_work,
            self.outcome,
        )?;
        Ok(())
    }

    /// The agent's prior work the agent resumes with — EXACTLY what
    /// the takeover preserved, verbatim (the projection law's
    /// handback side: nothing is dropped, nothing is rewritten).
    #[must_use]
    pub fn preserved(&self) -> &[PreservedArtifact] {
        &self.takeover.preserved
    }

    /// The agent the turn returns to.
    #[must_use]
    pub const fn agent(&self) -> &AgentRef {
        &self.takeover.from_agent
    }

    /// The human who handed the turn back (the same human who took
    /// it over).
    #[must_use]
    pub const fn human(&self) -> &ActorRef {
        &self.takeover.human
    }

    /// The task stream this record lands on (the same-stream law).
    #[must_use]
    pub const fn stream_task(&self) -> &TaskRef {
        &self.takeover.task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{ana_takes_over_research, fake_actors, fake_artifact_ids, now};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn a_takeover_requires_a_human_actor() {
        let actors = fake_actors();
        let scope = TakeoverScope::Node {
            node: ok(NodeName::parse("research")),
        };
        let preserved = vec![ok(PreservedArtifact::new(
            fake_artifact_ids()[0].clone(),
            "research notes",
        ))];
        // The human takeover constructs.
        assert!(
            TakeoverRecord::new(
                crate::fakes::fake_task(),
                scope.clone(),
                actors.dev_agent.clone(),
                actors.ana.clone(),
                now(),
                "the setup step needs a human eye",
                preserved.clone(),
            )
            .is_ok()
        );
        // An agent cannot take over from itself (HumanApproval !=
        // AgentDecision).
        assert!(
            TakeoverRecord::new(
                crate::fakes::fake_task(),
                scope,
                actors.dev_agent.clone(),
                actors.ana_as_agent(),
                now(),
                "the setup step needs a human eye",
                preserved,
            )
            .is_err(),
            "a non-user actor must be refused"
        );
    }

    #[test]
    fn preserved_artifacts_are_bounded_and_duplicate_free() {
        let actors = fake_actors();
        let duplicate = vec![
            ok(PreservedArtifact::new(
                fake_artifact_ids()[0].clone(),
                "research notes",
            )),
            ok(PreservedArtifact::new(
                fake_artifact_ids()[0].clone(),
                "the same artifact again",
            )),
        ];
        assert!(
            TakeoverRecord::new(
                crate::fakes::fake_task(),
                TakeoverScope::Run,
                actors.dev_agent,
                actors.ana,
                now(),
                "I will drive this run",
                duplicate,
            )
            .is_err(),
            "duplicate preserved artifacts must be refused"
        );
        assert!(PreservedArtifact::new(fake_artifact_ids()[0].clone(), "").is_err());
        let empty: Vec<PreservedArtifact> = Vec::new();
        assert!(
            TakeoverRecord::new(
                crate::fakes::fake_task(),
                TakeoverScope::Run,
                fake_actors().dev_agent,
                fake_actors().ana,
                now(),
                "I will drive this run",
                empty,
            )
            .is_ok(),
            "a takeover with no prior work is honest (nothing to preserve)"
        );
    }

    #[test]
    fn a_handback_names_its_takeover_and_preserves_its_work() {
        let takeover = ana_takes_over_research();
        let later = ok(Timestamp::parse("2026-09-23T15:26:00Z"));
        let handback = ok(HandbackRecord::for_takeover(
            takeover.clone(),
            later,
            "reviewed the sandbox credentials and approved the switch",
            HandbackOutcome::AgentResumes,
        ));
        // The projection law: the handback hands back EXACTLY the
        // preserved work.
        assert_eq!(handback.preserved(), takeover.preserved.as_slice());
        assert_eq!(handback.agent(), &takeover.from_agent);
        assert_eq!(handback.human(), &takeover.human);
        // The same-stream law: both records name the same task stream.
        assert_eq!(handback.stream_task(), takeover.stream_task());

        // A handback at or before the takeover moment is refused.
        assert!(
            HandbackRecord::for_takeover(
                takeover.clone(),
                takeover.taken_at,
                "too early",
                HandbackOutcome::AgentResumes
            )
            .is_err()
        );
        assert!(
            HandbackRecord::for_takeover(takeover, later, "", HandbackOutcome::RunCompletes)
                .is_err(),
            "an empty human-work summary is refused"
        );
    }

    #[test]
    fn takeover_records_round_trip_canonically() {
        let takeover = ana_takes_over_research();
        let serialized = ok(serde_json::to_string(&takeover));
        let reloaded: TakeoverRecord = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, takeover);
        assert!(serialized.contains("\"kind\":\"node\""));
        assert!(serialized.contains("\"human\":{"));
        assert!(serialized.contains("\"from_agent\":\"agent_"));
        assert!(
            serde_json::from_str::<TakeoverRecord>(&serialized.replace("\"taken_at\"", "\"at\""))
                .is_err(),
            "unknown fields are rejected"
        );
        let run_scope_serialized = ok(serde_json::to_string(&TakeoverScope::Run));
        assert_eq!(run_scope_serialized, "{\"kind\":\"run\"}");
        let node_scope_serialized = ok(serde_json::to_string(&TakeoverScope::Node {
            node: ok(NodeName::parse("research")),
        }));
        assert_eq!(
            node_scope_serialized,
            "{\"kind\":\"node\",\"node\":\"research\"}"
        );
    }
}
