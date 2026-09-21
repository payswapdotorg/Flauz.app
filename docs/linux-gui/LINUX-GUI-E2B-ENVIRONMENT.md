# Linux GUI — E2B Desktop Environment Record

**Lane:** Linux GUI usability (second Tech Lead — Linux lane)
**Primary Linux UX test environment:** E2B Desktop (operator directive — real
GUI interaction, not source inspection/unit tests/CI).
**Harness:** `/home/z/flauz-lane/e2b/flauz_e2b.py` (create / provision /
launch / interact / evidence).

Reproducibility rule: the Tech Lead must be able to destroy and recreate the
desktop environment and reproduce the same journey. Every journey evidence
run records the row below at run time.

## Current environment (recorded `2026-09-21`)

| Field | Value |
| --- | --- |
| E2B desktop template | `desktop` (e2b-desktop SDK default; `e2b_desktop` PyPI 2.6.0 / `e2b` 2.51.0) |
| Sandbox id | recorded in `/home/z/flauz-lane/e2b/state.json` (`sandbox_id`) |
| Screen | 1920×1080 (harness `resolution=(1920,1080)`) |
| Linux distribution | Debian-based E2B image (see `env-info.json` `os`) — verified per run |
| Desktop environment / WM | template default (X server on `DISPLAY` recorded per run) |
| Display backend | X11 (template Xvfb-class X server; GPUI renders via EGL/GLX surface) |
| GPU/renderer | software GL (llvmpipe) unless `env-info.json` reports otherwise |
| Flauz commit | pinned per run by `provision(commit)` — current lane base `f66965e` (main, F2 kernel freeze) |
| Rust toolchain | 1.97.1 (rustup, pinned by `rust-toolchain.toml`) |
| Codex CLI | `npm i -g @openai/codex` (version recorded in `env-info.json`) |
| Build | `cargo build --release -p codex-app` |
| Launch env | `DISPLAY` (desktop X server), `CODEX_RS_CODEX_BIN=$(command -v codex)` |
| App data | `$HOME/.local/share/codexRS` (codexRS-owned), `CODEX_HOME` default `~/.codex` |
| Key env vars | `CODEX_RS_CODEX_BIN`; no others required for the core journey set |

## Provision recipe (idempotent, marker-gated inside the sandbox)

1. `apt-get install` the Linux native build dependencies exactly as
   `docs/platform-support.md` (Ubuntu CI list) plus `build-essential git
   curl unzip tar`.
2. `rustup` default toolchain `1.97.1` with clippy + rustfmt.
3. `npm install -g @openai/codex` (runtime dependency; signed-out state is
   itself an honest journey, J-01/J-04).
4. `git clone` Flauz.app (token-scoped URL, token never enters any artifact).
5. `git checkout` the pinned commit; `cargo build --release -p codex-app`.
6. `launch_gui()`: kill prior instances, start the release binary under the
   desktop's X server, capture `~/.flauz/gui.log`.

## Operator-auth dependency (BOUNDED, honestly recorded)

The official Codex app-server requires ChatGPT authentication for any
model-backed journey step (streaming responses, live tasks). The lane runs
the app with the Codex CLI installed but signed out: the honest "sign in /
not signed in" product state is part of the J-01/J-04 cold-start battery.
Live model-backed journeys (streaming response, model switch while working)
remain **UNAVAILABLE-BUT-HONEST** until the operator performs a one-time
Codex sign-in inside the E2B desktop (the GUI offers the flow; the operator
would drive it via the harness screen stream). No operator credentials are
ever handled by the agent.

## Recreate-from-scratch procedure

```bash
# on the Lead station
source ~/.secrets/env.sh            # E2B_API_KEY, PAYSWAP_PAT
cd /home/z/flauz-lane/e2b
/home/z/.venv/bin/python3 -c "
from flauz_e2b import FlauzDesktop
fd = FlauzDesktop.create(timeout_s=3600)   # new sandbox (state.json updated)
fd.provision('<pinned-commit>')            # idempotent, ~15-40 min cold
fd.launch_gui(); fd.shot('fresh-boot')
"
```

Cold rebuild measured time is recorded in `prov.log` per run.
