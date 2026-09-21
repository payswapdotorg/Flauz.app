# ARCH-001 — Worker A completion report (archived)

> Archived 2026-09-21 by the Tech Lead from the live agents-tab session
> (chat 73cc6cc6-88b9-48fd-9fb5-245b150708e9, "Flauz-World Contracts
> Development"). Raw capture:
> replay2/scripts/worker-reports/arch-001-response-20260921-182421.txt.
> Lead gates: bundle verified on base f66965e; PR #42 CI green
> (windows-latest + ubuntu-24.04); Lead-local re-run at merge 3535b1d:
> 46/46 tests, fmt clean, clippy -D warnings clean; merged via PR #42,
> origin/main 63e9340 → ledger c68fded. Contract deviations: NONE.

=== ARCH-001 COMPLETION REPORT ===

Work Order ID: ARCH-001
base branch + base SHA: main @ f66965e1bba91eda168e4f94337f480ab86e28da
BRANCH: feat/arch-001-world-contracts | COMMITS: b22f8b21af1498ae8c1f6d95e1135667f968a6f9 (single clean commit, 40 files, +6932 lines)
changed files/surfaces:
crates/flauz-world/** (39 files: Cargo.toml, README.md, 16 src modules, 2 integration test files, 21 fixtures under tests/fixtures/f2/**)
Root Cargo.toml: exactly one added members line, "crates/flauz-world", (alphabetical after crates/codex-storage)
Nothing else. Cargo.lock was left untouched per the ownership rule (cargo regenerates the mechanical member entry on demand).
implementation summary (entity map, store traits, fakes):
Entities: Workspace, Session (≤1 task ref, may expire freely), Task (identity = task_ ID only), Artifact (content = bounded inline text or bounded reference, tagged enum), Resource + AccessSurface (kind + resource back-reference) + ResourceState (bounded projection record: permissions, last-verified observation, authoritative surface, lease/mutation/artifact references, capture attribution), Event (semantic content) + EventEnvelope (kernel §5 shape, field-for-field, "kind": "flauz.event"), Observation/Claim (statement + verification status + evidence links, surface attribution) and Evidence (verifier actor + verification event ref + timestamp, required), ResourceLease (absolute expires_at, owner, resource scope + access mode, conflict policy, released_at; is_held_at(now) = not released && now < expires_at), Procedure + ProcedureVersion (major.minor from 1.0, embedded versions, predecessor lineage enforced: exists, strictly increasing, unique, no orphans) + CapabilityKey (frozen namespaced-key grammar, references exec's Capability without redefining it).
Foundations: private monotonic ulid module (only entropy source, no external crate); typed canonical-ID newtypes (11 world kinds) + EntityKind covering the full frozen 17-kind registry so foreign IDs (env_, ctxsnap_, …) are referenceable and validatable; ActorRef/EntityRef/StreamRef (exact kernel JSON shapes); Timestamp (canonical YYYY-MM-DDTHH:MM:SSZ, parses RFC 3339 variants and truncates); CanonicalValue/Payload (floats and nulls unrepresentable-and-rejected, bounded, manual visitor); ContractVersion ("v": 1, rejects other versions at parse).
Store: WorldStore trait (create/get/update per entity with optimistic concurrency — passed version = expected, mismatch → VersionConflict; append_event assigns unique ev_ id + strictly increasing per-stream seq; bounded events(stream, after_seq, limit); record_observation/record_claim; verify_claim — the only path evidence enters a store, atomically appending evidence.verified, creating the Evidence and flipping the claim to verified; grant_lease/lease(id, now) (expiry enforced on read)/lease_record/active_leases/release_lease; procedure create/update; snapshot/restore) + WorldSnapshot (canonical whole-world document, v: 1, deterministic BTreeMap ordering, flat (stream, seq)-sorted events, validated on restore) + Verification result bundle.
Fakes: public fakes::FakeWorldStore — in-memory BTreeMap-backed, no I/O, no wall-clock reads (every timestamp caller-supplied), deterministic; Fake* naming throughout; all event-type vocabulary registered in crate docs (event_types consts) including task.model_changed, task.context_reset, task.environment_changed, task.agent_handoff, task.session_restarted.
tests/commands and exact results (real runs, toolchain = repo-pinned 1.97.1, installed in sandbox):
cargo test -p flauz-world → 46 passed, 0 failed (31 unit + 6 conformance + 9 world_dynamics; includes all 13 kernel §9 world-side names).
cargo clippy -p flauz-world --all-targets → clean, 0 warnings; also clean on -p codex-protocol -p codex-storage -p codex-core --all-targets.
cargo fmt --all --check → clean, workspace-wide.
cargo doc -p flauz-world --no-deps → clean (no rustdoc warnings under #![warn(missing_docs)]).
cargo check --workspace → fails in this sandbox on the pre-existing gpui GUI stack (libspa-sys/libpipewire-0.3 and wayland-sys system libraries missing; no root to install). Verified identical failure at the untouched base (members line stashed), so it is a sandbox limitation, not a regression; all non-GUI crates check green.
Bundle proof: applied ARCH-001-delivery.bundle to a fresh clone at base, checked out the branch, re-ran the gates there → 46/46 tests, clippy clean, fmt clean.
kernel compliance checklist (IDs/vectors, versions, serialization, envelope, task identity, claim!=evidence, leases, fixtures):
IDs/vectors: <kind>_<26-char Crockford Base32> with the frozen registry; frozen valid+invalid vectors pinned by kernel/ids.valid.json/ids.invalid.json and asserted reason-for-reason by kernel_id_valid_vectors/kernel_id_invalid_vectors; no external ulid crate (private monotonic module).
Versions: entities start at 1, store increments exactly once per stored mutation, VersionConflict on mismatch (never silent overwrite), events append-only with unique event_id + strictly increasing per-stream seq; procedure storage version separate from semantic major.minor (first = 1.0, predecessor lineage).
Serialization: "v": 1 on every top-level type, snake_case, deny_unknown_fields everywhere, internally tagged enums, no floats (and no null payload values — enforced by CanonicalValue), canonical RFC 3339 UTC timestamps, durations absent in v1 (rule vacuously satisfied, documented), byte payloads behind bounded references; round-trip law proven per-type by fixtures and by the snapshot reload test.
Envelope: EventEnvelope matches kernel §5 exactly (fixture event-envelope/typical.json is that shape; envelope_roundtrip_all_fields asserts the exact 12-field set and canonical forms; unknown fields rejected).
Task identity: task_identity_survives_model_switch — ID, entity equality, single stream, seq continuity across serialize→drop→reload and continued appends; model/context/environment/handoff/session-restart are registered task events, never identity changes or forks.
Claim ≠ Evidence: distinct types with non-overlapping required fields (Evidence requires verifier + verification event + timestamp), parse fails both directions (including a claim marked verified), and store-level: evidence only via atomic verify_claim.
Leases: absolute deadline, owner actor, resource scope + access mode (read|analyze|propose|write|commit), conflict policy (reject|queue|escalate) carried as contract data (enforcement is an execution-layer concern, documented); expiry enforced on read (lease(id, now) → None at/after deadline; active-lease listing filters; raw record kept for provenance); no implicit renewal; granted_at < expires_at enforced.
Fixtures: tests/fixtures/f2/<entity-kebab>/{minimal,typical,boundary,invalid-*}.json per §8, kernel vectors included; every fixture must parse strictly and re-serialize to its exact committed bytes (fixtures_roundtrip_canonical).
acceptance-criteria evidence (map each of the 10 bullets):
Single clean commit b22f8b2 on feat/arch-001-world-contracts based on the recorded base; the commit contains only crates/flauz-world/** + the one members line. ✓
fmt/clippy/test green for the crate (exact results above); workspace builds for every crate that can build in this sandbox — the two GUI crates fail only on pre-existing missing system libraries, verified identical at base. ✓ (with the sandbox caveat stated)
FakeWorldStore passes everything with no external services — including the bundle-applied fresh-clone run. ✓
state_survives_serialize_drop_reload_roundtrip: full world (all 13 entity kinds) → snapshot → JSON → drop store → parse → restore → snapshot equality, plus task-identity/evidence/expired-lease assertions on the reloaded state. ✓
task_identity_survives_model_switch: recorded task.model_changed event; ID/entity equal, no fork, stream continues after reload. ✓
resource_identity_stable_across_surface_changes: browser → api+cli+mcp at version 2, ID unchanged, surfaces reference the same resource, survives reload. ✓
claim_cannot_be_constructed_as_evidence: type level (Evidence construction requires the three anchors; no conversion exists), serialization + parse levels (both directions fail, even a "verified" claim). ✓
conflicting_observations_stay_distinct: same subject, different surfaces/actors/statements remain separate records with surface attribution, across reload. ✓
lease_is_attributable_and_bounded + lease_expiry_enforced_on_read: owner/scope/mode/policy present, inverted/unbounded deadlines rejected, deadline-inclusive expiry on read, released leases not held, attribution survives reload. ✓
Contract deviations: none (empty — does not block closure). ✓
known limitations:
cargo check/clippy/test --workspace cannot complete in this headless sandbox: the gpui GUI crates (codex-app, codex-platform) need system libpipewire/wayland dev libraries unavailable without root; failure verified pre-existing and identical at base (Lead verifies the full workspace on a proper machine).
Lease conflict enforcement (the §7 read/analyze/propose/write/commit coordination matrix) is deliberately not implemented in the fake — the kernel requires the policy as contract data; enforcement belongs to the execution/orchestration layers.
ResourceState is an unversioned projection record (no canonical ID kind exists for it in the frozen registry); it is replaced wholesale via put_resource_state.
CanonicalValue integers are i64 (u64 above i64::MAX rejected); payload string/array/map bounds are enforced at construction and parse.
ID generation (the sanctioned entropy source) means two runs of the same store scenario differ only in generated IDs; all tests assert structure, not generated values.
No listing/pagination APIs on WorldStore (YAGNI for Wave 1; additive methods can follow when a consumer needs them).
contract deviations: NONE.
follow-up work:
Lead: merge per kernel §11 (members-list conflict is mechanical; keep all added lines), then run the §10 F2 integration gate wiring flauz-exec/flauz-context through the frozen envelope/ID formats.
Future waves: lease conflict enforcement + handoff policy (deferred by kernel bullet scope), workspace listing/pagination when the GUI shell needs it, snapshot persistence engine (explicitly out of Wave-1 scope), Project entity (intentionally absent until its contract is frozen).
=== END REPORT ===
