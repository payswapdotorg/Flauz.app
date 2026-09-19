# WO-P2-010 Lab Evidence — palette rows for the evidenced registry commands

Delivery under test: `feat/wo-p2-010-palette-rows` @ `0eeef7d9` (variant 1 of
three over-delivered variants — adjudicated canonical: 3 tests incl.
palette↔registry title/description parity per evidenced id + availability
guards; +536/−4 additive; variants `-b` @ `4c66e73` and `-c` @ `0417faf`
recorded redundant). Lead harvest: fmt fixes → `767b824` → PR #25 from
`feat/wo-p2-010-verified` → merged `876bbe8` (2026-09-19, CI green both
matrices).

Binary under test (D13): guarded release build (`codegen-units=16`,
`lto=false`, `-j 1`, toolchain 1.97.1) from post-merge main `876bbe8`
(2026-09-19 07:05 UTC).

## D13 — palette-rows scene (entry surface, offline-honest scope)

`scenes/d13-palette-rows.sh` — Ctrl+K palette at the runtime-less entry
surface (no chat selected; D11b doctrine: the desktop app needs the official
codex CLI runtime, absent in this lab; no fabrication). Ten queries typed at
the real binary; a row that is state-guarded absent is POSITIVE evidence
(the no-visible-dead-rows rule — guarded commands must not render dead).

| Query | Expected row | Observed (VLM-read) |
|---|---|---|
| *(baseline, no query)* | Suggested + Settings groups | Suggested: "New chat" Ctrl+N, "Open folder" Ctrl+O, "Search files" Ctrl+P; Settings: General, Appearance, Keyboard shortcuts, Usage & billing, Computer use, Profile, Import, Browser — all rendered (vlm-02-palette.json) |
| `toggle review` | "Toggle review" (guarded: task workspace + git repo) | **Honest absence** — guard live (`toggleReviewTab` requires task_workspace_active + repository_root, ui.rs `keyboard_shortcut_command_enabled`) |
| `fork` | "Continue in new chat" (guarded: selected chat) | **Honest absence** — guard live (`forkThread` requires selected_task_id) |
| `copy deeplink` | "Copy deeplink" (guarded: selected chat) | **Honest absence** — guard live (selected_task_copy_value Deeplink) |
| `copy session id` | "Copy session id" (guarded: selected chat) | **Honest absence** — guard live (SessionId) |
| `copy working directory` | "Copy working directory" (guarded: selected chat) | **Honest absence** — guard live (WorkingDirectory) |
| `approve request` | guarded: pending approval | **Honest absence** — guard live (approval_shortcuts_available) |
| `decline request` | guarded: pending approval | **Honest absence** — guard live (approval_shortcut_available) |
| `rename chat` | guarded: selected chat | **Honest absence** — guard live (selected task) |
| `go to chat 1` | "Go to chat 1" (guarded: occupied slot) | **PRESENT, verbatim**: "Go to chat 1 — Open the visible chat in this shortcut slot — Ctrl+1" (vlm-batch2.json frame 5) |
| `maximize side panel` | guarded: maximizable panel | **Honest absence** — guard live (side_panel_maximize_available) |

The chat-search group shows an honest "Loading chats…" placeholder under
every query (offline; the group is backend-driven) — rows render above it
when their guards pass (frame 5 proves the ordering).

## Residuals (documented, not fabricated)

- Present-rendering of the eight chat-scoped rows requires a selected chat =
  the official codex CLI runtime — **not GUI-exercisable in this lab**
  (same class as WO-P2-008's D11 limitation). Covered by the delivery's
  unit tests (palette↔registry title/description parity per evidenced id;
  availability guards) verified in the PR #25 CI matrix (both matrices
  green).

Frames + md5 (`d13/d13-md5.txt`) + VLM reads (`d13/vlm-*.json`) captured
2026-09-19 07:27–07:31 UTC on display :106 from the real binary.
