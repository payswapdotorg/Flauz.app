# MOD-001 + RT-001 — Completion Report & Lead Gate Record (Wave 2)

**Status: MERGED (PR #46, merge a46b76f)** — the model fabric: provider/model
registry with secure credential references, the Codex adapter boundary + the
first non-Codex runtime (one shared AgentRuntime conformance suite), and the
model-selection UI slice (`flauz_model_picker`, 7 layers).

## Lead gate record

- **Delivery**: worker session 40259d08 (agents tab, GLM-5.3, Full-Stack),
  branch `feat/mod-001-rt-001-model-fabric`, single clean commit `2cd57e3`
  on base `fda38ea` (exact Wave-2 base). Bundle harvested from the worker
  pod (`leads-harvest/40259d08`, 38,528 bytes) — `git bundle verify` PASS.
- **Peak-hours battle**: the dispatch landed 14:23 UTC into a sustained
  GLM-5.3 capacity crunch. The turn was cut repeatedly (the platform's
  peak modal kills turns mid-flight); the Lead recovered it turn-by-turn
  with continuation nudges + capacity-modal dismissals. The worker
  completed the full ladder server-side across cut turns (todo 10/10),
  delivered the bundle, and emitted its report twice — both emissions cut
  after kernel §1–§2 (the archived report below).
- **Gate 0 (bundle)**: PASS — requires exactly `fda38ea`; one commit; 14
  files +3775/−2, all within the owned set.
- **Gate 1 (source review)**: PASS —
  - credential-references-only is documented, implemented (`SecretRef`
    IDs in `ProviderConnection`), and **tested**
    (`no_credential_material_in_serialized_state` + fixture
    `CREDENTIAL_MARKERS` scans);
  - ONE shared `adapter_conformance::assert_agent_runtime_conformance`
    suite runs over BOTH runtime adapters (fake-backed Codex + direct);
  - `runtime_direct` honestly advertises EMPTY capabilities with honest
    gap reporting (the fake-consistency provider, addendum §2);
  - registry semantics: version 1 at create, +1 per durable mutation,
    OCC via `ExecStoreError::VersionConflict`; canonical JSON snapshot
    (`deny_unknown_fields`, no floats, RFC3339-Z);
  - the picker deliberately does NOT import the flauz contract crates —
    catalog/world-store seams are plain data for later waves (kernel §1);
  - ui.rs changes are exactly the 16 named seams (each tagged).
- **Gate 2 (Lead-local + CI)**: fmt clean; clippy `-p flauz-exec` clean;
  `cargo test -p flauz-exec` **78/78** (65 lib + 11 conformance + 2
  dynamics — matches the worker's claim exactly); `cargo check --release
  -p codex-app` EXIT 0 (the picker type-checks; the box could not afford
  the full release codegen — OOM-killed rustc with the stack resident —
  so codegen/test/build ran in CI). **CI GREEN both platforms** at
  `78123b2` (clippy `--workspace --all-targets -D warnings`, test
  `--workspace`, release build, startup smoke).
- **Lead gate-fix r1 (`c03c266`)**: the worker's sandbox cannot compile
  codex-app (pre-existing libpipewire/xcap system-dep gap, verified
  identical at base), so its verification of the picker was static +
  a standalone gpui-stub harness (which ran the real delivered files:
  8/8 picker tests, catching two real bugs during development). The
  Lead's independent compile caught 5 warnings: one unused `gpui`
  import + dead-code on the picker's F7 wiring-seam API → removed the
  import; `#[allow(dead_code)]` with per-item wiring-owner comments.
- **Lead gate-fix r2 (`78123b2`)**: CI clippy failed both platforms on
  two `.expect()` calls in the picker's identity-preservation test
  (workspace denies `expect_used`) → converted to the house test idiom
  (a local `some()` matcher helper, the `runtime_codex` `ok()` pattern).
- **Contract deviations: NONE** (stated twice by the worker; confirmed by
  the Lead's review).

## Worker's COMPLETION REPORT (verbatim, as recovered)

> Emitted twice (both turns cut after §2 of the kernel-compliance
> checklist); recovered from the session's message containers. The
> closing statement is complete. The remaining checklist lines (§3–§7)
> and the later report fields (GUI layers, acceptance mapping,
> limitations, follow-up) were cut by the peak-hours turn deaths; the
> Lead's independent verification above stands in for them. The worker's
> sandbox verification of the GUI slice is the gpui-stub harness (8/8).

```text
=== MOD-001 + RT-001 COMPLETION REPORT ===

Work Order IDs: MOD-001 (model/provider registry) + RT-001 (runtime adapters) — single-worker order, one branch, both IDs

Base branch + SHA: main @ fda38ea576db4d2cf05bb35ad962febb5bc94cb9

Branch/Commits: feat/mod-001-rt-001-model-fabric
  - 2cd57e36d0027a60de227b877a27f792a3d98dd0 "feat(exec,app): MOD-001 + RT-001 — the model fabric: registry, runtime adapters, model picker" (SINGLE clean commit, exactly one commit on top of base; working tree clean)
  - Delivered as git bundle (NOT pushed): /home/z/my-project/MOD-001-RT-001-delivery.bundle — `git bundle verify` confirms it contains refs/heads/feat/mod-001-rt-001-model-fabric @ 2cd57e3 and requires the base ref fda38ea.

Changed files/surfaces (14 files, +3775/−2):
  NEW   crates/flauz-exec/src/registry.rs                    (1349 lines — MOD-001)
  NEW   crates/flauz-exec/src/runtime_codex.rs               (524 lines — RT-001)
  NEW   crates/flauz-exec/src/runtime_direct.rs              (292 lines — RT-001)
  NEW   crates/codex-app/src/ui/flauz_model_picker.rs        (1286 lines — UI slice)
  EDIT  crates/flauz-exec/src/lib.rs                         (+10 module declaration lines only)
  EDIT  crates/flauz-exec/src/fakes.rs                       (+65 additive — FakeCodexServerHandle seam)
  EDIT  crates/codex-app/src/ui.rs                           (+61/−2 — NAMED SEAMS ONLY, each tagged "MOD-001 + RT-001": mod decl, ChooseModelForTask palette variant + ALL/title/description/shortcut/icon/group match arms + dispatch, state field + init, navigation-close hook, task-surface control mount + panel mount, Ctrl+Alt+Shift+M key binding + scoped Escape, workspace-root on_action listener)
  NEW   crates/flauz-exec/tests/fixtures/w2/provider-registration/{typical,minimal,boundary,invalid-availability}.json
  NEW   crates/flauz-exec/tests/fixtures/w2/registry/{typical,minimal,invalid-unknown-field}.json

Implementation summary:
  PART 1 — registry.rs (MOD-001): ModelRegistry with ProviderRegistration CRUD; version 1 at create, +1 per durable mutation, optimistic concurrency via ExecStoreError::VersionConflict; ProviderAvailability with the three honest states (connected / configured-not-connected / not-configured) and connection-consistency rules (a not-configured registration can never advertise connected); ProviderConnection holds ONLY credential reference IDs (flauz-context Secret pattern); RegistrySnapshot serializes as canonical JSON ("v":1, snake_case, deny_unknown_fields, no floats, RFC3339-Z); ProviderSnapshot satisfies the frozen ModelProvider contract — zero trait signature changes. Fixtures: typical/minimal/boundary/invalid per entity family.
  PART 2 — runtime_codex.rs + runtime_direct.rs (RT-001): the Codex adapter maps the existing app-server runtime shape onto AgentRuntime through the CodexServerHandle seam (fake-backed in tests; NO live app-server dependency in the crate — capability gaps are answered with CapabilityGap BEFORE contacting the server). runtime_direct is the first non-Codex runtime: in-process, fake model, honest EMPTY capability advertisement with honest gap reporting. BOTH adapters pass the SAME adapter_conformance::assert_agent_runtime_conformance suite. fakes.rs extended additively with FakeCodexServerHandle.
  PART 3 — flauz_model_picker.rs (UI slice): shell-family surface following the flauz_shell/d23 patterns — visible labeled model control on every task, the picker panel, the "Choose a model…" palette row, the Ctrl+Alt+Shift+M chord (letter family — reports shift truthfully, so no shifted-symbol companion per the N6 gate-fix) + scoped Escape, honest empty state (what a model provides + next step pointing at Settings — NO connection flow, that is F7), success state (active model + provider + capability hints + identity-preserved confirmation), honest unavailable state (configured-not-connected / not-configured rows are visible but not runnable). Model switches record a task.model_changed event through the world-store seam (TaskModelEventLog, strictly increasing seq) and preserve task identity BY CONSTRUCTION (per-task_id keyed state, append-only log). The picker deliberately does NOT import the flauz contract crates — the catalog/world-store seams are plain data for later waves to wire (kernel §1 boundary).

Tests/commands + exact results (toolchain WAS available in sandbox):
  - cargo fmt --check -p flauz-exec                       → CLEAN (exit 0)
  - cargo clippy -p flauz-exec --all-targets -- -D warnings → CLEAN ("Finished dev profile", zero warnings)
  - cargo test -p flauz-exec                              → 78 passed, 0 failed (65 lib + 11 conformance + 2 exec_dynamics); includes version_monotonic_and_version_conflict_rejected, no_credential_material_in_serialized_state, provider_connection_holds_only_secret_references, fixtures_roundtrip_canonical
  - rustfmt --check --edition 2024 on ui.rs + flauz_model_picker.rs → CLEAN
  - Picker verification harness (/home/z/picker-verify): the sandbox cannot compile codex-app (xcap → libspa-sys needs system libpipewire-0.3, absent, uninstallable without root — verified IDENTICAL failure at the BASE SHA via git stash, i.e. pre-existing, not caused by this change). The harness type-checks and RUNS the REAL delivered files (symlinks, not copies) against inert gpui/gpui_component/codex_core stubs: cargo test → 8 passed, 0 failed — including model_switch_preserves_task_identity_through_the_world_store_seam, picker_seams_are_registered_in_the_ui_seams, unavailable_models_are_never_runnable, capability_hints_use_user_language, picker_copy_avoids_internal_type_names. The harness caught and fixed two real bugs during development (a cx.theme() syntax error; an outer-closure move of &mut cx).

Kernel-compliance checklist (Wave-2 addendum §1–§7):
  §1 Provider fabric behind frozen interfaces — COMPLIANT: registry/runtimes implement existing flauz-exec traits (ModelProvider satisfied by ProviderSnapshot; both adapters satisfy AgentRuntime); NO trait signature changes; fakes.rs additions keep all fake conformance implementations compiling (clippy -D warnings green).
  §2 First external provider is a fake consistency provider — COMPLIANT (as scoped): runtime_direct's fake model is deterministic in-process; no real remote provider introduced (non-goal respected).

Ladder closed. All parts (PART 1 registry, PART 2 runtimes, PART 3 picker + ui.rs seams) are delivered in single clean commit 2cd57e3, the bundle sits at /home/z/my-project/MOD-001-RT-001-delivery.bundle (verified, not pushed), every gate is green (fmt/clippy clean, 78/78 + 8/8 tests), and there are no contract deviations. Nothing remains for this work order.
```

## Post-merge notes

- The registry + runtime adapters + picker are now on `main` (a46b76f).
- The F7 wiring seams (`set_catalog`, `active_model_id`, `events`,
  availability construction) are `#[allow(dead_code)]`-annotated with the
  F7 wiring owner — the BYOP/registry-load slice wires them.
- Wave-2 integration gate: after ENV-001 + CAP-001 merge, extend the F2
  §10 harness with the registry/runtime/picker steps (provider
  registration → connection (SecretRef) → model lookup → adapter turn →
  model switch preserves task identity) per the F2 gate pattern.
