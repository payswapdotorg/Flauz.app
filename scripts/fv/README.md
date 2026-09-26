# FV-002 — The formal-verification harness runner doc

How **FV-003** (the Lead-run formal pass) executes each lane at the
pinned SHA. The authoring contract is the FV-CATALOG
(`docs/research/FV-CATALOG.md`, FV-001's delivery); this harness
implements its scene rows 1:1.

- **Pinned base (authored + calibrated):**
  `7f660c00407a5741eee975b2274570a200ff576b`
- **Harness law:** WAVE7-FV-WORK-ORDERS.md kernel addendum §1–§8
  (runtime truth, honest bounds, the evidence schema, verifies-not-fixes,
  credentials, bounded deliveries).

## The three lanes

| Lane | The REAL artifact | Driver | Evidence root |
| --- | --- | --- | --- |
| Linux desktop | the release-profile `codexrs` built at the pinned SHA | `scripts/fv/fv-lNN.sh` (this dir) | `docs/research/evidence/fv-gate/linux/<scene-id>/` |
| Windows desktop | the `codexrs.exe` built by CI (windows-latest) | `scripts/windows_fv_smoke.ps1` (CI-wired) | `docs/research/evidence/fv-gate/windows/<scene-id>/` (runner captures archived via workflow artifacts) |
| Web | the real web build (`web/dist`) served by the REAL Rust gateway | `node web/lab/journeys.mjs --manifest web/lab/fv-manifest.json` | `docs/research/evidence/fv-gate/web/<scene-id>/` |

## Prerequisites (every lane hard-fails with named messages)

**Common:** the repo checked out at the pinned SHA; `git`, `python3`.

**Linux lane (the warm Lead station):**
- build: `cargo build --locked --release -p codex-app` → `target/release/codexrs`
- tools: `Xvfb`, `picom`, `xdotool`, `ffmpeg`, `xwininfo`, `md5sum`
  (the Lead station carries them under `/home/z/parity-lab` +
  `/home/z/.local/desktop-tools`; the scripts prepend those paths)
- the pinned official CLI runtime:
  `/home/z/parity-lab/runtime/codex` (npm
  `@openai/codex@0.146.0-alpha.3.1-linux-x64` — the platform-support
  oracle pin; the wo-p2-008 README records the restore procedure)
- the donor-state fixture for seeded scenes (FV-L02/L07/L10/L16/L18):
  `/tmp/d10-home.*/.local/share/codexRS/state.sqlite3` (the wo-p2-008
  pattern). The scripts hard-fail with a named message when absent.
- the VLM reader at the gate station (L-8: the worker/lab archives
  frames + prompts; the reads happen at the Lead gate pass)

**Windows lane:** the CI windows-latest runner (the workflow steps are
already wired — additive, this wave). No local prerequisites for FV-003
beyond reading the workflow artifacts.

**Web lane:**
- the built gateway binary: `cargo build -p flauz-web-gateway` →
  `target/debug/flauz-web-gateway`
- the real web build: `web/dist` (`npm run build` in `web/`)
- `node` ≥ 20 with `web/node_modules` (playwright chromium installed)
- the pinned CLI for the supervised app-server (isolated
  `CODEX_HOME` — the gateway supervises a real `codex app-server`)
- E-1 bound (named): the supervised runtime is unauthenticated in the
  lab — live-turn depth is bounded to honest states, exactly like
  Linux L-1.

## Linux lane — executing the scenes

```bash
git checkout <pinned-sha>
cargo build --locked --release -p codex-app

# one scene (each writes frames + md5.txt + PROGRESS.txt + run.json
# into docs/research/evidence/fv-gate/linux/<scene-id>/):
bash scripts/fv/fv-l01.sh target/release/codexrs

# every Linux scene:
for s in scripts/fv/fv-l[0-9][0-9].sh; do bash "$s" target/release/codexrs; done
```

Scene list (the catalog §1 rows, 1:1):

| Scene | Journey | Script | Seeded (donor) |
| --- | --- | --- | --- |
| FV-L01 | J-01 | `fv-l01.sh` | — |
| FV-L02 | J-02 | `fv-l02.sh` | donor |
| FV-L03 | J-03 (guidance) | `fv-l03.sh` | — |
| FV-L04 | J-03 (retry cadence) | `fv-l04.sh` | — |
| FV-L05 | J-04 | `fv-l05.sh` | — |
| FV-L06 | J-05/J-06 guards | `fv-l06.sh` | — |
| FV-L07 | J-06 (bounded) | `fv-l07.sh` | donor |
| FV-L08 | J-07 | `fv-l08.sh` | — |
| FV-L09 | J-08 | `fv-l09.sh` | — |
| FV-L10 | J-09 | `fv-l10.sh` | donor |
| FV-L11 | J-10 | `fv-l11.sh` | — |
| FV-L12 | J-11 | `fv-l12.sh` | — |
| FV-L13 | J-12 (honest slice) | `fv-l13.sh` | — |
| FV-L14 | J-13 | `fv-l14.sh` | — |
| FV-L15 | J-14 | `fv-l15.sh` | — |
| FV-L16 | J-15 | `fv-l16.sh` | donor |
| FV-L17 | J-16 | `fv-l17.sh` | — |
| FV-L18 | J-17 | `fv-l18.sh` | donor |
| FV-L19 | DOMAIN-NEUTRAL | `fv-l19.sh` | — |
| FV-L20 | A11Y | `fv-l20.sh` | — |
| FV-L21 | FR/PKG | `fv-l21.sh` | — |

Evidence per scene (the addendum §5 schema): `<scene-id>/run.json`
(lineage: the pinned base + the executing SHA + the binary/runtime/donor
paths), `md5.txt` (the frame manifest), `PROGRESS.txt` (the named-moment
action log), `*.png` (the frames), `app.log` (+ `xvfb.log`, `picom.log`).
VLM reads are archived beside their frames by the Lead (`vlm-*.json`,
the w4-gate convention).

The adjacent release-gate scenes (RG-SOAK G-1..G-7, RG-RECONNECT-05..07)
remain Lead-station release scenes per the catalog — FV-003 records
their status in the gate record without folding them into the journey
catalog (the soak needs the 4 h Lead time budget — gap L-9).

## Windows lane — the CI journey-smoke

Wired additively into both workflows (FV-002 added the steps; nothing
existing changed):

- `.github/workflows/ci.yml` — the `native` job (windows-latest matrix
  leg) runs `scripts/windows_fv_smoke.ps1 -Binary target/release/codexrs.exe`
  after the release build, and uploads the captures as
  `fv-windows-journey-smoke` artifacts.
- `.github/workflows/release.yml` — the `build` job (windows-x86_64 leg)
  runs the same driver after packaging.

FV-003 reads the workflow artifacts into
`docs/research/evidence/fv-gate/windows/<scene-id>/` and adjudicates.
The driver implements the catalog §2 scenes FV-W01..FV-W12 as SendKeys
drives with window-state assertions (process alive, main window handle
present, clean `CloseMainWindow` exit) and .NET screen captures where
the runner session allows (honest WARN + skip when the capture API is
unavailable — never fabricated). Depth law: **SendKeys + window-state
assertions only** (addendum §2); everything beyond is a named gap (§5),
never silently narrowed.

## Web lane — the formal-pass manifest

```bash
# build the real artifacts first:
cargo build -p flauz-web-gateway        # target/debug/flauz-web-gateway
cd web && npm run build && cd ..        # web/dist

# the formal pass (the manifest drives; ×3 runs per scene, the w6
# convention; real gateway only — mock captures are NOT formal
# evidence, addendum §1):
node web/lab/journeys.mjs --manifest web/lab/fv-manifest.json

# one scene only (reruns the manifest PREFIX through that scene — the
# suite is sequential: j-02 opens the Context inspector on the session
# j-01 created, so a filtered rerun re-runs its precursors too):
node web/lab/journeys.mjs --manifest web/lab/fv-manifest.json --scene fv-e02

# fewer runs (debugging) or a different gateway profile:
node web/lab/journeys.mjs --manifest web/lab/fv-manifest.json --runs 1
node web/lab/journeys.mjs --manifest web/lab/fv-manifest.json \
  --gateway "target/release/flauz-web-gateway --web-root dist"
```

The manifest (`web/lab/fv-manifest.json`) freezes the catalog §3 scenes
FV-E00..FV-E13: journey id, gateway mode=real (the `--gateway`
command), the per-scene assertion set (the existing lab assertions),
×3 runs, and the evidence root. FV-E00 (the gateway security slice)
runs as a manifest scene: the gateway boots on the default localhost
bind, `GET /healthz` must be 200 with a loopback bind, `GET //etc/passwd`
must 404-refuse (never file content), and the SPA fallback must serve
only relative in-app routes. (Verified at authoring time: the mock
gateway can NEVER pass FV-E00 — its healthz carries no bind and its
`//etc/passwd` SPA-serves with 200 — the harness therefore structurally
enforces the real-transport law.)

Run semantics (the w6 ×3 convention, folded into one command): each run
= a fresh driver invocation — a freshly started gateway (fresh
supervised runtime), a fresh isolated `CODEX_HOME` for the supervised
app-server (set by the driver on its own environment so every gateway
spawn in the run, including the journeys' own kill/restart legs,
inherits it), and a fresh browser context in which the requested scenes
run in the manifest's suite order, sharing one page exactly as the
classic suite does. Scene N's evidence lands under
`<evidence_root>/<scene-id>/run-<n>/` (the parity-lab record schema),
plus the run-level `RUN.json` (lineage: the pinned base, the gateway
mode, the requested scenes, the failures).

The gateway command is relative to the driver's spawn cwd (`web/`):
`../target/debug/flauz-web-gateway` is the repo-root cargo target;
`--web-root dist` is `web/dist` relative to the same cwd. The driver
hard-fails with named messages when the gateway binary or the web build
is missing.

## Honest bounds (the catalog §5 gap list — never silently narrowed)

The full per-lane gap list lives in the FV-CATALOG §5. The bounds that
most affect execution:

- **L-1 / E-1 (the auth wall):** no authenticated Codex account in the
  lab — live-turn scenes are bounded to honest-state slices.
- **L-2/L-3:** one 1440×900 Xvfb screen, software rendering (lavapipe).
- **W-1/W-2:** no Windows GUI host here; the Windows lane is
  CI-drivable depth only.
- **L-8:** VLM adjudication is Lead-side, by design.
- **Credentials (§7):** no credential ever appears in scripts, logs,
  URLs, or evidence. The supervised runtime is unauthenticated +
  isolated.

## Re-running + the synchronization law

Every scene script is committed and re-runnable at any SHA (the
calibration comments pin the authored base; a re-run at a newer SHA
records the executing SHA in `run.json` and the Lead re-adjudicates).
Defects found during FV-003 become focused fix work orders (defect →
focused PR → merge → fresh environment → rerun the affected journey) —
this wave verifies, it does not fix.
