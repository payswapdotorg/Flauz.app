//! The resolver: the intersection with named gaps (addendum §4).
//!
//! [`resolve_capability`] is a pure function of its inputs: it takes the
//! requested capability and the [`ResolutionInputs`] (model capability
//! metadata, runtime advertisement, environment surfaces, and the
//! permission + policy decisions as call-side data) and produces the
//! [`CapabilityResolution`]. A capability is available iff
//!
//! ```text
//! model ∩ runtime ∩ environment ∩ permissions ∩ workspace policy
//! ```
//!
//! all admit it. Every missing dimension is NAMED in the record with an
//! honest reason and an honest unlock path — a capability quietly
//! missing from the offered set is a contract violation, so a requested
//! but unavailable capability always yields a NON-EMPTY gap list.
//!
//! Determinism (kernel §7): no wall-clock reads, no randomness, no
//! generated identifiers — identical inputs produce byte-identical
//! records.

use crate::dimension::Dimension;
use crate::inputs::{PermissionDecision, PolicyDecision, ResolutionInputs};
use crate::key::CapabilityKey;
use crate::resolution::{CapabilityResolution, DimensionAdmission, NamedGap, UnlockPath};
use crate::{CapError, CapVersion};

/// The per-dimension gap explanations and unlock hints: the honest copy
/// the resolver emits. Reasons state the fact; unlock actions name the
/// action that would close the gap on that dimension — never promising an
/// unlock the product cannot perform (a hint says what admits the
/// capability, not that a surface to do it ships today).
trait GapExplanation {
    /// The honest reason the dimension does not admit the capability.
    fn gap_reason(&self, inputs: &ResolutionInputs) -> String;
    /// The honest action hint that would close the gap.
    fn unlock_action(&self) -> &'static str;
}

impl GapExplanation for Dimension {
    fn gap_reason(&self, inputs: &ResolutionInputs) -> String {
        match self {
            Self::Model => "The model in use does not offer this capability.".to_owned(),
            Self::Runtime => "The runtime in use does not advertise this capability.".to_owned(),
            Self::Environment => match inputs.environment_advertised.as_ref() {
                None => "No environment is attached to this task.".to_owned(),
                Some(_) => "The attached environment does not offer this capability.".to_owned(),
            },
            Self::Permissions => match &inputs.permission {
                PermissionDecision::Denied {
                    detail: Some(detail),
                } => format!(
                    "Permission to use this capability has not been granted for this task. {detail}"
                ),
                PermissionDecision::Denied { detail: None } => {
                    "Permission to use this capability has not been granted for this task."
                        .to_owned()
                }
                PermissionDecision::Granted => {
                    unreachable!("the permissions dimension only explains a gap when denied")
                }
            },
            Self::WorkspacePolicy => match &inputs.policy {
                PolicyDecision::Restricted {
                    detail: Some(detail),
                } => format!(
                    "Workspace policy does not allow this capability for this task. {detail}"
                ),
                PolicyDecision::Restricted { detail: None } => {
                    "Workspace policy does not allow this capability for this task.".to_owned()
                }
                PolicyDecision::Allowed => {
                    unreachable!(
                        "the workspace-policy dimension only explains a gap when restricted"
                    )
                }
            },
        }
    }

    fn unlock_action(&self) -> &'static str {
        match self {
            Self::Model => "Choose a model that offers this capability.",
            Self::Runtime => "Use a runtime that advertises this capability.",
            Self::Environment => "Attach an environment that offers this capability.",
            Self::Permissions => "Grant permission for this capability on this task.",
            Self::WorkspacePolicy => "Ask the workspace owner to allow this capability.",
        }
    }
}

/// Resolves one requested capability over the inputs: the intersection
/// with named gaps (addendum §4). Pure and deterministic — identical
/// inputs produce identical records.
///
/// # Errors
///
/// Returns [`CapError`] when the capability key or the inputs fail
/// canonical validation (the caller's data is re-checked, never trusted
/// blindly).
pub fn resolve_capability(
    capability: &CapabilityKey,
    inputs: &ResolutionInputs,
) -> Result<CapabilityResolution, CapError> {
    capability.validate()?;
    inputs.validate()?;

    let mut admissions = Vec::with_capacity(crate::DIMENSIONS);
    let mut gaps = Vec::new();
    let mut unlock_paths = Vec::new();
    let mut available = true;
    for dimension in Dimension::ALL {
        if inputs.admits(dimension, capability) {
            admissions.push(DimensionAdmission::Admitted);
        } else {
            available = false;
            gaps.push(NamedGap::new(dimension, dimension.gap_reason(inputs))?);
            unlock_paths.push(UnlockPath::new(dimension, dimension.unlock_action())?);
            admissions.push(DimensionAdmission::Gap {
                reason: dimension.gap_reason(inputs),
            });
        }
    }
    let record = CapabilityResolution {
        v: CapVersion,
        capability: capability.clone(),
        available,
        admissions,
        gaps,
        unlock_paths,
    };
    record.validate()?;
    Ok(record)
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

    fn key(value: &str) -> CapabilityKey {
        ok(CapabilityKey::parse(value))
    }

    fn inputs(
        model: &[&str],
        runtime: &[&str],
        environment: Option<&[&str]>,
        permission: PermissionDecision,
        policy: PolicyDecision,
    ) -> ResolutionInputs {
        let to_keys = |values: &[&str]| {
            let mut keys: Vec<CapabilityKey> = values.iter().map(|value| key(value)).collect();
            keys.sort();
            keys.dedup();
            keys
        };
        ok(ResolutionInputs::new(
            to_keys(model),
            to_keys(runtime),
            environment.map(to_keys),
            permission,
            policy,
        ))
    }

    #[test]
    fn every_dimension_admitting_makes_the_capability_available() {
        let record = ok(resolve_capability(
            &key("terminal"),
            &inputs(
                &["terminal"],
                &["terminal"],
                Some(&["terminal"]),
                PermissionDecision::Granted,
                PolicyDecision::Allowed,
            ),
        ));
        assert!(record.available);
        assert!(record.gaps.is_empty());
        assert!(record.unlock_paths.is_empty());
        assert_eq!(record.admissions.len(), 5);
        assert!(record.admissions.iter().all(DimensionAdmission::admits));
        assert_eq!(
            record.capability.as_str(),
            "terminal",
            "the record names the requested capability"
        );
    }

    #[test]
    fn one_missing_dimension_names_exactly_one_gap() {
        let record = ok(resolve_capability(
            &key("browser.input"),
            &inputs(
                &["browser.input"],
                &["browser.input"],
                Some(&["terminal"]),
                PermissionDecision::Granted,
                PolicyDecision::Allowed,
            ),
        ));
        assert!(!record.available);
        assert_eq!(record.gaps.len(), 1, "exactly the environment is missing");
        assert_eq!(record.gaps[0].dimension, Dimension::Environment);
        assert_eq!(
            record.gaps[0].reason,
            "The attached environment does not offer this capability."
        );
        assert_eq!(record.unlock_paths.len(), 1);
        assert_eq!(record.unlock_paths[0].dimension, Dimension::Environment);
        assert_eq!(
            record.unlock_paths[0].action,
            "Attach an environment that offers this capability."
        );
        assert_eq!(
            record
                .admission(Dimension::Environment)
                .and_then(DimensionAdmission::reason),
            Some("The attached environment does not offer this capability.")
        );
        assert!(
            record
                .admission(Dimension::Model)
                .is_some_and(DimensionAdmission::admits)
        );
    }

    #[test]
    fn no_environment_and_denied_permissions_and_restricted_policy_all_name_their_gaps() {
        let record = ok(resolve_capability(
            &key("filesystem.write"),
            &inputs(
                &["filesystem.write"],
                &["filesystem.write"],
                None,
                PermissionDecision::Denied {
                    detail: Some("approval was declined for this task".to_owned()),
                },
                PolicyDecision::Restricted { detail: None },
            ),
        ));
        assert!(!record.available);
        let named: Vec<Dimension> = record.gaps.iter().map(|gap| gap.dimension).collect();
        assert_eq!(
            named,
            vec![
                Dimension::Environment,
                Dimension::Permissions,
                Dimension::WorkspacePolicy
            ],
            "every missing dimension is named, in canonical order"
        );
        assert_eq!(
            record.gaps[0].reason, "No environment is attached to this task.",
            "no environment is distinct from an environment lacking the surface"
        );
        assert_eq!(
            record.gaps[1].reason,
            "Permission to use this capability has not been granted for this task. \
             approval was declined for this task"
        );
        assert_eq!(
            record.gaps[2].reason,
            "Workspace policy does not allow this capability for this task."
        );
        let unlock: Vec<Dimension> = record
            .unlock_paths
            .iter()
            .map(|path| path.dimension)
            .collect();
        assert_eq!(
            unlock, named,
            "the unlock paths mirror the gaps one-for-one"
        );
    }

    #[test]
    fn resolution_is_a_pure_function_of_its_inputs() {
        let inputs = inputs(
            &["terminal"],
            &[],
            Some(&["terminal"]),
            PermissionDecision::Granted,
            PolicyDecision::Allowed,
        );
        let first = ok(resolve_capability(&key("terminal"), &inputs));
        let second = ok(resolve_capability(&key("terminal"), &inputs));
        assert_eq!(
            ok(serde_json::to_string(&first)),
            ok(serde_json::to_string(&second)),
            "identical inputs produce byte-identical records"
        );
    }

    #[test]
    fn invalid_inputs_are_rejected_not_silently_accepted() {
        // A CapabilityKey with invalid content cannot exist in memory
        // (parse validates the frozen grammar), so the re-validation the
        // resolver performs is exercised through struct-built inputs that
        // bypass the validating constructors.
        let bad = ResolutionInputs {
            v: CapVersion,
            model_advertised: vec![key("terminal")],
            runtime_advertised: vec![key("terminal"), key("terminal")],
            environment_advertised: None,
            permission: PermissionDecision::Granted,
            policy: PolicyDecision::Allowed,
        };
        assert!(
            resolve_capability(&key("terminal"), &bad).is_err(),
            "duplicated advertisements fail resolution, not silently accepted"
        );
        let unsorted = ResolutionInputs {
            v: CapVersion,
            model_advertised: vec![key("terminal"), key("browser.input")],
            runtime_advertised: vec![],
            environment_advertised: Some(vec![]),
            permission: PermissionDecision::Granted,
            policy: PolicyDecision::Allowed,
        };
        assert!(
            resolve_capability(&key("terminal"), &unsorted).is_err(),
            "unsorted advertisements fail resolution, not silently accepted"
        );
    }
}
