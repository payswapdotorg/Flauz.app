//! The resolution record: the evidence type every gap surface builds on
//! (addendum §4).
//!
//! A [`CapabilityResolution`] is the durable answer to "is this capability
//! available for this task, and if not, why?": the requested capability,
//! the per-dimension admission (each of the five dimensions either
//! admitted or carrying its **named gap**), and — per missing dimension —
//! an honest [`UnlockPath`]. The record is a pure function of its inputs
//! (no timestamps, no IDs, no randomness), so it serializes canonically
//! and replays deterministically.
//!
//! The named-gap list is the no-silent-fall-through guarantee made
//! inspectable: a capability quietly missing from the offered set cannot
//! produce an empty record — every missing dimension is named here.

use serde::{Deserialize, Serialize};

use crate::dimension::Dimension;
use crate::inputs::ResolutionInputs;
use crate::key::CapabilityKey;
use crate::{CapError, CapVersion, ensure_explanation};

/// One dimension's admission verdict: the dimension either admits the
/// capability or carries its named gap (the honest reason it does not).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum DimensionAdmission {
    /// The dimension admits the capability.
    Admitted,
    /// The dimension does not admit the capability. `reason` is the
    /// honest, human-readable explanation.
    Gap {
        /// Why the dimension does not admit the capability.
        reason: String,
    },
}

impl DimensionAdmission {
    /// Whether the dimension admits the capability.
    #[must_use]
    pub const fn admits(&self) -> bool {
        matches!(self, Self::Admitted)
    }

    /// The gap reason, or `None` when the dimension admits.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Admitted => None,
            Self::Gap { reason } => Some(reason.as_str()),
        }
    }

    /// Validates the admission's bounds.
    pub fn validate(&self) -> Result<(), CapError> {
        if let Self::Gap { reason } = self {
            ensure_explanation("dimension gap reason", reason)?;
        }
        Ok(())
    }
}

/// A named gap: one admission dimension that does not admit the
/// capability, with the honest reason (addendum §4 — every missing
/// dimension is NAMED in the resolution record).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedGap {
    /// The dimension that does not admit the capability.
    pub dimension: Dimension,
    /// The honest, human-readable reason the dimension does not admit it.
    pub reason: String,
}

impl NamedGap {
    /// Builds a named gap, validating the reason's bounds.
    pub fn new(dimension: Dimension, reason: impl Into<String>) -> Result<Self, CapError> {
        let reason = reason.into();
        ensure_explanation("dimension gap reason", &reason)?;
        Ok(Self { dimension, reason })
    }

    /// Validates the named gap's bounds.
    pub fn validate(&self) -> Result<(), CapError> {
        ensure_explanation("dimension gap reason", &self.reason)
    }
}

/// An unlock path for one missing dimension: an honest action hint saying
/// what would close the gap (addendum §4). The hint never promises an
/// unlock the product cannot perform — it names the action that admits
/// the capability on that dimension, not a UI that exists today.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnlockPath {
    /// The missing dimension this path would close.
    pub dimension: Dimension,
    /// The honest action hint for closing the gap.
    pub action: String,
}

impl UnlockPath {
    /// Builds an unlock path, validating the action hint's bounds.
    pub fn new(dimension: Dimension, action: impl Into<String>) -> Result<Self, CapError> {
        let action = action.into();
        ensure_explanation("unlock action hint", &action)?;
        Ok(Self { dimension, action })
    }

    /// Validates the unlock path's bounds.
    pub fn validate(&self) -> Result<(), CapError> {
        ensure_explanation("unlock action hint", &self.action)
    }
}

/// The resolution record (addendum §4): the requested capability, the
/// per-dimension admission, the named gap list, and the unlock paths —
/// one per missing dimension. This is the evidence type future waves
/// (skill unlock flows, permission UIs, the gap surface) build on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityResolution {
    /// Contract schema version (`"v": 1`).
    pub v: CapVersion,
    /// The capability that was requested.
    pub capability: CapabilityKey,
    /// Whether the capability is available: every dimension admits it.
    pub available: bool,
    /// The per-dimension admissions, in canonical order
    /// ([`Dimension::ALL`]): model, runtime, environment, permissions,
    /// workspace policy.
    pub admissions: Vec<DimensionAdmission>,
    /// The named gap list — one entry per missing dimension, in canonical
    /// order. NON-EMPTY whenever `available` is false (no silent
    /// fall-through); empty iff the capability is available.
    pub gaps: Vec<NamedGap>,
    /// The unlock paths — one per missing dimension, in canonical order,
    /// mirroring `gaps` one-for-one. Empty iff the capability is
    /// available.
    pub unlock_paths: Vec<UnlockPath>,
}

impl CapabilityResolution {
    /// Validates the record against the canonical rules: bounds on every
    /// explanation, admissions in canonical order covering all five
    /// dimensions exactly once, `available` consistent with the gap list,
    /// and the gaps and unlock paths mirroring each other one-for-one in
    /// canonical order.
    pub fn validate(&self) -> Result<(), CapError> {
        if self.admissions.len() != crate::DIMENSIONS {
            return Err(CapError::invalid(format!(
                "a resolution carries exactly {} admissions (one per dimension), found {}",
                crate::DIMENSIONS,
                self.admissions.len()
            )));
        }
        for admission in &self.admissions {
            admission.validate()?;
        }
        // The gap list and the unlock paths must name exactly the missing
        // dimensions, in canonical order, mirroring each other.
        let missing: Vec<Dimension> = self
            .admissions
            .iter()
            .zip(Dimension::ALL)
            .filter(|(admission, _)| !admission.admits())
            .map(|(_, dimension)| dimension)
            .collect();
        let gap_dimensions: Vec<Dimension> = self.gaps.iter().map(|gap| gap.dimension).collect();
        if gap_dimensions != missing {
            return Err(CapError::invalid(
                "the named gap list must mirror the missing dimensions in canonical order",
            ));
        }
        let unlock_dimensions: Vec<Dimension> = self
            .unlock_paths
            .iter()
            .map(|path| path.dimension)
            .collect();
        if unlock_dimensions != missing {
            return Err(CapError::invalid(
                "the unlock paths must mirror the missing dimensions in canonical order",
            ));
        }
        for gap in &self.gaps {
            gap.validate()?;
        }
        for path in &self.unlock_paths {
            path.validate()?;
        }
        if self.available != missing.is_empty() {
            return Err(CapError::invalid(
                "available must be true iff no dimension carries a gap",
            ));
        }
        Ok(())
    }

    /// The admission verdict of one dimension, or `None` if the record
    /// does not cover it.
    #[must_use]
    pub fn admission(&self, dimension: Dimension) -> Option<&DimensionAdmission> {
        self.admissions
            .iter()
            .zip(Dimension::ALL)
            .find(|(_, canonical)| *canonical == dimension)
            .map(|(admission, _)| admission)
    }
}

/// Recomputes the record's derived consistency from its inputs — used by
/// tests to prove a record is the honest output of the resolver, never a
/// hand-staged shape (the no-silent-fall-through companion law).
///
/// # Errors
///
/// Returns [`CapError`] when the record disagrees with resolving
/// `capability` over `inputs`.
pub fn assert_matches_inputs(
    record: &CapabilityResolution,
    capability: &CapabilityKey,
    inputs: &ResolutionInputs,
) -> Result<(), CapError> {
    inputs.validate()?;
    capability.validate()?;
    let expected = crate::resolver::resolve_capability(capability, inputs)?;
    if &expected != record {
        return Err(CapError::invalid(
            "the record does not equal the resolver output for its inputs",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_EXPLANATION_BYTES;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn gap(dimension: Dimension, reason: &str) -> NamedGap {
        ok(NamedGap::new(dimension, reason))
    }

    #[test]
    fn dimension_admissions_serialize_internally_tagged() {
        let admitted = DimensionAdmission::Admitted;
        assert_eq!(
            ok(serde_json::to_string(&admitted)),
            "{\"kind\":\"admitted\"}"
        );
        let gap = DimensionAdmission::Gap {
            reason: "the runtime in use does not advertise it".to_owned(),
        };
        let serialized = ok(serde_json::to_string(&gap));
        assert_eq!(
            serialized,
            concat!(
                "{\"kind\":\"gap\",",
                "\"reason\":\"the runtime in use does not advertise it\"}"
            )
        );
        let reloaded: DimensionAdmission = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, gap);
        assert!(!gap.admits());
        assert_eq!(
            gap.reason(),
            Some("the runtime in use does not advertise it")
        );
        assert_eq!(admitted.reason(), None);
        assert!(
            serde_json::from_str::<DimensionAdmission>("{\"kind\":\"partial\"}").is_err(),
            "unknown admission kinds are rejected"
        );
        assert!(
            serde_json::from_str::<DimensionAdmission>(
                &serialized.replace("\"reason\":", "\"because\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn named_gaps_and_unlock_paths_carry_the_dimension() {
        let named = gap(
            Dimension::Runtime,
            "the runtime in use does not advertise it",
        );
        let serialized = ok(serde_json::to_string(&named));
        assert_eq!(
            serialized,
            concat!(
                "{\"dimension\":\"runtime\",",
                "\"reason\":\"the runtime in use does not advertise it\"}"
            )
        );
        let reloaded: NamedGap = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, named);

        let path = ok(UnlockPath::new(
            Dimension::Runtime,
            "use a runtime that advertises it",
        ));
        let serialized = ok(serde_json::to_string(&path));
        assert_eq!(
            serialized,
            "{\"dimension\":\"runtime\",\"action\":\"use a runtime that advertises it\"}"
        );
        let reloaded: UnlockPath = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, path);
        assert!(
            serde_json::from_str::<NamedGap>(&serialized.replace("\"dimension\":", "\"dim\":"))
                .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn explanations_are_bounded_and_non_empty() {
        assert!(NamedGap::new(Dimension::Model, "").is_err());
        assert!(UnlockPath::new(Dimension::Model, "").is_err());
        let long = "x".repeat(MAX_EXPLANATION_BYTES + 1);
        assert!(NamedGap::new(Dimension::Model, &long).is_err());
        assert!(UnlockPath::new(Dimension::Model, &long).is_err());
    }

    #[test]
    fn records_reject_inconsistent_shapes() {
        let record = CapabilityResolution {
            v: CapVersion,
            capability: ok(CapabilityKey::parse("terminal")),
            available: true,
            admissions: Dimension::ALL
                .iter()
                .map(|_| DimensionAdmission::Admitted)
                .collect(),
            gaps: vec![],
            unlock_paths: vec![],
        };
        ok(record.validate());

        // available=true but a dimension carries a gap.
        let inconsistent = CapabilityResolution {
            available: true,
            admissions: vec![
                DimensionAdmission::Gap {
                    reason: "the model in use does not offer it".to_owned(),
                },
                DimensionAdmission::Admitted,
                DimensionAdmission::Admitted,
                DimensionAdmission::Admitted,
                DimensionAdmission::Admitted,
            ],
            gaps: vec![gap(Dimension::Model, "the model in use does not offer it")],
            unlock_paths: vec![ok(UnlockPath::new(
                Dimension::Model,
                "choose a model that offers it",
            ))],
            ..record.clone()
        };
        assert!(inconsistent.validate().is_err());

        // A gap dimension missing from the named gap list.
        let silent = CapabilityResolution {
            available: false,
            admissions: inconsistent.admissions.clone(),
            gaps: vec![],
            unlock_paths: vec![],
            ..record
        };
        assert!(
            silent.validate().is_err(),
            "a missing dimension with no named gap is the silent fall-through the kernel forbids"
        );

        // Gaps and unlock paths out of canonical order / not mirroring.
        let reordered = CapabilityResolution {
            v: CapVersion,
            capability: ok(CapabilityKey::parse("terminal")),
            available: false,
            admissions: vec![
                DimensionAdmission::Gap {
                    reason: "the model in use does not offer it".to_owned(),
                },
                DimensionAdmission::Gap {
                    reason: "the runtime in use does not advertise it".to_owned(),
                },
                DimensionAdmission::Admitted,
                DimensionAdmission::Admitted,
                DimensionAdmission::Admitted,
            ],
            gaps: vec![
                gap(
                    Dimension::Runtime,
                    "the runtime in use does not advertise it",
                ),
                gap(Dimension::Model, "the model in use does not offer it"),
            ],
            unlock_paths: vec![],
        };
        assert!(reordered.validate().is_err());
    }
}
