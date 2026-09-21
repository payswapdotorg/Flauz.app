# Linux GUI — ShareNet UX discipline rubric

**Lane:** Linux GUI usability (second Tech Lead — Linux lane).
**Status:** BINDING DESIGN DIRECTION — dispatched 2026-09-21 (LAB-004, Worker B).
**Base:** `docs/linux-gui-lane-baseline` @ `afeb0d5`.
**Contracts:** [PRODUCT-UX-JOURNEYS.md](../PRODUCT-UX-JOURNEYS.md) §1, §2.1,
§6, §7; [FLAUZ-SOURCE-OF-TRUTH.md](../FLAUZ-SOURCE-OF-TRUTH.md) — "GUI
discoverability is part of product correctness".
**Reference:** `pectoraux/ShareNet` — read-only design inspiration. Mechanisms
and principles are transferred; no code, copy, tokens, or assets are copied.
**Evidence base:** run-4 signed-out E2B battery —
[LINUX-GUI-USER-JOURNEY-MATRIX.md](LINUX-GUI-USER-JOURNEY-MATRIX.md) run log,
[LINUX-GUI-REGRESSION-LEDGER.md](LINUX-GUI-REGRESSION-LEDGER.md) L-006…L-012
and J-17, screenshots under `evidence/run4-*.png`.

## 0. Purpose, precedence, scoring

The operator's lane directive demands ShareNet-level UX discipline: persistent
shell, human vocabulary, always-visible state, calm hierarchy, visual topology,
first-class state blocks, accessibility. This document turns that demand into
the lane's checkable design contract. Every row is a pass/fail check mapped to
Flauz surfaces, with the ShareNet mechanism that inspired it cited by file and
concept.

- **Binding:** every Linux GUI wave must satisfy all MUST rows (§2 `Q*.*`,
  §3, §4) before review. `O` cells may be waived only with lane-lead sign-off.
- **Verdicts** reuse the journey-matrix vocabulary: `WORKS`,
  `WORKS-WITH-DEFECTS`, `UNAVAILABLE-BUT-HONEST`, `MISSING`, `HIDDEN`,
  `SILENT-NO-OP`, `MISLEADING`. `SILENT-NO-OP` and `MISLEADING` on any
  observed control or copy are automatic fails for the row they touch.
- **Precedence:** this rubric interprets PRODUCT-UX-JOURNEYS.md and
  FLAUZ-SOURCE-OF-TRUTH.md for the Linux GUI lane; it never overrides them.
- **GPUI translation:** ShareNet citations name web mechanisms (ARIA roles,
  CSS selectors). In Flauz's GPUI shell the binding equivalent is the visible
  worded text itself, plus one stable automation-addressable id per block,
  status triad, and primary action for Worker C's E2B battery (§5). Where the
  accessibility backend can announce state changes, blocks must announce.

## 1. Surface registry

Codes used throughout this rubric. §2.1 = PRODUCT-UX-JOURNEYS §2.1 target IA;
rc.14 = existing surfaces the rubric applies to unchanged.

| Code | Surface |
| --- | --- |
| `NAV` | Shell + persistent Workspace navigation: sidebar rail, section headers, collapse affordance, persistent service-status strip |
| `WS-PT` | Workspace Projects / Tasks view (§2.1) |
| `WS-PR` | Workspace Procedures library — rc.14 "Workflows" surface (§2.1) |
| `WS-AR` | Workspace Artifacts view (§2.1) |
| `WS-AC` | Workspace Activity / attention view — rc.14 Activity view (§2.1) |
| `TR-CX` | Task rail — Context (§2.1) |
| `TR-AG` | Task rail — Agents (§2.1) |
| `TR-EN` | Task rail — Environments (§2.1) |
| `TR-EV` | Task rail — Evidence (§2.1) |
| `TR-MI` | Task rail — More / Inspect: Resources, Artifacts, approvals, leases, activity detail (§2.1) |
| `CMP` | Composer + timeline with contextual actions (§2.1, rc.14) |
| `CL` | Chat/task list — rc.14 sidebar sections: New chat, Projects, Chats |
| `SV` | Sidebar capability views — rc.14: Repository, Pull requests, Plugins, Workflows |
| `TERM` | Terminal panel (rc.14) |
| `BRW` | Browser panel (rc.14) |
| `SET` | Settings (rc.14) |
| `PAL` | Command palette (rc.14) |
| `MOD` | Modals / dialogs / first-run promo (rc.14, run-4) |
| `TST` | Toasts / notifications (rc.14) |
| `GATE` | Signed-out auth gates — cross-cutting over all surfaces (run-4) |
| `ALL` | Every user-visible surface above |

## 2. The twelve operator questions — binding checks

Each question below is the operator's verbatim intent; each row is a MUST and
is binary. Surfaces use §1 codes. `Run-4` cites the live defect or keeper that
exercises the row today.

| # | Operator question (verbatim intent) | Rows |
| --- | --- | --- |
| Q1 | State immediately understandable | Q1.1–Q1.3 |
| Q2 | One obvious primary action | Q2.1–Q2.3 |
| Q3 | Stable navigation | Q3.1–Q3.3 |
| Q4 | Important status always visible | Q4.1–Q4.3 |
| Q5 | Human language first | Q5.1–Q5.3 |
| Q6 | Technical detail available but not dominant | Q6.1–Q6.2 |
| Q7 | Explicit loading / error / empty states | Q7.1–Q7.3 |
| Q8 | Summary → detail drill | Q8.1–Q8.3 |
| Q9 | Calm, not a telemetry dashboard | Q9.1–Q9.3 |
| Q10 | Reduced-motion respected | Q10.1–Q10.2 |
| Q11 | Keyboard focus visible | Q11.1–Q11.3 |
| Q12 | State not color-only | Q12.1–Q12.3 |

### Q1 — State immediately understandable

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q1.1 | Every surface renders a surface identity line — title plus a one-sentence purpose — above its content, before any interaction. | ALL | network/activity/devices/settings `page.tsx` — each page opens with an `h1` plus one plain-language purpose line under a `PageHeader` pattern. |
| Q1.2 | The surface's current condition (fresh, working, waiting, needs attention, blocked) is stated in words on first paint — never inferred from color, glyphs, or hover. | ALL | home/page.tsx `ConnectionHero` — the state word and a state-specific headline render as text (e.g. an offline state reads as a full sentence), not as a bare dot. |
| Q1.3 | A first-time signed-out user can answer "what is this showing, and what do I do next?" from one screenshot of the surface. | ALL | devices/page.tsx — header, purpose line, inline connected-count, and per-row worded status make the page self-describing without documentation. |

### Q2 — One obvious primary action

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q2.1 | Each surface-state presents exactly one visually dominant affordance; every other action is secondary or contextual in weight. | ALL | home/page.tsx — a single large pill CTA with a state-appropriate label (connect / disconnect / cancel / try again / enable) against one muted text-link secondary; onboarding/page.tsx — exactly one CTA per step. |
| Q2.2 | Empty and gated states name exactly one first action that starts the journey (create, sign in, set up, start). | WS-*, TR-*, CL, SV, GATE | state-blocks.tsx `EmptyState` — at most one action, deliberately rendered in outline weight so an empty state never outranks the surface primary. |
| Q2.3 | No silent no-ops: every visible control either acts or opens an honest gate / why-not state. `SILENT-NO-OP` on any control is an automatic fail. Run-4: L-009 (`run4-j05-projects-plus-silent.png`). | ALL | home/page.tsx — even the offline state gets a worded hero and its own primary action; no ShareNet control is visible-but-dead. |

### Q3 — Stable navigation

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q3.1 | Workspace navigation position, item order, and labels are identical across every state and auth mode; the nav never disappears due to loading, empty, or error. | NAV, ALL | app-shell.tsx — one `NAV_ITEMS` definition renders on every route (desktop sidebar `aside[aria-label]`, mobile header, bottom nav); active item marked `aria-current="page"`; zero layout shift between routes. |
| Q3.2 | Any collapse affordance keeps a persistent, visible restore affordance on screen — never only a hidden shortcut or palette command. Run-4: L-007 (`run4-l007-sidebar-collapsed.png`). | NAV | app-shell.tsx — the sidebar or an equivalent persistent nav surface is always present; no ShareNet state exists without on-screen navigation. |
| Q3.3 | Copy never instructs the user toward UI that is currently hidden. Run-4: L-007 — empty-state copy referenced a hidden sidebar. | ALL | ShareNet page one-liners reference only what is on screen (the network page's drill hint appears only while the hop list is visible). |

### Q4 — Important status always visible

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q4.1 | Service, auth, and active-run status render in a persistent shell status strip visible on every surface — never only inside a transient toast. | NAV, ALL | app-shell.tsx — `ConnectionStateIndicator` anchored in the sidebar footer connection box (and the mobile header), present on every route. |
| Q4.2 | Activity / attention is reachable from the nav or the palette in every auth state, with a worded signed-out state; a needs-attention count is visible wherever the nav shows counts. Run-4: J-17 — Activity missing signed-out. | WS-AC, NAV, PAL | app-shell.tsx — Activity is a first-class `NAV_ITEMS` peer of Home/Network/Devices/Settings; activity/page.tsx answers "what happened here" without auth dependencies. |
| Q4.3 | Status changes announce without stealing focus or keyboard input. | NAV, ALL | connection-state-indicator.tsx — a non-focusable element with `role="status"` and a descriptive label: implicit polite announcement, no focus theft; its pulse exists only for transient states. |

### Q5 — Human language first

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q5.1 | Primary copy uses the §4 labels; internal codenames, raw protocol / JSON-RPC codes, and process names never appear. Run-4: L-008 (raw `-32600` banner), L-006 ("chat" leak). | ALL | quality-helpers.ts — `qualitySentence()` composes full plain sentences; network-path-detail-sheet.tsx keeps raw protocol fields inside a demoted Advanced section, never in primary copy. |
| Q5.2 | Every error renders as a worded cause plus a next step in a state block; bare "Failed to load", error codes, or stack strings are fails. Run-4: L-008. | ALL | state-blocks.tsx `ErrorState` — one-sentence human message; raw error text is deliberately never rendered (diagnostics live behind a dedicated surface). |
| Q5.3 | Dependency-missing states name the missing thing in words and offer its setup action. Run-4 keeper: Pull-requests "GitHub CLI setup required" + Install / Check again. | SV, TERM, BRW, TR-EN | settings/page.tsx — an unbuilt subsystem is honestly worded as coming soon rather than hidden or erroring; offline states get worded heroes, not dead panels. |

### Q6 — Technical detail available but not dominant

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q6.1 | Error and unavailable blocks carry a collapsed Details affordance holding raw codes, request ids, and stack material; the default view shows none of it. | ALL error/unavailable blocks | activity-item.tsx — a collapsible show/hide-details control opens a small mono detail grid; network-path-detail-sheet.tsx demotes an Advanced section below the human summary. |
| Q6.2 | Deep identifiers (keys, fingerprints, event ids, protocol codes) appear only inside detail/inspect surfaces — never in lists or primary copy. | TR-MI, WS-AC, WS-PT, detail views | devices/page.tsx — the public-key fingerprint is never rendered in the device list; only the detail sheet shows it, with copy support and a plain-language explainer. |

### Q7 — Explicit loading / error / empty states

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q7.1 | Every async surface renders exactly one §3 block per reachable state; blank, stuck-spinner, or unending skeleton is a fail. | ALL stateful surfaces | state-blocks.tsx — the Loading/Error/Empty trio is exhaustive per surface; `LoadingSkeleton` variants mirror each page's real layout so loading never looks like a different screen. |
| Q7.2 | Error blocks present exactly one primary retry affordance — keyboard-reachable, disabled and visibly busy while retrying. | ALL | state-blocks.tsx `ErrorState` — a single "Try again" button, disabled + busy-flagged while retrying; no competing escape hatches. |
| Q7.3 | Unsent composer drafts survive restart; on relaunch the composer restores the draft or shows a resume prompt ("we saved your work"). Run-4: L-010 (`run4-j03-restart-draft-lost.png`). | CMP | onboarding/page.tsx — completion is durably remembered across restarts; durable user state is never dropped silently. |

### Q8 — Summary → detail drill

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q8.1 | List/timeline surfaces render summary rows (who / what / when + one-line outcome); selecting a row opens a detail surface. | WS-AC, WS-PT, WS-AR, TR-AG, TR-EV, CL | activity/page.tsx — rows carry time + title + description with details behind a per-row disclosure; devices/page.tsx — device cards open a detail sheet. |
| Q8.2 | Detail opens without destroying list context; returning restores the list. | All drill surfaces | network page — the path detail sheet overlays the path list rather than replacing it; the device sheet defers clearing content until its exit animation finishes. |
| Q8.3 | Full technical detail is at most two drills from any summary row (summary row → detail surface → optional advanced section). | ALL | network/page.tsx — quality summary card → path list → detail sheet with a demoted Advanced section: three calm levels end-to-end. |

### Q9 — Calm, not a telemetry dashboard

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q9.1 | At most 3 headline numbers or tiles are visible by default per surface; everything else is demoted behind drill. | WS-*, TR-* | network/page.tsx — one headline (quality word + latency) with three demoted metric cells below a divider; home/page.tsx — four word-valued tiles, no numerals; activity and settings — zero. |
| Q9.2 | No auto-refreshing charts or counters by default; refresh is an explicit secondary control that signals only while working. | WS-*, TR-* | network/devices/activity pages — refresh is a ghost or outline secondary button, spinning only during the refresh itself. |
| Q9.3 | Transient notifications auto-dismiss within 8 seconds, stack without covering nav or list rows, and never duplicate an inline state already on the surface. Run-4: L-011 (toast pinned >15 min). | TST, ALL | ShareNet has zero toast call sites — transient outcomes surface inline via state blocks and inline alert paragraphs, so nothing pins itself over navigation. |

### Q10 — Reduced-motion respected

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q10.1 | Every animated element (skeleton pulse, transitions, active-nav motion, spinners) consults the OS/app reduced-motion setting; when reduced, decorative motion is removed without removing meaning. | ALL | app-shell.tsx, connection-state-indicator.tsx, onboarding/page.tsx, activity-item.tsx — each animated element gates itself on the reduced-motion flag (nav pill dropped, indicator pulse off, step transitions to zero duration). Note the transfer nuance: gating is per animated element, not one global stylesheet switch. |
| Q10.2 | Loading feedback is never motion-only: skeletons and spinners are always paired with a text status, so progress is still visible with motion off. | ALL | state-blocks.tsx `LoadingSkeleton` — the root carries a busy flag, a polite live announcement, and a "Loading" label alongside the pulse. |

### Q11 — Keyboard focus visible

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q11.1 | Every interactive element shows a visible focus indicator on keyboard focus (2px outline with offset, or an equivalent ring); focus is never suppressed without replacement. | ALL | globals.css — a shell-wide `*:focus-visible` rule draws a 2px outline with 2px offset; nav links, node and device cards, and triggers add component-level rings. |
| Q11.2 | Tab order follows visual order; dialogs trap focus while open and restore it to their trigger on close. | MOD, PAL, ALL | device detail sheet — the sheet pattern provides escape-to-close, focus trapping, and focus return; content clearing is deferred so focus never lands on torn-down UI. |
| Q11.3 | Every dialog exposes a visible dismissal affordance — a real secondary control, not an invisible escape-only path. Run-4: L-012 — escape works (UX-003) but is invisible. | MOD | onboarding/page.tsx — the only CTA-only flow in ShareNet is a linear flow that ends in completion; no surface requires declining an offer to proceed. |

### Q12 — State not color-only

| ID | Check (MUST pass) | Surfaces | ShareNet reference |
| --- | --- | --- | --- |
| Q12.1 | Every status renders as a label + glyph + color triad, label on by default; no color-only dots, badges, or tinted text. | ALL | quality-helpers.ts — `qualityLabel` + `qualityPalette` + a quality glyph are always composed together; the file states the rule: color is never shown without both a textual label and an icon. connection-state-indicator.tsx renders its label by default. |
| Q12.2 | Status stays legible in grayscale — the state is readable from label + glyph alone. | ALL | privacy-overview.tsx — session guarantees pass/fail by icon and text treatment, never by hue alone. |
| Q12.3 | Status colors are defined as token triplets (base + soft background + on-color text) so text contrast is designed, not accidental. Exact values belong to the GPUI theming work order. | ALL | globals.css — connected/warning/error/neutral each ship as base, `-soft`, and `-text` token triplets backing every state color. |

## 3. State-block inventory

### 3.1 Block grammar

Six blocks. "Announce" is the ShareNet web mechanism; the GPUI binding is
visible worded text plus the automation id (§0, §5).

| Block | Rendered when | Required anatomy (visible text) | Announce | Max actions | ShareNet reference |
| --- | --- | --- | --- | --- | --- |
| Loading | Data requested, not yet arrived | Layout-mirroring skeleton + text status ("Loading …") | polite | 0 | state-blocks.tsx `LoadingSkeleton` — busy flag, polite announcement, label; per-surface variants mirror real layout |
| Success | An action completed and the result is not otherwise visible | One-line worded confirmation + next-step affordance (journey contract §1.5) | polite | 1 | ShareNet renders outcomes inline: settings switch optimistic confirm with rollback; copy-button check swap; zero toast call sites |
| Empty | Zero items; capability ready | What the capability will provide + one concrete first action (§1.4) | status | 1, secondary weight | state-blocks.tsx `EmptyState` — status role, dashed card, at most one outline action |
| Recovering | Degraded; automatic retry in progress | What is happening + what is preserved + a manual fallback | polite | 1 (manual fallback) | connection-state-indicator.tsx — a worded "recovering" label with motion-gated pulse; home hero shows the state sentence + cancel action |
| Error | Request or operation failed | Worded cause + next step; raw detail only behind a collapsed Details | alert (assertive) | 1 primary retry + Details | state-blocks.tsx `ErrorState` — alert role, assertive announcement, one sentence, single "Try again", raw error never rendered |
| Unavailable | Capability blocked by missing dependency or auth | What is missing + its setup or sign-in action | status | 1 (setup / sign in) | Flauz keeper (run-4): Pull-requests dependency card + Install / Check again; ShareNet analog: settings honest "coming soon" wording for an unbuilt subsystem |

### 3.2 Mandatory blocks per surface

`M` = must implement; `O` = must not be blank/stuck if the state occurs;
`—` = not applicable.

| Surface | Loading | Success | Empty | Recovering | Error | Unavailable |
| --- | --- | --- | --- | --- | --- | --- |
| NAV shell + status strip | M | — | — | M | M | M |
| WS-PT Projects / Tasks | M | — | M | O | M | M |
| WS-PR Procedures / Workflows | M | O | M | O | M | M |
| WS-AR Artifacts | M | O | M | O | M | M |
| WS-AC Activity | M | O | M | O | M | M |
| TR-CX Context | M | O | M | O | M | M |
| TR-AG Agents | M | O | M | O | M | M |
| TR-EN Environments | M | O | M | M | M | M |
| TR-EV Evidence | M | O | M | O | M | M |
| TR-MI More / Inspect | M | — | M | O | M | M |
| CMP Composer / timeline | M | M | M | M | M | M |
| CL Chat / task list | M | — | M | O | M | M |
| SV Sidebar capability views | M | — | M | O | M | M |
| TERM Terminal panel | M | O | — | O | M | M |
| BRW Browser panel | M | O | — | O | M | M |
| SET Settings | M | O | — | O | M | O |
| PAL Command palette | M | — | M | — | O | — |
| MOD Dialogs / promos | O | O | — | O | M | — |
| TST Toasts | — | — | — | — | — | — |
| GATE Signed-out gates | — | — | — | — | — | M |

Notes: TST is a transient carrier governed by Q9.3 and may never be the sole
carrier of persistent status (Q4.1). MOD additionally follows Q2.1 and Q11.3.
GATE is the Unavailable block for auth, rendered wherever a surface's data
requires sign-in (run-4 keeper: "Sign in to get started").

### 3.3 Honest-empty copy pattern (PRODUCT-UX-JOURNEYS §1.4)

Exemplars — final wording may vary, but both parts are mandatory: a sentence
naming what the capability will provide, plus one concrete first action.

| Surface | "What it provides" (exemplar) | First action (exemplar) |
| --- | --- | --- |
| WS-PT | Projects hold your tasks — each a running piece of work with its own agents, files, and results. | New project |
| CL | Tasks you start appear here with their current state. | New task |
| WS-PR | Reusable workflows remember the steps of work you did well, so you can run them again in one click. | Save a workflow (signed-out: Sign in) |
| WS-AR | Artifacts are the files your tasks produce — builds, documents, exports. | Start a task |
| WS-AC | Activity records what Flauz did while you worked — runs, changes, and approvals waiting on you. | Start a task |
| TR-CX | Context is what the agent can see right now — files, pages, terminals. | Add a file |
| TR-AG | Agents are the workers on this task; each has a role and its own tools. | Delegate work |
| TR-EN | Environments are the places commands run — terminal, browser, sandbox. | Add environment |
| TR-EV | Evidence is the proof behind each result — output, observations, verification. | Capture evidence |
| TR-MI | Inspect gathers this task's resources, artifacts, and approvals in one place. | Inspect task |
| SV Repository | A repository connects this task to a Git project. (Run-4 keeper: "No Git repository detected" + creation inputs.) | Connect a repository |
| SV Pull requests | Pull requests track review work for the connected repository. (Run-4 keeper: setup card + Install / Check again.) | Set up GitHub CLI |
| TERM (unavailable) | The terminal runs commands inside a task's environment. | Start a task |
| BRW (unavailable) | The browser panel drives a real browser inside a task. | Start a task |
| PAL (no results) | No commands match "…". | Clear search |
| GATE | Sign in to start tasks, save workflows, and keep your work across restarts. | Sign in |

## 4. Vocabulary — internal terms to human labels

Implements PRODUCT-UX-JOURNEYS §6 for the lane. The label column wins every
copy dispute; the copy scan (§5) enforces it.

| Internal term | Primary copy label | Rule |
| --- | --- | --- |
| Project | Project | §6 domain-neutral |
| Task | Task | The unit of work in all copy |
| chat / conversation / thread / session | Task | Never "chat", "thread", or "session" in primary copy (run-4 L-006); an environment session may be named inside detail views only |
| Procedure | reusable workflow | Primary copy per J-10: "Save as a reusable workflow"; library and nav label "Workflows"; the word "Procedure" appears only in secondary explanatory copy and docs |
| Resource | Resource, or the concrete noun (file, page, window) | Name the concrete thing when the user already knows it |
| Environment | Environment | §6 domain-neutral |
| Agent | Agent | §6 domain-neutral |
| Artifact | Artifact, or the concrete noun (file, build, export) | §6 domain-neutral |
| Evidence | Evidence | §6 domain-neutral |
| Activity | Activity | §6 domain-neutral |
| Approval | Approval ("Needs your approval") | §6 domain-neutral |
| Context | Context | §6 domain-neutral |
| ExecutionGraph / ExecutionNode | — (never user-visible) | Internal execution-planning structure; if a summary is unavoidable, say "steps" |
| app-server / backend process | service | Status strip reads Service: online / offline / reconnecting; raw process name only in Details |
| JSON-RPC codes (-32600), request ids, stack strings | — | Never in primary copy; Details disclosure only (run-4 L-008) |
| repository / branch / test / deployment | contextual only | §6: software-specific labels appear only when relevant to the current project |

## 5. How the E2B battery will score this

For Worker C's journey battery. Methods follow the journey-matrix protocol
(cold start per journey, pixel-verified clicks per the run-4 methodology
note); evidence lands under `docs/linux-gui/evidence/`.

| Method | Rubric rows | Battery procedure | Pass signal |
| --- | --- | --- | --- |
| SP — screenshot probe | Q1.1–Q1.3, Q2.1–Q2.3, Q4.1, Q4.2, Q7.1, Q9.1, Q9.2, Q12.1 | Visit each surface in each reachable state (signed-out, service stopped, empty data, populated, mid-run); capture PNG | Identity line + condition word + exactly one dominant action visible; verdict recorded in the journey matrix |
| KW — keyboard-path walk | Q3.1, Q11.1–Q11.3 | Tab-traverse the surface, screenshot each focus stop; escape every dialog | Visible focus indicator at every stop; tab order matches visual order; focus returns to the trigger |
| SBP — state-block presence | Q5.2, Q5.3, Q6.1, Q6.2, Q7.1, Q7.2 | Force each §3.2 state (signed-out, stop the app-server child, empty data, mid-retry) | The §3.1 anatomy renders: worded cause, single retry, Details collapsed |
| CS — copy scan | Q3.3, Q5.1 | Extract visible strings per surface; apply §4 allow/deny regexes | Zero denylist hits ("chat", "-326xx", "app-server", bare "Failed to load") |
| RM — reduced-motion probe | Q10.1, Q10.2 | Run with reduced motion enabled; diff frame pairs | No decorative motion; loading still carries its text status |
| TP — timing probe | Q9.3 | Trigger a notification; sample at 4 / 8 / 12 s | Auto-dismiss within 8 s; never overlaps nav or list rows |
| DR — draft-resume probe | Q7.3 | Type a composer draft; restart the app; screenshot the composer | Draft restored or a resume prompt shown |
| GS — grayscale probe | Q12.2 | Grayscale-convert SP captures | State readable from label + glyph alone |

Design requirement feeding the battery: every state block, status triad, and
primary action exposes one stable automation-addressable id naming its kind
(block kind, status kind, "primary"). The exact GPUI mechanism (test id or
accessibility id) is agreed with Worker C before the first scoring run;
screenshots alone do not scale to a per-surface × per-state matrix.

## 6. Run-4 grounding — what the rubric already outlaws or protects

| Finding (class · journey) | Rubric rows | Required direction |
| --- | --- | --- |
| L-009 Projects "+" silent no-op (P2 · J-05) | Q2.3 | Same honest sign-in gate as "New chat" |
| L-008 Workflows raw "-32600" banner (P2 · J-04) | Q5.1, Q5.2, Q6.1, WS-PR Error row | Worded Error block + Details; no codes in primary copy |
| L-007 Sidebar collapse without restore (P3 · J-04/J-11) | Q3.2, Q3.3 | Visible restore affordance; state-aware copy |
| L-010 Draft lost on restart (P2 · J-03) | Q7.3 | Persist draft; resume prompt on relaunch |
| L-011 Pinned, covering toasts (P3 · J-04) | Q9.3 | Timeout + stacking discipline; never duplicate an inline state |
| L-012 Forced-choice promo modal (P3 · J-01, cross-ref UX-003) | Q11.3 | Visible secondary dismiss (escape already verified by UX-003) |
| L-006 task / chat vocabulary drift (P3 · J-04) | Q5.1, §4 | "Task" wins; CS denylist enforces |
| J-17 Activity surface missing signed-out | Q4.2 | Activity reachable in every auth state (nav or palette) with a worded signed-out state |
| Keepers — protect as exemplars | Q1.3, Q2.2, Q5.3, Q8.1 | Repository honest empty state; Pull-requests dependency card + actions; palette inventory with shortcut badges; ≥2 discovery paths per capability |

## 7. Non-goals and change control

- No visual spec: colors, type scale, and pixel values belong to the GPUI
  theming work order; Q12.3 constrains token structure only.
- No code, CI, or repository changes outside `docs/linux-gui/` from this work
  order.
- ShareNet assets are never copied; citations name files and mechanisms for
  study only.
- Amendments go through the lane Tech Lead as a work order against this file;
  rows are appended, not silently rewritten, until a merged wave proves a row
  wrong.
