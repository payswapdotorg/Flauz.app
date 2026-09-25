//! Kernel conformance tests (Wave-5 addendum §1-§7, the takeover
//! fabric): canonical JSON, the fixture set, the takeover law
//! (attribution structural, same-stream, projection-preserved, the
//! handback explicit), the approval space (named needs, attributed
//! decisions, the denial consequence), the propagation matrix (every
//! dependent named, attributed work kept, one run record),
//! determinism, and the no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_takeover::approval::{ApprovalDecision, ApprovalGate, DecisionEffect};
use flauz_takeover::cancel::{
    CancelledWhat, CancellationRecord, DependentTerminal, GraphShape, Propagation,
    propagate_cancellation,
};
use flauz_takeover::events::{
    ApprovalDecidedPayload, ApprovalRequestedPayload, CancelledPayload, DependentCancelledPayload,
    EventKind, TakeoverHandbackPayload, TakeoverStartedPayload,
};
use flauz_takeover::fakes::{
    PropagationCase, ana_takes_over_research, cancel_run, env_switch_gate, fake_actors, fake_task,
    now, propagation_matrix,
};
use flauz_takeover::takeover::{HandbackOutcome, HandbackRecord, TakeoverRecord};
use flauz_takeover::{ActorRef, TakeoverVersion, Timestamp};

/// The credential-material marker family (the PROV-001/LEASE-001
/// scan, re-pinned locally — this crate imports no contract crate).
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

/// Every committed fixture of the new families, for the credential
/// scan (kept as one list so a new fixture cannot silently escape
/// it).
const ALL_FIXTURES: &[&str] = &[
    "takeover-record/typical.json",
    "takeover-record/minimal.json",
    "takeover-record/invalid-not-human.json",
    "handback-record/typical.json",
    "handback-record/invalid-before-takeover.json",
    "approval-gate/typical.json",
    "approval-gate/minimal.json",
    "approval-gate/invalid-missing-consequence.json",
    "approval-decision/approved.json",
    "approval-decision/denied.json",
    "approval-decision/invalid-agent-decides.json",
    "cancellation-record/node.json",
    "cancellation-record/run.json",
    "cancellation-record/invalid-empty-reason.json",
    "graph-shape/typical.json",
    "graph-shape/minimal.json",
    "graph-shape/invalid-unknown-dependency.json",
    "propagation/leaf.json",
    "propagation/mid.json",
    "propagation/root.json",
    "propagation/run.json",
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
    // checkout with autocrlf translates them to CRLF. Normalize
    // before comparison so the round-trip law is tested against the
    // CANONICAL form, not the platform's line-ending translation.
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
    for relative in [
        "takeover-record/typical.json",
        "takeover-record/minimal.json",
        "handback-record/typical.json",
        "approval-gate/typical.json",
        "approval-gate/minimal.json",
        "approval-decision/approved.json",
        "approval-decision/denied.json",
        "cancellation-record/node.json",
        "cancellation-record/run.json",
        "graph-shape/typical.json",
        "graph-shape/minimal.json",
        "propagation/leaf.json",
        "propagation/mid.json",
        "propagation/root.json",
        "propagation/run.json",
    ] {
        let content = read_fixture(relative);
        match relative {
            "handback-record/typical.json" => {
                assert_eq!(round_trip::<HandbackRecord>(&content), content);
            }
            "approval-gate/typical.json" | "approval-gate/minimal.json" => {
                assert_eq!(round_trip::<ApprovalGate>(&content), content);
            }
            "approval-decision/approved.json" | "approval-decision/denied.json" => {
                assert_eq!(round_trip::<ApprovalDecision>(&content), content);
            }
            "cancellation-record/node.json" | "cancellation-record/run.json" => {
                assert_eq!(round_trip::<CancellationRecord>(&content), content);
            }
            "graph-shape/typical.json" | "graph-shape/minimal.json" => {
                assert_eq!(round_trip::<GraphShape>(&content), content);
            }
            "propagation/leaf.json"
            | "propagation/mid.json"
            | "propagation/root.json"
            | "propagation/run.json" => {
                assert_eq!(round_trip::<Propagation>(&content), content);
            }
            _ => {
                assert_eq!(round_trip::<TakeoverRecord>(&content), content);
            }
        }
    }
}

/// Invalid fixtures fail strict parses or canonical validation — a
/// record that violates the frozen rules is rejected on read or on
/// validate, never silently misread.
#[test]
fn invalid_fixtures_fail_strict_parses_or_validation() {
    let not_human = read_fixture("takeover-record/invalid-not-human.json");
    let record: TakeoverRecord = test_ok(serde_json::from_str(&not_human));
    assert!(
        record.validate().is_err(),
        "a takeover by a non-human actor must be rejected"
    );
    let too_early = read_fixture("handback-record/invalid-before-takeover.json");
    let handback: HandbackRecord = test_ok(serde_json::from_str(&too_early));
    assert!(
        handback.validate().is_err(),
        "a handback at or before its takeover moment must be rejected"
    );
    let missing_consequence = read_fixture("approval-gate/invalid-missing-consequence.json");
    let gate: ApprovalGate = test_ok(serde_json::from_str(&missing_consequence));
    assert!(
        gate.validate().is_err(),
        "a gate without a denial consequence must be rejected"
    );
    let agent_decides = read_fixture("approval-decision/invalid-agent-decides.json");
    let decision: ApprovalDecision = test_ok(serde_json::from_str(&agent_decides));
    assert!(
        decision.validate().is_err(),
        "an agent's decision must be rejected (HumanApproval != AgentDecision)"
    );
    let empty_reason = read_fixture("cancellation-record/invalid-empty-reason.json");
    let record: CancellationRecord = test_ok(serde_json::from_str(&empty_reason));
    assert!(
        record.validate().is_err(),
        "a cancellation without a reason must be rejected"
    );
    let unknown_dependency = read_fixture("graph-shape/invalid-unknown-dependency.json");
    let graph: GraphShape = test_ok(serde_json::from_str(&unknown_dependency));
    assert!(
        graph.validate().is_err(),
        "a graph with an unknown dependency must be rejected"
    );
}

/// The honest-artifact law: every committed propagation fixture
/// EQUALS the projection run over the committed cancellation-record
/// and graph-shape fixtures it documents — the fixtures are honest
/// artifacts of the projection, never hand-staged shapes.
#[test]
fn propagation_fixtures_are_the_projection_output_over_their_inputs() {
    let graph: GraphShape =
        test_ok(serde_json::from_str(&read_fixture("graph-shape/typical.json")));

    let leaf: CancellationRecord =
        test_ok(serde_json::from_str(&read_fixture("cancellation-record/node.json")));
    // The node fixture is the MID cancellation (analysis); the leaf
    // and root fixtures pin their own records from the same family.
    let leaf_record = test_ok(CancellationRecord::new(
        leaf.task.clone(),
        CancelledWhat::Node {
            node: test_ok(flauz_takeover::NodeName::parse("report")),
        },
        "the report is not needed anymore",
        leaf.actor.clone(),
        leaf.cancelled_at,
    ));
    let root_record = test_ok(CancellationRecord::new(
        leaf.task.clone(),
        CancelledWhat::Node {
            node: test_ok(flauz_takeover::NodeName::parse("research")),
        },
        "the research direction changed",
        leaf.actor.clone(),
        leaf.cancelled_at,
    ));
    let run_record: CancellationRecord =
        test_ok(serde_json::from_str(&read_fixture("cancellation-record/run.json")));

    for (relative, record) in [
        ("propagation/leaf.json", &leaf_record),
        ("propagation/mid.json", &leaf),
        ("propagation/root.json", &root_record),
        ("propagation/run.json", &run_record),
    ] {
        let propagation = test_ok(propagate_cancellation(record, &graph));
        assert_eq!(
            test_ok(serde_json::to_string_pretty(&propagation)),
            read_fixture(relative),
            "fixture {relative} is not the projection's honest output"
        );
    }
}

/// The takeover round-trip law (addendum §3): take over → the human's
/// work attributed on the SAME task stream → the explicit handback →
/// the agent resumes with its prior work preserved VERBATIM.
#[test]
fn the_takeover_round_trip_law() {
    let actors = fake_actors();
    let takeover = ana_takes_over_research();

    // 1. Attribution is structural: the record names the human, the
    //    agent, the task stream, the moment and the reason.
    assert_eq!(takeover.human, actors.ana);
    assert_eq!(takeover.from_agent, actors.dev_agent);
    assert_eq!(takeover.stream_task(), &fake_task());

    // 2. The same-stream law: the human's turn is a TASK event — the
    //    started payload rides the task's own stream, and there is no
    //    second store anywhere in the payload's shape.
    let started = test_ok(TakeoverStartedPayload::new(takeover.clone()));
    let serialized = test_ok(serde_json::to_string(&started));
    assert_eq!(EventKind::TakeoverStarted.as_str(), "task.takeover_started");
    assert!(serialized.contains("\"task\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\""));

    // 3. The handback is explicit and validated against its takeover.
    let handback = test_ok(HandbackRecord::for_takeover(
        takeover.clone(),
        test_ok(Timestamp::parse("2026-09-23T15:26:00Z")),
        "reviewed the sandbox credentials and approved the switch",
        HandbackOutcome::AgentResumes,
    ));
    assert_eq!(handback.human(), &takeover.human, "the handback is attributed");
    assert_eq!(handback.stream_task(), takeover.stream_task());

    // 4. The projection law: the agent resumes with EXACTLY the
    //    preserved work — verbatim, nothing dropped, nothing
    //    rewritten.
    assert_eq!(handback.preserved(), takeover.preserved.as_slice());
    assert_eq!(handback.preserved().len(), 1);
    assert_eq!(handback.preserved()[0].label, "research notes");

    // 5. The committed fixtures are this round-trip: the handback
    //    fixture EQUALS the handback constructed over the takeover
    //    fixture.
    let takeover_fixture: TakeoverRecord =
        test_ok(serde_json::from_str(&read_fixture("takeover-record/typical.json")));
    assert_eq!(takeover_fixture, takeover);
    let handback_fixture: HandbackRecord =
        test_ok(serde_json::from_str(&read_fixture("handback-record/typical.json")));
    assert_eq!(handback_fixture, handback);
}

/// The approval space (addendum §3): named needs, attributed human
/// decisions, and the denial consequence — an approval proceeds, a
/// denial fails the node honestly with the named consequence, and an
/// agent can decide neither.
#[test]
fn the_approval_space_is_honest() {
    let actors = fake_actors();
    let gate = test_ok(env_switch_gate());

    // The named need and both consequences ride the gate.
    assert_eq!(gate.need, "approve the environment switch");
    assert_eq!(
        gate.approve_consequence,
        "moving to the remote sandbox will re-run the setup steps"
    );
    assert_eq!(
        gate.deny_consequence,
        "this step stops with your decision recorded as the reason"
    );

    // Approve: the node proceeds.
    let approved = test_ok(ApprovalDecision::approve(
        gate.clone(),
        actors.ana.clone(),
        now(),
    ));
    assert!(matches!(
        approved.effect(),
        DecisionEffect::NodeProceeds { .. }
    ));

    // Deny: the node fails honestly with the denial as its reason —
    // never a silent proceed.
    let denied = test_ok(ApprovalDecision::deny(gate.clone(), actors.ana.clone(), now()));
    match denied.effect() {
        DecisionEffect::NodeFailed { reason, .. } => {
            assert_eq!(
                *reason,
                "Declined by the human: this step stops with your decision recorded as the reason"
            );
        }
        DecisionEffect::NodeProceeds { .. } => {
            panic!("a denial must never silently proceed");
        }
    }

    // Only a human decides (HumanApproval != AgentDecision).
    assert!(
        ApprovalDecision::approve(gate.clone(), actors.ana_as_agent(), now()).is_err(),
        "an agent cannot approve"
    );
    assert!(
        ApprovalDecision::deny(gate, actors.ana_as_agent(), now()).is_err(),
        "an agent cannot deny"
    );

    // The committed decision fixtures are these two decisions.
    let approved_fixture: ApprovalDecision =
        test_ok(serde_json::from_str(&read_fixture("approval-decision/approved.json")));
    assert_eq!(approved_fixture, approved);
    let denied_fixture: ApprovalDecision =
        test_ok(serde_json::from_str(&read_fixture("approval-decision/denied.json")));
    assert_eq!(denied_fixture, denied);

    // The payloads ride the same vocabulary.
    assert_eq!(EventKind::ApprovalRequested.as_str(), "task.approval_requested");
    assert_eq!(EventKind::ApprovalDecided.as_str(), "task.approval_decided");
    test_ok(ApprovalRequestedPayload::new(test_ok(env_switch_gate())));
    test_ok(ApprovalDecidedPayload::new(approved));
    test_ok(ApprovalDecidedPayload::new(denied));
}

/// The propagation matrix (addendum §4) over the COMPLETE fake
/// propagation space: every dependent is named (in the graph's node
/// order, never twice), every terminal state is one of the two named
/// states (cancelled, or blocked-resolved with the reason),
/// already-attributed work is kept, and a run cancellation carries
/// exactly ONE run-level record.
#[test]
fn the_propagation_matrix_names_every_dependent() {
    for PropagationCase { record, graph } in propagation_matrix() {
        let propagation = test_ok(propagate_cancellation(&record, &graph));
        assert!(
            propagation.every_dependent_terminal(),
            "no dependent may sit silently Blocked ({:?})",
            record.what
        );
        // Dependents are named in the graph's node order, never twice.
        let mut order: Vec<&str> = graph.nodes.iter().map(|node| node.node.as_str()).collect();
        let mut named: Vec<&str> = propagation
            .dependents
            .iter()
            .map(|dependent| dependent.node.as_str())
            .collect();
        assert_eq!(named.len(), named.iter().collect::<std::collections::BTreeSet<_>>().len());
        order.retain(|candidate| named.contains(candidate));
        assert_eq!(named, order, "dependents follow the graph's node order");
        // Every dependent's terminal state carries the reason.
        for dependent in &propagation.dependents {
            let reason = match &dependent.terminal {
                DependentTerminal::Cancelled { reason }
                | DependentTerminal::BlockedResolved { reason } => reason,
            };
            assert_eq!(reason, &record.reason, "the truth is the original reason");
            // A blocked-resolved dependent keeps its attributed work;
            // a cancelled one that produced nothing keeps nothing.
            match dependent.terminal {
                DependentTerminal::BlockedResolved { .. } => assert!(!dependent.kept.is_empty()),
                DependentTerminal::Cancelled { .. } => {}
            }
        }
        // The run-level record law: a run cancellation carries exactly
        // one; a node cancellation carries none.
        if record.what.is_run() {
            assert!(propagation.carries_run_record());
            let serialized = test_ok(serde_json::to_string(&propagation));
            assert_eq!(serialized.matches("\"what\":{\"kind\":\"run\"}").count(), 1);
        } else {
            assert!(!propagation.carries_run_record());
        }
    }
}

/// Determinism (kernel §7): identical inputs produce byte-identical
/// records and projections over the whole fake space — no wall
/// clock, no entropy, no order sensitivity.
#[test]
fn records_and_projections_are_deterministic() {
    for PropagationCase { record, graph } in propagation_matrix() {
        let first = test_ok(propagate_cancellation(&record, &graph));
        let second = test_ok(propagate_cancellation(&record, &graph));
        assert_eq!(
            test_ok(serde_json::to_string(&first)),
            test_ok(serde_json::to_string(&second)),
            "the same inputs must produce identical bytes ({:?})",
            record.what
        );
    }
    let takeover_first = ana_takes_over_research();
    let takeover_second = ana_takes_over_research();
    assert_eq!(
        test_ok(serde_json::to_string(&takeover_first)),
        test_ok(serde_json::to_string(&takeover_second))
    );
    let gate_first = test_ok(env_switch_gate());
    let gate_second = test_ok(env_switch_gate());
    assert_eq!(
        test_ok(serde_json::to_string(&gate_first)),
        test_ok(serde_json::to_string(&gate_second))
    );
    let run_first = test_ok(cancel_run());
    let run_second = test_ok(cancel_run());
    assert_eq!(
        test_ok(serde_json::to_string(&run_first)),
        test_ok(serde_json::to_string(&run_second))
    );
}

/// Credentials are references, forever (the standing law): no
/// credential material appears in any serialized fake family or any
/// committed fixture of the new families.
#[test]
fn no_credential_material_in_serialized_state() {
    // Every fake family, serialized.
    let mut haystack = String::new();
    for PropagationCase { record, graph } in propagation_matrix() {
        let propagation = test_ok(propagate_cancellation(&record, &graph));
        haystack.push_str(&test_ok(serde_json::to_string(&propagation)));
    }
    haystack.push_str(&test_ok(serde_json::to_string(&ana_takes_over_research())));
    haystack.push_str(&test_ok(serde_json::to_string(&test_ok(env_switch_gate()))));
    let actors = fake_actors();
    haystack.push_str(&test_ok(serde_json::to_string(&actors.ana)));
    let handback = test_ok(HandbackRecord::for_takeover(
        ana_takes_over_research(),
        test_ok(Timestamp::parse("2026-09-23T15:26:00Z")),
        "reviewed the sandbox credentials and approved the switch",
        HandbackOutcome::AgentResumes,
    ));
    haystack.push_str(&test_ok(serde_json::to_string(&handback)));
    let decision = test_ok(ApprovalDecision::approve(
        test_ok(env_switch_gate()),
        test_ok(ActorRef::user("user_ana")),
        now(),
    ));
    haystack.push_str(&test_ok(serde_json::to_string(&decision)));
    let dependent_payload = test_ok(DependentCancelledPayload::new(
        fake_task(),
        test_ok(flauz_takeover::cancel::DependentResolution {
            v: TakeoverVersion,
            node: test_ok(flauz_takeover::NodeName::parse("analysis")),
            waited_on: Some(test_ok(flauz_takeover::NodeName::parse("research"))),
            terminal: DependentTerminal::Cancelled {
                reason: "the research direction changed".to_owned(),
            },
            kept: Vec::new(),
        }),
    ));
    haystack.push_str(&test_ok(serde_json::to_string(&dependent_payload)));
    test_ok(CancelledPayload::new(test_ok(cancel_run())));
    test_ok(TakeoverStartedPayload::new(ana_takes_over_research()));
    test_ok(TakeoverHandbackPayload::new(handback));
    for marker in CREDENTIAL_MARKERS {
        assert!(
            !haystack.contains(marker),
            "credential marker {marker:?} must never appear in serialized takeover state"
        );
    }

    // Every committed fixture, read as text.
    for relative in ALL_FIXTURES {
        let content = read_fixture(relative);
        for marker in CREDENTIAL_MARKERS {
            assert!(
                !content.contains(marker),
                "credential marker {marker:?} must never appear in fixture {relative}"
            );
        }
    }
}

/// The frozen wire shapes ride every record: the event vocabulary is
/// the exact `task.*` dotted grammar, scopes and decisions tag with
/// `"kind"`, and the takeover/approval/cancellation field names are
/// the frozen snake_case forms.
#[test]
fn the_frozen_wire_shapes_ride_the_fixtures() {
    // The event vocabulary, exactly.
    for (kind, text) in [
        (EventKind::TakeoverStarted, "task.takeover_started"),
        (EventKind::TakeoverHandback, "task.takeover_handback"),
        (EventKind::ApprovalRequested, "task.approval_requested"),
        (EventKind::ApprovalDecided, "task.approval_decided"),
        (EventKind::Cancelled, "task.cancelled"),
        (EventKind::DependentCancelled, "task.dependent_cancelled"),
    ] {
        assert_eq!(kind.as_str(), text);
    }
    assert_eq!(EventKind::ALL.len(), 6);

    // The takeover fixture's frozen fields.
    let takeover = read_fixture("takeover-record/typical.json");
    for field in [
        "\"v\": 1",
        "\"task\":",
        "\"from_agent\":",
        "\"human\":",
        "\"taken_at\":",
        "\"reason\":",
        "\"preserved\":",
    ] {
        assert!(
            takeover.contains(field),
            "the frozen takeover field {field} must ride the fixture"
        );
    }
    assert!(takeover.contains("\"kind\": \"node\""));

    // The gate fixture's named need and both consequences.
    let gate = read_fixture("approval-gate/typical.json");
    for field in [
        "\"need\":",
        "\"approve_consequence\":",
        "\"deny_consequence\":",
        "\"requested_by\":",
    ] {
        assert!(
            gate.contains(field),
            "the frozen gate field {field} must ride the fixture"
        );
    }

    // The cancellation fixture's distinct-from-failure shape.
    let cancellation = read_fixture("cancellation-record/node.json");
    assert!(cancellation.contains("\"what\": {"));
    assert!(cancellation.contains("\"kind\": \"node\""));

    // The decision fixtures' attributed effects.
    let denied = read_fixture("approval-decision/denied.json");
    assert!(denied.contains("\"kind\": \"denied\""));
    assert!(denied.contains("\"kind\": \"node_failed\""));
    assert!(denied.contains("Declined by the human:"));
    let approved = read_fixture("approval-decision/approved.json");
    assert!(approved.contains("\"kind\": \"approved\""));
    assert!(approved.contains("\"kind\": \"node_proceeds\""));

    // The propagation fixture's named terminal states.
    let root = read_fixture("propagation/root.json");
    assert!(root.contains("\"kind\": \"cancelled\""));
    assert!(root.contains("\"waited_on\": \"research\""));
    assert!(root.contains("\"kept_work\": ["));

    // Takeovers never fork the task (kernel §6): every record in the
    // family names the same task stream.
    assert_eq!(ana_takes_over_research().stream_task(), &fake_task());
}
