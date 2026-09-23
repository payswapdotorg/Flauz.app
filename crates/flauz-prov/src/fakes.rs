//! Public in-memory fakes: the deterministic conformance surface for the
//! BYOP flows (kernel §7, work order PROV-001, part 3).
//!
//! Everything here is a pure function of its arguments — no I/O, no
//! wall-clock reads, no randomness. The fakes prove the full F7 loop on
//! deterministic data:
//!
//! - [`FakeSecretStore`] — the secret-store seam: mints `flausec_`
//!   references from key **material** and keeps the material NOWHERE
//!   (references only, forever — addendum §3). Material that looks like a
//!   real credential is refused outright, so a real key can never enter a
//!   fake flow.
//! - [`FakeProviderBackend`] — a deterministic provider: quota windows
//!   that deplete, a rate limit that trips after a fixed number of calls,
//!   and a free/paid account pair whose tiers escalate through the
//!   scheduler.
//! - [`FakeRoutingScenario`] — the end-to-end driver: connect an account
//!   (key entry → the seam mints a reference → the account appears with
//!   its tier and quota state), schedule a need, consume through the
//!   chosen account, deplete the free tier, and watch the honest
//!   escalation happen — all byte-deterministic.
//!
//! Downstream waves (the F7 wiring, the Lead's lab scenes) use these fakes
//! as the reference semantics for real provider flows.

use crate::account::{ProviderAccount, QuotaWindow, TierKind};
use crate::key::{ConnectionRef, SecretRef, TaskRef};
use crate::ledger::{QuotaLedger, UsageRecord};
use crate::policy::{RoutingOrder, RoutingPolicy};
use crate::scheduler::{RoutingNeed, SchedulingChoice, SchedulingInputs, SessionLoad};
use crate::time::Timestamp;
use crate::{MAX_NAME_BYTES, ProvError, ensure_non_empty, ensure_str_bound};

/// The fixed secret reference the fake secret store's FIRST mint returns
/// (deterministic — never entropy).
pub const FAKE_FREE_SECRET_REF: &str = "flausec_01J8ZQ5V8K3T2B7N6X4R9DQPF6";

/// The fixed secret reference the fake secret store's SECOND mint returns.
pub const FAKE_PAID_SECRET_REF: &str = "flausec_01J8ZQ5V8K3T2B7N6X4R9DQPG7";

/// The fixed connection reference of the fake free account.
pub const FAKE_FREE_CONNECTION: &str = "conn_01J8ZQ5V8K3T2B7N6X4R9DQPF6";

/// The fixed connection reference of the fake paid account.
pub const FAKE_PAID_CONNECTION: &str = "conn_01J8ZQ5V8K3T2B7N6X4R9DQPG7";

/// The fixed task reference the fake scenario routes for.
pub const FAKE_TASK: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPD3";

/// The fake secret-store seam: mints `flausec_` references from key
/// material and stores NOTHING else — the material is validated, turned
/// into a reference, and dropped. Deterministic: the Nth mint returns the
/// Nth fixed reference. Material that resembles a real credential
/// (`sk-...`, `ghp_...`, `Bearer ...` — the
/// [`CREDENTIAL_MARKERS`](crate::CREDENTIAL_MARKERS) family) is REFUSED
/// outright: the fake flows accept only clearly fake practice keys, so no
/// real secret can ever slip into a fixture, a log line, or serialized
/// state through them.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct FakeSecretStore {
    minted: Vec<SecretRef>,
}

impl FakeSecretStore {
    /// An empty fake secret store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mints a secret reference from key material. The material is never
    /// retained — the store holds references only, forever. The Nth mint
    /// is deterministic (the Nth fixed reference).
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the material is empty or
    /// unbounded, or when it resembles a real credential — the fake store
    /// accepts only clearly fake practice keys.
    pub fn mint(&mut self, material: &str) -> Result<SecretRef, ProvError> {
        ensure_non_empty("practice key material", material)?;
        ensure_str_bound("practice key material", material, MAX_NAME_BYTES)?;
        for marker in crate::key::CREDENTIAL_MARKERS {
            if material.contains(marker) {
                return Err(ProvError::invalid(format!(
                    "the fake secret store refuses material that looks like a real credential \
                     (contains {marker:?}); use a clearly fake practice key instead"
                )));
            }
        }
        let reference = match self.minted.len() {
            0 => SecretRef::parse(FAKE_FREE_SECRET_REF)?,
            1 => SecretRef::parse(FAKE_PAID_SECRET_REF)?,
            sequence => SecretRef::parse(&format!("flausec_fake{sequence:04}"))?,
        };
        self.minted.push(reference.clone());
        Ok(reference)
    }

    /// The references minted so far, in mint order (the store's ENTIRE
    /// state — material never appears anywhere).
    #[must_use]
    pub fn minted(&self) -> &[SecretRef] {
        &self.minted
    }
}

/// One account's simulated backend state: a quota window that depletes on
/// consumption and a rate limit that trips after a fixed number of calls.
/// The fake holds the live simulation state; every account RECORD the
/// platform sees is a fresh DATA snapshot derived from it
/// ([`FakeAccountBackend::snapshot_account`]).
#[derive(Debug, Clone)]
pub struct FakeAccountBackend {
    connection_id: ConnectionRef,
    account_label: String,
    tier: TierKind,
    remaining: u64,
    limit: u64,
    window_start: Timestamp,
    window_end: Timestamp,
    /// The number of calls after which the rate limit trips (None: the
    /// account never rate-limits).
    rate_limit_after: Option<u32>,
    calls_in_window: u32,
    rate_limited_until: Option<Timestamp>,
}

impl FakeAccountBackend {
    /// The account's current state as a data snapshot (tier + quota
    /// window + the rate-limit bound when tripped).
    #[must_use]
    pub fn snapshot_account(&self, provider_kind: &str, secret_ref: &SecretRef) -> ProviderAccount {
        ProviderAccount {
            v: crate::ProvVersion,
            version: 1,
            connection_id: self.connection_id.clone(),
            secret_ref: secret_ref.clone(),
            account_label: self.account_label.clone(),
            provider_kind: provider_kind.to_owned(),
            tier: self.tier,
            quota: QuotaWindow {
                remaining: self.remaining,
                limit: self.limit,
                window_start: self.window_start,
                window_end: self.window_end,
            },
            rate_limited_until: self.rate_limited_until,
        }
    }

    /// Consumes `units` through the account: the window depletes and the
    /// call counter advances. Over-limit consumption is refused (the
    /// depletion moment); the rate limit trips after the fixed number of
    /// calls.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the account is rate-limited at
    /// the moment of consumption or the remaining quota cannot cover the
    /// units.
    pub fn consume(
        &mut self,
        task: &TaskRef,
        consumption: &str,
        units: u64,
        at: Timestamp,
    ) -> Result<UsageRecord, ProvError> {
        if self
            .rate_limited_until
            .is_some_and(|until| at.is_before(until))
        {
            return Err(ProvError::invalid(format!(
                "account {} is paused by the provider right now",
                self.connection_id
            )));
        }
        if self.remaining < units {
            return Err(ProvError::invalid(format!(
                "account {} has {} of {} uses left; consuming {units} would overrun the \
                 window — the honest remaining state is a named deficit, never a silent one",
                self.connection_id, self.remaining, self.limit
            )));
        }
        self.remaining -= units;
        self.calls_in_window += 1;
        if let Some(after) = self.rate_limit_after
            && self.calls_in_window >= after
        {
            self.rate_limited_until = Some(self.window_end);
        }
        Ok(UsageRecord {
            v: crate::ProvVersion,
            seq: self.calls_in_window as u64,
            connection_id: self.connection_id.clone(),
            task_id: task.clone(),
            consumption: consumption.to_owned(),
            units,
            consumed_at: at,
        })
    }
}

/// A deterministic fake provider: a free account (small daily quota, a
/// rate limit that trips) and a paid account (a larger daily quota) — the
/// tier pair whose escalation the scheduler names.
#[derive(Debug, Clone)]
pub struct FakeProviderBackend {
    /// The provider's neutral kind label.
    pub provider_kind: String,
    free: FakeAccountBackend,
    paid: FakeAccountBackend,
}

impl FakeProviderBackend {
    /// Builds the fake provider around a daily window opening at `now`:
    /// a free account with 5 uses and a rate limit that trips on the 5th
    /// call, and a paid account with 100 uses.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the provider kind is not a
    /// lowercase token.
    pub fn new(provider_kind: &str, now: Timestamp) -> Result<Self, ProvError> {
        crate::ensure_kind_label("fake provider kind", provider_kind)?;
        let day_end = Timestamp::from_datetime(now.as_datetime() + chrono::Duration::hours(24));
        Ok(Self {
            provider_kind: provider_kind.to_owned(),
            free: FakeAccountBackend {
                connection_id: ConnectionRef::parse(FAKE_FREE_CONNECTION)?,
                account_label: "Personal account".to_owned(),
                tier: TierKind::Free,
                remaining: 5,
                limit: 5,
                window_start: now,
                window_end: day_end,
                rate_limit_after: Some(5),
                calls_in_window: 0,
                rate_limited_until: None,
            },
            paid: FakeAccountBackend {
                connection_id: ConnectionRef::parse(FAKE_PAID_CONNECTION)?,
                account_label: "Work account".to_owned(),
                tier: TierKind::Paid,
                remaining: 100,
                limit: 100,
                window_start: now,
                window_end: day_end,
                rate_limit_after: None,
                calls_in_window: 0,
                rate_limited_until: None,
            },
        })
    }

    /// The backend for one account, mutably (the scenario's consume path).
    fn backend_for_mut(
        &mut self,
        connection_id: &ConnectionRef,
    ) -> Option<&mut FakeAccountBackend> {
        if connection_id.as_str() == FAKE_FREE_CONNECTION {
            Some(&mut self.free)
        } else if connection_id.as_str() == FAKE_PAID_CONNECTION {
            Some(&mut self.paid)
        } else {
            None
        }
    }

    /// The account records the platform sees right now: fresh data
    /// snapshots of the free and paid accounts, in canonical connection
    /// order, carrying the given secret references.
    #[must_use]
    pub fn account_snapshots(
        &self,
        free_secret: &SecretRef,
        paid_secret: &SecretRef,
    ) -> Vec<ProviderAccount> {
        vec![
            self.free.snapshot_account(&self.provider_kind, free_secret),
            self.paid.snapshot_account(&self.provider_kind, paid_secret),
        ]
    }
}

/// The end-to-end fake routing scenario: the connect flow (key entry →
/// the seam mints a reference → the account appears with tier + quota
/// state), free-tier-first scheduling, consumption through the chosen
/// account, and the honest escalation when the free tier depletes.
/// Deterministic end to end: fixed references, fixed windows, no entropy.
#[derive(Debug, Clone)]
pub struct FakeRoutingScenario {
    store: FakeSecretStore,
    backend: FakeProviderBackend,
    ledger: QuotaLedger,
    policy: RoutingPolicy,
    free_secret: Option<SecretRef>,
    paid_secret: Option<SecretRef>,
}

impl FakeRoutingScenario {
    /// Builds the scenario: a fake provider kind, a window opening at
    /// `now`, an empty ledger and the default free-tier-first policy.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the provider kind is invalid.
    pub fn new(provider_kind: &str, now: Timestamp) -> Result<Self, ProvError> {
        Ok(Self {
            store: FakeSecretStore::new(),
            backend: FakeProviderBackend::new(provider_kind, now)?,
            ledger: QuotaLedger::new(),
            policy: RoutingPolicy::new(),
            free_secret: None,
            paid_secret: None,
        })
    }

    /// The connect flow on fakes: the API-key entry (practice material)
    /// passes through the secret-store seam, which mints a `flausec_`
    /// reference — and the account appears. The material is never stored;
    /// the account carries the reference and its tier + quota state.
    /// Connecting the free account first, then the paid account, matches
    /// the fixed reference order.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the material is refused (see
    /// [`FakeSecretStore::mint`]) or the account is already connected.
    pub fn connect_account(
        &mut self,
        tier: TierKind,
        material: &str,
        account_label: &str,
    ) -> Result<ProviderAccount, ProvError> {
        let already_connected = match tier {
            TierKind::Free => self.free_secret.is_some(),
            TierKind::Paid => self.paid_secret.is_some(),
        };
        if already_connected {
            return Err(ProvError::invalid(format!(
                "the {tier:?} account is already connected in this scenario"
            )));
        }
        let secret = self.store.mint(material)?;
        match tier {
            TierKind::Free => self.free_secret = Some(secret.clone()),
            TierKind::Paid => self.paid_secret = Some(secret.clone()),
        }
        let mut account = match tier {
            TierKind::Free => self
                .backend
                .free
                .snapshot_account(&self.backend.provider_kind, &secret),
            TierKind::Paid => self
                .backend
                .paid
                .snapshot_account(&self.backend.provider_kind, &secret),
        };
        if !account_label.is_empty() {
            account.account_label = account_label.to_owned();
        }
        account.validate()?;
        Ok(account)
    }

    /// The account records the platform sees right now (only connected
    /// tiers appear), in canonical connection order.
    #[must_use]
    pub fn accounts(&self) -> Vec<ProviderAccount> {
        let mut accounts = Vec::new();
        if let Some(free_secret) = &self.free_secret {
            accounts.push(
                self.backend
                    .free
                    .snapshot_account(&self.backend.provider_kind, free_secret),
            );
        }
        if let Some(paid_secret) = &self.paid_secret {
            accounts.push(
                self.backend
                    .paid
                    .snapshot_account(&self.backend.provider_kind, paid_secret),
            );
        }
        accounts
    }

    /// Schedules the scenario's task: free-tier-first over the connected
    /// accounts, at the given moment.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when no account can be chosen (see
    /// [`schedule_routing`](crate::schedule_routing)).
    pub fn schedule(&self, now: Timestamp) -> Result<SchedulingChoice, ProvError> {
        let need = RoutingNeed::new(
            &self.backend.provider_kind,
            &["terminal"],
            Some(TaskRef::parse(FAKE_TASK)?),
        )?;
        let inputs = SchedulingInputs {
            v: crate::ProvVersion,
            need,
            accounts: self.accounts(),
            policy: self.policy.clone(),
            ledger: self.ledger.clone(),
            load: SessionLoad::default(),
            now,
        };
        crate::schedule_routing(&inputs)
    }

    /// Consumes through the chosen account: the backend depletes, the
    /// ledger records the attributed usage, and the record is returned.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the account cannot cover the
    /// consumption (the honest depletion refusal).
    pub fn consume(
        &mut self,
        choice: &SchedulingChoice,
        consumption: &str,
        units: u64,
        at: Timestamp,
    ) -> Result<UsageRecord, ProvError> {
        let task = TaskRef::parse(FAKE_TASK)?;
        let mut record = self
            .backend
            .backend_for_mut(&choice.connection_id)
            .ok_or_else(|| {
                ProvError::invalid(format!(
                    "no fake backend for connection {}",
                    choice.connection_id
                ))
            })?
            .consume(&task, consumption, units, at)?;
        // The ledger assigns its own per-account sequences; the backend's
        // call counter stays the provider-side simulation.
        let recorded = self.ledger.record_usage(
            self.ledger.version(),
            &choice.connection_id,
            &task,
            consumption,
            units,
            at,
        )?;
        record.seq = recorded.seq;
        Ok(record)
    }

    /// The usage ledger (attribution history).
    #[must_use]
    pub fn ledger(&self) -> &QuotaLedger {
        &self.ledger
    }

    /// The routing policy (see + change through the surface).
    #[must_use]
    pub fn policy(&self) -> &RoutingPolicy {
        &self.policy
    }

    /// Changes the scenario's routing order (the surface's "change the
    /// order" path).
    pub fn set_order(&mut self, order: RoutingOrder) {
        self.policy.set_order(order);
    }

    /// The secret store (references minted so far — the entire state).
    #[must_use]
    pub fn store(&self) -> &FakeSecretStore {
        &self.store
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

    fn test_timestamp(value: &str) -> Timestamp {
        ok(Timestamp::parse(value))
    }

    fn scenario() -> FakeRoutingScenario {
        ok(FakeRoutingScenario::new(
            "openai",
            test_timestamp("2026-09-23T08:00:00Z"),
        ))
    }

    #[test]
    fn the_fake_secret_store_mints_references_and_refuses_real_material() {
        let mut store = FakeSecretStore::new();
        // The connect flow: practice material in, a flausec_ reference
        // out — deterministically.
        let first = ok(store.mint("practice-key-1"));
        assert_eq!(first.as_str(), FAKE_FREE_SECRET_REF);
        let second = ok(store.mint("practice-key-2"));
        assert_eq!(second.as_str(), FAKE_PAID_SECRET_REF);
        let third = ok(store.mint("practice-key-3"));
        assert_eq!(third.as_str(), "flausec_fake0002");

        // Material that resembles a real credential is REFUSED — the
        // rejection is the test (addendum §3).
        for refused in [
            "sk-proj-abcdefgh1234",
            "ghp_0123456789abcdef",
            "Bearer abc123",
            "password hunter2",
            "xoxb-012345",
        ] {
            assert!(
                store.mint(refused).is_err(),
                "material {refused:?} must be refused by the fake secret store"
            );
        }
        assert!(store.mint("").is_err());

        // The store's ENTIRE state is the reference list: no material
        // anywhere (the Debug scan mirrors the serialized-state law).
        let state = format!("{store:?}");
        assert!(!state.contains("practice-key-1"));
        assert!(!state.contains("sk-"));
        assert_eq!(store.minted().len(), 3);
        assert!(store.minted()[0].as_str().starts_with("flausec_"));
    }

    #[test]
    fn the_connect_flow_works_end_to_end_with_reference_only_storage() {
        let mut scenario = scenario();
        // Nothing is connected yet: the honest empty state.
        assert!(scenario.accounts().is_empty());

        // The connect flow: key entry → the seam mints a reference → the
        // account appears with tier + quota state.
        let free =
            ok(scenario.connect_account(TierKind::Free, "practice-key-1", "Personal account"));
        assert_eq!(free.tier, TierKind::Free);
        assert_eq!(free.secret_ref.as_str(), FAKE_FREE_SECRET_REF);
        assert_eq!(free.quota.remaining, 5);
        assert_eq!(free.quota.limit, 5);
        assert_eq!(free.account_label, "Personal account");
        ok(free.validate());

        let paid = ok(scenario.connect_account(TierKind::Paid, "practice-key-2", "Work account"));
        assert_eq!(paid.tier, TierKind::Paid);
        assert_eq!(paid.quota.remaining, 100);

        // Multi-account per provider: both accounts, canonical order.
        assert_eq!(scenario.accounts().len(), 2);
        assert_eq!(scenario.accounts()[0].tier, TierKind::Free);
        // Connecting the same tier twice is refused.
        assert!(
            scenario
                .connect_account(TierKind::Free, "practice-key-3", "Another")
                .is_err()
        );

        // No credential material anywhere in the connected state.
        let serialized = ok(serde_json::to_string(&scenario.accounts()));
        for marker in crate::key::CREDENTIAL_MARKERS {
            assert!(
                !serialized.contains(marker),
                "the fake connected state must never contain credential material ({marker:?})"
            );
        }
        assert!(!serialized.contains("practice-key"));
        assert!(serialized.contains("flausec_"));
    }

    #[test]
    fn the_scenario_depletes_and_escalates_honestly() {
        let mut world = scenario();
        ok(world.connect_account(TierKind::Free, "practice-key-1", "Personal account"));
        ok(world.connect_account(TierKind::Paid, "practice-key-2", "Work account"));
        let noon = test_timestamp("2026-09-23T12:00:00Z");

        // Free tier first: the free account is chosen while it has quota.
        let first = ok(world.schedule(noon));
        assert_eq!(first.connection_id.as_str(), FAKE_FREE_CONNECTION);
        assert_eq!(first.tier, TierKind::Free);
        assert_eq!(
            first.policy_rule,
            crate::scheduler::PolicyRule::DefaultOrder
        );
        assert!(!first.escalated);

        // Consume the free tier down: three single runs, then a double
        // run — four calls, five units, so the window empties WITHOUT
        // tripping the five-call rate limit. Attribution names the task
        // on every record.
        let connection = ok(ConnectionRef::parse(FAKE_FREE_CONNECTION));
        for units in [1u64, 1, 1] {
            let record = ok(world.consume(&first, "one model run", units, noon));
            assert_eq!(record.units, units);
            assert_eq!(record.task_id.as_str(), FAKE_TASK);
            assert_eq!(record.connection_id.as_str(), FAKE_FREE_CONNECTION);
        }
        assert_eq!(
            world
                .ledger()
                .units_since(&connection, first.quota_at_choice.window_start),
            3
        );

        // The honest remaining state: two uses left.
        let near = ok(world.schedule(noon));
        assert_eq!(near.connection_id.as_str(), FAKE_FREE_CONNECTION);
        assert_eq!(near.quota_at_choice.remaining, 2);

        // The depletion moment: the last double run empties the window,
        // then the next schedule ESCALATES — named, with the free
        // account as the named skipped alternative.
        ok(world.consume(&near, "one model run for the review pass", 2, noon));
        let escalated = ok(world.schedule(noon));
        assert_eq!(escalated.connection_id.as_str(), FAKE_PAID_CONNECTION);
        assert_eq!(escalated.tier, TierKind::Paid);
        assert!(escalated.escalated);
        assert_eq!(
            escalated.policy_rule,
            crate::scheduler::PolicyRule::EscalatedToPaid
        );
        assert_eq!(escalated.skipped.len(), 1);
        assert_eq!(
            escalated.skipped[0].connection_id.as_str(),
            FAKE_FREE_CONNECTION
        );
        assert_eq!(
            escalated.skipped[0].reason,
            crate::scheduler::SkipReason::Depleted
        );
        ok(escalated.validate());

        // Over-consumption is refused honestly — never a silent negative.
        assert!(
            world
                .consume(&escalated, "one model run", 200, noon)
                .is_err()
        );

        // Determinism: a fresh identical scenario replays byte-identical
        // choices.
        let mut replay = scenario();
        ok(replay.connect_account(TierKind::Free, "practice-key-1", "Personal account"));
        ok(replay.connect_account(TierKind::Paid, "practice-key-2", "Work account"));
        let replay_first = ok(replay.schedule(noon));
        assert_eq!(
            ok(serde_json::to_string(&replay_first)),
            ok(serde_json::to_string(&first))
        );
    }

    #[test]
    fn the_rate_limit_trips_and_is_a_named_skip() {
        let mut scenario = scenario();
        ok(scenario.connect_account(TierKind::Free, "practice-key-1", "Personal account"));
        ok(scenario.connect_account(TierKind::Paid, "practice-key-2", "Work account"));
        let noon = test_timestamp("2026-09-23T12:00:00Z");

        // Five single-use calls empty the window AND trip the free
        // account's rate limit (the window end is the bound).
        let choice = ok(scenario.schedule(noon));
        for _ in 0..5 {
            ok(scenario.consume(&choice, "one model run", 1, noon));
        }
        let after = ok(scenario.schedule(noon));
        // The free account is depleted AND rate-limited: the walk names
        // the first reason it checks (the rate limit).
        assert_eq!(after.connection_id.as_str(), FAKE_PAID_CONNECTION);
        assert_eq!(
            after.skipped[0].reason,
            crate::scheduler::SkipReason::RateLimited
        );
        assert_eq!(
            after.skipped[0].connection_id.as_str(),
            FAKE_FREE_CONNECTION
        );
        ok(after.validate());
    }

    #[test]
    fn the_policy_order_changes_and_the_scenario_follows() {
        let mut scenario = scenario();
        ok(scenario.connect_account(TierKind::Free, "practice-key-1", "Personal account"));
        ok(scenario.connect_account(TierKind::Paid, "practice-key-2", "Work account"));
        let noon = test_timestamp("2026-09-23T12:00:00Z");

        // Free first by default; paid-first after the change (the
        // surface's see + change path).
        assert_eq!(scenario.policy().order, RoutingOrder::FreeTierFirst);
        scenario.set_order(RoutingOrder::PaidFirst);
        assert_eq!(scenario.policy().order, RoutingOrder::PaidFirst);
        let choice = ok(scenario.schedule(noon));
        assert_eq!(choice.connection_id.as_str(), FAKE_PAID_CONNECTION);
        assert!(!choice.escalated);
        assert_eq!(
            choice.rule_explanation,
            "Paid accounts are tried first, and this one had quota left."
        );
    }
}
