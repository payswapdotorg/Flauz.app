//! The per-model context compilation profile (architecture §13:
//! model-aware context compilation).
//!
//! A [`ModelContextProfile`] records how the Context Engine compiles the
//! same durable task state differently per model: the model's context
//! capacity, its multimodal behavior, and how tool schemas are handled. It
//! is keyed by the [`ModelRef`] it profiles — model identity is execution
//! state, never task identity (kernel §6), and switching models must not
//! silently discard durable task state.
//!
//! The profile is durable, mutable configuration: it carries a `version`
//! with the kernel §3 rules (start at 1, +1 per durable mutation, optimistic
//! concurrency through the store).

use serde::{Deserialize, Serialize};

use crate::ids::ModelRef;
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{ContextError, ContextVersion, MAX_MODEL_TOKENS, MIN_MODEL_TOKENS};

/// How a model handles non-text inputs (architecture §13 multimodal
/// behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultimodalBehavior {
    /// The model accepts text only.
    TextOnly,
    /// The model accepts images as input.
    ImageInput,
    /// The model accepts images as input and produces them as output.
    ImageInputAndOutput,
}

/// How tool schemas are presented in compiled context (architecture §11:
/// tool discovery, lazy schema loading, bounded tool-result retention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolSchemaHandling {
    /// All tool schemas are inlined in full.
    InlineFullSchemas,
    /// Tool schemas are loaded lazily, one tool at a time.
    LazyPerTool,
    /// Compact tool summaries are inlined; full schemas load lazily.
    SummariesWithLazySchemas,
}

/// The per-model context compilation profile, keyed by the model reference
/// it profiles (architecture §13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelContextProfile {
    /// Contract schema version (`"v": 1`).
    pub v: ContextVersion,
    /// The model this profile compiles for. A profile is keyed by its
    /// model reference.
    pub model_id: ModelRef,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// The model's context capacity, in tokens. Integers only — canonical
    /// state contains no floats (kernel §4).
    pub context_capacity_tokens: u64,
    /// The model's multimodal behavior.
    pub multimodal: MultimodalBehavior,
    /// How tool schemas are presented in compiled context.
    pub tool_schema: ToolSchemaHandling,
    /// The actor that recorded the profile.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl ModelContextProfile {
    /// Builds a new profile at version 1 for `model_id`.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::Invalid`] when the capacity is outside the
    /// canonical bounds.
    pub fn new(
        model_id: ModelRef,
        context_capacity_tokens: u64,
        multimodal: MultimodalBehavior,
        tool_schema: ToolSchemaHandling,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ContextError> {
        if !(MIN_MODEL_TOKENS..=MAX_MODEL_TOKENS).contains(&context_capacity_tokens) {
            return Err(ContextError::invalid(format!(
                "context capacity {context_capacity_tokens} tokens is outside the canonical \
                 bounds {MIN_MODEL_TOKENS}..={MAX_MODEL_TOKENS}"
            )));
        }
        Ok(Self {
            v: ContextVersion,
            model_id,
            version: 1,
            context_capacity_tokens,
            multimodal,
            tool_schema,
            created_by,
            created_at,
        })
    }

    /// Validates the profile.
    pub fn validate(&self) -> Result<(), ContextError> {
        Self::new(
            self.model_id.clone(),
            self.context_capacity_tokens,
            self.multimodal,
            self.tool_schema,
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(ContextError::invalid(
                "model context profile version must be at least 1",
            ));
        }
        Ok(())
    }
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
    fn profile_round_trips_and_is_keyed_by_its_model() {
        let profile = ok(ModelContextProfile::new(
            ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE")),
            200_000,
            MultimodalBehavior::ImageInput,
            ToolSchemaHandling::SummariesWithLazySchemas,
            ok(ActorRef::system("flauz-fake")),
            ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        ));
        let serialized = ok(serde_json::to_string(&profile));
        assert_eq!(
            serialized,
            "{\"v\":1,\
             \"model_id\":\"model_01J8ZQ5V8K3T2B7N6X4R9DQPRE\",\
             \"version\":1,\
             \"context_capacity_tokens\":200000,\
             \"multimodal\":\"image_input\",\
             \"tool_schema\":\"summaries_with_lazy_schemas\",\
             \"created_by\":{\"kind\":\"system\",\"id\":\"flauz-fake\"},\
             \"created_at\":\"2026-09-21T13:45:00Z\"}"
        );
        let reloaded: ModelContextProfile = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, profile);
        assert!(
            serde_json::from_str::<ModelContextProfile>(
                &serialized.replace("\"summaries_with_lazy_schemas\"", "\"everything\"")
            )
            .is_err()
        );
    }

    #[test]
    fn profile_rejects_out_of_bounds_capacity() {
        assert!(
            ModelContextProfile::new(
                ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE")),
                MIN_MODEL_TOKENS.saturating_sub(1),
                MultimodalBehavior::TextOnly,
                ToolSchemaHandling::InlineFullSchemas,
                ok(ActorRef::system("flauz-fake")),
                ok(Timestamp::parse("2026-09-21T13:45:00Z")),
            )
            .is_err()
        );
        assert!(
            ModelContextProfile::new(
                ok(ModelRef::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPRE")),
                MAX_MODEL_TOKENS + 1,
                MultimodalBehavior::TextOnly,
                ToolSchemaHandling::InlineFullSchemas,
                ok(ActorRef::system("flauz-fake")),
                ok(Timestamp::parse("2026-09-21T13:45:00Z")),
            )
            .is_err()
        );
    }
}
