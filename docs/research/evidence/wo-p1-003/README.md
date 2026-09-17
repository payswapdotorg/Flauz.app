# WO-P1-003 evidence — multi-folder local projects (LINUX_GUI_LAB)

Captured 2026-09-17 in the LINUX_GUI_LAB (local Debian 13 sandbox; Xvfb
:103 + picom, 1600x1000, isolated HOME/XDG/CODEX_HOME/CODEX_RS_DATA_DIR,
no credentials, lavapipe ICD — the sealed `session.sh` recipe).

Product binary: `codexrs` built from branch
`parity/wo-p1-003-multi-folder` (commit `1d2abca`, dev profile,
`debug=0`, `-Wl,--strip-debug`, the documented `-lXau -lXdmcp` lab link
args), version `0.1.0-rc.13`.

## Scene setup

`seed-wo-p1-003.py` pre-populates the v4 state DB
(`state.sqlite3`, `user_version = 4`, `workspace_folders` table) with
project **Alpha**: primary
`…/linux-B-003/projects/alpha`, related
`…/linux-B-003/projects/beta` (raw-UTF-8 BLOB paths, Unix encoding).
Fixture folders exist on disk (`alpha/PRIMARY-README.md`,
`alpha/src/main.rs`, `beta/BETA-NOTES.md`).

Pre-seeding is a lab-only device to keep the scene deterministic: the
Add-folder picker is a native OS dialog that does not render under bare
Xvfb. The add/remove reducer paths are covered by the unit tests in the
implementation commit; the GUI scene evidences the surface, the swap,
the discovery contract, and persistence.

## Captures (VLM-read: glm-5v-turbo; reads reproduced in vlm-reads.txt)

| # | Capture | Evidences |
|---|---------|-----------|
| 01 | `01-sidebar-multi-folder-project.png` | Seeded multi-folder project listed (and auto-selected) in the sidebar Projects section; the new-chat path indicator shows the PRIMARY folder (`Local …/projects/alpha`). |
| 02 | `02-file-search-hits-related-folder.png` | Palette file search (Ctrl+K → "search files") for `BETA` returns `BETA-NOTES.md` from the RELATED folder — related folders join file search (WO contract). |
| 03 | `03-edit-project-surface.png` | The Edit project surface: title "Edit Alpha", contract subtitle, primary row with the Primary badge, related row with Make-primary (star) and Remove affordances, Add folder, Done. |
| 04 | `04-primary-swapped-surface-follows.png` | After Make-primary on the related row: the surface follows the new primary — `…/beta` now carries the Primary badge, the old primary `…/alpha` is parked as a related folder, identity ("Alpha") preserved. |
| 05 | `05-path-follows-new-primary.png` | Surface closed via Done; the new-chat path indicator now reads `Local …/projects/beta` — the primary swap re-keyed the active cwd. |
| 06 | `06-swapped-state-persists.png` | Reopening the surface shows the swapped state unchanged — the v4 storage round-trip persists across close/reopen. |

## Calibration notes (scene iterations)

- `scene-d6-v5.sh` is the final scene; earlier calibration iterations
  (v1–v4) established: the project-row affordances sit at the row's
  right edge (plus / gear / ellipsis at x ≈ 354/382/410, y ≈ 380 on the
  1600x1000 frame — pixel-verified, VLM full-frame coordinate estimates
  were off by ~110 px), the modal's row actions at x ≈ 1015/1043, and
  the palette as the real file-search entry.
- **Runtime finding (new, surfaced by this scene):** the `Ctrl+P`
  keybinding is bound to the `OpenFileSearch` action, but no handler is
  registered anywhere in the workspace — the binding is a silent no-op
  (same pattern WO-P1-001/002 closed for Terminal/Browser). The palette
  command "Search files" (Ctrl+K → "Search files") is the working entry
  and is what capture 02 uses. Recorded as a P3-row candidate in the
  parity report; not self-scoped into this work order.

## Artifacts

- `scene-d6-v5.sh` — final scene script (uses the sealed
  `scene_helpers.sh` CAPTURE/XDO API).
- `seed-wo-p1-003.py` — DB seeding tool.
- `vlm-reads.txt` — raw VLM transcripts for captures 01–06.
