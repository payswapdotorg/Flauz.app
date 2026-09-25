# FV-001 — Completion Report

**Work order:** FV-001 — reclassify every historical finding against
current main + author the formal-verification catalog (journey → client →
scenes → pass criteria) + the honest FV gap list (Phase: Formal
verification, Wave 7, step 1; dispatched Worker A).

**Base branch + SHA:** `main` @ `7f660c00407a5741eee975b2274570a200ff576b`
(cloned HEAD verified equal to the pinned base by `git rev-parse HEAD` at
STEP ZERO — the Wave-7 dispatch head over the Wave-6 merge `f5e2a5c`).

**Branch/commits:** `feat/fv-001-reclassify`, one clean commit:
`docs(fv): FV-001 — findings reclassified against current main + the FV catalog + the honest gap list (Wave 7)`
Delivery bundle: `fv-001-delivery.bundle`
(`7f660c00407a5741eee975b2274570a200ff576b..feat/fv-001-reclassify`).

**RE-ENTRY LAW check:** the FV-001 artifacts did NOT exist at the pinned
base (`docs/research/evidence/fv-001/` absent; `docs/research/FV-CATALOG.md`
absent — verified immediately after STEP ZERO). This was a full authoring
dispatch, not verify-and-report.

## Changed files / surfaces (docs-only — git-diff-proven)

- `docs/research/evidence/fv-001/FINDINGS-RECLASSIFICATION.md` (NEW) —
  every historical finding from the input ledgers, reclassified against
  current main with current-source citations: §A the AES expectations
  families (A-1..A-7) · §B the KF controls + limitations (B-1..B-11) ·
  §C the accessibility inventory (C-1..C-20) · §D the journey inventory
  (D-1..D-18 + the cold-start surfaces) · §E the parity inventory (41
  rows + finding-16, carried under the F1 freeze) · §F the release-gate
  anchors (F-1..F-8) · §G the F1-closure honest notes (G-1..G-7) · §H the
  codex-linux official-side observations (H-1..H-8) · §I the wave-gate
  deferrals (I-1..I-12) · §J the reference-matrix sharpened findings
  (J-1..J-6) + the coverage-arithmetic table.
- `docs/research/FV-CATALOG.md` (NEW) — the journey→client→scene catalog:
  the three lane definitions + driving disciplines (§0), 21 Linux scenes
  (§1), 12 Windows scenes + the standing CI gates (§2), 14 web scenes
  incl. the gateway-security slice (§3), the N/A register with named
  reasons (§4), the FV gap list per lane + cross-lane with recovery paths
  (§5), the J-01..J-18 × 3-lane coverage table (§6), and FV-002's 1:1
  authoring notes (§7).

NOTHING else changed (`git diff --stat 7f660c0` = the two files above,
+606 lines, zero source-file changes).

## Implementation summary

The production gate (the 2026-09-23 sequencing amendment) requires formal
verification of Linux desktop + Windows desktop + Web; nobody had
reclassified the historical findings against current main or consolidated
which journeys must run on which client. This order did exactly that,
docs-only:

1. **The reclassification** reads every input ledger (the f1-sweep packet ×7,
   known-failures.md, codex-linux/**, the AES expectations list, the w2..w6
   gate records, the FLAUZ-REFERENCE-MATRIX) and reclassifies each finding
   of record against `7f660c0` using the closed §3 vocabulary
   (still-open / fixed-by-`<merge>` / changed-surface / needs-runtime-verify),
   with a current-source citation per verdict. No finding is closed on
   static reasoning: every `fixed-by` is marked PROVISIONAL with its
   FV-003 runtime scene named (the synchronization law). The one line-level
   product-code defect kept open (I-8: the duplicated FlauzConflicts
   listener remnant, `ui.rs:45906` + `ui.rs:45928`) is recorded, not
   patched — docs-only order.
2. **The catalog** maps J-01..J-18 onto the three production-gate clients
   with concrete scenes: Linux 21 scenes (the d-series pattern: Xvfb +
   picom + keyboard-only + named moments + frames + VLM), Windows 12 scenes
   (CI-drivable depth only: SendKeys + window-state assertions — addendum
   §2), Web 14 scenes (the real gateway via `web/lab/journeys.mjs
   --gateway`; mock captures are authoring fallbacks only — §1). Each
   client carries its domain-neutral non-code scene and its A11Y scene.
   Every scene row names: scene id · journey id · lane · moments · pass
   criteria (what a frame/assertion must show) · evidence target dir under
   `docs/research/evidence/fv-gate/`.
3. **The gap list** (the honesty surface, addendum §2) enumerates per lane
   what this environment cannot formally verify — Linux 9 bounds (the auth
   wall, single-screen, software rendering, X11-only, no portal picker,
   the flow-gated modals, the desktop-env matrix, Lead-side VLM, the soak
   time budget), Windows 6 bounds (no GUI host — the E2B platform truth;
   CI depth; ephemeral profile; toolchain; no authenticated runtime;
   load-depth surfaces), Web 6 bounds (unauthenticated runtime, chromium
   only, emulated mobile, the J-13 protocol gap, localhost, no concurrent
   multi-client), plus 4 cross-lane bounds — each with its honest reason
   and its recovery path. Never silently narrowed, never fabricated.

## Tests/commands + exact results

Docs-only order — no cargo/npm gates apply (the sandbox lacks the Rust
toolchain for this repo; stated plainly per the dispatch). The gates that
DO apply, run verbatim:

1. `git rev-parse HEAD` → `7f660c00407a5741eee975b2274570a200ff576b` (STEP
   ZERO — exact match).
2. `git diff --stat 7f660c00407a5741eee975b2274570a200ff576b` → exactly:
   `docs/research/FV-CATALOG.md | 264 +++` +
   `docs/research/evidence/fv-001/FINDINGS-RECLASSIFICATION.md | 342 +++`
   (2 files, 606 insertions, 0 deletions) — the docs-only proof
   (acceptance criterion 4).
3. **The citation spot-check list** (41 spot-checks executed at the pinned
   checkout — every citation class covered; grep/sed against the cited
   file:line, match recorded):
   - `ui.rs:48139` → `return Some("Select a chat before archiving it.");` ✓
   - `ui.rs:48207` → `return Some("Select a workspace before searching files.");` ✓
   - `ui.rs:10067` → `self.focus_before_command_palette = window.focused(cx);` ✓
   - `ui.rs:523` → `/// The visible command-palette entry (WO-P2-018): the chrome counterpart of` ✓
   - `ui.rs:436-442` → `fn terminal_dock_focus_transfer_due(...) -> bool { focus_pending && pty_input_mounted && !overlay_open }` ✓
   - `ui.rs:43450` → `.child(format!("Introducing {display_name}")),` ✓
   - `ui.rs:45906` + `ui.rs:45928` → BOTH `.on_action(cx.listener(|this, _: &FlauzConflictsShortcut, ...))` (the duplicated listener — I-8 verified present) ✓
   - `ui.rs:5788` → `KeyBinding::new(&shortcut("alt-2"), FlauzReusableWorkflowsShortcut, None),` ✓
   - `ui.rs:3860` → `const ALL: [Self; 93] = [` ✓
   - `ui.rs:24451` → `fn render_composer_status(` ✓
   - `ui.rs:25299` → `fn render_safety_buffering_banner(` ✓
   - `ui.rs:16277` → `"App-server online".to_owned(),` ✓
   - `ui.rs:6216` → `"Couldn't connect to the Codex app-server",` ✓
   - `backend.rs:13325` → `"Codex hit an error and is retrying.".to_owned()` ✓
   - `backend.rs:223-224` → `APP_SERVER_RECONNECT_INITIAL_DELAY … from_secs(1)` / `…MAX_DELAY … from_secs(20)` ✓
   - `backend.rs:1797` → `struct AppServerReconnectScheduler {` ✓
   - `backend.rs:22172` → `let delay = Duration::from_millis(300);` ✓
   - `codex-core/lib.rs:20336-20338` → `Action::SetStatus(message) => { state.status_message = Some(bounded_string(message, 16 * 1024));` ✓
   - `codex-core/lib.rs:9756-9757` → `// silently opening an empty dock (WO-P1-001 honest toggle).` + `"Select a task before opening a terminal."` ✓
   - `codex-core/lib.rs:27316` → `fn stale_same_chat_submission_failure_preserves_the_newer_draft() {` ✓
   - `codex-protocol/lib.rs:17` → `pub const DEFAULT_MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;` ✓
   - `codex-platform/app_server.rs:1062-1063` → `if !params.use_state_db_only { return Err(…` ✓
   - `codex-platform/computer_use.rs:34-35` → `/// Linux observation is intentionally limited to X11/XWayland…` ✓
   - `codex-app/main.rs:47` → `Some(command) if command == "--install-desktop-entry" => run_install_desktop_entry(args),` ✓
   - `ui.rs:11550` → `title: Some(SharedString::from("About codexRS")),` ✓
   - `flauz_shell/mod.rs:305` → `Self::Environments => "No environments attached",` ✓
   - `flauz_save_workflow.rs:115` → `const NOT_WIRED_TITLE: &str = "Saving isn't wired to live tasks yet";` ✓
   - `flauz_members.rs:120` → `const INVITE_NOT_WIRED_TITLE: &str = "Inviting isn't connected yet";` ✓
   - `web/src/state/capabilities.ts:24` → `import { REQUEST_METHODS } from "../protocol/generated";` ✓
   - `web/src/keyboard/keyboard.ts:17` → `{ id: "rail.context", chord: "Ctrl+Alt+Shift+1", … }` ✓
   - `flauz-web-gateway/config.rs:20` → `pub const DEFAULT_BIND_LABEL: &str = "127.0.0.1:8610";` ✓
   - `flauz-web-gateway/static_files.rs:99-100` → `/// Normalizes a request path to a safe relative path, refusing / /// traversal (…` ✓
   - `scripts/linux_desktop_smoke.sh:33-44` → `CODEX_HOME=…` / `CODEX_RS_CODEX_BIN=/bin/false` / `LIBGL_ALWAYS_SOFTWARE=1` / `xvfb-run …` / `timeout … 15s` ✓
   - `.github/workflows/ci.yml:24` → `os: [windows-latest, ubuntu-24.04]` ✓
   - `.github/workflows/release.yml:194` → `& scripts/windows_desktop_smoke.ps1 -Binary $binary` ✓
   - `ui.rs:3328` → `id: "toggleActivityView",` ✓
   - `ui.rs:4010` → `Self::OpenImportSettings => "Import",` ✓
   - `flauz_model_picker.rs:431` → `pub(crate) fn set_catalog(&mut self, catalog: Vec<PickerModelEntry>) {` ✓
   - `ui.rs:5846` → `KeyBinding::new(&shortcut("alt-^"), FlauzCapabilityGapShortcut, None),` (the N6 companion) ✓
   - `codex-core/lib.rs:30162` → `fn terminal_dock_toggle_is_honest_on_the_entry_surface() {` ✓
   - `codex-core/lib.rs:9360` → `Some("Open a chat before opening the Browser.".to_owned());` ✓

   Plus the catalog copy-string calibrations (read at the pinned base, the
   d26 discipline): the recovery guidance (`flauz_recovery.rs:132`), the
   capability subtitle (`flauz_capability_gap.rs:60`), the agents
   heading/empty (`flauz_agents_view.rs:66/75`), the needs-you empty
   (`flauz_takeover.rs:178`), the members heading (`flauz_members.rs:109`),
   the model empty + identity promise (`flauz_model_picker.rs:124/120`),
   the shell empty states (`flauz_shell/mod.rs:152/306/324`), the save
   affordance (`flauz_save_workflow.rs:103/108`), the worktree copy
   (`ui.rs:24274`), the unread statuses (`lib.rs:10697/10724`).

## Kernel-compliance checklist (Wave-7 addendum §1–§8)

1. **Verification truth is runtime truth** — HELD: the catalog defines
   runtime-only evidence (the real binary per lane; the real gateway for
   web — mock captures explicitly named authoring fallbacks); every
   `fixed-by` reclassification is PROVISIONAL pending FV-003 runtime
   confirmation; no finding closed on static reasoning.
2. **Three lanes, three truths, honestly bounded** — HELD: Windows depth
   bounded to CI-drivable SendKeys + window-state scenes (catalog §2);
   every beyond-depth item is a NAMED gap with its recovery path (§5 W-1..
   W-6); nothing silently narrowed or fabricated.
3. **Historical findings reclassified, not assumed** — HELD: the closed §3
   vocabulary used throughout; the synchronization law cited as the
   closure authority; the historical ledgers quoted as inputs, not truth.
4. **The journey catalog is the acceptance law** — HELD: J-01..J-18 mapped
   per client with scenes, named moments (d-series style), per-scene pass
   criteria, evidence dirs; the shared acceptance law judged per journey;
   the domain-neutral scenario per client + the A11Y scene per client.
5. **Evidence schema unchanged** — HELD: fv-gate/ targets follow the
   parity-lab schema (RUN.json lineage + per-journey dirs + captures +
   action logs + assertions + VLM reads + the pinned SHA); scripts
   committed and re-runnable.
6. **The FV wave verifies; it does not fix** — HELD: zero product changes
   (git-diff-proven); the one open defect (I-8) recorded for the next
   ui.rs-touching wave, not patched.
7. **Reproducibility and credentials** — HELD: the catalog's prerequisites
   assume the warm Lead station with hard-fail named messages (FV-002
   implements); no credential appears in any artifact authored here.
8. **Bounded deliveries under the platform wedge** — HELD: one clean
   commit-branch delivery + the git bundle, the Wave-5/6 contract; the
   runs (FV-003) are Lead-executed.

## Catalog coverage table (J-01..J-18 × client mapping summary)

| Journey | Linux | Windows | Web |
| --- | --- | --- | --- |
| J-01 | FV-L01 | FV-W01 | FV-E01 |
| J-02 | FV-L02 | FV-W02 | FV-E02 |
| J-03 | FV-L03+L04 | FV-W03 | FV-E03 |
| J-04 | FV-L05 | FV-W04 | FV-E04 |
| J-05 | FV-L06 | N/A (CI depth) | FV-E05 |
| J-06 | FV-L06+L07 | N/A (no live turns) | FV-E06 |
| J-07 | FV-L08 | FV-W10 | N/A (no surface) |
| J-08 | FV-L09 | FV-W09 | FV-E07 |
| J-09 | FV-L10 | N/A (no live turns) | FV-E08 |
| J-10 | FV-L11 | FV-W07 | N/A (no surface) |
| J-11 | FV-L12 | FV-W07 | N/A (no surface) |
| J-12 | FV-L13 (honest slice) | N/A | N/A |
| J-13 | FV-L14 | FV-W08 | FV-E09 |
| J-14 | FV-L15 | FV-W05 | FV-E10 |
| J-15 | FV-L16 | N/A (CI depth) | FV-E11 |
| J-16 | FV-L17 | FV-W09 | N/A (no surface) |
| J-17 | FV-L18 | FV-W06 | N/A (no surface) |
| J-18 | N/A (no surface) | N/A | N/A |
| Domain-neutral | FV-L19 | FV-W11 | FV-E12 |
| A11Y | FV-L20 | FV-W12 | FV-E13 |
| FR/PKG + gateway-security | FV-L21 | release.yml smoke (standing) | FV-E00 |

54 journey-cells = 33 scene cells + 21 N/A cells, every N/A with its named
reason (catalog §4).

## Acceptance-criteria evidence (map each bullet)

1. **Every historical finding appears exactly once with a current-main
   verdict** — FINDINGS-RECLASSIFICATION.md §A-§J: the coverage-arithmetic
   table maps every input ledger to its rows (AES 7/7 · KF 6+5 · AI
   defect/gap/platform-bound/unverifiable all · JI 18+6 · PI 41+11+2 ·
   RGI 8 · FCR 7 · CL 8 · wave gates 12 · FRM 6); folds are named in-place;
   every verdict cites current source lines (41 spot-checks above).
2. **Every J-01..J-18 journey appears with its client mapping (or a named
   N/A reason)** — FV-CATALOG.md §6: the full matrix; the N/A register §4
   names each reason (CI depth / no live turns / no surface / protocol gap
   / not wired / deferred-by-amendment).
3. **The gap list names every lane bound with its recovery path** —
   FV-CATALOG.md §5: Linux L-1..L-9, Windows W-1..W-6, Web E-1..E-6,
   cross-lane X-1..X-4 — each with the honest reason + the recovery path.
4. **Zero source-file changes** — `git diff --stat 7f660c0` = the two owned
   docs paths only (exact output above); acceptance criterion 4 GREEN.

## Known limitations

- This sandbox lacks the Rust toolchain for this repo (stated plainly) —
  no cargo gates were run; none apply to a docs-only order (the Lead
  source-spot-checks every citation class per the dispatch notes).
- Line citations are current-main lines at `7f660c0` — volatile by design;
  the symbol names are the stable anchors (the house convention, stated in
  the artifact).
- The F1 parity freeze is carried forward (§E), not re-sliced: the
  Wave-2..6 Flauz-native surfaces are beyond-parity additions; re-slicing
  the F1 classes is a Lead decision.
- The Windows scene spec is necessarily shallower than Linux (the §2
  bound); the catalog encodes that honestly rather than pretending parity
  of depth.

## Contract deviations

NONE.

## Follow-up work

1. **FV-002** (next dispatch): author the scene scripts per this catalog —
   `scripts/fv/` Linux scenes (d26 pattern), the Windows journey-smoke
   extension, the web `fv-manifest.json`, `scripts/fv/README.md`.
2. **FV-003** (Lead-run): execute the lanes at the pinned SHA; adjudicate;
   write the fv-gate record; confirm/refute the provisional `fixed-by`
   reclassifications.
3. The duplicated FlauzConflicts listener (I-8) — one-line cleanup for the
   next ui.rs-touching wave (NOT this wave).
4. The wiring backlog rows (I-1..I-3, I-5..I-7) grow the catalog when
   their waves land (X-3's recovery path).
