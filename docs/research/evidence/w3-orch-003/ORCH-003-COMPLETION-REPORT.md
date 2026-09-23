# ORCH-003 — Completion Report & Lead Gate Record (Wave 3)

**Status: MERGED (PR #51)** — the execution harness: the 10-state
lattice (prepare/execute/observe/verify/persist with
continue/recover/escalate first-class), event-sourced transitions
(observer-before-commit; recovery = replay_history over the machine's
OWN records — model output structurally excluded), the bounded
canonical-JSON telemetry trace, and the non-technical J-03 recovery
surface ("Picking up where we left off" / kept-vs-summarized /
Review-and-continue + Escalate-to-me; Ctrl+Alt+Shift+R; the d19/d21
focus disciplines).

## Lead gate record

- **Delivery**: worker session bb7d1360 (agents tab, GLM-5.3,
  Full-Stack — the grind survivor: two capacity-modal phantoms at
  create, then TWO turn deaths + a LOST TAB mid-work recovered by the
  Lead's fresh-tab swap + SENT-VERIFIED manual nudge), branch
  `feat/orch-003-harness`, single clean commit `6b50a3c` on base
  `19d16d5ab416` (exact — the base reconciliation with the order text's
  e9bee81 documented in the report; docs-only difference, the
  contracts identical). Bundle harvested from the worker pod
  (`leads-harvest/bb7d1360`, 47,068 bytes) — `git bundle verify` PASS.
- **Gate 0 (bundle)**: PASS — requires exactly 19d16d5ab416; one
  commit; 16 files +4568/−1, all within the owned set.
- **Gate 1 (source review)**: PASS — fakes.rs +144/−0 purely additive;
  runtime.rs/store.rs/registry.rs untouched; ContextCompiler and
  HarnessObserver are NEW seams in the NEW module (not edits to
  existing traits); **recovery is a fold over the machine's own
  transition records only** — execute details carry outcome status +
  named missing capabilities, never the runtime's summary text;
  `deny_unknown_fields` rejects injected summaries (the no-model-replay
  law structural); every transition emits through the observer BEFORE
  committing (observer rejects → no state change); corrupted histories
  (gap/fork/backwards-time/illegal transition/orphan) rejected;
  HarnessView exports plain data (the ORCH-004 seam).
- **Gate 2 (Lead-local)**: fmt clean; clippy `-p flauz-exec
  --all-targets` clean; `cargo test -p flauz-exec` **106/106** (81 lib
  [77 base + 4 new] + 11 conformance + 2 exec_dynamics + 12
  harness_dynamics — matches the worker's claim exactly);
  **codex-app `cargo check` EXIT 0** + the exact CI invocation
  `cargo clippy -p codex-app --all-targets` EXIT 0 after the gate-fix
  below. Worker's UI verification: the scratch-crate pattern (the
  picker-verify precedent — a SYMLINK to the real delivered file
  against inert stubs; 7/7 pure-logic tests incl. the forbidden-word
  copy scan and the focus-not-trapped assertions).
- **Lead gate-fix (`de4b792`)**: the recovery seam-test
  `include_str!("../../ui.rs")` path (the worker's scratch-crate
  layout sat one directory deeper) → `../ui.rs`, the house pattern —
  caught by CI clippy `--all-targets` (the PR-46 lesson re-learned:
  gate the TEST targets locally before pushing, not just the bin).
- **Contract deviations: NONE** (worker's report, confirmed by the
  Lead's review).

## Worker's COMPLETION REPORT (verbatim, as recovered)

```text
=== ORCH-003 COMPLETION REPORT ===` with those sections. Let me look at the WAVE3-WORK-ORDERS.md ORCH-003 entry to get the exact completion report requirements, plus verify the kernel compliance checklist §1–§7 items. All core evidence verified. Final checks — no-push confirmation, the scoped Escape seam, and the exact completion-report format required by the Wave-3 work order: [object Object] No upstream (not pushed) ✓, scoped Escape ✓. But the grep for "COMPLETION REPORT" in WAVE3-WORK-ORDERS.md returned nothing. Let me search for the completion report format in the work order docs — maybe it's in WORK-ORDER-TEMPLATE.md. [object Object] The completion report format is documented in prior evidence files. Let me look at one of the recent ones (CAP-001, since the worklog references the CAP-001 pattern) to see the exact format expected, and also check the WAVE3-WORK-ORDERS.md ORCH-003 entry for the exact acceptance criteria so my report addresses them precisely. [object Object],[object Object] I have the CAP-001 report format precedent. Now let me read the full ORCH-003 work order entry in WAVE3-WORK-ORDERS.md to make sure my completion report addresses every acceptance criterion exactly. It's at line 171 onwards. Let me read lines 171-395 or so. [object Object] I now have the full ORCH-003 work order. Note the acceptance criterion 1 says "Single clean commit on feat/orch-003-harness at base e9bee81" — but the dispatch message (user's original task, per summary) said base is `19d16d5` (which is the Wave-3 docs pin on top of e9bee81: "docs(research): Wave-3 base pinned to e9bee81 (the contracts are identical at 6368c2e/e9bee81 — docs-only on top; the workers' single clone base carries the Wave-3 orders)"). The user's latest message explicitly says: "single clean commit on feat/orch-003-harness at base 19d16d5ab41669197a63bb22809b6dada402313b". So the base used (19d16d5) is correct per the dispatch — the work order's "e9bee81" is reconciled by that commit message: workers clone at 19d16d5 which carries the Wave-3 orders on docs-only top of e9bee81. I should mention this in the report (base reconciliation).

Now let me also check the addendum §1-§3, §6-§7 sections of the Wave-3 work orders doc since ORCH-003 contracts include them. Let me read the beginning of the doc (lines 1-170). The work order is fully understood. Note the base reconciliation: the order says "at base e9bee81" but the workers' dispatch base is `19d16d5` (the docs-only pin carrying the Wave-3 orders on top of e9bee81 — exactly what the user's dispatch states). Let me read the addendum sections §1–§7 that ORCH-003 must comply with, to complete my report checklist: [object Object] Now I have the complete Wave-3 kernel addendum §1–§7 that ORCH-003 must comply with:

1. §1: Existing signatures frozen — no trait signature changes; fakes additive only
2. §2: Context is a projection, never a transcript — rebuildable from durable state without replaying model conversation; model/context switching never mutates canonical task state
3. §3: The harness is an explicit state machine, not a loop — prepare→execute→observe→verify→persist with continue/recover/escalate as FIRST-CLASS states; every transition event-sourced; recovery = reconstruction from durable state, never "start over"; reset/compaction visibly non-destructive
4. §4: Orchestration is a graph (ORCH-004's concern, not ORCH-003's)
5. §5: Procedure slice (ORCH-004's)
6. §6: Credentials stay references; canonical JSON discipline; task identity is sacred
7. §7: GUI slices follow the seven-layer rule; recovery language non-technical (never "context window", "token budget", "rehydration")

Now let me verify a few compliance points in the actual code to write an accurate report:

1. §1 — check no trait signature changes: diff of fakes.rs is additive only (+144), runtime.rs/store.rs untouched. Let me verify with git diff.
2. §6 — credentials as references: the test `no_credential_material_in_harness_state` covers this.
3. Task identity sacred: `harness_prepare_compiles_through_the_seam_and_survives_model_switch` covers model switch identity.

Let me check the diff of fakes.rs and lib.rs to make sure they're additive-only, and check that no other files were touched. The diff stat already shows only: ui.rs (seams), flauz_recovery.rs (new), fakes.rs (+144, 0 deletions), harness.rs (new), harness_telemetry.rs (new), lib.rs (+12), fixtures (new), harness_dynamics.rs (new). So no trait signature changes — runtime.rs, store.rs untouched. 

Let me quickly verify fakes.rs diff is purely additive and lib.rs is module lines + re-exports. I now have the full addendum §1–§7. Final compliance spot-checks — verify the fakes.rs and lib.rs diffs are purely additive (kernel §1) and no other files were touched: [object Object] Everything is verified:
- fakes.rs: 0 deletion lines (purely additive) ✓
- lib.rs: module lines + re-exports only ✓
- No other files touched (diff stat showed only the 16 owned files) ✓

Now let me check the seven-layer GUI discoverability in flauz_recovery.rs to write the line-by-line report. Let me look at the module doc header of flauz_recovery.rs which describes the layers. All compliance checks pass. Let me capture the seven-layer discoverability layout from the recovery module's header for the report, plus verify the J-17 activity connection and the task-stream Event emission mechanism: [object Object],[object Object] Everything is verified. Now let me run one final complete verification sweep to have exact numbers for the report, and then update the worklog and emit the report.

Let me get the final numbers:
- Commit SHA: 6b50a3cb13774dc0252137b44071e8e155db8e76
- Base: 19d16d5ab41669197a63bb22809b6dada402313b (1 commit ahead)
- Files: 16 changed, +4568/−1
- fmt: CLEAN
- clippy -D warnings: CLEAN
- tests: 81 + 11 + 2 + 12 = 106/106
- scratch-crate: 7/7
- Bundle: verified OK

Let me also double check the line count of each new file for the report:
- harness.rs: 2070 lines (from diff stat)
- harness_telemetry.rs: 362
- flauz_recovery.rs: 997
- fakes.rs: +144
- lib.rs: +12
- ui.rs: +67/-1 (66 insertions, 1 deletion — diff stat says 67 ± which is 66+1? Actually "67 +-" means 67 total changes. Let me get exact numbers.)

Let me get the exact insertion/deletion counts and also count the ORCH-003 tags in ui.rs (12 tags found earlier: lines 134, 3956, 4062, 4101, 4264, 4348, 5034, 5563, 5616, 6379, 8258, 19337 — that's 12).

Also the number of tests in harness_dynamics.rs: 12 (from test names: lifecycle, illegal, recovery-replay, recovery-requires-legal-history, prepare-compile-model-switch, capability-gap-escalate, view-export, events-round-trip-fixtures, no-credential, trace-durations, trace-round-trip-bounded, view-fixture-round-trip = 12 ✓)

And flauz_recovery.rs tests: 7 pure-logic tests ✓

Let me get exact insertions/deletions for ui.rs and check the exec_dynamics/conformance test counts are unchanged from base (to confirm no existing test was touched — actually harness_dynamics.rs is new, and the lib tests went from 77 to 81 (4 new), conformance 11 unchanged, exec_dynamics 2 unchanged... wait, at base it was 90 total. 81+11+2 = 94 now without harness_dynamics; 94 - 4 = 90 ✓. So 4 new lib tests were added inside src (in harness.rs/harness_telemetry.rs unit tests).

Let me verify the 4 new lib tests location, get exact ui.rs diff numbers, then finalize. [object Object] Let me get the exact 4 new lib unit test names:
- harness.rs: 18 fns in test mod but only 4 test fns? Let me check the test names more precisely.
- harness_telemetry.rs: 1 test fn (span_outcomes_have_distinct_canonical_names)

Total new lib tests should be 4. Let me check. [object Object] Now I have complete verification:

New lib unit tests (4):
1. `registered_event_types_follow_the_grammar` (harness.rs)
2. `every_state_is_reachable_and_terminals_are_final` (harness.rs)
3. `detail_round_trips_and_rejects_unknown_fields` (harness.rs)
4. `span_outcomes_have_distinct_canonical_names` (harness_telemetry.rs)

All verification is complete:
- fmt: CLEAN
- clippy -D warnings: CLEAN  
- test -p flauz-exec: 106/106 (81 lib [77 base + 4 new] + 11 conformance + 2 exec_dynamics + 12 harness_dynamics)
- scratch-crate recovery-verify: 7/7
- Single commit 6b50a3c on feat/orch-003-harness @ base 19d16d5ab41669197a63bb22809b6dada402313b
- Bundle verified OK at /home/z/my-project/ORCH-003-delivery.bundle
- No upstream (not pushed)
- Working tree clean
- 16 files: +4568/−1
  - ui.rs: +66/−1 (named seams, 12 ORCH-003 tags)
  - flauz_recovery.rs: +997 (new)
  - fakes.rs: +144/0 (additive)
  - harness.rs: +2070 (new)
  - harness_telemetry.rs: +362 (new)
  - lib.rs: +12/0
  - 9 fixtures + harness_dynamics.rs (+908)

Wait — one thing to double check. The user message says the bundle command from the original order was:
```
git bundle create /home/z/my-project/ORCH-003-delivery.bundle 19d16d5a749849e3e48aef0c9b45db7a4d8e4a2c2..feat/orch-003-harness
```
But the summary's SHA `19d16d5a749849e3e48aef0c9b45db7a4d8e4a2c2` conflicts with the actual full SHA `19d16d5ab41669197a63bb22809b6dada402313b`. The actual base in the repo is `19d16d5ab41669197a63bb22809b6dada402313b` (confirmed by `git rev-parse 19d16d5`). The user's current message says "at base 19d16d5ab41669197a63bb22809b6dada402313b" which matches the real SHA. The bundle was verified: contains ref 6b50a3c, requires 19d16d5ab41669197a63bb22809b6dada402313b. 

The bundle already exists and is valid. But let me re-create it to be 100% sure it reflects the exact range with the correct base SHA (the user's latest message gives the correct full SHA). Actually the bundle verify already showed it requires exactly `19d16d5ab41669197a63bb22809b6dada402313b` and contains `6b50a3cb13774dc0252137b44071e8e155db8e76` — the exact current HEAD. It's correct. No need to recreate. But recreating with the exact command format would be harmless and gives certainty. Let me re-create it using the user's stated base to be safe, then re-verify.

Actually — careful. If the previous session created the bundle with a different base range... but `git bundle verify` said "The bundle requires this ref: 19d16d5ab41669197a63bb22809b6dada402313b" — that's the exact base. It's fine. But to be extra safe and match the user's command exactly, let me recreate the bundle with the correct base SHA and verify again. This is idempotent and safe.

Also, I should update the worklog.md to append a final verification entry (per the system prompt instructions about the shared worklog). The worklog already has the ORCH-003 entry from the previous session. I'll append a short final-verification section.

Then emit the completion report. Let me also double-check the remaining details for the report:

1. Five acceptance criteria evidence:
   1. Single clean commit on feat/orch-003-harness at base (e9bee81 reconciled: dispatch base 19d16d5, docs-only on top of e9bee81 — the work order says e9bee81, the dispatch says 19d16d5; the commit 19d16d5 message itself says "the workers' single clone base carries the Wave-3 orders"). Owned files only ✓
   2. State machine: every transition event-sourced (harness_lifecycle_transitions_are_event_sourced), recovery reconstructs from machine's own history (harness_recovery_replays_the_machines_own_history_never_model_output), no model-output replay ✓
   3. Recovery UX: seven layers, non-technical copy tested (recovery_copy_uses_user_language_only), keyboard never trapped (recovery_focus_never_traps) ✓
   4. Canonical JSON + fixtures (harness_events_round_trip_canonical_json_fixtures, harness_trace_round_trips_and_is_bounded, harness_view_fixture_round_trips), no credential material (no_credential_material_in_harness_state), deterministic ✓
   5. Contract deviations: NONE ✓

2. Kernel compliance §1–§7:
   - §1: existing signatures frozen — 0 deletions in fakes.rs, runtime.rs/store.rs/registry.rs untouched, no trait changes; ContextCompiler and HarnessObserver are new traits in the NEW harness.rs (not changes to existing traits) ✓
   - §2: context as projection — prepare compiles through the ContextCompiler seam (the existing public compile step wired caller-side); model switch = re-prepare, task identity never forks ✓
   - §3: harness as explicit state machine — 10 states, continuations first-class, every transition event-sourced, recovery = reconstruction from own history ✓
   - §4: orchestration graph — n/a for ORCH-003 (ORCH-004's law), but the HarnessView plain-data export is the seam ORCH-004 consumes ✓
   - §5: procedure slice — n/a (ORCH-004) ✓
   - §6: credentials as references (no_credential_material_in_harness_state), canonical JSON (v/snake_case/deny_unknown_fields/no floats/RFC3339-Z), task identity sacred (task id never changes — lifecycle test) ✓
   - §7: seven-layer GUI rule + non-technical recovery language (tested forbidden words) ✓

3. Seven layers line-by-line (from module header):
   1. Visible primary entry: state-driven banner on task surface, always visible while Recovering/Escalated, absent when nothing to recover
   2. Contextual affordance: "What we kept / What was summarized" disclosure panel
   3. Palette fallback: "Resume this task where it left off" row under Workspace group
   4. Stateful empty state: nothing to recover → NO banner; chord/palette say so honestly
   5. Success-state continuation: "Resumed from <n> events · everything kept except what was summarized" + next step
   6. Keyboard path: Ctrl+Alt+Shift+R + scoped Escape
   7. Honest unavailable state: escalated "Needs you" with reason + review-and-continue path

4. Known limitations:
   - codex-app cannot link in this sandbox (pre-existing libpipewire/xcap gap, no root) — verified identical at base; UI verified via scratch-crate pattern (CAP-001 precedent); Lead independently compiles
   - The recovery banner is wired as a view-model seam (set_recovery_view) — no invented data until a later wave wires the harness view → the honest empty state
   - J-17: harness events surface in the activity feed — the harness emits through the HarnessObserver seam (world envelope assignment is Lead/world-side; the observer validates on write)

Wait, I should double check one thing — the J-17 connection: "the harness events surface in the activity feed". The harness emits HarnessEventRecord through the HarnessObserver seam. The existing task-stream Event types... the work order says "Every transition emits an Event to the task stream (existing Event types; new type names need the frozen-format discipline)". The implementation registers 10 harness.* event type names through the observer seam. The FakeHarnessObserver is the in-memory task-stream seam. The world-side envelope assignment happens Lead-side. This is what the worklog says. Good.

5. The exact test results:
   - cargo fmt --all --check → CLEAN (exit 0)
   - cargo clippy -p flauz-exec --all-targets → CLEAN (0 warnings, 0 errors; re-verified with -D warnings after touch)
   - cargo test -p flauz-exec → 106 passed, 0 failed (81 lib + 11 conformance + 2 exec_dynamics + 12 harness_dynamics)
   - Scratch crate /home/z/recovery-verify → 7 passed, 0 failed

6. Bundle: /home/z/my-project/ORCH-003-delivery.bundle — created via git bundle create <path> <base>..<branch>; git bundle verify OK; contains refs/heads/feat/orch-003-harness @ 6b50a3c; requires 19d16d5ab41669197a63bb22809b6dada402313b. NOT pushed (no upstream).

Let me also verify the fixtures are one-line canonical JSON (they showed as "1 +" in diff stat, meaning single-line files). Let me peek at one fixture to confirm canonical shape. And check "v":1, snake_case etc. Quick check.

Then: recreate the bundle with exact command, verify, update worklog, emit report.

Actually, let me also double-check the claim "90/90 at base" for context in the report (tests at base). Not strictly needed — the worklog said so and current totals reconcile (77+4=81 lib). Fine.

One more check — the exec_dynamics and conformance test files were NOT modified (diff stat shows they're not in the changed files list — only harness_dynamics.rs is new). ✓

Let me peek at a fixture and then finalize. Let me verify a fixture's canonical-JSON shape and then re-create the bundle with the exact dispatch command for certainty: [object Object] Fixtures are canonical JSON: `"v":1`, snake_case, RFC3339-Z timestamps, single-line deterministic serialization.

Now finalize:
1. Re-create the bundle with the exact command (using the correct full base SHA)
2. Verify the bundle
3. Append final verification to worklog.md
4. Emit the completion report Fixtures are canonical (v:1, snake_case, RFC3339-Z, single-line). Now re-creating the bundle with the exact dispatch command and verifying: [object Object] Bundle re-created and verified. Now append the final verification entry to the worklog and emit the completion report.

Let me read the current worklog first (I saw the tail earlier — it ends with the ORCH-003 entry). I'll append a new section for the final re-verification. Bundle verified. Appending the final re-verification record to the shared worklog: [object Object] [object Object] All done. Now emit the completion report in the exact format required, following the CAP-001 precedent structure with all required sections:
- Work Order ID
- Base
- BRANCH/COMMITS
- Changed files
- Implementation summary
- Test commands + exact results
- Kernel compliance checklist §1–§7
- Seven-layer GUI discoverability line-by-line
- 5 acceptance evidence
- Known limitations
- Contract deviations
- Follow-up === ORCH-003 COMPLETION REPORT ===

**Work Order ID:** ORCH-003 — The execution harness: prepare/execute/observe/verify/persist
with continue/recover/escalate as first-class states, event-sourced telemetry, and the
non-technical recovery surface (J-03) — F6 Wave 3, Worker B

**Base:** `19d16d5ab41669197a63bb22809b6dada402313b` (the dispatch base — the docs-only
Wave-3 pin on top of `e9bee81`; the contracts are identical at e9bee81, the commit message
on 19d16d5 states the workers' single clone base carries the Wave-3 orders, so the order's
"base e9bee81" and the dispatch base are the same contract surface)

**BRANCH|COMMITS:** `feat/orch-003-harness` | exactly 1 clean commit:
`6b50a3cb13774dc0252137b44071e8e155db8e76` — 16 files changed, +4568/−1.
**Not pushed** (no upstream configured, as ordered). Delivery bundle:
`/home/z/my-project/ORCH-003-delivery.bundle` (created with the exact dispatch command
`git bundle create … 19d16d5ab41669197a63bb22809b6dada402313b..feat/orch-003-harness`;
`git bundle verify` → "is okay", contains `refs/heads/feat/orch-003-harness @ 6b50a3c`,
requires the base ref). Working tree clean.

**Changed files/surfaces:**
- `crates/flauz-exec/src/harness.rs` — NEW (2070 lines): the state machine
- `crates/flauz-exec/src/harness_telemetry.rs` — NEW (362 lines): the trace projection
- `crates/flauz-exec/src/fakes.rs` — additive only, +144/−0: `FakeContextCompiler`,
  `FakeHarnessObserver`, `FAKE_CONTEXT_REF`, `fake_recovery_brief`
- `crates/flauz-exec/src/lib.rs` — +12/−0: module lines + re-exports only
- `crates/flauz-exec/tests/harness_dynamics.rs` — NEW (908 lines, 12 tests)
- `crates/flauz-exec/tests/fixtures/w3/**` — NEW, 9 canonical-JSON fixtures
  (harness-event: typical/minimal/boundary/invalid-unknown-field/invalid-detail-mismatch;
  harness-trace: typical/minimal/invalid-unknown-field; harness-view: typical)
- `crates/codex-app/src/ui/flauz_recovery.rs` — NEW (997 lines): the J-03 surface
- `crates/codex-app/src/ui.rs` — NAMED SEAMS ONLY, +66/−1, 12 ORCH-003-tagged points

**Implementation summary:**
- **PART 1 — the state machine.** `HarnessStateKind` is the 10-state lattice
  (Preparing/Executing/Observing/Verifying/Persisting/Continuing/Recovering/Escalated/
  Done/Failed) with `transition_is_legal` as a total function; the three continuations
  (continue/recover/escalate) are FIRST-CLASS states, and unit tests prove every state
  reachable + terminals final. `TaskHarness::prepare` attaches model/runtime/environment
  through the EXISTING public types and compiles context through the `ContextCompiler`
  seam (the EXISTING flauz-context public compile step wired caller-side — ORCH-002's
  engine later replaces the internals behind the same seam). `execute_turn` runs one turn
  through the caller-supplied `Box<dyn AgentRuntime>`; observe/verify/persist record the
  world pattern as data through the store seam. **Every transition emits a
  `HarnessEventRecord` through the `HarnessObserver` seam BEFORE committing** (observer
  rejects → no state change), registering ten `harness.*` event types under the frozen
  grammar. **Recovery = `replay_history`: a fold over the machine's OWN transition
  records only.** Model output structurally cannot enter the stream — execute details
  carry the outcome status and named missing capabilities, never the runtime's summary;
  `deny_unknown_fields` everywhere; corrupted histories (gap/fork/backwards-time/illegal
  transition/orphan record) are rejected. `ReplayedHarness` + `TaskHarness::resume`
  re-attach execution state; `HarnessView` exports the machine as plain data (the
  ORCH-004 seam). A model switch is a re-prepare: task identity is never forked.
- **PART 2 — telemetry.** `HarnessTrace::project` derives a bounded (256-span)
  canonical-JSON trace from the records: spans with integer-ms durations and outcome
  classifications (phase / turn_completed / turn_capability_gap / recovered / escalated /
  completed / failed) + `HarnessTraceSummary` (the observability counts). Deterministic —
  timestamps are caller-supplied; serialize → drop → reload → equality proven.
- **PART 3 — the recovery UX.** The shell-family banner renders a plain `RecoveryView`
  view-model (the wiring seam a later wave drives from the harness view + recovery
  brief — no invented data until then): "Picking up where we left off" → what was kept /
  what was summarized (the compaction record in user words) → "Review and continue" +
  "Escalate to me"; escalated is the explicit visible "Needs you" state with the recorded
  reason. Copy lives in one registry and is unit-tested non-technical. Keyboard:
  Ctrl+Alt+Shift+R focuses the banner (the only focus-claiming path — the d21 PTY
  discipline), scoped Escape returns focus to the task surface through the 017 restore
  contract (the d19 never-trapped discipline).

**Tests/commands + exact results** (sandbox HAS cargo 1.97.1 — the pinned workspace
rust-version — but CANNOT link the GUI crate: codex-platform → xcap →
libpipewire-0.3/libxkbcommon-x11 system libs absent, not installable without root;
verified identical at base, the MOD-001-documented limitation; stated plainly, the Lead
independently compiles and gates):
- `cargo fmt --all --check` → CLEAN (exit 0, no diffs)
- `cargo clippy -p flauz-exec --all-targets` → CLEAN (0 warnings, 0 errors;
  re-verified with a forced fresh `-- -D warnings` run)
- `cargo test -p flauz-exec` → **106 passed / 0 failed**: 81 lib (77 at base + 4 new unit
  tests) + 11 conformance + 2 exec_dynamics + **12 harness_dynamics**
  (lifecycle event-sourcing — 1 event/transition, strict seq, task id never changes;
  illegal-transition rejection live + full lattice table incl. no-dead-states +
  Fail-from-every-active-phase; THE recovery law — serialize → drop → reload → replay,
  summary text absent from stream, injected summary field rejected, resume → recover →
  continue-with-brief, turn counter continues; corrupted-history rejection; compile seam
  + model switch (identity survives, stale model rejected); capability gap → visible
  escalation → continue → browser turn completes; view export + canonical round-trip;
  fixture round-trips byte-for-byte; no credential material; telemetry durations/
  outcomes/summary/bound; trace + view fixture round-trips)
- UI module — scratch-crate pattern at `/home/z/recovery-verify` (the CAP-001
  picker-verify precedent: a symlink to the REAL delivered `flauz_recovery.rs`, not a
  copy, compiled against inert gpui/gpui-component/codex-core stubs): `cargo test` →
  **7 passed / 0 failed** — nothing-to-recover renders no banner; palette query resolves
  to the resume row; copy uses user language only (forbidden words: "context window",
  "token budget", "rehydration", "token", "compact*", "harness", "state machine", …);
  focus never traps; success copy names kept + summarized; view transitions stay honest;
  all 8 ui.rs seam registrations source-inspected.

**Kernel compliance (Wave-3 addendum §1–§7):**
- §1 frozen signatures — COMPLIANT: no trait changes; `runtime.rs`/`store.rs`/`registry.rs`
  untouched; `fakes.rs` diff is +144/−0 (purely additive); `ContextCompiler`/
  `HarnessObserver` are new seams in the NEW module, not edits to existing traits.
- §2 context is a projection — COMPLIANT: prepare compiles through the existing public
  compile step (caller-side); re-prepare for model B never mutates canonical task state
  and never forks the task (tested).
- §3 harness is an explicit state machine — COMPLIANT: 10 states, continuations
  first-class, every transition event-sourced, recovery is reconstruction from the
  machine's own durable history — never "start over"; compaction is visibly
  non-destructive (the kept/summarized disclosure).
- §4 graph of attributed agents — N/A for ORCH-003 (ORCH-004's law); the plain-data
  `HarnessView` export is the seam ORCH-004's nodes consume.
- §5 procedure slice — N/A (ORCH-004).
- §6 credentials/canonical JSON/identity — COMPLIANT: credentials stay references
  (`no_credential_material_in_harness_state` test); canonical JSON throughout
  (`"v":1`, snake_case, `deny_unknown_fields`, no floats, RFC3339-Z, deterministic —
  fixture round-trips are byte-for-byte); task identity sacred (lifecycle test asserts
  the task id never changes, including across model switch and recovery).
- §7 seven-layer GUI rule — COMPLIANT (line-by-line below); recovery language
  non-technical (tested).

**Seven-layer GUI discoverability (flauz_recovery.rs, line-by-line):**
1. Visible primary entry — the state-driven banner on the task surface (ui.rs mount
   directly in the task-surface render neighborhood): always visible while
   Recovering/Escalated; absent when there is nothing to recover (never permanent).
2. Contextual affordance — the "What we kept / What was summarized" disclosure panel
   inside the banner (kept families in user words: work products, verified results,
   notes; the compaction record named as what was summarized).
3. Palette fallback — the "Resume this task where it left off" row under the Workspace
   group (title/description/shortcut/icon/requires_selected_chat + dispatch seam);
   firing with nothing to recover surfaces honest guidance.
4. Stateful empty state — nothing to recover → NO banner, task surface unchanged; the
   chord/palette say so honestly instead of a silent no-op.
5. Success-state continuation — "Resumed from {n} events · everything kept [except what
   was summarized]" with the next step (continue working in the conversation).
6. Keyboard path — Ctrl+Alt+Shift+R (the recovery chord, letter family like the picker's
   M) + scoped Escape `FlauzRecovery`; focus returns to the task surface on close.
7. Honest unavailable state — the escalated "Needs you" state: explicit, visible, with
   the recorded reason and the review-and-continue path.

**Acceptance evidence (the 5 criteria):**
1. Single clean commit on `feat/orch-003-harness` at the dispatched base (19d16d5 =
   e9bee81 + docs-only); owned files only — verified by `git rev-list --count` = 1 and
   the numstat above.
2. Every transition event-sourced (1 event/transition, strict seq — tested live);
   `recover` reconstructs from the machine's own history (serialize → drop → reload →
   replay, turn counter continues); model output does NOT replay (the runtime's summary
   text is asserted absent from the stream, and an injected summary field is rejected by
   `deny_unknown_fields`).
3. Recovery UX: seven layers; non-technical copy tested (forbidden-word list + verbatim
   work-order phrases asserted); keyboard never trapped (request-once focus claim,
   chord-only, scoped Escape, 017 restore — tested).
4. Canonical JSON + 9 fixtures under `tests/fixtures/w3/**` (byte-for-byte round-trips);
   no credential material (tested); deterministic (caller-supplied timestamps;
   serialize→drop→reload→equality).
5. Contract deviations: **NONE** (block check passes).

**Known limitations:**
- The GUI crate cannot LINK in this sandbox (pre-existing libpipewire/xcap gap at base,
  no root) — the UI module is verified via the scratch-crate pure-logic pattern; the
   Lead independently compiles the app crate at the gate.
- The banner renders a view-model seam (`set_recovery_view`): until a later wave wires
  the harness view + recovery brief into it, there is no banner and no invented data —
  the honest empty state by design.
- The harness events reach the task's event stream through the `HarnessObserver` seam;
  world-envelope assignment remains Lead/world-side (envelope-free, opaque-value
  discipline) — J-17's activity-feed surfacing lands at the merged binary.

**Contract deviations:** NONE.

**Follow-up (for the Lead):**
- ORCH-002 wires the engine behind the `ContextCompiler` seam (same public types).
- ORCH-004's graph nodes drive `TaskHarness` and consume `HarnessView` as plain data.
- A later wave populates `RecoveryView` from `HarnessView` + `RecoveryBrief` (the
  `set_recovery_view` seam) and assigns world envelopes to the `harness.*` event types.
- Lab scenes for the Lead gate: J-03 banner (copy, chord, focus return) + escalation
  visible; the Wave-3 harness step drives prepare→…→persist with fake runtimes.

The delivery is complete: single clean commit `6b50a3c` on `feat/orch-003-harness`, bundle
verified at `/home/z/my-project/ORCH-003-delivery.bundle`, not pushed, sandbox working
tree left intact.
```


## Post-merge notes

- ORCH-002's engine wires behind the `ContextCompiler` seam (the same
  public types); ORCH-004's graph nodes drive `TaskHarness` and
  consume `HarnessView` as plain data.
- A later wave populates `RecoveryView` from `HarnessView` +
  `RecoveryBrief` (the `set_recovery_view` seam) and assigns world
  envelopes to the `harness.*` event types (J-17's activity-feed
  surfacing).
- The Wave-3 integration gate step drives prepare→…→persist with
  fake runtimes + the J-03 lab scenes at the merged binary (banner
  copy, chord, focus return; escalation visible).
