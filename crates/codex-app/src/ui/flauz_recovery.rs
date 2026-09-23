//! The recovery surface (work order ORCH-003, F6 Wave 3): the J-03
//! banner — "Picking up where we left off", in user language only.
//!
//! When a task was interrupted (a disconnect, a restart, context
//! pressure handled by summarizing older details), the task surface
//! shows a state-driven banner that says what is happening in words a
//! non-technical user needs: what was kept, what was summarized, and
//! the two actions — **Review and continue** and **Escalate to me**.
//! The escalation state is explicit and visible ("Needs you"), never a
//! silent failure.
//!
//! Seven discoverability layers (PRODUCT-UX-JOURNEYS §1 / the shell
//! family discipline):
//!
//! 1. **Visible primary entry** — the banner sits on the task surface,
//!    state-driven: always visible while the task is being picked up
//!    or needs the user, and absent when there is nothing to recover
//!    (never a permanent banner).
//! 2. **Contextual affordance** — the "What we kept / What was
//!    summarized" disclosure panel inside the banner.
//! 3. **Palette fallback** — the "Resume this task where it left off"
//!    row (the palette shows it under the Workspace group; firing it
//!    when there is nothing to pick up surfaces honest guidance, and
//!    the live wiring hides the row once the view-model reports the
//!    task is not recoverable).
//! 4. **Stateful empty state** — nothing to recover → NO banner; the
//!    task surface is unchanged, and the chord/palette say so honestly
//!    instead of a silent no-op.
//! 5. **Success-state continuation** — after review: "Resumed from <n>
//!    events · everything kept except what was summarized" with the
//!    next step ("continue working in the conversation").
//! 6. **Keyboard path** — `Ctrl+Alt+Shift+R` focuses the banner;
//!    scoped Escape releases focus back to the task surface.
//! 7. **Honest unavailable state** — the escalated "Needs you" state:
//!    explicit, visible, with the reason and the review-and-continue
//!    path.
//!
//! Boundary rules (work order ORCH-003 / kernel §1):
//!
//! - the app crate does NOT import the `flauz-exec` contract crate in
//!   this wave — the banner renders a plain **view-model**
//!   ([`RecoveryView`]) that a later wave populates from the harness
//!   view + the recovery brief (the `task.model_changed`-style seam
//!   discipline: [`set_recovery_view`] is the wiring point). Until
//!   wiring lands there is no banner and no invented data — the honest
//!   empty state;
//! - the open/close paths follow the F1 focus contracts and the d19
//!   modal-trap lesson: the banner captures the previously focused
//!   surface when the chord focuses it, auto-focuses its own handle
//!   once (the 019 request-once shape), and one scoped Escape returns
//!   focus to the task surface — keyboard focus is never trapped. The
//!   banner is passive on mount (no focus steal); the d21 PTY
//!   discipline applies when the recovery path lands on a terminal
//!   surface: focus transfers only through the explicit open path;
//! - existing F1 flows are untouched: the module is wired through
//!   minimal `ui.rs` named seams (module declaration, palette row,
//!   keyboard chord, scoped escape, state field, task-surface mount,
//!   navigation close) — distinct from MOD-001's and CAP-001's seams.
//!
//! Every user-facing string lives in the copy registry below so the
//! language rules are unit-testable in one place.

use codex_core::MainRoute;
use gpui::prelude::*;
use gpui::{AnyElement, Context, FocusHandle, IntoElement, Window, div, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::Escape,
    v_flex,
};

use super::WorkspaceView;

gpui::actions!(codexrs, [FlauzRecoveryShortcut]);

// ---------------------------------------------------------------------------
// Copy registry (user language; never a technical term — J-03)
// ---------------------------------------------------------------------------

/// The banner headline while the task is being picked up (the
/// work order's exact phrase).
pub(crate) const BANNER_TITLE_PICKING_UP: &str = "Picking up where we left off";
/// The banner headline once the user is back in the task.
pub(crate) const BANNER_TITLE_RESUMED: &str = "Back where you left off";
/// The escalated state's explicit headline (the work order's exact
/// phrase).
pub(crate) const BANNER_TITLE_NEEDS_YOU: &str = "Needs you";
/// The status line while the task is being picked up.
pub(crate) const PICKING_UP_STATUS: &str = "Everything this task kept is here — nothing was lost.";
/// The status line after review, when nothing was summarized (the
/// work order's exact shape: "Resumed from <n> events · everything
/// kept").
pub(crate) const RESUMED_STATUS_KEPT_ALL: &str = "Resumed from {events} events · everything kept";
/// The status line after review, when older details were summarized
/// (the work order's exact shape: "… · everything kept except <the
/// compaction record>", said in user language).
pub(crate) const RESUMED_STATUS_KEPT_EXCEPT_SUMMARIZED: &str =
    "Resumed from {events} events · everything kept except what was summarized";
/// The heading over the kept families (the disclosure panel).
pub(crate) const KEPT_HEADING: &str = "What we kept";
/// The heading over the summarized line.
pub(crate) const SUMMARIZED_HEADING: &str = "What was summarized";
/// The summarized line: honest about what summarizing means — nothing
/// was deleted.
pub(crate) const SUMMARIZED_LINE: &str =
    "We summarized {items} older details to make room — nothing was deleted.";
/// The heading over the escalation reason.
pub(crate) const ESCALATION_HEADING: &str = "Why this needs you";
/// The escalation reason recorded when the user takes the task over
/// themselves.
pub(crate) const ESCALATE_TO_ME_REASON: &str = "You asked to take this over yourself.";
/// The primary action (the work order's exact label).
pub(crate) const REVIEW_AND_CONTINUE: &str = "Review and continue";
/// The escalation action (the work order's exact label).
pub(crate) const ESCALATE_TO_ME: &str = "Escalate to me";
/// The next-step line after review (layer 5).
pub(crate) const NEXT_STEP_RESUMED: &str =
    "Continue working in the conversation — your task is exactly as it was.";
/// The disclosure toggle that opens the kept/summarized panel.
pub(crate) const DISCLOSURE_SHOW: &str = "See what was kept and summarized";
/// The disclosure toggle that closes it again.
pub(crate) const DISCLOSURE_HIDE: &str = "Hide the details";
/// The honest guidance when the chord or palette row fires with no
/// chat selected (the WO-P2-012 pattern: guidance, never a silent
/// no-op).
pub(crate) const RECOVERY_NO_CHAT_GUIDANCE: &str = "Open a chat to pick up where it left off.";
/// The honest guidance when the selected task has nothing to recover
/// (layer 4: never a permanent banner, never a silent no-op).
pub(crate) const NOTHING_TO_RECOVER_GUIDANCE: &str =
    "This task is up to date — there's nothing to pick up right now.";
/// The keyboard-hint footer (the focus contract, said plainly).
pub(crate) const ESCAPE_HINT: &str = "Escape returns to your work";
/// The command-palette row title (the work order's exact row).
pub(crate) const PALETTE_ROW_TITLE: &str = "Resume this task where it left off";
/// The command-palette row description.
pub(crate) const PALETTE_ROW_DESCRIPTION: &str =
    "Pick up an interrupted task with everything it kept";

// ---------------------------------------------------------------------------
// The view-model (plain data; the wiring seam populates it)
// ---------------------------------------------------------------------------

/// One kept family in the recovery view-model — the J-03 user
/// vocabulary for the durable families that survived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // constructed by the module's tests and (Wave-later) the harness-view population
pub(crate) enum KeptKind {
    /// Durable work products (artifacts, in the platform's words).
    WorkProducts,
    /// Verified results (evidence, in the platform's words).
    VerifiedResults,
    /// Remembered notes (memory, in the platform's words).
    Notes,
}

impl KeptKind {
    /// The user-facing label, pluralized for a count.
    pub(crate) fn label(self, count: u64) -> String {
        match (self, count == 1) {
            (Self::WorkProducts, true) => "work product".to_owned(),
            (Self::WorkProducts, false) => "work products".to_owned(),
            (Self::VerifiedResults, true) => "verified result".to_owned(),
            (Self::VerifiedResults, false) => "verified results".to_owned(),
            (Self::Notes, true) => "note".to_owned(),
            (Self::Notes, false) => "notes".to_owned(),
        }
    }
}

/// One kept family's summary row: a kind and how many survived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // constructed by the module's tests and (Wave-later) the harness-view population
pub(crate) struct KeptSummary {
    /// Which family survived.
    pub kind: KeptKind,
    /// How many of them the task still holds.
    pub count: u64,
}

impl KeptSummary {
    /// Builds one kept row.
    #[allow(dead_code)] // used by the module's tests; the Wave-later population builds rows through it
    pub(crate) const fn new(kind: KeptKind, count: u64) -> Self {
        Self { kind, count }
    }

    /// The row's user-facing line ("3 work products").
    pub(crate) fn line(&self) -> String {
        format!("{} {}", self.count, self.kind.label(self.count))
    }
}

/// What the summarizer did, in user language: how many older details
/// were summarized so the task could keep going.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // constructed by the module's tests and (Wave-later) the recovery-brief population
pub(crate) struct CompactionSummary {
    /// How many older details were summarized.
    pub summarized_items: u64,
}

/// The banner's phase: picking up, escalated, or resumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryPhase {
    /// The task is being picked up where it left off.
    PickingUp,
    /// The task needs the user (explicit and visible).
    NeedsYou,
    /// The user reviewed and continued; the success state.
    Resumed,
}

/// The recovery view-model: what the banner renders. Plain owned data
/// — a later wave populates it from the harness view + the recovery
/// brief; this module never invents recovery data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoveryView {
    /// The banner's phase.
    phase: RecoveryPhase,
    /// How many of the machine's own events the reconstruction
    /// replayed ("Resumed from <n> events").
    resumed_from_events: u64,
    /// The kept families (the disclosure panel).
    kept: Vec<KeptSummary>,
    /// What was summarized, when anything was.
    summarized: Option<CompactionSummary>,
    /// Why the task needs the user, when it does.
    escalation_reason: Option<String>,
}

impl RecoveryView {
    /// Builds the picking-up view (the wiring seam's entry state).
    #[allow(dead_code)] // used by the module's tests; the Wave-later population builds views through it
    pub(crate) fn picking_up(
        resumed_from_events: u64,
        kept: Vec<KeptSummary>,
        summarized: Option<CompactionSummary>,
    ) -> Self {
        Self {
            phase: RecoveryPhase::PickingUp,
            resumed_from_events,
            kept,
            summarized,
            escalation_reason: None,
        }
    }

    /// The banner headline for this phase.
    pub(crate) fn headline(&self) -> &'static str {
        match self.phase {
            RecoveryPhase::PickingUp => BANNER_TITLE_PICKING_UP,
            RecoveryPhase::NeedsYou => BANNER_TITLE_NEEDS_YOU,
            RecoveryPhase::Resumed => BANNER_TITLE_RESUMED,
        }
    }

    /// The banner icon for this phase.
    pub(crate) fn icon(&self) -> IconName {
        match self.phase {
            RecoveryPhase::PickingUp => IconName::Undo2,
            RecoveryPhase::NeedsYou => IconName::TriangleAlert,
            RecoveryPhase::Resumed => IconName::CircleCheck,
        }
    }

    /// The status line under the headline.
    pub(crate) fn status_line(&self) -> String {
        match self.phase {
            RecoveryPhase::PickingUp => PICKING_UP_STATUS.to_owned(),
            RecoveryPhase::NeedsYou => PICKING_UP_STATUS.to_owned(),
            RecoveryPhase::Resumed => {
                if self.summarized.is_some() {
                    RESUMED_STATUS_KEPT_EXCEPT_SUMMARIZED
                        .replace("{events}", &self.resumed_from_events.to_string())
                } else {
                    RESUMED_STATUS_KEPT_ALL
                        .replace("{events}", &self.resumed_from_events.to_string())
                }
            }
        }
    }

    /// The kept families' rows, in view order.
    pub(crate) fn kept_lines(&self) -> Vec<String> {
        self.kept.iter().map(KeptSummary::line).collect()
    }

    /// The summarized line, when anything was summarized.
    pub(crate) fn summarized_line(&self) -> Option<String> {
        self.summarized.map(|summary| {
            SUMMARIZED_LINE.replace("{items}", &summary.summarized_items.to_string())
        })
    }

    /// Why the task needs the user, when it does.
    pub(crate) fn escalation_reason(&self) -> Option<&str> {
        self.escalation_reason.as_deref()
    }

    /// Whether the task can be resumed where it left off (the palette
    /// row's visibility rule: picking up or needs-you, not already
    /// resumed).
    pub(crate) fn is_recoverable(&self) -> bool {
        self.phase != RecoveryPhase::Resumed
    }

    /// The user asked to take the task over: the banner moves to the
    /// explicit "Needs you" state with the recorded reason. Returns
    /// whether anything changed.
    pub(crate) fn escalate(&mut self, reason: &str) -> bool {
        if self.phase == RecoveryPhase::Resumed || self.phase == RecoveryPhase::NeedsYou {
            return false;
        }
        self.phase = RecoveryPhase::NeedsYou;
        self.escalation_reason = Some(reason.to_owned());
        true
    }

    /// The user reviewed and continued: the banner moves to the
    /// success state ("Resumed from <n> events · everything kept…").
    /// Returns whether anything changed.
    pub(crate) fn review_and_continue(&mut self) -> bool {
        if self.phase == RecoveryPhase::Resumed {
            return false;
        }
        self.phase = RecoveryPhase::Resumed;
        true
    }
}

// ---------------------------------------------------------------------------
// The additive surface state
// ---------------------------------------------------------------------------

/// The additive recovery state: the view-model seam, the kept
/// disclosure toggle, and the banner's focus bookkeeping.
pub(crate) struct RecoveryState {
    view: Option<RecoveryView>,
    /// Whether the "What we kept" disclosure is expanded.
    disclosure_open: bool,
    /// The banner's keyboard focus handle (the 019 request-once
    /// shape).
    banner_focus: FocusHandle,
    /// Request-once guard for `banner_focus`.
    banner_focus_requested: bool,
    /// The surface that held focus when the chord focused the banner,
    /// restored on close (the 017 close contract).
    focus_before_banner: Option<FocusHandle>,
}

impl RecoveryState {
    /// Builds the closed, empty recovery state (no banner: the honest
    /// empty state — nothing to recover).
    pub(crate) fn new(cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            view: None,
            disclosure_open: false,
            banner_focus: cx.focus_handle(),
            banner_focus_requested: false,
            focus_before_banner: None,
        }
    }

    /// The attached view-model, when one has been wired in.
    #[allow(dead_code)] // the Wave-later wiring reads it; the render paths check `view` directly today
    pub(crate) fn view(&self) -> Option<&RecoveryView> {
        self.view.as_ref()
    }

    /// Whether the selected task can be resumed where it left off (the
    /// palette row's visibility rule).
    #[allow(dead_code)] // the Wave-later palette wiring reads it; the module's tests pin the rule
    pub(crate) fn is_recoverable(&self) -> bool {
        self.view.as_ref().is_some_and(|view| view.is_recoverable())
    }

    /// Attaches (or clears) the recovery view-model — the wiring seam a
    /// later wave drives from the harness view + the recovery brief.
    /// Clearing the view removes the banner (the task surface returns
    /// to normal — never a permanent banner).
    pub(crate) fn set_view(&mut self, view: Option<RecoveryView>) {
        self.view = view;
        if self.view.is_none() {
            // Nothing to recover: drop the disclosure and the focus
            // bookkeeping with it.
            self.disclosure_open = false;
            self.banner_focus_requested = false;
            self.focus_before_banner = None;
        }
    }

    /// Quietly drops the focus bookkeeping and the disclosure for an
    /// F1 navigation action. The VIEW stays: the banner is
    /// state-driven, always visible while the task is being picked up
    /// or needs the user — navigation must not dismiss it. Returns
    /// whether anything changed.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.disclosure_open || self.focus_before_banner.is_some();
        self.disclosure_open = false;
        self.banner_focus_requested = false;
        self.focus_before_banner = None;
        changed
    }
}

// ---------------------------------------------------------------------------
// The surface actions
// ---------------------------------------------------------------------------

/// Focuses the recovery banner (the chord/palette path). With no chat
/// selected, or with nothing to recover, surfaces honest guidance
/// instead of a silent no-op (the WO-P2-012 pattern).
pub(crate) fn open_recovery_surface(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.state.selected_task_id.is_none() {
        workspace.dispatch_command_status(Some(RECOVERY_NO_CHAT_GUIDANCE), cx);
        return;
    }
    if !workspace.flauz_recovery.is_recoverable() {
        workspace.dispatch_command_status(Some(NOTHING_TO_RECOVER_GUIDANCE), cx);
        return;
    }
    // The banner lives on the task surface: navigate there first when
    // needed, so it is reachable from any route.
    if workspace.state.route != MainRoute::Tasks {
        workspace.navigate(MainRoute::Tasks, cx);
    }
    let state = &mut workspace.flauz_recovery;
    if state.focus_before_banner.is_none() {
        state.focus_before_banner = window.focused(cx);
    }
    state.banner_focus_requested = true;
    cx.notify();
}

/// Releases the banner's keyboard focus and restores the previously
/// focused surface (the 017 close contract — the d19 discipline: never
/// trapped). The banner itself stays while the task needs it.
pub(crate) fn release_recovery_focus(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let previous = workspace.flauz_recovery.focus_before_banner.take();
    workspace.flauz_recovery.banner_focus_requested = false;
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// The "Review and continue" action: the view moves to the resumed
/// success state and focus returns to the task surface. (A later wave
/// also drives the harness's continue transition through the same
/// seam.)
pub(crate) fn review_and_continue(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let mut resumed = false;
    if let Some(view) = workspace.flauz_recovery.view.as_mut() {
        resumed = view.review_and_continue();
    }
    if resumed {
        // The success state collapses the disclosure: the status line
        // carries the summary now.
        workspace.flauz_recovery.disclosure_open = false;
    }
    release_recovery_focus(workspace, window, cx);
}

/// The "Escalate to me" action: the view moves to the explicit "Needs
/// you" state with the recorded reason.
pub(crate) fn escalate_to_me(
    workspace: &mut WorkspaceView,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if let Some(view) = workspace.flauz_recovery.view.as_mut() {
        view.escalate(ESCALATE_TO_ME_REASON);
    }
    cx.notify();
}

/// Toggles the "What we kept" disclosure (the contextual affordance).
pub(crate) fn toggle_kept_disclosure(
    workspace: &mut WorkspaceView,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    workspace.flauz_recovery.disclosure_open = !workspace.flauz_recovery.disclosure_open;
    cx.notify();
}

/// Attaches (or clears) the recovery view-model — the wiring seam
/// (see [`RecoveryState::set_view`]).
#[allow(dead_code)] // the Wave-later harness wiring calls it; the module's tests pin the behavior
pub(crate) fn set_recovery_view(
    workspace: &mut WorkspaceView,
    view: Option<RecoveryView>,
    cx: &mut Context<WorkspaceView>,
) {
    workspace.flauz_recovery.set_view(view);
    cx.notify();
}

// ---------------------------------------------------------------------------
// The render
// ---------------------------------------------------------------------------

/// Renders the recovery banner on the task surface — state-driven:
/// nothing when there is no view-model (the honest empty state, never
/// a permanent banner), the picking-up/needs-you/resumed banner
/// otherwise.
pub(crate) fn render_recovery_banner(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let Some(view) = workspace.flauz_recovery.view.clone() else {
        // Layer 4: nothing to recover → NO banner. An empty element
        // keeps the mount cheap and the layout unchanged.
        return div().into_any_element();
    };
    if workspace.flauz_recovery.banner_focus_requested {
        // The 019 request-once shape: the banner claims the keyboard
        // when the chord focuses it, so the scoped Escape binding
        // reaches it.
        workspace.flauz_recovery.banner_focus_requested = false;
        workspace.flauz_recovery.banner_focus.focus(window);
    }
    let banner_focus = workspace.flauz_recovery.banner_focus.clone();
    let disclosure_open = workspace.flauz_recovery.disclosure_open;
    let phase = view.phase;

    let mut banner = v_flex()
        .key_context("FlauzRecovery")
        .track_focus(&banner_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            release_recovery_focus(this, window, cx);
        }))
        .mx_5()
        .my_2()
        .p_4()
        .gap_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().sidebar)
        // The headline row: the phase icon + headline.
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(view.icon()).small())
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(view.headline().to_owned()),
                ),
        )
        // The status line.
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(view.status_line()),
        );
    // The escalation reason: explicit and visible (J-08's marker).
    if let Some(reason) = view.escalation_reason() {
        banner = banner.child(
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
                        .child(ESCALATION_HEADING),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .child(reason.to_owned()),
                ),
        );
    }
    // The disclosure toggle (layer 2).
    banner = banner.child(
        Button::new("flauz-recovery-disclosure")
            .label(if disclosure_open {
                DISCLOSURE_HIDE
            } else {
                DISCLOSURE_SHOW
            })
            .tooltip("See what this task kept, and what was summarized")
            .small()
            .ghost()
            .selected(disclosure_open)
            .on_click(cx.listener(|this, _, window, cx| {
                toggle_kept_disclosure(this, window, cx);
            })),
    );
    // The disclosure panel: the kept rows and the summarized line.
    if disclosure_open {
        let mut panel = v_flex().gap_1().pl_2();
        panel = panel.child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(cx.theme().muted_foreground)
                .child(KEPT_HEADING),
        );
        for line in view.kept_lines() {
            panel = panel.child(
                div()
                    .text_sm()
                    .line_height(px(20.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(line),
            );
        }
        if let Some(summarized) = view.summarized_line() {
            panel = panel
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(cx.theme().muted_foreground)
                        .child(SUMMARIZED_HEADING),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(summarized),
                );
        }
        banner = banner.child(panel);
    }
    // The actions / next step (layer 5): review-and-continue always;
    // escalate-to-me while picking up (once escalated, the state IS
    // the escalation).
    let mut actions = h_flex().gap_2().items_center().flex_wrap();
    actions = actions.child(
        Button::new("flauz-recovery-continue")
            .label(REVIEW_AND_CONTINUE)
            .icon(IconName::ArrowRight)
            .tooltip("Look at what was kept, then continue this task")
            .small()
            .primary()
            .on_click(cx.listener(|this, _, window, cx| {
                review_and_continue(this, window, cx);
            })),
    );
    if phase == RecoveryPhase::PickingUp {
        actions = actions.child(
            Button::new("flauz-recovery-escalate")
                .label(ESCALATE_TO_ME)
                .icon(IconName::TriangleAlert)
                .tooltip("Take this task over yourself")
                .small()
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    escalate_to_me(this, window, cx);
                })),
        );
    }
    banner = banner.child(actions);
    if phase == RecoveryPhase::Resumed {
        banner = banner.child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .child(NEXT_STEP_RESUMED),
        );
    }
    banner = banner.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(ESCAPE_HINT),
    );
    banner.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picking_up_view() -> RecoveryView {
        RecoveryView::picking_up(
            12,
            vec![
                KeptSummary::new(KeptKind::WorkProducts, 3),
                KeptSummary::new(KeptKind::VerifiedResults, 2),
                KeptSummary::new(KeptKind::Notes, 5),
            ],
            Some(CompactionSummary {
                summarized_items: 4,
            }),
        )
    }

    fn all_user_copy() -> Vec<String> {
        let mut copy: Vec<String> = [
            BANNER_TITLE_PICKING_UP,
            BANNER_TITLE_RESUMED,
            BANNER_TITLE_NEEDS_YOU,
            PICKING_UP_STATUS,
            KEPT_HEADING,
            SUMMARIZED_HEADING,
            ESCALATION_HEADING,
            ESCALATE_TO_ME_REASON,
            REVIEW_AND_CONTINUE,
            ESCALATE_TO_ME,
            NEXT_STEP_RESUMED,
            DISCLOSURE_SHOW,
            DISCLOSURE_HIDE,
            RECOVERY_NO_CHAT_GUIDANCE,
            NOTHING_TO_RECOVER_GUIDANCE,
            ESCAPE_HINT,
            PALETTE_ROW_TITLE,
            PALETTE_ROW_DESCRIPTION,
            "See what this task kept, and what was summarized",
            "Look at what was kept, then continue this task",
            "Take this task over yourself",
        ]
        .iter()
        .map(|text| (*text).to_owned())
        .collect();
        // The generated copy renders through the view-model too.
        let view = picking_up_view();
        copy.push(view.status_line());
        copy.push(view.headline().to_owned());
        for line in view.kept_lines() {
            copy.push(line);
        }
        copy.push(view.summarized_line().unwrap_or_default());
        let mut escalated = picking_up_view();
        escalated.escalate(ESCALATE_TO_ME_REASON);
        copy.push(escalated.headline().to_owned());
        copy.push(escalated.status_line());
        let mut resumed = picking_up_view();
        resumed.review_and_continue();
        copy.push(resumed.headline().to_owned());
        copy.push(resumed.status_line());
        let no_compaction = RecoveryView::picking_up(3, Vec::new(), None);
        let mut resumed_clean = no_compaction;
        resumed_clean.review_and_continue();
        copy.push(resumed_clean.status_line());
        copy.push(KeptKind::WorkProducts.label(1));
        copy.push(KeptKind::WorkProducts.label(2));
        copy.push(KeptKind::VerifiedResults.label(1));
        copy.push(KeptKind::Notes.label(3));
        copy
    }

    /// J-03's law (the Wave-3 addendum §7): the recovery language is
    /// non-technical — never "context window", "token budget",
    /// "rehydration"; and no internal implementation term leaks.
    #[test]
    fn recovery_copy_uses_user_language_only() {
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "context window",
                "token budget",
                "token",
                "rehydrat",
                "compaction",
                "compact",
                "harness",
                "state machine",
                "event sourcing",
                "snapshot",
                "orch-003",
                "flauz-exec",
                "flauz-context",
                "recoveryview",
                "keptsummary",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "recovery copy must never say {forbidden:?}: {copy:?}"
                );
            }
        }
        // The work order's exact phrases are present, verbatim.
        assert_eq!(BANNER_TITLE_PICKING_UP, "Picking up where we left off");
        assert_eq!(BANNER_TITLE_NEEDS_YOU, "Needs you");
        assert_eq!(REVIEW_AND_CONTINUE, "Review and continue");
        assert_eq!(ESCALATE_TO_ME, "Escalate to me");
        assert_eq!(PALETTE_ROW_TITLE, "Resume this task where it left off");
    }

    /// The banner states transition honestly: picking up → escalate →
    /// needs-you (reason kept) → review-and-continue → resumed; and
    /// repeated transitions are no-ops, never lies.
    #[test]
    fn recovery_view_transitions_stay_honest() {
        let mut view = picking_up_view();
        assert_eq!(view.headline(), BANNER_TITLE_PICKING_UP);
        assert!(view.is_recoverable());
        assert_eq!(view.escalation_reason(), None);

        // Escalate: explicit, visible, with the recorded reason.
        assert!(view.escalate(ESCALATE_TO_ME_REASON));
        assert_eq!(view.headline(), BANNER_TITLE_NEEDS_YOU);
        assert_eq!(view.escalation_reason(), Some(ESCALATE_TO_ME_REASON));
        assert!(
            view.is_recoverable(),
            "a task that needs you can still be resumed"
        );
        // Escalating again changes nothing.
        assert!(!view.escalate("again"));

        // Review and continue: the success state.
        assert!(view.review_and_continue());
        assert_eq!(view.headline(), BANNER_TITLE_RESUMED);
        assert!(
            !view.is_recoverable(),
            "an already-resumed task has nothing to resume"
        );
        assert!(view.status_line().starts_with("Resumed from 12 events"));
        assert!(view.status_line().contains("except what was summarized"));
        // Continuing again changes nothing.
        assert!(!view.review_and_continue());

        // Direct review from picking up (no escalation on the way).
        let mut direct = picking_up_view();
        assert!(direct.review_and_continue());
        assert_eq!(direct.headline(), BANNER_TITLE_RESUMED);
    }

    /// The success copy names what was kept and what was summarized
    /// (the order's exact shapes), with and without a compaction
    /// record.
    #[test]
    fn recovery_success_copy_names_kept_and_summarized() {
        let mut view = picking_up_view();
        view.review_and_continue();
        assert_eq!(
            view.status_line(),
            "Resumed from 12 events · everything kept except what was summarized"
        );
        assert_eq!(
            view.kept_lines(),
            vec![
                "3 work products".to_owned(),
                "2 verified results".to_owned(),
                "5 notes".to_owned(),
            ]
        );
        assert_eq!(
            view.summarized_line().as_deref(),
            Some("We summarized 4 older details to make room — nothing was deleted.")
        );

        // Nothing was summarized: everything kept, no summarized line.
        let mut clean = RecoveryView::picking_up(3, Vec::new(), None);
        clean.review_and_continue();
        assert_eq!(
            clean.status_line(),
            "Resumed from 3 events · everything kept"
        );
        assert_eq!(clean.summarized_line(), None);
        assert!(clean.kept_lines().is_empty());

        // Pluralization stays honest at one.
        assert_eq!(
            KeptSummary::new(KeptKind::VerifiedResults, 1).line(),
            "1 verified result"
        );
    }

    /// Layer 4: nothing to recover → no banner — the state says so,
    /// and the honest guidance (never a silent no-op) is wired.
    #[test]
    fn nothing_to_recover_renders_no_banner() {
        // The wiring seam clears the view: no banner, not recoverable.
        // (The render path returns the empty element for a missing
        // view; the predicate is the rule the palette and the chord
        // consult, pinned here.)
        let view = RecoveryView::picking_up(1, Vec::new(), None);
        assert!(view.is_recoverable());
        let mut resumed = view;
        resumed.review_and_continue();
        assert!(!resumed.is_recoverable());
        // The honest guidance exists and says what happened.
        assert!(NOTHING_TO_RECOVER_GUIDANCE.contains("nothing to pick up"));
        assert!(RECOVERY_NO_CHAT_GUIDANCE.contains("Open a chat"));
    }

    /// Layer 3: the palette fallback finds the resume row through
    /// natural queries.
    #[test]
    fn palette_query_resolves_to_the_resume_row() {
        for query in ["resume", "pick up", "left off", "interrupted", "everything"] {
            let title = PALETTE_ROW_TITLE.to_lowercase();
            let description = PALETTE_ROW_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the resume row"
            );
        }
    }

    /// The ORCH-003 registration seams in `ui.rs` (the house
    /// source-inspection style): the module declaration, the palette
    /// row, the keyboard chord, the scoped escape, the state field, the
    /// task-surface mount, and the navigation close — each tagged
    /// ORCH-003, distinct from MOD-001's and CAP-001's seams.
    #[test]
    fn recovery_seams_are_registered_in_the_ui_seams() {
        let source = include_str!("../../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_recovery;"));

        // The palette registration seam.
        assert!(source.contains("PaletteCommand::ResumeTaskWhereItLeftOff"));
        assert!(source.contains("flauz_recovery::PALETTE_ROW_TITLE"));
        assert!(source.contains("flauz_recovery::open_recovery_surface(workspace, window, cx)"));

        // The keyboard chord seam (Ctrl+Alt+Shift+R, the letter family).
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzRecoveryShortcut, None"),
            "the recovery chord must be bound"
        );
        assert!(source.contains("Some(\"Ctrl+Alt+Shift+R\")"));

        // The scoped escape seam (the d19 discipline: one Escape
        // through the banner's own focus context, never a trap).
        assert!(source.contains("Some(\"FlauzRecovery\")"));

        // The state-field seam.
        assert!(source.contains("flauz_recovery: flauz_recovery::RecoveryState"));
        assert!(source.contains("flauz_recovery::RecoveryState::new(cx)"));

        // The task-surface mount seam (whitespace-insensitive: rustfmt
        // legitimately wraps the mount expression and adds its trailing
        // comma — the picker/shell seam-test precedent).
        let joined: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            joined.contains("flauz_recovery::render_recovery_banner("),
            "the banner must be mounted on the task surface"
        );

        // The navigation-close seam (quiet: the view is state-driven
        // and survives navigation).
        assert!(source.contains("self.flauz_recovery.close_for_navigation()"));

        // Every ORCH-003 seam is tagged.
        let seam_tags = source.matches("ORCH-003").count();
        assert!(
            seam_tags >= 7,
            "each seam is tagged ORCH-003 (found {seam_tags})"
        );
    }

    /// The d19 discipline, made checkable: the banner's focus paths
    /// follow the request-once + restore contract, and the release
    /// path is the one the scoped Escape dispatches through.
    #[test]
    fn recovery_focus_never_traps() {
        let source = include_str!("flauz_recovery.rs");
        // The request-once shape: focus is requested once and consumed
        // by the render.
        assert!(source.contains("banner_focus_requested = true"));
        assert!(source.contains("banner_focus_requested = false"));
        // The 017 close contract: the previously focused surface is
        // captured and restored on release.
        assert!(source.contains("focus_before_banner = window.focused(cx)"));
        assert!(source.contains("apply_overlay_close_focus_restore"));
        // The scoped Escape dispatches through the release path, not a
        // dismissal: the banner itself is state-driven.
        assert!(source.contains("release_recovery_focus(this, window, cx)"));
        // The banner takes focus ONLY through the explicit open path
        // (the d21 discipline: no focus steal on mount).
        assert!(source.contains(".focus(window)"));
    }
}
