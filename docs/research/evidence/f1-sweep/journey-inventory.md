# F1 closure packet — GUI journey inventory (WO-F1-SWEEP-001)

- **Work order:** WO-F1-SWEEP-001 (Wave 1, Worker C — evidence-only)
- **Base:** `a664644718210e254952db518606d80906eee448`
- **Date:** 2026-09-19
- **Scope:** map all 18 journeys of `docs/PRODUCT-UX-JOURNEYS.md` §3 against
  what EXISTS on main, separating F1 parity-relevant slices from future-phase
  (F2+) work, and run the §8 cold-start discovery checks for the F1-relevant
  surfaces only (palette, settings, terminal, browser, history, attention).
  Per governance, Workflow/Pack and Procedure development is frozen during
  parity (docs/FLAUZ-SOURCE-OF-TRUTH.md "Immediate priority"), so the
  Procedure/collaboration journeys are recorded as future-phase, not gaps.

Anchor conventions: `PJ` = docs/PRODUCT-UX-JOURNEYS.md, `PM` =
docs/parity-matrix.md, `FW` = work-orders doc, source =
`symbol (file:~line)` (line numbers volatile).

## 1. Journey-by-journey mapping

| Journey | F1-relevant? | Implemented today (steps + anchors) | Missing / broken / future | Verdict |
| --- | --- | --- | --- | --- |
| **J-01 Start any project** (PJ L148) | core slice yes | New chat / New standalone chat / Open folder: registry ids `newTask`, `newProjectlessTask`, `openFolder` (lib.rs:139–215); bindings Ctrl+N, Ctrl+Alt+O (ui.rs:3781), Ctrl+O (ui.rs:5091); local-project registry + `Add new project` + `Select Project Root` picker + `/project` picker (PM L77); natural-language objective = the composer starter turn | "Flauz suggests relevant capabilities/environments/agents" — F5 capability resolver; cloud projects bounded | **partial, F1 core implemented** |
| **J-02 Understand what the agent knows** (PJ L160) | no (future) | bounded analog: `/status` panel (session ID copy, live context tokens + remaining %, account windows — PM L79); compaction context percentage (PM L78) | Context indicator/Inspector = F2 Context Engine (PJ §2.1 task control rail) | **partial analog only; future-phase** |
| **J-03 Recover/continue a long-running task** (PJ L172) | yes | reconnect supervision + loaded/background session recovery (PM L76; `ConnectionStatus` lib.rs:242–252; scheduler backend.rs:1796–1837); selected active-turn restoration + manual `/compact` + edit-latest retry (PM L78); retryable startup failure (PM L111) | transparent/automatic compaction (provenance remainder, P3 — parity-inventory row 4); context reset/reconstruction = F2+ | **partial, F1 recovery slice implemented** |
| **J-04 Discover a capability gap** (PJ L184) | honest-failure slice yes | honest bounded statuses for guarded actions (FW WO-P2-007/008/011 doctrine; `Action::SetStatus` bounded 16 KiB, lib.rs:20311–20313); feature gates render guidance (`/memories` gate, PM L79); retryable failure surfaces (PM L111) | capability-gap diagnosis + unlock paths = F5 gate (roadmap F5); silent no-op family F-A1/A2/A3/A6/F-D1/D2 contradicts "never end in a silent no-op" until WO-P2-012 lands (in-flight) | **partial; the doctrine is implemented, full journey future-phase** |
| **J-05 Add another environment/site** (PJ L199) | local slice yes | browser panel + site permission rules incl. Blocks/Allow cards (PM §In-app Browser permission status L133–150); per-chat terminal tabs (PM L88); MCP server add/update + marketplace add (PM L94); worktree as explicit workspace (PM L86) | provider/environment chooser, authorize, attach = F3 environment fabric | **partial, local surfaces attachable; future-phase for providers** |
| **J-06 Coordinate browser + terminal + sandbox** (PJ L215) | yes | terminal activity summaries in the timeline (PM L82); browser agent-driven CDP + downloads (PM L91); Outputs/artifacts with end-resource cards (PM L97); Process Manager for background terminals (PM L89) | cross-boundary "evidence links the steps" + verification = F6 evidence plane; cross-boundary movement visibility is bounded to timeline summaries | **partial, F1 surfaces implemented; evidence linking future-phase** |
| **J-07 Parallelize work** (PJ L230) | analog only | side chats Ctrl+Alt+S + `/side` (FW WO-P2-006); worktree forks with bounded three-item FIFO queue (PM L86) | Agents panel, roles, assignments, dependency inspection = F6 | **partial analog; future-phase** |
| **J-08 Human takeover / handoff** (PJ L245) | yes | approval cards with Enter/Escape yielding (PM L83/L112); Computer Use Escape interruption monitor + input indicator overlay (PM L90); browser one-shot user input grant (PM L91); terminal as manual surface | explicit/visible takeover state = F6 human-gates | **partial, F1 approval/interruption slice implemented** |
| **J-09 Understand what happened** (PJ L258) | no (future) | streaming timeline with expandable summaries + Copy as Markdown export (PM L82); review/diff surfaces (PM L85) | claims/observations/verified distinctions = F6 evidence plane (PJ §3 J-09 wording) | **partial analog; future-phase** |
| **J-10 Save a successful task as a Procedure** (PJ L273) | no | none (Workflow/Pack frozen; no "Save as a reusable workflow" CTA exists) | the whole journey = F6 Procedure capture | **future-phase** |
| **J-11 Discover a learned Procedure later** (PJ L298) | no | none (no Procedures library surface) | F6 | **future-phase** |
| **J-12 Reuse / improve a Procedure** (PJ L312) | no | none | F6 | **future-phase** |
| **J-13 Collaborate on one task** (PJ L330) | no | bounded remote-connections UI only (PM L105, public-contract-gated) | presence/share/invite = F9 | **future-phase** |
| **J-14 Switch model without losing work** (PJ L342) | yes | model/effort/service-tier picker: `config/read` defaults for new chats, `thread/resume` restores, `thread/settings/update` applies immediately to the existing chat — continuity preserved, not a new blank chat (PM L80) | Context Engine recompile = F2/F6 (app-server-side continuity covers the F1 slice) | **implemented (F1 parity-relevant)** |
| **J-15 Switch execution environment without losing work** (PJ L354) | local slice yes | `Continue in new worktree` managed forks: tracked/untracked/ignored state transfer within 20 MiB, nested roots, activity flow, queue serialization (PM L86) | local→remote / provider switching = F3 | **partial, worktree switch implemented** |
| **J-16 Inspect resource conflicts** (PJ L366) | no | none | ResourceLease/conflict surfaces = F2 contracts (roadmap) | **future-phase** |
| **J-17 Review activity that needs the human** (PJ L381) | **yes — the F1 attention slice** | unread-attention state: background turn completion (incl. failed turns) + approval requests in non-selected chats flag the chat; visit clears, archive drops (FW WO-P2-008); sidebar 6 px dot + medium-weight title (ui.rs:14765–14782); bindings: `toggleThreadUnread` Ctrl+Shift+U, `nextUnreadChat` Ctrl+Alt+A (cyclic jump = **open exact task**), `clearAllUnread` Shift+Escape (honest count), `toggleActivityView` Ctrl+Alt+U (honest guidance) (ui.rs:2980/3057/3064/3071); background-completion banner with Open/Dismiss (PM L104) = return-to-attention analog | **Activity view surface itself: in-flight** (deferred `toggleActivityView` target; Workers A/B this wave; reference view shape auth-walled `[unverified]`, FW WO-R-REF R1); "failed environment / collaborator request" slices future-phase (F9 environments/collaborators) | **partial — attention slice implemented; the Activity view surface is the F1-relevant missing piece (in-flight)** |
| **J-18 Discover automation opportunities** (PJ L393) | no | none (no repeatability observation; scheduled tasks bounded proprietary) | F6 Procedures + bounded scheduled-task backend | **future-phase** |

## 2. Cold-start discovery validation (PJ §8) — F1-relevant surfaces only

Per PJ §8: begin from the normal Workspace/Task surface with no search;
confirm primary path, context path, recovery path, search fallback, and
success path. Findings below are source/doc-anchored; runtime probes are
listed where source inspection is inconclusive.

| Surface | Primary path (visible entry) | Context path | Search/palette fallback | Empty state | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Command palette** | **gap** — no visible toolbar button (PR §5.10 UX-parity cell); a cold-start user must learn the chord from Help → Keyboard shortcuts (the dialog lists the `openCommandMenu` binding, lib.rs:154) or the Ctrl+/ overlay | palette opens contextually from chat-search drill-in (Ctrl+G → "Back to commands") | n/a (it IS the search fallback for everything else — WO-P2-004 indexed all default-nav Settings sections) | "No matches" bounded empty state (ui.rs:5010–5024) | **partial — keyboard path pass, visible-chrome entry gap** |
| **Settings** | sidebar entry + Ctrl/Cmd+comma → General (PM L101; ui.rs:5096) | contextual: MCP/Plugins/etc. rows open from their own flows | palette Settings group with all 11 default-nav entries (FW WO-P2-004, merged `7aa7163`) | per-section loading/error/empty/no-match states (PM L101) | **pass** |
| **Terminal** | persistent sidebar affordance (FW WO-P1-001, merged `d15333e`; roadmap F1 merged list) | completed background-terminal activity opens Process Manager (PM L89) | palette rows: `toggleTerminal` / Open terminal in the verified registry subset (PM L112) | first-session start on open (PM L88) | **pass** |
| **Browser** | persistent sidebar affordance (FW WO-P1-002, merged `d15333e`) | site permission cards appear when the agent requests origin/download/CDP (PM §In-app Browser permission status) | palette: Browser settings indexed (WO-P2-004); browser chords context-scoped (FW WO-P2-011) | downloads manager bounded empty state (PM L91) | **pass** |
| **Browsing history** | Settings → Browser management surface: bounded first-8 rows + Clear… modal + footer (FW WO-P2-009, merged `5302e7e`/`fe3903e`) | address-bar revisit matching (`browsing_history_revisit_target`) | settings search reaches the Browser section (PM L101) | cleared/empty state with disabled Clear (FW WO-P2-009 D12f) | **pass** |
| **Attention (J-17 slice)** | sidebar dot + medium-weight title on unvisited chats (ui.rs:14765–14782) — visible without search | background-completion banner with Open (PM L104) | bindings are the accelerators (Ctrl+Alt+A jump); Ctrl+Alt+U gives honest guidance while the view is absent (FW WO-P2-008) | honest empty statuses ("No unread chats") on clear-all/next-unread (FW WO-P2-008) | **partial — state visible and honest; the aggregated Activity view (the journey's primary surface, PJ §2 Activity/collaboration surface) is in-flight** |

## 3. Summary

- **Implemented (F1 parity-relevant):** J-14 (model switch continuity);
  J-17's attention slice (state + bindings + dot + honest statuses); J-01's
  core start flow; J-03's recovery slice; J-06's surface coordination;
  J-08's approval/interruption slice; J-15's worktree switch.
- **Partial with in-flight F1 pieces:** J-17 Activity view surface (Workers
  A/B this wave); J-04's silent no-op residuals (WO-P2-012).
- **Future-phase (F2+):** J-02, J-05 (provider attach), J-07 (agents), J-09,
  J-10, J-11, J-12, J-13, J-16, J-18 — each blocked on its roadmap phase
  (F2 contracts / F3 environments / F5 capabilities / F6 orchestration+evidence
  +procedures / F9 collaboration), consistent with the frozen governance
  (FLAUZ-SOURCE-OF-TRUTH "Immediate priority": finish parity first).
- **Cold-start surface score:** 4 pass (settings, terminal, browser, browsing
  history), 2 partial (palette — no visible entry; attention — view surface
  in-flight).

## 4. Source-unverifiable notes

1. The palette's "context path" (Ctrl+G drill-in → Back to commands) is
   source-anchored (ui.rs:4906–4924) but was not runtime-probed for this
   packet; D-series evidence (wo-p2-009/010) covers palette surfaces at the
   entry state.
2. The reference Activity-view shape remains auth-walled `[unverified]`
   (FW WO-R-REF R1) — the in-flight implementation should re-verify against
   an authenticated reference before closing.
3. Help-menu reachability of the Keyboard shortcuts dialog on the Linux
   runtime is evidenced (PM L113, Computer Use-exercised); the full
   keyboard-only journey probe (RG-A11Y-01) will validate the remaining
   source-unverifiable focus findings (accessibility-inventory §
   source-unverifiable notes).
