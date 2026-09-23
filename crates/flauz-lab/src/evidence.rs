//! The normalized evidence record (LAB-001, Wave-4 kernel addendum §4):
//! the schema'd, canonical-JSON, provider-comparable output of one lab
//! run.
//!
//! An [`EvidenceRecord`] is what every compliant lab adapter produces for
//! a [`JourneySpec`](crate::JourneySpec): per-step action traces, frame
//! REFERENCES (not pixels inline) with their NORMALIZED anchor
//! observations, assertion results carrying the F1 accessibility patterns
//! as data with known-findings links by id, VLM-read slots (the
//! adjudication prompt + the read-result reference — no VLM calls inside
//! this crate), and the environment/provider metadata (topology data;
//! no credentials).
//!
//! # Provider comparability (the F8 law, made structural)
//!
//! The comparator may compare these fields across providers: action
//! outcomes, assertion verdicts/observations/findings, frame moments and
//! anchor observations, and VLM prompts. It must NEVER compare
//! provider-local fields: the frame references, the frame digests
//! (renderer fingerprints), the VLM read-result references (archive
//! locations) and the adapter descriptor itself — identical journeys on
//! different providers agree on the normalized content and differ only
//! in these, by design.

use serde::{Deserialize, Serialize};

use crate::journey::StateAssertion;
use crate::refs::{
    AnchorId, CapabilityKey, FindingId, FrameDigest, FrameMoment, FrameRef, JourneyId, ProbeId,
    RunId, StateKey, StepId, VlmReadRef,
};
use crate::{
    LabError, LabVersion, MAX_FINDING_LINKS, MAX_FRAME_ANCHORS, MAX_LAB_SURFACES, MAX_STEPS,
    ensure_explanation, ensure_list_bound, ensure_non_empty, ensure_sorted_unique,
    ensure_str_bound,
};

/// The locality a lab adapter runs at: the local lab (in-repo, the
/// reference provider) or a remote sandbox provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locality {
    /// The in-repo local lab (the Lead's Linux lab).
    Local,
    /// A remote sandbox provider (the fake-remote consistency provider
    /// now; E2B/Daytona/Azure/GitHub Actions/Codemagic later).
    Remote,
}

/// The adapter family a lab provider belongs to: the reference local
/// lab, the fake-remote consistency provider (the ENV-001 law — the fake
/// remote every real remote copies), or one of the real remote families
/// (skeletons this wave: data + fakes only, no network).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderFamily {
    /// The in-repo local Linux lab — the reference provider.
    Local,
    /// The deterministic fake-remote — the consistency provider.
    FakeRemote,
    /// E2B interactive sandboxes (skeleton this wave).
    E2b,
    /// Daytona environments (skeleton this wave).
    Daytona,
    /// Azure desktop hosts (skeleton this wave).
    Azure,
    /// GitHub Actions batch provider (skeleton this wave).
    GithubActions,
    /// Codemagic batch provider (skeleton this wave).
    Codemagic,
}

impl ProviderFamily {
    /// The canonical kind label of the family (the provider-kind prefix
    /// real adapters will carry).
    #[must_use]
    pub const fn kind(self) -> &'static str {
        match self {
            Self::Local => "flauz-lab-local",
            Self::FakeRemote => "flauz-lab-fake-remote",
            Self::E2b => "e2b",
            Self::Daytona => "daytona",
            Self::Azure => "azure",
            Self::GithubActions => "github_actions",
            Self::Codemagic => "codemagic",
        }
    }
}

/// The environment/provider metadata of an evidence run — TOPOLOGY data
/// only (addendum §3: credentials are references, forever, and provider
/// metadata never carries credential material): who ran the journey, at
/// what locality, through which family, with which lab surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterDescriptor {
    /// The provider kind (for example `flauz-lab-local` or `e2b`).
    pub provider_kind: String,
    /// The adapter family.
    pub family: ProviderFamily,
    /// The locality the adapter runs at.
    pub locality: Locality,
    /// The lab surfaces the adapter offers (the frozen capability-key
    /// grammar; sorted, deduplicated) — what journeys it can run.
    pub lab_surfaces: Vec<CapabilityKey>,
}

impl AdapterDescriptor {
    /// Builds a descriptor, validating the kind label and the surface
    /// list.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the kind is empty/oversized or the
    /// surfaces are unsorted, duplicated or oversized.
    pub fn new(
        provider_kind: &str,
        family: ProviderFamily,
        locality: Locality,
        lab_surfaces: Vec<CapabilityKey>,
    ) -> Result<Self, LabError> {
        ensure_non_empty("provider kind", provider_kind)?;
        ensure_str_bound("provider kind", provider_kind, crate::MAX_NAME_BYTES)?;
        ensure_list_bound("lab surfaces", lab_surfaces.len(), MAX_LAB_SURFACES)?;
        ensure_sorted_unique("lab surfaces", &lab_surfaces)?;
        Ok(Self {
            provider_kind: provider_kind.to_owned(),
            family,
            locality,
            lab_surfaces,
        })
    }

    /// Validates the descriptor's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the descriptor fails validation.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::new(
            &self.provider_kind,
            self.family,
            self.locality,
            self.lab_surfaces.clone(),
        )?;
        Ok(())
    }

    /// Whether the descriptor offers a lab surface.
    #[must_use]
    pub fn offers(&self, capability: &CapabilityKey) -> bool {
        self.lab_surfaces.contains(capability)
    }
}

/// The normalized observation of one named anchor in one frame: the
/// anchor and the state it was seen in (for example
/// `model-picker-panel: open-empty`). This — not the pixels — is the
/// frame content the comparator diffs across providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorObservation {
    /// The named anchor.
    pub anchor: AnchorId,
    /// The state the anchor was observed in.
    pub state: StateKey,
}

impl AnchorObservation {
    /// Builds an anchor observation, validating both ids.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the anchor or state is outside the
    /// grammar.
    pub fn new(anchor: &str, state: &str) -> Result<Self, LabError> {
        Ok(Self {
            anchor: AnchorId::parse(anchor)?,
            state: StateKey::parse(state)?,
        })
    }

    /// Validates the observation's grammar.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the anchor or state is outside the
    /// grammar.
    pub fn validate(&self) -> Result<(), LabError> {
        self.anchor.validate()?;
        self.state.validate()
    }
}

/// What a probe actually observed — the structured counterpart of the
/// assertion family it answers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum ObservedState {
    /// The anchor's actual state.
    Anchor {
        /// The anchor.
        anchor: AnchorId,
        /// The state the anchor was actually in.
        state: StateKey,
    },
    /// Where keyboard focus actually rests (`None` — focus landed
    /// nowhere, the pre-fix N1/017 shape).
    Focus {
        /// The anchor focus actually rests on, if any.
        anchor: Option<AnchorId>,
    },
    /// The anchors a tab walk actually cycles through, in order.
    FocusCycle {
        /// The observed cycle.
        anchors: Vec<AnchorId>,
    },
    /// The text the anchor actually holds (possibly empty — the
    /// typed-probe-landed-nowhere shape).
    Text {
        /// The anchor.
        anchor: AnchorId,
        /// The text the anchor actually holds.
        text: String,
    },
}

impl ObservedState {
    /// Validates the observation's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any field is outside its bound.
    pub fn validate(&self) -> Result<(), LabError> {
        match self {
            Self::Anchor { anchor, state } => {
                anchor.validate()?;
                state.validate()
            }
            Self::Focus { anchor } => {
                if let Some(anchor) = anchor {
                    anchor.validate()?;
                }
                Ok(())
            }
            Self::FocusCycle { anchors } => {
                ensure_list_bound("focus cycle anchors", anchors.len(), MAX_FRAME_ANCHORS)?;
                if anchors.is_empty() {
                    return Err(LabError::invalid("a focus cycle names at least one anchor"));
                }
                ensure_sorted_unique("focus cycle anchors", anchors)
            }
            Self::Text { anchor, text } => {
                anchor.validate()?;
                ensure_str_bound("observed text", text, crate::MAX_TEXT_BYTES)
            }
        }
    }
}

/// The outcome of one journey action, normalized: the trace of what the
/// adapter's surface did with the action.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum ActionOutcome {
    /// The action dispatched and the surface changed visibly.
    Applied,
    /// The action dispatched with no visible change (the documented
    /// no-op guard family — honest, never a silent failure).
    NoVisibleChange,
    /// The adapter's surface refused the action — `reason` is the
    /// bounded, honest explanation (for example an unknown anchor).
    Rejected {
        /// Why the surface refused the action.
        reason: String,
    },
}

impl ActionOutcome {
    /// Validates the outcome's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when a rejection reason is empty or
    /// oversized.
    pub fn validate(&self) -> Result<(), LabError> {
        if let Self::Rejected { reason } = self {
            ensure_explanation("action rejection reason", reason)?;
        }
        Ok(())
    }
}

/// A VLM-read slot: the operator-side adjudication contract for one
/// frame — the bounded adjudication prompt (data) and the reference to
/// the archived read result (filled operator-side). NO VLM calls happen
/// inside this crate; the slot is what the evidence carries so the
/// adjudication stays attributable and replayable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VlmSlot {
    /// The bounded adjudication prompt for the frame.
    pub prompt: String,
    /// The reference to the archived read result, when the operator has
    /// adjudicated (never inline content).
    pub read: Option<VlmReadRef>,
}

impl VlmSlot {
    /// Builds a slot, validating the prompt's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the prompt is empty or oversized.
    pub fn new(prompt: &str) -> Result<Self, LabError> {
        ensure_explanation("vlm adjudication prompt", prompt)?;
        Ok(Self {
            prompt: prompt.to_owned(),
            read: None,
        })
    }

    /// Validates the slot's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the prompt is empty or oversized or the
    /// read reference is outside the grammar.
    pub fn validate(&self) -> Result<(), LabError> {
        ensure_explanation("vlm adjudication prompt", &self.prompt)?;
        if let Some(read) = &self.read {
            read.validate()?;
        }
        Ok(())
    }
}

/// The normalized result of one probe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum ProbeResult {
    /// A frame captured at a named moment: the run-scoped frame
    /// REFERENCE, the provider-local digest (renderer determinism
    /// fingerprint — the comparator ignores it), the NORMALIZED anchor
    /// observations (the comparable content), and the optional VLM-read
    /// slot.
    Frame {
        /// The named moment the frame was captured at.
        moment: FrameMoment,
        /// The run-scoped frame reference (the archive pointer — not
        /// pixels inline).
        frame: FrameRef,
        /// The provider-local frame digest (ignored by the comparator).
        digest: FrameDigest,
        /// The normalized anchor observations (sorted by anchor).
        anchors: Vec<AnchorObservation>,
        /// The VLM-read slot (prompt + archived read reference).
        vlm: Option<VlmSlot>,
    },
    /// A state assertion's result: the assertion, whether it was
    /// satisfied, what was actually observed, and the known findings the
    /// observation links by id (never by prose).
    Assertion {
        /// The assertion that was probed.
        assertion: StateAssertion,
        /// Whether the observed state satisfied the assertion.
        satisfied: bool,
        /// What was actually observed.
        observed: ObservedState,
        /// The known findings this observation links, by id (sorted,
        /// deduplicated).
        findings: Vec<FindingId>,
    },
}

impl ProbeResult {
    /// Validates the result's bounds and canonical ordering.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any field is outside its bound or the
    /// anchor observations or finding links are unsorted/duplicated.
    pub fn validate(&self) -> Result<(), LabError> {
        match self {
            Self::Frame { anchors, vlm, .. } => {
                ensure_list_bound("frame anchors", anchors.len(), MAX_FRAME_ANCHORS)?;
                let anchor_ids: Vec<&AnchorId> = anchors.iter().map(|a| &a.anchor).collect();
                ensure_sorted_unique("frame anchors", &anchor_ids)?;
                for observation in anchors {
                    observation.validate()?;
                }
                if let Some(vlm) = vlm {
                    vlm.validate()?;
                }
                Ok(())
            }
            Self::Assertion {
                assertion,
                observed,
                findings,
                ..
            } => {
                assertion.validate()?;
                observed.validate()?;
                ensure_list_bound("finding links", findings.len(), MAX_FINDING_LINKS)?;
                ensure_sorted_unique("finding links", findings)?;
                Ok(())
            }
        }
    }
}

/// The evidence of one journey step: the action trace and the probe
/// results, in the spec's probe order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepEvidence {
    /// The step this evidences.
    pub step: StepId,
    /// The action's normalized outcome.
    pub outcome: ActionOutcome,
    /// The probe results (the spec'd probes of this step, in order).
    pub probes: Vec<ProbeEvidence>,
}

/// One probe's evidence: the probe id and its normalized result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeEvidence {
    /// The probe this evidences.
    pub probe: ProbeId,
    /// The normalized result.
    pub result: ProbeResult,
}

/// The normalized evidence record of one lab run (addendum §4): the
/// run's identity, the journey it ran, the adapter's environment/provider
/// metadata, and the per-step evidence. Steps appear in the journey's
/// canonical order; a record MAY carry a subset of the journey's steps
/// (an older or partial export) — the comparator names every step the
/// reference or candidate lacks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    /// Contract schema version (`"v": 1`).
    pub v: LabVersion,
    /// The run's identifier (for example
    /// `flauz-lab-local-shell-discovery`).
    pub run: RunId,
    /// The journey that was run.
    pub journey: JourneyId,
    /// The adapter's environment/provider metadata (topology data; no
    /// credentials).
    pub adapter: AdapterDescriptor,
    /// The per-step evidence, in the journey's step order (unique step
    /// ids).
    pub steps: Vec<StepEvidence>,
}

impl EvidenceRecord {
    /// Builds a record, validating every canonical rule.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any step or probe fails validation, the
    /// step list is oversized, or step ids repeat (steps follow the
    /// journey's canonical order — a subset is allowed, the comparator
    /// names what is missing; probes are canonically ordered by id).
    pub fn new(
        run: RunId,
        journey: JourneyId,
        adapter: AdapterDescriptor,
        steps: Vec<StepEvidence>,
    ) -> Result<Self, LabError> {
        run.validate()?;
        journey.validate()?;
        adapter.validate()?;
        ensure_list_bound("evidence steps", steps.len(), MAX_STEPS)?;
        // Step ids are unique but NOT id-sorted: steps appear in the
        // journey's canonical step order (the spec is the order
        // authority; a record may carry a subset).
        let step_ids: Vec<&StepId> = steps.iter().map(|step| &step.step).collect();
        if step_ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(LabError::invalid("evidence step ids must be unique"));
        }
        let mut steps = steps;
        for step in &mut steps {
            step.outcome.validate()?;
            step.probes
                .sort_by(|left, right| left.probe.cmp(&right.probe));
            let probe_ids: Vec<&ProbeId> = step.probes.iter().map(|probe| &probe.probe).collect();
            ensure_sorted_unique("step probe ids", &probe_ids)?;
            for probe in &step.probes {
                probe.result.validate()?;
            }
        }
        Ok(Self {
            v: LabVersion,
            run,
            journey,
            adapter,
            steps,
        })
    }

    /// Validates the record against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the record fails validation.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::new(
            self.run.clone(),
            self.journey.clone(),
            self.adapter.clone(),
            self.steps.clone(),
        )?;
        Ok(())
    }

    /// Finds a step's evidence by id.
    #[must_use]
    pub fn step(&self, id: &StepId) -> Option<&StepEvidence> {
        self.steps.iter().find(|step| &step.step == id)
    }

    /// Every known finding this record's probe results link, by id (the
    /// sorted, deduplicated union) — records reference findings by id,
    /// never by prose.
    #[must_use]
    pub fn observed_findings(&self) -> Vec<FindingId> {
        let mut findings: Vec<FindingId> = Vec::new();
        for step in &self.steps {
            for probe in &step.probes {
                if let ProbeResult::Assertion {
                    findings: linked, ..
                } = &probe.result
                {
                    for finding in linked {
                        if !findings.contains(finding) {
                            findings.push(finding.clone());
                        }
                    }
                }
            }
        }
        findings.sort();
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_EXPLANATION_BYTES;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn key(value: &str) -> CapabilityKey {
        ok(CapabilityKey::parse(value))
    }

    fn anchor(value: &str) -> AnchorId {
        ok(AnchorId::parse(value))
    }

    fn local_descriptor() -> AdapterDescriptor {
        ok(AdapterDescriptor::new(
            "flauz-lab-local",
            ProviderFamily::Local,
            Locality::Local,
            vec![key("computer.screen"), key("terminal")],
        ))
    }

    fn sample_record() -> EvidenceRecord {
        let assertion = StateAssertion::FocusOn {
            anchor: anchor("task-composer"),
        };
        ok(EvidenceRecord::new(
            ok(RunId::parse("flauz-lab-local-sample")),
            ok(JourneyId::parse("sample-discovery")),
            local_descriptor(),
            vec![StepEvidence {
                step: ok(StepId::parse("submit-task")),
                outcome: ActionOutcome::Applied,
                probes: vec![ProbeEvidence {
                    probe: ok(ProbeId::parse("assert-focus-on-composer")),
                    result: ProbeResult::Assertion {
                        assertion,
                        satisfied: true,
                        observed: ObservedState::Focus {
                            anchor: Some(anchor("task-composer")),
                        },
                        findings: vec![],
                    },
                }],
            }],
        ))
    }

    #[test]
    fn descriptors_carry_topology_data_only() {
        let descriptor = local_descriptor();
        let serialized = ok(serde_json::to_string(&descriptor));
        assert!(serialized.contains("\"provider_kind\":\"flauz-lab-local\""));
        assert!(serialized.contains("\"family\":\"local\""));
        assert!(serialized.contains("\"locality\":\"local\""));
        let reloaded: AdapterDescriptor = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, descriptor);
        assert!(descriptor.offers(&key("computer.screen")));
        assert!(!descriptor.offers(&key("desktop.gui")));
        // Unsorted surfaces and empty kinds are rejected.
        assert!(
            AdapterDescriptor::new(
                "flauz-lab-local",
                ProviderFamily::Local,
                Locality::Local,
                vec![key("terminal"), key("computer.screen")],
            )
            .is_err(),
            "lab surfaces must be sorted"
        );
        assert!(
            AdapterDescriptor::new("", ProviderFamily::Local, Locality::Local, vec![]).is_err(),
            "the provider kind must not be empty"
        );
    }

    #[test]
    fn families_serialize_as_their_snake_case_names() {
        let expected = [
            (ProviderFamily::Local, "local"),
            (ProviderFamily::FakeRemote, "fake_remote"),
            (ProviderFamily::E2b, "e2b"),
            (ProviderFamily::Daytona, "daytona"),
            (ProviderFamily::Azure, "azure"),
            (ProviderFamily::GithubActions, "github_actions"),
            (ProviderFamily::Codemagic, "codemagic"),
        ];
        for (family, name) in expected {
            assert_eq!(
                ok(serde_json::to_string(&family)),
                format!("\"{name}\""),
                "{family:?} serializes as {name}"
            );
            let reloaded: ProviderFamily = ok(serde_json::from_str(&format!("\"{name}\"")));
            assert_eq!(reloaded, family);
        }
        assert!(
            serde_json::from_str::<ProviderFamily>("\"spaceship\"").is_err(),
            "unknown families are rejected"
        );
        assert_eq!(ProviderFamily::FakeRemote.kind(), "flauz-lab-fake-remote");
        assert_eq!(ProviderFamily::GithubActions.kind(), "github_actions");
    }

    #[test]
    fn outcomes_and_observations_serialize_canonically() {
        assert_eq!(
            ok(serde_json::to_string(&ActionOutcome::Applied)),
            "{\"kind\":\"applied\"}"
        );
        assert_eq!(
            ok(serde_json::to_string(&ActionOutcome::NoVisibleChange)),
            "{\"kind\":\"no_visible_change\"}"
        );
        let rejected = ActionOutcome::Rejected {
            reason: "the fake surface has no anchor named missing".to_owned(),
        };
        assert_eq!(
            ok(serde_json::to_string(&rejected)),
            concat!(
                "{\"kind\":\"rejected\",",
                "\"reason\":\"the fake surface has no anchor named missing\"}"
            )
        );
        assert!(
            ActionOutcome::Rejected {
                reason: String::new()
            }
            .validate()
            .is_err()
        );
        assert!(
            ActionOutcome::Rejected {
                reason: "x".repeat(MAX_EXPLANATION_BYTES + 1)
            }
            .validate()
            .is_err()
        );

        let focus = ObservedState::Focus { anchor: None };
        assert_eq!(
            ok(serde_json::to_string(&focus)),
            "{\"kind\":\"focus\",\"anchor\":null}"
        );
        let text = ObservedState::Text {
            anchor: anchor("task-composer"),
            text: String::new(),
        };
        let reloaded: ObservedState = ok(serde_json::from_str(&ok(serde_json::to_string(&text))));
        assert_eq!(reloaded, text);
        let cycle = ObservedState::FocusCycle {
            anchors: vec![anchor("modal-cancel-button")],
        };
        ok(cycle.validate());
        assert!(
            ObservedState::FocusCycle { anchors: vec![] }
                .validate()
                .is_err(),
            "an empty focus cycle is invalid"
        );
    }

    #[test]
    fn records_round_trip_and_reject_inconsistent_shapes() {
        let record = sample_record();
        let serialized = ok(serde_json::to_string_pretty(&record));
        let reloaded: EvidenceRecord = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, record);
        ok(record.validate());

        // Duplicated step ids and unsorted frame anchors are rejected.
        let duplicated = EvidenceRecord {
            steps: vec![record.steps[0].clone(), record.steps[0].clone()],
            ..record.clone()
        };
        assert!(duplicated.validate().is_err());
        let frame_result = ProbeResult::Frame {
            moment: ok(FrameMoment::parse("picker-open")),
            frame: ok(FrameRef::parse("frame-0001")),
            digest: FrameDigest::from_fnv1a(crate::refs::fnv1a64(b"picker-open")),
            anchors: vec![
                ok(AnchorObservation::new("model-picker-panel", "open-empty")),
                ok(AnchorObservation::new("agents-view-panel", "closed")),
            ],
            vlm: None,
        };
        assert!(
            frame_result.validate().is_err(),
            "frame anchors must be sorted by anchor id"
        );
        let unknown_field = serialized.replace("\"run\":", "\"runner\":");
        assert!(
            serde_json::from_str::<EvidenceRecord>(&unknown_field).is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn observed_findings_union_deduplicates_and_sorts() {
        let first = ok(AnchorObservation::new("model-picker-panel", "open-empty"));
        let second = ok(AnchorObservation::new("agents-view-panel", "closed"));
        let record = ok(EvidenceRecord::new(
            ok(RunId::parse("flauz-lab-local-findings")),
            ok(JourneyId::parse("sample-discovery")),
            local_descriptor(),
            vec![StepEvidence {
                step: ok(StepId::parse("step-a")),
                outcome: ActionOutcome::Applied,
                probes: vec![
                    ProbeEvidence {
                        probe: ok(ProbeId::parse("assert-a")),
                        result: ProbeResult::Assertion {
                            assertion: StateAssertion::AnchorState {
                                anchor: first.anchor.clone(),
                                state: first.state.clone(),
                            },
                            satisfied: true,
                            observed: ObservedState::Anchor {
                                anchor: first.anchor.clone(),
                                state: first.state.clone(),
                            },
                            findings: vec![
                                ok(FindingId::parse("bracket-swap-chords")),
                                ok(FindingId::parse("pty-focus-transfer")),
                            ],
                        },
                    },
                    ProbeEvidence {
                        probe: ok(ProbeId::parse("assert-b")),
                        result: ProbeResult::Assertion {
                            assertion: StateAssertion::AnchorState {
                                anchor: second.anchor.clone(),
                                state: second.state.clone(),
                            },
                            satisfied: false,
                            observed: ObservedState::Anchor {
                                anchor: second.anchor.clone(),
                                state: second.state.clone(),
                            },
                            findings: vec![ok(FindingId::parse("pty-focus-transfer"))],
                        },
                    },
                ],
            }],
        ));
        assert_eq!(
            record
                .observed_findings()
                .iter()
                .map(|finding| finding.as_str().to_owned())
                .collect::<Vec<String>>(),
            vec!["bracket-swap-chords", "pty-focus-transfer"]
        );
    }
}
