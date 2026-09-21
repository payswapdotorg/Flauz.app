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

### Run 2 — reproducibility proof (2026-09-21, E2B sandbox `ivmy0lvttlc2csmbklaay`, Flauz @ `f66965e`, full cold reprovision in 460 s)

Fresh sandbox after the first expired (E2B pause-cleanup — the harness's
destroy-and-recreate contract exercised for real): same build, same
launch contract, same symptom — window maps at the identical geometry
(1278×818 @ +10+85) and `xwd` window content is uniform black
(mean RGB 0,0,0, std 0; `evidence/run2-j01-02-fresh-sandbox-xwd-black.png`).

| Journey | Verdict |
| --- | --- |
| E-00 fresh desktop → build → launch | **WORKS** (460 s cold; fully reproducible) |
| J-01 cold start | **BROKEN (P0, L-002 reproduced)** |

### Run 3 — L-002 root-cause discrimination + in-lane fix validation (2026-09-21, E2B sandbox `io23l6hz0z2g7mqoq54om`, Flauz @ `f66965e` + one-line gpui visual patch + native codex binary)

Environment additions: full Vulkan diagnostics (`vulkaninfo`, ICD
inventory, `VK_LOADER_DEBUG=all`), `strace` syscall profiling, `vkcube`
present-path control, VLM-verified screenshots. Root cause of L-002
confirmed: GPUI 0.2.2 X11 visual selection prefers the 32-bit ARGB
transparent visual; on the lavapipe software-Vulkan stack every presented
frame is alpha=0 (fully transparent). Controls: `vkcube` (24-bit visual)
presents opaque pixels correctly on the identical display/stack — the
environment is capable; the app's visual choice is the defect. Secondary
findings: L-004 (codex npm wrapper crashes on Node 12 → app-server never
spawns → ~250 Hz silent retry spin) and L-005 (no logging backend —
GPUI diagnostics invisible).

| Journey | Verdict | Detail |
| --- | --- | --- |
| E-00 environment discrimination | **WORKS** (evidence-gathering) | Xvfb `-retro` + xfwm4 + lavapipe ICD set; vkcube control renders (LunarG cube visible, VLM-verified). |
| E-00 app-server lifecycle | **BROKEN → FIXED (in-lane)** | Pre: `codex app-server` dies instantly (Node 12 vs top-level await), 250 Hz retry spin, no error state shown. Post (native binary): app-server alive, `App-server online` shown in UI sidebar. |
| J-01 Start a project (cold start) | **WORKS (with in-lane patch)** | Full product shell renders: sidebar (New chat / Repository / Pull requests / Plugins / Workflows / Projects / Chats / Terminal / Browser / Settings + App-server online), "What should we work on?" empty state, composer with model/effort/mode selectors, onboarding modal (click-dismissible), honest signed-out auth gate (Sign in with ChatGPT / device code / API key / Bedrock). **Real interaction verified:** composer keyboard input (typed text + blinking cursor), Ctrl+Shift+P command palette (search, keyboard hints, sections), palette-driven navigation to Settings → Appearance (Back button, category sidebar, theme preview cards). Window own-content `xwd` mean 246.3 (pre-fix 0.0). |
| J-02 … J-18 | **NOT-RUN (blocked → unblocked, pending re-run against patched build)** | Cold-start blocker removed; the full battery re-runs after LAB-003 merges the fix to `main`. |

Evidence: `evidence/run3-l002-defect-wallpaper-through.png` (pre-fix,
composited), `evidence/run3-vkcube-control.png` (Vulkan present control),
`evidence/run3-l002-fix-inherit-visual.png` (post-fix full UI),
`evidence/run3-j01-composer-keyboard.png`,
`evidence/run3-j01-command-palette.png`,
`evidence/run3-j01-settings-appearance.png`,
`evidence/run3-appserver-live-transparent-still.png` (app-server fixed
but still transparent pre-visual-patch — the two defects are independent).

### Run 4 — signed-out J-02..J-18 sub-battery (2026-09-21, E2B sandbox `izqinqfuxeylk815bn5sf`, Flauz @ `f66965e` + auto lane patch + native codex binary, cold reprovision 469 s)

The patched-build battery ran as far as the honest signed-out gate allows.
Codex auth remains operator-gated: every journey that needs live model
execution is honestly UNAVAILABLE-BUT-HONEST until a one-time operator
sign-in inside E2B. Interaction methodology upgraded: pixel-row scanning +
bounding-box cross-checks before every click (run-3's L-006 "dead click"
was a coordinate miss — see the correction below).

| Journey | Verdict | Detail |
| --- | --- | --- |
| J-01 Start a project (re-verify) | **WORKS** | Full shell renders on the auto-patched build (469 s cold provision, fully reproducible). Promo modal "Introducing GPT-5.6-Sol" at cold start: primary button dismisses + switches model to GPT-5.6-Sol (honest action) but there is NO secondary dismiss affordance (L-012, P3). |
| J-02 Understand what the agent knows (Context) | **UNAVAILABLE-BUT-HONEST (auth-gated)** | No Context surface is reachable signed-out (no task exists); surfaces that would host it (composer, task views) show the honest sign-in gate. Deeper test needs Codex auth (operator-gated). |
| J-03 Recover a task | **WORKS-WITH-DEFECTS** | Sidebar expanded state persists across full restart (positive), but typed composer draft is LOST with no resume prompt (L-010, P2). In-task resume needs auth (operator-gated). |
| J-04 Discover a capability gap | **WORKS-WITH-DEFECTS** | Sidebar sweep, pixel-verified clicks: Repository → honest empty state; Pull requests → honest dependency error + actionable buttons; Plugins → marketplace renders; Workflows → raw protocol error (L-008, P2); Terminal → honest toast "Select a task before opening a terminal."; Browser → honest toast "Open a chat before opening the Browser." (L-006 corrected to P3 wording pass: task vs chat vocabulary). Toasts never auto-dismiss (L-011, P3). |
| J-05 Add an environment / project | **SILENT-NO-OP (P2)** | The "Projects +" affordance does nothing visible signed-out — no dialog, no form, no toast (L-009). Inconsistent with "New chat"'s honest sign-in gate. Environment-adding surfaces live in Settings (Plugins/MCP servers/Browser/Computer use/Connections) — reachable and rendering; creating real environments needs auth. |
| J-06..J-10, J-12, J-13, J-16 | **UNAVAILABLE-BUT-HONEST (auth-gated)** | Multi-surface coordination, parallel agents, human takeover, evidence states, reusable workflows, collaboration, resource conflicts all require a live model-backed task. The signed-out gates are honest (sign-in card with 4 auth paths; task-required toasts). |
| J-11 Discover saved work later (cold-start discovery paths) | **WORKS** | ≥2 discovery paths verified: (1) persistent sidebar (all sections), (2) command palette with full inventory + working search (Suggested: New chat Ctrl+N / Open folder Ctrl+O; Settings pages; Thread: New standalone chat Ctrl+Alt+O; Navigation: Search chats Ctrl+G / Back Ctrl+[ / Forward Ctrl+]), (3) keyboard shortcuts, (4) first-run promo modal. Palette search "sidebar" → "Toggle sidebar (Ctrl+B)" works. |
| J-14 Switch model | **WORKS-WITH-DEFECTS** | Discovery path verified: promo modal switches composer model to GPT-5.6-Sol (honest, visible in composer selector). Model/effort/mode selectors present in composer. State preservation across task switch needs auth (operator-gated). |
| J-15 Switch environment | **UNAVAILABLE-BUT-HONEST (auth-gated)** | No environment exists to switch signed-out; settings surfaces for environments render. |
| J-17 Human attention (Activity) | **MISSING (signed-out surface)** | No Activity surface found in sidebar or palette inventory (no "Activity" entry). Either activity surfaces are chat/task-scoped (auth-gated) or not yet implemented in this build — needs signed-in verification before final classification. |
| J-18 Discover automation | **UNAVAILABLE-BUT-HONEST (auth-gated)** | Workflows surface exists (Teach a workflow / Published versions / Durable instances) — the discovery path renders; using it needs auth, and its signed-out state shows the raw error (L-008). No silent auto-scheduling observed. |

Sidebar state defect: L-007 (P3) — accidental collapse possible (toggle
adjacent to "Back to app"), no visible restore affordance (Ctrl+B and the
palette command work; the empty-state copy references the hidden sidebar).

Evidence: `evidence/run4-j01-launch.png`, `run4-modal-try-sol.png`,
`run4-j04-repository.png`, `run4-j04-pullrequests.png`,
`run4-j04-plugins.png`, `run4-j04-workflows.png`,
`run4-j04-settings.png`, `run4-j04-terminal-toast.png`,
`run4-j04-browser-toast.png`, `run4-j05-projects-plus-silent.png`,
`run4-j03-draft-typed.png`, `run4-j03-restart-draft-lost.png`,
`run4-j11-palette.png`, `run4-j11-palette-scrolled.png`,
`run4-j11-palette-sidebar.png`, `run4-l007-sidebar-collapsed.png`,
`run4-l007-ctrlb-restore.png`.

Methodology note (harness lesson): VLM coordinate estimates drift 15-60 px
on dense UI rows. Every journey click must be pixel-verified (row scanning /
bounding-box cross-check) — a lesson now baked into the harness discipline
and the reason L-006 was corrected.
