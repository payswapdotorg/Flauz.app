# RG-A11Y binding-pass verdict — WO-UX-001 at the published v0.1.0-rc.14

- **Work order:** WO-UX-001 (keyboard-only journey + focus-order probes) — the
  binding pass per A11Y-BASELINE-VERDICT §5
- **Tree:** binary `codexrs-v0.1.0-rc.14` — the PUBLISHED release archive
  (`codexrs-v0.1.0-rc.14-linux-x86_64.tar.gz`, sha256
  `9eb1139caeca6c6b9fec6c968715f7c715c4fa509dffa42609ef73b0002505d4`,
  re-downloaded 2026-09-21 and verified against the published SHA256SUMS)
- **Date:** 2026-09-21 (runs 10:54–11:22 UTC)
- **Scenes:** `rg-n1-keyboard-chat.sh` (UX-002 binding, 12 frames + promo
  record), `rg-a11y-01.sh` (journey, 24 frames), `rg-a11y-02.sh` (focus
  probes), `rg-a11y-04-live.sh` (NEW — the live-chat legs, 18 frames),
  `swap-reprobe` (directed, 6 frames); evidence in `n1-rc14/` (+
  `n1-rc14-attempt1-promo/`), `a01-rc14/`, `a02-rc14/`, `a04-live/`,
  `swap-reprobe/`
- **Adjudication model:** glm-5v-turbo targeted reads + md5 frame-diff
  reasoning (the renderer-determinism law holds at rc.14: the welcome base
  `9897a037`/`24706d22` families reproduce byte-identically across scenes).

## 1. Binding results (the baseline's PENDING items)

| Leg | Baseline state | rc.14 binding evidence | Verdict |
| --- | --- | --- | --- |
| N1 keyboard-only chat creation | IMPOSSIBLE (three no-op chains) | `n1-rc14`: A0=A1=A2 negative control byte-identical (the fix scopes to `begin_new_chat`, not boot focus); B2/C3 composer carries the typed text (VLM: "n1 keyboard chat one/two" with caret); B3/C4/D2 submits dispatch (chats created; the honest auth-wall banner — the documented lab family) | **PASS — N1 CLOSED** (WO-UX-002 @ `41b9ac3`) |
| Palette-close focus landing (017) | typed probe landed NOWHERE | `a02-rc14` P1a: "focusprobe-after-palette-close" in the composer, caret at end (VLM) | **PASS** (WO-P2-017 @ `8dcfcb9`) |
| Overlay-close focus landing (017) | typed probe landed NOWHERE | `a02-rc14` P1b: "focusprobe-after-overlay-close" in the composer (VLM) | **PASS** (WO-P2-017) |
| Find bar positive path | never opened (seeded chat ≠ live task) | `a04-live` B2: Find bar + query "findme" + counter "1 / 3+ results" + THREE yellow-highlighted matches in the live chat body; B3 advance delta; B4 close | **PASS** |
| Terminal PTY (positive) | honest empty state (no task) | `a04-live` C1: the dock opens a REAL shell PTY on a selected live chat (prompt `zc-…:/tmp/d10-seed-ws$`) — but see N5 below | **PASS (opens)** / N5 (focus) |
| Attention positive path | guards only ("No chats need attention.") | `a04-live` F1: toast "Chat marked unread" + unread dot on the selected chat; F3: toast "Cleared unread indicators for 1 chat" (both VLM-verbatim) | **PASS** |
| Full keyboard journey | 24-frame baseline | `a01-rc14`: 24 frames, same structure — palette open/filter/close byte-clean (01c = welcome base), settings/search legs, LEG-6 live chat created via ctrl+n and selected ("a11y attention chat", VLM) | **PASS (journey green)** |
| Modal focus traps (019) | in flight at baseline | `d19-rc14` (see D19 binding below): the Clear-modal trap ladder exact | **PASS** (WO-P2-019 @ `31f83da`→merge) |

## 2. New findings (recorded, non-blocking)

- **N5 — PTY focus does not transfer on Ctrl+backtick open.** The dock opens
  a live PTY (positive, above), but typed input immediately after the open
  does NOT land in it (`a04-live` C1==C2 byte-identical; the echo never
  rendered). The keyboard focus stays on the composer surface. Same
  focus-axis shape as N1/017 — recommend the bounded fix WO (focus the PTY
  pane on open, mirroring the 017 previous-surface/composer/no-focus
  contract) as the first F2 a11y item. Mouse-click engagement works (the
  soak/d-series evidence).
- **N6 — previous/next bracket chords not re-evidenced positively in the
  lab.** `a04-live` E-legs and the directed `swap-reprobe`: from a selected
  live chat and from the project-row (no-selection) boot state,
  Ctrl+Shift+]/[ produced no selection move. The no-selection case is the
  documented guard ("safely do nothing"); the selected-chat case is
  unexplained in this configuration. The alternate **Ctrl+PageDown moved the
  selection positively** (re-probe PGDN: "live chat two body" selected +
  titled, VLM). Prior committed evidence stands: the two-chat isolated
  fixture battery + focused order/no-wrap tests (parity-matrix L112).
  Non-blocking; carry to the F2 a11y binding item alongside N5.
- **N2/N4 re-confirmed** (Settings has no Escape exit; workspace chords are
  Settings-surface no-ops) — unchanged from the baseline, bounded notes.

## 3. Lab lessons (institutionalized)

- **The rc.14 promo modal**: the fresh-profile/pristine boot shows the
  "Introducing GPT-5.6-Sol" modal; the seen-flag does NOT carry across the
  rc.13→rc.14 version bump (the FR-3 baseline's donor-carried skip no longer
  holds). It swallows ALL keyboard input. **Escape is the verified
  dismissal** (the X-icon click at (649,403) missed; the D17-era button
  click hits the "Try GPT-5.6-Sol now" CTA). The defensive
  capture-then-Escape block is now in rg-n1/a01/a02/a04/fr1/fr1d/fr3/d19 —
  no-op when the modal is absent.
- **The a01 seeding template chain** (/tmp/rg-soak-data → a01's inline
  seeder) was wiped with /tmp; re-established from a REAL runtime-written
  rollout harvested from the N1 scene's own live-chat creation (the
  runtime's writer produced session_meta/task_started/turn_context/
  user_message/task_complete — the exact template shape).
- **Background-launched scene shells get reaped** (~40 s after the launching
  tool call returns) — scenes must run foreground or via daemonize.py; the
  aborted a01 attempt-1 frames through LEG-2 were discarded and re-run whole.

## 4. Verdict

**WO-UX-001 binding pass: GREEN at the published rc.14.** Every baseline
PENDING item is adjudicated: N1 closed (fix verified), 017 focus restoration
verified on both close paths, Find/attention positive paths evidenced, the
journey re-run green, 019's trap ladder green. N5 (PTY focus transfer) is the
one new bounded gap — recorded with the fix-shape recommendation, carried to
F2; it does not block F1 (the terminal surface itself is green and the gap
has a mouse-path workaround, same as the pre-fix N1 state was a gap).

— Tech Lead (Task 60, continuation 17), 2026-09-21
