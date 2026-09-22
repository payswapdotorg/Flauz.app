//! The first Flauz platform shell — GUI module (work order UX-001, F2 Wave 1).
//!
//! This module implements the target GUI information architecture of
//! PRODUCT-UX-JOURNEYS §2.1 on top of the existing app:
//!
//! - **Workspace navigation** (persistent, labeled): Projects & tasks,
//!   Reusable workflows, Artifacts, Activity — registered into the sidebar
//!   and reachable by pointer, palette, and direct keyboard chords.
//! - **Task rail** (visible while a task is selected): Context, Agents,
//!   Environments, Evidence, More — labeled controls on the task surface,
//!   never hidden behind developer settings.
//! - **Honest empty states**: every not-yet-implemented surface explains
//!   what the feature will do and what to do next; controls are never
//!   hidden and functionality is never faked.
//! - **User-facing language**: the platform concept "Procedure" is always
//!   called a *reusable workflow* here; internal type names never appear
//!   in UI copy.
//!
//! Boundary rules (work order UX-001 / kernel §1):
//!
//! - the shell does NOT import the flauz contract crates in this wave —
//!   surfaces are honest empty states until later slices wire them;
//! - existing F1 flows are untouched: the module is wired through minimal
//!   `ui.rs` seams (module declaration, navigation/palette registration,
//!   keyboard chords); the F1 focus contracts (017 palette close restore,
//!   019 modal focus traps, N1 composer focus) are followed by the shell's
//!   own open/close paths: the shell captures the previously focused
//!   surface on open, auto-focuses its own handle once (the 019
//!   request-once shape), and restores on close (the 017 contract).
//!
//! Every user-facing string lives in the copy registry below so the
//! language rules are unit-testable in one place.

use codex_core::{Action, MainRoute};
use gpui::prelude::*;
use gpui::{AnyElement, App, Context, FocusHandle, IntoElement, SharedString, Window, div, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::Escape,
    scroll::ScrollableElement,
    v_flex,
};

use super::WorkspaceView;

gpui::actions!(
    codexrs,
    [
        FlauzProjectsTasksShortcut,
        FlauzReusableWorkflowsShortcut,
        FlauzArtifactsShortcut,
        FlauzActivityShortcut,
        FlauzTaskContextShortcut,
        FlauzTaskAgentsShortcut,
        FlauzTaskEnvironmentsShortcut,
        FlauzTaskEvidenceShortcut,
        FlauzTaskMoreInspectShortcut
    ]
);

/// A workspace navigation surface (PRODUCT-UX-JOURNEYS §2.1 "Workspace
/// navigation"). Persistent in the sidebar, in the command palette, and on
/// direct keyboard chords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlauzWorkspaceSurface {
    /// Projects and their tasks — the durable home of ongoing work.
    ProjectsTasks,
    /// The library of saved reusable workflows (the platform concept
    /// "Procedure"; never named that in UI copy).
    ReusableWorkflows,
    /// The work products tasks produced.
    Artifacts,
    /// What needs attention across all work.
    Activity,
}

impl FlauzWorkspaceSurface {
    /// Every workspace surface, in navigation order.
    pub(crate) const ALL: [Self; 4] = [
        Self::ProjectsTasks,
        Self::ReusableWorkflows,
        Self::Artifacts,
        Self::Activity,
    ];

    /// The sidebar navigation label.
    pub(crate) const fn nav_label(self) -> &'static str {
        match self {
            Self::ProjectsTasks => "Projects & tasks",
            Self::ReusableWorkflows => "Reusable workflows",
            Self::Artifacts => "Artifacts",
            Self::Activity => "Activity",
        }
    }

    /// The navigation tooltip.
    pub(crate) const fn nav_tooltip(self) -> &'static str {
        match self {
            Self::ProjectsTasks => "See your projects and their tasks in the workspace",
            Self::ReusableWorkflows => "Saved steps you can run again",
            Self::Artifacts => "The work products your tasks produced",
            Self::Activity => "What needs your attention across all your work",
        }
    }

    /// The surface heading.
    pub(crate) const fn heading(self) -> &'static str {
        self.nav_label()
    }

    /// The one-line description under the heading.
    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::ProjectsTasks => "Your projects, their tasks, and everything attached to them",
            Self::ReusableWorkflows => "Successful steps, saved so you can run them again",
            Self::Artifacts => "Durable work products, attached to the tasks that made them",
            Self::Activity => "Everything that needs you, in one place",
        }
    }

    /// The command-palette row title.
    pub(crate) const fn palette_title(self) -> &'static str {
        match self {
            Self::ProjectsTasks => "Projects & tasks",
            Self::ReusableWorkflows => "Reusable workflows",
            Self::Artifacts => "Artifacts",
            Self::Activity => "Activity",
        }
    }

    /// The command-palette row description.
    pub(crate) const fn palette_description(self) -> &'static str {
        match self {
            Self::ProjectsTasks => "Open the workspace projects and tasks view",
            Self::ReusableWorkflows => "Browse and run the workflows you saved",
            Self::Artifacts => "Browse the work products from your tasks",
            Self::Activity => "See active work and what needs your attention",
        }
    }

    /// The honest empty-state title.
    pub(crate) const fn empty_title(self) -> &'static str {
        match self {
            Self::ProjectsTasks => "The workspace projects view is on its way",
            Self::ReusableWorkflows => "No reusable workflows yet",
            Self::Artifacts => "The workspace artifacts view is on its way",
            Self::Activity => "Nothing needs your attention",
        }
    }

    /// The honest empty-state body: what the feature will do.
    pub(crate) const fn empty_body(self) -> &'static str {
        match self {
            Self::ProjectsTasks => {
                "This is where every project and its tasks will live — objectives, agents, \
                 environments, artifacts and evidence, all in one place."
            }
            Self::ReusableWorkflows => {
                "Flauz can remember the successful steps of a task so you can run them \
                 again. Saved workflows appear here with their history, and are suggested \
                 on similar future tasks."
            }
            Self::Artifacts => {
                "This is where the work products from all your tasks will be collected — \
                 reports, documents, data, and files — each attached to the task and the \
                 evidence behind it."
            }
            Self::Activity => {
                "Activity will show running work, agent activity, waiting approvals, \
                 failures, and completed outcomes across every project — so you never \
                 have to inspect each task yourself."
            }
        }
    }

    /// The honest empty-state next step: what to do next, today.
    pub(crate) const fn next_step(self) -> &'static str {
        match self {
            Self::ProjectsTasks => {
                "Your chats and local projects remain in the sidebar today; the \
                 workspace view arrives with the Flauz platform."
            }
            Self::ReusableWorkflows => {
                "Finish a task successfully, then choose \u{201c}Save as a reusable \
                 workflow\u{201d} — it will appear here and be suggested on similar tasks."
            }
            Self::Artifacts => {
                "Files and outputs you produce today appear in a chat's Outputs panel; \
                 the workspace-wide artifacts view arrives with the Flauz platform."
            }
            Self::Activity => {
                "Chats that need your attention appear in the bell menu today; the \
                 full activity view arrives with the Flauz platform."
            }
        }
    }

    /// The surface icon.
    pub(crate) const fn icon(self) -> IconName {
        match self {
            Self::ProjectsTasks => IconName::LayoutDashboard,
            Self::ReusableWorkflows => IconName::Redo2,
            Self::Artifacts => IconName::File,
            Self::Activity => IconName::Bell,
        }
    }

    /// The stable element id fragment for this surface.
    pub(crate) const fn id(self) -> &'static str {
        match self {
            Self::ProjectsTasks => "projects-tasks",
            Self::ReusableWorkflows => "reusable-workflows",
            Self::Artifacts => "artifacts",
            Self::Activity => "activity",
        }
    }
}

/// A task-rail section (PRODUCT-UX-JOURNEYS §2.1 "Task control rail"). The
/// rail is visible while a task is selected; its controls are labeled and
/// remain available while work is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskRailSection {
    /// What the agent currently knows and why.
    Context,
    /// Participating agents, roles and assignments.
    Agents,
    /// Browser sites, terminals, sandboxes, desktops and other execution
    /// surfaces.
    Environments,
    /// Observations, claims, verification and provenance.
    Evidence,
    /// Resources, artifacts, approvals, conflicts and detailed activity.
    MoreInspect,
}

impl TaskRailSection {
    /// Every rail section, in rail order.
    pub(crate) const ALL: [Self; 5] = [
        Self::Context,
        Self::Agents,
        Self::Environments,
        Self::Evidence,
        Self::MoreInspect,
    ];

    /// The rail button label.
    pub(crate) const fn rail_label(self) -> &'static str {
        match self {
            Self::Context => "Context",
            Self::Agents => "Agents",
            Self::Environments => "Environments",
            Self::Evidence => "Evidence",
            Self::MoreInspect => "More",
        }
    }

    /// The rail button tooltip.
    pub(crate) const fn rail_tooltip(self) -> &'static str {
        match self {
            Self::Context => "What the agent currently knows and why",
            Self::Agents => "Participating agents, roles and assignments",
            Self::Environments => {
                "Browser sites, terminals, sandboxes, desktops and other execution surfaces"
            }
            Self::Evidence => "Observations, claims, verification and provenance",
            Self::MoreInspect => "Inspect resources, approvals, conflicts and detailed activity",
        }
    }

    /// The command-palette row title.
    pub(crate) const fn palette_title(self) -> &'static str {
        match self {
            Self::Context => "Inspect context",
            Self::Agents => "Task agents",
            Self::Environments => "Task environments",
            Self::Evidence => "Task evidence",
            Self::MoreInspect => "Inspect task details",
        }
    }

    /// The command-palette row description.
    pub(crate) const fn palette_description(self) -> &'static str {
        match self {
            Self::Context => "See what the agent working this task knows, and why",
            Self::Agents => "See participating agents, their roles and assignments",
            Self::Environments => "See the sites, terminals and sandboxes this task uses",
            Self::Evidence => "See observations, verification status and provenance",
            Self::MoreInspect => "Resources, approvals and detailed activity",
        }
    }

    /// The honest empty-state title.
    pub(crate) const fn empty_title(self) -> &'static str {
        match self {
            Self::Context => "The context view is on its way",
            Self::Agents => "No additional agents are assigned",
            Self::Environments => "No environments attached",
            Self::Evidence => "No evidence collected yet",
            Self::MoreInspect => "Nothing more to inspect yet",
        }
    }

    /// The honest empty-state body: what the feature will do.
    pub(crate) const fn empty_body(self) -> &'static str {
        match self {
            Self::Context => {
                "The context view will show what the agent currently knows — the sources \
                 it is drawing from and why each one is included — and let you pin or \
                 remove items where policy allows."
            }
            Self::Agents => {
                "Delegate a task when parallel work would help. Assigned agents, their \
                 roles, and their assignments will appear here."
            }
            Self::Environments => {
                "Environments are where work actually happens — browser sites, terminals, \
                 sandboxes, and desktops. Attached environments and their status will \
                 appear here."
            }
            Self::Evidence => {
                "Evidence separates what was merely claimed from what was actually \
                 verified — with observations, verification status, and where each fact \
                 came from."
            }
            Self::MoreInspect => {
                "The inspector will hold the task's resources, approvals, conflicts, and \
                 detailed activity — the state that belongs to the task but not to the \
                 primary rail."
            }
        }
    }

    /// The honest empty-state next step: what to do next, today.
    pub(crate) const fn next_step(self) -> &'static str {
        match self {
            Self::Context => {
                "Keep working in the conversation; the task's durable state is preserved \
                 even when context is reset."
            }
            Self::Agents => {
                "Ask the agent in the conversation to split the work when a task has \
                 independent parts."
            }
            Self::Environments => {
                "Use the Browser and Terminal panels in this chat today; attached \
                 environments will show up here."
            }
            Self::Evidence => {
                "As work is verified, verified outcomes will be listed here with their \
                 sources."
            }
            Self::MoreInspect => {
                "Start with Context, Agents, Environments and Evidence; more inspection \
                 surfaces arrive as the platform grows."
            }
        }
    }

    /// The chord label shown in palette rows and tooltips.
    pub(crate) const fn shortcut_label(self) -> &'static str {
        match self {
            Self::Context => "Ctrl+Alt+Shift+1",
            Self::Agents => "Ctrl+Alt+Shift+2",
            Self::Environments => "Ctrl+Alt+Shift+3",
            Self::Evidence => "Ctrl+Alt+Shift+4",
            Self::MoreInspect => "Ctrl+Alt+Shift+5",
        }
    }

    /// The section icon.
    pub(crate) const fn icon(self) -> IconName {
        match self {
            Self::Context => IconName::Info,
            Self::Agents => IconName::Bot,
            Self::Environments => IconName::Globe,
            Self::Evidence => IconName::CircleCheck,
            Self::MoreInspect => IconName::Inspector,
        }
    }

    /// The stable element id fragment for this section.
    pub(crate) const fn id(self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Agents => "agents",
            Self::Environments => "environments",
            Self::Evidence => "evidence",
            Self::MoreInspect => "more",
        }
    }
}

/// The honest guidance shown when a rail chord fires with no chat selected
/// (the WO-P2-012 pattern: guidance instead of a silent no-op).
pub(crate) const TASK_RAIL_NO_CHAT_GUIDANCE: &str =
    "Open a chat to inspect its context, agents, environments and evidence.";

/// The Projects & tasks empty state when the workspace truly has no work
/// yet (otherwise the surface explains what is coming).
const PROJECTS_TASKS_EMPTY_TITLE_FRESH: &str = "No projects or tasks yet";
const PROJECTS_TASKS_EMPTY_BODY_FRESH: &str = "Projects hold your ongoing work. Each task keeps its objective, what the \
     agent did, and everything it produced — whichever model or environment \
     ran it.";

/// The additive shell state: which workspace surface is open, which task
/// rail section is open, and the shell's focus bookkeeping. All logic lives
/// here; the `ui.rs` seams only read and render it.
pub(crate) struct FlauzShellState {
    workspace_surface: Option<FlauzWorkspaceSurface>,
    task_rail_section: Option<TaskRailSection>,
    /// The shell surface's keyboard focus handle (the 019 request-once
    /// shape: auto-focused on the first render after an open).
    shell_focus: FocusHandle,
    /// Request-once guard for `shell_focus`.
    shell_focus_requested: bool,
    /// The surface that held focus when the shell opened, restored on close
    /// (the 017 close contract).
    focus_before_shell: Option<FocusHandle>,
}

impl FlauzShellState {
    /// Builds the closed, empty shell state.
    pub(crate) fn new(cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            workspace_surface: None,
            task_rail_section: None,
            shell_focus: cx.focus_handle(),
            shell_focus_requested: false,
            focus_before_shell: None,
        }
    }

    /// The open workspace surface, if any.
    pub(crate) fn workspace_surface(&self) -> Option<FlauzWorkspaceSurface> {
        self.workspace_surface
    }

    /// Closes both shell surfaces for an F1 navigation action. Returns
    /// whether anything changed (the caller notifies). Focus is left where
    /// the navigation action itself puts it — the deliberate close paths
    /// (Escape, toggles) restore through the 017 contract instead.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.workspace_surface.is_some() || self.task_rail_section.is_some();
        self.workspace_surface = None;
        self.task_rail_section = None;
        self.shell_focus_requested = false;
        self.focus_before_shell = None;
        changed
    }
}

/// Whether an F1 action navigates the main area or mounts a task-workspace
/// surface, and therefore closes the shell surfaces (the UX-001 navigation
/// registration seam in `WorkspaceView::dispatch`). The reducer itself is
/// untouched — the shell is additive UI state.
pub(crate) fn action_closes_shell_surfaces(action: &Action) -> bool {
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

/// Opens (or toggles closed) a workspace surface. Called from the sidebar
/// nav buttons, the palette rows, and the direct keyboard chords.
pub(crate) fn open_workspace_surface(
    workspace: &mut WorkspaceView,
    surface: FlauzWorkspaceSurface,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.flauz_shell.workspace_surface == Some(surface) {
        dismiss_shell_surfaces(workspace, window, cx);
        return;
    }
    let shell = &mut workspace.flauz_shell;
    if shell.focus_before_shell.is_none() {
        shell.focus_before_shell = window.focused(cx);
    }
    shell.workspace_surface = Some(surface);
    shell.task_rail_section = None;
    shell.shell_focus_requested = true;
    cx.notify();
}

/// Opens (or toggles closed) a task-rail section on the selected chat.
/// Called from the rail buttons, the palette rows, and the direct keyboard
/// chords. With no chat selected, surfaces honest guidance instead of a
/// silent no-op (the WO-P2-012 pattern).
pub(crate) fn open_task_rail_section(
    workspace: &mut WorkspaceView,
    section: TaskRailSection,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.state.selected_task_id.is_none() {
        workspace.dispatch_command_status(Some(TASK_RAIL_NO_CHAT_GUIDANCE), cx);
        return;
    }
    // The rail lives on the task workspace: navigate there first when
    // needed (the dispatch seam closes any open shell surfaces), so the
    // rail is reachable from any route. When already there — the rail
    // buttons and in-task chords — no navigation runs, preserving the
    // toggle-close contract below.
    if workspace.state.route != MainRoute::Tasks {
        workspace.navigate(MainRoute::Tasks, cx);
    }
    if workspace.flauz_shell.task_rail_section == Some(section) {
        dismiss_shell_surfaces(workspace, window, cx);
        return;
    }
    let shell = &mut workspace.flauz_shell;
    if shell.focus_before_shell.is_none() {
        shell.focus_before_shell = window.focused(cx);
    }
    shell.task_rail_section = Some(section);
    shell.shell_focus_requested = true;
    cx.notify();
}

/// Closes any open shell surface and restores focus (the 017 close
/// contract: the surface that held focus before the open, else the
/// composer house default when rendered, else no element focus).
pub(crate) fn dismiss_shell_surfaces(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let changed = workspace.flauz_shell.close_for_navigation();
    if !changed {
        return;
    }
    let previous = workspace.flauz_shell.focus_before_shell.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// Renders the labeled workspace navigation section for the sidebar (the
/// UX-001 navigation registration seam in `render_sidebar`).
pub(crate) fn render_workspace_nav(
    workspace: &mut WorkspaceView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open_surface = workspace.flauz_shell.workspace_surface;
    let mut section = v_flex()
        .px_2()
        .py_2()
        .gap_0p5()
        .border_b_1()
        .border_color(cx.theme().sidebar_border)
        .child(
            div()
                .h(px(32.0))
                .px_2()
                .mt_1()
                .flex()
                .items_center()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(cx.theme().muted_foreground)
                .child("Workspace"),
        );
    for surface in FlauzWorkspaceSurface::ALL {
        section = section.child(
            Button::new(SharedString::from(format!("flauz-nav-{}", surface.id())))
                .label(surface.nav_label())
                .icon(surface.icon())
                .tooltip(surface.nav_tooltip())
                .small()
                .ghost()
                .w_full()
                .h(px(34.0))
                .justify_start()
                .selected(open_surface == Some(surface))
                .on_click(cx.listener(move |this, _, window, cx| {
                    open_workspace_surface(this, surface, window, cx);
                })),
        );
    }
    section.into_any_element()
}

/// Renders the open workspace surface as the main content (the UX-001
/// navigation registration seam in `render_main`). Honest empty state: the
/// control exists, the copy explains what the feature will do and what to
/// do next, and nothing pretends to be wired.
pub(crate) fn render_workspace_surface(
    workspace: &mut WorkspaceView,
    surface: FlauzWorkspaceSurface,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    if workspace.flauz_shell.shell_focus_requested {
        // The 019 request-once shape: the surface claims the keyboard on
        // mount so the scoped Escape binding reaches it.
        workspace.flauz_shell.shell_focus_requested = false;
        workspace.flauz_shell.shell_focus.focus(window);
    }
    let shell_focus = workspace.flauz_shell.shell_focus.clone();
    let has_work_today =
        !workspace.state.tasks.is_empty() || !workspace.state.local_projects.is_empty();
    let (empty_title, empty_body) =
        if surface == FlauzWorkspaceSurface::ProjectsTasks && !has_work_today {
            (
                PROJECTS_TASKS_EMPTY_TITLE_FRESH,
                PROJECTS_TASKS_EMPTY_BODY_FRESH,
            )
        } else {
            (surface.empty_title(), surface.empty_body())
        };

    v_flex()
        .key_context("FlauzWorkspaceSurface")
        .track_focus(&shell_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            dismiss_shell_surfaces(this, window, cx);
        }))
        .flex_1()
        .min_w_0()
        .h_full()
        .child(render_surface_header(surface, cx))
        .child(
            v_flex()
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .items_center()
                .p_6()
                .child(render_empty_state(
                    surface.icon(),
                    empty_title,
                    empty_body,
                    surface.next_step(),
                    cx,
                ))
                .when(surface == FlauzWorkspaceSurface::ProjectsTasks, |body| {
                    body.child(
                        v_flex().mt_3().child(
                            Button::new("flauz-projects-start-chat")
                                .label("Start a new chat")
                                .icon(IconName::Plus)
                                .small()
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.begin_new_chat(window, cx);
                                })),
                        ),
                    )
                }),
        )
        .child(render_surface_footer("Escape returns to your work", cx))
        .into_any_element()
}

/// Renders the task rail: a labeled row of section controls under the task
/// header, plus the open section's panel (the UX-001 navigation
/// registration seam in `render_task_workspace`). The rail stays available
/// while a task is selected — never hidden behind developer settings.
pub(crate) fn render_task_rail(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open_section = workspace.flauz_shell.task_rail_section;
    let mut rail = v_flex()
        .flex_none()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            h_flex()
                .h(px(36.0))
                .px_5()
                .gap_1()
                .items_center()
                .flex_wrap()
                .overflow_hidden()
                .children(TaskRailSection::ALL.map(|section| {
                    let tooltip =
                        format!("{} ({})", section.rail_tooltip(), section.shortcut_label());
                    Button::new(SharedString::from(format!(
                        "flauz-task-rail-{}",
                        section.id()
                    )))
                    .label(section.rail_label())
                    .tooltip(SharedString::from(tooltip))
                    .small()
                    .ghost()
                    .selected(open_section == Some(section))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        open_task_rail_section(this, section, window, cx);
                    }))
                })),
        );
    if let Some(section) = open_section {
        if workspace.flauz_shell.shell_focus_requested {
            // The 019 request-once shape for the rail panel.
            workspace.flauz_shell.shell_focus_requested = false;
            workspace.flauz_shell.shell_focus.focus(window);
        }
        let shell_focus = workspace.flauz_shell.shell_focus.clone();
        rail = rail.child(
            v_flex()
                .key_context("FlauzTaskRail")
                .track_focus(&shell_focus)
                .tab_group()
                .tab_stop(true)
                .on_action(cx.listener(|this, _: &Escape, window, cx| {
                    dismiss_shell_surfaces(this, window, cx);
                }))
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
                        .child(Icon::new(section.icon()).small())
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(section.empty_title()),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(section.empty_body()),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .child(section.next_step()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Escape closes this panel"),
                ),
        );
    }
    rail.into_any_element()
}

/// Renders a surface header row (the Workflows-route house shape).
fn render_surface_header(
    surface: FlauzWorkspaceSurface,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    h_flex()
        .flex_none()
        .h(px(62.0))
        .px_6()
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
                        .child(surface.heading()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(surface.description()),
                ),
        )
        .child(
            Button::new(SharedString::from(format!("flauz-close-{}", surface.id())))
                .label("Back to your work")
                .icon(IconName::ArrowLeft)
                .tooltip("Return to your chat (Escape)")
                .small()
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    dismiss_shell_surfaces(this, window, cx);
                })),
        )
        .into_any_element()
}

/// Renders the honest empty state: what the feature will do, and what to do
/// next. Never a dead end, never a fake control.
fn render_empty_state(
    icon: IconName,
    title: &str,
    body: &str,
    next_step: &str,
    cx: &App,
) -> AnyElement {
    v_flex()
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
                .child(Icon::new(icon).small())
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(title.to_owned()),
                ),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(body.to_owned()),
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
                        .child(next_step.to_owned()),
                ),
        )
        .into_any_element()
}

/// Renders the keyboard-hint footer (the Activity-view house shape).
fn render_surface_footer(hint: &str, cx: &App) -> AnyElement {
    h_flex()
        .flex_none()
        .px_6()
        .py_3()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(hint.to_owned())
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_user_copy() -> Vec<&'static str> {
        let mut copy = Vec::new();
        for surface in FlauzWorkspaceSurface::ALL {
            copy.push(surface.nav_label());
            copy.push(surface.nav_tooltip());
            copy.push(surface.heading());
            copy.push(surface.description());
            copy.push(surface.palette_title());
            copy.push(surface.palette_description());
            copy.push(surface.empty_title());
            copy.push(surface.empty_body());
            copy.push(surface.next_step());
        }
        for section in TaskRailSection::ALL {
            copy.push(section.rail_label());
            copy.push(section.rail_tooltip());
            copy.push(section.palette_title());
            copy.push(section.palette_description());
            copy.push(section.empty_title());
            copy.push(section.empty_body());
            copy.push(section.next_step());
        }
        copy.push(TASK_RAIL_NO_CHAT_GUIDANCE);
        copy.push("Workspace");
        // The surface furniture and state-aware copy render as literals —
        // they are user copy too and must obey the same language rules.
        copy.push(PROJECTS_TASKS_EMPTY_TITLE_FRESH);
        copy.push(PROJECTS_TASKS_EMPTY_BODY_FRESH);
        for literal in [
            "What to do next",
            "Escape closes this panel",
            "Escape returns to your work",
            "Back to your work",
            "Return to your chat (Escape)",
            "Start a new chat",
        ] {
            copy.push(literal);
        }
        copy
    }

    #[test]
    fn shell_copy_uses_reusable_workflow_language() {
        // UX-001: the internal word "Procedure" never appears in UI copy;
        // the user-facing language is "reusable workflow" (J-10/J-11).
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            assert!(
                !lower.contains("procedure"),
                "UI copy must never say \"Procedure\": {copy:?}"
            );
        }
        let workflows = FlauzWorkspaceSurface::ReusableWorkflows;
        assert!(workflows.nav_label().contains("Reusable workflows"));
        assert!(workflows.empty_body().contains("run them again"));
        assert!(
            workflows
                .next_step()
                .contains("Save as a reusable workflow")
        );
    }

    #[test]
    fn shell_copy_avoids_internal_type_names() {
        // No internal implementation terms leak into user copy.
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "contextsnapshot",
                "memoryitem",
                "modelcontextprofile",
                "flauz-context",
                "flauz-world",
                "flauz-exec",
                "ctxsnap",
                "orch-001",
                "ux-001",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "UI copy must not leak the internal term {forbidden:?}: {copy:?}"
                );
            }
        }
    }

    #[test]
    fn every_shell_surface_has_complete_empty_state_copy() {
        // Discoverability contract §1.4: a useful empty state explains the
        // capability and provides a concrete first action — no empty
        // strings, no dead ends.
        for surface in FlauzWorkspaceSurface::ALL {
            assert!(!surface.empty_title().is_empty());
            assert!(surface.empty_body().len() > 40, "body explains the feature");
            assert!(
                surface.next_step().len() > 40,
                "the next step says what to do"
            );
            assert!(!surface.palette_title().is_empty());
            assert!(!surface.palette_description().is_empty());
            assert!(!surface.nav_label().is_empty());
            assert!(!surface.nav_tooltip().is_empty());
        }
        for section in TaskRailSection::ALL {
            assert!(!section.empty_title().is_empty());
            assert!(section.empty_body().len() > 40, "body explains the feature");
            assert!(
                section.next_step().len() > 40,
                "the next step says what to do"
            );
            assert!(!section.palette_title().is_empty());
            assert!(!section.palette_description().is_empty());
            assert!(!section.rail_label().is_empty());
            assert!(!section.rail_tooltip().is_empty());
        }
        assert!(!TASK_RAIL_NO_CHAT_GUIDANCE.is_empty());
    }

    #[test]
    fn palette_queries_resolve_to_shell_rows() {
        // Discoverability contract §1.3: the palette fallback finds every
        // surface without knowing its location. Mirrors the WO-P2-010
        // runtime contract at the copy level: a natural query must hit a
        // row's title or description.
        let queries: &[(&str, FlauzWorkspaceSurface)] = &[
            ("projects", FlauzWorkspaceSurface::ProjectsTasks),
            ("tasks", FlauzWorkspaceSurface::ProjectsTasks),
            ("workflow", FlauzWorkspaceSurface::ReusableWorkflows),
            ("reusable", FlauzWorkspaceSurface::ReusableWorkflows),
            ("artifact", FlauzWorkspaceSurface::Artifacts),
            ("activity", FlauzWorkspaceSurface::Activity),
            ("attention", FlauzWorkspaceSurface::Activity),
        ];
        for (query, surface) in queries {
            let lower_title = surface.palette_title().to_lowercase();
            let lower_description = surface.palette_description().to_lowercase();
            assert!(
                lower_title.contains(query) || lower_description.contains(query),
                "query {query:?} must resolve to the {:?} palette row",
                surface
            );
        }
        let rail_queries: &[(&str, TaskRailSection)] = &[
            ("context", TaskRailSection::Context),
            ("agent", TaskRailSection::Agents),
            ("environment", TaskRailSection::Environments),
            ("evidence", TaskRailSection::Evidence),
            ("inspect", TaskRailSection::MoreInspect),
        ];
        for (query, section) in rail_queries {
            let lower_title = section.palette_title().to_lowercase();
            let lower_description = section.palette_description().to_lowercase();
            assert!(
                lower_title.contains(query) || lower_description.contains(query),
                "query {query:?} must resolve to the {:?} palette row",
                section
            );
        }
    }

    #[test]
    fn shell_surfaces_and_rail_sections_have_distinct_labels() {
        let mut nav_labels: Vec<&str> = FlauzWorkspaceSurface::ALL
            .iter()
            .map(|surface| surface.nav_label())
            .collect();
        let count = nav_labels.len();
        nav_labels.sort_unstable();
        nav_labels.dedup();
        assert_eq!(nav_labels.len(), count, "nav labels are distinct");

        let mut rail_labels: Vec<&str> = TaskRailSection::ALL
            .iter()
            .map(|section| section.rail_label())
            .collect();
        let count = rail_labels.len();
        rail_labels.sort_unstable();
        rail_labels.dedup();
        assert_eq!(rail_labels.len(), count, "rail labels are distinct");
    }

    /// UX-001 registration seam test (the house ui.rs source-inspection
    /// style): every new surface must be wired through the navigation,
    /// palette, and keyboard seams in `ui.rs`.
    #[test]
    fn shell_surfaces_are_registered_in_the_ui_seams() {
        let source = include_str!("../../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_shell;"));

        // The palette registration seam: every surface has a palette
        // command that dispatches through the shell module.
        let palette_commands = [
            "PaletteCommand::OpenProjectsTasks",
            "PaletteCommand::OpenReusableWorkflows",
            "PaletteCommand::OpenArtifacts",
            "PaletteCommand::OpenWorkspaceActivity",
            "PaletteCommand::InspectTaskContext",
            "PaletteCommand::InspectTaskAgents",
            "PaletteCommand::InspectTaskEnvironments",
            "PaletteCommand::InspectTaskEvidence",
            "PaletteCommand::InspectTaskMore",
        ];
        for command in palette_commands {
            assert!(
                source.contains(command),
                "{command} must be registered in the palette"
            );
        }
        // The palette groups the shell rows under one labeled heading.
        assert!(source.contains("PaletteGroup::WorkspaceShell"));
        assert!(source.contains("(PaletteGroup::WorkspaceShell, \"Workspace\")"));

        // The navigation registration seams: the sidebar section, the main
        // surface branch, the task rail, and the dispatch navigation close.
        assert!(source.contains("flauz_shell::render_workspace_nav(self, cx)"));
        assert!(source.contains("flauz_shell::render_workspace_surface("));
        assert!(source.contains("flauz_shell::render_task_rail(self, window, cx)"));
        assert!(source.contains("flauz_shell::action_closes_shell_surfaces(&action)"));

        // The keyboard registration seams: one chord per §2.1 surface (so
        // no surface depends on pointer input) plus the scoped escape
        // bindings for the shell's focus contexts. Scanned against the
        // whitespace-normalized source: rustfmt legitimately wraps long
        // `KeyBinding::new(...)` expressions across lines, and the chord
        // registration must keep matching regardless of formatting.
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        for binding in [
            "FlauzProjectsTasksShortcut, None",
            "FlauzReusableWorkflowsShortcut, None",
            "FlauzArtifactsShortcut, None",
            "FlauzActivityShortcut, None",
            "FlauzTaskContextShortcut, None",
            "FlauzTaskAgentsShortcut, None",
            "FlauzTaskEnvironmentsShortcut, None",
            "FlauzTaskEvidenceShortcut, None",
            "FlauzTaskMoreInspectShortcut, None",
        ] {
            assert!(
                normalized.contains(binding),
                "a keyboard chord must be bound for {binding}"
            );
        }
        // Gate-fix regression guard (F2 Gate B, the N6 shifted-keysym
        // family): the task-rail chords must ALSO be registered in their
        // shifted-symbol forms — on Linux a Shift+1..5 keystroke reports
        // "!"/"@"/"#"/"$"/"%" as the key, so the digit form alone can never
        // match the physical main-row keys.
        for companion in [
            "alt-shift-!",
            "alt-shift-@",
            "alt-shift-#",
            "alt-shift-$",
            "alt-shift-%",
        ] {
            assert!(
                source.contains(&format!("shortcut(\"{companion}\")")),
                "the shifted-symbol chord companion {companion} must be registered"
            );
        }
        assert!(source.contains("Some(\"FlauzWorkspaceSurface\")"));
        assert!(source.contains("Some(\"FlauzTaskRail\")"));
    }
}
