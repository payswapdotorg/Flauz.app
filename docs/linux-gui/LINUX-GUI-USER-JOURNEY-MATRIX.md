# Linux GUI — User Journey Matrix (J-01 … J-18)

**Lane:** Linux GUI usability (second Tech Lead — Linux lane).
**Test environment:** E2B Desktop (real GUI interaction — see
[LINUX-GUI-E2B-ENVIRONMENT.md](LINUX-GUI-E2B-ENVIRONMENT.md)).
**Protocol:** cold-start per journey — start from the normal UI, no
documentation, no command palette initially; primary path first, then
contextual controls, recovery/error states, then search/palette; inspect the
empty state; complete the journey; observe the next-step affordance.

Verdict vocabulary: `WORKS` / `WORKS-WITH-DEFECTS` / `UNAVAILABLE-BUT-HONEST`
(clear explanation + next step) / `MISSING` (no surface at all) /
`HIDDEN` (palette-only or developer-only) / `SILENT-NO-OP` (control exists,
does nothing visible) / `MISLEADING` (state contradicts reality).

| Journey | Implementation status (main) | Primary entry | Contextual entry | Search/palette entry | Empty state | Success state | Keyboard path | E2B result | Known limitations | Open defects |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| J-01 Start a project | recorded per run | recorded per run | recorded per run | recorded per run | recorded per run | recorded per run | recorded per run | recorded per run | recorded per run | recorded per run |
| J-02 Understand what the agent knows (Context) | see run log | | | | | | | | | |
| J-03 Recover a task | see run log | | | | | | | | | |
| J-04 Discover a capability gap | see run log | | | | | | | | | |
| J-05 Add an environment | see run log | | | | | | | | | |
| J-06 Coordinate browser + terminal + sandbox | see run log | | | | | | | | | |
| J-07 Parallel agents | see run log | | | | | | | | | |
| J-08 Human takeover | see run log | | | | | | | | | |
| J-09 Evidence / what happened | see run log | | | | | | | | | |
| J-10 Save successful work (reusable workflow) | see run log | | | | | | | | | |
| J-11 Discover saved work later | see run log | | | | | | | | | |
| J-12 Reuse / improve a workflow | see run log | | | | | | | | | |
| J-13 Collaboration | see run log | | | | | | | | | |
| J-14 Switch model | see run log | | | | | | | | | |
| J-15 Switch environment | see run log | | | | | | | | | |
| J-16 Resource conflict | see run log | | | | | | | | | |
| J-17 Human attention (Activity) | see run log | | | | | | | | | |
| J-18 Discover automation | see run log | | | | | | | | | |

> This matrix is populated by the E2B journey battery. Rows carry the
> per-run verdicts, entry paths exercised, evidence file names under
> `docs/linux-gui/evidence/`, and the classification of every failure
> (P0/P1/P2/P3/BOUNDED) into the
> [LINUX-GUI-REGRESSION-LEDGER.md](LINUX-GUI-REGRESSION-LEDGER.md).

## Run log

### Run 1 — baseline sweep (2026-09-21, E2B, Flauz @ `f66965e`)

Populated after the baseline sweep completes (see evidence dir).
