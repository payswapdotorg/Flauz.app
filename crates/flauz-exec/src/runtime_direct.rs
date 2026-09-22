//! The first non-Codex runtime (RT-001): the direct in-process runtime.
//!
//! The `flauz-direct` runtime drives models directly inside the Flauz
//! process — no external orchestration server, no app-server boundary.
//! In this wave it drives the deterministic fake models (real model
//! integrations arrive with their own waves); its own machinery — request
//! validation, model resolution, honest capability advertisement,
//! deterministic bounded summaries — is the real shape every later
//! direct-model runtime keeps. It satisfies the SAME
//! [`AgentRuntime`] conformance suite as the Codex adapter
//! (`runtime_codex`), proving provider neutrality of the contract.
//!
//! # Honest capability advertisement
//!
//! The in-process loop offers **no tool surface in this wave**:
//! [`AgentRuntime::capabilities`] is deliberately empty. Tool
//! capabilities (terminal, filesystem, browser, …) require environment
//! wiring that later waves add; until the runtime genuinely offers one it
//! advertises none, and every turn that requires a tool capability is
//! answered with a [`RuntimeStatus::CapabilityGap`] naming exactly the
//! missing keys — a capability the runtime lacks is REPORTED, never
//! silently omitted. Plain model turns (no required capabilities)
//! complete: the runtime drives the model for the turn.
//!
//! Model choice is execution state, never identity (kernel §6): the
//! model arrives by reference inside every [`RuntimeRequest`] and is
//! echoed back; switching models changes no identity anywhere.

use crate::capability::CapabilityId;
use crate::ids::ModelId;
use crate::model::{Model, ModelProvider};
use crate::runtime::{AgentRuntime, RuntimeOutcome, RuntimeRequest, missing_capabilities};
use crate::{ExecError, MAX_MODELS_PER_PROVIDER, MAX_SUMMARY_BYTES, ensure_list_bound};

/// The direct in-process runtime: drives models without any external
/// orchestration server. Deterministic, no I/O; the models it drives in
/// this wave are the fake models (the fakes surface), giving the F2 gate
/// a real non-Codex runtime shape.
#[derive(Debug, Clone)]
pub struct DirectRuntime {
    models: Vec<Model>,
}

impl DirectRuntime {
    /// The direct runtime driving the deterministic fake models (the
    /// `flauz-direct` runtime of the F2 gate; real models arrive with
    /// later waves).
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the fake models fail their own
    /// canonical construction.
    pub fn new() -> Result<Self, ExecError> {
        Self::from_models(crate::fakes::FakeModelProvider::new()?.models().to_vec())
    }

    /// Builds the runtime from an explicit model table: the models it can
    /// drive, in canonical ID order, at least one.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the list is empty or
    /// unbounded, a model fails validation, or the IDs are unsorted or
    /// duplicated.
    pub fn from_models(models: Vec<Model>) -> Result<Self, ExecError> {
        ensure_list_bound("direct runtime models", &models, MAX_MODELS_PER_PROVIDER)?;
        if models.is_empty() {
            return Err(ExecError::invalid(
                "a direct runtime needs at least one model to drive",
            ));
        }
        for model in &models {
            model.validate()?;
        }
        if models.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(ExecError::invalid(
                "direct runtime models must be sorted by canonical ID and free of duplicates",
            ));
        }
        Ok(Self { models })
    }

    /// The models this runtime can drive, in canonical ID order.
    #[must_use]
    pub fn models(&self) -> &[Model] {
        &self.models
    }

    /// Looks up a drivable model by canonical ID.
    #[must_use]
    pub fn model(&self, id: &ModelId) -> Option<Model> {
        self.models.iter().find(|model| &model.id == id).cloned()
    }
}

impl AgentRuntime for DirectRuntime {
    fn runtime_kind(&self) -> &str {
        "flauz-direct"
    }

    fn capabilities(&self) -> &[CapabilityId] {
        // The honest advertisement: the in-process loop offers no tool
        // surface in this wave. Every tool capability a turn requires is
        // reported as a gap — never silently omitted.
        &[]
    }

    fn execute(&self, request: &RuntimeRequest) -> Result<RuntimeOutcome, ExecError> {
        request.validate()?;
        let Some(model) = self
            .models
            .iter()
            .find(|model| model.id == request.model_id)
        else {
            return Err(ExecError::invalid(format!(
                "flauz-direct has no model {} to drive",
                request.model_id
            )));
        };
        let missing = missing_capabilities(&[], &request.required_capabilities);
        if !missing.is_empty() {
            return RuntimeOutcome::capability_gap(
                self.runtime_kind(),
                request.model_id.clone(),
                request.environment_id.clone(),
                missing,
                "flauz-direct offers no tool capabilities yet",
            );
        }
        // The in-process turn: the model is driven directly and the
        // runtime summarizes the turn deterministically.
        RuntimeOutcome::completed(
            self.runtime_kind(),
            request.model_id.clone(),
            request.environment_id.clone(),
            &direct_turn_summary(model),
        )
    }
}

/// The deterministic, bounded summary of one in-process turn: the
/// runtime's own report that it drove the model for the step.
fn direct_turn_summary(model: &Model) -> String {
    const PREFIX: &str = "flauz-direct drove ";
    const SUFFIX: &str = " for one in-process step";
    let budget = MAX_SUMMARY_BYTES - PREFIX.len() - SUFFIX.len();
    format!(
        "{PREFIX}{}{SUFFIX}",
        truncate_on_char_boundary(&model.name, budget)
    )
}

/// Truncates to at most `max_bytes` on a UTF-8 character boundary.
fn truncate_on_char_boundary(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{FakeModelProvider, fake_model_ids};
    use crate::ids::EnvironmentId;
    use crate::runtime::RuntimeStatus;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn direct_runtime_satisfies_the_shared_agent_runtime_conformance() {
        let direct = ok(DirectRuntime::new());
        crate::runtime_codex::adapter_conformance::assert_agent_runtime_conformance(&direct);
        crate::runtime_codex::adapter_conformance::assert_every_capability_is_either_admitted_or_named(
            &direct,
        );
    }

    #[test]
    fn direct_runtime_advertises_no_tool_surface_honestly() {
        // The Wave-2 advertisement is deliberately EMPTY: the in-process
        // loop offers no tools yet, and every tool requirement is a named
        // gap — the honest advertisement, pinned here so it cannot
        // regress silently.
        let direct = ok(DirectRuntime::new());
        assert!(
            direct.capabilities().is_empty(),
            "flauz-direct must not advertise capabilities it cannot honor"
        );
        let request = ok(RuntimeRequest::new(
            ok(ModelId::parse(fake_model_ids::TEXT)),
            None,
            vec![ok(crate::capability::CapabilityId::parse("terminal"))],
            "Run the terminal probe",
        ));
        let gap = ok(direct.execute(&request));
        assert_eq!(gap.status, RuntimeStatus::CapabilityGap);
        assert_eq!(
            gap.missing_capabilities,
            vec![ok(crate::capability::CapabilityId::parse("terminal"))]
        );
        ok(gap.validate());
    }

    #[test]
    fn direct_runtime_drives_the_requested_model_deterministically() {
        let direct = ok(DirectRuntime::new());
        assert_eq!(direct.models().len(), 2);
        assert!(direct.models()[0].id < direct.models()[1].id);
        let vision_id = ok(ModelId::parse(fake_model_ids::VISION));
        assert_eq!(
            direct.model(&vision_id).map(|model| model.name),
            Some("Fake Vision Model".to_owned())
        );

        let request = ok(RuntimeRequest::new(
            vision_id.clone(),
            Some(ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4"))),
            Vec::new(),
            "Draft the release notes",
        ));
        let outcome = ok(direct.execute(&request));
        assert_eq!(outcome.status, RuntimeStatus::Completed);
        assert_eq!(outcome.runtime_kind, "flauz-direct");
        assert_eq!(outcome.model_id, vision_id);
        assert_eq!(
            outcome.environment_id,
            Some(ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4")))
        );
        assert!(outcome.summary.contains("Fake Vision Model"));
        ok(outcome.validate());

        // Determinism: the same turn produces the identical outcome.
        let again = ok(direct.execute(&request));
        assert_eq!(again, outcome);

        // A model the runtime does not have is a contract error, never a
        // fabricated outcome.
        let unknown = ok(ModelId::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPZ9"));
        assert!(
            direct
                .execute(&ok(RuntimeRequest::new(
                    unknown,
                    None,
                    Vec::new(),
                    "Run the step",
                )))
                .is_err()
        );
    }

    #[test]
    fn direct_runtime_requires_a_canonical_model_table() {
        assert!(DirectRuntime::from_models(Vec::new()).is_err());
        let models = ok(FakeModelProvider::new()).models().to_vec();
        let mut unsorted = vec![models[1].clone(), models[0].clone()];
        assert!(DirectRuntime::from_models(unsorted.clone()).is_err());
        unsorted[1] = unsorted[0].clone();
        assert!(DirectRuntime::from_models(unsorted).is_err());
        let ok_models = ok(FakeModelProvider::new()).models().to_vec();
        assert!(DirectRuntime::from_models(ok_models).is_ok());
    }

    #[test]
    fn direct_turn_summaries_stay_bounded_for_long_model_names() {
        let long_name = "V".repeat(MAX_SUMMARY_BYTES);
        let model = ok(Model::new(
            ok(ModelId::parse(fake_model_ids::VISION)),
            &long_name,
            "flauz-fake",
            Vec::new(),
            None,
            ok(crate::refs::ActorRef::user("alice")),
            ok(crate::time::Timestamp::parse("2026-09-21T13:45:00Z")),
        ));
        let summary = direct_turn_summary(&model);
        assert!(
            summary.len() <= MAX_SUMMARY_BYTES,
            "summaries must stay within {MAX_SUMMARY_BYTES} bytes"
        );
        assert!(summary.starts_with("flauz-direct drove "));
    }
}
