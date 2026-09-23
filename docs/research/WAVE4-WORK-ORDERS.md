# Wave 4 Work Orders — F7 BYOP / F8 Labs / F9 Collaboration (the parallel wave)

> **Status: 3 of 3 MERGED — Wave 4 DELIVERED 2026-09-23** (LAB-001 PR #52
> `a134cec`; PROV-001 PR #53 `250109d`; COL-001 PR #54 `526d53c` — the
> ui.rs three-way resolved as the additive union, palette ALL 89/89).
> Evidence: [w4-lab-001](evidence/w4-lab-001/LAB-001-COMPLETION-REPORT.md) ·
> [w4-prov-001](evidence/w4-prov-001/PROV-001-COMPLETION-REPORT.md) ·
> [w4-col-001](evidence/w4-col-001/COL-001-COMPLETION-REPORT.md).
> The wave integration gate (the three-fabric scenario + the lab scenes
> at the merged binary) closes F7/F8/F9 — the record lands at
> [w4-gate](evidence/w4-gate/WAVE4-GATE-RECORD.md). Workers were
> dispatched 2026-09-23 (PROV-001 Worker A, LAB-001 Worker B, COL-001
> Worker C — all base = the Wave-3 gate-record head; the dispatch
> prompts pin the full SHA). Shared-contract authority:
> [F2-CONTRACT-KERNEL.md](F2-CONTRACT-KERNEL.md) (frozen) + the Wave-2
> addendum in [WAVE2-WORK-ORDERS.md](WAVE2-WORK-ORDERS.md) (frozen) + the
> Wave-3 addendum in [WAVE3-WORK-ORDERS.md](WAVE3-WORK-ORDERS.md) (frozen)
> + the **Wave-4 kernel addendum** below + the MERGED contract crates
> (flauz-world / flauz-exec / flauz-context / flauz-cap / flauz-orch).
> Every work order follows
> [WORK-ORDER-TEMPLATE.md](../WORK-ORDER-TEMPLATE.md). Non-empty
> `Contract deviations` in a worker report blocks closure. Workers deliver
> via `git bundle` on a single clean commit branch; the Lead gates, fixes,
> merges.
>
> **The three phase gates this wave serves** (IMPLEMENTATION-ROADMAP):
> **F7** — a user can consume their own provider quota without requiring a
> Flauz-owned global account (PROV-001 delivers the contracts + flows on
> fakes; real network integrations follow the lab-proven adapter pattern).
> **F8** — one journey runs unchanged against at least two providers
> (LAB-001 delivers the journey/evidence/comparator fabric with local +
> fake-remote adapters). **F9** — two independent clients can concurrently
> use one workspace/session without losing authorized state (COL-001
> delivers the contracts + a deterministic two-actor simulation + the
> members/presence surface; real multi-client transport arrives with the
> client adapters F10+).

## Wave-4 kernel addendum (frozen by the Tech Lead before dispatch)

1. **Existing signatures are frozen.** Wave-4 builds behind the merged
   Wave-1/2/3 contracts (`WorldStore` + its event stream, `AgentRuntime`,
   `ExecutionProvider`, `ModelProvider`, `ProviderConnection`/`SecretRef`,
   the capability resolver, the context engine, the harness, the
   orchestration graph). No trait signature changes. Additive methods
   require every existing fake conformance implementation to keep
   compiling — if an addition is unavoidable, mark it a deviation and
   justify.
2. **The quota-attribution law (F7).** Every scheduled execution that can
   consume a provider account's quota carries explicit, visible
   attribution: whose account (the connection label, user words), which
   provider, which tier (free/paid), and the policy that chose it.
   Ambient consumption — a task using "some account" invisibly — is
   forbidden. The routing policy is DATA (a record the user can see and
   change), and the scheduler's choice is recorded on the task's event
   stream (new event types follow the frozen-format discipline). When a
   free tier depletes and the policy moves to a paid tier, that
   escalation is NAMED to the user — no silent fall-through (the CAP-001
   law, applied to money).
3. **Credentials are references, forever (the Wave-1 §7 law, hardened).**
   Connection flows move credential MATERIAL only through the
   secret-store seam, which mints `flausec_` refs. No material in
   contracts, stores, fixtures, logs, telemetry, evidence, UI state, or
   worker reports. The fake flows use fake material, and the
   credential-material scan (the `CREDENTIAL_MARKERS` family) extends to
   cover the new record families. Real OAuth with real providers is OUT
   of scope this wave.
4. **Lab journeys are provider-independent by construction (F8).** A lab
   journey is a declarative spec (data) plus a normalized evidence
   schema; the same journey bytes run unchanged against every compliant
   lab adapter. The comparator compares candidate vs reference through
   the NORMALIZED schema — never raw screenshots alone; accessibility
   evidence is first-class, and the F1 a11y lessons (PTY focus transfer,
   bracket swap, modal traps, first-run keyboard swallowing) ride the
   schema as a named-findings registry. The local Linux lab is the
   reference provider; a fake-remote lab adapter is the consistency
   provider (the ENV-001 law: the fake remote every real remote copies).
   Real provider adapters (E2B/Daytona/Azure/GitHub/Codemagic) are
   skeleton contracts + fakes this wave — real network integrations are
   gated behind the lab proving the contract first.
5. **Collaboration extends the existing attention model — never a second
   event store (F9).** Membership/permission/presence/worktree-policy
   records land in the EXISTING world store and event stream (new event
   types under the frozen-format discipline). The attention model
   (unread/needs-attention, the Activity view) is EXTENDED with
   collaborator attribution — no parallel activity feed, no second event
   store, no second presence channel. Multi-client is SIMULATED this
   wave: a deterministic two-actor fake (two ActorRefs driving the same
   store APIs, interleaved) proves the concurrency semantics —
   permissions hold under interleaving, private context stays private,
   authorized state is never lost.
6. **Private vs shared context is explicit (F9, the Session≠Context law
   extended).** What a collaborator sees is a permission-filtered
   projection of the SAME durable task state (the projection law);
   sharing is never transcript-dumping; member-private memory items stay
   private. Visibility is carried as data on the new record families —
   mutating frozen record shapes requires the version discipline
   (additive/optional or a deviation).
7. **GUI slices follow the seven-layer rule; user language only.**
   Money/quota language is plain ("Using your OpenAI free tier — 3 of 5
   runs left today", never "consuming tokens from flausec_…"). Presence
   language is plain ("Dev is viewing this task", never "presence row for
   actor ref"). Sharing language states consequences ("Shared files —
   everyone on this task can read and write them"). The palette is never
   the only discovery mechanism.

---

## PROV-001 — BYOP connections, quota attribution, free-tier-first routing

> **Status: MERGED 2026-09-23** — worker commit `acb722b` + Lead gate-fixes
> `6c265a2` (first-real-compile corrections — the worker's sandbox had no
> Rust toolchain) + `765f66f` (CI round-2 UI-test corrections), merged via
> PR #53 (`250109d`, CI green both platforms); deviations NONE; Lead gates
> 37/37 + fmt/clippy clean + palette 87/87 + the chord listener
> seam-tested; evidence:
> [w4-prov-001](evidence/w4-prov-001/PROV-001-COMPLETION-REPORT.md).

```
ID: PROV-001
Title: User-owned provider accounts — the connection flow on fakes, the
  quota ledger with visible attribution, the routing policy as data with
  free-tier-first scheduling, and spend/concurrency limits
Phase: Wave 4 (F7 — user-owned providers / free-tier routing)
Owner: Worker A (agents-tab session)
Dependencies: flauz-exec merged (ProviderConnection + SecretRef, the
  registry with OCC + SecretRef-only connections — the MOD-001 surface);
  flauz-world (the event stream + frozen-format discipline); the model
  picker surface (flauz_model_picker.rs) and the shell-family patterns
Contract(s): F2-CONTRACT-KERNEL.md; Wave-2 addendum; Wave-3 addendum;
  this addendum §1-§3, §7
Problem: users cannot connect their own provider accounts; nothing models
  per-account quota, tier state, or spend; no routing policy exists, so
  any future execution would consume "some account" invisibly — the
  ambient-consumption anti-pattern this wave forbids.
User-visible outcome: a user connects a provider account (the API-key
  flow on fakes: key entry → the secret-store seam mints a flausec_ ref →
  the account appears with its tier + quota state), sees whose account a
  task will use and why ("Using your OpenAI free tier — 3 of 5 runs left
  today"), sees and changes the routing policy, and watches free-tier-
  first scheduling name its choice — including the honest escalation when
  a free tier depletes (J-14's account dimension).
Scope:
  - crates/flauz-prov/** (NEW self-contained crate, the flauz-cap
    pattern: serde-only, inputs as data, no contract-crate imports):
    - account.rs: ProviderAccount (the connection-ref link, the account
      label in user words, provider kind, tier kind free/paid, quota
      window state as DATA snapshots — remaining/limit/window-bounds —
      never live counters), multi-account per provider.
    - ledger.rs: QuotaLedger — per-account usage records (which logical
      consumption, when, how much, attributed to which task) + the
      depletion projection (given a window state + a projected
      consumption → the honest remaining state).
    - policy.rs: RoutingPolicy (data): the ordering (free-tier-first
      default), per-provider/per-task overrides, spend limits (per
      account + workspace), concurrency/session limits.
    - scheduler.rs: given a routing need (the requirement record as
      data: provider kind + capability needs) + accounts + policy → a
      SchedulingChoice naming the chosen account, the tier, the policy
      rule that chose it, and the named alternative when the preferred
      account is depleted/rate-limited/over-limit (no silent
      fall-through — the CAP-001 law applied to money); every choice
      serializes canonically.
    - fakes.rs: deterministic fake provider backends (quota windows that
      deplete, rate limits that trip, tiers that escalate) + public
    fixtures per family (typical/minimal/invalid) + conformance tests.
  - crates/codex-app/src/ui/flauz_providers.rs (NEW, the shell-family
    pattern): the provider accounts surface — the connect flow (API-key
    entry → honest "the key is stored securely; Flauz never shows it
    again" state), the accounts list (whose account, provider, tier,
    quota state in user words), the routing-policy surface (see + change,
    with the consequence stated), spend/concurrency limits display, and
    the task-surface attribution affordance (which account this task uses
    / will use and why). Honest empty state (no accounts → what
    connecting adds + the next step); keyboard path; scoped Escape.
  - ui.rs: ONLY the named seams (module + palette rows + chord +
    registrations) — distinct from every prior wave's seams.
Non-goals: NO real OAuth/network flows (fake backends prove the flows;
  real BYOP integrations follow the lab-proven adapter pattern in a later
  wave), NO billing/payment, NO trait signature changes, NO secret-store
  internals (the seam mints refs; the real keychain lands with F13
  hardening).
Files/subsystems owned: crates/flauz-prov/**; the workspace Cargo.toml
  members line (one line); crates/codex-app/src/ui/flauz_providers.rs;
  ui.rs NAMED SEAMS ONLY
Inputs: flauz-exec's ProviderConnection/SecretRef + registry (the frozen
  surface this builds on); flauz-cap's named-gap pattern; the MOD-001
  model-picker surface (the account dimension extends it conceptually but
  touches no MOD-001 files)
Outputs/artifacts: the flauz-prov crate + the providers surface + tests
  + fixtures
Tests: the attribution law (every SchedulingChoice names account + tier +
  policy rule; a choice with ambient attribution cannot be constructed);
  free-tier-first ordering incl. the honest escalation; limit enforcement
  (spend + concurrency, per account + workspace); depletion honesty (the
  projected remaining state never goes silently negative); multi-account
  ordering determinism; canonical JSON + fixtures; the
  credential-material scan extended to the new families (fixtures and
  fakes contain no material; fake material resembling credentials is
  REJECTED by SecretRef); the UI module tests (copy rules — user words
  only, tested; seven layers; keyboard chord; honest empty state)
GUI/lab evidence: Lead gate — lab scenes at the merged binary: the
  connect flow (fake key → account appears with tier + quota), the
  accounts list, the routing policy surface, the task-surface attribution
  affordance, the palette rows
UX journey IDs: J-14 (switch model without losing work — the account/
  tier attribution dimension), J-17 (activity — scheduling choices are
  visible activity), plus the new connection surface under the
  seven-layer rule
Primary discovery surface: the provider accounts surface (Workspace-
  level) + the task-surface attribution affordance
Contextual discovery surface: the depletion moment ("Your OpenAI free
  tier is used up today — next run uses your paid account. Change this")
Search/palette discovery: palette rows ("Connect a provider account",
  "See which account a task uses")
Empty/success-state behavior: no accounts → what connecting adds + the
  next step; connected → the account with live quota words; depleted →
  the honest escalation + the policy link
Keyboard path: a letter-family chord (Ctrl+Alt+Shift+P suggested —
  verify no conflict at implementation; the registration seams follow
  ORCH-003/004) + scoped Escape; tab-navigable flow
Acceptance criteria:
  1. Single clean commit on feat/prov-001-byop at the pinned Wave-4 base;
     owned files only.
  2. The attribution law proven: no SchedulingChoice without named
     account + tier + policy rule; escalation named, never silent.
  3. The connect flow works end-to-end on fakes with SecretRef-only
     storage; the credential scan passes on every new family.
  4. The surface: seven layers; user language only (tested); keyboard
     path with the on_action listener REGISTERED AND SEAM-TESTED (the
     d25 Gate-B lesson: a KeyBinding without a listener dispatches into
     the void — the seam test must pin the listener, not just the
     binding).
  5. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive crate + modules + seams; revert the commit.
Integration notes: the scheduler takes requirement records as DATA (no
  flauz-exec imports — the flauz-cap law); the task event-stream record
  of a scheduling choice uses NEW event types through the world-store
  seam (frozen-format discipline; do not widen existing types). The
  model picker surface is NOT modified this wave; the attribution
  affordance is a separate task-surface element.
Status: DISPATCHED 2026-09-23
```

---

## LAB-001 — The provider-neutral lab fabric: journey specs, normalized evidence, the comparator

> **Status: MERGED 2026-09-23** — commit `a6d37f5` + Lead lockfile gate-fix
> `b8b8223`, merged via PR #52 (CI green both platforms, run 35848069053);
> deviations NONE; Lead gates 60/60 + fmt/clippy clean; evidence:
> [w4-lab-001](evidence/w4-lab-001/LAB-001-COMPLETION-REPORT.md).

```
ID: LAB-001
Title: The provider-neutral parity-lab fabric — declarative journey
  specs, the normalized evidence schema (screenshots/actions/
  accessibility/known-findings), the reference-vs-candidate comparator,
  and the lab-adapter contract with local + fake-remote providers
Phase: Wave 4 (F8 — provider-neutral parity lab)
Owner: Worker B (agents-tab session)
Dependencies: the Lead's local Linux lab evidence patterns
  (docs/research/evidence/codex-linux + parity-matrix vocabulary); the
  F1 a11y findings registry (d17/d19/d21/d23 lessons —
  docs/research/evidence/f1-sweep); ENV-001's fake-consistency law; the
  frozen event/serialization discipline
Contract(s): F2-CONTRACT-KERNEL.md; Wave-2 addendum; Wave-3 addendum;
  this addendum §1, §4, §6
Problem: every lab run today is bespoke (hand-written scene scripts +
  one-off VLM reads); evidence is not normalized across providers, there
  is no comparator, and adding a provider (Windows/Azure/macOS/
  Codemagic/E2B/Daytona) would mean re-authoring journeys per provider —
  the exact duplication this wave forbids.
User-visible outcome: the OPERATOR (the Lead; provider teams later) can
  define a journey once and run it unchanged against any compliant lab
  adapter; evidence comes back normalized (schema'd, canonical-JSON,
  comparable), the comparator diffs a candidate against a reference with
  named divergences, and the F1 a11y lessons ride the schema as a
  named-findings registry that every future run can reference. The F8
  gate ("one journey runs unchanged against at least two providers") is
  proven by the local + fake-remote adapters this wave.
Scope:
  - crates/flauz-lab/** (NEW self-contained crate, serde-only, inputs as
    data, no contract-crate imports):
    - journey.rs: JourneySpec — the declarative journey (steps: UI
      actions as data (key chord, click at a named anchor, type text),
      probes (frame capture at a named moment, state assertions), the
      surfaces it exercises, the journeys it maps to (J-ids as data));
      deterministic ordering; canonical JSON.
    - evidence.rs: the normalized EvidenceRecord schema — versioned;
      frame references (not pixels inline), action traces, accessibility
      probe results (focus order, trap probes, chord ladders — the F1
      patterns as data), VLM-read slots (the adjudication prompt + the
      read result reference), environment/provider metadata, and the
      known-findings links.
    - findings.rs: the named-findings registry — the F1 a11y lessons
      (PTY focus transfer, bracket swap, modal traps, first-run keyboard
      swallowing, the N6 shifted-symbol family) as data: id, surface,
      description, the regression-guard status; every record references
      findings by id, never by prose.
    - comparator.rs: reference-vs-candidate diff through the NORMALIZED
      schema — matching/divergent/missing evidence per journey step,
      severity per divergence, canonical diff reports; deterministic.
    - adapter.rs: the LabAdapter contract (a trait): prepare(journey) →
      run(step) → capture(probe) → evidence; plus LocalLabAdapter (the
      in-repo reference implementation driver: the contract + a
      deterministic fake that stands in for the Lead's script-driven
      local lab) and FakeRemoteLabAdapter (the consistency provider —
      the fake remote every real remote copies: E2B/Daytona/Azure/
      GitHub/Codemagic skeletons as adapter-family DATA with their
      topology metadata; no network).
    - fakes.rs + fixtures: two deterministic journeys (a shell-discovery
      journey + an a11y chord-ladder journey — recognizable shapes from
      d23/d25) with full evidence families for both adapters.
  - No UI this wave (the lab's user is the operator; the evidence +
    comparator outputs are the product). Provider STATUS for end users
    rides PROV-001's accounts surface — a note, not a dependency.
Non-goals: NO real network integrations (E2B/Daytona/Azure/GitHub/
  Codemagic are adapter-family DATA + fakes; real adapters follow once
  the contract is lab-proven), NO CI-provider YAML generation, NO UI, NO
  VLM calls inside the crate (VLM reads are evidence SLOTS — the schema
  carries the prompt + result reference; the adjudication stays
  operator-side), NO trait changes to existing crates.
Files/subsystems owned: crates/flauz-lab/**; the workspace Cargo.toml
  members line (one line; coordinate with PROV-001 via the Lead — the
  one-line conflict is resolved keep-both at merge, the ENV-001
  precedent)
Inputs: the Lead's scene patterns (d23/d24/d25 — the journey steps are
  recognizable re-exports of those shapes as data); the F1 a11y
  findings; parity-matrix.md vocabulary
Outputs/artifacts: the flauz-lab crate + fixtures + conformance tests
Tests: the same JourneySpec bytes produce schema-comparable evidence
  from BOTH adapters (the F8 gate law); the comparator: identical runs →
  zero divergences; a seeded divergence → named, severe, located;
  findings referenced by id resolve; canonical JSON round-trips; no
  credential material; determinism (same inputs → byte-stable reports)
GUI/lab evidence: Lead gate — the wave harness step: define a journey →
  run against local + fake-remote adapters → normalized evidence →
  reference vs candidate comparator report with a seeded divergence
UX journey IDs: n/a (operator fabric; the evidence schema feeds the
  J-battery at every later gate)
Primary discovery surface: n/a (no end-user surface this wave)
Contextual discovery surface: n/a
Search/palette discovery: n/a
Empty/success-state behavior: n/a (the comparator's empty state is a
  zero-divergence report — tested)
Keyboard path: n/a
Acceptance criteria:
  1. Single clean commit on feat/lab-001-fabric at the pinned Wave-4
     base; owned files only.
  2. The F8 gate law proven in-tests: one journey, two adapters,
     comparable normalized evidence.
  3. The comparator catches a seeded divergence and names it; identical
     runs report zero.
  4. The findings registry carries the F1 a11y lessons as data; records
     reference by id.
  5. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive crate; revert the commit.
Integration notes: serde-only, inputs as data (the flauz-cap law); the
  adapter trait is NEW surface owned by this crate (not an
  ExecutionProvider extension); the Lead's wave-gate harness gains a
  lab step driving both adapters after merge.
Status: DISPATCHED 2026-09-23
```

---

## COL-001 — Collaboration contracts + the members/presence surface

> **Status: MERGED 2026-09-23** — worker commit `d9f3563` + Lead gate-fixes
> `77ca824` (lockfile) + `306d9b1` (CI dead-code + two missing renders),
> merged via PR #54 (`526d53c`, CI green both platforms); deviations NONE
> (the M→U chord change is documented necessity — M is owned by the model
> picker); worker gates 51/51 real runs; Lead merge gates green + palette
> ALL resolved 87+2=89/89; evidence:
> [w4-col-001](evidence/w4-col-001/COL-001-COMPLETION-REPORT.md).

```
ID: COL-001
Title: Workspace collaboration — membership/roles/permissions with named
  denial, presence records, private-vs-shared context visibility, the
  worktree/shared-filesystem policy, the deterministic two-actor
  simulation, and the members/presence surface (J-13)
Phase: Wave 4 (F9 — collaboration)
Owner: Worker C (agents-tab session)
Dependencies: flauz-world (WorldStore + the event stream + ActorRef +
  the frozen-format discipline); the attention model (unread/
  needs-attention + the Activity view — WO-P2-008/P2-013 surfaces); the
  shell-family patterns; flauz-context's projection law (the
  permission-filtered projection extends it conceptually, touching no
  flauz-context files)
Contract(s): F2-CONTRACT-KERNEL.md; Wave-2 addendum; Wave-3 addendum;
  this addendum §1, §5-§7
Problem: a workspace has exactly one implicit actor; nothing models
  members, roles, permissions, or presence; private-vs-shared context is
  undefined; sharing a task would mean transcript-dumping or ambient
  filesystem sharing — both forbidden.
User-visible outcome: a workspace shows its members with roles and
  presence ("Dev is viewing this task"); a task states its sharing
  posture in user words ("Isolated copy — your files stay separate" /
  "Shared files — everyone on this task can read and write them"; "Only
  me" / "The workspace" for remembered notes); a member who lacks a
  permission sees the honest unavailable state with the WHY (the CAP-001
  pattern); the two-actor simulation proves concurrency semantics hold
  (the F9 gate law) before any real transport exists.
Scope:
  - crates/flauz-collab/** (NEW self-contained crate, serde-only,
    inputs as data, no contract-crate imports):
    - membership.rs: WorkspaceMembership (member identity ref, display
      name, role: owner/admin/contributor/viewer) + the role lattice as
      DATA (what each role may do per surface family: tasks, provider
      accounts, environments, members).
    - permission.rs: PermissionGrant (explicit allow/deny per surface,
      overriding the role default) + the authorization evaluator:
      inputs as data (actor + action + surface + grants + lattice) →
      Allow/Deny with the NAMED reason (which rule denied; never a
      silent no-op — the CAP-001 law applied to access).
    - presence.rs: PresenceRecord (member ref + surface + freshness
      bounds as data) — bounded, replaceable, never an event stream of
      its own (the §5 law).
    - policy.rs: WorktreePolicy (isolated-by-default; the explicit
      shared-filesystem mode per task: off/read/write as a record) +
      ContextVisibility (member-private vs workspace-shared memory/
      artifacts — data on the new families; the projection filter the
      caller applies).
    - simulator.rs: the deterministic two-actor simulation — two
      ActorRefs driving the same store-facing seam with INTERLEAVED
      operations (a scripted interleaving table): permissions hold
      under interleaving, private records stay private to their actor,
      authorized state is never lost (every legal operation either
      lands or is denied-with-reason; no lost-update on the simulated
      seam), and the log of the interleaving is canonical-JSON
      replayable.
    - fakes.rs + fixtures per family + conformance tests.
  - crates/codex-app/src/ui/flauz_members.rs (NEW, the shell-family
    pattern): the members + presence panel (Workspace-level surface):
    the member list (name, role, presence in user words), the
    invite/role affordances with honest not-wired states (the real
    transport is F10+ — say so plainly), and the per-task sharing
    surface (worktree posture + shared-filesystem mode + context
    visibility in user words with consequences). Permission-gated
    affordances render the honest unavailable state with the WHY.
  - The attention-model EXTENSION seams (additive, in the EXISTING
    modules): collaborator attribution on attention/activity rows
    ("Ana needs your review" style) — additive data + additive render
    lines only; NO second event store, NO parallel feed (the §5 law;
    the seam test pins it).
  - ui.rs: ONLY the named seams — distinct from every prior wave's.
Non-goals: NO real multi-client networking/transport (F10+), NO
  CRDT/real-time sync engine, NO collaborative editors for documents/
  sheets/code (the contracts make room; nothing ships), NO trait
  changes, NO flauz-context/flauz-world file modifications (the
  projection filter + the event records go through the existing seams
  as data).
Files/subsystems owned: crates/flauz-collab/**; the workspace Cargo.toml
  members line (one line; the Lead resolves the three-way one-liner at
  merge — keep-both/union, the ENV-001 precedent); crates/codex-app/src/
  ui/flauz_members.rs; the EXISTING attention/activity modules'
  additive attribution seams (named files listed in the report); ui.rs
  NAMED SEAMS ONLY
Inputs: flauz-world's ActorRef + event discipline; the WO-P2-008/
  P2-013 attention + Activity surfaces; PRODUCT-UX-JOURNEYS J-13/J-17;
  the CAP-001 named-gap pattern
Outputs/artifacts: the flauz-collab crate + the members surface + the
  attention attribution seams + tests + fixtures
Tests: the authorization evaluator (role defaults, explicit grants,
  named denial — no silent no-ops); the lattice (each role's surface
  map); presence bounds; the two-actor interleavings (a table of
  interleaved operation pairs with expected outcomes — permissions
  hold, private stays private, no authorized state lost; canonical
  replay); the visibility filter (a member-private record never
  appears in another actor's projection); canonical JSON + fixtures;
  the UI module tests (copy rules — user words + consequences stated,
  tested; seven layers; the not-wired honesty; keyboard chord with the
  on_action listener REGISTERED AND SEAM-TESTED — the d25 Gate-B
  lesson); the attention-extension seam test (attribution additive; no
  second store — source-inspected, the house pattern)
GUI/lab evidence: Lead gate — lab scenes at the merged binary: the
  members panel (roles + presence words + not-wired honesty), the
  per-task sharing surface, a permission-denied affordance with the
  WHY, the palette rows
UX journey IDs: J-13 (collaborate on one task — the surface slice this
  wave; joining/presence/private-context/sharing posture visible), J-17
  (activity with collaborator attribution)
Primary discovery surface: the members + presence panel
  (Workspace-level) + the per-task sharing surface
Contextual discovery surface: permission-gated affordances state WHY
  they are unavailable (the CAP-001 pattern)
Search/palette discovery: palette rows ("See who is on this
  workspace", "Change a task's sharing")
Empty/success-state behavior: solo workspace → the honest "just you"
  state + what inviting adds; with members → the list + presence;
  sharing posture states consequences; not-wired flows say so plainly
Keyboard path: a letter-family chord (Ctrl+Alt+Shift+M suggested —
  verify no conflict at implementation) + scoped Escape; tab-navigable
Acceptance criteria:
  1. Single clean commit on feat/col-001-collab at the pinned Wave-4
     base; owned files only.
  2. The F9 gate law proven in simulation: two actors, interleaved, no
     authorized state lost; denials named; privacy holds.
  3. The surface: seven layers; user words with consequences (tested);
     not-wired honesty; keyboard path with the listener seam-tested
     (the d25 lesson).
  4. The attention extension is additive-only and seam-tested (no
     second event store).
  5. Non-empty Contract deviations blocks closure.
Rollback/recovery: additive crate + modules + seams; revert the commit.
Integration notes: the evaluator takes grants/lattice/presence as DATA
  (no flauz-world imports); the world-store event records for
  membership/presence changes use NEW event types through the existing
  seam (frozen-format discipline). The client adapters (F10+) later
  drive the same seams the simulator drives — keep the seam shapes
  adapter-neutral.
Status: DISPATCHED 2026-09-23
```

---

## Reporting contract (all Wave-4 workers)

The 11-field completion report (exact headers), delivered in the worker's
final message AND via the delivery bundle branch: WO ID(s); base branch +
SHA; branch/commits; changed files/surfaces; implementation summary;
tests/commands + exact results (state plainly if the sandbox lacks the
Rust toolchain — static reasoning is acceptable, the Lead independently
compiles and gates); kernel-compliance checklist (this addendum's §1-§7,
each item); GUI discoverability layers covered (UI-bearing WOs);
acceptance-criteria evidence (map each bullet); known limitations;
contract deviations (NONE if none); follow-up work.

## Wave-4 integration gate (Lead, after all three merge)

Extend the F2/Wave-2/3 harness (lead-tools/f2-integration-harness): the
three-fabric scenario — a task whose model need routes through the
PROV-001 scheduler (two fake accounts, free depleted mid-run → the named
escalation on the event stream), executed in the LAB-001 journey shape
(the same journey spec against local + fake-remote lab adapters, the
comparator reporting zero divergences on identical runs and a named
severe divergence on a seeded one), with COL-001's second actor present
(the two-actor interleaving: a member-private memory item never in the
other's projection; a permission denial named; the shared-filesystem
mode stated). Plus the lab scenes (the providers surface, the members
surface, a denial-with-why, the routing-policy moment) at the merged
binary. The record lands at evidence/w4-gate/WAVE4-GATE-RECORD.md; F7/
F8/F9 close only then.
