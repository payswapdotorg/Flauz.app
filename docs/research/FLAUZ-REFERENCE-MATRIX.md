# FLAUZ-REFERENCE-MATRIX — codexRS (Flauz.app) v0.1.0-rc.13

Worker B2 deliverable, Task 41-B2 (parity wave). Branch `parity/flauz-reference-matrix`.

**Product under test:** release binary `codexrs-v0.1.0-rc.13-linux-x86_64` (== `main` `f113515`).
**Lab:** `/home/z/parity-lab/linux-B` — Xvfb `:101` (1600x1000x24) + picom (xrender), lavapipe software
Vulkan, isolated `HOME`/`XDG_*`/`CODEX_HOME`/`CODEX_RS_DATA_DIR`, fork runtime `rust-v0.1.0`
(`CODEX_RS_CODEX_BIN`), xdotool input, ffmpeg x11grab capture, VLM (`z-ai vision`) verification.
**Companion docs:** Worker A `docs/research/CODEX-REFERENCE-MATRIX.md` (official side),
Worker C1 `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` (canonical 12-column skeleton
+ reconciliation procedure), `docs/parity-matrix.md` (historical in-repo record).

**Provenance labels** (per lab rules): `[runtime-observed]` = seen in a lab session (screenshot or
state DB); `[source-derived]` = verified in source at `main f113515` (path:line); `[historical-record]`
= claim in `docs/parity-matrix.md` from prior validation waves; `[docs-derived]` = repo documentation.

**Lab bounds (apply everywhere marked "bound"):**
1. **No authenticated Codex account** — ChatGPT sign-in, API key, and Bedrock flows were not
   credentialled. No thread/turn can be started, so every account-powered surface (streaming
   timeline, approvals, steer/stop live states, terminal/browser panes *inside a chat*) is evidenced
   from UI-shell behavior + source only.
2. **No `gh` CLI in the lab image** — the app's own surfaces report this honestly (banner
   "GitHub CLI (gh) is not installed"); PR/GitHub flows are bound, not absent.
3. **Coordinate-driven GUI automation** — xdotool clicks target measured pixels; two navigation
   targets (full Import page content, opened composer pickers) were not reached and are labeled.

**Evidence files** live in `docs/research/evidence/flauz/` (numbered `NN-*.png`, referenced below as
`ev/NN-*.png`); scene scripts in `docs/research/evidence/flauz/scenes/`. Session-unique frames were
md5-checked; "frame identical to X" statements are byte-identical captures.

---

## 1. Agent / task lifecycle

| Capability | Backend (source path/symbol) | Runtime (verified how) | GUI (surface) | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| New chat | `codex-app/src/ui.rs:4465-4466` `NewChatShortcut` (Ctrl+N/Ctrl+Shift+O); palette `NewChat` | Ctrl+N produced the chat entry view in session b2g | Sidebar "+ New chat" row; welcome card "What should we work on?" | **Persistent affordance** (sidebar) + shortcut + palette | ev/01, ev/19 `[runtime-observed]` | "New standalone chat" variant Ctrl+Alt+O (palette: "Start a new chat outside of any project") `[runtime-observed in palette, ev/08]` |
| Resume / history | `thread/loaded/list` + `thread/read` hydration (historical-record); sidebar Chats list | Sidebar "Chats" section renders with search + refresh icons, empty state "No chats" | Sidebar Chats list | Persistent affordance (sidebar) + Ctrl+G search | ev/06 `[runtime-observed empty state]`; `[historical-record]` | bound: no chats can be created unauthenticated |
| Streaming timeline | GFM markdown timeline, turn events, summaries (`parity-matrix.md` Streaming timeline row) | not evidenced — bound (no auth → no turns) | Chat timeline | — | `[historical-record]` | prior waves validated against pinned app-server |
| Stop | `ui.rs:23363-23382` `stop-turn` button → `Action::InterruptActiveTurn` | not evidenced — bound (no active turn possible) | Composer circular button (tooltip "Stop", appears when turn active & composer empty) | contextual (composer) | `[source-derived]` | |
| Steer | `ui.rs:23383-23396` send button tooltip switches Send/Steer/"Run shell command" | not evidenced — bound | Composer send button | contextual (composer) | `[source-derived]` | |
| Retry | `ui.rs:13861-13866` "Retry connection" → `Action::RetryConnection`; safety-buffer retry `Action::SafetyBufferedRetryFailed` (ui.rs:7043); reconnect timer 1/2/4/8/16/20s `[historical-record]` | /bin/false session: app stays up, runtime status stuck at "Resolving…" (auto-retry, no crash) | Footer runtime status + retry button | persistent-ish (footer status) | ev/24 `[runtime-observed]` | graceful degradation confirmed |
| Fork | `/fork` slash + `forkThread` picker (ui.rs:2746, 7558-7560); `PendingWorktreeFork` (core `lib.rs:2162`) | not evidenced — bound | Slash command + fork picker | slash-only | `[source-derived]` | worktree-fork flow per historical-record |
| Edit latest message | `thread/rollback` + replacement `turn/start` `[historical-record]` | not evidenced — bound | stable `Edit` action / double-click | hidden (in-turn) | `[historical-record]` | |
| Compact | `Action::CompactThread` (core `lib.rs:6493`); guards "Compact requires an empty composer" (12739), "disabled while a chat is in progress" (12753) | not evidenced — bound | `/compact` slash command | slash-only | `[source-derived]` | renders `contextCompaction` items + token percentage `[historical-record]` |
| Background execution | `BackgroundTerminal` registry (core `lib.rs:4160, 5119`); Process Manager palette command Ctrl+Alt+M (ui.rs:3310) | not evidenced — bound | Process Manager ("View and manage processes started by Codex chats") | palette-only (+shortcut) | `[source-derived]` | |
| Task search | palette `SearchChats` Ctrl+G "Search past chats and their content" (ui.rs:3297, 3410) | palette entry observed; search itself bound (no chats) | Palette + Ctrl+G | palette + shortcut | ev/08 `[runtime-observed palette entry]` | bounded full-text search with snippets `[historical-record]` |
| Memory | `ResetMemories` modal (ui.rs:4526-4530, 5118); `ChatMemoryPreferences`; Personalization settings page; `memories_1.sqlite` created in CODEX_HOME | memory DB file created by runtime `[runtime-observed filesystem]`; UI flows bound | Personalization settings; `/memories` slash | settings page (persistent) + slash | `[runtime-observed DB]`, `[source-derived]` | `codex/memories_1.sqlite` present after sessions |
| Reconnect | app-server supervision reconnect (historical-record); `RetryConnection` (ui.rs:13866) | /bin/false session kept retrying silently ("Resolving…" persists 6+s) | footer "App-server online/…" status | footer status (persistent) | ev/24, ev/06 `[runtime-observed]` | |
| Recovery / restart restore | `ui.rs:4555-4569` window placement restore; `ui_preferences` table (route, inspector, placement) in `CODEX_RS_DATA_DIR/state.sqlite3` | restart after resize + navigation: window restored to 1100x900, route restored to Settings (DB row verified pre- and post-restart) | automatic | n/a (automatic) | ev/22 `[runtime-observed]`; DB dump in doc below | `primary_window_placement_v1={"x":161,"y":91,"width":1100,"height":900}` persisted |

**State DB runtime dump** (`data/codexrs/state.sqlite3`, table `ui_preferences`): `route='settings'`,
`inspector='hidden'`, `primary_window_placement_v1` as above; tables `recent_workspaces`,
`browser_downloads` exist (empty). `[runtime-observed]`

## 2. Composer

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| Send | `ui.rs:23383-23396` send button → `submit`; editable `composer.submit` (Enter) | typed "hello" + Enter on new chat: draft retained, no thread, sign-in card remains → send blocked unauthenticated | circular send button | persistent affordance (composer) | ev/21 `[runtime-observed]` | honest auth bound |
| Steer / Stop | see §1 | bound | composer button (state-dependent) | contextual | `[source-derived]` | |
| Plan | `/plan` → `toggle_composer_plan_mode` (ui.rs:7619, 8150); "Plan mode" checked popup item (10893) | slash menu captured (8 visible items); /plan in full set per source | composer popup menu + slash | slash + composer menu | ev/20 `[runtime-observed menu]`, `[source-derived]` | menu is viewport-bounded/scrollable — visible page showed MCP, Work in a project, Fast, Status, Image Gen, OpenAI Docs, Plugin Creator, Review Agent |
| Goal | `goal_input` (ui.rs:5001, 5227, 10832); `/goal` opens goal editor; `GoalAttachment*` actions (7042) | bound | goal editor overlay | slash-only | `[source-derived]` | |
| Model picker | `composer.openModelPicker` Ctrl+Shift+M (ui.rs:3039-3044); `ModelPickerItem` (1638-1701) | composer bar shows "GPT-6-Astra" selector `[runtime-observed]`; opened-picker state not captured (coordinate bound) | composer bottom-bar selector | persistent affordance (composer bar) + shortcut | ev/01, ev/19 `[runtime-observed]` | searchable picker, catalog-backed |
| Reasoning effort | `ReasoningEffortStep` (ui.rs:2584); editable increase/decrease/cycle (3081-3095); `/reasoning` | composer bar shows "Low" selector `[runtime-observed]` | composer bottom-bar selector | persistent affordance + shortcut | ev/01, ev/19 | |
| Speed tier | `service_tier_picker` "Speed" (ui.rs:23360); `/fast` + service-tier slash rows (7538) | composer bar shows "Standard" `[runtime-observed]` | composer bottom-bar selector | persistent affordance + slash | ev/01, ev/19 | Standard/Fast/Ultrafast |
| Slash commands | `execute_composer_slash_command` (ui.rs:7510-7621): /init /chat /compact /feedback /fork /goal /mcp /memories /model /new /plan /project /reasoning /status /shell (+catalog plugin commands) | `/` in composer → popup menu rendered with catalog entries | composer popup | slash-only (no button) | ev/20 `[runtime-observed]` | 15 native commands `[historical-record]` + plugin-contributed (Image Gen, OpenAI Docs, Plugin Creator, Review Agent visible = marketplace plugin skills) |
| @ mentions | `ComposerCatalogMention` (ui.rs:4931-4942); kinds Mention/LocalImage/Skill/App/Plugin (22852-22856); fuzzy file search lifecycle | `@` typed on new chat: **no popup** (sign-in card visible; no project, unauthenticated catalog empty) | composer popup | hidden (typed trigger) | b2g-11 frame `[runtime-observed]` (not curated; described) | Plugins → Desktop apps → Apps → Skills → Files order `[historical-record]` |
| Attachments | "+" menu → Image / File / Scan / Paste from clipboard / Take screenshot | "+" click opened the menu with exactly those 5 items | composer "+" button | persistent affordance (composer) | ev/16 `[runtime-observed]` | `Action::AddComposerAttachments` (ui.rs:9147, 9180, 22119); approval dropdown "Ask for approval" beside it (ev/01) |

## 3. Terminal

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| PTY / execution | `codex-platform/src/terminal.rs` — `portable_pty` (line 19), `openpty` (178), `write` (253), `resize` (260), shells (61) | not evidenced this wave — bound (see below); native PTY validated in prior waves `[historical-record]` | terminal dock content | — | `[source-derived]`, `[historical-record]` | |
| Dock / tabs | `TerminalState` + per-task tabs (core `lib.rs:4754`, `tabs_for`), `MAX_TERMINAL_TABS`, `spawn_terminal_tab` (8157), dock location Bottom/Right (`TerminalDockLocation`, `SetTerminalDockLocation`) | **Ctrl+` on a new (draft) chat is a silent no-op** — capture after toggle is byte-identical (md5) to the pre-toggle chat view | bottom dock (default) or right panel | ❌ **shortcut-only** (Ctrl+`, palette "Open terminal", Ctrl+J "Toggle bottom panel") — **no persistent toolbar/affordance anywhere** | ev/25 vs ev/19 (identical frames) `[runtime-observed]`; guard "Select a task before opening a terminal." core `lib.rs:8163` `[source-derived]` | dock opens but spawns no tab without a selected task → renders nothing; on the app's entry surface the terminal is undiscoverable AND inert |
| Resize | `terminal.rs:260` resize(rows, cols) | bound | dock splitter | — | `[source-derived]` | dock size persisted (`terminal_bottom_height`/`terminal_right_width`, core 4866-4867) |
| Stop / restart | `Action::StopTerminal` (ui.rs:25078 "Stop" terminal action) | bound | terminal tab controls | — | `[source-derived]` | |
| Settings | "Default terminal location: Bottom / Right" toggle on Settings > General | observed on General page | Settings > General | settings page (persistent) | ev/07 `[runtime-observed]` | |
| Persistence | dock location + sizes persisted via `Effect::PersistTerminalDockLocation` | DB schema `[runtime-observed]` | automatic | n/a | `[source-derived]` | |

**KNOWN FINDING 1 — Terminal discoverability: PASS (confirmed and strengthened).**
Backend ✅ (`terminal.rs` real PTY) · Runtime ✅ (dock state machine + per-task tabs; prior-wave PTY
validation `[historical-record]`; this wave's runtime observation = inert no-op on draft chats) ·
GUI ✅ (dock + Bottom/Right setting + per-chat tabs) · **Discoverability ❌** — Ctrl+` binding at
`ui.rs:4499` (`KeyBinding::new("ctrl-`", ToggleTerminalShortcut, None)`), `Action::ToggleTerminalDock`
at core `lib.rs:4923`, palette entry "Open terminal" (Ctrl+`) at `ui.rs:3322/3411`. No persistent
affordance; on the new-chat entry surface the toggle is a silent no-op (task guard `lib.rs:8163`).
New evidence this wave: the no-op is *chat-scoped* — even keyboard/palette paths do nothing before a
task exists.

## 4. Browser

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| Engine / session | `codex-platform/src/browser.rs` — `BrowserSession::spawn` (460), `navigate`/`back`/`forward`/`reload`/`stop` (535-562), `BrowserTab`, `BrowserKeyInput` | bound — Ctrl+T on a draft chat is a **silent no-op** (guard below); no chat can exist unauthenticated | right inspector panel (`InspectorPane::Browser`, core `lib.rs:259-267`) | ❌ **palette/shortcut-only** — Ctrl+T / Ctrl+Shift+B / Ctrl+L (`ui.rs:3412-3414`), palette entries (3323-3325); no persistent affordance | b2g-06 frame identical to chat view `[runtime-observed]`; guard "Open a chat before opening the Browser." core `lib.rs:14521-14525` `[source-derived]` | |
| Address bar | `FocusBrowserAddressBar` Ctrl+L (ui.rs:3414) | bound | browser panel toolbar | shortcut + panel | `[source-derived]` | |
| Navigation history | back/forward buttons + Alt+Left/Alt+Right (shortcuts overlay) | overlay lists them `[runtime-observed ev/15]` | browser panel | in-panel | ev/15 | |
| Downloads | `BrowserDownload` (browser.rs:199); `BrowserDownloadsState` + preferences (core 1729-1780); `browser_downloads` state table; `Action::OpenBrowserDownload` (core 5628) | `browser_downloads` table exists in state DB `[runtime-observed schema]` | download management UI | in-panel | `[runtime-observed DB]`, `[source-derived]` | |
| Permissions | `BrowserSitePermission`; `with_permissions` (browser.rs:142); AllowAllBrowserSites modal (ui.rs:4542-4550) | bound | browser panel + settings | — | `[source-derived]` | |
| Save As | `with_prompt_for_user_downloads` (browser.rs:136) | bound | download prompt | — | `[source-derived]` | |

**KNOWN FINDING 2 — Browser discoverability: PASS (confirmed).** Backend ✅ (`browser.rs` full
session/navigation/download API) · Runtime ✅ (session spawn path + persisted downloads store;
panel runtime bound to chat-scoped guard) · GUI ✅ (`InspectorPane::Browser` panel, core
`lib.rs:259-267`) · **Discoverability ❌** — palette/shortcut-only (Ctrl+T/Ctrl+Shift+B/Ctrl+L),
plus new evidence: `OpenBrowserTab` is refused without an open chat ("Open a chat before opening
the Browser." toast, `lib.rs:14521`).

## 5. Computer use

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| Platform gate | `codex-platform/src/computer_use.rs:34-57` — "Linux observation is intentionally limited to X11/XWayland… Pure Wayland requires the separate portal path"; `computer_use_platform_available()` | Settings page renders platform-honest copy | — | — | `[source-derived]` | `docs/platform-support.md`: Linux = "X11/XWayland screenshot observation when DISPLAY is set" |
| Observation (Linux) | `capture_computer_window` (computer_use.rs:488+) | Settings > Computer use: card "Any App — Let ChatGPT observe screenshots of X11/Wayland apps — **Enabled**" (green) | Settings > Integrations > Computer use | settings page (persistent) + palette ("Computer use" settings; "Open Computer Use" inspector command ui.rs:3326) | ev/14 `[runtime-observed]` | runtime copy names the exact platform capability |
| Interaction APIs | move/click/drag/scroll/type/press (computer_use.rs:515-738; macOS impls live, Linux impls screenshot-only) | bound (no account → no agent driving) | inspector pane | palette | `[source-derived]` | |
| App discovery | `computer_apps.rs`; app launch/interruption `computer_interruption.rs`; overlays `computer_overlay.rs` | bound | inspector + overlays | — | `[source-derived]` | |
| Approvals | computer-use approvals per parity-matrix row | bound | approval cards | — | `[historical-record]` | |

**KNOWN FINDING 3 — Linux Computer Use is a platform gap, not a missing feature: PASS (confirmed).**
`computer_use.rs` implements the full interaction API surface with per-platform impls; Linux impl
is screenshot-observation over X11/XWayland by design, the in-app settings copy states exactly that
(ev/14), and `docs/platform-support.md` publishes the same. Correct classification:
**platform-limited** (matches the historical matrix's separate "Linux: platform" row).

## 6. Files / workspace

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| Open folder | palette `OpenFolder` Ctrl+O "Add a local project to Codex" (ui.rs:3296, 3409); native Select Project Root picker `[historical-record]` | palette suggestion observed with shortcut | native directory picker | palette + shortcut + sidebar "Projects +" | ev/08 `[runtime-observed]` | |
| Project registry | 64-entry local-project SQLite registry + `recent_workspaces` table `[historical-record]` | `recent_workspaces` table present `[runtime-observed schema]` | sidebar Projects section ("No projects" empty state) | persistent affordance (sidebar) | ev/06 | project rows: select/new chat/rename/pin/remove `[historical-record]` |
| Explorer | `InspectorPane::Files` (core lib.rs InspectorPane); Files inspector actions (core 9015-9176) | bound | right inspector | inspector (chat-scoped) | `[source-derived]` | |
| Preview / outputs | `InspectorPane::Outputs` (core 9085); Outputs viewer for generated images `[historical-record]` | bound | right inspector | inspector (chat-scoped) | `[source-derived]` | |
| Attachments | see §2 attachments row | "+" menu observed | composer | persistent affordance | ev/16 | file/folder/image/screenshot/clipboard |
| Citations | file citation chips in timeline `[historical-record]` | bound | timeline | — | `[historical-record]` | |

## 7. Git

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| Repository overview | `codex-platform/src/git.rs`; Repository route | sidebar "Repository" renders: stats row (Changes/Staged/Branches/Worktrees), "No Git repository detected." honest empty state | dedicated sidebar page | **persistent affordance (sidebar)** | ev/02 `[runtime-observed]` | |
| Staged / unstaged | Changed Files section with Stage/Unstage; Uncommitted changes surface `[historical-record]` | section headers observed | Repository page + Changes inspector | sidebar page + inspector | ev/02 | |
| Branches | `CreateBranch` palette command (ui.rs:3315); branch switching/create-checkout with conflict surfacing `[historical-record]` | "BRANCHES" section + "Create branch" link observed | Repository page | sidebar page + palette | ev/02 | |
| Worktrees | New Worktree form (branch name + path + helper text); worktree fork flow | "WORKTREES" section + form observed | Repository page | sidebar page | ev/02 | |
| Diff review | "BOUNDED DIFF REVIEW" panel, Unified/Split toggle; `gitDiffToRemote` default target `[historical-record]` | panel + toggle observed | Repository page right pane | in-page | ev/02 | |
| Commit / push | `CommitAndPush` (core 3167); commit dialog with generated message `[historical-record]` | "Commit or push" button observed on Repository page | Repository page + palette | sidebar page + palette | ev/02 | push via supervised `git push --porcelain` |
| Force push | `always_force_push` preference (core 3346, 18310, 28185) | bound | Git settings | settings | `[source-derived]` | |
| Pull requests | `codex-platform/src/github.rs` — **gh CLI is the transport** (CliMissing → "GitHub CLI (gh) is not installed", line 342) | PR page renders fully (tabs Reviewing/Review requested/Previously reviewed/Authored/All; Open/Merged/Closed/All states; search) then honest bound: banner + "Install GitHub CLI" / "Check again" buttons | dedicated sidebar page | **persistent affordance (sidebar)** | ev/03 `[runtime-observed]` | bound: lab has no gh binary; page degrades honestly |
| PR create / merge | palette `Create PR` / `Create draft PR` / `Merge PR` / `Open PR on GitHub` (ui.rs:3313-3317) | palette entries source-verified; flows bound (gh) | palette | palette | `[source-derived]` | |
| Review | `OpenReviewTab` Ctrl+Shift+G, `ToggleReviewPanel` Ctrl+Alt+B, `Enable/DisableGitReview` (ui.rs:3318-3321, 3417-3418) | bound | review tab + panel | palette + shortcut | `[source-derived]` | |

## 8. MCP / Apps / Skills / Plugins / Workflows

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| Plugins marketplace | marketplace repo sync to `CODEX_HOME/.tmp/plugins` (README + `plugins/` dir with 64 curated plugins: figma, notion, adobe, …) — **synced by the runtime in-lab** `[runtime-observed filesystem]` | Plugins page: tabs Plugins/Skills; filter chips "Curated by OpenAI / Shared with you / Created by me"; search; Create dropdown; empty state "No plugins found — No plugin marketplaces returned entries for this host" | dedicated sidebar page | **persistent affordance (sidebar)** | ev/04 `[runtime-observed]` | UI-empty vs disk-synced: catalog listing appears account/gh-bound — documented bound, not fabricated |
| Skills | `OpenSkills` / `ForceReloadSkills` palette (ui.rs:3327-3328); `skills/` dir created in CODEX_HOME `[runtime-observed filesystem]`; skill slash selection `select_composer_skill_slash` (ui.rs:7550) | palette entries source-verified; Skills tab observed on Plugins page | Plugins page tab + palette | sidebar tab + palette | ev/04, `[source-derived]` | |
| MCP | Settings > MCP servers; `EditMcpServer`/`InspectMcpServer` (ui.rs:2193-2197); `/mcp` submenu (7576); `McpElicitation` modal (4504) | settings nav row observed (ev/07/23); servers list bound (none configured) | Settings page + slash | settings (persistent) + slash | ev/07, ev/23 `[runtime-observed nav]` | |
| Apps | `render_apps_catalog` (ui.rs:30399); `MarketplaceManageTab::Apps` (33187); `app://` mentions (22852-22856) | bound | Plugins management tabs | in-page tab | `[source-derived]` | |
| Workflows | sidebar Workflows route; teach/publish/durable-instance surfaces | full page observed: "Teach a workflow" (Demonstrate/Instruct/Hybrid modes), "Published versions", "Durable instances" (loaded, empty states) | dedicated sidebar page | **persistent affordance (sidebar)** | ev/05 `[runtime-observed]` | FROZEN pack/workflow area — surfaced but not expanded per mission freeze |

## 9. Settings / account

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| Settings registry | `SettingsSection` enum (ui.rs:2398-2417): 18 sections | nav renders 15 rows in 3 groups — Personal (General, Appearance, Personalization, Keyboard shortcuts, Profile, Usage & billing, Import), Integrations (Plugins, MCP servers, Browser, Computer use, Connections), Coding (Configuration, Hooks, Git) | full-window settings view with "Back to app" | **persistent affordance** (sidebar Settings) + Ctrl+, + palette | ev/07 `[runtime-observed]` | CodeReview/Worktrees/ArchivedChats exist in enum but are not in the default nav (contextual/hidden) |
| Settings search | `settings_section_matches` (ui.rs:45519+) with per-section keyword sets | typed "import" → nav filtered to Personal + Import only | nav search box | in-view | ev/23 `[runtime-observed]` | |
| Auth / sign-in | sign-in card: Sign in with ChatGPT / Use device code / Use API key / Use Amazon Bedrock (Experimental) | card rendered on chat entry | welcome card | persistent affordance (entry surface) | ev/01, ev/21 `[runtime-observed]` | bound: no credentials in lab; Bedrock credential save restarts managed runtime (ui.rs:4860) |
| Account / usage | Usage & billing page | "Sign in to view usage and billing — Connect your OpenAI account to load its usage limits." + "Go to Profile" | Settings page | settings nav | ev/12 `[runtime-observed]` | account-powered → bound |
| Profile | `SettingsSection::Profile` (search keywords: account, plan, api key, sign in…) | nav row observed | Settings page | settings nav | ev/07 | bound |
| Keyboard shortcuts | Keyboard shortcuts settings page + editable command registry (searchFiles, composer.* set) + reset modal (4532-4540) | page observed with search + "Keys" filter + shortcut table | Settings page + Ctrl+/ overlay | settings + shortcut | ev/11, ev/15 `[runtime-observed]` | |
| Notifications | `codex-platform/src/desktop_notifications.rs`; toast banners | gh-missing toast banner observed on multiple pages | toast banners | passive | ev/03/04 `[runtime-observed]` | |
| Theme / appearance | `SetAppearanceTheme` → `PersistAppearanceTheme` (core reduce); theme cards System/Light/Dark + accent/background/foreground editors (ui.rs:32080-32130) | Appearance page observed with theme cards + code preview + color editors; app rendered Light (System default on bare Xvfb) | Settings page | settings nav | ev/10 `[runtime-observed]` | theme persists via effects (no ui_preferences row until changed) |
| Import / migration | `render_import_settings` (ui.rs:34696+); `ImportProvider::{ClaudeCode, ClaudeCowork, Cursor}` (core 3478-3485); statuses "Checking for imports" / "Found setup from <providers>" / "Review and import" | Import nav row + search filter observed; **full page content not captured** (coordinate-target miss across 3 attempts) — page + provider set source-verified; fabricated `~/.claude/projects/flauz-demo-2026` session present in lab HOME for detection | Settings page | settings nav | ev/23 `[runtime-observed nav]`, `[source-derived]` | honest bound: not evidenced (bound: coordinate navigation) |
| Connections / remote | Settings > Connections | "Remote control — This computer (Disabled)" + "Enable remote control"; "Paired devices — Enable remote control to pair and manage devices." (Pair a device disabled) | Settings page | settings nav | ev/13 `[runtime-observed]` | |
| Storage / reset | "codexRS state" section on General shows state DB path | observed | Settings > General | settings | ev/07 | reset modals exist (ResetKeyboardShortcuts/ResetMemories/RemoveLocalProject/DeleteArchivedTasks — ui.rs:4506-4551) |

## 10. Product shell

| Capability | Backend | Runtime | GUI | Discoverability | Evidence | Notes |
|---|---|---|---|---|---|---|
| Sidebar | WorkspaceView sidebar | New chat / Repository / Pull requests / Plugins / Workflows / Projects / Chats / Settings + footer "App-server online" (green dot) | left sidebar (275px class) | persistent | ev/06 `[runtime-observed]` | Ctrl+B toggles sidebar (binding ui.rs:4495; observed no-op on settings view — settings replaces the app sidebar with the settings nav) |
| Window / title bar | window options + placement (ui.rs:4553-4578) | custom title bar with Back/Forward, File/Edit/View/Help menus, window controls; min size enforced (`WINDOW_MIN_*`), placement persisted | title bar | persistent | ev/06, ev/22 | traffic-light style, transparent titlebar |
| Command palette | `OpenCommandMenu` Ctrl+K/Ctrl+Shift+P (ui.rs:4460-4461); `PaletteCommand` registry (3294-3392) with 40+ commands, groups (Suggested/Settings/Thread/Navigation/Skills…) | palette overlay rendered: "Search chats or run a command", suggestions New chat (Ctrl+N), Open folder (Ctrl+O), settings links, New standalone chat, Search chats; search "appearance" → single result "Appearance (Open Appearance settings)" → navigates | modal overlay | shortcut (Ctrl+K) — no toolbar button | ev/08, ev/09 `[runtime-observed]` | palette does NOT index all settings pages: "import" → "No matches" (ev/18) while the Import page exists in settings nav — discoverability gap |
| Inspector panes | `InspectorPane::{Hidden,Changes,Outputs,Files,Terminal,ComputerUse,Browser}` (core lib.rs:259-267) | inspector state persisted 'hidden' (DB) | right side panel | chat-scoped | `[runtime-observed DB]`, `[source-derived]` | |
| Terminal dock placement | `TerminalDockLocation::{Bottom,Right}` + General setting | Bottom/Right toggle observed | Settings > General + dock | settings | ev/07 | |
| Responsive | width classes (ShellWidthClass, ui.rs:241-260) | 640x700 window on the settings view: nav column stays expanded (no auto-collapse observed at that width on that view) | — | — | ev/17 `[runtime-observed]` | chat-view breakpoints (960/720 contract `[historical-record]`) not re-verified this wave |
| Empty / loading / error states | honest empty states across pages | "No Git repository detected." / "No plugins found" / "No chats" / "No projects" / gh banner / runtime "Resolving…" all observed | per-page | n/a | ev/02/03/04/06/24 `[runtime-observed]` | |
| Keyboard shortcuts overlay | `ShowKeyboardShortcutsShortcut` Ctrl+/ (ui.rs:4501) | overlay rendered with categorized list (Chat, Navigation, …) | modal overlay | shortcut (Ctrl+/) + settings page | ev/15 `[runtime-observed]` | |
| Process manager | palette Ctrl+Alt+M | bound | window | palette | `[source-derived]` | |
| Accessibility | pointer cursors preference (ui.rs:427-498), focus handles for modals | not evidenced — bound | Settings | — | `[source-derived]` | |

---

## Journey results (J1–J6)

**J1 — coding journey.** WORKS: app boot to entry surface with composer + pickers (ev/01, ev/19);
palette "Open folder" (Ctrl+O) surfaced with native picker path `[historical-record]`; settings,
appearance, shortcuts pages all navigable. BOUND: starting any coding turn requires an authenticated
account — send is blocked (ev/21: draft retained, sign-in card), so the whole in-chat journey
(streaming, approvals, file edits, review) is account-bound. GAP: none beyond auth.

**J2 — browser-assisted journey.** BOUND (entirely): the browser panel requires an open chat
("Open a chat before opening the Browser.", core `lib.rs:14521`); Ctrl+T on the entry surface is a
silent no-op `[runtime-observed]`. No chat possible without auth → navigation/downloads/permissions
flows are source-verified only (`browser.rs`). GAP: none beyond auth; discoverability flagged in
Finding 2.

**J3 — planning journey.** WORKS: composer surfaces (Plan via `/plan` + popup menu item, Goal
editor via `/goal`, model/effort/speed selectors visible). BOUND: plan/goal execution requires a
thread. GAP: none beyond auth.

**J4 — git journey.** WORKS: Repository page with stats, changed files, branches, worktree form,
bounded diff review (ev/02); "Commit or push" affordance. BOUND: PR list/creation/merge needs the
`gh` CLI (not installed in lab); the app reports this honestly with an actionable banner (ev/03).
Commit/push against a real repo was validated in prior waves `[historical-record]`; a demo git repo
(`demo-project` with staged/unstaged files) exists in the lab HOME for future runs. GAP: none.

**J5 — recovery journey.** WORKS: restart persistence verified end-to-end — window placement
(1100x900) and last route restored after a full app restart (ev/22 + state DB `ui_preferences`);
inspector state persisted; graceful no-runtime degradation (app stays up, runtime status
"Resolving…", auto-reconnect timer per historical record) (ev/24). BOUND: mid-turn crash recovery
(reconnect to an active turn) needs an account. GAP: none.

**J6 — extensibility journey.** WORKS: Plugins/Skills pages render with filters and search (ev/04);
the marketplace repository itself synced to `CODEX_HOME/.tmp/plugins` with 64 curated plugins
`[runtime-observed filesystem]`; Skills dir created; MCP settings nav + `/mcp` slash; Workflows
page (ev/05). BOUND: catalog listing in the UI is empty unauthenticated ("No plugin marketplaces
returned entries for this host") — install/enable flows not evidenced. GAP: UI-empty vs
disk-synced marketplace is flagged for C2 reconciliation (likely auth/gh-bound, not a defect).

---

## Keyboard-shortcut inventory

Source: key registry `crates/codex-app/src/ui.rs:4459-4552` (Linux = Ctrl prefix; macOS = Cmd),
palette hint strings `ui.rs:3396-3414`, editable composer commands `ui.rs:3030-3100`,
`docs/parity-matrix.md` (approval commands). "Vis" = visually confirmed this wave.

| Shortcut (Linux) | Action | Source anchor | Vis |
|---|---|---|---|
| Ctrl+K / Ctrl+Shift+P | Command palette | ui.rs:4460-4461 | ✅ ev/08 |
| Ctrl+N / Ctrl+Shift+O | New chat | 4465-4466 | ✅ (Ctrl+N used in b2g) |
| Ctrl+Alt+O | New standalone chat | palette listing | ✅ ev/08 |
| Ctrl+G | Search chats | 4462 | ✅ palette listing |
| Ctrl+P | Search files | 4463 | — |
| Ctrl+O | Open folder | 4464 | ✅ palette listing ev/08 |
| Ctrl+W / Ctrl+Q | Close window / Quit | 4467-4468 | — |
| Ctrl+Shift+A | Archive chat | 4469 | ✅ overlay ev/15 |
| Ctrl+Alt+R | Rename chat | 4470 | — |
| Ctrl+Alt+P | Toggle pin | 4471 | ✅ overlay |
| Ctrl+[ / Ctrl+] | Back / Forward | 4472-4473 | ✅ overlay |
| Ctrl+Shift+[ / Ctrl+PageUp | Previous chat | 4474-4483 | ✅ overlay |
| Ctrl+Shift+] / Ctrl+PageDown | Next chat | 4484-4493 | ✅ overlay |
| Ctrl+F | Find in thread | 4494 | ✅ overlay |
| Ctrl+B | Toggle sidebar | 4495 | runtime: no-op on settings view (app sidebar absent there) |
| Ctrl+J | Toggle bottom panel | 4496 | — |
| Ctrl+Alt+B | Toggle Review panel | 4497 | — |
| Ctrl+Shift+G | Open review tab | 4498 | — |
| **Ctrl+`** | **Toggle terminal** | **4499** | ✅ exercised — silent no-op on draft chat (ev/25) |
| Ctrl+, | Open settings | 4500 | — (settings reached via sidebar click) |
| Ctrl+/ | Keyboard shortcuts overlay | 4501 | ✅ ev/15 |
| F11 | Toggle fullscreen | 4502 | — |
| Ctrl+T | Open browser tab | palette 3412 | ✅ exercised — silent no-op on draft chat |
| Ctrl+Shift+B | Toggle browser panel | palette 3413 | ✅ exercised (no-op) |
| Ctrl+L | Focus browser address bar | palette 3414 | ✅ overlay listing |
| Alt+Left / Alt+Right | Browser back / forward | overlay (ev/15) | ✅ overlay |
| Ctrl+Alt+M | Process Manager | palette 3310 | — |
| Ctrl+Shift+M | Open model picker | composer 3039-3044 | — |
| Ctrl+Alt+Shift+O | Open composer project picker | composer 3047 region | — |
| Enter | Send message (`composer.submit`) | composer 3053 region | ✅ exercised (blocked unauth, ev/21) |
| (composer cmds) | Add photos; Attach files and folders; Toggle Fast mode; Increase/Decrease/Cycle reasoning effort | ui.rs:3060-3100 | controls visible ev/01 |
| Enter / Escape | approval.approve / approval.decline | parity-matrix record | — |
| Escape | close About/McpElicitation/StructuredUserInput modals | 4503-4505 | — |
| Tab / Shift+Tab | focus cycling in destructive modals (RemoveLocalProject, DeleteArchivedTasks, ResetMemories, ResetKeyboardShortcuts, AllowAllBrowserSites) | 4506-4551 | — |

The in-app **Keyboard shortcuts settings page** (ev/11) exposes the same registry as a searchable,
editable table with a Keys filter; the Ctrl+/ overlay (ev/15) shows the categorized quick reference
(Chat / Navigation / …). Both were visually confirmed.

---

## Corrected parity counts (audit of `docs/parity-matrix.md`)

Historical claim: **4 done / 29 partial / 7 missing (+1 platform)** across 41 capability rows.

Audited sample this wave (13 rows, ~32%): Runtime bootstrap (done ✅ confirmed — boots, app-server
online footer), Feedback (done ✅ palette entry exists; click-through not exercised), Active keyboard
shortcut reference (done ✅ confirmed — overlay + editable settings page), Terminal (partial ✅ +
discoverability flag), In-app browser (partial ✅ + discoverability flag), Computer Use (partial ✅
cross-platform; Linux cell = platform-limited observation), Repository status / Branches & worktrees /
Diff review (partial ✅ — page fully renders, flows prior-validated), Pull requests (partial ✅ —
honest gh-bound), Plugins marketplace / Skills (partial ✅ — sync works, catalog listing auth-bound),
MCP and apps (partial ✅), Settings (partial ✅ — 15/18 sections in nav), Import and migration
(partial ✅ — page + 3 providers source-verified, page-level runtime capture missed), Scheduled
tasks (missing ✅ — no surface in UI or source beyond plugin includes). Sites / Visualizations /
Appshots / Cloud environments / Voice / Pets: source sweep found no UI surfaces (missing ✅).

**Corrected counts: 4 complete / 29 partial / 7 missing / 1 platform-limited — UNCHANGED.**
The audited sample produced zero status flips; the historical record is accurate for what it claims.
What this wave *adds* is the dimension the historical matrix lacks: **discoverability annotations**
(persistent-affordance vs shortcut/palette-only vs hidden) and explicit **auth/gh bound labels** on
runtime-unevidencable cells — plus three sharpened findings: (a) terminal and browser toggles are
chat-scoped silent no-ops on the entry surface; (b) the command palette does not index all settings
pages ("import" → No matches while the Import page exists in settings nav); (c) the plugins
marketplace syncs its repository to disk while the catalog UI reports empty unauthenticated. These
flow to C2 as reconciliation inputs and to the P1 work-order drafts (WO-P1-DRAFT-001/002) as
strengthened evidence, not as count changes.

---

## Session log (this wave)

| Session | Scene | Runtime | Result |
|---|---|---|---|
| 1 | `b2-gaps.sh` | fork rust-v0.1.0 | 18 captures; terminal/browser no-op proof on chat view; slash menu; send-block; settings nav; mutation for persistence |
| 2 | `b4b-restore.sh` | fork rust-v0.1.0 | 4 captures; restart restore verified (window 1100x900 + route); settings search filter verified |
| 3 | `b3-error-b2.sh` | **/bin/false** | 6 captures; no-runtime graceful degradation ("Resolving…"); Import navigation attempts (not reached) |

Worker B's prior sessions (b1 nav/settings sweep, b2 composer/panels, b5a import attempt) are
curated alongside; their scenes are in `evidence/flauz/scenes/`.
