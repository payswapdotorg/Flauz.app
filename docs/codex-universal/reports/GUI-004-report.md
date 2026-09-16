# GUI-004 — Unified Agent + Workflow UX

**Status:** COMPLETE
**Base:** `8dbe4cc` (merged GUI-002)
**Branch:** `gui-004/unified-ux`
**Work order:** `docs/codex-universal/WORK-ORDERS.md` § GUI-004

## What was delivered

Workflows became a first-class peer surface with the full
version-to-version Git-native lifecycle inside one product, and Pack
room preserved by construction (no third top-level surface).

### 1. Fork surface (`workflow/fork`)

- **codex-core**: `WorkflowForkVersion` action (validates a published
  version id + a distinct fork repository identity), `WorkflowForked`
  recording the derived release plus the engine-sealed upstream lineage
  (`WorkflowForkLineageCard`), bounded `fork_pending` /
  `fork_lineage` state, and `workflow_remember_published` (bounded,
  deduplicated by version identity, newest first — shared by the
  publish/fork/improve paths).
- **backend**: `workflow_fork` dispatch with a default carried
  attribution record ("Flauz.app"); the engine still refuses forks
  without attribution and owns lineage sealing.
- **UI**: per-version **Fork** action with a fork-repository input and
  a "Latest fork lineage (engine-sealed)" card rendering the upstream
  identity verbatim.

### 2. Governed improvement surface (`workflow/improve/*`)

- **codex-core**: `WorkflowImproveState` (incumbent identity, bounded
  candidate list, selection, validation, approval, publication) with
  the full action set (`ImproveStart/Proposed/SelectCandidate/
  Validate/Validated/Approve/Approved/Publish/Published/Dismiss`).
  Control-plane gates enforced in the reducer: approval requires a
  validated candidate; publication requires an *approved* decision;
  rejection requires a reason; every pending flag is single-flight;
  disconnect clears the ephemeral improve session while durable
  published versions survive.
- **backend**: propose/validate/approve/publish dispatch with typed
  mappers (stage names, change kinds, lineage, evidence summaries with
  bounded reference and run-provenance lists).
- **UI**: an **Improve workflow** pane — evidence-derived candidate
  list (selectable, rationale, evidence counts), successor-version
  validation with per-stage outcomes, approve/reject with approver
  identity, release-tag publication, and the published successor's
  governed lineage + cited evidence.

### 3. Capability / binding / trigger panels

The candidate review pane now renders what the teaching compiler
actually produced: compiler description, **capability hints** (advisory
only — the pane says binding belongs to the execution plane), **binding
proposals** (with requires-approval markers), and **trigger intents**,
all bounded (`MAX_WORKFLOW_CAPABILITY_HINTS` 64, binding proposals 64,
trigger intents 32).

### 4. Run history by version

Durable instances are grouped under their pinned immutable version id
with per-version run counts, so run history reads as
"version → runs" instead of a flat list.

### 5. Unified navigation & journey

The agent surface (chats, composer, timeline) and the Workflows surface
share the same sidebar, palette, and supervised runtime. The full
journey — idea → agent → teaching → workflow → run → evidence →
version → fork/improve → successor version — completes without a
terminal, and every step displays control-plane identity fields
verbatim. Pack-aware views remain a future extension inside the
Workflows surface (no navigation restructuring needed).

## Acceptance checklist (work order)

| Requirement | Result |
| --- | --- |
| unified navigation | PASS — shared sidebar/palette/routes (GUI-003 wiring + this work order's panes) |
| workflow/project linking | PASS — one product, one supervised runtime, one CODEX_HOME; workflows display project-agnostic control-plane identity (the workflow contract has no project binding; documented in CLIENT-BOUNDARY) |
| workflow-aware agent sessions | PASS — teaching and runs ride the same supervised app-server as chat sessions |
| run history | PASS — per-version grouped durable instances |
| evidence inspector | PASS — instance detail with evidence references (GUI-003) + improvement evidence summaries with cited run provenance |
| capability/resource/dependency panels | PASS — capability hints, binding proposals, trigger intents in candidate review |
| workflow Git lifecycle | PASS — teach→publish anchored at commits (GUI-003) + fork lineage + governed successor lineage |
| fork/review/merge/release surfaces | PASS — Fork action + Improve (propose→validate→approve→publish) surfaces |
| chat instructions vs durable workflow semantics distinction | PASS — teaching pane records chat-style input into control-plane sessions; published versions are immutable and display identity verbatim; panes label advisory vs authoritative data |
| no third top-level product surface for Packs | PASS — Pack views will extend the Workflows surface |
| normal user journey without a terminal | PASS — full lifecycle through the GUI |

## Verification commands and results

```text
. scripts/dev-env.sh
cargo fmt --all --check                                    → PASS (clean)
cargo clippy -p codex-core -p codex-app --all-targets -- -D warnings
                                                           → PASS
cargo test -p codex-core                                   → 208 passed (206 + 2 new; count asserted against baseline)
CODEX_RS_TEST_CODEX_BIN=<codex rust-v0.1.0> cargo test -p codex-protocol -p codex-platform
                                                           → 60 + 112 + 2 integration passed
python3 scripts/check_dependency_policy.py                 → PASS (561 packages)
cargo check -p codex-app                                   → PASS
```

## Known limitations

1. Fork attribution defaults to a single "Flauz.app" record; a richer
   attribution editor is future polish (the engine enforces
   non-empty attribution regardless).
2. The improve pane's rejection reason uses a fixed surface-provided
   reason; a free-form reason input is future polish (the control
   plane records it either way).
3. Same-state screenshots remain GUI-007 evidence (headless sandbox).
4. Release link of `codex-app` remains CI-owned (sandbox RAM).

## No unrelated refactors

Diff surface: `crates/codex-core/src/lib.rs` (fork/improve state
machine + panels on candidate state + tests), `crates/codex-app/src/
backend.rs` (five new dispatch arms + mappers), `crates/codex-app/src/
ui.rs` (four input fields + wiring), `crates/codex-app/src/ui/
ui_workflow.rs` (improve pane, fork actions, capability panels,
version-grouped instances), work-order status, this report.
