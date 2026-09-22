//! The model/provider registry (MOD-001): the durable home of provider
//! registrations, the models they source with capability metadata, and
//! the provider connections through which user accounts are accessed.
//!
//! The registry is the F7 bring-your-own-provider foundation:
//!
//! - **Registrations** ([`ProviderRegistration`]) mirror the frozen
//!   [`ModelProvider`](crate::ModelProvider) contract's data: a provider
//!   kind label, the models it sources (each carrying its capability
//!   metadata from the frozen [`CapabilityId`](crate::CapabilityId)
//!   vocabulary), and the user-owned
//!   [`ProviderConnection`](crate::ProviderConnection) it draws on.
//! - **Availability is honest by construction** (the three states:
//!   connected / configured-not-connected / not-configured): a
//!   registration that is only configured can never serialize, parse, or
//!   report itself as connected, and a connected or configured
//!   registration always references a stored connection.
//! - **Credentials are references only** (kernel §7, addendum §3): the
//!   registry stores [`SecretRef`](crate::SecretRef) strings inside its
//!   connections and nothing else — no contract type, fixture, or
//!   serialized registry state can carry credential material.
//! - **Durable entity rules** (kernel §3, addendum §5): registrations and
//!   connections are created at version 1, every durable mutation
//!   increments by exactly 1, and updates carry the caller's expected
//!   version — a mismatch is a
//!   [`VersionConflict`](crate::ExecStoreError::VersionConflict), never a
//!   silent overwrite.
//! - **Canonical JSON** (kernel §4): `"v": 1`, snake_case fields,
//!   `deny_unknown_fields`, no floats, RFC 3339 UTC timestamps. Fixtures
//!   live under `tests/fixtures/w2/`.
//!
//! The registry is an in-memory store in this wave (deterministic, no
//! I/O, caller-supplied timestamps — kernel §7); the persistence wiring
//! arrives with the provider-connection wave. [`ModelRegistry::snapshot`]
//! / [`ModelRegistry::restore`] give the serialize → drop → reload
//! round-trip the platform's state discipline requires.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;
use crate::connection::ProviderConnection;
use crate::ids::{ModelId, ProviderConnectionId};
use crate::model::{Model, ModelProvider};
use crate::refs::ActorRef;
use crate::store::ExecStoreError;
use crate::time::Timestamp;
use crate::{
    ContractVersion, ExecError, MAX_MODELS_PER_PROVIDER, ensure_kind_label, ensure_list_bound,
};

/// The availability of a registered provider: the three honest states of
/// the F7 bring-your-own-provider foundation. The UI must never imply a
/// connection exists when the provider is only configured (work order
/// MOD-001 integration note), so the states are serialized distinctly and
/// the consistency rules are enforced where the registration is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAvailability {
    /// No connection is stored for the provider: nothing is configured,
    /// and the provider's models are visible only as honest
    /// not-yet-connectable offers.
    NotConfigured,
    /// A connection is stored but is not connected: the provider is
    /// configured, and it must never be reported as connected.
    ConfiguredNotConnected,
    /// A connection is stored and connected: the provider's models can
    /// run and draw on the connection's account.
    Connected,
}

impl ProviderAvailability {
    /// Whether the provider is connected (the only state in which its
    /// models can run).
    #[must_use]
    pub const fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Whether this state must reference a stored connection: every state
    /// except [`Self::NotConfigured`] does.
    #[must_use]
    pub const fn requires_connection(self) -> bool {
        !matches!(self, Self::NotConfigured)
    }
}

/// A model-provider registration: the durable record of one provider
/// whose models the platform knows. Mirrors the frozen
/// [`ModelProvider`](crate::ModelProvider) contract's data — provider
/// kind, sourced models with their capability metadata, and the
/// user-owned connection the provider draws on — plus the honest
/// [`ProviderAvailability`] state.
///
/// Durable entity rules (kernel §3): created at version 1, +1 per
/// mutation, optimistic concurrency through the registry. There is no
/// `prov_` ID kind in the frozen prefix registry: a registration is keyed
/// by its provider kind label (one registration per provider kind in v1;
/// per-account connections are the F7 flow).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderRegistration {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// The neutral kind label of the provider (for example `openai`,
    /// `ollama`). The registry key.
    pub provider_kind: String,
    /// The honest availability state of the provider.
    pub availability: ProviderAvailability,
    /// The connection the provider draws on. Required by
    /// [`ProviderAvailability::ConfiguredNotConnected`] and
    /// [`ProviderAvailability::Connected`]; forbidden by
    /// [`ProviderAvailability::NotConfigured`].
    pub connection_id: Option<ProviderConnectionId>,
    /// The models this provider sources, in canonical ID order, each
    /// carrying its capability metadata. Bounded by
    /// [`MAX_MODELS_PER_PROVIDER`](crate::MAX_MODELS_PER_PROVIDER).
    pub models: Vec<Model>,
    /// The actor that registered the provider.
    pub created_by: ActorRef,
    /// Registration timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl ProviderRegistration {
    /// Builds a new provider registration at version 1.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the provider kind is not a
    /// lowercase token, a model fails validation, a model is sourced from
    /// a different provider, the model list is unsorted or duplicated,
    /// or the availability state contradicts the connection reference
    /// (a not-configured provider cannot reference a connection; a
    /// configured or connected one must).
    pub fn new(
        provider_kind: &str,
        availability: ProviderAvailability,
        connection_id: Option<ProviderConnectionId>,
        models: Vec<Model>,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ExecError> {
        ensure_kind_label("registry provider kind", provider_kind)?;
        ensure_list_bound("registered models", &models, MAX_MODELS_PER_PROVIDER)?;
        for model in &models {
            model.validate()?;
            if model.provider_kind != provider_kind {
                return Err(ExecError::invalid(format!(
                    "model {} is sourced from provider {:?}, not {provider_kind:?}",
                    model.id, model.provider_kind
                )));
            }
        }
        if models.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(ExecError::invalid(
                "registered models must be sorted by canonical ID and free of duplicates",
            ));
        }
        match (availability.requires_connection(), &connection_id) {
            (true, None) => {
                return Err(ExecError::invalid(format!(
                    "provider availability {availability:?} must reference a stored connection"
                )));
            }
            (false, Some(connection_id)) => {
                return Err(ExecError::invalid(format!(
                    "a not-configured provider cannot reference connection {connection_id}"
                )));
            }
            _ => {}
        }
        Ok(Self {
            v: ContractVersion,
            version: 1,
            provider_kind: provider_kind.to_owned(),
            availability,
            connection_id,
            models,
            created_by,
            created_at,
        })
    }

    /// Validates the registration record.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when any canonical rule of
    /// [`Self::new`] is violated, or when `version` is zero.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(
            &self.provider_kind,
            self.availability,
            self.connection_id.clone(),
            self.models.clone(),
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(ExecError::invalid(
                "provider registration version must be at least 1",
            ));
        }
        Ok(())
    }
}

/// The canonical serialization of the entire registry state: provider
/// registrations keyed by provider kind, and provider connections keyed
/// by canonical ID. Used for serialize → drop → reload round-trips; the
/// state survives with equality (kernel round-trip law).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrySnapshot {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// All provider registrations, keyed by provider kind.
    pub registrations: BTreeMap<String, ProviderRegistration>,
    /// All provider connections, keyed by canonical ID.
    pub connections: BTreeMap<ProviderConnectionId, ProviderConnection>,
}

impl RegistrySnapshot {
    /// The empty snapshot.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            v: ContractVersion,
            registrations: BTreeMap::new(),
            connections: BTreeMap::new(),
        }
    }

    /// Validates the snapshot's structural invariants: map keys match the
    /// entities they store, every entity passes canonical validation,
    /// every referenced connection exists with a matching provider kind,
    /// and no model ID is registered twice.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError::Invalid`] when any invariant fails.
    pub fn validate(&self) -> Result<(), ExecStoreError> {
        let mut model_ids = std::collections::BTreeSet::new();
        for (kind, registration) in &self.registrations {
            if kind != &registration.provider_kind {
                return Err(ExecStoreError::Invalid {
                    reason: format!(
                        "registration key {kind:?} does not match provider kind {:?}",
                        registration.provider_kind
                    ),
                });
            }
            registration.validate().map_err(ExecStoreError::from)?;
            if let Some(connection_id) = &registration.connection_id {
                match self.connections.get(connection_id) {
                    None => {
                        return Err(ExecStoreError::Invalid {
                            reason: format!(
                                "provider {kind:?} references missing connection {connection_id}"
                            ),
                        });
                    }
                    Some(connection) => {
                        if connection.provider_kind != *kind {
                            return Err(ExecStoreError::Invalid {
                                reason: format!(
                                    "connection {connection_id} belongs to provider {:?}, \
                                     not {kind:?}",
                                    connection.provider_kind
                                ),
                            });
                        }
                    }
                }
            }
            for model in &registration.models {
                if !model_ids.insert(model.id.clone()) {
                    return Err(ExecStoreError::Invalid {
                        reason: format!("model {} is registered twice", model.id),
                    });
                }
            }
        }
        for (id, connection) in &self.connections {
            if id != &connection.id {
                return Err(ExecStoreError::Invalid {
                    reason: format!(
                        "connection key {id} does not match connection id {}",
                        connection.id
                    ),
                });
            }
            connection.validate().map_err(ExecStoreError::from)?;
        }
        Ok(())
    }
}

/// A point-in-time view of one registered provider, satisfying the frozen
/// [`ModelProvider`] contract (addendum §1: the fabric lives BEHIND the
/// frozen interfaces). The registry is how the platform sources models;
/// a snapshot is what a model-selection surface reads.
#[derive(Debug, Clone)]
pub struct ProviderSnapshot {
    registration: ProviderRegistration,
    capabilities: Vec<CapabilityId>,
}

impl ProviderSnapshot {
    /// Builds the view from a registration, computing the provider's
    /// capability advertisement as the sorted, deduplicated union of its
    /// models' capabilities.
    #[must_use]
    pub fn new(registration: ProviderRegistration) -> Self {
        let mut capabilities: Vec<CapabilityId> = registration
            .models
            .iter()
            .flat_map(|model| model.capabilities.iter().cloned())
            .collect();
        capabilities.sort();
        capabilities.dedup();
        Self {
            registration,
            capabilities,
        }
    }

    /// The registration the view was built from.
    #[must_use]
    pub fn registration(&self) -> &ProviderRegistration {
        &self.registration
    }

    /// The honest availability state of the provider.
    #[must_use]
    pub fn availability(&self) -> ProviderAvailability {
        self.registration.availability
    }
}

impl ModelProvider for ProviderSnapshot {
    fn provider_kind(&self) -> &str {
        &self.registration.provider_kind
    }

    fn capabilities(&self) -> &[CapabilityId] {
        &self.capabilities
    }

    fn connection_id(&self) -> Option<&ProviderConnectionId> {
        self.registration.connection_id.as_ref()
    }

    fn models(&self) -> &[Model] {
        &self.registration.models
    }

    fn model(&self, id: &ModelId) -> Option<Model> {
        self.registration
            .models
            .iter()
            .find(|model| &model.id == id)
            .cloned()
    }
}

/// The model/provider registry: provider registrations, their models,
/// and the provider connections through which accounts are accessed.
///
/// All mutations follow the kernel §3 version rules through
/// [`ExecStoreError`]; all timestamps are caller-supplied; the registry
/// performs no I/O and reads no clock (kernel §7 determinism).
#[derive(Debug, Default, Clone)]
pub struct ModelRegistry {
    registrations: BTreeMap<String, ProviderRegistration>,
    connections: BTreeMap<ProviderConnectionId, ProviderConnection>,
}

impl ModelRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a provider (created at version 1). The registration's
    /// connection, when present, must already be stored and belong to the
    /// same provider kind, and the registration's model IDs must not
    /// collide with models registered under another provider.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError::Invalid`] when the registration violates
    /// a canonical rule, [`ExecStoreError::Duplicate`] when the provider
    /// kind is already registered, and [`ExecStoreError::NotFound`] when
    /// the referenced connection is missing.
    pub fn register_provider(
        &mut self,
        registration: ProviderRegistration,
    ) -> Result<ProviderRegistration, ExecStoreError> {
        if registration.version != 1 {
            return Err(ExecStoreError::Invalid {
                reason: format!(
                    "provider {:?} must be registered at version 1",
                    registration.provider_kind
                ),
            });
        }
        registration.validate().map_err(ExecStoreError::from)?;
        if self.registrations.contains_key(&registration.provider_kind) {
            return Err(ExecStoreError::Duplicate {
                id: registration.provider_kind.clone(),
            });
        }
        self.ensure_connection(&registration)?;
        self.ensure_unique_models(&registration)?;
        let kind = registration.provider_kind.clone();
        self.registrations.insert(kind, registration.clone());
        Ok(registration)
    }

    /// Looks up a provider registration by kind.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError`] only when the store itself fails; a
    /// missing registration is `Ok(None)`.
    pub fn provider(
        &self,
        provider_kind: &str,
    ) -> Result<Option<ProviderRegistration>, ExecStoreError> {
        Ok(self.registrations.get(provider_kind).cloned())
    }

    /// All provider registrations, in provider-kind order.
    #[must_use]
    pub fn providers(&self) -> Vec<ProviderRegistration> {
        self.registrations.values().cloned().collect()
    }

    /// Updates a provider registration; the passed `version` is the
    /// expected version, and the stored registration is returned with its
    /// version incremented by exactly one.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError::NotFound`] when the kind is unknown,
    /// [`ExecStoreError::VersionConflict`] on a stale expected version,
    /// and [`ExecStoreError::Invalid`] when a canonical rule or
    /// cross-reference would break.
    pub fn update_provider(
        &mut self,
        registration: ProviderRegistration,
    ) -> Result<ProviderRegistration, ExecStoreError> {
        registration.validate().map_err(ExecStoreError::from)?;
        let kind = registration.provider_kind.clone();
        let Some(stored) = self.registrations.get(&kind) else {
            return Err(ExecStoreError::NotFound { id: kind });
        };
        if stored.version != registration.version {
            return Err(ExecStoreError::VersionConflict {
                id: kind,
                expected_version: registration.version,
                actual_version: stored.version,
            });
        }
        self.ensure_connection(&registration)?;
        self.ensure_unique_models(&registration)?;
        let mut next = registration;
        next.version = stored.version + 1;
        self.registrations.insert(kind, next.clone());
        Ok(next)
    }

    /// Removes a provider registration; its models leave the registry
    /// with it. The connection it referenced stays (its lifecycle is the
    /// connection's own).
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError::NotFound`] when the kind is unknown and
    /// [`ExecStoreError::VersionConflict`] on a stale expected version.
    pub fn unregister_provider(
        &mut self,
        provider_kind: &str,
        expected_version: u64,
    ) -> Result<ProviderRegistration, ExecStoreError> {
        let Some(stored) = self.registrations.get(provider_kind) else {
            return Err(ExecStoreError::NotFound {
                id: provider_kind.to_owned(),
            });
        };
        if stored.version != expected_version {
            return Err(ExecStoreError::VersionConflict {
                id: provider_kind.to_owned(),
                expected_version,
                actual_version: stored.version,
            });
        }
        match self.registrations.remove(provider_kind) {
            Some(registration) => Ok(registration),
            None => Err(ExecStoreError::NotFound {
                id: provider_kind.to_owned(),
            }),
        }
    }

    /// Stores a new provider connection (created at version 1).
    /// Credential references only — the connection type cannot carry
    /// material.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError::Invalid`] when the connection violates a
    /// canonical rule and [`ExecStoreError::Duplicate`] when its ID is
    /// already stored.
    pub fn add_connection(
        &mut self,
        connection: ProviderConnection,
    ) -> Result<ProviderConnection, ExecStoreError> {
        if connection.version != 1 {
            return Err(ExecStoreError::Invalid {
                reason: format!("connection {} must be added at version 1", connection.id),
            });
        }
        connection.validate().map_err(ExecStoreError::from)?;
        if self.connections.contains_key(&connection.id) {
            return Err(ExecStoreError::Duplicate {
                id: connection.id.as_str().to_owned(),
            });
        }
        self.connections
            .insert(connection.id.clone(), connection.clone());
        Ok(connection)
    }

    /// Looks up a provider connection.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError`] only when the store itself fails; a
    /// missing connection is `Ok(None)`.
    pub fn connection(
        &self,
        id: &ProviderConnectionId,
    ) -> Result<Option<ProviderConnection>, ExecStoreError> {
        Ok(self.connections.get(id).cloned())
    }

    /// Updates a provider connection (optimistic concurrency on the
    /// passed version). The connection's provider kind is immutable: a
    /// connection to a different provider is a new connection.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError::NotFound`] when the connection is
    /// unknown, [`ExecStoreError::VersionConflict`] on a stale expected
    /// version, and [`ExecStoreError::Invalid`] on a canonical violation
    /// or a provider-kind change.
    pub fn update_connection(
        &mut self,
        connection: ProviderConnection,
    ) -> Result<ProviderConnection, ExecStoreError> {
        connection.validate().map_err(ExecStoreError::from)?;
        let Some(stored) = self.connections.get(&connection.id) else {
            return Err(ExecStoreError::NotFound {
                id: connection.id.as_str().to_owned(),
            });
        };
        if stored.version != connection.version {
            return Err(ExecStoreError::VersionConflict {
                id: connection.id.as_str().to_owned(),
                expected_version: connection.version,
                actual_version: stored.version,
            });
        }
        if stored.provider_kind != connection.provider_kind {
            return Err(ExecStoreError::Invalid {
                reason: format!(
                    "connection {} belongs to provider {:?}; its kind is immutable \
                     (register a new connection instead)",
                    connection.id, stored.provider_kind
                ),
            });
        }
        let mut next = connection;
        next.version = stored.version + 1;
        self.connections.insert(next.id.clone(), next.clone());
        Ok(next)
    }

    /// Removes a provider connection. A connection still referenced by a
    /// provider registration is never removed — unregistering the
    /// provider first is the honest path.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError::NotFound`] when the connection is
    /// unknown, [`ExecStoreError::VersionConflict`] on a stale expected
    /// version, and [`ExecStoreError::Invalid`] while the connection is
    /// still referenced.
    pub fn remove_connection(
        &mut self,
        id: &ProviderConnectionId,
        expected_version: u64,
    ) -> Result<ProviderConnection, ExecStoreError> {
        let Some(stored) = self.connections.get(id) else {
            return Err(ExecStoreError::NotFound {
                id: id.as_str().to_owned(),
            });
        };
        if stored.version != expected_version {
            return Err(ExecStoreError::VersionConflict {
                id: id.as_str().to_owned(),
                expected_version,
                actual_version: stored.version,
            });
        }
        let referenced = self
            .registrations
            .values()
            .any(|registration| registration.connection_id.as_ref() == Some(id));
        if referenced {
            return Err(ExecStoreError::Invalid {
                reason: format!("connection {id} is still referenced by a provider registration"),
            });
        }
        match self.connections.remove(id) {
            Some(connection) => Ok(connection),
            None => Err(ExecStoreError::NotFound {
                id: id.as_str().to_owned(),
            }),
        }
    }

    /// Looks up a registered model by canonical ID, across providers.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError`] only when the store itself fails; a
    /// missing model is `Ok(None)`.
    pub fn model(&self, id: &ModelId) -> Result<Option<Model>, ExecStoreError> {
        Ok(self
            .registrations
            .values()
            .flat_map(|registration| registration.models.iter())
            .find(|model| &model.id == id)
            .cloned())
    }

    /// All registered models, in canonical ID order.
    #[must_use]
    pub fn models(&self) -> Vec<Model> {
        let mut models: Vec<Model> = self
            .registrations
            .values()
            .flat_map(|registration| registration.models.iter().cloned())
            .collect();
        models.sort_by(|left, right| left.id.cmp(&right.id));
        models
    }

    /// A [`ModelProvider`]-contract view of one registered provider: what
    /// a model-selection surface reads.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError`] only when the store itself fails; a
    /// missing provider is `Ok(None)`.
    pub fn provider_view(
        &self,
        provider_kind: &str,
    ) -> Result<Option<ProviderSnapshot>, ExecStoreError> {
        Ok(self
            .registrations
            .get(provider_kind)
            .cloned()
            .map(ProviderSnapshot::new))
    }

    /// [`ModelProvider`]-contract views of every registered provider, in
    /// provider-kind order.
    #[must_use]
    pub fn provider_views(&self) -> Vec<ProviderSnapshot> {
        self.registrations
            .values()
            .cloned()
            .map(ProviderSnapshot::new)
            .collect()
    }

    /// Captures the entire registry state as a canonical snapshot.
    #[must_use]
    pub fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            v: ContractVersion,
            registrations: self.registrations.clone(),
            connections: self.connections.clone(),
        }
    }

    /// Rebuilds the registry from a snapshot (serialize → drop →
    /// reload). The snapshot must satisfy the canonical invariants.
    ///
    /// # Errors
    ///
    /// Returns [`ExecStoreError::Invalid`] when the snapshot violates an
    /// invariant.
    pub fn restore(snapshot: RegistrySnapshot) -> Result<Self, ExecStoreError> {
        snapshot.validate()?;
        Ok(Self {
            registrations: snapshot.registrations,
            connections: snapshot.connections,
        })
    }

    /// The connection a registration references must exist and belong to
    /// the same provider kind.
    fn ensure_connection(&self, registration: &ProviderRegistration) -> Result<(), ExecStoreError> {
        if let Some(connection_id) = &registration.connection_id {
            match self.connections.get(connection_id) {
                None => {
                    return Err(ExecStoreError::NotFound {
                        id: connection_id.as_str().to_owned(),
                    });
                }
                Some(connection) => {
                    if connection.provider_kind != registration.provider_kind {
                        return Err(ExecStoreError::Invalid {
                            reason: format!(
                                "connection {connection_id} belongs to provider {:?}, \
                                 not {:?}",
                                connection.provider_kind, registration.provider_kind
                            ),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// A registration's model IDs must not collide with models already
    /// registered under another provider.
    fn ensure_unique_models(
        &self,
        registration: &ProviderRegistration,
    ) -> Result<(), ExecStoreError> {
        for model in &registration.models {
            let clash = self
                .registrations
                .iter()
                .filter(|(kind, _)| **kind != registration.provider_kind)
                .any(|(_, stored)| {
                    stored
                        .models
                        .iter()
                        .any(|stored_model| stored_model.id == model.id)
                });
            if clash {
                return Err(ExecStoreError::Invalid {
                    reason: format!(
                        "model {} is already registered under another provider",
                        model.id
                    ),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::connection::SecretRef;
    use crate::fakes::{FakeModelProvider, fake_model_ids};
    use crate::ids::ProviderConnectionId;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn test_actor() -> ActorRef {
        ok(ActorRef::user("alice"))
    }

    fn test_timestamp() -> Timestamp {
        ok(Timestamp::parse("2026-09-21T13:45:00Z"))
    }

    fn test_connection_id() -> ProviderConnectionId {
        ok(ProviderConnectionId::parse(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQP34",
        ))
    }

    fn test_connection() -> ProviderConnection {
        // A connection to the `flauz-fake` provider (the shared fake
        // connection is `e2b`-kinded — Wave-1 frozen — so the registry
        // tests build their own provider-matching connection).
        ok(ProviderConnection::new(
            test_connection_id(),
            "flauz-fake",
            "Fake sandbox account",
            ok(SecretRef::parse(crate::fakes::FAKE_SECRET_REF)),
            test_actor(),
            test_timestamp(),
        ))
    }

    fn fake_models() -> Vec<Model> {
        ok(FakeModelProvider::new()).models().to_vec()
    }

    fn fixture_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
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

    /// The honest availability matrix: every state is representable, and
    /// the connection consistency rules reject the dishonest pairs.
    #[test]
    fn registration_validates_availability_connection_consistency() {
        let models = fake_models();

        // (NotConfigured, no connection) is the honest unconfigured state.
        ok(ProviderRegistration::new(
            "ollama",
            ProviderAvailability::NotConfigured,
            None,
            Vec::new(),
            test_actor(),
            test_timestamp(),
        ));
        // (Connected, connection) and (ConfiguredNotConnected, connection)
        // are the honest connection states.
        ok(ProviderRegistration::new(
            "flauz-fake",
            ProviderAvailability::Connected,
            Some(test_connection_id()),
            models.clone(),
            test_actor(),
            test_timestamp(),
        ));
        ok(ProviderRegistration::new(
            "flauz-fake",
            ProviderAvailability::ConfiguredNotConnected,
            Some(test_connection_id()),
            Vec::new(),
            test_actor(),
            test_timestamp(),
        ));

        // A not-configured provider cannot reference a connection.
        assert!(
            ProviderRegistration::new(
                "ollama",
                ProviderAvailability::NotConfigured,
                Some(test_connection_id()),
                Vec::new(),
                test_actor(),
                test_timestamp(),
            )
            .is_err()
        );
        // Connected and configured providers must reference one.
        for availability in [
            ProviderAvailability::Connected,
            ProviderAvailability::ConfiguredNotConnected,
        ] {
            assert!(
                ProviderRegistration::new(
                    "flauz-fake",
                    availability,
                    None,
                    Vec::new(),
                    test_actor(),
                    test_timestamp(),
                )
                .is_err(),
                "{availability:?} without a connection must be rejected"
            );
        }

        // Sourcing provenance: a model from another provider is rejected,
        // as are unsorted and duplicated model lists.
        let mut foreign = models.clone();
        foreign[0].provider_kind = "ollama".to_owned();
        assert!(
            ProviderRegistration::new(
                "flauz-fake",
                ProviderAvailability::NotConfigured,
                None,
                foreign,
                test_actor(),
                test_timestamp(),
            )
            .is_err()
        );
        let mut unsorted = vec![models[1].clone(), models[0].clone()];
        assert!(
            ProviderRegistration::new(
                "flauz-fake",
                ProviderAvailability::NotConfigured,
                None,
                unsorted.clone(),
                test_actor(),
                test_timestamp(),
            )
            .is_err()
        );
        unsorted[1] = unsorted[0].clone();
        assert!(
            ProviderRegistration::new(
                "flauz-fake",
                ProviderAvailability::NotConfigured,
                None,
                unsorted,
                test_actor(),
                test_timestamp(),
            )
            .is_err()
        );

        // The states serialize distinctly (the UI must never read one as
        // another) and round-trip.
        for (availability, wire) in [
            (ProviderAvailability::NotConfigured, "not_configured"),
            (
                ProviderAvailability::ConfiguredNotConnected,
                "configured_not_connected",
            ),
            (ProviderAvailability::Connected, "connected"),
        ] {
            let serialized = ok(serde_json::to_string(&availability));
            assert_eq!(serialized, format!("\"{wire}\""));
            let parsed: ProviderAvailability = ok(serde_json::from_str(&serialized));
            assert_eq!(parsed, availability);
        }
        assert!(!ProviderAvailability::NotConfigured.requires_connection());
        assert!(ProviderAvailability::ConfiguredNotConnected.requires_connection());
        assert!(ProviderAvailability::Connected.requires_connection());
        assert!(!ProviderAvailability::NotConfigured.is_connected());
        assert!(!ProviderAvailability::ConfiguredNotConnected.is_connected());
        assert!(ProviderAvailability::Connected.is_connected());
    }

    /// Registry CRUD over registrations and connections, with the kernel
    /// §3 version rules: version 1 at create, +1 per mutation, stale
    /// expected versions conflict.
    #[test]
    fn registry_crud_and_version_rules() {
        let mut registry = ModelRegistry::new();
        ok(registry.add_connection(test_connection()));

        let registered = ok(registry.register_provider(ok(ProviderRegistration::new(
            "flauz-fake",
            ProviderAvailability::Connected,
            Some(test_connection_id()),
            fake_models(),
            test_actor(),
            test_timestamp(),
        ))));
        assert_eq!(
            registered.version, 1,
            "registrations are created at version 1"
        );
        assert_eq!(
            ok(registry.provider("flauz-fake"))
                .unwrap_or_else(|| panic!("the provider must be readable"))
                .provider_kind,
            "flauz-fake"
        );
        assert_eq!(registry.models().len(), 2);
        assert_eq!(registry.providers().len(), 1);

        // Duplicates and non-1 creation versions are rejected.
        assert!(matches!(
            registry.register_provider(registered.clone()),
            Err(ExecStoreError::Duplicate { .. })
        ));
        let mut poisoned = registered.clone();
        poisoned.version = 7;
        assert!(matches!(
            registry.register_provider(poisoned),
            Err(ExecStoreError::Invalid { .. })
        ));

        // A durable mutation (the honest availability transition) bumps
        // the version by exactly one; a stale expected version conflicts.
        let mut disconnected = registered.clone();
        disconnected.availability = ProviderAvailability::ConfiguredNotConnected;
        let updated = ok(registry.update_provider(disconnected));
        assert_eq!(updated.version, 2, "+1 per durable mutation");
        assert!(!updated.availability.is_connected());
        assert!(matches!(
            registry.update_provider(registered),
            Err(ExecStoreError::VersionConflict { .. })
        ));

        // Unregistering removes the provider and its models; a stale
        // expected version conflicts. Re-registering is a fresh version-1
        // registration (the removed registration's version history does
        // not carry over).
        let removed = ok(registry.unregister_provider("flauz-fake", 2));
        assert_eq!(removed.version, 2);
        assert!(registry.models().is_empty());
        assert!(ok(registry.provider("flauz-fake")).is_none());
        assert!(matches!(
            registry.unregister_provider("flauz-fake", 2),
            Err(ExecStoreError::NotFound { .. })
        ));
        ok(registry.register_provider(ok(ProviderRegistration::new(
            "flauz-fake",
            ProviderAvailability::Connected,
            Some(test_connection_id()),
            fake_models(),
            test_actor(),
            test_timestamp(),
        ))));
        assert!(matches!(
            registry.unregister_provider("flauz-fake", 0),
            Err(ExecStoreError::VersionConflict { .. })
        ));

        // Connections follow the same discipline: version 1 at add, +1
        // per update, stale versions conflict, kind is immutable, and a
        // referenced connection is never removed.
        let connection = ok(registry.connection(&test_connection_id()))
            .unwrap_or_else(|| panic!("the connection must be readable"));
        assert_eq!(connection.version, 1);
        let mut relabeled = connection.clone();
        relabeled.account_label = "Personal sandbox account".to_owned();
        let relabeled = ok(registry.update_connection(relabeled));
        assert_eq!(relabeled.version, 2);
        assert!(matches!(
            registry.update_connection(connection),
            Err(ExecStoreError::VersionConflict { .. })
        ));
        let mut rekinded = relabeled.clone();
        rekinded.provider_kind = "e2b".to_owned();
        assert!(matches!(
            registry.update_connection(rekinded),
            Err(ExecStoreError::Invalid { .. })
        ));
        assert!(matches!(
            registry.remove_connection(&test_connection_id(), 2),
            Err(ExecStoreError::Invalid { .. })
        ));
        ok(registry.unregister_provider("flauz-fake", 1));
        let removed_connection = ok(registry.remove_connection(&test_connection_id(), 2));
        assert_eq!(removed_connection.account_label, "Personal sandbox account");
        assert!(ok(registry.connection(&test_connection_id())).is_none());

        // A registration cannot reference a missing connection, a foreign
        // connection, or another provider's models.
        ok(registry.add_connection(test_connection()));
        let mut foreign_connection = test_connection();
        foreign_connection.provider_kind = "e2b".to_owned();
        foreign_connection.id = ok(ProviderConnectionId::parse(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
        ));
        ok(registry.add_connection(foreign_connection.clone()));
        assert!(matches!(
            registry.register_provider(ok(ProviderRegistration::new(
                "flauz-fake",
                ProviderAvailability::Connected,
                Some(foreign_connection.id.clone()),
                fake_models(),
                test_actor(),
                test_timestamp(),
            ))),
            Err(ExecStoreError::Invalid { .. })
        ));
        ok(registry.register_provider(ok(ProviderRegistration::new(
            "flauz-fake",
            ProviderAvailability::Connected,
            Some(test_connection_id()),
            fake_models(),
            test_actor(),
            test_timestamp(),
        ))));
        // A model ID already registered under another provider is
        // rejected: the re-kinded models pass their own provenance, but
        // their IDs clash with the registered fake models.
        let mut clash = fake_models();
        for model in &mut clash {
            model.provider_kind = "ollama".to_owned();
        }
        assert!(matches!(
            registry.register_provider(ok(ProviderRegistration::new(
                "ollama",
                ProviderAvailability::NotConfigured,
                None,
                clash,
                test_actor(),
                test_timestamp(),
            ))),
            Err(ExecStoreError::Invalid { .. })
        ));
    }

    /// The registry's provider views satisfy the frozen ModelProvider
    /// contract: kind, capability union, connection reference, models in
    /// canonical order, and lookup.
    #[test]
    fn registry_provider_view_satisfies_model_provider_contract() {
        let mut registry = ModelRegistry::new();
        ok(registry.add_connection(test_connection()));
        ok(registry.register_provider(ok(ProviderRegistration::new(
            "flauz-fake",
            ProviderAvailability::Connected,
            Some(test_connection_id()),
            fake_models(),
            test_actor(),
            test_timestamp(),
        ))));
        ok(registry.register_provider(ok(ProviderRegistration::new(
            "ollama",
            ProviderAvailability::NotConfigured,
            None,
            Vec::new(),
            test_actor(),
            test_timestamp(),
        ))));

        let views = registry.provider_views();
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].provider_kind(), "flauz-fake");
        assert_eq!(views[1].provider_kind(), "ollama");

        let view = ok(registry.provider_view("flauz-fake"))
            .unwrap_or_else(|| panic!("the provider view must be readable"));
        assert_eq!(view.provider_kind(), "flauz-fake");
        // The provider advertisement is the union of its models'
        // capabilities, sorted and deduplicated.
        assert_eq!(view.capabilities().len(), 2);
        assert_eq!(view.capabilities()[0].as_str(), "image.input");
        assert_eq!(view.capabilities()[1].as_str(), "vision");
        // The connection through which the provider is accessed.
        assert_eq!(view.connection_id(), Some(&test_connection_id()));
        assert_eq!(view.availability(), ProviderAvailability::Connected);
        // The models, in canonical ID order, and lookup by ID.
        assert_eq!(view.models().len(), 2);
        assert!(view.models()[0].id < view.models()[1].id);
        let vision_id = ok(ModelId::parse(fake_model_ids::VISION));
        assert_eq!(
            view.model(&vision_id).map(|model| model.name),
            Some("Fake Vision Model".to_owned())
        );
        let text_id = ok(ModelId::parse(fake_model_ids::TEXT));
        assert!(ok(registry.model(&text_id)).is_some());
        assert_eq!(registry.models().len(), 2);
        // The not-configured provider honestly reports no connection.
        let ollama = ok(registry.provider_view("ollama"))
            .unwrap_or_else(|| panic!("the provider view must be readable"));
        assert_eq!(ollama.connection_id(), None);
        assert_eq!(ollama.availability(), ProviderAvailability::NotConfigured);
        assert!(ollama.models().is_empty());
    }

    /// The registry state survives serialize → drop → reload with
    /// equality (kernel round-trip law).
    #[test]
    fn registry_snapshot_roundtrip_preserves_state() {
        let mut registry = ModelRegistry::new();
        ok(registry.add_connection(test_connection()));
        ok(registry.register_provider(ok(ProviderRegistration::new(
            "flauz-fake",
            ProviderAvailability::ConfiguredNotConnected,
            Some(test_connection_id()),
            fake_models(),
            test_actor(),
            test_timestamp(),
        ))));
        // The honest transition: the configured provider connects.
        let configured = ok(registry.provider("flauz-fake"))
            .unwrap_or_else(|| panic!("the provider must be readable"));
        let mut connected = configured.clone();
        connected.availability = ProviderAvailability::Connected;
        let updated = ok(registry.update_provider(connected));
        assert_eq!(updated.version, 2);

        let snapshot = registry.snapshot();
        let serialized = ok(serde_json::to_string(&snapshot));
        let reloaded: RegistrySnapshot = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, snapshot);
        let restored = ok(ModelRegistry::restore(reloaded));

        assert_eq!(restored.providers(), registry.providers());
        assert_eq!(restored.models(), registry.models());
        let restored_connection = ok(restored.connection(&test_connection_id()))
            .unwrap_or_else(|| panic!("the connection must survive reload"));
        assert_eq!(
            restored_connection,
            ok(registry.connection(&test_connection_id()))
                .unwrap_or_else(|| panic!("the connection must be readable"))
        );
        let restored_provider = ok(restored.provider("flauz-fake"))
            .unwrap_or_else(|| panic!("the provider must survive reload"));
        assert_eq!(restored_provider.version, 2);
        assert_eq!(
            restored_provider.availability,
            ProviderAvailability::Connected
        );
        assert_eq!(restored.snapshot(), registry.snapshot());

        // A snapshot whose cross-references break is refused on restore.
        let mut broken = registry.snapshot();
        if let Some(connection) = broken.connections.get_mut(&test_connection_id()) {
            connection.provider_kind = "e2b".to_owned();
        }
        assert!(ModelRegistry::restore(broken).is_err());
        // Removing the registration leaves the connection unreferenced —
        // legitimate state that must restore cleanly.
        let mut orphan = registry.snapshot();
        orphan.registrations.remove("flauz-fake");
        assert!(ModelRegistry::restore(orphan).is_ok());
    }

    /// Registry state and the w2 fixtures carry credential REFERENCES
    /// only: no credential material appears in any serialized form.
    #[test]
    fn registry_serializes_no_credential_material() {
        let mut registry = ModelRegistry::new();
        ok(registry.add_connection(test_connection()));
        ok(registry.register_provider(ok(ProviderRegistration::new(
            "flauz-fake",
            ProviderAvailability::Connected,
            Some(test_connection_id()),
            fake_models(),
            test_actor(),
            test_timestamp(),
        ))));
        let serialized = ok(serde_json::to_string(&registry.snapshot()));
        for marker in crate::connection::CREDENTIAL_MARKERS {
            assert!(
                !serialized.contains(marker),
                "serialized registry state must never contain credential material ({marker:?})"
            );
        }
        assert!(
            serialized.contains("flausec_"),
            "the connection is stored as an opaque reference"
        );

        // The committed w2 fixtures are data too: they stay
        // credential-free.
        let mut stack = vec![fixture_path("")];
        while let Some(directory) = stack.pop() {
            for entry in ok(std::fs::read_dir(&directory)) {
                let entry = ok(entry);
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path
                    .extension()
                    .is_some_and(|extension| extension == "json")
                {
                    let content = ok(std::fs::read_to_string(&path));
                    for marker in crate::connection::CREDENTIAL_MARKERS {
                        assert!(
                            !content.contains(marker),
                            "fixture {path:?} must never contain credential material ({marker:?})"
                        );
                    }
                }
            }
        }
    }

    /// Every w2 fixture is canonical JSON: it parses strictly and
    /// re-serializes to exactly the committed bytes.
    #[test]
    fn registry_fixtures_roundtrip_canonical() {
        for relative in [
            "provider-registration/minimal.json",
            "provider-registration/typical.json",
            "provider-registration/boundary.json",
        ] {
            let content = read_fixture(relative);
            let parsed: ProviderRegistration = ok(serde_json::from_str(&content));
            ok(parsed.validate());
            let serialized = ok(serde_json::to_string_pretty(&parsed));
            assert_eq!(
                serialized, content,
                "fixture {relative} is not canonical: re-serialization differs"
            );
        }
        for relative in ["registry/minimal.json", "registry/typical.json"] {
            let content = read_fixture(relative);
            let parsed: RegistrySnapshot = ok(serde_json::from_str(&content));
            ok(parsed.validate());
            let serialized = ok(serde_json::to_string_pretty(&parsed));
            assert_eq!(
                serialized, content,
                "fixture {relative} is not canonical: re-serialization differs"
            );
        }

        // The invalid fixtures fail their canonical checks: a connected
        // registration without a connection reference parses but fails
        // validation, and an unknown snapshot field fails the strict
        // parse.
        let dishonest = read_fixture("provider-registration/invalid-availability.json");
        let parsed: ProviderRegistration = ok(serde_json::from_str(&dishonest));
        assert!(
            parsed.validate().is_err(),
            "a connected registration without a connection must fail validation"
        );
        let unknown_field = read_fixture("registry/invalid-unknown-field.json");
        assert!(
            serde_json::from_str::<RegistrySnapshot>(&unknown_field).is_err(),
            "unknown fields must be rejected by the canonical read"
        );
    }

    /// SecretRef discipline re-asserted at the registry boundary: a raw
    /// credential can never become a connection the registry stores.
    #[test]
    fn registry_connections_hold_only_secret_references() {
        assert!(SecretRef::parse("sk-proj-abcdefgh1234").is_err());
        assert!(SecretRef::parse("flausec_ghp_0123456789abcdef").is_err());
        let connection = test_connection();
        let serialized = ok(serde_json::to_string(&connection));
        assert!(serialized.contains("flausec_"));
        ok(connection.validate());
    }
}
