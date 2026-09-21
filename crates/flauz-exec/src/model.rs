//! The intelligence contracts: the [`Model`] entity and the
//! [`ModelProvider`] that sources models.
//!
//! A Model is **intelligence** — what a model *is*, not where it runs or
//! who orchestrates it. The separation is structural and tested
//! (constitution, kernel):
//!
//! - A [`Model`] contains no provider-specific environment behavior: there
//!   is no environment, locality, connection or runtime state anywhere in
//!   the type. `provider_kind` is sourcing provenance (which kind of
//!   provider offers this model), never behavior.
//! - An [`Environment`](crate::Environment) does not determine a Model:
//!   the environment module never references this one, and no function in
//!   this crate derives models from environments.
//! - A Model is not bound to an [`AgentRuntime`](crate::AgentRuntime):
//!   runtimes receive models as values at execution time
//!   ([`RuntimeRequest`](crate::RuntimeRequest)); switching the model
//!   changes no identity.
//!
//! Model-provider targets (constitution): OpenAI/Codex, OpenAI API, Google
//! Gemini, Anthropic, GitHub Copilot/BYOK, Ollama, LM Studio,
//! OpenAI-compatible custom endpoints, OpenRouter, future adapters. The
//! official Codex app-server is one *runtime*, never the universal Flauz
//! model registry (constitution) — sourcing models is the provider's job,
//! and this trait is that contract.

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;
use crate::ids::ModelId;
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContractVersion, ExecError, MAX_NAME_BYTES, MAX_STATEMENT_BYTES, ensure_capability_list,
    ensure_kind_label, ensure_non_empty, ensure_str_bound,
};

/// A model: intelligence, sourced from a provider. The canonical record of
/// what a model **is** — identity, display name, sourcing provenance and
/// the capabilities the model itself offers (for example `vision`,
/// `image.input`).
///
/// The type deliberately carries **no** environment, locality, runtime,
/// connection or execution state: environment choice and model choice are
/// execution state (kernel §6), never model identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical model ID (`model_<ULID>`), opaque and never encoding
    /// provider or runtime.
    pub id: ModelId,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// Human-readable model name.
    pub name: String,
    /// The neutral kind label of the provider that sources this model (for
    /// example `openai`, `anthropic`, `ollama`). Provenance metadata only:
    /// which provider offers the model, never how it executes.
    pub provider_kind: String,
    /// The capabilities this model offers (sorted, deduplicated): model
    /// abilities such as `vision` or `image.input`.
    pub capabilities: Vec<CapabilityId>,
    /// A human-readable description of the model.
    pub description: Option<String>,
    /// The actor that registered this model.
    pub created_by: ActorRef,
    /// Registration timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Model {
    /// Builds a new model record at version 1. No environment, runtime or
    /// connection is required or accepted — a model exists independently
    /// of where it runs and who orchestrates it.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ModelId,
        name: &str,
        provider_kind: &str,
        capabilities: Vec<CapabilityId>,
        description: Option<&str>,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ExecError> {
        ensure_non_empty("model name", name)?;
        ensure_str_bound("model name", name, MAX_NAME_BYTES)?;
        ensure_kind_label("model provider kind", provider_kind)?;
        ensure_capability_list("model capabilities", &capabilities)?;
        if let Some(description) = description {
            ensure_non_empty("model description", description)?;
            ensure_str_bound("model description", description, MAX_STATEMENT_BYTES)?;
        }
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            name: name.to_owned(),
            provider_kind: provider_kind.to_owned(),
            capabilities,
            description: description.map(str::to_owned),
            created_by,
            created_at,
        })
    }

    /// Validates the model record.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(
            self.id.clone(),
            &self.name,
            &self.provider_kind,
            self.capabilities.clone(),
            self.description.as_deref(),
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(ExecError::invalid("model version must be at least 1"));
        }
        Ok(())
    }
}

/// A pluggable source of models (constitution: providers are pluggable
/// infrastructure sources; model-provider targets include OpenAI, Gemini,
/// Anthropic, Copilot/BYOK, Ollama, LM Studio, OpenAI-compatible endpoints
/// and OpenRouter, plus future adapters).
///
/// Capability advertisement (kernel §7): the provider advertises the
/// capabilities its models collectively offer. Users can bring their own
/// provider accounts through a
/// [`ProviderConnection`](crate::ProviderConnection).
pub trait ModelProvider: Send + Sync {
    /// The neutral kind label of this provider (for example `openai`,
    /// `ollama`). Never a credential.
    fn provider_kind(&self) -> &str;

    /// The capabilities offered by models of this provider (the union of
    /// its models' capabilities).
    fn capabilities(&self) -> &[CapabilityId];

    /// The user-owned connection through which this provider is accessed,
    /// where provider quota belongs to the connection. `None` for
    /// connection-less or BYOK-style providers.
    fn connection_id(&self) -> Option<&crate::ProviderConnectionId>;

    /// The models sourced from this provider (bounded, in canonical ID
    /// order).
    fn models(&self) -> &[Model];

    /// Looks up a sourced model by its canonical ID.
    fn model(&self, id: &ModelId) -> Option<Model>;
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

    #[test]
    fn model_serializes_without_environment_or_runtime_state() {
        let model = ok(Model::new(
            ok(ModelId::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPX0")),
            "Fake Vision Model",
            "flauz-fake",
            vec![ok(CapabilityId::parse("vision"))],
            None,
            test_actor(),
            test_timestamp(),
        ));
        let serialized = ok(serde_json::to_string(&model));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,\"id\":\"model_01J8ZQ5V8K3T2B7N6X4R9DQPX0\",\"version\":1,",
                "\"name\":\"Fake Vision Model\",\"provider_kind\":\"flauz-fake\",",
                "\"capabilities\":[\"vision\"],\"description\":null,",
                "\"created_by\":{\"kind\":\"user\",\"id\":\"alice\"},",
                "\"created_at\":\"2026-09-21T13:45:00Z\"}"
            )
        );
        let parsed: Model = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, model);
    }

    #[test]
    fn model_rejects_bad_kinds_and_unsorted_capabilities() {
        assert!(
            Model::new(
                ModelId::generate(),
                "M",
                "OpenAI",
                vec![],
                None,
                test_actor(),
                test_timestamp()
            )
            .is_err()
        );
        let duplicated = vec![
            ok(CapabilityId::parse("vision")),
            ok(CapabilityId::parse("vision")),
        ];
        assert!(
            Model::new(
                ModelId::generate(),
                "M",
                "flauz-fake",
                duplicated,
                None,
                test_actor(),
                test_timestamp()
            )
            .is_err()
        );
        assert!(
            Model::new(
                ModelId::generate(),
                "",
                "flauz-fake",
                vec![],
                None,
                test_actor(),
                test_timestamp()
            )
            .is_err()
        );
    }
}
