//! The frozen-format reference newtypes: the strings that cross the seam
//! between `flauz-exec`, `flauz-world` and this crate as **data** (the
//! flauz-cap pattern — never as cargo dependencies).
//!
//! - [`SecretRef`] re-validates the `flausec_...` secret-reference grammar
//!   `flauz-exec`'s `SecretRef` owns, including the
//!   [`CREDENTIAL_MARKERS`] rejection: references that look like raw
//!   credentials are refused outright so credential **material** can never
//!   smuggle itself past the boundary inside a reference (kernel §7,
//!   Wave-4 addendum §3).
//! - [`ConnectionRef`] re-validates the frozen `conn_<ULID>` canonical-ID
//!   format of `flauz-exec`'s `ProviderConnectionId`.
//! - [`TaskRef`] re-validates the frozen `task_<ULID>` canonical-ID format
//!   of `flauz-world`'s task identity.
//! - [`CapabilityKey`] re-validates the frozen namespaced-key grammar of
//!   `flauz-exec`'s `CapabilityId` (identical to `flauz-cap`'s local
//!   re-validation), used by routing needs.
//!
//! No new identifier kind is invented anywhere in this crate: an account's
//! identity is its connection reference, and usage records are per-account
//! sequences.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{MAX_NAME_BYTES, MAX_REFERENCE_BYTES, ProvError, ensure_str_bound};

/// The marker list of credential-material shapes that must never appear
/// in or behind a secret reference (the frozen `flauz-exec`
/// `CREDENTIAL_MARKERS` family, re-pinned here as data so the scan and the
/// rejection behave identically across the seam).
pub const CREDENTIAL_MARKERS: &[&str] = &[
    "sk-",
    "Bearer ",
    "api_key",
    "apikey",
    "password",
    "passwd",
    "client_secret",
    "access_token",
    "refresh_token",
    "PRIVATE KEY",
    "BEGIN RSA",
    "xoxb-",
    "ghp_",
];

/// An opaque reference to a secret held by the secret store: a
/// `flausec_...` string (kernel §7). The reference is a bounded, printable
/// token; it is never credential material itself, and references that look
/// like raw credentials are rejected outright so the type cannot smuggle
/// material past the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SecretRef(String);

impl SecretRef {
    /// The mandatory prefix of every secret reference.
    pub const PREFIX: &'static str = "flausec_";

    /// Parses and validates a secret reference: mandatory `flausec_`
    /// prefix, non-empty bounded remainder of printable ASCII tokens, and
    /// no credential-material markers anywhere in the reference.
    pub fn parse(value: &str) -> Result<Self, ProvError> {
        let Some(rest) = value.strip_prefix(Self::PREFIX) else {
            return Err(ProvError::invalid(format!(
                "secret reference must start with {:?}, found {value:?}",
                Self::PREFIX
            )));
        };
        if rest.is_empty() {
            return Err(ProvError::invalid(
                "secret reference must not be empty after the flausec_ prefix",
            ));
        }
        ensure_str_bound("secret reference", value, MAX_REFERENCE_BYTES)?;
        if !rest.chars().all(|character| character.is_ascii_graphic()) {
            return Err(ProvError::invalid(
                "secret reference must be a printable token without whitespace",
            ));
        }
        for marker in CREDENTIAL_MARKERS {
            if value.contains(marker) {
                return Err(ProvError::invalid(format!(
                    "secret reference must be an opaque token, not credential material \
                     (contains {marker:?})"
                )));
            }
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the secret reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the secret reference.
    pub fn validate(&self) -> Result<(), ProvError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for SecretRef {
    type Err = ProvError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for SecretRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SecretRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A reference to a provider connection: the frozen `conn_<ULID>`
/// canonical-ID format owned by `flauz-exec`'s `ProviderConnectionId`,
/// re-validated locally as data. This is the identity of a user-owned
/// provider account in this crate (no new ID prefix is invented).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConnectionRef(String);

impl ConnectionRef {
    /// The frozen kind prefix of a provider connection ID.
    pub const PREFIX: &'static str = "conn";

    /// Parses and validates a connection reference (`conn_<ULID>`).
    pub fn parse(value: &str) -> Result<Self, ProvError> {
        parse_canonical_id(value, Self::PREFIX)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the connection reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the connection reference.
    pub fn validate(&self) -> Result<(), ProvError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

impl fmt::Display for ConnectionRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ConnectionRef {
    type Err = ProvError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for ConnectionRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ConnectionRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A reference to a task: the frozen `task_<ULID>` canonical-ID format
/// owned by `flauz-world`, re-validated locally as data. Task identity is
/// sacred (kernel §6): routing attribution records WHICH task consumed
/// quota, and never forks or redefines it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskRef(String);

impl TaskRef {
    /// The frozen kind prefix of a task ID.
    pub const PREFIX: &'static str = "task";

    /// Parses and validates a task reference (`task_<ULID>`).
    pub fn parse(value: &str) -> Result<Self, ProvError> {
        parse_canonical_id(value, Self::PREFIX)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the task reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the task reference.
    pub fn validate(&self) -> Result<(), ProvError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

impl fmt::Display for TaskRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for TaskRef {
    type Err = ProvError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for TaskRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for TaskRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// Validates a canonical `<kind>_<ULID>` identifier of the expected kind
/// (kernel §2): lowercase kind prefix, `_` separator, 26-character
/// uppercase Crockford Base32 ULID.
fn parse_canonical_id(value: &str, expected_kind: &str) -> Result<(), ProvError> {
    let Some((kind, ulid_part)) = value.split_once('_') else {
        return Err(ProvError::invalid(format!(
            "canonical id must be `<kind>_<ULID>`, found {value:?}"
        )));
    };
    if kind != expected_kind {
        return Err(ProvError::invalid(format!(
            "expected a {expected_kind:?} id, found kind {kind:?}"
        )));
    }
    if !crate::ulid::is_valid_ulid(ulid_part) {
        return Err(ProvError::invalid(format!(
            "canonical id must carry a 26-character uppercase Crockford Base32 ULID, \
             found {ulid_part:?}"
        )));
    }
    Ok(())
}

/// A namespaced capability key (kernel §2): segments of
/// `[a-z][a-z0-9_]*` separated by dots, for example `terminal` or
/// `browser.input` — the frozen grammar owned by `flauz-exec`'s
/// `CapabilityId`, re-validated locally (identical to `flauz-cap`'s local
/// re-validation) so a key serialized by any crate has byte-identical
/// meaning on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CapabilityKey(String);

impl CapabilityKey {
    /// Parses and validates a capability key against the frozen grammar.
    pub fn parse(value: &str) -> Result<Self, ProvError> {
        if value.is_empty() {
            return Err(ProvError::invalid("capability key must not be empty"));
        }
        for segment in value.split('.') {
            let mut characters = segment.chars();
            let valid = characters
                .next()
                .is_some_and(|first| first.is_ascii_lowercase())
                && characters.all(|character| {
                    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
                });
            if !valid {
                return Err(ProvError::invalid(format!(
                    "capability key {value:?} segments must be `[a-z][a-z0-9_]*`"
                )));
            }
        }
        ensure_str_bound("capability key", value, MAX_NAME_BYTES)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the capability key string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the capability key.
    pub fn validate(&self) -> Result<(), ProvError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

impl fmt::Display for CapabilityKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for CapabilityKey {
    type Err = ProvError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for CapabilityKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CapabilityKey {
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
    fn secret_refs_are_opaque_flausec_tokens() {
        // The frozen flauz-exec vectors, re-pinned: the formats are
        // identical across the seam.
        assert!(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34").is_ok());
        assert!(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34:work").is_ok());
        assert!(SecretRef::parse("flausec_").is_err());
        assert!(SecretRef::parse("sk-proj-abcdefgh1234").is_err());
        assert!(SecretRef::parse("flausec_sk-proj-abcdefgh1234").is_err());
        assert!(SecretRef::parse("flausec_token with space").is_err());
        assert!(SecretRef::parse("flausec_ghp_0123456789abcdef").is_err());
        assert!(SecretRef::parse("").is_err());
        let reference = ok(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34"));
        let serialized = ok(serde_json::to_string(&reference));
        assert_eq!(serialized, "\"flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34\"");
        let parsed: SecretRef = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, reference);
    }

    #[test]
    fn every_credential_marker_is_rejected_inside_references() {
        for marker in CREDENTIAL_MARKERS {
            // Markers that contain whitespace fail the printable-token
            // check; markers without it fail the marker scan itself —
            // either way the reference is rejected, never stored.
            let smuggle = format!("flausec_{marker}value");
            assert!(
                SecretRef::parse(&smuggle).is_err(),
                "a reference containing {marker:?} must be rejected as credential material"
            );
        }
    }

    #[test]
    fn connection_and_task_refs_follow_the_frozen_id_format() {
        // The frozen kernel §2 vectors, re-pinned for both kinds.
        assert_eq!(
            ok(ConnectionRef::parse("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0")).as_str(),
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0"
        );
        assert_eq!(
            ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1")).as_str(),
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"
        );
        for invalid in [
            "taskX_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "TASK_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "task_01j8zq5v8k3t2b7n6x4r9dqpa0",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPA",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPI0",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPU0",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPL0",
            "task_01J8ZQ5V8K3T2B7N6X4R9DQPO0",
            "task",
            "task_",
            "",
            "01J8ZQ5V8K3T2B7N6X4R9DQPA0",
        ] {
            assert!(
                TaskRef::parse(invalid).is_err(),
                "{invalid:?} must not parse as a task reference"
            );
            assert!(
                ConnectionRef::parse(invalid).is_err(),
                "{invalid:?} must not parse as a connection reference"
            );
        }
        // A valid ULID of the wrong kind is refused by the typed parse.
        assert!(matches!(
            ConnectionRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"),
            Err(ProvError::Invalid(_))
        ));
        let conn = ok(ConnectionRef::parse("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0"));
        let serialized = ok(serde_json::to_string(&conn));
        assert_eq!(serialized, "\"conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0\"");
        let reloaded: ConnectionRef = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, conn);
        let task = ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"));
        let serialized = ok(serde_json::to_string(&task));
        let reloaded: TaskRef = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, task);
    }

    #[test]
    fn capability_keys_follow_the_frozen_grammar() {
        for valid in ["terminal", "browser.input", "filesystem.read", "web.search"] {
            assert_eq!(ok(CapabilityKey::parse(valid)).as_str(), valid);
        }
        for invalid in ["", "Terminal", "browser..input", ".input", "browser.Input"] {
            assert!(
                CapabilityKey::parse(invalid).is_err(),
                "{invalid:?} must not parse"
            );
        }
    }
}
