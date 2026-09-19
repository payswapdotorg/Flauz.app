# RWO-021 — Fresh end-to-end verification (WO-REVIEW-001 deep phase)

- **Task ID:** RWO-021 (one third of WO-REVIEW-001 — fresh verification pass)
- **Worker:** RWO-021 (review worker — verifies, does NOT fix; fixes are new work orders)
- **Date:** 2026-09-19 (UTC)
- **Base:** `main` @ `876bbe862e0626a1b6788038e5ebf4347c918be2` (verified with
  `git rev-parse '876bbe862e0626a1b6788038e5ebf4347c918be2^{commit}'` → the SHA
  itself; it is `origin/main` HEAD at clone time and the merge of PR #25 —
  post-wave-S base exactly as the brief described: PR #24 `0eed8c3`
  browsing-history, PR #25 `10b0c24` palette rows, PR #26 RWO-020 rubric all
  present in `git log`)
- **Branch:** `research/rwo-021` (off the base SHA)
- **Deliverable:** this file
- **Rubric executed:** `rubric.md` (13 rows / 71 criteria) executed with
  `rubric-r2.md`'s verified-anchor table (Appendix A), evidence-tree index
  (Appendix B), deferral guard (Appendix C) and the result-capture template
  (Appendix D). Per-criterion blocks below use the r2 template.
- **Review target:** `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md`
  (§5 rows, §8.1 counts — 8 `complete` rows re-verified from primary
  artifacts), `docs/research/FEATURE-PARITY-WORK-ORDERS.md` (ledger), the
  evidence tree `docs/research/evidence/`, and the live source tree at the
  base SHA.

## 0. Environment inventory (EQ resolution, binding)

| Item | Value at review time |
|---|---|
| Host | Linux x86_64 (kernel 5.10), **2 vCPU, 4.1 GiB RAM, no swap**, ~8 GB free disk |
| Rust | none preinstalled; installed during review via rustup 1.29.1 → toolchain **1.97.1** (matches `rust-toolchain.toml` channel; minimal profile) |
| `cargo` | `cargo 1.97.1 (c980f4866 2026-06-30)` |
| Display | `Xvfb`/`xvfb-run` binaries present; `DISPLAY` empty; **no `codex` CLI on PATH** (`command -v codex` → empty; no `CODEX_RS_CODEX_BIN`) → EQ-1 answer: `resolve_codex_binary` resolves to bare `codex` (not found) ⇒ thread-dependent GUI scenes fall to the D11b-class honest bound |
| System libs | `libxkbcommon`, `libxcb` present; **no `wayland-client` / pipewire (`libspa`) dev packages** and no root to install them ⇒ GPUI-wayland dependency chain cannot build in this sandbox |
| GUI lab | LINUX_GUI_LAB scenes **not runnable** (no build possible — see §1); archived-frame VLM re-reads used where applicable |
| CI access | GitHub API `check-runs` at the base SHA → **rate-limited** (unauthenticated, shared IP). The provided credential is scope-restricted to git clone/push URLs by the work order's binding token rules, so it may not be used to authenticate an API call. CI-green claims are therefore taken from the ledger's closure records (historical-record class), not re-verified live. |

## 1. Build and test battery (exact commands + exit codes)

| # | Command (run from repo root at the base SHA) | Result |
|---|---|---|
| 1 | `cargo build -p codex-app --release` | **exit 101** — `error: failed to run custom build command for 'libspa-sys v0.10.0'` (build-script panic; pkg-config cannot find the pipewire/libspa system dev package; non-root sandbox). Environment-bound failure, not a product defect. |
| 2 | `cargo test --workspace --release` | **NOT RUN** — blocked by the same system-dependency class as #1 for the `codex-app`/`codex-platform` legs (see #5), plus the 2-vCPU/4.1-GB/no-swap bound (the repo's own WO-P2-008 README records the same local-OOM discipline for gpui compiles). The strongest feasible battery (#4–#6) was run instead. |
| 3 | `cargo fmt --all --check` | **exit 0** — clean (AGENTS.md fmt gate holds at the base). |
| 4 | `cargo test -p codex-core --lib` | **exit 0 — 231 passed; 0 failed; 0 ignored** (finished in 0.27s after compile). Per-test output captured for every criterion-relevant test below. |
| 5 | `cargo test -p codex-platform --lib` | **exit 101** — `wayland-sys v0.31.11` build-script failure: `Package wayland-client was not found in the pkg-config search path`. NOT RUN (environment-bound). |
| 6 | `cargo test -p codex-protocol --lib` | **exit 0 — 60 passed; 0 failed** (incl. the `feedback/upload` and `thread/list` wire-contract tests). |
| 7 | `cargo test -p codex-storage --lib` | **exit 0 — 19 passed; 0 failed** (incl. the 594-MB-rejection control, workspace-folder round-trip/cascade/migration, and WO-P2-009's bounded browsing-history tests). |
| 8 | `cargo test -p codex-app --lib` | **NOT RUN** — requires the gpui system-dependency chain (#1/#5 class); also excluded by the repo's own local-OOM discipline (WO-P2-008 README, verification-matrix gate 2). The named app tests' existence and content were source-verified instead (§4, §5). |

Toolchain install commands (for the record): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh` (exit 0) → `sh /tmp/rustup-init.sh -y --default-toolchain 1.97.1 --profile minimal` → `rustup default 1.97.1`.

## 2. Input-surface invariants — WO-R-SWEEP re-run at the base (WO item 4)

Method: mechanical re-derivation from source (registry parsed structurally;
named shortcut constants `PREVIOUS_CHAT_SHORTCUTS`/`NEXT_CHAT_SHORTCUTS` and
`#[cfg(target_os = "macos")]` platform blocks resolved), plus the sweep's own
11-command verification battery re-run with `rg` at the base SHA.

| Invariant | Sweep value @ `3c9f113` | **RWO-021 value @ `876bbe8`** | Verdict |
|---|---|---|---|
| `MAX_KEYBOARD_SHORTCUT_COMMANDS` | 71 (quirk: 72 ids) | **76** (`codex-core/src/lib.rs:133`) | VERIFIED — the 71-vs-72 truncation quirk is cured; `KeyboardShortcutPreferences::normalized` (lib.rs:476-481) now iterates 76 ids bounded by MAX=76, so a fully-customized user no longer loses the last override |
| `KEYBOARD_SHORTCUT_COMMAND_IDS` length | 72 | **76** (declared `[&str; 76]` at lib.rs:138; **76** actual entries counted) | VERIFIED — MAX == declared == actual == 76 |
| Registry ↔ `ACTIVE_KEYBOARD_SHORTCUTS` set equality | set-equal, 72 | **set-equal, 76 ↔ 76** (0 diffs both directions; 0 duplicates) | VERIFIED |
| Dead commands (registry ids with no `execute_keyboard_shortcut_command` arm) | 0 | **0** — all 76 ids have match arms in `execute_keyboard_shortcut_command` (`ui.rs:10833`, string-arm match incl. the `"thread1" | … | "thread9"` alternation arm) | VERIFIED — zero dead commands |
| Attention commands present with documented defaults | (pre-WO-P2-008) | `toggleThreadUnread` `CmdOrCtrl+Shift+U`, `nextUnreadChat` `CmdOrCtrl+Alt+A`, `clearAllUnread` `Shift+Escape`, `toggleActivityView` `CmdOrCtrl+Alt+U` — all in registry + ACTIVE + armed | VERIFIED (AV-1) |
| Empty-default registry entries (unadvertised) | 25 | **25** (`shortcuts: &[]`; awk count over the ACTIVE block = 25, cfg-aware) | VERIFIED — consistent; advertised rows = **51** (76−25; was 47 — the +4 is the WO-P2-008 attention bindings, attributed) |
| `PaletteCommand::ALL` | 51 | **70** (`ui.rs:3480`, `const ALL: [Self; 70]`, 70 actual items, 0 dups) | VERIFIED with recorded drift — see KA-3 (growth +19 fully attributed to the WO-P2-010 delivery commit `10b0c24`, merged in this base; **not** silent drift) |
| Dead `OpenFileSearch` action (WO-P2-007 fix) | 2 hits (declared + bound) | **removed** — only a historical comment remains (`ui.rs:52079`); binding and declaration gone | VERIFIED — the fix landed as claimed |
| Sweep findings F-A1/A2/A3 (silent no-selection targets) | present | **persist** at `ui.rs:8899` (`archive_selected_chat`), `8905` (`toggle_selected_chat_pin`), `9052` (`rename_selected_chat`) | Persist as recorded; see KSR-C5 verdict (documentation gap) |
| F-A4 (Ctrl+P silent with no workspace) | present | **persists** at `ui.rs:8870` (`PaletteMode::Files && !self.has_local_workspace`) | Documented residual in the parity row Gap cell — not a new defect |
| F-A6 / F-D1 / F-D2 | present | **persist** (`pending_pull_request.is_some` guard `ui.rs:10963`; `command == "/review"` `ui.rs:8205`; `selected_thread_runtime_ready` guard `codex-core/src/lib.rs:13324`) | Persist as recorded; see KSR-C5 |
| 23 bound-but-never-handled GPUI actions + unbound `OpenBrowserTabShortcut` (structural watch list) | present | **persist** (e.g. `OpenCommandMenu`/`OpenChatSearch` still declared `ui.rs:1680-1681` + bound `4981-4983`; no `on_action`) | Unchanged watch list — menu-accelerator-label provider pattern; not dead commands (dispatch lives in the interceptor) |
| Slash catalog | 21 ids, all covered | **`const COMMANDS: [&str; 21]`** at `ui.rs:45730` | VERIFIED |
| Settings nav | 18/18 render arms | `SettingsSection` enum = **18** variants (`ui.rs:2548-2567`); render dispatch exhaustive | VERIFIED |
| Empty-default count battery (`awk … /shortcuts: &\[\]/`) | 25 | **25** | VERIFIED |

## 3. Evidence-artifact verification (WO item 3 — primary artifacts)

Executed by a read-only inventory subagent (Task 2-a) plus this worker's own
spot re-reads; repository untouched (HEAD and branch verified unchanged).

- **Frames:** 104 `.png` frames under `docs/research/evidence/` (0 webp/jpg).
- **md5 records:** 19 recorded md5s (`d10-md5.txt`, `ws-md5.txt`, `d11-md5.txt`,
  `d11b-md5.txt`) — **all 19 recomputed md5s MATCH; 0 mismatches; 0 recorded
  frames missing from disk**. Recompute command: `rg --files docs/research/evidence -g '*.png' | sort | xargs md5sum`.
- **VLM reads:** 9 archived `glm-5v-turbo` read files exist
  (`wo-p2-005/d8c4/vlm008.json`, `vlmd8c4.json`, `wo-p2-006/vlmd9a.json`,
  `vlmd9b.json`, `wo-p2-008/d11/vlm-d11-03.json`, `d11b/vlm-d11b-02..05.json`)
  + `vlm-reads.txt` transcripts in `wo-p1-003`, `wo-p2-004`, `codex-linux`.
- **Fresh VLM re-reads by RWO-021** (5 frames, this review, z-ai vision CLI —
  archived-frame re-verification, not new GUI runs):

| Frame | Claim under test | RWO-021 re-read result |
|---|---|---|
| `flauz/06-shell-sidebar-footer-appserver-online.png` | RB-3: boot footer "App-server online" | Footer reads **"App-server online"** — verbatim match |
| `flauz/15-shortcuts-overlay-ctrl-slash.png` | KS-2/KSR-C3: searchable overlay, stable category order | Title "Keyboard shortcuts", search field "Search shortcuts", headers top-to-bottom **Chat, Navigation** — matches the stable names (rendered via `KeyboardShortcutGroup::label()`, `ui.rs:2710-2721`) |
| `wo-p2-008/d11b/d11b-04-activity-view.png` | AV-3 verbatim status | Status line reads **"Activity view is not available yet. Use "Next chat needing attention" to jump to unread chats."** — verbatim incl. nested quotes; sidebar "No chats" (honest no-selection path) |
| `wo-p2-006/d9-005-side-typed.png` | SC-2/SC-3: side panel open, main chat still selected | Side panel open, composer placeholder "Ask a side question", **selected sidebar item "side chat fixture main"** — main chat still selected |
| `wo-p2-004/02-palette-top-settings-group.png` | KA-1: palette Settings group indexes nav sections | Palette placeholder "Search chats or run a command"; groups "Suggested" (2) + "Settings" (9 visible, continues below the fold) — consistent with the folded+scrolled capture set |

- **Anomalies (honest, non-defect observations):**
  - `wo-p2-008/d11/d11-02a-typed.png` exists on disk (52,714 B, md5
    `81a85b83290ae89b97cc82722d1729f6`) but is **absent from `d11-md5.txt`**
    (the README describes "the 7-frame md5 sequence"; the dir holds 8 frames).
  - `wo-p2-007/README.md` cites scene scripts `d10-ctrl-p-file-search.sh` and
    `d10b-ctrl-p-workspace.sh` "(parity-lab/scenes/)" that are **not shipped
    in-repo** (unlike other WO dirs which ship scenes in-dir).
  - The B2 `flauz/` frame set (ev/01–25) ships **no md5 list file** (the
    FLAUZ-REFERENCE-MATRIX says frames were md5-checked; the list itself was
    not archived).
  - Test citations in closure docs are mostly counts; the only verbatim
    test-function names repo-wide are
    `ctrl_p_routes_to_the_search_files_command` (WO-P2-007) and
    `composer_keeps_fork_picker` (WO-P2-005). RWO-021 independently located
    and ran/verified the others by name (see §4/§5).

## 4. The 8 `complete` rows — per-criterion verification (rubric.md §4.1-§4.8 with r2 anchors)

### 4.1 Runtime bootstrap — PR §5.1, `complete`

#### BOOT-C1 / RB-1 — verified
- Reviewed SHA: 876bbe862e0626a1b6788038e5ebf4347c918be2
- Step class run: S + U
- Evidence pointer: `crates/codex-core/src/lib.rs:228-235` `STABLE_REFERENCE`
  (`package_name "OpenAI.Codex"`, `package_version "26.721.3996.0"`,
  `cli_version "0.146.0-alpha.3.1"`,
  `cli_sha256 "39e9e041ea33ac34aad9578adfe660c5c7a6dc8f82620b77623960f9352a6ef3"`)
  == `reference/stable-26.721.3996.0/manifest.json` (`resources/codex.exe`
  sha256 identical); test `stable_reference_is_pinned` **passed** in battery #4.
- Version layer honored: 26.721

#### BOOT-C2 / RB-2 — verified
- Step class run: S
- Evidence pointer: `crates/codex-platform/src/lib.rs:262-284`
  `resolve_codex_binary`: explicit (non-empty-filtered) → `CODEX_RS_CODEX_BIN`
  → `#[cfg(windows)]` hash-gated stable cache → `#[cfg(windows)]` APPDATA npm
  candidate (`is_file` gate) → bare `codex`/`codex.exe`.
  `windows_stable_codex_cache_candidate` (`lib.rs:287-311`) is hash-gated on
  **both** the cached copy and any fresh copy from the packaged source
  (`sha256_matches(&source, …)` before copy; re-verified after copy).
- Version layer honored: 26.721

#### BOOT-C3 — verified (source + tests; Windows-arm execution bounded to CI)
- Step class run: S + U (Linux-runnable tests) + CI-citation bound
- Evidence pointer: mismatch-behavior tests
  `sha256_verification_is_streamed_and_exact` and
  `packaged_codex_candidate_points_to_the_pinned_stable_cli`
  (`codex-platform/src/lib.rs:462-475`, `451-460` — both `#[cfg(windows)]`,
  compiled/executed by the windows-latest CI matrix; not Linux-runnable here);
  `stable_reference_is_pinned` passed locally. CI check-runs at the base SHA
  not re-verifiable (API rate limit — §0); ledger closures record CI green on
  both matrices at the merge SHAs (historical-record).
- Version layer honored: 26.721

#### BOOT-C4 / RB-3 — verified (archived-frame VLM re-read); fresh GUI not-run
- Step class run: S + archived-G re-read (fresh G not-run: no build — §1 #1)
- Evidence pointer: source anchor `"App-server online"` at `ui.rs:14726`;
  VLM re-read of `evidence/flauz/06-…appserver-online.png` (§3) — footer
  verbatim. The ev/06 frame is B2-era (SHA d479c7b lineage); recorded as
  archived evidence, not a fresh capture at the review base.
- Version layer honored: 26.721

#### BOOT-C5 / RB-4 — not-run (fresh scene); archived evidence consistent
- Step class run: archived-G re-read only
- Evidence pointer: my VLM re-read of `wo-p2-008/d11b/d11b-04-activity-view.png`
  shows the degraded-but-alive surface (warning dialog "Couldn't connect to the
  Codex app-server … set CODEX_RS_CODEX_BIN, then retry" + visible status
  lines) — the no-runtime graceful-degradation shape, consistent with ev/24's
  record. Fresh re-run requires a build (blocked, §1).
- Version layer honored: 26.721 (Flauz-ahead difference; official fatal
  contrast is LX ref+1 and was not re-observed)

### 4.2 Multi-root workspace handling — PR §5.6, `complete` (regression control)

#### MRW-C1 / MW-1 — verified
- Step class run: S
- Evidence pointer: `LocalProjectSummary` (`codex-core/src/lib.rs:915-929`)
  with `path: PathBuf` and `folders: Vec<PathBuf>`;
  `normalize_local_project_folders` (`lib.rs:7509-7517`) enforces
  absolute-only, unique, never-equal-to-primary, cap
  `MAX_LOCAL_PROJECT_FOLDERS = 16` (`lib.rs:118`). No string/URL path shim
  found on the model path; zero `read_to_string` in `crates/` (AGENTS.md
  bounded-read invariant holds at source level).
- Version layer honored: 26.721

#### MRW-C2 / MW-2 — verified
- Step class run: U (storage battery) + S (app/windows test location)
- Evidence pointer: standing tests, all green in battery #7:
  `workspace_folders_round_trip_in_order_and_replace_all`,
  `removing_a_workspace_cascades_its_related_folders`,
  `version_three_storage_migrates_workspace_folders`,
  `workspace_folder_validation_rejects_relative_and_oversized_paths`;
  in `codex-app`: `edit_project_surface_renders_for_primary_and_related_folders`
  (`ui.rs:50676`) and `extended_windows_paths_are_displayed_normally`
  (`ui.rs:51612-51626`, `#[cfg(windows)]` — drive-letter + UNC extended paths
  through `display_path`; windows-latest CI is the load-bearing runner,
  Linux-run impossible here). windows-latest execution not re-verified live
  (API rate limit — §0); ledger records CI green both matrices at merge.
- Version layer honored: 26.721

#### MRW-C3 — verified
- Step class run: S (documentation)
- Evidence pointer: `docs/known-failures.md` row 1 present, status
  "Implemented", control wording matches the parity row.
- Version layer honored: 26.721

#### MW-3 — not-run (honest bound, as the criterion prescribes)
- Windows runtime white-screen validation: WINDOWS_GUI_LAB unavailable;
  Linux/CI evidence classes are the closure evidence (per PR §3 lab bounds —
  labeling verified intact in the parity row and KF).

### 4.3 Git process hygiene — PR §5.7, `complete` (regression control)

#### GPH-C1 — verified
- Step class run: S
- Evidence pointer: `RuntimePolicy::default()` — `git_debounce:
  Duration::from_millis(300)` and `max_parallel_git_processes:
  NonZeroUsize::MIN` (`codex-platform/src/lib.rs:381-382`); the debounce
  flows on the live path via `runtime_policy.git_debounce`
  (`codex-app/src/backend.rs:3723`); coalescing in `GitRefreshDebouncer`.
- Version layer honored: 26.721

#### GPH-C2 — verified
- Step class run: S
- Evidence pointer: single-flight = `max_parallel_git_processes == 1`
  (platform `lib.rs:373-382`), enforced by the runtime policy consumed on the
  git spawn path.
- Version layer honored: 26.721

#### GPH-C3 — verified (test exists; per-test local run for the platform crate blocked)
- Step class run: S + U-bounded
- Evidence pointer: standing acceptance tests
  `default_policy_prevents_parallel_git_storms`
  (`codex-platform/src/lib.rs:401-404`) and
  `git_refreshes_are_coalesced_until_the_debounce_expires`
  (`codex-app/src/backend.rs:22169-22187`). The platform-crate battery could
  not execute locally (§1 #5, wayland-sys build failure — the test itself is
  OS-independent but the crate's linux dep graph blocks the build); the app
  test is CI-delegated per the repo's own OOM discipline. Both are
  source-verified to assert exactly the claimed controls.
- Version layer honored: 26.721

#### GPH-C4 — verified
- KF row 4 present and accurate ("300 ms debounce, notification coalescing,
  one backend Git operation at a time" — "Implemented").

### 4.4 Marketplace admin-disabled install — PR §5.8, `complete`

#### MKT-C1 — verified
- Step class run: S + U
- Evidence pointer: `plugin_installability`
  (`codex-app/src/backend.rs:568-572`): `DISABLED_BY_ADMIN` matched explicitly
  and carried as a distinct `(installable, disabled_by_admin)` state — never
  collapsed to generic unavailability; test
  `plugin_installability_preserves_admin_disabled_state`
  (`backend.rs:21363-21369`, asserts `(false, true)`).
- Version layer honored: 26.721

#### MKT-C2 — verified
- Step class run: S
- Evidence pointer: catalog action button (`ui.rs:32220-32234`):
  `.tooltip(if disabled_by_admin { "Access is turned off by your admin" } …)`
  AND `.disabled(any_pending || !installable)` — actions disabled with the
  recovered tooltip attached.
- Version layer honored: 26.721

#### MKT-C3 — verified
- Step class run: S
- Evidence pointer: details surface (`ui.rs:30416-30431`): label
  `"Disabled by admin"` when `plugin.disabled_by_admin` — exact string.
- Version layer honored: 26.721

#### MKT-C4 — not-run (fixture/platform bound)
- No seedable `DISABLED_BY_ADMIN` catalog fixture exists in the evidence
  pattern (the wo-p1-003 `seed-*.py` pattern covers projects, not plugin
  catalogs), and no build is possible in this sandbox; GUI leg recorded
  not-run with that reason (never a claimed GUI run that did not happen).

### 4.5 Keyboard shortcut reference — PR §5.10, `complete`

#### KSR-C1 / KS-1 — verified
- Step class run: S
- Evidence pointer: Ctrl+/ bound (`KeyBinding::new(&shortcut("/"),
  ShowKeyboardShortcutsShortcut, …)` `ui.rs:5022`) with a live `on_action`
  listener (`ui.rs:43655`) — the one bound-and-handled global GPUI action
  (sweep battery #10 reproduced: 4 hits); registry ownership single
  (`showKeyboardShortcuts` is the only ACTIVE entry carrying `/`); Help/menu
  row at `ui.rs:13874` (accelerator attach) — same dialog target.
- Version layer honored: 26.721

#### KSR-C2 / KS-2 — verified (source); fresh overlay frame not-run
- Step class run: S (+ archived-G re-read)
- Evidence pointer: the effective-binding filter at `ui.rs:39550`
  (`.filter(|item| !self.effective_keyboard_shortcuts(item).is_empty())`)
  inside `render_keyboard_shortcuts_modal` (`ui.rs:39542`); no-results copy
  "No active shortcuts" / "No matching shortcuts" (`ui.rs:39593-39597`);
  advertised-row recount at the base = **51** (76 registry − 25 empty-default;
  the sweep's 47 at `3c9f113` + the four WO-P2-008 attention bindings —
  attributed growth). Archived ev/15 VLM re-read consistent (§3).
- Version layer honored: 26.721

#### KSR-C3 / KS-3 — verified (source + archived frame)
- Step class run: S (+ archived-G re-read)
- Evidence pointer: overlay iterates `KeyboardShortcutGroup::ALL`
  (`ui.rs:39546`) in enum order Thread → Navigation → Panels → Workspace →
  Skills → Configure → App → General; `label()` (`ui.rs:2710-2721`) renders
  the **stable reference names** — `Thread => "Chat"`, `Workspace =>
  "Project"` — so the displayed order is Chat → Navigation → Panels → Project
  → Skills → Configure → App → General, positionally identical to the stable
  comparator. Explicit inventory (per the criterion's requirement): the enum
  *variant* names `Thread`/`Workspace` are Flauz-internal identifiers for the
  first and fourth groups; the rendered headers preserve the stable names
  (VLM re-read of ev/15 shows "Chat", "Navigation" top-to-bottom). No group
  added, removed, or silently reordered at the display layer.
- Version layer honored: 26.721

#### KSR-C4 / KS-4 — verified (source + archived); fresh scene not-run
- Evidence pointer: the editable settings page
  (`render_keyboard_shortcut_settings`, sweep anchor re-resolved) iterates
  the same `ACTIVE_KEYBOARD_SHORTCUTS` registry; archived ev/11 is the
  B2-era confirmation. Fresh scene not-run (no build).

#### KSR-C5 — **defect-found (documentation-class)**
- Step class run: S (full sweep verification battery re-run at the base — §2)
- Evidence pointer / reproduction: the sweep's silent-state findings persist
  at the review base with exact anchors: F-A1 `ui.rs:8899`, F-A2 `ui.rs:8905`,
  F-A3 `ui.rs:9052`, F-A4 `ui.rs:8870`, F-A6 `ui.rs:10963`, F-D1 `ui.rs:8205`
  (`if command == "/review" { … return true; }` — silent swallow), F-D2
  `codex-core/src/lib.rs:13324`. F-A4 **is** documented as a residual in the
  parity report §5.10 Gap cell; **F-A1/A2/A3/A5/A6, F-D1, F-D2 are documented
  only in `docs/research/evidence/wo-p2-007/input-surface-sweep.md` and are
  not enumerated in the parity row's Gap cell.** Per the criterion's pass
  condition ("every still-silent advertised state is either fixed or
  documented as a residual in the parity row"), this is a documentation
  completeness defect, not a product-code regression (no new silent state was
  introduced — the set is unchanged since the sweep's record).
- Version layer honored: 26.721 + findings-of-record
- Follow-up: parity-row residual enumeration (see §7).

#### KS-5 — not-run (R-class cross-check; optional)
- No official 26.908 runtime available side-by-side in this sandbox; LX
  archived material already carries the ref+1 label and was not re-observed.

### 4.6 Feedback — PR §5.10, `complete`

#### FBK-C1 / FB-1 — verified
- Step class run: S
- Evidence pointer: `FeedbackClassification` (`codex-core/src/lib.rs:4370-4408`):
  exactly five variants; `ALL: [Self; 5]`; `as_str` literals exactly `bug`,
  `bad-result`, `good-result`, `safety_check`, `other`.
- Version layer honored: 26.721

#### FBK-C2 / FB-2 — verified
- Step class run: S + U
- Evidence pointer: `feedback_upload_is_validated_bounded_and_single_flight`
  (`codex-core/src/lib.rs:33013-33069`) — **passed** in battery #4: whitespace
  reason → no effect + error "Share details before submitting."; valid submit
  → exact `Effect::SubmitFeedback` with trimmed reason, thread id when
  present; second request while pending → single-flight empty.
- Version layer honored: 26.721

#### FBK-C3 — verified
- Step class run: S
- Evidence pointer: `feedback_include_logs = true` set on every dialog-open
  and reset path (`ui.rs:10096-10105` `open_feedback_modal`, `11469`, `6735`,
  `8015`) — session logs default ON.
- Version layer honored: 26.721

#### FBK-C4 — verified
- Step class run: S
- Evidence pointer: zero `browser_tabs|browserTabs` hits in `ui.rs` — no
  browser-tabs control renders.
- Version layer honored: 26.721

#### FBK-C5 / FB-3 — verified (source); GUI scene not-run
- Step class run: S
- Evidence pointer: three entry paths route to `open_feedback_modal`:
  palette/registry command `feedback` (interceptor arm `ui.rs:10475`),
  slash `/feedback` (`ui.rs:8259`), Help menu `Share feedback` (`ui.rs:42316`).
  Fresh three-path GUI scene not-run (no build).
- Version layer honored: 26.721

#### FBK-C6 / FB-5 — verified (source + wire test); authenticated e2e not-run
- Step class run: S + U
- Evidence pointer: `Effect::SubmitFeedback` handler
  (`codex-core/src/lib.rs:16227-16247`) → `codex-app/src/backend.rs:5956` →
  typed `feedback/upload` request on the supervised app-server
  (`codex-platform/src/app_server.rs:1492`); bounded details
  (`MAX_FEEDBACK_DETAILS_BYTES`, `lib.rs:125`); app-version tag + threadId +
  includeLogs verified by the protocol wire test
  (`codex-protocol/src/lib.rs:4049-4063`, **passed** in battery #6).
  Authenticated submission e2e: not-run (EQ-1/EQ-3 — no runtime, no
  credentials; per the criterion this leg stays source-read only).
- Version layer honored: 26.721

### 4.7 Stable-failure regression controls — PR §5.10, `complete`

#### SFR-C1 — verified (per MRW-C2, independently cited)
- The multi-root control tests listed under MRW-C2 were independently located
  and run (battery #7 green) — no evidence reuse without citation.

#### SFR-C2 — verified
- Step class run: S + U
- Evidence pointer: zero `read_to_string` in `crates/` (all readers bounded);
  standing test `rejects_the_observed_594_mb_failure_without_allocating_it`
  (**passed** in battery #7) — the 594-MB JSONL control as an acceptance test.
- Version layer honored: 26.721

#### SFR-C3 — verified
- Step class run: S + U
- Evidence pointer: `list_threads_state_db_only` +
  `ThreadListParams::state_db_page` (`app_server.rs:1047-1054`, call sites
  `505`/`1053`); the general `list_threads` entry **refuses** params without
  `use_state_db_only` (`app_server.rs:1062-1066`, "thread/list must use state
  DB only"); pagination via `validate_page_limit`; wire test
  `thread_list_is_bounded_to_state_database_metadata` **passed** (battery #6).
- Version layer honored: 26.721

#### SFR-C4 — verified (per GPH-C3, independently cited)
- `default_policy_prevents_parallel_git_storms` +
  `git_refreshes_are_coalesced_until_the_debounce_expires` independently
  located (§4.3).

#### SFR-C5 — verified
- Step class run: S (Windows arm read in `#[cfg(windows)]` code)
- Evidence pointer: `crates/codex-platform/src/process.rs:223-236` —
  `Job::create_with_limit_info` with `limit_kill_on_job_close()` +
  `job.assign_process(child.as_raw_handle())`, kill-on-close semantics on
  every exit path (`drop(job)` at 247-288); unix arms use `process_group(0)` +
  `terminate_process_tree`; `reserve_detached_process_supervisor` bounds the
  supervised tree; zero `taskkill` anywhere (AGENTS.md invariant).
- Version layer honored: 26.721

#### SFR-C6 — verified
- Step class run: S
- Evidence pointer: owned logging narrowly scoped — codexRS does not
  duplicate provider logs (bounded readers everywhere; owned-storage page
  caps in `docs/known-failures.md` "Current budgets"); no unbounded log-write
  path found on owned surfaces.
- Version layer honored: 26.721

#### SFR-C7 — not-run (CI citation live-check blocked)
- Both-matrix CI green at the review SHA could not be re-verified: GitHub API
  rate-limited unauthenticated; the work order's binding token rules restrict
  the credential to git transport (§0). The ledger's closure records (e.g.
  WO-P2-008: "PR #23 merged as 00a3392 (CI green both matrices)") remain the
  historical-record evidence; no single-matrix claim was found in the
  closures.

### 4.8 Side chats — PR §5.1 (C2 audit), `complete` (WO-P2-006, PR #18 → `5287c29f3f`)

#### SC-1 — verified
- Step class run: S + U (test located; app battery CI-bound)
- Evidence pointer: `OpenSideChatShortcut` declared (`ui.rs:1684`) + bound
  (`KeyBinding::new(&shortcut("alt-s"), …)` `ui.rs:4987`); interceptor arm
  `"openSideChat" => self.open_side_chat(...)` (`ui.rs:10841`);
  `KEYBOARD_SHORTCUT_COMMAND_IDS.contains(&"openSideChat")` asserted in
  `side_chat_binding_resolves_ctrl_alt_s_to_the_side_chat_action`
  (`ui.rs:48699-48713`).
- Version layer honored: 26.721 + current docs

#### SC-2 — verified (archived-frame VLM re-read); fresh scene not-run
- Evidence pointer: d9-005 re-read (§3): side panel open with composer
  placeholder "Ask a side question…" (`ui.rs:5754`) and the guidance string
  "Ask a side question without interrupting this chat." in-tree
  (`ui.rs:18285`); **selected sidebar item unchanged ("side chat fixture
  main")**.

#### SC-3 — verified
- Step class run: U
- Evidence pointer: `a_side_chat_first_message_creates_a_projectless_thread_without_selecting_it`
  (`codex-core/src/lib.rs:24312`) — **passed** in battery #4 (the
  load-bearing "submits without selecting" invariant); archived D9 frames
  005→007 + `vlmd9a.json` consistent.
- Version layer honored: 26.721 + current

#### SC-4 — verified
- Step class run: S + U
- Evidence pointer: `/side` executor arm (`ui.rs:8134`) + catalog row
  (`ui.rs:45749`) + availability guard (`ui.rs:23972`); test
  `side_slash_command_resolves_with_its_guard` (`ui.rs:48727`); D9b
  close-probe archived (`vlmd9b.json`: panel visible → closed → main view
  intact). Fresh probe not-run (no build).
- Version layer honored: 26.721 + current

#### SC-5 — verified
- Step class run: U
- Evidence pointer: all four state tests **passed** in battery #4:
  `a_side_chat_opens_and_closes_without_touching_the_selected_chat`,
  `a_side_chat_first_message_creates_a_projectless_thread_without_selecting_it`,
  `a_side_chat_continues_through_the_thread_turn_machinery`,
  `a_failed_side_chat_submission_restores_only_the_side_draft`
  (`codex-core/src/lib.rs:24235/24312/24404/24472`).
- Version layer honored: 26.721 + current

## 5. Partial rows covered by the executed rubric (r2 R-06–R-10) — condensed verdicts

| Criterion | Verdict | Evidence pointer (base SHA) |
|---|---|---|
| PC-1 | verified | `LocalProjectSummary.folders: Vec<PathBuf>` + cap 16 + invariants (`lib.rs:915-929`, `118`, `7509-7517`); storage tests green |
| PC-2 | verified | migration chain tests v1→v4 all green (battery #7: `version_one…four_storage_migrates_*`) |
| PC-3/PC-4/PC-5 | not-run (fresh scenes; no build) — archived D6 evidence + seed + vlm-reads consistent | `evidence/wo-p1-003/` (6 frames + `seed-wo-p1-003.py` + vlm-reads.txt) |
| PC-6 | not-run (fresh) — archived ev/06/08 consistent; ev/06 footer re-read verbatim | §3 |
| PC-7 | verified | historical-complete slice intact and green: `local_projects_load_and_support_add_rename_pin_open_and_remove`, `rename_validates_and_commits_the_confirmed_title`, `pinning_is_optimistic_and_archive_removes_the_local_pin`, `pinned_chats_load_from_bounded_ui_state_and_hydrate_from_app_server`, `archived_pagination_stops_at_the_visible_task_cap`, … (battery #4) |
| SS-1 | verified | `SettingsSection` = 18 variants (`ui.rs:2548-2567`); `DEFAULT_NAV_SECTIONS: [Self; 15]` (`ui.rs:2576-2592`, `#[cfg(test)]` — same gating as the sweep's record); CodeReview/Worktrees/ArchivedChats contextual-hidden |
| SS-2/SS-3 | not-run (fresh scenes) — archived ev/07/ev/23 consistent; production 3-group nav verified in source ("Personal"/"Integrations"/"Coding" at `ui.rs:32556/32635/32696`) | — |
| SS-4 | verified | `ImportProvider::{ClaudeCode, ClaudeCowork, Cursor}` + `ALL: [Self; 3]` (`lib.rs:3504-3511`) |
| SS-5 | verified | ledger §5 gap decisions + deferred set recorded ("no work orders by definition; blockers named on the rows"); KF limitations + `docs/platform-support.md` bounds intact |
| SS-6 / KA-1 | verified (source+tests; archived frames) | coverage tests `palette_indexes_every_default_nav_settings_section`, `palette_command_opens_settings_section`, `palette_settings_queries_resolve_to_settings_sections` (`ui.rs:49659+` region); wo-p2-004 captures consistent (§3) |
| KA-2 | not-run (fresh) — archived D7 captures 03-14 consistent | `evidence/wo-p2-004/` |
| KA-3 | **verified with recorded drift** | `PaletteCommand::ALL` = **70** at base vs claim's 51 at the rwo-020 base: growth +19 = WO-P2-010 delivery commit `10b0c24` (RenameChat, ForkThread, 4×Copy*, ApproveRequest, DeclineRequest, GoToChat1-9, ToggleReviewTab, ToggleMaximizeSidePanel — exactly the "review/fork/copy/approval/rename/goto-chat" scope), merged via PR #25 in this base. Attributed delivery, not silent drift. **However the parity §5.10 KA row still reads "51-command registry" → stale row (finding #2).** |
| KA-4 | verified | `ctrl_p_routes_to_the_search_files_command` (`ui.rs:52076-52117`, 5 assertions: single ownership, registry membership, palette mapping, default shortcut, metadata); single CmdOrCtrl+P owner = `searchFiles` (registry scan); F-A4 no-workspace residual documented in the parity row |
| KA-5 | not-run (fresh) — archived ev/08-11 consistent | — |
| KA-6 | verified | Gap cell lists the open residuals (remaining stable commands, focus order, screen-reader labels, reduced-motion) — honestly open |
| NT-1 | not-run (fresh) — archived ev/03/04 (error-banner class) consistent | — |
| NT-2 | verified (source half) + **not-run GUI half with the required honest note** | title-tooltip count logic + banner component source-verified; ev/03/04 are error-path banners, NOT completion events — the completion path needs an authenticated background turn (EQ-1/EQ-3): not-run, exactly as the criterion demands |
| NT-3 | verified | `background_chat_window_title` (`ui.rs:43830`) + named test `background_chat_window_title_counts_only_non_selected_running_chats` (`ui.rs:47765`) — non-selected subset semantics |
| NT-4 | verified | tray groups/badges/sounds in ledger §5 ledger-enhancement residuals; Linux tray/global shortcuts documented platform bounds (KF limitations + platform-support) |
| AV-1 | verified | MAX=76 + four attention ids registry members (`lib.rs:133-157`); defaults in ACTIVE (`ui.rs:2901-2997`) |
| AV-2 | verified | all **7** state tests passed with per-test output (battery #4): `background_turn_completion_marks_the_chat_needing_attention`, `approval_request_marks_background_chats_needing_attention`, `visiting_a_chat_clears_its_attention_flag`, `clear_all_unread_indicators_reports_honestly`, `toggle_selected_chat_unread_round_trips_with_status`, `archiving_a_chat_drops_its_attention_flag`, `toggle_activity_view_surfaces_honest_guidance`; bounded FIFO cap at `lib.rs:8908-8917` (evict `remove(0)` at `MAX_VISIBLE_THREADS`) |
| AV-3 | verified (archived-frame VLM re-read) | d11b-04 verbatim status re-read (§3); md5s match; all five D11b frames differ per the md5 records |
| AV-4 | verified (existence + content) / not-run (local per-test execution) | binding tests located by name: `activity_view_binding_resolves_ctrl_alt_u_to_visible_guidance`, `next_unread_chat_binding_resolves_ctrl_alt_a_and_jumps`, `clear_all_unread_binding_resolves_shift_escape`, `toggle_thread_unread_binding_resolves_ctrl_shift_u`, `next_unread_chat_follows_sidebar_order_cyclically` (+ `ctrl_p_routes…`); local run blocked (§1 #8: gpui system-dep chain + repo's own OOM discipline); CI live-check blocked (API rate limit) — ledger records CI green both matrices at merge `00a3392` |
| AV-5 | not-run (fresh full-flow; no runtime — D11 honest bound); archived D11 form consistent | — |
| AV-6 | verified | residuals documented in the parity row Gap cell + `wo-p2-008/README.md` "Documented residuals" (dot-on-row/jump GUI bound, activity-view surface future WO, persistence unverified) — no claim beyond scope found |

## 6. Findings (defects, with source line + reproduction)

1. **DEFECT (documentation completeness) — KSR-C5.** The sweep's
   silent-state findings F-A1 (`archive_selected_chat`, `ui.rs:8899`),
   F-A2 (`toggle_selected_chat_pin`, `ui.rs:8905`), F-A3
   (`rename_selected_chat`, `ui.rs:9052`), F-A5 (thread1-9 empty-slot,
   `ui.rs` navigate_chat_slot), F-A6 (`git.commit` pending-PR silent skip,
   `ui.rs:10963`), F-D1 (`/review` typed swallow, `ui.rs:8205`), F-D2
   (`/compact` runtime-not-ready silent guard, `codex-core/src/lib.rs:13324`)
   persist at the review base and are documented only in
   `docs/research/evidence/wo-p2-007/input-surface-sweep.md`; the parity
   report §5.10 Gap cell enumerates only F-A4. Reproduction (any one):
   `rg -n 'command == "/review"' crates/codex-app/src/ui.rs` → `8205` —
   silent `return true` with no composer change, no parity-row residual entry.
2. **DEFECT (stale parity row) — KA-3.** Parity §5.10 "Keyboard and
   accessibility" still claims the "**51-command** palette registry" while
   `main` @ `876bbe8` carries `PaletteCommand::ALL` = **70**
   (`ui.rs:3480`). The growth is fully attributed to the merged WO-P2-010
   delivery (`10b0c24`, PR #25) — not silent drift — but the matrix row was
   not updated with the delivery. Reproduction:
   `rg -n 'const ALL: \[Self; ' crates/codex-app/src/ui.rs` → `3480: const ALL: [Self; 70]`.
3. **DEFECT (ledger lag) — wave-S closures.** The ledger entries
   `### WO-P2-009 (IN FLIGHT)` and `### WO-P2-010 (IN FLIGHT)`
   (`docs/research/FEATURE-PARITY-WORK-ORDERS.md:455,459`) still read IN
   FLIGHT at the review base although both deliveries are **merged on main**
   (PR #24 `0eed8c3` browsing-history — its storage tests
   `browsing_history_is_bounded_and_evicts_the_oldest_visits`,
   `version_four_storage_migrates_browsing_history` and app test
   `browser_address_revisits_history_before_falling_back_to_search` are
   green/present; PR #25 `10b0c24` palette rows — 19 new live palette
   entries verified). Reproduction: `git log --oneline
   d479c7b..876bbe8` vs the ledger status lines.
4. **Evidence-hygiene observations (non-defect, recorded for the Lead):**
   `d11-02a-typed.png` has no md5 record in `d11-md5.txt` (8 frames vs the
   README's "7-frame md5 sequence"); `wo-p2-007/README.md` cites two
   parity-lab scene scripts that are not shipped in-repo; the B2 `flauz/`
   frame set ships no md5 list (the matrix says frames were md5-checked).

No product-code defect was found in any of the 8 `complete` rows: every
source anchor resolved at the base SHA, every cited test that was runnable
locally passed, all recorded md5s match, and the VLM re-reads corroborate the
archived claims. The three findings above are documentation/record
completeness defects (the class the rubric's attack rules target).

## 7. Residuals / follow-up WO candidates (advisory — Lead decides)

1. **Doc-reconciliation WO (wave-S closure records):** update ledger
   WO-P2-009/WO-P2-010 to CLOSED with their merge SHAs, and refresh the
   parity §5.10 KA row (51→70 palette registry, WO-P2-010 attribution) and
   any §8 counts affected.
2. **Parity-row residual enumeration:** add the sweep F-findings (or an
   explicit pointer to `input-surface-sweep.md`) to the §5.10 Gap cells so
   KSR-C5's documentation requirement is met without new evidence.
3. **Honest-status upgrade candidate (F-A1/A2/A3):** mirror the WO-P2-008
   honest-guidance pattern (status line on no-selection) for
   `archiveThread`/`toggleThreadPin`/`renameThread`, per the
   input-quality doctrine.
4. **Structural watch list:** the 23 bound-but-never-handled GPUI actions +
   unbound `OpenBrowserTabShortcut` (sweep §SUMMARY) remain a decoupling
   candidate (menu accelerator labels vs dispatch) — unchanged at base.
5. **Evidence packaging:** ship md5 records for the B2 frame set and the
   missing wo-p2-007 scene scripts, or annotate their parity-lab provenance,
   so future fresh-verification passes can re-derive them.

## 8. Honesty statement

Every `verified` above carries a concrete evidence pointer (file:line at the
base SHA, test name + per-test result, frame + md5, or archived-frame VLM
re-read) produced by this review's own commands. Every `not-run` carries its
true bound (build blocked by missing system dev packages; no codex CLI
runtime; WINDOWS_GUI_LAB unavailable; GitHub API rate limit under the token
rules; local OOM discipline). No mock runtime was used; no result was
invented. Defects are reported, not fixed — fixes belong to new work orders
for the implementation waves.
