//! The declarative journey spec (LAB-001, Wave-4 kernel addendum §4): UI
//! actions as data, probes at named moments, the surfaces exercised, and
//! the product journeys the lab journey maps to.
//!
//! A [`JourneySpec`] is authored ONCE and runs unchanged against every
//! compliant lab adapter: it contains no provider state, no timing, no
//! coordinates — a click targets a NAMED anchor (adapters resolve anchors
//! provider-side), a frame is captured at a NAMED moment, and every list
//! is canonically ordered so the spec's canonical JSON is byte-stable.
//!
//! The action and assertion vocabularies are the Lead's d-series scene
//! shapes re-exported as data: key chords (`ctrl+alt+shift+m` opens the
//! model picker), clicks at named affordances, typed text that must land
//! (`TextLanded`, the N1/017 focus-probe family), scoped-Escape focus
//! restoration (`FocusOn`, the 017 law), modal trap ladders
//! (`FocusConfined`, the d19 contract) and chord ladders (the N6
//! bracket-swap + shifted-symbol companion family).

use serde::{Deserialize, Serialize};

use crate::refs::{
    AnchorId, Chord, FrameMoment, JourneyId, JourneyRef, ProbeId, StateKey, StepId, SurfaceId,
};
use crate::{
    CapabilityKey, LabError, LabVersion, MAX_JOURNEY_REFS, MAX_PROBES_PER_STEP, MAX_STEPS,
    MAX_SURFACES, MAX_TEXT_BYTES, ensure_list_bound, ensure_non_empty, ensure_sorted_unique,
    ensure_str_bound,
};

/// One UI action, as data (addendum §4): a key chord, a click at a named
/// anchor, or typed text. No coordinates, no timing, no provider state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum JourneyAction {
    /// Press a canonical key chord (for example `ctrl+alt+shift+m`).
    KeyChord {
        /// The chord.
        chord: Chord,
    },
    /// Click at a NAMED anchor — adapters resolve the anchor to their
    /// provider's coordinates; the spec never carries pixels or geometry.
    ClickAnchor {
        /// The named anchor to click.
        anchor: AnchorId,
    },
    /// Type text into the currently focused text surface.
    TypeText {
        /// The text to type (bounded).
        text: String,
    },
}

impl JourneyAction {
    /// Builds a key-chord action, validating the chord.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the chord is outside the grammar.
    pub fn key_chord(chord: &str) -> Result<Self, LabError> {
        Ok(Self::KeyChord {
            chord: Chord::parse(chord)?,
        })
    }

    /// Builds a click action at a named anchor, validating the anchor.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the anchor is outside the grammar.
    pub fn click_anchor(anchor: &str) -> Result<Self, LabError> {
        Ok(Self::ClickAnchor {
            anchor: AnchorId::parse(anchor)?,
        })
    }

    /// Builds a type-text action, validating the text's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the text is empty or oversized.
    pub fn type_text(text: &str) -> Result<Self, LabError> {
        ensure_non_empty("typed text", text)?;
        ensure_str_bound("typed text", text, MAX_TEXT_BYTES)?;
        Ok(Self::TypeText {
            text: text.to_owned(),
        })
    }

    /// Validates the action's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any field is outside its bound.
    pub fn validate(&self) -> Result<(), LabError> {
        match self {
            Self::KeyChord { chord } => chord.validate(),
            Self::ClickAnchor { anchor } => anchor.validate(),
            Self::TypeText { text } => {
                ensure_non_empty("typed text", text)?;
                ensure_str_bound("typed text", text, MAX_TEXT_BYTES)
            }
        }
    }
}

/// One state assertion, as data — the F1 accessibility patterns expressed
/// structurally so evidence can carry them and the comparator can diff
/// them:
///
/// - [`StateAssertion::FocusOn`] — the focus-order family (the 017
///   palette/overlay-close restoration; the N1 composer-focus contract;
///   the N5 PTY-focus transfer);
/// - [`StateAssertion::FocusConfined`] — the d19 modal trap family (tab
///   cycles within exactly these anchors);
/// - [`StateAssertion::TextLanded`] — the typed-probe family (the N1/017
///   evidence: what was typed lands verbatim in the anchor);
/// - [`StateAssertion::AnchorState`] — the general named-anchor state
///   (a panel is open/closed, the honest empty state, the selection).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum StateAssertion {
    /// A named anchor is in a named state.
    AnchorState {
        /// The anchor.
        anchor: AnchorId,
        /// The expected state.
        state: StateKey,
    },
    /// Keyboard focus rests on the anchor.
    FocusOn {
        /// The anchor focus must rest on.
        anchor: AnchorId,
    },
    /// Tabbing cycles within exactly these anchors, in order — the modal
    /// focus-trap contract (the d19 family).
    FocusConfined {
        /// The anchors the tab cycle must be confined to.
        anchors: Vec<AnchorId>,
    },
    /// The text landed verbatim in the anchor (the typed-probe family).
    TextLanded {
        /// The text-bearing anchor.
        anchor: AnchorId,
        /// The exact text expected in the anchor.
        text: String,
    },
}

impl StateAssertion {
    /// Validates the assertion's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any field is empty, oversized or an
    /// anchor list is empty/duplicated/unsorted.
    pub fn validate(&self) -> Result<(), LabError> {
        match self {
            Self::AnchorState { anchor, state } => {
                anchor.validate()?;
                state.validate()
            }
            Self::FocusOn { anchor } => anchor.validate(),
            Self::FocusConfined { anchors } => {
                ensure_list_bound("focus confinement anchors", anchors.len(), MAX_SURFACES)?;
                if anchors.is_empty() {
                    return Err(LabError::invalid(
                        "a focus-confinement assertion names at least one anchor",
                    ));
                }
                ensure_sorted_unique("focus confinement anchors", anchors)
            }
            Self::TextLanded { anchor, text } => {
                anchor.validate()?;
                ensure_non_empty("landed text", text)?;
                ensure_str_bound("landed text", text, MAX_TEXT_BYTES)
            }
        }
    }
}

/// One probe of a journey step, as data: a frame capture at a named
/// moment, or a state assertion.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum JourneyProbe {
    /// Capture a frame at a named moment.
    FrameCapture {
        /// The named moment (for example `picker-open`).
        moment: FrameMoment,
    },
    /// Assert a state.
    StateAssertion {
        /// The assertion.
        assertion: StateAssertion,
    },
}

impl JourneyProbe {
    /// Builds a frame-capture probe, validating the moment.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the moment is outside the grammar.
    pub fn frame_capture(moment: &str) -> Result<Self, LabError> {
        Ok(Self::FrameCapture {
            moment: FrameMoment::parse(moment)?,
        })
    }

    /// Builds a state-assertion probe, validating the assertion.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the assertion fails validation.
    pub fn state_assertion(assertion: StateAssertion) -> Result<Self, LabError> {
        assertion.validate()?;
        Ok(Self::StateAssertion { assertion })
    }

    /// Validates the probe's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any field is outside its bound.
    pub fn validate(&self) -> Result<(), LabError> {
        match self {
            Self::FrameCapture { moment } => moment.validate(),
            Self::StateAssertion { assertion } => assertion.validate(),
        }
    }
}

/// One journey step: an action plus the probes that run after it. The
/// step's position in the spec's step list is its canonical execution
/// order; step ids are unique; probes are canonically ordered by id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JourneyStep {
    /// The step's identifier (unique within the journey; for example
    /// `open-picker`).
    pub id: StepId,
    /// The action the step performs.
    pub action: JourneyAction,
    /// The probes that run after the action (in order; unique ids within
    /// the step).
    pub probes: Vec<ProbeSpec>,
}

/// A journey step's probe, with its spec-side identity: the probe id the
/// evidence references and the probe itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeSpec {
    /// The probe's identifier (unique within the journey; for example
    /// `assert-picker-open`).
    pub id: ProbeId,
    /// The probe.
    pub probe: JourneyProbe,
}

impl JourneyStep {
    /// Builds a step, canonically ordering its probes by id and
    /// validating bounds and probe-id uniqueness. The step's position in
    /// the spec's step list is its execution order; probe order within a
    /// step is the canonical id order (every probe observes the same
    /// post-action state, so the id order is the deterministic order).
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the id, action or probes fail
    /// validation, the probe list is oversized, or probe ids repeat.
    pub fn new(
        id: &str,
        action: JourneyAction,
        mut probes: Vec<ProbeSpec>,
    ) -> Result<Self, LabError> {
        let id = StepId::parse(id)?;
        action.validate()?;
        ensure_list_bound("step probes", probes.len(), MAX_PROBES_PER_STEP)?;
        probes.sort_by(|left, right| left.id.cmp(&right.id));
        let ids: Vec<&ProbeId> = probes.iter().map(|probe| &probe.id).collect();
        ensure_sorted_unique("step probe ids", &ids)?;
        for probe in &probes {
            probe.probe.validate()?;
        }
        Ok(Self { id, action, probes })
    }

    /// Validates the step's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the step fails validation.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::new(self.id.as_str(), self.action.clone(), self.probes.clone())?;
        Ok(())
    }
}

/// The declarative journey spec (addendum §4): steps with their probes,
/// the UI surfaces exercised, and the product journeys (J-ids) the lab
/// journey maps to — all as data, canonically ordered, byte-stable under
/// canonical JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JourneySpec {
    /// Contract schema version (`"v": 1`).
    pub v: LabVersion,
    /// The journey's identifier (for example `shell-discovery`).
    pub id: JourneyId,
    /// The journey's human title (bounded).
    pub title: String,
    /// The UI surfaces the journey exercises (sorted, deduplicated).
    pub surfaces: Vec<SurfaceId>,
    /// The product journeys (J-ids) this lab journey maps to (sorted,
    /// deduplicated — data, per PRODUCT-UX-JOURNEYS §3).
    pub journeys: Vec<JourneyRef>,
    /// The journey's steps, in canonical execution order (unique ids).
    pub steps: Vec<JourneyStep>,
}

impl JourneySpec {
    /// Builds a journey spec, validating every canonical rule: sorted
    /// deduplicated surfaces and journey references, bounded step list
    /// with unique step ids, and unique probe ids within each step.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any canonical rule fails.
    pub fn new(
        id: &str,
        title: &str,
        surfaces: Vec<SurfaceId>,
        journeys: Vec<JourneyRef>,
        steps: Vec<JourneyStep>,
    ) -> Result<Self, LabError> {
        let id = JourneyId::parse(id)?;
        ensure_non_empty("journey title", title)?;
        ensure_str_bound("journey title", title, crate::MAX_NAME_BYTES)?;
        ensure_list_bound("journey surfaces", surfaces.len(), MAX_SURFACES)?;
        ensure_sorted_unique("journey surfaces", &surfaces)?;
        ensure_list_bound("journey references", journeys.len(), MAX_JOURNEY_REFS)?;
        ensure_sorted_unique("journey references", &journeys)?;
        ensure_list_bound("journey steps", steps.len(), MAX_STEPS)?;
        if steps.is_empty() {
            return Err(LabError::invalid("a journey carries at least one step"));
        }
        // Step ids are unique but NOT id-sorted: the step list's order is
        // the canonical EXECUTION order (the spec is the order
        // authority).
        let step_ids: Vec<&StepId> = steps.iter().map(|step| &step.id).collect();
        if step_ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(LabError::invalid("journey step ids must be unique"));
        }
        for step in &steps {
            step.validate()?;
        }
        Ok(Self {
            v: LabVersion,
            id,
            title: title.to_owned(),
            surfaces,
            journeys,
            steps,
        })
    }

    /// Validates the spec against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any canonical rule fails.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::new(
            self.id.as_str(),
            &self.title,
            self.surfaces.clone(),
            self.journeys.clone(),
            self.steps.clone(),
        )?;
        Ok(())
    }

    /// Finds a step by id.
    #[must_use]
    pub fn step(&self, id: &StepId) -> Option<&JourneyStep> {
        self.steps.iter().find(|step| &step.id == id)
    }
}

/// Computes the lab capabilities a journey requires (the frozen
/// capability-key grammar): every key chord and typed text requires
/// `computer.keyboard`, every click at a named anchor requires
/// `computer.mouse`, and every frame capture requires `computer.screen`.
/// State assertions ride the normalized observation channel and require
/// no additional surface. The result is sorted and deduplicated — the
/// input of the adapter's named-gap check on `prepare`.
///
/// # Errors
///
/// Returns [`LabError`] when a required capability key fails the frozen
/// grammar (a bug in the caller's data, named in the error).
pub fn required_capabilities(journey: &JourneySpec) -> Result<Vec<CapabilityKey>, LabError> {
    let mut required: Vec<CapabilityKey> = Vec::new();
    let mut push = |key: &str| -> Result<(), LabError> {
        let key = CapabilityKey::parse(key)?;
        if !required.contains(&key) {
            required.push(key);
        }
        Ok(())
    };
    for step in &journey.steps {
        match &step.action {
            JourneyAction::KeyChord { .. } | JourneyAction::TypeText { .. } => {
                push("computer.keyboard")?;
            }
            JourneyAction::ClickAnchor { .. } => {
                push("computer.mouse")?;
            }
        }
        for probe in &step.probes {
            if matches!(probe.probe, JourneyProbe::FrameCapture { .. }) {
                push("computer.screen")?;
            }
        }
    }
    required.sort();
    Ok(required)
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

    fn anchor(id: &str) -> AnchorId {
        ok(AnchorId::parse(id))
    }

    fn probe(id: &str, probe: JourneyProbe) -> ProbeSpec {
        ProbeSpec {
            id: ok(ProbeId::parse(id)),
            probe,
        }
    }

    fn sample_journey() -> JourneySpec {
        ok(JourneySpec::new(
            "sample-discovery",
            "A sample shell-discovery journey",
            vec![ok(SurfaceId::parse("task-surface"))],
            vec![ok(JourneyRef::parse("J-01"))],
            vec![
                ok(JourneyStep::new(
                    "anchor-task",
                    ok(JourneyAction::key_chord("ctrl+n")),
                    vec![
                        probe(
                            "frame-composer-open",
                            ok(JourneyProbe::frame_capture("composer-open")),
                        ),
                        probe(
                            "assert-composer-open",
                            ok(JourneyProbe::state_assertion(StateAssertion::AnchorState {
                                anchor: anchor("new-task-composer"),
                                state: ok(StateKey::parse("open")),
                            })),
                        ),
                    ],
                )),
                ok(JourneyStep::new(
                    "submit-task",
                    ok(JourneyAction::key_chord("ctrl+return")),
                    vec![probe(
                        "assert-focus-on-composer",
                        ok(JourneyProbe::state_assertion(StateAssertion::FocusOn {
                            anchor: anchor("task-composer"),
                        })),
                    )],
                )),
            ],
        ))
    }

    #[test]
    fn actions_serialize_internally_tagged() {
        let chord = ok(JourneyAction::key_chord("ctrl+alt+shift+m"));
        assert_eq!(
            ok(serde_json::to_string(&chord)),
            "{\"kind\":\"key_chord\",\"chord\":\"ctrl+alt+shift+m\"}"
        );
        let click = ok(JourneyAction::click_anchor("clear-history-affordance"));
        assert_eq!(
            ok(serde_json::to_string(&click)),
            "{\"kind\":\"click_anchor\",\"anchor\":\"clear-history-affordance\"}"
        );
        let typed = ok(JourneyAction::type_text("the lab anchor task"));
        assert_eq!(
            ok(serde_json::to_string(&typed)),
            "{\"kind\":\"type_text\",\"text\":\"the lab anchor task\"}"
        );
        assert!(
            serde_json::from_str::<JourneyAction>("{\"kind\":\"scroll\",\"anchor\":\"list\"}")
                .is_err(),
            "unknown action kinds are rejected"
        );
        assert!(
            serde_json::from_str::<JourneyAction>(
                "{\"kind\":\"click_anchor\",\"anchor\":\"list\",\"coords\":[1,2]}"
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn assertions_serialize_the_f1_families_as_data() {
        let focus = StateAssertion::FocusOn {
            anchor: anchor("task-composer"),
        };
        assert_eq!(
            ok(serde_json::to_string(&focus)),
            "{\"kind\":\"focus_on\",\"anchor\":\"task-composer\"}"
        );
        let confined = StateAssertion::FocusConfined {
            anchors: vec![
                anchor("modal-cancel-button"),
                anchor("modal-confirm-button"),
            ],
        };
        let serialized = ok(serde_json::to_string(&confined));
        assert_eq!(
            serialized,
            concat!(
                "{\"kind\":\"focus_confined\",\"anchors\":",
                "[\"modal-cancel-button\",\"modal-confirm-button\"]}"
            )
        );
        let landed = StateAssertion::TextLanded {
            anchor: anchor("task-composer"),
            text: "focusprobe-after-palette-close".to_owned(),
        };
        assert_eq!(
            ok(serde_json::to_string(&landed)),
            concat!(
                "{\"kind\":\"text_landed\",\"anchor\":\"task-composer\",",
                "\"text\":\"focusprobe-after-palette-close\"}"
            )
        );
        let state = StateAssertion::AnchorState {
            anchor: anchor("model-picker-panel"),
            state: ok(StateKey::parse("open-empty")),
        };
        assert_eq!(
            ok(serde_json::to_string(&state)),
            "{\"kind\":\"anchor_state\",\"anchor\":\"model-picker-panel\",\"state\":\"open-empty\"}"
        );
        for assertion in [&focus, &confined, &landed, &state] {
            ok(assertion.validate());
            let reloaded: StateAssertion =
                ok(serde_json::from_str(&ok(serde_json::to_string(assertion))));
            assert_eq!(reloaded, *assertion);
        }
        // The trap family is order-canonical: unsorted or duplicated
        // confinement anchors are rejected.
        assert!(
            StateAssertion::FocusConfined {
                anchors: vec![
                    anchor("modal-confirm-button"),
                    anchor("modal-cancel-button")
                ],
            }
            .validate()
            .is_err()
        );
        assert!(
            StateAssertion::FocusConfined { anchors: vec![] }
                .validate()
                .is_err()
        );
    }

    #[test]
    fn journeys_round_trip_and_reject_inconsistent_shapes() {
        let journey = sample_journey();
        let serialized = ok(serde_json::to_string_pretty(&journey));
        let reloaded: JourneySpec = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, journey);
        assert_eq!(reloaded.v, LabVersion);

        // Unsorted surfaces, unsorted step ids and duplicated probe ids
        // are all rejected.
        assert!(
            JourneySpec::new(
                "sample",
                "title",
                vec![
                    ok(SurfaceId::parse("task-surface")),
                    ok(SurfaceId::parse("command-palette"))
                ],
                vec![],
                vec![ok(JourneyStep::new(
                    "b",
                    ok(JourneyAction::key_chord("escape")),
                    vec![]
                ))],
            )
            .is_err(),
            "surfaces must be sorted"
        );
        let repeated_step = ok(JourneyStep::new(
            "a",
            ok(JourneyAction::key_chord("escape")),
            vec![],
        ));
        let other_step = ok(JourneyStep::new(
            "a",
            ok(JourneyAction::key_chord("escape")),
            vec![],
        ));
        assert!(
            JourneySpec::new(
                "sample",
                "title",
                vec![],
                vec![],
                vec![repeated_step, other_step],
            )
            .is_err(),
            "step ids must be unique"
        );
        let frame = ok(JourneyProbe::frame_capture("m"));
        assert!(
            JourneyStep::new(
                "a",
                ok(JourneyAction::key_chord("escape")),
                vec![probe("p-one", frame.clone()), probe("p-one", frame),],
            )
            .is_err(),
            "probe ids must be unique within a step"
        );
        assert!(
            JourneySpec::new("sample", "title", vec![], vec![], vec![]).is_err(),
            "a journey carries at least one step"
        );
    }

    #[test]
    fn required_capabilities_follow_the_actions_and_probes() {
        let journey = sample_journey();
        assert_eq!(
            ok(required_capabilities(&journey))
                .iter()
                .map(|key| key.as_str().to_owned())
                .collect::<Vec<String>>(),
            vec!["computer.keyboard", "computer.screen"],
            "the frame capture requires the screen surface; assertions need none"
        );
        let clicking = ok(JourneySpec::new(
            "clicking",
            "A clicking journey",
            vec![],
            vec![],
            vec![ok(JourneyStep::new(
                "open-modal",
                ok(JourneyAction::click_anchor("clear-history-affordance")),
                vec![],
            ))],
        ));
        assert_eq!(
            ok(required_capabilities(&clicking))
                .iter()
                .map(|key| key.as_str().to_owned())
                .collect::<Vec<String>>(),
            vec!["computer.mouse"]
        );
    }
}
