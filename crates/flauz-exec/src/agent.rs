//! The [`Agent`] entity: an agent identity, separate from environment
//! identity and from model identity (constitution: `Agent != Environment`).
//!
//! An Agent is a named execution principal registered against **at most
//! one** [`AgentRuntime`](crate::AgentRuntime), referenced by the runtime's
//! neutral kind label. The agent may additionally be bound to a current
//! [`Model`](crate::Model) — execution state, not identity: switching the
//! model or handing the agent off changes no ID and forks nothing (kernel
//! §6).
//!
//! The type deliberately carries **no environment state**: an agent is not
//! an environment, does not own one, and is not bound to one. Environments
//! are attached per execution turn
//! ([`RuntimeRequest::environment_id`](crate::RuntimeRequest)), never to
//! the agent identity.

use serde::{Deserialize, Serialize};

use crate::ids::{AgentId, ModelId};
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContractVersion, ExecError, MAX_NAME_BYTES, ensure_kind_label, ensure_non_empty,
    ensure_str_bound,
};

/// An agent: a named execution principal referencing at most one runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical agent ID (`agent_<ULID>`) — distinct from `env_`
    /// environment IDs and `model_` model IDs by kind prefix.
    pub id: AgentId,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// Human-readable agent name.
    pub name: String,
    /// The neutral kind label of the runtime this agent references (at
    /// most one), for example `codex-app-server` or `flauz-direct`. `None`
    /// while the agent is unassigned.
    pub runtime_kind: Option<String>,
    /// The model this agent currently drives, when bound. Execution state,
    /// never identity: model switches are events, not identity changes.
    pub model_id: Option<ModelId>,
    /// The actor that registered this agent.
    pub created_by: ActorRef,
    /// Registration timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Agent {
    /// Builds a new agent at version 1, referencing at most one runtime and
    /// optionally bound to a current model. No environment is required or
    /// accepted — an agent is never an environment.
    pub fn new(
        id: AgentId,
        name: &str,
        runtime_kind: Option<&str>,
        model_id: Option<ModelId>,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ExecError> {
        ensure_non_empty("agent name", name)?;
        ensure_str_bound("agent name", name, MAX_NAME_BYTES)?;
        if let Some(runtime_kind) = runtime_kind {
            ensure_kind_label("agent runtime kind", runtime_kind)?;
        }
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            name: name.to_owned(),
            runtime_kind: runtime_kind.map(str::to_owned),
            model_id,
            created_by,
            created_at,
        })
    }

    /// Validates the agent record.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(
            self.id.clone(),
            &self.name,
            self.runtime_kind.as_deref(),
            self.model_id.clone(),
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(ExecError::invalid("agent version must be at least 1"));
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

    fn test_actor() -> ActorRef {
        ok(ActorRef::user("alice"))
    }

    fn test_timestamp() -> Timestamp {
        ok(Timestamp::from_unix_seconds(1_789_998_300))
    }

    #[test]
    fn agent_serializes_with_at_most_one_runtime_and_no_environment() {
        let agent = ok(Agent::new(
            ok(AgentId::parse("agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6")),
            "Reconciliation agent",
            Some("codex-app-server"),
            Some(ok(ModelId::parse("model_01J8ZQ5V8K3T2B7N6X4R9DQPX0"))),
            test_actor(),
            test_timestamp(),
        ));
        let serialized = ok(serde_json::to_string(&agent));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,\"id\":\"agent_01J8ZQ5V8K3T2B7N6X4R9DQPT6\",\"version\":1,",
                "\"name\":\"Reconciliation agent\",\"runtime_kind\":\"codex-app-server\",",
                "\"model_id\":\"model_01J8ZQ5V8K3T2B7N6X4R9DQPX0\",",
                "\"created_by\":{\"kind\":\"user\",\"id\":\"alice\"},",
                "\"created_at\":\"2026-09-21T13:45:00Z\"}"
            )
        );
        let parsed: Agent = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, agent);
        // An agent carrying environment state is not even representable:
        // the unknown field `environment_id` must be rejected.
        assert!(
            serde_json::from_str::<Agent>(&serialized.replace(
                "\"runtime_kind\":\"codex-app-server\",",
                "\"runtime_kind\":\"codex-app-server\",\"environment_id\":null,"
            ))
            .is_err()
        );
    }

    #[test]
    fn agent_rejects_bad_runtime_kinds() {
        assert!(
            Agent::new(
                AgentId::generate(),
                "A",
                Some("Codex App Server"),
                None,
                test_actor(),
                test_timestamp()
            )
            .is_err()
        );
        assert!(
            Agent::new(
                AgentId::generate(),
                "",
                None,
                None,
                test_actor(),
                test_timestamp()
            )
            .is_err()
        );
    }
}
