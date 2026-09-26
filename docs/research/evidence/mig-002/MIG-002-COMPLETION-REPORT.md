# MIG-002 Completion Report — the migration guarantee made testable

**Date:** 2026-09-26
**Worker:** dispatched worker (Wave-8b, Worker B)
**Marker:** `MIG-002 COMPLETION REPORT`

---

## 1. WO ID(s)

MIG-002 — "the migration guarantee made testable: forward-migration tests for
every schema version + the user-facing migration story doc" (Wave 8b,
docs/research/WAVE8B-PROD-WORK-ORDERS.md §MIG-002).

## 2. Base branch + SHA

Base branch + SHA: main @ 61b8da9047377a5a5b9e5519f039ad96ace8e3db

STEP ZERO verified: fresh clone of
https://github.com/payswapdotorg/Flauz.app; `git checkout
61b8da9047377a5a5b9e5519f039ad96ace8e3db`; `git rev-parse HEAD` returned
exactly `61b8da9047377a5a5b9e5519f039ad96ace8e3db` (the Wave-8b work-orders
pin; the clone's origin/main HEAD equals the pin). Branch
`feat/mig-002-guarantee` created from that exact commit.

RE-ENTRY law checked: `crates/codex-storage/tests/` did not exist at the
pinned base (the crate had only its in-crate `#[cfg(test)]` module) — the
MIG-002 artifacts did not pre-exist; this was a full authoring dispatch, not
verify-and-report.

## 3. Branch / commits

- Branch: `feat/mig-002-guarantee`, based directly on the pinned base.
- Commits: ONE clean commit (the commit carrying this report; subject
  `test(mig): MIG-002 — the migration guarantee: forward-migration tests for
  every schema version (v0..v5 fixtures, preserved rows, future-version
  refusal) + the upgrade story (Wave 8b)`). The bundle's `list-heads` records
  its exact SHA.
- Bundle: `mig-002-delivery.bundle` at the repo root, created with exactly
  `git bundle create mig-002-delivery.bundle
  61b8da9047377a5a5b9e5519f039ad96ace8e3db..feat/mig-002-guarantee`;
  `git bundle verify` reports okay. NOT pushed (the Lead harvests).

## 4. Changed files / surfaces

| File | Change | Notes |
| --- | --- | --- |
| `crates/codex-storage/tests/migrations.rs` | NEW | the 7 migration-guarantee tests |
| `crates/codex-storage/tests/fixtures/mod.rs` | NEW | the fixture builder: hand-authored schema SQL per version v1..v5, representative credential-free seed rows, the expected-current-schema constants, named hard-fail fixture guards |
| `docs/platform-support.md` | EXTEND, additive (+60/−0) | "## The state database upgrade story" — what migrates, what never does, the backup recommendation, the downgrade refusal |
| `docs/research/evidence/mig-002/MIG-002-COMPLETION-REPORT.md` | NEW | this report (the on-branch report file per the wave evidence convention — see deviations) |

NOT touched: `crates/codex-storage/src/lib.rs` (the migrator — zero changes by
law and in fact), any other crate, the web client's storage, CI workflows,
Cargo manifests (no new dependencies — `rusqlite` was already a dependency of
the crate and is available to its integration tests).

## 5. Implementation summary

The pinned migrator (`crates/codex-storage/src/lib.rs`) writes
`SCHEMA_VERSION = 5` (chain: v0/v1/v2 → 3 → 4 → 5; a strictly newer
`user_version` returns `StoreError::UnsupportedSchema(version)` before any
schema or row is touched). The work order's inputs line "(PRAGMA user_version
= 3)" is stale against the pin — the governing scope sentence "for EVERY
schema version v < current" was implemented literally, covering v0..v4 plus
the current v5 and a future v6.

The fixture builder (`tests/fixtures/mod.rs`) hand-authors the exact SQL of
each historical version (restated from the migrator's own history and
cross-checked against the in-crate legacy tests) and seeds small,
representative, credential-free rows per table: 2 UI preferences, 2 recent
workspaces (one named + pinned), 2 browser downloads (one Canceled, one
Complete), 2 related workspace folders (ordered), 2 browsing visits (one
titled). Path blobs are written through a per-platform encoding mirror of the
store's private `encode_path` (Unix raw bytes / Windows UTF-16LE) so the
fixtures hold real bytes on both legs of the CI matrix. Every fixture build
ends in a named hard-fail guard if `PRAGMA user_version` does not land
exactly at the requested version. The expected current schema — the five
tables, the three named indexes, and every column's name/type/NOT NULL/PK —
is restated as constants so the tests assert what the migrator must land on
instead of trusting its internals.

The test suite (`tests/migrations.rs`) opens each fixture through the real
`Store::open` path (the migrator runs), then asserts (a) the resulting schema
(`assert_current_schema`: user_version = 5, exact table set, exact named
index set, every column), and (b) the preserved rows read through the same
public API the app uses — plus, per version, that tables created by the
migration come up empty. The seven tests:

1. `fresh_database_lands_on_the_current_schema` — the v0 empty/fresh case.
2. `version_one_storage_migrates_forward_preserving_rows` — v1 → v5.
3. `version_two_storage_migrates_forward_preserving_rows` — v2 → v5.
4. `version_three_storage_migrates_forward_preserving_rows` — v3 → v5.
5. `version_four_storage_migrates_forward_preserving_rows` — v4 → v5.
6. `current_version_storage_opens_unchanged` — v5 no-op, rows intact.
7. `future_version_storage_is_refused_never_downgraded` — a v6 database
   (current schema, version header one step ahead) is refused with
   `StoreError::UnsupportedSchema(6)`, the exact Display text
   `storage schema version 6 is newer than this build`, and — re-checked
   through a second connection after the refusal — the file's user_version is
   still 6 and its rows are intact: the named honest refusal, never a silent
   downgrade.

Every edge of the migration graph (0→3, 1→3, 2→3, 3→4, 4→5) is exercised by
at least one full-chain test; the v3 and v4 fixtures double as the
"interrupted upgrade left the database at an intermediate committed step"
case, proving resumption.

The doc section (additive, after Runtime directories) tells the upgrade story
in exactly the tests' terms: the per-version migration table maps 1:1 onto
the seven tests, the interrupted-upgrade resumption paragraph cites the v3/v4
fixtures as its proof, "what never migrates" (the app-server-owned
`CODEX_HOME`; nothing ever migrates down) and the precise refusal semantics
(keyed on `PRAGMA user_version`, error text identical to the string the test
asserts), and the backup recommendation (the migration rewrites the file in
place; the copy is also the only way back to an older build).

No migrator changes were needed: the existing behavior passed every
assertion on the first full run (the only test-code fixes during authoring
were a tuple-comparison type mismatch and an ordering-vs-set comparison in
the schema assertion — test-side only). Per the synchronization law, a real
migration failure would have blocked this order and become a focused fix
work order; none occurred.

## 6. Tests / commands + exact results

Environment note (honest bound + executed recovery): the station had no Rust
toolchain; the exact pinned toolchain from `rust-toolchain.toml` was
installed via rustup (1.97.1 + clippy + rustfmt) before any verification.
Baseline before authoring: `cargo test --locked -p codex-storage` → 19/19
green (the existing suite, unmodified).

| Command | Exact result |
| --- | --- |
| `cargo test --locked -p codex-storage` | lib tests: `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`; integration `tests/migrations.rs`: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (the seven tests above); doc-tests: 0 |
| `cargo clippy --locked -p codex-storage --all-targets -- -D warnings` | clean — `Finished \`dev\` profile` with zero warnings/errors (the CI clippy flag set, scoped to the changed crate + its test targets) |
| `cargo fmt --all --check` | clean — zero diffs workspace-wide (rustfmt needs no native deps, so this ran fully) |
| credential-law grep over the new test files (Bearer/api[_-]?key/sk-/secret/token/password/auth.json) | zero hits (only doc comments naming the E2B law) |
| `git bundle verify mig-002-delivery.bundle` | `mig-002-delivery.bundle is okay`; contains `refs/heads/feat/mig-002-guarantee`, requires the pinned base |

NAMED ABSENCE (never fabricated): the full-workspace
`cargo clippy --workspace --all-targets` / `cargo test --workspace` gates
could not be run to completion on this station — the gpui/codex-app GUI
crates fail to build for missing native dev packages (`libpipewire-0.3`,
`libxkbcommon`, `libwayland` among the ci.yml apt list), and the station has
no root to install them (sudo requires a password; attempted and declined).
Recovery path: the Lead's merge-gate station / GitHub Actions CI
(windows-latest + ubuntu-24.04 with the exact ci.yml apt list) runs these
gates unchanged. The risk is bounded by construction: this change adds only
a test target to `codex-storage` plus documentation — no product source in
any crate changed — and every gate that compiles the changed code
(fmt workspace-wide; clippy `-D warnings` and tests scoped to
`codex-storage --all-targets`) is green above. Likewise the Windows leg of
the matrix was not executable on this Linux-only station; the tests are
written platform-correctly (the per-platform path encoding mirror, absolute
per-OS seed bases) and the windows-latest CI leg is the named recovery path.

## 7. Kernel-compliance checklist (Wave-8 addendum §1–§5)

1. **Production hardening never weakens the verified state** — zero
   product-source changes (the diff is two new test files, the additive doc
   section, and this report file); the migrator is untouched; the existing 19
   in-crate tests are unchanged and green at the merged state.
2. **Honest bounds over theater** — the future-version case asserts the
   named refusal and the untouched file; the fixture builder hard-fails with
   named messages rather than passing silently on a misauthored fixture; the
   station's full-workspace CI absence is named with its recovery path; no
   verification result is claimed that was not actually run.
3. **The credential law is absolute** — fixtures are hand-authored synthetic
   rows (grep-audited: zero credential patterns); no production data; no
   `CODEX_HOME` access (tests use per-test sandbox temp directories only).
4. **One bounded commit-branch delivery** — STEP ZERO clone/checkout/rev-parse
   against the pin; RE-ENTRY law checked; one clean commit on
   `feat/mig-002-guarantee`; the bundle exported with the exact prescribed
   command; NOT pushed; this 11-field report on the branch and as the final
   message with the `MIG-002 COMPLETION REPORT` marker.
5. **The Lead executes the verification passes** — this report gates, it
   never closes: CI (both matrix legs), the full-workspace house-rule gates,
   and the merge remain the Lead's station.

## 8. Acceptance-criteria evidence (point by point)

1. **Every version transition covered by a test with preserved-row
   assertions** — v0→v5 (fresh: schema asserted, all reads empty);
   v1→v5, v2→v5, v3→v5, v4→v5 (each: resulting schema asserted — user_version,
   table set, named index set, every column's name/type/NOT NULL/PK — and
   preserved rows asserted through the public API: preferences, recent
   workspaces with their per-version columns/defaults, browser downloads,
   workspace folders, browsing history; tables created by the migration
   asserted empty). Every migration-graph edge (0→3, 1→3, 2→3, 3→4, 4→5) is
   exercised; v5→v5 opens unchanged with rows intact. All green.
2. **The future-version refusal tested (the named error)** —
   `future_version_storage_is_refused_never_downgraded`:
   `Store::open` on the v6 fixture returns
   `StoreError::UnsupportedSchema(6)` with Display exactly
   `storage schema version 6 is newer than this build`; after the refusal the
   file still reports user_version 6 and both seeded preference rows remain —
   never a silent downgrade. Green.
3. **The story doc matches the tests exactly** — the
   `docs/platform-support.md` migration table has one row per test (v0, v1,
   v2, v3, v4, v5-current) and its refusal section quotes the identical error
   string the test asserts; the resumption paragraph's claim is backed by the
   v3/v4 fixtures' tests; nothing in the section states a behavior the suite
   does not pin.
4. **CI green** — every gate runnable at the changed code's scope is green
   (fmt workspace-wide; clippy `-D warnings` on `codex-storage --all-targets`;
   `cargo test -p codex-storage` 19+7). The full-workspace/CI matrix run is
   the Lead's station pass; the station-level absence is named in §6 with the
   recovery path, per the honest-absence rules.

## 9. Known limitations

- The future-version fixture is the current schema with the version header
  bumped to 6: what a genuinely newer build's database contains is unknowable
  from this build, and the refusal keys on the version header — which is
  exactly what the test pins.
- Fixture rows are synthetic and small (two per table): they pin
  preservation semantics, not production data shapes (the E2B law forbids
  the latter by design).
- The Windows CI leg and the full-workspace gates were not executable on
  this station (named in §6, recovery paths included).
- The suite pins the schema version numerically (`CURRENT_USER_VERSION = 5`):
  any future migration step must extend the fixtures/tests/doc — that is the
  guarantee becoming self-enforcing, not a defect.

## 10. Contract deviations

- ONE disclosed reconciliation: the on-branch report file — this
  `docs/research/evidence/mig-002/MIG-002-COMPLETION-REPORT.md` — riding the
  single delivery commit, per the wave evidence convention (REL-001's
  precedent for the same "report on the branch" clause).
- ONE named correction, disclosed: the delivery template's commit-message
  range "(v0..v3 fixtures)" was corrected to "(v0..v5 fixtures)". The work
  order's inputs line ("PRAGMA user_version = 3") and its fixture-list
  parenthetical "(v1, v2, v3)" are stale against the dispatch pin — the
  pinned migrator carries `SCHEMA_VERSION = 5` (the v4→v5 browsing-history
  step predates the Wave-8b pin). The governing scope sentence — "for EVERY
  schema version v < current" — was implemented literally (v0..v4 + the
  current v5 + the future v6 refusal); shipping the template's range verbatim
  would have misdescribed the commit's own content. The commit subject
  otherwise matches the prescribed message exactly.
- No others: no migrator changes, no schema changes, no new migrations, no
  web-client storage, no new dependencies, owned surfaces respected.

## 11. Follow-up

- Whoever adds the next migration step (a hypothetical v6): add the v5→v6
  fixture + test row, bump `CURRENT_USER_VERSION` (and the migrator's
  `SCHEMA_VERSION`), extend the doc table — the suite fails closed until
  then (`fresh_database_lands_on_the_current_schema` pins user_version = 5),
  which is the point of MIG-002.
- Lead's merge-gate station: run the CI matrix (both legs) and the
  full-workspace house-rule gates; harvest `mig-002-delivery.bundle`.
- No product defects were found by this order, so no fix work orders are
  spawned from it.
- Adjacent (not this order): OBS-001 (in flight) may add a public read-only
  schema-version accessor to `src/lib.rs`; it does not conflict with these
  tests, which read `PRAGMA user_version` directly.
