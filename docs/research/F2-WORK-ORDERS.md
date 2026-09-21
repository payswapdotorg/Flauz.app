# F2 Contract Wave Work Orders

> **Status: ACTIVE — Wave 1 dispatched 2026-09-21** (ARCH-001, ARCH-002,
> ORCH-001+UX-001). Shared-contract authority:
> [F2-CONTRACT-KERNEL.md](../F2-CONTRACT-KERNEL.md) — frozen by the Tech Lead
> before any Wave-1 worker branched. Every work order below follows
> [WORK-ORDER-TEMPLATE.md](../WORK-ORDER-TEMPLATE.md). Non-empty
> `Contract deviations` in a worker report blocks closure.

**Repository:** `payswapdotorg/Flauz.app`
**Phase:** F2 — Canonical contracts (roadmap: ACTIVE)

---

## ARCH-001 — World-state, resource and evidence contracts

```
ID: ARCH-001
Title: flauz-world — canonical world-state, resource, evidence and lease contracts
Phase: F2 Wave 1
Owner: Worker A (dispatched 2026-09-21, agents-tab session)
Dependencies: docs/F2-CONTRACT-KERNEL.md (frozen); F1 closed at 16e38df
Contract(s): FLAUZ-SOURCE-OF-TRUTH.md; CONTEXT-HARNESS-ARCHITECTURE.md §5-7, §14;
             F2-CONTRACT-KERNEL.md (all sections)
Problem: the platform has no typed canonical durable-state model; every future
  layer (environments, models, context, orchestration) needs the world-state
  spine to exist exactly once, with claim/evidence and lease semantics that
  cannot be silently violated later.
User-visible outcome: indirect in F2 (enables the Workspace/Task/Artifacts/
  Activity shell surfaces and the later "Save as a reusable workflow" slice);
  direct user value arrives when the F2 gate round-trip demo runs.
Scope:
  - new crate crates/flauz-world: Workspace, Session, Task, Artifact,
    Resource, ResourceState, Event + EventEnvelope, Observation, Claim,
    Evidence, ResourceLease, Procedure, ProcedureVersion
  - canonical IDs + versions (kernel §2-3), event envelope v1 (kernel §5),
    task identity semantics (kernel §6)
  - resource/access-surface distinction; claim vs evidence; verification
    states; bounded expiring attributable leases
  - attribution/provenance references on every record
  - serialization/deserialization per kernel §4; store traits + in-memory
    fakes (public fakes module); conformance fixtures per kernel §8
Non-goals: no remote providers, no collaboration, no GUI, no persistence
  engine beyond in-memory fakes + canonical serialization, no execution
  semantics (that is ARCH-002's crate), no Project entity.
Files/subsystems owned: crates/flauz-world/**; one members line in root
  Cargo.toml (kernel §1 rule). Nothing else.
Inputs: kernel test vectors; CONTEXT-HARNESS-ARCHITECTURE.md
Outputs/artifacts: crate + unit tests + fixtures f2/* + crate README
Tests: kernel §9 names (world-side): kernel_id_valid/invalid_vectors,
  version_monotonic_and_version_conflict_rejected, envelope_roundtrip_all_fields,
  envelope_unknown_field_rejected, task_identity_survives_model_switch,
  claim_cannot_be_constructed_as_evidence, conflicting_observations_stay_distinct,
  lease_expiry_enforced_on_read, lease_is_attributable_and_bounded,
  resource_identity_stable_across_surface_changes,
  procedure_roundtrip_and_version_lineage,
  no_credential_material_in_serialized_state
GUI/lab evidence: none (contract layer); consumed by the Lead's F2 gate
  harness and UX-001 shell.
UX journey IDs: enables J-09/J-10/J-16 (later waves wire them)
Acceptance criteria:
  1. Single clean commit on feat/arch-001-world-contracts at the recorded
     base; crates/flauz-world + the one members line only.
  2. cargo fmt/clippy/test green for the crate; workspace builds.
  3. Fake in-memory implementation passes without external services.
  4. State survives serialize→drop→reload round-trip with equality.
  5. Task identity preserved across a recorded model-switch event.
  6. Resource identity stable when surfaces change.
  7. Claims cannot be represented as verified evidence (type + serialization
     + parse levels).
  8. Conflicting observations remain distinguishable.
  9. Leases bounded, expiring (enforced on read), attributable.
  10. Non-empty Contract deviations blocks closure.
Rollback/recovery: crate is additive; revert the merge commit.
Integration notes: envelope + IDs are the cross-crate seam (kernel §5-6);
  the Lead's gate harness wires flauz-exec/flauz-context to these types via
  the frozen formats.
Status: DISPATCHED (2026-09-21)
```

---

## ARCH-002 — Execution, provider, model and agent contracts

```
ID: ARCH-002
Title: flauz-exec — provider-neutral execution/model/agent/skill contracts + versioned transport
Phase: F2 Wave 1
Owner: Worker B (dispatched 2026-09-21, agents-tab session)
Dependencies: docs/F2-CONTRACT-KERNEL.md (frozen); F1 closed at 16e38df
Contract(s): FLAUZ-SOURCE-OF-TRUTH.md; CONTEXT-HARNESS-ARCHITECTURE.md §2-3, §9-13;
             F2-CONTRACT-KERNEL.md (all sections)
Problem: execution, models, runtimes, skills and providers have no
  provider-neutral contract layer; the current app binds Codex app-server
  behavior directly. Every future runtime (Copilot, direct-model, custom) and
  every future provider needs one neutral contract surface.
User-visible outcome: indirect in F2; enables model-switch-preserves-task
  (J-14), environment switching (J-15) and capability-gap diagnosis (J-04) in
  later waves.
Scope:
  - new crate crates/flauz-exec: Environment, ExecutionProvider,
    ModelProvider, Model, AgentRuntime, Agent, Skill, Capability,
    ProviderConnection
  - provider-neutral traits; capability advertisement; model/runtime
    separation; agent identity separate from environment identity
  - skill requirement representation (kernel §2 key grammar) — provider/model
    neutral
  - ProviderConnection with secret_ref references only (kernel §7)
  - versioned session/event transport generic over
    E: Serialize + DeserializeOwned + ... (kernel §5)
  - fake Codex runtime, fake non-Codex runtime, fake local environment, fake
    remote environment (public fakes module); fixtures per kernel §8
Non-goals: no real Gemini/Anthropic/E2B/Copilot/Ollama integrations, no
  credential storage, no registry UI, no scheduling/routing policy, no GUI.
Files/subsystems owned: crates/flauz-exec/**; one members line in root
  Cargo.toml. Nothing else.
Inputs: kernel test vectors; CONTEXT-HARNESS-ARCHITECTURE.md
Outputs/artifacts: crate + unit tests + fixtures f2/* + crate README
Tests: kernel §9 names (exec-side): kernel_id_valid/invalid_vectors,
  transport_roundtrip_preserves_envelope_value,
  fake_runtimes_satisfy_agent_runtime_contract (both fakes),
  environment_contract_same_for_local_and_remote_fakes,
  model_has_no_provider_environment_behavior,
  provider_connection_holds_only_secret_references,
  version_monotonic_and_version_conflict_rejected,
  no_credential_material_in_serialized_state
GUI/lab evidence: none (contract layer).
UX journey IDs: enables J-04/J-05/J-14/J-15 (later waves wire them)
Acceptance criteria:
  1. Single clean commit on feat/arch-002-exec-contracts at the recorded
     base; crates/flauz-exec + the one members line only.
  2. cargo fmt/clippy/test green for the crate; workspace builds.
  3. Fake Codex runtime AND fake non-Codex runtime satisfy the same
     AgentRuntime contract (same conformance test, both fakes).
  4. Fake local AND fake remote environments satisfy the same Environment
     contract (same conformance test, both fakes).
  5. Model contains no provider-specific environment behavior; Environment
     does not determine Model (compile-level separation, tested).
  6. Skill semantics provider/model neutral.
  7. Credentials represented only as opaque flausec_ references.
  8. Transport round-trips envelope values byte-preservingly (serialize →
     frame → parse → deserialize, value equality).
  9. Non-empty Contract deviations blocks closure.
Rollback/recovery: crate is additive; revert the merge commit.
Integration notes: no cargo dependency on flauz-world/flauz-context in Wave 1
  (kernel §1); the Lead's gate harness feeds flauz-world's EventEnvelope
  through the transport generic at integration.
Status: DISPATCHED (2026-09-21)
```

---

## ORCH-001 + UX-001 — Context contracts + discoverable platform shell

```
ID: ORCH-001 (context contracts) + UX-001 (platform GUI shell)
Title: flauz-context + the first discoverable Flauz platform shell
Phase: F2 Wave 1
Owner: Worker C (dispatched 2026-09-21, agents-tab session; one branch, both IDs)
Dependencies: docs/F2-CONTRACT-KERNEL.md (frozen); PRODUCT-UX-JOURNEYS.md §2.1;
             F1 closed at 16e38df
Contract(s): FLAUZ-SOURCE-OF-TRUTH.md; CONTEXT-HARNESS-ARCHITECTURE.md §2, §12-13;
             F2-CONTRACT-KERNEL.md; PRODUCT-UX-JOURNEYS.md §2.1
Problem: (a) context has no contract distinct from transcript/session;
  (b) the future platform's concepts (Projects/Tasks, Procedures, Artifacts,
  Activity; task rail Context/Agents/Environments/Evidence) are invisible in
  the product — future capabilities would land as developer-only
  infrastructure.
User-visible outcome: a cold-start user can see and reach every top-level
  platform concept in the GUI without knowing internal terminology; not-yet-
  implemented surfaces show useful empty states instead of hiding controls.
Scope — ORCH-001 (crates/flauz-context):
  - Context as projection, not transcript; MemoryItem with HOT/WARM/COLD tiers
  - ContextSnapshot (durable, serializable, reconstructible from references)
  - ContextProvenance (every included item attributable to a source)
  - ModelContextProfile (model-specific compilation profile: capacity,
    multimodal, tool-schema handling)
  - snapshot/reset representation; authorization-filtering representation
    (provenance carries authorization class)
  - public fakes; fixtures per kernel §8
Scope — UX-001 (crates/codex-app):
  - Workspace navigation: Projects / Tasks, Procedures, Artifacts, Activity
  - Task rail: Context, Agents, Environments, Evidence, More / Inspect
  - honest empty states (e.g. "No additional agents are assigned. Delegate a
    task when parallel work would help.") — never hidden controls
  - command-palette rows for every new surface; keyboard paths to all of them
  - user-facing language: "Save as a reusable workflow" (never the internal
    word "Procedure" in UI copy)
  - existing F1 flows unchanged (palette, navigation, focus contracts)
Non-goals: no live wiring of shell surfaces to contract crates (that begins
  with F3+ slices), no context compilation engine, no retrieval/ranking
  (F6), no new F1 behavior changes.
Files/subsystems owned: crates/flauz-context/**; one members line;
  crates/codex-app/src/ui/flauz_shell/** + minimal ui.rs seams (module
  declaration, navigation/palette registration). Nothing else.
Inputs: kernel vectors; PRODUCT-UX-JOURNEYS.md §2.1; existing ui.rs patterns
Outputs/artifacts: crate + tests + fixtures; shell module + palette/nav
  registration; unit tests for empty-state copy/palette rows
Tests: kernel §9 names (context-side): kernel_id_valid/invalid_vectors,
  context_snapshot_reconstructible_from_references,
  context_provenance_covers_every_item,
  context_compilation_does_not_mutate_task_refs,
  version_monotonic_and_version_conflict_rejected; plus UI unit tests for
  nav/palette registration (house ui.rs test style)
GUI/lab evidence: Lead gate — new shell-discovery scene (cold start: every
  §2.1 surface reachable via primary nav + palette + keyboard; empty states
  rendered) + F1 regression scenes (palette rows, palette/overlay close
  focus, navigation) at the merged binary.
UX journey IDs: J-01 (start), J-02 (context view), J-09 (evidence view),
  J-17 (activity), J-10/J-11 (procedure surfaces — empty states now,
  function later)
Primary discovery surface: workspace navigation (persistent, labeled)
Contextual discovery surface: task rail controls while a task is selected
Search/palette discovery: palette rows for every new surface
Empty/success-state behavior: every not-yet-implemented surface shows what
  it will do and what to do next; no dead ends
Acceptance criteria:
  1. Single clean commit on feat/orch-001-ux-001-context-shell at the
     recorded base; owned files only.
  2. cargo fmt/clippy/test green; workspace builds; F1 ui.rs tests still green.
  3. Context is a projection: snapshots reference durable entities by
     canonical ID; reconstructible; provenance covers every item.
  4. Context compilation/switching never mutates task references.
  5. Cold-start discoverability: every new top-level concept reachable via
     nav, palette and keyboard; empty-state copy explains the feature and
     the next action.
  6. UI copy uses "reusable workflow" language; no internal type names.
  7. F1 regression scenes green at the merged binary (Lead gate).
  8. Non-empty Contract deviations blocks closure.
Rollback/recovery: shell is additive UI; context crate is additive; revert
  the merge commit.
Integration notes: flauz-context is self-contained (kernel §1); the shell
  does not import flauz crates in Wave 1.
Status: DISPATCHED (2026-09-21)
```

---

## UX-003 — F2 accessibility carry-overs (queued)

```
ID: UX-003
Title: F2 accessibility carry-overs from the F1 binding battery
Phase: F2 (dispatch when a Wave-1 slot frees or immediately after Wave-1 merges)
Owner: unassigned (queued)
Dependencies: F1 closure record honest-notes section; A11Y-BINDING-VERDICT.md §2
Contract(s): PRODUCT-UX-JOURNEYS.md §7 (accessibility)
Problem: three bounded a11y findings were carried out of F1 and must be
  resolved inside F2 so the new shell inherits a clean keyboard contract.
User-visible outcome: keyboard focus transfers when opening the terminal
  (PTY) on a selected chat; bracket-swap chords positively evidenced; a
  fresh-profile first run is keyboard-usable without a hidden modal trap.
Scope:
  - N5: PTY focus transfer on open (Ctrl+` on a selected live chat opens a
    real shell but typed input does not land; fix-shape mirrors the
    017/N1 family — explicit focus transfer on open)
  - N6: bracket-swap chords positive re-evidence (Ctrl+PageDown positive
    stands; re-evidence the swap in a two-chat lab fixture)
  - fresh-profile first-run modal keyboard swallowing: the "Introducing
    GPT-5.6-Sol" promo modal swallows all keyboard input on pristine boots
    (seen-flag does not carry across version bumps); Escape is the verified
    dismissal — make first-run keyboard flow honest (auto-focus dismissable
    affordance / documented one-Escape contract) and re-check at each bump
Non-goals: platform-bound a11y items (screen-reader labels/AT tree,
  OS-level reduced-motion — upstream-GPUI, revisited at F11); N2/N3/N4
  bounded notes.
Files/subsystems owned: TBD at dispatch (expected: crates/codex-app ui.rs
  focus paths + a lab scene)
Tests: focused unit tests at the fix sites
GUI/lab evidence: lab scenes (PTY-focus probe; swap re-evidence;
  fresh-profile promo capture) — the Lead runs them at the gate
UX journey IDs: J-01, J-03 (keyboard recovery), J-17
Acceptance criteria:
  1. N5 CLOSED: typed input lands in the PTY after open (lab probe).
  2. N6 CLOSED: swap chords positively evidenced in the two-chat fixture.
  3. First-run: keyboard path documented + verified at the current version
     (no silent swallowing without an escape hatch).
  4. Non-empty Contract deviations blocks closure.
Status: DEFINED — QUEUED (2026-09-21)
```

---

## Reporting contract (all F2 workers)

Every worker returns: work-order ID(s); changed contracts; changed
files/subsystems; acceptance criteria mapping; tests + exact results;
journey IDs exercised; GUI discoverability layers covered (Worker C);
evidence produced; known limitations; architecture deviations;
recommended follow-up work. A worker may not declare a phase complete —
only the Tech Lead closes a gate after merged implementation and evidence.
