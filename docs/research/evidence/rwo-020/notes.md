# RWO-020 — derivation notes (supporting `rubric.md`)

Task ID: RWO-020. Base: `main` @ `d479c7b9e725e1af1353140d49226b3f9a16be9d`
(verified as origin/main HEAD at clone; branch `research/rwo-020` created
from it). These notes record the derivation decisions behind
`rubric.md` so the Tech Lead and Worker C can audit them.

## N1 — Row selection (why 13 rows)

The task directive names "the 8 `complete` + the highest-traffic `partial`
rows — at minimum: [10 names]". The named list contains only 5 of the 8
complete rows (Runtime bootstrap; Side chats; Keyboard shortcut reference;
Feedback; Multi-root workspace). PR §8.1 enumerates the complete set as:
Runtime bootstrap; Multi-root workspace handling (regression control); Git
process hygiene; Marketplace admin-disabled install; Keyboard shortcut
reference; Feedback; Stable-failure regression controls; Side chats. To
satisfy "the 8 complete" literally, the rubric covers all eight, plus the
five named partial rows (Projects and chats; Settings shell; Keyboard and
accessibility; Notifications and tray; Activity view & unread attention) —
**13 rows, 71 criteria**. Nothing was dropped from the directive's minimum.

## N2 — Version-tagging decisions

- `Runtime bootstrap / Marketplace / Feedback / Keyboard shortcut
  reference / Notifications / stable-failure controls` → 26.721 baseline
  `[historical-record]` (PM/KF); no delta evidence touches them.
- `Side chats` → 26.707 note + current-docs command set
  (`[historical-record + docs-derived]`, confidence medium-high per A
  §1/WO-P2-006) — tagged 26.721-era in the rubric with the medium-high
  caveat carried.
- `Multi-folder projects` (inside Projects and chats) → 26.715, i.e.
  **in-baseline** per A §6 (this is why WO-P1-003 was P1).
- `Activity view / unread attention bindings` → 26.727, i.e. **inside the
  26.825 current target** (PR §9 override 4: P3 polish → P2 by the
  version-skew rule). The rubric double-marks this in ACT-C* and its
  attack note (mislabeling in either direction is the attack).
- Everything from `codex-linux/` (LX) is 26.908 ref+1 and is only ever
  cited for naming/confirmation with the skew label — the rubric's
  KSR/KAX attack notes make this the #1 trap.
- Deferred set (cloud environments, scheduled tasks, Pets, Appshots,
  Sites slices, cloud-side pins/snapshots) → documented deferrals,
  excluded from every criterion; ACT-C6/PRJ bounds/SET-C6/NTR-C4 encode
  the "must not count, must not fake" rule as documentation checks.

## N3 — Evidence-class asymmetry (the rubric's central design fact)

For the 8 complete rows, the **official** side is evidence-layered
(`[historical-record]` PM/KF, `[docs-derived]` A), not runtime-observed —
the official Windows/macOS app never ran in this program; only the 26.908
Linux preview did (ref+1). Therefore B's verification of these rows is
**Flauz-side-heavy**: source-read + unit-test + Flauz GUI scenes, with the
official side pinned to the repo's own recorded evidence. The rubric says
this explicitly in §0-§2 so B never invents an official-side check that
cannot exist in this lab (and C flags any such invention).

## N4 — EQ-1: the D9 vs D11 lab-runtime discrepancy (recorded, not adjudicated)

- WO-P2-006 D9 (2026-09-18) created a main chat in-lab by composer submit
  (`d9-wo-p2-006-side-chats-verify.sh` step 1; VLM evidence shows sidebar
  rows "side chat fixture main" and later "quick aside question").
- WO-P2-008 D11 (2026-09-19) states the lab has no `codex` CLI, the app
  spawns it as its runtime, and without it the sidebar shows "No chats"
  (honest no-selection paths captured instead).

Both are archived evidence-of-record. This rubric does not resolve the
discrepancy (environment drift between runs is a Lead/lab question); it
records it as **EQ-1** and makes every thread-dependent GUI criterion
explicitly conditional on what `resolve_codex_binary` currently resolves
in the lab — with the D11b-class honest bound as the documented fallback.
Follow-up WO candidate #5 (rubric §6) proposes the lab-infrastructure fix.

## N5 — Row-boundary decisions (override-9 discipline)

"Multi-root workspace handling" (complete, Windows white-screen control)
and the multi-folder capability inside "Projects and chats" (partial) are
**different capabilities** per PR §9 override 9. The rubric keeps their
criteria and evidence strictly separate (MRW-* vs PRJ-C3..C6) and puts the
conflation attempt in both attack surfaces. SFR-C1/C4 similarly demand
independent citations rather than evidence reuse with MRW/GPH.

## N6 — Counting integrity at the review base

Two numbers drifted during the wave and the rubric forces a recount at
the review base instead of trusting archived counts:
- registry length / `MAX_KEYBOARD_SHORTCUT_COMMANDS`: 72 ids (SWEEP base
  `3c9f113`) → 76 with the WO-P2-008 four bindings (verified = 76 at
  `d479c7b`, `crates/codex-core/src/lib.rs:132`); SCH-C6/KAX-C2 require
  the recount at B's base.
- advertised overlay rows: 47 at `3c9f113` (SWEEP §F) — must be recounted
  (KSR-C2) because the registry grew.
- palette entries: 51 after WO-P2-004 (`PaletteCommand::ALL` 45→51) —
  KAX-C7 verifies entries render, not a stale count claim.

## N7 — What RWO-020 deliberately did NOT do

- No Flauz verification of any criterion (B's job) — including the
  source-reads in Appendix A, which are rubric grounding (do the anchors
  exist so B can be pointed at them), not verdicts.
- No cargo/test runs, no GUI scenes, no official-app runs (the lab bounds
  are documented in the rubric for B instead).
- No edits outside `docs/research/evidence/rwo-020/`; no product code; no
  parity-report/ledger changes (status vocabulary is C2/Tech Lead-owned).
- The token used for clone/push appears only in the git command lines,
  never in any file, commit message, or report field.
