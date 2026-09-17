# Feature Parity Work Orders

> **Status: ACTIVE — implementation wave 1 landed: WO-P1-001 + WO-P1-002
> CLOSED on main (merge `d15333e`, 2026-09-17).** Operator directive
> (2026-09-17): execute WO-LAB-001 (official Linux preview app in
> LINUX_GUI_LAB) and reconcile current-26.825 behavior BEFORE WO-P1-003
> implementation; then WO-P1-003 (multi-folder local projects), then the
> remaining P1/P2 backlog; Windows/macOS GUI labs follow. The three
> C1-seeded DRAFTs were
> confirmed by Worker C2 against Worker A's `CODEX-REFERENCE-MATRIX.md`
> (`c083c38`) and Worker B2's `FLAUZ-REFERENCE-MATRIX.md` (`a2343d3`); five
> further work orders were created from gaps the A/B audit surfaced. Every
> entry below is CONFIRMED with full evidence per the parity report §7.5
> confirmed-gap flow. No entry may enter IN PROGRESS before the Tech Lead
> converges the wave.

**Repository:** `payswapdotorg/Flauz.app`
**Feeds from:** `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` (§7.5
confirmed-gap → work-order flow; §8.2 priority lists)
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
`WO-PLAT-###` for platform-gap work orders, `WO-LAB-###` for
lab-infrastructure work orders (class added by C2 — see WO-LAB-001; lab
orders are enabling work, not feature gaps, and carry no P-class). DRAFT
entries carried the `-DRAFT-` infix until C2 confirmed; the confirmed entry
keeps the same number (WO-P1-DRAFT-001 → WO-P1-001).

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

## 4. Confirmed entries (C2 reconciliation, 2026-09-16)

### WO-P1-001

- **ID:** WO-P1-001 (confirmed from WO-P1-DRAFT-001)
- **Title:** Terminal discoverability — status: **CLOSED (merged 2026-09-17)** [implemented on parity/wo-p1-discoverability (08d060c; clippy follow-up aa2d89d); merged to main via PR #13 — merge commit d15333e718a7649238f33a183af27c885dbaa8aa; CI green on windows-latest + ubuntu-24.04 (dependency policy, fmt, clippy -D warnings, full workspace tests, release build, Linux Xvfb startup smoke); unit tests in-tree (default-layout affordance rendering; entry-surface toggles surface guidance, never silent no-op); runtime GUI evidence docs/research/evidence/wo-p1/ (VLM-read affordances in default layout; guard + Dismiss surfaced on entry surface; frames differ); all five closure gates satisfied 2026-09-17]
- **Platform:** all (UI-shell concern; Linux-verifiable in lab, Windows row closes per rule 5)
- **Reference behavior:** Official Codex exposes Terminal as a **persistent
  per-chat dock** (bottom or right) that a user can find and open without
  shortcut knowledge; `Toggle bottom panel` Ctrl/Cmd+J is a distinct action
  from opening the terminal; hiding preserves live sessions. **[historical-record:
  A §3 "Persistent affordance" row; PM "Terminal" row]** Current official docs
  additionally bind `Toggle terminal` to Ctrl/Cmd+`` ` `` with `Clear terminal`
  Ctrl+L/Ctrl+K when focused (post-baseline; introduction date
  `[unverified]`). **[docs-derived: A §3; commands doc]**
- **Current behavior:** Flauz's Terminal is implemented and release-green
  in-chat (bounded per-chat PTY/ConPTY tabs, new-tab/selection/close/stop/
  restart) **[historical-record: PM "Terminal", ledger green]** but is
  reachable only via: the `Ctrl+\`` keybinding
  (`crates/codex-app/src/ui.rs:4499`, `KeyBinding::new("ctrl-`",
  ToggleTerminalShortcut, None)`), the palette entry "Open terminal" (Ctrl+`)
  (`ui.rs:3322/3411`), and the dock toggle action (`Action::ToggleTerminalDock`,
  `crates/codex-core/src/lib.rs:4923`) — **no persistent primary affordance
  exists anywhere in the window chrome** **[source-derived]**. B2's runtime
  proof strengthens this to "undiscoverable AND inert" on the entry surface:
  **Ctrl+` on the new-chat (draft) surface is a silent no-op — the
  post-toggle capture is byte-identical (md5) to the pre-toggle chat view;
  the dock opens with zero tabs and renders nothing; the guard is "Select a
  task before opening a terminal." (`crates/codex-core/src/lib.rs:8163`)**
  **[runtime-observed: B2 §3 + Finding 1, ev/25 vs ev/19; anchors re-verified
  by C2 at `f113515`]**
- **Gap type:** UI-integration gap (capability exists, effectively undiscoverable)
- **Required change:** Add a persistent primary Terminal affordance in the
  main window chrome matching the official persistent-dock model, and make
  the entry-surface toggle honest (either open a usable terminal surface or
  show the guard message instead of a silent no-op — today the guard writes
  `status_message` but produces no visible change on the entry surface).
  OUT of scope: terminal session semantics, PTY/ConPTY backend, dock
  behavior beyond mounting the affordance, the Ctrl+` binding itself (it
  already matches the current official binding).
- **Files-crates:** `crates/codex-app/src/ui.rs` (window chrome / affordance
  placement, status-message surfacing), `crates/codex-core/src/lib.rs`
  (Action registry if a new action is needed)
- **Dependencies:** parity report §5.3 Terminal row; A §3 evidence; B2
  Finding 1 evidence; coordinate with WO-P1-002 (shared chrome surface).
- **Tests:** unit tests for action wiring; keymap/registry regression tests;
  UI-state test asserting the affordance renders in the default layout and
  that the entry-surface toggle produces a visible result (affordance or
  message), never a silent no-op.
- **GUI verification:** LINUX_GUI_LAB scene: default window state captured
  (Xvfb + picom + ffmpeg x11grab, VLM-read) showing the Terminal affordance
  visible with no shortcut knowledge; click-path opens the terminal dock;
  repeat the B2 b2-gaps.sh no-op probe on the new-chat surface and show the
  frame now differs (affordance visible / guard surfaced).
- **Acceptance criteria:** (1) implementing commit merged; (2) tests pass;
  (3) lab capture evidences the affordance in the default chrome and the
  entry-surface toggle no longer silently no-ops; (4) LINUX_GUI_LAB scene
  passes; (5) parity report §5.3 Terminal row Discoverability cell updated
  and the Gap cell references WO-P1-001.
- **Known limitations:** Windows/macOS GUI verification deferred until those
  labs exist; the exact official affordance shape (chrome placement) is
  `[historical-record]`-derived — the official Linux preview app, if run per
  WO-LAB-001, can upgrade the reference to `[runtime-observed]`.

### WO-P1-002

- **ID:** WO-P1-002 (confirmed from WO-P1-DRAFT-002)
- **Title:** Browser discoverability — status: **CLOSED (merged 2026-09-17)** [implemented on parity/wo-p1-discoverability (08d060c; clippy follow-up aa2d89d); merged to main via PR #13 — merge commit d15333e718a7649238f33a183af27c885dbaa8aa; CI green on windows-latest + ubuntu-24.04 (dependency policy, fmt, clippy -D warnings, full workspace tests, release build, Linux Xvfb startup smoke); unit tests in-tree (default-layout affordance rendering; entry-surface toggles surface guidance, never silent no-op); runtime GUI evidence docs/research/evidence/wo-p1/ (VLM-read affordances in default layout; guard + Dismiss surfaced on entry surface; frames differ); all five closure gates satisfied 2026-09-17]
- **Platform:** all (UI-shell concern; Linux-verifiable in lab)
- **Reference behavior:** Official Codex exposes the in-app Browser as a
  normally discoverable surface: a native browser panel with bounded tabs,
  URL/navigation state, JPEG frame streaming, and keyboard/pointer/scroll
  input, opened via `Open browser tab` Ctrl/Cmd+T and `Toggle browser panel`
  Ctrl/Cmd+Shift+B (baseline-verified bindings), with address-bar
  back/forward/reload/copy-URL shortcuts. **[historical-record: A §4
  "Persistent affordance" row; PM "In-app browser" row]**
- **Current behavior:** Flauz's Browser is implemented (native GPUI panel:
  `BrowserSession::spawn`/navigate/back/forward/reload/stop, tabs, downloads
  store) **[source-derived: `crates/codex-platform/src/browser.rs`; PM ledger
  green]** but lives as one variant of the inspector pane enum
  (`InspectorPane::Browser`, `crates/codex-core/src/lib.rs:259-267`, alongside
  Hidden/Changes/Outputs/Files/Terminal/ComputerUse) reachable via palette
  and contextual paths only — no persistent primary affordance
  **[source-derived]**. B2's runtime proof: **Ctrl+T / Ctrl+Shift+B on the
  new-chat entry surface are silent no-ops; `OpenBrowserTab` is refused
  without an open chat — the guard is "Open a chat before opening the
  Browser." (`crates/codex-core/src/lib.rs:14521-14525`)** **[runtime-observed:
  B2 §4 + Finding 2, b2g-06; anchor re-verified by C2 at `f113515`]**
- **Gap type:** UI-integration gap (capability exists, effectively undiscoverable)
- **Required change:** Surface a persistent, naturally discoverable Browser
  entry matching the official reference, and make the entry-surface refusal
  honest (visible guidance instead of a silent no-op). OUT of scope: browser
  runtime, permission cards, JPEG streaming, profile supervision, WebMCP
  (separate row, no work order yet).
- **Files-crates:** `crates/codex-app/src/ui.rs`,
  `crates/codex-core/src/lib.rs` (`InspectorPane` selection paths / Action
  registry, guard-message surfacing)
- **Dependencies:** parity report §5.4 In-app browser row; A §4 evidence;
  B2 Finding 2 evidence; coordinate with WO-P1-001 (shared chrome surface).
- **Tests:** unit tests for pane-selection wiring; UI-state test asserting
  the affordance renders and selects `InspectorPane::Browser`; regression
  test that the guard message is surfaced (not silent) when no chat exists.
- **GUI verification:** LINUX_GUI_LAB scene: default window state capture
  (VLM-read) showing the Browser affordance; click-path opens the browser
  pane; repeat the B2 no-op probe on the entry surface and show a visible
  result.
- **Acceptance criteria:** (1) implementing commit merged; (2) tests pass;
  (3) lab capture evidences discoverability without palette knowledge and
  the entry-surface toggle no longer silently no-ops; (4) LINUX_GUI_LAB
  scene passes; (5) parity report §5.4 In-app browser row updated with
  WO-P1-002.
- **Known limitations:** Windows/macOS GUI verification deferred; official
  affordance shape is `[historical-record]`-derived until WO-LAB-001
  upgrades it; the address-bar history/Google-fallback delta (26.727,
  P2) is explicitly NOT in this order's scope.

### WO-P1-003

- **ID:** WO-P1-003 (new — created by C2 from the audit)
- **Title:** Multi-folder local projects — in-baseline official capability wholly absent
- **Platform:** all (data-model + UI; Linux-verifiable in lab)
- **Reference behavior:** Official Codex supports **multi-folder local
  projects since 26.715 — in the historical baseline** (26.715 < 26.721):
  `Edit project` adds related folders and chooses the primary; new chats,
  Git operations, AGENTS.md/skills/config.toml discovery use the primary
  folder; secondary folders are used for file search/reading/editing.
  **[docs-derived: A §6 "Projects" row; changelog 2026-07-23]** The official
  64-entry local-project registry with selection/new chat/rename/pin/
  move/remove is `[historical-record]` (A §6, PM).
- **Current behavior:** Flauz's project model is single-path:
  `LocalProjectSummary` carries one `path` (64-entry registry,
  `recent_workspaces` table) — no related-folders or primary/secondary
  concept anywhere in the model. **[source-derived: `crates/codex-core/src/lib.rs:908,
  4719-4720`; C2 sweep at `f113515`; B2 §6]** Runtime: sidebar Projects
  section renders with honest "No projects" empty state; `recent_workspaces`
  table present. **[runtime-observed: ev/06; state-DB schema]** The
  historical ledger had carried "multi-root sources" as polish (P3) —
  reclassified P1 by the audit (parity report §9 override 3).
- **Gap type:** Backend gap (no data model, no UI)
- **Required change:** Extend the local-project model with related folders
  plus a designated primary; add an `Edit project` surface (add/remove
  related folders, choose primary); wire the primary folder into new-chat
  `cwd`, Git operations, and AGENTS.md/skills/config.toml discovery; wire
  secondary folders into file search/reading/editing. OUT of scope:
  multi-repository review UI (26.727 delta — separate future order after
  this lands), cloud projects (deferred), project migration tooling beyond
  transparently treating existing single-path projects as primary-only.
- **Files-crates:** `crates/codex-core/src/lib.rs` (project model,
  normalization, discovery wiring), `crates/codex-app/src/ui.rs` (Edit
  project surface, pickers), `crates/codex-storage` (persistence)
- **Dependencies:** parity report §5.1 Projects and chats row; A §6
  evidence; B2 §6 evidence; follow-on: multi-repo review (P2, blocked-by
  this order).
- **Tests:** model unit tests (related-folder sets, primary invariants,
  legacy single-path migration); discovery tests (primary drives cwd/Git/
  config discovery; secondary drives file search); UI-state tests for the
  Edit project surface.
- **GUI verification:** LINUX_GUI_LAB scene: create a multi-folder project
  via Edit project (VLM-read captures), start a new chat carrying the
  primary `cwd`, run file search hitting a secondary folder.
- **Acceptance criteria:** (1) implementing commit merged; (2) tests pass;
  (3) lab capture evidences the multi-folder project flow end to end;
  (4) LINUX_GUI_LAB scene passes; (5) parity report §5.1 row updated
  (multi-folder gap closed; multi-repo review unblocked as its own item).
- **Known limitations:** Implementation may land as multiple PRs (model →
  UI → discovery wiring) but closes as one capability; multi-repo review
  and richer project metadata remain open.

### WO-P2-004

- **ID:** WO-P2-004 (new — created by C2 from the audit)
- **Title:** Command palette does not index all settings pages
- **Platform:** all (UI-shell concern; Linux-verifiable in lab)
- **Reference behavior:** Official command palette (Ctrl/Cmd+K,
  Ctrl/Cmd+Shift+P, Ctrl/Cmd+G) surfaces a dynamic `Settings` group so
  settings surfaces are reachable from the palette; official Settings is a
  searchable, grouped registry. **[historical-record: A §10 "Command
  palette" + "Settings shell" rows; PM "Keyboard and accessibility"]**
- **Current behavior:** Flauz's palette (`PaletteCommand::ALL`, 45 commands,
  `crates/codex-app/src/ui.rs:3195-3241+`) indexes only 10 settings-ish
  surfaces; **six nav-reachable settings pages have no palette entry:
  Profile, Import, Browser, Configuration, Hooks, Git** (SettingsSection
  enum `ui.rs:2398-2417`; nav renders 15 rows). **[source-derived: C2 sweep
  at `f113515`]** Runtime proof: palette query "import" → **"No matches"**
  (ev/18) while the Import page exists in the settings nav and is found by
  the in-Settings search filter (ev/23). **[runtime-observed: B2 §10; b5a
  session]**
- **Gap type:** UI-integration gap
- **Required change:** Index every nav-reachable settings section in the
  command palette (add the missing six entries — or render a dynamic
  Settings group mirroring the SettingsSection registry so future sections
  are indexed automatically). OUT of scope: the Settings pages themselves,
  settings search behavior (already works, ev/23), palette command
  semantics beyond navigation.
- **Files-crates:** `crates/codex-app/src/ui.rs` (`PaletteCommand` registry
  or the palette's Settings group construction)
- **Dependencies:** parity report §5.10 Keyboard and accessibility row;
  B2 §10 evidence (ev/18 vs ev/23).
- **Tests:** registry unit test asserting every default-nav SettingsSection
  has a palette entry (or that the dynamic group enumerates the registry);
  palette filtering tests.
- **GUI verification:** LINUX_GUI_LAB scene: palette query "import"
  navigates to the Import settings page (VLM-read), plus one capture per
  previously-missing page.
- **Acceptance criteria:** (1) implementing commit merged; (2) tests pass;
  (3) lab capture shows "import" resolving to the Import page; (4)
  LINUX_GUI_LAB scene passes; (5) parity report §5.10 row updated with
  WO-P2-004.
- **Known limitations:** Contextual/hidden sections (CodeReview, Worktrees,
  ArchivedChats) stay out of the default index unless the dynamic-group
  approach naturally includes them; remaining stable palette commands are a
  separate ledger item.

### WO-P2-005

- **ID:** WO-P2-005 (new — created by C2 from the audit)
- **Title:** Slash-command coverage delta vs the current 24-command official set
- **Platform:** all (composer surface; Linux-verifiable in lab)
- **Reference behavior:** Official baseline shipped 15 slash commands
  (`/chat /compact /feedback /fork /goal /init /mcp /memories /model /new
  /plan /project /reasoning /status /shell`) **[historical-record: A §2; PM
  "Composer" row]**; the **current target documents 24** (`/approve /cloud
  /cloud-environment /compact /fast /feedback /fork /goal /ide-context
  /init /local /mcp /memories /model /pet /personality /plan /project
  /reasoning /review /side /status /task /worktree`), with skills invoked
  via `$` and custom prompts via `/prompts:` **[docs-derived: A §2;
  slash-commands doc; per-version introduction dates `[unverified]`]**.
- **Current behavior:** Flauz implements the full baseline set of 15 **plus
  `/review`** (matching the current target) — 16 named commands — and
  dynamic `/service-tier:<id>` rows (including the exact Fast on/off copy)
  and `/skill:<path>` rows. **[source-derived: slash executor
  `ui.rs:7510-7621`, menu catalog `ui.rs:22892-23120` — C2 re-verification
  overrides B2's 15-item enumeration, parity report §9 override 1;
  runtime-observed menu ev/20]** Missing literal names vs the current 24:
  `/approve`, `/cloud`, `/cloud-environment`, `/fast`, `/ide-context`,
  `/local`, `/pet`, `/personality`, `/side`, `/task`, `/worktree` (11).
- **Gap type:** UX gap (coverage delta; most missing names map to
  capabilities that are deferred or absent)
- **Required change:** Add the missing literal slash commands **whose
  backing capabilities already exist**: `/fast` (alias onto the existing
  service-tier selection path), `/personality` (opens the existing
  Personalization settings), `/approve` (resolves the active pending
  approval through the existing approval action), `/worktree` (opens the
  existing worktree-fork picker), and `/side` once WO-P2-006 lands. OUT of
  scope: `/cloud`, `/cloud-environment`, `/pet`, `/task` (deferred
  capabilities — cloud environments, Pets, scheduled tasks); `/ide-context`,
  `/local` (no backing capability; semantics `[unverified]`); `$`-prefix
  skill invocation and `/prompts:` custom prompts (separate Skills-row
  items).
- **Files-crates:** `crates/codex-app/src/ui.rs` (slash executor + menu
  catalog)
- **Dependencies:** parity report §5.2 Composer row; A §2 evidence; B2 §2
  evidence; WO-P2-006 (for `/side`).
- **Tests:** slash-executor unit tests for each new command; menu-catalog
  rendering tests; guards (e.g. `/approve` with no pending request,
  `/worktree` without a local workspace).
- **GUI verification:** LINUX_GUI_LAB scene: type each new command in the
  composer, capture the menu entry and the resulting native action
  (VLM-read).
- **Acceptance criteria:** (1) implementing commit merged; (2) tests pass;
  (3) lab capture shows each in-scope command executing its native action;
  (4) LINUX_GUI_LAB scene passes; (5) parity report §5.2 Composer row Gap
  cell updated with WO-P2-005 (residual out-of-scope names listed).
- **Known limitations:** Seven of the eleven missing names stay open
  (deferred capabilities or unverified semantics) and remain enumerated on
  the Composer row; the current-docs list is `[docs-derived]` — exact
  official behavior of each command beyond its doc line is `[unverified]`.

### WO-P2-006

- **ID:** WO-P2-006 (new — created by C2 from the audit)
- **Title:** Side chats — in-baseline conversation mode absent
- **Platform:** all (thread-surface concern; Linux-verifiable in lab)
- **Reference behavior:** Official Codex: `Open side chat`
  Ctrl/Cmd+Alt+S opens a temporary side conversation without interrupting
  the main chat; `/side` appears in the current command set. **[historical-record
  + docs-derived: A §1 "Side chats" row — confidence medium-high (26.707
  "side conversations" note + current docs command list)]**
- **Current behavior:** No side-chat surface: Ctrl+Alt+S is absent from the
  key registry (`ui.rs:4459-4552`) and `/side` from the slash executor
  (`ui.rs:7510-7621`). **[source-derived: C2 sweep at `f113515`; B2
  shortcut inventory]**
- **Gap type:** Backend gap (no side-conversation surface)
- **Required change:** Add the side-chat surface: a Ctrl+Alt+S binding and
  `/side` command that open a temporary side conversation leaving the
  selected chat untouched, reusing existing thread/projectless-chat
  machinery; the side chat is closable/dismissible without affecting the
  main chat. OUT of scope: changes to thread execution, the main-chat
  switcher, browser-extension side-chat features.
- **Files-crates:** `crates/codex-app/src/ui.rs` (binding, command, side
  conversation surface), `crates/codex-core/src/lib.rs` (action/state if a
  dedicated side-chat state is needed)
- **Dependencies:** parity report §5.1 Side chats row; A §1 evidence
  (medium-high confidence — flag to Tech Lead); feeds `/side` into
  WO-P2-005.
- **Tests:** binding/registry tests; executor test for `/side`; state test
  asserting the main chat's selection and active turn are untouched.
- **GUI verification:** LINUX_GUI_LAB scene: with a chat selected (fixture
  runtime), open a side chat via Ctrl+Alt+S and via `/side`; capture that
  the main chat remains selected/intact (VLM-read).
- **Acceptance criteria:** (1) implementing commit merged; (2) tests pass;
  (3) lab capture evidences both entry paths with the main chat intact;
  (4) LINUX_GUI_LAB scene passes; (5) parity report §5.1 Side chats row
  updated with WO-P2-006.
- **Known limitations:** Baseline evidence is medium-high confidence (not
  runtime-observed on the official side); authenticated lab runs will be
  needed to compare side-chat semantics precisely; extension side-chat
  surfaces are out of scope.

### WO-PLAT-001

- **ID:** WO-PLAT-001 (confirmed from WO-PLAT-DRAFT-001)
- **Title:** Linux Computer Use platform bound — document as platform gap, not missing feature
- **Platform:** linux
- **Reference behavior:** The official Any App computer-use surface
  (case-preserved AUMIDs / known-folder GUIDs / absolute executable paths,
  first-read consent, managed approvals, Window2 action surface, overlay,
  URL policy, Escape interruption) is defined for the official platform
  matrix (Windows; macOS mechanics partly `[unverified]`). **[historical-record:
  A §5]** The official Linux preview app's Computer Use scope is
  `[unverified]` **[docs-derived: A platform table]**.
- **Current behavior:** On Linux, Flauz Computer Use is limited to bounded,
  screenshot-only X11/XWayland observation when `DISPLAY` is set; pure
  Wayland, text extraction, input, app launch, persistent approvals,
  overlays, and interruption monitoring are unavailable; a portal-backed
  selection path remains future work. The platform gate is explicit in
  source: "Linux observation is intentionally limited to X11/XWayland…
  Pure Wayland requires the separate portal path"
  (`crates/codex-platform/src/computer_use.rs:34-57`), and the in-app
  settings copy is platform-honest ("Let ChatGPT observe screenshots of
  X11/Wayland apps — Enabled", ev/14). **[source-derived +
  runtime-observed: B2 §5 + Finding 3; KF "Active release-candidate
  limitations"]**
- **Gap type:** Platform gap — **documented bound, not a missing feature.**
  This work order exists to keep the parity row honest
  (`platform-limited`), not to implement the missing Linux modalities.
- **Required change:** Documentation/labeling only: keep the bound recorded
  in the parity report §5.5 (Computer Use — Linux row: Backend/UI/Functional
  parity = `platform-limited`, Gap = platform gap → WO-PLAT-001), keep
  `docs/known-failures.md` as the canonical limitation text, and track the
  portal-backed selection path as future work. OUT of scope: any
  implementation of Wayland/AT-SPI/portal input during the parity phase.
- **Files-crates:** `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md`
  (row label — done at C2 reconciliation); `docs/known-failures.md`
  (already documents the bound — verify wording stays accurate). No
  product code.
- **Dependencies:** none (documentation); C2 confirmed the labeling during
  reconciliation (this entry).
- **Tests:** not applicable (no source change). Standing regression tests
  for the screenshot-only bound remain as-is.
- **GUI verification:** LINUX_GUI_LAB: confirm the screenshot-only behavior
  is what the row claims (bounded X11/XWayland observation) — B2 ev/14
  already evidences the honest settings copy; no expectation of
  input/Wayland support.
- **Acceptance criteria:** parity row reads `platform-limited` with the
  Linux bound named and the future-work pointer present (done at
  reconciliation); no code change; documentation consistent across parity
  report and known-failures.
- **Known limitations:** The bound itself (no pure Wayland, no input, no
  persistent approvals) remains until dedicated platform work is authorized;
  if WO-LAB-001 runs the official Linux app, its Computer Use scope gets
  re-evidenced `[unverified]` → runtime-observed or confirmed-absent.

### WO-LAB-001

- **ID:** WO-LAB-001 (new — lab-infrastructure class added by C2; no P-class)
- **Title:** Run the official Linux desktop preview app in LINUX_GUI_LAB to upgrade Linux-A evidence to runtime-observed
- **Platform:** linux (lab infrastructure — affects the official-side evidence layer, not Flauz product code)
- **Reference behavior:** An OFFICIAL Linux desktop app exists in preview
  since 2026-08-11: `.deb`/`.rpm`/install script; Ubuntu 24.04/26.04,
  Debian 13, Fedora 43/44 (+Arch per current docs); x64 + ARM64; installs
  from `persistent.oaistatic.com/codex-app-prod/linux/...`. **[docs-derived:
  A platform table; changelog 2026-08-11; `docs/linux/linux-app.md`]**
- **Current behavior:** LINUX_GUI_LAB's official side is evidence-layers
  only (`[historical-record]`/`[docs-derived]`/`[source-derived]`): the
  official GUI is a closed-source Windows/macOS Electron app at baseline
  and has never been run in this lab. The local sandbox is Debian 13 —
  inside the official app's supported matrix. **[historical-record +
  docs-derived: E2B-PARITY-ENVIRONMENT.md (parity/lab)]**
- **Gap type:** Platform gap (evidence-class gap — lab capability, not a
  product feature)
- **Required change:** Attempt to run the official Linux preview app inside
  the local lab: fetch the `.deb`, extract userspace (established
  `fetch_debs.py` pattern), run under Xvfb + picom with capture + VLM
  verification. If it runs: capture baseline surfaces (entry, sidebar,
  settings, terminal/browser affordance shapes), upgrade the affected
  Linux-A provenance labels to `[runtime-observed]`, and hand the
  E2B-PARITY-ENVIRONMENT bounds update to the Tech Lead (platform bounds
  are NEVER changed silently). If it does not run: record the exact bound
  honestly. OUT of scope: any Flauz product change; any authenticated
  official-app usage (no credentials in lab); changing platform bounds
  without the Tech Lead.
- **Files-crates:** none in product code; lab assets under
  `/home/z/parity-lab/` (tools + scenes) and evidence captures under
  `docs/research/evidence/` on a research branch.
- **Dependencies:** disk headroom (~600 MB free at C2 time — clear caches
  or extend storage first); preview-app stability; unauthenticated bound
  stands (account-powered official surfaces stay evidence-layered).
- **Tests:** not applicable (lab infrastructure); success criterion is a
  reproducible scene script.
- **GUI verification:** the deliverable IS GUI verification: VLM-read
  captures of the official app's Linux surfaces; provenance-label upgrades
  recorded in the parity report with an override-audit-trail entry.
- **Acceptance criteria:** (1) reproducible run (or documented failure with
  root cause) recorded; (2) if successful: baseline captures archived +
  affected parity rows re-labeled + bounds-update proposal delivered to the
  Tech Lead; (3) no silent change to E2B-PARITY-ENVIRONMENT.md or §3 of the
  parity report without Tech Lead convergence; (4) `[unverified]` items it
  resolves (Linux Computer Use scope, Activity view, terminal/browser
  affordance shapes) enumerated and updated.
- **Known limitations:** Unauthenticated (account surfaces stay bound);
  preview-quality app (behavior may differ from 26.825.51511 stable);
  evidence upgrades apply to the Linux slice only — Windows/macOS cells
  stay `[historical-record]`/`[docs-derived]`.

## 5. Gaps recorded in the parity report WITHOUT work orders (audit decisions)

Per §7.5 rule 4 and work-order rule 6, the following reconciled gaps carry
**no work order** — recorded here so the Tech Lead sees the scope decisions:

- **In-app Markdown/code editing (P2, in-baseline 26.707)** — editor surface
  too large for one bounded order; needs Tech Lead scoping (surface set,
  annotation model) before an order can be written.
- **Activity view & unread attention (P2, 26.727)** — depends on an unread
  state that does not exist; scope after WO-P1-003/WO-P2-006 land or by
  directive.
- **WebMCP site tools (P2, 26.825)** — blocked on fork-runtime WebMCP
  capability; work order follows when the runtime track delivers it.
- **Record & Replay (P2, 26.727-era)** — Computer-Use-dependent; Linux slice
  platform-bound (WO-PLAT-001 context).
- **Browser extension beyond Chrome (P2, 26.825)** — adjacent deliverable
  (browser-store artifact), out of the parity wave's app-binary scope.
- **Plugins marketplace catalog-empty vs 64-plugin disk sync** — BOUND, not
  a confirmed defect: unauthenticated lab cannot distinguish an auth-bound
  catalog listing from an integration gap, and unauthenticated official
  reference behavior is `[unverified]` (auth-walled). Re-verification path:
  authenticated J6 re-run. No reference behavior → no parity work order
  (rule 6).
- **Ledger-enhancement residuals (P2 set: permission-profile granular
  editor, artifacts renderers/canvas, dynamic CU tools, billing entry
  points, first-run welcome/diagnostics, remaining stable palette commands/
  focus order/screen-reader labels, tray groups/badges/sounds, Skills
  recommended/install flows + `$`)** — real gaps, each awaiting a bounded
  scoping pass; the parity report rows carry them.
- **Deferred/proprietary set** (scheduled tasks + event triggers, voice,
  sites, visualizations, cloud environments, appshots, pets, connector
  approval methods, plugins/App OAuth callbacks, import unsupported-project
  reporting, SSH profiles/remote chats, settings host contracts, Computer
  History, unified pins/shared snapshots) — no work orders by definition;
  blockers named on the rows.

## 6. Backlog hygiene

- New work orders are created **only** through C2's confirmed-gap flow
  (parity report §7.5) or by operator directive; every entry must carry the
  full template.
- A WITHDRAWN entry keeps its record (audit trail) with one line stating why.
  **No withdrawals at C2 reconciliation:** all three C1-seeded DRAFTs
  (WO-P1-DRAFT-001, WO-P1-DRAFT-002, WO-PLAT-DRAFT-001) were CONFIRMED.
- Priority class is part of the gap confirmation, not negotiable at
  implementation time; reclassification requires C2 or the operator.
- Related material: `docs/codex-universal/WORK-ORDERS.md` (the completed
  GUI-001..007 program) is historical context — it is NOT this system and
  its IDs do not collide (`GUI-###` vs `WO-…`).

## 7. Change log

- 2026-09-16: WO-P1-001 and WO-P1-002 set to VERIFIED (pending merge) — implemented on parity/wo-p1-discoverability (sidebar footer affordances + honest entry-surface guards + regression tests + runtime evidence under docs/research/evidence/wo-p1/).
- 2026-09-17: **WO-P1-001 + WO-P1-002 CLOSED** — merged to main via PR #13 (merge commit d15333e718a7649238f33a183af27c885dbaa8aa; implementation 08d060c, clippy fix aa2d89d); CI green on both matrices (windows-latest, ubuntu-24.04); runtime GUI evidence archived under docs/research/evidence/wo-p1/; parity report §5.3/§5.4 rows + §8.2 P1 list + J1/J2 updated to reference the merge commit. Operator review 2026-09-17 accepted the implementation wave and directed: WO-LAB-001 (official Linux preview app runtime comparison + current-26.825 reconciliation) executes BEFORE WO-P1-003 implementation.

| Date | Change |
| --- | --- |
| 2026-09-16 | Framework + rules + lifecycle + template + three seeded DRAFTs (Worker C1). |
| 2026-09-17 | **WO-P1-001 + WO-P1-002 CLOSED on merge** (PR #13 → d15333e; CI green both matrices; GUI evidence docs/research/evidence/wo-p1/). Operator directive: WO-LAB-001 first, then WO-P1-003. |
| 2026-09-16 | **C2 reconciliation:** WO-P1-DRAFT-001 → **WO-P1-001 CONFIRMED** (full runtime proof: entry-surface silent no-op, zero-tab dock, guard lib.rs:8163); WO-P1-DRAFT-002 → **WO-P1-002 CONFIRMED** (palette/shortcut-only, InspectorPane::Browser lib.rs:259-267, guard lib.rs:14521); WO-PLAT-DRAFT-001 → **WO-PLAT-001 CONFIRMED** (platform gap, documentation-only). New confirmed orders from the audit: **WO-P1-003** (multi-folder local projects), **WO-P2-004** (palette settings-page indexing), **WO-P2-005** (slash-command coverage delta), **WO-P2-006** (side chats), **WO-LAB-001** (official Linux app lab upgrade; new LAB class). §5 records the no-work-order scope decisions. Pending Tech Lead convergence. |
