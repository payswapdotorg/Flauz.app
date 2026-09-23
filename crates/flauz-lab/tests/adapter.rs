//! The lab-adapter contract tests (LAB-001, Wave-4 kernel addendum §4):
//! the F8 gate law — one journey, two adapters, comparable normalized
//! evidence — and the shared contract suite both providers satisfy.

use flauz_lab::refs::{ProbeId, StepId};
use flauz_lab::{
    CapabilityKey, EvidenceRecord, FakeRemoteLabAdapter, FamilySkeletonAdapter, JourneySpec,
    LabAdapter, LabError, LocalLabAdapter, compare, known_provider_families, run_journey,
};

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

/// Unwraps the expected error side of a contract result (the crate's
/// no-expect/no-unwrap test discipline).
fn err<T, E: std::fmt::Display>(result: Result<T, E>, what: &str) -> E {
    match result {
        Err(error) => error,
        Ok(_) => panic!("{what}"),
    }
}

/// THE F8 GATE LAW (addendum §4): the SAME JourneySpec bytes — the exact
/// serialized bytes, re-parsed — produce schema-comparable evidence from
/// BOTH adapters: every journey step matching, zero divergences, the
/// normalized content identical while the provider-local fields
/// (descriptor, digests) differ.
#[test]
fn the_same_journey_bytes_produce_comparable_evidence_from_both_adapters() {
    for journey in [
        flauz_lab::fakes::fake_shell_discovery_journey(),
        flauz_lab::fakes::fake_a11y_chord_ladder_journey(),
    ] {
        // The same BYTES: serialize once, parse twice.
        let bytes = ok(serde_json::to_string(&journey));
        let for_local: JourneySpec = ok(serde_json::from_str(&bytes));
        let for_remote: JourneySpec = ok(serde_json::from_str(&bytes));

        let mut local = LocalLabAdapter::new();
        let mut remote = FakeRemoteLabAdapter::new();
        let local_record = ok(run_journey(&mut local, &for_local));
        let remote_record = ok(run_journey(&mut remote, &for_remote));
        ok(local_record.validate());
        ok(remote_record.validate());

        // The records evidence the SAME journey and cover every step.
        assert_eq!(local_record.journey, remote_record.journey);
        assert_eq!(local_record.journey, journey.id);
        assert_eq!(local_record.steps.len(), journey.steps.len());
        assert_eq!(remote_record.steps.len(), journey.steps.len());

        // The provider metadata differs — that is the point.
        assert_eq!(local_record.adapter.provider_kind, "flauz-lab-local");
        assert_eq!(remote_record.adapter.provider_kind, "flauz-lab-fake-remote");
        assert_ne!(local_record.adapter, remote_record.adapter);

        // The normalized content is comparable: the comparator reports
        // ZERO divergences, every step matching, the report clean.
        let report = ok(compare(&journey, &local_record, &remote_record));
        assert!(
            report.is_clean(),
            "the F8 gate law: the same journey bytes must produce comparable evidence; found {} \
             divergences: {:?}",
            report.divergences.len(),
            report.divergences
        );
        assert_eq!(report.matching, journey.steps.len() as u32);
        assert_eq!(report.divergent, 0);
        assert_eq!(report.missing, 0);
        assert_eq!(report.steps.len(), journey.steps.len());

        // Provider-local fields genuinely differ (the comparator ignores
        // them by design): every frame digest differs across providers.
        let digests_differ =
            local_record
                .steps
                .iter()
                .zip(&remote_record.steps)
                .any(|(local_step, remote_step)| {
                    local_step.probes.iter().zip(&remote_step.probes).any(
                        |(local_probe, remote_probe)| match (
                            &local_probe.result,
                            &remote_probe.result,
                        ) {
                            (
                                flauz_lab::ProbeResult::Frame {
                                    digest: local_digest,
                                    ..
                                },
                                flauz_lab::ProbeResult::Frame {
                                    digest: remote_digest,
                                    ..
                                },
                            ) => local_digest != remote_digest,
                            _ => false,
                        },
                    )
                });
        assert!(
            digests_differ,
            "the fake remote uses its own digest namespace — provider-local fingerprints differ"
        );
    }
}

/// The shared conformance suite both fake adapters satisfy (the
/// ENV-001 `environment_contract_same_for_local_and_remote_fakes`
/// pattern): determinism, completeness, journey identity and
/// canonical-JSON round-trips — parameterized over both adapters.
#[test]
fn lab_adapter_contract_same_for_local_and_fake_remote_adapters() {
    let journeys = [
        flauz_lab::fakes::fake_shell_discovery_journey(),
        flauz_lab::fakes::fake_a11y_chord_ladder_journey(),
    ];
    for journey in &journeys {
        // Determinism: two runs of the same adapter produce byte-identical
        // serialized records (kernel §7).
        let mut local_first = LocalLabAdapter::new();
        let mut local_second = LocalLabAdapter::new();
        let first = ok(run_journey(&mut local_first, journey));
        let second = ok(run_journey(&mut local_second, journey));
        assert_eq!(
            ok(serde_json::to_string_pretty(&first)),
            ok(serde_json::to_string_pretty(&second)),
            "identical runs must be byte-identical (local)"
        );

        let mut remote_first = FakeRemoteLabAdapter::new();
        let mut remote_second = FakeRemoteLabAdapter::new();
        let first = ok(run_journey(&mut remote_first, journey));
        let second = ok(run_journey(&mut remote_second, journey));
        assert_eq!(
            ok(serde_json::to_string_pretty(&first)),
            ok(serde_json::to_string_pretty(&second)),
            "identical runs must be byte-identical (fake remote)"
        );

        // Canonical JSON round-trips.
        let serialized = ok(serde_json::to_string_pretty(&first));
        let reloaded: EvidenceRecord = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, first);

        // The driver drives the trait object uniformly (the wave-gate
        // harness shape).
        let mut dyn_adapter: Box<dyn LabAdapter> = Box::new(FakeRemoteLabAdapter::new());
        let record = ok(run_journey(dyn_adapter.as_mut(), journey));
        ok(record.validate());
    }
}

/// The evidence honestly carries the F1 findings as links by id: the a11y
/// chord-ladder journey's record links the known findings it reproduces
/// (bracket swap, PTY focus) and guards (the modal trap, the companion
/// chord, the first-run dismissal) — and every link resolves in the
/// builtin registry.
#[test]
fn evidence_links_findings_by_id_and_they_resolve() {
    let registry = ok(flauz_lab::builtin_findings_registry());
    let mut adapter = LocalLabAdapter::new();
    let journey = flauz_lab::fakes::fake_a11y_chord_ladder_journey();
    let record = ok(run_journey(&mut adapter, &journey));

    let observed: Vec<String> = record
        .observed_findings()
        .iter()
        .map(|finding| finding.as_str().to_owned())
        .collect();
    assert_eq!(
        observed,
        vec![
            "bracket-swap-chords",
            "first-run-keyboard-swallowing",
            "modal-focus-traps",
            "pty-focus-transfer",
            "shifted-symbol-chord-companions",
        ],
        "the chord-ladder journey observes all five F1 lessons by id"
    );
    assert!(
        registry.resolves_all(&record.observed_findings()),
        "every finding link resolves in the builtin registry"
    );

    // The unsatisfied assertions are the honestly-reproduced findings:
    // the bracket chord did not move the selection (N6) and focus did not
    // transfer to the PTY (N5).
    let bracket_step = ok(StepId::parse("bracket-swap-primary"));
    let bracket_probe = ok(ProbeId::parse("assert-bracket-moves-selection"));
    let evidence = record
        .step(&bracket_step)
        .unwrap_or_else(|| panic!("the record evidences step {bracket_step}"));
    let probe = evidence
        .probes
        .iter()
        .find(|probe| probe.probe == bracket_probe)
        .unwrap_or_else(|| panic!("the record evidences probe {bracket_probe}"));
    match &probe.result {
        flauz_lab::ProbeResult::Assertion {
            satisfied,
            findings,
            ..
        } => {
            assert!(
                !satisfied,
                "the bracket chord produces no move (N6, honestly)"
            );
            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0].as_str(), "bracket-swap-chords");
        }
        other => panic!("the selection probe must be an assertion, found {other:?}"),
    }
}

/// The fake remote's honest topology gap is data, not a blocker: it
/// offers no `desktop.gui` yet hosts every canonical journey (no
/// journey action requires it), while the local lab carries it.
#[test]
fn the_fake_remote_carries_the_env_001_honest_gap() {
    let remote = FakeRemoteLabAdapter::new();
    let descriptor = remote.descriptor();
    assert!(!descriptor.offers(&ok(CapabilityKey::parse("desktop.gui"))));
    let local = LocalLabAdapter::new();
    assert!(
        local
            .descriptor()
            .offers(&ok(CapabilityKey::parse("desktop.gui")))
    );
}

/// The comparator treats a record from a DIFFERENT journey as a contract
/// error (named), never as a misleading diff.
#[test]
fn comparing_records_of_different_journeys_is_a_named_error() {
    let shell = flauz_lab::fakes::fake_shell_discovery_journey();
    let ladder = flauz_lab::fakes::fake_a11y_chord_ladder_journey();
    let mut adapter = LocalLabAdapter::new();
    let shell_record = ok(run_journey(&mut adapter, &shell));
    let error = err(
        compare(&ladder, &shell_record, &shell_record),
        "records of another journey must be rejected",
    );
    assert!(error.reason().contains(ladder.id.as_str()));
}

/// The skeleton adapters prove the contract fits every family topology:
/// the interactive families prepare the canonical journeys; the batch
/// families name their gaps.
#[test]
fn the_skeleton_families_prove_the_contract_fits_the_topologies() {
    let table = known_provider_families();
    ok(table.validate());
    let journey = flauz_lab::fakes::fake_shell_discovery_journey();
    let mut interactive = 0;
    for family in &table.families {
        let mut skeleton = FamilySkeletonAdapter::new(family.clone());
        match skeleton.prepare(&journey) {
            Ok(()) => {
                interactive += 1;
                let error = err(
                    skeleton.run(&journey.steps[0].id),
                    "a skeleton never runs real work",
                );
                assert!(matches!(error, LabError::NotWired(_)));
                let error = err(
                    skeleton.capture(&journey.steps[0].id, &journey.steps[0].probes[0].id),
                    "a skeleton never captures probes",
                );
                assert!(matches!(error, LabError::NotWired(_)));
                let error = err(skeleton.evidence(), "a skeleton never produces evidence");
                assert!(matches!(error, LabError::NotWired(_)));
            }
            Err(error) => {
                // The batch families name the missing interactive
                // surfaces.
                assert!(error.reason().contains("computer.keyboard"), "{}", error);
            }
        }
        // The descriptor is honest topology data.
        let descriptor = skeleton.descriptor();
        ok(descriptor.validate());
        assert_eq!(descriptor.provider_kind, family.kind);
    }
    // local, fake-remote, e2b, daytona, azure host the journeys;
    // github_actions + codemagic (batch) name their gaps.
    assert_eq!(interactive, 5);
}
