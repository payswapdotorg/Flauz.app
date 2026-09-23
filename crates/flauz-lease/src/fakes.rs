//! Public in-memory fakes: the deterministic conformance surface for
//! the lease manager (kernel §7).
//!
//! Everything here is a pure function of its arguments — no I/O, no
//! wall-clock reads, no randomness, no generated identifiers. The
//! canonical fake family covers the shapes the J-16 conflicts surface,
//! the conformance tests and the Wave-5 gate harness drive:
//!
//! - [`fake_actors`] — the two-actor conflict family (Ana holds; Dev
//!   requests; Dev's agent appears in multi-holder shapes);
//! - [`ana_write_lease`] — the canonical held exclusive lease (Ana
//!   holds the resource until 15:40, under any policy);
//! - [`dev_write_request`] / [`dev_read_request`] — the canonical
//!   contending requests (exclusive and shared);
//! - [`conflict_space`] — the **complete** fake conflict space: every
//!   holder mode × every requester mode × every holder policy, so the
//!   conflict matrix and the conflict-honesty law are proven by
//!   exhaustive deterministic enumeration rather than sampling.

use crate::LeaseError;
use crate::lease::LeaseRecord;
use crate::mode::{AccessMode, ConflictPolicy};
use crate::refs::{ActorRef, LeaseRef, ResourceRef};
use crate::request::LeaseRequest;
use crate::time::Timestamp;

/// The fake resource the canonical conflict family contends for.
pub const FAKE_RESOURCE_ID: &str = "res_01J8ZQ5V8K3T2B7N6X4R9DQPC2";
/// A second fake resource (queue isolation).
pub const FAKE_OTHER_RESOURCE_ID: &str = "res_01J8ZQ5V8K3T2B7N6X4R9DQPD3";
/// The deterministic moment the canonical family decides and sweeps
/// at (inside Ana's lease window).
pub const FAKE_NOW: &str = "2026-09-23T15:04:00Z";
/// Ana's user principal (the holder of the canonical lease).
pub const FAKE_ANA_ID: &str = "user_ana";
/// Dev's user principal (the canonical requester).
pub const FAKE_DEV_ID: &str = "user_dev";
/// Dev's agent (the second holder in multi-holder shapes).
pub const FAKE_DEV_AGENT_ID: &str = "agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6";

/// The canonical fake lease ids, in slot order:
/// 0 Ana's held lease · 1 Dev's request · 2 Ana's second task's
/// request · 3 the other-resource request · 4 the closed-window
/// request · 5 the determinism pair's second holder · 6 the conflict
/// space's holder · 7 the conflict space's request.
pub const FAKE_LEASE_ID_STRINGS: [&str; 8] = [
    "lease_01J8ZQ5V8K3T2B7N6X4R9DQPH7",
    "lease_01J8ZQ5V8K3T2B7N6X4R9DQPJ8",
    "lease_01J8ZQ5V8K3T2B7N6X4R9DQPK9",
    "lease_01J8ZQ5V8K3T2B7N6X4R9DQPMA",
    "lease_01J8ZQ5V8K3T2B7N6X4R9DQPMB",
    "lease_01J8ZQ5V8K3T2B7N6X4R9DQPMC",
    "lease_01J8ZQ5V8K3T2B7N6X4R9DQPMD",
    "lease_01J8ZQ5V8K3T2B7N6X4R9DQPME",
];

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("the fake lease family is canonically valid: {error}"),
    }
}

/// The two-actor conflict family: Ana (the holder), Dev (the
/// requester) and Dev's agent (the second holder in multi-holder
/// shapes).
#[derive(Debug, Clone)]
pub struct FakeActors {
    /// Ana — a `user` principal; holds the canonical lease.
    pub ana: ActorRef,
    /// Dev — a `user` principal; the canonical requester.
    pub dev: ActorRef,
    /// Dev's agent — an `agent` actor for multi-holder shapes.
    pub dev_agent: ActorRef,
}

/// The two-actor conflict family.
#[must_use]
pub fn fake_actors() -> FakeActors {
    FakeActors {
        ana: ok(ActorRef::user(FAKE_ANA_ID)),
        dev: ok(ActorRef::user(FAKE_DEV_ID)),
        dev_agent: ok(ActorRef::agent(FAKE_DEV_AGENT_ID)),
    }
}

/// The fake resource the canonical conflict family contends for.
#[must_use]
pub fn fake_resource() -> ResourceRef {
    ok(ResourceRef::parse(FAKE_RESOURCE_ID))
}

/// A second fake resource (queue isolation).
#[must_use]
pub fn fake_other_resource() -> ResourceRef {
    ok(ResourceRef::parse(FAKE_OTHER_RESOURCE_ID))
}

/// The canonical fake lease ids, in slot order (see
/// [`FAKE_LEASE_ID_STRINGS`]).
#[must_use]
pub fn fake_lease_ids() -> [LeaseRef; 8] {
    [
        ok(LeaseRef::parse(FAKE_LEASE_ID_STRINGS[0])),
        ok(LeaseRef::parse(FAKE_LEASE_ID_STRINGS[1])),
        ok(LeaseRef::parse(FAKE_LEASE_ID_STRINGS[2])),
        ok(LeaseRef::parse(FAKE_LEASE_ID_STRINGS[3])),
        ok(LeaseRef::parse(FAKE_LEASE_ID_STRINGS[4])),
        ok(LeaseRef::parse(FAKE_LEASE_ID_STRINGS[5])),
        ok(LeaseRef::parse(FAKE_LEASE_ID_STRINGS[6])),
        ok(LeaseRef::parse(FAKE_LEASE_ID_STRINGS[7])),
    ]
}

/// The deterministic decision moment (inside Ana's lease window).
#[must_use]
pub fn now() -> Timestamp {
    ok(Timestamp::parse(FAKE_NOW))
}

/// The canonical held exclusive lease: Ana holds the fake resource in
/// `Write` from 15:00 until 15:40, under the given policy — held at
/// [`now`], expired at 15:41.
#[must_use]
pub fn ana_write_lease(policy: ConflictPolicy) -> LeaseRecord {
    ok(LeaseRecord::new(
        fake_lease_ids()[0].clone(),
        fake_resource(),
        AccessMode::Write,
        fake_actors().ana,
        ok(Timestamp::parse("2026-09-23T15:00:00Z")),
        ok(Timestamp::parse("2026-09-23T15:40:00Z")),
        policy,
    ))
}

/// The canonical contending exclusive request: Dev asks for the fake
/// resource in `Write` over a window alive at [`now`] (15:04–16:30).
///
/// # Errors
///
/// Returns [`LeaseError`] never, in practice — the canonical shape is
/// valid; the `Result` keeps the construction discipline honest.
pub fn dev_write_request() -> Result<LeaseRequest, LeaseError> {
    LeaseRequest::new(
        fake_lease_ids()[1].clone(),
        fake_resource(),
        AccessMode::Write,
        fake_actors().dev,
        now(),
        ok(Timestamp::parse("2026-09-23T16:30:00Z")),
        ConflictPolicy::Queue,
    )
}

/// The canonical contending shared request: Dev asks for the fake
/// resource in `Read` over the same alive window.
///
/// # Errors
///
/// Returns [`LeaseError`] never, in practice — the canonical shape is
/// valid.
pub fn dev_read_request() -> Result<LeaseRequest, LeaseError> {
    LeaseRequest::new(
        fake_lease_ids()[1].clone(),
        fake_resource(),
        AccessMode::Read,
        fake_actors().dev,
        now(),
        ok(Timestamp::parse("2026-09-23T16:30:00Z")),
        ConflictPolicy::Queue,
    )
}

/// One case of the fake conflict space: a request, the holder it meets
/// (if any conflict), and the decision moment.
#[derive(Debug, Clone)]
pub struct ConflictCase {
    /// The contending request.
    pub request: LeaseRequest,
    /// The held lease it meets.
    pub holder: LeaseRecord,
    /// The decision moment.
    pub now: Timestamp,
}

/// The **complete** fake conflict space: every holder mode × every
/// requester mode × every holder policy (5 × 5 × 3 = 75 cases), each
/// with Ana holding and Dev requesting, decided at [`now`] inside the
/// holder's window. Exhaustive deterministic enumeration — the
/// conflict matrix, the policy outcomes and the conflict-honesty law
/// are proven over the whole space, not sampled.
#[must_use]
pub fn conflict_space() -> Vec<ConflictCase> {
    let actors = fake_actors();
    let resource = fake_resource();
    let holder_id = fake_lease_ids()[6].clone();
    let request_id = fake_lease_ids()[7].clone();
    let mut cases = Vec::new();
    for holder_mode in AccessMode::ALL {
        for policy in ConflictPolicy::ALL {
            let holder = ok(LeaseRecord::new(
                holder_id.clone(),
                resource.clone(),
                holder_mode,
                actors.ana.clone(),
                ok(Timestamp::parse("2026-09-23T15:00:00Z")),
                ok(Timestamp::parse("2026-09-23T16:40:00Z")),
                policy,
            ));
            for requester_mode in AccessMode::ALL {
                let request = ok(LeaseRequest::new(
                    request_id.clone(),
                    resource.clone(),
                    requester_mode,
                    actors.dev.clone(),
                    now(),
                    ok(Timestamp::parse("2026-09-23T17:00:00Z")),
                    ConflictPolicy::Queue,
                ));
                cases.push(ConflictCase {
                    request,
                    holder: holder.clone(),
                    now: now(),
                });
            }
        }
    }
    cases
}
