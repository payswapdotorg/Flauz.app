//! In-memory fakes: the deterministic conformance surface (kernel §7).
//!
//! Every fake is named `Fake*`, holds plain in-memory data, performs no
//! I/O and reads no wall clock: all timestamps are fixed constants and the
//! only entropy source is canonical ID generation (the private `ulid`
//! module), used solely where a caller has not supplied an identity.
//! Entities exposed by fake constructors carry fixed canonical IDs, so the
//! fakes are fully deterministic. Use them for tests, the F2 integration
//! gate, and as the reference semantics for real implementations.
//!
//! The conformance pairs the kernel §9 tests exercise:
//!
//! | Contract | Fakes |
//! |---|---|
//! | [`AgentRuntime`] | [`FakeCodexRuntime`] and [`FakeNonCodexRuntime`] — one fake Codex (app-server) runtime and one fake non-Codex (direct-model) runtime satisfying the SAME contract |
//! | [`CodexServerHandle`](crate::runtime_codex::CodexServerHandle) | [`FakeCodexServerHandle`] — the deterministic app-server boundary handle the RT-001 Codex adapter delegates turns to |
//! | [`Environment`] | [`FakeLocalEnvironment`] and [`FakeRemoteEnvironment`] — a local and a remote environment satisfying the SAME contract; the Wave-2 provider family (ENV-001) adds [`LocalEnvironment`](crate::environment_local::LocalEnvironment) and [`FakeRemoteSandbox`] |
//! | [`ModelProvider`] | [`FakeModelProvider`] with two fake models offering different capability sets |
//! | [`ExecutionProvider`] | [`FakeExecutionProvider`] sourcing both localities; the Wave-2 provider family (ENV-001) adds [`LocalEnvironmentProvider`](crate::environment_local::LocalEnvironmentProvider) — the local wrapper of the four F1 surfaces — and [`FakeRemoteEnvironmentProvider`] — the fake-consistency remote, the template every later real remote provider copies (re-exported below for downstream use) |
//! | [`ExecStore`] | [`FakeExecStore`] |

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::agent::Agent;
use crate::capability::CapabilityId;
use crate::connection::{ProviderConnection, SecretRef};
use crate::environment::{
    Environment, EnvironmentDescriptor, EnvironmentSpec, EnvironmentStatus, ExecutionProvider,
    Locality,
};
use crate::ids::{AgentId, EnvironmentId, ModelId, ProviderConnectionId};
use crate::model::{Model, ModelProvider};
use crate::refs::ActorRef;
use crate::runtime::{AgentRuntime, RuntimeOutcome, RuntimeRequest};
use crate::store::{ExecSnapshot, ExecStore, ExecStoreError};
use crate::time::Timestamp;
use crate::{ExecError, MAX_ENVIRONMENTS_PER_PROVIDER, MAX_MODELS_PER_PROVIDER};

// The fake-consistency remote provider family (ENV-001, addendum §2),
// re-exported so downstream consumers — the Lead's F2 gate extension and
// later provider waves — can source it from the fakes entry point. The
// local provider is a real provider (not a fake) and stays at the crate
// root: `flauz_exec::LocalEnvironmentProvider`.
pub use crate::environment_fake_remote::{
    FakeRemoteEnvironmentProvider, FakeRemoteSandbox, FakeRemoteSnapshot,
};

/// The fixed creation timestamp used by every fake: 2026-09-21T13:45:00Z.
/// Fakes never read the wall clock (kernel §7 determinism rule).
const FAKE_EPOCH_SECONDS: i64 = 1_789_998_300;

/// Panics on the impossible: a constant that must always validate.
macro_rules! constant {
    ($result:expr, $what:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => panic!("constant {} must always validate: {error}", $what),
        }
    };
}

fn fake_timestamp() -> Timestamp {
    constant!(
        Timestamp::from_unix_seconds(FAKE_EPOCH_SECONDS),
        "fake epoch"
    )
}

fn fake_actor() -> ActorRef {
    constant!(ActorRef::system("flauz-fake"), "fake actor")
}

fn parse_capability(key: &str) -> CapabilityId {
    constant!(CapabilityId::parse(key), "capability key")
}

/// The fake official Codex app-server runtime: ONE runtime among several,
/// never the universal registry (constitution). Deterministic, no I/O.
///
/// Advertises a terminal-and-filesystem tool surface. The runtime kind
/// label is `codex-app-server`.
#[derive(Debug, Clone)]
pub struct FakeCodexRuntime {
    capabilities: Vec<CapabilityId>,
}

impl FakeCodexRuntime {
    /// A fake Codex app-server runtime advertising terminal, filesystem
    /// and git capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self {
            capabilities: vec![
                parse_capability("filesystem.read"),
                parse_capability("filesystem.write"),
                parse_capability("git"),
                parse_capability("terminal"),
            ],
        }
    }
}

impl Default for FakeCodexRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRuntime for FakeCodexRuntime {
    fn runtime_kind(&self) -> &str {
        "codex-app-server"
    }

    fn capabilities(&self) -> &[CapabilityId] {
        &self.capabilities
    }

    fn execute(&self, request: &RuntimeRequest) -> Result<RuntimeOutcome, ExecError> {
        execute_turn(self.runtime_kind(), &self.capabilities, request)
    }
}

/// The fake non-Codex runtime: a Flauz direct-model runtime satisfying the
/// SAME [`AgentRuntime`] contract as [`FakeCodexRuntime`] — provider
/// neutrality proven by the shared conformance test. Deterministic, no
/// I/O.
///
/// Advertises a browser-and-web tool surface. The runtime kind label is
/// `flauz-direct`.
#[derive(Debug, Clone)]
pub struct FakeNonCodexRuntime {
    capabilities: Vec<CapabilityId>,
}

impl FakeNonCodexRuntime {
    /// A fake direct-model runtime advertising browser and web-search
    /// capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self {
            capabilities: vec![
                parse_capability("browser.input"),
                parse_capability("browser.navigation"),
                parse_capability("web.search"),
            ],
        }
    }
}

impl Default for FakeNonCodexRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRuntime for FakeNonCodexRuntime {
    fn runtime_kind(&self) -> &str {
        "flauz-direct"
    }

    fn capabilities(&self) -> &[CapabilityId] {
        &self.capabilities
    }

    fn execute(&self, request: &RuntimeRequest) -> Result<RuntimeOutcome, ExecError> {
        execute_turn(self.runtime_kind(), &self.capabilities, request)
    }
}

/// The shared, provider-neutral turn execution used by BOTH fake
/// runtimes: same contract, same behavior shape, different advertised
/// capabilities and kind labels.
fn execute_turn(
    runtime_kind: &str,
    advertised: &[CapabilityId],
    request: &RuntimeRequest,
) -> Result<RuntimeOutcome, ExecError> {
    request.validate()?;
    let missing = crate::runtime::missing_capabilities(advertised, &request.required_capabilities);
    if missing.is_empty() {
        RuntimeOutcome::completed(
            runtime_kind,
            request.model_id.clone(),
            request.environment_id.clone(),
            &format!("{runtime_kind} completed one orchestration step"),
        )
    } else {
        RuntimeOutcome::capability_gap(
            runtime_kind,
            request.model_id.clone(),
            request.environment_id.clone(),
            missing,
            &format!("{runtime_kind} is missing required capabilities"),
        )
    }
}

/// The fake local environment: execution happens on the user's own
/// machine. Deterministic, no I/O; fixed canonical identity by default.
///
/// Advertises terminal, filesystem and git capabilities — the same
/// [`Environment`] contract as [`FakeRemoteEnvironment`].
#[derive(Debug, Clone)]
pub struct FakeLocalEnvironment {
    descriptor: EnvironmentDescriptor,
}

impl FakeLocalEnvironment {
    /// The canonical identity of the default fake local environment.
    pub const DEFAULT_ID: &'static str = "env_01J8ZQ5V8K3T2B7N6X4R9DQPT6";

    /// A fake local environment with the given identity and name,
    /// advertising terminal, filesystem and git capabilities.
    pub fn new(id: EnvironmentId, name: &str) -> Result<Self, ExecError> {
        let descriptor = EnvironmentDescriptor::new(
            id,
            name,
            Locality::Local,
            Some("flauz-fake-local"),
            vec![
                parse_capability("filesystem.read"),
                parse_capability("filesystem.write"),
                parse_capability("git"),
                parse_capability("terminal"),
            ],
            EnvironmentStatus::Available,
            None,
            fake_actor(),
            fake_timestamp(),
        )?;
        Ok(Self { descriptor })
    }

    /// A fake local environment with the fixed default identity
    /// "Fake local workstation".
    pub fn with_default_id() -> Result<Self, ExecError> {
        Self::new(
            constant!(EnvironmentId::parse(Self::DEFAULT_ID), "local fake id"),
            "Fake local workstation",
        )
    }
}

impl Environment for FakeLocalEnvironment {
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

/// The fake remote environment: execution happens on external
/// infrastructure. Deterministic, no I/O; fixed canonical identity by
/// default.
///
/// Advertises browser, ports and snapshot capabilities — the same
/// [`Environment`] contract as [`FakeLocalEnvironment`].
#[derive(Debug, Clone)]
pub struct FakeRemoteEnvironment {
    descriptor: EnvironmentDescriptor,
}

impl FakeRemoteEnvironment {
    /// The canonical identity of the default fake remote environment.
    pub const DEFAULT_ID: &'static str = "env_01J8ZQ5V8K3T2B7N6X4R9DQPW9";

    /// A fake remote environment with the given identity and name,
    /// advertising browser, ports and snapshot capabilities.
    pub fn new(id: EnvironmentId, name: &str) -> Result<Self, ExecError> {
        let descriptor = EnvironmentDescriptor::new(
            id,
            name,
            Locality::Remote,
            Some("flauz-fake-remote"),
            vec![
                parse_capability("browser.input"),
                parse_capability("browser.navigation"),
                parse_capability("ports"),
                parse_capability("snapshots"),
            ],
            EnvironmentStatus::Available,
            None,
            fake_actor(),
            fake_timestamp(),
        )?;
        Ok(Self { descriptor })
    }

    /// A fake remote environment with the fixed default identity
    /// "Fake remote sandbox".
    pub fn with_default_id() -> Result<Self, ExecError> {
        Self::new(
            constant!(EnvironmentId::parse(Self::DEFAULT_ID), "remote fake id"),
            "Fake remote sandbox",
        )
    }
}

impl Environment for FakeRemoteEnvironment {
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

/// The canonical identities of the two fake models exposed by
/// [`FakeModelProvider`].
pub mod fake_model_ids {
    /// The fake text-only model.
    pub const TEXT: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPX0";
    /// The fake vision model.
    pub const VISION: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPY1";
}

/// The fake model provider: sources two fake models with **different
/// capability sets** (kernel: at least two fake models). Deterministic,
/// no I/O; the models carry fixed canonical identities.
///
/// The provider kind label is `flauz-fake`.
#[derive(Debug, Clone)]
pub struct FakeModelProvider {
    models: Vec<Model>,
    capabilities: Vec<CapabilityId>,
}

impl FakeModelProvider {
    /// A fake provider sourcing a text-only model and a vision model.
    pub fn new() -> Result<Self, ExecError> {
        let text = Model::new(
            constant!(ModelId::parse(fake_model_ids::TEXT), "text fake model id"),
            "Fake Text Model",
            "flauz-fake",
            Vec::new(),
            Some("Deterministic fake model with no extra capabilities"),
            fake_actor(),
            fake_timestamp(),
        )?;
        let vision = Model::new(
            constant!(
                ModelId::parse(fake_model_ids::VISION),
                "vision fake model id"
            ),
            "Fake Vision Model",
            "flauz-fake",
            vec![parse_capability("image.input"), parse_capability("vision")],
            Some("Deterministic fake model with vision capabilities"),
            fake_actor(),
            fake_timestamp(),
        )?;
        Ok(Self {
            capabilities: vec![parse_capability("image.input"), parse_capability("vision")],
            models: vec![text, vision],
        })
    }

    /// Builds a fake provider from an explicit model list. The list must
    /// contain at least two models with different capability sets.
    pub fn from_models(models: Vec<Model>) -> Result<Self, ExecError> {
        if models.len() < 2 {
            return Err(ExecError::invalid(
                "a fake model provider must source at least two models",
            ));
        }
        if models.len() > MAX_MODELS_PER_PROVIDER {
            return Err(ExecError::invalid(format!(
                "a model provider exposes at most {MAX_MODELS_PER_PROVIDER} models"
            )));
        }
        let offers_different_sets = models.windows(2).any(|pair| {
            let [left, right] = pair else {
                return false;
            };
            left.capabilities != right.capabilities
        });
        if !offers_different_sets {
            return Err(ExecError::invalid(
                "the fake models must offer different capability sets",
            ));
        }
        let mut models = models;
        for model in &models {
            model.validate()?;
        }
        models.sort_by(|left, right| left.id.cmp(&right.id));

        let mut capabilities: Vec<CapabilityId> = models
            .iter()
            .flat_map(|model| model.capabilities.iter().cloned())
            .collect();
        capabilities.sort();
        capabilities.dedup();
        Ok(Self {
            models,
            capabilities,
        })
    }

    /// The fake models, in canonical ID order.
    #[must_use]
    pub fn fake_models(&self) -> &[Model] {
        &self.models
    }
}

impl ModelProvider for FakeModelProvider {
    fn provider_kind(&self) -> &str {
        "flauz-fake"
    }

    fn capabilities(&self) -> &[CapabilityId] {
        &self.capabilities
    }

    fn connection_id(&self) -> Option<&ProviderConnectionId> {
        None
    }

    fn models(&self) -> &[Model] {
        &self.models
    }

    fn model(&self, id: &ModelId) -> Option<Model> {
        self.models.iter().find(|model| &model.id == id).cloned()
    }
}

/// The fake execution provider: sources environments for both localities
/// from [`EnvironmentSpec`]s, exercising the
/// [`ExecutionProvider`] contract. The provider kind label is
/// `flauz-fake`.
#[derive(Debug, Clone, Default)]
pub struct FakeExecutionProvider {
    environments: Vec<EnvironmentDescriptor>,
    connection_id: Option<ProviderConnectionId>,
}

impl FakeExecutionProvider {
    /// A fake execution provider sourcing local and remote environments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches a user-owned connection through which this provider is
    /// accessed (where provider quota belongs).
    #[must_use]
    pub fn with_connection(mut self, connection_id: ProviderConnectionId) -> Self {
        self.connection_id = Some(connection_id);
        self
    }
}

impl ExecutionProvider for FakeExecutionProvider {
    fn provider_kind(&self) -> &str {
        "flauz-fake"
    }

    fn supported_localities(&self) -> &[Locality] {
        &[Locality::Local, Locality::Remote]
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
                "provider `flauz-fake` cannot source {:?} environments",
                spec.locality
            )));
        }
        if self.environments.len() >= MAX_ENVIRONMENTS_PER_PROVIDER {
            return Err(ExecError::invalid(format!(
                "an execution provider exposes at most {MAX_ENVIRONMENTS_PER_PROVIDER} environments"
            )));
        }
        // ID generation is the crate's only entropy source (kernel §7).
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
        self.environments.push(descriptor.clone());
        Ok(descriptor)
    }

    fn environments(&self) -> Vec<EnvironmentDescriptor> {
        let mut environments = self.environments.clone();
        environments.sort_by(|left, right| left.id.cmp(&right.id));
        environments
    }
}

/// The fake execution store: an in-memory [`ExecStore`] over the durable
/// ULID entities. Deterministic; identical operation sequences produce
/// identical logical state (callers supply all identities, so the store
/// never generates IDs itself).
#[derive(Debug, Default, Clone)]
pub struct FakeExecStore {
    environments: BTreeMap<EnvironmentId, EnvironmentDescriptor>,
    models: BTreeMap<ModelId, Model>,
    agents: BTreeMap<AgentId, Agent>,
    connections: BTreeMap<ProviderConnectionId, ProviderConnection>,
}

/// Internal helpers shared by the fake's create/update paths.
mod entity {
    use super::*;

    /// The operations every durable entity supports in the fake.
    pub(super) trait ExecEntity {
        fn entity_id(&self) -> &str;
        fn entity_version(&self) -> u64;
        fn set_entity_version(&mut self, version: u64);
        fn validate_entity(&self) -> Result<(), crate::ExecError>;
    }

    impl ExecEntity for EnvironmentDescriptor {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), crate::ExecError> {
            self.validate()
        }
    }

    impl ExecEntity for Model {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), crate::ExecError> {
            self.validate()
        }
    }

    impl ExecEntity for Agent {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), crate::ExecError> {
            self.validate()
        }
    }

    impl ExecEntity for ProviderConnection {
        fn entity_id(&self) -> &str {
            self.id.as_str()
        }
        fn entity_version(&self) -> u64 {
            self.version
        }
        fn set_entity_version(&mut self, version: u64) {
            self.version = version;
        }
        fn validate_entity(&self) -> Result<(), crate::ExecError> {
            self.validate()
        }
    }

    /// Creates an entity: must be valid, at version 1, and not present.
    pub(super) fn create<E: ExecEntity + Clone, K: Ord + Clone>(
        map: &mut BTreeMap<K, E>,
        key: K,
        entity: E,
    ) -> Result<E, ExecStoreError> {
        entity.validate_entity()?;
        if map.contains_key(&key) {
            return Err(ExecStoreError::Duplicate {
                id: entity.entity_id().to_owned(),
            });
        }
        if entity.entity_version() != 1 {
            return Err(ExecStoreError::Invalid {
                reason: format!("entity {} must be created at version 1", entity.entity_id()),
            });
        }
        map.insert(key, entity.clone());
        Ok(entity)
    }

    /// Updates an entity: the passed version is the expected version; on
    /// mismatch the store returns a version conflict (kernel §3).
    pub(super) fn update<E: ExecEntity + Clone, K: Ord + Clone>(
        map: &mut BTreeMap<K, E>,
        key: &K,
        mut entity: E,
    ) -> Result<E, ExecStoreError> {
        entity.validate_entity()?;
        let Some(stored) = map.get(key) else {
            return Err(ExecStoreError::NotFound {
                id: entity.entity_id().to_owned(),
            });
        };
        if stored.entity_version() != entity.entity_version() {
            return Err(ExecStoreError::VersionConflict {
                id: entity.entity_id().to_owned(),
                expected_version: entity.entity_version(),
                actual_version: stored.entity_version(),
            });
        }
        entity.set_entity_version(stored.entity_version() + 1);
        map.insert(key.clone(), entity.clone());
        Ok(entity)
    }
}

impl FakeExecStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ExecStore for FakeExecStore {
    fn create_environment(
        &mut self,
        environment: EnvironmentDescriptor,
    ) -> Result<EnvironmentDescriptor, ExecStoreError> {
        entity::create(&mut self.environments, environment.id.clone(), environment)
    }

    fn environment(
        &self,
        id: &EnvironmentId,
    ) -> Result<Option<EnvironmentDescriptor>, ExecStoreError> {
        Ok(self.environments.get(id).cloned())
    }

    fn update_environment(
        &mut self,
        environment: EnvironmentDescriptor,
    ) -> Result<EnvironmentDescriptor, ExecStoreError> {
        entity::update(&mut self.environments, &environment.id.clone(), environment)
    }

    fn create_model(&mut self, model: Model) -> Result<Model, ExecStoreError> {
        entity::create(&mut self.models, model.id.clone(), model)
    }

    fn model(&self, id: &ModelId) -> Result<Option<Model>, ExecStoreError> {
        Ok(self.models.get(id).cloned())
    }

    fn update_model(&mut self, model: Model) -> Result<Model, ExecStoreError> {
        entity::update(&mut self.models, &model.id.clone(), model)
    }

    fn create_agent(&mut self, agent: Agent) -> Result<Agent, ExecStoreError> {
        entity::create(&mut self.agents, agent.id.clone(), agent)
    }

    fn agent(&self, id: &AgentId) -> Result<Option<Agent>, ExecStoreError> {
        Ok(self.agents.get(id).cloned())
    }

    fn update_agent(&mut self, agent: Agent) -> Result<Agent, ExecStoreError> {
        entity::update(&mut self.agents, &agent.id.clone(), agent)
    }

    fn create_connection(
        &mut self,
        connection: ProviderConnection,
    ) -> Result<ProviderConnection, ExecStoreError> {
        entity::create(&mut self.connections, connection.id.clone(), connection)
    }

    fn connection(
        &self,
        id: &ProviderConnectionId,
    ) -> Result<Option<ProviderConnection>, ExecStoreError> {
        Ok(self.connections.get(id).cloned())
    }

    fn update_connection(
        &mut self,
        connection: ProviderConnection,
    ) -> Result<ProviderConnection, ExecStoreError> {
        entity::update(&mut self.connections, &connection.id.clone(), connection)
    }

    fn snapshot(&self) -> ExecSnapshot {
        ExecSnapshot {
            v: crate::ContractVersion,
            environments: self.environments.clone(),
            models: self.models.clone(),
            agents: self.agents.clone(),
            connections: self.connections.clone(),
        }
    }

    fn restore(snapshot: ExecSnapshot) -> Result<Self, ExecStoreError> {
        snapshot.validate()?;
        Ok(Self {
            environments: snapshot.environments,
            models: snapshot.models,
            agents: snapshot.agents,
            connections: snapshot.connections,
        })
    }
}

/// The fixed opaque secret reference used by the fake provider
/// connection: an opaque `flausec_...` token, never credential material.
pub const FAKE_SECRET_REF: &str = "flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34";

/// The fixed summary the fake app-server handle reports for every served
/// turn: bounded, deterministic, never credential material.
pub const FAKE_CODEX_TURN_SUMMARY: &str = "fake app-server completed one orchestration turn";

/// A fake handle to the official Codex app-server boundary (RT-001): the
/// deterministic conformance surface for
/// [`CodexAppServerRuntime`](crate::runtime_codex::CodexAppServerRuntime).
/// Records every turn it served, in order — the adapter's tests and the
/// F2 gate assert that gap turns NEVER reach the server. Deterministic,
/// in-memory, no I/O.
#[derive(Debug, Default)]
pub struct FakeCodexServerHandle {
    served_turns: Mutex<Vec<crate::runtime_codex::CodexServerTurn>>,
}

impl FakeCodexServerHandle {
    /// An empty fake handle that has served no turns.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The turns served so far, in serve order.
    pub fn served_turns(&self) -> Vec<crate::runtime_codex::CodexServerTurn> {
        self.served_turns
            .lock()
            .map(|turns| turns.clone())
            .unwrap_or_default()
    }
}

impl crate::runtime_codex::CodexServerHandle for FakeCodexServerHandle {
    fn turn(
        &self,
        turn: &crate::runtime_codex::CodexServerTurn,
    ) -> Result<crate::runtime_codex::CodexServerTurnResult, crate::ExecError> {
        turn.validate()?;
        let mut served = self
            .served_turns
            .lock()
            .map_err(|_| crate::ExecError::invalid("the fake app-server handle is poisoned"))?;
        served.push(turn.clone());
        crate::runtime_codex::CodexServerTurnResult::completed(FAKE_CODEX_TURN_SUMMARY)
    }
}

/// Builds a fake provider connection (for tests and the F2 gate): provider
/// kind `e2b`, label "Fake sandbox account", the fixed opaque secret
/// reference, and the canonical identity
/// `conn_01J8ZQ5V8K3T2B7N6X4R9DQP34`.
pub fn fake_provider_connection() -> Result<ProviderConnection, ExecError> {
    ProviderConnection::new(
        constant!(
            ProviderConnectionId::parse("conn_01J8ZQ5V8K3T2B7N6X4R9DQP34"),
            "fake connection id"
        ),
        "e2b",
        "Fake sandbox account",
        SecretRef::parse(FAKE_SECRET_REF)?,
        fake_actor(),
        fake_timestamp(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::RuntimeStatus;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn fake_model_provider_sources_two_models_with_different_capabilities() {
        let provider = ok(FakeModelProvider::new());
        let models = provider.fake_models();
        assert_eq!(models.len(), 2);
        assert_ne!(models[0].capabilities, models[1].capabilities);
        assert_eq!(models[0].provider_kind, "flauz-fake");
        assert_eq!(
            ok(ModelId::parse(fake_model_ids::TEXT)).as_str(),
            models[0].id.as_str()
        );
        assert_eq!(
            ok(ModelId::parse(fake_model_ids::VISION)).as_str(),
            models[1].id.as_str()
        );
        // The provider advertisement is the union of model capabilities.
        assert_eq!(provider.capabilities().len(), 2);
        assert_eq!(provider.capabilities()[0].as_str(), "image.input");
        assert_eq!(provider.capabilities()[1].as_str(), "vision");
        assert_eq!(provider.connection_id(), None);
        let lookup = ok(ModelId::parse(fake_model_ids::VISION));
        assert_eq!(
            provider.model(&lookup).map(|model| model.name),
            Some("Fake Vision Model".to_owned())
        );
    }

    #[test]
    fn fake_model_provider_requires_two_different_models() {
        let one = ok(Model::new(
            ok(ModelId::parse(fake_model_ids::TEXT)),
            "A",
            "flauz-fake",
            Vec::new(),
            None,
            fake_actor(),
            fake_timestamp(),
        ));
        assert!(FakeModelProvider::from_models(vec![one.clone()]).is_err());
        let twin = ok(Model::new(
            ok(ModelId::parse(fake_model_ids::VISION)),
            "B",
            "flauz-fake",
            Vec::new(),
            None,
            fake_actor(),
            fake_timestamp(),
        ));
        assert!(FakeModelProvider::from_models(vec![one, twin]).is_err());
    }

    #[test]
    fn fake_runtimes_execute_the_same_contract_shape() {
        let codex = FakeCodexRuntime::new();
        let direct = FakeNonCodexRuntime::new();
        let model_id = ok(ModelId::parse(fake_model_ids::TEXT));
        let request = ok(RuntimeRequest::new(
            model_id.clone(),
            None,
            vec![parse_capability("terminal")],
            "Run the tests",
        ));
        let outcome = ok(codex.execute(&request));
        assert_eq!(outcome.status, RuntimeStatus::Completed);
        assert_eq!(outcome.runtime_kind, "codex-app-server");
        assert_eq!(outcome.model_id, model_id);
        let gap = ok(direct.execute(&request));
        assert_eq!(gap.status, RuntimeStatus::CapabilityGap);
        assert_eq!(gap.missing_capabilities, vec![parse_capability("terminal")]);
        ok(gap.validate());
    }

    #[test]
    fn fake_environments_carry_their_locality() {
        let local = ok(FakeLocalEnvironment::with_default_id());
        let remote = ok(FakeRemoteEnvironment::with_default_id());
        assert_eq!(local.locality(), Locality::Local);
        assert_eq!(remote.locality(), Locality::Remote);
        assert_eq!(
            local.environment_id().as_str(),
            FakeLocalEnvironment::DEFAULT_ID
        );
        assert_eq!(
            remote.environment_id().as_str(),
            FakeRemoteEnvironment::DEFAULT_ID
        );
        ok(local.descriptor().validate());
        ok(remote.descriptor().validate());
    }

    #[test]
    fn fake_execution_provider_sources_both_localities() {
        let mut provider = FakeExecutionProvider::new();
        let spec = ok(EnvironmentSpec::new(
            "Sandbox",
            Locality::Remote,
            vec![parse_capability("ports")],
            ok(ActorRef::user("alice")),
            fake_timestamp(),
        ));
        let descriptor = ok(provider.source_environment(&spec));
        assert_eq!(descriptor.locality, Locality::Remote);
        assert_eq!(descriptor.provider_kind.as_deref(), Some("flauz-fake"));
        assert_eq!(descriptor.version, 1);
        assert_eq!(provider.environments().len(), 1);
    }

    #[test]
    fn fake_connection_holds_only_references() {
        let connection = ok(fake_provider_connection());
        let serialized = ok(serde_json::to_string(&connection));
        assert!(serialized.contains("flausec_"));
        ok(connection.validate());
    }

    #[test]
    fn fake_codex_server_handle_serves_and_records_turns() {
        use crate::runtime_codex::CodexServerHandle;
        let handle = FakeCodexServerHandle::new();
        assert!(handle.served_turns().is_empty());
        let turn = ok(crate::runtime_codex::CodexServerTurn::new(
            ok(ModelId::parse(fake_model_ids::TEXT)),
            "Run the test suite and report failures",
        ));
        let result = ok(handle.turn(&turn));
        assert_eq!(result.summary, FAKE_CODEX_TURN_SUMMARY);
        let served = handle.served_turns();
        assert_eq!(served.len(), 1);
        assert_eq!(served[0], turn);
        ok(result.validate());
    }
}
