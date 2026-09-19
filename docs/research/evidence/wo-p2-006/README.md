# WO-P2-006 evidence (D9 / D9b) — reading guide (RWO-022 FW-6)

Adversarial re-read (RWO-022, `docs/research/evidence/rwo-020/adversarial.md`
T2) established the following frame-level facts; read this directory with
them in mind:

1. **The side panel's close affordance is its header button**
   (`side-chat-close`, ui.rs:18326-18331 → `close_side_chat`, ui.rs:8644-8645).
   **Escape is NOT a close binding for the side panel.**
2. **`d9-008-side-closed.png` is mislabeled** — fresh VLM re-reads (two
   independent prompts) show the right-docked Side-chat panel **still open**
   and the main area compressed. The D9 script's step 7 ("close the side
   panel (Escape)") was written against a wrong expectation.
3. **The close-path claim is proven by D9b instead**: the labeled re-read of
   the probe frames shows the panel closed with the main view expanded at
   click **(1418,140)** — the *third* candidate; the final frame
   (`d9b-click-1400-160.png`) shows a stray dropdown, not the close.
   The core state test `a_side_chat_opens_and_closes_without_touching_the_selected_chat`
   covers `CloseSideChat` semantics at unit level.
4. **`d9-006-side-submitted.png` and `d9-007-aftermath.png` are
   byte-identical** (md5 `8d733e08fce19256e60ba128c36aaa10` both) —
   "aftermath" adds no information beyond "side-submitted"; the 4-seconds
   gap proves only that the UI was static.
5. Consequently **`d9-010-side-reopened.png` evidences an idempotent re-open
   of an already-open panel** (the panel never closed within D9), not
   close→reopen. The reopen claim rests on the state test (re-open keeps the
   existing side conversation) plus `d9-009-slash-row.png` (the "/side" menu
   row).

The closure *claims* (binding, registry counts, not-selecting semantics,
file confinement) are unaffected — all held under adversarial re-derivation;
only the frame labels in this directory were over-read. Recorded per RWO-022
T2 / FW-6, 2026-09-19.
