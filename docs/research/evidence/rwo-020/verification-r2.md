# RWO-021 — Fresh end-to-end verification of main @ `876bbe8` (WO-REVIEW-001 deep phase, rubric execution)

> **Second-dispatch reconciliation (house precedent: rubric → rubric-r2).** A sibling
> first-dispatch execution of this same work order landed on `research/rwo-021` first
> (commit `a622b7b`, 2026-09-19 08:22 UTC, same base `876bbe8`) and its deliverable stays at
> `docs/research/evidence/rwo-020/verification.md`. That dispatch was environment-bound: no
> native build was possible (missing wayland/pipewire dev packages; no sysroot), so its
> battery covers core/protocol/storage only and its GUI evidence is archived-frame re-reads.
> This second dispatch materialized the userspace sysroot (per `scripts/dev-env.sh`), built
> the release binary, ran the full five-crate battery, and executed ten fresh GUI scenes on
> the real binary; it ships here as `verification-r2.md`. The two records agree on every
> input-surface invariant (76/76/76, zero dead commands, set-equality, 25 empty-default ids)
> and on the KSR-C5 documentation finding; the Lead chooses how to weigh the lineages.

- **Task ID:** RWO-021 (one third of WO-REVIEW-001 — fresh end-to-end verification pass)
- **Worker:** RWO-021 (review worker; reports defects, does not fix)
- **Date:** 2026-09-19 (UTC)
- **Base:** `main` @ `876bbe862e0626a1b6788038e5ebf4347c918be2` (verified: commit exists and is
  `origin/main` HEAD at clone time; post wave-S merges PR #24 WO-P2-009, PR #25 WO-P2-010,
  PR #26 RWO-020 rubric)
- **Branch:** `research/rwo-021` (off the base SHA; documents-only — no product code touched)
- **Deliverable:** this file. Executes `rubric.md` (13 rows / 71 criteria) with `rubric-r2.md`'s
  Appendix A anchor table + Appendix D result-capture template, plus the RWO-021 task-directive
  items (build, test battery, 8-complete-rows evidence re-verification, input-surface
  invariants, GUI-exercisable checks).
- **Consumers:** Tech Lead re-verifies every claim at the integration station; Worker C may
  attack this execution per rubric-r2 §1.5.

## 0. Environment and EQ resolutions

| Item | Value |
| --- | --- |
| Sandbox | Debian 13 (trixie) userspace, 2 cores, 4.1 GiB RAM, no root |
| Rust | rustup-installed pinned toolchain `1.97.1` (per `rust-toolchain.toml`); `cargo 1.97.1 (c980f4866 2026-06-30)` |
| Native deps | userspace sysroot at `/home/z/sysroot/prefix` materialized from 99 stock Debian `.deb`s (xkbcommon/x11/xcb/wayland/pipewire/egl/gbm/drm dev packages, libclang-19, mesa-vulkan-drivers incl. `lvp_icd.json`, picom/xdotool/x11-utils) — the `scripts/dev-env.sh` pattern; nothing on the host modified |
| Display | `Xvfb :104 -screen 0 1600x1000x24` + `picom --backend xrender` + `LIBGL_ALWAYS_SOFTWARE=1` + `VK_ICD_FILENAMES=…lvp_icd.json` (the sealed lab recipe); app window resolves to **1278x818 at screen +163,+91** |
| GUI capture/keys | `ffmpeg -f x11grab` frames + `xdotool` keys/clicks; md5 sequences per the house pattern |
| VLM reads | z-ai VLM (`glm-5v-turbo`) over every load-bearing frame; transcripts saved as `vlm-*.json` in the scene evidence dir; neutral prompts for contested frames |
| EQ-1 (runtime) | **No `codex` CLI in this lab** (`which codex` empty, `CODEX_RS_CODEX_BIN` unset; `resolve_codex_binary` falls to bare `codex` → unresolvable). Thread-dependent scenes fall to the D11b-class bound (source + unit-test + NOT RUN with reason), exactly as the rubric prescribes. |
| EQ-2 (win/mac labs) | Unavailable. Windows-side evidence stays `[historical-record]`; the windows-latest CI matrix could not be queried (see SFR-C7). |
| EQ-3 (auth wall) | No credentials; Flauz-side unauthenticated surfaces GUI-verified only. |
| EQ-4 (26.825 bundle) | No asar inspection exists; 26.825 micro-details stay `[unverified]` on the official side. |

Fresh scene evidence directory: **`docs/research/evidence/rwo-021/`** (this delivery adds
`verification.md` here under `rwo-020/` per the task directive; the frames, md5 files, VLM
transcripts, and scene scripts are archived in `rwo-021/` — see §2).

## 1. Build and test battery (exact commands + exit codes)

| # | Command (from repo root, dev-env exports active) | Result |
| --- | --- | --- |
| B-1 | `cargo build --locked -p codex-app --release` | **exit 0** — `Finished \`release\` profile [optimized] target(s) in 8m 50s` (two resumable chunks under the sandbox 550s process cap; `CARGO_TARGET_DIR=/home/z/flauz/target`, `CARGO_INCREMENTAL=0`). One benign warning: `proc-macro-error2 v2.0.1` future-incompat notice. |
| B-2 | `cargo test --locked -p codex-core --release` | **exit 0** — `231 passed; 0 failed; 0 ignored` |
| B-3 | `cargo test --locked -p codex-storage --release` | **exit 0** — `19 passed; 0 failed` |
| B-4 | `cargo test --locked -p codex-protocol --release` | **exit 0** — `60 passed; 0 failed` |
| B-5 | `cargo test --locked -p codex-platform --release` | **exit 0** — unit `114 passed; 0 failed; 2 ignored` + integration `2 passed` |
| B-6 | `cargo test --locked -p codex-app --release` | **exit 0** — `213 passed; 0 failed; 0 ignored` (2 `#[cfg(windows)]` tests compile only on the windows matrix: `windows_computer_use_approval_copy_remains_unchanged` backend.rs:22050, `extended_windows_paths_are_displayed_normally` ui.rs:51614) |
| B-7 | Per-test battery (WO-P2-008 house pattern) | 7/7 core state tests `ok` per-test (names below); 6/6 app binding/jump tests `ok` per-test |

**Battery totals (this sandbox, release profile): 639 passed / 0 failed / 2 ignored (platform).**
No test failure anywhere → no defect-found from the battery. The windows-latest leg of the
battery is CI-only (EQ-2/SFR-C7 bound).

Per-test names recorded green (B-7): `background_turn_completion_marks_the_chat_needing_attention`,
`approval_request_marks_background_chats_needing_attention`,
`visiting_a_chat_clears_its_attention_flag`, `clear_all_unread_indicators_reports_honestly`,
`toggle_selected_chat_unread_round_trips_with_status`,
`archiving_a_chat_drops_its_attention_flag`, `toggle_activity_view_surfaces_honest_guidance`;
`side_chat_binding_resolves_ctrl_alt_s_to_the_side_chat_action`,
`activity_view_binding_resolves_ctrl_alt_u_to_visible_guidance`,
`next_unread_chat_binding_resolves_ctrl_alt_a_and_jumps`,
`clear_all_unread_binding_resolves_shift_escape`,
`toggle_thread_unread_binding_resolves_ctrl_shift_u`,
`next_unread_chat_follows_sidebar_order_cyclically`.

Binary sanity: `/home/z/flauz/target/release/codexrs` (43,115,064 bytes) — `codexrs info`
exit 0 prints `reference: OpenAI.Codex 26.721.3996.0 x64 / codex-cli reference:
0.146.0-alpha.3.1 / git-processes=1` (BOOT-C1 runtime-confirmed).

## 2. Fresh GUI scene evidence (LINUX_GUI_LAB, sealed recipe)

Ten scenes were executed (scripts + frames + md5 files + VLM transcripts archived under
`docs/research/evidence/rwo-021/`); the load-bearing results:

| Scene frame | md5 | What it proves |
| --- | --- | --- |
| `a01-ws-baseline.png` | `6188bc9c…` | Workspace-seeded restore path (D10b pattern): fixture project selected, sidebar complete, main empty state — plus the no-runtime bounded-reconnect footer state (`Retry 4 in 8s` — the documented 1/2/4/8/16/20 s timer, BOOT-C5). |
| `a02-ctrl-p-palette.png` | `3d1f0501…` | **Ctrl+P opens the "Search files" palette** (placeholder "Search files", section "Files", "Type to search for files", input focused) — KAX-C1. |
| `a03-ctrl-p-escape.png` | `6188bc9c…` | **Byte-identical to a01** — Escape closes the palette cleanly (D10b fix-proof reproduced). |
| `a04-ctrl-slash-overlay.png` | `5e43f84c…` | Ctrl+/ overlay opens: title "Keyboard shortcuts", search "Search shortcuts", groups **Chat (6 rows)** → Navigation (7 visible of 21) — KSR-C1/C2/C3. |
| `a06-ctrl-g.png` | `bf910d1a…` | Ctrl+G "Search chats" palette with "Back to commands" + honest "No matches" — PRJ-C7. |
| `a08-feedback-dialog.png` | `c9459c6d…` | Palette → "feedback" → **"Share feedback" dialog**: five categories (+Bug/+Bad result/+Good result/+Safety check/+Other), "Include current ChatGPT session logs" **checked**, "Share details (required)", Submit disabled — FBK-C1/C2/C3/C5. |
| `a17–a20-d11b-*.png` | 4 distinct md5s | **All four D11b statuses reproduced verbatim**: "Select a chat before marking it unread." / "No chats need attention." / "Activity view is not available yet. Use "Next chat needing attention" to jump to unread chats." / "No unread chats" — ACT-C3. All four frames differ (never silent). |
| `f04-sidebar-pullrequests.png` | `82e1353b…` | Pull-requests page: **gh-missing toast "GitHub CLI (gh) is not installed"** + empty state "GitHub CLI setup required" with "Install GitHub CLI"/"Check again" — NTR-C5. |
| `f05-sidebar-plugins.png` | `c6e3fdbc…` | Plugins page renders the marketplace surface ("Make Codex work your way", Plugins/Skills tabs, Manage/Create) — NTR-C5/ev-04 class. |
| `f07-settings-search-import.png` | `fea…` (differs from f06) | Settings search "import" → nav filtered to **exactly the Personal group + Import row** — SET-C3 pass condition met exactly. |
| `j01/j02-composer+ctrl-enter` | `81bda658` → `552dc99f` | **Typed `/feedback` in the composer + Ctrl+Enter (the `PressEnter{secondary:true}` submit chord, gpui `secondary`=Ctrl on Linux) opens the "Share feedback" dialog** — FBK-C5 slash leg. |
| `j03-overlay-filtered-search.png` | `9c75d804…` | Overlay query "search" filters rows at runtime (Search chats/Search files/Open command menu/Find/…) — the shared `keyboard_shortcut_settings_matches` matcher GUI-verified. |
| App liveness | — | **App alive at the end of every scene** incl. the no-runtime runs (BOOT-C5: no fatal exit; degraded-but-alive surface; bounded reconnect backoff visible). |

Scene-mechanics notes (honest limits, not product defects): five earlier attempts at the
composer leg (plain Return / Shift+Enter / Alt+Enter) produced no frame change — the composer
is a multi-line input whose submit chord is `secondary-enter` = **Ctrl+Enter on Linux**
(gpui `platform/keystroke.rs:143-149` maps `secondary` → `modifiers.control` on non-macOS);
the working chord is recorded above. The shortcuts-**settings-page** search input could not be
focus-clicked reliably across five attempts (keystrokes repeatedly landed in the left
settings-nav search — window-offset calibration); the matcher itself is GUI-verified through
the overlay (j03) and source-verified on the page (KSR-C4 note).

## 3. Per-criterion results (rubric §4.1–§4.13, Appendix D blocks)

Verdict vocabulary per rubric-r2 §1.4: `verified` / `defect-found` / `not-run`. Reviewed SHA
for all blocks: `876bbe862e0626a1b6788038e5ebf4347c918be2`. Line anchors below were
re-located at this SHA (they drift from the r2 base as expected; symbols are the stable
reference).

### 4.1 Runtime bootstrap — PR §5.1, `complete`

#### BOOT-C1 — verified
- Step class: S (source-read) + binary `info` probe.
- Evidence: `crates/codex-core/src/lib.rs:228-235` `STABLE_REFERENCE` — `cli_version
  "0.146.0-alpha.3.1"`, `cli_sha256
  "39e9e041ea33ac34aad9578adfe660c5c7a6dc8f82620b77623960f9352a6ef3"` — exact pin;
  `codexrs info` prints both (§1).
- Version layer honored: 26.721.

#### BOOT-C2 — verified
- Step class: S. `crates/codex-platform/src/lib.rs:262-284` `resolve_codex_binary`: explicit
  arg → `CODEX_RS_CODEX_BIN` → `#[cfg(windows)]` stable cache (hash-gated) → `#[cfg(windows)]`
  npm candidate (`is_file`) → bare `codex`/`codex.exe`. Order matches PM:75; the Windows arms
  were read (not skipped).
- Version layer honored: 26.721.

#### BOOT-C3 — verified (with a coverage note)
- Step class: S + U. Comparison consumer on the live spawn path:
  `windows_stable_codex_cache_candidate` (lib.rs:287-311) gates the cache on
  `sha256_matches(destination, reference.cli_sha256)`, verifies the packaged source hash
  before copying, and re-verifies after copy — mismatch refuses (returns `None` → falls to
  the next arm), never silent acceptance. Spawn path: `crates/codex-app/src/backend.rs:10819`
  `resolve_codex_binary(None)` → `AppServerConnection::spawn`. Focused test:
  `sha256_verification_is_streamed_and_exact` (lib.rs:464+, `#[cfg(windows)]` — runs on the
  windows-latest matrix) + `packaged_codex_candidate_points_to_the_pinned_stable_cli`.
- **Note (advisory, not scored a defect):** no dedicated test exercises the
  mismatch→fall-through branch itself (the refusal path is `unwrap_or(false)` fall-through);
  the comparator's exactness is tested. Follow-up WO candidate (§6).
- Version layer honored: 26.721.

#### BOOT-C4 — not-run (EQ-1) + archived evidence verified
- The "app-server online" footer requires a resolvable runtime; this lab has none (EQ-1).
  The no-runtime state is captured instead (a01: "Connection failed"/"Retry 4 in 8s") and the
  app stays alive (BOOT-C5). Archived evidence `ev/06`
  (`flauz/06-shell-sidebar-footer-appserver-online.png`) exists and is the cited
  runtime-observed record; it was not re-run here.
- Version layer honored: 26.721; honest reason recorded.

#### BOOT-C5 — verified (GUI)
- Step class: G. Both fresh boots (seed + scene) with no resolvable runtime: no fatal exit,
  degraded-but-alive entry/workspace surface, bounded reconnect backoff visible and matching
  the documented 1/2/4/8/16/20 s schedule ("Retry 4 in 8s" — attempt 4 → 8 s). Frames a01,
  b01-class; app alive at every scene end.
- Official contrast honored as ref+1 (LX fatal-without-runtime is the official bound, not the
  Flauz bar).

### 4.2 Multi-root workspace handling — PR §5.6, `complete` (regression control)

#### MRW-C1 — verified
- Step class: S (sweep). Project/workspace path model is native `std::path` end-to-end:
  `LocalProjectSummary { path: PathBuf, folders: Vec<PathBuf> }` (core lib.rs:915-928),
  storage `encode_path`/`decode_path` raw `OsStr` bytes (storage lib.rs:869-890), fuzzy
  file-search plumbing over `PathBuf` roots. The only `file://`/URL tokens in `crates/**`
  are markdown-link sanitizers, plugin-logo URL validation, and test fixtures — **no browser
  path shim** on any project path (override 9 respected: this is the Windows-white-screen
  control, not the multi-folder capability).

#### MRW-C2 — verified (local) / windows-latest leg not-run (EQ-2, SFR-C7)
- Standing tests in-tree: `related_folders_join_file_search_while_primary_drives_cwd_and_skills`
  (core lib.rs:25702) exercises multi-root paths through the model with `cfg!(windows)`
  drive-letter branches; `extended_windows_paths_are_displayed_normally` (ui.rs:51614,
  `#[cfg(windows)]`); plus `C:\\repo`-style fixtures in cross-platform tests (e.g. the
  side-chat state test, ui.rs:48716-48722). All local-matrix instances green (§1). The
  windows-latest CI matrix is the load-bearing leg for the Windows row and could not be
  queried from this sandbox (SFR-C7 bound); the ledger records CI green at the merge SHAs
  (`[historical-record]`).

#### MRW-C3 — verified
- `docs/known-failures.md` row 1 present and accurate ("Native `Path`/`PathBuf`; no browser
  path shim | Implemented").

### 4.3 Git process hygiene — PR §5.7, `complete` (regression control)

#### GPH-C1 — verified
- `RuntimePolicy::default().git_debounce = Duration::from_millis(300)`
  (platform lib.rs:381); `GitRefreshDebouncer` (app backend.rs:1770-1790) holds a **single
  pending slot** — new schedules replace the pending entry (notification coalescing,
  latest-wins) — wired on the live `Effect::RefreshGit` path (backend.rs:3719-3726) and
  drained per loop iteration (backend.rs:3820-3822). Not dead code.

#### GPH-C2 — verified
- One-backend-op-at-a-time is enforced structurally: the single-threaded backend event loop
  drains at most one due git refresh per iteration and `refresh_git` runs `git_snapshot`
  synchronously on that loop (backend.rs:13340+); `max_parallel_git_processes =
  NonZeroUsize::MIN` = 1 (platform lib.rs:382), asserted by
  `default_policy_prevents_parallel_git_storms` (lib.rs:402). All git spawns route through
  the platform git module / the supervised backend loop.

#### GPH-C3 — verified
- Standing tests green locally (§1): `default_policy_prevents_parallel_git_storms` +
  `git_refreshes_are_coalesced_until_the_debounce_expires` (backend.rs:22169 — asserts
  not-due at 399 ms, due at 400 ms with generation 2/latest cwd, empty afterwards). CI at the
  review SHA: see SFR-C7 bound.

#### GPH-C4 — verified
- KF row 4 present and accurate (300 ms debounce, coalescing, one backend Git operation at a
  time | Implemented).

### 4.4 Marketplace admin-disabled install — PR §5.8, `complete`

#### MKT-C1 — verified
- `plugin_installability(availability, install_policy)` (app backend.rs:567-571): the
  `DISABLED_BY_ADMIN` string survives end-to-end as a distinct `disabled_by_admin` flag
  (never collapsed to generic "unavailable"); test
  `plugin_installability_preserves_admin_disabled_state` (backend.rs:21364) asserts
  `(false, true)`.

#### MKT-C2 — verified
- Catalog actions: install button `.disabled(any_pending || !installable)` with
  `.tooltip("Access is turned off by your admin")` in the admin-disabled branch
  (app ui.rs:32221-32236) — both the disabled state and the exact recovered tooltip.

#### MKT-C3 — verified
- Details surface: `Button … .label(… if plugin.disabled_by_admin {
  "Disabled by admin".to_owned() } …)` (ui.rs:30420-30425) — exact string.

#### MKT-C4 — not-run (no seedable fixture) + unit-test verified
- No `DISABLED_BY_ADMIN` catalog fixture exists (marketplace content comes from live
  marketplace sources; no wo-p1-003-style seed device covers it). Recorded per the rubric's
  honest path; the availability model + render arms are unit/source-verified above.

### 4.5 Keyboard shortcut reference — PR §5.10, `complete`

#### KSR-C1 — verified (GUI + source)
- GUI: Ctrl+/ opens the shortcuts overlay on the real binary (a04: title "Keyboard
  shortcuts", search "Search shortcuts", Chat → Navigation groups). Source: registry id
  `showKeyboardShortcuts` (ui.rs:3229-3230), dispatcher arm `"showKeyboardShortcuts" =>
  self.toggle_keyboard_shortcuts(...)` (ui.rs:10920), Help/menu row (ui.rs:13871+), and the
  `ShowKeyboardShortcutsShortcut` binding at ui.rs:5022 with its `on_action` handler at
  ui.rs:43655; discoverability test at ui.rs:49898 ("must remain discoverable").

#### KSR-C2 — verified (source + GUI viewport)
- The overlay filter is present and effective-only: `ACTIVE_KEYBOARD_SHORTCUTS.iter()
  .filter(|item| item.group == group).filter(|item| !
  self.effective_keyboard_shortcuts(item).is_empty())` (ui.rs:39547-39550). Recount at this
  base: registry = **76 items**, **25 empty-default ids** (list recorded in §5) →
  **source-derived advertised = 51**. GUI: a04's Chat group shows **6 rows = the exact
  source-derived Chat count (6)**; Navigation shows 7 rows in the viewport (truncated from
  21 below the fold — scroll-bound counted honestly). No stale 3c9f113 counts reused.

#### KSR-C3 — verified (source + GUI)
- `KeyboardShortcutGroup::ALL: [Self; 8]` = Thread, Navigation, Panels, Workspace, Skills,
  Configure, App, General (ui.rs:2698-2706) with `label()` mapping Thread→"Chat",
  Workspace→"Project" (ui.rs:2709-2720) — the visible order is exactly **Chat → Navigation →
  Panels → Project → Skills → Configure → App → General**; the modal iterates `ALL` in order
  (ui.rs:39546). a04 confirms the first two groups in order. No silently reordered stable
  group; the Thread/Workspace enum names are display-mapped to the stable labels (explicitly
  inventoried here).

#### KSR-C4 — verified (render GUI + filter source/matcher GUI; input-focus sub-leg not-run)
- The editable settings page renders on the real binary (frames b05/f08/g03/h04: heading
  "Keyboard shortcuts", "Search shortcuts" input, "Keys" pill, per-row key chips + edit
  affordances, e.g. "New chat — Start a new Codex chat — Ctrl N"). Filtering:
  `render_keyboard_shortcut_settings` filters items through
  `keyboard_shortcut_settings_matches(item, &effective, &query, search_by_keys)`
  (ui.rs:33093-33112) — the same matcher the overlay uses, GUI-verified at runtime via the
  overlay query "search" (j03: filtered row set). The settings-page input focus could not be
  click-targeted in this lab (5 attempts; §2 note) — that micro-leg is recorded not-run with
  the scene-mechanics reason; archived `ev/11` remains the runtime-observed record for it.

#### KSR-C5 — defect-found (documentation-class)
- Step class: S (battery re-run) + documentation check.
- **Defect:** the advertised bindings `archiveThread` (Ctrl+Shift+A), `toggleThreadPin`
  (Ctrl+Alt+P), `renameThread` (Ctrl+Alt+R) remain silent no-ops with no chat selected
  (source at this base: `archive_selected_chat` ui.rs:8899-8903 and
  `toggle_selected_chat_pin` ui.rs:8905-8909 — bare `if let Some(...)` with no status;
  `rename_selected_chat` ui.rs:9052-9054 — `let Some(...) else { return; }`), and **the
  PR §5.10 row does not name them as residuals** (its Gap cell documents F-A4 only). They
  are documented in the ledger's WO-R-SWEEP findings-of-record and the parity §10 changelog,
  but the rubric's pass condition requires the residual in the parity row. No NEW silent
  state was found (F-A4 guard unchanged at ui.rs:8870-8872 and documented; F-A6 unchanged at
  ui.rs:10963-10965; F-D1 unchanged at ui.rs:8205-8210; F-D2's runtime-not-ready leg
  unchanged at core lib.rs:13324-13326 — all ledger-documented).
- Repro (source): with no selected chat, press Ctrl+Shift+A on the entry surface — no status
  message renders (contrast `toggleThreadUnread`'s honest "Select a chat before marking it
  unread." from the same surface, a17).
- Severity: low (documentation gap; behavior matches the official "safely do nothing"
  contract class but is undocumented **in the row**). Fix path: name F-A1/A2/A3 in the
  §5.10 Gap cell or deliver honest statuses in a new WO.

### 4.6 Feedback — PR §5.10, `complete`

#### FBK-C1 — verified
- `FeedbackClassification` (core lib.rs:4370-4376) has exactly five variants; `ALL: [Self; 5]`
  (4379-4385); `as_str` literals exactly `bug`, `bad-result`, `good-result`, `safety_check`,
  `other` (4388-4396). GUI-confirmed: the dialog shows +Bug/+Bad result/+Good result/+
  Safety check/+Other (a08, j02).

#### FBK-C2 — verified
- Submit blocked without classification (button `.disabled(… || feedback_classification
  .is_none() || …)` ui.rs:42365-42370; `submit_feedback` early-return ui.rs:10177-10180) and
  without details (reduce guards: empty → "Share details before submitting.", length →
  "Feedback details are too long." core lib.rs:16230-16248; `MAX_FEEDBACK_DETAILS_BYTES`
  bound). Focused test `feedback_upload_is_validated_bounded_and_single_flight` (core
  lib.rs:33014) green. GUI: Submit renders disabled in the fresh dialog (a08, j02).

#### FBK-C3 — verified
- `open_feedback_modal` sets `self.feedback_include_logs = true` at every open path
  (ui.rs:10096-10105). GUI-confirmed twice: checkbox "Include current ChatGPT session logs"
  checked in both a08 (palette path) and j02 (typed-`/feedback` path).

#### FBK-C4 — verified
- The dialog render branch (ui.rs:42257+, read through the modal close) contains no
  browser-tabs control; zero browser-related tokens in the modal render block.

#### FBK-C5 — verified (both legs, GUI)
- Palette leg: Ctrl+K → "feedback" → Return opens the "Share feedback" dialog (a07 query
  frame + a08 dialog). Slash leg: composer typed `/feedback` + Ctrl+Enter opens the same
  dialog (j01 typed → j02 dialog; the submit chord is `secondary-enter` = Ctrl+Enter on
  Linux per gpui `platform/keystroke.rs:143-149`). Help/menu path: ui.rs:13897.

#### FBK-C6 — verified (source + unit); e2e leg not-run (EQ-1)
- `Effect::SubmitFeedback` handler (app backend.rs:5956-5988): typed
  `feedback/upload` via `upload_feedback(FeedbackUploadParams{…})` (platform app_server.rs:
  1492), app-version tag from `env!("CARGO_PKG_VERSION")`, `include_logs`, `thread_id`
  when present, bounded response (`bounded(response.thread_id, 512)`), exact retry copy
  "We couldn't submit your feedback. Please try again in a moment.". Wire test asserts the
  exact JSON (protocol lib.rs:4049-4063, green). A real authenticated submission needs a
  runtime (EQ-1) — recorded not-run for the e2e leg only.

### 4.7 Stable-failure regression controls — PR §5.10, `complete`

#### SFR-C1 — verified (independently cited)
- Per MRW-C2 (same tests, independently re-cited here): standing multi-root control tests
  green locally at this base; windows-latest leg per SFR-C7 bound.

#### SFR-C2 — verified
- Zero `read_to_string` occurrences in `crates/codex-{app,core,platform,storage}/src`
  (sweep at this base) — live history is queried through bounded app-server pages only; all
  readers bounded by construction (KF "Current budgets" table stands).

#### SFR-C3 — verified
- `ThreadListParams::state_db_page` always sets `use_state_db_only: true` (protocol
  lib.rs:624-636); `list_threads` **refuses** requests without the flag ("thread/list must
  use state DB only", platform app_server.rs:1062-1066); `validate_page_limit` bounds every
  page (1..=100). Wire test `thread_list_is_bounded_to_state_database_metadata` asserts the
  exact JSON incl. `useStateDbOnly:true` (protocol lib.rs:4074-4083, green).

#### SFR-C4 — verified (independently cited)
- Per GPH-C3: `default_policy_prevents_parallel_git_storms` +
  `git_refreshes_are_coalesced_until_the_debounce_expires` green locally at this base.

#### SFR-C5 — verified
- `crates/codex-platform/src/process.rs`: Windows Job Object arm read in the `#[cfg(windows)]`
  code (win32job `limit_kill_on_job_close` + `assign_process`, process.rs:218-262; kill-on-
  close drop on error paths); supervision: `supervise_detached_process`, bounded
  `MAX_DETACHED_PROCESS_SUPERVISORS = 8`, `reap_process_until` (1 s bound), graceful shutdown
  + one bounded fallback. **No `taskkill` token anywhere in `crates/`** (AGENTS.md invariant).

#### SFR-C6 — verified
- Owned state narrowly scoped: codexRS does not duplicate provider logs/raw payloads (KF row
  6 stands; bounded owned-storage page 500 rows; preference value 64 KiB; all owned writes
  bounded per the budgets table). No unbounded log-write path found on owned surfaces
  (spot-sweep of the storage writer + logging sites).

#### SFR-C7 — not-run (honest reason) + local-matrix substitute recorded
- The CI API could not be queried from this sandbox under the task's token-handling rules
  (the credential is bound to git command lines only), so a fresh both-matrices CI citation
  at `876bbe8` is not produced here. Substitute evidence: the full release battery green
  locally on this Debian-13 Linux sandbox (§1, 639/0/2), plus the ledger's `[historical-
  record]` CI-green citations at the constituent merge SHAs (PRs #16/#17/#18/#20/#23/#24/#25
  — each recorded "CI green both matrices"). The Lead's integration station can supply the
  authoritative fresh CI citation.

### 4.8 Side chats — PR §5.1 (C2-audit row), `complete` (WO-P2-006 @ `5287c29f`)

#### SCH-C1 — verified
- Registry metadata exact (ui.rs:2880-2885): id `openSideChat`, title "Open side chat",
  description "Start a temporary side conversation without leaving this chat", Thread group,
  default `CmdOrCtrl+Alt+S`. Ownership test
  `side_chat_binding_resolves_ctrl_alt_s_to_the_side_chat_action` (ui.rs:48696-48726, green
  per-test): `owners == ["openSideChat"]`, registry membership, and `Action::OpenSideChat`
  leaves `selected_task_id` unchanged.

#### SCH-C2 — not-run (EQ-1) + archived evidence verified from primary artifacts
- Thread creation needs a runtime (EQ-1). The archived D9 evidence is intact and matches the
  claims: `wo-p2-006/vlmd9a.json` image-004 read records the side panel text "Ask a side
  question without interrupting this chat." with the main chat still selected
  ("side chat fixture main").

#### SCH-C3 — not-run (EQ-1) + archived evidence verified
- `vlmd9a.json` image-007 (aftermath): "A new item 'quick aside question' appears above it…
  'side chat fixture main' remains highlighted/selected" — the non-selection semantics
  exactly as the ledger claims.

#### SCH-C4 — verified
- `/side` guard: `if self.state.selected_task_id.is_none() { return false; }` (ui.rs:8134-
  8137) — returns *unhandled*, i.e. the text falls through to the composer submit path
  (message fallback; the sweep's recorded WORKS semantics — not a silent `return true`
  swallow). Menu row hidden without a task (`show_side_command = slash_availability.side &&
  …`, ui.rs:23970-23972; `side: selected_task_id.is_some()`, ui.rs:8303).
  `side_slash_command_resolves_with_its_guard` (ui.rs:48729+) green.

#### SCH-C5 — verified (state test) + GUI leg not-run (EQ-1)
- The WO-P2-006 state test asserts selection/active-turn untouched (part of
  `side_chat_binding_resolves_ctrl_alt_s_to_the_side_chat_action`, ui.rs:48716-48725, green).
  Close-returns-to-main frames: archived D9 008 (`d9-008-side-closed.png`) present; fresh
  GUI not-run (EQ-1 — needs a created side chat).

#### SCH-C6 — verified
- Recounted at this base (see §5): `openSideChat` ∈ `KEYBOARD_SHORTCUT_COMMAND_IDS`
  (`[&str; 76]`, 76 elements counted); `MAX_KEYBOARD_SHORTCUT_COMMANDS = 76`
  (core lib.rs:133) == registry length == `ACTIVE_KEYBOARD_SHORTCUTS` item count (76,
  set- and order-equal). The SWEEP 71-vs-72 quirk stays superseded (override 18).

### 4.9 Projects and chats — PR §5.1, `partial` (claimed slice only)

#### PRJ-C1 — verified
- `search_threads` validates the page limit (platform app_server.rs:1079-1087 →
  `validate_page_limit` 1..=100); thread metadata pages bounded (KF budgets: 20 default /
  100 max); state-side search bounded through the same request layer. Wire tests green
  (protocol battery).

#### PRJ-C2 — verified (guards = the documented F-A1/A2/A3 class)
- Archive/rename/pin slice present with palette/sidebars wired; the no-selection guards are
  exactly the sweep-documented silent states (see KSR-C5 — the documentation gap is recorded
  there, once). Tests green (battery). No silent states beyond the documented set found in
  the battery re-run.

#### PRJ-C3 — verified
- `LocalProjectSummary.folders: Vec<PathBuf>` (core lib.rs:915-928, cap
  `MAX_LOCAL_PROJECT_FOLDERS = 16` at lib.rs:118, primary never a member);
  `normalize_local_project_folders` (lib.rs:7509). Focused WO-P1-003 tests green:
  `local_project_related_folders_add_remove_and_guard_duplicates` (25214),
  `local_project_primary_swap_rekeys_the_project_and_keeps_identity` (25487),
  `local_project_primary_swap_rejects_another_projects_primary` (25644),
  `related_folders_join_file_search_while_primary_drives_cwd_and_skills` (25702).

#### PRJ-C4 — not-run (EQ-1/lab bound) + archived evidence verified
- The D6 six-assertion scene needs the runtime-enabled lab (thread + native-picker bound);
  the archived `wo-p1-003/` evidence (D6 frames + seed + vlm-reads) is present in the tree.
  Fresh re-run not possible here — recorded not-run with the EQ-1 reason.

#### PRJ-C5 — verified
- Discovery contract holds at the call sites:
  `related_folders_join_file_search_while_primary_drives_cwd_and_skills` asserts
  `SelectWorkspace` composes new-chat cwd on the primary only and `RefreshSkills` on
  primary-only roots, while fuzzy file search roots = [primary, related…] (core
  lib.rs:25702-25760, green).

#### PRJ-C6 — verified
- Storage migrations 0→1→2→3→4→5 all present (storage lib.rs:692-784+); schema now **v5**
  (v4 = WO-P1-003 folders; v5 = WO-P2-009 browsing history — evolution recorded, not a
  defect); legacy single-path projects load primary-only (`folders` empty). Storage battery
  19/19 green.

#### PRJ-C7 — verified (GUI)
- a06: Ctrl+G opens the "Search chats" palette ("Back to commands", honest "No matches" on
  the empty lab); sidebar Chats list renders on the entry surface (a01). Archived ev/06,
  ev/08 present.

### 4.10 Settings shell — PR §5.9, `partial`

#### SET-C1 — verified (GUI)
- a11/f06: "Back to app" affordance, search input "Search settings...", Personal/
  Integrations/Coding group structure; the no-results state is GUI-observed too (f09-class
  nav "No results found" when the nav search matches nothing).

#### SET-C2 — verified
- `enum SettingsSection` = exactly **18 variants** (ui.rs:2548-2567; CodeReview/Worktrees/
  ArchivedChats contextual/hidden); `DEFAULT_NAV_SECTIONS: [Self; 15]` (ui.rs:2576-2592,
  `#[cfg(test)]`); settings frame shows the 3-group nav. Counts match source.

#### SET-C3 — verified (GUI, exact pass condition)
- f07: settings search typed "import" → nav filtered to **exactly the Personal group header
  + the Import row** (VLM read verbatim). Query crosses groups as required.

#### SET-C4 — verified
- Registry coverage + filtering tests green (battery): `palette_indexes_every_default_nav_
  settings_section` (ui.rs:49658), `palette_settings_queries_resolve_to_settings_sections`
  (49671). GUI: a05 shows the palette Settings group (General, Appearance, Keyboard
  shortcuts, Usage & billing, Computer use, Profile, Import, Browser visible); a07/a08 show
  query → resolve → navigate. Archived wo-p2-004/ (15 captures) present.

#### SET-C5 — verified (source) / GUI leg not-run
- The bounded sections render honest guard states in source (e.g. Hooks empty state); the
  Hooks page was not captured in this pass (scene budget) — archived wo-p2-004 evidence
  remains the GUI record. No fabricated content found in the render arms read.

#### SET-C6 — verified
- PR §5.9 re-read at this base: the 18-vs-26 host-contract bound wording stands; every LX
  citation in the row carries the `ref+1` label; Pets never counted as target.

### 4.11 Keyboard and accessibility — PR §5.10, `partial`

#### KAX-C1 — verified (GUI, D10b pattern reproduced)
- Workspace-seeded (recent_workspaces restore path, D10b seeding): Ctrl+P opens the
  "Search files" palette (a02, md5 differs from baseline) and Escape closes **byte-identical**
  (a03 == a01, md5 `6188bc9c…`). Ctrl+K/Ctrl+Shift+P/Ctrl+G palettes all open (a05/a07/a06).

#### KAX-C2 — verified (recount at base)
- Registry ↔ `ACTIVE_KEYBOARD_SHORTCUTS` **set-equal and order-equal** (76/76, computed at
  this base); zero dead command ids (§5); `MAX_KEYBOARD_SHORTCUT_COMMANDS = 76` == registry
  length == const-array length.

#### KAX-C3 — verified
- Live `OpenFileSearch` declarations/bindings: **zero** (the only token hit at this base is
  the explanatory comment inside the regression test itself, ui.rs:52079).
  `ctrl_p_routes_to_the_search_files_command` (ui.rs:52076+) present and green (battery).

#### KAX-C4 — verified
- The `open_command_palette` Files-mode guard is present and unchanged
  (`if mode == PaletteMode::Files && !self.has_local_workspace() { return; }`,
  ui.rs:8870-8872) and the F-A4 no-workspace residual remains documented (PR §5.10 Gap cell
  via the WO-P2-007 closure note + ledger). Behavior did not silently change.

#### KAX-C5 — verified (battery re-run)
- The SWEEP grep battery re-run at this base: silent-target functions present at the
  documented anchors (B2: archive/pin/rename at ui.rs:8899/8905/9052); Files-mode bail at
  8870; /review swallow at 8205; runtime-ready guard at core lib.rs:13324; pending-PR sites
  9 hits. No new dead palette entries (all 70 `PaletteCommand::ALL` rows dispatch — coverage
  tests green; F-A6 (CommitOrPush pending-PR silent skip) unchanged and still documented in
  the ledger findings-of-record).

#### KAX-C6 — verified
- PR §5.10 pending list intact at this base (remaining stable commands, focus order,
  screen-reader labels, reduced motion — none quietly claimed complete).

#### KAX-C7 — verified (GUI + source)
- a05: palette Settings group entries render with title/description mirroring the nav rows
  (General, Appearance, Keyboard shortcuts, Usage & billing, Computer use, Profile, Import,
  Browser visible in frame); Personalization alignment covered by the WO-P2-004 tests
  (green).

### 4.12 Notifications and tray — PR §5.10, `partial`

#### NTR-C1 — verified (source) / GUI leg not-run (EQ-1)
- Completion → banner path in source: `background_completion_task_ids` (bounded VecDeque;
  transition-gated push at ui.rs:7935-7945) → banner renders "A background chat completed."
  with **Open** (→ `Action::OpenBackgroundCompletion`, only when the task still exists) and
  **Dismiss** (→ `Action::DismissBackgroundCompletion`) buttons (ui.rs:43643-43745). A
  completing background turn needs a runtime (EQ-1) — GUI leg not-run with that reason; the
  gh-missing toast (f04) is the adjacent error-banner class and is NOT counted as completion
  evidence (the NT-2 trap avoided).

#### NTR-C2 — verified
- `background_chat_running_count` (ui.rs:43838-43850): `.take(MAX_VISIBLE_THREADS)` bounded,
  **non-selected** chats only, `Running | WaitingForApproval` only; title reverts to
  `PRIMARY_WINDOW_TITLE` when none remain (43830-43836); `sync_window_title` also mirrors
  the count into the notification-area tooltip (ui.rs:7801-7812). Test
  `background_chat_window_title_counts_only_non_selected_running_chats` (ui.rs:47765, green).

#### NTR-C3 — verified
- `crates/codex-platform/src/desktop_notifications.rs`: Linux freedesktop path
  (`org.freedesktop.Notifications` D-Bus via zbus), worker thread + bounded(1) command
  channel — best-effort, non-blocking (drops when full).

#### NTR-C4 — verified
- Tray groups/badges/sounds remain pending in the row; Linux tray/global-shortcut
  unavailability stays documented (packaging row cross-ref intact).

#### NTR-C5 — verified (GUI)
- f04: the Pull-requests page shows the gh-missing toast **"GitHub CLI (gh) is not
  installed"** (with Dismiss) plus the honest "GitHub CLI setup required" empty state with
  "Install GitHub CLI"/"Check again" — the ev/03 class re-verified at this base. f05: the
  Plugins/marketplace surface renders (ev/04 class).

### 4.13 Activity view & unread attention — PR §5.10 (C2-audit row), `partial` (WO-P2-008 @ `00a3392`)

#### ACT-C1 — verified (per-test)
- All **7 core state tests green per-test** (§1 B-7). Source semantics match the claim:
  `needs_attention_task_ids` (core lib.rs:4768), capped push/FIFO drop at
  `MAX_VISIBLE_THREADS` (lib.rs:8904-8917), flagging on background completion **including
  failed turns** (test asserts `["background", "failing"]`, lib.rs:37599) and approval
  requests, visit clears (10704-10708), archive drops (10550+), session-only (not persisted).

#### ACT-C2 — verified
- The four registry entries match the exact metadata (ui.rs:2901-2905, 2978-2997):
  `toggleThreadUnread` "Mark chat unread" Thread `CmdOrCtrl+Shift+U`; `nextUnreadChat`
  "Next chat needing attention" Navigation `CmdOrCtrl+Alt+A`; `clearAllUnread` "Clear all
  unread indicators" Navigation `Shift+Escape`; `toggleActivityView` "Toggle Activity view"
  Navigation `CmdOrCtrl+Alt+U`. All **6 app binding/jump tests green per-test** (§1 B-7).
  Local test-mode gpui compile caused no OOM in this sandbox (the WO-P2-008 discipline was
  not needed — recorded for honesty).

#### ACT-C3 — verified (GUI, verbatim)
- Fresh D11b-class scene: all four keypresses on the real binary resolve visibly, frames
  all differ (md5 sequence in `rwo-021/scene-md5.txt`), and the VLM reads match the D11b
  verbatim statuses exactly: "Select a chat before marking it unread." (a17) / "No chats
  need attention." (a18) / "Activity view is not available yet. Use "Next chat needing
  attention" to jump to unread chats." (a19) / "No unread chats" (a20).

#### ACT-C4 — verified (unit) / GUI not-run (EQ-1)
- `next_unread_chat_follows_sidebar_order_cyclically` green (per-test). The ≥2-thread jump
  scene needs a runtime (EQ-1) — not-run with the D11 precedent bound.

#### ACT-C5 — verified (source + named coverage) / dot-on-row GUI not-run (EQ-1)
- Sidebar render path: medium-weight title + 6px primary dot when `needs_attention`
  (ui.rs:14659-14677). GUI dot-on-row needs a runtime-enabled lab — the documented D11
  residual; unit-covered as recorded.

#### ACT-C6 — verified
- PR §5.10 residual wording re-read at this base: Activity view surface absent by design
  (separate future WO; binding gives honest guidance — GUI-confirmed in a19); unread
  persistence across restarts stays `[unverified]`/session-state by design. The absence was
  NOT counted against parity and no surface existence was claimed.

## 4-bis. Input-surface invariants (RWO-021 task directive, at this base)

| Invariant | Result | Evidence |
| --- | --- | --- |
| Zero dead commands | **PASS** — 76/76 registry ids have live arms in `execute_keyboard_shortcut_command` (ui.rs:10831-10958): 65 single-pattern arms + `"thread1"…"thread9"` multi-arm + `"closeWindow" \| "quit"` multi-arm | computed at base (§5 method) |
| Every registry shortcut has a live handler | **PASS** — the live interceptor `handle_keyboard_shortcut_keystroke` (ui.rs:10598-10717) matches accelerators against `effective_keyboard_shortcuts` of enabled registry items and dispatches through the verified dispatcher | source-read of the full live path |
| `MAX_KEYBOARD_SHORTCUT_COMMANDS` == array length == 76 | **PASS** — constant `76` (core lib.rs:133); `KEYBOARD_SHORTCUT_COMMAND_IDS: [&str; 76]` with **76 counted elements** (lib.rs:138-211); `ACTIVE_KEYBOARD_SHORTCUTS` **76 items**, set- and order-equal with the const | counted at base |
| (Context) `PaletteCommand::ALL` | 70 elements (WO-P2-010: 51 → 70; `const ALL: [Self; 70]` ui.rs:3480-3551, 70 counted) — coverage tests green | counted at base |
| (Context) Empty-default (unadvertised) ids | 25 — unchanged from the sweep base | counted at base |

## 5. Verification method appendix (reproducible commands)

Registry integrity (from repo root):

```sh
# constant + const-array + registry counts and set-equality (python3 one-liner used)
rg -n "MAX_KEYBOARD_SHORTCUT_COMMANDS|KEYBOARD_SHORTCUT_COMMAND_IDS" crates/codex-core/src/lib.rs
rg -n "const ACTIVE_KEYBOARD_SHORTCUTS|const ALL: \[Self;" crates/codex-app/src/ui.rs
# dead-command sweep: every registry id must appear as a dispatcher arm literal
rg -n '=> self\.|=> window\.|=> \{' crates/codex-app/src/ui.rs   # (arm-by-arm read of 10831-10958)
# SWEEP battery re-runs (expected results at this base in parentheses):
rg -n "OpenFileSearch" crates/codex-app/src/ui.rs            # 1 hit — test comment only (was 2 live at 3c9f113)
rg -nE "fn archive_selected_chat|fn toggle_selected_chat_pin|fn rename_selected_chat" crates/codex-app/src/ui.rs  # 3
rg -n "PaletteMode::Files && !self.has_local_workspace" crates/codex-app/src/ui.rs   # 1
rg -n 'command == "/review"' crates/codex-app/src/ui.rs     # 1
rg -n "selected_thread_runtime_ready" crates/codex-core/src/lib.rs                   # 3
rg -n "pending_pull_request.is_some" crates/codex-app/src/ui.rs                      # 9
rg -nE "OpenCommandMenu|OpenChatSearch" crates/codex-app/src/ui.rs                   # 5
rg -n "fn keyboard_shortcut_command_enabled" crates/codex-app/src/ui.rs              # 1
rg -n "ShowKeyboardShortcutsShortcut" crates/codex-app/src/ui.rs                     # 4
```

The 25 empty-default (unadvertised) ids at this base: `copyConversationMarkdown, forkThread,
toggleReviewTab, toggleMaximizeSidePanel, forceReloadSkills, openSkills, keyboardShortcuts,
mcpSettings, personalitySettings, logOut, feedback, composer.submit, composer.addPhotos,
composer.addFiles, composer.toggleFastMode, composer.increaseReasoningEffort,
composer.decreaseReasoningEffort, composer.cycleReasoningEffort, composer.togglePlanMode,
git.commit, git.createPullRequest, git.createDraftPullRequest, git.createBranch,
git.mergePullRequest, git.openPullRequest`.

## 6. Defects found and follow-up WO candidates

1. **[DEFECT — documentation-class, low] KSR-C5:** F-A1/A2/A3 advertised silent bindings
   (archive/pin/rename with no chat selected) are not named as residuals in the PR §5.10 row
   (documented only in the ledger sweep entry + parity changelog). Repro: source anchors
   ui.rs:8899/8905/9052 + the contrast probe (Ctrl+Shift+U gives an honest status; Ctrl+Shift+A
   stays silent on the same surface). Fix: row wording, or honest statuses via a new WO.
   *(Also found independently by the sibling first dispatch — agreement recorded.)*
1b. **[DEFECT — documentation-class, low] §5.10 palette-registry staleness (KA row):** the
   Flauz cell still reads "verified stable registry subset (51-command palette registry at
   WO-P2-004…)" while the base includes WO-P2-010 (PR #25, `10b0c24`) which grew
   `PaletteCommand::ALL` 51 → **70** (counted at base, §4-bis). The 51 is true only as the
   WO-P2-004-era snapshot; the row was not updated on the wave-S merge. *(Found by the
   sibling first dispatch as "stale KA 51-vs-70 row"; confirmed independently here by the
   70-element count + `git log` provenance.)*
1c. **[DEFECT — documentation-class, low] Ledger wave-S status lag:** the ledger's WO-P2-009
   and WO-P2-010 sections still read "IN FLIGHT … Watcher armed; harvest on delivery" while
   both PRs (#24 `5302e7e`/`0eed8c3` browsing history, #25 `876bbe8`/`10b0c24` palette rows)
   are merged on main at the review base; the harvest/closure entries never landed. *(Found
   by the sibling first dispatch as "wave-S ledger lag"; confirmed here by section read at
   the base.)*
2. **[Follow-up WO candidate] BOOT-C3 coverage:** add a focused mismatch→fall-through test
   for the hash-gated cache arm (the comparator is tested; the refusal branch is not).
3. **[Follow-up WO candidate] F-D1/F-D2 residuals:** typed `/review` while unavailable still
   silently swallows (ui.rs:8205-8210) and `/compact`'s runtime-not-ready leg is silent
   (core lib.rs:13324-13326) — both ledger-documented; candidates for the WO-P2-007
   input-quality doctrine extension.
4. **[Observation, no action]** `PaletteCommand::ALL` grew 51→70 (WO-P2-010) and the schema
   moved v4→v5 (WO-P2-009) since the r2 rubric base — both recorded in the ledger; the
   rubric's KA-3 anchor (51) is stale by design (symbols over line numbers).

## 7. Honest limits of this verification (and sibling-dispatch reconciliation)

0. **Duplicate dispatch:** a sibling first dispatch of this work order (commit `a622b7b` on
   this branch) delivered `verification.md` from the same base but without a native build
   (missing system dev packages; battery = core/protocol/storage only; GUI evidence =
   archived-frame re-reads). This second dispatch (sysroot build + full battery + ten fresh
   GUI scenes) ships as `verification-r2.md`; both records coexist per the rubric →
   rubric-r2 house precedent and agree on all input-surface invariants and on finding 1.

1. No `codex` CLI runtime in this lab (EQ-1): BOOT-C4's online footer, SCH-C2/C3/C5 GUI legs,
   PRJ-C4, ACT-C4/C5 GUI legs, NTR-C1's completion banner, and FBK-C6's e2e submission leg
   are not-run with that reason; archived evidence for each was verified to exist and to
   match the claims (md5s/VLM reads re-checked where recorded).
2. The windows-latest CI matrix could not be queried (token-handling rules; SFR-C7): the
   local Linux battery (639/0/2) + the ledger's historical CI-green citations substitute.
3. Scene mechanics: the shortcuts-settings-page search input could not be focus-clicked
   (5 attempts; window-offset calibration drift). The page render, the source wiring, and
   the shared matcher (via the auto-focused overlay, j03) are verified; that one micro-leg
   is not-run (scene-mechanics reason, not a product finding).
4. VLM reads were neutral-prompted for every load-bearing frame; leading-prompt reads that
   disagreed with neutral re-reads were discarded (the b04/b06 sequence) — recorded here so
   Worker C can attack the right things.
