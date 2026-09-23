//! Canonical timestamps (kernel §4) — the trimmed `flauz-world` /
//! `flauz-collab` pattern.
//!
//! Timestamps serialize as RFC 3339 UTC in the canonical form
//! `YYYY-MM-DDTHH:MM:SSZ` (seconds precision, `Z` suffix). Parsing
//! accepts standard RFC 3339 variants (offsets, fractional seconds);
//! sub-second precision is truncated on parse so that the canonical
//! re-serialization is always the seconds-precision form.
//!
//! Contract code never reads the wall clock: every [`Timestamp`] is
//! supplied by the caller (kernel §7 — determinism).

use std::fmt;

use chrono::{DateTime, SubsecRound, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::LeaseError;

/// An RFC 3339 UTC timestamp with seconds precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Parses an RFC 3339 timestamp (any standard offset or fractional
    /// precision) and normalizes it to UTC seconds precision.
    ///
    /// # Errors
    ///
    /// Returns [`LeaseError`] when the value is not a valid RFC 3339
    /// timestamp.
    pub fn parse(value: &str) -> Result<Self, LeaseError> {
        let parsed = DateTime::parse_from_rfc3339(value).map_err(|error| {
            LeaseError::invalid(format!("invalid RFC 3339 timestamp {value:?}: {error}"))
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

    /// Returns `true` when this timestamp is strictly before `other`.
    #[must_use]
    pub fn is_before(&self, other: &Timestamp) -> bool {
        self.0 < other.0
    }

    /// Returns `true` when this timestamp is strictly after `other`.
    #[must_use]
    pub fn is_after(&self, other: &Timestamp) -> bool {
        self.0 > other.0
    }

    /// Returns the whole milliseconds from `other` to this timestamp
    /// (integer milliseconds — the kernel duration law). Saturating at
    /// zero: a wait measured backwards is zero, never negative.
    #[must_use]
    pub fn milliseconds_since(&self, other: &Timestamp) -> u64 {
        self.0
            .signed_duration_since(other.0)
            .num_milliseconds()
            .max(0) as u64
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_rfc3339())
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
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
    fn timestamps_serialize_to_the_canonical_seconds_form() {
        let moment = ok(Timestamp::parse("2026-09-23T15:04:05Z"));
        assert_eq!(moment.to_rfc3339(), "2026-09-23T15:04:05Z");
        let serialized = ok(serde_json::to_string(&moment));
        assert_eq!(serialized, "\"2026-09-23T15:04:05Z\"");

        // Parsing accepts RFC 3339 variants and truncates to seconds.
        let fractional = ok(Timestamp::parse("2026-09-23T15:04:05.978+02:00"));
        assert_eq!(fractional.to_rfc3339(), "2026-09-23T13:04:05Z");
        let reloaded: Timestamp = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, moment);
        assert!(Timestamp::parse("2026-09-23 15:04:05").is_err());
    }

    #[test]
    fn durations_are_integer_milliseconds_and_saturate_at_zero() {
        let from = ok(Timestamp::parse("2026-09-23T15:00:00Z"));
        let to = ok(Timestamp::parse("2026-09-23T15:12:30Z"));
        assert_eq!(to.milliseconds_since(&from), (12 * 60 + 30) * 1000);
        assert_eq!(from.milliseconds_since(&to), 0, "a backwards wait is zero");
        assert!(from.is_before(&to));
        assert!(to.is_after(&from));
    }
}
