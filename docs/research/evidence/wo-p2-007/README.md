# WO-P2-007 evidence — Ctrl+P silent no-op fix (LINUX_GUI_LAB)

## D10 — baseline defect proof (unfixed main `3c9f113`)

Scene: `d10-ctrl-p-file-search.sh` (Xvfb :101 + picom xrender + software GL —
the calibrated lab recipe). Entry surface, no workspace:

| Capture | Bytes | md5 |
| --- | --- | --- |
| d10-01-entry-surface.png | 54058 | `bb5139b3…` |
| d10-02-after-ctrl-p.png | 54058 | **identical** — the silent no-op |
| d10-03-after-escape.png | 54058 | identical (nothing to close) |
| d10-04-palette-search-files.png | 51664 | differs — the working palette route (Ctrl+K → "search files") |

The byte-identical 01/02 pair is the defect proof: the bound `OpenFileSearch`
action dispatched into nothing.

## D10b — fix proof (branch `parity/wo-p2-007-ctrl-p-file-search` @ 5574c95; merged via PR #20 → `a3c0e01`)

The sweep finding F-A4 (input-surface-sweep §F): the Files palette
early-returns when no local workspace is open — so the entry-surface probe is
byte-identical BY DESIGN even post-fix. D10b seeds the workspace-state path
instead: `CODEX_RS_DATA_DIR` + a `recent_workspaces` row (the
`StorageOpened → state.new_chat_cwd` restore path), a fixture repo with
files, then the same probe:

| Capture | Bytes | md5 |
| --- | --- | --- |
| ws-01-workspace-surface.png | 57522 | `a9ad7e0c…` |
| ws-02-after-ctrl-p.png | 58675 | **differs — the "Search files" palette opened** |
| ws-03-after-escape.png | 57522 | identical to ws-01 — the palette closed cleanly |

VLM read of ws-02 (`vlm-ws02-read.md`): a search overlay centered on screen;
input placeholder "Search files"; a "Files" section with "Type to search for
files"; cursor focused in the input. Exactly the palette surface the fix
routes Ctrl+P to.

## Scene scripts

- `d10-ctrl-p-file-search.sh` — baseline probe (parity-lab/scenes/)
- `d10b-ctrl-p-workspace.sh` — workspace-seeded probe (parity-lab/scenes/)

Binary: guarded release build of the branch (BUILD_EXIT=0; the release deps
cache from the main `3c9f113` verification build). The post-clippy-fix head
`4f5a451` touches test code only — the shipped binary is unchanged.
