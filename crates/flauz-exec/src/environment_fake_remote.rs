//! The fake-consistency remote environment provider (ENV-001, Wave-2
//! kernel addendum §2): the FIRST external provider is a FAKE.
//!
//! [`FakeRemoteEnvironmentProvider`] sources deterministic, in-memory
//! remote sandboxes satisfying the SAME frozen [`Environment`] and
//! [`ExecutionProvider`] contracts as
//! [`LocalEnvironmentProvider`](crate::environment_local::LocalEnvironmentProvider)
//! — provider neutrality proven by the shared conformance test below.
//! It is the template every later REAL remote provider (E2B, Daytona,
//! Azure, …) must copy: honest capability sourcing with named gaps,
//! canonical `env_` identities stable across serialization, connection
//! REFERENCES only, and fully deterministic behavior.
//!
//! # Identity across serialization (addendum §2)
//!
//! A sourced environment's `env_` ULID is its identity, carried by the
//! durable [`EnvironmentDescriptor`] and preserved through every
//! canonical-JSON round-trip: the descriptor itself, the
//! [`ExecStore`](crate::ExecStore) snapshot that stores it, and the
//! provider's own [`FakeRemoteSnapshot`] — `snapshot()` → serialize →
//! drop → `restore()` yields the SAME canonical identities.
//!
//! # Credentials are references, never material (addendum §3)
//!
//! The provider optionally references the user-owned
//! [`ProviderConnection`](crate::ProviderConnection) through which it is
//! accessed (where provider quota belongs). The reference is a canonical
//! `conn_` ID; the connection it names carries only an opaque
//! `flausec_` [`SecretRef`](crate::SecretRef). No contract type, fixture,
//! log line or serialized state in this module contains credential
//! material — the tests prove it.
//!
//! # Honest capability sourcing (addendum §4)
//!
//! The fake sandbox has a fixed surface topology
//! ([`fake_remote_surface_capabilities`]). Sourcing a spec whose
//! capabilities fall outside it is rejected with the missing keys NAMED
//! — never a silent fall-through. Notably the fake sandbox does NOT
//! offer the F1 Computer Use surface: `desktop.gui` is a named gap here,
//! exactly as it would be on a real remote sandbox.
//!
//! # Cross-environment task identity (addendum §6, kernel §6)
//!
//! The evidence test `task_identity_survives_local_fake_remote_local_switch`
//! runs the environment journey local → fake-remote → local across a
//! serialize → drop → reload boundary and asserts: the SAME logical
//! task reference, environment events recorded on the task stream
//! (the exec-registered `environment.attached` / `environment.detached`
//! vocabulary), strictly increasing stream sequence with no fork, and
//! stable `env_` identities on both sides. The world-side
//! `task.environment_changed` events on the same journey are recorded by
//! the Lead's F2 gate harness through `flauz-world`'s `FakeWorldStore`
//! (cross-crate, Lead-owned — this crate stays self-contained per kernel
//! §1, so the test-local `TestTaskStream` stand-in carries the same
//! semantics here).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;
use crate::environment::{
    Environment, EnvironmentDescriptor, EnvironmentSpec, EnvironmentStatus, ExecutionProvider,
    Locality,
};
use crate::ids::{EnvironmentId, ProviderConnectionId};
use crate::{ContractVersion, ExecError, MAX_ENVIRONMENTS_PER_PROVIDER, ensure_list_bound};

/// The neutral kind label of the fake-consistency remote provider.
pub const FAKE_REMOTE_PROVIDER_KIND: &str = "flauz-fake-remote";

/// The capability advertisement of the fake remote sandbox: the fixed
/// surface topology every sourced sandbox offers. Valid, sorted,
/// deduplicated keys — everything beyond it cannot be sourced by this
/// provider (the gap is named, never silent).
#[must_use]
pub fn fake_remote_surface_capabilities() -> Vec<CapabilityId> {
    [
        "browser.input",
        "browser.navigation",
        "filesystem.read",
        "filesystem.write",
        "git",
        "ports",
        "snapshots",
        "terminal",
    ]
    .iter()
    .map(|key| frozen_capability(key))
    .collect()
}

/// Panics on the impossible: a frozen capability key that must always
/// parse (the keys come from the frozen vocabulary, never from input).
fn frozen_capability(key: &str) -> CapabilityId {
    match CapabilityId::parse(key) {
        Ok(value) => value,
        Err(error) => panic!("frozen capability key {key:?} must always parse: {error}"),
    }
}

/// The live surface of a fake remote sandbox: deterministic, in-memory,
/// no I/O — the fake-consistency counterpart of
/// [`LocalEnvironment`](crate::environment_local::LocalEnvironment)
/// under the SAME frozen [`Environment`] contract. Stateless over its
/// durable [`EnvironmentDescriptor`], so the canonical `env_` identity
/// lives entirely in the descriptor and survives serialization.
#[derive(Debug, Clone)]
pub struct FakeRemoteSandbox {
    descriptor: EnvironmentDescriptor,
}

impl FakeRemoteSandbox {
    /// Reconstructs the live fake remote sandbox from a durable
    /// descriptor (the reload path after a serialization round-trip):
    /// the canonical `env_` identity is the descriptor's, unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the descriptor is not a fake
    /// remote sandbox: wrong locality, a provider kind other than
    /// `flauz-fake-remote` (or absent), or capabilities outside the
    /// sandbox topology — each named in the error.
    pub fn from_descriptor(descriptor: EnvironmentDescriptor) -> Result<Self, ExecError> {
        descriptor.validate()?;
        if descriptor.locality != Locality::Remote {
            return Err(ExecError::invalid(format!(
                "a fake remote sandbox must have locality `remote`, found `{:?}`",
                descriptor.locality
            )));
        }
        match descriptor.provider_kind.as_deref() {
            Some(FAKE_REMOTE_PROVIDER_KIND) | None => {}
            Some(other) => {
                return Err(ExecError::invalid(format!(
                    "a fake remote sandbox is sourced by provider kind \
                     `{FAKE_REMOTE_PROVIDER_KIND}`, found {other:?}"
                )));
            }
        }
        let topology = fake_remote_surface_capabilities();
        let foreign: Vec<String> = descriptor
            .capabilities
            .iter()
            .filter(|capability| !topology.contains(capability))
            .map(|capability| capability.as_str().to_owned())
            .collect();
        if !foreign.is_empty() {
            return Err(ExecError::invalid(format!(
                "the fake remote sandbox topology does not offer: {}",
                foreign.join(", ")
            )));
        }
        Ok(Self { descriptor })
    }
}

impl Environment for FakeRemoteSandbox {
    fn environment_id(&self) -> &EnvironmentId {
        &self.descriptor.id
    }

    fn locality(&self) -> Locality {
        self.descriptor.locality
    }

    fn capabilities(&self) -> &[CapabilityId] {
        &self.descriptor.capabilities
    }

    fn status(&self) -> EnvironmentStatus {
        self.descriptor.status
    }

    fn descriptor(&self) -> EnvironmentDescriptor {
        self.descriptor.clone()
    }
}

/// The serialized state of a [`FakeRemoteEnvironmentProvider`]: the
/// canonical-JSON projection of the remote environments it sourced and
/// the connection reference it draws on. Round-trips through
/// `snapshot()` → serialize → drop → `restore()` with every `env_`
/// identity intact — the discipline later real remote providers copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FakeRemoteSnapshot {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The user-owned connection the provider draws on, where provider
    /// quota belongs. A canonical `conn_` REFERENCE — never credential
    /// material.
    pub connection_id: Option<ProviderConnectionId>,
    /// The sourced environments, in canonical ID order (sorted,
    /// deduplicated, bounded).
    pub environments: Vec<EnvironmentDescriptor>,
}

impl FakeRemoteSnapshot {
    /// Validates the snapshot against the canonical rules: bounded,
    /// sorted and deduplicated by canonical `env_` ID, every environment
    /// a valid fake remote sandbox descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the environment list is
    /// oversized, unsorted, duplicated, or contains a descriptor that is
    /// not a fake remote sandbox (wrong locality, foreign provider kind,
    /// or a capability outside the sandbox topology — named in the
    /// error).
    pub fn validate(&self) -> Result<(), ExecError> {
        ensure_list_bound(
            "fake remote environments",
            &self.environments,
            MAX_ENVIRONMENTS_PER_PROVIDER,
        )?;
        for environment in &self.environments {
            FakeRemoteSandbox::from_descriptor(environment.clone())?;
        }
        if self
            .environments
            .windows(2)
            .any(|pair| pair[0].id >= pair[1].id)
        {
            return Err(ExecError::invalid(
                "fake remote environments must be in canonical ID order, free of duplicates",
            ));
        }
        Ok(())
    }
}

/// The fake-consistency remote execution provider (addendum §2): the
/// first external provider is a FAKE — deterministic, in-memory, the
/// template every later real remote provider (E2B, Daytona, …) copies.
///
/// - `provider_kind` is [`FAKE_REMOTE_PROVIDER_KIND`]
///   (`flauz-fake-remote`).
/// - Only [`Locality::Remote`] can be sourced; a local spec is a named
///   contract error.
/// - Optionally references a user-owned
///   [`ProviderConnection`](crate::ProviderConnection) (a canonical
///   `conn_` reference; quota belongs to the connection where
///   supported) — sourced descriptors carry the reference as
///   provenance.
#[derive(Debug, Clone, Default)]
pub struct FakeRemoteEnvironmentProvider {
    environments: BTreeMap<EnvironmentId, EnvironmentDescriptor>,
    connection_id: Option<ProviderConnectionId>,
}

impl FakeRemoteEnvironmentProvider {
    /// An empty fake remote provider that has sourced no environments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches a user-owned connection through which this provider is
    /// accessed (where provider quota belongs). Sourced environments
    /// carry the connection REFERENCE — never credential material.
    #[must_use]
    pub fn with_connection(mut self, connection_id: ProviderConnectionId) -> Self {
        self.connection_id = Some(connection_id);
        self
    }

    /// The live [`FakeRemoteSandbox`] for a sourced canonical `env_` ID,
    /// if this provider sourced it. In-memory records are validated at
    /// source and restore time, so reconstruction cannot fail.
    #[must_use]
    pub fn environment(&self, id: &EnvironmentId) -> Option<FakeRemoteSandbox> {
        self.environments
            .get(id)
            .cloned()
            .and_then(|descriptor| FakeRemoteSandbox::from_descriptor(descriptor).ok())
    }

    /// The serialized state of this provider (for storage and the
    /// serialize → drop → reload discipline). Environments are listed in
    /// canonical ID order.
    #[must_use]
    pub fn snapshot(&self) -> FakeRemoteSnapshot {
        FakeRemoteSnapshot {
            v: ContractVersion,
            connection_id: self.connection_id.clone(),
            environments: self.environments.values().cloned().collect(),
        }
    }

    /// Rebuilds the provider from a serialized snapshot. Every `env_`
    /// identity is preserved: the restored provider resolves the SAME
    /// canonical IDs.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError`] when the snapshot fails validation (see
    /// [`FakeRemoteSnapshot::validate`]).
    pub fn restore(snapshot: FakeRemoteSnapshot) -> Result<Self, ExecError> {
        snapshot.validate()?;
        let environments: BTreeMap<EnvironmentId, EnvironmentDescriptor> = snapshot
            .environments
            .into_iter()
            .map(|descriptor| (descriptor.id.clone(), descriptor))
            .collect();
        Ok(Self {
            environments,
            connection_id: snapshot.connection_id,
        })
    }
}

impl ExecutionProvider for FakeRemoteEnvironmentProvider {
    fn provider_kind(&self) -> &str {
        FAKE_REMOTE_PROVIDER_KIND
    }

    fn supported_localities(&self) -> &[Locality] {
        &[Locality::Remote]
    }

    fn connection_id(&self) -> Option<&ProviderConnectionId> {
        self.connection_id.as_ref()
    }

    fn source_environment(
        &mut self,
        spec: &EnvironmentSpec,
    ) -> Result<EnvironmentDescriptor, ExecError> {
        spec.validate()?;
        if !self.supported_localities().contains(&spec.locality) {
            return Err(ExecError::invalid(format!(
                "provider `{}` cannot source {:?} environments",
                self.provider_kind(),
                spec.locality
            )));
        }
        // Honest sourcing (addendum §4): the sandbox topology is
        // everything a fake remote environment can offer — gaps are
        // named, never silent.
        let topology = fake_remote_surface_capabilities();
        let missing = crate::runtime::missing_capabilities(&topology, &spec.capabilities);
        if !missing.is_empty() {
            let keys: Vec<String> = missing.iter().map(|key| key.as_str().to_owned()).collect();
            return Err(ExecError::invalid(format!(
                "the fake remote sandbox topology does not offer: {}",
                keys.join(", ")
            )));
        }
        if self.environments.len() >= MAX_ENVIRONMENTS_PER_PROVIDER {
            return Err(ExecError::invalid(format!(
                "an execution provider exposes at most {MAX_ENVIRONMENTS_PER_PROVIDER} environments"
            )));
        }
        // ID generation is the crate's only entropy source (kernel §7):
        // the env_ ULID allocated here is the environment's identity,
        // stable across every later serialization.
        let descriptor = EnvironmentDescriptor::new(
            EnvironmentId::generate(),
            &spec.name,
            spec.locality,
            Some(self.provider_kind()),
            spec.capabilities.clone(),
            EnvironmentStatus::Available,
            self.connection_id.clone(),
            spec.requested_by.clone(),
            spec.requested_at,
        )?;
        self.environments
            .insert(descriptor.id.clone(), descriptor.clone());
        Ok(descriptor)
    }

    fn environments(&self) -> Vec<EnvironmentDescriptor> {
        self.environments.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment_local::{
        LocalEnvironment, LocalEnvironmentProvider, local_surface_capabilities,
    };
    use crate::event_types;
    use crate::fakes::{FakeCodexRuntime, FakeExecStore, fake_model_ids};
    use crate::ids::{EntityKind, EnvironmentId, ModelId, ProviderConnectionId, validate};
    use crate::refs::ActorRef;
    use crate::time::Timestamp;
    use crate::{AgentRuntime, Environment, ExecStore, RuntimeRequest, RuntimeStatus};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    /// Unwraps the expected error side of a contract result (the crate's
    /// no-expect/no-unwrap test discipline).
    fn err<T, E: std::fmt::Display>(result: Result<T, E>, what: &str) -> E {
        match result {
            Err(error) => error,
            Ok(_) => panic!("{what}"),
        }
    }

    fn test_timestamp() -> Timestamp {
        ok(Timestamp::parse("2026-09-21T13:45:00Z"))
    }

    fn test_actor() -> ActorRef {
        ok(ActorRef::user("alice"))
    }

    fn parse_capability(key: &str) -> CapabilityId {
        ok(CapabilityId::parse(key))
    }

    fn remote_spec(name: &str, capabilities: Vec<CapabilityId>) -> EnvironmentSpec {
        ok(EnvironmentSpec::new(
            name,
            Locality::Remote,
            capabilities,
            test_actor(),
            test_timestamp(),
        ))
    }

    #[test]
    fn fake_remote_provider_sources_remote_environments_only() {
        let mut provider = FakeRemoteEnvironmentProvider::new();
        assert_eq!(provider.provider_kind(), "flauz-fake-remote");
        assert_eq!(provider.supported_localities(), &[Locality::Remote]);
        assert_eq!(provider.connection_id(), None);
        assert!(provider.environments().is_empty());

        // Local is a named contract error, not a sourced environment.
        let local_spec = ok(EnvironmentSpec::new(
            "Workstation",
            Locality::Local,
            fake_remote_surface_capabilities(),
            test_actor(),
            test_timestamp(),
        ));
        let error = err(
            provider.source_environment(&local_spec),
            "local locality must be rejected",
        );
        assert!(error.to_string().contains("cannot source"));

        // Sourcing the full sandbox topology: descriptor at version 1,
        // remote, provider kind stamped, spec attribution.
        let descriptor = ok(provider.source_environment(&remote_spec(
            "Fake remote sandbox",
            fake_remote_surface_capabilities(),
        )));
        assert_eq!(descriptor.version, 1);
        assert_eq!(descriptor.locality, Locality::Remote);
        assert_eq!(
            descriptor.provider_kind.as_deref(),
            Some("flauz-fake-remote")
        );
        assert_eq!(descriptor.status, EnvironmentStatus::Available);
        assert_eq!(descriptor.connection_id, None);
        assert_eq!(descriptor.created_by, test_actor());
        assert_eq!(
            ok(validate(descriptor.id.as_str())),
            EntityKind::Environment
        );

        // The live surface is reconstructable and agrees with the record.
        let sandbox = provider
            .environment(&descriptor.id)
            .unwrap_or_else(|| panic!("the sourced environment must resolve"));
        assert_eq!(sandbox.descriptor(), descriptor);
        assert_eq!(provider.environments(), vec![descriptor]);

        // With a connection REFERENCE: sourced descriptors carry it —
        // where provider quota belongs. A reference, never material.
        let connection_id = ok(ProviderConnectionId::parse(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQP34",
        ));
        let mut connected =
            FakeRemoteEnvironmentProvider::new().with_connection(connection_id.clone());
        let connected_descriptor = ok(connected.source_environment(&remote_spec(
            "Quota-backed sandbox",
            fake_remote_surface_capabilities(),
        )));
        assert_eq!(connected_descriptor.connection_id, Some(connection_id));
        assert_eq!(
            connected.connection_id(),
            connected_descriptor.connection_id.as_ref()
        );
    }

    #[test]
    fn fake_remote_provider_names_capabilities_outside_the_sandbox_topology() {
        let mut provider = FakeRemoteEnvironmentProvider::new();
        // desktop.gui is an F1 Computer Use surface: NOT offered by the
        // fake remote sandbox — the gap is named, never silently sourced.
        let spec = remote_spec(
            "Impossible sandbox",
            vec![parse_capability("desktop.gui"), parse_capability("gpu")],
        );
        let error = err(
            provider.source_environment(&spec),
            "gaps must be named, never silently sourced",
        );
        let message = error.to_string();
        assert!(
            message.contains("desktop.gui"),
            "the missing key `desktop.gui` must be named"
        );
        assert!(
            message.contains("gpu"),
            "the missing key `gpu` must be named"
        );
        assert!(provider.environments().is_empty());
    }

    #[test]
    fn fake_remote_env_identity_stable_across_serialization() {
        let mut provider = FakeRemoteEnvironmentProvider::new();
        let sandbox_a = ok(provider.source_environment(&remote_spec(
            "Fake remote sandbox A",
            vec![
                parse_capability("filesystem.read"),
                parse_capability("filesystem.write"),
                parse_capability("git"),
                parse_capability("terminal"),
            ],
        )));
        let sandbox_b = ok(provider.source_environment(&remote_spec(
            "Fake remote sandbox B",
            fake_remote_surface_capabilities(),
        )));
        assert_ne!(sandbox_a.id, sandbox_b.id);

        // Descriptor-level round-trip: canonical JSON preserves identity.
        for descriptor in [&sandbox_a, &sandbox_b] {
            let serialized = ok(serde_json::to_string(descriptor));
            let reloaded: EnvironmentDescriptor = ok(serde_json::from_str(&serialized));
            assert_eq!(reloaded.id, descriptor.id);
            assert_eq!(reloaded, *descriptor);
        }

        // Provider-level round-trip: snapshot → serialize → drop →
        // restore keeps the SAME env_ ULID identities.
        let snapshot = provider.snapshot();
        let snapshot_json = ok(serde_json::to_string(&snapshot));
        drop(provider);
        let restored = ok(FakeRemoteEnvironmentProvider::restore(ok(
            serde_json::from_str(&snapshot_json),
        )));
        for descriptor in [&sandbox_a, &sandbox_b] {
            let sandbox = restored.environment(&descriptor.id).unwrap_or_else(|| {
                panic!(
                    "environment {} must survive the serialization round-trip",
                    descriptor.id
                )
            });
            assert_eq!(sandbox.environment_id(), &descriptor.id);
            assert_eq!(sandbox.descriptor(), *descriptor);
        }

        // Environments are listed in canonical ID order.
        let listed = restored.environments();
        assert_eq!(listed.len(), 2);
        assert!(listed.windows(2).all(|pair| pair[0].id < pair[1].id));

        // The snapshot is canonical serialized state: byte-stable across
        // the round-trip, strict on unknown fields.
        let re_serialized = ok(serde_json::to_string(&restored.snapshot()));
        assert_eq!(re_serialized, snapshot_json);
        let unknown_field = snapshot_json.replace("\"v\":1,", "\"v\":1,\"credentials\":null,");
        assert!(
            serde_json::from_str::<FakeRemoteSnapshot>(&unknown_field).is_err(),
            "unknown fields must be rejected"
        );

        // No credential material in the serialized provider state
        // (addendum §3): only canonical references appear.
        for marker in [
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
        ] {
            assert!(
                !snapshot_json.contains(marker),
                "serialized provider state must never contain credential material ({marker:?})"
            );
        }
    }

    /// The shared [`Environment`] conformance, parameterized over BOTH
    /// Wave-2 providers: the local wrapper AND the fake-consistency
    /// remote satisfy the SAME contract (kernel §9; the ARCH-002
    /// conformance pattern applied to ENV-001).
    #[test]
    fn environment_contract_same_for_local_and_remote_providers() {
        let mut local_provider = LocalEnvironmentProvider::new();
        let local_descriptor = ok(local_provider.source_environment(&ok(EnvironmentSpec::new(
            "Local workstation",
            Locality::Local,
            local_surface_capabilities(),
            test_actor(),
            test_timestamp(),
        ))));
        let local = local_provider
            .environment(&local_descriptor.id)
            .unwrap_or_else(|| panic!("the sourced local environment must resolve"));

        let mut remote_provider = FakeRemoteEnvironmentProvider::new();
        let remote_descriptor = ok(remote_provider.source_environment(&remote_spec(
            "Fake remote sandbox",
            fake_remote_surface_capabilities(),
        )));
        let remote = remote_provider
            .environment(&remote_descriptor.id)
            .unwrap_or_else(|| panic!("the sourced remote environment must resolve"));

        let environments: [&dyn Environment; 2] = [&local, &remote];

        for environment in environments {
            // Identity: a canonical env_ ID, opaque, kind-verified.
            let id = environment.environment_id();
            assert_eq!(ok(validate(id.as_str())), EntityKind::Environment);

            // The live surface and the durable descriptor agree.
            let descriptor = environment.descriptor();
            assert_eq!(descriptor.id, *id);
            assert_eq!(descriptor.locality, environment.locality());
            assert_eq!(descriptor.status, environment.status());
            assert_eq!(
                descriptor.capabilities.as_slice(),
                environment.capabilities()
            );
            ok(descriptor.validate());

            // Capability advertisement: valid, sorted, deduplicated,
            // non-empty.
            let capabilities = environment.capabilities();
            assert!(
                !capabilities.is_empty(),
                "environment {id} must advertise capabilities"
            );
            for capability in capabilities {
                ok(capability.validate());
            }
            assert!(
                capabilities.windows(2).all(|pair| pair[0] < pair[1]),
                "environment {id} capabilities must be sorted and deduplicated"
            );

            // The descriptor is canonical serialized state: it
            // round-trips.
            let serialized = ok(serde_json::to_string(&descriptor));
            let reloaded: EnvironmentDescriptor = ok(serde_json::from_str(&serialized));
            assert_eq!(reloaded, descriptor);
        }

        // Each provider advertises its topology honestly: the sourced
        // descriptor is exactly the provider's surface set.
        assert_eq!(local_descriptor.capabilities, local_surface_capabilities());
        assert_eq!(
            remote_descriptor.capabilities,
            fake_remote_surface_capabilities()
        );

        // The two differ exactly where the concept differs: locality.
        assert_eq!(local.locality(), Locality::Local);
        assert_eq!(remote.locality(), Locality::Remote);

        // Provider-level contract over BOTH providers: lowercase kind
        // labels, supported localities, sourced listing in canonical ID
        // order, honest connection references.
        let providers: [&dyn ExecutionProvider; 2] = [&local_provider, &remote_provider];
        for provider in providers {
            let kind = provider.provider_kind();
            assert!(!kind.is_empty(), "provider kind must not be empty");
            assert!(
                kind.chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_lowercase()),
                "provider kind {kind:?} must be a lowercase token"
            );
            assert!(
                !provider.supported_localities().is_empty(),
                "provider {kind} must support at least one locality"
            );
            let sourced = provider.environments();
            assert_eq!(sourced.len(), 1, "provider {kind} sourced one environment");
            assert!(
                provider
                    .supported_localities()
                    .contains(&sourced[0].locality)
            );
            assert_eq!(sourced[0].provider_kind.as_deref(), Some(kind));
        }
        assert_eq!(local_provider.connection_id(), None);
        assert_eq!(remote_provider.connection_id(), None);
    }

    /// A test-local stand-in for the world crate's task event stream (the
    /// Gate A harness pattern, reduced to scalar bones): events on ONE
    /// logical task, strictly increasing `seq`, environment subjects
    /// referenced by opaque canonical ID strings. The `seq` is the
    /// ordering authority; timestamps are fixed constants (kernel §7
    /// test-local helper naming).
    struct TestTaskStream {
        task_id: String,
        events: Vec<TestTaskEvent>,
    }

    struct TestTaskEvent {
        event_id: String,
        seq: u64,
        event_type: String,
        environment_id: String,
    }

    impl TestTaskStream {
        fn new(task_id: &str) -> Self {
            Self {
                task_id: task_id.to_owned(),
                events: Vec::new(),
            }
        }

        /// Appends one event on the task stream; `seq` is strictly
        /// increasing by construction (the no-fork evidence).
        fn append(&mut self, event_type: &str, environment_id: &EnvironmentId) -> u64 {
            let seq = self.events.len() as u64 + 1;
            self.events.push(TestTaskEvent {
                event_id: format!("ev_01J8ZQ5V8K3T2B7N6X4R9DQP{seq:02}"),
                seq,
                event_type: event_type.to_owned(),
                environment_id: environment_id.as_str().to_owned(),
            });
            seq
        }

        fn attach(&mut self, environment_id: &EnvironmentId) -> u64 {
            self.append(event_types::ENVIRONMENT_ATTACHED, environment_id)
        }

        fn detach(&mut self, environment_id: &EnvironmentId) -> u64 {
            self.append(event_types::ENVIRONMENT_DETACHED, environment_id)
        }

        /// The logical task reference: the canonical `task_` ID, opaque,
        /// never forked.
        fn logical_task_ref(&self) -> &str {
            &self.task_id
        }

        fn event_count(&self) -> usize {
            self.events.len()
        }

        fn sequences(&self) -> Vec<u64> {
            self.events.iter().map(|event| event.seq).collect()
        }

        fn event_types(&self) -> Vec<&str> {
            self.events
                .iter()
                .map(|event| event.event_type.as_str())
                .collect()
        }

        fn environment_subjects(&self) -> Vec<&str> {
            self.events
                .iter()
                .map(|event| event.environment_id.as_str())
                .collect()
        }

        /// The canonical `ev_` identity of every recorded event (unique
        /// per event, per the envelope shape).
        fn event_ids(&self) -> Vec<&str> {
            self.events
                .iter()
                .map(|event| event.event_id.as_str())
                .collect()
        }
    }

    /// The cross-environment identity evidence (addendum §6, kernel §6;
    /// the Gate A harness pattern): local → fake-remote → local across a
    /// serialize → drop → reload boundary keeps the SAME logical task —
    /// no fork, environment events recorded on the task stream, stable
    /// `env_` identities on both sides.
    #[test]
    fn task_identity_survives_local_fake_remote_local_switch() {
        // Fixed clock + actor (determinism rule).
        let actor = test_actor();

        // The logical task: a canonical task_ ID (kernel §2 frozen
        // vector), referenced from exec state only as an opaque string.
        let task_ref = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";
        assert_eq!(ok(validate(task_ref)), EntityKind::Task);
        let mut stream = TestTaskStream::new(task_ref);

        // -- Local environment A: the F1 terminal + Git facets.
        let mut local_provider = LocalEnvironmentProvider::new();
        let local_descriptor = ok(local_provider.source_environment(&ok(EnvironmentSpec::new(
            "Local workstation",
            Locality::Local,
            vec![
                parse_capability("filesystem.read"),
                parse_capability("filesystem.write"),
                parse_capability("git"),
                parse_capability("terminal"),
            ],
            actor.clone(),
            test_timestamp(),
        ))));
        let env_a = local_descriptor.id.clone();
        let local_env = local_provider
            .environment(&env_a)
            .unwrap_or_else(|| panic!("the sourced local environment must resolve"));
        assert_eq!(local_env.locality(), Locality::Local);

        // -- Fake-remote sandbox B.
        let mut remote_provider = FakeRemoteEnvironmentProvider::new();
        let remote_descriptor = ok(remote_provider.source_environment(&remote_spec(
            "Fake remote sandbox",
            vec![
                parse_capability("browser.navigation"),
                parse_capability("terminal"),
            ],
        )));
        let env_b = remote_descriptor.id.clone();
        assert_ne!(env_a, env_b);
        assert_eq!(
            remote_provider
                .environment(&env_b)
                .unwrap_or_else(|| panic!("the sourced remote environment must resolve"))
                .locality(),
            Locality::Remote
        );

        // -- Phase 1: attach local, then switch to fake-remote. Every
        //    environment change is an EVENT on the task stream (kernel
        //    §6) — using the exec-registered vocabulary.
        stream.attach(&env_a);
        stream.detach(&env_a);
        stream.attach(&env_b);
        assert_eq!(stream.logical_task_ref(), task_ref);

        // -- Durable state: both descriptors in the exec store (the
        //    platform's state discipline).
        let mut exec = FakeExecStore::new();
        ok(exec.create_environment(local_descriptor.clone()));
        ok(exec.create_environment(remote_descriptor.clone()));

        // -- serialize → drop → reload (the Gate A step-12 pattern).
        let exec_json = ok(serde_json::to_string(&exec.snapshot()));
        let provider_json = ok(serde_json::to_string(&remote_provider.snapshot()));
        drop(exec);
        drop(remote_provider);
        drop(local_provider);
        let exec_reloaded = ok(FakeExecStore::restore(ok(serde_json::from_str(&exec_json))));
        let remote_reloaded = ok(FakeRemoteEnvironmentProvider::restore(ok(
            serde_json::from_str(&provider_json),
        )));

        // -- Identities survive the boundary: the fake-remote provider
        //    restores its env_ ULIDs; the local environment reconstructs
        //    from the reloaded durable descriptor.
        let local_record_reloaded = ok(exec_reloaded.environment(&env_a))
            .unwrap_or_else(|| panic!("environment {env_a} must survive reload"));
        assert_eq!(local_record_reloaded.id, env_a);
        let local_env_reloaded = ok(LocalEnvironment::from_descriptor(local_record_reloaded));
        assert_eq!(local_env_reloaded.environment_id(), &env_a);
        let remote_sandbox_reloaded = remote_reloaded
            .environment(&env_b)
            .unwrap_or_else(|| panic!("environment {env_b} must survive the provider round-trip"));
        assert_eq!(remote_sandbox_reloaded.environment_id(), &env_b);

        // -- Phase 2: back to local, from RELOADED state — the same
        //    canonical identities as before the boundary.
        stream.detach(&env_b);
        stream.attach(&env_a);
        assert_eq!(stream.logical_task_ref(), task_ref);

        // -- Continuation: the stream continues on the same logical task
        //    (seq strictly increases, no fork, one stream).
        stream.detach(&env_a);
        assert_eq!(stream.logical_task_ref(), task_ref);
        assert_eq!(stream.event_count(), 6);
        assert_eq!(stream.sequences(), vec![1, 2, 3, 4, 5, 6]);
        // Every recorded event carries a canonical, unique ev_ identity.
        let event_ids = stream.event_ids();
        for event_id in &event_ids {
            assert_eq!(ok(validate(event_id)), EntityKind::Event);
        }
        let mut unique_event_ids = event_ids.clone();
        unique_event_ids.sort();
        unique_event_ids.dedup();
        assert_eq!(
            unique_event_ids.len(),
            event_ids.len(),
            "event IDs are unique per event"
        );
        assert_eq!(
            stream.event_types(),
            [
                "environment.attached",
                "environment.detached",
                "environment.attached",
                "environment.detached",
                "environment.attached",
                "environment.detached",
            ]
        );
        assert_eq!(
            stream.environment_subjects(),
            [
                env_a.as_str(),
                env_a.as_str(),
                env_b.as_str(),
                env_b.as_str(),
                env_a.as_str(),
                env_a.as_str(),
            ]
        );

        // -- Execution state references environments, never task
        //    identity: one runtime turn on the re-attached local
        //    environment echoes the environment; the request carries no
        //    task reference at all (kernel §6 — environment choice is
        //    execution state, never identity).
        let model_id = ok(ModelId::parse(fake_model_ids::TEXT));
        let request = ok(RuntimeRequest::new(
            model_id.clone(),
            Some(env_a.clone()),
            vec![parse_capability("git"), parse_capability("terminal")],
            "Run the reconciliation step on the local environment",
        ));
        let request_json = ok(serde_json::to_string(&request));
        assert!(
            !request_json.contains("task"),
            "a runtime request never carries task identity"
        );
        let outcome = ok(FakeCodexRuntime::new().execute(&request));
        assert_eq!(outcome.status, RuntimeStatus::Completed);
        assert_eq!(outcome.environment_id.as_ref(), Some(&env_a));
        assert_eq!(outcome.model_id, model_id);
    }

    #[test]
    fn fake_remote_fixtures_round_trip_canonical() {
        // Sandbox descriptor fixtures: canonical round-trip + live
        // reconstruction.
        for case in ["typical.json", "minimal.json"] {
            let content = read_fixture(&format!("fake-remote-sandbox/{case}"));
            let descriptor: EnvironmentDescriptor = ok(serde_json::from_str(&content));
            ok(descriptor.validate());
            let serialized = ok(serde_json::to_string_pretty(&descriptor));
            assert_eq!(
                serialized, content,
                "fixture fake-remote-sandbox/{case} must round-trip byte-for-byte"
            );
            let sandbox = ok(FakeRemoteSandbox::from_descriptor(descriptor));
            assert_eq!(sandbox.locality(), Locality::Remote);
        }

        // Invalid: a fake-remote sandbox with local locality is not a
        // fake remote sandbox.
        let content = read_fixture("fake-remote-sandbox/invalid-local-locality.json");
        let descriptor: EnvironmentDescriptor = ok(serde_json::from_str(&content));
        let error = err(
            FakeRemoteSandbox::from_descriptor(descriptor),
            "local locality must be rejected for a fake remote sandbox",
        );
        assert!(error.to_string().contains("locality"));

        // Snapshot fixtures: canonical round-trip + restore.
        for case in ["typical.json", "minimal.json"] {
            let content = read_fixture(&format!("fake-remote-snapshot/{case}"));
            let snapshot: FakeRemoteSnapshot = ok(serde_json::from_str(&content));
            ok(snapshot.validate());
            let serialized = ok(serde_json::to_string_pretty(&snapshot));
            assert_eq!(
                serialized, content,
                "fixture fake-remote-snapshot/{case} must round-trip byte-for-byte"
            );
            let provider = ok(FakeRemoteEnvironmentProvider::restore(snapshot));
            // The restored provider resolves the fixture's identities.
            for environment in provider.environments() {
                assert!(
                    provider.environment(&environment.id).is_some(),
                    "environment {} must resolve after restore",
                    environment.id
                );
            }
        }

        // Invalid: unsorted (duplicated) environments are rejected.
        let content = read_fixture("fake-remote-snapshot/invalid-unsorted-environments.json");
        let snapshot: FakeRemoteSnapshot = ok(serde_json::from_str(&content));
        assert!(snapshot.validate().is_err());
        assert!(FakeRemoteEnvironmentProvider::restore(snapshot).is_err());

        // Canonical reads are strict: unknown fields are rejected.
        let typical = read_fixture("fake-remote-snapshot/typical.json");
        let unknown_field = typical.replace("\"v\": 1,", "\"v\": 1, \"credentials\": null,");
        assert!(serde_json::from_str::<FakeRemoteSnapshot>(&unknown_field).is_err());

        // The committed fixtures are data too: credential material never
        // appears (addendum §3).
        assert_fixture_directory_credential_free("fake-remote-sandbox");
        assert_fixture_directory_credential_free("fake-remote-snapshot");
    }

    fn fixture_path(relative: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/w2")
            .join(relative)
    }

    fn read_fixture(relative: &str) -> String {
        let content = std::fs::read_to_string(fixture_path(relative))
            .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
        // Canonical fixtures are committed with LF endings; normalize so
        // the byte-for-byte round-trip law is tested against the
        // canonical form, not a platform's line-ending translation.
        content.replace("\r\n", "\n").trim_end().to_owned()
    }

    fn assert_fixture_directory_credential_free(directory: &str) {
        for entry in std::fs::read_dir(fixture_path(directory))
            .unwrap_or_else(|error| panic!("could not list fixture dir {directory}: {error}"))
        {
            let entry = ok(entry);
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
                for marker in [
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
                ] {
                    assert!(
                        !content.contains(marker),
                        "fixture {} must never contain credential material ({marker:?})",
                        path.display()
                    );
                }
            }
        }
    }
}
