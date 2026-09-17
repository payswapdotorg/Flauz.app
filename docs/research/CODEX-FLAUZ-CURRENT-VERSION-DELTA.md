# Codex ↔ Flauz Current Version Delta

> **Status: FINAL (post Worker A + C2 reconciliation).** Historical layer
> populated from repo records (C1); current layer (`26.825.51511`) populated
> from Worker A's `docs/research/evidence/codex-ref/version-delta-notes.md`
> (commit `c083c38`) and reconciled into the parity rows by Worker C2.
> Every claim is provenance-labeled; GUI surfacing of runtime-only signals is
> marked `[unverified]` where the desktop changelog is silent.

**Repository:** `payswapdotorg/Flauz.app`
**Owner of the current-layer research:** Worker A (CODEX-REFERENCE-MATRIX
wave); reconciliation into parity rows: Worker C2.
**Feeds:** `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` (§2 Reference
layers, §7.7 Version-skew handling — the affected rows are annotated
in-place there).

---

## 1. Method note — two reference layers

The parity mission compares against **two retained reference layers**:

```text
Historical baseline: Codex Desktop 26.721.3996.0 + CLI 0.146.0-alpha.3.1
        +
Current installed:   ChatGPT desktop 26.825.51511 (operator's machine)
        =
Current parity target
```

- The **historical baseline** is the pinned behavioral specification this
  repo was built against; its evidence (in-repo captures, manifest,
  parity-matrix rows) is retained and remains valid for that version.
- The **current installed reference** (`26.825.51511`) defines where the
  parity bar stands *today*: the current target is the historical baseline
  plus the delta introduced since.
- Historical evidence is never discarded in favor of the current layer; rows
  are annotated when the delta moves the bar (parity report §7.7).

## 2. Historical layer — what `26.721.3996.0` is (from repo records)

The historical baseline is the locally installed official Windows package
**`OpenAI.Codex_26.721.3996.0_x64__2p2nqsd0c76g0`**, fingerprinted in
`reference/stable-26.721.3996.0/manifest.json` (captured 2026-07-24): the
closed-source Electron app (`ChatGPT.exe`, `chrome.dll`,
`resources/app.asar`), the bundled CLI (`resources/codex.exe`), runtime
`owl` / Chromium `150.0.7871.128`, and file SHA-256 hashes.
`reference/README.md` states the directory stores metadata only and that
this manifest "fingerprints the locally installed build used as the
executable specification."

Repo records pinning this baseline:

| Record | Statement | Citation |
| --- | --- | --- |
| README | "The behavior reference is Codex Desktop `26.721.3996.0`, which bundled Codex CLI `0.146.0-alpha.3.1`" | `README.md` (lines 63–64) |
| Architecture | "codexRS uses Codex Desktop `26.721.3996.0` and its bundled Codex CLI `0.146.0-alpha.3.1` as an executable behavioral specification" | `docs/architecture.md` (lines 5–6) |
| Platform support | "The stable compatibility oracle is Windows Codex Desktop `26.721.3996.0` with Codex CLI `0.146.0-alpha.3.1`" | `docs/platform-support.md` (lines 12–13) |
| Agent guidance | "Treat stable `26.721.3996.0` as a behavioral reference, not a runtime" | `AGENTS.md` (line 5) |
| Parity matrix | Reference baseline table: Windows package, desktop bundle build `5828` / internal `26.721.31836`, bundled CLI `0.146.0-alpha.3.1` (+SHA-256), stable app-server schema 89 / experimental 126 client request methods, UI inspection date 2026-07-25 | `docs/parity-matrix.md` §Reference baseline |
| Manifest | Package identity, runtime, per-file hashes | `reference/stable-26.721.3996.0/manifest.json` |
| Code | `stable_reference()` pins `package_version: "26.721.3996.0"`, `cli_version: "0.146.0-alpha.3.1"` (asserted in tests) | `crates/codex-core/src/lib.rs` (~line 223) |
| Historical failure oracle | Stable failure modes + active release-candidate limitations recorded against this baseline | `docs/known-failures.md` |

Historical-layer bound: this baseline is a **Windows package**; its UI
evidence is `[historical-record]` (app.asar inspection 2026-07-25) and the
official runtime is reproducible in this lab only through the official CLI
`0.146.0-alpha.3.1` (`codexrs probe` — see
`docs/research/E2B-PARITY-ENVIRONMENT.md`, branch `parity/lab`).

Runtime-surface note (Worker A, `[runtime-observed]`, 2026-09-16): the
official CLI 0.146.0-alpha.3.1 schema generator emits **89 `ClientRequest`
methods** (stable and v2 bundles identical; 10 `ServerRequest`, 70
`ServerNotification` types), not the 126 the historical record attributes to
the experimental schema — recorded as a measurement difference (extra
methods presumably runtime-negotiated behind `experimentalApi: true`), not a
contradiction (parity report §9 override 8).

## 3. Current layer — ChatGPT desktop `26.825.51511`

**What it is:** the operator's installed reference is the unified "Codex in
ChatGPT" desktop app. The official changelog carries dated version lines
26.707 (2026-07-09), 26.715 (2026-07-23), 26.727 (2026-07-30), 26.908
(2026-09-11); `26.825.51511` has no dated entry of its own — by the weekly
`26.<MMDD>` cadence it maps to the **2026-08-25 release line** ("Browser
extensions, site tools, and cloud sign-in"). **[docs-derived — version-line
mapping is an explicit inference, per Worker A §B]** Its bundle/asar contents
cannot be inspected from this lab (installed on the operator's machine; no
public artifact listing) — any 26.825-only micro-details are `[unverified]`.

### 3.1 Desktop delta timeline, baseline → current (all `[docs-derived]`)

| Date | Entry | Parity-relevant content |
| --- | --- | --- |
| 2026-07-30 | **26.727** | Browser upgrades (address bar revisits history / Google fallback; browsing-history management in Settings; agent history search); Chrome extension open-tab mentions + page text into side chat; multi-repository review for multi-folder projects; generated-image editing; **Activity view** (bell, Ctrl/Cmd+Alt+U); "Record & Replay" skill-from-demo appears adjacent (Computer Use required) |
| 2026-07-29 | Sign in with ChatGPT (beta) | Plugin/partner-site OAuth via ChatGPT identity (Airtable, GitLab, HubSpot, Notion, Supabase, Vercel) |
| 2026-07-31 | Model churn | GPT-5.4 / GPT-5.4-mini retired from Codex (ChatGPT sign-in) 2026-08-31; remain on API-key sessions |
| 2026-08-10 | Daybreak Blue / Red | Two cyber access tiers (GPT-5.6 Sol / GPT-5.6 Cyber); least-privilege profiles + Auto-review |
| 2026-08-11 | **Linux desktop preview + agent imports** | Official ChatGPT desktop app for Linux in preview (`.deb`/`.rpm`; Ubuntu/Debian/Fedora; x64 + ARM64); **Settings > Import**: Claude Code, Claude Cowork, Cursor (instructions, settings, skills, plugins, projects, recent work) with auto-update sync; CLI `/import` |
| 2026-08-13 | **Computer History** | macOS, Pro/Business/Enterprise, opt-in: app/web activity → memories + timeline; EEA/UK/CH added 2026-08-20 |
| 2026-08-17 | Public plugin catalog CSV export | Enterprise owners/admins |
| 2026-08-19 | GitLab support in Codex cloud (beta) | Cloud-side (connect projects, environments, MR reviews) — distinct from local GitHub PR flow |
| 2026-08-20 | Codex and ChatGPT updates | Apple Messages plugin (macOS, approval-gated); Site co-editing + editable URLs; **shared thread snapshots** (read-only local-thread share, secret-pattern redaction, view/revoke under data controls > Shared links); **unified pinned threads** (desktop + iOS) |
| 2026-08-24 | `codex mcp-server` deprecated | Codex-as-MCP-server CLI deprecated (removed 2026-09-05); external MCP via `codex mcp` unaffected |
| 2026-08-25 | **26.825 line — Browser extensions, site tools, cloud sign-in** | Browser extension for **Edge, Brave, Opera, Vivaldi** (+Chrome; Opera without side chat), configured in Settings > Computer Use; **WebMCP site tools** in the built-in browser for ChatGPT Work and Codex (GPT-5.6 Sol/Terra; not Luna; not Enterprise/Edu); cloud browser sign-in; **event-triggered scheduled tasks** (Gmail/Slack/GitHub events; filters; `Run now`; Scheduled inbox) |

Pre-baseline context (already inside `26.721`, kept for clarity): 26.707
(Codex joins the ChatGPT desktop app; in-app Markdown/code editing + inline
annotations; PR Chat; Sites custom domains) and 26.715 (ChatGPT Voice
GPT-Live; macOS Screen context; **multi-folder local projects**). Both are
IN the historical baseline — their Flauz gaps are baseline-parity items, not
version-delta items (see §5).

### 3.2 Slash-command set: baseline 15 → current 24 `[docs-derived]`

- **Baseline (15)** `[historical-record]`: `/chat`, `/compact`, `/feedback`,
  `/fork`, `/goal`, `/init`, `/mcp`, `/memories`, `/model`, `/new`, `/plan`,
  `/project`, `/reasoning`, `/status`, `/shell` (A's enumeration lists 14
  names + "(15)"; `/shell` is evidenced in A §3 and PM "Thread execution" —
  parity report §9 override 2).
- **Current (24)** `[docs-derived]`: `/approve`, `/cloud`,
  `/cloud-environment`, `/compact`, `/fast`, `/feedback`, `/fork`, `/goal`,
  `/ide-context`, `/init`, `/local`, `/mcp`, `/memories`, `/model`, `/pet`,
  `/personality`, `/plan`, `/project`, `/reasoning`, `/review`, `/side`,
  `/status`, `/task`, `/worktree`; skills invoked with `$`; custom prompts
  as `/prompts:`. Per-version introduction dates `[unverified]`.
- **Flauz (16 named + dynamic)** `[source-derived]`: the baseline 15 plus
  `/review` (matches current), plus dynamic `/service-tier:<id>` (functional
  `/fast`) and `/skill:<path>` rows. Missing literal names: 11 (see
  WO-P2-005).

### 3.3 Runtime signals, 0.146 → 0.154 (upstream `openai/codex`, GitHub API) — all `[docs-derived]`; GUI surfacing `[unverified]` unless the desktop changelog names it

| Release | Signal | GUI surfacing at 26.825 |
| --- | --- | --- |
| 0.146.0 | `/new`//`/clear` naming; pin threads; side conversations; Agent Plugins manifests, workspace publishing, Bedrock + Claude Code marketplaces; fork with paginated history; remote Code Code hosts; executor-provided skills | partly baseline; plugin/marketplace surfacing `[unverified]` |
| 0.146.1 | Safer auto-review defaults (cyber models); terminal explains permission changes | `[unverified]` |
| 0.147.0 | Portable plugins; cross-catalog plugin search; persistent conversation sections; incremental transcripts; MCP 2026-07-28 protocol; Cursor skills import; Claude/Cursor conversation sync | `[unverified]` |
| 0.148.0 | `/export` TUI→Markdown; `codex exec fork`; archive/restore from resume picker; credits/cost in `/status`; Bedrock built-in provider; async hooks + hooks→MCP; turns reconnect through provider outages; MCP OAuth reauth without restart | `/status` credits "likely" surfaced `[unverified]` |
| 0.149.0 | `codex agents` dashboard; `/cd` `/pwd` `/cwd`; `codex queue`; doctor diagnoses desktop-app state; SDK `max`/`ultra` efforts | effort enumeration in picker `[unverified]` |
| 0.150.0 | `@` task mentions; `/copy` picker; auto titles for terminal tasks; permission-mode cycling; `Interrupt` hooks | `@` task mentions CLI-first `[unverified]` |
| 0.151.0 | optional-MCP grace period; extensions inspect/replace MCP results; per-repository plugin catalogs; model-aware Ultra fallback | `[unverified]` |
| 0.152.0 | rate-limit banners with actions; credential-refresh progress; MCP server-name charset; per-tool `output_token_limit`; configurable `thread/shellCommand` timeouts (>1 h) | timeout configurability runtime-side `[unverified]` |
| 0.153.0 | plugin CLI remote install; TUI reconnect preserving drafts; thread metadata nullable model/effort; async structured questions; `context_management.experimental_mode` (token-budget context) | experimental mode is flag-gated; GUI `[unverified]` |
| 0.153.1–4 | **GPT-6-Astra** model catalog (picker visibility; Fast tier copy "2x speed, increased usage") | picker: catalog-driven — Flauz's picker already shows GPT-6-Astra `[runtime-observed B2 ev/01]` |
| 0.154.0 | **GPT-6-Astra in model picker**; experimental `/worktree` (browse + resume isolated checkouts); inline questions while Codex continues; **Windows shared background Codex server**; rich-text copy; MCP OAuth refresh coordination; `codex mcp-server` entry removed | `/worktree` appears in the current slash docs (GUI) `[docs-derived]`; browse/resume + daemon GUI `[unverified]` |
| 0.155.0-alpha.* | in flight (not part of the 26.825 reference) | out of scope |

Model churn summary `[docs-derived]`: GPT-5.4/5.4-mini retired from Codex
2026-08-31; **GPT-6-Astra** added (0.153.1→0.154.0); **GPT-5.5 retires
2026-10-14** → GPT-5.6 Sol (`gpt-5.6-sol`) (2026-09-14 notice). Flauz's
model picker is catalog-driven (`config/read`) and already renders
GPT-6-Astra `[runtime-observed]` — churn is a data-side concern, not a code
gap.

## 4. Reference+1 (26.908 and later) — explicitly OUT of the current target

The 2026-09-11 **26.908** line ("Quick chats with Pets and Appshots on
Windows") and the 2026-09-14 GPT-5.5-retirement notice postdate the
installed `26.825.51511`. **These are planning signals only — no parity row
may cite them as the target bar** (Appshots row and Pets row in the parity
report keep them annotated as reference+1):

- Pets quick chat from floating controls (Option+Space / Win+Alt+P; `@`
  context, `$` skills, bell follow-progress; Settings > Pets; Show/Hide pet).
- **Appshots on Windows** (both Alt keys; screenshot + available text;
  customizable shortcut + target chat).
- Sources panel open/download from a conversation; Codex Micro `Insert
  text` key; unfinished comments preserved across chats; dictation Main
  language; browser tab width/scroll stability.
- GPT-5.5 retirement (2026-10-14) → switch to GPT-5.6 Sol.

### 4.1 Linux preview runtime observations (26.908.70816 — WO-LAB-001, 2026-09-17)

The `latest` Linux `.deb` (`chatgpt 26.908.70816`, built 2026-09-14) was run
in LINUX_GUI_LAB (selective userspace; Xvfb + picom; isolated profile; no
credentials). This is runtime evidence of the **reference+1 line**, NOT of
26.825.51511 — observations upgrade Linux official cells only, with the
version-skew label `[runtime-observed: linux-preview 26.908.70816]`; the
26.825 target and every 26.825-labeled claim are unchanged.

- Login surface gates the entire shell: "Sign in to ChatGPT" / "Continue to
  sign in" / "Sign in with an API key" / "Sign up"; no sidebar/composer/
  terminal/browser surfaces pre-auth (evidence codex-linux/01).
- Command palette at login: "Search chats or run a command"; Quick actions
  (New chat Ctrl+N; Open folder Ctrl+O); dynamic **Settings** group with 10
  pages (General, Import, Appearance, Voice, **Pets** — 26.908-only, Git,
  Connections, Environments, Worktrees, Configuration); query "terminal" →
  **Panels: Open terminal (Ctrl+`)**; "Chats" search group (codex-linux/02-04).
- **Keyboard shortcuts overlay (Ctrl+/)**, searchable, 22 rows across
  Chat/Navigation/General incl. bindings absent from Flauz (Search Files
  Ctrl+P; Toggle File Tree Ctrl+Shift+E — runtime-confirming the docs-derived
  binding; Copy deeplink Ctrl+Alt+L; Copy working directory Ctrl+Shift+C;
  Rename chat Ctrl+Alt+R; Close Tab Ctrl+W; browser-page reloads Ctrl+R /
  Ctrl+Shift+R; archive/pin/standalone-chat bindings; Back/Forward Ctrl+[ / ];
  recent-chat cycling; Switch to Work Alt+2) (codex-linux/05-07).
- Auth-pending surface: "Continue signing in with your browser" / "Cancel
  sign-in" / "Browser didn't open?" / "Copy sign-in link".
- Startup hard-depends on the bundled codex runtime (resources/codex,
  app-server stdio); without it the app fatals with NO window — vs Flauz's
  graceful degradation. DB at `$HOME/.codex/sqlite/codex-dev.db`
  (better-sqlite3) with a "Back Up and Rebuild" recovery dialog.
- Payload facts: terminal backend node-pty; @worklouder/device-kit-oai
  (serialport + node-hid) shipped; cua_node (~166 MB) present although the
  official Linux doc states Computer Use is NOT available in the Linux
  preview (payload presence ≠ availability).

## 5. Parity implications (C2 reconciliation)

Which deltas move Flauz work, and which do not:

### 5.1 Deltas that created new Flauz work orders (this wave)

| Delta | Parity-report row | Work order |
| --- | --- | --- |
| Current 24-command slash set (docs) | §5.2 Composer | **WO-P2-005** (11 missing literal names; in-scope subset only) |
| `/side` in the current command set + in-baseline side chats | §5.1 Side chats (added) | **WO-P2-006** |
| 26.715 multi-folder local projects — in-baseline, surfaced by A's timeline work | §5.1 Projects and chats | **WO-P1-003** (baseline item reclassified, not a delta per se) |
| Official Linux desktop preview (2026-08-11) | §3 lab bounds (unchanged) + §8.4 | **WO-LAB-001** (evidence-class upgrade; no product change) |

### 5.2 Deltas recorded as new `missing` rows — no work order yet

- **WebMCP site tools GA (26.825)** — §5.4 row; blocked on fork-runtime
  WebMCP capability; future order when the runtime track delivers.
- **Browser extensions beyond Chrome (26.825)** — §5.4 row; adjacent
  deliverable, out of app-binary scope.
- **Activity view (26.727)** — §5.10 row; unread-state dependent.
- **Record & Replay (26.727-era)** — §5.8 row; Computer-Use-dependent, Linux
  slice platform-bound.
- **Browser address-bar history / Google fallback (26.727)** — annotated on
  the §5.4 In-app browser row (P2).
- **Multi-repository review (26.727)** — annotated on the §5.7 Repository
  status row; blocked-by WO-P1-003.
- **Generated-image editing (26.727)** — annotated on the §5.6 Artifacts row
  (P2).
- **In-app Markdown/code editing (26.707, in-baseline)** — §5.6 row; needs
  Tech Lead scoping (baseline item surfaced by the audit, not a delta).

### 5.3 Deltas that do NOT create Flauz work (out of phase scope or already covered)

- **Settings > Import (2026-08-11)** — Flauz already implements the Import
  route with the same three providers (Claude Code / Claude Cowork / Cursor)
  `[source-derived + runtime-observed nav]`; it was ahead of the historical
  baseline. Remaining bound (unsupported-project reporting) is a public-
  protocol item, unchanged. No new work.
- **Model churn (GPT-6-Astra, GPT-5.6 Sol)** — catalog-driven picker; data
  flows through `config/read`. No code work; watch the catalog contract.
- **Event-triggered scheduled tasks (26.825)** — same proprietary cloud
  backend as baseline scheduled tasks; row stays `deferred`.
- **Shared thread snapshots + unified pins (2026-08-20)** — cloud surfaces
  (ChatGPT data controls / cross-device sync); deferred, proprietary.
- **Computer History (2026-08-13)** — macOS + cloud memories; deferred.
- **Sign in with ChatGPT for plugins (2026-07-29)**, **plugin catalog CSV
  export (2026-08-17)**, **GitLab cloud (2026-08-19)**, **Apple Messages
  (2026-08-20)** — cloud/enterprise/macOS surfaces; out of Flauz's
  local-desktop scope; no rows moved.
- **`codex mcp-server` deprecation/removal (2026-08-24/09-05)** — CLI
  surface; Flauz never shipped it; no impact.
- **Runtime 0.146→0.154 signals** (agents dashboard, queue, `/export`,
  `/copy`, Vim modes, `@` task mentions, worktree browse/resume, Windows
  shared daemon, MCP 2026-07-28 protocol, OAuth refresh coordination,
  rate-limit banners, context-management experimental mode) — all
  `[docs-derived]` runtime-side with GUI surfacing `[unverified]`; they
  belong to the fork-runtime compatibility track
  (`docs/codex-universal/RUNTIME-COMPATIBILITY.md`), not the GUI parity
  wave. Revisit per-signal when the runtime track moves or the official
  desktop changelog names the GUI surface.
- **Reference+1 (26.908) items** — out of the current target by definition
  (§4).

### 5.4 Version-skew decisions applied to the parity rows (per §7.7)

- Slash commands: the bar moved from the baseline 15 to the current 24 —
  Composer row annotated; baseline coverage is complete (15/15 + `/review`
  extra), so the historical "remaining slash commands (polish)" note is
  closed and the current delta is the live gap (WO-P2-005).
- Browser: the bar now includes address-bar history/Google fallback (26.727)
  and site tools (26.825) — row annotated; WO-P1-002 remains scoped to
  discoverability only.
- Scheduled tasks: event triggers folded into the existing deferred row.
- Appshots/Pets: 26.908 arrivals annotated as reference+1 and do NOT move
  the deferred status.
- Import: current target confirms Flauz's ahead-of-baseline implementation;
  row stays partial on the public-protocol bound only.

## 6. Change log

| Date | Change |
| --- | --- |
| 2026-09-16 | Skeleton created; historical layer populated from repo records; current layer marked PENDING Worker A (Worker C1). |
| 2026-09-16 | **FINAL (Worker C2):** current layer (`26.825.51511`) populated from A's `version-delta-notes.md` — version-line mapping (2026-08-25 line, explicit inference), desktop delta timeline, slash 15→24 with Flauz's 16+dynamic inventory, runtime 0.146→0.154 signal table (GUI surfacing `[unverified]` where changelog-silent), model churn, 89-vs-126 measurement note; reference+1 (26.908) kept explicitly out of the target; §5 parity implications split into deltas that created work orders (WO-P2-005/006, WO-P1-003 baseline-reclass, WO-LAB-001), new missing rows without orders, non-work deltas, and the per-row version-skew decisions. |
