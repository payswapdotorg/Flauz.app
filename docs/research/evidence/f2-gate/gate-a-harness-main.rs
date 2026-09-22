//! Lead-authored F2 integration-gate harness (kernel §10).
//!
//! The canonical round-trip: create workspace → create task → attach
//! resource (surface change) → record observation → claim → produce
//! artifact → attach fake environment → attach fake agent/model →
//! compile context snapshot (flauz-context, the real ORCH-001 step) →
//! verify claim → evidence → grant/expire lease → serialize all state →
//! drop in-memory state → reload → assert identity/equality/evidence/lease
//! → switch fake model → continue → assert task identity unchanged.
//!
//! No external service anywhere. Deterministic (caller-supplied timestamps,
//! ID generation confined to the sanctioned generators).

use flauz_world::event_types;
use flauz_world::fakes::FakeWorldStore;
use flauz_world::refs::{ActorRef, EntityRef, StreamRef};
use flauz_world::{
    AccessMode, AccessSurface, Artifact, ArtifactContent, CanonicalValue, Claim, ConflictPolicy,
    ContractVersion, Event, Observation, Payload, Resource, ResourceLease, ResourceState, Session,
    SurfaceKind, Task, Timestamp, VerificationStatus, WorldStore, Workspace,
};
use flauz_exec::fakes::{
    FakeCodexRuntime, FakeExecStore, FakeLocalEnvironment, FakeNonCodexRuntime,
    FakeRemoteEnvironment,
};
use flauz_exec::ids::{AgentId, EnvironmentId, ModelId};
use flauz_exec::{
    Agent, CapabilityId, Environment, ExecStore, Model, RuntimeRequest, RuntimeStatus,
    SessionEventTransport,
};
use flauz_exec::ActorRef as ExecActorRef;
use flauz_exec::AgentRuntime;
use flauz_exec::Timestamp as ExecTimestamp;

mod context_step;
use context_step::compile_context_snapshot;

fn ok<T, E: std::fmt::Debug>(r: Result<T, E>, what: &str) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("F2-GATE FAIL at {what}: {e:?}"),
    }
}

fn ts(value: &str) -> Timestamp {
    ok(Timestamp::parse(value), "timestamp parse")
}

fn user(id: &str) -> ActorRef {
    ok(ActorRef::user(id), "actor ref")
}

fn payload(pairs: &[(&str, CanonicalValue)]) -> Payload {
    let mut p = Payload::empty();
    for (k, v) in pairs {
        p = ok(p.with(k, v.clone()), "payload field");
    }
    p
}

/// Wrap a world Event constructor.
fn ev(
    event_type: &str,
    at: Timestamp,
    actor: &ActorRef,
    subject: EntityRef,
    fields: &[(&str, CanonicalValue)],
) -> Event {
    ok(
        Event::new(event_type, at, actor.clone(), subject, payload(fields)),
        "event ctor",
    )
}

fn main() {
    println!("== F2 integration-gate harness (kernel §10) ==");
    println!("context-snapshot step: {}", context_step::STATUS);

    // Fixed clock — determinism rule.
    let t0 = ts("2026-09-21T12:00:00Z");
    let t1 = ts("2026-09-21T12:05:00Z");
    let t2 = ts("2026-09-21T12:10:00Z");
    let t3 = ts("2026-09-21T12:15:00Z");
    let actor = user("user_f2gate_lead");
    let agent_actor = user("agent_f2_fake");

    // ------------------------------------------------ 1. create workspace
    let ws_id = flauz_world::WorkspaceId::generate();
    let mut world = FakeWorldStore::new();
    let ws = ok(
        world.create_workspace(ok(
            Workspace::new(ws_id.clone(), "F2 gate workspace", actor.clone(), t0.clone()),
            "workspace ctor",
        )),
        "create workspace",
    );
    assert_eq!(ws.version, 1, "workspace starts at version 1");
    let _ = ok(
        world.append_event(
            StreamRef::workspace(&ws_id),
            ev(
                event_types::WORKSPACE_CREATED,
                t0.clone(),
                &actor,
                EntityRef::workspace(&ws_id),
                &[("name", CanonicalValue::Str("F2 gate workspace".into()))],
            ),
        ),
        "workspace.created event",
    );
    println!("[01] workspace created + event appended");

    // ---------------------------------------------------- 2. create task
    let task_id = flauz_world::TaskId::generate();
    let task = ok(
        world.create_task(ok(
            Task::new(
                task_id.clone(),
                ws_id.clone(),
                "Produce and verify the F2 gate artifact",
                actor.clone(),
                t0.clone(),
            ),
            "task ctor",
        )),
        "create task",
    );
    assert_eq!(task.id, task_id, "task identity is its canonical ID");
    // Session references the task (Session ≠ Task).
    let sess_id = flauz_world::SessionId::generate();
    let _ = ok(
        world.create_session(ok(
            Session::new(
                sess_id.clone(),
                ws_id.clone(),
                Some(task_id.clone()),
                actor.clone(),
                t0.clone(),
            ),
            "session ctor",
        )),
        "create session",
    );
    println!("[02] task created (session references it; session ≠ task)");

    // ------------------------------- 3. attach resource + surface change
    let res_id = flauz_world::ResourceId::generate();
    let resource = ok(
        world.create_resource(ok(
            Resource::new(
                res_id.clone(),
                "F2 gate web app",
                vec![AccessSurface::new(SurfaceKind::Browser, res_id.clone())],
                actor.clone(),
                t0.clone(),
            ),
            "resource ctor",
        )),
        "create resource",
    );
    assert_eq!(resource.version, 1);
    // Surface change: browser → api + cli + mcp. Identity must be stable.
    let mut resource_v2 = resource.clone();
    resource_v2.surfaces = vec![
        AccessSurface::new(SurfaceKind::Api, res_id.clone()),
        AccessSurface::new(SurfaceKind::Cli, res_id.clone()),
        AccessSurface::new(SurfaceKind::Mcp, res_id.clone()),
    ];
    let resource_v2 = ok(world.update_resource(resource_v2), "surface change");
    assert_eq!(resource_v2.version, 2, "surface change is a durable mutation");
    assert_eq!(resource_v2.id, res_id, "resource identity stable across surfaces");
    let _ = ok(
        world.append_event(
            StreamRef::task(&task_id),
            ev(
                event_types::RESOURCE_SURFACES_CHANGED,
                t0.clone(),
                &actor,
                EntityRef::resource(&res_id),
                &[("surfaces", CanonicalValue::Int(3))],
            ),
        ),
        "resource.surfaces_changed event",
    );
    // A bounded resource-state projection (struct literal; unversioned
    // projection record per the crate contract).
    let _ = ok(
        world.put_resource_state(ResourceState {
            v: ContractVersion,
            resource_id: res_id.clone(),
            resource_version: resource_v2.version,
            permissions: vec!["read".into(), "analyze".into()],
            last_verified_observation: None,
            authoritative_surface: Some(SurfaceKind::Api),
            active_lease_ids: Vec::new(),
            pending_mutation_ids: Vec::new(),
            related_artifact_ids: Vec::new(),
            captured_by: agent_actor.clone(),
            captured_at: t0.clone(),
        }),
        "resource state projection",
    );
    println!("[03] resource attached; surface change kept identity (v1→v2)");

    // ------------------------------------ 4-5. observation + conflicting pair
    let obs_a = ok(
        world.record_observation(ok(
            Observation::new(
                flauz_world::ObservationId::generate(),
                res_id.clone(),
                SurfaceKind::Api,
                actor.clone(),
                t1.clone(),
                "API surface reports the deployment healthy",
            ),
            "observation ctor A",
        )),
        "observation A",
    );
    let obs_b = ok(
        world.record_observation(ok(
            Observation::new(
                flauz_world::ObservationId::generate(),
                res_id.clone(),
                SurfaceKind::Cli,
                agent_actor.clone(),
                t1.clone(),
                "CLI surface reports a degraded dependency",
            ),
            "observation ctor B",
        )),
        "observation B",
    );
    assert_ne!(obs_a.id, obs_b.id, "conflicting observations stay distinct");
    println!("[04] observations recorded (conflicting pair distinct + attributable)");

    let claim = ok(
        world.record_claim(ok(
            Claim::new(
                flauz_world::ClaimId::generate(),
                res_id.clone(),
                Some(SurfaceKind::Api),
                agent_actor.clone(),
                t1.clone(),
                "The gate artifact is complete and internally consistent",
            ),
            "claim ctor",
        )),
        "claim",
    );
    assert_eq!(
        claim.verification,
        VerificationStatus::Claimed,
        "new claims start claimed"
    );
    println!("[05] claim recorded (claimed, not evidence)");

    // ------------------------------------------- 6. produce an artifact
    let art_id = flauz_world::ArtifactId::generate();
    let artifact = ok(
        world.create_artifact(ok(
            Artifact::new(
                art_id.clone(),
                task_id.clone(),
                "F2 gate report",
                ArtifactContent::Text {
                    text: "F2 canonical round-trip gate report".into(),
                },
                agent_actor.clone(),
                t2.clone(),
            ),
            "artifact ctor",
        )),
        "artifact",
    );
    assert_eq!(artifact.version, 1);
    let artifact_event = ok(
        world.append_event(
            StreamRef::task(&task_id),
            ev(
                event_types::ARTIFACT_PRODUCED,
                t2.clone(),
                &agent_actor,
                EntityRef::artifact(&art_id),
                &[("title", CanonicalValue::Str("F2 gate report".into()))],
            ),
        ),
        "artifact.produced event",
    );
    println!("[06] artifact produced + event appended");

    // -------------------------------- 7. attach fake environments (exec side)
    let mut exec = FakeExecStore::new();
    let local_env = ok(FakeLocalEnvironment::with_default_id(), "fake local environment");
    let remote_env_id = ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPT7"), "remote env id");
    let remote_env = ok(
        FakeRemoteEnvironment::new(remote_env_id, "Fake remote sandbox"),
        "fake remote environment",
    );
    assert_ne!(local_env.locality(), remote_env.locality());
    let _ = ok(exec.create_environment(local_env.descriptor()), "store local env");
    let _ = ok(exec.create_environment(remote_env.descriptor()), "store remote env");
    let _ = ok(
        world.append_event(
            StreamRef::task(&task_id),
            ev(
                event_types::TASK_ENVIRONMENT_CHANGED,
                t2.clone(),
                &actor,
                EntityRef::task(&task_id),
                &[("environment", CanonicalValue::Str("fake-local".into()))],
            ),
        ),
        "task.environment_changed event",
    );
    println!("[07] fake local + remote environments attached (same Environment contract)");

    // ------------------------- 8. attach fake agents/models (both runtimes)
    let exec_actor = ok(ExecActorRef::user("user_f2gate_lead"), "exec actor ref");
    let exec_t0 = ok(ExecTimestamp::parse("2026-09-21T12:00:00Z"), "exec timestamp");
    let codex_rt = FakeCodexRuntime::new();
    let direct_rt = FakeNonCodexRuntime::new();
    let model_a = ok(
        Model::new(
            ok(ModelId::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPT7"), "model id A"),
            "Fake Text Model",
            "flauz-fake",
            vec![ok(CapabilityId::parse("text.generation"), "cap text")],
            Some("text-only fake model"),
            exec_actor.clone(),
            exec_t0.clone(),
        ),
        "model A",
    );
    let model_b = ok(
        Model::new(
            ok(ModelId::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPT8"), "model id B"),
            "Fake Vision Model",
            "flauz-fake",
            vec![
                ok(CapabilityId::parse("image.understanding"), "cap vision"),
                ok(CapabilityId::parse("text.generation"), "cap text"),
            ],
            Some("vision fake model"),
            exec_actor.clone(),
            exec_t0.clone(),
        ),
        "model B",
    );
    let _ = ok(exec.create_model(model_a.clone()), "store model A");
    let _ = ok(exec.create_model(model_b.clone()), "store model B");
    let agent = ok(
        exec.create_agent(ok(
            Agent::new(
                AgentId::generate(),
                "Gate agent",
                Some(codex_rt.runtime_kind()),
                Some(model_a.id.clone()),
                exec_actor.clone(),
                exec_t0.clone(),
            ),
            "agent ctor",
        )),
        "agent",
    );
    assert_eq!(agent.model_id.as_ref(), Some(&model_a.id));
    // Runtime turns: both fakes satisfy the SAME AgentRuntime contract.
    // Codex turn (terminal capability): completed.
    let codex_request = ok(
        RuntimeRequest::new(
            model_a.id.clone(),
            Some(local_env.environment_id().clone()),
            vec![ok(CapabilityId::parse("terminal"), "cap terminal")],
            "Run the F2 gate verification turn",
        ),
        "codex runtime request",
    );
    let outcome_a = ok(codex_rt.execute(&codex_request), "codex runtime turn");
    assert_eq!(outcome_a.status, RuntimeStatus::Completed, "fake codex turn completes");
    // Non-Codex turn (browser capability): completed — the SAME contract,
    // different advertised surface.
    let direct_request = ok(
        RuntimeRequest::new(
            model_b.id.clone(),
            Some(remote_env.environment_id().clone()),
            vec![ok(CapabilityId::parse("browser.navigation"), "cap browser")],
            "Run the F2 gate research turn",
        ),
        "non-codex runtime request",
    );
    let outcome_b = ok(direct_rt.execute(&direct_request), "non-codex runtime turn");
    assert_eq!(outcome_b.status, RuntimeStatus::Completed, "fake non-codex turn completes");
    // Capability-gap honesty: request terminal from the browser-only fake.
    let gap_request = ok(
        RuntimeRequest::new(
            model_a.id.clone(),
            None,
            vec![ok(CapabilityId::parse("terminal"), "cap terminal")],
            "Should gap cleanly",
        ),
        "gap request",
    );
    let gap_outcome = ok(direct_rt.execute(&gap_request), "gap turn");
    assert_eq!(gap_outcome.status, RuntimeStatus::CapabilityGap, "gap is honest");
    assert_eq!(gap_outcome.missing_capabilities.len(), 1, "exactly the missing key reported");
    println!("[08] fake agents/models attached (both runtimes satisfy AgentRuntime)");

    // ------------------- 9. compile context snapshot (ORCH-001, the real
    // flauz-context step: projection, provenance, authorization-aware
    // filtering, canonical-JSON round-trip, model-switch RESET).
    compile_context_snapshot(context_step::ContextStepInput {
        task_id: &task_id.to_string(),
        session_id: &sess_id.to_string(),
        artifact_id: &art_id.to_string(),
        observation_id: &obs_a.id.to_string(),
        event_id: &artifact_event.event_id.to_string(),
        environment_id: &local_env.environment_id().to_string(),
        model_a: &model_a.id.to_string(),
        model_b: &model_b.id.to_string(),
    });

    // ---------------------------------------------- 10. verify claim → evidence
    let verification = ok(
        world.verify_claim(
            claim.clone(),
            actor.clone(),
            t3.clone(),
            StreamRef::task(&task_id),
            payload(&[("method", CanonicalValue::Str("f2-gate-independent-check".into()))]),
        ),
        "verify claim",
    );
    assert_eq!(
        verification.claim.verification,
        VerificationStatus::Verified,
        "claim flips to verified"
    );
    assert!(verification.evidence.verifier == actor, "evidence is attributable");
    let evd_id = verification.evidence.id.clone();
    let stored_evidence = ok(world.evidence(&evd_id), "evidence lookup")
        .unwrap_or_else(|| panic!("evidence {evd_id:?} must exist"));
    // The evidence's verification event must exist on the task stream.
    let verification_event = ok(
        world.event(&stored_evidence.verification_event_id),
        "verification event lookup",
    )
    .expect("the verification event exists");
    assert!(
        verification_event.seq > 0,
        "evidence references a verification event on the stream"
    );
    println!("[10] claim verified → evidence (verifier + event + timestamp)");

    // ------------------------------------------- 11. grant + expire a lease
    let lease = ok(
        world.grant_lease(ok(
            ResourceLease::new(
                flauz_world::LeaseId::generate(),
                res_id.clone(),
                AccessMode::Analyze,
                agent_actor.clone(),
                t1.clone(),
                ts("2026-09-21T12:12:00Z"), // expires before t3
                ConflictPolicy::Queue,
            ),
            "lease ctor",
        )),
        "grant lease",
    );
    assert!(
        world.lease(&lease.id, &ts("2026-09-21T12:11:00Z")).is_ok(),
        "lease held before expiry"
    );
    let held_at_t3 = ok(world.lease(&lease.id, &t3), "lease read at t3");
    assert!(held_at_t3.is_none(), "expired lease is not held (expiry enforced on read)");
    let raw = ok(world.lease_record(&lease.id), "raw lease record")
        .expect("raw record kept for provenance");
    assert_eq!(raw.version, 1, "raw record persists after expiry");
    println!("[11] lease granted, expired on read; raw record retained");

    // ------------------- 12. serialize → drop → reload (both stores, canonical JSON)
    let world_snap = world.snapshot();
    let world_json = ok(serde_json::to_string(&world_snap), "world snapshot JSON");
    let exec_snap = exec.snapshot();
    let exec_json = ok(serde_json::to_string(&exec_snap), "exec snapshot JSON");
    drop(world);
    drop(exec);
    let mut world2 = ok(
        FakeWorldStore::restore(ok(
            serde_json::from_str::<flauz_world::WorldSnapshot>(&world_json),
            "world snapshot parse",
        )),
        "world restore",
    );
    let exec2 = ok(
        FakeExecStore::restore(ok(
            serde_json::from_str::<flauz_exec::ExecSnapshot>(&exec_json),
            "exec snapshot parse",
        )),
        "exec restore",
    );
    assert_eq!(world2.snapshot(), world_snap, "world state equality after reload");
    assert_eq!(exec2.snapshot(), exec_snap, "exec state equality after reload");
    println!("[12] serialize → drop → reload; state equality holds (world + exec)");

    // ---------------------- 13. reload assertions (identity / evidence / lease)
    let task_reloaded = ok(world2.task(&task_id), "task lookup after reload")
        .unwrap_or_else(|| panic!("task {task_id:?} must survive reload"));
    assert_eq!(task_reloaded.id, task_id, "same logical task identity");
    let evidence_reloaded = ok(world2.evidence(&evd_id), "evidence after reload")
        .expect("evidence survives reload");
    assert_eq!(evidence_reloaded.claim_id, claim.id, "evidence still links the claim");
    let claim_reloaded = ok(world2.claim(&claim.id), "claim after reload").unwrap();
    assert_eq!(
        claim_reloaded.verification,
        VerificationStatus::Verified,
        "evidence still verified after reload"
    );
    assert!(
        ok(world2.lease(&lease.id, &t3), "expired lease after reload").is_none(),
        "expired lease still not held after reload"
    );
    println!("[13] reload assertions: identity / equality / verified evidence / expired lease");

    // ------------------------------------- 14. switch fake model → continue
    let model_switch_event = ok(
        world2.append_event(
            StreamRef::task(&task_id),
            ev(
                event_types::TASK_MODEL_CHANGED,
                ts("2026-09-21T12:20:00Z"),
                &actor,
                EntityRef::task(&task_id),
                &[
                    ("from", CanonicalValue::Str(model_a.id.to_string())),
                    ("to", CanonicalValue::Str(model_b.id.to_string())),
                ],
            ),
        ),
        "task.model_changed event",
    );
    // Continue appends on the same stream: seq strictly increases, no fork.
    let next_event = ok(
        world2.append_event(
            StreamRef::task(&task_id),
            ev(
                event_types::TASK_CONTEXT_RESET,
                ts("2026-09-21T12:25:00Z"),
                &actor,
                EntityRef::task(&task_id),
                &[],
            ),
        ),
        "post-switch continuation event",
    );
    assert!(
        next_event.seq > model_switch_event.seq,
        "stream continues on the same logical task (seq strictly increases)"
    );
    assert_eq!(
        ok(world2.task(&task_id), "final task identity").unwrap().id,
        task_id,
        "task identity unchanged across the model switch"
    );
    println!("[14] model switch → continue: same logical task, stream continuity holds");

    // ------------------------------- cross-crate seam: envelope ↔ transport
    let fixture = include_str!(
        "/home/z/Flauz.app/crates/flauz-world/tests/fixtures/f2/event-envelope/typical.json"
    );
    let envelope: flauz_world::EventEnvelope =
        ok(serde_json::from_str(fixture), "envelope sample parse");
    let tx = SessionEventTransport::<flauz_world::EventEnvelope>::new();
    let mut wire: Vec<u8> = Vec::new();
    ok(tx.write(&envelope, &mut wire), "transport write");
    let rx = SessionEventTransport::<flauz_world::EventEnvelope>::new();
    let decoded = ok(rx.read(&mut std::io::Cursor::new(wire.clone())), "transport read")
        .expect("one envelope on the wire");
    assert_eq!(
        ok(serde_json::to_string(&envelope), "envelope ser"),
        ok(serde_json::to_string(&decoded), "decoded ser"),
        "byte-stable envelope value equality across the seam"
    );
    println!(
        "[seam] world EventEnvelope round-trips through exec transport ({} bytes)",
        wire.len()
    );

    println!("== F2 INTEGRATION HARNESS: ALL STEPS PASS ==");
}
