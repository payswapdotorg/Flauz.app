//! Resource leases and conflict policy (CONTEXT-HARNESS-ARCHITECTURE §7,
//! kernel §7).
//!
//! Parallel actors may read concurrently while writes require coordination.
//! Every [`ResourceLease`] is bounded and attributable: it has an absolute
//! `expires_at` deadline, an owner actor, a scope (resource ID + access
//! mode) and a conflict policy. Expiry is enforced on read: an expired lease
//! is not held. There is no implicit renewal.
//!
//! The default coordination pattern (read/analyze/propose parallel, write
//! coordinated, commit serialized) is carried by the access mode and the
//! conflict policy; enforcement belongs to the execution layers, not to the
//! contract.

use serde::{Deserialize, Serialize};

use crate::ids::{LeaseId, ResourceId};
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{ContractVersion, WorldError};

/// The access mode of a lease (kernel §7): `read | analyze | propose | write
/// | commit`.
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

/// The conflict policy of a lease: what happens when a conflicting lease
/// request arrives while this lease is held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    /// Conflicting requests fail immediately.
    Reject,
    /// Conflicting requests wait for the current lease to expire or be
    /// released.
    Queue,
    /// Conflicting requests require explicit resolution.
    Escalate,
}

/// A bounded, attributable lease on a resource scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLease {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical lease ID (`lease_<ULID>`).
    pub id: LeaseId,
    /// Durable entity version, starting at 1.
    pub version: u64,
    /// The resource this lease scopes.
    pub resource_id: ResourceId,
    /// The access mode of the lease.
    pub mode: AccessMode,
    /// The owner actor of the lease.
    pub owner: ActorRef,
    /// When the lease was granted (caller-supplied).
    pub granted_at: Timestamp,
    /// The absolute deadline: at and after this timestamp the lease is not
    /// held, without any implicit renewal.
    pub expires_at: Timestamp,
    /// The conflict policy for conflicting lease requests.
    pub conflict_policy: ConflictPolicy,
    /// When the lease was released, if it was released before its deadline.
    pub released_at: Option<Timestamp>,
}

impl ResourceLease {
    /// Builds a new lease at version 1. The absolute deadline must lie
    /// strictly after the grant time; a lease that is already expired when
    /// granted cannot exist.
    pub fn new(
        id: LeaseId,
        resource_id: ResourceId,
        mode: AccessMode,
        owner: ActorRef,
        granted_at: Timestamp,
        expires_at: Timestamp,
        conflict_policy: ConflictPolicy,
    ) -> Result<Self, WorldError> {
        if expires_at.is_before(&granted_at) || expires_at == granted_at {
            return Err(WorldError::invalid(
                "lease expires_at must be strictly after granted_at",
            ));
        }
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            resource_id,
            mode,
            owner,
            granted_at,
            expires_at,
            conflict_policy,
            released_at: None,
        })
    }

    /// Returns `true` when the lease is held at `now`: not released and
    /// strictly before the absolute deadline. This is the read-time
    /// enforcement point (kernel §7): an expired lease is not held.
    #[must_use]
    pub fn is_held_at(&self, now: &Timestamp) -> bool {
        self.released_at.is_none() && now.is_before(&self.expires_at)
    }

    /// Validates the lease.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(
            self.id.clone(),
            self.resource_id.clone(),
            self.mode,
            self.owner.clone(),
            self.granted_at,
            self.expires_at,
            self.conflict_policy,
        )?;
        if let Some(released_at) = self.released_at
            && released_at.is_before(&self.granted_at)
        {
            return Err(WorldError::invalid(
                "lease released_at must not be before granted_at",
            ));
        }
        if self.version == 0 {
            return Err(WorldError::invalid("lease version must be at least 1"));
        }
        Ok(())
    }
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
    fn access_modes_serialize_to_the_kernel_vocabulary() {
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
    fn conflict_policies_serialize_to_the_kernel_vocabulary() {
        for (policy, text) in [
            (ConflictPolicy::Reject, "\"reject\""),
            (ConflictPolicy::Queue, "\"queue\""),
            (ConflictPolicy::Escalate, "\"escalate\""),
        ] {
            assert_eq!(ok(serde_json::to_string(&policy)), text);
        }
    }

    #[test]
    fn lease_must_be_bounded_by_a_future_deadline() {
        let granted = ok(Timestamp::parse("2026-09-21T13:45:00Z"));
        let same = granted;
        let before = ok(Timestamp::parse("2026-09-21T13:44:59Z"));
        let actor = ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6"));
        for deadline in [same, before] {
            assert!(
                ResourceLease::new(
                    LeaseId::generate(),
                    ResourceId::generate(),
                    AccessMode::Write,
                    actor.clone(),
                    granted,
                    deadline,
                    ConflictPolicy::Queue
                )
                .is_err()
            );
        }
    }
}
