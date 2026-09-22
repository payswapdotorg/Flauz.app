# Wave-2 Integration Gate Record — PASSED

> **Status: PASSED 2026-09-23 — WAVE 2 CLOSED by the Tech Lead.**
> Final main SHA: **e9bee81** (PR #46 merge a46b76f + PR #47 merge 93125d0
> + PR #48 merge f5fae68 + evidence/ledger docs 6368c2e + Wave-3 work
> orders e9bee81). Authority: the WAVE2-WORK-ORDERS ledger + each work
> order's GUI/lab-evidence clause; the wave is closed by the Lead only
> after merged implementation AND evidence — both in place below.
> The Wave-2 ledger: 3/3 MERGED, deviations NONE ×3 (MOD-001+RT-001
> PR #46, CAP-001 PR #47, ENV-001 PR #48).

## Gate A — the 17-step integration harness: GREEN

The Lead-authored harness (`lead-tools/f2-integration-harness`,
verification infrastructure NOT a workspace crate; the Wave-2 steps
archived here: `environment_fabric_step.rs`, `capability_resolver_step.rs`;
run log `gate-a-run-main-6368c2e.log`) wires the four merged contract
crates together through the frozen formats. Steps 1–14 + the envelope
seam are the F2 gate (unchanged, re-proven at the merged tree); the
Wave-2 additions:

- **[15] model fabric (MOD-001+RT-001, pre-authored at the merge)**:
  registry round-trip — ProviderConnection (SecretRef only) →
  ProviderRegistration (Connected) → duplicate rejected →
  ModelProvider view (honest availability + capabilities + models) →
  lookups → OCC (correct version → +1 exactly; stale →
  VersionConflict) → snapshot → canonical JSON (independent
  CREDENTIAL_MARKERS scan: no material; the flausec_ REFERENCE
  round-trips) → restore → snapshot equality; BOTH runtime adapters
  through the PUBLIC `AgentRuntime` contract: direct (EMPTY
  advertisement, honest) + codex (fake handle), turns Completed on
  each, the codex turn evidenced via served_turns(), capability-gap
  probes on BOTH (exactly the missing key, no server contact).
- **[16] environment fabric (ENV-001)**: LocalEnvironmentProvider
  (kind `local`, `Locality::Local` only, no connection) sources the
  frozen 12-key F1 topology (terminal/browser/computer-use/git); the
  trait-object seam verified; locality honesty (a REMOTE spec is
  refused); the live `LocalEnvironment` is stateless over its
  descriptor (`from_descriptor` reload = identity-exact); the
  FakeRemoteEnvironmentProvider (kind `flauz-fake-remote`,
  `Locality::Remote` only, `with_connection` REFERENCE) sources the
  8-key sandbox topology; a LOCAL spec is refused; the snapshot →
  serialize (independent CREDENTIAL_MARKERS scan) → drop → restore
  round-trip serves the SAME `env_` identity with descriptor equality;
  cross-environment discipline: a fresh sourcing allocates a fresh
  canonical identity (no hidden global state) while the same spec
  yields the same capability advertisement.
- **[17] capability resolver (CAP-001)**: the COMPLETE deterministic
  144-case fake input space — every case resolved + self-validated;
  available ⇒ zero gaps AND all-dimensions-admitted; unavailable ⇒
  NON-EMPTY named gap list (the no-silent-fall-through property),
  gaps/unlock paths mirroring one-for-one per dimension with non-empty
  user-language reasons/actions; the per-dimension admission/gap
  consistency check; canonical byte-stable JSON round-trips with the
  independent credential scan on every record; both outcomes
  exercised; determinism/replay byte-identical; the resolver-output
  law against the COMMITTED fixtures (typical: browser.input with 2
  named gaps — environment + policy; all-admitted: terminal, the
  success state; the public fakes equal their committed fixtures);
  caller-data re-validation (the grammar at parse; the canonical
  constructor rejects unsorted lists; the resolver rejects a
  hand-tampered record).

Result: `== F2 INTEGRATION HARNESS: ALL STEPS PASS ==` (17 steps + the
envelope↔transport seam, 446 wire bytes) at merged main.

## Gate B — GUI discovery evidence: GREEN (binary codexrs-w2gate-e9bee81)

Scene `d24-w2-surfaces.sh` (archived here: `gate-b-scene-d24.sh`; run
log `gate-b-run-d24.log`; frames at `parity-lab/evidence/d24/`; VLM
reads archived as `gate-b-vlm-*.json`) — cold start → anchor task →
the three Wave-2 UI surfaces + the environment-rail regression:

- **J-13 the model picker (MOD-001)**:
  - layer 1 (visible entry): the task surface carries the labeled
    control **"Model: not chosen yet"** (B02);
  - layer 4 (keyboard): Ctrl+Alt+Shift+M opens **"Choose a model"**
    (B03) — the honest EMPTY state: *"No models connected yet"* +
    what a model provides (*"reading your instructions, writing,
    analyzing, and planning. Different models bring different
    strengths; for example, some can understand images"*) + the
    connection next-step pointing at Settings → Connections (B03) +
    the identity promise in user language: *"Switching models keeps
    this task — its objective, its plan, and everything it produced —
    exactly as it is"*;
  - layer 3 (palette): the row **"Choose a model…"** filters + lands
    on the same panel (B05/B05b);
  - scoped Escape returns to the task surface — focus not trapped
    (B04).
- **J-04 the capability gap (CAP-001)**:
  - layer 1: the **"Capabilities"** affordance on the task surface
    (B02, the More/Inspect neighborhood);
  - layer 4: Ctrl+Alt+Shift+6 opens the panel (B06) — subtitle
    *"What a task can do — and what's missing when it can't"*; the
    honest pre-wiring state promises the five named dimensions in
    user language: *"naming exactly what is missing: the model, the
    runtime, the environment, permissions, or workspace policy. No
    capability quietly disappears."* + the what-to-do-next copy;
  - layer 3: the palette row **"Why is a capability unavailable?"**
    filters + lands on the SAME panel (B09/B09b — frame-identical to
    the chord-open panel);
  - scoped Escape closes (B08).
- **J-05/J-06 the environment rail (ENV-001 regression)**: the
  Environments rail renders its honest empty state unchanged at the
  merged binary (B10): *"No environments attached"* + *"Environments
  are where work actually happens — browser sites, terminals,
  sandboxes, and desktops"* + the today-next-step (*"Use the Browser
  and Terminal panels in this chat"*) — the Wave-2 provider fabric is
  contracts-only (no UI change), and the rail regression confirms it.

Copy contract: user language throughout — "Choose a model…", "No
models connected yet", "Capabilities", "Why is a capability
unavailable?", never an internal type name.

## Gate C — CI: GREEN

The push-triggered CI (the webhook swallow had recovered by the final
pushes) ran the full gate on main: clippy `--workspace --all-targets
-D warnings` + test `--workspace` + release build + startup smoke,
both platforms. The authoritative full run on the code-final head
**19d16d5** (code-identical to e9bee81/f5fae68 — the later pushes are
docs-only; each earlier in-flight run was auto-superseded by the
concurrency group as the docs landed): **ubuntu-24.04 SUCCESS +
windows-latest SUCCESS** (run 35793521877). The per-PR dispatched runs
had already proven each merge head green (02d8473, 9839c09 — the
workflow_dispatch workaround during the event swallow).

## The wave record

- 3/3 work orders merged, deviations NONE ×3, evidence:
  - MOD-001+RT-001: `../w2-mod-001-rt-001/MOD-001-RT-001-COMPLETION-REPORT.md`
  - CAP-001: `../w2-cap-001/CAP-001-COMPLETION-REPORT.md`
  - ENV-001: `../w2-env-001/ENV-001-COMPLETION-REPORT.md`
- The ledger: `../../WAVE2-WORK-ORDERS.md` (header: 3 of 3 MERGED).
- The follow-ups carried into Wave 3: the picker's F7 wiring seams
  (set_catalog/active_model_id/events/availability — the BYOP slice
  wires them); the gap surface's view-model wiring (live
  CapabilityResolution records — an owned-files order); the
  environment rail's live wiring (F5+); all recorded in the
  respective post-merge notes.
- Wave 3 (F6 orchestration: ORCH-002/ORCH-003/ORCH-004) is dispatched
  per `../../WAVE3-WORK-ORDERS.md` — the frozen addendum + the wave
  integration gate spec (the F6 domain-neutral E2E scenario).
