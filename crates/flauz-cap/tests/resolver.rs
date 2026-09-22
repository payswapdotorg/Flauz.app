//! The resolver algebra (addendum §4): the intersection with named gaps
//! and the no-silent-fall-through property, proven by deterministic
//! exhaustive enumeration over the complete fake input space.

use std::fmt;

use flauz_cap::dimension::Dimension;
use flauz_cap::fakes::{EnvironmentShape, FakeCase, Offered, fake_input_space};
use flauz_cap::inputs::{PermissionDecision, PolicyDecision};
use flauz_cap::resolution::CapabilityResolution;
use flauz_cap::resolver::resolve_capability;

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn resolve_case(case: &FakeCase) -> CapabilityResolution {
    test_ok(resolve_capability(&case.capability, &case.inputs))
}

/// The no-silent-fall-through property (addendum §4, THE contract of this
/// crate): every requested-but-missing capability yields a resolution
/// whose named gap list is NON-EMPTY. A capability quietly missing from
/// the offered set is a contract violation, so the property is proven by
/// exhaustive deterministic enumeration over the whole fake input space —
/// not by sampling.
#[test]
fn no_silent_fall_through_every_missing_capability_names_its_gaps() {
    let space = test_ok(fake_input_space());
    assert!(!space.is_empty(), "the fake input space is populated");
    for case in &space {
        let record = resolve_case(case);
        // available ⟺ every dimension admits (cross-checked against the
        // inputs directly, not through the record's own bookkeeping).
        let all_admit = Dimension::ALL
            .iter()
            .all(|dimension| case.inputs.admits(*dimension, &case.capability));
        assert_eq!(
            record.available, all_admit,
            "availability must equal the intersection for {:?}",
            case.capability
        );
        if record.available {
            assert!(
                record.gaps.is_empty(),
                "an available capability carries no gaps"
            );
            assert!(record.unlock_paths.is_empty());
            assert!(record.admissions.iter().all(|a| a.admits()));
        } else {
            assert!(
                !record.gaps.is_empty(),
                "a missing capability MUST name its gaps — silent fall-through for {:?}",
                case.capability
            );
            assert!(
                !record.unlock_paths.is_empty(),
                "every named gap carries an unlock path"
            );
            // The named gaps are exactly the non-admitting dimensions, in
            // canonical order, mirrored one-for-one by the unlock paths.
            let missing: Vec<Dimension> = Dimension::ALL
                .iter()
                .filter(|dimension| !case.inputs.admits(**dimension, &case.capability))
                .copied()
                .collect();
            let named: Vec<Dimension> = record.gaps.iter().map(|gap| gap.dimension).collect();
            assert_eq!(named, missing, "the gaps name the missing dimensions");
            let unlocks: Vec<Dimension> = record
                .unlock_paths
                .iter()
                .map(|path| path.dimension)
                .collect();
            assert_eq!(unlocks, missing, "the unlock paths mirror the gaps");
            // Every gap reason and unlock action is honest, non-empty copy.
            for gap in &record.gaps {
                assert!(!gap.reason.is_empty());
            }
            for path in &record.unlock_paths {
                assert!(!path.action.is_empty());
            }
        }
        // The record is internally consistent and round-trips canonically.
        test_ok(record.validate());
        let serialized = test_ok(serde_json::to_string(&record));
        let reloaded: CapabilityResolution = test_ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, record);
    }
}

/// The intersection algebra: removing exactly one dimension from an
/// otherwise fully-admitting workspace names exactly that one gap — each
/// dimension individually, the algebra `A ∩ ¬d` for every `d`.
#[test]
fn the_intersection_algebra_names_each_single_missing_dimension() {
    let base = |model: Offered,
                runtime: Offered,
                environment: EnvironmentShape,
                permission: PermissionDecision,
                policy: PolicyDecision| {
        test_ok(FakeCase::new(
            "terminal",
            model,
            runtime,
            environment,
            permission,
            policy,
        ))
    };
    let checks: [(Dimension, FakeCase); 5] = [
        (
            Dimension::Model,
            base(
                Offered::No,
                Offered::Yes,
                EnvironmentShape::AttachedOffering,
                PermissionDecision::Granted,
                PolicyDecision::Allowed,
            ),
        ),
        (
            Dimension::Runtime,
            base(
                Offered::Yes,
                Offered::No,
                EnvironmentShape::AttachedOffering,
                PermissionDecision::Granted,
                PolicyDecision::Allowed,
            ),
        ),
        (
            Dimension::Environment,
            base(
                Offered::Yes,
                Offered::Yes,
                EnvironmentShape::AttachedLacking,
                PermissionDecision::Granted,
                PolicyDecision::Allowed,
            ),
        ),
        (
            Dimension::Permissions,
            base(
                Offered::Yes,
                Offered::Yes,
                EnvironmentShape::AttachedOffering,
                PermissionDecision::Denied { detail: None },
                PolicyDecision::Allowed,
            ),
        ),
        (
            Dimension::WorkspacePolicy,
            base(
                Offered::Yes,
                Offered::Yes,
                EnvironmentShape::AttachedOffering,
                PermissionDecision::Granted,
                PolicyDecision::Restricted { detail: None },
            ),
        ),
    ];
    for (dimension, case) in &checks {
        let record = resolve_case(case);
        assert!(!record.available);
        assert_eq!(
            record.gaps.len(),
            1,
            "removing {dimension:?} alone names exactly one gap"
        );
        assert_eq!(record.gaps[0].dimension, *dimension);
        assert_eq!(record.unlock_paths.len(), 1);
        assert_eq!(record.unlock_paths[0].dimension, *dimension);
    }
}

/// "No environment attached" is a different named gap from "an
/// environment is attached but does not offer the surface" — the honest
/// distinction the inputs carry and the record must preserve.
#[test]
fn detached_and_lacking_environments_name_different_gaps() {
    let detached = test_ok(FakeCase::new(
        "browser.input",
        Offered::Yes,
        Offered::Yes,
        EnvironmentShape::Detached,
        PermissionDecision::Granted,
        PolicyDecision::Allowed,
    ));
    let lacking = test_ok(FakeCase::new(
        "browser.input",
        Offered::Yes,
        Offered::Yes,
        EnvironmentShape::AttachedLacking,
        PermissionDecision::Granted,
        PolicyDecision::Allowed,
    ));
    let detached_record = resolve_case(&detached);
    let lacking_record = resolve_case(&lacking);
    assert!(!detached_record.available);
    assert!(!lacking_record.available);
    assert_eq!(detached_record.gaps[0].dimension, Dimension::Environment);
    assert_eq!(lacking_record.gaps[0].dimension, Dimension::Environment);
    assert_ne!(
        detached_record.gaps[0].reason, lacking_record.gaps[0].reason,
        "detached and attached-but-lacking are different named gaps"
    );
    assert_eq!(
        detached_record.gaps[0].reason,
        "No environment is attached to this task."
    );
    assert_eq!(
        lacking_record.gaps[0].reason,
        "The attached environment does not offer this capability."
    );
}

/// The permission and policy decisions contribute their honest detail to
/// the named reason when the deciding surface supplies one — the
/// explanation is passed through, never invented.
#[test]
fn decision_details_flow_into_the_named_reasons() {
    let case = test_ok(FakeCase::new(
        "filesystem.write",
        Offered::Yes,
        Offered::Yes,
        EnvironmentShape::AttachedOffering,
        PermissionDecision::Denied {
            detail: Some("approval was declined for this task".to_owned()),
        },
        PolicyDecision::Restricted {
            detail: Some("writing outside the project folder is blocked".to_owned()),
        },
    ));
    let record = resolve_case(&case);
    let Some(permission_gap) = record
        .gaps
        .iter()
        .find(|gap| gap.dimension == Dimension::Permissions)
    else {
        panic!("the permissions gap is named");
    };
    assert!(
        permission_gap
            .reason
            .contains("approval was declined for this task")
    );
    let Some(policy_gap) = record
        .gaps
        .iter()
        .find(|gap| gap.dimension == Dimension::WorkspacePolicy)
    else {
        panic!("the workspace-policy gap is named");
    };
    assert!(
        policy_gap
            .reason
            .contains("writing outside the project folder is blocked")
    );
}

/// Determinism (kernel §7): identical inputs produce byte-identical
/// records across the whole fake input space — the resolver is a pure
/// function, so resolution replays exactly.
#[test]
fn resolution_replays_identically_over_the_whole_space() {
    let space = test_ok(fake_input_space());
    for case in &space {
        let first = test_ok(serde_json::to_string(&resolve_case(case)));
        let second = test_ok(serde_json::to_string(&resolve_case(case)));
        assert_eq!(first, second, "resolution is deterministic");
    }
    // And the space itself is stable.
    assert_eq!(
        test_ok(fake_input_space()),
        space,
        "the fake input space is deterministic"
    );
}
