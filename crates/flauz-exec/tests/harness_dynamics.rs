//! Harness dynamics (ORCH-003): the event-sourced state machine, the
//! recovery law (replay of the machine's own history, never of model
//! output), the compile/store seams, the view-model export, and the
//! telemetry projection — with the canonical-JSON fixture round-trips
//! under `tests/fixtures/w3/`.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use flauz_exec::capability::CapabilityId;
use flauz_exec::fakes::{
    FAKE_CONTEXT_REF, FakeCodexRuntime, FakeContextCompiler, FakeHarnessObserver,
    FakeNonCodexRuntime, fake_recovery_brief,
};
use flauz_exec::harness::{
    HarnessError, HarnessEventDetail, HarnessEventRecord, HarnessStateKind, HarnessTransition,
    RecoveryBrief, TaskHarness, replay_history, transition_is_legal,
};
use flauz_exec::harness_telemetry::{HarnessSpanOutcome, HarnessTrace, MAX_HARNESS_TRACE_SPANS};
use flauz_exec::ids::{EnvironmentId, ModelId};
use flauz_exec::refs::ActorRef;
use flauz_exec::runtime::{RuntimeRequest, RuntimeStatus};
use flauz_exec::time::Timestamp;

const TASK_ID: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";
const MODEL_ID: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPX0";
const OTHER_MODEL_ID: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPY1";
const ENVIRONMENT_ID: &str = "env_01J8ZQ5V8K3T2B7N6X4R9DQPE4";
const ACTOR_AGENT: &str = "agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6";

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/w3")
        .join(relative)
}

fn read_fixture(relative: &str) -> String {
    let content = fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
    content.replace("\r\n", "\n").trim_end().to_owned()
}

fn ts(value: &str) -> Timestamp {
    test_ok(Timestamp::parse(value))
}

fn actor() -> ActorRef {
    test_ok(ActorRef::agent(ACTOR_AGENT))
}

fn model_id() -> ModelId {
    test_ok(ModelId::parse(MODEL_ID))
}

fn other_model_id() -> ModelId {
    test_ok(ModelId::parse(OTHER_MODEL_ID))
}

fn environment_id() -> EnvironmentId {
    test_ok(EnvironmentId::parse(ENVIRONMENT_ID))
}

fn parse_capability(key: &str) -> CapabilityId {
    test_ok(CapabilityId::parse(key))
}

/// A harness prepared against the fake Codex runtime with the fake
/// compile seam and the fake observer.
fn prepared_harness() -> TaskHarness {
    test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        Some(environment_id()),
        Box::new(FakeCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        Arc::new(FakeHarnessObserver::new()),
        ts("2026-09-21T13:45:00Z"),
    ))
}

fn turn_request(model: ModelId) -> RuntimeRequest {
    test_ok(RuntimeRequest::new(
        model,
        Some(environment_id()),
        vec![parse_capability("terminal")],
        "Run the verification turn",
    ))
}

/// The full lifecycle: every transition emits exactly one event, the
/// stream sequences strictly, and the states land where the lattice
/// says.
#[test]
fn harness_lifecycle_transitions_are_event_sourced() {
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        Some(environment_id()),
        Box::new(FakeCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    assert_eq!(harness.state(), HarnessStateKind::Preparing);
    assert_eq!(harness.seq(), 1);

    let outcome =
        test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:46:00Z")));
    assert_eq!(outcome.status, RuntimeStatus::Completed);
    assert_eq!(harness.state(), HarnessStateKind::Executing);
    assert_eq!(harness.turns(), 1);

    test_ok(harness.observe(
        "obs_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
        "res_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
        "api",
        ts("2026-09-21T13:47:00Z"),
    ));
    test_ok(harness.verify(
        "claim_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
        "evd_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
        "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
        ts("2026-09-21T13:48:00Z"),
    ));
    test_ok(harness.persist(
        vec![
            "art_01J8ZQ5V8K3T2B7N6X4R9DQPA0".to_owned(),
            "evd_01J8ZQ5V8K3T2B7N6X4R9DQPC2".to_owned(),
        ],
        ts("2026-09-21T13:49:00Z"),
    ));
    test_ok(harness.continue_cycle(ts("2026-09-21T13:50:00Z")));
    assert_eq!(harness.state(), HarnessStateKind::Continuing);

    let outcome =
        test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:51:00Z")));
    assert_eq!(outcome.status, RuntimeStatus::Completed);
    test_ok(harness.persist(Vec::new(), ts("2026-09-21T13:52:00Z")));
    test_ok(harness.complete(ts("2026-09-21T13:53:00Z")));
    assert_eq!(harness.state(), HarnessStateKind::Done);

    let records = observer.records();
    assert_eq!(records.len(), 9, "one event per transition");
    let expected = [
        (HarnessStateKind::Preparing, "harness.prepared"),
        (HarnessStateKind::Executing, "harness.executed"),
        (HarnessStateKind::Observing, "harness.observed"),
        (HarnessStateKind::Verifying, "harness.verified"),
        (HarnessStateKind::Persisting, "harness.persisted"),
        (HarnessStateKind::Continuing, "harness.continued"),
        (HarnessStateKind::Executing, "harness.executed"),
        (HarnessStateKind::Persisting, "harness.persisted"),
        (HarnessStateKind::Done, "harness.completed"),
    ];
    for (index, record) in records.iter().enumerate() {
        let (state, event_type) = expected[index];
        assert_eq!(record.to_state, state);
        assert_eq!(record.event_type, event_type);
        assert_eq!(record.seq, (index + 1) as u64);
        assert_eq!(record.task_id, TASK_ID, "task identity never changes");
    }
    assert_eq!(
        records[8].seq, 9,
        "the completion closes the stream at seq 9"
    );
}

/// The lattice is enforced: illegal transitions are rejected with
/// [`HarnessError::IllegalTransition`], the terminal states accept
/// nothing, and every first-class continuation is reachable.
#[test]
fn harness_illegal_transitions_are_rejected() {
    let mut harness = prepared_harness();
    // Observe before executing: illegal from Preparing.
    match harness.observe(
        "obs_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
        "res_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
        "api",
        ts("2026-09-21T13:46:00Z"),
    ) {
        Err(HarnessError::IllegalTransition { from, transition }) => {
            assert_eq!(from, HarnessStateKind::Preparing);
            assert_eq!(transition, HarnessTransition::Observe);
        }
        other => panic!("expected IllegalTransition, got {other:?}"),
    }
    assert_eq!(
        harness.state(),
        HarnessStateKind::Preparing,
        "no transition happened"
    );
    assert_eq!(harness.seq(), 1, "no event was emitted");

    // The lattice table itself.
    assert!(transition_is_legal(None, HarnessTransition::Prepare));
    assert!(!transition_is_legal(None, HarnessTransition::Execute));
    for terminal in [HarnessStateKind::Done, HarnessStateKind::Failed] {
        for transition in HarnessTransition::ALL {
            assert!(
                !transition_is_legal(Some(terminal), transition),
                "{terminal:?} must accept nothing"
            );
        }
    }
    // The three continuations are first-class and reachable.
    for (from, continuation) in [
        (HarnessStateKind::Persisting, HarnessTransition::Continue),
        (HarnessStateKind::Executing, HarnessTransition::Recover),
        (HarnessStateKind::Continuing, HarnessTransition::Recover),
        (HarnessStateKind::Recovering, HarnessTransition::Continue),
        (HarnessStateKind::Escalated, HarnessTransition::Continue),
    ] {
        assert!(
            transition_is_legal(Some(from), continuation),
            "{continuation:?} must be legal from {from:?}"
        );
    }
    // A model switch is a re-prepare from Continuing.
    assert!(transition_is_legal(
        Some(HarnessStateKind::Continuing),
        HarnessTransition::Prepare
    ));

    // Terminal machines reject everything, live.
    let mut done = prepared_harness();
    test_ok(done.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:46:00Z")));
    test_ok(done.persist(Vec::new(), ts("2026-09-21T13:47:00Z")));
    test_ok(done.complete(ts("2026-09-21T13:48:00Z")));
    assert!(done.continue_cycle(ts("2026-09-21T13:49:00Z")).is_err());
    assert!(
        done.fail("late failure", ts("2026-09-21T13:50:00Z"))
            .is_err()
    );
}

/// THE recovery law: the machine reconstructs from its OWN history —
/// serialize the records, drop everything, reload, replay — and model
/// output never appears in the stream (the fake runtime's summary text
/// is absent; an injected summary field is rejected).
#[test]
fn harness_recovery_replays_the_machines_own_history_never_model_output() {
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        Some(environment_id()),
        Box::new(FakeCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    let outcome =
        test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:46:00Z")));
    test_ok(harness.observe(
        "obs_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
        "res_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
        "api",
        ts("2026-09-21T13:47:00Z"),
    ));
    test_ok(harness.verify(
        "claim_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
        "evd_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
        "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
        ts("2026-09-21T13:48:00Z"),
    ));
    test_ok(harness.persist(
        vec!["art_01J8ZQ5V8K3T2B7N6X4R9DQPA0".to_owned()],
        ts("2026-09-21T13:49:00Z"),
    ));
    // ... interruption: everything drops.
    let records = observer.records();
    let stream_json = test_ok(serde_json::to_string(&records));
    drop(harness);
    drop(observer);

    // Model output never entered the stream: the runtime's summary is
    // recorded nowhere, and an injected summary field is rejected.
    assert!(
        !stream_json.contains(&outcome.summary),
        "the runtime summary must never enter the machine's history"
    );
    assert!(stream_json.contains("harness.prepared"));
    let poisoned = stream_json.replace("\"turn\":1,", "\"turn\":1,\"summary\":\"model output\",");
    assert!(
        serde_json::from_str::<Vec<HarnessEventRecord>>(&poisoned).is_err(),
        "an unknown summary field is rejected (deny_unknown_fields)"
    );

    // Reload + replay: the machine's own history reconstructs it.
    let reloaded: Vec<HarnessEventRecord> = test_ok(serde_json::from_str(&stream_json));
    let replayed = test_ok(TaskHarness::replay(&reloaded));
    assert_eq!(replayed.task_id, TASK_ID);
    assert_eq!(replayed.state, HarnessStateKind::Persisting);
    assert_eq!(replayed.turns, 1);
    assert_eq!(replayed.prepares, 1);
    assert_eq!(replayed.seq, 5);
    let attachments = replayed
        .attachments
        .clone()
        .unwrap_or_else(|| panic!("the replayed machine carries its attachments"));
    assert_eq!(attachments.model_id.as_str(), MODEL_ID);
    assert_eq!(attachments.context_ref.as_deref(), Some(FAKE_CONTEXT_REF));

    // Resume: the first-class recovery transition, then continue.
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut resumed = test_ok(TaskHarness::resume(
        replayed,
        actor(),
        Box::new(FakeCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        observer.clone(),
    ));
    assert_eq!(resumed.state(), HarnessStateKind::Persisting);
    assert!(resumed.is_recoverable());
    let brief = test_ok(fake_recovery_brief());
    let record = test_ok(resumed.recover(brief, ts("2026-09-21T14:00:00Z")));
    assert_eq!(resumed.state(), HarnessStateKind::Recovering);
    assert_eq!(record.seq, 6, "the stream continues where it left off");
    match &record.detail {
        HarnessEventDetail::Recover {
            replayed_events,
            kept,
            compaction,
        } => {
            assert_eq!(
                *replayed_events, 5,
                "resumed from five of the machine's own events"
            );
            assert_eq!(kept.total(), 3);
            assert_eq!(
                compaction.as_ref().map(|record| record.summarized_items),
                Some(2)
            );
        }
        other => panic!("expected a recover detail, got {other:?}"),
    }
    test_ok(resumed.continue_cycle(ts("2026-09-21T14:01:00Z")));
    let outcome =
        test_ok(resumed.execute_turn(&turn_request(model_id()), ts("2026-09-21T14:02:00Z")));
    assert_eq!(outcome.status, RuntimeStatus::Completed);
    assert_eq!(
        resumed.turns(),
        2,
        "the turn counter continued from the replay"
    );
    assert_eq!(resumed.seq(), 8);
    assert_eq!(
        observer.records().len(),
        3,
        "only the new transitions emitted"
    );
}

/// Recovery demands a legal history: corrupted streams are rejected,
/// each with the concrete rule that fired.
#[test]
fn harness_recovery_requires_a_legal_history() {
    // Empty history.
    assert!(matches!(replay_history(&[]), Err(HarnessError::Invalid(_))));

    let mut records = Vec::new();
    // A healthy prefix: prepare + execute.
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        Some(environment_id()),
        Box::new(FakeCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:46:00Z")));
    records.clone_from(&observer.records());

    // A sequence gap.
    let mut gapped = records.clone();
    gapped[1].seq = 3;
    assert!(matches!(
        replay_history(&gapped),
        Err(HarnessError::Invalid(reason)) if reason.contains("does not continue")
    ));

    // A task-id switch mid-stream (a fork attempt).
    let mut forked = records.clone();
    forked[1].task_id = "task_01J8ZQ5V8K3T2B7N6X4R9DQPZ9".to_owned();
    assert!(matches!(
        replay_history(&forked),
        Err(HarnessError::Invalid(reason)) if reason.contains("belongs to task")
    ));

    // A backwards timestamp.
    let mut rewound = records.clone();
    rewound[1].occurred_at = ts("2026-09-21T13:44:00Z");
    assert!(matches!(
        replay_history(&rewound),
        Err(HarnessError::Invalid(reason)) if reason.contains("backwards in time")
    ));

    // An illegal transition sequence (continue straight from
    // Executing: the lattice requires a persist first).
    let mut illegal = records.clone();
    illegal.push(test_ok(HarnessEventRecord::new(
        "harness.continued",
        3,
        TASK_ID,
        ts("2026-09-21T13:47:00Z"),
        actor(),
        HarnessStateKind::Continuing,
        HarnessEventDetail::Continue,
    )));
    assert!(matches!(
        replay_history(&illegal),
        Err(HarnessError::Invalid(reason)) if reason.contains("illegal transition")
    ));

    // A stream that does not start at the origin.
    let mut orphan = records.clone();
    orphan.remove(0);
    assert!(replay_history(&orphan).is_err());

    // A healthy stream still replays (control).
    assert!(replay_history(&records).is_ok());
}

/// The compile seam: prepare drives it with the attached model and
/// environment, the context reference lands in the attachments and the
/// event, and a model switch is a re-prepare that keeps the task
/// identity and continues the stream (kernel §6).
#[test]
fn harness_prepare_compiles_through_the_seam_and_survives_model_switch() {
    let compiler = Arc::new(FakeContextCompiler::new());
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        Some(environment_id()),
        Box::new(FakeCodexRuntime::new()),
        compiler.clone(),
        observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    let inputs = compiler.compiled_inputs();
    assert_eq!(inputs.len(), 1, "the seam was driven exactly once");
    assert_eq!(inputs[0].task_id, TASK_ID);
    assert_eq!(inputs[0].model_id.as_str(), MODEL_ID);
    assert_eq!(inputs[0].environment_id.as_ref(), Some(&environment_id()));
    let attachments = harness
        .attachments()
        .cloned()
        .unwrap_or_else(|| panic!("the harness carries its attachments"));
    assert_eq!(attachments.runtime_kind, "codex-app-server");
    assert_eq!(attachments.context_ref.as_deref(), Some(FAKE_CONTEXT_REF));

    // A turn, then the model switch: re-prepare from Continuing.
    test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:46:00Z")));
    test_ok(harness.persist(Vec::new(), ts("2026-09-21T13:47:00Z")));
    test_ok(harness.continue_cycle(ts("2026-09-21T13:48:00Z")));
    let record = test_ok(harness.reprepare(
        other_model_id(),
        Some(environment_id()),
        Box::new(FakeNonCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        ts("2026-09-21T13:49:00Z"),
    ));
    assert_eq!(record.event_type, "harness.prepared");
    assert_eq!(harness.state(), HarnessStateKind::Preparing);
    assert_eq!(harness.prepares(), 2);
    assert_eq!(
        harness.task_id(),
        TASK_ID,
        "the task identity survived the switch"
    );
    let attachments = harness
        .attachments()
        .cloned()
        .unwrap_or_else(|| panic!("the harness carries its attachments"));
    assert_eq!(attachments.model_id.as_str(), OTHER_MODEL_ID);
    assert_eq!(attachments.runtime_kind, "flauz-direct");
    // The stream continued: seq 5, and the task id never changed.
    assert_eq!(harness.seq(), 5);
    assert!(
        observer
            .records()
            .iter()
            .all(|record| record.task_id == TASK_ID)
    );
    // The switched machine executes with the NEW model.
    let request = test_ok(RuntimeRequest::new(
        other_model_id(),
        Some(environment_id()),
        vec![parse_capability("browser.navigation")],
        "Run the research turn on the switched model",
    ));
    let outcome = test_ok(harness.execute_turn(&request, ts("2026-09-21T13:50:00Z")));
    assert_eq!(outcome.status, RuntimeStatus::Completed);
    // The old model is rejected against the new attachments.
    let stale = turn_request(model_id());
    assert!(
        harness
            .execute_turn(&stale, ts("2026-09-21T13:51:00Z"))
            .is_err()
    );
}

/// A capability gap is an honest outcome that still transitions; the
/// escalation from it is explicit, visible and recoverable.
#[test]
fn harness_capability_gap_turns_escalate_visibly() {
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        Some(environment_id()),
        // The browser-only fake cannot serve a terminal turn.
        Box::new(FakeNonCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    let outcome =
        test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:46:00Z")));
    assert_eq!(outcome.status, RuntimeStatus::CapabilityGap);
    assert_eq!(
        outcome.missing_capabilities,
        vec![parse_capability("terminal")]
    );
    assert_eq!(harness.state(), HarnessStateKind::Executing);

    // The gap escalates: explicit, visible, with a bounded reason.
    let record = test_ok(harness.escalate(
        "The runtime cannot use the terminal this task needs.",
        ts("2026-09-21T13:47:00Z"),
    ));
    assert_eq!(harness.state(), HarnessStateKind::Escalated);
    assert_eq!(record.event_type, "harness.escalated");
    match &record.detail {
        HarnessEventDetail::Escalate { reason } => {
            assert!(reason.contains("terminal"));
        }
        other => panic!("expected an escalate detail, got {other:?}"),
    }

    // The human reviews and continues; the run goes on.
    test_ok(harness.continue_cycle(ts("2026-09-21T13:48:00Z")));
    assert_eq!(harness.state(), HarnessStateKind::Continuing);
    // A browser turn now completes on the same machine.
    let request = test_ok(RuntimeRequest::new(
        model_id(),
        Some(environment_id()),
        vec![parse_capability("browser.navigation")],
        "Run the research turn instead",
    ));
    let outcome = test_ok(harness.execute_turn(&request, ts("2026-09-21T13:49:00Z")));
    assert_eq!(outcome.status, RuntimeStatus::Completed);
    // The gap's missing keys are durable data on the executed event.
    match &observer.records()[1].detail {
        HarnessEventDetail::Execute {
            missing_capabilities,
            ..
        } => assert_eq!(missing_capabilities, &vec![parse_capability("terminal")]),
        other => panic!("expected an execute detail, got {other:?}"),
    }
}

/// The view-model export: plain data, canonical round-trip, honest
/// flags across the state kinds (the ORCH-004 seam).
#[test]
fn harness_view_exports_plain_data_for_the_graph() {
    let mut harness = prepared_harness();
    let view = harness.view();
    assert!(view.recoverable);
    assert!(!view.escalated);
    assert!(!view.terminal);
    assert_eq!(view.events, 1);
    test_ok(view.validate());

    test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:46:00Z")));
    test_ok(harness.escalate(
        "The verification needs a human eye.",
        ts("2026-09-21T13:47:00Z"),
    ));
    let view = harness.view();
    assert!(view.escalated);
    assert!(
        view.recoverable,
        "an escalated task is resumable after review"
    );
    test_ok(view.validate());
    let serialized = test_ok(serde_json::to_string(&view));
    let reloaded = test_ok(serde_json::from_str::<flauz_exec::HarnessView>(&serialized));
    assert_eq!(reloaded, view);

    test_ok(harness.continue_cycle(ts("2026-09-21T13:48:00Z")));
    test_ok(harness.complete(ts("2026-09-21T13:49:00Z")));
    let view = harness.view();
    assert!(view.terminal);
    assert!(!view.recoverable);
    test_ok(view.validate());

    // Failure is a live, first-class ending too: a bounded reason, a
    // terminal view, and the stream still one-event-per-transition.
    let mut failing = prepared_harness();
    let record = test_ok(failing.fail(
        "The environment went away mid-run.",
        ts("2026-09-21T13:50:00Z"),
    ));
    assert_eq!(failing.state(), HarnessStateKind::Failed);
    assert_eq!(record.event_type, "harness.failed");
    match &record.detail {
        HarnessEventDetail::Fail { reason } => {
            assert!(reason.contains("environment"));
        }
        other => panic!("expected a fail detail, got {other:?}"),
    }
    let view = failing.view();
    assert!(view.terminal);
    assert!(!view.recoverable);
    assert!(!view.escalated);
    test_ok(view.validate());
    assert_eq!(failing.seq(), 2, "one event per transition, still");
}

/// Canonical-JSON fixtures: the committed records parse, validate, and
/// re-serialize byte-for-byte; the invalid fixtures are rejected.
#[test]
fn harness_events_round_trip_canonical_json_fixtures() {
    for fixture in [
        "harness-event/typical.json",
        "harness-event/minimal.json",
        "harness-event/boundary.json",
    ] {
        let content = read_fixture(fixture);
        let record = test_ok(serde_json::from_str::<HarnessEventRecord>(&content));
        test_ok(record.validate());
        assert_eq!(
            test_ok(serde_json::to_string(&record)),
            content,
            "{fixture} must round-trip byte-for-byte"
        );
    }
    let unknown_field = read_fixture("harness-event/invalid-unknown-field.json");
    assert!(
        serde_json::from_str::<HarnessEventRecord>(&unknown_field).is_err(),
        "unknown fields are rejected"
    );
    // The detail/state pairing is a canonical rule enforced by the
    // record's own validation (and therefore by replay), not by serde's
    // shape check alone: the record parses, then fails validation.
    let mismatched = read_fixture("harness-event/invalid-detail-mismatch.json");
    let parsed_mismatch = test_ok(serde_json::from_str::<HarnessEventRecord>(&mismatched));
    assert!(
        parsed_mismatch.validate().is_err(),
        "the detail must match the event's target state"
    );
    assert!(replay_history(&[parsed_mismatch]).is_err());
}

/// No credential material anywhere in the machine's serialized state.
#[test]
fn no_credential_material_in_harness_state() {
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        Some(environment_id()),
        Box::new(FakeCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:46:00Z")));
    test_ok(harness.recover(
        RecoveryBrief {
            kept: test_ok(flauz_exec::HarnessKeptRefs::new(
                vec!["art_01J8ZQ5V8K3T2B7N6X4R9DQPA0".to_owned()],
                Vec::new(),
                Vec::new(),
            )),
            compaction: None,
        },
        ts("2026-09-21T13:47:00Z"),
    ));
    let stream = test_ok(serde_json::to_string(&observer.records()));
    let view = test_ok(serde_json::to_string(&harness.view()));
    let replayed = test_ok(TaskHarness::replay(&observer.records()));
    let replay = test_ok(serde_json::to_string(&replayed));
    for serialized in [&stream, &view, &replay] {
        for forbidden in [
            "flausec_",
            "sk-",
            "ghp_",
            "api-key",
            "password",
            "secret_value",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "no credential material may appear in harness state ({forbidden})"
            );
        }
    }
}

/// The telemetry projection: durations in integer milliseconds, outcome
/// classifications for turns/gaps/recoveries/escalations/endings, and
/// the observability summary.
#[test]
fn harness_trace_projects_durations_and_outcomes() {
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        Some(environment_id()),
        Box::new(FakeCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    test_ok(harness.execute_turn(&turn_request(model_id()), ts("2026-09-21T13:45:30Z")));
    test_ok(harness.observe(
        "obs_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
        "res_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
        "api",
        ts("2026-09-21T13:46:00Z"),
    ));
    test_ok(harness.escalate("A decision needs the human.", ts("2026-09-21T13:47:00Z")));
    test_ok(harness.continue_cycle(ts("2026-09-21T13:48:00Z")));
    test_ok(harness.complete(ts("2026-09-21T13:49:00Z")));

    let trace = test_ok(HarnessTrace::project(TASK_ID, &observer.records()));
    assert_eq!(trace.spans.len(), 6);
    // Durations: 30s, 30s, 60s, 60s, 60s, 0 (the last span holds 0).
    let durations: Vec<u64> = trace.spans.iter().map(|span| span.duration_ms).collect();
    assert_eq!(durations, [30_000, 30_000, 60_000, 60_000, 60_000, 0]);
    assert_eq!(trace.total_duration_ms(), 240_000);
    // Outcomes.
    let outcomes: Vec<HarnessSpanOutcome> = trace.spans.iter().map(|span| span.outcome).collect();
    assert_eq!(
        outcomes,
        [
            HarnessSpanOutcome::Phase,
            HarnessSpanOutcome::TurnCompleted,
            HarnessSpanOutcome::Phase,
            HarnessSpanOutcome::Escalated,
            HarnessSpanOutcome::Phase,
            HarnessSpanOutcome::Completed,
        ]
    );
    // A gap turn classifies as a gap.
    let gap_observer = Arc::new(FakeHarnessObserver::new());
    let mut gap_harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        None,
        Box::new(FakeNonCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        gap_observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    let unbound_request = test_ok(RuntimeRequest::new(
        model_id(),
        None,
        vec![parse_capability("terminal")],
        "Run the terminal turn",
    ));
    test_ok(gap_harness.execute_turn(&unbound_request, ts("2026-09-21T13:46:00Z")));
    test_ok(gap_harness.recover(RecoveryBrief::default(), ts("2026-09-21T13:47:00Z")));
    let gap_trace = test_ok(HarnessTrace::project(TASK_ID, &gap_observer.records()));
    assert_eq!(
        gap_trace.spans[1].outcome,
        HarnessSpanOutcome::TurnCapabilityGap
    );
    assert_eq!(gap_trace.spans[2].outcome, HarnessSpanOutcome::Recovered);

    // The summary counts what the observability questions ask.
    let summary = trace.summary();
    assert_eq!(summary.spans, 6);
    assert_eq!(summary.turns_completed, 1);
    assert_eq!(summary.escalations, 1);
    assert_eq!(summary.completed_runs, 1);
    assert_eq!(summary.recoveries, 0);
    test_ok(summary.validate());
    test_ok(gap_trace.summary().validate());

    // A trace for the wrong task is rejected.
    assert!(HarnessTrace::project("task_01J8ZQ5V8K3T2B7N6X4R9DQPZ9", &observer.records()).is_err());
}

/// The trace is bounded and round-trips through canonical JSON; the
/// fixtures prove the shape.
#[test]
fn harness_trace_round_trips_and_is_bounded() {
    for fixture in ["harness-trace/typical.json", "harness-trace/minimal.json"] {
        let content = read_fixture(fixture);
        let trace = test_ok(serde_json::from_str::<HarnessTrace>(&content));
        test_ok(trace.validate());
        assert_eq!(
            test_ok(serde_json::to_string(&trace)),
            content,
            "{fixture} must round-trip byte-for-byte"
        );
        // The summary of a reloaded trace equals the original's (the
        // display-model replay).
        let reloaded = test_ok(serde_json::from_str::<HarnessTrace>(&content));
        assert_eq!(reloaded.summary(), trace.summary());
    }
    let unknown_field = read_fixture("harness-trace/invalid-unknown-field.json");
    assert!(serde_json::from_str::<HarnessTrace>(&unknown_field).is_err());

    // The bound: a history longer than the span limit is rejected. The
    // stream stays legal: continuing → execute → persist → continue.
    let observer = Arc::new(FakeHarnessObserver::new());
    let mut harness = test_ok(TaskHarness::prepare(
        TASK_ID,
        actor(),
        model_id(),
        None,
        Box::new(FakeCodexRuntime::new()),
        Arc::new(FakeContextCompiler::new()),
        observer.clone(),
        ts("2026-09-21T13:45:00Z"),
    ));
    test_ok(harness.execute_turn(
        &test_ok(RuntimeRequest::new(model_id(), None, vec![], "Turn")),
        ts("2026-09-21T13:46:00Z"),
    ));
    test_ok(harness.persist(Vec::new(), ts("2026-09-21T13:47:00Z")));
    test_ok(harness.continue_cycle(ts("2026-09-21T13:48:00Z")));
    let mut records = observer.records();
    let mut clock = 1_789_998_400i64;
    let mut turn = 2u64;
    while records.len() <= MAX_HARNESS_TRACE_SPANS {
        let at = test_ok(Timestamp::from_unix_seconds(clock));
        records.push(test_ok(HarnessEventRecord::new(
            "harness.executed",
            (records.len() + 1) as u64,
            TASK_ID,
            at,
            actor(),
            HarnessStateKind::Executing,
            HarnessEventDetail::Execute {
                turn,
                outcome_status: RuntimeStatus::Completed,
                missing_capabilities: Vec::new(),
            },
        )));
        records.push(test_ok(HarnessEventRecord::new(
            "harness.persisted",
            (records.len() + 1) as u64,
            TASK_ID,
            test_ok(Timestamp::from_unix_seconds(clock + 1)),
            actor(),
            HarnessStateKind::Persisting,
            HarnessEventDetail::Persist {
                persisted_refs: Vec::new(),
            },
        )));
        records.push(test_ok(HarnessEventRecord::new(
            "harness.continued",
            (records.len() + 1) as u64,
            TASK_ID,
            test_ok(Timestamp::from_unix_seconds(clock + 2)),
            actor(),
            HarnessStateKind::Continuing,
            HarnessEventDetail::Continue,
        )));
        clock += 3;
        turn += 1;
    }
    assert!(HarnessTrace::project(TASK_ID, &records).is_err());
    let bounded: Vec<HarnessEventRecord> = records.into_iter().take(4).collect();
    let bounded_trace = test_ok(HarnessTrace::project(TASK_ID, &bounded));
    test_ok(bounded_trace.validate());
}

/// The view-model fixture round-trips (the ORCH-004 seam's committed
/// shape).
#[test]
fn harness_view_fixture_round_trips() {
    let content = read_fixture("harness-view/typical.json");
    let view = test_ok(serde_json::from_str::<flauz_exec::HarnessView>(&content));
    test_ok(view.validate());
    assert_eq!(
        test_ok(serde_json::to_string(&view)),
        content,
        "the view fixture must round-trip byte-for-byte"
    );
}
