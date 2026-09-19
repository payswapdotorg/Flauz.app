# WO-P2-011 evidence — context-scoped browser chords (D14/D14b)

Binary: `codexrs-main-3c812a7` (guarded lab build at the PR #31 merge head;
build log `parity-lab/logs/build-3c812a7.log`, exit 0). Scenes:
`d14-browser-chords.sh` + `d14b-copywd-row.sh` (Xvfb :107/:108, donor state +
seeded fixture repo, xdotool keys, ffmpeg x11grab frames, VLM adjudication).

## Provenance (stream-death ≠ work-death)

Worker session (GLM-5.3 + Full-Stack, agents tab): first chat `f1426555`
capacity-stuck 900 s → queue_watch staleness assault #1 → chat `8963e5f3`,
generated actively 13:14–13:42 UTC, then **stream death at 13:42:38**
(server probe: `updated` frozen, msgs=2, last assistant len=6 husk — the
in-chat completion report never landed). The deliverable was complete in
git: `feat/wo-p2-011-browser-bindings` @ `a0cfa5c` (exact base `5516084`,
single bounded commit) — harvested from git per the RWO-022 precedent; Lead
rustfmt pass `62683bf`; PR #31 → **merged `3c812a7`**, CI green both
matrices (windows-latest + ubuntu-24.04, full workspace battery incl. the 3
delivery tests). Local battery at the integration station: NOT RUN —
disk-exhausted 4G sandbox (100% full mid-build; recorded honestly, CI is
the authoritative gate and ran the identical gates).

## D14 — entry/workspace fall-through + no-global-hijack (all PASS)

| # | probe | md5 evidence | verdict |
|---|-------|--------------|---------|
| 02 | Ctrl+R on entry | `337062f2…` pre == post **byte-identical** | PASS — no hijack |
| 03 | Ctrl+Shift+R on entry | `337062f2…` == 02b **byte-identical** | PASS — no hijack |
| 04 | Ctrl+P palette | surface changed (`0b441df1…`) — file-search palette opens ("Search files / Files / Type to search for files", VLM-read) | PASS — other chords untouched |
| 05 | palette query "copy working" | file-search query resolves (`d0f44649…`, VLM-read "Searching files…") | PASS — searchFiles surface intact |
| 06 | create chat via composer | workspace surface (`8c55c15f…`, VLM-read: composer titled "d14 chord fixture chat") | PASS |
| 07 | Ctrl+Shift+C in workspace | no toast; bottom-strip crop VLM-read shows only the lab's persistent "Connection lost. Reconnecting… Dismiss" banner + "Retry 4 in 8s" counter (the 06→07 delta is the counter tick) | PASS (honest-negative) — the chord falls through silently: `copyWorkingDirectory` is guard-disabled without a chat cwd, exactly the pre-011 registry behavior |
| 08 | Ctrl+R in workspace | `e4196b79…` pre == post **byte-identical** (retry counter quiesced by then) | PASS — no reload hijack without browser focus |
| 09 | Ctrl+Shift+R in workspace | `e4196b79…` == 08b **byte-identical** | PASS |

## D14b — command-palette registry evidence (honest-absence chain)

| # | probe | md5 evidence | verdict |
|---|-------|--------------|---------|
| b1 | create chat via composer | draft stayed in the composer (Return did not submit this run); surface = empty workspace "What should we work on?" (VLM-read b2/b5) | recorded honestly — the D14-06 workspace creation is the working probe |
| b2 | Ctrl+K palette, query "copy working directory", 3 s settle | VLM-read: query echo + "Loading chats…" chat-group placeholder, **no Commands row** — the guarded `copyWorkingDirectory` row is honestly absent without a selected chat | PASS — matches the D13 record exactly (`evidence/wo-p2-010/` README: "copy working directory → Honest absence — guard live (WorkingDirectory)") |
| b3→b4 | pre-chord vs Ctrl+Shift+C @1.2 s | `692c7dd5…` == `692c7dd5…` **byte-identical** | PASS — silent fall-through, no surface change |
| b5 | @2.5 s | `edb4feb1…` differs (retry-counter/"No chats" state tick; bottom bar VLM-read unchanged banner) | consistent — no toast; the enabled-path render is runtime-gated |

## NOT RUN (runtime-gated, unit-covered — the D12/D13 doctrine)

- **Browser-pane-focused chord resolution** (Ctrl+R/Ctrl+Shift+R reload,
  Ctrl+Shift+C copy-URL with the "Browser" key context, live
  `Effect::BrowserReload`, "Copied Browser address"): needs a live browser
  surface context; the runtime-less lab does not expose the browser pane.
  Covered by `browser_pane_chords_resolve_only_inside_browser_focus`,
  `browser_pane_copy_url_copies_page_urls_and_guides_otherwise`,
  `browser_reload_chords_report_honestly_without_a_loaded_page` (CI green,
  PR #31 both matrices).
- **The enabled-path "Copied working directory" toast**: needs a
  cwd-bearing selected chat (runtime); the entry-surface composer chat has
  no cwd, so the command is guard-disabled and the chord falls through
  silently (D14-07/D14b-b4 byte-stable evidence). The
  exactly-one-owner registry assertion (Ctrl+Shift+C →
  `copyWorkingDirectory`) is unit-proven on CI; the row's palette render is
  runtime-gated (D13's chat-scoped-rows record).
- **Clipboard write itself**: headless tests exercise the decision path
  (`browser_url_copy_value`); the clipboard is a platform effect.

## Residual (documented, never faked)

The no-cache force-reload variant needs a platform-side CDP
`Page.reload {ignoreCache: true}` command (outside this WO's file
boundary). Both reload chords route to the live reload effect today; the
seam comment in `crates/codex-core/src/lib.rs` documents the plug-in point
(the distinct `Action::ForceReloadBrowser` id).
