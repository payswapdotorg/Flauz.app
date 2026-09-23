//! The [`ProviderAccount`] entity and its [`QuotaWindow`] snapshots (work
//! order PROV-001, part 1): the durable link between a user-owned
//! provider connection and the quota state the router schedules against.
//!
//! An account carries a **connection reference** (the frozen `conn_<ULID>`
//! format as data) and an opaque [`SecretRef`](crate::SecretRef) — the
//! `flausec_...` string minted by the secret-store seam when the user
//! connected the account — and nothing else. Credential material never
//! appears here (kernel §7; Wave-4 addendum §3).
//!
//! # Quota state is a data snapshot, never a live counter
//!
//! [`QuotaWindow`] records what the provider reported at a point in time:
//! how much is remaining, the limit, and the window's bounds. The crate
//! never mutates a window in place — depletion is a projection
//! ([`project_depletion`](crate::ledger::project_depletion)) and fresh
//! snapshots are new records supplied by callers. A window that does not
//! cover the scheduling moment is not silently treated as current: the
//! scheduler skips the account with a NAMED reason
//! ([`SkipReason::WindowNotActive`](crate::scheduler::SkipReason)).
//!
//! # Multi-account per provider
//!
//! Multiple accounts may exist for one provider kind (a personal free
//! account and a work paid account, for example). The
//! [`AccountStore`] holds them with the MOD-001 registry discipline:
//! created at version 1, +1 per durable mutation, optimistic concurrency
//! on update (a stale expected version is a
//! [`VersionConflict`](AccountStoreError::VersionConflict), never a
//! silent overwrite), and a canonical snapshot/restore round-trip.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::key::{ConnectionRef, SecretRef};
use crate::time::Timestamp;
use crate::{
    MAX_ACCOUNTS, MAX_NAME_BYTES, ProvError, ProvVersion, ensure_kind_label, ensure_non_empty,
    ensure_str_bound,
};

/// The tier of a provider account: which kind of quota it draws on. The
/// tier is part of every scheduling attribution — spending a paid tier
/// where a free tier was preferred must be NAMED (Wave-4 addendum §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TierKind {
    /// The account's free quota: used first by the default routing order.
    Free,
    /// A paid account: used when the policy (or the user) chooses it, or
    /// when free quota is used up — the escalation is always named.
    Paid,
}

impl TierKind {
    /// The user-facing noun for this tier (plain words: the tier is part
    /// of attribution copy, never an internal term).
    #[must_use]
    pub const fn user_label(self) -> &'static str {
        match self {
            Self::Free => "free tier",
            Self::Paid => "paid account",
        }
    }

    /// The scheduling preference rank under the default free-tier-first
    /// order (free ranks before paid).
    #[must_use]
    pub const fn default_order_rank(self) -> u8 {
        match self {
            Self::Free => 0,
            Self::Paid => 1,
        }
    }
}

/// A point-in-time quota snapshot for one account: how much remains, the
/// window's limit, and the window's bounds (caller-supplied timestamps —
/// the crate never reads the wall clock). Snapshots are DATA: depletion
/// is projected from them, never silently mutated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuotaWindow {
    /// How many units remain in the window at snapshot time.
    pub remaining: u64,
    /// The window's unit limit (at least 1: an account with no quota is
    /// not an account).
    pub limit: u64,
    /// When the window opened (inclusive), RFC 3339 UTC.
    pub window_start: Timestamp,
    /// When the window closes (exclusive), RFC 3339 UTC — strictly after
    /// `window_start`.
    pub window_end: Timestamp,
}

impl QuotaWindow {
    /// Validates the snapshot: `limit >= 1`, `remaining <= limit`, and
    /// `window_start < window_end`.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a bound is violated.
    pub fn validate(&self) -> Result<(), ProvError> {
        if self.limit == 0 {
            return Err(ProvError::invalid(
                "quota window limit must be at least 1 (an account with no quota is not an \
                 account)",
            ));
        }
        if self.remaining > self.limit {
            return Err(ProvError::invalid(format!(
                "quota window remaining {} exceeds its limit {}",
                self.remaining, self.limit
            )));
        }
        if !self.window_start.is_before(self.window_end) {
            return Err(ProvError::invalid(
                "quota window start must be strictly before its end",
            ));
        }
        Ok(())
    }

    /// Whether the window's quota is used up.
    #[must_use]
    pub const fn is_depleted(self) -> bool {
        self.remaining == 0
    }

    /// Whether the window covers the given moment (`window_start <= at <
    /// window_end`) — a snapshot that does not cover `at` cannot answer
    /// "is there quota left now" and must not be silently treated as
    /// current.
    #[must_use]
    pub fn covers(&self, at: Timestamp) -> bool {
        !at.is_before(self.window_start) && at.is_before(self.window_end)
    }

    /// Whether the remaining quota covers a projected consumption of
    /// `units`.
    #[must_use]
    pub const fn covers_units(self, units: u64) -> bool {
        self.remaining >= units
    }
}

/// A user-owned provider account: the connection-ref link, the account
/// label in user words, the provider kind, the tier, and the quota state
/// as a data snapshot. The account's identity is its connection
/// reference — the frozen `conn_<ULID>` format — so no new ID prefix is
/// invented (one account per connection; multi-account per provider means
/// multiple connections).
///
/// Durable entity rules (kernel §3, the MOD-001 registry pattern): the
/// account is created at version 1, every durable mutation through the
/// [`AccountStore`] increments by exactly 1, and updates carry the
/// caller's expected version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderAccount {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// The connection this account draws on (the frozen `conn_<ULID>`
    /// format as data). The account's identity.
    pub connection_id: ConnectionRef,
    /// The opaque secret reference (`flausec_...`) the secret-store seam
    /// minted when the account was connected. Credential material lives
    /// behind the secret store, never in this type.
    pub secret_ref: SecretRef,
    /// The user-facing label for the account (for example "Personal
    /// account") — the "whose account" half of every attribution.
    pub account_label: String,
    /// The neutral kind label of the provider (for example `openai`),
    /// matching the frozen `flauz-exec` provider-kind grammar.
    pub provider_kind: String,
    /// The account's tier (free/paid).
    pub tier: TierKind,
    /// The quota state at snapshot time — DATA, never a live counter.
    pub quota: QuotaWindow,
    /// When the provider last reported the account rate-limited, if it
    /// did: scheduling at a moment strictly before this bound skips the
    /// account with a NAMED reason. `null` when the account is not
    /// rate-limited.
    pub rate_limited_until: Option<Timestamp>,
}

impl ProviderAccount {
    /// Builds a new provider account at version 1. There is no way to
    /// construct an account carrying credential material: the secret
    /// reference is validated by [`SecretRef::parse`](crate::SecretRef::parse).
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the provider kind is not a
    /// lowercase token, the label is empty or unbounded, the secret
    /// reference is invalid (including credential-material shapes), or
    /// the quota snapshot fails [`QuotaWindow::validate`].
    pub fn new(
        connection_id: ConnectionRef,
        secret_ref: SecretRef,
        account_label: &str,
        provider_kind: &str,
        tier: TierKind,
        quota: QuotaWindow,
        rate_limited_until: Option<Timestamp>,
    ) -> Result<Self, ProvError> {
        connection_id.validate()?;
        secret_ref.validate()?;
        ensure_non_empty("account label", account_label)?;
        ensure_str_bound("account label", account_label, MAX_NAME_BYTES)?;
        ensure_kind_label("account provider kind", provider_kind)?;
        quota.validate()?;
        Ok(Self {
            v: ProvVersion,
            version: 1,
            connection_id,
            secret_ref,
            account_label: account_label.to_owned(),
            provider_kind: provider_kind.to_owned(),
            tier,
            quota,
            rate_limited_until,
        })
    }

    /// Validates the account record (the canonical rules of
    /// [`Self::new`], plus `version >= 1`).
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when any canonical rule is violated.
    pub fn validate(&self) -> Result<(), ProvError> {
        Self::new(
            self.connection_id.clone(),
            self.secret_ref.clone(),
            &self.account_label,
            &self.provider_kind,
            self.tier,
            self.quota,
            self.rate_limited_until,
        )?;
        if self.version == 0 {
            return Err(ProvError::invalid(
                "provider account version must be at least 1",
            ));
        }
        Ok(())
    }

    /// Whether the account is rate-limited at the given moment (the
    /// reported bound has not passed yet).
    #[must_use]
    pub fn is_rate_limited_at(&self, at: Timestamp) -> bool {
        self.rate_limited_until
            .is_some_and(|until| at.is_before(until))
    }
}

/// Store-level errors for the [`AccountStore`] (the MOD-001 registry
/// error family).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountStoreError {
    /// A value violates a canonical rule.
    Invalid {
        /// The reason the value was rejected.
        reason: String,
    },
    /// The connection is already connected as an account.
    Duplicate {
        /// The connection reference that collided.
        id: String,
    },
    /// The connection has no account in the store.
    NotFound {
        /// The connection reference that was missing.
        id: String,
    },
    /// The caller's expected version is stale (optimistic concurrency).
    VersionConflict {
        /// The connection reference whose version conflicted.
        id: String,
        /// The version the caller expected.
        expected_version: u64,
        /// The version actually stored.
        actual_version: u64,
    },
}

impl fmt::Display for AccountStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { reason } => write!(formatter, "invalid account state: {reason}"),
            Self::Duplicate { id } => write!(formatter, "connection {id} is already connected"),
            Self::NotFound { id } => {
                write!(formatter, "no account is connected for {id}")
            }
            Self::VersionConflict {
                id,
                expected_version,
                actual_version,
            } => write!(
                formatter,
                "account {id} changed since you read it (you expected version \
                 {expected_version}, it is at version {actual_version})"
            ),
        }
    }
}

impl Error for AccountStoreError {}

impl From<ProvError> for AccountStoreError {
    fn from(error: ProvError) -> Self {
        let ProvError::Invalid(reason) = error;
        Self::Invalid { reason }
    }
}

/// The canonical serialization of the entire account-store state: every
/// account keyed by its connection reference. Used for serialize → drop →
/// reload round-trips; the state survives with equality (kernel §4
/// round-trip law).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountSnapshot {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// All provider accounts, keyed by connection reference.
    pub accounts: BTreeMap<ConnectionRef, ProviderAccount>,
}

impl AccountSnapshot {
    /// The empty snapshot.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            v: ProvVersion,
            accounts: BTreeMap::new(),
        }
    }

    /// Validates the snapshot's structural invariants: map keys match the
    /// accounts they store and every account passes canonical validation.
    ///
    /// # Errors
    ///
    /// Returns [`AccountStoreError::Invalid`] when an invariant fails.
    pub fn validate(&self) -> Result<(), AccountStoreError> {
        if self.accounts.len() > MAX_ACCOUNTS {
            return Err(AccountStoreError::Invalid {
                reason: format!(
                    "the account store is bounded at {MAX_ACCOUNTS} accounts, found {}",
                    self.accounts.len()
                ),
            });
        }
        for (id, account) in &self.accounts {
            if id != &account.connection_id {
                return Err(AccountStoreError::Invalid {
                    reason: format!(
                        "account key {id} does not match its connection id {}",
                        account.connection_id
                    ),
                });
            }
            account.validate().map_err(AccountStoreError::from)?;
        }
        Ok(())
    }
}

/// The account store: the durable home of the user's connected provider
/// accounts, with the MOD-001 registry discipline — one account per
/// connection, optimistic concurrency on update, snapshot/restore for the
/// serialize → drop → reload round-trip. In-memory in this wave
/// (deterministic, no I/O, caller-supplied timestamps — kernel §7); the
/// persistence wiring arrives with the provider-connection wave.
#[derive(Debug, Default, Clone)]
pub struct AccountStore {
    accounts: BTreeMap<ConnectionRef, ProviderAccount>,
}

impl AccountStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Connects a new account (created at version 1). The account's
    /// connection must not already be connected — one account per
    /// connection, by identity.
    ///
    /// # Errors
    ///
    /// Returns [`AccountStoreError::Invalid`] when the account fails
    /// canonical validation or the store's bound is exceeded, and
    /// [`AccountStoreError::Duplicate`] when the connection already has
    /// an account.
    pub fn connect_account(
        &mut self,
        account: ProviderAccount,
    ) -> Result<ProviderAccount, AccountStoreError> {
        account.validate().map_err(AccountStoreError::from)?;
        if self.accounts.len() >= MAX_ACCOUNTS {
            return Err(AccountStoreError::Invalid {
                reason: format!(
                    "the account store is bounded at {MAX_ACCOUNTS} accounts; disconnect one \
                     before connecting another"
                ),
            });
        }
        if account.version != 1 {
            return Err(AccountStoreError::Invalid {
                reason: format!(
                    "account {} must be connected at version 1",
                    account.connection_id
                ),
            });
        }
        if self.accounts.contains_key(&account.connection_id) {
            return Err(AccountStoreError::Duplicate {
                id: account.connection_id.as_str().to_owned(),
            });
        }
        let id = account.connection_id.clone();
        self.accounts.insert(id, account.clone());
        Ok(account)
    }

    /// Looks up an account by its connection reference.
    #[must_use]
    pub fn account(&self, connection_id: &ConnectionRef) -> Option<&ProviderAccount> {
        self.accounts.get(connection_id)
    }

    /// All accounts, in canonical connection-reference order.
    #[must_use]
    pub fn accounts(&self) -> Vec<ProviderAccount> {
        self.accounts.values().cloned().collect()
    }

    /// The accounts of one provider kind, in canonical order (the
    /// multi-account surface).
    #[must_use]
    pub fn accounts_for(&self, provider_kind: &str) -> Vec<ProviderAccount> {
        self.accounts
            .values()
            .filter(|account| account.provider_kind == provider_kind)
            .cloned()
            .collect()
    }

    /// Updates an account (a durable mutation: +1 version). The update
    /// carries the caller's expected version; a stale version is a
    /// [`VersionConflict`](AccountStoreError::VersionConflict), never a
    /// silent overwrite. The connection
    /// reference is the account's identity and is immutable — a
    /// connection that needs different routing state is a new account.
    ///
    /// # Errors
    ///
    /// Returns [`AccountStoreError::NotFound`] when the connection has no
    /// account, [`AccountStoreError::VersionConflict`] on a stale
    /// expected version, and [`AccountStoreError::Invalid`] on a
    /// canonical violation or an identity change.
    pub fn update_account(
        &mut self,
        account: ProviderAccount,
    ) -> Result<ProviderAccount, AccountStoreError> {
        account.validate().map_err(AccountStoreError::from)?;
        let Some(stored) = self.accounts.get(&account.connection_id) else {
            return Err(AccountStoreError::NotFound {
                id: account.connection_id.as_str().to_owned(),
            });
        };
        if stored.version != account.version {
            return Err(AccountStoreError::VersionConflict {
                id: account.connection_id.as_str().to_owned(),
                expected_version: account.version,
                actual_version: stored.version,
            });
        }
        let mut next = account;
        next.version = stored.version + 1;
        self.accounts
            .insert(next.connection_id.clone(), next.clone());
        Ok(next)
    }

    /// Disconnects an account. An account is never removed silently while
    /// the ledger still attributes usage to it — the honest path removes
    /// the ledger's records explicitly first.
    ///
    /// # Errors
    ///
    /// Returns [`AccountStoreError::NotFound`] when the connection has no
    /// account, [`AccountStoreError::VersionConflict`] on a stale
    /// expected version, and [`AccountStoreError::Invalid`] while usage
    /// is still attributed to the connection.
    pub fn disconnect_account(
        &mut self,
        connection_id: &ConnectionRef,
        expected_version: u64,
        outstanding_usage: bool,
    ) -> Result<ProviderAccount, AccountStoreError> {
        let Some(stored) = self.accounts.get(connection_id) else {
            return Err(AccountStoreError::NotFound {
                id: connection_id.as_str().to_owned(),
            });
        };
        if stored.version != expected_version {
            return Err(AccountStoreError::VersionConflict {
                id: connection_id.as_str().to_owned(),
                expected_version,
                actual_version: stored.version,
            });
        }
        if outstanding_usage {
            return Err(AccountStoreError::Invalid {
                reason: format!(
                    "account {connection_id} still has usage attributed to it; settle the \
                     ledger before disconnecting"
                ),
            });
        }
        match self.accounts.remove(connection_id) {
            Some(account) => Ok(account),
            None => Err(AccountStoreError::NotFound {
                id: connection_id.as_str().to_owned(),
            }),
        }
    }

    /// Captures the entire store state as a canonical snapshot.
    #[must_use]
    pub fn snapshot(&self) -> AccountSnapshot {
        AccountSnapshot {
            v: ProvVersion,
            accounts: self.accounts.clone(),
        }
    }

    /// Rebuilds the store from a snapshot (serialize → drop → reload).
    ///
    /// # Errors
    ///
    /// Returns [`AccountStoreError::Invalid`] when the snapshot violates
    /// an invariant.
    pub fn restore(snapshot: AccountSnapshot) -> Result<Self, AccountStoreError> {
        snapshot.validate()?;
        Ok(Self {
            accounts: snapshot.accounts,
        })
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

    fn test_timestamp(value: &str) -> Timestamp {
        ok(Timestamp::parse(value))
    }

    fn test_window(remaining: u64, limit: u64, start: &str, end: &str) -> QuotaWindow {
        QuotaWindow {
            remaining,
            limit,
            window_start: test_timestamp(start),
            window_end: test_timestamp(end),
        }
    }

    fn test_account(
        connection: &str,
        label: &str,
        kind: &str,
        tier: TierKind,
        remaining: u64,
        limit: u64,
    ) -> ProviderAccount {
        ok(ProviderAccount::new(
            test_connection(connection),
            ok(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34")),
            label,
            kind,
            tier,
            test_window(
                remaining,
                limit,
                "2026-09-23T00:00:00Z",
                "2026-09-24T00:00:00Z",
            ),
            None,
        ))
    }

    #[test]
    fn quota_windows_validate_bounds_and_cover_moments() {
        ok(test_window(3, 5, "2026-09-23T00:00:00Z", "2026-09-24T00:00:00Z").validate());
        // remaining above the limit, zero limit, and empty bounds are all
        // refused.
        assert!(
            test_window(6, 5, "2026-09-23T00:00:00Z", "2026-09-24T00:00:00Z")
                .validate()
                .is_err()
        );
        assert!(
            test_window(0, 0, "2026-09-23T00:00:00Z", "2026-09-24T00:00:00Z")
                .validate()
                .is_err()
        );
        assert!(
            test_window(0, 5, "2026-09-24T00:00:00Z", "2026-09-23T00:00:00Z")
                .validate()
                .is_err()
        );
        let window = test_window(3, 5, "2026-09-23T00:00:00Z", "2026-09-24T00:00:00Z");
        assert!(!window.is_depleted());
        assert!(window.covers(test_timestamp("2026-09-23T12:00:00Z")));
        assert!(!window.covers(test_timestamp("2026-09-24T12:00:00Z")));
        assert!(!window.covers(test_timestamp("2026-09-22T12:00:00Z")));
        assert!(window.covers_units(3));
        assert!(!window.covers_units(4));
        let depleted = test_window(0, 5, "2026-09-23T00:00:00Z", "2026-09-24T00:00:00Z");
        assert!(depleted.is_depleted());
    }

    #[test]
    fn accounts_carry_references_and_snapshots_only() {
        let account = test_account(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "Personal account",
            "openai",
            TierKind::Free,
            3,
            5,
        );
        ok(account.validate());
        assert_eq!(account.tier.user_label(), "free tier");
        assert!(!account.is_rate_limited_at(test_timestamp("2026-09-23T12:00:00Z")));

        // The canonical serialization: references and snapshots, and
        // nothing else (byte-pinned).
        let serialized = ok(serde_json::to_string(&account));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,\"version\":1,\"connection_id\":\"conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0\",",
                "\"secret_ref\":\"flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34\",",
                "\"account_label\":\"Personal account\",\"provider_kind\":\"openai\",",
                "\"tier\":\"free\",\"quota\":{\"remaining\":3,\"limit\":5,",
                "\"window_start\":\"2026-09-23T00:00:00Z\",",
                "\"window_end\":\"2026-09-24T00:00:00Z\"},\"rate_limited_until\":null}"
            )
        );
        let reloaded: ProviderAccount = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, account);
        assert!(
            serde_json::from_str::<ProviderAccount>(&serialized.replace("\"v\":1,", "")).is_err(),
            "unknown or missing schema fields must be rejected"
        );

        // A rate-limited account reports it honestly, with the bound.
        let limited = ok(ProviderAccount::new(
            test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0"),
            ok(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34")),
            "Personal account",
            "openai",
            TierKind::Free,
            test_window(3, 5, "2026-09-23T00:00:00Z", "2026-09-24T00:00:00Z"),
            Some(test_timestamp("2026-09-23T13:00:00Z")),
        ));
        assert!(limited.is_rate_limited_at(test_timestamp("2026-09-23T12:00:00Z")));
        assert!(!limited.is_rate_limited_at(test_timestamp("2026-09-23T14:00:00Z")));

        // Canonical violations are rejected: credential-material-looking
        // references never even parse (the credentials-are-references law
        // enforced at the earliest layer — §3 of the Wave-4 addendum).
        assert!(SecretRef::parse("flausec_ghp_0123456789abcdef").is_err());
        assert!(
            ProviderAccount::new(
                test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0"),
                ok(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34")),
                "",
                "openai",
                TierKind::Free,
                test_window(3, 5, "2026-09-23T00:00:00Z", "2026-09-24T00:00:00Z"),
                None,
            )
            .is_err(),
            "an unlabeled account is ambient attribution — refused"
        );
    }

    #[test]
    fn account_store_follows_the_registry_discipline() {
        let mut store = AccountStore::new();
        let connected = ok(store.connect_account(test_account(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "Personal account",
            "openai",
            TierKind::Free,
            3,
            5,
        )));
        assert_eq!(connected.version, 1);
        // One account per connection, by identity.
        assert!(matches!(
            store.connect_account(test_account(
                "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
                "Other account",
                "openai",
                TierKind::Free,
                1,
                5,
            )),
            Err(AccountStoreError::Duplicate { .. })
        ));
        // Multi-account per provider: a second connection for the same
        // kind, plus an account of another kind.
        ok(store.connect_account(test_account(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPB1",
            "Work account",
            "openai",
            TierKind::Paid,
            90,
            100,
        )));
        ok(store.connect_account(test_account(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPC2",
            "Sandbox",
            "e2b",
            TierKind::Paid,
            40,
            50,
        )));
        assert_eq!(store.accounts_for("openai").len(), 2);
        assert_eq!(store.accounts().len(), 3);

        // Version rules: a fresh snapshot lands through update with +1;
        // a stale expected version conflicts, never silently overwrites.
        let mut fresh = store
            .account(&test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0"))
            .cloned()
            .unwrap_or_else(|| panic!("the account must be readable"));
        fresh.quota = test_window(1, 5, "2026-09-23T00:00:00Z", "2026-09-24T00:00:00Z");
        let updated = ok(store.update_account(fresh.clone()));
        assert_eq!(updated.version, 2);
        assert!(matches!(
            store.update_account(fresh),
            Err(AccountStoreError::VersionConflict { .. })
        ));

        // Disconnect: honest while usage is outstanding, clean after.
        let id = test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPC2");
        assert!(matches!(
            store.disconnect_account(&id, 1, true),
            Err(AccountStoreError::Invalid { .. })
        ));
        let removed = ok(store.disconnect_account(&id, 1, false));
        assert_eq!(removed.account_label, "Sandbox");
        assert!(store.account(&id).is_none());

        // Snapshot → drop → reload with equality.
        let snapshot = store.snapshot();
        let serialized = ok(serde_json::to_string(&snapshot));
        let reloaded: AccountSnapshot = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, snapshot);
        let restored = ok(AccountStore::restore(reloaded));
        assert_eq!(restored.snapshot(), store.snapshot());
        // A snapshot whose key does not match its account is refused.
        let mut broken = store.snapshot();
        if let Some(account) = broken
            .accounts
            .get_mut(&test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0"))
        {
            account.connection_id = test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPZ9");
        }
        assert!(AccountStore::restore(broken).is_err());
    }

    #[test]
    fn account_state_carries_no_credential_material() {
        let mut store = AccountStore::new();
        ok(store.connect_account(test_account(
            "conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0",
            "Personal account",
            "openai",
            TierKind::Free,
            3,
            5,
        )));
        let serialized = ok(serde_json::to_string(&store.snapshot()));
        for marker in crate::key::CREDENTIAL_MARKERS {
            assert!(
                !serialized.contains(marker),
                "serialized account state must never contain credential material ({marker:?})"
            );
        }
        assert!(
            serialized.contains("flausec_"),
            "the connection is stored as an opaque reference"
        );
    }
}
