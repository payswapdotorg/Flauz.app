//! The comparator law tests (LAB-001, Wave-4 kernel addendum §4): the
//! reference-vs-candidate diff through the normalized schema.
//!
//! - identical runs report ZERO divergences (the comparator's empty
//!   state — the zero-divergence report);
//! - a seeded divergence comes out NAMED (a canonical kind), SEVERE
//!   (major/missing) and LOCATED (step + probe);
//! - provider-local fields (frame digests, read references) never
//!   diverge — the raw-screenshots-alone rule made structural;
//! - determinism: identical inputs produce byte-identical reports.

use flauz_lab::evidence::ProbeResult;
use flauz_lab::refs::{FrameDigest, ProbeId, RunId, StepId};
use flauz_lab::{
    ComparatorReport, DivergenceDetail, EvidenceRecord, FakeRemoteLabAdapter, LocalLabAdapter,
    Severity, Side, StepComparison, StepEvidence, compare, run_journey,
};

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

/// The reference pair the seeded-divergence tests mutate: the local run
/// (reference) and the fake-remote run (candidate) of the
/// shell-discovery journey.
fn reference_pair() -> (flauz_lab::JourneySpec, EvidenceRecord, EvidenceRecord) {
    let journey = flauz_lab::fakes::fake_shell_discovery_journey();
    let mut local = LocalLabAdapter::new();
    let mut remote = FakeRemoteLabAdapter::new();
    let reference = ok(run_journey(&mut local, &journey));
    let candidate = ok(run_journey(&mut remote, &journey));
    (journey, reference, candidate)
}

/// Identical runs report zero divergences — the comparator's empty
/// state (the work order's acceptance criterion 3, second half).
#[test]
fn identical_runs_report_zero_divergences() {
    let journey = flauz_lab::fakes::fake_shell_discovery_journey();
    let mut first_adapter = LocalLabAdapter::new();
    let mut second_adapter = LocalLabAdapter::new();
    let first = ok(run_journey(&mut first_adapter, &journey));
    let second = ok(run_journey(&mut second_adapter, &journey));
    let report = ok(compare(&journey, &first, &second));
    assert!(report.is_clean());
    assert_eq!(report.divergences.len(), 0);
    assert_eq!(report.matching, journey.steps.len() as u32);
    assert_eq!(report.divergent, 0);
    assert_eq!(report.missing, 0);
    ok(report.validate());
    // Every step comparison is Matching.
    for comparison in &report.steps {
        assert!(matches!(comparison, StepComparison::Matching { .. }));
    }
}

/// The zero-divergence report between DIFFERENT providers (local vs the
/// fake remote) is the F8 artifact the wave gate archives: the report is
/// clean, byte-stable, and round-trips canonically.
#[test]
fn the_cross_provider_report_is_clean_and_canonical() {
    let (journey, reference, candidate) = reference_pair();
    let report = ok(compare(&journey, &reference, &candidate));
    assert!(report.is_clean());
    assert_eq!(report.reference_provider, "flauz-lab-local");
    assert_eq!(report.candidate_provider, "flauz-lab-fake-remote");
    let serialized = ok(serde_json::to_string_pretty(&report));
    let reloaded: ComparatorReport = ok(serde_json::from_str(&serialized));
    assert_eq!(reloaded, report);
    ok(report.validate());
}

/// A seeded divergence — flipping one frame's normalized anchor
/// observation on the candidate — comes out NAMED (the canonical
/// `frame_anchors` kind), SEVERE (major) and LOCATED (the exact step
/// and probe), and nothing else diverges.
#[test]
fn a_seeded_frame_divergence_is_named_severe_and_located() {
    let (journey, reference, candidate) = reference_pair();
    let step_id = ok(StepId::parse("open-picker"));
    let probe_id = ok(ProbeId::parse("frame-picker-open"));

    // Seed: on the candidate, the picker panel is observed closed at the
    // open-picker moment — the panel failed to open on that provider.
    let seeded = mutate_frame_anchor(
        &candidate,
        &step_id,
        &probe_id,
        "model-picker-panel",
        "closed",
    );

    let report = ok(compare(&journey, &reference, &seeded));
    assert_eq!(report.divergences.len(), 1, "exactly one divergence");
    let divergence = &report.divergences[0];
    assert_eq!(divergence.step, step_id, "located at the step");
    assert_eq!(
        divergence.probe.as_ref(),
        Some(&probe_id),
        "located at the probe"
    );
    assert_eq!(divergence.severity, Severity::Major, "severe");
    match &divergence.detail {
        DivergenceDetail::FrameAnchors {
            moment,
            reference: reference_anchors,
            candidate: candidate_anchors,
        } => {
            assert_eq!(moment.as_str(), "picker-open");
            let picker_reference = reference_anchors
                .iter()
                .find(|observation| observation.anchor.as_str() == "model-picker-panel")
                .unwrap_or_else(|| panic!("the reference frame observes the picker anchor"));
            let picker_candidate = candidate_anchors
                .iter()
                .find(|observation| observation.anchor.as_str() == "model-picker-panel")
                .unwrap_or_else(|| panic!("the candidate frame observes the picker anchor"));
            assert_eq!(picker_reference.state.as_str(), "open-empty");
            assert_eq!(picker_candidate.state.as_str(), "closed");
        }
        other => panic!("the seeded divergence must be frame_anchors, found {other:?}"),
    }
    // The step is divergent, the rest still matching, the summary honest.
    assert_eq!(report.divergent, 1);
    assert_eq!(report.matching, journey.steps.len() as u32 - 1);
    assert_eq!(report.missing, 0);
    ok(report.validate());
    // The report round-trips canonically.
    let serialized = ok(serde_json::to_string_pretty(&report));
    let reloaded: ComparatorReport = ok(serde_json::from_str(&serialized));
    assert_eq!(reloaded, report);
}

/// A seeded assertion divergence — the candidate's picker-open assertion
/// comes back unsatisfied — is named `assertion_verdict`, severe, and
/// located; the observed-state and findings fields that still agree do
/// not double-report.
#[test]
fn a_seeded_assertion_divergence_is_named_severe_and_located() {
    let (journey, reference, candidate) = reference_pair();
    let step_id = ok(StepId::parse("open-picker"));
    let probe_id = ok(ProbeId::parse("assert-picker-open"));

    let seeded = mutate_assertion_verdict(&candidate, &step_id, &probe_id, false);
    let report = ok(compare(&journey, &reference, &seeded));
    assert_eq!(report.divergences.len(), 1);
    let divergence = &report.divergences[0];
    assert_eq!(divergence.step, step_id);
    assert_eq!(divergence.probe.as_ref(), Some(&probe_id));
    assert_eq!(divergence.severity, Severity::Major);
    assert!(matches!(
        divergence.detail,
        DivergenceDetail::AssertionVerdict { .. }
    ));
    ok(report.validate());
}

/// A missing journey step on the candidate is the strongest divergence:
/// the step comparison is Missing with the side named, and the
/// divergence carries the `step_missing` kind at missing severity.
#[test]
fn a_missing_step_is_the_strongest_named_divergence() {
    let (journey, reference, candidate) = reference_pair();
    let missing_step = ok(StepId::parse("palette-land"));
    let seeded = EvidenceRecord {
        steps: candidate
            .steps
            .iter()
            .filter(|step| step.step != missing_step)
            .cloned()
            .collect(),
        ..candidate.clone()
    };
    ok(seeded.validate());
    let report = ok(compare(&journey, &reference, &seeded));
    assert_eq!(report.missing, 1);
    assert_eq!(report.matching, journey.steps.len() as u32 - 1);
    assert_eq!(report.divergences.len(), 1);
    let divergence = &report.divergences[0];
    assert_eq!(divergence.step, missing_step);
    assert_eq!(divergence.severity, Severity::Missing);
    assert!(matches!(
        divergence.detail,
        DivergenceDetail::StepMissing {
            side: Side::Candidate
        }
    ));
    match &report.steps[8] {
        StepComparison::Missing { step, side } => {
            assert_eq!(step, &missing_step);
            assert_eq!(side, &Side::Candidate);
        }
        other => panic!("the palette-land step must be missing, found {other:?}"),
    }
    ok(report.validate());
}

/// A missing PROBE on one side is a named `probe_missing` divergence at
/// missing severity, and the step turns divergent (the step ran but its
/// evidence is incomplete).
#[test]
fn a_missing_probe_is_a_named_divergence() {
    let (journey, reference, candidate) = reference_pair();
    let step_id = ok(StepId::parse("close-picker"));
    let dropped_probe = ok(ProbeId::parse("assert-focus-returns"));
    let seeded = EvidenceRecord {
        steps: candidate
            .steps
            .iter()
            .map(|step| StepEvidence {
                step: step.step.clone(),
                outcome: step.outcome.clone(),
                probes: step
                    .probes
                    .iter()
                    .filter(|probe| probe.probe != dropped_probe)
                    .cloned()
                    .collect(),
            })
            .collect(),
        ..candidate.clone()
    };
    ok(seeded.validate());
    let report = ok(compare(&journey, &reference, &seeded));
    assert_eq!(report.divergences.len(), 1);
    let divergence = &report.divergences[0];
    assert_eq!(divergence.step, step_id);
    assert_eq!(divergence.severity, Severity::Missing);
    match &divergence.detail {
        DivergenceDetail::ProbeMissing { probe, side } => {
            assert_eq!(probe, &dropped_probe);
            assert_eq!(side, &Side::Candidate);
        }
        other => panic!("expected probe_missing, found {other:?}"),
    }
    assert_eq!(report.divergent, 1);
    ok(report.validate());
}

/// Provider-local fields NEVER diverge: flipping every frame digest and
/// every VLM read reference on the candidate still reports zero
/// divergences — pixels and archive locations are never the comparison
/// (the raw-screenshots-alone rule made structural).
#[test]
fn provider_local_fields_never_diverge() {
    let (journey, reference, candidate) = reference_pair();
    let mut digest_flips = 0;
    let mut read_flips = 0;
    let seeded = EvidenceRecord {
        steps: candidate
            .steps
            .iter()
            .map(|step| {
                let probes = step
                    .probes
                    .iter()
                    .map(|probe| {
                        let mut probe = probe.clone();
                        match &mut probe.result {
                            ProbeResult::Frame { digest, vlm, .. } => {
                                *digest = FrameDigest::from_fnv1a(0x0123_4567_89ab_cdef);
                                digest_flips += 1;
                                if let Some(slot) = vlm {
                                    slot.read = Some(ok(flauz_lab::VlmReadRef::parse(
                                        "vlm-renamed-archive-9999.json",
                                    )));
                                    read_flips += 1;
                                }
                            }
                            ProbeResult::Assertion { .. } => {}
                        }
                        probe
                    })
                    .collect();
                StepEvidence {
                    step: step.step.clone(),
                    outcome: step.outcome.clone(),
                    probes,
                }
            })
            .collect(),
        ..candidate.clone()
    };
    ok(seeded.validate());
    assert!(digest_flips > 0, "the candidate digests were flipped");
    assert!(read_flips > 0, "the read references were flipped");
    let report = ok(compare(&journey, &reference, &seeded));
    assert!(
        report.is_clean(),
        "provider-local fields must never diverge; found {:?}",
        report.divergences
    );
}

/// Determinism: identical inputs produce byte-identical reports (kernel
/// §7), and the report identifies the runs and providers it compared.
#[test]
fn the_comparator_is_deterministic_and_byte_stable() {
    let (journey, reference, candidate) = reference_pair();
    let first = ok(compare(&journey, &reference, &candidate));
    let second = ok(compare(&journey, &reference, &candidate));
    assert_eq!(
        ok(serde_json::to_string_pretty(&first)),
        ok(serde_json::to_string_pretty(&second)),
        "identical inputs produce byte-identical reports"
    );
    assert_eq!(first.reference_run, reference.run);
    assert_eq!(first.candidate_run, candidate.run);
    assert_ne!(first.reference_run, first.candidate_run);
    // Run ids are canonical kebab identifiers.
    ok(RunId::parse(first.reference_run.as_str()));
}

/// Mutates one frame probe's observation of one anchor on a record copy
/// (the seeded divergence).
fn mutate_frame_anchor(
    record: &EvidenceRecord,
    step_id: &StepId,
    probe_id: &ProbeId,
    anchor: &str,
    state: &str,
) -> EvidenceRecord {
    let mut seeded = record.clone();
    let step = seeded
        .steps
        .iter_mut()
        .find(|step| &step.step == step_id)
        .unwrap_or_else(|| panic!("the seeded record has step {step_id}"));
    let probe = step
        .probes
        .iter_mut()
        .find(|probe| &probe.probe == probe_id)
        .unwrap_or_else(|| panic!("step {step_id} has probe {probe_id}"));
    match &mut probe.result {
        ProbeResult::Frame { anchors, .. } => {
            let observation = anchors
                .iter_mut()
                .find(|observation| observation.anchor.as_str() == anchor)
                .unwrap_or_else(|| panic!("the frame observes anchor {anchor}"));
            observation.state = ok(flauz_lab::StateKey::parse(state));
        }
        other => panic!("expected a frame probe, found {other:?}"),
    }
    ok(seeded.validate());
    seeded
}

/// Flips one assertion probe's verdict on a record copy.
fn mutate_assertion_verdict(
    record: &EvidenceRecord,
    step_id: &StepId,
    probe_id: &ProbeId,
    satisfied: bool,
) -> EvidenceRecord {
    let mut seeded = record.clone();
    let step = seeded
        .steps
        .iter_mut()
        .find(|step| &step.step == step_id)
        .unwrap_or_else(|| panic!("the seeded record has step {step_id}"));
    let probe = step
        .probes
        .iter_mut()
        .find(|probe| &probe.probe == probe_id)
        .unwrap_or_else(|| panic!("the record has probe {probe_id}"));
    match &mut probe.result {
        ProbeResult::Assertion {
            satisfied: verdict, ..
        } => *verdict = satisfied,
        other => panic!("expected an assertion probe, found {other:?}"),
    }
    ok(seeded.validate());
    seeded
}
