# F1 closure packet — release-gate inventory (WO-F1-SWEEP-001)

- **Work order:** WO-F1-SWEEP-001 (Wave 1, Worker C — evidence-only)
- **Base:** `a664644718210e254952db518606d80906eee448`
- **Date:** 2026-09-19
- **Purpose:** define the concrete reconnect/recovery, performance/soak, and
  fresh-machine/release validation procedures the Tech Lead's lab must
  exercise for the F1 close gate ("release-candidate soak/reconnect/
  performance gates" — docs/IMPLEMENTATION-ROADMAP.md F1 Remaining). This
  packet DEFINES the scenes; the Lead runs them.

Anchor conventions: `PM` = docs/parity-matrix.md, `KF` = docs/known-failures.md,
source = `symbol (file:~line)` (line numbers volatile; symbol is stable).

## (a) Reconnect / recovery — state enumeration and validation procedures

### Enumerated supervision states (source-anchored)

| State | Meaning | Source anchor |
| --- | --- | --- |
| `Offline` | no backend/session yet | `ConnectionStatus` enum (lib.rs:242–252) |
| `Connecting` | initial connection attempt in flight | same |
| `Online` | app-server initialized and connected | same; footer label "App-server online" (ui.rs:14829–14833) |
| `Recovering { attempt, retry_in_ms, last_error }` | unexpected local exit; bounded backoff retry cycle | same; footer labels "Reconnecting…" (attempt 0), "Retry {attempt} in {n}s", "Reconnecting · attempt {attempt}" (ui.rs:14837–14847); bounded tooltip "The Codex app-server stopped. codexRS will retry automatically." + `last_error` (ui.rs:14848–14852) |
| `Failed(String)` | initial startup failure — explicitly retryable | same; footer "Click to retry." when a backend exists, else "Restart codexRS to try again." (ui.rs:14849–14860); failure surface text "Couldn't connect to the Codex app-server" (ui.rs:5332); `Action::RetryConnection` dispatched from the footer click (ui.rs:14982) and the empty-Tasks Retry action (ui.rs:17697) |
| Backoff scheduler | one deduplicated reconnect; delay doubles from 1 s, capped at 20 s → the bounded **1/2/4/8/16/20**-second timer; `reset()` restores attempt 1 / 1 s | `AppServerReconnectScheduler` (backend.rs:1796–1837); `APP_SERVER_RECONNECT_INITIAL_DELAY` 1 s / `APP_SERVER_RECONNECT_MAX_DELAY` 20 s (backend.rs:223–224); dedup: `schedule()` returns None while a reconnect is pending (backend.rs:1805–1808); reset only on successful initialization (backend.rs:3671, 3694, 3803 `Ok(()) => app_server_reconnect.reset()`) |
| Loaded/background session recovery | bounded `thread/loaded/list` page hydrated through `thread/read`, prioritized in the task list, and resumed after connect | PM §Product parity › App-server supervision (L76); tests `loaded_threads_are_prioritized_and_resumed_once_after_reconnect` (lib.rs:28546) and `repeated_connection_loss_queues_only_one_reconnect` (lib.rs:36259) |

PM L76 records that "the timer progression, repeated-loss race,
blocked-restart path, and automatic return online were exercised against the
pinned CLI in an isolated `CODEX_HOME`" during the supervision work — the
procedures below re-run that class of exercise as a release gate on the
release candidate binary.

### Validation procedures (LINUX_GUI_LAB; isolated `CODEX_HOME` + pinned
official CLI `0.146.0-alpha.3.1`, `CODEX_RS_CODEX_BIN` per the RWO-022 FW-7
lab restore pattern)

| Scene | Procedure | Pass criteria (evidence) |
| --- | --- | --- |
| **RG-RECONNECT-01 — mid-session loss, timer progression** | start the app with a live supervised app-server and a selected chat; kill the app-server child process; observe the sidebar footer for ≥ 5 consecutive retry cycles without user input | footer shows `Reconnecting…` → `Retry 1 in 1s` → `Retry 2 in 2s` → `Retry 4 in 4s` → `Retry 8 in 8s` → `Retry 16 in 16s` → `Retry N in 20s` (cap holds); bounded tooltip present; frames + footer text VLM-read; log extract archived |
| **RG-RECONNECT-02 — repeated-loss dedup (the race)** | while a reconnect is pending, kill the (respawning) child again before the due attempt; repeat 3× | exactly one queued reconnect at any time (no parallel spawns, no attempt-number jumps > 1 per cycle); unit anchor `repeated_connection_loss_queues_only_one_reconnect` (lib.rs:36259) stays green |
| **RG-RECONNECT-03 — automatic return online + timer reset** | after RG-RECONNECT-01/02, let a retry succeed; then kill again | footer returns to "App-server online"; the NEXT loss restarts at attempt 1 / 1 s (reset only after successful initialization — backend.rs:3803); selected chat timeline reloads authoritatively (`reconnect_reloads_the_selected_timeline_authoritatively`, lib.rs:36381) |
| **RG-RECONNECT-04 — loaded/background session recovery** | with ≥ 2 loaded/background chats (one mid-turn), kill the child; let reconnect complete | loaded sessions rehydrate via `thread/loaded/list` → `thread/read`, are prioritized in the task list, and resume active-turn state/subscriptions (PM L76; lib.rs:28546) |
| **RG-RECONNECT-05 — initial startup failure is retryable** | launch with an unreachable/invalid `CODEX_RS_CODEX_BIN` (or blocked spawn) so the FIRST initialization fails | empty-Tasks area shows the failure state ("Couldn't connect to the Codex app-server", ui.rs:5332) with the Retry action; a missing backend yields restart guidance (PM L111); after fixing the runtime path, Retry (ui.rs:17697/14982) recovers to Online without app restart |
| **RG-RECONNECT-06 — approval mid-recovery** | trigger an approval request in a non-selected chat, then kill the child during the request | approval request survives reconnection (or is re-derived); no phantom approval card; footer states as above |
| **RG-RECONNECT-07 — Bedrock-login restart path (bounded)** | (Windows-or-mocked) accepted Bedrock login restarts the supervised app-server | exactly one runtime instance at all times; restart status cleared on success (`accepted_bedrock_login_reconnects_and_clears_the_restart_status`, lib.rs:32854); PM L102 ("A failed restart never launches a second runtime") |

## (b) Performance / soak — profile, measurable gates, baseline regression set

### Baseline regression set (standing controls — must stay green)

The six stable-failure regression controls (PR §5.10 row "Stable-failure
regression controls"; KF table) are the baseline set for every soak run:

| Control | Bound | KF anchor |
| --- | --- | --- |
| Windows multi-root white screen | native `Path`/`PathBuf`, no browser path shim | KF row 1 |
| 594 MB JSONL line | live history only through bounded app-server pages; no direct live JSONL reads | KF row 2 |
| ~9 GB startup history scan | `thread/list` paginated, always `useStateDbOnly: true` | KF row 3 |
| `git.exe` process storm | 300 ms debounce, notification coalescing, one backend Git operation at a time | KF row 4 |
| Process-cleanup storm | one supervised tree, graceful shutdown, one bounded fallback; Job Object on Windows | KF row 5 |
| Unbounded logging | no provider log duplication; narrowly scoped owned state | KF row 6 |

### Soak profile (one long session, many chats, bounded pages, streaming)

- **Session length:** ≥ 4 h continuous run (stretch: 8 h overnight), no app
  restart; window stays open (may be backgrounded for part of the run).
- **Chat population:** grow to ≥ 100 chats (bound: visible threads truncate
  at `MAX_VISIBLE_THREADS` = 500, lib.rs:12 + lib.rs:8107) across ≥ 5 local
  projects; ≥ 10 chats with full multi-page history (default metadata page
  20, max 100 — KF §Current budgets).
- **History paging:** scroll each long chat to its head through bounded
  `thread/items/list` pages (guards: 2,000-item / 32-page / 1,000-match caps
  per PM L112 Find row).
- **Streaming timeline:** ≥ 20 assistant turns with variable-height content
  (code blocks, generated-image placeholders, expandable summaries) streamed
  live; Find exercised against the fully loaded timeline.
- **Terminal/browser load:** ≥ 2 concurrent terminal tabs per chat bound
  (`MAX_TERMINAL_TABS` 16 / per-task 8, lib.rs:~118–120) with continuous
  output; browser panel streaming ≥ 30 min with one download cycle.
- **Bounded queues:** run the PR-list surface (100-item pages, 1,000
  accumulated cap) and the Outputs inspector (128-artifact bound) during the
  session.

### Measurable gates (pass/fail, recorded per run)

| Gate | Measure | Pass criterion |
| --- | --- | --- |
| G-1 Memory bound | process RSS sampled every 15 min | no monotonic unbounded growth: RSS returns to a stable band after transient loads; sustained growth over the last hour < 10% of the first-hour baseline (image cache warm-up excluded, one-time) |
| G-2 Frame/stream latency | UI frame pacing (GPUI) and first-token/stream-chunk latency for ≥ 20 turns | no sustained UI freeze > 250 ms while streaming; stream chunks render incrementally (visual frame sequence archived) |
| G-3 No unbounded growth | owned-storage row counts, event-queue depths, terminal scrollback | every store respects its KF §Current budgets page (500-row owned-storage page; 256 terminal events; 16 MiB frames); no store grows without bound across the 4 h window |
| G-4 Baseline controls | the six regression controls | all six stay green in the workspace battery (CI double matrix) at the release-candidate SHA |
| G-5 Reconnect under soak | kill the app-server child at T+2 h | recovery completes within the bounded 1/2/4/8/16/20 schedule; loaded-session recovery per RG-RECONNECT-04; no duplicated subscriptions afterwards |
| G-6 Attention/state correctness under soak | background completions in non-selected chats across the run | unread-attention flags set/clear correctly (visit clears, archive drops — WO-P2-008 semantics); bounded `needs_attention_task_ids` (capped at 500) |
| G-7 No silent error accumulation | stderr/log tail of the app process | zero panic/unwind; bounded diagnostics only (KF row 6) |

Suggested scene names: **RG-SOAK-01** (long-session memory),
**RG-SOAK-02** (many-chats + bounded pages), **RG-SOAK-03** (streaming
timeline stress), **RG-SOAK-04** (terminal/browser tabs long-run),
**RG-SOAK-05** (reconnect-under-soak composite). Each scene archives: the
scene script, a sampling log (RSS + timestamps), md5 manifest of frames, and
VLM reads of the first/last/mid frames (house evidence format — see
`docs/research/evidence/wo-p2-011/` for the packaging convention).

## (c) Fresh-machine / release validation — checklist

### First-run path (PM §Product parity › First run and updates, L111)

| Check | Expected |
| --- | --- |
| FR-1 About window | Help → `About codexRS` opens the native fixed-size 380×360 floating window, centered on parent, reports package version + copyright, refocuses existing instance, closes via OK/Escape/native close |
| FR-2 failed initial backend/app-server connection | remains visible in the empty Tasks area; app-server path offers Retry; missing backend directs the user to restart (RG-RECONNECT-05) |
| FR-3 signed-in empty workspace | offers `Open folder` reusing the native folder picker + explicit-workspace handoff |
| FR-4 runtime resolution on a fresh machine | resolution order honored (platform-support.md §Windows: explicit probe arg → `CODEX_RS_CODEX_BIN` → hash-pinned runtime copy → npm global package → `codex`/`codex.exe` on `PATH`); the exact packaged CLI hash check and override/fallback order preserved (PM L75) |

### Install / packaging checks (per docs/platform-support.md; GUI-006 ledger rows, PM L344–345)

| Check | Expected |
| --- | --- |
| PKG-1 Windows archive | unsigned portable ZIP; both executables kept together (`codexrs.exe` + `codex-computer-use-overlay.exe` sibling — the overlay is a fixed sibling path, PM L90); SHA-256 checksums published; documented verify step reproduces |
| PKG-2 Linux archive | unsigned portable tar.gz; `codexrs --install-desktop-entry` creates the per-user entry for the current extracted binary, captures an absolute `CODEX_RS_CODEX_BIN` when the CLI is off the desktop session PATH, and never overwrites an existing entry |
| PKG-3 documented bounds | no installer/uninstaller/URI registration/in-app updating is expected (KF §Active release-candidate limitations); Linux tray/global shortcuts absent by design |
| PKG-4 Linux Computer Use bound | attaches only with `DISPLAY` set; screenshot-only X11/XWayland observation; honest settings copy |
| PKG-5 first-launch smoke from the archive | Ubuntu CI pattern: start the extracted archive in an isolated Xvfb session (platform-support.md §Linux); broader desktop-environment smoke coverage remains pending — record which environments were exercised |

### CI matrix as the release battery

| Check | Expected |
| --- | --- |
| CI-1 | `cargo fmt --all --check` clean (AGENTS.md L24–25) |
| CI-2 | `cargo clippy --workspace --all-targets` clean on both matrices |
| CI-3 | `cargo test --workspace` green on the double matrix (windows-latest + ubuntu-24.04 — the house battery every closed WO ran; FW changelog) |
| CI-4 | No new `platform`-row regressions: the Linux row's bounded surface (packaging/tray/global-shortcut absences) stays documented, not silently changed |
| CI-5 | Release SHA recorded with the run IDs of CI-1..CI-3 plus the RG-* scene evidence bundle for the exact binary (md5 manifests house convention) |

### Release-candidate sign-off order (suggested)

1. CI-1..CI-3 on the release SHA.
2. RG-RECONNECT-01..05 (07 bounded to its platform) on the release binary.
3. RG-SOAK-01..05 (at least 01/02/05 full-length; 03/04 may share the session).
4. FR-1..FR-4 + PKG-1..PKG-5 from the published archives on a clean profile
   (isolated `CODEX_HOME`; never the live `~/.codex` — AGENTS.md L11–14).
5. The a11y keyboard-only pass scenes (see accessibility-inventory §
   source-unverifiable notes and recommendations.md RG-A11Y-01/02).

## Scene index (all LINUX_GUI_LAB, evidence to
`docs/research/evidence/f1-sweep/` or a Lead-named successor)

RG-RECONNECT-01..07 · RG-SOAK-01..05 · RG-FRESH-01..04 (FR checks) ·
RG-PKG-01..05 · RG-A11Y-01..02 (cross-file). Every scene ships: the script,
an md5 frame manifest, and VLM reads of load-bearing frames.
