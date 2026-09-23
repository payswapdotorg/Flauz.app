//! Wave-4 (F7/F8/F9) gate step: the three-fabric scenario — ONE task,
//! three fabrics, every Wave-4 law.
//!
//! **PROV (F7)** — the task's model need routes through the flauz-prov
//! scheduler: two fake accounts connected through the secret-store seam
//! (a 5-use free tier + a 100-use paid tier), free-tier-first; the free
//! tier depletes MID-RUN (turn 2 of 3) and turn 3 escalates — NAMED
//! (`PolicyRule::EscalatedToPaid` + the free account in `skipped` with
//! `SkipReason::Depleted`, never a silent fall-through); every choice is
//! recorded on the task's world event stream (`task.scheduled_on_account`
//! events carrying the choice's canonical JSON) and every one of them
//! carries full attribution (whose account, which provider, which tier,
//! which policy rule) — the attribution law.
//!
//! **LAB (F8)** — the SAME journey spec (the a11y chord-ladder — the
//! d25/N5/N6/d19 shapes) runs unchanged against the LocalLabAdapter and
//! the FakeRemoteLabAdapter through `run_journey`; the comparator reports
//! ZERO divergences on the identical runs, and exactly ONE named, severe
//! (major), located divergence on a seeded one (a flipped frame anchor).
//!
//! **COL (F9)** — the second actor is present: the canonical two-actor
//! interleaving (12 steps) through `run_interleaved` +
//! `verify_expectations` + `verify_f9_laws`; a member-private memory
//! item never appears in the other actor's projection; a permission
//! denial is NAMED (`role_denied` / `version_conflict` /
//! `private_record` all present); the shared-filesystem mode is STATED;
//! and the interleaving log round-trips through canonical JSON
//! byte-identically (replayability).
//!
//! Lead-authored gate infrastructure (the F2 §10 harness pattern), NOT a
//! workspace crate. No external service anywhere; every timestamp is
//! caller-supplied (kernel §7); no credential material anywhere (the
//! connect flow uses the fake practice keys the fake seam mints into
//! `flausec_` references).

use flauz_collab::simulator::{
    run_interleaved, verify_expectations, verify_f9_laws, SimulationOutcome,
};
use flauz_collab::{SharedFilesystemMode, Visibility};
use flauz_lab::comparator::{compare, DivergenceDetail, Severity};
use flauz_lab::{FakeRemoteLabAdapter, LocalLabAdapter, ProbeResult, run_journey};
use flauz_prov::fakes::{FakeRoutingScenario, FAKE_TASK};
use flauz_prov::scheduler::{PolicyRule, SkipReason};
use flauz_prov::{ConnectionRef, SchedulingChoice, TierKind};
use flauz_world::fakes::FakeWorldStore;
use flauz_world::refs::{ActorRef, EntityRef, StreamRef};
use flauz_world::{
    CanonicalValue, Event, EventEnvelope, Payload, Task, TaskId, Timestamp, Workspace,
    WorkspaceId, WorldStore,
};

fn ok<T, E: std::fmt::Debug>(r: Result<T, E>, what: &str) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("W4-GATE FAIL at {what}: {e:?}"),
    }
}

fn ts(value: &str) -> Timestamp {
    ok(Timestamp::parse(value), "timestamp parse")
}

/// flauz-prov's own frozen timestamp type (distinct from the world's —
/// the crate takes caller-supplied moments as its own data).
fn ts_prov(value: &str) -> flauz_prov::Timestamp {
    ok(flauz_prov::Timestamp::parse(value), "prov timestamp parse")
}

fn user(id: &str) -> ActorRef {
    ok(ActorRef::user(id), "actor ref")
}

/// Records one scheduling choice on the task's world event stream — the
/// quota-attribution law's durable half: the choice lands on the SAME
/// stream the task's history lives on (new event types through the
/// world-store seam, the frozen-format discipline; the type name follows
/// the frozen dotted grammar).
fn record_choice(
    world: &mut FakeWorldStore,
    task_id: &TaskId,
    actor: &ActorRef,
    choice: &SchedulingChoice,
    at: Timestamp,
) -> EventEnvelope {
    let choice_json = ok(serde_json::to_string(choice), "choice canonical JSON");
    let mut payload = Payload::empty();
    for (key, value) in [
        ("connection_id", CanonicalValue::Str(choice.connection_id.to_string())),
        ("account_label", CanonicalValue::Str(choice.account_label.clone())),
        ("provider_kind", CanonicalValue::Str(choice.provider_kind.clone())),
        (
            "tier",
            CanonicalValue::Str(
                match choice.tier {
                    TierKind::Free => "free",
                    TierKind::Paid => "paid",
                }
                .to_owned(),
            ),
        ),
        (
            "policy_rule",
            CanonicalValue::Str(
                serde_json::to_string(&choice.policy_rule)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned(),
            ),
        ),
        ("escalated", CanonicalValue::Bool(choice.escalated)),
        ("choice", CanonicalValue::Str(choice_json)),
    ] {
        payload = ok(payload.with(key, value), "scheduled_on_account payload field");
    }
    let event = ok(
        Event::new(
            "task.scheduled_on_account",
            at,
            actor.clone(),
            EntityRef::task(task_id),
            payload,
        ),
        "scheduled_on_account event ctor",
    );
    ok(
        world.append_event(StreamRef::task(task_id), event),
        "append scheduled_on_account",
    )
}

pub fn w4_three_fabric_step() {
    // Fixed clock — the determinism rule (caller-supplied everywhere).
    let t0 = ts("2026-09-23T11:00:00Z");
    let t1 = ts("2026-09-23T11:05:00Z");
    let t2 = ts("2026-09-23T11:10:00Z");
    let t3 = ts("2026-09-23T11:15:00Z");
    // The same moments on flauz-prov's own clock type.
    let p0 = ts_prov("2026-09-23T11:00:00Z");
    let p1 = ts_prov("2026-09-23T11:05:00Z");
    let p2 = ts_prov("2026-09-23T11:10:00Z");
    let p3 = ts_prov("2026-09-23T11:15:00Z");

    let actor = user("user_w4gate_lead");

    // ---------------------------------------------------------------
    // [19a] The world: workspace + the task whose model need routes.
    // The task's identity IS the routing scenario's task (the fake
    // scenario's fixed task ref — one task, three fabrics).
    // ---------------------------------------------------------------
    let ws_id = WorkspaceId::generate();
    let mut world = FakeWorldStore::new();
    let _ = ok(
        world.create_workspace(ok(
            Workspace::new(
                ws_id.clone(),
                "Wave-4 three-fabric workspace",
                actor.clone(),
                t0.clone(),
            ),
            "workspace ctor",
        )),
        "create workspace",
    );
    let task_id = ok(TaskId::parse(FAKE_TASK), "parse the scenario task ref");
    let _ = ok(
        world.create_task(ok(
            Task::new(
                task_id.clone(),
                ws_id.clone(),
                "Draft the launch post, then have it reviewed",
                actor.clone(),
                t0.clone(),
            ),
            "task ctor",
        )),
        "create three-fabric task",
    );

    // ---------------------------------------------------------------
    // [19b] PROV (F7): connect two accounts through the secret-store
    // seam, route free-tier-first, deplete mid-run, escalate NAMED.
    // ---------------------------------------------------------------
    let mut scenario = ok(
        FakeRoutingScenario::new("openai", p0.clone()),
        "fake routing scenario",
    );
    // The connect flow on fakes: PRACTICE material (never a real
    // credential) through the seam — which mints references and retains
    // nothing.
    let free_account = ok(
        scenario.connect_account(TierKind::Free, "practice-key-free", "Personal account"),
        "connect free account",
    );
    let paid_account = ok(
        scenario.connect_account(TierKind::Paid, "practice-key-paid", "Work account"),
        "connect paid account",
    );
    assert_eq!(free_account.tier, TierKind::Free, "the free account is free-tier");
    assert_eq!(paid_account.tier, TierKind::Paid, "the paid account is paid-tier");
    // The no-material law: the accounts carry flausec_ references, never
    // the practice material.
    assert!(
        free_account.secret_ref.as_str().starts_with("flausec_")
            && paid_account.secret_ref.as_str().starts_with("flausec_"),
        "accounts carry secret-store references, never material"
    );

    // Turn 1: free-tier-first picks the free account.
    let choice1 = ok(scenario.schedule(p1.clone()), "schedule turn 1");
    ok(choice1.validate(), "choice 1 validates (the attribution law)");
    assert_eq!(choice1.tier, TierKind::Free, "turn 1 routes the free tier");
    assert_eq!(choice1.policy_rule, PolicyRule::DefaultOrder, "turn 1 is the default free-tier-first order");
    assert!(!choice1.escalated, "turn 1 is not an escalation");
    let event1 = record_choice(&mut world, &task_id, &actor, &choice1, t1.clone());
    ok(
        scenario.consume(&choice1, "launch-post draft turn", 2, p1.clone()),
        "consume turn 1 (2 of 5 free uses)",
    );

    // Turn 2: still free (3 of 5 left) — and this consumption depletes
    // the free tier MID-RUN.
    let choice2 = ok(scenario.schedule(p2.clone()), "schedule turn 2");
    ok(choice2.validate(), "choice 2 validates");
    assert_eq!(choice2.tier, TierKind::Free, "turn 2 still routes the free tier");
    assert_eq!(
        choice2.quota_at_choice.remaining, 3,
        "turn 2 sees the honest remaining free quota (3 of 5)"
    );
    let event2 = record_choice(&mut world, &task_id, &actor, &choice2, t2.clone());
    ok(
        scenario.consume(&choice2, "launch-post draft turn", 3, p2.clone()),
        "consume turn 2 (the remaining 3 — the free tier is now used up)",
    );

    // Turn 3: the free tier is DEPLETED — the escalation must be NAMED:
    // the paid account chosen, the rule EscalatedToPaid, and the free
    // account left behind in `skipped` with SkipReason::Depleted (the
    // CAP-001 law applied to money — never a silent fall-through).
    let choice3 = ok(scenario.schedule(p3.clone()), "schedule turn 3 (the depletion moment)");
    ok(choice3.validate(), "choice 3 validates (the escalation law)");
    assert_eq!(choice3.tier, TierKind::Paid, "turn 3 escalates to the paid tier");
    assert!(choice3.escalated, "the escalation flag is set");
    assert_eq!(
        choice3.policy_rule,
        PolicyRule::EscalatedToPaid,
        "the escalation is NAMED by its policy rule"
    );
    let free_skip = choice3
        .skipped
        .iter()
        .find(|skipped| skipped.tier == TierKind::Free)
        .expect("the escalation names the free alternative it left behind");
    assert_eq!(
        free_skip.reason,
        SkipReason::Depleted,
        "the free account is skipped with the NAMED depletion reason"
    );
    let _event3 = record_choice(&mut world, &task_id, &actor, &choice3, t3.clone());
    ok(
        scenario.consume(&choice3, "launch-post review turn", 1, p3.clone()),
        "consume turn 3 through the paid account",
    );

    // The stream carries every choice, in order, fully attributed.
    let envelopes = ok(
        world.events(&StreamRef::task(&task_id), 0, usize::MAX),
        "read the task stream",
    );
    let scheduling_events: Vec<&EventEnvelope> = envelopes
        .iter()
        .filter(|envelope| envelope.event_type.as_str() == "task.scheduled_on_account")
        .collect();
    assert_eq!(
        scheduling_events.len(),
        3,
        "all three scheduling choices are recorded on the task's stream"
    );
    assert_eq!(scheduling_events[0].seq, event1.seq, "turn 1's event seq");
    assert_eq!(scheduling_events[1].seq, event2.seq, "turn 2's event seq");
    for (index, stream_event) in scheduling_events.iter().enumerate() {
        let tier = stream_event
            .payload
            .get("tier")
            .and_then(|v| v.as_str().map(|s| s.to_owned()))
            .unwrap_or_default();
        let label = stream_event
            .payload
            .get("account_label")
            .and_then(|v| v.as_str().map(|s| s.to_owned()))
            .unwrap_or_default();
        let rule = stream_event
            .payload
            .get("policy_rule")
            .and_then(|v| v.as_str().map(|s| s.to_owned()))
            .unwrap_or_default();
        assert!(!label.is_empty(), "choice {} names whose account", index + 1);
        assert!(!rule.is_empty(), "choice {} names its policy rule", index + 1);
        let choice_json = stream_event
            .payload
            .get("choice")
            .and_then(|v| v.as_str().map(|s| s.to_owned()))
            .unwrap_or_default();
        let round_trip: SchedulingChoice =
            ok(serde_json::from_str(&choice_json), "choice JSON round-trip");
        ok(round_trip.validate(), "round-tripped choice validates");
        let round_trip_tier = match round_trip.tier {
            TierKind::Free => "free",
            TierKind::Paid => "paid",
        };
        assert_eq!(
            round_trip_tier, tier,
            "the stream's tier field agrees with the canonical choice"
        );
        // Every escalated choice on the stream names its free alternative.
        if round_trip.escalated {
            assert!(
                round_trip
                    .skipped
                    .iter()
                    .any(|s| s.tier == TierKind::Free),
                "the escalated stream record names the free alternative"
            );
        }
    }
    let escalated_on_stream = scheduling_events
        .iter()
        .any(|e| e.payload.get("escalated").and_then(|v| v.as_bool()) == Some(true));
    assert!(
        escalated_on_stream,
        "the named escalation is visible on the task's event stream"
    );
    // The attribution ledger: usage recorded per account, attributed to
    // THIS task.
    let ledger = scenario.ledger();
    let free_conn = ok(ConnectionRef::parse(flauz_prov::fakes::FAKE_FREE_CONNECTION), "free conn");
    let paid_conn = ok(ConnectionRef::parse(flauz_prov::fakes::FAKE_PAID_CONNECTION), "paid conn");
    assert_eq!(
        ledger.records_for(&free_conn).iter().map(|r| r.units).sum::<u64>(),
        5,
        "the free account's 5 uses are attributed on the ledger"
    );
    assert_eq!(
        ledger.records_for(&paid_conn).iter().map(|r| r.units).sum::<u64>(),
        1,
        "the escalated turn's use is attributed to the paid account"
    );
    println!(
        "[19b] PROV fabric: free-tier-first routing with the mid-run depletion → the NAMED \
         escalation (EscalatedToPaid + the free account skipped Depleted); 3 choices on the \
         task's event stream, every one fully attributed; ledger attribution 5 free + 1 paid"
    );

    // ---------------------------------------------------------------
    // [19c] LAB (F8): the same journey spec against local + fake-remote
    // adapters; the comparator — zero on identical, named on seeded.
    // ---------------------------------------------------------------
    let journey = flauz_lab::fakes::fake_a11y_chord_ladder_journey();
    ok(journey.validate(), "journey spec validates");
    // The same journey BYTES: serialize once, parse back, run THAT on
    // both adapters (the F8 gate law's literal reading).
    let journey_bytes = ok(serde_json::to_string(&journey), "journey canonical JSON");
    let journey_run: flauz_lab::JourneySpec =
        ok(serde_json::from_str(&journey_bytes), "journey JSON round-trip");

    let mut local_adapter = LocalLabAdapter::new();
    let reference = ok(run_journey(&mut local_adapter, &journey_run), "local lab run");
    let mut remote_adapter = FakeRemoteLabAdapter::new();
    let candidate = ok(run_journey(&mut remote_adapter, &journey_run), "fake-remote lab run");
    assert_eq!(
        reference.journey, candidate.journey,
        "both records evidence the SAME journey"
    );
    assert_ne!(
        reference.adapter.provider_kind, candidate.adapter.provider_kind,
        "the two adapters are genuinely different providers"
    );

    let identical_report = ok(
        compare(&journey_run, &reference, &candidate),
        "comparator: identical runs",
    );
    ok(identical_report.validate(), "identical report validates");
    assert_eq!(
        identical_report.divergences.len(),
        0,
        "identical runs report ZERO divergences"
    );
    assert_eq!(
        identical_report.divergent, 0,
        "no divergent steps on identical runs"
    );
    assert_eq!(identical_report.missing, 0, "no missing steps on identical runs");
    assert_eq!(
        identical_report.matching as usize,
        journey_run.steps.len(),
        "every journey step matched on both sides"
    );

    // The seeded divergence: flip one frame anchor's observed state on
    // the candidate — the comparator must name it, sever it (major), and
    // locate it (step + probe).
    let mut seeded = candidate.clone();
    'seed: for step in &mut seeded.steps {
        for probe in &mut step.probes {
            if let ProbeResult::Frame { anchors, .. } = &mut probe.result {
                if let Some(first) = anchors.first_mut() {
                    let flipped = ok(
                        flauz_lab::StateKey::parse(
                            if first.state.as_str() == "visible" { "hidden" } else { "visible" },
                        ),
                        "flipped anchor state",
                    );
                    first.state = flipped;
                    break 'seed;
                }
            }
        }
    }
    ok(seeded.validate(), "the seeded record still validates (it is well-formed — just divergent)");
    let seeded_report = ok(
        compare(&journey_run, &reference, &seeded),
        "comparator: seeded divergence",
    );
    ok(seeded_report.validate(), "seeded report validates");
    assert_eq!(
        seeded_report.divergences.len(),
        1,
        "the seeded divergence is caught — exactly one"
    );
    let divergence = &seeded_report.divergences[0];
    assert!(
        matches!(divergence.detail, DivergenceDetail::FrameAnchors { .. }),
        "the seeded divergence is NAMED (kind frame_anchors)"
    );
    assert_eq!(
        divergence.severity,
        Severity::Major,
        "the seeded divergence is severe (major)"
    );
    assert!(
        divergence.probe.is_some(),
        "the seeded divergence is located (step + probe)"
    );
    assert!(
        seeded_report.divergent >= 1,
        "the seeded step counts as divergent"
    );
    println!(
        "[19c] LAB fabric: the same journey bytes through local + fake-remote adapters → \
         comparable normalized evidence (zero divergences, {matching} steps matched); the \
         seeded anchor flip caught as exactly ONE named major divergence, located",
        matching = identical_report.matching
    );

    // ---------------------------------------------------------------
    // [19d] COL (F9): the second actor present — the two-actor
    // interleaving; privacy holds, denials named, sharing stated.
    // ---------------------------------------------------------------
    let inputs = flauz_collab::fakes::fake_simulation_inputs();
    let table = flauz_collab::fakes::fake_interleaving_table();
    let result = ok(run_interleaved(&inputs, &table), "two-actor interleaved run");
    ok(
        verify_expectations(&table, &result),
        "every scripted step landed or denied exactly as predicted",
    );
    ok(
        verify_f9_laws(&inputs, &table, &result),
        "the F9 laws hold under interleaving",
    );
    assert_eq!(
        result.log.records.len(),
        table.steps.len(),
        "one record per scripted step"
    );

    // The named denials: role_denied (Dev's surface), version_conflict
    // (the stale racing write — no lost update), private_record (even
    // the owner cannot touch another member's private notes).
    let denial_reasons: Vec<String> = result
        .log
        .records
        .iter()
        .filter_map(|record| record.outcome.denial_reason())
        .map(|reason| serde_json::to_string(&reason).unwrap_or_default().trim_matches('"').to_owned())
        .collect();
    for expected in ["role_denied", "version_conflict", "private_record"] {
        assert!(
            denial_reasons.iter().any(|reason| reason == expected),
            "a denial is NAMED ({expected}) — never a silent no-op"
        );
    }

    // The privacy law: the member-private note never appears in the
    // other actor's projection. Dev creates the private note (step 6);
    // Ana's later projection (step 7) must not contain it.
    let mut dev_private_note = String::new();
    for record in &result.log.records {
        if let SimulationOutcome::Written { key, .. } = &record.outcome {
            if key.contains("note") && record.actor.id.contains("dev") {
                dev_private_note = key.clone();
            }
        }
    }
    assert!(
        !dev_private_note.is_empty(),
        "Dev's member-private note landed (written)"
    );
    let ana_projection = result.log.records.iter().rev().find_map(|record| {
        if record.actor.id.contains("ana") {
            if let SimulationOutcome::Projection { rows } = &record.outcome {
                return Some(rows.clone());
            }
        }
        None
    });
    let ana_rows = ana_projection.expect("Ana read a projection");
    assert!(
        ana_rows.iter().all(|row| row.key != dev_private_note),
        "Dev's member-private note is NEVER in Ana's projection (the §6 privacy law)"
    );

    // The sharing posture is STATED (consequences in the record):
    // shared files mode + the visibility split.
    let sharing = result.log.records.iter().find_map(|record| {
        if let SimulationOutcome::Sharing { shared_filesystem, memory, artifacts } = &record.outcome
        {
            Some((*shared_filesystem, *memory, *artifacts))
        } else {
            None
        }
    });
    let (shared_mode, memory_visibility, _artifacts_visibility) =
        sharing.expect("the task's sharing posture was stated");
    assert_ne!(
        shared_mode,
        SharedFilesystemMode::Off,
        "the canonical story ends with the shared-filesystem mode stated"
    );
    assert_eq!(
        memory_visibility,
        Visibility::WorkspaceShared,
        "the workspace-shared memory visibility is stated"
    );

    // Replayability: the interleaving log round-trips through canonical
    // JSON byte-identically.
    let log_bytes = ok(serde_json::to_string(&result.log), "simulation log canonical JSON");
    let replayed: flauz_collab::simulator::SimulationLog =
        ok(serde_json::from_str(&log_bytes), "simulation log round-trip");
    let replay_bytes = ok(serde_json::to_string(&replayed), "replayed log canonical JSON");
    assert_eq!(
        log_bytes, replay_bytes,
        "the interleaving log replays byte-identically (canonical JSON)"
    );

    // The no-credential-material law across all three fabrics' serialized
    // state: choices, evidence and logs carry references only. The scan
    // is VALUE-PREFIX-ADAPTED (the flauz-lab conformance precedent): each
    // marker is checked at a JSON string's OPENING position — a value
    // that STARTS like a credential — because legitimate identifiers
    // (anchor ids like "task-surface") contain the bare "sk-" substring
    // without being material.
    let mut everything = String::new();
    everything.push_str(&log_bytes);
    everything.push_str(&ok(serde_json::to_string(&seeded), "seeded evidence JSON"));
    for envelope in &envelopes {
        if let Ok(text) = serde_json::to_string(envelope) {
            everything.push_str(&text);
        }
    }
    for marker in flauz_prov::CREDENTIAL_MARKERS {
        let value_prefixed = format!("\"{marker}");
        let value_prefixed_lower = format!("\"{}", marker.to_lowercase());
        assert!(
            !everything.contains(&value_prefixed) && !everything.contains(&value_prefixed_lower),
            "no credential material (a value starting with {marker:?}) anywhere in the three \
             fabrics' serialized state"
        );
    }
    for forbidden in ["\"token\"", "\"credential", "\"xoxb-", "\"ghp_"] {
        assert!(
            !everything.contains(forbidden),
            "no credential material ({forbidden:?}) anywhere in the three fabrics' serialized state"
        );
    }

    println!(
        "[19d] COL fabric: the two-actor interleaving ({steps} steps) — permissions hold under \
         interleaving, the private note never leaks into the other's projection, three NAMED \
         denial kinds (role_denied + version_conflict + private_record), the sharing posture \
         stated with its visibility split, and the log replays byte-identically",
        steps = table.steps.len()
    );
}
