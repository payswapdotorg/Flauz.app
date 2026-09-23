//! The lease manager: resource-aware scheduling data with honest
//! conflicts (F2 Wave 5, work order **LEASE-001**).
//!
//! The lease CONTRACT lives in `flauz-world` (the frozen `ResourceLease`:
//! bounded, attributable, expiry enforced on read, no implicit renewal).
//! This crate implements the SEMANTICS over inputs taken as **data**
//! (addendum §5): who gets a contended resource, what happens to a
//! conflicting request, and what the human must decide when a conflict
//! escalates.
//!
//! # The conflict-honesty law (J-16, addendum §2)
//!
//! Every lease conflict is NAMED: who holds, who waits, which policy
//! decided (reject / queue / escalate), and what happens next. The law is
//! STRUCTURAL: every [`LeaseDecision`] variant carries the full request
//! plus its named cause (the holder record, the queue facts, or the
//! conflict pair) — a decision without the named cause cannot be
//! constructed. A silently-queued request, a silently-dropped waiter, or
//! an ambiguous conflict is forbidden:
//!
//! - the queue is FIFO and deterministic (waiters advance in request
//!   order; positions are honest, never flattering);
//! - expiry is honest (a lease not held at the moment is not held — no
//!   implicit renewal; renewal is an EXPLICIT new request that names the
//!   lease it extends);
//! - escalation produces a named "needs the human" record
//!   ([`NamedConflict`]) with the decision the human must make
//!   ([`EscalationChoice`]), never an auto-resolution of a contested
//!   resource — [`resolve_escalation`] applies the human's EXPLICIT,
//!   ATTRIBUTED call.
//!
//! # Inputs only — the frozen formats cross the seam (the flauz-cap law)
//!
//! The manager takes lease snapshots, wait-queue snapshots, requests and
//! policies as plain call-side data. This crate does NOT import
//! `flauz-world`, `flauz-orch` or any other contract crate: foreign
//! entities cross the seam exclusively as frozen format strings —
//! [`ResourceRef`] re-validates the `res_<ULID>` grammar, [`LeaseRef`]
//! the `lease_<ULID>` grammar, and [`ActorRef`] the frozen actor
//! reference shape that `flauz-world` owns — never as re-defined entity
//! types. The [`LeaseRecord`] mirrors the frozen `ResourceLease` field
//! names exactly, so a record serialized by either crate has
//! byte-identical meaning on the wire; the world store keeps owning the
//! durable lease entities (addendum §5).
//!
//! # Determinism (kernel §7, addendum §5)
//!
//! Contract code in this crate never reads wall-clock time or randomness
//! and generates no identifiers: `now` is caller-supplied for every
//! entry point, and the granted lease id is caller-minted through the
//! request. Identical inputs always produce byte-identical decisions
//! (proven by the conformance tests). The in-memory [`fakes`] are fully
//! deterministic.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields (`deny_unknown_fields`),
//! contains no floats, and tags enums internally with `"kind"`
//! (fieldless enums serialize as their lowercase names). Timestamps are
//! RFC 3339 UTC seconds precision (`YYYY-MM-DDTHH:MM:SSZ`); durations
//! are integer milliseconds.
//!
//! # The world-stream event vocabulary
//!
//! The crate defines the `lease.*` event vocabulary as DATA
//! ([`events`]): `lease.requested` / `lease.granted` /
//! `lease.rejected` / `lease.queued` / `lease.escalated` /
//! `lease.released` / `lease.expired` / `lease.renewed` — canonical
//! payload shapes the CALLER records through the existing world-store
//! seam (the frozen `<entity>.<verb_past>` grammar; `lease.granted` and
//! `lease.released` are already registered by `flauz-world`). This
//! crate records nothing itself.

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod events;
pub mod fakes;
pub mod lease;
pub mod manager;
pub mod mode;
pub mod refs;
pub mod request;
pub mod time;

pub use crate::events::{
    EscalatedPayload, EventKind, ExpiredPayload, GrantedPayload, QueuedPayload, RejectedPayload,
    ReleasedPayload, RenewedPayload, RequestedPayload,
};
pub use crate::lease::{LeaseRecord, WaitSnapshot};
pub use crate::manager::{
    LeaseSweep, NamedConflict, WaiterOutcome, decide, resolve_escalation, sweep,
};
pub use crate::mode::{AccessMode, ConflictPolicy, EscalationChoice, EscalationOutcome};
pub use crate::refs::{ActorKind, ActorRef, LeaseRef, ResourceRef, TaskRef};
pub use crate::request::{
    LeaseDecision, LeaseRequest, NamedCause, QueuedAhead, RenewalAttribution,
};
pub use crate::time::Timestamp;

/// Maximum length of an actor id (matches the flauz-world bound).
pub const MAX_ACTOR_ID_BYTES: usize = 256;

/// Maximum number of lease snapshots one manager call accepts (bounded
/// inputs — every external list the crate reads is bounded).
pub const MAX_LEASE_SNAPSHOTS: usize = 256;

/// Maximum number of wait-queue snapshots one manager call accepts.
pub const MAX_WAIT_SNAPSHOTS: usize = 256;

/// Maximum number of granted leases one sweep can produce (the sweep
/// grants at most one lease per waiter; the wait-queue bound governs).
pub const MAX_SWEEP_GRANTS: usize = MAX_WAIT_SNAPSHOTS;

/// Contract-level validation error for values that fail canonical rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseError {
    /// A value violates a canonical rule (bounds, ordering, grammar,
    /// record consistency).
    Invalid(String),
}

impl LeaseError {
    /// Builds a [`LeaseError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl fmt::Display for LeaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid lease state: {reason}"),
        }
    }
}

impl Error for LeaseError {}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LeaseVersion;

impl serde::Serialize for LeaseVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for LeaseVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = u32::deserialize(deserializer)?;
        if version == 1 {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported contract schema version {version}; this build reads v1"
            )))
        }
    }
}

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), LeaseError> {
    if value.is_empty() {
        Err(LeaseError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), LeaseError> {
    if value.len() > max_bytes {
        Err(LeaseError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

/// Validates that a lease-snapshot list is bounded and free of duplicate
/// lease ids (entity uniqueness — the world store owns it; the manager
/// refuses ambiguous data instead of guessing).
pub(crate) fn ensure_lease_snapshots(
    field: &'static str,
    leases: &[LeaseRecord],
) -> Result<(), LeaseError> {
    if leases.len() > MAX_LEASE_SNAPSHOTS {
        return Err(LeaseError::invalid(format!(
            "{field} exceeds {MAX_LEASE_SNAPSHOTS} entries"
        )));
    }
    for lease in leases {
        lease.validate()?;
    }
    let mut seen = std::collections::BTreeSet::new();
    for lease in leases {
        if !seen.insert(lease.id.as_str()) {
            return Err(LeaseError::invalid(format!(
                "{field} contains the lease id {} more than once",
                lease.id
            )));
        }
    }
    Ok(())
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
    fn contract_version_serializes_as_one_and_rejects_others() {
        let serialized = ok(serde_json::to_string(&LeaseVersion));
        assert_eq!(serialized, "1");
        assert!(serde_json::from_str::<LeaseVersion>("1").is_ok());
        assert!(serde_json::from_str::<LeaseVersion>("2").is_err());
        assert!(serde_json::from_str::<LeaseVersion>("0").is_err());
    }
}
