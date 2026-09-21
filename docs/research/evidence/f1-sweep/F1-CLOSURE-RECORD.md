# F1 CLOSURE RECORD — Codex Desktop parity

- **Gate:** every remaining full-reference difference is explicitly
  classified; release-critical journeys remain green; and the final
  parity/accessibility/release journeys pass.
- **Release under test:** `v0.1.0-rc.14` (tag `ddc614e`; current main
  `7d1d61d` = the tag + the REL-RC14 report). All F1 merges landed:
  WO-P2-004–013/018/020, WO-UX-002 (`41b9ac3`), WO-P2-019, WO-P2-017
  (PR #41 → `8dcfcb9`).
- **Binding binary:** the PUBLISHED release archive
  `codexrs-v0.1.0-rc.14-linux-x86_64.tar.gz` (sha256
  `9eb1139caeca6c6b9fec6c968715f7c715c4fa509dffa42609ef73b0002505d4`,
  re-verified against the published SHA256SUMS at the binding runs) plus the
  guarded build `codexrs-wo017fix-1886ce2` (code-identical; used for the
  D17 scene during the 017 gates).
- **Status:** COMPLETE — every gate clause satisfied; the F1→F2 flip awaits
  the user's review of this record.

## Clause 1 — every remaining full-reference difference is explicitly classified

- FINAL per the D-1 freeze: the parity-matrix is the F1 spine; the 41
  Product-parity rows classified by the WO-F1-SWEEP-001 inventory
  (`parity-inventory.md`, base `a664644`) — at the inventory's base:
  5 closed / 1 in-flight / 17 bounded / 10 P2 / 8 P3, plus the
  PR finding-16 shortcut delta (9 closed, 2 P3).
- **Reconciled at `7d1d61d`:** the one in-flight row (Keyboard and
  accessibility) graduated — its headline remainder (the WO-P2-012 six
  guard residuals) CLOSED with the merge; the row's open remainder is its
  itemized P2/platform-bound tail (remaining stable commands; complete
  focus order; contrast; screen-reader labels + OS-level reduced-motion =
  platform-bound). Final arithmetic: **5 closed / 17 bounded / 11 P2 /
  8 P3 = 41 ✓** (primary classes; per-item tallies unchanged from the
  inventory's §5).
- The re-sliced current-target items (multi-repository projects, artifacts
  renderers/canvas, first-run welcome set) carry as the explicit **F2
  annex** — NOT F1 blockers (the inventory's rows 10/23/37 notes).
- The Projects-and-chats row text staleness (inventory honesty note 3) was
  reconciled by the WO closures (the row now cites WO-P2-008/013 inline).

## Clause 2 — release-critical journeys remain green

- The D-series battery at `8a68c9a`: 9/9 scenes green
  (`battery-8a68c9a/VERDICT.md`) — side chats, files palette, palette rows +
  guards, browser chords, browsing history, unread attention, visible
  entries, files-palette status, palette close.
- The binding re-runs at the final RC (the affected-surface rule):
  - **D17 palette/overlay close focus** at `1886ce2` (the 017 gates): all
    ten frames clean — the md5 ladder + VLM x/y caret reads
    (`d17-wo017fix/`, `wo-p2-017/`).
  - **The a11y binding battery at the published rc.14** (see Clause 3): the
    017 focus contract re-verified through the a02 typed probes (P1a/P1b
    land in the composer), the 019 trap ladder green, the N1/UX-002
    keyboard-only chat creation green.
  - The remaining D-scenes (d7/d9/d10c/d11/d12/d14/d15/d18/d20) are
    unaffected by the 017/019/UX-002 surface set (UI-focus-only changes; the
    017-fix CI double matrix + the full workspace test suite at `1886ce2`
    re-ran their focused tests — 139 ui.rs tests green).

## Clause 3 — the final parity/accessibility/release journeys pass

### Accessibility (WO-UX-001) — baseline + binding BOTH GREEN

- BASELINE at `8a68c9a` (`A11Y-BASELINE-VERDICT.md`): the keyboard journey +
  focus probes + precondition-correct re-probes; the four
  source-unverifiable items settled or narrowed; N1–N4 recorded.
- **BINDING at the published rc.14** (`A11Y-BINDING-VERDICT.md`): every
  baseline PENDING item adjudicated —
  - N1 CLOSED: keyboard-only chat creation works (UX-002 verified: composer
    focus on all `begin_new_chat` activation paths; negative control
    byte-identical).
  - 017 verified: palette/overlay close restores composer focus (typed
    probes land with caret, both close paths).
  - Find positive path: bar + counter "1 / 3+ results" + three highlighted
    matches + advance, on a live chat.
  - Attention positive path: "Chat marked unread" → unread dot → "Cleared
    unread indicators for 1 chat" (VLM-verbatim).
  - The 24-frame journey re-run green (structure = baseline).
  - New bounded findings recorded: **N5** (PTY opens live on a selected chat
    but keyboard focus does not transfer on open — the fix-shape
    recommendation carried to F2 a11y, same axis as the N1/017 family) and
    **N6** (bracket swap chords not re-evidenced positively in the lab
    config; Ctrl+PageDown positive; prior two-chat-fixture + unit evidence
    stands). Neither blocks F1.

### Release gates (WO-REL-001/002, WO-LAB-002)

- **RG-RECONNECT-01..05**: ADJUDICATED PASS at the RC tree (06/07
  platform/mocked-bounded with unit anchors per the inventory).
- **RG-SOAK composite**: PASS — CLOSED (`SOAK-VERDICT.md`): 4 h 10 m at
  `8a68c9a`; G-1 −48.3 % (bound < +10 %), G-3 bounded band, G-5 child-kill
  recovery (+18 s online, task list served to T_END, flat post-recovery
  RSS), G-7 zero panics; G-2/G-4/G-6 honest unit-anchored notes; the −32600
  banner = the documented fail-closed runtime-compat contract.
- **FR-1/FR-3 GUI**: baselines PASS at `8a68c9a` +
  **BINDING PASS at the published rc.14** (`FR-BINDING-VERDICT.md`): the
  About window 380×360 exactly centered reporting **"Version 0.1.0-rc.14"**,
  refocus/close contract green; the Open-folder offer + the honest
  no-workspace guard + the promo-modal dismiss capture (the baseline's owed
  item).
- **FR-4/PKG**: fresh_machine_validate PASSED at the published rc.14 archive
  (FRESH_MACHINE_VALIDATION_OK; `REL-RC14-report.md` @ `7d1d61d`).
- **CI-1..3**: the rc.14 tag pipeline all green (run 35585925384 — licenses,
  both OS package jobs with full gates + startup smokes, publish).

### Release cut

v0.1.0-rc.14 cut at `ddc614e`, published (3 assets + SHA256SUMS), fresh-machine
validated, recorded. Twelve Added + five Fixed user-facing changelog entries
covering the rc.13..rc.14 range.

## Honest notes carried forward (not F1 blockers)

- **The rc.14 promo modal**: fresh-profile/pristine boots show the
  "Introducing GPT-5.6-Sol" modal; the seen-flag does not carry across the
  version bump; it swallows keyboard input until dismissed; **Escape is the
  verified dismissal**. Institutionalized as the defensive dismissal in the
  lab scenes (the FR-3 binding captured the modal + dismiss path). A
  first-run UX note for the product backlog (one Escape too many is needed
  before the app is keyboard-usable on a fresh profile).
- **The a11y platform-bound set** (document, do not block F1): screen-reader
  labels/AT tree, OS-level reduced-motion, status AT announcement, titlebar
  AT naming — upstream-GPUI dependencies, revisited at F11.
- **N2/N3/N4** (Settings escape, empty-state terminal toggle, Settings-surface
  chord silence): bounded notes in the a11y baseline verdict.
- **N5 (PTY focus transfer) / N6 (bracket swap re-evidence)**: the F2 a11y
  binding items (see A11Y-BINDING-VERDICT §2).
- **The lab bounds**: the native folder picker (FR-3, no portal on Xvfb), the
  four flow-gated 019 modals (unit-pinned + structure-shared; the reachable
  Clear modal carries the runtime proof), the seeded-session timeline
  emptiness (RG-RECONNECT-04's note), the auth-wall (all live-chat turns
  error honestly in the lab — the documented fail-closed family).

## The end-game checklist (final state)

1. ✅ Soak completion + gates evaluation — WO-LAB-002 CLOSED
2. ✅ WO-P2-017-fix: harvest → Lead gates (CI double matrix at `1886ce2`,
   D17 + VLM) → merge `8dcfcb9` → closure docs `432dc06`
3. ✅ WO-P2-019: harvest → verify → merge (binding re-run green at rc.14)
4. ✅ WO-UX-002 MERGED (`41b9ac3`; the N1 GUI probe green at the rc.14 binary)
5. ✅ Version bump rc.14 → push → tag `ddc614e` → release.yml published
   (run 35585925384)
6. ✅ fresh_machine_validate at the published rc.14 archive
7. ✅ The binding re-run battery at the final RC: the a11y journey + N1 +
   focus probes + live-chat legs (Find/PTY/attention/swap) + D17-affected
   surfaces + 019's modal scene + FR-1/FR-3 + the promo capture
8. ✅ This closure record → docs commit
9. ⬜ The F1 → F2 gate flip in the roadmap — **awaits the user's review of
   this record**

— Tech Lead (Task 60, continuation 17), 2026-09-21
