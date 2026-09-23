//! Permission grants and the authorization evaluator (work order
//! COL-001 part 1): explicit allow/deny per surface overriding the role
//! default, and the Allow/Deny decision with the **named reason** — the
//! CAP-001 law applied to access: a denial is never a silent no-op.
//!
//! A [`PermissionGrant`] records one explicit override for one actor on
//! one surface family: either [`GrantEffect::Allow`] at an access level
//! (elevating the actor above their role's default), or
//! [`GrantEffect::Deny`] (blocking the surface for them outright).
//!
//! The evaluator ([`authorize`]) takes its inputs as **data** — actor +
//! action + surface (+ optional target, for member-directed actions) +
//! the workspace roster + grants + the role lattice — and produces an
//! [`AuthorizationDecision`] carrying the outcome with the named rule
//! that produced it:
//!
//! - [`AllowRule::ExplicitAllow`] — an explicit grant at or above the
//!   requested action admitted it;
//! - [`AllowRule::RoleDefault`] — the role's lattice row admitted it;
//! - [`DenialReason::NotAMember`] — the actor holds no membership in
//!   this workspace;
//! - [`DenialReason::OwnerProtected`] — acting on the owner's membership
//!   requires the owner (the one rule where admin ≠ owner);
//! - [`DenialReason::ExplicitDeny`] — a deny grant blocks the surface;
//! - [`DenialReason::RoleDenied`] — the role's default level is below
//!   the requested action;
//! - [`DenialReason::RoleNotInLattice`] — the role has no lattice row (a
//!   named configuration gap);
//! - [`DenialReason::PrivateRecord`] — the member-private projection law
//!   (Wave-4 addendum §6): a member-private record is visible to its
//!   owner alone (used by the sharing projection filter and the
//!   simulator's store seam);
//! - [`DenialReason::VersionConflict`] — the optimistic-concurrency law
//!   (kernel §3): the mutation's expected version did not match (used
//!   by the simulator's store seam — the no-lost-update guarantee).
//!
//! # Precedence (deterministic, frozen)
//!
//! 1. membership: no matching roster entry → `not_a_member`;
//! 2. owner protection: managing the members surface **on the owner**
//!    (a `target` whose role is owner) requires the actor to be the
//!    owner → `owner_protected`;
//! 3. an explicit deny grant for (actor, surface) → `explicit_deny`;
//! 4. an explicit allow grant at or above the action → `explicit_allow`;
//! 5. the role default: the lattice level admits the action →
//!    `role_default`;
//! 6. the lattice carries no row for the role → `role_not_in_lattice`;
//! 7. otherwise → `role_denied` (with the role and its level recorded).
//!
//! A deny grant therefore beats an allow grant, and both beat the role
//! default; an allow grant rescues a low or missing role default. Every
//! path is named — the no-silent-no-op property is pinned by
//! `no_silent_no_op_every_denial_is_named`.

use serde::{Deserialize, Serialize};

use crate::membership::{
    AccessLevel, Role, RoleLattice, SurfaceFamily, WorkspaceMembership, validate_roster,
};
use crate::refs::{ActorRef, WorkspaceRef};
use crate::{CollabError, CollabVersion, MAX_GRANTS, ensure_list_bound};

/// The named rule that admitted an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllowRule {
    /// An explicit allow grant at or above the action admitted it.
    ExplicitAllow,
    /// The role's lattice default admitted it.
    RoleDefault,
}

/// The named reason an action was denied — never a silent no-op (the
/// CAP-001 law applied to access). The last two variants are the
/// seam-level laws (used by the projection filter and the simulator's
/// store seam, which re-uses this vocabulary so every denial in the
/// collaboration fabric is named by one frozen registry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DenialReason {
    /// The actor holds no membership in this workspace.
    NotAMember,
    /// Acting on the owner's membership requires the owner.
    OwnerProtected,
    /// An explicit deny grant blocks this surface for this actor.
    ExplicitDeny,
    /// The actor's role does not reach the requested action on this
    /// surface.
    RoleDenied,
    /// The actor's role has no row in the lattice (a named configuration
    /// gap).
    RoleNotInLattice,
    /// The member-private projection law (addendum §6): a member-private
    /// record belongs to its owner alone.
    PrivateRecord,
    /// The optimistic-concurrency law (kernel §3): the expected version
    /// did not match — the mutation is refused, never a silent
    /// lost-update.
    VersionConflict,
}

/// The effect of one explicit permission grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum GrantEffect {
    /// Explicitly allow the actor on the surface at this level
    /// (elevating them above the role default).
    Allow {
        /// The granted access level.
        level: AccessLevel,
    },
    /// Explicitly deny the surface for this actor (overriding any role
    /// default and any allow grant).
    Deny,
}

/// One explicit permission grant: an allow/deny override for one actor
/// on one surface family, scoped to one workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionGrant {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The workspace this grant belongs to.
    pub workspace: WorkspaceRef,
    /// The actor the grant applies to.
    pub actor: ActorRef,
    /// The surface family the grant overrides.
    pub surface: SurfaceFamily,
    /// The grant's effect.
    pub effect: GrantEffect,
}

impl PermissionGrant {
    /// Builds a grant, validating the actor reference.
    pub fn new(
        workspace: WorkspaceRef,
        actor: ActorRef,
        surface: SurfaceFamily,
        effect: GrantEffect,
    ) -> Result<Self, CollabError> {
        actor.validate()?;
        Ok(Self {
            v: CollabVersion,
            workspace,
            actor,
            surface,
            effect,
        })
    }

    /// Validates the grant's bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        self.actor.validate()?;
        Ok(())
    }
}

/// Validates a grant list: bounded, every grant well-formed, and at most
/// one grant per (actor, surface) — two grants for the same actor on the
/// same surface would be a silent ambiguity the frozen precedence would
/// have to hide.
pub(crate) fn validate_grants(
    field: &'static str,
    grants: &[PermissionGrant],
) -> Result<(), CollabError> {
    ensure_list_bound(field, grants.len(), MAX_GRANTS)?;
    for grant in grants {
        grant.validate()?;
    }
    for (index, grant) in grants.iter().enumerate() {
        let duplicate = grants[..index].iter().any(|other| {
            other.workspace == grant.workspace
                && other.actor == grant.actor
                && other.surface == grant.surface
        });
        if duplicate {
            return Err(CollabError::invalid(format!(
                "{field} carries more than one grant for actor {:?} on {:?} in workspace {:?}",
                grant.actor.id, grant.surface, grant.workspace
            )));
        }
    }
    Ok(())
}

/// The authorization request: inputs as data (the flauz-cap pattern).
/// `target` names the member an action acts ON, when the action is
/// member-directed (invite, role change, removal) — it is what the
/// owner-protection rule checks. All references borrow from the caller;
/// nothing is copied or stored.
#[derive(Debug, Clone, Copy)]
pub struct AuthorizationRequest<'a> {
    /// The workspace whose collaboration state is being acted on.
    pub workspace: &'a WorkspaceRef,
    /// The actor requesting the action.
    pub actor: &'a ActorRef,
    /// The requested access level.
    pub action: AccessLevel,
    /// The surface family the action applies to.
    pub surface: SurfaceFamily,
    /// The member the action acts on, when member-directed.
    pub target: Option<&'a ActorRef>,
    /// The workspace roster.
    pub memberships: &'a [WorkspaceMembership],
    /// The explicit grants.
    pub grants: &'a [PermissionGrant],
    /// The role lattice.
    pub lattice: &'a RoleLattice,
}

/// The outcome of one authorization: allow with the named rule, or deny
/// with the named reason (plus the actor's role and the role's level,
/// when known — the WHY the permission-gated affordances render).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum DecisionOutcome {
    /// The action is allowed.
    Allow {
        /// The role that admitted it (the actor's role).
        role: Role,
        /// The named rule that admitted it.
        via: AllowRule,
    },
    /// The action is denied.
    Deny {
        /// The named reason (which rule denied — never silent).
        reason: DenialReason,
        /// The actor's role, when a membership was found.
        role: Option<Role>,
        /// The role's lattice level on this surface, when applicable.
        role_level: Option<AccessLevel>,
    },
}

impl DecisionOutcome {
    /// Whether the outcome allows the action.
    #[must_use]
    pub const fn allows(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    /// The named denial reason, or `None` when allowed.
    #[must_use]
    pub const fn denial_reason(&self) -> Option<DenialReason> {
        match self {
            Self::Allow { .. } => None,
            Self::Deny { reason, .. } => Some(*reason),
        }
    }
}

/// The authorization decision record: the durable answer to "may this
/// actor do this here, and if not, why?" — the evidence type the
/// members surface and the two-actor simulation build on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationDecision {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The workspace the decision applies to.
    pub workspace: WorkspaceRef,
    /// The actor the decision is about.
    pub actor: ActorRef,
    /// The surface family.
    pub surface: SurfaceFamily,
    /// The requested access level.
    pub action: AccessLevel,
    /// The outcome with its named rule.
    pub outcome: DecisionOutcome,
}

/// The authorization evaluator: inputs as data → Allow/Deny with the
/// NAMED reason (which rule decided — never a silent no-op).
///
/// # Errors
///
/// Returns [`CollabError`] when the inputs are themselves invalid
/// (malformed roster, grants or lattice) — an invalid input set is a
/// caller bug, not a denial.
pub fn authorize(request: &AuthorizationRequest<'_>) -> Result<AuthorizationDecision, CollabError> {
    validate_roster("memberships", request.memberships)?;
    validate_grants("grants", request.grants)?;
    request.lattice.validate()?;
    request.actor.validate()?;

    let membership = request.memberships.iter().find(|membership| {
        membership.workspace == *request.workspace && membership.actor == *request.actor
    });
    let Some(membership) = membership else {
        return Ok(request.decision(DecisionOutcome::Deny {
            reason: DenialReason::NotAMember,
            role: None,
            role_level: None,
        }));
    };
    let role = membership.role;

    // The owner-protection rule: acting ON the owner's membership (a
    // member-directed manage action whose target is the owner) requires
    // the actor to BE the owner. This is the one place admin differs
    // from owner.
    if request.surface == SurfaceFamily::Members
        && request.action == AccessLevel::Manage
        && role != Role::Owner
    {
        let target_is_owner = request.target.is_some_and(|target| {
            request.memberships.iter().any(|candidate| {
                candidate.workspace == *request.workspace
                    && candidate.actor == *target
                    && candidate.role == Role::Owner
            })
        });
        if target_is_owner {
            return Ok(request.decision(DecisionOutcome::Deny {
                reason: DenialReason::OwnerProtected,
                role: Some(role),
                role_level: request.lattice.level(role, SurfaceFamily::Members),
            }));
        }
    }

    // Grants for this actor on this surface in this workspace.
    let matching = request.grants.iter().filter(|grant| {
        grant.workspace == *request.workspace
            && grant.actor == *request.actor
            && grant.surface == request.surface
    });
    if matching
        .clone()
        .any(|grant| grant.effect == GrantEffect::Deny)
    {
        return Ok(request.decision(DecisionOutcome::Deny {
            reason: DenialReason::ExplicitDeny,
            role: Some(role),
            role_level: request.lattice.level(role, request.surface),
        }));
    }
    let allow_level = matching
        .filter_map(|grant| match grant.effect {
            GrantEffect::Allow { level } => Some(level),
            GrantEffect::Deny => None,
        })
        .max();
    if allow_level.is_some_and(|level| level >= request.action) {
        return Ok(request.decision(DecisionOutcome::Allow {
            role,
            via: AllowRule::ExplicitAllow,
        }));
    }

    // The role default — with its named gaps.
    match request.lattice.level(role, request.surface) {
        Some(role_level) if role_level >= request.action => {
            Ok(request.decision(DecisionOutcome::Allow {
                role,
                via: AllowRule::RoleDefault,
            }))
        }
        Some(role_level) => Ok(request.decision(DecisionOutcome::Deny {
            reason: DenialReason::RoleDenied,
            role: Some(role),
            role_level: Some(role_level),
        })),
        None => Ok(request.decision(DecisionOutcome::Deny {
            reason: DenialReason::RoleNotInLattice,
            role: Some(role),
            role_level: None,
        })),
    }
}

impl<'a> AuthorizationRequest<'a> {
    fn decision(&self, outcome: DecisionOutcome) -> AuthorizationDecision {
        AuthorizationDecision {
            v: CollabVersion,
            workspace: self.workspace.clone(),
            actor: self.actor.clone(),
            surface: self.surface,
            action: self.action,
            outcome,
        }
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

    #[test]
    fn grants_serialize_canonically_and_reject_unknown_fields() {
        let allow = ok(PermissionGrant::new(
            workspace(),
            ok(ActorRef::user("dev")),
            SurfaceFamily::Environments,
            GrantEffect::Allow {
                level: AccessLevel::Contribute,
            },
        ));
        let serialized = ok(serde_json::to_string(&allow));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,",
                "\"workspace\":\"ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0\",",
                "\"actor\":{\"kind\":\"user\",\"id\":\"dev\"},",
                "\"surface\":\"environments\",",
                "\"effect\":{\"kind\":\"allow\",\"level\":\"contribute\"}}"
            )
        );
        let reloaded: PermissionGrant = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, allow);

        let deny = ok(PermissionGrant::new(
            workspace(),
            ok(ActorRef::user("mira")),
            SurfaceFamily::Environments,
            GrantEffect::Deny,
        ));
        let serialized = ok(serde_json::to_string(&deny));
        assert!(serialized.contains("\"effect\":{\"kind\":\"deny\"}"));
        assert!(
            serde_json::from_str::<PermissionGrant>(
                &serialized.replace("\"surface\":", "\"family\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn decisions_serialize_canonically_and_carry_the_named_rule() {
        let memberships = [membership("ana", "Ana", Role::Owner)];
        let lattice = RoleLattice::default_lattice();
        let request = AuthorizationRequest {
            workspace: &workspace(),
            actor: &ok(ActorRef::user("ana")),
            action: AccessLevel::Manage,
            surface: SurfaceFamily::Members,
            target: None,
            memberships: &memberships,
            grants: &[],
            lattice: &lattice,
        };
        let decision = ok(authorize(&request));
        let serialized = ok(serde_json::to_string(&decision));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,",
                "\"workspace\":\"ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0\",",
                "\"actor\":{\"kind\":\"user\",\"id\":\"ana\"},",
                "\"surface\":\"members\",",
                "\"action\":\"manage\",",
                "\"outcome\":{\"kind\":\"allow\",\"role\":\"owner\",\"via\":\"role_default\"}}"
            )
        );
        let reloaded: AuthorizationDecision = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, decision);
        assert!(decision.outcome.allows());
        assert_eq!(decision.outcome.denial_reason(), None);
    }

    #[test]
    fn every_denial_is_named_never_a_silent_no_op() {
        // A denial always carries one of the frozen reasons — the
        // no-silent-no-op law (the CAP-001 pattern applied to access).
        let memberships = [
            membership("ana", "Ana", Role::Owner),
            membership("dev", "Dev", Role::Contributor),
            membership("mira", "Mira", Role::Viewer),
        ];
        let grants = [ok(PermissionGrant::new(
            workspace(),
            ok(ActorRef::user("mira")),
            SurfaceFamily::Environments,
            GrantEffect::Deny,
        ))];
        let lattice = RoleLattice::default_lattice();
        let cases: Vec<(&str, AccessLevel, SurfaceFamily, Option<&str>, DenialReason)> = vec![
            (
                "outsider",
                AccessLevel::Read,
                SurfaceFamily::Tasks,
                None,
                DenialReason::NotAMember,
            ),
            (
                "dev",
                AccessLevel::Manage,
                SurfaceFamily::Members,
                Some("ana"),
                DenialReason::OwnerProtected,
            ),
            (
                "dev",
                AccessLevel::Manage,
                SurfaceFamily::Tasks,
                None,
                DenialReason::RoleDenied,
            ),
            (
                "mira",
                AccessLevel::Read,
                SurfaceFamily::Environments,
                None,
                DenialReason::ExplicitDeny,
            ),
        ];
        for (actor, action, surface, target, expected) in cases {
            let actor = ok(ActorRef::user(actor));
            let target = target.map(|id| ok(ActorRef::user(id)));
            let request = AuthorizationRequest {
                workspace: &workspace(),
                actor: &actor,
                action,
                surface,
                target: target.as_ref(),
                memberships: &memberships,
                grants: &grants,
                lattice: &lattice,
            };
            let decision = ok(authorize(&request));
            assert!(!decision.outcome.allows());
            assert_eq!(
                decision.outcome.denial_reason(),
                Some(expected),
                "actor {actor:?} on {surface:?} at {action:?} must be denied with {expected:?}"
            );
        }
    }

    #[test]
    fn explicit_grants_override_role_defaults_in_both_directions() {
        let memberships = [
            membership("ana", "Ana", Role::Owner),
            membership("dev", "Dev", Role::Contributor),
            membership("mira", "Mira", Role::Viewer),
        ];
        let lattice = RoleLattice::default_lattice();
        // Mira (viewer: environments read) is explicitly elevated to
        // contribute; Dev (contributor: provider accounts read) is
        // explicitly elevated to manage.
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
                SurfaceFamily::ProviderAccounts,
                GrantEffect::Allow {
                    level: AccessLevel::Manage,
                },
            )),
        ];
        let elevated = AuthorizationRequest {
            workspace: &workspace(),
            actor: &ok(ActorRef::user("mira")),
            action: AccessLevel::Contribute,
            surface: SurfaceFamily::Environments,
            target: None,
            memberships: &memberships,
            grants: &grants,
            lattice: &lattice,
        };
        let decision = ok(authorize(&elevated));
        assert_eq!(
            decision.outcome,
            DecisionOutcome::Allow {
                role: Role::Viewer,
                via: AllowRule::ExplicitAllow,
            }
        );
        // The grant covers lower actions too — an explicit allow at
        // contribute admits reading through the SAME named rule (the
        // grant overrides the role default within its level).
        let read = AuthorizationRequest {
            action: AccessLevel::Read,
            ..elevated
        };
        let decision = ok(authorize(&read));
        assert_eq!(
            decision.outcome,
            DecisionOutcome::Allow {
                role: Role::Viewer,
                via: AllowRule::ExplicitAllow,
            }
        );
        let managed = AuthorizationRequest {
            workspace: &workspace(),
            actor: &ok(ActorRef::user("dev")),
            action: AccessLevel::Manage,
            surface: SurfaceFamily::ProviderAccounts,
            target: None,
            memberships: &memberships,
            grants: &grants,
            lattice: &lattice,
        };
        let decision = ok(authorize(&managed));
        assert_eq!(
            decision.outcome,
            DecisionOutcome::Allow {
                role: Role::Contributor,
                via: AllowRule::ExplicitAllow,
            }
        );
    }

    #[test]
    fn deny_grants_beat_allow_grants_and_role_defaults() {
        let memberships = [membership("ana", "Ana", Role::Owner)];
        let lattice = RoleLattice::default_lattice();
        // The evaluator's input validation rejects two grants for the
        // same (actor, surface) — the ambiguity is refused up front, so
        // deny-vs-allow precedence can never hide a silent conflict.
        let conflicting = [
            ok(PermissionGrant::new(
                workspace(),
                ok(ActorRef::user("ana")),
                SurfaceFamily::Tasks,
                GrantEffect::Deny,
            )),
            ok(PermissionGrant::new(
                workspace(),
                ok(ActorRef::user("ana")),
                SurfaceFamily::Tasks,
                GrantEffect::Allow {
                    level: AccessLevel::Manage,
                },
            )),
        ];
        let request = AuthorizationRequest {
            workspace: &workspace(),
            actor: &ok(ActorRef::user("ana")),
            action: AccessLevel::Read,
            surface: SurfaceFamily::Tasks,
            target: None,
            memberships: &memberships,
            grants: &conflicting,
            lattice: &lattice,
        };
        assert!(authorize(&request).is_err());
        // A single deny grant overrides even the owner's role default.
        let deny_only = [ok(PermissionGrant::new(
            workspace(),
            ok(ActorRef::user("ana")),
            SurfaceFamily::Tasks,
            GrantEffect::Deny,
        ))];
        let request = AuthorizationRequest {
            grants: &deny_only,
            ..request
        };
        let decision = ok(authorize(&request));
        assert_eq!(
            decision.outcome.denial_reason(),
            Some(DenialReason::ExplicitDeny)
        );
    }

    #[test]
    fn a_role_missing_from_the_lattice_is_a_named_gap() {
        // The partial lattice covers only owner; dev's contributor role
        // has no row — the gap fires with its name, never a silent
        // default.
        let memberships = [membership("dev", "Dev", Role::Contributor)];
        let partial = ok(RoleLattice::new(vec![
            crate::membership::RoleDefaults::new(
                Role::Owner,
                [
                    AccessLevel::Manage,
                    AccessLevel::Manage,
                    AccessLevel::Manage,
                    AccessLevel::Manage,
                ],
            ),
        ]));
        let request = AuthorizationRequest {
            workspace: &workspace(),
            actor: &ok(ActorRef::user("dev")),
            action: AccessLevel::Read,
            surface: SurfaceFamily::Tasks,
            target: None,
            memberships: &memberships,
            grants: &[],
            lattice: &partial,
        };
        let decision = ok(authorize(&request));
        assert_eq!(
            decision.outcome.denial_reason(),
            Some(DenialReason::RoleNotInLattice),
            "a role with no lattice row denies with the named gap, never a silent default"
        );
    }

    #[test]
    fn foreign_workspace_records_never_match() {
        let other = ok(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPC2"));
        let memberships = [
            membership("ana", "Ana", Role::Owner),
            WorkspaceMembership {
                workspace: other,
                actor: ok(ActorRef::user("dev")),
                display_name: "Dev".to_owned(),
                role: Role::Owner,
                v: CollabVersion,
            },
        ];
        let lattice = RoleLattice::default_lattice();
        let request = AuthorizationRequest {
            workspace: &workspace(),
            actor: &ok(ActorRef::user("dev")),
            action: AccessLevel::Read,
            surface: SurfaceFamily::Tasks,
            target: None,
            memberships: &memberships,
            grants: &[],
            lattice: &lattice,
        };
        // Dev's membership belongs to another workspace: in THIS
        // workspace dev is not a member — the named denial.
        let decision = ok(authorize(&request));
        assert_eq!(
            decision.outcome.denial_reason(),
            Some(DenialReason::NotAMember)
        );
    }
}
