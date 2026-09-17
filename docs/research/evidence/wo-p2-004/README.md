# WO-P2-004 evidence — command palette settings-page indexing (LINUX_GUI_LAB)

Captured 2026-09-17 in the LINUX_GUI_LAB (local Debian 13 sandbox; Xvfb
:104 + picom, 1600x1000, isolated HOME/XDG/CODEX_HOME/CODEX_RS_DATA_DIR,
no credentials, lavapipe ICD — the sealed `session.sh` recipe).

Product binary: `codexrs` built from branch
`parity/wo-p2-004-palette-settings` (commit `5d4b083`, dev profile,
`debug=0`, `-Wl,--strip-debug`, the documented `-lXau -lXdmcp` lab link
args), version `0.1.0-rc.13`.

## Scene setup

Fresh unauthenticated workspace (`/home/z/parity-lab/linux-B-004`) —
the palette and every settings page are reachable without sign-in, so no
seeding is needed. Scene `scene-d7.sh`: open the palette (Ctrl+K),
capture the empty-query top, then for each of the six previously-missing
pages (Import, Profile, Browser, Configuration, Hooks, Git) type the
nav-name query, capture the palette resolution, press Return, and
capture the resulting settings page. Supplement `scene-d7b.sh`: reopen
the palette and wheel-scroll the list to reveal the Settings-group rows
that sit below the visible fold (Hooks, Git).

## Captures (VLM-read: glm-5v-turbo; reads reproduced in vlm-reads.txt)

| # | Capture | Evidences |
|---|---------|-----------|
| 01 | `01-boot-entry-surface.png` | Boot state: unauthenticated entry surface (baseline for the scene). |
| 02 | `02-palette-top-settings-group.png` | Palette (Ctrl+K, empty query): the Settings group lists General, Appearance, Keyboard shortcuts, Usage & billing, Computer use, **Profile, Import, Browser, Configuration** (the new entries; visible fold ends at Configuration). |
| 03 | `03-palette-import-resolves.png` | **The ev/18 remediation**: palette query "import" resolves to "Import / Open Import settings" (B2's identical query returned "No matches"). |
| 04 | `04-import-settings-page.png` | Return navigates to the Import settings page (heading "Import", nav row highlighted under Personal). |
| 05 | `05-palette-profile-resolves.png` | Query "profile" → "Profile / Open Profile settings". |
| 06 | `06-profile-settings-page.png` | Profile settings page (sign-in options; nav highlighted under Personal). |
| 07 | `07-palette-browser-resolves.png` | Query "browser" → "Browser / Open Browser settings". |
| 08 | `08-browser-settings-page.png` | Browser settings page (approvals, downloads, site permissions; nav highlighted under Integrations). |
| 09 | `09-palette-configuration-resolves.png` | Query "configuration" → "Configuration / Open Configuration settings". |
| 10 | `10-configuration-settings-page.png` | Configuration settings page (approval policy + sandbox; nav highlighted under Coding). |
| 11 | `11-palette-hooks-resolves.png` | Query "hooks" → "Hooks / Open Hooks settings". |
| 12 | `12-hooks-settings-page.png` | Hooks settings page ("No hooks found" empty state; nav highlighted under Coding). |
| 13 | `13-palette-git-resolves.png` | Query "git" → "Git / Open Git settings" (plus the pre-existing "Disable Git-Based Review" match). |
| 14 | `14-git-settings-page.png` | Git settings page (review toggle, branch prefix, PR merge, commit instructions; nav highlighted under Coding). |
| 15 | `15-palette-settings-group-scrolled.png` | Scrolled palette top: **Hooks** and **Git** rows visible in the Settings group (completing the 11-entry listing with capture 02), followed by the Thread/Navigation/Panels groups. |

## Calibration notes (scene iterations)

- The palette's empty-query list renders grouped with a visible fold:
  with the login-surface command set the fold ends after
  "Configuration", so Hooks and Git required a scrolled capture (D7b).
  Arrow-down selection cycling is NOT a reliable way to scroll the
  grouped list (the selection wraps and the window does not advance
  past the fold); a mouse-wheel scroll over the list body is.
- VLM full-frame reads reliably resolve the palette rows and settings
  page headings at 1600x1000; no zoomed crops were needed for this
  scene (the palette text is large enough at this scale).
- The query captures intentionally show the palette over the
  unauthenticated entry surface, mirroring the official runtime
  reference (evidence `codex-linux/03-04`: the official palette's
  Settings group lists pages at the login surface).

## Relation to prior evidence

- B2 `ev/18` (gap): palette query "import" → "No matches" — the exact
  query this scene proves fixed (capture 03).
- B2 `ev/23` (already working): the in-Settings search filter finds the
  Import page — unchanged by this work order (settings search was and
  stays in scope-out).
- Official runtime reference `codex-linux/03-04`: the official Linux
  preview palette (26.908.70816) lists a dynamic Settings group at the
  login surface (General, Import, Appearance, Voice, Pets, Git,
  Connections, Environments, Worktrees, Configuration). Flauz now
  indexes its own full default-nav set (11 pages) in its Settings group;
  the official grouping itself (single dynamic group for ALL pages,
  including with-query grouping) remains a presentation residual, not
  an indexing gap.
