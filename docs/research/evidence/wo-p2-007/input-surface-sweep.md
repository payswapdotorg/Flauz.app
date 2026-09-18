# WO-R-SWEEP — Input-Surface Integrity Sweep (evidence)

- **Base SHA:** `3c9f113851686e1774ce355ad744b56d6c937b2a` (verified: commit exists and is an ancestor of `origin/main`; `origin/main` HEAD at clone time was `0eb178380e76039b3a7060657fad72c636aa5079`)
- **Branch:** `research/input-surface-sweep` (documents-only; no product code touched)
- **Method:** static source analysis. Anchors are `file:line` at the base SHA. Files swept: `crates/codex-app/src/ui.rs` (51,053 lines), `crates/codex-core/src/lib.rs` (37,232 lines); cross-checks in `crates/codex-app/src/backend.rs`, `third_party/gpui-component/src/menu/popup_menu.rs`, and the `gpui` 0.2.2 crate source (crates.io tarball).
- **Scope:** A. key registry (`bind_keys`), B. command registry (`KEYBOARD_SHORTCUT_COMMAND_IDS`), C. palette (`PaletteCommand::ALL`), D. slash commands (executor + catalog), E. settings nav (`SettingsSection`), F. shortcut overlay/settings honesty.

## Architecture facts established first (needed to read every verdict)

1. The app has **two parallel key-dispatch layers**:
   - **Layer 1 — GPUI keymap:** `cx.bind_keys([...])` at `ui.rs:4632-4726` registers 41 `KeyBinding`s over 35 GPUI actions declared by `gpui::actions!` at `ui.rs:1631-1670`. Dispatch requires an `on_action` listener somewhere on the focused dispatch path.
   - **Layer 2 — command-id interceptor:** `WorkspaceView::new` registers `cx.intercept_keystrokes(...)` at `ui.rs:6219-6236`, which calls `handle_keyboard_shortcut_keystroke` (`ui.rs:10160-10276`). That matches the pressed accelerator (normalized by `normalized_accelerator`/`accelerator_from_keystroke`, `ui.rs:46234-46274`) against `ACTIVE_KEYBOARD_SHORTCUTS` (`ui.rs:2779-3290`, 72 items, id + default accelerators), filters via `keyboard_shortcut_command_enabled` (`ui.rs:10278-10368`), and dispatches `execute_keyboard_shortcut_command` (`ui.rs:10392-10516`). On match it calls `cx.stop_propagation()`.
2. **GPUI 0.2.2 dispatch order** (verified in the published crate source, `gpui-0.2.2/src/window.rs`, `dispatch_key_event`, interceptor call at line 3779 preceding binding matching): interceptors run **before** keymap bindings; a `stop_propagation()` in the interceptor prevents the keymap match. Statically, Layer 2 therefore owns every accelerator present in `ACTIVE_KEYBOARD_SHORTCUTS`. **However**, the program's own confirmed instance (WO-P2-007: Ctrl+P → `OpenFileSearch` was a live runtime no-op while Ctrl+K/Ctrl+G allegedly worked, despite the three being statically identical in shape) proves a runtime factor exists that static analysis cannot derive. See CONFIDENCE.
3. `bind_keys` entries have a **second, real consumer:** popup-menu rows attach the same actions via `.action(Box::new(<Action>))` (`ui.rs:13198-13408`) purely so gpui-component can render the accelerator label (`third_party/gpui-component/src/menu/popup_menu.rs:944-959`, `Kbd::binding_for_action`). Click behavior comes from the row's `on_click`; the attached action is dispatched on click only when no `on_click` exists (`popup_menu.rs:737-759`, `dispatch_confirm_action` at `765-777`). Every menu row in this app also sets `on_click`, so the attached actions are display tokens.
4. The known instance's fix (branch `origin/parity/wo-p2-007-ctrl-p-file-search`, commit `2948bf2`, parent = this base) deletes the `OpenFileSearch` action + binding and adds a registry test; it does **not** touch the interceptor or the registry, which already contained `searchFiles → CmdOrCtrl+P` at base (`ui.rs:3129-3135`, arm `ui.rs:10480`).

---

## A. Key registry — every `KeyBinding::new` in the `bind_keys` block (`ui.rs:4632-4726`; 41 bindings, 35 actions)

Verdicts: **WORKS** = the bound action has an `on_action` handler somewhere in the crate (GPUI dispatch is real). **SILENT NO-OP** = the confirmed WO-P2-007 shape: action declared + bound, zero handlers anywhere (the action "maps to no command and has no direct handler"). **PARTIAL** = action declared + bound with zero handlers (same dead shape) **but** its accelerator is also owned by an `ACTIVE_KEYBOARD_SHORTCUTS` item with a live interceptor arm — dual-path; runtime precedence not statically decidable (see CONFIDENCE). `Cmd id` = the registry item owning the same accelerator.

| Surface | Binding-or-ID | Action | Command id | Interceptor anchor (file:line) | Verdict | Notes |
|---|---|---|---|---|---|---|
| A | ctrl+shift+p (4633) | `OpenCommandMenu` | openCommandMenu | ui.rs:10477 | PARTIAL | Dead action (no handler crate-wide); interceptor owns CmdOrCtrl+Shift+P (ui.rs:3127); not menu-attached |
| A | ctrl+k (4634) | `OpenCommandMenu` | openCommandMenu | ui.rs:10477 | PARTIAL | Same as above; the fix commit's comment cites this key as "already working" via the interceptor |
| A | ctrl+g (4635) | `OpenChatSearch` | searchChats | ui.rs:10411 | PARTIAL | Dead action; interceptor owns CmdOrCtrl+G (ui.rs:2850-2856) |
| A | ctrl+p (4636) | `OpenFileSearch` | searchFiles | ui.rs:10480 | **SILENT NO-OP (confirmed-known)** | WO-P2-007 instance: declared 1636, bound 4636, zero handlers, zero menu attachments. Fixed in flight (2948bf2 removes 1636+4636). Not re-fixed here. NOTE: even post-fix, Ctrl+P silently no-ops with no local workspace — see finding F-A4 below |
| A | ctrl+o (4637) | `OpenFolderShortcut` | openFolder | ui.rs:10462 | PARTIAL | Dead action; menu-attached (13211) for the accelerator label |
| A | ctrl+shift+o (4638) | `NewChatShortcut` | newTask | ui.rs:10399 | PARTIAL | Dead action; menu-attached (13200) |
| A | ctrl+n (4639) | `NewChatShortcut` | newTask | ui.rs:10399 | PARTIAL | Same |
| A | ctrl+alt+s (4640) | `OpenSideChatShortcut` | openSideChat | ui.rs:10401 | PARTIAL | Dead action; not menu-attached |
| A | ctrl+w (4641) | `CloseWindowShortcut` | closeWindow | ui.rs:10512 | PARTIAL | Dead action; menu-attached (13221) |
| A | ctrl+q (4642) | `QuitShortcut` | quit | ui.rs:10512 | PARTIAL | Dead action; menu-attached (13240) |
| A | ctrl+shift+a (4643) | `ArchiveChatShortcut` | archiveThread | ui.rs:10402 | PARTIAL | Dead action; see state-dependent finding F-A1 (silent with no chat selected) |
| A | ctrl+alt+r (4644) | `RenameChatShortcut` | renameThread | ui.rs:10511 | PARTIAL | Dead action; see finding F-A3 |
| A | ctrl+alt+p (4645) | `ToggleChatPinShortcut` | toggleThreadPin | ui.rs:10403 | PARTIAL | Dead action; see finding F-A2 |
| A | ctrl+[ (4646) | `NavigateBackShortcut` | navigateBack | ui.rs:10418 | PARTIAL | Dead action; menu-attached (13383) |
| A | ctrl+] (4647) | `NavigateForwardShortcut` | navigateForward | ui.rs:10419 | PARTIAL | Dead action; menu-attached (13394) |
| A | ctrl+pageup / cmd-alt-left (4648-4656) | `PreviousChatShortcut` | previousThread | ui.rs:10420 | PARTIAL | Dead action; menu-attached (13361); registry defaults PREVIOUS_CHAT_SHORTCUTS (ui.rs:2770-2777) match both keys |
| A | ctrl+shift+[ (4657) | `PreviousChatShortcut` | previousThread | ui.rs:10420 | PARTIAL | Same |
| A | ctrl+pagedown / cmd-alt-right (4658-4666) | `NextChatShortcut` | nextThread | ui.rs:10421 | PARTIAL | Dead action; menu-attached (13372) |
| A | ctrl+shift+] (4667) | `NextChatShortcut` | nextThread | ui.rs:10421 | PARTIAL | Same |
| A | ctrl+f (4668) | `FindInThreadShortcut` | findInThread | ui.rs:10428 | PARTIAL | Dead action; menu-attached (13349); enabled-guarded (10302? no — findInThread guard at 10298-10301) |
| A | ctrl+b (4669) | `ToggleSidebarShortcut` | toggleSidebar | ui.rs:10439 | PARTIAL | Dead action; menu-attached (13288) |
| A | ctrl+j (4670) | `ToggleBottomPanelShortcut` | toggleBottomPanel | ui.rs:10440 | PARTIAL | Dead action; menu-attached (13298); enabled-guard silently disables the key when panel unavailable (10306), while the menu row is honestly `disabled(...)` (13299) — invisible-guard asymmetry |
| A | ctrl+alt+b (4671) | `ToggleReviewPanelShortcut` | toggleSidePanel | ui.rs:10451 | PARTIAL | Dead action; menu-attached (13336); note the action name says ReviewPanel but the registry id is `toggleSidePanel` (ui.rs:3011-3016) — `toggleReviewTab` is a different id with no default key |
| A | ctrl+shift+g (4672) | `OpenReviewShortcut` | openReviewTab | ui.rs:10445 | PARTIAL | Dead action; not menu-attached; enabled-guarded (10302-10305) |
| A | ctrl+` (4673) | `ToggleTerminalShortcut` | toggleTerminal | ui.rs:10459 | PARTIAL | Dead action; menu-attached (13309); reducer surfaces guidance on unavailable (comment 10310-10312, WO-P1-001/002) — the honest contrast to toggleBottomPanel |
| A | ctrl+, (4674) | `OpenSettingsShortcut` | settings | ui.rs:10472 | PARTIAL | Dead action; menu-attached (13263) |
| A | ctrl+/ (4675) | `ShowKeyboardShortcutsShortcut` | showKeyboardShortcuts | ui.rs:10476 | **WORKS** | `on_action` handler at ui.rs:42974-42978 (context `CodexWorkspace`); also menu-attached (13430) |
| A | f11 (4676) | `ToggleFullscreenShortcut` | toggleFullScreen | ui.rs:10513 | PARTIAL | Dead action; menu-attached (13405) |
| A | escape @AboutDialog (4677) | `Escape` | — (no command id) | n/a | **WORKS** | Handler ui.rs:4810-4812 (`AboutView`, key_context "AboutDialog" at 4808) |
| A | escape @McpElicitation (4678) | `Escape` | — | n/a | **WORKS** | Handler ui.rs:20586-20595 (key_context "McpElicitation" at 20584) |
| A | escape @StructuredUserInput (4679) | `Escape` | — | n/a | **WORKS** | Handler ui.rs:20108-20110 (key_context "StructuredUserInput" at 20106) |
| A | tab @RemoveLocalProjectModal (4680-4684) | `RemoveLocalProjectFocusNext` | — | n/a | **WORKS** | Handler ui.rs:41315 (key_context at 41314) |
| A | shift-tab @RemoveLocalProjectModal (4685-4689) | `RemoveLocalProjectFocusPrev` | — | n/a | **WORKS** | Handler ui.rs:41318 |
| A | tab @DeleteArchivedTasksModal (4690-4694) | `DeleteArchivedTasksFocusNext` | — | n/a | **WORKS** | Handler ui.rs:42576 (key_context at 42574) |
| A | shift-tab @DeleteArchivedTasksModal (4695-4699) | `DeleteArchivedTasksFocusPrev` | — | n/a | **WORKS** | Handler ui.rs:42581 |
| A | tab @ResetMemoriesModal (4700) | `ResetMemoriesFocusNext` | — | n/a | **WORKS** | Handler ui.rs:40961 (key_context at 40960) |
| A | shift-tab @ResetMemoriesModal (4701-4705) | `ResetMemoriesFocusPrev` | — | n/a | **WORKS** | Handler ui.rs:40964 |
| A | tab @ResetKeyboardShortcutsModal (4706-4710) | `ResetKeyboardShortcutsFocusNext` | — | n/a | **WORKS** | Handler ui.rs:41030 (key_context at 41022) |
| A | shift-tab @ResetKeyboardShortcutsModal (4711-4715) | `ResetKeyboardShortcutsFocusPrev` | — | n/a | **WORKS** | Handler ui.rs:41035 |
| A | tab @AllowAllBrowserSitesModal (4716-4720) | `AllowAllBrowserSitesFocusNext` | — | n/a | **WORKS** | Handler ui.rs:27043 (key_context at 27041) |
| A | shift-tab @AllowAllBrowserSitesModal (4721-4725) | `AllowAllBrowserSitesFocusPrev` | — | n/a | **WORKS** | Handler ui.rs:27048 |

**Structural A observations:**
- 24 of the 25 globally bound actions (all except `ShowKeyboardShortcutsShortcut`) have **zero `on_action` handlers crate-wide** (verified by exhaustive token search; every other occurrence is the `actions!` macro at 1631-1670, a `bind_keys` entry, or a menu `.action()` display token at 13198-13408). The whole `bind_keys` layer is, statically, a menu-accelerator-label provider plus dead dispatch weight; real behavior lives in the interceptor.
- `OpenBrowserTabShortcut` is declared (1655) and menu-attached (13320) but **never bound and never handled**: the "Open Browser Tab" menu row can therefore display no accelerator, while Ctrl+T works only via the interceptor (`openBrowserTab`, ui.rs:3031-3037 default, arm 10460).
- The WO-P2-007 removal pattern does **not** generalize mechanically to the 23 dead-but-menu-attached actions: deleting action+binding would also delete the menu rows' accelerator labels (see `Kbd::binding_for_action` lookup in `popup_menu.rs:944-959`). Fix scoping is the Lead's call; none proposed here.

---

## B. Command registry — every `KEYBOARD_SHORTCUT_COMMAND_IDS` entry (codex-core/src/lib.rs:137-210; 72 ids)

Definition per work order: a registered command id with **no interceptor match arm** is a DEAD COMMAND. `execute_keyboard_shortcut_command`'s match is at `ui.rs:10398-10514` (catch-all `_ => {}` at 10514). Every registry id also has an `ACTIVE_KEYBOARD_SHORTCUTS` entry (set-equality verified: registry↔active diff empty both ways).

| Surface | Binding-or-ID | Action (arm body) | Command id | Interceptor anchor (file:line) | Verdict | Notes |
|---|---|---|---|---|---|---|
| B | newTask | begin_new_chat | newTask | ui.rs:10399 | WORKS | default CmdOrCtrl+N / CmdOrCtrl+Shift+O (ui.rs:2785) |
| B | newProjectlessTask | begin_projectless_chat | newProjectlessTask | ui.rs:10400 | WORKS | default CmdOrCtrl+Alt+O (2792) |
| B | openSideChat | open_side_chat | openSideChat | ui.rs:10401 | WORKS | default CmdOrCtrl+Alt+S (2799) |
| B | archiveThread | archive_selected_chat | archiveThread | ui.rs:10402 | WORKS (state-silent) | Ctrl+Shift+A (2806); **silently no-ops with no selected chat** (8529-8533); no enabled-guard; overlay advertises unconditionally — finding F-A1 |
| B | toggleThreadPin | toggle_selected_chat_pin | toggleThreadPin | ui.rs:10403 | WORKS (state-silent) | Ctrl+Alt+P (2813); same silent skip (8535-8539) — finding F-A2 |
| B | copyConversationMarkdown | copy_selected_conversation_as_markdown | copyConversationMarkdown | ui.rs:10404 | WORKS | no default key (2820); palette/menu/customization-only |
| B | copyDeeplink | copy_selected_task_value | copyDeeplink | ui.rs:10405 | WORKS | Ctrl+Alt+L (2827); enabled-guard 10288-10290 (needs task copy value) — silently falls through when guarded off |
| B | copySessionId | copy_selected_task_value | copySessionId | ui.rs:10406 | WORKS | Ctrl+Alt+C (2834); enabled-guard 10291-10293 |
| B | copyWorkingDirectory | copy_selected_task_value | copyWorkingDirectory | ui.rs:10407 | WORKS | Ctrl+Shift+C (2841); enabled-guard 10294-10296 |
| B | forkThread | dispatch ForkSelectedTask | forkThread | ui.rs:10410 | WORKS | no default key (2848); enabled-guard 10297; reducer arm core lib.rs:10299 |
| B | searchChats | palette Chats mode / thread-find | searchChats | ui.rs:10411 | WORKS | Ctrl+G (2855) |
| B | navigateBack | navigate_history(false) | navigateBack | ui.rs:10418 | WORKS | Ctrl+[ (2862) |
| B | navigateForward | navigate_history(true) | navigateForward | ui.rs:10419 | WORKS | Ctrl+] (2869) |
| B | previousThread | navigate_adjacent_chat(false) | previousThread | ui.rs:10420 | WORKS | Ctrl+Shift+[ / Ctrl+PageUp (2876, 2773) |
| B | nextThread | navigate_adjacent_chat(true) | nextThread | ui.rs:10421 | WORKS | Ctrl+Shift+] / Ctrl+PageDown (2883, 2777) |
| B | thread1…thread9 | navigate_chat_slot(n) | thread1–thread9 | ui.rs:10422-10427 | WORKS (state-silent) | Ctrl+1..9 (2886-2947); **silently no-ops when the slot has no task** (8554-8561) — arguably by-design; finding F-A5 |
| B | findInThread | thread find / settings-search focus | findInThread | ui.rs:10428 | WORKS | Ctrl+F (2953); enabled-guard 10298-10301 |
| B | focusBrowserAddressBar | focus_browser_address | focusBrowserAddressBar | ui.rs:10436 | WORKS | Ctrl+L (2960); enabled-guard 10316-10320 |
| B | navigateBrowserBack | dispatch NavigateBrowserBack | navigateBrowserBack | ui.rs:10437 | WORKS | Alt+Left (2970); enabled-guard 10321-10335; reducer arm core lib.rs:15075 |
| B | navigateBrowserForward | dispatch NavigateBrowserForward | navigateBrowserForward | ui.rs:10438 | WORKS | Alt+Right (2980); enabled-guard 10336-10350; reducer arm core lib.rs:15086 |
| B | toggleSidebar | toggle_sidebar | toggleSidebar | ui.rs:10439 | WORKS | Ctrl+B (2987) |
| B | toggleBottomPanel | guarded dispatch ToggleBottomPanel | toggleBottomPanel | ui.rs:10440 | WORKS (guard) | Ctrl+J (2994); enabled-guard 10306 + in-arm re-check 10441 — silent when unavailable (contrast: menu row disabled at 13299) |
| B | openReviewTab | guarded dispatch ShowInspector(Changes) | openReviewTab | ui.rs:10445 | WORKS (guard) | Ctrl+Shift+G (3001); enabled-guard 10302-10305; in-arm re-check with empty context stack 10446 |
| B | toggleReviewTab | dispatch ToggleReviewTab | toggleReviewTab | ui.rs:10450 | WORKS | no default key (3008); enabled-guard 10302-10305 |
| B | toggleSidePanel | guarded dispatch ToggleReviewPanel | toggleSidePanel | ui.rs:10451 | WORKS (guard) | Ctrl+Alt+B (3015); enabled-guard 10307-10309 + in-arm re-check 10452 |
| B | toggleMaximizeSidePanel | dispatch ToggleMaximizeSidePanel | toggleMaximizeSidePanel | ui.rs:10456 | WORKS | no default key (3022); enabled-guard 10351-10358; reducer arm core lib.rs:9650 |
| B | toggleTerminal | dispatch ToggleTerminalDock | toggleTerminal | ui.rs:10459 | WORKS | Ctrl+` (3029); guard 10313-10315; reducer surfaces guidance when unavailable (comment 10310-10312) |
| B | openBrowserTab | dispatch OpenBrowserTab | openBrowserTab | ui.rs:10460 | WORKS | Ctrl+T (3036); guard 10313-10315; reducer arm core lib.rs:15012 |
| B | toggleBrowserPanel | dispatch ToggleBrowserPanel | toggleBrowserPanel | ui.rs:10461 | WORKS | Ctrl+Shift+B (3043); guard 10313-10315; reducer arm core lib.rs:15032 |
| B | openFolder | prompt_for_workspace | openFolder | ui.rs:10462 | WORKS | Ctrl+O (3050) |
| B | forceReloadSkills | dispatch RefreshSkills | forceReloadSkills | ui.rs:10463 | WORKS | no default key (3057); reducer arm core lib.rs:17486 |
| B | openSkills | open_skills | openSkills | ui.rs:10464 | WORKS | no default key (3064) |
| B | keyboardShortcuts | open_settings_section(KeyboardShortcuts) | keyboardShortcuts | ui.rs:10465 | WORKS | no default key (3071) |
| B | mcpSettings | open_mcp_settings | mcpSettings | ui.rs:10468 | WORKS | no default key (3078) |
| B | personalitySettings | open_settings_section(Personalization) | personalitySettings | ui.rs:10469 | WORKS | no default key (3085) |
| B | settings | open_general_settings | settings | ui.rs:10472 | WORKS | Ctrl+, (3092) |
| B | openProcessManager | open_process_manager | openProcessManager | ui.rs:10473 | WORKS | Ctrl+Alt+M (3099) |
| B | logOut | confirm_account_logout | logOut | ui.rs:10474 | WORKS (guard) | no default key (3106); enabled-guard 10359-10362 |
| B | feedback | open_feedback_modal | feedback | ui.rs:10475 | WORKS | no default key (3113) |
| B | showKeyboardShortcuts | toggle_keyboard_shortcuts | showKeyboardShortcuts | ui.rs:10476 | WORKS | Ctrl+/ (3120) |
| B | openCommandMenu | palette Unified mode | openCommandMenu | ui.rs:10477 | WORKS | Ctrl+K / Ctrl+Shift+P (3127) |
| B | searchFiles | palette Files mode | searchFiles | ui.rs:10480 | WORKS (state-silent) | Ctrl+P (3134); **silently no-ops with no local workspace** (8494 → early return 8500-8502) — persists after the in-flight WO-P2-007 fix; finding F-A4 |
| B | composer.openModelPicker | open_model_picker | composer.openModelPicker | ui.rs:10481 | WORKS | no default key (3141); target silently returns unless composer_settings available + non-empty models (8302-8305) |
| B | composer.openProjectPicker | open_composer_project_picker_shortcut | composer.openProjectPicker | ui.rs:10482 | WORKS (guard) | Ctrl+Alt+Shift+O (3148); enabled-guard 10363-10365 |
| B | composer.submit | submit_composer_shortcut | composer.submit | ui.rs:10485 | WORKS | no default key (3155); target guard 8334-8338 |
| B | composer.addPhotos | add_composer_photos_shortcut | composer.addPhotos | ui.rs:10486 | WORKS | no default key (3162) |
| B | composer.addFiles | attach_composer_files_shortcut | composer.addFiles | ui.rs:10487 | WORKS | no default key (3169) |
| B | composer.toggleFastMode | toggle_composer_fast_mode | composer.toggleFastMode | ui.rs:10488 | WORKS | no default key (3176); silent return when no fast tier (8356-8370) |
| B | composer.increaseReasoningEffort | change_composer_reasoning_effort | composer.increaseReasoningEffort | ui.rs:10489 | WORKS | no default key (3183) |
| B | composer.decreaseReasoningEffort | change_composer_reasoning_effort | composer.decreaseReasoningEffort | ui.rs:10492 | WORKS | no default key (3190) |
| B | composer.cycleReasoningEffort | change_composer_reasoning_effort | composer.cycleReasoningEffort | ui.rs:10495 | WORKS | no default key (3197) |
| B | composer.togglePlanMode | toggle_composer_plan_mode | composer.togglePlanMode | ui.rs:10498 | WORKS | no default key (3204) |
| B | approval.approve | resolve_active_approval(Accept) | approval.approve | ui.rs:10499 | WORKS (guard) | Enter (3211); enabled-guard 10284-10287; resolve_active_approval silently returns with no pending approval (10377-10382) |
| B | approval.decline | resolve_active_approval(Decline) | approval.decline | ui.rs:10500 | WORKS (guard) | Escape (3218); same guard |
| B | git.commit | open_commit_modal | git.commit | ui.rs:10501 | WORKS (state-silent) | no default key (3225); palette-reachable; **silent return when `state.git.pending_pull_request` is Some** (10519-10521) while the palette row is visible (only repository-guarded, 3890) — finding F-A6 |
| B | git.createPullRequest | open_create_pull_request_modal(false) | git.createPullRequest | ui.rs:10502 | WORKS | no default key (3232); honest SetStatus messages (10606-10627) |
| B | git.createDraftPullRequest | open_create_pull_request_modal(true) | git.createDraftPullRequest | ui.rs:10505 | WORKS | no default key (3239) |
| B | git.createBranch | open_create_branch_modal | git.createBranch | ui.rs:10508 | WORKS | no default key (3246) |
| B | git.mergePullRequest | open_linked_pull_request_merge | git.mergePullRequest | ui.rs:10509 | WORKS (guard) | no default key (3253); palette hides honestly (3895-3897); target guard 10593-10596 |
| B | git.openPullRequest | open_linked_pull_request | git.openPullRequest | ui.rs:10510 | WORKS (guard) | no default key (3260); palette hides honestly (3892-3894); silent return 10573-10582 (unreachable via palette when hidden) |
| B | renameThread | rename_selected_chat | renameThread | ui.rs:10511 | WORKS (state-silent) | Ctrl+Alt+R (3267); **silent with no selected chat** (8642-8654) — finding F-A3 |
| B | closeWindow | window.remove_window() | closeWindow | ui.rs:10512 | WORKS | Ctrl+W (3274) |
| B | quit | window.remove_window() | quit | ui.rs:10512 | WORKS | Ctrl+Q (3281) |
| B | toggleFullScreen | window.toggle_fullscreen() | toggleFullScreen | ui.rs:10513 | WORKS | F11 (3288) |

**B verdict: ZERO DEAD COMMANDS.** All 72 registry ids have interceptor arms, and registry↔`ACTIVE_KEYBOARD_SHORTCUTS` are set-equal.

**B observations:**
- 25/72 ids ship with `shortcuts: &[]` (no default accelerator; count verified) — keyboard-unreachable by default, reachable via palette/menu/user customization. Not bugs.
- 20/72 ids are contextually disabled by `keyboard_shortcut_command_enabled` (ui.rs:10278-10368). When a guard is false the keystroke silently falls through — the overlay still advertises the default accelerator (see F). This is the "guarded into invisibility" pattern; only `toggleTerminal` compensates with reducer guidance (10310-10312).
- Latent quirk: `MAX_KEYBOARD_SHORTCUT_COMMANDS: usize = 71` (core lib.rs:132) but the registry has 72 ids; `KeyboardShortcutPreferences::normalized` (core lib.rs:470-500) iterates the registry and `break`s at 71 (474), so a user who customizes all 72 commands silently loses the override for the last id in iteration order (`toggleFullScreen`).

---

## C. Palette — every `PaletteCommand::ALL` entry (ui.rs:3348-3400; 51 entries)

Executor: `CommandPaletteView::execute_command` (ui.rs:4059-4187) — a compiler-exhaustive match; every arm calls a real target. Dispatched `Action`s were verified to have reducer arms in `codex-core/src/lib.rs` (`reduce`, lib.rs:8864+): ShowInspector (9258), OpenFuzzyFileResult (9352), ArchiveTask (10487), ToggleBottomPanel (9663), ToggleTerminalDock (9677), OpenBrowserTab (15012), ToggleBrowserPanel (15032), ToggleReviewPanel (9593), ToggleMaximizeSidePanel (9650), RefreshSkills (17486), NavigateBrowserBack/Forward (15075/15086), ForkSelectedTask (10299), SetGitPreferences (9767), Navigate (9218), SelectTask (11014). Visibility guards: `filtered_commands` (ui.rs:3865-3910).

| Surface | Binding-or-ID | Action (execute_command arm) | Command id (row shortcut source) | Interceptor anchor (file:line) | Verdict | Notes |
|---|---|---|---|---|---|---|
| C | NewChat | begin_new_chat (4075) | newTask | ui.rs:10399 | WORKS | shortcut label from registry (4262-4277) |
| C | OpenFolder | prompt_for_workspace (4079) | openFolder | ui.rs:10462 | WORKS | |
| C | SearchChats | set_mode(Chats) early-return (4065-4068) | searchChats | ui.rs:10411 | WORKS | |
| C | SearchFiles | set_mode(Files) early-return (4069-4072) | searchFiles | ui.rs:10480 | WORKS | hidden without workspace (3898) — honest; contrast the keyboard path (F-A4) |
| C | OpenGeneralSettings … OpenGitSettings (12 entries, 3353-3363) | open_settings_section(...) (4152-4184) | settings / keyboardShortcuts / personalitySettings / — | ui.rs:10465-10472 | WORKS | each targets an existing `SettingsSection` render arm (32009-32028) |
| C | ArchiveChat | guarded ArchiveTask (4080-4084) | archiveThread | ui.rs:10402 | WORKS | palette requires selected chat (3706) — honest |
| C | NewStandaloneChat | begin_projectless_chat (4076) | newProjectlessTask | ui.rs:10400 | WORKS | |
| C | ToggleChatPin | toggle_selected_chat_pin (4085) | toggleThreadPin | ui.rs:10403 | WORKS | requires selected chat (3707) |
| C | NavigateBack / NavigateForward | navigate_history (4086-4087) | navigateBack / navigateForward | ui.rs:10418-10419 | WORKS | |
| C | PreviousChat / NextChat | navigate_adjacent_chat (4088-4089) | previousThread / nextThread | ui.rs:10420-10421 | WORKS | |
| C | FindInThread | open_thread_find (4090) | findInThread | ui.rs:10428 | WORKS | requires task workspace (3721) |
| C | ToggleSidebar | toggle_sidebar (4091) | toggleSidebar | ui.rs:10439 | WORKS | |
| C | ToggleBottomPanel | dispatch ToggleBottomPanel (4092-4094) | toggleBottomPanel | ui.rs:10440 | WORKS | requires task workspace (3722) |
| C | LogOut | confirm_account_logout (4095) | logOut | ui.rs:10474 | WORKS | requires account (3746) |
| C | Feedback | open_feedback_modal (4096) | feedback | ui.rs:10475 | WORKS | |
| C | OpenProcessManager | open_process_manager (4097) | openProcessManager | ui.rs:10473 | WORKS | requires selected chat (3708) |
| C | OpenRepository | navigate(MainRoute::Repository) (4098) | — (no command id; no shortcut shown) | n/a | WORKS | |
| C | CommitOrPush | open_commit_modal (4099) | git.commit | ui.rs:10501 | **PARTIAL** | requires repository (3736) but not "no pending PR": with a repository and a pending PR the visible row silently no-ops (10519-10521) — finding F-A6 |
| C | CreatePullRequest / CreateDraftPullRequest | open_create_pull_request_modal (4100-4105) | git.createPullRequest / git.createDraftPullRequest | ui.rs:10502 / 10505 | WORKS | honest SetStatus guards (10606-10627) |
| C | CreateBranch | open_create_branch_modal (4106) | git.createBranch | ui.rs:10508 | WORKS | |
| C | MergePullRequest | open_linked_pull_request_merge (4107-4109) | git.mergePullRequest | ui.rs:10509 | WORKS | palette-filtered honestly (3895-3897) |
| C | OpenPullRequest | open_linked_pull_request (4110) | git.openPullRequest | ui.rs:10510 | WORKS | palette-filtered honestly (3892-3894) |
| C | OpenReviewTab | dispatch ShowInspector(Changes) (4111-4113) | openReviewTab | ui.rs:10445 | WORKS | requires chat+workspace (3709, 3723) |
| C | ToggleReviewPanel | dispatch ToggleReviewPanel (4114-4116) | toggleSidePanel | ui.rs:10451 | WORKS | requires workspace (3724) |
| C | DisableGitReview / EnableGitReview | SetGitPreferences (4117-4125) | — (no command id) | n/a | WORKS | mutually filtered by review mode (3899-3903) |
| C | ToggleTerminal | dispatch ToggleTerminalDock (4126-4128) | toggleTerminal | ui.rs:10459 | WORKS | requires chat+workspace (3710, 3725); reducer guidance when unavailable |
| C | OpenBrowserTab | dispatch OpenBrowserTab (4129-4131) | openBrowserTab | ui.rs:10460 | WORKS | requires chat+workspace (3711, 3726) |
| C | ToggleBrowserPanel | dispatch ToggleBrowserPanel (4132-4134) | toggleBrowserPanel | ui.rs:10461 | WORKS | requires chat+workspace (3712, 3727) |
| C | FocusBrowserAddressBar | focus_browser_address (4135-4137) | focusBrowserAddressBar | ui.rs:10436 | WORKS | requires chat+workspace (3713, 3728) |
| C | ShowComputerUse | dispatch ShowInspector(ComputerUse) (4138-4140) | — | n/a | WORKS | requires chat+workspace (3714, 3729); reducer arm 9258 |
| C | OpenSkills | open_skills (4141) | openSkills | ui.rs:10464 | WORKS | |
| C | ForceReloadSkills | dispatch RefreshSkills (4142) | forceReloadSkills | ui.rs:10463 | WORKS | |
| C | OpenMcpSettings | open_mcp_settings (4143) | mcpSettings | ui.rs:10468 | WORKS | |
| C | OpenPersonalitySettings | open_settings_section(Personalization) (4144-4146) | personalitySettings | ui.rs:10469 | WORKS | |
| C | OpenPlugins | navigate(MainRoute::Marketplace) (4147) | — | n/a | WORKS | goes to the Marketplace route, **not** `SettingsSection::Plugins`; the Plugins *settings* page has no palette entry (see E) |
| C | OpenWorkflows | navigate(MainRoute::Workflows) (4148) | — | n/a | WORKS | |
| C | OpenConnectionsSettings | open_settings_section(Connections) (4149-4151) | — | n/a | WORKS | |

**C observations:**
- The empty arm `PaletteCommand::SearchChats | PaletteCommand::SearchFiles => {}` (4185) is unreachable (early returns at 4065-4072) — defensive, not a bug.
- Palette row shortcut labels come from `shortcut_command_id()` → `ACTIVE_KEYBOARD_SHORTCUTS` effective bindings (4262-4277), falling back to `PaletteCommand::shortcut()` (3514-3539). The fallback is dead weight: every variant with `shortcut() == Some` also has a command id (maps at 3514-3539 ⊂ 3541-3575), so rows without ids never show a stale label.
- 12 of the 15 default-nav settings sections are palette-openable; **Plugins is not** — `OpenPlugins` routes to Marketplace (4147) while the comment at 2485-2489 claims the palette "indexes every section in this registry" (DEFAULT_NAV_SECTIONS includes Plugins, 2502). Contextual sections CodeReview/Worktrees/ArchivedChats are documented as intentionally out (WO-P2-004).

---

## D. Slash commands — catalog (ui.rs:45043-45093, 21 ids) × executor (ui.rs:7738-7906) × reducer SubmitComposer special-cases (codex-core lib.rs:13178-13252)

Submit path: `WorkspaceView::submit` (7701) → `execute_composer_slash_command` (7725); unhandled text is dispatched as `Action::SubmitComposer` (7728-7729), whose reducer arm handles `/new`, `/fork`, `/compact`, `/shell <cmd>` specially (core 13180-13260). The slash menu rows are rendered at 23455-23830 with per-command visibility guards.

| Surface | Binding-or-ID | Action (handler + anchor) | Command id | Interceptor anchor (file:line) | Verdict | Notes |
|---|---|---|---|---|---|---|
| D | /approve | resolve_active_approval — ui.rs:7747-7759 | — | n/a | WORKS | guard 7748-7750 returns false when no approval → text submitted as plain message (visible); menu row hidden honestly (23503-23505) |
| D | /chat | begin_projectless_chat — ui.rs:7760-7763 | — | n/a | WORKS | guard has_local_workspace; row hidden without workspace (23455-23457) |
| D | /compact | reducer CompactThread — core lib.rs:13202-13229 | — | n/a | **PARTIAL** | guards: attachments → composer_error (13203-13205, honest); no task → error (13207-13210, honest); **runtime not ready → `return Vec::new()` with no message (13212-13214, SILENT)**; turn active → error (13216-13219); already compacting → error (13221-13224). Menu row visible whenever a task is selected (23458-23459), so the silent state is reachable from a visible row — finding F-D2 |
| D | /fast | select_service_tier_slash — ui.rs:7807-7813 | — | n/a | WORKS | guard fast_service_tier_id (7808-7810) → message fallback; row hidden without fast tier (availability 7930) |
| D | /feedback | open_feedback_modal — ui.rs:7889 | — | n/a | WORKS | |
| D | /fork | fork picker — ui.rs:7825-7834 | — | n/a | WORKS | executor always handles (opens/confirms picker); the reducer's own /fork special-case (core 13187-13200) is unreachable from the composer submit path (single submit site 7729; executor returns true for "/fork" unconditionally) — redundant defensive arm, not user-facing |
| D | /goal | edit_goal — ui.rs:7890 | — | n/a | WORKS | row visible with task (23465-23466) |
| D | /init | submit_init_prompt — ui.rs:7891 | — | n/a | WORKS | guard 7744-7746 → message fallback; row hidden when unavailable (23467-23469) |
| D | /mcp | open_mcp_slash_status — ui.rs:7846-7849 | — | n/a | WORKS | always available (45078) |
| D | /memories | ChatMemories modal — ui.rs:7892-7895 | — | n/a | WORKS | guard 7854-7856 → message fallback; row hidden when memory unavailable (23471-23473) |
| D | /model | open_model_picker — ui.rs:7896 | — | n/a | WORKS | row requires model description (23474-23477); picker target silently returns if models empty (8303) — menu row visibility covers the common case |
| D | /new | reducer BeginNewChat — core lib.rs:13180-13186 | — | n/a | WORKS | honest error on attachments (13181-13183); row visible with task (23478-23479) |
| D | /personality | settings Personalization — ui.rs:7897-7899 | — | n/a | WORKS | always available (45078) |
| D | /plan | toggle_composer_plan_mode — ui.rs:7900 | — | n/a | WORKS | row requires model description (23480-23483) |
| D | /project | project picker — ui.rs:7850-7853 | — | n/a | WORKS | always available (45078) |
| D | /reasoning | open_reasoning_picker — ui.rs:7901 | — | n/a | WORKS | row requires reasoning description (23486-23489) |
| D | /review | review submenu / start — ui.rs:7835-7845 | — | n/a | **PARTIAL** | **when review is unavailable and the submenu is closed: `return true` with no action, no message, no composer clear (7836-7838) — silent swallow**; the menu row is honestly hidden in that state (23531-23534), but a typed/memorized /review silently no-ops — finding F-D1 |
| D | /shell | prefill "/shell " — ui.rs:7861-7870; with args → reducer — core 13231-13260 | — | n/a | WORKS | guard 7861 (task selected) → message fallback; reducer guards set composer_error honestly (13238-13252) |
| D | /side | open_side_chat — ui.rs:7764-7775 | — | n/a | WORKS | guard 7765-7767 → message fallback; row hidden without task (23499-23501) |
| D | /status | open_composer_status — ui.rs:7857-7860 | — | n/a | WORKS | always available (45078) |
| D | /worktree | worktree picker — ui.rs:7814-7824 | — | n/a | WORKS | guard 7815-7817 → message fallback; row hidden when guard fails (availability 7931) |
| D | /service-tier:{id} (dynamic family) | select_service_tier_slash — ui.rs:7776-7793 | — | n/a | WORKS | guarded by model/tier validity → message fallback; not in the static COMMANDS list |
| D | /skill:{path} (dynamic family) | select_composer_skill_slash — ui.rs:7794-7806 | — | n/a | WORKS | guarded by skill enabled+absolute path → message fallback; not in the static COMMANDS list |

**D observations:**
- All 21 catalog ids are covered (19 by the UI executor, `/compact` + `/new` by the reducer's SubmitComposer arm). **No dead catalog entries.**
- Guard-honesty asymmetry: most unavailable guarded commands fall through to message submission (visible in the thread); `/review` is the only executor command that swallows silently; `/compact`'s runtime-not-ready guard is the only reducer guard without a `composer_error`.

---

## E. Settings nav — every `SettingsSection` (enum ui.rs:2463-2482; 18 variants)

Render dispatch: `render_settings` page match at ui.rs:32009-32028 (compiler-exhaustive, all 18 arms present); nav rows rendered at 31960-32296 behind search-visibility flags; `DEFAULT_NAV_SECTIONS` (2491-2507) is `#[cfg(test)]`-only documentation of the 15 default rows.

| Surface | Binding-or-ID | Action (render fn) | Command id | Interceptor anchor (file:line) | Verdict | Notes |
|---|---|---|---|---|---|---|
| E | General | render_general_settings (32010) | settings | ui.rs:10472 | WORKS | nav row 32087-32095 |
| E | Appearance | render_appearance_settings (32011) | — (palette: OpenAppearanceSettings → 4155-4157) | n/a | WORKS | nav row 32096-32104 |
| E | Personalization | render_personalization_settings (32012) | personalitySettings | ui.rs:10469 | WORKS | nav row 32105-32113 |
| E | KeyboardShortcuts | render_keyboard_shortcut_settings (32013) | keyboardShortcuts | ui.rs:10465 | WORKS | nav row 32114-32122 |
| E | Profile | render_profile_settings (32014) | — | n/a | WORKS | nav row 32123-32131 |
| E | Usage | render_usage_settings (32015) | — | n/a | WORKS | nav row 32132-32140 |
| E | Import | render_import_settings (32016) | — | n/a | WORKS | nav row 32141-32149 |
| E | Configuration | render_agent_configuration_settings (32017) | — | n/a | WORKS | nav row 32227-32235 |
| E | Git | render_git_settings (32018) | — (palette OpenGitSettings → 4182-4184) | n/a | WORKS | nav row 32245-32253 |
| E | CodeReview | render_code_review_settings (32019) | — | n/a | WORKS | contextual; nav row 32254-32262 (search-visible); intentionally out of palette index (2485-2489) |
| E | Hooks | render_hooks_settings (32020) | — | n/a | WORKS | nav row 32236-32244 |
| E | Worktrees | render_worktrees_settings (32021) | — | n/a | WORKS | contextual; nav row 32263-32271; intentionally out of palette index |
| E | Plugins | render_plugins_settings (32022) | — | n/a | WORKS | nav row 32166-32174; **no palette entry opens this section** — palette `OpenPlugins` routes to Marketplace (4147) despite the comment at 2485-2489 claiming full default-nav indexing |
| E | McpServers | render_mcp_server_settings (32023) | mcpSettings | ui.rs:10468 | WORKS | nav row 32175-32183 |
| E | Browser | render_browser_settings (32024) | — | n/a | WORKS | nav row 32184-32192 |
| E | ComputerUse | render_computer_use_settings_page (32025) | — | n/a | WORKS | nav row 32193-32201 |
| E | Connections | render_connections_settings_page (32026) | — | n/a | WORKS | nav row 32202-32210 |
| E | ArchivedChats | render_archived_chat_settings (32027) | — | n/a | WORKS | contextual; nav row 32288-32294; intentionally out of palette index |

**E verdict: 18/18 WORKS — zero dead nav rows; every section has both a nav row and a render arm.**

---

## F. Shortcut-label honesty — overlay (`render_keyboard_shortcuts_modal`, ui.rs:38935-39047) and settings page (`render_keyboard_shortcut_settings`, ui.rs:32622-32681)

Both surfaces iterate `ACTIVE_KEYBOARD_SHORTCUTS`; the overlay additionally filters to items with ≥1 effective binding (38943), so the 25 empty-default commands are **not** advertised (honest). 47 commands are advertised with default accelerators.

| Surface | Binding-or-ID | Action | Command id | Interceptor anchor (file:line) | Verdict | Notes |
|---|---|---|---|---|---|---|
| F | Ctrl+P — "Search files" | palette Files mode | searchFiles | ui.rs:10480 | **DISHONEST (confirmed-known)** | The WO-P2-007 instance: advertised at base while the key is a confirmed runtime no-op. Also remains dishonest in the no-workspace state even after the in-flight fix (8500-8502) |
| F | Ctrl+Shift+A — "Archive chat" | archive_selected_chat | archiveThread | ui.rs:10402 | DISHONEST (state-dependent) | advertised unconditionally; silent when no chat selected (8529-8533) |
| F | Ctrl+Alt+P — "Toggle pin" | toggle_selected_chat_pin | toggleThreadPin | ui.rs:10403 | DISHONEST (state-dependent) | silent when no chat selected (8535-8539) |
| F | Ctrl+Alt+R — "Rename chat" | rename_selected_chat | renameThread | ui.rs:10511 | DISHONEST (state-dependent) | silent when no chat selected (8642-8654) |
| F | Ctrl+1…Ctrl+9 — "Go to chat N" (9 rows) | navigate_chat_slot | thread1–thread9 | ui.rs:10422 | DISHONEST (state-dependent, arguably by-design) | silent when the slot has no task (8554-8561) |
| F | 16 guard-gated rows: Enter/Escape (approval.*), Ctrl+Alt+L/C, Ctrl+Shift+C (copy*), Ctrl+F, Ctrl+Shift+G, Ctrl+Alt+B, Ctrl+J, Ctrl+`, Ctrl+T, Ctrl+Shift+B, Ctrl+L, Alt+Left/Right, Ctrl+Alt+Shift+O | various | 16 ids | ui.rs:10284-10365 | ADVERTISED-WHILE-DISABLED | when the `keyboard_shortcut_command_enabled` guard is false, the advertised key silently falls through (invisible disabling); only toggleTerminal's reducer gives guidance |
| F | remaining 18 advertised rows | various | 18 ids | ui.rs:10398-10514 | HONEST | no silent state found statically |
| F | 25 empty-default commands | — | 25 ids | arms exist | HONEST (unadvertised) | correctly excluded by the 38943 filter |

---

## SUMMARY

**Counts per verdict per surface**

| Surface | Total | WORKS | PARTIAL | SILENT NO-OP / dead |
|---|---|---|---|---|
| A. key registry (bindings) | 41 | 14 | 26 | 1 (OpenFileSearch — confirmed-known, fixed in flight) |
| B. command registry (ids) | 72 | 72 armed (0 DEAD COMMANDS) | — | — |
| C. palette entries | 51 | 50 | 1 (CommitOrPush pending-PR silent skip) | — |
| D. slash catalog ids | 21 | 19 | 2 (/review, /compact) | — |
| E. settings sections | 18 | 18 | — | — |
| F. advertised shortcut rows | 47 | 18 clean + 25 correctly unadvertised | 16 advertised-while-guard-disabled | 13 rows with provable silent states (1 confirmed-known + 12 state-dependent) |

**SILENT NO-OP findings (full list, exact anchors)** — excluding the known Ctrl+P/`OpenFileSearch` instance, which is confirmed-known and fixed in flight (`origin/parity/wo-p2-007-ctrl-p-file-search` @ 2948bf2; action decl ui.rs:1636, binding ui.rs:4636):

- **F-A1 — `archiveThread` (Ctrl+Shift+A) with no selected chat.** Arm ui.rs:10402 → `archive_selected_chat` ui.rs:8529-8533 (`if let Some(task_id) …` else silent). No enabled-guard; overlay row advertised unconditionally (ui.rs:2801-2807).
- **F-A2 — `toggleThreadPin` (Ctrl+Alt+P) with no selected chat.** Arm ui.rs:10403 → `toggle_selected_chat_pin` ui.rs:8535-8539. Same shape.
- **F-A3 — `renameThread` (Ctrl+Alt+R) with no selected chat.** Arm ui.rs:10511 → `rename_selected_chat` ui.rs:8642-8654 (two silent `return`s). Same shape.
- **F-A4 — `searchFiles` (Ctrl+P) with no local workspace — survives the in-flight fix.** Arm ui.rs:10480 → `open_command_palette` ui.rs:8494, early `return` at 8500-8502 when `mode == PaletteMode::Files && !self.has_local_workspace()`. The interceptor has consumed the keystroke; nothing visible happens. The palette entry is honestly hidden in this state (3898); the keyboard shortcut and its overlay row (ui.rs:3129-3135) are not.
- **F-A5 — `thread1`–`thread9` (Ctrl+1…9) with an empty slot.** Arm ui.rs:10422-10427 → `navigate_chat_slot` ui.rs:8554-8561 (silent `return` when `task_slot_id` is None). Listed for completeness; plausibly by-design.
- **F-A6 — `git.commit` palette row with a pending pull request.** Palette arm ui.rs:4099 → `open_commit_modal` ui.rs:10518-10521 (`if self.state.git.pending_pull_request.is_some() { return; }` — silent). Row visible whenever a repository exists (filter 3890 does not model the pending-PR state).
- **F-D1 — `/review` typed while review is unavailable.** Executor ui.rs:7835-7838: `if !submenu_open && !composer_review_available() { return true; }` — swallows the command with no message and no composer change. The menu row is honestly hidden (23531-23534); only a typed command hits the silent path.
- **F-D2 — `/compact` while the selected thread's runtime is not ready.** Reducer guard codex-core/src/lib.rs:13212-13214: `if !selected_thread_runtime_ready(state) { return Vec::new(); }` — no `composer_error`, no clear, no effect. Every neighboring `/compact` guard (13203-13224) reports honestly.

**DEAD COMMAND findings:** none. All 72 `KEYBOARD_SHORTCUT_COMMAND_IDS` entries have an `execute_keyboard_shortcut_command` arm (ui.rs:10398-10514) and an `ACTIVE_KEYBOARD_SHORTCUTS` entry (set-equal, ui.rs:2779-3290).

**Structural watch list (the sibling shapes of the known instance, PARTIAL verdicts):** 23 bound GPUI actions with zero handlers — `OpenCommandMenu` (4633, 4634), `OpenChatSearch` (4635), `OpenFolderShortcut` (4637), `NewChatShortcut` (4638, 4639), `OpenSideChatShortcut` (4640), `CloseWindowShortcut` (4641), `QuitShortcut` (4642), `ArchiveChatShortcut` (4643), `RenameChatShortcut` (4644), `ToggleChatPinShortcut` (4645), `NavigateBackShortcut` (4646), `NavigateForwardShortcut` (4647), `PreviousChatShortcut` (4648, 4657), `NextChatShortcut` (4658, 4667), `FindInThreadShortcut` (4668), `ToggleSidebarShortcut` (4669), `ToggleBottomPanelShortcut` (4670), `ToggleReviewPanelShortcut` (4671), `OpenReviewShortcut` (4672), `ToggleTerminalShortcut` (4673), `OpenSettingsShortcut` (4674), `ToggleFullscreenShortcut` (4676) — plus unbound `OpenBrowserTabShortcut` (declared 1655, menu-attached 13320, never bound, never handled). Each bound action's accelerator is also owned by the interceptor registry, and most are menu accelerator-label tokens; removal (the WO-P2-007 pattern) would strip menu labels, so any fix must decouple those concerns. No fixes proposed here.

**Secondary findings:**
- `MAX_KEYBOARD_SHORTCUT_COMMANDS = 71` vs 72 registry ids (core lib.rs:132; normalization loop core lib.rs:473-498 breaks at 71) — a fully-customized user silently loses the last id's override (`toggleFullScreen`).
- Reducer `/fork` special-case (core lib.rs:13187-13200) is unreachable from the composer submit path (single submit site ui.rs:7729 behind the unconditional executor arm at 7825-7834).
- Doc comment inaccuracy: ui.rs:2485-2489 claims the palette indexes every default-nav settings section; `SettingsSection::Plugins` has no palette entry (palette `OpenPlugins` routes to Marketplace, ui.rs:4147).

**Single most surprising finding:** the in-flight WO-P2-007 fix does not fully cure Ctrl+P. Removing the dead `OpenFileSearch` action routes the key to `searchFiles`, but `open_command_palette(PaletteMode::Files)` silently returns when no local workspace is open (ui.rs:8500-8502) — so on a fresh install with no folder open, the advertised Ctrl+P remains a silent no-op even after the fix lands. More broadly, 24 of the 25 globally bound GPUI actions are declared-but-never-handled: the entire `bind_keys` layer functions as a menu-accelerator-label provider, with real key dispatch living in the parallel string-matching interceptor.

---

## VERIFICATION BATTERY (reproduce each finding in one command; run from repo root at base SHA)

| # | Command | Expected hits (file:line) | Finding |
|---|---|---|---|
| 1 | `grep -n "OpenFileSearch" crates/codex-app/src/ui.rs` | 2 — 1636, 4636 | Known instance (confirmed; fixed in flight) |
| 2 | `grep -nE "fn archive_selected_chat\|fn toggle_selected_chat_pin\|fn rename_selected_chat" crates/codex-app/src/ui.rs` | 3 — 8529, 8535, 8642 | F-A1/A2/A3 silent targets |
| 3 | `grep -n "PaletteMode::Files && !self.has_local_workspace" crates/codex-app/src/ui.rs` | 1 — 8500 | F-A4 Files-mode silent bail |
| 4 | `grep -n 'command == "/review"' crates/codex-app/src/ui.rs` | 1 — 7835 | F-D1 /review swallow (read 7835-7845) |
| 5 | `grep -n "selected_thread_runtime_ready" crates/codex-core/src/lib.rs` | 3 — 7591, 13212, 13400 | F-D2 /compact silent guard (13212-13214) |
| 6 | `grep -n "pending_pull_request.is_some" crates/codex-app/src/ui.rs` | 9 — 7485, 10519, 10655, 10668, 10954, 16107, 39643, 39717, 41192 | F-A6 commit-modal silent return (10519-10521) |
| 7 | `grep -nE "OpenCommandMenu\|OpenChatSearch" crates/codex-app/src/ui.rs` | 5 — 1634, 1635, 4633, 4634, 4635 | Dead-action sibling shapes (no `on_action` anywhere) |
| 8 | `grep -n "fn keyboard_shortcut_command_enabled" crates/codex-app/src/ui.rs` | 1 — 10278 | Invisible-guard block (read 10278-10368) |
| 9 | `grep -n "MAX_KEYBOARD_SHORTCUT_COMMANDS" crates/codex-core/src/lib.rs` | 2 — 132, 474 | Preference truncation quirk |
| 10 | `grep -n "ShowKeyboardShortcutsShortcut" crates/codex-app/src/ui.rs` | 4 — 1657, 4675, 13430, 42975 | The one bound+handled global action |
| 11 | `awk 'NR>=2779 && NR<=3290 && /shortcuts: &\[\]/' crates/codex-app/src/ui.rs \| wc -l` | 25 | Empty-default-shortcut registry items (unadvertised) |

(`rg -n` equivalents were used during the sweep and produce the same lines; both `grep` and `rg` are available in the environment.)

**Cargo:** `cargo test -p codex-app` — **NOT RUN**: the `cargo` binary is not installed in this sandbox (`which cargo` → empty). Substitute evidence that no product code changed: this branch's diff vs base is a single new file under `docs/` (`git diff --stat 3c9f113` shows only `docs/research/evidence/wo-p2-007/input-surface-sweep.md`), and `git status --porcelain` is clean apart from that file.

---

## CONFIDENCE — what static analysis could NOT verify

1. **Runtime precedence between the two dispatch layers.** GPUI 0.2.2 dispatches `intercept_keystrokes` callbacks before keymap binding matching (gpui-0.2.2 `src/window.rs`, `dispatch_key_event`, interceptor invocation preceding `dispatch_tree.dispatch_key`), and the registry at base already maps `CmdOrCtrl+P → searchFiles` with a live arm — so pure static reading predicts Ctrl+P should have worked at base. The program's records (WO-P2-007) prove it did not, while claiming Ctrl+K/Ctrl+G did, despite the three being statically identical in shape (dead action + binding + interceptor coverage). The factor that separates them is not derivable from source; candidates include platform event delivery, focus/context state, or build-specific behavior. Consequences: (a) the 23 PARTIAL sibling actions cannot be classified WORKS vs live-no-op without GUI runs; (b) the state-dependent silent findings (F-A1…F-A6, F-D1, F-D2) prove the *guard paths* statically, but whether users hit those states is a runtime question.
2. **Arm-target visibility.** Whether each interceptor/palette/slash target actually renders a modal, moves focus, writes the clipboard, or emits an effect that the backend executes was not traced beyond the reducer (`reduce` arms verified to exist; `backend.rs` effect handlers were not swept).
3. **Platform variance.** macOS Cmd vs Ctrl normalization (ui.rs:46176-46219), F11/Tab delivery by window managers, and IME/keyboard-layout edge cases are unverifiable statically.
4. **User customization.** The sweep used default accelerators; `effective_keyboard_shortcuts` (ui.rs:9950-9961) overlays persisted preferences, so a real user's advertised rows and match results can differ per profile.
5. **In-repo third-party behavior.** gpui-component menu/accelerator semantics were read from the vendored copy (`third_party/gpui-component`); any divergence between it and the published 0.5.1 crate is out of scope.
6. Line anchors are valid **only at base SHA `3c9f113851686e1774ce355ad744b56d6c937b2a`**; the in-flight WO-P2-007 branch shifts ui.rs lines by −2 after its two deletions.
