//! The capability key: the frozen namespaced-string-key grammar (kernel §2).
//!
//! Capability identity is a namespaced string key — **not** a ULID entity:
//! `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*`, for example `terminal` or
//! `browser.input`. The grammar is owned by `flauz-exec`'s `CapabilityId`;
//! this crate re-validates the same frozen format locally (the
//! flauz-context pattern: references cross the seam as frozen format
//! strings, never as cargo dependencies), so a key serialized by either
//! crate has byte-identical meaning on the wire.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{CapError, MAX_NAME_BYTES, ensure_str_bound};

/// A namespaced capability key (kernel §2): segments of
/// `[a-z][a-z0-9_]*` separated by dots, for example `terminal` or
/// `browser.input`. Capability identity is a string key, never a ULID
/// entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CapabilityKey(String);

impl CapabilityKey {
    /// Parses and validates a capability key against the frozen grammar.
    pub fn parse(value: &str) -> Result<Self, CapError> {
        if value.is_empty() {
            return Err(CapError::invalid("capability key must not be empty"));
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
                return Err(CapError::invalid(format!(
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
    pub fn validate(&self) -> Result<(), CapError> {
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
    type Err = CapError;

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
    fn capability_keys_follow_the_frozen_grammar() {
        // The frozen grammar vectors of flauz-exec's CapabilityId, re-pinned
        // here: the formats are identical across the seam.
        for valid in [
            "terminal",
            "browser.input",
            "filesystem.read",
            "computer.screen",
            "persistent_storage",
            "web.search",
            "mcp",
        ] {
            assert_eq!(ok(CapabilityKey::parse(valid)).as_str(), valid);
        }
        for invalid in [
            "",
            "Terminal",
            "browser..input",
            ".input",
            "input.",
            "browser input",
            "browser.Input",
        ] {
            assert!(
                CapabilityKey::parse(invalid).is_err(),
                "{invalid:?} must not parse"
            );
        }
    }

    #[test]
    fn capability_keys_serialize_as_plain_strings() {
        let key = ok(CapabilityKey::parse("browser.input"));
        let serialized = ok(serde_json::to_string(&key));
        assert_eq!(serialized, "\"browser.input\"");
        let reloaded: CapabilityKey = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, key);
        assert!(
            serde_json::from_str::<CapabilityKey>("\"browser input\"").is_err(),
            "invalid keys are rejected on read, not ignored"
        );
    }
}
