//! The world-stream event vocabulary (LEASE-001): the `lease.*` event
//! types as **data** — canonical payload shapes the CALLER records
//! through the existing world-store seam (the frozen
//! `<entity>.<verb_past>` grammar; addendum §1).
//!
//! This crate records nothing itself: the world store keeps owning
//! the durable lease entities and their streams. `lease.granted` and
//! `lease.released` are already registered by `flauz-world`; this
//! vocabulary completes the family the manager's outcomes need:
//!
//! | event type | recorded when | payload |
//! |---|---|---|
//! | `lease.requested` | a request arrives for a decision | [`RequestedPayload`] |
//! | `lease.granted` | a request is granted (fresh) | [`GrantedPayload`] |
//! | `lease.rejected` | a request is rejected (named holder) | [`RejectedPayload`] |
//! | `lease.queued` | a request joins a wait queue | [`QueuedPayload`] |
//! | `lease.escalated` | a conflict needs the human | [`EscalatedPayload`] |
//! | `lease.released` | a lease is released (explicit / by decision) | [`ReleasedPayload`] |
//! | `lease.expired` | a lease expires by its own deadline | [`ExpiredPayload`] |
//! | `lease.renewed` | an explicit renewal is granted | [`RenewedPayload`] |
//!
//! Every payload carries full attribution — the conflict-honesty law
//! rides the wire: no event without the named cause.

use serde::{Deserialize, Serialize};

use crate::LeaseError;
use crate::LeaseVersion;
use crate::lease::LeaseRecord;
use crate::manager::NamedConflict;
use crate::manager::{ExpiredLease, ReleasedLease};
use crate::mode::ConflictPolicy;
use crate::refs::{ActorRef, LeaseRef, ResourceRef};
use crate::request::{LeaseDecision, LeaseRequest, QueuedAhead, RenewalAttribution};
use crate::time::Timestamp;

/// The registered event-type vocabulary of this crate (kernel §5).
pub mod event_types {
    /// A lease request arrived for a decision.
    pub const LEASE_REQUESTED: &str = "lease.requested";
    /// A lease was granted (registered by `flauz-world`).
    pub const LEASE_GRANTED: &str = "lease.granted";
    /// A lease request was rejected, naming the holder.
    pub const LEASE_REJECTED: &str = "lease.rejected";
    /// A lease request joined a wait queue.
    pub const LEASE_QUEUED: &str = "lease.queued";
    /// A lease conflict needs the human's decision.
    pub const LEASE_ESCALATED: &str = "lease.escalated";
    /// A lease was released (registered by `flauz-world`).
    pub const LEASE_RELEASED: &str = "lease.released";
    /// A lease expired by its own deadline.
    pub const LEASE_EXPIRED: &str = "lease.expired";
    /// A lease was extended by an explicit renewal.
    pub const LEASE_RENEWED: &str = "lease.renewed";
}

/// One event kind of the lease vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// A lease request arrived for a decision.
    Requested,
    /// A lease was granted.
    Granted,
    /// A lease request was rejected, naming the holder.
    Rejected,
    /// A lease request joined a wait queue.
    Queued,
    /// A lease conflict needs the human's decision.
    Escalated,
    /// A lease was released.
    Released,
    /// A lease expired by its own deadline.
    Expired,
    /// A lease was extended by an explicit renewal.
    Renewed,
}

impl EventKind {
    /// Every event kind, in vocabulary order.
    pub const ALL: [Self; 8] = [
        Self::Requested,
        Self::Granted,
        Self::Rejected,
        Self::Queued,
        Self::Escalated,
        Self::Released,
        Self::Expired,
        Self::Renewed,
    ];

    /// The frozen `<entity>.<verb_past>` event-type string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requested => event_types::LEASE_REQUESTED,
            Self::Granted => event_types::LEASE_GRANTED,
            Self::Rejected => event_types::LEASE_REJECTED,
            Self::Queued => event_types::LEASE_QUEUED,
            Self::Escalated => event_types::LEASE_ESCALATED,
            Self::Released => event_types::LEASE_RELEASED,
            Self::Expired => event_types::LEASE_EXPIRED,
            Self::Renewed => event_types::LEASE_RENEWED,
        }
    }

    /// The event kind a decision records on the world stream: a fresh
    /// grant records `lease.granted`; an explicit renewal records
    /// `lease.renewed`; a rejection, queueing or escalation records
    /// its own kind. The caller records `lease.requested` on arrival,
    /// and `lease.released` / `lease.expired` from the sweep.
    #[must_use]
    pub fn for_decision(decision: &LeaseDecision) -> Self {
        match decision {
            LeaseDecision::Granted { renewal, .. } => {
                if renewal.is_some() {
                    Self::Renewed
                } else {
                    Self::Granted
                }
            }
            LeaseDecision::Rejected { .. } => Self::Rejected,
            LeaseDecision::Queued { .. } => Self::Queued,
            LeaseDecision::Escalated { .. } => Self::Escalated,
        }
    }
}

/// The payload of a `lease.requested` event: the request, in full.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The request that arrived.
    pub request: LeaseRequest,
}

impl RequestedPayload {
    /// Builds the payload from a request.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the request fails validation.
    pub fn new(request: LeaseRequest) -> Result<Self, LeaseError> {
        request.validate()?;
        Ok(Self {
            v: LeaseVersion,
            request,
        })
    }
}

/// The payload of a `lease.granted` event: the granted lease record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The granted lease record.
    pub lease: LeaseRecord,
}

impl GrantedPayload {
    /// Builds the payload from a granted lease record.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the record fails validation.
    pub fn new(lease: LeaseRecord) -> Result<Self, LeaseError> {
        lease.validate()?;
        Ok(Self {
            v: LeaseVersion,
            lease,
        })
    }

    /// Builds the payload from a granted decision (`None` for any
    /// other decision kind — a renewal records
    /// [`RenewedPayload`] instead).
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the record fails validation.
    pub fn from_decision(decision: &LeaseDecision) -> Result<Option<Self>, LeaseError> {
        match decision {
            LeaseDecision::Granted { lease, renewal, .. } if renewal.is_none() => {
                Ok(Some(Self::new(lease.clone())?))
            }
            _ => Ok(None),
        }
    }
}

/// The payload of a `lease.rejected` event: the rejected request, the
/// named holder, and the policy that rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The rejected request.
    pub request: LeaseRequest,
    /// The holder whose lease blocked it, named in full.
    pub holder: LeaseRecord,
    /// The holder-side policy that decided the rejection.
    pub policy: ConflictPolicy,
}

impl RejectedPayload {
    /// Builds the payload from a rejected decision (`None` for any
    /// other decision kind).
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the decision's records fail
    /// validation.
    pub fn from_decision(decision: &LeaseDecision) -> Result<Option<Self>, LeaseError> {
        match decision {
            LeaseDecision::Rejected {
                request,
                holder,
                policy,
                ..
            } => {
                request.validate()?;
                holder.validate()?;
                Ok(Some(Self {
                    v: LeaseVersion,
                    request: request.clone(),
                    holder: holder.clone(),
                    policy: *policy,
                }))
            }
            _ => Ok(None),
        }
    }
}

/// The payload of a `lease.queued` event: the queued request, the
/// honest position, the named waiters ahead, and the projected grant
/// moment given the holders' deadlines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueuedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The queued request.
    pub request: LeaseRequest,
    /// The 1-based position in the resource's wait queue.
    pub position: u32,
    /// The named waiters ahead, in request order.
    pub ahead: Vec<QueuedAhead>,
    /// The earliest moment the resource can free up, given the
    /// conflicting holders' deadlines.
    pub projected_grant_at: Timestamp,
}

impl QueuedPayload {
    /// Builds the payload from a queued decision (`None` for any other
    /// decision kind).
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the decision's records fail
    /// validation.
    pub fn from_decision(decision: &LeaseDecision) -> Result<Option<Self>, LeaseError> {
        match decision {
            LeaseDecision::Queued {
                request,
                position,
                ahead,
                projected_grant_at,
                ..
            } => {
                request.validate()?;
                Ok(Some(Self {
                    v: LeaseVersion,
                    request: request.clone(),
                    position: *position,
                    ahead: ahead.clone(),
                    projected_grant_at: *projected_grant_at,
                }))
            }
            _ => Ok(None),
        }
    }
}

/// The payload of a `lease.escalated` event: the named conflict the
/// human must decide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EscalatedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The named conflict (holder in full, contending request in
    /// full).
    pub conflict: NamedConflict,
}

impl EscalatedPayload {
    /// Builds the payload from an escalated decision (`None` for any
    /// other decision kind).
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the decision's records fail
    /// validation.
    pub fn from_decision(decision: &LeaseDecision) -> Result<Option<Self>, LeaseError> {
        match decision {
            LeaseDecision::Escalated {
                request, holder, ..
            } => {
                request.validate()?;
                holder.validate()?;
                Ok(Some(Self {
                    v: LeaseVersion,
                    conflict: NamedConflict {
                        v: LeaseVersion,
                        request: request.clone(),
                        holder: holder.clone(),
                    },
                }))
            }
            _ => Ok(None),
        }
    }
}

/// The payload of a `lease.released` event: the released lease, named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The released lease's canonical id.
    pub lease_id: LeaseRef,
    /// The resource the lease scoped.
    pub resource_id: ResourceRef,
    /// The holder who held it.
    pub holder: ActorRef,
    /// When it was released.
    pub released_at: Timestamp,
}

impl ReleasedPayload {
    /// Builds the payload from a sweep's released lease.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the record fails validation.
    pub fn new(released: &ReleasedLease) -> Result<Self, LeaseError> {
        released.lease.validate()?;
        Ok(Self {
            v: LeaseVersion,
            lease_id: released.lease.id.clone(),
            resource_id: released.lease.resource_id.clone(),
            holder: released.lease.owner.clone(),
            released_at: released.lease.released_at.ok_or_else(|| {
                LeaseError::invalid("a released payload needs the release moment")
            })?,
        })
    }
}

/// The payload of a `lease.expired` event: the expired lease, named,
/// with the moment it stopped being held (its own deadline).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpiredPayload {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The expired lease's canonical id.
    pub lease_id: LeaseRef,
    /// The resource the lease scoped.
    pub resource_id: ResourceRef,
    /// The holder who held it.
    pub holder: ActorRef,
    /// The moment it stopped being held (its own deadline).
    pub expired_at: Timestamp,
}

impl ExpiredPayload {
    /// Builds the payload from a sweep's expired lease.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the record fails validation.
    pub fn new(expired: &ExpiredLease) -> Result<Self, LeaseError> {
        expired.lease.validate()?;
        Ok(Self {
            v: LeaseVersion,
            lease_id: expired.lease.id.clone(),
            resource_id: expired.lease.resource_id.clone(),
            holder: expired.lease.owner.clone(),
            expired_at: expired.expired_at,
        })
    }
}

/// The payload of a `lease.renewed` event: the renewed lease record
/// and the deadline it used to have (the explicit renewal's honest
/// attribution).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenewedPayload {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The renewed lease record (same id, version + 1).
    pub lease: LeaseRecord,
    /// What the renewal extended (prior version and deadline).
    pub renewal: RenewalAttribution,
}

impl RenewedPayload {
    /// Builds the payload from a granted renewal decision (`None` for
    /// any other decision kind).
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the decision's records fail
    /// validation.
    pub fn from_decision(decision: &LeaseDecision) -> Result<Option<Self>, LeaseError> {
        match decision {
            LeaseDecision::Granted { lease, renewal, .. } => {
                Ok(renewal.as_ref().map(|attribution| Self {
                    v: LeaseVersion,
                    lease: lease.clone(),
                    renewal: attribution.clone(),
                }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{ana_write_lease, dev_write_request, now};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn the_vocabulary_is_the_frozen_dotted_grammar() {
        for (kind, text) in [
            (EventKind::Requested, "lease.requested"),
            (EventKind::Granted, "lease.granted"),
            (EventKind::Rejected, "lease.rejected"),
            (EventKind::Queued, "lease.queued"),
            (EventKind::Escalated, "lease.escalated"),
            (EventKind::Released, "lease.released"),
            (EventKind::Expired, "lease.expired"),
            (EventKind::Renewed, "lease.renewed"),
        ] {
            assert_eq!(kind.as_str(), text);
        }
        assert_eq!(EventKind::ALL.len(), 8);
        // The two types flauz-world already registers keep their exact
        // frozen strings.
        assert_eq!(event_types::LEASE_GRANTED, "lease.granted");
        assert_eq!(event_types::LEASE_RELEASED, "lease.released");
    }

    #[test]
    fn decisions_map_to_their_event_kinds() {
        let request = ok(dev_write_request());
        let holder = ana_write_lease(crate::mode::ConflictPolicy::Queue);
        let queued = ok(crate::manager::decide(
            &request,
            std::slice::from_ref(&holder),
            &[],
            now(),
        ));
        assert_eq!(EventKind::for_decision(&queued), EventKind::Queued);

        let rejected_holder = ana_write_lease(crate::mode::ConflictPolicy::Reject);
        let rejected = ok(crate::manager::decide(
            &request,
            &[rejected_holder],
            &[],
            now(),
        ));
        assert_eq!(EventKind::for_decision(&rejected), EventKind::Rejected);

        let escalated_holder = ana_write_lease(crate::mode::ConflictPolicy::Escalate);
        let escalated = ok(crate::manager::decide(
            &request,
            &[escalated_holder],
            &[],
            now(),
        ));
        assert_eq!(EventKind::for_decision(&escalated), EventKind::Escalated);

        let free = ok(crate::manager::decide(&request, &[], &[], now()));
        assert_eq!(EventKind::for_decision(&free), EventKind::Granted);

        let renewal = ok(crate::request::LeaseRequest::renewal(
            holder.id.clone(),
            holder.resource_id.clone(),
            holder.mode,
            holder.owner.clone(),
            now(),
            ok(Timestamp::parse("2026-09-23T16:30:00Z")),
            crate::mode::ConflictPolicy::Queue,
        ));
        let renewed = ok(crate::manager::decide(&renewal, &[holder], &[], now()));
        assert_eq!(EventKind::for_decision(&renewed), EventKind::Renewed);
    }

    #[test]
    fn payloads_round_trip_canonically_and_carry_attribution() {
        let request = ok(dev_write_request());
        let requested = ok(RequestedPayload::new(request.clone()));
        let serialized = ok(serde_json::to_string(&requested));
        let reloaded: RequestedPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, requested);
        assert!(serialized.contains("\"lease_id\":\"lease_"));

        let holder = ana_write_lease(crate::mode::ConflictPolicy::Reject);
        let decision = ok(crate::manager::decide(&request, &[holder], &[], now()));
        let rejected = match ok(RejectedPayload::from_decision(&decision)) {
            Some(payload) => payload,
            None => panic!("a rejected decision yields the payload"),
        };
        let serialized = ok(serde_json::to_string(&rejected));
        let reloaded: RejectedPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, rejected);
        assert!(
            serialized.contains("\"holder\":{"),
            "the rejection names its holder on the wire"
        );
        assert!(ok(GrantedPayload::from_decision(&decision)).is_none());

        let queued_holder = ana_write_lease(crate::mode::ConflictPolicy::Queue);
        let queued_decision = ok(crate::manager::decide(
            &request,
            &[queued_holder],
            &[],
            now(),
        ));
        let queued = match ok(QueuedPayload::from_decision(&queued_decision)) {
            Some(payload) => payload,
            None => panic!("a queued decision yields the payload"),
        };
        let serialized = ok(serde_json::to_string(&queued));
        let reloaded: QueuedPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, queued);
        assert!(serialized.contains("\"projected_grant_at\":\"2026-09-23T15:40:00Z\""));

        let escalated_holder = ana_write_lease(crate::mode::ConflictPolicy::Escalate);
        let escalated_decision = ok(crate::manager::decide(
            &request,
            &[escalated_holder],
            &[],
            now(),
        ));
        let escalated = match ok(EscalatedPayload::from_decision(&escalated_decision)) {
            Some(payload) => payload,
            None => panic!("an escalated decision yields the payload"),
        };
        let serialized = ok(serde_json::to_string(&escalated));
        assert!(serialized.contains("\"conflict\":{"));

        // Unknown fields are rejected on every payload family.
        let requested_serialized = ok(serde_json::to_string(&requested));
        assert!(
            serde_json::from_str::<RequestedPayload>(
                &requested_serialized.replace("\"request\":", "\"ask\":")
            )
            .is_err()
        );
    }

    #[test]
    fn sweep_payloads_carry_the_named_lease() {
        let holder = ana_write_lease(crate::mode::ConflictPolicy::Queue);
        let sweep = ok(crate::manager::sweep(
            &[holder],
            &[],
            ok(Timestamp::parse("2026-09-23T15:41:00Z")),
        ));
        let expired = ok(ExpiredPayload::new(&sweep.expired[0]));
        let serialized = ok(serde_json::to_string(&expired));
        let reloaded: ExpiredPayload = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, expired);
        assert_eq!(
            expired.expired_at,
            ok(Timestamp::parse("2026-09-23T15:40:00Z"))
        );
        assert!(sweep.released.is_empty());
    }
}
