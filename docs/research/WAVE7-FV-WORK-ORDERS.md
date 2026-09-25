# Wave 7 Work Orders — Formal Verification (the production-gate verification wave)

> **Status: PINNED 2026-09-25 — the FV wave opens after Wave 6 closed
> (main `88a4b3b`). FV-001 is the first dispatch.**
>
> Shared-contract authority:
> [F2-CONTRACT-KERNEL.md](../F2-CONTRACT-KERNEL.md) (frozen) + the Wave-2..6
> addenda in [WAVE2-WORK-ORDERS.md](WAVE2-WORK-ORDERS.md) through
> [WAVE6-WORK-ORDERS.md](WAVE6-WORK-ORDERS.md) (all frozen) + **the Wave-7
> kernel addendum below**. Every work order follows
> [WORK-ORDER-TEMPLATE.md](../WORK-ORDER-TEMPLATE.md). Non-empty
> `Contract deviations` in a worker report blocks closure.
>
> **The phase this wave serves** (ACTIVE-EXECUTION-STATE.md immediate order
> items 5-6: "Bring Linux + Windows + Web to formal verification readiness"
> then "Run formal production verification across all three"; ROADMAP F13
> requires formal verification of Linux desktop + Windows desktop + Web
> client before the first production release). The 2026-09-23 sequencing
> amendment is the governing order; macOS/Mobile remain deferred.

## Wave-7 kernel addendum (frozen by the Tech Lead before dispatch)

1. **Verification truth is runtime truth.** A client is formally verified
   only by running the REAL artifact at the PINNED main SHA: the Linux
   desktop binary under LINUX_GUI_LAB (Xvfb + keyboard-only drives + frame
   captures, VLM-adjudicated), the Windows desktop binary on the CI
   windows-latest runner (cargo gates + the launch smoke + the
   journey-smoke extension), and the real web build served by the REAL
   Rust gateway in a real headless browser. Static reasoning, unit tests
   alone, and mock-gateway captures are NOT formal evidence (the mock
   gateway remains a worker-sandbox authoring fallback only).
2. **Three lanes, three truths, honestly bounded.** The E2B parity
   environment doc records the platform bounds (no Windows/macOS GUI host
   exists in this environment; WINDOWS_GUI_LAB unavailable). The Windows
   lane's formal depth is therefore bounded to what the CI runner can
   drive: compile + workspace tests + clippy (already green) + the launch
   smoke + the SendKeys journey-smoke scenes this wave authors. Anything
   beyond that is a NAMED gap in the FV gap list with its recovery path
   (a Windows GUI host or a Windows operator session) — never silently
   narrowed, never fabricated.
3. **Historical findings are reclassified, not assumed.** Every historical
   finding (the f1-sweep ledgers — journey-inventory, release-gate-
   inventory, accessibility-inventory, parity-inventory; known-failures.md;
   the codex-linux runtime observations; the ACTIVE-EXECUTION-STATE
   expectations list — GPUI/X11/lavapipe L-002, runtime/environment
   incompatibilities, silent no-ops, draft loss, raw protocol errors,
   toast persistence) is reclassified against CURRENT main:
   still-open / fixed-by-`<merge>` / changed-surface / needs-runtime-verify.
   A finding closes only per the synchronization law (verified in a fresh
   environment at current main).
4. **The journey catalog is the acceptance law.** The FV catalog maps
   J-01..J-18 (applicable variants per client) to concrete scenes with
   per-scene pass criteria, following the d-series discipline: calibration
   comments verified against the merged source, keyboard-only drives,
   named moments, frames + logs + truthful-state assertions. Each client
   includes at least one domain-neutral non-code scenario. The shared
   acceptance law (discoverable → usable → observable → recoverable →
   truthful → evidenced) is judged per journey.
5. **Evidence schema unchanged.** Formal evidence lives under
   `docs/research/evidence/fv-*/` in the parity-lab schema (RUN.json +
   per-journey dirs + captures + action logs + assertions + VLM reads +
   the pinned SHA). Every run pins its SHA; every scene script is
   committed and re-runnable; every VLM read is archived with its frame.
6. **The FV wave verifies; it does not fix.** Defects found during FV
   become focused fix work orders through the synchronization law
   (defect → focused PR → merge → fresh environment → rerun the affected
   journey). The ONLY code this wave may add: the FV harness itself
   (scene scripts, CI workflow extensions, lab runner flags) — no product
   behavior changes, no contract changes, no desktop/web client changes.
7. **Reproducibility and credentials.** Scene scripts assume the warm Lead
   station (source /home/z/parity-lab/build_env.sh) but hard-fail with a
   named message when their prerequisites are missing. No credential ever
   appears in scripts, logs, URLs, or evidence (the E2B harness law).
8. **Bounded deliveries under the platform wedge.** One bounded
   commit-branch delivery per worker, the same git-bundle contract as
   Wave-5/6; the Lead executes the formal passes (the runs need the
   toolchain + displays the worker sandboxes lack).

## FV-001 — Findings reclassification + the FV catalog (read-heavy, docs-only)

```
ID: FV-001
Title: reclassify every historical finding against current main + author
       the formal-verification catalog (journey → client → scenes → pass
       criteria) + the honest FV gap list
Phase: Formal verification (Wave 7, step 1)
Owner: dispatched worker (Wave-7 Worker A)
Dependencies: Wave 6 CLOSED (main past 88a4b3b); all wave ledgers read-only
             available
Contract(s): F2-CONTRACT-KERNEL + Wave-2..6 addenda + this file's addendum
Problem: the production gate requires formal verification of three clients,
         but the historical findings were recorded against older SHAs and
         older surfaces; nobody has reclassified them against current main
         or consolidated which journeys must run on which client
User-visible outcome: none directly (this is the verification-design step);
         the artifacts gate the formal passes
Scope:
  - docs/research/evidence/fv-001/FINDINGS-RECLASSIFICATION.md (NEW): every
    historical finding from the ledgers below, reclassified against current
    main with evidence pointers (the source surface today, the merge that
    changed it if any, and the runtime-verify need)
  - docs/research/FV-CATALOG.md (NEW): the journey→client→scene catalog —
    for each of J-01..J-18: which client(s) it applies to (Linux desktop /
    Windows desktop / Web), the scene(s) with named moments, the pass
    criteria, and the evidence target dir; includes the domain-neutral
    non-code scenario per client and the A11Y scene per client
  - the FV gap list (inside FV-CATALOG.md): what CANNOT be formally
    verified from this environment, per lane, with the honest reason and
    the recovery path (per addendum §2)
  - NO source changes; NO evidence fabrication; every reclassification
    cites its source reading
Non-goals: authoring the scene scripts (FV-002); running anything;
         changing product code; closing findings without runtime evidence
Files/subsystems owned: docs/research/evidence/fv-001/**,
         docs/research/FV-CATALOG.md; NOTHING else
Inputs: main @ 88a4b3b (pin at dispatch); docs/research/evidence/f1-sweep/**
         (journey-inventory.md, release-gate-inventory.md,
         accessibility-inventory.md, parity-inventory.md, recommendations.md,
         binding/FR-BINDING-VERDICT.md, F1-CLOSURE-RECORD.md);
         docs/known-failures.md; docs/research/evidence/codex-linux/**;
         docs/ACTIVE-EXECUTION-STATE.md (the expectations section);
         the wave gate records (w2..w6); docs/PRODUCT-UX-JOURNEYS.md
Outputs/artifacts: one clean commit branch `feat/fv-001-reclassify`,
         git bundle, completion report (11-field contract, Wave-5 shape)
Tests: n/a (docs-only) — the Lead gates by source-spot-check: every
         reclassified finding must cite real current-main source lines,
         and every catalog scene must name its client + moments + criteria
GUI/lab evidence: n/a (the catalog DEFINES the lab evidence for FV-002/003)
UX journey IDs: J-01..J-18 (cataloged, not run)
Acceptance criteria:
  1. Every historical finding in the input ledgers appears exactly once in
     the reclassification with a current-main verdict
  2. Every J-01..J-18 journey appears in the catalog with its client
     mapping (or a named N/A reason)
  3. The gap list names every lane bound with its recovery path
  4. Zero source-file changes (git diff proves it)
Rollback/recovery: revert the merge; docs-only
Integration notes: the catalog is the FV-002 authoring contract — scene
         scripts implement catalog rows 1:1; FV-003 executes them
Status: DISPATCHED 2026-09-25 (base pinned at dispatch)
```

## FV-002 — The FV harness (scene scripts + CI journey-smoke + web formal-pass manifest)

```
ID: FV-002
Title: author the formal-verification harness per the FV catalog: the
       Linux desktop J-suite scene scripts (d-series pattern), the Windows
       CI journey-smoke extension (SendKeys scenes), and the web
       formal-pass runner manifest
Phase: Formal verification (Wave 7, step 2)
Owner: dispatched worker (Wave-7 Worker B, after FV-001 merges)
Dependencies: FV-001 merged (the catalog is the authoring contract)
Contract(s): as FV-001 + the lab evidence schema (flauz-lab crate docs)
Problem: the catalog defines WHAT must run; the harness (scripts) that runs
         it does not exist yet
User-visible outcome: none directly; the harness executes the formal passes
Scope:
  - the Linux desktop scene scripts under scripts/fv/ (NEW): one script per
    catalog Linux scene, the d26 pattern (calibration comments verified
    against current main: chords, anchors, copy; keyboard-only drives;
    named moments; frame captures; hard-fail named prerequisites)
  - the Windows journey-smoke extension: scripts/windows_fv_smoke.ps1 (NEW)
    + its release.yml/ci.yml wiring (the SendKeys scene driver — launch,
    drive the catalog's Windows scenes, assert windows/states, capture what
    the runner can) — additive workflow steps only
  - the web formal-pass manifest: web/lab/fv-manifest.json (NEW) — the
    catalog's web scenes as runner inputs (journey id, gateway mode=real,
    assertions), plus any web/lab/journeys.mjs flags needed (additive)
  - a top-level runner doc: scripts/fv/README.md — how FV-003 executes each
    lane at the pinned SHA (commands, prerequisites, evidence targets)
Non-goals: running the formal passes (FV-003, Lead-run); product changes;
         protocol changes; new dependencies beyond the lab's existing ones
Files/subsystems owned: scripts/fv/**, scripts/windows_fv_smoke.ps1,
         .github/workflows/ci.yml + release.yml (ADDITIVE steps only),
         web/lab/fv-manifest.json, web/lab/journeys.mjs (additive flags
         only); NOTHING else
Inputs: main at the FV-001 merge SHA (pin at dispatch);
         docs/research/FV-CATALOG.md (the contract); the d-series precedents
         (docs/research/evidence/w4-gate/d26-w4-surfaces.sh and its kin);
         scripts/linux_desktop_smoke.sh; scripts/windows_desktop_smoke.ps1;
         web/lab/journeys.mjs
Outputs/artifacts: one clean commit branch `feat/fv-002-harness`,
         git bundle, completion report
Tests: bash -n / pwsh parse checks on every script; the Lead spot-runs one
         Linux scene + the Windows workflow in CI before merge
GUI/lab evidence: one calibration capture per authored Linux scene (the
         Lead runs them at the gate station)
UX journey IDs: per the catalog rows implemented
Acceptance criteria:
  1. Every catalog Linux scene has a script; every catalog Windows scene is
     wired into the CI journey-smoke; every catalog web scene is in the
     manifest
  2. Scripts hard-fail with named messages when prerequisites are missing
  3. Zero product-source changes (git diff proves it; only the owned paths)
  4. The CI workflow remains green (the additive steps must not break the
     existing gates)
Rollback/recovery: revert the merge; harness-only, no product impact
Integration notes: calibration against current main is the load-bearing
         work — every chord/anchor/copy string in a scene script must be
         verified against the merged source (the d26 discipline)
Status: NOT DISPATCHED (Lead pins after FV-001 merges)
```

## FV-003 — The formal pass (Lead-run; closes the FV wave)

```
ID: FV-003
Title: execute the formal verification passes at the pinned SHA across all
       three lanes; adjudicate; write the FV gate record; the production
       go/no-go input
Phase: Formal verification (Wave 7, step 3)
Owner: Tech Lead (executed at the Lead station + CI; NOT dispatched)
Dependencies: FV-002 merged
Contract(s): as FV-001/002
Problem: the production gate requires the formal verification to have RUN
User-visible outcome: the verified, evidenced state of all three clients at
         one pinned SHA — the production decision's input
Scope: run every catalog scene per lane at the pinned SHA; VLM-adjudicate
         the frames; assemble docs/research/evidence/fv-gate/ (RUN.json
         lineage, captures, reads, the lane verdicts, the defects found →
         fix work orders per the synchronization law); update
         ACTIVE-EXECUTION-STATE.md; the FV gate record
Non-goals: fixing defects (focused fix WOs); production release itself
Status: NOT STARTED (Lead-run after FV-002)
```

## Dispatch notes (Lead-only)

- Dispatch mode under the §16 platform wedge: ONE worker at a time (FV-001
  first; FV-002 only after FV-001 merges). The packet follows the proven
  Wave-5/6 wrapper: pinned full SHA, STEP ZERO (clone+checkout+branch),
  RE-ENTRY LAW, one-commit + bundle contract, 11-field report,
  honest-absence rules.
- The completion gate marker for the queue watcher: `FV-001 COMPLETION
  REPORT` (the WEB-prefix gate pattern generalizes — the FV- prefix vector
  needs the same 4-vector test discipline before dispatch).
- Harvest chain: extract/reconstruct → Lead source-spot-check (the
  reclassification citations) → docs gates (no cargo needed for FV-001;
  FV-002 additionally: bash -n, pwsh parse, one calibration scene run, CI
  green) → PR → merge.
- After FV-003: production hardening/release gates (immediate order item 7)
  per ROADMAP F13.
