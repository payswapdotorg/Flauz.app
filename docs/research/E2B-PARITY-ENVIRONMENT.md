# E2B Parity Environment — Official Codex GUI ↔ Flauz GUI Laboratory

Phase-0 lab establishment report (Codex parity mission).
Author: Tech Lead, resident operator. Date: 2026-09-16 (UTC).
Status: **Lab established. Linux GUI pair operational. Windows/macOS pairs
unavailable — platform-bound, evidenced below.**

The comparison target for this mission is the **official Codex GUI** (the
desktop product), not the Codex CLI. This document records what environments
could and could not be created, with evidence, per the operator's platform
rule: no silent substitution, no fabricated platform validation.

---

## 1. Lab verdict (required report)

```text
WINDOWS_GUI_LAB: unavailable
MACOS_GUI_LAB:   unavailable
LINUX_GUI_LAB:   available (LOCAL Debian 13 sandbox — proven end-to-end)
```

> **Lab upgrade (2026-09-17, WO-LAB-001 — dated entry, not a silent bound
> change):** LINUX_GUI_LAB now runs BOTH sides of the Linux pair. The
> official Linux preview app (`chatgpt 26.908.70816`, `latest` `.deb`,
> selective userspace at `/home/z/parity-lab/official-app/`) launches under
> Xvfb + picom with an isolated profile and no credentials. Runtime evidence
> (unauthenticated slice only) is archived under
> `docs/research/evidence/codex-linux/`; affected official Linux cells in the
> parity report carry the version-skew label
> `[runtime-observed: linux-preview 26.908.70816]`. Platform bounds are
> otherwise UNCHANGED: official account surfaces remain auth-walled;
> WINDOWS_GUI_LAB and MACOS_GUI_LAB remain unavailable (evidence below).

### WINDOWS_GUI_LAB: unavailable — evidence

1. E2B sandboxes are Linux microVMs; templates are Linux Docker images
   (account templates API returns Linux images only; no Windows/macOS
   template mechanism exists in the E2B API surface in use).
2. This environment has no Windows host, no Windows VM, and no hosted
   Windows service credentials.
3. The official Codex GUI **does** ship for Windows
   (`OpenAI.Codex_26.721.3996.0_x64__2p2nqsd0c76g0` MSIX — recorded in
   `reference/stable-26.721.3996.0/manifest.json`), but there is no
   environment in this laboratory that can run it.
4. Consequence: the Windows pair (Official Codex ↔ Flauz Windows GUI) is
   **deferred** until a Windows environment is provided. Windows-side
   evidence is limited to `[historical-record]` and `[docs-derived]`
   provenance.

### MACOS_GUI_LAB: unavailable — evidence

1. Same E2B Linux-only bound.
2. No Mac hardware in this environment.
3. codexRS itself does not target macOS (`docs/platform-support.md`), so the
   macOS Flauz side does not exist either. The macOS pair is doubly
   unavailable and is **out of scope** for this phase.

### LINUX_GUI_LAB: available (LOCAL) — evidence

The local sandbox (Debian 13 trixie, x86_64, 2 vCPU, 4 GB RAM, no GPU) runs
the real Flauz GUI end-to-end. Verified chain (2026-09-16, Tech Lead):

1. Flauz `v0.1.0-rc.13` release binary (== `main` @ `f113515`; GUI-007 was
   docs-only) launches under `Xvfb` with userspace-extracted runtime libs.
2. GPUI 0.2.2 renders via blade/Vulkan — satisfied by **lavapipe**
   (software Vulkan, `mesa-vulkan-drivers` 25.0.7 extracted userspace, no
   root, via `VK_ICD_FILENAMES`).
3. The codexRS window is a **32-bit ARGB** visual — it renders no content
   without a compositor; **picom** (`--backend xrender`, userspace) is
   required and sufficient.
4. First frame requires an input/focus event on bare Xvfb (no WM):
   a single `xdotool` click wakes rendering (verified: frame went 4,877 →
   58,381 bytes at t+10s after click).
5. Capture: `ffmpeg -f x11grab`. Verification: VLM (`z-ai vision`) read the
   rendered UI: sidebar (New chat / Repository / Pull requests / Plugins /
   Workflows / Projects / Chats / Settings / "App-server online"), welcome
   card "Sign in to get started", composer with model/reasoning/speed
   pickers. Screenshot of record: `50-s3-7.png` (parity-lab local archive).
6. The supervised official `codex app-server` runs (state DBs created,
   migrations through v42, plugin sync performed), and `codexrs probe`
   **passes with both** the official CLI `0.146.0-alpha.3.1` and the fork
   runtime `rust-v0.1.0`.

**E2B's role for Linux**: E2B Linux sandboxes were tested and are **not**
used for the GUI lab: the `base` template is Debian 12, 2 vCPU, **478 MB
RAM**, 22 GB disk — insufficient for GPUI+lavapipe (and the GUI-stack apt
install failed inside it, rc=100). A custom `flauz-parity` template
(ubuntu:24.04, cpu_count=2, memory_mb=2048) was built twice by the prior
session and **failed both times** (`buildStatus: error`; the ready command
`while true; do sleep 1; done` never terminates — a template-build bug worth
fixing only if E2B isolation becomes a requirement). The public
`openai-codex` E2B template was probed: it provides codex-cli **0.11.0**
(research preview), unauthenticated, with no GUI stack — retained as
CLI-shape evidence only (see `e2b-evidence/workspace-a.json`).

---

## 2. Environment records (§9)

### Linux-B — Flauz GUI (LOCAL; the implementation under test)

| Field | Value |
| --- | --- |
| OS | Debian GNU/Linux 13 (trixie) |
| Architecture | x86_64 |
| Display server | Xvfb (X11), screen 1600x1000x24 |
| GUI transport | local X socket; ffmpeg x11grab capture (no VNC) |
| Compositor | picom (xrender backend) — **required** (32-bit ARGB window) |
| Vulkan | lavapipe software ICD (userspace mesa-vulkan-drivers 25.0.7) |
| Flauz version | v0.1.0-rc.13 (release tarball; code == main `f113515`) |
| codex runtime | fork `rust-v0.1.0` (probe-verified; official `0.146.0-alpha.3.1` also probe-verified) |
| Installation source | GitHub release `v0.1.0-rc.13` (`codexrs-v0.1.0-rc.13-linux-x86_64.tar.gz`) |
| Launch command | `/home/z/parity-lab/session.sh <ws> :101 <bin> <codex-bin> <scene>` |
| Runtime deps | libxkbcommon(-x11), libxcb-*, libfontconfig1, libfreetype6, libwayland-*, libegl/gles/gbm/drm, libxkbcommon, lavapipe+libLLVM19 (all extracted userspace under `/home/z/parity-lab/tools/rootfs`) |

### Linux-A — official Codex reference (evidence layers, not a GUI)

The official Codex GUI (Codex Desktop 26.721.3996.0; current installed
reference ChatGPT desktop 26.825.51511) is a **closed-source Electron
application** shipped for Windows/macOS only (manifest:
`ChatGPT.exe`, `chrome.dll`, `app.asar`, `codex.exe` — owl/Chromium
150.0.7871.128). Upstream `openai/codex` contains **no desktop crate**
(verified by repository listing: cli/tui/app-server/exec/… only), so the
official GUI cannot be built or run on Linux anywhere. Linux-A therefore
consists of **reference layers**, each row in downstream matrices carries a
provenance label:

1. `[historical-record]` — in-repo captures from the 26.721.3996.0 era:
   `docs/parity-matrix.md`, `docs/known-failures.md`,
   `reference/stable-26.721.3996.0/manifest.json`, `docs/codex-universal/**`
   (GUI-001..007 reports, boundaries, work orders), CHANGELOG.
2. `[runtime-observed]` — official runtime behavior reproducible in this
   lab: official Codex CLI `0.146.0-alpha.3.1` (downloaded from the
   upstream release, `codexrs probe` handshake verified from this machine),
   `codexrs info` bootstrap output (reference: OpenAI.Codex 26.721.3996.0,
   CLI 0.146.0-alpha.3.1, Owl/Chromium 150.0.7871.128).
3. `[docs-derived]` — public release notes/announcements for versions
   between the baseline and the current installed reference
   (26.721.3996.0 → 26.825.51511), plus upstream `openai/codex` release
   evolution (rust-v0.146.0-alpha.3.1 → rust-v0.154.0, latest) as the
   runtime/protocol delta signal.

---

## 3. Reference layers (§6)

```text
Historical baseline: Codex Desktop 26.721.3996.0 + CLI 0.146.0-alpha.3.1
        +
Current installed:   ChatGPT desktop 26.825.51511 (operator's machine)
        =
Current parity target (delta research → docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md)
```

Historical parity work is retained and forms the baseline layer; nothing
from it is discarded.

---

## 4. Isolation rules (§5)

- Workspace A (official reference evidence) and Workspace B (Flauz) keep
  **separate** `HOME`, `XDG_DATA_HOME`, `XDG_RUNTIME_DIR`, `CODEX_HOME`,
  `CODEX_RS_DATA_DIR`, projects, screenshots, and artifacts
  (`/home/z/parity-lab/linux-A` vs `/home/z/parity-lab/linux-B`).
- The codex CLI is **never** executed without `CODEX_HOME` set (a bare CLI
  writes to `~/.codex` — observed and prevented).
- Lab displays `:100`/`:101`; display `:99` belongs to the operator console
  stack and is never touched.
- E2B and GitHub credentials live only in the secure env file; they are
  never echoed, logged, committed, or placed in sandbox filesystems.

---

## 5. Honest bounds of this laboratory

1. **No authenticated Codex account** exists in the lab. Account-powered
   flows (live agent turns, cloud tasks, marketplace installs requiring
   auth) are evidenced only through UI-shell behavior, source, and protocol
   surfaces; every such row is bound-labeled rather than guessed.
2. **Windows and macOS pairs cannot be validated here.** Any Windows-only
   behavior (e.g., full native Computer Use) stays
   `[historical-record]`/`[docs-derived]` until a Windows environment is
   provided.
3. **E2B is not the GUI lab** (resource-insufficient at base; custom
   template build failed twice — root cause identified, fix deferred).
4. The local box is memory-constrained (4 GB; OOM killer active). Lab
   sessions are single-call scoped via `session.sh`; one GUI session at a
   time.

---

## 6. Artifact index

- Lab infrastructure (local, not in-repo): `/home/z/parity-lab/`
  (`session.sh`, `tools/scene_helpers.sh`, `tools/fetch_debs.py`,
  `tools/rootfs/`, `linux-B/`, `scenes/`, `artifacts/`).
- E2B probes (prior session, retained): `/home/z/my-project/e2b-evidence/`
  (`workspace-a.json` openai-codex template inventory; `workspace-b.json`
  base-template GUI attempt; journey probes).
- Screenshot of record for the first verified render:
  `50-s3-7.png` (local archive), VLM reading reproduced in
  `/home/z/parity-lab/README.md`.
