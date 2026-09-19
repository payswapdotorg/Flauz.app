# RWO-020 — Reference-Behavior Review Rubric (WO-REVIEW-001 deep phase)

> **Deliverable of review worker RWO-020** (one third of WO-REVIEW-001, the
> operator-accepted comprehensive review + test of Flauz.app as a faithful
> Codex clone). This document is the **execution contract for Worker B**
> (verifier) and the **attack map for Worker C** (adversarial reviewer).
> RWO-020 does NOT verify Flauz here — every criterion below is written to
> be executed by B and attacked by C, then re-verified at the integration
> station by the Tech Lead.

- **Base:** `main` @ `d479c7b9e725e1af1353140d49226b3f9a16be9d` (verified:
  commit exists; `research/rwo-020` branches from it)
- **Rubric target:** the reconciled parity matrix in
  `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` (§5, §7–§9)
- **Governing rules:** parity report §4 (definitions), §7 (procedure),
  `docs/research/FEATURE-PARITY-WORK-ORDERS.md` §1 (five closure conditions),
  `AGENTS.md`, `docs/WORK-ORDER-TEMPLATE.md` (closure gates)
- **Constructed:** 2026-09-19 (UTC), from the repo's own reference evidence
  only — every criterion below cites evidence that exists in this repository
  at the base SHA.

---

## 0. Scope, method, and honesty rules

### 0.1 Rows in scope (13)

Per the work order: every parity row claiming a non-missing status that is
either (a) one of the **8 `complete` rows** (PR §8.1) or (b) one of the
**named highest-traffic `partial` rows**. The named-at-minimum list of ten
plus the remaining three `complete` rows gives 13 rows:

| Rubric ID | Parity row (PR §) | Claimed status |
| --- | --- | --- |
| R-01 | Runtime bootstrap (§5.1) | `complete` |
| R-02 | Side chats (§5.1) | `complete` (WO-P2-006) |
| R-03 | Keyboard shortcut reference (§5.10) | `complete` |
| R-04 | Feedback (§5.10) | `complete` |
| R-05 | Multi-root workspace handling (§5.6) | `complete` (regression control) |
| R-06 | Settings shell (§5.9) | `partial` |
| R-07 | Projects and chats (§5.1) | `partial` (multi-folder P1 CLOSED WO-P1-003) |
| R-08 | Keyboard and accessibility (§5.10) | `partial` (WO-P2-004/007 closed) |
| R-09 | Notifications and tray (§5.10) | `partial` |
| R-10 | Activity view & unread attention (§5.10) | `partial` (WO-P2-008 closed state+bindings) |
| R-11 | Git process hygiene (§5.7) | `complete` (regression control) |
| R-12 | Marketplace admin-disabled install (§5.8) | `complete` |
| R-13 | Stable-failure regression controls (§5.10) | `complete` (regression control) |

Rows NOT in scope: `missing`, `platform-limited`, and `deferred` rows
(§4 of this document), and partial rows not named above.

### 0.2 Reference-version markers (binding)

Every criterion carries the version its reference behavior comes from:

| Marker | Meaning | Rule |
| --- | --- | --- |
| `[V-B]` | Historical baseline — Codex Desktop `26.721.3996.0` + bundled CLI `0.146.0-alpha.3.1` | The pinned behavioral spec (PR §2). Historical evidence stays valid. |
| `[V-C]` | Current target — ChatGPT desktop `26.825.51511` | Current parity bar = baseline + installed delta (PR §2 method rule; PR §7.7). |
| `[V-C+]` | Reference+1 — `26.908+` (the Linux preview line the lab ran, `26.908.70816`) | **Out of target.** Recorded as forward-looking notes only; NEVER counts against parity and NEVER upgrades a claim silently (PR §9 override 14; LX README version-skew rule). |

Feature-to-version facts used below (VDN §B; A shortcuts table): multi-folder
local projects 26.715 (in-baseline); in-app Markdown/code editing 26.707
(in-baseline, out of this rubric's rows); Activity view + unread bindings
26.727 (in current target); Settings > Import 2026-08-11 (26.825 line);
WebMCP site tools / extension browsers / event-triggered tasks 2026-08-25
(26.825 line); Pets quick chat + Windows Appshots + Sources panel 26.908
(reference+1).

### 0.3 Evidence classes for verification (cheapest-honest-step taxonomy)

B picks the **cheapest class that can still falsify** the claim, per the
program doctrine (WOL rule 3: GUI verification is first-class; PR §7.6:
source presence is never feature-complete):

| Class | Name | What it is | Relative cost |
| --- | --- | --- | --- |
| `SR` | source-read | Static read of Flauz source at a **named SHA** with `file:line` anchors | cheapest |
| `UT` | unit/integration test | Named focused test(s), `cargo test -p <crate> <name>` | cheap |
| `GUI` | lab scene | LINUX_GUI_LAB scene (Xvfb + picom, isolated HOME/XDG/CODEX_HOME), frames + md5 + VLM-read | expensive |
| `DB` | runtime state | Observed state-DB / filesystem artifact of a real run | medium |

Provenance labels on findings follow PR §4.3 exactly
(`[runtime-observed]` > `[source-derived]` > `[docs-derived]` >
`[historical-record]`; PR §7.3 conflict order). A claim may never be labeled
above its actual evidence class.

### 0.4 Honesty rules for B (binding)

1. **NOT RUN with a reason is acceptable; fabrication is failure.** If the
   lab cannot exercise a step (no codex CLI runtime in lab, no credentials,
   no `gh` binary), record the bound honestly — precedent: WO-P2-008 D11,
   WO-P1-003 native-picker bound, J-journey auth bounds.
2. **Byte-identical frames = no visible change.** The md5-diff method
   (WO-P2-007 D10, B2 ev/25 vs ev/19) is the canonical silent-no-op proof.
   The WO-P2-007 input-quality doctrine: a keypress at a visible surface
   must either do something or say something.
3. **Re-derive every `file:line` anchor at the SHA under test.** Anchors in
   this rubric were verified at `d479c7b`; line numbers drift. The anchor
   semantics (symbol names, string literals, ordering) are the stable part.
4. **Calibration note (real trap, hit during rubric construction):**
   some terminal/transport pipelines strip ANSI-like sequences and render
   `#[must_use]` as `#ust_use]`, making intact source look corrupted. Before
   reporting any "source corruption" defect, confirm bytes with
   `sed -n '<N>p' <file> | od -c`. (At `d479c7b`,
   `crates/codex-core/src/lib.rs` attributes are intact; the apparent
   `#ust_use]` mangle is a display artifact, not a defect.)
5. **Two dispatch layers exist** (SWEEP architecture facts): the GPUI keymap
   (`cx.bind_keys`, menu-accelerator labels — mostly display tokens) and the
   command-id interceptor (`intercept_keystrokes` →
   `handle_keyboard_shortcut_keystroke` → `ACTIVE_KEYBOARD_SHORTCUTS` →
   `execute_keyboard_shortcut_command`). **Runtime behavior is proven only
   through the interceptor path or a live `on_action` handler** — a
   `bind_keys` entry alone proves nothing (WO-P2-007 precedent). Any "it's
   bound, so it works" claim is invalid evidence.
6. **Platform honesty (PR §7.4):** Linux runtime evidence never fills
   Windows/macOS cells. Name the platform slice of every finding.

### 0.5 What would prove / disprove (evidence-class guidance per claim type)

- A claim that a **binding dispatches** → disprovable only by `GUI` scene
  (keypress → visible change or honest status) or by `SR` proof of a live
  handler + registry arm **plus** a UT that exercises the dispatch path.
- A claim that a **data-model invariant holds** (caps, uniqueness, ordering)
  → `UT` (focused test) + `SR` (the invariant in code).
- A claim about **official behavior** → only the repo's own reference layers
  (A matrix, VDN, LX, PM, KF, changelog extract) with labels; never B's
  memory of the official app.
- A claim that a **surface renders** → `GUI` (VLM-read frame) or prior
  runtime evidence at a named SHA; `SR` alone never closes a renders-claim
  (PR §7.6).

---

## 1. Row index (summary)

| ID | Claimed | Version span | Highest-value check | Cheapest class |
| --- | --- | --- | --- | --- |
| R-01 | complete | `[V-B]` | CLI hash pin + override/fallback order intact | SR |
| R-02 | complete | `[V-B]`+`[V-C]` | Side submit does not select the side thread | UT + GUI |
| R-03 | complete | `[V-B]` | Overlay lists only dispatchable bindings, stable category order | SR + GUI |
| R-04 | complete | `[V-B]` | Five category IDs + default-on logs + bounded typed upload | SR + UT |
| R-05 | complete | `[V-B]` | No browser path shim; native paths under multi-root load | SR + UT |
| R-06 | partial | `[V-B]`+`[V-C]`+`[V-C+]` | 15 default-nav rows, 3 groups, search filters all sections | SR + GUI |
| R-07 | partial | `[V-B]`+`[V-C]` | Folders model invariants + primary-only discovery contract | SR + UT + GUI |
| R-08 | partial | `[V-B]`+`[V-C]`+`[V-C+]` | Registry↔active set-equality; palette indexes all nav sections; no dead commands | UT + SR |
| R-09 | partial | `[V-B]` | Banner + tooltip count semantics (runtime-bound parts honestly bounded) | SR + UT |
| R-10 | partial | `[V-C]` | Four bindings resolve visibly; state invariants; honest Activity-view guidance | UT + GUI |
| R-11 | complete | `[V-B]` | 300 ms debounce + coalescing + single-op serialization tests | UT |
| R-12 | complete | `[V-B]` | DISABLED_BY_ADMIN preserved; actions truly blocked; exact copy | SR + UT |
| R-13 | complete | `[V-B]` | Six KF controls: each has code + standing test + budget constant | SR + UT |

---

## 2. Per-row verification criteria

Format per row: the parity claim (exact, cited), the official reference
behavior (exact, cited, version-marked), then a criteria table. Each
criterion: **ID · checkable statement · evidence class that proves/disproves
· cheapest honest step (with anchors at `d479c7b` unless noted)**. "Fail"
means the criterion's checkable statement is false at the SHA under test.

### R-01 — Runtime bootstrap (PR §5.1; claimed `complete`)

**Flauz claim (PR §5.1):** "Exact packaged-CLI hash check, override/fallback
order preserved `[historical-record] PM`; boots to entry surface with
app-server online footer `[runtime-observed] B2 ev/06`." PM line 75: "Keep
the exact packaged CLI hash check and explicit override/fallback order."

**Official reference behavior `[V-B]`:** the baseline is the installed
package (OpenAI.Codex 26.721.3996.0, bundled CLI 0.146.0-alpha.3.1) —
`reference/stable-26.721.3996.0/manifest.json`,
`crates/codex-core/src/lib.rs` `STABLE_REFERENCE` (package name/version,
CLI version, `cli_sha256`, architecture, Owl/Chromium runtime string).
Official startup behavior contrast (LX finding 2, `[V-C+]` runtime-observed
linux-preview): the official app **hard-fails without its bundled runtime**;
Flauz's graceful degradation is an intentional-ahead axis, not a parity
defect (PR §9 override 15).

| ID | Criterion (checkable) | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-01-a | `STABLE_REFERENCE` pins `cli_version = "0.146.0-alpha.3.1"` and `cli_sha256 = "39e9e041ea33ac34aad9578adfe660c5c7a6dc8f82620b77623960f9352a6ef3"` | SR | Read `crates/codex-core/src/lib.rs` ~220–233 @ `d479c7b`; cross-check `reference/stable-26.721.3996.0/manifest.json` |
| R-01-b | Resolution order is exactly: explicit param → `CODEX_RS_CODEX_BIN` → (Windows) hash-pinned stable cache → (Windows) APPDATA npm candidate → bare `codex`/`codex.exe` | SR | Read `resolve_codex_binary` + `windows_stable_codex_cache_candidate`, `crates/codex-platform/src/lib.rs` ~258–310 @ `d479c7b` |
| R-01-c | The stable-cache path accepts a candidate **only on sha256 match**; on mismatch it copies nothing and returns None (no silent fallback to unverified binary) | SR | Same read: `sha256_matches` gates both source and destination (double-check after copy) |
| R-01-d | Boot with no runtime stays up, shows "Resolving…"-class status + auto-retry (no crash, no window death) | GUI | Lab scene equivalent to ev/24 (`evidence/flauz/24-no-runtime-error-state.png`, B2 §10); re-run only if bootstrap code changed since |
| R-01-e | Entry surface renders with the app-server online footer (green dot, "App-server online") once a runtime connects | GUI | ev/06 (`evidence/flauz/06-shell-sidebar-footer-appserver-online.png`); re-run with a real runtime if R-01-b/c code changed |

**Attack notes for C:** (1) any change to the hash constant without a
parity-report/§9 trail; (2) reorder or widening of the fallback chain (e.g.
PATH lookup before cache, or cache accepting hash-mismatch "best effort");
(3) `CODEX_RS_CODEX_BIN`/explicit paths bypassing the hash gate is
**intentional** (explicit override) — do not file as defect, but check the
override is documented in the same doc comment; (4) claiming R-01-d/e from
stale evidence when bootstrap code changed after ev/06/24 were captured.

### R-02 — Side chats (PR §5.1; claimed `complete`, WO-P2-006 CLOSED)

**Flauz claim (PR §5.1):** "Ctrl+Alt+S (Cmd+Alt+S on macOS) opens the side
panel with the main chat still selected; the side composer submits into the
side thread without selecting it; `/side` (availability-guarded, menu row)
reopens the panel; close dismisses it with the main view intact; the main
chat's selection and active turn are untouched."

**Official reference behavior:** `Open side chat` Ctrl/Cmd+Alt+S; temporary
side conversation without interrupting the main chat; `/side` in the current
command set. `[historical-record + docs-derived] A §1` — **confidence
medium-high** (26.707 "side conversations" note + current docs); binding is
baseline-● (A shortcuts table) `[V-B]`; `/side` name is current-docs
`[V-C]`. WO-P2-006 (WOL) records the closure: merge `5287c29f3f` via PR #18,
implementation `764e4db`, evidence `evidence/wo-p2-006/` (D9/D9b VLM-read).

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-02-a | Registry: `openSideChat` with default accelerator Cmd/Ctrl+Alt+S, present in `ACTIVE_KEYBOARD_SHORTCUTS` and `KEYBOARD_SHORTCUT_COMMAND_IDS`; live interceptor arm calls `open_side_chat` | SR | `ui.rs`: registry entry ~2826, action decl ~1670, `bind_keys` ~4698, interceptor arm ~10499, `fn open_side_chat` ~8323 @ `d479c7b` |
| R-02-b | `/side` slash command is availability-guarded and has a menu row | SR | Executor `ui.rs` ~7822 (`if command == "/side"`), menu row ~23993; guard semantics read |
| R-02-c | Side submit creates/feeds the side thread **without selecting it**; main chat selection + active turn untouched | UT | WO-P2-006 state test (delivered with `764e4db`); re-run `cargo test -p codex-core` focused side-chat tests at SHA under test |
| R-02-d | Close dismisses the side panel with the main view intact | GUI | D9/D9b scenes re-run (`evidence/wo-p2-006/d9-*.png`, `vlmd9a/d9b.json`); md5-diff frames |
| R-02-e | Cmd+Alt+S macOS form is the registered macOS variant (not a second binding that collides) | SR | Registry entry's platform-conditional accelerator read |

**Attack notes for C:** (1) the state test passing while the GUI actually
selects the side thread (test-vs-scene drift); (2) `/side` unguarded crash
with no runtime / no selected chat (must give honest guidance per the
WO-P2-007 doctrine, not a silent no-op or panic); (3) close stealing focus
or clearing the main composer draft; (4) side chat interrupting the main
active turn (the exact thing "without interrupting" forbids). Reference
semantics beyond the claim (side-chat persistence, side-chat history) are
`[unverified]` officially — do not invent bars (WO-P2-006 known
limitations).

### R-03 — Keyboard shortcut reference (PR §5.10; claimed `complete`)

**Flauz claim (PR §5.10):** "opens from Help + Ctrl/Cmd+/, stable category
order; overlay + searchable editable settings page both visually confirmed
`[historical-record] PM; runtime-observed ev/11, ev/15]`." PM line 113
("Active keyboard shortcut reference") adds: lists **only bindings active
in codexRS**, stable category order Chat → Navigation → Panels → Project →
Skills → Configure → App → General, platform-aware keycaps, bounded search
with stable no-results copy; `/settings/keyboard-shortcuts` editable
registry with capture/conflict/replace/append/remove/reset and persisted
versioned overrides.

**Official reference behavior `[V-B]`:** "Exact stable `Keyboard shortcuts`
dialog listing only active bindings `[historical-record]`" (PR §5.10,
A §10 Accessibility: editable shortcut registry with stable grouping
comparator; Ctrl/Cmd+/ dialog with search and platform-aware keycaps).
`[V-C+]` contrast: the official Linux preview runs a searchable overlay
("Search shortcuts", 22 rows) — LX captures 05–07 — reference+1, useful as
shape corroboration only, never as the bar.

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-03-a | Ctrl+/ opens the overlay through a **live** path (this is the one `bind_keys` action with a real `on_action` handler per SWEEP) | SR + GUI | Handler for `ShowKeyboardShortcutsShortcut` (`ui.rs` ~4733 binding; `on_action` handler; `fn toggle_keyboard_shortcuts` ~10011 @ `d479c7b`); ev/15 re-run if changed |
| R-03-b | Overlay rows ⊆ dispatchable set: every row's command id is in `ACTIVE_KEYBOARD_SHORTCUTS` **and** has a live interceptor arm (no advertised-dead rows) | SR | Cross-check overlay construction against `execute_keyboard_shortcut_command` arms; SWEEP §B says registry↔active set-equal held at `3c9f113` — re-derive at SHA under test |
| R-03-c | Stable category order Chat → Navigation → Panels → Project → Skills → Configure → App → General with the special chat-command priority | SR | Comparator read (PM line 112–113 contract; WO-P2-006 added the Thread group — check where "Thread" sits vs the stable eight and whether PR records the deviation) |
| R-03-d | Help menu opens the same dialog; `/settings/keyboard-shortcuts` page renders, searchable, editable (capture/replace/append/remove/reset-all with confirmation) | GUI | ev/11 + ev/15; PM records Computer Use exercise of conflict/replacement/persistence — cite as prior evidence, re-run only on change |
| R-03-e | Search is bounded (title/description/id) with stable no-results copy; first Escape clears search, second closes (non-empty query) | UT or GUI | Focused tests delivered with the PM-era work; PM line 113 |

**Attack notes for C:** (1) rows advertised but state-silent — SWEEP
F-A1/F-A2/A3 (archive/pin/rename silently no-op with no selected chat;
overlay advertises unconditionally). The official thread1–9 contract is
"safely do nothing" (F-A5, by design), but archive/pin/rename silence is a
live honesty question: does the official reference no-op silently too, or
guide? Reference is silent (`[unverified]`) — C should probe the **claim
wording** ("listing only active bindings") vs a row whose command
silently no-ops in the entry state; classify as wording-level finding, not
a parity flip. (2) Category-order drift after the Thread group addition.
(3) `MAX_KEYBOARD_SHORTCUT_COMMANDS` vs array length (the 71-vs-72 quirk
was superseded by WO-P2-008's 76=76 — recheck set-equality and constant
parity at the SHA under test).

### R-04 — Feedback (PR §5.10; claimed `complete`)

**Flauz claim (PR §5.10):** "native `Share feedback` dialog with the five
recovered category IDs, validation, default-on logs, no browser-tabs
control; palette entry exists." PM line 103 adds: exact stable `Feedback`
command + `/feedback`; required-details validation; default-on
current-session logs; pinned typed `feedback/upload` through the supervised
app-server; bounded details; selected thread ID when present; app version
tag; redacted provider payloads; exact stable `Feedback uploaded` + retry
copy; Help menu + editable App-group unassigned command route to the same
dialog.

**Official reference behavior `[V-B]`:** "Exact stable `Feedback` command +
`/feedback`; five category IDs; required details; default-on session logs
`[historical-record]`" (PR §5.10).

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-04-a | Exactly five category IDs, stable set | SR | `FeedbackClassification` enum + `ALL`, `crates/codex-core/src/lib.rs` ~4359–4376 @ `d479c7b` (Bug, BadResult, GoodResult, SafetyCheck, Other) |
| R-04-b | All three entries route to one dialog: `/feedback` slash, palette `feedback` command, Help menu | SR | `ui.rs` ~7947 (slash executor), ~10577 (interceptor arm `feedback` → `open_feedback_modal`), Help menu + palette arm ~4155; `fn open_feedback_modal` ~9754 @ `d479c7b` |
| R-04-c | Required-details validation blocks submission; session-logs control defaults ON; no browser-tabs control when no browser surface exists | UT or GUI | Dialog construction read (~41700–41800: `feedback-category-{}` buttons, close button); focused tests or one lab scene |
| R-04-d | Submission uses the pinned typed `feedback/upload`, bounded payload, thread ID only when present, redacted payloads, exact success/retry copy | SR | Submission path read; PM line 103 contract; no live submission possible unauthenticated (honest bound if GUI attempted) |

**Attack notes for C:** (1) categories relabeled or reordered vs the
recovered IDs; (2) logs default OFF (flips the claim); (3) unbounded
details field (violates the bounded-payload doctrine, KF budgets); (4)
"palette entry exists" claimed while the palette row is guarded off in all
states (a dead row — cross-check against R-08's no-dead-commands criterion).

### R-05 — Multi-root workspace handling (PR §5.6; claimed `complete`, regression control)

**Flauz claim (PR §5.6):** "Native `Path`/`PathBuf`, no browser path shim —
Windows multi-root white screen controlled (acceptance test)
`[historical-record] KF]`." KF line 8: failure "Windows multi-root white
screen" (POSIX path handling reached Windows drive paths + missing process
cwd) → control "Native `Path`/`PathBuf`; no browser path shim" → status
Implemented.

**Official reference behavior `[V-B]`:** the official app **fails** here
(public failure report); the reference "behavior" is the documented failure
mode (KF). This row is a **regression control**, not a feature parity
claim. PR §9 override 9: this row and the multi-folder projects gap
(R-07) are **different capabilities — never merge, never substitute**.

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-05-a | Workspace/project paths flow as native `Path`/`PathBuf` end-to-end (no string/URL intermediate in project handling) | SR | Read `LocalProjectSummary` (lib.rs ~914) + workspace open/normalize paths; targeted grep for path→string→path round-trips in project/workspace code |
| R-05-b | Multi-folder project (primary + related on different roots) loads and renders without a blank/white surface | UT + GUI | WO-P1-003 workspace-test battery (delivered with `d06ae3b`; 601 tests referenced in WOL) + evidence `wo-p1-003/01` (multi-folder project listed + auto-selected) |
| R-05-c | The KF control has a standing test anchor (the "acceptance test" of PM §5.6) | SR | Locate the standing test(s) covering non-POSIX/multi-root path load at the SHA under test. **Honest note:** a dedicated white-screen acceptance test was not identified by static search at `d479c7b` (the control is architectural + the WO-P1-003 battery); if B cannot locate one, record "test anchor not found — architectural control only" as a residual, do not fabricate a test name |

**Attack notes for C:** (1) any new `to_string_lossy`/URL-encoding of
project paths reintroducing the shim class; (2) Windows drive-path
edge cases (trailing separators, UNC) — probe via UT only, WINDOWS_GUI_LAB
is unavailable (PR §3 bounds — do not demand Windows runtime evidence);
(3) conflating this row with R-07's multi-folder capability (override 9).

### R-06 — Settings shell (PR §5.9; claimed `partial`)

**Flauz claim (PR §5.9):** "Stable-shaped shell implemented; nav renders 15
rows in 3 groups; SettingsSection enum has 18 sections
(CodeReview/Worktrees/ArchivedChats contextual/hidden) `[source-derived]
ui.rs:2398-2417; runtime-observed ev/07]; settings search filters nav
correctly ("import" → Personal + Import) `[runtime-observed ev/23`;
remaining sections only with working host contracts (ledger bounded)."

**Official reference behavior:** 274 px shell: `Back to app`, bounded search
(Ctrl/Cmd+F), Personal/Integrations/Coding/Archived groups, filtering,
no-results state; full stable registry of 26 sections `[historical-record]
A §9]` `[V-B]`; Settings > Import added 2026-08-11 `[docs-derived]`
`[V-C]`; Linux preview palette lists 10 Settings pages at the login surface
`[runtime-observed: linux-preview 26.908.70816]` `[V-C+]` (LX captures
03–04) — shape corroboration only.

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-06-a | `SettingsSection::DEFAULT_NAV_SECTIONS` = 15; enum total = 18 with exactly CodeReview/Worktrees/ArchivedChats outside the default nav | SR | `ui.rs` ~2494 (enum) + ~2522 (const) @ `d479c7b`; count + diff |
| R-06-b | Nav renders the 15 rows grouped Personal/Integrations/Coding (Archived contextual), stable-shaped 274 px shell with Back-to-app + bounded search | GUI | ev/07 (`07-settings-general.png`) class; re-run if nav code changed |
| R-06-c | In-Settings search filters across **all** sections (query "import" → Personal + Import rows), stable no-results state | GUI | ev/23 (`23-settings-search-filtered.png`) re-run; include a no-match query |
| R-06-d | Every rendered nav row navigates to a working page (no dead nav rows); sections beyond nav exist only with working host contracts | SR + GUI | For each of the 15: palette query resolves + Return navigates (wo-p2-004 captures 03–14 cover six; B spot-checks the rest); `keyboardShortcuts` arm etc. live in the interceptor |
| R-06-e | Palette Settings group indexes every default-nav section (WO-P2-004 closure) — cross-row check with R-08-b | UT | WO-P2-004 coverage test (registry: every default-nav section has a palette entry) at SHA under test |

**Attack notes for C:** (1) hidden sections leaking into the nav or the
palette unconditionally (contextual rows must stay contextual); (2) search
filtering only the currently-visible group; (3) a nav row whose page is a
dead surface in the unauthenticated state without honest empty state (the
"working host contracts" bound is the claim — a row rendering an empty
stub violates it); (4) counting the `[V-C+]` 10-page official palette list
as the bar (the official *shell* registry of 26 is `[V-B]`; the Linux
preview list is reference+1 corroboration).

### R-07 — Projects and chats (PR §5.1; claimed `partial`; multi-folder P1 CLOSED WO-P1-003)

**Flauz claim (PR §5.1):** "Bounded search with stable snippets +
pagination, command menu, archive/delete/rename, codexRS-owned bounded
pinning `[historical-record] PM`; sidebar Chats list + Ctrl+G palette entry
render `[runtime-observed] ev/06, ev/08`; multi-folder model on main
(WO-P1-003, merged `d06ae3b`): `LocalProjectSummary.folders` + Edit project
surface + primary-swap re-key + related folders in file search; single-path
legacy projects load primary-only `[source-derived] core lib.rs:908, 4719;
B2 §6`."

**Official reference behavior:** grouping, bounded full-text chat search,
archive/unarchive/delete, rename, pinning `[historical-record] A §1`
`[V-B]`; **multi-folder local projects in-baseline (26.715)** — Edit
project adds related folders + primary choice; new chats, Git,
AGENTS.md/skills/config.toml discovery use the **primary** folder;
secondary folders for file search/read/edit `[docs-derived] A §6`
`[V-B]` (VDN pre-baseline context); unified pinned threads + shared thread
snapshots (2026-08-20) are cloud-side `[docs-derived] A §9` `[V-C]`,
deferred — out of scope (§4 below).

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-07-a | `LocalProjectSummary.folders` invariants: primary `path` never a member; absolute + unique; capped at `MAX_LOCAL_PROJECT_FOLDERS = 16`; legacy single-path loads primary-only (empty list) | SR + UT | `crates/codex-core/src/lib.rs` ~117 (cap const) + ~914–940 (struct + doc contract) @ `d479c7b`; WO-P1-003 model unit tests (related-folder sets, primary invariants, legacy migration) re-run focused |
| R-07-b | Edit project surface: Add/Remove/SetPrimary with honest guards (cap tooltip, no crash on empty), Primary badge, Done; surface follows a successful swap | GUI | `evidence/wo-p1-003/03–04` (VLM-read: title, badge, Make-primary/Remove affordances, Add folder, surface-follow); re-run on change |
| R-07-c | Primary swap re-keys the registry row (identity + manual order preserved; old primary parks at front of related); new-chat cwd follows the new primary | UT + GUI | WO-P1-003 tests + `wo-p1-003/04–05` (path indicator `…/alpha` → `…/beta`) |
| R-07-d | Related folders join **file search** after the primary; cwd/Git/AGENTS.md/skills/config.toml discovery stay **primary-only** | UT + GUI | WO-P1-003 discovery tests; `wo-p1-003/02` (palette search hits `BETA-NOTES.md` in the related folder) |
| R-07-e | Swapped state persists across close/reopen (storage schema v4: `workspace_folders`, cascade, v3→v4 migration) | UT + DB | WO-P1-003 storage tests; `wo-p1-003/06`; optional state-DB check (`user_version = 4`) |
| R-07-f | Chat search bounded with stable snippets + pagination; archive/delete/rename; bounded pinning; Ctrl+G palette entry renders | SR + GUI | PM line 77 contract; ev/06/08; interceptor arm `searchChats` (Ctrl+G) live-read |

**Attack notes for C:** (1) invariant breaks — primary becoming a member of
its own `folders`, relative/duplicate folders, cap off-by-one (17th
accepted or 16th rejected wrongly); (2) discovery leakage — AGENTS.md or
config.toml discovered from a **secondary** folder (contract: primary-only);
(3) file search hitting related folders but **missing** the primary
(priority inversion); (4) legacy single-path projects migrating
destructively (must load primary-only, never drop data); (5) rename/pin
ordering lost across the re-key. Cloud-side deltas (unified pins, shared
snapshots) are documented deferrals — never count them.

### R-08 — Keyboard and accessibility (PR §5.10; claimed `partial`; WO-P2-004 + WO-P2-007 closed)

**Flauz claim (PR §5.10, condensed):** native command palette with verified
stable registry subset (51-command registry; every default-nav settings
section indexed — WO-P2-004 merged `7aa7163`: six new entries Profile,
Import, Browser, Configuration, Hooks, Git; `SettingsSection::
DEFAULT_NAV_SECTIONS` registry with coverage + filtering tests); Ctrl+/
overlay + editable settings page `[runtime-observed ev/11, ev/15]`; WO-P2-007
merged `a3c0e01`: the direct Ctrl+P keypress routes through the
`searchFiles` interceptor arm, dead `OpenFileSearch` action removed,
5-assertion regression test, D10b lab evidence. Pending: remaining stable
commands, complete focus order, screen-reader labels, reduced-motion
(OS-level following).

**Official reference behavior:** Ctrl/Cmd+K, Shift+P, G, P search-files
drill-in, arrows/Enter/Escape; dynamic Settings group in palette; editable
shortcut registry with stable grouping; complete focus order, screen-reader
labels `[historical-record] A §10]` `[V-B]`; current docs add Clear
terminal (Ctrl+L/Ctrl+K), font-size Ctrl+±/0, Toggle file tree Ctrl+Shift+E
(post-baseline, intro dates `[unverified]`) `[docs-derived]` `[V-C]`;
Linux preview overlay 22 rows incl. Search Files Ctrl+P, Toggle File Tree
Ctrl+Shift+E, Copy deeplink/working directory, Rename chat, Close Tab,
Reload/Force Reload Browser Page `[runtime-observed: linux-preview
26.908.70816]` `[V-C+]` (LX captures 05–07; PR §9 override 16 folds the
runtime-surfaced delta into the P3 gap).

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-08-a | Zero dead commands: `KEYBOARD_SHORTCUT_COMMAND_IDS` ↔ `ACTIVE_KEYBOARD_SHORTCUTS` set-equal, and every id has a live interceptor arm in `execute_keyboard_shortcut_command` | SR + UT | SWEEP §B method re-run at SHA under test: registry `crates/codex-core/src/lib.rs` ~132/137 (76/76 @ `d479c7b`), arms `ui.rs` ~10490+ |
| R-08-b | Palette indexes every default-nav settings section (six WO-P2-004 entries present; Personalization palette title aligned with its nav label) | UT | WO-P2-004 coverage + filtering tests; `PaletteCommand::ALL` count ≥ 51 |
| R-08-c | Ctrl+P is owned by exactly `searchFiles` (no dead `OpenFileSearch` action/binding remains); advertised shortcut Ctrl+P; registry metadata (title "Search files", General group) | UT | `ctrl_p_routes_to_the_search_files_command` (5 assertions, delivered with `a3c0e01`) at SHA under test |
| R-08-d | Palette open paths live: Ctrl+K / Ctrl+Shift+P (openCommandMenu), Ctrl+G (searchChats), arrows/Enter/Escape semantics | GUI | ev/08/09 class; D10b (workspace-seeded Ctrl+P opens "Search files" palette; Escape closes byte-identical) — `evidence/wo-p2-007/d10b/` |
| R-08-e | Known residuals stay honest and documented: F-A4 (Ctrl+P silent with NO workspace — early-return by design), F-A1/A2/A3 (state-silent archive/pin/rename), 24/25 menu-attached display-token actions, Plugins-section palette aliasing | SR | Re-derive SWEEP findings F-A1..A6/F-D1/D2 at the SHA under test; each must still carry its documented classification — a residual silently *widening* (new dead command) is a defect |
| R-08-f | Focus traps on destructive confirmations (Reset all keyboard shortcuts, Reset Memories, Delete archived) confine Tab/Shift+Tab to Cancel/confirm | SR or GUI | SWEEP §A rows for the modal key contexts (WORKS verdicts); PM line 112 |
| R-08-g | In-app reduced-motion control works (On removes Switch/Checkbox transitions + scrollbar idle fade); OS-signal following is the documented pending part — do not conflate | SR | PM Settings row (reduced-motion functional) vs PR §5.10 pending item (OS-level following); verify the distinction is preserved in code + docs |

**Attack notes for C:** (1) any new command id without an arm (dead
command) or any arm without registry membership — re-run the set-equality
both directions; (2) palette entries guarded off in every reachable state
(dead rows); (3) `MAX_KEYBOARD_SHORTCUT_COMMANDS` ≠ array length (the
71-vs-72 class — fully-customized users lose the last override); (4)
claiming Ctrl+P fixed from the UT alone while F-A4 silently regressed into
a *with-workspace* no-op (D10b is the GUI proof — demand the scene, not the
test); (5) counting the `[V-C+]` 22-row overlay inventory (file tree,
deeplink, etc.) as current-target failures — those are the documented P3
shortcut-delta gap (PR §9 override 16), reference+1 corroborated, current
bar is the A shortcuts table `[V-B]`/`[V-C]`.

### R-09 — Notifications and tray (PR §5.10; claimed `partial`)

**Flauz claim (PR §5.10):** "Bounded in-app banner (Open/Dismiss) +
matching window-title/notification-area tooltip; gh-missing toast banners
observed on multiple pages `[runtime-observed ev/03/04]; pending tray
groups, badges, sounds (ledger enhancement); Linux tray/global shortcuts
unavailable (see packaging row)." PM line 104: title + Windows
notification-area tooltip report the same bounded count of non-selected
chats that are running or waiting for approval, returning to `codexRS`
when none remain; Windows quiet-hours-respecting alert; Linux freedesktop
best-effort.

**Official reference behavior `[V-B]`:** in-app completion banner with
Open/Dismiss; title + tray tooltip count of running/awaiting-approval
non-selected chats; Windows quiet-hours toast; Linux freedesktop
best-effort (A §10 Notifications row). Event-triggered scheduled-task
notifications are the 26.825 Scheduled-inbox line `[V-C]` — deferred
(scheduled tasks row), out of scope.

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-09-a | Background completion in a **non-selected** chat raises the bounded banner with Open + Dismiss | SR + UT (GUI runtime-bound) | Banner state path read (`crates/codex-platform/src/desktop_notifications.rs` + core state); completion-flag tests; **honest bound:** full GUI proof needs a runtime with background turns — no codex CLI in lab (WO-P2-008 D11 precedent) — record NOT RUN reason if unavailable |
| R-09-b | Tooltip/title count = non-selected running-or-awaiting-approval chats; resets to `codexRS` when none remain; **selected chat never counted** | SR + UT | Title/tooltip update path read + focused count tests |
| R-09-c | Linux completion alert goes through freedesktop best-effort without blocking the UI; Windows path is quiet-hours-respecting | SR | `desktop_notifications.rs` platform arms read (WINDOWS_GUI_LAB unavailable — SR only for the Windows slice, per PR §7.4) |
| R-09-d | Toast/banner payloads bounded (KF budgets) and gh-missing banners honest (ev/03/04 precedent) | GUI | ev/03 (`03-sidebar-pullrequests-gh-required.png`), ev/04 class — re-run if banner code changed |

**Attack notes for C:** (1) counting the selected chat in the tooltip;
(2) banner raised for a chat the user is currently viewing (the exact
non-selected contract); (3) unbounded notification payload growth (KF
unbounded-logging class); (4) demanding tray groups/badges/sounds as
parity failures — they are the documented ledger-enhancement gap, priority
P2, no WO yet (PR §5.10 Gap cell); (5) demanding Linux tray/global
shortcuts — documented platform bound (Linux packaging row, WO-PLAT-001
class).

### R-10 — Activity view & unread attention (PR §5.10; claimed `partial`; WO-P2-008 CLOSED state+bindings; row flipped `missing` → `partial` 2026-09-19, PR §9 override 18)

**Flauz claim (PR §5.10):** "Unread-attention session state + four registry
bindings on main (WO-P2-008, merged via PR #23 → `00a3392`): bounded
`needs_attention_task_ids` (capped `MAX_VISIBLE_THREADS`, not persisted)
flagged on background turn completion (failed turns included — attention
regardless of outcome) + approval requests in non-selected chats; visit
clears, archive drops; sidebar 6px dot + medium-weight title;
`toggleThreadUnread` Ctrl+Shift+U (round-trip + honest no-selection
status), `nextUnreadChat` Ctrl+Alt+A (cyclic sidebar-order jump, selected
chat never a candidate, honest empty statuses), `clearAllUnread`
Shift+Escape (honest count incl. "No unread chats"), `toggleActivityView`
Ctrl+Alt+U (honest guidance — view surface is a separate future WO);
`MAX_KEYBOARD_SHORTCUT_COMMANDS` 71→76; 7 core state tests + 6 app binding
tests."

**Official reference behavior:** 26.727 Activity view — bell icon /
Ctrl/Cmd+Alt+U shows recently engaged chats needing attention; Shift+Esc
clears unread indicators; Ctrl+Alt+A next chat needing attention; per-chat
mark-unread Ctrl+Shift+U `[historical-record + docs-derived] A §10]`
`[V-C]` (in current target since 26.727 < 26.825; VDN §B). **View shape is
auth-walled `[unverified]`** (R2REF R1.8: the official Linux preview gates
the whole shell pre-auth; no layer describes the view's contents, ordering,
grouping, per-row actions, or visual unread treatment — R2REF open
questions 1–5). Officially evidenced: only the three attention bindings +
adjacent mark-unread + the pre-26.727 tray-count signal.

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-10-a | State: background turn completion — including **failed** turns — and approval requests in **non-selected** chats flag the chat; selected/unknown ids excluded | UT | The 7 codex-core state tests delivered with WO-P2-008 (marking incl. failed turns, selected/unknown exclusions, approval marking) — re-run focused at SHA under test |
| R-10-b | Bound: `needs_attention_task_ids` capped at `MAX_VISIBLE_THREADS` (FIFO drop-oldest); session-only (not persisted) | SR | `crates/codex-core/src/lib.rs` ~4757 (field) + ~8882–8891 (flag + cap) @ `d479c7b`; `MAX_VISIBLE_THREADS = 500` (lib.rs:12) |
| R-10-c | Visit clears; archive drops; manual toggle round-trips; clear-all reports an honest count (incl. "No unread chats") | UT | Same 7-test battery (visit-clears, archive-drops, toggle round-trip, clear-all honest reporting) |
| R-10-d | Bindings: exact-one-owner accelerators Ctrl+Shift+U / Ctrl+Alt+A / Shift+Escape / Ctrl+Alt+U; registry membership + metadata (MAX 76 = array length); reducer resolution; jump semantics (cyclic sidebar order, selected never a candidate) | UT | The 6 codex-app binding tests (CI double matrix is authoritative per WO-P2-008 known limitations — local test-mode gpui compile is OOM-disciplined) |
| R-10-e | All four bindings resolve **visibly** at the entry surface with the verbatim honest statuses | GUI | D11b scene re-run: `evidence/wo-p2-008/d11b/` frames 02–05 + `d11b-md5.txt` (all frames differ) + `vlm-d11b-0*.json` VLM reads ("Select a chat before marking it unread." / "No chats need attention." / Activity-view guidance / "No unread chats") |
| R-10-f | The Activity view *surface* is honestly absent: Ctrl+Alt+U gives guidance, no bell/view rendered, and the absence is documented as a separate future WO (PR §8.2 item 8 rescoped) | SR + GUI | Absence sweep (bell/view symbols) + D11b frame 04 |
| R-10-g | Sidebar dot + medium-weight title render on flagged rows — **documented residual:** no runtime-enabled GUI observation exists (no codex CLI in lab; unit-covered only) | UT (GUI NOT RUN by bound) | Do NOT demand fresh GUI proof without a runtime-enabled lab; verify the unit coverage exists and the residual is recorded (WO-P2-008 documented residuals — same class as 007's F-A4) |

**Attack notes for C:** (1) failed-turn completion silently dropped from
the flag path (the "attention regardless of outcome" contract is unusual —
probe it specifically); (2) selected-chat exclusion broken when the
selection changes mid-turn; (3) jump semantics: selected chat becoming a
candidate, or the cycle not following displayed sidebar order (pinned
first); (4) Shift+Escape clearing without the honest count (silent clear);
(5) **do not** attack the missing view surface, missing persistence, or
missing dot-GUI-proof as defects — each is a documented residual/deferral
with a named bound; (6) **do not** let Flauz's binding set be measured
against invented official semantics (ordering, grouping, per-row actions
are `[unverified]` — R2REF open questions); the current-target bar is the
four bindings + the state contract above.

### R-11 — Git process hygiene (PR §5.7; claimed `complete`, regression control)

**Flauz claim (PR §5.7):** "300 ms debounce, notification coalescing, one
backend Git operation at a time (acceptance test) `[historical-record]
KF]`." KF line 11 + budgets: Git metadata 2 MiB per command; Git files
2,000; branches 500; worktrees 100; review commits 30.

**Official reference behavior `[V-B]`:** the official `git.exe` process
storm is a documented public failure (KF) — reference "behavior" is the
failure mode; Flauz's claim is the control.

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-11-a | Git refresh debounced at 300 ms with coalescing (due only after expiry) | UT | `git_refreshes_are_coalesced_until_the_debounce_expires`, `crates/codex-app/src/backend.rs` ~22107 @ `d479c7b` (asserts None before 399 ms in the fixture — read the exact boundary) |
| R-11-b | One backend Git operation at a time (serialized; no concurrent spawn storm) | SR + UT | Backend git op queue/lock read; focused serialization tests |
| R-11-c | KF Git budgets hold in code: 2 MiB/command metadata cap; 2,000 files; 500 branches; 100 worktrees; 30 review commits | SR | Budget constants read; cross-check `docs/known-failures.md` "Current budgets" table — any drift between doc and code is a finding |

**Attack notes for C:** (1) debounce shortened or bypassed on specific
notification classes (watch coalescing keys); (2) serialization hole under
error paths (a failed op leaving the lock held, or spawning retries); (3)
budget constants drifted from the KF table after `d479c7b`; (4) a new
unbounded git output read (KF unbounded-logging class).

### R-12 — Marketplace admin-disabled install (PR §5.8; claimed `complete`)

**Flauz claim (PR §5.8):** "availability preserved, actions disabled with
recovered `Access is turned off by your admin` tooltip, details show
`Disabled by admin` `[historical-record] PM]`." PM line 93: public
`plugin/list` availability preserves `DISABLED_BY_ADMIN`; catalog actions
stay disabled with the recovered tooltip; details show `Disabled by admin`
instead of collapsing policy into generic unavailability.

**Official reference behavior `[V-B]`:** `DISABLED_BY_ADMIN` availability;
disabled catalog actions + tooltip; `Disabled by admin` details
`[historical-record]` (PR §5.8).

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-12-a | Availability string `DISABLED_BY_ADMIN` preserved end-to-end from `plugin/list` into the UI state | SR | `crates/codex-app/src/backend.rs` ~568 (`availability == Some("DISABLED_BY_ADMIN")`) @ `d479c7b` |
| R-12-b | Exact copy: tooltip `Access is turned off by your admin`; details `Disabled by admin` | SR | `ui.rs` ~31885 (tooltip) + ~30080 (details) @ `d479c7b`; string-literal compare |
| R-12-c | Install/enable actions are **actually blocked**, not merely visually disabled (keyboard/palette paths too) | UT | `plugin_installability(Some("DISABLED_BY_ADMIN"), None)` fixture test, `backend.rs` ~21304; probe any palette/menu path that bypasses the guard |
| R-12-d | Policy is not collapsed into generic unavailability (the row's explicit non-goal) | SR | UI branch read: the disabled-by-admin path renders its specific copy, not the generic empty/error state |

**Attack notes for C:** (1) a dispatchable install path that skips the
availability check (palette command, drag-install, deep link); (2) copy
drift ("disabled by admin" casing/wording vs the recovered strings); (3)
the availability value truncated or mapped at the protocol boundary.

### R-13 — Stable-failure regression controls (PR §5.10; claimed `complete`, regression control)

**Flauz claim (PR §5.10):** "All six controlled as acceptance tests: native
paths; bounded app-server pages + `useStateDbOnly: true`; 300 ms Git
debounce; one supervised tree + Windows Job Object; narrowly scoped owned
logging `[historical-record] KF]`." KF lines 8–13.

**Official reference behavior `[V-B]`:** the six public failure reports
(Windows multi-root white screen; 594 MB JSONL line; ~9 GB startup history
scan; git.exe storm; process-cleanup storm; unbounded logging) — reference
is the failure set; Flauz's claim is the controls.

| ID | Criterion | Class | Cheapest honest step |
| --- | --- | --- | --- |
| R-13-a | Multi-root white screen control | — | Covered by R-05 (do not double-count) |
| R-13-b | Unbounded JSONL: live history queried only through bounded app-server pages; **zero** `read_to_string`-class whole-file reads in `codex-core` live paths | SR | Targeted sweep at SHA under test (at `d479c7b`: `read_to_string` count in `crates/codex-core/src/lib.rs` = 0); check any new whole-read introduced since |
| R-13-c | Startup history scan: `thread/list` paginated and always sets `useStateDbOnly: true` | SR + UT | Serialization test `crates/codex-protocol/src/lib.rs` ~4083 (wire JSON carries `useStateDbOnly: true`) + the list call-site read |
| R-13-d | Git storm control | — | Covered by R-11 (do not double-count) |
| R-13-e | Process cleanup: one supervised tree, graceful cancellation, bounded fallback; Job Object on Windows; **no polling taskkill loops** (AGENTS.md rule) | SR | `crates/codex-platform/src/process.rs` read; targeted grep for polling kill loops |
| R-13-f | Owned logging narrowly scoped: no provider-log duplication, no raw payload logging (AGENTS.md + KF) | SR | Owned-state write paths read; KF budgets table vs constants |

**Attack notes for C:** (1) any new unbounded read/scan/log path introduced
after `d479c7b` (this is a regression-control row — the attack is drift,
not the original control); (2) `useStateDbOnly` dropped from any new
list-adjacent call site; (3) a polling kill loop added for a new platform
case; (4) budgets table in `docs/known-failures.md` out of sync with code
constants (doc-vs-code drift is itself a finding under the
single-source-of-truth doctrine).

---

## 3. Anti-gaming appendix (for Worker C)

1. **Evidence-class inflation** — a `complete` row propped only by SR
   claims where the rubric demands GUI/UT (R-02-d, R-03-a, R-06-c, R-08-d,
   R-10-e). PR §7.6: source presence is never feature-complete.
2. **Stale-evidence laundering** — citing ev/NN or wo-* captures for code
   that changed after the capture SHA without a re-run (each capture names
   its commit; check the delta since).
3. **Silent no-op laundering** — "the binding exists" answers where the
   criterion asks for visible resolution (the WO-P2-007 doctrine; md5-diff
   is the proof standard). Remember the two-dispatch-layer trap (§0.4.5).
4. **Version-marker laundering** — counting `[V-C+]` (26.908+) surfaces as
   current-target failures, or using `[V-C+]` corroboration to upgrade a
   `[V-B]`/`[V-C]` claim's label (PR §9 override 14 forbids silent
   upgrades).
5. **Deferral mislabeling** — filing documented deferrals (tray groups,
   Activity view surface, cloud-side deltas, host-contract-gated sections)
   as defects, or conversely hiding a real defect behind a deferral label.
6. **Bound-inversion** — demanding Windows/macOS runtime evidence
   (WINDOWS_GUI_LAB/MACOS_GUI_LAB unavailable, PR §3) or a mocked runtime
   (no-fabrication doctrine, WO-P2-008 D11) — both are invalid attack
   vectors; the honest NOT RUN + reason is the correct terminal state.
7. **Merge/conflate** — R-05 vs R-07 (override 9), R-03 vs R-08 (dialog vs
   registry/palette), R-13-a/d double-counting R-05/R-11.
8. **Test-vs-behavior drift** — tests passing on fixtures while the real
   binary behaves differently (WO-P2-005 fork-picker precedent: the lab
   found what unit tests missed). Any load-behavior claim ultimately needs
   one lab scene or one live-handler proof, not just a green test name.

## 4. Out of scope — documented deferrals (never count against parity)

Per the work order and PR §8: cloud environments, scheduled tasks
(+ event-triggered), Pets and Codex Micro, Appshots, some Sites
capabilities, Visualizations, Voice — all `deferred` with named blockers
(proprietary cloud backend / pending public protocol). Also out: 26.908+
(reference+1) features; Windows/macOS runtime validation (labs
unavailable); the Plugins catalog-empty-vs-disk-sync watch item (auth
bound, PR §9 override 7); in-row ledger-enhancement residuals enumerated on
their rows (tray groups/badges/sounds, remaining stable commands, focus
order, screen-reader labels, OS-reduced-motion following, richer project
metadata, billing entry points, granular permission editor, renderers,
dynamic CU tools). These are recorded gaps with priorities — they are not
rubric failures and must not be re-litigated as such by B or C.

## 5. Citation index (all verified present at base `d479c7b`)

| Tag | Path |
| --- | --- |
| PR | `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` (§4 definitions, §5.1/5.6/5.7/5.8/5.9/5.10 rows, §7 procedure, §8 counts, §9 overrides) |
| WOL | `docs/research/FEATURE-PARITY-WORK-ORDERS.md` (§1 rules; WO-P1-003, WO-P2-004/005/006/007/008, WO-PLAT-001, WO-LAB-001, WO-R-REF, WO-R-SWEEP) |
| A | `docs/research/CODEX-REFERENCE-MATRIX.md` (§1, §6, §9, §10, §11, shortcuts table, error states) |
| B2 | `docs/research/FLAUZ-REFERENCE-MATRIX.md` (§10, journeys, shortcut inventory) |
| PM | `docs/parity-matrix.md` (lines 75, 77, 93, 101, 103, 104, 112, 113 — historical row text) |
| KF | `docs/known-failures.md` (failure table + current budgets) |
| VDN | `docs/research/evidence/codex-ref/version-delta-notes.md` (§B delta timeline, version-line mapping) |
| LX | `docs/research/evidence/codex-linux/` (WO-LAB-001: README, captures 01–09, vlm-reads.txt, scene scripts, log extracts) |
| R2REF | `docs/research/evidence/p2-batch2-reference/README.md` (R1 Activity view reference; R3 palette/a11y) |
| SWEEP | `docs/research/evidence/wo-p2-007/input-surface-sweep.md` (architecture facts; §A/§B registries; F-findings; base `3c9f113`) |
| ev/NN | `docs/research/evidence/flauz/` — 03, 04, 06, 07, 08, 11, 15, 18, 23, 24 cited |
| wo-p1-003 | `docs/research/evidence/wo-p1-003/` (README, captures 01–06, vlm-reads.txt, scene-d6-v5.sh, seed tool) |
| wo-p2-004 | `docs/research/evidence/wo-p2-004/` (README, captures 01–15, vlm-reads.txt) |
| wo-p2-006 | `docs/research/evidence/wo-p2-006/` (d9-* frames, vlmd9a/d9b.json, scene scripts) |
| wo-p2-007 | `docs/research/evidence/wo-p2-007/` (d10/d10b, ws-*, vlm reads) |
| wo-p2-008 | `docs/research/evidence/wo-p2-008/` (README, d11/d11b frames + md5 + VLM jsons, scene scripts) |
| Source anchors | `crates/codex-core/src/lib.rs`, `crates/codex-app/src/ui.rs`, `crates/codex-app/src/backend.rs`, `crates/codex-platform/src/lib.rs`, `crates/codex-platform/src/desktop_notifications.rs`, `crates/codex-protocol/src/lib.rs`, `reference/stable-26.721.3996.0/manifest.json` — anchors verified at `d479c7b` (see §0.4.3) |

