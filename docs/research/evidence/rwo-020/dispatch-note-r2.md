# RWO-020 duplicate-dispatch note (for Tech Lead adjudication)

**Situation:** the remote branch `research/rwo-020` already carried a
delivery of this same work order when this dispatch attempted its push:

- Remote (first arrival): `research/rwo-020` @ `734b1e7cf34f9d51fbf24ce260260ab03c2b231a`
  — one docs commit on the same base `d479c7b9e725e1af1353140d49226b3f9a16be9d`,
  delivering `docs/research/evidence/rwo-020/rubric.md` (594 lines) +
  `notes.md` (104 lines).
- This dispatch (second arrival): `research/rwo-020-alt` @ this branch's
  tip — one docs commit on the same base, delivering
  `docs/research/evidence/rwo-020/rubric.md` (678 lines; 13 rows, 64
  criteria) + this note.

**Handling (per AGENTS.md concurrent-work rules and the WO-P2-008 duplicate
precedent):** the existing remote branch was NOT force-pushed, reverted, or
reformatted. Both lineages are preserved for adjudication — same pattern as
the WO-P2-008 duplicate delivery (`c37c21b` vs `64b28f6`), where the Lead
kept the verified lineage and recorded the duplicate as redundant.

**Differences the Lead may care about when choosing:**

1. This rubric covers **13 rows** (the 8 `complete` rows — including Git
   process hygiene, Marketplace admin-disabled install, and the
   Stable-failure regression controls, which the "at minimum" list omits —
   plus the 5 named partials); 64 criteria total.
2. Source anchors in this rubric were **re-verified at `d479c7b`** during
   construction (e.g. `KEYBOARD_SHORTCUT_COMMAND_IDS` 76/76 at
   `codex-core/src/lib.rs:137`, `MAX_KEYBOARD_SHORTCUT_COMMANDS = 76` at
   `:132`, `SettingsSection::DEFAULT_NAV_SECTIONS` 15-entry const at
   `ui.rs:2522`, `needs_attention_task_ids` at `lib.rs:4757`,
   `LocalProjectSummary.folders` at `lib.rs:914`, `FeedbackClassification`
   5-entry `ALL` at `lib.rs:4359`, `resolve_codex_binary` order at
   `codex-platform/src/lib.rs:261`, `useStateDbOnly` wire test at
   `codex-protocol/src/lib.rs:4083`, `read_to_string` count in codex-core
   = 0).
3. Calibration finding recorded for B: terminal pipelines that strip
   bare-`[m` ANSI-like sequences render `#[must_use]` as `#ust_use]`,
   producing phantom "source corruption" — the rubric instructs byte-level
   (`od -c`) confirmation before any corruption report (rubric §0.4.4).
4. Honest residual recorded: R-05-c — no dedicated multi-root
   white-screen acceptance test was located by static search at `d479c7b`
   (the control is architectural + the WO-P1-003 battery); B records the
   test-anchor status rather than fabricating one.

**Either lineage satisfies the work order's deliverable** (rubric on a
research branch, derived from the repo's own reference evidence); the Lead
picks one (or merges selectively) and adjudicates the other redundant, per
program precedent.
