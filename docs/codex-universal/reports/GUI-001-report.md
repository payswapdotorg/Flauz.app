# GUI-001 — Foundation + Boundary Lock

**Status:** COMPLETE
**Base:** `98717c0` (`docs: finalize Universal 0.4.0 tech lead handoff`)
**Work order:** `docs/codex-universal/WORK-ORDERS.md` § GUI-001
**Boundary authority (this delivery):** `docs/codex-universal/CLIENT-BOUNDARY.md`

## What was delivered

### 1. Typed Universal Workflow client boundary

`crates/codex-protocol/src/workflow.rs` (new module, re-exported from
`lib.rs`): the complete typed binding of the experimental `workflow/*`
request family owned by `payswapdotorg/codex`
(`app-server-protocol/src/protocol/v2/workflow.rs`):

- 18 request methods, 40 wire types (params/responses), 14 enums;
- provenance-preserving shapes: session/candidate/instance/version ids
  and digests are opaque strings; every response carries its workflow
  identity fields;
- version-compatibility policy encoded in the types: responses ignore
  unknown fields (newer runtime can add fields), request/response enums
  reject unknown variants (compatibility errors surface, never silent
  fallback);
- 6 focused unit tests (wire serialization, control-plane response
  shapes captured from the real runtime, strictness, optionality).

`crates/codex-platform/src/app_server.rs`: 18 typed wrapper methods on
`AppServerConnection` (`workflow_teach_start` …
`workflow_instance_cancel`), each behind `require_initialized()` and the
existing bounded, timeout-protected request router. No new transport, no
new engine, no new authority — the boundary rides the supervised
official app-server stdio connection codexRS already owns.

### 2. Real Universal connectivity (integration scaffolding)

`crates/codex-platform/tests/workflow_boundary.rs`: env-gated on
`CODEX_RS_TEST_CODEX_BIN`, spawning the **real** `codex app-server`
(verified against release `rust-v0.1.0`) under an isolated throwaway
`CODEX_HOME`:

- `workflow_lifecycle_round_trips_through_the_supervised_boundary`:
  teach (hybrid) → instruct+evidence → demonstrate → reconcile →
  compile → review → approve → publish → run → list/get →
  improve/propose, plus the unknown-session error path. Asserts identity
  propagation (workflow name, version pinning, instance↔version
  agreement, improvement evidence citing the recorded run).
- `workflow_durable_instances_survive_connection_restart`: full
  connection shutdown + respawn against the same `CODEX_HOME`; durable
  instances and version pinning survive; restart never moves durable
  workflow authority into GUI state.

### 3. Boundary documentation

`docs/codex-universal/CLIENT-BOUNDARY.md`: the boundary decision
(official supervised app-server transport, `experimentalApi` capability
negotiation), pinned baselines (codexRS `26.721.3996.0`; Universal
runtime `rust-v0.1.0`+ with the 18-method inventory), state-ownership
matrix, provenance rules, restart/reconnect semantics (durable vs
ephemeral, no auto-retry of non-idempotent operations), error taxonomy,
redaction rules, the forward-compatible Pack boundary lock (PACK-001..004
staged in `payswapdotorg/codex`; GUI implements no Pack state until
typed contracts land), and dispatch readiness for GUI-002 / GUI-003 /
PACK-UX-001.

`scripts/dev-env.sh`: userspace build-environment helper for
disk/root-constrained Linux sandboxes (sysroot pkg-config/libclang
remapping). Development tooling only; not part of the release contract.

## Acceptance checklist (work order)

| Requirement | Result |
| --- | --- |
| inspect the current codexRS fork and identify all retained subsystems | PASS — 5-crate workspace confirmed (`codex-app/core/protocol/storage/platform`); subsystem inventory in `docs/architecture.md` unchanged |
| pin the codexRS compatibility baseline | PASS — `26.721.3996.0` / CLI `0.146.0-alpha.3.1` (parity matrix) + Universal runtime `rust-v0.1.0` (CLIENT-BOUNDARY §2) |
| inspect current Universal Workflow and Pack contracts in `payswapdotorg/codex` | PASS — workflow family fully mapped (18 methods); Pack contracts confirmed staged/not-yet-served (`PACK-WORK-ORDERS.md`, architecture 0.3.0/0.4.0) |
| define the client/app-server boundary for Universal workflow and Pack operations | PASS — CLIENT-BOUNDARY §1/§9 |
| define transport/version/capability negotiation | PASS — CLIENT-BOUNDARY §1/§2/§5 (`experimentalApi` handshake; method-not-found → actionable UX) |
| document ownership of runtime/UI/workflow/Pack state and credentials | PASS — CLIENT-BOUNDARY §3 matrix |
| add integration test scaffolding with isolated `CODEX_HOME` | PASS — `tests/workflow_boundary.rs` (isolated throwaway homes; live `~/.codex` never touched) |
| preserve codexRS safety/boundedness rules | PASS — wrappers reuse the bounded router (16 MiB frames, 64 pending, 256 interleaved, per-request timeouts); no new channels or scans |
| Pack state requested only through typed Universal contracts | PASS by lock — CLIENT-BOUNDARY §9; no Pack state exists in the GUI |

### Work-order acceptance gates

| Gate | Result |
| --- | --- |
| no duplicate runtime/engine introduced | PASS — bindings + wrappers only; zero workflow semantics in the GUI |
| the GUI can connect to the existing Codex runtime | PASS — both integration tests against real `rust-v0.1.0` runtime |
| typed Universal Workflow and Pack client boundaries exist | PASS (workflow: implemented + tested); Pack: forward-compatible lock documented, contracts staged upstream (no runtime serves them yet) |
| Pack identity/revision and WorkflowVersion references have explicit provenance boundaries | PASS — CLIENT-BOUNDARY §4 (opaque control-plane ids, `sha256:<hex>` digests, engine-sealed lineage, no client-side synthesis) |
| Tech Lead can dispatch GUI-002, GUI-003, staged Pack work independently | PASS — CLIENT-BOUNDARY §11 |

## Verification commands and results

```text
# format
cargo fmt --all --check                                   → PASS (clean)

# lint (all changed crates, all targets, warnings denied)
cargo clippy -p codex-protocol -p codex-platform --all-targets -- -D warnings
                                                           → PASS
cargo clippy --release -p codex-core -- -D warnings        → PASS

# unit + integration tests (real supervised app-server)
CODEX_RS_TEST_CODEX_BIN=<codex rust-v0.1.0> cargo test -p codex-protocol -p codex-platform
  codex-platform (lib)          112 passed, 2 ignored (pre-existing)
  workflow_boundary (integration) 2 passed   ← real runtime round-trip + restart
  codex-protocol (lib)           60 passed   (54 pre-existing + 6 new)
cargo test -p codex-core                                  → 199 passed

# dependency policy
python3 scripts/check_dependency_policy.py                 → PASS (561 packages; no Electron/Tauri/Wry/WebView/Node)

# release compilation of changed crates
cargo build --release -p codex-protocol -p codex-platform   → PASS (4m28s)

# whole-workspace type check including the GPUI app
cargo check --release -p codex-app                          → PASS (2m57s)
```

## Known limitations

1. **Full `cargo build --release -p codex-app` (optimized link) was not
   completed in the delivery sandbox**: the gpui release codegen
   (`codegen-units = 1`, `lto = "thin"`) was OOM-killed on the 4 GB RAM
   sandbox machine. The whole workspace — including `codex-app` —
   type-checks in the release profile against these changes, the changed
   crates build in release, and the changes are purely additive public
   API; the CI release gate (Windows + Ubuntu) owns the final link.
2. **`cargo test --workspace` in one invocation** was likewise not run
   to completion in the sandbox (disk/RAM bound for the `codex-app`
   debug tree). Per-package tests cover every changed crate plus
   `codex-core` (all green); `codex-app` has no test target changes.
3. The integration tests skip silently when
   `CODEX_RS_TEST_CODEX_BIN` is unset — by design, so ordinary
   `cargo test --workspace` stays green on machines without the
   runtime; CI should set the variable with a `rust-v0.1.0`+ artifact.
4. Teaching sessions and compiled candidates are app-server process
   memory (RWO-001 upstream bound): after a runtime restart they are
   gone. This is documented as boundary semantics
   (CLIENT-BOUNDARY §6), not worked around.

## No unrelated refactors

Diff surface: `crates/codex-protocol/src/lib.rs` (+3 lines: module
declaration), `crates/codex-protocol/src/workflow.rs` (new),
`crates/codex-platform/src/app_server.rs` (+~200 lines: imports + 18
wrappers), `crates/codex-platform/tests/workflow_boundary.rs` (new),
`docs/codex-universal/CLIENT-BOUNDARY.md` (new), this report, and
`scripts/dev-env.sh` (sandbox dev helper). Nothing else touched.
