//! The Codex adapter boundary (RT-001): the official Codex app-server
//! runtime mapped onto the frozen [`AgentRuntime`] contract.
//!
//! The official Codex app-server is ONE runtime among several — never the
//! universal Flauz model registry (constitution). This module is the
//! ADAPTER: it owns the contract side — request validation, capability
//! advertisement, honest gap reporting — and delegates the turn itself to
//! a [`CodexServerHandle`], the neutral boundary shape of one app-server
//! turn. **No live app-server dependency lives in this crate**: the live
//! bridge (a later wave) implements the handle over the supervised
//! app-server process; tests and the F2 gate drive it with the fake
//! handle (`fakes::FakeCodexServerHandle`).
//!
//! The mapping is deliberately thin:
//!
//! - a [`RuntimeRequest`] whose required capabilities the app-server
//!   surface does not advertise is answered with a
//!   [`RuntimeStatus::CapabilityGap`] naming exactly the missing keys —
//!   **before** any turn is sent: the server never sees a turn it cannot
//!   satisfy;
//! - a satisfiable request is mapped to a [`CodexServerTurn`] (the model
//!   the server drives plus the bounded instruction) and its result to a
//!   completed [`RuntimeOutcome`]; the environment binding is Flauz
//!   execution state (kernel §6) the adapter maps itself — the
//!   app-server turn shape carries only what the server needs;
//! - a server-side failure is an [`ExecError`], never a fake outcome.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;
use crate::ids::ModelId;
use crate::runtime::{AgentRuntime, RuntimeOutcome, RuntimeRequest, missing_capabilities};
use crate::{
    ContractVersion, ExecError, MAX_STATEMENT_BYTES, MAX_SUMMARY_BYTES, ensure_non_empty,
    ensure_str_bound,
};

/// One orchestration turn as the Codex app-server boundary sees it: the
/// model the server drives plus the bounded instruction to run. This is
/// the neutral boundary shape — the live bridge maps the actual
/// app-server protocol onto it, and nothing app-server-specific leaks
/// through it in either direction.
///
/// The environment binding is deliberately absent: it is Flauz execution
/// state (kernel §6) that the adapter maps itself when building the
/// outcome; the app-server turn carries only what the server itself
/// needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexServerTurn {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The model the app-server drives for this turn.
    pub model_id: ModelId,
    /// The bounded instruction for the turn.
    pub instruction: String,
}

impl CodexServerTurn {
    /// Builds a server turn.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the instruction is empty or
    /// unbounded.
    pub fn new(model_id: ModelId, instruction: &str) -> Result<Self, ExecError> {
        ensure_non_empty("app-server turn instruction", instruction)?;
        ensure_str_bound(
            "app-server turn instruction",
            instruction,
            MAX_STATEMENT_BYTES,
        )?;
        Ok(Self {
            v: ContractVersion,
            model_id,
            instruction: instruction.to_owned(),
        })
    }

    /// Validates the turn.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when a canonical rule is violated.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(self.model_id.clone(), &self.instruction)?;
        Ok(())
    }
}

/// The result of one app-server turn as the boundary reports it: a
/// bounded deterministic summary. Server-side failures are [`ExecError`]s
/// on the handle, never results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexServerTurnResult {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// A bounded, deterministic summary of the completed turn.
    pub summary: String,
}

impl CodexServerTurnResult {
    /// Builds a completed turn result.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the summary is empty or
    /// unbounded.
    pub fn completed(summary: &str) -> Result<Self, ExecError> {
        ensure_non_empty("app-server turn summary", summary)?;
        ensure_str_bound("app-server turn summary", summary, MAX_SUMMARY_BYTES)?;
        Ok(Self {
            v: ContractVersion,
            summary: summary.to_owned(),
        })
    }

    /// Validates the result.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when a canonical rule is violated.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::completed(&self.summary)?;
        Ok(())
    }
}

/// The app-server handle seam: what the adapter needs from the official
/// Codex app-server runtime. The live bridge (a later wave) implements
/// this over the supervised app-server process; fakes implement it for
/// tests and the F2 gate.
///
/// Implementations must not perform Flauz contract validation — the
/// adapter owns the contract side and only sends turns it has already
/// validated and gap-checked.
pub trait CodexServerHandle: Send + Sync {
    /// Runs one turn against the app-server.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError`] when the server-side turn fails; the adapter
    /// propagates it as a contract error.
    fn turn(&self, turn: &CodexServerTurn) -> Result<CodexServerTurnResult, ExecError>;
}

/// The Codex adapter: the official Codex app-server runtime satisfying
/// the frozen [`AgentRuntime`] contract through a [`CodexServerHandle`].
///
/// Advertises the app-server tool surface (terminal, filesystem, git).
/// Deterministic apart from the handle it delegates to; the fake handle
/// keeps the whole path deterministic for tests and the F2 gate.
pub struct CodexAppServerRuntime {
    capabilities: Vec<CapabilityId>,
    handle: Arc<dyn CodexServerHandle>,
}

impl CodexAppServerRuntime {
    /// Builds the adapter over a server handle.
    #[must_use]
    pub fn new(handle: Arc<dyn CodexServerHandle>) -> Self {
        Self {
            capabilities: vec![
                parse_capability("filesystem.read"),
                parse_capability("filesystem.write"),
                parse_capability("git"),
                parse_capability("terminal"),
            ],
            handle,
        }
    }

    /// The handle the adapter delegates turns to (tests and the gate
    /// inspect it for turn evidence).
    #[must_use]
    pub fn handle(&self) -> &Arc<dyn CodexServerHandle> {
        &self.handle
    }
}

fn parse_capability(key: &str) -> CapabilityId {
    match CapabilityId::parse(key) {
        Ok(capability) => capability,
        Err(error) => panic!("constant capability key {key:?} must always parse: {error}"),
    }
}

impl AgentRuntime for CodexAppServerRuntime {
    fn runtime_kind(&self) -> &str {
        "codex-app-server"
    }

    fn capabilities(&self) -> &[CapabilityId] {
        &self.capabilities
    }

    fn execute(&self, request: &RuntimeRequest) -> Result<RuntimeOutcome, ExecError> {
        request.validate()?;
        let missing = missing_capabilities(&self.capabilities, &request.required_capabilities);
        if !missing.is_empty() {
            // The gap is reported BEFORE any turn is sent: the server
            // never sees a turn it cannot satisfy (the honest-gap path).
            return RuntimeOutcome::capability_gap(
                self.runtime_kind(),
                request.model_id.clone(),
                request.environment_id.clone(),
                missing,
                "codex-app-server is missing required capabilities",
            );
        }
        let turn = CodexServerTurn::new(request.model_id.clone(), &request.instruction)?;
        let result = self.handle.turn(&turn)?;
        RuntimeOutcome::completed(
            self.runtime_kind(),
            request.model_id.clone(),
            request.environment_id.clone(),
            &result.summary,
        )
    }
}

#[cfg(test)]
pub(crate) mod adapter_conformance {
    //! The SHARED AgentRuntime conformance suite for the RT-001 adapters
    //! (kernel §9 discipline, addendum §1): `runtime_codex` and
    //! `runtime_direct` satisfy the SAME contract through these
    //! assertions — one suite, both adapters, no per-adapter branching.

    use crate::capability::CapabilityId;
    use crate::fakes::{FakeCodexServerHandle, fake_model_ids};
    use crate::ids::{EnvironmentId, ModelId};
    use crate::runtime::{AgentRuntime, RuntimeRequest, RuntimeStatus};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    /// The shared conformance: a valid kind label, a canonical
    /// advertisement, a satisfiable turn that completes and echoes the
    /// model, environment and runtime kind, an unsatisfiable turn that
    /// reports exactly the capability gap, and a structurally invalid
    /// request that is a contract error. `capabilities()` may honestly be
    /// EMPTY (see `runtime_direct`); the suite then conformance-checks
    /// the no-required-capability turn.
    pub(crate) fn assert_agent_runtime_conformance(runtime: &dyn AgentRuntime) {
        // A neutral lowercase kind label.
        let kind = runtime.runtime_kind();
        assert!(!kind.is_empty(), "the runtime kind must not be empty");
        assert!(
            kind.chars()
                .next()
                .is_some_and(|first| first.is_ascii_lowercase())
                && kind.chars().all(|character| character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || character == '-'),
            "runtime kind {kind:?} must be a lowercase token"
        );

        // The advertisement is canonical: valid keys, sorted, deduplicated.
        let capabilities = runtime.capabilities();
        for capability in capabilities {
            ok(capability.validate());
        }
        assert!(
            capabilities.windows(2).all(|pair| pair[0] < pair[1]),
            "{kind} capabilities must be sorted and deduplicated"
        );

        let model_id = ok(ModelId::parse(fake_model_ids::TEXT));

        // A satisfiable request completes and echoes the model, the
        // environment and the runtime kind.
        let required: Vec<CapabilityId> = capabilities.iter().take(2).cloned().collect();
        let request = ok(RuntimeRequest::new(
            model_id.clone(),
            Some(ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4"))),
            required,
            "Run the reconciliation step",
        ));
        let outcome = ok(runtime.execute(&request));
        assert_eq!(outcome.status, RuntimeStatus::Completed, "{kind}");
        assert_eq!(outcome.runtime_kind, kind);
        assert_eq!(outcome.model_id, model_id);
        assert_eq!(outcome.environment_id, request.environment_id);
        assert!(outcome.missing_capabilities.is_empty());
        ok(outcome.validate());
        // Outcomes are canonical serialized state: they round-trip.
        let serialized = ok(serde_json::to_string(&outcome));
        let reloaded: crate::RuntimeOutcome = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, outcome);

        // An unsatisfiable request produces exactly the capability gap —
        // the actionable diagnosis surface, not an error.
        let gap_request = ok(RuntimeRequest::new(
            model_id.clone(),
            None,
            vec![ok(CapabilityId::parse("mcp"))],
            "Talk to the MCP server",
        ));
        let gap = ok(runtime.execute(&gap_request));
        assert_eq!(gap.status, RuntimeStatus::CapabilityGap, "{kind}");
        assert_eq!(
            gap.missing_capabilities,
            vec![ok(CapabilityId::parse("mcp"))]
        );
        assert_eq!(gap.model_id, model_id);
        ok(gap.validate());

        // A structurally invalid request is a contract error, not an
        // outcome.
        let invalid = RuntimeRequest {
            v: crate::ContractVersion,
            model_id: model_id.clone(),
            environment_id: None,
            required_capabilities: Vec::new(),
            instruction: String::new(),
        };
        assert!(runtime.execute(&invalid).is_err(), "{kind}");
    }

    /// The no-silent-omission property (addendum §4 discipline on the
    /// runtime side): for a representative vocabulary, every required
    /// capability the runtime does not advertise is NAMED in a gap — and
    /// every advertised one is honored by a completed turn. Nothing is
    /// quietly dropped from either side.
    pub(crate) fn assert_every_capability_is_either_admitted_or_named(runtime: &dyn AgentRuntime) {
        let kind = runtime.runtime_kind();
        let model_id = ok(ModelId::parse(fake_model_ids::TEXT));
        for key in [
            "terminal",
            "filesystem.read",
            "filesystem.write",
            "git",
            "browser.input",
            "browser.navigation",
            "web.search",
            "vision",
            "image.input",
            "mcp",
            "gpu",
        ] {
            let capability = ok(CapabilityId::parse(key));
            let request = ok(RuntimeRequest::new(
                model_id.clone(),
                None,
                vec![capability.clone()],
                "Probe one capability",
            ));
            if runtime.capabilities().contains(&capability) {
                let outcome = ok(runtime.execute(&request));
                assert_eq!(
                    outcome.status,
                    RuntimeStatus::Completed,
                    "{kind} advertises {key} and must honor it"
                );
                assert!(outcome.missing_capabilities.is_empty());
            } else {
                let outcome = ok(runtime.execute(&request));
                assert_eq!(
                    outcome.status,
                    RuntimeStatus::CapabilityGap,
                    "{kind} lacks {key} and must report it"
                );
                assert_eq!(
                    outcome.missing_capabilities,
                    vec![capability],
                    "{kind} must name exactly {key} — never silently omit it"
                );
            }
        }
    }

    /// Builds the fake-backed Codex adapter the shared suite runs against.
    pub(crate) fn fake_backed_codex_adapter() -> crate::runtime_codex::CodexAppServerRuntime {
        crate::runtime_codex::CodexAppServerRuntime::new(std::sync::Arc::new(
            FakeCodexServerHandle::new(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::fakes::{FakeCodexServerHandle, fake_model_ids};
    use crate::ids::EnvironmentId;
    use crate::runtime::RuntimeStatus;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn server_turns_and_results_round_trip_canonically() {
        let turn = ok(CodexServerTurn::new(
            ok(ModelId::parse(fake_model_ids::TEXT)),
            "Run the test suite and report failures",
        ));
        let serialized = ok(serde_json::to_string(&turn));
        let parsed: CodexServerTurn = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, turn);
        ok(turn.validate());
        assert!(
            serde_json::from_str::<CodexServerTurn>(&serialized.replace("\"v\":1,", "")).is_err()
        );
        assert!(CodexServerTurn::new(ok(ModelId::parse(fake_model_ids::TEXT)), "").is_err());

        let result = ok(CodexServerTurnResult::completed(
            "codex app-server completed one orchestration turn",
        ));
        let serialized = ok(serde_json::to_string(&result));
        let parsed: CodexServerTurnResult = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, result);
        ok(result.validate());
        assert!(CodexServerTurnResult::completed("").is_err());
    }

    #[test]
    fn codex_adapter_satisfies_the_shared_agent_runtime_conformance() {
        let adapter = adapter_conformance::fake_backed_codex_adapter();
        adapter_conformance::assert_agent_runtime_conformance(&adapter);
        adapter_conformance::assert_every_capability_is_either_admitted_or_named(&adapter);
    }

    #[test]
    fn codex_adapter_completes_turns_through_the_handle() {
        let handle = Arc::new(FakeCodexServerHandle::new());
        let adapter = CodexAppServerRuntime::new(handle.clone());
        let model_id = ok(ModelId::parse(fake_model_ids::VISION));
        let request = ok(RuntimeRequest::new(
            model_id.clone(),
            Some(ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4"))),
            vec![parse_capability("terminal")],
            "Run the test suite and report failures",
        ));
        let outcome = ok(adapter.execute(&request));
        assert_eq!(outcome.status, RuntimeStatus::Completed);
        assert_eq!(outcome.runtime_kind, "codex-app-server");
        assert_eq!(outcome.model_id, model_id);
        assert_eq!(
            outcome.environment_id,
            Some(ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4")))
        );

        // The boundary mapping is visible at the handle: exactly one turn
        // served, carrying the requested model and instruction — the
        // app-server shape the adapter maps onto.
        let served = handle.served_turns();
        assert_eq!(served.len(), 1);
        assert_eq!(served[0].model_id, outcome.model_id);
        assert_eq!(
            served[0].instruction,
            "Run the test suite and report failures"
        );
        // The summary the outcome carries came from the server side.
        assert_eq!(outcome.summary, served_summary());
    }

    #[test]
    fn codex_adapter_reports_gaps_without_contacting_the_server() {
        let handle = Arc::new(FakeCodexServerHandle::new());
        let adapter = CodexAppServerRuntime::new(handle.clone());
        let model_id = ok(ModelId::parse(fake_model_ids::TEXT));
        let request = ok(RuntimeRequest::new(
            model_id.clone(),
            None,
            vec![
                parse_capability("browser.input"),
                parse_capability("mcp"),
                parse_capability("terminal"),
            ],
            "Drive the browser and talk to the MCP server",
        ));
        let gap = ok(adapter.execute(&request));
        assert_eq!(gap.status, RuntimeStatus::CapabilityGap);
        assert_eq!(
            gap.missing_capabilities,
            vec![parse_capability("browser.input"), parse_capability("mcp")]
        );
        assert_eq!(gap.model_id, model_id);
        ok(gap.validate());
        // The honest-gap path: the server never saw the turn.
        assert!(
            handle.served_turns().is_empty(),
            "a gap turn must never reach the app-server"
        );
    }

    #[test]
    fn codex_adapter_propagates_server_failures() {
        struct TestFailingServerHandle;
        impl CodexServerHandle for TestFailingServerHandle {
            fn turn(&self, _turn: &CodexServerTurn) -> Result<CodexServerTurnResult, ExecError> {
                Err(ExecError::invalid("the app-server turn failed"))
            }
        }
        let adapter = CodexAppServerRuntime::new(Arc::new(TestFailingServerHandle));
        let request = ok(RuntimeRequest::new(
            ok(ModelId::parse(fake_model_ids::TEXT)),
            None,
            Vec::new(),
            "Run the reconciliation step",
        ));
        let error = adapter
            .execute(&request)
            .err()
            .unwrap_or_else(|| panic!("a server-side failure must surface as a contract error"));
        assert!(error.to_string().contains("failed"));
    }

    fn served_summary() -> &'static str {
        crate::fakes::FAKE_CODEX_TURN_SUMMARY
    }
}
