//! The execution harness (ORCH-003, F6 Wave 3): an explicit state machine
//! around an [`AgentRuntime`], not a loop.
//!
//! The reusable loop of the approved context/harness architecture
//! (CONTEXT-HARNESS-ARCHITECTURE §3) becomes a first-class, event-sourced
//! machine:
//!
//! ```text
//! prepare → execute → observe → verify → persist
//!          ↘ continue / recover / escalate (first-class states)
//! ```
//!
//! # The state lattice
//!
//! ```text
//! origin ──prepare──► Preparing
//! Preparing ──execute──► Executing
//! Executing ──observe──► Observing ──verify──► Verifying
//! Executing ──verify/persist──► Verifying / Persisting
//! Observing ──verify/persist──► Verifying / Persisting
//! Verifying ──persist──► Persisting
//! Persisting ──continue/complete──► Continuing / Done
//! Continuing ──execute──► Executing            (the loop)
//! Continuing ──prepare──► Preparing            (re-prepare: model switch)
//! any active state ──recover──► Recovering     (interruption)
//! any active state ──escalate──► Escalated     (needs the human)
//! Recovering ──continue/escalate──► Continuing / Escalated
//! Escalated ──continue/complete/fail──► Continuing / Done / Failed
//! Preparing/Executing/…/Continuing ──fail──► Failed
//! Done, Failed — terminal
//! ```
//!
//! # Every transition is event-sourced
//!
//! Each transition emits one [`HarnessEventRecord`] through the
//! [`HarnessObserver`] seam — the interface through which harness
//! transitions land on the task's world event stream (the caller wires
//! the world envelope; this crate treats envelopes as opaque values and
//! never imports the world crate, kernel §1/§5). The record is written
//! BEFORE the machine commits: if the observer rejects the record, the
//! transition did not happen and the machine state is unchanged.
//!
//! # Recovery is reconstruction, never replay of model output
//!
//! [`TaskHarness::replay`] reconstructs a harness from its OWN history —
//! the transition records and nothing else. The records are structurally
//! incapable of carrying model output: an executed turn records its
//! outcome status and the named missing capabilities, never the
//! runtime's summary text, and unknown fields are rejected
//! (`deny_unknown_fields`). A recovered harness resumes from durable
//! state; it never "starts over" (the Wave-3 addendum §3 law).
//!
//! # The seams (kernel §1 — existing signatures frozen)
//!
//! - [`AgentRuntime`] — the caller supplies the boxed runtime at
//!   `prepare`; execution state, never task identity.
//! - [`ContextCompiler`] — the context-compilation seam. The caller
//!   wires the EXISTING public `flauz-context` compile step
//!   (`Context::compile_from_snapshot`) behind it; ORCH-002's engine
//!   later replaces the seam's internals with the same public types.
//! - [`HarnessObserver`] — the task-stream seam every transition is
//!   emitted through.
//! - [`ExecStore`](crate::ExecStore) — the store seam the persist step
//!   records its durable references against (the owning stores are
//!   caller-side; the harness names what was written, refs only).
//!
//! # Registered event types (kernel §5)
//!
//! This module registers the following `event_type` vocabulary (grammar
//! `<entity>.<verb_past>`); other modules register their own.
//!
//! | Constant | `event_type` |
//! |---|---|
//! | [`harness_event_types::HARNESS_PREPARED`] | `harness.prepared` |
//! | [`harness_event_types::HARNESS_EXECUTED`] | `harness.executed` |
//! | [`harness_event_types::HARNESS_OBSERVED`] | `harness.observed` |
//! | [`harness_event_types::HARNESS_VERIFIED`] | `harness.verified` |
//! | [`harness_event_types::HARNESS_PERSISTED`] | `harness.persisted` |
//! | [`harness_event_types::HARNESS_CONTINUED`] | `harness.continued` |
//! | [`harness_event_types::HARNESS_RECOVERED`] | `harness.recovered` |
//! | [`harness_event_types::HARNESS_ESCALATED`] | `harness.escalated` |
//! | [`harness_event_types::HARNESS_COMPLETED`] | `harness.completed` |
//! | [`harness_event_types::HARNESS_FAILED`] | `harness.failed` |
//!
//! # Canonical JSON (kernel §4) and determinism (kernel §7)
//!
//! Every serialized type carries `"v": 1`, uses snake_case fields,
//! rejects unknown fields, contains no floats, and emits timestamps as
//! RFC 3339 UTC `YYYY-MM-DDTHH:MM:SSZ` (durations are integer
//! milliseconds). The machine never reads the wall clock or randomness:
//! every timestamp is caller-supplied and the only entropy source is the
//! crate's private `ulid` module, unused here — harness records carry no
//! generated identities (the world stream assigns event IDs when the
//! observer seam lands them).

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;
use crate::ids::{EntityKind, ModelId, validate};
use crate::refs::ActorRef;
use crate::runtime::{AgentRuntime, RuntimeRequest, RuntimeStatus};
use crate::time::Timestamp;
use crate::{
    ContractVersion, ExecError, MAX_NAME_BYTES, MAX_REFERENCE_BYTES, ensure_non_empty,
    ensure_str_bound,
};

/// The `event_type` vocabulary registered by this module (kernel §5).
/// Shared grammar: `<entity>.<verb_past>`.
pub mod harness_event_types {
    /// The harness attached model/runtime/environment and compiled its
    /// context (state → `Preparing`).
    pub const HARNESS_PREPARED: &str = "harness.prepared";
    /// The harness ran one turn through the attached runtime (state →
    /// `Executing`).
    pub const HARNESS_EXECUTED: &str = "harness.executed";
    /// The harness recorded an observation (state → `Observing`).
    pub const HARNESS_OBSERVED: &str = "harness.observed";
    /// The harness turned a claim into evidence (state → `Verifying`).
    pub const HARNESS_VERIFIED: &str = "harness.verified";
    /// The harness recorded a cycle's durable writes (state →
    /// `Persisting`).
    pub const HARNESS_PERSISTED: &str = "harness.persisted";
    /// The harness continued onto its next cycle (state → `Continuing`).
    pub const HARNESS_CONTINUED: &str = "harness.continued";
    /// The harness resumed from its own history (state → `Recovering`).
    pub const HARNESS_RECOVERED: &str = "harness.recovered";
    /// The harness asked for the human (state → `Escalated`).
    pub const HARNESS_ESCALATED: &str = "harness.escalated";
    /// The harness finished the task (state → `Done`).
    pub const HARNESS_COMPLETED: &str = "harness.completed";
    /// The harness failed the task (state → `Failed`).
    pub const HARNESS_FAILED: &str = "harness.failed";
}

/// The states of the harness machine (ORCH-003, Wave-3 addendum §3).
/// Serialized in snake_case; the three continuations — `Continuing`,
/// `Recovering`, `Escalated` — are first-class states, not flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessStateKind {
    /// Attaching model/runtime/environment and compiling context.
    Preparing,
    /// A turn is running (or just ran) through the attached runtime.
    Executing,
    /// Recording an observation about the world.
    Observing,
    /// Turning a claim into evidence (the world pattern).
    Verifying,
    /// Recording the cycle's durable writes (the store seam).
    Persisting,
    /// The first-class continuation: another cycle is coming.
    Continuing,
    /// The first-class continuation: picking up from the machine's own
    /// history after an interruption.
    Recovering,
    /// The first-class continuation: the human is needed.
    Escalated,
    /// The task finished. Terminal.
    Done,
    /// The task failed. Terminal.
    Failed,
}

impl HarnessStateKind {
    /// Every state, in declaration order.
    pub const ALL: [Self; 10] = [
        Self::Preparing,
        Self::Executing,
        Self::Observing,
        Self::Verifying,
        Self::Persisting,
        Self::Continuing,
        Self::Recovering,
        Self::Escalated,
        Self::Done,
        Self::Failed,
    ];

    /// Whether the state is terminal (`Done` or `Failed`).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Failed)
    }

    /// Whether a harness in this state can be resumed after an
    /// interruption (any non-terminal state).
    #[must_use]
    pub const fn is_resumable(self) -> bool {
        !self.is_terminal()
    }

    /// The registered `event_type` for a transition INTO this state.
    #[must_use]
    pub const fn entry_event_type(self) -> &'static str {
        match self {
            Self::Preparing => harness_event_types::HARNESS_PREPARED,
            Self::Executing => harness_event_types::HARNESS_EXECUTED,
            Self::Observing => harness_event_types::HARNESS_OBSERVED,
            Self::Verifying => harness_event_types::HARNESS_VERIFIED,
            Self::Persisting => harness_event_types::HARNESS_PERSISTED,
            Self::Continuing => harness_event_types::HARNESS_CONTINUED,
            Self::Recovering => harness_event_types::HARNESS_RECOVERED,
            Self::Escalated => harness_event_types::HARNESS_ESCALATED,
            Self::Done => harness_event_types::HARNESS_COMPLETED,
            Self::Failed => harness_event_types::HARNESS_FAILED,
        }
    }

    /// The canonical (serialized) name of this state.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preparing => "preparing",
            Self::Executing => "executing",
            Self::Observing => "observing",
            Self::Verifying => "verifying",
            Self::Persisting => "persisting",
            Self::Continuing => "continuing",
            Self::Recovering => "recovering",
            Self::Escalated => "escalated",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// One transition of the harness machine: the verb a caller drives.
/// The three continuations — [`Continue`](Self::Continue),
/// [`Recover`](Self::Recover), [`Escalate`](Self::Escalate) — are
/// first-class transitions, not error paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessTransition {
    /// Attach model/runtime/environment and compile context.
    Prepare,
    /// Run one turn through the attached runtime.
    Execute,
    /// Record an observation.
    Observe,
    /// Turn a claim into evidence.
    Verify,
    /// Record the cycle's durable writes.
    Persist,
    /// Continue onto the next cycle.
    Continue,
    /// Resume from the machine's own history.
    Recover,
    /// Ask for the human.
    Escalate,
    /// Finish the task.
    Complete,
    /// Fail the task.
    Fail,
}

impl HarnessTransition {
    /// Every transition, in declaration order.
    pub const ALL: [Self; 10] = [
        Self::Prepare,
        Self::Execute,
        Self::Observe,
        Self::Verify,
        Self::Persist,
        Self::Continue,
        Self::Recover,
        Self::Escalate,
        Self::Complete,
        Self::Fail,
    ];

    /// The state this transition moves the machine INTO.
    #[must_use]
    pub const fn target_state(self) -> HarnessStateKind {
        match self {
            Self::Prepare => HarnessStateKind::Preparing,
            Self::Execute => HarnessStateKind::Executing,
            Self::Observe => HarnessStateKind::Observing,
            Self::Verify => HarnessStateKind::Verifying,
            Self::Persist => HarnessStateKind::Persisting,
            Self::Continue => HarnessStateKind::Continuing,
            Self::Recover => HarnessStateKind::Recovering,
            Self::Escalate => HarnessStateKind::Escalated,
            Self::Complete => HarnessStateKind::Done,
            Self::Fail => HarnessStateKind::Failed,
        }
    }

    /// The registered `event_type` this transition emits.
    #[must_use]
    pub const fn event_type(self) -> &'static str {
        self.target_state().entry_event_type()
    }
}

/// Whether `transition` is legal from `from` (`None` is the origin: a
/// harness is born by preparing). The lattice is explicit and total: the
/// five pipeline phases chain forward, `Continuing` re-enters the loop
/// and re-prepares for model switches, `Recover` is legal from every
/// active (non-terminal, non-continuation) state, `Escalate` from every
/// non-terminal state except `Done`/`Failed`, and the terminal states
/// accept no transitions.
#[must_use]
pub fn transition_is_legal(from: Option<HarnessStateKind>, transition: HarnessTransition) -> bool {
    use HarnessStateKind as State;
    use HarnessTransition as Move;
    match from {
        None => matches!(transition, Move::Prepare),
        Some(State::Preparing) => matches!(
            transition,
            Move::Execute | Move::Recover | Move::Escalate | Move::Fail
        ),
        Some(State::Executing) => matches!(
            transition,
            Move::Observe
                | Move::Verify
                | Move::Persist
                | Move::Recover
                | Move::Escalate
                | Move::Fail
        ),
        Some(State::Observing) => matches!(
            transition,
            Move::Verify | Move::Persist | Move::Recover | Move::Escalate | Move::Fail
        ),
        Some(State::Verifying) => {
            matches!(
                transition,
                Move::Persist | Move::Recover | Move::Escalate | Move::Fail
            )
        }
        Some(State::Persisting) => matches!(
            transition,
            Move::Continue | Move::Complete | Move::Recover | Move::Escalate | Move::Fail
        ),
        Some(State::Continuing) => matches!(
            transition,
            Move::Execute
                | Move::Prepare
                | Move::Complete
                | Move::Recover
                | Move::Escalate
                | Move::Fail
        ),
        Some(State::Recovering) => matches!(transition, Move::Continue | Move::Escalate),
        Some(State::Escalated) => {
            matches!(transition, Move::Continue | Move::Complete | Move::Fail)
        }
        Some(State::Done) | Some(State::Failed) => false,
    }
}

/// Errors returned by the harness machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessError {
    /// The transition is not legal from the current state (the lattice
    /// above is total; illegal moves are rejected, never coerced).
    IllegalTransition {
        /// The state the machine was in.
        from: HarnessStateKind,
        /// The transition that was attempted.
        transition: HarnessTransition,
    },
    /// A canonical rule was violated (bounds, ordering, grammar,
    /// consistency).
    Invalid(String),
    /// The attached runtime or the context-compilation seam rejected the
    /// step. No transition happened: the machine state is unchanged.
    Runtime(ExecError),
}

impl HarnessError {
    /// Builds an [`HarnessError::Invalid`] from any string-like reason.
    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}

impl std::fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalTransition { from, transition } => write!(
                formatter,
                "illegal harness transition: {transition:?} from {}",
                from.as_str()
            ),
            Self::Invalid(reason) => write!(formatter, "invalid harness state: {reason}"),
            Self::Runtime(error) => write!(formatter, "harness step failed: {error}"),
        }
    }
}

impl std::error::Error for HarnessError {}

impl From<ExecError> for HarnessError {
    fn from(error: ExecError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::ids::IdError> for HarnessError {
    fn from(error: crate::ids::IdError) -> Self {
        Self::Runtime(ExecError::Id(error))
    }
}

/// Maximum number of durable references one persist step may name.
pub const MAX_PERSISTED_REFS: usize = 64;

/// Maximum number of kept references per family in a recovery brief.
pub const MAX_KEPT_REFS: usize = 64;

/// Maximum length of an escalation or failure reason.
pub const MAX_HARNESS_REASON_BYTES: usize = 512;

/// The frozen access-surface kind vocabulary (kernel §7: Resource !=
/// AccessSurface). An observation names the surface it came from; the
/// resource identity is stable when surfaces change.
pub const SURFACE_KINDS: [&str; 7] = [
    "browser",
    "api",
    "cli",
    "mcp",
    "native_desktop",
    "file",
    "service_integration",
];

/// The model/runtime/environment a harness attached at `prepare`, plus
/// the durable reference of the compiled context. Execution state
/// (kernel §6): a re-prepare replaces the attachments and never changes
/// the task identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessAttachments {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The model the harness drives.
    pub model_id: ModelId,
    /// The environment execution is bound to, when one is attached.
    pub environment_id: Option<crate::ids::EnvironmentId>,
    /// The neutral kind label of the attached runtime.
    pub runtime_kind: String,
    /// The durable reference of the compiled context (the
    /// context-compilation seam's output), when compilation produced
    /// one.
    pub context_ref: Option<String>,
}

impl HarnessAttachments {
    /// Builds attachments, validating the kind label and the context
    /// reference bounds.
    pub fn new(
        model_id: ModelId,
        environment_id: Option<crate::ids::EnvironmentId>,
        runtime_kind: &str,
        context_ref: Option<&str>,
    ) -> Result<Self, HarnessError> {
        crate::ensure_kind_label("runtime kind", runtime_kind)?;
        if let Some(reference) = context_ref {
            ensure_non_empty("context reference", reference)?;
            ensure_str_bound("context reference", reference, MAX_REFERENCE_BYTES)?;
            if validate(reference)? != EntityKind::ContextSnapshot {
                return Err(HarnessError::invalid(format!(
                    "context reference {reference:?} is not a canonical ctxsnap_ id"
                )));
            }
        }
        Ok(Self {
            v: ContractVersion,
            model_id,
            environment_id,
            runtime_kind: runtime_kind.to_owned(),
            context_ref: context_ref.map(str::to_owned),
        })
    }

    /// Validates the attachments.
    pub fn validate(&self) -> Result<(), HarnessError> {
        Self::new(
            self.model_id.clone(),
            self.environment_id.clone(),
            &self.runtime_kind,
            self.context_ref.as_deref(),
        )?;
        Ok(())
    }
}

/// The input to the context-compilation seam: the task and the attached
/// execution state. Plain data — the seam's wiring assembles the real
/// snapshot and profile caller-side and calls the EXISTING public
/// compile step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextCompileInput {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The task whose context is compiled (canonical `task_` id).
    pub task_id: String,
    /// The model the context is compiled for.
    pub model_id: ModelId,
    /// The environment the task is bound to, when one is attached.
    pub environment_id: Option<crate::ids::EnvironmentId>,
}

/// The output of the context-compilation seam: the durable reference of
/// the compiled view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextCompileOutput {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The durable reference of the compiled context (a canonical
    /// `ctxsnap_` id; a bounded reference, never bulk content).
    pub context_ref: String,
}

/// The context-compilation seam (ORCH-003 → ORCH-002): the harness
/// calls the EXISTING public compile step through this trait. The
/// caller wires `flauz-context`'s `Context::compile_from_snapshot`
/// behind it today; ORCH-002's engine replaces the internals later
/// behind the same seam, with the same public types.
pub trait ContextCompiler: Send + Sync {
    /// Compiles a context view for the attached model from the task's
    /// durable state. Deterministic: caller-supplied timestamps, no
    /// I/O.
    ///
    /// # Errors
    ///
    /// Returns the seam's error when compilation is rejected (the
    /// harness surfaces it as [`HarnessError::Runtime`] and commits no
    /// transition).
    fn compile(&self, input: &ContextCompileInput) -> Result<ContextCompileOutput, ExecError>;
}

/// The task-stream seam: every harness transition emits exactly one
/// record through this observer, and the record is written BEFORE the
/// machine commits the transition. The caller wires the observer to the
/// task's world event stream (envelope assignment is world-side; this
/// crate treats envelopes as opaque values).
pub trait HarnessObserver: Send + Sync {
    /// Records one harness transition on the task's stream.
    ///
    /// # Errors
    ///
    /// Returns an error when the stream rejects the record; the harness
    /// then leaves its state unchanged (the transition did not happen).
    fn record(&self, record: &HarnessEventRecord) -> Result<(), ExecError>;
}

/// What a recovery kept (the J-03 disclosure): the durable families
/// that survived the interruption, as bounded canonical references.
/// The full durable state lives in the owning stores; the brief names
/// what the resumed task still holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessKeptRefs {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The durable work products kept (`art_` references, sorted).
    pub artifact_refs: Vec<String>,
    /// The verified evidence kept (`evd_` references, sorted).
    pub evidence_refs: Vec<String>,
    /// The remembered notes kept (`mem_` references, sorted).
    pub memory_refs: Vec<String>,
}

impl Default for HarnessKeptRefs {
    fn default() -> Self {
        Self::empty()
    }
}

impl HarnessKeptRefs {
    /// The empty brief (nothing was kept).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            v: ContractVersion,
            artifact_refs: Vec::new(),
            evidence_refs: Vec::new(),
            memory_refs: Vec::new(),
        }
    }

    /// Builds a kept brief from the three families. Each list must be
    /// bounded, sorted, deduplicated, and canonically kinded.
    pub fn new(
        artifact_refs: Vec<String>,
        evidence_refs: Vec<String>,
        memory_refs: Vec<String>,
    ) -> Result<Self, HarnessError> {
        ensure_ref_family("artifact refs", &artifact_refs, Some(EntityKind::Artifact))?;
        ensure_ref_family("evidence refs", &evidence_refs, Some(EntityKind::Evidence))?;
        ensure_ref_family("memory refs", &memory_refs, Some(EntityKind::MemoryItem))?;
        Ok(Self {
            v: ContractVersion,
            artifact_refs,
            evidence_refs,
            memory_refs,
        })
    }

    /// Validates the brief.
    pub fn validate(&self) -> Result<(), HarnessError> {
        Self::new(
            self.artifact_refs.clone(),
            self.evidence_refs.clone(),
            self.memory_refs.clone(),
        )?;
        Ok(())
    }

    /// The total number of kept references.
    #[must_use]
    pub fn total(&self) -> usize {
        self.artifact_refs.len() + self.evidence_refs.len() + self.memory_refs.len()
    }
}

/// The compaction record a recovery resumed from: what was summarized
/// so the task could keep going, and where the record lives. Every
/// summarized item is named in the record itself (ORCH-002's law); the
/// harness carries the record's reference and the summarized count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessCompactionRecord {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The durable reference of the compaction record (a canonical
    /// `ctxsnap_` id — the compacted snapshot the record names).
    pub record_ref: String,
    /// How many items the record summarized.
    pub summarized_items: u64,
}

impl HarnessCompactionRecord {
    /// Builds a compaction record reference.
    pub fn new(record_ref: &str, summarized_items: u64) -> Result<Self, HarnessError> {
        ensure_non_empty("compaction record reference", record_ref)?;
        ensure_str_bound(
            "compaction record reference",
            record_ref,
            MAX_REFERENCE_BYTES,
        )?;
        if validate(record_ref)? != EntityKind::ContextSnapshot {
            return Err(HarnessError::invalid(format!(
                "compaction record reference {record_ref:?} is not a canonical ctxsnap_ id"
            )));
        }
        if summarized_items == 0 {
            return Err(HarnessError::invalid(
                "a compaction record must summarize at least one item",
            ));
        }
        Ok(Self {
            v: ContractVersion,
            record_ref: record_ref.to_owned(),
            summarized_items,
        })
    }

    /// Validates the record.
    pub fn validate(&self) -> Result<(), HarnessError> {
        Self::new(&self.record_ref, self.summarized_items)?;
        Ok(())
    }
}

/// The recovery brief: what the caller's durable state says was kept
/// and what was summarized. Assembled caller-side from the world and
/// context stores; carried by the `harness.recovered` event so the
/// recovery surface (J-03) can say "everything kept except what was
/// summarized" from durable state alone.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecoveryBrief {
    /// What was kept.
    pub kept: HarnessKeptRefs,
    /// The compaction record the recovery resumed from, when one
    /// exists.
    pub compaction: Option<HarnessCompactionRecord>,
}

/// The detail payload of one harness event. Internally tagged with
/// `"kind"` (kernel §4); serialization is hand-rolled so canonical
/// reads stay strict — unknown fields are rejected, not ignored. The
/// variants carry references and outcome classifications only: model
/// output never enters the machine's history (the replay law).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessEventDetail {
    /// A prepare: the attachments and the compiled context reference.
    Prepare {
        /// The model the harness attached.
        model_id: ModelId,
        /// The environment bound at prepare time, if any.
        environment_id: Option<crate::ids::EnvironmentId>,
        /// The neutral kind label of the attached runtime.
        runtime_kind: String,
        /// The compiled context's durable reference, when the seam
        /// produced one.
        context_ref: Option<String>,
    },
    /// An executed turn: its number, outcome status and the named
    /// missing capabilities on a gap. Never the runtime's summary.
    Execute {
        /// The 1-based turn number.
        turn: u64,
        /// The turn's outcome status (the runtime outcome's
        /// classification, as data).
        outcome_status: RuntimeStatus,
        /// Exactly the required capabilities the runtime does not
        /// advertise (empty unless the status is a capability gap).
        missing_capabilities: Vec<CapabilityId>,
    },
    /// An observation: the world pattern as data (refs + surface).
    Observe {
        /// The observation's canonical `obs_` reference.
        observation_ref: String,
        /// The resource the observation is about (`res_`).
        subject_resource: String,
        /// The surface the observation came from (the frozen vocabulary).
        surface: String,
    },
    /// A verification: the claim that became evidence, the evidence
    /// produced, and the verification event that proves it (kernel §7:
    /// evidence requires verifier attribution — the record's actor — a
    /// verification event reference, and a timestamp — the record's
    /// `occurred_at`).
    Verify {
        /// The verified claim (`claim_`).
        claim_ref: String,
        /// The evidence produced (`evd_`).
        evidence_ref: String,
        /// The verification event (`ev_`).
        verification_event_ref: String,
    },
    /// A persist: the durable references the cycle wrote through the
    /// owning stores (the store seam), sorted and deduplicated.
    Persist {
        /// The canonical references written this cycle.
        persisted_refs: Vec<String>,
    },
    /// A recovery: how many of the machine's own events were replayed,
    /// what was kept, and the compaction record resumed from.
    Recover {
        /// The number of the machine's own transition records the
        /// reconstruction replayed.
        replayed_events: u64,
        /// What the durable state kept.
        kept: HarnessKeptRefs,
        /// The compaction record the recovery resumed from, when one
        /// exists.
        compaction: Option<HarnessCompactionRecord>,
    },
    /// An escalation: the bounded reason the human is needed.
    Escalate {
        /// Why the harness escalated (bounded prose).
        reason: String,
    },
    /// A failure: the bounded reason.
    Fail {
        /// Why the harness failed (bounded prose).
        reason: String,
    },
    /// A continuation onto the next cycle.
    Continue,
    /// A completion.
    Complete,
}

impl HarnessEventDetail {
    /// Validates the detail against the canonical rules.
    pub fn validate(&self) -> Result<(), HarnessError> {
        match self {
            Self::Prepare {
                model_id,
                environment_id,
                runtime_kind,
                context_ref,
            } => {
                HarnessAttachments::new(
                    model_id.clone(),
                    environment_id.clone(),
                    runtime_kind,
                    context_ref.as_deref(),
                )?;
                Ok(())
            }
            Self::Execute {
                turn,
                outcome_status,
                missing_capabilities,
            } => {
                if *turn == 0 {
                    return Err(HarnessError::invalid("turn numbers are 1-based"));
                }
                crate::ensure_capability_list("missing capabilities", missing_capabilities)
                    .map_err(HarnessError::Runtime)?;
                match outcome_status {
                    RuntimeStatus::Completed => {
                        if !missing_capabilities.is_empty() {
                            return Err(HarnessError::invalid(
                                "a completed turn must not carry missing capabilities",
                            ));
                        }
                    }
                    RuntimeStatus::CapabilityGap => {
                        if missing_capabilities.is_empty() {
                            return Err(HarnessError::invalid(
                                "a capability gap must carry at least one missing capability",
                            ));
                        }
                    }
                }
                Ok(())
            }
            Self::Observe {
                observation_ref,
                subject_resource,
                surface,
            } => {
                ensure_canonical_ref(
                    "observation reference",
                    observation_ref,
                    EntityKind::Observation,
                )?;
                ensure_canonical_ref("subject resource", subject_resource, EntityKind::Resource)?;
                ensure_non_empty("surface", surface)?;
                ensure_str_bound("surface", surface, MAX_NAME_BYTES)?;
                if !SURFACE_KINDS.contains(&surface.as_str()) {
                    return Err(HarnessError::invalid(format!(
                        "surface {surface:?} is not in the frozen access-surface vocabulary"
                    )));
                }
                Ok(())
            }
            Self::Verify {
                claim_ref,
                evidence_ref,
                verification_event_ref,
            } => {
                ensure_canonical_ref("claim reference", claim_ref, EntityKind::Claim)?;
                ensure_canonical_ref("evidence reference", evidence_ref, EntityKind::Evidence)?;
                ensure_canonical_ref(
                    "verification event reference",
                    verification_event_ref,
                    EntityKind::Event,
                )?;
                Ok(())
            }
            Self::Persist { persisted_refs } => {
                ensure_ref_family("persisted refs", persisted_refs, None)?;
                Ok(())
            }
            Self::Recover {
                replayed_events,
                kept,
                compaction,
            } => {
                if *replayed_events == 0 {
                    return Err(HarnessError::invalid(
                        "a recovery replays at least one of the machine's own events",
                    ));
                }
                kept.validate()?;
                if let Some(record) = compaction {
                    record.validate()?;
                }
                Ok(())
            }
            Self::Escalate { reason } | Self::Fail { reason } => {
                ensure_non_empty("harness reason", reason)?;
                ensure_str_bound("harness reason", reason, MAX_HARNESS_REASON_BYTES)?;
                Ok(())
            }
            Self::Continue | Self::Complete => Ok(()),
        }
    }
}

impl Serialize for HarnessEventDetail {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        match self {
            Self::Prepare {
                model_id,
                environment_id,
                runtime_kind,
                context_ref,
            } => {
                map.serialize_entry("kind", "prepare")?;
                map.serialize_entry("model_id", model_id)?;
                map.serialize_entry("environment_id", environment_id)?;
                map.serialize_entry("runtime_kind", runtime_kind)?;
                map.serialize_entry("context_ref", context_ref)?;
            }
            Self::Execute {
                turn,
                outcome_status,
                missing_capabilities,
            } => {
                map.serialize_entry("kind", "execute")?;
                map.serialize_entry("turn", turn)?;
                map.serialize_entry("outcome_status", outcome_status)?;
                map.serialize_entry("missing_capabilities", missing_capabilities)?;
            }
            Self::Observe {
                observation_ref,
                subject_resource,
                surface,
            } => {
                map.serialize_entry("kind", "observe")?;
                map.serialize_entry("observation_ref", observation_ref)?;
                map.serialize_entry("subject_resource", subject_resource)?;
                map.serialize_entry("surface", surface)?;
            }
            Self::Verify {
                claim_ref,
                evidence_ref,
                verification_event_ref,
            } => {
                map.serialize_entry("kind", "verify")?;
                map.serialize_entry("claim_ref", claim_ref)?;
                map.serialize_entry("evidence_ref", evidence_ref)?;
                map.serialize_entry("verification_event_ref", verification_event_ref)?;
            }
            Self::Persist { persisted_refs } => {
                map.serialize_entry("kind", "persist")?;
                map.serialize_entry("persisted_refs", persisted_refs)?;
            }
            Self::Recover {
                replayed_events,
                kept,
                compaction,
            } => {
                map.serialize_entry("kind", "recover")?;
                map.serialize_entry("replayed_events", replayed_events)?;
                map.serialize_entry("kept", kept)?;
                map.serialize_entry("compaction", compaction)?;
            }
            Self::Escalate { reason } => {
                map.serialize_entry("kind", "escalate")?;
                map.serialize_entry("reason", reason)?;
            }
            Self::Fail { reason } => {
                map.serialize_entry("kind", "fail")?;
                map.serialize_entry("reason", reason)?;
            }
            Self::Continue => {
                map.serialize_entry("kind", "continue")?;
            }
            Self::Complete => {
                map.serialize_entry("kind", "complete")?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for HarnessEventDetail {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct DetailVisitor;

        impl<'de> serde::de::Visitor<'de> for DetailVisitor {
            type Value = HarnessEventDetail;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("harness event detail tagged with `kind`")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                use serde::de::Error;

                let mut kind: Option<String> = None;
                let mut fields: Vec<(String, serde_json::Value)> = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    if key == "kind" {
                        if kind.replace(map.next_value()?).is_some() {
                            return Err(A::Error::custom("duplicate field `kind`"));
                        }
                        continue;
                    }
                    let value = map.next_value::<serde_json::Value>()?;
                    if fields.iter().any(|(existing, _)| *existing == key) {
                        return Err(A::Error::custom(format!("duplicate field {key:?}")));
                    }
                    fields.push((key, value));
                }
                let take = |fields: &mut Vec<(String, serde_json::Value)>,
                            name: &'static str|
                 -> Result<serde_json::Value, A::Error> {
                    let position = fields
                        .iter()
                        .position(|(existing, _)| existing == name)
                        .ok_or_else(|| A::Error::missing_field(name))?;
                    Ok(fields.remove(position).1)
                };
                let reject_leftovers =
                    |fields: &[(String, serde_json::Value)]| -> Result<(), A::Error> {
                        if let Some((name, _)) = fields.first() {
                            return Err(A::Error::custom(format!(
                                "unknown field {name:?} for this harness event detail kind"
                            )));
                        }
                        Ok(())
                    };
                match kind.as_deref() {
                    Some("prepare") => {
                        let model_id: ModelId =
                            serde_json::from_value(take(&mut fields, "model_id")?)
                                .map_err(A::Error::custom)?;
                        let environment_id = match take(&mut fields, "environment_id")? {
                            serde_json::Value::Null => None,
                            value => Some(serde_json::from_value(value).map_err(A::Error::custom)?),
                        };
                        let runtime_kind: String =
                            serde_json::from_value(take(&mut fields, "runtime_kind")?)
                                .map_err(A::Error::custom)?;
                        let context_ref = match take(&mut fields, "context_ref")? {
                            serde_json::Value::Null => None,
                            value => Some(serde_json::from_value(value).map_err(A::Error::custom)?),
                        };
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Prepare {
                            model_id,
                            environment_id,
                            runtime_kind,
                            context_ref,
                        })
                    }
                    Some("execute") => {
                        let turn: u64 = serde_json::from_value(take(&mut fields, "turn")?)
                            .map_err(A::Error::custom)?;
                        let outcome_status: RuntimeStatus =
                            serde_json::from_value(take(&mut fields, "outcome_status")?)
                                .map_err(A::Error::custom)?;
                        let missing_capabilities: Vec<CapabilityId> =
                            serde_json::from_value(take(&mut fields, "missing_capabilities")?)
                                .map_err(A::Error::custom)?;
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Execute {
                            turn,
                            outcome_status,
                            missing_capabilities,
                        })
                    }
                    Some("observe") => {
                        let observation_ref: String =
                            serde_json::from_value(take(&mut fields, "observation_ref")?)
                                .map_err(A::Error::custom)?;
                        let subject_resource: String =
                            serde_json::from_value(take(&mut fields, "subject_resource")?)
                                .map_err(A::Error::custom)?;
                        let surface: String = serde_json::from_value(take(&mut fields, "surface")?)
                            .map_err(A::Error::custom)?;
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Observe {
                            observation_ref,
                            subject_resource,
                            surface,
                        })
                    }
                    Some("verify") => {
                        let claim_ref: String =
                            serde_json::from_value(take(&mut fields, "claim_ref")?)
                                .map_err(A::Error::custom)?;
                        let evidence_ref: String =
                            serde_json::from_value(take(&mut fields, "evidence_ref")?)
                                .map_err(A::Error::custom)?;
                        let verification_event_ref: String =
                            serde_json::from_value(take(&mut fields, "verification_event_ref")?)
                                .map_err(A::Error::custom)?;
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Verify {
                            claim_ref,
                            evidence_ref,
                            verification_event_ref,
                        })
                    }
                    Some("persist") => {
                        let persisted_refs: Vec<String> =
                            serde_json::from_value(take(&mut fields, "persisted_refs")?)
                                .map_err(A::Error::custom)?;
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Persist { persisted_refs })
                    }
                    Some("recover") => {
                        let replayed_events: u64 =
                            serde_json::from_value(take(&mut fields, "replayed_events")?)
                                .map_err(A::Error::custom)?;
                        let kept: HarnessKeptRefs =
                            serde_json::from_value(take(&mut fields, "kept")?)
                                .map_err(A::Error::custom)?;
                        let compaction = match take(&mut fields, "compaction")? {
                            serde_json::Value::Null => None,
                            value => Some(serde_json::from_value(value).map_err(A::Error::custom)?),
                        };
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Recover {
                            replayed_events,
                            kept,
                            compaction,
                        })
                    }
                    Some("escalate") => {
                        let reason: String = serde_json::from_value(take(&mut fields, "reason")?)
                            .map_err(A::Error::custom)?;
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Escalate { reason })
                    }
                    Some("fail") => {
                        let reason: String = serde_json::from_value(take(&mut fields, "reason")?)
                            .map_err(A::Error::custom)?;
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Fail { reason })
                    }
                    Some("continue") => {
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Continue)
                    }
                    Some("complete") => {
                        reject_leftovers(&fields)?;
                        Ok(HarnessEventDetail::Complete)
                    }
                    other => Err(A::Error::custom(format!(
                        "unknown harness event detail kind {other:?}"
                    ))),
                }
            }
        }

        deserializer.deserialize_map(DetailVisitor)
    }
}

/// One transition of one harness, as durable data: the registered event
/// type, the stream sequence, the task it belongs to, when it happened,
/// who drove it, the state it moved the machine into, and the
/// transition's detail. This record IS the machine's history —
/// reconstruction replays these and nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessEventRecord {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The registered `event_type` (one of the
    /// [`harness_event_types`] vocabulary).
    pub event_type: String,
    /// The record's sequence on the task's harness stream: strictly
    /// increasing from 1.
    pub seq: u64,
    /// The task the harness drives (canonical `task_` id — the identity
    /// that survives every transition).
    pub task_id: String,
    /// When the transition happened (caller-supplied).
    pub occurred_at: Timestamp,
    /// Who or what drove the transition.
    pub actor: ActorRef,
    /// The state the transition moved the machine into.
    pub to_state: HarnessStateKind,
    /// The transition's detail.
    pub detail: HarnessEventDetail,
}

impl HarnessEventRecord {
    /// Builds a record, validating the canonical rules: the event type
    /// matches the target state, the task id is a canonical `task_`
    /// id, the sequence is 1-based, and the detail is consistent with
    /// the event type.
    pub fn new(
        event_type: &str,
        seq: u64,
        task_id: &str,
        occurred_at: Timestamp,
        actor: ActorRef,
        to_state: HarnessStateKind,
        detail: HarnessEventDetail,
    ) -> Result<Self, HarnessError> {
        ensure_non_empty("event type", event_type)?;
        ensure_str_bound("event type", event_type, MAX_NAME_BYTES)?;
        if event_type != to_state.entry_event_type() {
            return Err(HarnessError::invalid(format!(
                "event type {event_type:?} does not match target state {}",
                to_state.as_str()
            )));
        }
        if seq == 0 {
            return Err(HarnessError::invalid("sequence numbers are 1-based"));
        }
        ensure_canonical_ref("task id", task_id, EntityKind::Task)?;
        actor.validate().map_err(HarnessError::Runtime)?;
        detail_matches_state(&detail, to_state)?;
        detail.validate()?;
        Ok(Self {
            v: ContractVersion,
            event_type: event_type.to_owned(),
            seq,
            task_id: task_id.to_owned(),
            occurred_at,
            actor,
            to_state,
            detail,
        })
    }

    /// Validates the record.
    pub fn validate(&self) -> Result<(), HarnessError> {
        Self::new(
            &self.event_type,
            self.seq,
            &self.task_id,
            self.occurred_at,
            self.actor.clone(),
            self.to_state,
            self.detail.clone(),
        )?;
        Ok(())
    }
}

/// The plain-data reconstruction of a harness from its own history: the
/// replayed machine's state, attachments, counters and stream position.
/// The runtime, compiler and observer are re-supplied by
/// [`TaskHarness::resume`] — they are execution state, never identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayedHarness {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The task the harness drives (its identity, unchanged by replay).
    pub task_id: String,
    /// The state the history left the machine in.
    pub state: HarnessStateKind,
    /// The attachments of the last prepare, when the machine prepared.
    pub attachments: Option<HarnessAttachments>,
    /// How many prepares the history recorded.
    pub prepares: u64,
    /// How many turns the history executed.
    pub turns: u64,
    /// The last sequence number on the stream (the history's length).
    pub seq: u64,
}

impl ReplayedHarness {
    /// Validates the reconstruction.
    pub fn validate(&self) -> Result<(), HarnessError> {
        ensure_canonical_ref("task id", &self.task_id, EntityKind::Task)?;
        if self.prepared_without_attachments() {
            return Err(HarnessError::invalid(
                "a history that prepared must carry attachments",
            ));
        }
        if self.turns > 0 && self.attachments.is_none() {
            return Err(HarnessError::invalid(
                "a history that executed turns must carry attachments",
            ));
        }
        if self.seq == 0 {
            return Err(HarnessError::invalid(
                "a replayed history carries at least one event",
            ));
        }
        Ok(())
    }

    fn prepared_without_attachments(&self) -> bool {
        self.prepares > 0 && self.attachments.is_none()
    }
}

/// The harness view-model: the machine exported as plain data for
/// ORCH-004's graph view (a node's execution is a harness run; the
/// graph references harness states as data, never the machine itself).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessView {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The task the harness drives.
    pub task_id: String,
    /// The current state.
    pub state: HarnessStateKind,
    /// The current attachments, when the machine has prepared.
    pub attachments: Option<HarnessAttachments>,
    /// How many prepares ran (a model switch is a re-prepare).
    pub prepares: u64,
    /// How many turns executed.
    pub turns: u64,
    /// How many of the machine's own events the history holds.
    pub events: u64,
    /// Whether the task can be resumed where it left off (a
    /// non-terminal state).
    pub recoverable: bool,
    /// Whether the task currently needs the human.
    pub escalated: bool,
    /// Whether the run is over (`Done` or `Failed`).
    pub terminal: bool,
}

impl HarnessView {
    /// Validates the view.
    pub fn validate(&self) -> Result<(), HarnessError> {
        ensure_canonical_ref("task id", &self.task_id, EntityKind::Task)?;
        if self.recoverable != self.state.is_resumable() {
            return Err(HarnessError::invalid(
                "the recoverable flag must match a non-terminal state",
            ));
        }
        if self.terminal != self.state.is_terminal() {
            return Err(HarnessError::invalid(
                "the terminal flag must match the state",
            ));
        }
        if self.escalated != (self.state == HarnessStateKind::Escalated) {
            return Err(HarnessError::invalid(
                "the escalated flag must match the escalated state",
            ));
        }
        Ok(())
    }
}

/// The execution harness: an explicit state machine around one
/// [`AgentRuntime`] driving one task.
///
/// Every transition is event-sourced through the [`HarnessObserver`]
/// seam before the machine commits; recovery reconstructs the machine
/// from those records alone ([`TaskHarness::replay`]). The machine
/// holds no model output: executed turns record their outcome
/// classification, never the runtime's summary text.
pub struct TaskHarness {
    task_id: String,
    state: HarnessStateKind,
    attachments: Option<HarnessAttachments>,
    prepares: u64,
    turns: u64,
    seq: u64,
    runtime: Option<Box<dyn AgentRuntime>>,
    compiler: Option<Arc<dyn ContextCompiler>>,
    observer: Arc<dyn HarnessObserver>,
    actor: ActorRef,
}

impl TaskHarness {
    /// Prepares a harness: attaches the model, environment and the
    /// boxed runtime, compiles the context through the seam, and emits
    /// the machine's first event (`harness.prepared`). The origin
    /// transition — a harness is born by preparing.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        task_id: &str,
        actor: ActorRef,
        model_id: ModelId,
        environment_id: Option<crate::ids::EnvironmentId>,
        runtime: Box<dyn AgentRuntime>,
        compiler: Arc<dyn ContextCompiler>,
        observer: Arc<dyn HarnessObserver>,
        at: Timestamp,
    ) -> Result<Self, HarnessError> {
        ensure_canonical_ref("task id", task_id, EntityKind::Task)?;
        actor.validate().map_err(HarnessError::Runtime)?;
        let mut harness = Self {
            task_id: task_id.to_owned(),
            state: HarnessStateKind::Preparing,
            attachments: None,
            prepares: 0,
            turns: 0,
            seq: 0,
            runtime: Some(runtime),
            compiler: Some(compiler),
            observer,
            actor,
        };
        harness.run_prepare(model_id, environment_id, at)?;
        Ok(harness)
    }

    /// Re-prepares the harness (a model or environment switch): replaces
    /// the attachments, the runtime and the compiler through the same
    /// compile seam, and emits another `harness.prepared` event. The
    /// task identity never changes (kernel §6).
    pub fn reprepare(
        &mut self,
        model_id: ModelId,
        environment_id: Option<crate::ids::EnvironmentId>,
        runtime: Box<dyn AgentRuntime>,
        compiler: Arc<dyn ContextCompiler>,
        at: Timestamp,
    ) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Prepare)?;
        self.runtime = Some(runtime);
        self.compiler = Some(compiler);
        let record = self.run_prepare(model_id, environment_id, at)?;
        Ok(record)
    }

    /// The prepare step shared by the origin transition and re-prepares.
    fn run_prepare(
        &mut self,
        model_id: ModelId,
        environment_id: Option<crate::ids::EnvironmentId>,
        at: Timestamp,
    ) -> Result<HarnessEventRecord, HarnessError> {
        let compiler = self
            .compiler
            .clone()
            .ok_or_else(|| HarnessError::invalid("the harness has no compile seam attached"))?;
        let output = compiler
            .compile(&ContextCompileInput {
                v: ContractVersion,
                task_id: self.task_id.clone(),
                model_id: model_id.clone(),
                environment_id: environment_id.clone(),
            })
            .map_err(HarnessError::Runtime)?;
        let runtime_kind = self
            .runtime
            .as_ref()
            .map_or_else(String::new, |runtime| runtime.runtime_kind().to_owned());
        let detail = HarnessEventDetail::Prepare {
            model_id: model_id.clone(),
            environment_id: environment_id.clone(),
            runtime_kind: runtime_kind.clone(),
            context_ref: Some(output.context_ref.clone()),
        };
        let record = self.emit(HarnessStateKind::Preparing, at, detail)?;
        self.attachments = Some(HarnessAttachments::new(
            model_id,
            environment_id,
            &runtime_kind,
            Some(&output.context_ref),
        )?);
        self.prepares += 1;
        Ok(record)
    }

    /// Executes one turn through the attached runtime. The request's
    /// model and environment must match the attachments (execution
    /// state rides on requests; the attachments are authoritative). On
    /// success the machine is `Executing`; a capability gap is an
    /// honest outcome that still transitions (the caller escalates).
    /// The returned outcome is the caller's to read; the machine keeps
    /// only the classification.
    pub fn execute_turn(
        &mut self,
        request: &RuntimeRequest,
        at: Timestamp,
    ) -> Result<crate::runtime::RuntimeOutcome, HarnessError> {
        self.drive(HarnessTransition::Execute)?;
        let attachments = self
            .attachments
            .clone()
            .ok_or_else(|| HarnessError::invalid("the harness must prepare before executing"))?;
        let runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| HarnessError::invalid("the harness has no runtime attached"))?;
        if request.model_id != attachments.model_id {
            return Err(HarnessError::invalid(format!(
                "the request's model {} does not match the attached model {}",
                request.model_id, attachments.model_id
            )));
        }
        if request.environment_id != attachments.environment_id {
            return Err(HarnessError::invalid(
                "the request's environment does not match the attached environment",
            ));
        }
        let outcome = runtime.execute(request).map_err(HarnessError::Runtime)?;
        let detail = HarnessEventDetail::Execute {
            turn: self.turns + 1,
            outcome_status: outcome.status,
            missing_capabilities: outcome.missing_capabilities.clone(),
        };
        self.emit(HarnessStateKind::Executing, at, detail)?;
        self.turns += 1;
        Ok(outcome)
    }

    /// Records an observation (the world pattern as data: refs + the
    /// frozen surface vocabulary). The observation entity itself is
    /// world-side; the harness carries the reference.
    pub fn observe(
        &mut self,
        observation_ref: &str,
        subject_resource: &str,
        surface: &str,
        at: Timestamp,
    ) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Observe)?;
        let detail = HarnessEventDetail::Observe {
            observation_ref: observation_ref.to_owned(),
            subject_resource: subject_resource.to_owned(),
            surface: surface.to_owned(),
        };
        self.emit(HarnessStateKind::Observing, at, detail)
    }

    /// Turns a claim into evidence: the claim reference, the evidence
    /// produced and the verification event that proves it (the world
    /// pattern — verifier attribution is this record's actor, the
    /// timestamp is its `occurred_at`).
    pub fn verify(
        &mut self,
        claim_ref: &str,
        evidence_ref: &str,
        verification_event_ref: &str,
        at: Timestamp,
    ) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Verify)?;
        let detail = HarnessEventDetail::Verify {
            claim_ref: claim_ref.to_owned(),
            evidence_ref: evidence_ref.to_owned(),
            verification_event_ref: verification_event_ref.to_owned(),
        };
        self.emit(HarnessStateKind::Verifying, at, detail)
    }

    /// Records the cycle's durable writes (the store seam): the
    /// canonical references the caller wrote through the owning stores.
    /// The harness's own durable state is its event stream —
    /// reconstructible by replay, never by re-reading model output.
    pub fn persist(
        &mut self,
        persisted_refs: Vec<String>,
        at: Timestamp,
    ) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Persist)?;
        let detail = HarnessEventDetail::Persist { persisted_refs };
        self.emit(HarnessStateKind::Persisting, at, detail)
    }

    /// Continues onto the next cycle (the first-class continuation).
    pub fn continue_cycle(&mut self, at: Timestamp) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Continue)?;
        self.emit(
            HarnessStateKind::Continuing,
            at,
            HarnessEventDetail::Continue,
        )
    }

    /// Marks the harness as picking up from its own history (the
    /// first-class continuation): emits `harness.recovered` carrying the
    /// recovery brief — what was kept, and the compaction record the
    /// recovery resumed from. Legal from any active state; the
    /// reconstruction itself is [`TaskHarness::replay`].
    pub fn recover(
        &mut self,
        brief: RecoveryBrief,
        at: Timestamp,
    ) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Recover)?;
        let detail = HarnessEventDetail::Recover {
            replayed_events: self.seq,
            kept: brief.kept,
            compaction: brief.compaction,
        };
        self.emit(HarnessStateKind::Recovering, at, detail)
    }

    /// Escalates to the human (the first-class continuation): the
    /// bounded reason is durable state — the escalation is explicit and
    /// visible, never a silent failure.
    pub fn escalate(
        &mut self,
        reason: &str,
        at: Timestamp,
    ) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Escalate)?;
        let detail = HarnessEventDetail::Escalate {
            reason: reason.to_owned(),
        };
        self.emit(HarnessStateKind::Escalated, at, detail)
    }

    /// Completes the task.
    pub fn complete(&mut self, at: Timestamp) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Complete)?;
        self.emit(HarnessStateKind::Done, at, HarnessEventDetail::Complete)
    }

    /// Fails the task with a bounded reason.
    pub fn fail(
        &mut self,
        reason: &str,
        at: Timestamp,
    ) -> Result<HarnessEventRecord, HarnessError> {
        self.drive(HarnessTransition::Fail)?;
        let detail = HarnessEventDetail::Fail {
            reason: reason.to_owned(),
        };
        self.emit(HarnessStateKind::Failed, at, detail)
    }

    /// The current state.
    #[must_use]
    pub fn state(&self) -> HarnessStateKind {
        self.state
    }

    /// The task the harness drives (its identity — unchanged by every
    /// transition).
    #[must_use]
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// The current attachments, when the machine has prepared.
    #[must_use]
    pub fn attachments(&self) -> Option<&HarnessAttachments> {
        self.attachments.as_ref()
    }

    /// How many turns executed.
    #[must_use]
    pub fn turns(&self) -> u64 {
        self.turns
    }

    /// How many prepares ran (a model switch is a re-prepare).
    #[must_use]
    pub fn prepares(&self) -> u64 {
        self.prepares
    }

    /// The last sequence number emitted (the machine's own history
    /// length).
    #[must_use]
    pub fn seq(&self) -> u64 {
        self.seq
    }

    /// Whether the task can be resumed where it left off.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        self.state.is_resumable()
    }

    /// Exports the machine as plain data (the ORCH-004 view-model seam).
    #[must_use]
    pub fn view(&self) -> HarnessView {
        HarnessView {
            v: ContractVersion,
            task_id: self.task_id.clone(),
            state: self.state,
            attachments: self.attachments.clone(),
            prepares: self.prepares,
            turns: self.turns,
            events: self.seq,
            recoverable: self.state.is_resumable(),
            escalated: self.state == HarnessStateKind::Escalated,
            terminal: self.state.is_terminal(),
        }
    }

    /// Validates that `transition` is legal, without committing it.
    fn drive(&self, transition: HarnessTransition) -> Result<(), HarnessError> {
        if transition_is_legal(Some(self.state), transition) {
            Ok(())
        } else {
            Err(HarnessError::IllegalTransition {
                from: self.state,
                transition,
            })
        }
    }

    /// Emits one transition record through the observer seam and
    /// commits the state. The record is written FIRST: if the observer
    /// rejects it, the machine does not move.
    fn emit(
        &mut self,
        to_state: HarnessStateKind,
        at: Timestamp,
        detail: HarnessEventDetail,
    ) -> Result<HarnessEventRecord, HarnessError> {
        let record = HarnessEventRecord::new(
            to_state.entry_event_type(),
            self.seq + 1,
            &self.task_id,
            at,
            self.actor.clone(),
            to_state,
            detail,
        )?;
        self.observer
            .record(&record)
            .map_err(HarnessError::Runtime)?;
        self.seq += 1;
        self.state = to_state;
        Ok(record)
    }

    /// Reconstructs a harness from its own history: replays the
    /// transition records — the machine's history, never model output —
    /// and returns the replayed machine as plain data. The stream must
    /// be a legal history: one task, sequences strictly increasing from
    /// 1, non-decreasing timestamps, every transition legal from the
    /// state the fold has reached, and every event type consistent
    /// with its target state.
    pub fn replay(records: &[HarnessEventRecord]) -> Result<ReplayedHarness, HarnessError> {
        replay_history(records)
    }

    /// Resumes a replayed harness: re-attaches the runtime, the compile
    /// seam and the observer, and leaves the machine at the state its
    /// own history left it in. Terminal histories cannot be resumed
    /// (the view is readable from the [`ReplayedHarness`] itself).
    pub fn resume(
        replayed: ReplayedHarness,
        actor: ActorRef,
        runtime: Box<dyn AgentRuntime>,
        compiler: Arc<dyn ContextCompiler>,
        observer: Arc<dyn HarnessObserver>,
    ) -> Result<Self, HarnessError> {
        replayed.validate()?;
        actor.validate().map_err(HarnessError::Runtime)?;
        if replayed.state.is_terminal() {
            return Err(HarnessError::invalid(format!(
                "a terminal harness ({}) cannot be resumed",
                replayed.state.as_str()
            )));
        }
        Ok(Self {
            task_id: replayed.task_id,
            state: replayed.state,
            attachments: replayed.attachments,
            prepares: replayed.prepares,
            turns: replayed.turns,
            seq: replayed.seq,
            runtime: Some(runtime),
            compiler: Some(compiler),
            observer,
            actor,
        })
    }
}

/// The replay fold (shared by [`TaskHarness::replay`] and the telemetry
/// projection): validates the stream as a legal machine history and
/// reconstructs the state, attachments and counters. This is the
/// recovery law made executable — only the machine's own transitions
/// are read; the records are structurally incapable of carrying model
/// output.
pub fn replay_history(records: &[HarnessEventRecord]) -> Result<ReplayedHarness, HarnessError> {
    let Some(first) = records.first() else {
        return Err(HarnessError::invalid(
            "a replayed history carries at least one event",
        ));
    };
    let task_id = first.task_id.clone();
    let mut state: Option<HarnessStateKind> = None;
    let mut attachments: Option<HarnessAttachments> = None;
    let mut prepares = 0u64;
    let mut turns = 0u64;
    let mut previous: Option<&HarnessEventRecord> = None;
    for record in records {
        record.validate()?;
        if record.task_id != task_id {
            return Err(HarnessError::invalid(format!(
                "record {} belongs to task {} but the history is task {task_id}",
                record.seq, record.task_id
            )));
        }
        if record.seq != previous.map_or(1, |last| last.seq + 1) {
            return Err(HarnessError::invalid(format!(
                "record sequence {} does not continue the stream (expected {})",
                record.seq,
                previous.map_or(1, |last| last.seq + 1)
            )));
        }
        if let Some(last) = previous
            && record.occurred_at.is_before(&last.occurred_at)
        {
            return Err(HarnessError::invalid(format!(
                "record {} moves backwards in time",
                record.seq
            )));
        }
        let transition = transition_for(record.to_state);
        if !transition_is_legal(state, transition) {
            return Err(HarnessError::invalid(format!(
                "record {} is an illegal transition into {}",
                record.seq,
                record.to_state.as_str()
            )));
        }
        match &record.detail {
            HarnessEventDetail::Prepare {
                model_id,
                environment_id,
                runtime_kind,
                context_ref,
            } => {
                attachments = Some(HarnessAttachments::new(
                    model_id.clone(),
                    environment_id.clone(),
                    runtime_kind,
                    context_ref.as_deref(),
                )?);
                prepares += 1;
            }
            HarnessEventDetail::Execute { .. } => {
                turns += 1;
            }
            _ => {}
        }
        state = Some(record.to_state);
        previous = Some(record);
    }
    let state =
        state.ok_or_else(|| HarnessError::invalid("the history must carry at least one event"))?;
    Ok(ReplayedHarness {
        v: ContractVersion,
        task_id,
        state,
        attachments,
        prepares,
        turns,
        seq: previous.map_or(0, |last| last.seq),
    })
}

/// The transition whose target is this state (the 1:1 verb↔state map).
const fn transition_for(state: HarnessStateKind) -> HarnessTransition {
    use HarnessStateKind as State;
    use HarnessTransition as Move;
    match state {
        State::Preparing => Move::Prepare,
        State::Executing => Move::Execute,
        State::Observing => Move::Observe,
        State::Verifying => Move::Verify,
        State::Persisting => Move::Persist,
        State::Continuing => Move::Continue,
        State::Recovering => Move::Recover,
        State::Escalated => Move::Escalate,
        State::Done => Move::Complete,
        State::Failed => Move::Fail,
    }
}

/// Checks that the detail is the one its target state requires.
fn detail_matches_state(
    detail: &HarnessEventDetail,
    state: HarnessStateKind,
) -> Result<(), HarnessError> {
    use HarnessStateKind as State;
    let matches = match detail {
        HarnessEventDetail::Prepare { .. } => state == State::Preparing,
        HarnessEventDetail::Execute { .. } => state == State::Executing,
        HarnessEventDetail::Observe { .. } => state == State::Observing,
        HarnessEventDetail::Verify { .. } => state == State::Verifying,
        HarnessEventDetail::Persist { .. } => state == State::Persisting,
        HarnessEventDetail::Recover { .. } => state == State::Recovering,
        HarnessEventDetail::Escalate { .. } => state == State::Escalated,
        HarnessEventDetail::Fail { .. } => state == State::Failed,
        HarnessEventDetail::Continue => state == State::Continuing,
        HarnessEventDetail::Complete => state == State::Done,
    };
    if matches {
        Ok(())
    } else {
        Err(HarnessError::invalid(format!(
            "the {:?} detail does not belong to the {} state",
            detail,
            state.as_str()
        )))
    }
}

/// Validates a reference list: bounded, non-empty strings, each a
/// canonical id of the expected kind (any kind when `expected` is
/// `None`), sorted and deduplicated.
fn ensure_ref_family(
    field: &'static str,
    refs: &[String],
    expected: Option<EntityKind>,
) -> Result<(), HarnessError> {
    let max = if expected.is_some() {
        MAX_KEPT_REFS
    } else {
        MAX_PERSISTED_REFS
    };
    if refs.len() > max {
        return Err(HarnessError::invalid(format!(
            "{field} exceeds {max} entries"
        )));
    }
    for reference in refs {
        let kind = validate(reference)?;
        if let Some(expected) = expected
            && kind != expected
        {
            return Err(HarnessError::invalid(format!(
                "{field} entry {reference:?} is not a canonical {}_ id",
                expected.prefix()
            )));
        }
    }
    if refs.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(HarnessError::invalid(format!(
            "{field} must be sorted and free of duplicates"
        )));
    }
    Ok(())
}

/// Validates one canonical reference of an exact kind.
fn ensure_canonical_ref(
    field: &'static str,
    reference: &str,
    expected: EntityKind,
) -> Result<(), HarnessError> {
    ensure_non_empty(field, reference)?;
    ensure_str_bound(field, reference, MAX_REFERENCE_BYTES)?;
    let kind = validate(reference)?;
    if kind != expected {
        return Err(HarnessError::invalid(format!(
            "{field} {reference:?} is not a canonical {}_ id",
            expected.prefix()
        )));
    }
    Ok(())
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

    #[test]
    fn registered_event_types_follow_the_grammar() {
        for state in HarnessStateKind::ALL {
            let name = state.entry_event_type();
            let Some((entity, verb)) = name.split_once('.') else {
                panic!("{name} must contain exactly one `.`");
            };
            assert!(!verb.contains('.'), "{name} must contain exactly one `.`");
            for segment in [entity, verb] {
                let mut characters = segment.chars();
                let valid = characters
                    .next()
                    .is_some_and(|first| first.is_ascii_lowercase())
                    && characters.all(|character| {
                        character.is_ascii_lowercase()
                            || character.is_ascii_digit()
                            || character == '_'
                    });
                assert!(valid, "{name} segments must be `[a-z][a-z0-9_]*`");
            }
            assert_eq!(entity, "harness", "the entity is the harness");
        }
        assert_eq!(harness_event_types::HARNESS_PREPARED, "harness.prepared");
    }

    #[test]
    fn every_state_is_reachable_and_terminals_are_final() {
        // The origin prepares; the lattice reaches every state; the
        // terminals accept nothing; no active state is a dead end.
        for state in HarnessStateKind::ALL {
            assert!(
                transition_is_legal(None, HarnessTransition::Prepare),
                "the origin prepares"
            );
            let reachable = HarnessTransition::ALL
                .iter()
                .any(|transition| transition_is_legal(Some(state), *transition));
            if state.is_terminal() {
                assert!(
                    !reachable,
                    "{:?} is terminal and accepts no transition",
                    state
                );
            } else {
                assert!(
                    reachable,
                    "{:?} is active and must accept some transition",
                    state
                );
            }
        }
        // Failure is reachable from every active phase.
        for from in [
            HarnessStateKind::Preparing,
            HarnessStateKind::Executing,
            HarnessStateKind::Observing,
            HarnessStateKind::Verifying,
            HarnessStateKind::Persisting,
            HarnessStateKind::Continuing,
            HarnessStateKind::Escalated,
        ] {
            assert!(
                transition_is_legal(Some(from), HarnessTransition::Fail),
                "Fail must be legal from {from:?}"
            );
        }
        // The three continuations are reachable first-class states.
        assert!(transition_is_legal(
            Some(HarnessStateKind::Persisting),
            HarnessTransition::Continue
        ));
        assert!(transition_is_legal(
            Some(HarnessStateKind::Executing),
            HarnessTransition::Recover
        ));
        assert!(transition_is_legal(
            Some(HarnessStateKind::Executing),
            HarnessTransition::Escalate
        ));
        // Every transition has a distinct registered event type.
        let mut names: Vec<&str> = HarnessStateKind::ALL
            .iter()
            .map(|state| state.entry_event_type())
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), HarnessStateKind::ALL.len());
    }

    #[test]
    fn detail_round_trips_and_rejects_unknown_fields() {
        let detail = HarnessEventDetail::Recover {
            replayed_events: 12,
            kept: ok(HarnessKeptRefs::new(
                vec!["art_01J8ZQ5V8K3T2B7N6X4R9DQPA0".to_owned()],
                vec![],
                vec!["mem_01J8ZQ5V8K3T2B7N6X4R9DQPB1".to_owned()],
            )),
            compaction: Some(ok(HarnessCompactionRecord::new(
                "ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5",
                7,
            ))),
        };
        let serialized = ok(serde_json::to_string(&detail));
        let parsed: HarnessEventDetail = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, detail);
        assert!(serialized.contains("\"kind\":\"recover\""));
        // Unknown fields are rejected (the strict-read law for the
        // hand-rolled tagged enum).
        assert!(
            serde_json::from_str::<HarnessEventDetail>(&serialized.replace(
                "\"replayed_events\":12",
                "\"replayed_events\":12,\"surprise\":1"
            ))
            .is_err()
        );
        // A Continue carries only its kind.
        let unit = ok(serde_json::to_string(&HarnessEventDetail::Continue));
        assert_eq!(unit, "{\"kind\":\"continue\"}");
        let parsed_unit: HarnessEventDetail = ok(serde_json::from_str(&unit));
        assert_eq!(parsed_unit, HarnessEventDetail::Continue);
        assert!(
            serde_json::from_str::<HarnessEventDetail>(&unit.replace('}', ",\"x\":1}")).is_err()
        );
    }
}
