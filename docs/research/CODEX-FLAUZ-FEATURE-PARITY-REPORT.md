# Codex ↔ Flauz Feature Parity Report

> **STATUS: PRE-A/B BASELINE — pending reconciliation by Worker C2.**
> This document is the canonical parity report SKELETON. The matrix rows below
> are pre-populated from historical repo records only
> (`docs/parity-matrix.md`, `docs/known-failures.md`). They have **not** yet
> been reconciled against `CODEX-REFERENCE-MATRIX.md` (Worker A) or
> `FLAUZ-REFERENCE-MATRIX.md` (Worker B). Every pre-populated row is marked
> `PRE-A/B` in its Evidence column. Worker C2 reconciles; nobody else edits
> statuses.

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
  Windows/macOS only; it **cannot run on Linux anywhere**. Official-side
  (Linux-A) evidence therefore consists of reference **layers** with
  provenance labels, not live runs — except official *runtime* behavior that
  is reproducible in this lab (official CLI `0.146.0-alpha.3.1`,
  `codexrs probe`/`info`).
- Windows-pair and macOS-pair validation is **deferred** until those
  environments exist. Windows/macOS cells in this matrix may only carry
  `[historical-record]`, `[docs-derived]`, or `[source-derived]` evidence.
- No silent platform substitution: a Linux observation never fills a Windows
  cell, and vice versa (see §7.4).

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
| `Flauz` | Current behavior (from Worker B's matrix / historical record), with provenance label. |
| `Platform` | Platform slices this row's evidence covers (`win`, `linux`, `macos`). Split rows per platform when statuses differ — never merge silently. |
| `Backend` | Status of the backend/protocol/engine layer. |
| `UI` | Status of the UI surface layer. |
| `Discoverability` | Status of natural discoverability (persistent affordances, menus, labels — not just shortcuts/palette). |
| `UX parity` | Status vs the "behaves materially differently" bar. |
| `Functional parity` | Overall end-to-end functional status (the roll-up cell). |
| `Recovery` | Recovery/verification path: how a user or the lab recovers or re-verifies this capability (reconnect timers, retry, fallback, lab scene). `—` if none recorded. |
| `Evidence` | `PRE-A/B` marker + provenance label + source citation. After C2 reconciliation: A/B matrix row references. |
| `Gap` | Gap type (§4.1) + priority class (§4.4), or `none recorded`. Work-order ID once one exists. |

Cell conventions:

- `—` = not derivable from the current evidence layer; **C2 fills it**.
- A cell status is one of §4.2 only (plus `—` pre-reconciliation).
- Pre-populated mapping from the historical ledger (for C2 to revisit):
  `done → complete`, `partial → partial`, `missing → missing`,
  `platform → platform-limited`, ledger verdict `bounded` (proprietary
  backend / pending public protocol) → status `deferred`; ledger delta class
  `polish → P3`, `enhancement → P2`.

---

## 5. Canonical parity matrix (PRE-A/B baseline)

Citations: **PM** = `docs/parity-matrix.md`, **KF** = `docs/known-failures.md`.
All rows are `[historical-record]` unless stated otherwise. **PM ledger**
refers to PM §"Release-critical parity ledger (GUI-002)".

### 5.1 Agent / task lifecycle

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Runtime bootstrap | Packaged-CLI hash pin + explicit override/fallback order | Implemented: exact packaged CLI hash check, override/fallback order preserved | win, linux | complete | complete | — | — | complete | fallback order on CLI mismatch | PRE-A/B · [historical-record] PM "Runtime bootstrap" | none recorded |
| App-server supervision | Stable startup capabilities declaration, legacy-notification opt-out, reconnect, loaded/background session recovery | Bounded `thread/loaded/list` hydration via `thread/read`, prioritized + resumed after connect; deduplicated 1/2/4/8/16/20 s reconnect timer; retryable startup failure; pending: remaining stable methods, network-aware transport diagnostics | win, linux | partial | partial | — | — | partial | reconnect timer + manual retry; resume after reconnect | PRE-A/B · [historical-record] PM "App-server supervision" | backend — P2 |
| Projects and chats | Grouping, bounded full-text chat search, unified command menu, archive/unarchive/delete, rename, pinning | Bounded search with stable snippets + pagination, command menu, archive/delete/rename, codexRS-owned bounded pinning; pending: multi-root sources, unread state, richer metadata (PM ledger: polish) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Projects and chats" | ux — P3 |
| Thread execution | Start, steer, interrupt, reconnect, active-turn restoration, edit latest message, manual compaction | Typed stable methods incl. `thread/rollback` + replacement `turn/start`; pending: compaction provenance, richer edit metadata (ledger: polish) | win, linux | partial | partial | — | — | partial | selected active-turn restoration; reconnect | PRE-A/B · [historical-record] PM "Thread execution" | ux — P3 |
| Approvals and user input | Stable public `item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `item/permissions/requestApproval` | Native command/file/permission/dynamic-tool approvals with bounded typed payloads; pending: connector-specific methods where public contracts expose them (ledger: bounded) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Approvals and user input" | backend — deferred (public protocol) |
| Streaming timeline | Compact `Explored`/`Ran`, file-change, web-search, subagent, image-inspection/generation, background-terminal summaries | Bottom-aligned variable-height native list with stable-style summaries; pending: activity grouping, citation navigation, source aggregation (ledger: polish) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Streaming timeline" | ux — P3 |
| Scheduled tasks | Suggestions, manual/chat-assisted creation, schedule editor, run history, unread/archive states, plugin templates, pause/edit/delete, notifications | Not implemented — cloud tasks backend proprietary (ledger: bounded) | win, linux | deferred | deferred | — | — | deferred | — | PRE-A/B · [historical-record] PM "Scheduled tasks" | backend (proprietary) — deferred |

### 5.2 Composer

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Composer (input surface) | File/folder/image attachments; Plan, Goal, Send, Steer, Stop states | All listed states work; Goal mode in new + existing chats incl. file-backed Goal attachments via public starter-turn path; pending: remaining slash commands (polish), cloud projects/voice (proprietary) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Composer" | ux — P3 |
| Model, effort, and speed picker | Catalog-backed model, reasoning-effort, Standard/Fast/Ultrafast service-tier selection | Implemented with `config/read` + managed `configRequirements/read` defaults, profile-aware `config/batchWrite` (ledger: none listed) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Model, effort, and speed picker" | none recorded (P3 residual) |
| Permission profiles | Stable Ask for approval / Auto-review / Full access / Read only presets + custom profiles | Catalog-backed built-in/custom selection matching presets with catalog descriptions in native picker; pending: granular/custom editor, sandbox detail, per-project resolution (ledger: enhancement) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Permission profiles" | ux — P2 |
| Voice input | Dictation, realtime voice, handoff target, stage layout, settings, accessibility states | Not implemented — proprietary realtime stack (ledger: bounded) | win, linux | deferred | deferred | — | — | deferred | — | PRE-A/B · [historical-record] PM "Voice" | backend (proprietary) — deferred |

### 5.3 Terminal

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Terminal (sessions) | Per-chat terminal tabs as a first-class surface | Bounded per-chat PTY/ConPTY tabs in bottom or right dock; new-tab, selection, close-one, stop, restart, bounded input/output, truncation; `Ctrl+\`` binding at `crates/codex-app/src/ui.rs:4499`, dock toggle action at `crates/codex-core/src/lib.rs:4923` (ledger: green) | win, linux | partial | partial | audit pending — reachable only via `Ctrl+\``, palette, dock/menu; no persistent primary affordance (WO-P1-DRAFT-001) | — | partial | stop/restart per session | PRE-A/B · [historical-record] PM "Terminal"; [source-derived] ui.rs/lib.rs | ui-integration? — P1 DRAFT (pending C2) |
| Process Manager | Stable-shaped manager for the selected chat | `Ctrl+Alt+M`, command palette, and completed background-terminal activity open the native manager; bounded `thread/backgroundTerminals/list`, per-process `terminate`, chat-wide stop (ledger: green) | win, linux | partial | partial | — (three affordances recorded) | — | partial | — | PRE-A/B · [historical-record] PM "Process Manager" | none recorded |

### 5.4 Browser

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| In-app browser | Browser as a normally discoverable surface | Native GPUI panel: bounded tabs, URL/navigation state, JPEG frame streaming, keyboard/pointer/scrolling vs supervised isolated Edge/Chrome/Chromium profile; same runtime exposes stable `codex-browser` (ledger: green); reachable via `InspectorPane::Browser` (`crates/codex-core/src/lib.rs:259–267`) + palette/contextual paths only | win, linux | partial | partial | audit pending — hidden in inspector pane set; no persistent primary affordance (WO-P1-DRAFT-002) | — | partial | — | PRE-A/B · [historical-record] PM "In-app browser"; [source-derived] lib.rs | ui-integration? — P1 DRAFT (pending C2) |
| Browser permissions & settings | General Approval, site-specific rules, Full CDP switch; origin/download/upload/raw-CDP elicitation cards; persistent vs session grants | Stable-shaped Browser settings incl. bounded site Browse/Download/Upload/Debug rules + Full CDP switch, single-writer storage, hot-apply; exact typed MCP elicitations intercepted; `Allow once`/`Allow for this site`/`Allow for all sites` + file-transfer cards; `_meta.persist` always/session handoff; existing rules resolve prompts without UI | win, linux | partial | partial | — | — | partial | existing Block/Allow rules + global modes resolve later prompts without UI | PRE-A/B · [historical-record] PM §"In-app Browser permission status" | none recorded |

### 5.5 Computer use

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Computer Use — Windows | Any App control: case-preserved AUMIDs, known-folder GUID IDs, absolute executable paths | Native Any App with oversized-ID/shared-host rejection, self-control exclusion, ask-before-every-app's-first-read/control, managed approvals (ledger: green Windows) | win | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Computer Use"; PM ledger | none recorded (Windows) |
| Computer Use — Linux | (Same Any App surface on the reference platform matrix) | Screenshot-only, bounded X11/XWayland observation when `DISPLAY` is set; no pure Wayland, text extraction, input, app launch, persistent approvals, overlays, or interruption monitoring; portal-backed selection path = future work (WO-PLAT-DRAFT-001) | linux | platform-limited | platform-limited | — | — | platform-limited | future: portal-backed selection path | PRE-A/B · [historical-record] KF "Active release-candidate limitations"; PM ledger | platform — WO-PLAT-DRAFT-001 |
| Dynamic Computer Use tools | Tools available across a task's lifetime | Attached at `thread/start` only; an existing task cannot gain them after creation | win, linux | partial | partial | — | difference recorded | partial | — | PRE-A/B · [historical-record] KF "Active release-candidate limitations" | ux — P2 |

### 5.6 Files / workspace

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Artifacts and outputs | Stable discovery for completed file changes, `::output` citations, generated images | Bounded native Outputs inspector: latest-first dedupe, directives hidden, exact empty state; pending: PDF/Office renderers, AVIF/GIF, canvas actions (ledger: enhancement) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Artifacts and files" | ux — P2 |
| Multi-root workspace handling | No multi-root white screen (public failure report documents the official failure) | Native `Path`/`PathBuf`, no browser path shim — Windows multi-root white screen controlled (acceptance test) | win | complete | complete | — | — | complete | — | PRE-A/B · [historical-record] KF failure table | none (regression control) |
| Sites | Create, preview, annotate, version, share, publish, return-to-chat | Not implemented — OpenAI cloud surface (ledger: bounded) | win, linux | deferred | deferred | — | — | deferred | — | PRE-A/B · [historical-record] PM "Sites" | backend (proprietary) — deferred |
| Visualizations | Native chart/report rendering, source inspection, interaction, export, editor handoff | Not implemented — OpenAI cloud surface (ledger: bounded) | win, linux | deferred | deferred | — | — | deferred | — | PRE-A/B · [historical-record] PM "Visualizations" | backend (proprietary) — deferred |

### 5.7 Git

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Repository status | Current branch, ahead/behind, bounded changed files, local branches, worktrees, ≤30 unique commits | Native repository snapshot incl. stable-shaped wide `Uncommitted changes` surface (ledger: green) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Repository status" | none recorded beyond row text |
| Diff review | Unstaged/Staged bounded query, real old/new line numbers, hunk headers, added/removed colors | Implemented with source-switch invalidation; multi-file Changes baseline (grouped patches, expanded-by-default, keyboard-focusable folds, Unified/Split across Last Turn/Uncommitted/Committed/Branch); open: syntax highlighting, inline comments, per-hunk actions, guarded revert | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Diff review" + §"Multi-file diff baseline" | ux — P3 |
| Branches and worktrees | Local branches, stable create-and-checkout dialog, worktree as explicit chat workspace | Implemented; official `thread/start` receives the worktree (ledger: green) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Branches and worktrees" | none recorded beyond row text |
| Pull requests | 420 px Create PR flow: title, generated description, preflight | Stable-shaped flow incl. `gh --version`, authenticated-status fallback, existing open-PR lookup; pending: branch prefix, force push, draft/merge-method options (ledger: polish) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Pull requests" | ux — P3 |
| Git process hygiene | No `git.exe` process storm (public failure report documents the official failure) | 300 ms debounce, notification coalescing, one backend Git operation at a time (acceptance test) | win | complete | complete | — | — | complete | — | PRE-A/B · [historical-record] KF failure table | none (regression control) |

### 5.8 MCP / Apps / Skills / Plugins

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Plugins marketplace | `plugin/list`, `plugin/read`, install/uninstall, search, refresh, enable/disable; OpenAI/Shared/Created/Workspace/Local directories | End-to-end native with exact marketplace kinds; pending: OAuth/no-auth callback completion (awaits official protocol — ledger: bounded) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Plugins marketplace" | backend — deferred (public protocol) |
| Marketplace admin-disabled install | `DISABLED_BY_ADMIN` availability; disabled catalog actions + tooltip; `Disabled by admin` details | Implemented: availability preserved, actions disabled with recovered `Access is turned off by your admin` tooltip, details show `Disabled by admin` | win, linux | complete | complete | — | — | complete | — | PRE-A/B · [historical-record] PM "Marketplace admin-disabled install" | none recorded |
| Skills | `skills/list` discovery, browse + Manage search, scope/path metadata, partial scan errors, refresh/invalidation, `skills/config/write` enable/disable | Implemented against selected project context; Manage loads installed Skills; pending: recommended/install flows, creation surfaces (ledger: enhancement) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Skills" | ux — P2 |
| MCP and apps | Manage Apps / Manage MCPs: bounded list/search, App tool inspection (`app/read`), App `Try now`, enablement, external management, browser install/connect fallback, full MCP tool/schema/resource detail | Implemented for the listed surface; pending: App OAuth completion (awaits official protocol — ledger: bounded) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "MCP and apps" | backend — deferred (public protocol) |

### 5.9 Settings / account

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Settings shell | Stable-shaped 274 px Settings shell: `Back to app`, bounded search (Ctrl/Cmd+F), Personal/Integrations/Coding/Archived groups, filtering, no-results state | Implemented stable-shaped; remaining sections only with working host contracts (ledger: bounded) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Settings" | backend — deferred (host contracts) |
| Account and usage | `account/read`, login start/cancel, logout, rate limits, usage | Separate stable-shaped settings surfaces over one typed official surface; pending: billing entry points (ledger: enhancement) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Account and usage" | ux — P2 |
| Personalization and memory | Pinned renderer `Friendly`/`Pragmatic` tone contract | Native Personalization route follows the contract and fixes the renderer's known `None` omission using the pinned app-server's `none`/`friendly`/`pragmatic` values | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Personalization and memory" | none recorded |
| Import and migration | `externalAgentConfig/detect`, `import`, `import/readHistories` for Claude Code, Claude Coworker, … | Native Personal → Import route implemented; pending: unsupported-project reporting (awaits public protocol — ledger: bounded) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Import and migration" | backend — deferred (public protocol) |
| Remote control and SSH | `remoteControl/status/read`, enable/disable, pairing, paginated client discovery, revoke; keep-awake, SSH profiles, remote chats | Native Connections UI implemented for the public methods; pending: SSH profiles/remote chats (await public contracts — ledger: bounded) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Remote control and SSH" | backend — deferred (public contracts) |
| Cloud environments | List/detail/create links, repository/machine metadata, target selection, connection state, cloud execution | Not implemented — OpenAI cloud backend (ledger: bounded) | win, linux | deferred | deferred | — | — | deferred | — | PRE-A/B · [historical-record] PM "Cloud environments" | backend (proprietary) — deferred |

### 5.10 Product shell

| Capability | Official Codex | Flauz | Platform | Backend | UI | Discoverability | UX parity | Functional parity | Recovery | Evidence | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| First run and updates | About window: package version, copyright, focus behavior | Help → `About codexRS` native 380×360 floating window, centered, refocuses existing instance, OK/Escape close; pending: fuller welcome, dependency diagnostics, update prompt (ledger: enhancement) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "First run and updates" | ux — P2 |
| Keyboard and accessibility | Ctrl/Cmd+K, Shift+P, G, P search-files drill-in, arrows/Enter/Escape; complete focus order, screen-reader labels | Native command menu with verified stable registry subset; pending: remaining stable commands, complete focus order, screen-reader labels, reduced-motion (ledger: enhancement) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Keyboard and accessibility" + §"Keyboard accessibility baseline" | ux — P2 |
| Keyboard shortcut reference | Exact stable `Keyboard shortcuts` dialog listing only active bindings | Implemented: opens from Help + Ctrl/Cmd+/, stable category order | win, linux | complete | complete | — | — | complete | — | PRE-A/B · [historical-record] PM "Active keyboard shortcut reference" | none recorded |
| Notifications and tray | Background completion notification; tray status | Bounded in-app banner (Open/Dismiss) + matching window-title/notification-area tooltip; pending: tray groups, badges, sounds (ledger: enhancement); Linux tray/global shortcuts unavailable (see packaging row) | win, linux | partial | partial | — | — | partial | — | PRE-A/B · [historical-record] PM "Notifications and tray" | ux — P2 |
| Feedback | Exact stable `Feedback` command + `/feedback`; five category IDs; required details; default-on session logs | Implemented as a native `Share feedback` dialog with the five recovered category IDs, validation, default-on logs, no browser-tabs control | win, linux | complete | complete | — | — | complete | — | PRE-A/B · [historical-record] PM "Feedback" | none recorded |
| Appshots | Foreground-window capture, destination policy, hotkey, sound, preview, text/offscreen context | Not implemented — proprietary capture flow (ledger: bounded) | win, linux | deferred | deferred | — | — | deferred | — | PRE-A/B · [historical-record] PM "Appshots" | backend (proprietary) — deferred |
| Pets and Codex Micro | Overlay lifecycle, device integration, commands, settings, mini-game/composer, sound, accessibility | Not implemented — proprietary companion surface (ledger: bounded) | win, linux | deferred | deferred | — | — | deferred | — | PRE-A/B · [historical-record] PM "Pets and Codex Micro" | backend (proprietary) — deferred |
| Windows packaging | Signed MSIX installer + desktop integration + updating | Unsigned portable ZIP release strategy (code signing unavailable to this program); SHA-256 checksums + documented verify step; no MSI/installer/updater | win | partial | partial | — | difference documented | partial | verify via published SHA-256 | PRE-A/B · [historical-record] PM ledger "Windows"; KF limitations | intentional difference — documented (GUI-006) |
| Linux packaging | (Reference ships Windows/macOS only — no official Linux target exists) | Unsigned portable tar.gz; explicit `codexrs --install-desktop-entry` per-user integration (never overwrites); no system package, Wayland portals, tray, or global shortcuts — documented in `docs/platform-support.md` | linux | platform-limited | platform-limited | — | — | platform-limited | documented `--install-desktop-entry` path | PRE-A/B · [historical-record] PM ledger "Linux"; KF limitations; docs/platform-support.md | platform — documented |
| Stable-failure regression controls | Public failure reports: Windows multi-root white screen; 594 MB JSONL line; ~9 GB startup history scan; `git.exe` storm; process-cleanup storm; unbounded logging | All six controlled as acceptance tests: native paths; bounded app-server pages + `useStateDbOnly: true`; 300 ms Git debounce; one supervised tree + Windows Job Object; narrowly scoped owned logging | win, linux | complete | complete | — | — | complete | controls are standing acceptance tests | PRE-A/B · [historical-record] KF failure table | none (regression controls) |

---

## 6. Pre-population method (audit trail)

- Source records: `docs/parity-matrix.md` (§Product parity table, §Installed
  Desktop route inventory, §Settings inventory, §Protocol progress,
  §Release-critical parity ledger) and `docs/known-failures.md` (failure
  table + active release-candidate limitations). All are `[historical-record]`
  evidence pinned to the 26.721.3996.0 baseline.
- One canonical row per historical capability row, regrouped into the ten
  operator feature sections. Rows that the historical record splits by
  platform (Computer Use, packaging) are split here too.
- Historical status mapping: §4.5. `—` marks cells the historical layer
  cannot honestly fill (Backend/UI decomposition, Discoverability, UX parity
  per-row) — these are exactly what Workers A/B matrices and C2
  reconciliation exist to fill.
- Known gaps in the pre-populated data: the historical matrix does not
  decompose discoverability; the route inventory (PM §"Installed Desktop
  route inventory") and settings-section inventory (PM §"Settings inventory")
  are recorded as inventories only and are not yet row-matched — C2 uses them
  as cross-checks (a Flauz surface missing for an inventoried official route
  or settings section is a candidate UI-integration/backend gap).
- The `26.825.51511` delta is NOT reflected in these rows yet (see
  `docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md`, pending Worker A).

## 7. Reconciliation procedure (for C2)

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
runtime-observed > source-derived > docs-derived > historical-record
```

- A higher-order label **overrides** a lower-order label for the same cell;
  the losing value is recorded in the Evidence cell (e.g.,
  "`[source-derived]` overrides `[historical-record]`") so the audit trail
  survives.
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
  WO-PLAT-DRAFT-001 keeps it a platform gap, not a missing feature).

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

## 8. Change log

| Date | Change |
| --- | --- |
| 2026-09-16 | Skeleton + definitions + PRE-A/B pre-population from `docs/parity-matrix.md` + `docs/known-failures.md` (Worker C1). Pending reconciliation by Worker C2. |
