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
invisible (L-004); no logging backend exists (L-005); visual parity with
the product's client-side decoration design is unverified on Linux (xfwm4
server-side decorations currently shown).

## Run-4 update (2026-09-21, signed-out sub-battery)

Run-4 (sandbox `izqinqfuxeylk815bn5sf`, auto-patched build, 469 s cold
reprovision — third consecutive reproducible environment) advanced the
matrix past cold start as far as the honest signed-out gate allows:

- **J-01, J-11 WORKS**; **J-03, J-04, J-14 WORKS-WITH-DEFECTS**;
  J-02/J-06..J-10/J-12/J-13/J-15/J-16/J-18 honestly auth-gated (Codex
  sign-in = operator dependency, surfaced);
- **J-05 = SILENT-NO-OP** (Projects "+" does nothing signed-out — L-009,
  P2) and **J-17 Activity surface not found** signed-out (MISSING pending
  signed-in verification) — the two open acceptance gaps;
- L-006 corrected (Terminal honest toast confirmed; run-3 dead-click was a
  coordinate miss); new findings L-007..L-012 registered;
- Toasts never auto-dismiss (L-011); Workflows surfaces a raw JSON-RPC
  error (L-008); composer drafts are lost across restart (L-010).

Verdict UNCHANGED: **WORKS-WITH-DEFECTS** — the patched build's shell and
navigation are solid and honest in most signed-out states, but the shipped
`main` remains P0-broken (L-002) until LAB-003 merges, and J-05/J-17 leave
open acceptance questions.

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
