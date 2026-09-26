# Platform support

## Support matrix

| Platform | Build | UI and app-server | Git and terminal | Computer Use |
| --- | --- | --- | --- | --- |
| Windows 10/11 x64 | Supported; unsigned portable archive | Locally source-build smoke-tested | Native Git and ConPTY; child trees use Job Objects | Supported and smoke-tested |
| Ubuntu 24.04 x64 | CI; unsigned portable tar.gz | Native UI and app-server build and tests in CI | Native Git and PTY | X11/XWayland screenshot observation when `DISPLAY` is set |
| Other Linux distributions | Source | Expected with equivalent native packages | Expected | X11/XWayland screenshot observation when `DISPLAY` is set |
| macOS | Not targeted | Not supported in this release | Not supported | Not supported |

The stable compatibility oracle is Windows Codex Desktop `26.721.3996.0` with
Codex CLI `0.146.0-alpha.3.1`. The runtime executable is still the user's
separately installed official Codex CLI.

## Windows

Requirements:

- Windows 10 or 11 on x64;
- Git;
- Rust `1.97.1` through `rustup`;
- Visual Studio 2022 Build Tools with Desktop development with C++ and a
  Windows 10/11 SDK;
- the official Codex CLI, signed in once.

Optimized GPUI builds compile their Windows shaders with the SDK's x64
`fxc.exe`. If the compiler is installed but is not on `PATH`, point GPUI at it:

```powershell
$fxc = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" `
  -Filter fxc.exe -Recurse |
  Where-Object FullName -Match '\\x64\\fxc\.exe$' |
  Sort-Object FullName -Descending |
  Select-Object -First 1
$env:GPUI_FXC_PATH = $fxc.FullName
```

codexRS resolves the CLI in this order:

1. an explicit probe argument;
2. `CODEX_RS_CODEX_BIN`;
3. a hash-pinned runtime copy of the CLI from the exact installed stable
   Desktop package;
4. the native executable inside the official global npm package, when present;
5. `codex.exe` on `PATH`.

Windows prevents third-party processes from launching the CLI in
`WindowsApps` directly. codexRS therefore verifies the exact stable executable
against the committed SHA-256 and copies it once into its owned local runtime
directory. The npm location remains only a binary discovery path. Node.js is
not started or linked by codexRS.

Native Computer Use binds every action to an exact discovery-returned
`Window { app, id, title? }`, rehydrates the opaque id, and verifies the
current owner. Packaged apps keep the case-preserved AUMID exposed by
`System.AppUserModel.ID`. Executables use stable's known-folder GUID form when
possible and otherwise keep a case-preserved absolute path. Legacy `process:`
values still match, but are no longer emitted. It excludes the codexRS process,
rejects oversized identifiers instead of truncating them, and does not offer
known shared hosts for persistent approval. Window-level AUMID resolution for
multiplexed host processes remains a tracked parity gap.
Task-only approval remains in memory. `Always allow` is read and written through
the official app-server at
`computer_use.windows.always_allowed_app_ids`; codexRS never opens the live
Codex config directly. Task and persistent approval cannot override the
stable-derived product-policy block for Codex, terminal, password-manager,
identity, and security targets. The dynamic-tool router rechecks that guard
after window-owner validation and again after a delayed approval; the supervised
accessibility helper enforces the same guard before reading or acting on a
window.

The same supervised boundary builds a bounded installed-app catalog from the
native AppsFolder namespace, the per-user and all-users Start Menu trees,
`%LOCALAPPDATA%\Microsoft\WindowsApps` execution aliases, and bounded
`WindowsApps` package manifests. AppsFolder reads the exact AppUserModel ID and
`System.Link.TargetParsingPath`; bounded process-name keys correlate UserAssist
records even when Windows reports a different executable path. A bounded
read-only UserAssist query adds the stable date-only `lastUsedDate` and integer
`useCount` fields without logging registry names or values. Differential checks
against stable cover all 40 reference IDs and display names and all 30 populated
usage records on the Windows smoke host. The catalog returns at most 40 apps
and resolves launch targets without shell-script parsing. Model-initiated
launches require the existing per-app approval; user-clicked Open buttons are
direct user actions. Stable's remaining uninstall, App Paths, package-repository,
and Appx registry sources plus the exact final source filter/order remain tracked
parity gaps.

The installed stable bundle ships `@oai/sky` 0.5.2 and isolates Windows UI
Automation behind a hidden line-delimited helper. codexRS follows that
behavioral boundary with its own native Rust helper inside the codexRS
executable; no official helper or Node runtime is redistributed or invoked.
Requests and responses are bounded, the helper is placed in a kill-on-close Job
Object, and every request has a 10-second deadline. Accessibility state is
limited to 512 elements and 128 KiB, cached indexes are scoped to the latest
text snapshot for one window, and password values are never read or set.
Screenshot IDs similarly scope coordinate remapping to the latest bounded
capture. Input actions foreground their exact target through the helper, and
the explicit `activate_window` recovery action uses the same supervised path.
For common Windows browsers, the helper also reads one shallowest,
unambiguous URL from UI Automation `Document.Value` without depending on a
localized address-bar label. HTTP(S) URLs are checked through the authenticated
stable site-status route before an action and re-read afterward; response
frames, redirects, deadlines, and turn-level failure state remain bounded.
Between Computer Use calls, a bounded native monitor observes fresh foreground
cursor movement, mouse-button edges, and common keyboard edges for the exact
target window. It invalidates the latest screenshot and requires a new
`get_window_state` before another action. Low-level wheel/touch capture and
injected-event distinction remain tracked Windows parity gaps.

Guarded input also requires the native system indicator to initialize and
become visible first. The window spans the bounded virtual-desktop geometry,
does not activate or intercept pointer input, remains topmost, and is excluded
from screenshots. A fixed sibling `codex-computer-use-overlay.exe` owns this
window behind bounded JSON-lines IPC and a kill-on-close Job Object; Windows
release archives ship it next to `codexrs.exe`. It uses the stable default
accent, top offset, width/radius limits, `Segoe UI Variable` typography, and exact
`ChatGPT is using your computer. Esc to cancel` accessible title. Stable's
separate animated cursor surface, DirectComposition shimmer/shadow fidelity,
and live locale/theme-accent updates remain tracked visual parity gaps.

## Linux

The Ubuntu CI installs these native build dependencies:

```bash
sudo apt-get install --yes --no-install-recommends \
  clang \
  libclang-dev \
  libdrm-dev \
  libegl1-mesa-dev \
  libfontconfig1-dev \
  libfreetype6-dev \
  libgbm-dev \
  libgl1-mesa-dev \
  libpipewire-0.3-dev \
  libwayland-dev \
  libx11-dev \
  libx11-xcb-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxcb1-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  pkg-config
```

Package names vary across distributions.

The published Linux tar.gz is a portable archive, not a system package. It
does not automatically install runtime dependencies, a desktop entry, or a URI
handler. `codexrs --install-desktop-entry` explicitly creates a per-user entry
for the current extracted binary. If the CLI is outside the desktop session's
`PATH`, invoke the installer with an absolute `CODEX_RS_CODEX_BIN`; the entry
captures that path. Existing entries are never changed, so remove and recreate
the entry after either binary moves. Ubuntu CI
starts the extracted archive in an isolated Xvfb session; broader
desktop-environment smoke coverage remains pending.

On Linux, Computer Use attaches only when `DISPLAY` is nonempty and exposes a
bounded, read-only X11/XWayland observation slice: window/app discovery,
window rehydration, and screenshot-only window state. It has no accessibility
text, input, launch, persistent app approvals, overlay, or interruption
monitoring. Pure Wayland without XWayland remains unavailable; a portal-backed
selection path is future work.

## Linux distribution strategy

Every distribution channel is named here with its status and its recovery
path. The release workflow publishes, per tag: the archives
(`codexrs-<tag>-linux-x86_64.tar.gz` and the Windows
`codexrs-<tag>-windows-x86_64.zip`), `SHA256SUMS.txt` with one SHA-256 entry
per packaged archive, and — once the signing key is provisioned — the
detached signature `SHA256SUMS.txt.asc`.

| Channel | Status | Recovery path |
| --- | --- | --- |
| Portable `tar.gz` + `SHA256SUMS.txt` | **Supported** — the release channel (Ubuntu 24.04 CI builds, packages, and desktop-smokes it) | n/a — supported; verify every download as described below |
| Detached OpenPGP signature (`SHA256SUMS.txt.asc`) | **Scaffolded, not yet produced** — the workflow signs automatically once the key exists and skips with a named log line otherwise (never a fake signature, never a failed build) | The operator provisions the `FLAUZ_RELEASE_SIGNING_KEY` repository secret (an ASCII-armored OpenPGP private key; optional `FLAUZ_RELEASE_SIGNING_KEY_PASSPHRASE`) and re-runs the release workflow |
| AppImage | **Not shipped — under evaluation** | The CI environment has no AppImage toolchain today (`appimagetool`/`linuxdeploy` are not installed; the honest environment bound). Recovery: add the toolchain and a packaging step to CI, run the desktop startup smoke against the AppImage, then decide against the parity matrix |
| `.deb` package | **Not shipped — under evaluation** | CI has no deb packaging path today (`cargo-deb` is not a dependency and no packaging step exists) and no apt repository exists to distribute through (the honest environment bound). Recovery: choose `cargo-deb` or a manual `dpkg-deb` step, decide the apt repository question, then wire it into the release workflow |
| Snap / Flatpak | **Not evaluated** | Out of current scope; revisit after the tar.gz channel carries signatures |
| Source build | **Supported** | See the Linux build dependencies above |

Until the signing key is provisioned, the checksum manifest detects
corruption and accidental mismatch after download; it is not an independent
publisher signature (the release notes state the same bound). The manifest
and the archives travel over the same channel, so a determined
man-in-the-middle could alter both; the detached signature closes exactly
that gap once the key exists.

### Verifying a downloaded archive

Download the archive and `SHA256SUMS.txt` (and, when published,
`SHA256SUMS.txt.asc`) from the same GitHub release, then, from the directory
holding them:

```bash
# with the repository script (from a checkout of this repository); it also
# verifies the detached signature when one is present next to the manifest
bash scripts/verify_release_archive.sh codexrs-<tag>-linux-x86_64.tar.gz SHA256SUMS.txt

# or directly, without a checkout (the --ignore-missing flag lets you verify
# only the archives you downloaded)
sha256sum --check --ignore-missing SHA256SUMS.txt

# when SHA256SUMS.txt.asc is published, after importing the publisher's
# public key
gpg --verify SHA256SUMS.txt.asc SHA256SUMS.txt
```

`scripts/verify_release_archive.sh` hard-fails with a named message when the
manifest is malformed, when it has no (or duplicate) entries for the archive,
when the checksum does not match, or when a present signature does not
verify. Do not extract, run, or enable Computer Use from an archive that
failed verification.

## Runtime directories

The two data boundaries are independent:

| Variable | Purpose | Default |
| --- | --- | --- |
| `CODEX_HOME` | Official app-server-owned Codex state | `~/.codex` |
| `CODEX_RS_DATA_DIR` | codexRS-owned UI preferences, recent workspaces, and managed worktrees | OS data directory |
| `CODEX_RS_CODEX_BIN` | Native official Codex executable | `codex` / `codex.exe` resolution |

Never point development tests at a live `CODEX_HOME`. Create an isolated
directory and keep fixtures free of credentials and user history.
