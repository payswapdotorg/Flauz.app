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

### Run 1 — baseline sweep (2026-09-21, E2B sandbox `ipoaz31c43jj5vtjy8qvf`, Flauz @ `f66965e`, release build, Ubuntu 22.04.5 / Xfce4 / Xvfb :0 1920×1080, software GL)

Environment note: provisioning required the L-001 accommodation (pipewire
1.0.5 side-load) before the release build compiled.

| Journey | Verdict | Detail |
| --- | --- | --- |
| E-00 fresh desktop → build → launch → window appears | **WORKS-WITH-DEFECTS** | Build needs L-001 accommodation; binary launches; window maps (`com.codexrs.CodexRS`, 1278×818 @ +10+85, focused). |
| J-01 Start a project (cold start) | **BROKEN (P0, L-002)** | Window content never presented — black on Xfce4 AND bare Xvfb; desktop shows through the 32-bit ARGB window. No first-run/onboarding/sign-in surface reachable → every downstream journey is blocked at cold start. Keyboard (ctrl-shift-p), mouse move/click input events produced no visible change. |

Blocked-by note: J-02 … J-18 could not proceed past cold start (L-002).
The battery re-runs after L-002 closes; entries stay `NOT-RUN (blocked by
L-002)` until then.

Evidence: `evidence/run1-j01-02-gui-mapped-unrendered-xfce.png`,
`evidence/run1-j01-04-relaunch-softwaregl.png`,
`evidence/run1-j01-05-xwd-window-content-black.png`,
`evidence/run1-j01-06-bare-xvfb-root-all-black.png`.
