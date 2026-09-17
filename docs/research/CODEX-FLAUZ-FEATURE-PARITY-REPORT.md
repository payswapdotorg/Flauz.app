# Codex ↔ Flauz Feature Parity Report

> **STATUS: RECONCILED (post A/B audit) — pending Tech Lead convergence.**
> This document is the canonical parity report. The matrix in §5 was reconciled by
> Worker C2 (2026-09-16) from Worker A's `docs/research/CODEX-REFERENCE-MATRIX.md`
> (branch `parity/codex-reference-matrix`, commit `c083c38`), Worker B2's
> `docs/research/FLAUZ-REFERENCE-MATRIX.md` (branch `parity/flauz-reference-matrix`,
> commit `a2343d3`), and the historical repo records, following the reconciliation
> procedure in §7. Every cell carries its provenance label; overrides are recorded
> in §9. Statuses are reconciled — only C2 / the Tech Lead edit them from here.

**Repository:** `payswapdotorg/Flauz.app`
**Framework authority:** this file + `docs/research/FEATURE-PARITY-WORK-ORDERS.md`
**Work-order system:** `docs/research/FEATURE-PARITY-WORK-ORDERS.md`
**Version delta research:** `docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md`
**Deferred research (not under implementation):**
`docs/research/LOCAL-AGENT-INTEGRATION-RESEARCH.md`,
`docs/research/LOCAL-AGENT-ADAPTER-ARCHITECTURE.md`

---

## 1. Mission

Flauz.app ("codexRS" — native Rust/GPUI Codex Desktop reimplementation) must
reach **feature and UX parity with the official Codex GUI** (the desktop
product; not the CLI). This report is the single canonical matrix that the
parity wave reconciles into, and the input queue for the work-order system.
Pack/Workflow development is **frozen** during this phase; no matrix row may
be closed by Pack/Workflow work.

## 2. Reference layers

The comparison uses **two reference layers**, both retained:

| Layer | Identity | Role |
| --- | --- | --- |
| Historical baseline | **Codex Desktop `26.721.3996.0`** (Windows package `OpenAI.Codex_26.721.3996.0_x64__2p2nqsd0c76g0`, build `5828`, internal `26.721.31836`, owl/Chromium `150.0.7871.128`) + bundled **Codex CLI `0.146.0-alpha.3.1`** | Pinned behavioral specification captured in this repo's historical records (`docs/parity-matrix.md` §Reference baseline, `reference/stable-26.721.3996.0/manifest.json`). All historical evidence remains valid for this layer. |
| Current target | **ChatGPT desktop `26.825.51511`** (operator's installed reference) | The *current* parity target. Delta research vs the historical baseline is owned by Worker A → `docs/research/evidence/codex-ref/version-delta-notes.md`; see `docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md`. |

Method rule: **current parity target = historical baseline + current installed
delta.** Historical evidence is never discarded; where `26.825.51511` changed
behavior, C2 annotates the affected rows rather than deleting baseline rows.

## 3. Lab bounds (platform truth)

Per `docs/research/E2B-PARITY-ENVIRONMENT.md` (branch `parity/lab`, PR #9 —
pending wave convergence):

```text
WINDOWS_GUI_LAB: unavailable
MACOS_GUI_LAB:   unavailable
LINUX_GUI_LAB:   available (LOCAL Debian 13 sandbox — proven end-to-end)
```

Consequences for this matrix:

- The official Codex GUI is a closed-source Electron app shipped for
  Windows/macOS at baseline; it does not run in this lab. Official-side
  (Linux-A) evidence therefore consists of reference **layers** with
  provenance labels, not live GUI runs — except official *runtime* behavior
  that is reproducible in this lab (official CLI `0.146.0-alpha.3.1`,
  `codexrs probe`/`info`).
- Windows-pair and macOS-pair validation is **deferred** until those
  environments exist. Windows/macOS cells in this matrix may only carry
  `[historical-record]`, `[docs-derived]`, or `[source-derived]` evidence.
- No silent platform substitution: a Linux observation never fills a Windows
  cell, and vice versa (see §7.4).
- **2026-09-16 annotation (Worker A finding, `[docs-derived]`):** an OFFICIAL
  Linux desktop app now exists in preview (since 2026-08-11; `.deb`/`.rpm`;
  Ubuntu 24.04/26.04, Debian 13, Fedora 43/44; x64 + ARM64). The lab bounds
  above are **unchanged** — upgrading Linux-A from evidence-layers to
  runtime-observed requires running that app in LINUX_GUI_LAB first, tracked
  as WO-LAB-001 (see §8.4). Platform bounds are never changed silently.

## 4. Definitions (operator-specified — do not reword)

### 4.1 Gap taxonomy

| Gap type | Meaning |
| --- | --- |
| **Backend gap** | Functionality does not exist (no backend/protocol/engine support). |
| **UI-integration gap** | Functionality exists but cannot be naturally discovered or used in the UI. |
| **Platform gap** | Constrained by OS/runtime (not implementable or only partially implementable on a given platform). |
| **UX gap** | Functionality exists but behaves materially differently from the reference. |
| **Intentional difference** | Explicitly approved and documented divergence (e.g., unsigned portable releases per GUI-006; native GPUI instead of Electron). |

### 4.2 Status vocabulary

| Status | Meaning |
| --- | --- |
| `complete` | Usable end to end, covered by the relevant release gates, and compared with the reference at the same state/viewport. |
| `partial` | A real vertical slice exists, but the reference exposes more. |
| `missing` | No usable end-to-end implementation. |
| `platform-limited` | Implemented/valid on only part of the platform matrix (the Platform column must name the bound). |
| `deferred` | Consciously not being implemented now (e.g., blocked on a proprietary cloud backend or a pending public protocol, or postponed by operator directive). |

### 4.3 Provenance labels

| Label | Meaning |
| --- | --- |
| `[runtime-observed]` | Behavior observed by running the artifact in this laboratory (LINUX_GUI_LAB, official CLI runtime, `codexrs probe`). |
| `[source-derived]` | Derived from reading source code (`crates/**`, official bundle inspection records). |
| `[docs-derived]` | Derived from public documentation/release notes (including 26.721→26.825 release notes and upstream `openai/codex` release evolution). |
| `[historical-record]` | In-repo captures from the 26.721.3996.0 era: `docs/parity-matrix.md`, `docs/known-failures.md`, `docs/codex-universal/**`, `reference/stable-26.721.3996.0/manifest.json`, CHANGELOG. |

### 4.4 Priority classes

| Class | Meaning |
| --- | --- |
| **P0** | Blocks normal usage. |
| **P1** | Major feature missing — or effectively undiscoverable. |
| **P2** | Meaningful UX-functionality difference. |
| **P3** | Cosmetic. |

### 4.5 Column semantics (canonical matrix)

| Column | Semantics |
| --- | --- |
| `Capability` | One capability per row, named after the reference behavior, grouped under the fixed feature sections (§5). |
| `Official Codex` | Reference behavior (from Worker A's matrix / historical record), with provenance label. |
| `Flauz` | Current behavior (from Worker B2's matrix / historical record), with provenance label. |
| `Platform` | Platform slices this row's evidence covers (`win`, `linux`, `macos`). Split rows per platform when statuses differ — never merge silently. |
| `Backend` | Status of the backend/protocol/engine layer. |
| `UI` | Status of the UI surface layer. |
| `Discoverability` | Status of natural discoverability (persistent affordances, menus, labels — not just shortcuts/palette). |
| `UX parity` | Status vs the "behaves materially differently" bar. |
| `Functional parity` | Overall end-to-end functional status (the roll-up cell). |
| `Recovery` | Recovery/verification path: how a user or the lab recovers or re-verifies this capability (reconnect timers, retry, fallback, lab scene). `—` if none recorded. |
| `Evidence` | Provenance labels + source citations: A-matrix section, B2-matrix section, historical record, evidence captures. |
| `Gap` | Gap type (§4.1) + priority class (§4.4), or `none recorded`. Work-order ID once one exists. |

Cell conventions:

- `—` = not derivable from any evidence layer (named bound).
- A cell status is one of §4.2 only (plus `—`).
- Historical-status mapping applied at pre-population and revisited by C2:
  `done → complete`, `partial → partial`, `missing → missing`,
  `platform → platform-limited`, ledger verdict `bounded` (proprietary
  backend / pending public protocol) → status `deferred`; ledger delta class
  `polish → P3`, `enhancement → P2`. C2 overrides where the A/B audit
  produced higher-order evidence (§9).

---

## 5. Canonical parity matrix (RECONCILED post A/B audit)

Citations: **PM** = `docs/parity-matrix.md`; **KF** = `docs/known-failures.md`;
**A §n** = `CODEX-REFERENCE-MATRIX.md` §n (Worker A, commit `c083c38`);
**B2 §n** = `FLAUZ-REFERENCE-MATRIX.md` §n (Worker B2, commit `a2343d3`);
**ev/NN** = `docs/research/evidence/flauz/NN-*.png` (B2 curated runtime
captures). Rows carried from the C1 skeleton keep their historical-layer
content, now merged with A/B evidence; rows marked **(added by C2 audit)**
were surfaced by the A/B matrices with no canonical predecessor (§7.2
A-row-without-B-counterpart rule; Flauz-side absence verified by source sweep
at `main f113515`). Linux cells are `[runtime-observed]` where B2 evidenced
them; Windows/macOS official behavior is `[historical-record]`/`[docs-derived]`
only — no platform validation is claimed.

### 5.1 Agent / task lifecycle

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Runtime bootstrap | Packaged-CLI hash pin + explicit override/fallback order `[historical-record] PM` | Exact packaged-CLI hash check, override/fallback order preserved `[historical-record] PM`; boots to entry surface with app-server online footer `[runtime-observed] B2 ev/06` | win `[historical-record]`; linux `[runtime-observed]` | complete | complete | complete (automatic) | complete | complete | fallback order on CLI mismatch | PM "Runtime bootstrap"; B2 §1 | none recorded |
| App-server supervision | Stable startup capabilities declaration, legacy-notification opt-out, deduplicated 1/2/4/8/16/20 s reconnect, `thread/loaded/list` rehydration incl. active-turn state `[historical-record] A §1` | Bounded `thread/loaded/list` hydration via `thread/read`, resumed after connect; same reconnect timer; retryable startup failure; pending remaining stable methods `[historical-record] PM`; /bin/false runtime → app stays up, status "Resolving…", silent auto-retry, no crash `[runtime-observed] B2 ev/24` | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (persistent footer status + Retry action, ev/06/24) | partial | partial | reconnect timer + manual retry; resume after reconnect | PM "App-server supervision"; A §1 Reconnect; B2 §1 Retry/Reconnect | backend — P2 (remaining stable methods; no WO — bounded by public protocol surface) |
| Projects and chats | Grouping, bounded full-text chat search, archive/unarchive/delete, rename, pinning `[historical-record] A §1`; **multi-folder local projects in-baseline (26.715)** — `Edit project` adds related folders + primary choice; new chats, Git, AGENTS.md/skills/config.toml discovery use the primary folder; secondary folders for file search/read/edit `[docs-derived] A §6`; unified pinned threads + shared thread snapshots (2026-08-20) are cloud-side `[docs-derived] A §9` | Bounded search with stable snippets + pagination, command menu, archive/delete/rename, codexRS-owned bounded pinning `[historical-record] PM`; sidebar Chats list + Ctrl+G palette entry render `[runtime-observed] ev/06, ev/08`; multi-folder model IMPLEMENTED (WO-P1-003, PR #15): `LocalProjectSummary.folders` + Edit project surface + primary-swap re-key + related folders in file search; single-path legacy projects load primary-only `[source-derived] core lib.rs:908, 4719; B2 §6` | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (persistent sidebar affordances) | partial | partial | — | PM "Projects and chats"; A §1/§6; B2 §1/§6, ev/06/08 | backend — P1 multi-folder local projects **IMPLEMENTED (WO-P1-003, PR #15: 1d2abca + 2ea5970, 2026-09-17)**: `LocalProjectSummary.folders` (cap 16), Edit project surface with primary swap, related folders in file search, storage schema v4; runtime evidence docs/research/evidence/wo-p1-003/; closure row-flip on merge; cloud sync deltas (unified pins, shared snapshots) deferred (proprietary); P3 residual (richer metadata) |
| Thread execution | Start, steer, interrupt (`turn/interrupt`), reconnect, active-turn restoration, edit latest message (`thread/rollback` + replacement `turn/start`), manual compaction (`thread/compact/start`), safety-buffering faster-model retry `[historical-record] A §1` | Same typed stable methods incl. rollback + replacement start; compaction guards ("Compact requires an empty composer", "disabled while a chat is in progress"); Stop/Steer/Send button states wired `[source-derived] ui.rs:23363-23396, core lib.rs:12739/12753; `[historical-record] PM`; live states bound unauthenticated | win `[historical-record]`; linux `[source-derived]` | partial | partial | partial (Edit hidden in-turn; `/compact` slash-only; Stop/Steer contextual in composer) | partial | partial | selected active-turn restoration; reconnect | PM "Thread execution"; A §1; B2 §1/§2 | ux — P3 (compaction provenance, richer edit metadata) |
| Approvals and user input | Stable public `item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `item/permissions/requestApproval`; tool user-input cards with countdown/snooze `[historical-record] A §1/§8` | Native command/file/permission/dynamic-tool approvals with bounded typed payloads `[historical-record] PM`; approval dropdown beside composer "+" affordance `[runtime-observed] ev/01`; pending connector-specific methods (ledger bounded) | win `[historical-record]`; linux `[runtime-observed shell]` | partial | partial | partial (contextual cards; composer dropdown persistent) | partial | partial | — | PM "Approvals and user input"; A §8; B2 §2 ev/01 | backend — deferred (connector-specific public contracts) |
| Streaming timeline | Compact `Explored`/`Ran`, file-change, web-search, subagent, image-inspection/generation, background-terminal summaries; bottom-aligned variable-height list `[historical-record] A §1]` | Same stable-style summaries in native list `[historical-record] PM`; live streaming bound unauthenticated (no turns possible) `[runtime-observed bound] B2 §1` | win `[historical-record]`; linux `[historical-record]` (runtime-bound) | partial | partial | complete (timeline is the primary chat surface) | partial | partial | — | PM "Streaming timeline"; A §1; B2 §1 | ux — P3 (activity grouping, citation navigation, source aggregation) |
| Scheduled tasks | `/automations` route + Scheduled: suggestions, manual/chat-assisted creation, schedule editor, run history, unread/archive states, pause/edit/delete, notifications — cloud-backed `[historical-record] A §10`; **26.825: event-triggered tasks (Gmail/Slack/GitHub events, filters, `Run now`, Scheduled inbox)** `[docs-derived]` | Not implemented — cloud tasks backend proprietary; no surface in UI or source beyond plugin includes `[historical-record] PM; source-derived absence B2 audit]` | win `[historical-record]`; linux `[source-derived absence]` | deferred | deferred | — (no surface) | — | deferred | — | PM "Scheduled tasks"; A §10; B2 corrected-count audit | backend (proprietary) — deferred (event triggers same bound) |
| Side chats **(added by C2 audit)** | `Open side chat` Ctrl/Cmd+Alt+S; temporary side conversation without interrupting the main chat; `/side` in current command set `[historical-record + docs-derived] A §1 (confidence medium-high — 26.707 "side conversations" note + current docs)` | No side-chat surface: no Ctrl+Alt+S binding in the key registry, no `/side` in the slash executor `[source-derived absence: ui.rs:4459-4552 key registry; ui.rs:7510-7621 slash executor; sweep at f113515]` | win `[historical-record]`; linux `[source-derived absence]` | missing | missing | missing | — | missing | — | A §1 Side chats; B2 shortcut inventory (no Ctrl+Alt+S); C2 source sweep | backend — P2 → **WO-P2-006** |

### 5.2 Composer

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Composer (input surface) | Send/Steer/Stop states; Plan (`/plan`), Goal (`/goal` + file-backed attachments); attachments Add photos (PNG/JPEG/GIF/WebP) + file/folder; baseline slash set of 15 `[historical-record] A §2`; **current target lists 24 slash commands** (`/approve /cloud /cloud-environment /compact /fast /feedback /fork /goal /ide-context /init /local /mcp /memories /model /pet /personality /plan /project /reasoning /review /side /status /task /worktree`; skills via `$`; custom prompts via `/prompts:`) `[docs-derived] A §2`; `@` mention order Plugins → Desktop apps → Apps → Skills → Files `[historical-record]` | All states work; Plan via `/plan` + composer popup item; Goal editor via `/goal`; "+" attachment menu (Image/File/Scan/Paste from clipboard/Take screenshot) `[runtime-observed ev/16, ev/20]`; native named slash set of 16 (baseline 15 + `/review`) plus dynamic `/service-tier:<id>` and `/skill:<path>` rows `[source-derived ui.rs:7510-7621, 22892-23120 — C2 re-verified, overrides B2's 15-item enumeration, §9]`; `@` typed trigger (no popup unauthenticated/without project) `[runtime-observed b2g-11]`; `Continue in` Work picker on Git chats `[historical-record] PM` | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | partial (`/goal` slash-only; `@` hidden typed trigger; `/plan` also in composer menu) | partial | partial | failed submission restores draft without overwriting newer drafts `[historical-record]` | PM "Composer"; A §2; B2 §2 ev/01/16/19/20/21 | ux — P2 (current-target slash coverage → **WO-P2-005**); P3 residual (attachment surface copy deltas) |
| Model, effort, and speed picker | Catalog-backed searchable model picker (Ctrl/Cmd+Shift+M), reasoning effort, Standard/Fast/Ultrafast tiers `[historical-record] A §2`; model churn post-baseline: GPT-5.4/5.4-mini retired 2026-08-31, GPT-6-Astra added, GPT-5.5 retires 2026-10-14 → GPT-5.6 Sol `[docs-derived] A §2` | Implemented with `config/read` + managed defaults, profile-aware `config/batchWrite`; service-tier slash rows incl. exact Fast on/off copy `[historical-record] PM`; composer-bar selectors render live ("GPT-6-Astra", "Low", "Standard") `[runtime-observed ev/01, ev/19]` — catalog already carries GPT-6-Astra | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (persistent composer-bar affordances + Ctrl+Shift+M) | partial | partial | — | PM "Model, effort, and speed picker"; A §2; B2 §2 ev/01/19 | none recorded (P3 residual: model churn is catalog-data-side, picker is data-driven) |
| Permission profiles | Stable Ask for approval / Auto-review / Full access / Read only presets + granular/custom editor + sandbox detail + per-project resolution (present in stable, not yet reproduced) `[historical-record] A §9` | Catalog-backed built-in/custom selection matching presets with catalog descriptions in native picker; approval dropdown beside composer "+" `[runtime-observed ev/01]`; pending granular/custom editor, sandbox detail, per-project resolution (ledger enhancement) `[historical-record] PM` | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (persistent composer dropdown) | partial | partial | — | PM "Permission profiles"; A §9; B2 §2 ev/01 | ux — P2 (granular editor, sandbox detail, per-project resolution; no WO yet — needs scoping) |
| Voice input | Dictation, realtime voice, handoff target, stage layout, settings, accessibility states `[historical-record] PM`; Voice shipped 26.715 (GPT-Live) `[docs-derived] A §5` | Not implemented — proprietary realtime stack `[historical-record] PM; source-derived absence B2 audit]` | win, macos `[historical-record]` | deferred | deferred | — | — | deferred | — | PM "Voice"; A §5/§10; B2 audit | backend (proprietary) — deferred |

### 5.3 Terminal

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Terminal (sessions) | Persistent per-chat terminal dock (bottom or right); `Toggle bottom panel` Ctrl/Cmd+J distinct from `Open terminal`; hiding preserves live sessions `[historical-record] A §3`; current docs add `Toggle terminal` Ctrl/Cmd+`` ` `` + `Clear terminal` Ctrl+L/Ctrl+K when focused (post-baseline; intro date `[unverified]`) `[docs-derived] A §3`; **Linux preview runtime (WO-LAB-001): palette exposes "Panels: Open terminal (Ctrl+`)" even at the login surface; the persistent dock itself is auth-walled** `[runtime-observed: linux-preview 26.908.70816; evidence docs/research/evidence/codex-linux/`02]` | Bounded per-chat PTY/ConPTY tabs in bottom/right dock; new-tab, selection, close-one, stop, restart, bounded I/O, truncation `[historical-record] PM`; `portable_pty` backend `[source-derived] terminal.rs`; ctrl-` binding ui.rs:4499, `Action::ToggleTerminalDock` core lib.rs:4923, palette "Open terminal" (Ctrl+`) ui.rs:3322/3411 `[source-derived]`; **Ctrl+` on the new-chat entry surface is a silent no-op — post-toggle frame byte-identical (md5) to pre-toggle; dock opens with zero tabs and renders nothing; guard "Select a task before opening a terminal." core lib.rs:8163** `[runtime-observed] ev/25 vs ev/19` *(pre-wave evidence; the silent no-op is FIXED by WO-P1-001 — merged d15333e 2026-09-17 — the guard now surfaces visibly with a Dismiss control, ev wo-p1/11)* | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (WO-P1-001 CLOSED — merged d15333e 2026-09-17): persistent sidebar affordance renders in the default Tasks layout (ev wo-p1/01, VLM-read "Terminal" row above Settings); Ctrl+`/palette/dock retained; entry-surface toggle surfaces the guard visibly (ev wo-p1/11, VLM-read "Select a task before opening a terminal." + Dismiss) — GUI evidence docs/research/evidence/wo-p1/ | partial | partial | stop/restart per session | PM "Terminal"; A §3; B2 §3 + Finding 1, ev/25/19; C2 anchor re-verification | ui-integration — P1 → **WO-P1-001 CLOSED (merged d15333e718a7649238f33a183af27c885dbaa8aa via PR #13; unit+runtime evidence; CI green windows-latest + ubuntu-24.04; GUI evidence docs/research/evidence/wo-p1/)** |
| Process Manager | Ctrl+Alt+M / command palette / completed background-terminal activity open the stable-shaped manager: paginated `thread/backgroundTerminals/list`, per-process `terminate`, chat-wide clean, 1 s refresh `[historical-record] A §3]` | Palette command + Ctrl+Alt+M (ui.rs:3310) `[source-derived]`; completed background-terminal activity opens manager `[historical-record] PM`; bounded list/terminate/stop `[historical-record] PM`; live flows bound unauthenticated | win `[historical-record]`; linux `[source-derived]` | partial | partial | complete (matches the official affordance set: shortcut + palette + activity trigger) | partial | partial | — | PM "Process Manager"; A §3; B2 §1 | none recorded |

### 5.4 Browser

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| In-app browser | Native browser panel with bounded tabs, URL/navigation state, JPEG frame streaming, keyboard/pointer/scroll input vs supervised isolated Edge/Chrome/Chromium profile; `Open browser tab` Ctrl/Cmd+T, `Toggle browser panel` Ctrl/Cmd+Shift+B (baseline-verified bindings) `[historical-record] A §4`; **26.727: address bar revisits browsing history or searches Google on no match; browsing-history management in Settings; agent can search history** `[docs-derived] A §4]`**Linux preview runtime (WO-LAB-001): shortcuts overlay lists "Reload Browser Page" Ctrl+R and "Force Reload Browser Page" Ctrl+Shift+R** `[runtime-observed: linux-preview 26.908.70816; evidence docs/research/evidence/codex-linux/`06]` | Native GPUI panel: `BrowserSession::spawn`/navigate/back/forward/reload/stop, tabs, downloads store (`browser_downloads` table) `[source-derived] browser.rs, B2 §4; runtime-observed DB schema]`; panel is `InspectorPane::Browser` core lib.rs:259-267 `[source-derived]`; **Ctrl+T / Ctrl+Shift+B on the entry surface are silent no-ops; `OpenBrowserTab` refused without an open chat — "Open a chat before opening the Browser." core lib.rs:14521** *(pre-wave evidence; FIXED by WO-P1-002 — merged d15333e 2026-09-17 — toggles now surface the guard visibly, ev wo-p1/12)* `[runtime-observed] b2g-06; source-derived]` | win `[historical-record]`; linux `[runtime-observed no-op]` | partial | partial | complete (WO-P1-002 CLOSED — merged d15333e 2026-09-17): persistent sidebar affordance renders in the default Tasks layout (ev wo-p1/01, VLM-read "Browser" row above Settings); entry-surface toggles surface the guard visibly (ev wo-p1/12) — GUI evidence docs/research/evidence/wo-p1/ | partial | partial | — | PM "In-app browser"; A §4; B2 §4 + Finding 2; C2 anchor re-verification | ui-integration — P1 → **WO-P1-002 CLOSED (merged d15333e718a7649238f33a183af27c885dbaa8aa via PR #13; unit+runtime evidence; CI green windows-latest + ubuntu-24.04; GUI evidence docs/research/evidence/wo-p1/)**; ux — P2 (address-bar history/Google fallback, 26.727; no WO yet — needs scoping) |
| Browser permissions & settings | General Approval + site-specific Browse/Download/Upload/Debug rules + Full CDP switch; origin/download/upload/raw-CDP elicitation cards; `Allow once`/`Allow for this site`/`Allow for all sites`; persistent vs session grants `[historical-record] A §4]` | Stable-shaped Browser settings incl. bounded site rules + Full CDP switch, single-writer storage, hot-apply; typed MCP elicitations intercepted; `AllowAllBrowserSites` modal `[source-derived] ui.rs:4542-4550; `[historical-record] PM`; live cards bound unauthenticated | win `[historical-record]`; linux `[source-derived]` | partial | partial | partial (Settings page persistent; cards contextual) | partial | partial | existing Block/Allow rules + global modes resolve later prompts without UI | PM §"In-app Browser permission status"; A §4; B2 §4 | none recorded |
| WebMCP site tools **(added by C2 audit)** | **26.825: WebMCP "site tools" available in the desktop app's built-in browser for ChatGPT Work and Codex (GPT-5.6 Sol/Terra; not Luna; not Enterprise/Edu)** — was internal/Public-Beta gated at baseline `[docs-derived] A §4; changelog 2026-08-25]` | No WebMCP surface; browser panel streams frames/session only `[source-derived absence: browser.rs; sweep at f113515]` | win, macos `[docs-derived]`; linux `[source-derived absence]` | missing | missing | missing | — | missing | — | A §4 Browser state/capabilities; C2 source sweep | backend — P2 (no WO — blocked on fork-runtime WebMCP capability; future work order when runtime supports it) |
| Browser extension (adjacent surface) **(added by C2 audit)** | Chrome extension at baseline (tab mention/side chat); **26.825: Edge, Brave, Opera, Vivaldi (Opera without side chat), configured in Settings > Computer Use; right-click "Ask ChatGPT"** `[docs-derived] A §4]` | No extension surface (extension is a browser-store artifact pairing with the app; none shipped) `[source-derived absence]` | win, macos, linux `[docs-derived]` | missing | missing | missing | — | missing | — | A §4 Browser extension; C2 source sweep | backend (adjacent deliverable) — P2 (out of parity-wave scope; distribution surface beyond the app binary) |

### 5.5 Computer use

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Computer Use — Windows | Any App control: case-preserved AUMIDs, known-folder GUID IDs, absolute executable paths; ask-before-first-read/control; product-policy guard; Window2 action surface; screenshot bounds; URL policy; Escape interruption; overlay indicator `[historical-record] A §5]` | Native Any App with oversized-ID/shared-host rejection, self-control exclusion, ask-before-every-app's-first-read/control, managed approvals (ledger green Windows) `[historical-record] PM]` | win `[historical-record]` | partial | partial | partial (Settings page + `@` Desktop apps + inspector) | partial | partial | — | PM "Computer Use"; A §5 | none recorded (Windows; WINDOWS_GUI_LAB unavailable — unvalidated at runtime by bounds, not by absence) |
| Computer Use — Linux | No official Linux app at baseline; official Linux preview app exists since 2026-08-11 but its Computer Use scope is `[unverified]` `[docs-derived] A §5 platform table]` | Screenshot-only, bounded X11/XWayland observation when `DISPLAY` is set; pure Wayland, text extraction, input, app launch, persistent approvals, overlays, interruption monitoring unavailable; portal-backed selection path = future work; in-app copy is platform-honest ("Let ChatGPT observe screenshots of X11/Wayland apps — Enabled") `[source-derived] computer_use.rs:34-57; runtime-observed] ev/14` | linux `[runtime-observed]` | platform-limited | platform-limited | partial (Settings page persistent + palette "Open Computer Use") | platform-limited (by design, honestly labeled) | platform-limited | future: portal-backed selection path | KF "Active release-critical limitations"; PM ledger; B2 §5 + Finding 3, ev/14 | platform gap (not a missing feature) → **WO-PLAT-001** |
| Dynamic Computer Use tools | Tools available across a task's lifetime `[historical-record]` | Attached at `thread/start` only; an existing task cannot gain them after creation `[historical-record] KF "Active release-candidate limitations"]` | win, linux `[historical-record]` | partial | partial | — | difference recorded | partial | — | KF limitations | ux — P2 (no WO yet — needs runtime contract analysis) |

### 5.6 Files / workspace

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Artifacts and outputs | Outputs inspector: completed file changes, `::output` citations, generated images; Markdown/CSV/TSV + PNG/JPEG/WebP previews; 40 MiB guard; Reveal; end-resource cards; 680 px viewer; `Download` with Save As suggestion `[historical-record] A §6`; **26.727: generated-image editing (Focused/Canvas views, comments, targeted edits)** `[docs-derived]`; 26.908 Sources-panel open/download = reference+1 (out of target) | Bounded native Outputs inspector: latest-first dedupe, directives hidden, exact empty state `[historical-record] PM`; Outputs pane renders via `InspectorPane::Outputs` `[source-derived]`; pending PDF/Office renderers, AVIF/GIF, canvas actions (ledger enhancement); no image-editing surface `[historical-record] PM + source-derived absence]` | win `[historical-record]`; linux `[source-derived]` | partial | partial | partial (inspector pane; chat-scoped) | partial | partial | — | PM "Artifacts and files"; A §6; B2 §6 | ux — P2 (renderers, canvas actions; + generated-image editing 26.727; no WO yet — needs scoping) |
| Multi-root workspace handling | No multi-root white screen (public failure report documents the official failure) `[historical-record] KF]` | Native `Path`/`PathBuf`, no browser path shim — Windows multi-root white screen controlled (acceptance test) `[historical-record] KF]` | win `[historical-record]` | complete | complete | complete (automatic) | complete | complete | — | KF failure table | none (regression control) |
| In-app Markdown/code editing **(added by C2 audit)** | **26.707 (in-baseline): edit Markdown and code directly in the app, inline annotations, ask Codex to revise selected content** `[docs-derived] A §6 (changelog 2026-07-09)]` | Preview/Outputs surfaces only; no in-app editing or inline-annotation surface `[source-derived absence: Files/Outputs inspectors are render-only; sweep at f113515]` | win, macos `[docs-derived]`; linux `[source-derived absence]` | missing | missing | missing | — | missing | — | A §6 Markdown/code editing; B2 §6 (preview only); C2 source sweep | backend — P2 (no WO yet — editor surface needs Tech Lead scoping before a bounded order can be written) |
| Sites | Create, preview, annotate, version, share, publish, return-to-chat; custom domains (26.707); co-editing + editable URLs (2026-08-20) — OpenAI cloud surface `[historical-record] PM; docs-derived A §6]` | Not implemented `[historical-record] PM; source-derived absence B2 audit]` | win `[historical-record]` | deferred | deferred | — | — | deferred | — | PM "Sites"; A §6 | backend (proprietary) — deferred |
| Visualizations | Native chart/report rendering, source inspection, interaction, export, editor handoff — OpenAI cloud surface `[historical-record] PM]` | Not implemented `[historical-record] PM; source-derived absence B2 audit]` | win `[historical-record]` | deferred | deferred | — | — | deferred | — | PM "Visualizations" | backend (proprietary) — deferred |

### 5.7 Git

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Repository status | Current branch, ahead/behind, bounded changed files, local branches, worktrees, ≤30 unique commits; wide `Uncommitted changes` surface; repository picker still open official-side `[historical-record] A §7`; **26.727: multi-repository review across a multi-folder project's repos** `[docs-derived] A §7]` | Native repository snapshot incl. stable-shaped wide `Uncommitted changes`; Repository page renders stats row (Changes/Staged/Branches/Worktrees) with honest "No Git repository detected." empty state `[historical-record] PM; runtime-observed ev/02]`; multi-repo review not possible (single-root projects) `[source-derived]` | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (dedicated persistent sidebar page) | partial | partial | — | PM "Repository status"; A §7; B2 §7 ev/02 | ux — P2 (multi-repository review, 26.727; unblocked once WO-P1-003 merges (PR #15); repository picker P3 residual) |
| Diff review | Changes inspector sources Last Turn/Uncommitted/Unstaged/Staged/Committed/Branch; Unified/Split; Stage all/Unstage all; per-file numstat `[historical-record] A §7]` | Implemented with source-switch invalidation; multi-file Changes baseline (grouped patches, expanded-by-default, keyboard-focusable folds, Unified/Split); "BOUNDED DIFF REVIEW" panel + toggle observed `[historical-record] PM; runtime-observed ev/02]`; open: syntax highlighting, inline comments, per-hunk actions, guarded revert | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (Repository page + Changes inspector) | partial | partial | — | PM "Diff review" + §"Multi-file diff baseline"; A §7; B2 §7 ev/02 | ux — P3 (syntax highlighting, inline comments, per-hunk actions, guarded revert) |
| Branches and worktrees | Branch switching/create-and-checkout without force or implicit stash; managed fork worktrees under persisted Worktree root; Settings > Worktrees `[historical-record] A §7]; 0.154 runtime adds experimental `/worktree` (browse + resume) — GUI surfacing at 26.825 `[unverified]` `[docs-derived]` | Implemented; `thread/start` receives the worktree; New Worktree form (branch + path) on Repository page `[historical-record] PM; runtime-observed ev/02]`; `/fork` → worktree destination picker `[source-derived] ui.rs:2746, 7558-7560]` | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (Repository page sections + palette) | partial | partial | — | PM "Branches and worktrees"; A §7; B2 §7 ev/02 | none recorded (watch: `/worktree` GUI surfacing `[unverified]` at 26.825) |
| Pull requests | 420 px Create PR flow: gh preflight, title, generated description, commit-and-push, draft/ready/error states; `/pull-requests` review/merge surface `[historical-record] A §7]; GitLab cloud support (2026-08-19) is cloud-side `[docs-derived]` | Stable-shaped flow incl. `gh --version`, authenticated-status fallback, existing open-PR lookup; PR page renders tabs/filters/search then honest gh-missing banner with "Install GitHub CLI" / "Check again" `[historical-record] PM; runtime-observed ev/03 — lab bound: no gh binary]`; pending branch prefix, force push, draft/merge-method options | win `[historical-record]`; linux `[runtime-observed honest bound]` | partial | partial | complete (dedicated persistent sidebar page) | partial | partial | install gh → "Check again" re-detects | PM "Pull requests"; A §7; B2 §7 ev/03 | ux — P3 (branch prefix, force push, draft/merge-method options) |
| Git process hygiene | No `git.exe` process storm (public failure report documents the official failure) `[historical-record] KF]` | 300 ms debounce, notification coalescing, one backend Git operation at a time (acceptance test) `[historical-record] KF]` | win `[historical-record]` | complete | complete | complete (automatic) | complete | complete | — | KF failure table | none (regression control) |

### 5.8 MCP / Apps / Skills / Plugins

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Plugins marketplace | `plugin/list`/`read`/install/uninstall, search, refresh, enable/disable; OpenAI/Shared/Created/Workspace/Local kinds; marketplace add/upgrade/remove; plugin sharing in protocol `[historical-record] A §8]; 0.146-0.147 runtime: Agent Plugins manifests, workspace publishing, Bedrock + Claude Code marketplaces, portable plugins, cross-catalog search `[docs-derived]` | End-to-end native with exact marketplace kinds `[historical-record] PM]`; marketplace repo syncs to `CODEX_HOME/.tmp/plugins` with 64 curated plugins on disk `[runtime-observed filesystem] B2 §8`; UI catalog empty unauthenticated — "No plugin marketplaces returned entries for this host" with Retry `[runtime-observed ev/04 — BOUND: cannot distinguish auth-bound listing from integration gap without credentials; recorded as watch item, §9]; pending OAuth/no-auth callback completion (ledger bounded) | win `[historical-record]`; linux `[runtime-observed bound]` | partial | partial | complete (dedicated persistent sidebar page) | partial | partial | Retry action re-fetches catalog | PM "Plugins marketplace"; A §8; B2 §8 ev/04 | backend — deferred (OAuth callback, public protocol); watch item: unauthenticated catalog-empty vs disk-sync (needs authenticated re-run, no WO) |
| Marketplace admin-disabled install | `DISABLED_BY_ADMIN` availability; disabled catalog actions + tooltip; `Disabled by admin` details `[historical-record]` | Implemented: availability preserved, actions disabled with recovered `Access is turned off by your admin` tooltip, details show `Disabled by admin` `[historical-record] PM]` | win `[historical-record]` | complete | complete | complete | complete | complete | — | PM "Marketplace admin-disabled install" | none recorded |
| Skills | `skills/list` discovery, scope/path metadata, refresh, `skills/config/write` enable/disable; enabled skills as slash rows + `@` Skills section `[historical-record] A §8]; current docs: skills invoked explicitly with `$`; enabled skills appear in slash list; `/prompts:` custom prompts `[docs-derived]` | Implemented against selected project context; enabled absolute-path skills register `/skill:<path>` rows + `@` Skills section `[source-derived] ui.rs:7550; `[historical-record] PM`; Skills tab on Plugins page `[runtime-observed ev/04]`; `skills/` dir created in CODEX_HOME `[runtime-observed filesystem]`; pending recommended/install flows, creation surfaces; `$` invocation not present `[source-derived absence]` | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (Plugins-page tab + palette "Go to skills") | partial | partial | Force reload skills palette action | PM "Skills"; A §8; B2 §8 ev/04 | ux — P2 (recommended/install flows, creation surfaces, `$` invocation; no WO yet — needs scoping) |
| MCP and apps | Manage Apps / Manage MCPs: bounded list/search, `app/read`, App `Try now`, enablement, browser install/connect fallback, full MCP tool/schema/resource detail; MCP config editor + OAuth + elicitation forms `[historical-record] A §8]` | Implemented for the listed surface `[historical-record] PM]; Settings > MCP servers nav + `/mcp` submenu + `McpElicitation` modal `[source-derived] ui.rs:2193-2197, 7576, 4504; runtime-observed nav ev/07/23]`; pending App OAuth completion (awaits official protocol — ledger bounded) | win `[historical-record]`; linux `[runtime-observed nav]` | partial | partial | complete (Settings nav persistent + slash) | partial | partial | — | PM "MCP and apps"; A §8; B2 §8 | backend — deferred (App OAuth, public protocol) |
| Record & Replay **(added by C2 audit)** | 26.727-era entry: demonstrate a workflow once and turn it into a reusable skill; Computer Use must be available and enabled `[docs-derived] A §8 (changelog 2026-07-30, docs/extend/record-and-replay.md)]` | No Record & Replay surface `[source-derived absence: sweep at f113515]`; Linux Computer Use is screenshot-only (see §5.5) which bounds the demonstrate step on Linux | win, macos `[docs-derived]`; linux `[source-derived absence + platform-bound]` | missing | missing | missing | — | missing | — | A §8 Record & Replay; C2 source sweep | backend — P2 (no WO — Computer-Use-dependent; Linux slice platform-bound; future work order after CU modality decisions) |

### 5.9 Settings / account

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Settings shell | 274 px shell: `Back to app`, bounded search (Ctrl/Cmd+F), Personal/Integrations/Coding/Archived groups, filtering, no-results state; full stable registry of 26 sections `[historical-record] A §9]; **2026-08-11: Settings > Import added** `[docs-derived]`; **Linux preview runtime (WO-LAB-001): the Ctrl+K palette's dynamic Settings group lists 10 pages at the login surface — General, Import, Appearance, Voice, Pets (26.908-only line), Git, Connections, Environments, Worktrees, Configuration** `[runtime-observed: linux-preview 26.908.70816; evidence docs/research/evidence/codex-linux/`03]` | Stable-shaped shell implemented; nav renders 15 rows in 3 groups; SettingsSection enum has 18 sections (CodeReview/Worktrees/ArchivedChats contextual/hidden) `[source-derived] ui.rs:2398-2417; runtime-observed ev/07]; settings search filters nav correctly ("import" → Personal + Import) `[runtime-observed ev/23]; remaining sections only with working host contracts (ledger bounded) | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (persistent sidebar Settings + Ctrl+, + palette) | partial | partial | — | PM "Settings"; A §9; B2 §9 ev/07/23 | backend — deferred (host contracts); cross-ref WO-P2-004 (palette indexing of settings pages) |
| Account and usage | `account/read`, login start/cancel, logout, rate limits, usage; Profile + Usage & billing surfaces `[historical-record] A §9]; usage-limit banners with actions (0.152 runtime) `[docs-derived]` | Separate stable-shaped settings surfaces over one typed official surface; Usage page renders honest sign-in wall ("Sign in to view usage and billing") `[runtime-observed ev/12]; pending billing entry points (ledger enhancement) | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (settings nav persistent) | partial | partial | sign-in unlocks account-powered surfaces | PM "Account and usage"; A §9; B2 §9 ev/12 | ux — P2 (billing entry points; no WO yet — needs scoping) |
| Personalization and memory | None/Friendly/Pragmatic personality (app-server enum authoritative); memory controls feature-gated; `Reset memories` `[historical-record] A §9]; Computer History (2026-08-13, macOS, Pro/Business/Enterprise, opt-in) augments memories — platform + cloud bound `[docs-derived]` | Native Personalization route follows the contract and fixes the renderer's known `None` omission; `memories_1.sqlite` created by runtime; `/memories` + Reset modal `[historical-record] PM; runtime-observed DB + ev/14-adjacent B2 §1/§9]` | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (settings page + slash) | partial | partial | — | PM "Personalization and memory"; A §9; B2 §1/§9 | none recorded (Computer History = deferred: macOS + cloud) |
| Import and migration | Baseline: NO official import surface — `externalAgentConfig/*` typed methods existed in the pinned app-server only `[historical-record] A §9]; **current target (2026-08-11): Settings > Import for Claude Code / Claude Cowork / Cursor** with auto-update sync `[docs-derived]` | Native Personal → Import route implemented with the same three providers (`ImportProvider::{ClaudeCode,ClaudeCowork,Cursor}`) `[source-derived] core lib.rs:3478-3485, ui.rs:34696+]; nav row + search filter observed `[runtime-observed ev/23]; full page content not captured (coordinate-navigation bound) `[runtime-observed bound] B2 §9]; pending unsupported-project reporting (awaits public protocol) — Flauz is AHEAD of the historical baseline here (implemented before the official UI shipped) | win `[historical-record]`; linux `[runtime-observed nav]` | partial | partial | complete (settings nav persistent; palette indexing gap → WO-P2-004) | partial | partial | — | PM "Import and migration"; A §9; B2 §9 ev/23 | backend — deferred (unsupported-project reporting, public protocol) |
| Remote control and SSH | `remoteControl/status/read`, enable/disable, pairing, paginated client discovery, revoke; keep-awake, SSH profiles, remote chats `[historical-record]` | Native Connections UI implemented for the public methods; "Remote control — This computer (Disabled)" + paired-devices empty state observed `[runtime-observed ev/13]; pending SSH profiles/remote chats (await public contracts — ledger bounded) | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (settings nav persistent) | partial | partial | — | PM "Remote control and SSH"; B2 §9 ev/13 | backend — deferred (public contracts) |
| Cloud environments | List/detail/create links, repository/machine metadata, target selection, connection state, cloud execution — OpenAI cloud backend `[historical-record]` | Not implemented `[historical-record] PM; source-derived absence B2 audit]` | win `[historical-record]` | deferred | deferred | — | — | deferred | — | PM "Cloud environments" | backend (proprietary) — deferred |

### 5.10 Product shell

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| First run and updates | About window: package version, copyright, focus behavior; first-run/login flow; failed backend connection visible with Retry/restart guidance `[historical-record] A §10]` | Help → `About codexRS` native 380×360 floating window, centered, refocuses existing instance, OK/Escape close; no-runtime error state degrades gracefully ("Resolving…" + auto-retry) `[historical-record] PM; runtime-observed ev/24]; pending fuller welcome, dependency diagnostics, update prompt (ledger enhancement) | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | complete (Help menu persistent) | partial | partial | retry/restart guidance visible in empty Tasks area | PM "First run and updates"; A §10; B2 §10 ev/24 | ux — P2 (welcome, diagnostics, update prompt; no WO yet — needs scoping) |
| Keyboard and accessibility | Ctrl/Cmd+K, Shift+P, G, P search-files drill-in, arrows/Enter/Escape; dynamic Settings group in palette; editable shortcut registry with stable grouping; complete focus order, screen-reader labels `[historical-record] A §10]; current docs add Clear terminal (Ctrl+L/Ctrl+K), font-size Ctrl+±/0, Toggle file tree Ctrl+Shift+E (post-baseline, intro dates `[unverified]`) `[docs-derived]`; **Linux preview runtime (WO-LAB-001, 2026-09-17): Ctrl+/ opens a searchable "Keyboard shortcuts" overlay ("Search shortcuts") with 22 runtime-read rows — Chat (New chat Ctrl+N/Ctrl+Shift+O; Archive chat Ctrl+Shift+A; New standalone chat Ctrl+Alt+O; Toggle pin Ctrl+Alt+P), Navigation (Find Ctrl+F; Back Ctrl+[/Mouse Back; Forward Ctrl+]/Mouse Forward; Next/Previous recently viewed chat Ctrl+Tab/Ctrl+Shift+Tab; Switch to Work Alt+2), General (Close Tab Ctrl+W/Ctrl+F4; Copy deeplink Ctrl+Alt+L; Copy working directory Ctrl+Shift+C; Reload/Force Reload Browser Page Ctrl+R/Ctrl+Shift+R; Open command menu Ctrl+K/Ctrl+Shift+P; Rename chat Ctrl+Alt+R; Search Files... Ctrl+P; Show keyboard shortcuts Ctrl+/; Toggle File Tree Ctrl+Shift+E — runtime-confirming the docs-derived file-tree binding; Clear terminal/font-size rows not in the captured sections)** `[runtime-observed: linux-preview 26.908.70816; evidence docs/research/evidence/codex-linux/`05-07]` | Native command palette with verified stable registry subset (45-command registry `[source-derived] ui.rs:3244+]); Ctrl+/ overlay + editable settings page `[runtime-observed ev/11, ev/15]; **palette does not index all settings pages — "import" → "No matches" while the Import page exists in settings nav** `[runtime-observed ev/18 vs ev/23; source-derived: PaletteCommand has no entries for Profile/Import/Browser/Configuration/Hooks/Git settings]; pending remaining stable commands, complete focus order, screen-reader labels, reduced-motion | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | partial (palette lacks settings-page entries; no toolbar button for palette) | partial | partial | — | PM "Keyboard and accessibility" + §"Keyboard accessibility baseline"; A §10; B2 §10 ev/08-11/15/18 | ux — P2 (palette settings-page indexing → **WO-P2-004**; remaining stable commands, focus order, screen-reader labels); P3 (Clear terminal / font-size / file-tree shortcut deltas) |
| Keyboard shortcut reference | Exact stable `Keyboard shortcuts` dialog listing only active bindings `[historical-record]` | Implemented: opens from Help + Ctrl/Cmd+/, stable category order; overlay + searchable editable settings page both visually confirmed `[historical-record] PM; runtime-observed ev/11, ev/15]` | win `[historical-record]`; linux `[runtime-observed]` | complete | complete | complete (Help + Ctrl+/) | complete | complete | — | PM "Active keyboard shortcut reference"; B2 §10 ev/11/15 | none recorded |
| Notifications and tray | Background completion notification (banner + Windows quiet-hours toast / Linux freedesktop best-effort); tray tooltip count `[historical-record] A §10]` | Bounded in-app banner (Open/Dismiss) + matching window-title/notification-area tooltip; gh-missing toast banners observed on multiple pages `[runtime-observed ev/03/04]; pending tray groups, badges, sounds (ledger enhancement); Linux tray/global shortcuts unavailable (see packaging row) | win `[historical-record]`; linux `[runtime-observed]` | partial | partial | partial (passive banners) | partial | partial | — | PM "Notifications and tray"; A §10; B2 §9 ev/03/04 | ux — P2 (tray groups, badges, sounds; no WO yet — needs scoping) |
| Feedback | Exact stable `Feedback` command + `/feedback`; five category IDs; required details; default-on session logs `[historical-record]` | Implemented as native `Share feedback` dialog with the five recovered category IDs, validation, default-on logs, no browser-tabs control; palette entry exists `[historical-record] PM; source-derived palette]` | win `[historical-record]`; linux `[source-derived]` | complete | complete | complete (palette + slash + Help paths) | complete | complete | — | PM "Feedback"; B2 audit | none recorded |
| Appshots | macOS-only at baseline (foreground-window capture flow; Settings `appshots` section in stable registry) `[historical-record] A §5]; **26.908 (reference+1): Appshots on Windows — NOT part of the 26.825 target** `[docs-derived]` | Not implemented — proprietary capture flow (ledger bounded) `[historical-record] PM; source-derived absence B2 audit]` | macos `[historical-record]` | deferred | deferred | — | — | deferred | — | PM "Appshots"; A §5/§9 | backend (proprietary) — deferred (26.908 Windows arrival is reference+1, out of target) |
| Pets and Codex Micro | Settings sections registered at baseline (`pets`, `codex-micro`) `[historical-record] A §10]; **26.908 (reference+1): Pets quick chat + Codex Micro `Insert text` — NOT part of the 26.825 target** `[docs-derived]` | Not implemented — proprietary companion surface (ledger bounded) `[historical-record] PM; source-derived absence B2 audit]` | win, macos `[historical-record]` | deferred | deferred | — | — | deferred | — | PM "Pets and Codex Micro"; A §10 | backend (proprietary) — deferred (26.908 features are reference+1, out of target) |
| Windows packaging | Signed MSIX installer + desktop integration + updating `[historical-record]` | Unsigned portable ZIP release strategy (code signing unavailable to this program); SHA-256 checksums + documented verify step; no MSI/installer/updater `[historical-record] PM ledger; KF limitations]` | win `[historical-record]` | partial | partial | partial (documented verify step) | difference documented | partial | verify via published SHA-256 | PM ledger "Windows"; KF limitations | intentional difference — documented (GUI-006) |
| Linux packaging | No official Linux target at baseline `[historical-record]`; **official Linux preview app exists since 2026-08-11** `[docs-derived] A platform table]` | Unsigned portable tar.gz; explicit `codexrs --install-desktop-entry` per-user integration (never overwrites); no system package, Wayland portals, tray, or global shortcuts — documented in `docs/platform-support.md` `[historical-record] PM ledger; KF; runtime-observed B2 wave]` | linux `[runtime-observed]` | platform-limited | platform-limited | partial (documented `--install-desktop-entry` path) | platform-limited (documented) | platform-limited | documented `--install-desktop-entry` path | PM ledger "Linux"; KF limitations; docs/platform-support.md | platform — documented; lab-upgrade opportunity WO-LAB-001 may re-baseline this row's reference layer |
| Stable-failure regression controls | Public failure reports: Windows multi-root white screen; 594 MB JSONL line; ~9 GB startup history scan; `git.exe` storm; process-cleanup storm; unbounded logging `[historical-record] KF]` | All six controlled as acceptance tests: native paths; bounded app-server pages + `useStateDbOnly: true`; 300 ms Git debounce; one supervised tree + Windows Job Object; narrowly scoped owned logging `[historical-record] KF]` | win, linux `[historical-record]` | complete | complete | complete (automatic) | complete | complete | controls are standing acceptance tests | KF failure table | none (regression controls) |
| Activity view & unread attention **(added by C2 audit)** | **26.727: Activity view** — bell icon / Ctrl/Cmd+Alt+U shows recently engaged chats needing attention; Shift+Esc clears unread indicators; Ctrl/Cmd+Alt+A next chat needing attention `[historical-record + docs-derived] A §10]` | No Activity view or unread-attention surface: no bell, no Ctrl+Alt+U / Ctrl+Alt+A / Shift+Esc bindings, no unread state `[source-derived absence: key registry ui.rs:4459-4552; sweep at f113515]` | win, macos `[docs-derived]`; linux `[source-derived absence]` | missing | missing | missing | — | missing | — | A §10 Sidebar; B2 shortcut inventory; C2 source sweep | backend — P2 (unread-state dependent; no WO yet — needs scoping; upgrades the PM ledger's P3 "unread state" note, §9) |

---

## 6. Pre-population and reconciliation method (audit trail)

- **Pre-population (C1):** one canonical row per historical capability row from
  `docs/parity-matrix.md` (§Product parity table, §Installed Desktop route
  inventory, §Settings inventory, §Protocol progress, §Release-critical parity
  ledger) and `docs/known-failures.md`, regrouped into the ten operator
  feature sections; platform-split rows preserved (Computer Use, packaging).
- **Reconciliation (C2, 2026-09-16):** inputs were (1) Worker A's
  `CODEX-REFERENCE-MATRIX.md` at `c083c38` (official side, 11 sections +
  runtime surface + journeys + shortcuts), (2) Worker B2's
  `FLAUZ-REFERENCE-MATRIX.md` at `a2343d3` (Flauz side, 10 sections +
  journeys + 35-row shortcut inventory + 3 findings + corrected counts), (3)
  A's `evidence/codex-ref/version-delta-notes.md` (+ changelog extract,
  CLI runtime surface, features list, doctor), (4) B2's
  `evidence/flauz/` (25 curated runtime captures + 8 scene scripts), and
  (5) the C1 skeleton at `5d8ad61`.
- C2 re-verified load-bearing Flauz-side source claims directly at
  `main f113515` (read-only): slash executor + menu catalog
  (`ui.rs:7510-7621`, `22892-23120`), `PaletteCommand::ALL` (45 commands,
  `ui.rs:3195-3241+`), `SettingsSection` enum (`ui.rs:2398-2417`), terminal
  guard (`core lib.rs:8163`), browser guard (`core lib.rs:14521`),
  `InspectorPane` (`core lib.rs:259-267`), ctrl-` binding (`ui.rs:4499`),
  `Action::ToggleTerminalDock` (`core lib.rs:4923`), Computer Use Linux gate
  (`computer_use.rs:34-57`), project model (`LocalProjectSummary` single
  path, `core lib.rs:908/4719`), and absence sweeps for side chats, unread/
  Activity view, in-app editing, Record & Replay, WebMCP, browser extension.
- Row-matching followed §7; conflicts resolved by §7.3 order with every
  override recorded in §9. Six rows were added under the §7.2
  A-row-without-B-counterpart rule (marked "added by C2 audit"), each with
  the Flauz-side absence source-verified before declaring.
- The `26.825.51511` delta is annotated inside the affected rows (per §7.7)
  and detailed in `docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md`.
  Historical rows were never deleted.

## 7. Reconciliation procedure (for C2 — executed 2026-09-16; retained as governing record)

### 7.1 Inputs

1. `CODEX-REFERENCE-MATRIX.md` — Worker A (official Codex side, provenance-labeled).
2. `FLAUZ-REFERENCE-MATRIX.md` — Worker B (Flauz side, source-derived + LINUX_GUI_LAB runtime-observed).
3. This report's PRE-A/B rows (historical baseline layer).
4. `docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md` (+ Worker A's
   `docs/research/evidence/codex-ref/version-delta-notes.md`) for
   version-skew decisions.

### 7.2 Row-matching rules

- Match by **capability identity, not wording**. Canonical row names live in
  §5; A/B rows join onto them through the ten feature sections.
- A-row without B counterpart → candidate `Backend gap` or `missing`
  (confirm the Flauz side truly has nothing: source sweep + runtime check
  before declaring).
- B-row without A counterpart → candidate `Intentional difference` (requires
  an operator approval on record) or Flauz-extra surface; never silently
  deleted.
- One capability = one row **per platform slice** when behavior or status
  differs by OS (Computer Use is the existing precedent: Windows row + Linux
  row).
- Never merge rows across feature sections; if A's row spans sections, split
  it at reconciliation time.
- Historical PRE-A/B rows are the fallback layer: where A/B matrices are
  silent, the historical cell stands and keeps its `[historical-record]`
  label.

### 7.3 Conflict resolution order

```text
runtime-observed > source-derived > docs-derived > [historical-record]
```

- A higher-order label **overrides** a lower-order label for the same cell;
  the losing value is recorded in the Evidence cell (and §9) so the audit
  trail survives.
- Within the same label class, the more recent capture wins; ties are broken
  by Worker A (official side) / Worker B (Flauz side) ownership of the cell,
  and recorded.
- A claim may never be labeled above its actual evidence class (docs-derived
  material is never presented as runtime-observed).

### 7.4 Per-platform column rules

- The `Platform` column names the slices the row's evidence covers. A status
  is only valid for a platform slice that has evidence.
- **Linux runtime-observed evidence never fills Windows/macOS cells**, and
  Windows/macOS `[historical-record]`/`[docs-derived]`/`[source-derived]`
  evidence never asserts Linux runtime behavior. No silent platform
  substitution — if a platform has no evidence, its cell reads `—` with the
  bound named (e.g., "Windows: unvalidated — WINDOWS_GUI_LAB unavailable").
- `platform-limited` is only used with the binding platform named in the
  `Platform` column (Computer Use — Linux is the precedent;
  WO-PLAT-001 keeps it a platform gap, not a missing feature).

### 7.5 Confirmed-gap → work-order flow

1. A gap is **confirmed** only when both sides are evidenced (A-row + B-row
   or historical fallback), the conflict order (§7.3) has been applied, and
   the gap type (§4.1) + priority class (§4.4) are assigned.
2. Confirmed gaps enter `docs/research/FEATURE-PARITY-WORK-ORDERS.md` via
   its template; DRAFT entries there are confirmed, rewritten, or withdrawn
   at this point.
3. The parity row's `Gap` cell receives the work-order ID.
4. Deferred/proprietary rows do not spawn work orders; they stay `deferred`
   with the blocker named until the operator or a public contract changes.

### 7.6 Row closure

A row (or cell) moves to `complete` only through the work-order closure
rules (five conditions, see `FEATURE-PARITY-WORK-ORDERS.md` §Rules): source
changed, tests pass, GUI behavior verified, the relevant platform lab passes,
and the row is updated. **Source-code presence is never "feature complete".**

### 7.7 Version-skew handling

Where `26.825.51511` changed behavior vs the `26.721.3996.0` baseline (per
the version-delta doc), C2 annotates the affected rows with the delta and
decides per row whether the parity bar moves (current target) while keeping
the historical evidence. Historical rows are never deleted.

---

## 8. Reconciliation summary (C2, post A/B audit)

### 8.1 Final counts

**53 rows** (47 carried from the C1 skeleton — all preserved — plus 6 added
by the audit per §7.2):

| Status | Count | Rows |
| --- | --- | --- |
| `complete` | **7** | Runtime bootstrap; Multi-root workspace handling (regression control); Git process hygiene; Marketplace admin-disabled install; Keyboard shortcut reference; Feedback; Stable-failure regression controls |
| `partial` | **31** | all §5.1-§5.10 rows not listed elsewhere (incl. Terminal, In-app browser, Projects and chats, Settings shell, Import and migration, etc.) |
| `missing` | **6** | Side chats; WebMCP site tools; Browser extension (adjacent); In-app Markdown/code editing; Record & Replay; Activity view & unread attention |
| `platform-limited` | **2** | Computer Use — Linux; Linux packaging |
| `deferred` | **7** | Scheduled tasks; Voice input; Sites; Visualizations; Cloud environments; Appshots; Pets and Codex Micro |

Relationship to B2's corrected historical counts (4 done / 29 partial /
7 missing / 1 platform over 41 PM rows): **zero status flips** — B2's 13-row
audit sample confirmed the historical record for what it claims. The 7 PM
"missing" rows are carried here as `deferred` per the C1 mapping rule
(ledger verdict `bounded` → deferred: proprietary cloud backend or pending
public protocol). The 6 `missing` rows are capabilities the historical
ledger never decomposed, surfaced by the A/B audit (side chats, WebMCP,
browser extension, in-app editing, Record & Replay, Activity view) — each
with Flauz-side absence source-verified at `f113515` before declaring.

### 8.2 Priority lists (derived from the reconciled Gap cells)

**P0 — none confirmed.** No gap blocks normal usage: the app boots to a
working entry surface, auth is honest, and degradation paths recover. The
P1 discoverability gaps make existing features hard to find, not the app
unusable.

**P1 — major feature missing or effectively undiscoverable (3 confirmed; 2
CLOSED 2026-09-17, 1 open):**

1. Terminal discoverability — **CLOSED (WO-P1-001, merged `d15333e` via PR
   #13, 2026-09-17)**: persistent sidebar affordance + honest entry-surface
   guard on main; runtime evidence docs/research/evidence/wo-p1/.
2. Browser discoverability — **CLOSED (WO-P1-002, merged `d15333e` via PR
   #13, 2026-09-17)**: persistent sidebar affordance + honest entry-surface
   guard on main; runtime evidence docs/research/evidence/wo-p1/.
3. Multi-folder local projects — in-baseline official capability (26.715)
   wholly absent (single-path project model) → **WO-P1-003 IMPLEMENTED
   (PR #15: 1d2abca + clippy 2ea5970, 2026-09-17)**: model + Edit project
   surface + primary-swap re-key + related-folder file search + storage
   schema v4; 601 tests green locally; runtime evidence
   docs/research/evidence/wo-p1-003/; CLOSED on merge with row flip.

**P2 — meaningful difference (with work orders, 3):**

4. Command palette does not index all settings pages ("import" → No matches
   while the Import page exists in nav; 6 nav pages unindexed) → **WO-P2-004**.
5. Slash-command coverage delta vs current 24-command set (11 literal names
   absent; subset implementable now) → **WO-P2-005**.
6. Side chats (in-baseline Ctrl+Alt+S + `/side`) → **WO-P2-006**.

**P2 — recorded in rows, no work order yet (needs scoping / blocked, 11):**

7. In-app Markdown/code editing (26.707, in-baseline) — editor surface needs
   Tech Lead scoping before a bounded order can be written.
8. Activity view & unread attention (26.727) — depends on unread state.
9. WebMCP site tools (26.825) — blocked on fork-runtime WebMCP capability.
10. Record & Replay (26.727-era) — Computer-Use-dependent; Linux slice
    platform-bound.
11. Browser extension beyond Chrome (26.825) — adjacent deliverable, out of
    parity-wave scope (distribution surface beyond the app binary).
12. Browser address-bar history / Google fallback (26.727).
13. Multi-repository review (26.727) — blocked-by WO-P1-003.
14. Permission profiles granular/custom editor, sandbox detail, per-project
    resolution (ledger enhancement).
15. Artifacts/outputs renderers + canvas actions + generated-image editing
    (26.727) (ledger enhancement).
16. Dynamic Computer Use tools (attach after creation) (KF limitation).
17. Remaining stable app-server methods (supervision); Account billing entry
    points; First-run welcome/diagnostics/update prompt; remaining stable
    palette commands + focus order + screen-reader labels + reduced-motion;
    tray groups/badges/sounds; Skills recommended/install flows + `$`
    invocation (ledger enhancements).

**P3 — cosmetic (7):** thread-execution compaction provenance + edit
metadata; streaming-timeline grouping/citation navigation; projects richer
metadata; model-catalog churn residuals (data-side); diff-review syntax
highlighting/inline comments/per-hunk actions/guarded revert; PR branch
prefix/force push/draft-merge options; shortcut deltas (Clear terminal
Ctrl+L/Ctrl+K, font-size Ctrl+±/0, Toggle file tree Ctrl+Shift+E — current
docs, not in Flauz registry).

**Deferred (no priority class — bounded, 15 items; 7 whole rows + 8 in-row
bounds):** Scheduled tasks (+
event-triggered, 26.825); Voice; Sites; Visualizations; Cloud environments;
Appshots (26.908 Windows = reference+1); Pets (26.908 = reference+1);
approvals connector-specific methods; plugins OAuth/no-auth callback; App
OAuth; import unsupported-project reporting; SSH profiles/remote chats;
Settings host contracts; Computer History (macOS + cloud); unified
pins/shared thread snapshots (cloud).

**Intentional differences (documented, not fixed):** Windows unsigned
portable packaging (GUI-006); native GPUI instead of Electron (program-level).

### 8.3 Journey table (J1-J6)

| Journey | Official expected flow (Worker A) | Flauz result (Worker B2) | Works | Bound | GAP → work order |
| --- | --- | --- | --- | --- | --- |
| J1 Coding | New chat → Open folder (Ctrl+O) → task + streaming timeline + approvals → terminal + Process Manager → outputs/changes → diff/commit (+ multi-repo review 26.727+) `[historical-record]` | Entry surface + composer pickers + palette "Open folder" + settings all render (ev/01/08/19); Repository page full Git surface (ev/02) | Shell + Git surfaces work | Authenticated turn execution (send blocked unauth, ev/21 — draft retained, sign-in card) | None beyond auth; terminal affordance — WO-P1-001 CLOSED (d15333e); multi-folder — WO-P1-003 IMPLEMENTED (PR #15; runtime evidence wo-p1-003/) |
| J2 Browser-assisted | Browser context per turn → Ctrl+T / Ctrl+Shift+B → permission cards → downloads (+ site tools 26.825) `[historical-record + docs-derived]` | Ctrl+T/Ctrl+Shift+B were silent no-ops on the entry surface pre-wave (WO-P1-002 merged d15333e now surfaces honest guards, ev wo-p1/12); panel source-verified (`browser.rs`) | In-chat panel (source-derived) | Needs open chat → auth | WO-P1-002 CLOSED (d15333e) — discoverability landed; WebMCP row (P2, no WO) remains |
| J3 Planning | `/plan` → `/goal` → progress row → `/status` → `/compact` `[historical-record]` | `/plan` + composer menu item, `/goal` editor, model/effort/speed selectors visible (ev/01/19/20) | Composer surfaces work | Plan/goal execution needs a thread (auth) | None beyond auth |
| J4 Git | Snapshot → worktree fork → diffs → review → commit/push → PR `[historical-record]` | Repository page (stats, changed files, branches, worktree form, bounded diff review, "Commit or push") renders (ev/02); PR page renders then honest gh-missing banner (ev/03) | Works (commit/push prior-wave validated `[historical-record]`) | gh CLI absent in lab (honest, actionable banner) | None; multi-repo review unblocked once WO-P1-003 merges (PR #15) |
| J5 Recovery | Interrupt → reconnect timer → `thread/loaded/list` rehydrate → resume `[historical-record]` | Restart persistence verified end-to-end (window 1100×900 + route + inspector restored; state-DB row verified, ev/22); no-runtime graceful degradation ("Resolving…", auto-retry, no crash, ev/24) | Works | Mid-turn crash recovery needs an account | None |
| J6 Extensibility | MCP add/oauth/status → plugin install → skills (`$`/slash) → approvals (+ Record & Replay 26.727-era) `[historical-record + docs-derived]` | Plugins/Skills pages render with filters/search (ev/04); marketplace repo synced to disk (64 curated plugins) `[runtime-observed filesystem]`; MCP settings nav + `/mcp`; Workflows page (frozen area) (ev/05) | Surfaces work | Catalog UI empty unauthenticated ("No plugin marketplaces returned entries for this host"); install/enable flows not evidenced | None confirmed — catalog-empty vs disk-sync recorded as BOUND watch item (§9); settings-page discovery → WO-P2-004 |

### 8.4 Lab upgrade opportunity (follow-up)

Worker A discovered that an **OFFICIAL Linux desktop app now exists in
preview** (since 2026-08-11; `.deb`/`.rpm`/install script; Ubuntu 24.04/26.04,
Debian 13, Fedora 43/44 + Arch; x64 + ARM64; install host
`persistent.oaistatic.com/codex-app-prod/linux/...`) `[docs-derived]`. The
local LINUX_GUI_LAB sandbox is Debian 13 — inside the app's supported
matrix. **Recommendation (follow-up work order WO-LAB-001, NOT a silent
change to the platform bounds):** attempt to run the official Linux preview
app inside the local lab (userspace `.deb` extraction per the established
`fetch_debs.py` pattern; Xvfb + picom + capture). If it runs, Linux-A
official-side evidence upgrades from evidence-layers to
`[runtime-observed]` for every captured surface, the E2B-PARITY-ENVIRONMENT
bounds doc is updated through the Tech Lead, and several `[unverified]`
questions (Linux Computer Use scope, Activity view, terminal/browser
affordance shapes) become directly checkable. Until then, all platform
bounds in §3 stand unchanged. Constraints to respect: unauthenticated lab
(account surfaces stay bound), preview-quality app, ~600 MB free disk.

**EXECUTED (2026-09-17, WO-LAB-001):** the official Linux preview app
(`chatgpt 26.908.70816`, selective `.deb` userspace) now runs in
LINUX_GUI_LAB (Xvfb + picom; isolated profile; no credentials). Unauthenticated
surfaces captured + VLM-verified under
`docs/research/evidence/codex-linux/`: login surface ("Sign in to ChatGPT" /
"Continue to sign in" / "Sign in with an API key" / "Sign up"), command
palette (dynamic Settings group with 10 pages; "Panels: Open terminal
(Ctrl+`)"; "Chats" group), searchable "Keyboard shortcuts" overlay (22 rows),
auth-pending surface ("Continue signing in with your browser" / "Cancel
sign-in" / "Copy sign-in link"), and the DB-recovery dialog elicited by the
lab's missing-native state ("Back Up and Rebuild"). Affected official Linux
cells upgrade to `[runtime-observed: linux-preview 26.908.70816]` with
version-skew labels (26.908 = reference+1; the 26.825 target is unchanged).
Bounds unchanged: account surfaces stay auth-walled; Windows/macOS cells stay
evidence-layered. Startup-behavior delta recorded (§9 override 15): official
fatals without its bundled runtime; Flauz degrades gracefully (ev/24).

---

## 9. Override audit trail (C2 reconciliation)

Every application of the §7.3 conflict order that changed or reclassified a
cell, in report order:

1. **Flauz slash inventory (B2 enumeration → C2 source re-verification).**
   B2 §2 enumerated 15 native named commands. Direct source read at `f113515`
   shows **16** named commands (baseline 15 + `/review` — "Code review",
   executor `ui.rs:7557-7570`, menu `ui.rs:22935-22946`) plus dynamic
   `/service-tier:<id>` and `/skill:<path>` rows. Same evidence class
   (`[source-derived]`), more recent capture wins per §7.3 → C2's 16-command
   inventory stands. Consequence: Flauz already matches the current official
   `/review` command; the coverage delta is 11 missing literal names, not 12.
2. **Baseline slash count (A enumeration vs count).** A §2 labels the
   baseline set "(15)" but enumerates 14 names; the 15th (`/shell`) is
   evidenced in A §3 and PM "Thread execution". Resolution: baseline set =
   15 incl. `/shell`; Flauz covers the baseline set 15/15. No status change;
   discrepancy recorded, not silently resolved.
3. **Projects "multi-root sources" classification (P3 polish → P1 backend
   gap).** The PM ledger carried "multi-root sources" as `polish` (P3).
   A's finding that multi-folder local projects are **in-baseline**
   (26.715 < 26.721, `[docs-derived]`) plus C2's source verification that
   Flauz's project model is single-path (`[source-derived]`) reclassifies
   this to a backend gap, P1 → WO-P1-003. Higher-order evidence (docs +
   source) overrides the historical ledger's delta class per §7.3.
4. **Unread state (P3 polish → P2, current target).** Same pattern: PM
   ledger listed "unread state" as polish; A's Activity view finding
   (26.727, in the current target) makes unread/attention a P2 capability
   (Activity view row). Historical evidence retained; the current-target
   bar moves it per §7.7.
5. **Slash "remaining slash commands (polish)" (P3 → resolved for baseline;
   P2 for current target).** PM ledger P3 note referred to the baseline set;
   C2 reconciliation shows the baseline set fully covered (15/15) plus
   `/review` extra. The current-target delta (11 missing literal names) is
   a P2 gap → WO-P2-005, scope-limited to commands with existing backing
   capability.
6. **Terminal row: "ledger green" vs entry-surface no-op.** Not a
   contradiction: the ledger's green verdict covers in-chat terminal
   functionality `[historical-record]`; B2's `[runtime-observed]` no-op
   concerns the entry surface and discoverability only. Row stays `partial`
   with a UI-integration gap (P1, WO-P1-001); no status flip, explicit
   merge recorded.
7. **Plugins marketplace: "end-to-end native" (ledger) vs catalog-empty
   (B2 runtime).** Not a contradiction: the ledger claim rests on
   authenticated/native-fixture validation `[historical-record]`; B2's lab
   is unauthenticated. The catalog-empty vs 64-plugin-disk-sync observation
   is recorded as a **BOUND watch item** (cannot distinguish auth-bound
   listing from integration gap without credentials; official reference
   behavior unauthenticated is `[unverified]` — auth-walled). No work order
   (rule 6: no reference behavior); re-verification path: authenticated
   J6 re-run.
8. **Experimental method count (126 vs 89).** PM `[historical-record]`
   claims "experimental schema: 126 client request methods"; A's
   `[runtime-observed]` schema generator emits 89 `ClientRequest` methods
   in both stable and v2 bundles. A's resolution adopted: measurement
   difference (extra methods presumably runtime-negotiated behind
   `experimentalApi: true`), not a contradiction. Runtime observation
   outranks the historical claim for the reproducible number.
9. **"Multi-root workspace handling" (complete) vs multi-folder gap (P1).**
   Both rows stand: the former is the Windows multi-root white-screen
   regression control (KF failure table); the latter is official multi-folder
   project management (A §6). Different capabilities — no merge, no
   substitution.
10. **Process Manager affordances.** B2 §1 called it "palette-only
    (+shortcut)"; PM records three affordances (palette, Ctrl+Alt+M,
    background-terminal activity). Merged: the historical record retains the
    third affordance; Discoverability cell reads complete (matches official
    affordance set). No conflict of classes.
11. **Ctrl+` binding provenance.** Official `Toggle terminal` Ctrl+` is a
    current-docs binding with `[unverified]` introduction date (post-baseline
    per A §3); Flauz already binds ctrl-` (`ui.rs:4499`). No gap on the
    binding itself — WO-P1-001 concerns the missing persistent affordance
    and the entry-surface no-op, not the key binding.
12. **Import and migration direction.** A §9 records that officially there
    was NO import UI at baseline (protocol methods only); Flauz implemented
    the typed methods + native Import route before the official Settings >
    Import shipped (2026-08-11). Flauz is ahead of the historical baseline
    and ≈ at parity with the current target (same three providers); the
    remaining bound (unsupported-project reporting) stays deferred. Row
    stays `partial` pending that public protocol.
13. **Computer Use — Linux reference layer.** The official reference for
    this row is Windows `[historical-record]`; the official Linux preview
    app's Computer Use scope is `[unverified]` `[docs-derived]`. No Linux
    official claim is made; WO-PLAT-001 documents the bound as-is.
14. **Official Linux preview runtime layer (WO-LAB-001, 2026-09-17).**
    Linux-A official-side evidence upgraded from evidence-layers to
    `[runtime-observed: linux-preview 26.908.70816]` for the unauthenticated
    slice: login surface (the full shell is auth-gated pre-sign-in), palette
    (Settings group with 10 pages; Panels: Open terminal; Chats group),
    Ctrl+/ shortcuts overlay (22 rows), auth-pending surface, DB path
    (`~/.codex/sqlite/codex-dev.db`) + recovery dialog. Version-skew rule:
    26.908 observations never silently upgrade 26.825 claims — rows cite the
    preview build; Pets flagged 26.908-only. Affected rows: §5.3 Terminal,
    §5.4 Browser, §5.9 Settings shell, §5.10 Keyboard and accessibility.
15. **Official startup dependency (runtime-observed).** The official Linux
    app hard-fails at startup without its bundled codex runtime (fatal
    "Unable to locate the Codex CLI binary or required runtime components";
    no window opens). Recorded as an official-side resilience bound, NOT a
    Flauz gap — Flauz's graceful no-runtime degradation ("Resolving…",
    auto-retry, ev/24) is ahead on this axis. No status flips (rows already
    partial for other reasons).
16. **Runtime-surfaced shortcut delta (folded into the §5.10 P3 gap, not a
    new work order).** The official overlay carries bindings absent from
    Flauz's 45-command registry/palette: Search Files Ctrl+P, Copy deeplink
    Ctrl+Alt+L, Copy working directory Ctrl+Shift+C, Rename chat Ctrl+Alt+R,
    Close Tab Ctrl+W, Archive chat Ctrl+Shift+A, New standalone chat
    Ctrl+Alt+O, Toggle pin Ctrl+Alt+P, Back/Forward Ctrl+[/], recent-chat
    cycling Ctrl+Tab/Ctrl+Shift+Tab, Switch to Work Alt+2. The existing P3
    shortcut-delta gap's scope is extended by this runtime list; any new
    work order follows the C2 confirmed-gap flow or operator directive.
17. **Runtime finding during the WO-P1-003 lab scene (2026-09-17): Ctrl+P
    is a silent no-op in Flauz.** The keybinding exists
    (ui.rs:4527 binds Ctrl+P to the `OpenFileSearch` action) but no handler
    is registered anywhere in the workspace, so the binding dispatches into
    nothing — the same silent-no-op pattern WO-P1-001/002 closed for the
    Terminal/Browser entries. The palette command "Search files"
    (Ctrl+K → "search files") is the working file-search entry and is what
    the WO-P1-003 evidence uses. Folded into the §5.10 P3 gap (binding
    wiring, not a capability gap — the search itself works); any new work
    order follows the C2 confirmed-gap flow or operator directive.

---

## 10. Change log

- 2026-09-17 (WO-P1-001 + WO-P1-002 merge closure): both P1 discoverability
  gaps CLOSED on main — merge commit d15333e718a7649238f33a183af27c885dbaa8aa
  (PR #13; implementation 08d060c + clippy fix aa2d89d); CI green on
  windows-latest and ubuntu-24.04; §5.3/§5.4 rows now reference the merge
  commit + GUI evidence (docs/research/evidence/wo-p1/); §8.2 P1 list
  updated (2 closed, 1 open); J1/J2 updated. Operator review 2026-09-17
  accepted the wave and directed: WO-LAB-001 (official Linux preview app,
  runtime comparison vs Flauz Linux GUI) before WO-P1-003 implementation.
- 2026-09-17 (WO-LAB-001 executed — official Linux preview app in the lab):
  `chatgpt 26.908.70816` (.deb selective userspace) launched under Xvfb+picom
  with an isolated profile; unauthenticated surfaces captured + VLM-verified
  (docs/research/evidence/codex-linux/: 9 frames + scene scripts + log
  extracts + vlm transcripts + deb metadata). §5.3/§5.4/§5.9/§5.10 official
  Linux cells upgraded to `[runtime-observed: linux-preview 26.908.70816]`
  with version-skew labels; §8.4 marked EXECUTED; §9 overrides 14-16 added
  (runtime layer; official fatal-vs-Flauz-graceful startup dependency;
  runtime-surfaced shortcut delta folded into the P3 gap). Bounds unchanged:
  account surfaces auth-walled; Windows/macOS cells evidence-layered.
- 2026-09-16 (WO-P1-001 + WO-P1-002 implementation, branch parity/wo-p1-discoverability):
  Terminal and Browser Discoverability cells updated to IMPLEMENTED (pending
  merge) with runtime evidence (docs/research/evidence/wo-p1/: default-layout
  affordances VLM-read; entry-surface frames now differ with the guard
  surfaced and a Dismiss control). Statuses remain partial pending merge+CI;
  closure gate 5 satisfied on merge.

| Date | Change |
| --- | --- |
| 2026-09-16 | Skeleton + definitions + PRE-A/B pre-population from `docs/parity-matrix.md` + `docs/known-failures.md` (Worker C1). Pending reconciliation by Worker C2. |
| 2026-09-17 | **Merge closure:** WO-P1-001 + WO-P1-002 CLOSED (PR #13 → `d15333e`); §5.3/§5.4/§8.2/J1-J2 updated to reference the merge commit + GUI evidence; operator directive: WO-LAB-001 before WO-P1-003. |
| 2026-09-17 | **WO-LAB-001 executed:** official Linux preview app (26.908.70816) runs in LINUX_GUI_LAB; unauthenticated runtime evidence archived (codex-linux/); §5.3/§5.4/§5.9/§5.10 Linux official cells → runtime-observed (version-skew labeled); §9 overrides 14-16. |
| 2026-09-17 | **WO-P1-003 implemented (PR #15: 1d2abca + clippy 2ea5970):** multi-folder local projects (model, Edit project surface, primary-swap re-key, related-folder file search, storage schema v4); §5.1/§5.2/§8.2/J1/J4 updated; §9 override 17 (Ctrl+P silent-no-op finding); runtime evidence wo-p1-003/. Row flip to complete-on-merge. |
| 2026-09-16 | **RECONCILED (Worker C2):** all 47 rows reconciled from A (`c083c38`) + B2 (`a2343d3`) + historical records per §7; 6 audit-surfaced rows added (§5.1 side chats, §5.4 WebMCP + browser extension, §5.6 in-app editing, §5.8 Record & Replay, §5.10 Activity view); Discoverability/UX-parity cells filled; §8 summary (counts 7/31/6/2/7, P0-P3 lists, J1-J6 journeys, lab-upgrade note); §9 override audit trail (13 entries); work orders confirmed in `FEATURE-PARITY-WORK-ORDERS.md` (WO-P1-001/002/003, WO-P2-004/005/006, WO-PLAT-001, WO-LAB-001). Pending Tech Lead convergence. |
