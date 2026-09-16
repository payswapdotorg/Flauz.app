# Codex Universal Client Boundary

**Status:** ACTIVE — locked by GUI-001
**Owner:** `payswapdotorg/Flauz.app` (client side) / `payswapdotorg/codex` (contract authority)
**Contract source of truth:** `codex-rs/app-server-protocol/src/protocol/v2/workflow.rs` in `payswapdotorg/codex`
**Client binding:** `crates/codex-protocol/src/workflow.rs` + `AppServerConnection` workflow methods in `crates/codex-platform/src/app_server.rs`

## 1. The boundary decision

All Universal Workflow (and, when the contracts land, Pack) operations ride
the **official supervised Codex app-server** over the existing stdio
JSON-RPC transport that codexRS already supervises:

```text
Flauz.app GPUI process
   |
   codex-core actions/effects (UI state only)
   |
   codex-platform supervisor  --spawn-->  codex app-server --stdio
   |                                        (user's installed codex binary)
   typed workflow/* request family          |
   |                                        v
   +-- workflow/teach/* … workflow/instance/*   Universal control plane
```

The GUI does **not**:

- import any `payswapdotorg/codex` runtime crate into the GPUI process;
- spawn a second workflow engine, agent runtime, or skills runtime;
- open live `CODEX_HOME` databases, JSONL, auth, or logs directly;
- create an alternate durable workflow or Pack authority.

The `workflow/*` family is served by the app-server only when the client
initializes with `experimentalApi: true`; that capability flag is the
negotiation for this boundary. A runtime binary without the family
responds with `method not found`, which the GUI must surface as an
actionable compatibility error (see §7).

## 2. Pinned baselines

| Baseline | Value |
| --- | --- |
| codexRS behavioral reference | Codex Desktop `26.721.3996.0` / CLI `0.146.0-alpha.3.1` (unchanged, see `docs/parity-matrix.md`) |
| Universal runtime contract | `payswapdotorg/codex` release `rust-v0.1.0` and later; the 18-method family below verified against that release |
| Minimum runtime for workflow UX | `codex` executable whose `app-server` serves `workflow/*` with `experimentalApi` |

The workflow request family (all experimental, all require
`experimentalApi: true`):

```text
workflow/teach/start          workflow/teach/instruct
workflow/teach/demonstrate    workflow/teach/reconcile
workflow/compile              workflow/review
workflow/approve              workflow/publish
workflow/fork
workflow/improve/propose      workflow/improve/validate
workflow/improve/approve      workflow/improve/publish
workflow/instance/run         workflow/instance/list
workflow/instance/get         workflow/instance/resume
workflow/instance/cancel
```

## 3. State ownership matrix

| State | Owner | GUI role |
| --- | --- | --- |
| Live Codex runtime state (auth, config, sessions, threads) | official app-server in `CODEX_HOME` | request through supervisor only |
| Universal Workflow semantic state (versions, instances, evidence, run positions) | Universal control plane behind app-server | render; never persist a second copy as authority |
| Teaching sessions and compiled candidates | app-server process memory (RWO-001 bound: in-memory doubles first) | render; treat as ephemeral (see §6) |
| UI preferences, workspace paths, download records | codexRS single-writer `state.sqlite3` | own (unchanged codexRS rule) |
| Pack identity / revisions / system state | Universal Pack control plane (contracts staged, PACK-001..004) | future: request through typed contracts only |
| Credentials | app-server / OS keychain paths owned by the runtime | never in workflow params, UI state, prompts, or evidence |

## 4. Provenance rules

- Session, candidate, instance, and version identifiers are
  **control-plane allocated opaque strings**. The client never generates,
  mutates, or infers them.
- Version ids, definition digests, dependency-lock digests, and evidence
  digests use the frozen `sha256:<64 lowercase hex>` form. The client
  treats them as opaque; it never synthesizes or shortens them.
- Every response carries the workflow identity fields that apply
  (`workflow`, `versionId`, digests, `epoch`). The GUI displays them
  verbatim so provenance is always inspectable.
- Fork and improvement lineages are engine-sealed records; the GUI renders
  them and never re-derives lineage locally.
- Commit anchors are full 40/64-hex SHAs supplied by the user or the
  runtime; the client never fabricates a SHA.

## 5. Transport, framing, and boundedness

The boundary reuses the codexRS supervisor rules unchanged:

- one long-lived multiplexed stdio connection per supervised runtime;
- `initialize` (with `experimentalApi: true`) → `initialized` handshake
  before any workflow request;
- 16 MiB per newline-delimited frame; 64 pending requests; 256 interleaved
  messages per request; bounded command/event channels; per-request
  timeouts;
- one graceful shutdown, then one bounded process-tree termination.

Server-side input bounds (workflow name, evidence label/locator/digest,
approver principal, commit SHA, semantic version, rejection reason) are
validated by the control plane; the client forwards user input verbatim
and surfaces `invalid …` errors as actionable UI feedback rather than
pre-validating a duplicated rule set.

## 6. Restart and reconnect semantics

- **Durable across restart:** published versions, durable instances,
  evidence references, and run positions (control-plane stores under
  `CODEX_HOME`). Proven by
  `workflow_durable_instances_survive_connection_restart`.
- **Ephemeral across restart:** open teaching sessions and compiled
  candidates live in the app-server process (RWO-001 bound). After a
  runtime restart the GUI must treat session/candidate ids as gone:
  reads fail with a control-plane error, and the honest recovery is to
  restart teaching, not to fake continuity.
- Reconnect after supervisor respawn: re-run the initialize handshake,
  then prefer idempotent reads (`workflow/instance/list`,
  `workflow/instance/get`) to rehydrate views. Never auto-retry
  non-idempotent operations (`teach/*`, `approve`, `publish`,
  `improve/approve`, `improve/publish`, `instance/run`,
  `instance/cancel`) after a timeout; the request may have settled.

## 7. Error taxonomy at the boundary

| Condition | Surface |
| --- | --- |
| runtime binary lacks the family | `method not found` → actionable "Codex runtime too old for workflows" UX |
| invalid input (name, SHA, version, principal, evidence digest) | `invalid …` control-plane error → field-level UI feedback |
| unknown session/candidate/instance id | `unknown teaching session …` style error → stale-view recovery |
| lifecycle gate violations (approve before validate, publish before approve) | control-plane error → state-aware UI |
| transport death / timeout | `AppServerError::TransportClosed` / `RequestTimedOut` → supervisor respawn + §6 recovery |

Response decoding is strict on enums: an unknown variant is a
compatibility error surfaced to the user, never a silent fallback.
Unknown *fields* in responses are ignored so newer runtimes can add
fields without breaking this client.

## 8. Redaction and secrets

No credential, token, or provider payload ever crosses this boundary.
Repository identity is canonical and credential-free. Evidence locators
are opaque; the GUI never dereferences them. Diagnostics follow the
codexRS redaction rules already enforced by the supervisor.

## 9. Pack boundary (forward-compatible lock)

The Pack contracts are staged in `payswapdotorg/codex`
(`PACK-001`..`PACK-004`, architecture-frozen, not yet implemented in any
served runtime). Until they land:

- the GUI implements **no** Pack state, storage, or derivation;
- Pack-aware views (PACK-UX-001) will consume typed Universal Pack
  contracts served through this same app-server boundary — never a
  GUI-local Pack runtime, optimizer, or generator;
- Pack identity/revision references obey the same provenance rules as
  workflow versions (opaque, control-plane allocated, immutable);
- when the contracts land, this file and the client bindings grow a
  `pack/*` section the same way §2 pins the workflow family.

## 10. Verification

Integration scaffolding: `crates/codex-platform/tests/workflow_boundary.rs`
(spawn a real supervised app-server under an isolated throwaway
`CODEX_HOME`; never touches `~/.codex`):

```text
CODEX_RS_TEST_CODEX_BIN=/path/to/codex \
  cargo test -p codex-platform --test workflow_boundary
```

`/path/to/codex` is any executable from
`payswapdotorg/codex` release `rust-v0.1.0` or newer (or built from
`main`). Without the variable the tests skip, keeping ordinary
`cargo test --workspace` green on machines without the runtime.

The two tests prove: (1) the full lifecycle round-trip
(teach → instruct/demonstrate → reconcile → compile → review → approve →
publish → run → list/get → improve/propose, plus the unknown-session
error path) through the typed client against the real runtime; (2)
durable instances survive a full connection restart.

## 11. Dispatch readiness

With this boundary locked:

- **GUI-002** (Desktop Parity Closure) proceeds on the codexRS surface;
  no workflow dependency.
- **GUI-003** (Universal Workflow Client Integration) builds its vertical
  slice on `AppServerConnection::workflow_*` and the typed
  `codex_protocol::Workflow*` contract types.
- **PACK-UX-001** waits on `PACK-001`..`PACK-004` in `payswapdotorg/codex`
  per the staged Pack program; nothing in Flauz.app blocks it.
