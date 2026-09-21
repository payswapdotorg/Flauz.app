# D19 binding-pass verdict — WO-P2-019 modal focus traps at the published v0.1.0-rc.14

- **Tree:** binary `codexrs-v0.1.0-rc.14` (published archive, sha256-verified)
- **Date:** 2026-09-21 (run 11:12–11:16 UTC; the first attempt's Clear-click
  miss is preserved below as the calibration record)
- **Scene:** `d19-modal-traps-regression.sh` (phases A/B/C); evidence in this
  directory; the committed 019 evidence ladder at
  `docs/research/evidence/wo-p2-019/d19-md5.txt` is the comparison baseline.

## The trap ladder (rc.14 vs the committed 019 record)

| Frame | rc.14 md5 | Meaning | Matches the 019 signature |
| --- | --- | --- | --- |
| F04 | c3063ea2… | Clear modal OPEN ("Clear browsing history?" / "This removes all stored browsing history from this device" / Cancel + Clear — VLM) | ✓ (modal open ≠ settings frame) |
| F05 | ae531f43… | Tab #1 → focus moved | ✓ |
| F06 | 324cf2d2… | Tab #2 → WRAP (the trap proof) | ✓ |
| F07 | ae531f43… | Tab #3 → alternated back | ✓ **F07 == F05 byte-identical** |
| F08 | 324cf2d2… | Shift+Tab → backward wrap | ✓ **F08 == F06 byte-identical** |
| F09 | ab1c202d… | Escape closed the modal | ✓ |
| F11–F13 | 659e92cd… | composer-focus ladder + background Tabs (focus advances without surface change) | ✓ |

The F05=F07 / F06=F08 byte-identical alternation is the same trap signature
as the committed 019 ladder (5252…/3f42… alternation). The four flow-gated
modals (remote pairing / remote confirmation / account logout / plugin
install) remain unit-pinned + structure-shared per the 019 closure record
(`docs/research/evidence/wo-p2-019/README.md` — no backend auth/flows in the
lab; the reachable Clear modal carries the runtime proof of the shared trap
component).

## Calibration record (the rc.14 lesson)

The 019-era Clear-button coordinate (757,652) MISSED at rc.14 — the layout
shifted; VLM's re-estimate (758,654) was ~320 px off (the FR-1 error-bar
lesson). **Pixel-band ground truth**: the red-button cluster spans
x 1061–1099 / y 571–594 → click (1080,582) — the corrected scene comment
records it. First-attempt frames (F04–F10 all `977744a5…` = no modal) are
preserved in this directory's git-ignored predecessors as the honest miss
record.

## Verdict

**WO-P2-019 binding at rc.14: GREEN** — the trap ladder reproduces the
committed signature exactly at the published binary.

— Tech Lead (Task 60, continuation 17), 2026-09-21
