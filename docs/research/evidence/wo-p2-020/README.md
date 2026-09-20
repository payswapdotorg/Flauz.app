# WO-P2-020 evidence — Lead verification (honest status for Ctrl+P without a workspace)

- **Work order:** WO-P2-020 (F-A4 — honest feedback for Ctrl+P without a workspace)
- **Delivery under test:** `feat/wo-p2-020-files-palette-status-r2` @ `a69e63b` (base main `8172f6e`, single commit, `crates/codex-app/src/ui.rs` only, +87/−12 vs main; r1 `8155f7c` superseded)
- **Binary:** `parity-lab/binaries/codexrs-020r2-a69e63b` (guarded units=16/LTO=false; sha256 head `9a0b1513ceb0e5d5`)
- **Lab:** LINUX_GUI_LAB — Xvfb :110 + picom, **FRESH EMPTY state** (deliberately NO donor: the F-A4 precondition is the no-workspace state), pinned runtime 0.146.0-alpha.3.1 via `CODEX_RS_CODEX_BIN`, isolated env

## Local gates (Lead integration station)

- Build: clean link (guarded profile)
- Focused test: `files_palette_command_reports_when_no_workspace_is_open` — 1P/0F
- Full codex-app suite: **226 passed / 0 failed** (main = 225 + the 1 new)
- fmt: CLEAN (write-mode diff)

## D20 scene (`scenes/d20-files-palette-status.sh`, display :110) — RESULT: **PASS**

| Step | Probe | GUI outcome (VLM-read) | Verdict |
| --- | --- | --- | --- |
| 01 | entry baseline (no workspace) | app renders fresh state — sidebar "No projects" | ✅ |
| 02 | **Ctrl+P with NO workspace** | honest status, verbatim: **"Select a workspace before searching files."** — NO silent no-op, no palette opened | ✅ VERIFIED |
| 03 | settle (3s) | status persists (B03 == B02 md5 — stable, not a flicker) | ✅ |
| 04 | Ctrl+K regression guard | the Unified palette still opens everywhere — "Search chats or run a command" | ✅ |

## Adjudication against the WO acceptance

| Criterion | Evidence | Verdict |
| --- | --- | --- |
| F-A4 closed: Ctrl+P without workspace emits bounded honest status via the house SetStatus path | step 02 (exact copy, the WO-P2-008/012 status family) | ✅ |
| not a silent no-op | frame delta + VLM read (status text visible; nothing swallowed) | ✅ |
| with a workspace open the guard stays clear (Files palette unchanged) | unit test + the D13/WO-P2-010 evidence on the Files palette (unchanged code path — the guard only fires when `state_local_workspace_cwd` is None) | ✅ (unit-carried) |
| no other behavior change | full suite 226/0; Ctrl+K regression step 04 | ✅ |

## Honest notes

- The first D20 run died to a LAB infrastructure failure (disk-full at 100% —
  the box filled during earlier evidence copying; the scene's X server and
  captures failed). The scene re-ran clean after freeing space; no app-behavior
  involvement.
- The status-line persistence (step 03) is captured at settle; the house
  SetStatus expiry behavior is unchanged from WO-P2-008/012 (same dispatch
  path).

## Frame/VLM inventory

4 frames (d20-01..04) + 2 VLM reads (vlm-d20-02, vlm-d20-04) + scene script +
app/xvfb/picom logs + md5 table in the scene output.
