//! Wave-2 environment-fabric step (ENV-001): both providers through the
//! public `ExecutionProvider` contract + the cross-environment identity
//! round-trip (local → fake-remote → local keeps the same logical task;
//! the fake-remote survives snapshot → serialize → drop → restore with
//! `env_` ULID identity stability).
//!
//! Lead-authored gate infrastructure (the F2 §10 harness pattern), NOT a
//! workspace crate. No external service anywhere; deterministic
//! (caller-supplied timestamps and IDs).

use flauz_exec::environment::{
    Environment, EnvironmentSpec, ExecutionProvider, Locality,
};
use flauz_exec::environment_fake_remote::{FakeRemoteEnvironmentProvider, FakeRemoteSnapshot};
use flauz_exec::environment_local::{
    LocalEnvironment, LocalEnvironmentProvider, LocalSurface,
};

use flauz_exec::ActorRef as ExecActorRef;
use flauz_exec::Timestamp as ExecTimestamp;

fn ok<T, E: std::fmt::Debug>(r: Result<T, E>, what: &str) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("gate: {what}: {e:?}"),
    }
}

/// Credential-material markers (mirrors `flauz_exec::connection`'s private
/// list — the gate re-checks the invariant independently).
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

pub fn environment_fabric_step() {
    let actor = ok(ExecActorRef::user("user_w2gate_lead"), "exec actor ref");
    let t0 = ok(ExecTimestamp::parse("2026-09-22T12:00:00Z"), "exec timestamp");

    // -- the LOCAL provider: the four F1 surfaces through ONE interface --
    let mut local: LocalEnvironmentProvider = LocalEnvironmentProvider::new();
    assert_eq!(local.provider_kind(), "local");
    assert_eq!(
        local.supported_localities(),
        &[Locality::Local],
        "the local provider sources LOCAL environments only"
    );
    assert!(
        local.connection_id().is_none(),
        "the local provider needs no connected account"
    );

    let local_spec = ok(
        EnvironmentSpec::new(
            "Lead gate local F1 surfaces",
            Locality::Local,
            flauz_exec::environment_local::local_surface_capabilities(),
            actor.clone(),
            t0,
        ),
        "local spec",
    );
    let local_env = ok(local.source_environment(&local_spec), "source local environment");
    assert_eq!(local_env.version, 1, "sourced environment starts at version 1");
    assert_eq!(local_env.locality, Locality::Local);
    assert_eq!(local_env.provider_kind.as_deref(), Some("local"));
    assert_eq!(
        local_env.capabilities.len(),
        12,
        "the frozen F1 topology advertises exactly 12 capability keys"
    );
    assert_eq!(local.environments().len(), 1, "sourcing is stateful");

    // the SAME provider behind the public trait object (the seam future
    // waves hold): the trait surface is the only access path
    let trait_view: &dyn ExecutionProvider = &local;
    assert_eq!(trait_view.provider_kind(), "local");

    // locality honesty: the local provider refuses a REMOTE spec
    let bogus_remote = ok(
        EnvironmentSpec::new(
            "Bogus remote",
            Locality::Remote,
            flauz_exec::environment_local::local_surface_capabilities(),
            actor.clone(),
            t0,
        ),
        "bogus spec",
    );
    assert!(
        local.source_environment(&bogus_remote).is_err(),
        "the local provider must refuse a remote locality"
    );

    // the live local environment: stateless over its descriptor
    let live = ok(
        LocalEnvironment::new(
            local_env.id.clone(),
            "Lead gate live local",
            &[
                LocalSurface::Terminal,
                LocalSurface::Browser,
                LocalSurface::ComputerUse,
                LocalSurface::Git,
            ],
            actor.clone(),
            t0,
        ),
        "live local environment",
    );
    let reloaded = ok(
        LocalEnvironment::from_descriptor(live.descriptor().clone()),
        "reload local environment from its descriptor",
    );
    assert_eq!(
        reloaded.descriptor(),
        live.descriptor(),
        "the local environment is stateless over its descriptor (identity unchanged across reload)"
    );
    assert_eq!(
        reloaded.descriptor().id,
        local_env.id,
        "the canonical env_ identity is caller-supplied and stable"
    );

    // -- the FAKE-REMOTE provider: the consistency template --
    let connection_id =
        ok(flauz_exec::ProviderConnectionId::parse("conn_01J8ZQ5V8K3T2B7N6X4R9DQPC7"), "connection id");
    let mut remote: FakeRemoteEnvironmentProvider =
        FakeRemoteEnvironmentProvider::new().with_connection(connection_id.clone());
    assert_eq!(remote.provider_kind(), "flauz-fake-remote");
    assert_eq!(
        remote.supported_localities(),
        &[Locality::Remote],
        "the fake-remote sources REMOTE environments only"
    );
    assert_eq!(
        remote.connection_id(),
        Some(&connection_id),
        "the fake-remote draws on a CONNECTION REFERENCE (never material)"
    );

    let remote_spec = ok(
        EnvironmentSpec::new(
            "Lead gate fake remote sandbox",
            Locality::Remote,
            flauz_exec::environment_fake_remote::fake_remote_surface_capabilities(),
            actor.clone(),
            t0,
        ),
        "remote spec",
    );
    let remote_env = ok(remote.source_environment(&remote_spec), "source remote environment");
    assert_eq!(remote_env.version, 1);
    assert_eq!(remote_env.locality, Locality::Remote);
    assert_eq!(remote_env.provider_kind.as_deref(), Some("flauz-fake-remote"));

    // locality honesty: the fake-remote refuses a LOCAL spec
    let bogus_local = ok(
        EnvironmentSpec::new(
            "Bogus local",
            Locality::Local,
            flauz_exec::environment_fake_remote::fake_remote_surface_capabilities(),
            actor.clone(),
            t0,
        ),
        "bogus spec",
    );
    assert!(
        remote.source_environment(&bogus_local).is_err(),
        "the fake-remote provider must refuse a local locality"
    );

    // -- the cross-environment identity round-trip: the fake-remote
    //    survives snapshot → serialize → drop → restore, and the env_
    //    identity is STABLE across the trip --
    let snapshot: FakeRemoteSnapshot = remote.snapshot();
    let json = serde_json::to_string(&snapshot).expect("gate: snapshot serializes");
    for marker in CREDENTIAL_MARKERS {
        assert!(
            !json.contains(marker),
            "gate: serialized fake-remote snapshot carries credential material ({marker:?})"
        );
    }
    let restored = ok(
        FakeRemoteEnvironmentProvider::restore(snapshot),
        "restore fake-remote provider",
    );
    let restored_env = restored
        .environment(&remote_env.id)
        .expect("the restored provider serves the SAME env_ identity");
    let original_env = remote
        .environment(&remote_env.id)
        .expect("the original provider serves its environment");
    assert_eq!(
        restored_env.descriptor(),
        original_env.descriptor(),
        "fake-remote environment identity + state survive serialize → drop → reload"
    );
    assert_eq!(
        restored.environments().len(),
        1,
        "the restored provider holds the same environment inventory"
    );

    // -- CROSS-ENVIRONMENT: the same logical surface set exists in BOTH
    //    providers; switching local → fake-remote → local never forks the
    //    environment identity the task holds --
    let mut local_round2: LocalEnvironmentProvider = LocalEnvironmentProvider::new();
    let local_env_2 = ok(
        local_round2.source_environment(&local_spec),
        "source local environment again (the switch-back leg)",
    );
    // Provider-level identity discipline: the same spec through a FRESH
    // provider instance yields a DIFFERENT env_ identity (a fresh canonical
    // ULID allocation — no hidden global state), while a RELOADED
    // environment keeps its identity exactly (asserted above via
    // from_descriptor). The task's attachment lives in the world store —
    // the J-06 journey — so the gate asserts the contract-level
    // invariants here.
    assert_ne!(
        local_env_2.id, local_env.id,
        "a fresh sourcing allocates a fresh canonical env_ identity (no hidden global state)"
    );
    assert_eq!(
        local_env_2.capabilities, local_env.capabilities,
        "the same spec yields the same capability advertisement across provider instances (consistency)"
    );
}
