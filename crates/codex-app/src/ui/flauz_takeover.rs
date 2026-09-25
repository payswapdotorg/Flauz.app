//! The needs-you review surface (work order TAKE-001, F2 Wave 5):
//! the J-17 journey — when a run needs the human, the user sees the
//! named need with the consequence of each side, decides, can take
//! over the agent's turn and hand it back, and can cancel a step or
//! the run with every dependent told the truth. Nothing waits on the
//! human in silence.
//!
//! The surface:
//!
//! - the **needs-you panel** (Workspace-level): one row per task that
//!   needs the human, carrying the KIND of the need (an approval
//!   gate, an escalation, or a takeover opportunity) — the approval
//!   card ("Needs you: approve the environment switch — moving to
//!   the remote sandbox will re-run the setup steps" with Approve /
//!   Decline, each side's consequence stated), the takeover
//!   affordance ("Take over this step" — the agent's state preserved
//!   note + the handback path), and the cancellation affordances
//!   (this step / the whole run, each with what gets cancelled
//!   downstream, every dependent told the truth: "Cancelled — the
//!   research step it waited on was cancelled");
//! - the **task-surface affordance**: when THIS task needs a
//!   decision, a state-driven line rides the task surface ("Needs
//!   your decision — approve the environment switch") that opens the
//!   panel;
//! - the **attention extension** (the COL-001 precedent, additive
//!   only): the needs-you rows on the EXISTING Activity surface carry
//!   the kind ("Approval gate — approve the environment switch") —
//!   one extra render line inside the existing row, from the same
//!   attention flag; no second store, no parallel feed.
//!
//! Seven discoverability layers (PRODUCT-UX-JOURNEYS §1 / the shell
//! family discipline):
//!
//! 1. **Visible primary entry** — the needs-you button in the title
//!    bar (the WO-P2-018 bell / members / conflicts precedent).
//! 2. **Contextual affordance** — the task-surface decision line
//!    when this task needs a decision; the approval card inside the
//!    panel when a gate needs the human.
//! 3. **Palette fallback** — the "See what needs you" and "Take over
//!    a running step" rows (the palette is never the only discovery
//!    mechanism).
//! 4. **Stateful empty state** — nothing needs you: the honest quiet
//!    state ("Nothing needs you right now" + what lands here); no
//!    tasks wired: the honest not-wired state (nothing is invented).
//! 5. **Success/next-step** — after deciding: the attributed outcome
//!    with the consequence ("Decision made — declined. this step
//!    stops with your decision recorded as the reason"); after
//!    taking over: the handback path ("Hand back when you're done").
//! 6. **Keyboard path** — `Ctrl+Alt+Shift+Y` opens the panel (the
//!    letter family; verified free — the family in use is
//!    M/R/P/S/U/L and the digits 1-7), one scoped Escape closes, the
//!    flow is tab-navigable.
//! 7. **Honest unavailable state** — the live human-in-the-loop
//!    wiring is not yet in the run: the panel's note says so plainly,
//!    and the state shown is the module's view-model (the
//!    picker/providers/conflicts precedent).
//!
//! Boundary rules (work order TAKE-001 / Wave-5 kernel addendum §1,
//! §3, §4, §6, §7):
//!
//! - the app crate does NOT import the `flauz-takeover` contract
//!   crate in this wave (the app's Cargo.toml is outside this order's
//!   owned files): the panel renders a plain **view-model**
//!   ([`NeedsYouRow`]) that a later wave populates from the takeover
//!   fabric's records ([`set_needs_you_view`] /
//!   [`set_task_decision_affordance`] are the wiring seams);
//! - the world-stream event vocabulary is re-pinned here as strings
//!   (the members/conflicts precedent): the decision, takeover,
//!   handback and cancellation affordances record
//!   `task.approval_decided` / `task.takeover_started` /
//!   `task.takeover_handback` / `task.cancelled` /
//!   `task.dependent_cancelled` through the in-session
//!   **world-store seam** ([`TakeoverEventLog`]) a later slice wires
//!   to the real store;
//! - the takeover rides the FROZEN harness states (the escalated
//!   `Escalated` + continue/recover transitions — read-only
//!   reference): no state machine is changed here; the records are
//!   data the caller drives;
//! - the open/close paths follow the F1 focus contracts: capture the
//!   previously focused surface on open, auto-focus the panel handle
//!   once (the 019 request-once shape), restore on close (the 017
//!   contract) — keyboard focus is never trapped (the d19 lesson);
//! - the module is wired through minimal `ui.rs` named seams only
//!   (module declaration, palette rows, keyboard chord, scoped
//!   escape, state field, title-bar entry, task-surface mount,
//!   workspace-root panel mount, navigation close, the chord
//!   listener, the Activity attention kind line) — distinct from
//!   every prior wave's seams, each tagged TAKE-001.
//!
//! Every user-facing string lives in the copy registry below so the
//! language rules are unit-testable in one place.

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::{AnyElement, Context, FocusHandle, IntoElement, SharedString, Window, div, hsla, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::Escape,
    scroll::ScrollableElement,
    v_flex,
};

use super::WorkspaceView;

gpui::actions!(codexrs, [FlauzTakeoverShortcut]);

// ---------------------------------------------------------------------------
// Copy registry (user language, WITH consequences — addendum §6)
// ---------------------------------------------------------------------------

/// The title-bar entry tooltip (the chord rides the letter family).
pub(crate) const ENTRY_TOOLTIP: &str = "See what needs you · Ctrl+Alt+Shift+Y";
/// The needs-you panel heading.
const PANEL_HEADING: &str = "What needs you";
/// The one-line description under the heading.
const PANEL_DESCRIPTION: &str =
    "Approvals, escalations and takeovers across your tasks — nothing waits on you in silence";
/// The approval card's headline shape (the work order's exact
/// example: the named need, then the consequence of approving).
const APPROVAL_HEADLINE: &str = "Needs you: {need} — {approve_consequence}";
/// The approve affordance's label.
const APPROVE_LABEL: &str = "Approve";
/// The approve side's stated consequence.
const APPROVE_SIDE: &str = "If you approve: {approve_consequence}";
/// The decline affordance's label.
const DECLINE_LABEL: &str = "Decline";
/// The decline side's stated consequence (the denial never proceeds
/// silently — the consequence is stated before the click).
const DECLINE_SIDE: &str = "If you decline: {deny_consequence}";
/// The success line after approving (the attributed outcome + the
/// resume state).
const DECISION_APPROVED_STATUS: &str =
    "Decision made — approved. {approve_consequence}";
/// The success line after declining (the attributed outcome + the
/// honest stop).
const DECISION_DECLINED_STATUS: &str = "Decision made — declined. {deny_consequence}";
/// The takeover affordance's label (the work order's exact phrase).
const TAKEOVER_LABEL: &str = "Take over this step";
/// The agent's-state-preserved note (the projection law, in user
/// words — stated BEFORE the click).
const TAKEOVER_PRESERVED_NOTE: &str =
    "Your work joins the task — the agent's work so far is kept exactly as it was";
/// The handback path note (the handback is explicit — stated up
/// front).
const HANDBACK_PATH_NOTE: &str =
    "Hand back when you're done — the agent picks up from where you left it";
/// The success line after taking over.
const TAKEOVER_STARTED_STATUS: &str =
    "You have this step — the agent's work so far is kept exactly as it was";
/// The success line after handing back (the resume state).
const HANDBACK_STATUS: &str = "Handed back — the agent picks up from where you left it";
/// The handback affordance's label (shown while the human holds the
/// turn).
const HANDBACK_LABEL: &str = "Hand back to the agent";
/// The cancel-this-step affordance's label.
const CANCEL_STEP_LABEL: &str = "Cancel this step";
/// The cancel-the-run affordance's label.
const CANCEL_RUN_LABEL: &str = "Cancel the whole run";
/// The stated consequence shape of a cancellation (what gets
/// cancelled downstream — never implied).
const CANCEL_CONSEQUENCE: &str = "What gets cancelled with it: {downstream}";
/// The dependent's truth line (the work order's exact example shape,
/// stated for every dependent).
const DEPENDENT_LINE: &str = "Cancelled — the {step} step it waited on was cancelled";
/// The success line after cancelling (the attributed outcome).
const CANCELLED_STATUS: &str = "Cancelled — {downstream}";
/// The escalation-kind row's body (the call is made on the
/// who-is-using-what surface — the two surfaces stay disjoint).
const ESCALATION_ROW_BODY: &str =
    "Two tasks want the same thing — make the call from who-is-using-what (Ctrl+Alt+Shift+L)";
/// The task-surface affordance when this task needs a decision.
const TASK_DECISION_LINE: &str = "Needs your decision — {need}";
/// The task-surface affordance's tooltip.
const TASK_DECISION_TOOLTIP: &str = "Open what needs you (Ctrl+Alt+Shift+Y)";
/// The honest quiet state's title (the work order's exact phrase).
const EMPTY_TITLE: &str = "Nothing needs you right now";
/// The honest quiet state's body: what lands here.
const EMPTY_BODY: &str = "When a step needs your approval, a conflict needs your call, or you want \
     to take a step over yourself, it lands here — with what happens on each side.";
/// The honest not-wired note under the panel (the
/// picker/providers/conflicts precedent — say so plainly).
const NOT_WIRED_NOTE: &str = "This picture is the panel's own state right now — it connects to live \
     tasks once human-in-the-loop decisions are wired into the run.";
/// The honest guidance when the takeover row fires with nothing to
/// take over (the WO-P2-012 pattern: guidance, never a silent
/// no-op).
const NOTHING_TO_TAKE_OVER_GUIDANCE: &str =
    "No step can be taken over right now — the panel shows everything that needs you.";
/// The close affordance.
const BACK_TO_YOUR_WORK: &str = "Back to your work";
/// The close tooltip.
const BACK_TO_YOUR_WORK_TOOLTIP: &str = "Return to what you were doing (Escape)";
/// The keyboard-hint footer.
const ESCAPE_HINT: &str = "Escape closes this panel";
/// The command-palette row title (the work order's exact row).
pub(crate) const PALETTE_ROW_TITLE: &str = "See what needs you";
/// The command-palette row description.
pub(crate) const PALETTE_ROW_DESCRIPTION: &str =
    "Approvals, escalations and takeovers — what waits on you, with the consequences stated";
/// The take-over palette row title (the work order's exact row).
pub(crate) const PALETTE_ROW_TAKEOVER_TITLE: &str = "Take over a running step";
/// The take-over palette row description.
pub(crate) const PALETTE_ROW_TAKEOVER_DESCRIPTION: &str =
    "Step into the agent's work — your turn joins the task, the agent's work is kept";
/// The chord label shown in tooltips (the letter family — the
/// verified free letter).
pub(crate) const KEYBOARD_CHORD_LABEL: &str = "Ctrl+Alt+Shift+Y";

// ---------------------------------------------------------------------------
// The world-store seam (the frozen event vocabulary, re-pinned as
// strings — the members/conflicts precedent; the app crate does not
// import flauz-takeover)
// ---------------------------------------------------------------------------

/// The frozen event vocabulary the world-store seam records for the
/// needs-you decisions (the `flauz-takeover` crate's registered
/// types; the caller records them through the existing world-store
/// seam).
pub(crate) const TAKEOVER_STARTED_EVENT_TYPE: &str = "task.takeover_started";
/// The handback event type.
pub(crate) const TAKEOVER_HANDBACK_EVENT_TYPE: &str = "task.takeover_handback";
/// The approval-decided event type.
pub(crate) const APPROVAL_DECIDED_EVENT_TYPE: &str = "task.approval_decided";
/// The cancellation event type.
pub(crate) const CANCELLED_EVENT_TYPE: &str = "task.cancelled";
/// The dependent-cancellation event type.
pub(crate) const DEPENDENT_CANCELLED_EVENT_TYPE: &str = "task.dependent_cancelled";

/// One world-store seam event: the frozen event vocabulary a later
/// slice wires to the real world store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TakeoverEvent {
    /// The strictly increasing sequence number.
    pub seq: u64,
    /// The event type (the frozen `task.*` takeover vocabulary).
    pub event_type: &'static str,
    /// The task the event is about (its title, in user words).
    pub task: String,
    /// The consequence line the event recorded, in user words.
    pub detail: String,
}

/// The world-store seam: an append-only log of the events the real
/// world store will record. Events are immutable and never mutated or
/// deleted (kernel §3).
#[derive(Debug, Default)]
pub(crate) struct TakeoverEventLog {
    events: Vec<TakeoverEvent>,
}

impl TakeoverEventLog {
    /// An empty event log.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn record(&mut self, event_type: &'static str, task: &str, detail: &str) {
        let seq = self.events.len() as u64 + 1;
        self.events.push(TakeoverEvent {
            seq,
            event_type,
            task: task.to_owned(),
            detail: detail.to_owned(),
        });
    }

    /// The recorded events, in append order.
    pub(crate) fn events(&self) -> &[TakeoverEvent] {
        &self.events
    }
}

// ---------------------------------------------------------------------------
// The view-models (plain data; the wiring seams populate them)
// ---------------------------------------------------------------------------

/// The kind of a need (the attention attribution the rows carry —
/// the work order's three kinds, in user words).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NeedsYouKind {
    /// An approval gate: a step blocked on the human's decision.
    ApprovalGate,
    /// An escalation: a conflict the human must call (decided on the
    /// who-is-using-what surface).
    Escalation,
    /// A takeover opportunity: a step the human can take over.
    TakeoverOpportunity,
}

impl NeedsYouKind {
    /// Every kind, in the work order's order.
    #[allow(dead_code)] // pinned by the module's tests; the Wave-later attention wiring enumerates it
    pub(crate) const ALL: [Self; 3] = [
        Self::ApprovalGate,
        Self::Escalation,
        Self::TakeoverOpportunity,
    ];

    /// The kind's user word (what the attention rows and the panel
    /// rows carry).
    #[must_use]
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::ApprovalGate => "Approval gate",
            Self::Escalation => "Escalation",
            Self::TakeoverOpportunity => "Takeover opportunity",
        }
    }
}

/// The approval card's data: the named need with the consequence of
/// each side (a card without both consequences cannot be built —
/// the constructor is the only path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApprovalCardView {
    /// The named need ("approve the environment switch").
    pub need: String,
    /// The consequence of approving.
    pub approve_consequence: String,
    /// The consequence of declining.
    pub deny_consequence: String,
}

impl ApprovalCardView {
    /// Builds the card, refusing an empty need or consequence (the
    /// named-need law: the consequence of each side is structural).
    #[allow(dead_code)] // the module's tests and the Wave-later wiring construct cards; the render consumes them
    pub(crate) fn new(
        need: &str,
        approve_consequence: &str,
        deny_consequence: &str,
    ) -> Option<Self> {
        if need.is_empty() || approve_consequence.is_empty() || deny_consequence.is_empty() {
            return None;
        }
        Some(Self {
            need: need.to_owned(),
            approve_consequence: approve_consequence.to_owned(),
            deny_consequence: deny_consequence.to_owned(),
        })
    }

    /// The card's headline (the work order's exact example shape).
    #[must_use]
    pub(crate) fn headline(&self) -> String {
        APPROVAL_HEADLINE
            .replace("{need}", &self.need)
            .replace("{approve_consequence}", &self.approve_consequence)
    }

    /// The approve side's stated consequence.
    #[must_use]
    pub(crate) fn approve_side(&self) -> String {
        APPROVE_SIDE.replace("{approve_consequence}", &self.approve_consequence)
    }

    /// The decline side's stated consequence.
    #[must_use]
    pub(crate) fn decline_side(&self) -> String {
        DECLINE_SIDE.replace("{deny_consequence}", &self.deny_consequence)
    }
}

/// The takeover affordance's data: the step the human can take over,
/// and whether the human currently holds the turn (the handback
/// path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TakeoverCardView {
    /// The step's user-facing label ("the environment switch").
    pub step: String,
    /// Whether the human currently holds the turn (the handback
    /// affordance renders instead of the takeover one).
    pub active: bool,
}

/// One dependent's truth line, as user words (the downstream
/// consequence of a cancellation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DependentLineView {
    /// The step that was told the truth ("the research step").
    pub step: String,
}

impl DependentLineView {
    /// The dependent's line (the work order's exact example shape).
    #[must_use]
    pub(crate) fn line(&self) -> String {
        DEPENDENT_LINE.replace("{step}", &self.step)
    }
}

/// One task's needs-you row: the kind of the need, the approval card
/// when a gate needs the human, the takeover affordance, and the
/// cancellation affordances with the downstream truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NeedsYouRow {
    /// The task's id (the Activity-row join key).
    pub task_id: String,
    /// The task's title (shown on the row).
    pub task_title: String,
    /// The kind of the need (the attention attribution).
    pub kind: NeedsYouKind,
    /// The approval card, when a gate needs the human.
    pub approval: Option<ApprovalCardView>,
    /// The takeover affordance, when a step can be taken over (or
    /// the human holds the turn).
    pub takeover: Option<TakeoverCardView>,
    /// The cancellation truth for THIS task: every dependent named
    /// (empty when nothing waits on this task's steps).
    pub dependents: Vec<DependentLineView>,
}

impl NeedsYouRow {
    /// The downstream consequence line for the cancel-this-step
    /// affordance (what gets cancelled downstream — stated, never
    /// implied).
    #[must_use]
    pub(crate) fn cancel_step_consequence(&self) -> String {
        CANCEL_CONSEQUENCE.replace("{downstream}", &self.downstream_words())
    }

    /// The downstream consequence line for the cancel-the-run
    /// affordance.
    #[must_use]
    pub(crate) fn cancel_run_consequence(&self) -> String {
        CANCEL_CONSEQUENCE.replace(
            "{downstream}",
            "every unfinished step (finished work is kept)",
        )
    }

    /// The downstream words: every dependent named, honestly counted.
    #[must_use]
    fn downstream_words(&self) -> String {
        match self.dependents.len() {
            0 => "nothing waits on it".to_owned(),
            1 => "the 1 step that waits on it".to_owned(),
            count => format!("the {count} steps that wait on it"),
        }
    }
}

/// The task-surface affordance's data: one line for the selected task
/// when it needs a decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskDecisionAffordanceView {
    /// The affordance line ("Needs your decision — approve the
    /// environment switch").
    pub line: String,
}

/// Builds the task-surface decision line from the named need.
///
/// F-later wiring note: the human-in-the-loop wiring constructs the
/// per-task affordances from the takeover fabric's gate records
/// (pinned by the module tests today).
#[allow(dead_code)]
pub(crate) fn task_decision_line(need: &str) -> String {
    TASK_DECISION_LINE.replace("{need}", need)
}

// ---------------------------------------------------------------------------
// The attention extension (additive, the COL-001 precedent)
// ---------------------------------------------------------------------------

/// One needs-you kind row for the EXISTING attention model (the
/// additive extension, the COL-001 precedent): which kind of need
/// waits on the local user, on which chat. The rows are additive data
/// held by this module; the existing Activity rows render them as one
/// additional line — there is no second store and no parallel feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AttentionKindNeed {
    /// The chat the need is about.
    pub task_id: String,
    /// The kind of the need (approval gate / escalation / takeover
    /// opportunity).
    pub kind: NeedsYouKind,
    /// The need, phrased for the line ("approve the environment
    /// switch").
    pub need: String,
}

/// The kind-attribution line for one chat's Activity row (the
/// additive extension): "Approval gate — approve the environment
/// switch". Absent when no kind row is wired for the chat — the
/// existing row renders unchanged.
pub(crate) fn attention_kind_attribution_line(
    state: &TakeoverState,
    task_id: &str,
) -> Option<String> {
    state
        .attention_needs
        .iter()
        .find(|need| need.task_id == task_id)
        .map(|need| {
            ATTENTION_KIND_LINE
                .replace("{kind_word}", need.kind.word())
                .replace("{need}", &need.need)
        })
}

/// The attention kind line's shape ("Approval gate — approve the
/// environment switch").
const ATTENTION_KIND_LINE: &str = "{kind_word} — {need}";

// ---------------------------------------------------------------------------
// The additive surface state
// ---------------------------------------------------------------------------

/// The additive needs-you state: the panel bookkeeping, the
/// view-model seam, the per-task decision affordances, the attention
/// extension's kind rows, the world-store seam, and the panel's
/// focus bookkeeping.
pub(crate) struct TakeoverState {
    /// Whether the workspace-level needs-you panel is open.
    panel_open: bool,
    /// The needs-you rows, when wiring has populated them.
    view: Option<Vec<NeedsYouRow>>,
    /// The per-task decision affordances (a task without an entry
    /// renders no line).
    task_affordances: HashMap<String, TaskDecisionAffordanceView>,
    /// The needs-you kind rows for the EXISTING attention model (the
    /// attention extension's additive data).
    attention_needs: Vec<AttentionKindNeed>,
    /// The world-store seam (the frozen event vocabulary a later
    /// slice wires to the real store).
    event_log: TakeoverEventLog,
    /// The panel's keyboard focus handle (the 019 request-once
    /// shape).
    panel_focus: FocusHandle,
    /// Request-once guard for `panel_focus`.
    panel_focus_requested: bool,
    /// The surface that held focus when the panel opened, restored on
    /// close (the 017 close contract).
    focus_before_panel: Option<FocusHandle>,
    /// The last decision/takeover/cancellation status (the success
    /// state, shown until the panel closes).
    decision_status: Option<String>,
}

impl TakeoverState {
    /// Builds the closed, empty needs-you state (the honest
    /// not-wired state — nothing is invented).
    pub(crate) fn new(cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            panel_open: false,
            view: None,
            task_affordances: HashMap::new(),
            attention_needs: Vec::new(),
            event_log: TakeoverEventLog::new(),
            panel_focus: cx.focus_handle(),
            panel_focus_requested: false,
            focus_before_panel: None,
            decision_status: None,
        }
    }

    /// Whether the workspace-level needs-you panel is open (the
    /// workspace-root mount's visibility rule).
    pub(crate) fn panel_open(&self) -> bool {
        self.panel_open
    }

    /// The world-store seam's recorded events, in append order.
    #[allow(dead_code)] // read by the module's tests; the Wave-later world-store wiring surfaces them
    pub(crate) fn events(&self) -> &[TakeoverEvent] {
        self.event_log.events()
    }

    /// The selected task's decision affordance line, when one is
    /// wired.
    fn affordance_for(&self, task_id: &str) -> Option<TaskDecisionAffordanceView> {
        self.task_affordances.get(task_id).cloned()
    }

    /// Quietly closes the panel for an F1 navigation action (the
    /// deliberate close paths restore focus through the 017 contract
    /// instead). Returns whether anything changed.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.panel_open;
        self.panel_open = false;
        self.panel_focus_requested = false;
        self.focus_before_panel = None;
        self.decision_status = None;
        changed
    }
}

// ---------------------------------------------------------------------------
// The surface actions
// ---------------------------------------------------------------------------

/// Attaches (or clears) the needs-you rows — the wiring seam a later
/// wave drives from the takeover fabric's records. Clearing the view
/// returns the panel to the honest not-wired state.
#[allow(dead_code)] // the Wave-later human-in-the-loop wiring calls it; the module's tests pin the behavior
pub(crate) fn set_needs_you_view(
    workspace: &mut WorkspaceView,
    view: Option<Vec<NeedsYouRow>>,
    cx: &mut Context<WorkspaceView>,
) {
    workspace.flauz_takeover.view = view;
    cx.notify();
}

/// Attaches (or clears) one task's decision affordance — the wiring
/// seam.
#[allow(dead_code)] // the Wave-later human-in-the-loop wiring calls it; the module's tests pin the behavior
pub(crate) fn set_task_decision_affordance(
    workspace: &mut WorkspaceView,
    task_id: &str,
    affordance: Option<TaskDecisionAffordanceView>,
    cx: &mut Context<WorkspaceView>,
) {
    match affordance {
        Some(affordance) => {
            workspace
                .flauz_takeover
                .task_affordances
                .insert(task_id.to_owned(), affordance);
        }
        None => {
            workspace.flauz_takeover.task_affordances.remove(task_id);
        }
    }
    cx.notify();
}

/// Attaches (or clears) the attention extension's kind rows — the
/// additive wiring seam (additive data on the EXISTING attention
/// model; never a second store).
#[allow(dead_code)] // the Wave-later human-in-the-loop wiring calls it; the module's tests pin the behavior
pub(crate) fn set_attention_kind_needs(
    workspace: &mut WorkspaceView,
    needs: Vec<AttentionKindNeed>,
    cx: &mut Context<WorkspaceView>,
) {
    workspace.flauz_takeover.attention_needs = needs;
    cx.notify();
}

/// Opens the needs-you panel (the chord, the palette row and the
/// title-bar button all land here). Toggles closed when already open.
pub(crate) fn open_needs_you_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.flauz_takeover.panel_open {
        close_needs_you_panel(workspace, window, cx);
        return;
    }
    let state = &mut workspace.flauz_takeover;
    if state.focus_before_panel.is_none() {
        state.focus_before_panel = window.focused(cx);
    }
    state.panel_open = true;
    state.panel_focus_requested = true;
    cx.notify();
}

/// The take-over palette row's entry: opens the panel, and when no
/// step can be taken over, surfaces the honest guidance instead of a
/// silent no-op (the WO-P2-012 pattern).
pub(crate) fn open_needs_you_panel_for_takeover(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let can_take_over = workspace
        .flauz_takeover
        .view
        .as_ref()
        .is_some_and(|rows| rows.iter().any(|row| row.takeover.is_some()));
    if !can_take_over {
        workspace.dispatch_command_status(Some(NOTHING_TO_TAKE_OVER_GUIDANCE), cx);
    }
    open_needs_you_panel(workspace, window, cx);
}

/// Closes the needs-you panel and restores focus (the 017 close
/// contract).
pub(crate) fn close_needs_you_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if !workspace.flauz_takeover.panel_open {
        return;
    }
    workspace.flauz_takeover.panel_open = false;
    workspace.flauz_takeover.panel_focus_requested = false;
    workspace.flauz_takeover.decision_status = None;
    let previous = workspace.flauz_takeover.focus_before_panel.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// Applies the human's decision on one approval card (the Approve /
/// Decline affordances land here): the frozen
/// `task.approval_decided` vocabulary records through the
/// world-store seam, the row's gate resolves, and the status line
/// states the attributed outcome with its consequence. The decision
/// is EXPLICIT and attributed to the local user — a denial never
/// silently proceeds.
pub(crate) fn decide_approval(
    workspace: &mut WorkspaceView,
    task_id: &str,
    approve: bool,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let Some(rows) = workspace.flauz_takeover.view.clone() else {
        return;
    };
    let Some(row) = rows.iter().find(|row| row.task_id == task_id).cloned() else {
        return;
    };
    let Some(approval) = row.approval.clone() else {
        return;
    };
    let status = if approve {
        workspace.flauz_takeover.event_log.record(
            APPROVAL_DECIDED_EVENT_TYPE,
            &row.task_title,
            &approval.approve_side(),
        );
        DECISION_APPROVED_STATUS.replace("{approve_consequence}", &approval.approve_consequence)
    } else {
        workspace.flauz_takeover.event_log.record(
            APPROVAL_DECIDED_EVENT_TYPE,
            &row.task_title,
            &approval.decline_side(),
        );
        DECISION_DECLINED_STATUS.replace("{deny_consequence}", &approval.deny_consequence)
    };
    // The decided gate no longer needs the human: the row's approval
    // card resolves; the row itself stays only while another need
    // remains.
    let mut updated = rows;
    if let Some(entry) = updated.iter_mut().find(|row| row.task_id == task_id) {
        entry.approval = None;
    }
    updated.retain(|row| row.approval.is_some() || row.takeover.is_some());
    workspace.flauz_takeover.view = Some(updated);
    workspace.flauz_takeover.decision_status = Some(status);
    cx.notify();
}

/// The human takes over one step (the "Take over this step"
/// affordance): the frozen `task.takeover_started` vocabulary records
/// through the world-store seam, the row flips to the handback path,
/// and the status states the preserved-work note (the agent's state
/// preserved — the projection law, in user words).
pub(crate) fn begin_takeover(
    workspace: &mut WorkspaceView,
    task_id: &str,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let Some(rows) = workspace.flauz_takeover.view.clone() else {
        return;
    };
    let Some(row) = rows.iter().find(|row| row.task_id == task_id).cloned() else {
        return;
    };
    let mut updated = rows;
    let mut flipped = false;
    if let Some(entry) = updated.iter_mut().find(|row| row.task_id == task_id) {
        if let Some(takeover) = entry.takeover.as_mut() {
            if !takeover.active {
                takeover.active = true;
                flipped = true;
            }
        }
    }
    if !flipped {
        return;
    }
    workspace.flauz_takeover.event_log.record(
        TAKEOVER_STARTED_EVENT_TYPE,
        &row.task_title,
        TAKEOVER_PRESERVED_NOTE,
    );
    workspace.flauz_takeover.decision_status = Some(TAKEOVER_STARTED_STATUS.to_owned());
    workspace.flauz_takeover.view = Some(updated);
    cx.notify();
}

/// The human hands the turn back (the "Hand back to the agent"
/// affordance): the frozen `task.takeover_handback` vocabulary
/// records, and the status states the resume path. The handback is
/// EXPLICIT — the agent resumes from where the human left it.
pub(crate) fn hand_back(
    workspace: &mut WorkspaceView,
    task_id: &str,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let Some(rows) = workspace.flauz_takeover.view.clone() else {
        return;
    };
    let Some(row) = rows.iter().find(|row| row.task_id == task_id).cloned() else {
        return;
    };
    let mut updated = rows;
    let mut handed_back = false;
    if let Some(entry) = updated.iter_mut().find(|row| row.task_id == task_id) {
        if let Some(takeover) = entry.takeover.as_mut() {
            if takeover.active {
                takeover.active = false;
                handed_back = true;
            }
        }
    }
    if !handed_back {
        return;
    }
    workspace.flauz_takeover.event_log.record(
        TAKEOVER_HANDBACK_EVENT_TYPE,
        &row.task_title,
        HANDBACK_PATH_NOTE,
    );
    workspace.flauz_takeover.decision_status = Some(HANDBACK_STATUS.to_owned());
    workspace.flauz_takeover.view = Some(updated);
    cx.notify();
}

/// Cancels this task's step (the "Cancel this step" affordance): the
/// frozen `task.cancelled` + `task.dependent_cancelled` vocabulary
/// records — one dependent event per named dependent, every one told
/// the truth — and the status states the downstream consequence.
pub(crate) fn cancel_step(
    workspace: &mut WorkspaceView,
    task_id: &str,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    cancel(workspace, task_id, false, cx);
}

/// Cancels the whole run (the "Cancel the whole run" affordance):
/// the same frozen vocabulary, with every unfinished step named as a
/// dependent.
pub(crate) fn cancel_run_for_task(
    workspace: &mut WorkspaceView,
    task_id: &str,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    cancel(workspace, task_id, true, cx);
}

/// The shared cancellation path: records `task.cancelled`, then one
/// `task.dependent_cancelled` per named dependent (every dependent
/// told the truth), then the status line with the downstream
/// consequence.
fn cancel(
    workspace: &mut WorkspaceView,
    task_id: &str,
    whole_run: bool,
    cx: &mut Context<WorkspaceView>,
) {
    let Some(rows) = workspace.flauz_takeover.view.clone() else {
        return;
    };
    let Some(row) = rows.iter().find(|row| row.task_id == task_id).cloned() else {
        return;
    };
    let downstream = if whole_run {
        "every unfinished step (finished work is kept)".to_owned()
    } else {
        row.downstream_words()
    };
    workspace
        .flauz_takeover
        .event_log
        .record(CANCELLED_EVENT_TYPE, &row.task_title, &downstream);
    for dependent in &row.dependents {
        workspace.flauz_takeover.event_log.record(
            DEPENDENT_CANCELLED_EVENT_TYPE,
            &row.task_title,
            &dependent.line(),
        );
    }
    let status = CANCELLED_STATUS.replace("{downstream}", &downstream);
    // A cancelled task no longer needs the human.
    let mut updated = rows;
    updated.retain(|entry| entry.task_id != task_id);
    workspace.flauz_takeover.view = Some(updated);
    workspace.flauz_takeover.decision_status = Some(status);
    cx.notify();
}

// ---------------------------------------------------------------------------
// The renders
// ---------------------------------------------------------------------------

/// Renders the needs-you button in the title bar (layer 1's visible
/// primary entry — the bell/members/conflicts precedent).
pub(crate) fn render_needs_you_entry_button(
    workspace: &WorkspaceView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open = workspace.flauz_takeover.panel_open;
    Button::new("flauz-takeover-entry")
        .icon(IconName::Bell)
        .tooltip(ENTRY_TOOLTIP)
        .xsmall()
        .w(px(28.0))
        .h(px(28.0))
        .ghost()
        .selected(open)
        .on_click(cx.listener(|this, _, window, cx| {
            open_needs_you_panel(this, window, cx);
        }))
        .into_any_element()
}

/// Renders the task-surface decision affordance: one state-driven
/// line when the selected task needs a decision, with a control that
/// opens the panel. Absent when the task needs nothing — never a
/// permanent banner.
pub(crate) fn render_task_decision_affordance(
    workspace: &mut WorkspaceView,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let Some(task_id) = workspace.state.selected_task_id.clone() else {
        return v_flex().flex_none().into_any_element();
    };
    let Some(affordance) = workspace.flauz_takeover.affordance_for(&task_id) else {
        return v_flex().flex_none().into_any_element();
    };
    h_flex()
        .flex_none()
        .h(px(32.0))
        .px_5()
        .items_center()
        .gap_2()
        .child(
            Button::new("flauz-task-decision-affordance")
                .label(affordance.line)
                .icon(IconName::Bell)
                .tooltip(TASK_DECISION_TOOLTIP)
                .small()
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    open_needs_you_panel(this, window, cx);
                })),
        )
        .into_any_element()
}

/// Renders the workspace-level needs-you panel overlay: the rows
/// (the approval cards with both consequences, the takeover
/// affordances with the preserved note and the handback path, the
/// cancellation affordances with the downstream truth), the honest
/// quiet state, the success state, the not-wired note, and the close
/// affordance.
pub(crate) fn render_needs_you_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    if workspace.flauz_takeover.panel_focus_requested {
        // The 019 request-once shape: the panel claims the keyboard on
        // mount so the scoped Escape binding reaches it.
        workspace.flauz_takeover.panel_focus_requested = false;
        workspace.flauz_takeover.panel_focus.focus(window);
    }
    let view = workspace.flauz_takeover.view.clone();
    let decision_status = workspace.flauz_takeover.decision_status.clone();
    let panel_focus = workspace.flauz_takeover.panel_focus.clone();

    let mut panel = v_flex()
        .key_context("FlauzTakeover")
        .track_focus(&panel_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            close_needs_you_panel(this, window, cx);
        }))
        .w(px(super::modal_surface_width(
            workspace.shell_viewport_width,
            560.0,
        )))
        .max_h(px(super::modal_surface_max_height(
            workspace.shell_viewport_height,
            620.0,
        )))
        .rounded(px(16.0))
        .bg(cx.theme().popover)
        .shadow_xl()
        .overflow_hidden()
        .occlude()
        .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
        .child(
            h_flex()
                .h(px(52.0))
                .px_4()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(PANEL_HEADING),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(PANEL_DESCRIPTION),
                        ),
                )
                .child(
                    Button::new("flauz-takeover-close")
                        .label(BACK_TO_YOUR_WORK)
                        .icon(IconName::ArrowLeft)
                        .tooltip(BACK_TO_YOUR_WORK_TOOLTIP)
                        .small()
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            close_needs_you_panel(this, window, cx);
                        })),
                ),
        );
    match view {
        Some(rows) if !rows.is_empty() => {
            // The live picture: one row per task that needs the human.
            let mut list = v_flex()
                .px_4()
                .py_3()
                .gap_3()
                .max_h(px(430.0))
                .overflow_y_scrollbar();
            for (index, row) in rows.iter().enumerate() {
                list = list.child(render_needs_you_row(index, row, cx));
            }
            panel = panel.child(list);
        }
        _ => {
            // Layer 4: the honest quiet state — nothing needs you,
            // what lands here stated, nothing invented.
            panel = panel.child(
                v_flex()
                    .p_4()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Icon::new(IconName::CircleCheck).small())
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(EMPTY_TITLE),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(20.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(EMPTY_BODY),
                    ),
            );
        }
    }
    // The success state (layer 5): the attributed outcome WITH its
    // consequence.
    if let Some(status) = decision_status {
        panel = panel.child(
            h_flex()
                .gap_2()
                .px_4()
                .py_2()
                .items_center()
                .child(Icon::new(IconName::CircleCheck).small())
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .child(status.to_owned()),
                ),
        );
    }
    panel = panel
        .child(
            div()
                .px_4()
                .py_2()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(NOT_WIRED_NOTE),
        )
        .child(
            div()
                .px_4()
                .py_3()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!(
                    "{KEYBOARD_CHORD_LABEL} opens this panel · {ESCAPE_HINT}"
                )),
        );
    // The overlay backdrop: one click closes (the Activity overlay
    // precedent); the panel restores focus through the 017 contract.
    div()
        .absolute()
        .top_0()
        .right_0()
        .bottom_0()
        .left_0()
        .flex()
        .items_center()
        .justify_center()
        .occlude()
        .bg(hsla(0.0, 0.0, 0.0, 0.133))
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            close_needs_you_panel(this, window, cx);
        }))
        .on_any_mouse_down(
            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                cx.stop_propagation();
                if event.button == gpui::MouseButton::Left {
                    this.flauz_takeover.panel_open = false;
                    this.flauz_takeover.panel_focus_requested = false;
                    cx.notify();
                }
            }),
        )
        .child(panel)
        .into_any_element()
}

/// Renders one needs-you row: the task's kind-word header, the
/// approval card with both consequences and the Approve/Decline
/// affordances, the takeover affordance (the preserved note + the
/// handback path), and the cancellation affordances with the
/// downstream truth stated.
fn render_needs_you_row(
    index: usize,
    row: &NeedsYouRow,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let mut card = v_flex()
        .id(SharedString::from(format!("flauz-takeover-row-{index}")))
        .p_3()
        .gap_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background);
    // The row header: the task's title and the kind of the need (the
    // attention attribution, in user words).
    card = card.child(
        h_flex()
            .gap_2()
            .items_center()
            .child(Icon::new(IconName::Bell).xsmall())
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(row.task_title.clone()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(row.kind.word()),
            ),
    );
    // The escalation-kind row: the conflict's call is made on the
    // who-is-using-what surface (the two surfaces stay disjoint) —
    // the row says where, in user words.
    if row.kind == NeedsYouKind::Escalation && row.approval.is_none() {
        card = card.child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(ESCALATION_ROW_BODY),
        );
    }
    // The approval card (layer 2's contextual affordance): the named
    // need with the consequence of each side, decided explicitly.
    if let Some(approval) = &row.approval {
        let task_id_for_approve = row.task_id.clone();
        let task_id_for_decline = row.task_id.clone();
        card = card.child(
            v_flex()
                .gap_2()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().sidebar)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(Icon::new(IconName::TriangleAlert).small())
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(approval.headline()),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .child(
                            Button::new(SharedString::from(format!(
                                "flauz-takeover-approve-{index}"
                            )))
                            .label(APPROVE_LABEL)
                            .icon(IconName::ArrowRight)
                            .tooltip(approval.approve_side())
                            .small()
                            .primary()
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    decide_approval(
                                        this,
                                        &task_id_for_approve,
                                        true,
                                        window,
                                        cx,
                                    );
                                },
                            )),
                        )
                        .child(
                            Button::new(SharedString::from(format!(
                                "flauz-takeover-decline-{index}"
                            )))
                            .label(DECLINE_LABEL)
                            .icon(IconName::CircleX)
                            .tooltip(approval.decline_side())
                            .small()
                            .ghost()
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    decide_approval(
                                        this,
                                        &task_id_for_decline,
                                        false,
                                        window,
                                        cx,
                                    );
                                },
                            )),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .line_height(px(18.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(approval.approve_side()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .line_height(px(18.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(approval.decline_side()),
                        ),
                ),
        );
    }
    // The takeover affordance: "Take over this step" with the
    // preserved note and the handback path; the handback affordance
    // while the human holds the turn.
    if let Some(takeover) = &row.takeover {
        let task_id_for_takeover = row.task_id.clone();
        let task_id_for_handback = row.task_id.clone();
        let mut affordance = v_flex()
            .gap_2()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::User).xsmall())
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(TAKEOVER_LABEL),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .line_height(px(20.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(TAKEOVER_PRESERVED_NOTE),
            )
            .child(
                div()
                    .text_sm()
                    .line_height(px(20.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(HANDBACK_PATH_NOTE),
            );
        affordance = if takeover.active {
            affordance.child(
                Button::new(SharedString::from(format!(
                    "flauz-takeover-handback-{index}"
                )))
                .label(HANDBACK_LABEL)
                .icon(IconName::Undo2)
                .tooltip(HANDBACK_STATUS)
                .small()
                .ghost()
                .on_click(cx.listener(move |this, _, window, cx| {
                    hand_back(this, &task_id_for_handback, window, cx);
                })),
            )
        } else {
            affordance.child(
                Button::new(SharedString::from(format!(
                    "flauz-takeover-begin-{index}"
                )))
                .label(TAKEOVER_LABEL)
                .icon(IconName::ArrowRight)
                .tooltip(TAKEOVER_PRESERVED_NOTE)
                .small()
                .primary()
                .on_click(cx.listener(move |this, _, window, cx| {
                    begin_takeover(this, &task_id_for_takeover, window, cx);
                })),
            )
        };
        card = card.child(affordance);
    }
    // The cancellation affordances: this step and the whole run, each
    // with what gets cancelled downstream stated (every dependent
    // told the truth).
    let task_id_for_step = row.task_id.clone();
    let task_id_for_run = row.task_id.clone();
    let step_consequence = row.cancel_step_consequence();
    let run_consequence = row.cancel_run_consequence();
    card = card.child(
        v_flex()
            .gap_2()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .flex_wrap()
                    .child(
                        Button::new(SharedString::from(format!(
                            "flauz-takeover-cancel-step-{index}"
                        )))
                        .label(CANCEL_STEP_LABEL)
                        .tooltip(step_consequence.clone())
                        .small()
                        .ghost()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cancel_step(this, &task_id_for_step, window, cx);
                        })),
                    )
                    .child(
                        Button::new(SharedString::from(format!(
                            "flauz-takeover-cancel-run-{index}"
                        )))
                        .label(CANCEL_RUN_LABEL)
                        .tooltip(run_consequence.clone())
                        .small()
                        .ghost()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cancel_run_for_task(this, &task_id_for_run, window, cx);
                        })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .line_height(px(18.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(step_consequence),
            )
            .child(
                div()
                    .text_xs()
                    .line_height(px(18.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(run_consequence),
            ),
    );
    // The named dependents (the downstream truth, one line each).
    if !row.dependents.is_empty() {
        let mut lines = v_flex().gap_1().pl_2();
        for dependent in &row.dependents {
            lines = lines.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::CircleX).xsmall())
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(20.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(dependent.line()),
                    ),
            );
        }
        card = card.child(lines);
    }
    card.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_user_copy() -> Vec<String> {
        [
            ENTRY_TOOLTIP,
            PANEL_HEADING,
            PANEL_DESCRIPTION,
            APPROVAL_HEADLINE,
            APPROVE_LABEL,
            APPROVE_SIDE,
            DECLINE_LABEL,
            DECLINE_SIDE,
            DECISION_APPROVED_STATUS,
            DECISION_DECLINED_STATUS,
            TAKEOVER_LABEL,
            TAKEOVER_PRESERVED_NOTE,
            HANDBACK_PATH_NOTE,
            TAKEOVER_STARTED_STATUS,
            HANDBACK_STATUS,
            HANDBACK_LABEL,
            CANCEL_STEP_LABEL,
            CANCEL_RUN_LABEL,
            CANCEL_CONSEQUENCE,
            DEPENDENT_LINE,
            ESCALATION_ROW_BODY,
            TASK_DECISION_LINE,
            TASK_DECISION_TOOLTIP,
            EMPTY_TITLE,
            EMPTY_BODY,
            NOT_WIRED_NOTE,
            NOTHING_TO_TAKE_OVER_GUIDANCE,
            BACK_TO_YOUR_WORK,
            BACK_TO_YOUR_WORK_TOOLTIP,
            ESCAPE_HINT,
            PALETTE_ROW_TITLE,
            PALETTE_ROW_DESCRIPTION,
            PALETTE_ROW_TAKEOVER_TITLE,
            PALETTE_ROW_TAKEOVER_DESCRIPTION,
            KEYBOARD_CHORD_LABEL,
        ]
        .iter()
        .map(|text| (*text).to_owned())
        .collect()
    }

    /// The copy states consequences and never leaks an implementation
    /// term (addendum §6: user language only — "Needs you: approve
    /// the environment switch", never "approval gate record" or
    /// "cancellation propagation").
    #[test]
    fn takeover_copy_uses_user_language_with_consequences() {
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "takeover record",
                "takeover scope",
                "handback record",
                "cancellation record",
                "cancellation propagation",
                "approval decision",
                "approval gate record",
                "take-001",
                "flauz-takeover",
                "task_",
                "agent_",
                "art_",
                "dependent resolution",
                "terminal state",
                "propagation",
                "projection law",
                "same-stream",
                "harness",
                "escalated state",
                "actor",
                "actorref",
                "node name",
                "graph shape",
                "serde",
                "canonical",
                "world store",
                "event stream",
                "view-model",
                "viewmodel",
                "ulid",
                "preserved artifact",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "takeover copy must never say {forbidden:?}: {copy:?}"
                );
            }
        }
        // The work order's exact phrases are present, verbatim, WITH
        // their consequences.
        assert_eq!(PALETTE_ROW_TITLE, "See what needs you");
        assert_eq!(PALETTE_ROW_TAKEOVER_TITLE, "Take over a running step");
        assert_eq!(TAKEOVER_LABEL, "Take over this step");
        assert_eq!(EMPTY_TITLE, "Nothing needs you right now");
        assert_eq!(
            ApprovalCardView {
                need: "approve the environment switch".to_owned(),
                approve_consequence:
                    "moving to the remote sandbox will re-run the setup steps".to_owned(),
                deny_consequence:
                    "this step stops with your decision recorded as the reason".to_owned(),
            }
            .headline(),
            "Needs you: approve the environment switch — moving to the remote sandbox will \
             re-run the setup steps"
        );
        assert_eq!(
            DependentLineView {
                step: "research".to_owned(),
            }
            .line(),
            "Cancelled — the research step it waited on was cancelled"
        );
        assert_eq!(
            task_decision_line("approve the environment switch"),
            "Needs your decision — approve the environment switch"
        );
    }

    /// The honest states say so plainly: the not-wired note names
    /// what is and isn't connected; the quiet state explains what
    /// lands here; the guidance never silently no-ops.
    #[test]
    fn the_honest_states_say_so_plainly() {
        assert!(NOT_WIRED_NOTE.contains("right now"));
        assert!(NOT_WIRED_NOTE.contains("connects to live tasks"));
        assert!(NOT_WIRED_NOTE.contains("human-in-the-loop decisions are wired"));
        assert!(EMPTY_BODY.contains("it lands here"));
        assert!(EMPTY_BODY.contains("with what happens on each side"));
        assert!(NOTHING_TO_TAKE_OVER_GUIDANCE.contains("No step can be taken over"));
        // Every cancellation states what gets cancelled downstream.
        let row = NeedsYouRow {
            task_id: "task_1".to_owned(),
            task_title: "Market scan".to_owned(),
            kind: NeedsYouKind::ApprovalGate,
            approval: None,
            takeover: None,
            dependents: vec![
                DependentLineView {
                    step: "research".to_owned(),
                },
                DependentLineView {
                    step: "analysis".to_owned(),
                },
            ],
        };
        assert_eq!(
            row.cancel_step_consequence(),
            "What gets cancelled with it: the 2 steps that wait on it"
        );
        let lonely = NeedsYouRow {
            task_id: "task_1".to_owned(),
            task_title: "Market scan".to_owned(),
            kind: NeedsYouKind::ApprovalGate,
            approval: None,
            takeover: None,
            dependents: Vec::new(),
        };
        assert_eq!(
            lonely.cancel_step_consequence(),
            "What gets cancelled with it: nothing waits on it"
        );
        assert_eq!(
            lonely.cancel_run_consequence(),
            "What gets cancelled with it: every unfinished step (finished work is kept)"
        );
    }

    /// Layer 3: the palette fallback finds both rows through natural
    /// queries.
    #[test]
    fn palette_queries_resolve_to_the_rows() {
        for query in ["needs", "you", "what", "approval", "waits"] {
            let title = PALETTE_ROW_TITLE.to_lowercase();
            let description = PALETTE_ROW_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the needs-you row"
            );
        }
        for query in ["take", "over", "running", "step", "agent"] {
            let title = PALETTE_ROW_TAKEOVER_TITLE.to_lowercase();
            let description = PALETTE_ROW_TAKEOVER_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the take-over row"
            );
        }
    }

    /// The attention extension's kind attribution: the three kinds,
    /// their user words, and the line shape the Activity rows carry.
    #[test]
    fn attention_kind_lines_carry_the_kind() {
        assert_eq!(NeedsYouKind::ALL.len(), 3);
        assert_eq!(NeedsYouKind::ApprovalGate.word(), "Approval gate");
        assert_eq!(NeedsYouKind::Escalation.word(), "Escalation");
        assert_eq!(
            NeedsYouKind::TakeoverOpportunity.word(),
            "Takeover opportunity"
        );
        let line = ATTENTION_KIND_LINE
            .replace("{kind_word}", NeedsYouKind::ApprovalGate.word())
            .replace("{need}", "approve the environment switch");
        assert_eq!(line, "Approval gate — approve the environment switch");
    }

    /// The approval card cannot be built without both consequences
    /// (the named-need law, structural at the view-model too).
    #[test]
    fn an_approval_card_requires_both_consequences() {
        assert!(
            ApprovalCardView::new(
                "approve the environment switch",
                "moving to the remote sandbox will re-run the setup steps",
                "this step stops with your decision recorded as the reason",
            )
            .is_some()
        );
        assert!(ApprovalCardView::new("", "a", "b").is_none());
        assert!(ApprovalCardView::new("a need", "", "b").is_none());
        assert!(ApprovalCardView::new("a need", "a", "").is_none());
        let card = ApprovalCardView {
            need: "approve the environment switch".to_owned(),
            approve_consequence:
                "moving to the remote sandbox will re-run the setup steps".to_owned(),
            deny_consequence: "this step stops with your decision recorded as the reason"
                .to_owned(),
        };
        assert_eq!(
            card.approve_side(),
            "If you approve: moving to the remote sandbox will re-run the setup steps"
        );
        assert_eq!(
            card.decline_side(),
            "If you decline: this step stops with your decision recorded as the reason"
        );
        assert_eq!(
            DECISION_DECLINED_STATUS.replace("{deny_consequence}", &card.deny_consequence),
            "Decision made — declined. this step stops with your decision recorded as the reason"
        );
    }

    /// The world-store seam records the frozen vocabulary,
    /// append-only, with the consequence line as the detail.
    #[test]
    fn the_seam_records_the_frozen_vocabulary() {
        assert_eq!(TAKEOVER_STARTED_EVENT_TYPE, "task.takeover_started");
        assert_eq!(TAKEOVER_HANDBACK_EVENT_TYPE, "task.takeover_handback");
        assert_eq!(APPROVAL_DECIDED_EVENT_TYPE, "task.approval_decided");
        assert_eq!(CANCELLED_EVENT_TYPE, "task.cancelled");
        assert_eq!(DEPENDENT_CANCELLED_EVENT_TYPE, "task.dependent_cancelled");
        let mut log = TakeoverEventLog::new();
        assert!(log.events().is_empty());
        log.record(
            APPROVAL_DECIDED_EVENT_TYPE,
            "Market scan",
            "If you decline: this step stops with your decision recorded as the reason",
        );
        log.record(TAKEOVER_STARTED_EVENT_TYPE, "Market scan", TAKEOVER_PRESERVED_NOTE);
        log.record(
            DEPENDENT_CANCELLED_EVENT_TYPE,
            "Market scan",
            "Cancelled — the research step it waited on was cancelled",
        );
        let events = log.events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[1].seq, 2);
        assert_eq!(events[2].seq, 3);
        assert_eq!(events[0].event_type, "task.approval_decided");
        assert_eq!(events[1].event_type, "task.takeover_started");
        assert_eq!(events[2].event_type, "task.dependent_cancelled");
        assert_eq!(events[0].task, "Market scan");
    }

    /// The TAKE-001 registration seams in `ui.rs` (the house
    /// source-inspection style): the module declaration, the palette
    /// rows, the keyboard chord, the scoped escape, the state field,
    /// the title-bar entry, the task-surface mount, the
    /// workspace-root panel mount, the navigation close, the chord
    /// LISTENER, and the Activity attention kind line — each tagged
    /// TAKE-001, distinct from every prior wave's seams.
    #[test]
    fn takeover_seams_are_registered_in_the_ui_seams() {
        let source = include_str!("../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_takeover;"));
        assert!(source.contains("use flauz_takeover::FlauzTakeoverShortcut;"));

        // The palette registration seams (both rows).
        assert!(source.contains("PaletteCommand::SeeWhatNeedsYou"));
        assert!(source.contains("PaletteCommand::TakeOverARunningStep"));
        assert!(source.contains("flauz_takeover::PALETTE_ROW_TITLE"));
        assert!(source.contains("flauz_takeover::PALETTE_ROW_TAKEOVER_TITLE"));
        assert!(source.contains("flauz_takeover::open_needs_you_panel(workspace, window, cx)"));
        assert!(source.contains(
            "flauz_takeover::open_needs_you_panel_for_takeover(workspace, window, cx)"
        ));

        // The keyboard chord seam (Ctrl+Alt+Shift+Y — the letter
        // family, verified conflict-free: the letter family in use is
        // M (model picker), R (recovery), P (providers), S (save
        // flow), U (members) and L (conflicts); Y is the verified
        // free letter).
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzTakeoverShortcut, None"),
            "the needs-you chord must be bound"
        );
        assert!(source.contains("Some(\"Ctrl+Alt+Shift+Y\")"));
        assert_eq!(
            source.matches("Some(\"Ctrl+Alt+Shift+Y\")").count(),
            2,
            "both needs-you rows ride the chord (the conflicts precedent: the see row and the \
             take-over row open the same surface)"
        );

        // THE D25 LESSON (Gate B, the law): a KeyBinding without an
        // `.on_action` listener dispatches into the void — the chord
        // silently no-ops while the palette row works. The listener
        // must be registered on the workspace root next to the
        // picker/gap/agents/save/recovery/providers/members/conflicts
        // chord listeners.
        let listener_form: String = "cx.listener(|this, _: &FlauzTakeoverShortcut, window, cx| { \
             flauz_takeover::open_needs_you_panel(this, window, cx); })"
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            normalized.contains(&listener_form),
            "the needs-you chord must have an on_action listener that calls open_needs_you_panel \
             (the d25 Gate-B lesson — the seam test pins the LISTENER, not just the KeyBinding)"
        );

        // The binding registration itself (the alt-shift-y chord, the
        // letter family).
        assert!(source.contains("shortcut(\"alt-shift-y\")"));

        // The scoped escape seam (the d19 discipline: one Escape
        // through the panel's own focus context, never a trap).
        assert!(source.contains("Some(\"FlauzTakeover\")"));

        // The state-field seam.
        assert!(source.contains("flauz_takeover: flauz_takeover::TakeoverState"));
        assert!(source.contains("flauz_takeover::TakeoverState::new(cx)"));

        // The title-bar entry seam (layer 1's visible primary entry).
        let joined: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            joined.contains("flauz_takeover::render_needs_you_entry_button("),
            "the needs-you button must be mounted in the title bar"
        );

        // The task-surface mount seam (the decision affordance).
        assert!(
            joined.contains("flauz_takeover::render_task_decision_affordance("),
            "the decision affordance must be mounted on the task surface"
        );

        // The workspace-root panel mount seam (the needs-you overlay).
        assert!(
            joined.contains("flauz_takeover::render_needs_you_panel("),
            "the needs-you panel must be mounted at the workspace root"
        );

        // The navigation-close seam.
        assert!(source.contains("self.flauz_takeover.close_for_navigation()"));

        // The Activity attention kind line seam (the additive
        // extension, the COL-001 precedent).
        assert!(source.contains("flauz_takeover::attention_kind_attribution_line"));

        // Every TAKE-001 seam is tagged.
        let seam_tags = source.matches("TAKE-001").count();
        assert!(seam_tags >= 10, "each seam is tagged TAKE-001 (found {seam_tags})");
    }

    /// The attention extension is ADDITIVE-ONLY (the COL-001
    /// precedent — the no-second-store law, source-inspected): the
    /// existing Activity rows still derive from the ONE attention
    /// list, the kind line is one extra line drawn inside the
    /// existing row renderer, and this module holds no event store of
    /// its own beyond the frozen world-store seam vocabulary.
    #[test]
    fn the_attention_extension_is_additive_only() {
        let ui_source = include_str!("../ui.rs");
        // The existing rows still come from the same single attention
        // list (needs_attention_task_ids) — the one store.
        assert!(ui_source.contains("fn activity_view_rows(state: &AppState) -> Vec<TaskSummary>"));
        assert!(ui_source.contains("needs_attention_task_ids"));
        // The kind line renders INSIDE the existing Activity row
        // renderer — additive render line, not a parallel feed.
        assert!(ui_source.contains("render_activity_view_row"));
        assert!(ui_source.contains("flauz_takeover::attention_kind_attribution_line"));
        let this_source = include_str!("flauz_takeover.rs");
        // This module's kind-row storage is a plain replaceable vec
        // (set wholesale by the wiring seam) — never an append-only
        // event log.
        assert!(this_source.contains("attention_needs: Vec<AttentionKindNeed>"));
        assert!(this_source.contains("workspace.flauz_takeover.attention_needs = needs;"));
        // The only append-only log here is the world-store seam, and
        // it records ONLY the task.* takeover vocabulary (never
        // attention rows) — the forbidden name is built with concat!
        // so this assertion cannot match its own source.
        assert!(this_source.contains("TAKEOVER_STARTED_EVENT_TYPE"));
        assert!(!this_source.contains(concat!("ATTENTION_", "CHANGED_EVENT_TYPE")));
    }

    /// The d19 discipline, made checkable: the panel's focus path
    /// follows the request-once + restore contract.
    #[test]
    fn takeover_focus_never_traps() {
        let source = include_str!("flauz_takeover.rs");
        assert!(source.contains("panel_focus_requested = true"));
        assert!(source.contains("panel_focus_requested = false"));
        assert!(source.contains("focus_before_panel = window.focused(cx)"));
        assert!(source.contains("apply_overlay_close_focus_restore"));
        assert!(source.contains(".focus(window)"));
    }
}
