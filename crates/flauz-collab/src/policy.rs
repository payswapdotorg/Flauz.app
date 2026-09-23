//! The sharing policy (work order COL-001 part 2): the per-task worktree
//! posture and the private-vs-shared context visibility — both explicit
//! records, never ambient.
//!
//! # WorktreePolicy — isolated by default
//!
//! Code must preserve Git/worktree semantics and distinguish isolated
//! worktrees from intentional shared-filesystem mode
//! (FLAUZ-SOURCE-OF-TRUTH, Collaboration). A [`WorktreePolicy`] is one
//! task's record of that intent: [`SharedFilesystemMode::Off`] (the
//! default — every collaborator works in an isolated copy), `Read` (one
//! shared filesystem, everyone on the task can read it), or `Write` (one
//! shared filesystem, everyone can read and write it). There is no
//! ambient sharing: a task without a record is isolated, and every
//! non-isolated posture is an explicit record someone set.
//!
//! # ContextVisibility — the projection filter
//!
//! What a collaborator sees is a **permission-filtered projection of the
//! same durable task state** (the Wave-4 addendum §6 law): sharing is
//! never transcript-dumping, and member-private memory items stay
//! private. A [`ContextVisibility`] record carries, per task, whether
//! remembered notes (memory) and work products (artifacts) are
//! member-private or workspace-shared. The filter the caller applies is
//! [`visible_to_member`]: a member-private item is visible to its owner
//! alone — structurally, not by convention.

use serde::{Deserialize, Serialize};

use crate::refs::{ActorRef, TaskRef};
use crate::{CollabError, CollabVersion};

/// The explicit shared-filesystem mode of one task's worktree posture:
/// `off` (isolated — the default), `read`, or `write`. Never ambient.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedFilesystemMode {
    /// Isolated copies: every collaborator's files stay separate (the
    /// default).
    Off,
    /// One shared filesystem: everyone on this task can read it.
    Read,
    /// One shared filesystem: everyone on this task can read and write
    /// it.
    Write,
}

impl SharedFilesystemMode {
    /// The user-facing posture line WITH its consequence (the members
    /// surface renders these words).
    #[must_use]
    pub const fn consequence_line(self) -> &'static str {
        match self {
            Self::Off => "Isolated copy — your files stay separate",
            Self::Read => "Shared files — everyone on this task can read them",
            Self::Write => "Shared files — everyone on this task can read and write them",
        }
    }
}

/// One task's worktree posture: isolated by default, with the explicit
/// shared-filesystem mode as a record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorktreePolicy {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The task the posture belongs to.
    pub task: TaskRef,
    /// The explicit shared-filesystem mode.
    pub shared_filesystem: SharedFilesystemMode,
}

impl WorktreePolicy {
    /// The isolated default for a task: no shared filesystem, everyone
    /// works in their own copy.
    #[must_use]
    pub fn isolated(task: &TaskRef) -> Self {
        Self {
            v: CollabVersion,
            task: task.clone(),
            shared_filesystem: SharedFilesystemMode::Off,
        }
    }

    /// Builds an explicit posture.
    #[must_use]
    pub fn new(task: &TaskRef, shared_filesystem: SharedFilesystemMode) -> Self {
        Self {
            v: CollabVersion,
            task: task.clone(),
            shared_filesystem,
        }
    }

    /// Whether the task is in the isolated default (no shared
    /// filesystem).
    #[must_use]
    pub const fn is_isolated(&self) -> bool {
        matches!(self.shared_filesystem, SharedFilesystemMode::Off)
    }

    /// Validates the record's bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        self.task.validate()?;
        Ok(())
    }
}

/// Whether one family of a task's context is member-private or
/// workspace-shared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Visible to its owner alone ("Only me").
    MemberPrivate,
    /// Visible to every member of the workspace ("The workspace").
    WorkspaceShared,
}

impl Visibility {
    /// The user-facing label (the members surface renders these words).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::MemberPrivate => "Only me",
            Self::WorkspaceShared => "The workspace",
        }
    }
}

/// One task's context visibility: whether its remembered notes (memory)
/// and work products (artifacts) are member-private or workspace-shared.
/// This is the record the projection filter reads — sharing is a
/// permission-filtered projection of the SAME durable state, never
/// transcript-dumping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextVisibility {
    /// Contract schema version (`"v": 1`).
    pub v: CollabVersion,
    /// The task the record belongs to.
    pub task: TaskRef,
    /// Whether remembered notes are member-private or workspace-shared.
    pub memory: Visibility,
    /// Whether work products are member-private or workspace-shared.
    pub artifacts: Visibility,
}

impl ContextVisibility {
    /// The default for a task: member-private notes ("Only me"),
    /// workspace-shared work products (artifacts are the task's shared
    /// durable state — the world model).
    #[must_use]
    pub fn default_for(task: &TaskRef) -> Self {
        Self {
            v: CollabVersion,
            task: task.clone(),
            memory: Visibility::MemberPrivate,
            artifacts: Visibility::WorkspaceShared,
        }
    }

    /// Builds an explicit visibility record.
    #[must_use]
    pub fn new(task: &TaskRef, memory: Visibility, artifacts: Visibility) -> Self {
        Self {
            v: CollabVersion,
            task: task.clone(),
            memory,
            artifacts,
        }
    }

    /// Validates the record's bounds.
    pub fn validate(&self) -> Result<(), CollabError> {
        self.task.validate()?;
        Ok(())
    }
}

/// The projection filter (addendum §6): whether an item owned by
/// `owner` with this `visibility` appears in `viewer`'s projection of
/// the same durable state. A member-private item is visible to its
/// owner alone — **structurally**: no projection built through this
/// filter can ever contain another member's private records.
#[must_use]
pub fn visible_to_member(owner: &ActorRef, visibility: Visibility, viewer: &ActorRef) -> bool {
    match visibility {
        Visibility::WorkspaceShared => true,
        Visibility::MemberPrivate => owner == viewer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn task() -> TaskRef {
        ok(TaskRef::parse("task_01J8ZQ5V8K3T2B7N6X4R9DQPB1"))
    }

    #[test]
    fn modes_serialize_snake_case_and_state_consequences() {
        assert_eq!(
            ok(serde_json::to_string(&SharedFilesystemMode::Off)),
            "\"off\""
        );
        assert_eq!(
            ok(serde_json::to_string(&SharedFilesystemMode::Read)),
            "\"read\""
        );
        assert_eq!(
            ok(serde_json::to_string(&SharedFilesystemMode::Write)),
            "\"write\""
        );
        assert_eq!(
            SharedFilesystemMode::Off.consequence_line(),
            "Isolated copy — your files stay separate"
        );
        assert_eq!(
            SharedFilesystemMode::Read.consequence_line(),
            "Shared files — everyone on this task can read them"
        );
        assert_eq!(
            SharedFilesystemMode::Write.consequence_line(),
            "Shared files — everyone on this task can read and write them"
        );
    }

    #[test]
    fn worktree_policies_serialize_canonically_and_default_to_isolated() {
        let isolated = WorktreePolicy::isolated(&task());
        assert!(isolated.is_isolated());
        let serialized = ok(serde_json::to_string(&isolated));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,",
                "\"task\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\",",
                "\"shared_filesystem\":\"off\"}"
            )
        );
        let reloaded: WorktreePolicy = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, isolated);
        let shared = WorktreePolicy::new(&task(), SharedFilesystemMode::Write);
        assert!(!shared.is_isolated());
        let serialized = ok(serde_json::to_string(&shared));
        assert!(serialized.contains("\"shared_filesystem\":\"write\""));
        assert!(
            serde_json::from_str::<WorktreePolicy>(
                &serialized.replace("\"shared_filesystem\":", "\"mode\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn context_visibility_defaults_to_private_notes_and_shared_artifacts() {
        let visibility = ContextVisibility::default_for(&task());
        assert_eq!(visibility.memory, Visibility::MemberPrivate);
        assert_eq!(visibility.artifacts, Visibility::WorkspaceShared);
        let serialized = ok(serde_json::to_string(&visibility));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,",
                "\"task\":\"task_01J8ZQ5V8K3T2B7N6X4R9DQPB1\",",
                "\"memory\":\"member_private\",",
                "\"artifacts\":\"workspace_shared\"}"
            )
        );
        let reloaded: ContextVisibility = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, visibility);
        assert_eq!(Visibility::MemberPrivate.label(), "Only me");
        assert_eq!(Visibility::WorkspaceShared.label(), "The workspace");
    }

    #[test]
    fn the_projection_filter_keeps_member_private_records_private() {
        // The §6 law, structurally: a member-private item is visible to
        // its owner alone, whoever asks.
        let ana = ok(ActorRef::user("ana"));
        let dev = ok(ActorRef::user("dev"));
        assert!(visible_to_member(&ana, Visibility::MemberPrivate, &ana));
        assert!(
            !visible_to_member(&ana, Visibility::MemberPrivate, &dev),
            "another member NEVER sees a member-private record — even the workspace owner"
        );
        assert!(visible_to_member(&ana, Visibility::WorkspaceShared, &dev));
        assert!(visible_to_member(&ana, Visibility::WorkspaceShared, &ana));
    }
}
