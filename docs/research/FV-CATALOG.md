# FV-CATALOG — The formal-verification catalog (journey → client → scenes → pass criteria)

- **Work order:** FV-001 (Wave 7, step 1 — the verification-design artifact)
- **Base (pinned):** `7f660c00407a5741eee975b2274570a200ff576b`
- **Date:** 2026-09-25
- **Law:** WAVE7-FV-WORK-ORDERS.md kernel addendum §4 (the catalog is the
  acceptance law), §1 (runtime truth), §2 (honest bounds), §5 (evidence
  schema), §7 (credentials), §8 (bounded deliveries). The shared acceptance
  law per journey: **discoverable → usable → observable → recoverable →
  truthful → evidenced** (AES L118-130).
- **Companion artifact:** [evidence/fv-001/FINDINGS-RECLASSIFICATION.md](evidence/fv-001/FINDINGS-RECLASSIFICATION.md)
  (every historical finding reclassified against this base)
- **Consumer:** FV-002 authors these scenes 1:1 (scene scripts under
  `scripts/fv/`, the Windows journey-smoke extension, the web formal-pass
  manifest); FV-003 executes them at the pinned SHA and writes
  `docs/research/evidence/fv-gate/`.

## 0. The three lanes and their driving disciplines

| Lane | The REAL artifact | Driving discipline | Evidence target root |
| --- | --- | --- | --- |
| **Linux desktop** | the release-profile `codexrs` binary built at the pinned SHA | the **d-series pattern** (the d24/d25/d26 precedents): LINUX_GUI_LAB (Xvfb + picom + lavapipe — the sealed `session.sh` recipe), **keyboard-only drives** (xdotool key events, no pointer except the wake click), **named moments**, frame captures (ffmpeg x11grab), md5 frame manifests, VLM-adjudicated reads, truthful-state assertions | `docs/research/evidence/fv-gate/linux/<scene-id>/` |
| **Windows desktop** | the `codexrs.exe` built by CI (windows-latest) | **CI-drivable depth only** (addendum §2): cargo gates (already green) + the launch smoke + **SendKeys scenes** — launch the process, drive chords/typed text via SendKeys, assert **window states** (process alive, window present/title, clean exit), capture what the runner can | `docs/research/evidence/fv-gate/windows/<scene-id>/` |
| **Web** | the real web build (`web/dist`) served by the REAL Rust gateway (`flauz-web-gateway`) in a real headless browser | the existing lab driver with the real transport: `node web/lab/journeys.mjs --gateway "<flauz-web-gateway> --web-root dist"` — screenshots + action logs + truthful-state assertions per journey (the parity-lab schema) | `docs/research/evidence/fv-gate/web/<scene-id>/` |

**Evidence schema (addendum §5, unchanged):** every run pins its SHA
(RUN.json lineage); every scene ships its script, frames/captures, action
log, assertions, and VLM reads archived with their frames; every script is
committed and re-runnable; scripts assume the warm Lead station
(`/home/z/parity-lab/build_env.sh`) but **hard-fail with a named message**
when prerequisites are missing; **no credential ever appears in scripts,
logs, URLs, or evidence** (§7). Mock-gateway captures are authoring
fallbacks only — formal evidence is real-gateway evidence (§1).

**Scene-row contract (FV-002's authoring spec):** every row below carries —
scene id · journey id · client lane · the named moments (d-series style) ·
the pass criteria (what a frame/assertion must show) · the evidence target
dir. Calibration comments in the scripts must be verified against the merged
source (the d26 discipline: every chord/anchor/copy string cited below was
read at the pinned base — see the citation column in the reclassification).

---

## 1. Linux desktop lane (FV-L## — 21 scenes)

Prerequisites (hard-fail named): the sealed session recipe (Xvfb + picom +
lavapipe + isolated HOME/XDG/CODEX_HOME), the pinned-SHA binary, the pinned
official CLI `0.146.0-alpha.3.1` via `CODEX_RS_CODEX_BIN` (isolated), the
donor-state fixture for seeded chats (the wo-p2-008 pattern), xdotool,
ffmpeg, the VLM reader at the gate station.

| Scene | Journey | Named moments | Pass criteria (frame/assertion must show) | Evidence dir |
| --- | --- | --- | --- | --- |
| **FV-L01** | J-01 | wake click → first frame; workspace navigation (Projects/Tasks · Reusable workflows · Artifacts · Activity); the title-bar palette entry (WO-P2-018); Ctrl+K palette; new-task entry; objective typed; the task surface lands; fresh-profile promo modal dismissed with ONE Escape (the UX-003 contract) | VLM reads: the four nav labels + the palette entry visible in chrome; the new-task objective echoed on the task surface; the promo modal (if fresh profile) gone after one Escape with focus landed (no swallowed keyboard — type lands in the composer) | `fv-gate/linux/fv-l01/` |
| **FV-L02** | J-02 | Ctrl+Alt+Shift+1 → the task-rail Context section; the honest "on its way" state; `/status` + Enter on the seeded chat → the status panel (Session row + live context remaining %) | VLM reads: "The context view is on its way" + its body verbatim; the status panel's Session row and the context-usage row; Escape closes both cleanly (frame returns to the task surface) | `fv-gate/linux/fv-l02/` |
| **FV-L03** | J-03 | Ctrl+Alt+Shift+R on an up-to-date task → the honest nothing-to-pick-up guidance; NO banner; palette row "Resume this task where it left off" lands the same guidance | VLM read: "This task is up to date — there's nothing to pick up right now." + [Dismiss] via the command-status line; the task-surface frame is banner-free; the palette row filters and lands the identical guidance (frame-compare) | `fv-gate/linux/fv-l03/` |
| **FV-L04** | J-03 | kill the supervised app-server child mid-session → footer retry cadence; let a retry succeed → auto-return online; kill again → timer restarts at attempt 1 | Frames at each cadence step: "Reconnecting…" → "Retry 1 in 1s" → … → "Retry N in 20s" (cap holds) → "App-server online" → the NEXT loss starts at attempt 1/1 s; log extract archived; the seeded-session bound named in the run notes (G-6) | `fv-gate/linux/fv-l04/` |
| **FV-L05** | J-04 | Ctrl+Alt+Shift+6 → the capability panel; the honest pre-wiring copy naming the five dimensions; scoped Escape; palette row "Why is a capability unavailable?" lands the SAME panel | VLM reads: the panel subtitle "What a task can do — and what's missing when it can't" + the no-silent-disappearance copy; the chord frame and the palette-row frame are pixel-identical; Escape returns to the task surface (frame-compare) | `fv-gate/linux/fv-l05/` |
| **FV-L06** | J-05/J-06 guards | entry surface (no chat): Ctrl+` → honest terminal status; Ctrl+T → honest browser status; Ctrl+Alt+Shift+3 → the environments rail's honest empty state | VLM reads: "Select a task before opening a terminal." / "Open a chat before opening the Browser." as bottom-banner statuses (bounded, dismissible); "No environments attached" + "Environments are where work actually happens…" verbatim; zero silent frames (every guarded chord changes the frame) | `fv-gate/linux/fv-l06/` |
| **FV-L07** | J-06 (chat-scoped slice) | seeded chat selected: Ctrl+` → dock opens with a live tab; the PTY focus transfer (typed keys land in the PTY, not the composer); browser panel honest state; timeline visible. Bounded: no live turn (the auth wall — gap L-1) | Frames: the dock with a tab + a prompt; a typed echo frame (the N5 contract observable); the browser panel's honest state; run notes name the auth bound | `fv-gate/linux/fv-l07/` |
| **FV-L08** | J-07 | Ctrl+Alt+Shift+7 → the agents view; the honest empty + parallel-work body + the not-wired line; palette row "See who is working on this task" lands the same panel | VLM reads: "Agents on this task" + "No extra agents on this task yet" + the delegation body + "Live agent progress is on its way … No agent is invented here…" verbatim; chord frame == palette frame (pixel-identical) | `fv-gate/linux/fv-l08/` |
| **FV-L09** | J-08 | Ctrl+Alt+Shift+Y → the needs-you panel; the approval card with BOTH consequences; the takeover affordance + handback copy; the cancellation affordances with downstream truth; the honest quiet state; scoped Escape | VLM reads: "Nothing needs you right now" + what-lands-here copy; the approval card's consequence pair; "Take over this step" + the state-preserved note; "Hand back when you're done"; one Escape closes (the 017 restore contract) | `fv-gate/linux/fv-l09/` |
| **FV-L10** | J-09 | the rail Evidence section (Ctrl+Alt+Shift+4) honest empty; the seeded chat timeline; Ctrl+F Find positive path; Copy as Markdown export | VLM reads: the evidence empty state verbatim ("No evidence collected yet" + the claims-vs-verified body); the Find bar + counter "N / M+ results" + highlighted matches + advance (the non-color distinction — C-17 decided here); the markdown export contains the seeded turn | `fv-gate/linux/fv-l10/` |
| **FV-L11** | J-10 | the task-surface affordance "☆ Save as a reusable workflow"; Ctrl+Alt+Shift+S → the save panel; the honest not-wired state; scoped Escape | VLM reads: the affordance + its description ("Flauz can remember the successful steps…"); "Saving isn't wired to live tasks yet" + "the steps that actually ran — exactly what happened, nothing aspirational" + the next-step + "Escape closes this panel" verbatim | `fv-gate/linux/fv-l11/` |
| **FV-L12** | J-11 | Ctrl+Alt+2 → the Reusable workflows surface; the honest empty; the sidebar highlight; the palette row | VLM reads: "Reusable workflows" + "Successful steps, saved so you can run them again" + "No reusable workflows yet" + the body + the what-to-do-next; the sidebar's Workspace section highlights the surface | `fv-gate/linux/fv-l12/` |
| **FV-L13** | J-12 (honest-state slice) | the save panel's deviation promise + the list's empty state — the full run/deviation/improve loop is N/A (not wired; named) | VLM reads: the not-wired copy naming that deviations/improvement arrive with the wiring; the run notes name the N/A reason (cross-lane gap X-2) | `fv-gate/linux/fv-l13/` |
| **FV-L14** | J-13 | Ctrl+Alt+Shift+U → the members panel ("Just you" + both not-wired states); palette "Change a task's sharing" → the sharing surface (Files/Remembered notes/Work products + the honest sync note); scoped Escape both | VLM reads: "Who is on this workspace" + "Just you" + "Inviting isn't connected yet … nothing is sent today" + "Changing roles isn't connected yet"; the sharing surface's three sections + "Change sharing"; both panels close on one Escape | `fv-gate/linux/fv-l14/` |
| **FV-L15** | J-14 | Ctrl+Alt+Shift+M → the model picker; the honest empty; the identity promise; (donor-state, when seeded) a model switch keeps the task; the composer model selector | VLM reads: "No models connected yet" + what-a-model-provides + the Settings→Connections next-step + "Switching models keeps this task — its objective, its plan, and everything it produced — exactly as it is"; donor-state: the task surface is unchanged across the switch (frame-compare of the task identity elements) | `fv-gate/linux/fv-l15/` |
| **FV-L16** | J-15 | seeded git chat: the "Continue in" footer → "New worktree" → the fork explanation → the worktree created (the task continues in the fork); the environments rail honest state | Frames: the picker with "Work locally" checked + "New worktree" + "Create a copy of your local project to work in parallel"; the post-fork task surface; run notes record the fork state transfer (bounded: the 20 MiB transfer contract is unit-anchored) | `fv-gate/linux/fv-l16/` |
| **FV-L17** | J-16 | Ctrl+Alt+Shift+L → the conflicts panel; the view-model states (Free / held-with-expiry / queue position / the escalation card with consequences); scoped Escape | VLM reads: the panel title + "Free" states + "2 tasks waiting — yours is next" (view-model) + the escalation card's decision affordances each stating its consequence; the not-wired note (the live lease state is future wiring) | `fv-gate/linux/fv-l17/` |
| **FV-L18** | J-17 | the unread flow: Ctrl+Shift+U mark → the dot + label → Ctrl+Alt+A jump (visit clears) → Shift+Escape clear-all honest count; Ctrl+Alt+U → the Activity view; the needs-you extension rows | VLM reads: "Chat marked unread" → the dot on the row → the jump lands selection on the flagged chat with the dot cleared → "Cleared unread indicators for 1 chat"; the Activity view renders with keyboard navigation (up/down/enter/escape — the ActivityView rows); a needs-you-kind row shape when seeded | `fv-gate/linux/fv-l18/` |
| **FV-L19** | DOMAIN-NEUTRAL (J-01/J-14/J-09 composite, non-code) | a garden-research task: new task "Research seasonal planting calendars for the community garden" (no code); the model picker visit; the plan/status surfaces; the artifacts/evidence honest states; the save affordance visible post-success-path | VLM reads: the non-code objective echoed on the task surface; every visited surface answers in domain-neutral language (no repo/branch/test vocabulary on these surfaces — PJ §6); the honest states verbatim | `fv-gate/linux/fv-l19/` |
| **FV-L20** | A11Y (the RG-A11Y-01/02 re-run at current main) | keyboard-only across the six F1 surfaces (palette → settings → terminal → browser → history → attention), no pointer; the focus probes: palette close restore (typed probe lands in composer), shortcuts-overlay two-Escape contract, PTY focus transfer, the reachable modal traps, Ctrl+P no-workspace honest status, the guarded-chord ladder (every advertised chord answers) | Frames + typed-probe reads: each surface opens/closes keyboard-only; the typed probe lands with caret in the composer after palette/overlay close; first Escape clears the overlay query, second closes; PTY receives typed keys on open; each guarded chord produces its honest status (zero silent frames — the A-3 verdict re-proven); the chord ladder covers the Flauz panel family (M/6/R/P/S/U/L/Y/7 + rail 1..5 + nav 1..4) | `fv-gate/linux/fv-l20/` |
| **FV-L21** | FR/PKG slice (J-01 adjacent — the fresh-machine contract) | FR-1: Help → "About codexRS" (380×360, centered, version + copyright, refocus, OK/Escape/native close); FR-3: the empty-workspace "Open folder" offer + the honest no-workspace guard; PKG-2: `--install-desktop-entry` creates the per-user entry for the extracted binary and never overwrites an existing entry | xwininfo: Width 380 / Height 360 exactly; VLM: the version string of THIS build; the entry file created (and unchanged on the second run); the picker stays the documented headless bound (gap L-4) | `fv-gate/linux/fv-l21/` |

**Adjacent release-gate references (not journey scenes):** the RG-SOAK
battery (G-1..G-7) and RG-RECONNECT-05..07 remain Lead-station release
scenes per the f1-sweep release-gate inventory; FV-003 records their status
at the pinned SHA in the gate record without folding them into the journey
catalog (the soak needs the 4 h Lead time budget — gap L-9).

---

## 2. Windows desktop lane (FV-W## — 12 scenes + the standing gates)

Prerequisites (hard-fail named): the CI windows-latest runner; the built
`codexrs.exe`; a clean temp profile (isolated `CODEX_HOME`/`CODEX_RS_DATA_DIR`);
the SendKeys driver (FV-002's `scripts/windows_fv_smoke.ps1` + its
release.yml/ci.yml wiring — ADDITIVE steps only). Depth law: **SendKeys +
window-state assertions only** (addendum §2); everything beyond is a named
gap (§5 below), never silently narrowed.

**Standing gates (already in CI; FV-003 records them at the pinned SHA):**
`cargo fmt --all --check` · `cargo clippy --workspace --all-targets` ·
`cargo test --workspace` on the double matrix (ci.yml:24) · the launch
smoke (`scripts/windows_desktop_smoke.ps1` via release.yml:194).

| Scene | Journey | Named moments | Pass criteria (window-state assertion + capture) | Evidence dir |
| --- | --- | --- | --- | --- |
| **FV-W01** | J-01 | launch → main window present; SendKeys Ctrl+N → the new-task surface; type a short objective; Enter | process alive; main window handle present through the drive; a capture of the new-task surface; the objective echoed (capture); no crash, clean state at scene end | `fv-gate/windows/fv-w01/` |
| **FV-W02** | J-02 | type "/status" + Enter → the composer status panel; Escape closes | window stays responsive (message pump alive — the panel opens and closes); capture of the panel state; Escape returns to the task surface (capture) | `fv-gate/windows/fv-w02/` |
| **FV-W03** | J-03 (retryable startup) | launch with `CODEX_RS_CODEX_BIN` pointing at a missing binary → the failure surface; window stays up ≥ 15 s; relaunch with a valid runtime → the online footer | process alive ≥ 15 s under the missing runtime (no crash — the graceful-degradation contract); capture of the failure surface; the relaunch reaches the online state (capture); exit code 0 on clean CloseMainWindow | `fv-gate/windows/fv-w03/` |
| **FV-W04** | J-04 | Ctrl+Alt+Shift+6 → the capability panel; Escape | panel open/close window states (captures); no crash; the scoped-Escape close returns to the task surface | `fv-gate/windows/fv-w04/` |
| **FV-W05** | J-14 | Ctrl+Alt+Shift+M → the model picker; Escape | panel open/close captures; the honest empty state visible in the capture; no crash | `fv-gate/windows/fv-w05/` |
| **FV-W06** | J-17 | Ctrl+Alt+U → the Activity view; Ctrl+Alt+4 → the Activity surface; Escape | both surfaces open/close (captures); keyboard rows answer (the ActivityView context); no crash | `fv-gate/windows/fv-w06/` |
| **FV-W07** | J-10/J-11 | Ctrl+Alt+Shift+S → the save panel; Ctrl+Alt+2 → the Reusable workflows surface; Escape each | panel/surface open/close captures; the honest not-wired/empty states visible; no crash | `fv-gate/windows/fv-w07/` |
| **FV-W08** | J-13 | Ctrl+Alt+Shift+U → the members panel; Escape | panel open/close captures; the "Just you" state visible; no crash | `fv-gate/windows/fv-w08/` |
| **FV-W09** | J-16/J-08 | Ctrl+Alt+Shift+L → the conflicts panel; Ctrl+Alt+Shift+Y → the needs-you panel; Escape each | both panels open/close (captures); the honest quiet states visible; no crash | `fv-gate/windows/fv-w09/` |
| **FV-W10** | J-07 | Ctrl+Alt+Shift+7 → the agents view; Escape | panel open/close capture; the honest empty visible; no crash | `fv-gate/windows/fv-w10/` |
| **FV-W11** | DOMAIN-NEUTRAL | type the non-code objective "Research seasonal planting calendars for the community garden"; Enter; visit the model picker | the task surface shows the non-code objective (capture); the visited surfaces render domain-neutral language (capture); no crash. Bounded: no live turn (gap W-5) | `fv-gate/windows/fv-w11/` |
| **FV-W12** | A11Y (bounded depth) | keyboard-only drill on the entry surface: Tab/Shift+Tab cycles; Ctrl+K palette → Escape restores; Ctrl+/ overlay → two-Escape contract; a guarded chord (Ctrl+P no-workspace) answers honestly | window states + captures: the palette/overlay open and close; the second overlay Escape closes it; the guarded chord produces a visible status (capture); no crash, no swallowed keyboard (a typed probe lands after each close) | `fv-gate/windows/fv-w12/` |

---

## 3. Web lane (FV-E## — the real-gateway formal pass)

Prerequisites (hard-fail named): the built gateway binary
(`target/…/flauz-web-gateway`), the real web build (`web/dist`), the
pinned CLI for the supervised app-server (isolated `CODEX_HOME`),
playwright chromium, the driver `web/lab/journeys.mjs` with
`--gateway` (the real transport — mock captures are NOT formal evidence,
addendum §1). FV-002 freezes these as `web/lab/fv-manifest.json`.

| Scene | Journey | Named moments | Pass criteria (assertions + captures) | Evidence dir |
| --- | --- | --- | --- | --- |
| **FV-E00** | gateway security slice (W6 I-11/I-12) | the gateway boots with the default localhost bind; `GET /healthz` ok; `GET //etc/passwd` → 404 refusal; the SPA fallback serves only relative in-app routes | config assert: bind is `127.0.0.1:8610` (or the run's documented override); healthz 200; the traversal probe returns the named 404 (never file content); a relative route serves the shell | `fv-gate/web/fv-e00/` |
| **FV-E01** | J-01 | the j-01 drive (real gateway): sign in → workspace home → Start a new task → objective → live session with success state; palette fallback | the existing j-01 assertions green ×3 runs (the w6 convention): connection.state connected; the start entry visible; the session created; palette rows present | `fv-gate/web/fv-e01/` |
| **FV-E02** | J-02 | the j-02 drive: the context surface — what the session knows, the honest bounds | j-02 assertions green ×3; the context surface renders protocol-truthful state | `fv-gate/web/fv-e02/` |
| **FV-E03** | J-03 | the j-03 drive: the four-state connection machine + reconnect (kill/ restart the gateway's supervised runtime; the banner + retry + recovery) | j-03 assertions green ×3: the truthful connection states + the session-load retry path | `fv-gate/web/fv-e03/` |
| **FV-E04** | J-04 | the j-04 drive: the capability-gap surfaces — the named protocol gaps with recovery paths (never fabricated) | j-04 assertions green ×3: the gap cards name the missing protocol surface + the recovery path | `fv-gate/web/fv-e04/` |
| **FV-E05** | J-05 | the j-05 drive: the environments panel — the named environment-listing gap + the per-turn `cwd` the protocol DOES carry | j-05 assertions green ×3: the gap card named; the cwd control applies per-turn | `fv-gate/web/fv-e05/` |
| **FV-E06** | J-06 | the j-06 drive: cross-surface work — composer → turn → items → artifacts with evidence links | j-06 assertions green ×3 | `fv-gate/web/fv-e06/` |
| **FV-E07** | J-08 | the j-07 drive — **journey J-08** (takeover/approval/cancellation): the aria-live approvals queue; a decided approval kept as a NAMED record; turn interrupt with named cancelled state | j-08 assertions green ×3: the approvals surface + the decided-record honesty + the cancellation propagation reason | `fv-gate/web/fv-e07/` |
| **FV-E08** | J-09 | the j-09 drive: the artifacts/items panel over real turn items | j-09 assertions green ×3 | `fv-gate/web/fv-e08/` |
| **FV-E09** | J-13 | the j-13 drive: the collaborators panel — the membership/presence protocol gap NAMED with its recovery path | j-13 assertions green ×3: the gap card + the honest empty | `fv-gate/web/fv-e09/` |
| **FV-E10** | J-14 | the j-14 drive: the per-turn model choice riding the real `turn/start` fields | j-14 assertions green ×3: the chosen model reported + applied per-turn | `fv-gate/web/fv-e10/` |
| **FV-E11** | J-15 | the j-15 drive: the per-turn `cwd` environment switch (the protocol-carried slice) | j-15 assertions green ×3 | `fv-gate/web/fv-e11/` |
| **FV-E12** | DOMAIN-NEUTRAL | the j-domain-neutral-research drive: the garden-planning research task — model at compose time, skill reference, produced items, the honest collaboration gap | the existing domain-neutral assertions green ×3 (the shared acceptance law on a non-code workflow) | `fv-gate/web/fv-e12/` |
| **FV-E13** | A11Y | the j-a11y-responsive drive: keyboard-complete surfaces (tab/enter/escape + focus restoration), aria-truthful states (aria-pressed toggles, aria-live approvals), mobile + desktop captures | the existing a11y assertions green ×3; mobile + desktop frame sets archived | `fv-gate/web/fv-e13/` |

---

## 4. The N/A register (named reasons — addendum §4's "or a named N/A reason")

| Journey | Lane | N/A reason (named) |
| --- | --- | --- |
| J-05 (full provider authorize/attach) | Windows | beyond CI-drivable depth (account/OAuth flows need an interactive session) — the Linux lane's FV-L06 honest-state slice + the web FV-E05 gap card carry the verifiable truth |
| J-06 (full cross-boundary coordination) | Windows | needs live turns (no authenticated runtime on the runner — gap W-5); the guard slices verify FV-W12 |
| J-07 | Web | the web client has no agents surface — the app-server protocol carries no agent namespace (the capability engine derives availability from the generated constants; a named gap, not a fabrication) |
| J-08 (live approval runtime) | Windows | needs a live turn requesting approval (auth wall); the needs-you panel's honest states verify instead (FV-W09) |
| J-09 (deep evidence over live turns) | Windows | needs live turns; the honest evidence states verify instead |
| J-10/J-11/J-12 | Web | the web client has no Procedures surface (the protocol does not carry the workflow/procedure family for the web session model); the desktop lanes carry the library surfaces |
| J-12 (full run/deviation/improve loop) | ALL | not wired on any client (the save flow's honest not-wired state is the current truth — reclassification I-5/D-12); the honest-state slices (FV-L13) verify what exists; no scene invents deviation data |
| J-15 (full worktree fork flow) | Windows | beyond CI-drivable depth (repo fixture + fork serialization flow); the Linux lane (FV-L16) carries it |
| J-16 | Web | the web client has no conflicts surface (no lease/protocol namespace); the desktop lanes carry the J-16 panels |
| J-17 | Web | the web client has no Activity/attention surface (protocol carries no attention stream for the web session); the desktop lanes carry J-17 |
| J-18 | ALL | no repeatability-observation surface exists on any client (reclassification D-18); future wiring (roadmap F6+ backlog); nothing to verify — no scene is authored |
| macOS / Mobile lanes | — | deferred until after the production gate (the 2026-09-23 sequencing amendment); out of FV scope by law |

## 5. The FV gap list (addendum §2 — the honesty surface)

What this environment CANNOT formally verify, per lane, with the honest
reason and the recovery path. Never silently narrowed; never fabricated.

### Linux lane

| Gap ID | Bound | Honest reason | Recovery path |
| --- | --- | --- | --- |
| L-1 | The auth wall — no authenticated Codex account in the lab | all live-turn journeys (streaming timeline, live approvals, live model switches, live agent turns) cannot run end-to-end; every such scene is bounded to honest-state slices + unit anchors | an authenticated operator session at the Lead station (credentials stay in the secure env file — §7; never in evidence) or the donor-state pattern for seeded surfaces |
| L-2 | Single 1600×1000 Xvfb screen | no multi-monitor, no HiDPI/scaling verification | a multi-head Xvfb config or real hardware at the Lead station |
| L-3 | Software rendering only (lavapipe) | real-GPU rendering (blade/Vulkan on hardware) unverified | a GPU-equipped host |
| L-4 | X11 only | pure-Wayland (portal path) untested — the platform doc keeps it future work | a Wayland session host |
| L-5 | No native folder picker on Xvfb | no portal backend in the lab image (FR-3's picker step is the documented bound) | a portal-backed lab image |
| L-6 | The four flow-gated 019 modals | remote pairing / remote confirmation / account logout / plugin install need their gating flows; they stay unit-pinned | flow-gated runtime paths at the Lead station (an authenticated or flow-seeded session) |
| L-7 | Desktop-environment matrix | Ubuntu CI + Debian lab only; broader desktop-environment smoke pending (KF B-10) | the broader smoke matrix (a Lead release-stage item) |
| L-8 | VLM adjudication is Lead-side | the worker/lab archives frames + prompts; the reads happen at the gate station (the flauz-lab `VlmSlot` law) | the Lead gate pass (FV-003) — by design, not a defect |
| L-9 | Soak/perf depth | the ≥ 4 h RG-SOAK battery needs the Lead station time budget; it stays a release-gate scene adjacent to the journey catalog | the Lead release pass at the pinned SHA (recorded in the FV gate record, not the journey set) |

### Windows lane

| Gap ID | Bound | Honest reason | Recovery path |
| --- | --- | --- | --- |
| W-1 | No Windows GUI host in this environment | WINDOWS_GUI_LAB unavailable (E2B §1: Linux microVMs only; no Windows host/VM/service credentials) — the platform bound the Wave-7 addendum §2 names | a Windows GUI host or a Windows operator session; until then the lane's formal depth stays CI-bounded (this catalog's design) |
| W-2 | CI-drivable depth only | the runner drives SendKeys + window-state assertions; no real mouse journeys, no deep in-runner visual VLM adjudication, no multi-monitor/DPI, no DirectComposition/animated-cursor visual verification (the Computer Use P2 Windows tail), no native notification/tray driving | a Windows GUI host (the same host closes L-2/L-3's Windows analogues) |
| W-3 | Ephemeral clean profile | no persistent profile across runs; fresh-machine FR/PKG depth lives in release.yml's archive smoke | a persistent Windows operator profile (post-CI stage) |
| W-4 | Toolchain bound (process, not product) | the worker sandbox lacks the Rust toolchain; the Lead executes the Windows gates at the integration station (the Wave-5/6 precedent) | the Lead gate pass — by design |
| W-5 | No authenticated runtime on the runner | live-turn journeys (J-06 full, J-08 live approvals, J-09 deep) cannot run | an authenticated runtime at a Windows operator station; meanwhile the honest-state slices carry the truth |
| W-6 | Windows-only runtime surfaces under load | Job Object teardown under load, native notifications, tray behavior are not journey-drivable at SendKeys depth | a Windows GUI host with load scenarios |

### Web lane

| Gap ID | Bound | Honest reason | Recovery path |
| --- | --- | --- | --- |
| E-1 | The supervised runtime is unauthenticated in the lab | the real gateway supervises a real `codex app-server` (isolated CODEX_HOME, pinned CLI, no credentials) — live-turn depth is bounded to honest states, exactly like Linux L-1 | an authenticated runtime at the Lead station |
| E-2 | Chromium headless only | WebKit/Gecko engines unverified | extend fv-manifest.json to the other engines (a Lead-stage item) |
| E-3 | Mobile is emulated | responsive captures are emulated viewports in headless chromium, not real devices | real-device passes post-production (the mobile client's own phase) |
| E-4 | J-13 collaboration depth | membership/presence methods are absent from the app-server protocol surface — a NAMED gap card with a protocol-work-order recovery path (never a fabrication) | additive protocol work orders (deviation-gated seams) in a future wave, per the W6 record |
| E-5 | Localhost only | the gateway binds 127.0.0.1:8610 by default; real-network degradation (latency/loss/proxies) unverified | a staged network profile at the Lead station |
| E-6 | Concurrent multi-client sessions | two simultaneous clients on one workspace (the J-13 two-actor depth) are not drivable — single-session supervision + the presence protocol gap | the J-13 protocol path (E-4) + multi-session gateway wiring (future wave) |

### Cross-lane

| Gap ID | Bound | Honest reason | Recovery path |
| --- | --- | --- | --- |
| X-1 | J-18 (automation discovery) | no surface on any client (reclassification D-18) | the F6+ wiring backlog; a future wave's catalog adds the scenes |
| X-2 | The full J-12 loop | the run/deviation/improve loop is not wired on any client (honest not-wired states only) | the harness-wiring waves; the catalog grows with the wiring |
| X-3 | The live orchestration wiring | the agents/save/conflicts/needs-you/providers/members panels render honest view-models; live data paths are future waves (reclassification I-1..I-3, I-5..I-7) | the wiring waves; FV scenes never invent data — they verify the honest states |
| X-4 | macOS / Mobile | deferred until after the production gate (the sequencing amendment) | their own post-production formal-verification waves |

---

## 6. Catalog coverage table (J-01..J-18 × lanes — the acceptance-criterion summary)

| Journey | Linux desktop | Windows desktop | Web |
| --- | --- | --- | --- |
| J-01 Start any project | FV-L01 | FV-W01 | FV-E01 |
| J-02 Understand what the agent knows | FV-L02 | FV-W02 | FV-E02 |
| J-03 Recover/continue | FV-L03 + FV-L04 | FV-W03 | FV-E03 |
| J-04 Discover a capability gap | FV-L05 | FV-W04 | FV-E04 |
| J-05 Add another environment/site | FV-L06 (honest-state slice) | N/A (§4: CI depth) | FV-E05 |
| J-06 Coordinate browser+terminal+sandbox | FV-L06 + FV-L07 (bounded) | N/A (§4: no live turns) | FV-E06 |
| J-07 Parallelize work | FV-L08 | FV-W10 | N/A (§4: no surface) |
| J-08 Human takeover / handoff | FV-L09 | FV-W09 (honest states) | FV-E07 |
| J-09 Understand what happened | FV-L10 | N/A (§4: no live turns) | FV-E08 |
| J-10 Save as reusable workflow | FV-L11 | FV-W07 | N/A (§4: no surface) |
| J-11 Discover a learned Procedure | FV-L12 | FV-W07 | N/A (§4: no surface) |
| J-12 Reuse/improve a Procedure | FV-L13 (honest-state slice) | N/A (§4) | N/A (§4) |
| J-13 Collaborate on one task | FV-L14 | FV-W08 | FV-E09 |
| J-14 Switch model | FV-L15 | FV-W05 | FV-E10 |
| J-15 Switch execution environment | FV-L16 | N/A (§4: CI depth) | FV-E11 |
| J-16 Inspect resource conflicts | FV-L17 | FV-W09 | N/A (§4: no surface) |
| J-17 Review activity needing the human | FV-L18 | FV-W06 | N/A (§4: no surface) |
| J-18 Discover automation | N/A (§4: no surface) | N/A (§4) | N/A (§4) |
| Domain-neutral (per client) | FV-L19 | FV-W11 | FV-E12 |
| A11Y (per client) | FV-L20 | FV-W12 | FV-E13 |
| Fresh-machine/packaging slice | FV-L21 | (release.yml archive smoke — standing) | FV-E00 (gateway security slice) |

**Arithmetic:** 18 journeys × 3 lanes = 54 cells → 33 scene cells +
21 N/A cells (each with its named reason in §4). Linux 21 scenes,
Windows 12 scenes + standing gates, Web 14 scenes (incl. FV-E00).
Every J-01..J-18 appears on at least one lane (J-18 appears as N/A-everywhere
with the named no-surface reason — the honest bound, per addendum §4).

## 7. FV-002's authoring notes (the 1:1 contract)

1. **One script per Linux scene** under `scripts/fv/` (the d26 pattern:
calibration comments verified against the merged source — every chord,
anchor, and copy string in §1 was read at the pinned base and is cited in
the reclassification; keyboard-only drives; named moments; frame captures;
hard-fail named prerequisites).
2. **The Windows journey-smoke extension** (`scripts/windows_fv_smoke.ps1`
+ additive ci.yml/release.yml steps) implements §2's scenes as SendKeys
drives with window-state assertions; capture what the runner can (screenshots
via the .NET capture API where available), assert what it must (process/window
states), and hard-fail with named messages.
3. **The web formal-pass manifest** (`web/lab/fv-manifest.json`) freezes §3's
scenes as runner inputs: journey id, gateway mode=real (the `--gateway`
command), the per-journey assertion set (the existing lab assertions), ×3
runs (the w6 convention).
4. **No product changes** (addendum §6): the harness is the only code this
wave adds; FV-001 changed none (docs-only, git-diff-proven).
5. **Defects found during FV-003** become focused fix work orders through
the synchronization law (defect → focused PR → merge → fresh environment →
rerun the affected journey) — the wave verifies, it does not fix.
