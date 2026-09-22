//! Public in-memory fakes: the deterministic conformance surface for the
//! resolver (kernel §7).
//!
//! Everything here is a pure function of its arguments — no I/O, no
//! wall-clock reads, no randomness. [`fake_input_space`] enumerates the
//! **complete** fake input space (every combination of the representative
//! capability vocabulary against every advertisement/decision shape), so
//! the no-silent-fall-through property can be proven by exhaustive
//! deterministic enumeration rather than sampling.
//!
//! Downstream waves (the F2 integration harness, the gap surface wiring,
//! skill unlock flows) use these fakes as the reference semantics for
//! real resolution inputs.

use crate::CapError;
use crate::inputs::{PermissionDecision, PolicyDecision, ResolutionInputs};
use crate::key::CapabilityKey;

/// The representative fake capability vocabulary (a fixed, deterministic
/// slice of the frozen grammar): one plain key, one namespaced key, and
/// one key that environments typically advertise so the "attached but
/// lacking" shape is representable.
pub const FAKE_CAPABILITY_KEYS: [&str; 3] = ["browser.input", "filesystem.write", "terminal"];

/// The advertisement shapes one dimension can take in the fake input
/// space: the capability is offered, or it is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offered {
    /// The dimension advertises the requested capability.
    Yes,
    /// The dimension does not advertise the requested capability.
    No,
}

/// The environment shapes in the fake input space: no environment
/// attached, an environment attached that offers the capability, and an
/// environment attached that offers something else (the "attached but
/// lacking the surface" gap, which is a DIFFERENT named gap from no
/// environment at all).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentShape {
    /// No environment is attached to the task.
    Detached,
    /// An attached environment offers the requested capability.
    AttachedOffering,
    /// An attached environment offers other surfaces but not the requested
    /// capability.
    AttachedLacking,
}

/// One entry of the fake input space: a requested capability plus the
/// canonical inputs it is resolved against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeCase {
    /// The capability whose resolution the case exercises.
    pub capability: CapabilityKey,
    /// The resolver inputs for the case.
    pub inputs: ResolutionInputs,
}

impl FakeCase {
    /// Builds one fake case, validating canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`CapError`] when the constructed inputs fail canonical
    /// validation.
    pub fn new(
        capability: &str,
        model: Offered,
        runtime: Offered,
        environment: EnvironmentShape,
        permission: PermissionDecision,
        policy: PolicyDecision,
    ) -> Result<Self, CapError> {
        let capability = CapabilityKey::parse(capability)?;
        // A second, always-canonically-ordered key so "offers
        // something else" lists are never empty and always sort above
        // every fake vocabulary key ("browser.input" < "filesystem.write"
        // < "terminal" < "web.search" holds alphabetically).
        let other = CapabilityKey::parse("web.search")?;
        let advertised = |offered: Offered| {
            let mut keys = Vec::with_capacity(2);
            if offered == Offered::Yes {
                keys.push(capability.clone());
            }
            keys
        };
        let environment_advertised = match environment {
            EnvironmentShape::Detached => None,
            EnvironmentShape::AttachedOffering => Some(vec![capability.clone()]),
            EnvironmentShape::AttachedLacking => Some(vec![other.clone()]),
        };
        // The "other" key joins the model/runtime lists too when they lack
        // the requested capability, so every advertisement list is
        // non-empty (and sorted) even in the lacking shapes — proving the
        // resolver checks for THE capability, not for list emptiness.
        let mut model_advertised = advertised(model);
        if model == Offered::No {
            model_advertised.push(other.clone());
        }
        let mut runtime_advertised = advertised(runtime);
        if runtime == Offered::No {
            runtime_advertised.push(other.clone());
        }
        let inputs = ResolutionInputs::new(
            model_advertised,
            runtime_advertised,
            environment_advertised,
            permission,
            policy,
        )?;
        Ok(Self { capability, inputs })
    }
}

/// The complete fake input space, deterministically enumerated: every
/// capability in [`FAKE_CAPABILITY_KEYS`] against every combination of
/// model ∈ {offered, lacking}, runtime ∈ {offered, lacking}, environment
/// ∈ {detached, attached-offering, attached-lacking}, permission ∈
/// {granted, denied}, policy ∈ {allowed, restricted} — 48 shapes per
/// capability, 144 cases total, in a fixed order.
///
/// # Errors
///
/// Returns [`CapError`] if any constructed case fails canonical
/// validation (a bug in the fakes, not in caller data).
pub fn fake_input_space() -> Result<Vec<FakeCase>, CapError> {
    let mut space = Vec::new();
    for capability in FAKE_CAPABILITY_KEYS {
        for model in [Offered::Yes, Offered::No] {
            for runtime in [Offered::Yes, Offered::No] {
                for environment in [
                    EnvironmentShape::Detached,
                    EnvironmentShape::AttachedOffering,
                    EnvironmentShape::AttachedLacking,
                ] {
                    for permission in [
                        PermissionDecision::Granted,
                        PermissionDecision::Denied { detail: None },
                    ] {
                        for policy in [
                            PolicyDecision::Allowed,
                            PolicyDecision::Restricted { detail: None },
                        ] {
                            space.push(FakeCase::new(
                                capability,
                                model,
                                runtime,
                                environment,
                                permission.clone(),
                                policy.clone(),
                            )?);
                        }
                    }
                }
            }
        }
    }
    debug_assert_eq!(space.len(), 144);
    Ok(space)
}

/// A typical fake input set for demos and downstream wiring: a workspace
/// where the model and runtime offer browser automation, an environment
/// is attached that offers a terminal but not the browser, permission is
/// granted, and workspace policy restricts browser input — two named
/// gaps, one honest record.
///
/// # Errors
///
/// Returns [`CapError`] if the constructed inputs fail canonical
/// validation.
pub fn typical_inputs() -> Result<ResolutionInputs, CapError> {
    FakeCase::new(
        "browser.input",
        Offered::Yes,
        Offered::Yes,
        EnvironmentShape::AttachedLacking,
        PermissionDecision::Granted,
        PolicyDecision::Restricted {
            detail: Some(
                "this workspace allows browser input only in approved projects".to_owned(),
            ),
        },
    )
    .map(|case| case.inputs)
}

/// A minimal fake input set: a fresh workspace where nothing advertises
/// anything, no environment is attached, permission was never requested,
/// and policy is unset — every dimension names its gap.
///
/// # Errors
///
/// Returns [`CapError`] if the constructed inputs fail canonical
/// validation.
pub fn minimal_inputs() -> Result<ResolutionInputs, CapError> {
    FakeCase::new(
        "terminal",
        Offered::No,
        Offered::No,
        EnvironmentShape::Detached,
        PermissionDecision::Denied { detail: None },
        PolicyDecision::Restricted { detail: None },
    )
    .map(|case| case.inputs)
}

/// An all-admitted fake input set: every dimension offers the terminal
/// capability and both decisions admit — the success-state record.
///
/// # Errors
///
/// Returns [`CapError`] if the constructed inputs fail canonical
/// validation.
pub fn all_admitted_inputs() -> Result<ResolutionInputs, CapError> {
    FakeCase::new(
        "terminal",
        Offered::Yes,
        Offered::Yes,
        EnvironmentShape::AttachedOffering,
        PermissionDecision::Granted,
        PolicyDecision::Allowed,
    )
    .map(|case| case.inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dimension::Dimension;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn the_fake_input_space_is_complete_and_deterministic() {
        let first = ok(fake_input_space());
        let second = ok(fake_input_space());
        assert_eq!(first, second, "the space is deterministic");
        assert_eq!(first.len(), 144, "3 capabilities x 48 input shapes");

        // Every case is canonically valid and resolves.
        for case in &first {
            ok(case.inputs.validate());
            ok(crate::resolver::resolve_capability(
                &case.capability,
                &case.inputs,
            ));
        }

        // The space covers the all-admitted shape and the all-gap shape.
        assert!(
            first
                .iter()
                .any(|case| ok(crate::resolver::resolve_capability(
                    &case.capability,
                    &case.inputs
                ))
                .available),
            "the space contains an available resolution"
        );
        let all_gap = ok(FakeCase::new(
            "terminal",
            Offered::No,
            Offered::No,
            EnvironmentShape::Detached,
            PermissionDecision::Denied { detail: None },
            PolicyDecision::Restricted { detail: None },
        ));
        let record = ok(crate::resolver::resolve_capability(
            &all_gap.capability,
            &all_gap.inputs,
        ));
        assert_eq!(record.gaps.len(), crate::DIMENSIONS);
    }

    #[test]
    fn fake_advertisements_stay_sorted_and_bounded() {
        for case in ok(fake_input_space()) {
            assert!(case.inputs.model_advertised.len() <= 2);
            assert!(case.inputs.runtime_advertised.len() <= 2);
            if let Some(environment) = case.inputs.environment_advertised.as_ref() {
                assert!(!environment.is_empty());
            }
            for list in [
                case.inputs.model_advertised.as_slice(),
                case.inputs.runtime_advertised.as_slice(),
            ] {
                assert!(
                    list.windows(2).all(|pair| pair[0] < pair[1]),
                    "advertisements stay sorted and deduplicated"
                );
            }
        }
    }

    #[test]
    fn dimension_admission_over_the_fakes_matches_the_offered_shape() {
        let case = ok(FakeCase::new(
            "browser.input",
            Offered::Yes,
            Offered::No,
            EnvironmentShape::AttachedLacking,
            PermissionDecision::Granted,
            PolicyDecision::Allowed,
        ));
        assert!(case.inputs.admits(Dimension::Model, &case.capability));
        assert!(!case.inputs.admits(Dimension::Runtime, &case.capability));
        assert!(!case.inputs.admits(Dimension::Environment, &case.capability));
        assert!(case.inputs.admits(Dimension::Permissions, &case.capability));
        assert!(
            case.inputs
                .admits(Dimension::WorkspacePolicy, &case.capability)
        );
    }
}
