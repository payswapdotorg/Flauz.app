# WEB-002 — Completion Report

**Work order:** WEB-002 — remote environment control, model/provider
selection, skill-unlock UI, collaboration/presence,
artifacts/documents/sheets, and the accessibility/responsive pass on the
WEB-001 foundation (Phase F11, Wave 6b, dispatched Worker B).

**Base branch + SHA:** `main` @ `d953edda1fdb27f1850e6ab434bc2ab129d7e5bb`
(cloned HEAD verified equal to the pinned base by `git rev-parse HEAD` at
STEP ZERO).

**Branch/commits:** `feat/web-002-capabilities`, one clean commit:
`feat(web): WEB-002 — the web capability surfaces: environments, models, skills, collaboration, artifacts + the a11y/responsive pass (Wave 6)`
Delivery bundle: `web-002-delivery.bundle`
(`d953edda1fdb27f1850e6ab434bc2ab129d7e5bb..feat/web-002-capabilities`).

**RE-ENTRY LAW check:** the WEB-002 capability surfaces did NOT exist at
the pinned base (the WEB-001 tree has no environments/model/skills/
collab/artifacts surfaces — only the shell, the connection state machine,
the session view with Context/Environments drawers, and the J-01..J-05
lab). This was a full implementation dispatch, not verify-and-report.

## Changed files / surfaces

- `web/src/state/capabilities.ts` (+ tests) — the protocol-derived
  capability engine: availability is computed from the generated
  `REQUEST_METHODS` constants (the schema snapshot), never asserted by
  hand; the structural capabilities (turn/start `model`/`effort`/`cwd`,
  the `skill` UserInput variant) are verified against `schema.json` in
  the tests.
- `web/src/state/turnParams.ts` (+ tests) — the pure turn-param builders
  (`buildTurnStart`, `buildSkillReferenceInput`): the per-turn execution
  choices ride the generated `TurnStartParams` fields exactly; nothing
  invented.
- `web/src/state/items.ts` (+ tests via turnParams.test.ts) — the
  defensive, hard-bounded decoder for the turns page
  (`thread/turns/list` data → `SessionItem`s labeled by their own
  protocol type).
- `web/src/state/app.tsx` — the capability data flows: model choice,
  per-turn cwd, session model/effort (from `thread/resume`), session
  items (from `thread/turns/list`), live turn-running tracking,
  `turn/interrupt` (named cancelled state), the skill-reference turn,
  the transient notice strip, and the session-load retry on
  (re)connect (a real defect found by the lab: a reload raced the
  gateway attach and left the task surface stuck — fixed).
- `web/src/app/panels/` (NEW, + tests per surface) — `GapCard`,
  `PanelFrame`, `EnvironmentsPanel`, `ModelPanel`, `SkillsPanel`,
  `CollaboratorsPanel`, `ArtifactsPanel`.
- `web/src/app/SessionView.tsx` — the task control rail (Context,
  Environments, Model, Skills, Collaborators, Artifacts) with
  `aria-pressed` toggles, Escape-close + focus restoration (the 017
  law), the Stop-this-turn control, the aria-live approvals queue, and
  §6 decided approvals kept as NAMED records (a WEB-001 honesty gap
  fixed: resolved approvals no longer vanish silently).
- `web/src/app/NewTaskView.tsx` — the contextual model affordance at
  compose time (J-14's contextual entry).
- `web/src/app/AppShell.tsx` — the palette commands for all five
  surfaces (the fallback discovery layer; task-scoped surfaces with no
  open task produce a NAMED notice, never a silent no-op), the
  Ctrl+Alt+Shift+1..6 panel chords, and the notice strip.
- `web/src/keyboard/keyboard.ts` — the six panel chords in the
  shortcuts sheet.
- `web/src/strings/en.ts` — the new surface copy (what/why/unlock per
  gap; all user-visible strings through `t()`); dead WEB-001 drawer
  keys removed.
- `web/src/styles.css` — the panel/rail/gap-card/item styles and the
  responsive pass: panels STACK below the task at mobile width instead
  of being hidden, and the workspace nav stays reachable as a
  horizontal strip (previously `display: none` at ≤860px).
- `web/lab/mock-gateway.mjs` — the honest protocol-slice extension:
  reflects the client's own `model`/`effort`/`cwd` choices through
  `thread/resume`/thread summaries; richer task items
  (`commandExecution`, `fileChange`); the `skill` input join; long turns
  honouring `turn/interrupt` with a named `cancelled` terminal state.
  Still implements ONLY the snapshot's methods (`model/list`,
  `skills/list` etc. are refused with method-not-found — the gap cards
  stay evidenced as real).
- `web/lab/journeys.mjs` — J-04/J-05 updated to the upgraded surfaces;
  new: J-06, J-08, J-09 (artifacts variant), J-13, J-14, J-15, the
  domain-neutral research scenario, and the A11Y/responsive journey.
- `web/tsconfig.json` — `resolveJsonModule` for the schema assertions.
- `docs/research/evidence/web/**` — regenerated evidence: 13 journeys,
  screenshots + action logs + truthful-state assertions (4.9 MB), plus
  this report.

**Totals:** 87 files, +5257/−159; the source-only diff (web/src, web/lab,
web/tsconfig.json) is 28 files, +3349/−123. `git diff` proves ZERO
changes outside `web/**` and `docs/research/evidence/web/**`.

## Implementation summary

The frozen protocol snapshot carries NO `environment/*`, `model/*`,
`skill(s)/*`, membership/presence, or `artifact/*` methods. Per the
Wave-6 kernel addendum §1 (the app-server protocol is the ONLY
capability contract; a missing capability is a NAMED gap, never a
fabrication), every surface is built from what the snapshot DOES carry,
with the gaps named and recovery-pathed:

1. **Environments** — the reported working directory (thread summary
   `cwd`) + the ONE real environment control the protocol carries: the
   per-turn `cwd` on `turn/start` (applied, reflected, and honest about
   the remote-attach gap whose recovery is a protocol extension; the
   desktop is the reference).
2. **Model/provider** — a REAL picker: current model/effort from
   `thread/resume`, provider truth from `account/read`, and the choice
   riding `turn/start`'s own `model`/`effort` params (same task
   continues — J-14 evidenced). The catalog-enumeration gap is named
   with the snapshot-regeneration recovery path (the real app-server
   carries `model/list` per the codex-protocol crate — NOT hand-copied
   into the snapshot; that would violate §3).
3. **Skills** — the protocol-native skill reference form (the generated
   `UserInput` `skill` variant: name + location + optional note on the
   next turn) + the J-04 locked-skill unlock path; listing/unlocking is
   a named gap.
4. **Collaboration/presence** — the named protocol gap with the
   flauz-collab shapes as the recovery path (owner/admin/contributor/
   viewer roles, presence), the F9 privacy law stated on the surface,
   and the §6 conflict-honesty NOW: approvals are aria-live named
   records that never auto-resolve and stay as attributed decided
   records; cancellations (turn/interrupt) surface as named `Cancelled:
   <reason>` timeline records.
5. **Artifacts** — the items the app-server actually reports
   (`thread/turns/list`), labeled by their own protocol type (agent
   message / command / file change), with honest
   loading/error/empty/success states and the documents/sheets gap
   named.
6. **A11y/responsive** — keyboard-complete rail (Tab/Enter/Escape with
   focus restoration per the 017 law), `aria-pressed`/`role=note`/
   `aria-live` wiring, Ctrl+Alt+Shift+1..6 chords, mobile stacking
   (surfaces no longer hidden at ≤860px), nav kept reachable on mobile,
   reduced-motion respected (inherited).

## Tests / commands + exact results

Sandbox toolchain: Node v24.21.0 + npm 11.19.0 present; **cargo is NOT
installed in this worker sandbox** (same absence the WEB-001 worker
declared; the Lead's gate station compiles the Rust side).

```
cd web && npm install            # OK (no new dependencies added)
npm run build                    # tsc --noEmit strict + vite build → PASS
                                 #   dist 0.74 kB html / 12.26 kB css / 268.97 kB js
npm test                         # vitest: 11 files, 86/86 PASS
                                 #   (19 WEB-001 + 67 new; duration ~6.4 s)
node lab/journeys.mjs            # 13/13 PASS — run three times consecutively
                                 #   (webrun_mugu5ra2, webrun_mugu6fvw, webrun_mugu84nr all pass)
cargo check -p flauz-web-gateway # NOT RUN — the crate is UNTOUCHED
                                 #   (git diff --stat over crates/, Cargo.toml,
                                 #   Cargo.lock = 0 lines); nothing to gate
```

The 67 new tests: the capability engine (13 — including the flip tests
that prove the gap cards go truthful when the snapshot regenerates),
turn-params/items (14), environments (7), model (8), skills (6),
collaborators (5), artifacts (7), the task-rail integration incl. the
017 focus law (7).

## Kernel-compliance checklist (§1–§9)

- **§1 protocol-only law:** every capability claim is derived from the
  generated `REQUEST_METHODS`/schema snapshot (`state/capabilities.ts`);
  zero fabricated capabilities; the gap cards name what/why/unlock. The
  mock gateway implements ONLY the snapshot's methods so the gaps are
  evidenced against a faithful transport.
- **§2 gateway crate:** UNTOUCHED (git diff proves zero changes under
  `crates/`; no deviation requested — no gateway seam was needed).
- **§3 generated types only:** zero hand-written protocol types; the
  new surfaces ride the generated `TurnStartParams`/`UserInput`/
  `ThreadResumeResult`/`ThreadTurnsListParams` shapes; the
  `resolveJsonModule` tsconfig change only lets TESTS assert the
  schema document itself.
- **§4 auth/credential law:** no credentials in URLs, logs, evidence;
  the model picker stores a model id and effort only; nothing new
  touches auth.
- **§5 truthful state:** the connection-state machine is UNTOUCHED and
  its tests stay green; the new surfaces inherit the four canonical
  states; a real reload/reconnect race that left the session surface
  stuck was found by the lab and FIXED (session load retries on
  bridge attach).
- **§6 conflict-honesty/takeover:** approvals arrive as aria-live named
  records, never auto-resolved (J-08 evidenced), decided approvals stay
  as attributed records, cancellations propagate as named `Cancelled`
  timeline records with the reason.
- **§7 frozen desktop:** zero desktop changes (owned paths only).
- **§8 journeys + evidence:** J-01..J-05 still green + J-06/J-08/J-09/
  J-13/J-14/J-15 web variants + the domain-neutral research scenario
  (non-code: community-garden planning) + the A11Y/responsive journey
  (keyboard-only drives, aria assertions, mobile 375px + desktop 1280px
  captures) — all under the parity-lab evidence schema.
- **§9 scope honesty:** one bounded commit-branch delivery; the full
  source tree kept; node_modules/dist/lab state cleaned after the
  gates were recorded.

## GUI discoverability layers (per surface)

| Surface | Primary visible | Contextual | Palette fallback | Empty/error/success | Keyboard |
|---|---|---|---|---|---|
| Environments | task rail button | the cwd truth + apply state on the session | "Show this task's environments" | unreported-cwd named; apply error surfaced; applied success state | Tab/Enter/Escape + Ctrl+Alt+Shift+2 |
| Model | task rail button | the compose-time affordance (NewTaskView) + current-model line | "Choose the model for this task" | "Not reported yet" honesty; applied confirmation; catalog gap named | Tab/Enter/Escape + Ctrl+Alt+Shift+3 |
| Skills | task rail button | the skill-reference form on the session | "Show this task's skills" | disabled-until-valid form; named send error; locked-skill unlock path | Tab/Enter/Escape + Ctrl+Alt+Shift+4 |
| Collaborators | task rail button | shared-decisions count + show-decisions action | "Show collaborators and shared decisions" | zero-decisions honesty; gap named | Tab/Enter/Escape + Ctrl+Alt+Shift+5 |
| Artifacts | task rail button | post-turn item refresh; refresh control | "Show this task's artifacts" | loading/empty/error/ready all named; gap named | Tab/Enter/Escape + Ctrl+Alt+Shift+6 |
| (task-scoped surfaces with no task open) | — | — | palette command → NAMED notice ("Open a task first…") | notice strip with dismiss | — |

## Acceptance-criteria evidence

1. **All WEB-001 gates still green:** npm build PASS; npm test 86/86
   (the 19 WEB-001 tests untouched and green); cargo gates untouched
   (zero Rust changes — no deviation to approve).
2. **Every surface: primary + contextual + palette + empty/error/
   success + keyboard + named-gap honesty:** the table above; the
   capability engine derives the gaps from the snapshot; J-04/J-05/
   J-06/J-13/J-15/J-09 journey assertions pin the named-gap copy; the
   model/catalog and skills/listing gaps name the snapshot-regeneration
   recovery path.
3. **§6 conflict-honesty:** J-08 — the approval arrives (aria-live),
   never auto-resolves (900 ms open-wait asserted), the decision stays
   a named `data-outcome="approved"` record, and the interrupted turn
   lands as a named `Cancelled: …` record with the propagation reason.
   The takeover approval cards ride the WEB-001 server-request routing
   unchanged.
4. **A11y/responsive evidenced per surface:** the A11Y journey —
   keyboard-only opens/closes all six rail surfaces with focus
   restoration (18 assertions: opens/escape/focus × 6), aria role
   checks (rail group label, gap cards as `role=note`), mobile 375px
   full-width captures per surface + the shell, desktop 1280px side
   panel, mobile nav visibility. Screenshots at both widths per
   surface are in the evidence tree.
5. **Zero changes outside owned paths:** `git diff` proves it (87
   files, all under `web/**` + `docs/research/evidence/web/**`);
   protocol types remain ONLY the generated set.

## Known limitations

- The five capability surfaces are honestly gap-carded where the
  frozen snapshot lacks methods; the REAL app-server (per the
  codex-protocol crate at the Flauz base) carries `model/list` and
  `skills/list`, which the snapshot does not — regenerating the export
  (CODEX_RS_CODEX_BIN=… `npm run protocol:export && npm run
  protocol:generate`) flips the engine to `available` and the gap cards
  self-silence (proven by the flip tests), at which point list-driven
  UI work is a follow-up.
- The lab ran against the Node mock gateway (the sandbox lacks the Rust
  toolchain — the same declared absence as WEB-001); the Lead reruns
  with `node lab/journeys.mjs --gateway "target/debug/flauz-web-gateway
  --web-root web/dist"`.
- The cwd control is the only environment control the protocol carries;
  remote attach/detach needs the protocol extension named on the card.
- Model ids are entered, not browsed (the catalog gap above).

## Contract deviations

**NONE.** No gateway-crate change, no protocol-seam addition, no
hand-written protocol types, no desktop changes.

## Follow-up work

1. Regenerate the schema snapshot from the real app-server export →
   model catalog + skill listing light up (engine + gap cards already
   react mechanically).
2. A protocol work order for environment control methods (F3 fabric →
   app-server surface) to make the environments surface fully live.
3. Presence/membership methods bridging the flauz-collab contract
   (F9) — the web surface is ready to render them.
4. Artifact/document/sheet methods (F9/F6) for the artifacts surface.
5. The Lead's gate-station lab rerun against the real Rust gateway.
