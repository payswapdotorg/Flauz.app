//! Wave-3 (F6) gate step: the domain-neutral end-to-end scenario on
//! fakes — the non-software-development run the F6 gate requires.
//!
//! "Research, analyze and independently verify the three most-cited
//! renewable-energy policy shifts of 2026": one research task, a browser
//! resource, the fake browser sandbox, BOTH fake runtimes, the ORCH-004
//! attributed graph (research/analysis/review with an independent
//! verifier), the ORCH-003 harness driving the pipeline
//! (prepare/execute/observe/verify/persist/continue with recover as a
//! first-class continuation), the ORCH-002 engine rebuilding context
//! from durable state across a mid-run model switch (the rebuild law),
//! world-side attribution (artifacts + independently verified evidence),
//! Save-as-a-reusable-workflow (the contract-level Procedure), and
//! run-again as a NEW task — both tasks' identities intact.
//!
//! Lead-authored gate infrastructure (the F2 §10 harness pattern), NOT a
//! workspace crate. No external service anywhere; every timestamp is
//! caller-supplied (kernel §7).

use std::sync::{Arc, Mutex};

use flauz_context::{
    ActorRef as ContextActorRef, ArtifactRecord as ContextArtifactRecord,
    AuthorizationClass, DurableStateInputs, EvidenceRecord as ContextEvidenceRecord,
    EventRef as ContextEventRef, MemoryContent, MemoryItem, MemoryItemId, MemoryTier,
    ModelContextProfile, ModelRef as ContextModelRef, MultimodalBehavior,
    ObservationRecord as ContextObservationRecord, SessionRef as ContextSessionRef,
    TaskEventRecord, TaskRef as ContextTaskRef, Timestamp as ContextTimestamp,
    ToolSchemaHandling, compile_from_durable_state, projections_equivalent,
};
use flauz_exec::fakes::{FakeExecStore, FakeNonCodexRuntime, FakeRemoteEnvironment};
use flauz_exec::harness::{
    ContextCompileInput, ContextCompileOutput, ContextCompiler, HarnessEventRecord,
    HarnessKeptRefs, HarnessObserver, HarnessStateKind, RecoveryBrief, TaskHarness,
};
use flauz_exec::ids::{AgentId, EnvironmentId, ModelId};
use flauz_exec::runtime::{RuntimeRequest, RuntimeStatus};
use flauz_exec::store::ExecStore;
use flauz_exec::{Agent, CapabilityId, ContractVersion as ExecContractVersion, Environment, ExecError, Model};
use flauz_exec::ActorRef as ExecActorRef;
use flauz_exec::Timestamp as ExecTimestamp;
use flauz_orch::evaluator::{GraphEvent, GraphEventSequence};
use flauz_orch::node::{AgentAssignment, NodeId, NodeState, RoleLabel, VerifierScope};
use flauz_orch::refs::{
    ActorRef as OrchActorRef, ArtifactRef as OrchArtifactRef, EvidenceRef as OrchEvidenceRef,
    TaskRef as OrchTaskRef, WaitOn,
};
use flauz_orch::ExecutionGraph;
use flauz_world::event_types;
use flauz_world::fakes::FakeWorldStore;
use flauz_world::refs::{ActorRef, EntityRef, StreamRef};
use flauz_world::{
    Artifact, ArtifactContent, CanonicalValue, Claim, Observation, Payload, Procedure,
    ProcedureInput, ProcedureStep, ProcedureVersion, Resource, ResourceBinding, ResourceState,
    SemanticVersion, SurfaceKind, Task, Timestamp, VerificationStatus, Workspace, WorldStore,
    AccessSurface, CapabilityKey,
};

fn ok<T, E: std::fmt::Debug>(r: Result<T, E>, what: &str) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("f6 gate: {what}: {e:?}"),
    }
}

fn ts(value: &str) -> Timestamp {
    ok(Timestamp::parse(value), "world timestamp")
}

fn exec_ts(value: &str) -> ExecTimestamp {
    ok(ExecTimestamp::parse(value), "exec timestamp")
}

fn ctx_ts(value: &str) -> ContextTimestamp {
    ok(ContextTimestamp::parse(value), "context timestamp")
}

fn user(id: &str) -> ActorRef {
    ok(ActorRef::user(id), "world actor")
}

/// Credential-material markers (mirrors the Wave-2 steps' independent
/// re-check of the no-credential-material invariant).
const CREDENTIAL_MARKERS: &[&str] = &[
    "sk-",
    "Bearer ",
    "api_key",
    "apikey",
    "password",
    "passwd",
    "client_secret",
    "access_token",
];

// ---------------------------------------------------------------------------
// The ORCH-003 seam implementations (gate-side wiring of the contract
// traits — exactly what a later slice wires in production)
// ---------------------------------------------------------------------------

/// The observer seam: collects the machine's own history (its records
/// ARE the recovery source) — the same records a world-side envelope
/// assigner would land on the task's stream.
#[derive(Default)]
struct CollectingObserver {
    records: Mutex<Vec<HarnessEventRecord>>,
}

impl HarnessObserver for CollectingObserver {
    fn record(&self, record: &HarnessEventRecord) -> Result<(), ExecError> {
        self.records.lock().expect("observer lock").push(record.clone());
        Ok(())
    }
}

/// The ORCH-002 engine behind the ORCH-003 compile seam: compiles the
/// task's durable state (projection, never transcript) for the attached
/// model through the REAL public `compile_from_durable_state` step.
struct DurableStateCompiler {
    inputs: DurableStateInputs,
    profile: ModelContextProfile,
    actor: ContextActorRef,
    at: ContextTimestamp,
    snapshots: Mutex<Vec<flauz_context::ContextSnapshot>>,
}

impl DurableStateCompiler {
    fn new(
        inputs: DurableStateInputs,
        profile: ModelContextProfile,
        actor: ContextActorRef,
        at: ContextTimestamp,
    ) -> Arc<Self> {
        Arc::new(Self {
            inputs,
            profile,
            actor,
            at,
            snapshots: Mutex::new(Vec::new()),
        })
    }
}

impl ContextCompiler for DurableStateCompiler {
    fn compile(&self, input: &ContextCompileInput) -> Result<ContextCompileOutput, ExecError> {
        if input.model_id.to_string() != self.profile.model_id.as_str() {
            return Err(ExecError::Invalid(format!(
                "the seam compiles for model {}, not {}",
                self.profile.model_id, input.model_id
            )));
        }
        let snapshot = compile_from_durable_state(
            &self.inputs,
            &self.profile,
            self.actor.clone(),
            self.at.clone(),
        )
        .map_err(|error| ExecError::Invalid(error.to_string()))?;
        let context_ref = snapshot.id.as_str().to_owned();
        self.snapshots
            .lock()
            .expect("compiler lock")
            .push(snapshot);
        Ok(ContextCompileOutput {
            v: ExecContractVersion,
            context_ref,
        })
    }
}

// ---------------------------------------------------------------------------
// The F6 scenario
// ---------------------------------------------------------------------------

pub fn f6_scenario_step() {
    // Fixed clock — the determinism rule (caller-supplied everywhere).
    let t0 = ts("2026-09-23T08:00:00Z");
    let t1 = ts("2026-09-23T08:05:00Z");
    let t2 = ts("2026-09-23T08:10:00Z");
    let t3 = ts("2026-09-23T08:15:00Z");
    let t4 = ts("2026-09-23T08:20:00Z");
    let t5 = ts("2026-09-23T08:25:00Z");
    let t6 = ts("2026-09-23T08:30:00Z");
    let t7 = ts("2026-09-23T08:35:00Z");
    let t8 = ts("2026-09-23T08:40:00Z");
    let t9 = ts("2026-09-23T08:45:00Z");
    let t10 = ts("2026-09-23T08:50:00Z");
    let t11 = ts("2026-09-23T08:55:00Z");
    let t12 = ts("2026-09-23T09:00:00Z");
    let t13 = ts("2026-09-23T09:05:00Z");
    let t14 = ts("2026-09-23T09:10:00Z");

    let actor = user("user_f6gate_lead");
    let research_agent_actor = user("agent_f6_research");
    let analysis_agent_actor = user("agent_f6_analysis");
    let review_agent_actor = user("agent_f6_review");

    // ---- [18a] The world: workspace + the domain-neutral research task --
    let ws_id = flauz_world::WorkspaceId::generate();
    let mut world = FakeWorldStore::new();
    let _ = ok(
        world.create_workspace(ok(
            Workspace::new(ws_id.clone(), "F6 gate workspace", actor.clone(), t0.clone()),
            "workspace ctor",
        )),
        "create workspace",
    );

    const OBJECTIVE: &str =
        "Research, analyze and independently verify the three most-cited renewable-energy \
         policy shifts of 2026";
    let task_id = flauz_world::TaskId::generate();
    let _ = ok(
        world.create_task(ok(
            Task::new(task_id.clone(), ws_id.clone(), OBJECTIVE, actor.clone(), t0.clone()),
            "task ctor",
        )),
        "create research task",
    );
    let sess_id = flauz_world::SessionId::generate();
    let _ = ok(
        world.create_session(ok(
            flauz_world::Session::new(
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

    // The browser resource: the research source (domain-neutral — policy
    // trackers on the web, not a codebase).
    let res_id = flauz_world::ResourceId::generate();
    let _ = ok(
        world.create_resource(ok(
            Resource::new(
                res_id.clone(),
                "Policy tracker web sources",
                vec![AccessSurface::new(SurfaceKind::Browser, res_id.clone())],
                actor.clone(),
                t0.clone(),
            ),
            "resource ctor",
        )),
        "create browser resource",
    );
    let _ = ok(
        world.put_resource_state(ResourceState {
            v: flauz_world::ContractVersion,
            resource_id: res_id.clone(),
            resource_version: 1,
            permissions: vec!["read".into()],
            last_verified_observation: None,
            authoritative_surface: Some(SurfaceKind::Browser),
            active_lease_ids: Vec::new(),
            pending_mutation_ids: Vec::new(),
            related_artifact_ids: Vec::new(),
            captured_by: research_agent_actor.clone(),
            captured_at: t0.clone(),
        }),
        "resource state",
    );

    // The exec side: the fake browser sandbox + both fake models.
    let mut exec = FakeExecStore::new();
    let sandbox_env_id =
        ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPT7"), "sandbox env id");
    let sandbox_env = ok(
        FakeRemoteEnvironment::new(sandbox_env_id.clone(), "F6 gate browser sandbox"),
        "browser sandbox",
    );
    let _ = ok(exec.create_environment(sandbox_env.descriptor()), "store sandbox env");

    let exec_actor = ok(ExecActorRef::user("user_f6gate_lead"), "exec actor");
    let exec_t0 = exec_ts("2026-09-23T08:00:00Z");
    let model_text = ok(
        Model::new(
            ok(ModelId::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPT7"), "model text id"),
            "Fake Research Model",
            "flauz-fake",
            vec![
                ok(CapabilityId::parse("browser.navigation"), "cap navigation"),
                ok(CapabilityId::parse("text.generation"), "cap text"),
                ok(CapabilityId::parse("web.search"), "cap search"),
            ],
            Some("browser-capable fake model"),
            exec_actor.clone(),
            exec_t0.clone(),
        ),
        "model text",
    );
    let model_vision = ok(
        Model::new(
            ok(ModelId::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPT8"), "model vision id"),
            "Fake Analysis Model",
            "flauz-fake",
            vec![
                ok(CapabilityId::parse("image.understanding"), "cap vision"),
                ok(CapabilityId::parse("text.generation"), "cap text"),
            ],
            Some("vision fake model"),
            exec_actor.clone(),
            exec_t0.clone(),
        ),
        "model vision",
    );
    let _ = ok(exec.create_model(model_text.clone()), "store model text");
    let _ = ok(exec.create_model(model_vision.clone()), "store model vision");
    // The three attributed agents (who/role — the graph's actors).
    for (name, runtime_kind, model) in [
        ("F6 research agent", "flauz-direct", &model_text),
        ("F6 analysis agent", "flauz-direct", &model_vision),
        ("F6 review agent", "flauz-direct", &model_text),
    ] {
        let _ = ok(
            exec.create_agent(ok(
                Agent::new(
                    AgentId::generate(),
                    name,
                    Some(runtime_kind),
                    Some(model.id.clone()),
                    exec_actor.clone(),
                    exec_t0.clone(),
                ),
                "agent ctor",
            )),
            "create agent",
        );
    }

    // ---- [18b] ORCH-003 harness drives the pipeline (the research half) --
    // The ORCH-002 engine behind the seam: durable state = the workspace
    // memory + (initially) nothing else. Context is a projection, never
    // a transcript.
    let memory_item = ok(
        MemoryItem::new(
            MemoryItemId::generate(),
            None,
            MemoryTier::Warm,
            AuthorizationClass::Workspace,
            MemoryContent::Text {
                text: "The analyst prefers bullet-point briefings with source links.".into(),
            },
            ok(ContextActorRef::user("user_f6gate_lead"), "context actor"),
            ctx_ts("2026-09-23T07:00:00Z"),
        ),
        "workspace memory item",
    );
    let inputs_v1 = ok(
        DurableStateInputs::new(
            ok(ContextTaskRef::parse(&task_id.to_string()), "context task ref"),
            Some(ok(
                ContextSessionRef::parse(&sess_id.to_string()),
                "context session ref",
            )),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![memory_item.clone()],
        ),
        "durable inputs v1",
    );
    let profile_text = ok(
        ModelContextProfile::new(
            ok(ContextModelRef::parse(&model_text.id.to_string()), "profile model ref"),
            128_000,
            MultimodalBehavior::TextOnly,
            ToolSchemaHandling::SummariesWithLazySchemas,
            ok(ContextActorRef::user("user_f6gate_lead"), "profile actor"),
            ctx_ts("2026-09-23T07:05:00Z"),
        ),
        "text model profile",
    );
    let compiler_v1 = DurableStateCompiler::new(
        inputs_v1.clone(),
        profile_text,
        ok(ContextActorRef::user("user_f6gate_lead"), "compiler actor"),
        ctx_ts("2026-09-23T08:05:00Z"),
    );
    let observer = Arc::new(CollectingObserver::default());
    let mut harness = ok(
        TaskHarness::prepare(
            &task_id.to_string(),
            exec_actor.clone(),
            model_text.id.clone(),
            Some(sandbox_env_id.clone()),
            Box::new(FakeNonCodexRuntime::new()),
            compiler_v1.clone(),
            observer.clone(),
            exec_ts("2026-09-23T08:05:00Z"),
        ),
        "harness prepare",
    );
    assert_eq!(harness.state(), HarnessStateKind::Preparing);
    assert_eq!(harness.task_id(), task_id.to_string());
    assert_eq!(harness.seq(), 1, "prepare emits exactly one record");

    // Execute the research turn: browser navigation + web search in the
    // fake sandbox (the SAME AgentRuntime contract as the codex fake).
    let research_request = ok(
        RuntimeRequest::new(
            model_text.id.clone(),
            Some(sandbox_env_id.clone()),
            vec![
                ok(CapabilityId::parse("browser.navigation"), "cap navigation"),
                ok(CapabilityId::parse("web.search"), "cap search"),
            ],
            "Research the policy shifts in the sandbox browser",
        ),
        "research runtime request",
    );
    let research_outcome = ok(
        harness.execute_turn(&research_request, exec_ts("2026-09-23T08:10:00Z")),
        "research turn",
    );
    assert_eq!(research_outcome.status, RuntimeStatus::Completed);

    // Observe: a world-side observation on the browser resource, then
    // the harness records its reference.
    let obs = ok(
        world.record_observation(ok(
            Observation::new(
                flauz_world::ObservationId::generate(),
                res_id.clone(),
                SurfaceKind::Browser,
                research_agent_actor.clone(),
                t1.clone(),
                "All three policy trackers confirm the three shifts; citation counts recorded",
            ),
            "observation ctor",
        )),
        "record observation",
    );
    let _ = ok(
        harness.observe(
            &obs.id.to_string(),
            &res_id.to_string(),
            "browser",
            exec_ts("2026-09-23T08:15:00Z"),
        ),
        "harness observe",
    );

    // The attributed artifacts: research notes (research agent) and the
    // analysis brief (analysis agent) — shared task state, the graph's
    // currency.
    let research_art = ok(
        world.create_artifact(ok(
            Artifact::new(
                flauz_world::ArtifactId::generate(),
                task_id.clone(),
                "Research notes: the three policy shifts",
                ArtifactContent::Text {
                    text: "1) Grid-feed tariff reform 2) Offshore wind permitting fast-track \
                           3) Cross-border green certificate mutual recognition"
                        .into(),
                },
                research_agent_actor.clone(),
                t2.clone(),
            ),
            "research artifact ctor",
        )),
        "research artifact",
    );
    let research_event = ok(
        world.append_event(
            StreamRef::task(&task_id),
            ok(
                flauz_world::Event::new(
                    event_types::ARTIFACT_PRODUCED,
                    t2.clone(),
                    research_agent_actor.clone(),
                    EntityRef::artifact(&research_art.id),
                    ok(
                        Payload::empty().with("title", CanonicalValue::Str("Research notes".into())),
                        "payload",
                    ),
                ),
                "event ctor",
            ),
        ),
        "artifact.produced event",
    );
    let analysis_art = ok(
        world.create_artifact(ok(
            Artifact::new(
                flauz_world::ArtifactId::generate(),
                task_id.clone(),
                "Analysis brief: impact assessment",
                ArtifactContent::Text {
                    text: "Tariff reform shifts utility economics; fast-track permitting \
                           compresses offshore timelines; mutual recognition unlocks cross-border \
                           trade."
                        .into(),
                },
                analysis_agent_actor.clone(),
                t3.clone(),
            ),
            "analysis artifact ctor",
        )),
        "analysis artifact",
    );

    // The claim + INDEPENDENT verification: the review agent (not the
    // analysis agent) turns the claim into evidence.
    let claim = ok(
        world.record_claim(ok(
            Claim::new(
                flauz_world::ClaimId::generate(),
                res_id.clone(),
                Some(SurfaceKind::Browser),
                analysis_agent_actor.clone(),
                t3.clone(),
                "The three shifts are the most-cited of 2026 and the brief covers each",
            ),
            "claim ctor",
        )),
        "record claim",
    );
    assert_eq!(claim.verification, VerificationStatus::Claimed);
    let verification = ok(
        world.verify_claim(
            claim.clone(),
            review_agent_actor.clone(),
            t4.clone(),
            StreamRef::task(&task_id),
            ok(
                Payload::empty().with(
                    "method",
                    CanonicalValue::Str("independent-cross-check".into()),
                ),
                "verification payload",
            ),
        ),
        "verify claim",
    );
    assert_eq!(
        verification.claim.verification,
        VerificationStatus::Verified,
        "the independent review verifies the claim"
    );
    assert_eq!(verification.evidence.verifier, review_agent_actor);

    // The harness verifies + persists + continues.
    let _ = ok(
        harness.verify(
            &claim.id.to_string(),
            &verification.evidence.id.to_string(),
            &verification.evidence.verification_event_id.to_string(),
            exec_ts("2026-09-23T08:25:00Z"),
        ),
        "harness verify",
    );
    let _ = ok(
        harness.persist(
            {
                let mut refs = vec![
                    research_art.id.to_string(),
                    analysis_art.id.to_string(),
                    obs.id.to_string(),
                    verification.evidence.id.to_string(),
                ];
                refs.sort();
                refs.dedup();
                refs
            },
            exec_ts("2026-09-23T08:30:00Z"),
        ),
        "harness persist",
    );
    let _ = ok(
        harness.continue_cycle(exec_ts("2026-09-23T08:35:00Z")),
        "harness continue",
    );

    // ---- [18c] The mid-run MODEL SWITCH through the ORCH-002 engine ----
    // The durable state is REBUILT from the world store as it stands
    // NOW (artifacts + observation + evidence + recent events + the
    // memory items): the rebuild law — no model conversation replayed.
    let inputs_v2 = ok(
        DurableStateInputs::new(
            ok(ContextTaskRef::parse(&task_id.to_string()), "context task ref v2"),
            Some(ok(
                ContextSessionRef::parse(&sess_id.to_string()),
                "context session ref v2",
            )),
            vec![
                ok(
                    ContextArtifactRecord::new(
                        ok(
                            flauz_context::ArtifactRef::parse(&research_art.id.to_string()),
                            "context artifact ref",
                        ),
                        "The research notes naming the three shifts",
                    ),
                    "artifact record research",
                ),
                ok(
                    ContextArtifactRecord::new(
                        ok(
                            flauz_context::ArtifactRef::parse(&analysis_art.id.to_string()),
                            "context artifact ref 2",
                        ),
                        "The analysis brief assessing each shift's impact",
                    ),
                    "artifact record analysis",
                ),
            ],
            vec![ok(
                ContextObservationRecord::new(
                    ok(
                        flauz_context::ObservationRef::parse(&obs.id.to_string()),
                        "context observation ref",
                    ),
                    "The browser session confirming all three trackers",
                ),
                "observation record",
            )],
            vec![ok(
                ContextEvidenceRecord::new(
                    ok(
                        flauz_context::engine::EvidenceRef::parse(
                            &verification.evidence.id.to_string(),
                        ),
                        "context evidence ref",
                    ),
                    ok(
                        ContextEventRef::parse(
                            &verification.evidence.verification_event_id.to_string(),
                        ),
                        "context verification event ref",
                    ),
                    "The independent review's verified cross-check",
                ),
                "evidence record",
            )],
            vec![ok(
                TaskEventRecord::new(
                    ok(
                        ContextEventRef::parse(&research_event.event_id.to_string()),
                        "context event ref",
                    ),
                    "The research notes were produced",
                ),
                "task event record",
            )],
            vec![memory_item.clone()],
        ),
        "durable inputs v2",
    );
    let profile_vision = ok(
        ModelContextProfile::new(
            ok(ContextModelRef::parse(&model_vision.id.to_string()), "vision model ref"),
            200_000,
            MultimodalBehavior::ImageInput,
            ToolSchemaHandling::SummariesWithLazySchemas,
            ok(ContextActorRef::user("user_f6gate_lead"), "profile actor v2"),
            ctx_ts("2026-09-23T08:38:00Z"),
        ),
        "vision model profile",
    );
    let task_version_before_switch = ok(world.task(&task_id), "task lookup").unwrap().version;
    let compiler_v2 = DurableStateCompiler::new(
        inputs_v2.clone(),
        profile_vision.clone(),
        ok(ContextActorRef::user("user_f6gate_lead"), "compiler actor v2"),
        ctx_ts("2026-09-23T08:40:00Z"),
    );
    let _ = ok(
        harness.reprepare(
            model_vision.id.clone(),
            Some(sandbox_env_id.clone()),
            Box::new(FakeNonCodexRuntime::new()),
            compiler_v2.clone(),
            exec_ts("2026-09-23T08:40:00Z"),
        ),
        "harness reprepare (the model switch)",
    );
    assert_eq!(harness.prepares(), 2, "a model switch is a re-prepare");
    assert_eq!(harness.task_id(), task_id.to_string(), "identity unchanged");
    assert_eq!(
        ok(world.task(&task_id), "task after switch").unwrap().version,
        task_version_before_switch,
        "the switch is execution state — canonical task state unchanged"
    );

    // The rebuild law: compiling the same durable state twice yields
    // EQUIVALENT projections (everything but compilation metadata).
    let snap_a = compiler_v2.snapshots.lock().expect("lock")[0].clone();
    let rebuild = ok(
        compile_from_durable_state(
            &inputs_v2,
            &profile_vision,
            ok(ContextActorRef::user("user_f6gate_lead"), "rebuild actor"),
            ctx_ts("2026-09-23T08:42:00Z"),
        ),
        "independent rebuild",
    );
    assert!(
        projections_equivalent(&snap_a, &rebuild),
        "context rebuilds from durable state without any conversation replay"
    );
    // The negative case: different durable state → NOT equivalent.
    let inputs_v3 = ok(
        DurableStateInputs::new(
            ok(ContextTaskRef::parse(&task_id.to_string()), "task ref v3"),
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![memory_item],
        ),
        "durable inputs v3",
    );
    let different = ok(
        compile_from_durable_state(
            &inputs_v3,
            &profile_vision,
            ok(ContextActorRef::user("user_f6gate_lead"), "different actor"),
            ctx_ts("2026-09-23T08:43:00Z"),
        ),
        "different-state compile",
    );
    assert!(
        !projections_equivalent(&snap_a, &different),
        "different durable state compiles to a different projection"
    );

    // One more turn under the switched model (vision-capable, same
    // browser sandbox — the fake completes it).
    let vision_request = ok(
        RuntimeRequest::new(
            model_vision.id.clone(),
            Some(sandbox_env_id.clone()),
            vec![ok(CapabilityId::parse("browser.input"), "cap browser input")],
            "Cross-check the citation charts under the switched model",
        ),
        "vision runtime request",
    );
    let vision_outcome = ok(
        harness.execute_turn(&vision_request, exec_ts("2026-09-23T08:45:00Z")),
        "post-switch turn",
    );
    assert_eq!(vision_outcome.status, RuntimeStatus::Completed);
    assert_eq!(harness.turns(), 2);

    // ---- [18d] RECOVERY: the machine's own history is the source ------
    let history = observer.records.lock().expect("lock").clone();
    assert!(!history.is_empty(), "the observer collected the machine's history");
    let history_json = ok(serde_json::to_string(&history), "history JSON");
    drop(harness);

    // Serialize → drop → reload → replay → resume: recovery is
    // reconstruction from durable state, never "start over".
    let history2: Vec<HarnessEventRecord> =
        ok(serde_json::from_str(&history_json), "history reparse");
    let replayed = ok(TaskHarness::replay(&history2), "replay");
    assert_eq!(replayed.task_id, task_id.to_string());
    assert_eq!(replayed.turns, 2, "the turn counter survives");
    assert_eq!(replayed.prepares, 2, "the prepare counter survives");

    let observer2 = Arc::new(CollectingObserver::default());
    let mut resumed = ok(
        TaskHarness::resume(
            replayed,
            exec_actor.clone(),
            Box::new(FakeNonCodexRuntime::new()),
            compiler_v2.clone(),
            observer2.clone(),
        ),
        "resume",
    );
    assert_eq!(resumed.task_id(), task_id.to_string());
    assert_eq!(resumed.turns(), 2);

    // The first-class recover continuation: the J-03 brief (what was
    // kept) is durable data on the recovered event.
    let brief = RecoveryBrief {
        kept: ok(
            HarnessKeptRefs::new(
                vec![
                    research_art.id.to_string(),
                    analysis_art.id.to_string(),
                ],
                vec![verification.evidence.id.to_string()],
                Vec::new(),
            ),
            "kept refs",
        ),
        compaction: None,
    };
    let recovered_event = ok(
        resumed.recover(brief, exec_ts("2026-09-23T08:50:00Z")),
        "recover",
    );
    assert_eq!(resumed.state(), HarnessStateKind::Recovering);
    assert_eq!(
        recovered_event.event_type,
        flauz_exec::harness::harness_event_types::HARNESS_RECOVERED
    );

    // The machine CONTINUES (not restarts): recover → continue → turn 3.
    let _ = ok(
        resumed.continue_cycle(exec_ts("2026-09-23T08:52:00Z")),
        "continue after recovery",
    );
    let resumed_outcome = ok(
        resumed.execute_turn(&vision_request, exec_ts("2026-09-23T08:55:00Z")),
        "post-recovery turn",
    );
    assert_eq!(resumed_outcome.status, RuntimeStatus::Completed);
    assert_eq!(resumed.turns(), 3, "the turn counter CONTINUES after recovery");
    let _ = ok(
        resumed.persist(
            vec![research_art.id.to_string()],
            exec_ts("2026-09-23T08:57:00Z"),
        ),
        "post-recovery persist",
    );
    let _ = ok(resumed.complete(exec_ts("2026-09-23T08:58:00Z")), "complete");
    assert_eq!(resumed.state(), HarnessStateKind::Done);

    // The machine's history lands on the task's world stream (envelope
    // assignment is world-side): every collected record appends as a
    // world event with the harness's registered event_type.
    let mut harness_seq_on_stream = 0;
    for record in observer
        .records
        .lock()
        .expect("lock")
        .iter()
        .chain(observer2.records.lock().expect("lock").iter())
    {
        let _ = ok(
            world.append_event(
                StreamRef::task(&task_id),
                ok(
                    flauz_world::Event::new(
                        &record.event_type,
                        ts("2026-09-23T09:00:00Z"),
                        actor.clone(),
                        EntityRef::task(&task_id),
                        ok(
                            Payload::empty().with(
                                "to_state",
                                CanonicalValue::Str(record.to_state.as_str().to_owned()),
                            ),
                            "harness payload",
                        ),
                    ),
                    "harness world event ctor",
                ),
            ),
            "harness event onto the task stream",
        );
        harness_seq_on_stream += 1;
    }
    assert!(harness_seq_on_stream >= 12, "the machine's history reached the task stream");

    // ---- [18e] The ORCH-004 attributed graph: research/analysis/review --
    let orch_task = ok(OrchTaskRef::parse(&task_id.to_string()), "orch task ref");
    let research_node = ok(AgentAssignment::worker(
        ok(NodeId::parse("research"), "node id research"),
        ok(OrchActorRef::agent("agent_f6_research"), "orch research actor"),
        ok(RoleLabel::parse("Research"), "role research"),
        ok(
            flauz_orch::node::NodeInputs::new(Vec::new()),
            "research inputs",
        ),
        NodeState::Ready,
        Some("done".to_owned()),
    ),
        "research node",
    );
    let analysis_node = ok(AgentAssignment::worker(
        ok(NodeId::parse("analysis"), "node id analysis"),
        ok(OrchActorRef::agent("agent_f6_analysis"), "orch analysis actor"),
        ok(RoleLabel::parse("Analysis"), "role analysis"),
        ok(
            flauz_orch::node::NodeInputs::new(vec![ok(
                WaitOn::artifact("research", "the research notes"),
                "analysis wait",
            )]),
            "analysis inputs",
        ),
        NodeState::Blocked,
        Some("done".to_owned()),
    ),
        "analysis node",
    );
    let review_node = ok(AgentAssignment::verifier(
        ok(NodeId::parse("review"), "node id review"),
        ok(OrchActorRef::agent("agent_f6_review"), "orch review actor"),
        ok(RoleLabel::parse("Independent review"), "role review"),
        ok(
            flauz_orch::node::NodeInputs::new(vec![
                ok(
                    WaitOn::artifact("research", "the research notes"),
                    "review wait research",
                ),
                ok(
                    WaitOn::artifact("analysis", "the analysis notes"),
                    "review wait analysis",
                ),
            ]),
            "review inputs",
        ),
        NodeState::Blocked,
        ok(
            VerifierScope::new(vec!["analysis".to_owned(), "research".to_owned()]),
            "verifier scope",
        ),
        None,
    ),
        "review node",
    );
    let graph = ok(
        ExecutionGraph::new(
            orch_task.clone(),
            vec![research_node, analysis_node, review_node],
            Some(OBJECTIVE),
        ),
        "execution graph",
    );

    // The events reference the WORLD-side artifact/evidence ids — the
    // graph's currency is shared task state, and the waits carry the
    // same labels the producing events carry.
    let events = ok(
        GraphEventSequence::new(vec![
            GraphEvent::NodeStarted {
                node: "research".to_owned(),
            },
            GraphEvent::ArtifactProduced {
                by_node: "research".to_owned(),
                artifact: ok(
                    OrchArtifactRef::parse(&research_art.id.to_string()),
                    "orch research artifact",
                ),
                label: "the research notes".to_owned(),
            },
            GraphEvent::NodeFinished {
                node: "research".to_owned(),
            },
            GraphEvent::NodeStarted {
                node: "analysis".to_owned(),
            },
            GraphEvent::ArtifactProduced {
                by_node: "analysis".to_owned(),
                artifact: ok(
                    OrchArtifactRef::parse(&analysis_art.id.to_string()),
                    "orch analysis artifact",
                ),
                label: "the analysis notes".to_owned(),
            },
            GraphEvent::NodeFinished {
                node: "analysis".to_owned(),
            },
            GraphEvent::NodeStarted {
                node: "review".to_owned(),
            },
            GraphEvent::EvidenceRecorded {
                by_node: "review".to_owned(),
                evidence: ok(
                    OrchEvidenceRef::parse(&verification.evidence.id.to_string()),
                    "orch evidence",
                ),
                label: "the review verdict".to_owned(),
            },
            GraphEvent::NodeFinished {
                node: "review".to_owned(),
            },
        ]),
        "graph event sequence",
    );
    let evaluation = ok(graph.advance(&events), "graph advance");
    let merge = evaluation
        .merge
        .as_ref()
        .expect("the run reaches the merge point");
    assert_eq!(merge.task, orch_task, "the merge point names the task");
    assert_eq!(
        merge.verified,
        vec!["analysis".to_owned(), "research".to_owned()],
        "BOTH workers independently verified (the derived overlay)"
    );
    assert_eq!(merge.done, vec!["review".to_owned()], "the reviewer is done");
    assert_eq!(merge.artifacts.len(), 2, "both artifacts attributed");
    assert_eq!(merge.evidence.len(), 1, "the verdict attributed");
    assert_eq!(evaluation.attribution.len(), 3, "the attribution ledger is complete");
    for node in &evaluation.nodes {
        if node.node_id.as_str() == "review" {
            assert_eq!(node.state, NodeState::Done);
        } else {
            assert_eq!(
                node.state,
                NodeState::Verified,
                "worker {} independently verified",
                node.node_id.as_str()
            );
        }
    }
    // The merge-point descriptor is the task-level verification event —
    // append it to the task's stream (the J-09 law).
    let _ = ok(
        world.append_event(
            StreamRef::task(&task_id),
            ok(
                flauz_world::Event::new(
                    merge.event_type.as_str(),
                    t5.clone(),
                    actor.clone(),
                    EntityRef::task(&task_id),
                    ok(
                        Payload::empty().with(
                            "verified",
                            CanonicalValue::Int(merge.verified.len() as i64),
                        ),
                        "merge payload",
                    ),
                ),
                "merge event ctor",
            ),
        ),
        "task.verification_merged event",
    );

    // ---- [18f] Save as a reusable workflow (the Procedure contract) ----
    // The steps are what ACTUALLY ran — never aspirational.
    let procedure_version = ProcedureVersion {
        major: 1,
        minor: 0,
        predecessor: None,
        objective: OBJECTIVE.to_owned(),
        preconditions: vec!["A browser-capable model and a web research source".into()],
        inputs: vec![ProcedureInput {
            name: "topic".to_owned(),
            kind: Some("text".to_owned()),
            description: Some("The policy research topic".into()),
        }],
        steps: vec![
            ProcedureStep {
                summary: "Research the sources in the browser sandbox".into(),
                details: Some("Navigate the trackers; record the observation".into()),
            },
            ProcedureStep {
                summary: "Write the analysis brief".into(),
                details: Some("Assess each shift's impact".into()),
            },
            ProcedureStep {
                summary: "Independent review".into(),
                details: Some("Cross-check the citations; verify the claim".into()),
            },
        ],
        required_capabilities: vec![
            ok(CapabilityKey::parse("browser.navigation"), "cap key navigation"),
            ok(CapabilityKey::parse("web.search"), "cap key search"),
            ok(CapabilityKey::parse("text.generation"), "cap key text"),
        ],
        resource_bindings: vec![ResourceBinding {
            role: "policy_sources".to_owned(),
            resource_id: res_id.clone(),
        }],
        validation_rules: Vec::new(),
        recovery_rules: Vec::new(),
        evidence: vec![verification.evidence.id.clone()],
        created_by: actor.clone(),
        created_at: t6.clone(),
    };
    let procedure = ok(
        Procedure::new(
            flauz_world::ProcedureId::generate(),
            "Renewable-energy policy shift research",
            procedure_version,
            actor.clone(),
            t6.clone(),
        ),
        "procedure ctor",
    );
    let stored_procedure = ok(world.create_procedure(procedure), "create procedure");
    assert_eq!(stored_procedure.version, 1);
    assert_eq!(
        stored_procedure.current().expect("current version").number(),
        SemanticVersion::new(1, 0),
        "the first saved version is 1.0"
    );
    let _ = ok(
        world.append_event(
            StreamRef::task(&task_id),
            ok(
                flauz_world::Event::new(
                    event_types::PROCEDURE_CREATED,
                    t7.clone(),
                    actor.clone(),
                    EntityRef::task(&task_id),
                    ok(
                        Payload::empty().with(
                            "name",
                            CanonicalValue::Str("Renewable-energy policy shift research".into()),
                        ),
                        "procedure payload",
                    ),
                ),
                "procedure event ctor",
            ),
        ),
        "procedure.created event",
    );

    // ---- [18g] Run-again as a NEW task (the identity law) --------------
    let source_stream_events = ok(
        world.events(&StreamRef::task(&task_id), 0, 100),
        "source stream before run-again",
    )
    .len() as u64;
    let task2_id = flauz_world::TaskId::generate();
    assert_ne!(task2_id, task_id, "run-again creates a NEW task, never a fork");
    let _ = ok(
        world.create_task(ok(
            Task::new(
                task2_id.clone(),
                ws_id.clone(),
                "Re-run: the renewable-energy policy shift research",
                actor.clone(),
                t8.clone(),
            ),
            "task2 ctor",
        )),
        "create the run-again task",
    );
    let _ = ok(
        world.append_event(
            StreamRef::task(&task2_id),
            ok(
                flauz_world::Event::new(
                    event_types::TASK_CREATED,
                    t8.clone(),
                    actor.clone(),
                    EntityRef::task(&task2_id),
                    ok(
                        Payload::empty().with(
                            "from_workflow",
                            CanonicalValue::Str(
                                "Renewable-energy policy shift research".into(),
                            ),
                        ),
                        "task2 payload",
                    ),
                ),
                "task2 event ctor",
            ),
        ),
        "task.created on the NEW task's stream",
    );

    // The saved workflow's shape re-instantiates for the new task: the
    // same graph shape (fresh node states), the same objective.
    let graph2 = ok(
        ExecutionGraph::new(
            ok(OrchTaskRef::parse(&task2_id.to_string()), "orch task2 ref"),
            vec![
                ok(AgentAssignment::worker(
                    ok(NodeId::parse("research"), "node id research 2"),
                    ok(OrchActorRef::agent("agent_f6_research"), "orch research actor 2"),
                    ok(RoleLabel::parse("Research"), "role research 2"),
                    ok(
                        flauz_orch::node::NodeInputs::new(Vec::new()),
                        "research inputs 2",
                    ),
                    NodeState::Ready,
                    None,
                ), "run-again research node"),
                ok(AgentAssignment::worker(
                    ok(NodeId::parse("analysis"), "node id analysis 2"),
                    ok(OrchActorRef::agent("agent_f6_analysis"), "orch analysis actor 2"),
                    ok(RoleLabel::parse("Analysis"), "role analysis 2"),
                    ok(
                        flauz_orch::node::NodeInputs::new(vec![ok(
                            WaitOn::artifact("research", "the research notes"),
                            "analysis wait 2",
                        )]),
                        "analysis inputs 2",
                    ),
                    NodeState::Blocked,
                    None,
                ), "run-again analysis node"),
                ok(AgentAssignment::verifier(
                    ok(NodeId::parse("review"), "node id review 2"),
                    ok(OrchActorRef::agent("agent_f6_review"), "orch review actor 2"),
                    ok(RoleLabel::parse("Independent review"), "role review 2"),
                    ok(
                        flauz_orch::node::NodeInputs::new(vec![
                            ok(
                                WaitOn::artifact("research", "the research notes"),
                                "review wait research 2",
                            ),
                            ok(
                                WaitOn::artifact("analysis", "the analysis notes"),
                                "review wait analysis 2",
                            ),
                        ]),
                        "review inputs 2",
                    ),
                    NodeState::Blocked,
                    ok(
                        VerifierScope::new(vec!["analysis".to_owned(), "research".to_owned()]),
                        "verifier scope 2",
                    ),
                    None,
                ), "run-again review node"),
            ],
            Some(OBJECTIVE),
        ),
        "the run-again graph",
    );
    // The new task's run reaches its own merge point (fresh artifacts —
    // attributed to the NEW run, never the old task's).
    let art2 = ok(
        world.create_artifact(ok(
            Artifact::new(
                flauz_world::ArtifactId::generate(),
                task2_id.clone(),
                "Research notes: the re-run",
                ArtifactContent::Text {
                    text: "The re-run reproduces the three shifts".into(),
                },
                research_agent_actor.clone(),
                t9.clone(),
            ),
            "re-run artifact ctor",
        )),
        "re-run artifact",
    );
    let events2 = ok(
        GraphEventSequence::new(vec![
            GraphEvent::NodeStarted {
                node: "research".to_owned(),
            },
            GraphEvent::ArtifactProduced {
                by_node: "research".to_owned(),
                artifact: ok(
                    OrchArtifactRef::parse(&art2.id.to_string()),
                    "orch re-run artifact",
                ),
                label: "the research notes".to_owned(),
            },
            GraphEvent::NodeFinished {
                node: "research".to_owned(),
            },
        ]),
        "re-run event sequence",
    );
    let evaluation2 = ok(graph2.advance(&events2), "re-run advance");
    assert!(
        evaluation2.merge.is_none(),
        "the re-run is mid-flight (its verifier has not finished) — honest, not merged"
    );

    // Identity discipline: the source task's stream is UNTOUCHED by the
    // run-again, and both tasks coexist with their own identities.
    let source_stream_after = ok(
        world.events(&StreamRef::task(&task_id), 0, 100),
        "source stream after run-again",
    );
    assert_eq!(
        source_stream_after.len() as u64,
        source_stream_events,
        "the run-again never appended to the source task's stream"
    );
    let both = world.snapshot();
    assert!(
        both.tasks.contains_key(&task_id) && both.tasks.contains_key(&task2_id),
        "both tasks coexist"
    );
    assert_eq!(
        ok(world.task(&task_id), "source task lookup").unwrap().id,
        task_id,
        "the source task's identity is intact"
    );
    assert!(
        both.procedures.contains_key(&stored_procedure.id),
        "the saved workflow survives in the world store"
    );

    // ---- [18h] Cross-cutting invariants ---------------------------------
    // Serialize → drop → reload: world equality (both tasks + the
    // procedure survive).
    let world_snap = world.snapshot();
    let world_json = ok(serde_json::to_string(&world_snap), "world snapshot JSON");
    drop(world);
    let mut world2 = ok(
        FakeWorldStore::restore(ok(
            serde_json::from_str::<flauz_world::WorldSnapshot>(&world_json),
            "world snapshot reparse",
        )),
        "world restore",
    );
    assert_eq!(world2.snapshot(), world_snap, "world state equality after reload");
    assert!(world2.task(&task2_id).is_ok(), "the re-run task survives reload");

    // The source task's stream still CONTINUES after the save (a saved
    // workflow never freezes its source).
    let continuation = ok(
        world2.append_event(
            StreamRef::task(&task_id),
            ok(
                flauz_world::Event::new(
                    event_types::TASK_MODEL_CHANGED,
                    t14.clone(),
                    actor.clone(),
                    EntityRef::task(&task_id),
                    ok(
                        ok(
                            Payload::empty().with(
                                "from",
                                CanonicalValue::Str(model_text.id.to_string()),
                            ),
                            "continuation payload from",
                        )
                        .with("to", CanonicalValue::Str(model_vision.id.to_string())),
                        "continuation payload to",
                    ),
                ),
                "continuation event ctor",
            ),
        ),
        "source task continuation event",
    );
    assert!(
        continuation.seq > source_stream_events,
        "the source task's stream continues (seq strictly increases)"
    );

    // No credential material anywhere in the run's serialized state.
    let mut everything = String::new();
    everything.push_str(&world_json);
    everything.push_str(&history_json);
    everything.push_str(&ok(serde_json::to_string(&evaluation), "evaluation JSON"));
    everything.push_str(&ok(serde_json::to_string(&exec.snapshot()), "exec snapshot JSON"));
    everything.push_str(&ok(
        serde_json::to_string(&stored_procedure),
        "procedure JSON",
    ));
    everything.push_str(&ok(
        serde_json::to_string(&*observer2.records.lock().expect("lock")),
        "resumed history JSON",
    ));
    for marker in CREDENTIAL_MARKERS {
        assert!(
            !everything.contains(marker),
            "credential material marker {marker:?} must not appear anywhere"
        );
    }

    let _ = (t10, t11, t12, t13); // the fixed clock is complete; unused tails
}
