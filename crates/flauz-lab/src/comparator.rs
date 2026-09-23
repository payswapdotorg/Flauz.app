//! The reference-vs-candidate comparator (LAB-001, Wave-4 kernel
//! addendum §4): the diff through the NORMALIZED schema — never raw
//! screenshots alone.
//!
//! [`compare`] walks a [`JourneySpec`](crate::JourneySpec) against a
//! reference and a candidate [`EvidenceRecord`](crate::evidence) and
//! produces a canonical [`ComparatorReport`]: per journey step,
//! matching / divergent / missing evidence, and per divergence a NAMED
//! kind, a SEVERITY and a LOCATION (step + probe). Provider-local
//! fields — frame references, frame digests (renderer fingerprints),
//! VLM read-result references (archive locations) and the adapter
//! descriptors — are ignored BY DESIGN: identical journeys on different
//! providers agree on the normalized content, so the F8 gate law reads
//! "the same journey bytes produce schema-comparable evidence".
//!
//! # Determinism
//!
//! The comparator is a pure function of (journey, reference, candidate):
//! no wall clock, no randomness, no identifiers. Identical inputs produce
//! byte-identical reports — the determinism law the conformance tests
//! pin; identical runs report ZERO divergences, and a seeded divergence
//! comes out named, severe and located.

use serde::{Deserialize, Serialize};

use crate::evidence::{
    ActionOutcome, AnchorObservation, EvidenceRecord, ObservedState, ProbeResult, VlmSlot,
};
use crate::journey::{JourneySpec, StateAssertion};
use crate::refs::{FindingId, FrameMoment, JourneyId, ProbeId, RunId, StepId};
use crate::{LabError, LabVersion, MAX_NAME_BYTES, MAX_STEPS, ensure_list_bound, ensure_str_bound};

/// Which side of a comparison lacks evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// The reference record lacks the evidence.
    Reference,
    /// The candidate record lacks the evidence.
    Candidate,
    /// Both records lack the evidence (the run did not cover it).
    Both,
}

/// The severity of one divergence (descending severity: `missing` >
/// `major` > `minor`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Evidence is absent on one side: a journey step or a spec'd probe
    /// with no counterpart — the run did not cover the journey.
    Missing,
    /// A behavioral divergence: the action outcome, an assertion verdict,
    /// the observed state, the normalized frame anchors or the linked
    /// findings disagree.
    Major,
    /// A metadata-only divergence: the adjudication prompt differs. Never
    /// behavioral.
    Minor,
}

/// One step's comparison verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum StepComparison {
    /// Both sides carry the step and every comparable field agrees.
    Matching {
        /// The step.
        step: StepId,
    },
    /// Both sides carry the step but comparable fields disagree.
    Divergent {
        /// The step.
        step: StepId,
    },
    /// At least one side lacks the step's evidence.
    Missing {
        /// The step.
        step: StepId,
        /// Which side lacks the evidence.
        side: Side,
    },
}

/// The named, located detail of one divergence — the canonical kind is
/// the `kind` tag; the payload carries both sides' values so the report
/// names WHAT diverged without re-reading the records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum DivergenceDetail {
    /// The step's action outcome diverged.
    ActionOutcome {
        /// The reference side's outcome.
        reference: ActionOutcome,
        /// The candidate side's outcome.
        candidate: ActionOutcome,
    },
    /// A journey step's evidence is missing on one side.
    StepMissing {
        /// Which side lacks the step's evidence.
        side: Side,
    },
    /// A spec'd probe's evidence is missing on one side.
    ProbeMissing {
        /// The probe.
        probe: ProbeId,
        /// Which side lacks the probe's evidence.
        side: Side,
    },
    /// The probe's result shape diverged (a frame on one side, an
    /// assertion on the other).
    ProbeShape {
        /// The probe.
        probe: ProbeId,
        /// The reference side's result shape (`frame` or `assertion`).
        reference: String,
        /// The candidate side's result shape.
        candidate: String,
    },
    /// The assertion's verdict diverged (satisfied on one side, not on
    /// the other).
    AssertionVerdict {
        /// The assertion.
        assertion: StateAssertion,
        /// The reference side's verdict.
        reference: bool,
        /// The candidate side's verdict.
        candidate: bool,
    },
    /// The assertion's observed state diverged.
    AssertionObserved {
        /// The assertion.
        assertion: StateAssertion,
        /// The reference side's observation.
        reference: ObservedState,
        /// The candidate side's observation.
        candidate: ObservedState,
    },
    /// The assertion's known-finding links diverged.
    AssertionFindings {
        /// The assertion.
        assertion: StateAssertion,
        /// The reference side's finding links.
        reference: Vec<FindingId>,
        /// The candidate side's finding links.
        candidate: Vec<FindingId>,
    },
    /// The frame's normalized anchor observations diverged (the frame
    /// content — the raw-screenshots-alone rule made structural: pixels
    /// are never compared, the NORMALIZED anchors are).
    FrameAnchors {
        /// The named moment the frame was captured at.
        moment: FrameMoment,
        /// The reference side's anchor observations.
        reference: Vec<AnchorObservation>,
        /// The candidate side's anchor observations.
        candidate: Vec<AnchorObservation>,
    },
    /// The frame's VLM adjudication prompt diverged (metadata-only).
    VlmPrompt {
        /// The named moment.
        moment: FrameMoment,
        /// The reference side's prompt.
        reference: String,
        /// The candidate side's prompt.
        candidate: String,
    },
}

impl DivergenceDetail {
    /// The divergence's canonical severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            Self::StepMissing { .. } | Self::ProbeMissing { .. } => Severity::Missing,
            Self::VlmPrompt { .. } => Severity::Minor,
            Self::ActionOutcome { .. }
            | Self::ProbeShape { .. }
            | Self::AssertionVerdict { .. }
            | Self::AssertionObserved { .. }
            | Self::AssertionFindings { .. }
            | Self::FrameAnchors { .. } => Severity::Major,
        }
    }
}

/// One named, located, severity-ranked divergence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Divergence {
    /// The journey step where the divergence was found.
    pub step: StepId,
    /// The probe where the divergence was found (`None` — the step's
    /// action trace itself).
    pub probe: Option<ProbeId>,
    /// The divergence's severity.
    pub severity: Severity,
    /// The named detail (the canonical kind is its `kind` tag).
    pub detail: DivergenceDetail,
}

/// The canonical diff report of one reference-vs-candidate comparison:
/// the runs' identities and providers, the per-step comparison in journey
/// order, the flat divergence list, and the step-count summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparatorReport {
    /// Contract schema version (`"v": 1`).
    pub v: LabVersion,
    /// The journey both records ran.
    pub journey: JourneyId,
    /// The reference run's identifier.
    pub reference_run: RunId,
    /// The candidate run's identifier.
    pub candidate_run: RunId,
    /// The reference provider's kind (topology identification only —
    /// descriptors are never compared).
    pub reference_provider: String,
    /// The candidate provider's kind.
    pub candidate_provider: String,
    /// The per-step comparison, in the journey's canonical order.
    pub steps: Vec<StepComparison>,
    /// Every divergence, in walk order (journey step order, then probe
    /// order) — deterministic.
    pub divergences: Vec<Divergence>,
    /// How many journey steps matched on both sides.
    pub matching: u32,
    /// How many journey steps diverged.
    pub divergent: u32,
    /// How many journey steps are missing evidence on a side.
    pub missing: u32,
}

impl ComparatorReport {
    /// Validates the report's canonical shape: non-empty bounded
    /// provider labels, the summary counts matching the per-step
    /// comparisons, and every divergence located at a divergent or
    /// missing step.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the canonical shape is violated.
    pub fn validate(&self) -> Result<(), LabError> {
        if self.reference_provider.is_empty() || self.candidate_provider.is_empty() {
            return Err(LabError::invalid("provider labels must not be empty"));
        }
        ensure_str_bound(
            "reference provider",
            &self.reference_provider,
            MAX_NAME_BYTES,
        )?;
        ensure_str_bound(
            "candidate provider",
            &self.candidate_provider,
            MAX_NAME_BYTES,
        )?;
        ensure_list_bound("report steps", self.steps.len(), MAX_STEPS)?;
        let mut matching = 0_u32;
        let mut divergent = 0_u32;
        let mut missing = 0_u32;
        let mut covered: Vec<&StepId> = Vec::with_capacity(self.steps.len());
        for comparison in &self.steps {
            match comparison {
                StepComparison::Matching { step } => {
                    matching += 1;
                    covered.push(step);
                }
                StepComparison::Divergent { step } => {
                    divergent += 1;
                    covered.push(step);
                }
                StepComparison::Missing { step, .. } => {
                    missing += 1;
                    covered.push(step);
                }
            }
        }
        if (matching, divergent, missing) != (self.matching, self.divergent, self.missing) {
            return Err(LabError::invalid(
                "the step summary must match the per-step comparisons",
            ));
        }
        for divergence in &self.divergences {
            if !covered.contains(&&divergence.step) {
                return Err(LabError::invalid(format!(
                    "the divergence at step {} names a step outside the comparison",
                    divergence.step
                )));
            }
        }
        Ok(())
    }

    /// Whether the report is the zero-divergence report (the
    /// comparator's empty state — tested).
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.divergences.is_empty()
    }
}

/// Compares a candidate evidence record against a reference through the
/// NORMALIZED schema: per journey step, matching / divergent / missing
/// evidence with named, located, severity-ranked divergences. Frame
/// references, digests, VLM read references and adapter descriptors are
/// provider-local and never compared.
///
/// # Errors
///
/// Returns [`LabError`] when the journey or either record fails
/// validation, or when either record's journey id does not match the
/// spec.
pub fn compare(
    journey: &JourneySpec,
    reference: &EvidenceRecord,
    candidate: &EvidenceRecord,
) -> Result<ComparatorReport, LabError> {
    journey.validate()?;
    reference.validate()?;
    candidate.validate()?;
    if reference.journey != journey.id || candidate.journey != journey.id {
        return Err(LabError::invalid(format!(
            "both records must evidence journey {}, found {} and {}",
            journey.id, reference.journey, candidate.journey
        )));
    }
    let mut steps = Vec::with_capacity(journey.steps.len());
    let mut divergences = Vec::new();
    let mut matching = 0_u32;
    let mut divergent = 0_u32;
    let mut missing = 0_u32;
    for step in &journey.steps {
        let reference_step = reference.step(&step.id);
        let candidate_step = candidate.step(&step.id);
        let (reference_step, candidate_step) = match (reference_step, candidate_step) {
            (Some(reference_step), Some(candidate_step)) => (reference_step, candidate_step),
            (reference_side, candidate_side) => {
                let side = match (reference_side.is_some(), candidate_side.is_some()) {
                    (false, false) => Side::Both,
                    (false, true) => Side::Reference,
                    (true, false) => Side::Candidate,
                    (true, true) => unreachable!("both sides present is the first arm"),
                };
                missing += 1;
                steps.push(StepComparison::Missing {
                    step: step.id.clone(),
                    side,
                });
                divergences.push(Divergence {
                    step: step.id.clone(),
                    probe: None,
                    severity: Severity::Missing,
                    detail: DivergenceDetail::StepMissing { side },
                });
                continue;
            }
        };
        let mut step_divergences = Vec::new();
        if reference_step.outcome != candidate_step.outcome {
            step_divergences.push(Divergence {
                step: step.id.clone(),
                probe: None,
                severity: Severity::Major,
                detail: DivergenceDetail::ActionOutcome {
                    reference: reference_step.outcome.clone(),
                    candidate: candidate_step.outcome.clone(),
                },
            });
        }
        for probe in &step.probes {
            let reference_probe = reference_step
                .probes
                .iter()
                .find(|evidence| evidence.probe == probe.id);
            let candidate_probe = candidate_step
                .probes
                .iter()
                .find(|evidence| evidence.probe == probe.id);
            match (reference_probe, candidate_probe) {
                (Some(reference_probe), Some(candidate_probe)) => compare_probe_results(
                    &step.id,
                    &probe.id,
                    &reference_probe.result,
                    &candidate_probe.result,
                    &mut step_divergences,
                ),
                (reference_side, candidate_side) => {
                    let side = match (reference_side.is_some(), candidate_side.is_some()) {
                        (false, false) => Side::Both,
                        (false, true) => Side::Reference,
                        (true, false) => Side::Candidate,
                        (true, true) => unreachable!("both sides present is the first arm"),
                    };
                    step_divergences.push(Divergence {
                        step: step.id.clone(),
                        probe: Some(probe.id.clone()),
                        severity: Severity::Missing,
                        detail: DivergenceDetail::ProbeMissing {
                            probe: probe.id.clone(),
                            side,
                        },
                    });
                }
            }
        }
        if step_divergences.is_empty() {
            matching += 1;
            steps.push(StepComparison::Matching {
                step: step.id.clone(),
            });
        } else {
            divergent += 1;
            steps.push(StepComparison::Divergent {
                step: step.id.clone(),
            });
            divergences.append(&mut step_divergences);
        }
    }
    let report = ComparatorReport {
        v: LabVersion,
        journey: journey.id.clone(),
        reference_run: reference.run.clone(),
        candidate_run: candidate.run.clone(),
        reference_provider: reference.adapter.provider_kind.clone(),
        candidate_provider: candidate.adapter.provider_kind.clone(),
        steps,
        divergences,
        matching,
        divergent,
        missing,
    };
    report.validate()?;
    Ok(report)
}

/// The normalized result shape of a probe result (for shape divergences).
const fn result_shape(result: &ProbeResult) -> &'static str {
    match result {
        ProbeResult::Frame { .. } => "frame",
        ProbeResult::Assertion { .. } => "assertion",
    }
}

/// Compares one probe's two results through the normalized schema,
/// appending every divergence (provider-local fields are skipped by
/// design).
fn compare_probe_results(
    step: &StepId,
    probe: &ProbeId,
    reference: &ProbeResult,
    candidate: &ProbeResult,
    divergences: &mut Vec<Divergence>,
) {
    match (reference, candidate) {
        (
            ProbeResult::Frame {
                moment,
                anchors: reference_anchors,
                vlm: reference_vlm,
                ..
            },
            ProbeResult::Frame {
                anchors: candidate_anchors,
                vlm: candidate_vlm,
                ..
            },
        ) => {
            if reference_anchors != candidate_anchors {
                divergences.push(Divergence {
                    step: step.clone(),
                    probe: Some(probe.clone()),
                    severity: Severity::Major,
                    detail: DivergenceDetail::FrameAnchors {
                        moment: moment.clone(),
                        reference: reference_anchors.clone(),
                        candidate: candidate_anchors.clone(),
                    },
                });
            }
            if let Some(divergence) =
                compare_vlm_prompts(step, probe, moment, reference_vlm, candidate_vlm)
            {
                divergences.push(divergence);
            }
        }
        (
            ProbeResult::Assertion {
                assertion,
                satisfied: reference_satisfied,
                observed: reference_observed,
                findings: reference_findings,
                ..
            },
            ProbeResult::Assertion {
                satisfied: candidate_satisfied,
                observed: candidate_observed,
                findings: candidate_findings,
                ..
            },
        ) => {
            if reference_satisfied != candidate_satisfied {
                divergences.push(Divergence {
                    step: step.clone(),
                    probe: Some(probe.clone()),
                    severity: Severity::Major,
                    detail: DivergenceDetail::AssertionVerdict {
                        assertion: assertion.clone(),
                        reference: *reference_satisfied,
                        candidate: *candidate_satisfied,
                    },
                });
            }
            if reference_observed != candidate_observed {
                divergences.push(Divergence {
                    step: step.clone(),
                    probe: Some(probe.clone()),
                    severity: Severity::Major,
                    detail: DivergenceDetail::AssertionObserved {
                        assertion: assertion.clone(),
                        reference: reference_observed.clone(),
                        candidate: candidate_observed.clone(),
                    },
                });
            }
            if reference_findings != candidate_findings {
                divergences.push(Divergence {
                    step: step.clone(),
                    probe: Some(probe.clone()),
                    severity: Severity::Major,
                    detail: DivergenceDetail::AssertionFindings {
                        assertion: assertion.clone(),
                        reference: reference_findings.clone(),
                        candidate: candidate_findings.clone(),
                    },
                });
            }
        }
        (reference_result, candidate_result) => {
            divergences.push(Divergence {
                step: step.clone(),
                probe: Some(probe.clone()),
                severity: Severity::Major,
                detail: DivergenceDetail::ProbeShape {
                    probe: probe.clone(),
                    reference: result_shape(reference_result).to_owned(),
                    candidate: result_shape(candidate_result).to_owned(),
                },
            });
        }
    }
}

/// Compares two optional VLM slots' PROMPTS (the read-result references
/// are provider-local archive locations and are never compared).
fn compare_vlm_prompts(
    step: &StepId,
    probe: &ProbeId,
    moment: &FrameMoment,
    reference: &Option<VlmSlot>,
    candidate: &Option<VlmSlot>,
) -> Option<Divergence> {
    match (reference, candidate) {
        (Some(reference), Some(candidate)) if reference.prompt != candidate.prompt => {
            Some(Divergence {
                step: step.clone(),
                probe: Some(probe.clone()),
                severity: Severity::Minor,
                detail: DivergenceDetail::VlmPrompt {
                    moment: moment.clone(),
                    reference: reference.prompt.clone(),
                    candidate: candidate.prompt.clone(),
                },
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn severities_are_ordered_and_named() {
        // The canonical scale: missing > major > minor.
        assert!(matches!(
            DivergenceDetail::StepMissing { side: Side::Both }.severity(),
            Severity::Missing
        ));
        assert!(matches!(
            DivergenceDetail::ProbeMissing {
                probe: ok(ProbeId::parse("p")),
                side: Side::Candidate
            }
            .severity(),
            Severity::Missing
        ));
        assert!(matches!(
            DivergenceDetail::VlmPrompt {
                moment: ok(FrameMoment::parse("m")),
                reference: "a".to_owned(),
                candidate: "b".to_owned(),
            }
            .severity(),
            Severity::Minor
        ));
        assert!(matches!(
            DivergenceDetail::FrameAnchors {
                moment: ok(FrameMoment::parse("m")),
                reference: vec![],
                candidate: vec![],
            }
            .severity(),
            Severity::Major
        ));
    }

    #[test]
    fn step_comparisons_serialize_internally_tagged() {
        let matching = StepComparison::Matching {
            step: ok(StepId::parse("open-picker")),
        };
        assert_eq!(
            ok(serde_json::to_string(&matching)),
            "{\"kind\":\"matching\",\"step\":\"open-picker\"}"
        );
        let missing = StepComparison::Missing {
            step: ok(StepId::parse("open-picker")),
            side: Side::Candidate,
        };
        assert_eq!(
            ok(serde_json::to_string(&missing)),
            "{\"kind\":\"missing\",\"step\":\"open-picker\",\"side\":\"candidate\"}"
        );
        let divergent = StepComparison::Divergent {
            step: ok(StepId::parse("open-picker")),
        };
        let reloaded: StepComparison =
            ok(serde_json::from_str(&ok(serde_json::to_string(&divergent))));
        assert_eq!(reloaded, divergent);
        assert!(
            serde_json::from_str::<Side>("\"neither\"").is_err(),
            "unknown sides are rejected"
        );
    }
}
