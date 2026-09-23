//! Canonical timestamps (kernel §4) — the trimmed `flauz-world` pattern.
//!
//! Timestamps serialize as RFC 3339 UTC in the canonical form
//! `YYYY-MM-DDTHH:MM:SSZ` (seconds precision, `Z` suffix). Parsing accepts
//! standard RFC 3339 variants (offsets, fractional seconds); sub-second
//! precision is truncated on parse so that the canonical re-serialization
//! is always the seconds-precision form.
//!
//! Contract code never reads the wall clock: every [`Timestamp`] is
//! supplied by the caller (kernel §7 — determinism).

use std::fmt;

use chrono::{DateTime, SubsecRound, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::CollabError;

/// An RFC 3339 UTC timestamp with seconds precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Parses an RFC 3339 timestamp (any standard offset or fractional
    /// precision) and normalizes it to UTC seconds precision.
    pub fn parse(value: &str) -> Result<Self, CollabError> {
        let parsed = DateTime::parse_from_rfc3339(value).map_err(|error| {
            CollabError::invalid(format!("invalid RFC 3339 timestamp {value:?}: {error}"))
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

    /// Returns this timestamp advanced by `milliseconds` (integer
    /// milliseconds — the kernel duration law). Used by freshness
    /// bounds; saturating on overflow.
    #[must_use]
    pub fn plus_milliseconds(&self, milliseconds: u64) -> Timestamp {
        let added = self
            .0
            .checked_add_signed(chrono::Duration::milliseconds(
                milliseconds.min(i64::MAX as u64) as i64,
            ))
            .unwrap_or(self.0);
        Self(added.trunc_subsecs(0))
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
    fn canonical_form_is_seconds_precision_utc() {
        let ts = ok(Timestamp::parse("2026-09-23T10:00:00Z"));
        assert_eq!(ts.to_rfc3339(), "2026-09-23T10:00:00Z");
        assert_eq!(ts.to_string(), "2026-09-23T10:00:00Z");
    }

    #[test]
    fn parsing_accepts_offsets_and_fractional_seconds_and_truncates() {
        let from_offset = ok(Timestamp::parse("2026-09-23T12:00:30+02:00"));
        let from_fraction = ok(Timestamp::parse("2026-09-23T10:00:30.987Z"));
        assert_eq!(from_offset, from_fraction);
        assert_eq!(from_fraction.to_rfc3339(), "2026-09-23T10:00:30Z");
    }

    #[test]
    fn rejects_non_rfc3339_input() {
        assert!(Timestamp::parse("not a timestamp").is_err());
        assert!(Timestamp::parse("2026-09-23").is_err());
        assert!(Timestamp::parse("").is_err());
    }

    #[test]
    fn serde_uses_the_canonical_string_form() {
        let ts = ok(Timestamp::parse("2026-09-23T10:00:00.500+01:00"));
        let serialized = ok(serde_json::to_string(&ts));
        assert_eq!(serialized, "\"2026-09-23T09:00:00Z\"");
        let parsed: Timestamp = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, ts);
    }

    #[test]
    fn plus_milliseconds_advances_deterministically() {
        let ts = ok(Timestamp::parse("2026-09-23T10:00:00Z"));
        assert_eq!(
            ok(Timestamp::parse("2026-09-23T10:01:00Z")),
            ts.plus_milliseconds(60_000)
        );
        assert!(
            ts.plus_milliseconds(60_000)
                .is_before(&ts.plus_milliseconds(120_000))
        );
    }
}
