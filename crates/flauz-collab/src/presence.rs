//! Presence records (work order COL-001 part 1): who is where, right
//! now — **bounded, replaceable, never an event stream of its own**
//! (the Wave-4 addendum §5 law).
//!
//! A [`PresenceRecord`] is a member ref + a surface locator + freshness
//! bounds, all as data: when the member was last seen, and how many
//! milliseconds may pass before the record is stale. Freshness is
//! enforced **on read** ([`PresenceRecord::is_stale`]) — the lease law
//! applied to presence: a stale record simply is not fresh, with no
//! background sweep and no mutation.
//!
//! The [`PresenceTable`] holds at most one record per member
//! ([`PresenceTable::update`] replaces, never appends): there is no
//! history, no sequence number, and no event vocabulary for presence —
//! the world event stream records membership and sharing changes, while
//! presence stays a bounded, replaceable snapshot. The table's size is
//! hard-bounded ([`MAX_PRESENCE_RECORDS`]).
//!
//! Presence carries no wall clock: every freshness evaluation takes the
//! caller's `now` (kernel §7 determinism).

use serde::{Deserialize, Serialize};

use crate::refs::{ActorRef, TaskRef, WorkspaceRef};
use crate::time::Timestamp;
use crate::{CollabError, CollabVersion, MAX_PRESENCE_RECORDS, ensure_list_bound};

/// The default freshness bound: a presence record older than sixty
/// seconds reads as stale (integer milliseconds — the kernel duration
/// law).
pub const DEFAULT_PRESENCE_STALE_AFTER_MS: u64 = 60_000;

/// Where a member currently is: the workspace at large, or one
/// specific task.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum SurfaceLocator {
    /// The member is in the workspace at large.
    Workspace,
    /// The member is viewing one specific task.
    Task {
        /// The task the member is viewing.
        task: TaskRef,
    },
}

/// The freshness bounds of a presence record, as data: when the member
/// was last seen, and how long the record may stand before it reads as
/// stale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresenceFreshness {
    /// When the member was last seen (RFC 3339 UTC, seconds precision;
    /// caller-supplied — never the wall clock).
    pub last_seen: Timestamp,
    /// How many milliseconds may pass before the record is stale
    /// (integer milliseconds — the kernel duration law).
    pub stale_after_ms: u64,
}

impl PresenceFreshness {
    /// Builds freshness bounds, rejecting non-positive windows (a
    /// zero-width window would make every record instantly stale — a
    /// caller bug, not a denial).
    pub fn new(last_seen: Timestamp, stale_after_ms: u64) -> Result<Self, CollabError> {
        if stale_after_ms == 0 {
            return Err(CollabError::invalid(
                "stale_after_ms must be greater than zero",
            ));
        }
        Ok(Self {
            last_seen,
            stale_after_ms,
        })
    }

    /// Whether the record is stale at the caller's `now` — expiry
    /// enforced on read, never mutated in place.
    #[must_use]
    pub fn is_stale(&self, now: &Timestamp) -> bool {
        !now.is_before(&self.last_seen.plus_milliseconds(self.stale_after_ms))
    }

    /// Validates the bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        if self.stale_after_ms == 0 {
            return Err(CollabError::invalid(
                "stale_after_ms must be greater than zero",
            ));
        }
        Ok(())
    }
}

/// One presence record: a member ref + a surface + freshness bounds —
/// replaceable data, never an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresenceRecord {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The workspace the record belongs to.
    pub workspace: WorkspaceRef,
    /// The member the record is about.
    pub member: ActorRef,
    /// Where the member is.
    pub locator: SurfaceLocator,
    /// The freshness bounds.
    pub freshness: PresenceFreshness,
}

impl PresenceRecord {
    /// Builds a presence record, validating member and freshness.
    pub fn new(
        workspace: WorkspaceRef,
        member: ActorRef,
        locator: SurfaceLocator,
        freshness: PresenceFreshness,
    ) -> Result<Self, CollabError> {
        member.validate()?;
        freshness.validate()?;
        Ok(Self {
            v: CollabVersion,
            workspace,
            member,
            locator,
            freshness,
        })
    }

    /// Whether the record is stale at the caller's `now`.
    #[must_use]
    pub fn is_stale(&self, now: &Timestamp) -> bool {
        self.freshness.is_stale(now)
    }

    /// Validates the record's bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        self.member.validate()?;
        self.freshness.validate()?;
        Ok(())
    }
}

/// The bounded, replaceable presence table: at most one record per
/// member — updates REPLACE, never append. There is no history, no
/// sequence, and no event stream here (addendum §5: presence is never
/// an event stream of its own; the world event stream records
/// membership and sharing changes instead).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresenceTable {
    workspace: WorkspaceRef,
    records: Vec<PresenceRecord>,
}

impl PresenceTable {
    /// Builds an empty table for one workspace.
    #[must_use]
    pub fn new(workspace: WorkspaceRef) -> Self {
        Self {
            workspace,
            records: Vec::new(),
        }
    }

    /// Updates the member's presence record, REPLACING any previous one.
    /// Returns whether a previous record was replaced. The record's
    /// workspace must match the table's; the table's size stays bounded.
    ///
    /// # Errors
    ///
    /// Returns [`CollabError`] when the record is malformed, belongs to
    /// another workspace, or would exceed the table's bound.
    pub fn update(&mut self, record: PresenceRecord) -> Result<bool, CollabError> {
        record.validate()?;
        if record.workspace != self.workspace {
            return Err(CollabError::invalid(format!(
                "the presence record belongs to workspace {:?}, not this table's {:?}",
                record.workspace, self.workspace
            )));
        }
        if let Some(existing) = self
            .records
            .iter_mut()
            .find(|existing| existing.member == record.member)
        {
            *existing = record;
            return Ok(true);
        }
        ensure_list_bound(
            "presence records",
            self.records.len() + 1,
            MAX_PRESENCE_RECORDS,
        )?;
        self.records.push(record);
        Ok(false)
    }

    /// The member's current record, if any.
    #[must_use]
    pub fn get(&self, member: &ActorRef) -> Option<&PresenceRecord> {
        self.records.iter().find(|record| record.member == *member)
    }

    /// Every record, in insertion order.
    #[must_use]
    pub fn records(&self) -> &[PresenceRecord] {
        &self.records
    }

    /// The records that are still fresh at the caller's `now` — expiry
    /// enforced on read (a stale record is not fresh; it is not mutated
    /// or removed).
    #[must_use]
    pub fn fresh_records(&self, now: &Timestamp) -> Vec<&PresenceRecord> {
        self.records
            .iter()
            .filter(|record| !record.is_stale(now))
            .collect()
    }

    /// The member's record when it is still fresh at the caller's `now`.
    #[must_use]
    pub fn fresh_get(&self, member: &ActorRef, now: &Timestamp) -> Option<&PresenceRecord> {
        self.get(member).filter(|record| !record.is_stale(now))
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

    fn workspace() -> WorkspaceRef {
        ok(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0"))
    }

    fn task() -> TaskRef {
        ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"))
    }

    fn at(moment: &str) -> Timestamp {
        ok(Timestamp::parse(moment))
    }

    #[test]
    fn locators_serialize_internally_tagged() {
        let workspace_locator = SurfaceLocator::Workspace;
        assert_eq!(
            ok(serde_json::to_string(&workspace_locator)),
            "{\"kind\":\"workspace\"}"
        );
        let task_locator = SurfaceLocator::Task { task: task() };
        let serialized = ok(serde_json::to_string(&task_locator));
        assert_eq!(
            serialized,
            "{\"kind\":\"task\",\"task\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\"}"
        );
        let reloaded: SurfaceLocator = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, task_locator);
        assert!(
            serde_json::from_str::<SurfaceLocator>("{\"kind\":\"surface\",\"task\":\"x\"}")
                .is_err(),
            "unknown locator kinds are rejected"
        );
    }

    #[test]
    fn records_serialize_canonically_and_reject_unknown_fields() {
        let record = ok(PresenceRecord::new(
            workspace(),
            ok(ActorRef::user("dev")),
            SurfaceLocator::Task { task: task() },
            ok(PresenceFreshness::new(at("2026-09-23T10:00:00Z"), 60_000)),
        ));
        let serialized = ok(serde_json::to_string(&record));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,",
                "\"workspace\":\"ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0\",",
                "\"member\":{\"kind\":\"user\",\"id\":\"dev\"},",
                "\"locator\":{\"kind\":\"task\",\"task\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\"},",
                "\"freshness\":{\"last_seen\":\"2026-09-23T10:00:00Z\",\"stale_after_ms\":60000}}"
            )
        );
        let reloaded: PresenceRecord = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, record);
        assert!(
            serde_json::from_str::<PresenceRecord>(
                &serialized.replace("\"stale_after_ms\":", "\"window\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn freshness_is_enforced_on_read_never_mutated() {
        let freshness = ok(PresenceFreshness::new(at("2026-09-23T10:00:00Z"), 60_000));
        assert!(!freshness.is_stale(&at("2026-09-23T10:00:59Z")));
        assert!(
            freshness.is_stale(&at("2026-09-23T10:01:01Z")),
            "one second past the window the record reads as stale"
        );
        // Exactly at the boundary: last_seen + window == now is stale
        // (the record no longer stands).
        assert!(freshness.is_stale(&at("2026-09-23T10:01:00Z")));
        // Zero-width windows are refused.
        assert!(PresenceFreshness::new(at("2026-09-23T10:00:00Z"), 0).is_err());
    }

    #[test]
    fn the_table_replaces_never_appends_and_stays_bounded() {
        let mut table = PresenceTable::new(workspace());
        let dev = ok(ActorRef::user("dev"));
        let first = ok(PresenceRecord::new(
            workspace(),
            dev.clone(),
            SurfaceLocator::Workspace,
            ok(PresenceFreshness::new(at("2026-09-23T10:00:00Z"), 60_000)),
        ));
        assert!(!ok(table.update(first.clone())));
        assert_eq!(table.records().len(), 1);
        // The member's second record REPLACES the first: no history, no
        // growth — presence is never an event stream.
        let second = ok(PresenceRecord::new(
            workspace(),
            dev.clone(),
            SurfaceLocator::Task { task: task() },
            ok(PresenceFreshness::new(at("2026-09-23T10:00:30Z"), 60_000)),
        ));
        assert!(ok(table.update(second.clone())));
        assert_eq!(table.records().len(), 1);
        assert_eq!(table.get(&dev), Some(&second));
        // Fresh reads respect the clock the caller supplies.
        assert!(table.fresh_get(&dev, &at("2026-09-23T10:00:31Z")).is_some());
        assert!(
            table.fresh_get(&dev, &at("2026-09-23T11:00:00Z")).is_none(),
            "a stale record is simply not fresh — never swept, never mutated"
        );
        // A record from another workspace is refused.
        let other = ok(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPC2"));
        let foreign = ok(PresenceRecord::new(
            other,
            dev.clone(),
            SurfaceLocator::Workspace,
            ok(PresenceFreshness::new(at("2026-09-23T10:00:00Z"), 60_000)),
        ));
        assert!(table.update(foreign).is_err());
    }
}
