# Runtime compatibility policy

This document is the single source of truth for which Codex runtimes a
released Flauz.app build supports, and how the product behaves when a
runtime does not provide a capability. It is referenced by the release
work order (`WORK-ORDERS.md` § GUI-006) and the platform support
matrix (`docs/platform-support.md`).

## Runtime discovery

Flauz.app never bundles or links a second agent runtime. It resolves
the user's Codex CLI in the order documented in
`docs/platform-support.md` (explicit probe argument, then
`CODEX_RS_CODEX_BIN`, then the hash-pinned runtime copy on Windows,
then the npm global package binary, then `PATH`), then starts the
official app-server protocol over the supervised boundary.

## Base surfaces (parity scope)

The Codex Desktop parity surfaces — tasks, composer, timeline, Git,
terminal, files, browser, Computer Use, settings — target the stable
compatibility oracle recorded in `docs/platform-support.md`:
Windows Codex Desktop `26.721.3996.0` with Codex CLI
`0.146.0-alpha.3.1`. These surfaces degrade per-feature, not
whole-app: a runtime without a method fails that request with an
actionable error, and the app stays usable.

## Universal workflow family

The Universal workflow surface (teaching, candidates, publication,
runs, evidence, versions, fork and improvement) uses the
`workflow/*` app-server method family. This family is served when the
connection initializes with `experimentalApi: true`; Flauz.app always
sends that capability flag.

**Compatibility version: `workflow/*` family, first served by
`payswapdotorg/codex` release `rust-v0.1.0`.**

Behavior with runtimes that do not serve the family:

- The rest of the application is unaffected; only workflow requests
  fail.
- Every workflow request fails closed with an actionable control-plane
  error surfaced in the Workflows surface; nothing is cached,
  guessed, or synthesized client-side.
- The GUI never falls back to a terminal or to a client-side workflow
  implementation; there is no second engine.

## Pack contracts

No Pack contract family is served by any released runtime at the time
of this release. The GUI remains Pack-unaware by design; when the
governed Pack contracts land (`PACK-001`…`PACK-004` in
`payswapdotorg/codex`), the Pack-aware views (`PACK-UX-001`) will
consume them read-only through the same supervised boundary, and this
document will record the first Pack-serving runtime release.

## Release-to-runtime matrix

| Flauz.app release | Base oracle | Workflow family | Pack family |
| --- | --- | --- | --- |
| `v0.1.0-rc.13` | Desktop `26.721.3996.0` / CLI `0.146.0-alpha.3.1` | `rust-v0.1.0` (`payswapdotorg/codex`) | none served |

## Version pinning rules

- A Flauz.app release records the exact runtime versions it was
  validated against (the matrix above); CI smoke tests validate the
  packaged archive, and the real-runtime integration suite runs
  against the recorded workflow-family runtime.
- Adding a workflow-family method to the GUI requires a runtime that
  serves it; the GUI keeps no compatibility shims and synthesizes no
  responses.
- Breaking protocol changes in `payswapdotorg/codex` require a new
  row in the matrix and a re-validation before the next Flauz.app
  release.
