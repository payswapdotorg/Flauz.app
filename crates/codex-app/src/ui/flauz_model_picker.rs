//! The model-selection surface (MOD-001 + RT-001, F3 model/runtime
//! fabric) — GUI module.
//!
//! This module implements the model picker on the task surface: the
//! user-visible slice of the model fabric (the registry, the runtimes,
//! and the capability metadata live behind the frozen contracts in
//! `flauz-exec`). Seven discoverability layers (the PRODUCT-UX-JOURNEYS
//! §1 / d23 discipline the shell family follows):
//!
//! 1. **Visible labeled control** — a `Model: …` control on every task
//!    surface, showing the active model's name or an honest "not chosen
//!    yet" — never hidden behind developer settings.
//! 2. **Picker panel** — opened from the control, the palette, or the
//!    keyboard chord; lists the available models with their provider and
//!    capability hints.
//! 3. **Palette row** — "Choose a model…" under the Workspace group.
//! 4. **Keyboard path** — `Ctrl+Alt+Shift+M` opens the picker; Escape
//!    closes it through the scoped focus context.
//! 5. **Honest empty state** — no models configured: the panel explains
//!    what a model provides and how to connect one (pointing at
//!    Settings), and implements NO connection flow — that is F7.
//! 6. **Success state** — models present: the list with provider +
//!    capability hints; after a switch, the active model is named and
//!    the task identity is visibly preserved (J-13/J-14: switching a
//!    model never forks the task).
//! 7. **Honest unavailable state** — models from providers that are only
//!    configured, or not configured at all, are listed with their honest
//!    availability and cannot be selected — the UI never implies a
//!    connection exists when it does not (the MOD-001 integration note).
//!
//! Boundary rules (work order MOD-001 + RT-001 / kernel §1):
//!
//! - the picker does NOT import the Flauz contract crates in this wave —
//!   the **catalog seam** ([`PickerModelEntry`]) and the **world store
//!   seam** ([`TaskModelEventLog`]) below are the minimal, unit-testable
//!   shapes later slices wire to the real registry (`flauz-exec`) and
//!   the real world store (`flauz-world`); the seams keep the frozen
//!   vocabulary (the three honest availability states; the
//!   `task.model_changed` event type name);
//! - model choice is execution state, never task identity (kernel §6):
//!   a switch records a `task.model_changed` event on the task's stream
//!   through the world store seam and changes nothing else — the task
//!   ID is the identity, and it is preserved by construction;
//! - existing F1 flows are untouched: the module is wired through
//!   minimal `ui.rs` named seams (module declaration, palette
//!   registration, keyboard chord, the task-surface control, the panel
//!   mount, the navigation close) — distinct from any other worker's
//!   seams;
//! - the F1 focus contracts (017 palette close restore, 019 modal focus
//!   traps) are followed by the picker's own open/close paths: capture
//!   the previously focused surface on open, auto-focus the picker's
//!   handle once, restore on close.
//!
//! Every user-facing string lives in the copy registry below so the
//! language rules (user language, never internal type names) are
//! unit-testable in one place.

use std::collections::HashMap;

use codex_core::Action;
use gpui::prelude::*;
use gpui::{AnyElement, Context, FocusHandle, IntoElement, SharedString, Window, div, px};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::Escape,
    scroll::ScrollableElement,
    v_flex,
};

use super::WorkspaceView;

gpui::actions!(codexrs, [FlauzModelPickerShortcut]);

/// The event type the world store seam records when the model executing
/// a task changes (the frozen `flauz-world` event vocabulary; the task
/// identity is preserved — kernel §6).
pub(crate) const MODEL_CHANGED_EVENT_TYPE: &str = "task.model_changed";

// ---------------------------------------------------------------------------
// Copy registry (user language; no internal type names — J-10/J-11)
// ---------------------------------------------------------------------------

/// The labeled control's noun: what the control is called on the task
/// surface (layer 1).
pub(crate) const MODEL_CONTROL_LABEL: &str = "Model";

/// The control label when no model is active for the task (honest empty
/// control state; the tooltip and the panel carry the action).
pub(crate) const MODEL_CONTROL_EMPTY_LABEL: &str = "Model: not chosen yet";

/// The control tooltip when a model is active (the chord label matches
/// the keyboard registration).
pub(crate) const MODEL_CONTROL_TOOLTIP_ACTIVE: &str =
    "Switch which model runs this task (Ctrl+Alt+Shift+M)";

/// The control tooltip when no model is active.
pub(crate) const MODEL_CONTROL_TOOLTIP_EMPTY: &str =
    "Choose which model runs this task (Ctrl+Alt+Shift+M)";

/// The honest guidance when the picker chord or palette row fires with
/// no chat selected (the WO-P2-012 pattern: guidance, never a silent
/// no-op).
pub(crate) const MODEL_PICKER_NO_CHAT_GUIDANCE: &str = "Open a chat to choose which model runs it.";

/// The command-palette row title (layer 3).
pub(crate) const PALETTE_ROW_TITLE: &str = "Choose a model…";

/// The command-palette row description.
pub(crate) const PALETTE_ROW_DESCRIPTION: &str =
    "See the models available to this task and switch between them";

/// The picker panel heading (layer 2).
pub(crate) const MODEL_PICKER_HEADING: &str = "Choose a model";

/// The one-line description under the panel heading: what a model is,
/// and the identity promise (J-13/J-14).
pub(crate) const MODEL_PICKER_DESCRIPTION: &str = "The model is the engine that reads your \
     instructions and does the work. Switching models keeps this task — its objective, its \
     plan, and everything it produced — exactly as it is.";

/// The honest empty-state title (layer 5): no models configured.
pub(crate) const MODEL_EMPTY_TITLE: &str = "No models connected yet";

/// The honest empty-state body: what a model provides.
pub(crate) const MODEL_EMPTY_BODY: &str = "A model is what does the work of a task — reading \
     your instructions, writing, analyzing, and planning. Different models bring different \
     strengths; for example, some can understand images. Models become available when you \
     connect a provider.";

/// The honest empty-state next step: how to connect one, pointing at
/// Settings. This slice implements NO connection flow (that is F7); the
/// pointer is the honest next step.
pub(crate) const MODEL_EMPTY_NEXT_STEP: &str = "To make models available, connect a provider: \
     open Settings, then Connections, and follow the steps there. Models from connected \
     providers appear in this panel, ready to run your tasks.";

/// The empty-state button label (an existing F1 settings surface, not a
/// new connection flow).
pub(crate) const MODEL_EMPTY_SETTINGS_BUTTON: &str = "Open Connections settings";

/// The success-state label naming the active model (layer 6).
pub(crate) const MODEL_ACTIVE_PREFIX: &str = "Now running";

/// The success-state identity line after a switch: the active model is
/// named and the task identity is visibly preserved.
pub(crate) const MODEL_SWITCHED_CONFIRMATION: &str = "Switched. This task keeps its objective, \
     its history, and everything it produced.";

/// The provider line for a runnable model: the BYOP preview — the UI
/// names the provider the model draws on (F7 does the full connection
/// flows and quota metering).
pub(crate) const MODEL_DRAWS_ON_PREFIX: &str = "Draws on your connected";

/// The honest unavailable label for a provider that is configured but
/// not connected (layer 7): never implying a connection exists.
pub(crate) const MODEL_UNAVAILABLE_CONFIGURED: &str =
    "Not connected yet — connect this provider in Settings to run its models.";

/// The honest unavailable label for a provider with no connection
/// configured at all (layer 7).
pub(crate) const MODEL_UNAVAILABLE_NOT_CONFIGURED: &str =
    "Needs setup — this provider has no connection configured yet.";

/// The panel footer hint (the keyboard close path).
pub(crate) const MODEL_PICKER_ESCAPE_HINT: &str = "Escape closes this panel";

/// The list header for the catalog rows.
pub(crate) const MODEL_LIST_CAPTION: &str = "Available models";

// ---------------------------------------------------------------------------
// The catalog seam (mirrors the registry's honest states; wired to the
// real registry by a later slice)
// ---------------------------------------------------------------------------

/// The availability of a model's provider, mirroring the registry's
/// three honest states (MOD-001): connected / configured-not-connected /
/// not-configured. The UI must never imply a connection exists when the
/// provider is only configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // variants are matched by the render today; constructed by the module's tests and (F7) the registry→catalog wiring
pub(crate) enum PickerModelAvailability {
    /// No connection is stored for the provider: nothing is configured.
    NotConfigured,
    /// A connection is stored but is not connected.
    ConfiguredNotConnected,
    /// A connection is stored and connected: the provider's models can
    /// run and draw on the connection's account.
    Connected,
}

impl PickerModelAvailability {
    /// The only state in which a model can be selected to run.
    pub(crate) const fn is_runnable(self) -> bool {
        matches!(self, Self::Connected)
    }

    /// The honest user-facing label for this state.
    pub(crate) const fn user_label(self) -> &'static str {
        match self {
            Self::Connected => "Connected",
            Self::ConfiguredNotConnected => "Not connected yet",
            Self::NotConfigured => "Needs setup",
        }
    }
}

/// One model in the picker's catalog: the seam a later slice wires to
/// the real registry's model metadata (model name, provider, capability
/// keys, and the provider's honest availability). Copy only — no
/// credential material ever reaches the UI (the registry stores
/// references only; addendum §3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PickerModelEntry {
    /// The model's canonical identifier (opaque; never shown as copy).
    pub(crate) model_id: String,
    /// The human-readable model name (the active-model copy).
    pub(crate) model_name: String,
    /// The provider's neutral kind label (the registry key; data, not
    /// UI copy).
    pub(crate) provider_kind: String,
    /// The provider's user-facing label.
    pub(crate) provider_label: String,
    /// The model's capability keys (the frozen vocabulary; rendered as
    /// user-facing hints).
    pub(crate) capabilities: Vec<String>,
    /// The provider's honest availability.
    pub(crate) availability: PickerModelAvailability,
}

impl PickerModelEntry {
    /// Builds a catalog entry.
    #[allow(dead_code)] // used by the module's tests; the F7 registry wiring builds production entries through it
    pub(crate) fn new(
        model_id: &str,
        model_name: &str,
        provider_kind: &str,
        provider_label: &str,
        capabilities: Vec<String>,
        availability: PickerModelAvailability,
    ) -> Self {
        Self {
            model_id: model_id.to_owned(),
            model_name: model_name.to_owned(),
            provider_kind: provider_kind.to_owned(),
            provider_label: provider_label.to_owned(),
            capabilities,
            availability,
        }
    }

    /// The user-facing capability hints for this model's capability keys.
    pub(crate) fn capability_hints(&self) -> Vec<String> {
        self.capabilities
            .iter()
            .map(|key| capability_hint(key))
            .collect()
    }
}

/// Renders one capability key as a user-facing hint. Known keys of the
/// frozen vocabulary map to plain language; unknown keys pass through
/// unchanged — capability keys are the platform's public feature
/// vocabulary (named capabilities, J-04), never internal type names.
pub(crate) fn capability_hint(key: &str) -> String {
    match key {
        "text" => "Handles text".to_owned(),
        "vision" => "Understands images".to_owned(),
        "image.input" => "Accepts images".to_owned(),
        "terminal" => "Can use a terminal".to_owned(),
        "browser.input" => "Can drive a browser".to_owned(),
        "browser.navigation" => "Can navigate websites".to_owned(),
        other => other.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// The world store seam (task.model_changed events; wired to the real
// world store by a later slice)
// ---------------------------------------------------------------------------

/// One `task.model_changed` event recorded on a task's stream: the seam
/// a later slice wires to the real event envelope. The task ID is
/// recorded on every event and never changes — model choice is
/// execution state, never task identity (kernel §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskModelEvent {
    /// The strictly increasing per-stream sequence number.
    pub(crate) seq: u64,
    /// The event type name (always [`MODEL_CHANGED_EVENT_TYPE`]).
    pub(crate) event_type: &'static str,
    /// The task the event belongs to. The identity: never changes.
    pub(crate) task_id: String,
    /// The model that ran the task before the switch, if any.
    pub(crate) from_model: Option<String>,
    /// The model that runs the task after the switch.
    pub(crate) to_model: String,
}

/// The world store seam: an append-only log of `task.model_changed`
/// events. Events are immutable and never mutated or deleted (kernel
/// §3); appends never change a task's ID and never fork a task.
#[derive(Debug, Default)]
pub(crate) struct TaskModelEventLog {
    events: Vec<TaskModelEvent>,
}

impl TaskModelEventLog {
    /// An empty event log.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Records a model switch on the task's stream, assigning the next
    /// strictly increasing sequence number. Returns the recorded event.
    /// The task ID is stored verbatim — the identity is preserved by
    /// construction.
    pub(crate) fn record_model_changed(
        &mut self,
        task_id: &str,
        from_model: Option<&str>,
        to_model: &str,
    ) -> TaskModelEvent {
        let seq = self.events.len() as u64 + 1;
        let event = TaskModelEvent {
            seq,
            event_type: MODEL_CHANGED_EVENT_TYPE,
            task_id: task_id.to_owned(),
            from_model: from_model.map(str::to_owned),
            to_model: to_model.to_owned(),
        };
        self.events.push(event.clone());
        event
    }

    /// The recorded events, in append order.
    #[allow(dead_code)] // read by the module's tests and the state-level accessor; the F7 wiring surfaces it
    pub(crate) fn events(&self) -> &[TaskModelEvent] {
        &self.events
    }
}

// ---------------------------------------------------------------------------
// The picker state (additive UI state; all logic lives here)
// ---------------------------------------------------------------------------

/// The additive model-picker state: the open panel, the catalog seam,
/// the per-task active model (execution state), and the world store
/// seam. The `ui.rs` seams only read and render it.
pub(crate) struct FlauzModelPickerState {
    open: bool,
    /// The catalog seam: the models available to pick. Empty at cold
    /// start (the honest empty state); a later slice wires the real
    /// registry through [`Self::set_catalog`].
    catalog: Vec<PickerModelEntry>,
    /// The active model per task (execution state, never identity).
    active_models: HashMap<String, String>,
    /// The world store seam: the `task.model_changed` event log.
    event_log: TaskModelEventLog,
    /// The picker panel's keyboard focus handle (the 019 request-once
    /// shape: auto-focused on the first render after an open).
    picker_focus: FocusHandle,
    /// Request-once guard for `picker_focus`.
    picker_focus_requested: bool,
    /// The surface that held focus when the picker opened, restored on
    /// close (the 017 close contract).
    focus_before_picker: Option<FocusHandle>,
}

/// The model-switch decision shared by the state machine: a switch
/// records a `task.model_changed` event through the world store seam and
/// updates the per-task execution state; the task ID is untouched —
/// switching a model never forks a task (kernel §6).
///
/// Returns `Some` (the recorded event) when the model changed and `None`
/// when it did not (the same model, an unknown model, or a model whose
/// provider is not connected — the honest unavailable state: such a model
/// is never selected, and no event is recorded).
fn switch_model_through_seam(
    catalog: &[PickerModelEntry],
    active_models: &mut HashMap<String, String>,
    log: &mut TaskModelEventLog,
    task_id: &str,
    model_id: &str,
) -> Option<TaskModelEvent> {
    if active_models.get(task_id).map(String::as_str) == Some(model_id) {
        return None;
    }
    let is_runnable = catalog
        .iter()
        .any(|entry| entry.model_id == model_id && entry.availability.is_runnable());
    if !is_runnable {
        return None;
    }
    let from_model = active_models.get(task_id).cloned();
    let event = log.record_model_changed(task_id, from_model.as_deref(), model_id);
    active_models.insert(task_id.to_owned(), model_id.to_owned());
    Some(event)
}

impl FlauzModelPickerState {
    /// Builds the closed, empty picker state (the cold-start honest
    /// empty state: no catalog, no active models, no events).
    pub(crate) fn new(cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            open: false,
            catalog: Vec::new(),
            active_models: HashMap::new(),
            event_log: TaskModelEventLog::new(),
            picker_focus: cx.focus_handle(),
            picker_focus_requested: false,
            focus_before_picker: None,
        }
    }

    /// Whether the picker panel is open.
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// The catalog seam's current entries (the honest empty state at
    /// cold start is an empty catalog).
    pub(crate) fn catalog(&self) -> &[PickerModelEntry] {
        &self.catalog
    }

    /// The catalog seam for a later slice: replaces the catalog (the
    /// real registry wiring — F7 — calls this when the registry loads).
    #[allow(dead_code)] // F7 wiring seam; exercised by the module's tests today
    pub(crate) fn set_catalog(&mut self, catalog: Vec<PickerModelEntry>) {
        self.catalog = catalog;
    }

    /// The active model's identifier for a task, if any (execution
    /// state; the task's identity is its ID, unchanged).
    #[allow(dead_code)] // F7 wiring seam; exercised by the module's tests today
    pub(crate) fn active_model_id(&self, task_id: &str) -> Option<&str> {
        self.active_models.get(task_id).map(String::as_str)
    }

    /// The active model's catalog entry for a task, if any.
    pub(crate) fn active_model(&self, task_id: &str) -> Option<&PickerModelEntry> {
        self.active_models.get(task_id).and_then(|model_id| {
            self.catalog
                .iter()
                .find(|entry| &entry.model_id == model_id)
        })
    }

    /// The world store seam's recorded events, in append order.
    #[allow(dead_code)] // F7 wiring seam; exercised by the module's tests today
    pub(crate) fn events(&self) -> &[TaskModelEvent] {
        self.event_log.events()
    }

    /// Switches the model running a task: records a
    /// `task.model_changed` event on the task's stream through the world
    /// store seam and updates the execution state. The task ID is
    /// untouched — switching a model never forks a task (kernel §6).
    ///
    /// Returns `Some` (the recorded event) when the model changed and
    /// `None` when it did not (the same model, an unknown model, or a
    /// model whose provider is not connected — the honest unavailable
    /// state: such a model is never selected, and no event is recorded).
    pub(crate) fn select_model(&mut self, task_id: &str, model_id: &str) -> Option<TaskModelEvent> {
        switch_model_through_seam(
            &self.catalog,
            &mut self.active_models,
            &mut self.event_log,
            task_id,
            model_id,
        )
    }

    /// Closes the panel for an F1 navigation action. Returns whether
    /// anything changed (the caller notifies). Focus is left where the
    /// navigation action itself puts it — the deliberate close paths
    /// (Escape, the control toggle) restore through the 017 contract.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.open;
        self.open = false;
        self.picker_focus_requested = false;
        self.focus_before_picker = None;
        changed
    }
}

/// Whether an F1 action navigates the main area or mounts another
/// surface, and therefore closes the picker panel (the navigation close
/// seam in `WorkspaceView::dispatch`). The reducer is untouched — the
/// picker is additive UI state.
pub(crate) fn action_closes_model_picker(action: &Action) -> bool {
    matches!(
        action,
        Action::Navigate(_)
            | Action::BeginNewChat
            | Action::BeginProjectlessChat
            | Action::SelectTask(_)
            | Action::ArchiveTask(_)
            | Action::OpenSideChat
            | Action::ForkSelectedTask
            | Action::ShowInspector(_)
            | Action::ToggleTerminalDock
            | Action::ToggleBottomPanel
            | Action::OpenBrowserTab
            | Action::ToggleBrowserPanel
            | Action::ToggleActivityView
    )
}

// ---------------------------------------------------------------------------
// Entry points (the ui.rs seams call these)
// ---------------------------------------------------------------------------

/// Opens (or toggles closed) the model picker on the selected chat.
/// Called from the task-surface control, the palette row, and the
/// direct keyboard chord. With no chat selected, surfaces honest
/// guidance instead of a silent no-op (the WO-P2-012 pattern).
pub(crate) fn open_model_picker(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.state.selected_task_id.is_none() {
        workspace.dispatch_command_status(Some(MODEL_PICKER_NO_CHAT_GUIDANCE), cx);
        return;
    }
    // The picker lives on the task workspace: navigate there first when
    // needed (the dispatch seam closes the panel when it is not the
    // deliberate close), so the picker is reachable from any route.
    // When already there — the control and the in-task chord — no
    // navigation runs, preserving the toggle-close contract below.
    if workspace.state.route != codex_core::MainRoute::Tasks {
        workspace.navigate(codex_core::MainRoute::Tasks, cx);
    }
    let picker = &mut workspace.flauz_model_picker;
    if picker.open {
        dismiss_model_picker(workspace, window, cx);
        return;
    }
    if picker.focus_before_picker.is_none() {
        picker.focus_before_picker = window.focused(cx);
    }
    picker.open = true;
    picker.picker_focus_requested = true;
    cx.notify();
}

/// Closes the picker panel and restores focus (the 017 close contract:
/// the surface that held focus before the open, else the composer house
/// default when rendered, else no element focus).
pub(crate) fn dismiss_model_picker(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if !workspace.flauz_model_picker.close_for_navigation() {
        return;
    }
    let previous = workspace.flauz_model_picker.focus_before_picker.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// Switches the model running the selected task (the picker rows call
/// this): records the `task.model_changed` event through the world store
/// seam, keeps the task identity, and leaves the panel open showing the
/// success state (the active model named, the task preserved). Unknown
/// or not-runnable models record nothing (honest unavailable state).
pub(crate) fn select_model_for_task(
    workspace: &mut WorkspaceView,
    task_id: &str,
    model_id: &str,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace
        .flauz_model_picker
        .select_model(task_id, model_id)
        .is_some()
    {
        cx.notify();
    }
}

/// The visible labeled model control on the task surface (layer 1):
/// `Model: {active model name}` when a model is active, the honest
/// `Model: not chosen yet` when none is. Always labeled, always
/// clickable — the entry point never depends on the palette or the
/// chord being discovered first.
pub(crate) fn render_model_control(
    workspace: &mut WorkspaceView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let task_id = workspace.state.selected_task_id.clone();
    let active = task_id
        .as_deref()
        .and_then(|task_id| workspace.flauz_model_picker.active_model(task_id));
    let (label, tooltip) = match active {
        Some(entry) => (
            format!("{MODEL_CONTROL_LABEL}: {}", entry.model_name),
            MODEL_CONTROL_TOOLTIP_ACTIVE,
        ),
        None => (
            MODEL_CONTROL_EMPTY_LABEL.to_owned(),
            MODEL_CONTROL_TOOLTIP_EMPTY,
        ),
    };
    h_flex()
        .flex_none()
        .px_5()
        .py_1()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            Button::new("flauz-model-control")
                .label(SharedString::from(label))
                .icon(IconName::Bot)
                .tooltip(tooltip)
                .small()
                .ghost()
                .selected(workspace.flauz_model_picker.is_open())
                .on_click(cx.listener(|this, _, window, cx| {
                    open_model_picker(this, window, cx);
                })),
        )
        .into_any_element()
}

/// The picker panel (layers 2, 5, 6, 7): mounted under the task rail
/// while a task is selected; renders nothing when closed. The panel
/// auto-focuses its own handle once on mount (the 019 request-once
/// shape) so the scoped Escape binding reaches it.
pub(crate) fn render_model_picker(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let Some(task_id) = workspace.state.selected_task_id.clone() else {
        return v_flex().flex_none().into_any_element();
    };
    let picker = &mut workspace.flauz_model_picker;
    if !picker.is_open() {
        return v_flex().flex_none().into_any_element();
    }
    if picker.picker_focus_requested {
        // The 019 request-once shape: the panel claims the keyboard on
        // mount so the scoped Escape binding reaches it.
        picker.picker_focus_requested = false;
        picker.picker_focus.focus(window);
    }
    let picker_focus = picker.picker_focus.clone();
    let catalog = picker.catalog().to_vec();
    let active = picker.active_model(&task_id).cloned();
    let task_title = workspace
        .state
        .tasks
        .iter()
        .find(|task| task.id == task_id)
        .map(|task| task.title.clone())
        .unwrap_or_default();

    let mut panel = v_flex()
        .key_context("FlauzModelPicker")
        .track_focus(&picker_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            dismiss_model_picker(this, window, cx);
        }))
        .flex_none()
        .mx_5()
        .my_3()
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
                .child(Icon::new(IconName::Bot).small())
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(MODEL_PICKER_HEADING),
                ),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(MODEL_PICKER_DESCRIPTION),
        );

    if catalog.is_empty() {
        // Layer 5 — the honest empty state: what a model provides and
        // how to connect one, pointing at Settings (the connection flow
        // itself is F7; this slice implements none of it).
        panel = panel
            .child(
                v_flex()
                    .gap_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(MODEL_EMPTY_TITLE),
                    )
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(20.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(MODEL_EMPTY_BODY),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .pt_2()
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
                                    .child(MODEL_EMPTY_NEXT_STEP),
                            ),
                    ),
            )
            .child(
                Button::new("flauz-model-open-connections")
                    .label(MODEL_EMPTY_SETTINGS_BUTTON)
                    .icon(IconName::Settings)
                    .small()
                    .primary()
                    .on_click(cx.listener(|this, _, _window, cx| {
                        // An existing F1 settings surface — the honest
                        // next step, NOT a connection flow (F7). The
                        // navigation closes the panel through the
                        // dispatch seam.
                        this.open_settings_section(super::SettingsSection::Connections, cx);
                    })),
            );
    } else {
        // Layer 6 — the success state: the active model named, the
        // provider named, capability hints, and the identity preserved.
        if let Some(active) = &active {
            let hints = active.capability_hints();
            let hint_copy = if hints.is_empty() {
                String::new()
            } else {
                format!(" · {}", hints.join(" · "))
            };
            panel = panel.child(
                v_flex()
                    .gap_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(format!(
                                "{MODEL_ACTIVE_PREFIX}: {} from {}{hint_copy}",
                                active.model_name, active.provider_label
                            )),
                    )
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(20.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(MODEL_SWITCHED_CONFIRMATION),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{MODEL_DRAWS_ON_PREFIX} {} account.",
                                active.provider_label
                            )),
                    ),
            );
        }
        // The catalog list: every model with its provider and honest
        // availability. Runnable models are selectable; unavailable ones
        // are listed with their honest reason and stay unselectable
        // (layer 7 — never implying a connection exists).
        let task_id_for_rows = task_id.clone();
        panel = panel
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground)
                    .child(MODEL_LIST_CAPTION),
            )
            .child(
                v_flex()
                    .gap_1()
                    .max_h(px(288.0))
                    .overflow_y_scrollbar()
                    .children(catalog.iter().map(|entry| {
                        let is_active = active
                            .as_ref()
                            .is_some_and(|active| active.model_id == entry.model_id);
                        let hints = entry.capability_hints();
                        let hint_copy = if hints.is_empty() {
                            String::new()
                        } else {
                            format!(" · {}", hints.join(" · "))
                        };
                        let task_id = task_id_for_rows.clone();
                        let model_id = entry.model_id.clone();
                        let row = v_flex().gap_0p5().child(div().text_sm().child(format!(
                            "{} from {}{hint_copy}",
                            entry.model_name, entry.provider_label
                        )));
                        let row = match entry.availability {
                            PickerModelAvailability::Connected => row.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{MODEL_DRAWS_ON_PREFIX} {} account. {}",
                                        entry.provider_label,
                                        entry.availability.user_label()
                                    )),
                            ),
                            PickerModelAvailability::ConfiguredNotConnected => row.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(MODEL_UNAVAILABLE_CONFIGURED),
                            ),
                            PickerModelAvailability::NotConfigured => row.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(MODEL_UNAVAILABLE_NOT_CONFIGURED),
                            ),
                        };
                        h_flex()
                            .flex_none()
                            .child(
                                Button::new(SharedString::from(format!(
                                    "flauz-model-row-{}",
                                    entry.model_id
                                )))
                                .label(if is_active {
                                    SharedString::from(format!(
                                        "{} — running this task",
                                        entry.model_name
                                    ))
                                } else {
                                    SharedString::from(entry.model_name.clone())
                                })
                                .small()
                                .ghost()
                                .selected(is_active)
                                .disabled(!entry.availability.is_runnable())
                                .tooltip(if entry.availability.is_runnable() {
                                    format!(
                                        "Run this task with {} ({})",
                                        entry.model_name,
                                        entry.availability.user_label()
                                    )
                                } else {
                                    match entry.availability {
                                        PickerModelAvailability::ConfiguredNotConnected => {
                                            MODEL_UNAVAILABLE_CONFIGURED.to_owned()
                                        }
                                        _ => MODEL_UNAVAILABLE_NOT_CONFIGURED.to_owned(),
                                    }
                                })
                                .on_click({
                                    let task_id = task_id.clone();
                                    let model_id = model_id.clone();
                                    cx.listener(move |this, _, _window, cx| {
                                        select_model_for_task(this, &task_id, &model_id, cx);
                                    })
                                }),
                            )
                            .child(row)
                            .into_any_element()
                    })),
            );
    }

    panel
        .child(
            h_flex()
                .flex_none()
                .pt_2()
                .border_t_1()
                .border_color(cx.theme().border)
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "Applies to “{task_title}” — switching never starts a new task"
                        )),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(MODEL_PICKER_ESCAPE_HINT),
                ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The house unwrap helpers (the runtime_codex test idiom):
    /// `expect()`/`unwrap()` are workspace warnings and CI denies warnings.
    fn some<T>(value: Option<T>) -> T {
        match value {
            Some(value) => value,
            None => panic!("expected Some(_), got None"),
        }
    }

    fn connected_entry(model_id: &str, name: &str) -> PickerModelEntry {
        PickerModelEntry::new(
            model_id,
            name,
            "openai",
            "OpenAI",
            vec!["text".to_owned(), "vision".to_owned()],
            PickerModelAvailability::Connected,
        )
    }

    fn configured_not_connected_entry(model_id: &str, name: &str) -> PickerModelEntry {
        PickerModelEntry::new(
            model_id,
            name,
            "ollama",
            "Ollama",
            Vec::new(),
            PickerModelAvailability::ConfiguredNotConnected,
        )
    }

    fn not_configured_entry(model_id: &str, name: &str) -> PickerModelEntry {
        PickerModelEntry::new(
            model_id,
            name,
            "anthropic",
            "Anthropic",
            Vec::new(),
            PickerModelAvailability::NotConfigured,
        )
    }

    fn all_user_copy() -> Vec<&'static str> {
        vec![
            MODEL_CONTROL_LABEL,
            MODEL_CONTROL_EMPTY_LABEL,
            MODEL_CONTROL_TOOLTIP_ACTIVE,
            MODEL_CONTROL_TOOLTIP_EMPTY,
            MODEL_PICKER_NO_CHAT_GUIDANCE,
            PALETTE_ROW_TITLE,
            PALETTE_ROW_DESCRIPTION,
            MODEL_PICKER_HEADING,
            MODEL_PICKER_DESCRIPTION,
            MODEL_EMPTY_TITLE,
            MODEL_EMPTY_BODY,
            MODEL_EMPTY_NEXT_STEP,
            MODEL_EMPTY_SETTINGS_BUTTON,
            MODEL_ACTIVE_PREFIX,
            MODEL_SWITCHED_CONFIRMATION,
            MODEL_DRAWS_ON_PREFIX,
            MODEL_UNAVAILABLE_CONFIGURED,
            MODEL_UNAVAILABLE_NOT_CONFIGURED,
            MODEL_PICKER_ESCAPE_HINT,
            MODEL_LIST_CAPTION,
            "What to do next",
        ]
    }

    #[test]
    fn picker_copy_avoids_internal_type_names() {
        // MOD-001 user-language rule: internal type names never appear in
        // UI copy (the J-10/J-11 discipline; the shell family test).
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "providerregistration",
                "registrysnapshot",
                "providersnapshot",
                "modelregistry",
                "agentruntime",
                "runtimerequest",
                "runtimeoutcome",
                "capabilityid",
                "capabilitygap",
                "codexappserverruntime",
                "directruntime",
                "codexserverturn",
                "modeleventlog",
                "flauz-exec",
                "flauz-world",
                "flauz-context",
                "flauz_model_picker",
                "mod-001",
                "rt-001",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "UI copy must not leak the internal term {forbidden:?}: {copy:?}"
                );
            }
        }
    }

    #[test]
    fn picker_copy_is_complete_and_honest() {
        // Discoverability contract §1.4: every layer's copy exists, the
        // empty state explains the capability, and the next step says
        // what to do — no empty strings, no dead ends.
        for copy in all_user_copy() {
            assert!(!copy.trim().is_empty(), "copy must not be empty");
        }
        assert!(MODEL_EMPTY_BODY.len() > 40, "body explains the feature");
        assert!(
            MODEL_EMPTY_NEXT_STEP.len() > 40,
            "the next step says what to do"
        );
        assert!(
            MODEL_EMPTY_NEXT_STEP.to_lowercase().contains("settings"),
            "the empty state points at settings"
        );
        // The empty state must NOT claim a connection flow exists (F7).
        assert!(!MODEL_EMPTY_NEXT_STEP.to_lowercase().contains("connect now"));
        // The identity promise is user language about the task, not
        // about internals.
        assert!(
            MODEL_SWITCHED_CONFIRMATION
                .to_lowercase()
                .contains("this task keeps"),
            "the switch confirmation preserves task identity visibly"
        );
        assert!(
            MODEL_PICKER_DESCRIPTION
                .to_lowercase()
                .contains("keeps this task"),
            "the panel description carries the identity promise"
        );
    }

    #[test]
    fn palette_query_resolves_to_the_model_row() {
        // Discoverability contract §1.3: a natural query must hit the
        // palette row's title or description.
        for query in ["model", "choose", "switch"] {
            let lower_title = PALETTE_ROW_TITLE.to_lowercase();
            let lower_description = PALETTE_ROW_DESCRIPTION.to_lowercase();
            assert!(
                lower_title.contains(query) || lower_description.contains(query),
                "query {query:?} must resolve to the model picker palette row"
            );
        }
    }

    #[test]
    fn capability_hints_use_user_language() {
        // Known keys of the frozen vocabulary map to plain language;
        // unknown keys pass through unchanged (honest, never invented).
        assert_eq!(capability_hint("text"), "Handles text");
        assert_eq!(capability_hint("vision"), "Understands images");
        assert_eq!(capability_hint("image.input"), "Accepts images");
        assert_eq!(capability_hint("terminal"), "Can use a terminal");
        assert_eq!(capability_hint("browser.input"), "Can drive a browser");
        assert_eq!(
            capability_hint("browser.navigation"),
            "Can navigate websites"
        );
        assert_eq!(capability_hint("reasoning.tools"), "reasoning.tools");
    }

    #[test]
    fn model_switch_preserves_task_identity_through_the_world_store_seam() {
        // MOD-001 acceptance: a model switch records a
        // `task.model_changed` event via the world store seam and the
        // task identity is preserved (kernel §6: model choice is
        // execution state, never identity). The real switch decision
        // (`switch_model_through_seam`, the logic
        // `FlauzModelPickerState::select_model` delegates to) is
        // exercised directly — the seam types are plain data, exactly
        // what a later slice wires to the real world store.
        let task_id = "task_01J8ZQ5V8K3T2B7N6X4R9DQP34";
        let model_a = "model_01J8ZQ5V8K3T2B7N6X4R9DQPX0";
        let model_b = "model_01J8ZQ5V8K3T2B7N6X4R9DQPY1";
        let catalog = vec![
            connected_entry(model_a, "Fast model"),
            connected_entry(model_b, "Strong model"),
        ];
        let mut active_models: HashMap<String, String> = HashMap::new();
        let mut log = TaskModelEventLog::new();
        assert!(log.events().is_empty());

        let first = some(switch_model_through_seam(
            &catalog,
            &mut active_models,
            &mut log,
            task_id,
            model_a,
        ));
        assert_eq!(first.event_type, MODEL_CHANGED_EVENT_TYPE);
        assert_eq!(first.task_id, task_id);
        assert_eq!(first.from_model, None);
        assert_eq!(first.to_model, model_a);
        assert_eq!(first.seq, 1);

        let second = some(switch_model_through_seam(
            &catalog,
            &mut active_models,
            &mut log,
            task_id,
            model_b,
        ));
        assert_eq!(second.event_type, MODEL_CHANGED_EVENT_TYPE);
        assert_eq!(second.task_id, task_id, "the task ID never changes");
        assert_eq!(second.from_model.as_deref(), Some(model_a));
        assert_eq!(second.to_model, model_b);
        assert_eq!(second.seq, 2, "sequence numbers strictly increase");

        // Identity preserved: every event on the stream names the same
        // task; switching never forked it. The active model moved (A →
        // B) while the task key never did.
        for event in log.events() {
            assert_eq!(event.task_id, task_id);
            assert_eq!(event.event_type, MODEL_CHANGED_EVENT_TYPE);
        }
        assert_eq!(log.events().len(), 2);
        assert!(
            log.events()
                .windows(2)
                .all(|pair| pair[0].seq < pair[1].seq)
        );
        assert_eq!(
            active_models.get(task_id).map(String::as_str),
            Some(model_b),
            "the active model is execution state keyed by the unchanged task"
        );
    }

    #[test]
    fn switches_reselects_and_unavailable_models_are_rejected() {
        // The full switch decision table on the REAL logic: re-selecting
        // the same model records nothing; a model from a
        // configured-not-connected or not-configured provider is never
        // selected (the honest unavailable state); an unknown model id
        // is rejected the same way.
        let task_id = "task_01J8ZQ5V8K3T2B7N6X4R9DQP34";
        let model_a = "model_01J8ZQ5V8K3T2B7N6X4R9DQPX0";
        let model_b = "model_01J8ZQ5V8K3T2B7N6X4R9DQPY1";
        let model_local = "model_01J8ZQ5V8K3T2B7N6X4R9DQPZ2";
        let catalog = vec![
            connected_entry(model_a, "Fast model"),
            connected_entry(model_b, "Strong model"),
            configured_not_connected_entry(model_local, "Local model"),
            not_configured_entry("model_01J8ZQ5V8K3T2B7N6X4R9DQPW3", "Cloud model"),
        ];
        let mut active_models: HashMap<String, String> = HashMap::new();
        let mut log = TaskModelEventLog::new();

        // Switch to A, then to B: two events, same task, active model B.
        assert!(
            switch_model_through_seam(&catalog, &mut active_models, &mut log, task_id, model_a)
                .is_some()
        );
        assert!(
            switch_model_through_seam(&catalog, &mut active_models, &mut log, task_id, model_b)
                .is_some()
        );
        assert_eq!(
            active_models.get(task_id).map(String::as_str),
            Some(model_b)
        );
        assert_eq!(log.events().len(), 2);
        for event in log.events() {
            assert_eq!(event.task_id, task_id, "identity preserved");
        }

        // Re-selecting B records nothing (nothing changed).
        assert!(
            switch_model_through_seam(&catalog, &mut active_models, &mut log, task_id, model_b)
                .is_none()
        );
        assert_eq!(log.events().len(), 2);

        // A model from a configured-not-connected provider is never
        // selected and records nothing.
        assert!(
            switch_model_through_seam(&catalog, &mut active_models, &mut log, task_id, model_local)
                .is_none()
        );
        assert_eq!(
            active_models.get(task_id).map(String::as_str),
            Some(model_b),
            "the unavailable switch changed nothing"
        );
        assert_eq!(log.events().len(), 2);

        // An unknown model id is rejected the same way.
        assert!(
            switch_model_through_seam(
                &catalog,
                &mut active_models,
                &mut log,
                task_id,
                "model_missing"
            )
            .is_none()
        );
        assert_eq!(log.events().len(), 2);
    }

    #[test]
    fn unavailable_models_are_never_runnable() {
        // Layer 7: the two non-connected states are honestly labeled and
        // never runnable — the UI can never imply a connection exists.
        assert!(PickerModelAvailability::Connected.is_runnable());
        assert!(!PickerModelAvailability::ConfiguredNotConnected.is_runnable());
        assert!(!PickerModelAvailability::NotConfigured.is_runnable());
        assert_eq!(PickerModelAvailability::Connected.user_label(), "Connected");
        assert_eq!(
            PickerModelAvailability::ConfiguredNotConnected.user_label(),
            "Not connected yet"
        );
        assert_eq!(
            PickerModelAvailability::NotConfigured.user_label(),
            "Needs setup"
        );
        assert!(
            configured_not_connected_entry("m", "Local model")
                .capability_hints()
                .is_empty()
        );
        assert_eq!(
            connected_entry("m", "Fast model").capability_hints(),
            vec!["Handles text".to_owned(), "Understands images".to_owned()]
        );
        let _ = not_configured_entry("m", "Strong model");
    }

    #[test]
    fn picker_seams_are_registered_in_the_ui_seams() {
        // The MOD-001 registration seam test (the house ui.rs
        // source-inspection style): every layer is wired through the
        // named seams in `ui.rs`.
        let source = include_str!("../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_model_picker;"));

        // The palette registration seam: the row, its group, and its
        // dispatch.
        assert!(source.contains("PaletteCommand::ChooseModelForTask"));
        assert!(source.contains("flauz_model_picker::PALETTE_ROW_TITLE"));
        assert!(source.contains("flauz_model_picker::PALETTE_ROW_DESCRIPTION"));
        assert!(source.contains("(PaletteGroup::WorkspaceShell, \"Workspace\")"));
        assert!(source.contains("flauz_model_picker::open_model_picker("));

        // The navigation registration seams: the task-surface control,
        // the panel mount, and the dispatch navigation close.
        assert!(source.contains("flauz_model_picker::render_model_control(self, cx)"));
        assert!(source.contains("flauz_model_picker::render_model_picker("));
        assert!(source.contains("flauz_model_picker::action_closes_model_picker(&action)"));

        // The state field seam.
        assert!(source.contains("flauz_model_picker::FlauzModelPickerState"));

        // The keyboard registration seams: the chord (scanned against
        // the whitespace-normalized source — rustfmt legitimately wraps
        // long `KeyBinding::new(...)` expressions across lines) and the
        // scoped escape binding for the picker's focus context.
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzModelPickerShortcut, None"),
            "a keyboard chord must be bound for the model picker"
        );
        assert!(source.contains("Some(\"FlauzModelPicker\")"));
    }
}
