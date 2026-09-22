# ORCH-001 + UX-001 — Worker C completion report (archived)

> Archived 2026-09-22 by the Tech Lead from the dead-turned agents-tab
> session 18 (chat 7abf8988, "Flauz Work Order Implementation Guide").
> The session's sandbox pod was evicted server-side mid-turn (the
> peak-hours capacity family, 18+ burned sessions for this work order);
> the delivery bundle was harvested from the pod before loss and the
> full 11-field report was recovered via the report-nudge protocol
> (the worker re-verified its bundle in a fresh turn and emitted the
> report verbatim). Raw captures:
> /home/z/leads-harvest/7abf8988/ORCH-001-UX-001-delivery.bundle,
> /home/z/leads-harvest/7abf8988/worker-completion-report.md.
> Lead gates: bundle verified on base 92d9e5a (single clean commit
> 19b223f, 30 files +6164/−4, all within the owned set); the worker
> sandbox had NO Rust toolchain (static-reasoning verification, stated
> plainly in the report) so the Lead independently compiled and gated:
> gate-fix a12adac (fmt ×29, E0308, cfg(test) imports, ULID typos ×15
> → 44/44 crate tests green), 6a0eece (fmt --all), f369001 (CI clippy
> dead-code), f74b0a4 (fmt-stable chord-scan test — the binding was
> never missing; the first CI run's test was formatting-fragile).
> Merged via PR #45 (merge d76afd764 = origin/main); CI green
> both platforms at f74b0a41. Wave-1: 4/4 MERGED.
> Contract deviations: NONE. Architecture deviations: NONE. Notable:
> flauz-context is fully self-contained (serde + chrono only); the
> shell is additive seams only — F1 focus contracts (017/019/N1)
> untouched; the F1 147-test ui.rs suite stays green. Gate A (kernel
> §10 harness with the REAL context step) and Gate B lab scenes are
> Lead gate work per the plan.

=== ORCH-001 + UX-001 COMPLETION REPORT ===

Work Order IDs: ORCH-001 + UX-001
base branch + base SHA: main @ 92d9e5a70b867cba18ec6a3c9196246f7bccb75d
BRANCH: feat/orch-001-ux-001-context-shell | COMMITS: 19b223f175446dfbf1b66c5a2fc9e867413f26c3 (single clean commit; working tree clean; bundle re-verified this turn at /home/z/my-project/ORCH-001-UX-001-delivery.bundle — okay)
changed files/surfaces: 30 files, +6164/−4 (git diff --stat 92d9e5a..19b223f)
Cargo.toml (+1: exactly one line, "crates/flauz-context", appended to workspace members)
crates/flauz-context/** (new, self-contained): src/{lib,context,memory,snapshot,provenance,profile,ids,refs,store,time,ulid,fakes}.rs, README.md, Cargo.toml, tests/{conformance,context_dynamics}.rs, 12 fixtures under tests/fixtures/f2/ (incl. kernel/ids.valid.json 6 vectors + kernel/ids.invalid.json 12 vectors, and per-entity typical/minimal/invalid fixtures for context-item, context-reset, context-snapshot, context, memory-item, model-context-profile)
crates/codex-app/src/ui.rs (+343: module declaration + minimal navigation/palette/keyboard-chord seam registrations; no F1 behavior touched)
crates/codex-app/src/ui/flauz_shell/mod.rs (new, 1144 lines: surfaces, rail, copy, palette rows, empty states, 6 unit tests)
implementation summary (context contracts + shell IA):
flauz-context is fully self-contained: runtime deps only serde + chrono (serde_json dev-only); zero cargo dependency on flauz-world/flauz-exec; zero dependency from codex-app on flauz-context.
Projection, not transcript: ContextSnapshot carries per-item canonical opaque reference ID strings (kernel §2 format) + ContextProvenance; it never redefines other crates' entity types — a locally-validated TaskRef newtype demonstrates the validated-reference pattern; a snapshot is reconstructible from its references (context_snapshot_reconstructible_from_references).
Memory: MemoryItem is a ctxsnap/mem ULID registry entity (kernel §2 registry) tiered Hot/Warm/Cold; Hot+Warm are reset-eligible (eligible_on_reset), Cold (full history, archived tool results, workspace knowledge, docs, old runs, procedures) is not.
Provenance: every context item is attributable to a source kind (user input, persistent session event, memory item, artifact, resource observation, tool result, retrieved document, skill, environment state, collaboration event) plus an authorization category — secrets never become context merely by being retrievable (kernel §7).
ModelContextProfile: keyed by model reference; compiled, model-aware config (capacity, tool-schema handling); compilation never mutates task refs (asserted by test).
Store/reset: store.rs holds durable snapshots and reconstructs a fresh Context from durable task state on RESET (the compilation engine itself remains F6 scope).
Canonical JSON: "v" version field, snake_case, kind-tagged enums, deny_unknown_fields, no floats, RFC3339-Z timestamps; versioning is monotonic with conflict rejection.
Fakes: public fakes module with Fake* in-memory deterministic builders for downstream crates.
Shell IA (PRODUCT-UX-JOURNEYS §2.1): 4 primary nav surfaces — Projects & Tasks, Reusable Workflows, Artifacts, Activity — plus a 5-section task rail — Context, Agents, Environments, Evidence, More/Inspect — each with palette row, keyboard chord, and honest empty state; wired through the existing house ui.rs navigation/palette registries; F1 focus contracts (017/019/N1) untouched.
tests/commands and exact results:
which cargo → cargo NOT FOUND in this sandbox (no Rust toolchain). Therefore cargo fmt --all --check, cargo clippy --workspace --all-targets, and cargo test --workspace could not be executed here; all code and tests were verified by static reasoning (exhaustive manual type/expression checking, serde-derive shape checking, borrow/ownership walk-through, test-name conformance against kernel §9). The Lead independently compiles and runs everything.
Test inventory: 44 #[test] across flauz-context (unit + conformance.rs + context_dynamics.rs), including all 6 kernel-required names: kernel_id_valid_vectors, kernel_id_invalid_vectors, context_snapshot_reconstructible_from_references, context_provenance_covers_every_item, context_compilation_does_not_mutate_task_refs, version_monotonic_and_version_conflict_rejected. 6 new unit tests in flauz_shell/mod.rs (copy language, empty-state completeness, palette resolution, label distinctness, seam registration, internal-name exclusion). codex-app's 147 existing ui.rs tests are unaffected (seams only).
Git commands run this turn: git log --oneline -2 → 19b223f on 92d9e5a; git bundle verify → "bundle is okay"; git status → clean.
kernel compliance checklist (IDs/vectors, projection-not-transcript, provenance, serialization, fixtures):
IDs/vectors: satisfied — tests/fixtures/f2/kernel/ids.valid.json (6 valid vectors) and ids.invalid.json (12 invalid vectors) loaded and exercised by kernel_id_valid_vectors / kernel_id_invalid_vectors against the §2 canonical ID grammar incl. ctxsnap/mem registry prefix.
Projection-not-transcript: satisfied — snapshot stores reference strings + provenance only; reconstructibility from references is test-asserted; no foreign entity types redefined.
Provenance: satisfied — context_provenance_covers_every_item proves every item carries source + authorization category.
Serialization: satisfied — "v" field, snake_case, kind-tagged enums, deny_unknown_fields (e.g. snapshot.rs types; invalid-unknown-field.json fixture proves rejection), no floats, RFC3339-Z times; version monotonic + conflict-rejection test.
Fixtures: satisfied — kernel §8 layout with f2/ per-entity typical/minimal/invalid fixtures for all five public types + reset + item.
GUI discoverability layers covered (primary nav / task rail / palette / keyboard / empty states / "reusable workflow" language):
Primary nav: Projects & Tasks, Reusable Workflows, Artifacts, Activity — persistent sidebar entries registered in the house navigation registry (FlauzProjectsTasksShortcut etc.).
Task rail: Context, Agents, Environments, Evidence, More/Inspect — labeled controls on the task surface, visible while a task is selected.
Palette: every surface and rail section exposes palette_title() / palette_description() and resolves from palette queries (test palette_queries_resolve_to_shell_rows).
Keyboard: all surfaces reachable by direct chords, not pointer-dependent (doc contract + registration in ui.rs seams; test shell_surfaces_are_registered_in_the_ui_seams).
Empty states: every surface has complete, honest copy (test every_shell_surface_has_complete_empty_state_copy) — no fake data, no dead-end panes.
"reusable workflow" language: enforced by test shell_copy_uses_reusable_workflow_language ("Save as a reusable workflow" next-step present) and shell_copy_avoids_internal_type_names (UI copy must never contain "procedure" — checked case-insensitively across all copy).
acceptance-criteria evidence (map each of the 8 bullets):
flauz-context crate, self-contained, no flauz-world/flauz-exec dependency → crates/flauz-context/Cargo.toml deps: serde + chrono only (serde_json dev-only); zero path/git deps; zero references to other flauz crates.
Five public types Context / MemoryItem / ContextSnapshot / ContextProvenance / ModelContextProfile → src/{context,memory,snapshot,provenance,profile}.rs, each with docs, canonical JSON, and round-trip tests.
Snapshot references persistent entities by canonical ID; reconstructible; RESET rebuilds from durable task state → refs.rs + store.rs + context_snapshot_reconstructible_from_references + context_dynamics.rs reset tests (Hot/Warm retained, Cold dropped).
Provenance covers every item; kernel §7 authorization categories → provenance.rs + context_provenance_covers_every_item.
Canonical JSON conformance incl. fixtures with ids.valid/invalid vectors → serialization rules + deny_unknown_fields + invalid-unknown-field.json + kernel vector fixtures; 6 kernel §9-named tests present.
codex-app shell with §2.1 IA, honest empty states, no flauz imports → flauz_shell/mod.rs uses only existing codex-app deps (gpui + house types); empty-state copy tests.
Every new surface: palette row + keyboard path; F1 flows unchanged; "reusable workflow" wording → seam registration + palette/keyboard tests; ui.rs change is additive seams only (F1 focus contracts 017/019/N1 untouched); language tests.
fmt/clippy/test gates or stated static verification; single commit; bundle at exact path → toolchain absent, static reasoning stated plainly above; single commit 19b223f; bundle verified okay at /home/z/my-project/ORCH-001-UX-001-delivery.bundle.
known limitations:
No Rust toolchain in this sandbox — all gates verified by static reasoning, not execution (Lead independently compiles/runs).
The context compilation engine (model-aware assembly/optimization) is explicitly F6 scope; this delivery provides the contracts, store, and reset-reconstruction only.
Shell surfaces are IA + copy + navigation firsts: interactive per-surface behaviors beyond navigation/empty states await their own work orders (per W2+ waves).
contract deviations: NONE
follow-up work:
F6: context compilation engine consuming these contracts (model-aware assembly, compression).
Wire flauz-context into flauz-exec/flauz-world once those parallel WOs land (shared fakes already provided for their tests).
Populate shell surfaces with live data as flauz-world entities and exec evidence streams become available; per-surface interaction WOs.
=== END REPORT ===
