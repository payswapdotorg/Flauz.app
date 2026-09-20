# WO-P2-018 evidence — Lead verification (visible palette entry, Activity bell, label parity)

- **Work order:** WO-P2-018 (visible-entry + label parity for icon-only controls)
- **Delivery under test:** `feat/wo-p2-018-visible-entry-labels` @ `0ece5bb` (base main `8172f6e`, single commit, `crates/codex-app/src/ui.rs` only, +241/−15)
- **Binary:** `parity-lab/binaries/codexrs-018-0ece5bb` (guarded units=16/LTO=false; sha256 head `b019dc6764e4676a`)
- **Lab:** LINUX_GUI_LAB — Xvfb :108 + picom, donor-seeded state (`seed-ws`), pinned runtime 0.146.0-alpha.3.1 via `CODEX_RS_CODEX_BIN`, isolated env per the D16 pattern
- **Coordinate note (lab calibration, house record):** the app window is NOT at (0,0) on this box — geometry (83,41), 1278×818. Entry-button coordinates are derived from `xwininfo` at scene time: search center = WX+WW−206, bell center = WX+WW−174, bar y = WY+17 (verified via geo-probe + VLM).

## Local gates (Lead integration station)

- Build: clean link (guarded profile)
- Focused tests: `chrome_command_palette_entry_reuses_the_registry_copy`, `activity_bell_entry_toggles_the_activity_view_surface`, `unread_attention_dot_exposes_its_tooltip_text`, `archived_chat_single_deletion_exposes_its_accessible_label` — 4P/0F
- Full codex-app suite: **229 passed / 0 failed** (main = 225 + the 4 new)
- fmt: CLEAN (write-mode diff)

## D18 scene (`scenes/d18-visible-entries.sh`, display :108) — RESULT: **PASS**

| Step | Probe | GUI outcome (VLM-read) | Verdict |
| --- | --- | --- | --- |
| 02 | title-bar crop (top-right) | magnifier (Search) + bell icons, then min/max/close — the two new visible entries render in the chrome | ✅ |
| 03 | CLICK the Search entry | **command palette OPENS** — "Search chats or run a command" + suggested commands, dimmed background | ✅ |
| 04 | Escape | palette closes (B04≠B03 toggle) | ✅ |
| 05 | CLICK the Activity bell | **Activity view OPENS** — honest empty state: "No chats need attention" + "Chats that finish or ask for your approval while you work elsewhere appear here. Use \"Mark chat unread\"…" + Ctrl+Alt+U tooltip | ✅ |
| 06 | Escape | closes (B06≠B05 toggle) | ✅ |
| 07 | create chat + Ctrl+Shift+U | sidebar row "alpha visible entry chat" WITH the unread dot | ✅ |
| 08 | hover the dot | **tooltip "Unread activity"** (exact WO copy) | ✅ |
| 09 | Ctrl+Shift+A archive | status "Chat archived" + sidebar "No chats" | ✅ |
| 10 | Settings → search "archived" → Archived chats | nav filters to "Archived / Archived chats"; section opens; the archived row shows **visible "Delete"** (trash icon + text) + **"Unarchive"** labels; header "Delete all" present | ✅ |

Steps 03/05 cross-verified by the geo-probe run (vlm-d18-geo-search/bell) —
both entries open their surfaces from a cold start with no chord used.

## Adjudication against the WO acceptance

| Criterion | Evidence | Verdict |
| --- | --- | --- |
| visible command-palette entry button in the chrome | step 02/03 (renders + opens the same Unified palette as Ctrl+K) | ✅ |
| Activity bell toggling the existing `ToggleActivityView` | step 05/06 (opens/closes the J-17 surface) | ✅ |
| unread-attention dot tooltip in plain language | step 08 ("Unread activity", registry vocabulary — no internal terms) | ✅ |
| archived-chats single deletion visible label | step 10c ("Delete" text + icon, "Unarchive", "Delete all" header) | ✅ |
| no behavior change outside scope | full suite 229/0 + Escape/toggle regression steps | ✅ |

## Honest notes

- The tooltip hover needed a calibrated row position (first attempt caught the
  Workflows nav tooltip instead — recorded in the first-run frame
  d18-08-dot-tooltip.png superseded by the official re-run); the final frame
  shows the dot-row tooltip verbatim.
- The settings "Archived chats" section is below the nav fold with an empty
  query; the scene reaches it deterministically via the settings search box
  ("archived"), which is itself the section's indexed discovery path.
- The 018 branch is based on main 8172f6e which does NOT include WO-P2-017
  (rejected); the archived-delete label evidence is independent of 017.

## Frame/VLM inventory

16 frames + md5 table + 10 VLM reads (vlm-d18-03/05/08/10a/10b/10c/geo-search/
geo-bell/donor-verify) + scene script + app/xvfb/picom logs + geo-probe frames.
