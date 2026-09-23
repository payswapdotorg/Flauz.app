//! Kernel conformance tests (F2 Wave-3 addendum §5-§6, the execution
//! graph): canonical JSON, the fixture set, the evaluator-output law,
//! the no-chat-relay law, and the no-credential-material rule.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use flauz_orch::ExecutionGraph;
use flauz_orch::evaluator::{GraphEvaluation, GraphEventSequence, event_types};
use flauz_orch::node::AgentAssignment;

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
    let assignment_fixtures = [
        "agent-assignment/typical.json",
        "agent-assignment/minimal.json",
    ];
    for relative in assignment_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<AgentAssignment>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let graph_fixtures = [
        "execution-graph/typical.json",
        "execution-graph/minimal.json",
    ];
    for relative in graph_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<ExecutionGraph>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
        let graph: ExecutionGraph = test_ok(serde_json::from_str(&content));
        test_ok(graph.validate());
    }
    let sequence_fixtures = [
        "graph-event-sequence/typical.json",
        "graph-event-sequence/minimal.json",
    ];
    for relative in sequence_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<GraphEventSequence>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
    let evaluation_fixtures = [
        "graph-evaluation/typical.json",
        "graph-evaluation/blocked.json",
    ];
    for relative in evaluation_fixtures {
        let content = read_fixture(relative);
        let serialized = round_trip::<GraphEvaluation>(&content);
        assert_eq!(
            serialized, content,
            "fixture {relative} is not canonical: re-serialization differs"
        );
    }
}

/// Invalid fixtures fail strict parses or validations — a record that
/// violates the canonical grammar, the graph laws, or the independence
/// rule is rejected on read or on validation, never silently misread
/// (addendum §5's invalid-per-family rule).
#[test]
fn invalid_fixtures_fail_strict_parses_or_validations() {
    // Unknown fields are rejected on read.
    let unknown_field = read_fixture("agent-assignment/invalid-unknown-field.json");
    assert!(
        serde_json::from_str::<AgentAssignment>(&unknown_field).is_err(),
        "unknown fields must be rejected"
    );
    let evaluation_unknown = read_fixture("graph-evaluation/invalid-unknown-field.json");
    assert!(
        serde_json::from_str::<GraphEvaluation>(&evaluation_unknown).is_err(),
        "unknown fields must be rejected"
    );

    // A reference outside the frozen canonical-ID grammar is rejected on
    // read.
    let bad_grammar = read_fixture("agent-assignment/invalid-grammar.json");
    assert!(
        serde_json::from_str::<AgentAssignment>(&bad_grammar).is_err(),
        "references outside the frozen grammar must be rejected on read"
    );

    // A context-snapshot wait is rejected on read: the wait grammar has
    // exactly the two shared-state kinds — the relay abstraction does
    // not exist (addendum §4's no-chat-relay law, made structural).
    let snapshot = read_fixture("agent-assignment/invalid-context-snapshot-wait.json");
    assert!(
        serde_json::from_str::<AgentAssignment>(&snapshot).is_err(),
        "a context-snapshot wait must fail the canonical read — the grammar has no such kind"
    );

    // A dependency cycle parses but fails validation.
    let cycle = read_fixture("execution-graph/invalid-cycle.json");
    let cyclic: ExecutionGraph = test_ok(serde_json::from_str(&cycle));
    assert!(
        cyclic.validate().is_err(),
        "dependency cycles must fail validation"
    );

    // A verifier without artifact inputs is the context-snapshot shape:
    // it parses but fails validation — the independence rule is
    // structural (addendum §4).
    let relay = read_fixture("execution-graph/invalid-non-independent-verifier.json");
    let non_independent: ExecutionGraph = test_ok(serde_json::from_str(&relay));
    let error = non_independent.validate().err().unwrap_or_else(|| {
        panic!("a verifier whose inputs are not the artifacts it verifies is invalid")
    });
    assert!(
        error.reason().contains("never a context snapshot"),
        "the rejection names the independence rule: {error}"
    );

    // An event sequence whose events are illegal against the graph state
    // fails evaluation — producing while blocked is never silently
    // accepted.
    let graph = flauz_orch::fakes::fake_research_analysis_review_graph();
    let illegal = read_fixture("graph-event-sequence/invalid-illegal-transition.json");
    let sequence: GraphEventSequence = test_ok(serde_json::from_str(&illegal));
    assert!(
        graph.advance(&sequence).is_err(),
        "producing while blocked must fail evaluation"
    );
}

/// The evaluator-output law: every committed evaluation fixture EQUALS
/// the evaluator run over the committed graph + sequence fixtures (or
/// the documented fake input family) it documents — the fixtures are
/// honest artifacts of the evaluator, never hand-staged shapes.
#[test]
fn evaluation_fixtures_are_the_evaluator_output_over_their_inputs() {
    // The typical evaluation: the fake graph over the committed happy
    // path.
    let graph: ExecutionGraph = test_ok(serde_json::from_str(&read_fixture(
        "execution-graph/typical.json",
    )));
    let sequence: GraphEventSequence = test_ok(serde_json::from_str(&read_fixture(
        "graph-event-sequence/typical.json",
    )));
    let expected: GraphEvaluation = test_ok(serde_json::from_str(&read_fixture(
        "graph-evaluation/typical.json",
    )));
    let evaluation = test_ok(graph.advance(&sequence));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&evaluation)),
        test_ok(serde_json::to_string_pretty(&expected)),
        "the typical evaluation fixture must equal advance(graph, events)"
    );

    // The blocked evaluation: the same graph over the documented blocked
    // prefix (the fake family's mid-run shape).
    let (fake_graph, blocked_prefix) = flauz_orch::fakes::fake_blocked_resolution_run();
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&fake_graph)),
        test_ok(serde_json::to_string_pretty(&graph)),
        "the fake graph family and the committed graph fixture are identical"
    );
    let expected_blocked: GraphEvaluation = test_ok(serde_json::from_str(&read_fixture(
        "graph-evaluation/blocked.json",
    )));
    let blocked = test_ok(graph.advance(&blocked_prefix));
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&blocked)),
        test_ok(serde_json::to_string_pretty(&expected_blocked)),
        "the blocked evaluation fixture must equal advance(graph, blocked prefix)"
    );
}

/// The committed fixtures equal the public fakes exactly: the fake
/// surface is the reference semantics for real graph inputs (kernel §7
/// determinism — the fakes are stable across runs).
#[test]
fn the_public_fakes_reproduce_the_committed_fixtures() {
    let (graph, events) = flauz_orch::fakes::fake_research_analysis_review_run();
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&graph)),
        read_fixture("execution-graph/typical.json")
    );
    assert_eq!(
        test_ok(serde_json::to_string_pretty(&events)),
        read_fixture("graph-event-sequence/typical.json")
    );
}

/// No credential material anywhere: the serialized state of every
/// fixture and every fake output contains no secret-looking values
/// (addendum §6). Credentials are references, and the graph carries
/// none at all.
#[test]
fn no_credential_material_in_serialized_state() {
    let mut documents = Vec::new();
    for family in [
        "agent-assignment",
        "execution-graph",
        "graph-event-sequence",
        "graph-evaluation",
    ] {
        let directory = fixture_path(family);
        let entries = fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("could not list fixture family {family}: {error}"));
        for entry in entries {
            let path = entry
                .unwrap_or_else(|error| panic!("could not read fixture entry: {error}"))
                .path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default();
                if name.starts_with("invalid-") {
                    continue;
                }
                documents.push((
                    format!("{family}/{name}"),
                    fs::read_to_string(&path).unwrap_or_else(|error| {
                        panic!("could not read {}: {error}", path.display())
                    }),
                ));
            }
        }
    }
    let (graph, events) = flauz_orch::fakes::fake_research_analysis_review_run();
    documents.push((
        "fake-graph".to_owned(),
        test_ok(serde_json::to_string_pretty(&graph)),
    ));
    documents.push((
        "fake-events".to_owned(),
        test_ok(serde_json::to_string_pretty(&events)),
    ));
    let evaluation = test_ok(graph.advance(&events));
    documents.push((
        "fake-evaluation".to_owned(),
        test_ok(serde_json::to_string_pretty(&evaluation)),
    ));

    assert!(!documents.is_empty());
    for (name, document) in documents {
        let lowered = document.to_lowercase();
        for marker in [
            "flausec_", "api_key", "password", "secret", "bearer ", "token=",
        ] {
            assert!(
                !lowered.contains(marker),
                "serialized state {name} must never contain credential material ({marker})"
            );
        }
    }
}

/// The canonical determinism law: the same (graph, events) inputs
/// produce byte-identical serialized evaluations — proven over the
/// committed fixtures (the fakes replay identically).
#[test]
fn determinism_over_the_committed_fixtures() {
    let graph: ExecutionGraph = test_ok(serde_json::from_str(&read_fixture(
        "execution-graph/typical.json",
    )));
    let sequence: GraphEventSequence = test_ok(serde_json::from_str(&read_fixture(
        "graph-event-sequence/typical.json",
    )));
    let first = test_ok(serde_json::to_string(&test_ok(graph.advance(&sequence))));
    let second = test_ok(serde_json::to_string(&test_ok(graph.advance(&sequence))));
    assert_eq!(first, second);
}

/// The registered event vocabulary: the merge point's event type is
/// `task.verification_merged`, following the kernel's
/// `<entity>.<verb_past>` grammar, and every serialized evaluation that
/// merges carries exactly it.
#[test]
fn the_merge_event_type_is_registered_canonically() {
    assert_eq!(
        event_types::TASK_VERIFICATION_MERGED,
        "task.verification_merged"
    );
    let evaluation: GraphEvaluation = test_ok(serde_json::from_str(&read_fixture(
        "graph-evaluation/typical.json",
    )));
    let merge = evaluation
        .merge
        .as_ref()
        .unwrap_or_else(|| panic!("the typical evaluation merges"));
    assert_eq!(merge.event_type, event_types::TASK_VERIFICATION_MERGED);
    let blocked: GraphEvaluation = test_ok(serde_json::from_str(&read_fixture(
        "graph-evaluation/blocked.json",
    )));
    assert!(
        blocked.merge.is_none(),
        "the blocked evaluation has not merged yet"
    );
}

/// The version-marker law: documents with a schema version other than
/// 1 fail the canonical read (kernel §4).
#[test]
fn future_schema_versions_fail_canonical_reads() {
    let content = read_fixture("execution-graph/minimal.json");
    let bumped = content.replace("\"v\": 1", "\"v\": 2");
    let error = serde_json::from_str::<ExecutionGraph>(&bumped);
    assert!(
        error.is_err(),
        "a v2 graph document must fail the canonical read, never silently misread"
    );
    let description = error.err().unwrap_or_else(|| panic!("checked")).to_string();
    assert!(
        description.contains("schema version"),
        "the failure names the schema version: {description}"
    );
}
