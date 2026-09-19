# RWO-020 — Reference-Behavior Review Rubric

> **Deliverable of WO-REVIEW-001 (deep phase), review-worker slot RWO-020.**
> Base: `main @ d479c7b9e725e1af1353140d49226b3f9a16be9d` (verified via
> `git rev-parse …^{commit}` on the cloned repo). Branch: `research/rwo-020`.
> Scope: the 8 `complete` + the 2 highest-traffic `partial` rows of the canonical
> parity matrix (`docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` §5):
> Runtime bootstrap; Side chats; Keyboard shortcut reference; Feedback;
> Multi-root workspace; Settings shell; Projects and chats; Keyboard and
> accessibility; Notifications and tray; Activity view & unread attention.
> **This rubric is the contract Worker B executes and Worker C attacks.** It was
> derived ONLY from the repository's own reference evidence (citations below);
> no Flauz behavior was verified during rubric construction (per the WO split).

## 0. Rules of use (binding on B and C)

1. **Honesty over coverage.** A criterion that cannot be executed in the
   available lab is recorded `NOT RUN` with the exact reason — never simulated,
   never mocked. The no-fabrication doctrine applies verbatim:
   > "no codex CLI exists in this lab, and the no-fabrication doctrine forbids
   > mocking a runtime." — `docs/research/evidence/wo-p2-008/README.md:14-16`
2. **Platform honesty.**
   > "Linux runtime evidence never closes a Windows row and vice versa.
   > Windows/macOS GUI verification is deferred until WINDOWS_GUI_LAB /
   > MACOS_GUI_LAB exist… such rows close only on the evidence classes
   > available to them and stay labeled." — `FEATURE-PARITY-WORK-ORDERS.md:52-57`
3. **Source presence is never completion.**
   > "Source-code presence is NEVER 'feature complete'. An implemented backend
   > or an unreachable UI surface is not a closed gap; discoverability and GUI
   > verification are first-class closure conditions." — `FEATURE-PARITY-WORK-ORDERS.md:49-51`
4. **Never a silent no-op** (input-quality doctrine, WO-P2-007): every
   advertised binding must resolve visibly or carry an honest guard status.
5. **Version-skew discipline.** A fact tagged **ref+1 (26.908+)** may NEVER be
   cited as the parity bar (26.825.51511 / baseline 26.721.3996.0). Delayed
   cloud capabilities (unified pinned threads, shared thread snapshots,
   Activity view *surface*, remaining Settings sections behind host contracts)
   are **documented deferrals, not defects** — they must not be counted against
   parity, and equally must not be quietly marked done.
6. **Frame discipline** (B2): GUI scenes capture numbered frames, md5-check
   them, and VLM-read the load-bearing ones; "identical frame" statements mean
   byte-identical captures.

### 0.1 Version key

| Tag | Meaning | Evidence base in this repo |
| --- | --- | --- |
| **baseline** | Codex Desktop 26.721.3996.0 (captured 2026-07-24; bundled CLI 0.146.0-alpha.3.1, SHA-256 pinned in `reference/stable-26.721.3996.0/manifest.json`) | `docs/parity-matrix.md` (PM), `docs/known-failures.md` (KF), A-matrix ● rows |
| **current** | ChatGPT desktop 26.825.51511 (2026-08-25 release line) — the parity TARGET | changelog deltas ≤ 2026-08-25 (`codex-ref/changelog-delta-extract.md`, `version-delta-notes.md`) |
| **ref+1** | 26.908+ (linux preview package 26.908.70816, built 2026-09-14) — OUT of target; forward-looking notes only | `codex-linux/*` runtime captures |
| **marker** | Intermediate introduction versions (26.707 / 26.715 / 26.727 / 2026-08-11 / 2026-08-20) — facts introduced between baseline and current, hence IN target | changelog lines cited per row |

### 0.2 Evidence-class key

- **source-read** — static reading of `crates/**` at the SHA under review.
- **unit-test** — focused test execution (`cargo test -p …` filtered); per-test
  output recorded.
- **GUI scene** — LINUX_GUI_LAB scene (Xvfb + picom, isolated
  HOME/XDG/CODEX_HOME/CODEX_RS_DATA_DIR, no credentials), numbered frames +
  md5 + VLM reads. Scene scripts reuse the archived per-WO scripts where they
  exist.
- **runtime overlay** — behavior observable only with a live official runtime
  (codex CLI present). Currently bounded: no codex CLI in lab → expected
  `NOT RUN` unless the lab is upgraded (§8.4 lab-upgrade opportunity of the
  parity report).

### 0.3 Execution protocol for B (cheapest-first)

Per criterion: run the cheapest class first (source-read < unit-test < GUI
scene < runtime overlay); record `VERIFIED` / `DEFECT-FOUND` / `NOT RUN +
reason`; attach `file:line` anchors or frame+md5+VLM evidence; never infer a
status the evidence class cannot carry. B does not edit statuses in the parity
report — findings flow to the Tech Lead.

### 0.4 Attack surface for C (general)

C attacks: (a) rows whose deep claims are `[historical-record]`-only (Feedback,
Runtime bootstrap hash-check, Keyboard shortcut reference "active bindings",
Notifications completion-banner); (b) platform-cell laundering (Windows rows
"closed" by Linux evidence); (c) ref+1 facts smuggled in as the bar; (d)
advertised-while-guard-disabled bindings (WO-R-SWEEP found 16 such rows);
(e) honest-status regressions (guards silently removed). Per-row attack notes
below.

## 0.5 Row index

| ID | Row | Matrix status (§5) | Version tag of the bar | Weakest evidence class to defend |
| --- | --- | --- | --- | --- |
| RUB-01 | Runtime bootstrap | complete (§5.1, L163) | baseline (hash pin) + linux runtime | source-read (hash pin is PM-only claim) |
| RUB-02 | Side chats | complete (§5.3, L170) | baseline ● (timing medium-high) | GUI scene |
| RUB-03 | Keyboard shortcut reference | complete (§5.10, L252) | baseline (PM wording) | source-read × GUI cross-check |
| RUB-04 | Feedback | complete (§5.10, L254) | baseline (PM-only deep facts) | GUI click-through (never exercised) |
| RUB-05 | Multi-root workspace handling | complete (§5.6, L210) | baseline (KF failure record) | platform honesty (win cell) |
| RUB-06 | Settings shell | partial (§5.9, L239) | baseline + 2026-08-11 marker | source-read (registry bound) |
| RUB-07 | Projects and chats | partial (§5.1, L165) | baseline + 26.715 marker | unit-test + GUI scene |
| RUB-08 | Keyboard and accessibility | partial (§5.10, L251) | baseline + undated post-baseline rows | unit-test + GUI scene |
| RUB-09 | Notifications and tray | partial (§5.10, L253) | baseline | source-read (banner claims HR-only) |
| RUB-10 | Activity view & unread attention | partial (§5.10, L260) | 26.727 marker (in target) | runtime overlay (view surface auth-walled) |

---

## RUB-01 — Runtime bootstrap (matrix: complete)

**Reference version:** baseline 26.721.3996.0 (`[historical-record]` PM);
linux boot `[runtime-observed]` B2 ev/06. The 26.908 hard-fatal contrast is
**ref+1 — NOT the bar**.

**Official behavior (from the repo's own reference evidence):**
> Packaged-CLI hash pin + explicit override/fallback order — parity report
> §5.1 L163, citing PM "Runtime bootstrap" (`docs/parity-matrix.md`).
> Layering fact (A-matrix L38): bundled CLI 0.146.0-alpha.3.1, SHA-256
> `39e9e041…52a6ef3`, pinned in `reference/stable-26.721.3996.0/manifest.json`.
> Reconnect context (A §1 L82, J5 L328-333): deduplicated 1/2/4/8/16/20 s
> reconnect timer; retryable initial startup failure.
> ref+1 contrast only (`codex-linux/deb-metadata.txt:33-34`,
> `official-app-log-extracts.txt:2-14`): without `resources/codex` the official
> app fatals with "Unable to locate the Codex CLI binary or required runtime
> components" — no window. Never cite as 26.825 behavior.

**Flauz claim (matrix L163):** exact packaged-CLI hash check, override/fallback
order preserved `[historical-record] PM`; boots to entry surface with
app-server online footer `[runtime-observed] B2 ev/06`; graceful no-runtime
degradation ("Resolving…", auto-retry, ev/24).

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 01-A | `resolve_codex_binary` preserves the exact override/fallback order: explicit arg → `CODEX_RS_CODEX_BIN` → Windows stable-cache (sha256 vs `stable_reference().cli_sha256`) → WindowsApps packaged copy (hash-checked before copy AND re-verified after) → APPDATA npm candidate → bare PATH name | source-read | Read `crates/codex-platform/src/lib.rs:261-329` (verified anchor during rubric construction: order and both `sha256_matches` checks are present there at base) | FAIL if any step reordered/removed or a hash check dropped |
| 01-B | `stable_reference()` identity matches `reference/stable-26.721.3996.0/manifest.json` (package 26.721.3996.0, cli 0.146.0-alpha.3.1, same sha256) | source-read | Compare `codex_core::stable_reference()` fields vs the manifest JSON | FAIL on any drift |
| 01-C | Fresh isolated boot reaches the entry surface with sidebar footer "App-server online" (green dot) | GUI scene | Replicate ev/06 scene (b1-nav class); frame + VLM read | FAIL if footer missing/lying |
| 01-D | No-runtime degradation: `/bin/false` as resolved binary → app stays up, status "Resolving…", silent auto-retry, no crash | GUI scene | Replicate ev/24 scene (b3-error class) | FAIL on crash or user-hostile fatal |

**Attack notes (C):** the hash-pin path is `#[cfg(windows)]`-gated
(`codex-platform/src/lib.rs:285-324`) — on Linux the resolution is env/PATH
only. Probe any prose that extends "exact hash check" to Linux. The win cell
is `[historical-record]` + source; a Windows runtime re-check is a future
WINDOWS_GUI_LAB bound, not available now. Probe whether 01-D resilience gets
over-claimed as *parity* (it is an intentional-difference-grade advantage; the
official ref+1 fatal is out of target).
**Honest bounds:** none blocking; 01-C/01-D are executable in the current lab.

---

## RUB-02 — Side chats (matrix: complete; closed WO-P2-006, PR #18 → `5287c29f3f`)

**Reference version:** baseline ● binding Ctrl+Alt+S (B2R:442); official
timing traced to the 26.707 line with **medium-high** confidence (A §1 L84);
`/side` confirmed in the current 24-command docs set.

**Official behavior:**
> "Open side chat Ctrl/Cmd+Alt+S; temporary side conversation without
> interrupting the main chat (/side)" — A §1 L84, `[historical-record]` +
> `[docs-derived]`, confidence medium-high (26.707 "side conversations" note).
> Runtime-side support: rust-v0.146.0 notes "side conversations without
> closing" (`codex-ref/version-delta-notes.md:163-167`).

**Flauz claim (matrix L170):** Ctrl+Alt+S (Cmd+Alt+S macOS) opens the side
panel with the main chat still selected; side composer submits into the side
thread without selecting it; guarded `/side` (menu row) reopens; close returns
to the main view; main-chat selection and active turn untouched
`[source-derived ui.rs + codex-core lib.rs @764e4db; runtime-observed D9/D9b]`.

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 02-A | Registry carries the side-chat binding: `openSideChat` interceptor command in the Thread group, Ctrl+Alt+S/Cmd+Alt+S advertised, id present in `KEYBOARD_SHORTCUT_COMMAND_IDS` | source-read | Inspect `crates/codex-app/src/ui.rs` bind_keys + interceptor registry + `MAX_KEYBOARD_SHORTCUT_COMMANDS` (76 at base) | FAIL if binding/metadata drifted |
| 02-B | Side submit creates the side thread WITHOUT selecting it; main-chat selection + active turn untouched | unit-test | Re-run the WO-P2-006 state tests (221 core tests were green at closure) | FAIL on any selection leak |
| 02-C | Both entry paths work: Ctrl+Alt+S opens panel with main chat still selected; `/side` reopens it; side composer accepts text and submits (Ctrl+Return) | GUI scene | Re-run `docs/research/evidence/wo-p2-006/d9-wo-p2-006-side-chats-verify.sh` scene; compare frames 001-010 + `vlmd9a.json` verdicts | FAIL if any of the script's PASS criteria (a)-(e) break |
| 02-D | Close (panel-header close button) dismisses the panel and restores the main view full width | GUI scene | Re-run `d9b-close-probe.sh`; expected close button ≈ (1418,140) per `vlmd9b.json` | FAIL if close disturbs the main chat |

**Attack notes (C):** official-side evidence is historical+docs (medium-high
confidence) — B may verify FLAUZ behavior at runtime but must never label the
OFFICIAL behavior runtime-verified. Probe the guard on `/side`
(availability-guarded): does the guard's honest status fire when unavailable,
per the never-silent-no-op doctrine? Probe side-panel geometry claims (360 px
right dock) — no official pixel evidence exists; do not allow pixel-parity
claims.
**Honest bounds:** none blocking (D9/D9b scenes are unauthenticated-lab safe).

---

## RUB-03 — Keyboard shortcut reference (matrix: complete)

**Reference version:** baseline dialog (Ctrl+/ ●, B2R:438); the searchable
22-row overlay is **ref+1 (26.908.70816)** — naming corroboration only.

**Official behavior:**
> "Exact stable `Keyboard shortcuts` dialog listing only active bindings" —
> PM, `[historical-record]` (parity report L252).
> "Keyboard shortcuts dialog (Ctrl/Cmd+/) with search and platform-aware
> keycaps" — A §10 L222, `[historical-record]`.
> ref+1, naming only: overlay title "Keyboard shortcuts", close x, cyan
> "Search shortcuts" field (`codex-linux/vlm-reads.txt:54-55`); rows enumerated
> READ 7 (VLM:55-78) — note the 21-enumerated-vs-22-claimed discrepancy
> (Appendix B).

**Flauz claim (matrix L252):** opens from Help + Ctrl/Cmd+/; stable category
order; overlay + searchable editable settings page both visually confirmed
`[historical-record] PM; runtime-observed ev/11, ev/15]`.

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 03-A | Ctrl+/ opens the shortcuts overlay with categorized rows (Chat, Navigation, …) | GUI scene | Replicate ev/15 capture; frame + VLM read | FAIL if overlay absent/miscategorized |
| 03-B | `/settings/keyboard-shortcuts` page renders search + editable shortcut table | GUI scene | Replicate ev/11 capture | FAIL if page degraded |
| 03-C | "Listing only active bindings": every row shown in overlay/settings maps to a live dispatch arm (registry ↔ `ACTIVE_KEYBOARD_SHORTCUTS` set-equality; zero dead ids) — cross-checked against WO-R-SWEEP's finding of 16 advertised-while-guard-disabled rows | source-read | Re-derive the sweep's registry set-equality at the current SHA (`docs/research/evidence/wo-p2-007/input-surface-sweep.md` method); enumerate which guard-disabled commands appear as overlay rows | FAIL if any overlay row advertises a binding that cannot dispatch at all; honest-guard rows are a C-judgment zone, not auto-fail |
| 03-D | Shortcut registry membership + metadata for `keyboardShortcuts` (title, group, Ctrl+/ default) | unit-test | Run the keyboard-registry metadata tests (same family as WO-P2-004 coverage tests) | FAIL on metadata drift |

**Attack notes (C):** PM's "only active bindings" is the strongest untested
claim in this row — the WO-R-SWEEP already found 16 advertised-while-guard-
disabled bindings; C demands the overlay-vs-arms cross-check (03-C) before
defending `complete`. The 22-row ref+1 overlay must not become the bar; if B
cites it, require the version-skew label.
**Honest bounds:** none blocking.

---

## RUB-04 — Feedback (matrix: complete)

**Reference version:** baseline. `/feedback` in the baseline 15-command slash
set and the current 24-command set (A §2 L96); `feedback/upload` is one of the
89 `ClientRequest` methods (`codex-ref/cli-runtime-surface.txt:34`).

**Official behavior:**
> "Exact stable `Feedback` command + `/feedback`; five category IDs; required
> details; default-on session logs" — PM, `[historical-record]` (parity
> report L254). These deep facts are PM-owned; the A-matrix carries no
> Feedback feature row (rubric-construction finding).

**Flauz claim (matrix L254):** native `Share feedback` dialog with the five
recovered category IDs, validation, default-on logs, no browser-tabs control;
palette entry exists `[historical-record] PM; source-derived palette]`.

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 04-A | Exactly five feedback category IDs exist and match the PM-recovered set | source-read | Count `FeedbackClassification` variants at `crates/codex-core/src/lib.rs:4359` (usages show at least Bug/BadResult/Other at :32949-:32982); compare against PM's recovered IDs | FAIL on count ≠ 5 or ID drift |
| 04-B | Submit path enforces required details; `includeLogs` default-on; bounded details + threadId + app-version tags; typed `feedback/upload` request | source-read | Read `crates/codex-app/src/ui.rs` modal flow (WorkspaceModal::Feedback; classification guard ≈ :9836; `Action::FeedbackSubmitted` ≈ :7595; `ClearFeedbackError` ≈ :9758) and `crates/codex-platform/src/app_server.rs` (`request("feedback/upload", …)`) | FAIL if validation/logs-default/tags/bounded-payload claims unsupported |
| 04-C | Dialog opens from palette row "Feedback" (App group) and Help path; empty-submit shows the validation error; guard statuses honest | GUI scene | New scene: palette query "feedback" → open modal → attempt empty submit → capture frames + VLM read (first click-through ever — B2 only confirmed the palette entry exists, `FLAUZ-REFERENCE-MATRIX.md:288`) | FAIL if modal unreachable or validation silent |
| 04-D | `feedback/upload` wire shape: classification/includeLogs/reason/tags/threadId | unit-test | Run the protocol serialization test (`crates/codex-protocol/src/lib.rs` contains the `feedback/upload` JSON fixture) | FAIL on shape drift |

**Attack notes (C):** this is the weakest `complete` row — the deep dialog
claims are `[historical-record]`-only and B2 never clicked through
("done ✅ palette entry exists; click-through not exercised"). C invokes rule
3 (source presence ≠ completion) and demands 04-C before the status is
defensible. The upload round-trip needs a live runtime → expected `NOT RUN`
(no codex CLI in lab); C checks nobody claims it was run.
**Honest bounds:** 04-C executable unauthenticated; upload round-trip NOT RUN
expected.

---

## RUB-05 — Multi-root workspace handling (matrix: complete, regression control)

**Reference version:** baseline-era public failure report (KF).

**Official behavior:**
> "Windows multi-root white screen | POSIX path handling reached Windows
> drive paths and a missing process cwd | Native `Path`/`PathBuf`; no browser
> path shim | Implemented" — `docs/known-failures.md:8`, with the framing
> "These are observed failure modes from the compatibility reference and
> public reports. They are test inputs, not behavior to reproduce."
> (:3-4). The official multi-folder capability facts (26.715 marker) are in
> CL:890-899 / VDN:152-155 (used by RUB-07).

**Flauz claim (matrix L210):** native `Path`/`PathBuf`, no browser path shim —
Windows multi-root white screen controlled (acceptance test)
`[historical-record] KF]`.

**Criteria (inverted — the official artifact is a failure to NOT reproduce):**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 05-A | No browser-style path shim exists in workspace resolution (native `Path`/`PathBuf` end-to-end) | source-read | Sweep `crates/codex-core` workspace/path resolution for URL-encoded or JS-style path normalization; the control is the ABSENCE of a shim | FAIL if any browser path shim found |
| 05-B | Multi-root regression coverage exists and passes (the historical "acceptance test" successor: multi-folder model tests from WO-P1-003) | unit-test | Run the WO-P1-003 multi-folder test family (part of the 601 workspace tests green at closure; schema-v4 `workspace_folders` tests in `crates/codex-storage`) | FAIL on any red |
| 05-C | Platform honesty: the win cell stays labeled `[historical-record]`; no Linux-derived closure of the Windows row | source-read (process) | Check the parity row + any new evidence dirs for a Windows runtime claim | FAIL if Linux evidence is used to close the win cell |

**Attack notes (C):** the row's `complete (automatic)` cells rest on
architecture + historical acceptance; the Windows runtime re-check is a
WINDOWS_GUI_LAB future bound. C probes for status laundering (05-C) and for
silent erosion of the native-path discipline (05-A).
**Honest bounds:** Windows runtime NOT RUN expected (no lab).

---

## RUB-06 — Settings shell (matrix: partial)

**Reference version:** baseline 274 px shell + 26-section registry
(`[historical-record]` A §9 L195-196); Settings > Import = 2026-08-11 marker
(in target); the login-surface palette Settings group of 10 pages is **ref+1
(26.908.70816)** — corroboration only (`codex-linux/README.md:63-67`,
`vlm-reads.txt:17`).

**Official behavior:**
> "274 px shell: `Back to app`, search field (Ctrl/Cmd+F focused),
> Personal/Integrations/Coding/Archived groups, filtering, no-results state;
> Ctrl/Cmd+comma opens Settings (General)" — A §9 L195 `[historical-record]`.
> "Full stable registry (26 sections): agent, appearance, appshots,
> browser-use, chronicle, cloud-environments, cloud-settings, code-review,
> codex-micro, computer-use, connections, data-controls, debug, environments,
> general-settings, git-settings, hooks-settings, keyboard-shortcuts,
> local-environments, mcp-settings, personalization, pets, profile, usage,
> voice, worktrees (+ skills-settings registered but hidden)" — A §9 L196.
> "2026-08-11: Settings > Import" — changelog CL:571-587.

**Flauz claim (matrix L239):** stable-shaped shell; nav renders 15 rows in 3
groups; `SettingsSection` enum has 18 sections (CodeReview/Worktrees/
ArchivedChats contextual/hidden) `[source-derived ui.rs:2398-2417;
runtime-observed ev/07]`; settings search filters correctly ("import" →
Personal + Import) `[runtime-observed ev/23]`; remaining sections only with
working host contracts (ledger bounded).

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 06-A | `SettingsSection` enum cardinality = 18 with the named contextual/hidden members | source-read | Count variants at `crates/codex-app/src/ui.rs:2494` (enum anchor verified at base; matrix cited :2398-2417 pre-P2-008 drift) | FAIL on cardinality/membership drift vs the matrix claim |
| 06-B | Settings nav renders 15 rows in 3 groups; "Back to app" present; Ctrl+, opens Settings | GUI scene | Replicate ev/07 capture + VLM read | FAIL on row/group regression |
| 06-C | In-Settings search: query "import" filters nav to Personal + Import; no-results state exists | GUI scene | Replicate ev/23; add a no-match query capture | FAIL if filter regresses or no-results state missing |
| 06-D | Palette indexes every default-nav settings section (post-WO-P2-004): queries import/profile/browser/configuration/hooks/git resolve AND navigate to their pages | GUI scene + unit-test | Re-run the wo-p2-004 scene (frames 03/05/07/09/11/13 + pages 04/06/08/10/12/14) and the registry-coverage + filtering tests | FAIL on any No-matches regression (the ev/18 defect class) |
| 06-E | Deferral honesty: the missing sections (vs the official 26) remain labeled bounded-by-host-contracts, not silently closed | source-read (process) | Check the row's Gap cell + ledger bound wording unchanged | FAIL if the bound is dropped without a WO |

**Attack notes (C):** "274 px" is A-owned `[historical-record]`; the runtime
evidence tree carries no 274 px measurement (only the 275 px *sidebar*,
B2R:62-64) — no pixel-parity claim is defensible without a capture. The
ref+1 palette group (10 pages incl. Pets) must stay version-labeled. The
presentation residual ("single dynamic group for ALL pages" vs Flauz's fixed
grouping, wo-p2-004 README:70-74) is documented — C verifies it is not
recounted as an indexing gap.
**Honest bounds:** none blocking.

---

## RUB-07 — Projects and chats (matrix: partial; multi-folder backend CLOSED WO-P1-003, PR #15 → `d06ae3b`)

**Reference version:** baseline registry/grouping/search/archive/pin; the
multi-folder capability is a 26.715 marker (IN baseline, 26.715 < 26.721);
unified pinned threads + shared thread snapshots = 2026-08-20 marker, cloud
side → **documented deferral (proprietary)**.

**Official behavior:**
> "64-entry local-project registry; project rows: selection, new chat, rename,
> pin/unpin ordering, platform file-manager open, missing-folder warning,
> removal; `Add new project` via picker; manual `Move up`/`Move down`;
> projects render independently of chats | 26.715: multi-folder local projects
> — `Edit project` adds related folders and chooses the primary; new chats,
> Git, AGENTS.md/skills/config.toml discovery use the primary folder;
> secondary folders for file search/reading/editing" — A §6 L156.
> "Bounded paginated `thread/list` (`useStateDbOnly: true`), `thread/read`
> hydration, `thread/resume`; archived chats via `thread/archive`/`unarchive`,
> `thread/delete`; side conversations switchable without closing" — A §1 L71.
> "Bounded full-text chat search via stable `thread/search` snippets +
> pagination; unified command menu drill-in" — A §1 L80.
> 2026-08-20 shared snapshots/unified pins — CL:405-416 (cloud side).

**Flauz claim (matrix L165):** bounded search with stable snippets +
pagination, command menu, archive/delete/rename, codexRS-owned bounded pinning
`[historical-record] PM`; sidebar Chats list + Ctrl+G palette entry render
`[runtime-observed ev/06, ev/08]`; multi-folder model on main (WO-P1-003):
`LocalProjectSummary.folders` + Edit project surface + primary-swap re-key +
related folders in file search; single-path legacy projects load primary-only
`[source-derived core lib.rs:908, 4719; B2 §6]`.

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 07-A | Bounded thread search/pagination + archive/delete/rename + bounded pinning still green | unit-test | Run the codex-storage / codex-core conversation + registry test family (`crates/codex-storage/src/lib.rs` pinned/search/pagination tests) | FAIL on any red or unbounded query path |
| 07-B | Multi-folder model: `LocalProjectSummary.folders` (cap 16, primary never a member), primary-swap re-keys the registry row, related folders join fuzzy file search, cwd/Git/AGENTS.md/config stay primary-only, schema v4 round-trip, legacy single-path loads primary-only | source-read + unit-test | Read the model at `crates/codex-core/src/lib.rs` (matrix anchors :908, :4719); run the WO-P1-003 test family | FAIL on cap/primary/re-key regression |
| 07-C | End-to-end multi-folder flow visible: seeded project shows primary path indicator; file search hits a related folder; Edit project surface (Primary badge, Make-primary/Remove/Add folder/Done); primary swap follows; state persists | GUI scene | Re-run `docs/research/evidence/wo-p1-003/scene-d6-v5.sh` with `seed-wo-p1-003.py`; compare frames 01-06 + `vlm-reads.txt` | FAIL on any of the six frame assertions |
| 07-D | Sidebar Chats + Ctrl+G palette entry render; honest empty states ("No chats"/"No projects") | GUI scene | Replicate ev/06 + palette Ctrl+G capture | FAIL if surfaces/empty states regress |
| 07-E | Deferral honesty: unified pins / shared snapshots remain labeled proprietary-cloud deferred; richer metadata remains P3 residual | source-read (process) | Check row Gap cell + ledger | FAIL if deferral silently drops |

**Attack notes (C):** the cap of 16 folders is a Flauz bound with NO official
counterpart in the evidence tree — it must stay labeled a bound, never
"parity". Primary-swap re-key under duplicate/overlapping folders is the
sharpest unit-test corner (C requests a duplicate-folder case). Native
Add-folder picker is invisible under bare Xvfb (wo-p1-003 README:22-26 —
pre-seeding is the documented lab device); C accepts that bound but demands
the reducer paths stay unit-covered.
**Honest bounds:** 07-C needs the seed script (documented lab device).

---

## RUB-08 — Keyboard and accessibility (matrix: partial; palette-indexing slice CLOSED WO-P2-004; Ctrl+P integrity CLOSED WO-P2-007)

**Reference version:** baseline palette mechanics + editable registry +
grouping comparator + focus traps (A §10 L215/L222); post-baseline rows
(font-size, Clear terminal, Toggle file tree, etc.) are in-target with
**undated** introductions (`[unverified]`, VDN:283-286); the 22-row overlay
is ref+1.

**Official behavior:**
> "Ctrl/Cmd+K, Ctrl/Cmd+Shift+P, Ctrl/Cmd+G; arrow/Enter/pointer/Escape;
> Suggested order `New chat`, `Open folder`, workspace-aware `Search files`
> then dynamic `Settings` group; compact one-line density unfiltered,
> descriptions when filtered" — A §10 L215 `[historical-record]`.
> "Editable shortcut registry with stable grouping comparator (Chat →
> Navigation → Panels → Project → Skills → Configure → App → General);
> Keyboard shortcuts dialog (Ctrl/Cmd+/) with search and platform-aware
> keycaps; focus traps on destructive confirmations" — A §10 L222
> `[historical-record]`. NOTE: "complete focus order, screen-reader labels"
> in the parity row is PM wording; A evidences only the focus traps.

**Flauz claim (matrix L251):** native palette with verified stable registry
subset (51-command registry; every default-nav settings section indexed post
WO-P2-004); Ctrl+/ overlay + editable settings page `[runtime-observed
ev/11, ev/15]`; pending remaining stable commands, complete focus order,
screen-reader labels, reduced-motion.

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 08-A | Ctrl+P routes through the `searchFiles` interceptor arm (dead `OpenFileSearch` action stays removed); exactly one owner | unit-test | Run `ctrl_p_routes_to_the_search_files_command` (5 assertions, WO-P2-007) | FAIL on dead-action reintroduction or multi-ownership |
| 08-B | Workspace-seeded Ctrl+P opens the "Search files" palette; Escape closes byte-identically (md5 pair) | GUI scene | Re-run `docs/research/evidence/wo-p2-007/d10b/` scene; compare md5s against `ws-md5.txt` pattern (open differs, close identical) | FAIL if frames go byte-identical on Ctrl+P (the defect class) |
| 08-C | Grouping comparator order preserved (Chat → Navigation → Panels → Project → Skills → Configure → App → General) | source-read | Read the group comparator in `crates/codex-app/src/ui.rs` | FAIL on reorder |
| 08-D | Zero dead interceptor arms (registry ↔ `ACTIVE_KEYBOARD_SHORTCUTS` set-equality; 72/72 at sweep base `3c9f113`) | source-read | Re-derive the WO-R-SWEEP set-equality at the current SHA | FAIL on any dead id |
| 08-E | Open-gap honesty: focus order completeness, screen-reader labels, reduced-motion remain OPEN partial cells (not silently closed); F-A4 (Ctrl+P silent with no workspace, by-design guard) stays documented | source-read (process) | Check row wording + sweep residual F-A4 documentation | FAIL if an open gap is closed without evidence |

**Attack notes (C):** F-A4 survives the WO-P2-007 fix by design — C probes
whether "never a silent no-op" is violated on the bare entry surface and
whether the guard is honest (visible guidance vs silence). The sweep's 16
advertised-while-guard-disabled rows are the densest attack surface; C
samples them against the never-silent-no-op doctrine. Focus-trap evidence
covers destructive confirmations only — no per-surface traversal contract
exists (B2R:527-529 `[unverified]`).
**Honest bounds:** none blocking.

---

## RUB-09 — Notifications and tray (matrix: partial; no WO yet)

**Reference version:** baseline banner + tray-tooltip count + Windows
quiet-hours toast + Linux freedesktop best-effort (A §10 L219); the 26.825
scheduled-inbox delta is a separate capability; `libnotify4` in the 26.908 deb
depends is a ref+1 weak signal only.

**Official behavior:**
> "In-app completion banner with Open/Dismiss; title + tray tooltip count of
> running/awaiting-approval non-selected chats; Windows quiet-hours-respecting
> notification-area alert; Linux freedesktop best-effort (Linux app did not
> exist at baseline — this row was the codexRS Linux behavior)" — A §10 L219
> `[historical-record]` + `[docs-derived]`.
> "Chats keep running while unselected; background-terminal activity;
> completed background chats notify (banner + Windows quiet-hours toast /
> Linux freedesktop notification)" — A §1 L79.

**Flauz claim (matrix L253):** bounded in-app banner (Open/Dismiss) +
matching window-title/notification-area tooltip; gh-missing toast banners
observed on multiple pages `[runtime-observed ev/03/04]`; pending tray
groups, badges, sounds (ledger enhancement); Linux tray/global shortcuts
unavailable (packaging row).

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 09-A | Banner construction is bounded with Open/Dismiss actions; window-title/tooltip count path reports a bounded count of running/awaiting-approval non-selected chats | source-read | Read `crates/codex-platform/src/desktop_notifications.rs` + the title/tooltip update path + banner emission sites in `crates/codex-app/src/ui.rs` | FAIL if unbounded, actionless, or count-free |
| 09-B | Count-reporting logic is unit-covered and bounded | unit-test | Run the notification/count test family if present; if absent, record DEFECT-FOUND (claim without coverage) or open a follow-up | FAIL on red; absence = documented finding, not auto-fail of the row (row is partial) |
| 09-C | Observed banner class still renders: gh-missing toast banner with honest actions | GUI scene | Replicate ev/03/04 scene (PR page / Plugins page) | FAIL if the observed class regresses |
| 09-D | Status honesty: completion-banner/tooltip/quiet-hours claims stay `[historical-record]`-labeled; partial must not upgrade on toast-banner-only evidence | source-read (process) | Check row provenance labels | FAIL if provenance inflation occurs |

**Attack notes (C):** the weakest-evidenced row of the ten — the matrix's
Flauz cell mixes one `[runtime-observed]` fact (toast banner) with
`[historical-record]` claims (completion banner, tooltip count, quiet hours).
C demands the distinction stay visible (09-D) and probes whether the title
count actually updates on background completion (needs runtime overlay →
expected NOT RUN). Tray groups/badges/sounds are an enhancement gap (P2, no
WO) — deferred, not defect.
**Honest bounds:** completion-banner runtime observation NOT RUN expected
(no codex CLI in lab); Windows quiet-hours NOT RUN expected (no Windows lab).

---

## RUB-10 — Activity view & unread attention (matrix: partial; state+bindings CLOSED WO-P2-008, PR #23 → `00a3392`)

**Reference version:** 26.727 marker (2026-07-30 — post-baseline, IN the
26.825 target); no 26.908 change evidenced; the Pets bell is a different
ref+1 surface; official view shape auth-walled `[unverified]`.

**Official behavior:**
> "Added a new 'Activity view' in the sidebar to view which chats you engaged
> with recently and require attention. Click the bell or use Cmd/Ctrl+Opt+U
> to change to the new view." — changelog CL:786-795 (26.727).
> Bindings (current-docs table, post-baseline): "Clear all unread indicators
> — Shift+Esc; Next chat needing attention — Ctrl+Alt+A; Toggle Activity view
> — Ctrl+Alt+U" — B2R:76-80; adjacent "Mark chat unread Ctrl+Shift+U".
> Reference digest: membership = "recently-engaged + needs-attention"; entry
> points evidenced = bell + Ctrl/Cmd+Alt+U only; event classes/ordering/
> grouping/per-row actions `[unverified]`; visual unread treatment
> `[unverified]` (B2R:82-84).

**Flauz claim (matrix L260):** bounded `needs_attention_task_ids` (capped
`MAX_VISIBLE_THREADS`, not persisted) flagged on background turn completion
(failed turns included) + approval requests in non-selected chats; visit
clears, archive drops; sidebar 6px dot + medium-weight title; four registry
bindings (Ctrl+Shift+U round-trip, Ctrl+Alt+A cyclic jump, Shift+Escape
honest count, Ctrl+Alt+U honest guidance — view surface is a separate future
WO); 7 core state tests + 6 app binding tests.

**Criteria:**

| ID | Checkable criterion | Class | Cheapest honest step | Pass/fail |
| --- | --- | --- | --- | --- |
| 10-A | The 7 core state tests pass per-test (background completion marking incl. failed turns + selected/unknown exclusions; approval marking; visit-clears; manual toggle round-trip; archive drops; clear-all honest reporting; activity-view honest guidance) | unit-test | Re-run with per-test output (WO-P2-008 closure standard) | FAIL on any red |
| 10-B | The 6 app binding tests pass (exact-one-owner accelerators, registry membership + metadata, reducer resolution, jump semantics) | unit-test | CI both matrices or local guarded run (OOM discipline applies) | FAIL on any red |
| 10-C | All four bindings resolve visibly on the empty surface with the verbatim honest statuses: "Select a chat before marking it unread." / "No chats need attention." / "Activity view is not available yet. Use \"Next chat needing attention\" to jump to unread chats." / "No unread chats" | GUI scene | Re-run `docs/research/evidence/wo-p2-008/d11b-empty-surface.sh`; compare frames + md5s (`d11b-md5.txt`) + VLM verbatim reads | FAIL on any silent no-op or status drift |
| 10-D | Full flow (mark → create second chat → jump → clear) executes when a runtime-enabled lab exists; until then the bound stays documented | runtime overlay | Expected NOT RUN (no codex CLI in lab — same residual class as 007's F-A4); record the reason | N/A until lab upgrade |
| 10-E | Scope honesty: the Activity view SURFACE stays a separate future WO; the partial status must not be read as view-delivered; unread persistence across restarts stays labeled unverified (reference behavior itself unverified) | source-read (process) | Check row wording + ledger | FAIL if the surface is implied delivered |

**Attack notes (C):** the official visual treatment (dot shape/size, row
weighting) is `[unverified]` — Flauz's "6px dot + medium-weight title" is a
Flauz choice; no pixel-parity claim is defensible in either direction. The
cyclic next-unread jump semantics vs the official "next chat needing
attention" ordering is `[unverified]` on the official side — C forbids
over-claiming semantic parity. Session-only (unpersisted) unread state is a
documented design bound with the reference unverified — attack any wording
that calls it a defect or a match.
**Honest bounds:** 10-D NOT RUN expected.

---

## Appendix A — Version boundary map (per the repo's own reference evidence)

- **baseline 26.721.3996.0 (bar):** CLI hash pin + fallback order; palette
  mechanics + ● bindings (incl. Ctrl+Alt+S, Ctrl+/, Ctrl+P); editable shortcut
  system + grouping comparator + focus traps; 274 px settings shell +
  26-section registry; 64-entry project registry + grouping/search/archive/
  pin; multi-folder projects (arrived 26.715 — in baseline); notifications
  banner + tray tooltip count + quiet-hours toast; Feedback command +
  `/feedback` (PM deep facts); multi-root white-screen failure record (KF).
- **In-target markers (≤ 26.825.51511):** 26.727 Activity view (+ Ctrl+Alt+U),
  multi-repo review, address-bar history; 2026-08-11 Settings > Import +
  Linux preview; 2026-08-20 shared snapshots + unified pins (cloud side —
  documented deferral for Flauz); undated post-baseline shortcut rows
  (font-size, Clear terminal, Toggle file tree, Shift+Esc, Ctrl+Alt+A,
  Ctrl+Shift+U, …).
- **ref+1 (26.908+ — NEVER the bar):** 22-row searchable overlay; palette
  Settings group of 10 pages at login (incl. Pets); Pets quick chat + bell;
  Appshots on Windows; hard-fatal bootstrap without runtime; libnotify4 deb
  dependency; Sources-panel downloads; browser tab-width stability.

## Appendix B — Discrepancy & reconciliation register (rubric findings)

1. **Overlay row count 22 vs 21** — LXR:69 and B2R:468 claim 22 rows; VLM
   READ 7 enumerates 21 (4 Chat + 6 Navigation + 11 General). B must not
   treat either number as exact without a recount; the discrepancy is
   recorded, not adjudicated, here.
2. **274 px vs evidence tree** — the settings-shell width is A-owned
   `[historical-record]` (A §9 L195); no runtime capture in the evidence tree
   carries 274 px (the only pixel fact in-tree is the 275 px *sidebar*,
   B2R:62-64). Pixel parity is unverifiable in-lab; forbid pixel-parity claims.
3. **126 vs 89 ClientRequest methods** — the PM-era "126 experimental
   methods" figure was not reproduced (89 in both bundles, VDN:67-71);
   recorded as a measurement difference, not a contradiction.
4. **Side-chat official timing** — medium-high confidence (26.707 trace,
   A §1 L84). B/C must label official side-chat facts accordingly.
5. **cap-16 folders / 6px dot / 360 px side panel** — Flauz-side bounds and
   choices with no official counterpart in the evidence tree; they are honest
   divergences/choices, never parity claims.
6. **"complete focus order, screen-reader labels"** — PM wording; A evidences
   only focus traps on destructive confirmations (A §10 L222). The row's open
   a11y cells must not be closed on PM wording alone.
7. **PM-cited matrix wording vs A-matrix ownership** — e.g. "stable startup
   capabilities declaration, legacy-notification opt-out" (bootstrap row) and
   "five category IDs … default-on session logs" (Feedback row) are PM facts,
   not A facts; citation swaps would be misattribution.

## Appendix C — Evidence source map (what proves what)

| Source | Carries |
| --- | --- |
| `docs/research/CODEX-REFERENCE-MATRIX.md` (A, c083c38) | official-side rows, ● binding table, version markers, lab methodology |
| `docs/research/FLAUZ-REFERENCE-MATRIX.md` (B2, a2343d3) | Flauz-side claims + ev/NN map + audit verdicts |
| `docs/parity-matrix.md` (PM) | historical-record baseline facts (hash pin, Feedback deep facts, "only active bindings", focus-order wording) |
| `docs/known-failures.md` (KF) | multi-root white-screen failure record + control |
| `docs/research/evidence/codex-ref/*` | CLI runtime surface (89 methods, 37 flags, doctor), version deltas, changelog extract |
| `docs/research/evidence/codex-linux/*` | ref+1 runtime captures (login, palette, overlay, DB dialog, fatal-without-runtime) + VLM reads |
| `docs/research/evidence/p2-batch2-reference/README.md` | batch-2 official reference (Activity view R1, history R2, palette/a11y R3) + version doctrine |
| `docs/research/evidence/wo-p1-003|004|006|007|008/*` | per-WO closure frames + md5 + VLM reads + honest residuals |
| `docs/research/evidence/wo-p2-007/input-surface-sweep.md` | registry/palette/slash/settings integrity sweep (F-A1..F-D2) |
| `crates/**` (source-read anchors) | behavior anchors cited per criterion |

## Appendix D — Execution summary for the Lead

- B runs RUB-01..RUB-10 criteria cheapest-first, records VERIFIED /
  DEFECT-FOUND / NOT RUN + reason per criterion, with `file:line` or
  frame+md5+VLM evidence. Expected NOT RUN classes: 04 upload round-trip,
  05 Windows runtime, 09 completion-banner + quiet-hours, 10-D full flow
  (all documented lab bounds, not defects).
- C attacks along §0.4 + per-row attack notes; the highest-value targets are
  RUB-03-C (active-bindings cross-check), RUB-04-C (first Feedback
  click-through), RUB-09-D (provenance inflation), RUB-05-C (platform
  laundering).
- Findings feed follow-up WO candidates (see Appendix B and the residuals
  already recorded in the ledger).
