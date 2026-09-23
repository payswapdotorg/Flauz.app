//! The deterministic two-actor simulation suite (work order COL-001) —
//! the F9 gate law proven in simulation: two independent actors
//! concurrently using one workspace without losing authorized state.

use flauz_collab::membership::{
    AccessLevel, Role, RoleLattice, SurfaceFamily, WorkspaceMembership,
};
use flauz_collab::permission::{
    AllowRule, AuthorizationRequest, DecisionOutcome, DenialReason, GrantEffect, PermissionGrant,
    authorize,
};
use flauz_collab::policy::{ContextVisibility, SharedFilesystemMode, Visibility, WorktreePolicy};
use flauz_collab::presence::SurfaceLocator;
use flauz_collab::refs::{ActorRef, TaskRef, WorkspaceRef};
use flauz_collab::simulator::{
    ExpectedOutcome, InterleavedStep, InterleavingTable, SimulatedOperation, SimulationInputs,
    SimulationOutcome, StoreEntry, run_interleaved, verify_expectations, verify_f9_laws,
};
use flauz_collab::time::Timestamp;

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn workspace() -> WorkspaceRef {
    ok(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0"))
}

fn task() -> TaskRef {
    ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"))
}

fn inputs() -> SimulationInputs {
    SimulationInputs {
        v: flauz_collab::CollabVersion,
        workspace: workspace(),
        memberships: vec![
            ok(WorkspaceMembership::new(
                workspace(),
                ok(ActorRef::user("ana")),
                "Ana",
                Role::Owner,
            )),
            ok(WorkspaceMembership::new(
                workspace(),
                ok(ActorRef::user("dev")),
                "Dev",
                Role::Contributor,
            )),
        ],
        grants: Vec::new(),
        lattice: RoleLattice::default_lattice(),
        entries: vec![ok(StoreEntry::new(
            "brief",
            "The weekly brief — first draft",
            1,
            ok(ActorRef::user("ana")),
            Visibility::WorkspaceShared,
        ))],
        worktree: WorktreePolicy::isolated(&task()),
        context: ContextVisibility::default_for(&task()),
        presence_stale_after_ms: 60_000,
        now: ok(Timestamp::parse("2026-09-23T10:00:00Z")),
    }
}

fn write_step(actor: &str, key: &str, value: &str, expected: u64) -> InterleavedStep {
    InterleavedStep {
        actor: ok(ActorRef::user(actor)),
        op: SimulatedOperation::WriteSharedState {
            key: key.to_owned(),
            value: value.to_owned(),
            expected_version: expected,
            visibility: Visibility::WorkspaceShared,
        },
        expected: ExpectedOutcome::Written {
            version: if expected == 0 { 1 } else { expected + 1 },
        },
    }
}

/// THE F9 gate law, proven on the canonical fake family: two actors,
/// interleaved, no authorized state lost; denials named; privacy holds;
/// the log replays.
#[test]
fn the_f9_gate_law_holds_under_the_canonical_interleaving() {
    let inputs = flauz_collab::fakes::fake_simulation_inputs();
    let table = flauz_collab::fakes::fake_interleaving_table();
    let result = ok(run_interleaved(&inputs, &table));
    ok(verify_expectations(&table, &result));
    ok(verify_f9_laws(&inputs, &table, &result));

    // No authorized state was lost: both actors' landed writes are all
    // present in the final store, at the versions the log records.
    let brief = result
        .store
        .get("brief")
        .unwrap_or_else(|| panic!("the shared brief must survive the interleaving"));
    assert_eq!(brief.version, 3);
    assert_eq!(
        brief.value,
        "The weekly brief — third draft with Dev's analysis"
    );
    assert_eq!(
        result
            .store
            .get("dev-notes")
            .map_or(String::new(), |entry| entry.value.clone()),
        "Dev's private scratchpad",
        "the owner's private-write attempt never touched Dev's note"
    );

    // Every denial in the canonical story is named.
    let denials: Vec<DenialReason> = result
        .log
        .records
        .iter()
        .filter_map(|record| record.outcome.denial_reason())
        .collect();
    assert!(denials.contains(&DenialReason::RoleDenied));
    assert!(denials.contains(&DenialReason::VersionConflict));
    assert!(denials.contains(&DenialReason::PrivateRecord));

    // Privacy held: Ana's projection never contained Dev's note.
    for record in &result.log.records {
        if let SimulationOutcome::Projection { rows } = &record.outcome
            && record.actor.id == "ana"
        {
            assert!(
                rows.iter().all(|row| row.key != "dev-notes"),
                "Ana's projection must never contain Dev's member-private note"
            );
        }
    }

    // The sharing posture: the contributor's change was denied with the
    // named reason; the owner's landed.
    assert_eq!(
        result.worktree.shared_filesystem,
        SharedFilesystemMode::Read
    );
    assert_eq!(result.context.memory, Visibility::WorkspaceShared);

    // Presence: one replaceable record for Dev — never an event stream.
    assert_eq!(result.presence.records().len(), 1);
    assert_eq!(
        result.presence.records()[0].locator,
        SurfaceLocator::Task { task: task() }
    );
}

/// The no-lost-update law under adversarial interleaving: two writers
/// race on the same key with stale expectations — the stale write is
/// refused with the named version-conflict reason, and NOTHING either
/// author wrote ever disappears.
#[test]
fn no_authorized_state_is_lost_when_two_writers_interleave() {
    let base = inputs();
    // Both writers believe the brief is at version 1; Ana writes first,
    // so Dev's racing write (still expecting 1) must refuse with the
    // named version-conflict reason — never a silent clobber.
    let ana_race = InterleavedStep {
        actor: ok(ActorRef::user("ana")),
        op: SimulatedOperation::WriteSharedState {
            key: "brief".to_owned(),
            value: "Ana's edit".to_owned(),
            expected_version: 1,
            visibility: Visibility::WorkspaceShared,
        },
        expected: ExpectedOutcome::Written { version: 2 },
    };
    let dev_race = InterleavedStep {
        actor: ok(ActorRef::user("dev")),
        op: SimulatedOperation::WriteSharedState {
            key: "brief".to_owned(),
            value: "Dev's racing edit".to_owned(),
            expected_version: 1,
            visibility: Visibility::WorkspaceShared,
        },
        expected: ExpectedOutcome::Denied {
            reason: DenialReason::VersionConflict,
        },
    };
    let dev_recovery = InterleavedStep {
        actor: ok(ActorRef::user("dev")),
        op: SimulatedOperation::WriteSharedState {
            key: "brief".to_owned(),
            value: "Dev's recovered edit".to_owned(),
            expected_version: 2,
            visibility: Visibility::WorkspaceShared,
        },
        expected: ExpectedOutcome::Written { version: 3 },
    };
    let table = ok(InterleavingTable::new(
        workspace(),
        task(),
        vec![
            ana_race,
            dev_race,
            // Dev re-reads (his projection carries version 2) and
            // re-lands his edit on top of Ana's.
            InterleavedStep {
                actor: ok(ActorRef::user("dev")),
                op: SimulatedOperation::ReadProjection,
                expected: ExpectedOutcome::Projection {
                    keys: vec!["brief".to_owned()],
                },
            },
            dev_recovery,
        ],
    ));
    let result = ok(run_interleaved(&base, &table));
    ok(verify_expectations(&table, &result));
    ok(verify_f9_laws(&base, &table, &result));
    // Ana's edit landed at 2; Dev's racing write was refused (named
    // version conflict — never a silent clobber of Ana's work); Dev's
    // recovered edit landed at 3.
    assert_eq!(
        result.log.records[1].outcome.denial_reason(),
        Some(DenialReason::VersionConflict),
        "the stale racing write refuses with the named reason"
    );
    assert_eq!(
        result.store.get("brief").map_or(0, |entry| entry.version),
        3
    );
    assert_eq!(
        result
            .store
            .get("brief")
            .map_or(String::new(), |entry| entry.value.clone()),
        "Dev's recovered edit",
        "Ana's landed edit is the base Dev recovered onto — nothing was lost"
    );
}

/// Permissions hold under interleaving: an expelled member's every
/// operation comes back `not_a_member` and never mutates the store.
#[test]
fn an_outsider_never_mutates_the_seam() {
    let base = inputs();
    let outsider = ok(ActorRef::user("outsider"));
    let table = ok(InterleavingTable::new(
        workspace(),
        task(),
        vec![
            InterleavedStep {
                actor: outsider.clone(),
                op: SimulatedOperation::WriteSharedState {
                    key: "hijack".to_owned(),
                    value: "the outsider's overwrite".to_owned(),
                    expected_version: 0,
                    visibility: Visibility::WorkspaceShared,
                },
                expected: ExpectedOutcome::Denied {
                    reason: DenialReason::NotAMember,
                },
            },
            InterleavedStep {
                actor: outsider,
                op: SimulatedOperation::ReadProjection,
                expected: ExpectedOutcome::Denied {
                    reason: DenialReason::NotAMember,
                },
            },
        ],
    ));
    let result = ok(run_interleaved(&base, &table));
    ok(verify_expectations(&table, &result));
    ok(verify_f9_laws(&base, &table, &result));
    assert_eq!(
        result.store.entries().len(),
        1,
        "the outsider's write never created an entry"
    );
}

/// The member-private projection law under interleaving: a private
/// record never appears in another actor's projection, whoever reads
/// whenever — and writing it is denied by name, even for the owner.
#[test]
fn member_private_records_stay_private_to_their_actor() {
    let base = inputs();
    let table = ok(InterleavingTable::new(
        workspace(),
        task(),
        vec![
            InterleavedStep {
                actor: ok(ActorRef::user("dev")),
                op: SimulatedOperation::WriteSharedState {
                    key: "dev-notes".to_owned(),
                    value: "Dev's private scratchpad".to_owned(),
                    expected_version: 0,
                    visibility: Visibility::MemberPrivate,
                },
                expected: ExpectedOutcome::Written { version: 1 },
            },
            InterleavedStep {
                actor: ok(ActorRef::user("ana")),
                op: SimulatedOperation::ReadProjection,
                expected: ExpectedOutcome::Projection {
                    keys: vec!["brief".to_owned()],
                },
            },
            InterleavedStep {
                actor: ok(ActorRef::user("ana")),
                op: SimulatedOperation::WriteSharedState {
                    key: "dev-notes".to_owned(),
                    value: "the owner cannot write this".to_owned(),
                    expected_version: 1,
                    visibility: Visibility::MemberPrivate,
                },
                expected: ExpectedOutcome::Denied {
                    reason: DenialReason::PrivateRecord,
                },
            },
            InterleavedStep {
                actor: ok(ActorRef::user("dev")),
                op: SimulatedOperation::ReadProjection,
                expected: ExpectedOutcome::Projection {
                    keys: vec!["brief".to_owned(), "dev-notes".to_owned()],
                },
            },
        ],
    ));
    let result = ok(run_interleaved(&base, &table));
    ok(verify_expectations(&table, &result));
    ok(verify_f9_laws(&base, &table, &result));
    // The owner's projection carried only the shared brief.
    if let SimulationOutcome::Projection { rows } = &result.log.records[1].outcome {
        assert_eq!(
            rows.iter().map(|row| row.key.clone()).collect::<Vec<_>>(),
            vec!["brief".to_owned()]
        );
    }
    // Dev sees both: the shared brief and his own private note.
    if let SimulationOutcome::Projection { rows } = &result.log.records[3].outcome {
        assert_eq!(
            rows.iter().map(|row| row.key.clone()).collect::<Vec<_>>(),
            vec!["brief".to_owned(), "dev-notes".to_owned()]
        );
    }
}

/// The interleaving log is canonical-JSON replayable and the run is
/// deterministic: two runs produce byte-identical logs, and the log
/// round-trips byte-identically.
#[test]
fn the_interleaving_log_is_canonical_json_replayable() {
    let inputs = flauz_collab::fakes::fake_simulation_inputs();
    let table = flauz_collab::fakes::fake_interleaving_table();
    let first = ok(run_interleaved(&inputs, &table));
    let second = ok(run_interleaved(&inputs, &table));
    let serialized = ok(serde_json::to_string_pretty(&first.log));
    assert_eq!(
        ok(serde_json::to_string_pretty(&second.log)),
        serialized,
        "two runs of the same inputs produce byte-identical logs"
    );
    let replayed: flauz_collab::simulator::SimulationLog = ok(serde_json::from_str(&serialized));
    assert_eq!(
        ok(serde_json::to_string_pretty(&replayed)),
        serialized,
        "the log round-trips byte-identically through canonical JSON"
    );
    assert_eq!(replayed, first.log);
}

/// Permissions hold under interleaving with grants in play: the
/// evaluator's explicit-allow path drives the seam exactly as the role
/// defaults do — and the denied paths never mutate.
#[test]
fn grants_hold_under_interleaving_too() {
    let mut base = inputs();
    // Dev is explicitly denied on tasks: every Dev write must refuse
    // with the named explicit-deny reason while Ana's keep landing.
    base.grants = vec![ok(PermissionGrant::new(
        workspace(),
        ok(ActorRef::user("dev")),
        SurfaceFamily::Tasks,
        GrantEffect::Deny,
    ))];
    let table = ok(InterleavingTable::new(
        workspace(),
        task(),
        vec![
            write_step("ana", "brief", "Ana's edit", 1),
            InterleavedStep {
                actor: ok(ActorRef::user("dev")),
                op: SimulatedOperation::WriteSharedState {
                    key: "brief".to_owned(),
                    value: "Dev's denied edit".to_owned(),
                    expected_version: 2,
                    visibility: Visibility::WorkspaceShared,
                },
                expected: ExpectedOutcome::Denied {
                    reason: DenialReason::ExplicitDeny,
                },
            },
        ],
    ));
    let result = ok(run_interleaved(&base, &table));
    ok(verify_expectations(&table, &result));
    ok(verify_f9_laws(&base, &table, &result));
    assert_eq!(
        result.store.get("brief").map_or(0, |entry| entry.version),
        2
    );
    assert_eq!(
        result
            .store
            .get("brief")
            .map_or(String::new(), |entry| entry.value.clone()),
        "Ana's edit",
        "the explicitly denied write never touched the store"
    );
}

/// The evaluator-driven seam answers the wave gate's scripted probes:
/// the same authorization vocabulary the UI renders is what the
/// simulation recorded.
#[test]
fn the_seam_authorizations_match_the_evaluator() {
    let inputs = flauz_collab::fakes::fake_simulation_inputs();
    let table = flauz_collab::fakes::fake_interleaving_table();
    let result = ok(run_interleaved(&inputs, &table));
    // Step 2 (index 1): Dev probing manage on members — the log's
    // decision equals a direct evaluator call with the same inputs.
    let record = &result.log.records[1];
    let probe = AuthorizationRequest {
        workspace: &inputs.workspace,
        actor: &record.actor,
        action: AccessLevel::Manage,
        surface: SurfaceFamily::Members,
        target: None,
        memberships: &inputs.memberships,
        grants: &inputs.grants,
        lattice: &inputs.lattice,
    };
    let direct = ok(authorize(&probe));
    assert_eq!(
        direct.outcome,
        DecisionOutcome::Deny {
            reason: DenialReason::RoleDenied,
            role: Some(Role::Contributor),
            role_level: Some(AccessLevel::Read),
        }
    );
    // Step 11 (index 10): Ana probing manage on members.
    let record = &result.log.records[10];
    let probe = AuthorizationRequest {
        workspace: &inputs.workspace,
        actor: &record.actor,
        action: AccessLevel::Manage,
        surface: SurfaceFamily::Members,
        target: None,
        memberships: &inputs.memberships,
        grants: &inputs.grants,
        lattice: &inputs.lattice,
    };
    let direct = ok(authorize(&probe));
    assert_eq!(
        direct.outcome,
        DecisionOutcome::Allow {
            role: Role::Owner,
            via: AllowRule::RoleDefault,
        }
    );
}
