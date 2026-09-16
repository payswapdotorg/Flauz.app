//! Universal Workflow client boundary integration tests (GUI-001).
//!
//! These tests prove the typed workflow client boundary against a real
//! supervised Codex app-server process: the same boundary the GUI will
//! use. They are gated on the `CODEX_RS_TEST_CODEX_BIN` environment
//! variable pointing at a `codex` executable whose `app-server`
//! subcommand serves the experimental `workflow/*` method family
//! (release `rust-v0.1.0` of `payswapdotorg/codex` or newer built from
//! `main`).
//!
//! Every run uses an isolated throwaway `CODEX_HOME`; the live
//! `~/.codex` is never touched.
//!
//! ```text
//! CODEX_RS_TEST_CODEX_BIN=/path/to/codex cargo test -p codex-platform --test workflow_boundary
//! ```

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

use codex_platform::{AppServerConfig, AppServerConnection, CodexHome};
use codex_protocol::{
    ClientInfo, InitializeCapabilities, WorkflowApprovalDecision, WorkflowDemonstrationKind,
    WorkflowImproveProposeParams, WorkflowInstanceGetParams, WorkflowInstanceRunParams,
    WorkflowTeachDemonstrateParams, WorkflowTeachEvidenceInput, WorkflowTeachInstructParams,
    WorkflowTeachMode, WorkflowTeachReconcileParams, WorkflowTeachSessionStatus,
    WorkflowTeachStartParams,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn test_codex_binary() -> Option<PathBuf> {
    let value = env::var_os("CODEX_RS_TEST_CODEX_BIN")?;
    if value.is_empty() {
        return None;
    }
    let path = PathBuf::from(value);
    if path.is_file() {
        Some(path)
    } else {
        panic!(
            "CODEX_RS_TEST_CODEX_BIN points at a missing file; \
             set it to a codex executable that serves the workflow app-server family"
        );
    }
}

fn isolated_codex_home(label: &str) -> CodexHome {
    let path = env::temp_dir().join(format!(
        "codex-platform-workflow-boundary-{label}-{}-{}",
        process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|error| panic!("system clock after epoch: {error}"))
            .as_nanos()
    ));
    fs::create_dir_all(&path)
        .unwrap_or_else(|error| panic!("could not create isolated CODEX_HOME fixture: {error}"));
    CodexHome::resolve(Some(path))
        .unwrap_or_else(|error| panic!("could not resolve isolated CODEX_HOME: {error}"))
}

fn spawn_connection(codex_binary: &Path, codex_home: &CodexHome) -> AppServerConnection {
    let mut config = AppServerConfig::new(codex_binary.to_path_buf(), codex_home.clone());
    config.request_timeout = REQUEST_TIMEOUT;
    let connection = AppServerConnection::spawn(config)
        .unwrap_or_else(|error| panic!("could not spawn supervised app-server: {error}"));
    connection
        .initialize_with_capabilities(
            ClientInfo {
                name: "codexRS-workflow-boundary-test".to_owned(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            Some(InitializeCapabilities {
                experimental_api: true,
                request_attestation: false,
                mcp_server_openai_form_elicitation: None,
                opt_out_notification_methods: None,
            }),
        )
        .unwrap_or_else(|error| panic!("could not initialize app-server: {error}"));
    connection
}

#[test]
fn workflow_lifecycle_round_trips_through_the_supervised_boundary() -> TestResult {
    let Some(codex_binary) = test_codex_binary() else {
        eprintln!("skipping: CODEX_RS_TEST_CODEX_BIN is not set");
        return Ok(());
    };
    let codex_home = isolated_codex_home("lifecycle");
    let mut connection = spawn_connection(&codex_binary, &codex_home);

    // Teach: open a hybrid-mode session under an explicit name.
    let started = connection.workflow_teach_start(WorkflowTeachStartParams {
        mode: WorkflowTeachMode::Hybrid,
        name: Some("gui-boundary-round-trip".to_owned()),
    })?;
    assert_eq!(started.name, "gui-boundary-round-trip");
    assert_eq!(started.mode, WorkflowTeachMode::Hybrid);
    assert_eq!(started.status, WorkflowTeachSessionStatus::Open);

    // Teach: record one instruction and one demonstration event with an
    // evidence reference, proving both record paths stay typed.
    let evidence = WorkflowTeachEvidenceInput {
        label: "boundary-probe".to_owned(),
        locator: "memory://boundary-probe".to_owned(),
        sha256: "b".repeat(64),
    };
    let instructed = connection.workflow_teach_instruct(WorkflowTeachInstructParams {
        session_id: started.session_id.clone(),
        text: "Open the release notes and summarize the newest entry.".to_owned(),
        evidence: vec![evidence],
    })?;
    assert_eq!(instructed.session_id, started.session_id);
    assert_eq!(instructed.sequence, 0, "sequence numbers are zero-based");

    let demonstrated = connection.workflow_teach_demonstrate(WorkflowTeachDemonstrateParams {
        session_id: started.session_id.clone(),
        kind: WorkflowDemonstrationKind::Action,
        text: "Clicked the newest release heading.".to_owned(),
        evidence: Vec::new(),
    })?;
    assert_eq!(demonstrated.record_count, 2);

    // Reconcile freezes the trajectory for compilation.
    let reconciled = connection.workflow_teach_reconcile(WorkflowTeachReconcileParams {
        session_id: started.session_id.clone(),
    })?;
    assert_eq!(reconciled.instruction_records, 1);
    assert_eq!(reconciled.demonstration_records, 1);

    // Compile produces a reviewable candidate.
    let compiled = connection.workflow_compile(codex_protocol::WorkflowCompileParams {
        session_id: started.session_id.clone(),
    })?;
    assert!(compiled.step_count >= 1, "compiled candidate has steps");

    // Review presents the compiled steps for the approver.
    let reviewed = connection.workflow_review(codex_protocol::WorkflowReviewParams {
        candidate_id: compiled.candidate_id.clone(),
    })?;
    assert_eq!(reviewed.candidate_id, compiled.candidate_id);
    assert_eq!(reviewed.steps.len() as u64, compiled.step_count);

    // Approve, then publish an immutable version pinned at a commit.
    let approved = connection.workflow_approve(codex_protocol::WorkflowApproveParams {
        candidate_id: compiled.candidate_id.clone(),
        approver: "boundary-test".to_owned(),
        reference: "gui-001-client-boundary".to_owned(),
        decision: WorkflowApprovalDecision::Approved,
    })?;

    let published = connection.workflow_publish(codex_protocol::WorkflowPublishParams {
        candidate_id: approved.candidate_id.clone(),
        repository: Some("https://github.com/payswapdotorg/Flauz.app".to_owned()),
        commit_sha: "c".repeat(40),
        semantic_version: Some("1.0.0".to_owned()),
    })?;
    assert_eq!(published.workflow, "gui-boundary-round-trip");
    assert_eq!(published.semantic_version, "1.0.0");
    assert!(!published.version_id.is_empty());

    // Run the published version as a durable instance.
    let run = connection.workflow_instance_run(WorkflowInstanceRunParams {
        version_id: published.version_id.clone(),
    })?;
    assert_eq!(run.workflow, published.workflow);
    assert_eq!(run.version_id, published.version_id);
    assert!(!run.instance_id.is_empty());

    // The durable instance list and per-instance read agree.
    let listed = connection.workflow_instance_list()?;
    let record = listed
        .instances
        .iter()
        .find(|instance| instance.instance_id == run.instance_id)
        .unwrap_or_else(|| panic!("run instance {} present in instance list", run.instance_id));
    assert_eq!(record.version_id, published.version_id);

    let fetched = connection.workflow_instance_get(WorkflowInstanceGetParams {
        instance_id: run.instance_id.clone(),
    })?;
    assert_eq!(fetched.instance.instance_id, run.instance_id);

    // Improvement proposals derive from recorded execution evidence.
    let proposal = connection.workflow_improve_propose(WorkflowImproveProposeParams {
        version_id: published.version_id.clone(),
    })?;
    assert_eq!(proposal.incumbent_version_id, published.version_id);
    assert!(
        proposal
            .evidence
            .runs
            .iter()
            .any(|run_record| run_record.version_id == published.version_id),
        "improvement proposal cites the recorded run"
    );

    // Unknown session ids surface as protocol errors, never as silent
    // success: the control plane owns session authority.
    let unknown_session = connection.workflow_teach_reconcile(WorkflowTeachReconcileParams {
        session_id: "ws-does-not-exist".to_owned(),
    });
    assert!(unknown_session.is_err());

    connection.shutdown()?;
    drop(codex_home);
    Ok(())
}

#[test]
fn workflow_durable_instances_survive_connection_restart() -> TestResult {
    let Some(codex_binary) = test_codex_binary() else {
        eprintln!("skipping: CODEX_RS_TEST_CODEX_BIN is not set");
        return Ok(());
    };
    let codex_home = isolated_codex_home("restart");

    // First connection: teach, publish, and run once.
    let mut first = spawn_connection(&codex_binary, &codex_home);
    let started = first.workflow_teach_start(WorkflowTeachStartParams {
        mode: WorkflowTeachMode::Instruct,
        name: Some("gui-boundary-restart".to_owned()),
    })?;
    first.workflow_teach_instruct(WorkflowTeachInstructParams {
        session_id: started.session_id.clone(),
        text: "Write the current date into a scratch note.".to_owned(),
        evidence: Vec::new(),
    })?;
    first.workflow_teach_reconcile(WorkflowTeachReconcileParams {
        session_id: started.session_id.clone(),
    })?;
    let compiled = first.workflow_compile(codex_protocol::WorkflowCompileParams {
        session_id: started.session_id.clone(),
    })?;
    first.workflow_approve(codex_protocol::WorkflowApproveParams {
        candidate_id: compiled.candidate_id.clone(),
        approver: "boundary-test".to_owned(),
        reference: "gui-001-restart".to_owned(),
        decision: WorkflowApprovalDecision::Approved,
    })?;
    let published = first.workflow_publish(codex_protocol::WorkflowPublishParams {
        candidate_id: compiled.candidate_id.clone(),
        repository: None,
        commit_sha: "d".repeat(40),
        semantic_version: None,
    })?;
    let run = first.workflow_instance_run(WorkflowInstanceRunParams {
        version_id: published.version_id.clone(),
    })?;
    first.shutdown()?;

    // Second connection against the same CODEX_HOME: the control plane
    // still serves the durable instance — restart never moves durable
    // workflow authority into GUI state.
    let mut second = spawn_connection(&codex_binary, &codex_home);
    let listed = second.workflow_instance_list()?;
    assert!(
        listed
            .instances
            .iter()
            .any(|instance| instance.instance_id == run.instance_id),
        "durable instance survives app-server restart"
    );
    let fetched = second.workflow_instance_get(WorkflowInstanceGetParams {
        instance_id: run.instance_id.clone(),
    })?;
    assert_eq!(fetched.instance.version_id, published.version_id);
    second.shutdown()?;
    drop(codex_home);
    Ok(())
}
