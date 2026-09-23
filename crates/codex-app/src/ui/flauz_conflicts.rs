//! The resource-conflicts surface (work order LEASE-001, F2 Wave 5):
//! the J-16 journey — who holds what, who is waiting, and the human's
//! call when a conflict escalates. No silent contention.
//!
//! The surface:
//!
//! - the **resource-conflicts panel** (Workspace-level): every
//!   resource's live state in user words ("Free", "Ana is using the
//!   browser — exclusive until 3:40 PM"), the queue view with honest
//!   positions ("2 tasks waiting — yours is next"), and the
//!   escalation card ("Needs you: two tasks want the browser
//!   exclusively — Ana's task holds it until 3:40 PM; Dev's task has
//!   waited 12 minutes") with the decision affordances, each stating
//!   its consequence;
//! - the **task-surface affordance**: when this task waits or holds,
//!   a state-driven line rides the task surface ("Waiting for the
//!   browser — 2nd in line") that opens the panel.
//!
//! Seven discoverability layers (PRODUCT-UX-JOURNEYS §1 / the shell
//! family discipline):
//!
//! 1. **Visible primary entry** — the conflicts button in the title
//!    bar (the WO-P2-018 bell / members precedent).
//! 2. **Contextual affordance** — the task-surface wait/hold line;
//!    the escalation card inside the panel when a conflict needs the
//!    human.
//! 3. **Palette fallback** — the "See who is using what" and
//!    "Resolve a resource conflict" rows (the palette is never the
//!    only discovery mechanism).
//! 4. **Stateful empty state** — no contention: the honest free state
//!    ("Free" + what the panel adds); no resources wired: the honest
//!    not-wired state (nothing is invented).
//! 5. **Success/next-step** — after a decision: the outcome restated
//!    with its consequence ("Dev's task takes over — Ana's task
//!    released the browser"); after opening: the live picture.
//! 6. **Keyboard path** — `Ctrl+Alt+Shift+L` opens the panel (the
//!    letter family; verified free — the family in use is M/6/R/P/S/U
//!    and the digits 1-7), one scoped Escape closes, the flow is
//!    tab-navigable.
//! 7. **Honest unavailable state** — the live lease state is not yet
//!    wired into the graph: the panel's note says so plainly, and the
//!    state shown is the module's view-model (the picker/providers
//!    precedent).
//!
//! Boundary rules (work order LEASE-001 / Wave-5 kernel addendum §1,
//! §2, §5, §6):
//!
//! - the app crate does NOT import the `flauz-lease` contract crate in
//!   this wave (the app's Cargo.toml is outside this order's owned
//!   files): the panel renders a plain **view-model**
//!   ([`ResourceConflictView`]) that a later wave populates from the
//!   manager's records ([`set_conflicts_view`] /
//!   [`set_task_affordance`] are the wiring seams);
//! - the world-stream event vocabulary is re-pinned here as strings
//!   (the members/save-workflow precedent): the decision affordances
//!   record `lease.granted` / `lease.released` through the in-session
//!   **world-store seam** ([`ConflictsEventLog`]) a later slice wires
//!   to the real store;
//! - the open/close paths follow the F1 focus contracts: capture the
//!   previously focused surface on open, auto-focus the panel handle
//!   once (the 019 request-once shape), restore on close (the 017
//!   contract) — keyboard focus is never trapped (the d19 lesson);
//! - the module is wired through minimal `ui.rs` named seams only
//!   (module declaration, palette rows, keyboard chord, scoped
//!   escape, state field, title-bar entry, task-surface mount,
//!   workspace-root panel mount, navigation close, the chord
//!   listener) — distinct from every prior wave's seams, each tagged
//!   LEASE-001.
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

gpui::actions!(codexrs, [FlauzConflictsShortcut]);

// ---------------------------------------------------------------------------
// Copy registry (user language, WITH consequences — addendum §6)
// ---------------------------------------------------------------------------

/// The title-bar entry tooltip (the chord rides the letter family).
pub(crate) const ENTRY_TOOLTIP: &str = "See who is using what · Ctrl+Alt+Shift+L";
/// The conflicts panel heading.
const PANEL_HEADING: &str = "Who is using what";
/// The one-line description under the heading.
const PANEL_DESCRIPTION: &str =
    "Every resource's state — who holds it, who is waiting, and what happens next";
/// The state word for an unheld resource (the work order's exact word).
const FREE_STATE: &str = "Free";
/// What the free state says the surface adds (the honest empty state).
const FREE_STATE_BODY: &str = "Nothing is waiting and no one holds it — any task can use it right \
     away. This panel shows the picture the moment two tasks want the same thing.";
/// The held-state line's shape (the work order's exact example, with
/// the holder named and the consequence stated).
const HELD_LINE: &str = "{holder} is using {resource} — {mode} until {until}";
/// The queue line when the local user's task is next (the work order's
/// exact phrase).
const QUEUE_LINE_YOURS_NEXT: &str = "{count} waiting — yours is next";
/// The queue line when the local user's task holds another position.
const QUEUE_LINE_YOURS_POSITION: &str = "{count} waiting — yours is {ordinal} in line";
/// The queue line when no waiter is the local user's.
const QUEUE_LINE_OTHERS: &str = "{count} waiting — {first} is next";
/// One waiter's queue row ("Dev — asked 12 minutes ago").
const WAITER_ROW: &str = "{name} — asked {waited} ago";
/// The escalation card's headline (the work order's exact example
/// shape).
const ESCALATION_HEADLINE: &str = "Needs you: two tasks want {resource} exclusively — {holder}'s task holds it until {until}; \
     {waiter}'s task has waited {waited}";
/// The grant-to-waiter affordance's label.
const GRANT_TO_WAITER_LABEL: &str = "Grant it to {waiter}'s task";
/// The grant-to-waiter consequence (stated, never implied).
const GRANT_TO_WAITER_CONSEQUENCE: &str =
    "{holder}'s task releases {resource} and {waiter}'s task takes over";
/// The keep-holder affordance's label.
const KEEP_HOLDER_LABEL: &str = "Keep it with {holder}'s task";
/// The keep-holder consequence (the named refusal).
const KEEP_HOLDER_CONSEQUENCE: &str =
    "{waiter}'s task is told no — it can ask again with a different plan";
/// The success line after a grant decision.
const DECISION_GRANTED_STATUS: &str =
    "Decision made — {waiter}'s task takes over {resource} ({holder}'s task released it)";
/// The success line after a keep decision.
const DECISION_KEPT_STATUS: &str =
    "Decision made — {holder}'s task keeps {resource} ({waiter}'s task was told no)";
/// The task-surface affordance when this task waits (the work order's
/// exact phrase).
const TASK_WAITING_LINE: &str = "Waiting for {resource} — {ordinal} in line";
/// The task-surface affordance when this task holds.
const TASK_HOLDING_LINE: &str = "Using {resource} until {until}";
/// The task-surface affordance's tooltip.
const TASK_AFFORDANCE_TOOLTIP: &str =
    "See the full picture — who holds what, who waits (Ctrl+Alt+Shift+L)";
/// The honest not-wired note under the panel (the picker/providers
/// precedent — say so plainly).
const NOT_WIRED_NOTE: &str = "This picture is the panel's own state right now — it connects to live tasks once resource \
     scheduling is wired into the run.";
/// The honest empty state's title (no resources wired).
const EMPTY_TITLE: &str = "Nothing is being shared yet";
/// The honest empty state's body: what the surface adds.
const EMPTY_BODY: &str = "When two tasks want the same thing — the browser, a file, a service — \
     this panel shows who holds it, who is waiting, and lets you make the call when they collide. \
     No contention stays hidden.";
/// The honest guidance when the resolve row fires with nothing to
/// decide (the WO-P2-012 pattern: guidance, never a silent no-op).
const NOTHING_TO_RESOLVE_GUIDANCE: &str =
    "No resource conflict needs you right now — the panel shows who is using what.";
/// The close affordance.
const BACK_TO_YOUR_WORK: &str = "Back to your work";
/// The close tooltip.
const BACK_TO_YOUR_WORK_TOOLTIP: &str = "Return to what you were doing (Escape)";
/// The keyboard-hint footer.
const ESCAPE_HINT: &str = "Escape closes this panel";
/// The command-palette row title (the see row — the work order's
/// exact row).
pub(crate) const PALETTE_ROW_TITLE: &str = "See who is using what";
/// The command-palette row description.
pub(crate) const PALETTE_ROW_DESCRIPTION: &str =
    "Every resource's state — who holds it, who is waiting, what happens next";
/// The resolve palette row title (the work order's exact row).
pub(crate) const PALETTE_ROW_RESOLVE_TITLE: &str = "Resolve a resource conflict";
/// The resolve palette row description.
pub(crate) const PALETTE_ROW_RESOLVE_DESCRIPTION: &str =
    "Make the call when two tasks want the same resource";
/// The chord label shown in tooltips (the letter family — the verified
/// free letter).
pub(crate) const KEYBOARD_CHORD_LABEL: &str = "Ctrl+Alt+Shift+L";

// ---------------------------------------------------------------------------
// The world-store seam (the frozen event vocabulary, re-pinned as
// strings — the members precedent; the app crate does not import
// flauz-lease)
// ---------------------------------------------------------------------------

/// The event types the world-store seam records for lease decisions
/// (the frozen `flauz-lease` event vocabulary; `lease.granted` and
/// `lease.released` are registered by `flauz-world`).
pub(crate) const LEASE_GRANTED_EVENT_TYPE: &str = "lease.granted";
/// The release event type.
pub(crate) const LEASE_RELEASED_EVENT_TYPE: &str = "lease.released";

/// One world-store seam event: the frozen event vocabulary a later
/// slice wires to the real world store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConflictsEvent {
    /// The strictly increasing sequence number.
    pub seq: u64,
    /// The event type (`lease.granted` / `lease.released` today).
    pub event_type: &'static str,
    /// The resource the event is about (its user-facing label).
    pub resource: String,
    /// The consequence line the event recorded, in user words.
    pub detail: String,
}

/// The world-store seam: an append-only log of the events the real
/// world store will record. Events are immutable and never mutated or
/// deleted (kernel §3).
#[derive(Debug, Default)]
pub(crate) struct ConflictsEventLog {
    events: Vec<ConflictsEvent>,
}

impl ConflictsEventLog {
    /// An empty event log.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn record(&mut self, event_type: &'static str, resource: &str, detail: &str) -> ConflictsEvent {
        let seq = self.events.len() as u64 + 1;
        let event = ConflictsEvent {
            seq,
            event_type,
            resource: resource.to_owned(),
            detail: detail.to_owned(),
        };
        self.events.push(event.clone());
        event
    }

    /// The recorded events, in append order.
    pub(crate) fn events(&self) -> &[ConflictsEvent] {
        &self.events
    }
}

// ---------------------------------------------------------------------------
// The view-models (plain data; the wiring seams populate them)
// ---------------------------------------------------------------------------

/// One resource's live state, in user words. The holder line states
/// the mode's consequence ("exclusive" / "shared") and the deadline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResourceStateView {
    /// No one holds it and nothing waits.
    #[allow(dead_code)]
    // read by the render today; constructed by the module's tests and (Wave-later) the run-shape wiring
    Free,
    /// A holder holds it: the named holder, the mode word, and the
    /// deadline in user words ("until 3:40 PM").
    Held {
        /// The holder's display name ("Ana").
        holder: String,
        /// The mode's consequence word ("exclusive" / "shared").
        mode_word: &'static str,
        /// The deadline in user words ("3:40 PM").
        until: String,
    },
}

impl ResourceStateView {
    /// The state line for the resource row: "Free" or the named
    /// holder line with its consequence.
    #[must_use]
    pub(crate) fn line(&self, resource: &str) -> String {
        match self {
            Self::Free => FREE_STATE.to_owned(),
            Self::Held {
                holder,
                mode_word,
                until,
            } => HELD_LINE
                .replace("{holder}", holder)
                .replace("{resource}", resource)
                .replace("{mode}", mode_word)
                .replace("{until}", until),
        }
    }
}

/// One waiter in the queue view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WaiterView {
    /// The waiting task's owner display name ("Dev").
    pub name: String,
    /// How long they have waited, in user words ("12 minutes").
    pub waited: String,
    /// Whether this row is the local user's task.
    pub is_you: bool,
}

impl WaiterView {
    /// The queue row line ("Dev — asked 12 minutes ago").
    #[must_use]
    pub(crate) fn line(&self) -> String {
        WAITER_ROW
            .replace("{name}", &self.name)
            .replace("{waited}", &self.waited)
    }
}

/// The escalation card's data: the named picture the human decides
/// over — both sides, with their windows in user words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EscalationView {
    /// The holder's display name ("Ana").
    pub holder: String,
    /// The holder's deadline in user words ("3:40 PM").
    pub until: String,
    /// The waiting task's owner display name ("Dev").
    pub waiter: String,
    /// How long the waiter has waited, in user words ("12 minutes").
    pub waited: String,
    /// The waiter's window end in user words ("4:30 PM") — what the
    /// waiter asked for, so the grant states the real deadline.
    pub waiter_until: String,
}

impl EscalationView {
    /// The escalation card's headline (the work order's exact example
    /// shape).
    #[must_use]
    pub(crate) fn headline(&self, resource: &str) -> String {
        ESCALATION_HEADLINE
            .replace("{resource}", resource)
            .replace("{holder}", &self.holder)
            .replace("{until}", &self.until)
            .replace("{waiter}", &self.waiter)
            .replace("{waited}", &self.waited)
    }
}

/// One resource's conflicts row: the resource label, its live state,
/// its queue, and its escalation card when a conflict needs the human.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceConflictView {
    /// The resource's user-facing label ("the browser").
    pub resource_label: String,
    /// The resource's live state.
    pub state: ResourceStateView,
    /// The queue, in request order.
    pub waiters: Vec<WaiterView>,
    /// The escalation card, when a conflict on this resource needs the
    /// human.
    pub escalation: Option<EscalationView>,
}

impl ResourceConflictView {
    /// The queue summary line: honest about the count and the local
    /// user's place ("2 tasks waiting — yours is next").
    #[must_use]
    pub(crate) fn queue_line(&self) -> Option<String> {
        if self.waiters.is_empty() {
            return None;
        }
        let count = self.waiters.len();
        let plural = if count == 1 { "task" } else { "tasks" };
        let count_text = format!("{count} {plural}");
        if let Some(position) = self.waiters.iter().position(|waiter| waiter.is_you) {
            if position == 0 {
                return Some(QUEUE_LINE_YOURS_NEXT.replace("{count}", &count_text));
            }
            return Some(
                QUEUE_LINE_YOURS_POSITION
                    .replace("{count}", &count_text)
                    .replace("{ordinal}", &ordinal(position as u32 + 1)),
            );
        }
        let first = &self.waiters[0].name;
        Some(
            QUEUE_LINE_OTHERS
                .replace("{count}", &count_text)
                .replace("{first}", first),
        )
    }

    /// Whether a conflict on this resource needs the human.
    #[must_use]
    pub(crate) fn needs_you(&self) -> bool {
        self.escalation.is_some()
    }
}

/// The task-surface affordance's data: one line for the selected task
/// when it waits or holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskAffordanceView {
    /// The affordance line ("Waiting for the browser — 2nd in line").
    pub line: String,
}

/// Builds the task-surface waiting line (the work order's exact
/// phrase) from the resource label and the 1-based position.
///
/// F-later wiring note: the lease wiring constructs the per-task
/// affordances from the manager's queue records (pinned by the module
/// tests today).
#[allow(dead_code)]
pub(crate) fn task_waiting_line(resource: &str, position: u32) -> String {
    TASK_WAITING_LINE
        .replace("{resource}", resource)
        .replace("{ordinal}", &ordinal(position))
}

/// Builds the task-surface holding line from the resource label and
/// the deadline in user words.
///
/// F-later wiring note: the lease wiring constructs the per-task
/// affordances from the manager's held records (pinned by the module
/// tests today).
#[allow(dead_code)]
pub(crate) fn task_holding_line(resource: &str, until: &str) -> String {
    TASK_HOLDING_LINE
        .replace("{resource}", resource)
        .replace("{until}", until)
}

/// The ordinal word for a queue position ("1st", "2nd", "3rd",
/// "4th" …) — honest positions, never flattering.
#[must_use]
pub(crate) fn ordinal(position: u32) -> String {
    let suffix = match position % 100 {
        11..=13 => "th",
        _ => match position % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{position}{suffix}")
}

// ---------------------------------------------------------------------------
// The additive surface state
// ---------------------------------------------------------------------------

/// The additive conflicts state: the panel bookkeeping, the
/// view-model seam, the per-task affordances, the world-store seam,
/// and the panel's focus bookkeeping.
pub(crate) struct ConflictsState {
    /// Whether the workspace-level conflicts panel is open.
    conflicts_open: bool,
    /// The resource view-models, when wiring has populated them.
    view: Option<Vec<ResourceConflictView>>,
    /// The per-task wait/hold affordances (a task without an entry
    /// renders no line).
    task_affordances: HashMap<String, TaskAffordanceView>,
    /// The world-store seam (the frozen event vocabulary a later slice
    /// wires to the real store).
    event_log: ConflictsEventLog,
    /// The panel's keyboard focus handle (the 019 request-once shape).
    panel_focus: FocusHandle,
    /// Request-once guard for `panel_focus`.
    panel_focus_requested: bool,
    /// The surface that held focus when the panel opened, restored on
    /// close (the 017 close contract).
    focus_before_panel: Option<FocusHandle>,
    /// The last decision status (the success state, shown until the
    /// panel closes).
    decision_status: Option<String>,
}

impl ConflictsState {
    /// Builds the closed, empty conflicts state (the honest
    /// not-wired state — nothing is invented).
    pub(crate) fn new(cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            conflicts_open: false,
            view: None,
            task_affordances: HashMap::new(),
            event_log: ConflictsEventLog::new(),
            panel_focus: cx.focus_handle(),
            panel_focus_requested: false,
            focus_before_panel: None,
            decision_status: None,
        }
    }

    /// Whether the workspace-level conflicts panel is open (the
    /// workspace-root mount's visibility rule).
    pub(crate) fn conflicts_open(&self) -> bool {
        self.conflicts_open
    }

    /// The world-store seam's recorded events, in append order.
    #[allow(dead_code)] // read by the module's tests; the Wave-later world-store wiring surfaces them
    pub(crate) fn events(&self) -> &[ConflictsEvent] {
        self.event_log.events()
    }

    /// The selected task's affordance line, when one is wired.
    fn affordance_for(&self, task_id: &str) -> Option<TaskAffordanceView> {
        self.task_affordances.get(task_id).cloned()
    }

    /// Quietly closes the panel for an F1 navigation action (the
    /// deliberate close paths restore focus through the 017 contract
    /// instead). Returns whether anything changed.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.conflicts_open;
        self.conflicts_open = false;
        self.panel_focus_requested = false;
        self.focus_before_panel = None;
        self.decision_status = None;
        changed
    }
}

// ---------------------------------------------------------------------------
// The surface actions
// ---------------------------------------------------------------------------

/// Attaches (or clears) the resource view-models — the wiring seam a
/// later wave drives from the lease manager's records. Clearing the
/// view returns the panel to the honest not-wired state.
#[allow(dead_code)] // the Wave-later lease wiring calls it; the module's tests pin the behavior
pub(crate) fn set_conflicts_view(
    workspace: &mut WorkspaceView,
    view: Option<Vec<ResourceConflictView>>,
    cx: &mut Context<WorkspaceView>,
) {
    workspace.flauz_conflicts.view = view;
    cx.notify();
}

/// Attaches (or clears) one task's wait/hold affordance — the wiring
/// seam.
#[allow(dead_code)] // the Wave-later lease wiring calls it; the module's tests pin the behavior
pub(crate) fn set_task_affordance(
    workspace: &mut WorkspaceView,
    task_id: &str,
    affordance: Option<TaskAffordanceView>,
    cx: &mut Context<WorkspaceView>,
) {
    match affordance {
        Some(affordance) => {
            workspace
                .flauz_conflicts
                .task_affordances
                .insert(task_id.to_owned(), affordance);
        }
        None => {
            workspace.flauz_conflicts.task_affordances.remove(task_id);
        }
    }
    cx.notify();
}

/// Opens the conflicts panel (the chord, the palette row and the
/// title-bar button all land here). Toggles closed when already open.
pub(crate) fn open_conflicts_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.flauz_conflicts.conflicts_open {
        close_conflicts_panel(workspace, window, cx);
        return;
    }
    let state = &mut workspace.flauz_conflicts;
    if state.focus_before_panel.is_none() {
        state.focus_before_panel = window.focused(cx);
    }
    state.conflicts_open = true;
    state.panel_focus_requested = true;
    cx.notify();
}

/// The resolve palette row's entry: opens the panel, and when nothing
/// needs the human's call, surfaces the honest guidance instead of a
/// silent no-op (the WO-P2-012 pattern).
pub(crate) fn open_conflicts_panel_for_resolution(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let needs_you = workspace
        .flauz_conflicts
        .view
        .as_ref()
        .is_some_and(|rows| rows.iter().any(|row| row.needs_you()));
    if !needs_you {
        workspace.dispatch_command_status(Some(NOTHING_TO_RESOLVE_GUIDANCE), cx);
    }
    open_conflicts_panel(workspace, window, cx);
}

/// Closes the conflicts panel and restores focus (the 017 close
/// contract).
pub(crate) fn close_conflicts_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if !workspace.flauz_conflicts.conflicts_open {
        return;
    }
    workspace.flauz_conflicts.conflicts_open = false;
    workspace.flauz_conflicts.panel_focus_requested = false;
    workspace.flauz_conflicts.decision_status = None;
    let previous = workspace.flauz_conflicts.focus_before_panel.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// Applies the human's decision on one escalated conflict (the
/// escalation card's affordances land here): the view-model reflects
/// the outcome, the frozen `lease.granted` / `lease.released`
/// vocabulary records through the world-store seam, and the status
/// line states the consequence. The decision is EXPLICIT and
/// attributed to the local user — never auto-resolved.
pub(crate) fn resolve_conflict(
    workspace: &mut WorkspaceView,
    resource_label: &str,
    grant_to_waiter: bool,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let Some(rows) = workspace.flauz_conflicts.view.clone() else {
        return;
    };
    let Some(row) = rows
        .iter()
        .find(|row| row.resource_label == resource_label)
        .cloned()
    else {
        return;
    };
    let Some(escalation) = row.escalation.clone() else {
        return;
    };
    let mut updated = rows;
    let index = updated
        .iter()
        .position(|candidate| candidate.resource_label == resource_label);
    let status = if grant_to_waiter {
        // Grant to the waiter: the holder releases, the waiter holds
        // until the end of its own ask — the row's state flips, the
        // queue shortens, and the frozen vocabulary records both
        // sides.
        let granted_detail = GRANT_TO_WAITER_CONSEQUENCE
            .replace("{holder}", &escalation.holder)
            .replace("{resource}", &row.resource_label)
            .replace("{waiter}", &escalation.waiter);
        if let Some(index) = index {
            let row = &mut updated[index];
            row.escalation = None;
            row.state = ResourceStateView::Held {
                holder: escalation.waiter.clone(),
                mode_word: "exclusive",
                until: escalation.waiter_until.clone(),
            };
            if let Some(position) = row
                .waiters
                .iter()
                .position(|waiter| waiter.name == escalation.waiter)
            {
                row.waiters.remove(position);
            }
        }
        workspace.flauz_conflicts.event_log.record(
            LEASE_GRANTED_EVENT_TYPE,
            &row.resource_label,
            &granted_detail,
        );
        workspace.flauz_conflicts.event_log.record(
            LEASE_RELEASED_EVENT_TYPE,
            &row.resource_label,
            &granted_detail,
        );
        DECISION_GRANTED_STATUS
            .replace("{waiter}", &escalation.waiter)
            .replace("{resource}", &row.resource_label)
            .replace("{holder}", &escalation.holder)
    } else {
        // Keep the holder: the waiter is told no, the named
        // consequence states it, and the holder is untouched.
        let kept_detail = KEEP_HOLDER_CONSEQUENCE
            .replace("{holder}", &escalation.holder)
            .replace("{resource}", &row.resource_label)
            .replace("{waiter}", &escalation.waiter);
        if let Some(index) = index {
            updated[index].escalation = None;
        }
        workspace.flauz_conflicts.event_log.record(
            LEASE_RELEASED_EVENT_TYPE,
            &row.resource_label,
            &kept_detail,
        );
        DECISION_KEPT_STATUS
            .replace("{holder}", &escalation.holder)
            .replace("{resource}", &row.resource_label)
            .replace("{waiter}", &escalation.waiter)
    };
    workspace.flauz_conflicts.view = Some(updated);
    workspace.flauz_conflicts.decision_status = Some(status);
    cx.notify();
}

// ---------------------------------------------------------------------------
// The renders
// ---------------------------------------------------------------------------

/// Renders the conflicts button in the title bar (layer 1's visible
/// primary entry — the members/bell precedent).
pub(crate) fn render_conflicts_entry_button(
    workspace: &WorkspaceView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open = workspace.flauz_conflicts.conflicts_open;
    Button::new("flauz-conflicts-entry")
        .icon(IconName::Globe)
        .tooltip(ENTRY_TOOLTIP)
        .xsmall()
        .w(px(28.0))
        .h(px(28.0))
        .ghost()
        .selected(open)
        .on_click(cx.listener(|this, _, window, cx| {
            open_conflicts_panel(this, window, cx);
        }))
        .into_any_element()
}

/// Renders the task-surface affordance: one state-driven line when
/// the selected task waits or holds, with a control that opens the
/// panel. Absent when the task neither waits nor holds — never a
/// permanent banner.
pub(crate) fn render_task_resource_affordance(
    workspace: &mut WorkspaceView,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let Some(task_id) = workspace.state.selected_task_id.clone() else {
        return v_flex().flex_none().into_any_element();
    };
    let Some(affordance) = workspace.flauz_conflicts.affordance_for(&task_id) else {
        return v_flex().flex_none().into_any_element();
    };
    h_flex()
        .flex_none()
        .h(px(32.0))
        .px_5()
        .items_center()
        .gap_2()
        .child(
            Button::new("flauz-task-resource-affordance")
                .label(affordance.line)
                .icon(IconName::Globe)
                .tooltip(TASK_AFFORDANCE_TOOLTIP)
                .small()
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    open_conflicts_panel(this, window, cx);
                })),
        )
        .into_any_element()
}

/// Renders the workspace-level conflicts panel overlay: the resource
/// rows with live state, the queue views, the escalation cards with
/// the decision affordances, the honest not-wired note, and the close
/// affordance.
pub(crate) fn render_conflicts_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    if workspace.flauz_conflicts.panel_focus_requested {
        // The 019 request-once shape: the panel claims the keyboard on
        // mount so the scoped Escape binding reaches it.
        workspace.flauz_conflicts.panel_focus_requested = false;
        workspace.flauz_conflicts.panel_focus.focus(window);
    }
    let view = workspace.flauz_conflicts.view.clone();
    let decision_status = workspace.flauz_conflicts.decision_status.clone();
    let panel_focus = workspace.flauz_conflicts.panel_focus.clone();

    let mut panel = v_flex()
        .key_context("FlauzConflicts")
        .track_focus(&panel_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            close_conflicts_panel(this, window, cx);
        }))
        .w(px(super::modal_surface_width(
            workspace.shell_viewport_width,
            520.0,
        )))
        .max_h(px(super::modal_surface_max_height(
            workspace.shell_viewport_height,
            600.0,
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
                    Button::new("flauz-conflicts-close")
                        .label(BACK_TO_YOUR_WORK)
                        .icon(IconName::ArrowLeft)
                        .tooltip(BACK_TO_YOUR_WORK_TOOLTIP)
                        .small()
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            close_conflicts_panel(this, window, cx);
                        })),
                ),
        );
    match view {
        Some(rows) if !rows.is_empty() => {
            // The live picture: one row per resource (state + queue +
            // escalation card).
            let mut list = v_flex()
                .px_4()
                .py_3()
                .gap_3()
                .max_h(px(420.0))
                .overflow_y_scrollbar();
            for (index, row) in rows.iter().enumerate() {
                list = list.child(render_resource_row(index, row, cx));
            }
            panel = panel.child(list);
        }
        _ => {
            // Layer 4: the honest empty state — no resources wired,
            // nothing invented, what the surface adds stated.
            panel = panel.child(
                v_flex()
                    .p_4()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(EMPTY_TITLE),
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
    // The success state (layer 5): the decision restated WITH its
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
            close_conflicts_panel(this, window, cx);
        }))
        .on_any_mouse_down(
            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                cx.stop_propagation();
                if event.button == gpui::MouseButton::Left {
                    this.flauz_conflicts.conflicts_open = false;
                    this.flauz_conflicts.panel_focus_requested = false;
                    cx.notify();
                }
            }),
        )
        .child(panel)
        .into_any_element()
}

/// Renders one resource row: the state line (holder + mode word +
/// deadline, or "Free"), the queue summary, the queue rows, and the
/// escalation card with the decision affordances.
fn render_resource_row(
    index: usize,
    row: &ResourceConflictView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let mut card = v_flex()
        .id(SharedString::from(format!("flauz-conflict-row-{index}")))
        .p_3()
        .gap_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background);
    // The state line: "Free" (with what the surface adds) or the
    // named holder line with its consequence.
    match &row.state {
        ResourceStateView::Free => {
            card = card
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(Icon::new(IconName::CircleCheck).small())
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(format!("{} — {}", row.resource_label, FREE_STATE)),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(FREE_STATE_BODY),
                );
        }
        ResourceStateView::Held { .. } => {
            card = card.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::User).xsmall())
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(20.0))
                            .child(row.state.line(&row.resource_label)),
                    ),
            );
        }
    }
    // The queue view: the honest summary line + one row per waiter.
    if let Some(queue_line) = row.queue_line() {
        card = card.child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(queue_line),
        );
        let mut queue = v_flex().gap_1().pl_2();
        for waiter in &row.waiters {
            queue = queue.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::LoaderCircle).xsmall())
                    .child(div().text_sm().line_height(px(20.0)).child(format!(
                        "{}{}",
                        waiter.line(),
                        if waiter.is_you { " (you)" } else { "" }
                    ))),
            );
        }
        card = card.child(queue);
    }
    // The escalation card (layer 2's contextual affordance): the
    // named picture and the decision, each consequence stated.
    if let Some(escalation) = &row.escalation {
        let resource_label = row.resource_label.clone();
        let grant_consequence = GRANT_TO_WAITER_CONSEQUENCE
            .replace("{holder}", &escalation.holder)
            .replace("{resource}", &row.resource_label)
            .replace("{waiter}", &escalation.waiter);
        let keep_consequence = KEEP_HOLDER_CONSEQUENCE
            .replace("{holder}", &escalation.holder)
            .replace("{resource}", &row.resource_label)
            .replace("{waiter}", &escalation.waiter);
        let grant_label = GRANT_TO_WAITER_LABEL.replace("{waiter}", &escalation.waiter);
        let keep_label = KEEP_HOLDER_LABEL.replace("{holder}", &escalation.holder);
        let resource_for_grant = resource_label.clone();
        let resource_for_keep = resource_label.clone();
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
                                .child(escalation.headline(&row.resource_label)),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .child(
                            Button::new(SharedString::from(format!(
                                "flauz-conflict-grant-{index}"
                            )))
                            .label(grant_label)
                            .icon(IconName::ArrowRight)
                            .tooltip(grant_consequence.clone())
                            .small()
                            .primary()
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    resolve_conflict(this, &resource_for_grant, true, window, cx);
                                },
                            )),
                        )
                        .child(
                            Button::new(SharedString::from(format!("flauz-conflict-keep-{index}")))
                                .label(keep_label)
                                .icon(IconName::CircleCheck)
                                .tooltip(keep_consequence.clone())
                                .small()
                                .ghost()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    resolve_conflict(this, &resource_for_keep, false, window, cx);
                                })),
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
                                .child(grant_consequence),
                        )
                        .child(
                            div()
                                .text_xs()
                                .line_height(px(18.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(keep_consequence),
                        ),
                ),
        );
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
            FREE_STATE,
            FREE_STATE_BODY,
            HELD_LINE,
            QUEUE_LINE_YOURS_NEXT,
            QUEUE_LINE_YOURS_POSITION,
            QUEUE_LINE_OTHERS,
            WAITER_ROW,
            ESCALATION_HEADLINE,
            GRANT_TO_WAITER_LABEL,
            GRANT_TO_WAITER_CONSEQUENCE,
            KEEP_HOLDER_LABEL,
            KEEP_HOLDER_CONSEQUENCE,
            DECISION_GRANTED_STATUS,
            DECISION_KEPT_STATUS,
            TASK_WAITING_LINE,
            TASK_HOLDING_LINE,
            TASK_AFFORDANCE_TOOLTIP,
            NOT_WIRED_NOTE,
            EMPTY_TITLE,
            EMPTY_BODY,
            NOTHING_TO_RESOLVE_GUIDANCE,
            BACK_TO_YOUR_WORK,
            BACK_TO_YOUR_WORK_TOOLTIP,
            ESCAPE_HINT,
            PALETTE_ROW_TITLE,
            PALETTE_ROW_DESCRIPTION,
            PALETTE_ROW_RESOLVE_TITLE,
            PALETTE_ROW_RESOLVE_DESCRIPTION,
            KEYBOARD_CHORD_LABEL,
        ]
        .iter()
        .map(|text| (*text).to_owned())
        .collect()
    }

    /// The copy states consequences and never leaks an implementation
    /// term (addendum §6: user language only — "Ana is using the
    /// browser — exclusive until 3:40 PM", never "lease ref" or
    /// "conflict policy enum").
    #[test]
    fn conflicts_copy_uses_user_language_with_consequences() {
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "lease ref",
                "lease id",
                "lease record",
                "lease-001",
                "flauz-lease",
                "res_",
                "lease_",
                "conflict policy",
                "access mode",
                "policy enum",
                "actor",
                "actorref",
                "snapshot",
                "queue position",
                "fifo",
                "escalation record",
                "canonical",
                "serde",
                "world store",
                "event stream",
                "view-model",
                "viewmodel",
                "ulid",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "conflicts copy must never say {forbidden:?}: {copy:?}"
                );
            }
        }
        // The work order's exact phrases are present, verbatim, WITH
        // their consequences.
        assert_eq!(PALETTE_ROW_TITLE, "See who is using what");
        assert_eq!(PALETTE_ROW_RESOLVE_TITLE, "Resolve a resource conflict");
        assert_eq!(
            ResourceStateView::Held {
                holder: "Ana".to_owned(),
                mode_word: "exclusive",
                until: "3:40 PM".to_owned(),
            }
            .line("the browser"),
            "Ana is using the browser — exclusive until 3:40 PM"
        );
        assert_eq!(
            QUEUE_LINE_YOURS_NEXT.replace("{count}", "2 tasks"),
            "2 tasks waiting — yours is next"
        );
        assert_eq!(
            task_waiting_line("the browser", 2),
            "Waiting for the browser — 2nd in line"
        );
        assert_eq!(
            EscalationView {
                holder: "Ana".to_owned(),
                until: "3:40 PM".to_owned(),
                waiter: "Dev".to_owned(),
                waited: "12 minutes".to_owned(),
                waiter_until: "4:30 PM".to_owned(),
            }
            .headline("the browser"),
            "Needs you: two tasks want the browser exclusively — Ana's task holds it until 3:40 \
             PM; Dev's task has waited 12 minutes"
        );
    }

    /// The honest states say so plainly: the not-wired note names what
    /// is and isn't connected; the empty state explains the surface;
    /// the free state says what it adds.
    #[test]
    fn the_honest_states_say_so_plainly() {
        assert!(NOT_WIRED_NOTE.contains("connects to live tasks"));
        assert!(NOT_WIRED_NOTE.contains("right now"));
        assert!(EMPTY_BODY.contains("this panel shows who holds it"));
        assert!(EMPTY_BODY.contains("No contention stays hidden"));
        assert!(FREE_STATE_BODY.contains("any task can use it right away"));
        assert!(NOTHING_TO_RESOLVE_GUIDANCE.contains("No resource conflict needs you"));
    }

    /// The queue lines are honest about counts and positions, in every
    /// shape: yours next, yours later, someone else's.
    #[test]
    fn queue_lines_are_honest_in_every_shape() {
        let dev = WaiterView {
            name: "Dev".to_owned(),
            waited: "12 minutes".to_owned(),
            is_you: true,
        };
        let mira = WaiterView {
            name: "Mira".to_owned(),
            waited: "4 minutes".to_owned(),
            is_you: false,
        };
        let row = ResourceConflictView {
            resource_label: "the browser".to_owned(),
            state: ResourceStateView::Free,
            waiters: vec![dev.clone()],
            escalation: None,
        };
        assert_eq!(
            row.queue_line().as_deref(),
            Some("1 task waiting — yours is next")
        );
        let row = ResourceConflictView {
            resource_label: "the browser".to_owned(),
            state: ResourceStateView::Free,
            waiters: vec![mira.clone(), dev],
            escalation: None,
        };
        assert_eq!(
            row.queue_line().as_deref(),
            Some("2 tasks waiting — yours is 2nd in line")
        );
        let row = ResourceConflictView {
            resource_label: "the browser".to_owned(),
            state: ResourceStateView::Free,
            waiters: vec![mira],
            escalation: None,
        };
        assert_eq!(
            row.queue_line().as_deref(),
            Some("1 task waiting — Mira is next")
        );
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(2), "2nd");
        assert_eq!(ordinal(3), "3rd");
        assert_eq!(ordinal(4), "4th");
        assert_eq!(ordinal(11), "11th");
        assert_eq!(ordinal(12), "12th");
        assert_eq!(ordinal(13), "13th");
        assert_eq!(ordinal(21), "21st");
        assert_eq!(
            task_holding_line("the browser", "3:40 PM"),
            "Using the browser until 3:40 PM"
        );
    }

    /// Layer 3: the palette fallback finds both rows through natural
    /// queries.
    #[test]
    fn palette_queries_resolve_to_the_rows() {
        for query in ["who", "using", "what", "resource", "holds"] {
            let title = PALETTE_ROW_TITLE.to_lowercase();
            let description = PALETTE_ROW_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the conflicts row"
            );
        }
        for query in ["resolve", "conflict", "two tasks", "same"] {
            let title = PALETTE_ROW_RESOLVE_TITLE.to_lowercase();
            let description = PALETTE_ROW_RESOLVE_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the resolve row"
            );
        }
    }

    /// The world-store seam records the frozen vocabulary, append-only,
    /// with the consequence line as the detail.
    #[test]
    fn the_seam_records_the_frozen_vocabulary() {
        assert_eq!(LEASE_GRANTED_EVENT_TYPE, "lease.granted");
        assert_eq!(LEASE_RELEASED_EVENT_TYPE, "lease.released");
        let mut log = ConflictsEventLog::new();
        assert!(log.events().is_empty());
        log.record(
            LEASE_GRANTED_EVENT_TYPE,
            "the browser",
            "Ana's task releases the browser and Dev's task takes over",
        );
        log.record(
            LEASE_RELEASED_EVENT_TYPE,
            "the browser",
            "Ana's task releases the browser and Dev's task takes over",
        );
        let events = log.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[1].seq, 2);
        assert_eq!(events[0].event_type, "lease.granted");
        assert_eq!(events[1].event_type, "lease.released");
        assert_eq!(events[0].resource, "the browser");
    }

    /// The escalation view's `needs_you` law: only a resource with an
    /// escalation card needs the human.
    #[test]
    fn only_escalated_resources_need_you() {
        let calm = ResourceConflictView {
            resource_label: "the browser".to_owned(),
            state: ResourceStateView::Free,
            waiters: Vec::new(),
            escalation: None,
        };
        assert!(!calm.needs_you());
        assert_eq!(calm.queue_line(), None);
        let contested = ResourceConflictView {
            resource_label: "the browser".to_owned(),
            state: ResourceStateView::Held {
                holder: "Ana".to_owned(),
                mode_word: "exclusive",
                until: "3:40 PM".to_owned(),
            },
            waiters: Vec::new(),
            escalation: Some(EscalationView {
                holder: "Ana".to_owned(),
                until: "3:40 PM".to_owned(),
                waiter: "Dev".to_owned(),
                waited: "12 minutes".to_owned(),
                waiter_until: "4:30 PM".to_owned(),
            }),
        };
        assert!(contested.needs_you());
    }

    /// The LEASE-001 registration seams in `ui.rs` (the house
    /// source-inspection style): the module declaration, the palette
    /// rows, the keyboard chord, the scoped escape, the state field,
    /// the title-bar entry, the task-surface mount, the
    /// workspace-root panel mount, the navigation close, and the chord
    /// LISTENER — each tagged LEASE-001, distinct from every prior
    /// wave's seams.
    #[test]
    fn conflicts_seams_are_registered_in_the_ui_seams() {
        let source = include_str!("../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_conflicts;"));
        assert!(source.contains("use flauz_conflicts::FlauzConflictsShortcut;"));

        // The palette registration seams (both rows).
        assert!(source.contains("PaletteCommand::SeeWhoIsUsingWhat"));
        assert!(source.contains("PaletteCommand::ResolveResourceConflict"));
        assert!(source.contains("flauz_conflicts::PALETTE_ROW_TITLE"));
        assert!(source.contains("flauz_conflicts::PALETTE_ROW_RESOLVE_TITLE"));
        assert!(source.contains("flauz_conflicts::open_conflicts_panel(workspace, window, cx)"));
        assert!(source.contains(
            "flauz_conflicts::open_conflicts_panel_for_resolution(workspace, window, cx)"
        ));

        // The keyboard chord seam (Ctrl+Alt+Shift+L — the letter
        // family, verified conflict-free: the letter family in use is
        // M (model picker), R (recovery), P (providers), S (save
        // flow), U (members); L is free).
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzConflictsShortcut, None"),
            "the conflicts chord must be bound"
        );
        assert!(source.contains("Some(\"Ctrl+Alt+Shift+L\")"));
        assert_eq!(
            source.matches("Some(\"Ctrl+Alt+Shift+L\")").count(),
            2,
            "both conflicts rows ride the chord (the providers precedent: the see row and the \
             resolve row open the same surface)"
        );

        // THE D25 LESSON (Gate B, the law): a KeyBinding without an
        // `.on_action` listener dispatches into the void — the chord
        // silently no-ops while the palette row works. The listener
        // must be registered on the workspace root next to the
        // picker/gap/agents/save/recovery/providers/members chord
        // listeners.
        let listener_form: String = "cx.listener(|this, _: &FlauzConflictsShortcut, window, cx| { \
             flauz_conflicts::open_conflicts_panel(this, window, cx); })"
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            normalized.contains(&listener_form),
            "the conflicts chord must have an on_action listener that calls \
             open_conflicts_panel (the d25 Gate-B lesson — the seam test pins the LISTENER, not \
             just the KeyBinding)"
        );

        // The binding registration itself (the alt-shift-l chord, the
        // letter family).
        assert!(source.contains("shortcut(\"alt-shift-l\")"));

        // The scoped escape seam (the d19 discipline: one Escape
        // through the panel's own focus context, never a trap).
        assert!(source.contains("Some(\"FlauzConflicts\")"));

        // The state-field seam.
        assert!(source.contains("flauz_conflicts: flauz_conflicts::ConflictsState"));
        assert!(source.contains("flauz_conflicts::ConflictsState::new(cx)"));

        // The title-bar entry seam (layer 1's visible primary entry).
        let joined: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            joined.contains("flauz_conflicts::render_conflicts_entry_button("),
            "the conflicts button must be mounted in the title bar"
        );

        // The task-surface mount seam (the wait/hold affordance).
        assert!(
            joined.contains("flauz_conflicts::render_task_resource_affordance("),
            "the wait/hold affordance must be mounted on the task surface"
        );

        // The workspace-root panel mount seam (the conflicts overlay).
        assert!(
            joined.contains("flauz_conflicts::render_conflicts_panel("),
            "the conflicts panel must be mounted at the workspace root"
        );

        // The navigation-close seam.
        assert!(source.contains("self.flauz_conflicts.close_for_navigation()"));

        // Every LEASE-001 seam is tagged.
        let seam_tags = source.matches("LEASE-001").count();
        assert!(
            seam_tags >= 10,
            "each seam is tagged LEASE-001 (found {seam_tags})"
        );
    }

    /// The d19 discipline, made checkable: the panel's focus path
    /// follows the request-once + restore contract.
    #[test]
    fn conflicts_focus_never_traps() {
        let source = include_str!("flauz_conflicts.rs");
        assert!(source.contains("panel_focus_requested = true"));
        assert!(source.contains("panel_focus_requested = false"));
        assert!(source.contains("focus_before_panel = window.focused(cx)"));
        assert!(source.contains("apply_overlay_close_focus_restore"));
        assert!(source.contains(".focus(window)"));
    }
}
