# F2 Contract Kernel — frozen shared contract for the F2 contract wave

**Status:** FROZEN — 2026-09-21, by the Tech Lead, before any Wave-1 worker
branched.
**Authority:** this document sits below
[FLAUZ-SOURCE-OF-TRUTH.md](FLAUZ-SOURCE-OF-TRUTH.md) and
[CONTEXT-HARNESS-ARCHITECTURE.md](CONTEXT-HARNESS-ARCHITECTURE.md) and above
the individual work orders for every cross-crate seam in F2. Only the Tech
Lead amends it. A worker reporting a non-empty `Contract deviations` field
cannot close until the Lead resolves the deviation against this kernel.

**Purpose:** the small shared contract that lets three workers implement the
F2 canonical contracts concurrently, in self-contained crates, with zero
competing definitions of the same domain objects. Cross-crate compatibility
is guaranteed by frozen FORMATS (IDs, envelope schema, serialization rules),
not by cross-crate code dependencies.

## 1. Wave-1 ownership map

| Worker | Work order | Crate / files owned | Entities owned |
|---|---|---|---|
| A | ARCH-001 | `crates/flauz-world/**` | Workspace, Session, Task, Artifact, Resource, ResourceState, Event + EventEnvelope, Observation, Claim, Evidence, ResourceLease, Procedure, ProcedureVersion |
| B | ARCH-002 | `crates/flauz-exec/**` | Environment, ExecutionProvider, ModelProvider, Model, AgentRuntime, Agent, Skill, Capability, ProviderConnection, versioned session/event transport |
| C | ORCH-001 + UX-001 | `crates/flauz-context/**` and `crates/codex-app/src/ui/flauz_shell/**` (+ minimal `ui.rs` seams: module declaration, navigation/palette registration) | Context, MemoryItem, ContextSnapshot, ContextProvenance, ModelContextProfile, and the first platform GUI shell |

Rules:

- Each crate is **self-contained** in Wave 1: no cargo dependency between
  `flauz-world`, `flauz-exec` and `flauz-context`. Each branch must build and
  pass `cargo test` alone.
- Only the owning crate defines an entity TYPE. Other crates reference foreign
  entities exclusively through opaque canonical-ID strings (see §2),
  optionally wrapped in a local validating newtype (e.g. `TaskRef`).
  Re-defining another crate's entity is a contract deviation.
- Worker C is the only Wave-1 worker touching `crates/codex-app`; existing F1
  flows must remain unchanged (Lead re-runs the F1 regression scenes).
- Root `Cargo.toml` `members` list: each worker appends exactly their own
  crate path (alphabetical position after `crates/codex-storage`). The Lead
  resolves the mechanical members-list conflict at sequential merge — no
  design content is involved.
- Project (as an entity) is intentionally NOT part of Wave 1; the GUI shell
  surfaces the concept with honest empty states until its contract is frozen
  in a later wave.

## 2. Entity identity rules

Every durable entity has a canonical ID:

```
<kind>_<ULID>
```

- `ULID` = 26 characters, Crockford Base32 (`0123456789ABCDEFGHJKMNPQRSTVWXYZ`
  — note: no `I`, `L`, `O`, `U`), uppercase; 48-bit millisecond timestamp +
  80-bit randomness, lexicographically sortable.
- IDs are opaque: they never encode provider, surface, location, model or
  client. Equality is string equality. Case-sensitive.
- Kind prefixes (frozen registry):

| Prefix | Entity | Owning crate |
|---|---|---|
| `ws` | Workspace | flauz-world |
| `sess` | Session | flauz-world |
| `task` | Task | flauz-world |
| `art` | Artifact | flauz-world |
| `res` | Resource | flauz-world |
| `ev` | Event | flauz-world |
| `obs` | Observation | flauz-world |
| `claim` | Claim | flauz-world |
| `evd` | Evidence | flauz-world |
| `lease` | ResourceLease | flauz-world |
| `proc` | Procedure | flauz-world |
| `env` | Environment | flauz-exec |
| `model` | Model | flauz-exec |
| `agent` | Agent | flauz-exec |
| `conn` | ProviderConnection | flauz-exec |
| `ctxsnap` | ContextSnapshot | flauz-context |
| `mem` | MemoryItem | flauz-context |

- Capability and Skill identities are **namespaced string keys**, not ULIDs:
  `CapabilityId` = `[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*` (e.g.
  `browser.input`, `terminal`); `SkillId` = same grammar with at least one
  dot (e.g. `flauz.research.collect`). Skill carries a semantic version
  (§3).
- Actor references: `{"kind": "user|agent|system|provider", "id": "<canonical
  entity id or principal string>"}`.
- No external `ulid` crate dependency: each crate contains a private
  `ulid` utility module (validation + generation; monotonic within a
  process). Duplication of this ~small utility across the three crates is
  accepted for Wave-1 parallelism; the frozen vectors below make behavior
  identical. The Lead may consolidate later.

### Frozen ID test vectors (identical in all three crates)

Valid: `ws_01J8ZQ5V8K3T2B7N6X4R9DQPA0`,
`task_01J8ZQ5V8K3T2B7N6X4R9DQPB1`,
`res_01J8ZQ5V8K3T2B7N6X4R9DQPC2`,
`ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3`,
`env_01J8ZQ5V8K3T2B7N6X4R9DQPE4`,
`ctxsnap_01J8ZQ5V8K3T2B7N6X4R9DQPF5`.

Invalid (each with its reason, asserted by the vector tests):
`taskX_01J8ZQ5V8K3T2B7N6X4R9DQPA0` (bad separator),
`TASK_01J8ZQ5V8K3T2B7N6X4R9DQPA0` (uppercase kind),
`task_01j8zq5v8k3t2b7n6x4r9dqpa0` (lowercase ULID),
`task_01J8ZQ5V8K3T2B7N6X4R9DQPA` (25-char ULID),
`task_01J8ZQ5V8K3T2B7N6X4R9DQPI0` (contains `I` — not in the Crockford
alphabet), `task_01J8ZQ5V8K3T2B7N6X4R9DQPU0` (contains `U`),
`task_01J8ZQ5V8K3T2B7N6X4R9DQPL0` (contains `L`),
`task_01J8ZQ5V8K3T2B7N6X4R9DQPO0` (contains `O`),
`task` (no separator), `task_` (empty ULID), `` (empty string),
`01J8ZQ5V8K3T2B7N6X4R9DQPA0` (missing kind).

## 3. Version rules

- Every durable entity carries `version: u64`, starting at `1` on creation,
  incremented by exactly 1 per durable mutation.
- Optimistic concurrency: mutations carry `expected_version`; on mismatch the
  store returns a `VersionConflict` error. Silent overwrite is forbidden.
- Events are append-only and immutable: `event_id` unique, per-stream
  `seq: u64` strictly increasing within a stream (workspace / task / session
  streams). Events are never mutated or deleted; corrections are new events.
- `Procedure` has both a storage `version` and semantic versions on
  `ProcedureVersion` (`major.minor`, both u32, starting `1.0`); each saved
  improvement creates a new `ProcedureVersion` referencing its predecessor.

## 4. Serialization format (canonical JSON)

- serde + serde_json, as already used in the workspace.
- Every top-level serialized contract type carries `"v": <u32>` (`"v": 1` for
  F2). A schema change to a type bumps its `v`.
- Field names: snake_case, stable; renaming is a schema bump.
- Enums: internally tagged with `"kind"`.
- Canonical reads are strict: `#[serde(deny_unknown_fields)]` on contract
  types. Unknown fields are rejected, not ignored.
- Canonical state contains no floats: integers, bools, strings, enums,
  arrays, and string-keyed maps only.
- Timestamps: RFC 3339, UTC. Canonical serialized form is
  `YYYY-MM-DDTHH:MM:SSZ` (seconds precision, `Z` suffix); parsing may accept
  standard RFC 3339 variants; serialization always emits the canonical form
  (`chrono` is the allowed helper — already a workspace dependency).
- Durations: integer milliseconds. Byte payloads: base64 strings or —
  preferred — bounded references (artifact/event IDs), never inlined bulk.
- Round-trip law: for every contract type, `deserialize(serialize(x)) == x`
  where the type defines equality. Conformance fixtures prove it (§8).

## 5. Event envelope (v1) — owned by `flauz-world`

Concrete type `EventEnvelope`; canonical JSON shape:

```json
{
  "v": 1,
  "kind": "flauz.event",
  "event_id": "ev_01J8...",
  "seq": 42,
  "stream": { "kind": "workspace|task|session", "id": "ws_01J8..." },
  "event_type": "task.created",
  "ts": "2026-09-21T13:45:00Z",
  "actor": { "kind": "agent", "id": "agent_01J8..." },
  "causation_id": null,
  "correlation_id": null,
  "subject": { "entity_kind": "task", "id": "task_01J8..." },
  "payload": { }
}
```

- `event_type` vocabulary: `<entity>.<verb_past>` (e.g. `task.created`,
  `resource.observed`, `evidence.verified`, `task.model_changed`). Each work
  order registers its event types in its crate docs/tests.
- `flauz-exec`'s versioned session/event transport treats envelopes as
  opaque values: transport APIs are generic over
  `E: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static`.
  The transport has its own framing version field and never inspects
  envelope internals. No exec-crate trait is ever implemented on world-crate
  types (orphan-rule safety).

## 6. Task identity semantics (the crux)

- A Task is identified solely by its canonical `task_` ID. Task identity is
  never derived from — and never changes with — session, model, runtime,
  environment, provider, agent, or client.
- Durable task state (objective, plan, artifacts, evidence, resources,
  environment bindings, agent assignments, approvals) hangs off the Task via
  its event stream and snapshots, and is fully reconstructible from them
  without any model context.
- Model switch, context reset/compaction, environment switch, agent handoff
  and session restart are EVENTS on the task (e.g. `task.model_changed`,
  `task.context_reset`). They never change the ID and never fork the logical
  task.
- Session ≠ Task: a Session references at most one Task
  (`task_id: Option<...>`); sessions may be created and may expire freely.
- Environment choice and model choice are execution state, not task
  identity.

## 7. Cross-cutting semantic rules

- **Claim ≠ Evidence:** verification states are
  `claimed | observed | verified | contradicted | stale | unknown`. A Claim
  can never be typed, serialized, or parsed as Evidence. Evidence requires
  verifier actor attribution, a verification event reference, and a
  timestamp.
- **Conflicting observations stay distinct:** observations about the same
  subject from different surfaces/actors remain separate records with
  surface attribution; no merging, no silent deduplication.
- **Resource ≠ AccessSurface:** a Resource ID is stable when surfaces change
  (browser → API → CLI → MCP). AccessSurface descriptors carry surface kind
  (`browser|api|cli|mcp|native_desktop|file|service_integration`) and
  reference the resource ID.
- **Leases are bounded and attributable:** every `ResourceLease` has an
  absolute `expires_at` deadline, an owner actor, a scope (resource id +
  access mode `read|analyze|propose|write|commit`), and a conflict policy.
  Expiry is enforced on read: an expired lease is not held. No implicit
  renewal.
- **Credentials are references only:** `ProviderConnection` holds
  `secret_ref` strings (`flausec_...`), never secret material. No contract
  type, fixture, log line, or serialized state may contain credential
  material.
- **Determinism:** contract code never reads wall-clock time or randomness
  directly — timestamps are passed in by callers; ID generation is the only
  entropy source and is confined to the private ulid module. Fakes are fully
  deterministic.
- **Fakes:** each crate exposes a public `fakes` module (in-memory, no I/O,
  deterministic — the conformance surface). Every fake type is named
  `Fake*`. Test-local helpers are named `Test*`.

## 8. Conformance fixture naming scheme

```
crates/<crate>/tests/fixtures/f2/<entity-kebab>/<case>.json
```

- Case vocabulary: `minimal`, `typical`, `boundary`, `invalid-<what>`.
- Shared kernel vectors (identical files in all three crates):
  `crates/<crate>/tests/fixtures/f2/kernel/ids.valid.json` and
  `ids.invalid.json` — containing the frozen vectors of §2.
- Fixture files are canonical JSON (§4) — a fixture that fails
  `deny_unknown_fields` round-trip is itself a bug.

## 9. Test vocabulary (canonical test names)

`kernel_id_valid_vectors`, `kernel_id_invalid_vectors`,
`version_monotonic_and_version_conflict_rejected`,
`envelope_roundtrip_all_fields`, `envelope_unknown_field_rejected`,
`task_identity_survives_model_switch`,
`claim_cannot_be_constructed_as_evidence`,
`conflicting_observations_stay_distinct`,
`lease_expiry_enforced_on_read`, `lease_is_attributable_and_bounded`,
`resource_identity_stable_across_surface_changes`,
`procedure_roundtrip_and_version_lineage`,
`transport_roundtrip_preserves_envelope_value`,
`fake_runtimes_satisfy_agent_runtime_contract` (both fake Codex and fake
non-Codex), `environment_contract_same_for_local_and_remote_fakes`,
`model_has_no_provider_environment_behavior`,
`provider_connection_holds_only_secret_references`,
`context_snapshot_reconstructible_from_references`,
`context_provenance_covers_every_item`,
`context_compilation_does_not_mutate_task_refs`,
`no_credential_material_in_serialized_state`.

## 10. F2 integration gate (Lead-run after the Wave-1 merges)

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

No external service anywhere. The harness is Lead-authored verification
infrastructure wiring the three merged crates together through the frozen
formats above. GUI discovery is verified in parallel (Lead lab scene): cold
start → every §2.1 PRODUCT-UX-JOURNEYS surface reachable via primary nav,
palette and keyboard, with honest empty states.

## 11. Merge protocol

- Workers deliver one clean commit each on `feat/<work-order-id>-<short-name>`
  (Worker C: one branch covering both its IDs), based on the recorded
  origin/main SHA, as a git bundle. No pushes by workers.
- The Lead merges sequentially in harvest order; the root `Cargo.toml`
  members-list conflict is resolved by keeping all added lines.
- After each merge: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets`, `cargo test --workspace` (the
  house rule). After Worker C's merge: F1 regression scenes (palette rows,
  palette/overlay close focus, navigation) re-run at the merged binary.
- The F2 gate (§10) closes the phase; only the Tech Lead flips the roadmap.
