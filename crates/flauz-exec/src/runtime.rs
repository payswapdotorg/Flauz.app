//! The [`AgentRuntime`] contract: the orchestration loop around a model
//! and its tools.
//!
//! First-class runtime targets (constitution): the official Codex
//! app-server, the GitHub Copilot runtime/SDK, the Flauz direct-model
//! runtime, future/custom runtimes. **The official Codex app-server is ONE
//! runtime — never the universal Flauz registry.** Runtimes are identified
//! by neutral kind labels (for example `codex-app-server`,
//! `flauz-direct`), referenced at most once per
//! [`Agent`](crate::Agent), and every runtime — Codex or not — satisfies
//! this same contract (kernel §9: one conformance test runs against both
//! the fake Codex runtime and the fake non-Codex runtime).
//!
//! Capability advertisement (kernel §7): every runtime advertises the
//! capabilities it offers. When a [`RuntimeRequest`] requires capabilities
//! the runtime does not advertise, the runtime reports a
//! [`RuntimeStatus::CapabilityGap`] carrying exactly the missing keys —
//! an actionable diagnosis surface for later waves. Availability is the
//! intersection `model ∩ runtime ∩ environment ∩ permissions ∩ policy`;
//! computing that intersection (and any unlock path) is future resolution
//! work — this contract only reports the runtime's own side honestly.
//!
//! Model/runtime separation is structural here: the runtime *receives* a
//! model reference inside [`RuntimeRequest`] as plain data and echoes it
//! back; it never owns, determines or stores models.

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;
use crate::ids::{EnvironmentId, ModelId};
use crate::{
    ContractVersion, ExecError, MAX_STATEMENT_BYTES, MAX_SUMMARY_BYTES, ensure_capability_list,
    ensure_kind_label, ensure_non_empty, ensure_str_bound,
};

/// One neutral orchestration request: the model to drive, where execution
/// happens, the capabilities the turn requires, and a bounded instruction.
/// Model choice and environment choice are execution state (kernel §6) —
/// they ride on requests, never on identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRequest {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The model the orchestration loop drives for this turn.
    pub model_id: ModelId,
    /// The environment where execution happens for this turn, when one is
    /// bound. `None` means the turn runs without an attached environment.
    pub environment_id: Option<EnvironmentId>,
    /// The capabilities this turn requires (sorted, deduplicated). The
    /// runtime reports a capability gap when it does not advertise them.
    pub required_capabilities: Vec<CapabilityId>,
    /// A bounded instruction for the turn.
    pub instruction: String,
}

impl RuntimeRequest {
    /// Builds a new runtime request.
    pub fn new(
        model_id: ModelId,
        environment_id: Option<EnvironmentId>,
        required_capabilities: Vec<CapabilityId>,
        instruction: &str,
    ) -> Result<Self, ExecError> {
        ensure_non_empty("runtime instruction", instruction)?;
        ensure_str_bound("runtime instruction", instruction, MAX_STATEMENT_BYTES)?;
        ensure_capability_list("required capabilities", &required_capabilities)?;
        Ok(Self {
            v: ContractVersion,
            model_id,
            environment_id,
            required_capabilities,
            instruction: instruction.to_owned(),
        })
    }

    /// Validates the request.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(
            self.model_id.clone(),
            self.environment_id.clone(),
            self.required_capabilities.clone(),
            &self.instruction,
        )?;
        Ok(())
    }
}

/// The outcome status of one orchestration turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    /// The turn completed.
    Completed,
    /// The turn could not run because the runtime does not advertise the
    /// required capabilities; `missing_capabilities` on the outcome
    /// carries the gap.
    CapabilityGap,
}

/// The neutral outcome of one orchestration turn: which runtime produced
/// it, which model it drove, where execution was bound, whether it
/// completed or hit a capability gap, and a bounded deterministic summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeOutcome {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The neutral kind label of the runtime that produced this outcome.
    pub runtime_kind: String,
    /// The model the loop drove (echoed from the request).
    pub model_id: ModelId,
    /// The environment execution was bound to (echoed from the request).
    pub environment_id: Option<EnvironmentId>,
    /// The turn status.
    pub status: RuntimeStatus,
    /// Exactly the required capabilities the runtime does not advertise;
    /// empty unless the status is
    /// [`RuntimeStatus::CapabilityGap`].
    pub missing_capabilities: Vec<CapabilityId>,
    /// A bounded, deterministic summary of the turn.
    pub summary: String,
}

impl RuntimeOutcome {
    /// Builds a completed outcome.
    pub fn completed(
        runtime_kind: &str,
        model_id: ModelId,
        environment_id: Option<EnvironmentId>,
        summary: &str,
    ) -> Result<Self, ExecError> {
        ensure_kind_label("runtime kind", runtime_kind)?;
        ensure_non_empty("runtime summary", summary)?;
        ensure_str_bound("runtime summary", summary, MAX_SUMMARY_BYTES)?;
        Ok(Self {
            v: ContractVersion,
            runtime_kind: runtime_kind.to_owned(),
            model_id,
            environment_id,
            status: RuntimeStatus::Completed,
            missing_capabilities: Vec::new(),
            summary: summary.to_owned(),
        })
    }

    /// Builds a capability-gap outcome carrying exactly the missing keys
    /// (sorted, deduplicated, non-empty).
    pub fn capability_gap(
        runtime_kind: &str,
        model_id: ModelId,
        environment_id: Option<EnvironmentId>,
        missing: Vec<CapabilityId>,
        summary: &str,
    ) -> Result<Self, ExecError> {
        ensure_kind_label("runtime kind", runtime_kind)?;
        ensure_non_empty("runtime summary", summary)?;
        ensure_str_bound("runtime summary", summary, MAX_SUMMARY_BYTES)?;
        if missing.is_empty() {
            return Err(ExecError::invalid(
                "a capability gap must carry at least one missing capability",
            ));
        }
        ensure_capability_list("missing capabilities", &missing)?;
        Ok(Self {
            v: ContractVersion,
            runtime_kind: runtime_kind.to_owned(),
            model_id,
            environment_id,
            status: RuntimeStatus::CapabilityGap,
            missing_capabilities: missing,
            summary: summary.to_owned(),
        })
    }

    /// Validates the outcome invariants: summaries are bounded, gap
    /// outcomes carry a non-empty missing list, completed outcomes carry an
    /// empty one.
    pub fn validate(&self) -> Result<(), ExecError> {
        ensure_kind_label("runtime kind", &self.runtime_kind)?;
        ensure_non_empty("runtime summary", &self.summary)?;
        ensure_str_bound("runtime summary", &self.summary, MAX_SUMMARY_BYTES)?;
        ensure_capability_list("missing capabilities", &self.missing_capabilities)?;
        match self.status {
            RuntimeStatus::Completed => {
                if !self.missing_capabilities.is_empty() {
                    return Err(ExecError::invalid(
                        "a completed outcome must not carry missing capabilities",
                    ));
                }
            }
            RuntimeStatus::CapabilityGap => {
                if self.missing_capabilities.is_empty() {
                    return Err(ExecError::invalid(
                        "a capability gap must carry at least one missing capability",
                    ));
                }
            }
        }
        Ok(())
    }
}

/// The orchestration loop around a model and its tools (constitution:
/// AgentRuntime/Harness). Provider-neutral: the official Codex app-server
/// is one implementing runtime among several, never the universal registry.
///
/// All methods are pure on the contract surface: implementations perform no
/// I/O and read no clock; fakes are deterministic.
pub trait AgentRuntime: Send + Sync {
    /// The neutral kind label of this runtime (for example
    /// `codex-app-server`, `flauz-direct`). Never a credential.
    fn runtime_kind(&self) -> &str;

    /// The capabilities this runtime offers (kernel advertisement rule):
    /// a sorted, deduplicated list of capability keys.
    fn capabilities(&self) -> &[CapabilityId];

    /// Executes one orchestration turn.
    ///
    /// The runtime validates the request; if the request's required
    /// capabilities are not a subset of the runtime's advertisement, the
    /// result is an [`RuntimeOutcome`] with status
    /// [`RuntimeStatus::CapabilityGap`] carrying exactly the missing keys
    /// (an actionable diagnosis, not an error). Contract violations of the
    /// request itself are [`ExecError`]s.
    fn execute(&self, request: &RuntimeRequest) -> Result<RuntimeOutcome, ExecError>;
}

/// Computes the required capabilities a runtime does not advertise, in
/// canonical (sorted) order. Shared by runtimes to build capability gaps.
pub(crate) fn missing_capabilities(
    advertised: &[CapabilityId],
    required: &[CapabilityId],
) -> Vec<CapabilityId> {
    required
        .iter()
        .filter(|required| !advertised.contains(required))
        .cloned()
        .collect()
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

    fn test_model_id() -> ModelId {
        ok(ModelId::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPX0"))
    }

    fn test_environment_id() -> EnvironmentId {
        ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4"))
    }

    #[test]
    fn request_round_trips_and_validates() {
        let request = ok(RuntimeRequest::new(
            test_model_id(),
            Some(test_environment_id()),
            vec![ok(CapabilityId::parse("terminal"))],
            "Run the test suite and report failures",
        ));
        let serialized = ok(serde_json::to_string(&request));
        let parsed: RuntimeRequest = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, request);
        assert!(RuntimeRequest::new(test_model_id(), None, vec![], "").is_err());
        assert!(
            serde_json::from_str::<RuntimeRequest>(&serialized.replace("\"v\":1,", "")).is_err()
        );
    }

    #[test]
    fn outcomes_serialize_their_status_and_gap() {
        let completed = ok(RuntimeOutcome::completed(
            "codex-app-server",
            test_model_id(),
            Some(test_environment_id()),
            "codex-app-server completed one orchestration step",
        ));
        let serialized = ok(serde_json::to_string(&completed));
        assert!(serialized.contains("\"status\":\"completed\""));
        assert!(serialized.contains("\"missing_capabilities\":[]"));
        let parsed: RuntimeOutcome = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, completed);
        ok(completed.validate());

        let gap = ok(RuntimeOutcome::capability_gap(
            "flauz-direct",
            test_model_id(),
            None,
            vec![ok(CapabilityId::parse("mcp"))],
            "flauz-direct lacks mcp",
        ));
        let gap_serialized = ok(serde_json::to_string(&gap));
        assert!(gap_serialized.contains("\"status\":\"capability_gap\""));
        assert!(gap_serialized.contains("\"missing_capabilities\":[\"mcp\"]"));
        let gap_parsed: RuntimeOutcome = ok(serde_json::from_str(&gap_serialized));
        assert_eq!(gap_parsed, gap);
        ok(gap.validate());
    }

    #[test]
    fn gap_outcomes_require_missing_capabilities() {
        assert!(
            RuntimeOutcome::capability_gap(
                "flauz-direct",
                test_model_id(),
                None,
                vec![],
                "nothing missing"
            )
            .is_err()
        );
        let mut poisoned = ok(RuntimeOutcome::completed(
            "flauz-direct",
            test_model_id(),
            None,
            "done",
        ));
        poisoned.missing_capabilities = vec![ok(CapabilityId::parse("mcp"))];
        assert!(poisoned.validate().is_err());
    }

    #[test]
    fn missing_capabilities_subtracts_advertised_from_required() {
        let advertised = [
            ok(CapabilityId::parse("filesystem.read")),
            ok(CapabilityId::parse("terminal")),
        ];
        // Canonical capability lists are sorted (validated on requests).
        let required = [
            ok(CapabilityId::parse("browser.input")),
            ok(CapabilityId::parse("mcp")),
            ok(CapabilityId::parse("terminal")),
        ];
        let missing = missing_capabilities(&advertised, &required);
        assert_eq!(
            missing,
            vec![
                ok(CapabilityId::parse("browser.input")),
                ok(CapabilityId::parse("mcp"))
            ]
        );
    }
}
