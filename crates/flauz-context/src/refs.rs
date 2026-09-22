//! Actor references and skill references (kernel §2).
//!
//! Actors are `{"kind": "user|agent|system|provider", "id": "<canonical
//! entity id or principal string>"}` — the frozen cross-crate actor format.
//! This is a local validating type over the frozen format, not a dependency
//! on the sibling crates: the serialized shape is identical on the wire.
//!
//! A [`SkillRef`] is a validated reference to a foreign Skill identity: the
//! namespaced string-key grammar with at least one dot (for example
//! `flauz.research.collect`). Skill identity is a namespaced key owned by
//! flauz-exec; this crate references it without defining the Skill entity.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{ContextError, MAX_ACTOR_ID_BYTES, MAX_NAME_BYTES, ensure_non_empty, ensure_str_bound};

/// The kind of an actor attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    /// A human user principal.
    User,
    /// An agent (referenced by its `agent_` canonical ID).
    Agent,
    /// A Flauz system component.
    System,
    /// A provider (referenced by its canonical ID or principal string).
    Provider,
}

/// Attribution to the actor that produced a record. Every context decision
/// (compilation, reset, memory mutation) stays attributable through actor
/// references.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorRef {
    /// The actor kind.
    pub kind: ActorKind,
    /// The canonical entity ID or principal string of the actor.
    pub id: String,
}

impl ActorRef {
    /// Builds an actor reference, validating that the id is a non-empty,
    /// bounded string.
    pub fn new(kind: ActorKind, id: &str) -> Result<Self, ContextError> {
        ensure_non_empty("actor id", id)?;
        ensure_str_bound("actor id", id, MAX_ACTOR_ID_BYTES)?;
        Ok(Self {
            kind,
            id: id.to_owned(),
        })
    }

    /// Builds a `user` actor reference.
    pub fn user(id: &str) -> Result<Self, ContextError> {
        Self::new(ActorKind::User, id)
    }

    /// Builds an `agent` actor reference.
    pub fn agent(id: &str) -> Result<Self, ContextError> {
        Self::new(ActorKind::Agent, id)
    }

    /// Builds a `system` actor reference.
    pub fn system(id: &str) -> Result<Self, ContextError> {
        Self::new(ActorKind::System, id)
    }

    /// Builds a `provider` actor reference.
    pub fn provider(id: &str) -> Result<Self, ContextError> {
        Self::new(ActorKind::Provider, id)
    }

    /// Validates the actor reference bounds.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.kind, &self.id)?;
        Ok(())
    }
}

/// A validated reference to a foreign Skill identity: the capability-key
/// grammar with at least one dot, for example `flauz.research.collect`
/// (kernel §2). Skill identity is a namespaced string key owned by
/// flauz-exec; this type references it without owning it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SkillRef(String);

impl SkillRef {
    /// Parses and validates a skill key: segments of `[a-z][a-z0-9_]*`
    /// separated by dots, with at least one dot.
    pub fn parse(value: &str) -> Result<Self, ContextError> {
        if value.is_empty() {
            return Err(ContextError::invalid("skill key must not be empty"));
        }
        if !value.contains('.') {
            return Err(ContextError::invalid(format!(
                "skill key {value:?} must contain at least one `.` \
                 (namespaced, for example `flauz.research.collect`)"
            )));
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
                return Err(ContextError::invalid(format!(
                    "skill key {value:?} segments must be `[a-z][a-z0-9_]*`"
                )));
            }
        }
        ensure_str_bound("skill key", value, MAX_NAME_BYTES)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the skill key string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the skill key.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

impl fmt::Display for SkillRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for SkillRef {
    type Err = ContextError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for SkillRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SkillRef {
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
    fn actor_ref_serializes_with_kind_field() {
        let actor = ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD"));
        let serialized = ok(serde_json::to_string(&actor));
        assert_eq!(
            serialized,
            "{\"kind\":\"agent\",\"id\":\"agent_01J8ZQ5V8K3T2B7N6X4R9DQPQD\"}"
        );
        assert!(ActorRef::new(ActorKind::User, "").is_err());
        assert!(serde_json::from_str::<ActorRef>("{\"kind\":\"robot\",\"id\":\"x\"}").is_err());
        assert!(
            serde_json::from_str::<ActorRef>("{\"kind\":\"user\",\"id\":\"x\",\"x\":1}").is_err()
        );
    }

    #[test]
    fn skill_ref_uses_the_frozen_key_grammar() {
        assert!(SkillRef::parse("flauz.research.collect").is_ok());
        assert!(SkillRef::parse("vendor.skill").is_ok());
        assert!(SkillRef::parse("terminal").is_err());
        assert!(SkillRef::parse("flauz..collect").is_err());
        assert!(SkillRef::parse("Flauz.collect").is_err());
        assert!(SkillRef::parse("flauz.collect.").is_err());
        assert!(SkillRef::parse("").is_err());
        let key = ok(SkillRef::parse("flauz.research.collect"));
        let serialized = ok(serde_json::to_string(&key));
        assert_eq!(serialized, "\"flauz.research.collect\"");
        let reloaded: SkillRef = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, key);
    }
}
