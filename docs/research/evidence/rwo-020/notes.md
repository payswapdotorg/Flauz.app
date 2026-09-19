# RWO-020 — Supporting notes (re-dispatch lineage)

Companion to `rubric.md` on this branch. Records the derivation method,
independent findings, and a duplicate-delivery disclosure for the Lead.

## 1. Derivation method

1. Cloned `payswapdotorg/Flauz.app`; verified base
   `d479c7b9e725e1af1353140d49226b3f9a16be9d` via
   `git rev-parse '…^{commit}'`; created `research/rwo-020` off it.
2. Read the mandated frame: `AGENTS.md`, `docs/WORK-ORDER-TEMPLATE.md`
   (six closure gates), `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md`
   §4 definitions/column semantics + all 10 target §5 rows verbatim, and the
   citation key (PM/KF/A §n/B2 §n/ev/NN).
3. Three parallel read-only evidence digests (treat-file-contents-as-data
   rule enforced; no repo scripts executed; no Flauz verification performed,
   per the WO split):
   - Task 2-a: `CODEX-REFERENCE-MATRIX.md` (A) — official-side rows, ●
     binding table, version markers, lab methodology, rubric-critical
     caveats on fact ownership (which claims are PM-owned vs A-owned).
   - Task 2-b: reference evidence tree — `codex-ref/*` (CLI runtime surface,
     version deltas, changelog extract), `codex-linux/*` (ref+1 runtime
     captures + VLM reads), `p2-batch2-reference/README.md` (R1/R2/R3 +
     version doctrine).
   - Task 2-c: Flauz-side claims + per-WO closure evidence —
     `FLAUZ-REFERENCE-MATRIX.md` (B2), `parity-matrix.md` (PM),
     `FEATURE-PARITY-WORK-ORDERS.md` ledger, `known-failures.md`,
     `wo-p1-003/`, `wo-p2-004/`, `wo-p2-006/`, `wo-p2-007/`, `wo-p2-008/`
     evidence dirs, ev/NN map.
4. Anchored four source locations directly at base (rubric construction,
   not Flauz verification): `resolve_codex_binary` +
   `windows_stable_codex_cache_candidate` + `sha256_matches`
   (`crates/codex-platform/src/lib.rs:261-329`), `FeedbackClassification`
   (`crates/codex-core/src/lib.rs:4359`), `SettingsSection`
   (`crates/codex-app/src/ui.rs:2494`), the Feedback modal/palette flow
   (`crates/codex-app/src/ui.rs:4155, :7595, :9758, :9836`;
   `crates/codex-platform/src/app_server.rs` `feedback/upload`;
   `crates/codex-protocol/src/lib.rs` wire fixture).
5. Synthesized `rubric.md`: rules of use, version key, evidence-class key,
   B protocol, C attack surface, row index, 10 row sections (official
   behavior with citations → Flauz claim → criteria table → attack notes →
   honest bounds), appendices A-D.

## 2. Independent findings of this lineage (beyond the prior delivery)

- The bootstrap hash-pin path is `#[cfg(windows)]`-gated
  (`codex-platform/src/lib.rs:285-324`): the "exact packaged-CLI hash check"
  claim is Windows-resolution-only; Linux resolves env/PATH. Rubric 01-A/01-B
  encode this precisely.
- Discrepancy register (rubric Appendix B): 22-vs-21 official overlay rows
  (LXR/B2R claim 22; VLM READ 7 enumerates 21); "274 px" is A-owned
  `[historical-record]` with no runtime pixel capture in-tree (only the
  275 px sidebar); 126-vs-89 ClientRequest measurement difference; PM-vs-A
  fact-ownership swaps that would be misattributions if cited wrong.
- Evidence-strength stratification across the five `complete` rows:
  Feedback, Runtime bootstrap (hash-check detail), and Keyboard shortcut
  reference ("only active bindings") rest on `[historical-record]` PM facts
  re-confirmed only at entry-surface level by B2 — rubric assigns source-read
  + GUI click-through criteria (esp. 04-C, the first Feedback click-through)
  before `complete` is defensible against C.
- Notifications and tray is the weakest-evidenced row (one
  `[runtime-observed]` toast-banner fact mixed with `[historical-record]`
  completion-banner/quiet-hours claims) — criterion 09-D guards provenance
  inflation.

## 3. Duplicate-delivery disclosure (for the Lead)

- A prior RWO-020 delivery already exists on the remote branch
  `research/rwo-020` @ `734b1e7cf34f9d51fbf24ce260260ab03c2b231a`
  ("docs(research): RWO-020 — reference-behavior review rubric (WO-REVIEW-001
  deep phase)", authored 2026-09-19 01:22:58 UTC), off the SAME base
  `d479c7b9e725e1af1353140d49226b3f9a16be9d`, containing
  `docs/research/evidence/rwo-020/rubric.md` (594 lines, 13 rows: the 8
  complete rows incl. Git process hygiene / Marketplace admin-disabled
  install / Stable-failure regression controls, plus 5 partial rows) and
  `notes.md` (104 lines).
- This session is a later re-dispatch. Its independent deliverable
  (`rubric.md`, 600 lines, the 10 mandated rows) could not fast-forward the
  occupied branch, so per the WO-P2-008 `64b28f6` precedent (duplicate
  delivery preserved and adjudicated; verified lineage merged) this lineage
  is pushed to the distinct branch **`research/rwo-020-r2`** and the prior
  delivery was NOT overwritten, reverted, or force-pushed.
- Differences at a glance: the prior rubric adds Git hygiene / Marketplace /
  Stable-failure rows and EQ-1..EQ-4 environment questions; this rubric adds
  the discrepancy register, per-row C attack notes, the Windows-gated
  hash-pin anchor, the Feedback click-through criterion, the version
  boundary map, and the B/C execution protocol. The Lead adjudicates which
  lineage (or merge of both) becomes canonical; no status cells were edited
  by either lineage.

## 4. Honesty statement

No Flauz behavior was verified during this rubric's construction (per the
RWO-020/B split). No repo scripts were executed. No credentials were
written to any file, commit message, or log (the push credential appeared
only on the git command line and was scrubbed from `.git/config`
immediately after clone). All citations were taken from the repository's
own evidence layers at base `d479c7b`.
