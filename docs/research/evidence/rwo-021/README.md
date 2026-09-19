# RWO-021 evidence — fresh end-to-end verification pass (WO-REVIEW-001 deep phase)

- **Base:** `main` @ `876bbe862e0626a1b6788038e5ebf4347c918be2`
- **Branch:** `research/rwo-021` (documents-only)
- **Deliverable:** `docs/research/evidence/rwo-020/verification.md` (per the task directive)
- **Date:** 2026-09-19 (UTC)

## Contents

- `frames/` — fresh LINUX_GUI_LAB captures from ten scenes (sealed recipe: Xvfb :104
  1600x1000x24 + picom xrender + `LIBGL_ALWAYS_SOFTWARE=1` + `VK_ICD_FILENAMES=lvp_icd.json`,
  isolated HOME/XDG/`CODEX_RS_DATA_DIR`, workspace seeded via the `recent_workspaces`
  restore path — the D10b device; window resolves to 1278x818 at screen +163,+91).
  Key frames: `a02/a03` (Ctrl+P palette + byte-identical Escape close), `a04` (Ctrl+/
  overlay), `a08` + `j02` (Share-feedback dialog via palette and via typed `/feedback` +
  Ctrl+Enter), `a17–a20` (the four D11b honest statuses, verbatim), `f04` (gh-missing toast
  on the Pull-requests page), `f05` (Plugins marketplace), `f07` (settings search "import" →
  Personal + Import), `j03` (overlay query filter). md5 sequences per the house pattern:
  `scene-md5.txt`, `reprobe-md5.txt`, `scene3-md5.txt`, `scene5-md5.txt`, `scene6-md5.txt`.
- `vlm-reads/` — VLM transcripts (`glm-5v-turbo` via z-ai) for every load-bearing frame.
  Contested frames were re-read with neutral prompts; the neutral reads govern (see
  verification.md §7).
- `scenes/` — the scene scripts (plus `sysroot-materialize2.sh` and `build-env.sh`, the
  userspace build-environment recipe used in this sandbox).

## Build/test summary (details in the deliverable §1)

- `cargo build --locked -p codex-app --release` — exit 0 (8m 50s, pinned 1.97.1)
- Battery (release): core 231/231, storage 19/19, protocol 60/60, platform 114+2/0 (2
  ignored), app 213/213 — **639 passed / 0 failed / 2 ignored**; WO-P2-008's 7 core + 6 app
  tests re-run per-test, all green.
- Input-surface invariants: `MAX_KEYBOARD_SHORTCUT_COMMANDS` = registry = const-array =
  **76**; set- and order-equal; zero dead commands; `PaletteCommand::ALL` = 70; 25
  empty-default ids.
- Defects: one documentation-class finding (KSR-C5 — F-A1/A2/A3 residuals not named in the
  PR §5.10 row); follow-up candidates listed in the deliverable §6.
