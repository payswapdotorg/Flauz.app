# Wave-4 Gate Record — F7 BYOP / F8 Labs / F9 Collaboration CLOSED

> **The Wave-4 integration gate** (per WAVE4-WORK-ORDERS.md): the
> three-fabric scenario at the merged binary + the lab scenes + CI green.
> All three gates GREEN on 2026-09-23. **F7, F8 and F9 are CLOSED.**

## The merged wave (the gate's subject)

3 of 3 MERGED: LAB-001 (PR #52, `a134cec`), PROV-001 (PR #53, `250109d`),
COL-001 (PR #54, `526d53c` — the ui.rs three-way resolved as the additive
union: palette `ALL` 87+2 = **89/89**, both chord listeners retained on
the workspace root, the members-line + lockfile three-entry unions).
Evidence + ledger at
[w4-lab-001](../w4-lab-001/LAB-001-COMPLETION-REPORT.md) ·
[w4-prov-001](../w4-prov-001/PROV-001-COMPLETION-REPORT.md) ·
[w4-col-001](../w4-col-001/COL-001-COMPLETION-REPORT.md); docs head
`f761252` (526d53c + docs-only).

## Gate A — the three-fabric scenario (harness, ALL 19 STEPS PASS)

New harness step `[19]` (`w4_three_fabric_step.rs`, archived here;
run log `w4-harness-run.log`): **ONE task through three fabrics** —
the task's identity IS the routing scenario's task (`task_…DQPD3`,
created in the world store), so the same task's event stream carries
its scheduling history.

- **[19b] PROV (F7)** — two fake accounts connected through the
  secret-store seam (PRACTICE material → `flausec_` refs; the accounts
  carry references, never material); free-tier-first routing across
  three turns: turn 1 free (`DefaultOrder`), turn 2 free (3 of 5
  remaining — then consumed to depletion MID-RUN), turn 3 the **NAMED
  escalation**: paid chosen, `PolicyRule::EscalatedToPaid`, the free
  account left behind in `skipped` with `SkipReason::Depleted` — never
  a silent fall-through. All 3 choices recorded on the task's world
  event stream as `task.scheduled_on_account` events (new event type
  through the world-store seam, frozen-format discipline), each
  carrying the choice's canonical JSON + the flat attribution fields;
  every choice round-trips and re-validates (the attribution law);
  the escalation is visible on the stream; ledger attribution 5 free
  + 1 paid, attributed to THIS task.
- **[19c] LAB (F8)** — the a11y chord-ladder journey (the d25/N5/N6/d19
  shapes) serialized ONCE and re-parsed (the same journey BYTES) ran
  through `run_journey` on BOTH the LocalLabAdapter and the
  FakeRemoteLabAdapter: the comparator reports **zero divergences
  (14 steps matched)** on the identical runs, and **exactly ONE named
  major divergence** (`frame_anchors`, located at step + probe) on the
  seeded anchor flip — which still validates as well-formed evidence
  (it is divergent, not malformed).
- **[19d] COL (F9)** — the canonical two-actor interleaving (12 steps)
  through `run_interleaved` + `verify_expectations` +
  `verify_f9_laws`: permissions hold under interleaving; **Dev's
  member-private note never appears in Ana's projection** (the §6
  privacy law, checked against the actual projection rows); THREE
  named denial kinds present (`role_denied`, `version_conflict`,
  `private_record`); the sharing posture STATED (the shared-filesystem
  mode + the visibility split); the interleaving log round-trips
  through canonical JSON **byte-identically**.
- **The no-credential-material law** across all three fabrics'
  serialized state (the simulation log + the seeded evidence + every
  stream envelope): the `CREDENTIAL_MARKERS` scan is VALUE-PREFIX
  ADAPTED (the flauz-lab conformance precedent — a value that STARTS
  like a credential; legitimate identifiers like the anchor
  `task-surface` contain the bare `sk-` substring without being
  material) — clean.

Full-harness context: steps [1]-[18] (the F2 core, Wave-2 fabrics,
Wave-3 F6 domain-neutral scenario) all still PASS at the merged head —
no regression from the three merges.

## Gate B — the lab scenes at the merged binary (VLM-adjudicated)

Binary `codexrs-w4gate-f761252` (release build at the merged head;
script `d26-w4-surfaces.sh` archived here; evidence frames + VLM reads
in this directory). Scene d26 — 14 frames, keyboard-only, the d25
discipline:

| Frame | Probe | Result |
|---|---|---|
| B03/B04 | `Ctrl+Alt+Shift+P` — the providers panel | **PASS** — VLM verbatim: "Your provider accounts" + "No provider accounts connected yet" + the full empty body + the connect form (OpenAI / Account name / API key / "Connect account" / "The key is stored securely; Flauz never shows it again.") + **the routing-policy moment** ("Which account Flauz uses" · "Order today: free accounts first" · "Change the order") + "Spending and task limits" |
| B05 | scoped Escape | **PASS** — B05 == B02 pixel-identical (clean close, no trap; the 017 contract) |
| B06b | palette "Connect a provider account" + Return | **PASS** — the panel lands (VLM confirms the providers panel open) |
| B07b | palette "See which account a task uses" + Return | **PASS** — the panel + the honest guidance verbatim: "This task has not been set up with an account yet — it will get one when it next runs." |
| B08/B09 | `Ctrl+Alt+Shift+U` — the members panel | **PASS** — VLM verbatim: "Who is on this workspace" + "Just you" + the solo body + BOTH not-wired states ("Inviting isn't connected yet … nothing is sent today" / "Changing roles isn't connected yet") + "Invite someone" |
| B11b | palette "See who is on this workspace" + Return | **PASS** — B11b == B08 pixel-identical (the palette landing equals the chord landing) |
| B12b/B13 | palette "Change a task's sharing" + Return | **PASS** — VLM verbatim: "Sharing" + Files/"Isolated copy — your files stay separate" + "Remembered notes"/"Only me — remembered notes stay visible only to you" + "Work products"/"The workspace — everyone here can see this task's work products" + "Change sharing" + the honest sync note |
| B14 | final Escape settle | **PASS** — B14 == B10 (the task surface stands) |

The d25 listener law holds: both new chords (`Ctrl+Alt+Shift+P`,
`Ctrl+Alt+Shift+U`) have their `on_action` listeners on the workspace
root (verified in the merge; both fire in the scene — B03 and B08
render their panels through the CHORD path, not just the palette rows).

**The denial-with-why** ("You can't change this sharing — your role
here is {role}. Ask an admin or the owner to change it.") is F10+
wiring: the `why_not_for_role` seam is pinned by the module's copy
tests, and the GUI honestly renders the owner's can-change posture at
cold start (nothing invented) — the same honest-not-wired discipline
the d25 gate accepted for the invite/role states.

## Gate C — CI at the final main head

Run 35862314966 at `f7612524` (the docs head over the code head
`526d53c` — zero code delta): **ubuntu-24.04 SUCCESS** +
**windows-latest SUCCESS** (the concurrency pattern: the superseded
runs at 526d53c/250109d auto-cancelled).

## Verdict

**WAVE 4 (F7/F8/F9) CLOSED** — 3/3 merged + evidence + ledger + Gates
A/B/C green. The roadmap advances per ACTIVE-EXECUTION-STATE.md:
remaining F6 orchestration depth → the Web client → Linux+Windows+Web
verification readiness.

Follow-ups carried forward (from the three completion reports):
- Wire the providers/members UI seams to the real crate types (the F7
  persistence slice; the catalog/attribution population).
- Record scheduling choices on live tasks through the world-store seam
  (the `task.scheduled_on_account` event type this gate exercised).
- The real local lab driver behind `LocalLabAdapter`'s contract; real
  provider adapters copying the fake-remote template.
- F10+ transport: the members roster/presence/attribution wiring, the
  sharing persistence, the invite/role flows.
