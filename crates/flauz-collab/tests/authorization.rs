//! The authorization evaluator suite (work order COL-001): role
//! defaults, explicit grants, the owner-protection rule, named denial —
//! and the no-silent-no-op property over the whole fake input space.

use flauz_collab::membership::{
    AccessLevel, Role, RoleLattice, SurfaceFamily, WorkspaceMembership,
};
use flauz_collab::permission::{
    AllowRule, AuthorizationRequest, DecisionOutcome, DenialReason, GrantEffect, PermissionGrant,
    authorize,
};
use flauz_collab::refs::{ActorRef, WorkspaceRef};

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn workspace() -> WorkspaceRef {
    ok(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0"))
}

fn membership(actor: &str, name: &str, role: Role) -> WorkspaceMembership {
    ok(WorkspaceMembership::new(
        workspace(),
        ok(ActorRef::user(actor)),
        name,
        role,
    ))
}

fn three_role_roster() -> Vec<WorkspaceMembership> {
    vec![
        membership("ana", "Ana", Role::Owner),
        membership("dev", "Dev", Role::Contributor),
        membership("mira", "Mira", Role::Viewer),
    ]
}

fn request<'a>(
    workspace: &'a WorkspaceRef,
    actor: &'a ActorRef,
    action: AccessLevel,
    surface: SurfaceFamily,
    memberships: &'a [WorkspaceMembership],
    grants: &'a [PermissionGrant],
    lattice: &'a RoleLattice,
) -> AuthorizationRequest<'a> {
    AuthorizationRequest {
        workspace,
        actor,
        action,
        surface,
        target: None,
        memberships,
        grants,
        lattice,
    }
}

fn four_role_roster() -> Vec<WorkspaceMembership> {
    vec![
        membership("ana", "Ana", Role::Owner),
        membership("sam", "Sam", Role::Admin),
        membership("dev", "Dev", Role::Contributor),
        membership("mira", "Mira", Role::Viewer),
    ]
}

/// The role defaults: the frozen lattice admits exactly the shipping
/// defaults per role and surface, and nothing above them.
#[test]
fn role_defaults_admit_the_frozen_lattice_and_nothing_above() {
    let ws = workspace();
    let roster = four_role_roster();
    let lattice = RoleLattice::default_lattice();
    let empty: Vec<PermissionGrant> = Vec::new();
    // (role, surface, expected level)
    let expected: Vec<(Role, SurfaceFamily, AccessLevel)> = vec![
        (Role::Owner, SurfaceFamily::Tasks, AccessLevel::Manage),
        (Role::Admin, SurfaceFamily::Members, AccessLevel::Manage),
        (
            Role::Contributor,
            SurfaceFamily::Tasks,
            AccessLevel::Contribute,
        ),
        (
            Role::Contributor,
            SurfaceFamily::ProviderAccounts,
            AccessLevel::Read,
        ),
        (Role::Viewer, SurfaceFamily::Tasks, AccessLevel::Read),
        (
            Role::Viewer,
            SurfaceFamily::ProviderAccounts,
            AccessLevel::None,
        ),
        (Role::Viewer, SurfaceFamily::Members, AccessLevel::Read),
    ];
    for (role, surface, level) in expected {
        let actor = roster
            .iter()
            .find(|membership| membership.role == role)
            .map(|membership| membership.actor.clone())
            .unwrap_or_else(|| panic!("the four-role roster covers {role:?}"));
        let probe = request(&ws, &actor, level, surface, &roster, &empty, &lattice);
        let decision = ok(authorize(&probe));
        assert_eq!(
            decision.outcome,
            DecisionOutcome::Allow {
                role,
                via: AllowRule::RoleDefault
            },
            "{role:?} must reach {level:?} on {surface:?} via the role default"
        );
        // One level above the default (when the default is not manage):
        // the named role denial.
        if level < AccessLevel::Manage {
            let above = request(
                &ws,
                &actor,
                AccessLevel::Manage,
                surface,
                &roster,
                &empty,
                &lattice,
            );
            let decision = ok(authorize(&above));
            assert_eq!(
                decision.outcome.denial_reason(),
                Some(DenialReason::RoleDenied),
                "{role:?} asking manage on {surface:?} denies with the named reason"
            );
        }
    }
}

/// Explicit grants override role defaults in both directions: a viewer
/// elevated on environments contributes; a contributor elevated on
/// provider accounts manages; an explicit deny blocks even the owner.
#[test]
fn explicit_grants_override_role_defaults_in_both_directions() {
    let ws = workspace();
    let roster = three_role_roster();
    let lattice = RoleLattice::default_lattice();
    let grants = vec![
        ok(PermissionGrant::new(
            workspace(),
            ok(ActorRef::user("mira")),
            SurfaceFamily::Environments,
            GrantEffect::Allow {
                level: AccessLevel::Contribute,
            },
        )),
        ok(PermissionGrant::new(
            workspace(),
            ok(ActorRef::user("dev")),
            SurfaceFamily::ProviderAccounts,
            GrantEffect::Allow {
                level: AccessLevel::Manage,
            },
        )),
    ];
    let mira = ok(ActorRef::user("mira"));
    let dev = ok(ActorRef::user("dev"));
    // Mira (viewer: environments read) contributes via the grant.
    let probe = request(
        &ws,
        &mira,
        AccessLevel::Contribute,
        SurfaceFamily::Environments,
        &roster,
        &grants,
        &lattice,
    );
    assert_eq!(
        ok(authorize(&probe)).outcome,
        DecisionOutcome::Allow {
            role: Role::Viewer,
            via: AllowRule::ExplicitAllow,
        }
    );
    // Mira still cannot manage environments (the grant stops at
    // contribute) — the named role denial carries her level.
    let probe = request(
        &ws,
        &mira,
        AccessLevel::Manage,
        SurfaceFamily::Environments,
        &roster,
        &grants,
        &lattice,
    );
    let decision = ok(authorize(&probe));
    assert_eq!(
        decision.outcome.denial_reason(),
        Some(DenialReason::RoleDenied)
    );
    if let DecisionOutcome::Deny { role_level, .. } = decision.outcome {
        assert_eq!(role_level, Some(AccessLevel::Read));
    }
    // Dev (contributor: provider accounts read) manages via the grant.
    let probe = request(
        &ws,
        &dev,
        AccessLevel::Manage,
        SurfaceFamily::ProviderAccounts,
        &roster,
        &grants,
        &lattice,
    );
    assert_eq!(
        ok(authorize(&probe)).outcome,
        DecisionOutcome::Allow {
            role: Role::Contributor,
            via: AllowRule::ExplicitAllow,
        }
    );
    // An explicit deny blocks even the owner on the denied surface.
    let mut deny_all = grants.clone();
    deny_all.push(ok(PermissionGrant::new(
        workspace(),
        ok(ActorRef::user("ana")),
        SurfaceFamily::Tasks,
        GrantEffect::Deny,
    )));
    let ana = ok(ActorRef::user("ana"));
    let probe = request(
        &ws,
        &ana,
        AccessLevel::Read,
        SurfaceFamily::Tasks,
        &roster,
        &deny_all,
        &lattice,
    );
    assert_eq!(
        ok(authorize(&probe)).outcome.denial_reason(),
        Some(DenialReason::ExplicitDeny),
        "an explicit deny beats the owner's role default"
    );
}

/// The owner-protection rule: managing the members surface ON the owner
/// requires the owner. Admin differs from owner exactly here.
#[test]
fn acting_on_the_owner_requires_the_owner() {
    let ws = workspace();
    let roster = vec![
        membership("ana", "Ana", Role::Owner),
        membership("sam", "Sam", Role::Admin),
        membership("dev", "Dev", Role::Contributor),
    ];
    let lattice = RoleLattice::default_lattice();
    let empty: Vec<PermissionGrant> = Vec::new();
    let ana = ok(ActorRef::user("ana"));
    let sam = ok(ActorRef::user("sam"));
    let dev = ok(ActorRef::user("dev"));
    // The admin manages members generally (role default)…
    let probe = AuthorizationRequest {
        target: Some(&dev),
        ..request(
            &ws,
            &sam,
            AccessLevel::Manage,
            SurfaceFamily::Members,
            &roster,
            &empty,
            &lattice,
        )
    };
    assert_eq!(
        ok(authorize(&probe)).outcome,
        DecisionOutcome::Allow {
            role: Role::Admin,
            via: AllowRule::RoleDefault,
        }
    );
    // …but acting on the OWNER's membership denies with the named
    // owner-protected reason — for the admin…
    let probe = AuthorizationRequest {
        target: Some(&ana),
        ..request(
            &ws,
            &sam,
            AccessLevel::Manage,
            SurfaceFamily::Members,
            &roster,
            &empty,
            &lattice,
        )
    };
    let decision = ok(authorize(&probe));
    assert_eq!(
        decision.outcome.denial_reason(),
        Some(DenialReason::OwnerProtected)
    );
    // …for the contributor (before the role denial would fire — the
    // owner protection is checked first, so the WHY is precise)…
    let probe = AuthorizationRequest {
        target: Some(&ana),
        ..request(
            &ws,
            &dev,
            AccessLevel::Manage,
            SurfaceFamily::Members,
            &roster,
            &empty,
            &lattice,
        )
    };
    assert_eq!(
        ok(authorize(&probe)).outcome.denial_reason(),
        Some(DenialReason::OwnerProtected)
    );
    // …and the owner themself may act on their own membership.
    let probe = AuthorizationRequest {
        target: Some(&ana),
        ..request(
            &ws,
            &ana,
            AccessLevel::Manage,
            SurfaceFamily::Members,
            &roster,
            &empty,
            &lattice,
        )
    };
    assert_eq!(
        ok(authorize(&probe)).outcome,
        DecisionOutcome::Allow {
            role: Role::Owner,
            via: AllowRule::RoleDefault,
        }
    );
}

/// The no-silent-no-op property: over the whole deterministic fake
/// input space (three roles × every surface × every level, with and
/// without grants), every evaluation either allows with a named rule or
/// denies with a named reason — there is no third outcome.
#[test]
fn no_silent_no_op_every_denial_is_named() {
    let ws = workspace();
    let roster = three_role_roster();
    let lattice = RoleLattice::default_lattice();
    let grants = [
        ok(PermissionGrant::new(
            workspace(),
            ok(ActorRef::user("mira")),
            SurfaceFamily::Environments,
            GrantEffect::Allow {
                level: AccessLevel::Contribute,
            },
        )),
        ok(PermissionGrant::new(
            workspace(),
            ok(ActorRef::user("dev")),
            SurfaceFamily::Environments,
            GrantEffect::Deny,
        )),
    ];
    let actors = [
        ok(ActorRef::user("ana")),
        ok(ActorRef::user("dev")),
        ok(ActorRef::user("mira")),
        ok(ActorRef::user("outsider")),
    ];
    let mut evaluations = 0usize;
    let mut denials = 0usize;
    for actor in &actors {
        for surface in SurfaceFamily::ALL {
            for action in AccessLevel::ALL {
                for grant_set in [&grants[..], &[][..]] {
                    let probe = request(&ws, actor, action, surface, &roster, grant_set, &lattice);
                    let decision = ok(authorize(&probe));
                    evaluations += 1;
                    match decision.outcome {
                        DecisionOutcome::Allow { via, .. } => {
                            assert!(
                                via == AllowRule::RoleDefault || via == AllowRule::ExplicitAllow
                            );
                        }
                        DecisionOutcome::Deny { reason, .. } => {
                            denials += 1;
                            assert!(
                                matches!(
                                    reason,
                                    DenialReason::NotAMember
                                        | DenialReason::OwnerProtected
                                        | DenialReason::ExplicitDeny
                                        | DenialReason::RoleDenied
                                        | DenialReason::RoleNotInLattice
                                ),
                                "every denial carries one of the frozen reasons (found {reason:?})"
                            );
                        }
                    }
                }
            }
        }
    }
    // The full deterministic space: 4 actors × 4 surface families × 4
    // levels × 2 grant sets = 128 evaluations, every one either an
    // allow with a named rule or a denial with a named reason.
    assert_eq!(evaluations, 128, "the enumeration covers the whole space");
    assert!(denials >= 40, "the denial family is exercised ({denials})");
}
