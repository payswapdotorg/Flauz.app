# Flauz Product UX Journeys & Discoverability Contract

**Status:** APPROVED PRODUCT/UX CONTRACT — 2026-09-19

Flauz must make the capabilities in the Context, Harness, Resource, Execution,
Evidence, Procedure, and Collaboration architecture discoverable through the
GUI.

A capability that exists technically but is not discoverable through normal
user behavior is not considered product-complete.

## 1. Discoverability rules

Every major capability must have all applicable discovery layers:

1. **Primary visible entry** — a normal user can see where to start.
2. **Contextual affordance** — the action appears when the current task makes it
   relevant.
3. **Universal search/palette fallback** — users can search for the capability
   without knowing its exact location.
4. **Stateful empty state** — an empty list explains what the capability is and
   provides a concrete first action.
5. **Success-state continuation** — completing a task exposes the next useful
   action, such as saving a successful task as a Procedure.

The command palette is a fallback, never the only discovery path for a major
capability.

## 2. Primary product surfaces

The GUI should converge on a small number of recognizable surfaces rather than
a large collection of product-specific mini-apps.

### Workspace surface

The user's durable home for:

- Projects;
- Tasks/sessions;
- Procedures;
- Artifacts;
- shared work.

The navigation should show meaningful counts/attention state where useful, but
must remain bounded and uncluttered.

### Task surface

The primary work area for one logical task.

It should make these states directly discoverable without leaving the task:

- current objective;
- agent/model;
- Context;
- Agents;
- Environments;
- Resources;
- Artifacts;
- Evidence;
- approvals/human gates;
- execution history.

Use contextual side panels, drawers or inspectors rather than turning each into a
new top-level product unless scale/evidence requires it.

### Procedure surface

A dedicated, discoverable library for reusable Procedures.

Minimum sections:

- Suggested / recently learned;
- My Procedures;
- Shared Procedures;
- Versions/history;
- Search/filter.

A Procedure can be launched from this surface, attached to a task, inspected,
edited/improved, shared, duplicated/forked, or archived.

### Activity / collaboration surface

A discoverable view of:

- active work;
- agent activity;
- human collaborators;
- waiting approvals;
- failures requiring attention;
- completed outcomes.

It should answer "what needs my attention?" without requiring users to inspect
every task.


## 2.1 Target GUI information architecture

The product should expose capabilities through a small number of stable surfaces,
with contextual inspectors rather than a growing list of unrelated top-level
screens.

### Workspace navigation

The persistent navigation should make these concepts discoverable:

- Projects / Tasks;
- Procedures;
- Artifacts;
- Activity / attention;
- shared work where applicable.

### Task control rail

A task should expose visible, labeled controls for the most important cross-cutting
state:

- **Context** — what the agent currently knows and why;
- **Agents** — participating agents, roles and assignments;
- **Environments** — browser sites, terminals, sandboxes, desktops and other
  execution surfaces;
- **Evidence** — observations, claims, verification and provenance;
- **More / Inspect** — Resources, Artifacts, approvals, leases/conflicts and
  detailed activity when these are not appropriate for the primary rail.

These controls should remain available while work is active. They are not
hidden behind developer settings.

### Composer and timeline contextual actions

The task composer/timeline should surface context-sensitive actions such as:

- Use a reusable workflow;
- Add environment/site;
- Delegate to another agent;
- Inspect context;
- Take over;
- Approve/reject;
- Save as a reusable workflow;
- Retry / recover / switch environment;
- Inspect evidence.

The exact controls shown depend on task state, permissions and capability
availability.

## 3. Required major journeys

### J-01 Start any project

```
Workspace
  -> New Project/Task
  -> describe objective in natural language
  -> Flauz suggests relevant capabilities/environments/agents
  -> user starts immediately or refines
```

No software-project terminology is required.

### J-02 Understand what the agent knows

```
Task
  -> Context indicator
  -> Context Inspector
  -> see current context sources
  -> inspect why an item is present
  -> pin/remember or remove where policy allows
```

The active context budget/usage should be visible in a compact status affordance.

### J-03 Recover/continue a long-running task

```
Task
  -> context pressure / disconnect / interruption
  -> Flauz preserves durable task state
  -> compact or reset transparently
  -> resume with reconstructed context
  -> show recovery state/evidence
```

Users should not need to understand context windows to resume work.

### J-04 Discover a capability gap

```
Agent needs capability
  -> task-visible explanation
  -> "What is missing?"
  -> available unlock paths
  -> switch model / connect provider / attach compatible agent /
     attach environment / request approval
```

A failed capability request must never end in a silent no-op.

### J-05 Add another environment/site

```
Task
  -> Environments
  -> Add environment/site
  -> choose or discover provider/environment
  -> authorize
  -> attach
  -> environment appears in task topology
```

For browser work, multiple sites/accounts remain distinct sessions while being
part of the same logical task.

### J-06 Coordinate browser + terminal + sandbox

```
Task
  -> agent sees Resource/Environment topology
  -> browser observation
  -> terminal analysis
  -> sandbox computation
  -> artifact produced
  -> evidence links the steps
  -> verification
```

The GUI should make cross-boundary movement visible rather than looking like
separate unrelated tool calls.

### J-07 Parallelize work

```
Task
  -> Agents
  -> delegate/specialize
  -> see agent roles and assignments
  -> inspect dependencies
  -> watch parallel progress
  -> merge artifacts/evidence
  -> verify
```

The user should understand what is happening without reading model transcripts.

### J-08 Human takeover / handoff

```
Task
  -> agent requests help OR user chooses Take over
  -> enter browser/desktop/terminal surface
  -> perform or approve action
  -> hand control back
  -> agent resumes with updated state
```

Takeover state must be explicit and visible.

### J-09 Understand what happened

```
Task
  -> Evidence
  -> execution timeline
  -> claims/observations
  -> supporting artifacts
  -> verification status
  -> resource/environment provenance
```

The UI must distinguish claimed, observed, verified, contradicted, stale and
unknown state.

### J-10 Save a successful task as a Procedure

This is a mandatory discoverability journey.

```
successful task
  -> "Save as Procedure"
  -> name / objective / preconditions
  -> review learned steps
  -> save
  -> Procedure appears in Procedures
  -> suggested on similar future tasks
```

The user must not be expected to know that the internal object is called a
Procedure.

Primary copy:

**Save as a reusable workflow**

Secondary copy may explain:

**Flauz can remember the successful steps so you can run them again.**

### J-11 Discover a learned Procedure later

Multiple routes must exist:

```
Workspace -> Procedures
Task -> Use a reusable workflow
Composer/contextual suggestion
Global search/palette
Similar-task suggestion
Post-success suggestion from a previous run
```

A Procedure should never exist solely as hidden agent memory.

### J-12 Reuse / improve a Procedure

```
Procedure
  -> Run
  -> inspect preconditions
  -> execute
  -> verify
  -> show deviations
  -> improve
  -> review successor
  -> publish/version
```

Repeated deviation should surface as an improvement opportunity rather than
silently mutating the Procedure.

### J-13 Collaborate on one task

```
Task
  -> Share / Invite
  -> collaborator joins
  -> presence visible
  -> each participant has private context
  -> shared task state/artifacts/evidence remain canonical
  -> participants can comment, delegate, take over or approve
```

### J-14 Switch model without losing work

```
Task
  -> Model
  -> choose another compatible model/runtime
  -> Context Engine recompiles durable task state
  -> continue
```

The user should see continuity, not a new blank chat.

### J-15 Switch execution environment without losing work

```
Task
  -> Environment
  -> choose compatible environment
  -> transfer/attach required artifacts/state
  -> continue
```

This is especially important for local -> remote, Linux -> Windows, or one
provider -> another.

### J-16 Inspect resource conflicts

```
Task
  -> Resource
  -> conflicting observations or pending write
  -> show conflict
  -> identify source/lease
  -> refresh / handoff / resolve
  -> continue
```

Do not hide browser/API/CLI state divergence.

### J-17 Review activity that needs the human

```
Activity
  -> pending approval / failed environment / collaborator request
  -> open exact task
  -> act
  -> return to Activity
```

This should work across domains, not just code reviews.

### J-18 Discover automation opportunities

After repeated successful executions:

```
Flauz observes repeatability
  -> "This looks repeatable"
  -> suggest reusable Procedure
  -> optionally suggest future scheduling/triggering
```

No silent auto-automation; the user chooses whether to make it reusable.

## 4. Journey-to-surface matrix

| Journey | Primary surface | Contextual entry | Palette/search | Empty/success guidance |
| --- | --- | --- | --- | --- |
| Start project | Workspace | Task composer | Yes | Yes |
| Inspect Context | Task | Context indicator | Yes | Yes |
| Recover task | Task | recovery banner | Yes | Yes |
| Capability gap | Task | capability card | Yes | Yes |
| Add environment | Task | Environment panel | Yes | Yes |
| Cross-environment work | Task | environment topology | Yes | Yes |
| Parallel agents | Task | Agents panel | Yes | Yes |
| Takeover | Task | action/approval card | Yes | Yes |
| Evidence | Task | verification card | Yes | Yes |
| Save Procedure | Task + Procedures | post-success CTA | Yes | Yes |
| Find Procedure | Procedures | task suggestion | Yes | Yes |
| Improve Procedure | Procedures | deviation/result CTA | Yes | Yes |
| Collaborate | Task + Activity | Share/presence | Yes | Yes |
| Switch model | Task | model control | Yes | Yes |
| Switch environment | Task | environment control | Yes | Yes |
| Resolve resource conflict | Task | conflict card | Yes | Yes |
| Human attention | Activity | notification/attention card | Yes | Yes |
| Discover automation | Task + Procedures | repeatability CTA | Yes | Yes |

## 5. Procedure discoverability requirements

A Procedure is not considered shipped until:

- it is visible from a persistent library;
- a first-run empty state explains it in ordinary language;
- successful work can surface a save/reuse action;
- similar future tasks can suggest it;
- it is searchable;
- it has a readable detail page;
- it can be run without opening implementation internals;
- execution shows whether the current run followed or deviated from the
  Procedure;
- the user can improve/version it intentionally.

## 6. General-purpose UX vocabulary

The UI should favor domain-neutral labels:

- Project
- Task
- Procedure
- Resource
- Environment
- Agent
- Artifact
- Evidence
- Activity
- Approval
- Context

Software-specific labels such as repository, branch, test, or deployment should
appear when relevant to the current project, not as the universal information
architecture.

## 7. Accessibility and discoverability

Major journeys must be usable with keyboard navigation and must not depend on
hover-only affordances.

Critical state changes should have:

- visible labels;
- stable focus behavior;
- bounded status messages;
- non-color-only state;
- readable empty states;
- accessible names for major controls.


## 9. Cold-start discoverability validation

A major capability must be discoverable by a user who has not read Flauz's
architecture documentation and does not know internal implementation terms.

For each journey, validation should include:

1. **Cold start:** begin from the normal Workspace/Task surface with no search.
2. **Primary path:** confirm the user can identify the main entry and understand
   what it does from the UI label/description.
3. **Context path:** repeat while the capability becomes relevant during work;
   the contextual affordance should appear at the correct moment.
4. **Recovery path:** verify the capability remains discoverable after an error,
   reconnect, interruption or empty state.
5. **Search fallback:** verify the command palette/global search can locate it.
6. **Success path:** verify the resulting state exposes the next relevant
   action without forcing the user to rediscover the feature.

For the first implementation wave after F1, the Tech Lead should maintain a
GUI journey battery covering at least:

- Context Inspector;
- multi-environment task topology;
- agent delegation/parallel work;
- Evidence/Verification;
- human takeover;
- Procedures: learn -> save -> find -> run -> deviation -> improve;
- collaboration/presence;
- model/environment switching with continuity.

### Procedure-specific cold-start test

A new user completing a successful repeatable task must encounter a visible
post-success option such as **Save as a reusable workflow**. The user must not
need to know the word "Procedure" to use it.

After saving, a later blank task must provide at least one non-search discovery
route to **Use a reusable workflow**, and the Procedures library must explain
what the object is in plain language.

A Procedure should remain visible after restart and should be discoverable from
both the library and relevant task contexts.

## 10. Domain-neutral journey validation

The journey battery must eventually contain at least one non-code workflow,
for example a research, operations, finance, administrative, creative or
analysis task. This prevents the GUI information architecture from silently
reverting to coding-specific assumptions.

## 11. Product-completeness gate

For each new platform capability, the implementation work order must identify:

- journey IDs;
- primary discovery surface;
- contextual affordance;
- search/palette entry;
- empty state;
- success/next-step state;
- evidence used to verify the journey.

A technically complete backend capability without its required product journey
is not considered complete.
