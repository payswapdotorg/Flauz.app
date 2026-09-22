# flauz-context

Canonical context, memory and model-profile contracts for Flauz
(F2 contract wave, work order **ORCH-001**).

This crate owns the platform's context-side types exactly once:
`Context`, `MemoryItem`, `ContextSnapshot`, `ContextProvenance`,
`ModelContextProfile`, and the reset representation `ContextReset`. Every
cross-crate rule is defined by the frozen
[F2 contract kernel](../../docs/F2-CONTRACT-KERNEL.md) and the approved
[context/harness architecture](../../docs/CONTEXT-HARNESS-ARCHITECTURE.md).
It is self-contained (no dependency on `flauz-world` or `flauz-exec` —
those crates' entities are referenced through opaque canonical IDs and
validated local newtypes only).

## Semantics

- **Context is a projection, not a transcript.** A `ContextSnapshot` is a
  durable, serializable projection of what a model was told at one
  compilation: it references durable entities by their canonical
  `<kind>_<ULID>` IDs (`task`, `sess`, `model`, `ev`, `art`, `obs`, `env`,
  `mem`), each wrapped in a validating local newtype (`TaskRef`,
  `SessionRef`, `ModelRef`, `EventRef`, `ArtifactRef`, `ObservationRef`,
  `EnvironmentRef`) — never re-definitions of the owning crates' entity
  types. `ContextSnapshot::durable_references()` lists every canonical
  entity the projection depends on, so a fresh context is reconstructible
  from its references (the compilation engine itself is F6).
- **Memory tiers.** `MemoryItem` (`mem_<ULID>`) is durable rememberable
  state tiered `hot | warm | cold` (HOT: current turn / tool result / plan /
  errors / environment state; WARM: recent conversation, task summary,
  decisions, unresolved questions, discoveries, recent artifacts; COLD:
  complete history, archived tool results, workspace knowledge, documents,
  old executions, reusable procedures). HOT is eligible by default, WARM
  selectively included, COLD retrieved just in time
  (`MemoryTier::eligible_on_reset`).
- **Provenance covers every item.** Every included context item carries
  `ContextProvenance`: a source from the ten frozen kinds (user input,
  durable session event, memory item, artifact, resource observation, tool
  result, retrieved document, skill, environment state, collaboration
  event) plus an authorization class. An item without provenance cannot be
  constructed — the type requires it.
- **Authorization-aware filtering is representable.** Items whose
  provenance (or memory item) carries `AuthorizationClass::Secret` are
  authorized for retrieval by their owner but never compiled into model
  context — secrets never become context merely because they are
  technically retrievable (kernel §7). `Context::compile_from_snapshot`
  filters them; a view that carried one would fail canonical validation.
- **Model-aware compilation (representation only).**
  `ModelContextProfile` records per-model compilation behavior — context
  capacity (integer tokens), multimodal behavior, tool-schema handling —
  keyed by the `ModelRef` it profiles. Compilation never mutates task
  references, and a model switch is represented as a reset under the new
  model's profile: the task identity (its canonical `task_` ID) never
  changes and the logical task is never forked (kernel §6).
- **Snapshots and resets (architecture §12).** Snapshots are immutable
  once compiled (version always 1; a new compilation is a new snapshot).
  `ContextReset` represents the RESET operation — a fresh context
  reconstructed from durable task state, superseding the previous snapshot
  — with reasons `manual | pressure | model_changed | environment_changed |
  recovery`. RESET is distinct from COMPACTION (which preserves continuity
  while shrinking); representing compaction is later work.
- **Versions (kernel §3).** Memory items and model profiles are durable
  mutable entities: created at version 1, +1 per durable mutation, with
  optimistic concurrency (`ContextStoreError::VersionConflict` on an
  expected-version mismatch; silent overwrite is forbidden).
- **Canonical JSON (kernel §4).** `"v": 1` on every top-level contract
  type, snake_case fields, `deny_unknown_fields`, internally `kind`-tagged
  enums, no floats, RFC 3339 UTC `YYYY-MM-DDTHH:MM:SSZ` timestamps, byte
  payloads behind bounded references (inline text is bounded; bulk is
  referenced), bounded strings/lists throughout.
- **Determinism (kernel §7).** Contract code never reads wall-clock time
  or randomness; timestamps are passed in by callers. The only entropy
  source is ID generation in the private `ulid` module (no external ULID
  crate; monotonic within a process).

## Store contract and fakes

`ContextStore` is the storage contract: create/get/update for memory items
and model profiles (versioned, optimistic concurrency), `put_snapshot` /
`snapshot` for immutable snapshots, and `reset_context` — the represented
RESET operation, which records the reset, reconstructs a fresh durable
snapshot from the task's authorized, reset-eligible memory items, and
returns the compiled view. `fakes::FakeContextStore` is the deterministic
in-memory implementation: no I/O, no wall-clock reads.
`ContextStateSnapshot` is the canonical whole-state document used for
serialize → drop → reload round-trips with equality.

```rust
use flauz_context::{fakes::FakeContextStore, ContextStore, ...};

let mut store = FakeContextStore::new();
store.create_profile(profile)?;
store.create_memory_item(memory)?;
store.put_snapshot(snapshot)?;
let reconstruction = store.reset_context(
    task_ref, model_ref, ResetReason::ModelChanged, actor, at, snapshot.id,
)?;
let json = serde_json::to_string(&store.state())?;
let restored = FakeContextStore::restore(serde_json::from_str(&json)?)?;
```

## Conformance fixtures

`tests/fixtures/f2/<entity-kebab>/<case>.json` per kernel §8, including the
shared kernel vectors `kernel/ids.valid.json` and `kernel/ids.invalid.json`
(byte-identical to the other F2 crates). Every fixture is canonical JSON:
it parses strictly (unknown fields rejected) and re-serializes to exactly
the committed bytes (`fixtures_roundtrip_canonical`).

## Verification

`cargo fmt --all --check`, `cargo clippy -p flauz-context --all-targets`
and `cargo test -p flauz-context` are green. The full workspace
additionally requires the GUI crates' system libraries (Linux:
wayland/pipewire dev packages), which are not needed by this crate.
