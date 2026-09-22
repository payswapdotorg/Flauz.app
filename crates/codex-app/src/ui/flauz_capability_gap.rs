//! The capability-gap surface (work order CAP-001, F2 Wave 2): the J-04
//! disclosure chain — no silent fall-through in the UI.
//!
//! When a capability is unavailable, the user sees **Capability
//! unavailable → Why? → the named missing dimensions (the model lacks it
//! / the runtime doesn't advertise it / no environment offers it /
//! permission hasn't been granted / workspace policy doesn't allow it) →
//! ways to unlock** — and when a capability IS available, the success
//! state names what is admitted, from which dimensions. Nothing quietly
//! disappears.
//!
//! Boundary rules (the UX-001 shell-family discipline):
//!
//! - the surface lives in the task rail's **More/Inspect neighborhood**
//!   (`render_task_workspace`, directly under the task control rail) —
//!   reachable by its visible labeled affordance, its palette row ("Why
//!   is a capability unavailable?"), and its direct keyboard chord
//!   (Ctrl+Alt+Shift+6, the task-rail chord family), never hidden behind
//!   developer settings;
//! - the module wires through minimal `ui.rs` seams only (module
//!   declaration, palette registration, keyboard chord, dispatch, the
//!   task-surface render registration, and the navigation close hook);
//! - the app crate does not import the `flauz-cap` contract crate in
//!   this wave: the panel renders a plain **view-model**
//!   ([`CapabilityGapDetail`]) that a later wave populates from a
//!   resolution record. Until wiring lands, the panel shows the honest
//!   empty state below — the chain is implemented, but no data is
//!   invented;
//! - the open/close paths follow the F1 focus contracts: the panel
//!   captures the previously focused surface on open, auto-focuses its
//!   own handle once (the 019 request-once shape), and restores on close
//!   (the 017 contract).
//!
//! Every user-facing string lives in the copy registry below so the
//! language rules are unit-testable in one place.

use codex_core::MainRoute;
use gpui::prelude::*;
use gpui::{AnyElement, App, Context, FocusHandle, Window, div, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::Escape,
    v_flex,
};

use super::WorkspaceView;

gpui::actions!(codexrs, [FlauzCapabilityGapShortcut]);

/// The entry affordance label on the task surface.
const ENTRY_LABEL: &str = "Capabilities";
/// The entry affordance tooltip (the chord matches the rail family).
const ENTRY_TOOLTIP: &str =
    "See what this task can do, and why a capability may be unavailable (Ctrl+Alt+Shift+6)";
/// The panel heading.
const PANEL_HEADING: &str = "Capabilities";
/// The one-line description under the panel heading.
const PANEL_DESCRIPTION: &str = "What a task can do — and what's missing when it can't";
/// The honest guidance shown when the chord fires with no chat selected
/// (the WO-P2-012 pattern: guidance instead of a silent no-op).
const CAPABILITY_GAP_NO_CHAT_GUIDANCE: &str =
    "Open a chat to see its capabilities and why one may be unavailable.";
/// The headline of the unavailable state (J-04: the user asks "why?").
const GAP_HEADLINE: &str = "Capability unavailable";
/// The disclosure toggle that opens the named dimensions.
const WHY_LABEL: &str = "Why?";
/// The disclosure toggle that closes them again.
const HIDE_DETAILS_LABEL: &str = "Hide the details";
/// The heading over the named missing dimensions.
const WHATS_MISSING_HEADING: &str = "What's missing";
/// The heading over the unlock paths.
const WAYS_TO_UNLOCK_HEADING: &str = "Ways to unlock";
/// The headline of the available state (J-04: the success state names
/// what is admitted).
const AVAILABLE_HEADLINE: &str = "Capability available";
/// The body of the available state.
const AVAILABLE_BODY: &str = "Everything this capability needs is in place.";
/// The row suffix naming an admitting dimension.
const ADMITTED_SUFFIX: &str = "available";
/// The honest empty state while capability resolution is not yet wired
/// to the task surface.
const EMPTY_TITLE: &str = "Capability resolution is on its way";
/// What the panel will do (honest: what is implemented, what is not).
const EMPTY_BODY: &str = "When a capability is unavailable, this panel will say why — naming \
     exactly what is missing: the model, the runtime, the environment, \
     permissions, or workspace policy. No capability quietly disappears.";
/// What to do next, today.
const EMPTY_NEXT_STEP: &str = "Run a task and ask for something it can't do — the named gaps \
     and the steps that would unlock them appear here once capability \
     resolution is wired into the task surface.";
/// The close affordance.
const BACK_TO_YOUR_WORK: &str = "Back to your work";
/// The close tooltip.
const BACK_TO_YOUR_WORK_TOOLTIP: &str = "Return to your chat (Escape)";
/// The keyboard-hint footer.
const ESCAPE_HINT: &str = "Escape closes this panel";
/// The command-palette row title (CAP-001 registration seam copy: the
/// work-order's exact palette row).
pub(crate) const PALETTE_ROW_TITLE: &str = "Why is a capability unavailable?";
/// The command-palette row description.
pub(crate) const PALETTE_ROW_DESCRIPTION: &str =
    "See what's missing when a task capability isn't available, and how to unlock it";

/// One admission dimension of the capability view-model — the J-04 user
/// vocabulary. The serialized dimension names of the contract crate never
/// appear in UI copy; these labels do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // matched by the render today; constructed by the module's tests and (Wave-later) the resolution-record population
pub(crate) enum CapabilityDimension {
    /// The model in use offers the capability.
    Model,
    /// The runtime in use advertises the capability.
    Runtime,
    /// An attached environment offers the capability.
    Environment,
    /// Permission to use the capability has been granted.
    Permissions,
    /// Workspace policy allows the capability.
    WorkspacePolicy,
}

impl CapabilityDimension {
    /// Every admission dimension, in canonical order.
    #[allow(dead_code)] // iterated by the module's tests; the Wave-later resolution population iterates it in production
    pub(crate) const ALL: [Self; 5] = [
        Self::Model,
        Self::Runtime,
        Self::Environment,
        Self::Permissions,
        Self::WorkspacePolicy,
    ];

    /// The user-facing dimension label.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Model => "Model",
            Self::Runtime => "Runtime",
            Self::Environment => "Environment",
            Self::Permissions => "Permissions",
            Self::WorkspacePolicy => "Workspace policy",
        }
    }

    /// The user-facing gap headline for a missing dimension (J-04: "model
    /// lacks it / runtime doesn't advertise it / environment has no such
    /// surface / permission denied / workspace policy").
    pub(crate) const fn gap_headline(self) -> &'static str {
        match self {
            Self::Model => "The model in use doesn't offer it",
            Self::Runtime => "The runtime doesn't advertise it",
            Self::Environment => "No attached environment offers it",
            Self::Permissions => "Permission hasn't been granted",
            Self::WorkspacePolicy => "Workspace policy doesn't allow it",
        }
    }

    /// The user-facing unlock hint for a missing dimension — honest
    /// action copy: what would close the gap, never a promise that a
    /// surface to do it ships today.
    pub(crate) const fn unlock_hint(self) -> &'static str {
        match self {
            Self::Model => "Choose a model that offers it",
            Self::Runtime => "Use a runtime that advertises it",
            Self::Environment => "Attach an environment that offers it",
            Self::Permissions => "Grant permission for this task",
            Self::WorkspacePolicy => "Ask the workspace owner to allow it",
        }
    }
}

/// One dimension's status in the view-model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DimensionStatus {
    /// The admission dimension.
    dimension: CapabilityDimension,
    /// Whether the dimension admits the capability.
    admitted: bool,
}

impl DimensionStatus {
    /// Builds one dimension status.
    #[allow(dead_code)] // used by the module's tests; the Wave-later resolution population builds statuses through it
    pub(crate) const fn new(dimension: CapabilityDimension, admitted: bool) -> Self {
        Self {
            dimension,
            admitted,
        }
    }
}

/// The capability-gap view-model: what the panel renders once a
/// resolution is attached. Plain owned data — a later wave populates it
/// from a resolution record; this module never invents capability data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapabilityGapDetail {
    /// The user-facing capability name ("Browser input", "Terminal", …).
    capability: String,
    /// The five dimension statuses, in canonical order.
    dimensions: Vec<DimensionStatus>,
}

impl CapabilityGapDetail {
    /// Builds a detail view-model. A dimension status must be supplied
    /// for every dimension exactly once, in canonical order — the same
    /// record-consistency law the contract crate enforces.
    #[allow(dead_code)] // used by the module's tests; the Wave-later resolution population builds details through it
    pub(crate) fn new(capability: String, dimensions: Vec<DimensionStatus>) -> Self {
        debug_assert_eq!(
            dimensions.len(),
            CapabilityDimension::ALL.len(),
            "a detail carries one status per dimension"
        );
        Self {
            capability,
            dimensions,
        }
    }

    /// The user-facing capability name.
    pub(crate) fn capability(&self) -> &str {
        &self.capability
    }

    /// Whether the capability is available (every dimension admits it).
    #[allow(dead_code)] // used by the module's tests; the Wave-later resolution population reads it
    pub(crate) fn available(&self) -> bool {
        self.dimensions.iter().all(|status| status.admitted)
    }

    /// The missing dimensions, in canonical order.
    pub(crate) fn missing(&self) -> Vec<&DimensionStatus> {
        self.dimensions
            .iter()
            .filter(|status| !status.admitted)
            .collect()
    }

    /// The admitting dimensions, in canonical order.
    pub(crate) fn admitted(&self) -> Vec<&DimensionStatus> {
        self.dimensions
            .iter()
            .filter(|status| status.admitted)
            .collect()
    }
}

/// The additive surface state: open flag, the Why-disclosure toggle, the
/// attached resolution view-model, and the panel's focus bookkeeping.
pub(crate) struct CapabilityGapState {
    open: bool,
    /// Whether the Why-disclosure is expanded (the chain starts
    /// collapsed: headline → Why? → the named dimensions).
    why_expanded: bool,
    /// The attached resolution view-model, when one has been wired in.
    detail: Option<CapabilityGapDetail>,
    /// The panel's keyboard focus handle (the 019 request-once shape).
    panel_focus: FocusHandle,
    /// Request-once guard for `panel_focus`.
    panel_focus_requested: bool,
    /// The surface that held focus when the panel opened, restored on
    /// close (the 017 close contract).
    focus_before_panel: Option<FocusHandle>,
}

impl CapabilityGapState {
    /// Builds the closed, empty surface state.
    pub(crate) fn new(cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            open: false,
            why_expanded: false,
            detail: None,
            panel_focus: cx.focus_handle(),
            panel_focus_requested: false,
            focus_before_panel: None,
        }
    }

    /// Whether the panel is open.
    #[allow(dead_code)] // the Wave-later wiring reads it; the render paths check `open` directly today
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Quietly closes the panel for an F1 navigation action (no focus
    /// restore — the deliberate close paths restore through the 017
    /// contract instead). Returns whether anything changed.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.open;
        self.open = false;
        self.why_expanded = false;
        self.panel_focus_requested = false;
        self.focus_before_panel = None;
        changed
    }
}

/// Opens (or toggles closed) the capability-gap panel on the selected
/// chat. Called from the entry affordance, the palette row, and the
/// direct keyboard chord. With no chat selected, surfaces honest
/// guidance instead of a silent no-op (the WO-P2-012 pattern).
pub(crate) fn open_capability_gap_surface(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.state.selected_task_id.is_none() {
        workspace.dispatch_command_status(Some(CAPABILITY_GAP_NO_CHAT_GUIDANCE), cx);
        return;
    }
    if workspace.flauz_capability_gap.open {
        close_capability_gap_surface(workspace, window, cx);
        return;
    }
    // The panel lives on the task workspace: navigate there first when
    // needed, so it is reachable from any route.
    if workspace.state.route != MainRoute::Tasks {
        workspace.navigate(MainRoute::Tasks, cx);
    }
    // One inspection panel at a time in the More/Inspect neighborhood:
    // opening the capability panel closes any open rail panel.
    super::flauz_shell::dismiss_shell_surfaces(workspace, window, cx);
    let state = &mut workspace.flauz_capability_gap;
    if state.focus_before_panel.is_none() {
        state.focus_before_panel = window.focused(cx);
    }
    state.open = true;
    state.why_expanded = false;
    state.panel_focus_requested = true;
    cx.notify();
}

/// Closes the panel and restores focus (the 017 close contract).
pub(crate) fn close_capability_gap_surface(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if !workspace.flauz_capability_gap.open {
        return;
    }
    workspace.flauz_capability_gap.open = false;
    workspace.flauz_capability_gap.why_expanded = false;
    workspace.flauz_capability_gap.panel_focus_requested = false;
    let previous = workspace.flauz_capability_gap.focus_before_panel.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// Toggles the Why-disclosure (the J-04 chain: headline → Why? → the
/// named dimensions → the unlock paths).
pub(crate) fn toggle_why_disclosure(
    workspace: &mut WorkspaceView,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    workspace.flauz_capability_gap.why_expanded = !workspace.flauz_capability_gap.why_expanded;
    cx.notify();
}

/// Renders the entry affordance (the visible, labeled capability control
/// on the task surface) plus the open panel: the full disclosure chain
/// when a resolution view-model is attached, the honest empty state until
/// wiring lands. Rendered in the task rail's More/Inspect neighborhood.
pub(crate) fn render_capability_gap_entry(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open = workspace.flauz_capability_gap.open;
    let mut entry = v_flex().flex_none();
    if workspace.flauz_capability_gap.panel_focus_requested {
        // The 019 request-once shape: the panel claims the keyboard on
        // mount so the scoped Escape binding reaches it.
        workspace.flauz_capability_gap.panel_focus_requested = false;
        workspace.flauz_capability_gap.panel_focus.focus(window);
    }
    entry = entry.child(
        h_flex()
            .h(px(32.0))
            .px_5()
            .items_center()
            .child(
                Button::new("flauz-capability-gap-entry")
                    .label(ENTRY_LABEL)
                    .icon(IconName::Asterisk)
                    .tooltip(ENTRY_TOOLTIP)
                    .small()
                    .ghost()
                    .selected(open)
                    .on_click(cx.listener(|this, _, window, cx| {
                        open_capability_gap_surface(this, window, cx);
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
        entry = entry.child(render_gap_panel(&workspace.flauz_capability_gap, cx));
    }
    entry.into_any_element()
}

/// Renders the gap panel: the Why/What-missing/Ways-to-unlock disclosure
/// chain over the attached view-model, or the honest empty state.
fn render_gap_panel(state: &CapabilityGapState, cx: &mut Context<WorkspaceView>) -> AnyElement {
    let panel_focus = state.panel_focus.clone();
    let why_expanded = state.why_expanded;
    let detail = state.detail.clone();
    let has_detail = detail.is_some();
    let mut panel = v_flex()
        .key_context("FlauzCapabilityGap")
        .track_focus(&panel_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            close_capability_gap_surface(this, window, cx);
        }))
        .mx_5()
        .my_2()
        .p_4()
        .gap_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().sidebar)
        .child(render_panel_header(cx))
        .when_some(detail, |panel, detail| {
            panel.child(render_disclosure_chain(&detail, why_expanded, cx))
        })
        .when(!has_detail, |panel| panel.child(render_empty_state(cx)));
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
            Button::new("flauz-capability-gap-close")
                .label(BACK_TO_YOUR_WORK)
                .icon(IconName::ArrowLeft)
                .tooltip(BACK_TO_YOUR_WORK_TOOLTIP)
                .small()
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    close_capability_gap_surface(this, window, cx);
                })),
        )
        .into_any_element()
}

/// Renders the J-04 disclosure chain over one detail view-model:
///
/// ```text
/// Capability unavailable            (the headline)
/// Why?  /  Hide the details         (the disclosure toggle)
/// What's missing                    (each named missing dimension)
/// Ways to unlock                    (per missing dimension, the honest hint)
/// ```
///
/// and the success state when every dimension admits:
///
/// ```text
/// Capability available
/// Everything this capability needs is in place.
/// Model — available · Runtime — available · …
/// ```
fn render_disclosure_chain(
    detail: &CapabilityGapDetail,
    why_expanded: bool,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let mut chain = v_flex().gap_2();
    let missing = detail.missing();
    let available = missing.is_empty();
    chain = chain
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(if available {
                    IconName::CircleCheck
                } else {
                    IconName::CircleX
                }))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(if available {
                            format!("{AVAILABLE_HEADLINE} — {}", detail.capability())
                        } else {
                            format!("{GAP_HEADLINE} — {}", detail.capability())
                        }),
                ),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(if available {
                    AVAILABLE_BODY.to_owned()
                } else {
                    "Ask for it and it can't run — here is exactly why.".to_owned()
                }),
        );
    if available {
        // The success state names what is admitted, from which dimensions.
        let admitted = detail.admitted();
        debug_assert!(
            !admitted.is_empty(),
            "an available detail admits every dimension"
        );
        let labels = admitted
            .iter()
            .map(|status| format!("{} — {}", status.dimension.label(), ADMITTED_SUFFIX))
            .collect::<Vec<_>>()
            .join(" · ");
        chain = chain.child(div().text_sm().line_height(px(20.0)).child(labels));
        return chain.into_any_element();
    }
    // The disclosure toggle: the chain starts at the headline; "Why?"
    // opens the named dimensions.
    chain = chain.child(
        Button::new("flauz-capability-gap-why")
            .label(if why_expanded {
                HIDE_DETAILS_LABEL
            } else {
                WHY_LABEL
            })
            .icon(if why_expanded {
                IconName::ChevronUp
            } else {
                IconName::ChevronDown
            })
            .small()
            .ghost()
            .on_click(cx.listener(|this, _, window, cx| {
                toggle_why_disclosure(this, window, cx);
            })),
    );
    if why_expanded {
        chain = chain
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
                            .child(WHATS_MISSING_HEADING),
                    )
                    .children(missing.iter().map(|status| {
                        h_flex()
                            .gap_2()
                            .items_start()
                            .child(Icon::new(IconName::CircleX).small())
                            .child(div().text_sm().line_height(px(20.0)).child(format!(
                                "{} — {}",
                                status.dimension.label(),
                                status.dimension.gap_headline()
                            )))
                    })),
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
                            .child(WAYS_TO_UNLOCK_HEADING),
                    )
                    .children(missing.iter().map(|status| {
                        h_flex()
                            .gap_2()
                            .items_start()
                            .child(Icon::new(IconName::CircleCheck).small())
                            .child(
                                div()
                                    .text_sm()
                                    .line_height(px(20.0))
                                    .child(status.dimension.unlock_hint()),
                            )
                    })),
            );
    } else {
        // The collapsed chain still promises the answer: the toggle's
        // label is the question the expansion answers, and the count
        // says how many pieces are missing.
        let pieces = if missing.len() == 1 {
            "piece is"
        } else {
            "pieces are"
        };
        chain = chain.child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(format!(
                    "{WHY_LABEL} — {} {} missing.",
                    missing.len(),
                    pieces
                )),
        );
    }
    chain.into_any_element()
}

/// Renders the honest empty state: capability resolution is implemented
/// (this panel) but not yet wired to live task data — so the copy says
/// what will appear and what to do today. Never a dead end, never a fake
/// capability.
fn render_empty_state(cx: &App) -> AnyElement {
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
                .child(Icon::new(IconName::Asterisk).small())
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(EMPTY_TITLE),
                ),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(EMPTY_BODY),
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
                .child(div().text_sm().line_height(px(20.0)).child(EMPTY_NEXT_STEP)),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_user_copy() -> Vec<&'static str> {
        let mut copy = Vec::new();
        for dimension in CapabilityDimension::ALL {
            copy.push(dimension.label());
            copy.push(dimension.gap_headline());
            copy.push(dimension.unlock_hint());
        }
        copy.extend([
            ENTRY_LABEL,
            ENTRY_TOOLTIP,
            PANEL_HEADING,
            PANEL_DESCRIPTION,
            CAPABILITY_GAP_NO_CHAT_GUIDANCE,
            GAP_HEADLINE,
            WHY_LABEL,
            HIDE_DETAILS_LABEL,
            WHATS_MISSING_HEADING,
            WAYS_TO_UNLOCK_HEADING,
            AVAILABLE_HEADLINE,
            AVAILABLE_BODY,
            ADMITTED_SUFFIX,
            EMPTY_TITLE,
            EMPTY_BODY,
            EMPTY_NEXT_STEP,
            BACK_TO_YOUR_WORK,
            BACK_TO_YOUR_WORK_TOOLTIP,
            ESCAPE_HINT,
            PALETTE_ROW_TITLE,
            PALETTE_ROW_DESCRIPTION,
            "What to do next",
        ]);
        copy
    }

    #[test]
    fn gap_copy_uses_user_language_not_internal_type_names() {
        // CAP-001: the user language is the J-04 vocabulary; internal
        // type names (the contract crate, the record types, the work
        // order id) never appear in UI copy.
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "capabilityresolution",
                "capabilitykey",
                "dimensionadmission",
                "namedgap",
                "unlockpath",
                "resolutioninputs",
                "flauz-cap",
                "flauz_world",
                "flauz-exec",
                "cap-001",
                "view-model",
                "viewmodel",
                "intersection",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "UI copy must not leak the internal term {forbidden:?}: {copy:?}"
                );
            }
        }
    }

    #[test]
    fn every_state_of_the_chain_has_complete_copy() {
        // Discoverability contract §1.4/§1.7: every state explains
        // itself — the headline, the disclosure labels, the section
        // headings, the empty state, the success state. No empty
        // strings, no dead ends.
        assert!(!ENTRY_LABEL.is_empty());
        assert!(ENTRY_TOOLTIP.contains("Ctrl+Alt+Shift+6"));
        assert!(EMPTY_BODY.len() > 40, "the empty body explains the feature");
        assert!(
            EMPTY_NEXT_STEP.len() > 40,
            "the empty state says what to do"
        );
        // The honest empty state promises the no-silent-fall-through law
        // in user language.
        assert!(EMPTY_BODY.contains("No capability quietly disappears"));
        for dimension in CapabilityDimension::ALL {
            assert!(!dimension.label().is_empty());
            assert!(dimension.gap_headline().len() > 10);
            assert!(dimension.unlock_hint().len() > 10);
        }
        // The five named-gap headlines are the J-04 vocabulary: model,
        // runtime, environment, permission, workspace policy.
        let joined = CapabilityDimension::ALL
            .iter()
            .map(|dimension| dimension.gap_headline())
            .collect::<Vec<_>>()
            .join(" | ");
        for term in ["model", "runtime", "environment", "permission", "policy"] {
            assert!(
                joined.to_lowercase().contains(term),
                "the J-04 vocabulary names {term:?}"
            );
        }
    }

    #[test]
    fn the_disclosure_chain_model_is_complete_for_both_states() {
        // The J-04 chain over a missing capability: headline (unavailable)
        // → the missing dimensions (all five, each named) → one unlock
        // hint per missing dimension.
        let missing_all = CapabilityGapDetail::new(
            "Browser input".to_owned(),
            CapabilityDimension::ALL
                .iter()
                .map(|dimension| DimensionStatus::new(*dimension, false))
                .collect(),
        );
        assert!(!missing_all.available());
        assert_eq!(missing_all.missing().len(), 5);
        assert!(missing_all.admitted().is_empty());
        for (status, dimension) in missing_all
            .missing()
            .iter()
            .zip(CapabilityDimension::ALL.iter())
        {
            assert_eq!(status.dimension, *dimension);
            assert!(!status.dimension.gap_headline().is_empty());
            assert!(!status.dimension.unlock_hint().is_empty());
        }

        // The success state: every dimension admitted, nothing missing.
        let admitted_all = CapabilityGapDetail::new(
            "Terminal".to_owned(),
            CapabilityDimension::ALL
                .iter()
                .map(|dimension| DimensionStatus::new(*dimension, true))
                .collect(),
        );
        assert!(admitted_all.available());
        assert!(admitted_all.missing().is_empty());
        assert_eq!(admitted_all.admitted().len(), 5);
    }

    #[test]
    fn palette_queries_resolve_to_the_gap_row() {
        // Discoverability contract §1.3: the palette fallback finds the
        // surface without knowing its location — natural queries hit the
        // row's title or description (WO-P2-010 contract at the copy
        // level).
        let title = PALETTE_ROW_TITLE;
        let description = PALETTE_ROW_DESCRIPTION;
        for query in ["capability", "unavailable", "why"] {
            assert!(
                title.to_lowercase().contains(query) || description.to_lowercase().contains(query),
                "query {query:?} must resolve to the gap palette row"
            );
        }
        assert_eq!(title, "Why is a capability unavailable?");
    }

    /// CAP-001 registration seam test (the house ui.rs source-inspection
    /// style): the gap surface must be wired through the palette, the
    /// keyboard, the dispatch, the task-surface render, and the
    /// navigation-close seams in `ui.rs` — the seven-layer discipline,
    /// with the palette never the only discovery mechanism.
    #[test]
    fn the_gap_surface_is_registered_in_the_ui_seams() {
        let source = include_str!("../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_capability_gap;"));

        // The palette registration seam: the row dispatches through this
        // module.
        assert!(source.contains("PaletteCommand::InspectCapabilityGaps"));
        assert!(source.contains("flauz_capability_gap::open_capability_gap_surface("));

        // The visible-affordance registration seam: the labeled entry
        // control on the task surface, directly under the task control
        // rail (the More/Inspect neighborhood). rustfmt legitimately
        // wraps the call across lines, so assert the call fragment plus
        // the rail it follows.
        assert!(source.contains("flauz_capability_gap::render_capability_gap_entry"));
        assert!(
            source.contains("flauz_shell::render_task_rail(self, window, cx)"),
            "the entry renders in the task rail's More/Inspect neighborhood"
        );

        // The keyboard registration seams: the direct chord (the rail
        // family) plus the shifted-symbol companion (the N6 gate-fix
        // family: on Linux a Shift+6 keystroke reports "^" as the key
        // with the shift modifier stripped).
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzCapabilityGapShortcut, None"),
            "a keyboard chord must be bound for the gap surface"
        );
        assert!(source.contains("shortcut(\"alt-shift-6\")"));
        assert!(source.contains("shortcut(\"alt-^\")"));

        // The navigation-close seam: F1 navigation closes the panel.
        assert!(source.contains("self.flauz_capability_gap.close_for_navigation()"));

        // The palette hint and the scoped Escape context.
        assert!(source.contains("Some(\"Ctrl+Alt+Shift+6\")"));
        assert!(source.contains("Some(\"FlauzCapabilityGap\")"));
    }
}
