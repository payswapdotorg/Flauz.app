# Codex Universal GUI Integration Architecture

**Status:** ACTIVE — implementation authority for the Flauz.app GUI integration program
**GUI base:** `payswapdotorg/Flauz.app` (fork of `Kiwunaka/codexRS`)
**Universal runtime:** `payswapdotorg/codex`
**Reference client:** codexRS / OpenAI Codex Desktop behavioral surface

## 1. Mission

Flauz.app is the native desktop GUI client for Codex Universal.

The forked codexRS implementation is the GUI foundation. Do not rebuild the Desktop GUI from scratch. Preserve its native GPUI architecture, app-server supervision, protocol implementation, platform layer, Git/worktree handling, terminal, Browser, Computer Use, settings, persistence, and release machinery unless a concrete integration requirement requires a bounded change.

`payswapdotorg/codex` remains the authoritative Universal runtime and Workflow Platform. Flauz.app is a client/presentation layer over those contracts.

## 2. Runtime ownership

```text
Flauz.app GUI
   |
   +--> codexRS native state/UI/platform implementation
   |
   +--> official Codex app-server / app protocol
   |
   +--> Codex Universal workflow/application contracts
   |
   +--> Universal runtime/model/workflow/execution services
```

The GUI MUST NOT create:

- a second agent runtime;
- a second workflow engine;
- a second skills/plugins runtime;
- provider-specific workflow semantics;
- direct writes to live `CODEX_HOME` state owned by app-server;
- alternate durable workflow authority in GUI state.

## 3. Two product surfaces

The desktop client has two tightly integrated surfaces:

### Agent surface

Maintain codexRS parity for:

- projects and chats;
- composer;
- streaming timeline;
- attachments;
- model/reasoning/permission selection;
- approvals;
- Git, branches, worktrees and diff review;
- terminal and files;
- Browser;
- Computer Use;
- Skills, plugins and MCP Apps;
- settings/account/configuration;
- notifications/background work;
- restart/reconnect behavior.

The codexRS parity matrix remains the GUI parity reference.

### Universal Workflow surface

Add first-class GUI affordances for:

- workflow library;
- create/teach workflow;
- teaching modes `DEMONSTRATE`, `INSTRUCT`, `HYBRID`;
- live teaching session state;
- trajectory/evidence review;
- compilation and semantic validation;
- capability/resource/dependency inspection;
- workflow approval/publication;
- immutable version history;
- run/instance monitoring;
- step timeline and evidence;
- pause/resume/cancel/recovery;
- fork/branch/review/merge/release;
- subworkflow composition;
- scheduling/triggers;
- installation/resource binding;
- workflow sharing/distribution.

## 4. Universal architecture compatibility

The Codex Universal frozen architecture remains authoritative for workflow semantics, model portability, execution environments, evidence, resources, security, Git-native workflow identity and durable control-plane transitions.

The GUI may display or request those semantics but never redefine them.

For example:

```text
GUI: "Teach workflow"
      |
      v
Universal TeachingSession / compiler
      |
      v
WorkflowCandidate / WorkflowIR
      |
      v
Control-plane approval
      |
      v
Immutable WorkflowVersion
```

The GUI must not encode the workflow graph as UI-only state and then invent its own execution semantics.

## 5. Integration boundary

Prefer a typed app/application protocol boundary.

The existing codexRS app-server supervisor remains responsible for the official Codex app-server lifecycle and live Codex session state.

Universal-specific services should be integrated through explicit client-facing contracts rather than importing the entire `payswapdotorg/codex` runtime into the GPUI process.

A bounded local transport is preferred when a long-running Universal service is required. The transport must define:

- request/response types;
- streamed events;
- correlation IDs;
- capability negotiation;
- bounded frames;
- authorization/error semantics;
- version compatibility;
- restart/reconnect behavior;
- redaction and secret handling.

Do not couple Universal workflow semantics directly to GPUI reducers or widgets.

## 6. Reuse before extension

Before adding code, inspect existing codexRS implementations and existing Universal crates/contracts.

Prefer:

1. existing codexRS UI/state/platform primitive;
2. existing OpenAI Codex app-server/public protocol;
3. existing Universal workflow/model/execution contract;
4. a small explicit adapter;
5. only then a new abstraction.

## 7. Product goal

The resulting desktop application should feel like a first-class Codex desktop client while extending that interaction model to the Universal workflow product.

A user should be able to:

1. open a project;
2. chat with an agent;
3. teach an arbitrary computer workflow;
4. review the resulting workflow;
5. publish an immutable version;
6. run it;
7. inspect live progress/evidence;
8. recover or resume it;
9. fork/improve/share it;
10. combine workflows and schedule them.

The user should not need the terminal for the normal workflow lifecycle.

## 8. Branding

Do not make irreversible branding changes solely because the repository is named `Flauz.app`.
The implementation may use `Flauz.app` as the working product/repository name while the Codex Universal product naming and final distribution identity are finalized separately.

## 9. Non-goals

- Reimplementing codexRS UI components solely to change technology.
- Replacing the Codex runtime.
- Replacing the existing workflow engine.
- Building a web UI as a shortcut for desktop delivery.
- Adding Electron, Tauri, Wry, WebView or Node runtime dependencies.
- Directly reading/writing live `CODEX_HOME` databases or logs.
- Moving durable workflow authority into local UI state.
