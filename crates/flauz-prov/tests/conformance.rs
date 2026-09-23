//! Kernel conformance tests (Wave-4 addendum §3/§5 + kernel §4, work
//! order PROV-001): canonical JSON, the fixture set, the
//! scheduler-output law (the fixtures are honest artifacts of the
//! scheduler run over the fakes, never hand-staged shapes), and the
//! credential-material scan extended to every new record family.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_prov::account::{AccountStore, ProviderAccount};
use flauz_prov::fakes::{FakeRoutingScenario, FAKE_TASK};
use flauz_prov::key::CREDENTIAL_MARKERS;
use flauz_prov::ledger::{DepletionProjection, QuotaLedger};
use flauz_prov::policy::RoutingPolicy;
use flauz_prov::scheduler::SchedulingChoice;
use flauz_prov::{TierKind, project_depletion};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("could not read fixture value: {error}"),
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

/// The fake scenario the choice fixtures must equal (deterministic:
/// fixed references, fixed windows, the default free-tier-first policy).
fn connected_scenario() -> FakeRoutingScenario {
    let mut scenario = test_ok(FakeRoutingScenario::new(
        "openai",
        test_ok(flauz_prov::Timestamp::parse("2026-09-23T08:00:00Z")),
    ));
    test_ok(scenario.connect_account(
        TierKind::Free,
        "practice-key-1",
        "Personal account",
    ));
    test_ok(scenario.connect_account(
        TierKind::Paid,
        "practice-key-2",
        "Work account",
    ));
    scenario
}

/// Every valid conformance fixture is canonical JSON: it parses strictly
/// (unknown fields rejected) and re-serializes to exactly the committed
/// bytes (kernel §4 — the round-trip law).
#[test]
fn fixtures_roundtrip_canonical() {
    let account_fixtures = [
        "provider-account/typical.json",
        "provider-account/minimal.json",
    ];
    for relative in account_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<ProviderAccount>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let ledger_fixtures = ["quota-ledger/typical.json", "quota-ledger/minimal.json"];
    for relative in ledger_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<QuotaLedger>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let policy_fixtures = ["routing-policy/typical.json", "routing-policy/minimal.json"];
    for relative in policy_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<RoutingPolicy>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let choice_fixtures = [
        "scheduling-choice/typical.json",
        "scheduling-choice/minimal.json",
    ];
    for relative in choice_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<SchedulingChoice>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let projection_fixtures = [
        "depletion-projection/typical.json",
        "depletion-projection/boundary.json",
    ];
    for relative in projection_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<DepletionProjection>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
}

/// Invalid fixtures fail strict parses or canonical validation — never a
/// silent misread: a credential-material secret reference is rejected on
/// read; a broken window, a non-contiguous usage sequence, a zero spend
/// limit, a SILENT escalation (the attribution-law fixture), an ambient
/// unlabeled account, and a hand-staged inconsistent projection all fail
/// validation after parsing.
#[test]
fn invalid_fixtures_fail_strict_parses() {
    // Credential material in the reference: the strict parse itself
    // fails (addendum §3 — the rejection is the test).
    let material = read_fixture("provider-account/invalid-secret-ref.json");
    assert!(
        serde_json::from_str::<ProviderAccount>(&material).is_err(),
        "a credential-material secret reference must be rejected on read"
    );
    // A window that does not open before it closes.
    let window = read_fixture("provider-account/invalid-window.json");
    let account: ProviderAccount = test_ok(serde_json::from_str(&window));
    assert!(
        account.validate().is_err(),
        "a quota window must start strictly before it ends"
    );
    // A usage stream that is not contiguous from sequence 1.
    let seq = read_fixture("quota-ledger/invalid-seq.json");
    let ledger: QuotaLedger = test_ok(serde_json::from_str(&seq));
    assert!(
        ledger.validate().is_err(),
        "usage records must be contiguous from sequence 1"
    );
    // A zero spend limit is not a limit.
    let limit = read_fixture("routing-policy/invalid-limit.json");
    let policy: RoutingPolicy = test_ok(serde_json::from_str(&limit));
    assert!(
        policy.validate().is_err(),
        "a zero spend limit must be rejected"
    );
    // THE attribution-law fixture: an escalation with no named free
    // alternative is the silent fall-through the kernel forbids.
    let silent = read_fixture("scheduling-choice/invalid-silent-escalation.json");
    let choice: SchedulingChoice = test_ok(serde_json::from_str(&silent));
    assert!(
        choice.validate().is_err(),
        "an escalation must name the free alternative it left behind"
    );
    // Ambient attribution: an unlabeled account cannot be attributed.
    let ambient = read_fixture("scheduling-choice/invalid-ambient-account.json");
    let choice: SchedulingChoice = test_ok(serde_json::from_str(&ambient));
    assert!(
        choice.validate().is_err(),
        "a choice without a named account is ambient attribution — refused"
    );
    // A hand-staged inconsistent projection fails its own law.
    let inconsistent = read_fixture("depletion-projection/invalid-inconsistent.json");
    let projection: DepletionProjection = test_ok(serde_json::from_str(&inconsistent));
    assert!(
        projection.validate().is_err(),
        "the projected remaining, depleted flag and deficit must be consistent"
    );
}

/// The scheduler-output law (the CAP-001 resolver-output pattern): every
/// committed choice fixture EQUALS the scheduler run over the committed
/// fake scenario — the fixtures are honest artifacts of the scheduler,
/// never hand-staged shapes.
#[test]
fn choice_fixtures_are_the_scheduler_output_over_the_fakes() {
    let noon = test_ok(flauz_prov::Timestamp::parse("2026-09-23T12:00:00Z"));

    // The typical fixture: free-tier-first over the connected pair.
    let scenario = connected_scenario();
    let choice = test_ok(scenario.schedule(noon));
    let expected: SchedulingChoice =
        test_ok(serde_json::from_str(&read_fixture("scheduling-choice/typical.json")));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&choice)),
        test_ok(serde_json::to_string_pretty(&expected)),
        "scheduling-choice/typical.json must equal the scheduler output over the fakes"
    );

    // The minimal fixture: the paid account alone is the only available
    // one — no escalation (nothing was depleted).
    let mut paid_only = test_ok(FakeRoutingScenario::new(
        "openai",
        test_ok(flauz_prov::Timestamp::parse("2026-09-23T08:00:00Z")),
    ));
    test_ok(paid_only.connect_account(
        TierKind::Paid,
        "practice-key-2",
        "Work account",
    ));
    let choice = test_ok(paid_only.schedule(noon));
    let expected: SchedulingChoice =
        test_ok(serde_json::from_str(&read_fixture("scheduling-choice/minimal.json")));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&choice)),
        test_ok(serde_json::to_string_pretty(&expected)),
        "scheduling-choice/minimal.json must equal the scheduler output over the fakes"
    );
}

/// The public fakes reproduce the committed account fixture exactly: the
/// fake surface is the reference semantics for real account state (the
/// connect flow's account, byte for byte).
#[test]
fn the_public_fakes_match_the_committed_account_fixture() {
    let scenario = connected_scenario();
    let accounts = scenario.accounts();
    assert_eq!(accounts.len(), 2);
    let expected: ProviderAccount =
        test_ok(serde_json::from_str(&read_fixture("provider-account/typical.json")));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&accounts[0])),
        test_ok(serde_json::to_string_pretty(&expected)),
        "the fake free account must match provider-account/typical.json byte for byte"
    );
    // The depletion-projection fixtures are the pure function's outputs.
    let window = expected.quota;
    let typical: DepletionProjection =
        test_ok(serde_json::from_str(&read_fixture("depletion-projection/typical.json")));
    assert_eq!(project_depletion(&window, 3), typical);
    let boundary: DepletionProjection =
        test_ok(serde_json::from_str(&read_fixture("depletion-projection/boundary.json")));
    let over_window = flauz_prov::account::QuotaWindow {
        remaining: 3,
        ..window
    };
    assert_eq!(project_depletion(&over_window, 5), boundary);
}

/// Canonical JSON carries no floats (kernel §4) and every contract
/// fixture carries `"v": 1`.
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
        "provider-account/typical.json",
        "provider-account/minimal.json",
        "quota-ledger/typical.json",
        "quota-ledger/minimal.json",
        "routing-policy/typical.json",
        "routing-policy/minimal.json",
        "scheduling-choice/typical.json",
        "scheduling-choice/minimal.json",
        "depletion-projection/typical.json",
        "depletion-projection/boundary.json",
    ];
    for relative in all_fixtures {
        let value: serde_json::Value = test_ok(serde_json::from_str(&read_fixture(relative)));
        assert_no_floats(&value, relative);
        // The schema version marker is exactly 1.
        assert_eq!(value["v"], serde_json::json!(1), "{relative} carries \"v\": 1");
    }
}

/// No credential material anywhere (addendum §3, the scan EXTENDED to
/// the new families): every committed fixture, the serialized state of
/// every record family built through the fakes, and the fake secret
/// store's entire state stay free of the marker family — while the
/// references (`flausec_...`) remain visible as the storage form.
#[test]
fn no_credential_material_in_serialized_state() {
    // Every committed fixture (the whole w4 tree, recursively — the
    // registry's scan pattern).
    let mut stack = vec![fixture_path("")];
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("could not list {directory:?}: {error}"))
        {
            let entry = test_ok(entry);
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "json") {
                let content =
                    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
                for marker in CREDENTIAL_MARKERS {
                    assert!(
                        !content.contains(marker),
                        "fixture {path:?} must never contain credential material ({marker:?})"
                    );
                }
            }
        }
    }

    // The serialized state of every new family, built through the fakes.
    let mut scenario = connected_scenario();
    let noon = test_ok(flauz_prov::Timestamp::parse("2026-09-23T12:00:00Z"));
    let choice = test_ok(scenario.schedule(noon));
    test_ok(scenario.consume(&choice, "one model run", 1, noon));
    let mut store = AccountStore::new();
    test_ok(store.connect_account(scenario.accounts()[0].clone()));
    let mut policy = RoutingPolicy::new();
    test_ok(policy.set_spend_limits(Some(100), Some(500)));

    let states = [
        test_ok(serde_json::to_string(&scenario.accounts())),
        test_ok(serde_json::to_string(&scenario.ledger())),
        test_ok(serde_json::to_string(&scenario.store())),
        test_ok(serde_json::to_string(&store.snapshot())),
        test_ok(serde_json::to_string(&policy)),
        test_ok(serde_json::to_string(&choice)),
    ];
    for state in &states {
        for marker in CREDENTIAL_MARKERS {
            assert!(
                !state.contains(marker),
                "serialized provider state must never contain credential material ({marker:?})"
            );
        }
        // The material never survives the seam: nothing the scenario
        // exposes contains the practice key that was entered.
        assert!(!state.contains("practice-key"));
    }
    // The connected families store opaque references — the reference
    // form, never material (checked where references belong: accounts,
    // the secret store, the account store's snapshot).
    for reference_carrying in [
        test_ok(serde_json::to_string(&scenario.accounts())),
        test_ok(serde_json::to_string(&scenario.store())),
        test_ok(serde_json::to_string(&store.snapshot())),
    ] {
        assert!(
            reference_carrying.contains("flausec_"),
            "the connected state stores opaque references"
        );
    }
    // The attribution record names the task (FAKE_TASK is the task the
    // scenario routes for).
    assert!(test_ok(serde_json::to_string(&choice)).contains(FAKE_TASK));
}
