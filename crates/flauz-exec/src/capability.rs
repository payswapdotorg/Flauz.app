//! Capability identity and advertisement (kernel §2, §7).
//!
//! A capability is identified by a **namespaced string key**, not a ULID
//! entity: [`CapabilityId`] with the frozen grammar
//! `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*` (for example `terminal`,
//! `browser.input`). Every [`Environment`](crate::Environment),
//! [`ModelProvider`](crate::ModelProvider) and
//! [`AgentRuntime`](crate::AgentRuntime) advertises the capabilities it
//! offers; a [`Skill`](crate::Skill) expresses the capabilities it
//! *requires*. Availability is the intersection
//! `model ∩ runtime ∩ environment ∩ permissions ∩ policy`; computing that
//! intersection is future resolution work — this crate only represents
//! advertisements and requirements.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    ContractVersion, ExecError, MAX_NAME_BYTES, MAX_STATEMENT_BYTES, ensure_non_empty,
    ensure_str_bound,
};

/// A namespaced capability key (kernel §2): `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*`,
/// for example `terminal` or `browser.input`. Capability identity is a
/// string key, never a ULID entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CapabilityId(String);

impl CapabilityId {
    /// Parses and validates a capability key.
    pub fn parse(value: &str) -> Result<Self, ExecError> {
        if value.is_empty() {
            return Err(ExecError::invalid("capability key must not be empty"));
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
                return Err(ExecError::invalid(format!(
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
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for CapabilityId {
    type Err = ExecError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for CapabilityId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CapabilityId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// The representative capability vocabulary of the frozen architecture
/// (FLAUZ-SOURCE-OF-TRUTH "Skills and capabilities"). The grammar is frozen;
/// the vocabulary is representative, not exhaustive — new keys parse
/// freely.
pub mod capability_keys {
    /// Shell execution.
    pub const TERMINAL: &str = "terminal";
    /// Read access to a filesystem.
    pub const FILESYSTEM_READ: &str = "filesystem.read";
    /// Write access to a filesystem.
    pub const FILESYSTEM_WRITE: &str = "filesystem.write";
    /// Git repository operations.
    pub const GIT: &str = "git";
    /// Browser automation.
    pub const BROWSER: &str = "browser";
    /// Browser navigation.
    pub const BROWSER_NAVIGATION: &str = "browser.navigation";
    /// Browser input (forms, clicks, typing).
    pub const BROWSER_INPUT: &str = "browser.input";
    /// Computer screen access.
    pub const COMPUTER_SCREEN: &str = "computer.screen";
    /// Keyboard control.
    pub const KEYBOARD: &str = "keyboard";
    /// Mouse control.
    pub const MOUSE: &str = "mouse";
    /// Window management.
    pub const WINDOW: &str = "window";
    /// Desktop GUI control.
    pub const DESKTOP_GUI: &str = "desktop.gui";
    /// Image understanding.
    pub const VISION: &str = "vision";
    /// Image input to a model.
    pub const IMAGE_INPUT: &str = "image.input";
    /// Image output from a model.
    pub const IMAGE_OUTPUT: &str = "image.output";
    /// Web search.
    pub const WEB_SEARCH: &str = "web.search";
    /// Model Context Protocol servers.
    pub const MCP: &str = "mcp";
    /// Exposed network ports.
    pub const PORTS: &str = "ports";
    /// SSH access.
    pub const SSH: &str = "ssh";
    /// Environment snapshots.
    pub const SNAPSHOTS: &str = "snapshots";
    /// Persistent storage across restarts.
    pub const PERSISTENT_STORAGE: &str = "persistent_storage";
    /// Long-running processes.
    pub const LONG_RUNNING: &str = "long_running";
    /// GPU access.
    pub const GPU: &str = "gpu";
}

/// A described capability: a catalog entry pairing a capability key with a
/// human-readable description. Capabilities have no version and no owner
/// beyond their namespaced key (kernel §2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capability {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The namespaced capability key.
    pub id: CapabilityId,
    /// A human-readable description of what the capability offers.
    pub description: Option<String>,
}

impl Capability {
    /// Builds a described capability.
    pub fn new(id: CapabilityId, description: Option<&str>) -> Result<Self, ExecError> {
        id.validate()?;
        if let Some(description) = description {
            ensure_non_empty("capability description", description)?;
            ensure_str_bound("capability description", description, MAX_STATEMENT_BYTES)?;
        }
        Ok(Self {
            v: ContractVersion,
            id,
            description: description.map(str::to_owned),
        })
    }

    /// Validates the capability entry.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(self.id.clone(), self.description.as_deref())?;
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
    fn capability_keys_follow_the_frozen_grammar() {
        assert!(CapabilityId::parse("terminal").is_ok());
        assert!(CapabilityId::parse("browser.input").is_ok());
        assert!(CapabilityId::parse("filesystem.read").is_ok());
        assert!(CapabilityId::parse("computer.screen").is_ok());
        assert!(CapabilityId::parse("persistent_storage").is_ok());
        assert!(CapabilityId::parse("Terminal").is_err());
        assert!(CapabilityId::parse("browser..input").is_err());
        assert!(CapabilityId::parse(".input").is_err());
        assert!(CapabilityId::parse("input.").is_err());
        assert!(CapabilityId::parse("").is_err());
        assert!(serde_json::from_str::<CapabilityId>("\"browser.input\"").is_ok());
        assert!(serde_json::from_str::<CapabilityId>("\"browser input\"").is_err());
    }

    #[test]
    fn representative_vocabulary_parses() {
        for key in [
            capability_keys::TERMINAL,
            capability_keys::FILESYSTEM_READ,
            capability_keys::FILESYSTEM_WRITE,
            capability_keys::GIT,
            capability_keys::BROWSER,
            capability_keys::BROWSER_NAVIGATION,
            capability_keys::BROWSER_INPUT,
            capability_keys::COMPUTER_SCREEN,
            capability_keys::KEYBOARD,
            capability_keys::MOUSE,
            capability_keys::WINDOW,
            capability_keys::DESKTOP_GUI,
            capability_keys::VISION,
            capability_keys::IMAGE_INPUT,
            capability_keys::IMAGE_OUTPUT,
            capability_keys::WEB_SEARCH,
            capability_keys::MCP,
            capability_keys::PORTS,
            capability_keys::SSH,
            capability_keys::SNAPSHOTS,
            capability_keys::PERSISTENT_STORAGE,
            capability_keys::LONG_RUNNING,
            capability_keys::GPU,
        ] {
            assert!(
                ok(CapabilityId::parse(key)).as_str() == key,
                "{key} must parse"
            );
        }
    }

    #[test]
    fn capability_entries_round_trip_and_validate() {
        let capability = ok(Capability::new(
            ok(CapabilityId::parse("browser.input")),
            Some("Drive forms, clicks and typing in a browser"),
        ));
        let serialized = ok(serde_json::to_string(&capability));
        let parsed: Capability = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, capability);
        ok(capability.validate());
        assert!(serde_json::from_str::<Capability>(&serialized.replace("\"v\":1,", "")).is_err());
        assert!(
            Capability::new(ok(CapabilityId::parse("vision")), Some("")).is_err(),
            "empty descriptions are rejected"
        );
    }
}
