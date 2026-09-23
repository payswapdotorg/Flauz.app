//! The Agents panel (work order ORCH-004, F2 Wave 3): the J-07 view —
//! who is working on this task, what they're waiting for, and what has
//! been independently checked, without reading any transcript.
//!
//! The task rail's **Agents section** renders this panel (the shell
//! mount delegates the panel body here when the section is open). The
//! node list shows every agent assignment with its **who** and **role**,
//! a state chip in user language ("Waiting for the research notes" —
//! never "blocked-on-artifact-id"), a per-node dependency disclosure
//! ("Blocked until: the analysis notes"), and **verification badges that
//! distinguish independently verified from merely finished** (J-09's
//! law: verified ≠ claimed).
//!
//! Seven discoverability layers (PRODUCT-UX-JOURNEYS §1 / the shell
//! family):
//!
//! 1. **Visible primary entry** — the task rail's labeled `Agents`
//!    control (the shell's; always visible while a task is selected).
//! 2. **Contextual affordance** — the per-node dependency disclosure
//!    that appears exactly when a node is waiting, and the parallel
//!    progress view while several agents work.
//! 3. **Palette fallback** — the "See who is working on this task" row
//!    (the WorkspaceShell group).
//! 4. **Stateful empty state** — no agents: what parallel agents add
//!    and how to add one (ask in the conversation to split the work).
//! 5. **Success-state continuation** — the merge summary when everyone
//!    finished (how many independently verified, how many merely
//!    finished) pointing at "Save as a reusable workflow".
//! 6. **Keyboard path** — `Ctrl+Alt+Shift+7` opens the panel; one
//!    scoped Escape (the rail panel's own focus context) closes it;
//!    the panel is tab-navigable.
//! 7. **Honest unavailable state** — until the execution graph is wired
//!    into live tasks, the panel shows the honest wiring state: what
//!    will appear here, and no invented agents.
//!
//! Boundary rules (work order ORCH-004 / kernel §1):
//!
//! - the module wires through minimal `ui.rs` seams only (module
//!   declaration, palette registration, keyboard chord + its
//!   shifted-symbol companion, and the workspace-root action handler) —
//!   distinct from MOD-001's, CAP-001's and ORCH-003's seams;
//! - the app crate does not import the `flauz-orch` contract crate in
//!   this wave (the app's Cargo.toml is outside this order's owned
//!   files): the panel renders a plain **view-model**
//!   ([`AgentsViewDetail`]) whose shapes re-pin the crate's evaluation
//!   output in user language. A later slice populates it from a real
//!   graph evaluation; until then the panel shows the honest wiring
//!   state below — the chain is implemented, but no data is invented;
//! - the panel has no focus state of its own: it renders inside the
//!   shell's task-rail panel, whose scoped Escape (the `FlauzTaskRail`
//!   focus context) dismisses it — the keyboard is never trapped
//!   (the d19 discipline).
//!
//! Every user-facing string lives in the copy registry below so the
//! language rules are unit-testable in one place.

use gpui::prelude::*;
use gpui::{AnyElement, Window, div, px};
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};

use super::WorkspaceView;

gpui::actions!(codexrs, [FlauzAgentsViewShortcut]);

/// The panel heading (J-07's question: "who is doing what?").
const PANEL_HEADING: &str = "Agents on this task";
/// The one-line description under the heading.
const PANEL_DESCRIPTION: &str =
    "Who is doing what, what they're waiting for, and what has been independently checked";
/// The parallel-progress view's headline hint (J-07: "watch parallel
/// progress").
const PARALLEL_PROGRESS_HINT: &str =
    "Working in parallel — watch each agent's progress here, without reading any transcript";
/// The honest empty state: no agents assigned.
const EMPTY_TITLE: &str = "No extra agents on this task yet";
/// What parallel agents add (the empty state's body).
const EMPTY_BODY: &str = "Extra agents let one task work in parallel: one agent researches \
     while another analyzes, and an independent reviewer checks the results. Everything each \
     one produces stays separate and attributed to its maker.";
/// How to add one, today (the empty state's next step).
const EMPTY_NEXT_STEP: &str = "Ask the agent in this conversation to split the work when a \
     task has independent parts. Assigned agents, their roles and what they're waiting for \
     appear here.";
/// The honest wiring state: the panel is implemented, the live graph
/// data is not wired yet.
const WIRED_TITLE: &str = "Live agent progress is on its way";
const WIRED_BODY: &str = "When a task runs with several agents, this panel shows each one's \
     role and progress: who is working, who is waiting for another's notes, and whose results \
     were checked by an independent reviewer. No agent is invented here before that wiring \
     lands.";
/// The honest wiring state's next step.
const WIRED_NEXT_STEP: &str = "The panel is ready — the run data arrives with the multi-agent \
     execution wiring. Ask in the conversation to split work with extra agents in the \
     meantime.";
/// The verification badge for an independently verified node (J-09).
const VERIFIED_BADGE: &str = "Independently verified";
/// The badge's explanation (independence in user language: the reviewer
/// only ever saw the work products).
const VERIFIED_BADGE_EXPLANATION: &str =
    "Checked by a separate reviewer that worked from the finished results alone";
/// The badge shown on a node that failed.
const DIDNT_FINISH_BADGE: &str = "Didn't finish";
/// The dependency disclosure's prefix (the work order's exact copy
/// shape: "blocked until: Analysis notes artifact").
const DEPENDENCY_PREFIX: &str = "Blocked until";
/// The merge summary's heading (all-done).
const MERGED_HEADING: &str = "Everyone has finished";
/// The cross-link to the save affordance in the merge summary (J-10's
/// next step after a successful run).
const MERGED_SAVE_HINT: &str = "This finished run can now be saved as a reusable workflow — see “Save as a reusable \
     workflow” on this task";
/// The guidance shown when the chord or palette row fires with no chat
/// selected (the WO-P2-012 pattern: guidance instead of a silent
/// no-op).
const NO_CHAT_GUIDANCE: &str = "Open a chat to see who is working on its task.";
/// The keyboard-hint footer (the rail panel's Escape, restated).
const ESCAPE_HINT: &str = "Escape closes this panel";
/// The command-palette row title (ORCH-004's exact palette row).
pub(crate) const PALETTE_ROW_TITLE: &str = "See who is working on this task";
/// The command-palette row description.
pub(crate) const PALETTE_ROW_DESCRIPTION: &str =
    "Watch each agent's role, progress and what they're waiting for";
/// The chord label shown in tooltips (the rail family, after
/// Capabilities).
#[allow(dead_code)] // asserted by the module's tests; the chord text is embedded in the tooltip copy
pub(crate) const KEYBOARD_CHORD_LABEL: &str = "Ctrl+Alt+Shift+7";

/// One node's state in the view-model — the six frozen graph states in
/// user language. The chip copy lives on [`AgentNodeView`] because the
/// waiting chip names what it waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // matched by the render today; constructed by the module's tests and (Wave-later) the evaluation population
pub(crate) enum AgentStateView {
    /// Ready to start.
    ReadyToStart,
    /// Currently working.
    Working,
    /// Waiting for another agent's work.
    Waiting,
    /// Finished (claimed, not yet independently verified).
    Finished,
    /// Failed.
    DidntFinish,
    /// Finished and independently verified by a separate reviewer.
    IndependentlyVerified,
}

/// One uncleared wait in the view-model: what is awaited and from which
/// role it comes ("the analysis notes" from "Analysis").
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // read by the render today; constructed by the module's tests and (Wave-later) the evaluation population
pub(crate) struct AgentWaitView {
    /// What is awaited, in user language ("the analysis notes").
    pub label: String,
    /// The role whose work it comes from ("Analysis").
    pub from_role: String,
}

/// One agent assignment in the view-model: who, role, state, waits.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // read by the render today; constructed by the module's tests and (Wave-later) the evaluation population
pub(crate) struct AgentNodeView {
    /// The agent's display name ("Research agent").
    pub who: String,
    /// The role label ("Research", "Independent review").
    pub role: String,
    /// The node's state.
    pub state: AgentStateView,
    /// The node's uncleared waits (empty unless waiting).
    pub waiting_on: Vec<AgentWaitView>,
    /// Whether this node is the independent reviewer.
    pub verifier: bool,
}

impl AgentNodeView {
    /// The state chip in user language. A waiting node NAMES what it
    /// waits for ("Waiting for the research notes") — never
    /// "blocked-on-artifact-id".
    pub(crate) fn state_chip(&self) -> String {
        match self.state {
            AgentStateView::ReadyToStart => "Ready to start".to_owned(),
            AgentStateView::Working => "Working".to_owned(),
            AgentStateView::Waiting => {
                let labels: Vec<&str> = self
                    .waiting_on
                    .iter()
                    .map(|wait| wait.label.as_str())
                    .collect();
                if labels.is_empty() {
                    "Waiting for another agent's work".to_owned()
                } else {
                    format!("Waiting for {}", labels.join(" and "))
                }
            }
            AgentStateView::Finished => "Finished".to_owned(),
            AgentStateView::DidntFinish => DIDNT_FINISH_BADGE.to_owned(),
            AgentStateView::IndependentlyVerified => VERIFIED_BADGE.to_owned(),
        }
    }

    /// The per-node dependency disclosure ("Blocked until: the analysis
    /// notes (from Analysis)") — shown exactly when the node waits.
    pub(crate) fn dependency_disclosure(&self) -> Option<String> {
        if self.state != AgentStateView::Waiting || self.waiting_on.is_empty() {
            return None;
        }
        let waits: Vec<String> = self
            .waiting_on
            .iter()
            .map(|wait| format!("{} (from {})", wait.label, wait.from_role))
            .collect();
        Some(format!("{DEPENDENCY_PREFIX}: {}", waits.join(" and ")))
    }

    /// The verification badge: independently verified nodes carry it,
    /// merely finished ones do not — J-09's law (verified ≠ claimed).
    pub(crate) fn verification_badge(&self) -> Option<&'static str> {
        match self.state {
            AgentStateView::IndependentlyVerified => Some(VERIFIED_BADGE),
            _ => None,
        }
    }
}

/// The Agents panel's view-model: what the panel renders once an
/// execution graph's evaluation is attached. Plain owned data — a later
/// wave populates it from a real graph evaluation; this module never
/// invents agent data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // read by the render today; constructed by the module's tests and (Wave-later) the evaluation population
pub(crate) struct AgentsViewDetail {
    /// Every agent assignment, in display order.
    pub nodes: Vec<AgentNodeView>,
    /// Whether the run merged (every node finished).
    pub merged: bool,
}

impl AgentsViewDetail {
    /// How many nodes are independently verified (J-09's split).
    pub(crate) fn independently_verified(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.state == AgentStateView::IndependentlyVerified)
            .count()
    }

    /// How many nodes finished without independent verification.
    pub(crate) fn finished_unverified(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.state == AgentStateView::Finished)
            .count()
    }

    /// The merge summary line (the success state): everyone finished,
    /// with the verified/finished split named.
    pub(crate) fn merge_summary(&self) -> String {
        format!(
            "{} independently verified · {} finished, not yet independently checked",
            self.independently_verified(),
            self.finished_unverified()
        )
    }

    /// Whether the parallel-progress hint applies (more than one agent
    /// still active).
    pub(crate) fn shows_parallel_progress(&self) -> bool {
        self.nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.state,
                    AgentStateView::ReadyToStart
                        | AgentStateView::Working
                        | AgentStateView::Waiting
                )
            })
            .count()
            > 1
    }
}

/// The additive agents-view state: the attached evaluation view-model.
/// The panel's open/close state is the shell's task-rail section (the
/// panel renders inside the shell's rail panel; no focus state of its
/// own — the rail's scoped Escape dismisses it).
pub(crate) struct AgentsViewState {
    detail: Option<AgentsViewDetail>,
}

impl AgentsViewState {
    /// Builds the empty agents-view state (no detail attached).
    pub(crate) fn new() -> Self {
        Self { detail: None }
    }

    /// The wiring seam: attaches an evaluation view-model (a later wave
    /// calls this when a task's execution graph is evaluated; the
    /// module's tests drive it).
    #[allow(dead_code)] // called by the module's tests; the Wave-later graph wiring calls it in production
    pub(crate) fn set_detail(&mut self, detail: AgentsViewDetail) {
        self.detail = Some(detail);
    }

    /// The attached detail, if any.
    #[allow(dead_code)] // read by the module's tests; the render reads the field directly; the Wave-later wiring reads it
    pub(crate) fn detail(&self) -> Option<&AgentsViewDetail> {
        self.detail.as_ref()
    }
}

/// Opens the Agents panel on the selected chat: the task rail's Agents
/// section, through the shell's own logic (so the rail button, the
/// chord and the palette row all drive the same surface). Called from
/// the keyboard chord and the palette row. With no chat selected,
/// surfaces honest guidance instead of a silent no-op.
pub(crate) fn open_agents_view(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.state.selected_task_id.is_none() {
        workspace.dispatch_command_status(Some(NO_CHAT_GUIDANCE), cx);
        return;
    }
    super::flauz_shell::open_task_rail_section(
        workspace,
        super::flauz_shell::TaskRailSection::Agents,
        window,
        cx,
    );
}

/// Renders the Agents panel body (mounted by the shell inside the task
/// rail's section panel): the node list with who/role/state chips, the
/// dependency disclosures, the verification badges and the parallel
/// progress view — or the honest empty/wiring states until a graph
/// evaluation is attached.
pub(crate) fn render_agents_panel(
    workspace: &mut WorkspaceView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let detail = workspace.flauz_agents_view.detail.clone();
    let panel = v_flex()
        .gap_2()
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
        .when_some(detail, |panel, detail| {
            let mut content = v_flex().gap_2();
            if detail.shows_parallel_progress() {
                content = content.child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(PARALLEL_PROGRESS_HINT),
                );
            }
            for node in &detail.nodes {
                content = content.child(render_node_row(node, cx));
            }
            if detail.merged {
                content = content.child(
                    v_flex()
                        .gap_1()
                        .pt_2()
                        .border_t_1()
                        .border_color(cx.theme().border)
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(MERGED_HEADING),
                        )
                        .child(
                            div()
                                .text_sm()
                                .line_height(px(20.0))
                                .child(detail.merge_summary()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .line_height(px(20.0))
                                .text_color(cx.theme().muted_foreground)
                                .child(MERGED_SAVE_HINT),
                        ),
                );
            }
            panel.child(content)
        })
        .when(workspace.flauz_agents_view.detail.is_none(), |panel| {
            panel.child(render_empty_state(cx))
        })
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(ESCAPE_HINT),
        );
    panel.into_any_element()
}

/// Renders one agent node row: who/role, the state chip, the dependency
/// disclosure when waiting, and the verification badge when
/// independently verified.
fn render_node_row(node: &AgentNodeView, cx: &mut Context<WorkspaceView>) -> AnyElement {
    let mut row = v_flex()
        .gap_1()
        .pt_2()
        .border_t_1()
        .border_color(cx.theme().border)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(node.role.clone()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(node.who.clone()),
                )
                .when(node.verifier, |line| {
                    line.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("· independent reviewer"),
                    )
                }),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .when(
                    node.state == AgentStateView::IndependentlyVerified,
                    |line| line.child(Icon::new(IconName::CircleCheck).small()),
                )
                .when(node.state == AgentStateView::DidntFinish, |line| {
                    line.child(Icon::new(IconName::CircleX).small())
                })
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .child(node.state_chip()),
                )
                .when_some(node.verification_badge(), |line, badge| {
                    line.child(
                        div()
                            .text_xs()
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(badge),
                    )
                }),
        );
    if let Some(disclosure) = node.dependency_disclosure() {
        row = row.child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(disclosure),
        );
    }
    if node.state == AgentStateView::IndependentlyVerified {
        row = row.child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(VERIFIED_BADGE_EXPLANATION),
        );
    }
    row.into_any_element()
}

/// Renders the honest empty state: no agents → what parallel agents add
/// and how to add one. Never a dead end, never an invented agent.
fn render_empty_state(cx: &mut Context<WorkspaceView>) -> AnyElement {
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
                .child(Icon::new(IconName::Bot).small())
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
                        .child(WIRED_TITLE),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(WIRED_BODY),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(WIRED_NEXT_STEP),
                ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_user_copy() -> Vec<&'static str> {
        let mut copy = vec![
            PANEL_HEADING,
            PANEL_DESCRIPTION,
            PARALLEL_PROGRESS_HINT,
            EMPTY_TITLE,
            EMPTY_BODY,
            EMPTY_NEXT_STEP,
            WIRED_TITLE,
            WIRED_BODY,
            WIRED_NEXT_STEP,
            VERIFIED_BADGE,
            VERIFIED_BADGE_EXPLANATION,
            DIDNT_FINISH_BADGE,
            DEPENDENCY_PREFIX,
            MERGED_HEADING,
            MERGED_SAVE_HINT,
            NO_CHAT_GUIDANCE,
            ESCAPE_HINT,
            PALETTE_ROW_TITLE,
            PALETTE_ROW_DESCRIPTION,
            KEYBOARD_CHORD_LABEL,
            "What to do next",
            "· independent reviewer",
            "Ready to start",
            "Working",
            "Waiting for another agent's work",
            "Finished",
        ];
        copy.extend(["and", "from", "independently checked"]);
        copy
    }

    fn waiting_node() -> AgentNodeView {
        AgentNodeView {
            who: "Analysis agent".to_owned(),
            role: "Analysis".to_owned(),
            state: AgentStateView::Waiting,
            waiting_on: vec![AgentWaitView {
                label: "the research notes".to_owned(),
                from_role: "Research".to_owned(),
            }],
            verifier: false,
        }
    }

    #[test]
    fn agents_copy_uses_user_language_not_internal_type_names() {
        // ORCH-004: user language only. Internal type names (the
        // contract crate, the record types, the work order id, raw
        // state jargon) never appear in UI copy.
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "agentassignment",
                "executiongraph",
                "graphevaluation",
                "nodeevaluation",
                "verifierscope",
                "waiton",
                "flauz-orch",
                "flauz_world",
                "flauz-world",
                "orch-004",
                "view-model",
                "viewmodel",
                "attribution",
                "artifact-id",
                "artifactid",
                "blocked-on",
                "node id",
                "ulid",
                "evaluator",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "UI copy must not leak the internal term {forbidden:?}: {copy:?}"
                );
            }
        }
    }

    #[test]
    fn a_waiting_node_names_what_it_waits_for_in_user_language() {
        // The work order's law: the state chip says "Waiting for the
        // research notes" — never "blocked-on-artifact-id".
        let node = waiting_node();
        assert_eq!(node.state_chip(), "Waiting for the research notes");
        // The per-node dependency disclosure names it too, with the
        // role it comes from.
        assert_eq!(
            node.dependency_disclosure(),
            Some("Blocked until: the research notes (from Research)".to_owned())
        );

        // Two waits join naturally.
        let two = AgentNodeView {
            waiting_on: vec![
                AgentWaitView {
                    label: "the research notes".to_owned(),
                    from_role: "Research".to_owned(),
                },
                AgentWaitView {
                    label: "the analysis notes".to_owned(),
                    from_role: "Analysis".to_owned(),
                },
            ],
            ..waiting_node()
        };
        assert_eq!(
            two.state_chip(),
            "Waiting for the research notes and the analysis notes"
        );
        assert_eq!(
            two.dependency_disclosure(),
            Some(
                "Blocked until: the research notes (from Research) and the analysis notes \
                  (from Analysis)"
                    .to_owned()
            )
        );

        // A node that is not waiting carries no disclosure.
        let working = AgentNodeView {
            state: AgentStateView::Working,
            waiting_on: Vec::new(),
            ..waiting_node()
        };
        assert_eq!(working.state_chip(), "Working");
        assert!(working.dependency_disclosure().is_none());
    }

    #[test]
    fn verification_badges_distinguish_verified_from_claimed() {
        // J-09's law: independently verified is distinct from merely
        // finished (claimed). The badge, the chip and the explanation
        // all distinguish them.
        let finished = AgentNodeView {
            who: "Review agent".to_owned(),
            role: "Independent review".to_owned(),
            state: AgentStateView::Finished,
            waiting_on: Vec::new(),
            verifier: true,
        };
        let verified = AgentNodeView {
            who: "Research agent".to_owned(),
            role: "Research".to_owned(),
            state: AgentStateView::IndependentlyVerified,
            waiting_on: Vec::new(),
            verifier: false,
        };
        assert_eq!(finished.state_chip(), "Finished");
        assert!(
            finished.verification_badge().is_none(),
            "a merely finished node carries NO verified badge — its result is claimed, not \
             independently checked"
        );
        assert_eq!(verified.state_chip(), VERIFIED_BADGE);
        assert_eq!(verified.verification_badge(), Some(VERIFIED_BADGE));
        assert_ne!(finished.state_chip(), verified.state_chip());

        // The merge summary names the split (J-09 at the task level).
        let detail = AgentsViewDetail {
            nodes: vec![verified.clone(), finished.clone(), verified],
            merged: true,
        };
        assert_eq!(detail.independently_verified(), 2);
        assert_eq!(detail.finished_unverified(), 1);
        assert_eq!(
            detail.merge_summary(),
            "2 independently verified · 1 finished, not yet independently checked"
        );
        assert!(detail.merged);
    }

    #[test]
    fn the_parallel_progress_view_applies_only_with_multiple_active_agents() {
        let node = |state| AgentNodeView {
            who: "Agent".to_owned(),
            role: "Role".to_owned(),
            state,
            waiting_on: Vec::new(),
            verifier: false,
        };
        let one_active = AgentsViewDetail {
            nodes: vec![
                node(AgentStateView::Working),
                node(AgentStateView::Finished),
            ],
            merged: false,
        };
        assert!(!one_active.shows_parallel_progress());
        let two_active = AgentsViewDetail {
            nodes: vec![node(AgentStateView::Working), node(AgentStateView::Waiting)],
            merged: false,
        };
        assert!(two_active.shows_parallel_progress());
        let ready_counts = AgentsViewDetail {
            nodes: vec![
                node(AgentStateView::ReadyToStart),
                node(AgentStateView::ReadyToStart),
            ],
            merged: false,
        };
        assert!(ready_counts.shows_parallel_progress());
    }

    #[test]
    fn every_state_has_a_user_language_chip() {
        for state in [
            AgentStateView::ReadyToStart,
            AgentStateView::Working,
            AgentStateView::Waiting,
            AgentStateView::Finished,
            AgentStateView::DidntFinish,
            AgentStateView::IndependentlyVerified,
        ] {
            let node = AgentNodeView {
                who: "Agent".to_owned(),
                role: "Role".to_owned(),
                state,
                waiting_on: Vec::new(),
                verifier: false,
            };
            let chip = node.state_chip();
            assert!(!chip.is_empty(), "every state has a chip");
            assert!(
                !chip.eq_ignore_ascii_case("blocked"),
                "the raw internal state never shows as a chip: {chip:?}"
            );
        }
    }

    #[test]
    fn palette_queries_resolve_to_the_agents_row() {
        // Discoverability contract §1.3: the palette fallback finds the
        // surface without knowing its location — natural queries hit
        // the row's title or description.
        let title = PALETTE_ROW_TITLE.to_lowercase();
        let description = PALETTE_ROW_DESCRIPTION.to_lowercase();
        for query in ["who", "working", "agent", "waiting"] {
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the agents palette row"
            );
        }
        assert_eq!(PALETTE_ROW_TITLE, "See who is working on this task");
        assert_eq!(KEYBOARD_CHORD_LABEL, "Ctrl+Alt+Shift+7");
    }

    /// ORCH-004 registration seam test (the house ui.rs
    /// source-inspection style): the agents view must be wired through
    /// the palette, the keyboard, and the action-handler seams in
    /// `ui.rs`, and mounted in the shell's task-rail Agents panel.
    #[test]
    fn the_agents_view_is_registered_in_the_ui_and_shell_seams() {
        let source = include_str!("../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_agents_view;"));

        // The palette registration seam: the row dispatches through this
        // module.
        assert!(source.contains("PaletteCommand::SeeWhoIsWorking"));
        assert!(source.contains("flauz_agents_view::open_agents_view("));

        // The keyboard registration seams: the direct chord plus the
        // shifted-symbol companion (Shift+7 → "&", the N6 gate-fix
        // family), scanned against the whitespace-normalized source.
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzAgentsViewShortcut, None"),
            "a keyboard chord must be bound for the agents view"
        );
        assert!(source.contains("shortcut(\"alt-shift-7\")"));
        assert!(source.contains("shortcut(\"alt-&\")"));

        // The scoped Escape rides the shell's task-rail panel (the
        // FlauzTaskRail focus context) — the panel has no focus state
        // of its own, so the keyboard is never trapped.
        assert!(source.contains("Some(\"FlauzTaskRail\")"));

        // The shell mount seam: the task rail's Agents section renders
        // this module's panel.
        let shell = include_str!("flauz_shell/mod.rs");
        assert!(
            shell.contains("flauz_agents_view::render_agents_panel"),
            "the Agents rail section mounts this module's panel"
        );
    }

    #[test]
    fn the_view_model_state_starts_honest_and_attaches_cleanly() {
        let mut state = AgentsViewState::new();
        assert!(state.detail().is_none());
        state.set_detail(AgentsViewDetail {
            nodes: vec![waiting_node()],
            merged: false,
        });
        let detail = state.detail().unwrap_or_else(|| panic!("attached"));
        assert_eq!(detail.nodes.len(), 1);
        assert_eq!(detail.nodes[0].state, AgentStateView::Waiting);
        assert!(!detail.merged);
        assert!(!detail.shows_parallel_progress());
    }
}
