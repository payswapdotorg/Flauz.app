//! Workspace membership and the role lattice (work order COL-001 part 1):
//! who belongs to a workspace, under which [`Role`], and what each role
//! may do per surface family — carried as **data** in a [`RoleLattice`].
//!
//! The lattice is a record, not code: the four frozen roles
//! (owner / admin / contributor / viewer) map to an [`AccessLevel`] per
//! [`SurfaceFamily`] (tasks / provider accounts / environments /
//! members). The frozen default lattice ([`RoleLattice::default_lattice`])
//! encodes the platform's shipping defaults:
//!
//! | Role | tasks | provider accounts | environments | members |
//! |---|---|---|---|---|
//! | owner | manage | manage | manage | manage |
//! | admin | manage | manage | manage | manage |
//! | contributor | contribute | read | contribute | read |
//! | viewer | read | none | read | read |
//!
//! A lattice that does not cover a role is legal data (a named gap: the
//! authorization evaluator denies with the `role_not_in_lattice`
//! reason, never a silent no-op). The evaluator's owner-protection rule
//! — only the owner may act on the owner's membership — is carried by
//! the evaluator, not the lattice; admin differs from owner exactly
//! there.

use serde::{Deserialize, Serialize};

use crate::refs::{ActorRef, WorkspaceRef};
use crate::{
    CollabError, CollabVersion, MAX_LATTICE_ROLES, MAX_MEMBERS, SURFACE_FAMILIES,
    ensure_list_bound, ensure_name,
};

/// The frozen workspace roles (kernel snake_case wire names).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The workspace's accountable owner. Only the owner may act on the
    /// owner's own membership (the evaluator's owner-protection rule).
    Owner,
    /// Manages every surface family, but cannot act on the owner's
    /// membership.
    Admin,
    /// Works on tasks and environments; reads the attribution surfaces.
    Contributor,
    /// Sees tasks, environments and the member list; changes nothing.
    Viewer,
}

impl Role {
    /// Every frozen role, in canonical order.
    pub const ALL: [Self; 4] = [Self::Owner, Self::Admin, Self::Contributor, Self::Viewer];

    /// The user-facing label (the members surface renders these words).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Admin => "Admin",
            Self::Contributor => "Contributor",
            Self::Viewer => "Viewer",
        }
    }
}

/// The frozen surface families the lattice speaks about (COL-001's
/// four: tasks, provider accounts, environments, members).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceFamily {
    /// Tasks: the workspace's chats/projects and their work.
    Tasks,
    /// Provider accounts: whose account work draws on (attribution is
    /// visible; connecting and changing stay managed).
    ProviderAccounts,
    /// Environments: the execution surfaces tasks attach.
    Environments,
    /// Members: the roster itself — seeing it, inviting, changing roles.
    Members,
}

impl SurfaceFamily {
    /// Every frozen surface family, in canonical order.
    pub const ALL: [Self; 4] = [
        Self::Tasks,
        Self::ProviderAccounts,
        Self::Environments,
        Self::Members,
    ];

    /// The user-facing label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tasks => "tasks",
            Self::ProviderAccounts => "provider accounts",
            Self::Environments => "environments",
            Self::Members => "members",
        }
    }
}

/// What a role may do on a surface family. The variant order IS the
/// lattice order: `none < read < contribute < manage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    /// No access — the surface is not visible to this role.
    None,
    /// See the surface's records.
    Read,
    /// See and work on the surface (create, edit, run).
    Contribute,
    /// Full control, including the surface's settings and sharing.
    Manage,
}

impl AccessLevel {
    /// Every level, in lattice order.
    pub const ALL: [Self; 4] = [Self::None, Self::Read, Self::Contribute, Self::Manage];

    /// The user-facing verb for requesting this level.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "no access",
            Self::Read => "see",
            Self::Contribute => "work on",
            Self::Manage => "manage",
        }
    }
}

/// One workspace membership: a member identity ref (the frozen actor
/// grammar as data), a display name, and the member's role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceMembership {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The workspace this roster entry belongs to.
    pub workspace: WorkspaceRef,
    /// The member's identity ref (the frozen actor grammar, as data).
    pub actor: ActorRef,
    /// The member's display name (user words; never credential
    /// material).
    pub display_name: String,
    /// The member's role.
    pub role: Role,
}

impl WorkspaceMembership {
    /// Builds a membership, validating the display name's bounds.
    pub fn new(
        workspace: WorkspaceRef,
        actor: ActorRef,
        display_name: &str,
        role: Role,
    ) -> Result<Self, CollabError> {
        ensure_name("display name", display_name)?;
        actor.validate()?;
        Ok(Self {
            v: CollabVersion,
            workspace,
            actor,
            display_name: display_name.to_owned(),
            role,
        })
    }

    /// Validates the membership's bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        self.actor.validate()?;
        ensure_name("display name", &self.display_name)?;
        Ok(())
    }
}

/// One role's default access on one surface family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceAccess {
    /// The surface family.
    pub surface: SurfaceFamily,
    /// The role's default access level on it.
    pub level: AccessLevel,
}

/// One role's row in the lattice: its default access on every surface
/// family, in canonical family order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleDefaults {
    /// The role this row describes.
    pub role: Role,
    /// The role's access per surface family, in canonical order
    /// ([`SurfaceFamily::ALL`]), each family exactly once.
    pub access: Vec<SurfaceAccess>,
}

impl RoleDefaults {
    /// Builds one role's row from an ordered level list (one per family,
    /// canonical order).
    pub fn new(role: Role, levels: [AccessLevel; SURFACE_FAMILIES]) -> Self {
        Self {
            role,
            access: SurfaceFamily::ALL
                .iter()
                .zip(levels)
                .map(|(surface, level)| SurfaceAccess {
                    surface: *surface,
                    level,
                })
                .collect(),
        }
    }

    /// Validates the row: every frozen family exactly once, in canonical
    /// order.
    pub fn validate(&self) -> Result<(), CollabError> {
        ensure_list_bound("role access rows", self.access.len(), SURFACE_FAMILIES)?;
        let canonical: Vec<SurfaceFamily> = SurfaceFamily::ALL.to_vec();
        let found: Vec<SurfaceFamily> = self.access.iter().map(|row| row.surface).collect();
        if found != canonical {
            return Err(CollabError::invalid(format!(
                "the {:?} role row must cover every surface family exactly once, in canonical \
                 order (tasks, provider_accounts, environments, members)",
                self.role
            )));
        }
        Ok(())
    }
}

/// The role lattice as DATA: what each role may do per surface family.
/// The frozen default ([`RoleLattice::default_lattice`]) encodes the
/// platform's shipping defaults; a workspace may carry its own record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleLattice {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// One row per role, in canonical role order; at most one row per
    /// role. A role with no row is a named gap (the evaluator denies
    /// with the `role_not_in_lattice` reason).
    pub roles: Vec<RoleDefaults>,
}

impl RoleLattice {
    /// The frozen default lattice: the platform's shipping defaults per
    /// role and surface family.
    #[must_use]
    pub fn default_lattice() -> Self {
        Self {
            v: CollabVersion,
            roles: vec![
                RoleDefaults::new(
                    Role::Owner,
                    [
                        AccessLevel::Manage,
                        AccessLevel::Manage,
                        AccessLevel::Manage,
                        AccessLevel::Manage,
                    ],
                ),
                RoleDefaults::new(
                    Role::Admin,
                    [
                        AccessLevel::Manage,
                        AccessLevel::Manage,
                        AccessLevel::Manage,
                        AccessLevel::Manage,
                    ],
                ),
                RoleDefaults::new(
                    Role::Contributor,
                    [
                        AccessLevel::Contribute,
                        AccessLevel::Read,
                        AccessLevel::Contribute,
                        AccessLevel::Read,
                    ],
                ),
                RoleDefaults::new(
                    Role::Viewer,
                    [
                        AccessLevel::Read,
                        AccessLevel::None,
                        AccessLevel::Read,
                        AccessLevel::Read,
                    ],
                ),
            ],
        }
    }

    /// Builds a lattice from role rows, validating canonical shape.
    pub fn new(roles: Vec<RoleDefaults>) -> Result<Self, CollabError> {
        let lattice = Self {
            v: CollabVersion,
            roles,
        };
        lattice.validate()?;
        Ok(lattice)
    }

    /// Validates the lattice: bounded, each row well-formed, at most one
    /// row per role.
    pub fn validate(&self) -> Result<(), CollabError> {
        ensure_list_bound("lattice roles", self.roles.len(), MAX_LATTICE_ROLES)?;
        for row in &self.roles {
            row.validate()?;
        }
        for (index, row) in self.roles.iter().enumerate() {
            if self.roles[..index]
                .iter()
                .any(|other| other.role == row.role)
            {
                return Err(CollabError::invalid(format!(
                    "the {:?} role appears more than once in the lattice",
                    row.role
                )));
            }
        }
        Ok(())
    }

    /// The role's default access level on a surface family, or `None`
    /// when the lattice carries no row for the role (the named gap the
    /// evaluator denies with).
    #[must_use]
    pub fn level(&self, role: Role, surface: SurfaceFamily) -> Option<AccessLevel> {
        self.roles
            .iter()
            .find(|row| row.role == role)?
            .access
            .iter()
            .find(|row| row.surface == surface)
            .map(|row| row.level)
    }
}

/// Validates a roster: bounded, and at most one membership per actor per
/// workspace (a member with two roles is a contract violation — the
/// evaluator resolves the FIRST match, so duplicates would be a silent
/// ambiguity).
pub(crate) fn validate_roster(
    field: &'static str,
    roster: &[WorkspaceMembership],
) -> Result<(), CollabError> {
    ensure_list_bound(field, roster.len(), MAX_MEMBERS)?;
    for membership in roster {
        membership.validate()?;
    }
    for (index, membership) in roster.iter().enumerate() {
        let duplicate = roster[..index].iter().any(|other| {
            other.workspace == membership.workspace && other.actor == membership.actor
        });
        if duplicate {
            return Err(CollabError::invalid(format!(
                "{field} carries more than one membership for actor {:?} in workspace {:?}",
                membership.actor.id, membership.workspace
            )));
        }
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

    fn workspace() -> WorkspaceRef {
        ok(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0"))
    }

    #[test]
    fn roles_and_families_serialize_snake_case() {
        assert_eq!(ok(serde_json::to_string(&Role::Owner)), "\"owner\"");
        assert_eq!(
            ok(serde_json::to_string(&SurfaceFamily::ProviderAccounts)),
            "\"provider_accounts\""
        );
        assert_eq!(
            ok(serde_json::to_string(&AccessLevel::Contribute)),
            "\"contribute\""
        );
        assert_eq!(Role::ALL.len(), 4);
        assert_eq!(SurfaceFamily::ALL.len(), SURFACE_FAMILIES);
        assert_eq!(
            AccessLevel::ALL,
            [
                AccessLevel::None,
                AccessLevel::Read,
                AccessLevel::Contribute,
                AccessLevel::Manage
            ]
        );
    }

    #[test]
    fn memberships_serialize_canonically_and_reject_unknown_fields() {
        let membership = ok(WorkspaceMembership::new(
            workspace(),
            ok(ActorRef::user("ana")),
            "Ana",
            Role::Owner,
        ));
        let serialized = ok(serde_json::to_string(&membership));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,",
                "\"workspace\":\"ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0\",",
                "\"actor\":{\"kind\":\"user\",\"id\":\"ana\"},",
                "\"display_name\":\"Ana\",",
                "\"role\":\"owner\"}"
            )
        );
        let reloaded: WorkspaceMembership = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, membership);
        assert!(
            serde_json::from_str::<WorkspaceMembership>(
                &serialized.replace("\"display_name\":", "\"name\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
        assert!(
            WorkspaceMembership::new(workspace(), ok(ActorRef::user("ana")), "", Role::Owner)
                .is_err()
        );
    }

    #[test]
    fn the_default_lattice_is_canonical_and_covers_every_role() {
        let lattice = RoleLattice::default_lattice();
        ok(lattice.validate());
        for role in Role::ALL {
            for surface in SurfaceFamily::ALL {
                assert!(
                    lattice.level(role, surface).is_some(),
                    "the default lattice covers {role:?} on {surface:?}"
                );
            }
        }
        // The frozen shipping defaults.
        assert_eq!(
            lattice.level(Role::Contributor, SurfaceFamily::Tasks),
            Some(AccessLevel::Contribute)
        );
        assert_eq!(
            lattice.level(Role::Contributor, SurfaceFamily::ProviderAccounts),
            Some(AccessLevel::Read)
        );
        assert_eq!(
            lattice.level(Role::Viewer, SurfaceFamily::ProviderAccounts),
            Some(AccessLevel::None)
        );
        assert_eq!(
            lattice.level(Role::Viewer, SurfaceFamily::Members),
            Some(AccessLevel::Read)
        );
        assert_eq!(
            lattice.level(Role::Admin, SurfaceFamily::Members),
            Some(AccessLevel::Manage)
        );
    }

    #[test]
    fn lattices_reject_duplicates_and_malformed_rows() {
        let lattice = RoleLattice::default_lattice();
        // Duplicate role rows are rejected.
        let duplicated = RoleLattice {
            roles: lattice
                .roles
                .iter()
                .chain(std::iter::once(&lattice.roles[0]))
                .cloned()
                .collect(),
            ..lattice.clone()
        };
        assert!(duplicated.validate().is_err());
        // A row missing a family is rejected.
        let missing_family = RoleDefaults {
            role: Role::Owner,
            access: vec![SurfaceAccess {
                surface: SurfaceFamily::Tasks,
                level: AccessLevel::Manage,
            }],
        };
        assert!(missing_family.validate().is_err());
        // A row out of canonical family order is rejected.
        let reordered = RoleDefaults {
            role: Role::Owner,
            access: vec![
                SurfaceAccess {
                    surface: SurfaceFamily::Members,
                    level: AccessLevel::Manage,
                },
                SurfaceAccess {
                    surface: SurfaceFamily::Tasks,
                    level: AccessLevel::Manage,
                },
                SurfaceAccess {
                    surface: SurfaceFamily::ProviderAccounts,
                    level: AccessLevel::Manage,
                },
                SurfaceAccess {
                    surface: SurfaceFamily::Environments,
                    level: AccessLevel::Manage,
                },
            ],
        };
        assert!(reordered.validate().is_err());
        // A partial lattice (one role only) is legal data — the named
        // gap stays named at evaluation time.
        let partial = RoleLattice {
            v: CollabVersion,
            roles: vec![RoleDefaults::new(
                Role::Owner,
                [
                    AccessLevel::Manage,
                    AccessLevel::Manage,
                    AccessLevel::Manage,
                    AccessLevel::Manage,
                ],
            )],
        };
        ok(partial.validate());
        assert_eq!(
            partial.level(Role::Viewer, SurfaceFamily::Tasks),
            None,
            "a role with no row is a named gap, never a silent default"
        );
    }
}
