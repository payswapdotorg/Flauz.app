//! Kernel conformance tests (F2 Wave-2 addendum §4-§5, the capability
//! resolver): canonical JSON, the fixture set, the resolver-output law,
//! and the no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_cap::fakes::{all_admitted_inputs, minimal_inputs, typical_inputs};
use flauz_cap::inputs::ResolutionInputs;
use flauz_cap::key::CapabilityKey;
use flauz_cap::resolution::CapabilityResolution;
use flauz_cap::resolver::resolve_capability;

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/w2")
        .join(relative)
}

fn read_fixture(relative: &str) -> String {
    let content = fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
    // Canonical fixtures are committed with LF endings; a Windows checkout
    // with autocrlf translates them to CRLF. Normalize before comparison
    // so the round-trip law is tested against the CANONICAL form, not the
    // platform's line-ending translation.
    content.replace("\r\n", "\n").trim_end().to_owned()
}

fn round_trip<T>(content: &str) -> String
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = test_ok(serde_json::from_str(content));
    test_ok(serde_json::to_string_pretty(&parsed))
}

/// Every valid conformance fixture is canonical JSON: it parses strictly
/// (unknown fields rejected) and re-serializes to exactly the committed
/// bytes (addendum §5 — canonical JSON per the kernel §4 rules).
#[test]
fn fixtures_roundtrip_canonical() {
    let inputs_fixtures = [
        "resolution-inputs/typical.json",
        "resolution-inputs/minimal.json",
        "resolution-inputs/all-admitted.json",
    ];
    for relative in inputs_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<ResolutionInputs>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let record_fixtures = [
        "capability-resolution/typical.json",
        "capability-resolution/minimal.json",
        "capability-resolution/all-admitted.json",
    ];
    for relative in record_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<CapabilityResolution>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
}

/// Invalid fixtures fail strict parses — a record that violates the
/// canonical grammar, the frozen capability-key grammar, or the record
/// consistency laws is rejected on read, never silently misread
/// (addendum §5's invalid-per-entity rule).
#[test]
fn invalid_fixtures_fail_strict_parses() {
    let unknown_field = read_fixture("capability-resolution/invalid-unknown-field.json");
    assert!(
        serde_json::from_str::<CapabilityResolution>(&unknown_field).is_err(),
        "unknown fields must be rejected"
    );
    let bad_key = read_fixture("capability-resolution/invalid-capability-key.json");
    assert!(
        serde_json::from_str::<CapabilityResolution>(&bad_key).is_err(),
        "capability keys outside the frozen grammar must be rejected on read"
    );
    let inconsistent = read_fixture("capability-resolution/invalid-inconsistent.json");
    let record: CapabilityResolution = test_ok(serde_json::from_str(&inconsistent));
    assert!(
        record.validate().is_err(),
        "available=true with named gaps is the silent fall-through shape — invalid"
    );
    let unsorted = read_fixture("resolution-inputs/invalid-unsorted.json");
    let inputs: ResolutionInputs = test_ok(serde_json::from_str(&unsorted));
    assert!(
        inputs.validate().is_err(),
        "unsorted advertisements must be rejected"
    );
}

/// The resolver-output law: every committed resolution fixture EQUALS the
/// resolver run over the committed inputs fixture it documents — the
/// fixtures are honest artifacts of the resolver, never hand-staged
/// shapes (the no-silent-fall-through companion law).
#[test]
fn resolution_fixtures_are_the_resolver_output_over_their_inputs() {
    let checks: [(&str, &str, &str); 3] = [
        (
            "capability-resolution/typical.json",
            "resolution-inputs/typical.json",
            "browser.input",
        ),
        (
            "capability-resolution/minimal.json",
            "resolution-inputs/minimal.json",
            "terminal",
        ),
        (
            "capability-resolution/all-admitted.json",
            "resolution-inputs/all-admitted.json",
            "terminal",
        ),
    ];
    for (record_fixture, inputs_fixture, capability) in checks {
        let inputs: ResolutionInputs = test_ok(serde_json::from_str(&read_fixture(inputs_fixture)));
        let expected: CapabilityResolution =
            test_ok(serde_json::from_str(&read_fixture(record_fixture)));
        let record = test_ok(resolve_capability(
            &test_ok(CapabilityKey::parse(capability)),
            &inputs,
        ));
        assert_eq!(
            test_ok(serde_json::to_string_pretty(&record)),
            test_ok(serde_json::to_string_pretty(&expected)),
            "{record_fixture} must equal the resolver output over {inputs_fixture}"
        );
    }
}

/// The public fakes reproduce the committed inputs fixtures exactly: the
/// fake surface is the reference semantics for real resolution inputs
/// (kernel §7 determinism — the fakes are stable across runs).
#[test]
fn the_public_fakes_match_the_committed_inputs_fixtures() {
    let typical: ResolutionInputs = test_ok(serde_json::from_str(&read_fixture(
        "resolution-inputs/typical.json",
    )));
    assert_eq!(test_ok(typical_inputs()), typical);
    let minimal: ResolutionInputs = test_ok(serde_json::from_str(&read_fixture(
        "resolution-inputs/minimal.json",
    )));
    assert_eq!(test_ok(minimal_inputs()), minimal);
    let admitted: ResolutionInputs = test_ok(serde_json::from_str(&read_fixture(
        "resolution-inputs/all-admitted.json",
    )));
    assert_eq!(test_ok(all_admitted_inputs()), admitted);
}

/// Canonical JSON carries no floats (kernel §4): every serialized
/// contract value is an integer, a string, a boolean, null, an array or
/// an object — scanning the fixtures keeps the law visible at the
/// contract surface.
#[test]
fn serialized_contract_state_contains_no_floats() {
    fn assert_no_floats(value: &serde_json::Value, path: &str) {
        match value {
            serde_json::Value::Number(number) => assert!(
                number.is_u64() || number.is_i64(),
                "float at {path}: {number}"
            ),
            serde_json::Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    assert_no_floats(item, &format!("{path}[{index}]"));
                }
            }
            serde_json::Value::Object(fields) => {
                for (name, item) in fields {
                    assert_no_floats(item, &format!("{path}.{name}"));
                }
            }
            _ => {}
        }
    }
    let all_fixtures = [
        "resolution-inputs/typical.json",
        "resolution-inputs/minimal.json",
        "resolution-inputs/all-admitted.json",
        "capability-resolution/typical.json",
        "capability-resolution/minimal.json",
        "capability-resolution/all-admitted.json",
    ];
    for relative in all_fixtures {
        let value: serde_json::Value = test_ok(serde_json::from_str(&read_fixture(relative)));
        assert_no_floats(&value, relative);
        // The schema version marker is exactly 1.
        assert_eq!(
            value["v"],
            serde_json::json!(1),
            "{relative} carries \"v\": 1"
        );
    }
}

/// No credential material anywhere (addendum §3): the resolver's state is
/// advertisements and decisions — no credential VALUES, no secret
/// material, only plain capability keys. Any credential-looking field in
/// serialized state is an automatic gate failure.
#[test]
fn no_credential_material_in_serialized_state() {
    let all_fixtures = [
        "resolution-inputs/typical.json",
        "resolution-inputs/minimal.json",
        "resolution-inputs/all-admitted.json",
        "capability-resolution/typical.json",
        "capability-resolution/minimal.json",
        "capability-resolution/all-admitted.json",
    ];
    for relative in all_fixtures {
        let content = read_fixture(relative);
        let lower = content.to_lowercase();
        for forbidden in [
            "\"token\"",
            "\"secret\"",
            "\"password\"",
            "\"api_key\"",
            "\"apikey\"",
            "\"credential\"",
            "sk-",
            "bearer ",
        ] {
            assert!(
                !lower.contains(forbidden),
                "credential material {forbidden:?} must never appear in {relative}"
            );
        }
    }
}
