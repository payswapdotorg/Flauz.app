//! Kernel conformance tests (F2 Wave-4 addendum §4-§6 + kernel §4/§7,
//! work order COL-001): canonical JSON, the fixture set, the
//! evaluator-output and simulation-output laws, the replay law, the
//! no-floats rule, and the no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_collab::membership::{
    AccessLevel, Role, RoleLattice, SurfaceFamily, WorkspaceMembership,
};
use flauz_collab::permission::{AuthorizationRequest, DecisionOutcome, DenialReason, authorize};
use flauz_collab::policy::{ContextVisibility, SharedFilesystemMode, Visibility, WorktreePolicy};
use flauz_collab::presence::{PresenceFreshness, PresenceRecord, SurfaceLocator};
use flauz_collab::refs::{ActorRef, WorkspaceRef};
use flauz_collab::simulator::{
    InterleavingTable, SimulationInputs, SimulationLog, run_interleaved, verify_expectations,
    verify_f9_laws,
};
use flauz_collab::time::Timestamp;

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/w4")
        .join(relative)
}

fn read_fixture(relative: &str) -> String {
    let content = fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
    // Canonical fixtures are committed with LF endings; a Windows checkout
    // with autocrlf translates them to CRLF. Normalize before comparison
    // so the round-trip law is tested against the CANONICAL form, not the
    // platform's line-ending translation.
    content.replace("\r\n", "\n").trim_end().to_owned()
}

fn round_trip<T>(content: &str) -> String
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = test_ok(serde_json::from_str(content));
    test_ok(serde_json::to_string_pretty(&parsed))
}

/// Every valid fixture family round-trips: a strict parse (unknown
/// fields rejected) that re-serializes to exactly the committed bytes
/// (kernel §4 — canonical JSON; the byte-identical round-trip law).
#[test]
fn fixtures_roundtrip_canonical() {
    let membership_fixtures = [
        "workspace-membership/typical.json",
        "workspace-membership/minimal.json",
    ];
    for relative in membership_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<WorkspaceMembership>(&content),
            content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let lattice_fixtures = ["role-lattice/typical.json", "role-lattice/minimal.json"];
    for relative in lattice_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<RoleLattice>(&content),
            content,
            "fixture {relative} is not canonical"
        );
    }
    let decision_fixtures = [
        "authorization-decision/typical.json",
        "authorization-decision/minimal.json",
    ];
    for relative in decision_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<flauz_collab::permission::AuthorizationDecision>(&content),
            content,
            "fixture {relative} is not canonical"
        );
    }
    let grant_fixtures = [
        "permission-grant/typical.json",
        "permission-grant/minimal.json",
    ];
    for relative in grant_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<flauz_collab::permission::PermissionGrant>(&content),
            content,
            "fixture {relative} is not canonical"
        );
    }
    let presence_fixtures = [
        "presence-record/typical.json",
        "presence-record/minimal.json",
    ];
    for relative in presence_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<PresenceRecord>(&content),
            content,
            "fixture {relative} is not canonical"
        );
    }
    let worktree_fixtures = [
        "worktree-policy/typical.json",
        "worktree-policy/minimal.json",
    ];
    for relative in worktree_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<WorktreePolicy>(&content),
            content,
            "fixture {relative} is not canonical"
        );
    }
    let visibility_fixtures = [
        "context-visibility/typical.json",
        "context-visibility/minimal.json",
    ];
    for relative in visibility_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<ContextVisibility>(&content),
            content,
            "fixture {relative} is not canonical"
        );
    }
    let inputs_fixtures = [
        "simulation-inputs/typical.json",
        "simulation-inputs/minimal.json",
    ];
    for relative in inputs_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<SimulationInputs>(&content),
            content,
            "fixture {relative} is not canonical"
        );
    }
    let table_fixtures = [
        "interleaving-table/typical.json",
        "interleaving-table/minimal.json",
    ];
    for relative in table_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<InterleavingTable>(&content),
            content,
            "fixture {relative} is not canonical"
        );
    }
    let log_fixtures = ["simulation-log/typical.json", "simulation-log/minimal.json"];
    for relative in log_fixtures {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<SimulationLog>(&content),
            content,
            "fixture {relative} is not canonical — the interleaving log must replay \
             byte-identically"
        );
    }
}

/// Invalid fixtures fail strict parses — a record that violates the
/// canonical grammar (unknown fields, wrong kind prefixes, malformed
/// references) is rejected on read, never silently misread; records
/// that parse but break record-consistency laws fail `validate()`
/// (addendum §5's invalid-per-family rule).
#[test]
fn invalid_fixtures_fail_strict_parses() {
    // Unknown fields are rejected on read, per family.
    let unknown_field_fixtures = [
        ("workspace-membership", "WorkspaceMembership"),
        ("role-lattice", "RoleLattice"),
        ("permission-grant", "PermissionGrant"),
        ("authorization-decision", "AuthorizationDecision"),
        ("presence-record", "PresenceRecord"),
        ("worktree-policy", "WorktreePolicy"),
        ("context-visibility", "ContextVisibility"),
    ];
    for (family, _type_name) in unknown_field_fixtures {
        let path = fixture_path(&format!("{family}/invalid-unknown-field.json"));
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("missing invalid fixture for {family}: {error}"));
        let content = content.replace("\r\n", "\n");
        let rejected = [
            serde_json::from_str::<WorkspaceMembership>(&content).is_err(),
            serde_json::from_str::<RoleLattice>(&content).is_err(),
            serde_json::from_str::<flauz_collab::permission::PermissionGrant>(&content).is_err(),
            serde_json::from_str::<flauz_collab::permission::AuthorizationDecision>(&content)
                .is_err(),
            serde_json::from_str::<PresenceRecord>(&content).is_err(),
            serde_json::from_str::<WorktreePolicy>(&content).is_err(),
            serde_json::from_str::<ContextVisibility>(&content).is_err(),
        ];
        assert!(
            rejected.iter().any(|failed| *failed),
            "{family}'s unknown-field fixture must fail at least one strict parse"
        );
    }

    // Grammar failures: wrong kind prefixes are rejected on read.
    let bad_workspace = read_fixture("workspace-membership/invalid-grammar.json");
    assert!(
        serde_json::from_str::<WorkspaceMembership>(&bad_workspace).is_err(),
        "a membership referencing a non-workspace id must be rejected on read"
    );
    let bad_task = read_fixture("worktree-policy/invalid-bad-task.json");
    assert!(
        serde_json::from_str::<WorktreePolicy>(&bad_task).is_err(),
        "a worktree policy referencing a malformed task id must be rejected on read"
    );

    // Record-consistency failures: the fixture parses but fails
    // validation — the duplicate lattice row is a silent ambiguity the
    // validation refuses.
    let duplicate_role = read_fixture("role-lattice/invalid-duplicate-role.json");
    let lattice: RoleLattice = test_ok(serde_json::from_str(&duplicate_role));
    assert!(
        lattice.validate().is_err(),
        "a lattice with a duplicated role row is invalid — never a silent ambiguity"
    );
    let zero_window = read_fixture("presence-record/invalid-zero-window.json");
    let presence: PresenceRecord = test_ok(serde_json::from_str(&zero_window));
    assert!(
        presence.validate().is_err(),
        "a zero-width freshness window is invalid"
    );

    // The big families reject unknown fields too — injected at runtime
    // (proportional: no 200-line duplicated fixtures).
    let inputs = read_fixture("simulation-inputs/typical.json");
    let injected = inputs.replace("\"presence_stale_after_ms\"", "\"window_ms\"");
    assert!(
        serde_json::from_str::<SimulationInputs>(&injected).is_err(),
        "simulation inputs reject renamed fields"
    );
    let table = read_fixture("interleaving-table/typical.json");
    let injected = table.replace("\"workspace\":", "\"ws\":");
    assert!(
        serde_json::from_str::<InterleavingTable>(&injected).is_err(),
        "interleaving tables reject renamed fields"
    );
    let log = read_fixture("simulation-log/typical.json");
    let injected = log.replace("\"records\":", "\"steps\":");
    assert!(
        serde_json::from_str::<SimulationLog>(&injected).is_err(),
        "simulation logs reject renamed fields"
    );
    // A future schema version fails the canonical read.
    let versioned = inputs.replace("\"v\": 1", "\"v\": 2");
    assert!(
        serde_json::from_str::<SimulationInputs>(&versioned).is_err(),
        "a v2 document must fail the v1 canonical read, never silently misread"
    );
}

/// The evaluator-output law: every committed authorization-decision
/// fixture EQUALS the evaluator run over the committed membership and
/// lattice fixtures it documents — the fixtures are honest artifacts of
/// the evaluator, never hand-staged shapes.
#[test]
fn decision_fixtures_are_the_evaluator_output_over_their_inputs() {
    let typical_membership: WorkspaceMembership = test_ok(serde_json::from_str(&read_fixture(
        "workspace-membership/typical.json",
    )));
    let minimal_membership: WorkspaceMembership = test_ok(serde_json::from_str(&read_fixture(
        "workspace-membership/minimal.json",
    )));
    let memberships = vec![typical_membership, minimal_membership];
    let lattice: RoleLattice = test_ok(serde_json::from_str(&read_fixture(
        "role-lattice/typical.json",
    )));
    let workspace: WorkspaceRef = test_ok(serde_json::from_str(&test_ok(serde_json::to_string(
        &memberships[0].workspace,
    ))));

    let dev = test_ok(ActorRef::user("dev"));
    let probe = AuthorizationRequest {
        workspace: &workspace,
        actor: &dev,
        action: AccessLevel::Manage,
        surface: SurfaceFamily::Members,
        target: None,
        memberships: &memberships,
        grants: &[],
        lattice: &lattice,
    };
    let decision = test_ok(authorize(&probe));
    let expected: flauz_collab::permission::AuthorizationDecision = test_ok(serde_json::from_str(
        &read_fixture("authorization-decision/typical.json"),
    ));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&decision)),
        test_ok(serde_json::to_string_pretty(&expected)),
        "authorization-decision/typical.json must equal the evaluator output"
    );
    assert_eq!(
        decision.outcome,
        DecisionOutcome::Deny {
            reason: DenialReason::RoleDenied,
            role: Some(Role::Contributor),
            role_level: Some(AccessLevel::Read),
        }
    );

    let ana = test_ok(ActorRef::user("ana"));
    let probe = AuthorizationRequest {
        workspace: &workspace,
        actor: &ana,
        action: AccessLevel::Manage,
        surface: SurfaceFamily::Members,
        target: None,
        memberships: &memberships,
        grants: &[],
        lattice: &lattice,
    };
    let decision = test_ok(authorize(&probe));
    let expected: flauz_collab::permission::AuthorizationDecision = test_ok(serde_json::from_str(
        &read_fixture("authorization-decision/minimal.json"),
    ));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&decision)),
        test_ok(serde_json::to_string_pretty(&expected)),
        "authorization-decision/minimal.json must equal the evaluator output"
    );
}

/// The simulation-output law: every committed simulation-log fixture
/// EQUALS the deterministic run over the committed inputs and table
/// fixtures it documents — the F9 gate law's replayable evidence, never
/// a hand-staged shape.
#[test]
fn log_fixtures_are_the_run_output_over_their_fixtures() {
    let checks = [
        (
            "simulation-log/typical.json",
            "simulation-inputs/typical.json",
            "interleaving-table/typical.json",
        ),
        (
            "simulation-log/minimal.json",
            "simulation-inputs/minimal.json",
            "interleaving-table/minimal.json",
        ),
    ];
    for (log_fixture, inputs_fixture, table_fixture) in checks {
        let inputs: SimulationInputs = test_ok(serde_json::from_str(&read_fixture(inputs_fixture)));
        let table: InterleavingTable = test_ok(serde_json::from_str(&read_fixture(table_fixture)));
        let expected: SimulationLog = test_ok(serde_json::from_str(&read_fixture(log_fixture)));
        let result = test_ok(run_interleaved(&inputs, &table));
        assert_eq!(
            test_ok(serde_json::to_string_pretty(&result.log)),
            test_ok(serde_json::to_string_pretty(&expected)),
            "{log_fixture} must equal the run over {inputs_fixture} + {table_fixture}"
        );
        // The run over the committed fixtures also satisfies its own
        // expectations and the F9 gate laws — replay proves the laws
        // hold from the fixture bytes alone.
        test_ok(verify_expectations(&table, &result));
        test_ok(verify_f9_laws(&inputs, &table, &result));
    }
}

/// The public fakes reproduce the committed fixtures exactly: the fake
/// surface is the reference semantics for real collaboration inputs
/// (kernel §7 determinism — the fakes are stable across runs).
#[test]
fn the_public_fakes_match_the_committed_fixtures() {
    let inputs: SimulationInputs = test_ok(serde_json::from_str(&read_fixture(
        "simulation-inputs/typical.json",
    )));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(
            &flauz_collab::fakes::fake_simulation_inputs()
        )),
        test_ok(serde_json::to_string_pretty(&inputs))
    );
    let table: InterleavingTable = test_ok(serde_json::from_str(&read_fixture(
        "interleaving-table/typical.json",
    )));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(
            &flauz_collab::fakes::fake_interleaving_table()
        )),
        test_ok(serde_json::to_string_pretty(&table))
    );
    let lattice: RoleLattice = test_ok(serde_json::from_str(&read_fixture(
        "role-lattice/typical.json",
    )));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(
            &flauz_collab::fakes::fake_default_lattice()
        )),
        test_ok(serde_json::to_string_pretty(&lattice))
    );
    let grants: Vec<flauz_collab::permission::PermissionGrant> =
        test_ok(serde_json::from_str(&test_ok(
            serde_json::to_string_pretty(&flauz_collab::fakes::fake_grants()),
        )));
    assert_eq!(grants.len(), 2);
}

/// Canonical JSON carries no floats (kernel §4): every serialized
/// contract value is an integer, a string, a boolean, null, an array or
/// an object — and every top-level record carries `"v": 1`.
#[test]
fn serialized_contract_state_contains_no_floats() {
    fn assert_no_floats(value: &serde_json::Value, path: &str) {
        match value {
            serde_json::Value::Number(number) => assert!(
                number.is_u64() || number.is_i64(),
                "float at {path}: {number}"
            ),
            serde_json::Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    assert_no_floats(item, &format!("{path}[{index}]"));
                }
            }
            serde_json::Value::Object(fields) => {
                for (name, item) in fields {
                    assert_no_floats(item, &format!("{path}.{name}"));
                }
            }
            _ => {}
        }
    }
    let all_fixtures = [
        "workspace-membership/typical.json",
        "workspace-membership/minimal.json",
        "role-lattice/typical.json",
        "role-lattice/minimal.json",
        "permission-grant/typical.json",
        "permission-grant/minimal.json",
        "authorization-decision/typical.json",
        "authorization-decision/minimal.json",
        "presence-record/typical.json",
        "presence-record/minimal.json",
        "worktree-policy/typical.json",
        "worktree-policy/minimal.json",
        "context-visibility/typical.json",
        "context-visibility/minimal.json",
        "simulation-inputs/typical.json",
        "simulation-inputs/minimal.json",
        "interleaving-table/typical.json",
        "interleaving-table/minimal.json",
        "simulation-log/typical.json",
        "simulation-log/minimal.json",
    ];
    for relative in all_fixtures {
        let value: serde_json::Value = test_ok(serde_json::from_str(&read_fixture(relative)));
        assert_no_floats(&value, relative);
        // The schema version marker is exactly 1.
        assert_eq!(
            value["v"],
            serde_json::json!(1),
            "{relative} carries \"v\": 1"
        );
    }
}

/// No credential material anywhere (Wave-4 addendum §3): the
/// collaboration families carry member identity references, display
/// names and policy data only. Any credential-looking field in
/// serialized state is an automatic gate failure.
#[test]
fn no_credential_material_in_serialized_state() {
    let all_fixtures = [
        "workspace-membership/typical.json",
        "workspace-membership/minimal.json",
        "role-lattice/typical.json",
        "role-lattice/minimal.json",
        "permission-grant/typical.json",
        "permission-grant/minimal.json",
        "authorization-decision/typical.json",
        "authorization-decision/minimal.json",
        "presence-record/typical.json",
        "presence-record/minimal.json",
        "worktree-policy/typical.json",
        "worktree-policy/minimal.json",
        "context-visibility/typical.json",
        "context-visibility/minimal.json",
        "simulation-inputs/typical.json",
        "simulation-inputs/minimal.json",
        "interleaving-table/typical.json",
        "interleaving-table/minimal.json",
        "simulation-log/typical.json",
        "simulation-log/minimal.json",
    ];
    for relative in all_fixtures {
        let content = read_fixture(relative);
        let lower = content.to_lowercase();
        for forbidden in [
            "\"token\"",
            "\"secret\"",
            "\"password\"",
            "\"api_key\"",
            "\"apikey\"",
            "\"credential\"",
            "sk-",
            "bearer ",
            "flausec_",
        ] {
            assert!(
                !lower.contains(forbidden),
                "credential material {forbidden:?} must never appear in {relative}"
            );
        }
    }
}

/// Presence records never serialize as a stream (addendum §5): the
/// family's serialization is the single replaceable record — no
/// sequence numbers, no history fields.
#[test]
fn presence_records_carry_no_stream_vocabulary() {
    let content = read_fixture("presence-record/typical.json");
    for stream_word in [
        "\"seq\"",
        "\"sequence\"",
        "\"event_id\"",
        "\"history\"",
        "\"log\"",
    ] {
        assert!(
            !content.contains(stream_word),
            "presence must never serialize stream vocabulary ({stream_word})"
        );
    }
    // The registered event vocabulary excludes presence entirely.
    assert_eq!(
        flauz_collab::simulator::event_types::MEMBER_ADDED,
        "member.added"
    );
    assert_eq!(
        flauz_collab::simulator::event_types::MEMBER_ROLE_CHANGED,
        "member.role_changed"
    );
    assert_eq!(
        flauz_collab::simulator::event_types::MEMBER_REMOVED,
        "member.removed"
    );
    assert_eq!(
        flauz_collab::simulator::event_types::PERMISSION_GRANTED,
        "permission.granted"
    );
    assert_eq!(
        flauz_collab::simulator::event_types::PERMISSION_REVOKED,
        "permission.revoked"
    );
    assert_eq!(
        flauz_collab::simulator::event_types::SHARING_CHANGED,
        "sharing.changed"
    );
}

/// The sharing-posture fixtures state the platform defaults honestly:
/// isolated-by-default worktrees, member-private notes by default.
#[test]
fn sharing_fixtures_pin_the_isolated_by_default_posture() {
    let isolated: WorktreePolicy = test_ok(serde_json::from_str(&read_fixture(
        "worktree-policy/minimal.json",
    )));
    assert!(isolated.is_isolated());
    assert_eq!(isolated.shared_filesystem, SharedFilesystemMode::Off);
    let read_mode: WorktreePolicy = test_ok(serde_json::from_str(&read_fixture(
        "worktree-policy/typical.json",
    )));
    assert_eq!(read_mode.shared_filesystem, SharedFilesystemMode::Read);
    let typical: ContextVisibility = test_ok(serde_json::from_str(&read_fixture(
        "context-visibility/typical.json",
    )));
    assert_eq!(typical.memory, Visibility::MemberPrivate);
    assert_eq!(typical.artifacts, Visibility::WorkspaceShared);
    // Freshness bounds parse and evaluate deterministically.
    let presence: PresenceRecord = test_ok(serde_json::from_str(&read_fixture(
        "presence-record/typical.json",
    )));
    let now = test_ok(Timestamp::parse("2026-09-23T10:00:30Z"));
    assert!(!presence.is_stale(&now));
    let later = test_ok(Timestamp::parse("2026-09-23T10:02:00Z"));
    assert!(presence.is_stale(&later));
    assert_eq!(
        presence.locator,
        SurfaceLocator::Task {
            task: test_ok(flauz_collab::refs::TaskRef::parse(
                "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"
            ))
        }
    );
    assert_eq!(
        presence.freshness.stale_after_ms,
        flauz_collab::DEFAULT_PRESENCE_STALE_AFTER_MS
    );
    let _ = test_ok(PresenceFreshness::new(presence.freshness.last_seen, 1));
}
