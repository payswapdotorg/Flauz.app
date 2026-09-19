# Flauz Architecture & Implementation Constitution

**Status:** FROZEN — base architecture 2026-09-18; approved runtime/UX extension 2026-09-19

This is the authoritative product/architecture contract for Flauz. It governs the
approved evolution from the current native Codex-compatible desktop client into
a multi-client, multi-model, multi-agent, multi-environment shared agent
workspace.

## Source-of-truth precedence

1. `AGENTS.md`: repository/process/safety rules.
2. This document: frozen product and architecture decisions.
3. `docs/IMPLEMENTATION-ROADMAP.md`: phase order, dependencies, and status.
4. The assigned work order: concrete scope and acceptance.
5. `docs/architecture.md`: detailed implementation boundaries.
6. `docs/parity-matrix.md`: Codex Desktop parity only.
7. `docs/CONTEXT-HARNESS-ARCHITECTURE.md`: detailed approved context/harness/runtime extension.
8. `docs/PRODUCT-UX-JOURNEYS.md`: approved product journeys and discoverability contract.
9. Code/tests are implementation evidence, not permission to redefine contracts.

External documentation/research may provide evidence, but it does not change
Flauz requirements. Architecture changes require a repository-recorded
amendment/ADR before implementation.

## Immediate priority

Finish Codex Desktop parity first. Do not pause parity to implement the future
platform.

During parity closure, only narrow scaffolding explicitly needed to avoid
hard-coupling is allowed. Do not implement Web, Mobile, arbitrary model-provider
routing, remote providers, multi-user collaboration, or multi-model orchestration
without an assigned future work order.

Workflow/Pack development is frozen. It is not the new execution architecture.

## Frozen architecture

```
Client != Model != AgentRuntime != Skill != Environment != Provider
```

### Client adapters

Target clients:

- Linux desktop
- Windows desktop
- macOS desktop
- Web
- Mobile

Desktop clients can control **local and remote** environments.

Web/mobile use the same Workspace/Session model and are not separate products.

### Environment

An Environment is where execution actually happens.

Examples:

- local Linux/Windows/macOS;
- E2B;
- Daytona;
- Azure;
- Vercel Sandbox;
- Cloudflare;
- GitHub Actions;
- Codemagic;
- future/custom infrastructure.

No provider is the canonical environment.

### Providers

Providers are pluggable infrastructure sources. Support interactive sandboxes,
persistent hosts, desktop hosts, and batch/CI providers as appropriate.

Users can connect their own provider accounts. Provider quota belongs to the
user-owned connection where supported.

Provider credentials must never be committed, logged, exposed to model context,
or stored in fixtures/screenshots.

### Models and agent runtimes

A Model is intelligence. An AgentRuntime is the runtime/orchestration loop.

First-class runtime targets:

- official Codex app-server;
- GitHub Copilot runtime/SDK;
- Flauz direct-model runtime;
- future/custom runtimes.

Model-provider targets include:

- OpenAI/Codex;
- OpenAI API;
- Google Gemini;
- Anthropic;
- GitHub Copilot/BYOK;
- Ollama;
- LM Studio;
- OpenAI-compatible custom endpoints;
- OpenRouter;
- future adapters.

Codex app-server is a first-class runtime, not the universal Flauz model registry.

### Skills and capabilities

Skills are reusable, model-agnostic behavior packages with explicit capability
requirements.

Representative capabilities:

```
terminal
filesystem.read / filesystem.write
git
browser / browser.navigation / browser.input
computer.screen / keyboard / mouse / window
desktop.gui
vision
image.input / image.output
web.search
mcp
ports
ssh
snapshots
persistent_storage
long_running
gpu
```

Availability is resolved from:

```
model capabilities
∩ runtime capabilities
∩ environment/provider capabilities
∩ permissions
∩ workspace policy
```

Computer Use is a compound capability: model ability + tool protocol +
environment GUI/input + permission/policy.

A missing capability must produce an actionable diagnosis and an unlock path
when one exists.

### Multi-model / multi-agent orchestration

A single skill may be fulfilled by multiple independent models/agents.

Required future patterns include:

- serial delegation;
- parallel specialists;
- model-as-tool;
- planner/worker/reviewer roles;
- cross-model verification;
- fallback/rerouting;
- shared artifacts/context.

### Multi-environment orchestration

One workflow may concurrently use environments from multiple providers.

Example:

```
Architect -> model A
Coder -> E2B
Windows QA -> Azure
macOS QA -> Codemagic
CI -> GitHub Actions
Linux integration -> local Linux
```

The orchestrator owns dependencies, lifecycle, cancellation and provenance.

### Collaboration

Workspace is the collaboration root for:

users, devices/clients, projects, sessions, agents, environments, skills,
documents, sheets, artifacts, presence, permissions and policy.

Shared documents/sheets may use true real-time collaboration. Code must preserve
Git/worktree semantics and distinguish isolated worktrees from intentional
shared-filesystem mode.

## Approved runtime architecture evolution — 2026-09-19

The Flauz architecture now explicitly includes the following cross-cutting
primitives:

- **Context Engine** — compiles model context from durable state, memory,
  retrieval, tools, environment state, policy and provenance.
- **Harness** — manages task state, execution, observation, verification,
  recovery and context around an AgentRuntime.
- **Task Execution Graph** — reactive dependency graph supporting parallelism,
  conditions, loops, waits, human gates, cancellation and recovery.
- **Resource Graph** — canonical resource identities decoupled from browser,
  API, CLI, MCP and other access surfaces.
- **Evidence plane** — claims and observations are distinct from verified
  evidence.
- **Leases/conflict management** — concurrent reads are distinct from
  coordinated writes and commits.
- **Procedure** — durable, reusable, versioned representations of successful
  work.
- **Collaboration state** — private agent/user context is distinct from shared
  task state.
- **Harness observability** — context, retrieval, execution, verification and
  recovery are measurable.
- **General-purpose project model** — project/task/resource/artifact concepts
  are domain-neutral; software development is only one specialization.

The detailed contracts are recorded in
[docs/CONTEXT-HARNESS-ARCHITECTURE.md](CONTEXT-HARNESS-ARCHITECTURE.md).

### GUI discoverability is part of product correctness

A platform primitive is not complete merely because the runtime path exists.
Major capabilities must have discoverable product journeys recorded in
[docs/PRODUCT-UX-JOURNEYS.md](PRODUCT-UX-JOURNEYS.md).

At minimum, applicable capabilities must have:

- a visible primary entry point;
- a contextual entry when the capability becomes relevant;
- a global search/command-palette fallback;
- a useful empty state for first discovery;
- a success or next-step affordance.

The command palette may accelerate discovery but may never be the sole entry
point for a major capability.

**Procedure is explicitly discoverable.** A successful repeatable task should
be able to surface a user-facing "Save as a reusable workflow" path; Procedures
must also have a persistent library, search, task-context suggestions, readable
detail, execution, deviation visibility and intentional improvement/versioning.

The GUI must use domain-neutral concepts such as Project, Task, Procedure,
Resource, Environment, Agent, Artifact, Evidence, Activity and Context.
Software-specific concepts remain contextual specializations.

### Architectural separation

The platform must preserve these distinctions:

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

Durable state cannot depend on the current model context window. A model switch,
context reset, environment switch or agent handoff must preserve the logical
Task and its authorized durable state.

## Implementation invariants

- Keep official Codex live state behind the official app-server.
- Keep native desktop runtime free of Electron/Tauri/Wry/WebView/Node.
- Keep provider code behind provider boundaries.
- Keep model-specific behavior behind model/runtime adapters.
- Keep skill semantics Flauz-owned.
- Keep all remote execution attributable, bounded, cancellable and permissioned.
- Every implementation change maps to a named work order.
- Workers never silently redefine architecture.
