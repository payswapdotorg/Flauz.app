//! User-owned provider accounts, quota attribution and free-tier-first
//! routing (F2 Wave 4, work order **PROV-001**).
//!
//! This crate is the F7 bring-your-own-provider foundation: the contracts
//! that make every scheduled execution **attributable** — whose account it
//! draws on, which provider, which tier (free/paid), and the policy rule
//! that chose it.
//!
//! # The attribution law (Wave-4 addendum §2)
//!
//! Ambient consumption — a task using "some account" invisibly — is
//! forbidden. Every [`SchedulingChoice`](scheduler::SchedulingChoice)
//! names the account (its connection reference and user-words label), the
//! provider, the tier, and the policy rule that chose it; when a free tier
//! depletes and the policy moves to a paid tier, that escalation is NAMED
//! — the preferred account appears in the skipped list with its reason,
//! never a silent fall-through (the CAP-001 law, applied to money). A
//! choice without named account + tier + policy rule cannot be
//! constructed: the fields are non-optional and
//! [`SchedulingChoice::validate`](scheduler::SchedulingChoice::validate)
//! rejects empty labels, invalid references, and escalated choices with no
//! named alternative.
//!
//! # Inputs only — the frozen formats cross the seam (the flauz-cap pattern)
//!
//! The scheduler takes the routing need, the accounts, the routing policy,
//! the usage ledger and the session load as plain call-side **data**. This
//! crate does NOT import `flauz-world`, `flauz-exec`, `flauz-cap` or any
//! other contract crate: foreign entities cross the seam exclusively as
//! frozen format strings —
//! [`SecretRef`](key::SecretRef) re-validates the `flausec_...` reference
//! grammar (including the credential-material rejection) that
//! `flauz-exec`'s `SecretRef` owns, [`ConnectionRef`](key::ConnectionRef)
//! and [`TaskRef`](key::TaskRef) re-validate the frozen `conn_<ULID>` and
//! `task_<ULID>` canonical-ID formats, and
//! [`CapabilityKey`](key::CapabilityKey) re-validates the frozen
//! namespaced-key grammar `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*`. No new ID
//! prefix is invented: an account's identity is its connection reference,
//! and usage records are per-account sequences (append-only, like events).
//!
//! # Credentials are references, forever (kernel §7, addendum §3)
//!
//! An account carries a `flausec_...` secret **reference** and nothing
//! else. Credential **material** never appears in any contract type,
//! fixture, log line or serialized state of this crate: the
//! [`CREDENTIAL_MARKERS`](CREDENTIAL_MARKERS) family is re-validated
//! locally and the conformance scan extends it to every new record family.
//! References that look like raw credentials (`sk-...`, `ghp_...`,
//! `Bearer ...`) are rejected outright so the types cannot smuggle
//! material past the boundary.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields (`deny_unknown_fields`),
//! contains no floats, and tags enums internally with `"kind"` (fieldless
//! enums serialize as their lowercase names). Timestamps are RFC 3339 UTC
//! in the canonical `YYYY-MM-DDTHH:MM:SSZ` form (seconds precision,
//! `Z` suffix), re-validated locally with the same semantics as
//! `flauz-exec`'s `Timestamp` so serialized forms are byte-identical.
//!
//! # Determinism (kernel §7)
//!
//! Contract code in this crate never reads wall-clock time or randomness
//! and generates no identifiers: every timestamp is caller-supplied, the
//! scheduler is a pure function of its inputs, and ordering ties break on
//! the canonical connection reference (never insertion order). The
//! in-memory [`fakes`] are fully deterministic.
//!
//! # Quota state is data snapshots, never live counters
//!
//! A [`QuotaWindow`](account::QuotaWindow) is a point-in-time snapshot
//! (remaining / limit / window bounds) recorded on the account. Depletion
//! is a **projection**
//! ([`project_depletion`](ledger::project_depletion)): given a window
//! state and a projected consumption, the honest remaining state — the
//! projected remaining saturates at zero and any deficit is NAMED, never
//! silently negative.

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod account;
pub mod fakes;
pub mod key;
pub mod ledger;
pub mod policy;
pub mod scheduler;

pub use crate::account::{AccountStore, AccountStoreError, ProviderAccount, QuotaWindow, TierKind};
pub use crate::key::{CREDENTIAL_MARKERS, CapabilityKey, ConnectionRef, SecretRef, TaskRef};
pub use crate::ledger::{DepletionProjection, QuotaLedger, UsageRecord, project_depletion};
pub use crate::policy::{ProviderOverride, RoutingOrder, RoutingPolicy, TaskOverride};
pub use crate::scheduler::{
    PolicyRule, RoutingNeed, SchedulingChoice, SchedulingInputs, SessionLoad, SkipReason,
    SkippedAccount, schedule_routing,
};
pub use crate::time::Timestamp;

/// Maximum length of a name or label (matches the flauz-exec bound).
pub const MAX_NAME_BYTES: usize = 256;

/// Maximum length of a reference string (matches the flauz-exec bound).
pub const MAX_REFERENCE_BYTES: usize = 2048;

/// Maximum number of capability keys in one routing need (matches the
/// flauz-exec bound).
pub const MAX_CAPABILITY_KEYS: usize = 64;

/// Maximum number of accounts held by one account store (multi-account
/// per provider, bounded).
pub const MAX_ACCOUNTS: usize = 64;

/// Maximum number of usage records per account in one ledger (bounded,
/// like every external frame and history query).
pub const MAX_USAGE_RECORDS_PER_ACCOUNT: usize = 256;

/// Maximum length of a user-words explanation (matches the flauz-cap
/// bound).
pub const MAX_EXPLANATION_BYTES: usize = 512;

/// Contract-level validation error for values that fail canonical rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvError {
    /// A value violates a canonical rule (bounds, ordering, grammar,
    /// record consistency).
    Invalid(String),
}

impl ProvError {
    /// Builds a [`ProvError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl fmt::Display for ProvError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid provider state: {reason}"),
        }
    }
}

impl Error for ProvError {}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProvVersion;

impl serde::Serialize for ProvVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for ProvVersion {
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

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), ProvError> {
    if value.is_empty() {
        Err(ProvError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), ProvError> {
    if value.len() > max_bytes {
        Err(ProvError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_explanation(field: &'static str, value: &str) -> Result<(), ProvError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_EXPLANATION_BYTES)
}

/// Validates a neutral kind label: a lowercase token `[a-z][a-z0-9-]*`
/// (the frozen flauz-exec grammar shared by provider kinds).
pub(crate) fn ensure_kind_label(field: &'static str, value: &str) -> Result<(), ProvError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_NAME_BYTES)?;
    let valid = value
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        });
    if !valid {
        return Err(ProvError::invalid(format!(
            "{field} must be a lowercase token (`[a-z][a-z0-9-]*`), found {value:?}"
        )));
    }
    Ok(())
}

/// The private ULID validation module (kernel §2): the frozen Crockford
/// Base32 form, 26 characters, uppercase, no `I`/`L`/`O`/`U`. This crate
/// never GENERATES identifiers — references arrive as caller-supplied
/// data — so the module validates only (stronger than the kernel's
/// determinism rule: no entropy source exists here at all).
pub(crate) mod ulid {
    /// The frozen Crockford Base32 alphabet (no `I`, `L`, `O`, `U`).
    pub(crate) const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

    /// The frozen ULID length.
    pub(crate) const ULID_LEN: usize = 26;

    /// Validates the ULID part of a canonical ID.
    pub(crate) fn is_valid_ulid(value: &str) -> bool {
        value.len() == ULID_LEN && value.bytes().all(|byte| CROCKFORD_ALPHABET.contains(&byte))
    }
}

/// The canonical-timestamp module (kernel §4): RFC 3339 UTC, seconds
/// precision, `Z` suffix — the same semantics as `flauz-exec`'s
/// `Timestamp`, so serialized timestamps are byte-identical across the
/// seam. Parsing accepts standard RFC 3339 variants and truncates
/// sub-second precision; contract code never reads the wall clock.
pub mod time {
    use std::fmt;

    use chrono::{DateTime, SubsecRound, Utc};

    use crate::ProvError;

    /// An RFC 3339 UTC timestamp with seconds precision.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Timestamp(DateTime<Utc>);

    impl Timestamp {
        /// Parses an RFC 3339 timestamp (any standard offset or fractional
        /// precision) and normalizes it to UTC seconds precision.
        ///
        /// # Errors
        ///
        /// Returns [`ProvError::Invalid`] when the value is not RFC 3339.
        pub fn parse(value: &str) -> Result<Self, ProvError> {
            let parsed = DateTime::parse_from_rfc3339(value).map_err(|error| {
                ProvError::invalid(format!("invalid RFC 3339 timestamp {value:?}: {error}"))
            })?;
            Ok(Self(parsed.with_timezone(&Utc).trunc_subsecs(0)))
        }

        /// Builds a timestamp from a UTC date-time, truncating sub-second
        /// precision.
        #[must_use]
        pub fn from_datetime(value: DateTime<Utc>) -> Self {
            Self(value.trunc_subsecs(0))
        }

        /// Returns the underlying UTC date-time (seconds precision).
        #[must_use]
        pub fn as_datetime(&self) -> DateTime<Utc> {
            self.0
        }

        /// Returns the canonical `YYYY-MM-DDTHH:MM:SSZ` serialization.
        #[must_use]
        pub fn to_rfc3339(&self) -> String {
            self.0.format("%Y-%m-%dT%H:%M:%SZ").to_string()
        }

        /// Whether this timestamp is strictly before `other`.
        #[must_use]
        pub fn is_before(&self, other: Timestamp) -> bool {
            self.0 < other.0
        }
    }

    impl fmt::Display for Timestamp {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(&self.to_rfc3339())
        }
    }

    impl serde::Serialize for Timestamp {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_rfc3339())
        }
    }

    impl<'de> serde::Deserialize<'de> for Timestamp {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let text = String::deserialize(deserializer)?;
            Self::parse(&text).map_err(serde::de::Error::custom)
        }
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
    fn contract_version_serializes_as_one_and_rejects_others() {
        let serialized = ok(serde_json::to_string(&ProvVersion));
        assert_eq!(serialized, "1");
        assert!(serde_json::from_str::<ProvVersion>("1").is_ok());
        assert!(serde_json::from_str::<ProvVersion>("2").is_err());
        assert!(serde_json::from_str::<ProvVersion>("0").is_err());
    }

    #[test]
    fn kind_labels_follow_the_frozen_grammar() {
        assert!(ensure_kind_label("provider kind", "openai").is_ok());
        assert!(ensure_kind_label("provider kind", "flauz-fake").is_ok());
        assert!(ensure_kind_label("provider kind", "e2b").is_ok());
        for invalid in ["", "OpenAI", "open ai", "openai!", "1openai"] {
            assert!(
                ensure_kind_label("provider kind", invalid).is_err(),
                "{invalid:?} must not be a kind label"
            );
        }
        let long = "a".repeat(MAX_NAME_BYTES + 1);
        assert!(ensure_kind_label("provider kind", &long).is_err());
    }

    #[test]
    fn timestamps_serialize_canonically_and_order() {
        let ts = ok(Timestamp::parse("2026-09-23T09:00:00Z"));
        assert_eq!(ts.to_rfc3339(), "2026-09-23T09:00:00Z");
        assert_eq!(ts.to_string(), "2026-09-23T09:00:00Z");
        let serialized = ok(serde_json::to_string(&ts));
        assert_eq!(serialized, "\"2026-09-23T09:00:00Z\"");
        let reloaded: Timestamp = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, ts);
        // Parsing accepts offsets and fractional seconds, truncating to
        // the canonical seconds-precision form (the flauz-exec vectors).
        let offset = ok(Timestamp::parse("2026-09-23T11:00:00.900+02:00"));
        assert_eq!(offset, ts);
        assert!(Timestamp::parse("not a timestamp").is_err());
        assert!(Timestamp::parse("2026-09-23").is_err());
        let later = ok(Timestamp::parse("2026-09-23T09:00:01Z"));
        assert!(ts.is_before(later));
        assert!(!later.is_before(ts));
    }

    #[test]
    fn ulids_validate_the_frozen_crockford_form() {
        assert!(ulid::is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPA0"));
        assert!(!ulid::is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPA"));
        assert!(!ulid::is_valid_ulid("01j8zq5v8k3t2b7n6x4r9dqpa0"));
        assert!(!ulid::is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPI0"));
        assert!(!ulid::is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPL0"));
        assert!(!ulid::is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPO0"));
        assert!(!ulid::is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPU0"));
        assert!(!ulid::is_valid_ulid(""));
    }
}
