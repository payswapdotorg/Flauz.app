//! Wave-2 capability-resolver step (CAP-001): the resolver over the PUBLIC
//! fake input space (the complete deterministic 144-case enumeration —
//! every case re-validated, every gap named, no silent fall-through),
//! the resolver-output law against the committed fixtures, canonical JSON
//! discipline, and the byte-identical determinism/replay property.
//!
//! Lead-authored gate infrastructure (the F2 §10 harness pattern), NOT a
//! workspace crate. No external service anywhere; deterministic.

use flauz_cap::dimension::Dimension;
use flauz_cap::fakes::{all_admitted_inputs, fake_input_space, typical_inputs};
use flauz_cap::key::CapabilityKey;
use flauz_cap::resolution::CapabilityResolution;
use flauz_cap::resolver::resolve_capability;
use flauz_cap::{PermissionDecision, PolicyDecision, ResolutionInputs};

fn ok<T, E: std::fmt::Debug>(r: Result<T, E>, what: &str) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("gate: {what}: {e:?}"),
    }
}

/// Credential-material markers (mirrors the contract-crates' private
/// lists — the gate re-checks the invariant independently).
const CREDENTIAL_MARKERS: &[&str] = &[
    "sk-",
    "Bearer ",
    "api_key",
    "apikey",
    "password",
    "passwd",
    "client_secret",
    "access_token",
];

pub fn capability_resolver_step() {
    // -- the complete fake input space: 144 deterministic cases --
    let space = ok(fake_input_space(), "fake input space");
    assert_eq!(
        space.len(),
        144,
        "3 capability keys × 48 dimension shapes = the complete enumeration"
    );

    let mut available_count = 0usize;
    let mut gap_count = 0usize;
    for case in &space {
        let record = ok(
            resolve_capability(&case.capability, &case.inputs),
            "resolve over the fake space",
        );
        ok(record.validate(), "every record self-validates");

        if record.available {
            available_count += 1;
            assert!(
                record.gaps.is_empty() && record.unlock_paths.is_empty(),
                "an available capability carries NO gaps and NO unlock paths"
            );
            assert!(
                record.admissions.iter().all(|a| a.admits()),
                "an available capability is admitted by every dimension"
            );
        } else {
            gap_count += 1;
            // THE no-silent-fall-through property: every requested-but-
            // missing capability yields a resolution with a NON-EMPTY
            // named gap list, gaps/unlocks mirroring the missing
            // dimensions exactly.
            assert!(
                !record.gaps.is_empty(),
                "an unavailable capability MUST name its gaps (no silent fall-through)"
            );
            assert_eq!(
                record.gaps.len(),
                record.unlock_paths.len(),
                "gaps and unlock paths mirror one-for-one"
            );
            for (gap, unlock) in record.gaps.iter().zip(record.unlock_paths.iter()) {
                assert_eq!(
                    gap.dimension, unlock.dimension,
                    "each gap pairs with its same-dimension unlock path"
                );
                assert!(
                    !gap.reason.trim().is_empty() && !unlock.action.trim().is_empty(),
                    "gap reasons and unlock actions are non-empty user language"
                );
            }
            // the admitted dimensions are exactly the non-gap dimensions
            let gap_dims: Vec<Dimension> =
                record.gaps.iter().map(|g| g.dimension).collect();
            for dimension in Dimension::ALL {
                let admission = record
                    .admission(dimension)
                    .unwrap_or_else(|| panic!("admission for {dimension:?}"));
                let is_gap = gap_dims.contains(&dimension);
                assert_eq!(
                    admission.admits(),
                    !is_gap,
                    "the admission for {dimension:?} matches the gap list"
                );
            }
        }

        // canonical JSON round-trip: byte-stable serialization
        let json = serde_json::to_string(&record).expect("gate: record serializes");
        let reloaded: CapabilityResolution =
            serde_json::from_str(&json).expect("gate: record re-parses");
        assert_eq!(
            serde_json::to_string(&reloaded).expect("gate: reloaded serializes"),
            json,
            "canonical JSON is byte-stable across round-trips"
        );
        for marker in CREDENTIAL_MARKERS {
            assert!(
                !json.contains(marker),
                "gate: serialized resolution carries credential material ({marker:?})"
            );
        }
    }
    assert!(
        available_count > 0 && gap_count > 0,
        "the space exercises BOTH outcomes (available {available_count}, gap {gap_count})"
    );

    // -- determinism/replay: the same inputs resolve byte-identically --
    let first = ok(
        resolve_capability(&space[0].capability, &space[0].inputs),
        "first resolution",
    );
    let replay = ok(
        resolve_capability(&space[0].capability, &space[0].inputs),
        "replay resolution",
    );
    assert_eq!(
        serde_json::to_string(&first).expect("ser"),
        serde_json::to_string(&replay).expect("ser"),
        "the resolver is deterministic: replays are byte-identical"
    );

    // -- the resolver-output law against the COMMITTED fixtures: every
    //    resolution fixture EQUALS the resolver run over its documented
    //    inputs fixture (the crates' conformance law, re-proven here
    //    against the merged binary) --
    let inputs_fixture = include_str!(
        "/home/z/Flauz.app/crates/flauz-cap/tests/fixtures/w2/resolution-inputs/typical.json"
    );
    let resolution_fixture = include_str!(
        "/home/z/Flauz.app/crates/flauz-cap/tests/fixtures/w2/capability-resolution/typical.json"
    );
    let inputs: ResolutionInputs =
        serde_json::from_str(inputs_fixture).expect("inputs fixture parses");
    let expected: CapabilityResolution =
        serde_json::from_str(resolution_fixture).expect("resolution fixture parses");
    let capability = ok(CapabilityKey::parse("browser.input"), "the typical fixture key");
    let produced = ok(resolve_capability(&capability, &inputs), "resolve typical");
    assert_eq!(
        serde_json::to_string(&produced).expect("ser"),
        serde_json::to_string(&expected).expect("ser"),
        "the resolver-output law: the committed resolution fixture IS the resolver over the committed inputs fixture"
    );
    assert!(!produced.available, "the typical fixture is the gap case");
    assert_eq!(produced.gaps.len(), 2, "environment + policy are the named gaps");

    // -- the all-admitted fixture: the success state (terminal) --
    let admitted_inputs = ok(all_admitted_inputs(), "all-admitted inputs");
    let terminal_key = ok(CapabilityKey::parse("terminal"), "the all-admitted fixture key");
    let admitted = ok(
        resolve_capability(&terminal_key, &admitted_inputs),
        "resolve all-admitted",
    );
    assert!(
        admitted.available && admitted.gaps.is_empty(),
        "every dimension admitting ⇒ available with no gaps"
    );
    // the fixture law on the all-admitted pair too
    let admitted_inputs_fixture = include_str!(
        "/home/z/Flauz.app/crates/flauz-cap/tests/fixtures/w2/resolution-inputs/all-admitted.json"
    );
    let admitted_fixture = include_str!(
        "/home/z/Flauz.app/crates/flauz-cap/tests/fixtures/w2/capability-resolution/all-admitted.json"
    );
    let inputs_json: ResolutionInputs =
        serde_json::from_str(admitted_inputs_fixture).expect("all-admitted inputs fixture parses");
    assert_eq!(inputs_json, admitted_inputs, "the public fake equals its committed fixture");
    let expected_admitted: CapabilityResolution =
        serde_json::from_str(admitted_fixture).expect("all-admitted fixture parses");
    assert_eq!(
        serde_json::to_string(&admitted).expect("ser"),
        serde_json::to_string(&expected_admitted).expect("ser"),
        "the resolver-output law holds on the all-admitted pair too"
    );

    // -- caller-data re-validation: the resolver never trusts
    //    pre-validated inputs. The canonical constructor rejects
    //    non-canonical data at the boundary, and the resolver re-validates
    //    even a hand-tampered record --
    assert!(
        CapabilityKey::parse("Not-A-Valid-Key").is_err(),
        "the capability key grammar is enforced at parse"
    );
    assert!(
        ResolutionInputs::new(
            vec![
                ok(CapabilityKey::parse("web.search"), "key"),
                ok(CapabilityKey::parse("browser.input"), "key"),
            ],
            vec![ok(CapabilityKey::parse("browser.input"), "key")],
            Some(vec![ok(CapabilityKey::parse("browser.input"), "key")]),
            PermissionDecision::Granted,
            PolicyDecision::Allowed,
        )
        .is_err(),
        "the canonical constructor rejects unsorted advertisement lists at the boundary"
    );
    let probe = ok(CapabilityKey::parse("terminal"), "a valid probe key");
    let mut tampered = ok(typical_inputs(), "typical inputs");
    // hand-tamper past the constructor: an unsorted model advertisement
    tampered.model_advertised = vec![
        ok(CapabilityKey::parse("web.search"), "key"),
        ok(CapabilityKey::parse("browser.input"), "key"),
    ];
    let rejected = resolve_capability(&probe, &tampered);
    assert!(
        rejected.is_err(),
        "the resolver re-validates caller data (a hand-tampered unsorted list is rejected): {rejected:?}"
    );
}
