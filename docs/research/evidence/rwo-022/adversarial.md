# RWO-022 (second dispatch, RWO-022-R2) — Independent adversarial re-challenge of the P2 closure set (WO-REVIEW-001, Worker C)

- **Task ID:** RWO-022 (adversarial third of WO-REVIEW-001 deep phase) — executed as
  an independent second pass; see the collision disclosure below.
- **Base:** `main` @ `dbee3612bbe1aab200099e1b998d38b73d4b897b` (verified with
  `git rev-parse dbee3612…^{commit}` after clone; branch
  `research/rwo-022-r2` created from it). The base carries PR #24/#27
  (WO-P2-009), PR #25 (WO-P2-010), PR #26 (RWO-020 rubric), ledger CLOSED
  rows, parity §5.4/§5.10 updates, and the completed evidence trees exactly
  as the dispatch guide advertises.
- **Method:** every claim below was re-derived from **primary evidence only**:
  git objects at the cited SHAs (`git show`/`git diff`/ancestry), the source
  at the review base, recomputed md5s of every evidence frame, pixel-level
  differential analysis of contested frames (PIL/NumPy), and **fresh VLM
  reads of the archived frames** performed by this worker via the sandbox
  vision CLI (archived VLM transcripts were used only for cross-comparison,
  never as the sole source). Ledger/parity prose was never trusted as
  evidence. Where this pass confirms a prior finding, it says so; where it
  adds or diverges, it says so too.

## Dispatch collision disclosure (binding context for the Lead)

The remote already contained a **prior RWO-022 deliverable** when this
session cloned the repository: branch `research/rwo-022` @ `17a1897`
(452-line `docs/research/evidence/rwo-020/adversarial.md`), merged to main
via PR #29 → `230f6fc`, plus a remediation commit PR #30 → `aedb771`
(FW-1..FW-4, FW-6, FW-8 remediations). The dispatch guide's stated
deliverable path (`docs/research/evidence/rwo-020/adversarial.md`) is
therefore **occupied**, and the guide's branch name (`research/rwo-022`) is
**taken** on the remote. This pass consequently:

1. based on the pinned base SHA exactly as instructed (dbee3612, reachable);
2. pushed on **`research/rwo-022-r2`** (following the repo's second-dispatch
   convention: `rwo-020-alt`/`rwo-020-r2`, `rwo-021-verified`);
3. delivered to **`docs/research/evidence/rwo-022/adversarial.md`** (this
   file) instead of the guide's literal path, to avoid a merge collision with
   the prior run's file — the guide's `rwo-020/` path is presumed a typo for
   this worker's own ID (RWO-020 was the rubric worker, PR #26).

Nothing in this file was copied from the prior deliverable without
independent re-derivation; every confirmed prior finding below was
re-derived from primary evidence first, then cross-checked against the prior
run's text on main.

## Verification bounds (honest NOT RUN list)

1. **No Rust toolchain in this sandbox** (`cargo`/`rustc` absent; the repo
   pins 1.97.1). Every *test-run* claim — "221/221 core tests", "605 green
   (406 + 199)", "7/7 core state tests per-test verified", "CI green both
   matrices", "fmt gates pass on pinned 1.97.1" — is **NOT RUN** here. What
   was done instead: existence, count, and assertion strength of every cited
   test was verified statically at the cited SHAs (bodies read in full, not
   just names). Static corroboration noted per target.
2. **The LINUX_GUI_LAB is not re-runnable here.** Scene scripts reference
   `/home/z/parity-lab/...` hosts and donor DBs absent from this sandbox
   (`/home/z/parity-lab` does not exist). What was done instead: md5
   recomputation of every archived frame against the in-tree manifests,
   pixel-differential analysis of contested frames, and fresh VLM re-reads
   of the frames the closures rest on.
3. CI run records (GitHub Actions logs) are not archived in-repo; "CI green
   both matrices" remains a recorded claim, not re-verified.

---

## T1 — WO-P2-005 (fork-picker fix + "15-site picker-lifecycle audit") — **HELD**

Closure claims attacked (ledger L668; parity changelog L730): four guarded
slash commands; picker wiring defect fixed in `4726dd4` via the testable
`composer_keeps_fork_picker` predicate + regression test; "15-site
picker-lifecycle audit clean"; "grep-verified no test touches the picker
flag"; D8c4 GUI evidence.

**Held, with primary evidence (re-derived at dbee3612):**

1. **The four commands + guards exist in the executor**
   (`execute_composer_slash_command`, ui.rs:8110+): `/approve` (guard:
   `selected_approval_request(&self.state).is_none() → return false`),
   `/fast` (guard: `fast_service_tier_id(&self.state)` else `return false`),
   `/worktree` (guard: `!(has_local_workspace() &&
   selected_task_id.is_some()) → return false`), `/personality`
   (unguarded → `SettingsSection::Personalization`, consistent with its
   "unguarded like /mcp" test comment). Menu-row availability bits at the
   resolver (`composer_slash_availability`, ui.rs:8295+). Named-command
   count at base: exactly **21** (15 baseline + `/review` + 4 from 005 +
   `/side`) — matches the Composer row's "21 named". Commit `2e6358f`
   (ui.rs +394/−132, single file) is an ancestor of merge `e46ad4f3df`
   (PR #17), as is fix `4726dd4` — topology verified.
2. **The fix is exactly as described** (`git show 4726dd4`, ui.rs +30/−1):
   the Change-subscription close condition `value.trim() != "/fork"` became
   `!composer_keeps_fork_picker(&value)`; the predicate is
   `trimmed == "/fork" || trimmed == "/worktree"` (ui.rs:45795 at base);
   the regression test `fork_picker_stays_open_for_both_fork_destination_commands`
   asserts both commands, trimmed variants, and 7 negative cases
   (ui.rs:48784+).
3. **The "15-site" audit number re-derives exactly.** This worker counted
   the occurrences of `composer_fork_picker_open` in ui.rs with ripgrep:
   **15 at `4726dd4` and 15 at the review base** (5636 declaration; 6051
   Change-subscription close via the predicate; 6836 init-false; 7684
   selection-change reset; 8188/8196 dispatch-on-open for /worktree and
   /fork; 8339/8399/8433/8533 mutual-exclusion closes in the review-submenu
   / mcp-status / project-picker / status openers; 8472 setter; 8482
   submit-path close; 23062/23068 Escape close + composer reset; 23984
   render gate). All 15 sites are lifecycle-coherent; none bypasses the
   predicate. The audit *claim* is therefore source-derivable and true on
   re-derivation — but note it still has no archived per-site artifact
   (see caveats).
4. **"No test touches the picker flag" re-verifies:** all 15 sites lie
   outside `mod tests`; the only test references use the predicate.
5. **D8c4 GUI evidence:** archived `vlmd8c4.json` shows frames 004/005/006
   all with the picker row "Continue in new worktree" visible for composer
   text `/worktree` — the regression gate (picker-still-open 2.5 s later)
   passes on the archived frames. `git-worktree-list.txt` records two
   worktrees @ `97f6b70` (one detached) and archived `vlm008.json` reads
   the aftermath frame as the sidebar gaining the worktree entry — the
   fork actually executed on the fixture repo. Consistent with the fix
   claim.
6. Static corroboration of counts: `#[test]` count in codex-core at
   `764e4db` is exactly **221** (matches "221/221" as a count; pass status
   NOT RUN — no toolchain).

**Defects found (record/evidence hygiene, not product):**

- **BROKEN (missing manifest at base):** `wo-p2-005/d8c4/` carried **no md5
  manifest** at the review base (unlike 007/008/009/010 evidence dirs). This
  worker recomputed the frame hashes itself (see the manifest values quoted
  below under N-2). Remediated on main by PR #30 (FW-8) — verified present
  at `origin/main`.
- **NEW — BROKEN (unannotated duplicate frame, survives on main):**
  `d8c4-007-fork-dispatched.png` ≡ `d8c4-008-aftermath.png` are
  **byte-identical** (md5 `23925833ed3bd5ecdca9ce15a67b7f29` both,
  recomputed at base). The "aftermath" step adds zero information beyond
  "fork-dispatched" — exactly the same defect class as the
  `d9-006 ≡ d9-007` duplicate in wo-p2-006 that the prior run flagged
  (FW-6) and PR #30 remediated. Neither the prior adversarial report nor
  the PR #30 manifest note annotates this pair (the PR #30 manifest hashes
  both files, making the duplication visible, but no annotation exists).
  → follow-up FW-N2.
- **BROKEN (script self-mislabel, cosmetic):** `d8c4-wo-p2-005-happypath.sh`
  line 2 self-identifies as "Scene D8c2" while every other artifact names
  the scene D8c4 (confirmed at base; annotated in the PR #30 manifest note).
- INCONCLUSIVE (honest limit): "605 existing tests unaffected" — the 605
  total is not statically re-derivable with confidence without cargo
  (NOT RUN); the *picker-flag isolation* half of the claim IS re-verified
  (item 4).

**Verdict: HELD** — the closure survives; the evidence tree carries one new
unannotated duplicate (N-2) and the base-state manifest gap (remediated).

---

## T2 — WO-P2-006 (side-chat selection / active-turn semantics) — **HELD on substance; three evidence-integrity defects**

Closure claims attacked: Ctrl+Alt+S binding via registry command
`openSideChat` (ids 71→72); guarded `/side` (21 named commands); side submit
creates the side thread **without selecting it**; main-chat **selection and
active turn untouched (state test)**; close returns to the main view; diff
confinement ui.rs +381 / core lib.rs +469; D9/D9b GUI evidence.

**Held, with primary evidence (re-derived at dbee3612):**

1. **Registry arithmetic exact, re-derived independently:**
   `KEYBOARD_SHORTCUT_COMMAND_IDS` counts 71 at `f113515`, **72 at
   `764e4db`** (array literal `[&str; 72]`), 76 at base. Diff confinement
   exact: `764e4db` = ui.rs +381, lib.rs +469, exactly 2 files (ledger
   wording matches to the line). Ancestry `764e4db → 73eb55c → 5287c29f3f`
   (PR #18) verified.
2. **The state tests assert the full claimed semantics — bodies read in
   full** (codex-core at base):
   `a_side_chat_first_message_creates_a_projectless_thread_without_selecting_it`
   (lib.rs:24312) seeds an active turn (`turn-1`) and a main draft, submits
   a side message, asserts the `Effect::CreateTask` (projectless, cwd None),
   then after `NewChatTaskCreated` asserts `side.task_id == "side-1"` while
   `selected_task_id` stays `"t1"` **and** the active turn survives **and**
   the main composer draft survives; a stale-creation control is asserted
   (an unrelated `NewChatTaskCreated` is never claimed). The sibling tests
   (`a_side_chat_opens_and_closes_without_touching_the_selected_chat`
   lib.rs:24235, `a_side_chat_continues_through_the_thread_turn_machinery`
   lib.rs:24404, `a_failed_side_chat_submission_restores_only_the_side_draft`
   lib.rs:24472) cover open/close/steer/failed-submit paths. No
   weaker-than-claimed assertion found.
3. **Executor + binding wiring verified:** `/side` guard
   (`selected_task_id.is_none() → return false`) in the executor
   (ui.rs:8125+); `submit_side_chat` (lib.rs:7909) creates the side thread
   through the shared projectless machinery with a generation guard, and
   surfaces the honest status "Open a chat before starting a side
   conversation." when no chat is selected — no silent no-op. App-side
   binding test asserts sole accelerator ownership (`owners ==
   ["openSideChat"]`), registry membership, and reducer
   open-without-selection.
4. **D9 GUI evidence, cross-verified two ways:** archived `vlmd9a.json`
   documents frames 002/004/005/007 — main chat "side chat fixture main"
   created and **remains highlighted** while the side panel opens, accepts
   "quick aside question", and after side submit a new row appears above
   with the main chat still selected. This worker **independently
   re-read d9-002 with a fresh VLM call**: footer reads **"App-server
   online"**, sidebar shows the created main chat, no side panel —
   consistent. The D9 scene script itself uses `ctrl+Return` for submits
   (correct — see T4 for the contrast that matters).

**Defects found (evidence tree, independently confirmed):**

- **BROKEN (mislabel): `d9-008-side-closed.png` does not show a closed side
  panel.** Two independent methods, this worker's own: (a) *pixel
  differential* — the right-docked panel region (x1081-1441) of d9-008
  differs from the panel-OPEN frame d9-004 by only 2,481 px but from the
  no-panel frame d9-002 by 9,030 px → the panel is open; (b) *fresh VLM
  read of d9-008*: "right-docked Side chat panel visible: yes; composer
  placeholder 'Ask a side question'; selected row 'side chat fixture
  main'". Source explains why: the only `CloseSideChat` dispatch is the
  panel header close button; Escape is not a close binding. The D9 script's
  step 7 ("close the side panel (Escape)") was written against a wrong
  expectation. (Prior run FW-6 — confirmed independently.)
- **BROKEN (duplicate frame): `d9-006-side-submitted.png` ≡
  `d9-007-aftermath.png`** — byte-identical, md5
  `8d733e08fce19256e60ba128c36aaa10` both (recomputed). "Aftermath" adds
  no information. (Prior FW-6 — confirmed.)
- Consequence, confirmed by pixels: **`d9-010-side-reopened.png` differs
  from d9-008 by only 173 px in the panel region (278 px full-frame)** —
  an idempotent re-open of an already-open panel, not close→reopen.
- **D9b close-probe re-derived:** the probe script clicks four candidate
  positions; archived `vlmd9b.json` reads three frames "panel visible:
  yes", one "panel closed, main view expanded" — the close affordance
  exists at one of the clicked positions — and a final frame "a dropdown
  menu appeared" (a stray misclick artifact, not a close). The close-path
  claim holds via D9b + the core `CloseSideChat` test.

**Verdict: HELD on substance** (binding, registry arithmetic, state
semantics, diff confinement, not-selecting GUI proof all survive) — with the
two frame-integrity defects above (both prior-run-confirmed; README
remediated on main by PR #30).

---

## T3 — WO-P2-007 (Ctrl+P silent no-op fix + F-A4 no-workspace residual) — **HELD**

Closure claims attacked: dead `OpenFileSearch` action removed;
`searchFiles` interceptor owns Ctrl+P; 5-assertion regression test; diff
ui.rs +40/−2; D10 baseline defect proof (byte-identical frames); D10b fix
proof (57522B `a9ad7e0c…` → 58675B `857b824c…` → byte-identical return);
F-A4 residual honestly documented.

**Held, with primary evidence (re-derived at dbee3612):**

1. **Removal exact:** `OpenFileSearch` appears exactly once at base — a
   historical comment inside the regression test (ui.rs:52095). No
   declaration, no `bind_keys` entry. The `searchFiles` registry entry
   carries `shortcuts: &["CmdOrCtrl+P"]` (ui.rs registry at 2864+) and the
   interceptor arm dispatches `open_command_palette(PaletteMode::Files)` —
   sole routing path.
2. **Diff confinement exact:** `2948bf2` = ui.rs **+40/−2** (single file);
   `5574c95` (rustfmt) = +1/−4; ancestry → PR #20 → `a3c0e01` verified.
3. **The regression test carries exactly the 5 advertised assertions**
   (`ctrl_p_routes_to_the_search_files_command`, ui.rs:52093+): sole owner
   `["searchFiles"]`; registry membership; `shortcut_command_id() ==
   Some("searchFiles")`; advertised `Some("Ctrl+P")`; metadata tuple
   ("Search files", CmdOrCtrl+P, General). Nothing weaker than claimed.
4. **D10 baseline re-derived:** d10-01/02/03 md5s all
   `bb5139b3d119c06db7009c3e663cfd1c`, all 54,058 bytes (recomputed;
   manifest matches) — the byte-identical triple is the defect proof;
   d10-04 (51664 B) differs = the working palette route.
5. **D10b fix proof re-derived, byte-exact:** ws-01 = 57,522 B / md5
   `a9ad7e0ce57201bf1a8217302d945a36`; ws-02 = 58,675 B /
   `857b824c5872d1dd37a233579aaadc31`; ws-03 = 57,522 B / identical to
   ws-01 — **the ledger's cited sizes and hashes match this worker's
   recomputation exactly**. In-tree `vlm-ws02-read.md` ("Search files"
   placeholder, Files section, focused input) is consistent with the fix
   claim.
6. **F-A4 residual honest:** the guard exists at base
   (`open_command_palette`, ui.rs:8864+: `if mode == PaletteMode::Files &&
   !self.has_local_workspace() { return; }`) and the ledger/override 17
   record the residual as follow-up material rather than hiding it.

**Defects found:**

- **NEW — BROKEN (reproducibility pointer): the D10/D10b scene scripts are
  not archived in the repository.** The wo-p2-007 README's "Scene scripts"
  section points at `parity-lab/scenes/` (`d10-ctrl-p-file-search.sh`,
  `d10b-ctrl-p-workspace.sh`), but neither file exists in **any ref** of
  this repository (checked every remote ref via `git ls-tree -r`), and the
  referenced `/home/z/parity-lab` host is external. Every sibling closure
  archived its scene script in-tree (d8c4-happypath.sh; d9/d9b scripts;
  d11/d11b scripts; scenes/d12-*.sh; scenes/d13-*.sh) — wo-p2-007 is the
  only GUI-evidenced closure whose scenes cannot be re-run from the repo.
  Closure gate 5 ("UI/provider/lab behavior is reproducible") is therefore
  only partially supported for this WO: frames + md5s + VLM read are
  archived, the scene recipe is not. → follow-up FW-N3.

**Verdict: HELD** — every closure claim survives with exact primary
matches; one new reproducibility-gap finding (N-3).

---

## T4 — WO-P2-008 (no-runtime-lab limitation + MAX 71→76 supersession) — **HELD on substance; BROKEN on two record defects**

Closure claims attacked: MAX_KEYBOARD_SHORTCUT_COMMANDS 71→76 with registry
76 in step (superseding the sweep's 71-vs-72 quirk); four persisted
bindings with honest statuses; 7 core state tests + "6 codex-app binding
tests"; D11b all-four-bindings evidence; D11 honest no-selection path with
the no-runtime-lab limitation documented; dot-on-row unit-covered.

**Held, with primary evidence (re-derived at dbee3612):**

1. **The MAX chain re-derives exactly, at every cited SHA** (this worker's
   own counts): `f113515` registry 71 / ids-array 71; `764e4db` 72/72
   (WO-P2-006's +1); sweep base `3c9f113` 72-array with MAX constant 71
   (the documented quirk, verbatim re-derived); delivery `c37c21b` 76/76
   with `MAX_KEYBOARD_SHORTCUT_COMMANDS = 76` (lib.rs:133); review base
   dbee3612 76/76/76. The §9 override 18 supersession claim is exactly
   true. The four new commands are registry members with the advertised
   bindings (toggleThreadUnread / nextUnreadChat / clearAllUnread /
   toggleActivityView), no duplicate ids.
2. **The 7 core state tests assert exactly what is claimed — bodies read in
   full** (codex-core at base):
   `background_turn_completion_marks_the_chat_needing_attention`
   (lib.rs:37557) asserts background flagging, **failed turns included**,
   selected-chat exclusion, and unknown-task exclusion;
   `approval_request_marks_background_chats_needing_attention` (37634)
   asserts background + selected + unknown; visiting-clears (37669);
   clear-all honest reporting (37704); toggle round-trip with status
   (37736); archive-drops (37761); activity-view honest guidance (37790).
3. **D11b evidence re-derived:** all five frames md5-distinct (manifest
   recomputed, matches); the four archived VLM transcripts contain the four
   advertised statuses **verbatim** ("Select a chat before marking it
   unread." / "No chats need attention." / "Activity view is not available
   yet. Use "Next chat needing attention" to jump to unread chats." / "No
   unread chats") — the closure's verbatim-status claim is exact.
4. **D11's no-runtime honesty, plus a second cause the record missed — see
   defects.** The README documents the no-codex-CLI limitation and the
   not-GUI-exercisable dot-on-row candidly.

**Defects found (record, independently confirmed):**

- **BROKEN (count): "6 codex-app binding tests" — the tree contains 5.**
  This worker counted the WO-P2-008 binding tests at base:
  `activity_view_binding_resolves_ctrl_alt_u_to_visible_guidance` (52136),
  `next_unread_chat_binding_resolves_ctrl_alt_a_and_jumps` (52178),
  `clear_all_unread_binding_resolves_shift_escape` (52217),
  `toggle_thread_unread_binding_resolves_ctrl_shift_u` (52259),
  `next_unread_chat_follows_sidebar_order_cyclically` (52305) — **5
  tests**, claimed as 6 at five doc sites at base (ledger L448; parity
  §5.10 row L260; parity changelog L737; wo-p2-008 README L19 and L64).
  (Prior FW-1 — confirmed independently; remediated on main by PR #30 at
  all five sites.)
- **BROKEN (scene-script defect + mislabeled frames): the D11 full-flow
  scene submits with plain `Return`, which is not a submit key in this
  app.** Primary evidence, this worker's own derivation: the composer's
  Enter handling requires `InputEvent::PressEnter { secondary: true }`
  (ctrl+Return) to submit (ui.rs:6066; all sibling handlers likewise);
  the D9 script (wo-p2-006, one day earlier) correctly uses `ctrl+Return`;
  the D11 script uses bare `key Return` at both submission steps
  (d11-unread-attention.sh). So D11's chat-creation steps failed for **two
  stacked reasons** — no runtime AND a non-submitting key — while the
  README attributes the failure solely to the missing runtime. Fresh VLM
  read of `d11-02b-chatA-created.png` (this worker): footer **"Connection
  failed"** (red dot), sidebar **"No chats"**, composer empty — the frame
  label "chatA-created" is false; by script symmetry `d11-04-chatB-created`
  is the same class. The d11 manifest also omits d11-02a (the script hashes
  7 frames, not 8). (Prior FW-5/FW-7 territory — the script defect and
  mislabels confirmed independently; the README's incomplete root-cause
  attribution is additionally noted here.)

**Verdict: HELD on substance** (bindings, MAX supersession chain, test
strength, D11b verbatim statuses) — **BROKEN on the record**: the 6-vs-5
count (remediated) and the D11 scene's key defect + mislabeled frames +
incomplete root-cause attribution (FW-5/FW-7 open on main).

---

## T5 — §9 override trail + §8 counts vs rows — **HELD on substance; BROKEN on four stale/misdirected cells (all independently re-derived)**

**Held, with primary evidence:**

1. **The §8.1 counts table is exact against the matrix.** This worker
   census-counted the §5 matrix rows at base: **53 rows total; 8 complete
   (incl. Side chats); 32 partial; 4 missing (WebMCP site tools, Browser
   extension, In-app editing, Record & Replay); 2 platform-limited; 7
   deferred** — byte-for-byte the §8.1 table (53 = 47 carried + 6 added,
   and the audit-added six are all present and labeled). The flip chain
   (complete 7→8 at WO-P2-006; missing 5→4 / partial 31→32 at WO-P2-008)
   reconciles exactly with the changelog.
2. **No §9 override contradicts the row it cites on substance.** Re-checked
   the load-bearing ones: override 1/2 (16 = 15 + /review; 24-command
   target = 13 shared + 11 absent; 11 = 4 shipped by 005 + /side by 006 + 6
   open) — arithmetic exact; override 5→WO-P2-005 scope matches the
   delivered guards; override 17's FIXED paragraph matches the source state
   (dead action gone, searchFiles sole owner); override 18's counts match
   the matrix. Historical line-number drift between override 17 (ui.rs:4527
   at the 09-17 base) and the WO-P2-007 row (ui.rs:4636 at `3c9f113`) is
   explained by the different cited SHAs — not a contradiction.

**BROKEN cells (each re-derived at base, then checked against main):**

- **Stale Gap cell (self-contradictory row):** the §5.2 Composer row's Gap
  cell (parity L176) says "**seven** of the eleven absent names stay open"
  while the *same row's* Flauz cell says WO-P2-006 added the guarded
  `/side` — 11 − 4 − 1 = **six**. The row contradicts itself at base.
  (Prior FW-2 — confirmed; the live cell was fixed on main by PR #30;
  three historical mentions were annotated.)
- **Stale priority item:** §8.2 item 12 (parity L465) "Browser address-bar
  history / Google fallback (26.727)." — unannotated at base although
  WO-P2-009 had already CLOSED the history + Settings-management part
  (residual: Google fallback only). (Prior FW-3 — confirmed; annotated on
  main.)
- **Stale census prose:** §8.1 L404-407 "The 6 `missing` rows are … (side
  chats, …, Activity view)" contradicts the counts table three lines above
  it (missing = 4; side chats complete; Activity view partial). (Prior
  FW-4 — confirmed; historical-census note added on main.)
- **Citation drift (wrong §-numbers):** at base, closure records direct
  readers to the wrong sections: "§5.3 Composer row" (Composer is §5.2) at
  parity L449/L730/L731 and ledger L669; "§5.3 Side chats row" (Side chats
  is §5.1) at parity L732/L733 and ledger L671; "§5.9 Keyboard row"
  (Keyboard shortcut reference is §5.10) at parity L735. (Prior FW-4 —
  confirmed; fixed on main.)

**Verdict: HELD on substance; BROKEN on the four cells above** — all four
verified at base by this worker first, all four remediated on main by
PR #30.

---

## EQ-1 (D9 vs D11 lab-runtime discrepancy) — **RESOLVED, re-verified with this worker's own fresh VLM reads**

This worker performed its own fresh vision reads of the archived frames
(not reusing the prior run's reads):

- `wo-p2-006/d9-002-main-created.png` (2026-09-18 scene): footer reads
  **"App-server online"**; the main chat was created and selected.
- `wo-p2-008/d11/d11-02b-chatA-created.png` (2026-09-19 scene): footer
  reads **"Connection failed"** (red dot); sidebar shows "No chats".

Conclusion independently reproduced: the lab environment drifted between
the two days (runtime present on 09-18, absent on 09-19) — real environment
drift, not fabrication. The D9-side "runtime-observed" provenance labels
for WO-P2-006 are supported by the frames themselves.

---

## NEW findings beyond the prior run's five targets

### N-1 — WO-P2-009 "reload/copy-URL keybindings" is an unbacked scope claim (BROKEN; survives on main)

The ledger's WO-P2-009 **Scope delivered** line (ledger L459 at base;
still L459 on current main) and the changelog row (L677 base / main) claim:
"… Settings > Browser management surface (…), **reload/copy-URL
keybindings**."

Primary evidence against:

1. The delivery commit `0eed8c3` (PR #24, post-rebase) contains **no**
   reload or copy-URL keybinding code (diff searched); the pre-rebase
   over-delivery `5c795e7` likewise (its only KeyBinding additions are
   tab/shift-tab focus bindings for the Clear-Browsing-History modal); the
   integration fix `0375e0f` likewise.
2. The review-base tree contains **no copy-URL affordance at all** (search
   across ui.rs and browser.rs: no copy-URL command, keybinding, or menu
   action; the only clipboard-copy commands are chat-scoped deeplink /
   session-id / working-directory). No reload *keybinding* exists either —
   the registry has focusBrowserAddressBar / navigateBrowserBack /
   navigateBrowserForward (pre-existing since `7fcffe7`), and reload is
   reachable only via the pre-existing button and address-bar
   Enter-on-unchanged-URL.
3. Root cause is traceable: Wave-R reference research finding R2 (ledger
   L588) listed "… + reload/copy-URL keybindings" as part of the *gap*; the
   closure record then pasted the full gap list into *Scope delivered*.
4. Severity is confined to the ledger record: the parity §5.4 row itself
   does NOT claim the keybindings (it correctly records the official
   Ctrl+R / Ctrl+Shift+R overlay rows as runtime-observed reference and
   lists only the history store + revisit + Settings rows as delivered) —
   so no parity status is inflated. But a **real parity residual**
   (official reload/force-reload/copy-URL shortcuts vs none in Flauz) is
   thereby hidden behind an overclaiming scope line, and the prior
   adversarial run did not catch it (no mention in rwo-020/adversarial.md),
   nor did PR #30 remediate it. → follow-up FW-N1 (ledger correction +
   follow-up WO candidate).

### N-2 — d8c4-007 ≡ d8c4-008 byte-identical duplicate (BROKEN; unannotated on main)

See T1. md5 `23925833ed3bd5ecdca9ce15a67b7f29` for both
`d8c4-007-fork-dispatched.png` and `d8c4-008-aftermath.png`, recomputed at
base and matching the PR #30 manifest (which hashes both without
annotating the duplication). Same defect class as the remediated
d9-006≡d9-007 pair. → FW-N2.

### N-3 — WO-P2-007 scene scripts not archived in any ref (BROKEN pointer; reproducibility gap)

See T3. The README's "Scene scripts … (parity-lab/scenes/)" targets are
absent from every ref in the repository; `/home/z/parity-lab` is an
external host. wo-p2-007 is the only GUI-evidenced closure without its
scene scripts in-tree. → FW-N3.

---

## Cross-check against the prior RWO-022 run (rwo-020/adversarial.md @ 230f6fc)

Prior findings FW-1..FW-9 were re-derived here from primary evidence before
reading the prior text; agreement map:

| Prior finding | This pass |
| --- | --- |
| FW-1 6→5 binding-test count | **Confirmed independently** (5 tests counted at base; 5 doc sites listed 6) |
| FW-2 seven→six absent names | **Confirmed** (Composer Gap cell self-contradicts at base) |
| FW-3 §8.2 item 12 unannotated | **Confirmed** (base L465) |
| FW-4 §8.1 prose + §-ref drift | **Confirmed** (prose L404-407; §-refs at parity L449/730-735, ledger L669/671) |
| FW-5 D11 plain-Return script defect | **Confirmed and strengthened** (source requires `secondary: true`; D9 contrast; fresh VLM read of 02b) |
| FW-6 wo-p2-006 mislabeled/duplicate frames | **Confirmed by independent methods** (md5 + pixel-diff + fresh VLM) |
| FW-7 lab runtime pin/restore | Consistent with this pass's EQ-1 re-verification (open on main) |
| FW-8 wo-p2-005 md5 manifest + audit pointer | **Confirmed** (no manifest at base; 15 sites re-derived exactly) |
| FW-9 executor guard fall-through tests | Not re-attacked beyond confirming the resolution-layer tests exist (agreed open) |
| — | **New:** N-1 (WO-P2-009 scope overclaim), N-2 (d8c4 duplicate), N-3 (007 scene scripts) — none present in the prior run or its remediation |

The prior run's headline verdicts (five targets HELD on substance; zero
product-code defects) are **endorsed** by this independent pass: this
worker's own re-derivation found no product-code defect in any of the five
closures either — every BROKEN item above is a record/evidence-hygiene
defect, plus one scope-overclaim (N-1) that is likewise record-level (the
parity row stays honest).

## Verdict table

| Target | Closure substance | Record/evidence |
| --- | --- | --- |
| T1 WO-P2-005 picker fix + 15-site audit | **HELD** | no manifest at base (remediated); **NEW N-2 duplicate frame (open)**; D8c2 script self-label (annotated) |
| T2 WO-P2-006 side-chat semantics | **HELD** | d9-008 mislabel + d9-006≡d9-007 duplicate + d9-010 idempotent reopen (all confirmed independently; README remediated) |
| T3 WO-P2-007 Ctrl+P fix + F-A4 | **HELD** | **NEW N-3 scene scripts not archived (open)** |
| T4 WO-P2-008 no-runtime-lab + MAX 71→76 | **HELD** | 6-vs-5 count (remediated); D11 Return-key defect + mislabeled 02b/04 + manifest omission + incomplete root-cause attribution (FW-5/FW-7 open) |
| T5 §9 overrides + §8 counts | **HELD** | four stale/misdirected cells (all remediated on main) |
| EQ-1 D9/D11 environment drift | **RESOLVED** (re-verified with fresh VLM reads by this worker) | — |
| **NEW** N-1 WO-P2-009 scope overclaim | n/a (record-level) | **BROKEN — open on main** (ledger L459/L677) |

## Follow-up work-order candidates (from this pass)

- **FW-N1** (docs + WO): correct the WO-P2-009 Scope-delivered line
  (ledger L459 + changelog L677 on main) to drop "reload/copy-URL
  keybindings" or mark it NOT DELIVERED; open a small follow-up work order
  for the actual gap (official reload/force-reload/copy-URL shortcuts —
  runtime-observed reference rows in §5.4 — vs none in Flauz's registry).
- **FW-N2** (evidence hygiene): annotate the d8c4-007≡d8c4-008 duplicate
  in the wo-p2-005 manifest note (mirroring the d9-006/007 annotation
  pattern from FW-6).
- **FW-N3** (evidence hygiene): archive the D10/D10b scene scripts
  in-tree (or repoint the README to wherever they actually live) so the
  wo-p2-007 closure satisfies the reproducibility gate the way its sibling
  closures do.
- Endorse the still-open prior follow-ups: FW-5 (D11 script ctrl+Return +
  relabel), FW-7 (lab runtime pin/restore + D11 full-flow re-run), FW-9
  (executor guard fall-through tests, folded into WO-P2-012 scope).
