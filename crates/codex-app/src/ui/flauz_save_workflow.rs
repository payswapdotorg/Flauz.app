//! The Save-as-a-reusable-workflow slice (work order ORCH-004, F2 Wave
//! 3): the J-10/J-11 product loop — save a successful task's real
//! steps, discover them later from the Workspace nav, run them again as
//! a NEW task.
//!
//! The internal object is a Procedure (`flauz-world`'s contract); the
//! user-facing language is **reusable workflow** — the word "Procedure"
//! never appears in user-visible copy (tested).
//!
//! The slice:
//!
//! - the **affordance** — a labeled "Save as a reusable workflow"
//!   control on the task surface (the next-step neighborhood after a
//!   finished, verified run in the full product; honest on every task
//!   today), opening the save flow;
//! - the **save flow** — name the workflow, review the steps that
//!   actually ran (from the task's run shape — exactly what happened,
//!   never aspirational), save. The record lands through the
//!   **world-store seam** with the frozen event vocabulary
//!   (`procedure.created`);
//! - the **Workspace nav upgrade** — the "Reusable workflows" surface
//!   goes from its honest empty state (what reusable workflows are +
//!   how to save one) to listing saved workflows with **Run again**;
//! - **Run again** — seeds a NEW task from the workflow (a new task,
//!   never a fork of the old one — the identity discipline), with the
//!   **deviation hint** (a run that takes different steps says so; the
//!   full library is F10).
//!
//! Seven discoverability layers (PRODUCT-UX-JOURNEYS §1 / the shell
//! family):
//!
//! 1. **Visible primary entry** — the labeled "Save as a reusable
//!    workflow" control on the task surface.
//! 2. **Contextual affordance** — the save flow panel (from the
//!    affordance, the palette row or the chord) and the Run again flow
//!    from the Workspace nav.
//! 3. **Palette fallback** — the "Save this task as a reusable
//!    workflow" row plus the workspace's "Reusable workflows"
//!    navigation row (J-11's multiple routes).
//! 4. **Stateful empty state** — no saved workflows: what they are and
//!    how to save one; no run shape wired: the honest not-yet state
//!    (never invented steps).
//! 5. **Success-state continuation** — after save: "find it any time
//!    under Reusable workflows"; after run-again: a new task prepared
//!    from the workflow with the original untouched.
//! 6. **Keyboard path** — `Ctrl+Alt+Shift+S` opens the save flow; one
//!    scoped Escape closes it; the panels are tab-navigable.
//! 7. **Honest unavailable state** — no run shape is wired into live
//!    tasks yet: the save flow says so and invents nothing.
//!
//! Boundary rules (work order ORCH-004 / kernel §1):
//!
//! - the module wires through minimal `ui.rs` seams only (module
//!   declaration, palette registration, keyboard chord, the
//!   workspace-root action handler, the task-surface render
//!   registration, and the navigation close hook) — distinct from
//!   MOD-001's, CAP-001's and ORCH-003's seams;
//! - the app crate does not import the `flauz-world` contract crate in
//!   this wave (the app's Cargo.toml is outside this order's owned
//!   files): the saved-workflow record re-pins the minimal
//!   ProcedureVersion shape in plain view-model data, and the
//!   **world-store seam** ([`WorkflowEventLog`]) records the frozen
//!   event vocabulary a later slice wires to the real store;
//! - run-again never forks a task: the UI path prepares a brand-new
//!   chat (the app's new-task path, which clears any previous
//!   selection) seeded from the workflow, and the seam-level
//!   [`WorkflowCatalog::run_again`] refuses a "new" task id that
//!   equals the source task (the identity law, tested);
//! - the open/close paths follow the F1 focus contracts: capture the
//!   previously focused surface on open, auto-focus the panel handle
//!   once (the 019 request-once shape), restore on close (the 017
//!   contract).
//!
//! Every user-facing string lives in the copy registry below so the
//! language rules are unit-testable in one place.

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::{AnyElement, Context, Entity, FocusHandle, SharedString, Window, div, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Escape, Input, InputState},
    v_flex,
};

use super::WorkspaceView;

gpui::actions!(codexrs, [FlauzSaveWorkflowShortcut]);

/// The event type the world-store seam records when a workflow is
/// saved (the frozen `flauz-world` event vocabulary).
pub(crate) const PROCEDURE_CREATED_EVENT_TYPE: &str = "procedure.created";
/// The event type the world-store seam records when a run-again seeds
/// its new task (the frozen `flauz-world` event vocabulary; recorded
/// on the NEW task's stream).
pub(crate) const TASK_CREATED_EVENT_TYPE: &str = "task.created";

/// The entry affordance label on the task surface (J-10's primary
/// copy).
const ENTRY_LABEL: &str = "Save as a reusable workflow";
/// The entry affordance tooltip (the chord rides the letter family).
const ENTRY_TOOLTIP: &str =
    "Remember the successful steps of this task so you can run them again (Ctrl+Alt+Shift+S)";
/// The panel heading.
const PANEL_HEADING: &str = "Save as a reusable workflow";
/// The one-line description under the heading (J-10's secondary copy).
const PANEL_DESCRIPTION: &str = "Flauz can remember the successful steps so you can run them again";
/// The honest guidance when the chord or palette row fires with no chat
/// selected (the WO-P2-012 pattern).
const NO_CHAT_GUIDANCE: &str = "Open a chat to save its steps as a reusable workflow.";
/// The honest not-wired state's title.
const NOT_WIRED_TITLE: &str = "Saving isn't wired to live tasks yet";
/// What the save flow will hold (the not-wired state's body).
const NOT_WIRED_BODY: &str = "When a task finishes successfully, the steps that actually ran — \
     exactly what happened, nothing aspirational — can be saved here under a name you choose, \
     together with the task's inputs and the resources it used.";
/// The not-wired state's next step.
const NOT_WIRED_NEXT_STEP: &str = "Finish a task successfully, then open this panel again — the \
     run's real steps appear here once the run wiring lands.";
/// The name field's label.
const NAME_LABEL: &str = "Name this workflow";
/// The name field's placeholder.
const NAME_PLACEHOLDER: &str = "For example: Weekly brief research";
/// The steps preview heading.
const STEPS_HEADING: &str = "The steps that will be saved";
/// The honesty note under the steps preview.
const STEPS_NOTE: &str = "Only the steps that actually ran are saved — nothing aspirational.";
/// The inputs heading.
const INPUTS_HEADING: &str = "Inputs";
/// The resources heading.
const BINDINGS_HEADING: &str = "Resources used";
/// The save action.
const SAVE_BUTTON: &str = "Save workflow";
/// The close affordance.
const BACK_TO_YOUR_WORK: &str = "Back to your work";
/// The close tooltip.
const BACK_TO_YOUR_WORK_TOOLTIP: &str = "Return to your chat (Escape)";
/// The success status after a save (J-10: it appears in the workflows
/// surface).
const SAVED_STATUS: &str = "Saved — find it any time under Reusable workflows in the workspace";
/// The keyboard-hint footer.
const ESCAPE_HINT: &str = "Escape closes this panel";
/// The workflows surface card's title when workflows exist.
const LIST_TITLE: &str = "Your reusable workflows";
/// The workflows surface empty state's title.
const WORKFLOWS_EMPTY_TITLE: &str = "No reusable workflows yet";
/// The workflows surface empty state's body (what they are).
const WORKFLOWS_EMPTY_BODY: &str = "A reusable workflow remembers the successful steps of a task \
     — exactly what ran — so you can run them again on a new task any time.";
/// The workflows surface empty state's next step (how to save one).
const WORKFLOWS_EMPTY_NEXT_STEP: &str = "Finish a task successfully, then choose “Save as a \
     reusable workflow” on it — the saved workflow appears here.";
/// The run-again action.
const RUN_AGAIN_LABEL: &str = "Run again";
/// The run-again status (static — the app's status channel takes
/// static copy).
const RUN_AGAIN_STATUS: &str =
    "Started a new task from this saved workflow — the original task is untouched";
/// The deviation hint (a diverging run says so; the full library is
/// F10).
const DEVIATION_HINT: &str =
    "If a run takes different steps than the workflow, the run will say so — no silent drift.";
/// The command-palette row title (ORCH-004's exact palette row).
pub(crate) const PALETTE_ROW_TITLE: &str = "Save this task as a reusable workflow";
/// The command-palette row description.
pub(crate) const PALETTE_ROW_DESCRIPTION: &str =
    "Remember the successful steps of this task so you can run them again";
/// The chord label shown in tooltips (the letter family).
#[allow(dead_code)] // asserted by the module's tests; the chord text is embedded in the tooltip copy
pub(crate) const KEYBOARD_CHORD_LABEL: &str = "Ctrl+Alt+Shift+S";
/// The maximum length of a workflow name.
const MAX_WORKFLOW_NAME_BYTES: usize = 120;

// ---------------------------------------------------------------------------
// The run-shape + record view-models (the minimal ProcedureVersion
// re-pinned in plain data; the world-store seam wires them later)
// ---------------------------------------------------------------------------

/// One step of a saved workflow — a summary of what ACTUALLY ran.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // read by the render today; constructed by the module's tests and (Wave-later) the run-shape wiring
pub(crate) struct WorkflowStepView {
    /// The step's summary ("Research the week's sources").
    pub summary: String,
}

/// One input of a saved workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // read by the render today; constructed by the module's tests and (Wave-later) the run-shape wiring
pub(crate) struct WorkflowInputView {
    /// The input's name ("the week").
    pub name: String,
    /// The input's kind ("text", "file"), if declared.
    pub kind: Option<String>,
}

/// One resource binding of a saved workflow: the role a resource played.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // read by the render today; constructed by the module's tests and (Wave-later) the run-shape wiring
pub(crate) struct WorkflowBindingView {
    /// The role ("the shared browser").
    pub role: String,
    /// The bound resource's reference (data, never user copy).
    pub resource_ref: String,
}

/// A task's run shape: what ACTUALLY ran — the objective, the steps,
/// the inputs and the resources used. The wiring seam a later wave
/// populates from a real run; the save flow saves exactly this shape,
/// never an aspirational one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunShape {
    /// The task's objective.
    pub objective: String,
    /// The steps that actually ran, in order.
    pub steps: Vec<WorkflowStepView>,
    /// The task's inputs.
    pub inputs: Vec<WorkflowInputView>,
    /// The resources the task used.
    pub resource_bindings: Vec<WorkflowBindingView>,
}

/// A saved reusable workflow: the record the world-store seam persists
/// (the minimal ProcedureVersion shape) and the Workspace nav lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SavedWorkflow {
    /// The user-chosen name.
    pub name: String,
    /// The task whose successful run was saved (provenance; the
    /// original is never forked by a run-again).
    pub source_task_id: String,
    /// The run shape that was saved.
    pub shape: RunShape,
}

/// One world-store seam event: the frozen event vocabulary a later
/// slice wires to the real world store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkflowEvent {
    /// The strictly increasing sequence number.
    pub seq: u64,
    /// The event type (`procedure.created` on save; `task.created` on
    /// run-again, on the NEW task's stream).
    pub event_type: &'static str,
    /// The subject task (the source task on save; the new task on
    /// run-again).
    pub task_id: String,
    /// The workflow the event is about.
    pub workflow_name: String,
}

/// The world-store seam: an append-only log of the events the real
/// world store will record. Events are immutable and never mutated or
/// deleted (kernel §3).
#[derive(Debug, Default)]
pub(crate) struct WorkflowEventLog {
    events: Vec<WorkflowEvent>,
}

impl WorkflowEventLog {
    /// An empty event log.
    #[allow(dead_code)] // used by the module's tests; the Wave-later world-store wiring builds it in production
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn record(
        &mut self,
        event_type: &'static str,
        task_id: &str,
        workflow_name: &str,
    ) -> WorkflowEvent {
        let seq = self.events.len() as u64 + 1;
        let event = WorkflowEvent {
            seq,
            event_type,
            task_id: task_id.to_owned(),
            workflow_name: workflow_name.to_owned(),
        };
        self.events.push(event.clone());
        event
    }

    /// The recorded events, in append order.
    pub(crate) fn events(&self) -> &[WorkflowEvent] {
        &self.events
    }
}

/// Why a run-again was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunAgainError {
    /// No workflow exists at that index.
    UnknownWorkflow,
    /// The "new" task id equals the source task's id — a run-again
    /// creates a NEW task, never a fork of the old one (the identity
    /// law, kernel §6).
    ForkAttempt,
}

/// The staged run of the save flow: which task's shape is being saved.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StagedRun {
    task_id: String,
    shape: RunShape,
}

// ---------------------------------------------------------------------------
// The workflow catalog: the pure save→discover→run-again logic (no
// gpui types — unit-tested directly)
// ---------------------------------------------------------------------------

/// The workflow catalog: the per-task run shapes (the wiring seam), the
/// saved-workflow records (the Workspace nav lists these), the
/// staged save, and the world-store seam. Pure data + logic; the UI
/// state ([`SaveWorkflowState`]) wraps it.
#[derive(Debug, Default)]
pub(crate) struct WorkflowCatalog {
    workflows: Vec<SavedWorkflow>,
    run_shapes: HashMap<String, RunShape>,
    staged: Option<StagedRun>,
    event_log: WorkflowEventLog,
    /// Whether the last save succeeded (the success status shows until
    /// the panel closes).
    saved_status: bool,
}

impl WorkflowCatalog {
    /// An empty catalog.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// The wiring seam: records a task's run shape (a later wave calls
    /// this when a task's run is evaluated; the module's tests drive
    /// it).
    #[allow(dead_code)] // called by the module's tests; the Wave-later run wiring calls it in production
    pub(crate) fn set_run_shape(&mut self, task_id: &str, shape: RunShape) {
        self.run_shapes.insert(task_id.to_owned(), shape);
    }

    /// A task's run shape, if one is wired in.
    #[allow(dead_code)] // read by the module's tests; the Wave-later run wiring reads it in production
    pub(crate) fn run_shape(&self, task_id: &str) -> Option<&RunShape> {
        self.run_shapes.get(task_id)
    }

    /// The saved workflows, in save order (the Workspace nav lists
    /// these).
    pub(crate) fn workflows(&self) -> &[SavedWorkflow] {
        &self.workflows
    }

    /// The workflow at `index`, if any.
    pub(crate) fn workflow_at(&self, index: usize) -> Option<&SavedWorkflow> {
        self.workflows.get(index)
    }

    /// The world-store seam's recorded events, in append order.
    #[allow(dead_code)] // read by the module's tests; the Wave-later world-store wiring surfaces them
    pub(crate) fn events(&self) -> &[WorkflowEvent] {
        self.event_log.events()
    }

    /// The staged run shape of the in-flight save, if any.
    pub(crate) fn staged(&self) -> Option<&RunShape> {
        self.staged.as_ref().map(|staged| &staged.shape)
    }

    /// Whether the last save succeeded (the success status).
    pub(crate) fn saved_status(&self) -> bool {
        self.saved_status
    }

    /// Clears the staged save (the panel closed).
    pub(crate) fn clear_staged(&mut self) {
        self.staged = None;
        self.saved_status = false;
    }

    /// Stages the selected task's run shape for saving. Returns whether
    /// a run shape was found (without one, the panel shows the honest
    /// not-wired state — no steps are invented).
    pub(crate) fn begin_save(&mut self, task_id: &str) -> bool {
        let found = self.run_shapes.get(task_id).cloned();
        self.saved_status = false;
        match found {
            Some(shape) => {
                self.staged = Some(StagedRun {
                    task_id: task_id.to_owned(),
                    shape,
                });
                true
            }
            None => {
                self.staged = None;
                false
            }
        }
    }

    /// Confirms the save: builds the record from the staged run shape
    /// (the steps that actually ran — never aspirational), records the
    /// `procedure.created` event through the world-store seam, and
    /// clears the staged state.
    ///
    /// Returns the saved record, or a static reason when there is
    /// nothing to save or the name is invalid.
    pub(crate) fn confirm_save(&mut self, name: &str) -> Result<SavedWorkflow, &'static str> {
        let staged = self.staged.clone().ok_or("nothing to save")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("the workflow needs a name");
        }
        if name.len() > MAX_WORKFLOW_NAME_BYTES {
            return Err("the workflow name is too long");
        }
        let record = SavedWorkflow {
            name: name.to_owned(),
            source_task_id: staged.task_id,
            shape: staged.shape,
        };
        self.event_log.record(
            PROCEDURE_CREATED_EVENT_TYPE,
            &record.source_task_id,
            &record.name,
        );
        self.workflows.push(record.clone());
        self.staged = None;
        self.saved_status = true;
        Ok(record)
    }

    /// Seeds a run-again through the world-store seam: the workflow runs
    /// again on `new_task_id` — a NEW task, never a fork of the source
    /// task. Records the `task.created` event on the new task's stream.
    ///
    /// # Errors
    ///
    /// Returns [`RunAgainError::UnknownWorkflow`] when no workflow
    /// exists at `index`, and [`RunAgainError::ForkAttempt`] when
    /// `new_task_id` equals the workflow's source task (the identity
    /// law: run-again creates a new task, never forks the old one).
    #[allow(dead_code)] // driven by the module's tests; the Wave-later world-store wiring calls it in production (the UI path today prepares the new chat directly)
    pub(crate) fn run_again(
        &mut self,
        index: usize,
        new_task_id: &str,
    ) -> Result<&SavedWorkflow, RunAgainError> {
        let workflow = self
            .workflows
            .get(index)
            .ok_or(RunAgainError::UnknownWorkflow)?;
        if new_task_id == workflow.source_task_id {
            return Err(RunAgainError::ForkAttempt);
        }
        let name = workflow.name.clone();
        self.event_log
            .record(TASK_CREATED_EVENT_TYPE, new_task_id, &name);
        self.workflows
            .get(index)
            .ok_or(RunAgainError::UnknownWorkflow)
    }
}

// ---------------------------------------------------------------------------
// The save-workflow UI state (additive; the panel wraps the catalog)
// ---------------------------------------------------------------------------

/// The additive save-workflow state: the save panel (open flag, name
/// entry, focus bookkeeping) wrapping the pure workflow catalog. The
/// `ui.rs` seams only read and render it.
pub(crate) struct SaveWorkflowState {
    open: bool,
    /// The name entry for the save flow.
    name_input: Entity<InputState>,
    /// The pure save→discover→run-again logic.
    catalog: WorkflowCatalog,
    /// The panel's keyboard focus handle (the 019 request-once shape).
    panel_focus: FocusHandle,
    /// Request-once guard for `panel_focus`.
    panel_focus_requested: bool,
    /// The surface that held focus when the panel opened, restored on
    /// close (the 017 close contract).
    focus_before_panel: Option<FocusHandle>,
}

impl SaveWorkflowState {
    /// Builds the closed, empty save-workflow state.
    pub(crate) fn new(window: &mut Window, cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            open: false,
            name_input: cx.new(|cx| InputState::new(window, cx).placeholder(NAME_PLACEHOLDER)),
            catalog: WorkflowCatalog::new(),
            panel_focus: cx.focus_handle(),
            panel_focus_requested: false,
            focus_before_panel: None,
        }
    }

    /// Whether the save panel is open.
    #[allow(dead_code)] // the render paths check the `open` field directly; the Wave-later wiring reads it
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// The workflow catalog (the pure logic).
    pub(crate) fn catalog(&self) -> &WorkflowCatalog {
        &self.catalog
    }

    /// The workflow catalog, mutably (the open path stages a save).
    pub(crate) fn catalog_mut(&mut self) -> &mut WorkflowCatalog {
        &mut self.catalog
    }

    /// Quietly closes the panel for an F1 navigation action (no focus
    /// restore — the deliberate close paths restore through the 017
    /// contract instead). Returns whether anything changed.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.open;
        self.open = false;
        self.catalog.clear_staged();
        self.panel_focus_requested = false;
        self.focus_before_panel = None;
        changed
    }
}

// ---------------------------------------------------------------------------
// The UI actions
// ---------------------------------------------------------------------------

/// Opens (or toggles closed) the save flow on the selected chat.
/// Called from the task-surface affordance, the palette row, and the
/// keyboard chord. With no chat selected, surfaces honest guidance
/// instead of a silent no-op (the WO-P2-012 pattern).
pub(crate) fn open_save_flow(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.state.selected_task_id.is_none() {
        workspace.dispatch_command_status(Some(NO_CHAT_GUIDANCE), cx);
        return;
    }
    if workspace.flauz_save_workflow.open {
        close_save_flow(workspace, window, cx);
        return;
    }
    // The panel lives on the task workspace: navigate there first when
    // needed, so it is reachable from any route.
    if workspace.state.route != codex_core::MainRoute::Tasks {
        workspace.navigate(codex_core::MainRoute::Tasks, cx);
    }
    // One save flow at a time: opening it closes any open rail/shell
    // panel.
    super::flauz_shell::dismiss_shell_surfaces(workspace, window, cx);
    // Stage the selected task's run shape (None → the honest not-wired
    // state; nothing is invented).
    if let Some(task_id) = workspace.state.selected_task_id.clone() {
        workspace
            .flauz_save_workflow
            .catalog_mut()
            .begin_save(&task_id);
    }
    let state = &mut workspace.flauz_save_workflow;
    if state.focus_before_panel.is_none() {
        state.focus_before_panel = window.focused(cx);
    }
    state.open = true;
    state.panel_focus_requested = true;
    cx.notify();
}

/// Closes the save flow and restores focus (the 017 close contract).
pub(crate) fn close_save_flow(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if !workspace.flauz_save_workflow.open {
        return;
    }
    workspace.flauz_save_workflow.open = false;
    workspace.flauz_save_workflow.catalog_mut().clear_staged();
    workspace.flauz_save_workflow.panel_focus_requested = false;
    let previous = workspace.flauz_save_workflow.focus_before_panel.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// Saves the staged workflow from the panel (the Save button): reads
/// the name entry, confirms through the catalog's logic, and keeps the
/// panel open showing the success status.
pub(crate) fn save_from_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let name = workspace
        .flauz_save_workflow
        .name_input
        .read(cx)
        .value()
        .to_string();
    match workspace
        .flauz_save_workflow
        .catalog_mut()
        .confirm_save(&name)
    {
        Ok(_) => {
            workspace
                .flauz_save_workflow
                .name_input
                .update(cx, |input, cx| {
                    input.set_value("", window, cx);
                });
            cx.notify();
        }
        Err(reason) => {
            workspace.dispatch_command_status(Some(reason), cx);
        }
    }
}

/// Runs a saved workflow again: prepares a brand-new task (the app's
/// new-task path — a NEW task, never a fork of the old one) seeded
/// from the workflow, and surfaces the honest status. The durable
/// `task.created` record lands through the world-store seam when the
/// run wiring lands (the seam's pure path is
/// [`WorkflowCatalog::run_again`]).
pub(crate) fn run_saved_workflow_again(
    workspace: &mut WorkspaceView,
    index: usize,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let Some(workflow) = workspace.flauz_save_workflow.catalog().workflow_at(index) else {
        return;
    };
    let seed = workflow_seed(workflow);
    // A NEW task, never a fork: the app's new-task path clears any
    // previous selection and prepares a fresh task.
    workspace.begin_new_chat(window, cx);
    // Seed the new task from the workflow (the objective + the saved
    // steps, in user language).
    workspace
        .composer
        .update(cx, |input, cx| input.set_value(&seed, window, cx));
    workspace.dispatch_command_status(Some(RUN_AGAIN_STATUS), cx);
}

/// Builds the new task's seed text from a saved workflow (user
/// language only — the internal object is never named).
fn workflow_seed(workflow: &SavedWorkflow) -> String {
    let steps = workflow
        .shape
        .steps
        .iter()
        .map(|step| format!("- {}", step.summary))
        .collect::<Vec<_>>()
        .join("\n");
    let mut seed = format!(
        "Run the reusable workflow “{}”\n\nObjective: {}",
        workflow.name, workflow.shape.objective
    );
    if !steps.is_empty() {
        seed.push_str("\n\nSteps:\n");
        seed.push_str(&steps);
    }
    seed
}

// ---------------------------------------------------------------------------
// The renders
// ---------------------------------------------------------------------------

/// Renders the task-surface affordance (the visible, labeled Save
/// control) plus the open save-flow panel. Rendered in the task
/// surface's next-step neighborhood (the capability-gap entry's
/// neighborhood).
pub(crate) fn render_save_workflow_entry(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open = workspace.flauz_save_workflow.open;
    let mut entry = v_flex().flex_none();
    if workspace.flauz_save_workflow.panel_focus_requested {
        // The 019 request-once shape: the panel claims the keyboard on
        // mount so the scoped Escape binding reaches it.
        workspace.flauz_save_workflow.panel_focus_requested = false;
        workspace.flauz_save_workflow.panel_focus.focus(window);
    }
    entry = entry.child(
        h_flex()
            .h(px(32.0))
            .px_5()
            .items_center()
            .child(
                Button::new("flauz-save-workflow-entry")
                    .label(ENTRY_LABEL)
                    .icon(IconName::Star)
                    .tooltip(ENTRY_TOOLTIP)
                    .small()
                    .ghost()
                    .selected(open)
                    .on_click(cx.listener(|this, _, window, cx| {
                        open_save_flow(this, window, cx);
                    })),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(PANEL_DESCRIPTION),
            ),
    );
    if open {
        let state = &workspace.flauz_save_workflow;
        let staged = state.catalog.staged().cloned();
        let saved_status = state.catalog.saved_status();
        let name_input = state.name_input.clone();
        let panel_focus = state.panel_focus.clone();
        entry = entry.child(render_save_panel(
            panel_focus,
            name_input,
            staged.as_ref(),
            saved_status,
            cx,
        ));
    }
    entry.into_any_element()
}

/// Renders the save-flow panel: the name entry, the steps that will be
/// saved (from the staged run shape — exactly what ran), and the Save
/// action — or the honest not-wired state until a run shape is wired.
fn render_save_panel(
    panel_focus: FocusHandle,
    name_input: Entity<InputState>,
    staged: Option<&RunShape>,
    saved_status: bool,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let mut panel = v_flex()
        .key_context("FlauzSaveWorkflow")
        .track_focus(&panel_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            close_save_flow(this, window, cx);
        }))
        .mx_5()
        .my_2()
        .p_4()
        .gap_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().sidebar)
        .child(render_panel_header(cx));
    if saved_status {
        panel = panel.child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(IconName::CircleCheck).small())
                .child(div().text_sm().line_height(px(20.0)).child(SAVED_STATUS)),
        );
    }
    match staged {
        Some(shape) => {
            panel = panel
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(cx.theme().muted_foreground)
                                .child(NAME_LABEL),
                        )
                        .child(Input::new(&name_input).small().cleanable(true)),
                )
                .child(render_shape_preview(shape, cx))
                .child(
                    Button::new("flauz-save-workflow-confirm")
                        .label(SAVE_BUTTON)
                        .icon(IconName::Check)
                        .small()
                        .primary()
                        .on_click(cx.listener(|this, _, window, cx| {
                            save_from_panel(this, window, cx);
                        })),
                );
        }
        None => {
            panel = panel.child(render_not_wired_state(cx));
        }
    }
    panel = panel.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(ESCAPE_HINT),
    );
    panel.into_any_element()
}

/// Renders the panel header row (the house surface-header shape).
fn render_panel_header(cx: &mut Context<WorkspaceView>) -> AnyElement {
    h_flex()
        .gap_2()
        .items_center()
        .justify_between()
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_sm()
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
            Button::new("flauz-save-workflow-close")
                .label(BACK_TO_YOUR_WORK)
                .icon(IconName::ArrowLeft)
                .tooltip(BACK_TO_YOUR_WORK_TOOLTIP)
                .small()
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    close_save_flow(this, window, cx);
                })),
        )
        .into_any_element()
}

/// Renders the staged run shape's preview: the steps that actually ran,
/// the inputs, and the resources used.
fn render_shape_preview(shape: &RunShape, cx: &mut Context<WorkspaceView>) -> AnyElement {
    let mut preview = v_flex()
        .gap_1()
        .pt_2()
        .border_t_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(cx.theme().muted_foreground)
                .child(STEPS_HEADING),
        );
    for step in &shape.steps {
        preview = preview.child(
            h_flex()
                .gap_2()
                .items_start()
                .child(Icon::new(IconName::Check).small())
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .child(step.summary.clone()),
                ),
        );
    }
    preview = preview.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(STEPS_NOTE),
    );
    if !shape.inputs.is_empty() {
        preview = preview.child(
            v_flex()
                .gap_1()
                .pt_2()
                .border_t_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(cx.theme().muted_foreground)
                        .child(INPUTS_HEADING),
                )
                .children(shape.inputs.iter().map(|input| {
                    let text = match &input.kind {
                        Some(kind) => format!("{} ({kind})", input.name),
                        None => input.name.clone(),
                    };
                    div().text_sm().line_height(px(20.0)).child(text)
                })),
        );
    }
    if !shape.resource_bindings.is_empty() {
        preview = preview.child(
            v_flex()
                .gap_1()
                .pt_2()
                .border_t_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(cx.theme().muted_foreground)
                        .child(BINDINGS_HEADING),
                )
                .children(shape.resource_bindings.iter().map(|binding| {
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .child(binding.role.clone())
                })),
        );
    }
    preview.into_any_element()
}

/// Renders the honest not-wired state: the save flow is implemented,
/// the live run data is not wired yet — no steps are invented.
fn render_not_wired_state(cx: &mut Context<WorkspaceView>) -> AnyElement {
    v_flex()
        .w_full()
        .min_w_0()
        .max_w(px(560.0))
        .p_4()
        .gap_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(IconName::Star).small())
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(NOT_WIRED_TITLE),
                ),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(NOT_WIRED_BODY),
        )
        .child(
            v_flex()
                .gap_1()
                .pt_2()
                .border_t_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(cx.theme().muted_foreground)
                        .child("What to do next"),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .child(NOT_WIRED_NEXT_STEP),
                ),
        )
        .into_any_element()
}

/// Renders the Workspace nav's "Reusable workflows" surface content
/// (mounted by the shell inside the surface body): the honest empty
/// state, or the saved-workflow list with Run again and the deviation
/// hint.
pub(crate) fn render_workflows_card(
    workspace: &mut WorkspaceView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let workflows: Vec<SavedWorkflow> =
        workspace.flauz_save_workflow.catalog().workflows().to_vec();
    if workflows.is_empty() {
        return v_flex()
            .w_full()
            .min_w_0()
            .max_w(px(560.0))
            .p_4()
            .gap_2()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::Redo2).small())
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(WORKFLOWS_EMPTY_TITLE),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .line_height(px(20.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(WORKFLOWS_EMPTY_BODY),
            )
            .child(
                v_flex()
                    .gap_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(cx.theme().muted_foreground)
                            .child("What to do next"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(20.0))
                            .child(WORKFLOWS_EMPTY_NEXT_STEP),
                    ),
            )
            .into_any_element();
    }
    let mut card = v_flex()
        .w_full()
        .min_w_0()
        .max_w(px(560.0))
        .p_4()
        .gap_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().sidebar)
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(LIST_TITLE),
        );
    for (index, workflow) in workflows.iter().enumerate() {
        let details = format!(
            "{} steps · {} input{}",
            workflow.shape.steps.len(),
            workflow.shape.inputs.len(),
            if workflow.shape.inputs.len() == 1 {
                ""
            } else {
                "s"
            },
        );
        card = card.child(
            v_flex()
                .gap_1()
                .pt_2()
                .border_t_1()
                .border_color(cx.theme().border)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(workflow.name.clone()),
                        )
                        .child(
                            Button::new(SharedString::from(format!("flauz-run-again-{index}")))
                                .label(RUN_AGAIN_LABEL)
                                .icon(IconName::Redo2)
                                .small()
                                .primary()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    run_saved_workflow_again(this, index, window, cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(workflow.shape.objective.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(details),
                ),
        );
    }
    card = card.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(DEVIATION_HINT),
    );
    card.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_user_copy() -> Vec<&'static str> {
        vec![
            ENTRY_LABEL,
            ENTRY_TOOLTIP,
            PANEL_HEADING,
            PANEL_DESCRIPTION,
            NO_CHAT_GUIDANCE,
            NOT_WIRED_TITLE,
            NOT_WIRED_BODY,
            NOT_WIRED_NEXT_STEP,
            NAME_LABEL,
            NAME_PLACEHOLDER,
            STEPS_HEADING,
            STEPS_NOTE,
            INPUTS_HEADING,
            BINDINGS_HEADING,
            SAVE_BUTTON,
            BACK_TO_YOUR_WORK,
            BACK_TO_YOUR_WORK_TOOLTIP,
            SAVED_STATUS,
            ESCAPE_HINT,
            LIST_TITLE,
            WORKFLOWS_EMPTY_TITLE,
            WORKFLOWS_EMPTY_BODY,
            WORKFLOWS_EMPTY_NEXT_STEP,
            RUN_AGAIN_LABEL,
            RUN_AGAIN_STATUS,
            DEVIATION_HINT,
            PALETTE_ROW_TITLE,
            PALETTE_ROW_DESCRIPTION,
            KEYBOARD_CHORD_LABEL,
            "What to do next",
        ]
    }

    fn run_shape() -> RunShape {
        RunShape {
            objective: "Research, analyze and independently review the weekly brief".to_owned(),
            steps: vec![
                WorkflowStepView {
                    summary: "Research the week's sources".to_owned(),
                },
                WorkflowStepView {
                    summary: "Analyze the findings".to_owned(),
                },
                WorkflowStepView {
                    summary: "Independently review the results".to_owned(),
                },
            ],
            inputs: vec![WorkflowInputView {
                name: "the week".to_owned(),
                kind: Some("text".to_owned()),
            }],
            resource_bindings: vec![WorkflowBindingView {
                role: "the shared browser".to_owned(),
                resource_ref: "res_01J8ZQ5V8K3T2B7N6X4R9DQPD3".to_owned(),
            }],
        }
    }

    #[test]
    fn save_copy_uses_reusable_workflow_language_never_procedure() {
        // ORCH-004's exact law: user language ONLY — "reusable
        // workflow", never "Procedure" in any user-visible copy. The
        // seed text the run-again flow writes into the new task obeys
        // the same rule.
        let copy = all_user_copy();
        let seed = workflow_seed(&SavedWorkflow {
            name: "Weekly brief research".to_owned(),
            source_task_id: "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1".to_owned(),
            shape: run_shape(),
        });
        for text in copy.iter().chain(std::iter::once(&seed.as_str())) {
            let lower = text.to_lowercase();
            assert!(
                !lower.contains("procedure"),
                "user copy must never say \"Procedure\": {text:?}"
            );
            for forbidden in [
                "procedureversion",
                "flauz-world",
                "flauz_world",
                "orch-004",
                "view-model",
                "viewmodel",
                "runshape",
                "proc_",
                "world store",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "user copy must not leak the internal term {forbidden:?}: {text:?}"
                );
            }
        }
        assert!(seed.contains("reusable workflow"));
        assert!(seed.contains(
            "Objective: Research, analyze and independently review the weekly \
             brief"
        ));
        assert!(seed.contains("- Research the week's sources"));
    }

    #[test]
    fn every_state_of_the_flow_has_complete_copy() {
        // Discoverability contract §1.4/§1.7: every state explains
        // itself — no empty strings, no dead ends.
        for copy in all_user_copy() {
            assert!(!copy.is_empty());
        }
        assert!(ENTRY_TOOLTIP.contains("Ctrl+Alt+Shift+S"));
        assert!(NOT_WIRED_BODY.contains("actually ran"));
        assert!(STEPS_NOTE.contains("nothing aspirational"));
        assert!(WORKFLOWS_EMPTY_NEXT_STEP.contains("Save as a reusable workflow"));
        assert!(DEVIATION_HINT.contains("say so"));
        assert_eq!(KEYBOARD_CHORD_LABEL, "Ctrl+Alt+Shift+S");
    }

    #[test]
    fn the_save_discover_run_again_loop_works_end_to_end() {
        // The acceptance-criteria loop, at the seam level: save a
        // successful task's real steps → discover it in the catalog →
        // run it again as a NEW task, never a fork.
        let mut catalog = WorkflowCatalog::new();
        let source_task = "task_01J8ZQ5V8K3T2B7N6X4R9DQPB1";

        // Save: the run shape is wired in (the wiring seam), staged, and
        // saved under a name.
        catalog.set_run_shape(source_task, run_shape());
        assert!(catalog.begin_save(source_task));
        let record = catalog
            .confirm_save("Weekly brief research")
            .unwrap_or_else(|reason| panic!("{reason}"));
        assert_eq!(record.name, "Weekly brief research");
        assert_eq!(record.source_task_id, source_task);
        // The steps that will be saved are EXACTLY the run's steps —
        // never aspirational.
        assert_eq!(record.shape.steps, run_shape().steps);
        assert_eq!(record.shape.objective, run_shape().objective);
        assert_eq!(record.shape.inputs, run_shape().inputs);
        assert_eq!(
            record.shape.resource_bindings,
            run_shape().resource_bindings
        );
        assert!(catalog.saved_status());

        // The save is recorded through the world-store seam with the
        // frozen vocabulary.
        assert_eq!(catalog.events().len(), 1);
        assert_eq!(catalog.events()[0].seq, 1);
        assert_eq!(catalog.events()[0].event_type, PROCEDURE_CREATED_EVENT_TYPE);
        assert_eq!(catalog.events()[0].event_type, "procedure.created");
        assert_eq!(catalog.events()[0].task_id, source_task);

        // Discover: the catalog lists it (the Workspace nav surface
        // renders from this list).
        assert_eq!(catalog.workflows().len(), 1);
        assert_eq!(
            catalog.workflow_at(0).map(|w| w.name.as_str()),
            Some("Weekly brief research")
        );

        // Run again: a NEW task id — never the source task.
        let new_task = "task_01J8ZQ5V8K3T2B7N6X4R9DQPC2";
        assert_ne!(new_task, source_task);
        let again = catalog.run_again(0, new_task);
        assert_eq!(
            again.map(|w| w.name.as_str()),
            Ok("Weekly brief research"),
            "the run-again seeds the workflow"
        );
        assert_eq!(catalog.events().len(), 2);
        assert_eq!(catalog.events()[1].seq, 2);
        assert_eq!(catalog.events()[1].event_type, TASK_CREATED_EVENT_TYPE);
        assert_eq!(catalog.events()[1].event_type, "task.created");
        assert_eq!(catalog.events()[1].task_id, new_task);

        // The identity law: "running again" INTO the source task is a
        // fork attempt — refused.
        assert_eq!(
            catalog.run_again(0, source_task),
            Err(RunAgainError::ForkAttempt)
        );
        assert_eq!(
            catalog.run_again(9, new_task),
            Err(RunAgainError::UnknownWorkflow)
        );

        // The original record and events are immutable through the
        // loop; the source task's run shape is untouched.
        assert_eq!(catalog.workflows().len(), 1);
        assert_eq!(
            catalog.events().len(),
            2,
            "the fork attempt recorded nothing"
        );
        assert!(catalog.run_shape(source_task).is_some());
    }

    #[test]
    fn a_save_without_a_run_shape_is_honest_not_invented() {
        // No run shape wired → nothing staged → the panel shows the
        // honest not-wired state, and confirming saves nothing.
        let mut catalog = WorkflowCatalog::new();
        assert!(!catalog.begin_save("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"));
        assert!(catalog.staged().is_none());
        assert_eq!(catalog.confirm_save("Name"), Err("nothing to save"));
        assert!(catalog.workflows().is_empty());
        assert!(catalog.events().is_empty());
        assert!(!catalog.saved_status());
    }

    #[test]
    fn workflow_names_are_validated() {
        let mut catalog = WorkflowCatalog::new();
        catalog.set_run_shape("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1", run_shape());
        assert!(catalog.begin_save("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"));
        assert_eq!(
            catalog.confirm_save("   "),
            Err("the workflow needs a name")
        );
        assert_eq!(
            catalog.confirm_save(&"x".repeat(MAX_WORKFLOW_NAME_BYTES + 1)),
            Err("the workflow name is too long")
        );
        // A trimmed name is saved verbatim.
        let record = catalog
            .confirm_save("  Weekly brief research  ")
            .unwrap_or_else(|reason| panic!("{reason}"));
        assert_eq!(record.name, "Weekly brief research");
        // Clearing the staged save (the panel closing) forgets the
        // in-flight save but keeps the saved records.
        catalog.clear_staged();
        assert!(catalog.staged().is_none());
        assert_eq!(catalog.workflows().len(), 1);
    }

    #[test]
    fn palette_queries_resolve_to_the_save_row() {
        // Discoverability contract §1.3: the palette fallback finds the
        // surface without knowing its location — natural queries hit
        // the row's title or description (J-11's routes).
        let title = PALETTE_ROW_TITLE.to_lowercase();
        let description = PALETTE_ROW_DESCRIPTION.to_lowercase();
        for query in ["save", "workflow", "reusable", "again"] {
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the save palette row"
            );
        }
        assert_eq!(PALETTE_ROW_TITLE, "Save this task as a reusable workflow");
    }

    /// ORCH-004 registration seam test (the house ui.rs
    /// source-inspection style): the save flow must be wired through
    /// the palette, the keyboard, the task-surface render, and the
    /// navigation-close seams in `ui.rs`, and the Workspace nav surface
    /// must mount the workflows list.
    #[test]
    fn the_save_flow_is_registered_in_the_ui_and_shell_seams() {
        let source = include_str!("../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_save_workflow;"));

        // The palette registration seam.
        assert!(source.contains("PaletteCommand::SaveReusableWorkflow"));
        assert!(source.contains("flauz_save_workflow::open_save_flow("));

        // The keyboard registration seams: the save chord (the letter
        // family — letters report the shift modifier truthfully, so no
        // shifted-symbol companion is needed).
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzSaveWorkflowShortcut, None"),
            "a keyboard chord must be bound for the save flow"
        );
        assert!(source.contains("shortcut(\"alt-shift-s\")"));

        // The scoped Escape context for the save panel.
        assert!(source.contains("Some(\"FlauzSaveWorkflow\")"));

        // The task-surface render registration seam (the affordance +
        // panel, in the next-step neighborhood).
        assert!(source.contains("flauz_save_workflow::render_save_workflow_entry"));

        // The navigation-close seam.
        assert!(source.contains("self.flauz_save_workflow.close_for_navigation()"));

        // The shell mount seam: the Workspace nav's Reusable workflows
        // surface renders this module's card.
        let shell = include_str!("flauz_shell/mod.rs");
        assert!(
            shell.contains("flauz_save_workflow::render_workflows_card"),
            "the Reusable workflows surface mounts this module's card"
        );
    }
}
