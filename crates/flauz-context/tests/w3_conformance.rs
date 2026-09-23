//! Wave-3 conformance tests (ORCH-002, kernel §5/§8): the w3 fixture
//! families round-trip byte-identical, invalid fixtures fail strict
//! parses or canonical validation, the committed fixtures are honest
//! artifacts of the fakes and the engine functions, and no fixture
//! carries credential material or floats.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_context::fakes::{
    fake_capability_admissions, fake_large_budget_profile, fake_minimal_durable_state,
    fake_pressure_durable_state, fake_small_budget_profile, fake_tool_catalog,
    fake_typical_durable_state,
};
use flauz_context::profile::{ModelContextProfile, MultimodalBehavior, ToolSchemaHandling};
use flauz_context::provenance::{ContextProvenance, ContextSource};
use flauz_context::{
    ActorRef, CapabilityAdmission, CompactionBudget, CompactionReason, CompactionRecord,
    ContextItem, ContextItemContent, ContextSnapshot, ContextSnapshotId, DurableStateInputs,
    MemoryContent, MemoryItem, ModelRef, SessionRef, TaskRef, Timestamp, ToolDefinition,
    ToolExposure, compute_tool_exposure, plan_compaction,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/w3")
        .join(relative)
}

fn read_fixture(relative: &str) -> String {
    let content = fs::read_to_string(fixture_path(relative))
        .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
    content.replace("\r\n", "\n").trim_end().to_owned()
}

fn round_trip<T>(content: &str) -> String
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = test_ok(serde_json::from_str(content));
    test_ok(serde_json::to_string_pretty(&parsed))
}

const TASK: &str = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";
const SESSION: &str = "sess_01J8ZQ5V8K3T2B7N6X4R9DQPG6";
const MODEL_A: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPRE";
const SNAPSHOT: &str = "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5";

fn memory_item_context(item: &MemoryItem) -> ContextItem {
    let content = match &item.content {
        MemoryContent::Text { text } => ContextItemContent::Text { text: text.clone() },
        MemoryContent::Reference { reference } => ContextItemContent::Reference {
            reference: reference.clone(),
        },
    };
    test_ok(ContextItem::new(
        content,
        item.tier,
        test_ok(ContextProvenance::new(
            ContextSource::MemoryItem {
                memory_item_id: item.id.clone(),
            },
            item.authorization,
        )),
    ))
}

/// The pressure snapshot (fixed `ctxsnap` ID) the compaction fixtures
/// document — identical to the w3_tiers builder.
fn pressure_snapshot() -> ContextSnapshot {
    let items = test_ok(fake_pressure_durable_state())
        .memory_items
        .iter()
        .map(memory_item_context)
        .collect();
    test_ok(ContextSnapshot::new(
        test_ok(ContextSnapshotId::parse(SNAPSHOT)),
        test_ok(TaskRef::parse(TASK)),
        Some(test_ok(SessionRef::parse(SESSION))),
        test_ok(ModelRef::parse(MODEL_A)),
        test_ok(ActorRef::system("flauz-context-engine")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
        items,
    ))
}

fn exposure_profile() -> ModelContextProfile {
    test_ok(ModelContextProfile::new(
        test_ok(ModelRef::parse(MODEL_A)),
        200_000,
        MultimodalBehavior::ImageInput,
        ToolSchemaHandling::SummariesWithLazySchemas,
        test_ok(ActorRef::system("flauz-fake")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ))
}

/// Every valid w3 fixture is canonical JSON: it parses strictly (unknown
/// fields rejected) and re-serializes to exactly the committed bytes
/// (kernel §4 round-trip law, addendum §5).
#[test]
fn w3_fixtures_roundtrip_canonical() {
    let checks: &[(&str, FixtureKind)] = &[
        ("durable-state/typical.json", FixtureKind::DurableState),
        ("durable-state/minimal.json", FixtureKind::DurableState),
        ("tool-definitions/typical.json", FixtureKind::ToolList),
        ("tool-definitions/minimal.json", FixtureKind::ToolList),
        (
            "capability-admissions/typical.json",
            FixtureKind::AdmissionList,
        ),
        (
            "capability-admissions/minimal.json",
            FixtureKind::AdmissionList,
        ),
        ("tool-exposure/typical.json", FixtureKind::Exposure),
        ("tool-exposure/minimal.json", FixtureKind::Exposure),
        (
            "compaction-record/typical.json",
            FixtureKind::CompactionRecord,
        ),
        (
            "compaction-record/minimal.json",
            FixtureKind::CompactionRecord,
        ),
    ];
    for (relative, kind) in checks {
        let content = read_fixture(relative);
        let serialized = match kind {
            FixtureKind::DurableState => round_trip::<DurableStateInputs>(&content),
            FixtureKind::ToolList => round_trip::<Vec<ToolDefinition>>(&content),
            FixtureKind::AdmissionList => round_trip::<Vec<CapabilityAdmission>>(&content),
            FixtureKind::Exposure => round_trip::<ToolExposure>(&content),
            FixtureKind::CompactionRecord => round_trip::<CompactionRecord>(&content),
        };
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
}

enum FixtureKind {
    DurableState,
    ToolList,
    AdmissionList,
    Exposure,
    CompactionRecord,
}

/// Invalid fixtures fail strict parses or canonical validation — never
/// silently misread (addendum §5's invalid-per-family rule).
#[test]
fn w3_invalid_fixtures_fail_strict_parses() {
    // Unknown fields are rejected on read.
    for relative in [
        "durable-state/invalid-unknown-field.json",
        "tool-definitions/invalid-unknown-field.json",
        "tool-exposure/invalid-unknown-field.json",
        "compaction-record/invalid-unknown-field.json",
    ] {
        let content = read_fixture(relative);
        assert!(
            serde_json::from_str::<serde_json::Value>(&content).is_ok(),
            "{relative} must be well-formed JSON (invalid by contract, not by syntax)"
        );
        assert!(
            serde_json::from_str::<DurableStateInputs>(&content).is_err()
                || serde_json::from_str::<Vec<ToolDefinition>>(&content).is_err()
                || serde_json::from_str::<ToolExposure>(&content).is_err()
                || serde_json::from_str::<CompactionRecord>(&content).is_err(),
            "{relative} must fail a strict canonical parse"
        );
    }

    // Duplicate memory records parse but fail canonical validation.
    let duplicated = read_fixture("durable-state/invalid-duplicate-memory.json");
    let inputs: DurableStateInputs = test_ok(serde_json::from_str(&duplicated));
    assert!(
        inputs.validate().is_err(),
        "duplicate memory-item records must fail validation, never silently merge"
    );

    // The silent-fall-through admission shape parses but fails
    // validation (available=true with a named gap).
    let inconsistent = read_fixture("capability-admissions/invalid-inconsistent.json");
    let admissions: Vec<CapabilityAdmission> = test_ok(serde_json::from_str(&inconsistent));
    assert!(
        admissions
            .iter()
            .any(|admission| admission.validate().is_err()),
        "available=true with named gaps is the silent fall-through shape — invalid"
    );

    // A capability key outside the frozen grammar fails validation.
    let bad_key = read_fixture("tool-exposure/invalid-capability-key.json");
    let exposure: ToolExposure = test_ok(serde_json::from_str(&bad_key));
    assert!(
        exposure.validate().is_err(),
        "capability keys outside the frozen grammar must be rejected on read"
    );
}

/// The committed fixtures are honest artifacts: every record fixture
/// equals the engine function (or fake) run over the input fixture it
/// documents — never hand-staged shapes (the w2 resolver-output law,
/// carried to Wave 3).
#[test]
fn w3_fixtures_are_honest_artifacts_of_the_fakes_and_engine() {
    // The durable-state fixtures are the fakes, byte-identical.
    let typical: DurableStateInputs = test_ok(serde_json::from_str(&read_fixture(
        "durable-state/typical.json",
    )));
    assert_eq!(typical, test_ok(fake_typical_durable_state()));
    let minimal: DurableStateInputs = test_ok(serde_json::from_str(&read_fixture(
        "durable-state/minimal.json",
    )));
    assert_eq!(minimal, test_ok(fake_minimal_durable_state()));

    // The tool catalog and admissions fixtures are the fakes.
    let catalog: Vec<ToolDefinition> = test_ok(serde_json::from_str(&read_fixture(
        "tool-definitions/typical.json",
    )));
    assert_eq!(catalog, test_ok(fake_tool_catalog()));
    let admissions: Vec<CapabilityAdmission> = test_ok(serde_json::from_str(&read_fixture(
        "capability-admissions/typical.json",
    )));
    assert_eq!(admissions, test_ok(fake_capability_admissions()));

    // The exposure fixture is compute_tool_exposure over the catalog and
    // admissions fixtures, for the documented profile.
    let exposure: ToolExposure = test_ok(serde_json::from_str(&read_fixture(
        "tool-exposure/typical.json",
    )));
    let computed = test_ok(compute_tool_exposure(
        &exposure_profile(),
        &catalog,
        &admissions,
    ));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&computed)),
        test_ok(serde_json::to_string_pretty(&exposure))
    );

    // The compaction-record fixtures are plan_compaction over the
    // pressure snapshot, for the documented budgets.
    let typical_record: CompactionRecord = test_ok(serde_json::from_str(&read_fixture(
        "compaction-record/typical.json",
    )));
    let compacted = test_ok(plan_compaction(
        &pressure_snapshot(),
        &test_ok(CompactionBudget::from_profile(&test_ok(
            fake_small_budget_profile(),
        ))),
        CompactionReason::Pressure,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
    ));
    assert_eq!(compacted.record, typical_record);

    let minimal_record: CompactionRecord = test_ok(serde_json::from_str(&read_fixture(
        "compaction-record/minimal.json",
    )));
    let within = test_ok(plan_compaction(
        &pressure_snapshot(),
        &test_ok(CompactionBudget::from_profile(&test_ok(
            fake_large_budget_profile(),
        ))),
        CompactionReason::Manual,
        test_ok(ActorRef::user("alice")),
        test_ok(Timestamp::parse("2026-09-21T14:00:00Z")),
    ));
    assert_eq!(within.record, minimal_record);
    assert!(minimal_record.summarized.is_empty());
}

/// No credential material anywhere (Wave-2 addendum §3): the w3 fixtures
/// carry references only — secret-class records name secret-store
/// locations, never material.
#[test]
fn no_credential_material_in_w3_fixtures() {
    let fixtures_root = fixture_path("");
    let mut stack = vec![fixtures_root];
    while let Some(directory) = stack.pop() {
        for entry in test_ok(fs::read_dir(&directory)) {
            let entry = test_ok(entry);
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let content = test_ok(fs::read_to_string(&path));
                assert_credential_free(&content);
            }
        }
    }
}

fn assert_credential_free(serialized: &str) {
    for marker in [
        "sk-",
        "Bearer ",
        "api_key",
        "apikey",
        "password",
        "passwd",
        "client_secret",
        "access_token",
        "refresh_token",
        "PRIVATE KEY",
        "BEGIN RSA",
        "xoxb-",
        "ghp_",
    ] {
        assert!(
            !serialized.contains(marker),
            "fixtures must never contain credential material ({marker:?})"
        );
    }
}

/// Canonical JSON carries no floats (kernel §4), and every serialized
/// contract document carries the `"v": 1` schema marker.
#[test]
fn w3_serialized_contract_state_contains_no_floats() {
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
    let documents = [
        "durable-state/typical.json",
        "durable-state/minimal.json",
        "tool-definitions/typical.json",
        "tool-definitions/minimal.json",
        "capability-admissions/typical.json",
        "capability-admissions/minimal.json",
        "tool-exposure/typical.json",
        "tool-exposure/minimal.json",
        "compaction-record/typical.json",
        "compaction-record/minimal.json",
    ];
    for relative in documents {
        let value: serde_json::Value = test_ok(serde_json::from_str(&read_fixture(relative)));
        assert_no_floats(&value, relative);
        if value.is_object() {
            assert_eq!(
                value["v"],
                serde_json::json!(1),
                "{relative} carries \"v\": 1"
            );
        }
    }
}
