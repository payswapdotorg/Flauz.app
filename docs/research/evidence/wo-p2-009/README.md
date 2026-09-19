# WO-P2-009 Lab Evidence — persistent browsing-history store + address-bar revisit + Settings management

Delivery under test: `feat/wo-p2-009-browsing-history` @ `5c795e7` (the
over-delivered third push; supersedes `96d611e`) → Lead rebase onto main
`d479c7b` → `0eed8c3` (single trivial ui.rs test-import conflict,
union-resolved + fmt) → PR #24 from `feat/wo-p2-009-verified` → merged
`5302e7e` (2026-09-19, CI green both matrices). Focused tests verified
locally with per-test output at harvest: storage 2/2, core 3/3, platform
2/2 (platform needed the lab sysroot); codex-app binding tests delegated to
the authoritative CI matrix (local OOM discipline with the live replay
stack).

## D12 — Settings Browser scene (offline-honest scope)

`scenes/d12-browsing-history.sh` — same doctrine as D11b: the desktop app
needs the official codex CLI runtime (absent in this lab; no fabrication);
the browser PANEL (address-bar revisit) requires a live browser surface the
runtime-less entry surface does not expose — recorded NOT RUN (covered by
codex-platform matching tests + codex-app binding tests on CI).

What the scene verifies on the real binary: (A) first launch migrates
state to schema v5 + seeded rows persist; (B) Ctrl+K → "browser settings"
resolves the Settings > Browser section with the seeded history rows
visible; (C) Clear… opens the confirmation modal; confirming clears the
history (empty state + disabled Clear).

## The integration bug — and the five build/verify cycles that isolated it

**Finding (d12/):** every history row rendered a lone `…` — titles and
URLs collapsed while timestamps rendered. The bug survived three
first-theory fixes (flex_1; justify_start; w_full) with **byte-identical
frames each time** — which itself became the diagnostic: pixel forensics
(ink-cluster measurement at native resolution, d12c/d12e) showed the
text divs at ~0 width in every build.

**Root cause (isolated by differential analysis against the in-repo
proven pattern):** the row's text stack diverged from the repo's working
convention in several stacked ways — `justify_between` mixed with a
basis-0 `flex_1` column, `min_w_0` + `truncate()` directly on the child
divs, and `gap_1` on the column. The command-palette row
(`render_command`, VLM-verified rendering in evidence `wo-p2-010/d13/`
frame 5) is the repo's proven two-line pattern: a stateful row div, a
`flex_1`+`min_w_0` column WITHOUT gap, a plain title div (no truncate —
wraps), and a `truncate()` div for the secondary line.

**Fix (`3617789`):** the history rows and the downloads-modal rows
(same text-stack class) now mirror the palette-row convention exactly.

| Run | Binary | Phase A | Phase B rows | Phase C (Clear) |
|---|---|---|---|---|
| d12/ (06:33) | pre-fix (post-merge main) | PASS | **FAIL — `…`-only rows** (crop5 zoom) | — (driven later on d12b) |
| d12b/ (07:15) | + flex_1 attempt | PASS | FAIL — frames **byte-identical** to d12 | PASS (modal verbatim; cleared + empty state + disabled Clear) |
| d12c/ (08:1x) | + justify_start | PASS | FAIL — still byte-identical (grow works; divs still 0-wide — pixel-proven) | — |
| d12e/ (09:0x) | same binary, mixed seed lengths (incl. 1-char title) | PASS | FAIL — even 1-char text renders `…` ⇒ zero-width divs, not content | — |
| **d12f/ (09:1x)** | **palette-convention clone** | **PASS** | **PASS — rows verbatim**: "Hacker News"/"news.ycombinator.com"/148w; "Chat"/"chatgpt.com/c/abc123"/148w; "payswapdotorg/Flauz.app"/"github.com/payswapdotorg/Flauz.app"/148w (first frame-hash change across all builds; ink 0→1112 px) | **PASS** — modal "Clear browsing history?" / "This removes all stored browsing history from this device" / Cancel+Clear → 0 rows, "No browsing history yet", Clear disabled |

d12e/ holds the seed-length experiment frames; d12f/ the final full-pass
frames + md5s + VLM reads.

## Residuals (documented, not fabricated)

- Browser panel / address-bar revisit surfaces: NOT RUN (runtime-less entry
  surface, D11b doctrine) — covered by codex-platform matching tests +
  codex-app binding tests on CI.
- Downloads-modal rows (same text-stack fix applied): GUI verification is
  runtime-gated in this lab (the modal opens from the browser surface);
  the fix mirrors the same proven convention, source-cited.
