# ARCH-002 — Worker B completion report (archived)

> Archived 2026-09-21 by the Tech Lead from the live agents-tab session
> (chat e83434a3-6d5f-4da3-ad5f-492d5750904d, "ARCH-002 Exec Contracts
> Work Order"). Raw capture: replay2/scripts/worker-reports/arch-002-response-raw.txt.
> Lead gates: bundle verified on base 67e2b74; Lead-local re-run: 60/60 tests
> (47 unit + 11 conformance + 2 exec_dynamics), fmt clean, clippy -D warnings
> clean, flauz-world 46/46 unaffected on the branch; PR #43 CI (both
> platforms); merged via PR #43 — worker commit e2913a7, merge 10825f6, lock
> follow-up fb54936 = origin/main. Contract deviations: NONE.

=== ARCH-002 COMPLETION REPORT ===

Work Order ID: ARCH-002
base branch + base SHA: main @ 67e2b74c8910c4288cb60b8f757f775c35c4e7b4
BRANCH: feat/arch-002-exec-contracts | COMMITS: e2913a78db71c594646e768967bdd753feea815a (single clean commit, 36 files, +6375 lines)
changed files/surfaces:
crates/flauz-exec/** (35 files: Cargo.toml, README.md, 15 src modules — lib, ulid, ids, time, refs, capability, environment, model, runtime, agent, skill, connection, transport, store, fakes — plus tests/conformance.rs, tests/exec_dynamics.rs, and 16 fixtures under tests/fixtures/f2/**)
Root Cargo.toml: exactly one added members line, "crates/flauz-exec" (alphabetical after "crates/codex-storage")
Nothing else. Cargo.lock deliberately left uncommitted per the ownership rule (cargo regenerates the mechanical member entry on demand; same as ARCH-001). No push performed; delivery is /home/z/my-project/ARCH-002-delivery.bundle (base..branch), and the sandbox working tree at /home/z/Flauz.app is complete (source + tests + fixtures).
implementation summary (trait map, fakes, transport design):
Traits: Environment (where execution happens: environment_id/locality/capabilities/status/descriptor — pure, object-safe); ExecutionProvider (sources environments: provider_kind, supported_localities, connection_id, source_environment(&mut self, spec) → EnvironmentDescriptor, environments()); ModelProvider (sources models: provider_kind, capabilities = union of model capabilities, connection_id, models(), model()); AgentRuntime (the orchestration loop: runtime_kind, capabilities, execute(&RuntimeRequest) → RuntimeOutcome). Constitution enforced: the official Codex app-server is ONE runtime (fake kind "codex-app-server") among several (fake "flauz-direct"), never a universal registry — runtimes are neutral kind labels referenced at most once per Agent.
Entities: EnvironmentDescriptor (durable env_ record, versioned, provider_kind/connection as provenance, NO model state); Model (model_ record — intelligence only: id/name/provider_kind/capabilities/description; NO environment, runtime or connection state, no method derives models from environments); Agent (agent_ ULID, at most one runtime reference via runtime_kind: Option, optional current model as execution state, NO environment state — Agent ≠ Environment structural); Skill (SkillId namespaced key with ≥1 dot + SemanticVersion major.minor from 1.0 + required_capabilities ONLY — requirement representation, resolution is F5); Capability/CapabilityId (namespaced string keys per kernel §2, NOT ULIDs; representative catalog constants); ProviderConnection (conn_ ULID, provider kind, account label, SecretRef ONLY).
Capability advertisement: every Environment/ModelProvider/AgentRuntime advertises sorted, deduplicated CapabilityId lists; a runtime turn requiring unadvertised capabilities returns status "capability_gap" with exactly the missing keys (the J-04 diagnosis surface); the full availability intersection is F5.
Store: ExecStore (create/get/update per entity; passed version = expected; mismatch → VersionConflict; never silent overwrite) + ExecSnapshot (canonical whole-state document for serialize → drop → reload) + FakeExecStore.
Fakes (public fakes module, all deterministic, no I/O, fixed canonical IDs and fixed creation timestamp): FakeCodexRuntime + FakeNonCodexRuntime (SAME AgentRuntime contract, shared execute_turn core), FakeLocalEnvironment + FakeRemoteEnvironment (SAME Environment contract), FakeModelProvider (two fake models with different capability sets: text-only and vision), FakeExecutionProvider (sources both localities), FakeExecStore, fake_provider_connection helper.
Transport: SessionEventTransport<E>, generic over exactly E: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static; wire format one JSON object per line {"v":1,"kind":"flauz.transport.frame","envelope":<opaque>}; the frame's "v" is the transport's OWN framing version (TransportFrameVersion, rejects ≠1) — the transport never reads the envelope's version or any internals; two-phase decode reads only the frame header (version + kind) then parses the full frame strictly (deny_unknown_fields) handing the envelope to E untouched; bounded frames (house 16 MiB default, NonZeroUsize-configurable) with the house bounded-line-reader pattern (oversize consumed through its delimiter, \r trimming, blank-line tolerance on read); no exec-crate trait is implemented on any foreign type (orphan-rule safety; proven by round-tripping arbitrary serde_json::Value).
tests/commands and exact results (real runs — a Rust toolchain WAS found installable, so the repo-pinned 1.97.1 (rustup, clippy + rustfmt) was installed and everything below ran for real):
cargo test -p flauz-exec → 60 passed, 0 failed (47 unit + 11 conformance + 2 exec_dynamics; includes all 9 kernel §9 exec-side names).
cargo test -p flauz-exec -p flauz-world -p codex-protocol -p codex-storage -p codex-core → 420 passed, 0 failed (235 + 60 + 19 + 60 + 46).
cargo clippy -p flauz-exec -p flauz-world -p codex-protocol -p codex-storage -p codex-core --all-targets -- -D warnings → clean, 0 warnings.
cargo fmt --all --check → clean, workspace-wide.
cargo doc -p flauz-exec --no-deps → clean (no rustdoc warnings under #![warn(missing_docs)]).
cargo check --workspace → fails in this sandbox on the pre-existing gpui GUI stack (libspa-sys needs system libpipewire-0.3 dev libraries; no root to install). Verified identical failure at the untouched base with the members line stashed — a sandbox limitation, not a regression; all non-GUI crates check green (same caveat as the accepted ARCH-001 report).
Bundle proof: applied ARCH-002-delivery.bundle to a fresh clone at base 67e2b74, checked out the branch, re-ran the gates there → fmt clean, 60/60 tests, clippy -D warnings clean.
kernel compliance checklist (IDs/vectors, transport genericity, model/runtime separation, secret references, fixtures):
IDs/vectors: <kind><26-char Crockford Base32> with the full frozen 17-kind registry (foreign ws/task_/ctxsnap_ referenceable via EntityKind + validate without redefining their types); typed env_/model_/agent_/conn_ newtypes; frozen valid+invalid vectors pinned by kernel/ids.valid.json + ids.invalid.json (byte-identical to flauz-world's) and asserted reason-for-reason; no external ulid crate (private monotonic module, the crate's only entropy source); CapabilityId and SkillId are namespaced string keys per §2 (SkillId requires ≥1 dot), never ULID entities.
Transport genericity: exact kernel §5 bound; framing carries its own version field; envelopes opaque (event IDs/streams/seq/payloads never read); proven generic over a kernel §5-shaped test envelope AND arbitrary serde_json::Value.
Model/runtime separation: structural (distinct modules; Model has no environment/runtime/connection fields or methods; environment records have no model state; no function derives models from environments; runtimes receive models as request data only) and tested (model_has_no_provider_environment_behavior asserts exact serialized field sets both ways, construction with no environment in scope, and model-value invariance across local/remote execution).
Secret references: ProviderConnection holds provider kind + account label + opaque flausec_ SecretRef only; SecretRef::parse rejects missing prefix, empty/oversized/non-token remainders, and credential-marker shapes outright; no_credential_material_in_serialized_state scans the full snapshot serialization AND every committed fixture.
Fixtures: tests/fixtures/f2/<entity-kebab>/{minimal,typical,boundary,invalid-*}.json per §8 (16 files incl. the two kernel vector files); every fixture parses strictly and re-serializes to its exact committed bytes (fixtures_roundtrip_canonical); the invalid fixture fails the strict parse.
Serialization: "v": 1 on every top-level type, snake_case, deny_unknown_fields everywhere, kind-tagged frame/envelope tags, no floats, canonical RFC 3339 UTC Z timestamps, bounded strings/lists; 6 event types registered in crate docs/tests (environment.attached/detached, model.registered, agent.registered, connection.connected/disconnected).
acceptance-criteria evidence (map each of the 9 bullets):
Single clean commit e2913a7 on feat/arch-002-exec-contracts at the recorded base 67e2b74; the commit contains only crates/flauz-exec/** + the one members line (git show --stat verified; Cargo.lock excluded). ✓
fmt/clippy/test green for the crate with exact results above; workspace builds for every crate that can build in this sandbox — the two GUI crates fail only on pre-existing missing system libraries, verified identical at the untouched base. ✓ (sandbox caveat stated)
fake_runtimes_satisfy_agent_runtime_contract — ONE parameterized conformance test running the SAME assertions over both FakeCodexRuntime and FakeNonCodexRuntime as &dyn AgentRuntime (advertisement validity/order, completed-turn echo semantics, exact capability-gap contents, canonical outcome round-trip, invalid-request rejection). ✓
environment_contract_same_for_local_and_remote_fakes — ONE parameterized conformance test over both FakeLocalEnvironment and FakeRemoteEnvironment as &dyn Environment (canonical env_ identity, descriptor agreement, advertisement validity/order, canonical round-trip), differing only in locality. ✓
model_has_no_provider_environment_behavior — structural + tested as detailed in the kernel checklist. ✓
skill_semantics_are_provider_and_model_neutral — by construction (no provider/model/runtime/environment fields or constructor params) and by test (exact serialized field set, fixture assertions). ✓
provider_connection_holds_only_secret_references — exact serialized field set asserted; SecretRef parse-time rejection of every classic credential shape; invalid fixture rejected. ✓
transport_roundtrip_preserves_envelope_value — serialize → frame → parse → deserialize with value equality AND byte-stable envelope serialization, for a kernel §5-shaped envelope and for arbitrary JSON values, plus in-order stream round-trip and future-framing-version rejection. ✓
Contract deviations: NONE (empty — does not block closure). ✓
known limitations:
cargo check/clippy/test --workspace cannot complete in this headless sandbox: the gpui GUI crates (codex-app, codex-platform) need system libpipewire/wayland dev libraries unavailable without root; failure verified pre-existing and identical at base (the Lead verifies the full workspace on a proper machine, as with ARCH-001).
Capability RESOLUTION (the model ∩ runtime ∩ environment ∩ permissions ∩ policy intersection) and unlock paths are future (F5) work; this crate represents advertisements, requirements and runtime-side gaps only.
Fake runtimes execute single deterministic orchestration turns — no real streaming/tool loop (contract layer; proportional engineering).
Agent→runtime references are neutral kind-label strings, not typed handles (runtime identity has no §2 registry kind).
Skills are value entities outside ExecStore (namespaced-key identity, semantic versioning per kernel §2 — reasoning documented in store.rs).
FakeExecutionProvider-sourced environments get generated IDs (the sanctioned entropy source); all other fake entities use fixed canonical IDs.
contract deviations: NONE.
follow-up work:
Lead: merge per kernel §11 (members-list conflict is mechanical; keep all added lines), then run the §10 F2 integration gate feeding flauz-world's EventEnvelope through SessionEventTransport<E>.
Future waves: capability resolution + actionable unlock paths (F5, J-04); real provider/runtime adapters behind these traits (Gemini/Anthropic/E2B/Copilot/Ollama follow their own phases); the full harness loop (CONTEXT-HARNESS §3); scheduling/routing policy (explicit non-goal here); possible Lead consolidation of the duplicated private ulid module.
=== END REPORT ===
