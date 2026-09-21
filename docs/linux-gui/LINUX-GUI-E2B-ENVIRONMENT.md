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
| Plan limits (run-3) | create-timeout hard cap **1 h** (API 400 above), idle sandboxes paused/cleaned (~40 min observed) → destroy-and-recreate is mandatory |
| Screen | 1920×1080 (harness `resolution=(1920,1080)`) |
| Linux distribution | Ubuntu 22.04.5 LTS (kernel 6.1.158+), 8 vCPU / 7.8 GiB |
| Display server | **`Xvfb :0 -ac -screen 0 1920x1080x24 -retro -dpi 96`** (verified run-3) |
| Desktop environment / WM | Xfce4 with **xfwm4** (WM present; compositing not relied upon) |
| Vulkan (renderer) | loader 1.3.204; ICDs installed by the lane harness: intel, intel_hasvk, **lvp (lavapipe)**, radeon, virtio (`mesa-vulkan-drivers`); app GPU = lavapipe (software). GPUI renders via **blade/Vulkan**, NOT EGL (run-3 correction) |
| OpenGL (control) | llvmpipe (LLVM 15.0.7, 256 bits), GL 4.5 compat — `glxgears`/`vkcube` present correctly (run-3 control) |
| Flauz commit | pinned per run by `provision(commit)` — current lane base `f66965e` (main, F2 kernel freeze) |
| Rust toolchain | 1.97.1 (rustup, pinned by `rust-toolchain.toml`) |
| Codex CLI | **native platform binary** extracted from `@openai/codex@0.146.0-alpha.3.1-linux-x64` tarball (`package/vendor/x86_64-unknown-linux-musl/bin/codex`), symlinked as `codex` — the npm wrapper `bin/codex.js` requires Node ≥16 and crashes on the image's Node 12.22.9 (top-level await). NEVER `npm i -g @openai/codex` on this image (L-004). |
| Build | `cargo build --release -p codex-app` (plus `~/pw10` pipewire side-load per L-001) |
| Launch env | `DISPLAY=:0`, `CODEX_RS_CODEX_BIN=$(command -v codex)`, `LD_LIBRARY_PATH=~/pw10/usr/lib/x86_64-linux-gnu` |
| App data | `$HOME/.local/share/codexRS` (codexRS-owned), `CODEX_HOME` default `~/.codex` |
| In-lane render fix (until LAB-003 merges) | `[patch.crates-io] gpui = { path = "~/gpui-patched" }` with `x11/window.rs` forcing `visual_set.inherit` (24-bit) instead of the 32-bit ARGB transparent visual — root cause L-002 |

## Provision recipe (idempotent, marker-gated inside the sandbox)

1. `apt-get install` the Linux native build dependencies exactly as
   `docs/platform-support.md` (Ubuntu CI list) plus `build-essential git
   curl unzip tar` and the lane diagnostics/Vulkan stack: `imagemagick
   x11-apps mesa-utils strace x11-utils xdotool procps mesa-vulkan-drivers
   vulkan-tools libvulkan1 xcompmgr`.
2. `rustup` default toolchain `1.97.1` with clippy + rustfmt.
3. Codex CLI **native binary** (pinned 0.146.0-alpha.3.1):
   `curl -sL -o cx.tgz
   https://registry.npmjs.org/@openai/codex/-/codex-0.146.0-alpha.3.1-linux-x64.tgz`
   → extract → symlink `vendor/x86_64-unknown-linux-musl/bin/codex` as
   `codex`. (The npm wrapper requires Node ≥16; the image ships Node 12 —
   L-004. The native binary needs no Node at all.)
4. `git clone` Flauz.app (token-scoped URL, token never enters any artifact).
5. `git checkout` the pinned commit; `cargo build --release -p codex-app`.
6. (Until LAB-003 merges) apply the in-lane gpui visual patch:
   copy `~/.cargo/registry/src/*/gpui-0.2.2` → `~/gpui-patched`, change the
   `visual_set.transparent` match in `src/platform/linux/x11/window.rs` to
   `let visual = visual_set.inherit;`, add `[patch.crates-io] gpui =
   { path = "~/gpui-patched" }` (merged into the EXISTING patch section),
   rebuild.
7. `launch_gui()`: kill prior instances, start the release binary under the
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

## Sandbox run record

| Run | Sandbox ID | Date | Outcome |
| --- | --- | --- | --- |
| 1 | `ipoaz31c43jj5vtjy8qvf` | 2026-09-21 | baseline sweep; expired via pause-cleanup (~40 min idle limit on this plan) |
| 2 | `ivmy0lvttlc2csmbklaay` | 2026-09-21 | reproducibility proof (460 s cold reprovision); L-002 reproduced identically |
| 3 | `io23l6hz0z2g7mqoq54om` | 2026-09-21 | root-cause discrimination + in-lane fix validation + J-01; expired at the 1 h plan cap |
| 4 | `izqinqfuxeylk815bn5sf` | 2026-09-21 | signed-out J-02..J-18 sub-battery (auto lane patch; 469 s cold reprovision) |

Plan constraints re-confirmed every run: 1 h max create-timeout, ~40 min
idle pause-cleanup — the destroy-and-recreate contract is mandatory.
