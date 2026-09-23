//! The deterministic lease manager (LEASE-001 §manager): who gets a
//! contended resource, what happens to a conflicting request, and what
//! the human must decide when a conflict escalates.
//!
//! Three pure functions over data (addendum §5 — no live counters, no
//! clock reads; `now` is caller-supplied):
//!
//! - [`decide`]: `decide(request, lease_snapshots, wait_queue_snapshots,
//!   now) → LeaseDecision`. The conflict matrix is
//!   [`AccessMode::conflicts_with`](crate::AccessMode::conflicts_with);
//!   the holder-side policy of the **primary conflicting holder**
//!   decides the outcome (reject / queue / escalate). The primary
//!   holder is the earliest-granted conflicting lease, tie-broken by
//!   canonical lease id — deterministic. A renewal (a request that
//!   names the lease it extends) extends that lease at version + 1,
//!   and its conflict check excludes the lease being renewed.
//! - [`sweep`]: the expiry sweep — which leases stopped being held at
//!   `now` (expired by deadline, or explicitly released — the honest
//!   distinction), and which waiters advance, **in request order**,
//!   with the advanced positions named. A waiter whose requested
//!   window has closed is named as closed — never silently dropped.
//! - [`resolve_escalation`]: applies the human's EXPLICIT, ATTRIBUTED
//!   decision to a named conflict (grant to waiter / keep holder).
//!   Escalation is never auto-resolved (addendum §2); a decision by a
//!   non-human actor is refused.
//!
//! The queue law (addendum §2): FIFO and deterministic — waiters
//! advance in request order (`enqueued_at`, tie-broken by canonical
//! lease id); positions are honest; expiry is honest (a lease not held
//! at the moment is not held); renewal is explicit only.

use serde::{Deserialize, Serialize};

use crate::LeaseError;
use crate::lease::{LeaseRecord, WaitSnapshot};
use crate::mode::{ConflictPolicy, EscalationChoice, EscalationOutcome};
use crate::refs::ActorRef;
use crate::request::{LeaseDecision, LeaseRequest, QueuedAhead, RenewalAttribution};
use crate::time::Timestamp;
use crate::{LeaseVersion, MAX_SWEEP_GRANTS, MAX_WAIT_SNAPSHOTS, ensure_lease_snapshots};

/// The named conflict an escalation carries (the "needs the human"
/// record): the holder in full, the contending request in full, and
/// nothing else — the human decides between exactly these two named
/// sides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedConflict {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The contending request (the waiting side, in full).
    pub request: LeaseRequest,
    /// The holder's lease (the holding side, in full).
    pub holder: LeaseRecord,
}

/// One waiter's outcome from an expiry sweep, in request order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum WaiterOutcome {
    /// The waiter advanced to a grant: the granted lease record as
    /// data (it now holds; later waiters in this sweep were checked
    /// against it).
    Granted {
        /// The granted lease record.
        lease: LeaseRecord,
    },
    /// The waiter is still waiting, with its honest new position and
    /// the named waiters ahead.
    StillWaiting {
        /// The queued request.
        request: LeaseRequest,
        /// When it was enqueued.
        enqueued_at: Timestamp,
        /// The 1-based position after the sweep.
        position: u32,
        /// The named waiters ahead, in request order.
        ahead: Vec<QueuedAhead>,
    },
    /// The waiter's requested window closed before the resource freed
    /// up — the request can no longer be granted. Named, never
    /// silently dropped (the conflict-honesty law).
    WindowClosed {
        /// The queued request whose window closed.
        request: LeaseRequest,
        /// When it was enqueued.
        enqueued_at: Timestamp,
    },
}

/// One lease that stopped being held at the sweep moment: expired by
/// its own deadline (the honest expiry) — distinct from an explicit
/// release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpiredLease {
    /// The lease that expired, named in full.
    pub lease: LeaseRecord,
    /// The moment it stopped being held (its own deadline — not the
    /// sweep moment).
    pub expired_at: Timestamp,
}

/// One lease that was explicitly released before its deadline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedLease {
    /// The released lease, named in full (with its `released_at`).
    pub lease: LeaseRecord,
}

/// The result of one expiry sweep over one resource's wait queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSweep {
    /// The resource whose queue was swept.
    pub resource_id: crate::refs::ResourceRef,
    /// The waiters' outcomes, in request order.
    pub outcomes: Vec<WaiterOutcome>,
}

/// The result of one expiry sweep: which leases stopped being held
/// (expired vs explicitly released — the honest distinction), and how
/// every wait queue advanced (in request order, positions named).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseSweep {
    /// Contract schema version (`"v": 1`).
    pub v: LeaseVersion,
    /// The caller-supplied sweep moment.
    pub swept_at: Timestamp,
    /// Leases that expired by their own deadline at `swept_at`.
    pub expired: Vec<ExpiredLease>,
    /// Leases that were explicitly released (distinct from expiry).
    pub released: Vec<ReleasedLease>,
    /// Per-resource waiter outcomes, in request order.
    pub resources: Vec<ResourceSweep>,
}

/// Decides one request against the lease and wait-queue snapshots.
///
/// The conflict check considers only leases **held at `now`** (expiry
/// is enforced on read — an expired or released lease never blocks).
/// When no conflicting holder exists the request is granted (a
/// renewal extends the named lease at version + 1). When conflicting
/// holders exist, the primary holder's policy decides:
///
/// - [`ConflictPolicy::Reject`] → [`LeaseDecision::Rejected`] with the
///   named holder;
/// - [`ConflictPolicy::Queue`] → [`LeaseDecision::Queued`] with the
///   honest position (the resource's existing waiters + 1, FIFO), the
///   named waiters ahead, and the projected grant moment given the
///   conflicting holders' deadlines;
/// - [`ConflictPolicy::Escalate`] → [`LeaseDecision::Escalated`] with
///   the named conflict pair (never auto-resolved).
///
/// # Errors
///
/// Returns [`LeaseError`] when the snapshots are inconsistent
/// (duplicates, invalid records, duplicate wait requests), when the
/// request window has already closed at `now`, or when a renewal does
/// not name a held lease owned by the requester in the same mode.
pub fn decide(
    request: &LeaseRequest,
    leases: &[LeaseRecord],
    waiters: &[WaitSnapshot],
    now: Timestamp,
) -> Result<LeaseDecision, LeaseError> {
    request.validate()?;
    ensure_lease_snapshots("lease snapshots", leases)?;
    ensure_wait_snapshots(waiters)?;
    if request.requested_until.is_before(&now) || request.requested_until == now {
        return Err(LeaseError::invalid(
            "request window has already closed at the decision moment",
        ));
    }

    let mut extended: Option<&LeaseRecord> = None;
    if let Some(renewed_id) = request.renewal_of.as_ref() {
        // The explicit renewal: the named lease must be held at now,
        // owned by the requester, in the same mode, on the same
        // resource — anything else is a named inconsistency.
        let target = leases
            .iter()
            .find(|lease| &lease.id == renewed_id)
            .ok_or_else(|| {
                LeaseError::invalid(format!(
                    "renewal names lease {renewed_id}, which is not in the snapshots"
                ))
            })?;
        if !target.is_held_at(&now) {
            return Err(LeaseError::invalid(format!(
                "renewal names lease {renewed_id}, which is not held at the decision moment"
            )));
        }
        if target.owner != request.requester {
            return Err(LeaseError::invalid(format!(
                "renewal names lease {renewed_id}, held by another actor"
            )));
        }
        if target.mode != request.mode {
            return Err(LeaseError::invalid(format!(
                "renewal names lease {renewed_id}, held in a different access mode"
            )));
        }
        if target.resource_id != request.resource_id {
            return Err(LeaseError::invalid(format!(
                "renewal names lease {renewed_id}, scoped to a different resource"
            )));
        }
        extended = Some(target);
    } else {
        // A fresh grant must not reuse a lease id already in the
        // snapshots (entity uniqueness — the world store owns minting).
        if leases.iter().any(|lease| lease.id == request.lease_id) {
            return Err(LeaseError::invalid(format!(
                "fresh request reuses the lease id {} already present in the snapshots",
                request.lease_id
            )));
        }
    }

    // Conflicting holders: held at now, same resource, conflicting
    // mode — excluding the lease a renewal extends (extending your own
    // lease never conflicts with itself).
    let conflicting: Vec<&LeaseRecord> = leases
        .iter()
        .filter(|lease| lease.is_held_at(&now))
        .filter(|lease| lease.resource_id == request.resource_id)
        .filter(|lease| lease.mode.conflicts_with(request.mode))
        .filter(|lease| extended.is_none_or(|target| lease.id != target.id))
        .collect();

    if conflicting.is_empty() {
        return Ok(grant(request, extended, now));
    }

    // The primary holder: earliest granted, tie-broken by canonical
    // lease id — deterministic.
    let primary = match conflicting
        .iter()
        .min_by(|a, b| (a.granted_at, a.id.as_str()).cmp(&(b.granted_at, b.id.as_str())))
    {
        Some(primary) => *primary,
        None => unreachable!("conflicting is non-empty"),
    };

    match primary.conflict_policy {
        ConflictPolicy::Reject => Ok(LeaseDecision::Rejected {
            v: LeaseVersion,
            request: request.clone(),
            holder: primary.clone(),
            policy: primary.conflict_policy,
        }),
        ConflictPolicy::Queue => {
            let resource_waiters: Vec<&WaitSnapshot> = waiters
                .iter()
                .filter(|waiter| waiter.request.resource_id == request.resource_id)
                .collect();
            let ahead = resource_waiters
                .iter()
                .map(|waiter| QueuedAhead {
                    requester: waiter.request.requester.clone(),
                    mode: waiter.request.mode,
                    enqueued_at: waiter.enqueued_at,
                })
                .collect();
            // The projected grant moment: every conflicting holder's
            // deadline must pass before the resource frees for this
            // request — the max of their deadlines (all held at now,
            // so the projection lies strictly in the future).
            let projected = match conflicting.iter().map(|lease| lease.expires_at).max() {
                Some(projected) => projected,
                None => unreachable!("conflicting is non-empty"),
            };
            Ok(LeaseDecision::Queued {
                v: LeaseVersion,
                request: request.clone(),
                position: resource_waiters.len() as u32 + 1,
                ahead,
                projected_grant_at: projected,
            })
        }
        ConflictPolicy::Escalate => Ok(LeaseDecision::Escalated {
            v: LeaseVersion,
            request: request.clone(),
            holder: primary.clone(),
        }),
    }
}

/// Builds the granted record for a request (fresh or renewal).
fn grant(request: &LeaseRequest, extended: Option<&LeaseRecord>, now: Timestamp) -> LeaseDecision {
    if let Some(target) = extended {
        // The explicit renewal: the same lease id at version + 1,
        // granted at the renewal moment, bounded by the new deadline.
        let renewed = match target.renewed(now, request.requested_until, request.policy) {
            Ok(renewed) => renewed,
            Err(error) => unreachable!("decide validated the renewal window: {error}"),
        };
        return LeaseDecision::Granted {
            v: LeaseVersion,
            request: request.clone(),
            lease: renewed,
            renewal: Some(RenewalAttribution {
                prior_version: target.version,
                prior_expires_at: target.expires_at,
            }),
        };
    }
    // The fresh grant: version 1, granted no earlier than the window's
    // start and no earlier than now.
    let granted_at = if request.requested_from.is_after(&now) {
        request.requested_from
    } else {
        now
    };
    let lease = match LeaseRecord::new(
        request.lease_id.clone(),
        request.resource_id.clone(),
        request.mode,
        request.requester.clone(),
        granted_at,
        request.requested_until,
        request.policy,
    ) {
        Ok(lease) => lease,
        Err(error) => unreachable!("decide validated the request window: {error}"),
    };
    LeaseDecision::Granted {
        v: LeaseVersion,
        request: request.clone(),
        lease,
        renewal: None,
    }
}

/// Runs the expiry sweep over the snapshots at `now`: which leases
/// stopped being held (expired by deadline vs explicitly released —
/// the honest distinction), and which waiters advance, **in request
/// order**.
///
/// The queue law: for each resource, waiters are processed in request
/// order (`enqueued_at`, tie-broken by canonical lease id). A waiter
/// whose window closed is named as closed — never silently dropped. A
/// waiter whose mode no longer conflicts with the remaining holders
/// (including waiters granted earlier in this sweep) is granted and
/// its new lease holds up later conflicting waiters. The still-waiting
/// carry honest new positions and the named waiters ahead.
///
/// # Errors
///
/// Returns [`LeaseError`] when the snapshots are inconsistent
/// (duplicates, invalid records, duplicate wait requests) or the sweep
/// would produce more grants than the bounded sweep allows.
pub fn sweep(
    leases: &[LeaseRecord],
    waiters: &[WaitSnapshot],
    now: Timestamp,
) -> Result<LeaseSweep, LeaseError> {
    ensure_lease_snapshots("lease snapshots", leases)?;
    ensure_wait_snapshots(waiters)?;

    let mut expired = Vec::new();
    let mut released = Vec::new();
    for lease in leases {
        if lease.is_held_at(&now) {
            continue;
        }
        if lease.released_at.is_some() {
            released.push(ReleasedLease {
                lease: lease.clone(),
            });
        } else {
            expired.push(ExpiredLease {
                lease: lease.clone(),
                expired_at: lease.expires_at,
            });
        }
    }

    // Group waiters by resource, in request order (FIFO): enqueued_at,
    // tie-broken by canonical lease id.
    let mut ordered: Vec<&WaitSnapshot> = waiters.iter().collect();
    ordered.sort_by(|a, b| {
        (a.enqueued_at, a.request.lease_id.as_str())
            .cmp(&(b.enqueued_at, b.request.lease_id.as_str()))
    });
    let mut resource_ids: Vec<crate::refs::ResourceRef> = Vec::new();
    for waiter in &ordered {
        if !resource_ids.contains(&waiter.request.resource_id) {
            resource_ids.push(waiter.request.resource_id.clone());
        }
    }

    let mut resources = Vec::new();
    let mut grants_total = 0usize;
    for resource_id in resource_ids {
        // The holders that remain for this resource at now — including
        // leases granted earlier in this sweep (they hold now).
        let mut holders: Vec<LeaseRecord> = leases
            .iter()
            .filter(|lease| lease.is_held_at(&now) && lease.resource_id == resource_id)
            .cloned()
            .collect();
        let mut outcomes = Vec::new();
        let mut still_waiting: Vec<(&LeaseRequest, Timestamp)> = Vec::new();
        for waiter in ordered
            .iter()
            .filter(|waiter| waiter.request.resource_id == resource_id)
        {
            let request = &waiter.request;
            let window_closed =
                request.requested_until.is_before(&now) || request.requested_until == now;
            let conflicts = holders
                .iter()
                .any(|lease| lease.mode.conflicts_with(request.mode));
            if window_closed {
                // Honest drop: the ask outlived its own window. Named,
                // never silent.
                outcomes.push(WaiterOutcome::WindowClosed {
                    request: request.clone(),
                    enqueued_at: waiter.enqueued_at,
                });
                continue;
            }
            if conflicts {
                let ahead = still_waiting
                    .iter()
                    .map(|(ahead, enqueued_at)| QueuedAhead {
                        requester: ahead.requester.clone(),
                        mode: ahead.mode,
                        enqueued_at: *enqueued_at,
                    })
                    .collect();
                outcomes.push(WaiterOutcome::StillWaiting {
                    request: request.clone(),
                    enqueued_at: waiter.enqueued_at,
                    position: still_waiting.len() as u32 + 1,
                    ahead,
                });
                still_waiting.push((request, waiter.enqueued_at));
                continue;
            }
            // The waiter advances to a grant: the record as data, and
            // it now holds for the rest of this sweep.
            grants_total += 1;
            if grants_total > MAX_SWEEP_GRANTS {
                return Err(LeaseError::invalid(format!(
                    "sweep would exceed {MAX_SWEEP_GRANTS} grants"
                )));
            }
            let granted_at = if request.requested_from.is_after(&now) {
                request.requested_from
            } else {
                now
            };
            let lease = LeaseRecord::new(
                request.lease_id.clone(),
                request.resource_id.clone(),
                request.mode,
                request.requester.clone(),
                granted_at,
                request.requested_until,
                request.policy,
            )?;
            holders.push(lease.clone());
            outcomes.push(WaiterOutcome::Granted { lease });
        }
        resources.push(ResourceSweep {
            resource_id,
            outcomes,
        });
    }

    Ok(LeaseSweep {
        v: LeaseVersion,
        swept_at: now,
        expired,
        released,
        resources,
    })
}

/// Applies the human's EXPLICIT, ATTRIBUTED decision to a named
/// conflict (addendum §2: escalation is the human's call — never an
/// auto-resolution).
///
/// - [`EscalationChoice::GrantToWaiter`]: the holder's lease is
///   released (version + 1, `released_at = decided_at`) and the
///   waiter's request is granted — both records named, the decision
///   attributed to the deciding human.
/// - [`EscalationChoice::KeepHolder`]: the holder keeps the lease
///   unchanged and the named waiter is told no — the consequence is
///   structural (the outcome variant), the surfaces state it in user
///   words.
///
/// # Errors
///
/// Returns [`LeaseError`] when the deciding actor is not a human
/// principal (an escalation is decided by a human, never auto-resolved
/// by an agent or system actor), when the conflict's records fail
/// validation, or when the waiter's window has already closed.
pub fn resolve_escalation(
    conflict: &NamedConflict,
    choice: EscalationChoice,
    decided_by: ActorRef,
    decided_at: Timestamp,
) -> Result<EscalationOutcome, LeaseError> {
    decided_by.validate()?;
    conflict.request.validate()?;
    conflict.holder.validate()?;
    if decided_by.kind != crate::refs::ActorKind::User {
        return Err(LeaseError::invalid(
            "an escalated conflict is decided by a human actor — auto-resolution is forbidden",
        ));
    }
    if !conflict.holder.is_held_at(&decided_at) {
        return Err(LeaseError::invalid(
            "the named holder's lease is not held at the decision moment",
        ));
    }
    match choice {
        EscalationChoice::GrantToWaiter => {
            let request = &conflict.request;
            if request.requested_until.is_before(&decided_at)
                || request.requested_until == decided_at
            {
                return Err(LeaseError::invalid(
                    "the waiter's window has already closed at the decision moment",
                ));
            }
            let released_holder = conflict.holder.released(decided_at)?;
            let granted_at = if request.requested_from.is_after(&decided_at) {
                request.requested_from
            } else {
                decided_at
            };
            let granted = LeaseRecord::new(
                request.lease_id.clone(),
                request.resource_id.clone(),
                request.mode,
                request.requester.clone(),
                granted_at,
                request.requested_until,
                request.policy,
            )?;
            Ok(EscalationOutcome::WaiterGranted {
                v: LeaseVersion,
                released_holder,
                granted,
                decided_by,
                decided_at,
            })
        }
        EscalationChoice::KeepHolder => Ok(EscalationOutcome::HolderKept {
            v: LeaseVersion,
            holder: conflict.holder.clone(),
            waiter: conflict.request.requester.clone(),
            decided_by,
            decided_at,
        }),
    }
}

/// Validates the wait-queue snapshots: bounded, valid, and free of
/// duplicate lease ids (one queued request per lease id).
fn ensure_wait_snapshots(waiters: &[WaitSnapshot]) -> Result<(), LeaseError> {
    if waiters.len() > MAX_WAIT_SNAPSHOTS {
        return Err(LeaseError::invalid(format!(
            "wait queue exceeds {MAX_WAIT_SNAPSHOTS} entries"
        )));
    }
    for waiter in waiters {
        waiter.validate()?;
    }
    let mut seen = std::collections::BTreeSet::new();
    for waiter in waiters {
        if !seen.insert(waiter.request.lease_id.as_str()) {
            return Err(LeaseError::invalid(format!(
                "wait queue contains the lease id {} more than once",
                waiter.request.lease_id
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{
        ana_write_lease, conflict_space, dev_read_request, dev_write_request, fake_actors,
        fake_lease_ids, fake_other_resource, fake_resource, now,
    };
    use crate::mode::AccessMode;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn a_free_resource_grants_immediately() {
        let request = ok(dev_write_request());
        let decision = ok(decide(&request, &[], &[], now()));
        match &decision {
            LeaseDecision::Granted { lease, renewal, .. } => {
                assert_eq!(lease.id, request.lease_id);
                assert_eq!(lease.version, 1);
                assert_eq!(lease.mode, AccessMode::Write);
                assert_eq!(lease.owner, request.requester);
                assert_eq!(lease.granted_at, now());
                assert_eq!(renewal, &None);
            }
            other => panic!("expected a grant, got {other:?}"),
        }
    }

    #[test]
    fn the_full_conflict_matrix_and_all_three_policies_hold() {
        // Every (holder mode × requester mode × policy) combination in
        // the fake space behaves per the frozen law: compatible modes
        // grant; conflicting modes follow the holder's policy.
        for case in conflict_space() {
            let decision = ok(decide(
                &case.request,
                std::slice::from_ref(&case.holder),
                &[],
                case.now,
            ));
            if !case.holder.mode.conflicts_with(case.request.mode) {
                assert!(
                    matches!(decision, LeaseDecision::Granted { .. }),
                    "compatible modes must grant: {:?} × {:?}",
                    case.holder.mode,
                    case.request.mode
                );
                continue;
            }
            match (case.holder.conflict_policy, &decision) {
                (ConflictPolicy::Reject, LeaseDecision::Rejected { holder, policy, .. }) => {
                    assert_eq!(holder.id, case.holder.id);
                    assert_eq!(*policy, ConflictPolicy::Reject);
                }
                (ConflictPolicy::Escalate, LeaseDecision::Escalated { holder, .. }) => {
                    assert_eq!(holder.id, case.holder.id);
                    assert_eq!(holder.owner, case.holder.owner);
                    assert_eq!(decision.request().requester, case.request.requester);
                }
                (
                    ConflictPolicy::Queue,
                    LeaseDecision::Queued {
                        position,
                        ahead,
                        projected_grant_at,
                        ..
                    },
                ) => {
                    assert_eq!(*position, 1, "an empty queue puts the newcomer first");
                    assert!(ahead.is_empty());
                    assert_eq!(*projected_grant_at, case.holder.expires_at);
                }
                (policy, other) => {
                    panic!("policy {policy:?} produced {other:?}");
                }
            }
        }
    }

    #[test]
    fn an_expired_lease_never_blocks() {
        // Expiry is enforced on read: the holder's deadline has passed
        // at now, so the conflicting request grants.
        let expired_holder = ok(LeaseRecord::new(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana.clone(),
            ok(Timestamp::parse("2026-09-23T14:00:00Z")),
            ok(Timestamp::parse("2026-09-23T15:00:00Z")),
            ConflictPolicy::Reject,
        ));
        let request = ok(dev_write_request());
        let decision = ok(decide(&request, &[expired_holder], &[], now()));
        assert!(matches!(decision, LeaseDecision::Granted { .. }));
    }

    #[test]
    fn a_released_lease_never_blocks() {
        let held = ok(LeaseRecord::new(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana.clone(),
            ok(Timestamp::parse("2026-09-23T14:00:00Z")),
            ok(Timestamp::parse("2026-09-23T16:00:00Z")),
            ConflictPolicy::Reject,
        ));
        let released = ok(held.released(ok(Timestamp::parse("2026-09-23T14:30:00Z"))));
        let request = ok(dev_write_request());
        let decision = ok(decide(&request, &[released], &[], now()));
        assert!(matches!(decision, LeaseDecision::Granted { .. }));
    }

    #[test]
    fn queue_positions_are_fifo_and_honest() {
        let holder = ana_write_lease(ConflictPolicy::Queue);
        let first = ok(dev_write_request());
        let second = ok(LeaseRequest::new(
            fake_lease_ids()[2].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana,
            now(),
            ok(Timestamp::parse("2026-09-23T17:00:00Z")),
            ConflictPolicy::Queue,
        ));
        let first_wait = ok(WaitSnapshot::new(first.clone(), now()));
        // A queue-policy request with one waiter already present.
        let decision = ok(decide(
            &second,
            std::slice::from_ref(&holder),
            std::slice::from_ref(&first_wait),
            now(),
        ));
        match &decision {
            LeaseDecision::Queued {
                position,
                ahead,
                projected_grant_at,
                ..
            } => {
                assert_eq!(*position, 2, "the newcomer joins the back of the queue");
                assert_eq!(ahead.len(), 1);
                assert_eq!(ahead[0].requester, first.requester);
                assert_eq!(ahead[0].enqueued_at, first_wait.enqueued_at);
                assert_eq!(*projected_grant_at, holder.expires_at);
            }
            other => panic!("expected a queue decision, got {other:?}"),
        }
        // Waiters on ANOTHER resource never affect this queue.
        let other_resource_wait = ok(WaitSnapshot::new(
            ok(LeaseRequest::new(
                fake_lease_ids()[3].clone(),
                fake_other_resource(),
                AccessMode::Write,
                fake_actors().ana,
                now(),
                ok(Timestamp::parse("2026-09-23T17:00:00Z")),
                ConflictPolicy::Queue,
            )),
            now(),
        ));
        let decision = ok(decide(
            &second,
            &[holder],
            &[first_wait, other_resource_wait],
            now(),
        ));
        match &decision {
            LeaseDecision::Queued {
                position, ahead, ..
            } => {
                assert_eq!(*position, 2, "only this resource's waiters count");
                assert_eq!(ahead.len(), 1);
            }
            other => panic!("expected a queue decision, got {other:?}"),
        }
    }

    #[test]
    fn shared_requesters_grant_together_and_exclusive_blocks_everyone() {
        // exclusive × shared: the write holder blocks a read requester
        // under every policy; shared × shared: two readers coexist.
        let holder = ana_write_lease(ConflictPolicy::Reject);
        let read = ok(dev_read_request());
        assert!(matches!(
            ok(decide(&read, &[holder], &[], now())),
            LeaseDecision::Rejected { .. }
        ));

        let reader_holder = ok(LeaseRecord::new(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Read,
            fake_actors().ana.clone(),
            ok(Timestamp::parse("2026-09-23T14:30:00Z")),
            ok(Timestamp::parse("2026-09-23T16:00:00Z")),
            ConflictPolicy::Reject,
        ));
        let other_reader = ok(dev_read_request());
        assert!(matches!(
            ok(decide(&other_reader, &[reader_holder], &[], now())),
            LeaseDecision::Granted { .. }
        ));
    }

    #[test]
    fn renewal_success_conflict_and_ownership() {
        let holder = ana_write_lease(ConflictPolicy::Reject);
        let now = now();

        // A successful renewal: no other conflicting holder — the
        // lease extends at version + 1 with the new deadline.
        let renewal = ok(LeaseRequest::renewal(
            holder.id.clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana.clone(),
            now,
            ok(Timestamp::parse("2026-09-23T17:30:00Z")),
            ConflictPolicy::Queue,
        ));
        match ok(decide(&renewal, std::slice::from_ref(&holder), &[], now)) {
            LeaseDecision::Granted {
                lease,
                renewal: attribution,
                ..
            } => {
                assert_eq!(lease.id, holder.id);
                assert_eq!(lease.version, holder.version + 1);
                assert_eq!(
                    lease.expires_at,
                    ok(Timestamp::parse("2026-09-23T17:30:00Z"))
                );
                let attribution = match attribution {
                    Some(attribution) => attribution,
                    None => panic!("a granted renewal carries its attribution"),
                };
                assert_eq!(attribution.prior_version, holder.version);
                assert_eq!(attribution.prior_expires_at, holder.expires_at);
            }
            other => panic!("expected a renewal grant, got {other:?}"),
        }

        // A conflicting renewal: another exclusive holder blocks it and
        // the holder's policy decides — a renewal never silently
        // overrides a conflict.
        let other_holder = ok(LeaseRecord::new(
            fake_lease_ids()[1].clone(),
            fake_resource(),
            AccessMode::Commit,
            fake_actors().dev,
            ok(Timestamp::parse("2026-09-23T14:00:00Z")),
            ok(Timestamp::parse("2026-09-23T16:30:00Z")),
            ConflictPolicy::Escalate,
        ));
        match ok(decide(
            &renewal,
            &[holder.clone(), other_holder.clone()],
            &[],
            now,
        )) {
            // The primary conflicting holder is the OTHER lease
            // (the renewal excludes its own target).
            LeaseDecision::Escalated {
                holder: named,
                request,
                ..
            } => {
                assert_eq!(named.id, other_holder.id);
                assert_eq!(request.renewal_of, Some(holder.id.clone()));
            }
            other => panic!("expected an escalation, got {other:?}"),
        }

        // A renewal by the wrong actor is refused — named.
        let stolen = ok(LeaseRequest::renewal(
            holder.id.clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().dev,
            now,
            ok(Timestamp::parse("2026-09-23T17:30:00Z")),
            ConflictPolicy::Queue,
        ));
        let error = match decide(&stolen, &[holder], &[], now) {
            Err(error) => error,
            Ok(decision) => panic!("a renewal by another actor must be refused, got {decision:?}"),
        };
        assert!(
            error.to_string().contains("held by another actor"),
            "the error names the mismatch: {error}"
        );

        // A closed-window request never earns a decision.
        let late = ok(LeaseRequest::new(
            fake_lease_ids()[4].clone(),
            fake_resource(),
            AccessMode::Read,
            fake_actors().dev,
            ok(Timestamp::parse("2026-09-23T14:00:00Z")),
            ok(Timestamp::parse("2026-09-23T14:30:00Z")),
            ConflictPolicy::Queue,
        ));
        assert!(decide(&late, &[], &[], now).is_err());
    }

    #[test]
    fn the_expiry_sweep_advances_waiters_in_request_order() {
        // Ana holds until 15:40 (queue policy). Dev (write) and Ana's
        // second task (write) wait. At 15:41 the holder expired: Dev
        // (enqueued first) is granted; the second write waiter now
        // conflicts with Dev's fresh lease and waits at position 1 —
        // the FIFO law, with named advancement.
        let holder = ana_write_lease(ConflictPolicy::Queue);
        let first_request = ok(dev_write_request());
        let second_request = ok(LeaseRequest::new(
            fake_lease_ids()[2].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana.clone(),
            now(),
            ok(Timestamp::parse("2026-09-23T17:00:00Z")),
            ConflictPolicy::Queue,
        ));
        let waiters = [
            ok(WaitSnapshot::new(first_request, now())),
            ok(WaitSnapshot::new(
                second_request,
                ok(Timestamp::parse("2026-09-23T15:10:00Z")),
            )),
        ];
        let after_expiry = ok(Timestamp::parse("2026-09-23T15:41:00Z"));
        let result = ok(sweep(&[holder], &waiters, after_expiry));
        assert_eq!(
            result.expired.len(),
            1,
            "the holder expired by its deadline"
        );
        assert_eq!(result.expired[0].lease.id, fake_lease_ids()[0]);
        assert!(result.released.is_empty());
        assert_eq!(result.resources.len(), 1);
        let outcomes = &result.resources[0].outcomes;
        assert_eq!(outcomes.len(), 2);
        match &outcomes[0] {
            WaiterOutcome::Granted { lease } => {
                assert_eq!(lease.owner, fake_actors().dev);
                assert_eq!(lease.granted_at, after_expiry);
            }
            other => panic!("the first waiter advances, got {other:?}"),
        }
        match &outcomes[1] {
            WaiterOutcome::StillWaiting {
                position, ahead, ..
            } => {
                assert_eq!(*position, 1);
                assert_eq!(ahead.len(), 0, "the granted waiter is no longer ahead");
            }
            other => panic!("the second write waiter waits behind the new grant, got {other:?}"),
        }
    }

    #[test]
    fn the_sweep_names_closed_windows_and_distinguishes_release() {
        // A waiter whose window closed before the resource freed is
        // named as closed — never silently dropped; a shared waiter
        // behind a still-held exclusive holder keeps waiting.
        let sweep_moment = ok(Timestamp::parse("2026-09-23T15:35:00Z"));
        let holder = ok(LeaseRecord::new(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana.clone(),
            ok(Timestamp::parse("2026-09-23T15:00:00Z")),
            ok(Timestamp::parse("2026-09-23T16:40:00Z")),
            ConflictPolicy::Queue,
        ));
        let closed_request = ok(LeaseRequest::new(
            fake_lease_ids()[4].clone(),
            fake_resource(),
            AccessMode::Read,
            fake_actors().dev,
            ok(Timestamp::parse("2026-09-23T15:05:00Z")),
            ok(Timestamp::parse("2026-09-23T15:30:00Z")),
            ConflictPolicy::Queue,
        ));
        let alive_request = ok(dev_read_request());
        let waiters = [
            ok(WaitSnapshot::new(
                closed_request,
                ok(Timestamp::parse("2026-09-23T15:05:00Z")),
            )),
            ok(WaitSnapshot::new(
                alive_request,
                ok(Timestamp::parse("2026-09-23T15:06:00Z")),
            )),
        ];
        let result = ok(sweep(&[holder], &waiters, sweep_moment));
        let outcomes = &result.resources[0].outcomes;
        assert!(matches!(outcomes[0], WaiterOutcome::WindowClosed { .. }));
        assert!(matches!(outcomes[1], WaiterOutcome::StillWaiting { .. }));

        // An explicitly released lease is reported as released, not
        // expired — the honest distinction.
        let held = ok(LeaseRecord::new(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana.clone(),
            ok(Timestamp::parse("2026-09-23T15:00:00Z")),
            ok(Timestamp::parse("2026-09-23T16:40:00Z")),
            ConflictPolicy::Queue,
        ));
        let released = ok(held.released(ok(Timestamp::parse("2026-09-23T15:20:00Z"))));
        let result = ok(sweep(&[released], &[], sweep_moment));
        assert!(result.expired.is_empty());
        assert_eq!(result.released.len(), 1);
    }

    #[test]
    fn resolve_escalation_requires_a_human_and_names_both_sides() {
        let holder = ana_write_lease(ConflictPolicy::Escalate);
        let request = ok(dev_write_request());
        let conflict = NamedConflict {
            v: LeaseVersion,
            request: request.clone(),
            holder: holder.clone(),
        };
        let human = fake_actors().ana;
        let agent = fake_actors().dev_agent;

        // An agent deciding is refused — never auto-resolved.
        assert!(resolve_escalation(&conflict, EscalationChoice::KeepHolder, agent, now()).is_err());

        // Grant to waiter: the holder is released (version + 1) and
        // the waiter holds.
        match ok(resolve_escalation(
            &conflict,
            EscalationChoice::GrantToWaiter,
            human.clone(),
            now(),
        )) {
            EscalationOutcome::WaiterGranted {
                released_holder,
                granted,
                decided_by,
                decided_at,
                ..
            } => {
                assert_eq!(released_holder.id, holder.id);
                assert_eq!(released_holder.version, holder.version + 1);
                assert_eq!(released_holder.released_at, Some(now()));
                assert_eq!(granted.owner, request.requester);
                assert_eq!(granted.id, request.lease_id);
                assert_eq!(decided_by, human);
                assert_eq!(decided_at, now());
            }
            other => panic!("expected a waiter grant, got {other:?}"),
        }

        // Keep holder: the waiter is named and told no, the holder
        // unchanged.
        match ok(resolve_escalation(
            &conflict,
            EscalationChoice::KeepHolder,
            human,
            now(),
        )) {
            EscalationOutcome::HolderKept {
                holder: kept,
                waiter,
                ..
            } => {
                assert_eq!(kept.id, holder.id);
                assert_eq!(kept.version, holder.version);
                assert_eq!(waiter, request.requester);
            }
            other => panic!("expected a holder keep, got {other:?}"),
        }
    }

    #[test]
    fn decisions_are_deterministic_and_byte_identical() {
        // Same inputs → byte-identical decisions; and the primary
        // holder pick is order-free: swapping the snapshot order (the
        // two conflicting holders granted at different moments) does
        // not change the decision bytes.
        let request = ok(dev_write_request());
        let waiters = [ok(WaitSnapshot::new(request.clone(), now()))];
        let early = ok(LeaseRecord::new(
            fake_lease_ids()[0].clone(),
            fake_resource(),
            AccessMode::Write,
            fake_actors().ana.clone(),
            ok(Timestamp::parse("2026-09-23T14:00:00Z")),
            ok(Timestamp::parse("2026-09-23T16:00:00Z")),
            ConflictPolicy::Queue,
        ));
        let late = ok(LeaseRecord::new(
            fake_lease_ids()[5].clone(),
            fake_resource(),
            AccessMode::Commit,
            fake_actors().dev_agent,
            ok(Timestamp::parse("2026-09-23T14:30:00Z")),
            ok(Timestamp::parse("2026-09-23T16:30:00Z")),
            ConflictPolicy::Escalate,
        ));
        let first = ok(serde_json::to_string(&ok(decide(
            &request,
            &[early.clone(), late.clone()],
            &waiters,
            now(),
        ))));
        let second = ok(serde_json::to_string(&ok(decide(
            &request,
            &[early.clone(), late.clone()],
            &waiters,
            now(),
        ))));
        assert_eq!(first, second, "identical inputs produce identical bytes");
        let reordered = ok(serde_json::to_string(&ok(decide(
            &request,
            &[late, early],
            &waiters,
            now(),
        ))));
        assert_eq!(
            first, reordered,
            "snapshot order never changes the decision (the primary pick is order-free)"
        );
    }
}
