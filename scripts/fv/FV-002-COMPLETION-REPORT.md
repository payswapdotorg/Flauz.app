# FV-002 — Completion Report

**Work order:** FV-002 — author the formal-verification harness per the
FV catalog: the Linux desktop J-suite scene scripts (d-series pattern),
the Windows CI journey-smoke extension (SendKeys scenes), and the web
formal-pass runner manifest (Phase: Formal verification, Wave 7, step 2,
dispatched Worker B).

**Base branch + SHA:** `main` @
`7f660c00407a5741eee975b2274570a200ff576b` (the Lead-authorized dispatch
pin — identical for calibration; FV-001 is delivered + Lead-approved but
merge-pending). Cloned HEAD verified equal to the pinned base by
`git rev-parse HEAD` at STEP ZERO.

**Branch/commits:** `feat/fv-002-harness`, one clean commit:
`feat(fv): FV-002 — the FV harness: Linux scene scripts (d-series), the Windows CI journey-smoke, the web formal-pass manifest (Wave 7)`
Delivery bundle: `fv-002-delivery.bundle`
(`7f660c00407a5741eee975b2274570a200ff576b..feat/fv-002-harness`), exported
at the repo root. NOT pushed — at the delivery close-out the push was
attempted once (per the continuation instruction) and the worker sandbox has
no GitHub credentials (`fatal: could not read Username for
'https://github.com': terminal prompts disabled`, exit 128; no
git-credentials / credential.helper / GITHUB tokens configured), so the
bundle is the delivery vehicle, per the work order's contract.

**RE-ENTRY LAW check:** the FV-002 harness artifacts did NOT exist at the
pinned base (`scripts/fv/` absent, `scripts/windows_fv_smoke.ps1` absent,
`web/lab/fv-manifest.json` absent — verified at STEP ZERO). This was a full
authoring dispatch, not verify-and-report.

## Changed files / surfaces

- `scripts/fv/fv-lib.sh` (NEW) — the sealed d-series recipe factored once:
  hard-fail named prerequisites (tools / the pinned-SHA binary / the pinned
  official CLI runtime `0.146.0-alpha.3.1` / the wo-p2-008 donor state),
  Xvfb + picom + lavapipe with isolated HOME/XDG/CODEX_HOME/CODEX_RS_DATA_DIR,
  keyboard-only drive helpers (refocus before every key event — the d26 law),
  frame captures + the md5 manifest, the named-moment action log
  (PROGRESS.txt), and the run.json lineage record pinning the repo SHA.
- `scripts/fv/fv-l01.sh` … `scripts/fv/fv-l21.sh` (NEW, 21 scripts) — one
  per catalog Linux scene (§1 rows 1:1): each carries the CALIBRATION header
  (every chord/anchor/copy string cited file:line against the pinned base),
  the catalog's named moments as drive steps, frame captures with md5s, the
  pass criteria in the header, and the honest bounds named in the run notes.
- `scripts/fv/README.md` (NEW) — the FV-003 runner doc: per-lane commands,
  prerequisites, evidence targets, the honest-bounds summary.
- `scripts/windows_fv_smoke.ps1` (NEW) — the SendKeys scene driver for the
  catalog's 12 Windows scenes (FV-W01..W12): per-scene isolated temp
  profiles, window-state assertions (process alive, main-window handle,
  clean `CloseMainWindow` exit 0), .NET screen captures with honest
  WARN-skip when the runner session lacks the capture API, the ≥ 15 s
  graceful-degradation watch (FV-W03), bounded app-log tails, and the
  RUN.json lineage record. The SendKeys chords are calibrated to the app's
  registered bindings (cited in the header comment).
- `.github/workflows/ci.yml` + `.github/workflows/release.yml` (MODIFIED —
  ADDITIVE steps only): the pinned-CLI npm install
  (`@openai/codex@0.146.0-alpha.3.1`, `continue-on-error: true` with the
  honest stub fallback in the driver), the Windows journey-smoke step on
  the Windows matrix legs, and the evidence artifact upload. The diffs are
  pure additions — no existing step changed.
- `web/lab/fv-manifest.json` (NEW) — the catalog's 14 web scenes
  (FV-E00..FV-E13) frozen as runner inputs: journey id, gateway mode=real
  (the spawn-cwd-relative `--gateway` command), the per-scene assertion set
  (the existing lab assertions), ×3 runs (the w6 convention), the fv-gate
  evidence root; the scenes array is the suite's sequential execution order.
- `web/lab/journeys.mjs` (MODIFIED — additive flags + one disclosed
  harness-defect fix) — the manifest mode: `--manifest` / `--scene` /
  `--runs`, the FV-E00 gateway-security probe (healthz + bind + traversal
  refusal + relative-only SPA fallback), per-run fresh gateway + fresh
  isolated CODEX_HOME + fresh browser context with the suite sharing one
  page per run, prefix semantics for `--scene` (the suite is sequential),
  named hard-fails for the missing gateway binary / web build, and the
  run-level RUN.json lineage.
- `scripts/fv/FV-002-COMPLETION-REPORT.md` (NEW — this file).

**Zero product-source changes:** `git diff --stat` vs the pinned base shows
only the owned paths: **29 files changed, 4427 insertions(+), 4
deletions(−)**. 26 files are NEW (pure additions — the fv scripts, the
PowerShell driver, the manifest, the docs); the 3 MODIFIED files are
`.github/workflows/ci.yml` +23/−0, `.github/workflows/release.yml` +23/−0,
`web/lab/journeys.mjs` +340/−4 — all four deletions sit in the harness
driver's flag-resolution rework (whose default behavior is byte-identical,
regression-proven) plus the disclosed one-line spawn fix (details under
Contract deviations). `crates/`, `web/src/`, `Cargo.toml`, `Cargo.lock`:
untouched.

## Implementation summary

The catalog defines WHAT must run; this delivery authors the harness that
runs it, 1:1 per scene row.

**Linux lane** — the d-series pattern, factored: `fv-lib.sh` owns the sealed
LINUX_GUI_LAB recipe so each scene script stays pure scene. Every script
hard-fails FIRST (named, actionable messages — verified live in this
sandbox, which lacks the Lead station: `FATAL: prerequisite tools missing at
this station: picom xdotool xwininfo …`, exit 1), then boots the app under
the isolated session with the pinned runtime, waits for the first >30 KiB
frame, and drives the catalog's named moments keyboard-only (the pointer
appears only in the boot-render nudge and FV-L21's Help-menu leg — the fr1d
placement-law exception, named in the script). Calibration was the
load-bearing work: every chord cited to its `KeyBinding::new(&shortcut(…))`
line (the `shortcut()` helper prefixes `ctrl-` on Linux — ui.rs:5731-5736),
every copy string cited to its constant (e.g. "Nothing needs you right now"
flauz_takeover.rs:178, "The context view is on its way" flauz_shell/mod.rs:303,
the retry cadence "Retry {n} in {s}s" ui.rs:16289-16291, the Find counter
"{n} / {m}{+} results" ui.rs:14777-14781, the worktree picker
"Create a copy of your local project to work in parallel" ui.rs:26355).
Scenes needing seeded chats (FV-L02/L07/L10/L16/L18) declare
`fv_need_donor` and hard-fail named when the donor fixture is absent.
Each scene writes run.json (the SHA lineage), md5.txt, PROGRESS.txt,
frames, and the app/xvfb/picom logs — the addendum §5 schema.

**Windows lane** — `scripts/windows_fv_smoke.ps1` implements the 12
catalog scenes as SendKeys drives with window-state assertions only (the
addendum §2 depth law), following the `windows_desktop_smoke.ps1`
conventions (CmdletBinding, StrictMode, bounded log tails, env restore,
force-kill fallback). FV-W03's graceful-degradation leg launches with
`CODEX_RS_CODEX_BIN` pointing at a missing binary and asserts the process
stays alive ≥ 15 s; its relaunch leg uses the pinned CLI when the CI
install succeeded, else the harmless `where.exe` stub with the named
honest bound recorded in the scene notes. The CI wiring adds three steps
per workflow (install / run / upload) — additive only, `yaml.safe_load`
green. The captures upload as workflow artifacts for the Lead's FV-003
adjudication.

**Web lane** — the manifest freezes the catalog's 14 scenes; the driver's
new manifest mode runs each scene ×3 with per-run fresh semantics (a fresh
gateway spawn, a fresh isolated CODEX_HOME — set on the driver's own env so
the journeys' own kill/restart legs inherit it — and a fresh browser
context), with the suite's scenes sharing one page per run exactly as the
classic driver does (j-02 opens the Context inspector on the session j-01
created; the scenes array is the execution order and `--scene` runs the
prefix). FV-E00 runs as a manifest scene: `/healthz` 200 with a loopback
bind, `//etc/passwd` refused 404 (never file content), the SPA fallback
serving only relative in-app routes — calibrated against
`flauz-web-gateway/src/static_files.rs` (the 2026-09-25 Lead gate fix) and
`server.rs` (the healthz payload carries the bind). Verified structurally:
the mock gateway can NEVER pass FV-E00 (its healthz carries no bind; its
`//etc/passwd` SPA-serves with 200) — the harness therefore enforces the
addendum §1 real-transport law mechanically.

## Tests / commands + exact results

| Command | Exact result |
| --- | --- |
| `git rev-parse HEAD` (STEP ZERO) | `7f660c00407a5741eee975b2274570a200ff576b` — equals the pinned base |
| `bash -n scripts/fv/*.sh` | **PASS** — all 22 scripts (21 scenes + fv-lib.sh) parse clean, zero output |
| `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); yaml.safe_load(open('.github/workflows/release.yml'))"` | **PASS** — both workflows parse as valid YAML (exit 0) |
| `python3 -m json.tool web/lab/fv-manifest.json > /dev/null` | **PASS** — valid JSON (exit 0); 14 scenes, runs=3, gateway mode=real |
| `git diff --stat 7f660c00407a5741eee975b2274570a200ff576b` | **29 files changed, 4427 insertions(+), 4 deletions(−)** — only the owned paths (26 NEW files are pure additions; the 3 modified: ci.yml +23/−0, release.yml +23/−0, journeys.mjs +340/−4) |
| `node --check web/lab/journeys.mjs` | **PASS** — syntax clean |
| Default-driver regression: `node web/lab/journeys.mjs --out /tmp/…` (13 journeys, mock transport) | **13/13 pass** — `lab: run webrun_… → pass` — the additive law proven (behavior unchanged without `--manifest`) |
| Manifest mode end-to-end (authoring-time, mock double): `--manifest … --scene fv-e01 --runs 3` | **fv-e01 run 1/3 → pass · run 2/3 → pass · run 3/3 → pass · run → pass**; evidence under `<out>/fv-e01/run-{1,2,3}/` |
| Manifest mode FV-E00 (authoring-time, mock double): `--scene fv-e00` | **fail** — `gateway.bind_loopback: unreported`, `gateway.traversal_refused: 200` — the mock structurally cannot pass the formal E00 (the §1 law); `gateway.healthz_ok: pass` |
| Manifest hard-fails (this sandbox — no Rust toolchain): `--manifest web/lab/fv-manifest.json` | `lab: fatal: the real gateway binary is missing: /home/z/flauz/target/debug/flauz-web-gateway — build it first (cargo build -p flauz-web-gateway; the Lead station has the toolchain — addendum §8), then rerun the FV manifest lane` — the named message, exit 1 |
| Scene hard-fails (this sandbox — no Lead station): `bash scripts/fv/fv-l01.sh /nonexistent/binary` | `FATAL: prerequisite tools missing at this station: picom xdotool xwininfo — install them (the warm Lead station has them under /home/z/parity-lab + desktop-tools) or run the scene from the Lead station` — exit 1 (verified; the runtime/donor named messages fire at their layers) |
| `npm view @openai/codex@0.146.0-alpha.3.1 version` | `0.146.0-alpha.3.1` — the CI install step's target exists on npm |
| Credential scan (`rg -i "sk-…|api[_-]?key|password|secret|token|bearer|credential"` over every authored file) | **Clean** — no credential material in scripts, workflows, manifest, or driver |
| pwsh parse check (`scripts/windows_fv_smoke.ps1`) | **HONEST ABSENCE** — `pwsh`/`powershell` do not exist in this worker sandbox (`command -v pwsh` → not found); the script received a careful manual review (StrictMode-safe patterns, SendKeys escaping, guarded capture/AppActivate with honest WARN-skip). The Lead's pre-merge spot-run executes it on CI (pwsh present there). |

## Kernel-compliance checklist (§1–§8)

- **§1 runtime truth:** the harness drives the REAL artifacts only — the
  pinned-SHA release binary under LINUX_GUI_LAB, the CI-built codexrs.exe,
  the real web build behind the REAL Rust gateway (the manifest mode
  hard-fails on mode≠real; FV-E00 structurally fails against the mock).
- **§2 honest bounds:** every scene names its bounds in the run notes
  (L-1/E-1 auth wall, X-3 not-wired view-models, L-5 no portal, W-5 no
  authenticated runtime, the L4/W-2 depth law); nothing silently narrowed,
  nothing fabricated.
- **§3 reclassification respected:** the scenes implement the catalog's
  current-main truth (the honest not-wired states are the verified
  content — no scene invents live data).
- **§4 the catalog is the acceptance law:** every catalog scene row maps
  1:1 to a script / CI wiring / manifest row (the coverage table below —
  all 47 scene rows accounted).
- **§5 evidence schema:** run.json lineage (pinned base + executing SHA),
  frames + md5 manifests, action logs, per-scene dirs under
  `docs/research/evidence/fv-gate/<lane>/<scene-id>/`, scripts committed
  and re-runnable, VLM reads Lead-side (L-8, by design).
- **§6 verifies-not-fixes:** zero product-source changes (git-diff-proven);
  the only code added is the harness itself (plus the disclosed harness
  defect fix in the lab driver, below).
- **§7 credentials:** none anywhere (the scan is clean); the supervised
  runtime stays unauthenticated + isolated.
- **§8 bounded delivery:** one clean commit on `feat/fv-002-harness`, git
  bundle at the repo root, this report on the branch and as the final
  message; the formal passes are Lead-run (FV-003).

## Harness coverage table (every catalog scene accounted)

| Catalog scene | Linux script | Windows wiring | Web manifest row |
| --- | --- | --- | --- |
| FV-L01 (J-01) | `scripts/fv/fv-l01.sh` | — | — |
| FV-L02 (J-02) | `scripts/fv/fv-l02.sh` (donor) | — | — |
| FV-L03 (J-03 guidance) | `scripts/fv/fv-l03.sh` | — | — |
| FV-L04 (J-03 retry cadence) | `scripts/fv/fv-l04.sh` | — | — |
| FV-L05 (J-04) | `scripts/fv/fv-l05.sh` | — | — |
| FV-L06 (J-05/J-06 guards) | `scripts/fv/fv-l06.sh` | — | — |
| FV-L07 (J-06 bounded) | `scripts/fv/fv-l07.sh` (donor) | — | — |
| FV-L08 (J-07) | `scripts/fv/fv-l08.sh` | — | — |
| FV-L09 (J-08) | `scripts/fv/fv-l09.sh` | — | — |
| FV-L10 (J-09) | `scripts/fv/fv-l10.sh` (donor) | — | — |
| FV-L11 (J-10) | `scripts/fv/fv-l11.sh` | — | — |
| FV-L12 (J-11) | `scripts/fv/fv-l12.sh` | — | — |
| FV-L13 (J-12 honest slice) | `scripts/fv/fv-l13.sh` | — | — |
| FV-L14 (J-13) | `scripts/fv/fv-l14.sh` | — | — |
| FV-L15 (J-14) | `scripts/fv/fv-l15.sh` | — | — |
| FV-L16 (J-15) | `scripts/fv/fv-l16.sh` (donor) | — | — |
| FV-L17 (J-16) | `scripts/fv/fv-l17.sh` | — | — |
| FV-L18 (J-17) | `scripts/fv/fv-l18.sh` (donor) | — | — |
| FV-L19 (DOMAIN-NEUTRAL) | `scripts/fv/fv-l19.sh` | — | — |
| FV-L20 (A11Y) | `scripts/fv/fv-l20.sh` | — | — |
| FV-L21 (FR/PKG) | `scripts/fv/fv-l21.sh` | — | — |
| FV-W01 (J-01) | — | `scripts/windows_fv_smoke.ps1` `Invoke-FvW01` (CI: ci.yml + release.yml) | — |
| FV-W02 (J-02) | — | `Invoke-FvW02` | — |
| FV-W03 (J-03 retryable startup) | — | `Invoke-FvW03` | — |
| FV-W04 (J-04) | — | `Invoke-FvW04` | — |
| FV-W05 (J-14) | — | `Invoke-FvW05` | — |
| FV-W06 (J-17) | — | `Invoke-FvW06` | — |
| FV-W07 (J-10/J-11) | — | `Invoke-FvW07` | — |
| FV-W08 (J-13) | — | `Invoke-FvW08` | — |
| FV-W09 (J-16/J-08) | — | `Invoke-FvW09` | — |
| FV-W10 (J-07) | — | `Invoke-FvW10` | — |
| FV-W11 (DOMAIN-NEUTRAL) | — | `Invoke-FvW11` | — |
| FV-W12 (A11Y bounded) | — | `Invoke-FvW12` | — |
| FV-E00 (gateway security) | — | — | `web/lab/fv-manifest.json` scene `fv-e00` → driver scene `fv-e00-gateway-security` |
| FV-E01 (J-01) | — | — | scene `fv-e01` → `j-01-start-project` |
| FV-E02 (J-02) | — | — | scene `fv-e02` → `j-02-understand-context` |
| FV-E03 (J-03) | — | — | scene `fv-e03` → `j-03-recover-reconnect` |
| FV-E04 (J-04) | — | — | scene `fv-e04` → `j-04-capability-gap` |
| FV-E05 (J-05) | — | — | scene `fv-e05` → `j-05-add-environment` |
| FV-E06 (J-06) | — | — | scene `fv-e06` → `j-06-cross-environment` |
| FV-E07 (J-08) | — | — | scene `fv-e07` → `j-08-takeover-approval-cancellation` |
| FV-E08 (J-09) | — | — | scene `fv-e08` → `j-09-artifacts-items` |
| FV-E09 (J-13) | — | — | scene `fv-e09` → `j-13-collaborate` |
| FV-E10 (J-14) | — | — | scene `fv-e10` → `j-14-switch-model` |
| FV-E11 (J-15) | — | — | scene `fv-e11` → `j-15-switch-environment` |
| FV-E12 (DOMAIN-NEUTRAL) | — | — | scene `fv-e12` → `j-domain-neutral-research` |
| FV-E13 (A11Y) | — | — | scene `fv-e13` → `j-a11y-responsive` |

Arithmetic: 21 Linux scripts + 12 Windows scene functions (wired into both
workflows) + 14 manifest rows = **47/47 catalog scene rows implemented
1:1**; the 14 N/A cells stay N/A with their named reasons (untouched).

## Acceptance-criteria evidence

1. **Every catalog Linux scene has a script; every catalog Windows scene is
   wired into the CI journey-smoke; every catalog web scene is in the
   manifest** — the coverage table above: 21/21, 12/12 (both ci.yml and
   release.yml run the driver; the evidence uploads as workflow artifacts),
   14/14.
2. **Scripts hard-fail with named messages when prerequisites are missing**
   — verified live: the tools check (`FATAL: prerequisite tools missing at
   this station: picom xdotool xwininfo …`, exit 1), the binary check, the
   pinned-runtime check (named, with the restore procedure), the donor
   check (named, with the wo-p2-008 pointer), and on the web lane the
   gateway/web-build checks (`lab: fatal: the real gateway binary is
   missing: …`). Never silent, never fabricated.
3. **Zero product-source changes** — `git diff --stat
   7f660c00…`: only `scripts/fv/**`,
   `scripts/windows_fv_smoke.ps1`, `.github/workflows/{ci,release}.yml`,
   `web/lab/fv-manifest.json`, `web/lab/journeys.mjs` (all owned paths);
   `crates/`, `web/src/`, `Cargo.toml`, `Cargo.lock` untouched.
4. **The CI workflow remains green** — the added steps are pure additions
   (yaml-verified); the smoke is `continue-on-error`-shielded only for the
   CLI install (with the honest stub fallback + named bound in the scene
   notes — the driver itself asserts strictly); the Lead spot-runs the
   Windows workflow in CI before merge per the work order's Tests row
   (the runner-side execution is the Lead's gate, addendum §8).

## Known limitations

- **pwsh absent in this worker sandbox** — the PowerShell parse check could
  not run here (honest absence); the Lead's CI spot-run executes the script
  (CI windows-latest has pwsh).
- **No Rust toolchain / no Lead station in this sandbox** (the W-4-class
  bound): no gateway build, no calibration scene run — the calibration
  captures are the Lead's at the gate station (the work order's GUI/lab
  evidence row: "one calibration capture per authored Linux scene (the Lead
  runs them at the gate station)").
- **The Linux scene scripts' keyboard-focus walks** (e.g. the seeded-chat
  selection via Tab/Down/Return) are calibrated against the d-series
  precedents but unverified at runtime in this sandbox — the Lead's
  calibration run will confirm them (any focus-order drift becomes a
  focused fix through the synchronization law).
- **FV-L21's About-dialog leg** drives the Help dropdown menu with the
  fr1d-placement-law coordinate clicks (the one named pointer exception —
  menus; cited in the script) — the keyboard path to the menu item was not
  verifiable from source alone.
- **The web manifest's real-gateway run** was validated end-to-end only
  against a local mock test double (the §1 law makes that structurally
  insufficient for FV-E00, which is the point); the first real-gateway
  execution is FV-003's at the Lead station.
- **The FV-L16 worktree-fork leg** depends on the donor chat being bound to
  a local git workspace; the donor fixture's shape is the Lead station's
  (the wo-p2-008 pattern).

## Contract deviations

**ONE, disclosed:** the work order scopes `web/lab/journeys.mjs` to
"additive flags only". Beyond the additive manifest mode, this delivery
also fixed a one-line **pre-existing harness defect** on the driver's
real-gateway spawn path: `[...rest.split(" ")…]` called `.split` on the
already-split token array, so **every `--gateway` invocation threw
`TypeError: rest.split is not a function`** (verified empirically at this
base: the real-transport flag — and therefore the entire FV web lane this
work order builds — could never start). The fix passes the token array
itself (identical intent, one line, commented in place with the
disclosure). It is harness code only (web/lab/), no journey behavior
change, no product source — and the default-path regression (13/13
journeys pass) proves it. Recorded as a deviation because the letter of
the scope says "additive flags only"; the alternative (leaving the
mandated `--gateway` path broken) would have failed acceptance criterion 1
for the web lane. Two related additive-only notes: the three
flag-resolution lines (`GATEWAY_COMMAND`/`OUT_ROOT`/`PORT`) were reworked
into resolver functions whose default (non-manifest) behavior is
byte-identical (regression-proven), and the driver's run-scoped CODEX_HOME
per manifest run is set on the driver's own environment (manifest mode
only).

**Base-arrangement note (Lead-authorized, per the dispatch packet's BASE
NOTE — not counted as a deviation):** the work order's dependency line
reads "FV-001 merged"; FV-001 is delivered + Lead-approved but
merge-pending, and the pinned base `main @ 7f660c00…` is identical for
calibration (FV-001 is docs-only). The FV-CATALOG was saved locally at
`docs/research/FV-CATALOG.md` for reference and **not committed** (it is
FV-001's delivery).

## Follow-up work

- **FV-003 (Lead):** execute every lane at the pinned SHA — run the 21
  Linux scenes at the gate station (VLM-adjudicate; archive
  `docs/research/evidence/fv-gate/linux/`), spot-run the Windows workflow
  in CI before merge (the artifacts are the evidence), and run the web
  manifest (`node web/lab/journeys.mjs --manifest web/lab/fv-manifest.json`)
  ×3; write the FV gate record.
- **FV-002's own merge gates (per the work order):** bash -n ✓ (this
  report), pwsh parse (Lead-side, CI), one calibration scene run (Lead),
  CI green (Lead spot-run).
- **Defects found during FV-003** become focused fix work orders through
  the synchronization law (defect → focused PR → merge → fresh environment
  → rerun the affected journey) — this wave verifies, it does not fix.
- The RG-SOAK battery + RG-RECONNECT-05..07 stay Lead-station release
  scenes (gap L-9) — FV-003 records their status in the gate record
  without folding them into the journey catalog.
