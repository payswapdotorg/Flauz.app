//! The provider-neutral parity-lab fabric (F2 Wave 4, work order **LAB-001**):
//! declarative journey specs, the normalized evidence schema, the
//! named-findings registry, the reference-vs-candidate comparator, and the
//! lab-adapter contract with the local + fake-remote providers.
//!
//! The lab's user is the **operator** (the Tech Lead; provider teams later).
//! A journey is defined ONCE as data ([`JourneySpec`]) and runs unchanged
//! against every compliant lab adapter ([`LabAdapter`]); evidence comes back
//! normalized ([`EvidenceRecord`] — schema'd, canonical-JSON, comparable),
//! the comparator ([`compare`]) diffs a candidate against a reference with
//! named divergences, and the F1 accessibility lessons ride the schema as a
//! named-findings registry ([`findings`]) that every future run can reference
//! by id.
//!
//! # The F8 gate law — provider neutrality by construction (addendum §4)
//!
//! A lab journey is a declarative spec (data) plus a normalized evidence
//! schema; the same journey bytes run unchanged against every compliant lab
//! adapter. The local Linux lab is the **reference provider**
//! ([`LocalLabAdapter`]); the fake-remote lab adapter is the **consistency
//! provider** ([`FakeRemoteLabAdapter`]) — the ENV-001 law: the fake remote
//! every later real remote (E2B, Daytona, Azure, GitHub Actions, Codemagic)
//! copies. Real provider adapters are adapter-family DATA with their topology
//! metadata this wave ([`known_provider_families`]) — skeletons and fakes
//! only, NO network; real integrations are gated behind the lab proving the
//! contract first. The test
//! `the_same_journey_bytes_produce_comparable_evidence_from_both_adapters`
//! proves the law in-tests: one journey, two adapters, comparable normalized
//! evidence.
//!
//! # Frames are references, never pixels (addendum §4)
//!
//! The comparator never compares raw screenshots alone: an evidence record
//! carries frame REFERENCES ([`FrameRef`]) plus the NORMALIZED frame content
//! — the observed named-anchor states ([`AnchorObservation`]) — and
//! provider-local fingerprints ([`FrameDigest`]) that the comparator ignores
//! by design. Accessibility evidence is first-class: focus-order probes,
//! trap probes and chord ladders are assertion kinds ([`StateAssertion`])
//! whose results carry the F1 patterns as data.
//!
//! # VLM adjudication stays operator-side (the evidence SLOT)
//!
//! No VLM calls happen inside this crate. A frame probe's evidence carries a
//! [`VlmSlot`]: the bounded adjudication prompt (data) and a reference to the
//! archived read result ([`VlmReadRef`], filled operator-side). The fake
//! adapters fill deterministic slots; the comparator compares prompts but
//! never the read-result references (archive locations are provider-local).
//!
//! # The named-findings registry (addendum §4)
//!
//! The F1 accessibility lessons — PTY focus transfer, bracket swap, modal
//! traps, first-run keyboard swallowing, the N6 shifted-symbol family — ride
//! the schema as DATA ([`findings::builtin_findings_registry`]): id, surface,
//! description, regression-guard status. Records reference findings by ID,
//! never by prose: a probe result carries [`FindingId`] links that must
//! resolve in the registry.
//!
//! # Inputs only — the frozen formats cross the seam (the flauz-cap law)
//!
//! The crate does NOT import `flauz-world`, `flauz-exec`, `flauz-context`,
//! `flauz-cap` or `flauz-orch`: foreign entities cross the seam exclusively
//! as frozen format strings. [`refs`] re-validates the frozen
//! namespaced capability-key grammar (`[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*`,
//! owned by `flauz-exec`'s `CapabilityId`) and the frozen canonical-ID
//! grammar where referenced. The adapter trait is NEW surface owned by this
//! crate — not an `ExecutionProvider` extension.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields (`deny_unknown_fields`),
//! contains no floats, and tags enums internally with `"kind"` (fieldless
//! enums serialize as their lowercase snake_case names). Timestamps do not
//! occur in v1 records: runs are ordered by the journey's own step order,
//! and the operator's archive carries run metadata — the crate never reads
//! wall-clock time.
//!
//! # Determinism (kernel §7)
//!
//! Contract code in this crate never reads wall-clock time or randomness
//! and generates no identifiers: the same journey bytes over the same
//! adapter produce byte-identical evidence, and the comparator is a pure
//! function of (journey, reference, candidate) — byte-stable reports. The
//! in-memory [`fakes`] are fully deterministic.
//!
//! # Credentials are references, forever (Wave-4 addendum §3)
//!
//! No contract type, fixture, log line or serialized state in this crate
//! contains credential material: provider metadata is topology data
//! (provider kind, family, locality, lab surfaces). The credential-marker
//! scan extends to the new record families (the conformance tests).

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod adapter;
pub mod comparator;
pub mod evidence;
pub mod fakes;
pub mod findings;
pub mod journey;
pub mod refs;

pub use crate::adapter::{
    FAMILY_SKELETON_NOT_WIRED, FakeRemoteLabAdapter, FamilySkeletonAdapter, FamilyStatus,
    Interaction, LabAdapter, LabProviderFamily, LocalLabAdapter, ProviderFamilies,
    derive_required_capabilities, known_provider_families, run_journey,
};
pub use crate::comparator::{
    ComparatorReport, Divergence, DivergenceDetail, Severity, Side, StepComparison, compare,
};
pub use crate::evidence::{
    ActionOutcome, AdapterDescriptor, AnchorObservation, EvidenceRecord, Locality, ObservedState,
    ProbeEvidence, ProbeResult, ProviderFamily, StepEvidence, VlmSlot,
};
pub use crate::findings::{FindingEntry, FindingsRegistry, GuardStatus, builtin_findings_registry};
pub use crate::journey::{
    JourneyAction, JourneyProbe, JourneySpec, JourneyStep, ProbeSpec, StateAssertion,
    required_capabilities,
};
pub use crate::refs::{
    AnchorId, CapabilityKey, Chord, FindingId, FrameDigest, FrameMoment, FrameRef, JourneyId,
    JourneyRef, ProbeId, RunId, StateKey, StepId, SurfaceId, VlmReadRef,
};

/// Maximum length of short human-readable names and identifiers (matches
/// the flauz-world bound).
pub const MAX_NAME_BYTES: usize = 256;

/// Maximum length of bounded prose fields (descriptions, prompts, honest
/// reasons — matches the flauz-cap bound).
pub const MAX_EXPLANATION_BYTES: usize = 512;

/// Maximum length of typed journey text (the anchor-task objective family).
pub const MAX_TEXT_BYTES: usize = 1024;

/// Maximum number of steps in one journey.
pub const MAX_STEPS: usize = 64;

/// Maximum number of probes on one journey step.
pub const MAX_PROBES_PER_STEP: usize = 8;

/// Maximum number of UI surfaces one journey declares.
pub const MAX_SURFACES: usize = 16;

/// Maximum number of product journeys (J-ids) one lab journey maps to.
pub const MAX_JOURNEY_REFS: usize = 8;

/// Maximum number of known findings one registry carries.
pub const MAX_FINDINGS: usize = 64;

/// Maximum number of finding links one probe result carries.
pub const MAX_FINDING_LINKS: usize = 8;

/// Maximum number of anchors one normalized frame observes.
pub const MAX_FRAME_ANCHORS: usize = 32;

/// Maximum number of lab surfaces one adapter descriptor carries.
pub const MAX_LAB_SURFACES: usize = 32;

/// Contract-level validation error for values that fail canonical rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabError {
    /// A value violates a canonical rule (bounds, ordering, grammar,
    /// record consistency, adapter-contract misuse — each named).
    Invalid(String),
    /// A skeleton adapter was asked to perform work its real integration
    /// does not have yet: the honest not-wired refusal (no network this
    /// wave; real integrations follow the lab-proven contract).
    NotWired(String),
}

impl LabError {
    /// Builds a [`LabError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }

    /// Returns the error's reason text.
    #[must_use]
    pub fn reason(&self) -> &str {
        match self {
            Self::Invalid(reason) | Self::NotWired(reason) => reason,
        }
    }
}

impl fmt::Display for LabError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid lab state: {reason}"),
            Self::NotWired(reason) => write!(formatter, "not wired yet: {reason}"),
        }
    }
}

impl Error for LabError {}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabVersion;

impl serde::Serialize for LabVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for LabVersion {
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

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), LabError> {
    if value.is_empty() {
        Err(LabError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), LabError> {
    if value.len() > max_bytes {
        Err(LabError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_name(field: &'static str, value: &str) -> Result<(), LabError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_NAME_BYTES)
}

pub(crate) fn ensure_explanation(field: &'static str, value: &str) -> Result<(), LabError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_EXPLANATION_BYTES)
}

pub(crate) fn ensure_list_bound(
    field: &'static str,
    list_len: usize,
    max_entries: usize,
) -> Result<(), LabError> {
    if list_len > max_entries {
        Err(LabError::invalid(format!(
            "{field} exceeds {max_entries} entries"
        )))
    } else {
        Ok(())
    }
}

/// Validates that a list of ordered reference values is canonically
/// ordered: strictly ascending, so neither duplicated nor out of order.
pub(crate) fn ensure_sorted_unique<T: PartialOrd>(
    field: &'static str,
    values: &[T],
) -> Result<(), LabError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(LabError::invalid(format!(
            "{field} must be sorted ascending and free of duplicates"
        )));
    }
    Ok(())
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
    fn the_schema_version_marker_round_trips_and_rejects_other_versions() {
        assert_eq!(ok(serde_json::to_string(&LabVersion)), "1");
        let reloaded: LabVersion = ok(serde_json::from_str("1"));
        assert_eq!(reloaded, LabVersion);
        assert!(
            serde_json::from_str::<LabVersion>("2").is_err(),
            "a future schema version must fail the canonical read, never silently misread"
        );
    }

    #[test]
    fn reasons_are_displayable_for_every_error_shape() {
        let invalid = LabError::invalid("journey steps must be unique");
        assert_eq!(
            invalid.to_string(),
            "invalid lab state: journey steps must be unique"
        );
        assert_eq!(invalid.reason(), "journey steps must be unique");
        let not_wired = LabError::NotWired("the e2b lab adapter is a skeleton".to_owned());
        assert_eq!(
            not_wired.to_string(),
            "not wired yet: the e2b lab adapter is a skeleton"
        );
        assert_eq!(not_wired.reason(), "the e2b lab adapter is a skeleton");
    }
}
