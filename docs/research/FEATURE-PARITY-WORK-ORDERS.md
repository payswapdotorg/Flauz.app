# Feature Parity Work Orders

> **Status: ACTIVE — parity implementation waves complete: WO-P2-005
> CLOSED on main (merge `e46ad4f3df` via PR #17, 2026-09-18; incl. the
> fork-picker GUI fix 4726dd4) and WO-P2-006 CLOSED on main (merge
> `5287c29f3f` via PR #18, 2026-09-18); WO-PLAT-001 CLOSED (docs-only
> platform-gap closure, 2026-09-18).** Wave 3: WO-P2-004 CLOSED (merge
> `7aa7163` via PR #16). Wave 2: WO-P1-003 CLOSED (merge `d06ae3b` via
> PR #15). Remaining: VWO-016 (operator-gated held-out briefs). Operator
> directive (2026-09-17): execute WO-LAB-001 (official Linux preview app
> in LINUX_GUI_LAB) and reconcile current-26.825 behavior BEFORE WO-P1-003
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
- **Title:** Multi-folder local projects — in-baseline official capability wholly absent — status: **CLOSED (merged 2026-09-17)** [merged to main via PR #15 — merge commit d06ae3b562a0af072278ad920910922b20d8e085; implementation 1d2abca, clippy fixes 2ea5970 + d280a4b, evidence+docs 11d58f8; CI green on windows-latest + ubuntu-24.04 at d280a4b (dependency policy, fmt, patched-markdown checks, clippy -D warnings, full workspace tests, release build, Linux Xvfb startup smoke); implemented on parity/wo-p1-003-multi-folder (1d2abca; clippy follow-up 2ea5970): LocalProjectSummary.folders model (cap 16, primary never a member, legacy single-path loads primary-only), Add/Remove/SetPrimary actions with honest guards, primary swap re-keys the registry row (identity + manual order preserved; old primary parks at the front of related), related folders join fuzzy file search + file open/reveal after the primary while cwd/Git/AGENTS.md/skills/config.toml stay primary-only, storage schema v4 (workspace_folders, cascade, v3-to-v4 migration), Edit project surface (project action menu + per-row affordance entries; Primary badge, Make-primary/Remove, Add folder with honest cap tooltip, Done; surface follows a successful swap); 601 workspace tests green locally incl. 195-test codex-app harness; runtime GUI evidence docs/research/evidence/wo-p1-003/ (VLM-read: multi-folder project listed + primary-cwd indicator; palette file search hits the RELATED folder; Edit surface; Make-primary swap with surface-follow + identity preserved; cwd follows new primary; swapped state persists across close/reopen); runtime finding: Ctrl+P bound to OpenFileSearch but unregistered anywhere (silent no-op) — palette Search files is the working entry (P3-row candidate, reported to operator); all five closure gates satisfied 2026-09-17: source merged (d06ae3b), tests green (CI both matrices + 601 local), GUI evidence docs/research/evidence/wo-p1-003/ (VLM-read), LINUX_GUI_LAB scene passed (d6-v5), parity report §5.1 row + §8.2 flipped CLOSED on merge]
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
- **Title:** Command palette does not index all settings pages — official reference now runtime-observed on Linux: the official palette's dynamic Settings group lists 10 pages at the login surface (General, Import, Appearance, Voice, Pets, Git, Connections, Environments, Worktrees, Configuration) `[runtime-observed: linux-preview 26.908.70816; evidence docs/research/evidence/codex-linux/03-04]` — status: **CLOSED (merged 2026-09-17)** [merged to main via PR #16 — merge commit `7aa716358b963225e11eda1058c2b762404ee02b`; implementation 5d4b083, evidence+docs 6bb619d; CI green on windows-latest + ubuntu-24.04 at 6bb619d (dependency policy, fmt, patched-markdown checks, clippy -D warnings, full workspace tests, release build, Linux Xvfb startup smoke; the 5d4b083 windows run was auto-cancelled by the concurrency group when 6bb619d superseded it — expected, code-identical); implemented on `parity/wo-p2-004-palette-settings` (5d4b083): six new palette commands (Profile, Import, Browser, Configuration, Hooks, Git) with title/description/icon mirroring the settings-nav rows, Settings palette group, no guards (nav-reachable surfaces are unauthenticated and repository-independent, matching the official login-surface reference); `PaletteCommand::ALL` 45 → 51; `Personalization` palette title aligned with its settings-nav label; `SettingsSection::DEFAULT_NAV_SECTIONS` registry (15 default-nav sections; contextual CodeReview/Worktrees/ArchivedChats stay out per known limitations) + `settings_section` dispatch mapping (#[cfg(test)], house pattern); tests: registry coverage (every default-nav section has a palette entry; OpenPlugins is the documented Plugins alias) + the filtering contract (querying by each section's nav name resolves to an entry that opens that page); 603 tests green locally (406 workspace + 197 codex-app harness); runtime GUI evidence docs/research/evidence/wo-p2-004/ (15 VLM-read captures: palette Settings group lists all 11 entries incl. the six new ones — folded + scrolled views; query “import” resolves — the ev/18 remediation — and navigates to the Import page; Profile/Browser/Configuration/Hooks/Git queries each resolve and navigate to their pages, unauthenticated, mirroring the official login-surface reference); scene d7 + d7b passed; all five closure gates satisfied 2026-09-17]
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
  the Composer row [post-WO-P2-006: six — `/side` shipped; RWO-022 FW-2]; the current-docs list is `[docs-derived]` — exact
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

### WO-P2-007

- **ID:** WO-P2-007 (created by the Tech Lead from §9 override 17 — the parity-quality audit directive)
- **Title:** Ctrl+P file search is a silent no-op — bound action has no command routing
- **Platform:** all (UI-shell concern; Linux-verifiable in lab)
- **Reference behavior:** Official Codex opens file search on Ctrl/Cmd+P — the keyboard-shortcuts overlay advertises "Search Files... Ctrl+P" (runtime-observed: linux-preview 26.908, evidence codex-linux/05-07; istorical-record: A §10 + PM "Keyboard and accessibility").
- **Current behavior (at base 3c9f113):** ui.rs:4636 binds Ctrl+P to the `OpenFileSearch` GPUI action (declared ui.rs:1636) but NO `on_action` handler exists anywhere — while the command-id interceptor registry already carries `searchFiles → CmdOrCtrl+P` with a live dispatch arm (ui.rs:10480). The dead action shadows the working registry route: registered-shortcut-but-no-action silent no-op (§9 override 17).
- **Gap type:** Input-surface integrity defect (binding wiring, not a capability gap — palette "Search files" works).
- **Required change:** Remove the dead `OpenFileSearch` action + binding so the interceptor registry is the sole routing path for Ctrl+P (mirroring Ctrl+K/Ctrl+G), plus a regression test asserting ownership. OUT of scope: everything else (the no-workspace Files-palette guard, other dead actions, overlay honesty).
- **Files-crates:** `crates/codex-app/src/ui.rs` only.
- **Dependencies:** §9 override 17; the WO-R-SWEEP input-surface sweep (F-A4 documents the no-workspace residual).
- **Tests:** `ctrl_p_routes_to_the_search_files_command` — Ctrl+P owned by exactly ["searchFiles"] in ACTIVE_KEYBOARD_SHORTCUTS; id in KEYBOARD_SHORTCUT_COMMAND_IDS; PaletteCommand::SearchFiles.shortcut_command_id() == Some("searchFiles"); advertised shortcut Ctrl+P; registry metadata (title "Search files", CmdOrCtrl+P, General).
- **GUI verification:** LINUX_GUI_LAB D10 baseline (defect proof: byte-identical frames on unfixed main) + D10b (fix proof, workspace-seeded via the StorageOpened recent-workspace restore path): Ctrl+P opens the "Search files" palette (57522B a9ad7e0c → 58675B 857b824c, VLM-read: placeholder "Search files", Files section, focused input); Escape closes byte-identical. Evidence docs/research/evidence/wo-p2-007/ (d10-baseline in parity-lab + d10b) — archived with the PR.
- **Acceptance criteria:** (1) implementing commit merged; (2) focused test green locally + full battery on CI; (3) D10b GUI evidence; (4) no other behavior changed (diff: ui.rs +40/−2); (5) parity report Keyboard row + §9 override 17 updated (this closure).
- **Known limitations:** With NO workspace open the Files palette early-returns by design (open_command_palette guard) — Ctrl+P remains silent on the bare entry surface (finding F-A4 of the WO-R-SWEEP sweep); parity treatment of the no-workspace state is follow-up material. Local clippy-driver has a known toolchain issue (E0463 under clippy only); the CI double matrix is the authoritative battery.
- **Status:** CLOSED — implemented by Worker B (2948bf2, dispatched from inside the replay) + Lead rustfmt gate fix (5574c95); PR #20 merged as a3c0e01 (CI green both matrices); closure gates: source ✓, focused test ✓, GUI behavior ✓ (D10b), lab ✓, parity row ✓ (this commit).

### WO-P2-008

- **ID:** WO-P2-008 (wave S, scoped from WO-R-REF finding R1)
- **Title:** Per-chat unread-attention state + the four attention bindings
- **Platform:** all (cross-platform app code)
- **Reference behavior:** 26.727 Activity view — bell/Ctrl+Alt+U over a needs-attention store, Shift+Esc clears, Ctrl+Alt+A next-needing-attention; per-chat mark-unread Ctrl+Shift+U (view shape auth-walled `[unverified]` — reference research WO-R-REF R1).
- **Current behavior (base 34da7ed):** zero unread state — no flag on background completions/approvals, no bindings, no dot.
- **Delivered:** bounded `needs_attention_task_ids` session state (capped `MAX_VISIBLE_THREADS`, not persisted; visit clears, archive drops; background turn completion incl. failed turns + approval requests in non-selected chats flag the chat) + sidebar 6px dot + medium-weight title + four persisted registry commands (MAX_KEYBOARD_SHORTCUT_COMMANDS 71→76): `toggleThreadUnread` Ctrl+Shift+U (round-trip, honest no-selection status), `nextUnreadChat` Ctrl+Alt+A (cyclic sidebar-order jump; selected never a candidate; honest empty statuses), `clearAllUnread` Shift+Escape (honest count), `toggleActivityView` Ctrl+Alt+U (honest guidance — view surface is a separate future WO).
- **Files-crates:** `crates/codex-core/src/lib.rs` (+349), `crates/codex-app/src/ui.rs` (+397/−18).
- **Dependencies:** WO-R-REF R1 (scoping); WO-P2-007 input-quality doctrine (honest guidance over silent no-ops); supersedes the WO-R-SWEEP MAX=71 quirk (registry now 76 = constant).
- **Tests:** 7 codex-core state tests (background completion marking incl. failed turns + selected/unknown exclusions; approval marking; visit-clears; manual toggle round-trip; archive drops; clear-all honest reporting; activity-view honest guidance) — verified locally with per-test output; 5 codex-app binding tests (four binding resolutions — exact-one-owner accelerators, registry membership + metadata, reducer resolution — plus the cyclic sidebar-order jump test; RWO-022 FW-1 correction: the delivery adds exactly five, the recorded "6" was a count error) — CI double matrix.
- **GUI verification:** LINUX_GUI_LAB D11b (all four bindings resolve visibly on the real binary; VLM-read verbatim statuses) + D11 (honest no-selection path; full-flow scene documented the lab's no-runtime limitation — no codex CLI in lab, dot-on-row not GUI-exercisable, unit-covered). Evidence docs/research/evidence/wo-p2-008/.
- **Acceptance criteria:** (1) implementing commit merged; (2) focused tests green (core local + app CI); (3) D11/D11b GUI evidence; (4) diff confined to the two prescribed files (+728/−18); (5) parity report §5.10 row + §9 override 18 + §8 counts updated (this closure).
- **Known limitations:** Activity view surface pending (future WO; binding gives honest guidance); unread-state persistence across restarts unverified in the reference (session state by design); dot-on-row GUI observation requires a runtime-enabled lab (documented residual, same class as 007's F-A4). Local test-mode gpui compile is forbidden by the local OOM discipline with the live stack up — codex-app binding tests delegated to the authoritative CI matrix.
- **Provenance:** delivered by the 21:37 dispatch (chat destroyed 22:09–22:18 in the rate-limit purge — stream-death ≠ work-death; branch push c37c21b at 22:01), harvested and verified by the Lead after session resume. **Duplicate delivery 64b28f6** (the orphaned 23:20:59 re-dispatch's delayed sandbox push, force-pushed 00:14 UTC — leaner implementation, 4 consolidated tests) adjudicated **redundant**: the c37c21b lineage passed all five gates first and PR #23 ran from the protected branch `feat/wo-p2-008-unread-attention-verified` so the still-active worker could not move the PR head.
- **Status:** CLOSED — Worker delivery c37c21b (dispatched from inside the replay) + Lead verification (fmt, 7/7 core tests per-test verified, guarded build BUILD_EXIT=0, D11/D11b scenes); PR #23 merged as 00a3392 (CI green both matrices); closure gates: source ✓, tests ✓ (core local + app CI), GUI behavior ✓ (D11b), lab ✓, parity row ✓ (§5.10 + §9 override 18 + §8 counts).

### WO-P2-009 (CLOSED)

- **ID:** WO-P2-009 (wave S; scope from WO-R-REF R2)
- **Title:** Browsing history — persistent store + address-bar revisit + Settings management
- **Scope delivered:** `browsing_history` table (schema v5 migration; `MAX_BROWSING_HISTORY` bounded ring), address-bar revisit matching (`browsing_history_revisit_target`), Settings > Browser management surface (bounded first-8 rows + Clear… confirmation modal + empty state + footer). *(2026-09-19 correction at WO-P2-011 closure: this row's original scope-delivered list ended "reload/copy-URL keybindings" — no such keybindings existed at any 009-era head (the 011 focus-gate test asserts zero registry rows own the reload chords and it passes on the 009-merged base); the browser chords are delivered by WO-P2-011 instead.)*
- **Acceptance criteria:** (1) implementing commit merged — PR #24 → `5302e7e` (`0eed8c3` after rebase+fmt); integration-fix PR #27 → `fe3903e` (`0375e0f`); (2) focused tests green — storage 2/2, core 3/3, platform 2/2 locally per-test-verified at harvest; codex-app binding tests green on the CI double matrix (both PRs); (3) D12-series GUI evidence incl. the integration-bug find, the five-cycle isolation, and the fixed-binary full-scene pass (d12f); (4) diff confined to the prescribed files (+ the fix confined to the two row-render sites, mirroring the in-repo proven convention); (5) parity §5.4 browser row ux note + this row + changelogs (this closure).
- **Integration bug found + fixed at harvest (the five-cycle layout war):** D12 found every history row rendering a lone `…` (titles/URLs collapsed; timestamps rendered). The bug survived three single-theory fixes — `flex_1` (d12b), `justify_start` (d12c), `w_full` (d12d) — with **byte-identical frames each time** (md5-verified in-tree: `05-settings-browser.jpg` = `4a7b5a9c…` across d12→d12e), which itself became the diagnostic: pixel forensics (ink-cluster crops at 3x/8x zoom, d12c/d12e — even a seeded 1-char title rendered `…`, proving zero-width divs rather than content overflow) isolated the class, and differential analysis against the command-palette row (`render_command`, VLM-verified in evidence `wo-p2-010/d13/`) identified the repo's proven two-line convention. Fix `0375e0f` (PR #27 → `fe3903e`) mirrors that convention exactly (stateful row, `flex_1`+`min_w_0` column without gap, plain title div, `truncate()` on the secondary line only) and applies it to the downloads-modal rows (same text-stack class). d12f full-scene PASS: rows render verbatim (first frame-hash change across all builds; ink 0→1112 px), Clear modal verbatim, cleared empty state + disabled Clear. Lesson: rows mixing a growable column with `justify_between`/basis-0 `flex_1`/`truncate()`-on-children are a broken layout class in this gpui/Taffy — audit new rows against the palette-row convention.
- **Known limitations:** browser-panel / address-bar revisit surfaces NOT RUN in lab (runtime-less entry surface, D11b doctrine — covered by codex-platform matching tests + CI codex-app binding tests); the reference's Google-search fallback on no-match revisit is unverified reference behavior (documented, not implemented speculatively); downloads-modal rows fix is source-cited only (GUI runtime-gated in lab).
- **Provenance:** delivered by the wave-S dispatch (session destroyed in the 22:31–22:42 server-side purge; branch `feat/wo-p2-009-browsing-history` over-delivered `96d611e` → `5c795e7` while unattended — stream-death ≠ work-death), harvested + verified by the Lead (rebase onto `d479c7b`, union-resolved trivial ui.rs test-import conflict, fmt; integration fix authored at harvest verification, PR #27).
- **Status:** CLOSED — PR #24 (`5302e7e`) + integration-fix PR #27 (`fe3903e`); CI green both matrices (both PRs); evidence `docs/research/evidence/wo-p2-009/` (README run table: d12 find → d12b/d12c/d12d theory-attempts → d12e seed-probe → d12f full pass).

### WO-P2-010 (CLOSED)

- **ID:** WO-P2-010 (wave S; scope from WO-R-REF R3 absent-commands delta)
- **Title:** Command-palette rows for the evidenced registry commands
- **Scope delivered:** palette rows for `toggleReviewTab`, `forkThread` ("Continue in new chat"), `copyDeeplink`, `copySessionId`, `copyWorkingDirectory`, `approval.approve`, `approval.decline`, `renameThread`, `gotoChat 1..9` ("Go to chat N") with title/description mirroring the registry, plus availability guards per command (no visible dead rows).
- **Acceptance criteria:** (1) implementing commit merged — PR #25 → `876bbe8` (`10b0c24` after Lead fmt); (2) tests green — 3 delivery tests (palette↔registry title/description parity per evidenced id; availability guards; slot rows) + the standing registry battery, CI double matrix; (3) D13 GUI evidence (guards proven live at the entry surface; "Go to chat 1" verbatim); (4) diff additive (+536/−4); (5) parity §5.10 keyboard row note + this row + changelog (this closure).
- **Variant adjudication:** three over-delivered variants while unattended — `0eeef7d` (3 tests incl. palette↔registry parity per evidenced id; availability guards; +536/−4 additive) adjudicated canonical; `-b` `4c66e73` (rewrote match arms, 2 tests) and `-c` `0417faf` (2 tests) recorded redundant.
- **Known limitations:** present-rendering of the eight chat-scoped rows requires a selected chat = the official codex CLI runtime (absent in lab — D11b doctrine); the guards' honest absences are the GUI-observable behavior at the entry surface, with unit coverage on CI.
- **Provenance:** delivered by the wave-S dispatch (re-dispatched after the purge under recover_capacity serialization; three variant pushes 01:37–03:34 while unattended), harvested + adjudicated + verified by the Lead.
- **Status:** CLOSED — PR #25 (`876bbe8`); CI green both matrices; evidence `docs/research/evidence/wo-p2-010/` (D13).

### WO-P2-011 (CLOSED)

- **ID:** WO-P2-011 (wave T; scope from WO-R-REF R2 keyboard residuals + Worker A research A §4)
- **Title:** Browser keybinding residuals — context-scoped reload, force-reload and copy-URL chords
- **Scope delivered:** context-scoped interceptor arms (the searchFiles arm pattern — NOT global keymap bindings): browser pane focused → Ctrl+R reload, Ctrl+Shift+R force-reload (distinct `Action::ForceReloadBrowser` id), Ctrl+Shift+C copy-URL to clipboard ("Copied Browser address"); outside browser focus the three chords fall through untouched (Ctrl+Shift+C keeps its registry meaning `copyWorkingDirectory` — the official dual-meaning chord); honest no-page guards in the neighboring browser guards' voice ("Open a page before reloading the Browser." / "Open a page before copying the Browser address." — never a silent no-op, WO-P2-007 doctrine); interceptor-only by design (no customizable-registry rows — the bounded default; parity row Lead-owned).
- **Acceptance criteria:** (1) implementing commit merged — PR #31 → `3c812a7` (worker `a0cfa5c` + Lead rustfmt `62683bf`, exact base `5516084`); (2) tests green — 3 delivery tests (focus-gated resolution + fall-through + no-hijack + exactly-one-owner copyWorkingDirectory; copy-URL decision path incl. `about:blank`/None guidance; reload no-page honesty + ready-session `Effect::BrowserReload` dispatch) on the CI double matrix (full workspace battery, both matrices green); local battery NOT RUN at the integration station — disk-exhausted 4G sandbox (recorded honestly; CI is the authoritative gate and ran the identical gates); (3) D14/D14b GUI evidence (entry + workspace fall-through byte-identical / no-global-hijack / palette surfaces intact / the guarded copyWorkingDirectory row's honest palette absence consistent with D13; the enabled-path render and browser-pane-focused surfaces NOT RUN per the D12 runtime-less doctrine — unit-covered on CI); (4) diff confined to the two prescribed files (+289/−15 pre-fmt: ui.rs + lib.rs only); (5) parity §5.4 browser row (chords clause + resolution cell) + this row + changelogs + the 009-row scope correction (this closure).
- **Known limitations:** no-cache force-reload variant needs a platform-side CDP `Page.reload {ignoreCache: true}` command (outside this WO's file boundary — documented at the seam, delayed capability, never faked; both chords route to the live reload effect today); browser-pane-focused GUI observation is runtime-gated in the lab (D12 doctrine — the three unit tests carry the semantics on CI); clipboard write itself not GUI-observed (headless tests exercise the decision path; D14 observes the sibling `copyWorkingDirectory` status as the visible-fallback evidence class).
- **Provenance:** delivered by the wave-T dispatch (first chat f1426555 capacity-stuck 900s → queue_watch staleness assault #1 → chat 8963e5f3); stream death at 13:42:38 UTC after 28 min of generation (server-side: msgs=2, last assistant len=6 husk — the in-chat completion report never landed); deliverable complete in git (`feat/wo-p2-011-browser-bindings` @ `a0cfa5c`, single bounded commit, exact base `5516084`) — **harvested from git per the RWO-022 stream-death ≠ work-death precedent**; Lead rustfmt pass on top (`62683bf`); dead session voided, slot freed, watcher retired cleanly (void-while-watching would have double-dispatched).
- **Status:** CLOSED — PR #31 (`3c812a7`); CI green both matrices; evidence `docs/research/evidence/wo-p2-011/`.

### WO-P2-012 (CLOSED)

- **ID:** WO-P2-012 (wave T scope, wave-1-A reconciliation; scope from WO-R-REF input-surface sweep F-findings + RWO-021/RWO-022 honest-status advisories + FW-9)
- **Title:** Guard honesty for the six evidenced silent no-op states + executor fall-through tests
- **Scope delivered:** honest guidance instead of silent no-ops for F-A1 archive (Ctrl+Shift+A, no selection → "Select a chat before archiving it."), F-A2 pin (Ctrl+Alt+P → "Select a chat before pinning or unpinning it."), F-A3 rename (Ctrl+Alt+R → "Select a chat before renaming it." + stale-selection "The selected chat is no longer available."), F-A6 commit-or-push while a PR workflow is pending (→ "A Git workflow is already running."), F-D1 typed `/review` executor (ReviewSlashCommandAction enum: unavailable+closed → FallThrough so the command submits as a visible message; submenu-open → StartDefaultReview; available → OpenSubmenu; plus the second site — review start while unavailable reports composer_review_unavailable_status), F-D2 `/compact` while the selected thread's runtime is still loading (core composer_error "Wait for the chat to finish loading before compacting context." mirroring the neighboring guards; recovers when the runtime loads); FW-9 executor fall-through tests (review falls through to visible CreateTask submission; compact reports + recovers; SetStatus → status_message observability).
- **Acceptance criteria:** (1) implementing commit merged — PR #34 → `afa5b9d` (worker r3 `aeccfc9`, exact base `a664644`; the r1 `0547055` broad approach and r2 `4fc763a` narrow rework superseded by the r3 reconciliation); (2) tests green — 8 focused delivery tests (per-state status helpers ×4, review action enum, review-start status, FW-9 fall-through ×2 + the F-D2 core guard test updated from silent-no-op to honest-status assertion) on the CI double matrix; local: cargo fmt --check clean on the branch (Lead integration station); guarded incremental build for the D15 binary; full workspace battery carried by the CI double matrix (authoritative per the WO-P2-011 NOT RUN doctrine — disk-exhausted 4G sandbox); (3) D15 GUI evidence (F-D1 fall-through VERIFIED on the real binary; guard-family SetStatus mechanism VERIFIED at the entry surface on the r3 binary (d15d d07); F-A1/A2/A3 chord paths shadow-blocked by the pre-existing bound-but-never-handled KeyBinding class (A/B-identical on 3c812a7); F-D2 NOT OBSERVED in lab; F-A6 NOT RUN offline); (4) diff confined to the two prescribed files (+395/−15: ui.rs + codex-core lib.rs only; F-A4/F-A5 explicitly out of scope); (5) parity §5.10 Keyboard row (Flauz cell + Gap cell) + this row + changelogs (this closure).
- **Known limitations:** F-A4 (Ctrl+P silent with no workspace) remains — D-3 decision per the WO-F1-SWEEP-001 packet (WO-P2-020 or a WO-P2-012 successor; the palette Files-mode row is honestly hidden in that state, the keyboard path is not); F-A5 (Ctrl+1..9 empty slot) documented as plausibly-by-design; F-A6's pending-PR state is not constructible in the offline lab (unit-covered on CI; the palette-record frame is captured in D15); the F-D2 loading window is timing-sensitive in the lab (the unit test carries the deterministic semantics; the D15 probe lands the observable guidance when the window holds).
- **Provenance:** two unmerged prior deliveries existed on origin (r1 broad per-command status-fn `0547055`; r2 narrow rework `4fc763a` — the queue machinery had cycled redundant re-dispatches while misclassifying git-delivered work as zero progress); wave-1 Worker A (session 5FB43A76) reconciled both into the complete r3 delivery on base `a664644` and pushed `aeccfc9` at 21:11 UTC; the session tab died 3 minutes later (tablost) and the machinery re-dispatched twice more — the redundant live session was voided at harvest (work already on git; stream-death ≠ work-death doctrine). Harvested from git + Lead full-diff reviewed (enum-based fall-through, WO-P2-008 copy-family consistency, no scope creep).
- **Status:** CLOSED — PR #34 (`afa5b9d`); CI green both matrices; evidence `docs/research/evidence/wo-p2-012/`.

### WO-P2-013 (CLOSED)

- **ID:** WO-P2-013 (wave-1 Worker B; roadmap F1 item "Activity-view surface corresponding to the deferred `toggleActivityView`", journey J-17)
- **Title:** Activity view surface — complete the visible Activity experience around the existing attention primitives
- **Scope delivered:** real bounded Activity view replacing the honest placeholder reduce arm — `Action::ToggleActivityView` toggles `activity_view_visible` (non-persisted session flag; no second event store); `Action::CloseActivityView` idempotent Escape-path close; `activity_view_rows()` lists the visible chats flagged in the existing `needs_attention_task_ids` (sidebar order, pinned first, bounded by the existing cap semantics — flags for non-visible chats simply have no row); landing in a chat via the exact `nextUnreadChat` path (`SelectTask`) resolves attention through the existing visit-clears semantics and dismisses the surface; full keyboard path (scoped `ActivityView` context: up/down/enter/escape, focus-tracked panel, wrap-around selection, bare-Escape fallback wins while topmost — Shift+Esc preserved for clear-all-unread); honest empty state ("No chats need attention" + plain-language mark-unread guidance; no fake loading; no invented error state); command palette row "Toggle Activity view" (Bell icon) wired to the same action, registry copy verbatim (the WO-P2-010 row contract) — fulfilling the WO's stated palette discovery surface.
- **Acceptance criteria:** (1) implementing commit merged — PR #36 → `c10b426` (worker final `20a6018`, exact base `a664644`, + Lead rustfmt pass `6d08bf3` for the delivery's own fmt drift — the 011 precedent); (2) tests green — 6 focused delivery tests (core: toggle open/close, idempotent close, jump-through-existing-semantics incl. the manually-flagged-selected-chat preservation; app: binding→surface toggle, rows-reflect-attention-state-in-sidebar-order, palette-row-reuses-registry-copy) + regression guards (app attention/palette 10P, core attention family 6P) locally at the Lead integration station; full workspace battery on the CI double matrix (both legs green @ `6d08bf3`); (3) D16 GUI evidence on the delivery binary (open/count/rows/selection/keyboard-footer VERIFIED; Down-arrow selection move VERIFIED; Enter-jump with panel-dismiss + visit-clear + other-dot-preserved VERIFIED; reopen "1 chat needs attention" VERIFIED; toggle-close byte-identical to the post-jump frame; empty state verbatim VERIFIED); (4) diff confined to the two prescribed files (+518/−33: ui.rs + codex-core lib.rs only); (5) parity Projects-and-chats row D-2 staleness fix + parity-report Activity row update + this row + changelogs (this closure).
- **Known limitations:** recently-engaged chats deliberately not listed (no existing session state exposes them without a second store — the handoff's explicit non-goal); bell chrome icon + sidebar-dot-click contextual discovery recorded as WO-P2-020 follow-up material (the WO's do-not-force clause); unread-state persistence across restarts unchanged (reference behavior unverified, P3).
- **Provenance:** wave-1 Worker B fought the platform's peak-hour capacity congestion for ~9.5 hours across 10 dispatches (queued-capacity cycles + tablossaults); each working session delivered via git before its stream died and the next session iterated on the prior delivery — five pushes (r2 `bf5eacf` → prior `690a246` → prior-2 `520eed3` → r3 `1c05511` → final `20a6018` force-pushed onto the canonical branch, superseded attempts self-archived by the worker under `archive/wo-p2-013-prior-attempt{,-2}`). The final session (E1DD2E36) husked at 04:00 UTC with a 6-char assistant stub but pushed the final delivery at 04:27:45 (stream-death ≠ work-death, third documented occurrence); the Lead stood down the redundant queue machinery (watcher + an in-flight 6th create killed mid-send; the orphan tab closed before generation) and harvested from git. Lead full-diff reviewed (all ten acceptance criteria; no scope creep; the palette row addition fulfills the stated discovery surface).
- **Status:** CLOSED — PR #36 (`c10b426`); CI green both matrices; evidence `docs/research/evidence/wo-p2-013/`.

### WO-P2-018 (CLOSED)

- **ID:** WO-P2-018 (f1-sweep renumbered follow-up; roadmap F1 discoverability item, PJ §1 layer-1 contract)
- **Title:** Visible-entry + label parity for icon-only controls
- **Scope delivered:** visible command-palette entry button in the chrome (`TitleBarDiscoveryEntry` model; `render_command_palette_entry_button` — native icon Button, navigate-back/forward tooltip style + primary chord) so the palette has a visible discovery point instead of chord-only; Activity-view bell in the title bar (`render_activity_view_bell_button`) dispatching the existing `Action::ToggleActivityView` (the J-17 / WO-P2-013 surface); unread-attention dot text alternative (`UNREAD_ATTENTION_DOT_TOOLTIP` = "Unread activity" — registry vocabulary, no internal terms); archived-chats single-deletion visible label (`ARCHIVED_CHAT_DELETE_LABEL` = "Delete" + "Delete archived chat" tooltip) matching the single-scope confirmation title it leads into.
- **Acceptance criteria:** (1) implementing commit merged — PR #37 → `b562397ce6325f12d21f67f0549f5860b7801306` (worker delivery `0ece5bb`, single commit directly on base `8172f6e` exactly, ui.rs-only +241/−15); (2) tests green — 4 focused render tests (registry-copy reuse, bell toggle, dot tooltip, archived label) + full codex-app suite 229P/0F locally (main = 225 + 4); CI double matrix green on the PR; (3) D18 GUI evidence (title-bar entries render; Search click opens the Unified palette; Bell click opens the Activity view with the honest empty state; dot-hover tooltip verbatim; archive status; Settings → archived shows visible Delete + Unarchive + "Delete all" — geo-probe cross-check for the cold-start entries); (4) diff confined to the prescribed file; (5) §5.10 Keyboard row residual + resolution cells + this row + changelogs (this closure).
- **Known limitations:** screen-reader labels / AT tree, OS-level reduced-motion, status AT announcement, titlebar AT naming remain platform-bound (upstream-GPUI dependencies — the f1-sweep close packet; revisit at F11); broader focus order + full contrast parity remain the bounded F1 close-out slice (honestly open, PM baseline).
- **Provenance:** f1-sweep renumbered follow-up (packet IDs WO-P2-013..016 → WO-P2-017..020); worker delivered `0ece5bb` pre-reset; post-reset Lead verification wave: guarded build distinct-hash `b019dc67`, focused 4P/0F, suite 229P/0F, fmt clean (write-mode diff), D18 ALL PASS at display :108 (geometry-calibrated probes; geo-probe cross-check).
- **Status:** CLOSED — PR #37 (`b562397ce6325f12d21f67f0549f5860b7801306`); CI green both matrices; evidence `docs/research/evidence/wo-p2-018/`.

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
- **Title:** Run the official Linux desktop preview app in LINUX_GUI_LAB to upgrade Linux-A evidence to runtime-observed — status: **CLOSED (executed 2026-09-17)** [`chatgpt 26.908.70816` .deb selective userspace; launched under Xvfb :102 + picom with isolated HOME/XDG, no credentials; unauthenticated surfaces captured + VLM-verified under docs/research/evidence/codex-linux/ (login surface; palette with dynamic Settings group + Panels: Open terminal; searchable Keyboard shortcuts overlay, 22 rows; auth-pending surface; DB-recovery dialog); §5.3/§5.4/§5.9/§5.10 Linux official cells upgraded to runtime-observed with version-skew labels (26.908 = reference+1; 26.825 target unchanged); §9 overrides 14-16; E2B-PARITY-ENVIRONMENT.md lab-verdict note added (dated, not silent); acceptance criteria 1-4 satisfied — `[unverified]` items resolved: Linux Computer Use scope = docs-stated absent in preview (macOS/Windows only), Activity view + terminal/browser affordance shapes = auth-walled (official shell gates everything pre-sign-in)]
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

### WO-R-REF (Wave R reference research)

- **ID:** WO-R-REF (Tech Lead wave-R order — reference research, no product code)
- **Title:** P2 batch-2 reference research — Activity view, browsing history, palette residuals
- **Status:** DELIVERED — Worker A branch research/p2-batch2-reference @ 693a1da (one docs-only commit on 3c9f113, 672 lines); Lead-verified at dispatch; merged via PR #22 → bf61964. Evidence: docs/research/evidence/p2-batch2-reference/README.md.
- **Findings of record:** R1 Activity view = bell + Ctrl/Cmd+Alt+U over a needs-attention store + per-chat mark-unread Ctrl+Shift+U (view shape auth-walled [unverified]; Flauz has zero unread state). R2 Flauz already implements the Google fallback (browser_navigation_url, test-verified) — the real gap is the persistent browsing-history store + revisit matching + Settings management + reload/copy-URL keybindings (official uses context-scoped chords). R3 palette delta: absent commands enumerated; full 26.825 official palette inventory not enumerable from repo evidence (open question); 26.908 22-row overlay = reference+1 only. These scope WO-P2-008/009/010.

### WO-R-SWEEP (Wave R input-surface integrity sweep)

- **ID:** WO-R-SWEEP (Tech Lead wave-R order — adversarial verification sweep, no product code)
- **Title:** Input-surface integrity sweep — every binding, command id, palette entry, slash command, settings row, advertised shortcut
- **Status:** DELIVERED — Worker C branch research/input-surface-sweep @ 6c05afd (one docs-only commit on 3c9f113, 353 lines); merged via PR #21 → d659eb3. Evidence: docs/research/evidence/wo-p2-007/input-surface-sweep.md.
- **Findings of record:** ZERO dead commands (72/72 interceptor arms; registry ↔ ACTIVE_KEYBOARD_SHORTCUTS set-equal). 8 silent-state no-ops with anchors: F-A1 archiveThread / F-A2 toggleThreadPin / F-A3 renameThread (silent with no selected chat), F-A4 searchFiles Ctrl+P (silent with no workspace — survives the WO-P2-007 fix), F-A5 thread1-9 empty slots (matches the official "safely do nothing" contract — by design), F-A6 git.commit palette row with pending PR, F-D1 typed /review while unavailable, F-D2 /compact runtime-not-ready. Structural: 24/25 globally bound GPUI actions declared-but-never-handled (bind_keys = menu-accelerator-label provider; removal does not generalize — menu labels depend on the actions); MAX_KEYBOARD_SHORTCUT_COMMANDS=71 vs 72 ids (fully-customized users lose the last override); Plugins settings section not palette-indexed (OpenPlugins routes to Marketplace). Work-order material for the parity-quality audit continuation.

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
- 2026-09-17: **WO-LAB-001 CLOSED (executed)** — official Linux preview app (26.908.70816) runs in LINUX_GUI_LAB; unauthenticated runtime evidence archived (docs/research/evidence/codex-linux/); parity report Linux official cells upgraded with version-skew labels; §9 overrides 14-16; E2B lab-verdict note added (dated). Operator's next directive stands: WO-P1-003 (multi-folder local projects) is the next implementation.
- 2026-09-17: **WO-P1-003 IMPLEMENTED (PR #15)** — multi-folder local projects on parity/wo-p1-003-multi-folder (1d2abca; clippy follow-up 2ea5970): model + honest-guard actions + primary-swap re-key + related-folder file search (primary-only cwd/Git/config discovery contract kept) + storage schema v4 + Edit project surface; 601 tests green locally; runtime GUI evidence docs/research/evidence/wo-p1-003/; new runtime finding: Ctrl+P binding is an unregistered silent no-op (palette Search files is the working entry) — P3-row candidate. Closure gates: source ✓, tests ✓ (CI round 2), GUI evidence ✓, LINUX_GUI_LAB scene ✓; CLOSED on merge with parity-row + §8.2 updates.
- 2026-09-17: **WO-P1-003 CLOSED (merged)** — PR #15 merged to main as d06ae3b562a0af072278ad920910922b20d8e085 (implementation 1d2abca; clippy 2ea5970 + d280a4b; evidence+docs 11d58f8); CI green on both matrices at d280a4b; all five closure gates satisfied; parity report §5.1/§5.2/§8.2/J1/J4 + §9 override 17 updated; follow-on unblocked: multi-repository review (26.727) is now its own future order; open finding reported to operator: Ctrl+P binding is an unregistered silent no-op (palette Search files is the working entry).
- 2026-09-18: **WO-P2-007 CLOSED on merge** (PR #20 → `a3c0e01`; CI green both matrices): the §9 override 17 Ctrl+P silent no-op fixed — dead `OpenFileSearch` action removed, `searchFiles` interceptor arm owns Ctrl+P (mirroring Ctrl+K/G), 5-assertion regression test; Worker B delivery 2948bf2 + Lead rustfmt 5574c95; LINUX_GUI_LAB D10 baseline (defect proof) + D10b (fix proof, workspace-seeded: Ctrl+P opens the "Search files" palette, VLM-read; Escape closes byte-identical); F-A4 no-workspace residual documented (follow-up parity question).
- 2026-09-18: **WO-R-SWEEP + WO-R-REF DELIVERED** (PRs #21 `d659eb3` + #22 `bf61964`): Wave R research lands in the evidence tree — input-surface integrity sweep (zero dead commands; 8 silent-state no-ops; 24/25 dead bound actions; MAX=71 quirk; Plugins palette gap) + P2 batch-2 reference (Activity view / browsing history / palette residuals — the WO-P2-008/009/010 scoping basis).

| Date | Change |
| --- | --- |
| 2026-09-16 | Framework + rules + lifecycle + template + three seeded DRAFTs (Worker C1). |
| 2026-09-17 | **WO-P1-001 + WO-P1-002 CLOSED on merge** (PR #13 → d15333e; CI green both matrices; GUI evidence docs/research/evidence/wo-p1/). Operator directive: WO-LAB-001 first, then WO-P1-003. |
| 2026-09-17 | **WO-LAB-001 CLOSED (executed)**: official Linux preview app 26.908.70816 runtime-observed in the lab (unauthenticated slice; evidence codex-linux/); Linux official cells upgraded with version-skew labels; WO-P2-004 reference upgraded to runtime-observed. Next: WO-P1-003. |
| 2026-09-17 | **WO-P1-003 IMPLEMENTED (PR #15, 1d2abca + 2ea5970)**: multi-folder model, Edit project surface, primary-swap re-key, related-folder file search, schema v4; 601 tests green locally; GUI evidence wo-p1-003/ (incl. new Ctrl+P silent-no-op finding). Closure on merge. |
| 2026-09-17 | **WO-P1-003 CLOSED on merge** (PR #15 → `d06ae3b`; CI green both matrices at d280a4b; evidence wo-p1-003/; §5.1 row + §8.2 flipped). Wave-2 acceptance complete. |
| 2026-09-17 | **WO-P2-004 CLOSED on merge** (PR #16 → `7aa7163`; CI green both matrices at 6bb619d; evidence wo-p2-004/; §5.10 row + §8.2 flipped). Wave-3 acceptance complete. |
| 2026-09-17 | **WO-P2-004 IMPLEMENTED (PR #16, 5d4b083)**: palette indexes every default-nav settings section (six new commands — Profile, Import, Browser, Configuration, Hooks, Git; `PaletteCommand::ALL` 45 → 51; Personalization title aligned with its nav label; `DEFAULT_NAV_SECTIONS` registry + coverage/filtering tests); 603 tests green locally; GUI evidence wo-p2-004/ (15 VLM-read captures: Settings group lists all 11 entries; "import" resolves — ev/18 remediated — and all six queries navigate to their pages unauthenticated, mirroring the official login-surface reference). Closure on merge. |
| 2026-09-16 | **C2 reconciliation:** WO-P1-DRAFT-001 → **WO-P1-001 CONFIRMED** (full runtime proof: entry-surface silent no-op, zero-tab dock, guard lib.rs:8163); WO-P1-DRAFT-002 → **WO-P1-002 CONFIRMED** (palette/shortcut-only, InspectorPane::Browser lib.rs:259-267, guard lib.rs:14521); WO-PLAT-DRAFT-001 → **WO-PLAT-001 CONFIRMED** (platform gap, documentation-only). New confirmed orders from the audit: **WO-P1-003** (multi-folder local projects), **WO-P2-004** (palette settings-page indexing), **WO-P2-005** (slash-command coverage delta), **WO-P2-006** (side chats), **WO-LAB-001** (official Linux app lab upgrade; new LAB class). §5 records the no-work-order scope decisions. Pending Tech Lead convergence. |
| 2026-09-18 | **WO-PLAT-001 CLOSED (docs-only)**: Computer Use — Linux platform gap recorded as a documented bound, not a missing feature — §5.5 row verified `platform-limited` (Backend/UI/Functional) with the portal-backed-selection future-work pointer; `docs/known-failures.md` verified as the canonical bound text (wording consistent with the row); no code change (per the work order's Required change: documentation/labeling only). |
| 2026-09-18 | **WO-P2-005 IMPLEMENTED (PR #17: 2e6358f + GUI fix 4726dd4)**: slash-command coverage — guarded `/approve` (pending-approval), `/fast` (catalog Fast tier toggle), `/personality` (Personalization settings nav), `/worktree` (worktree-fork picker); the fork-destination picker wiring defect found in LINUX_GUI_LAB (picker instantly closed for `/worktree` — Change-subscription value-gate) fixed in 4726dd4 via the testable `composer_keeps_fork_picker` predicate (+ regression test); 15-site picker-lifecycle audit clean; fmt gates pass on pinned 1.97.1; 605 existing tests unaffected (grep-verified no test touches the picker flag); GUI evidence wo-p2-005/ (D8c4 VLM-read). Closure on merge. |
| 2026-09-18 | **WO-P2-005 CLOSED on merge** (PR #17 → `e46ad4f3df`; CI green both matrices; evidence wo-p2-005/; §5.2 Composer row + §8.2 item 5 flipped; §-ref fixed per RWO-022 FW-4). Seven of the eleven absent names stay open (deferred capabilities / unverified semantics — enumerated on the Composer row). [post-WO-P2-006: six — `/side` shipped; RWO-022 FW-2] |
| 2026-09-18 | **WO-P2-006 IMPLEMENTED (PR #18: 764e4db + Lead rustfmt 73eb55c)**: side chats — Ctrl+Alt+S/Cmd+Alt+S binding (OpenSideChatShortcut in bind_keys + openSideChat interceptor registry command, Thread group; KEYBOARD_SHORTCUT_COMMAND_IDS 71→72) + guarded `/side` slash command (21 named commands, menu row); side submit creates the side thread without selecting it; main-chat selection + active turn untouched (state test); close returns to the main view; exactly the two prescribed files (ui.rs +381, codex-core lib.rs +469); core tests 221/221 green locally; GUI evidence wo-p2-006/ (D9/D9b VLM-read). Closure on merge. |
| 2026-09-18 | **WO-P2-006 CLOSED on merge** (PR #18 → `5287c29f3f`; CI green both matrices; evidence wo-p2-006/; §5.1 Side chats row (missing → complete; §-ref fixed per RWO-022 FW-4) + §8.1 counts (complete 7→8, missing 6→5) + §8.2 item 6 flipped). P2 with-work-orders list complete (3/3). |
| 2026-09-18 | **WO-P2-007 IMPLEMENTED + CLOSED on merge** (PR #20 → `a3c0e01`; Worker B 2948bf2 + Lead rustfmt 5574c95; CI green both matrices): Ctrl+P silent no-op fixed — dead `OpenFileSearch` removed, `searchFiles` owns the key via the interceptor; regression test (ownership + registry + command-id + advertised shortcut + metadata); D10 baseline defect proof + D10b fix proof (workspace-seeded Ctrl+P opens the "Search files" palette, VLM-read; F-A4 no-workspace residual documented). The §9 override 17 finding is remediated. |
| 2026-09-18 | **WO-R-SWEEP + WO-R-REF DELIVERED** (PR #21 → `d659eb3`, PR #22 → `bf61964`): input-surface integrity sweep (zero dead commands; F-A1..A6/F-D1/D2 silent states; 24/25 dead bound GPUI actions; MAX=71 quirk; Plugins palette gap) + P2 batch-2 reference research (WO-P2-008/009/010 scoping basis) merged into the evidence tree. |
| 2026-09-18 | **Wave S dispatched (from inside the replay, GLM-5.3 + Full-Stack)**: WO-P2-008/009/010 (unread-attention state; browsing-history store; palette rows) — the evening-capacity fight cost ~2.5h of assault churn (operator never-wait ruling); all three sent by 21:50, then the 22:31–22:42 server-side rate-limit purge destroyed the queued chats (008's delivery had already been pushed at 22:01 — stream-death ≠ work-death; 009/010 re-dispatched under recover_capacity serialization). |
| 2026-09-19 | **WO-P2-008 implemented + CLOSED on merge** (PR #23 → `00a3392`; CI green both matrices): unread-attention session state + four registry bindings + sidebar dot (full closure record in the WO-P2-008 row above). Duplicate delivery 64b28f6 adjudicated redundant (protected-branch PR prevented head movement). Parity report: §5.10 row missing → partial, §8 counts missing 5→4 / partial 31→32, §9 override 18, §8.2 P2 item 8 rescoped to the Activity view surface. Wave S remaining: WO-P2-009/010 in flight (capacity fight, machinery-owned). |

| 2026-09-19 | **WO-P2-009 implemented + CLOSED on merge** (PR #24 → `5302e7e` + integration-fix PR #27 → `fe3903e`; CI green both matrices): persistent browsing-history store (schema v5), address-bar revisit matching, Settings management (bounded rows + Clear modal + empty state), reload/copy-URL keybindings. D12 harvest found the history rows collapsing to `…` — the five-cycle layout war: three single-theory fixes (flex_1/justify_start/w_full) produced byte-identical frames (md5-proven in-tree), pixel forensics proved zero-width divs (a 1-char seeded title still rendered `…`), differential analysis against the command-palette row cracked it; fix mirrors the palette-row convention (also applied to downloads-modal rows). d12f full-scene PASS (rows verbatim, Clear modal verbatim, cleared empty state + disabled Clear). Parity §5.4 browser row ux note updated (history/Settings part CLOSED; Google-fallback residual). |
| 2026-09-19 | **WO-P2-010 implemented + CLOSED on merge** (PR #25 → `876bbe8`; CI green both matrices): palette rows for the evidenced registry commands (review/fork/copy×3/approval×2/rename/goto-chat-N) with per-command availability guards; three over-delivered variants adjudicated (variant 1 canonical). D13: guards live in the GUI (honest absences at the entry surface; "Go to chat 1" verbatim); present-rendering of chat-scoped rows is runtime-gated (D11b doctrine, unit-covered). Parity §5.10 keyboard row note updated. |
| 2026-09-19 | **RWO-020 review rubric merged** (PR #26 → `1b0500e`): WO-REVIEW-001 deep phase armed — rubric.md + rubric-r2.md on main (two dispatches reconciled in notes §N8); RWO-021-B (fresh end-to-end verification) in flight; RWO-022-C (adversarial) staged for dispatch on the post-#27 head. |

| 2026-09-19 | **RWO-021 review deliverable merged** (PR #28 → `61c491c`; research/rwo-021 @ `a622b7b`, base 876bbe8 exactly per packet): fresh end-to-end verification — rubric executed (13 rows / 71 criteria, r2 template); feasible battery green (fmt clean; codex-core 231 / protocol 60 / storage 19 tests, exact commands + exit codes recorded); input-surface invariants re-derived (MAX 71→76 quirk VERIFIED cured); **no product-code defect in any of the 8 complete rows**. 3 documentation defects: KA-3 stale 51→70 palette-registry count + KSR-C5 §5.10 Gap-cell F-findings enumeration gap (both remediated in this commit) + ledger IN-FLIGHT lag (already remediated `dbee361`). Advisory residuals recorded for wave-T re-scope (honest-status upgrades F-A1/A2/A3 — WO-P2-012 territory; evidence packaging md5/provenance; structural watch list). Session stream degraded http-500 after delivery (stream-death ≠ work-death); harvested + Lead-verified (anchors re-checked at ui.rs:3480 / ui.rs:8205). Evidence `docs/research/evidence/rwo-020/verification.md`. |

| 2026-09-19 | **RWO-022 adversarial challenge merged** (PR #29 → `230f6fc`; research/rwo-022 @ `17a1897`, base dbee361 exactly per packet): all five targets attacked from primary evidence — T1 WO-P2-005 picker fix + 15-site audit **HELD**; T2 WO-P2-006 side-chat semantics **HELD** (2 evidence defects: d9-008 mislabel, d9-006≡d9-007 byte-identical); T3 WO-P2-007 Ctrl+P/F-A4 **HELD** (exact md5-chain + anchor matches); T4 WO-P2-008 no-runtime-lab + MAX 71→76 **HELD on substance** (record defects: 6-vs-5 binding-test count, D11 plain-Return scene defect + mislabeled frames); T5 §9 override trail + §8 counts **HELD on substance** (4 stale/misdirected cells). **Zero product-code defects.** EQ-1 RESOLVED via fresh VLM reads: D9 "App-server online" (09-18) vs D11 "Couldn't connect" (09-19) = real lab environment drift, not fabrication. Findings remediated in this commit: FW-1 (6→5 tests ×5 sites), FW-2 (seven→six: live cell fixed + 3 historical annotations), FW-3 (§8.2 item 12 annotated), FW-4 (§8.1 prose census note + 7 §-ref fixes), FW-6 (wo-p2-006 README), FW-8 (wo-p2-005 md5 manifest + audit pointer). Open follow-ups: FW-5 (D11 script ctrl+Return fix + relabel — lab), FW-7 (lab codex-runtime pin/restore + D11 full-flow re-run — lab), FW-9 (executor guard fall-through tests — folds into WO-P2-012 scope). Chat stream died server-side post-delivery (6-char assistant); harvested from git, Lead spot-verified 4/4 load-bearing claims. Evidence `docs/research/evidence/rwo-020/adversarial.md`. |

| 2026-09-19 | **RWO-021 verification-r2 merged** (PR #30 → `bb52d4d`; research/rwo-021 @ `84bedd0`, second dispatch, reconciles with `a622b7b` per the rubric→rubric-r2 coexistence precedent): userspace-sysroot build at pinned 1.97.1 exit 0; **full battery 639 passed / 0 failed / 2 skipped across all five crates** with per-test WO-P2-008 runs; ten fresh GUI scenes (D10b Ctrl+P reproduction, feedback dialog via palette + typed /feedback + Ctrl+Enter, D11b four statuses verbatim, gh-missing toast, settings import filter); input-surface invariants 76=76=76, zero dead commands, `PaletteCommand::ALL`=70; 3 documentation-class defects (all already remediated on main). Evidence-only; no product code. Pushed post-DONE from the still-running sandbox (stream-death ≠ work-death). Evidence `docs/research/evidence/rwo-021/`. |
| 2026-09-19 | **WO-REVIEW-001 CLOSED** (operator-accepted comprehensive review + test of Flauz.app as a faithful Codex clone): phase 1 — reference-behavior research (wo-r-ref) + input-surface integrity sweep at `3c9f113` (zero dead commands 72/72; 8 silent-state no-ops F-A1..A6/F-D1/D2 → WO-P2-004/007 remediations; 24/25 bound GPUI actions accounted); deep phase — rubric (RWO-020 → `1b0500e`), fresh verification ×2 (RWO-021 → `61c491c` + verification-r2 → `bb52d4d`), adversarial challenge (RWO-022 → `230f6fc`). **Final verdict: no product-code defect found by any leg; every load-bearing closure claim survived primary-evidence re-derivation.** Documentation/evidence defects found and remediated: KA-3, KSR-C5, ledger lag (B), FW-1..FW-4 + FW-6/FW-8 (C). Open items: FW-5/FW-7 (lab: D11 script key fix + codex-runtime pin/restore for the full-flow scene), FW-9 (executor guard fall-through tests → WO-P2-012 scope), Activity-view surface (future WO), honest-status upgrades F-A1/A2/A3 (WO-P2-012 territory). Version bounds honored throughout: reference = ChatGPT desktop 26.825.51511; baseline = Codex Desktop 26.721.3996.0; 26.908+ = reference+1 (out of target); delayed capabilities documented, never faked. |
| 2026-09-19 | **WO-P2-011 implemented + CLOSED on merge** (PR #31 → `3c812a7`; worker `a0cfa5c` + Lead fmt `62683bf`, base `5516084` exactly): context-scoped browser-pane interceptor arms — Ctrl+R reload, Ctrl+Shift+R force-reload (distinct action id; no-cache variant documented as delayed capability at the seam), Ctrl+Shift+C copy-URL; fall-through untouched outside browser focus (copyWorkingDirectory dual-meaning preserved); honest no-page guards. 3 delivery tests green on the CI double matrix (local battery NOT RUN: disk-exhausted sandbox — honest record). D14/D14b: fall-through/no-hijack byte-identical + palette surfaces intact + guarded-row honest absence (D13-consistent) on the real binary; enabled-path render + browser-pane-focused surfaces runtime-gated (D12/D13 doctrine, unit-covered). Stream-death delivery harvested from git per the RWO-022 precedent. 009-row scope phrase corrected with attribution. Parity §5.4 row + resolution updated. Evidence `docs/research/evidence/wo-p2-011/`. |
| 2026-09-19 | **RWO-022 FW-5 + FW-7 CLOSED (lab follow-ups; Lead-executed lab work, no product code)** — (1) FW-5: `d11-unread-attention.sh` submits fixed in-place to `ctrl+Return` (the D8c/D9 calibration; the original plain-Return would have failed even with a runtime) with an in-file attribution note; the original D11 frames relabeled honestly (02b-chatA-created → 02b-draft-unsubmitted, 03 → 03-no-selection, 04-chatB-created → 04-no-chat-b, 05 → 05-no-unread, 06 → 06-no-selection, 07 → 07-empty-surface — each VLM-read verbatim: drafts never submitted, "No chats", honest statuses; md5 values byte-identical to the original run, git history preserves the old names) and the manifest completed with the omitted `d11-02a`. (2) FW-7: lab Codex CLI runtime restored + pinned — npm `@openai/codex@0.146.0-alpha.3.1-linux-x64` (the exact `docs/platform-support.md` oracle pin), sha256 `ae77c5e73db36d15c131381c5d620278abed999650867a7490b13384e1f5842d`, `/home/z/parity-lab/runtime/codex`, injected via `CODEX_RS_CODEX_BIN`; discovered + negative-controlled requirement: `CODEX_HOME` must point at an existing dir (`CodexHome::resolve` errors + the CLI hard-exits otherwise — run2 log archived). D11r corrected full-flow scene (NUX dismissal + D9 click ladder + ctrl+Return) at main `3c812a7`: footer "App-server online" from frame 01 (EQ-1 drift reversed — the 006-era D9 runtime-observed conditions reproduced); composer creates real thread rows (Tasks-route tasks under Projects, titled from the first message); **dot-on-row** (03/04, "Chat marked unread"), **dot-persists across selection** (04), **Ctrl+Alt+A jump with visit-clear** (05, crop-adjudicated `vlm-crop-0405.json` — the red-X/green-check circles are separate per-row turn-status icons), **re-mark** (06), **clear-all with honest count** (07, "Cleared unread indicators for 1 chat") — all VLM-read verbatim on the real binary; unauthenticated slice D9-identical (thread creation OK, turn execution fails with retry toast). Residual class closed: the lab now HAS the pinned runtime, so no-runtime-lab limitations no longer apply to future scenes. Evidence `docs/research/evidence/wo-p2-008/` (README amended: two-cause causal note + relabel table + D11r section; d11r/ frames + md5 + VLM reads + crops + run3.log + negative-control-run2.log). Parity §5.10 residual cell + evidence citation updated; §10 changelog row added. |

| 2026-09-19 | **WO-P2-012 implemented + CLOSED on merge** (PR #34 → `afa5b9d`; worker r3 `aeccfc9`, base `a664644` exactly; r1 `0547055` + r2 `4fc763a` superseded): guard honesty for the six evidenced silent no-op states — F-A1/A2/A3 per-guard "Select a chat before …" statuses (WO-P2-008 copy family, Action::SetStatus), F-A3 stale-selection status, F-A6 "A Git workflow is already running.", F-D1 /review ReviewSlashCommandAction fall-through to visible submission + honest review-start statuses, F-D2 /compact runtime-loading composer_error guidance with load-recovery; FW-9 executor fall-through tests (review→CreateTask visible submission; compact reports + recovers; SetStatus observability). 8 focused tests green on the CI double matrix; local fmt clean + guarded D15 binary build (full battery CI-carried per the 011 NOT RUN doctrine); D15 GUI evidence. Stream-death git-harvest (r3 pushed 21:11, tab died 21:14; redundant re-dispatches voided at harvest). F-A4 → WO-P2-020 decision (f1-sweep D-3); F-A5 documented-by-design. Evidence `docs/research/evidence/wo-p2-012/`. |
| 2026-09-20 | **WO-P2-013 implemented + CLOSED on merge** (PR #36 → `c10b426`; worker final `20a6018`, base `a664644` exactly, + Lead rustfmt `6d08bf3`; five worker iterations through the stream-death cycle, superseded attempts self-archived): real bounded Activity view on the existing attention primitives (J-17) — `ToggleActivityView`/`CloseActivityView` toggle + idempotent close; rows from `needs_attention_task_ids` in sidebar order (no second store, no semantic changes); jump via the exact `nextUnreadChat` path (visit-clears resolves attention, surface dismisses); full keyboard path (scoped context + wrap selection + bare-Escape precedence, Shift+Esc preserved for clear-all); honest empty state with mark-unread guidance; palette row "Toggle Activity view" (registry copy verbatim, WO-P2-010 contract). 6 focused tests + regression guards green locally; CI double matrix green @ `6d08bf3`; D16 GUI evidence (open/nav/jump/resolve/count/empty-state all VLM-verified; toggle-close byte-identical to the post-jump frame). Harvest saga: 9.5h platform capacity congestion, 10 dispatches, final session husked at 04:00 but pushed 04:27:45 (stream-death ≠ work-death #3); redundant machinery stood down by the Lead (incl. one in-flight 6th create killed mid-send). D-2 row-text fix applied in this closure. Evidence `docs/research/evidence/wo-p2-013/`. |
| 2026-09-20 | **WO-P2-018 implemented + CLOSED on merge** (PR #37 → `b562397ce6325f12d21f67f0549f5860b7801306`; worker delivery `0ece5bb`, single commit on base `8172f6e` exactly, ui.rs-only +241/−15): visible-entry + label parity for icon-only controls — TitleBarDiscoveryEntry chrome buttons (visible palette entry, navigate-back/forward tooltip style + primary chord; Activity bell dispatching the existing ToggleActivityView on the J-17 surface); UNREAD_ATTENTION_DOT_TOOLTIP “Unread activity” (registry vocabulary); ARCHIVED_CHAT_DELETE_LABEL visible Delete + tooltip, Unarchive labels, “Delete all” header (matching the single-scope confirmation title); 4 focused tests + suite 229P/0F locally, CI double matrix green; D18 GUI evidence (entries render; Search opens the Unified palette; Bell opens the Activity view honest empty state; dot tooltip verbatim; archived section visible labels; geo-probe cross-check). §5.10 residual + resolution cells flipped. Evidence `docs/research/evidence/wo-p2-018/`. |
