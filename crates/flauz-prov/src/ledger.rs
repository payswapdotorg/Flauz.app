//! The [`QuotaLedger`] and the depletion projection (work order PROV-001,
//! part 1): per-account usage records with visible attribution, and the
//! honest remaining state under projected consumption.
//!
//! # Usage is attributed, never ambient
//!
//! Every [`UsageRecord`] names the logical consumption (in user words),
//! when it happened, how much it consumed, and **which task** it is
//! attributed to — the task reference crossing the seam as the frozen
//! `task_<ULID>` format. Records are append-only and immutable (kernel
//! §3's event discipline): per-account sequences are strictly increasing,
//! records are never mutated or deleted, and corrections would be new
//! records.
//!
//! # The depletion projection is honest, never silently negative
//!
//! [`project_depletion`] answers "if this consumption happens, what
//! remains?" with a [`DepletionProjection`]: the projected remaining
//! saturates at zero and any deficit is NAMED in the record — a projection
//! that would push the remaining state below zero can never be
//! constructed, and a hand-staged inconsistent projection fails
//! [`DepletionProjection::validate`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::account::QuotaWindow;
use crate::key::{ConnectionRef, TaskRef};
use crate::time::Timestamp;
use crate::{
    MAX_NAME_BYTES, MAX_USAGE_RECORDS_PER_ACCOUNT, ProvError, ProvVersion, ensure_non_empty,
    ensure_str_bound,
};

/// One unit of logical consumption attributed to a task through an
/// account: what was consumed, when, how much, and which task. Append-only
/// and immutable; the per-account sequence number is strictly increasing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageRecord {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// The strictly increasing per-account sequence number.
    pub seq: u64,
    /// The account whose quota was consumed (its connection reference).
    pub connection_id: ConnectionRef,
    /// The task the consumption is attributed to (the frozen `task_<ULID>`
    /// format as data — task identity is sacred, kernel §6).
    pub task_id: TaskRef,
    /// What logical consumption happened, in user words (for example
    /// "one model run for the research notes").
    pub consumption: String,
    /// How many units the consumption used.
    pub units: u64,
    /// When the consumption happened (caller-supplied).
    pub consumed_at: Timestamp,
}

impl UsageRecord {
    /// Validates the record's canonical rules: sequence at least 1, units
    /// at least 1 (a zero-unit record is not consumption), and a bounded
    /// non-empty consumption label.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when a rule is violated.
    pub fn validate(&self) -> Result<(), ProvError> {
        if self.seq == 0 {
            return Err(ProvError::invalid("usage record sequence must be at least 1"));
        }
        if self.units == 0 {
            return Err(ProvError::invalid(
                "usage record units must be at least 1 (a zero-unit record is not consumption)",
            ));
        }
        ensure_non_empty("usage consumption label", &self.consumption)?;
        ensure_str_bound("usage consumption label", &self.consumption, MAX_NAME_BYTES)?;
        self.connection_id.validate()?;
        self.task_id.validate()?;
        Ok(())
    }
}

/// The per-account usage ledger: append-only usage records keyed by
/// connection reference. A durable entity (kernel §3): the ledger is
/// created at version 1 and every append increments the version by
/// exactly 1 — an append against a stale expected version is a
/// [`ProvError`], never a silent overwrite.
///
/// The ledger records what WAS consumed; the quota state itself lives on
/// the accounts as data snapshots, and the future under projected
/// consumption is [`project_depletion`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuotaLedger {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// Durable entity version, starting at 1, +1 per append.
    pub version: u64,
    /// Usage records per account, keyed by connection reference, in
    /// append order (per-account sequences strictly increasing).
    pub records: BTreeMap<ConnectionRef, Vec<UsageRecord>>,
}

impl QuotaLedger {
    /// An empty ledger at version 1.
    #[must_use]
    pub fn new() -> Self {
        Self {
            v: ProvVersion,
            version: 1,
            records: BTreeMap::new(),
        }
    }

    /// The ledger's durable version.
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Appends a usage record to an account's stream, assigning the next
    /// strictly increasing per-account sequence. The append carries the
    /// caller's expected ledger version; a stale version is rejected,
    /// never a silent overwrite.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the record fails canonical
    /// validation, the per-account record bound is exceeded, or the
    /// expected version is stale.
    pub fn record_usage(
        &mut self,
        expected_version: u64,
        connection_id: &ConnectionRef,
        task_id: &TaskRef,
        consumption: &str,
        units: u64,
        consumed_at: Timestamp,
    ) -> Result<UsageRecord, ProvError> {
        if expected_version != self.version {
            return Err(ProvError::invalid(format!(
                "ledger version conflict: you expected version {expected_version}, it is at \
                 version {}",
                self.version
            )));
        }
        connection_id.validate()?;
        task_id.validate()?;
        ensure_non_empty("usage consumption label", consumption)?;
        ensure_str_bound("usage consumption label", consumption, MAX_NAME_BYTES)?;
        if units == 0 {
            return Err(ProvError::invalid(
                "usage record units must be at least 1 (a zero-unit record is not consumption)",
            ));
        }
        let account_records = self.records.entry(connection_id.clone()).or_default();
        if account_records.len() >= MAX_USAGE_RECORDS_PER_ACCOUNT {
            return Err(ProvError::invalid(format!(
                "account {connection_id} is bounded at {MAX_USAGE_RECORDS_PER_ACCOUNT} usage \
                 records"
            )));
        }
        let seq = match account_records.last() {
            None => 1,
            Some(last) => last.seq.checked_add(1).ok_or_else(|| {
                ProvError::invalid("usage record sequence overflowed")
            })?,
        };
        let record = UsageRecord {
            v: ProvVersion,
            seq,
            connection_id: connection_id.clone(),
            task_id: task_id.clone(),
            consumption: consumption.to_owned(),
            units,
            consumed_at,
        };
        account_records.push(record.clone());
        self.version = self
            .version
            .checked_add(1)
            .ok_or_else(|| ProvError::invalid("ledger version overflowed"))?;
        Ok(record)
    }

    /// The usage records attributed to one account, in append order.
    #[must_use]
    pub fn records_for(&self, connection_id: &ConnectionRef) -> &[UsageRecord] {
        self.records
            .get(connection_id)
            .map_or(&[], Vec::as_slice)
    }

    /// The units consumed through one account since a bound (inclusive) —
    /// the spend the account's spend limit is measured against.
    #[must_use]
    pub fn units_since(&self, connection_id: &ConnectionRef, since: Timestamp) -> u64 {
        self.records_for(connection_id)
            .iter()
            .filter(|record| !record.consumed_at.is_before(since))
            .map(|record| record.units)
            .sum()
    }

    /// The units consumed through EVERY account since a bound — the spend
    /// the workspace spend limit is measured against.
    #[must_use]
    pub fn total_units_since(&self, since: Timestamp) -> u64 {
        self.records
            .values()
            .flat_map(|records| records.iter())
            .filter(|record| !record.consumed_at.is_before(since))
            .map(|record| record.units)
            .sum()
    }

    /// The tasks usage is attributed to through one account, in first-seen
    /// order — the attribution surface's "who used this account" view.
    #[must_use]
    pub fn tasks_through(&self, connection_id: &ConnectionRef) -> Vec<TaskRef> {
        let mut tasks: Vec<TaskRef> = Vec::new();
        for record in self.records_for(connection_id) {
            if !tasks.contains(&record.task_id) {
                tasks.push(record.task_id.clone());
            }
        }
        tasks
    }

    /// Validates the ledger's canonical invariants: every record passes
    /// validation, per-account sequences are strictly increasing and
    /// contiguous from 1, every record belongs to the stream it is stored
    /// under, and the version is at least 1.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when an invariant fails.
    pub fn validate(&self) -> Result<(), ProvError> {
        if self.version == 0 {
            return Err(ProvError::invalid("ledger version must be at least 1"));
        }
        for (connection_id, records) in &self.records {
            if records.len() > MAX_USAGE_RECORDS_PER_ACCOUNT {
                return Err(ProvError::invalid(format!(
                    "account {connection_id} exceeds the {MAX_USAGE_RECORDS_PER_ACCOUNT} usage \
                     record bound"
                )));
            }
            for (position, record) in records.iter().enumerate() {
                record.validate()?;
                if record.connection_id != *connection_id {
                    return Err(ProvError::invalid(format!(
                        "a record for {connection_id} carries a foreign connection id {}",
                        record.connection_id
                    )));
                }
                if record.seq != position as u64 + 1 {
                    return Err(ProvError::invalid(format!(
                        "usage records for {connection_id} must be contiguous from 1; record at \
                         position {} carries sequence {}",
                        position + 1,
                        record.seq
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Default for QuotaLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// The honest remaining state under a projected consumption: the window,
/// the projection, the saturated remaining, and — when the projection
/// exceeds what is left — the NAMED deficit. A projection can never carry
/// a negative remaining: the deficit is the honest form of "more than
/// what is left".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepletionProjection {
    /// Contract schema version (`"v": 1`).
    pub v: ProvVersion,
    /// The window state the projection was computed from.
    pub window: QuotaWindow,
    /// The projected consumption.
    pub projected_units: u64,
    /// What remains after the projection — saturates at zero, never
    /// negative.
    pub projected_remaining: u64,
    /// Whether the projection uses the window up.
    pub depleted: bool,
    /// How far the projection exceeds what is left — zero unless
    /// `depleted` (the named deficit; never silently negative).
    pub deficit: u64,
}

impl DepletionProjection {
    /// Validates the projection's consistency with its window: the
    /// saturated remaining, the depleted flag, and the deficit must all
    /// match `project_depletion`'s law.
    ///
    /// # Errors
    ///
    /// Returns [`ProvError::Invalid`] when the projection is not the
    /// honest output for its inputs (a hand-staged lie fails here).
    pub fn validate(&self) -> Result<(), ProvError> {
        self.window.validate()?;
        let expected = project_depletion(&self.window, self.projected_units);
        if self.projected_remaining != expected.projected_remaining
            || self.depleted != expected.depleted
            || self.deficit != expected.deficit
        {
            return Err(ProvError::invalid(
                "the depletion projection must be the honest remaining state for its window and \
                 projection (saturated remaining, depleted flag, and named deficit all \
                 consistent)",
            ));
        }
        Ok(())
    }
}

/// The depletion projection (the ledger's forward view): given a quota
/// window state and a projected consumption, the honest remaining state.
/// The remaining saturates at zero and the deficit is NAMED — never
/// silently negative.
#[must_use]
pub fn project_depletion(window: &QuotaWindow, projected_units: u64) -> DepletionProjection {
    DepletionProjection {
        v: ProvVersion,
        window: *window,
        projected_units,
        projected_remaining: window.remaining.saturating_sub(projected_units),
        depleted: projected_units >= window.remaining,
        deficit: projected_units.saturating_sub(window.remaining),
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

    fn test_timestamp(value: &str) -> Timestamp {
        ok(Timestamp::parse(value))
    }

    fn test_window(remaining: u64, limit: u64) -> QuotaWindow {
        QuotaWindow {
            remaining,
            limit,
            window_start: test_timestamp("2026-09-23T00:00:00Z"),
            window_end: test_timestamp("2026-09-24T00:00:00Z"),
        }
    }

    #[test]
    fn the_depletion_projection_is_never_silently_negative() {
        // A projection that fits: everything named, nothing lost.
        let fits = project_depletion(&test_window(5, 5), 3);
        assert_eq!(fits.projected_remaining, 2);
        assert!(!fits.depleted);
        assert_eq!(fits.deficit, 0);
        ok(fits.validate());

        // The depletion moment: consuming exactly what is left.
        let exact = project_depletion(&test_window(5, 5), 5);
        assert_eq!(exact.projected_remaining, 0);
        assert!(exact.depleted);
        assert_eq!(exact.deficit, 0);
        ok(exact.validate());

        // Over-consumption: the remaining saturates at zero and the
        // deficit is NAMED — never a negative remaining.
        let over = project_depletion(&test_window(3, 5), 5);
        assert_eq!(over.projected_remaining, 0);
        assert!(over.depleted);
        assert_eq!(over.deficit, 2);
        ok(over.validate());
        let serialized = ok(serde_json::to_string(&over));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,\"window\":{\"remaining\":3,\"limit\":5,",
                "\"window_start\":\"2026-09-23T00:00:00Z\",",
                "\"window_end\":\"2026-09-24T00:00:00Z\"},\"projected_units\":5,",
                "\"projected_remaining\":0,\"depleted\":true,\"deficit\":2}"
            )
        );

        // A hand-staged lie fails validation: a depleted projection with
        // remaining left, and a deficit without depletion.
        let mut lie = project_depletion(&test_window(3, 5), 5);
        lie.projected_remaining = 2;
        assert!(lie.validate().is_err());
        let mut silent = project_depletion(&test_window(3, 5), 1);
        silent.deficit = 4;
        assert!(silent.validate().is_err());

        // Depleted windows project honestly too.
        let depleted = project_depletion(&test_window(0, 5), 1);
        assert_eq!(depleted.projected_remaining, 0);
        assert!(depleted.depleted);
        assert_eq!(depleted.deficit, 1);
    }

    #[test]
    fn usage_records_are_attributed_and_append_only() {
        let connection = test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0");
        let other = test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPB1");
        let task = test_task("task_01J8ZQ5V8K3T2B7N6X4R9DQPD3");
        let other_task = test_task("task_01J8ZQ5V8K3T2B7N6X4R9DQPE4");
        let mut ledger = QuotaLedger::new();
        assert_eq!(ledger.version(), 1);

        let first = ok(ledger.record_usage(
            1,
            &connection,
            &task,
            "one model run for the research notes",
            1,
            test_timestamp("2026-09-23T09:00:00Z"),
        ));
        assert_eq!(first.seq, 1);
        assert_eq!(ledger.version(), 2);
        let second = ok(ledger.record_usage(
            2,
            &connection,
            &other_task,
            "one model run for the review pass",
            2,
            test_timestamp("2026-09-23T10:00:00Z"),
        ));
        assert_eq!(second.seq, 2);
        let third = ok(ledger.record_usage(
            3,
            &other,
            &task,
            "one model run for the research notes",
            4,
            test_timestamp("2026-09-23T11:00:00Z"),
        ));
        assert_eq!(third.seq, 1);

        // A stale expected version is rejected — never a silent overwrite.
        assert!(ledger
            .record_usage(
                2,
                &connection,
                &task,
                "a stale append",
                1,
                test_timestamp("2026-09-23T12:00:00Z"),
            )
            .is_err());
        // Zero-unit and empty-label records are not consumption.
        assert!(ledger
            .record_usage(
                ledger.version(),
                &connection,
                &task,
                "nothing",
                0,
                test_timestamp("2026-09-23T12:00:00Z"),
            )
            .is_err());
        assert!(ledger
            .record_usage(
                ledger.version(),
                &connection,
                &task,
                "",
                1,
                test_timestamp("2026-09-23T12:00:00Z"),
            )
            .is_err());

        ok(ledger.validate());
        assert_eq!(ledger.records_for(&connection).len(), 2);
        assert_eq!(ledger.records_for(&other).len(), 1);
        assert_eq!(
            ledger.units_since(&connection, test_timestamp("2026-09-23T00:00:00Z")),
            3
        );
        assert_eq!(
            ledger.units_since(&connection, test_timestamp("2026-09-23T10:00:00Z")),
            2
        );
        assert_eq!(
            ledger.total_units_since(test_timestamp("2026-09-23T00:00:00Z")),
            7
        );
        assert_eq!(
            ledger.total_units_since(test_timestamp("2026-09-23T11:00:00Z")),
            4
        );
        // Attribution: the tasks that used each account, first-seen.
        assert_eq!(ledger.tasks_through(&connection), vec![task.clone(), other_task]);
        assert_eq!(ledger.tasks_through(&other), vec![task]);

        // Records are immutable and the sequences stay contiguous: a
        // hand-edited stream fails validation.
        let mut broken = ledger.clone();
        if let Some(records) = broken.records.get_mut(&connection) {
            records.remove(0);
        }
        assert!(broken.validate().is_err());
        let mut foreign = ledger.clone();
        if let Some(records) = foreign.records.get_mut(&connection) {
            records[0].connection_id = other.clone();
        }
        assert!(foreign.validate().is_err());

        // Canonical JSON round-trip.
        let serialized = ok(serde_json::to_string(&ledger));
        let reloaded: QuotaLedger = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, ledger);
        assert!(
            serde_json::from_str::<QuotaLedger>(&serialized.replace("\"v\":1,", "")).is_err(),
            "unknown fields must be rejected"
        );
    }

    #[test]
    fn serialized_ledger_state_carries_no_credential_material() {
        let connection = test_connection("conn_01J8ZQ5V8K3T2B7N6X4R9DQPA0");
        let task = test_task("task_01J8ZQ5V8K3T2B7N6X4R9DQPD3");
        let mut ledger = QuotaLedger::new();
        ok(ledger.record_usage(
            1,
            &connection,
            &task,
            "one model run for the research notes",
            1,
            test_timestamp("2026-09-23T09:00:00Z"),
        ));
        let serialized = ok(serde_json::to_string(&ledger));
        for marker in crate::key::CREDENTIAL_MARKERS {
            assert!(
                !serialized.contains(marker),
                "serialized ledger state must never contain credential material ({marker:?})"
            );
        }
    }
}
