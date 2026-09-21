# flauz-world

Canonical world-state, resource and evidence contracts for Flauz
(F2 contract wave, work order **ARCH-001**).

This crate owns the platform's durable world-state types exactly once:
`Workspace`, `Session`, `Task`, `Artifact`, `Resource`, `ResourceState`,
`Event` + `EventEnvelope`, `Observation`, `Claim`, `Evidence`,
`ResourceLease`, `Procedure` and `ProcedureVersion`. Every cross-crate rule
is defined by the frozen
[F2 contract kernel](../../docs/F2-CONTRACT-KERNEL.md); this crate is its
world-state implementation. It is self-contained (no dependency on
`flauz-exec` or `flauz-context` — those crates reference world entities
through opaque canonical IDs only).

## Semantics

- **Canonical IDs** — `<kind>_<26-char Crockford Base32 ULID>` with the
  frozen kind-prefix registry (`ws`, `sess`, `task`, `art`, `res`, `ev`,
  `obs`, `claim`, `evd`, `lease`, `proc`, plus the foreign `env`, `model`,
  `agent`, `conn`, `ctxsnap`, `mem` referenced through `EntityKind` +
  `EntityRef`). The frozen kernel vectors are pinned by
  `tests/fixtures/f2/kernel/ids.valid.json` / `ids.invalid.json`. ID
  generation is the crate's only entropy source and lives in a private
  `ulid` module (monotonic within a process).
- **Versions** — entities start at `version: 1`, +1 per durable mutation,
  with optimistic concurrency: updates pass the expected version and a
  mismatch is a `WorldStoreError::VersionConflict`. Events are append-only
  with strictly increasing per-stream `seq` (workspace / task / session
  streams).
- **Canonical JSON** — `"v": 1` on every top-level contract type, snake_case
  fields, `deny_unknown_fields` everywhere, internally tagged enums, no
  floats (and no null payload values — `CanonicalValue` rejects them at
  parse), RFC 3339 UTC timestamps in `YYYY-MM-DDTHH:MM:SSZ` form, byte
  payloads behind bounded references (`ArtifactContent::Reference`), and
  bounded strings/lists throughout. Durations do not occur in v1; when
  introduced they must be integer milliseconds.
- **Event envelope v1** — `EventEnvelope` is frozen field-for-field by
  kernel §5 (`v`, `kind: "flauz.event"`, `event_id`, `seq`, `stream`,
  `event_type`, `ts`, `actor`, `causation_id`, `correlation_id`, `subject`,
  `payload`).
- **Task identity** — a Task is identified solely by its `task_` ID. Model
  switch, context reset, environment switch, agent handoff and session
  restart are events on the task stream; they never change the ID and never
  fork the task. A Session references at most one task.
- **Claim ≠ Evidence** — verification states are `claimed | observed |
  verified | contradicted | stale | unknown`. `Evidence` requires a verifier
  actor, a verification event reference and a timestamp; a `Claim` can never
  be typed, serialized or parsed as `Evidence`. In a store, evidence enters
  only through `verify_claim`, which atomically produces the evidence
  record, its `evidence.verified` event and the claim update.
- **Conflicting observations stay distinct** — no merging, no silent
  dedup; surface attribution is preserved.
- **Resource ≠ AccessSurface** — resource IDs are stable across
  `browser | api | cli | mcp | native_desktop | file | service_integration`
  surface changes; surface changes are durable mutations of the same
  entity.
- **Leases** — every `ResourceLease` carries an absolute `expires_at`,
  owner actor, resource scope + access mode (`read | analyze | propose |
  write | commit`) and a conflict policy (`reject | queue | escalate`).
  Expiry is enforced on read (`lease(id, now)` returns `None` for expired or
  released leases); there is no implicit renewal. Conflict *enforcement*
  belongs to the execution layers; the policy is contract data.
- **Procedures** — durable, reusable, versioned work. The first
  `ProcedureVersion` is `1.0`; each improvement adds a new version whose
  `predecessor` references an existing one (strictly increasing, no
  orphans). Capabilities are referenced by namespaced keys
  (`browser.input`), never redefining flauz-exec's `Capability` entity.

## Registered event types

See the crate-level documentation (`cargo doc -p flauz-world --open`), which
registers this crate's `event_type` vocabulary:
`workspace.created`, `session.created`, `task.created`,
`task.model_changed`, `task.context_reset`, `task.environment_changed`,
`task.agent_handoff`, `task.session_restarted`, `artifact.produced`,
`resource.created`, `resource.surfaces_changed`, `resource.observed`,
`claim.made`, `evidence.verified`, `lease.granted`, `lease.released`,
`procedure.created`, `procedure.version_added`.

## Store contract and fakes

`WorldStore` is the storage contract (create/get/update per entity,
`append_event` with store-assigned identity and `seq`, bounded `events`
reads, `record_*` + `verify_claim`, lease grant/read-with-`now`/release,
`snapshot`/`restore`). `fakes::FakeWorldStore` is the deterministic
in-memory implementation: no I/O, no wall-clock reads — every timestamp is
caller-supplied. `WorldSnapshot` is the canonical whole-world document used
for serialize → drop → reload round-trips with equality.

```rust
use flauz_world::{fakes::FakeWorldStore, Event, WorldStore};

let mut store = FakeWorldStore::new();
let task = store.create_task(/* ... */)?;
let envelope = store.append_event(
    StreamRef::task(&task.id),
    Event::new("task.model_changed", ts, actor, subject, payload)?,
)?;
let json = serde_json::to_string(&store.snapshot())?;
let reloaded = FakeWorldStore::restore(serde_json::from_str(&json)?)?;
```

## Conformance fixtures

`tests/fixtures/f2/<entity-kebab>/<case>.json` per kernel §8, including the
shared kernel vectors `kernel/ids.valid.json` and `kernel/ids.invalid.json`.
Every fixture is canonical JSON: it parses strictly (unknown fields
rejected) and re-serializes to exactly the committed bytes
(`fixtures_roundtrip_canonical`).

## Determinism

Contract code never reads wall-clock time or randomness; timestamps are
passed in by callers. The only entropy source is ID generation in the
private `ulid` module. The fakes are fully deterministic apart from
generated IDs.

## Verification

`cargo fmt --all --check`, `cargo clippy -p flauz-world --all-targets` and
`cargo test -p flauz-world` are green. The full workspace additionally
requires the GUI crates' system libraries (Linux: wayland/pipewire dev
packages), which are not needed by this crate.
