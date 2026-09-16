# GUI-007 — Parity + Product Validation (final)

**Status:** COMPLETE
**Base:** `f6c60a9` (final main; all GUI work orders merged)
**Release:** `v0.1.0-rc.13`
**Work order:** `docs/codex-universal/WORK-ORDERS.md` § GUI-007

This report validates the three dimensions separately, maps every
required evidence item to its verification, and closes the program
with an honest final status.

## Dimension 1 — Desktop parity

Release-critical Codex Desktop reference behaviors are reproduced
through public/runtime-owned interfaces per the GUI-002 parity ledger
(`docs/parity-matrix.md`):

- 27 green rows (public-contract scope complete);
- 7 bounded rows (proprietary/cloud surfaces unavailable: scheduled
  tasks, sites, visualizations, appshots, cloud environments, voice,
  pets);
- 2 platform rows resolved bounded in GUI-006 (Windows signing
  infrastructure unavailable; Linux portals/tray/global shortcuts out
  of scope, documented);
- the one closable gap found in GUI-002 (manual project ordering) was
  closed in code with a focused reducer test.

Verification: 210 core reducer tests (the native UI state-machine
suite), 60 protocol tests, CI workspace gates on the release tag,
parity differential checks recorded in the matrix.

## Dimension 2 — Universal value

### Workflow lifecycle

The complete lifecycle runs through the supervised app-server
boundary with no terminal fallback and no second engine:
create → teach (DEMONSTRATE / INSTRUCT / HYBRID) → reconcile →
compile → review → approve → publish (immutable, commit-anchored,
binding-resolution audit) → run → evidence → inspect → resume/cancel →
fork → governed improve (propose → validate → approve → publish).

Verification: the real-runtime boundary suite on the exact release
tag tree — `CODEX_RS_TEST_CODEX_BIN=<codex rust-v0.1.0> cargo test -p
codex-platform` → **112 passed + 2 integration passed** — plus the
workflow reducer tests inside the 210-test core suite (teaching
modes, single-flight pendings, control-plane gates, disconnect
clears ephemeral / durable survives, bounded bookkeeping).

### Multi-environment product surface

- Evidence kinds are classified onto the Universal environment
  classes (Browser, Computer, Terminal, API/tool/MCP, Human gate,
  Binding, Trace); unknown kinds never guess (focused test).
- The instance detail renders a numbered environment timeline with
  explicit boundary-crossing markers between consecutive known
  classes, plus a per-instance environment summary.
- Binding history: publication binding-resolution audit records are
  captured verbatim and displayed per published version.
- Takeover/recovery: instance resume/cancel with single-flight
  pendings, reason-required cancel, fail-closed error surfacing.

Verification: classifier + lifecycle reducer tests (in the 210), UI
wiring compiled and clippy-clean, boundary suite proves the typed
evidence reference channel against the real runtime.

**Honest bound (probed, recorded):** a synthetic no-credential run
against the real runtime executes (instance status `Succeeded`) but
records **zero evidence references** — live evidence requires
model-authenticated execution in real environments, which this
sandbox cannot perform (no signed-in account, no real browser/desktop
driving inside the runtime). The mixed-environment machinery is
validated at every layer available short of that: classification
(unit-pinned), the typed evidence channel (boundary-pinned), and the
timeline rendering (reducer + compile). A real cross-environment
execution recording multi-kind evidence is **not demonstrated** here.

## Dimension 3 — Pack readiness

No released runtime serves the Pack contract family (`PACK-001`…
`PACK-004` remain staged upstream in `payswapdotorg/codex`). The GUI
therefore consumes no Pack contracts and holds no Pack state — which
is the correct no-duplicate-authority posture. The product keeps two
top-level surfaces; Pack views will extend the Workflow/System
surface through the same supervised boundary when contracts land
(`PACK-UX-001` remains blocked upstream, not deferred locally).

Verification: `docs/codex-universal/RUNTIME-COMPATIBILITY.md` records
the Pack forward statement; the GUI has no Pack code paths to audit.

## Required evidence mapping

| Evidence item | Verification | Result |
| --- | --- | --- |
| Native UI tests | 210 core reducer/state tests (incl. workflow, resume/cancel, classifier, ordering); 60 protocol; 112 platform | ✅ all green on final main + CI on the tag |
| Same-state screenshots | Not possible in this sandbox (no Vulkan ICD; GPUI cannot create a surface — recorded since GUI-003). CI owns graphical archive startup smoke on real runner hardware | ⚠️ bounded |
| Fresh-machine validation | `scripts/fresh_machine_validate.sh` against the published release: download → checksum → extract → info → desktop entry → runtime probe → restart | ✅ FRESH_MACHINE_VALIDATION_OK |
| Workflow teaching/execution validation | Real-runtime boundary suite (2 integration) + workflow reducer tests | ✅ |
| Restart/recovery validation | Runtime-restart integration test (GUI-001), disconnect/rehydrate reducer test, fresh-machine restart probe | ✅ |
| Cross-environment validation | Classifier + typed evidence boundary + timeline machinery validated; live multi-kind evidence execution not demonstrable without credentials | ⚠️ machinery ✅, live execution bounded |
| Pack contract/provenance validation where implemented | None implemented upstream; forward-locked, no GUI-side authority | ✅ (vacuously, documented) |
| Security/secret scan of release evidence | Tag tree `git grep` (no token patterns); release archive docs grep + binary strings scan; worklog/outbox hygiene check | ✅ clean |

## Final gates (from the tech lead handoff)

| Gate | Status |
| --- | --- |
| Real Codex agent interaction | ✅ supervised app-server boundary against real `rust-v0.1.0` runtime (112+2 suite; fresh-machine probe) |
| Desktop-grade core UX | ✅ release-critical parity green through public/runtime-owned interfaces |
| DEMONSTRATE / INSTRUCT / HYBRID teaching | ✅ all three modes; typed records proven at the boundary |
| Immutable Workflow publication | ✅ version identity + digests + commit anchor + binding-resolution audit, boundary-pinned |
| GUI Workflow execution | ✅ run → durable instance list/detail through the boundary |
| Durable run history / evidence | ✅ durable across restart (reducer + runtime-restart integration + fresh-machine restart) |
| At least one mixed-environment workflow | ⚠️ machinery validated at all available layers; live multi-kind evidence execution not demonstrated (no credentials) — recorded, not claimed |
| Pack contract consumption, no duplicate authority | ✅ no contracts served; GUI forward-locked, zero Pack authority |
| Restart / reconnect | ✅ |
| Fresh-machine installation | ✅ Linux executed end-to-end; Windows owned by CI archive smoke |
| Release artifact verification | ✅ SHA256SUMS published and verified; CI startup smokes both platforms; checksum-verified download |

## Final status

```text
Desktop parity
  COMPLETE for the release-critical public-contract scope (27 green);
  7 proprietary rows and 2 platform rows bounded with reasons.

Universal workflow UX
  COMPLETE: full lifecycle (teach → compile → approve → publish → run
  → evidence → resume/cancel → fork → governed improve) through the
  supervised boundary; no terminal fallback; no second engine.

Universal execution UX
  COMPLETE at the product surface: environment classes, mixed-
  environment timeline with boundary markers, binding history,
  takeover/recovery. Bounded: live cross-environment execution
  evidence requires model-authenticated runtime execution not
  available in this program's sandbox (probed: synthetic runs record
  zero evidence).

Pack contract / UX readiness
  NOT STARTED, blocked upstream: no runtime serves Pack contracts;
  the GUI is forward-locked (two surfaces, zero Pack authority) and
  PACK-UX-001 is specified and waiting on PACK-001…004.

Distribution readiness
  COMPLETE: Flauz.app v0.1.0-rc.13 published (Windows ZIP, Linux
  tar.gz, SHA256SUMS), CI-gated with startup smokes, runtime
  compatibility policy documented, fresh-machine validated on Linux;
  Windows execution owned by CI archive smoke; artifacts unsigned
  (documented strategy with checksums).

Known limitations
  1. No same-state screenshots in this program (no Vulkan ICD in the
     sandbox); CI owns graphical startup smoke only.
  2. Live mixed-environment workflow evidence not demonstrated
     (requires signed-in runtime and real environments).
  3. Sign-in flow not exercised end-to-end (no account credentials in
     the sandbox); the workflow family itself is control-plane state
     and proven credential-independent.
  4. Pack-aware views not implemented (contracts unserved upstream).
  5. Artifacts unsigned; signing infrastructure unavailable to this
     program.
```

Handoff report block:

```text
CODEX_DESKTOP_PARITY: COMPLETE (release-critical scope; bounded proprietary/platform rows documented)
UNIVERSAL_WORKFLOW_GUI: COMPLETE (full lifecycle via supervised boundary; no terminal fallback; no second engine)
MULTI_ENVIRONMENT_GUI: COMPLETE at product surface; live multi-kind evidence execution bounded (no credentials) and honestly recorded
PACK_CONTRACT_UX: BLOCKED UPSTREAM (no Pack contracts served; GUI forward-locked, zero duplicate authority)
CLIENT_PORTABILITY_BOUNDARY: COMPLETE (typed client boundary; native GPUI; runtime = official CLI/app-server only)
DISTRIBUTION: COMPLETE (v0.1.0-rc.13 published, checksummed, smoke-gated, fresh-machine validated on Linux)
OVERALL: PROGRAM COMPLETE WITH RECORDED EXTERNAL BOUNDS — GUI-001…007 delivered, merged, and released; the two open items are externally gated (live environment execution credentials; upstream Pack contracts)
```

## Verification commands (final main, `f6c60a9`)

```
. scripts/dev-env.sh
cargo fmt --all --check                                            → PASS
cargo clippy -p codex-core -p codex-app -p codex-protocol \
            -p codex-platform --all-targets                        → 0 errors, 0 warnings
cargo test -p codex-core                                           → 210 passed
cargo test -p codex-protocol                                       → 60 passed
CODEX_RS_TEST_CODEX_BIN=<codex rust-v0.1.0> cargo test -p codex-platform
                                                                   → 112 + 2 passed
python3 scripts/check_dependency_policy.py                         → 561 packages passed
bash scripts/fresh_machine_validate.sh                             → FRESH_MACHINE_VALIDATION_OK
```
