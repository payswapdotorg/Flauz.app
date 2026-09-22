# Wave 2 Work Orders — Environment / Model / Capability fabric

> **Status: 3 of 3 MERGED — Wave 2 DELIVERED** (MOD-001+RT-001 Worker B —
> PR #46, merge a46b76f; CAP-001 Worker C — PR #47, merge 93125d0, see
> [evidence/w2-cap-001](evidence/w2-cap-001/CAP-001-COMPLETION-REPORT.md);
> ENV-001 Worker A — PR #48, merge f5fae68, see
> [evidence/w2-env-001](evidence/w2-env-001/ENV-001-COMPLETION-REPORT.md);
> all CI both platforms, deviations NONE ×3; the Wave-2 integration gate
> record at [evidence/w2-gate](evidence/w2-gate/WAVE2-GATE-RECORD.md)).
> Shared-contract authority: [F2-CONTRACT-KERNEL.md](F2-CONTRACT-KERNEL.md)
> (frozen, Wave 1) + the **Wave-2 kernel addendum** below + the MERGED
> contract crates themselves (flauz-world / flauz-exec / flauz-context at
> 5fd7178 — the crates are the implementation truth). Every work order
> follows [WORK-ORDER-TEMPLATE.md](../WORK-ORDER-TEMPLATE.md). Non-empty
> `Contract deviations` in a worker report blocks closure. Workers deliver
> via `git bundle` on a single clean commit branch; the Lead gates, fixes,
> merges.

## Wave-2 kernel addendum (frozen by the Tech Lead before dispatch)

1. **Provider fabric lives BEHIND the frozen interfaces.** Every Wave-2
   provider (local environment wrappers, the fake-consistency remote
   environment, the model/provider registry, the Codex adapter, the first
   non-Codex runtime) implements an existing `flauz-exec` trait
   (`Environment`, `ExecutionProvider`, `ModelProvider`, `AgentRuntime`).
   NO trait signature changes without a Contract deviation. Additive
   trait methods require the fake conformance implementations to keep
   compiling — if an addition is unavoidable, mark it a deviation and
   justify.
2. **The first external provider is a FAKE consistency provider.** ENV-001
   must deliver `FakeRemoteEnvironmentProvider` (deterministic, in-memory,
   satisfying the SAME `Environment` contract as the local wrapper) BEFORE
   any real remote provider (E2B etc. is a later wave). Cross-environment
   task-identity evidence runs local → fake-remote → local.
3. **Credentials are references, never material.** Provider connections
   carry secure reference IDs (the `flauz-context` Secret pattern: a
   reference string, storable, never compiled, never logged). Any
   credential VALUE in code, fixtures, logs, or serialized state is an
   automatic gate failure.
4. **Capability resolution is an intersection with named gaps.** A
   capability is available iff model ∩ runtime ∩ environment ∩
   permissions ∩ workspace-policy ALL admit it. Every missing dimension
   is NAMED in the resolution record. Silent fall-through (a capability
   quietly missing from the offered set) is a contract violation.
5. **Registry entities follow the Wave-1 entity rules** (`flauz-world`
   kernel §3): version 1 at create, +1 per durable mutation, optimistic
   concurrency on update, canonical JSON (`"v":1`, snake_case,
   deny_unknown_fields, no floats, RFC3339-Z), fixture per entity
   (typical/minimal/invalid).
6. **Task identity is sacred.** No model switch, environment switch, or
   capability re-resolution may fork a logical task (the Gate A
   round-trip asserts this today; Wave-2 UI must preserve it).
7. **GUI slices follow the seven-layer rule** (PRODUCT-UX-JOURNEYS §1):
   visible primary entry + contextual affordance + palette fallback +
   useful empty state + success/next state + keyboard path + honest
   unavailable state. The command palette is never the only discovery
   mechanism.

---

## ENV-001 — Environment fabric: the local provider + the fake-consistency remote

```
ID: ENV-001
Title: LocalEnvironmentProvider (terminal/browser/computer-use/git topology)
  + FakeRemoteEnvironmentProvider (the first external provider is a fake)
Phase: Wave 2 (F3 environment fabric)
Owner: Worker A (agents-tab session)
Dependencies: flauz-exec merged at 5fd7178 (Environment/ExecutionProvider
  contracts + fakes); flauz-world (task identity); F2 gate A harness
  pattern (lead-tools/f2-integration-harness + docs/research/evidence/f2-gate/)
Contract(s): F2-CONTRACT-KERNEL.md; this addendum §1-§6;
  FLAUZ-SOURCE-OF-TRUTH.md (environment invariants)
Problem: the Environment contract exists but nothing real implements it —
  no local provider wrapping the F1 surfaces (terminal, browser, Computer
  Use, Git), and no path to remote environments that does not silently
  fork task identity.
User-visible outcome: a task can attach a local environment (its
  terminal/browser/git surfaces) and a remote sandbox environment through
  ONE interface; switching local → fake-remote → local keeps the same
  logical task (J-05 Environment, J-06 Cross-environment).
Scope:
  - crates/flauz-exec/src/environment_local.rs (or a provider module
    family): LocalEnvironmentProvider — wraps the four F1 surfaces as
    Environment facets; descriptor + topology metadata; deterministic
  - crates/flauz-exec/src/environment_fake_remote.rs:
    FakeRemoteEnvironmentProvider — deterministic in-memory remote
    (latency/echo semantics optional), same Environment contract, an
    env_ ULID identity stable across serialization
  - cross-environment evidence test: attach local → attach fake-remote →
    back to local → assert same TaskRef, no fork, environment events on
    the task stream (use flauz-world fakes as the Gate A harness does)
  - fixtures per kernel addendum §5; public fakes extended for downstream
Non-goals: NO real remote provider (E2B/Daytona — later wave), NO UI work
  (the environment surfaces already render honest empty states; live
  wiring is a later slice), NO trait signature changes.
Files/subsystems owned: crates/flauz-exec/src/environment_local.rs,
  environment_fake_remote.rs, fixtures under
  crates/flauz-exec/tests/fixtures/w2/, fakes.rs (additive only),
  Cargo.toml ONLY if a module list line is needed. Nothing else.
Inputs: flauz-exec Environment/ExecutionProvider traits; the F1 surface
  topology (codex-app terminal/browser/computer-use/git — read for the
  descriptor shape, do NOT import codex-app)
Outputs/artifacts: the two providers + conformance tests + fixtures
Tests: both providers satisfy the SAME Environment contract test set
  (the ARCH-002 conformance pattern); the cross-environment identity
  test; fmt/clippy/test green (state plainly if the sandbox lacks the
  toolchain — the Lead independently gates)
GUI/lab evidence: Lead gate — extend the F2 harness cross-environment
  step (local→fake-remote→local) + a lab scene at the merged binary
UX journey IDs: J-05, J-06
Primary discovery surface: n/a (contracts; UI is a later slice)
Contextual discovery surface: n/a
Search/palette discovery: n/a
Empty/success-state behavior: n/a
Acceptance criteria:
  1. Single clean commit on feat/env-001-environment-fabric at the
     recorded base; owned files only.
  2. LocalEnvironmentProvider + FakeRemoteEnvironmentProvider both
     satisfy the same Environment conformance suite.
  3. Cross-environment test: local → fake-remote → local keeps the same
     logical task; events recorded; no fork.
  4. No credential material anywhere (addendum §3).
  5. Canonical JSON + fixtures per addendum §5.
  6. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive modules; revert the commit.
Integration notes: the provider family is the template every later real
  provider (E2B, Daytona) must copy; keep the fake-remote deterministic.
Status: MERGED (2026-09-22) — PR #48 (merge f5fae68): worker commit 0a71706
  (bundle-verified, base fda38ea exact) + the ci.yml workflow_dispatch
  cherry-pick (9839c09, the event-swallow workaround). CI dispatched GREEN
  both platforms at 9839c09. Merge resolution: fakes.rs keep-all-lines
  (doc table: mod-001's CodexServerHandle row + env-001's Environment row);
  lib.rs auto-merged complete. Gates on the merged tree: fmt clean, clippy
  -p flauz-exec clean, test -p flauz-exec 90/90. Deviations: NONE. Evidence:
  evidence/w2-env-001/ENV-001-COMPLETION-REPORT.md
```

---

## MOD-001 + RT-001 — Model/provider registry + the first non-Codex runtime

```
ID: MOD-001 (model/provider registry) + RT-001 (runtime adapters)
Title: The model fabric — registry, capability metadata, secure
  credential references, the Codex adapter boundary, the first non-Codex
  runtime, and the model-selection UI slice
Phase: Wave 2 (F3 model/runtime fabric)
Owner: Worker B (agents-tab session; one branch, both IDs)
Dependencies: flauz-exec merged at 5fd7178 (ModelProvider/Model/
  AgentRuntime contracts + fakes); flauz-context (ModelRef, profiles);
  the shell (flauz_shell surfaces)
Contract(s): F2-CONTRACT-KERNEL.md; this addendum §1-§7;
  CONTEXT-HARNESS-ARCHITECTURE.md §13 (model-aware compilation profiles)
Problem: models and runtimes have contracts but no registry, no
  capability metadata layer, no credential-reference discipline, and the
  app has no model-selection surface — every future model/provider
  capability would land as developer-only configuration.
User-visible outcome: a user can see the available models (the registry),
  pick one for a task, and switch — the task keeps its identity and
  context (J-13 Model switching); the UI names the provider and shows
  which quota/connection a model draws on (the BYOP preview — F7 does the
  full connection flows).
Scope:
  - crates/flauz-exec/src/registry.rs: the model/provider registry —
    ModelProvider registrations, model capability metadata (the frozen
    CapabilityId vocabulary), ProviderConnection with SECURE REFERENCE
    credentials (addendum §3), availability states (connected /
    configured-not-connected / not-configured)
  - crates/flauz-exec/src/runtime_codex.rs: the Codex adapter boundary —
    maps the existing Codex app-server runtime shape onto AgentRuntime
    (fake-backed in tests; no live app-server dependency in the crate)
  - crates/flauz-exec/src/runtime_direct.rs: the first non-Codex runtime
    (direct in-process runtime; fake model, honest capability
    advertisement — the CapabilityGap path must stay honest)
  - crates/codex-app/src/ui/flauz_model_picker.rs: the model-selection
    surface — a shell-family surface (the d23 patterns): visible entry
    point on the task surface (a labeled control), palette row, keyboard
    path, honest empty state (no models configured → what a model is +
    how to connect one, pointing at settings), model switch keeps task
    identity (a task.model_changed event via the world store seam)
  - ui.rs: ONLY the named seams (module declaration, palette/nav
    registration, bindings) — your seams are distinct from CAP-001's
Non-goals: NO real provider integrations (Gemini/Anthropic/E2B — F7+),
  NO OAuth/API-key flows (F7), NO quota metering (F7), NO trait signature
  changes, NO shell/other-surface edits.
Files/subsystems owned: crates/flauz-exec/src/{registry,runtime_codex,
  runtime_direct}.rs; crates/flauz-exec/tests/fixtures/w2/**; fakes.rs
  (additive); crates/codex-app/src/ui/flauz_model_picker.rs; ui.rs NAMED
  SEAMS ONLY (declaration + your palette/nav/binding registrations);
  Cargo.toml only if module lines are needed.
Inputs: the ARCH-002 contracts; the shell module patterns
  (ui/flauz_shell/mod.rs); d23's chord/label discipline
Outputs/artifacts: registry + adapters + the picker surface + tests +
  fixtures
Tests: registry CRUD + version rules (addendum §5); credential-reference
  discipline test (no material in serialized state — the flauz-context
  Secret pattern); both runtimes satisfy the SAME AgentRuntime
  conformance suite; the model-switch UI keeps task identity (unit-level
  seam test); shell-family UI tests (copy, empty state, palette row,
  keyboard chord) in the flauz_model_picker module
GUI/lab evidence: Lead gate — a lab scene at the merged binary: model
  picker visible on the task surface → open → select model B → same task
  (J-13); empty state at cold start
UX journey IDs: J-13 (model switching), J-01 (start — the picker is part
  of the task surface)
Primary discovery surface: the labeled model control on the task surface
Contextual discovery surface: the picker panel while a task is selected
Search/palette discovery: a palette row ("Choose a model…")
Empty/success-state behavior: no models → honest copy (what a model
  provides + the connection next-step); models present → the list with
  provider + capability hints; after a switch → the active model named,
  task identity visibly preserved
Acceptance criteria:
  1. Single clean commit on feat/mod-001-rt-001-model-fabric at the
     recorded base; owned files only.
  2. Registry: CRUD, version rules, canonical JSON, fixtures; credential
     REFERENCES only — the no-material test proves it.
  3. runtime_codex + runtime_direct satisfy the same AgentRuntime suite.
  4. The picker: seven layers (visible control, panel, palette row,
     keyboard chord, honest empty state, success state, honest
     unavailable state); "reusable workflow"-style user language (no
     internal type names in copy).
  5. Model switch preserves task identity (unit seam test + the Lead's
     lab scene).
  6. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive modules + seams; revert the commit.
Integration notes: the registry is the F7 BYOP foundation — keep the
  ProviderConnection states honest (the UI must never imply a connection
  exists when it is only configured).
Status: MERGED (2026-09-22) — PR #46 (merge a46b76f): worker commit 2cd57e3
  (bundle-verified, base fda38ea exact) + Lead gate-fixes c03c266 (picker
  compile warnings: unused import + F7 wiring-seam allows) & 78123b2 (CI
  clippy: expect_used → the house some() idiom). Gates: fmt/clippy clean;
  flauz-exec 78/78; picker type-checked + stub-harness 8/8 (the sandbox's
  libpipewire gap is pre-existing, verified at base); CI GREEN both
  platforms. Deviations: NONE. Evidence:
  evidence/w2-mod-001-rt-001/MOD-001-RT-001-COMPLETION-REPORT.md
```

---

## CAP-001 — The capability resolver + the gap UX

```
ID: CAP-001
Title: Capability resolution (model ∩ runtime ∩ environment ∩ permissions
  ∩ workspace policy) with the named-gap UX — no silent fall-through
Phase: Wave 2 (F3 capability fabric)
Owner: Worker C (agents-tab session)
Dependencies: flauz-exec merged at 5fd7178 (CapabilityId, skills,
  runtimes); flauz-world (resource states, permissions); the shell
Contract(s): F2-CONTRACT-KERNEL.md; this addendum §4-§7;
  PRODUCT-UX-JOURNEYS.md §1 (discoverability), §2.1 (the rail)
Problem: capabilities exist as IDs but nothing resolves whether a
  capability is ACTUALLY available for a task — and when it is not, the
  product today has no way to say why, what is missing, or how to unlock
  it (the silent fall-through the kernel forbids).
User-visible outcome: when a capability is unavailable, the user sees
  "Capability unavailable" → Why? → the named missing dimensions (model
  lacks it / runtime doesn't advertise it / environment has no such
  surface / permission denied / workspace policy) → ways to unlock
  (J-04 Capability gap). No capability silently disappears.
Scope:
  - crates/flauz-cap/** (NEW self-contained crate, the flauz-context
    pattern: serde+chrono only): CapabilityResolution records —
    requested capability, per-dimension admission (model, runtime,
    environment, permissions, workspace policy), the named gap list,
    unlock paths (per missing dimension, an honest action hint);
    canonical JSON + fixtures per addendum §5; public fakes
  - the resolver function: intersection over the frozen interfaces
    (Model capability metadata, AgentRuntime advertisement, Environment
    surfaces, the permission/policy inputs as call-side data — the crate
    takes inputs, does not import the world)
  - crates/codex-app/src/ui/flauz_capability_gap.rs: the gap surface —
    reachable where capabilities are surfaced (the task rail's
    More/Inspect neighborhood), the Why/What-missing/Ways-to-unlock
    disclosure chain, keyboard path, honest copy; and the
    capability-available success state (what is admitted, from which
    dimensions)
  - ui.rs: ONLY the named seams (declaration + your registrations) —
    distinct from MOD-001's seams
Non-goals: NO permission-system implementation (permissions are inputs),
  NO workspace-policy engine (inputs), NO skill installation flows, NO
  trait changes, NO other-surface edits.
Files/subsystems owned: crates/flauz-cap/**; crates/codex-app/src/ui/
  flauz_capability_gap.rs; ui.rs NAMED SEAMS ONLY; the workspace
  Cargo.toml members line (one line, the flauz-context pattern).
Inputs: the ARCH-002 capability/runtime contracts; PRODUCT-UX-JOURNEYS
  J-04; the d23 shell discipline
Outputs/artifacts: the resolver crate + the gap surface + tests + fixtures
Tests: the intersection algebra (all dimensions, each gap named); the
  no-silent-fall-through property (every requested-but-missing
  capability yields a resolution with a non-empty gap list); canonical
  JSON + fixtures; the UI module tests (copy, disclosure chain, palette
  row, keyboard chord, honest states)
GUI/lab evidence: Lead gate — a lab scene: a capability gap surfaced on
  the task surface → Why? → the named dimensions → unlock hints (J-04)
UX journey IDs: J-04 (capability gap), J-02 (context view — capability
  admission is inspectable)
Primary discovery surface: the affordance where capabilities are listed
  on the task surface
Contextual discovery surface: the gap disclosure panel
Search/palette discovery: a palette row ("Why is a capability
  unavailable?")
Empty/success-state behavior: no gaps → the honest "everything
  available" state with dimensions; gaps → the named-dimension chain;
  nothing hidden
Acceptance criteria:
  1. Single clean commit on feat/cap-001-capability-resolver at the
     recorded base; owned files only.
  2. The resolver: intersection with named gaps; the no-silent-
     fall-through property test; deterministic; inputs only (no world
     imports).
  3. The gap UX: the full Why/What-missing/Ways-to-unlock chain,
     seven-layer discipline, user language.
  4. Canonical JSON + fixtures per addendum §5.
  5. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive crate + module; revert the commit.
Integration notes: the resolution record is the evidence type future
  waves (skill unlock flows, permission UIs) build on; keep the unlock
  hints honest (never promise an unlock the product cannot perform).
Status: MERGED (2026-09-22) — PR #47 (merge 93125d0): worker commit 158fa0a
  (bundle-verified, base fda38ea exact) + Lead gate-fix 8393012 (gap-surface
  dead-code allows — the picker precedent) + the ci.yml workflow_dispatch
  cherry-pick (02d8473, the event-swallow workaround). CI dispatched GREEN
  both platforms at 02d8473. Gates: flauz-cap 31/31, codex-app clippy clean,
  fmt clean. Deviations: NONE. Evidence:
  evidence/w2-cap-001/CAP-001-COMPLETION-REPORT.md
```

---

## Reporting contract (all Wave-2 workers)

The 11-field completion report (exact headers), delivered in the worker's
final message AND via the delivery bundle branch: WO ID(s); base branch +
SHA; branch/commits; changed files/surfaces; implementation summary;
tests/commands + exact results (state plainly if the sandbox lacks the
Rust toolchain — static reasoning is acceptable, the Lead independently
compiles and gates); kernel-compliance checklist (this addendum's §1-§7,
each item); GUI discoverability layers covered (UI-bearing WOs);
acceptance-criteria evidence (map each bullet); known limitations;
contract deviations (NONE if none); follow-up work.
