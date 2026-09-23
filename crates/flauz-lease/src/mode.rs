//! Access modes and conflict policies — the frozen `flauz-world`
//! vocabulary, carried as local data (the flauz-cap law: the enums
//! serialize to the exact frozen wire forms, so a mode serialized by
//! either crate has byte-identical meaning).
//!
//! The frozen contract (CONTEXT-HARNESS-ARCHITECTURE §7, kernel §7):
//! parallel actors may read, analyze and propose concurrently while
//! writes require coordination and commits are serialized or explicitly
//! resolved. This module fixes the DETERMINISTIC conflict matrix over
//! that vocabulary:
//!
//! - the **parallel (shared) family** is `read | analyze | propose` —
//!   two shared modes never conflict;
//! - the **coordinated (exclusive) family** is `write | commit` — an
//!   exclusive mode conflicts with EVERY other mode, including another
//!   exclusive mode.
//!
//! The conflict policy of a held lease decides what happens when a
//! conflicting request arrives (the frozen law): reject (fail
//! immediately, named), queue (wait for expiry or release, FIFO), or
//! escalate (the human decides — never auto-resolved).

use serde::{Deserialize, Serialize};

/// The access mode of a lease (kernel §7): `read | analyze | propose |
/// write | commit`. The wire vocabulary is owned by `flauz-world`;
/// this local copy serializes identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessMode {
    /// Parallel reads.
    Read,
    /// Parallel analysis.
    Analyze,
    /// Parallel proposals.
    Propose,
    /// Coordinated writes.
    Write,
    /// Serialized or explicitly resolved commits.
    Commit,
}

impl AccessMode {
    /// Every access mode, in the frozen vocabulary order.
    pub const ALL: [Self; 5] = [
        Self::Read,
        Self::Analyze,
        Self::Propose,
        Self::Write,
        Self::Commit,
    ];

    /// Whether the mode is in the coordinated (exclusive) family:
    /// `write | commit`. An exclusive mode conflicts with every other
    /// mode; the parallel family (`read | analyze | propose`) never
    /// conflicts with itself.
    #[must_use]
    pub const fn is_exclusive(self) -> bool {
        matches!(self, Self::Write | Self::Commit)
    }

    /// The deterministic conflict matrix: `true` when a holder in
    /// `self` and a requester in `other` cannot hold the same resource
    /// at the same moment.
    ///
    /// - exclusive × exclusive → conflict (coordination is pairwise);
    /// - exclusive × shared → conflict (a write/commit needs the
    ///   resource clear);
    /// - shared × shared → no conflict (parallel reads, analyses and
    ///   proposals coexist).
    #[must_use]
    pub const fn conflicts_with(self, other: AccessMode) -> bool {
        self.is_exclusive() || other.is_exclusive()
    }
}

/// The conflict policy of a lease: what happens when a conflicting
/// request arrives while the lease is held. The wire vocabulary is
/// owned by `flauz-world`; this local copy serializes identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    /// Conflicting requests fail immediately (with the named holder).
    Reject,
    /// Conflicting requests wait for the current lease to expire or be
    /// released (FIFO, with honest positions).
    Queue,
    /// Conflicting requests require explicit human resolution (never
    /// auto-resolved).
    Escalate,
}

impl ConflictPolicy {
    /// Every conflict policy, in the frozen vocabulary order.
    pub const ALL: [Self; 3] = [Self::Reject, Self::Queue, Self::Escalate];
}

/// The decision a human makes on an escalated conflict (addendum §2:
/// escalation produces a named "needs the human" record with the
/// decision the human must make — never an auto-resolution). Carried
/// as data through [`crate::manager::resolve_escalation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationChoice {
    /// Release the current holder and grant the waiting requester.
    GrantToWaiter,
    /// Keep the current holder; the waiting requester is told no.
    KeepHolder,
}

/// The outcome of an explicit, attributed escalation decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum EscalationOutcome {
    /// The human granted the waiting requester: the holder's lease is
    /// released (versioned) and the waiter's lease is granted — each
    /// record named, the decision attributed.
    WaiterGranted {
        /// Contract schema version (`"v": 1`).
        v: crate::LeaseVersion,
        /// The holder's lease, released by this decision (version +1,
        /// `released_at` set).
        released_holder: crate::lease::LeaseRecord,
        /// The waiter's newly granted lease.
        granted: crate::lease::LeaseRecord,
        /// The human who made the call.
        decided_by: crate::refs::ActorRef,
        /// When the decision was applied (caller-supplied).
        decided_at: crate::time::Timestamp,
    },
    /// The human kept the current holder: the waiter is NOT granted —
    /// the outcome says so with the named waiter (the consequence is
    /// structural; the surfaces state it in user words).
    HolderKept {
        /// Contract schema version (`"v": 1`).
        v: crate::LeaseVersion,
        /// The holder's lease, unchanged.
        holder: crate::lease::LeaseRecord,
        /// The waiting requester who is told no.
        waiter: crate::refs::ActorRef,
        /// The human who made the call.
        decided_by: crate::refs::ActorRef,
        /// When the decision was applied (caller-supplied).
        decided_at: crate::time::Timestamp,
    },
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
    fn modes_serialize_to_the_kernel_vocabulary() {
        for (mode, text) in [
            (AccessMode::Read, "\"read\""),
            (AccessMode::Analyze, "\"analyze\""),
            (AccessMode::Propose, "\"propose\""),
            (AccessMode::Write, "\"write\""),
            (AccessMode::Commit, "\"commit\""),
        ] {
            assert_eq!(ok(serde_json::to_string(&mode)), text);
        }
        assert!(serde_json::from_str::<AccessMode>("\"admin\"").is_err());
    }

    #[test]
    fn policies_serialize_to_the_kernel_vocabulary() {
        for (policy, text) in [
            (ConflictPolicy::Reject, "\"reject\""),
            (ConflictPolicy::Queue, "\"queue\""),
            (ConflictPolicy::Escalate, "\"escalate\""),
        ] {
            assert_eq!(ok(serde_json::to_string(&policy)), text);
        }
        assert!(serde_json::from_str::<ConflictPolicy>("\"maybe\"").is_err());
    }

    #[test]
    fn the_full_conflict_matrix_is_the_coordinated_family_law() {
        // exclusive × exclusive → conflict; exclusive × shared →
        // conflict; shared × shared → never a conflict.
        for holder in AccessMode::ALL {
            for requester in AccessMode::ALL {
                let expected = holder.is_exclusive() || requester.is_exclusive();
                assert_eq!(
                    holder.conflicts_with(requester),
                    expected,
                    "{holder:?} × {requester:?} must be {}",
                    if expected { "a conflict" } else { "compatible" }
                );
            }
        }
        assert!(AccessMode::Write.conflicts_with(AccessMode::Write));
        assert!(AccessMode::Commit.conflicts_with(AccessMode::Read));
        assert!(AccessMode::Read.conflicts_with(AccessMode::Commit));
        assert!(!AccessMode::Read.conflicts_with(AccessMode::Analyze));
        assert!(!AccessMode::Analyze.conflicts_with(AccessMode::Propose));
        assert!(!AccessMode::Propose.conflicts_with(AccessMode::Read));
    }

    #[test]
    fn escalation_choices_serialize_snake_case() {
        assert_eq!(
            ok(serde_json::to_string(&EscalationChoice::GrantToWaiter)),
            "\"grant_to_waiter\""
        );
        assert_eq!(
            ok(serde_json::to_string(&EscalationChoice::KeepHolder)),
            "\"keep_holder\""
        );
    }
}
