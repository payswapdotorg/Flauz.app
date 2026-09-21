//! Procedures: durable, reusable, versioned representations of successful
//! work (CONTEXT-HARNESS-ARCHITECTURE §14, kernel §3).
//!
//! A [`Procedure`] carries a storage `version` (durable entity versioning)
//! plus semantic [`ProcedureVersion`]s. The first version is `1.0`; each
//! saved improvement creates a new `ProcedureVersion` referencing its
//! predecessor, so version lineage is preserved. Procedures are
//! domain-neutral.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ids::{EvidenceId, ProcedureId, ResourceId};
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContractVersion, MAX_NAME_BYTES, MAX_PROCEDURE_VERSIONS, MAX_RELATED_REFS, MAX_STATEMENT_BYTES,
    WorldError, ensure_list_bound, ensure_non_empty, ensure_str_bound,
};

/// A `major.minor` semantic version (both `u32`, patch deliberately absent
/// per kernel §3). The first version of a procedure is `1.0`.
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

    /// The first version of every procedure, `1.0` (kernel §3).
    pub const FIRST: Self = Self::new(1, 0);
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

/// A namespaced capability key (kernel §2): `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*`,
/// for example `terminal` or `browser.input`. Capability keys reference
/// flauz-exec's Capability entity by its frozen identity grammar; they do
/// not define it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CapabilityKey(String);

impl CapabilityKey {
    /// Parses and validates a capability key.
    pub fn parse(value: &str) -> Result<Self, WorldError> {
        if value.is_empty() {
            return Err(WorldError::invalid("capability key must not be empty"));
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
                return Err(WorldError::invalid(format!(
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
    pub fn validate(&self) -> Result<(), WorldError> {
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
    type Err = WorldError;

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

/// A procedure input declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcedureInput {
    /// The input name.
    pub name: String,
    /// The input kind (domain-declared, for example `text` or `file`).
    pub kind: Option<String>,
    /// A human-readable description of the input.
    pub description: Option<String>,
}

/// One step of a procedure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcedureStep {
    /// A short summary of the step.
    pub summary: String,
    /// Optional details for executing the step.
    pub details: Option<String>,
}

/// A resource binding: the role a resource plays in the procedure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBinding {
    /// The role name, for example `target_repository`.
    pub role: String,
    /// The bound resource.
    pub resource_id: ResourceId,
}

/// One semantic version of a procedure: the full reusable content. Each
/// saved improvement creates a new version referencing its predecessor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcedureVersion {
    /// The semantic major version.
    pub major: u32,
    /// The semantic minor version.
    pub minor: u32,
    /// The predecessor version this improvement builds on; `None` only for
    /// the first version.
    pub predecessor: Option<SemanticVersion>,
    /// The objective this version accomplishes.
    pub objective: String,
    /// Preconditions that must hold before execution.
    pub preconditions: Vec<String>,
    /// Input declarations.
    pub inputs: Vec<ProcedureInput>,
    /// The ordered steps.
    pub steps: Vec<ProcedureStep>,
    /// Required capability keys.
    pub required_capabilities: Vec<CapabilityKey>,
    /// Resource bindings by role.
    pub resource_bindings: Vec<ResourceBinding>,
    /// Validation rules (bounded statements).
    pub validation_rules: Vec<String>,
    /// Recovery rules (bounded statements).
    pub recovery_rules: Vec<String>,
    /// Evidence references backing this procedure.
    pub evidence: Vec<EvidenceId>,
    /// The actor that recorded this version.
    pub created_by: ActorRef,
    /// When this version was recorded (caller-supplied).
    pub created_at: Timestamp,
}

impl ProcedureVersion {
    /// The semantic version number of this procedure version.
    #[must_use]
    pub const fn number(&self) -> SemanticVersion {
        SemanticVersion::new(self.major, self.minor)
    }

    /// Validates the procedure version against canonical bounds.
    pub fn validate(&self) -> Result<(), WorldError> {
        if self.major == 0 {
            return Err(WorldError::invalid(
                "procedure semantic versions start at 1.0",
            ));
        }
        ensure_non_empty("procedure objective", &self.objective)?;
        ensure_str_bound("procedure objective", &self.objective, MAX_STATEMENT_BYTES)?;
        check_statement_list("precondition", &self.preconditions)?;
        ensure_list_bound("procedure inputs", &self.inputs, MAX_RELATED_REFS)?;
        ensure_list_bound("procedure steps", &self.steps, MAX_PROCEDURE_VERSIONS)?;
        for step in &self.steps {
            ensure_non_empty("procedure step summary", &step.summary)?;
            ensure_str_bound("procedure step summary", &step.summary, MAX_STATEMENT_BYTES)?;
            if let Some(details) = &step.details {
                ensure_str_bound("procedure step details", details, MAX_STATEMENT_BYTES)?;
            }
        }
        for input in &self.inputs {
            ensure_non_empty("procedure input name", &input.name)?;
            ensure_str_bound("procedure input name", &input.name, MAX_NAME_BYTES)?;
            if let Some(kind) = &input.kind {
                ensure_str_bound("procedure input kind", kind, MAX_NAME_BYTES)?;
            }
            if let Some(description) = &input.description {
                ensure_str_bound(
                    "procedure input description",
                    description,
                    MAX_STATEMENT_BYTES,
                )?;
            }
        }
        ensure_list_bound(
            "required capabilities",
            &self.required_capabilities,
            MAX_RELATED_REFS,
        )?;
        for capability in &self.required_capabilities {
            capability.validate()?;
        }
        ensure_list_bound(
            "resource bindings",
            &self.resource_bindings,
            MAX_RELATED_REFS,
        )?;
        for binding in &self.resource_bindings {
            ensure_non_empty("resource binding role", &binding.role)?;
            ensure_str_bound("resource binding role", &binding.role, MAX_NAME_BYTES)?;
        }
        check_statement_list("validation rule", &self.validation_rules)?;
        check_statement_list("recovery rule", &self.recovery_rules)?;
        ensure_list_bound("procedure evidence", &self.evidence, MAX_RELATED_REFS)?;
        self.created_by.validate()?;
        Ok(())
    }
}

fn check_statement_list(field: &'static str, list: &[String]) -> Result<(), WorldError> {
    ensure_list_bound(field, list, MAX_RELATED_REFS)?;
    for statement in list {
        ensure_non_empty(field, statement)?;
        ensure_str_bound(field, statement, MAX_STATEMENT_BYTES)?;
    }
    Ok(())
}

/// A durable, reusable, versioned representation of a successful way to
/// accomplish an objective.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Procedure {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical procedure ID (`proc_<ULID>`).
    pub id: ProcedureId,
    /// The storage version of the procedure entity, starting at 1; each
    /// saved improvement (a new [`ProcedureVersion`]) is a durable mutation.
    pub version: u64,
    /// Human-readable procedure name.
    pub name: String,
    /// The recorded semantic versions, in creation order. The first is
    /// always `1.0`; each later version references its predecessor.
    pub versions: Vec<ProcedureVersion>,
    /// The actor that created the procedure.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Procedure {
    /// Builds a new procedure whose first semantic version is `1.0`.
    pub fn new(
        id: ProcedureId,
        name: &str,
        first: ProcedureVersion,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, WorldError> {
        ensure_non_empty("procedure name", name)?;
        ensure_str_bound("procedure name", name, MAX_NAME_BYTES)?;
        first.validate()?;
        if first.number() != SemanticVersion::FIRST {
            return Err(WorldError::invalid(
                "the first procedure version must be 1.0",
            ));
        }
        if first.predecessor.is_some() {
            return Err(WorldError::invalid(
                "the first procedure version must not reference a predecessor",
            ));
        }
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            name: name.to_owned(),
            versions: vec![first],
            created_by,
            created_at,
        })
    }

    /// Records an improved version. The new version must reference an
    /// existing predecessor and carry a strictly greater, previously unused
    /// semantic version. This does not bump the storage `version`: durable
    /// version increments happen exactly once per stored mutation (the
    /// store performs them, kernel §3).
    pub fn add_version(&mut self, next: ProcedureVersion) -> Result<(), WorldError> {
        next.validate()?;
        ensure_list_bound("procedure versions", &self.versions, MAX_PROCEDURE_VERSIONS)?;
        if self.versions.len() >= MAX_PROCEDURE_VERSIONS {
            return Err(WorldError::invalid(format!(
                "procedure exceeds {MAX_PROCEDURE_VERSIONS} versions"
            )));
        }
        let predecessor = next.predecessor.ok_or_else(|| {
            WorldError::invalid("procedure version must reference its predecessor")
        })?;
        if !self
            .versions
            .iter()
            .any(|version| version.number() == predecessor)
        {
            return Err(WorldError::invalid(format!(
                "procedure version predecessor {predecessor} does not exist"
            )));
        }
        if next.number() <= predecessor {
            return Err(WorldError::invalid(
                "procedure version must be greater than its predecessor",
            ));
        }
        if self
            .versions
            .iter()
            .any(|version| version.number() == next.number())
        {
            return Err(WorldError::invalid(format!(
                "procedure version {} already exists",
                next.number()
            )));
        }
        self.versions.push(next);
        Ok(())
    }

    /// Returns the current (latest) procedure version.
    #[must_use]
    pub fn current(&self) -> Option<&ProcedureVersion> {
        self.versions.last()
    }

    /// Validates the procedure, including its version lineage.
    pub fn validate(&self) -> Result<(), WorldError> {
        ensure_non_empty("procedure name", &self.name)?;
        ensure_str_bound("procedure name", &self.name, MAX_NAME_BYTES)?;
        ensure_list_bound("procedure versions", &self.versions, MAX_PROCEDURE_VERSIONS)?;
        if self.versions.is_empty() {
            return Err(WorldError::invalid("procedure has no versions"));
        }
        for (index, version) in self.versions.iter().enumerate() {
            version.validate()?;
            if index == 0 {
                if version.number() != SemanticVersion::FIRST || version.predecessor.is_some() {
                    return Err(WorldError::invalid(
                        "the first procedure version must be 1.0 without a predecessor",
                    ));
                }
            } else {
                let predecessor = version.predecessor.ok_or_else(|| {
                    WorldError::invalid("procedure version must reference its predecessor")
                })?;
                let previous = &self.versions[index - 1];
                if predecessor != previous.number() {
                    return Err(WorldError::invalid(
                        "procedure version lineage must reference the previous version",
                    ));
                }
                if version.number() <= previous.number() {
                    return Err(WorldError::invalid(
                        "procedure versions must be strictly increasing",
                    ));
                }
            }
        }
        if self.version == 0 {
            return Err(WorldError::invalid("procedure version must be at least 1"));
        }
        self.created_by.validate()?;
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

    /// Builds a minimal valid first procedure version.
    fn test_first_version() -> ProcedureVersion {
        ProcedureVersion {
            major: 1,
            minor: 0,
            predecessor: None,
            objective: "Reconcile the weekly finance spreadsheet".to_owned(),
            preconditions: vec!["Spreadsheet is shared with the agent".to_owned()],
            inputs: vec![ProcedureInput {
                name: "week".to_owned(),
                kind: Some("text".to_owned()),
                description: Some("ISO week to reconcile".to_owned()),
            }],
            steps: vec![ProcedureStep {
                summary: "Export ledger deltas".to_owned(),
                details: Some("Use the finance API export".to_owned()),
            }],
            required_capabilities: vec![
                ok(CapabilityKey::parse("terminal")),
                ok(CapabilityKey::parse("browser.input")),
            ],
            resource_bindings: vec![ResourceBinding {
                role: "ledger".to_owned(),
                resource_id: ResourceId::generate(),
            }],
            validation_rules: vec!["Totals must match to the cent".to_owned()],
            recovery_rules: vec!["Re-export and retry once".to_owned()],
            evidence: Vec::new(),
            created_by: ok(ActorRef::user("alice")),
            created_at: ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        }
    }

    #[test]
    fn procedure_version_lineage_is_enforced() {
        let mut procedure = ok(Procedure::new(
            ProcedureId::generate(),
            "Weekly reconciliation",
            test_first_version(),
            ok(ActorRef::user("alice")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        ));
        assert_eq!(procedure.version, 1);
        assert_eq!(
            procedure.current().map(|v| v.number()),
            Some(SemanticVersion::FIRST)
        );

        // An improvement that skips the predecessor reference is rejected.
        let orphan = ProcedureVersion {
            major: 1,
            minor: 1,
            predecessor: None,
            ..test_first_version()
        };
        assert!(procedure.add_version(orphan).is_err());

        // A version that does not advance is rejected.
        let stale = ProcedureVersion {
            major: 1,
            minor: 0,
            predecessor: Some(SemanticVersion::FIRST),
            ..test_first_version()
        };
        assert!(procedure.add_version(stale).is_err());

        // A valid improvement is accepted, keeps the lineage and leaves the
        // storage version alone (the store increments it per durable
        // mutation).
        let improved = ProcedureVersion {
            major: 1,
            minor: 1,
            predecessor: Some(SemanticVersion::FIRST),
            objective: "Reconcile the weekly finance spreadsheet faster".to_owned(),
            ..test_first_version()
        };
        ok(procedure.add_version(improved));
        assert_eq!(procedure.version, 1);
        assert_eq!(procedure.versions.len(), 2);
        ok(procedure.validate());
    }

    #[test]
    fn capability_keys_follow_the_frozen_grammar() {
        assert!(CapabilityKey::parse("terminal").is_ok());
        assert!(CapabilityKey::parse("browser.input").is_ok());
        assert!(CapabilityKey::parse("filesystem.read").is_ok());
        assert!(CapabilityKey::parse("Terminal").is_err());
        assert!(CapabilityKey::parse("browser..input").is_err());
        assert!(CapabilityKey::parse(".input").is_err());
        assert!(CapabilityKey::parse("input.").is_err());
        assert!(CapabilityKey::parse("").is_err());
        assert!(serde_json::from_str::<CapabilityKey>("\"browser.input\"").is_ok());
        assert!(serde_json::from_str::<CapabilityKey>("\"browser input\"").is_err());
    }
}
