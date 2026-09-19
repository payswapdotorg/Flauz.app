# WO-P2-008 Lab Evidence — per-chat unread-attention state + four attention bindings

Delivery under test: `feat/wo-p2-008-unread-attention` @ `c37c21b20`
(binary: guarded release build, `codegen-units=16`, `lto=false`, `-j 1`,
toolchain 1.97.1; built 2026-09-19 00:08 UTC from this branch).
D11r re-run binary: current main `3c812a7` (WO-P2-008..011 all merged;
guarded lab build, exit 0).

## Scenes

### D11 — full-flow scene (original run, 2026-09-19 05:00 UTC — LIMITED, causes now adjudicated)

`d11-unread-attention.sh` — the originally staged 7-step scene (create chats
via composer → mark unread → new chat → jump → remark → clear). The scene
could not exercise its chat-creation steps for **two stacked causes**
(adjudicated by RWO-022 T4 + EQ-1, remediated as FW-5/FW-7 — see D11r):

1. **Environment gap (EQ-1)** — the lab's pinned Codex CLI runtime (lost in a
   post-Task-49 cleanup) was absent, so the desktop app's spawned app-server
   never connected: footer **"Connection failed"**, banner "Couldn't connect
   to the Codex app-server". RWO-022 proved this was real environment drift
   (D9 on 09-18 had "App-server online"), not fabrication.
2. **Script key defect (T4)** — the scene submitted with plain `Return`,
   which is not the composer submit key (D8c/D9 calibration: submits are
   **Ctrl+Return**); chat creation would have failed even with a runtime.

The no-fabrication doctrine forbids mocking a runtime, so the original run
stands as the honest record of the runtime-less lab. Frames relabeled
2026-09-19 (FW-5; git history preserves the original names) — the old
`d11-md5.txt` omitted `d11-02a`; the regenerated manifest covers all eight
frames (values byte-identical to the original run's files):

| Frame (relabeled) | Actually shows (VLM-read verbatim) |
|---|---|
| d11-01-entry | entry surface, "No chats", footer "Connection failed" |
| d11-02a-typed | draft "alpha attention chat" in composer (not submitted) |
| d11-02b-draft-unsubmitted *(was 02b-chatA-created)* | draft still in composer; no chat; "Couldn't connect to the Codex app-server" banner |
| d11-03-no-selection *(was 03-marked-unread)* | status **"Select a chat before marking it unread."** — the reducer's honest-guidance path end-to-end through the real key binding |
| d11-04-no-chat-b *(was 04-chatB-created)* | no chat B row; "No chats"; "Connection lost. Reconnecting…" |
| d11-05-no-unread *(was 05-jumped-to-A)* | status **"No chats need attention."** |
| d11-06-no-selection *(was 06-remarked)* | status **"Select a chat before marking it unread."** |
| d11-07-empty-surface *(was 07-cleared)* | empty tasks surface ("What should we work on?"); nothing to clear |

`d11-unread-attention.sh` in this directory now carries the FW-5 key fix
(both submits → `ctrl+Return`) with an in-file note; the frames above were
produced by the original defective version (pre-fix, git history).

### D11r — corrected full-flow re-run (2026-09-19, COMPLETE — FW-5 + FW-7)

`d11r/d11r-unread-attention.sh` — the corrected scene: runtime restored and
pinned, submits Ctrl+Return, model-NUX modal dismissed (Escape), composer
focused via the D9-calibrated click ladder (x802, y849→789; a single click
at y700 lands on the central first-run sign-in card and typing is eaten).

**Runtime pin (FW-7):** npm `@openai/codex@0.146.0-alpha.3.1-linux-x64`
(`vendor/x86_64-unknown-linux-musl/bin/codex`), verified
`codex-cli 0.146.0-alpha.3.1` — the exact oracle pin from
`docs/platform-support.md` (compatibility oracle: Codex Desktop
26.721.3996.0 + Codex CLI 0.146.0-alpha.3.1). Restored at
`/home/z/parity-lab/runtime/codex`, sha256
`ae77c5e73db36d15c131381c5d620278abed999650867a7490b13384e1f5842d`,
injected via `CODEX_RS_CODEX_BIN` (resolve_codex_binary's explicit
override). **`CODEX_HOME` must point at an existing directory** —
`CodexHome::resolve` (codex-platform `app_server.rs`) requires it and the
CLI hard-exits on a missing home; a negative-control run
(`d11r/negative-control-run2.log`) pins `CODEX_RS_CODEX_BIN` without
`CODEX_HOME` and reproduces the original "Connection failed" state,
proving the requirement.

**Run (Xvfb :103, donor state, main `3c812a7`):** footer
**"App-server online"** from frame 01 onward — EQ-1's drift reversed, the
006-era (D9) runtime-observed conditions reproduced. Composer submits
create threads (the Tasks-route composer creates a **task row under
"Projects"** — titled from the first message; the "Chats" section stays
"No chats"; the attention bindings operate on these thread/task rows, per
the codex-core `next_unread_task_id` semantics). Unauthenticated slice
(D9-identical): thread creation and turn start succeed; turn execution
fails with the "Codex hit an error and is retrying." toast, which does not
block the sidebar or the bindings.

| Step | Probe | md5 | VLM-read verdict (verbatim) |
|---|---|---|---|
| 01 | entry baseline | `e7185a64…` | footer **"App-server online"** (green dot); "No chats"; model-NUX modal "Introducing GPT-5.6-Sol" |
| 01b | NUX dismissed (Escape) | `2f5b4a6d…` | modal gone |
| 02a | type + ladder focus | `468345ed…` | draft **"alpha attention chat"** in composer (sign-in card visible but non-blocking) |
| 02b | Ctrl+Return → create A | `9bc34b22…` | submit accepted (composer cleared); "Untitled task" row appears under Projects; retry toast (unauth turn) |
| 03 | Ctrl+Shift+U on A | `efedb5a6…` | row now titled "alpha attention chat", selected; **unread dot on the row**; status **"Chat marked unread"** |
| 04 | Ctrl+N + create B | `7df5e791…` | "beta background chat" selected; **A keeps its unread dot**; retry toast |
| 05 | Ctrl+Alt+A → jump | `8ca7e38c…` | **selection jumps to A**; **A's unread dot cleared by the visit** (crop-adjudicated: `vlm-crop-0405.json` — dot present on A in 04, absent in 05; the red-X/green-check circles are per-row turn-status icons, a separate indicator class) |
| 06 | Ctrl+Shift+U again | `6ccaaa17…` | **unread dot back on A**; status **"Chat marked unread"** |
| 07 | Shift+Escape | `b2d1809e…` | **no unread dot on any row**; status **"Cleared unread indicators for 1 chat"** (honest count: only A was flagged) |

Frames + `d11r-md5.txt` + VLM reads (`d11r/vlm-d11r-*.json`,
`vlm-crop-0405.json`) + run log (`d11r/run3.log`). The app writes nothing
to stdout (`app.log` empty) — frame + VLM evidence only.

### D11b — empty-surface binding scene (COMPLETE, unchanged)

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
3. **GUI behavior observed** — D11b frames 02–05 (all four bindings resolve
   visibly; verbatim statuses above) + **D11r full flow (2026-09-19
   FW-5/FW-7 re-run): dot-on-row, dot-persists-across-selection,
   jump-to-unread with visit-clear, re-mark, and clear-all-with-honest-count
   all runtime-observed on real thread rows** (was: D11 frame 03 honest
   no-selection path only).
4. **Lab pass** — same scenes, LINUX_GUI_LAB (Xvfb + software GL), binary
   built by the guarded release pipeline; D11r adds the pinned CLI runtime
   (`@openai/codex@0.146.0-alpha.3.1-linux-x64`, sha256 above).
5. **Parity row** — updated on merge in the parity report + work-order
   ledger; residual cell updated again at the D11r closure.

## Documented residuals (not bugs; deferred)

- ~~Dot-on-thread-row and jump-between-chats have no runtime-enabled GUI
  observation in this lab~~ **CLOSED by D11r (2026-09-19): both are now
  runtime-observed** — the pinned CLI runtime is restored for the lab
  (FW-7), so this residual class no longer applies to future scenes either.
- Activity view surface: separate future work order (binding gives honest
  guidance meanwhile).
- Unread-state persistence across restarts: reference behavior unverified
  (session state by design here).
- Unauthenticated slice bounds (honest scope note): turn execution cannot
  be GUI-observed succeeding without credentials; thread lifecycle,
  bindings, and unread-state semantics are observed on the real binary.
