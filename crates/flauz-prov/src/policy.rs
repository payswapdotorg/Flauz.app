//! The [`RoutingPolicy`] — DATA, not an engine (work order PROV-001, part
//! 2): the ordering (free-tier-first by default), the per-provider and
//! per-task overrides, and the spend and concurrency limits.
//!
//! The policy is a record the user can **see and change**: every mutation
//! is a durable +1 version (kernel §3) and the scheduling surface states
//! the consequence of the current shape in plain words. There is no
//! policy engine here — the [`scheduler`](crate::scheduler) reads the
//! record as data and NAMES the rule that chose each account.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::key::{ConnectionRef, TaskRef};
use crate::{MAX_ACCOUNTS, ProvError, ProvVersion, ensure_kind_label};

/// The account ordering the router follows: which tier is tried first.
/// The default is free-tier-first — paid accounts are used only when the
/// policy (or the user) chooses them, or when free quota is used up (the
/// escalation is always named).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingOrder {
    /// Free accounts are tried first; a paid account is used only when no
    /// free account is usable (an escalation that is always named).
    FreeTierFirst,
    /// Paid accounts are tried first — the user's explicit choice, with
    /// the consequence stated on the surface: tasks spend paid quota even
    /// while free quota is left.
    PaidFirst,
}

impl RoutingOrder {
    /// The default routing order (free-tier-first).
    #[must_use]
    pub const DEFAULT: Self = Self::FreeTierFirst;

    /// The user-facing label for this order (plain words).
    #[must_use]
    pub const fn user_label(self) -> &'static str {
        match self {
            Self::FreeTierFirst => "free accounts first",
            Self::PaidFirst => "paid accounts first",
        }
    }

    /// The consequence of this order, stated plainly (the surface always
    /// states consequences — Wave-4 addendum §7).
    #[must_use]
    pub const fn consequence(self) -> &'static str {
        match self {
            Self::FreeTierFirst => {
                "Flauz uses free quota before paid accounts, and tells you before it uses a \
                 paid account."
            }
            Self::PaidFirst => {
                "Tasks use your paid accounts first, even when free quota is left."
            }
        }
    }
}

/// A per-provider override: which account this provider's routing prefers
/// first, and (optionally) a provider-specific ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderOverride {
    /// The account this provider's routing tries first (its connection
    /// reference), when it is usable.
    pub preferred_account: Option<ConnectionRef>,
    /// A provider-specific ordering, when the provider's routing should
    /// not follow the workspace order.
    pub order: Option<RoutingOrder>,
}

impl ProviderOverride {
    /// Validates the override: at least one field set, and a valid
    /// preferred-account reference.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when both fields are unset or the
    /// reference is invalid.
    pub fn validate(&self) -> Result<(), ProvError> {
        if self.preferred_account.is_none() && self.order.is_none() {
            return Err(ProvError::invalid(
                "a provider override must set a preferred account or an order",
            ));
        }
        if let Some(account) = &self.preferred_account {
            account.validate()?;
        }
        Ok(())
    }
}

/// A per-task override: which account this one task's routing prefers
/// first, when it is usable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskOverride {
    /// The account this task's routing tries first (its connection
    /// reference), when it is usable.
    pub preferred_account: Option<ConnectionRef>,
}

impl TaskOverride {
    /// Validates the override: a valid preferred-account reference.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the reference is invalid.
    pub fn validate(&self) -> Result<(), ProvError> {
        if let Some(account) = &self.preferred_account {
            account.validate()?;
        }
        Ok(())
    }
}

/// The routing policy (DATA): the ordering, the per-provider and per-task
/// overrides, and the spend and concurrency limits. A durable record the
/// user can see and change — every mutation through the setters below
/// increments the version by exactly 1.
///
/// Limits are measured in the ledger's usage units: a spend limit bounds
/// how many units may be consumed through one account (or across the
/// whole workspace) within the current quota window; a concurrency limit
/// bounds how many sessions may run at the same time through one account
/// (or across the workspace).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingPolicy {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// Durable entity version, starting at 1, +1 per change.
    pub version: u64,
    /// The account ordering (free-tier-first by default).
    pub order: RoutingOrder,
    /// Per-provider overrides, keyed by provider kind.
    pub provider_overrides: BTreeMap<String, ProviderOverride>,
    /// Per-task overrides, keyed by the task reference string.
    pub task_overrides: BTreeMap<String, TaskOverride>,
    /// The per-account spend limit in usage units per quota window, when
    /// set.
    pub account_spend_limit: Option<u64>,
    /// The workspace-wide spend limit in usage units per quota window,
    /// when set.
    pub workspace_spend_limit: Option<u64>,
    /// The per-account concurrency limit (how many sessions may run at
    /// the same time through one account), when set.
    pub account_concurrency_limit: Option<u32>,
    /// The workspace-wide concurrency limit (how many sessions may run at
    /// the same time across all accounts), when set.
    pub workspace_concurrency_limit: Option<u32>,
}

impl RoutingPolicy {
    /// The default policy: free-tier-first, no overrides, no limits.
    #[must_use]
    pub fn new() -> Self {
        Self {
            v: ProvVersion,
            version: 1,
            order: RoutingOrder::DEFAULT,
            provider_overrides: BTreeMap::new(),
            task_overrides: BTreeMap::new(),
            account_spend_limit: None,
            workspace_spend_limit: None,
            account_concurrency_limit: None,
            workspace_concurrency_limit: None,
        }
    }

    /// Validates the policy's canonical rules: bounded override maps with
    /// valid kind keys and references, and limits of at least 1.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a rule is violated.
    pub fn validate(&self) -> Result<(), ProvError> {
        if self.version == 0 {
            return Err(ProvError::invalid("routing policy version must be at least 1"));
        }
        if self.provider_overrides.len() > MAX_ACCOUNTS {
            return Err(ProvError::invalid(format!(
                "provider overrides are bounded at {MAX_ACCOUNTS} providers"
            )));
        }
        if self.task_overrides.len() > MAX_ACCOUNTS {
            return Err(ProvError::invalid(format!(
                "task overrides are bounded at {MAX_ACCOUNTS} tasks"
            )));
        }
        for (kind, provider_override) in &self.provider_overrides {
            ensure_kind_label("provider override key", kind)?;
            provider_override.validate()?;
        }
        for (task, task_override) in &self.task_overrides {
            TaskRef::parse(task)?;
            task_override.validate()?;
        }
        for (field, limit) in [
            ("account spend limit", self.account_spend_limit),
            ("workspace spend limit", self.workspace_spend_limit),
        ] {
            if limit == Some(0) {
                return Err(ProvError::invalid(format!(
                    "{field} must be at least 1 unit per window"
                )));
            }
        }
        for (field, limit) in [
            ("account concurrency limit", self.account_concurrency_limit),
            ("workspace concurrency limit", self.workspace_concurrency_limit),
        ] {
            if limit == Some(0) {
                return Err(ProvError::invalid(format!(
                    "{field} must be at least 1 concurrent session"
                )));
            }
        }
        Ok(())
    }

    /// The provider override for a kind, when one is set.
    #[must_use]
    pub fn provider_override(&self, provider_kind: &str) -> Option<&ProviderOverride> {
        self.provider_overrides.get(provider_kind)
    }

    /// The task override for a task, when one is set.
    #[must_use]
    pub fn task_override(&self, task: &TaskRef) -> Option<&TaskOverride> {
        self.task_overrides.get(task.as_str())
    }

    /// Sets (or replaces) the per-provider override for a kind — a
    /// durable change (+1 version).
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the override fails validation
    /// or the kind label is invalid.
    pub fn set_provider_override(
        &mut self,
        provider_kind: &str,
        provider_override: ProviderOverride,
    ) -> Result<(), ProvError> {
        ensure_kind_label("provider override key", provider_kind)?;
        provider_override.validate()?;
        self.provider_overrides
            .insert(provider_kind.to_owned(), provider_override);
        self.version += 1;
        Ok(())
    }

    /// Clears the per-provider override for a kind — a durable change
    /// (+1 version). Clearing an unset override is a no-op that does not
    /// bump the version (nothing changed).
    pub fn clear_provider_override(&mut self, provider_kind: &str) {
        if self.provider_overrides.remove(provider_kind).is_some() {
            self.version += 1;
        }
    }

    /// Sets (or replaces) the per-task override for a task — a durable
    /// change (+1 version).
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the override fails validation
    /// or the task reference is invalid.
    pub fn set_task_override(
        &mut self,
        task: &TaskRef,
        task_override: TaskOverride,
    ) -> Result<(), ProvError> {
        task.validate()?;
        task_override.validate()?;
        self.task_overrides
            .insert(task.as_str().to_owned(), task_override);
        self.version += 1;
        Ok(())
    }

    /// Clears the per-task override for a task — a durable change (+1
    /// version). Clearing an unset override is a no-op that does not bump
    /// the version (nothing changed).
    pub fn clear_task_override(&mut self, task: &TaskRef) {
        if self.task_overrides.remove(task.as_str()).is_some() {
            self.version += 1;
        }
    }

    /// Changes the account ordering — a durable change (+1 version); the
    /// surface states the consequence in plain words
    /// ([`RoutingOrder::consequence`]).
    pub fn set_order(&mut self, order: RoutingOrder) {
        if self.order != order {
            self.order = order;
            self.version += 1;
        }
    }

    /// Sets the spend limits (per account and workspace-wide, in usage
    /// units per quota window) — a durable change (+1 version). `None`
    /// clears a limit.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a limit is zero.
    pub fn set_spend_limits(
        &mut self,
        account_spend_limit: Option<u64>,
        workspace_spend_limit: Option<u64>,
    ) -> Result<(), ProvError> {
        for (field, limit) in [
            ("account spend limit", account_spend_limit),
            ("workspace spend limit", workspace_spend_limit),
        ] {
            if limit == Some(0) {
                return Err(ProvError::invalid(format!(
                    "{field} must be at least 1 unit per window"
                )));
            }
        }
        self.account_spend_limit = account_spend_limit;
        self.workspace_spend_limit = workspace_spend_limit;
        self.version += 1;
        Ok(())
    }

    /// Sets the concurrency limits (per account and workspace-wide) — a
    /// durable change (+1 version). `None` clears a limit.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a limit is zero.
    pub fn set_concurrency_limits(
        &mut self,
        account_concurrency_limit: Option<u32>,
        workspace_concurrency_limit: Option<u32>,
    ) -> Result<(), ProvError> {
        for (field, limit) in [
            ("account concurrency limit", account_concurrency_limit),
            ("workspace concurrency limit", workspace_concurrency_limit),
        ] {
            if limit == Some(0) {
                return Err(ProvError::invalid(format!(
                    "{field} must be at least 1 concurrent session"
                )));
            }
        }
        self.account_concurrency_limit = account_concurrency_limit;
        self.workspace_concurrency_limit = workspace_concurrency_limit;
        self.version += 1;
        Ok(())
    }
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        Self::new()
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

    fn test_connection(id: &str) -> ConnectionRef {
        ok(ConnectionRef::parse(id))
    }

    fn test_task(id: &str) -> TaskRef {
        ok(TaskRef::parse(id))
    }

    #[test]
    fn the_default_policy_is_free_tier_first_with_honest_words() {
        let policy = RoutingPolicy::new();
        ok(policy.validate());
        assert_eq!(policy.order, RoutingOrder::FreeTierFirst);
        assert_eq!(policy.version, 1);
        assert_eq!(RoutingOrder::DEFAULT, RoutingOrder::FreeTierFirst);
        assert_eq!(policy.order.user_label(), "free accounts first");
        assert_eq!(
            RoutingOrder::PaidFirst.user_label(),
            "paid accounts first"
        );
        // The consequences are stated plainly (addendum §7).
        assert!(RoutingOrder::FreeTierFirst
            .consequence()
            .contains("tells you before it uses a paid account"));
        assert!(RoutingOrder::PaidFirst
            .consequence()
            .contains("even when free quota is left"));
        // The canonical default serializes deterministically.
        let serialized = ok(serde_json::to_string(&policy));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,\"version\":1,\"order\":\"free_tier_first\",",
                "\"provider_overrides\":{},\"task_overrides\":{},",
                "\"account_spend_limit\":null,\"workspace_spend_limit\":null,",
                "\"account_concurrency_limit\":null,\"workspace_concurrency_limit\":null}"
            )
        );
        let reloaded: RoutingPolicy = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, policy);
    }

    #[test]
    fn policy_changes_are_durable_and_validated() {
        let mut policy = RoutingPolicy::new();
        let task = test_task("task_01J8ZQ5V8K3T2B7N6X4R9DQPD3");
        let connection = test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0");

        policy.set_order(RoutingOrder::PaidFirst);
        assert_eq!(policy.version, 2);
        // Setting the same order again changes nothing (no phantom
        // version bumps).
        policy.set_order(RoutingOrder::PaidFirst);
        assert_eq!(policy.version, 2);

        ok(policy.set_task_override(
            &task,
            TaskOverride {
                preferred_account: Some(connection.clone()),
            },
        ));
        assert_eq!(policy.version, 3);
        ok(policy.set_provider_override(
            "openai",
            ProviderOverride {
                preferred_account: Some(connection.clone()),
                order: Some(RoutingOrder::FreeTierFirst),
            },
        ));
        assert_eq!(policy.version, 4);
        assert!(policy.provider_override("openai").is_some());
        assert!(policy.task_override(&task).is_some());

        ok(policy.set_spend_limits(Some(100), Some(500)));
        assert_eq!(policy.version, 5);
        ok(policy.set_concurrency_limits(Some(2), Some(8)));
        assert_eq!(policy.version, 6);
        ok(policy.validate());

        // Clearing is durable too; clearing unset entries is a no-op.
        policy.clear_task_override(&task);
        assert_eq!(policy.version, 7);
        policy.clear_task_override(&task);
        assert_eq!(policy.version, 7);
        policy.clear_provider_override("openai");
        assert_eq!(policy.version, 8);
        policy.clear_provider_override("openai");
        assert_eq!(policy.version, 8);

        // Zero limits, empty overrides, and invalid keys are refused.
        assert!(policy.set_spend_limits(Some(0), None).is_err());
        assert!(policy.set_concurrency_limits(None, Some(0)).is_err());
        assert!(policy
            .set_provider_override(
                "OpenAI",
                ProviderOverride {
                    preferred_account: None,
                    order: Some(RoutingOrder::FreeTierFirst),
                },
            )
            .is_err());
        assert!(policy
            .set_provider_override(
                "openai",
                ProviderOverride {
                    preferred_account: None,
                    order: None,
                },
            )
            .is_err());

        // The canonical round-trip survives a full policy.
        let serialized = ok(serde_json::to_string(&policy));
        let reloaded: RoutingPolicy = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, policy);
        assert!(
            serde_json::from_str::<RoutingPolicy>(&serialized.replace("\"v\":1,", "")).is_err(),
            "unknown fields must be rejected"
        );
        let mut invalid = policy.clone();
        invalid.account_spend_limit = Some(0);
        assert!(invalid.validate().is_err());
        let mut foreign_task_key = policy.clone();
        foreign_task_key
            .task_overrides
            .insert("not-a-task-id".to_owned(), TaskOverride {
                preferred_account: None,
            });
        assert!(foreign_task_key.validate().is_err());
    }
}
