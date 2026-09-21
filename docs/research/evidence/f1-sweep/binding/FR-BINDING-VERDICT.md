# RG-FR binding-pass verdict — WO-REL-002 scenes FR-1/FR-3 at the published v0.1.0-rc.14

- **Work order:** WO-REL-002 (fresh-machine/packaging release scenes — the
  GUI subset; FR-2 rides RG-RECONNECT-05, FR-4 is the fresh-machine
  validation at the published archive)
- **Tree:** binary `codexrs-v0.1.0-rc.14` (the published release archive,
  sha256-verified `9eb1139c…05d4` — see A11Y-BINDING-VERDICT header)
- **Date:** 2026-09-21 (runs 11:17–11:20 UTC)
- **Scenes:** `rg-fr1-about.sh` (calibration + DATA) → `rg-fr1d-about.sh`
  (the completed flow); `rg-fr3-openfolder.sh` (pristine boot + offer +
  picker attempt + workspace truth); evidence in `fr1-rc14/`,
  `fr1d-rc14/`, `fr3-rc14/`

## FR-1 — About window: **PASS (binding at rc.14)**

| Spec clause | Evidence | Verdict |
| --- | --- | --- |
| Help → "About codexRS" opens the native fixed-size 380×360 floating window | fr1d: window `0x400006`, xwininfo Width 380 / Height 360 exactly (`about-window-full-xwininfo.txt`) | PASS |
| Centered on parent | About at +532+270 size 380×360 → center (722,450); the main window center is (722,450) — exact | PASS |
| Reports package version + copyright | VLM: `codexRS` / **`Version 0.1.0-rc.14`** / `© codexRS contributors` — the release artifact's own version string | PASS |
| Refocuses existing instance | second Help→About: about-window-count-refocus = 1; frame 04 == 02 byte-identical | PASS |
| Closes via OK/Escape/native close | Escape → about-window-count-closed = 0 (frame 05 delta) | PASS |

Calibration note: fr1's VLM menubar estimate calibrated Help at (263,64) but
could not locate the About item in the open menu (honest note recorded —
`fr1-rc14/vlm-fr1-aboutitem.json`); fr1d's deterministic pixel-band coords
(Help (375,65) → About (416,275)) carried across boots per the placement law
and opened the window first try.

## FR-3 — empty-workspace "Open folder" offer + handoff: **PASS with the bounded lab note (binding at rc.14)**

| Spec clause | Evidence | Verdict |
| --- | --- | --- |
| The empty workspace offers `Open folder` | fr3-02 VLM: palette Suggested rows "+ New chat (Ctrl+N)", "Open folder (Ctrl+O)" | PASS |
| The promo modal's dismiss path (the FR-BASELINE binding item) | fr3-01 = the pristine-boot "Introducing GPT-5.6-Sol" modal captured; fr3-01b = the post-Escape clean welcome (byte-distinct) | **PASS** (captured at rc.14) |
| Reusing the native folder picker | Ctrl+O on the headless lab opens NO picker (the "non-codexRS window" the ladder detected was picom's own 1×1 utility window — `picker-windows.txt`); no crash, no error toast | bounded lab note (unchanged — no portal backend on Xvfb; not a product defect) |
| Explicit-workspace handoff | the honest guard fires: fr3-05 toast "Select a workspace before searching files." (VLM-verbatim; the d20 contract) + the donor-route runtime evidence (d10c) | PASS (composed evidence) |

## FR-2 / FR-4 (cross-references, no re-run)

- **FR-2**: covered by the RG-RECONNECT-05 evidence family (adjudicated PASS
  at the RC tree — the C-scene failure surface + retry path).
- **FR-4/PKG**: `fresh_machine_validate.sh` PASSED at the PUBLISHED rc.14
  archive (FRESH_MACHINE_VALIDATION_OK; recorded in
  `docs/codex-universal/reports/REL-RC14-report.md` @ `7d1d61d`).

## Verdict

**WO-REL-002 GUI binding: GREEN at the published rc.14** — FR-1 green on
every clause with the release version string verified; FR-3 green with the
promo-dismiss capture the baseline owed; the picker stays the documented
headless-lab bound.

— Tech Lead (Task 60, continuation 17), 2026-09-21
