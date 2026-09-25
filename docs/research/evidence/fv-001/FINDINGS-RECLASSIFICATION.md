# FV-001 — Findings reclassification against current main

- **Work order:** FV-001 (Wave 7, Worker A — docs-only)
- **Base (pinned):** `7f660c00407a5741eee975b2274570a200ff576b` (the Wave-7
  dispatch head over the Wave-6 merge `f5e2a5c`)
- **Date:** 2026-09-25
- **Companion artifact:** [docs/research/FV-CATALOG.md](../../../FV-CATALOG.md)
  (the journey→client→scene catalog + the FV gap list — FV-002's authoring
  contract)
- **Law:** WAVE7-FV-WORK-ORDERS.md kernel addendum §3 (the reclassification
  law), §2 (the honest-bounds law), §5 (the evidence schema), §7 (the
  credential law). The ACTIVE-EXECUTION-STATE synchronization law governs
  closure: **a finding closes only on runtime evidence in a fresh
  environment at current main**; every `fixed-by` verdict below is a
  PROVISIONAL classification that FV-003's formal pass confirms.

## Method and verdict vocabulary (addendum §3 — closed vocabulary)

Every historical finding from the input ledgers is quoted compactly,
then reclassified against what main IS at `7f660c0`:

| Verdict | Meaning |
| --- | --- |
| `still-open` | the current source keeps the defect/gap/bound; citations name the current lines |
| `fixed-by-<merge>` | a landed merge demonstrably removed the cause (commit/PR + current source cited) — provisional until FV-003 re-verifies at runtime |
| `changed-surface` | the surface the finding described no longer exists in that shape; the replacement is cited |
| `needs-runtime-verify` | static reading cannot decide; the FV scene that will decide it is named |

Anchors: `AES` = ACTIVE-EXECUTION-STATE.md; `KF` = known-failures.md;
`AI` = f1-sweep/accessibility-inventory.md; `JI` = f1-sweep/journey-inventory.md;
`PI` = f1-sweep/parity-inventory.md; `RGI` = f1-sweep/release-gate-inventory.md;
`REC` = f1-sweep/recommendations.md; `FCR` = f1-sweep/F1-CLOSURE-RECORD.md;
`FRB`/`ABV` = f1-sweep/binding verdicts; `CL` = evidence/codex-linux/README.md;
`FRM` = research/FLAUZ-REFERENCE-MATRIX.md; `W2..W6` = wave gate records;
`W7` = WAVE7-FV-WORK-ORDERS.md; `E2B` = research/E2B-PARITY-ENVIRONMENT.md.
Source citations are `file:line` **at the pinned base `7f660c0`** (read at
this checkout; line numbers are the current-main lines, symbols are stable).

---

## A. The ACTIVE-EXECUTION-STATE expectations list (AES L110-116; W7 §3)

The named historical families, each reclassified:

| ID | Original claim (compact) | Ledger | Verdict | Current-main evidence + runtime-verify need |
| --- | --- | --- | --- | --- |
| **A-1** | "GPUI/X11/lavapipe rendering issue (L-002)" — GPUI 0.2.2 renders via blade/Vulkan; the codexRS window is 32-bit ARGB (renders nothing without a compositor); first frame needs an input/focus event on bare Xvfb | AES L114; E2B §Linux-B (Xvfb 1600x1000 + picom xrender + lavapipe ICD, userspace mesa 25.0.7) | **still-open** (an environment property of the lane, not a product defect) | The rendering architecture is unchanged at `7f660c0`: the desktop is still GPUI/blade (no renderer change landed in Waves 2-6 — `crates/codex-app` remains the GPUI shell). The CI startup smoke proves boot under software GL without a compositor: `scripts/linux_desktop_smoke.sh:41-50` (xvfb-run, `LIBGL_ALWAYS_SOFTWARE=1`, `CODEX_RS_CODEX_BIN=/bin/false`, 15 s survival assert). FULL rendering (frame content) still requires the sealed lab recipe (Xvfb + picom + lavapipe + wake click — E2B §1.5). **FV need:** every Linux FV scene runs under the sealed `session.sh` recipe (FV-CATALOG §Linux lane); the render chain itself is re-proven by FV-L01's first frame. |
| **A-2** | "runtime/environment incompatibilities" — the fail-closed runtime-compat family: missing runtime (`/bin/false`) degrades to "Resolving…" auto-retry; `workflow/*` methods fail closed with actionable errors; the codex-app test binary cannot link in the Linux sandbox (the pre-existing libpipewire gap) | AES L114; FRM §1 Retry row (ev/24); codex-universal/RUNTIME-COMPATIBILITY.md; W3 gate record (Gate C note) | **still-open** (by-design degradation + a named sandbox toolchain bound) | Graceful degradation is the documented contract (RUNTIME-COMPATIBILITY.md "degrade per-feature, not whole-app") and the current scheduler retries with bounded backoff (`backend.rs:223-224` `APP_SERVER_RECONNECT_INITIAL_DELAY` 1 s / `MAX_DELAY` 20 s; `backend.rs:1797-1837` `AppServerReconnectScheduler`). The libpipewire link gap is a property of the worker sandbox, not of main (CI ubuntu-24.04 links and tests green — ci.yml matrix). **FV need:** FV-L04 (the reconnect family re-run) + FV-W03 (the Windows retryable-startup scene) verify the runtime-missing degradation at runtime at the pinned SHA. |
| **A-3** | "silent no-ops" — advertised chords dispatching with zero feedback in guarded states (F-A1/A2/A3/A6/F-D1/D2 family; F-A4 Ctrl+P; the entry-surface terminal/browser toggles) | AES L114; AI §(e); FRM findings (a); PI row 38 | **fixed-by-WO-P2-012 + WO-P2-020 + WO-P1-001/002** (provisional; see C-rows for the per-finding split) | The six-family guards are in current source: `ui.rs:48136-48174` (`archive_thread_command_status` "Select a chat before archiving it." / pin / rename / "The selected chat is no longer available." / `commit_or_push_pending_status` "A Git workflow is already running."), `ui.rs:48120-48128` (`composer_review_unavailable_status`), `ui.rs:48202-48210` + `ui.rs:10054-10061` (F-A4: "Select a workspace before searching files."), with the executor wiring + tests at `ui.rs:55485-55635`. The entry-surface toggles are honest now: `codex-core/src/lib.rs:9750-9757` ("No open chat (entry surface): surface the guard instead of silently opening an empty dock (WO-P1-001 honest toggle)" — test `lib.rs:30162`) and `lib.rs:9355-9361` (browser guard; test `lib.rs:30191`). **FV need:** FV-L06 + FV-L20 re-exercise the guarded chords at runtime. |
| **A-4** | "draft loss" — composer drafts lost on submission failure / navigation / project-picker handoff | AES L114; PM Composer row ("active chats preserve the draft instead of steering Goal attachments"; the project picker "preserves the current draft, attachments, and composer mode") | **fixed-by the F1 parity merges** (the draft-preservation contract; provisional) | The draft-generation machinery is in current source: `codex-core/src/lib.rs:4781` (`composer_draft_generation`), `lib.rs:7889` (`advance_new_chat_draft_generation`), and the regression test `lib.rs:27316` (`stale_same_chat_submission_failure_preserves_the_newer_draft`). The retryable-message restoration path is `lib.rs:20542` (`restore_retry_prompt`). **FV need:** FV-L01/L15 exercise compose→navigate→return on the real binary (draft retention is a frame-visible assertion). |
| **A-5** | "raw protocol errors" — provider/protocol errors surfacing raw to the user | AES L114; PM rows ("without exposing raw configuration or provider errors") | **changed-surface** (errors are translated to bounded honest statuses; the raw surface no longer exists in that shape) | The error-event path emits bounded user-language statuses, never raw payloads: `backend.rs:13315-13335` ("Codex hit an error and is retrying." / "Codex couldn't complete the request. Try again, or check the account and connection settings.") through `Action::SetStatus` bounded at 16 KiB (`codex-core/src/lib.rs:20336-20339`). The auth-wall family ("all live-chat turns error honestly in the lab" — FCR lab bounds) is the runtime-observable slice. **FV need:** FV-L05/L07 (the honest failure surfaces at the auth wall) confirm no raw error text renders. |
| **A-6** | "toast persistence" — status toasts/banners persisting (the retry toast; the "Connection lost. Reconnecting… Dismiss" banner) | AES L114; wo-p2-008 README (retry toast "does not block"); wo-p2-011 README (persistent reconnect banner); FCR (the −32600 banner = the documented fail-closed contract) | **changed-surface** (persistent banners are now state-driven and dismissible by design; a stuck-toast defect surface no longer exists in that shape) | The safety-buffering banner is dismissible and turn-scoped: `ui.rs:25299-25337` (`render_safety_buffering_banner` — `!buffering.dismissed && timeline.active_turn_id == buffering.turn_id`, Retry/Dismiss affordances). The reconnect footer is a truthful persistent status with bounded labels by design (`ui.rs:16277-16292` "App-server online" / "Reconnecting…" / "Reconnecting · attempt {n}"). **FV need:** FV-L04 asserts the footer cadence + the banner's dismiss path at runtime. |
| **A-7** | "other UX defects" — the umbrella for the inventory findings | AES L114 | **reclassified item-by-item** in §C-§J below | — |

---

## B. known-failures.md (the six stable-failure controls + the active limitations)

### B.1 The six stable-failure regression controls (KF table — controls that must STAY implemented)

These are reference failure modes with codexRS controls; the reclassification
question is "does the control still exist at current main?" Every control is
**still-implemented (fixed-by its original pre-Flauz merge; provisional)** and
re-proven by the standing CI battery + the FV soak/reconnect references:

| ID | Control (KF row) | Verdict | Current-main citation |
| --- | --- | --- | --- |
| B-1 | Windows multi-root white screen → native `Path`/`PathBuf`, no browser path shim | **still-implemented** | `crates/codex-core` path handling is native throughout (no path shim exists; the desktop never embeds a browser runtime — AGENTS.md L9-10 unchanged). Re-proven by the double-matrix CI (`.github/workflows/ci.yml:24` `os: [windows-latest, ubuntu-24.04]`). |
| B-2 | Unbounded JSONL line (594 MB observed) → live history only through bounded app-server pages; no direct live JSONL reads | **still-implemented** | Frame bound `crates/codex-protocol/src/lib.rs:17` (`DEFAULT_MAX_FRAME_BYTES = 16 * 1024 * 1024`); the byte-budget machinery `crates/codex-platform/src/byte_budget.rs`; no `read_to_string` on live JSONL anywhere in the owned storage path (AGENTS.md L16-18 law). |
| B-3 | Startup history scan (~9 GB) → `thread/list` paginated, always `useStateDbOnly: true` | **still-implemented** | Enforced at the boundary: `crates/codex-platform/src/app_server.rs:1062-1066` (a `thread/list` without `use_state_db_only` is rejected with "thread/list must use state DB only"); the param is typed at `crates/codex-protocol/src/lib.rs:613`; the gateway rides the same law (`crates/flauz-web-gateway/tests/integration.rs:186,284` pass it verbatim). |
| B-4 | `git.exe` process storm → 300 ms debounce, coalescing, one backend Git op at a time | **still-implemented** | `crates/codex-app/src/backend.rs:22172` (`Duration::from_millis(300)`); `crates/codex-platform/src/lib.rs:381` (`git_debounce: Duration::from_millis(300)` default). |
| B-5 | Process-cleanup storm → one supervised tree, graceful shutdown, Job Object on Windows | **still-implemented** | `crates/codex-platform/src/process.rs:74` (Job Object supervision error path); `browser.rs:1467-1472` (Browser Job Object assign); no polling taskkill loop exists (AGENTS.md L19-21 law). |
| B-6 | Unbounded logging → no provider-log duplication; narrowly scoped owned state | **still-implemented** | The gateway's credential-free structured logs (`crates/flauz-web-gateway/src/logging.rs`) and the owned-storage page bound (KF budgets, 500 rows) stand; soak gate G-7 (zero panics, bounded diagnostics) re-proves at the release gate. |

### B.2 The active release-candidate limitations (KF §Active)

| ID | Limitation (KF) | Verdict | Current-main evidence |
| --- | --- | --- | --- |
| B-7 | Linux Computer Use limited to bounded X11/XWayland screenshot observation when `DISPLAY` is set; Wayland/input/overlays unavailable | **still-open** (a platform bound, honestly documented — not a defect to fix in FV) | `crates/codex-platform/src/computer_use.rs:34-57` ("Linux observation is intentionally limited to X11/XWayland… Pure Wayland requires the separate portal path"; `computer_use_platform_available()` gates on `DISPLAY`). FV gap list Linux lane carries the Wayland bound. |
| B-8 | Dynamic Computer Use tools attach at `thread/start`; existing tasks cannot gain them | **still-open** (protocol-shaped bound) | Unchanged at `7f660c0` (no post-start tool-attachment path exists in the app-server surface). Needs a protocol change; recorded, not FV-blocking. |
| B-9 | Release archives are unsigned portable previews; installers/updaters absent; Linux `--install-desktop-entry` only, never overwriting | **still-open** (release-stage work, F13) | `crates/codex-app/src/main.rs:47-59,158` (`--install-desktop-entry` handling). The FV catalog verifies the entry behavior (FV-L22); signing/updaters are production-gate items (ROADMAP F13), not FV lane scope. |
| B-10 | Linux validated by Ubuntu CI; broader desktop-environment smoke in progress | **still-open** (coverage bound) | CI runs ubuntu-24.04 + windows-latest (`ci.yml:24`) with the Linux startup smoke (`ci.yml:105-111` → `scripts/linux_desktop_smoke.sh`). The FV Linux lane runs Debian 13 Xvfb (the lab) — the desktop-environment matrix stays a named gap (FV gap list). |
| B-11 | "Accessibility semantics and full keyboard-only navigation need a dedicated pass before the stable release" | **changed-surface** (the dedicated pass RAN at F1; the residual is the platform-bound AT set) | The keyboard-only pass is green at the published rc.14 (`ABV`: the 24-frame journey, N1 closed via UX-002, 017 focus contract verified, 019 trap ladder green). At `7f660c0` the PTY focus transfer (N5) landed too (`ui.rs:425-442` + test `ui.rs:55817-55826`), and the promo modal joined the 019 family (`ui.rs:55829-55859`). The residual: screen-reader labels/AT tree, OS-level reduced-motion, status AT announcement, titlebar AT naming — platform-bound upstream-GPUI (see C-17..C-20). FV-L20 re-runs the keyboard-only battery at current main. |

---

## C. f1-sweep accessibility-inventory findings (AI)

The AI findings-of-record (defect/gap/platform-bound classes), each exactly once:

| ID | Finding (AI class) | Verdict | Current-main evidence |
| --- | --- | --- | --- |
| C-1 | F-A1/F-A2/F-A3/F-A6/F-D1/F-D2 silent-no-op family (defect) | **fixed-by-WO-P2-012** (PR merged in the F1 set — roadmap F1 L67; provisional per the sync law) | Guards: `ui.rs:48136-48174` + `ui.rs:48120-48128`; executor wiring + fall-through tests `ui.rs:55485-55635` (e.g. `ui.rs:55615` asserts "Select a chat before archiving it."). **FV need:** FV-L20 chord ladder re-proves at runtime. |
| C-2 | F-A4 Ctrl+P silent without workspace (defect) | **fixed-by-WO-P2-020** (PR #38 → `30572a4`; provisional) | `ui.rs:10054-10061` (open path reports the honest status) + `ui.rs:48202-48210` (`files_palette_command_status` "Select a workspace before searching files.") + test `ui.rs:55635`. Binding evidence: FR-BINDING-VERDICT fr3-05 (toast VLM-verbatim at rc.14). **FV need:** FV-L20. |
| C-3 | F-A5 thread1-9 empty slots do nothing (pass-by-design) | **still-open** (matches the official "safely do nothing" contract — no action needed) | Unchanged semantics at `7f644c0`… at `7f660c0` (registry `ui.rs` `thread1..9` commands; PM L112 contract). |
| C-4 | Palette close focus restore (gap) | **fixed-by-WO-P2-017** (PR #41 → `8dcfcb9`; provisional) | `ui.rs:10063-10068` (capture on open) + `ui.rs:10082-10105` (`close_command_palette` → `apply_overlay_close_focus_restore`, the r2 parameter-passed close context — the r1 panic fix is in the comment `ui.rs:10088-10094`) + tests `ui.rs:55670-55755`. Binding: D17 + ABV typed probes at rc.14. **FV need:** FV-L20 focus probes. |
| C-5 | Palette visible entry — no toolbar button (gap; also a PJ §1 layer-1 contract issue) | **fixed-by-WO-P2-018** (PR #37 → `b562397`; provisional) | The title-bar discovery-entry model: `ui.rs:513-535` ("The visible command-palette entry (WO-P2-018): the chrome counterpart…" / "The Activity bell (WO-P2-018, J-17)…") rendered at `ui.rs:15571-15610`. **FV need:** FV-L01 cold-start frame shows the entry. |
| C-6 | Attention-dot label (gap) | **fixed-by-WO-P2-018** (provisional) | The unread-dot tooltip/text alternative rides the 018 label-parity set (`ui.rs:513-535` family; roadmap F1 L69 "unread-dot tooltip"). **FV need:** FV-L18 asserts the dot + its label at runtime. |
| C-7 | Archived-chats icon-only single deletion (gap) | **fixed-by-WO-P2-018** (provisional) | Roadmap F1 L69 ("archived-deletion label"); the label-parity set is the 018 merge (current copy registry `ui.rs:1892` "and Activity view rows render (WO-P2-018): plain language…"). **FV need:** bounded — the archived surface is flow-gated; unit-pinned (see FV gap list Linux). |
| C-8 | Tab-trap completion for the four non-evidenced modals (remote pairing / remote confirmation / account logout / plugin install) (gap) | **fixed-by-WO-P2-019** (merged; the flow-gated four remain unit-pinned — runtime probes at the RC binding passed on the reachable Clear modal; provisional for the gated four) | The 019 trap pattern is in the bind_keys family (`ui.rs:5868-5935` region — the scoped-escape/trap contexts incl. the new Flauz surfaces); roadmap F1 L72 records the closure with the runtime-probe bound. **FV need:** FV-L20 (reachable traps) + named gap (the flow-gated four — FV gap list). |
| C-9 | Broader focus order across all surfaces (gap) | **still-open** (bounded honestly — PM baseline L130-131) | No whole-app focus-order pass has landed (the 017/019/UX-002 family covered the load-bearing surfaces). FV-L20 samples the order across the six F1 surfaces; the remainder is the F2+ a11y backlog. |
| C-10 | Full contrast parity (gap) | **still-open** (bounded — the control exists, full parity open) | The contrast preference feeds theme mixing (`ui.rs:670-690` region, PM L101); no full-surface contrast audit exists. FV gap list (visual-depth bound). |
| C-11 | Platform-bound: screen-reader labels / AT tree | **still-open** (platform-bound — upstream GPUI; do not block) | GPUI exposes no accessibility tree on Windows/Linux at the current layer (unchanged at `7f660c0`). The WEB lane carries its own aria pass (WEB-002 a11y journey — `web/lab/journeys.mjs:692` j-a11y-responsive). |
| C-12 | Platform-bound: OS-level reduced-motion following | **still-open** (platform-bound) | Unchanged: legacy `System` values resolve to `Off` (PM L101; the native shell exposes no shared OS motion signal). |
| C-13 | Platform-bound: status-message AT announcement (live-region equivalent) | **still-open** (platform-bound on desktop) | Desktop has no live-region concept at the GPUI layer; the WEB lane has aria-live (approvals queue — W6 gate record). |
| C-14 | Platform-bound: titlebar-control AT naming | **still-open** (platform-bound) | `render_title_bar_control` glyph chrome unchanged in shape. |
| C-15 | AI source-unverifiable 1 (runtime focus landing after palette/overlay close) | **fixed-by-WO-P2-017 + ABV binding** (provisional) | See C-4; ABV P1a/P1b typed probes land in the composer at rc.14. FV-L20 re-proves at current main. |
| C-16 | AI source-unverifiable 2 (terminal open → PTY focus transfer) | **fixed-by-UX-003** (merged in the F2 Wave-1 set — roadmap F2 UX-003; provisional) | `ui.rs:425-442` (`terminal_dock_focus_transfer_due` — armed/pending/overlay-deferred contract) + source-assertion tests `ui.rs:55817-55826`. FV-L20 PTY probe re-proves at runtime. |
| C-17 | AI source-unverifiable 3 (Find active-occurrence non-color distinction at runtime) | **needs-runtime-verify** (ABV verified the positive path at rc.14: "bar + counter '1 / 3+ results' + three highlighted matches + advance") | FV scene: FV-L10 (timeline/Find slice on a seeded chat) — assert counter + emphasis at current main. |
| C-18 | AI source-unverifiable 4 (the four gated modals' Tab behavior at runtime) | **needs-runtime-verify** (unit-pinned by 019; the flows are gated — remote pairing/logout/plugin install need accounts or flows) | FV gap list (Linux lane): the four stay unit-pinned unless a flow-gated runtime path exists at the Lead station. |
| C-19 | F-A4-adjacent: the Files palette no-workspace early-return shape | **changed-surface** (the early-return now reports the honest status instead of silence) | `ui.rs:10054-10061` (the guard fires through `dispatch_command_status`). |
| C-20 | The rc.14 promo modal (ModelAvailabilityNux — fresh-profile keyboard swallowing; FCR honest note) | **fixed-by-UX-003** (provisional) | The modal now owns the 019-family trap + one-Escape: `ui.rs:43415-43474` (render with `tab_group`/`track_focus`/`key_context("ModelAvailabilityNuxModal")`) + test `ui.rs:55829-55859` (`model_availability_nux_modal_owns_the_keyboard_with_one_escape`). FV need: FV-L01/L20 fresh-profile boot includes the defensive dismissal + asserts the one-Escape contract. |

---

## D. f1-sweep journey-inventory rows (JI — the 18 journeys at base `a664644`)

The JI verdicts predate F2/Waves 2-6. Each row is reclassified to its
current-main state; the FV catalog (companion artifact) carries the scene
mapping, so this table records the STATE, not the scenes:

| ID | JI verdict (compact) | Verdict at `7f660c0` | Current-main evidence |
| --- | --- | --- | --- |
| D-1 | J-01 "partial, F1 core implemented" | **changed-surface** (the F2 UX-001 shell + the palette title entries landed; the workspace new-task surface is the primary entry now) | `flauz_shell/mod.rs` (Workspace navigation: Projects/Tasks, Procedures→Reusable workflows, Artifacts, Activity — the §2.1 shell); palette `ALL` = 93 rows (`ui.rs:3860`); title-bar entries `ui.rs:513-535`. Web: `web/lab/journeys.mjs:193` j-01. |
| D-2 | J-02 "partial analog only; future-phase" | **changed-surface** (the task rail's Context section exists with the honest "on its way" state; the /status bounded analog remains) | `flauz_shell/mod.rs:230-338` (TaskRailSection::Context + empty states); `/status` panel with live context usage `ui.rs:24451-24510` (session row + remaining % + window label). Web: j-02 (`journeys.mjs:260`). |
| D-3 | J-03 "partial, F1 recovery slice implemented" | **changed-surface** (the ORCH-003 recovery surface + the honest nothing-to-pick-up guidance landed — W3 d25) | `ui.rs` `FlauzRecoveryShortcut` chord (bind block `ui.rs:5780-5860`) + the scoped Escape `Some("FlauzRecovery")`; the reconnect family stands (`backend.rs:1797-1837`). Web: j-03 (`journeys.mjs:287`). |
| D-4 | J-04 "partial; the doctrine is implemented, full journey future-phase" | **changed-surface** (the CAP-001 capability-gap panel landed — W2 d24) | `ui/flauz_capability_gap.rs` (panel; honest pre-wiring copy; the view-model bound is cited in its module doc) + chord `ctrl-alt-shift-6` (bind block `ui.rs:5809`). Web: j-04 (`journeys.mjs:323`). |
| D-5 | J-05 "partial, local surfaces attachable; future-phase for providers" | **changed-surface** (ENV-001 fabric contracts + the rail's honest state; PROV-001 providers panel) | Rail Environments section (`flauz_shell/mod.rs:305` "No environments attached"); `ui/flauz_providers.rs` (PROV-001 surface). Web: j-05 (`journeys.mjs:365`) — the env gap card is protocol-named. |
| D-6 | J-06 "partial, F1 surfaces implemented; evidence linking future-phase" | **still-open** (the cross-boundary evidence LINKING remains future; the surfaces stand) | Terminal dock + browser panel machinery unchanged in kind (guards `lib.rs:8448-8457`, `9355-9361`); the Evidence rail section exists (`flauz_shell/mod.rs` Evidence). Web: j-06 (`journeys.mjs:400`). |
| D-7 | J-07 "partial analog; future-phase" | **changed-surface** (the ORCH-004 agents view landed — W3 d25; live wiring honestly not-yet) | `ui/flauz_agents_view.rs` (panel; "No agent is invented here before that wiring lands" — line 89 region) + chord Ctrl+Alt+Shift+7 (`ui.rs:5847` region). |
| D-8 | J-08 "partial, F1 approval/interruption slice implemented" | **changed-surface** (TAKE-001 needs-you/takeover surface landed — W5; live wiring honestly not-yet) | `ui/flauz_takeover.rs` (needs-you panel; the view-model bound in the module doc) + chord Ctrl+Alt+Shift+Y (`ui.rs:5870` region). Web: j-08 (`journeys.mjs:437`) — the aria-live approvals queue is REAL on web (real `UserInput` events). |
| D-9 | J-09 "partial analog; future-phase" | **changed-surface** (the Evidence rail section + the web artifacts/items panel) | Rail Evidence (`flauz_shell/mod.rs`); web `web/src/app/panels/ArtifactsPanel.tsx` (items panel over real `turn` items) + j-09 (`journeys.mjs:605`). |
| D-10 | J-10 "future-phase" | **changed-surface** (the ORCH-004 save flow landed with the honest not-wired state — W3 d25) | `ui/flauz_save_workflow.rs:115-121` (`NOT_WIRED_TITLE` "Saving isn't wired to live tasks yet" + "the steps that actually ran — exactly what happened, nothing aspirational") + chord Ctrl+Alt+Shift+S (`ui.rs:5855`). |
| D-11 | J-11 "future-phase" | **changed-surface** (the workflows library surface landed — the F2 shell + ORCH-004 list states) | Ctrl+Alt+2 surface (`ui.rs:5788` `FlauzReusableWorkflowsShortcut`; render `ui.rs:16449`); W3 d25 B14 evidence ("Reusable workflows" + "No reusable workflows yet"). |
| D-12 | J-12 "future-phase" | **still-open** (the run/deviation/improve loop is not wired on any client; the honest not-wired states are the verifiable slice) | The save panel's not-wired copy is the current truth (`flauz_save_workflow.rs:115-121`). FV catalog maps J-12 to the honest-state slice + names the full-loop N/A reason. |
| D-13 | J-13 "future-phase" | **changed-surface** (COL-001 members/presence + sharing landed — W4 d26; live transport honestly not-yet) | `ui/flauz_members.rs:120-127` (invite/roles not-wired) + the sharing surface (W4 d26 B12b evidence) + chord Ctrl+Alt+Shift+U (`ui.rs:5865`). Web: j-13 (`journeys.mjs:485`) with the membership/presence gap NAMED. |
| D-14 | J-14 "implemented (F1 parity-relevant)" | **changed-surface** (the MOD-001 picker panel + the identity promise landed on top — W2 d24) | `ui/flauz_model_picker.rs` (picker; the catalog/world-store seams at lines 173-279, 431 — honest empty "No models connected yet" + the J-14 identity promise); F1 continuity (thread/settings/update) stands (PM L80). Web: j-14 (`journeys.mjs:525`) — per-turn model params are REAL on web. |
| D-15 | J-15 "partial, worktree switch implemented" | **still-open** (the local worktree slice stands; provider/env switching remains honest-state) | "Continue in new worktree" (`ui.rs:13525`, `24273`, `25980`); rail Environments honest state (`flauz_shell/mod.rs:305`). Web: j-15 (`journeys.mjs:567`) — the per-turn cwd slice is REAL on web. |
| D-16 | J-16 "future-phase" | **changed-surface** (LEASE-001 conflicts surface landed — W5; live lease wiring honestly not-yet) | `ui/flauz_conflicts.rs` (the view-model bound in the module doc: "the app crate does NOT import the `flauz-lease` contract crate… the panel renders a plain view-model") + chord Ctrl+Alt+Shift+L (`ui.rs:5868`). |
| D-17 | J-17 "partial — attention slice implemented; the Activity view in-flight" | **fixed-by-WO-P2-013** (the Activity view landed, PR #36 → `c10b426`; provisional) | The `toggleActivityView` registry command (`ui.rs:3328`, dispatch `ui.rs:12316`) + the ActivityView keyboard rows (`ui.rs:5882-5885`) + the WO-P2-018 bell (`ui.rs:535`) + the TAKE-001 needs-you attention extension (`flauz_takeover.rs`). |
| D-18 | J-18 "future-phase" | **still-open** (no repeatability-observation surface exists on any client) | No automation-discovery surface in `crates/codex-app` or `web/` (grep-clean at `7f660c0`). FV catalog: N/A all lanes with the named reason. |

### D.19 Cold-start surfaces (JI §2)

| Surface | JI verdict | Verdict now | Evidence |
| --- | --- | --- | --- |
| Command palette | partial — no visible entry | **fixed-by-WO-P2-018** (provisional) | Title-bar entry `ui.rs:513-535` + `15571-15610`. FV-L01 asserts the visible entry. |
| Settings / Terminal / Browser / Browsing history | pass | **needs-runtime-verify** (re-prove at current main — the surfaces stand, the F1 evidence predates Waves 2-6) | FV-L20 re-runs the six-surface keyboard battery. |
| Attention | partial — view in-flight | **fixed-by-WO-P2-013 + TAKE-001** (provisional) | See D-17. FV-L18. |

---

## E. f1-sweep parity-inventory rows (PI — 41 rows + finding 16)

The F1 closure froze the classification at `7d1d61d`: **5 closed / 17 bounded /
11 P2 / 8 P3 = 41** (FCR clause 1) plus finding-16 (9 closed, 2 P3). The FV
reclassification carries each row forward against `7f660c0` WITHOUT reopening
the F1 freeze (the rows are Codex-parity rows; the Flauz-native surfaces
added by Waves 2-6 are beyond-parity additions that do not change the
Codex-parity class). Verdict semantics here: `frozen-class` = the F1 class
still describes the current Codex-parity surface (spot-cited);
`changed-surface` = a wave merge changed the described surface's shape.

**Row-level table (41 rows, PI §2 numbering):**

| PI row | F1 class (FCR) | FV verdict at `7f660c0` | Current-main spot citation |
| --- | --- | --- | --- |
| 1 Runtime bootstrap | closed | **frozen-class** | Runtime resolution order unchanged (platform-support.md; `main.rs` probe arg → `CODEX_RS_CODEX_BIN` path). |
| 2 App-server supervision | P3 | **frozen-class** | Supervision stands (`backend.rs:1797-1837`); network-aware diagnostics remain the polish tail. |
| 3 Projects and chats | P2 | **frozen-class** (the D-2 reconciliation landed: the row cites WO-P2-008/013 inline) | Unread state + Activity view closed (see D-17); multi-root source coverage remains P2. |
| 4 Thread execution | P3 | **frozen-class** | Compaction/edit-metadata polish tail (PM L78). |
| 5 Composer | P2 (bounded cloud/voice) | **frozen-class** | Slash set + catalog unchanged in kind (`ui.rs:9407` region — /status etc.). |
| 6 Model/effort/speed picker | closed | **frozen-class** | PM L80 contract stands; the Flauz picker (MOD-001) is beyond-parity addition. |
| 7 Permission profiles | P2 | **frozen-class** | Granular/custom editor remains P2. |
| 8 Streaming timeline | P3 | **frozen-class** | Timeline machinery unchanged in kind. |
| 9 Approvals and user input | bounded | **frozen-class** | Connector-gated. |
| 10 Repository status | P2 (pull) / P3 | **frozen-class** | Repository page stands. |
| 11 Diff review | P3 | **frozen-class** | Bounded diff review stands. |
| 12 Branches and worktrees | P3 | **frozen-class** | Worktree flows stand (`ui.rs:13525` family). |
| 13 Pull requests | bounded/P3 | **frozen-class** | gh-transport bound stands (`codex-platform/src/github.rs:342` region). |
| 14 Terminal | P3 | **frozen-class** | Terminal dock stands; PTY focus transfer (N5) is beyond-parity addition (`ui.rs:425-442`). |
| 15 Process Manager | bounded | **frozen-class** | Upstream-contract-gated. |
| 16 Computer Use | P2 (Windows) / bounded (Linux) | **frozen-class** | `computer_use.rs:34-57` bound unchanged. |
| 17 In-app browser | P2/P3/bounded | **frozen-class** | Browser panel + guards stand (`lib.rs:9355-9361`). |
| 18 Scheduled tasks | bounded | **frozen-class** | Proprietary backend. |
| 19 Marketplace admin-disabled install | closed | **frozen-class** | — |
| 20 Plugins marketplace | bounded/P3 | **frozen-class** | — |
| 21 Skills | P2/P3 | **frozen-class** | The web skills panel is a beyond-parity WEB surface (real `skill` UserInput — W6). |
| 22 MCP and apps | bounded | **frozen-class** | — |
| 23 Artifacts and files | P3 | **frozen-class** | The web items panel is a beyond-parity WEB surface. |
| 24 Sites | bounded | **frozen-class** | — |
| 25 Visualizations | bounded | **frozen-class** | — |
| 26 Appshots | bounded | **frozen-class** | — |
| 27 Settings | bounded/P3 | **frozen-class** | Settings registry stands (`ui.rs:2398-2417` region). |
| 28 Account and usage | P2 | **frozen-class** | — |
| 29 Feedback | closed | **frozen-class** | — |
| 30 Notifications and tray | P2/bounded | **frozen-class** | — |
| 31 Remote control and SSH | bounded | **frozen-class** | — |
| 32 Cloud environments | bounded | **frozen-class** | — |
| 33 Voice | bounded | **frozen-class** | — |
| 34 Personalization and memory | bounded | **frozen-class** | — |
| 35 Pets and Codex Micro | bounded | **frozen-class** | — |
| 36 Import and migration | bounded/P3 | **frozen-class** | — |
| 37 First run and updates | P2 | **frozen-class** | FR-1/FR-3 binding-passed at rc.14 (FRB); re-verify at current main in FV-L22 (FR slice). |
| 38 Keyboard and accessibility | in-flight → graduated closed (FCR) | **changed-surface** (the in-flight headline closed with WO-P2-012; the itemized P2/platform-bound tail is the open remainder) | See C-1..C-14. |
| 39 Active keyboard shortcut reference | closed | **frozen-class** | The Ctrl+/ overlay stands (`ui.rs:5741-5743` handler; PM L113). |
| 40 Windows | bounded | **frozen-class** | Unsigned-portable strategy stands; the FV Windows lane runs inside this bound (W7 §2). |
| 41 Linux | bounded (platform) | **frozen-class** | Platform bounds stand (`platform-support.md` §Linux; B-7/B-9/B-10). |

**Finding-16 shortcut delta (PI §3, 11 shortcuts):** 9 closed
(searchFiles/copyDeeplink/copyWorkingDirectory/renameThread/closeWindow/
archiveThread/newProjectlessTask/toggleThreadPin/navigateBack-Forward) +
2 P3 (Ctrl+Tab cycling, Switch to Work — reference+1). At `7f660c0`:
**frozen-class** for all 11 (the registry `KEYBOARD_SHORTCUT_COMMAND_IDS`
stands; `ui.rs:54597` asserts `toggleActivityView` membership — the family
grew, nothing regressed). needs-runtime-verify via FV-L20's chord ladder.

**PI §4 in-flight streams:** WO-P2-012 → closed (C-1); Activity view →
closed (D-17). **PI §5 arithmetic** unchanged by this reclassification
(41 = 5 + 17 + 11 + 8; the F1 freeze stands).

---

## F. f1-sweep release-gate-inventory anchors (RGI)

The RGI scene definitions remain the reference battery; the anchors are
re-verified at current main:

| ID | RGI anchor (compact) | Verdict | Current-main citation |
| --- | --- | --- | --- |
| F-1 | `ConnectionStatus` enum Offline/Connecting/Online/Recovering{attempt,retry_in_ms,last_error}/Failed | **still-implemented** | Footer labels `ui.rs:16277-16292`; failure surface `ui.rs:6216` ("Couldn't connect to the Codex app-server"); retry dispatch stands (RG-RECONNECT-05 path). |
| F-2 | Backoff scheduler 1/2/4/8/16/20 s, dedup, reset-on-success | **still-implemented** | `backend.rs:223-224` (1 s initial / 20 s max) + `backend.rs:1797-1837` (`AppServerReconnectScheduler`, schedule-dedup). **needs-runtime-verify**: FV-L04 (the RG-RECONNECT-01..03 re-run at current main). |
| F-3 | RG-RECONNECT-04 loaded/background recovery | **still-implemented** (unit-anchored) | The loaded-thread hydration + prioritization path stands (PM L76; tests `lib.rs:28546`/`36259` family). Runtime re-verify rides FV-L04 with the seeded-session bound named (FCR lab bound). |
| F-4 | RG-RECONNECT-06/07 (approval mid-recovery; Bedrock restart) | **needs-runtime-verify** (platform/mocked-bounded — unchanged bound) | The Bedrock restart path stands (PM L102; `ui.rs` Bedrock credential flow). FV: unit anchors + the named platform bound (FV gap list). |
| F-5 | Soak gates G-1..G-7 | **needs-runtime-verify** (release-gate scenes; adjacent to the FV journey set) | The FV catalog references the soak battery as the adjacent release gate (FV-L23 reference row); the soak itself stays a Lead-station release scene (time budget), not a journey scene. |
| F-6 | FR-1 About window / FR-3 Open-folder offer | **still-implemented** (binding-passed at rc.14 — FRB; provisional for current main) | About window `ui.rs:11550` (`title: "About codexRS"`) + menu `ui.rs:15378`; Open-folder offer + honest guard (C-2). **FV need:** FV-L22 (FR slice at current main). |
| F-7 | PKG-1..5 packaging checks | **still-open** (release-stage; the archives remain unsigned portable previews — B-9) | `--install-desktop-entry` `main.rs:47-59`; the release pipeline smoke (release.yml:194). FV-L22 carries the desktop-entry + archive-verify slice; signing stays F13. |
| F-8 | CI-1..3 the double matrix | **still-green** (the standing battery) | `ci.yml:24` (windows-latest + ubuntu-24.04), `ci.yml:105-111` (Linux startup smoke). W6 gate: both platforms green at the wave head. |

---

## G. F1-CLOSURE-RECORD honest notes (FCR)

| ID | Note (compact) | Verdict | Current-main evidence |
| --- | --- | --- | --- |
| G-1 | The rc.14 promo modal (fresh-profile keyboard swallowing; Escape the verified dismissal) | **fixed-by-UX-003** (provisional — see C-20) | `ui.rs:43415-43474` + test `ui.rs:55829-55859`. FV-L01/L20 include the defensive dismissal + one-Escape assert. |
| G-2 | N5 PTY focus transfer | **fixed-by-UX-003** (provisional — see C-16) | `ui.rs:425-442` + `55817-55826`. FV-L20 PTY probe. |
| G-3 | N6 bracket-swap chords not re-evidenced positively in the lab config | **needs-runtime-verify** | The shifted-symbol companions are in the bind block (`ui.rs:5837-5841` `alt-!`..`alt-%` + `alt-^` at `5846`) with the xkb rationale comment (`ui.rs:5825-5836`). FV-L20 re-evidences Ctrl+PageDown + the bracket chords at current main. |
| G-4 | Lab bounds: the native folder picker (no portal on Xvfb) | **still-open** (environment bound) | No portal backend exists on the Xvfb lab (E2B; FRB bounded note). FV gap list Linux lane. |
| G-5 | Lab bounds: the four flow-gated 019 modals | **still-open** (unit-pinned bound — see C-18) | FV gap list. |
| G-6 | Lab bounds: the seeded-session timeline emptiness (RG-RECONNECT-04's note) | **still-open** (fixture bound) | FV-L04 carries the bound; donor-state pattern is the mitigation (wo-p2-008 precedent). |
| G-7 | Lab bounds: the auth wall (live-chat turns error honestly in the lab — the fail-closed family) | **still-open** (the load-bearing lane bound) | Every FV live-turn scene is bounded to honest-state slices + unit anchors (FV gap list, all three lanes). |

---

## H. codex-linux official-side observations (CL — the 8 load-bearing findings)

These are reference-evidence findings about the OFFICIAL app (unauthenticated
Linux preview 26.908.70816). Reclassified against current main = does the
observation still hold as reference evidence, and what is Flauz's current
counterpart:

| ID | CL finding (compact) | Verdict | Current-main counterpart |
| --- | --- | --- | --- |
| H-1 | Auth gates the entire official shell (login surface only) | **still-open** (reference evidence; retained as a provenance label) | Flauz's unauthenticatable-but-explorable shell remains a real, evidenced difference (the FV cold-start scenes ride it). |
| H-2 | Official startup hard-depends on the bundled runtime (fatal without it) | **still-open** (reference evidence) | Flauz degrades gracefully (`backend.rs:1797-1837`; smoke script proves 15 s survival under `/bin/false` — `scripts/linux_desktop_smoke.sh:44-50`). Flauz stays ahead on this axis. |
| H-3 | Official palette indexes settings pages at login | **still-open** (reference evidence) | Flauz matched it (WO-P2-004; palette Settings group — `ui.rs` palette rows incl. `OpenImportSettings` at `ui.rs:3871`, title `4010`, hint `4126`). |
| H-4 | Official shortcuts overlay carries 22 rows incl. bindings absent from Flauz; "Flauz has no shortcuts overlay surface — new gap" | **changed-surface** (Flauz HAS the searchable Ctrl+/ overlay at current main; the binding delta is reclassified by finding-16 — E above) | The overlay stands (`ui.rs:5741-5743` toggle handler; PM L113 done row; ev/15-era capture + current source). The residual binding delta = the 2 P3 reference+1 items (E). |
| H-5 | DB contract `$HOME/.codex/sqlite/codex-dev.db` (better-sqlite3) + recovery surface | **still-open** (reference evidence; Flauz never opens it directly) | The AGENTS law stands (supervised app-server only; isolated CODEX_HOME in every lab/CI scene — `linux_desktop_smoke.sh:29-35`). |
| H-6 | Official terminal backend node-pty; cua_node payload present though CU is docs-absent on Linux preview | **still-open** (reference evidence) | Flauz uses `portable_pty` (`codex-platform/src/terminal.rs:19` region); no payload-vs-availability question exists on the Flauz side. |
| H-7 | Window/userData identity (class "ChatGPT"; `codex://` registration) | **still-open** (reference evidence) | Flauz's window identity unchanged in kind (codexRS). |
| H-8 | Official login-surface binding no-ops (Ctrl+Shift+O, Ctrl+F, Ctrl+Alt+O — byte-identical frames) | **still-open** (reference evidence) | Flauz's guarded chords answer honestly instead (the A-3 family — C-1/C-2; `lib.rs:9750-9757`). The FV chord ladder asserts the honest answers. |

---

## I. Wave-gate honest deferrals (W2..W6)

| ID | Deferral (compact) | Verdict | Current-main evidence |
| --- | --- | --- | --- |
| I-1 | W2: the picker's F7 wiring seams (set_catalog/active_model_id/events/availability) | **still-open** (wiring backlog; the BYOP slice wired the providers panel, not the picker's live catalog) | `flauz_model_picker.rs:173-279` (the catalog seam + world-store seam comments), `:431` (`set_catalog` — the seam, not live data). |
| I-2 | W2: the gap surface's view-model wiring (live CapabilityResolution records) | **still-open** | `flauz_capability_gap.rs` module doc: "the app crate does not import the flauz-cap contract crate in this wave: the panel renders a plain view-model". |
| I-3 | W2: the environment rail's live wiring (F5+) | **still-open** | `flauz_shell/mod.rs:305` (the honest "No environments attached" empty state is the current truth). |
| I-4 | W3: the palette "Task evidence" Return from a shell-surface focus context (the d23 minor note) | **changed-surface** (the rail's Evidence section + the 017 focus contract changed the shape; the specific d23 edge is absorbed by the 017 restore contract) | `ui.rs:10082-10105` (the restore contract) + the rail Evidence palette rows (`ui.rs:4049`). **needs-runtime-verify**: FV-L20 includes a palette→rail-surface→Return focus probe. |
| I-5 | W3: the recovery banner's recoverable-state path + agents/save panels' live wiring | **still-open** (engine-side proven at Gate A; GUI view-models await harness wiring) | `flauz_agents_view.rs:89` region + `flauz_save_workflow.rs:115-121` (the honest not-wired states). |
| I-6 | W4: wire providers/members UI seams to real crate types; scheduling choices on live tasks; the real local lab driver; F10+ transport | **still-open** (backlog) | `flauz_providers.rs` (PROV-001 surface; the registration-seam comments at `:1507`); `flauz_members.rs:120-127`. |
| I-7 | W5: the live human-in-the-loop wiring (needs-you/conflicts view-models; the six event payloads are the frozen wiring surface) | **still-open** | `flauz_takeover.rs:55-63` (view-model bound) + `flauz_conflicts.rs` (the same bound). |
| I-8 | W5+W6: the duplicated FlauzConflicts listener remnant in ui.rs (~45896; a LEASE-001 gate-fix leftover; harmless — re-registers the same listener) | **still-open** (verified present at `7f660c0` — the one line-level product-code finding this reclassification keeps open) | The LEASE-001 registration `ui.rs:45906-45908` + the stale duplicate `ui.rs:45928-45930` (orphaned comment `ui.rs:45921-45927`). Cosmetic/harmless (idempotent re-register); cleanup belongs to the next ui.rs-touching wave — NOT this docs-only order. |
| I-9 | W6: the web named protocol gaps (environment listing/control; membership/presence; artifact listing) — truthful gap cards, protocol-work-order recovery paths | **still-open** (protocol-gated by design; never a fabrication) | The gap engine: `web/src/state/capabilities.ts:1-80` (namespace-derived from the generated `REQUEST_METHODS`); the gap cards: `web/src/app/panels/EnvironmentsPanel.tsx`, `CollaboratorsPanel.tsx`, `ArtifactsPanel.tsx`. Recovery path: additive protocol work orders (deviation-gated) per the W6 record. |
| I-10 | W6: formal verification itself (the lab evidence is mock-gateway on the worker side; the Lead reruns with the real gateway) | **needs-runtime-verify** (THIS wave) | The formal pass contract: `web/lab/journeys.mjs:8-17` (the two transports; `--gateway "<cmd>"` spawns the real Rust gateway) + W7 §1 (mock captures are NOT formal evidence). FV-E01..E13 run with `--gateway`. |
| I-11 | W6: the WEB-001 SPA absolute-path leakage (fixed by the Lead gate: `//etc/passwd` → named 404 refusal) | **fixed-by-the-WEB-001-Lead-gate** (PR #57; provisional) | `crates/flauz-web-gateway/src/static_files.rs:99-107` ("Normalizes a request path to a safe relative path, refusing traversal (`..`, absolute components, backslash tricks, NUL…"; the 2026-09-25 gate-fix comment names `//etc/passwd`). **FV need:** FV-E00 (the gateway security slice: traversal refusal + healthz + localhost bind). |
| I-12 | W6: the gateway's localhost-only default bind | **still-implemented** (a security property to keep) | `crates/flauz-web-gateway/src/config.rs:12-20` (`DEFAULT_BIND_LABEL: "127.0.0.1:8610"`, "Non-local binding is an [explicit opt-in]") + test `config.rs:368`. FV-E00 asserts it. |

---

## J. FLAUZ-REFERENCE-MATRIX sharpened findings (FRM, rc.13 `f113515`)

| ID | FRM finding (compact) | Verdict | Current-main evidence |
| --- | --- | --- | --- |
| J-1 | (a) Terminal/browser toggles are chat-scoped silent no-ops on the entry surface | **fixed-by-WO-P1-001/002** (the honest toggles; provisional) | `lib.rs:9750-9757` (terminal; the WO-P1-001 comment) + `lib.rs:9355-9361` (browser); tests `lib.rs:30162` / `30191`. FV-L06 asserts the honest statuses at runtime. |
| J-2 | (b) The command palette does not index all settings pages ("import" → No matches while the Import page exists) | **fixed-by-WO-P2-004** (merged `7aa7163`; provisional) | The palette Settings group indexes the default-nav sections incl. Import: `ui.rs:3871` (`Self::OpenImportSettings` in `ALL`), title `ui.rs:4010` ("Import"), hint `ui.rs:4126` ("Open Import settings"). FV-L20 palette drill asserts it. |
| J-3 | (c) The plugins marketplace syncs to disk while the catalog UI reports empty unauthenticated | **still-open** (a documented auth/gh bound, not a defect) | Unchanged shape at `7f660c0` (the marketplace catalog is account/gh-bound; the honest empty state is the truth). FV gap list (auth wall). |
| J-4 | FRM KNOWN FINDING 1 — terminal discoverability: shortcut-only, no persistent affordance | **changed-surface** (the WO-P1-001 persistent sidebar affordance + the F2 shell landed) | The sidebar terminal affordance (WO-P1-001, `d15333e`) + the honest toggle (J-1). FV-L06/L20 verify. |
| J-5 | FRM KNOWN FINDING 2 — browser discoverability: palette/shortcut-only | **changed-surface** (the WO-P1-002 persistent sidebar affordance landed) | Same family as J-4. |
| J-6 | FRM KNOWN FINDING 3 — Linux Computer Use is a platform gap, not a missing feature | **still-open** (platform-limited by design — unchanged) | `computer_use.rs:34-57` (B-7). |

---

## Coverage arithmetic

| Ledger | Findings of record | Rows in this file |
| --- | --- | --- |
| AES expectations families | 7 | A-1..A-7 ✓ |
| KF controls | 6 | B-1..B-6 ✓ |
| KF active limitations | 5 | B-7..B-11 ✓ |
| accessibility-inventory | 2 defect families + 7 gaps + 4 platform-bound + 4 source-unverifiable + F-A5 + the FCR promo note | C-1..C-20 ✓ (F-A5 = C-3; the promo note = C-20; the F-A1..F-D2 family = C-1; F-A4 = C-2/C-19; gaps C-4..C-10; platform-bound C-11..C-14; unverifiable C-15..C-18) |
| journey-inventory | 18 journeys + 6 cold-start surfaces | D-1..D-18 + D.19 ✓ |
| parity-inventory | 41 rows + 11 finding-16 shortcuts + 2 in-flight streams | E (41-row table + finding-16 paragraph; the streams fold into C-1/D-17) ✓ |
| release-gate-inventory | 8 anchor families | F-1..F-8 ✓ |
| F1-closure honest notes | 7 | G-1..G-7 ✓ |
| codex-linux | 8 | H-1..H-8 ✓ |
| wave gates w2..w6 | 12 deferral items | I-1..I-12 ✓ |
| FLAUZ-REFERENCE-MATRIX | 3 sharpened + 3 known findings | J-1..J-6 ✓ |

Every finding appears exactly once (fold-rows name their fold). No finding
was closed on static reasoning alone: every `fixed-by` verdict is marked
provisional with its FV-003 runtime-verification scene named; every
`needs-runtime-verify` names the deciding scene; the synchronization law
(AES L90-108) governs closure.

## Honesty notes

1. This file contains **zero product-code changes** (git-diff-proven: only
   `docs/research/evidence/fv-001/**` + `docs/research/FV-CATALOG.md`).
2. Line citations were read at the pinned checkout `7f660c0`; they are
   current-main lines, volatile by design — the symbol names are the stable
   anchors (the house convention).
3. The one line-level product-code defect kept open (I-8, the duplicated
   listener) is recorded, NOT patched — this order is docs-only; the fix
   belongs to the next ui.rs-touching wave.
4. The F1 parity freeze (E) is carried forward, not reopened: the Wave-2..6
   Flauz-native surfaces are beyond-parity additions, and re-slicing the F1
   classes is a Lead decision, not a worker reclassification.
