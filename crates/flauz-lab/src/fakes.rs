//! Public in-memory fakes: the deterministic conformance surface for the
//! lab fabric (kernel §7).
//!
//! Everything here is a pure function of its arguments — no I/O, no
//! wall-clock reads, no randomness, no generated identifiers. The
//! [`FakeAppSurface`] is the deterministic fake application both lab
//! adapters drive: the in-memory stand-in for the operator's
//! script-driven local lab (the Lead's d-series scenes re-exported as
//! data), and — through the consistency law — the exact behavior the
//! fake-remote adapter reproduces, so the same journey bytes yield the
//! same normalized evidence from both.
//!
//! The fake surface honestly models the F1 findings it carries: the
//! fresh-profile promo modal swallows keyboard input until a defensive
//! Escape (settled first in both canonical journeys), the terminal dock
//! opens a live PTY without transferring focus (N5), the bracket chords
//! do not move the chat selection while Ctrl+PageDown does (N6), tab
//! cycles are confined to a modal's actions while it is open (the d19
//! trap), and the shifted-symbol companion chord opens the same panel as
//! its digit chord (the N6 family). The two canonical fake journeys —
//! [`fake_shell_discovery_journey`] (the d24 model-picker shape) and
//! [`fake_a11y_chord_ladder_journey`] (the d25/N5/N6/d19 ladder) —
//! exercise exactly those shapes.

use crate::LabError;
use crate::evidence::{ActionOutcome, AnchorObservation, ObservedState};
use crate::journey::{
    JourneyAction, JourneyProbe, JourneySpec, JourneyStep, ProbeSpec, StateAssertion,
};
use crate::refs::{AnchorId, FindingId, JourneyRef, StateKey, SurfaceId};

/// The shell-discovery journey's id (the d24 model-picker scene shape).
pub const FAKE_SHELL_DISCOVERY_ID: &str = "shell-discovery";

/// The a11y chord-ladder journey's id (the d25/N5/N6/d19 shapes).
pub const FAKE_A11Y_CHORD_LADDER_ID: &str = "a11y-chord-ladder";

/// The anchor the fresh-profile promo modal occupies (swallowing
/// keyboard input until the defensive Escape — the rc.14 lesson).
const PROMO_ANCHOR: &str = "first-run-promo-modal";

/// The base surface focus lands on after the promo settles.
const ENTRY_ANCHOR: &str = "entry-surface";

/// The new-task composer (the ctrl+n overlay).
const NEW_TASK_COMPOSER: &str = "new-task-composer";

/// The live task surface's composer (where focus must return after
/// scoped closes — the 017 law).
const TASK_COMPOSER: &str = "task-composer";

/// The task surface.
const TASK_SURFACE: &str = "task-surface";

/// The model picker panel (the honest empty state: `open-empty` — no
/// models configured).
const MODEL_PICKER_PANEL: &str = "model-picker-panel";

/// The unified command palette.
const COMMAND_PALETTE: &str = "command-palette";

/// The palette's input.
const PALETTE_INPUT: &str = "palette-input";

/// The palette's model-picker row (visible when the query matches).
const PALETTE_ROW: &str = "palette-row-model-picker";

/// The agents view panel.
const AGENTS_VIEW_PANEL: &str = "agents-view-panel";

/// The terminal dock (a live PTY when open).
const TERMINAL_DOCK: &str = "terminal-dock";

/// The PTY pane inside the terminal dock (where focus does NOT transfer
/// on open — N5).
const TERMINAL_PTY: &str = "terminal-pty";

/// The clear-confirmation modal.
const CLEAR_MODAL: &str = "clear-confirmation-modal";

/// The clear-history affordance that opens the modal.
const CLEAR_HISTORY_AFFORDANCE: &str = "clear-history-affordance";

/// The modal's cancel action.
const MODAL_CANCEL: &str = "modal-cancel-button";

/// The modal's confirm action.
const MODAL_CONFIRM: &str = "modal-confirm-button";

/// The chat-list selection anchor.
const CHAT_SELECTION: &str = "chat-selection";

/// The seeded chat.
const CHAT_ONE: &str = "chat-one";

/// The chat the anchored task creates.
const CHAT_TWO: &str = "chat-two";

/// The model-picker palette query that must reveal the row.
const PALETTE_QUERY: &str = "choose a model";

/// The deterministic fake application surface both lab adapters drive:
/// the Lead's d-series scene shapes as an in-memory state machine. Pure
/// data + deterministic transitions — no I/O, no timing, no
/// provider-specific behavior (the consistency law: local and fake
/// remote run the SAME surface model).
///
/// The tracked anchor set (what frames observe, always sorted) is the
/// ten named anchors the canonical journeys exercise:
/// `agents-view-panel`, `chat-selection`, `clear-confirmation-modal`,
/// `command-palette`, `first-run-promo-modal`, `model-picker-panel`,
/// `new-task-composer`, `palette-row-model-picker`, `task-surface`,
/// `terminal-dock`.
#[derive(Debug, Clone)]
pub struct FakeAppSurface {
    promo_open: bool,
    focus: AnchorId,
    focus_returns: Vec<AnchorId>,
    new_task_composer: bool,
    task_active: bool,
    model_picker: bool,
    palette: bool,
    palette_query: String,
    agents_view: bool,
    terminal: bool,
    clear_modal: bool,
    chats: usize,
    selected: usize,
    new_task_text: String,
    task_composer_text: String,
    last_action: Option<JourneyAction>,
}

impl Default for FakeAppSurface {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeAppSurface {
    /// A fresh profile: the first-run promo modal is open (swallowing
    /// keyboard input until the defensive Escape), the seeded chat is
    /// selected, nothing else is open.
    #[must_use]
    pub fn new() -> Self {
        Self {
            promo_open: true,
            focus: parse_anchor(PROMO_ANCHOR),
            focus_returns: Vec::new(),
            new_task_composer: false,
            task_active: false,
            model_picker: false,
            palette: false,
            palette_query: String::new(),
            agents_view: false,
            terminal: false,
            clear_modal: false,
            chats: 1,
            selected: 0,
            new_task_text: String::new(),
            task_composer_text: String::new(),
            last_action: None,
        }
    }

    /// Applies one journey action, returning the normalized outcome.
    /// Deterministic: the same action sequence always produces the same
    /// states and outcomes.
    pub fn apply(&mut self, action: &JourneyAction) -> ActionOutcome {
        self.last_action = Some(action.clone());
        match action {
            JourneyAction::KeyChord { chord } => self.apply_chord(chord.as_str()),
            JourneyAction::ClickAnchor { anchor } => self.apply_click(anchor),
            JourneyAction::TypeText { text } => self.apply_text(text),
        }
    }

    /// Applies a key chord.
    fn apply_chord(&mut self, chord: &str) -> ActionOutcome {
        // The first-run promo modal swallows ALL keyboard input until the
        // defensive Escape (the rc.14 lesson, honestly modeled).
        if self.promo_open {
            if chord == "escape" {
                self.promo_open = false;
                self.focus = parse_anchor(ENTRY_ANCHOR);
                return ActionOutcome::Applied;
            }
            return ActionOutcome::NoVisibleChange;
        }
        match chord {
            "escape" => self.apply_escape(),
            "ctrl+n" => {
                if self.new_task_composer {
                    return ActionOutcome::NoVisibleChange;
                }
                self.push_focus();
                self.new_task_composer = true;
                self.focus = parse_anchor(NEW_TASK_COMPOSER);
                ActionOutcome::Applied
            }
            "ctrl+return" => self.apply_submit(),
            "ctrl+k" => {
                if self.palette {
                    return ActionOutcome::NoVisibleChange;
                }
                self.push_focus();
                self.palette = true;
                self.palette_query.clear();
                self.focus = parse_anchor(PALETTE_INPUT);
                ActionOutcome::Applied
            }
            "ctrl+alt+shift+m" => {
                if self.model_picker {
                    return ActionOutcome::NoVisibleChange;
                }
                self.push_focus();
                self.model_picker = true;
                self.focus = parse_anchor(MODEL_PICKER_PANEL);
                ActionOutcome::Applied
            }
            "ctrl+alt+shift+7" | "alt+^" => {
                // The digit chord and its shifted-symbol companion open
                // the same panel (the N6 family discipline: both forms
                // must stay effective).
                if self.agents_view {
                    return ActionOutcome::NoVisibleChange;
                }
                self.agents_view = true;
                ActionOutcome::Applied
            }
            "ctrl+shift+]" | "ctrl+shift+[" => {
                // N6, honestly modeled: the bracket chords produce no
                // selection move in this configuration.
                ActionOutcome::NoVisibleChange
            }
            "ctrl+page_down" => {
                if self.chats < 2 {
                    return ActionOutcome::NoVisibleChange;
                }
                self.selected = (self.selected + 1) % self.chats;
                ActionOutcome::Applied
            }
            "ctrl+`" => {
                if self.terminal {
                    return ActionOutcome::NoVisibleChange;
                }
                // N5, honestly modeled: the dock opens a live PTY but
                // focus does NOT transfer to it.
                self.terminal = true;
                ActionOutcome::Applied
            }
            "tab" | "shift+tab" => {
                if !self.clear_modal {
                    // The broader surface focus order is an open gap
                    // (the F1 inventory) — tab acts only inside the
                    // evidenced trap family.
                    return ActionOutcome::NoVisibleChange;
                }
                // The d19 trap: tab cycles within the modal's actions.
                if self.focus == parse_anchor(MODAL_CANCEL) {
                    self.focus = parse_anchor(MODAL_CONFIRM);
                } else {
                    self.focus = parse_anchor(MODAL_CANCEL);
                }
                ActionOutcome::Applied
            }
            _ => ActionOutcome::NoVisibleChange,
        }
    }

    /// Applies the scoped Escape (the 017 law: the topmost surface
    /// closes and focus returns to the surface that opened it; nothing
    /// traps).
    fn apply_escape(&mut self) -> ActionOutcome {
        if self.clear_modal {
            self.clear_modal = false;
            self.pop_focus();
            return ActionOutcome::Applied;
        }
        if self.palette {
            self.palette = false;
            self.palette_query.clear();
            self.pop_focus();
            return ActionOutcome::Applied;
        }
        if self.model_picker {
            self.model_picker = false;
            self.pop_focus();
            return ActionOutcome::Applied;
        }
        if self.new_task_composer {
            self.new_task_composer = false;
            self.new_task_text.clear();
            self.pop_focus();
            return ActionOutcome::Applied;
        }
        if self.agents_view {
            self.agents_view = false;
            return ActionOutcome::Applied;
        }
        // Escape does not close the terminal dock (ctrl+j toggles it).
        ActionOutcome::NoVisibleChange
    }

    /// Applies ctrl+return: submit the new task (create chat two, select
    /// it, focus the task composer) or activate the palette's top row
    /// (open the picker).
    fn apply_submit(&mut self) -> ActionOutcome {
        if self.new_task_composer {
            if self.new_task_text.is_empty() {
                return ActionOutcome::NoVisibleChange;
            }
            self.new_task_composer = false;
            self.new_task_text.clear();
            self.task_active = true;
            self.chats = 2;
            self.selected = 1;
            self.focus_returns.clear();
            self.focus = parse_anchor(TASK_COMPOSER);
            return ActionOutcome::Applied;
        }
        if self.palette && self.row_visible() {
            self.palette = false;
            self.palette_query.clear();
            self.pop_focus();
            self.push_focus();
            self.model_picker = true;
            self.focus = parse_anchor(MODEL_PICKER_PANEL);
            return ActionOutcome::Applied;
        }
        ActionOutcome::NoVisibleChange
    }

    /// Applies a click at a named anchor.
    fn apply_click(&mut self, target: &AnchorId) -> ActionOutcome {
        if self.promo_open {
            // The promo modal intercepts pointer input too (its own
            // affordances only).
            return ActionOutcome::NoVisibleChange;
        }
        if target.as_str() == CLEAR_HISTORY_AFFORDANCE {
            if self.clear_modal {
                return ActionOutcome::NoVisibleChange;
            }
            self.push_focus();
            self.clear_modal = true;
            self.focus = parse_anchor(MODAL_CANCEL);
            return ActionOutcome::Applied;
        }
        ActionOutcome::Rejected {
            reason: format!("the fake surface has no anchor named {}", target),
        }
    }

    /// Applies typed text into the focused text surface.
    fn apply_text(&mut self, text: &str) -> ActionOutcome {
        if self.promo_open {
            return ActionOutcome::NoVisibleChange;
        }
        if self.focus == parse_anchor(NEW_TASK_COMPOSER) {
            self.new_task_text.push_str(text);
            return ActionOutcome::Applied;
        }
        if self.focus == parse_anchor(PALETTE_INPUT) {
            self.palette_query.push_str(text);
            return ActionOutcome::Applied;
        }
        if self.focus == parse_anchor(TASK_COMPOSER) {
            self.task_composer_text.push_str(text);
            return ActionOutcome::Applied;
        }
        ActionOutcome::NoVisibleChange
    }

    fn push_focus(&mut self) {
        self.focus_returns.push(self.focus.clone());
    }

    fn pop_focus(&mut self) {
        if let Some(previous) = self.focus_returns.pop() {
            self.focus = previous;
        }
    }

    fn row_visible(&self) -> bool {
        self.palette && self.palette_query.to_lowercase().contains(PALETTE_QUERY)
    }

    /// The normalized frame content: the tracked anchor states, sorted
    /// by anchor — the comparable observation both adapters produce
    /// identically.
    #[must_use]
    pub fn anchors(&self) -> Vec<AnchorObservation> {
        let observations = [
            (
                AGENTS_VIEW_PANEL,
                if self.agents_view { "open" } else { "closed" },
            ),
            (
                CHAT_SELECTION,
                if self.selected == 0 {
                    CHAT_ONE
                } else {
                    CHAT_TWO
                },
            ),
            (
                CLEAR_MODAL,
                if self.clear_modal { "open" } else { "closed" },
            ),
            (
                COMMAND_PALETTE,
                if self.palette { "open" } else { "closed" },
            ),
            (
                PROMO_ANCHOR,
                if self.promo_open { "open" } else { "closed" },
            ),
            (
                MODEL_PICKER_PANEL,
                if self.model_picker {
                    "open-empty"
                } else {
                    "closed"
                },
            ),
            (
                NEW_TASK_COMPOSER,
                if self.new_task_composer {
                    "open"
                } else {
                    "closed"
                },
            ),
            (
                PALETTE_ROW,
                if self.row_visible() {
                    "visible"
                } else {
                    "hidden"
                },
            ),
            (
                TASK_SURFACE,
                if self.task_active {
                    "active"
                } else {
                    "inactive"
                },
            ),
            (
                TERMINAL_DOCK,
                if self.terminal {
                    "open-live-pty"
                } else {
                    "closed"
                },
            ),
        ];
        let mut anchors: Vec<AnchorObservation> = observations
            .iter()
            .map(|(anchor, state)| observation(anchor, state))
            .collect::<Result<_, _>>()
            .unwrap_or_else(|error| panic!("the fake anchor set is canonically valid: {error}"));
        anchors.sort_by(|left, right| left.anchor.cmp(&right.anchor));
        anchors
    }

    /// Observes one state assertion: the verdict, the structured
    /// observation and the known-findings links (by id) the observation
    /// deterministically relates to.
    #[must_use]
    pub fn observe(&self, assertion: &StateAssertion) -> (bool, ObservedState, Vec<FindingId>) {
        match assertion {
            StateAssertion::AnchorState { anchor, state } => {
                let actual = self
                    .anchors()
                    .into_iter()
                    .find(|observation| &observation.anchor == anchor)
                    .map(|observation| observation.state)
                    .unwrap_or_else(|| parse_state("unknown"));
                let satisfied = &actual == state;
                let findings = self.finding_links(assertion);
                (
                    satisfied,
                    ObservedState::Anchor {
                        anchor: anchor.clone(),
                        state: actual,
                    },
                    findings,
                )
            }
            StateAssertion::FocusOn { anchor } => {
                let satisfied = &self.focus == anchor;
                let findings = self.finding_links(assertion);
                (
                    satisfied,
                    ObservedState::Focus {
                        anchor: Some(self.focus.clone()),
                    },
                    findings,
                )
            }
            StateAssertion::FocusConfined { anchors } => {
                // The d19 trap: while the modal is open the tab walk is
                // confined to its actions; elsewhere tab is a no-op and
                // the cycle is the single focused anchor.
                let cycle: Vec<AnchorId> = if self.clear_modal {
                    vec![parse_anchor(MODAL_CANCEL), parse_anchor(MODAL_CONFIRM)]
                } else {
                    vec![self.focus.clone()]
                };
                let satisfied = &cycle == anchors;
                let findings = self.finding_links(assertion);
                (
                    satisfied,
                    ObservedState::FocusCycle { anchors: cycle },
                    findings,
                )
            }
            StateAssertion::TextLanded { anchor, text } => {
                let actual = if anchor.as_str() == NEW_TASK_COMPOSER {
                    self.new_task_text.clone()
                } else if anchor.as_str() == TASK_COMPOSER {
                    self.task_composer_text.clone()
                } else {
                    String::new()
                };
                let satisfied = &actual == text;
                let findings = self.finding_links(assertion);
                (
                    satisfied,
                    ObservedState::Text {
                        anchor: anchor.clone(),
                        text: actual,
                    },
                    findings,
                )
            }
        }
    }

    /// The deterministic known-findings links of one observation (the
    /// F1 lessons the fake surface carries, referenced by id):
    ///
    /// - any assertion on the promo modal links
    ///   `first-run-keyboard-swallowing` (the defensive-dismissal
    ///   guard);
    /// - a `FocusOn { terminal-pty }` probe while the dock is open links
    ///   `pty-focus-transfer` (N5 — the honest unsatisfied verdict);
    /// - a chat-selection assertion right after a bracket chord links
    ///   `bracket-swap-chords` (N6 — the chord family under test);
    /// - an agents-view assertion right after the shifted-symbol
    ///   companion chord links `shifted-symbol-chord-companions` (the
    ///   N6 family);
    /// - a focus-confinement probe while the modal is open links
    ///   `modal-focus-traps` (the d19 trap).
    fn finding_links(&self, assertion: &StateAssertion) -> Vec<FindingId> {
        let mut links: Vec<FindingId> = Vec::new();
        let mut link = |id: &str| {
            if let Ok(finding) = FindingId::parse(id)
                && !links.contains(&finding)
            {
                links.push(finding);
            }
        };
        match assertion {
            StateAssertion::AnchorState { anchor, .. } => match anchor.as_str() {
                PROMO_ANCHOR => link("first-run-keyboard-swallowing"),
                AGENTS_VIEW_PANEL => {
                    if matches!(
                        &self.last_action,
                        Some(JourneyAction::KeyChord { chord }) if chord.as_str() == "alt+^"
                    ) {
                        link("shifted-symbol-chord-companions");
                    }
                }
                CHAT_SELECTION => {
                    if matches!(
                        &self.last_action,
                        Some(JourneyAction::KeyChord { chord })
                            if chord.as_str() == "ctrl+shift+]" || chord.as_str() == "ctrl+shift+["
                    ) {
                        // The selection assertion runs right after a
                        // bracket chord: the chord family under test.
                        link("bracket-swap-chords");
                    }
                }
                _ => {}
            },
            StateAssertion::FocusOn { anchor } => {
                if anchor.as_str() == TERMINAL_PTY && self.terminal {
                    link("pty-focus-transfer");
                }
            }
            StateAssertion::FocusConfined { .. } => {
                if self.clear_modal {
                    link("modal-focus-traps");
                }
            }
            StateAssertion::TextLanded { .. } => {}
        }
        links.sort();
        links
    }
}

fn parse_anchor(value: &str) -> AnchorId {
    AnchorId::parse(value)
        .unwrap_or_else(|error| panic!("the fake anchor {value:?} is canonically valid: {error}"))
}

fn parse_state(value: &str) -> StateKey {
    StateKey::parse(value)
        .unwrap_or_else(|error| panic!("the fake state {value:?} is canonically valid: {error}"))
}

fn observation(anchor: &str, state: &str) -> Result<AnchorObservation, LabError> {
    AnchorObservation::new(anchor, state)
}

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("the fake journeys are canonically valid: {error}"),
    }
}

fn step(id: &str, action: JourneyAction, probes: Vec<ProbeSpec>) -> JourneyStep {
    ok(JourneyStep::new(id, action, probes))
}

fn frame_probe(id: &str, moment: &str) -> ProbeSpec {
    ProbeSpec {
        id: ok(crate::refs::ProbeId::parse(id)),
        probe: ok(JourneyProbe::frame_capture(moment)),
    }
}

fn assert_probe(id: &str, assertion: StateAssertion) -> ProbeSpec {
    ProbeSpec {
        id: ok(crate::refs::ProbeId::parse(id)),
        probe: ok(JourneyProbe::state_assertion(assertion)),
    }
}

fn anchor_state(anchor_id: &str, state_id: &str) -> StateAssertion {
    StateAssertion::AnchorState {
        anchor: parse_anchor(anchor_id),
        state: parse_state(state_id),
    }
}

fn focus_on(anchor_id: &str) -> StateAssertion {
    StateAssertion::FocusOn {
        anchor: parse_anchor(anchor_id),
    }
}

fn text_landed(anchor_id: &str, text: &str) -> StateAssertion {
    StateAssertion::TextLanded {
        anchor: parse_anchor(anchor_id),
        text: text.to_owned(),
    }
}

fn focus_confined(anchor_list: &[&str]) -> StateAssertion {
    StateAssertion::FocusConfined {
        anchors: anchor_list
            .iter()
            .map(|value| parse_anchor(value))
            .collect(),
    }
}

/// The shell-discovery journey (the d24 model-picker scene shape): settle
/// the first-run promo → anchor a task (ctrl+n + type + ctrl+return, the
/// d23-proven chain) → open the model picker by chord (the honest empty
/// state) → close with scoped Escape (focus returns, never trapped) →
/// open the palette → type the query (the row appears) → Return lands
/// the panel.
///
/// # Panics
///
/// Panics if the canonical journey fails canonical validation (a bug in
/// the fakes, not in caller data).
#[must_use]
pub fn fake_shell_discovery_journey() -> JourneySpec {
    ok(JourneySpec::new(
        FAKE_SHELL_DISCOVERY_ID,
        "Shell discovery: anchor a task, open the model picker by chord and by palette",
        vec![
            ok(SurfaceId::parse("command-palette")),
            ok(SurfaceId::parse("model-picker")),
            ok(SurfaceId::parse("task-surface")),
        ],
        vec![ok(JourneyRef::parse("J-01")), ok(JourneyRef::parse("J-13"))],
        vec![
            step(
                "settle-first-run",
                ok(JourneyAction::key_chord("escape")),
                vec![assert_probe(
                    "assert-first-run-dismissed",
                    anchor_state(PROMO_ANCHOR, "closed"),
                )],
            ),
            step(
                "anchor-task",
                ok(JourneyAction::key_chord("ctrl+n")),
                vec![
                    frame_probe("frame-composer-open", "composer-open"),
                    assert_probe(
                        "assert-composer-open",
                        anchor_state(NEW_TASK_COMPOSER, "open"),
                    ),
                    assert_probe("assert-composer-focus", focus_on(NEW_TASK_COMPOSER)),
                ],
            ),
            step(
                "type-objective",
                ok(JourneyAction::type_text("the lab anchor task")),
                vec![
                    frame_probe("frame-typed-objective", "typed-objective"),
                    assert_probe(
                        "assert-objective-landed",
                        text_landed(NEW_TASK_COMPOSER, "the lab anchor task"),
                    ),
                ],
            ),
            step(
                "submit-task",
                ok(JourneyAction::key_chord("ctrl+return")),
                vec![
                    frame_probe("frame-task-surface", "task-surface"),
                    assert_probe("assert-task-active", anchor_state(TASK_SURFACE, "active")),
                    assert_probe("assert-task-composer-focus", focus_on(TASK_COMPOSER)),
                ],
            ),
            step(
                "open-picker",
                ok(JourneyAction::key_chord("ctrl+alt+shift+m")),
                vec![
                    frame_probe("frame-picker-open", "picker-open"),
                    assert_probe(
                        "assert-picker-open",
                        anchor_state(MODEL_PICKER_PANEL, "open-empty"),
                    ),
                    assert_probe("assert-picker-focus", focus_on(MODEL_PICKER_PANEL)),
                ],
            ),
            step(
                "close-picker",
                ok(JourneyAction::key_chord("escape")),
                vec![
                    frame_probe("frame-picker-closed", "picker-closed"),
                    assert_probe(
                        "assert-picker-closed",
                        anchor_state(MODEL_PICKER_PANEL, "closed"),
                    ),
                    assert_probe("assert-focus-returns", focus_on(TASK_COMPOSER)),
                ],
            ),
            step(
                "open-palette",
                ok(JourneyAction::key_chord("ctrl+k")),
                vec![
                    frame_probe("frame-palette-open", "palette-open"),
                    assert_probe("assert-palette-open", anchor_state(COMMAND_PALETTE, "open")),
                    assert_probe("assert-palette-input-focus", focus_on(PALETTE_INPUT)),
                ],
            ),
            step(
                "palette-query",
                ok(JourneyAction::type_text(PALETTE_QUERY)),
                vec![
                    frame_probe("frame-palette-row", "palette-row"),
                    assert_probe(
                        "assert-palette-row-visible",
                        anchor_state(PALETTE_ROW, "visible"),
                    ),
                ],
            ),
            step(
                "palette-land",
                ok(JourneyAction::key_chord("return")),
                vec![
                    frame_probe("frame-picker-landed", "picker-landed"),
                    assert_probe(
                        "assert-picker-landed",
                        anchor_state(MODEL_PICKER_PANEL, "open-empty"),
                    ),
                    assert_probe("assert-picker-landed-focus", focus_on(MODEL_PICKER_PANEL)),
                ],
            ),
        ],
    ))
}

/// The a11y chord-ladder journey (the d25/N5/N6/d19 shapes): settle the
/// promo → anchor a task → open the agents view by the digit chord →
/// close with scoped Escape → REOPEN by the shifted-symbol companion
/// (`alt+^`) → close → try the bracket-swap chord (N6: no move, the
/// known finding) → the Ctrl+PageDown alternate (moves) → open the
/// terminal dock (N5: the live PTY without focus transfer) → open the
/// clear-confirmation modal by a named-anchor click → tab through the
/// trapped actions (d19) → scoped Escape returns focus (017).
///
/// # Panics
///
/// Panics if the canonical journey fails canonical validation (a bug in
/// the fakes, not in caller data).
#[must_use]
pub fn fake_a11y_chord_ladder_journey() -> JourneySpec {
    ok(JourneySpec::new(
        FAKE_A11Y_CHORD_LADDER_ID,
        "A11y chord ladder: digit and companion chords, bracket swap, PTY focus, the modal trap",
        vec![
            ok(SurfaceId::parse("agents-view")),
            ok(SurfaceId::parse("chat-list")),
            ok(SurfaceId::parse("confirmation-modals")),
            ok(SurfaceId::parse("task-surface")),
            ok(SurfaceId::parse("terminal-dock")),
        ],
        vec![ok(JourneyRef::parse("J-01")), ok(JourneyRef::parse("J-07"))],
        vec![
            step(
                "settle-first-run",
                ok(JourneyAction::key_chord("escape")),
                vec![assert_probe(
                    "assert-first-run-dismissed",
                    anchor_state(PROMO_ANCHOR, "closed"),
                )],
            ),
            step(
                "anchor-task",
                ok(JourneyAction::key_chord("ctrl+n")),
                vec![
                    frame_probe("frame-composer-open", "composer-open"),
                    assert_probe(
                        "assert-composer-open",
                        anchor_state(NEW_TASK_COMPOSER, "open"),
                    ),
                ],
            ),
            step(
                "type-objective",
                ok(JourneyAction::type_text("the a11y chord ladder task")),
                vec![assert_probe(
                    "assert-objective-landed",
                    text_landed(NEW_TASK_COMPOSER, "the a11y chord ladder task"),
                )],
            ),
            step(
                "submit-task",
                ok(JourneyAction::key_chord("ctrl+return")),
                vec![
                    frame_probe("frame-task-surface", "task-surface"),
                    assert_probe("assert-task-active", anchor_state(TASK_SURFACE, "active")),
                    assert_probe("assert-task-composer-focus", focus_on(TASK_COMPOSER)),
                ],
            ),
            step(
                "agents-digit-chord",
                ok(JourneyAction::key_chord("ctrl+alt+shift+7")),
                vec![
                    frame_probe("frame-agents-open", "agents-open"),
                    assert_probe(
                        "assert-agents-open",
                        anchor_state(AGENTS_VIEW_PANEL, "open"),
                    ),
                ],
            ),
            step(
                "agents-close",
                ok(JourneyAction::key_chord("escape")),
                vec![
                    assert_probe(
                        "assert-agents-closed",
                        anchor_state(AGENTS_VIEW_PANEL, "closed"),
                    ),
                    assert_probe("assert-agents-close-focus", focus_on(TASK_COMPOSER)),
                ],
            ),
            step(
                "agents-companion-chord",
                ok(JourneyAction::key_chord("alt+^")),
                vec![
                    frame_probe("frame-agents-companion-open", "agents-companion-open"),
                    assert_probe(
                        "assert-agents-companion-open",
                        anchor_state(AGENTS_VIEW_PANEL, "open"),
                    ),
                ],
            ),
            step(
                "agents-close-again",
                ok(JourneyAction::key_chord("escape")),
                vec![assert_probe(
                    "assert-agents-closed-again",
                    anchor_state(AGENTS_VIEW_PANEL, "closed"),
                )],
            ),
            step(
                "bracket-swap-primary",
                ok(JourneyAction::key_chord("ctrl+shift+]")),
                vec![
                    frame_probe("frame-bracket-primary", "bracket-primary"),
                    assert_probe(
                        "assert-bracket-moves-selection",
                        anchor_state(CHAT_SELECTION, CHAT_ONE),
                    ),
                ],
            ),
            step(
                "bracket-swap-alternate",
                ok(JourneyAction::key_chord("ctrl+page_down")),
                vec![
                    frame_probe("frame-bracket-alternate", "bracket-alternate"),
                    assert_probe(
                        "assert-alternate-moves-selection",
                        anchor_state(CHAT_SELECTION, CHAT_ONE),
                    ),
                ],
            ),
            step(
                "open-terminal",
                ok(JourneyAction::key_chord("ctrl+`")),
                vec![
                    frame_probe("frame-terminal-open", "terminal-open"),
                    assert_probe(
                        "assert-terminal-open",
                        anchor_state(TERMINAL_DOCK, "open-live-pty"),
                    ),
                    assert_probe("assert-terminal-focus", focus_on(TERMINAL_PTY)),
                ],
            ),
            step(
                "trap-modal-open",
                ok(JourneyAction::click_anchor(CLEAR_HISTORY_AFFORDANCE)),
                vec![
                    frame_probe("frame-modal-open", "modal-open"),
                    assert_probe("assert-modal-open", anchor_state(CLEAR_MODAL, "open")),
                    assert_probe("assert-modal-focus", focus_on(MODAL_CANCEL)),
                ],
            ),
            step(
                "trap-cycle",
                ok(JourneyAction::key_chord("tab")),
                vec![assert_probe(
                    "assert-trap-confined",
                    focus_confined(&[MODAL_CANCEL, MODAL_CONFIRM]),
                )],
            ),
            step(
                "trap-close",
                ok(JourneyAction::key_chord("escape")),
                vec![
                    assert_probe("assert-modal-closed", anchor_state(CLEAR_MODAL, "closed")),
                    assert_probe("assert-trap-close-focus", focus_on(TASK_COMPOSER)),
                ],
            ),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_TEXT_BYTES;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn the_fake_surface_settles_the_promo_and_anchors_tasks() {
        let mut surface = FakeAppSurface::new();
        // Fresh profile: the promo is open and swallows keyboard input.
        assert!(matches!(
            surface.apply(&ok(JourneyAction::key_chord("ctrl+n"))),
            crate::evidence::ActionOutcome::NoVisibleChange
        ));
        // The defensive Escape settles it.
        assert!(matches!(
            surface.apply(&ok(JourneyAction::key_chord("escape"))),
            crate::evidence::ActionOutcome::Applied
        ));
        // The d23 anchor-task chain.
        surface.apply(&ok(JourneyAction::key_chord("ctrl+n")));
        surface.apply(&ok(JourneyAction::type_text("the lab anchor task")));
        assert!(matches!(
            surface.apply(&ok(JourneyAction::key_chord("ctrl+return"))),
            crate::evidence::ActionOutcome::Applied
        ));
        let (satisfied, observed, findings) = surface.observe(&focus_on(TASK_COMPOSER));
        assert!(satisfied);
        assert_eq!(
            observed,
            ObservedState::Focus {
                anchor: Some(parse_anchor(TASK_COMPOSER))
            }
        );
        assert!(findings.is_empty());
    }

    #[test]
    fn the_fake_surface_models_the_first_run_swallowing_honestly() {
        let mut surface = FakeAppSurface::new();
        let (satisfied, observed, findings) = surface.observe(&anchor_state(PROMO_ANCHOR, "open"));
        assert!(satisfied, "the promo is open at the fresh profile");
        assert_eq!(
            observed,
            ObservedState::Anchor {
                anchor: parse_anchor(PROMO_ANCHOR),
                state: parse_state("open"),
            }
        );
        assert_eq!(
            findings,
            vec![ok(FindingId::parse("first-run-keyboard-swallowing"))]
        );
        surface.apply(&ok(JourneyAction::key_chord("escape")));
        let (satisfied, observed, _) = surface.observe(&anchor_state(PROMO_ANCHOR, "closed"));
        assert!(satisfied);
        assert_eq!(
            observed,
            ObservedState::Anchor {
                anchor: parse_anchor(PROMO_ANCHOR),
                state: parse_state("closed"),
            }
        );
    }

    #[test]
    fn the_fake_surface_models_n5_n6_and_the_trap() {
        let mut surface = settled_surface_with_task();

        // N6: the bracket chord does not move the selection; the
        // assertion fails and links the finding by id.
        surface.apply(&ok(JourneyAction::key_chord("ctrl+shift+]")));
        let (satisfied, observed, findings) =
            surface.observe(&anchor_state(CHAT_SELECTION, CHAT_ONE));
        assert!(!satisfied, "the bracket chord does not move the selection");
        assert_eq!(
            observed,
            ObservedState::Anchor {
                anchor: parse_anchor(CHAT_SELECTION),
                state: parse_state(CHAT_TWO),
            }
        );
        assert_eq!(findings, vec![ok(FindingId::parse("bracket-swap-chords"))]);

        // The alternate moves it positively.
        surface.apply(&ok(JourneyAction::key_chord("ctrl+page_down")));
        let (satisfied, _, findings) = surface.observe(&anchor_state(CHAT_SELECTION, CHAT_ONE));
        assert!(satisfied, "ctrl+page_down moves the selection");
        assert!(findings.is_empty(), "the alternate is not the finding");

        // N5: the terminal opens a live PTY without transferring focus.
        surface.apply(&ok(JourneyAction::key_chord("ctrl+`")));
        let (terminal_open, _, _) = surface.observe(&anchor_state(TERMINAL_DOCK, "open-live-pty"));
        assert!(terminal_open);
        let (satisfied, observed, findings) = surface.observe(&focus_on(TERMINAL_PTY));
        assert!(!satisfied, "focus does not transfer to the PTY on open");
        assert_eq!(
            observed,
            ObservedState::Focus {
                anchor: Some(parse_anchor(TASK_COMPOSER))
            }
        );
        assert_eq!(findings, vec![ok(FindingId::parse("pty-focus-transfer"))]);

        // The d19 trap: the modal confines tab and scoped Escape
        // returns focus to the composer.
        surface.apply(&ok(JourneyAction::click_anchor(CLEAR_HISTORY_AFFORDANCE)));
        surface.apply(&ok(JourneyAction::key_chord("tab")));
        let (satisfied, observed, findings) =
            surface.observe(&focus_confined(&[MODAL_CANCEL, MODAL_CONFIRM]));
        assert!(satisfied, "the modal trap confines tab");
        assert_eq!(
            observed,
            ObservedState::FocusCycle {
                anchors: vec![parse_anchor(MODAL_CANCEL), parse_anchor(MODAL_CONFIRM)],
            }
        );
        assert_eq!(findings, vec![ok(FindingId::parse("modal-focus-traps"))]);
        surface.apply(&ok(JourneyAction::key_chord("escape")));
        let (satisfied, _, _) = surface.observe(&focus_on(TASK_COMPOSER));
        assert!(satisfied, "scoped Escape returns focus — nothing traps");
    }

    #[test]
    fn the_companion_chord_opens_the_same_panel_and_links_the_family() {
        let mut surface = settled_surface_with_task();
        surface.apply(&ok(JourneyAction::key_chord("alt+^")));
        let (satisfied, _, findings) = surface.observe(&anchor_state(AGENTS_VIEW_PANEL, "open"));
        assert!(satisfied, "the shifted-symbol companion opens the panel");
        assert_eq!(
            findings,
            vec![ok(FindingId::parse("shifted-symbol-chord-companions"))]
        );
    }

    #[test]
    fn unknown_click_anchors_are_rejected_with_named_reasons() {
        let mut surface = FakeAppSurface::new();
        surface.apply(&ok(JourneyAction::key_chord("escape")));
        match surface.apply(&ok(JourneyAction::click_anchor("no-such-affordance"))) {
            crate::evidence::ActionOutcome::Rejected { reason } => {
                assert!(reason.contains("no-such-affordance"));
            }
            other => panic!("the unknown anchor must be rejected, found {other:?}"),
        }
    }

    fn settled_surface_with_task() -> FakeAppSurface {
        let mut surface = FakeAppSurface::new();
        surface.apply(&ok(JourneyAction::key_chord("escape")));
        surface.apply(&ok(JourneyAction::key_chord("ctrl+n")));
        surface.apply(&ok(JourneyAction::type_text("the a11y chord ladder task")));
        surface.apply(&ok(JourneyAction::key_chord("ctrl+return")));
        surface
    }

    #[test]
    fn the_fake_journeys_are_canonically_valid() {
        for journey in [
            fake_shell_discovery_journey(),
            fake_a11y_chord_ladder_journey(),
        ] {
            ok(journey.validate());
            ok(crate::journey::required_capabilities(&journey));
            let serialized = ok(serde_json::to_string_pretty(&journey));
            let reloaded: JourneySpec = ok(serde_json::from_str(&serialized));
            assert_eq!(reloaded, journey);
        }
        let shell = fake_shell_discovery_journey();
        assert_eq!(shell.id.as_str(), FAKE_SHELL_DISCOVERY_ID);
        assert_eq!(shell.steps.len(), 9);
        let ladder = fake_a11y_chord_ladder_journey();
        assert_eq!(ladder.id.as_str(), FAKE_A11Y_CHORD_LADDER_ID);
        assert_eq!(ladder.steps.len(), 14);
    }

    #[test]
    fn typed_text_bounds_are_enforced_by_the_actions() {
        assert!(JourneyAction::type_text("").is_err());
        assert!(JourneyAction::type_text(&"t".repeat(MAX_TEXT_BYTES + 1)).is_err());
    }
}
