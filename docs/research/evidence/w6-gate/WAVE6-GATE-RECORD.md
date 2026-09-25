# Wave-6 Gate Record — F11: Web Client CLOSED

> **The Wave-6 gate** (per WAVE6-WORK-ORDERS.md): both work orders merged
> with per-WO gates green + CI green on both platforms, the frozen kernel
> addendum §1-§9 compliance verified (deviations NONE ×2), the
> protocol-derived capability law honored (no fabricated capabilities),
> and the named protocol gaps recorded with their recovery paths.
> **The F11 Web client — production-gate client #3 — is CLOSED.**

## The merged wave

2 of 2 MERGED: WEB-001 (PR #57, merge `e99941e`), WEB-002 (PR #58, merge
`f5e2a5c`). Both dispatched from inside the replay (GLM-5.3 agents-tab
sessions), both reconstructed deterministically from the batch-store work
logs (zero skipped edits in both), both Lead-gate-fixed at the integration
station (the workers' sandboxes lack the toolchain — honestly declared in
both reports). WEB-002's first attempt died mid-stream (open-empty turn,
the platform stream instability) and was absorbed by the automatic
assault re-dispatch chain — attempt 2 delivered in 43 minutes.

- **WEB-001** — the foundation: `crates/flauz-web-gateway` (axum
  WebSocket↔app-server JSON-RPC bridge, supervised app-server per
  authenticated session, static-file hosting, localhost-only default
  bind, graceful shutdown, credential-free structured logs) + the `web/`
  TypeScript/React shell (layout, nav, palette, theming, keyboard map,
  auth flow, workspace/session list + connect, the four-state truthful
  connection machine) with protocol types GENERATED from the schema
  export (34 method sets, 7 named types, 22 methods — zero hand-written
  duplicates). 70 files +14,191. Includes one real security fix caught
  by the Lead gate: absolute-path SPA leakage (`//etc/passwd`) is now a
  named 404 refusal.
- **WEB-002** — the capability surfaces: the protocol-derived capability
  engine (availability computed from the generated `REQUEST_METHODS`
  constants, never hand-asserted), per-turn model/effort/cwd riding the
  real `turn/start` fields, the `skill` UserInput reference turn, the
  aria-live approvals queue with §6 decided approvals kept as NAMED
  records, turn interrupt with named cancelled state, the six task
  surfaces (Context, Environments, Model, Skills, Collaborators,
  Artifacts) with `aria-pressed` toggles + Escape-close + focus
  restoration, palette commands for all surfaces + Ctrl+Alt+Shift+1..6
  chords, and the full a11y/responsive pass. 87 files +5,510/−159,
  `crates/` untouched (git-diff-proven).

## Gate results (both WOs at the integration station)

| Gate | WEB-001 (PR #57) | WEB-002 (PR #58) |
|---|---|---|
| cargo tests | **26/26** (21 lib + 5 integration) | n/a — crates/ empty diff |
| clippy `--all-targets -D warnings` | **CLEAN** | n/a (workspace unchanged) |
| `cargo fmt --all --check` | **CLEAN** | n/a |
| `cargo check --workspace` | **RC=0** | n/a |
| dependency policy | **579 pkgs PASS** | n/a |
| web build (`npm run build`) | **PASS** | **PASS** (268.97 kB JS, byte-matching) |
| web tests (`npm test`) | **19/19** | **86/86** |
| lab journeys | **5/5 ×3** (J-01..J-05) | **13/13 ×3** (J-01..J-06, J-08, J-09, J-13..J-15, DOMAIN-NEUTRAL, A11Y) |
| CI Gate C | ubuntu **success** · windows **success** | ubuntu **success** · windows **success** |

The lab evidence (15 entries under `docs/research/evidence/web/`) is the
acceptance law made visible: the shared acceptance law (discoverable →
usable → observable → recoverable → truthful → evidenced) exercised by
headless-browser drives with screenshots + action logs + truthful-state
assertions per journey, including the A11Y journey (keyboard-only drives
+ aria assertions + mobile/desktop captures) and the domain-neutral
non-code scenario (the research-planning journey — the shared acceptance
law's non-software requirement).

## The kernel-addendum compliance (§1-§9)

1. Protocol-only capability truth — **held**: the gap engine derives
   every gap from the generated constants; zero fabricated capabilities.
2. One new crate, gateway = transparent transport — **held** (WEB-001).
3. web/ outside the Rust workspace, generated types only — **held**:
   the schema export is the single protocol type source.
4. Auth/credential law — **held**: no credential in URLs, logs, or
   evidence; localhost-only default bind evidenced.
5. Truthful connection state — **held**: the four-state machine +
   the session-load retry defect found and fixed by the lab.
6. Conflict-honesty/takeover laws on web — **held**: approvals surface
   as NAMED records; decided approvals kept (a WEB-001 honesty gap
   fixed in WEB-002); cancellation named with propagation reason.
7. Frozen desktop behavior — **held**: WEB-002's `crates/` diff is
   empty; the desktop remains the parity reference.
8. Journeys + evidence — **held**: 13 journeys ×3 runs, domain-neutral
   + A11Y included.
9. Scope honesty under the wedge — **held**: bounded deliveries; the
   WEB-002 attempt-1 death absorbed by the assault chain.

## Honest deferrals (recorded, not hidden)

1. **The named protocol gaps** — environment listing/control,
   membership/presence, artifact listing are not in the app-server
   protocol surface today; the web client shows truthful, NAMED gap
   cards with protocol-work-order recovery paths (never a fabrication,
   never a silent no-op). Closing them requires protocol work orders
   (deviation-gated additive seams) in a future wave if the roadmap
   mandates those capabilities for production.
2. **Formal verification** — this record closes Wave 6 (the client is
   BUILT and evidenced at the lab level); the formal production
   verification across Linux + Windows + Web (immediate-order items
   5-6) is the next stage, and it will re-exercise these journeys at
   the current-main SHA.
3. The pre-existing duplicated FlauzConflicts listener remnant in
   ui.rs (~line 45896, a LEASE-001 gate-fix leftover; harmless) —
   Wave 6 did not touch desktop `crates/` or `codex-app` UI, so the
   remnant remains flagged for the next ui.rs-touching wave.

## Provenance archive

Worker reports + work logs + the reconstruction tooling live in the
replay repo (payswapdotorg/replay2, scripts/worker-reports/ +
harvests/); Flauz delivery bundles were unreachable (sandbox boundary),
the deterministic batch-store reconstructions (WEB-001: 52 Write + 39
Edit/MultiEdit; WEB-002: 23 Write + 48 Edit/MultiEdit; zero skipped
edits in both) produced the merged trees. WEB-001's protocol-types
pipeline + 21 lab evidence artifacts were regenerated authentically at
the gate station; WEB-002's lab was run live (13/13 ×3).

**Roadmap position after this record:** F0-F2 ✅, Waves 2-5 ✅,
Wave 6 (F11 Web client) ✅ **CLOSED 2026-09-25**. Next: bring Linux +
Windows + Web to formal verification readiness, then run the formal
production verification across all three (ACTIVE-EXECUTION-STATE.md
immediate order items 5-6).
