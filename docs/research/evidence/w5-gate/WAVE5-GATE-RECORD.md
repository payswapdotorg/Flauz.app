# Wave-5 Gate Record — F6 Depth: Leases/Conflicts + Human Takeover CLOSED

> **The Wave-5 gate** (per WAVE5-WORK-ORDERS.md): both work orders merged
> with per-WO gates green + CI green on both platforms, the frozen-kernel
> compliance verified (deviations NONE ×2), and the honest not-wired
> state of the live wiring recorded. **The remaining F6 orchestration
> depth is CLOSED.**

## The merged wave

2 of 2 MERGED: LEASE-001 (PR #55, merge `b346895`), TAKE-001 (PR #56,
merge `3546201`). Both dispatched from inside the replay (GLM-5.3
agents-tab sessions), both delivered under adverse platform conditions,
both reconstructed from the batch-store work logs after the sandbox
bundles proved unreachable (the files-API workspace boundary), both
Lead-gate-fixed at the integration station (the workers' sandboxes lack
the Rust toolchain — honestly declared in both reports).

- **LEASE-001** — the lease manager: queue/escalate/expiry/renewal with
  structural conflict-honesty over the frozen world lease contract,
  resource-aware scheduling data, the J-16 conflicts surface
  (flauz_conflicts.rs), 31 unit + 9 conformance tests, 14 fixtures.
- **TAKE-001** — the human fabric: takeover/approval/handoff records
  with structural attribution (a record without full attribution cannot
  be constructed), approval gates with named needs + BOTH consequences
  (human-only decisions; denial fails the node honestly with the named
  consequence), cancellation with full dependency propagation (every
  dependent named; exactly one run-level record; attributed work kept),
  the six `task.*` world-stream event payloads, the J-17 needs-you
  surface (flauz_takeover.rs), the attention additive kind seams,
  23 unit + 9 conformance tests, 21 fixtures.

## Gate results (TAKE-001 at the integration station, warm cache)

| Gate | Result |
|---|---|
| `cargo test -p flauz-takeover` | **32/32 PASS** (23 lib + 9 conformance) |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | **CLEAN** |
| `cargo fmt --all --check` | **CLEAN** |
| `cargo check --workspace` | **RC=0** (codex-app TAKE-001 seams compile) |
| CI Gate C (PR #56 @ df85b05) | ubuntu-24.04 **success** · windows-latest **success** |

The conformance suite is the acceptance evidence this wave: the takeover
round-trip law (attribution structural, same-stream, projection-preserved,
handback explicit), the approval space (approve/deny × consequences,
agent-decision refusal), the propagation matrix (8 cases: leaf/mid/root/run
× 2 graphs — every dependent named, attributed work kept, one run record),
determinism byte-equality, the no-credential scan over all fakes + all
21 fixtures, the canonical-JSON wire shapes.

## Honest deferrals (recorded, not hidden)

1. **The live human-in-the-loop wiring** — the UI's state is the module's
   fake-seam view-model this wave (the not-wired note in the surface says
   so plainly); driving set_needs_you_view / set_task_decision_affordance
   / set_attention_kind_needs from the harness's Escalated state + the
   takeover fabric through the world-store seam is the Wave-later wiring
   contract (the six event payloads are the frozen wiring surface).
2. **Lab scenes at the merged binary** (the J-16/J-17 journeys on a live
   desktop) — deferred to the client-verification stage where the Web
   client's journeys are also evidenced (one lab pass over all surfaces).
3. A pre-existing duplicated FlauzConflicts listener remnant in ui.rs
   (~line 45896, a LEASE-001 gate-fix leftover; harmless — re-registers
   the same listener) — flagged for the next ui.rs-touching wave's
   cleanup.

## Provenance archive

Worker reports + work logs + the reconstruction tooling live in the
replay repo (payswapdotorg/replay2, scripts/worker-reports/ +
harvests/); Flauz delivery bundles were unreachable (sandbox boundary),
the deterministic batch-store reconstruction (34 Write + 45 Edit/MultiEdit
replays, Task-92 lineage) produced the merged trees (residuals +6 lines
— the known log-corruption pattern, all in comments/formatting).
