//! The lease request and the decision it earns (LEASE-001 §request) —
//! the conflict-honesty law made STRUCTURAL (addendum §2).
//!
//! A [`LeaseRequest`] is the resource ref + access mode + requester
//! actor + requested window + policy, taken as data. A renewal is a
//! request that NAMES the lease it extends (`renewal_of`): never
//! implicit, never inferred.
//!
//! A [`LeaseDecision`] is one of exactly four outcomes, and every
//! variant carries **the full request plus its named cause** — the
//! holder record that blocked it, the queue facts (position, who is
//! ahead, the projected grant moment), or the conflict pair the human
//! must decide. A decision without the named cause cannot be
//! constructed: there is no variant that says "queued" without naming
//! who is ahead, none that says "rejected" without the holder, none
//! that escalates without both sides.

use serde::{Deserialize, Serialize};

use crate::LeaseError;
use crate::LeaseVersion;
use crate::lease::LeaseRecord;
use crate::mode::{AccessMode, ConflictPolicy};
use crate::refs::{ActorRef, LeaseRef, ResourceRef};
use crate::time::Timestamp;

/// A lease request: the resource, the mode, the requester, the
/// requested window and the policy, as data. The `lease_id` is
/// caller-minted (the world store allocates canonical ids; this crate
/// generates none — kernel §7) and becomes the granted record's id.
/// For a renewal the `lease_id` IS the extended lease's id, and
/// `renewal_of` names it explicitly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseRequest {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The canonical id the granted lease carries — caller-minted; for
    /// a renewal this is the extended lease's id.
    pub lease_id: LeaseRef,
    /// The resource this request scopes.
    pub resource_id: ResourceRef,
    /// The access mode requested.
    pub mode: AccessMode,
    /// The requesting actor (named on every decision).
    pub requester: ActorRef,
    /// The window's start (the grant moment is never before it).
    pub requested_from: Timestamp,
    /// The window's absolute end: at and after it the request can no
    /// longer be granted.
    pub requested_until: Timestamp,
    /// The conflict policy the granted lease will carry.
    pub policy: ConflictPolicy,
    /// The lease this request explicitly extends, when it is a
    /// renewal. A renewal is a NEW request that names the lease it
    /// extends — never implicit, never inferred.
    pub renewal_of: Option<LeaseRef>,
}

impl LeaseRequest {
    /// Builds a fresh (non-renewal) request, validating the window and
    /// references.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the window is not strictly ordered
    /// or a reference fails validation.
    pub fn new(
        lease_id: LeaseRef,
        resource_id: ResourceRef,
        mode: AccessMode,
        requester: ActorRef,
        requested_from: Timestamp,
        requested_until: Timestamp,
        policy: ConflictPolicy,
    ) -> Result<Self, LeaseError> {
        let request = Self {
            v: LeaseVersion,
            lease_id,
            resource_id,
            mode,
            requester,
            requested_from,
            requested_until,
            policy,
            renewal_of: None,
        };
        request.validate()?;
        Ok(request)
    }

    /// Builds an explicit renewal request: a new request that names
    /// the lease it extends. The extended lease's id must equal the
    /// carried `lease_id` (the renewed record is that lease's next
    /// version).
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the window is not strictly ordered
    /// or a reference fails validation.
    pub fn renewal(
        lease_id: LeaseRef,
        resource_id: ResourceRef,
        mode: AccessMode,
        requester: ActorRef,
        requested_from: Timestamp,
        requested_until: Timestamp,
        policy: ConflictPolicy,
    ) -> Result<Self, LeaseError> {
        let request = Self {
            v: LeaseVersion,
            renewal_of: Some(lease_id.clone()),
            lease_id,
            resource_id,
            mode,
            requester,
            requested_from,
            requested_until,
            policy,
        };
        request.validate()?;
        Ok(request)
    }

    /// Whether this request is an explicit renewal.
    #[must_use]
    pub const fn is_renewal(&self) -> bool {
        self.renewal_of.is_some()
    }

    /// Validates the request against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), LeaseError> {
        self.lease_id.validate()?;
        self.resource_id.validate()?;
        self.requester.validate()?;
        if self
            .renewal_of
            .as_ref()
            .is_some_and(|extended| *extended != self.lease_id)
        {
            return Err(LeaseError::invalid(
                "a renewal request must name the lease it extends: renewal_of must equal lease_id",
            ));
        }
        if self.requested_until.is_before(&self.requested_from)
            || self.requested_until == self.requested_from
        {
            return Err(LeaseError::invalid(
                "request window must be strictly ordered: requested_until after requested_from",
            ));
        }
        Ok(())
    }
}

/// What a renewal names about the lease it extended (the honest
/// attribution carried by a granted renewal).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenewalAttribution {
    /// The version the extended lease had before the renewal.
    pub prior_version: u64,
    /// The deadline the extended lease had before the renewal.
    pub prior_expires_at: Timestamp,
}

/// One named waiter ahead in the queue (the honest position's
/// evidence).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueuedAhead {
    /// The waiting actor's attribution.
    pub requester: ActorRef,
    /// The mode they asked for.
    pub mode: AccessMode,
    /// When they were enqueued (their place in the request order).
    pub enqueued_at: Timestamp,
}

/// The decision one request earns. Every variant carries the full
/// request plus the NAMED cause — the conflict-honesty law is
/// structural: a decision without the named cause cannot be
/// constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum LeaseDecision {
    /// The request is granted: the lease record as data. When the
    /// grant is a renewal, the attribution names the lease it extended
    /// (prior version and deadline).
    Granted {
        /// Contract schema version (`"v": 1`).
        v: LeaseVersion,
        /// The request that earned this grant.
        request: LeaseRequest,
        /// The granted lease record.
        lease: LeaseRecord,
        /// What the renewal extended, when this grant is a renewal.
        renewal: Option<RenewalAttribution>,
    },
    /// The request is rejected: the NAMED holder and the policy that
    /// rejected it. Failing fast is honest only with the full picture.
    Rejected {
        /// Contract schema version (`"v": 1`).
        v: LeaseVersion,
        /// The request that was rejected.
        request: LeaseRequest,
        /// The holder whose lease blocks this request, named in full.
        holder: LeaseRecord,
        /// The holder-side policy that decided the rejection.
        policy: ConflictPolicy,
    },
    /// The request is queued: the NAMED queue — position, who is
    /// ahead, and the projected grant moment given the holders'
    /// deadlines. FIFO, deterministic, never flattering.
    Queued {
        /// Contract schema version (`"v": 1`).
        v: LeaseVersion,
        /// The request that was queued.
        request: LeaseRequest,
        /// The 1-based position in the resource's wait queue.
        position: u32,
        /// The named waiters ahead, in request order.
        ahead: Vec<QueuedAhead>,
        /// The earliest moment the resource can free up, given the
        /// conflicting holders' deadlines (an honest projection, never
        /// a promise).
        projected_grant_at: Timestamp,
    },
    /// The request escalates: the NAMED conflict — the holder, the
    /// requester, their modes — and the decision the human must make.
    /// Never auto-resolved (addendum §2).
    Escalated {
        /// Contract schema version (`"v": 1`).
        v: LeaseVersion,
        /// The request that escalated.
        request: LeaseRequest,
        /// The holder whose lease blocks this request, named in full.
        holder: LeaseRecord,
    },
}

impl LeaseDecision {
    /// The request every decision carries (the named requester's own
    /// ask — the structural attribution).
    #[must_use]
    pub fn request(&self) -> &LeaseRequest {
        match self {
            Self::Granted { request, .. }
            | Self::Rejected { request, .. }
            | Self::Queued { request, .. }
            | Self::Escalated { request, .. } => request,
        }
    }

    /// The named cause every decision carries (the conflict-honesty
    /// law's one-line summary for tests and the event wiring): who
    /// holds, who waits ahead, or the conflict pair.
    #[must_use]
    pub fn named_cause(&self) -> NamedCause {
        match self {
            Self::Granted { .. } => NamedCause::NoConflict,
            Self::Rejected { holder, .. } => NamedCause::Holder(holder.clone()),
            Self::Queued { ahead, .. } => NamedCause::Queue(ahead.clone()),
            Self::Escalated {
                holder, request, ..
            } => NamedCause::Conflict(Box::new(crate::manager::NamedConflict {
                v: LeaseVersion,
                request: request.clone(),
                holder: holder.clone(),
            })),
        }
    }
}

/// The named cause of a decision (the honest summary shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamedCause {
    /// No conflicting holder existed at the decision moment.
    NoConflict,
    /// This holder blocked the request.
    Holder(LeaseRecord),
    /// These waiters are ahead in the queue.
    Queue(Vec<QueuedAhead>),
    /// This conflict pair needs the human.
    Conflict(Box<crate::manager::NamedConflict>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{dev_write_request, fake_actors, fake_lease_ids, fake_resource};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn requests_require_a_strictly_ordered_window() {
        let from = ok(Timestamp::parse("2026-09-23T15:00:00Z"));
        let until = ok(Timestamp::parse("2026-09-23T15:40:00Z"));
        assert!(
            LeaseRequest::new(
                fake_lease_ids()[0].clone(),
                fake_resource(),
                AccessMode::Write,
                fake_actors().dev,
                until,
                from,
                ConflictPolicy::Queue,
            )
            .is_err(),
            "a backwards window is rejected"
        );
        assert!(
            LeaseRequest::new(
                fake_lease_ids()[0].clone(),
                fake_resource(),
                AccessMode::Write,
                fake_actors().dev,
                from,
                from,
                ConflictPolicy::Queue,
            )
            .is_err(),
            "an empty window is rejected"
        );
        assert!(dev_write_request().is_ok());
    }

    #[test]
    fn renewals_name_the_lease_they_extend() {
        let request = ok(LeaseRequest::renewal(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana,
            ok(Timestamp::parse("2026-09-23T15:20:00Z")),
            ok(Timestamp::parse("2026-09-23T16:00:00Z")),
            ConflictPolicy::Queue,
        ));
        assert!(request.is_renewal());
        assert_eq!(request.renewal_of, Some(fake_lease_ids()[0].clone()));
        let fresh = ok(dev_write_request());
        assert!(!fresh.is_renewal());
        assert_eq!(fresh.renewal_of, None);

        // A renewal whose renewal_of names a DIFFERENT lease is
        // invalid — the extension is explicit or it is nothing (the
        // wire carries the mismatch; validation refuses it).
        let mut mismatched = ok(LeaseRequest::new(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana,
            ok(Timestamp::parse("2026-09-23T15:20:00Z")),
            ok(Timestamp::parse("2026-09-23T16:00:00Z")),
            ConflictPolicy::Queue,
        ));
        mismatched.renewal_of = Some(fake_lease_ids()[1].clone());
        assert!(mismatched.validate().is_err());
    }

    #[test]
    fn requests_round_trip_canonically() {
        let request = ok(dev_write_request());
        let serialized = ok(serde_json::to_string(&request));
        let reloaded: LeaseRequest = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, request);
        assert!(serialized.contains("\"renewal_of\":null"));
        assert!(
            serde_json::from_str::<LeaseRequest>(
                &serialized.replace("\"requested_from\"", "\"from\"")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }
}
