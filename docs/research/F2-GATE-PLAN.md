# F2 Gate Plan — the Wave-1 integration gate (Lead-run)

> Status: **PASSED 2026-09-22 — F2 CLOSED.** Executed by the Lead after the
> Wave-1 merges (4/4). Record:
> [evidence/f2-gate/F2-GATE-RECORD.md](evidence/f2-gate/F2-GATE-RECORD.md).
> Authority: F2-CONTRACT-KERNEL.md §10-11; F2-WORK-ORDERS.md.

## Inputs (merge order)

| # | Work order | State | Artifact |
|---|------------|-------|----------|
| 1 | ARCH-001 (flauz-world) | MERGED (PR #42; 3535b1d, lock 63e9340) | 46/46 tests, CI both platforms |
| 2 | ARCH-002 (flauz-exec) | MERGED (PR #43; 10825f6, lock fb54936) | 60/60 tests, CI both platforms |
| 3 | ORCH-001+UX-001 (flauz-context + shell) | MERGED (PR #45; d76afd764, incl. Lead gate-fix a12adac/6a0eece/f369001/f74b0a4) | 44/44 crate tests + 6 shell tests, CI both platforms |
| 4 | UX-003 (a11y carry-overs) | MERGED (PR #44; 4378892) | N5/N6/NUX tests, CI both platforms |

## Gate A — kernel §10 integration round-trip (Lead-authored harness)

Lead-authored verification infrastructure (a scratch crate outside the
workspace or a gated test target — NOT a Wave-1 worker crate): wires the
three merged crates together through the frozen formats. No external
service anywhere.

Scenario (kernel §10, verbatim order):

```
create workspace → create task → attach resource (surface change) →
record observation → claim → produce artifact →
attach fake environment (FakeLocalEnvironment + FakeRemoteEnvironment) →
attach fake agent/model (FakeCodexRuntime + FakeNonCodexRuntime) →
compile context snapshot → verify claim → evidence → grant/expire lease →
serialize all state (canonical JSON) → drop in-memory state → reload →
assert same logical task identity + state equality + evidence still
verified + expired lease not held →
switch fake model → continue → assert task identity unchanged
```

Assertions beyond the scenario steps (from the kernel rules):
- the world EventEnvelope flows through exec's SessionEventTransport
  (already smoke-proven 2026-09-21: byte-stable round-trip, 446 wire bytes)
- IDs validate against the frozen kernel vectors in all three crates
- claim≠evidence holds at the seam (a verified claim produces Evidence via
  verify_claim only)
- the context snapshot is reconstructible from references; provenance
  covers every included item; compilation does not mutate task refs
- no credential material anywhere in the serialized state

## Gate B — GUI discovery evidence (Lead lab scenes)

Binary: the post-merge incremental release build (guarded recipe:
CODEGEN_UNITS=16, LTO=false, STRIP=symbols; CARGO_TARGET_DIR per
scripts/dev-env.sh) staged as `parity-lab/binaries/codexrs-f2gate-<sha>`.

1. **Shell-discovery scene (new, Worker C's merge)**: cold start → every
   §2.1 PRODUCT-UX-JOURNEYS surface reachable via (a) primary workspace
   navigation, (b) the command palette, (c) keyboard-only; not-yet-
   implemented surfaces show honest empty states (what it will do + the
   next action), never hidden controls; UI copy uses "reusable workflow"
   language (no internal type names). Frames + VLM reads per the house
   scene pattern (d17 template).
2. **F1 regression scenes (Worker C's merge, kernel §11 mandate)**:
   palette rows (d18-visible-entries), palette/overlay close focus
   (d17-palette-close-focus), navigation (d19-renav). All green at the
   merged binary = F1 flows unchanged.
3. **UX-003 probes (Worker D's merge)**: PTY-focus probe (typed input
   lands in the PTY after Ctrl+` open — the N5 closure), swap-chord
   re-evidence in the two-chat fixture (N6), fresh-profile promo capture
   (one-Escape contract at the current version).

## Gate C — house gates on merged main

After each merge and at the gate: `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` (local, dev-env where the GUI links), plus CI
green on both platforms for the final main SHA.

## Pass criteria (all must hold)

1. Kernel §10 round-trip green (Gate A) with the harness code + output
   archived under `docs/research/evidence/f2-gate/`.
2. All Gate B scenes green at the same merged binary; F1 regressions
   byte-comparable where the scenes pin md5 sequences.
3. Gate C green at the final main SHA.
4. Every Wave-1 work-order ledger entry MERGED with empty Contract
   deviations.
5. Non-code-domain validation NOT required at F2 (that is the F6 gate's
   mandate); F2's user-facing outcome is the discoverable shell.

## Known risks / notes

- The GUI crates do not build in worker sandboxes (missing system libs) —
  all GUI evidence is Lead-side (dev-env sysroot + parity-lab).
- Disk headroom on the Lead sandbox is the build constraint — the release
  cache is warm; incremental merges are ~150M each.
- Worker session churn (capacity lottery + pod resets) is ops noise, not a
  gate risk; the registry + watchers own it.
