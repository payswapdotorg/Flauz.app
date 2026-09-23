# Wave-3 Integration Gate Record — PASSED

> **Status: PASSED 2026-09-23 — WAVE 3 (F6 orchestration) CLOSED by the
> Tech Lead.** Final code head: **db79499** (PR #49 merge cd59421 +
> PR #50 merge ef43a0e + PR #51 merge 82ae6af + evidence/ledger docs
> 8da4468/eff1322 + the Lead Gate-B gate-fix db79499). Authority: the
> WAVE3-WORK-ORDERS ledger + each work order's GUI/lab-evidence clause;
> the wave closes only after merged implementation AND evidence — both
> in place below. The Wave-3 ledger: 3/3 MERGED, deviations NONE ×3
> (ORCH-002 PR #49, ORCH-004 PR #50, ORCH-003 PR #51).

## Gate A — the 18-step integration harness (the F6 domain-neutral
scenario): GREEN

The Lead-authored harness (`lead-tools/f2-integration-harness`,
verification infrastructure NOT a workspace crate; the Wave-3 step
archived here: `f6_scenario_step.rs`; run log `gate-a-run-main-82ae6af.log`)
wires the merged crates through the frozen formats. Steps 1–17 are the
F2/Wave-2 gate (unchanged, re-proven at the merged tree); the Wave-3
addition:

- **[18] the F6 domain-neutral scenario** — "Research, analyze and
  independently verify the three most-cited renewable-energy policy
  shifts of 2026" (the non-software-development run the F6 gate
  requires): world setup (browser resource + fake browser sandbox +
  3 attributed agents + both fake models) → the ORCH-003 TaskHarness
  drives prepare/execute/observe/verify/persist/continue with the REAL
  ORCH-002 engine (`compile_from_durable_state`) behind the
  ContextCompiler seam → mid-run model switch via reprepare (the
  rebuild law: two compiles from the same durable state are
  projections-equivalent; different state is NOT; canonical task state
  unchanged) → serialize → drop → replay → resume → recover(brief) →
  continue with the turn counter continuing (3 turns) → the ORCH-004
  attributed graph (research/analysis/review over the WORLD-side
  artifact/evidence ids) → the merge point with both workers Verified +
  the review Done + the 3-entry attribution ledger →
  `task.verification_merged` appended world-side → the machine's own
  history lands on the task stream (harness.* event types) →
  Save-as-a-reusable-workflow (Procedure 1.0, steps = what actually
  ran) → run-again as a NEW task (graph2 validates; the source task's
  stream untouched; both identities intact; the source stream continues
  after the save) → world snapshot/restore equality + the
  no-credential-material scan. Three build-law lessons fixed en route:
  model capabilities sorted/deduped; harness persist refs
  sorted/deduped; the RUNTIME's advertised capabilities (not the
  model's) gate the turn (the vision turn exercises browser.input
  through FakeNonCodexRuntime — the honest capability interplay).

Result: `== F2 INTEGRATION HARNESS: ALL STEPS PASS ==` (18 steps) at
merged main.

## Gate B — GUI discovery evidence: GREEN (binary
codexrs-w3gate-fix1, eff1322 + the db79499 gate-fix)

Scene `d25-w3-surfaces.sh` (archived here: `gate-b-scene-d25.sh`; the
final run log `gate-b-run-d25.log`; frames at
`parity-lab/evidence/d25-fix1/`; VLM reads archived as
`gate-b-vlm-*.json`) — cold start → anchor task → the four Wave-3 UI
surfaces:

- **J-03 the recovery surface (ORCH-003)**:
  - layer 4 (keyboard): Ctrl+Alt+Shift+R surfaces the honest
    nothing-to-pick-up guidance through the command-status line —
    verbatim VLM read at the fixed binary (B03):
    *"This task is up to date — there's nothing to pick up right now.
    [Dismiss]"* — and NO banner renders (never a permanent banner;
    the banner is state-driven);
  - layer 3 (palette): the row **"Resume this task where it left
    off"** / *"Pick up an interrupted task with everything it kept"*
    filters + Return lands the same guidance (B05/B05b);
  - nothing traps: Escape settles; the task surface stands (B04 ==
    B02 pixel-identical);
  - **the gate-fix story (the Gate doing its job)**: at the pre-fix
    binary the chord produced NO visible change (B03 == B02
    pixel-identical) while the palette row worked — root-caused via
    the discriminating probe (`gate-b-probe-d25.sh` +
    `gate-b-probe-run.log` + `gate-b-vlm-d25-probe-02.json`: the
    retry-status race excluded; the 17-pixel caret-blink diff
    identified as animation, not the guidance) to a MISSING
    `on_action` listener — the KeyBinding dispatched
    `FlauzRecoveryShortcut` into the void. Lead gate-fix db79499 adds
    the listener on the workspace root (next to the picker/gap/
    agents/save chord listeners) + the regression guard in the seam
    test (a binding without a listener is a silent no-op — the seam
    test now pins the listener FORM, not just the binding). Verified:
    the fixed binary's B03 ≠ B02 with the verbatim guidance; the
    systemic check (all 14 Flauz chord actions × listeners) shows no
    other gap.
- **J-07 the agents view (ORCH-004)**: layer 1 — the rail "Agents"
  tab on the task surface (B02's tab bar); layer 4 — Ctrl+Alt+Shift+7
  (through the N6 shifted-symbol companion) opens the panel (B06);
  the honest states verbatim (VLM): heading **"Agents on this task"**,
  empty title *"No extra agents on this task yet"*, the
  parallel-work body (*"one agent researches while another analyzes,
  and an independent reviewer checks the results. Everything each one
  produces stays separate and attributed to its maker"*), the
  what-to-do-next guidance, and the honest wiring state: *"Live agent
  progress is on its way … No agent is invented here before that
  wiring lands."*; layer 3 — the palette row **"See who is working on
  this task"** / *"Watch each agent's role, progress and what they're
  waiting for"* + the chord hint, Return lands the SAME panel (B09b ==
  B06 pixel-identical); scoped Escape (the rail's) closes (B08).
- **J-10 the save flow (ORCH-004)**: layer 1 — the task-surface entry
  affordance **"☆ Save as a reusable workflow"** / *"Flauz can
  remember the successful steps so you can run them again"* (B02/B10);
  layer 4 — Ctrl+Alt+Shift+S opens the panel: the expanded header
  *"Save as a reusable workflow"* + *"← Back to your work"* + the
  honest not-wired state verbatim: *"Saving isn't wired to live tasks
  yet"* + *"the steps that actually ran — exactly what happened,
  nothing aspirational"* + the next step + *"Escape closes this
  panel"*; layer 3 — the palette row **"Save this task as a reusable
  workflow"** / *"Remember the successful steps of this task so you
  can run them again"* / Ctrl+Alt+Shift+S, Return lands the same panel
  (B13b == B10 pixel-identical); scoped Escape closes (B12).
- **J-11 the workflows list (the F2 shell surface, upgraded by
  ORCH-004's list states)**: Ctrl+Alt+2 → the surface (B14): list
  title *"Reusable workflows"* + subtitle *"Successful steps, saved so
  you can run them again"* + the honest empty state *"No reusable
  workflows yet"* + the body + the what-to-do-next pointing at the
  save affordance; the sidebar's Workspace section highlights
  "Reusable workflows" (the layer-1 nav).

Copy contract: user language throughout — "Resume this task where it
left off", "See who is working on this task", "Save as a reusable
workflow", "No reusable workflows yet"; never "Procedure", never
"context window", never an internal type name.

Frame-stability note: the palette/panel frames are pixel-identical
between the pre-fix and post-fix runs (B05/B05b/B06/B09b/B10/B13b/
B14/B15 hashes equal) — the gate-fix changed exactly one behavior (the
chord) and nothing else.

## Gate C — CI: GREEN

- The code-final Wave-3 head **eff1322** (the ORCH-003 merge tree +
  evidence/ledger docs): CI run **35824388043** — ubuntu-24.04
  SUCCESS + windows-latest SUCCESS (the full gate: clippy --workspace
  --all-targets -D warnings + test --workspace + release build +
  startup smoke).
- The Gate-B gate-fix **db79499**: CI green both platforms (the
  clippy/test/build gate re-run at the fix head; the fix is
  codex-app-only — ui.rs + the seam test — and the seam test's
  assertions were additionally verified by exact local simulation
  before the push: the codex-app test binary cannot LINK in this
  sandbox — the pre-existing libpipewire gap documented in the
  ORCH-003 report; clippy --all-targets (0 errors), flauz-exec
  106/106 locally, and the CI suite stand in, the ORCH-003 merge
  precedent).

## The wave record

- 3/3 work orders merged, deviations NONE ×3, evidence:
  - ORCH-002: `../w3-orch-002/ORCH-002-COMPLETION-REPORT.md`
  - ORCH-003: `../w3-orch-003/ORCH-003-COMPLETION-REPORT.md`
  - ORCH-004: `../w3-orch-004/ORCH-004-COMPLETION-REPORT.md`
- The ledger: `../../WAVE3-WORK-ORDERS.md` (header: 3 of 3 MERGED —
  DELIVERED).
- The known limitations carried forward: the recovery banner's
  recoverable-state path and the agents/save panels' live wiring are
  engine-side (proven at Gate A) — the GUI view-models populate when
  the harness wiring lands in a later wave (the honest not-wired
  states say so in user words); the palette "Task evidence" Return
  from a shell-surface focus context (the d23 minor note) remains a
  Wave-4+ follow-up.
- The d25 lesson recorded for every future wave: **a KeyBinding
  without an on_action listener is a silent no-op** — the Wave-4
  orders' acceptance criteria now require the listener seam-tested,
  not just the binding (PROV-001/COL-001 both carry it).
- Wave 4 (the parallel wave: PROV-001 BYOP + LAB-001 labs + COL-001
  collaboration) is dispatched per `../../WAVE4-WORK-ORDERS.md` — the
  frozen addendum + the wave integration gate spec (the three-fabric
  scenario).
