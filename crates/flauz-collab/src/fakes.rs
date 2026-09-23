//! Public in-memory fakes: the deterministic conformance surface for
//! workspace collaboration (kernel §7).
//!
//! Everything here is a pure function of its arguments — no I/O, no
//! wall-clock reads, no randomness, no generated identifiers. The
//! canonical fake family covers the shapes the members surface, the
//! authorization evaluator and the F9 wave gate drive:
//!
//! - [`fake_owner`] / [`fake_contributor`] — the two actors of the F9
//!   simulation (Ana owns the workspace; Dev contributes to it);
//! - [`fake_memberships`] — the two-actor roster;
//! - [`fake_default_lattice`] — the frozen default lattice (the
//!   reference semantics for real lattices);
//! - [`fake_grants`] — the explicit grant family for the evaluator's
//!   grant tests (one elevation, one deny);
//! - [`fake_simulation_inputs`] — the deterministic F9 inputs (roster,
//!   lattice, initial shared state, the isolated-by-default posture, a
//!   fixed clock);
//! - [`fake_interleaving_table`] — the scripted two-actor interleaving:
//!   Ana and Dev driving the same store-facing seam — interleaved
//!   writes (one version conflict, one recovered landing), a private
//!   note that never crosses projections, presence, and the sharing
//!   posture (denied for the contributor, stated by the owner).

use crate::membership::{AccessLevel, Role, RoleLattice, SurfaceFamily, WorkspaceMembership};
use crate::permission::{
    AllowRule, AuthorizationRequest, DecisionOutcome, DenialReason, GrantEffect, PermissionGrant,
    authorize,
};
use crate::policy::{ContextVisibility, SharedFilesystemMode, Visibility, WorktreePolicy};
use crate::presence::SurfaceLocator;
use crate::refs::{ActorRef, TaskRef, WorkspaceRef};
use crate::simulator::{
    ExpectedOutcome, InterleavedStep, InterleavingTable, SimulatedOperation, SimulationInputs,
    StoreEntry,
};
use crate::time::Timestamp;

/// The fake workspace the canonical fake family collaborates in.
pub const FAKE_WORKSPACE_ID: &str = "ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0";
/// The fake task the sharing posture and the private note belong to.
pub const FAKE_TASK_ID: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";

/// The F9 simulation's owner principal (Ana).
pub const FAKE_OWNER_ID: &str = "ana";
/// The F9 simulation's contributor principal (Dev).
pub const FAKE_CONTRIBUTOR_ID: &str = "dev";
/// The evaluator grant tests' viewer principal (Mira).
pub const FAKE_VIEWER_ID: &str = "mira";

/// The deterministic clock the canonical fake simulation runs at.
pub const FAKE_NOW: &str = "2026-09-23T10:00:00Z";
/// The freshness window the canonical fake simulation uses
/// (milliseconds).
pub const FAKE_PRESENCE_STALE_AFTER_MS: u64 = 60_000;

/// The shared brief entry the simulation interleaves writes on.
pub const FAKE_BRIEF_KEY: &str = "brief";
/// The contributor's member-private note.
pub const FAKE_PRIVATE_NOTE_KEY: &str = "dev-notes";

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("the fake collaboration family is canonically valid: {error}"),
    }
}

/// The fake workspace the canonical fake family collaborates in.
#[must_use]
pub fn fake_workspace() -> WorkspaceRef {
    ok(WorkspaceRef::parse(FAKE_WORKSPACE_ID))
}

/// The fake task the sharing posture and the private note belong to.
#[must_use]
pub fn fake_task() -> TaskRef {
    ok(TaskRef::parse(FAKE_TASK_ID))
}

/// The F9 simulation's owner (Ana — a `user` principal string, the
/// frozen actor grammar as data).
#[must_use]
pub fn fake_owner() -> ActorRef {
    ok(ActorRef::user(FAKE_OWNER_ID))
}

/// The F9 simulation's contributor (Dev).
#[must_use]
pub fn fake_contributor() -> ActorRef {
    ok(ActorRef::user(FAKE_CONTRIBUTOR_ID))
}

/// The evaluator grant tests' viewer (Mira).
#[must_use]
pub fn fake_viewer() -> ActorRef {
    ok(ActorRef::user(FAKE_VIEWER_ID))
}

/// The two-actor roster: Ana owns the workspace, Dev contributes to it.
#[must_use]
pub fn fake_memberships() -> Vec<WorkspaceMembership> {
    vec![
        ok(WorkspaceMembership::new(
            fake_workspace(),
            fake_owner(),
            "Ana",
            Role::Owner,
        )),
        ok(WorkspaceMembership::new(
            fake_workspace(),
            fake_contributor(),
            "Dev",
            Role::Contributor,
        )),
    ]
}

/// The frozen default lattice — the reference semantics for real
/// lattices ([`RoleLattice::default_lattice`]).
#[must_use]
pub fn fake_default_lattice() -> RoleLattice {
    RoleLattice::default_lattice()
}

/// The explicit grant family for the evaluator's grant tests: Mira (a
/// viewer) is explicitly elevated on environments to contribute; Dev is
/// explicitly elevated on provider accounts to manage.
#[must_use]
pub fn fake_grants() -> Vec<PermissionGrant> {
    vec![
        ok(PermissionGrant::new(
            fake_workspace(),
            fake_viewer(),
            SurfaceFamily::Environments,
            GrantEffect::Allow {
                level: AccessLevel::Contribute,
            },
        )),
        ok(PermissionGrant::new(
            fake_workspace(),
            fake_contributor(),
            SurfaceFamily::ProviderAccounts,
            GrantEffect::Allow {
                level: AccessLevel::Manage,
            },
        )),
    ]
}

/// The initial shared state of the canonical fake simulation: Ana's
/// brief at version 1, workspace-shared.
#[must_use]
pub fn fake_initial_entries() -> Vec<StoreEntry> {
    vec![ok(StoreEntry::new(
        FAKE_BRIEF_KEY,
        "The weekly brief — first draft",
        1,
        fake_owner(),
        Visibility::WorkspaceShared,
    ))]
}

/// The deterministic F9 simulation inputs: the two-actor roster, the
/// default lattice, no grants, Ana's shared brief at version 1, the
/// isolated-by-default posture with member-private notes, and the fixed
/// clock.
#[must_use]
pub fn fake_simulation_inputs() -> SimulationInputs {
    SimulationInputs {
        v: crate::CollabVersion,
        workspace: fake_workspace(),
        memberships: fake_memberships(),
        grants: Vec::new(),
        lattice: fake_default_lattice(),
        entries: fake_initial_entries(),
        worktree: WorktreePolicy::isolated(&fake_task()),
        context: ContextVisibility::default_for(&fake_task()),
        presence_stale_after_ms: FAKE_PRESENCE_STALE_AFTER_MS,
        now: ok(Timestamp::parse(FAKE_NOW)),
    }
}

/// The scripted two-actor interleaving — the F9 gate law's canonical
/// story, twelve steps:
///
/// 1. Ana lands the brief's second version (version 2);
/// 2. Dev probes managing members — denied `role_denied`;
/// 3. Dev interleaves a stale write (expected version 1 against the
///    live 2) — denied `version_conflict`, **no lost update**;
/// 4. Dev reads the projection — sees the shared brief only;
/// 5. Dev re-reads and lands the brief's third version (the recovered
///    interleaving);
/// 6. Dev creates a member-private note;
/// 7. Ana reads her projection — Dev's private note is structurally
///    absent;
/// 8. Dev updates presence (viewing the task);
/// 9. Dev tries to change the sharing posture — denied `role_denied`;
/// 10. Ana states the sharing posture (shared files, read; workspace
///     notes);
/// 11. Ana probes managing members — allowed via the role default;
/// 12. Ana tries to write Dev's private note — denied `private_record`
///     (even the owner cannot touch another member's private records).
#[must_use]
pub fn fake_interleaving_table() -> InterleavingTable {
    let steps = vec![
        InterleavedStep {
            actor: fake_owner(),
            op: SimulatedOperation::WriteSharedState {
                key: FAKE_BRIEF_KEY.to_owned(),
                value: "The weekly brief — second draft".to_owned(),
                expected_version: 1,
                visibility: Visibility::WorkspaceShared,
            },
            expected: ExpectedOutcome::Written { version: 2 },
        },
        InterleavedStep {
            actor: fake_contributor(),
            op: SimulatedOperation::RequestAccess {
                surface: SurfaceFamily::Members,
                action: AccessLevel::Manage,
            },
            expected: ExpectedOutcome::Denied {
                reason: DenialReason::RoleDenied,
            },
        },
        InterleavedStep {
            actor: fake_contributor(),
            op: SimulatedOperation::WriteSharedState {
                key: FAKE_BRIEF_KEY.to_owned(),
                value: "Dev's stale overwrite".to_owned(),
                expected_version: 1,
                visibility: Visibility::WorkspaceShared,
            },
            expected: ExpectedOutcome::Denied {
                reason: DenialReason::VersionConflict,
            },
        },
        InterleavedStep {
            actor: fake_contributor(),
            op: SimulatedOperation::ReadProjection,
            expected: ExpectedOutcome::Projection {
                keys: vec![FAKE_BRIEF_KEY.to_owned()],
            },
        },
        InterleavedStep {
            actor: fake_contributor(),
            op: SimulatedOperation::WriteSharedState {
                key: FAKE_BRIEF_KEY.to_owned(),
                value: "The weekly brief — third draft with Dev's analysis".to_owned(),
                expected_version: 2,
                visibility: Visibility::WorkspaceShared,
            },
            expected: ExpectedOutcome::Written { version: 3 },
        },
        InterleavedStep {
            actor: fake_contributor(),
            op: SimulatedOperation::WriteSharedState {
                key: FAKE_PRIVATE_NOTE_KEY.to_owned(),
                value: "Dev's private scratchpad".to_owned(),
                expected_version: 0,
                visibility: Visibility::MemberPrivate,
            },
            expected: ExpectedOutcome::Written { version: 1 },
        },
        InterleavedStep {
            actor: fake_owner(),
            op: SimulatedOperation::ReadProjection,
            expected: ExpectedOutcome::Projection {
                keys: vec![FAKE_BRIEF_KEY.to_owned()],
            },
        },
        InterleavedStep {
            actor: fake_contributor(),
            op: SimulatedOperation::UpdatePresence {
                locator: SurfaceLocator::Task { task: fake_task() },
            },
            expected: ExpectedOutcome::PresenceUpdated,
        },
        InterleavedStep {
            actor: fake_contributor(),
            op: SimulatedOperation::SetSharing {
                shared_filesystem: SharedFilesystemMode::Write,
                memory: Visibility::WorkspaceShared,
                artifacts: Visibility::WorkspaceShared,
            },
            expected: ExpectedOutcome::Denied {
                reason: DenialReason::RoleDenied,
            },
        },
        InterleavedStep {
            actor: fake_owner(),
            op: SimulatedOperation::SetSharing {
                shared_filesystem: SharedFilesystemMode::Read,
                memory: Visibility::WorkspaceShared,
                artifacts: Visibility::WorkspaceShared,
            },
            expected: ExpectedOutcome::SharingRead {
                shared_filesystem: SharedFilesystemMode::Read,
                memory: Visibility::WorkspaceShared,
                artifacts: Visibility::WorkspaceShared,
            },
        },
        InterleavedStep {
            actor: fake_owner(),
            op: SimulatedOperation::RequestAccess {
                surface: SurfaceFamily::Members,
                action: AccessLevel::Manage,
            },
            expected: ExpectedOutcome::Allowed {
                via: AllowRule::RoleDefault,
            },
        },
        InterleavedStep {
            actor: fake_owner(),
            op: SimulatedOperation::WriteSharedState {
                key: FAKE_PRIVATE_NOTE_KEY.to_owned(),
                value: "The owner cannot write another member's private note".to_owned(),
                expected_version: 1,
                visibility: Visibility::MemberPrivate,
            },
            expected: ExpectedOutcome::Denied {
                reason: DenialReason::PrivateRecord,
            },
        },
    ];
    ok(InterleavingTable::new(fake_workspace(), fake_task(), steps))
}

/// One pre-authorization probe over the canonical fake family — a small
/// convenience for downstream waves wiring the evaluator.
#[must_use]
pub fn fake_authorization(
    actor: &ActorRef,
    surface: SurfaceFamily,
    action: AccessLevel,
) -> DecisionOutcome {
    let memberships = fake_memberships();
    let lattice = fake_default_lattice();
    let request = AuthorizationRequest {
        workspace: &fake_workspace(),
        actor,
        action,
        surface,
        target: None,
        memberships: &memberships,
        grants: &[],
        lattice: &lattice,
    };
    ok(authorize(&request)).outcome
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

    /// The canonical fake family is internally consistent: the inputs
    /// validate, the table validates, and the run reproduces every
    /// expected outcome.
    #[test]
    fn the_fake_family_runs_clean() {
        let inputs = fake_simulation_inputs();
        let table = fake_interleaving_table();
        let result = ok(crate::simulator::run_interleaved(&inputs, &table));
        ok(crate::simulator::verify_expectations(&table, &result));
        ok(crate::simulator::verify_f9_laws(&inputs, &table, &result));
        // The canonical story beats hold on the final state.
        assert_eq!(
            result
                .store
                .get(FAKE_BRIEF_KEY)
                .map_or(0, |entry| entry.version),
            3,
            "Ana's second draft and Dev's third draft both landed"
        );
        assert_eq!(
            result
                .store
                .get(FAKE_PRIVATE_NOTE_KEY)
                .map_or(String::new(), |entry| entry.value.clone()),
            "Dev's private scratchpad",
            "the owner's private-write attempt never touched the note"
        );
        assert_eq!(result.presence.records().len(), 1);
        assert_eq!(
            result.worktree.shared_filesystem,
            SharedFilesystemMode::Read,
            "the owner stated the shared-files posture; the contributor's attempt was denied"
        );
    }
}
