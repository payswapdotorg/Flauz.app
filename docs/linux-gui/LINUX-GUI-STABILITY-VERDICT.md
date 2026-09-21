# Linux GUI — Stability Verdict

**Lane:** Linux GUI usability (second Tech Lead — Linux lane).

The verdict is derived ONLY from E2B desktop journey evidence
([LINUX-GUI-USER-JOURNEY-MATRIX.md](LINUX-GUI-USER-JOURNEY-MATRIX.md)), never
from `cargo test`, CI or a successful source build alone (operator directive).

## Current verdict

**WORKS-WITH-DEFECTS — after the run-3 in-lane fix (2026-09-21), the Linux
GUI is REAL and INTERACTIVE in the E2B desktop; the shipped `main` binary
remains P0-broken (L-002) until LAB-003 merges.**

Run-3 (sandbox `io23l6hz0z2g7mqoq54om`, Flauz @ `f66965e` + one-line
in-lane gpui patch + native codex binary) demonstrated, with real mouse and
keyboard through the E2B desktop:

- J-01 cold start WORKS: window renders the full product shell — sidebar
  (New chat / Repository / Pull requests / Plugins / Workflows / Projects /
  Chats / Terminal / Browser / Settings, `App-server online` status), main
  content ("What should we work on?"), composer (placeholder, model/
  effort/mode selectors, submit), onboarding modal (dismissed by click),
  honest signed-out auth gate ("Sign in to get started" with four auth
  paths), keyboard input into the composer (typed text + blinking cursor),
  command palette via Ctrl+Shift+P (search + keyboard hints + navigation),
  and Settings → Appearance navigation via palette search + Enter.
- Window own-content check: `xwd` mean 246.3 (was 0.0 pre-fix).

Caveats that keep the verdict from `WORKS`: the fix is **not merged**
(`main` still ships the L-002 P0); the app-server-failure error state is
invisible (L-004); no logging backend exists (L-005); J-02..J-18 remain
unre-run against the patched build; visual parity with the product's
client-side decoration design is unverified on Linux (xfwm4 server-side
decorations currently shown).

The verdict will state, per acceptance area:

1. fresh Linux desktop → download/install/launch → first run;
2. create task / Workspace / composer / streaming response / model switch /
   command palette / Settings / terminal / Browser / Activity /
   Git workflow / reconnect / close-reopen / keyboard-only core journey;
3. future-platform surfaces (Context, Agents, Environments, Evidence,
   Procedures, Resources, Collaboration) — each `WORKS` or
   `UNAVAILABLE-BUT-HONEST` (never HIDDEN / SILENT-NO-OP / MISLEADING).

## Acceptance standard (operator text, binding)

> Could a new user sitting in the E2B Linux desktop discover, understand,
> use, recover from, and continue this capability without knowing Flauz's
> internal architecture?
