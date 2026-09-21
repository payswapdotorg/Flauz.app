//! The provider-neutral execution environment contracts: [`Environment`],
//! its durable [`EnvironmentDescriptor`], and the
//! [`ExecutionProvider`] that sources environments.
//!
//! An Environment is **where execution actually happens** — local or
//! remote, from any provider (local Linux/Windows/macOS, E2B, Daytona,
//! Azure, Vercel Sandbox, Cloudflare, GitHub Actions, Codemagic, future
//! custom infrastructure). No provider is the canonical environment.
//!
//! Structural separation (constitution, kernel):
//!
//! - An [`Environment`] never determines a [`Model`](crate::Model): the
//!   descriptor below carries no model state at all.
//! - An [`Environment`] is not an [`Agent`](crate::Agent): environment
//!   identity is an `env_` ULID, agent identity is an `agent_` ULID, and
//!   neither type references the other. Model and environment choices are
//!   *execution state* (they appear in a
//!   [`RuntimeRequest`](crate::RuntimeRequest)), never entity identity.
//!
//! Capability advertisement (kernel §7): every environment advertises the
//! capabilities it offers as a sorted, deduplicated list of
//! [`CapabilityId`] keys.

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;
use crate::ids::EnvironmentId;
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContractVersion, ExecError, MAX_NAME_BYTES, ensure_capability_list, ensure_kind_label,
    ensure_non_empty, ensure_str_bound,
};

/// Where execution happens for an environment (constitution): local
/// (the user's own machine) or remote (any external infrastructure).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locality {
    /// Execution happens on a machine the user controls directly.
    Local,
    /// Execution happens on external infrastructure.
    Remote,
}

/// The lifecycle status of an environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvironmentStatus {
    /// The environment accepts work.
    Available,
    /// The environment is temporarily occupied.
    Busy,
    /// The environment is not reachable.
    Offline,
}

/// The provider-neutral execution environment contract. This is the ONE
/// contract every environment satisfies — local and remote alike, from any
/// provider (kernel §9: the same conformance test runs against both the
/// local and the remote fake).
///
/// The trait is the live surface; [`EnvironmentDescriptor`] is the durable,
/// serializable projection returned by [`Environment::descriptor`]. All
/// methods are pure: implementing types perform no I/O and read no clock.
pub trait Environment: Send + Sync {
    /// The canonical identity of this environment (`env_<ULID>`), opaque
    /// and never encoding provider or locality.
    fn environment_id(&self) -> &EnvironmentId;

    /// Where execution happens for this environment.
    fn locality(&self) -> Locality;

    /// The capabilities this environment offers (kernel advertisement
    /// rule): a sorted, deduplicated list of capability keys.
    fn capabilities(&self) -> &[CapabilityId];

    /// The current lifecycle status.
    fn status(&self) -> EnvironmentStatus;

    /// The durable canonical record of this environment (for storage,
    /// serialization and versioning).
    fn descriptor(&self) -> EnvironmentDescriptor;
}

/// The durable canonical record of an environment: the serialized,
/// versioned projection of an [`Environment`]. Stored through
/// [`ExecStore`](crate::ExecStore) with optimistic concurrency on
/// `version`.
///
/// The record deliberately contains **no model state**: an environment
/// never determines which model runs inside it (kernel §6 — model choice
/// is execution state, and environment choice is execution state; neither
/// is part of any identity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentDescriptor {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical environment ID (`env_<ULID>`).
    pub id: EnvironmentId,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// Human-readable environment name.
    pub name: String,
    /// Where execution happens for this environment.
    pub locality: Locality,
    /// The neutral kind label of the [`ExecutionProvider`] that sourced
    /// this environment, when known (for example `local`, `e2b`). This is
    /// provenance metadata, not behavior: the descriptor works the same
    /// regardless of which provider sourced it.
    pub provider_kind: Option<String>,
    /// The capabilities this environment offers (sorted, deduplicated).
    pub capabilities: Vec<CapabilityId>,
    /// The current lifecycle status.
    pub status: EnvironmentStatus,
    /// The user-owned [`ProviderConnection`](crate::ProviderConnection)
    /// through which this environment is reached, where provider quota
    /// belongs to the connection. Local environments carry `None`.
    pub connection_id: Option<crate::ProviderConnectionId>,
    /// The actor that recorded this environment.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl EnvironmentDescriptor {
    /// Builds a new environment record at version 1.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: EnvironmentId,
        name: &str,
        locality: Locality,
        provider_kind: Option<&str>,
        capabilities: Vec<CapabilityId>,
        status: EnvironmentStatus,
        connection_id: Option<crate::ProviderConnectionId>,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ExecError> {
        ensure_non_empty("environment name", name)?;
        ensure_str_bound("environment name", name, MAX_NAME_BYTES)?;
        if let Some(provider_kind) = provider_kind {
            ensure_kind_label("environment provider kind", provider_kind)?;
        }
        ensure_capability_list("environment capabilities", &capabilities)?;
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            name: name.to_owned(),
            locality,
            provider_kind: provider_kind.map(str::to_owned),
            capabilities,
            status,
            connection_id,
            created_by,
            created_at,
        })
    }

    /// Validates the environment record against canonical bounds and
    /// version rules.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(
            self.id.clone(),
            &self.name,
            self.locality,
            self.provider_kind.as_deref(),
            self.capabilities.clone(),
            self.status,
            self.connection_id.clone(),
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(ExecError::invalid("environment version must be at least 1"));
        }
        Ok(())
    }
}

/// A request to source a new environment from an
/// [`ExecutionProvider`]. The provider supplies identity (a generated
/// `env_` ULID — its only entropy source), the durable version and its own
/// kind label; everything else comes from the spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentSpec {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Human-readable environment name.
    pub name: String,
    /// Where execution should happen.
    pub locality: Locality,
    /// The capabilities the environment must offer (sorted, deduplicated).
    pub capabilities: Vec<CapabilityId>,
    /// The actor requesting the environment.
    pub requested_by: ActorRef,
    /// Request timestamp (caller-supplied).
    pub requested_at: Timestamp,
}

impl EnvironmentSpec {
    /// Builds a new environment sourcing spec.
    pub fn new(
        name: &str,
        locality: Locality,
        capabilities: Vec<CapabilityId>,
        requested_by: ActorRef,
        requested_at: Timestamp,
    ) -> Result<Self, ExecError> {
        ensure_non_empty("environment name", name)?;
        ensure_str_bound("environment name", name, MAX_NAME_BYTES)?;
        ensure_capability_list("environment capabilities", &capabilities)?;
        Ok(Self {
            v: ContractVersion,
            name: name.to_owned(),
            locality,
            capabilities,
            requested_by,
            requested_at,
        })
    }

    /// Validates the spec.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(
            &self.name,
            self.locality,
            self.capabilities.clone(),
            self.requested_by.clone(),
            self.requested_at,
        )?;
        Ok(())
    }
}

/// A pluggable infrastructure source of execution environments
/// (constitution: "Providers are pluggable infrastructure sources"). The
/// provider sources environments; the environments themselves advertise
/// capabilities — per the kernel advertisement rule, capability
/// advertisement belongs to each [`Environment`], not to the provider.
///
/// Users can connect their own provider accounts through a
/// [`ProviderConnection`](crate::ProviderConnection); provider quota
/// belongs to the user-owned connection where supported, which is why the
/// provider optionally references one.
pub trait ExecutionProvider: Send + Sync {
    /// The neutral kind label of this provider (for example `local`,
    /// `e2b`, `daytona`). Never a credential.
    fn provider_kind(&self) -> &str;

    /// The localities this provider can source environments for.
    fn supported_localities(&self) -> &[Locality];

    /// The user-owned connection through which this provider is accessed,
    /// where provider quota belongs to the connection. `None` when the
    /// provider needs no connected account (for example a local provider).
    fn connection_id(&self) -> Option<&crate::ProviderConnectionId>;

    /// Sources a new environment from the spec. The provider allocates the
    /// canonical `env_` identity, validates the requested locality against
    /// its supported localities, and returns the durable record at its
    /// initial version. Sourcing is stateful (the provider remembers what
    /// it sourced), so it takes `&mut self`; implementations perform no
    /// I/O in fakes.
    fn source_environment(
        &mut self,
        spec: &EnvironmentSpec,
    ) -> Result<EnvironmentDescriptor, ExecError>;

    /// The environments sourced so far, in canonical ID order (bounded).
    fn environments(&self) -> Vec<EnvironmentDescriptor>;
}

#[cfg(test)]
mod tests {
    use super::*;

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
        ok(Timestamp::from_unix_seconds(1_789_998_300))
    }

    fn test_capabilities() -> Vec<CapabilityId> {
        vec![
            ok(CapabilityId::parse("filesystem.read")),
            ok(CapabilityId::parse("terminal")),
        ]
    }

    #[test]
    fn descriptor_validates_bounds_and_kind_labels() {
        let id = EnvironmentId::generate();
        assert!(
            EnvironmentDescriptor::new(
                id.clone(),
                "",
                Locality::Local,
                None,
                test_capabilities(),
                EnvironmentStatus::Available,
                None,
                test_actor(),
                test_timestamp()
            )
            .is_err()
        );
        assert!(
            EnvironmentDescriptor::new(
                id,
                "Local Linux",
                Locality::Local,
                Some("Local"),
                test_capabilities(),
                EnvironmentStatus::Available,
                None,
                test_actor(),
                test_timestamp()
            )
            .is_err(),
            "provider kind labels must be lowercase tokens"
        );
        let unsorted = vec![
            ok(CapabilityId::parse("terminal")),
            ok(CapabilityId::parse("filesystem.read")),
        ];
        assert!(
            EnvironmentDescriptor::new(
                EnvironmentId::generate(),
                "Local Linux",
                Locality::Local,
                None,
                unsorted,
                EnvironmentStatus::Available,
                None,
                test_actor(),
                test_timestamp()
            )
            .is_err(),
            "capability advertisements must be sorted and deduplicated"
        );
    }

    #[test]
    fn descriptor_serializes_without_model_state() {
        let descriptor = ok(EnvironmentDescriptor::new(
            ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4")),
            "Local Linux workstation",
            Locality::Local,
            Some("local"),
            test_capabilities(),
            EnvironmentStatus::Available,
            None,
            test_actor(),
            test_timestamp(),
        ));
        let serialized = ok(serde_json::to_string(&descriptor));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,\"id\":\"env_01J8ZQ5V8K3T2B7N6X4R9DQPE4\",\"version\":1,",
                "\"name\":\"Local Linux workstation\",\"locality\":\"local\",",
                "\"provider_kind\":\"local\",\"capabilities\":[\"filesystem.read\",\"terminal\"],",
                "\"status\":\"available\",\"connection_id\":null,",
                "\"created_by\":{\"kind\":\"user\",\"id\":\"alice\"},",
                "\"created_at\":\"2026-09-21T13:45:00Z\"}"
            )
        );
        let parsed: EnvironmentDescriptor = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, descriptor);
        assert!(
            serde_json::from_str::<EnvironmentDescriptor>(&serialized.replace(
                "\"provider_kind\":\"local\",",
                "\"provider_kind\":\"local\",\"model_id\":null,"
            ))
            .is_err(),
            "unknown fields must be rejected"
        );
    }

    #[test]
    fn spec_round_trips() {
        let spec = ok(EnvironmentSpec::new(
            "CI runner",
            Locality::Remote,
            test_capabilities(),
            test_actor(),
            test_timestamp(),
        ));
        let serialized = ok(serde_json::to_string(&spec));
        let parsed: EnvironmentSpec = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, spec);
        assert!(
            EnvironmentSpec::new("", Locality::Local, vec![], test_actor(), test_timestamp())
                .is_err()
        );
    }
}
