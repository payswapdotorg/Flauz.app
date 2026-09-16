//! Universal Workflows view — GUI-003 vertical slice.
//!
//! Renders the complete Universal workflow lifecycle in the native GUI:
//! teach (demonstrate / instruct / hybrid) → reconcile → compile → review
//! → approve → publish an immutable version → run → inspect execution
//! evidence and durable state.
//!
//! Boundary rules (see `docs/codex-universal/CLIENT-BOUNDARY.md`):
//!
//! - every semantic value rendered here (ids, statuses, record counts,
//!   digests, validation findings) comes from a control-plane response;
//!   the view never derives or recomputes workflow semantics;
//! - identifiers are opaque control-plane strings displayed verbatim;
//! - every failure path (lifecycle gates, unknown ids, transport) is
//!   surfaced as an actionable error banner, never silently swallowed;
//! - teaching sessions and compiled candidates are ephemeral app-server
//!   state; the view says so instead of faking continuity.

use codex_core::{
    Action, LoadStatus, WorkflowCandidateState, WorkflowDemonstrationKind, WorkflowInstanceStatus,
    WorkflowState, WorkflowTeachMode, WorkflowTeachSessionState, WorkflowTeachSessionStatus,
    WorkflowValidationSeverity,
};

use codex_core::WorkflowImproveState;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, Context, Entity, FontWeight, Hsla, IntoElement, SharedString, div, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    scroll::ScrollableElement,
    v_flex,
};

use super::WorkspaceView;

/// Panel width for the bounded workflow surface.
const WORKFLOW_PANE_WIDTH: f32 = 720.0;

pub(super) fn render_workflows(
    workspace: &mut WorkspaceView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let state = workspace.state.workflow.clone();
    let name_input = workspace.workflow_name.clone();
    let instruction_input = workspace.workflow_instruction.clone();
    let demonstration_input = workspace.workflow_demonstration.clone();
    let approver_input = workspace.workflow_approver.clone();
    let reference_input = workspace.workflow_reference.clone();
    let commit_input = workspace.workflow_commit_sha.clone();
    let repository_input = workspace.workflow_repository.clone();
    let semver_input = workspace.workflow_semver.clone();
    let selected_mode = workspace.workflow_mode;
    let selected_kind = workspace.workflow_demonstration_kind;
    let name_empty = workspace.workflow_name.read(cx).value().trim().is_empty();
    let instruction_empty = workspace
        .workflow_instruction
        .read(cx)
        .value()
        .trim()
        .is_empty();
    let demonstration_empty = workspace
        .workflow_demonstration
        .read(cx)
        .value()
        .trim()
        .is_empty();
    let approver_empty = workspace
        .workflow_approver
        .read(cx)
        .value()
        .trim()
        .is_empty();
    let reference_empty = workspace
        .workflow_reference
        .read(cx)
        .value()
        .trim()
        .is_empty();
    let commit_empty = workspace
        .workflow_commit_sha
        .read(cx)
        .value()
        .trim()
        .is_empty();
    let fork_repository_input = workspace.workflow_fork_repository.clone();
    let improve_semver_input = workspace.workflow_improve_semver.clone();
    let improve_approver_input = workspace.workflow_improve_approver.clone();
    let improve_release_tag_input = workspace.workflow_improve_release_tag.clone();
    let fork_repository_empty = workspace
        .workflow_fork_repository
        .read(cx)
        .value()
        .trim()
        .is_empty();

    v_flex()
        .flex_1()
        .h_full()
        .min_w_0()
        .child(
            h_flex()
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
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Workflows"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                    "Teach, compile, approve, and publish Universal workflows \
                                     through the Codex control plane",
                                ),
                        ),
                )
                .child(
                    Button::new("workflow-refresh-instances")
                        .label(if state.instances_status == LoadStatus::Loading {
                            "Refreshing…"
                        } else {
                            "Refresh instances"
                        })
                        .icon(IconName::Redo)
                        .small()
                        .ghost()
                        .disabled(state.instances_status == LoadStatus::Loading)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.dispatch(Action::WorkflowRefreshInstances, cx);
                        })),
                ),
        )
        .child(
            v_flex()
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .px_6()
                .py_4()
                .gap_4()
                .when_some(state.error.clone(), |column, error| {
                    column.child(render_error_banner(&error, cx))
                })
                .child(render_teaching_pane(
                    &state,
                    &name_input,
                    &instruction_input,
                    &demonstration_input,
                    selected_mode,
                    selected_kind,
                    name_empty,
                    instruction_empty,
                    demonstration_empty,
                    cx,
                ))
                .when(state.candidate.is_some(), |column| {
                    column.child(render_candidate_pane(
                        state.candidate.as_ref(),
                        &approver_input,
                        &reference_input,
                        &commit_input,
                        &repository_input,
                        &semver_input,
                        approver_empty,
                        reference_empty,
                        commit_empty,
                        cx,
                    ))
                })
                .child(render_versions_pane(
                    &state,
                    &fork_repository_input,
                    fork_repository_empty,
                    cx,
                ))
                .when(state.improve.is_some(), |column| {
                    column.child(render_improve_pane(
                        state.improve.as_ref(),
                        &improve_semver_input,
                        &improve_approver_input,
                        &improve_release_tag_input,
                        cx,
                    ))
                })
                .child(render_instances_pane(&state, cx)),
        )
        .into_any_element()
}

fn render_error_banner(message: &str, cx: &mut Context<WorkspaceView>) -> AnyElement {
    h_flex()
        .items_start()
        .gap_2()
        .rounded_lg()
        .bg(cx.theme().secondary)
        .px_3()
        .py_2()
        .child(
            Icon::new(IconName::TriangleAlert)
                .small()
                .text_color(cx.theme().danger),
        )
        .child(
            div()
                .flex_1()
                .text_sm()
                .text_color(cx.theme().danger)
                .child(message.to_owned()),
        )
        .child(
            Button::new("workflow-dismiss-error")
                .label("Dismiss")
                .small()
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.dispatch(Action::WorkflowDismissError, cx);
                })),
        )
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_teaching_pane(
    state: &WorkflowState,
    name_input: &Entity<InputState>,
    instruction_input: &Entity<InputState>,
    demonstration_input: &Entity<InputState>,
    selected_mode: WorkflowTeachMode,
    selected_kind: WorkflowDemonstrationKind,
    name_empty: bool,
    instruction_empty: bool,
    demonstration_empty: bool,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    v_flex()
        .w(px(WORKFLOW_PANE_WIDTH))
        .max_w_full()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Teach a workflow"),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "Describe the workflow in instruction statements, demonstrate it with \
                     events, or mix both. Reconciling closes the session and freezes the \
                     trajectory for compilation.",
                ),
        )
        // Mode picker + workflow name.
        .child(
            h_flex()
                .gap_2()
                .child(
                    h_flex()
                        .gap_1()
                        .rounded_lg()
                        .bg(cx.theme().secondary)
                        .p_1()
                        .child(
                            Button::new("workflow-mode-demonstrate")
                                .label(workflow_mode_label(WorkflowTeachMode::Demonstrate))
                                .xsmall()
                                .ghost()
                                .selected(selected_mode == WorkflowTeachMode::Demonstrate)
                                .disabled(state.teach.is_some())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.workflow_mode = WorkflowTeachMode::Demonstrate;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("workflow-mode-instruct")
                                .label(workflow_mode_label(WorkflowTeachMode::Instruct))
                                .xsmall()
                                .ghost()
                                .selected(selected_mode == WorkflowTeachMode::Instruct)
                                .disabled(state.teach.is_some())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.workflow_mode = WorkflowTeachMode::Instruct;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("workflow-mode-hybrid")
                                .label(workflow_mode_label(WorkflowTeachMode::Hybrid))
                                .xsmall()
                                .ghost()
                                .selected(selected_mode == WorkflowTeachMode::Hybrid)
                                .disabled(state.teach.is_some())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.workflow_mode = WorkflowTeachMode::Hybrid;
                                    cx.notify();
                                })),
                        ),
                )
                .child(Input::new(name_input).disabled(state.teach.is_some()))
                .child(
                    Button::new("workflow-teach-start")
                        .label(if state.teach_pending && state.teach.is_none() {
                            "Starting…"
                        } else {
                            "New workflow"
                        })
                        .small()
                        .primary()
                        .disabled(state.teach.is_some() || state.teach_pending || name_empty)
                        .on_click(cx.listener(|this, _, _, cx| {
                            let mode = this.workflow_mode;
                            let name = this.workflow_name.read(cx).value().trim().to_owned();
                            this.dispatch(Action::WorkflowTeachStart { mode, name }, cx);
                        })),
                ),
        )
        .when_some(state.teach.clone(), |pane, session| {
            pane.child(render_teaching_session(
                &session,
                state.teach_pending,
                instruction_input,
                demonstration_input,
                selected_kind,
                instruction_empty,
                demonstration_empty,
                state.compile_pending,
                cx,
            ))
        })
        .when(state.teach.is_none(), |pane| {
            pane.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "Teaching sessions and compiled candidates live in the Codex runtime \
                         and are cleared when it restarts. Published versions and durable \
                         instances persist.",
                    ),
            )
        })
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_teaching_session(
    session: &WorkflowTeachSessionState,
    teach_pending: bool,
    instruction_input: &Entity<InputState>,
    demonstration_input: &Entity<InputState>,
    selected_kind: WorkflowDemonstrationKind,
    instruction_empty: bool,
    demonstration_empty: bool,
    compile_pending: bool,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let session_open = session.status == WorkflowTeachSessionStatus::Open;
    v_flex()
        .gap_3()
        .rounded_lg()
        .bg(cx.theme().secondary)
        .p_3()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(session.name.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "{} · {} · session {} · {} records · last sequence {}",
                            workflow_mode_label(session.mode),
                            workflow_teach_status_label(session.status),
                            session.session_id,
                            session.record_count,
                            session.last_sequence,
                        )),
                )
                .when_some(
                    session
                        .instruction_records
                        .zip(session.demonstration_records),
                    |row, (instructions, demonstrations)| {
                        row.child(
                            div().text_xs().text_color(cx.theme().muted_foreground).child(
                                format!("reconciled: {instructions} instructions · {demonstrations} demonstrations"),
                            ),
                        )
                    },
                ),
        )
        .when(session_open, |pane| {
            pane.child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Instructions"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("sequence numbers are zero-based"),
                            ),
                    )
                    .child(
                        h_flex().gap_2().child(
                            Input::new(instruction_input)
                                .disabled(teach_pending),
                        )
                        .child(
                            Button::new("workflow-record-instruction")
                                .label("Record instruction")
                                .small()
                                .primary()
                                .disabled(teach_pending || instruction_empty)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let text = this
                                        .workflow_instruction
                                        .read(cx)
                                        .value()
                                        .trim()
                                        .to_owned();
                                    this.dispatch(Action::WorkflowTeachInstruct(text), cx);
                                    this.workflow_instruction.update(cx, |input, cx| {
                                        input.set_value("", window, cx);
                                    });
                                })),
                        ),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Demonstrations"),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .rounded_md()
                                    .bg(cx.theme().popover)
                                    .p_1()
                                    .child(
                                        Button::new("workflow-demo-observation")
                                            .label(workflow_demo_kind_label(
                                                WorkflowDemonstrationKind::Observation,
                                            ))
                                            .xsmall()
                                            .ghost()
                                            .selected(
                                                selected_kind
                                                    == WorkflowDemonstrationKind::Observation,
                                            )
                                            .disabled(teach_pending)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.workflow_demonstration_kind =
                                                    WorkflowDemonstrationKind::Observation;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("workflow-demo-action")
                                            .label(workflow_demo_kind_label(
                                                WorkflowDemonstrationKind::Action,
                                            ))
                                            .xsmall()
                                            .ghost()
                                            .selected(
                                                selected_kind
                                                    == WorkflowDemonstrationKind::Action,
                                            )
                                            .disabled(teach_pending)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.workflow_demonstration_kind =
                                                    WorkflowDemonstrationKind::Action;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("workflow-demo-result")
                                            .label(workflow_demo_kind_label(
                                                WorkflowDemonstrationKind::Result,
                                            ))
                                            .xsmall()
                                            .ghost()
                                            .selected(
                                                selected_kind
                                                    == WorkflowDemonstrationKind::Result,
                                            )
                                            .disabled(teach_pending)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.workflow_demonstration_kind =
                                                    WorkflowDemonstrationKind::Result;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("workflow-demo-recovery")
                                            .label(workflow_demo_kind_label(
                                                WorkflowDemonstrationKind::Recovery,
                                            ))
                                            .xsmall()
                                            .ghost()
                                            .selected(
                                                selected_kind
                                                    == WorkflowDemonstrationKind::Recovery,
                                            )
                                            .disabled(teach_pending)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.workflow_demonstration_kind =
                                                    WorkflowDemonstrationKind::Recovery;
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Input::new(demonstration_input)
                                    .disabled(teach_pending),
                            )
                            .child(
                                Button::new("workflow-record-demonstration")
                                    .label("Record demonstration")
                                    .small()
                                    .primary()
                                    .disabled(teach_pending || demonstration_empty)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        let kind = this.workflow_demonstration_kind;
                                        let text = this
                                            .workflow_demonstration
                                            .read(cx)
                                            .value()
                                            .trim()
                                            .to_owned();
                                        this.dispatch(
                                            Action::WorkflowTeachDemonstrate { kind, text },
                                            cx,
                                        );
                                        this.workflow_demonstration.update(cx, |input, cx| {
                                            input.set_value("", window, cx);
                                        });
                                    })),
                            ),
                    ),
            )
            .child(
                h_flex().gap_2().child(
                    Button::new("workflow-reconcile")
                        .label(if teach_pending {
                            "Working…"
                        } else {
                            "Reconcile (close session)"
                        })
                        .small()
                        .disabled(teach_pending)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.dispatch(Action::WorkflowTeachReconcile, cx);
                        })),
                ),
            )
        })
        .when(!session_open, |pane| {
            pane.child(
                h_flex().gap_2().child(
                    Button::new("workflow-compile")
                        .label(if compile_pending {
                            "Compiling…"
                        } else {
                            "Compile candidate"
                        })
                        .small()
                        .primary()
                        .disabled(compile_pending)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.dispatch(Action::WorkflowCompile, cx);
                        })),
                ),
            )
        })
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_candidate_pane(
    candidate: Option<&WorkflowCandidateState>,
    approver_input: &Entity<InputState>,
    reference_input: &Entity<InputState>,
    commit_input: &Entity<InputState>,
    repository_input: &Entity<InputState>,
    semver_input: &Entity<InputState>,
    approver_empty: bool,
    reference_empty: bool,
    commit_empty: bool,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let Some(candidate) = candidate else {
        return div().into_any_element();
    };
    v_flex()
        .w(px(WORKFLOW_PANE_WIDTH))
        .max_w_full()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Candidate review"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "candidate {} · {} · epoch {} · {} steps",
                            candidate.candidate_id,
                            workflow_candidate_status_label(candidate.status),
                            candidate.epoch,
                            candidate.step_count,
                        )),
                ),
        )
        .child(
            h_flex().gap_2().child(
                Button::new("workflow-review")
                    .label("Review steps")
                    .small()
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.dispatch(Action::WorkflowReview, cx);
                    })),
            ),
        )
        .when(!candidate.steps.is_empty(), |pane| {
            pane.child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Steps"),
                    )
                    .children(candidate.steps.iter().map(|step| {
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_color(cx.theme().muted_foreground)
                                    .child(step.node_id.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(workflow_step_origin_label(step.origin)),
                            )
                            .child(
                                div().text_sm().child(
                                    step.description
                                        .clone()
                                        .unwrap_or_else(|| "(no recorded intent)".to_owned()),
                                ),
                            )
                            .when(step.evidence_count > 0, |row| {
                                row.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("{} evidence", step.evidence_count)),
                                )
                            })
                    })),
            )
        })
        .when_some(candidate.description.clone(), |pane, description| {
            pane.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(description),
            )
        })
        .when(!candidate.capability_hints.is_empty(), |pane| {
            pane.child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Capability hints"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                "Advisory hints derived by the teaching compiler; binding \
                                 decisions belong to the execution plane.",
                            ),
                    )
                    .children(candidate.capability_hints.iter().map(|hint| {
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_color(cx.theme().muted_foreground)
                                    .child(hint.node_id.clone()),
                            )
                            .child(div().text_sm().child(hint.hint.clone()))
                            .when(!hint.rationale.is_empty(), |row| {
                                row.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(hint.rationale.clone()),
                                )
                            })
                    })),
            )
        })
        .when(!candidate.binding_proposals.is_empty(), |pane| {
            pane.child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Binding proposals"),
                    )
                    .children(candidate.binding_proposals.iter().map(|proposal| {
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_color(cx.theme().muted_foreground)
                                    .child(proposal.node_id.clone()),
                            )
                            .child(div().text_sm().child(format!(
                                "{} → {}{}",
                                proposal.kind,
                                proposal.reference,
                                if proposal.requires_approval {
                                    " · requires approval"
                                } else {
                                    ""
                                },
                            )))
                    })),
            )
        })
        .when(!candidate.trigger_intents.is_empty(), |pane| {
            pane.child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Trigger intents"),
                    )
                    .children(candidate.trigger_intents.iter().map(|intent| {
                        h_flex().gap_2().child(
                            div()
                                .text_sm()
                                .child(format!("[{}] {}", intent.class_hint, intent.description,)),
                        )
                    })),
            )
        })
        .child(
            v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child("Validation"),
                        )
                        .when_some(candidate.simulation_outcome.clone(), |row, outcome| {
                            row.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "simulation {outcome} · {} steps taken{}",
                                        candidate.simulation_steps_taken,
                                        candidate
                                            .simulation_node
                                            .as_deref()
                                            .map(|node| format!(" · stopped at {node}"))
                                            .unwrap_or_default(),
                                    )),
                            )
                        }),
                )
                .child(div().text_sm().child(format!(
                    "{} errors · {} warnings · {}",
                    candidate.validation.error_count,
                    candidate.validation.warning_count,
                    if candidate.validation.clean {
                        "clean"
                    } else {
                        "not clean"
                    },
                )))
                .children(candidate.validation.findings.iter().map(|finding| {
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(
                                    if finding.severity == WorkflowValidationSeverity::Error {
                                        cx.theme().danger
                                    } else {
                                        cx.theme().warning
                                    },
                                )
                                .child(workflow_severity_label(finding.severity)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_color(cx.theme().muted_foreground)
                                .child(finding.code.clone()),
                        )
                        .child(div().text_sm().child(finding.message.clone()))
                })),
        )
        .child(
            v_flex()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .child("Approve"),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(Input::new(approver_input))
                        .child(Input::new(reference_input)),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("workflow-approve")
                                .label("Approve")
                                .small()
                                .primary()
                                .disabled(approver_empty || reference_empty)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let approver =
                                        this.workflow_approver.read(cx).value().trim().to_owned();
                                    let reference =
                                        this.workflow_reference.read(cx).value().trim().to_owned();
                                    this.dispatch(
                                        Action::WorkflowApprove {
                                            approver,
                                            reference,
                                            decision:
                                                codex_core::WorkflowApprovalDecision::Approved,
                                        },
                                        cx,
                                    );
                                })),
                        )
                        .child(
                            Button::new("workflow-reject")
                                .label("Reject")
                                .small()
                                .ghost()
                                .disabled(approver_empty || reference_empty)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let approver =
                                        this.workflow_approver.read(cx).value().trim().to_owned();
                                    let reference =
                                        this.workflow_reference.read(cx).value().trim().to_owned();
                                    this.dispatch(
                                        Action::WorkflowApprove {
                                            approver,
                                            reference,
                                            decision:
                                                codex_core::WorkflowApprovalDecision::Rejected,
                                        },
                                        cx,
                                    );
                                })),
                        ),
                ),
        )
        .child(
            v_flex()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .child("Publish immutable version"),
                )
                .child(Input::new(commit_input))
                .child(
                    h_flex()
                        .gap_2()
                        .child(Input::new(repository_input))
                        .child(Input::new(semver_input)),
                )
                .child(
                    h_flex().gap_2().child(
                        Button::new("workflow-publish")
                            .label("Publish version")
                            .small()
                            .primary()
                            .disabled(commit_empty)
                            .on_click(cx.listener(|this, _, _, cx| {
                                let commit_sha =
                                    this.workflow_commit_sha.read(cx).value().trim().to_owned();
                                let repository =
                                    this.workflow_repository.read(cx).value().trim().to_owned();
                                let semantic_version =
                                    this.workflow_semver.read(cx).value().trim().to_owned();
                                this.dispatch(
                                    Action::WorkflowPublish {
                                        commit_sha,
                                        repository,
                                        semantic_version,
                                    },
                                    cx,
                                );
                            })),
                    ),
                ),
        )
        .into_any_element()
}

fn render_versions_pane(
    state: &WorkflowState,
    fork_repository_input: &Entity<InputState>,
    fork_repository_empty: bool,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    v_flex()
        .w(px(WORKFLOW_PANE_WIDTH))
        .max_w_full()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Published versions"),
        )
        .when(state.published.is_empty(), |pane| {
            pane.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "No published versions yet. Teach a workflow, compile and approve \
                         the candidate, then publish it anchored at a commit.",
                    ),
            )
        })
        .children(state.published.iter().map(|version| {
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .flex_1()
                        .min_w_0()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(version.workflow.clone()),
                                )
                                .child(div().text_sm().child(version.semantic_version.clone())),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_color(cx.theme().muted_foreground)
                                        .child(version.version_id.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!(
                                            "commit {} · repo {}",
                                            version.commit_sha,
                                            if version.repository.is_empty() {
                                                "(default)"
                                            } else {
                                                version.repository.as_str()
                                            }
                                        )),
                                ),
                        )
                        .when_some(
                            version.binding_resolution.as_ref(),
                            |rows, resolution| {
                                rows.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!(
                                            "binding resolution approved {} → executable {}",
                                            resolution.approved_digest,
                                            resolution.executable_digest
                                        )),
                                )
                            },
                        ),
                )
                .child(
                    Button::new(SharedString::from(format!(
                        "workflow-run-{}",
                        version.version_id
                    )))
                    .label(
                        if state.run_pending.as_deref() == Some(version.version_id.as_str()) {
                            "Running…"
                        } else {
                            "Run"
                        },
                    )
                    .small()
                    .primary()
                    .disabled(state.run_pending.is_some())
                    .on_click(cx.listener({
                        let version_id = version.version_id.clone();
                        move |this, _, _, cx| {
                            this.dispatch(
                                Action::WorkflowInstanceRun {
                                    version_id: version_id.clone(),
                                },
                                cx,
                            );
                        }
                    })),
                )
                .child(
                    Button::new(SharedString::from(format!(
                        "workflow-fork-{}",
                        version.version_id
                    )))
                    .label(if state.fork_pending.as_deref() == Some(version.version_id.as_str()) {
                        "Forking…"
                    } else {
                        "Fork"
                    })
                    .small()
                    .ghost()
                    .disabled(
                        state.fork_pending.is_some()
                            || fork_repository_empty
                            || state.improve_propose_pending.is_some(),
                    )
                    .on_click(cx.listener({
                        let version_id = version.version_id.clone();
                        move |this, _, _, cx| {
                            let fork_repository = this
                                .workflow_fork_repository
                                .read(cx)
                                .value()
                                .trim()
                                .to_owned();
                            this.dispatch(
                                Action::WorkflowForkVersion {
                                    version_id: version_id.clone(),
                                    fork_repository,
                                },
                                cx,
                            );
                        }
                    })),
                )
                .child(
                    Button::new(SharedString::from(format!(
                        "workflow-improve-{}",
                        version.version_id
                    )))
                    .label(
                        if state.improve_propose_pending.as_deref()
                            == Some(version.version_id.as_str())
                        {
                            "Proposing…"
                        } else {
                            "Improve"
                        },
                    )
                    .small()
                    .ghost()
                    .disabled(
                        state.improve_propose_pending.is_some()
                            || state.fork_pending.is_some(),
                    )
                    .on_click(cx.listener({
                        let version_id = version.version_id.clone();
                        move |this, _, _, cx| {
                            this.dispatch(
                                Action::WorkflowImproveStart {
                                    version_id: version_id.clone(),
                                },
                                cx,
                            );
                        }
                    })),
                )
        }))
        .when(!state.published.is_empty(), |pane| {
            pane.child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Fork a published version"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        "Derives a new immutable release carrying the upstream                                          lineage; the fork repository identity must differ from                                          the upstream's.",
                                    ),
                            ),
                    )
                    .child(
                        h_flex().gap_2().child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(
                                    Input::new(fork_repository_input),
                                ),
                        ),
                    ),
            )
        })
        .when_some(state.fork_lineage.clone(), |pane, lineage| {
            pane.child(
                v_flex()
                    .gap_1()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Latest fork lineage (engine-sealed)"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "forked from {} {} ({}) · commit {} · definition {} · lock {}",
                                lineage.workflow,
                                lineage.semantic_version,
                                lineage.repository,
                                lineage.commit_sha,
                                lineage.definition_digest,
                                lineage.dependency_lock_digest,
                            )),
                    ),
            )
        })
        .into_any_element()
}

fn render_improve_pane(
    improve: Option<&WorkflowImproveState>,
    semver_input: &Entity<InputState>,
    approver_input: &Entity<InputState>,
    release_tag_input: &Entity<InputState>,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let Some(improve) = improve else {
        return div().into_any_element();
    };
    let selected = improve
        .selected_candidate_id
        .as_deref()
        .and_then(|candidate_id| {
            improve
                .candidates
                .iter()
                .find(|candidate| candidate.candidate_id == candidate_id)
        });
    v_flex()
        .w(px(WORKFLOW_PANE_WIDTH))
        .max_w_full()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Improve workflow"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "incumbent {} · governed evolution: propose → validate → approve → publish",
                            improve.incumbent_semantic_version,
                        )),
                )
                .child(
                    Button::new("workflow-improve-dismiss")
                        .label("Close")
                        .small()
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.dispatch(Action::WorkflowImproveDismiss, cx);
                        })),
                ),
        )
        .when(improve.candidates.is_empty(), |pane| {
            pane.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "No improvement candidates were supported by the recorded execution \
                         evidence. Run the workflow to record evidence first.",
                    ),
            )
        })
        .children(improve.candidates.iter().map(|candidate| {
            let candidate_selected = improve.selected_candidate_id.as_deref()
                == Some(candidate.candidate_id.as_str());
            v_flex()
                .gap_1()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .p_3()
                .when(candidate_selected, |card| {
                    card.border_color(cx.theme().ring)
                })
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Button::new(SharedString::from(format!(
                                "workflow-improve-select-{}",
                                candidate.candidate_id
                            )))
                            .label(if candidate_selected {
                                "Selected"
                            } else {
                                "Select"
                            })
                            .small()
                            .ghost()
                            .disabled(candidate_selected)
                            .on_click(cx.listener({
                                let candidate_id = candidate.candidate_id.clone();
                                move |this, _, _, cx| {
                                    this.dispatch(
                                        Action::WorkflowImproveSelectCandidate {
                                            candidate_id: candidate_id.clone(),
                                        },
                                        cx,
                                    );
                                }
                            })),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child(candidate.change_kind.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "candidate {} · {} evidence refs · {} cited runs",
                                    candidate.candidate_id,
                                    candidate.evidence.references.len(),
                                    candidate.evidence.runs.len(),
                                )),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(candidate.rationale.clone()),
                )
        }))
        .when_some(selected, |pane, _candidate| {
            pane.child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Validate the selected candidate"),
                            )
                            .child(
                                div()
                                    .w(px(220.0))
                                    .child(
                                        Input::new(semver_input),
                                    ),
                            )
                            .child(
                                Button::new("workflow-improve-validate")
                                    .label(if improve.validate_pending.is_some() {
                                        "Validating…"
                                    } else {
                                        "Validate"
                                    })
                                    .small()
                                    .disabled(improve.validate_pending.is_some())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let successor_version = this
                                            .workflow_improve_semver
                                            .read(cx)
                                            .value()
                                            .trim()
                                            .to_owned();
                                        this.dispatch(
                                            Action::WorkflowImproveValidate { successor_version },
                                            cx,
                                        );
                                    })),
                            ),
                    ),
            )
        })
        .when_some(improve.validated.as_ref(), |pane, validated| {
            pane.child(
                v_flex()
                    .gap_1()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_3()
                    .child(
                        h_flex().gap_2().child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child(format!(
                                    "Validation {} · successor {}",
                                    if validated.passed { "passed" } else { "failed" },
                                    validated.successor_version,
                                )),
                        ),
                    )
                    .children(validated.stages.iter().map(|stage| {
                        div()
                            .text_xs()
                            .text_color(if stage.passed {
                                cx.theme().muted_foreground
                            } else {
                                cx.theme().danger
                            })
                            .child(format!(
                                "{}: {}",
                                stage.stage,
                                if stage.passed { "passed" } else { "failed" },
                            ))
                    })),
            )
        })
        .when_some(improve.validated.as_ref(), |pane, _| {
            pane.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .w(px(220.0))
                            .child(Input::new(approver_input)),
                    )
                    .child(
                        Button::new("workflow-improve-approve")
                            .label(if improve.approve_pending.is_some() {
                                "Recording…"
                            } else {
                                "Approve"
                            })
                            .small()
                            .primary()
                            .disabled(improve.approve_pending.is_some())
                            .on_click(cx.listener(|this, _, _, cx| {
                                let approver = this
                                    .workflow_improve_approver
                                    .read(cx)
                                    .value()
                                    .trim()
                                    .to_owned();
                                this.dispatch(
                                    Action::WorkflowImproveApprove {
                                        approver,
                                        decision: codex_core::WorkflowApprovalDecision::Approved,
                                        note: String::new(),
                                        reason: String::new(),
                                    },
                                    cx,
                                );
                            })),
                    )
                    .child(
                        Button::new("workflow-improve-reject")
                            .label("Reject")
                            .small()
                            .ghost()
                            .disabled(improve.approve_pending.is_some())
                            .on_click(cx.listener(|this, _, _, cx| {
                                let approver = this
                                    .workflow_improve_approver
                                    .read(cx)
                                    .value()
                                    .trim()
                                    .to_owned();
                                this.dispatch(
                                    Action::WorkflowImproveApprove {
                                        approver,
                                        decision: codex_core::WorkflowApprovalDecision::Rejected,
                                        note: String::new(),
                                        reason: "Rejected from the Flauz.app improve surface"
                                            .to_owned(),
                                    },
                                    cx,
                                );
                            })),
                    ),
            )
        })
        .when_some(improve.approved.as_ref(), |pane, approved| {
            pane.child(
                div()
                    .text_sm()
                    .text_color(if approved.approved {
                        cx.theme().muted_foreground
                    } else {
                        cx.theme().danger
                    })
                    .child(format!(
                        "decision recorded by {} · validation {}",
                        approved.approver,
                        approved.validation_digest,
                    )),
            )
        })
        .when_some(
            improve
                .approved
                .as_ref()
                .filter(|approved| approved.approved),
            |pane, _| {
                pane.child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w(px(220.0))
                                .child(
                                    Input::new(release_tag_input),
                                ),
                        )
                        .child(
                            Button::new("workflow-improve-publish")
                                .label(if improve.publish_pending.is_some() {
                                    "Publishing…"
                                } else {
                                    "Publish successor"
                                })
                                .small()
                                .primary()
                                .disabled(improve.publish_pending.is_some())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let release_tag = this
                                        .workflow_improve_release_tag
                                        .read(cx)
                                        .value()
                                        .trim()
                                        .to_owned();
                                    this.dispatch(
                                        Action::WorkflowImprovePublish { release_tag },
                                        cx,
                                    );
                                })),
                        ),
                )
            },
        )
        .when_some(improve.published.as_ref(), |pane, published| {
            pane.child(
                v_flex()
                    .gap_1()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child(format!(
                                "Published successor {} ({})",
                                published.semantic_version, published.release_tag,
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "lineage: {} {} → {} · candidate {} · approver {} · {} stages",
                                published.lineage.workflow,
                                published.lineage.predecessor_semantic_version,
                                published.lineage.successor_semantic_version,
                                published.lineage.candidate_id,
                                published.lineage.approver,
                                published.lineage.validation_stages.len(),
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "cites {} evidence references and {} recorded runs",
                                published.evidence.references.len(),
                                published.evidence.runs.len(),
                            )),
                    ),
            )
        })
        .into_any_element()
}

fn render_instances_pane(state: &WorkflowState, cx: &mut Context<WorkspaceView>) -> AnyElement {
    v_flex()
        .w(px(WORKFLOW_PANE_WIDTH))
        .max_w_full()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Durable instances"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(match state.instances_status {
                            LoadStatus::Idle => "not loaded",
                            LoadStatus::Loading => "loading…",
                            LoadStatus::Ready => "loaded",
                            LoadStatus::Failed => "failed to load",
                        }),
                ),
        )
        .when(state.instances.is_empty(), |pane| {
            pane.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(match state.instances_status {
                        LoadStatus::Loading => "Loading durable instances…",
                        _ => {
                            "No durable instances. Published versions run as durable \
                              instances that survive restarts."
                        }
                    }),
            )
        })
        .children(
            group_instances_by_version(&state.instances)
                .into_iter()
                .map(|(version_id, instances)| {
                    v_flex()
                        .gap_2()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_color(cx.theme().muted_foreground)
                                        .child(version_id),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!(
                                            "{} run{}",
                                            instances.len(),
                                            if instances.len() == 1 { "" } else { "s" }
                                        )),
                                ),
                        )
                        .children(instances.into_iter().map(|instance| {
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .flex_1()
                                        .min_w_0()
                                        .child(
                                            h_flex()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_family(
                                                            cx.theme().mono_font_family.clone(),
                                                        )
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(instance.instance_id.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .child(instance.workflow.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(workflow_instance_status_color(
                                                            instance.status,
                                                            cx,
                                                        ))
                                                        .child(workflow_instance_status_label(
                                                            instance.status,
                                                        )),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!(
                                                    "version {} · {} steps taken",
                                                    instance.version_id, instance.steps_taken,
                                                )),
                                        ),
                                )
                                .child(
                                    Button::new(SharedString::from(format!(
                                        "workflow-inspect-{}",
                                        instance.instance_id
                                    )))
                                    .label(
                                        if state.instance_get_pending.as_deref()
                                            == Some(instance.instance_id.as_str())
                                        {
                                            "Inspecting…"
                                        } else {
                                            "Inspect"
                                        },
                                    )
                                    .small()
                                    .ghost()
                                    .disabled(state.instance_get_pending.is_some())
                                    .on_click(cx.listener({
                                        let instance_id = instance.instance_id.clone();
                                        move |this, _, _, cx| {
                                            this.dispatch(
                                                Action::WorkflowInstanceGet {
                                                    instance_id: instance_id.clone(),
                                                },
                                                cx,
                                            );
                                        }
                                    })),
                                )
                                .child(
                                    Button::new(SharedString::from(format!(
                                        "workflow-resume-{}",
                                        instance.instance_id
                                    )))
                                    .label(
                                        if state.instance_resume_pending.as_deref()
                                            == Some(instance.instance_id.as_str())
                                        {
                                            "Resuming…"
                                        } else {
                                            "Resume"
                                        },
                                    )
                                    .small()
                                    .ghost()
                                    .disabled(
                                        state.instance_resume_pending.is_some()
                                            || instance.status != WorkflowInstanceStatus::Paused,
                                    )
                                    .on_click(cx.listener({
                                        let instance_id = instance.instance_id.clone();
                                        move |this, _, _, cx| {
                                            this.dispatch(
                                                Action::WorkflowInstanceResume {
                                                    instance_id: instance_id.clone(),
                                                },
                                                cx,
                                            );
                                        }
                                    })),
                                )
                                .child(
                                    Button::new(SharedString::from(format!(
                                        "workflow-cancel-{}",
                                        instance.instance_id
                                    )))
                                    .label(
                                        if state.instance_cancel_pending.as_deref()
                                            == Some(instance.instance_id.as_str())
                                        {
                                            "Cancelling…"
                                        } else {
                                            "Cancel"
                                        },
                                    )
                                    .small()
                                    .ghost()
                                    .disabled(
                                        state.instance_cancel_pending.is_some()
                                            || !matches!(
                                                instance.status,
                                                WorkflowInstanceStatus::Pending
                                                    | WorkflowInstanceStatus::Running
                                                    | WorkflowInstanceStatus::Paused
                                            ),
                                    )
                                    .on_click(cx.listener({
                                        let instance_id = instance.instance_id.clone();
                                        move |this, _, _, cx| {
                                            this.dispatch(
                                                Action::WorkflowInstanceCancel {
                                                    instance_id: instance_id.clone(),
                                                    reason: "Cancelled from the Flauz.app                                                              workflows surface"
                                                        .to_owned(),
                                                },
                                                cx,
                                            );
                                        }
                                    })),
                                )
                        }))
                }),
        )
        .when_some(state.instance_detail.clone(), |pane, detail| {
            pane.child(
                v_flex()
                    .gap_2()
                    .rounded_lg()
                    .bg(cx.theme().secondary)
                    .p_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child(format!(
                                "Instance {} — execution and evidence",
                                detail.instance.instance_id
                            )),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{} · version {}",
                                        workflow_instance_status_label(detail.instance.status),
                                        detail.instance.version_id,
                                    )),
                            )
                            .when_some(detail.terminal_kind.clone(), |row, terminal| {
                                row.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!(
                                            "terminal {}{}",
                                            terminal,
                                            detail
                                                .terminal_reason
                                                .as_deref()
                                                .map(|reason| format!(": {reason}"))
                                                .unwrap_or_default(),
                                        )),
                                )
                            }),
                    )
                    .when(!detail.path.is_empty(), |block| {
                        block.child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("Execution timeline"),
                                )
                                .children(detail.path.iter().map(|node| {
                                    div()
                                        .text_xs()
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("• {node}"))
                                })),
                        )
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "environments in evidence: {}",
                                workflow_environment_summary(&detail.evidence),
                            )),
                    )
                    .when(detail.evidence.is_empty(), |block| {
                        block.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("No evidence references recorded on this instance."),
                        )
                    })
                    .when(!detail.evidence.is_empty(), |block| {
                        let mut rows: Vec<AnyElement> = Vec::new();
                        let mut previous_environment: Option<&str> = None;
                        for (index, reference) in detail.evidence.iter().enumerate() {
                            let environment =
                                codex_core::workflow_evidence_environment(&reference.kind);
                            // A boundary crossing is only claimed between two
                            // environments the control plane actually labeled;
                            // unrecognized kinds never produce one.
                            if let (Some(previous), Some(current)) =
                                (previous_environment, environment)
                                && previous != current
                            {
                                rows.push(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!(
                                            "── environment change: {previous} → {current} ──"
                                        ))
                                        .into_any_element(),
                                );
                            }
                            previous_environment = environment;
                            rows.push(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!("#{}", index + 1)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .child(
                                                environment
                                                    .unwrap_or("Unrecognized kind")
                                                    .to_owned(),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!("kind {}", reference.kind)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_family(cx.theme().mono_font_family.clone())
                                            .text_color(cx.theme().muted_foreground)
                                            .child(reference.locator.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_family(cx.theme().mono_font_family.clone())
                                            .text_color(cx.theme().muted_foreground)
                                            .child(reference.digest.clone()),
                                    )
                                    .into_any_element(),
                            );
                        }
                        block.child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("Environment timeline"),
                                )
                                .children(rows),
                        )
                    }),
            )
        })
        .into_any_element()
}

fn workflow_mode_label(mode: WorkflowTeachMode) -> &'static str {
    match mode {
        WorkflowTeachMode::Demonstrate => "Demonstrate",
        WorkflowTeachMode::Instruct => "Instruct",
        WorkflowTeachMode::Hybrid => "Hybrid",
    }
}

fn workflow_teach_status_label(status: WorkflowTeachSessionStatus) -> &'static str {
    match status {
        WorkflowTeachSessionStatus::Open => "open",
        WorkflowTeachSessionStatus::Closed => "closed",
    }
}

fn workflow_demo_kind_label(kind: WorkflowDemonstrationKind) -> &'static str {
    match kind {
        WorkflowDemonstrationKind::Observation => "Observation",
        WorkflowDemonstrationKind::Action => "Action",
        WorkflowDemonstrationKind::Result => "Result",
        WorkflowDemonstrationKind::Recovery => "Recovery",
    }
}

fn workflow_candidate_status_label(status: codex_core::WorkflowCandidateStatus) -> &'static str {
    match status {
        codex_core::WorkflowCandidateStatus::Compiled => "compiled",
        codex_core::WorkflowCandidateStatus::Validated => "validated",
        codex_core::WorkflowCandidateStatus::Approved => "approved",
        codex_core::WorkflowCandidateStatus::PublicationReady => "publication ready",
    }
}

fn workflow_step_origin_label(origin: codex_core::WorkflowStepOrigin) -> &'static str {
    match origin {
        codex_core::WorkflowStepOrigin::Observed => "observed",
        codex_core::WorkflowStepOrigin::Instructed => "instructed",
    }
}

fn workflow_severity_label(severity: WorkflowValidationSeverity) -> &'static str {
    match severity {
        WorkflowValidationSeverity::Error => "error",
        WorkflowValidationSeverity::Warning => "warning",
    }
}

fn workflow_instance_status_label(status: WorkflowInstanceStatus) -> &'static str {
    match status {
        WorkflowInstanceStatus::Pending => "pending",
        WorkflowInstanceStatus::Running => "running",
        WorkflowInstanceStatus::Paused => "paused",
        WorkflowInstanceStatus::Succeeded => "succeeded",
        WorkflowInstanceStatus::Failed => "failed",
        WorkflowInstanceStatus::Cancelled => "cancelled",
    }
}

fn workflow_instance_status_color(status: WorkflowInstanceStatus, cx: &App) -> Hsla {
    match status {
        WorkflowInstanceStatus::Pending => cx.theme().muted_foreground,
        WorkflowInstanceStatus::Running => cx.theme().info,
        WorkflowInstanceStatus::Paused => cx.theme().warning,
        WorkflowInstanceStatus::Succeeded => cx.theme().success,
        WorkflowInstanceStatus::Failed => cx.theme().danger,
        WorkflowInstanceStatus::Cancelled => cx.theme().muted_foreground,
    }
}

/// Groups durable instances by their pinned immutable version, preserving
/// the control plane's instance order inside each group.
fn group_instances_by_version(
    instances: &[codex_core::WorkflowInstanceCard],
) -> Vec<(String, Vec<codex_core::WorkflowInstanceCard>)> {
    let mut groups: Vec<(String, Vec<codex_core::WorkflowInstanceCard>)> = Vec::new();
    for instance in instances {
        if let Some(group) = groups
            .iter_mut()
            .find(|(version_id, _)| *version_id == instance.version_id)
        {
            group.1.push(instance.clone());
        } else {
            groups.push((instance.version_id.clone(), vec![instance.clone()]));
        }
    }
    groups
}

/// Renders the distinct environment classes present in an instance's
/// evidence, or an honest placeholder when none are recorded.
fn workflow_environment_summary(evidence: &[codex_core::WorkflowEvidenceCard]) -> String {
    let mut environments: Vec<&str> = Vec::new();
    for reference in evidence {
        if let Some(environment) = codex_core::workflow_evidence_environment(&reference.kind)
            && !environments.contains(&environment)
        {
            environments.push(environment);
        }
    }
    if environments.is_empty() {
        "none recorded on this run".to_owned()
    } else {
        environments.join(", ")
    }
}
