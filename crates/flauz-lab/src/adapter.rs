//! The lab-adapter contract and its providers (LAB-001, Wave-4 kernel
//! addendum §4): [`prepare`](LabAdapter::prepare) →
//! [`run`](LabAdapter::run) → [`capture`](LabAdapter::capture) →
//! [`evidence`](LabAdapter::evidence).
//!
//! The trait is NEW surface owned by this crate — not an
//! `ExecutionProvider` extension. Three implementations ship:
//!
//! - [`LocalLabAdapter`] — the in-repo reference-implementation driver:
//!   the contract plus a deterministic fake
//!   ([`FakeAppSurface`](crate::fakes::FakeAppSurface)) standing in for
//!   the operator's script-driven local lab. The local Linux lab is the
//!   reference provider (addendum §4).
//! - [`FakeRemoteLabAdapter`] — the CONSISTENCY provider (the ENV-001
//!   law: the fake remote every later real remote copies): the same
//!   driver, the same deterministic surface, remote topology metadata
//!   and a provider-local digest namespace — so the same journey bytes
//!   produce schema-comparable evidence from both adapters (the F8 gate
//!   law, proven in-tests). Its remote topology carries the ENV-001
//!   honest gap: no `desktop.gui`.
//! - [`FamilySkeletonAdapter`] — the real-provider adapter families
//!   (E2B/Daytona/Azure/GitHub Actions/Codemagic) as skeletons: pure
//!   topology DATA ([`known_provider_families`]) plus honest named-gap
//!   checks on `prepare` and not-wired refusals on `run`/`capture` —
//!   NO network (real integrations are gated behind the lab proving the
//!   contract first).
//!
//! # The named-gap law on prepare (the CAP-001 law, applied to labs)
//!
//! A journey's required lab capabilities are derived from its actions
//! and probes ([`derive_required_capabilities`]); `prepare` checks them
//! against the adapter's offered surfaces and names every missing key —
//! a journey a provider cannot run is REJECTED with its gaps named,
//! never silently accepted.

use serde::{Deserialize, Serialize};

use crate::evidence::{
    ActionOutcome, AdapterDescriptor, AnchorObservation, EvidenceRecord, Locality, ProbeEvidence,
    ProbeResult, ProviderFamily, StepEvidence, VlmSlot,
};
use crate::fakes::FakeAppSurface;
use crate::journey::{JourneyProbe, JourneySpec};
use crate::refs::{
    CapabilityKey, FrameDigest, FrameRef, ProbeId, RunId, StepId, VlmReadRef, fnv1a64,
};
use crate::{LabError, LabVersion, MAX_LAB_SURFACES, ensure_list_bound, ensure_sorted_unique};

/// The not-wired reason every skeleton adapter returns for real work:
/// real network integrations follow the lab-proven contract (addendum
/// §4) — no network exists in this crate.
pub const FAMILY_SKELETON_NOT_WIRED: &str = "real network integrations are gated behind the lab \
                                             proving the contract first (Wave-4 addendum §4); \
                                             no network exists in this crate";

/// How a provider family's environments are driven: interactively or
/// as batch runs (the source-of-truth provider taxonomy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interaction {
    /// Interactive sandboxes and hosts (keyboard/mouse/screen driven).
    Interactive,
    /// Batch/CI providers (headless run matrices, no interactive UI).
    Batch,
}

/// The maturity of an adapter family this wave.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilyStatus {
    /// The reference provider (the local Linux lab).
    Reference,
    /// The consistency provider (the fake remote — the ENV-001 law).
    Consistency,
    /// A skeleton: adapter-family data + honest not-wired refusals; the
    /// real integration follows the lab-proven contract.
    Skeleton,
}

/// One provider family's topology metadata — pure DATA (addendum §4):
/// the family, its kind label, locality, interaction model, the lab
/// surfaces it will offer, and its status this wave. No credentials ever
/// (addendum §3: provider metadata is topology data).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabProviderFamily {
    /// The family.
    pub family: ProviderFamily,
    /// The family's kind label.
    pub kind: String,
    /// The locality the family runs at.
    pub locality: Locality,
    /// The interaction model.
    pub interaction: Interaction,
    /// The lab surfaces the family offers (sorted, deduplicated).
    pub lab_surfaces: Vec<CapabilityKey>,
    /// The family's status this wave.
    pub status: FamilyStatus,
}

/// The provider-families table: the adapter-family DATA with topology
/// metadata (the serialized form of [`known_provider_families`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderFamilies {
    /// Contract schema version (`"v": 1`).
    pub v: LabVersion,
    /// The families, sorted by kind label.
    pub families: Vec<LabProviderFamily>,
}

impl ProviderFamilies {
    /// Validates the table: every family well-formed, kinds unique and
    /// sorted, surfaces sorted and deduplicated.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), LabError> {
        ensure_list_bound("provider families", self.families.len(), MAX_LAB_SURFACES)?;
        let mut kinds: Vec<&String> = self.families.iter().map(|family| &family.kind).collect();
        let unsorted = kinds.windows(2).any(|pair| pair[0] >= pair[1]);
        let sorted = !unsorted;
        if !sorted {
            return Err(LabError::invalid(
                "provider families must be sorted by kind and free of duplicates",
            ));
        }
        kinds.dedup();
        if kinds.len() != self.families.len() {
            return Err(LabError::invalid("provider family kinds must be unique"));
        }
        for family in &self.families {
            if family.family.kind() != family.kind {
                return Err(LabError::invalid(format!(
                    "family {:?} must carry its kind label {}",
                    family.family, family.kind
                )));
            }
            if family.family == ProviderFamily::Local && family.locality != Locality::Local {
                return Err(LabError::invalid("the local family is local"));
            }
            if family.family != ProviderFamily::Local && family.locality != Locality::Remote {
                return Err(LabError::invalid(format!(
                    "the {family:?} family is remote"
                )));
            }
            ensure_list_bound(
                "family lab surfaces",
                family.lab_surfaces.len(),
                MAX_LAB_SURFACES,
            )?;
            ensure_sorted_unique("family lab surfaces", &family.lab_surfaces)?;
        }
        Ok(())
    }
}

/// The local lab's offered surfaces: the full F1 surface set — terminal,
/// browser, filesystem, git, snapshots, and the interactive UI surface
/// family (keyboard, mouse, screen, desktop.gui).
#[must_use]
pub fn local_lab_surfaces() -> Vec<CapabilityKey> {
    keys(&[
        "browser.input",
        "browser.navigation",
        "computer.keyboard",
        "computer.mouse",
        "computer.screen",
        "desktop.gui",
        "filesystem.read",
        "filesystem.write",
        "git",
        "snapshots",
        "terminal",
    ])
}

/// The fake remote's offered surfaces: the ENV-001 sandbox topology plus
/// the lab input family — NO `desktop.gui` (the honest named gap a plain
/// remote sandbox carries). Frames are captured through the adapter's
/// normalized observation channel (`computer.screen`), not by promising
/// pixel-identical remote rendering.
#[must_use]
pub fn fake_remote_lab_surfaces() -> Vec<CapabilityKey> {
    keys(&[
        "browser.input",
        "browser.navigation",
        "computer.keyboard",
        "computer.mouse",
        "computer.screen",
        "filesystem.read",
        "filesystem.write",
        "git",
        "ports",
        "snapshots",
        "terminal",
    ])
}

/// The provider-families table (addendum §4): local (reference),
/// fake-remote (consistency), and the five real remote families as
/// skeletons — E2B, Daytona, Azure, GitHub Actions, Codemagic — each
/// with its honest topology metadata. Real network integrations follow
/// the lab-proven contract in a later wave.
#[must_use]
pub fn known_provider_families() -> ProviderFamilies {
    let families = vec![
        LabProviderFamily {
            family: ProviderFamily::Azure,
            kind: "azure".to_owned(),
            locality: Locality::Remote,
            interaction: Interaction::Interactive,
            lab_surfaces: keys(&[
                "computer.keyboard",
                "computer.mouse",
                "computer.screen",
                "desktop.gui",
                "filesystem.read",
                "filesystem.write",
                "snapshots",
                "terminal",
            ]),
            status: FamilyStatus::Skeleton,
        },
        LabProviderFamily {
            family: ProviderFamily::Codemagic,
            kind: "codemagic".to_owned(),
            locality: Locality::Remote,
            interaction: Interaction::Batch,
            lab_surfaces: keys(&["filesystem.read", "filesystem.write", "git", "terminal"]),
            status: FamilyStatus::Skeleton,
        },
        LabProviderFamily {
            family: ProviderFamily::Daytona,
            kind: "daytona".to_owned(),
            locality: Locality::Remote,
            interaction: Interaction::Interactive,
            lab_surfaces: keys(&[
                "computer.keyboard",
                "computer.mouse",
                "computer.screen",
                "filesystem.read",
                "filesystem.write",
                "git",
                "ports",
                "snapshots",
                "ssh",
                "terminal",
            ]),
            status: FamilyStatus::Skeleton,
        },
        LabProviderFamily {
            family: ProviderFamily::E2b,
            kind: "e2b".to_owned(),
            locality: Locality::Remote,
            interaction: Interaction::Interactive,
            lab_surfaces: keys(&[
                "computer.keyboard",
                "computer.mouse",
                "computer.screen",
                "filesystem.read",
                "filesystem.write",
                "git",
                "ports",
                "snapshots",
                "terminal",
            ]),
            status: FamilyStatus::Skeleton,
        },
        LabProviderFamily {
            family: ProviderFamily::FakeRemote,
            kind: "flauz-lab-fake-remote".to_owned(),
            locality: Locality::Remote,
            interaction: Interaction::Interactive,
            lab_surfaces: fake_remote_lab_surfaces(),
            status: FamilyStatus::Consistency,
        },
        LabProviderFamily {
            family: ProviderFamily::Local,
            kind: "flauz-lab-local".to_owned(),
            locality: Locality::Local,
            interaction: Interaction::Interactive,
            lab_surfaces: local_lab_surfaces(),
            status: FamilyStatus::Reference,
        },
        LabProviderFamily {
            family: ProviderFamily::GithubActions,
            kind: "github_actions".to_owned(),
            locality: Locality::Remote,
            interaction: Interaction::Batch,
            lab_surfaces: keys(&["filesystem.read", "filesystem.write", "git", "terminal"]),
            status: FamilyStatus::Skeleton,
        },
    ];
    ProviderFamilies {
        v: LabVersion,
        families,
    }
}

fn keys(values: &[&str]) -> Vec<CapabilityKey> {
    values
        .iter()
        .map(|value| {
            CapabilityKey::parse(value).unwrap_or_else(|error| {
                panic!("frozen capability key {value:?} must parse: {error}")
            })
        })
        .collect()
}

/// The provider-neutral lab-adapter contract (addendum §4): prepare the
/// journey, run its steps, capture its probes, return the normalized
/// evidence. This is NEW surface owned by this crate — not an
/// `ExecutionProvider` extension — and it is the contract every later
/// real remote adapter (E2B, Daytona, Azure, GitHub Actions, Codemagic)
/// copies from the fake-remote consistency provider.
pub trait LabAdapter {
    /// The adapter's environment/provider metadata — topology data
    /// only (no credentials).
    fn descriptor(&self) -> AdapterDescriptor;

    /// Prepares a journey run: validates the journey's required lab
    /// capabilities against the adapter's offered surfaces — naming
    /// every missing key (no silent fall-through) — and resets the run
    /// state.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the journey is invalid or requires
    /// surfaces this adapter does not offer (the gaps named).
    fn prepare(&mut self, journey: &JourneySpec) -> Result<(), LabError>;

    /// Runs one step of the prepared journey, returning the action's
    /// normalized outcome.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when no journey is prepared, the step id is
    /// unknown, or the step already ran.
    fn run(&mut self, step: &StepId) -> Result<ActionOutcome, LabError>;

    /// Captures one probe of the prepared journey at its step,
    /// returning the normalized result.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when no journey is prepared, the step or
    /// probe id is unknown, the step has not run, or the probe was
    /// already captured.
    fn capture(&mut self, step: &StepId, probe: &ProbeId) -> Result<ProbeResult, LabError>;

    /// Returns the normalized evidence record of the prepared run.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when no journey is prepared or the run is
    /// incomplete (the first missing step or probe named).
    fn evidence(&self) -> Result<EvidenceRecord, LabError>;
}

/// The shared deterministic run driver behind the local and fake-remote
/// adapters (the consistency law: both providers drive the SAME
/// [`FakeAppSurface`], differ only in provider metadata and the digest
/// namespace). The driver accumulates step evidence in run order and
/// assembles the record in journey order at
/// [`evidence`](LabRunState::evidence) time.
#[derive(Debug, Clone)]
struct LabRunState {
    provider_kind: &'static str,
    family: ProviderFamily,
    locality: Locality,
    lab_surfaces: Vec<CapabilityKey>,
    namespace: &'static str,
    surface: FakeAppSurface,
    journey: Option<JourneySpec>,
    evidence: Vec<StepEvidence>,
    frames: usize,
}

impl LabRunState {
    fn new(
        provider_kind: &'static str,
        family: ProviderFamily,
        locality: Locality,
        lab_surfaces: Vec<CapabilityKey>,
        namespace: &'static str,
    ) -> Self {
        Self {
            provider_kind,
            family,
            locality,
            lab_surfaces,
            namespace,
            surface: FakeAppSurface::new(),
            journey: None,
            evidence: Vec::new(),
            frames: 0,
        }
    }

    fn descriptor(&self) -> AdapterDescriptor {
        AdapterDescriptor {
            provider_kind: self.provider_kind.to_owned(),
            family: self.family,
            locality: self.locality,
            lab_surfaces: self.lab_surfaces.clone(),
        }
    }

    fn prepare(&mut self, journey: &JourneySpec) -> Result<(), LabError> {
        journey.validate()?;
        let required = crate::journey::required_capabilities(journey)?;
        let missing: Vec<String> = required
            .iter()
            .filter(|capability| !self.lab_surfaces.contains(capability))
            .map(|capability| capability.as_str().to_owned())
            .collect();
        if !missing.is_empty() {
            return Err(LabError::invalid(format!(
                "the {} lab adapter does not offer: {}",
                self.provider_kind,
                missing.join(", ")
            )));
        }
        self.surface = FakeAppSurface::new();
        self.journey = Some(journey.clone());
        self.evidence.clear();
        self.frames = 0;
        Ok(())
    }

    fn require_journey(&self) -> Result<&JourneySpec, LabError> {
        self.journey.as_ref().ok_or_else(|| {
            LabError::invalid(format!(
                "the {} lab adapter has no prepared journey",
                self.provider_kind
            ))
        })
    }

    fn run(&mut self, step: &StepId) -> Result<ActionOutcome, LabError> {
        if self.evidence.iter().any(|evidence| &evidence.step == step) {
            return Err(LabError::invalid(format!("step {step} already ran")));
        }
        let action = {
            let journey = self.require_journey()?;
            let spec_step = journey
                .step(step)
                .ok_or_else(|| LabError::invalid(format!("the journey has no step {step}")))?;
            spec_step.action.clone()
        };
        let outcome = self.surface.apply(&action);
        self.evidence.push(StepEvidence {
            step: step.clone(),
            outcome: outcome.clone(),
            probes: Vec::new(),
        });
        Ok(outcome)
    }

    fn capture(&mut self, step: &StepId, probe: &ProbeId) -> Result<ProbeResult, LabError> {
        let (spec_probe, step_index) = {
            let journey = self.require_journey()?;
            let spec_step = journey
                .step(step)
                .ok_or_else(|| LabError::invalid(format!("the journey has no step {step}")))?;
            let spec_probe = spec_step
                .probes
                .iter()
                .find(|spec| &spec.id == probe)
                .ok_or_else(|| LabError::invalid(format!("step {step} has no probe {probe}")))?
                .clone();
            let step_index = self
                .evidence
                .iter()
                .position(|evidence| &evidence.step == step)
                .ok_or_else(|| LabError::invalid(format!("step {step} has not run yet")))?;
            if self.evidence[step_index]
                .probes
                .iter()
                .any(|evidence| &evidence.probe == probe)
            {
                return Err(LabError::invalid(format!(
                    "probe {probe} of step {step} was already captured"
                )));
            }
            (spec_probe, step_index)
        };
        let result = self.observe(&spec_probe)?;
        self.evidence[step_index].probes.push(ProbeEvidence {
            probe: probe.clone(),
            result: result.clone(),
        });
        Ok(result)
    }

    fn observe(&mut self, spec_probe: &crate::journey::ProbeSpec) -> Result<ProbeResult, LabError> {
        let journey_id = self.require_journey()?.id.clone();
        match &spec_probe.probe {
            JourneyProbe::FrameCapture { moment } => {
                self.frames += 1;
                if self.frames > 9_999 {
                    return Err(LabError::invalid(
                        "a run captures at most 9999 frames (the frame-reference bound)",
                    ));
                }
                let frame = FrameRef::parse(&format!("frame-{:04}", self.frames))?;
                let anchors = self.surface.anchors();
                let canonical = anchors
                    .iter()
                    .map(|AnchorObservation { anchor, state }| format!("{}={}", anchor, state))
                    .collect::<Vec<String>>()
                    .join(";");
                let digest = FrameDigest::from_fnv1a(fnv1a64(
                    format!("{}|{}", self.namespace, canonical).as_bytes(),
                ));
                let prompt = format!(
                    "Adjudicate the frame at moment '{}' of journey '{}': list the visible \
                     labeled controls, panels and affordances with their open/closed states.",
                    moment, journey_id
                );
                let vlm = VlmSlot {
                    prompt,
                    read: Some(VlmReadRef::parse(&format!("vlm-{}.json", frame))?),
                };
                Ok(ProbeResult::Frame {
                    moment: moment.clone(),
                    frame,
                    digest,
                    anchors,
                    vlm: Some(vlm),
                })
            }
            JourneyProbe::StateAssertion { assertion } => {
                let (satisfied, observed, findings) = self.surface.observe(assertion);
                Ok(ProbeResult::Assertion {
                    assertion: assertion.clone(),
                    satisfied,
                    observed,
                    findings,
                })
            }
        }
    }

    fn evidence(&self) -> Result<EvidenceRecord, LabError> {
        let journey = self.require_journey()?;
        for step in &journey.steps {
            let step_evidence = self
                .evidence
                .iter()
                .find(|evidence| evidence.step == step.id);
            let missing_probe = step_evidence.and_then(|evidence| {
                step.probes
                    .iter()
                    .find(|spec| !evidence.probes.iter().any(|probe| probe.probe == spec.id))
            });
            if let Some(spec) = missing_probe {
                return Err(LabError::invalid(format!(
                    "step {} is missing probe {}",
                    step.id, spec.id
                )));
            }
            if step_evidence.is_none() {
                return Err(LabError::invalid(format!(
                    "step {} of journey {} has not run",
                    step.id, journey.id
                )));
            }
        }
        // The record's steps are ordered by the journey's canonical step
        // order (the spec is the order authority, not the run sequence).
        let mut steps: Vec<StepEvidence> = Vec::with_capacity(journey.steps.len());
        for step in &journey.steps {
            let step_evidence = self
                .evidence
                .iter()
                .find(|evidence| evidence.step == step.id)
                .ok_or_else(|| {
                    LabError::invalid(format!(
                        "step {} of journey {} has not run",
                        step.id, journey.id
                    ))
                })?;
            let mut ordered = step_evidence.clone();
            ordered.probes = step
                .probes
                .iter()
                .filter_map(|spec| {
                    ordered
                        .probes
                        .iter()
                        .find(|evidence| evidence.probe == spec.id)
                        .cloned()
                })
                .collect();
            steps.push(ordered);
        }
        let run = RunId::parse(&format!("{}-{}", self.provider_kind, journey.id))?;
        EvidenceRecord::new(run, journey.id.clone(), self.descriptor(), steps)
    }
}

/// The in-repo reference-implementation lab adapter (the reference
/// provider, addendum §4): the [`LabAdapter`] contract plus a
/// deterministic fake standing in for the operator's script-driven local
/// lab. The real local driver (the Lead's Xvfb/xdotool/ffmpeg scenes)
/// lands behind the same contract once the fabric is lab-proven; the
/// normalized evidence this adapter produces is byte-identical in shape
/// to what the real driver will archive.
#[derive(Debug, Clone)]
pub struct LocalLabAdapter {
    state: LabRunState,
}

impl LocalLabAdapter {
    /// An empty local adapter with no prepared journey.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: LabRunState::new(
                "flauz-lab-local",
                ProviderFamily::Local,
                Locality::Local,
                local_lab_surfaces(),
                "local",
            ),
        }
    }
}

impl Default for LocalLabAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LabAdapter for LocalLabAdapter {
    fn descriptor(&self) -> AdapterDescriptor {
        self.state.descriptor()
    }

    fn prepare(&mut self, journey: &JourneySpec) -> Result<(), LabError> {
        self.state.prepare(journey)
    }

    fn run(&mut self, step: &StepId) -> Result<ActionOutcome, LabError> {
        self.state.run(step)
    }

    fn capture(&mut self, step: &StepId, probe: &ProbeId) -> Result<ProbeResult, LabError> {
        self.state.capture(step, probe)
    }

    fn evidence(&self) -> Result<EvidenceRecord, LabError> {
        self.state.evidence()
    }
}

/// The fake-remote consistency provider (the ENV-001 law): the fake
/// remote every later real remote copies. Deterministic, in-memory, no
/// network — it drives the SAME surface model as the local adapter under
/// the SAME contract, with remote topology metadata (no `desktop.gui`,
/// the honest named gap) and a provider-local digest namespace, so the
/// same journey bytes produce schema-comparable evidence from both
/// adapters (the F8 gate law).
#[derive(Debug, Clone)]
pub struct FakeRemoteLabAdapter {
    state: LabRunState,
}

impl FakeRemoteLabAdapter {
    /// An empty fake-remote adapter with no prepared journey.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: LabRunState::new(
                "flauz-lab-fake-remote",
                ProviderFamily::FakeRemote,
                Locality::Remote,
                fake_remote_lab_surfaces(),
                "fake-remote",
            ),
        }
    }
}

impl Default for FakeRemoteLabAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LabAdapter for FakeRemoteLabAdapter {
    fn descriptor(&self) -> AdapterDescriptor {
        self.state.descriptor()
    }

    fn prepare(&mut self, journey: &JourneySpec) -> Result<(), LabError> {
        self.state.prepare(journey)
    }

    fn run(&mut self, step: &StepId) -> Result<ActionOutcome, LabError> {
        self.state.run(step)
    }

    fn capture(&mut self, step: &StepId, probe: &ProbeId) -> Result<ProbeResult, LabError> {
        self.state.capture(step, probe)
    }

    fn evidence(&self) -> Result<EvidenceRecord, LabError> {
        self.state.evidence()
    }
}

/// The skeleton adapter for a real provider family (addendum §4): pure
/// topology data plus honest named-gap checks on `prepare` and not-wired
/// refusals on real work — NO network. Preparing a journey the family's
/// topology can host SUCCEEDS (proving the contract fits the family);
/// running it refuses honestly.
#[derive(Debug, Clone)]
pub struct FamilySkeletonAdapter {
    family: LabProviderFamily,
}

impl FamilySkeletonAdapter {
    /// Builds the skeleton adapter for a known family.
    #[must_use]
    pub fn new(family: LabProviderFamily) -> Self {
        Self { family }
    }

    /// The skeleton's family record (topology data).
    #[must_use]
    pub fn family(&self) -> &LabProviderFamily {
        &self.family
    }

    fn not_wired(&self, what: &str) -> LabError {
        LabError::NotWired(format!(
            "the {} lab adapter is a skeleton: {what} ({FAMILY_SKELETON_NOT_WIRED})",
            self.family.kind
        ))
    }
}

impl LabAdapter for FamilySkeletonAdapter {
    fn descriptor(&self) -> AdapterDescriptor {
        AdapterDescriptor {
            provider_kind: self.family.kind.clone(),
            family: self.family.family,
            locality: self.family.locality,
            lab_surfaces: self.family.lab_surfaces.clone(),
        }
    }

    fn prepare(&mut self, journey: &JourneySpec) -> Result<(), LabError> {
        journey.validate()?;
        let required = crate::journey::required_capabilities(journey)?;
        let missing: Vec<String> = required
            .iter()
            .filter(|capability| !self.family.lab_surfaces.contains(capability))
            .map(|capability| capability.as_str().to_owned())
            .collect();
        if !missing.is_empty() {
            return Err(LabError::invalid(format!(
                "the {} lab adapter does not offer: {}",
                self.family.kind,
                missing.join(", ")
            )));
        }
        Ok(())
    }

    fn run(&mut self, _step: &StepId) -> Result<ActionOutcome, LabError> {
        Err(self.not_wired("running a journey step"))
    }

    fn capture(&mut self, _step: &StepId, _probe: &ProbeId) -> Result<ProbeResult, LabError> {
        Err(self.not_wired("capturing a probe"))
    }

    fn evidence(&self) -> Result<EvidenceRecord, LabError> {
        Err(self.not_wired("producing evidence"))
    }
}

/// Derives the lab capabilities a journey requires from its actions and
/// probes (the public form of the prepare-time check): every key chord
/// and typed text requires `computer.keyboard`, every click at a named
/// anchor requires `computer.mouse`, and every frame capture requires
/// `computer.screen`. Sorted and deduplicated.
///
/// # Errors
///
/// Returns [`LabError`] when a derived key fails the frozen grammar
/// (never, for valid journeys — the keys are frozen constants).
pub fn derive_required_capabilities(journey: &JourneySpec) -> Result<Vec<CapabilityKey>, LabError> {
    crate::journey::required_capabilities(journey)
}

/// Drives one adapter through the full four-beat contract — prepare →
/// run every step → capture every probe → evidence — returning the
/// normalized record. The wave-gate harness and the conformance tests
/// use this driver for every adapter uniformly.
///
/// # Errors
///
/// Returns [`LabError`] when any beat fails (named in the error).
pub fn run_journey(
    adapter: &mut dyn LabAdapter,
    journey: &JourneySpec,
) -> Result<EvidenceRecord, LabError> {
    adapter.prepare(journey)?;
    for step in &journey.steps {
        adapter.run(&step.id)?;
        for probe in &step.probes {
            adapter.capture(&step.id, &probe.id)?;
        }
    }
    adapter.evidence()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journey::StateAssertion;
    use crate::refs::JourneyId;

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

    #[test]
    fn the_provider_families_table_is_canonical_topology_data() {
        let table = known_provider_families();
        ok(table.validate());
        assert_eq!(table.families.len(), 7);
        let kinds: Vec<&str> = table
            .families
            .iter()
            .map(|family| family.kind.as_str())
            .collect();
        assert_eq!(
            kinds,
            vec![
                "azure",
                "codemagic",
                "daytona",
                "e2b",
                "flauz-lab-fake-remote",
                "flauz-lab-local",
                "github_actions",
            ],
            "the families are sorted by kind label"
        );
        let serialized = ok(serde_json::to_string_pretty(&table));
        let reloaded: ProviderFamilies = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, table);
        assert!(serialized.contains("\"status\": \"skeleton\""));
        assert!(serialized.contains("\"interaction\": \"batch\""));

        // The honest gap: the fake remote offers no desktop.gui; the
        // local lab and the Azure desktop hosts do.
        let family_of = |family: ProviderFamily| {
            table
                .families
                .iter()
                .find(|entry| entry.family == family)
                .unwrap_or_else(|| panic!("the {family:?} family must be in the table"))
        };
        let fake_remote = family_of(ProviderFamily::FakeRemote);
        assert!(
            !fake_remote
                .lab_surfaces
                .contains(&ok(CapabilityKey::parse("desktop.gui")))
        );
        let local = family_of(ProviderFamily::Local);
        assert!(
            local
                .lab_surfaces
                .contains(&ok(CapabilityKey::parse("desktop.gui")))
        );
        let azure = family_of(ProviderFamily::Azure);
        assert!(
            azure
                .lab_surfaces
                .contains(&ok(CapabilityKey::parse("desktop.gui")))
        );
        // The batch families offer no interactive UI surface.
        for batch_family in [ProviderFamily::GithubActions, ProviderFamily::Codemagic] {
            let batch = family_of(batch_family);
            assert_eq!(batch.interaction, Interaction::Batch);
            assert!(
                !batch
                    .lab_surfaces
                    .contains(&ok(CapabilityKey::parse("computer.keyboard")))
            );
        }
    }

    #[test]
    fn the_local_and_fake_remote_adapters_offer_their_topologies() {
        let local = LocalLabAdapter::new();
        let descriptor = local.descriptor();
        ok(descriptor.validate());
        assert_eq!(descriptor.provider_kind, "flauz-lab-local");
        assert_eq!(descriptor.family, ProviderFamily::Local);
        assert_eq!(descriptor.locality, Locality::Local);
        assert_eq!(descriptor.lab_surfaces, local_lab_surfaces());

        let remote = FakeRemoteLabAdapter::new();
        let descriptor = remote.descriptor();
        ok(descriptor.validate());
        assert_eq!(descriptor.provider_kind, "flauz-lab-fake-remote");
        assert_eq!(descriptor.family, ProviderFamily::FakeRemote);
        assert_eq!(descriptor.locality, Locality::Remote);
        assert_eq!(descriptor.lab_surfaces, fake_remote_lab_surfaces());
    }

    #[test]
    fn the_skeleton_families_are_honest() {
        let table = known_provider_families();
        let shell = crate::fakes::fake_shell_discovery_journey();

        let family_of = |family: ProviderFamily| {
            table
                .families
                .iter()
                .find(|entry| entry.family == family)
                .cloned()
                .unwrap_or_else(|| panic!("the {family:?} family must be in the table"))
        };

        // GitHub Actions cannot host an interactive UI journey: prepare
        // fails with the named missing keys (the CAP-001 law).
        let mut github = FamilySkeletonAdapter::new(family_of(ProviderFamily::GithubActions));
        let error = err(
            github.prepare(&shell),
            "a batch provider cannot host a UI journey",
        );
        assert!(error.reason().contains("computer.keyboard"));
        assert!(error.reason().contains("computer.screen"));

        // Azure CAN host it topologically (interactive, desktop
        // surfaces) — but running refuses honestly: not wired.
        let mut azure = FamilySkeletonAdapter::new(family_of(ProviderFamily::Azure));
        ok(azure.prepare(&shell));
        let step = ok(StepId::parse("anchor-task"));
        let error = err(azure.run(&step), "a skeleton never runs real work");
        assert!(matches!(error, LabError::NotWired(_)));
        assert!(error.reason().contains("azure"));
        assert!(error.reason().contains(FAMILY_SKELETON_NOT_WIRED));
        let error = err(azure.evidence(), "a skeleton never produces evidence");
        assert!(matches!(error, LabError::NotWired(_)));
    }

    #[test]
    fn the_driver_enforces_the_contract() {
        let journey = crate::fakes::fake_shell_discovery_journey();
        let mut adapter = LocalLabAdapter::new();

        // No journey prepared: every beat names it.
        let step = ok(StepId::parse("anchor-task"));
        assert!(adapter.run(&step).is_err());
        let probe = ok(ProbeId::parse("assert-first-run-dismissed"));
        let settle = ok(StepId::parse("settle-first-run"));
        assert!(adapter.capture(&settle, &probe).is_err());
        assert!(adapter.evidence().is_err());

        ok(adapter.prepare(&journey));
        // Unknown step/probe ids are named errors.
        let unknown = ok(StepId::parse("no-such-step"));
        assert!(adapter.run(&unknown).is_err());
        assert!(adapter.capture(&unknown, &probe).is_err());
        // Capturing before the step ran is an error.
        assert!(adapter.capture(&settle, &probe).is_err());
        // Evidence before the run is incomplete: the first missing step
        // is named.
        let error = err(adapter.evidence(), "the run is incomplete");
        assert!(error.reason().contains("settle-first-run"));

        // Run the first step, capture its probe; re-runs and re-captures
        // are errors (determinism).
        ok(adapter.run(&settle));
        ok(adapter.capture(&settle, &probe));
        assert!(adapter.run(&settle).is_err());
        assert!(adapter.capture(&settle, &probe).is_err());

        // A fresh prepare resets the state deterministically.
        ok(adapter.prepare(&journey));
        let record = ok(run_journey(&mut adapter, &journey));
        ok(record.validate());
        assert_eq!(record.run.as_str(), "flauz-lab-local-shell-discovery");
        assert_eq!(record.steps.len(), journey.steps.len());
    }

    #[test]
    fn a_journey_missing_surfaces_is_rejected_with_named_gaps() {
        // A journey whose required surfaces the provider does not offer is
        // REJECTED at prepare with every missing key named — here a
        // keyboard journey on the batch families.
        let table = known_provider_families();
        let github_family = table
            .families
            .iter()
            .find(|family| family.family == ProviderFamily::GithubActions)
            .cloned()
            .unwrap_or_else(|| panic!("the github_actions family must be in the table"));
        let mut github = FamilySkeletonAdapter::new(github_family);
        let journey = ok(crate::journey::JourneySpec::new(
            "batch-incompatible",
            "A journey the batch families cannot host",
            vec![],
            vec![],
            vec![ok(crate::journey::JourneyStep::new(
                "press-escape",
                ok(crate::journey::JourneyAction::key_chord("escape")),
                vec![],
            ))],
        ));
        let error = err(
            github.prepare(&journey),
            "keyboard journeys need computer.keyboard",
        );
        assert!(error.reason().contains("computer.keyboard"));
    }

    #[test]
    fn the_fake_remote_desktop_gui_gap_is_data_not_a_blocker() {
        // The fake remote lacks desktop.gui but still offers the full
        // lab input family — the canonical journeys prepare fine on it
        // (F8), because no journey action requires desktop.gui.
        let mut remote = FakeRemoteLabAdapter::new();
        for journey in [
            crate::fakes::fake_shell_discovery_journey(),
            crate::fakes::fake_a11y_chord_ladder_journey(),
        ] {
            let required = ok(crate::journey::required_capabilities(&journey));
            assert!(!required.contains(&ok(CapabilityKey::parse("desktop.gui"))));
            ok(remote.prepare(&journey));
        }
    }

    #[test]
    fn state_assertions_need_no_lab_surface() {
        // An assertion-only journey requires nothing beyond its actions.
        let journey = ok(crate::journey::JourneySpec::new(
            "assert-only",
            "An assertion-only journey",
            vec![],
            vec![],
            vec![ok(crate::journey::JourneyStep::new(
                "settle",
                ok(crate::journey::JourneyAction::key_chord("escape")),
                vec![crate::journey::ProbeSpec {
                    id: ok(ProbeId::parse("assert-settled")),
                    probe: ok(crate::journey::JourneyProbe::state_assertion(
                        StateAssertion::AnchorState {
                            anchor: ok(crate::refs::AnchorId::parse("first-run-promo-modal")),
                            state: ok(crate::refs::StateKey::parse("closed")),
                        },
                    )),
                }],
            ))],
        ));
        let required = ok(derive_required_capabilities(&journey));
        assert_eq!(
            required
                .iter()
                .map(|key| key.as_str().to_owned())
                .collect::<Vec<String>>(),
            vec!["computer.keyboard"]
        );
    }

    #[test]
    fn the_run_id_is_derived_from_the_provider_and_journey() {
        let mut remote = FakeRemoteLabAdapter::new();
        let journey = crate::fakes::fake_shell_discovery_journey();
        let record = ok(run_journey(&mut remote, &journey));
        assert_eq!(record.run.as_str(), "flauz-lab-fake-remote-shell-discovery");
        let id: JourneyId = ok(JourneyId::parse("shell-discovery"));
        assert_eq!(record.journey, id);
    }
}
