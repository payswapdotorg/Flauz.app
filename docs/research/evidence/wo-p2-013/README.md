# WO-P2-013 evidence — Activity view surface on the existing attention primitives (J-17)

- **Work order:** WO-P2-013 (wave-1 Worker B)
- **Delivery:** `feat/wo-p2-013-activity-view` @ `20a6018` (base `a664644`; the worker iterated five deliveries through the platform's stream-death cycle — r2 `bf5eacf` → prior `690a246` → prior-2 `520eed3` → r3 `1c05511` → final `20a6018` force-pushed onto the canonical branch; superseded attempts self-archived under `archive/wo-p2-013-prior-attempt{,-2}`) + Lead rustfmt pass `6d08bf3`
- **Binary under test:** `parity-lab/binaries/codexrs-013-6d08bf3` (guarded units=16/LTO=false release build, BUILD_EXIT=0; sha256 head `38376e1858c7da3d`)
- **Lab:** LINUX_GUI_LAB — Xvfb :104 + picom, donor state, pinned runtime `@openai/codex@0.146.0-alpha.3.1-linux-x64` via `CODEX_RS_CODEX_BIN` + pre-created `CODEX_HOME` (D11r run-3 pattern; composer submits ctrl+Return; NUX modal Escape-dismissed; composer focus via the D9 click ladder)

## Local battery (Lead integration station, independently re-run)

- fmt: delivery carried fmt drift (rustfmt's `--check` diff emitter OOMs on this 4G box — 5.8GB alloc on the 50K-line ui.rs; the emitter only runs when differences exist) → resolved by the Lead rustfmt pass `6d08bf3` (write-mode diff = the delivery's own code only: closure re-wraps + palette arm one-liner; the WO-P2-011 precedent)
- codex-app `activity_view`: 3P/0F · attention/palette regression: 10P/0F
- codex-core `activity_view`: 3P/0F · attention-family regression: 6P/0F · doctests: ok
- Profile note: the repo's release LTO setting compiles registry deps (gpui) at codegen-units=1 with 1.4GB+ rustc peaks — kernel OOM territory on this box; all local gates ran at units=16/LTO=false (the build_guarded recipe). CI's double matrix runs the authoritative default-profile battery.

## Scene `d16/` (scenes/d16-activity-view.sh, display :104) — the J-17 F1 slice

| Step | Probe | GUI outcome (VLM-read) | Verdict |
| --- | --- | --- | --- |
| 01 | entry baseline | footer "App-server online" | ✅ |
| 02 | create chat A (composer ctrl+Return) | sidebar row "alpha activity chat" | ✅ |
| 03 | Ctrl+Shift+U on A | dot on A + status "Chat marked unread" (vlm-d16-03) | ✅ |
| 04 | Ctrl+N → create chat B | B selected, A keeps its dot | ✅ |
| 05 | Ctrl+Shift+U on B | dots on BOTH rows, "Chat marked unread" (vlm-d16-05) | ✅ |
| 06 | **Ctrl+Alt+U — Activity view OPENS** | centered panel "Activity", count "2 chats need attention", rows "beta activity chat" (selected) + "alpha activity chat" in sidebar order, per-row dots, keyboard footer "Up and Down select · Enter opens the chat · Escape closes" (vlm-d16-06) | **✅ VERIFIED** |
| 07 | Down arrow | selection moved to the second row ("alpha activity chat" highlighted) (vlm-d16-07) | **✅ VERIFIED** |
| 08 | **Enter — jump** | panel GONE, main pane opened "alpha activity chat", its dot CLEARED, "beta activity chat" keeps its dot (vlm-d16-08) — the existing visit-clears semantics resolved the attention through the exact `nextUnreadChat` navigation path | **✅ VERIFIED** |
| 09 | Ctrl+Alt+U — reopen | "1 chat needs attention", single row "beta activity chat", header close button present (vlm-d16-09) | **✅ VERIFIED** |
| 10 | Ctrl+Alt+U — toggle-close | panel gone; frame **byte-identical to step 08** (md5 `584498b9…` both — the post-jump screen) | ✅ |
| 11 | Shift+Escape — clear all | no dots | ✅ |
| 12 | Ctrl+Alt+U — **empty state** | heading "No chats need attention" + guidance "Chats that finish or ask for your approval while you work elsewhere appear here. Use \"Mark chat unread\" to keep a chat on this list." verbatim; no rows (vlm-d16-12) | **✅ VERIFIED** |
| 13 | Escape — close | panel gone | ✅ |

## Adjudication against the acceptance criteria

| # | Criterion | Evidence | Verdict |
| --- | --- | --- | --- |
| 1 | clean commit on the prescribed branch/base | 1 commit @ `20a6018`, merge-base `a664644` | ✅ |
| 2 | battery pass or honest NOT RUN | Lead-re-run battery (above); CI double matrix on PR #36 | ✅ |
| 3 | Ctrl+Alt+U opens/closes a real bounded surface | steps 06/09/10/13 | ✅ |
| 4 | rows reflect `needs_attention_task_ids` | steps 06/09 (count + rows track the flags exactly) | ✅ |
| 5 | Enter/click jumps; attention resolves via EXISTING semantics | step 08 (dot cleared by the visit, other dot preserved) + reducer tests | ✅ |
| 6 | full keyboard-only path | steps 06→07→08 (open/navigate/jump without mouse) + 13 (Escape close) | ✅ |
| 7 | honest empty state | step 12 (plain language, mark-unread guidance, no fake loading) | ✅ |
| 8 | no second store / no semantic changes / no new deps | diff review (non-persisted visible flag only) + regression battery 10P/6P | ✅ |
| 9 | no contract deviations | exactly the two owned files; palette row added to fulfill the WO's stated discovery surface (WO-P2-010 row contract: registry copy verbatim) | ✅ |
| 10 | out-of-scope observations listed honestly | worker stream-died before any report; the WO's own follow-up material (bell chrome icon, sidebar-dot click integration) recorded here by the Lead | ✅ (Lead-carried) |

## Honest residuals / follow-up material (non-blocking)

- **Bell chrome icon** — the WO names it cosmetic follow-up material; the palette row carries the Bell icon (`IconName::Bell`) and the binding is the F1 parity surface. Candidate for the WO-P2-020 UI set.
- **Sidebar-dot click → open Activity view** — the WO's contextual-discovery clause ("IF one-line, otherwise list as follow-up"); the one-line integration did not present itself against the existing sidebar row patterns. Candidate for WO-P2-020.
- **Recently-engaged chats in the surface** — deliberately not listed: no existing session state exposes "recently engaged" without a second store (the handoff's explicit non-goal). The surface lists attention-flagged chats only, matching `nextUnreadChat`'s bounded view.

## Frame/VLM inventory

15 frames + md5 table (`d16-md5.txt`) + 7 VLM reads (`vlm-d16-{03,05,06,07,08,09,12}.json`) + scene script (`d16-activity-view.sh`) + `app.log` + `xvfb.log`/`picom.log`.
