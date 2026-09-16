# GUI-006 — Distribution + Release

**Status:** COMPLETE
**Base:** `c300b6b` (merged GUI-005 + release groundwork, PR #6)
**Release:** `v0.1.0-rc.13` — first Flauz.app release
**Work order:** `docs/codex-universal/WORK-ORDERS.md` § GUI-006

## What was delivered

### Release: Flauz.app v0.1.0-rc.13

Published from tag `v0.1.0-rc.13` by the `Release` GitHub workflow
(run 35071020086, all four jobs green):

- **Third-party licenses** — `THIRD_PARTY_LICENSES.html` inventory
  (cargo-about), dependency policy 561 packages.
- **Package windows-x86_64** — full gates (fmt, patched-renderer
  clippy/test, workspace clippy `-D warnings`, workspace tests,
  release build), Windows ZIP packaging, archive startup smoke.
- **Package linux-x86_64** — same gates, tar.gz packaging, isolated
  Xvfb startup smoke (`scripts/linux_desktop_smoke.sh`).
- **Publish GitHub release** — `SHA256SUMS.txt` + release notes
  describing the Universal workflow surface, runtime requirements,
  and the honest unsigned-portable posture.

Assets (3): `codexrs-v0.1.0-rc.13-windows-x86_64.zip`,
`codexrs-v0.1.0-rc.13-linux-x86_64.tar.gz`, `SHA256SUMS.txt`.
Marked prerelease (rc tag). Title: "Flauz.app v0.1.0-rc.13".

### Version and compatibility policy

`docs/codex-universal/RUNTIME-COMPATIBILITY.md` (new): base-surface
oracle (Desktop `26.721.3996.0` / CLI `0.146.0-alpha.3.1`), the
Universal workflow compatibility version (`workflow/*` family first
served by `payswapdotorg/codex` `rust-v0.1.0`, gated on
`experimentalApi: true`), the Pack-family forward statement (no
runtime serves it yet), the release-to-runtime matrix, and pinning
rules.

### Platform parity closure

The two GUI-002-deferred platform rows in `docs/parity-matrix.md`
resolved to bounded with reasons: Windows unsigned-portable strategy
(signing infrastructure unavailable to this program; checksums +
verify steps documented), Linux portable tar.gz + explicit
`--install-desktop-entry` (no system package/portals/tray/global
shortcuts, documented in `docs/platform-support.md`).

### Release hygiene

- Version bumped to `0.1.0-rc.13` (first Flauz.app release; upstream
  codexRS ended at rc.12); CI tag validation satisfied.
- README/README.ru/README.zh-CN download badges and checksum links
  repointed to `payswapdotorg/Flauz.app` releases at rc.13.
- CHANGELOG `0.1.0-rc.13` entry documenting the Universal workflow
  surface, version lifecycle, multi-environment UX, runtime policy,
  and distribution.
- `scripts/fresh_machine_validate.sh` added: reproducible
  fresh-machine validation of a published release.

## Fresh-machine validation (executed, Linux)

Via `scripts/fresh_machine_validate.sh` against the published
release, isolated HOME/XDG/CODEX_HOME, no source checkout:

| Step | Result |
| --- | --- |
| Download release tar.gz + SHA256SUMS | ✅ |
| Checksum verify | ✅ `696969e9…` matches published sums |
| Install (extract portable archive) | ✅ |
| `codexrs info` (bounded runtime information) | ✅ stable reference + limits printed |
| Desktop integration (`--install-desktop-entry`, fresh XDG) | ✅ `com.codexrs.CodexRS.desktop` with `Exec=` |
| Runtime probe (Universal runtime, fresh CODEX_HOME) | ✅ app-server initialized, home verified |
| Restart (second probe, same home) | ✅ |

Workflow teaching/execution validation on the exact tag tree:
`CODEX_RS_TEST_CODEX_BIN=<codex rust-v0.1.0> cargo test -p codex-platform`
→ **112 passed + 2 real-runtime integration passed** (the same
compiled boundary the released binary embeds).

## Honest bounds of this validation

- The sandbox has no Vulkan ICD, so the graphical UI drive (mouse
  and keyboard through create → teach → publish → run) could not run
  here. CI owns the graphical archive startup smoke on real runner
  hardware (Xvfb + mesa-vulkan-drivers); the UI state machine is
  covered by the 210 core reducer tests and the workflow lifecycle
  by the real-runtime boundary suite above.
- Sign-in with real account credentials is not exercisable in this
  environment. The workflow family is control-plane state and is
  proven auth-independent by the boundary suite; the sign-in surface
  is the official runtime flow.
- Windows fresh-machine execution is owned by the CI Windows archive
  smoke (expand, binary presence, startup smoke); no Windows machine
  exists in this program's sandbox.
- The full user path from the work order's acceptance ("sign
  in/configure" through graphical teaching) is therefore validated
  piecewise: archive integrity + startup + integration + runtime
  connectivity here and in CI, workflow lifecycle against the real
  runtime in the boundary suite, and the reducer-verified UI flow —
  not as one continuous graphical session.

## Verification

```
# CI (run 35071020086): licenses + dep policy + fmt + clippy(-D warnings)
#   + workspace tests + release builds (win/linux) + packaging
#   + archive startup smokes + SHA256SUMS + release publish        → all green

# Local, on the exact tag tree (v0.1.0-rc.13):
CODEX_RS_TEST_CODEX_BIN=<codex rust-v0.1.0> cargo test -p codex-platform
                                                                 → 112 + 2 passed
bash scripts/fresh_machine_validate.sh                           → FRESH_MACHINE_VALIDATION_OK
git grep -E "ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9]{20,}" <tag>
                                                                 → no matches (secret scan)
```
