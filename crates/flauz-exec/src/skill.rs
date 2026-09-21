//! The [`Skill`] entity: reusable, model-agnostic behavior packages with
//! explicit capability requirements (constitution: "Skills are reusable,
//! model-agnostic behavior packages with explicit capability
//! requirements").
//!
//! - **Identity** (kernel §2): a [`SkillId`] is a namespaced string key —
//!   the capability-key grammar with **at least one dot** (for example
//!   `flauz.research.collect`) — never a ULID entity.
//! - **Version** (kernel §2/§3): a skill carries a semantic version
//!   (`major.minor`, both `u32`, from `1.0`). Skill identity is not a
//!   ULID, so the semantic version is the skill's version; there is no
//!   separate storage `u64`.
//! - **Requirements**: `required_capabilities` is *requirement
//!   representation only* — resolving requirements against model/runtime/
//!   environment advertisements (the availability intersection) is future
//!   work, deliberately not this crate's.
//! - **Neutrality**: skill semantics stay provider- and model-neutral —
//!   the type carries no provider kind, no runtime kind, no model and no
//!   environment reference, and nothing about a skill changes when models
//!   or providers change.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::capability::CapabilityId;
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContractVersion, ExecError, MAX_NAME_BYTES, MAX_STATEMENT_BYTES, ensure_capability_list,
    ensure_non_empty, ensure_str_bound,
};

/// A `major.minor` semantic version (kernel §3 style: both `u32`, patch
/// deliberately absent). The first version of a skill is `1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticVersion {
    /// Major version.
    pub major: u32,
    /// Minor version.
    pub minor: u32,
}

impl SemanticVersion {
    /// Builds a semantic version.
    #[must_use]
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// The first version of every skill, `1.0` (kernel §3).
    pub const FIRST: Self = Self::new(1, 0);
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

/// A [`Skill`] identity: the capability-key grammar with at least
/// one dot, for example `flauz.research.collect` (kernel §2). Skill
/// identity is a namespaced string key, never a ULID entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SkillId(String);

impl SkillId {
    /// Parses and validates a skill key: segments of
    /// `[a-z][a-z0-9_]*` separated by dots, with at least one dot.
    pub fn parse(value: &str) -> Result<Self, ExecError> {
        if value.is_empty() {
            return Err(ExecError::invalid("skill key must not be empty"));
        }
        if !value.contains('.') {
            return Err(ExecError::invalid(format!(
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
                return Err(ExecError::invalid(format!(
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
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for SkillId {
    type Err = ExecError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for SkillId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SkillId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A skill: a reusable, model-agnostic behavior package expressing the
/// capabilities it **requires**.
///
/// Requirement representation only: whether a given model/runtime/
/// environment satisfies `required_capabilities` is future resolution
/// work. The type is deliberately provider-, model-, runtime- and
/// environment-neutral.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The namespaced skill key (at least one dot).
    pub id: SkillId,
    /// The semantic version of this skill (`major.minor`, from `1.0`).
    pub semantic_version: SemanticVersion,
    /// Human-readable skill name.
    pub name: String,
    /// A human-readable description of what the skill does.
    pub description: Option<String>,
    /// The capabilities this skill requires (sorted, deduplicated).
    pub required_capabilities: Vec<CapabilityId>,
    /// The actor that recorded this skill.
    pub created_by: ActorRef,
    /// Recording timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Skill {
    /// Builds a new skill. No provider, model, runtime or environment is
    /// required or accepted — a skill is neutral by construction.
    pub fn new(
        id: SkillId,
        semantic_version: SemanticVersion,
        name: &str,
        description: Option<&str>,
        required_capabilities: Vec<CapabilityId>,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ExecError> {
        id.validate()?;
        if semantic_version.major == 0 {
            return Err(ExecError::invalid("skill semantic versions start at 1.0"));
        }
        ensure_non_empty("skill name", name)?;
        ensure_str_bound("skill name", name, MAX_NAME_BYTES)?;
        if let Some(description) = description {
            ensure_non_empty("skill description", description)?;
            ensure_str_bound("skill description", description, MAX_STATEMENT_BYTES)?;
        }
        ensure_capability_list("required capabilities", &required_capabilities)?;
        Ok(Self {
            v: ContractVersion,
            id,
            semantic_version,
            name: name.to_owned(),
            description: description.map(str::to_owned),
            required_capabilities,
            created_by,
            created_at,
        })
    }

    /// Validates the skill record.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(
            self.id.clone(),
            self.semantic_version,
            &self.name,
            self.description.as_deref(),
            self.required_capabilities.clone(),
            self.created_by.clone(),
            self.created_at,
        )?;
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

    fn test_actor() -> ActorRef {
        ok(ActorRef::user("alice"))
    }

    fn test_timestamp() -> Timestamp {
        ok(Timestamp::from_unix_seconds(1_789_998_300))
    }

    #[test]
    fn skill_keys_require_at_least_one_dot() {
        assert!(SkillId::parse("flauz.research.collect").is_ok());
        assert!(SkillId::parse("vendor.skill").is_ok());
        assert!(SkillId::parse("terminal").is_err());
        assert!(SkillId::parse("flauz..collect").is_err());
        assert!(SkillId::parse("Flauz.collect").is_err());
        assert!(SkillId::parse("flauz.collect.").is_err());
        assert!(SkillId::parse("").is_err());
        assert!(serde_json::from_str::<SkillId>("\"flauz.research.collect\"").is_ok());
        assert!(serde_json::from_str::<SkillId>("\"flauz\"").is_err());
    }

    #[test]
    fn skill_round_trips_with_requirements_only() {
        let skill = ok(Skill::new(
            ok(SkillId::parse("flauz.research.collect")),
            SemanticVersion::FIRST,
            "Collect research",
            Some("Collect and reconcile evidence from web sources"),
            vec![
                ok(CapabilityId::parse("browser.input")),
                ok(CapabilityId::parse("web.search")),
            ],
            test_actor(),
            test_timestamp(),
        ));
        let serialized = ok(serde_json::to_string(&skill));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,\"id\":\"flauz.research.collect\",",
                "\"semantic_version\":{\"major\":1,\"minor\":0},",
                "\"name\":\"Collect research\",",
                "\"description\":\"Collect and reconcile evidence from web sources\",",
                "\"required_capabilities\":[\"browser.input\",\"web.search\"],",
                "\"created_by\":{\"kind\":\"user\",\"id\":\"alice\"},",
                "\"created_at\":\"2026-09-21T13:45:00Z\"}"
            )
        );
        let parsed: Skill = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, skill);
        ok(skill.validate());
    }

    #[test]
    fn skill_versions_start_at_one_zero() {
        assert!(
            Skill::new(
                ok(SkillId::parse("flauz.research.collect")),
                SemanticVersion::new(0, 1),
                "Collect research",
                None,
                vec![],
                test_actor(),
                test_timestamp()
            )
            .is_err()
        );
        assert_eq!(SemanticVersion::FIRST.to_string(), "1.0");
    }
}
