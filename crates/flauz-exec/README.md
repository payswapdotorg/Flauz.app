# flauz-exec

Provider-neutral execution, model and agent contracts for Flauz (F2
contract wave, work order **ARCH-002**).

This crate owns the platform's execution-side contracts exactly once:
`Environment`, `ExecutionProvider`, `ModelProvider`, `Model`,
`AgentRuntime`, `Agent`, `Skill`, `Capability`, `ProviderConnection`, and
the versioned session/event transport. Every cross-crate rule is defined
by the frozen [F2 contract kernel](../../docs/F2-CONTRACT-KERNEL.md); this
crate is its execution-side implementation. It is self-contained (no
dependency on `flauz-world` or `flauz-context` — those crates reference
exec entities through opaque canonical IDs only, and the world crate's
`EventEnvelope` flows through this crate's transport generic at
integration, without any exec-crate trait implemented on the foreign type).

## Semantics

- **The frozen separation** — `Client != Model != AgentRuntime != Skill !=
  Environment != Provider` and `Agent != Environment` (constitution).
  An Environment is *where execution happens* (local or remote, any
  provider); an ExecutionProvider *sources* environments; a Model is
  *intelligence*; an AgentRuntime is *the orchestration loop around a
  model and its tools*. The official Codex app-server is ONE runtime,
  never the universal Flauz registry.
- **Model/runtime separation (structural + tested)** — `Model` carries
  identity, display name, sourcing provenance (`provider_kind`) and
  offered capabilities; it has no environment, runtime or connection
  state, and no function derives models from environments. `Environment`
  records carry no model state. Model and environment choices are
  execution state: they ride on `RuntimeRequest`s, never on identities.
  `tests/conformance.rs::model_has_no_provider_environment_behavior`
  asserts the serialized field sets of both types and that switching
  environments never mutates model values.
- **Agent identity** — `agent_` ULIDs are distinct from `env_` and
  `model_` ULIDs; an `Agent` references **at most one** runtime (by
  neutral kind label) and may be bound to a current model as execution
  state. Model switches bump the entity version and never change the ID.
- **Capability advertisement (kernel §2/§7)** — every Environment,
  ModelProvider and AgentRuntime advertises its offered capabilities as
  sorted, deduplicated [`CapabilityId`] namespaced string keys
  (`terminal`, `browser.input`) — capability identity is NOT a ULID.
  Availability is the intersection
  `model ∩ runtime ∩ environment ∩ permissions ∩ policy`; computing it is
  future (F5) work. This crate represents advertisements, requirements and
  gaps: a runtime turn whose required capabilities are not advertised
  returns a `RuntimeOutcome` with status `capability_gap` carrying exactly
  the missing keys — the actionable-diagnosis surface.
- **Skills** — `SkillId` is a namespaced key with at least one dot (e.g.
  `flauz.research.collect`) plus a `major.minor` semantic version from
  `1.0`. Skills express **required capabilities** only (requirement
  representation — resolution is F5) and stay provider/model/runtime/
  environment-neutral by construction and by test.
- **ProviderConnection** — `conn_` ULID, provider kind, account label and
  an **opaque `flausec_...` secret reference only** (kernel §7). No
  contract type, fixture, log line or serialized state in this crate
  contains credential material; `SecretRef::parse` rejects references
  that look like raw credentials outright.
- **Canonical IDs** — `<kind>_<26-char Crockford Base32 ULID>` with the
  frozen kind-prefix registry; the private `ulid` module (no external
  crate) is the crate's only entropy source, monotonic within a process.
  The frozen kernel vectors are pinned by
  `tests/fixtures/f2/kernel/ids.valid.json` / `ids.invalid.json`.
- **Versions** — durable ULID entities (`EnvironmentDescriptor`, `Model`,
  `Agent`, `ProviderConnection`) start at `version: 1`, +1 per durable
  mutation, with optimistic concurrency: updates pass the expected
  version and a mismatch is an `ExecStoreError::VersionConflict`. Skills
  are namespaced-key entities versioned semantically instead (kernel §2).
- **Canonical JSON** — `"v": 1` on every top-level contract type,
  snake_case fields, `deny_unknown_fields` everywhere, no floats, RFC 3339
  UTC timestamps in `YYYY-MM-DDTHH:MM:SSZ` form, bounded strings/lists
  throughout.
- **Determinism** — contract code never reads the wall clock or
  randomness; timestamps are caller-supplied and fake entities carry fixed
  canonical IDs and a fixed creation timestamp.

## Traits and fakes

| Contract | Type | In-memory fake |
|---|---|---|
| Execution environment (local or remote, any provider) | `Environment` | `FakeLocalEnvironment`, `FakeRemoteEnvironment` — the SAME contract for both localities |
| Source of environments | `ExecutionProvider` | `FakeExecutionProvider` (sources both localities) |
| Source of models | `ModelProvider` | `FakeModelProvider` (two fake models, different capability sets) |
| Orchestration loop around a model + tools | `AgentRuntime` | `FakeCodexRuntime` (the official app-server as ONE runtime), `FakeNonCodexRuntime` (direct-model runtime) — the SAME contract for both |
| Durable execution state | `ExecStore` / `ExecSnapshot` | `FakeExecStore` (serialize → drop → reload round-trips) |

All fakes are public, in-memory, deterministic and perform no I/O.

## Versioned session/event transport

`SessionEventTransport<E>` carries session event streams as bounded,
newline-delimited JSON frames:

```json
{"v":1,"kind":"flauz.transport.frame","envelope":{...opaque...}}
```

- Generic over exactly the kernel §5 bound:
  `E: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static`.
- Envelopes are **opaque values**: the transport never inspects envelope
  internals (event IDs, streams, sequences, payloads).
- The frame carries **its own framing version** (`"v"`, currently 1,
  `TransportFrameVersion`) and its own kind tag; decoding reads exactly
  those two fields (rejecting unsupported framing versions) before
  handing the envelope to `E` untouched.
- Frames are bounded (`DEFAULT_MAX_TRANSPORT_FRAME_BYTES`, house default
  16 MiB, configurable via `NonZeroUsize`); reads and writes are bounded
  with no unbounded allocations.
- Round-trip law: `decode(encode(e)) == e`, with byte-stable envelope
  serialization (proven for a kernel §5-shaped envelope type and for
  arbitrary `serde_json::Value`s).

## Registered event types (kernel §5)

| Constant | `event_type` |
|---|---|
| `event_types::ENVIRONMENT_ATTACHED` | `environment.attached` |
| `event_types::ENVIRONMENT_DETACHED` | `environment.detached` |
| `event_types::MODEL_REGISTERED` | `model.registered` |
| `event_types::AGENT_REGISTERED` | `agent.registered` |
| `event_types::CONNECTION_CONNECTED` | `connection.connected` |
| `event_types::CONNECTION_DISCONNECTED` | `connection.disconnected` |

## Tests

`tests/conformance.rs` carries the kernel §9 exec-side vocabulary —
`kernel_id_valid_vectors`, `kernel_id_invalid_vectors`,
`transport_roundtrip_preserves_envelope_value`,
`fake_runtimes_satisfy_agent_runtime_contract` (both fake runtimes through
one parameterized conformance),
`environment_contract_same_for_local_and_remote_fakes` (both fake
environments through one parameterized conformance),
`model_has_no_provider_environment_behavior`,
`provider_connection_holds_only_secret_references`,
`version_monotonic_and_version_conflict_rejected`,
`no_credential_material_in_serialized_state` — plus canonical fixture
round-trips and skill neutrality. `tests/exec_dynamics.rs` covers
snapshot round-trips and the gate flow. Conformance fixtures live under
`tests/fixtures/f2/` per kernel §8.
