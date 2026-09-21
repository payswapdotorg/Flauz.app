//! The execution-state store contract, the store error surface, and the
//! canonical execution snapshot used for serialize → drop → reload
//! round-trips.
//!
//! Mutations follow the kernel §3 version rules: entities are created at
//! version 1, every durable mutation increments by exactly 1, and updates
//! carry the caller's expected version — a mismatch is an
//! [`ExecStoreError::VersionConflict`]; silent overwrite is forbidden.
//!
//! The store covers the four durable ULID entities of this crate
//! (environment records, models, agents, provider connections). Skills are
//! keyed by their namespaced [`SkillId`](crate::SkillId) and versioned
//! semantically (kernel §2), so they are requirement-representation values
//! rather than registry rows and do not live in the store.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::connection::ProviderConnection;
use crate::environment::EnvironmentDescriptor;
use crate::ids::{AgentId, EnvironmentId, ModelId, ProviderConnectionId};
use crate::model::Model;

/// Errors returned by execution-state store operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecStoreError {
    /// An entity with the same canonical ID already exists.
    Duplicate {
        /// The conflicting canonical ID.
        id: String,
    },
    /// No entity exists with the given canonical ID.
    NotFound {
        /// The missing canonical ID.
        id: String,
    },
    /// The caller's expected version did not match the stored version
    /// (kernel §3: optimistic concurrency; silent overwrite is forbidden).
    VersionConflict {
        /// The conflicting canonical ID.
        id: String,
        /// The version the caller expected.
        expected_version: u64,
        /// The version actually stored.
        actual_version: u64,
    },
    /// The operation violates a canonical rule (bounds, ordering, state).
    Invalid {
        /// The reason the operation was rejected.
        reason: String,
    },
}

impl fmt::Display for ExecStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate { id } => write!(formatter, "entity {id} already exists"),
            Self::NotFound { id } => write!(formatter, "entity {id} does not exist"),
            Self::VersionConflict {
                id,
                expected_version,
                actual_version,
            } => write!(
                formatter,
                "version conflict on {id}: expected {expected_version}, stored {actual_version}"
            ),
            Self::Invalid { reason } => write!(formatter, "invalid operation: {reason}"),
        }
    }
}

impl Error for ExecStoreError {}

impl From<crate::ExecError> for ExecStoreError {
    fn from(error: crate::ExecError) -> Self {
        Self::Invalid {
            reason: error.to_string(),
        }
    }
}

/// The canonical serialization of the entire execution state: environment
/// records, models, agents and provider connections. Used by fakes and
/// real stores alike for serialize → drop → reload round-trips; the state
/// survives with equality (kernel round-trip law).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecSnapshot {
    /// Contract schema version (`"v": 1`).
    pub v: crate::ContractVersion,
    /// All environment records, keyed by canonical ID.
    pub environments: BTreeMap<EnvironmentId, EnvironmentDescriptor>,
    /// All models, keyed by canonical ID.
    pub models: BTreeMap<ModelId, Model>,
    /// All agents, keyed by canonical ID.
    pub agents: BTreeMap<AgentId, Agent>,
    /// All provider connections, keyed by canonical ID.
    pub connections: BTreeMap<ProviderConnectionId, ProviderConnection>,
}

impl ExecSnapshot {
    /// The empty snapshot.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            v: crate::ContractVersion,
            environments: BTreeMap::new(),
            models: BTreeMap::new(),
            agents: BTreeMap::new(),
            connections: BTreeMap::new(),
        }
    }

    /// Validates the snapshot's structural invariants: map keys match the
    /// entity IDs they store, and every entity passes canonical
    /// validation.
    pub fn validate(&self) -> Result<(), ExecStoreError> {
        for (id, environment) in &self.environments {
            check_key(id.as_str(), environment.id.as_str(), environment.validate())?;
        }
        for (id, model) in &self.models {
            check_key(id.as_str(), model.id.as_str(), model.validate())?;
        }
        for (id, agent) in &self.agents {
            check_key(id.as_str(), agent.id.as_str(), agent.validate())?;
        }
        for (id, connection) in &self.connections {
            check_key(id.as_str(), connection.id.as_str(), connection.validate())?;
        }
        Ok(())
    }
}

fn check_key(
    key: &str,
    entity_id: &str,
    validation: Result<(), crate::ExecError>,
) -> Result<(), ExecStoreError> {
    if key != entity_id {
        return Err(ExecStoreError::Invalid {
            reason: format!("map key {key} does not match entity id {entity_id}"),
        });
    }
    validation.map_err(ExecStoreError::from)
}

/// The execution-state store contract for the durable ULID entities of
/// this crate.
///
/// All timestamps are supplied by callers; the store never reads the wall
/// clock. Updates follow optimistic concurrency: the passed entity's
/// `version` is the expected version and the store returns the entity with
/// its version incremented by exactly one.
pub trait ExecStore {
    /// Stores a new environment record (created at version 1).
    fn create_environment(
        &mut self,
        environment: EnvironmentDescriptor,
    ) -> Result<EnvironmentDescriptor, ExecStoreError>;
    /// Looks up an environment record.
    fn environment(
        &self,
        id: &EnvironmentId,
    ) -> Result<Option<EnvironmentDescriptor>, ExecStoreError>;
    /// Updates an environment record; the passed `version` is the expected
    /// version, and the stored record is returned with its version
    /// incremented.
    fn update_environment(
        &mut self,
        environment: EnvironmentDescriptor,
    ) -> Result<EnvironmentDescriptor, ExecStoreError>;

    /// Stores a new model (created at version 1).
    fn create_model(&mut self, model: Model) -> Result<Model, ExecStoreError>;
    /// Looks up a model.
    fn model(&self, id: &ModelId) -> Result<Option<Model>, ExecStoreError>;
    /// Updates a model (optimistic concurrency on the passed version).
    fn update_model(&mut self, model: Model) -> Result<Model, ExecStoreError>;

    /// Stores a new agent (created at version 1).
    fn create_agent(&mut self, agent: Agent) -> Result<Agent, ExecStoreError>;
    /// Looks up an agent.
    fn agent(&self, id: &AgentId) -> Result<Option<Agent>, ExecStoreError>;
    /// Updates an agent (optimistic concurrency on the passed version).
    fn update_agent(&mut self, agent: Agent) -> Result<Agent, ExecStoreError>;

    /// Stores a new provider connection (created at version 1).
    fn create_connection(
        &mut self,
        connection: ProviderConnection,
    ) -> Result<ProviderConnection, ExecStoreError>;
    /// Looks up a provider connection.
    fn connection(
        &self,
        id: &ProviderConnectionId,
    ) -> Result<Option<ProviderConnection>, ExecStoreError>;
    /// Updates a provider connection (optimistic concurrency on the passed
    /// version).
    fn update_connection(
        &mut self,
        connection: ProviderConnection,
    ) -> Result<ProviderConnection, ExecStoreError>;

    /// Captures the entire execution state as a canonical snapshot.
    fn snapshot(&self) -> ExecSnapshot;
    /// Rebuilds a store from a snapshot (serialize → drop → reload). The
    /// snapshot must satisfy the canonical invariants.
    fn restore(snapshot: ExecSnapshot) -> Result<Self, ExecStoreError>
    where
        Self: Sized;
}
