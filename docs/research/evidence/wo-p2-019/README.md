# D19 — WO-P2-019 pre-merge GUI regression scene (LINUX_GUI_LAB)

**Date:** 2026-09-21 05:20–05:55 UTC
**Binary:** `codexrs-wo019-31f83da` (guarded release build: units=16, LTO=false,
strip=symbols — the 013 recipe; built locally at the merge-candidate SHA
31f83da = 2574049 + the rustfmt follow-up)
**Runtime:** pinned Codex runtime 0.146.0-alpha.3.1; Xvfb :110 + picom;
1440x900x24; donor state.sqlite3 + 5 seeded browsing_history rows + d19
fixture workspace (the d12 recipe).

## Scope (per the established plan)

The four WO-P2-019 modals (remote pairing / remote confirmation / account
logout / plugin install) are **flow-gated** in the lab (no backend auth /
pairing session / marketplace — the a02 P4 note, A11Y-BASELINE-VERDICT §5.4).
Their runtime trap evidence rides the **RC binding pass** at the merged tree.
This pre-merge scene verifies at the merge-candidate SHA:

- **LEG A — trap-family regression**: the evidenced destructive-modal family
  (Clear browsing history — the d12 phase-C flow) still traps Tab inside the
  modal at 31f83da. 019's additions (8 actions, 8 context-scoped bindings,
  4 handlers, 6 panel wirings) must not break the pre-existing six.
- **LEG B — background Tab**: outside any modal, Tab keeps its background
  meaning (live, bidirectional; not captured/swallowed by the new bindings).

## Frame ladder (md5)

| Frame | Action | md5 | Reading |
|---|---|---|---|
| 01 | entry baseline | a278df63… | clean boot (main surface; promo flag persisted) |
| 02 | palette → "browser settings" | 7b9eb2b1… | Settings Browser section |
| 03 | settled (history rows visible) | 7b9eb2b1… | 5 seeded rows + enabled Clear |
| 04 | Clear clicked → modal open | 29d929d2… | "Clear browsing history?" — Clear button focused (solid red fill) |
| 05 | **Tab #1** | 5252c1c6… | focus → **Cancel** (light-blue ring on Cancel) |
| 06 | **Tab #2** | 3f42bd18… | focus → **Clear** (blue ring) — **WRAP, no escape** |
| 07 | Tab #3 | 5252c1c6… | = F05: Cancel again (alternation continues) |
| 08 | **Shift+Tab** | 3f42bd18… | = F06: Clear — **backward wrap** |
| 09 | Escape | 504b16fe… | modal closed; Settings Browser intact |
| 10 | "Back to app" click | bd7da099… | main surface ("What should we work on?") |
| 11 | composer ladder | 61960c38… | composer caret visible |
| 12 | background Tab #1 | e22c5fee… | frame delta (background advance) |
| 13 | background Tab #2 | a3779023… | further delta |
| 14 | background Shift+Tab | e22c5fee… | **= F12 — bidirectional revert** |

## LEG A verdict: PASS

- The md5 alternation F05==F07 (Cancel) and F06==F08 (Clear) is the structural
  signature of the trap: each Tab advances within the modal's two-button
  focusable set and **wraps**; Shift+Tab wraps backward. A broken trap would
  drop focus to the background on Tab #2 (a third distinct state or a
  no-focus state), not return to the exact F06 pixels.
- Zoomed VLM reads (crop 3x, glm-5v-turbo): F04 Clear focused (solid red
  active fill) → F05 Cancel focused (light-blue ring) → F06 Clear focused
  (blue ring) → F08 Clear focused (blue ring). Modal title/body verbatim:
  "Clear browsing history?" / "This removes all stored browsing history from
  this device" — the d12 phase-C evidence, reproduced.
- Escape closes; the underlying Settings page is intact.

## LEG B verdict: PASS (with an honest note)

- Tab on the main surface at 31f83da produces frame deltas (F11→F12→F13) and
  Shift+Tab reverts to the exact F12 state (F14==F12) — Tab remains a live,
  bidirectional background operation. The failure mode 019 could have
  introduced (a globally-matching Tab binding swallowing background Tab) is
  absent.
- Honest note: full-frame VLM reads place the caret in the composer across
  F11–F13; the per-frame md5 deltas are consistent with either focus
  advancement through background elements or caret-blink phases. The
  structural no-leak claim does not rest on this leg alone: the four new
  KeyBindings are context-scoped by construction (`Some("RemotePairingModal")`
  etc.), pinned by the four source-pinned tests, and compiled+tested green in
  the CI matrix (both OSes) at this SHA.

## Supporting evidence already on file

- CI run 35563215777 on 31f83da: windows-latest + ubuntu-24.04 both success
  (Format, Clippy, full workspace Test — 139 ui.rs tests incl. the four new
  modal-trap tests, Release build, Linux startup smoke).
- The four flow-gated modals: runtime probes scheduled at the RC binding pass
  (rg-a11y-02 P4 note + A11Y-BASELINE-VERDICT §5.4).

## Scene scripts

- `d19-modal-traps-regression.sh (superseded mid-run by the pair below)` (bootstrap/nav; superseded mid-run)
- `../../scenes/d19-phaseC.sh` + `../../scenes/d19-probes.sh` (the parked-app
  driving pair)
- `d19-sweep.sh` (the Clear-button locator — brightness-delta
  modal detection; button found at (1080,575))

## Operational notes (recorded for the machinery backlog)

- The fresh-profile boot shows a "What's New" promo modal that swallows the
  keyboard until dismissed (click its primary button at ~(580,619)); its
  dismissal flag persists in the data dir. Scene scripts must dismiss it
  before palette navigation.
- Full-frame VLM coordinate reads are unreliable for small controls (three
  contradicting reads placed the Clear button at (760,650)/(763,651) — both
  hallucinated; the region actually contains only the rendered mouse cursor).
  The brightness-delta click-sweep (d19-sweep.sh) located the true button at
  (1080,575) in one pass. Zoomed-crop reads are reliable; full-frame reads
  are not.
- xdotool/xwininfo need the complete desktop-tools PATH **and**
  LD_LIBRARY_PATH in every driving shell; a truncated export fails silently
  (libxdo.so.3 not found) and looks exactly like a dead UI.
- Re-assert `xdotool windowfocus` before every driving burst; focus goes
  stale between script phases.

— Tech Lead (Task 60, continuation 15), 2026-09-21
