# WO-P2-008 Lab Evidence — per-chat unread-attention state + four attention bindings

Delivery under test: `feat/wo-p2-008-unread-attention` @ `c37c21b20`
(binary: guarded release build, `codegen-units=16`, `lto=false`, `-j 1`,
toolchain 1.97.1; built 2026-09-19 00:08 UTC from this branch).

## Scenes

### D11 — full-flow scene (LIMITED: no runtime in lab)

`d11-unread-attention.sh` — the originally staged 7-step scene (create chats
via composer → mark unread → new chat → jump → remark → clear). The scene
could not exercise its chat-creation steps: **the desktop app spawns the
official Codex CLI as its runtime** (`resolve_codex_binary` → PATH `codex` /
`CODEX_RS_CODEX_BIN`), no codex CLI exists in this lab, and the no-fabrication
doctrine forbids mocking a runtime. Without a runtime the sidebar is the
entry surface with zero threads ("No chats"), so dot-on-row and
jump-between-chats are **not GUI-exercisable here** — they are covered by the
codex-core state-machine tests (7) and codex-app binding/jump tests (6, CI).

Captured anyway (d11/): the 7-frame md5 sequence + VLM read of frame 03
showing the honest no-selection status **"Select a chat before marking it
unread."** rendered by the real binary (the reducer's honest-guidance path
working end-to-end through the actual key binding).

### D11b — empty-surface binding scene (COMPLETE)

`d11b-empty-surface.sh` — all four attention bindings exercised on the real
binary at the entry surface; every keypress must resolve visibly (the
WO-P2-007 input-quality doctrine: never a silent no-op).

| Step | Binding | Frame | VLM-read status (verbatim) |
|---|---|---|---|
| 02 | Ctrl+Shift+U (`toggleThreadUnread`) | d11b-02 | "Select a chat before marking it unread." |
| 03 | Ctrl+Alt+A (`nextUnreadChat`) | d11b-03 | "No chats need attention." |
| 04 | Ctrl+Alt+U (`toggleActivityView`) | d11b-04 | "Activity view is not available yet. Use "Next chat needing attention" to jump to unread chats." |
| 05 | Shift+Escape (`clearAllUnread`) | d11b-05 | "No unread chats" |

All five frames differ (md5 in `d11b-md5.txt`); VLM reads in
`d11b/vlm-d11b-0*.json`.

## Verification matrix (five closure gates)

1. **Source changes** — +728/−18 across codex-core (state machine) and
   codex-app (bindings + sidebar dot), reviewed.
2. **Tests pass** — codex-core focused: **7 passed, 0 failed** (local,
   verified with per-test output 2026-09-19 00:1x UTC); codex-app binding
   tests: CI double matrix (authoritative; the local test-mode gpui compile
   peaks >1.5 GB and is excluded by the local OOM discipline with the live
   stack up).
3. **GUI behavior observed** — D11 frame 03 + D11b frames 02–05 (all four
   bindings resolve visibly; verbatim statuses above).
4. **Lab pass** — same scenes, LINUX_GUI_LAB (Xvfb + software GL), binary
   built by the guarded release pipeline.
5. **Parity row** — updated on merge in the parity report + work-order
   ledger.

## Documented residuals (not bugs; deferred)

- Dot-on-thread-row and jump-between-chats have no runtime-enabled GUI
  observation in this lab (same class as the 007 F-A4 no-workspace
  residual): the lab has no Codex CLI runtime and no runtime may be mocked.
  Unit coverage: `next_unread_task_id` + reducer semantics (7 core tests);
  binding ownership/metadata/dispatch (6 app tests, CI).
- Activity view surface: separate future work order (binding gives honest
  guidance meanwhile).
- Unread-state persistence across restarts: reference behavior unverified
  (session state by design here).
