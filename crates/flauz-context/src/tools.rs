//! Dynamic tool exposure (Wave 3, work order **ORCH-002**; architecture
//! §11: tool registry and dynamic tool exposure).
//!
//! Given a [`ModelContextProfile`] (its
//! [`ToolSchemaHandling`](crate::ToolSchemaHandling)) plus a capability
//! admission list — the flauz-cap `CapabilityResolution` record shape
//! taken as **data** — [`compute_tool_exposure`] computes the tool
//! schemas a model sees.
//!
//! # Exclusions are NAMED, never silently dropped
//!
//! A tool is exposed iff the capability it requires resolves as
//! available. Every tool that is NOT exposed is **named** in the
//! exposure's exclusion list with the honest reason: the named gap
//! reasons of its capability resolution, or — when no resolution is
//! recorded at all — an explicit "no capability resolution is recorded"
//! reason. Silent fall-through (a tool quietly missing from the offered
//! set) is a contract violation, the same law the capability kernel
//! addendum §4 states for capabilities.
//!
//! # Schema-format conversion per profile
//!
//! - [`ToolSchemaHandling::InlineFullSchemas`] → every exposed tool
//!   carries its full schema document inlined
//!   ([`ToolPresentation::Full`]);
//! - [`ToolSchemaHandling::SummariesWithLazySchemas`] → every exposed
//!   tool carries its one-line summary; full schemas load lazily
//!   ([`ToolPresentation::Summary`]);
//! - [`ToolSchemaHandling::LazyPerTool`] → exposed tools are listed by
//!   name only; schemas load lazily one tool at a time
//!   ([`ToolPresentation::Lazy`]).
//!
//! # Inputs as data (the CAP-001 frozen-format pattern)
//!
//! This crate does NOT import `flauz-cap`: the capability-admission
//! records cross the seam as **frozen format strings**. The frozen
//! capability-key grammar `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*` and the
//! five frozen admission-dimension names (`model`, `runtime`,
//! `environment`, `permissions`, `workspace_policy`) are re-validated
//! locally ([`validate_capability_key`],
//! [`CAPABILITY_DIMENSIONS`]) — byte-identical in meaning on the wire,
//! never re-defined entity types. The unlock paths of the resolution
//! record are not needed to compute exposure and are not mirrored.
//!
//! # Determinism (kernel §7)
//!
//! The exposure is a pure function of its inputs: no wall clock, no
//! randomness, no I/O. Output lists are sorted by tool name, so the
//! exposure is independent of the order the caller lists tools or
//! admissions in.

use serde::{Deserialize, Serialize};

use crate::ids::ModelRef;
use crate::profile::{ModelContextProfile, ToolSchemaHandling};
use crate::{
    ContextError, ContextVersion, MAX_NAME_BYTES, MAX_STATEMENT_BYTES, ensure_non_empty,
    ensure_str_bound,
};

/// Maximum number of tool definitions in one exposure (the bounded tool
/// surface of architecture §11).
pub const MAX_TOOL_DEFINITIONS: usize = 128;

/// Maximum length of one inlined tool schema document.
pub const MAX_TOOL_SCHEMA_BYTES: usize = 16 * 1024;

/// Maximum length of one tool summary line.
pub const MAX_TOOL_SUMMARY_BYTES: usize = 512;

/// Maximum length of one admission gap reason (the flauz-cap bound,
/// re-pinned).
pub const MAX_EXPLANATION_BYTES: usize = 512;

/// The frozen admission-dimension names of the capability intersection
/// (re-pinned from the resolution record's shape; owned by `flauz-cap`'s
/// `Dimension` vocabulary): model, runtime, environment, permissions,
/// workspace policy.
pub const CAPABILITY_DIMENSIONS: [&str; 5] = [
    "model",
    "runtime",
    "environment",
    "permissions",
    "workspace_policy",
];

/// Re-validates the frozen capability-key grammar locally: segments of
/// `[a-z][a-z0-9_]*` separated by dots (for example `terminal` or
/// `browser.input`). The grammar is owned by `flauz-exec`'s
/// `CapabilityId`; this is the frozen-format re-pin, never a
/// re-definition.
///
/// # Errors
///
/// Returns [`ContextError::Invalid`] when the key violates the frozen
/// grammar or length bound.
pub fn validate_capability_key(key: &str) -> Result<(), ContextError> {
    if key.is_empty() {
        return Err(ContextError::invalid("capability key must not be empty"));
    }
    for segment in key.split('.') {
        let mut characters = segment.chars();
        let valid = characters
            .next()
            .is_some_and(|first| first.is_ascii_lowercase())
            && characters.all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
            });
        if !valid {
            return Err(ContextError::invalid(format!(
                "capability key {key:?} segments must be `[a-z][a-z0-9_]*`"
            )));
        }
    }
    ensure_str_bound("capability key", key, MAX_NAME_BYTES)?;
    Ok(())
}

/// One tool offered to a model, as call-side data: its name, the
/// capability that admits it, its full bounded schema document, and its
/// one-line summary. Input record family (like the capability
/// resolver's inputs), carried across seams as canonical JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDefinition {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// The tool's name (unique within one exposure computation).
    pub name: String,
    /// The capability key that admits this tool (the frozen grammar).
    pub capability: String,
    /// The full tool schema document (a bounded JSON document carried as
    /// a string).
    pub schema: String,
    /// The one-line summary inlined when the profile uses
    /// summaries-with-lazy-schemas.
    pub summary: String,
}

impl ToolDefinition {
    /// Builds a tool definition, validating every field against the
    /// frozen grammars and bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the name, capability,
    /// schema or summary fails canonical validation.
    pub fn new(
        name: impl Into<String>,
        capability: impl Into<String>,
        schema: impl Into<String>,
        summary: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let name = name.into();
        let capability = capability.into();
        let schema = schema.into();
        let summary = summary.into();
        ensure_non_empty("tool name", &name)?;
        ensure_str_bound("tool name", &name, MAX_NAME_BYTES)?;
        validate_capability_key(&capability)?;
        ensure_non_empty("tool schema", &schema)?;
        ensure_str_bound("tool schema", &schema, MAX_TOOL_SCHEMA_BYTES)?;
        ensure_non_empty("tool summary", &summary)?;
        ensure_str_bound("tool summary", &summary, MAX_TOOL_SUMMARY_BYTES)?;
        Ok(Self {
            v: ContextVersion,
            name,
            capability,
            schema,
            summary,
        })
    }

    /// Validates the tool definition.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.name.clone(),
            self.capability.clone(),
            self.schema.clone(),
            self.summary.clone(),
        )?;
        Ok(())
    }
}

/// One named gap of a capability admission, as data — the frozen
/// `NamedGap` shape (dimension + honest reason) crossed as format
/// strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionGap {
    /// The admission dimension that does not admit the capability (one
    /// of [`CAPABILITY_DIMENSIONS`]).
    pub dimension: String,
    /// The honest, human-readable reason the dimension does not admit it.
    pub reason: String,
}

impl AdmissionGap {
    /// Builds one admission gap, validating the dimension name against
    /// the frozen five and the reason's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the dimension is not one
    /// of the frozen names, or the reason is empty or over the
    /// explanation bound.
    pub fn new(
        dimension: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let dimension = dimension.into();
        let reason = reason.into();
        if !CAPABILITY_DIMENSIONS.contains(&dimension.as_str()) {
            return Err(ContextError::invalid(format!(
                "admission gap dimension {dimension:?} is not one of the frozen dimensions \
                 {CAPABILITY_DIMENSIONS:?}"
            )));
        }
        ensure_non_empty("admission gap reason", &reason)?;
        ensure_str_bound("admission gap reason", &reason, MAX_EXPLANATION_BYTES)?;
        Ok(Self { dimension, reason })
    }

    /// Validates the admission gap.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.dimension.clone(), self.reason.clone())?;
        Ok(())
    }
}

/// One capability admission decision, as data — the frozen
/// `CapabilityResolution` record shape (capability, availability, the
/// named gap list) crossed as format strings. The unlock paths are not
/// needed to compute tool exposure and are not mirrored.
///
/// The no-silent-fall-through law is re-pinned locally: the gap list is
/// empty **iff** the capability is available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityAdmission {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// The capability key (the frozen grammar, re-validated locally).
    pub capability: String,
    /// Whether every admission dimension admits the capability.
    pub available: bool,
    /// The named gaps — exactly one entry per missing dimension when the
    /// capability is unavailable; empty when it is available.
    pub gaps: Vec<AdmissionGap>,
}

impl CapabilityAdmission {
    /// Builds one capability admission, validating the capability key,
    /// every gap, and the availability/gaps consistency.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the capability key or a
    /// gap fails validation, a dimension repeats, or the gap list
    /// disagrees with `available` (the silent-fall-through shape).
    pub fn new(
        capability: impl Into<String>,
        available: bool,
        gaps: Vec<AdmissionGap>,
    ) -> Result<Self, ContextError> {
        let capability = capability.into();
        validate_capability_key(&capability)?;
        for gap in &gaps {
            gap.validate()?;
        }
        let mut dimensions: Vec<&str> = gaps.iter().map(|gap| gap.dimension.as_str()).collect();
        dimensions.sort_unstable();
        for pair in dimensions.windows(2) {
            if pair[0] == pair[1] {
                return Err(ContextError::invalid(format!(
                    "admission for {capability:?} repeats dimension {}",
                    pair[0]
                )));
            }
        }
        if available != gaps.is_empty() {
            return Err(ContextError::invalid(format!(
                "admission for {capability:?} must be available iff it carries no named gaps"
            )));
        }
        Ok(Self {
            v: ContextVersion,
            capability,
            available,
            gaps,
        })
    }

    /// Validates the admission.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(self.capability.clone(), self.available, self.gaps.clone())?;
        Ok(())
    }
}

/// How one exposed tool's schema is presented to the model, per the
/// profile's [`ToolSchemaHandling`]. Serialized as an internally
/// `kind`-tagged, strictly-read map (the `MemoryContent` pattern: the
/// serde derive cannot combine internal tagging with
/// `deny_unknown_fields`, so the strict canonical read is hand-written).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolPresentation {
    /// The full schema document is inlined
    /// (`inline_full_schemas`).
    Full {
        /// The full schema document.
        schema: String,
    },
    /// A one-line summary is inlined; the full schema loads lazily
    /// (`summaries_with_lazy_schemas`).
    Summary {
        /// The one-line summary.
        summary: String,
    },
    /// The tool is listed by name only; its schema loads lazily, one
    /// tool at a time (`lazy_per_tool`).
    Lazy,
}

impl ToolPresentation {
    /// Validates the presentation's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when an inlined schema or
    /// summary is empty or over its bound.
    pub fn validate(&self) -> Result<(), ContextError> {
        match self {
            Self::Full { schema } => {
                ensure_non_empty("tool schema", schema)?;
                ensure_str_bound("tool schema", schema, MAX_TOOL_SCHEMA_BYTES)
            }
            Self::Summary { summary } => {
                ensure_non_empty("tool summary", summary)?;
                ensure_str_bound("tool summary", summary, MAX_TOOL_SUMMARY_BYTES)
            }
            Self::Lazy => Ok(()),
        }
    }
}

impl Serialize for ToolPresentation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            Self::Full { schema } => {
                map.serialize_entry("kind", "full")?;
                map.serialize_entry("schema", schema)?;
            }
            Self::Summary { summary } => {
                map.serialize_entry("kind", "summary")?;
                map.serialize_entry("summary", summary)?;
            }
            Self::Lazy => {
                map.serialize_entry("kind", "lazy")?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for ToolPresentation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ToolPresentationVisitor;

        impl<'de> serde::de::Visitor<'de> for ToolPresentationVisitor {
            type Value = ToolPresentation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("tool presentation tagged with `kind`")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                use serde::de::Error;

                let mut kind: Option<String> = None;
                let mut schema: Option<String> = None;
                let mut summary: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "kind" => {
                            if kind.replace(map.next_value()?).is_some() {
                                return Err(A::Error::duplicate_field("kind"));
                            }
                        }
                        "schema" => {
                            if schema.replace(map.next_value()?).is_some() {
                                return Err(A::Error::duplicate_field("schema"));
                            }
                        }
                        "summary" => {
                            if summary.replace(map.next_value()?).is_some() {
                                return Err(A::Error::duplicate_field("summary"));
                            }
                        }
                        other => {
                            return Err(A::Error::unknown_field(
                                other,
                                &["kind", "schema", "summary"],
                            ));
                        }
                    }
                }

                match kind.as_deref() {
                    Some("full") => {
                        let schema = schema.ok_or_else(|| A::Error::missing_field("schema"))?;
                        if summary.is_some() {
                            return Err(A::Error::custom(
                                "full tool presentation must not carry a summary field",
                            ));
                        }
                        Ok(ToolPresentation::Full { schema })
                    }
                    Some("summary") => {
                        let summary = summary.ok_or_else(|| A::Error::missing_field("summary"))?;
                        if schema.is_some() {
                            return Err(A::Error::custom(
                                "summary tool presentation must not carry a schema field",
                            ));
                        }
                        Ok(ToolPresentation::Summary { summary })
                    }
                    Some("lazy") => {
                        if schema.is_some() || summary.is_some() {
                            return Err(A::Error::custom(
                                "lazy tool presentation carries neither schema nor summary",
                            ));
                        }
                        Ok(ToolPresentation::Lazy)
                    }
                    Some(other) => Err(A::Error::unknown_variant(
                        other,
                        &["full", "summary", "lazy"],
                    )),
                    None => Err(A::Error::missing_field("kind")),
                }
            }
        }

        deserializer.deserialize_map(ToolPresentationVisitor)
    }
}

/// One tool the model sees, in the profile's schema format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExposedTool {
    /// The tool's name.
    pub name: String,
    /// The capability that admitted the tool.
    pub capability: String,
    /// The schema presentation per the profile's tool-schema handling.
    pub presentation: ToolPresentation,
}

impl ExposedTool {
    /// Builds one exposed-tool entry, validating the bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the name, capability or
    /// presentation fails canonical validation.
    pub fn new(
        name: impl Into<String>,
        capability: impl Into<String>,
        presentation: ToolPresentation,
    ) -> Result<Self, ContextError> {
        let name = name.into();
        let capability = capability.into();
        ensure_non_empty("tool name", &name)?;
        ensure_str_bound("tool name", &name, MAX_NAME_BYTES)?;
        validate_capability_key(&capability)?;
        presentation.validate()?;
        Ok(Self {
            name,
            capability,
            presentation,
        })
    }

    /// Validates the entry.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.name.clone(),
            self.capability.clone(),
            self.presentation.clone(),
        )?;
        Ok(())
    }
}

/// One tool the model does NOT see, **named** with the honest reason:
/// the named gap reasons of its capability resolution, or the explicit
/// no-resolution reason. Never silently dropped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcludedTool {
    /// The tool's name.
    pub name: String,
    /// The capability that was not admitted (or not resolved).
    pub capability: String,
    /// The honest reason the tool is excluded.
    pub reason: String,
}

impl ExcludedTool {
    /// Builds one excluded-tool entry, validating the bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the name, capability or
    /// reason fails canonical validation.
    pub fn new(
        name: impl Into<String>,
        capability: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let name = name.into();
        let capability = capability.into();
        let reason = reason.into();
        ensure_non_empty("tool name", &name)?;
        ensure_str_bound("tool name", &name, MAX_NAME_BYTES)?;
        validate_capability_key(&capability)?;
        ensure_non_empty("excluded tool reason", &reason)?;
        ensure_str_bound("excluded tool reason", &reason, MAX_STATEMENT_BYTES)?;
        Ok(Self {
            name,
            capability,
            reason,
        })
    }

    /// Validates the entry.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.name.clone(),
            self.capability.clone(),
            self.reason.clone(),
        )?;
        Ok(())
    }
}

/// The tool exposure for one model profile: the tools the model sees, in
/// the profile's schema format, and the NAMED exclusions — every input
/// tool appears in exactly one of the two lists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolExposure {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// The model this exposure was computed for (from the profile).
    pub model_id: ModelRef,
    /// The tool-schema handling the profile demands (recorded for
    /// observability).
    pub tool_schema: ToolSchemaHandling,
    /// The exposed tools, sorted by name.
    pub exposed: Vec<ExposedTool>,
    /// The excluded tools, sorted by name, each with its honest reason.
    pub excluded: Vec<ExcludedTool>,
}

impl ToolExposure {
    /// Builds one tool exposure, validating every entry, the list
    /// bounds, the per-list name uniqueness and the disjointness of the
    /// exposed and excluded name sets.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any entry or shape rule
    /// fails.
    pub fn new(
        model_id: ModelRef,
        tool_schema: ToolSchemaHandling,
        exposed: Vec<ExposedTool>,
        excluded: Vec<ExcludedTool>,
    ) -> Result<Self, ContextError> {
        if exposed.len() + excluded.len() > MAX_TOOL_DEFINITIONS {
            return Err(ContextError::invalid(format!(
                "tool exposure exceeds {MAX_TOOL_DEFINITIONS} tools"
            )));
        }
        for tool in &exposed {
            tool.validate()?;
        }
        for tool in &excluded {
            tool.validate()?;
        }
        let exposed_names: Vec<&str> = exposed.iter().map(|tool| tool.name.as_str()).collect();
        let excluded_names: Vec<&str> = excluded.iter().map(|tool| tool.name.as_str()).collect();
        let mut all_names = [&exposed_names[..], &excluded_names[..]].concat();
        all_names.sort_unstable();
        for pair in all_names.windows(2) {
            if pair[0] == pair[1] {
                return Err(ContextError::invalid(format!(
                    "tool {} appears in both the exposed and the excluded list",
                    pair[0]
                )));
            }
        }
        Ok(Self {
            v: ContextVersion,
            model_id,
            tool_schema,
            exposed,
            excluded,
        })
    }

    /// Validates the exposure.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.model_id.clone(),
            self.tool_schema,
            self.exposed.clone(),
            self.excluded.clone(),
        )?;
        Ok(())
    }
}

/// Computes the tool exposure for one model profile: the tools whose
/// capabilities are admitted, presented in the profile's schema format,
/// and the NAMED exclusions for every tool whose capability is not
/// admitted or has no resolution recorded at all. Output lists are
/// sorted by tool name — deterministic and order-independent.
///
/// # Errors
///
/// Returns [`ContextError::Invalid`] when the profile, a tool
/// definition or an admission fails canonical validation, when tool
/// names repeat, or when the same capability carries two admissions.
pub fn compute_tool_exposure(
    profile: &ModelContextProfile,
    tools: &[ToolDefinition],
    admissions: &[CapabilityAdmission],
) -> Result<ToolExposure, ContextError> {
    profile.validate()?;
    if tools.len() > MAX_TOOL_DEFINITIONS {
        return Err(ContextError::invalid(format!(
            "more than {MAX_TOOL_DEFINITIONS} tool definitions"
        )));
    }
    if admissions.len() > MAX_TOOL_DEFINITIONS {
        return Err(ContextError::invalid(format!(
            "more than {MAX_TOOL_DEFINITIONS} capability admissions"
        )));
    }
    for tool in tools {
        tool.validate()?;
    }
    for admission in admissions {
        admission.validate()?;
    }

    // Unique tool names; unique admission capabilities.
    let mut tool_names: Vec<&str> = tools.iter().map(|tool| tool.name.as_str()).collect();
    tool_names.sort_unstable();
    for pair in tool_names.windows(2) {
        if pair[0] == pair[1] {
            return Err(ContextError::invalid(format!(
                "duplicate tool name {}",
                pair[0]
            )));
        }
    }
    let mut capabilities: Vec<&str> = admissions
        .iter()
        .map(|admission| admission.capability.as_str())
        .collect();
    capabilities.sort_unstable();
    for pair in capabilities.windows(2) {
        if pair[0] == pair[1] {
            return Err(ContextError::invalid(format!(
                "duplicate capability admission for {}",
                pair[0]
            )));
        }
    }

    let mut exposed = Vec::new();
    let mut excluded = Vec::new();
    for tool in tools {
        let admission = admissions
            .iter()
            .find(|admission| admission.capability == tool.capability);
        match admission {
            None => {
                excluded.push(ExcludedTool::new(
                    tool.name.clone(),
                    tool.capability.clone(),
                    format!(
                        "no capability resolution is recorded for {}",
                        tool.capability
                    ),
                )?);
            }
            Some(admission) if !admission.available => {
                let reason = admission
                    .gaps
                    .iter()
                    .map(|gap| format!("{}: {}", gap.dimension, gap.reason))
                    .collect::<Vec<_>>()
                    .join("; ");
                excluded.push(ExcludedTool::new(
                    tool.name.clone(),
                    tool.capability.clone(),
                    reason,
                )?);
            }
            Some(_) => {
                let presentation = match profile.tool_schema {
                    ToolSchemaHandling::InlineFullSchemas => ToolPresentation::Full {
                        schema: tool.schema.clone(),
                    },
                    ToolSchemaHandling::SummariesWithLazySchemas => ToolPresentation::Summary {
                        summary: tool.summary.clone(),
                    },
                    ToolSchemaHandling::LazyPerTool => ToolPresentation::Lazy,
                };
                exposed.push(ExposedTool::new(
                    tool.name.clone(),
                    tool.capability.clone(),
                    presentation,
                )?);
            }
        }
    }
    exposed.sort_by(|a, b| a.name.cmp(&b.name));
    excluded.sort_by(|a, b| a.name.cmp(&b.name));
    ToolExposure::new(
        profile.model_id.clone(),
        profile.tool_schema,
        exposed,
        excluded,
    )
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
    fn the_frozen_capability_key_grammar_is_repinned() {
        for valid in ["terminal", "browser.input", "filesystem.write", "a.b.c_d"] {
            ok(validate_capability_key(valid));
        }
        for invalid in [
            "",
            "Terminal",
            "browser..input",
            ".browser",
            "browser.",
            "browser.Input",
            "9browser",
            "browser-input",
        ] {
            assert!(
                validate_capability_key(invalid).is_err(),
                "{invalid:?} must fail the frozen grammar"
            );
        }
    }

    #[test]
    fn the_frozen_dimension_names_are_repinned() {
        for dimension in CAPABILITY_DIMENSIONS {
            ok(AdmissionGap::new(
                dimension,
                "the dimension does not admit it",
            ));
        }
        assert!(AdmissionGap::new("mood", "not a frozen dimension").is_err());
    }

    #[test]
    fn presentations_serialize_kind_tagged() {
        let full = ToolPresentation::Full {
            schema: "{\"type\":\"object\"}".to_owned(),
        };
        assert_eq!(
            ok(serde_json::to_string(&full)),
            "{\"kind\":\"full\",\"schema\":\"{\\\"type\\\":\\\"object\\\"}\"}"
        );
        let summary = ToolPresentation::Summary {
            summary: "reads one file".to_owned(),
        };
        assert_eq!(
            ok(serde_json::to_string(&summary)),
            "{\"kind\":\"summary\",\"summary\":\"reads one file\"}"
        );
        assert_eq!(
            ok(serde_json::to_string(&ToolPresentation::Lazy)),
            "{\"kind\":\"lazy\"}"
        );
        for serialized in [
            "{\"kind\":\"full\",\"schema\":\"{}\"}",
            "{\"kind\":\"summary\",\"summary\":\"s\"}",
            "{\"kind\":\"lazy\"}",
        ] {
            let reloaded: ToolPresentation = ok(serde_json::from_str(serialized));
            ok(reloaded.validate());
        }
        assert!(
            serde_json::from_str::<ToolPresentation>("\"eager\"").is_err(),
            "unknown presentation kinds are rejected"
        );
        assert!(
            serde_json::from_str::<ToolPresentation>(
                "{\"kind\":\"full\",\"schema\":\"{}\",\"extra\":1}"
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn admissions_reject_the_silent_fall_through_shape() {
        let gap = ok(AdmissionGap::new(
            "model",
            "the model in use does not offer it",
        ));
        assert!(
            CapabilityAdmission::new("browser.navigation", true, vec![gap.clone()]).is_err(),
            "available=true with a named gap is the silent fall-through shape"
        );
        assert!(
            CapabilityAdmission::new("browser.navigation", false, Vec::new()).is_err(),
            "unavailable with no named gap is silent fall-through too"
        );
        ok(CapabilityAdmission::new(
            "browser.navigation",
            false,
            vec![gap, ok(AdmissionGap::new("environment", "no such surface"))],
        ));
    }
}
