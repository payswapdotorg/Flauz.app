# WO-P2-017 / WO-P2-017-FIX evidence — palette & shortcuts-overlay close restore focus (D17)

Closing the last P2 follow-up of the F1 parity batch: the command palette and
keyboard-shortcuts overlay close paths restore focus to a determinate surface,
and the r1 deterministic palette-close panic (entity-map double-lease) is
eliminated — verified structurally (compile-time signature pin), by CI, and at
runtime (this D17 scene).

- **Merged:** PR #41 → merge commit `8dcfcb97b34bc9d4fd73bee534628d5e9c789abe`
  (worker delivery `a079caf` on base `c2de31e` exactly, ui.rs-only +298/−25;
  Lead rustfmt follow-up `1886ce2` — four layout-only sites per the CI Format
  step, the WO-P2-019 house pattern).
- **CI:** double matrix green at `1886ce2` (ubuntu-24.04 + windows-latest:
  Format, Clippy, full workspace Test including the four focused tests, Release
  build, Linux startup smoke). The worker sandbox had no Rust toolchain —
  compile/test/fmt/clippy rode CI (the established UX-002/019 pattern).
- **Binary:** guarded local release build `codexrs-wo017fix-1886ce2`
  (units=16/LTO=false/strip=symbols, 49.5 MB).

## The fix (verified-design addendum, applied mechanically)

`WorkspaceView::close_command_palette` now takes a captured
`CommandPaletteCloseContext { files_mode }` argument — the palette entity is
never read from the workspace side while mid-update. Both palette-side close
paths (`close_and_clear`, `open_file_result`) capture the context BEFORE
entering `self.workspace.update(...)`; the workspace-side paths (backdrop
click, the `toggle_keyboard_shortcuts` chord) read the idle palette through
`command_palette_close_context` under a workspace lease only. The r1 crash
shape is structurally excluded by the compile-time fn-pointer signature pin
`workspace_close_command_palette_requires_a_captured_close_context` (a runtime
test cannot catch r1: its unit tests passed while the binary crashed), with
the capture-mapping regression
`command_palette_close_context_captures_the_files_mode` and the two r1
focus-contract tests ported verbatim.

## D17 scene (this directory)

`d17-palette-close-focus.sh` (patched with the D19 lessons: defensive
fresh-profile promo dismissal + windowfocus re-assertion before every keyboard
burst; the B01–B10 frame contract unchanged). Display :107, pinned runtime,
donor state, picom; typed-probe focus method: after each close, type one
character — it lands in the composer iff focus was restored there.

| Frame | Step | md5 |
|---|---|---|
| B01 | entry baseline | `656047f76b6a8cb29d23f18432b48e20` |
| B02 | composer focus (D9 click ladder) | `2caa826523d506f4778289ec666a8a43` |
| B03 | Ctrl+K — palette OPEN | `72b22a76cfc629b5f794f56fbdf013f5` |
| B04 | Escape — palette CLOSE (restore fires) | `edd5d5080637fe9f4686e2be6a6d39c0` |
| B05 | TYPE "x" — focus probe | `c1f2c6723d866ce82aefb92942d9a864` |
| B06 | clear composer | `edd5d5080637fe9f4686e2be6a6d39c0` |
| B07 | Ctrl+/ — shortcuts overlay OPEN | `a5d5365153e210edd5041a12e5dddb64` |
| B08 | Escape (empty query) — overlay CLOSE | `edd5d5080637fe9f4686e2be6a6d39c0` |
| B09 | TYPE "y" — second focus probe | `b24e16611c240e2629a3a040b5fb5645` |
| B10 | settle | `edd5d5080637fe9f4686e2be6a6d39c0` |

**Adjudication.**

1. **No panic on Ctrl+K → Escape** — the r1 reproduction killed the app at
   exactly this step (gpui `double_lease_panic`, entity_map.rs:138). At
   `1886ce2` the palette closes and the app keeps rendering (B04 differs from
   B03; B05–B10 continue interacting with a live app).
2. **Focus restored to the composer after the palette close** — the typed "x"
   visibly lands (B05 ≠ B04) and the VLM read (`vlm-d17-05-probe-x.json`)
   confirms: the composer input contains the single letter 'x' with the caret
   immediately after it. Clearing the composer returns the frame to the exact
   closed-state md5 (B06 == B04), proving the "x" was the only delta.
3. **Focus restored after the shortcuts-overlay close** — same pattern on the
   second leg: the typed "y" lands (B09 ≠ B08; VLM read
   `vlm-d17-09-probe-y.json` confirms the letter + caret), and the settle
   frame returns to the closed-state md5 (B10 == B08 == B04).
4. **Deterministic restore semantics** — the previous-surface → composer →
   no-focus ladder and the two-Escape overlay contract are covered by the four
   focused unit tests in the CI matrix; the scene exercises the dominant
   journey (composer focused → Ctrl+K → Escape → composer).

## Provenance

Fought the platform capacity regime across ~13 recycle generations (sessions
01:49–09:08 UTC: e9d0ce3c → 3816a669 → 00184fab → 56815089 → 3f36d55c →
a4ecac7a → 93a0b967 → a800ae3a → cadd702a → dbf86af5 → 05ac6bfd → 11dc8f4d →
2d94b1c4; queue_watch unstick/assault + stall_recovery r2 dead-turn
recycles). The winning session `2d94b1c4` (dispatched 08:44 through the
assault chain) landed ~08:45, streamed 179K chars/165 blocks, filled report at
09:08 — contract deviations NONE, base `c2de31e` re-verified at clone time per
the work order's own rule (the addendum's `e4a9894` acknowledged stale in the
report). Harvest via the workspaces files API within minutes of COMPLETE
(`ws-a67231d7`; the autonomous harvest watcher fired — its WS-id parse heuristic
missed the `function_name` field, manual harvest completed; runbook recorded).
An earlier generation's server-confirmed COMPLETE (`e9d0ce3c`, 02:23) was
forensically closed as a report-shaped failure: no branch push, no marker, the
workspace recycled — lesson: harvest must follow COMPLETE promptly (now
institutionalized in the harvest watcher).

Lead verification: full-diff source review (454 lines) against the
verified-design addendum — every element confirmed; CI double matrix (the
pulled_request event fired normally, no swallowed-event workaround needed);
guarded release build (disk-tight: 304 MB free at completion, the 019
mitigation playbook applied — scaffold/.next, lab target, crate archives
freed); D17 scene at the verified SHA with VLM adjudication of both probes.
