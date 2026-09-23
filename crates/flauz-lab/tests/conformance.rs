//! Kernel conformance tests (LAB-001, Wave-4 addendum §4-§5, the
//! provider-neutral lab fabric): canonical JSON, the fixture set, the
//! adapter-output law, the findings-references-resolve law, and the
//! no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_lab::evidence::ProbeResult;
use flauz_lab::refs::{ProbeId, StateKey, StepId};
use flauz_lab::{
    ComparatorReport, EvidenceRecord, FakeRemoteLabAdapter, FindingsRegistry, JourneySpec,
    LocalLabAdapter, ProviderFamilies, compare, known_provider_families, run_journey,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/w4")
        .join(relative)
}

fn read_fixture(relative: &str) -> String {
    let content = fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
    // Canonical fixtures are committed with LF endings; a Windows checkout
    // with autocrlf translates them to CRLF. Normalize before comparison
    // so the round-trip law is tested against the CANONICAL form.
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
    let journey_fixtures = [
        "journey-spec/typical.json",
        "journey-spec/a11y-chord-ladder.json",
        "journey-spec/minimal.json",
    ];
    for relative in journey_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<JourneySpec>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
        let journey: JourneySpec = test_ok(serde_json::from_str(&content));
        test_ok(journey.validate());
    }
    let evidence_fixtures = [
        "evidence-record/typical.json",
        "evidence-record/fake-remote.json",
        "evidence-record/a11y-local.json",
        "evidence-record/a11y-fake-remote.json",
        "evidence-record/minimal.json",
    ];
    for relative in evidence_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<EvidenceRecord>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
        let record: EvidenceRecord = test_ok(serde_json::from_str(&content));
        test_ok(record.validate());
    }
    let report_fixtures = [
        "comparator-report/identical.json",
        "comparator-report/divergent.json",
    ];
    for relative in report_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<ComparatorReport>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
        let report: ComparatorReport = test_ok(serde_json::from_str(&content));
        test_ok(report.validate());
    }
    let registry_content = read_fixture("findings-registry/typical.json");
    let serialized = round_trip::<FindingsRegistry>(&registry_content);
    assert_eq!(
        serialized, registry_content,
        "the findings registry fixture is not canonical"
    );
    let families_content = read_fixture("provider-families/typical.json");
    let serialized = round_trip::<ProviderFamilies>(&families_content);
    assert_eq!(
        serialized, families_content,
        "the provider families fixture is not canonical"
    );
}

/// Invalid fixtures fail strict parses or validations — a record that
/// violates the canonical grammar or the record consistency laws is
/// rejected on read or on validation, never silently misread (addendum
/// §5's invalid-per-family rule).
#[test]
fn invalid_fixtures_fail_strict_parses_or_validations() {
    // Unknown fields are rejected on read, everywhere.
    for relative in [
        "journey-spec/invalid-unknown-field.json",
        "evidence-record/invalid-unknown-field.json",
        "findings-registry/invalid-unknown-field.json",
        "comparator-report/invalid-unknown-field.json",
        "provider-families/invalid-unknown-field.json",
    ] {
        let content = read_fixture(relative);
        let parsed: Result<JourneySpec, _> = serde_json::from_str(&content);
        let parsed_evidence: Result<EvidenceRecord, _> = serde_json::from_str(&content);
        let parsed_registry: Result<FindingsRegistry, _> = serde_json::from_str(&content);
        let parsed_report: Result<ComparatorReport, _> = serde_json::from_str(&content);
        let parsed_families: Result<ProviderFamilies, _> = serde_json::from_str(&content);
        assert!(
            parsed.is_err()
                || parsed_evidence.is_err()
                || parsed_registry.is_err()
                || parsed_report.is_err()
                || parsed_families.is_err(),
            "unknown fields must be rejected in {relative}"
        );
    }
    // A non-canonical chord is rejected on read.
    let bad_chord = read_fixture("journey-spec/invalid-chord.json");
    assert!(
        serde_json::from_str::<JourneySpec>(&bad_chord).is_err(),
        "non-canonical chords must be rejected on read"
    );
    // Duplicated step ids parse but fail validation (the journey law).
    let duplicated = read_fixture("journey-spec/invalid-duplicate-step.json");
    let journey: JourneySpec = test_ok(serde_json::from_str(&duplicated));
    assert!(
        journey.validate().is_err(),
        "duplicated step ids must fail validation"
    );
    // A malformed frame digest is rejected on read.
    let bad_digest = read_fixture("evidence-record/invalid-bad-digest.json");
    assert!(
        serde_json::from_str::<EvidenceRecord>(&bad_digest).is_err(),
        "malformed digests must be rejected on read"
    );
    // Duplicated finding ids parse but fail validation.
    let duplicated_findings = read_fixture("findings-registry/invalid-duplicate-findings.json");
    let registry: FindingsRegistry = test_ok(serde_json::from_str(&duplicated_findings));
    assert!(
        registry.validate().is_err(),
        "duplicated finding ids must fail validation"
    );
}

/// The adapter-output law (the CAP-001 pattern): every committed
/// evidence fixture EQUALS the adapter run over the committed journey
/// fixture it documents — the fixtures are honest artifacts of the
/// adapters, never hand-staged shapes. Full evidence families for BOTH
/// adapters: the shell-discovery and a11y chord-ladder journeys, each
/// run on local and fake-remote.
#[test]
fn evidence_fixtures_are_the_adapter_output_over_their_journeys() {
    let checks: [(&str, &str, bool); 5] = [
        (
            "evidence-record/typical.json",
            "journey-spec/typical.json",
            false,
        ),
        (
            "evidence-record/fake-remote.json",
            "journey-spec/typical.json",
            true,
        ),
        (
            "evidence-record/a11y-local.json",
            "journey-spec/a11y-chord-ladder.json",
            false,
        ),
        (
            "evidence-record/a11y-fake-remote.json",
            "journey-spec/a11y-chord-ladder.json",
            true,
        ),
        (
            "evidence-record/minimal.json",
            "journey-spec/minimal.json",
            false,
        ),
    ];
    for (record_fixture, journey_fixture, remote) in checks {
        let journey: JourneySpec = test_ok(serde_json::from_str(&read_fixture(journey_fixture)));
        let expected: EvidenceRecord = test_ok(serde_json::from_str(&read_fixture(record_fixture)));
        let record = if remote {
            let mut adapter = FakeRemoteLabAdapter::new();
            test_ok(run_journey(&mut adapter, &journey))
        } else {
            let mut adapter = LocalLabAdapter::new();
            test_ok(run_journey(&mut adapter, &journey))
        };
        assert_eq!(
            test_ok(serde_json::to_string_pretty(&record)),
            test_ok(serde_json::to_string_pretty(&expected)),
            "{record_fixture} must equal the adapter output over {journey_fixture}"
        );
    }
}

/// The comparator-report fixtures are the comparator's honest outputs:
/// the identical fixture is the cross-provider zero-divergence report
/// (the F8 artifact), and the divergent fixture is the seeded-divergence
/// report — named, severe, located.
#[test]
fn comparator_report_fixtures_are_the_comparator_outputs() {
    let journey: JourneySpec = test_ok(serde_json::from_str(&read_fixture(
        "journey-spec/typical.json",
    )));
    let reference: EvidenceRecord = test_ok(serde_json::from_str(&read_fixture(
        "evidence-record/typical.json",
    )));
    let candidate: EvidenceRecord = test_ok(serde_json::from_str(&read_fixture(
        "evidence-record/fake-remote.json",
    )));

    let identical: ComparatorReport = test_ok(serde_json::from_str(&read_fixture(
        "comparator-report/identical.json",
    )));
    let computed = test_ok(compare(&journey, &reference, &candidate));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&computed)),
        test_ok(serde_json::to_string_pretty(&identical)),
        "the identical fixture is the cross-provider zero-divergence report"
    );

    let divergent: ComparatorReport = test_ok(serde_json::from_str(&read_fixture(
        "comparator-report/divergent.json",
    )));
    // The seeded candidate: the fake-remote record with the picker panel
    // observed closed at the open-picker moment.
    let step_id = test_ok(StepId::parse("open-picker"));
    let probe_id = test_ok(ProbeId::parse("frame-picker-open"));
    let mut seeded = candidate.clone();
    let step = seeded
        .steps
        .iter_mut()
        .find(|step| step.step == step_id)
        .unwrap_or_else(|| panic!("the record evidences step {step_id}"));
    let probe = step
        .probes
        .iter_mut()
        .find(|probe| probe.probe == probe_id)
        .unwrap_or_else(|| panic!("the record evidences probe {probe_id}"));
    match &mut probe.result {
        ProbeResult::Frame { anchors, .. } => {
            let observation = anchors
                .iter_mut()
                .find(|observation| observation.anchor.as_str() == "model-picker-panel")
                .unwrap_or_else(|| panic!("the frame observes the picker anchor"));
            observation.state = test_ok(StateKey::parse("closed"));
        }
        _ => panic!("the frame probe must be a frame"),
    }
    test_ok(seeded.validate());
    let computed = test_ok(compare(&journey, &reference, &seeded));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&computed)),
        test_ok(serde_json::to_string_pretty(&divergent)),
        "the divergent fixture is the seeded-divergence report"
    );
}

/// The public fakes reproduce the committed journey fixtures exactly:
/// the fake surface is the reference semantics (kernel §7 determinism —
/// the fakes are stable across runs), and the provider-families fixture
/// is the known topology table.
#[test]
fn the_public_fakes_match_the_committed_fixtures() {
    let shell = flauz_lab::fakes::fake_shell_discovery_journey();
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&shell)),
        read_fixture("journey-spec/typical.json")
    );
    let ladder = flauz_lab::fakes::fake_a11y_chord_ladder_journey();
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&ladder)),
        read_fixture("journey-spec/a11y-chord-ladder.json")
    );
    let registry = test_ok(flauz_lab::builtin_findings_registry());
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&registry)),
        read_fixture("findings-registry/typical.json")
    );
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&known_provider_families())),
        read_fixture("provider-families/typical.json")
    );
}

/// Findings referenced by id resolve (the work order's acceptance
/// criterion 4): every finding link in every committed evidence record
/// resolves against the builtin registry — records reference findings
/// by id, never by prose.
#[test]
fn findings_referenced_by_id_resolve() {
    let registry = test_ok(flauz_lab::builtin_findings_registry());
    let evidence_fixtures = [
        "evidence-record/typical.json",
        "evidence-record/fake-remote.json",
        "evidence-record/a11y-local.json",
        "evidence-record/a11y-fake-remote.json",
        "evidence-record/minimal.json",
    ];
    for relative in evidence_fixtures {
        let record: EvidenceRecord = test_ok(serde_json::from_str(&read_fixture(relative)));
        assert!(
            registry.resolves_all(&record.observed_findings()),
            "every finding link in {relative} must resolve in the builtin registry"
        );
    }
    // The registry carries the five F1 a11y lessons the work order names.
    let ids: Vec<String> = registry
        .findings
        .iter()
        .map(|finding| finding.id.as_str().to_owned())
        .collect();
    assert_eq!(
        ids,
        vec![
            "bracket-swap-chords",
            "first-run-keyboard-swallowing",
            "modal-focus-traps",
            "pty-focus-transfer",
            "shifted-symbol-chord-companions",
        ]
    );
}

/// Canonical JSON carries no floats (kernel §4): every serialized
/// contract value is an integer, a string, a boolean, null, an array or
/// an object — and every document carries `"v": 1`.
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
        "journey-spec/typical.json",
        "journey-spec/a11y-chord-ladder.json",
        "journey-spec/minimal.json",
        "evidence-record/typical.json",
        "evidence-record/fake-remote.json",
        "evidence-record/a11y-local.json",
        "evidence-record/a11y-fake-remote.json",
        "evidence-record/minimal.json",
        "comparator-report/identical.json",
        "comparator-report/divergent.json",
        "findings-registry/typical.json",
        "provider-families/typical.json",
    ];
    for relative in all_fixtures {
        let value: serde_json::Value = test_ok(serde_json::from_str(&read_fixture(relative)));
        assert_no_floats(&value, relative);
        assert_eq!(
            value["v"],
            serde_json::json!(1),
            "{relative} carries \"v\": 1"
        );
    }
}

/// No credential material anywhere (Wave-4 addendum §3): the lab's state
/// is journeys, evidence, findings and provider topology — no credential
/// VALUES, no secret material. Any credential-looking field in serialized
/// state is an automatic gate failure.
///
/// The scan adapts the repo's `CREDENTIAL_MARKERS` family to a
/// whole-document scan: field-name markers are scanned quoted (they name
/// JSON fields), and prefix markers (`sk-`, `xoxb-`, `ghp_`, `bearer `)
/// are scanned at value starts (`"sk-`) — the marker's meaning in
/// `flauz-exec`'s single-field check — so legitimate identifiers that
/// merely CONTAIN the letters (for example the `task-surface` UI
/// surface) do not false-positive while real key-shaped values still do.
#[test]
fn no_credential_material_in_serialized_state() {
    let all_fixtures = [
        "journey-spec/typical.json",
        "journey-spec/a11y-chord-ladder.json",
        "journey-spec/minimal.json",
        "evidence-record/typical.json",
        "evidence-record/fake-remote.json",
        "evidence-record/a11y-local.json",
        "evidence-record/a11y-fake-remote.json",
        "evidence-record/minimal.json",
        "comparator-report/identical.json",
        "comparator-report/divergent.json",
        "findings-registry/typical.json",
        "provider-families/typical.json",
    ];
    for relative in all_fixtures {
        let content = read_fixture(relative);
        let lower = content.to_lowercase();
        for forbidden in [
            "\"token\"",
            "\"secret\"",
            "\"password\"",
            "\"passwd\"",
            "\"api_key\"",
            "\"apikey\"",
            "\"client_secret\"",
            "\"access_token\"",
            "\"refresh_token\"",
            "credential",
            "\"sk-",
            "\"bearer ",
            "\"xoxb-",
            "\"ghp_",
        ] {
            assert!(
                !lower.contains(forbidden),
                "credential material {forbidden:?} must never appear in {relative}"
            );
        }
    }
}
