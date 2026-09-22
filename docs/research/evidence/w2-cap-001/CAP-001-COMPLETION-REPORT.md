# CAP-001 — Completion Report & Lead Gate Record (Wave 2)

**Status: MERGED (PR #47, merge 93125d0)** — the capability resolver:
model ∩ runtime ∩ environment ∩ permissions ∩ workspace-policy with NAMED
gaps and the no-silent-fall-through property (exhaustive 144-case
enumeration), plus the gap UX surface (`flauz_capability_gap`, the J-04
Why?/What's-missing/Ways-to-unlock chain, 7 layers).

## Lead gate record

- **Delivery**: worker session 0cf3cf95 (agents tab, GLM-5.3, Full-Stack —
  session 14 of the peak-hours grind), branch
  `feat/cap-001-capability-resolver`, single clean commit `158fa0a` on base
  `fda38ea` (exact Wave-2 base). Bundle harvested from the worker pod
  (`leads-harvest/0cf3cf95`, 40,643 bytes) — `git bundle verify` PASS.
- **The grind**: CAP-001 burned ~9 create attempts + 1 capacity-swallowed
  phantom under the sustained GLM-5.3 peak-hours crunch (the supervisor's
  recover_capacity daemon cycling fresh creates; every attempt free) before
  the 20:05 gate-open landing; the continuation nudge then completed the
  remaining ladder 6/10 → delivery in one big turn.
- **Gate 0 (bundle)**: PASS — requires exactly `fda38ea`; one commit;
  25 files +3793/−2, all within the owned set.
- **Gate 1 (source review)**: PASS —
  - the resolver is a **pure function over inputs** (zero flauz-world /
    flauz-exec / flauz-context imports; foreign references cross the seam
    as frozen format strings with the grammar re-pinned in tests);
  - the five admission dimensions carry canonical names + the J-04
    user-facing vocabulary; **no-silent-fall-through proven by exhaustive
    deterministic enumeration** over the complete 144-case fake input
    space (not sampling); `CapabilityResolution::validate()` rejects the
    silent-fall-through record shape;
  - canonical JSON discipline (`"v":1`, snake_case,
    `deny_unknown_fields`, no floats, kind-tagged enums) + the
    resolver-output law tying every resolution fixture to the resolver run
    over its documented inputs fixture;
  - the resolver re-validates caller data (no trust in pre-validated
    inputs);
  - ui.rs changes are exactly the 16 named seams (each CAP-001-tagged).
- **Gate 2 (Lead-local)**: fmt clean; clippy `-p flauz-cap --all-targets`
  clean; `cargo test -p flauz-cap` **31/31** (20 lib + 6 conformance + 5
  resolver — matches the worker's claim exactly). Pre-empted the PR-46
  lesson: ran codex-app clippy `--all-targets` BEFORE pushing → found the
  same dead-code family on the gap surface's view-model API → **Lead
  gate-fix `8393012`** (`#[allow(dead_code)]` with per-item wiring-owner
  comments, the picker precedent). codex-app clippy: 0 errors. The local
  workspace-wide clippy attempt died on disk-full (the release-profile
  check artifacts; 563M reclaimed after) — CI owns the authoritative
  workspace run.
- **The CI event-swallow battle**: PR #47 created at head `8393012`; ZERO
  check-runs fired across 30+ min (no pull_request webhook events —
  close/reopen and empty-commit re-push all swallowed, while API-triggered
  reruns executed fine; githubstatus all-operational). **Workaround**:
  `workflow_dispatch` added to ci.yml (main `381e5c7`, cherry-picked to the
  branch as `02d8473`), CI dispatched on all three refs → **GREEN both
  platforms** at `02d8473` (ubuntu-24.04 + windows-latest).
- **Merge**: local `--no-ff` merge `93125d0` (ui.rs keep-both resolution —
  mod-001's picker seams + cap-001's gap seams, the `PaletteCommand::ALL`
  array 82) → push → GitHub auto-merged PR #47.
- **Contract deviations: NONE** (worker's report, confirmed by the Lead's
  review).

## Worker's COMPLETION REPORT (verbatim, as recovered)

> The full report was delivered in-thread at the 20:05 landing (recovered
> from the session's message batch store — the per-message containers
> retain the complete text). Fields condensed below where the work-log
> narrative duplicated them.

```text
=== CAP-001 COMPLETION REPORT ===

Work Order ID: CAP-001 — Capability resolution (model ∩ runtime ∩
environment ∩ permissions ∩ workspace policy) with the named-gap UX — no
silent fall-through (F2 Wave 2, Worker C)

Base branch + SHA: main @ fda38ea576db4d2cf05bb35ad962febb5bc94cb9
(recorded base of dispatch)

BRANCH|COMMITS: feat/cap-001-capability-resolver | exactly 1 clean commit:
158fa0a103bc28cda4966c61a31bcf95954c78f9 — 25 files changed, +3793/−2.
Not pushed (as ordered). Delivery bundle:
/home/z/my-project/CAP-001-delivery.bundle (created + git bundle verify OK).

Changed files/surfaces:
- crates/flauz-cap/** — NEW self-contained crate (Cargo.toml, README.md,
  src/{lib,dimension,key,inputs,resolution,resolver,fakes}.rs,
  tests/{conformance,resolver}.rs, tests/fixtures/w2/{resolution-inputs,
  capability-resolution}/*.json — 22 files)
- crates/codex-app/src/ui/flauz_capability_gap.rs — NEW gap surface
  (shell-family)
- crates/codex-app/src/ui.rs — NAMED SEAMS ONLY: 16 clearly-commented
  CAP-001 registration points (module declaration; palette enum variant
  InspectCapabilityGaps + ALL 80→81 + title/description/shortcut/icon/
  group/requires_selected_chat arms; dispatch arm; alt-shift-6 chord +
  alt-^ shifted-symbol companion + scoped Escape binding; on_action
  listener; WorkspaceView state field + init; task-surface render
  registration directly under flauz_shell::render_task_rail — the
  More/Inspect neighborhood; the navigation-close hook). Diff +55/−2.
- Root Cargo.toml — the one members line "crates/flauz-cap"; Cargo.lock —
  the package entry that line implies.

Implementation summary:
- The resolver crate follows the flauz-context pattern exactly: serde-only
  (serde_json dev-only), no flauz-world/flauz-exec/flauz-context imports —
  foreign references cross the seam as frozen format strings
  (CapabilityKey re-validates the frozen [a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*
  grammar that flauz-exec's CapabilityId owns; the grammar vectors are
  re-pinned in tests so the formats stay byte-identical across the seam).
  resolve_capability(capability, inputs) is a pure function: it walks the
  five admission dimensions in canonical order, admits iff
  inputs.admits(dimension, capability), and for every missing dimension
  emits a NamedGap (honest reason — including the detached-vs-attached-
  lacking environment distinction and decision-detail pass-through)
  mirrored one-for-one by an UnlockPath (honest action hint that names
  what admits the capability, never claiming a surface ships today).
  CapabilityResolution::validate() enforces record consistency: gaps/unlock
  paths must mirror the missing dimensions in canonical order, and
  available ⟺ gaps.is_empty(). Public fakes: fake_input_space()
  (deterministic complete 144-case enumeration: 3 capability keys × 2
  model × 2 runtime × 3 environment × 2 permission × 2 policy shapes) +
  typical/minimal/all_admitted_inputs().
- The gap surface implements the full J-04 disclosure chain over a plain
  view-model (CapabilityGapDetail: capability name + five DimensionStatus
  rows): "Capability unavailable — {capability}" → the Why? toggle →
  "What's missing" (each named dimension: "Model — The model in use
  doesn't offer it", …) → "Ways to unlock" ("Choose a model that offers
  it", …); the capability-available success state names what is admitted;
  the honest empty state covers the panel until a later wave attaches
  real resolution data — the chain is fully implemented and unit-tested,
  no data invented. The app crate does not import flauz-cap (codex-app's
  Cargo.toml is not an owned file). 019 request-once focus capture + 017
  close-restore; WO-P2-012 no-chat guidance; all copy in one registry.

Tests/commands + exact results (sandbox HAS the Rust toolchain —
cargo 1.98.1 + the pinned 1.97.1 — but CANNOT link the GUI crate:
codex-platform → xcap → libpipewire-0.3/libxkbcommon-x11 system libs are
absent and not installable without root; stated plainly, the Lead
independently compiles and gates):
- cargo fmt --all --check → CLEAN (exit 0, no diffs)
- cargo clippy -p flauz-cap --all-targets → CLEAN (0 warnings, 0 errors)
- cargo test -p flauz-cap → 31/31 GREEN: 20 unit (lib) + 6 conformance
  (fixtures round-trip byte-identical; invalid fixtures fail strict
  parses; the resolver-output law; the public fakes equal the committed
  inputs fixtures; no floats; no credential material) + 5 resolver (the
  no-silent-fall-through property over the complete 144-case space; the
  single-dimension intersection algebra; detached-vs-lacking honesty;
  decision detail pass-through; determinism/replay byte-identical)
- The gap surface module was verified via a temporary scratch crate
  (deleted before commit, not in the bundle): cargo check --all-targets +
  cargo clippy --all-targets of the FULL module against the real
  gpui/gpui-component/codex-core APIs → CLEAN (caught and fixed two real
  bugs: a moved-value borrow in the panel render and a wrong
  include_str! path); its 5 pure-logic unit tests run 5/5 GREEN via a
  zero-dependency stripped copy; every seam string the test asserts was
  additionally grep-verified against the real ui.rs.
- cargo check -p codex-core, -p gpui-component → GREEN (the dependency
  chain up to the environmentally-blocked GUI link).

Kernel-compliance checklist (Wave-2 addendum §1–§7):
1. Provider fabric behind frozen interfaces — COMPLIANT: no trait touched,
   no provider implemented; the resolver takes everything as data. No
   flauz-exec import at all.
2. Fake-consistency-first — N/A for this order (no external provider
   here); the crate's fakes are deterministic in-memory inputs, kernel
   §7-shaped.
3. Credentials are references, never material — COMPLIANT: no credential
   values anywhere; the no-credential-material conformance test scans all
   serialized fixtures.
4. Intersection with named gaps; silent fall-through is a violation —
   COMPLIANT (the core of the delivery): every missing dimension NAMED;
   the property test proves non-empty gap lists by exhaustive enumeration;
   CapabilityResolution::validate() rejects the silent-fall-through record
   shape (available=true with gaps); the UI chain surfaces the named
   dimensions.
5. Registry entity rules — COMPLIANT in the parts that apply: resolution
   records are pure value records (no version counter/no optimistic
   concurrency — they are not registry entities); "v":1, snake_case,
   deny_unknown_fields, no floats, kind-tagged enums; typical/minimal/
   invalid fixtures per record type, generated by the resolver over the
   documented inputs.
6. Task identity is sacred — COMPLIANT: the resolver never creates or
   forks task identity (pure function, no IDs); the UI surface is additive
   inspection state with no reducer changes.
7. Seven-layer GUI rule — COMPLIANT (see layers below); the palette is
   never the only discovery mechanism (labeled affordance + chord +
   palette).

GUI discoverability layers covered (J-04):
1. Visible primary entry — the labeled "Capabilities" affordance on the
   task surface, directly under the task control rail (More/Inspect
   neighborhood), always visible while a task is selected.
2. Contextual affordance — the gap disclosure panel with the
   Why?/What's-missing/Ways-to-unlock chain.
3. Palette fallback — the row "Why is a capability unavailable?"
   (WorkspaceShell group, requires a selected chat).
4. Keyboard path — Ctrl+Alt+Shift+6 + the alt-^ shifted-symbol companion
   (the N6 gate-fix family), scoped Escape close, focusable/tab-navigable
   panel.
5. Useful empty state — honest copy: what the panel will do, the
   no-quietly-disappears promise, and what to do today.
6. Success/next state — the capability-available state naming what is
   admitted, from which dimensions.
7. Honest unavailable state — the full named-gap chain with per-dimension
   unlock hints.

Acceptance-criteria evidence:
1. Single clean commit at the recorded base; owned files only — ✅ exactly
   one commit 158fa0a on fda38ea; the diff touches only crates/flauz-cap/
   **, ui/flauz_capability_gap.rs, the named ui.rs seams (+55/−2, each
   hunk a CAP-001-commented registration), the one workspace members line
   + its lock entry; working tree clean.
2. Resolver: intersection with named gaps; the no-silent-fall-through
   property test; deterministic; inputs only — ✅ resolve_capability + the
   exhaustive 144-case property test; determinism test replays
   byte-identically; zero flauz-world/flauz-exec/flauz-context imports.
3. Gap UX: the full Why/What-missing/Ways-to-unlock chain, seven-layer
   discipline, user language — ✅ the chain + success state fully
   implemented over the view-model; the copy test forbids internal type
   names and asserts the J-04 vocabulary.
4. Canonical JSON + fixtures per addendum §5 — ✅ "v":1 (rejecting other
   versions), snake_case, deny_unknown_fields, no floats, internally
   kind-tagged enums; typical/minimal/invalid per record type + the
   resolver-output law.
5. Non-empty Contract deviations blocks closure — ✅ see below: NONE.

Known limitations:
- The GUI crate cannot LINK in this sandbox (missing system
  libpipewire-0.3 / libxkbcommon-x11; no root). UI compile verification
  was done by typechecking the full module against the real APIs via a
  temporary scratch crate (which caught and fixed two real compile bugs)
  plus running its pure-logic tests dependency-free plus grep-verifying
  every ui.rs seam string. The Lead's independent compile is the final
  gate (as the work order anticipates).
- The panel renders its honest empty state until a later wave wires live
  resolution records into the view-model (by design: no cargo dependency
  on flauz-cap was addable within the owned-files boundary, and inventing
  capability data would fake functionality).
- The GUI/lab evidence scene (J-04 at the merged binary) is the Lead's
  gate, not the worker's, per the work order.

Contract deviations: NONE.

Follow-up work:
- Wire the gap surface to real resolutions: add flauz-cap to codex-app's
  dependencies and populate CapabilityGapDetail from CapabilityResolution
  records at the task surface (one owned-files work order).
- The Lead's J-04 lab scene at the merged binary (gap surfaced → Why? →
  named dimensions → unlock hints).
- Future waves the record is built for: skill unlock flows, permission
  UIs (the resolution record is their evidence type); runtime/model switch
  re-resolution keeping task identity (addendum §6).

Delivery: commit 158fa0a on feat/cap-001-capability-resolver, not pushed;
bundle at /home/z/my-project/CAP-001-delivery.bundle (verified). Worklog
recorded at /home/z/my-project/worklog.md.
```

## Post-merge notes

- The resolver crate + gap surface are now on `main` (93125d0).
- The gap surface's view-model API is `#[allow(dead_code)]`-annotated with
  its wiring owner — the Wave-2+/F4 slice that feeds live
  `CapabilityResolution` records into the panel wires it.
- Wave-2 integration gate: the capability-resolver harness step runs the
  resolver over the public fake input space at the merged binary and
  asserts the no-silent-fall-through property end-to-end (added at the
  Wave-2 gate run).
