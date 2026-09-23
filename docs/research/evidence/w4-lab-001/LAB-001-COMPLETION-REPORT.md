# LAB-001 Completion Report — Provider-Neutral Lab Fabric (Wave 4 / F8)

> **Lead gate record.** Worker B (GLM-5.3 + Full-Stack, agents-tab session
> `f29719f0`). Base: main @ 83796f5 (pinned). Branch:
> `feat/lab-001-fabric`, single clean commit `a6d37f501ab9` (parent = the
> pinned base exactly, `git bundle verify` clean). Delivered via git bundle
> (83,340 bytes, harvested from pod ws-8e91cacb); NOT pushed by the worker —
> the Lead applies, verifies, pushes, merges.
>
> **Lead gates.** GATE 0 PASS (single commit, parent = base exactly).
> GATE 1 PASS (35 files, +14,239 insertions, all owned:
> `crates/flauz-lab/**` + the one workspace members line).
> GATE 2 PASS: `cargo fmt --check -p flauz-lab` clean;
> `cargo clippy -p flauz-lab --all-targets` clean; `cargo test -p flauz-lab`
> **60/60** (38 lib + 22 integration/conformance). Lead lockfile gate-fix
> `b8b8223` (the 8-line flauz-lab entry, serde-family only — the
> CAP-001/ORCH-004 precedent). CI at `feat/lab-001-fabric`: ubuntu-24.04 +
> windows-latest both SUCCESS (run 35848069053). **Merged PR #52.**
>
> **Contract deviations: NONE** (the worker's report field, verified by the
> Lead against the frozen Wave-4 kernel addendum §1-§7).
>
> The verbatim worker report follows.

=== LAB-001 COMPLETION REPORT ===
- Work Order ID: LAB-001
- base branch + base SHA: main @ 83796f59805948a503760d03c5d06d8d872fd758
- BRANCH: feat/lab-001-fabric | COMMITS: a6d37f501ab90011514b7ec1dff7b6df87673e29 (single clean commit, parent = the pinned base exactly; `git bundle verify` clean; NOT pushed)
- changed files/surfaces:
  - `crates/flauz-lab/**` — NEW self-contained crate (35 files total incl. the one-line workspace members addition; 14,239 insertions): `Cargo.toml`, `README.md`, `src/{lib,refs,journey,evidence,findings,comparator,adapter,fakes}.rs`, `tests/{adapter,comparator,conformance}.rs`, `tests/fixtures/w4/**` (21 fixtures: journey-spec, evidence-record, findings-registry, comparator-report, provider-families; typical/minimal/invalid per family)
  - root `Cargo.toml` — the ONE members line (`"crates/flauz-lab",` alphabetically between flauz-exec and flauz-orch; the Lead resolves the expected three-way one-liner with PROV-001/COL-001 at merge)
  - NOTHING else. Cargo.lock intentionally not committed (see known limitations — the CAP-001/ORCH-004 precedent)
- implementation summary (journey/evidence/findings/comparator/adapters):
  - **journey.rs — JourneySpec (PART 1):** the declarative journey — steps as UI actions as data (key chord, click at a NAMED anchor, type text), probes (frame capture at a named moment, state assertions), the surfaces exercised, and the J-ids it maps to as data. Deterministic ordering (steps in execution order with unique ids; probes in canonical id order); canonical JSON. StateAssertion carries the F1 patterns structurally: `FocusOn` (the 017/N1/N5 focus family), `FocusConfined` (the d19 trap ladder), `TextLanded` (the typed-probe family), `AnchorState`. `required_capabilities` derives the journey's lab-surface needs (keyboard/mouse/screen) — the input of the prepare-time named-gap check.
  - **evidence.rs — the normalized EvidenceRecord (PART 1):** versioned (`"v":1`); per-step action traces (applied/no_visible_change/rejected); frame REFERENCES (run-scoped pointer + provider-local FNV-1a digest + the NORMALIZED anchor observations — never pixels inline); assertion results (verdict + structured observed state + known-findings links by id); VLM-read SLOTS (the bounded adjudication prompt + the archived read-result reference — NO VLM calls inside the crate, adjudication stays operator-side); environment/provider metadata as topology data (kind, family, locality, lab surfaces — no credentials ever). Provider comparability is structural: the comparator-facing fields are the normalized content; digests, frame refs, read refs and descriptors are provider-local by design.
  - **findings.rs — the named-findings registry (PART 2):** the F1 a11y lessons as DATA (id, surface, description, regression-guard status): `pty-focus-transfer` (N5, unguarded), `bracket-swap-chords` (N6, unguarded), `modal-focus-traps` (d19, guarded), `first-run-keyboard-swallowing` (the rc.14 promo modal, unguarded), `shifted-symbol-chord-companions` (the N6 family + the d25 Gate-B listener lesson, guarded). Records reference findings by id, never by prose; `observed_findings()` unions the links.
  - **comparator.rs — the comparator (PART 2):** `compare(journey, reference, candidate)` — the diff through the NORMALIZED schema: per journey step matching/divergent/missing evidence; per divergence a NAMED canonical kind (`frame_anchors`, `assertion_verdict`, `assertion_observed`, `assertion_findings`, `action_outcome`, `probe_missing`, `step_missing`, `probe_shape`, `vlm_prompt`), a severity (`missing` > `major` > `minor`) and a location (step + probe). Deterministic: identical runs → zero divergences; identical inputs → byte-identical reports; provider-local fields never diverge (proven by flipping every digest and read ref).
  - **adapter.rs + fakes.rs — the adapters (PART 3):** the `LabAdapter` trait (NEW surface owned by this crate, not an ExecutionProvider extension): `descriptor / prepare(journey) / run(step) / capture(step, probe) / evidence`. `prepare` names every missing lab surface (the CAP-001 law applied to labs). `LocalLabAdapter` — the in-repo reference implementation (the contract + the deterministic `FakeAppSurface` standing in for the operator's script-driven local lab); `FakeRemoteLabAdapter` — the CONSISTENCY provider (the ENV-001 law: the fake remote every real remote copies — same driver, remote topology, no `desktop.gui` (the honest named gap), its own digest namespace); `FamilySkeletonAdapter` + `known_provider_families()` — E2B/Daytona/Azure/GitHub Actions/Codemagic as adapter-family DATA with topology metadata (locality, interaction model, lab surfaces, skeleton status) and honest NotWired refusals — NO network. `run_journey` drives every adapter through the four-beat contract uniformly. `FakeAppSurface` honestly models the F1 findings (the promo swallowing keyboard until the defensive Escape, the live PTY without focus transfer, the bracket chords not moving selection while ctrl+page_down does, the modal trap, the companion chord); the two canonical journeys — `shell-discovery` (the d24 model-picker shape) and `a11y-chord-ladder` (the d25/N5/N6/d19 shapes) — with full evidence families for BOTH adapters (four committed evidence fixtures + minimal).
  - **PART 4 — no UI this wave:** the lab's user is the operator; the evidence + comparator outputs are the product. Provider STATUS for end users rides PROV-001's accounts surface — a note, not a dependency.
- tests/commands and exact results (or static-verification statement):
  - The sandbox initially lacked the Rust toolchain; rather than relying on static reasoning alone, I installed the EXACT pinned toolchain (rustup → rustc 1.97.1 / cargo 1.97.1 / clippy 0.1.97 / rustfmt 1.9.0, matching `rust-toolchain.toml`) and ran the gates for real:
  - `cargo fmt --all --check` → **CLEAN** (no diff, exit 0)
  - `cargo clippy --locked -p flauz-lab --all-targets -- -D warnings` → **CLEAN** ("Finished `dev` profile", exit 0)
  - `cargo test --locked -p flauz-lab` → **60 passed, 0 failed** (38 lib + 6 `tests/adapter.rs` + 8 `tests/comparator.rs` + 8 `tests/conformance.rs`)
  - Final re-verification from the clean committed tree via the Lead-side flow (cargo regenerating the lockfile entry): fmt CLEAN, clippy CLEAN, **60/60 PASS**
  - Sibling sanity: `cargo check --locked -p flauz-cap -p flauz-orch` → CLEAN at the new members line
  - `git bundle verify /home/z/my-project/LAB-001-delivery.bundle` → "is okay"; contains `feat/lab-001-fabric` @ a6d37f5; requires the pinned base ref
- kernel compliance checklist (Wave-4 addendum §1-§7, each item):
  1. **Existing signatures frozen — COMPLIANT.** Zero changes to any existing crate (no trait signatures, no additive methods); the `LabAdapter` trait is NEW surface owned by this crate, explicitly sanctioned by the order ("not an ExecutionProvider extension"); no contract-crate imports at all (serde-only, inputs as data).
  2. **The quota-attribution law (F7) — COMPLIANT/N-A.** LAB-001 schedules no provider quota; evidence records carry explicit adapter attribution (topology data) and no credential material.
  3. **Credentials are references, forever — COMPLIANT.** No credential material in any contract type, fixture, log line or serialized state; the `CREDENTIAL_MARKERS` family scan is extended to the new record families (`no_credential_material_in_serialized_state`, value-prefix-adapted for a whole-document scan and documented in the test); no real OAuth anywhere.
  4. **Lab journeys provider-independent by construction (F8) — COMPLIANT (the core law).** The journey is declarative data + the normalized evidence schema; the same journey bytes run unchanged against every compliant adapter (proven in-tests across the local reference + fake-remote consistency providers); the comparator compares through the NORMALIZED schema, never raw screenshots alone; a11y evidence is first-class (the F1 patterns as assertion kinds + the named-findings registry); the real families are skeleton data + fakes with no network.
  5. **Collaboration extends the existing attention model (F9) — COMPLIANT/N-A.** Nothing in flauz-lab creates a second event store or a parallel feed (COL-001's law untouched).
  6. **Private vs shared context explicit (F9) — COMPLIANT/N-A.** No frozen record shapes were mutated; all new types are new v1 families with `deny_unknown_fields`.
  7. **GUI seven-layer rule / user language — COMPLIANT BY ORDER.** No UI this wave (PART 4); the order itself designates the operator as the lab's user and defers end-user provider status to PROV-001's accounts surface.
- acceptance-criteria evidence (map each of the 5 bullets):
  1. **Single clean commit at the pinned base; owned files only** — commit `a6d37f5`, parent exactly `83796f5...`; `git show --stat` = 35 files: `crates/flauz-lab/**` + the one members line; `git status` clean at hand-off; bundle verified.
  2. **The F8 gate law proven in-tests** — `the_same_journey_bytes_produce_comparable_evidence_from_both_adapters`: the journey is serialized ONCE and re-parsed per adapter (same bytes), both records validate, `compare` reports **zero divergences with every step matching** — while the test separately proves the frame digests genuinely differ across the two providers (comparability despite provider-local differences, not because of their absence). Run for BOTH canonical journeys.
  3. **The comparator catches a seeded divergence and names it; identical runs report zero** — `a_seeded_frame_divergence_is_named_severe_and_located` (exactly one divergence: kind `frame_anchors`, severity `major`, step `open-picker` + probe `frame-picker-open`, with both sides' values in the canonical report) and `a_seeded_assertion_divergence_is_named_severe_and_located` (kind `assertion_verdict`); `a_missing_step_is_the_strongest_named_divergence` / `a_missing_probe_is_a_named_divergence` (severity `missing`, side named); `identical_runs_report_zero_divergences` + `provider_local_fields_never_diverge` (zero divergences with every digest and read reference flipped) + `the_comparator_is_deterministic_and_byte_stable`.
  4. **The findings registry carries the F1 a11y lessons as data; records reference by id** — `builtin_findings_registry` carries exactly the five named lessons (ids pinned by `BUILTIN_FINDING_IDS` and asserted in tests + the registry fixture); the a11y chord-ladder journey's evidence links **all five by id** (`evidence_links_findings_by_id_and_they_resolve`, including the honest unsatisfied verdicts with the finding linked); `findings_referenced_by_id_resolve` proves every committed evidence link resolves against the registry.
  5. **Non-empty Contract deviations blocks closure** — see below: **NONE**.
- known limitations:
  - **Cargo.lock entry intentionally not committed** (8 additive lines for `flauz-lab`) per the ownership rules and the documented precedent (the Lead's gate-fixes 8393012/b662c4f for CAP-001/ORCH-004); the local sandbox lockfile carries it for verification, so `--locked` commands at the Lead's merge need the expected lockfile refresh after the members-line union.
  - `FakeAppSurface` is the deterministic stand-in for the operator's script-driven local lab (per the order); the real Xvfb/xdotool/ffmpeg local driver lands behind the same contract in a later wave — the normalized evidence shape is already what it will archive.
  - The frame digest is FNV-1a (dependency-free, per the serde/chrono-family rule): a provider-local determinism fingerprint, NOT cryptographic, and never compared cross-provider (by design).
  - The comparator's top behavioral severity is `major` (a seeded observation/verdict flip); absent evidence yields `missing` — both named and located (the acceptance's "severe" reading).
  - The batch provider families (GitHub Actions, Codemagic) honestly reject interactive UI journeys at `prepare` with named gaps — a real lab journey for batch providers needs a batch-shaped journey vocabulary (later wave).
- contract deviations: **NONE**
- follow-up work:
  - The Lead's wave-gate harness step (per the order): define a journey → run against local + fake-remote via `run_journey` → normalized evidence → the comparator report with a seeded divergence — the fixtures and the driver make this a direct wiring.
  - The real local lab driver (the Lead's d-series scene scripts) behind `LocalLabAdapter`'s contract.
  - Real provider adapters (E2B/Daytona/Azure/GitHub Actions/Codemagic) copying the `FakeRemoteLabAdapter` template once the contract is lab-proven; the `known_provider_families` topologies are the skeleton contracts they fill.
  - Registry growth: new findings land as data entries (the schema is frozen; the entries are operator-maintained).
  - A batch-shaped journey vocabulary for CI-class providers if those ever need parity journeys.
=== END REPORT ===
