//! The deterministic two-actor simulation (work order COL-001 part 2) —
//! the **F9 gate law proven in simulation** before any real transport
//! exists: two independent actors can concurrently use one workspace
//! without losing authorized state.
//!
//! Two [`ActorRef`]s drive the same store-facing seam ([`SimulatedStore`])
//! with **interleaved** operations, scripted as data in an
//! [`InterleavingTable`] (each step names its actor, its operation, and
//! the expected outcome). The runner ([`run_interleaved`]) is a pure
//! function of its inputs: no wall clock, no randomness, no I/O — the
//! same table over the same inputs always produces the same
//! [`SimulationLog`], byte-identical under canonical JSON (the
//! replayable law).
//!
//! # The four laws the simulation proves
//!
//! 1. **Permissions hold under interleaving** — every mutating step
//!    passes through the authorization evaluator first; a denied step
//!    never mutates the store ([`verify_f9_laws`] re-derives this from
//!    the log).
//! 2. **Private records stay private to their actor** — the
//!    [`SimulatedOperation::ReadProjection`] step answers through the
//!    §6 projection filter: a member-private entry owned by another
//!    actor is structurally absent from the projection; writing another
//!    member's private entry is denied with the named
//!    [`DenialReason::PrivateRecord`] reason (even for the owner).
//! 3. **Authorized state is never lost** — every legal operation either
//!    lands or is denied-with-reason ([`DenialReason`] names each one);
//!    writes go through the optimistic-concurrency seam
//!    (`expected_version`), so a stale interleaved write is refused with
//!    the named [`DenialReason::VersionConflict`] reason — **never a
//!    silent lost-update**. [`verify_f9_laws`] counts every landed write
//!    against the final store: nothing vanishes, nothing is invented.
//! 4. **The interleaving log is canonical-JSON replayable** — the log
//!    round-trips byte-identically and two runs of the same inputs
//!    produce byte-identical logs.
//!
//! # The registered event vocabulary (frozen-format discipline)
//!
//! The records this simulation's real-world counterpart appends through
//! the existing world-store seam register the `event_type` vocabulary in
//! [`event_types`]: `member.added`, `member.role_changed`,
//! `member.removed`, `permission.granted`, `permission.revoked` and
//! `sharing.changed` — all under the frozen `<entity>.<verb_past>`
//! grammar. Presence registers NO event type (addendum §5).

use serde::{Deserialize, Serialize};

use crate::membership::{
    AccessLevel, RoleLattice, SurfaceFamily, WorkspaceMembership, validate_roster,
};
use crate::permission::{
    AllowRule, AuthorizationRequest, DecisionOutcome, DenialReason, PermissionGrant, authorize,
};
use crate::policy::{
    ContextVisibility, SharedFilesystemMode, Visibility, WorktreePolicy, visible_to_member,
};
use crate::presence::{PresenceFreshness, PresenceRecord, PresenceTable, SurfaceLocator};
use crate::refs::{ActorRef, TaskRef, WorkspaceRef};
use crate::time::Timestamp;
use crate::{
    CollabError, CollabVersion, MAX_INTERLEAVING_STEPS, MAX_STORE_ENTRIES, MAX_STORE_VALUE_BYTES,
    ensure_list_bound, ensure_name, ensure_str_bound,
};

/// The `event_type` vocabulary this crate registers for the world-store
/// seam (kernel §5: each work order registers its event types in its
/// crate docs/tests; the grammar `<entity>.<verb_past>` applies).
///
/// Presence deliberately registers nothing — presence records are
/// bounded and replaceable, never an event stream (addendum §5).
pub mod event_types {
    /// A member was added to a workspace (`workspace` stream).
    pub const MEMBER_ADDED: &str = "member.added";
    /// A member's role changed (`workspace` stream).
    pub const MEMBER_ROLE_CHANGED: &str = "member.role_changed";
    /// A member was removed from a workspace (`workspace` stream).
    pub const MEMBER_REMOVED: &str = "member.removed";
    /// An explicit permission grant was recorded (`workspace` stream).
    pub const PERMISSION_GRANTED: &str = "permission.granted";
    /// An explicit permission grant was revoked (`workspace` stream).
    pub const PERMISSION_REVOKED: &str = "permission.revoked";
    /// A task's sharing posture changed (`task` stream).
    pub const SHARING_CHANGED: &str = "sharing.changed";
}

/// One entry in the simulated store: a bounded key, a bounded value, the
/// entry's version (1 at create, +1 per landed update — kernel §3), its
/// owner, and its visibility. The store seam mimics the world store's
/// laws: optimistic concurrency on `expected_version`, and the §6
/// projection filter on reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreEntry {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The entry's unique key (a short label, e.g. "brief").
    pub key: String,
    /// The entry's value (bounded text).
    pub value: String,
    /// The entry's version: 1 at create, +1 per landed update.
    pub version: u64,
    /// The actor that owns the entry.
    pub owner: ActorRef,
    /// Whether the entry is member-private or workspace-shared.
    pub visibility: Visibility,
}

impl StoreEntry {
    /// Builds an entry, validating bounds.
    pub fn new(
        key: &str,
        value: &str,
        version: u64,
        owner: ActorRef,
        visibility: Visibility,
    ) -> Result<Self, CollabError> {
        ensure_name("store key", key)?;
        ensure_str_bound("store value", value, MAX_STORE_VALUE_BYTES)?;
        owner.validate()?;
        if version == 0 {
            return Err(CollabError::invalid("store entry versions start at 1"));
        }
        Ok(Self {
            v: CollabVersion,
            key: key.to_owned(),
            value: value.to_owned(),
            version,
            owner,
            visibility,
        })
    }

    /// Validates the entry's bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        StoreEntry::new(
            &self.key,
            &self.value,
            self.version,
            self.owner.clone(),
            self.visibility,
        )
        .map(|_| ())
    }
}

/// The simulated store-facing seam: the in-memory, deterministic stand-in
/// for the world store's task-state APIs. Writes are permission-checked
/// by the caller (the runner), private-protected structurally, and
/// version-guarded — the no-lost-update law.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulatedStore {
    entries: Vec<StoreEntry>,
}

impl SimulatedStore {
    /// Builds a store from initial entries, validating bounds and key
    /// uniqueness.
    pub fn new(entries: Vec<StoreEntry>) -> Result<Self, CollabError> {
        ensure_list_bound("store entries", entries.len(), MAX_STORE_ENTRIES)?;
        for entry in &entries {
            entry.validate()?;
        }
        for (index, entry) in entries.iter().enumerate() {
            if entries[..index].iter().any(|other| other.key == entry.key) {
                return Err(CollabError::invalid(format!(
                    "the store carries more than one entry keyed {:?}",
                    entry.key
                )));
            }
        }
        Ok(Self { entries })
    }

    /// The entry for `key`, if any.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&StoreEntry> {
        self.entries.iter().find(|entry| entry.key == key)
    }

    /// Every entry, in insertion order.
    #[must_use]
    pub fn entries(&self) -> &[StoreEntry] {
        &self.entries
    }

    /// The actor's permission-filtered projection of the same durable
    /// state (the §6 law): exactly the entries `visible_to_member`
    /// admits for this viewer.
    #[must_use]
    pub fn projection_for(&self, viewer: &ActorRef) -> Vec<ProjectionRow> {
        self.entries
            .iter()
            .filter(|entry| visible_to_member(&entry.owner, entry.visibility, viewer))
            .map(|entry| ProjectionRow {
                key: entry.key.clone(),
                owner: entry.owner.clone(),
                visibility: entry.visibility,
                version: entry.version,
            })
            .collect()
    }

    /// Applies one write through the optimistic-concurrency seam.
    /// Returns the new version on landing, or the named denial reason —
    /// never a silent lost-update.
    ///
    /// The private-record law (§6) is checked BEFORE the version guard:
    /// another member's private entry is untouchable regardless of the
    /// expected version.
    fn apply_write(
        &mut self,
        actor: &ActorRef,
        key: &str,
        value: &str,
        expected_version: u64,
        visibility: Visibility,
    ) -> Result<u64, DenialReason> {
        ensure_name("store key", key)
            .and(ensure_str_bound(
                "store value",
                value,
                MAX_STORE_VALUE_BYTES,
            ))
            .map_err(|_| DenialReason::VersionConflict)?;
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key == key) {
            if entry.visibility == Visibility::MemberPrivate && entry.owner != *actor {
                return Err(DenialReason::PrivateRecord);
            }
            if expected_version != entry.version {
                return Err(DenialReason::VersionConflict);
            }
            entry.value = value.to_owned();
            entry.visibility = visibility;
            entry.version += 1;
            return Ok(entry.version);
        }
        // Creation: expected_version 0 lands the entry at version 1.
        if expected_version != 0 {
            return Err(DenialReason::VersionConflict);
        }
        if self.entries.len() >= MAX_STORE_ENTRIES {
            return Err(DenialReason::VersionConflict);
        }
        let entry = StoreEntry::new(key, value, 1, actor.clone(), visibility)
            .map_err(|_| DenialReason::VersionConflict)?;
        self.entries.push(entry);
        Ok(1)
    }
}

/// One row of a projection: what one actor saw of the same durable
/// state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionRow {
    /// The entry's key.
    pub key: String,
    /// The entry's owner.
    pub owner: ActorRef,
    /// The entry's visibility.
    pub visibility: Visibility,
    /// The entry's version at read time.
    pub version: u64,
}

/// One scripted operation against the simulated seam. Every mutating
/// operation is permission-checked by the runner before it touches the
/// store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum SimulatedOperation {
    /// Probe the authorization evaluator directly (the permission
    /// check, surfaced as a decision).
    RequestAccess {
        /// The surface family being probed.
        surface: SurfaceFamily,
        /// The access level being probed.
        action: AccessLevel,
    },
    /// Write one store entry through the OCC seam (permission-checked
    /// at `tasks`/`contribute`; creation carries `expected_version` 0
    /// and lands at version 1).
    WriteSharedState {
        /// The entry's key.
        key: String,
        /// The new value.
        value: String,
        /// The version the writer believes the entry is at.
        expected_version: u64,
        /// The entry's visibility (set on create, restated on update).
        visibility: Visibility,
    },
    /// Read the actor's permission-filtered projection of the store.
    ReadProjection,
    /// Update the actor's presence (replaces their previous record —
    /// presence is never an event stream).
    UpdatePresence {
        /// Where the actor is.
        locator: SurfaceLocator,
    },
    /// Change the task's sharing posture (permission-checked at
    /// `tasks`/`manage`).
    SetSharing {
        /// The new shared-filesystem mode.
        shared_filesystem: SharedFilesystemMode,
        /// The new memory visibility.
        memory: Visibility,
        /// The new artifacts visibility.
        artifacts: Visibility,
    },
    /// Read the task's sharing posture back.
    ReadSharing,
}

impl SimulatedOperation {
    /// The authorization the runner requires for this operation: the
    /// surface family and access level that must be admitted before the
    /// operation may touch the seam.
    #[must_use]
    pub const fn required_access(&self) -> (SurfaceFamily, AccessLevel) {
        match self {
            Self::RequestAccess { .. } => (SurfaceFamily::Tasks, AccessLevel::None),
            Self::WriteSharedState { .. } => (SurfaceFamily::Tasks, AccessLevel::Contribute),
            Self::ReadProjection | Self::ReadSharing => (SurfaceFamily::Tasks, AccessLevel::Read),
            Self::UpdatePresence { .. } => (SurfaceFamily::Members, AccessLevel::Read),
            Self::SetSharing { .. } => (SurfaceFamily::Tasks, AccessLevel::Manage),
        }
    }
}

/// The expected outcome of one scripted step — the table is a table of
/// interleaved operations WITH expected outcomes (the deterministic
/// prediction the run must reproduce).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum ExpectedOutcome {
    /// The access probe comes back allowed via this rule.
    Allowed {
        /// The named allow rule.
        via: AllowRule,
    },
    /// The step is denied with this named reason.
    Denied {
        /// The named denial reason — never silent.
        reason: DenialReason,
    },
    /// The write lands at this version.
    Written {
        /// The version after the write lands.
        version: u64,
    },
    /// The projection contains exactly these keys, in store order.
    Projection {
        /// The visible keys.
        keys: Vec<String>,
    },
    /// The presence record was placed (created or replaced).
    PresenceUpdated,
    /// The sharing posture read back as these values.
    SharingRead {
        /// The shared-filesystem mode.
        shared_filesystem: SharedFilesystemMode,
        /// The memory visibility.
        memory: Visibility,
        /// The artifacts visibility.
        artifacts: Visibility,
    },
}

/// One scripted step: which actor acts, what they do, and what the
/// deterministic run must produce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterleavedStep {
    /// The acting actor (one of the two driving the seam — or an
    /// outsider, whose steps must come back `not_a_member`).
    pub actor: ActorRef,
    /// The operation.
    pub op: SimulatedOperation,
    /// The expected outcome.
    pub expected: ExpectedOutcome,
}

/// The scripted interleaving table: the two actors' operations in the
/// exact order the simulation drives the seam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterleavingTable {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The workspace the simulation runs against.
    pub workspace: WorkspaceRef,
    /// The task the sharing posture and presence locators belong to.
    pub task: TaskRef,
    /// The steps, in order.
    pub steps: Vec<InterleavedStep>,
}

impl InterleavingTable {
    /// Builds a table, validating bounds and actor references.
    pub fn new(
        workspace: WorkspaceRef,
        task: TaskRef,
        steps: Vec<InterleavedStep>,
    ) -> Result<Self, CollabError> {
        ensure_list_bound("interleaving steps", steps.len(), MAX_INTERLEAVING_STEPS)?;
        for step in &steps {
            step.actor.validate()?;
        }
        workspace.validate()?;
        task.validate()?;
        Ok(Self {
            v: CollabVersion,
            workspace,
            task,
            steps,
        })
    }

    /// Validates the table's bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        InterleavingTable::new(
            self.workspace.clone(),
            self.task.clone(),
            self.steps.clone(),
        )
        .map(|_| ())
    }
}

/// The simulation inputs: everything the deterministic run needs, as
/// data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationInputs {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The workspace the simulation runs against.
    pub workspace: WorkspaceRef,
    /// The workspace roster.
    pub memberships: Vec<WorkspaceMembership>,
    /// The explicit grants.
    pub grants: Vec<PermissionGrant>,
    /// The role lattice.
    pub lattice: RoleLattice,
    /// The store's initial entries.
    pub entries: Vec<StoreEntry>,
    /// The task's initial worktree posture.
    pub worktree: WorktreePolicy,
    /// The task's initial context visibility.
    pub context: ContextVisibility,
    /// The presence freshness window every presence step uses
    /// (milliseconds).
    pub presence_stale_after_ms: u64,
    /// The deterministic clock every freshness evaluation uses.
    pub now: Timestamp,
}

impl SimulationInputs {
    /// Validates the inputs: roster, grants, lattice, store, policies,
    /// workspace/task consistency.
    pub fn validate(&self) -> Result<(), CollabError> {
        validate_roster("memberships", &self.memberships)?;
        crate::permission::validate_grants("grants", &self.grants)?;
        self.lattice.validate()?;
        SimulatedStore::new(self.entries.clone())?;
        self.worktree.validate()?;
        self.context.validate()?;
        self.workspace.validate()?;
        if self.presence_stale_after_ms == 0 {
            return Err(CollabError::invalid(
                "presence_stale_after_ms must be greater than zero",
            ));
        }
        for membership in &self.memberships {
            if membership.workspace != self.workspace {
                return Err(CollabError::invalid(format!(
                    "membership of {:?} belongs to workspace {:?}, not the simulation's {:?}",
                    membership.actor.id, membership.workspace, self.workspace
                )));
            }
        }
        for grant in &self.grants {
            if grant.workspace != self.workspace {
                return Err(CollabError::invalid(format!(
                    "grant for {:?} belongs to workspace {:?}, not the simulation's {:?}",
                    grant.actor.id, grant.workspace, self.workspace
                )));
            }
        }
        Ok(())
    }

    /// The simulation's task (both policies carry it).
    #[must_use]
    pub fn task(&self) -> TaskRef {
        self.worktree.task.clone()
    }
}

/// One record in the replayable interleaving log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationRecord {
    /// The step's index into the table (0-based).
    pub step: usize,
    /// The acting actor.
    pub actor: ActorRef,
    /// The operation that ran (echoed, so the log replays standalone).
    pub op: SimulatedOperation,
    /// The outcome.
    pub outcome: SimulationOutcome,
}

/// The outcome of one simulated step: landed (with its result) or
/// denied (with the named reason).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum SimulationOutcome {
    /// The access probe's decision (allow or deny, with its named rule).
    Access {
        /// The decision's outcome.
        outcome: DecisionOutcome,
    },
    /// The write landed at this version.
    Written {
        /// The written key.
        key: String,
        /// The version after the write.
        version: u64,
    },
    /// The step was denied — with the named reason (never silent).
    Denied {
        /// The named denial reason.
        reason: DenialReason,
    },
    /// The actor's projection of the same durable state.
    Projection {
        /// The visible rows, in store order.
        rows: Vec<ProjectionRow>,
    },
    /// The actor's presence record was placed (created or replaced).
    PresenceUpdated {
        /// The member whose presence was updated.
        member: ActorRef,
        /// Where they are.
        locator: SurfaceLocator,
    },
    /// The task's sharing posture (after a change, or as read back).
    Sharing {
        /// The shared-filesystem mode.
        shared_filesystem: SharedFilesystemMode,
        /// The memory visibility.
        memory: Visibility,
        /// The artifacts visibility.
        artifacts: Visibility,
    },
}

impl SimulationOutcome {
    /// The named denial reason, when this outcome is a denial.
    #[must_use]
    pub const fn denial_reason(&self) -> Option<DenialReason> {
        match self {
            Self::Access { outcome } => outcome.denial_reason(),
            Self::Denied { reason } => Some(*reason),
            _ => None,
        }
    }

    /// Whether the step landed (mutated state or answered a read).
    #[must_use]
    pub const fn landed(&self) -> bool {
        !matches!(self, Self::Denied { .. })
    }
}

/// The replayable interleaving log: one record per scripted step, in
/// order, carrying everything a replay needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationLog {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The workspace the simulation ran against.
    pub workspace: WorkspaceRef,
    /// The task the sharing posture belonged to.
    pub task: TaskRef,
    /// One record per step, in order.
    pub records: Vec<SimulationRecord>,
}

/// The simulation result: the replayable log plus the final seam state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationResult {
    /// The replayable interleaving log.
    pub log: SimulationLog,
    /// The final store state.
    pub store: SimulatedStore,
    /// The final presence table.
    pub presence: PresenceTable,
    /// The final worktree posture.
    pub worktree: WorktreePolicy,
    /// The final context visibility.
    pub context: ContextVisibility,
}

/// Runs the scripted interleaving deterministically: two actors driving
/// the same store-facing seam, every step permission-checked, every
/// denial named, every landed write version-guarded.
///
/// # Errors
///
/// Returns [`CollabError`] when the inputs or the table are invalid, or
/// the table's workspace/task do not match the inputs'.
pub fn run_interleaved(
    inputs: &SimulationInputs,
    table: &InterleavingTable,
) -> Result<SimulationResult, CollabError> {
    inputs.validate()?;
    table.validate()?;
    if table.workspace != inputs.workspace {
        return Err(CollabError::invalid(
            "the interleaving table's workspace must match the inputs'",
        ));
    }
    if table.task != inputs.task() {
        return Err(CollabError::invalid(
            "the interleaving table's task must match the inputs'",
        ));
    }

    let mut store = SimulatedStore::new(inputs.entries.clone())?;
    let mut presence = PresenceTable::new(table.workspace.clone());
    let mut worktree = inputs.worktree.clone();
    let mut context = inputs.context.clone();
    let mut records = Vec::with_capacity(table.steps.len());

    for (index, step) in table.steps.iter().enumerate() {
        let actor = &step.actor;
        let outcome = match &step.op {
            SimulatedOperation::RequestAccess { surface, action } => {
                let request = AuthorizationRequest {
                    workspace: &table.workspace,
                    actor,
                    action: *action,
                    surface: *surface,
                    target: None,
                    memberships: &inputs.memberships,
                    grants: &inputs.grants,
                    lattice: &inputs.lattice,
                };
                let decision = authorize(&request)?;
                SimulationOutcome::Access {
                    outcome: decision.outcome,
                }
            }
            SimulatedOperation::WriteSharedState {
                key,
                value,
                expected_version,
                visibility,
            } => {
                // The permission gate first: a denied write never
                // touches the store (law 1).
                let (surface, action) = step.op.required_access();
                let request = AuthorizationRequest {
                    workspace: &table.workspace,
                    actor,
                    action,
                    surface,
                    target: None,
                    memberships: &inputs.memberships,
                    grants: &inputs.grants,
                    lattice: &inputs.lattice,
                };
                if let DecisionOutcome::Deny { reason, .. } = authorize(&request)?.outcome {
                    SimulationOutcome::Denied { reason }
                } else {
                    match store.apply_write(actor, key, value, *expected_version, *visibility) {
                        Ok(version) => SimulationOutcome::Written {
                            key: key.clone(),
                            version,
                        },
                        Err(reason) => SimulationOutcome::Denied { reason },
                    }
                }
            }
            SimulatedOperation::ReadProjection => {
                let (surface, action) = step.op.required_access();
                let request = AuthorizationRequest {
                    workspace: &table.workspace,
                    actor,
                    action,
                    surface,
                    target: None,
                    memberships: &inputs.memberships,
                    grants: &inputs.grants,
                    lattice: &inputs.lattice,
                };
                if let DecisionOutcome::Deny { reason, .. } = authorize(&request)?.outcome {
                    SimulationOutcome::Denied { reason }
                } else {
                    SimulationOutcome::Projection {
                        rows: store.projection_for(actor),
                    }
                }
            }
            SimulatedOperation::UpdatePresence { locator } => {
                let (surface, action) = step.op.required_access();
                let request = AuthorizationRequest {
                    workspace: &table.workspace,
                    actor,
                    action,
                    surface,
                    target: None,
                    memberships: &inputs.memberships,
                    grants: &inputs.grants,
                    lattice: &inputs.lattice,
                };
                if let DecisionOutcome::Deny { reason, .. } = authorize(&request)?.outcome {
                    SimulationOutcome::Denied { reason }
                } else {
                    let record = PresenceRecord::new(
                        table.workspace.clone(),
                        actor.clone(),
                        locator.clone(),
                        PresenceFreshness::new(inputs.now, inputs.presence_stale_after_ms)?,
                    )?;
                    presence.update(record)?;
                    SimulationOutcome::PresenceUpdated {
                        member: actor.clone(),
                        locator: locator.clone(),
                    }
                }
            }
            SimulatedOperation::SetSharing {
                shared_filesystem,
                memory,
                artifacts,
            } => {
                let (surface, action) = step.op.required_access();
                let request = AuthorizationRequest {
                    workspace: &table.workspace,
                    actor,
                    action,
                    surface,
                    target: None,
                    memberships: &inputs.memberships,
                    grants: &inputs.grants,
                    lattice: &inputs.lattice,
                };
                if let DecisionOutcome::Deny { reason, .. } = authorize(&request)?.outcome {
                    SimulationOutcome::Denied { reason }
                } else {
                    worktree = WorktreePolicy::new(&table.task, *shared_filesystem);
                    context = ContextVisibility::new(&table.task, *memory, *artifacts);
                    SimulationOutcome::Sharing {
                        shared_filesystem: *shared_filesystem,
                        memory: *memory,
                        artifacts: *artifacts,
                    }
                }
            }
            SimulatedOperation::ReadSharing => {
                let (surface, action) = step.op.required_access();
                let request = AuthorizationRequest {
                    workspace: &table.workspace,
                    actor,
                    action,
                    surface,
                    target: None,
                    memberships: &inputs.memberships,
                    grants: &inputs.grants,
                    lattice: &inputs.lattice,
                };
                if let DecisionOutcome::Deny { reason, .. } = authorize(&request)?.outcome {
                    SimulationOutcome::Denied { reason }
                } else {
                    SimulationOutcome::Sharing {
                        shared_filesystem: worktree.shared_filesystem,
                        memory: context.memory,
                        artifacts: context.artifacts,
                    }
                }
            }
        };
        records.push(SimulationRecord {
            step: index,
            actor: actor.clone(),
            op: step.op.clone(),
            outcome,
        });
    }

    Ok(SimulationResult {
        log: SimulationLog {
            v: CollabVersion,
            workspace: table.workspace.clone(),
            task: table.task.clone(),
            records,
        },
        store,
        presence,
        worktree,
        context,
    })
}

/// Verifies the table's expected outcomes against the run's records —
/// the scripted prediction the deterministic run must reproduce.
///
/// # Errors
///
/// Returns [`CollabError`] naming the first step whose outcome disagrees
/// with its expectation.
pub fn verify_expectations(
    table: &InterleavingTable,
    result: &SimulationResult,
) -> Result<(), CollabError> {
    if table.steps.len() != result.log.records.len() {
        return Err(CollabError::invalid(
            "the log's record count must match the table's step count",
        ));
    }
    for (index, (step, record)) in table.steps.iter().zip(&result.log.records).enumerate() {
        let matches = match (&step.expected, &record.outcome) {
            (
                ExpectedOutcome::Allowed { via },
                SimulationOutcome::Access {
                    outcome: DecisionOutcome::Allow { via: found, .. },
                },
            ) => via == found,
            (
                ExpectedOutcome::Denied { reason },
                SimulationOutcome::Access {
                    outcome: DecisionOutcome::Deny { reason: found, .. },
                },
            ) => reason == found,
            (ExpectedOutcome::Denied { reason }, SimulationOutcome::Denied { reason: found }) => {
                reason == found
            }
            (
                ExpectedOutcome::Written { version },
                SimulationOutcome::Written { version: found, .. },
            ) => version == found,
            (ExpectedOutcome::Projection { keys }, SimulationOutcome::Projection { rows }) => {
                keys == &rows.iter().map(|row| row.key.clone()).collect::<Vec<_>>()
            }
            (ExpectedOutcome::PresenceUpdated, SimulationOutcome::PresenceUpdated { .. }) => true,
            (
                ExpectedOutcome::SharingRead {
                    shared_filesystem,
                    memory,
                    artifacts,
                },
                SimulationOutcome::Sharing {
                    shared_filesystem: found_mode,
                    memory: found_memory,
                    artifacts: found_artifacts,
                },
            ) => {
                shared_filesystem == found_mode
                    && memory == found_memory
                    && artifacts == found_artifacts
            }
            _ => false,
        };
        if !matches {
            return Err(CollabError::invalid(format!(
                "step {index} (actor {:?}) expected {:?} but the run produced {:?}",
                step.actor.id, step.expected, record.outcome
            )));
        }
    }
    Ok(())
}

/// Verifies the F9 gate laws against a completed run — the proof the
/// wave gate and the work order's acceptance criteria lean on:
///
/// 1. every step either landed or carries a named denial (no silent
///    no-ops);
/// 2. permissions held: every mutating step that landed was admitted by
///    the evaluator (re-derived from the inputs), and every denied
///    mutation never touched the store;
/// 3. private records stayed private: every projection contains no
///    member-private entry owned by another actor, and no private
///    write by another actor ever landed;
/// 4. authorized state was never lost: for every key, the final version
///    equals the initial version plus the number of landed writes —
///    nothing vanishes, nothing is invented.
///
/// # Errors
///
/// Returns [`CollabError`] naming the first broken law.
pub fn verify_f9_laws(
    inputs: &SimulationInputs,
    table: &InterleavingTable,
    result: &SimulationResult,
) -> Result<(), CollabError> {
    if table.workspace != inputs.workspace || table.task != inputs.task() {
        return Err(CollabError::invalid(
            "the laws check requires a table and inputs from the same simulation",
        ));
    }
    if table.steps.len() != result.log.records.len() {
        return Err(CollabError::invalid(
            "the laws check requires the log of THIS table's run",
        ));
    }

    // Law 1: every outcome is landed or named-denied. (Structurally
    // guaranteed by SimulationOutcome; re-derive for the record.)
    for record in &result.log.records {
        let named = record.outcome.landed() || record.outcome.denial_reason().is_some();
        if !named {
            return Err(CollabError::invalid(format!(
                "step {} produced neither a landing nor a named denial — the silent no-op the \
                 kernel forbids",
                record.step
            )));
        }
    }

    // Law 2: permissions held under interleaving. Every landed mutating
    // step was admitted; every denied step never mutated.
    for record in &result.log.records {
        let mutating = matches!(
            record.op,
            SimulatedOperation::WriteSharedState { .. }
                | SimulatedOperation::SetSharing { .. }
                | SimulatedOperation::UpdatePresence { .. }
        );
        if !mutating {
            continue;
        }
        let (surface, action) = record.op.required_access();
        let request = AuthorizationRequest {
            workspace: &table.workspace,
            actor: &record.actor,
            action,
            surface,
            target: None,
            memberships: &inputs.memberships,
            grants: &inputs.grants,
            lattice: &inputs.lattice,
        };
        let admitted = authorize(&request)?.outcome.allows();
        match (admitted, record.outcome.landed()) {
            (true, true) | (false, false) => {}
            (true, false) => {
                // Admitted but did not land: must be a named seam
                // denial (version conflict / private record), never a
                // silent no-op.
                if record.outcome.denial_reason().is_none() {
                    return Err(CollabError::invalid(format!(
                        "step {} was admitted but produced no landing and no named reason",
                        record.step
                    )));
                }
            }
            (false, true) => {
                return Err(CollabError::invalid(format!(
                    "step {} mutated the seam without an admission — permissions did not hold",
                    record.step
                )));
            }
        }
    }

    // Law 3: private records stayed private to their actor. Projections
    // never contained another actor's private entries…
    for record in &result.log.records {
        if let SimulationOutcome::Projection { rows } = &record.outcome {
            for row in rows {
                if row.visibility == Visibility::MemberPrivate && row.owner != record.actor {
                    return Err(CollabError::invalid(format!(
                        "step {} projected a member-private entry owned by {:?} into {:?}'s \
                         projection — the §6 law broke",
                        record.step, row.owner.id, record.actor.id
                    )));
                }
            }
        }
    }
    // …and no member-private entry was ever written by another actor.
    for entry in result.store.entries() {
        if entry.visibility == Visibility::MemberPrivate {
            let foreign_write = result.log.records.iter().any(|record| {
                matches!(&record.outcome,
                    SimulationOutcome::Written { key, .. } if key == &entry.key)
                    && record.actor != entry.owner
            });
            if foreign_write {
                return Err(CollabError::invalid(format!(
                    "a member-private entry keyed {:?} was written by another actor — the §6 \
                     law broke",
                    entry.key
                )));
            }
        }
    }

    // Law 4: authorized state was never lost. For every key the final
    // version equals the initial version plus the landed writes; every
    // non-landed write carries a named reason.
    let mut keys: Vec<String> = inputs
        .entries
        .iter()
        .map(|entry| entry.key.clone())
        .collect();
    for record in &result.log.records {
        if let SimulatedOperation::WriteSharedState { key, .. } = &record.op
            && !keys.contains(key)
        {
            keys.push(key.clone());
        }
    }
    for key in &keys {
        let initial = inputs
            .entries
            .iter()
            .find(|entry| &entry.key == key)
            .map_or(0, |entry| entry.version);
        let landed = result
            .log
            .records
            .iter()
            .filter(|record| {
                matches!(&record.outcome,
                    SimulationOutcome::Written { key: written, .. } if written == key)
            })
            .count() as u64;
        let final_version = result.store.get(key).map_or(0, |entry| entry.version);
        if initial + landed != final_version {
            return Err(CollabError::invalid(format!(
                "store key {key:?} ended at version {final_version} but started at {initial} \
                 with {landed} landed writes — authorized state was lost or invented"
            )));
        }
    }
    for record in &result.log.records {
        if matches!(record.op, SimulatedOperation::WriteSharedState { .. })
            && !matches!(record.outcome, SimulationOutcome::Written { .. })
            && record.outcome.denial_reason().is_none()
        {
            return Err(CollabError::invalid(format!(
                "step {} did not land and carries no named denial — a silent no-op",
                record.step
            )));
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

    fn workspace() -> WorkspaceRef {
        ok(WorkspaceRef::parse("ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0"))
    }

    fn task() -> TaskRef {
        ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"))
    }

    /// The smallest honest two-step table: the owner writes, a viewer
    /// reads their projection. The full F9 suite lives in
    /// `tests/simulation.rs`.
    #[test]
    fn the_runner_is_deterministic_and_the_log_replays() {
        let ana = ok(ActorRef::user("ana"));
        let dev = ok(ActorRef::user("dev"));
        let memberships = [
            ok(WorkspaceMembership::new(
                workspace(),
                ana.clone(),
                "Ana",
                crate::membership::Role::Owner,
            )),
            ok(WorkspaceMembership::new(
                workspace(),
                dev.clone(),
                "Dev",
                crate::membership::Role::Viewer,
            )),
        ];
        let inputs = SimulationInputs {
            v: CollabVersion,
            workspace: workspace(),
            memberships: memberships.to_vec(),
            grants: Vec::new(),
            lattice: RoleLattice::default_lattice(),
            entries: Vec::new(),
            worktree: WorktreePolicy::isolated(&task()),
            context: ContextVisibility::default_for(&task()),
            presence_stale_after_ms: 60_000,
            now: ok(Timestamp::parse("2026-09-23T10:00:00Z")),
        };
        let table = ok(InterleavingTable::new(
            workspace(),
            task(),
            vec![
                InterleavedStep {
                    actor: ana.clone(),
                    op: SimulatedOperation::WriteSharedState {
                        key: "brief".to_owned(),
                        value: "the weekly brief".to_owned(),
                        expected_version: 0,
                        visibility: Visibility::WorkspaceShared,
                    },
                    expected: ExpectedOutcome::Written { version: 1 },
                },
                InterleavedStep {
                    actor: dev.clone(),
                    op: SimulatedOperation::ReadProjection,
                    expected: ExpectedOutcome::Projection {
                        keys: vec!["brief".to_owned()],
                    },
                },
            ],
        ));
        let first = ok(run_interleaved(&inputs, &table));
        let second = ok(run_interleaved(&inputs, &table));
        ok(verify_expectations(&table, &first));
        ok(verify_f9_laws(&inputs, &table, &first));
        // Determinism: two runs, byte-identical logs.
        assert_eq!(
            ok(serde_json::to_string_pretty(&first.log)),
            ok(serde_json::to_string_pretty(&second.log))
        );
        // Replay: the log round-trips byte-identically.
        let serialized = ok(serde_json::to_string_pretty(&first.log));
        let replayed: SimulationLog = ok(serde_json::from_str(&serialized));
        assert_eq!(
            ok(serde_json::to_string_pretty(&replayed)),
            serialized,
            "the interleaving log is canonical-JSON replayable"
        );
        // The viewer saw the owner's shared entry.
        assert_eq!(first.store.entries().len(), 1);
        assert_eq!(first.presence.records().len(), 0);
    }
}
