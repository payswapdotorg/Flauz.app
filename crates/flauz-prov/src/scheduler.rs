//! The scheduler (work order PROV-001, part 2): free-tier-first routing
//! with the **attribution law** — no [`SchedulingChoice`] without a named
//! account, tier and policy rule, and escalation to a paid tier is NAMED,
//! never silent (Wave-4 addendum §2 — the CAP-001 law applied to money).
//!
//! # Inputs as data (the flauz-cap pattern)
//!
//! [`schedule_routing`] is a pure function of [`SchedulingInputs`]: the
//! routing need (provider kind + capability needs + the task being
//! routed), the accounts (with their quota snapshots), the routing
//! policy, the usage ledger and the session load. Capability needs are
//! recorded for attribution — capability **resolution** is the
//! flauz-cap intersection's job, and account matching here is by
//! provider kind, never a silent capability fall-through.
//!
//! # The ordering
//!
//! The walk is deterministic: the task override's account first, then the
//! provider override's account, then the policy's tier ordering
//! (free-tier-first by default) over the remaining candidates, ties
//! broken by canonical connection reference. Every account the walk
//! passes over is a NAMED [`SkippedAccount`] with its reason — depleted,
//! rate-limited, over a spend limit, over a concurrency limit, or a
//! window that does not cover the scheduling moment. When the preferred
//! tier's accounts are all skipped and a paid account is chosen instead,
//! the choice is an ESCALATION: the rule names it, and the skipped list
//! names the free account it left behind.
//!
//! Workspace-wide limits that stop all routing (the workspace spend and
//! concurrency limits) are honest errors naming the limit — never a
//! silent choice of "some account".

use serde::{Deserialize, Serialize};

use crate::account::{ProviderAccount, QuotaWindow, TierKind};
use crate::key::{CapabilityKey, ConnectionRef, TaskRef};
use crate::ledger::QuotaLedger;
use crate::policy::{RoutingOrder, RoutingPolicy};
use crate::time::Timestamp;
use crate::{
    MAX_ACCOUNTS, MAX_CAPABILITY_KEYS, MAX_NAME_BYTES, ProvError, ProvVersion, ensure_explanation,
    ensure_kind_label, ensure_non_empty, ensure_str_bound,
};

/// The routing need — the requirement record as data: which provider the
/// routed work draws on, which capabilities it requires (recorded for
/// attribution; resolution is the capability intersection's job), and
/// which task the routing is for (attribution's "which task", as the
/// frozen `task_<ULID>` format).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingNeed {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// The neutral kind label of the provider the need draws on (for
    /// example `openai`).
    pub provider_kind: String,
    /// The capability keys the routed work requires — canonically sorted
    /// and deduplicated, recorded on every choice for attribution.
    pub capabilities: Vec<CapabilityKey>,
    /// The task the routing is attributed to, when the need is for one
    /// task (workspace-level needs carry `null`).
    pub task_id: Option<TaskRef>,
}

impl RoutingNeed {
    /// Builds a routing need, validating the canonical rules: a kind
    /// label, and a bounded, sorted, deduplicated capability list.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a rule is violated.
    pub fn new(
        provider_kind: &str,
        capabilities: &[&str],
        task_id: Option<TaskRef>,
    ) -> Result<Self, ProvError> {
        ensure_kind_label("routing need provider kind", provider_kind)?;
        let mut keys = Vec::with_capacity(capabilities.len());
        for capability in capabilities {
            keys.push(CapabilityKey::parse(capability)?);
        }
        if keys.len() > MAX_CAPABILITY_KEYS {
            return Err(ProvError::invalid(format!(
                "routing need capabilities are bounded at {MAX_CAPABILITY_KEYS} keys"
            )));
        }
        keys.sort();
        keys.dedup();
        if let Some(task) = &task_id {
            task.validate()?;
        }
        Ok(Self {
            v: ProvVersion,
            provider_kind: provider_kind.to_owned(),
            capabilities: keys,
            task_id,
        })
    }

    /// Validates the routing need (the canonical rules of [`Self::new`]).
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a rule is violated.
    pub fn validate(&self) -> Result<(), ProvError> {
        ensure_kind_label("routing need provider kind", &self.provider_kind)?;
        if self.capabilities.len() > MAX_CAPABILITY_KEYS {
            return Err(ProvError::invalid(format!(
                "routing need capabilities are bounded at {MAX_CAPABILITY_KEYS} keys"
            )));
        }
        for key in &self.capabilities {
            key.validate()?;
        }
        if self
            .capabilities
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(ProvError::invalid(
                "routing need capabilities must be sorted and free of duplicates",
            ));
        }
        if let Some(task) = &self.task_id {
            task.validate()?;
        }
        Ok(())
    }
}

/// Which policy rule chose the account — the named half of the
/// attribution law. A choice without one cannot be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyRule {
    /// A per-task override chose the account (the user's explicit choice
    /// for this one task).
    TaskOverride,
    /// A per-provider override chose the account (the user's explicit
    /// preference for this provider).
    ProviderOverride,
    /// The default account ordering chose it (free accounts first by
    /// default).
    DefaultOrder,
    /// The preferred free tier was used up or unavailable, so a paid
    /// account is used — the escalation is NAMED, never silent, and the
    /// skipped list names the free account it left behind.
    EscalatedToPaid,
    /// Every other candidate was skipped; this is the only account with
    /// quota left.
    OnlyAvailable,
}

impl PolicyRule {
    /// The user-facing label for this rule (plain words — part of
    /// attribution copy).
    #[must_use]
    pub const fn user_label(self) -> &'static str {
        match self {
            Self::TaskOverride => "you chose this account for this task",
            Self::ProviderOverride => "you set this provider to prefer this account",
            Self::DefaultOrder => "the account order picked it",
            Self::EscalatedToPaid => "the free tier was used up, so this uses your paid account",
            Self::OnlyAvailable => "it is the only account with quota left",
        }
    }
}

/// The named reason a candidate account was passed over (the CAP-001
/// no-silent-fall-through law, applied to money).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// The account's quota for the current window is used up.
    Depleted,
    /// The provider reported the account rate-limited (the bound has not
    /// passed yet).
    RateLimited,
    /// The account reached the spending limit set for it.
    SpendLimit,
    /// The account is at its limit for tasks running at the same time.
    ConcurrencyLimit,
    /// The account's quota snapshot does not cover the scheduling moment
    /// — its current numbers are not in yet, and it must not be silently
    /// treated as current.
    WindowNotActive,
}

impl SkipReason {
    /// The user-facing label for this reason (plain words).
    #[must_use]
    pub const fn user_label(self) -> &'static str {
        match self {
            Self::Depleted => "its quota is used up for now",
            Self::RateLimited => "it is paused by the provider right now",
            Self::SpendLimit => "it reached the spending limit you set",
            Self::ConcurrencyLimit => "it is at its limit for tasks running at the same time",
            Self::WindowNotActive => "its current quota numbers are not in yet",
        }
    }
}

/// One skipped alternative: the account the router passed over, with the
/// named reason (part of every choice — the honest record of what was
/// NOT used and why).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkippedAccount {
    /// The account's connection reference.
    pub connection_id: ConnectionRef,
    /// The account's user-words label.
    pub account_label: String,
    /// The account's provider kind.
    pub provider_kind: String,
    /// The account's tier.
    pub tier: TierKind,
    /// The named reason the account was passed over.
    pub reason: SkipReason,
}

impl SkippedAccount {
    /// Validates the skipped-account record.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a reference, label or kind is
    /// invalid.
    pub fn validate(&self) -> Result<(), ProvError> {
        self.connection_id.validate()?;
        ensure_non_empty("skipped account label", &self.account_label)?;
        ensure_str_bound("skipped account label", &self.account_label, MAX_NAME_BYTES)?;
        ensure_kind_label("skipped account provider kind", &self.provider_kind)?;
        Ok(())
    }
}

/// The session load as data: how many sessions run through each account
/// right now, and how many run workspace-wide. Caller-supplied (the
/// harness's own view) — the scheduler reads it, never invents it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionLoad {
    /// The sessions running through each account, keyed by connection
    /// reference.
    pub per_account: std::collections::BTreeMap<ConnectionRef, u32>,
    /// The sessions running workspace-wide, across every account.
    pub workspace: u32,
}

impl SessionLoad {
    /// The sessions running through one account (zero when unlisted).
    #[must_use]
    pub fn sessions_for(&self, connection_id: &ConnectionRef) -> u32 {
        self.per_account.get(connection_id).copied().unwrap_or(0)
    }

    /// Validates the load's canonical rules: bounded, valid keys, and a
    /// workspace count that is never less than the per-account sum.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a rule is violated.
    pub fn validate(&self) -> Result<(), ProvError> {
        if self.per_account.len() > MAX_ACCOUNTS {
            return Err(ProvError::invalid(format!(
                "the session load is bounded at {MAX_ACCOUNTS} accounts"
            )));
        }
        let per_account_sum: u64 = self.per_account.values().map(|count| u64::from(*count)).sum();
        if u64::from(self.workspace) < per_account_sum {
            return Err(ProvError::invalid(
                "the workspace session count must cover every per-account session",
            ));
        }
        for connection_id in self.per_account.keys() {
            connection_id.validate()?;
        }
        Ok(())
    }
}

/// The scheduler's inputs, as data: the need, the accounts (canonically
/// ordered by connection reference), the policy, the ledger, the session
/// load, and the scheduling moment (caller-supplied — the crate never
/// reads the wall clock).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulingInputs {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// The routing need being scheduled.
    pub need: RoutingNeed,
    /// The accounts to route across, sorted by connection reference with
    /// no duplicates (canonical state — the walk is deterministic).
    pub accounts: Vec<ProviderAccount>,
    /// The routing policy in force.
    pub policy: RoutingPolicy,
    /// The usage ledger (spend limits are measured against it).
    pub ledger: QuotaLedger,
    /// The session load (concurrency limits are measured against it).
    pub load: SessionLoad,
    /// The scheduling moment.
    pub now: Timestamp,
}

impl SchedulingInputs {
    /// Validates the inputs' canonical rules: every component valid, and
    /// the account list canonically ordered with no duplicates.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a rule is violated.
    pub fn validate(&self) -> Result<(), ProvError> {
        self.need.validate()?;
        if self.accounts.len() > MAX_ACCOUNTS {
            return Err(ProvError::invalid(format!(
                "scheduler inputs are bounded at {MAX_ACCOUNTS} accounts"
            )));
        }
        for account in &self.accounts {
            account.validate()?;
        }
        if self
            .accounts
            .windows(2)
            .any(|pair| pair[0].connection_id >= pair[1].connection_id)
        {
            return Err(ProvError::invalid(
                "scheduler input accounts must be sorted by connection reference and free of \
                 duplicates (canonical state — the walk is deterministic)",
            ));
        }
        self.policy.validate()?;
        self.ledger.validate()?;
        self.load.validate()?;
        Ok(())
    }
}

/// The scheduling choice — the attribution record (Wave-4 addendum §2):
/// the chosen account (its connection reference and user-words label),
/// the provider, the tier, the policy rule that chose it, the plain-words
/// explanation, the quota snapshot at choice time, and the named
/// alternatives that were passed over. Canonical JSON; deterministic.
///
/// **The attribution law:** the account, tier and policy rule fields are
/// non-optional, and [`SchedulingChoice::validate`] rejects empty labels,
/// invalid references, provider mismatches, and escalated choices with no
/// named free alternative — ambient attribution cannot be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulingChoice {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// The need the choice answers (provider kind, capability needs, and
    /// the task the routing is attributed to).
    pub need: RoutingNeed,
    /// The chosen account's connection reference.
    pub connection_id: ConnectionRef,
    /// The chosen account's user-words label — the "whose account" half
    /// of the attribution.
    pub account_label: String,
    /// The chosen account's provider kind (matching the need).
    pub provider_kind: String,
    /// The chosen account's tier (free/paid).
    pub tier: TierKind,
    /// The policy rule that chose the account.
    pub policy_rule: PolicyRule,
    /// The plain-words explanation of the choice (deterministic,
    /// user-words).
    pub rule_explanation: String,
    /// Whether the choice escalates from the preferred free tier to a
    /// paid account — named, never silent.
    pub escalated: bool,
    /// The named alternatives the walk passed over, in walk order (the
    /// override shortlist first, then the tier ordering — deterministic
    /// for identical inputs).
    pub skipped: Vec<SkippedAccount>,
    /// The chosen account's quota snapshot at choice time — the honest
    /// "what was left when this was chosen".
    pub quota_at_choice: QuotaWindow,
}

impl SchedulingChoice {
    /// Validates the choice against the attribution law: named account
    /// (valid reference, non-empty bounded label), provider matching the
    /// need, a bounded non-empty explanation, skipped alternatives in
    /// canonical order and never containing the chosen account, and the
    /// escalation law — `escalated` exactly when the rule names the
    /// escalation, escalating to a paid tier, with at least one named
    /// free alternative left behind.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the law is violated — ambient
    /// attribution cannot be constructed.
    pub fn validate(&self) -> Result<(), ProvError> {
        self.need.validate()?;
        self.connection_id.validate()?;
        ensure_non_empty("choice account label", &self.account_label)?;
        ensure_str_bound("choice account label", &self.account_label, MAX_NAME_BYTES)?;
        ensure_kind_label("choice provider kind", &self.provider_kind)?;
        if self.provider_kind != self.need.provider_kind {
            return Err(ProvError::invalid(
                "the choice's provider must match its routing need",
            ));
        }
        ensure_explanation("choice rule explanation", &self.rule_explanation)?;
        self.quota_at_choice.validate()?;
        for skipped in &self.skipped {
            skipped.validate()?;
            if skipped.provider_kind != self.need.provider_kind {
                return Err(ProvError::invalid(
                    "a skipped alternative must belong to the need's provider",
                ));
            }
            if skipped.connection_id == self.connection_id {
                return Err(ProvError::invalid(
                    "the chosen account cannot also be listed as skipped",
                ));
            }
        }
        // The escalation law: `escalated` ⇔ the rule names it, it
        // escalates TO a paid tier, and it names the free alternative it
        // left behind. A silent escalation cannot validate.
        if self.escalated != (self.policy_rule == PolicyRule::EscalatedToPaid) {
            return Err(ProvError::invalid(
                "escalated must be true exactly when the policy rule is the named escalation",
            ));
        }
        if self.escalated {
            if self.tier != TierKind::Paid {
                return Err(ProvError::invalid(
                    "an escalation chooses a paid account — a free choice is not an escalation",
                ));
            }
            if self.skipped.is_empty() {
                return Err(ProvError::invalid(
                    "an escalation must name the free alternative it left behind (no silent \
                     escalation)",
                ));
            }
            if !self.skipped.iter().any(|skipped| skipped.tier == TierKind::Free) {
                return Err(ProvError::invalid(
                    "an escalation must name the free account it passed over",
                ));
            }
        }
        Ok(())
    }
}

/// Schedules one routing need over the accounts, policy, ledger and load:
/// a pure, deterministic function returning the attributed choice — or an
/// honest, named error when no account can be chosen.
///
/// # Errors
///
/// Returns [`ProvError::Invalid`] when the inputs fail canonical
/// validation, when no account is connected for the needed provider, or
/// when a workspace-wide limit (spend or concurrency) stops all routing —
/// each error names its cause; nothing is chosen silently.
pub fn schedule_routing(inputs: &SchedulingInputs) -> Result<SchedulingChoice, ProvError> {
    inputs.validate()?;
    let need = &inputs.need;
    let now = inputs.now;
    let policy = &inputs.policy;

    // The candidates: the accounts of the needed provider kind, already
    // in canonical connection order (the inputs are validated canonical).
    let candidates: Vec<&ProviderAccount> = inputs
        .accounts
        .iter()
        .filter(|account| account.provider_kind == need.provider_kind)
        .collect();
    if candidates.is_empty() {
        return Err(ProvError::invalid(format!(
            "no account is connected for provider {:?}: connect an account for this provider \
             before routing",
            need.provider_kind
        )));
    }

    // Workspace-wide gates, named honestly: a reached workspace limit
    // stops ALL routing (never a silent choice of some account).
    let workspace_window_start = candidates
        .iter()
        .map(|account| account.quota.window_start)
        .min()
        .unwrap_or_else(|| candidates[0].quota.window_start);
    if let Some(limit) = policy.workspace_spend_limit {
        let spent = inputs.ledger.total_units_since(workspace_window_start);
        if spent >= limit {
            return Err(ProvError::invalid(format!(
                "the workspace spending limit of {limit} uses is already reached ({spent} used \
                 in this window): no account is used until the limit changes or the window \
                 resets"
            )));
        }
    }
    if let Some(limit) = policy.workspace_concurrency_limit {
        if inputs.load.workspace >= limit {
            return Err(ProvError::invalid(format!(
                "the workspace limit of {limit} tasks running at the same time is already \
                 reached: no account is used until a task finishes or the limit changes"
            )));
        }
    }

    // The walk order: the task override's account first, then the
    // provider override's account, then the policy's tier ordering over
    // the remaining candidates (canonical connection order breaks ties).
    let task_preferred = need
        .task_id
        .as_ref()
        .and_then(|task| policy.task_override(task))
        .and_then(|task_override| task_override.preferred_account.clone());
    let provider_preferred = policy
        .provider_override(&need.provider_kind)
        .and_then(|provider_override| provider_override.preferred_account.clone());
    let mut ordered: Vec<&ProviderAccount> = Vec::with_capacity(candidates.len());
    if let Some(preferred) = &task_preferred {
        if let Some(account) = candidates
            .iter()
            .copied()
            .find(|account| &account.connection_id == preferred)
        {
            ordered.push(account);
        }
    }
    if let Some(preferred) = &provider_preferred {
        if let Some(account) = candidates
            .iter()
            .copied()
            .find(|account| &account.connection_id == preferred && !ordered.contains(&account))
        {
            ordered.push(account);
        }
    }
    let mut rest: Vec<&ProviderAccount> = candidates
        .iter()
        .copied()
        .filter(|account| !ordered.contains(account))
        .collect();
    rest.sort_by_key(|account| {
        (tier_rank(account.tier, policy.order), account.connection_id.clone())
    });
    ordered.extend(rest);

    // The walk: check every account in order; pass over the unusable
    // ones with NAMED reasons; choose the first usable account.
    let mut skipped: Vec<SkippedAccount> = Vec::new();
    let mut chosen: Option<&ProviderAccount> = None;
    for account in ordered {
        match usability_of(account, policy, &inputs.ledger, &inputs.load, now) {
            Ok(()) => {
                chosen = Some(account);
                break;
            }
            Err(reason) => skipped.push(SkippedAccount {
                connection_id: account.connection_id.clone(),
                account_label: account.account_label.clone(),
                provider_kind: account.provider_kind.clone(),
                tier: account.tier,
                reason,
            }),
        }
    }
    let Some(chosen) = chosen else {
        // Nothing usable: every candidate is named with its reason — the
        // honest end state, never a silent fallback.
        let reasons = skipped
            .iter()
            .map(|skipped| {
                format!(
                    "{} ({})",
                    skipped.account_label,
                    skipped.reason.user_label()
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Err(ProvError::invalid(format!(
            "no account for provider {:?} can be used right now: {reasons}",
            need.provider_kind
        )));
    };

    // The rule that chose it — the named half of the attribution law.
    let override_rule = if task_preferred.as_ref() == Some(&chosen.connection_id) {
        Some(PolicyRule::TaskOverride)
    } else if provider_preferred.as_ref() == Some(&chosen.connection_id) {
        Some(PolicyRule::ProviderOverride)
    } else {
        None
    };
    let preferred_tier = preferred_tier_of(policy.order);
    let preferred_tier_existed = candidates
        .iter()
        .any(|account| account.tier == preferred_tier);
    let escalated = override_rule.is_none()
        && policy.order == RoutingOrder::FreeTierFirst
        && chosen.tier == TierKind::Paid
        && preferred_tier_existed;
    let policy_rule = match override_rule {
        Some(rule) => rule,
        None if escalated => PolicyRule::EscalatedToPaid,
        None if skipped.len() == candidates.len() - 1 => PolicyRule::OnlyAvailable,
        None => PolicyRule::DefaultOrder,
    };
    let rule_explanation =
        rule_explanation_for(policy_rule, policy.order, preferred_tier_existed);

    let choice = SchedulingChoice {
        v: ProvVersion,
        need: need.clone(),
        connection_id: chosen.connection_id.clone(),
        account_label: chosen.account_label.clone(),
        provider_kind: chosen.provider_kind.clone(),
        tier: chosen.tier,
        policy_rule,
        rule_explanation,
        escalated,
        skipped,
        quota_at_choice: chosen.quota,
    };
    choice.validate()?;
    Ok(choice)
}

/// The tier's rank under the policy's ordering (the preferred tier ranks
/// first).
fn tier_rank(tier: TierKind, order: RoutingOrder) -> u8 {
    match order {
        RoutingOrder::FreeTierFirst => tier.default_order_rank(),
        RoutingOrder::PaidFirst => 1 - tier.default_order_rank(),
    }
}

/// The tier the policy's ordering prefers.
const fn preferred_tier_of(order: RoutingOrder) -> TierKind {
    match order {
        RoutingOrder::FreeTierFirst => TierKind::Free,
        RoutingOrder::PaidFirst => TierKind::Paid,
    }
}

/// The deterministic, user-words explanation for a policy rule.
fn rule_explanation_for(
    rule: PolicyRule,
    order: RoutingOrder,
    preferred_tier_existed: bool,
) -> String {
    match rule {
        PolicyRule::TaskOverride => "You chose this account for this task.".to_owned(),
        PolicyRule::ProviderOverride => {
            "You set this provider to prefer this account.".to_owned()
        }
        PolicyRule::DefaultOrder => match (order, preferred_tier_existed) {
            (RoutingOrder::FreeTierFirst, true) => {
                "Free accounts are tried first, and this one had quota left.".to_owned()
            }
            (RoutingOrder::FreeTierFirst, false) => {
                "No free account is connected for this provider, so this uses your paid \
                 account."
                    .to_owned()
            }
            (RoutingOrder::PaidFirst, true) => {
                "Paid accounts are tried first, and this one had quota left.".to_owned()
            }
            (RoutingOrder::PaidFirst, false) => {
                "No paid account is connected for this provider, so this uses your free \
                 account."
                    .to_owned()
            }
        },
        PolicyRule::EscalatedToPaid => {
            "Your free tier was used up or unavailable, so this uses your paid account.".to_owned()
        }
        PolicyRule::OnlyAvailable => {
            "It is the only account for this provider with quota left.".to_owned()
        }
    }
}

/// The usability check for one account at the scheduling moment: usable,
/// or the NAMED reason it is not (checked in a fixed order — rate limit,
/// window currency, depletion, spend limit, concurrency limit).
fn usability_of(
    account: &ProviderAccount,
    policy: &RoutingPolicy,
    ledger: &QuotaLedger,
    load: &SessionLoad,
    now: Timestamp,
) -> Result<(), SkipReason> {
    if account.is_rate_limited_at(now) {
        return Err(SkipReason::RateLimited);
    }
    if !account.quota.covers(now) {
        return Err(SkipReason::WindowNotActive);
    }
    if account.quota.is_depleted() {
        return Err(SkipReason::Depleted);
    }
    if let Some(limit) = policy.account_spend_limit {
        if ledger.units_since(&account.connection_id, account.quota.window_start) >= limit {
            return Err(SkipReason::SpendLimit);
        }
    }
    if let Some(limit) = policy.account_concurrency_limit {
        if load.sessions_for(&account.connection_id) >= limit {
            return Err(SkipReason::ConcurrencyLimit);
        }
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

    fn err<T: std::fmt::Debug, E: std::fmt::Display>(result: Result<T, E>) -> E {
        match result {
            Ok(value) => panic!("expected an honest refusal, got {value:?}"),
            Err(error) => error,
        }
    }

    fn test_connection(id: &str) -> ConnectionRef {
        ok(ConnectionRef::parse(id))
    }

    fn test_task(id: &str) -> TaskRef {
        ok(TaskRef::parse(id))
    }

    fn test_timestamp(value: &str) -> Timestamp {
        ok(Timestamp::parse(value))
    }

    fn test_need(task: Option<&str>) -> RoutingNeed {
        ok(RoutingNeed::new(
            "openai",
            &["terminal", "web.search"],
            task.map(|task| test_task(task)),
        ))
    }

    fn test_account(
        connection: &str,
        label: &str,
        tier: TierKind,
        remaining: u64,
        limit: u64,
    ) -> ProviderAccount {
        ok(ProviderAccount::new(
            test_connection(connection),
            ok(crate::key::SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34")),
            label,
            "openai",
            tier,
            QuotaWindow {
                remaining,
                limit,
                window_start: test_timestamp("2026-09-23T00:00:00Z"),
                window_end: test_timestamp("2026-09-24T00:00:00Z"),
            },
            None,
        ))
    }

    fn test_inputs(accounts: Vec<ProviderAccount>) -> SchedulingInputs {
        SchedulingInputs {
            v: ProvVersion,
            need: test_need(Some("task_01J8ZQ5V8K3T2B7N6X4R9DQPD3")),
            accounts,
            policy: RoutingPolicy::new(),
            ledger: QuotaLedger::new(),
            load: SessionLoad::default(),
            now: test_timestamp("2026-09-23T12:00:00Z"),
        }
    }

    const FREE: &str = "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0";
    const PAID: &str = "conn_01J8ZQ5V8K3T2B7N6X4R9DQPB1";

    fn free_and_paid() -> Vec<ProviderAccount> {
        vec![
            test_account(FREE, "Personal account", TierKind::Free, 3, 5),
            test_account(PAID, "Work account", TierKind::Paid, 90, 100),
        ]
    }

    #[test]
    fn routing_needs_validate_canonically() {
        ok(test_need(None).validate());
        // The capability list is canonicalized: sorted and deduplicated.
        let need = ok(RoutingNeed::new("openai", &["web.search", "terminal", "web.search"], None));
        assert_eq!(need.capabilities.len(), 2);
        assert_eq!(need.capabilities[0].as_str(), "terminal");
        assert_eq!(need.capabilities[1].as_str(), "web.search");
        assert!(RoutingNeed::new("OpenAI", &[], None).is_err());
        assert!(RoutingNeed::new("openai", &["Terminal"], None).is_err());
        // A round-trip survives the canonical form.
        let serialized = ok(serde_json::to_string(&need));
        let reloaded: RoutingNeed = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, need);
    }

    #[test]
    fn no_scheduling_choice_without_full_attribution() {
        // The law: every choice names account + tier + policy rule. The
        // happy shape validates.
        let choice = ok(schedule_routing(&test_inputs(free_and_paid())));
        ok(choice.validate());
        assert_eq!(choice.connection_id.as_str(), FREE);
        assert_eq!(choice.account_label, "Personal account");
        assert_eq!(choice.tier, TierKind::Free);
        assert_eq!(choice.policy_rule, PolicyRule::DefaultOrder);
        assert!(!choice.escalated);
        assert_eq!(choice.quota_at_choice.remaining, 3);
        assert_eq!(
            choice.rule_explanation,
            "Free accounts are tried first, and this one had quota left."
        );

        // Ambient attribution cannot be constructed: an unlabeled account
        // fails validation.
        let mut ambient = choice.clone();
        ambient.account_label = String::new();
        assert!(
            ambient.validate().is_err(),
            "a choice without a named account (empty label) is ambient — refused"
        );
        // A provider mismatch is refused.
        let mut mismatched = choice.clone();
        mismatched.provider_kind = "e2b".to_owned();
        assert!(mismatched.validate().is_err());
        // An empty explanation is refused.
        let mut silent = choice.clone();
        silent.rule_explanation = String::new();
        assert!(silent.validate().is_err());
        // The chosen account cannot appear in its own skipped list.
        let mut self_skipped = choice.clone();
        self_skipped.skipped.push(SkippedAccount {
            connection_id: test_connection(FREE),
            account_label: "Personal account".to_owned(),
            provider_kind: "openai".to_owned(),
            tier: TierKind::Free,
            reason: SkipReason::Depleted,
        });
        assert!(self_skipped.validate().is_err());
        // The escalation law: escalated without the rule naming it is
        // refused, and so is the rule without the flag.
        let mut unnamed_escalation = choice.clone();
        unnamed_escalation.escalated = true;
        assert!(unnamed_escalation.validate().is_err());
        let mut flagged_not_paid = choice.clone();
        flagged_not_paid.policy_rule = PolicyRule::EscalatedToPaid;
        assert!(flagged_not_paid.validate().is_err());
        // The silent escalation: the rule and the flag, but no named free
        // alternative.
        let mut silent_escalation = choice.clone();
        silent_escalation.policy_rule = PolicyRule::EscalatedToPaid;
        silent_escalation.escalated = true;
        silent_escalation.tier = TierKind::Paid;
        assert!(
            silent_escalation.validate().is_err(),
            "an escalation with no named free alternative is the silent fall-through the \
             kernel forbids"
        );

        // A choice missing its attribution fields cannot even be parsed:
        // the canonical read rejects unknown/missing shapes.
        let serialized = ok(serde_json::to_string(&choice));
        let reloaded: SchedulingChoice = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, choice);
        assert!(
            serde_json::from_str::<SchedulingChoice>(&serialized.replace("\"v\":1,", "")).is_err(),
            "unknown fields must be rejected"
        );
    }

    #[test]
    fn free_tier_first_with_the_named_escalation() {
        // Free tier first: with quota left, the free account is chosen.
        let first = ok(schedule_routing(&test_inputs(free_and_paid())));
        assert_eq!(first.connection_id.as_str(), FREE);
        assert_eq!(first.policy_rule, PolicyRule::DefaultOrder);
        assert!(first.skipped.is_empty());

        // Deplete the free account: the escalation is chosen and NAMED —
        // the free account appears in the skipped list with its reason,
        // the rule is the escalation, and the quota at choice is the paid
        // account's.
        let mut depleted = test_inputs(free_and_paid());
        depleted.accounts[0].quota = QuotaWindow {
            remaining: 0,
            limit: 5,
            window_start: test_timestamp("2026-09-23T00:00:00Z"),
            window_end: test_timestamp("2026-09-24T00:00:00Z"),
        };
        let escalated = ok(schedule_routing(&depleted));
        assert_eq!(escalated.connection_id.as_str(), PAID);
        assert_eq!(escalated.tier, TierKind::Paid);
        assert_eq!(escalated.policy_rule, PolicyRule::EscalatedToPaid);
        assert!(escalated.escalated);
        assert_eq!(escalated.skipped.len(), 1);
        assert_eq!(escalated.skipped[0].connection_id.as_str(), FREE);
        assert_eq!(escalated.skipped[0].tier, TierKind::Free);
        assert_eq!(escalated.skipped[0].reason, SkipReason::Depleted);
        assert_eq!(escalated.quota_at_choice.remaining, 90);
        assert_eq!(
            escalated.rule_explanation,
            "Your free tier was used up or unavailable, so this uses your paid account."
        );
        ok(escalated.validate());

        // When no free account exists at all, choosing the paid account
        // is NOT an escalation (nothing was depleted).
        let paid_only = test_inputs(vec![test_account(
            PAID,
            "Work account",
            TierKind::Paid,
            90,
            100,
        )]);
        let chosen = ok(schedule_routing(&paid_only));
        assert_eq!(chosen.policy_rule, PolicyRule::OnlyAvailable);
        assert!(!chosen.escalated);

        // Determinism: identical inputs produce identical choices,
        // byte-for-byte.
        let again = ok(schedule_routing(&depleted));
        assert_eq!(
            ok(serde_json::to_string(&again)),
            ok(serde_json::to_string(&escalated))
        );
    }

    #[test]
    fn multi_account_ordering_is_deterministic() {
        // Several accounts of the same tier: the canonical connection
        // order breaks ties. The first account is depleted, so it is a
        // named skip and the second (canonical) free account is chosen.
        let accounts = vec![
            test_account(
                "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
                "First free account",
                TierKind::Free,
                0,
                5,
            ),
            test_account(
                "conn_01J8ZQ5V8K3T2B7N6X4R9DQPB1",
                "Second free account",
                TierKind::Free,
                2,
                5,
            ),
            test_account(
                "conn_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
                "Third free account",
                TierKind::Free,
                4,
                5,
            ),
        ];
        let inputs = test_inputs(accounts);
        let choice = ok(schedule_routing(&inputs));
        assert_eq!(choice.connection_id.as_str(), "conn_01J8ZQ5V8K3T2B7N6X4R9DQPB1");
        // The depleted first account is a named skip in walk order.
        assert_eq!(choice.skipped.len(), 1);
        assert_eq!(
            choice.skipped[0].connection_id.as_str(),
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0"
        );
        assert_eq!(choice.skipped[0].reason, SkipReason::Depleted);

        // Non-canonical input (unsorted accounts) is rejected — the walk
        // is deterministic by construction, never by re-sorting.
        let mut unsorted = test_inputs(vec![
            test_account(
                "conn_01J8ZQ5V8K3T2B7N6X4R9DQPB1",
                "Second free account",
                TierKind::Free,
                2,
                5,
            ),
            test_account(
                "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
                "First free account",
                TierKind::Free,
                3,
                5,
            ),
        ]);
        assert!(schedule_routing(&unsorted).is_err());
        // Duplicated accounts are rejected too.
        unsorted.accounts[1] = unsorted.accounts[0].clone();
        assert!(schedule_routing(&unsorted).is_err());

        // The same inputs twice: identical choices (determinism pinned).
        let first = ok(schedule_routing(&inputs));
        let second = ok(schedule_routing(&inputs));
        assert_eq!(first, second);
    }

    #[test]
    fn rate_limits_and_stale_windows_are_named_skips() {
        // A rate-limited free account is passed over with the named
        // reason, and the paid account is chosen as a NAMED escalation.
        let mut limited = test_inputs(free_and_paid());
        limited.accounts[0].rate_limited_until = Some(test_timestamp("2026-09-23T13:00:00Z"));
        let choice = ok(schedule_routing(&limited));
        assert_eq!(choice.connection_id.as_str(), PAID);
        assert_eq!(choice.policy_rule, PolicyRule::EscalatedToPaid);
        assert_eq!(choice.skipped[0].reason, SkipReason::RateLimited);

        // A stale window (not covering the scheduling moment) is never
        // silently treated as current.
        let mut stale = test_inputs(free_and_paid());
        stale.accounts[0].quota = QuotaWindow {
            remaining: 5,
            limit: 5,
            window_start: test_timestamp("2026-09-22T00:00:00Z"),
            window_end: test_timestamp("2026-09-23T00:00:00Z"),
        };
        let choice = ok(schedule_routing(&stale));
        assert_eq!(choice.connection_id.as_str(), PAID);
        assert_eq!(choice.skipped[0].reason, SkipReason::WindowNotActive);

        // Scheduling after the rate-limit bound passes uses the account
        // again (the bound is data, not a sticky flag).
        let mut recovered = limited;
        recovered.now = test_timestamp("2026-09-23T14:00:00Z");
        let choice = ok(schedule_routing(&recovered));
        assert_eq!(choice.connection_id.as_str(), FREE);
    }

    #[test]
    fn spend_and_concurrency_limits_are_enforced() {
        // The per-account spend limit: usage already at the limit skips
        // the account with the named reason.
        let mut at_limit = test_inputs(free_and_paid());
        at_limit.policy.account_spend_limit = Some(3);
        let connection = test_connection(FREE);
        let task = test_task("task_01J8ZQ5V8K3T2B7N6X4R9DQPD3");
        ok(at_limit.ledger.record_usage(
            1,
            &connection,
            &task,
            "one model run",
            3,
            test_timestamp("2026-09-23T09:00:00Z"),
        ));
        let choice = ok(schedule_routing(&at_limit));
        assert_eq!(choice.connection_id.as_str(), PAID);
        assert_eq!(choice.policy_rule, PolicyRule::EscalatedToPaid);
        assert_eq!(choice.skipped[0].reason, SkipReason::SpendLimit);

        // The per-account concurrency limit: a loaded account is passed
        // over.
        let mut loaded = test_inputs(free_and_paid());
        loaded.policy.account_concurrency_limit = Some(1);
        loaded.load.per_account.insert(test_connection(FREE), 1);
        loaded.load.workspace = 1;
        let choice = ok(schedule_routing(&loaded));
        assert_eq!(choice.connection_id.as_str(), PAID);
        assert_eq!(choice.skipped[0].reason, SkipReason::ConcurrencyLimit);

        // The workspace spend limit stops ALL routing with a named error.
        let mut workspace_limit = test_inputs(free_and_paid());
        workspace_limit.policy.workspace_spend_limit = Some(3);
        ok(workspace_limit.ledger.record_usage(
            1,
            &connection,
            &task,
            "one model run",
            3,
            test_timestamp("2026-09-23T09:00:00Z"),
        ));
        let refused = err(schedule_routing(&workspace_limit));
        assert!(
            refused.to_string().contains("workspace spending limit"),
            "the refusal names the workspace limit: {refused}"
        );

        // The workspace concurrency limit stops all routing too.
        let mut workspace_busy = test_inputs(free_and_paid());
        workspace_busy.policy.workspace_concurrency_limit = Some(1);
        workspace_busy.load.workspace = 1;
        let refused = err(schedule_routing(&workspace_busy));
        assert!(
            refused.to_string().contains("tasks running at the same time"),
            "the refusal names the workspace concurrency limit: {refused}"
        );
    }

    #[test]
    fn overrides_choose_their_account_and_are_named() {
        // A task override: the task's preferred account is chosen with
        // the named rule — even when it is the paid account (the user
        // chose it; no escalation happened).
        let mut overridden = test_inputs(free_and_paid());
        ok(overridden.policy.set_task_override(
            &test_task("task_01J8ZQ5V8K3T2B7N6X4R9DQPD3"),
            crate::policy::TaskOverride {
                preferred_account: Some(test_connection(PAID)),
            },
        ));
        let choice = ok(schedule_routing(&overridden));
        assert_eq!(choice.connection_id.as_str(), PAID);
        assert_eq!(choice.policy_rule, PolicyRule::TaskOverride);
        assert!(!choice.escalated);
        assert_eq!(choice.rule_explanation, "You chose this account for this task.");

        // A provider override: the provider's preferred account is chosen
        // with the named rule.
        let mut provider_preferred = test_inputs(free_and_paid());
        ok(provider_preferred.policy.set_provider_override(
            "openai",
            crate::policy::ProviderOverride {
                preferred_account: Some(test_connection(PAID)),
                order: None,
            },
        ));
        let choice = ok(schedule_routing(&provider_preferred));
        assert_eq!(choice.policy_rule, PolicyRule::ProviderOverride);
        assert_eq!(
            choice.rule_explanation,
            "You set this provider to prefer this account."
        );

        // An override pointing at an unusable account is passed over with
        // its named reason — never a silent fallback to it.
        let mut unusable = test_inputs(free_and_paid());
        ok(unusable.policy.set_task_override(
            &test_task("task_01J8ZQ5V8K3T2B7N6X4R9DQPD3"),
            crate::policy::TaskOverride {
                preferred_account: Some(test_connection(FREE)),
            },
        ));
        unusable.accounts[0].quota = QuotaWindow {
            remaining: 0,
            limit: 5,
            window_start: test_timestamp("2026-09-23T00:00:00Z"),
            window_end: test_timestamp("2026-09-24T00:00:00Z"),
        };
        let choice = ok(schedule_routing(&unusable));
        assert_eq!(choice.connection_id.as_str(), PAID);
        assert_eq!(choice.policy_rule, PolicyRule::EscalatedToPaid);
        assert_eq!(choice.skipped[0].reason, SkipReason::Depleted);
    }

    #[test]
    fn no_candidates_and_no_usable_accounts_are_honest_errors() {
        // No account for the needed provider: a named error, never a
        // choice of some other provider's account.
        let mut foreign = test_inputs(vec![test_account(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPZ9",
            "Sandbox",
            TierKind::Paid,
            40,
            50,
        )]);
        foreign.accounts[0].provider_kind = "e2b".to_owned();
        let refused = err(schedule_routing(&foreign));
        assert!(
            refused.to_string().contains("no account is connected for provider"),
            "the refusal names the missing provider: {refused}"
        );

        // Every candidate unusable: every account is named with its
        // reason.
        let mut all_bad = test_inputs(free_and_paid());
        all_bad.accounts[0].quota = QuotaWindow {
            remaining: 0,
            limit: 5,
            window_start: test_timestamp("2026-09-23T00:00:00Z"),
            window_end: test_timestamp("2026-09-24T00:00:00Z"),
        };
        all_bad.accounts[1].rate_limited_until = Some(test_timestamp("2026-09-23T13:00:00Z"));
        let refused = err(schedule_routing(&all_bad));
        assert!(refused.to_string().contains("Personal account"));
        assert!(refused.to_string().contains("Work account"));
        assert!(refused.to_string().contains("quota is used up"));
        assert!(refused.to_string().contains("paused by the provider"));
    }

    #[test]
    fn choices_carry_no_credential_material_and_round_trip() {
        let choice = ok(schedule_routing(&test_inputs(free_and_paid())));
        let serialized = ok(serde_json::to_string(&choice));
        for marker in crate::key::CREDENTIAL_MARKERS {
            assert!(
                !serialized.contains(marker),
                "serialized choices must never contain credential material ({marker:?})"
            );
        }
        let reloaded: SchedulingChoice = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, choice);
    }
}
