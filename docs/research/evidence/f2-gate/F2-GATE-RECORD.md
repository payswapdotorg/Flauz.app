# F2 Integration Gate Record — PASSED

> **Status: PASSED 2026-09-22 — F2 CLOSED by the Tech Lead.**
> Final main SHA: **9977302** (PR #45 merge d76afd764 + Lead gate-fixes
> fb512f3/9977302 + docs 4692e93). Authority: F2-GATE-PLAN.md; the gate is
> closed by the Lead only after merged implementation AND evidence — both
> in place below. The Wave-1 ledger: 4/4 MERGED, deviations NONE
> (ARCH-001 PR #42, ARCH-002 PR #43, UX-003 PR #44, ORCH-001+UX-001
> PR #45).

## Gate A — kernel §10 integration round-trip: GREEN

The Lead-authored harness (verification infrastructure, NOT a workspace
crate; code + output archived here: `gate-a-harness-*.rs`,
`gate-a-harness-cargo.toml`, `gate-a-run-main-d76afd764.log`) wires the
three merged contract crates together through the frozen formats with the
REAL flauz-context step (the ORCH-001 merge landed it; the placeholder is
gone):

- frozen-format seam: world/exec canonical IDs parse as validated context
  refs (prefix tables identical across crates: sess/task/art/obs/claim/
  env/model/ctxsnap/mem)
- durable memory: HOT + WARM + COLD (jit-only) + Secret (reference-only,
  storable, never compiled)
- dual model profiles (text-only 128k + vision 200k)
- an 8-item durable snapshot — a projection, not a transcript; provenance
  structurally on every item
- authorization-aware compile: 7/8 (Secret excluded), snapshot unmutated,
  task reference preserved untouched
- reconstructibility: `durable_references()` names task/session/model/
  artifact/observation/event/environment/memory
- canonical JSON: `"v":1`, zero credential material, serialize → drop →
  reload → state equality
- model-switch RESET: fresh `ctxsnap` from durable task state (HOT/WARM
  retained, COLD dropped, Secret never compiled), superseded snapshot
  unmutated, same logical task — no fork

Result: `== F2 INTEGRATION HARNESS: ALL STEPS PASS ==` (14 steps + the
envelope↔transport seam, 446 wire bytes).

## Gate B — GUI discovery evidence: GREEN (binary codexrs-f2gate-9977302)

### The shell-discovery scene (d23) — the F2 centerpiece

Four runs to a full pass (calibration → chord discovery → gate-fix r1 →
gate-fix r2); evidence: `parity-lab/evidence/d23/` (final run) +
`d23-run1-calibration/`. Final adjudication, every layer:

1. **Primary navigation**: the sidebar Workspace section renders all four
   labeled entries (Projects & tasks, Reusable workflows, Artifacts,
   Activity); the four-row click ladder reaches all four surfaces.
2. **Contextual affordance**: the task rail renders on the task view
   (Context/Agents/Environments/Evidence/More), visible while a task is
   selected.
3. **Palette**: rows for every surface and rail section with correct
   titles, descriptions, and chord hints ("Reusable workflows …
   Ctrl+Alt+2", "Task evidence … Ctrl+Alt+Shift+4"); Return executes
   (the reusable leg lands the surface).
4. **Honest empty states**: verbatim code copy on every surface and rail
   section — including "No additional agents are assigned. Delegate a
   task when parallel work would help." and Evidence's claims-vs-verified
   copy.
5. **Keyboard paths**: all four surface chords (Ctrl+Alt+1..4) and all
   five rail chords (Ctrl+Alt+Shift+1..5) fire and open their surfaces/
   panels.
6. **Copy contract**: "Finish a task successfully, then choose 'Save as a
   reusable workflow'…" — user-facing language, no internal type words.

### The chord gate-fixes (Lead, post-merge)

d23 runs 2-3 exposed the N6 shifted-keysym family hitting the new rail
chords. Root-cause chain, complete: (a) the house `shortcut()` helper
prepends the platform secondary modifier, so the registered chords are
ctrl-alt-N (the shell's labels honestly display this); (b) Linux xkb
reports Shift+1..5 as the shifted SYMBOLS ("!"…"%" ); (c) GPUI `from_xkb`
then STRIPS the shift modifier for single-char symbol keys (lower==upper
convention) — so the live keystroke is key "!" with ctrl+alt and NO
shift, matching neither the digit form nor a shift-carrying companion.
Fix 9977302 registers SHIFTLESS symbol companions (ctrl-alt-! …
ctrl-alt-%); labels stay "Ctrl+Alt+Shift+N" (physical position). A
regression guard pins the companion forms in
`shell_surfaces_are_registered_in_the_ui_seams`. (First attempt fb512f3
carried the shift component — kept in history as the documented miss.)

### F1 regression + UX-003 probe scenes (all at 9977302)

- **d17 palette/overlay close-focus**: base-restore sequence holds
  (B02==B04==B06==B08==B10); probes 'x'/'y' land in the composer after
  every close. F1 focus contract unchanged.
- **d18 visible-entries**: palette opens via the titlebar entry;
  Suggested/Settings rows render.
- **d19-renav**: palette navigation reaches the browser-settings page
  (F02≠F01, settles byte-stable). NOTE: this scene had silently captured
  blank frames since its binary was hardcoded to a deleted WO-019 build —
  fixed to take the binary argument; the F1-era reference frames were the
  same blank-frame artifact, so the comparison baseline is the scene
  semantics, which pass.
- **d21 PTY focus (N5)**: the full PTY round-trip echoes ("echo
  focus-transfer-ok" → output → new prompt) in the bottom crop. The N5
  closure holds.
- **d22 swap chords (N6)**: bracket-chord traversal correct (Ctrl+Shift+]
  from alpha wraps to seed-ws per the pinned model). The N6 closure
  holds — and the same crop shows the new Workspace section coexisting
  in the sidebar.
- **rg-n1 keyboard chat + promo**: A-leg negative control byte-stable
  (A0==A1==A2); Ctrl+N focuses the composer ("n1 keyboard chat one"
  typed + chat created); palette New-chat row path green; promo captured
  and dismissed (one-Escape contract). The N1 closure holds.

## Gate C — house gates at the final SHA: GREEN

- `cargo fmt --all --check`: clean, local, at 9977302.
- `cargo clippy --workspace --all-targets -- -D warnings`: green via CI on
  BOTH platforms at the exact SHA (CI gates clippy; the f369001 lineage).
- `cargo test --workspace` local: **830 passed / 0 failed** (guarded
  release profile — the dev-env GUI linking is identical to the gate
  binary builds; the default release profile OOM-kills rustc's gpui
  units=1+LTO compile on this 4GB box; CI runs the debug-profile matrix).
  Includes the 250-test codex-app suite (147 F1 ui.rs tests + 6 shell
  tests + the chord guard), 60 flauz-exec, 47+31 flauz-world, 44
  flauz-context.
- CI: ubuntu-24.04 ✓ windows-latest ✓ at 9977302.

## Follow-up notes (non-blocking, carried to Wave 2+)

1. The unfiltered palette inventory does not surface the WorkspaceShell
   rows (query-only discovery) — acceptable (search is the palette's
   function) but a "Workspace" section in the default palette would
   improve cold-start discovery.
2. Palette Return from a shell-surface focus context did not execute the
   "Task evidence" row in the d23 run (row + hint + chord paths fully
   evidence the section) — investigate the palette row-selection focus
   interaction.
3. The Flauz chords are not yet listed in the keyboard-shortcuts overlay
   (the "?" help), and the workspace-surface chords have no visible hint
   outside the palette rows — add both.

## Wave-2 handoff

The F2 contracts (flauz-world, flauz-exec, flauz-context) + the shell are
merged and gated. Wave 2 per the roadmap: ENV-001 (environment fabric),
MOD-001/RT-001 (model + runtime fabric), CAP-001 (capability resolver) —
work orders to be authored in the wave ledger and dispatched from the
agents tab.
