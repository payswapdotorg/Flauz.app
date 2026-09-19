# RWO-022 — Adversarial challenge of the P2 closure set (WO-REVIEW-001, Worker C)

- **Task ID:** RWO-022 (adversarial third of WO-REVIEW-001 deep phase)
- **Base:** `main` @ `dbee3612bbe1aab200099e1b998d38b73d4b897b` (verified: cloned as
  `origin/main` HEAD; branch `research/rwo-022` created from it; the closure
  head carries PR #24/#27 (WO-P2-009), PR #25 (WO-P2-010), PR #26 (RWO-020
  rubric) as advertised).
- **Method:** every claim below was re-derived from **primary evidence only**:
  git objects at the cited SHAs (`git show`/`git diff`/ancestry checks), the
  source at the review base, recomputed md5s of the evidence frames, and
  **fresh VLM reads of the archived frames** (this worker re-read the PNGs
  itself; archived VLM transcripts were used only for cross-comparison, never
  as the sole source). Ledger/parity prose was never trusted as evidence.
- **Scope attacked:** the five targets named by the work order. WO-P2-009/010
  closures were not deep-attacked (out of the assigned five) except where
  their rows interact with the targets.

## Verification bounds (honest NOT RUN list)

1. **No Rust toolchain in this sandbox** (`cargo`/`rustc` absent; the repo pins
   toolchain 1.97.1). Therefore every *test-run* claim — "221/221 core tests",
   "605 green (406 + 199)", "7/7 core tests per-test verified", "CI green both
   matrices", "fmt gates pass on pinned 1.97.1" — is **NOT RUN** here. What was
   done instead: the *existence, count, and assertion strength* of every cited
   test was verified statically at the cited SHAs (bodies read, not just
   names). Static corroboration where noted.
2. **The LINUX_GUI_LAB is not re-runnable here.** Scene scripts reference
   `/home/z/parity-lab/...` hosts and donor DBs that do not exist in this
   sandbox. What was done instead: md5 recomputation of every archived frame
   against the in-tree manifests, plus fresh VLM reads of the frames the
   closures rest on.
3. CI run records (GitHub Actions logs) are not archived in-repo; "CI green
   both matrices" remains a recorded claim, not re-verified.

---

## T1 — WO-P2-005 (fork-picker fix + "15-site picker-lifecycle audit") — **HELD**

Closure claims attacked (ledger L668, parity L730): four guarded slash
commands; picker wiring defect fixed in `4726dd4` via the testable
`composer_keeps_fork_picker` predicate + regression test; "15-site
picker-lifecycle audit clean"; "grep-verified no test touches the picker
flag"; D8c4 GUI evidence.

**Held, with primary evidence:**

1. **The four commands + guards exist at main** (`crates/codex-app/src/ui.rs`):
   `/approve` (guard: `selected_approval_request(&self.state).is_none() →
   return false`), `/fast` (guard: `fast_service_tier_id(&self.state)`), 
   `/worktree` (guard: `has_local_workspace() && selected_task_id.is_some()`),
   plus menu-row availability bits (`slash_availability.worktree` etc.) at the
   render gate. Commit `2e6358f` (ui.rs +394/−132) is an ancestor of merge
   `e46ad4f3df` (PR #17), as is fix `4726dd4` — topology verified.
2. **The fix is exactly as described** (`git show 4726dd4`, ui.rs +30/−1): the
   composer Change-subscription close condition `value.trim() != "/fork"` was
   replaced by `!composer_keeps_fork_picker(&value)` where the predicate is
   `trimmed == "/fork" || trimmed == "/worktree"`; the regression test
   `fork_picker_stays_open_for_both_fork_destination_commands` asserts both
   commands, trimmed variants, and 7 negative cases. Test present at main.
3. **The "15-site" audit re-derives from source and holds.** The 15 sites are
   the 15 occurrences of `composer_fork_picker_open` in ui.rs at `4726dd4`
   (still 15 at main; line drift 5636/6051/6836/7684/8188/8196/8339/8399/8433/
   8472/8482/8533/23062/23068/23984 at base). Re-derivation of each site's
   lifecycle role: declaration (5636); init false (6836); Change-subscription
   close **via the new predicate** (6051); selection-change reset (7684);
   `/worktree` dispatch-on-open (8188); `/fork` dispatch-on-open (8196);
   mutual-exclusion closes in `open_composer_review_submenu` /
   `open_mcp_slash_status` / `show_project_picker` / `open_composer_status`
   (8339/8399/8433/8533); the setter (8472); the submit-path close
   `select_fork_slash_destination` (8482); Escape close + composer reset to
   "/" (23062/23068); the render gate `composer_fork_picker_open &&
   (composer_text == "/fork" || composer_text == "/worktree")` (23984). All 15
   coherent; no site bypasses the predicate's semantics.
4. **"No test touches the picker flag" re-verifies:** all 15 sites lie outside
   the `mod tests` region; the only test references use the predicate, not the
   flag.
5. **D8c4 GUI evidence re-read (fresh VLM):** frame 005 (picker-open): composer
   `/worktree`, inline row "Continue in new worktree", retry toast; frame 006
   (picker-still-open, the 2.5 s regression gate): **picker still rendered** —
   PASS; frame 007 (fork-dispatched): picker gone, "Work locally" tag, sidebar
   shows the second `repo` entry — matches `git-worktree-list.txt` (two
   worktrees @ `97f6b70`, one detached). Archived `vlmd8c4.json`/`vlm008.json`
   descriptions are consistent with my re-reads.
6. Static corroboration of counts: codex-core `#[test]` count at `764e4db` is
   exactly **221** (matches "221/221" as a count; pass status NOT RUN).

**Caveats recorded (not closure-breaking):**

- The "15-site audit" has **no archived artifact** — no doc, no per-site table
  anywhere in the tree (grep: `15-site`/`picker-lifecycle` appear only in the
  two changelog sentences). The claim is *re-derivable* (this review did), but
  a Lead re-verifier has nothing to diff against except the source itself.
  → follow-up FW-9.
- The WO's test plan asked for "slash-executor unit tests for each new
  command"; the delivered tests (`parity_slash_commands_resolve_with_their_guards`,
  `fast_slash_command_follows_the_selected_model_catalog`, etc.) cover the
  **resolution/guard layer** (`composer_slash_command_for_prefix` +
  availability bits), not the executor's guard **fall-through** (guard failure
  `return false` → the typed text is submitted as a visible chat message —
  visible, not silent, but untested at unit level and only happy-path
  GUI-evidenced).
- `wo-p2-005/` evidence has **no md5 manifest** (unlike 007/008 dirs) — frames
  not hash-anchored.
- "605 existing tests unaffected (grep-verified)": the 605 total is not
  statically re-derivable with confidence (first-party `#[test]` greps give
  ~627 including platform-gated tests; cargo is the only authoritative
  counter) — **NOT VERIFIED, NOT REFUTED** (no toolchain).
- Script quirk: `d8c4-wo-p2-005-happypath.sh` line 2 self-identifies as
  "Scene D8c2" while everything else names it D8c4.

**Verdict: HELD** (closure claim survives adversarial re-derivation; three
minor evidence-hygiene caveats above).

---

## T2 — WO-P2-006 (side-chat selection / active-turn semantics) — **HELD**, with two evidence-integrity defects

Closure claims attacked: Ctrl+Alt+S binding via registry command
`openSideChat` (ids 71→72); guarded `/side` (21 named commands); side submit
creates the side thread **without selecting it**; main-chat **selection and
active turn untouched (state test)**; close returns to the main view; exactly
two prescribed files (ui.rs +381, core lib.rs +469); D9/D9b GUI evidence.

**Held, with primary evidence:**

1. **Registry arithmetic exact:** `KEYBOARD_SHORTCUT_COMMAND_IDS` counts 71 at
   `f113515`, **72 at `764e4db`** (`openSideChat` added), 76 at main (after
   008). Diff confinement exact: `764e4db` = ui.rs +381, lib.rs +469, 2 files
   (ledger wording matches to the line). Ancestry `764e4db → 73eb55c →
   ddff97f → 5287c29f3f` (PR #18) verified.
2. **The state tests assert the full claimed semantics** (codex-core at main):
   `a_side_chat_opens_and_closes_without_touching_the_selected_chat` seeds an
   **active turn** (`active_turn_id = Some("turn-1")`), a main draft, then
   asserts through open/close: `selected_task_id` unchanged,
   `active_turn_id` unchanged, main composer unchanged, plus the honest
   no-selection guidance ("Open a chat before starting a side conversation.").
   `a_side_chat_first_message_creates_a_projectless_thread_without_selecting_it`
   asserts the `Effect::CreateTask` (projectless), and after
   `NewChatTaskCreated` that `side.task_id == "side-1"` while
   `selected_task_id` stays `"t1"` **and** the active turn survives. The app-side
   binding test (`side_chat_binding_resolves_ctrl_alt_s_to_the_side_chat_action`)
   asserts sole accelerator ownership + registry membership + reducer
   open-without-selection. These are strong tests — no weaker-than-claimed
   assertion found.
3. **Named command count exact:** the prefix resolver at main indexes exactly
   **21 named commands** (15 baseline + `/review` + 4 from 005 + `/side`) —
   matches the Composer row's "21 named".
4. **D9 GUI evidence re-read (fresh VLM), frames 002/004/005/007:** sidebar row
   "side chat fixture main" created and selected; side panel opens with
   placeholder "Ask a side question without interrupting this chat."; typed
   "quick aside question"; after side submit a new row "quick aside question"
   appears **above** while "side chat fixture main" **remains highlighted** —
   the not-selecting semantics is visually confirmed. Archived `vlmd9a.json`
   matches. d9-009 shows the "/side — Start a temporary side conversation"
   menu row verbatim.

**Defects found (evidence tree, not product):**

- **BROKEN (mislabel): `d9-008-side-closed.png` does not show a closed side
  panel.** Fresh VLM re-read (two independent prompts): the right-docked
  "Side chat" panel is **still open** and the main area compressed. Source
  confirms why: the *only* `CloseSideChat` dispatch is the panel header's
  close button (`side-chat-close`, ui.rs:18326-18331 → `close_side_chat`,
  ui.rs:8644-8645); **Escape is not a close binding for the side panel**. The
  D9 script's step 7 ("close the side panel (Escape)") was written against a
  wrong expectation, and the archived `vlmd9a.json` never read frames
  006/008/009/010 — so the mislabel was invisible in the archived record.
  The *close-path claim itself* still holds — via **D9b**: the labeled
  re-read of the probe frames shows the panel closed with the main view
  expanded at click (1418,140), and the core state test covers
  `CloseSideChat` semantics. (Correction to the archived interpretation: the
  close click was (1418,140) — the *third* candidate — and the final frame
  (1400,160) shows a stray dropdown, not the close.)
- **BROKEN (duplicate frame): `d9-006-side-submitted.png` and
  `d9-007-aftermath.png` are byte-identical** (md5 `8d733e08fce19256e60ba128c36aaa10`
  both). "Aftermath" adds no information beyond "side-submitted"; the
  4-seconds-later capture proves only that the UI was static.
- Consequence: **`d9-010-side-reopened.png` actually evidences an idempotent
  re-open on an already-open panel** (the panel never closed in D9), not
  close→reopen. The reopen claim rests on the state test
  (re-open keeps the existing side conversation) plus d9-009's menu row.

**Verdict: HELD** (every closure *claim* survives: binding, registry counts,
state semantics, file confinement, not-selecting GUI proof) — but the
wo-p2-006 evidence tree carries a mislabeled close frame and a duplicate
frame that the closure records describe more favorably than the pixels
support (→ FW-6).

---

## T3 — WO-P2-007 (Ctrl+P silent no-op fix + F-A4 no-workspace residual) — **HELD**

Closure claims attacked: dead `OpenFileSearch` action removed; `searchFiles`
interceptor owns Ctrl+P; 5-assertion regression test; diff ui.rs +40/−2; D10
baseline defect proof (byte-identical frames); D10b fix proof
(a9ad7e0c → 857b824c → a9ad7e0c); F-A4 residual honestly documented.

**Held, with primary evidence:**

1. **Removal exact:** `OpenFileSearch` at main appears exactly once — a
   historical *comment* inside the regression test (ui.rs:52095). No
   declaration, no `bind_keys` entry; `git grep '"ctrl-p"'` returns nothing
   (no GPUI keybinding remains). The `searchFiles` registry entry carries
   `shortcuts: &["CmdOrCtrl+P"]` (ui.rs:3243-3247) and the interceptor arm
   dispatches `open_command_palette(PaletteMode::Files)` (ui.rs:10924) — sole
   routing path, mirroring Ctrl+K/G as claimed.
2. **Diff-confinement exact:** `2948bf2` = ui.rs **+40/−2** (single file),
   `5574c95` (rustfmt) = +1/−4; ancestry `2948bf2, 5574c95 → 4f5a451 →
   a3c0e01` (PR #20) verified.
3. **Regression test has exactly the 5 advertised assertions**
   (`ctrl_p_routes_to_the_search_files_command`, ui.rs:52093+): sole owner
   `["searchFiles"]`; registry membership; `shortcut_command_id() ==
   Some("searchFiles")`; advertised `Some("Ctrl+P")`; metadata tuple
   ("Search files", CmdOrCtrl+P, General). Nothing weaker than claimed.
4. **D10 baseline (defect proof) re-derived:** d10-01/02/03 md5s all
   `bb5139b3d119c06db7009c3e663cfd1c` — recomputed, **byte-identical triple**:
   Ctrl+P then Escape produced identical frames on unfixed main. d10-04
   (working palette path) md5 also matches its manifest entry.
5. **D10b (fix proof) re-derived:** ws-01 `a9ad7e0c…` → ws-02 `857b824c…`
   (palette open) → ws-03 `a9ad7e0c…` (Escape returns to the **byte-identical**
   pre-keystroke frame) — recomputed md5s match the ledger's *cited hashes
   exactly*. Fresh VLM re-read of ws-02: command palette open, input
   placeholder **"Search files"**, section label **"Files"**, cursor in the
   input — the archived `vlm-ws02-read.md` is accurate.
6. **F-A4 residual honestly present and honestly documented:** the guard
   exists at main (ui.rs:8870: `if mode == PaletteMode::Files &&
   !self.has_local_workspace() { return; … }`); the sweep doc's F-A4 wording
   (input-surface-sweep.md L305, anchors at base `3c9f113`: arm 10480 →
   `open_command_palette` 8494, early return 8500-8502) re-verifies at the
   cited base — I confirmed `OpenFileSearch` declared at 1636, bound at 4636,
   `searchFiles` arm at 10480 **at `3c9f113`** exactly as §9 override 17 and
   the WO row state. No silent no-op was reintroduced between the fix and
   main (registry + arm + absence of the dead action re-checked at dbee3612).

**Verdict: HELD** — this closure survived every attack with exact primary
matches (diff stats, md5 chains, test body, anchors, residual honesty).

---

## T4 — WO-P2-008 (no-runtime-lab limitation + MAX 71→76 supersession) — **HELD on substance; BROKEN on two record defects**

Closure claims attacked: four registry bindings (ids 71→76, MAX=76); 7 core +
**6 app** binding tests; diff +728/−18 in exactly two files; D11b all four
bindings resolve visibly; D11 documents the no-runtime-lab limitation
("no codex CLI in lab", dot-on-row not GUI-exercisable, unit-covered); §9
override 18's supersession of the MAX=71 quirk.

**Held, with primary evidence:**

1. **Supersession exact:** `KEYBOARD_SHORTCUT_COMMAND_IDS: [&str; 76]` and
   `MAX_KEYBOARD_SHORTCUT_COMMANDS: usize = 76` at main (lib.rs:139/133); the
   full chain re-derives: 71 ids/MAX 71 at `f113515` → 72 ids/MAX 71 at
   `764e4db` (the sweep's quirk, real) → 76/76 at main. The four new ids
   (`toggleThreadUnread`, `nextUnreadChat`, `clearAllUnread`,
   `toggleActivityView`) are in the registry, in `ACTIVE_KEYBOARD_SHORTCUTS`
   with the claimed chords (Ctrl+Shift+U / Ctrl+Alt+A / Shift+Escape /
   Ctrl+Alt+U), and have live interceptor arms (ui.rs:10862-10865).
2. **Core tests (7) exist and assert at claimed strength:** enumerated in the
   `c37c21b` diff and at main; the pivotal
   `background_turn_completion_marks_the_chat_needing_attention` asserts
   background completion flags, **failed turns flag**, selected-chat never
   flags, unknown-task never flags — exactly the ledger's wording. The other
   six cover approval marking, visit-clears, round-trip + honest status,
   archive-drops, clear-all honest count, activity-view honest guidance.
3. **Diff confinement exact:** `c37c21b` = ui.rs +397/−…, lib.rs +349/−…,
   **+728/−18 in exactly 2 files**; identical on the merged side
   (`54bd7f5..00a3392` code-side stat matches line-for-line).
   *Lineage nuance (verified, honest):* the worker's `c37c21b` is **not an
   ancestor of main** — the merged branch carries `999b4bb`, whose `crates/`
   tree is **byte-identical** to `c37c21b`'s (empty `git diff c37c21b 45e0ad8
   -- crates/`). This matches the ledger's "harvested and verified by the
   Lead" account (re-parented harvest, identical content); recorded here so
   future auditors do not assume SHA ancestry.
4. **D11b GUI evidence re-read (fresh VLM):** d11b-03 shows the verbatim
   status **"No chats need attention."** with the sidebar showing "No chats";
   all five d11b md5s match the manifest; each binding chord is a real
   registry chord at main. D11 frame 03's honest no-selection status
   ("Select a chat before marking it unread.") is archived in
   `vlm-d11-03.json` — consistent with the frame's position in the md5 chain.
5. **The no-runtime environment claim is visually true in the D11 frames:**
   fresh VLM read of d11-02a/02b shows the main area banner **"Couldn't
   connect to the Codex app-server"** and the sidebar footer "Connection
   failed". And architecturally the limitation is real: `Effect::CreateTask`
   → `app_server.start_thread(...)` (backend.rs:6695+) — **no connection, no
   thread row**, so dot-on-row/jump-between-chats genuinely cannot be
   GUI-exercised in that environment. The README's residual classification
   (unit-covered, same class as F-A4) is defensible.

**Defects found:**

- **BROKEN (count): "6 codex-app binding tests" — the tree contains 5.** The
  delivery diff `c37c21b` adds exactly five test functions
  (`activity_view_binding_resolves_ctrl_alt_u_to_visible_guidance`,
  `next_unread_chat_binding_resolves_ctrl_alt_a_and_jumps`,
  `clear_all_unread_binding_resolves_shift_escape`,
  `toggle_thread_unread_binding_resolves_ctrl_shift_u`,
  `next_unread_chat_follows_sidebar_order_cyclically`); the merge side adds
  the same five; nothing adds a sixth by main. The "6" is claimed in four
  places: ledger WO row (L448), ledger changelog (L675), parity §5.10 row
  (L260), parity changelog (L737), plus wo-p2-008/README (L19, L64).
- **BROKEN (mislabeled evidence + script defect): the D11 full-flow scene
  never dispatched a submit.** `d11-unread-attention.sh` steps 02/04 type and
  then press **plain `Return`** — but the composer submits only on
  `InputEvent::PressEnter { secondary: true }` (Ctrl/Cmd+Enter), verified on
  the exact D11 binary (`c37c21b` ui.rs:5764-5766) and on the d8c4/d9
  binaries. Fresh VLM re-read of d11-02b: the typed text **"alpha attention
  chat" is still sitting in the composer with an added newline**, sidebar
  "No chats" — the frames named `d11-02b-chatA-created` /
  `d11-04-chatB-created` show **no created chats and no submit attempt**.
  The README's conclusion (chat creation not exercisable without runtime) is
  nonetheless *true* (see 5 above), but these two frames do not show what
  their names claim, and the scene's causal story ("the scene could not
  exercise its chat-creation steps [because] no codex CLI exists") conflates
  a genuine environment gap with a script key defect. Also minor: the
  "7-frame md5 sequence" manifest omits the md5 of the eighth file,
  `d11-02a-typed.png`.

**Verdict: HELD on the closure substance** (bindings, counts chain, tests,
diff confinement, D11b evidence, and the *truth* of the no-runtime
limitation) — **BROKEN on the record**: the "6 app binding tests" count and
the D11 scene's frame labels/script (→ FW-1, FW-7).

---

## T5 — §9 override trail + §8 counts vs rows — **HELD on substance; BROKEN on four stale/misdirected cells**

**Held, with primary evidence (the audit trail's load-bearing entries):**

1. **§8.1 census (full re-derivation):** 53 table rows across §5.1–§5.10;
   status distribution **complete 8 / partial 32 / missing 4 /
   platform-limited 2 / deferred 7** — every one of the 21 §8.1-named rows
   matches its cells (the 8 complete rows are exactly the named 8; no unnamed
   row is fully complete/missing/deferred); the arithmetic history
   7/31/6/2/7 → 8/31/5/2/7 (006) → 8/32/4/2/7 (008) is internally coherent.
2. **Override 1:** at `f113515` the `/review` executor sits at ui.rs:7557+
   with the availability guard — 16 named commands pre-wave, as recorded.
3. **Override 2:** CODEX-REFERENCE-MATRIX.md L96 labels the baseline set
   "(15)" while enumerating exactly 14 names (`/shell` absent from the list)
   — the discrepancy record is accurate.
4. **Override 8:** parity-matrix.md L24 "126 client request methods" vs A
   matrix L246 "89 `ClientRequest` methods" — both quotes exist as recorded.
5. **Override 17:** all three anchors at base `3c9f113` verified exact
   (declaration 1636, binding 4636, interceptor arm 10480); the FIXED addendum
   matches the merged reality (see T3).
6. **Override 18:** the flip (missing 5→4, partial 31→32) and the MAX=71-quirk
   supersession are both true at main (T4).
7. **21/24-command arithmetic:** current-target list has exactly 24 names;
   Flauz's named set at main has exactly 21; the delta is exactly the
   enumerated 11-minus-implemented set.

**Defects found (documentation-level, none flips a status):**

- **BROKEN (stale count): "seven of the eleven absent names stay open"** —
  post-WO-P2-006 (`/side` shipped) only **six** stay open (`/cloud`,
  `/cloud-environment`, `/ide-context`, `/local`, `/pet`, `/task`). The
  Composer row is *internally inconsistent*: its Flauz cell says "21 named
  … WO-P2-006 added the guarded `/side`" while its Gap cell (parity L176)
  still says "seven … stay open". Repeated at parity L449 (§8.2 item 5),
  ledger L375, ledger L669. (At 005-closure time "seven" was correct; the 006
  flip never propagated.)
- **BROKEN (stale list item): §8.2 item 12** (parity L465) — "Browser
  address-bar history / Google fallback (26.727)" still reads as an open
  no-WO gap, with no WO-P2-009 annotation, although §5.4's row now records
  the history/Settings part CLOSED (residual: Google fallback). Item 8
  received its WO-P2-008 annotation; item 12 was left inconsistent with the
  row it parallels.
- **BROKEN (stale prose): §8.1 L404-407** — "The 6 `missing` rows are …
  (side chats, WebMCP, browser extension, in-app editing, Record & Replay,
  Activity view)": side chats is `complete` and Activity view `partial` at
  this head; only 4 rows are `missing`. The counts table was maintained; the
  explanatory sentence was not.
- **BROKEN (citation drift):** closure records misdirect section references:
  "§5.3 Composer row" (Composer is §5.2) at parity L730/L731 and L449; "§5.3
  Side chats row" (Side chats is §5.1) at parity L732/L733 and ledger L671;
  "§5.9 Keyboard row" (the Keyboard row is §5.10) at parity L735. The
  work-order rows themselves carry the correct references (§5.2/§5.1/§5.10) —
  the drift is confined to changelogs and the §8.2 list, but it degrades the
  audit trail's navigability.

**Verdict: HELD on substance** (no override contradicts the row it cites at
the level that matters; counts and named-row statuses all re-derive) —
**BROKEN on four specific stale/misdirected cells** above (→ FW-2..FW-5).

---

## EQ-1 (D9 vs D11 lab-runtime discrepancy) — **RESOLVED with new primary evidence**

RWO-020 notes N4 recorded the contradiction without adjudicating it. This
review closes it with frame-level evidence:

- **D9 (2026-09-18)** frame 002, fresh VLM read: footer **"App-server
  online"**; sidebar row "side chat fixture main" created and selected; toast
  "Codex hit an error and is retrying." → the 006-era lab **had a resolved,
  connected (unauthenticated) runtime**: thread creation succeeded
  (`start_thread` OK), turn execution failed.
- **D11 (2026-09-19)** frames 02a/02b, fresh VLM read: **"Couldn't connect to
  the Codex app-server"** / "Connection failed"; "No chats". → the 008-era lab
  **genuinely lost the runtime**.

Conclusion: EQ-1 is **real environment drift between 2026-09-18 and
2026-09-19**, not an evidence fabrication on either side — exactly the
"Lead/lab question" RWO-020 suspected, now proven. The residual wrinkle is
that D11's scene *also* contained the plain-Return submit-key defect (T4), so
its "chat creation not exercisable" demonstration is architecturally- but
not scene-proven. → FW-8.

---

## Verdict table

| # | Target | Verdict | Broken sub-findings |
| --- | --- | --- | --- |
| T1 | WO-P2-005 picker fix + 15-site audit | **HELD** | (caveats: no archived audit artifact; executor guard fall-through untested; no md5 manifest in evidence dir) |
| T2 | WO-P2-006 side-chat semantics | **HELD** | d9-008 mislabeled (panel open; Escape is not a close binding); d9-006 ≡ d9-007 byte-identical; d9-010 = idempotent reopen, not close→reopen |
| T3 | WO-P2-007 Ctrl+P fix + F-A4 | **HELD** | none |
| T4 | WO-P2-008 no-runtime-lab + MAX 71→76 | **HELD (substance)** | "6 app binding tests" ≠ 5 delivered; D11 full-flow scene used non-submitting `Return`; frames 02b/04 mislabeled; manifest omits d11-02a |
| T5 | §9 override trail + §8 counts | **HELD (substance)** | stale "seven of eleven" (×4 sites); §8.2 item 12 not annotated post-009; §8.1 "6 missing rows" prose; §5.3/§5.9 changelog citation drift (×6 sites) |

Zero product-code defects were found: every BROKEN finding above is a
**record/evidence-tree defect** (counts, labels, stale prose, script keys),
and every load-bearing closure claim survived primary re-derivation.

## Follow-up work-order candidates

- **FW-1** (docs): correct "6 codex-app binding tests" → 5 (or add the
  missing sixth test) in ledger L448/L675, parity L260/L737, wo-p2-008
  README L19/L64.
- **FW-2** (docs): "seven of the eleven absent names stay open" → six, at
  parity L176/L449, ledger L375/L669.
- **FW-3** (docs): annotate §8.2 item 12 with the WO-P2-009 history closure
  (Google-fallback residual remains open).
- **FW-4** (docs): repair §8.1 L404-407 prose (historical "6 missing rows"
  note) and the changelog §-references (§5.3→§5.2/§5.1; §5.9→§5.10).
- **FW-5** (lab): fix `d11-unread-attention.sh` submits to `ctrl+Return`,
  relabel `d11-02b`/`d11-04`, add `d11-02a` to the md5 manifest, and amend
  the README's causal note (environment gap + script key defect).
- **FW-6** (evidence): add a README to `wo-p2-006/` documenting that the side
  panel's close affordance is the header button (D9b (1418,140) is the
  close proof; D9 frame 008 is mislabeled; frames 006/007 are identical) —
  or re-run a corrected D9.
- **FW-7** (lab-infra): pin/restore the lab's codex runtime and re-run the
  D11 full-flow scene (dot-on-row + jump-between-chats GUI observation);
  records EQ-1 as resolved (D9 "App-server online" vs D11 "Couldn't connect").
- **FW-8** (evidence hygiene): add md5 manifests to `wo-p2-005/` and archive
  a per-site table for the 15-site picker audit (or note in the ledger that
  the audit is source-derivable, not artifact-archived).
- **FW-9** (test, optional): add executor-level guard fall-through tests for
  the 005/006 slash commands (guard failure → visible message submit), closing
  the gap between the WO test plan and the delivered resolution-layer tests.

*No product code was modified by this review. Base, SHAs, and line anchors
were captured at `dbee3612` on branch `research/rwo-022`.*
