//! The members + presence surface (work order COL-001, F9 Wave 4): who
//! is on this workspace, what they can do, what they're doing — and the
//! per-task sharing posture, in user language with consequences.
//!
//! The surface pair:
//!
//! - the **members panel** (Workspace-level): the member list (name,
//!   role, presence in user words — "Dev is viewing this task"), the
//!   invite/role affordances with honest **not-wired** states (the real
//!   transport arrives with the connected workspaces of a later wave —
//!   the module says so plainly and invents nothing), and the honest
//!   solo empty state ("Just you" + what inviting adds);
//! - the **per-task sharing surface** (task surface): the worktree
//!   posture + shared-filesystem mode + context visibility in user
//!   words WITH consequences ("Isolated copy — your files stay separate"
//!   / "Shared files — everyone on this task can read and write them";
//!   "Only me" / "The workspace"), the change affordance, and the
//!   permission-gated honest unavailable state with the WHY (the
//!   CAP-001 pattern).
//!
//! Seven discoverability layers (PRODUCT-UX-JOURNEYS §1 / the shell
//! family discipline):
//!
//! 1. **Visible primary entry** — the members button in the title bar
//!    (the WO-P2-018 bell precedent) and the labeled Sharing control on
//!    the task surface (the picker/save-family precedent).
//! 2. **Contextual affordance** — the per-task sharing panel from the
//!    task surface; the invite affordances inside the members panel;
//!    permission-gated affordances state WHY they are unavailable.
//! 3. **Palette fallback** — the "See who is on this workspace" and
//!    "Change a task's sharing" rows (the palette is never the only
//!    discovery mechanism).
//! 4. **Stateful empty state** — solo workspace: the honest "Just you"
//!    state + what inviting adds; no roster wired: the same honest
//!    state (nothing is invented).
//! 5. **Success/next-step** — after a sharing change: the posture
//!    restated WITH its consequence ("Sharing changed — Shared files:
//!    everyone on this task can read them"); after opening with
//!    members: the list with presence.
//! 6. **Keyboard path** — `Ctrl+Alt+Shift+U` opens the members panel
//!    (the letter family; the work order's suggested `M` was verified
//!    TAKEN by the model picker — `U` is the verified free letter), one
//!    scoped Escape closes, the panels are tab-navigable.
//! 7. **Honest unavailable state** — the not-wired invite/role flows
//!    say so plainly ("nothing is sent today"); the permission-gated
//!    sharing change states the role-based WHY.
//!
//! Boundary rules (work order COL-001 / Wave-4 kernel addendum §1, §5,
//! §6):
//!
//! - the app crate does NOT import the `flauz-collab` contract crate in
//!   this wave (the app's Cargo.toml is outside this order's owned
//!   files): the panel renders a plain **view-model**
//!   ([`MembersView`], [`SharingView`]) that a later wave populates
//!   from the collaboration contracts ([`set_members_view`] /
//!   [`set_sharing_view`] are the wiring seams). Until wiring lands
//!   the roster is the honest empty state and the sharing posture is
//!   the honest platform default (isolated files, "Only me" notes) —
//!   both true today, neither invented;
//! - the **attention-model extension** (addendum §5): collaborator
//!   attribution on the EXISTING Activity rows is ADDITIVE data
//!   ([`AttentionNeed`] rows held here) + an additive render line the
//!   existing Activity row renderer draws through
//!   [`attention_attribution_line`] — there is NO second event store
//!   and NO parallel feed (the seam test pins it by source
//!   inspection);
//! - the sharing change records the frozen `sharing.changed` event
//!   vocabulary through the in-session **world-store seam**
//!   ([`MembersEventLog`]) a later slice wires to the real store (the
//!   save-workflow precedent), and says so honestly in the status
//!   line;
//! - the open/close paths follow the F1 focus contracts: capture the
//!   previously focused surface on open, auto-focus the panel handle
//!   once (the 019 request-once shape), restore on close (the 017
//!   contract) — keyboard focus is never trapped (the d19 lesson);
//! - existing F1 flows are untouched: the module is wired through
//!   minimal `ui.rs` named seams (module declaration, palette rows,
//!   keyboard chord, scoped escape, state field, title-bar entry,
//!   task-surface mount, workspace-root panel mount, navigation close,
//!   the Activity attribution line) — distinct from every prior
//!   wave's seams, each tagged COL-001.
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
    v_flex,
};

use super::WorkspaceView;

gpui::actions!(codexrs, [FlauzMembersShortcut]);

// ---------------------------------------------------------------------------
// Copy registry (user language, WITH consequences — addendum §7)
// ---------------------------------------------------------------------------

/// The title-bar entry tooltip (the chord rides the letter family).
pub(crate) const ENTRY_TOOLTIP: &str = "See who is on this workspace · Ctrl+Alt+Shift+U";
/// The members panel heading.
const PANEL_HEADING: &str = "Who is on this workspace";
/// The one-line description under the heading.
const PANEL_DESCRIPTION: &str =
    "The people with access here, what they can do, and what they're doing";
/// The honest solo empty state's title.
const SOLO_TITLE: &str = "Just you";
/// What the solo state says inviting adds (the empty state's body).
const SOLO_BODY: &str = "Right now this workspace is just you. Inviting someone lets them see \
     the same projects and tasks, work on them with you, and show up here with what they're \
     doing.";
/// The not-wired invite state's title.
const INVITE_NOT_WIRED_TITLE: &str = "Inviting isn't connected yet";
/// The not-wired invite state's body (say so plainly).
const INVITE_NOT_WIRED_BODY: &str = "Invitations arrive when Flauz connects workspaces across \
     devices — nothing is sent today, and no one else can see your work.";
/// The invite affordance's label (rendered with the not-wired state).
const INVITE_LABEL: &str = "Invite someone";
/// The not-wired role-change state's title.
const ROLES_NOT_WIRED_TITLE: &str = "Changing roles isn't connected yet";
/// The not-wired role-change state's body.
const ROLES_NOT_WIRED_BODY: &str = "Roles change here once workspaces connect — everyone \
     invited gets to see the workspace, and roles decide what they can change.";
/// The roster count line when members are wired in.
const MEMBERS_COUNT_LABEL: &str = "{count} people have access to this workspace";
/// The "this is you" marker on the local member's row.
const YOU_MARKER: &str = "You";
/// The close affordance.
const BACK_TO_YOUR_WORK: &str = "Back to your work";
/// The close tooltip.
const BACK_TO_YOUR_WORK_TOOLTIP: &str = "Return to what you were doing (Escape)";
/// The keyboard-hint footer.
const ESCAPE_HINT: &str = "Escape closes this panel";
/// The honest guidance when the sharing palette row fires with no chat
/// selected (the WO-P2-012 pattern: guidance, never a silent no-op).
const SHARING_NO_CHAT_GUIDANCE: &str = "Open a chat to see its sharing.";
/// The per-task sharing entry's label (the task-surface control).
const SHARING_ENTRY_LABEL: &str = "Sharing";
/// The sharing entry's tooltip.
const SHARING_ENTRY_TOOLTIP: &str = "See who this task's files and notes are shared with (Ctrl+Alt+Shift+U opens the members panel)";
/// The sharing panel heading.
const SHARING_PANEL_HEADING: &str = "Sharing";
/// The sharing panel description.
const SHARING_PANEL_DESCRIPTION: &str =
    "What this task shares with the workspace, and what stays yours";
/// The files section heading.
const FILES_HEADING: &str = "Files";
/// The files posture line, isolated (the work order's exact phrase).
const FILES_ISOLATED_LINE: &str = "Isolated copy — your files stay separate";
/// The files posture line, shared read (the work order's exact phrase).
const FILES_SHARED_READ_LINE: &str = "Shared files — everyone on this task can read them";
/// The files posture line, shared write (the work order's exact phrase).
const FILES_SHARED_WRITE_LINE: &str =
    "Shared files — everyone on this task can read and write them";
/// The remembered-notes section heading.
const NOTES_HEADING: &str = "Remembered notes";
/// The notes visibility line, member-private.
const NOTES_ONLY_ME_LINE: &str = "Only me — remembered notes stay visible only to you";
/// The notes visibility line, workspace-shared.
const NOTES_THE_WORKSPACE_LINE: &str =
    "The workspace — everyone here can see what this task remembers";
/// The work-products section heading.
const WORK_PRODUCTS_HEADING: &str = "Work products";
/// The work-products visibility line, member-private.
const WORK_PRODUCTS_ONLY_ME_LINE: &str =
    "Only me — this task's work products stay visible only to you";
/// The work-products visibility line, workspace-shared.
const WORK_PRODUCTS_THE_WORKSPACE_LINE: &str =
    "The workspace — everyone here can see this task's work products";
/// The change affordance's label.
const CHANGE_SHARING_LABEL: &str = "Change sharing";
/// The change affordance's tooltip.
const CHANGE_SHARING_TOOLTIP: &str = "Cycle this task's file sharing and note visibility";
/// The success status after a change (layer 5, WITH the consequence).
const SHARING_CHANGED_STATUS: &str = "Sharing changed — {line}";
/// The honest sync note under a change (the seam-log discipline said
/// plainly).
const SHARING_SYNC_NOTE: &str =
    "This applies while this workspace is open; it syncs to other devices once workspaces connect.";
/// The permission-gated unavailable state (the CAP-001 WHY pattern).
const SHARING_CHANGE_UNAVAILABLE: &str = "You can't change this sharing — your role here is {role}. Ask an admin or the owner to \
     change it.";
/// The command-palette row title (the members panel).
pub(crate) const PALETTE_ROW_TITLE: &str = "See who is on this workspace";
/// The command-palette row description.
pub(crate) const PALETTE_ROW_DESCRIPTION: &str =
    "The people with access here, their roles and what they're doing";
/// The sharing palette row title (the work order's exact row).
pub(crate) const PALETTE_ROW_SHARING_TITLE: &str = "Change a task's sharing";
/// The sharing palette row description.
pub(crate) const PALETTE_ROW_SHARING_DESCRIPTION: &str =
    "See and change what a task's files and notes share with the workspace";
/// The chord label shown in tooltips (the letter family — the verified
/// free letter; the work order's suggested M is the model picker's).
pub(crate) const KEYBOARD_CHORD_LABEL: &str = "Ctrl+Alt+Shift+U";
/// The attribution line's shape (the attention extension).
const ATTENTION_ATTRIBUTION_LINE: &str = "{member} {need}";
/// The default attribution need phrase (the work order's example
/// shape).
const ATTENTION_NEED_REVIEW: &str = "needs your review";

// ---------------------------------------------------------------------------
// The world-store seam (the frozen event vocabulary, re-pinned as
// strings — the save-workflow precedent; the app crate does not import
// flauz-collab)
// ---------------------------------------------------------------------------

/// The event type the world-store seam records when a task's sharing
/// posture changes (the frozen `flauz-collab` event vocabulary).
pub(crate) const SHARING_CHANGED_EVENT_TYPE: &str = "sharing.changed";

/// One world-store seam event: the frozen event vocabulary a later
/// slice wires to the real world store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MembersEvent {
    /// The strictly increasing sequence number.
    pub seq: u64,
    /// The event type (`sharing.changed` today).
    pub event_type: &'static str,
    /// The subject task.
    pub task_id: String,
    /// The posture the event recorded, in user words (the consequence
    /// line).
    pub detail: String,
}

/// The world-store seam: an append-only log of the events the real
/// world store will record. Events are immutable and never mutated or
/// deleted (kernel §3).
#[derive(Debug, Default)]
pub(crate) struct MembersEventLog {
    events: Vec<MembersEvent>,
}

impl MembersEventLog {
    /// An empty event log.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn record(&mut self, event_type: &'static str, task_id: &str, detail: &str) -> MembersEvent {
        let seq = self.events.len() as u64 + 1;
        let event = MembersEvent {
            seq,
            event_type,
            task_id: task_id.to_owned(),
            detail: detail.to_owned(),
        };
        self.events.push(event.clone());
        event
    }

    /// The recorded events, in append order.
    pub(crate) fn events(&self) -> &[MembersEvent] {
        &self.events
    }
}

// ---------------------------------------------------------------------------
// The view-models (plain data; the wiring seams populate them)
// ---------------------------------------------------------------------------

/// One member's role, in user words (the frozen collaboration roles
/// re-pinned as view data).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // constructed by the module's tests and (Wave-later) the roster wiring
pub(crate) enum RoleView {
    /// The workspace's accountable owner.
    Owner,
    /// Manages the workspace and its members.
    Admin,
    /// Works on tasks and environments.
    Contributor,
    /// Sees the workspace, changes nothing.
    Viewer,
}

impl RoleView {
    /// The user-facing label.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Admin => "Admin",
            Self::Contributor => "Contributor",
            Self::Viewer => "Viewer",
        }
    }

    /// The role's consequence line (what the role may do, in user
    /// words).
    pub(crate) const fn consequence_line(self) -> &'static str {
        match self {
            Self::Owner => "Manages everything, including members and sharing",
            Self::Admin => "Manages the workspace, its members and its sharing",
            Self::Contributor => "Works on tasks and environments; sees whose account work uses",
            Self::Viewer => "Sees the workspace; changes nothing",
        }
    }
}

/// One member's presence, in user words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PresenceView {
    /// The member is viewing this task.
    Task,
    /// The member is elsewhere in the workspace.
    Workspace,
    /// The member was here recently (the record went stale — said
    /// plainly, never "stale").
    Recently,
}

impl PresenceView {
    /// The presence line for a member with this presence ("Dev is
    /// viewing this task").
    pub(crate) fn line(self, name: &str) -> String {
        match self {
            Self::Task => format!("{name} is viewing this task"),
            Self::Workspace => format!("{name} is in the workspace"),
            Self::Recently => format!("{name} was here recently"),
        }
    }
}

/// One member row of the members panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MemberView {
    /// The member's display name.
    pub display_name: String,
    /// The member's role.
    pub role: RoleView,
    /// The member's presence, when wired.
    pub presence: Option<PresenceView>,
    /// Whether this row is the local user.
    pub is_you: bool,
}

/// The roster view-model: what the members panel renders. Plain owned
/// data — a later wave populates it from the collaboration contracts;
/// this module never invents members.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MembersView {
    /// The member rows, in roster order.
    pub members: Vec<MemberView>,
}

impl MembersView {
    /// The roster count line ("2 people have access to this
    /// workspace").
    pub(crate) fn count_line(&self) -> String {
        MEMBERS_COUNT_LABEL.replace("{count}", &self.members.len().to_string())
    }

    /// Whether the roster is the honest solo state.
    #[must_use]
    pub(crate) fn is_solo(&self) -> bool {
        self.members.iter().all(|member| member.is_you) || self.members.is_empty()
    }
}

/// One task's file-sharing mode, in user words (the worktree posture).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SharingFileMode {
    /// Isolated copies (the platform default).
    Isolated,
    /// One shared filesystem, everyone on the task can read it.
    SharedRead,
    /// One shared filesystem, everyone can read and write it.
    SharedWrite,
}

impl SharingFileMode {
    /// The posture line WITH its consequence (the work order's exact
    /// phrases).
    pub(crate) const fn consequence_line(self) -> &'static str {
        match self {
            Self::Isolated => FILES_ISOLATED_LINE,
            Self::SharedRead => FILES_SHARED_READ_LINE,
            Self::SharedWrite => FILES_SHARED_WRITE_LINE,
        }
    }

    /// The next mode in the change cycle: isolated → shared read →
    /// shared write → isolated.
    pub(crate) const fn next(self) -> Self {
        match self {
            Self::Isolated => Self::SharedRead,
            Self::SharedRead => Self::SharedWrite,
            Self::SharedWrite => Self::Isolated,
        }
    }
}

/// One family's visibility, in user words ("Only me" / "The
/// workspace").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SharingVisibility {
    /// Visible to its owner alone.
    OnlyMe,
    /// Visible to every member of the workspace.
    TheWorkspace,
}

impl SharingVisibility {
    /// The visibility label (the work order's exact words).
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::OnlyMe => "Only me",
            Self::TheWorkspace => "The workspace",
        }
    }

    /// The toggled visibility.
    pub(crate) const fn toggled(self) -> Self {
        match self {
            Self::OnlyMe => Self::TheWorkspace,
            Self::TheWorkspace => Self::OnlyMe,
        }
    }
}

/// One task's sharing posture: what the per-task sharing surface
/// renders. Plain owned data — the honest platform default is
/// constructed directly (isolated files, "Only me" notes, workspace
/// work products — true today); a later wave wires the live record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SharingView {
    /// The task's file-sharing mode.
    pub files: SharingFileMode,
    /// The remembered-notes visibility.
    pub notes: SharingVisibility,
    /// The work-products visibility.
    pub work_products: SharingVisibility,
    /// Whether the local member may change this sharing (the
    /// permission gate; the CAP-001 WHY renders when false).
    pub can_change: bool,
    /// Why the change is unavailable, when it is (the named WHY).
    pub why_not: Option<String>,
}

impl SharingView {
    /// The honest platform default posture (true today, nothing
    /// invented): isolated files, "Only me" notes, workspace work
    /// products — changeable locally.
    #[must_use]
    pub(crate) fn platform_default() -> Self {
        Self {
            files: SharingFileMode::Isolated,
            notes: SharingVisibility::OnlyMe,
            work_products: SharingVisibility::TheWorkspace,
            can_change: true,
            why_not: None,
        }
    }

    /// The permission-gated unavailable WHY (the named reason, in user
    /// words).
    #[must_use]
    pub(crate) fn why_not_for_role(role: RoleView) -> String {
        SHARING_CHANGE_UNAVAILABLE.replace("{role}", role.label())
    }

    /// The notes visibility line WITH its consequence.
    #[must_use]
    pub(crate) const fn notes_line(&self) -> &'static str {
        match self.notes {
            SharingVisibility::OnlyMe => NOTES_ONLY_ME_LINE,
            SharingVisibility::TheWorkspace => NOTES_THE_WORKSPACE_LINE,
        }
    }

    /// The work-products visibility line WITH its consequence.
    #[must_use]
    pub(crate) const fn work_products_line(&self) -> &'static str {
        match self.work_products {
            SharingVisibility::OnlyMe => WORK_PRODUCTS_ONLY_ME_LINE,
            SharingVisibility::TheWorkspace => WORK_PRODUCTS_THE_WORKSPACE_LINE,
        }
    }
}

/// One collaborator attribution row for the EXISTING attention model
/// (the additive extension, addendum §5): which member needs what from
/// the local user, on which chat. The rows are additive data held by
/// this module; the existing Activity rows render them as one
/// additional line — there is no second store and no parallel feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AttentionNeed {
    /// The collaborator's display name ("Ana").
    pub member: String,
    /// The chat the need is about.
    pub task_id: String,
    /// The need, phrased for the line ("needs your review").
    pub need: String,
}

// ---------------------------------------------------------------------------
// The additive surface state
// ---------------------------------------------------------------------------

/// The additive members state: the panel bookkeeping, the view-model
/// seams, the per-task sharing postures, the attribution rows, and the
/// world-store seam.
pub(crate) struct MembersState {
    /// Whether the workspace-level members panel is open.
    members_open: bool,
    /// Whether the per-task sharing panel is open.
    sharing_open: bool,
    /// The roster view-model seam (None → the honest solo state).
    view: Option<MembersView>,
    /// The per-task sharing postures (a task without an entry renders
    /// the honest platform default).
    sharing: HashMap<String, SharingView>,
    /// The collaborator attribution rows (the attention extension's
    /// additive data).
    attention_needs: Vec<AttentionNeed>,
    /// The world-store seam (the frozen event vocabulary a later slice
    /// wires to the real store).
    event_log: MembersEventLog,
    /// The members panel's keyboard focus handle (the 019
    /// request-once shape).
    panel_focus: FocusHandle,
    /// Request-once guard for `panel_focus`.
    panel_focus_requested: bool,
    /// The surface that held focus when a panel opened, restored on
    /// close (the 017 close contract).
    focus_before_panel: Option<FocusHandle>,
    /// The last sharing-change status (the success state, shown until
    /// the panel closes).
    sharing_status: Option<String>,
}

impl MembersState {
    /// Builds the closed, empty members state (the honest solo state —
    /// no roster wired, nothing invented).
    pub(crate) fn new(cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            members_open: false,
            sharing_open: false,
            view: None,
            sharing: HashMap::new(),
            attention_needs: Vec::new(),
            event_log: MembersEventLog::new(),
            panel_focus: cx.focus_handle(),
            panel_focus_requested: false,
            focus_before_panel: None,
            sharing_status: None,
        }
    }

    /// Whether the workspace-level members panel is open (the
    /// workspace-root mount's visibility rule).
    pub(crate) fn members_open(&self) -> bool {
        self.members_open
    }

    /// The world-store seam's recorded events, in append order.
    #[allow(dead_code)] // read by the module's tests; the Wave-later world-store wiring surfaces them
    pub(crate) fn events(&self) -> &[MembersEvent] {
        self.event_log.events()
    }

    /// The roster view-model, when one has been wired in.
    #[allow(dead_code)] // the Wave-later roster wiring reads it; the render paths check `view` directly today
    pub(crate) fn view(&self) -> Option<&MembersView> {
        self.view.as_ref()
    }

    /// The selected task's sharing posture: the wired record, or the
    /// honest platform default.
    fn sharing_for(&self, task_id: &str) -> SharingView {
        self.sharing
            .get(task_id)
            .cloned()
            .unwrap_or_else(SharingView::platform_default)
    }

    /// Quietly closes both panels for an F1 navigation action (the
    /// deliberate close paths restore focus through the 017 contract
    /// instead). Returns whether anything changed.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.members_open || self.sharing_open;
        self.members_open = false;
        self.sharing_open = false;
        self.panel_focus_requested = false;
        self.focus_before_panel = None;
        self.sharing_status = None;
        changed
    }
}

// ---------------------------------------------------------------------------
// The surface actions
// ---------------------------------------------------------------------------

/// Attaches (or clears) the roster view-model — the wiring seam a later
/// wave drives from the collaboration contracts. Clearing the view
/// returns the panel to the honest solo state.
#[allow(dead_code)] // the Wave-later roster wiring calls it; the module's tests pin the behavior
pub(crate) fn set_members_view(
    workspace: &mut WorkspaceView,
    view: Option<MembersView>,
    cx: &mut Context<WorkspaceView>,
) {
    workspace.flauz_members.view = view;
    cx.notify();
}

/// Attaches (or clears) one task's sharing posture — the wiring seam.
#[allow(dead_code)] // the Wave-later sharing wiring calls it; the module's tests pin the behavior
pub(crate) fn set_sharing_view(
    workspace: &mut WorkspaceView,
    task_id: &str,
    view: Option<SharingView>,
    cx: &mut Context<WorkspaceView>,
) {
    match view {
        Some(view) => {
            workspace
                .flauz_members
                .sharing
                .insert(task_id.to_owned(), view);
        }
        None => {
            workspace.flauz_members.sharing.remove(task_id);
        }
    }
    cx.notify();
}

/// Records collaborator attribution rows — the attention extension's
/// wiring seam (additive data on the EXISTING attention model; never a
/// second store).
#[allow(dead_code)] // the Wave-later collaboration wiring calls it; the module's tests pin the behavior
pub(crate) fn set_attention_needs(
    workspace: &mut WorkspaceView,
    needs: Vec<AttentionNeed>,
    cx: &mut Context<WorkspaceView>,
) {
    workspace.flauz_members.attention_needs = needs;
    cx.notify();
}

/// The attribution line for one chat's Activity row (the additive
/// render seam the existing row renderer calls): "Ana needs your
/// review", or `None` when no collaborator need is wired for the chat.
/// The rows stay data here — the attention flags themselves live in the
/// existing session state, exactly where they lived before.
pub(crate) fn attention_attribution_line(state: &MembersState, task_id: &str) -> Option<String> {
    attribution_line(&state.attention_needs, task_id)
}

/// Builds the attribution line from the need rows (the pure path the
/// seam and the module's tests share).
fn attribution_line(needs: &[AttentionNeed], task_id: &str) -> Option<String> {
    needs
        .iter()
        .find(|need| need.task_id == task_id)
        .map(|need| {
            ATTENTION_ATTRIBUTION_LINE
                .replace("{member}", &need.member)
                .replace("{need}", &need.need)
        })
}

/// Opens the members panel (the chord, the palette row and the
/// title-bar button all land here). Toggles closed when already open.
pub(crate) fn open_members_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.flauz_members.members_open {
        close_members_panel(workspace, window, cx);
        return;
    }
    let state = &mut workspace.flauz_members;
    if state.focus_before_panel.is_none() {
        state.focus_before_panel = window.focused(cx);
    }
    state.members_open = true;
    state.panel_focus_requested = true;
    cx.notify();
}

/// Closes the members panel and restores focus (the 017 close
/// contract).
pub(crate) fn close_members_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if !workspace.flauz_members.members_open {
        return;
    }
    workspace.flauz_members.members_open = false;
    workspace.flauz_members.panel_focus_requested = false;
    let previous = workspace.flauz_members.focus_before_panel.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// Opens the per-task sharing surface (the task-surface control and the
/// palette row land here). With no chat selected, surfaces honest
/// guidance instead of a silent no-op (the WO-P2-012 pattern).
pub(crate) fn open_sharing_surface(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.state.selected_task_id.is_none() {
        workspace.dispatch_command_status(Some(SHARING_NO_CHAT_GUIDANCE), cx);
        return;
    }
    if workspace.flauz_members.sharing_open {
        close_sharing_surface(workspace, window, cx);
        return;
    }
    // The panel lives on the task surface: navigate there first when
    // needed, so it is reachable from any route.
    if workspace.state.route != codex_core::MainRoute::Tasks {
        workspace.navigate(codex_core::MainRoute::Tasks, cx);
    }
    // One panel at a time: opening the sharing surface closes any open
    // rail/shell panel (the save-flow pattern).
    super::flauz_shell::dismiss_shell_surfaces(workspace, window, cx);
    let state = &mut workspace.flauz_members;
    if state.focus_before_panel.is_none() {
        state.focus_before_panel = window.focused(cx);
    }
    state.sharing_open = true;
    state.panel_focus_requested = true;
    cx.notify();
}

/// Closes the per-task sharing surface and restores focus.
pub(crate) fn close_sharing_surface(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if !workspace.flauz_members.sharing_open {
        return;
    }
    workspace.flauz_members.sharing_open = false;
    workspace.flauz_members.panel_focus_requested = false;
    workspace.flauz_members.sharing_status = None;
    let previous = workspace.flauz_members.focus_before_panel.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// Cycles the selected task's file sharing (isolated → shared read →
/// shared write → isolated) through the world-store seam: the posture
/// changes in-session, the frozen `sharing.changed` event is recorded,
/// and the status line states the new posture WITH its consequence.
/// A permission-gated member gets the honest WHY in the panel instead
/// (the CAP-001 pattern — never a silent no-op).
pub(crate) fn cycle_file_sharing(
    workspace: &mut WorkspaceView,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let Some(task_id) = workspace.state.selected_task_id.clone() else {
        return;
    };
    let mut posture = workspace.flauz_members.sharing_for(&task_id);
    if !posture.can_change {
        // The affordance never renders in this state (the WHY panel
        // replaces it); this is the defensive path — it surfaces the
        // WHY as the panel status, never a silent drop.
        let why = posture.why_not.clone().unwrap_or_default();
        workspace.flauz_members.sharing_status = Some(why);
        cx.notify();
        return;
    }
    posture.files = posture.files.next();
    let line = posture.files.consequence_line();
    workspace
        .flauz_members
        .event_log
        .record(SHARING_CHANGED_EVENT_TYPE, &task_id, line);
    workspace
        .flauz_members
        .sharing
        .insert(task_id.clone(), posture);
    workspace.flauz_members.sharing_status = Some(SHARING_CHANGED_STATUS.replace("{line}", line));
    cx.notify();
}

/// Toggles the selected task's remembered-notes visibility ("Only me"
/// ↔ "The workspace") through the same seam, with the same honesty.
pub(crate) fn toggle_notes_visibility(
    workspace: &mut WorkspaceView,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let Some(task_id) = workspace.state.selected_task_id.clone() else {
        return;
    };
    let mut posture = workspace.flauz_members.sharing_for(&task_id);
    if !posture.can_change {
        let why = posture.why_not.clone().unwrap_or_default();
        workspace.flauz_members.sharing_status = Some(why);
        cx.notify();
        return;
    }
    posture.notes = posture.notes.toggled();
    let line = posture.notes_line();
    workspace
        .flauz_members
        .event_log
        .record(SHARING_CHANGED_EVENT_TYPE, &task_id, line);
    workspace
        .flauz_members
        .sharing
        .insert(task_id.clone(), posture);
    workspace.flauz_members.sharing_status = Some(SHARING_CHANGED_STATUS.replace("{line}", line));
    cx.notify();
}

// ---------------------------------------------------------------------------
// The renders
// ---------------------------------------------------------------------------

/// Renders the members button in the title bar (layer 1's visible
/// primary entry — the WO-P2-018 bell precedent).
pub(crate) fn render_members_entry_button(
    workspace: &WorkspaceView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open = workspace.flauz_members.members_open;
    Button::new("flauz-members-entry")
        .icon(IconName::User)
        .tooltip(ENTRY_TOOLTIP)
        .xsmall()
        .w(px(28.0))
        .h(px(28.0))
        .ghost()
        .selected(open)
        .on_click(cx.listener(|this, _, window, cx| {
            open_members_panel(this, window, cx);
        }))
        .into_any_element()
}

/// Renders the per-task sharing entry + panel on the task surface (the
/// picker/save-family neighborhood): a labeled Sharing control, the
/// open panel below it.
pub(crate) fn render_task_sharing_entry(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open = workspace.flauz_members.sharing_open;
    if workspace.flauz_members.panel_focus_requested {
        // The 019 request-once shape: whichever panel is open claims
        // the keyboard on mount so the scoped Escape binding reaches
        // it.
        workspace.flauz_members.panel_focus_requested = false;
        workspace.flauz_members.panel_focus.focus(window);
    }
    let mut entry = v_flex().flex_none();
    entry = entry.child(
        h_flex()
            .h(px(32.0))
            .px_5()
            .items_center()
            .child(
                Button::new("flauz-task-sharing-entry")
                    .label(SHARING_ENTRY_LABEL)
                    .icon(IconName::FolderOpen)
                    .tooltip(SHARING_ENTRY_TOOLTIP)
                    .small()
                    .ghost()
                    .selected(open)
                    .on_click(cx.listener(|this, _, window, cx| {
                        open_sharing_surface(this, window, cx);
                    })),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(SHARING_PANEL_DESCRIPTION),
            ),
    );
    if open {
        let Some(task_id) = workspace.state.selected_task_id.clone() else {
            return entry.into_any_element();
        };
        let posture = workspace.flauz_members.sharing_for(&task_id);
        let status = workspace.flauz_members.sharing_status.clone();
        let panel_focus = workspace.flauz_members.panel_focus.clone();
        entry = entry.child(render_sharing_panel(
            panel_focus,
            &posture,
            status.as_deref(),
            cx,
        ));
    }
    entry.into_any_element()
}

/// Renders the sharing panel: the posture lines WITH consequences, the
/// change affordance (or the permission-gated WHY), the success status,
/// and the honest sync note.
fn render_sharing_panel(
    panel_focus: FocusHandle,
    posture: &SharingView,
    status: Option<&str>,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let mut panel = v_flex()
        .key_context("FlauzMembers")
        .track_focus(&panel_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            close_sharing_surface(this, window, cx);
        }))
        .mx_5()
        .my_2()
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
                .justify_between()
                .child(
                    v_flex().gap_1().child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(SHARING_PANEL_HEADING),
                    ),
                )
                .child(
                    Button::new("flauz-sharing-close")
                        .label(BACK_TO_YOUR_WORK)
                        .icon(IconName::ArrowLeft)
                        .tooltip(BACK_TO_YOUR_WORK_TOOLTIP)
                        .small()
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            close_sharing_surface(this, window, cx);
                        })),
                ),
        )
        // The files posture line (layer 2's contextual affordance is
        // the disclosure of what each family shares).
        .child(render_posture_row(
            FILES_HEADING,
            posture.files.consequence_line(),
            cx,
        ))
        .child(render_posture_row(NOTES_HEADING, posture.notes_line(), cx))
        .child(render_posture_row(
            WORK_PRODUCTS_HEADING,
            posture.work_products_line(),
            cx,
        ));
    // The change affordance, or the permission-gated honest WHY.
    if posture.can_change {
        panel = panel
            .child(
                h_flex()
                    .gap_2()
                    .pt_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        Button::new("flauz-sharing-cycle-files")
                            .label(CHANGE_SHARING_LABEL)
                            .icon(IconName::FolderOpen)
                            .tooltip(CHANGE_SHARING_TOOLTIP)
                            .small()
                            .primary()
                            .on_click(cx.listener(|this, _, window, cx| {
                                cycle_file_sharing(this, window, cx);
                            })),
                    )
                    .child(
                        Button::new("flauz-sharing-toggle-notes")
                            .label(format!("Notes: {}", posture.notes.label()))
                            .icon(IconName::Info)
                            .tooltip("Switch remembered notes between Only me and The workspace")
                            .small()
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                toggle_notes_visibility(this, window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(SHARING_SYNC_NOTE),
            );
    } else {
        let why = posture.why_not.clone().unwrap_or_default();
        panel = panel.child(
            v_flex()
                .gap_1()
                .pt_2()
                .border_t_1()
                .border_color(cx.theme().border)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(Icon::new(IconName::TriangleAlert).small())
                        .child(div().text_sm().line_height(px(20.0)).child(why)),
                ),
        );
    }
    // The success status (layer 5): the posture restated WITH its
    // consequence.
    if let Some(status) = status {
        panel = panel.child(
            h_flex()
                .gap_2()
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
    panel = panel.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(ESCAPE_HINT),
    );
    panel.into_any_element()
}

/// Renders one posture row: the family heading and the consequence
/// line.
fn render_posture_row(heading: &str, line: &str, cx: &mut Context<WorkspaceView>) -> AnyElement {
    v_flex()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(cx.theme().muted_foreground)
                .child(heading.to_owned()),
        )
        .child(div().text_sm().line_height(px(20.0)).child(line.to_owned()))
        .into_any_element()
}

/// Renders the workspace-level members panel overlay: the member list
/// (name, role, presence in user words), the honest solo empty state,
/// the not-wired invite/role affordances, and the close affordance.
pub(crate) fn render_members_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    if workspace.flauz_members.panel_focus_requested {
        // The 019 request-once shape: the panel claims the keyboard on
        // mount so the scoped Escape binding reaches it.
        workspace.flauz_members.panel_focus_requested = false;
        workspace.flauz_members.panel_focus.focus(window);
    }
    let view = workspace.flauz_members.view.clone();
    let panel_focus = workspace.flauz_members.panel_focus.clone();

    let mut panel = v_flex()
        .key_context("FlauzMembers")
        .track_focus(&panel_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            close_members_panel(this, window, cx);
        }))
        .w(px(super::modal_surface_width(
            workspace.shell_viewport_width,
            480.0,
        )))
        .max_h(px(super::modal_surface_max_height(
            workspace.shell_viewport_height,
            560.0,
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
                    Button::new("flauz-members-close")
                        .label(BACK_TO_YOUR_WORK)
                        .icon(IconName::ArrowLeft)
                        .tooltip(BACK_TO_YOUR_WORK_TOOLTIP)
                        .small()
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            close_members_panel(this, window, cx);
                        })),
                ),
        );
    match view {
        Some(roster) if !roster.is_solo() => {
            // The roster: the count line, then one row per member (name,
            // the "You" marker, role, role consequence, presence line).
            panel = panel.child(
                div()
                    .px_4()
                    .pt_3()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(roster.count_line()),
            );
            let mut list = v_flex().px_4().py_3().gap_2();
            for (index, member) in roster.members.iter().enumerate() {
                list = list.child(render_member_row(index, member, cx));
            }
            panel = panel.child(list).child(render_invite_section(cx));
        }
        _ => {
            // Layer 4: the honest solo state (no roster wired, or only
            // the local user) + what inviting adds.
            panel = panel.child(
                v_flex()
                    .p_4()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(SOLO_TITLE),
                    )
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(20.0))
                            .text_color(cx.theme().muted_foreground)
                            .child(SOLO_BODY),
                    )
                    .child(render_invite_section(cx)),
            );
        }
    }
    panel = panel.child(
        div()
            .px_4()
            .py_3()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(ESCAPE_HINT),
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
            close_members_panel(this, window, cx);
        }))
        .on_any_mouse_down(
            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                cx.stop_propagation();
                if event.button == gpui::MouseButton::Left {
                    this.flauz_members.members_open = false;
                    this.flauz_members.panel_focus_requested = false;
                    cx.notify();
                }
            }),
        )
        .child(panel)
        .into_any_element()
}

/// Renders one member row: the name (with the "You" marker), the role
/// chip + consequence, and the presence line in user words.
fn render_member_row(
    index: usize,
    member: &MemberView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let mut row = v_flex().gap_1();
    let mut title = h_flex().gap_2().items_center();
    title = title.child(
        div()
            .text_sm()
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(member.display_name.clone()),
    );
    if member.is_you {
        title = title.child(
            div()
                .text_xs()
                .px_2()
                .rounded_sm()
                .border_1()
                .border_color(cx.theme().border)
                .text_color(cx.theme().muted_foreground)
                .child(YOU_MARKER),
        );
    }
    title = title.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(member.role.label()),
    );
    row = row.child(title).child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(member.role.consequence_line()),
    );
    if let Some(presence) = member.presence {
        row = row.child(
            h_flex()
                .gap_1()
                .items_center()
                .child(Icon::new(IconName::User).xsmall())
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .child(presence.line(&member.display_name)),
                ),
        );
    }
    v_flex()
        .id(SharedString::from(format!("flauz-member-row-{index}")))
        .p_3()
        .gap_1()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(row)
        .into_any_element()
}

/// Renders the invite/role section with the honest not-wired states
/// (the real transport arrives with connected workspaces — say so
/// plainly; nothing is sent today).
fn render_invite_section(cx: &mut Context<WorkspaceView>) -> AnyElement {
    v_flex()
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
                .child(
                    Button::new("flauz-members-invite")
                        .label(INVITE_LABEL)
                        .icon(IconName::User)
                        .small()
                        .primary()
                        .on_click(cx.listener(|this, _ev, _window, cx| {
                            // The not-wired state says so plainly — the
                            // click surfaces the same honesty through the
                            // status line, never a silent no-op.
                            this.dispatch_command_status(Some(INVITE_NOT_WIRED_BODY), cx);
                        })),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(INVITE_NOT_WIRED_TITLE),
                ),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(INVITE_NOT_WIRED_BODY),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(ROLES_NOT_WIRED_BODY),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_user_copy() -> Vec<String> {
        [
            ENTRY_TOOLTIP,
            PANEL_HEADING,
            PANEL_DESCRIPTION,
            SOLO_TITLE,
            SOLO_BODY,
            INVITE_NOT_WIRED_TITLE,
            INVITE_NOT_WIRED_BODY,
            INVITE_LABEL,
            ROLES_NOT_WIRED_BODY,
            MEMBERS_COUNT_LABEL,
            YOU_MARKER,
            BACK_TO_YOUR_WORK,
            BACK_TO_YOUR_WORK_TOOLTIP,
            ESCAPE_HINT,
            SHARING_NO_CHAT_GUIDANCE,
            SHARING_ENTRY_LABEL,
            SHARING_ENTRY_TOOLTIP,
            SHARING_PANEL_HEADING,
            SHARING_PANEL_DESCRIPTION,
            FILES_HEADING,
            FILES_ISOLATED_LINE,
            FILES_SHARED_READ_LINE,
            FILES_SHARED_WRITE_LINE,
            NOTES_HEADING,
            NOTES_ONLY_ME_LINE,
            NOTES_THE_WORKSPACE_LINE,
            WORK_PRODUCTS_HEADING,
            WORK_PRODUCTS_ONLY_ME_LINE,
            WORK_PRODUCTS_THE_WORKSPACE_LINE,
            CHANGE_SHARING_LABEL,
            CHANGE_SHARING_TOOLTIP,
            SHARING_CHANGED_STATUS,
            SHARING_SYNC_NOTE,
            SHARING_CHANGE_UNAVAILABLE,
            PALETTE_ROW_TITLE,
            PALETTE_ROW_DESCRIPTION,
            PALETTE_ROW_SHARING_TITLE,
            PALETTE_ROW_SHARING_DESCRIPTION,
            KEYBOARD_CHORD_LABEL,
            "Switch remembered notes between Only me and The workspace",
        ]
        .iter()
        .map(|text| (*text).to_owned())
        .collect()
    }

    /// The copy states consequences and never leaks an implementation
    /// term (addendum §7: user language only — "Dev is viewing this
    /// task", never "presence row for actor ref").
    #[test]
    fn members_copy_uses_user_language_with_consequences() {
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "membership",
                "actor",
                "actorref",
                "presence record",
                "worktree",
                "collab",
                "col-001",
                "flauz-collab",
                "permission grant",
                "lattice",
                "projection",
                "canonical",
                "serde",
                "surface family",
                "access level",
                "event stream",
                "world store",
                "transport",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "members copy must never say {forbidden:?}: {copy:?}"
                );
            }
        }
        // The work order's exact phrases are present, verbatim, WITH
        // their consequences.
        assert_eq!(
            FILES_ISOLATED_LINE,
            "Isolated copy — your files stay separate"
        );
        assert_eq!(
            FILES_SHARED_WRITE_LINE,
            "Shared files — everyone on this task can read and write them"
        );
        assert_eq!(
            FILES_SHARED_READ_LINE,
            "Shared files — everyone on this task can read them"
        );
        assert_eq!(SharingVisibility::OnlyMe.label(), "Only me");
        assert_eq!(SharingVisibility::TheWorkspace.label(), "The workspace");
        assert_eq!(PALETTE_ROW_TITLE, "See who is on this workspace");
        assert_eq!(PALETTE_ROW_SHARING_TITLE, "Change a task's sharing");
        // Presence words: plain, task-shaped.
        assert_eq!(PresenceView::Task.line("Dev"), "Dev is viewing this task");
        assert_eq!(
            PresenceView::Workspace.line("Dev"),
            "Dev is in the workspace"
        );
        assert_eq!(PresenceView::Recently.line("Dev"), "Dev was here recently");
    }

    /// The honest not-wired states say so plainly: nothing is sent
    /// today, and the real arrival is named.
    #[test]
    fn the_not_wired_states_say_so_plainly() {
        assert!(INVITE_NOT_WIRED_TITLE.contains("isn't connected yet"));
        assert!(INVITE_NOT_WIRED_BODY.contains("nothing is sent today"));
        assert!(ROLES_NOT_WIRED_BODY.contains("once workspaces connect"));
        assert!(SHARING_SYNC_NOTE.contains("syncs to other devices once workspaces connect"));
    }

    /// The solo empty state is honest ("Just you") and says what
    /// inviting adds.
    #[test]
    fn the_solo_empty_state_is_honest() {
        assert_eq!(SOLO_TITLE, "Just you");
        assert!(SOLO_BODY.contains("just you"));
        assert!(SOLO_BODY.contains("Inviting someone"));
        let solo = MembersView {
            members: vec![MemberView {
                display_name: "You".to_owned(),
                role: RoleView::Owner,
                presence: None,
                is_you: true,
            }],
        };
        assert!(solo.is_solo());
        let empty = MembersView::default();
        assert!(empty.is_solo());
        let duo = MembersView {
            members: vec![
                MemberView {
                    display_name: "Ana".to_owned(),
                    role: RoleView::Owner,
                    presence: Some(PresenceView::Task),
                    is_you: true,
                },
                MemberView {
                    display_name: "Dev".to_owned(),
                    role: RoleView::Contributor,
                    presence: Some(PresenceView::Workspace),
                    is_you: false,
                },
            ],
        };
        assert!(!duo.is_solo());
        assert_eq!(duo.count_line(), "2 people have access to this workspace");
    }

    /// The sharing posture cycles and toggles deterministically, and
    /// every posture states its consequence.
    #[test]
    fn sharing_postures_cycle_and_state_consequences() {
        assert_eq!(
            SharingFileMode::Isolated.next(),
            SharingFileMode::SharedRead
        );
        assert_eq!(
            SharingFileMode::SharedRead.next(),
            SharingFileMode::SharedWrite
        );
        assert_eq!(
            SharingFileMode::SharedWrite.next(),
            SharingFileMode::Isolated
        );
        assert_eq!(
            SharingVisibility::OnlyMe.toggled(),
            SharingVisibility::TheWorkspace
        );
        assert_eq!(
            SharingVisibility::TheWorkspace.toggled(),
            SharingVisibility::OnlyMe
        );
        let posture = SharingView::platform_default();
        assert_eq!(posture.files, SharingFileMode::Isolated);
        assert_eq!(posture.notes, SharingVisibility::OnlyMe);
        assert_eq!(posture.work_products, SharingVisibility::TheWorkspace);
        assert!(posture.can_change);
        assert_eq!(posture.notes_line(), NOTES_ONLY_ME_LINE);
        assert_eq!(
            posture.work_products_line(),
            WORK_PRODUCTS_THE_WORKSPACE_LINE
        );
        // The permission-gated WHY names the role (the CAP-001
        // pattern).
        let why = SharingView::why_not_for_role(RoleView::Viewer);
        assert_eq!(
            why,
            "You can't change this sharing — your role here is Viewer. Ask an admin or the owner \
             to change it."
        );
    }

    /// The attribution line takes the work order's example shape, and
    /// stays additive data — the attention flags themselves are the
    /// existing session state's, not a second store.
    #[test]
    fn attribution_lines_are_collaborator_named() {
        assert_eq!(attribution_line(&[], "chat-1"), None);
        let needs = vec![AttentionNeed {
            member: "Ana".to_owned(),
            task_id: "chat-1".to_owned(),
            need: ATTENTION_NEED_REVIEW.to_owned(),
        }];
        assert_eq!(
            attribution_line(&needs, "chat-1").as_deref(),
            Some("Ana needs your review")
        );
        assert_eq!(attribution_line(&needs, "chat-2"), None);
    }

    /// The world-store seam records the frozen `sharing.changed`
    /// vocabulary, append-only, with the consequence line as the
    /// detail.
    #[test]
    fn the_seam_records_the_frozen_vocabulary() {
        assert_eq!(SHARING_CHANGED_EVENT_TYPE, "sharing.changed");
        let mut log = MembersEventLog::new();
        assert!(log.events().is_empty());
        log.record(SHARING_CHANGED_EVENT_TYPE, "chat-1", FILES_SHARED_READ_LINE);
        log.record(SHARING_CHANGED_EVENT_TYPE, "chat-1", FILES_ISOLATED_LINE);
        let events = log.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[1].seq, 2);
        assert_eq!(events[0].event_type, "sharing.changed");
        assert_eq!(events[0].detail, FILES_SHARED_READ_LINE);
        assert_eq!(events[1].detail, FILES_ISOLATED_LINE);
    }

    /// Layer 3: the palette fallback finds both rows through natural
    /// queries.
    #[test]
    fn palette_queries_resolve_to_the_rows() {
        for query in ["who", "workspace", "people", "access", "here"] {
            let title = PALETTE_ROW_TITLE.to_lowercase();
            let description = PALETTE_ROW_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the members row"
            );
        }
        for query in ["sharing", "share", "files", "notes", "task"] {
            let title = PALETTE_ROW_SHARING_TITLE.to_lowercase();
            let description = PALETTE_ROW_SHARING_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the sharing row"
            );
        }
    }

    /// The COL-001 registration seams in `ui.rs` (the house
    /// source-inspection style): the module declaration, the palette
    /// rows, the keyboard chord, the scoped escape, the state field,
    /// the title-bar entry, the task-surface mount, the
    /// workspace-root panel mount, the navigation close, and the
    /// Activity attribution line — each tagged COL-001, distinct from
    /// every prior wave's seams.
    #[test]
    fn members_seams_are_registered_in_the_ui_seams() {
        let source = include_str!("../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_members;"));
        assert!(source.contains("use flauz_members::FlauzMembersShortcut;"));

        // The palette registration seams (both rows).
        assert!(source.contains("PaletteCommand::SeeWhoIsOnWorkspace"));
        assert!(source.contains("PaletteCommand::ChangeTaskSharing"));
        assert!(source.contains("flauz_members::PALETTE_ROW_TITLE"));
        assert!(source.contains("flauz_members::PALETTE_ROW_SHARING_TITLE"));
        assert!(source.contains("flauz_members::open_members_panel(workspace, window, cx)"));
        assert!(source.contains("flauz_members::open_sharing_surface(workspace, window, cx)"));

        // The keyboard chord seam (Ctrl+Alt+Shift+U — the verified free
        // letter; the work order's suggested M is the model picker's,
        // verified taken at ui.rs's ChooseModelForTask arm).
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzMembersShortcut, None"),
            "the members chord must be bound"
        );
        assert!(source.contains("Some(\"Ctrl+Alt+Shift+U\")"));
        // The chord-conflict verification: the members chord is unique
        // in the palette registry, and the work order's suggested M is
        // taken by the model picker (verified — the conflict that moved
        // this surface to U).
        assert_eq!(
            source.matches("Some(\"Ctrl+Alt+Shift+U\")").count(),
            1,
            "the members chord must be unique in the palette registry"
        );
        assert!(source.contains("Some(\"Ctrl+Alt+Shift+M\")"));

        // THE D25 LESSON (Gate B, the law): a KeyBinding without an
        // `.on_action` listener dispatches into the void — the chord
        // silently no-ops while the palette row works. The listener
        // must be registered on the workspace root next to the
        // picker/gap/agents/save/recovery chord listeners.
        let listener_form: String = "cx.listener(|this, _: &FlauzMembersShortcut, window, cx| { \
             flauz_members::open_members_panel(this, window, cx); })"
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            normalized.contains(&listener_form),
            "the members chord must have an on_action listener that calls \
             open_members_panel (the d25 Gate-B lesson — the seam test pins the LISTENER, not \
             just the KeyBinding)"
        );

        // The binding registration itself (the alt-shift-u chord, the
        // letter family).
        assert!(source.contains("shortcut(\"alt-shift-u\")"));

        // The scoped escape seam (the d19 discipline: one Escape
        // through the panel's own focus context, never a trap).
        assert!(source.contains("Some(\"FlauzMembers\")"));

        // The state-field seam.
        assert!(source.contains("flauz_members: flauz_members::MembersState"));
        assert!(source.contains("flauz_members::MembersState::new(cx)"));

        // The title-bar entry seam (layer 1's visible primary entry).
        let joined: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            joined.contains("flauz_members::render_members_entry_button("),
            "the members button must be mounted in the title bar"
        );

        // The task-surface mount seam (the per-task sharing entry).
        assert!(
            joined.contains("flauz_members::render_task_sharing_entry("),
            "the sharing entry must be mounted on the task surface"
        );

        // The workspace-root panel mount seam (the members overlay).
        assert!(
            joined.contains("flauz_members::render_members_panel("),
            "the members panel must be mounted at the workspace root"
        );

        // The navigation-close seam.
        assert!(source.contains("self.flauz_members.close_for_navigation()"));

        // The attention-extension seam (addendum §5): the EXISTING
        // Activity row renderer draws the attribution line through this
        // module's additive data.
        assert!(
            source.contains("flauz_members::attention_attribution_line("),
            "the existing Activity rows must render the attribution line"
        );

        // Every COL-001 seam is tagged.
        let seam_tags = source.matches("COL-001").count();
        assert!(
            seam_tags >= 10,
            "each seam is tagged COL-001 (found {seam_tags})"
        );
    }

    /// The attention extension is ADDITIVE-ONLY (addendum §5 — the
    /// no-second-store law, source-inspected): the existing Activity
    /// rows still derive from the ONE attention list, the attribution
    /// is one extra line drawn inside the existing row renderer, and
    /// this module holds no event store of its own beyond the frozen
    /// world-store seam vocabulary.
    #[test]
    fn the_attention_extension_is_additive_only() {
        let ui_source = include_str!("../ui.rs");
        // The existing rows still come from the same single attention
        // list (needs_attention_task_ids) — the one store.
        assert!(ui_source.contains("fn activity_view_rows(state: &AppState) -> Vec<TaskSummary>"));
        assert!(ui_source.contains("needs_attention_task_ids"));
        // The attribution line renders INSIDE the existing Activity row
        // renderer — additive render line, not a parallel feed.
        assert!(ui_source.contains("render_activity_view_row"));
        let this_source = include_str!("flauz_members.rs");
        // This module's attribution storage is a plain replaceable vec
        // (set wholesale by the wiring seam) — never an append-only
        // event log.
        assert!(this_source.contains("attention_needs: Vec<AttentionNeed>"));
        assert!(this_source.contains("workspace.flauz_members.attention_needs = needs;"));
        // The only append-only log here is the world-store seam, and it
        // records ONLY the sharing vocabulary (never attention rows) —
        // the forbidden name is built with concat! so this assertion
        // cannot match its own source.
        assert!(this_source.contains("SHARING_CHANGED_EVENT_TYPE"));
        assert!(!this_source.contains(concat!("ATTENTION_", "CHANGED_EVENT_TYPE")));
    }

    /// The d19 discipline, made checkable: the panels' focus paths
    /// follow the request-once + restore contract.
    #[test]
    fn members_focus_never_traps() {
        let source = include_str!("flauz_members.rs");
        // The request-once shape: focus is requested once and consumed
        // by the render.
        assert!(source.contains("panel_focus_requested = true"));
        assert!(source.contains("panel_focus_requested = false"));
        // The 017 close contract: the previously focused surface is
        // captured and restored on close.
        assert!(source.contains("focus_before_panel = window.focused(cx)"));
        assert!(source.contains("apply_overlay_close_focus_restore"));
        // The panels take focus ONLY through the explicit open path (no
        // focus steal on mount beyond the request-once).
        assert!(source.contains(".focus(window)"));
    }
}
