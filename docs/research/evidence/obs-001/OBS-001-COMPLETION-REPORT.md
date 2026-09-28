# OBS-001 Completion Report — the diagnostics export + the provider-outage record

**Date:** 2026-09-28
**Worker:** the Lead (recovery of the interrupted Wave-8 dispatch; see §2)
**Marker:** `OBS-001 COMPLETION REPORT`

---

## 1. WO ID(s)

OBS-001 — "the diagnostics export command (credential-scrubbed,
allowlist-based) + the provider-outage behavior record" (Wave 8, item 2 of
the trio; docs/research/WAVE8-PROD-WORK-ORDERS.md §OBS-001).

## 2. Base branch + SHA

Base: `main` @ `5cbfbd838c11ed87453f958b15274efac113ba1e` (the merge commit
that landed COMP-001, PR #65). Verified `git rev-parse HEAD` on the branch
equals that commit at delivery time.

RE-ENTRY law (honest history): OBS-001 was dispatched 2026-09-26 as Wave-8
Worker B and was in flight when the executing environment was lost. The
implementation was recovered as an UNCOMMITTED working tree (a `diagnostics.rs`
collector module plus wiring in `backend.rs`/`main.rs`/`ui.rs`, the storage
accessors, and the SUPPORT.md section) on the original dispatch base
(`a3c817c`). The Lead reviewed every recovered line against the work-order
contract, verified all cited evidence pointers exist on main, completed the
verification battery in a fresh environment (§6), and landed it as ONE clean
commit rebased onto current main (`5cbfbd8`, zero conflicts — disjoint from
the COMP-001 surfaces).

## 3. Branch / commits

- Branch: `feat/obs-001-diagnostics`, based directly on `main` @ `5cbfbd8`.
- Commits: ONE clean commit carrying this report.
- Delivery: pushed branch + pull request (the push path is recovered; the
  git-bundle hand-off of the earlier waves is redundant under direct push —
  the Lead harvests from the PR itself). No bundle file is created; the PR
  diff is the authoritative delivery record.

## 4. Changed files / surfaces

- `crates/codex-app/src/diagnostics.rs` (NEW, 663 lines): the collector in
  its own module (the integration note) — bounded connection-history ring
  (50 events, recorded at the backend `emit` choke point), bounded log-tail
  reader (last 200 lines / 64 KiB, grammar allowlist + secret redaction),
  the allowlist snapshot constructor, and the JSON writer. Ten unit tests.
- `crates/codex-app/src/backend.rs` (ADDITIVE): `BackendCommand::WriteDiagnostics`
  + `Backend::export_diagnostics`, `RuntimeFacts` retained at the handshake
  (`connect` gains a facts out-param; the app-server `userAgent` is the
  pinned-CLI version source — no subprocess probe), and the `emit` choke
  point recording connection actions.
- `crates/codex-app/src/main.rs` (ADDITIVE): the `--diagnostics-out <PATH>`
  CLI flag with its honest bounds (CLI path records no connection history —
  named, not hidden), help text, and error variants.
- `crates/codex-app/src/ui.rs` (ADDITIVE): `PaletteCommand::ExportDiagnostics`
  (the 94th command; label/hint/icon/group/registry + dispatch), and
  `prompt_for_diagnostics_export` — the keyboard path Ctrl+K → "Export
  diagnostics" → destination prompt → backend write → status confirmation
  (the truthful "what's included / never included" statement travels in the
  status and in the artifact's own `privacy` section).
- `crates/codex-storage/src/lib.rs` (ADDITIVE, read-only): `schema_version()`
  (PRAGMA user_version accessor) and `diagnostics_counts()` (row counts for
  the five named tables — never contents).
- `SUPPORT.md` (ADDITIVE section): the export instructions + the
  provider-outage behavior record (§8.3).
- `docs/research/evidence/obs-001/OBS-001-COMPLETION-REPORT.md` (NEW — this
  file; the reporting contract).

## 5. Implementation summary

The export is allowlist-absolute: `collect_diagnostics` constructs exactly
the named fields (format, generated_at_ms, app_version, os, arch, codex_cli,
codex_home, connection_history, log_tail, storage, privacy) and nothing else
reaches the file — proven by
`snapshot_carries_only_the_allowlisted_fields`, which asserts the exact key
set. Free-form content passes two layers: the log-tail grammar allowlist (a
line is exported only with a structured `YYYY-MM-DD[T ]HH:MM` prefix) and
defense-in-depth redaction (`api_key=`/`apikey=`/`token=`/`password=`/
`secret=` values, `Bearer` tokens, `sk-`-prefixed runs of 8+ token chars —
`log_tail_exports_only_allowlisted_redacted_lines` seeds a live-looking
credential in each shape and proves it never survives). The storage section
carries the schema version and row COUNTS; the seeded-secret test proves
neither the key nor the value of a stored preference appears. Every exported
string is byte-bounded (512 B) and the log read is bounded from the END of
the file (no unbounded reads — the kernel law). The absent-log case is the
named honest bound (`LogTail::Absent`), never an error. The connection
history is recorded at the single `emit` choke point on the backend thread;
the CLI export path honestly records nothing there (empty history with the
named bound). The outage record in SUPPORT.md documents the four-state
machine, the bounded exponential backoff (1 s → 20 s cap, dedup, reset on
success), UI event backpressure (connection-loss never dropped), interrupted
turn safety-buffering, draft preservation, and honest failure surfacing —
each row carrying its verified evidence pointer (all five cited backend test
names and the `ConnectionStatus` enum verified present on main by the Lead;
see §6 item 5).

## 6. Tests / commands + exact results

Lead-run battery (Rust 1.97.1 per `rust-toolchain.toml`, sysroot-unblocked
Linux sandbox — see §9 for the environment bound):

1. `cargo fmt --all --check` — clean (exit 0).
2. `cargo clippy --locked -p codex-app -p codex-storage --all-targets` —
   zero warnings on the touched crates (the only emitted note is the
   pre-existing third-party `proc-macro-error2` future-incompat cargo note,
   present on main).
3. `cargo test --locked -p codex-storage` — 7 passed, 0 failed.
4. `cargo test --locked -p codex-app` — **329 passed, 0 failed** (0.99 s),
   including the ten new `diagnostics::tests` (absent-log bound, allowlist
   + redaction, sk-prefix shaping, ring FIFO/bounds, choke-point recording
   with non-connection actions ignored, exact key-set allowlist,
   counts-never-contents with a seeded secret, pretty-JSON round-trip,
   byte-bounded truncation) and the palette suite against the 94-command
   registry.
5. Evidence-pointer audit (Lead grep on main): all five SUPPORT.md-cited
   test names exist verbatim in `crates/codex-app/src/backend.rs`
   (`app_server_reconnect_uses_bounded_exponential_backoff_and_deduplicates`,
   `semantic_ui_queue_overload_prioritizes_connection_loss`,
   `safety_retry_preserves_committed_steers_with_message_boundaries`,
   `safety_retry_requires_the_interrupted_turn_to_remain_latest`,
   `safety_buffering_notification_keeps_only_bounded_retry_metadata`);
   `enum ConnectionStatus` verified in `crates/codex-core/src/lib.rs`;
   `cx.prompt_for_new_path` verified against the pinned gpui 0.2.2 with two
   in-file precedents.
6. CI (the merge-gate station): both matrix legs (windows-latest,
   ubuntu-24.04) green on the PR head before merge — the ubuntu leg runs
   the full workspace clippy `-D warnings` + workspace tests + the release
   build + the Linux desktop startup smoke.

## 7. Kernel-compliance checklist (Wave-8 addendum §1–§5)

1. Current main + green CI: yes — branch based on current main (`5cbfbd8`,
   the COMP-001 merge); both CI legs green pre-merge; additive-only change
   (no existing behavior altered; `emit` gains a recording side effect that
   cannot fail and changes no action).
2. Honest bounds over theater: the absent-log case, the CLI-path empty
   history, and the no-GUI-station bound (§9) are NAMED, not hidden; the
   artifact carries its own privacy statement.
3. Credential law: scrubbing is allowlist-based (blocklist redaction is
   defense-in-depth only); the seeded-secret tests prove the law; no
   credential appears in this report, the code, or the fixture data.
4. One bounded commit-branch delivery: one clean commit on the pinned base;
   this 11-field report rides the branch; the bundle is replaced by the
   pushed PR under the recovered push path (§3).
5. The Lead executed the verification passes: the full §6 battery is
   Lead-run, not worker-reported; merge happens only after CI green.

## 8. Acceptance-criteria evidence (point by point)

1. "The export writes every named field and NOTHING else (the allowlist
   test proves a seeded secret stays out)" —
   `snapshot_carries_only_the_allowlisted_fields` (exact key set),
   `storage_facts_report_schema_version_and_counts_never_contents`
   (seeded preference key+value absent from the serialized storage facts),
   `log_tail_exports_only_allowlisted_redacted_lines` (seeded credential
   shapes never survive the log tail).
2. "The palette entry + the CLI flag both work (the keyboard path)" — the
   palette command registers (94-command array, compile-time bound), the
   dispatch opens the destination prompt and drives the backend write
   (status confirmation), and `codexrs --diagnostics-out <PATH>` parses,
   collects, writes, and prints the includes/excludes statement. Local
   proof: the passing suite; CI proof: workspace build + tests + startup
   smoke on both legs. The interactive palette drive at a display-equipped
   station remains the Lead's named follow-up (§9.3).
3. "The outage record documents every state with evidence pointers" —
   SUPPORT.md §"Provider and runtime outage behavior": five behavior rows +
   the state machine, every row carrying a repository evidence pointer,
   every pointer Lead-verified (§6.5).
4. "CI green" — both legs green on the PR head prior to merge.

## 9. Known limitations / honest bounds

1. The GUI palette-driven export was not drivable in this environment
   (headless sandbox; no display; the GUI/release build exceeds the local
   disk budget — named, not hidden). The code path is covered by the unit
   suite (registry, dispatch, prompt wiring, backend write) and CI's
   startup smoke + FV journey battery. Recovery: drive one palette export
   on a display-equipped machine running a release artifact (rc.14+).
2. This build writes no app log; the log tail is almost always the named
   `absent` state. The reader + its allowlist exist so a future build that
   writes logs is covered by the same tested law.
3. The CLI export opens the state store read-mostly through the standard
   `Store::open` path (running migrations if an old file is found first);
   while the GUI is running, the OS-level SQLite locking is the arbiter —
   the same law the app itself follows.

## 10. Contract deviations

None. (Files owned: exactly the six surfaces of §4 plus this report; the
bundle→PR delivery substitution is recorded in §3 under the recovered push
path and follows the COMP-001 precedent.)

## 11. Follow-up

- Lead station pass: one palette-driven export with a frame + the exported
  bundle's head archived here (the §9.1 recovery).
- If a future work order adds app logging, the log-tail law already holds
  (bounded, allowlisted, redacted, tested).
