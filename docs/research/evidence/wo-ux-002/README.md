# WO-UX-002 — keyboard-only chat creation (RG-A11Y N1) + the a11y baseline battery

- **Work order:** WO-UX-002 (dispatched from the RG-A11Y baseline findings;
  the keyboard-only journey battery itself was WO-UX-001's evidence-only
  scope)
- **Finding:** the RG-A11Y baseline battery (scenes RG-A11Y-01/02/03/03b at
  main `8a68c9a`) proved with byte-identical frame evidence that a
  keyboard-only user cannot create a chat: `begin_new_chat` (Ctrl+N, the
  palette "New chat" row, the sidebar new-chat row) never focuses the
  composer; there is no global focus-composer chord; the boot-default focus
  is not the composer — typed text goes nowhere (finding N1 in
  `A11Y-BASELINE-VERDICT.md`).
- **Fix:** the `begin_new_chat_with_prompt` pattern applied to
  `begin_new_chat` — `composer.update(cx, |input, cx| { input.focus(window,
  cx); })` immediately AFTER the `Action::BeginNewChat` dispatch (nothing in
  the dispatch path focuses, so the call survives the route change).
- **Merged:** PR #39 squash-merged to main @ `41b9ac3` (worker commit
  `18a1815` on base `c8f6831`; `crates/codex-app/src/ui.rs` only, +102;
  CI green on the full rust-ci matrix; Lead-verified diff).
- **Tests:** `begin_new_chat_focuses_composer_after_dispatch` (the
  focus-wiring pin + the dispatch ordering + the bounded-body guard + the
  reducer semantics: selection cleared, draft emptied, route Tasks) and
  `begin_new_chat_with_prompt_still_focuses_composer` (the regression
  guard) — the WO-P2-011 source-pin house style (GPUI focus is window
  state, unobservable in pure-logic tests; the runtime behavior is the
  Lead's N1 probe at the release binary).

## The evidence set

| File | What it evidences |
| --- | --- |
| `A11Y-BASELINE-VERDICT.md` | The full battery verdict: the settled legs, the four source-unverifiable items, findings N1–N4, the lab laws |
| `rg-a11y-01.sh` | The keyboard-only full journey scene (24 frames, 8 legs) |
| `rg-a11y-02.sh` | The focus-order probes scene (P1a/P1b close-landing, P2 Find, P3 PTY, P4 modal reach) |
| `rg-a11y-03.sh` | The precondition-correct follow-up (F/G legs ironclad: 3 escapes never exit Settings; post-ctrl+n typing goes nowhere — byte-identical to the pristine welcome base) |
| `rg-a11y-03b.sh` | The fresh-data re-run (A0=A1=A2 byte-identical: the N1 proof; C: the empty-state terminal dock; E: the swap no-ops) |
| `a03b-PROGRESS.txt` + `a03-0{0,1,2}-*.png` | The N1 no-op chain: entry = typed = submitted frames byte-identical (36b6dae5) |
| `a01-15/16-*.png` | The guard cascade: after ctrl+n the mark-unread chord hits the honest "Select a chat before marking it unread." guard (no chat was ever selected — the submit never fired) |
| `a03-16/19-*.png` | The ctrl+n focus gap + the post-Settings-excursion gap (F1/G3 = the pristine welcome base 4583c177 byte-identical) |
| `a03b-07-terminal-open.json` | The honest empty-state terminal ("Select a task before opening a terminal." + the same-text toast) |
| `a01-21-overlay-open-r2.json` | The Ctrl+/ keyboard-shortcuts overlay (LEG-8 PASS) |
| `a01-17-jump-clear.json` | The honest "No chats need attention." report |

The full frame sets + VLM reads live in the lab
(`parity-lab/evidence/battery-rg/a11y-0{1,2,3,3b}/` + `vlm/`); this
directory carries the load-bearing subset per the house convention.

## Binding verification

The N1 GUI probe (Ctrl+N → typed text lands in the composer → Ctrl+Enter
creates the chat) re-runs at the rc.14 release binary as part of the F1
binding battery (with the WO-P2-017 focus-restoration merge verifying the
palette/overlay close legs of the same journey).
