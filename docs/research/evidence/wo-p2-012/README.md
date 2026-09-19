# WO-P2-012 evidence — guard honesty (six evidenced silent no-op states + executor fall-through tests)

- **Work order:** WO-P2-012 (wave-1 Worker A, reconciliation leg)
- **Delivery:** `feat/wo-p2-012-guard-honesty-r3` @ `aeccfc9` (base `a664644`; supersedes r1 `0547055` + r2 `4fc763a`)
- **Merged:** PR #34 → `afa5b9d` (CI double-green: ubuntu-24.04 + windows-latest, started 22:59:27Z)
- **Binary under test:** `parity-lab/binaries/codexrs-r3-aeccfc9` (guarded units=16 release build, BUILD_EXIT=0; snapshot md5 `3f7f9330c2e445902159a91aa45f21fd`)
- **Lab:** LINUX_GUI_LAB — Xvfb+picom, donor state, pinned runtime `@openai/codex@0.146.0-alpha.3.1-linux-x64` via `CODEX_RS_CODEX_BIN` + pre-created `CODEX_HOME` (D11r run-3 pattern)

## Run inventory

| Scene | Display | Purpose | Result |
| --- | --- | --- | --- |
| `d15/` (scenes/d15-guard-honesty.sh) | :109 | first full scene: 8 probes | F-D1 verified (late frames); F-A1/A2/A3 chord probes no-op — NUX modal + focus forensics started |
| `d15b/` (scenes/d15b-guard-honesty.sh) | :110 | corrected re-run: neutral defocus, dual-timing captures, immediate /compact | F-D1 re-observed; F-D2 submit never left the composer (retry-gated); chords still no-op (click landed on the modal) |
| `d15c/` (scenes/d15c-chord-probe.sh) | :111 | discriminating probe incl. the D11b-proven `ctrl+shift+u` cross-check | c02–c06 absorbed by the "Introducing GPT-5.6-Sol" NUX modal (up the whole run); c07 `shift+Escape` delta = the modal clearing (VLM pair-diff) |
| `d15c-ab-3c812a7/` (same script, `codexrs-main-3c812a7`) | :111 | A/B against the 011-merge build | **byte-identical frames to the r3 run** (entry `e7185a64…`, c07 `9f953cba…`) — pre-existing behavior, not an r3 regression |
| `d15d/` (scenes/d15d-chord-defocus.sh) | :112 | modal cleared (Escape ×2) + sidebar defocus, then the four chords | **d07 cross-check FIRED** (notification bar "Select a chat before marking it unread." + Dismiss — VLM verbatim); F-A1/A2/A3 chords still no-op in the identical state |

## Adjudication (per state)

| State | Expected honest feedback | GUI outcome | Verdict |
| --- | --- | --- | --- |
| **F-A1** archive, no selection (`Ctrl+Shift+A`) | "Select a chat before archiving it." | chord never reaches the handler at the entry surface — see the chord-shadow finding below; guard unit-covered on CI | **unit-covered; GUI chord path shadow-blocked (pre-existing)** |
| **F-A2** pin, no selection (`Ctrl+Alt+P`) | "Select a chat before pinning or unpinning it." | same | **unit-covered; GUI chord path shadow-blocked (pre-existing)** |
| **F-A3** rename, no selection (`Ctrl+Alt+R`) | "Select a chat before renaming it." / "The selected chat is no longer available." | same (no rename dialog opened — the no-op is honest-side silent) | **unit-covered; GUI chord path shadow-blocked (pre-existing)** |
| **F-A6** commit-or-push, pending PR | "A Git workflow is already running." | pending-PR state not constructible offline; palette "commit" query honestly returns "No matches" (d15-05) | **NOT RUN offline (unit-covered on CI)** |
| **F-D1** `/review` executor, review unavailable | fall-through to a VISIBLE message submission (input never swallowed) | thread row `/review` under Projects (d15-07/08) + context card shows the literal `/review` and follow-up message text (VLM verbatim) | **VERIFIED on the real binary** |
| **F-D2** `/compact`, thread runtime still loading | composer error "Wait for the chat to finish loading before compacting context." + recovery on load | two attempts (d15-07, d15b-06): the /compact submit never left the composer in this lab's unauthenticated retry state (frames + VLM record the staged command pill and the retry banner) | **NOT OBSERVED in lab (unit-covered on CI incl. the TaskRuntimeLoaded recovery path)** |
| guard-family mechanism (`Action::SetStatus` → bottom notification bar + Dismiss) | honest status renders at the entry surface | **d15d d07**: `ctrl+shift+u` → "Select a chat before marking it unread." rendered verbatim on the r3 binary, card-only state, same surface/session where the F-A1/A2/A3 chords no-op | **VERIFIED (the exact mechanism r3's guards use)** |

## The chord-shadow finding (pre-existing, documented residual class)

`Ctrl+Shift+A` / `Ctrl+Alt+P` / `Ctrl+Alt+R` are registered TWICE: as GPUI `KeyBinding`s
(`ArchiveChatShortcut` / `RenameChatShortcut` / `ToggleChatPinShortcut`, ui.rs ~5065–5067,
context `None`) **with no `on_action` listener anywhere**, and as registry items
(`archiveThread` / `renameThread` / `toggleThreadPin`) resolved by the
`intercept_keystrokes` → `execute_keyboard_shortcut_command` path. Empirically
(d15d, same session, same surface, same delivery method):

- `ctrl+shift+u` — registry-only chord (no GPUI binding) → interceptor fires → status renders;
- `ctrl+shift+a` / `ctrl+alt+p` / `ctrl+alt+r` — dual-registered chords → no visible effect.

The working hypothesis (binding-shadow: the bound-but-unhandled action consumes the
keystroke before the interceptor path resolves the registry command) matches the
documented parity-matrix residual "**23 bound-but-never-handled GPUI actions**"
(§5.10 Keyboard row, recorded per RWO-021 KSR-C5 — the three Shortcut actions are
members of that class). A/B-verified pre-existing: the d15c probe produces
byte-identical frames on `codexrs-main-3c812a7` (011-merge build, r3's diff does
not touch key bindings). WO-P2-012's guards are correct at the handler level and
become GUI-reachable via these chords once that residual class is worked —
recommended follow-up scope pairs with the f1-sweep D-3 decision set.

Also recorded: single-modifier chords (`ctrl+k`, `shift+escape`) and the
workspace-surface `ctrl+shift+c` (D14-07, registry-only) deliver fine; the
"Introducing GPT-5.6-Sol" first-run NUX modal absorbs ALL chords while open
(d15c c01–c06; `shift+escape`/`escape` dismiss it — the d15c c07 delta is the
modal clearing, VLM pair-diff).

## Frame/VLM inventory

- `d15/`: 9 frames + md5 table; VLM reads `vlm-d15-01/05/06/07/08.json`
- `d15b/`: 12 frames + md5 table; VLM reads `vlm-d15b-05/06e/06l.json`
- `d15c/` + `d15c-ab-3c812a7/`: 7 frames each + md5 tables (byte-identical across binaries); VLM `vlm-d15c-07.json`, `vlm-d15c-0107.json`, `vlm-d11b-01-recheck.json` + `vlm-d11b-02-where.json` (re-check reads of the D11b frames, copied here)
- `d15d/`: 11 frames + md5 table; VLM `vlm-d15d-07.json`
- Scene scripts: `parity-lab/scenes/d15-guard-honesty.sh`, `d15b-guard-honesty.sh`, `d15c-chord-probe.sh`, `d15d-chord-defocus.sh`

## Local battery record

- `cargo fmt --check -p codex-app -p codex-core` — clean on the branch (Lead station)
- guarded incremental build (units=16, lto=false, `-j 1`, memory-guarded): BUILD_EXIT=0; binary snapshotted
- full workspace battery: carried by the PR #34 CI double matrix (authoritative per the WO-P2-011 NOT RUN doctrine — disk/memory-exhausted 4G sandbox; two guard kills at 450M/250M floors + a leaked-console maintenance cycle are recorded in the run logs)
