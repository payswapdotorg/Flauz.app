# RG-A11Y baseline verdict — WO-UX-001 scene battery at main `8a68c9a`

- **Work order:** WO-UX-001 (keyboard-only journey battery + focus-order probes)
- **Tree:** binary `codexrs-main-8a68c9a` (main `c8f6831` code; the F1 RC tree)
- **Date:** 2026-09-20 (runs 19:55–20:46 UTC)
- **Scenes:** `rg-a11y-01.sh` (24 frames), `rg-a11y-02.sh` (10 frames),
  `rg-a11y-03.sh` (19 frames), `rg-a11y-03b.sh` (16 frames) — all in
  `parity-lab/scenes/`; evidence in `evidence/battery-rg/a11y-01|02|03|03b/`;
  VLM reads in `evidence/battery-rg/vlm/` (a01-* 12 + 4 follow-ups, a02-* 5 +
  3 follow-ups, a03-* 2, a03b-* 1).
- **Adjudication model:** glm-5v-turbo targeted reads + md5 frame-diff
  reasoning + source cross-verification (`crates/codex-app/src/ui.rs`,
  `crates/codex-core/src/lib.rs` at the same tree).

## 1. Settled legs (PASS — keyboard-only works)

| Leg | Evidence | Verdict |
| --- | --- | --- |
| Palette open (Ctrl+K) | a01-01 read: overlay + placeholder + Suggested/Settings sections | PASS |
| Palette filter | a01-02 read: typed "settings" filters to settings entries | PASS |
| Palette close (Escape) | a01-03 md5 = entry base (byte-identical return) | PASS |
| Settings open (Ctrl+,) | a01-04 read: General page + search placeholder | PASS |
| Settings search (Ctrl+F) | a01-05 read: "appearance" filters sidebar to Appearance | PASS |
| Keyboard-shortcuts overlay (Ctrl+/) | a01-21-r2 + a02-03 reads: modal "Keyboard shortcuts" with Chat/Navigation entries; Escape ladder closes (a01-22/23 md5 = base) | PASS |
| Palette/overlay open+close cycle | a02 P1: open/close pair byte-stable | PASS |
| Attention chords fire globally with honest reports | a01-16 "Select a chat before marking it unread."; a01-17 "No chats need attention."; a01-18 "No unread chats" (all unit-covered at ui.rs:53150–53230) | PASS (honest guards) |
| Terminal dock opens with honest empty state (no task) | a03b-07 read: dock + "Select a task before opening a terminal." + same-text toast | PASS (honest empty state) |
| ctrl+n exits Settings to a new chat | a01-15 (Settings→main), a03 G3 (byte-identical welcome base) | PASS (the evidenced keyboard exit path) |

## 2. Source-unverifiable settlement (the four inventory items)

1. **Palette/overlay close focus landing — SETTLED (pre-fix state documented).**
   a02 P1a/P1b: typed probes after close land NOWHERE (composer shows
   placeholder; frames byte-identical to the pristine welcome base across
   runs). The landing is unmanaged at this tree — exactly the pre-017-fix
   state. WO-P2-017(-fix) owns the repair (previous surface → composer
   default → no-focus fallback); the D17 scene re-run at the merged SHA is
   the binding verification.
2. **Terminal open → PTY focus transfer — NARROWED, positive path pending.**
   Without a live task the dock opens in its honest empty state and typed
   input has no PTY to land in (a02-08, a03b C1=C2). The positive transfer
   (type immediately lands in the PTY) requires a live task; it is a
   binding-pass item at the final RC (with the composer-focus path repaired
   or a calibrated click, then a live chat + terminal echo probe). The
   soak's TERM01 frame was byte-identical to its own baseline (no dock
   change) — it adds no positive evidence either.
3. **Find active-occurrence non-color distinction — NARROWED, positive path
   pending.** `FindInThread` (Ctrl+F, "Search the current chat") is
   task-workspace-guarded (ui.rs `requires_task_workspace`); the seeded
   synthetic chats are not live tasks, so the Find bar never opened (a02-05
   read: no Find bar; a03b B1–B3 md5-stable). Binding-pass item with a live
   chat (query → Enter advance → capture pair).
4. **The four confirmation modals' Tab traps — rides on WO-P2-019** (in
   flight). The Settings-reachable destructive modal family is already
   evidenced by the D12 phase-C run (Clear browsing history: modal + Cancel/
   Clear + click-confirm + honest empty state after). The a02 P4 frames
   document the settings-search reach ("reset" → "No results found" — the
   reset actions live inside sections, not as search results).

## 3. New findings (not in the f1-sweep inventory)

- **N1 — No keyboard-only chat creation at this tree (gap, F1-a11y-relevant).**
  Three byte-identical no-op chains: (a) boot-default focus is not the
  composer (a03b A0=A1=A2 — typed text + Ctrl+Return produce zero pixel
  change); (b) `begin_new_chat` (Ctrl+N) clears the composer and dispatches
  BeginNewChat but never focuses the composer (ui.rs:8826 — only
  `begin_new_chat_with_prompt` focuses, ui.rs:9064), so typing after Ctrl+N
  goes nowhere (a01 LEG-6 → the mark-unread guard cascade; a03 F1 = the
  pristine welcome base byte-identical; a03 G3 same); (c) there is no global
  focus-composer chord (`composer.submit` is CmdOrCtrl+Alt+Shift+O; plain
  typing needs the input focused). Consequence: a keyboard-only user cannot
  create a chat; every downstream guarded surface (terminal PTY, Find,
  browser panel, swap) is unreachable for them. The composer submits on
  Ctrl+Enter (`InputEvent::PressEnter { secondary: true }` → `submit`,
  ui.rs:6246) — the submit chord itself is fine; the gap is focus.
  Classification: this is the keyboard-journey analogue of the 017 focus
  axis — recommend a bounded fix WO (focus the composer in
  `begin_new_chat`, mirroring `begin_new_chat_with_prompt`) in the F1
  end-game or as the first F2 a11y item; recorded here as the baseline
  truth.
- **N2 — Escape does not exit the Settings page (bounded note).** a01-06
  (two escapes: still Settings, "appearance" text intact); a03 G1=G2
  byte-identical after THREE escapes; the exit paths are the "Back to app"
  link (pointer) and Ctrl+N (keyboard, evidenced). Not a silent no-op defect
  (a keyboard exit exists); recorded as a UX note for the settings surface.
- **N3 — Empty-state terminal dock does not close on the second Ctrl+`**
  (minor gap). a03b C1=C2=C3 byte-identical with the dock open (the echo
  attempt and the second toggle produced no change). The toggle-close works
  in the dock-open state evidenced elsewhere (a01 LEG-3 03c on Settings is
  not comparable; the soak's live-terminal legs own the positive close).
  Recorded as observed; binding-pass re-check at the final RC.
- **N4 — Workspace chords are silent no-ops on the Settings surface**
  (bounded note). Ctrl+` / Ctrl+Shift+B / Ctrl+F do nothing on Settings
  (a01 LEG-3/4: typed text landed in the settings search — "cho
  a11y-terminal-focus" with "No results found" — after the chords failed to
  act). The Settings page is a separate surface from the workspace; the
  chords' scope is the workspace (the shortcuts overlay lists them
  globally, which is the cosmetic mismatch). Consistent with the merged
  WO-P2-012 honest-status family applying to the workspace surfaces.

## 4. Scene-design findings (the lab's own lessons — not product findings)

- **L-a03-route: the app persists its route in the shared sqlite** — a03
  booted INTO Settings (a02's final route) via the shared
  `/tmp/rg-a11y-data`, invalidating its A–E legs (A1=A2 byte-identical to
  a01's settings frame). a03b re-ran with a fresh DATA dir. LAW: every
  scene run gets a fresh `CODEX_RS_DATA_DIR` unless route carry-over is the
  thing under test.
- **L-a03-md5: the renderer is deterministic across boots** — the welcome
  base frame is byte-identical across a01/a03/a03b runs (4583c177/9897a037
  families), which turns md5 equality into a sound no-op proof for this
  app.
- **L-a03-seed: seeded sessions are not live tasks** — they enumerate as
  sidebar rows (project-nested) but are not in `state.tasks`; swap/mark/
  Find/PTY guards all see "no selection / no task". Positive-path probes
  need a real live chat.

## 5. Binding-pass plan (at the final RC, post 017/019 merges)

1. Re-run rg-a11y-01 (the full journey) — the palette/overlay close legs
   should then show focus restored to the composer (017-fix contract).
2. The N1 probe (Ctrl+N → type → submit) — either the fix WO landed (text
   lands, chat created) or the gap is re-recorded verbatim.
3. Live-chat legs: Find bar (query + Enter advance + capture pair), PTY
   focus transfer (type immediately after Ctrl+`), swap next/prev with two
   live chats, attention positive path ("Chat marked unread" → "No other
   chats need attention." → "Cleared unread indicators for 1 chat").
4. 019's modal-trap D-scene at the merged tree (its own WO evidence).

## 6. Verdict

**WO-UX-001 baseline: GREEN as a baseline record** — the scene battery ran,
every leg adjudicated, the four source-unverifiable items settled or
narrowed with explicit owners (017-fix / 019 / binding pass), and the new
findings N1–N4 recorded with source anchors. The baseline is NOT a
clean-pass claim: N1 is a real keyboard-only gap that the binding pass will
re-adjudicate; F1 closure must carry N1's disposition (fix WO or explicit
defer with the user-facing note).

— Tech Lead (Task 60, continuation 7), 2026-09-20
