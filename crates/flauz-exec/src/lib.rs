//! Provider-neutral execution and intelligence contracts for Flauz (F2
//! contract wave, work order ARCH-002).
//!
//! This crate is the single owner of the platform's execution-side
//! contracts: [`Environment`] and [`ExecutionProvider`], [`Model`] and
//! [`ModelProvider`], [`AgentRuntime`], [`Agent`], [`Skill`],
//! [`Capability`] and [`ProviderConnection`], plus the versioned
//! session/event [`transport`](crate::transport). Every cross-crate rule
//! implemented here comes from the frozen [F2 contract kernel]
//! (`docs/F2-CONTRACT-KERNEL.md`) and the frozen architecture
//! (`docs/FLAUZ-SOURCE-OF-TRUTH.md`).
//!
//! # The frozen separation (constitution)
//!
//! ```text
//! Client != Model != AgentRuntime != Skill != Environment != Provider
//! Agent != Environment
//! ```
//!
//! - An [`Environment`] is **where execution happens** (local or remote, any
//!   provider); an [`ExecutionProvider`] *sources* environments. No provider
//!   is the canonical environment.
//! - A [`Model`] is **intelligence**: what a model *is* (identity, sourced
//!   provider kind, offered capabilities). It contains no
//!   provider-specific environment behavior, and an [`Environment`] never
//!   determines a [`Model`] — the separation is structural (distinct types
//!   in distinct modules, no cross references) and tested.
//! - An [`AgentRuntime`] is the **orchestration loop** around a model and
//!   its tools. The official Codex app-server is ONE runtime — never the
//!   universal Flauz registry — so runtimes are referenced by neutral kind
//!   labels and [`Agent`] references at most one runtime.
//! - An [`Agent`] (`agent_` ULID) is never an [`Environment`] (`env_` ULID);
//!   the agent entity carries no environment state at all.
//! - A [`Skill`] is a reusable, model-agnostic behavior package expressing
//!   *required capabilities* (requirement representation only — capability
//!   resolution is later work, not this crate's).
//!
//! # Capability advertisement (kernel §2, §7)
//!
//! Every [`Environment`], [`ModelProvider`] and [`AgentRuntime`] advertises
//! the capabilities it offers. Capability identity is a namespaced string
//! key ([`CapabilityId`]: `terminal`, `browser.input`) — **not** a ULID
//! entity. Availability is the intersection
//! `model ∩ runtime ∩ environment ∩ permissions ∩ policy`; computing that
//! intersection is future resolution work. This crate only represents the
//! advertisements and the gaps: a runtime that cannot satisfy a request's
//! required capabilities reports a [`RuntimeStatus::CapabilityGap`]
//! carrying exactly the missing keys.
//!
//! # Credentials are references only (kernel §7)
//!
//! A [`ProviderConnection`] carries a provider kind, an account label and
//! an opaque [`SecretRef`] (`flausec_...`) — never credential material. No
//! contract type, fixture, log line or serialized state in this crate
//! contains credential material.
//!
//! # Versioned session/event transport (kernel §5)
//!
//! [`transport::SessionEventTransport`] carries session event streams as
//! versioned, bounded, newline-delimited JSON frames. It is generic over
//! `E: Serialize + DeserializeOwned + Clone + Debug + Send + Sync +
//! 'static` and treats envelopes as **opaque values**: the transport has
//! its own framing version field and never inspects envelope internals. No
//! exec-crate trait is implemented on foreign types (orphan-rule safety).
//!
//! # Registered event types (kernel §5)
//!
//! This crate registers the following `event_type` vocabulary (grammar
//! `<entity>.<verb_past>`); other crates register their own.
//!
//! | Constant | `event_type` |
//! |---|---|
//! | [`event_types::ENVIRONMENT_ATTACHED`] | `environment.attached` |
//! | [`event_types::ENVIRONMENT_DETACHED`] | `environment.detached` |
//! | [`event_types::MODEL_REGISTERED`] | `model.registered` |
//! | [`event_types::AGENT_REGISTERED`] | `agent.registered` |
//! | [`event_types::CONNECTION_CONNECTED`] | `connection.connected` |
//! | [`event_types::CONNECTION_DISCONNECTED`] | `connection.disconnected` |
//!
//! # Determinism (kernel §7)
//!
//! Contract code in this crate never reads wall-clock time or randomness
//! directly: every timestamp is passed in by callers. The only entropy
//! source is ID generation, confined to a private `ulid` module (no
//! external ULID crate). The in-memory [`fakes`] are fully deterministic
//! apart from generated IDs, and the fake entities exposed by constructors
//! use fixed canonical IDs.
//!
//! # Canonical JSON (kernel §4)
//!
//! Every top-level serialized contract type carries `"v": 1`, uses
//! snake_case field names, rejects unknown fields, contains no floats,
//! emits timestamps as RFC 3339 UTC `YYYY-MM-DDTHH:MM:SSZ`, and keeps all
//! strings and lists bounded. Durations do not occur in the v1 contracts;
//! when they are introduced they must be integer milliseconds.

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod agent;
pub mod capability;
pub mod connection;
pub mod environment;
pub mod fakes;
pub mod ids;
pub mod model;
pub mod refs;
pub mod runtime;
pub mod skill;
pub mod store;
pub mod time;
pub mod transport;

mod ulid;

pub use crate::agent::Agent;
pub use crate::capability::{Capability, CapabilityId};
pub use crate::connection::{ProviderConnection, SecretRef};
pub use crate::environment::{
    Environment, EnvironmentDescriptor, EnvironmentSpec, EnvironmentStatus, ExecutionProvider,
    Locality,
};
pub use crate::ids::{
    AgentId, EntityKind, EnvironmentId, IdError, ModelId, ProviderConnectionId, validate,
};
pub use crate::model::{Model, ModelProvider};
pub use crate::refs::{ActorKind, ActorRef};
pub use crate::runtime::{AgentRuntime, RuntimeOutcome, RuntimeRequest, RuntimeStatus};
pub use crate::skill::{SemanticVersion, Skill, SkillId};
pub use crate::store::{ExecSnapshot, ExecStore, ExecStoreError};
pub use crate::time::Timestamp;
pub use crate::transport::{
    DEFAULT_MAX_TRANSPORT_FRAME_BYTES, SessionEventTransport, TransportError, TransportFrame,
    TransportFrameKind, TransportFrameVersion,
};

/// The `event_type` vocabulary registered by this crate (kernel §5). Other
/// crates register their own; the shared grammar is
/// `<entity>.<verb_past>`.
pub mod event_types {
    /// An execution environment was attached to a task. Task identity is
    /// preserved (environment choice is execution state).
    pub const ENVIRONMENT_ATTACHED: &str = "environment.attached";
    /// An execution environment was detached from a task.
    pub const ENVIRONMENT_DETACHED: &str = "environment.detached";
    /// A model became registered and usable.
    pub const MODEL_REGISTERED: &str = "model.registered";
    /// An agent became registered against a runtime.
    pub const AGENT_REGISTERED: &str = "agent.registered";
    /// A provider connection was connected.
    pub const CONNECTION_CONNECTED: &str = "connection.connected";
    /// A provider connection was disconnected.
    pub const CONNECTION_DISCONNECTED: &str = "connection.disconnected";
}

/// Maximum length of short human-readable names (environments, models,
/// agents, skills, connection labels).
pub const MAX_NAME_BYTES: usize = 256;

/// Maximum length of statements, objectives, instructions and other
/// bounded prose fields.
pub const MAX_STATEMENT_BYTES: usize = 8 * 1024;

/// Maximum length of bounded reference strings (secret references).
pub const MAX_REFERENCE_BYTES: usize = 2048;

/// Maximum length of an actor identifier string.
pub const MAX_ACTOR_ID_BYTES: usize = 256;

/// Maximum number of capabilities advertised by one entity, required by one
/// skill, or requested by one runtime turn.
pub const MAX_CAPABILITY_KEYS: usize = 64;

/// Maximum number of models exposed by one model provider.
pub const MAX_MODELS_PER_PROVIDER: usize = 64;

/// Maximum number of environments exposed by one execution provider.
pub const MAX_ENVIRONMENTS_PER_PROVIDER: usize = 64;

/// Maximum length of a runtime outcome summary.
pub const MAX_SUMMARY_BYTES: usize = 256;

/// Contract-level validation error for values that fail canonical rules.
///
/// Store operations surface these through
/// [`ExecStoreError::Invalid`](store::ExecStoreError).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    /// A canonical identifier is malformed (see [`IdError`] for the frozen
    /// failure reasons).
    Id(IdError),
    /// A value violates a canonical rule (bounds, ordering, grammar).
    Invalid(String),
}

impl ExecError {
    /// Builds an [`ExecError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl fmt::Display for ExecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(error) => error.fmt(formatter),
            Self::Invalid(reason) => write!(formatter, "invalid execution state: {reason}"),
        }
    }
}

impl Error for ExecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Id(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<IdError> for ExecError {
    fn from(error: IdError) -> Self {
        Self::Id(error)
    }
}

/// The F2 contract schema version marker. Serializes as `"v": 1` and
/// rejects any other value, so a document written by a different schema
/// version fails canonical reads instead of being silently misread.
///
/// A schema change to a contract type bumps its `v`; at that point the
/// affected types move to a new marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContractVersion;

impl serde::Serialize for ContractVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(1)
    }
}

impl<'de> serde::Deserialize<'de> for ContractVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = u32::deserialize(deserializer)?;
        if version == 1 {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported contract schema version {version}; this build reads v1"
            )))
        }
    }
}

/// Validates that a machine label (provider kind, runtime kind) is a
/// non-empty, bounded lowercase token: `[a-z][a-z0-9-]*` (for example
/// `codex-app-server`, `openai`, `e2b`, `flauz-direct`).
pub(crate) fn ensure_kind_label(field: &'static str, value: &str) -> Result<(), ExecError> {
    ensure_non_empty(field, value)?;
    ensure_str_bound(field, value, MAX_NAME_BYTES)?;
    let valid = value
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        });
    if !valid {
        return Err(ExecError::invalid(format!(
            "{field} must be a lowercase token (`[a-z][a-z0-9-]*`), found {value:?}"
        )));
    }
    Ok(())
}

pub(crate) fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), ExecError> {
    if value.is_empty() {
        Err(ExecError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_str_bound(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), ExecError> {
    if value.len() > max_bytes {
        Err(ExecError::invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_list_bound<T>(
    field: &'static str,
    list: &[T],
    max_items: usize,
) -> Result<(), ExecError> {
    if list.len() > max_items {
        Err(ExecError::invalid(format!(
            "{field} exceeds {max_items} entries"
        )))
    } else {
        Ok(())
    }
}

/// Validates that a capability list is bounded, syntactically valid,
/// deduplicated and canonically ordered. Advertisement lists are canonical
/// state: sorted ascending, no duplicates.
pub(crate) fn ensure_capability_list(
    field: &'static str,
    keys: &[CapabilityId],
) -> Result<(), ExecError> {
    ensure_list_bound(field, keys, MAX_CAPABILITY_KEYS)?;
    for key in keys {
        key.validate()?;
    }
    if keys.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ExecError::invalid(format!(
            "{field} must be sorted and free of duplicates"
        )));
    }
    Ok(())
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

    #[test]
    fn contract_version_serializes_as_one_and_rejects_others() {
        let serialized = ok(serde_json::to_string(&ContractVersion));
        assert_eq!(serialized, "1");
        assert!(serde_json::from_str::<ContractVersion>("1").is_ok());
        assert!(serde_json::from_str::<ContractVersion>("2").is_err());
        assert!(serde_json::from_str::<ContractVersion>("0").is_err());
    }

    #[test]
    fn kind_labels_are_lowercase_tokens() {
        assert!(ensure_kind_label("provider kind", "codex-app-server").is_ok());
        assert!(ensure_kind_label("provider kind", "e2b").is_ok());
        assert!(ensure_kind_label("runtime kind", "flauz-direct").is_ok());
        assert!(ensure_kind_label("provider kind", "OpenAI").is_err());
        assert!(ensure_kind_label("provider kind", "").is_err());
        assert!(ensure_kind_label("runtime kind", "flauz runtime").is_err());
        assert!(ensure_kind_label("provider kind", "-leading").is_err());
    }

    #[test]
    fn registered_event_types_follow_the_grammar() {
        for name in [
            event_types::ENVIRONMENT_ATTACHED,
            event_types::ENVIRONMENT_DETACHED,
            event_types::MODEL_REGISTERED,
            event_types::AGENT_REGISTERED,
            event_types::CONNECTION_CONNECTED,
            event_types::CONNECTION_DISCONNECTED,
        ] {
            let Some((entity, verb)) = name.split_once('.') else {
                panic!("{name} must contain exactly one `.`");
            };
            assert!(!verb.contains('.'), "{name} must contain exactly one `.`");
            for segment in [entity, verb] {
                let mut characters = segment.chars();
                let valid = characters
                    .next()
                    .is_some_and(|first| first.is_ascii_lowercase())
                    && characters.all(|character| {
                        character.is_ascii_lowercase()
                            || character.is_ascii_digit()
                            || character == '_'
                    });
                assert!(valid, "{name} segments must be `[a-z][a-z0-9_]*`");
            }
        }
    }
}
