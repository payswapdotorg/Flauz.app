# Feature Parity Work Orders

> **Status: FRAMEWORK + SEEDED DRAFTS.** The three entries in §4 are
> **DRAFT — pending audit confirmation by Worker C2** (reconciliation against
> Worker A's `CODEX-REFERENCE-MATRIX.md` and Worker B's
> `FLAUZ-REFERENCE-MATRIX.md`). No DRAFT entry may enter implementation
> before C2 confirms it.

**Repository:** `payswapdotorg/Flauz.app`
**Feeds from:** `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` (§7.5
confirmed-gap → work-order flow)
**Gap taxonomy / status / provenance / priorities:** defined in the parity
report §4 (operator-specified; do not reword).

---

## 1. Rules

1. **Bounded work orders only.** A work order is a single capability change
   on a single platform slice. No large unrelated refactors, no drive-by
   modernization, no cross-cutting rewrites. If a change cannot be described
   in the Required-change field in a few sentences, split it.
2. **Pack/Workflow freeze.** No work order may modify Pack or Workflow code
   or contracts (operator directive; Pack/Workflow development is frozen for
   the parity phase).
3. **A gap is closed ONLY when all five conditions hold:**
   1. the **source changed** (the implementing commit exists);
   2. **unit/integration tests pass** (focused coverage for the changed
      behavior);
   3. **GUI behavior is verified** (the visible behavior was observed, not
      assumed);
   4. **the relevant platform lab passes** (LINUX_GUI_LAB scene for Linux
      rows; Windows/macOS rows stay open until those labs exist — see rule 5);
   5. **the matching parity row is updated** in
      `CODEX-FLAUZ-FEATURE-PARITY-REPORT.md`.
4. **Source-code presence is NEVER "feature complete".** An implemented
   backend or an unreachable UI surface is not a closed gap; discoverability
   and GUI verification are first-class closure conditions.
5. **Platform honesty.** A work order names its platform slice(s) in the
   Platform field. Linux runtime evidence never closes a Windows row and
   vice versa. Windows/macOS GUI verification is deferred until
   WINDOWS_GUI_LAB / MACOS_GUI_LAB exist (see
   `docs/research/E2B-PARITY-ENVIRONMENT.md`, branch `parity/lab`); such rows
   close only on the evidence classes available to them and stay labeled.
6. **Reference-first.** Every non-platform work order cites the official
   reference behavior with a provenance label. A change without a reference
   behavior is not a parity work order.
7. **Intentional differences are documented, not "fixed".** Approved
   divergences (e.g., unsigned portable releases, native GPUI instead of
   Electron) close as `intentional difference` rows with the approval
   recorded — no work order is issued to "repair" them.

## 2. Work-order lifecycle

```text
DRAFT (seeded, pre-confirmation)
  → CONFIRMED (C2 reconciliation confirms the gap + reference + priority)
  → IN PROGRESS (assigned; bounded change underway)
  → VERIFIED (all five closure conditions evidenced)
  → CLOSED (parity row updated; evidence archived)
WITHDRAWN (DRAFT found unsupported by C2 — kept for audit, never deleted)
```

**ID scheme:** `WO-P{n}-###` for priority-class work orders (P0–P3),
`WO-PLAT-###` for platform-gap work orders. DRAFT entries carry the
`-DRAFT-` infix until C2 confirms; the confirmed entry keeps the same number
(WO-P1-DRAFT-001 → WO-P1-001).

## 3. Template (exact fields — every work order uses exactly these, in order)

```markdown
### WO-<ID>

- **ID:** WO-<ID>
- **Title:** <one line>
- **Platform:** <win | linux | macos | all — the platform slice(s) this order closes>
- **Reference behavior:** <official Codex behavior, with provenance label
  [runtime-observed] / [source-derived] / [docs-derived] / [historical-record]>
- **Current behavior:** <Flauz behavior today, with provenance label and
  source/runtime citation>
- **Gap type:** <Backend gap | UI-integration gap | Platform gap | UX gap |
  Intentional difference>
- **Required change:** <bounded description of the change; what is OUT of
  scope>
- **Files-crates:** <crates/files expected to change — best-known at DRAFT,
  refined at CONFIRMED>
- **Dependencies:** <other work orders, Worker A/B evidence, public
  protocols, platform labs>
- **Tests:** <unit/integration tests that must exist and pass>
- **GUI verification:** <how the GUI behavior is verified — LINUX_GUI_LAB
  scene, VLM-read screenshots, interaction evidence>
- **Acceptance criteria:** <checkable conditions; must map to the five
  closure conditions>
- **Known limitations:** <what remains open after this order closes>
```

## 4. Seeded entries

### WO-P1-DRAFT-001

- **ID:** WO-P1-DRAFT-001
- **Title:** Terminal discoverability — no persistent primary affordance
- **Platform:** all (UI-shell concern; Linux-verifiable in lab, Windows row closes per rule 5)
- **Reference behavior:** Official Codex exposes a persistent Terminal
  affordance in the primary UI (a user can find and open Terminal without
  knowing a shortcut). **[historical-record — pending Worker A confirmation;
  the official affordance's exact shape must come from the A matrix]**
- **Current behavior:** Flauz's Terminal is reachable only through:
  the `Ctrl+\`` keybinding (`crates/codex-app/src/ui.rs:4499`,
  `KeyBinding::new("ctrl-`", ToggleTerminalShortcut, None)`), the command
  palette, an "Open Terminal" menu item, and the dock toggle action
  (`Action::ToggleTerminalDock`, `crates/codex-core/src/lib.rs:4923`). No
  persistent primary affordance exists in the main window chrome.
  **[source-derived]** Terminal functionality itself is implemented and
  release-green **[historical-record: PM "Terminal" row, ledger green]** —
  the suspected gap is discoverability only.
- **Gap type:** UI-integration gap (suspected — capability exists but cannot
  be naturally discovered)
- **Required change:** Add a persistent primary Terminal affordance matching
  the official reference (exact affordance shape TBD from Worker A
  evidence). OUT of scope: terminal session semantics, PTY/ConPTY backend,
  dock behavior beyond mounting the affordance.
- **Files-crates:** `crates/codex-app/src/ui.rs` (window chrome / affordance
  placement), `crates/codex-core/src/lib.rs` (Action registry if a new action
  is needed)
- **Dependencies:** Worker A matrix (official Terminal affordance evidence);
  C2 confirmation of the gap; parity report §5.3 Terminal row.
- **Tests:** unit tests for action wiring; keymap/registry regression tests;
  UI-state test asserting the affordance renders in the default layout.
- **GUI verification:** LINUX_GUI_LAB scene: default window state captured
  (Xvfb + picom + ffmpeg x11grab, VLM-read) showing the Terminal affordance
  visible with no shortcut knowledge; click-path opens the terminal dock.
- **Acceptance criteria:** (1) implementing commit merged; (2) tests pass;
  (3) lab capture evidences the affordance in the default chrome; (4)
  LINUX_GUI_LAB scene passes; (5) parity report §5.3 Terminal row
  Discoverability cell updated and the Gap cell references WO-P1-001.
- **Known limitations:** Windows/macOS GUI verification deferred until those
  labs exist; official affordance shape is provisional until Worker A
  delivers.

### WO-P1-DRAFT-002

- **ID:** WO-P1-DRAFT-002
- **Title:** Browser discoverability — Browser hidden in the inspector pane set
- **Platform:** all (UI-shell concern; Linux-verifiable in lab)
- **Reference behavior:** Official Codex exposes the in-app Browser as a
  normally discoverable surface (a user can find and open it without palette
  or contextual knowledge). **[historical-record — pending Worker A
  confirmation; exact official affordance TBD]**
- **Current behavior:** Flauz's Browser is one variant of the inspector pane
  enum (`InspectorPane::Browser`, `crates/codex-core/src/lib.rs:259–267`,
  alongside Hidden/Changes/Outputs/Files/Terminal/ComputerUse), reachable
  via the palette and contextual paths only; no persistent primary
  affordance. **[source-derived]** Browser functionality itself is
  implemented and release-green **[historical-record: PM "In-app browser"
  row, ledger green]** — the suspected gap is discoverability only.
- **Gap type:** UI-integration gap (suspected)
- **Required change:** Surface a persistent, naturally discoverable Browser
  entry matching the official reference (shape TBD from Worker A evidence).
  OUT of scope: browser runtime, permission cards, JPEG streaming, profile
  supervision.
- **Files-crates:** `crates/codex-app/src/ui.rs`, `crates/codex-core/src/lib.rs`
  (`InspectorPane` selection paths / Action registry)
- **Dependencies:** Worker A matrix (official Browser affordance evidence);
  C2 confirmation; parity report §5.4 Browser row; coordinate with
  WO-P1-DRAFT-001 (shared chrome surface).
- **Tests:** unit tests for pane-selection wiring; UI-state test asserting
  the affordance renders and selects `InspectorPane::Browser`.
- **GUI verification:** LINUX_GUI_LAB scene: default window state capture
  (VLM-read) showing the Browser affordance; click-path opens the browser
  pane.
- **Acceptance criteria:** (1) implementing commit merged; (2) tests pass;
  (3) lab capture evidences discoverability without palette knowledge;
  (4) LINUX_GUI_LAB scene passes; (5) parity report §5.4 In-app browser row
  updated with WO-P1-002.
- **Known limitations:** Windows/macOS GUI verification deferred; official
  affordance shape provisional until Worker A delivers.

### WO-PLAT-DRAFT-001

- **ID:** WO-PLAT-DRAFT-001
- **Title:** Linux Computer Use platform bound — document as platform gap, not missing feature
- **Platform:** linux
- **Reference behavior:** The official Any App computer-use surface
  (case-preserved AUMIDs / known-folder GUIDs / absolute executable paths,
  first-read consent, managed approvals) is defined for the official
  platform matrix (Windows). **[historical-record]**
- **Current behavior:** On Linux, Flauz Computer Use is limited to bounded,
  screenshot-only X11/XWayland observation when `DISPLAY` is set. Pure
  Wayland, text extraction, input, app launch, persistent approvals,
  overlays, and interruption monitoring are unavailable; a portal-backed
  selection path remains future work. **[historical-record:
  `docs/known-failures.md` "Active release-candidate limitations"; runtime
  behavior re-checkable in LINUX_GUI_LAB]**
- **Gap type:** Platform gap — **documented bound, not a missing feature.**
  This work order exists to keep the parity row honest
  (`platform-limited`), not to implement the missing Linux modalities.
- **Required change:** Documentation/labeling only: keep the bound recorded
  in the parity report §5.5 (Computer Use — Linux row: Backend/UI/Functional
  parity = `platform-limited`, Gap = platform gap), keep
  `docs/known-failures.md` as the canonical limitation text, and track the
  portal-backed selection path as future work. OUT of scope: any
  implementation of Wayland/AT-SPI/portal input during the parity phase.
- **Files-crates:** `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md`
  (row label); `docs/known-failures.md` (already documents the bound —
  verify wording stays accurate). No product code.
- **Dependencies:** none (documentation); C2 confirms the labeling during
  reconciliation.
- **Tests:** not applicable (no source change). Standing regression tests
  for the screenshot-only bound remain as-is.
- **GUI verification:** LINUX_GUI_LAB: confirm the screenshot-only behavior
  is what the row claims (bounded X11/XWayland observation) — no expectation
  of input/Wayland support.
- **Acceptance criteria:** parity row reads `platform-limited` with the
  Linux bound named and the future-work pointer present; no code change;
  documentation consistent across parity report and known-failures.
- **Known limitations:** The bound itself (no pure Wayland, no input, no
  persistent approvals) remains until dedicated platform work is authorized.

## 5. Backlog hygiene

- New work orders are created **only** through C2's confirmed-gap flow
  (parity report §7.5) or by operator directive; every entry must carry the
  full template.
- A WITHDRAWN entry keeps its record (audit trail) with one line stating why.
- Priority class is part of the gap confirmation, not negotiable at
  implementation time; reclassification requires C2 or the operator.
- Related material: `docs/codex-universal/WORK-ORDERS.md` (the completed
  GUI-001..007 program) is historical context — it is NOT this system and
  its IDs do not collide (`GUI-###` vs `WO-…`).
