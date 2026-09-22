//! The capability resolver: intersection with named gaps (F2 Wave 2, work
//! order **CAP-001**).
//!
//! A capability is available for a task iff **every** admission dimension
//! admits it (the frozen intersection of the Wave-2 kernel addendum §4):
//!
//! ```text
//! model ∩ runtime ∩ environment ∩ permissions ∩ workspace policy
//! ```
//!
//! This crate computes that intersection over inputs taken as **data** and
//! produces the resolution record every gap surface builds on:
//! [`CapabilityResolution`] — the requested capability, the per-dimension
//! admission (each dimension either admitted or carrying its **named gap**
//! with an honest reason), and the unlock paths (per missing dimension, an
//! honest action hint that never promises an unlock the product cannot
//! perform).
//!
//! # No silent fall-through (addendum §4)
//!
//! Every requested-but-missing capability yields a resolution whose named
//! gap list is NON-EMPTY: a capability quietly missing from the offered set
//! is a contract violation. The property
//! `no_silent_fall_through_every_missing_capability_names_its_gaps` proves
//! it by deterministic enumeration over the whole fake input space
//! ([`fakes::fake_input_space`]).
//!
//! # Inputs only — the frozen formats cross the seam (the flauz-context
//! pattern)
//!
//! The resolver takes the model's capability metadata, the runtime
//! advertisement, the environment surfaces, and the permission + policy
//! decisions as plain call-side data. This crate does NOT import
//! `flauz-world`, `flauz-exec` or `flauz-context`: foreign entities cross
//! the seam exclusively as frozen format strings —
//! [`CapabilityKey`] re- validates the frozen namespaced-key grammar
//! `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*` (for example `terminal`,
//! `browser.input`) that `flauz-exec`'s `CapabilityId` owns — never as
//! re-defined entity types. There is no permission system and no policy
//! engine here: [`PermissionDecision`] and [`PolicyDecision`] are inputs.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields (`deny_unknown_fields`),
//! contains no floats, and tags enums internally with `"kind"` (fieldless
//! enums serialize as their lowercase names). Records carry no timestamps
//! or IDs: a resolution is a pure, deterministic function of its inputs.
//!
//! # Determinism (kernel §7)
//!
//! Contract code in this crate never reads wall-clock time or randomness
//! and generates no identifiers: the resolver is a pure function, so
//! identical inputs always produce identical records. The in-memory
//! [`fakes`] are fully deterministic.

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod dimension;
pub mod fakes;
pub mod inputs;
pub mod key;
pub mod resolution;
pub mod resolver;

pub use crate::dimension::Dimension;
pub use crate::inputs::{PermissionDecision, PolicyDecision, ResolutionInputs};
pub use crate::key::CapabilityKey;
pub use crate::resolution::{CapabilityResolution, DimensionAdmission, NamedGap, UnlockPath};
pub use crate::resolver::resolve_capability;

/// Maximum length of a capability key (matches the flauz-exec bound).
pub const MAX_NAME_BYTES: usize = 256;

/// Maximum length of a gap reason or an unlock action hint.
pub const MAX_EXPLANATION_BYTES: usize = 512;

/// Maximum number of capabilities in one advertisement list (matches the
/// flauz-exec bound).
pub const MAX_CAPABILITY_KEYS: usize = 64;

/// The number of admission dimensions in the frozen intersection
/// (model, runtime, environment, permissions, workspace policy).
pub const DIMENSIONS: usize = 5;

/// Contract-level validation error for values that fail canonical rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapError {
    /// A value violates a canonical rule (bounds, ordering, grammar,
    /// record consistency).
    Invalid(String),
}

impl CapError {
    /// Builds a [`CapError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl fmt::Display for CapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid capability state: {reason}"),
        }
    }
}

impl Error for CapError {}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapVersion;

impl serde::Serialize for CapVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for CapVersion {
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

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), CapError> {
    if value.is_empty() {
        Err(CapError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), CapError> {
    if value.len() > max_bytes {
        Err(CapError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_explanation(field: &'static str, value: &str) -> Result<(), CapError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_EXPLANATION_BYTES)
}

/// Validates that a capability list is bounded, syntactically valid,
/// deduplicated and canonically ordered. Advertisement lists are canonical
/// state: sorted ascending, no duplicates (the flauz-exec rule).
pub(crate) fn ensure_capability_list(
    field: &'static str,
    keys: &[CapabilityKey],
) -> Result<(), CapError> {
    if keys.len() > MAX_CAPABILITY_KEYS {
        return Err(CapError::invalid(format!(
            "{field} exceeds {MAX_CAPABILITY_KEYS} entries"
        )));
    }
    for key in keys {
        key.validate()?;
    }
    if keys.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(CapError::invalid(format!(
            "{field} must be sorted and free of duplicates"
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
    fn contract_version_serializes_as_one_and_rejects_others() {
        let serialized = ok(serde_json::to_string(&CapVersion));
        assert_eq!(serialized, "1");
        assert!(serde_json::from_str::<CapVersion>("1").is_ok());
        assert!(serde_json::from_str::<CapVersion>("2").is_err());
        assert!(serde_json::from_str::<CapVersion>("0").is_err());
    }
}
