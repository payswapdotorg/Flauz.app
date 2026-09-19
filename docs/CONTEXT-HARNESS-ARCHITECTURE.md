# Flauz Context, Harness & General-Purpose Agent Architecture

**Status:** APPROVED ARCHITECTURE EXTENSION — 2026-09-19

This document extends the frozen Flauz architecture with the runtime primitives
needed to make context engineering, harness engineering, multi-environment
orchestration, verification, reusable Procedures, and human/agent collaboration
first-class capabilities.

It does not replace or weaken Codex Desktop parity work. During F1, only bounded
design/contracts and narrowly necessary seams may be introduced. Implementation
of the new platform begins through the ordered roadmap and named work orders.

## 1. Core invariants

The existing invariant remains:

```
Client != Model != AgentRuntime != Skill != Environment != Provider
```

The platform additionally treats these as distinct:

```
Session != Context
Context != Memory
TaskState != AgentContext
Resource != AccessSurface
ResourceState != ToolResult
Claim != Evidence
Plan != Execution
Execution != Verification
Artifact != Message
HumanApproval != AgentDecision
Agent != Environment
```

Durable project state must not depend on the model's current context window.

## 2. Context Engine

The Context Engine decides what information is presented to a model at a
particular inference step.

A Session retains bounded durable history. The active model context is a
deliberate projection of that history and of other state.

The Context Engine owns:

- context compilation;
- model-specific context budgets and reservations;
- tiered memory selection;
- just-in-time retrieval;
- context ranking/reranking;
- tool discovery and schema loading;
- compaction;
- clean context reset and reconstruction;
- provenance for included context;
- authorization-aware filtering;
- context observability and replay.

### Memory tiers

```
HOT
  current turn
  current tool result
  active plan
  current errors
  current environment state

WARM
  recent conversation
  task summary
  decisions
  unresolved questions
  important discoveries
  recent artifacts

COLD
  complete history
  archived tool results
  workspace knowledge
  documents
  old executions
  reusable Procedures
```

HOT is eligible by default. WARM is selectively included. COLD is retrieved
just in time.

### Context item provenance

Every context item should be attributable to a source:

- user input;
- durable session event;
- memory item;
- artifact;
- resource observation;
- tool result;
- retrieved document;
- skill;
- environment state;
- collaboration event.

Secrets and provider credentials never become model context merely because they
are technically retrievable.

## 3. Harness

A Harness is the runtime control layer around a model and its tools. It is
distinct from the Model and from the Environment.

Conceptually:

```
AgentRuntime / Harness
  + Context Engine
  + Tool Registry
  + Policy
  + Task State
  + Execution
  + Verification
  + Recovery
```

The harness should support the reusable loop:

```
manage task state
  -> prepare context
  -> execute
  -> observe
  -> verify
  -> persist state
  -> continue / recover / escalate
```

The executor and evaluator should be independently replaceable.

Evaluators may be:

- deterministic assertions;
- API checks;
- browser checks;
- tests;
- policy engines;
- reconciliation rules;
- human approval;
- model review;
- external system confirmation.

Self-evaluation by the acting agent is not sufficient for consequential
operations.

## 4. Task Execution Graph

Tasks are represented as a reactive execution graph, not only a linear
conversation and not only a static DAG.

Nodes may express:

- objective;
- inputs;
- required capabilities;
- preferred execution surfaces;
- preconditions;
- action;
- expected outputs;
- evaluator;
- retry policy;
- timeout;
- side-effect policy;
- human approval policy;
- compensation/recovery.

The graph must support:

- serial dependencies;
- parallel work;
- conditional branches;
- loops;
- waits;
- external events;
- human gates;
- cancellation;
- retry;
- rerouting;
- rollback/compensation where meaningful.

## 5. Resource Graph

The orchestrator reasons about resources rather than isolated tools.

A Resource may be:

- a website/account;
- a Git repository;
- a document or spreadsheet;
- a CRM record;
- a database;
- a dataset;
- a cloud host;
- a local folder;
- a sandbox;
- an API endpoint;
- a browser session;
- a physical device.

An AccessSurface is a way of interacting with that resource:

- browser UI;
- API;
- CLI;
- MCP;
- native desktop control;
- file interface;
- service integration.

Multiple AccessSurfaces may represent the same Resource.

The system must detect divergence between observations from multiple surfaces and
must not silently treat conflicting observations as equivalent.

### ResourceState

A ResourceState contains bounded, attributable observations such as:

- identity;
- permissions;
- last verified state;
- authoritative observation source where known;
- active leases;
- pending mutations;
- related artifacts;
- provenance.

## 6. Observation, Claim and Evidence

An agent statement is not automatically truth.

The platform distinguishes:

```
CLAIMED
OBSERVED
VERIFIED
CONTRADICTED
STALE
UNKNOWN
```

A Claim or Observation should carry:

- source;
- actor;
- timestamp;
- resource;
- supporting artifact/evidence references;
- verification status;
- contradictory evidence references where applicable.

This enables an evidence plane usable for software, finance, operations,
research, healthcare, design, and other domains.

## 7. Leases and conflict management

Parallel actors may read concurrently while writes require coordination.

A generalized ResourceLease contains:

- resource;
- owner;
- scope;
- access mode;
- expiration;
- conflict policy;
- handoff policy.

Default pattern:

```
READ        -> parallel
ANALYZE     -> parallel
PROPOSE     -> parallel
WRITE       -> coordinated
COMMIT      -> serialized or explicitly resolved
```

Locks and leases must be domain-neutral; Git worktrees are one implementation
of the broader concept, not the definition.

## 8. Collaboration

Participants have:

```
Private Context
  +
Shared Task State
```

Shared state may include:

- objective;
- approved plan;
- artifacts;
- verified facts;
- decisions;
- current task state;
- resource status;
- approvals.

Private context may contain:

- temporary exploration;
- model-specific reasoning state;
- personal notes;
- unshared intermediate discoveries.

Collaboration supports:

- observe;
- guide;
- pause;
- take over;
- approve/reject;
- annotate;
- delegate;
- hand off;
- resume.

A participant should be able to join an existing task without inheriting an
unbounded transcript.

## 9. Multi-environment orchestration

A single logical task may simultaneously use:

- browser sessions across multiple sites;
- local terminals;
- remote terminals;
- interactive sandboxes;
- persistent hosts;
- desktop environments;
- APIs/MCP;
- CI/batch providers;
- human gates.

The orchestrator schedules execution based on:

- required capabilities;
- data locality;
- permissions;
- security policy;
- platform requirements;
- persistence;
- latency;
- provider health;
- quota;
- cost;
- availability.

Environment choice is an execution decision, not part of task identity.

## 10. Browser orchestration

Browser work supports both:

**Interactive mode**

For exploration, unknown UIs, visual tasks, human takeover, and one-off work.

**Programmable mode**

For reproducible procedures and repeated operations using automation programs and
fresh browser sessions.

A successful interactive procedure may be promoted into a reusable Procedure.

Multiple sites belong to one task while preserving per-site/session/account
isolation.

The browser is an access surface; it is not the durable source of truth for the
task.

## 11. Tool registry and dynamic tool exposure

Tool definitions consume context and may overlap in what resource they modify.

The platform therefore supports:

- tool discovery;
- lazy schema loading;
- context-aware tool selection;
- duplicate/overlapping tool detection where practical;
- authorization-aware exposure;
- resource-aware routing;
- bounded tool-result retention.

Tool results should be stored as references to durable/bounded payloads rather
than automatically injected in full into every later context.

## 12. Structured compaction and reset

When context pressure rises, Flauz should preserve structured state rather than
produce only a generic conversation summary.

The durable state should capture at least:

- objective;
- constraints;
- current plan;
- implementation/execution state;
- decisions;
- files/resources touched;
- important observations;
- evidence;
- known failures;
- unresolved questions;
- pending approvals;
- next actions.

Two operations are distinct:

```
COMPACTION
  preserve continuity while shrinking context

RESET
  reconstruct a fresh context from durable task state
```

The agent must not be encouraged to prematurely terminate solely because the
current model context is nearly full.

## 13. Model-aware context compilation

A logical task/session must survive switching models.

The Context Engine may compile the same durable task state differently for:

- GPT-family models;
- Claude-family models;
- Gemini;
- Copilot;
- local models;
- custom endpoints.

Model context capacity, multimodal behavior, tool-schema handling, caching and
serialization constraints belong in a ModelContextProfile.

Switching models must not silently discard durable task state.

## 14. Procedure

A Procedure is a durable, reusable representation of a successful way to
accomplish an objective.

Conceptually:

```
Procedure
  objective
  preconditions
  inputs
  steps
  required capabilities
  resource bindings
  validation rules
  recovery rules
  version
  evidence
```

Lifecycle:

```
one-off task
   -> successful execution
   -> suggest "Save as Procedure"
   -> user reviews
   -> Procedure
   -> reusable execution
   -> observe deviations
   -> improve
   -> new version
```

Procedures are domain-neutral. Examples include:

- software release;
- reconcile a spreadsheet;
- prepare a weekly report;
- onboard a customer;
- collect research evidence;
- perform a browser-based administrative task;
- run a creative production pipeline.

User-facing terminology should normally be **Procedures** or **Reusable
workflows**, with explanation on first exposure.

Procedures must never be hidden behind an advanced settings page or require
users to know the internal type name.

## 15. Harness observability

Flauz must eventually measure:

- context utilization;
- relevant-token ratio;
- retrieval precision/recall;
- context duplication;
- tool-schema overhead;
- compaction loss;
- context cache hit rate;
- stale-context rate;
- recovery success;
- task completion;
- verification outcomes;
- environment conflicts;
- human escalation.

Harness improvements should be evaluated as measurable experiments rather than
unbounded prompt changes.

## 16. General-purpose requirement

Nothing in this architecture assumes that a Project is a software repository.

A Project may contain:

- objectives;
- stakeholders;
- tasks;
- resources;
- artifacts;
- agents;
- environments;
- procedures;
- evidence;
- policies;
- history.

Software repositories, tests and deployments are one domain specialization.

The same primitives must support research, finance, operations, education,
creative production, administration, healthcare-support workflows, and future
domains without introducing domain-specific execution architecture into the
platform core.
