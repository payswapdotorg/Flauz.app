//! Workspace collaboration contracts (F2 Wave 4, work order **COL-001**):
//! membership, roles, permissions with named denial, presence, and the
//! private-vs-shared sharing policy — the F9 collaboration fabric.
//!
//! A workspace has members ([`WorkspaceMembership`]), each carrying a
//! [`Role`] whose defaults come from a [`RoleLattice`] carried as **data**:
//! what each role may do per surface family (tasks, provider accounts,
//! environments, members). [`PermissionGrant`] records override the role
//! default per surface — an explicit allow or an explicit deny. The
//! authorization evaluator ([`authorize`]) takes actor + action + surface +
//! grants + lattice as plain call-side data and answers
//! [`AuthorizationDecision`]: **Allow or Deny with the NAMED reason** —
//! never a silent no-op (the CAP-001 law applied to access).
//!
//! Presence is a [`PresenceRecord`] (member ref + surface + freshness
//! bounds as data) in a bounded, replaceable [`PresenceTable`] — never an
//! event stream of its own (the Wave-4 addendum §5 law). The sharing
//! posture is explicit per task: [`WorktreePolicy`] (isolated by default;
//! the shared-filesystem mode `off | read | write` as a record, never
//! ambient) and [`ContextVisibility`] (member-private vs workspace-shared
//! memory/artifact visibility — the projection filter the caller applies).
//!
//! The deterministic two-actor simulation ([`run_interleaved`]) proves the
//! **F9 gate law** before any real transport exists: two actors driving
//! the same store-facing seam with interleaved operations — permissions
//! hold under interleaving, private records stay private to their actor,
//! authorized state is never lost (every legal operation lands or is
//! denied-with-reason; no lost-update on the simulated seam), and the
//! interleaving log is canonical-JSON replayable.
//!
//! # Inputs only — the frozen formats cross the seam (the flauz-cap
//! pattern)
//!
//! This crate does NOT import `flauz-world`, `flauz-exec`,
//! `flauz-context`, `flauz-cap` or `flauz-orch`: foreign entities cross
//! the seam exclusively as frozen format strings. [`refs`] re-validates
//! the frozen canonical-ID grammar (`<kind>_<ULID>`, Crockford Base32)
//! for the kinds collaboration references (`ws`, `task`), plus the frozen
//! actor-reference grammar `{"kind": "user|agent|system|provider", "id":
//! "…"}` — byte-identical in meaning to `flauz-world`'s types on the
//! wire. Membership/permission/presence/sharing records land in the real
//! world store through the EXISTING seams as data; this crate never
//! touches the store itself.
//!
//! # Registered event types (kernel §5)
//!
//! This crate registers the `event_type` vocabulary its records produce
//! through the existing world-store seam (frozen-format discipline; the
//! grammar `<entity>.<verb_past>` applies):
//!
//! | Constant | `event_type` |
//! |---|---|
//! | [`event_types::MEMBER_ADDED`] | `member.added` |
//! | [`event_types::MEMBER_ROLE_CHANGED`] | `member.role_changed` |
//! | [`event_types::MEMBER_REMOVED`] | `member.removed` |
//! | [`event_types::PERMISSION_GRANTED`] | `permission.granted` |
//! | [`event_types::PERMISSION_REVOKED`] | `permission.revoked` |
//! | [`event_types::SHARING_CHANGED`] | `sharing.changed` |
//!
//! Presence deliberately registers NO event type: presence records are
//! bounded and replaceable, never an event stream (addendum §5).
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields
//! (`deny_unknown_fields`), contains no floats, and tags enums
//! internally with `"kind"` (fieldless enums serialize as their lowercase
//! snake_case names). Timestamps serialize as RFC 3339 UTC
//! `YYYY-MM-DDTHH:MM:SSZ` (seconds precision); durations are integer
//! milliseconds.
//!
//! # Determinism (kernel §7)
//!
//! Contract code in this crate never reads wall-clock time or randomness
//! and generates no identifiers: every timestamp is passed in by callers,
//! and the evaluator/simulator are pure functions of their inputs, so
//! identical inputs produce identical records — byte-identical
//! serialized logs. The in-memory [`fakes`] are fully deterministic.
//!
//! # Credentials are references, forever (Wave-4 addendum §3)
//!
//! Collaboration records carry member identity references, display names
//! and policy data only. No credential material occurs anywhere in this
//! crate's contracts, fakes, fixtures or logs (the conformance test
//! scans for it).

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod fakes;
pub mod membership;
pub mod permission;
pub mod policy;
pub mod presence;
pub mod refs;
pub mod simulator;
pub mod time;

pub use crate::membership::{
    AccessLevel, Role, RoleLattice, SurfaceAccess, SurfaceFamily, WorkspaceMembership,
};
pub use crate::permission::{
    AllowRule, AuthorizationDecision, AuthorizationRequest, DecisionOutcome, DenialReason,
    GrantEffect, PermissionGrant, authorize,
};
pub use crate::policy::{ContextVisibility, SharedFilesystemMode, Visibility, WorktreePolicy};
pub use crate::presence::{
    DEFAULT_PRESENCE_STALE_AFTER_MS, PresenceFreshness, PresenceRecord, PresenceTable,
    SurfaceLocator,
};
pub use crate::refs::{ActorKind, ActorRef, TaskRef, WorkspaceRef};
pub use crate::simulator::{
    ExpectedOutcome, InterleavedStep, InterleavingTable, ProjectionRow, SimulatedOperation,
    SimulatedStore, SimulationInputs, SimulationLog, SimulationOutcome, SimulationRecord,
    SimulationResult, StoreEntry, event_types, run_interleaved, verify_expectations,
    verify_f9_laws,
};
pub use crate::time::Timestamp;

/// Maximum length of short human-readable strings (display names, store
/// keys — matches the flauz-world name bound).
pub const MAX_NAME_BYTES: usize = 256;

/// Maximum length of honest reasons, consequences and other bounded prose
/// fields (matches the flauz-cap explanation bound).
pub const MAX_EXPLANATION_BYTES: usize = 512;

/// Maximum length of a simulated store entry's value.
pub const MAX_STORE_VALUE_BYTES: usize = 2048;

/// Maximum number of members in one workspace roster.
pub const MAX_MEMBERS: usize = 64;

/// Maximum number of grants in one authorization input set.
pub const MAX_GRANTS: usize = 64;

/// Maximum number of roles in one lattice (the four frozen roles, plus
/// headroom for a future family — validation still rejects duplicates).
pub const MAX_LATTICE_ROLES: usize = 8;

/// The number of frozen surface families (tasks, provider accounts,
/// environments, members).
pub const SURFACE_FAMILIES: usize = 4;

/// Maximum number of records in one presence table (bounded, addendum
/// §5).
pub const MAX_PRESENCE_RECORDS: usize = 16;

/// Maximum number of entries in the simulated store.
pub const MAX_STORE_ENTRIES: usize = 64;

/// Maximum number of steps in one interleaving table.
pub const MAX_INTERLEAVING_STEPS: usize = 128;

/// Contract-level validation error for values that fail canonical rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollabError {
    /// A value violates a canonical rule (bounds, ordering, grammar,
    /// record consistency, simulation invariant).
    Invalid(String),
}

impl CollabError {
    /// Builds a [`CollabError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl fmt::Display for CollabError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid collaboration state: {reason}"),
        }
    }
}

impl Error for CollabError {}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollabVersion;

impl serde::Serialize for CollabVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for CollabVersion {
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

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), CollabError> {
    if value.is_empty() {
        Err(CollabError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), CollabError> {
    if value.len() > max_bytes {
        Err(CollabError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_name(field: &'static str, value: &str) -> Result<(), CollabError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_NAME_BYTES)
}

pub(crate) fn ensure_list_bound(
    field: &'static str,
    list_len: usize,
    max_entries: usize,
) -> Result<(), CollabError> {
    if list_len > max_entries {
        Err(CollabError::invalid(format!(
            "{field} exceeds {max_entries} entries"
        )))
    } else {
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
    fn the_schema_version_marker_round_trips_and_rejects_other_versions() {
        assert_eq!(ok(serde_json::to_string(&CollabVersion)), "1");
        let reloaded: CollabVersion = ok(serde_json::from_str("1"));
        assert_eq!(reloaded, CollabVersion);
        assert!(
            serde_json::from_str::<CollabVersion>("2").is_err(),
            "a future schema version must fail the canonical read, never silently misread"
        );
    }
}
