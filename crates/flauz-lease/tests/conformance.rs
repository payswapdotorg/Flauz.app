//! Kernel conformance tests (Wave-5 addendum §1-§7, the lease
//! manager): canonical JSON, the fixture set, the conflict-honesty law
//! over the complete fake conflict space, the queue and expiry laws,
//! determinism, and the no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_lease::fakes::{ConflictCase, conflict_space, dev_write_request, now};
use flauz_lease::lease::LeaseRecord;
use flauz_lease::manager::{WaiterOutcome, decide, sweep};
use flauz_lease::mode::AccessMode;
use flauz_lease::refs::ActorKind;
use flauz_lease::request::{LeaseDecision, LeaseRequest, NamedCause};
use flauz_lease::{LeaseSweep, WaitSnapshot};

/// The credential-material marker family (the PROV-001 scan, re-pinned
/// locally — this crate imports no contract crate).
const CREDENTIAL_MARKERS: &[&str] = &[
    "sk-",
    "Bearer ",
    "api_key",
    "apikey",
    "password",
    "passwd",
    "client_secret",
    "access_token",
    "refresh_token",
    "PRIVATE KEY",
    "BEGIN RSA",
    "xoxb-",
    "ghp_",
    "flausec_",
];

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/w5")
        .join(relative)
}

fn read_fixture(relative: &str) -> String {
    let content = fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
    // Canonical fixtures are committed with LF endings; a Windows
    // checkout with autocrlf translates them to CRLF. Normalize before
    // comparison so the round-trip law is tested against the CANONICAL
    // form, not the platform's line-ending translation.
    content.replace("\r\n", "\n").trim_end().to_owned()
}

fn round_trip<T>(content: &str) -> String
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = test_ok(serde_json::from_str(content));
    test_ok(serde_json::to_string_pretty(&parsed))
}

/// Every valid conformance fixture is canonical JSON: it parses
/// strictly (unknown fields rejected) and re-serializes to exactly the
/// committed bytes (kernel §4). The round-trips run through the
/// CONTRACT TYPES (never `serde_json::Value`, whose map reorders
/// fields — the committed order is the struct order).
#[test]
fn fixtures_roundtrip_canonical() {
    let requests = ["lease-request/typical.json", "lease-request/minimal.json"];
    for relative in requests {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<LeaseRequest>(&content),
            content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let records = ["lease-record/typical.json", "lease-record/minimal.json"];
    for relative in records {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<LeaseRecord>(&content),
            content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let waiters = ["wait-snapshot/typical.json", "wait-snapshot/minimal.json"];
    for relative in waiters {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<WaitSnapshot>(&content),
            content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let decisions = [
        "lease-decision/granted.json",
        "lease-decision/rejected.json",
        "lease-decision/queued.json",
        "lease-decision/escalated.json",
    ];
    for relative in decisions {
        let content = read_fixture(relative);
        assert_eq!(
            round_trip::<LeaseDecision>(&content),
            content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
}

/// Invalid fixtures fail strict parses or canonical validation — a
/// record that violates the frozen rules is rejected on read or on
/// validate, never silently misread.
#[test]
fn invalid_fixtures_fail_strict_parses_or_validation() {
    let unknown_field = read_fixture("lease-decision/invalid-unknown-field.json");
    assert!(
        serde_json::from_str::<LeaseDecision>(&unknown_field).is_err(),
        "unknown fields must be rejected"
    );
    let bad_window = read_fixture("lease-request/invalid-window.json");
    let request: LeaseRequest = test_ok(serde_json::from_str(&bad_window));
    assert!(
        request.validate().is_err(),
        "a backwards window must be rejected"
    );
    let bad_renewal = read_fixture("lease-request/invalid-renewal-mismatch.json");
    let request: LeaseRequest = test_ok(serde_json::from_str(&bad_renewal));
    assert!(
        request.validate().is_err(),
        "a renewal that names a different lease must be rejected"
    );
    let bad_deadline = read_fixture("lease-record/invalid-deadline.json");
    let record: LeaseRecord = test_ok(serde_json::from_str(&bad_deadline));
    assert!(
        record.validate().is_err(),
        "a lease already expired when granted cannot exist"
    );
}

/// The honest-artifact law: every committed decision fixture EQUALS the
/// manager run over the committed request and record fixtures it
/// documents — the fixtures are honest artifacts of the manager, never
/// hand-staged shapes.
#[test]
fn decision_fixtures_are_the_manager_output_over_their_inputs() {
    let request: LeaseRequest = test_ok(serde_json::from_str(&read_fixture(
        "lease-request/typical.json",
    )));
    let holder: LeaseRecord = test_ok(serde_json::from_str(&read_fixture(
        "lease-record/typical.json",
    )));

    // granted: no holders at all.
    let granted = test_ok(decide(&request, &[], &[], now()));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&granted)),
        read_fixture("lease-decision/granted.json")
    );

    // rejected / queued / escalated: the same request against the same
    // holder under each policy.
    let mut rejected_holder = holder.clone();
    rejected_holder.conflict_policy = flauz_lease::ConflictPolicy::Reject;
    let rejected = test_ok(decide(&request, &[rejected_holder], &[], now()));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&rejected)),
        read_fixture("lease-decision/rejected.json")
    );

    let queued = test_ok(decide(&request, std::slice::from_ref(&holder), &[], now()));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&queued)),
        read_fixture("lease-decision/queued.json")
    );

    let mut escalated_holder = holder;
    escalated_holder.conflict_policy = flauz_lease::ConflictPolicy::Escalate;
    let escalated = test_ok(decide(&request, &[escalated_holder], &[], now()));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&escalated)),
        read_fixture("lease-decision/escalated.json")
    );
}

/// The conflict-honesty law (addendum §2), proven over the COMPLETE
/// fake conflict space: every decision carries the full request plus
/// its named cause — a decision without attribution cannot exist. A
/// conflicting decision is never silently queued, dropped or granted.
#[test]
fn every_decision_in_the_conflict_space_carries_its_named_cause() {
    for case in conflict_space() {
        let decision = test_ok(decide(
            &case.request,
            std::slice::from_ref(&case.holder),
            &[],
            case.now,
        ));
        // Every decision carries the requester's own ask.
        assert_eq!(decision.request().requester, case.request.requester);
        assert_eq!(decision.request().resource_id, case.request.resource_id);
        let conflict = case.holder.mode.conflicts_with(case.request.mode);
        match decision.named_cause() {
            NamedCause::NoConflict => {
                assert!(
                    !conflict,
                    "a conflicting request ({:?} × {:?}) was granted silently",
                    case.holder.mode, case.request.mode
                );
            }
            NamedCause::Holder(holder) => {
                assert!(conflict, "a compatible pair produced a named holder");
                assert_eq!(holder.id, case.holder.id);
                assert!(holder.is_held_at(&case.now));
            }
            NamedCause::Queue(_) => {
                assert!(conflict, "a compatible pair produced a queue decision");
            }
            NamedCause::Conflict(conflict_record) => {
                assert!(conflict, "a compatible pair escalated");
                assert_eq!(conflict_record.holder.id, case.holder.id);
                assert_eq!(conflict_record.request.requester, case.request.requester);
            }
        }
        // And every decision re-serializes canonically (snake_case,
        // internally tagged, no floats — the field types enforce it).
        let serialized = test_ok(serde_json::to_string(&decision));
        assert!(serialized.contains("\"kind\":\""));
    }
}

/// Determinism (kernel §7): identical inputs produce byte-identical
/// decisions over the whole conflict space — no wall clock, no
/// entropy, no snapshot-order sensitivity.
#[test]
fn decisions_are_deterministic_over_the_whole_space() {
    for case in conflict_space() {
        let first = test_ok(decide(
            &case.request,
            std::slice::from_ref(&case.holder),
            &[],
            case.now,
        ));
        let second = test_ok(decide(
            &case.request,
            std::slice::from_ref(&case.holder),
            &[],
            case.now,
        ));
        assert_eq!(
            test_ok(serde_json::to_string(&first)),
            test_ok(serde_json::to_string(&second)),
            "the same inputs must produce identical bytes ({:?} × {:?})",
            case.holder.mode,
            case.request.mode
        );
        // Snapshot order never matters (a holder on ANOTHER resource
        // rides the same snapshot list — with its own lease id, so the
        // fresh-id guard stays satisfied).
        let other_resource_holder = test_ok(LeaseRecord::new(
            test_ok(flauz_lease::LeaseRef::parse(
                "lease_01J8ZQ5V8K3T2B7N6X4R9DQPMF",
            )),
            test_ok(flauz_lease::ResourceRef::parse(
                "res_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
            )),
            AccessMode::Write,
            case.holder.owner.clone(),
            case.holder.granted_at,
            case.holder.expires_at,
            case.holder.conflict_policy,
        ));
        let reordered = test_ok(decide(
            &case.request,
            &[other_resource_holder, case.holder],
            &[],
            case.now,
        ));
        assert_eq!(
            test_ok(serde_json::to_string(&first)),
            test_ok(serde_json::to_string(&reordered)),
            "snapshot order must never change the decision"
        );
    }
}

/// The queue + expiry laws (addendum §2) end to end: a queued waiter
/// advances IN REQUEST ORDER when the holder expires, positions stay
/// honest, a closed window is named — and there is no implicit
/// renewal anywhere (the expired holder never reappears).
#[test]
fn the_queue_and_expiry_laws_hold_end_to_end() {
    let holder = flauz_lease::fakes::ana_write_lease(flauz_lease::ConflictPolicy::Queue);
    let first = test_ok(dev_write_request());
    let second = test_ok(LeaseRequest::new(
        test_ok(flauz_lease::LeaseRef::parse(
            "lease_01J8ZQ5V8K3T2B7N6X4R9DQPK9",
        )),
        flauz_lease::fakes::fake_resource(),
        AccessMode::Write,
        flauz_lease::fakes::fake_actors().ana,
        now(),
        test_ok(flauz_lease::Timestamp::parse("2026-09-23T17:00:00Z")),
        flauz_lease::ConflictPolicy::Queue,
    ));
    let waiters = [
        test_ok(WaitSnapshot::new(first, now())),
        test_ok(WaitSnapshot::new(
            second,
            test_ok(flauz_lease::Timestamp::parse("2026-09-23T15:10:00Z")),
        )),
    ];

    // Inside the holder's window both wait.
    let inside = test_ok(sweep(std::slice::from_ref(&holder), &waiters, now()));
    assert_eq!(inside.expired.len(), 0);
    let outcomes = &inside.resources[0].outcomes;
    assert!(matches!(
        outcomes[0],
        WaiterOutcome::StillWaiting { position: 1, .. }
    ));
    match &outcomes[1] {
        WaiterOutcome::StillWaiting {
            position, ahead, ..
        } => {
            assert_eq!(*position, 2);
            assert_eq!(ahead.len(), 1, "the honest position names who is ahead");
        }
        other => panic!("expected the second waiter to wait, got {other:?}"),
    }

    // After the deadline the holder expired; the FIRST waiter (in
    // request order) is granted and the second waits behind the new
    // holder — FIFO advancement, named.
    let after = test_ok(sweep(
        &[holder],
        &waiters,
        test_ok(flauz_lease::Timestamp::parse("2026-09-23T15:41:00Z")),
    ));
    assert_eq!(after.expired.len(), 1);
    let outcomes = &after.resources[0].outcomes;
    match &outcomes[0] {
        WaiterOutcome::Granted { lease } => {
            assert_eq!(
                lease.granted_at,
                test_ok(flauz_lease::Timestamp::parse("2026-09-23T15:41:00Z"))
            );
        }
        other => panic!("the first waiter must advance, got {other:?}"),
    }
    assert!(matches!(
        outcomes[1],
        WaiterOutcome::StillWaiting { position: 1, .. }
    ));

    // The sweep result itself round-trips canonically.
    let serialized = test_ok(serde_json::to_string_pretty(&after));
    let reloaded: LeaseSweep = test_ok(serde_json::from_str(&serialized));
    assert_eq!(reloaded, after);
}

/// The escalation law (addendum §2): an escalated conflict is decided
/// by a HUMAN, explicitly and attributed — an agent's decision is
/// refused, and both outcomes name both sides.
#[test]
fn escalation_is_never_auto_resolved() {
    let holder = flauz_lease::fakes::ana_write_lease(flauz_lease::ConflictPolicy::Escalate);
    let request = test_ok(dev_write_request());
    let decision = test_ok(decide(&request, &[holder], &[], now()));
    let LeaseDecision::Escalated {
        request, holder, ..
    } = decision
    else {
        panic!("the escalate policy must escalate");
    };
    let conflict = flauz_lease::NamedConflict {
        v: flauz_lease::LeaseVersion,
        request,
        holder,
    };
    let actors = flauz_lease::fakes::fake_actors();
    assert_eq!(actors.ana.kind, ActorKind::User);

    // An agent cannot decide.
    assert!(
        flauz_lease::resolve_escalation(
            &conflict,
            flauz_lease::EscalationChoice::KeepHolder,
            actors.dev_agent,
            now(),
        )
        .is_err()
    );

    // The human can, both ways, with both sides named.
    match test_ok(flauz_lease::resolve_escalation(
        &conflict,
        flauz_lease::EscalationChoice::GrantToWaiter,
        actors.ana.clone(),
        now(),
    )) {
        flauz_lease::EscalationOutcome::WaiterGranted {
            released_holder,
            granted,
            ..
        } => {
            assert_eq!(released_holder.released_at, Some(now()));
            assert_eq!(granted.owner, conflict.request.requester);
        }
        other => panic!("expected a waiter grant, got {other:?}"),
    }
    match test_ok(flauz_lease::resolve_escalation(
        &conflict,
        flauz_lease::EscalationChoice::KeepHolder,
        actors.ana,
        now(),
    )) {
        flauz_lease::EscalationOutcome::HolderKept { waiter, holder, .. } => {
            assert_eq!(waiter, conflict.request.requester);
            assert_eq!(holder.id, conflict.holder.id);
            assert_eq!(holder.released_at, None, "the kept holder is untouched");
        }
        other => panic!("expected a holder keep, got {other:?}"),
    }
}

/// Credentials are references, forever (the standing law): no
/// credential material appears in any serialized fake family or any
/// committed fixture of the new families.
#[test]
fn no_credential_material_in_serialized_state() {
    // Every fake family, serialized.
    let mut haystack = String::new();
    for case in conflict_space() {
        haystack.push_str(&test_ok(serde_json::to_string(&case.request)));
        haystack.push_str(&test_ok(serde_json::to_string(&case.holder)));
    }
    let request = test_ok(dev_write_request());
    haystack.push_str(&test_ok(serde_json::to_string(&request)));
    let holder = flauz_lease::fakes::ana_write_lease(flauz_lease::ConflictPolicy::Queue);
    haystack.push_str(&test_ok(serde_json::to_string(&holder)));
    let sweep_result = test_ok(sweep(
        &[holder],
        &[test_ok(WaitSnapshot::new(request, now()))],
        now(),
    ));
    haystack.push_str(&test_ok(serde_json::to_string(&sweep_result)));
    for marker in CREDENTIAL_MARKERS {
        assert!(
            !haystack.contains(marker),
            "credential marker {marker:?} must never appear in serialized lease state"
        );
    }

    // Every committed fixture, read as text.
    for relative in [
        "lease-request/typical.json",
        "lease-request/minimal.json",
        "lease-request/invalid-window.json",
        "lease-request/invalid-renewal-mismatch.json",
        "lease-record/typical.json",
        "lease-record/minimal.json",
        "lease-record/invalid-deadline.json",
        "wait-snapshot/typical.json",
        "wait-snapshot/minimal.json",
        "lease-decision/granted.json",
        "lease-decision/rejected.json",
        "lease-decision/queued.json",
        "lease-decision/escalated.json",
        "lease-decision/invalid-unknown-field.json",
    ] {
        let content = read_fixture(relative);
        for marker in CREDENTIAL_MARKERS {
            assert!(
                !content.contains(marker),
                "credential marker {marker:?} must never appear in fixture {relative}"
            );
        }
    }
}

/// The frozen wire vocabulary rides every record: modes and policies
/// serialize to the exact `flauz-world` words, and the lease record's
/// field names match the frozen `ResourceLease` shape (the byte-wise
/// seam).
#[test]
fn the_frozen_wire_shapes_ride_the_fixtures() {
    let record = read_fixture("lease-record/typical.json");
    for field in [
        "\"v\": 1",
        "\"id\":",
        "\"version\": 1",
        "\"resource_id\":",
        "\"mode\": \"write\"",
        "\"owner\":",
        "\"granted_at\":",
        "\"expires_at\":",
        "\"conflict_policy\": \"queue\"",
        "\"released_at\": null",
    ] {
        assert!(
            record.contains(field),
            "the frozen ResourceLease field {field} must ride the record fixture"
        );
    }
    let request = read_fixture("lease-request/typical.json");
    assert!(request.contains("\"mode\": \"write\""));
    assert!(request.contains("\"policy\": \"queue\""));
    let _ = ConflictCase {
        request: test_ok(dev_write_request()),
        holder: flauz_lease::fakes::ana_write_lease(flauz_lease::ConflictPolicy::Reject),
        now: now(),
    };
}
