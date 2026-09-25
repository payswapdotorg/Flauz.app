//! The approval gate and its decisions (TAKE-001 §approval) — the
//! named need and the consequence of each side, made STRUCTURAL
//! (addendum §3).
//!
//! An [`ApprovalGate`] is the NAMED need that blocks a node: what
//! must be approved, the requesting node, the agent asking (on whose
//! authority the request rides), the moment, and the consequence of
//! EACH side — approving and denying. A gate without both
//! consequences cannot be constructed. An approval gate is DATA the
//! evaluator/caller consults, not a thread block: this crate never
//! blocks, sleeps or waits.
//!
//! An [`ApprovalDecision`] is the attributed human call: approve or
//! deny, the deciding human (kind `user`, validated — an agent
//! cannot decide a gate; `HumanApproval != AgentDecision`), the
//! timestamp, and the attributed [`DecisionEffect`]:
//!
//! - approving proceeds — the node resumes;
//! - denying FAILS the node honestly, with the gate's named denial
//!   consequence as the failure reason — never a silent proceed
//!   ([`ApprovalDecision::deny`] builds exactly that effect, and no
//!   other denial effect exists).

use serde::{Deserialize, Serialize};

use crate::TakeoverError;
use crate::TakeoverVersion;
use crate::refs::{ActorRef, AgentRef, NodeName, TaskRef};
use crate::time::Timestamp;
use crate::ensure_explanation;

/// The prefix of an honest denial's failure reason (the named
/// consequence the node fails with — the UI and the conformance tests
/// pin the shape: the reason is the gate's own denial consequence,
/// never a bare "denied").
pub const DENIAL_FAILURE_PREFIX: &str = "Declined by the human: ";

/// The approval gate: the named need that blocks a node (addendum
/// §3). What must be approved (`need`), the requesting node, the
/// agent asking, the moment, and the consequence of each side. A
/// gate without full attribution and both consequences cannot be
/// constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalGate {
    /// Contract schema version (`"v": 1`).
    pub v: TakeoverVersion,
    /// The task whose event stream the gate's events land on.
    pub task: TaskRef,
    /// The node blocked on this gate (the requesting node).
    pub node: NodeName,
    /// The named need, in the user's words ("approve the environment
    /// switch").
    pub need: String,
    /// The consequence of approving ("moving to the remote sandbox
    /// will re-run the setup steps").
    pub approve_consequence: String,
    /// The consequence of declining (the node fails honestly with
    /// this as its reason).
    pub deny_consequence: String,
    /// The agent requesting the approval (on whose authority the
    /// request rides).
    pub requested_by: AgentRef,
    /// When the gate was raised.
    pub requested_at: Timestamp,
}

impl ApprovalGate {
    /// Builds an approval gate, enforcing the named-need law
    /// structurally: the need and BOTH consequences are required,
    /// bounded and honest.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the need or a consequence is
    /// empty or over the bound, or a reference fails validation.
    pub fn new(
        task: TaskRef,
        node: NodeName,
        need: &str,
        approve_consequence: &str,
        deny_consequence: &str,
        requested_by: AgentRef,
        requested_at: Timestamp,
    ) -> Result<Self, TakeoverError> {
        task.validate()?;
        node.validate()?;
        requested_by.validate()?;
        ensure_explanation("approval need", need)?;
        ensure_explanation("approve consequence", approve_consequence)?;
        ensure_explanation("deny consequence", deny_consequence)?;
        Ok(Self {
            v: TakeoverVersion,
            task,
            node,
            need: need.to_owned(),
            approve_consequence: approve_consequence.to_owned(),
            deny_consequence: deny_consequence.to_owned(),
            requested_by,
            requested_at,
        })
    }

    /// Validates the gate against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), TakeoverError> {
        Self::new(
            self.task.clone(),
            self.node.clone(),
            &self.need,
            &self.approve_consequence,
            &self.deny_consequence,
            self.requested_by.clone(),
            self.requested_at,
        )?;
        Ok(())
    }

    /// The task stream this gate's events land on.
    #[must_use]
    pub const fn stream_task(&self) -> &TaskRef {
        &self.task
    }
}

/// The attributed effect of a decision: the node proceeds, or the
/// node fails honestly with the named reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum DecisionEffect {
    /// The gate cleared: the node resumes its work.
    NodeProceeds {
        /// The node that resumes.
        node: NodeName,
    },
    /// The node fails honestly, with the named reason (a denial's
    /// consequence — never a silent proceed).
    NodeFailed {
        /// The node that failed.
        node: NodeName,
        /// The honest failure reason (the gate's denial consequence).
        reason: String,
    },
}

/// The human's decision on one approval gate (addendum §3): approve
/// or deny, the deciding human, the timestamp, and the attributed
/// effect. Both variants embed the gate in full — a decision without
/// the named need it decides cannot be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum ApprovalDecision {
    /// The human approved: the node proceeds.
    Approved {
        /// Contract schema version (`"v": 1`).
        v: TakeoverVersion,
        /// The gate that was approved, in full.
        gate: ApprovalGate,
        /// The deciding human (kind `user`, validated).
        decided_by: ActorRef,
        /// When the decision was made.
        decided_at: Timestamp,
        /// The attributed effect (the node proceeds).
        effect: DecisionEffect,
    },
    /// The human declined: the node fails honestly with the denial
    /// as its reason — never a silent proceed.
    Denied {
        /// Contract schema version (`"v": 1`).
        v: TakeoverVersion,
        /// The gate that was declined, in full.
        gate: ApprovalGate,
        /// The deciding human (kind `user`, validated).
        decided_by: ActorRef,
        /// When the decision was made.
        decided_at: Timestamp,
        /// The attributed effect (the node fails with the named
        /// consequence).
        effect: DecisionEffect,
    },
}

impl ApprovalDecision {
    /// Builds an APPROVED decision: the deciding human must be a
    /// `user` principal; the effect is the node proceeding.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the gate fails validation or
    /// the deciding actor is not a human.
    pub fn approve(
        gate: ApprovalGate,
        decided_by: ActorRef,
        decided_at: Timestamp,
    ) -> Result<Self, TakeoverError> {
        gate.validate()?;
        decided_by.validate()?;
        if !decided_by.is_human() {
            return Err(TakeoverError::invalid(
                "an approval is a HUMAN decision: the deciding actor must have kind `user`",
            ));
        }
        let node = gate.node.clone();
        Ok(Self::Approved {
            v: TakeoverVersion,
            gate,
            decided_by,
            decided_at,
            effect: DecisionEffect::NodeProceeds { node },
        })
    }

    /// Builds a DENIED decision: the deciding human must be a `user`
    /// principal; the effect is the node failing honestly with the
    /// gate's named denial consequence as its reason — never a
    /// silent proceed.
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when the gate fails validation or
    /// the deciding actor is not a human.
    pub fn deny(
        gate: ApprovalGate,
        decided_by: ActorRef,
        decided_at: Timestamp,
    ) -> Result<Self, TakeoverError> {
        gate.validate()?;
        decided_by.validate()?;
        if !decided_by.is_human() {
            return Err(TakeoverError::invalid(
                "a denial is a HUMAN decision: the deciding actor must have kind `user`",
            ));
        }
        let node = gate.node.clone();
        let reason = format!("{DENIAL_FAILURE_PREFIX}{}", gate.deny_consequence);
        Ok(Self::Denied {
            v: TakeoverVersion,
            gate,
            decided_by,
            decided_at,
            effect: DecisionEffect::NodeFailed { node, reason },
        })
    }

    /// The gate every decision carries (the named need it decides).
    #[must_use]
    pub const fn gate(&self) -> &ApprovalGate {
        match self {
            Self::Approved { gate, .. } | Self::Denied { gate, .. } => gate,
        }
    }

    /// The deciding human every decision carries.
    #[must_use]
    pub const fn decided_by(&self) -> &ActorRef {
        match self {
            Self::Approved { decided_by, .. } | Self::Denied { decided_by, .. } => decided_by,
        }
    }

    /// Whether this is an approval.
    #[must_use]
    pub const fn is_approved(&self) -> bool {
        matches!(self, Self::Approved { .. })
    }

    /// Validates the decision against the canonical rules (the
    /// constructors' laws, re-checkable on read — including that a
    /// denial's effect is the honest failure, not a proceed).
    ///
    /// # Errors
    ///
    /// Returns [`TakeoverError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), TakeoverError> {
        match self {
            Self::Approved {
                gate,
                decided_by,
                decided_at,
                effect,
                ..
            } => {
                let rebuilt = Self::approve(gate.clone(), decided_by.clone(), *decided_at)?;
                if effect != rebuilt.effect() {
                    return Err(TakeoverError::invalid(
                        "an approved decision's effect must be the node proceeding",
                    ));
                }
            }
            Self::Denied {
                gate,
                decided_by,
                decided_at,
                effect,
                ..
            } => {
                let rebuilt = Self::deny(gate.clone(), decided_by.clone(), *decided_at)?;
                if effect != rebuilt.effect() {
                    return Err(TakeoverError::invalid(
                        "a denied decision's effect must be the honest failure with the named \
                         consequence — never a silent proceed",
                    ));
                }
            }
        }
        Ok(())
    }

    const fn effect(&self) -> &DecisionEffect {
        match self {
            Self::Approved { effect, .. } | Self::Denied { effect, .. } => effect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{env_switch_gate, fake_actors, now};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn a_gate_requires_the_need_and_both_consequences() {
        assert!(env_switch_gate().is_ok());
        let gate = ok(env_switch_gate());
        let task = gate.task.clone();
        let node = gate.node.clone();
        let requested_by = gate.requested_by.clone();
        // A gate without a consequence cannot be constructed.
        assert!(
            ApprovalGate::new(
                task.clone(),
                node.clone(),
                "approve the environment switch",
                "",
                "the node fails honestly",
                requested_by.clone(),
                now(),
            )
            .is_err(),
            "an empty approve consequence is refused"
        );
        assert!(
            ApprovalGate::new(
                task,
                node,
                "approve the environment switch",
                "moving to the remote sandbox will re-run the setup steps",
                "",
                requested_by,
                now(),
            )
            .is_err(),
            "an empty deny consequence is refused"
        );
    }

    #[test]
    fn only_a_human_decides() {
        let actors = fake_actors();
        let gate = ok(env_switch_gate());
        assert!(
            ApprovalDecision::approve(gate.clone(), actors.ana_as_agent(), now()).is_err(),
            "an agent cannot approve"
        );
        assert!(
            ApprovalDecision::deny(gate.clone(), actors.ana_as_agent(), now()).is_err(),
            "an agent cannot deny"
        );
        assert!(ApprovalDecision::approve(gate.clone(), actors.ana.clone(), now()).is_ok());
        assert!(ApprovalDecision::deny(gate.clone(), actors.ana, now()).is_ok());
    }

    #[test]
    fn denial_carries_the_named_consequence_and_never_silently_proceeds() {
        let gate = ok(env_switch_gate());
        let denied = ok(ApprovalDecision::deny(
            gate.clone(),
            ok(ActorRef::user("user_ana")),
            now(),
        ));
        assert!(!denied.is_approved());
        match denied.effect() {
            DecisionEffect::NodeFailed { node, reason } => {
                assert_eq!(*node, gate.node);
                assert_eq!(
                    reason,
                    &format!("{DENIAL_FAILURE_PREFIX}{}", gate.deny_consequence)
                );
                assert_eq!(
                    reason,
                    "Declined by the human: this step stops with your decision recorded as the \
                     reason"
                );
            }
            DecisionEffect::NodeProceeds { .. } => {
                panic!("a denial must never silently proceed");
            }
        }
        assert!(denied.validate().is_ok());
        // A hand-tampered denial that proceeds is refused on read.
        let mut tampered = denied.clone();
        if let ApprovalDecision::Denied { effect, .. } = &mut tampered {
            *effect = DecisionEffect::NodeProceeds {
                node: gate.node.clone(),
            };
        }
        assert!(
            tampered.validate().is_err(),
            "a denial whose effect proceeds is dishonest and must be refused"
        );
    }

    #[test]
    fn approval_proceeds_and_decisions_round_trip_canonically() {
        let gate = ok(env_switch_gate());
        let approved = ok(ApprovalDecision::approve(
            gate.clone(),
            ok(ActorRef::user("user_ana")),
            now(),
        ));
        assert!(approved.is_approved());
        assert_eq!(approved.gate(), &gate);
        match approved.effect() {
            DecisionEffect::NodeProceeds { node } => assert_eq!(*node, gate.node),
            DecisionEffect::NodeFailed { .. } => panic!("an approval must proceed"),
        }
        assert!(approved.validate().is_ok());

        let serialized = ok(serde_json::to_string(&approved));
        let reloaded: ApprovalDecision = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, approved);
        assert!(serialized.contains("\"kind\":\"approved\""));
        assert!(serialized.contains("\"kind\":\"node_proceeds\""));
        let denied = ok(ApprovalDecision::deny(
            gate,
            ok(ActorRef::user("user_ana")),
            now(),
        ));
        let serialized = ok(serde_json::to_string(&denied));
        assert!(serialized.contains("\"kind\":\"denied\""));
        assert!(serialized.contains("\"kind\":\"node_failed\""));
        let reloaded: ApprovalDecision = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, denied);
        assert!(
            serde_json::from_str::<ApprovalDecision>(&serialized.replace("\"need\":", "\"ask\":"))
                .is_err(),
            "unknown fields are rejected"
        );
    }
}
