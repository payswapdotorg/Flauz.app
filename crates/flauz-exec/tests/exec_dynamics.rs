//! Execution-state dynamics: sourcing, storing, snapshot round-trips and
//! the model-switch semantics of exec entities (mirroring the world
//! crate's dynamics suite).

use std::fmt;

use flauz_exec::capability::CapabilityId;
use flauz_exec::fakes::{
    FakeCodexRuntime, FakeExecStore, FakeExecutionProvider, FakeLocalEnvironment,
    FakeModelProvider, FakeNonCodexRuntime, FakeRemoteEnvironment, fake_model_ids,
    fake_provider_connection,
};
use flauz_exec::ids::{EnvironmentId, ModelId};
use flauz_exec::refs::ActorRef;
use flauz_exec::time::Timestamp;
use flauz_exec::{
    Agent, AgentRuntime, Environment, EnvironmentSpec, ExecStore, ExecStoreError,
    ExecutionProvider, Locality, ModelProvider, RuntimeRequest, RuntimeStatus,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn test_timestamp() -> Timestamp {
    test_ok(Timestamp::parse("2026-09-21T13:45:00Z"))
}

fn test_actor() -> ActorRef {
    test_ok(ActorRef::user("alice"))
}

fn parse_capability(key: &str) -> CapabilityId {
    test_ok(CapabilityId::parse(key))
}

/// The full execution state survives serialize → drop → reload with
/// equality (kernel round-trip law; the F2 gate's serialize step).
#[test]
fn execution_state_survives_snapshot_roundtrip() {
    let mut store = FakeExecStore::new();

    let local = test_ok(FakeLocalEnvironment::with_default_id());
    let remote = test_ok(FakeRemoteEnvironment::with_default_id());
    test_ok(store.create_environment(local.descriptor()));
    test_ok(store.create_environment(remote.descriptor()));

    let provider = test_ok(FakeModelProvider::new());
    for model in provider.models() {
        test_ok(store.create_model(model.clone()));
    }

    test_ok(store.create_agent(test_ok(Agent::new(
        test_ok(flauz_exec::AgentId::parse(
            "agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6",
        )),
        "Codex agent",
        Some("codex-app-server"),
        Some(test_ok(ModelId::parse(fake_model_ids::TEXT))),
        test_actor(),
        test_timestamp(),
    ))));
    test_ok(store.create_connection(test_ok(fake_provider_connection())));

    // serialize → drop → reload.
    let snapshot = store.snapshot();
    let serialized = test_ok(serde_json::to_string(&snapshot));
    drop(store);
    let reloaded_snapshot: flauz_exec::ExecSnapshot = test_ok(serde_json::from_str(&serialized));
    let mut reloaded = test_ok(FakeExecStore::restore(reloaded_snapshot));

    // State equality.
    assert_eq!(reloaded.snapshot(), snapshot);

    // And the reloaded state keeps working: a mutation on the reloaded
    // store observes the reloaded versions.
    let agent = test_ok(flauz_exec::AgentId::parse(
        "agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6",
    ));
    let stored = match test_ok(reloaded.agent(&agent)) {
        Some(agent) => agent,
        None => panic!("the agent must survive the reload"),
    };
    assert_eq!(stored.version, 1);
    let mut switched = stored;
    switched.model_id = Some(test_ok(ModelId::parse(fake_model_ids::VISION)));
    let updated = test_ok(reloaded.update_agent(switched));
    assert_eq!(updated.version, 2);
}

/// The fake providers and runtimes support the F2 gate flow: source
/// environments, register models and agents, execute turns across both
/// runtimes, and switch models without identity changes.
#[test]
fn fake_providers_support_the_gate_flow() {
    // Source one environment per locality.
    let mut execution_provider = FakeExecutionProvider::new();
    let mut sourced_ids = Vec::new();
    for (name, locality) in [
        ("Local workstation", Locality::Local),
        ("Remote sandbox", Locality::Remote),
    ] {
        let spec = test_ok(EnvironmentSpec::new(
            name,
            locality,
            vec![parse_capability("terminal")],
            test_actor(),
            test_timestamp(),
        ));
        let descriptor = test_ok(execution_provider.source_environment(&spec));
        assert_eq!(descriptor.locality, locality);
        assert_eq!(descriptor.version, 1);
        sourced_ids.push(descriptor.id.clone());
    }
    assert_eq!(execution_provider.environments().len(), 2);
    // Sourced environments have generated canonical identities (the
    // crate's only entropy source).
    for id in &sourced_ids {
        test_ok(EnvironmentId::parse(id.as_str()));
    }

    // Register both fake models and execute one turn on each runtime —
    // the SAME request shape works for the Codex and the non-Codex runtime.
    let _provider = test_ok(FakeModelProvider::new());
    let text = test_ok(ModelId::parse(fake_model_ids::TEXT));
    let vision = test_ok(ModelId::parse(fake_model_ids::VISION));
    for runtime in [
        &FakeCodexRuntime::new() as &dyn AgentRuntime,
        &FakeNonCodexRuntime::new(),
    ] {
        let required: Vec<CapabilityId> = runtime.capabilities().iter().take(1).cloned().collect();
        let request = test_ok(RuntimeRequest::new(
            text.clone(),
            Some(sourced_ids[0].clone()),
            required,
            "Do the work",
        ));
        let outcome = test_ok(runtime.execute(&request));
        test_ok(outcome.validate());
        assert_eq!(outcome.status, RuntimeStatus::Completed);
    }

    // Model switch at the exec level: the agent identity is untouched and
    // the version increments exactly once.
    let mut store = FakeExecStore::new();
    let agent_id = test_ok(flauz_exec::AgentId::parse(
        "agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6",
    ));
    let agent = test_ok(Agent::new(
        agent_id.clone(),
        "Switching agent",
        Some("flauz-direct"),
        Some(text),
        test_actor(),
        test_timestamp(),
    ));
    let created = test_ok(store.create_agent(agent));
    let mut switched = created.clone();
    switched.model_id = Some(vision);
    let updated = test_ok(store.update_agent(switched));
    assert_eq!(updated.id, agent_id);
    assert_eq!(
        updated.model_id,
        Some(test_ok(ModelId::parse(fake_model_ids::VISION)))
    );
    assert_eq!(updated.version, 2);

    // Stale writes still conflict after the switch.
    let conflict = store
        .update_agent(created)
        .err()
        .unwrap_or_else(|| panic!("a stale expected version must conflict"));
    assert!(matches!(conflict, ExecStoreError::VersionConflict { .. }));
}
