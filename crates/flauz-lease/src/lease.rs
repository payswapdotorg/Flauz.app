//! The lease record as data — the frozen `flauz-world` `ResourceLease`
//! shape, mirrored field-for-field with local types (addendum §5: the
//! manager takes lease snapshots as data; the world store keeps owning
//! the durable lease entities).
//!
//! Field names match the frozen wire format exactly (`v`, `id`,
//! `version`, `resource_id`, `mode`, `owner`, `granted_at`,
//! `expires_at`, `conflict_policy`, `released_at`), so a record
//! serialized by either crate has byte-identical meaning on the wire.
//!
//! The frozen laws this record carries:
//!
//! - **bounded and attributable** — an absolute `expires_at` deadline,
//!   an owner actor, a scope (resource + access mode), a conflict
//!   policy;
//! - **expiry enforced on read** — [`LeaseRecord::is_held_at`] is the
//!   enforcement point: an expired or released lease is not held;
//! - **no implicit renewal** — a lease is extended only through an
//!   explicit renewal request that names it (see
//!   [`crate::request::LeaseRequest`]); [`LeaseRecord::renewed`] is the
//!   versioned record that results;
//! - **version discipline** — version 1 at creation, +1 per durable
//!   mutation (renewal, release).

use serde::{Deserialize, Serialize};

use crate::LeaseError;
use crate::LeaseVersion;
use crate::mode::{AccessMode, ConflictPolicy};
use crate::refs::{ActorRef, LeaseRef, ResourceRef};
use crate::time::Timestamp;

/// A bounded, attributable lease on a resource scope — the frozen
/// `ResourceLease` wire shape, carried as data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseRecord {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// Canonical lease ID (`lease_<ULID>`), caller-minted.
    pub id: LeaseRef,
    /// Durable entity version, starting at 1.
    pub version: u64,
    /// The resource this lease scopes.
    pub resource_id: ResourceRef,
    /// The access mode of the lease.
    pub mode: AccessMode,
    /// The owner actor of the lease.
    pub owner: ActorRef,
    /// When the lease was granted (caller-supplied).
    pub granted_at: Timestamp,
    /// The absolute deadline: at and after this timestamp the lease is
    /// not held, without any implicit renewal.
    pub expires_at: Timestamp,
    /// The conflict policy for conflicting lease requests.
    pub conflict_policy: ConflictPolicy,
    /// When the lease was released, if it was released before its
    /// deadline.
    pub released_at: Option<Timestamp>,
}

impl LeaseRecord {
    /// Builds a new lease at version 1. The absolute deadline must lie
    /// strictly after the grant time; a lease that is already expired
    /// when granted cannot exist (the frozen law).
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the deadline is not strictly after
    /// the grant time or a reference fails validation.
    pub fn new(
        id: LeaseRef,
        resource_id: ResourceRef,
        mode: AccessMode,
        owner: ActorRef,
        granted_at: Timestamp,
        expires_at: Timestamp,
        conflict_policy: ConflictPolicy,
    ) -> Result<Self, LeaseError> {
        id.validate()?;
        resource_id.validate()?;
        owner.validate()?;
        if expires_at.is_before(&granted_at) || expires_at == granted_at {
            return Err(LeaseError::invalid(
                "lease expires_at must be strictly after granted_at",
            ));
        }
        Ok(Self {
            v: LeaseVersion,
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

    /// The versioned record of an explicit renewal: the same lease id
    /// at version + 1, granted at the renewal moment, bounded by the
    /// new deadline, carrying the renewal request's policy. The new
    /// deadline must lie strictly after the renewal moment.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the new deadline is not strictly
    /// after the renewal moment.
    pub fn renewed(
        &self,
        granted_at: Timestamp,
        expires_at: Timestamp,
        conflict_policy: ConflictPolicy,
    ) -> Result<Self, LeaseError> {
        if expires_at.is_before(&granted_at) || expires_at == granted_at {
            return Err(LeaseError::invalid(
                "renewed lease expires_at must be strictly after the renewal moment",
            ));
        }
        Ok(Self {
            v: LeaseVersion,
            id: self.id.clone(),
            version: self.version + 1,
            resource_id: self.resource_id.clone(),
            mode: self.mode,
            owner: self.owner.clone(),
            granted_at,
            expires_at,
            conflict_policy,
            released_at: None,
        })
    }

    /// The versioned record of an explicit release: the same lease at
    /// version + 1 with `released_at` set. The release moment must not
    /// be before the grant time.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the release moment precedes the
    /// grant time.
    pub fn released(&self, released_at: Timestamp) -> Result<Self, LeaseError> {
        if released_at.is_before(&self.granted_at) {
            return Err(LeaseError::invalid(
                "lease released_at must not be before granted_at",
            ));
        }
        Ok(Self {
            v: LeaseVersion,
            id: self.id.clone(),
            version: self.version + 1,
            resource_id: self.resource_id.clone(),
            mode: self.mode,
            owner: self.owner.clone(),
            granted_at: self.granted_at,
            expires_at: self.expires_at,
            conflict_policy: self.conflict_policy,
            released_at: Some(released_at),
        })
    }

    /// Validates the lease record against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when any frozen rule fails.
    pub fn validate(&self) -> Result<(), LeaseError> {
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
            return Err(LeaseError::invalid(
                "lease released_at must not be before granted_at",
            ));
        }
        if self.version == 0 {
            return Err(LeaseError::invalid("lease version must be at least 1"));
        }
        Ok(())
    }
}

/// A wait-queue snapshot: one queued request and the moment it was
/// enqueued (the request order). The manager consumes wait queues as
/// data — the durable queue belongs to the caller's store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaitSnapshot {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The queued request.
    pub request: crate::request::LeaseRequest,
    /// When the request was enqueued (its place in the FIFO order).
    pub enqueued_at: Timestamp,
}

impl WaitSnapshot {
    /// Builds a wait snapshot, validating the request and the bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the request fails validation.
    pub fn new(
        request: crate::request::LeaseRequest,
        enqueued_at: Timestamp,
    ) -> Result<Self, LeaseError> {
        request.validate()?;
        Ok(Self {
            v: LeaseVersion,
            request,
            enqueued_at,
        })
    }

    /// Validates the snapshot against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the request fails validation.
    pub fn validate(&self) -> Result<(), LeaseError> {
        self.request.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{fake_actors, fake_lease_ids, fake_resource, now};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn base_record() -> LeaseRecord {
        ok(LeaseRecord::new(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana,
            ok(Timestamp::parse("2026-09-23T15:00:00Z")),
            ok(Timestamp::parse("2026-09-23T15:40:00Z")),
            ConflictPolicy::Queue,
        ))
    }

    #[test]
    fn lease_must_be_bounded_by_a_future_deadline() {
        let granted = ok(Timestamp::parse("2026-09-23T15:00:00Z"));
        let same = granted;
        let before = ok(Timestamp::parse("2026-09-23T14:59:59Z"));
        for deadline in [same, before] {
            assert!(
                LeaseRecord::new(
                    fake_lease_ids()[0].clone(),
                    fake_resource(),
                    AccessMode::Write,
                    fake_actors().ana,
                    granted,
                    deadline,
                    ConflictPolicy::Queue
                )
                .is_err()
            );
        }
    }

    #[test]
    fn expiry_is_enforced_on_read() {
        let record = base_record();
        let moment = now();
        assert!(record.is_held_at(&moment));
        let at_deadline = ok(Timestamp::parse("2026-09-23T15:40:00Z"));
        assert!(
            !record.is_held_at(&at_deadline),
            "at and after the deadline the lease is not held"
        );
        let released = ok(record.released(moment));
        assert!(
            !released.is_held_at(&moment),
            "a released lease is not held"
        );
        assert_eq!(released.version, 2);
        assert_eq!(released.released_at, Some(moment));
    }

    #[test]
    fn renewal_and_release_are_versioned_and_explicit() {
        let record = base_record();
        let renewal_moment = ok(Timestamp::parse("2026-09-23T15:20:00Z"));
        let renewed = ok(record.renewed(
            renewal_moment,
            ok(Timestamp::parse("2026-09-23T16:00:00Z")),
            ConflictPolicy::Reject,
        ));
        assert_eq!(renewed.version, 2);
        assert_eq!(renewed.id, record.id);
        assert_eq!(renewed.granted_at, renewal_moment);
        assert_eq!(
            renewed.expires_at,
            ok(Timestamp::parse("2026-09-23T16:00:00Z"))
        );
        assert_eq!(renewed.conflict_policy, ConflictPolicy::Reject);
        assert_eq!(renewed.mode, record.mode);
        // The original record is untouched — a renewal never mutates in
        // place (no implicit renewal; the extended record is a new
        // version).
        assert_eq!(record.version, 1);
        assert!(
            record
                .renewed(renewal_moment, renewal_moment, ConflictPolicy::Queue)
                .is_err(),
            "a renewal must extend strictly into the future"
        );
    }

    #[test]
    fn records_round_trip_canonically() {
        let record = base_record();
        let serialized = ok(serde_json::to_string(&record));
        let reloaded: LeaseRecord = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, record);
        assert!(serialized.contains("\"resource_id\":\"res_"));
        assert!(serialized.contains("\"expires_at\":\"2026-09-23T15:40:00Z\""));
        assert!(
            serde_json::from_str::<LeaseRecord>(&serialized.replace("\"version\":1", "\"vers\":1"))
                .is_err(),
            "unknown fields are rejected"
        );
    }
}
