# F1 closure packet — parity inventory (WO-F1-SWEEP-001)

- **Work order:** WO-F1-SWEEP-001 (Wave 1, Worker C — evidence-only)
- **Base:** `a664644718210e254952db518606d80906eee448` (origin/main HEAD at dispatch)
- **Date:** 2026-09-19
- **Status:** input to the Tech Lead's F1 close-gate decision; this file inventories
  REMAINING differences and does not re-verify closed claims (merged evidence stands).

## 1. Method and classification vocabulary

Spine input: `docs/parity-matrix.md` — every row of the **Product parity** table
(41 rows, lines 75–115) plus the **Release-critical parity ledger (GUI-002)**
(lines 299–349). Secondary inputs: `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md`
("PR" below — §5.10, §8.1/§8.2, findings 16–18), `docs/research/FEATURE-PARITY-WORK-ORDERS.md`
("FW" below), and read-only source inspection (`crates/codex-app/src/ui.rs`,
`crates/codex-app/src/backend.rs`, `crates/codex-core/src/lib.rs`).

Classification of each extracted remainder (per the work order):

| Class | Definition |
| --- | --- |
| `closed` | remainder landed on main with merged work-order evidence |
| `in-flight` | WO-P2-012 (unmerged guard-honesty branches) or the Activity-view surface (owned by Workers A/B this wave) |
| `P2` | in-baseline `26.721.3996.0` full-reference difference — the pinned reference exposes the behavior and Flauz lacks it |
| `P3` | polish (no reference behavior claimed) or future-reference (post-baseline 26.727/26.825 delta carried by the parity report, or 26.908+ = reference+1) |
| `bounded` | explicitly out: proprietary backend, platform row, or awaiting a public host contract (per the GUI-002 ledger verdicts) |

Layer tags record where each remainder's reference behavior lives, so the Lead
can re-slice P2/P3 against the current-target rule if desired:
`[26.721]` baseline · `[26.727]`/`[26.825]` current-target delta · `[26.908+]`
reference+1 (out of target) · `[polish]` no reference behavior.

Anchor conventions: `PM §Product parity › <row>` = parity-matrix row;
`PM §ledger › <row>` = GUI-002 release-critical ledger row; `PR §<section>` /
`PR finding <n>` = parity report; source anchors are `symbol (file:~line)` —
line numbers are volatile; the symbol is the stable identifier.

## 2. Row-by-row inventory — all 41 Product-parity rows

Row statuses as recorded in the matrix: `done` 4, `partial` 29, `missing` 7,
`platform` 1 (the work order's "~25 partial" is an approximation; the actual
count is 29). "Extracted remainder" quotes the matrix's "Reference contract
still required" tail verbatim (truncated with "…" only where noted).

| # | Row (PM anchor) | Status | Extracted remainder | Classification (per item) | Evidence anchors |
| --- | --- | --- | --- | --- | --- |
| 1 | §Product parity › Runtime bootstrap | done | none ("Keep the exact packaged CLI hash check and explicit override/fallback order") | **closed** — no remainder; ledger has no row | PM L75; PM §Reference baseline |
| 2 | §Product parity › App-server supervision | partial | "Complete the remaining stable methods plus remote and network-aware transport diagnostics" | **P3**: network-aware/remote transport diagnostics `[polish — ledger "enhancement"]`; remaining stable methods `[26.721 schema, but no visible vertical slice requires them — PM's own rule "Method count alone does not define parity" (PM §Protocol progress L279–281)]` → P3 with that note | PM L76; PM §ledger › App-server supervision ("network-aware diagnostics; remaining experimental methods (enhancement)"); PR §8.2 item 17 |
| 3 | §Product parity › Projects and chats | partial | "Add multi-root sources, manual project ordering, richer metadata, and unread state" | **P2**: multi-root sources `[26.721 family — WO-P1-003 delivered the local multi-folder model; the residual is source-type coverage, needs Lead scoping]`; **closed**: manual project ordering (GUI-002 closure, PM L347–349); **P3**: richer metadata `[polish — PR §8.2 P3 list]`; **closed-with-staleness**: "unread state" — row text predates WO-P2-008 (merged `00a3392`): per-chat unread-attention state + bindings exist (see §4); residual = persistence across restarts (reference behavior unverified → P3) and, under one reading, project-row unread counts (P3). **In-flight note**: the Activity-view surface that consumes this state is in-flight (§4) | PM L77; PM §ledger › Projects and chats; PR finding 18; FW WO-P2-008; `needs_attention_task_ids` (lib.rs:~960s region, reduce at lib.rs:~9041); dot render (ui.rs:14774–14782) |
| 4 | §Product parity › Thread execution | partial | "Add automatic/manual compaction provenance and richer edit metadata" | **P3** both `[polish — PR §8.2 P3 list item 1]` | PM L78; PM §ledger › Thread execution; PR §8.2 P3 |
| 5 | §Product parity › Composer | partial | "Add cloud projects, the remaining stable slash commands, Cloud target, and voice" | **bounded**: cloud projects, Cloud target, voice (proprietary/cloud — PR §8.2 deferred set); **P2**: remaining stable slash commands `[26.721 stable set]` — post-WO-P2-005 six names stay open (FW WO-P2-005 closure): those split into capability-deferred (bounded) and unverified-semantics (P3 until reference-verified); the P2 class covers any evidenced in-baseline name still absent | PM L79; PM §ledger › Composer ("cloud projects/voice (proprietary, unavailable); remaining slash commands (polish)"); FW WO-P2-005; PR §5.2 Composer row |
| 6 | §Product parity › Model, effort, and speed picker | partial | (no "Add" tail; row ends with exercised evidence) | **closed** — ledger verdict "none listed" | PM L80; PM §ledger › Model, effort, and speed picker |
| 7 | §Product parity › Permission profiles | partial | "Add sandbox detail, the granular/custom editor, and per-project resolution" | **P2** all three `[26.721 — reference exposes the granular/custom permission editor; PR §8.2 item 14 carries them as ledger enhancements]` | PM L81; PM §ledger › Permission profiles; PR §8.2 item 14 |
| 8 | §Product parity › Streaming timeline | partial | "Add adjacent activity grouping, internal file navigation from citation chips, turn-level source aggregation, subagent navigation/grouping, and exact history-position restoration" | **P3** all five `[polish — PR §8.2 P3 list item 2 ("streaming-timeline grouping/citation navigation")]` | PM L82; PM §ledger › Streaming timeline; PR §8.2 P3 |
| 9 | §Product parity › Approvals and user input | partial | "Add the remaining connector-specific request methods and persistence choices where the public contract exposes them" | **bounded** both (explicitly conditioned on a public contract that does not exist) | PM L83; PM §ledger › Approvals and user input ("connector-specific methods where public contracts expose them (bounded)") |
| 10 | §Product parity › Repository status | partial | "Add pull, guarded force-push UX, richer conflict resolution, repository picker, and multi-repository projects" | **P2**: pull `[26.721 row contract — needs scoping]`; **P3**: guarded force-push UX (PR §8.2 P3 "PR … force push … options"), richer conflict resolution, repository picker `[polish]`; **P3**: multi-repository projects `[26.727 — future-reference vs the pinned baseline; PR §8.2 item 13 carries it as P2 blocked-by WO-P1-003; recorded for Lead reconciliation]` | PM L84; PM §ledger › Repository status; PR §8.2 items 13 + P3 list |
| 11 | §Product parity › Diff review | partial | "Add multi-file expansion, syntax parity, inline comments and handoff, per-hunk actions, and guarded revert controls" | **P3** all five `[PR §8.2 P3 list item 6 verbatim: "diff-review syntax highlighting/inline comments/per-hunk actions/guarded revert"; "multi-file expansion" is partially delivered by PM §Multi-file diff baseline (L117–124) — residual scope ambiguous, needs Lead read]` | PM L85; PM §Multi-file diff baseline; PR §8.2 P3 |
| 12 | §Product parity › Branches and worktrees | partial | "Add guarded removal and cleanup policy, setup scripts/local environments, synced branches, richer worktree status/repair, and remote worktrees" | **P3** all five `[polish/future — no PR P2/P3 entry; row contract claims reference exposure but each item is scope/robustness depth, not a missing reference surface]` | PM L86; PM §ledger › Branches and worktrees |
| 13 | §Product parity › Pull requests | partial | "Add automatic linked-PR detail loading for merge shortcuts, the GitHub connector path, detached-worktree branch creation, reviewer conversation threads and inline replies, and the remaining guarded GitHub mutations" | **bounded**: GitHub connector path (connector awaits public contract); **P3**: the other four `[polish/future reference depth]` | PM L87; PM §ledger › Pull requests; PR §8.2 deferred set |
| 14 | §Product parity › Terminal | partial | "Add generic movable panel tabs, session transfer, and workspace-binding warnings" | **P3** all three `[polish]` | PM L88; PM §ledger › Terminal |
| 15 | §Product parity › Process Manager | partial | "Add runtime/status values when the official protocol exposes them and cross-chat aggregation if an upstream contract becomes available" | **bounded** both (upstream-contract-gated by the row's own text) | PM L89; PM §ledger › Process Manager |
| 16 | §Product parity › Computer Use | partial | "Add the remaining bounded registry catalog sources, persistent 24-hour owned cache parity, broader browser identification, low-level wheel/touch and injected-event distinction, the stable separate animated cursor surface plus exact DirectComposition shimmer/shadow and live locale/theme-accent updates, window-level AUMID resolution for multiplexed hosts, richer transient-window screenshot state, official plugin discovery, Chrome/Excel/PowerPoint controls, transfer approvals, Linux AT-SPI/Wayland parity, and platform equivalents for picture-in-picture/locked use" | **P2** (Windows, in-baseline `[26.721]`): registry catalog sources; persistent 24-hour owned cache parity; broader browser identification; wheel/touch + injected-event distinction; animated cursor surface + DirectComposition shimmer/shadow + live locale/theme-accent; window-level AUMID resolution; transient-window screenshot state; official plugin discovery; Chrome/Excel/PowerPoint controls; transfer approvals. **bounded** (platform): Linux AT-SPI/Wayland parity; picture-in-picture/locked-use platform equivalents — ledger verdict "green (Windows), bounded (Linux); AT-SPI/Wayland parity is platform work" | PM L90; PM §ledger › Computer Use; WO-PLAT-001 (FW); `crates/codex-platform/src/computer_use.rs:34–57` (Linux observation bound); docs/platform-support.md §Linux |
| 17 | §Product parity › In-app browser | partial | "Add local-preview comments and annotations, site permissions, exact remaining Browser settings, and repeatable official-client smoke coverage; WebMCP remains gated to internal/Public Beta builds in stable" | **P2**: local-preview comments/annotations, site permissions, exact remaining Browser settings `[26.721 row contract]`; **P3**: repeatable official-client smoke coverage `[process/coverage, not user-visible]`; **bounded**: WebMCP (internal/Public-Beta gated in stable per the row's own text; PR §8.2 item 9 carries it as blocked-on fork-runtime) | PM L91; PM §In-app Browser permission status; PM §ledger › In-app browser; PR §8.2 item 9 |
| 18 | §Product parity › Scheduled tasks | missing | whole row: "Suggestions, manual/chat-assisted creation, schedule editor, run history, unread/archive states, plugin templates, pause/edit/delete, and notifications" | **bounded** (cloud tasks backend proprietary) | PM L92; PM §ledger › Scheduled tasks; PR §8.1 deferred |
| 19 | §Product parity › Marketplace admin-disabled install | done | none | **closed** | PM L93 |
| 20 | §Product parity › Plugins marketplace | partial | "Add native first-party App OAuth/no-auth callback completion when the official app-server exposes that private ChatGPT/Electron flow, action read/write grouping when exposed by the official protocol, remaining plugin disclosure states, sharing/editing, richer filters, hero rotation, and admin states" | **bounded**: App OAuth/no-auth callback; action read/write grouping (both explicitly await the official protocol); **P3**: disclosure states, sharing/editing, richer filters, hero rotation, admin states `[polish]` | PM L94; PM §ledger › Plugins marketplace; PR §8.2 deferred |
| 21 | §Product parity › Skills | partial | "Add recommended/install flows, creation and detail surfaces, extra roots, and higher-priority override guidance" | **P2**: recommended/install flows; creation and detail surfaces `[26.721 — PR §8.2 item 17 "Skills recommended/install flows + `$` invocation"]`; **P3**: extra roots; higher-priority override guidance `[polish]` | PM L95; PM §ledger › Skills; PR §8.2 item 17 |
| 22 | §Product parity › MCP and apps | partial | "Add native App OAuth/no-auth callback completion when exposed by the official app-server and sandboxed MCP App surfaces" | **bounded** both (official-protocol-gated) | PM L96; PM §ledger › MCP and apps |
| 23 | §Product parity › Artifacts and files | partial | "Add host-side existence filtering, PDF and Office renderers, presentation navigation/zoom, generated-image edit/canvas actions, AVIF/GIF, website/HTML detection, syntax-highlighted source parity, diff counts, native-app target menus, general artifact downloads, and full-screen/tab controls" | **P3** all eleven `[26.727 per PR §8.2 item 15 ("Artifacts/outputs renderers + canvas actions + generated-image editing (26.727)") → future-reference vs the pinned baseline; PR carries them as P2 for the current target — recorded for Lead reconciliation; the remainder are polish]` | PM L97; PM §ledger › Artifacts and files; PR §8.2 item 15 |
| 24 | §Product parity › Sites | missing | whole row: "Create, preview, annotate, version, share, publish, and return-to-chat flows" | **bounded** (OpenAI cloud surface proprietary) | PM L98; PM §ledger › Sites |
| 25 | §Product parity › Visualizations | missing | whole row: "Native chart/report rendering, source inspection, interaction, export, and editor handoff" | **bounded** (OpenAI cloud surface proprietary) | PM L99; PM §ledger › Visualizations |
| 26 | §Product parity › Appshots | missing | whole row: "Foreground-window capture, destination policy, hotkey, sound, preview, and text/offscreen context" | **bounded** (proprietary capture flow; 26.908 Windows arrival = reference+1, out of target) | PM L100; PM §ledger › Appshots; PR §5.10 Appshots row |
| 27 | §Product parity › Settings | partial | "Add remaining visible sections only with their working host contracts, plus real backdrop translucency, system-font stack presentation, and syntax-highlighting parity with host scoping, policy states, validation, and persistence" | **bounded**: remaining visible sections (host-contract-gated by the row's own text — Import/Codex Micro/Appshots/Cloud preferences/Cloud environments/Computer history/Environments/Voice/Pets/gated Debug registry, PM L101); **P3**: backdrop translucency, system-font stack presentation, syntax-highlighting parity `[polish]` | PM L101; PM §ledger › Settings; PM §Settings inventory (26-section reference) |
| 28 | §Product parity › Account and usage | partial | "Add reset-credit detail UI and guarded upgrade/purchase/auto-reload billing entry points" | **P2** both `[26.721 reference billing entry points; PR §8.2 item 17 carries billing entry points as ledger enhancement]` | PM L102; PM §ledger › Account and usage; PR §8.2 item 17 |
| 29 | §Product parity › Feedback | done | none | **closed** | PM L103 |
| 30 | §Product parity › Notifications and tray | partial | "Add unread/recent tray groups, deep links, sounds, badges, and scheduled-task behavior" | **P2**: unread/recent tray groups, deep links, sounds, badges `[26.721 Windows tray contract; Linux tray/global shortcuts are platform-bounded — cross-ref row 41]`; **bounded**: scheduled-task behavior (proprietary backend) | PM L104; PM §ledger › Notifications and tray; PR §8.2 item 17 |
| 31 | §Product parity › Remote control and SSH | partial | "Keep-awake, SSH profiles, remote chats, and handoff remain unavailable until a public app-server contract exists" | **bounded** all four (public-contract-gated by the row's own text) | PM L105; PM §ledger › Remote control and SSH |
| 32 | §Product parity › Cloud environments | missing | whole row: "List/detail/create links, repository/machine metadata, target selection, environment connection state, and cloud execution" | **bounded** (OpenAI cloud backend proprietary) | PM L106; PM §ledger › Cloud environments |
| 33 | §Product parity › Voice | missing | whole row: "Dictation, realtime voice, handoff target, stage layout, settings, and accessibility states" | **bounded** (proprietary realtime stack) | PM L107; PM §ledger › Voice |
| 34 | §Product parity › Personalization and memory | partial | "Add Custom instructions, retention controls, computer history, Chronicle permissions, and other memory surfaces only when their currently private host contracts have a native public boundary" | **bounded** all (private-host-contract-gated by the row's own text) | PM L108; PM §ledger › (Personalization carries no separate ledger row; the §ledger Settings row's "host contracts" bound covers it) |
| 35 | §Product parity › Pets and Codex Micro | missing | whole row: "Overlay lifecycle, device integration, commands, settings, mini-game/composer, sound, and accessibility" | **bounded** (proprietary companion surface; 26.908 = reference+1) | PM L109; PM §ledger › Pets and Codex Micro; PR §5.10 Pets row |
| 36 | §Product parity › Import and migration | partial | "Add exact unsupported-project reporting when the public protocol exposes it, richer per-history warning correlation, startup migration prompts, and remaining private-host recovery affordances" | **bounded**: unsupported-project reporting (public-protocol-gated); private-host recovery affordances; **P3**: warning correlation, startup migration prompts `[polish]` | PM L110; PM §ledger › Import and migration |
| 37 | §Product parity › First run and updates | partial | "Add fuller welcome, dependency setup/diagnostics, update prompt, and quit confirmation" | **P2** all four `[26.721 first-run/update contract; PR §5.10 First-run row Gap is "ux — P2 (welcome, diagnostics, update prompt; no WO yet — needs scoping)"; ledger treats them as release-bar enhancements — recorded for Lead reconciliation]` | PM L111; PM §ledger › First run and updates; PR §5.10 + §8.2 item 17 |
| 38 | §Product parity › Keyboard and accessibility | partial | "Add the remaining stable commands and Settings sections, complete focus order, screen-reader labels, OS-level reduced-motion following, and contrast" | **in-flight** (headline): the six evidenced silent-no-op guard residuals F-A1 (archiveThread), F-A2 (toggleThreadPin), F-A3 (renameThread), F-A6 (git.commit with pending PR), F-D1 (/review unavailable), F-D2 (/compact not-ready) + executor fall-through tests — WO-P2-012 branches exist unmerged (`origin/feat/wo-p2-012-guard-honesty` `0547055`, `-r2` `4fc763a`; roadmap requires rebase/reconcile before merge). **P2**: remaining evidenced stable commands `[26.721 registry — full reference inventory not enumerable from repo evidence, PR WO-R-REF R3]`; complete focus order `[26.721 a11y baseline — PM §Keyboard accessibility baseline L126–131]`; contrast (control exists, full parity open). **platform-bound**: screen-reader labels (GPUI exposes no accessibility tree at the current layer); OS-level reduced-motion following (no shared OS motion signal on the native Windows/Linux shell — PM L101). F-A4 (Ctrl+P silent with no workspace, documented residual of WO-P2-007) is WO-P2-012-territory scope material (see FW WO-P2-007 known limitations) | PM L112; PM §Keyboard accessibility baseline; PM §ledger › Keyboard and accessibility; PR finding 16/17 + §5.10 Gap cell; FW WO-R-SWEEP findings of record; `KEYBOARD_SHORTCUT_COMMAND_IDS` (lib.rs:139–215, 76 ids); WO-P2-012 branch diff (six SetStatus guards: "Select a chat before archiving it." etc.) |
| 39 | §Product parity › Active keyboard shortcut reference | done | none | **closed** | PM L113 |
| 40 | §Product parity › Windows | partial | "complete packaging, signing, native browser/Computer Use integration, notifications, and accessibility" | **bounded** (primary): packaging/signing — GUI-006 verdict, unsigned portable ZIP with SHA-256 + documented verify step is the release strategy, code signing unavailable to the program; native browser/Computer Use integration, notifications, accessibility are tracked on their own rows (16/17/30/38) — cross-references, not separate remainders here | PM L114; PM §ledger › Windows; docs/platform-support.md §Windows; docs/known-failures.md §Active release-candidate limitations |
| 41 | §Product parity › Linux | platform | "complete Wayland portals, desktop matrix, packaging, tray/badges, global shortcuts, and parity smoke tests" | **bounded** all (GUI-006 platform verdict — unsigned portable tar.gz, `--install-desktop-entry` only; no system package, Wayland portals, tray, or global shortcuts, documented in docs/platform-support.md §Linux) | PM L115; PM §ledger › Linux; docs/platform-support.md §Linux |

Rows 1–41 account for the entire Product-parity table. The four `done` rows
(1, 19, 29, 39) plus row 6 ("none listed") carry no open remainder.

## 3. PR finding 16 — runtime-surfaced shortcut delta (classified)

Finding 16 (PR lines 652–660): the official Linux-preview overlay (26.908.70816
= reference+1 evidence layer) carries bindings that were absent from Flauz's
then-45-command registry. Current state at base `a664644` (registry is now 76
ids — `KEYBOARD_SHORTCUT_COMMAND_IDS`, lib.rs:139–215;
`MAX_KEYBOARD_SHORTCUT_COMMANDS = 76`, lib.rs:137):

| Shortcut (official) | Flauz state at `a664644` | Classification | Evidence anchors |
| --- | --- | --- | --- |
| Search Files Ctrl+P | `searchFiles` interceptor command owns Ctrl+P; dead `OpenFileSearch` action removed | **closed** (WO-P2-007, PR #20 → `a3c0e01`) | PM L112; PR finding 17 + FIXED block; FW WO-P2-007 |
| Copy deeplink Ctrl+Alt+L | registry `copyDeeplink`, default CmdOrCtrl+Alt+L; palette row (WO-P2-010) | **closed** | ui.rs:2994; PM L112; FW WO-P2-010 |
| Copy working directory Ctrl+Shift+C | registry `copyWorkingDirectory`, default CmdOrCtrl+Shift+C; dual-meaning chord in browser context (WO-P2-011) | **closed** | ui.rs:3008; ui.rs:1427–1441 (BrowserPaneChord); FW WO-P2-011 |
| Rename chat Ctrl+Alt+R | registry `renameThread` (lib.rs:211); GPUI binding ctrl-alt-r (ui.rs:5066); palette row; silent-when-unselected residual is WO-P2-012 territory | **closed** (+ in-flight guard residual) | lib.rs:211; ui.rs:5066; FW WO-R-SWEEP F-A3 |
| Close Tab Ctrl+W | registry `closeWindow` (lib.rs:213); GPUI binding ctrl-w (ui.rs:5063) — close-window semantics in a single-window shell; the official's tab semantics have no tab-strip counterpart | **closed with semantic delta** (window-close vs tab-close) — noted for the Lead | lib.rs:213; ui.rs:5063 |
| Archive chat Ctrl+Shift+A | registry `archiveThread` (lib.rs:142); GPUI binding ctrl-shift-a (ui.rs:5065); silent-when-unselected residual is WO-P2-012 territory | **closed** (+ in-flight guard residual) | lib.rs:142; ui.rs:5065; FW WO-R-SWEEP F-A1 |
| New standalone chat Ctrl+Alt+O | registry `newProjectlessTask`, default Ctrl+Alt+O | **closed** | ui.rs:3781 (`Self::NewStandaloneChat => Some("Ctrl+Alt+O")`); PM L112 |
| Toggle pin Ctrl+Alt+P | registry `toggleThreadPin` (lib.rs:143); default Ctrl+Alt+P (ui.rs:3795) | **closed** | ui.rs:3795; PM L112 |
| Back / Forward Ctrl+[/] | registry `navigateBack`/`navigateForward`; GPUI bindings (ui.rs:5073–5074) | **closed** | PM L112; ui.rs:5073–5074 |
| Recent-chat cycling Ctrl+Tab / Ctrl+Shift+Tab | absent — no registry id; adjacent-chat navigation exists as `previousThread`/`nextThread` (Ctrl+Shift+[ / ] and Ctrl+PageUp/PageDown) | **P3** `[26.908+ reference+1 observation; no in-baseline evidence in repo records]` | PR finding 16; lib.rs:139–215 (no cycling id); PM L112 (Previous/Next chat contract) |
| Switch to Work Alt+2 | absent — no registry id; the "Work" surface is a 26.908 concept | **P3** `[26.908+ reference+1]` | PR finding 16; lib.rs:139–215 |

Finding 16's P3 extension therefore reduces, at `a664644`, to two reference+1
items (Ctrl+Tab cycling, Switch to Work) plus the guard-honesty residuals that
WO-P2-012 already owns. The PR's original P3 shortcut deltas (Clear terminal
Ctrl+L/Ctrl+K, font-size Ctrl+±/0, Toggle file tree Ctrl+Shift+E —
post-baseline docs-derived, intro dates `[unverified]`) remain **P3** unchanged
(PR §8.2 P3 list; PR §5.10 Keyboard row).

## 4. In-flight work streams (not double-counted as P2/P3)

1. **WO-P2-012 — guard honesty for the six evidenced silent no-op states +
   executor fall-through tests.** Unmerged branches exist:
   `origin/feat/wo-p2-012-guard-honesty` (`0547055`) and
   `origin/feat/wo-p2-012-guard-honesty-r2` (`4fc763a`); diff vs base is
   +331/−2654 (code +246/−14 across ui.rs + lib.rs; the deletions are evidence
   files that postdate the branch's older base — the branch must be
   rebased/reconciled per PM roadmap L72–73). Added guards observed in the
   branch diff: "Select a chat before archiving it." / "…pinning or unpinning
   it." / "…renaming it." / "The selected chat is no longer available." /
   "A Git workflow is already running." / composer review-unavailable status.
   This wave: Workers A/B own the product-code lanes; this packet records, it
   does not patch.
2. **Activity-view surface** (deferred `toggleActivityView` target). The
   unread-attention state + four bindings landed (WO-P2-008, PR #23 →
   `00a3392`): `toggleThreadUnread` Ctrl+Shift+U, `nextUnreadChat` Ctrl+Alt+A,
   `clearAllUnread` Shift+Escape, `toggleActivityView` Ctrl+Alt+U (honest
   guidance while the view is absent) — ui.rs:2980/3057/3064/3071. The view
   surface itself is assigned to Workers A/B this wave (roadmap L74–75);
   reference view shape is auth-walled `[unverified]` (FW WO-R-REF R1).

## 5. Reconciliation counts

Row-level (primary class — the acceptance-criterion arithmetic):

| Total rows | closed | in-flight | bounded | P2 | P3 |
| --- | --- | --- | --- | --- | --- |
| **41** | **5** | **1** | **17** | **10** | **8** |

- closed (5): Runtime bootstrap; Model, effort, and speed picker; Marketplace
  admin-disabled install; Feedback; Active keyboard shortcut reference.
- in-flight (1): Keyboard and accessibility (WO-P2-012 guard residuals are its
  headline open remainder; its other items are P2/platform-bound, itemized in
  §2 row 38).
- bounded (17): Approvals and user input; Process Manager; Scheduled tasks;
  Plugins marketplace; MCP and apps; Sites; Visualizations; Appshots; Settings;
  Remote control and SSH; Cloud environments; Voice; Personalization and
  memory; Pets and Codex Micro; Import and migration; Windows; Linux.
- P2 (10): Projects and chats; Composer; Permission profiles; Repository
  status; Computer Use (Windows slice); In-app browser; Skills; Account and
  usage; Notifications and tray; First run and updates.
- P3 (8): App-server supervision; Thread execution; Streaming timeline; Diff
  review; Branches and worktrees; Pull requests; Terminal; Artifacts and files.

41 = 5 + 1 + 17 + 10 + 8 ✓

Status-column cross-check: `done` 4 + `partial` 29 + `missing` 7 + `platform` 1
= 41 ✓ (done rows 1/19/29/39 → closed; row 6 partial → closed via "none
listed"; the 7 `missing` rows → all bounded; the 1 `platform` row → bounded).

Coarse item-level tally across the per-item classifications in §2 (a row
contributes its remainder items, not its primary class): ~34 P2 items, ~56 P3
items, ~43 bounded items, 1 explicitly-closed item (manual project ordering)
plus the row-level closures, 2 in-flight streams, and 11 finding-16 shortcuts
(9 closed, 2 P3). These are counting aids, not gates.

Cross-reference to PR §8.1 (the parity report's own 53-row census — a
different row universe that adds audit-surfaced rows like side chats,
WebMCP, Record & Replay, in-app editing): zero contradictions observed — every
PR `deferred` row maps to a bounded PM remainder here, and PR P2/P3 splits are
preserved with layer notes where this inventory's baseline-26.721 rule
re-slices them (rows 10, 23, 37).

## 6. Source-unverifiable / honesty notes

1. **Full reference command inventory not enumerable** (PR WO-R-REF R3): the
   "remaining stable commands" P2 item cannot be enumerated from repo
   evidence; runtime evidence against an authenticated reference would settle
   the true delta.
2. **Reference exposure of row tails 10/12/13/17 is taken from the matrix's
   own "Reference contract still required" column** — this inventory does not
   independently re-derive the 26.721 behavior for "pull", "repository
   picker", "local-preview comments", or "official plugin discovery"; an
   authenticated reference pass would confirm.
3. **Projects-and-chats "unread state" staleness**: PM row text lists "unread
   state" as remaining while WO-P2-008 (merged) delivers per-chat
   unread-attention state — the Lead should reconcile the row text (Lead-owned
   doc; out of this packet's scope to edit).
4. **26.727/26.825 re-slicing**: items carried as P2 by the parity report but
   P3 under this inventory's pinned-baseline rule are flagged in rows 10, 23,
   37 (and PR §8.2 items 7–16 generally) — the Lead decides which layer the F1
   gate freezes.
5. **WO-P2-012 branch content**: inspected read-only via git; the branch's
   older base makes the raw diff misleading (evidence-file deletions); the
   merge decision belongs to the Lead/Workers A/B.
