# RWO-022-R3 — Adversarial Challenge of the Wave-S Closures (WO-REVIEW-001 deep phase)

- **Task ID:** RWO-022 (adversarial third of WO-REVIEW-001) — executed as the
  **third independent dispatch** (run label R3).
- **Base:** `main` @ `dbee3612bbe1aab200099e1b998d38b73d4b897b` (verified:
  `git rev-parse dbee3612^{commit}` resolves; it is an ancestor of the
  current remote `main` HEAD `5516084`, 9 commits back).
- **Branch:** `research/rwo-022-r3` (from the base SHA exactly).
- **Deliverable path note (deviation, recorded honestly):** the dispatch
  guide names `docs/research/evidence/rwo-020/adversarial.md`. That path is
  already occupied on `main` by the **first** RWO-022 run (`17a1897`, merged
  via PR #29 → `230f6fc`), and a **second** independent run exists on the
  pushed-but-**unmerged** branch `research/rwo-022-r2` (`cc80d30`, deliverable
  `docs/research/evidence/rwo-022/adversarial.md`, 546 lines). To preserve
  both prior records without clobbering, this run writes to
  `docs/research/evidence/rwo-022-r3/adversarial.md`, following the repo's own
  suffixed-re-run convention (`rwo-020` → `rwo-020-r2`, `rwo-021` →
  `rwo-021-verified`, `rwo-022` → `rwo-022-r2`).
- **Method:** every claim below was re-derived from **primary evidence
  only** — the source at the cited SHAs (`git show <sha>:<path>`), the
  actual test bodies, the actual evidence files (byte/md5 re-hashed in this
  session), and **fresh VLM reads of the evidence frames taken by this
  worker** (z-ai vision, glm-5v-turbo — independent of the recorded
  `vlm*.json` files). Ledger/parity prose was treated as the claim under
  attack, never as evidence.

## Verification bounds (honest NOT RUN list)

1. **No Rust toolchain in this sandbox** (`cargo`/`rustc` absent). No test
   was executed; every test claim was verified by reading the test bodies
   and by exact counting of `#[test]` markers / diff contents at the cited
   SHAs. Test **execution** results (e.g. "221/221 green", "CI double
   matrix") are NOT RUN here.
2. **Official-side numbers not re-derivable:** override 8's 126-vs-89
   method count concerns the official app's runtime schema; no official app
   exists in this sandbox. Internal consistency of the note was checked
   only.
3. **"605 existing tests unaffected" (WO-P2-005):** the exact figure is not
   reproducible from source markers (the workspace carries **631** `#[test]`
   markers at `aa3f7a1`, the PR #17 main parent; battery vs marker counting
   bases differ — RWO-021-r2's later full battery at `876bbe8` reports 639
   passed vs 667 markers at base). INCONCLUSIVE on the number; the
   grep-verification component was independently reproduced (no test
   references the picker flag at `2e6358f`).

## Dispatch context (binding for the Lead)

This is the third execution of the RWO-022 guide against the same base
SHA. State of the prior two runs:

- Run 1 (`research/rwo-022` @ `17a1897`): merged to main via PR #29 →
  `230f6fc`; findings FW-1..FW-8 remediated in `aedb771`.
- Run 2 (`research/rwo-022-r2` @ `cc80d30`): **pushed but NOT merged** —
  `git merge-base --is-ancestor cc80d30 origin/main` fails; its findings
  (N-1/N-2/N-3) appear nowhere on `main` (grep of both governing docs:
  zero hits), yet `main` HEAD (`5516084`) records **WO-REVIEW-001 CLOSED**.
  See R3-2/R3-3 below — this run independently re-derived and **confirms**
  N-1, and flags the unmerged record as a closure-integrity gap.

---

## T1 — WO-P2-005 (fork-picker fix + "15-site picker-lifecycle audit") — **HELD**

### Claims re-derived from primary evidence

- **Commit chain:** `2e6358f` (impl) → `4726dd4` (picker fix, `ui.rs +30/−1`)
  → merged `e46ad4f3df` (PR #17, parents `aa3f7a1` + `4726dd4`) — all
  resolve; `2e6358f` and `4726dd4` are the only commits on the PR branch. ✓
- **The fix is real and testable:** `4726dd4` replaces the change-subscription
  gate `value.trim() != "/fork"` with `!composer_keeps_fork_picker(&value)`;
  the predicate (`trimmed == "/fork" || trimmed == "/worktree"`) plus
  regression test `fork_picker_stays_open_for_both_fork_destination_commands`
  (both positives, trimmed positives, 8 negatives) read in the actual diff. ✓
- **"15-site picker-lifecycle audit":** `composer_fork_picker_open` occurs at
  **exactly 15 sites** in `ui.rs` @ `e46ad4f3df` (declaration 5270,
  subscription gate 5673, init 6436, closes 7245/7887/7947/7981/8030/8081,
  reads 7737/7745, Escape arm 22313–22319, render gate 23232). The audit's
  object is source-derivable and the lifecycle is coherent: the open path
  (`open_fork_destination_picker`) dispatches `ComposerChanged` then sets the
  flag; the render gate (`show_fork_picker = … && (text == "/fork" ||
  text == "/worktree")`) matches the predicate semantics exactly; Escape
  resets the composer to `/`. ✓
- **The four commands + guards:** executor arms for `/approve` (7678),
  `/fast` (7726), `/worktree` (7733), `/personality` (7816); availability
  mapping (44784–44788: `/mcp|/personality|/project|/status` unguarded,
  `/approve`/`/fast`/`/worktree` guarded); menu rows; prefix-resolution
  tests at 47667+. ✓
- **Slash-inventory arithmetic (override 1 + §5.2 row):** at `f113515` the
  named-command array is `[&str; 16]` (baseline 15 + `/review`) — override
  1's count is exact. At base the array is `[&str; 21]` (15+1+4+`/side`) —
  the §5.2 Composer row's "native named slash set of 21" is exact. The
  current-target enumeration (24 names) and "11 missing literal names" were
  re-counted by script: 24 names enumerated, 11 missing vs the 16-set —
  numbers add up. ✓
- **GUI evidence (D8c4), fresh VLM reads by this worker:** first probe asked
  for a list "below the composer" and found none — the picker renders
  **above** the composer as a suggestion card. Detailed probe of
  `d8c4-006-picker-still-open.png` (the regression gate, 2.5 s settle):
  composer `/worktree`; a card titled **"Continue in new worktree"** with
  subtext "Create a copy of your local project to work in parallel" is
  rendered above the composer — the picker stayed open (the fix works).
  `git-worktree-list.txt` shows the fork actually created a worktree
  (`…/worktrees/18d63946ceb41895/repo`), corroborating the Enter-#2
  dispatch. ✓
- **"grep-verified no test touches the picker flag":** reproduced — at
  `2e6358f` no occurrence of `composer_fork_picker_open` lies in the test
  module (last site 23232). ✓

### BROKEN (record/evidence hygiene — none affect the closure verdict)

- **Unannotated byte-identical duplicate (confirms stranded r2 finding N-2,
  open on main):** `d8c4-007-fork-dispatched.png` ≡ `d8c4-008-aftermath.png`
  (md5 `23925833ed3bd5ecdca9ce15a67b7f29` both). The "aftermath" label
  implies a second distinct observation; the bytes are identical, so frame
  008 carries zero additional information.
- **No md5 manifest at base** (remediated on main by `aedb771` FW-8 —
  verified present in run 1's remediation record).
- **Script self-mislabel (cosmetic):** `d8c4-wo-p2-005-happypath.sh`'s header
  comment reads "Scene D8c2" while every capture/file name is `d8c4-*`.

**Verdict: HELD** — the closure's load-bearing claims (fix predicate,
regression test, 15-site object, four guarded commands, GUI regression gate)
all survive primary-evidence re-derivation, including this worker's own VLM
read of the gate frame.

---

## T2 — WO-P2-006 (side-chat selection / active-turn semantics) — **HELD on substance; three evidence-integrity defects (all independently re-derived)**

### Claims re-derived from primary evidence

- **Diff discipline:** `764e4db` = `ui.rs +381`, `codex-core/src/lib.rs +469`
  — the ledger's "exactly the two prescribed files (ui.rs +381, codex-core
  lib.rs +469)" is digit-exact. ✓
- **Registry arithmetic:** at `5287c29f3f`,
  `KEYBOARD_SHORTCUT_COMMAND_IDS: [&str; 72]` contains `openSideChat`
  (71→72 exact); `MAX_KEYBOARD_SHORTCUT_COMMANDS` is still 71 at this stage —
  the pre-008 quirk state, consistent with the WO-R-SWEEP record. ✓
- **Binding wiring:** `OpenSideChatShortcut` action (1639),
  `KeyBinding::new(&shortcut("alt-s"), …)` (4640), registry item
  `openSideChat` / "Open side chat" / Thread group / `CmdOrCtrl+Alt+S`
  (2795), interceptor arm (10401), and the exact-one-owner binding test
  `side_chat_binding_resolves_ctrl_alt_s_to_the_side_chat_action`
  (owners == `["openSideChat"]`, registry membership, reducer opens side
  chat with selection untouched). ✓
- **Selection/active-turn semantics — the state test is NOT weaker than
  claimed:** `a_side_chat_first_message_creates_a_projectless_thread_without_selecting_it`
  asserts: submit dispatches `Effect::CreateTask` (projectless, message
  "quick aside"); **`selected_task_id` stays "t1"**; **t1's
  `active_turn_id` stays "turn-1"**; the main composer draft "main draft" is
  preserved; after `NewChatTaskCreated` the side chat claims "side-1"
  **without selection moving**; a stale creation ("other-1") is never
  claimed. Every element of the ledger phrase "side submit creates the side
  thread without selecting it; main-chat selection + active turn untouched
  (state test)" is literally asserted. ✓
- **Guard symmetry:** `/side` availability = `selected_task_id.is_some()`;
  `Action::OpenSideChat` with no selection renders the honest status "Open a
  chat before starting a side conversation." — both entry paths guard
  identically, matching the recorded test comment. ✓
- **Test count:** `#[test]` count in `lib.rs` @ `5287c29f3f` = **221**
  (217 at the pre-006 merge `e46ad4f3df`) — "core tests 221/221" count is
  source-exact (execution NOT RUN: no cargo). ✓
- **Close path (D9b close-probe, re-derived):** the probe script clicks four
  candidate header positions; the recorded VLM read shows the panel closed
  and the main view expanded after the click at (1418,140) — a real,
  probed close affordance. In source, `CloseSideChat` is dispatched only
  from the panel's close **button** (`side-chat-close`, `on_click` at 17859).
  ✓ (this source fact is also what breaks the D9 labels below).

### BROKEN (evidence-integrity; independently re-derived, confirm run-1 FW-6 / run-2 findings)

- **`d9-008-side-closed.png` is mislabeled:** this worker's fresh VLM read
  shows the side panel **still open** ("Side chat" header visible), main
  chat still selected, error banner present. The D9 script's step 7 presses
  **Escape** under the comment "close the side panel (Escape)" — but Escape
  does not close the side panel (source: only the header button dispatches
  `CloseSideChat`). The frame documents an Escape no-op, not a close.
- **Consequence — reopen criterion (e) is not evidenced by D9:** because the
  panel never closed, `d9-010-side-reopened.png` (fresh read: panel open,
  main chat selected, no slash row visible) shows the panel **remaining**
  open through `/side` — an idempotent no-op — not a reopen-from-closed.
  The closure's criterion (e) ("`/side` reopens it") rests on source only
  (`OpenSideChat` sets `side_chat` when `None`).
- **Duplicate frame:** `d9-006-side-submitted.png` ≡ `d9-007-aftermath.png`
  (md5 `8d733e08fce19256e60ba128c36aaa10` both) — "aftermath" adds nothing.
- **Coverage note:** the recorded `vlmd9a.json` VLM-read covers frames
  002/004/005/007 only; frames 008/009/010 were never VLM-read in the
  recorded evidence (consistent with them being the weak/mislabeled set).

**Verdict: HELD on substance** (binding, registry arithmetic, state-test
strength, guard symmetry, close path via D9b + source) — **BROKEN on the
frame labels** (d9-008 mislabel; d9-006≡007 duplicate; d9-010 weaker than
its label). Run 1's remediation `aedb771` (FW-6: wo-p2-006 README close
affordance + mislabeled/duplicate frame annotations) is the right class of
fix; the labels themselves remain as historical artifacts.

---

## T3 — WO-P2-007 (Ctrl+P silent no-op fix + F-A4 no-workspace residual) — **HELD**

### Claims re-derived from primary evidence

- **Base defect claims are line-exact:** at `3c9f113`, `OpenFileSearch` is
  declared at `ui.rs:1636`, bound at `ui.rs:4636`
  (`KeyBinding::new(&shortcut("p"), OpenFileSearch, None)`), and **zero**
  `on_action`/`Action::OpenFileSearch` handlers exist anywhere; the
  interceptor registry carries `searchFiles` with a live dispatch arm at
  `ui.rs:10480` (`open_command_palette(PaletteMode::Files, …)`). Every
  anchor in the ledger row and §9 override 17 matches the source at the
  cited SHA, character for character. ✓
- **The fix:** at `a3c0e01` the action and binding are gone (the sole
  remaining "OpenFileSearch" mention is inside the regression test's
  comment); `2948bf2` = `ui.rs +40/−2` exactly as claimed. ✓
- **The 5-assertion regression test**
  (`ctrl_p_routes_to_the_search_files_command`) read in full: (1)
  exact-one-owner `owners == ["searchFiles"]` for `CmdOrCtrl+P`; (2)
  `KEYBOARD_SHORTCUT_COMMAND_IDS.contains("searchFiles")`; (3)
  `PaletteCommand::SearchFiles.shortcut_command_id() == Some("searchFiles")`;
  (4) advertised shortcut `Ctrl+P`; (5) registry metadata (title "Search
  files", `CmdOrCtrl+P`, General group). Exactly the five asserted elements
  the ledger enumerates — not weaker. ✓
- **D10/D10b md5 chains re-hashed this session:** d10-01≡02≡03 =
  `bb5139b3d119c06db7009c3e663cfd1c` (byte-identical defect proof on
  unfixed main); d10-04 = `c8c351fb…` (palette route works); d10b ws-01 =
  `a9ad7e0c…` → ws-02 = `857b824c…` (differs — palette opened) → ws-03 ≡
  ws-01 (closed cleanly). All manifest entries match the actual bytes. ✓
- **Fresh VLM read of `ws-02-after-ctrl-p.png` by this worker:** centered
  search overlay; input placeholder **"Search files"**; **"Files"** section
  with "Type to search for files"; focused cursor visible. Matches the
  recorded `vlm-ws02-read.md` and the ledger's claim verbatim. ✓
- **F-A4 residual is honest and source-exact:** `open_command_palette`
  early-returns when `mode == Files && !has_local_workspace()` (at
  `a3c0e01`: lines 8498–8500) — the no-workspace silent state persists by
  design, is documented in the ledger row, the parity row, and the sweep
  (`input-surface-sweep.md` finding F-A4 with its own anchors). ✓

### BROKEN (one reproducibility-gap record defect)

- **Scene scripts not archived (confirms stranded r2 finding N-3, open on
  main):** the wo-p2-007 README cites `d10-ctrl-p-file-search.sh` and
  `d10b-ctrl-p-workspace.sh` as living in `parity-lab/scenes/` — outside the
  repository. Unlike wo-p2-005/006/008 (whose scene scripts ARE in-tree),
  the 007 evidence folder carries frames + manifests only. The scenes are
  documented-pointer-only; not reproducible from the repo.

**Verdict: HELD** — every load-bearing closure claim survives with exact
primary matches (source anchors, diff stats, md5 chains, fresh VLM read);
one open reproducibility gap (N-3).

---

## T4 — WO-P2-008 (no-runtime-lab limitation + MAX 71→76 supersession) — **HELD on substance; BROKEN on the record (count overclaim + scene defects)**

### Claims re-derived from primary evidence

- **Diff discipline:** `c37c21b` = `ui.rs +397/−18` + `lib.rs +349` = the
  claimed `+728/−18` across exactly the two prescribed files. ✓
- **MAX supersession:** at `00a3392`,
  `MAX_KEYBOARD_SHORTCUT_COMMANDS: usize = 76` **and**
  `KEYBOARD_SHORTCUT_COMMAND_IDS: [&str; 76]` — the WO-R-SWEEP MAX=71-vs-72
  quirk is cured exactly as override 18 records (registry now constant =
  array = 76). ✓
- **The four bindings:** registry items read in full — `toggleThreadUnread`
  "Mark chat unread" `CmdOrCtrl+Shift+U` (Thread); `nextUnreadChat` "Next
  chat needing attention" `CmdOrCtrl+Alt+A` (Navigation); `clearAllUnread`
  "Clear all unread indicators" `Shift+Escape` (Navigation);
  `toggleActivityView` "Toggle Activity view" `CmdOrCtrl+Alt+U`
  (Navigation). ✓
- **Core state tests — count and strength:** the diff adds **exactly 7**
  `#[test]` functions; the lead test's body asserts: background completion
  flags the chat; **a failed turn still flags** ("failing" joins the list);
  **the selected chat is never flagged**; **terminal events for unknown
  tasks never flag**. The other six cover approval marking, visit-clears,
  toggle round-trip with honest statuses, archive-drops, clear-all honest
  count ("Cleared unread indicators for 2 chats" / "No unread chats"), and
  activity-view honest guidance. The ledger's enumeration matches the
  actual test set one-for-one. ✓
- **Sidebar claims are source-exact:** the row renders a
  `.size(px(6.0)).rounded_full().bg(primary)` dot (the claimed 6 px) and
  `.font_weight(FontWeight::MEDIUM)` title when flagged. ✓
- **Jump semantics:** `next_unread_chat_follows_sidebar_order_cyclically`
  asserts cyclic wrap-around in both directions with pinned-first ordering;
  the binding test asserts exact-one-owner accelerators. ✓
- **D11b — fresh VLM reads of all four frames by this worker, verbatim
  statuses confirmed:**
  - `d11b-02` (Ctrl+Shift+U): **"Select a chat before marking it unread."**
  - `d11b-03` (Ctrl+Alt+A): **"No chats need attention."**
  - `d11b-04` (Ctrl+Alt+U): **"Activity view is not available yet. Use
    "Next chat needing attention" to jump to unread chats."**
  - `d11b-05` (Shift+Escape): **"No unread chats"**
  All four match the README table character-for-character. ✓
- **The no-runtime-lab limitation is honestly documented and real:** every
  D11/D11b frame (fresh reads) shows the sidebar with **"No chats"** and a
  **"Connection failed"** row (red dot) — the lab genuinely had no codex CLI
  runtime; the README says so explicitly and correctly scopes what the
  scenes can and cannot prove (dot-on-row / jump not GUI-exercisable,
  unit-covered). ✓
- **Duplicate-delivery adjudication is factually grounded:** `64b28f6`
  exists on `origin/feat/wo-p2-008-unread-attention` (not the merged
  verified branch), is leaner (+582/−19), and consolidates the tests — the
  "adjudicated redundant" record describes a real artifact. ✓

### BROKEN (record defects)

- **Count overclaim "6 codex-app binding tests" (confirms run-1 FW-1):** the
  diff adds **exactly 5** app-side `#[test]` functions
  (`activity_view_binding_resolves_ctrl_alt_u…`,
  `next_unread_chat_binding_resolves_ctrl_alt_a_and_jumps`,
  `clear_all_unread_binding_resolves_shift_escape`,
  `toggle_thread_unread_binding_resolves_ctrl_shift_u`,
  `next_unread_chat_follows_sidebar_order_cyclically`); no sixth test was
  added between `c37c21b` and the merge. The "6" appears at **five** doc
  sites at base: ledger L448 (Tests row), parity L260 (§5.10 row), parity
  L737 (§10 changelog), wo-p2-008 README ×2 ("binding/jump tests (6, CI)"
  and "binding ownership/metadata/dispatch (6 app tests, CI)"). Remediated
  on main by `aedb771` (FW-1) — re-derivation here confirms the finding and
  the five-site count.
- **D11 scene-script defect + mislabeled frames (confirms run-1 FW-5 class,
  open on main):** the D11 script submits composer text with plain **`Return`**
  (steps 02/04) although the calibrated lab recipe — recorded in the D8c4
  and D9 scripts themselves — is **ctrl+Return**; and the frame filenames
  describe states that never occurred on a runtime-less boot:
  `d11-02b-chatA-created`, `d11-04-chatB-created`, `d11-05-jumped-to-A`,
  `d11-06-remarked`, `d11-07-cleared` (fresh read of 03: "No chats" sidebar,
  honest no-selection status, nothing marked). The README prose is honest
  about the limitation, but the labels remain aspirational; the root-cause
  attribution ("no runtime") is incomplete — the wrong submit key is a
  second, independent defect in the same scene.

**Verdict: HELD on substance** (bindings, MAX 71→76 chain, 7 core tests,
state semantics, D11b verbatim statuses — all re-derived, including fresh
VLM reads) — **BROKEN on the record**: the 6-vs-5 count (remediated) and
the D11 script/label defects (open: FW-5/FW-7 class).

---

## T5 — §9 override trail + §8 counts vs rows — **HELD on substance; BROKEN on override 16 (NEW finding) + four stale cells (re-derived)**

### Counts and arithmetic (all mechanically re-derived)

- **Row census:** §5 contains exactly **53** data rows (scripted count).
  Backend-column census: complete **8**, partial **32**, missing **4**,
  platform-limited **2**, deferred **7** — **exactly** the §8.1 table, and
  the named row lists match one-for-one (8 complete incl. Side chats; 4
  missing; 2 platform; 7 deferred). The override-18 chain (complete 7→8 via
  WO-P2-006; missing 6→5→4, partial 31→32 via WO-P2-008) is internally
  consistent at every hop. **The §8 numbers add up against the rows.** ✓
- **Override 1:** 16 named commands at `f113515` — `[&str; 16]` verified. ✓
- **Override 2:** A §2's parenthetical says "(15)" while enumerating 14
  names; `/shell` is the 15th — the override records the discrepancy
  honestly rather than silently resolving it. ✓ (A §2's current-set
  enumeration re-counted by script: exactly 24 names.)
- **Override 11:** `f113515:ui.rs:4499` is exactly
  `KeyBinding::new("ctrl-`", ToggleTerminalShortcut, None)` — line-exact. ✓
- **Overrides 3/4/5:** reclassifications feeding WO-P1-003 / WO-P2-008 /
  WO-P2-005 — all three closed with evidence trees; consistent. ✓
- **Overrides 6/7/9/10/12/13:** row cross-checks — Terminal row stays
  partial with WO-P1-001 closure noted (no status flip); Plugins watch item
  recorded as BOUND with no WO; Multi-root (complete, regression control)
  and multi-folder (P1, WO-P1-003 CLOSED) both stand as distinct
  capabilities; Process Manager discoverability reads complete with the
  three affordances; Import row stays partial; Computer-Use Linux stays
  platform-limited/unverified. No contradiction between any of these
  overrides and the rows they cite. ✓
- **Overrides 14/15:** the `codex-linux/` evidence tree exists (9 frames +
  README + deb-metadata + official-app-log-extracts; the fatal
  "Unable to locate the Codex CLI…" string is present in the extracts). ✓
- **Override 17 + FIXED annotation:** base claims line-exact (see T3); the
  annotation's account of the fix matches `a3c0e01`. ✓
- **Override 18:** flip + counts consistent (above). ✓

### NEW — BROKEN: override 16 conflates the palette with the registry and contradicts override 17 (open on main)

Override 16 (verbatim, at base and — re-verified — **still verbatim on main
HEAD**): "The official overlay carries bindings **absent from Flauz's
45-command registry/palette**: Search Files Ctrl+P, Copy deeplink
Ctrl+Alt+L, Copy working directory Ctrl+Shift+C, Rename chat Ctrl+Alt+R,
Close Tab Ctrl+W, Archive chat Ctrl+Shift+A, New standalone chat
Ctrl+Alt+O, Toggle pin Ctrl+Alt+P, Back/Forward Ctrl+[/], recent-chat
cycling Ctrl+Tab/Ctrl+Shift+Tab, Switch to Work Alt+2."

Primary evidence at `f113515` (the C2 reconciliation base at which the
override was recorded):

- `ACTIVE_KEYBOARD_SHORTCUTS` (the persisted, customizable keyboard
  registry) contains **71 items** — including, with the exact chords listed:
  `searchFiles` → `CmdOrCtrl+P`, `copyDeeplink` → `CmdOrCtrl+Alt+L`,
  `copyWorkingDirectory` → `CmdOrCtrl+Shift+C`, `renameThread` →
  `CmdOrCtrl+Alt+R`, `closeWindow` → `CmdOrCtrl+W`, `archiveThread` →
  `CmdOrCtrl+Shift+A`, `newProjectlessTask` → `CmdOrCtrl+Alt+O`,
  `toggleThreadPin` → `CmdOrCtrl+Alt+P`, `navigateBack`/`navigateForward`.
  **Ten of the eleven listed "absent" bindings were present in the registry
  when the override was written.** Only recent-chat cycling
  (Ctrl+Tab/Ctrl+Shift+Tab) and Switch-to-Work (Alt+2) were genuinely
  absent (no such chords anywhere in the file).
- The **45** figure is the `PaletteCommand` enum count at `f113515` (45
  variants — verified by script). The palette did lack rows for
  copyDeeplink/copyWorkingDirectory/renameThread (the true gap, later
  closed by WO-P2-010), but it **had** `SearchFiles`.
- **Internal contradiction:** override 17 — written one entry later against
  the same base — states the interceptor registry "already carries
  searchFiles → CmdOrCtrl+P with a live dispatch arm" (and its FIXED
  annotation repeats "already carrying CmdOrCtrl+P"). Entries 16 and 17
  describe the same key oppositely: 16 says the binding is absent; 17 says
  it is present (and correctly locates the defect in the dead GPUI action).
  17 is the accurate account.
- **Staleness:** unlike 17, override 16 never received a post-remediation
  annotation; at base (and on main) it still misdirects readers on what the
  waves closed — WO-P2-007 fixed the direct-keypress routing and
  WO-P2-010 added the palette rows, so the entry's list is now wrong on
  both of its own readings.

**Assessment:** this is a record defect in the audit trail of record, not a
product-code defect — the underlying §5.10 row text is accurate — but the
§9 trail is explicitly the document future reconciliations trust, and one
of its entries misstates the facts it cites and survives un-annotated on a
CLOSED review program.

### BROKEN (stale cells — all re-derived at base; remediation state noted)

- **§5.2 Composer Gap cell (live row, self-contradictory at base):** "seven
  of the eleven absent names stay open" — after WO-P2-005 (+4) and
  WO-P2-006 (+`/side`) the correct count is **six** (`/cloud`,
  `/cloud-environment`, `/ide-context`, `/local`, `/pet`, `/task`).
  Remediated on main (`aedb771` FW-2; main now reads "six of the eleven").
- **§8.2 item 12 (live priority list, stale):** "Browser address-bar history
  / Google fallback (26.727)" carries no closure annotation, while the §5.4
  row records the history/Settings part CLOSED (WO-P2-009) with the
  Google-fallback residual. Remediated on main (FW-3).
- **§8.1 census prose (stale):** the paragraph below the counts table still
  narrates "The 6 `missing` rows are … (side chats, … Activity view)" —
  both named rows have since flipped (missing is now 4). Historical prose,
  but unannotated at base. Remediated on main (FW-4).
- **§-number drift:** closure records direct readers to wrong sections:
  **"§5.3 Composer"** ×4 (parity L449 — a **live** §8.2 item-5 pointer —
  plus changelog L730/L731; ledger L669; Composer is §5.2), **"§5.3 Side
  chats"** ×3 (parity L732/L733; ledger L671; Side chats is §5.1), and
  **"§5.9 Keyboard row"** ×1 (parity L735; Keyboard and accessibility is
  §5.10). My mechanical count is **8 sites**; run 1's remediation (FW-4)
  reports 7 §-reference fixes — the delta is classification (one live
  pointer vs changelog entries), not substance. Remediated on main.
- **"51-command registry" (live §5.10 row, stale count):** at base the
  palette has **70** `PaletteCommand` variants and the keyboard registry
  **76** items; "51" was the post-WO-P2-004 palette size, never updated
  through the 008/010 growth. Remediated on main by RWO-021 (`ad9685c`,
  KA-3: "51→70 palette-registry count, verified at ui.rs:3480").

### INCONCLUSIVE

- **Override 8 (126 vs 89 experimental methods):** the 89 count is an
  official-runtime observation; not re-derivable in this sandbox. The note's
  internal logic (runtime outranks historical for the reproducible number)
  is consistent; no Flauz-side claim depends on it.

**Verdict: HELD on substance** (census exact; override chain arithmetic
exact; overrides 1–15, 17, 18 verified against rows/source) — **BROKEN on
override 16** (NEW, open on main) **plus the four stale-cell classes**
(re-derived; all remediated on main except where noted).

---

## EQ-1 (D9 vs D11 lab-runtime discrepancy) — **RESOLVED, re-verified with this worker's own fresh VLM reads**

- D8c4/D9-era boots: my detailed read of `d8c4-006` shows the sidebar footer
  **"App-server online"** with a green dot (and D9 frames show chats created
  in the sidebar — runtime present).
- D11/D11b-era boots: my fresh reads of `d11-03` and all four `d11b-*`
  frames show **"Connection failed"** with a red dot and **"No chats"**.
- Conclusion (independently reaching the same resolution as both prior
  runs): real lab environment drift between boots, not fabrication. The
  D11/D11b honest statuses are runtime-independent reducer outputs and are
  verbatim-verified; the no-runtime state is documented in the README and
  correctly bounds what those scenes prove.

---

## NEW findings beyond both prior runs

### R3-1 — §9 override 16 registry/palette conflation + 16↔17 contradiction (BROKEN; **open on main**)

Full derivation in T5. Neither run 1 (whose T5 findings were the four stale
cells) nor run 2 (whose T5 verdict lists the same four cells) nor any
main-HEAD remediation (`aedb771`, `ad9685c`, `5516084`) touches override
16; the entry is verbatim-identical on main. Suggested remediation: annotate
override 16 in place (preserving history per AGENTS.md "relabel unclear
history; do not erase it") — e.g. note that the 45-command figure is the
palette, that ten of the listed bindings were already registry-present at
`f113515` (see override 17 for the accurate Ctrl+P account), that
WO-P2-007/010 closed the actual gaps, and that only recent-chat cycling
(Ctrl+Tab/Ctrl+Shift+Tab) and Switch-to-Work (Alt+2) remain chord-absent.

### R3-2 — WO-P2-009 "reload/copy-URL keybindings" scope overclaim (BROKEN; **confirms stranded run-2 finding N-1; open on main**)

Independently re-derived from the diffs, not from run 2's report: the
WO-P2-009 ledger row ("Scope delivered … reload/copy-URL keybindings",
L459) and §7 changelog (L677) claim keybindings that no commit in the
lineage delivers — PR #24 (`5302e7e`) adds exactly two `KeyBinding::new`
entries, both **ClearBrowsingHistoryModal focus-navigation** (tab /
shift-tab); PR #27 (`fe3903e`) adds none; and the registry delta
`f113515 → base` is exactly the five 006/008 attention/side commands — no
reload or copy-URL command ever existed. The phrase appears to have been
copied from the WO-R-REF R2 *scoping* text ("the real gap is … reload/copy-
URL keybindings") into the *delivered-scope* line without implementation.
**Still open on main** (ledger L459 and L677 carry it verbatim; RWO-021's
verification leg did not flag it; run 2's branch carrying the finding is
unmerged).

### R3-3 — Run 2's deliverable is unmerged while WO-REVIEW-001 is CLOSED (process/closure-integrity gap)

`research/rwo-022-r2` (`cc80d30`, base `dbee3612` exactly, +546-line
deliverable) is pushed but unmerged; `main` HEAD `5516084` records
WO-REVIEW-001 CLOSED with zero mention of run 2 or its findings
(N-1/N-2/N-3 appear nowhere in the governing docs on main). The Lead should
harvest and adjudicate that branch (its N-1 is confirmed by this run; N-2
and N-3 are also independently confirmed here) so the closure record
accounts for all dispatched legs.

### Confirmed-prior (independently re-derived by this run, not trusted from the reports)

N-2 (d8c4-007≡008), N-3 (007 scene scripts absent), FW-1 (6→5 tests, five
sites), FW-2 (seven→six), FW-3 (§8.2 item 12), FW-4 (census prose +
§-references), d9-008 mislabel, d9-006≡d9-007, D11 plain-Return defect +
aspirational frame labels, EQ-1. The "605 existing tests" figure (T1) is
INCONCLUSIVE as recorded (battery-vs-marker counting bases differ; 631
`#[test]` markers at `aa3f7a1`).

---

## Verdict table

| Target | Verdict | Key defects (state) |
| --- | --- | --- |
| T1 WO-P2-005 picker fix + 15-site audit | **HELD** | d8c4-007≡008 duplicate (open); no manifest at base (remediated); D8c2 script self-label; "605" INCONCLUSIVE |
| T2 WO-P2-006 side-chat semantics | **HELD on substance** | d9-008 mislabel + d9-006≡007 duplicate + d9-010 idempotent-reopen (README remediated; labels historical) |
| T3 WO-P2-007 Ctrl+P fix + F-A4 | **HELD** | scene scripts not archived (open, N-3) |
| T4 WO-P2-008 no-runtime-lab + MAX 71→76 | **HELD on substance** | 6-vs-5 count (remediated); D11 Return-key defect + mislabeled frames + incomplete root-cause attribution (open, FW-5/FW-7 class) |
| T5 §9 overrides + §8 counts | **HELD on substance** | **override 16 conflation/contradiction (NEW, open)**; four stale-cell classes (all remediated on main) |
| EQ-1 lab-runtime drift | **RESOLVED** | — (fresh VLM reads both sides) |

**Zero product-code defects** — consistent with both prior runs. Every
BROKEN item is a record/evidence-hygiene defect or a scope-overclaim in
prose; none changes a closure verdict on its substance.

## Follow-up work-order candidates (from this pass)

- **FW-R3a** (docs): annotate §9 override 16 in place (registry vs palette;
cross-reference override 17; note the 007/010 closures and the two
genuinely-absent chords). Small, bounded, closure-record integrity.
- **FW-R3b** (docs): correct the WO-P2-009 "Scope delivered" line (ledger
L459 + changelog L677) — strike or implement the reload/copy-URL
keybindings claim; if implemented, scope it as its own bounded order
against the official Ctrl+R/Ctrl+Shift+R reference (§5.4 already cites the
overlay rows). (Subsumes stranded run-2 FW-N1.)
- **FW-R3c** (process): harvest/adjudicate `research/rwo-022-r2` into main's
closure record (or explicitly supersede it with this run's confirmations).
- Carried from prior runs (still open on main): FW-5/FW-7 (lab: D11 script
key fix + codex-runtime pin/restore for the full-flow scene), N-2/N-3
evidence-hygiene annotations, FW-9 (→ WO-P2-012 scope).

## Token/credential note

No token material appears in this deliverable, the branch, or any file in
the repository; the clone credential existed only on the git command line.
