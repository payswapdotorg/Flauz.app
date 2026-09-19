# RWO-020 — Reference-behavior review rubric (WO-REVIEW-001 deep phase)

- **Task ID:** RWO-020 (one third of WO-REVIEW-001)
- **Worker:** RWO-020 (review worker — rubric producer; does NOT verify Flauz)
- **Date:** 2026-09-19 (UTC)
- **Base:** `main` @ `d479c7b9e725e1af1353140d49226b3f9a16be9d` (verified:
  commit exists, is `origin/main` HEAD at clone time)
- **Branch:** `research/rwo-020` (off the base SHA)
- **Deliverable:** this file (`docs/research/evidence/rwo-020/rubric.md`) +
  `notes.md` (derivation notes)
- **Consumers:** Worker **B** executes this rubric against Flauz at the
  Tech Lead's designated review base; Worker **C** attacks B's execution.
  The Tech Lead re-verifies every claim at the integration station.

## 0. What this rubric is — and is not

This rubric decomposes the parity-matrix rows that claim a non-missing
status into **checkable verification criteria**. Each criterion states the
exact official reference behavior, the exact Flauz claim under test, the
evidence class that proves or disproves it, and the **cheapest honest
verification step** available inside this repo's evidence doctrine.

It is **not** a verification report. Nothing here claims Flauz passes or
fails; B produces those verdicts. Every criterion carries the reference
version it derives from, per the program's version rules:

| Tag | Meaning |
| --- | --- |
| `26.721` | Historical baseline (Codex Desktop `26.721.3996.0` + bundled CLI `0.146.0-alpha.3.1`) — in target |
| `26.727→26.825` | Post-baseline feature line that is inside the current target (ChatGPT desktop `26.825.51511`) — in target |
| `26.908 ref+1` | Linux preview `26.908.70816` runtime observations — **never the target bar**; usable for naming/confirmation only, always with the version-skew label |

Delayed capabilities (cloud environments, scheduled tasks, Pets, Appshots,
some Sites, and the other §8.2 "Deferred" entries) are **documented
deferrals, not bugs** — no criterion may count them against parity, and no
verification may fake them.

## 1. Citation key (this rubric cites only the repo's own evidence layers)

| Tag | Source |
| --- | --- |
| PR §n | `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` §n (canonical matrix) |
| PM | `docs/parity-matrix.md` (26.721-era historical record) |
| KF | `docs/known-failures.md` (stable-failure table) |
| A §n | `docs/research/CODEX-REFERENCE-MATRIX.md` §n (Worker A, `c083c38`) |
| B2 §n | `docs/research/FLAUZ-REFERENCE-MATRIX.md` §n (Worker B2, `a2343d3`) |
| Δ | `docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md` |
| LX | `docs/research/evidence/codex-linux/` (official Linux preview, 26.908 ref+1: `README.md`, captures `01`–`09`, `vlm-reads.txt`, scene scripts) |
| REF | `docs/research/evidence/p2-batch2-reference/README.md` (WO-R-REF: R1/R2/R3) |
| SWEEP | `docs/research/evidence/wo-p2-007/input-surface-sweep.md` (WO-R-SWEEP; anchors at base `3c9f113`) |
| VDN | `docs/research/evidence/codex-ref/version-delta-notes.md` |
| CL | `docs/research/evidence/codex-ref/changelog-delta-extract.md` (verbatim changelog) |
| ev/NN | `docs/research/evidence/flauz/NN-*.png` (B2 curated runtime captures) |
| WO-xxx | per-work-order evidence dirs under `docs/research/evidence/` (`wo-p1/`, `wo-p1-003/`, `wo-p2-004/`, `wo-p2-005/`, `wo-p2-006/`, `wo-p2-007/`, `wo-p2-008/`) |
| src | read-only source reads at HEAD `d479c7b` (anchors I re-verified for this rubric are listed in Appendix A; anchors inherited from older bases drift — B re-locates by token, not line number) |

Provenance labels follow PR §4.3 exactly (`[runtime-observed]`,
`[source-derived]`, `[docs-derived]`, `[historical-record]`, plus
`[runtime-observed: linux-preview 26.908.70816]` and `[unverified]`).
Conflict order (PR §7.3): runtime-observed > source-derived > docs-derived >
historical-record. Platform honesty (PR §7.4 / ledger rule 5): Linux runtime
evidence never closes a Windows/macOS cell and vice versa.

## 2. Verification classes and fraud-resistant evidence requirements

B's cheapest honest step is always one of:

1. **source-read** — read the anchor in `crates/**` (re-locate by token at
   the review base; line numbers in older evidence drift);
2. **unit-test** — run (or cite CI-green at the exact SHA for) the named
   focused test(s); never a paraphrase — record the test id and result;
3. **GUI scene** — LINUX_GUI_LAB scene (Xvfb + picom + capture +
   VLM-read), following the sealed `session.sh` recipe used by every WO
   evidence dir; frames must come with an md5 sequence when the claim is
   "something changed / nothing changed";
4. **runtime overlay** — comparison against the official Linux preview
   app evidence already archived under LX (never a fresh claim about
   26.825; 26.908 is ref+1).

Evidence-quality rules (the program's own patterns; C rejects violations):

- GUI claims need the frame(s) + VLM-read transcript + (for
  changed/unchanged claims) md5 sequences — see `d11b-md5.txt`,
  `d10-md5.txt`, `d9b-*` for the house pattern.
- "Silent no-op" claims need byte-identical-frame md5 proof (the
  `09-noprobe-ctrl-shift-o.png` / D10 baseline pattern).
- Test claims need the test name + where it ran (local vs CI matrix).
  CI is the authoritative battery (both `windows-latest` and
  `ubuntu-24.04`).
- **Honest NOT RUN:** if a step cannot run in the sandbox, record the
  exact reason (per the WO-P2-008 D11 precedent). Fabricating a result is
  automatic failure with zero credit.
- Version-skew discipline: any use of LX (26.908) material must carry the
  `ref+1` label in the same sentence; any criterion whose reference is
  `26.727→26.825` must not be re-bar-lowered to baseline-only.

## 3. Environment questions and bounds B must resolve first

- **EQ-1 (lab runtime availability).** The D9 evidence (WO-P2-006,
  2026-09-18) shows a main chat created in-lab via composer submit
  (`d9-wo-p2-006-side-chats-verify.sh` step 1), while the D11 evidence
  (WO-P2-008, 2026-09-19) states no `codex` CLI exists in the lab and the
  sidebar shows "No chats" without a runtime. Before planning any
  thread-dependent GUI scene, B must determine what
  `resolve_codex_binary` (src: `crates/codex-platform/src/lib.rs:261`)
  currently resolves to in the lab, and state it. If a runtime (or
  CODEX_RS_CODEX_BIN) is available, thread-dependent scenes (SCH-C2/C3,
  PRJ-C4, ACT-C4/C5, NTR-C1) are GUI-verifiable in the D9 pattern; if
  not, they fall to the D11b-class honest bound (source-read + unit-test +
  NOT RUN with reason).
- **EQ-2 (Windows/macOS).** `WINDOWS_GUI_LAB` / `MACOS_GUI_LAB` remain
  unavailable (PR §3). Windows-row evidence stays `[historical-record]`;
  the windows-latest CI matrix is the only Windows-side executable
  evidence class available.
- **EQ-3 (auth wall).** No credentials in the lab. Official account
  surfaces (sidebar, Activity view contents, settings page content) are
  auth-walled `[unverified]` — LX README finding 1. Flauz-side surfaces
  that render unauthenticated are GUI-verifiable.
- **EQ-4 (26.825 bundle).** No 26.825 asar inspection exists (VDN §E.1).
  Any criterion needing official 26.825 micro-detail must carry
  `[unverified]` on the official side and verify only the Flauz claim
  against the documented reference.

## 4. The rubric

Row order follows the task directive: the 8 `complete` rows first (PR
§8.1), then the 5 named highest-traffic `partial` rows. Statuses and cells
are quoted from PR §5 at the base SHA.

### 4.1 Runtime bootstrap — PR §5.1, status `complete` (all cells)

**Official reference (26.721, `[historical-record]` PM:75):** packaged-CLI
hash pin + explicit override/fallback order. Recovery cell: "fallback
order on CLI mismatch". Official-side contrast (26.908 ref+1,
`[runtime-observed]` LX README finding 2 + §9 override 15): the official
app **fatal-errors without its bundled runtime** — an official resilience
bound, NOT a Flauz bar; Flauz's graceful degradation is the ahead-axis.

**Flauz claim under test (PR §5.1):** exact packaged-CLI hash check;
override/fallback order preserved; boots to entry surface with app-server
online footer `[runtime-observed B2 ev/06]`.

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| BOOT-C1 | The CLI SHA-256 pin exists and equals the pinned reference hash | 26.721 `[historical-record]` PM:21-22 | source-read | `STABLE_REFERENCE` at src `crates/codex-core/src/lib.rs:227-234` (anchor re-verified, Appendix A) | `cli_sha256 == 39e9e041ea33ac34aad9578adfe660c5c7a6dc8f82620b77623960f9352a6ef3`, `cli_version == "0.146.0-alpha.3.1"` |
| BOOT-C2 | The override/fallback order is explicit and preserved (explicit arg → `CODEX_RS_CODEX_BIN` → Windows stable cache → Windows npm candidate → remaining fallbacks) | 26.721 `[historical-record]` PM:75 | source-read | `resolve_codex_binary`, src `crates/codex-platform/src/lib.rs:261-281+` (read to the end of the function) | order matches the PM-recorded order; no silent reordering or removed branch |
| BOOT-C3 | The hash pin is **enforced** (a resolved binary that mismatches the pin takes the recorded fallback/refuse path, never silent acceptance) | 26.721 `[historical-record]` PM:75 + PM:75 Recovery cell | source-read + unit-test | locate the comparison consumer of `cli_sha256`/`stable_reference()`; find the focused test covering mismatch behavior | comparison exists on the live spawn path AND a focused test exercises it; if no test exists, record defect-found (missing coverage) or not-run with reason |
| BOOT-C4 | Boot reaches the entry surface with the app-server online footer | Linux `[runtime-observed]` ev/06 | GUI scene | re-run the boot scene (b1/d2 pattern, `wo-p1/d2-default.sh`); VLM-read the footer | frame shows the online footer state; cite frame + VLM read |
| BOOT-C5 | No-runtime degradation is graceful (no fatal, no crash; "Resolving…"/auto-retry per ev/24) — the row's Recovery claim | 26.721 Flauz-side claim; official contrast is ref+1 LX | GUI scene | with no resolvable runtime (the EQ-1 state), boot and capture | no fatal exit; degraded-but-alive surface; frame + md5 sequence |

**Attack surface (C):** a hash constant that exists but is never compared
(dead pin); C2 verified only on the non-Windows branches (`#[cfg(windows)]`
arms must be read, not skipped); BOOT-C4 "booted" evidence produced on a
machine where a runtime exists while claiming the no-runtime lab bound;
footer status read from a stale frame (must be from B's own capture).

### 4.2 Multi-root workspace handling — PR §5.6, status `complete` (regression control)

**Official reference (26.721-era public failure report, `[historical-record]`
KF row 1):** the official Windows app white-screened on multi-root
workspaces. **Flauz claim (KF):** native `Path`/`PathBuf`, no browser path
shim — Windows multi-root white screen controlled (acceptance test).
PR §9 override 9 is binding: this row is the **Windows white-screen
regression control**, NOT the multi-folder projects capability (4.9
below). Any verification that conflates them is invalid.

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| MRW-C1 | Workspace/project roots are handled with native `Path`/`PathBuf` end-to-end (no browser/JS string path shim anywhere on the project path) | 26.721 `[historical-record]` KF | source-read | sweep the project/workspace path model (`LocalProjectSummary` paths, storage layer, file-search plumbing) for any URL/string-join path handling | native `std::path` throughout; any shim found = defect-found |
| MRW-C2 | The control is a **standing acceptance test** that exercises a Windows-style (drive-letter / multi-root) path through the model | 26.721 `[historical-record]` KF "acceptance test" | unit-test | locate the acceptance test(s) in-tree; confirm they run on the windows-latest CI matrix at the review SHA | test exists + green on both CI matrices; windows-latest absence = not-run with reason (Windows row needs it) |
| MRW-C3 | The KF failure-table entry stands (documented, "Implemented") | documentation | source-read | `docs/known-failures.md` row 1 | row present and accurate |

**Attack surface (C):** substituting WO-P1-003 multi-folder GUI evidence for
this row (conflation — override 9); claiming "controlled" with only
ubuntu CI green (the failure is Windows-specific; the windows-latest matrix
is the load-bearing evidence); MRW-C1 "swept" by grepping two files.

### 4.3 Git process hygiene — PR §5.7, status `complete` (regression control)

**Official reference (26.721-era public failure report, `[historical-record]`
KF row 4):** repeated `git.exe` spawning after filesystem notifications.
**Flauz claim (KF):** 300 ms debounce, notification coalescing, one backend
Git operation at a time (acceptance test).

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| GPH-C1 | Git filesystem-notification debouncing (~300 ms) + notification coalescing exist on the live path | 26.721 `[historical-record]` KF | source-read | `crates/codex-platform/src/git.rs` (and the watcher wiring): locate the debounce constant and the coalescing queue | both present on the notification→spawn path, not dead code |
| GPH-C2 | One backend Git operation at a time (serialization/single-flight) is enforced | 26.721 `[historical-record]` KF | source-read | same file: the single-flight lock/queue on backend Git ops | serialization exists and covers all git spawns (not a subset) |
| GPH-C3 | The control is a standing acceptance test, green on CI | 26.721 `[historical-record]` KF | unit-test | locate the focused test(s); cite the CI matrix at the review SHA | named test green on both matrices |
| GPH-C4 | KF row stands | documentation | source-read | `docs/known-failures.md` row 4 | row present and accurate |

**Attack surface (C):** debounce constant present but bypassed by a second
un-debounced watcher; serialization covering only one of several git spawn
sites; "green on CI" cited without the SHA.

### 4.4 Marketplace admin-disabled install — PR §5.8, status `complete`

**Official reference (26.721 `[historical-record]` PM:93):**
`DISABLED_BY_ADMIN` availability; disabled catalog actions + tooltip;
`Disabled by admin` details. **Flauz claim (PM:93):** availability
preserved, actions disabled with the recovered `Access is turned off by
your admin` tooltip, details show `Disabled by admin`.

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| MKT-C1 | `plugin/list` availability values (incl. `DISABLED_BY_ADMIN`) pass through uncollapsed — never mapped to generic "unavailable" | 26.721 `[historical-record]` PM:93 | source-read | the marketplace/plugin availability model + its render/consumption sites | the enum/value survives end-to-end; any collapse into a generic state = defect-found (this is exactly the failure PM warns about) |
| MKT-C2 | Admin-disabled catalog actions render disabled with the recovered tooltip `Access is turned off by your admin` | 26.721 `[historical-record]` PM:93 | source-read (+ unit-test if a copy test exists) | src anchor `crates/codex-app/src/ui.rs:31885` (re-verified, Appendix A): read the surrounding block — actions disabled AND tooltip attached | both the disabled state and the exact tooltip string on the actions |
| MKT-C3 | Plugin details show `Disabled by admin` (not collapsed policy) | 26.721 `[historical-record]` PM:93 | source-read | locate the details render for the admin-disabled state | exact string present in the details surface |
| MKT-C4 | Rendering check (bounded): a plugin catalog fixture with `DISABLED_BY_ADMIN` renders the disabled surface | 26.721; Platform cell is `win [historical-record]` | GUI scene **or honest not-run** | only if a seedable fixture exists (the wo-p1-003 `seed-*.py` pattern); otherwise unit-test + not-run | GUI frames OR recorded not-run with the platform/fixture reason — never a claimed GUI run that did not happen |

**Attack surface (C):** tooltip string matched while actions remain
clickable; details string present but availability already collapsed
upstream (C1 is the load-bearing criterion); C4 "verified" via screenshot
of a different state.

### 4.5 Keyboard shortcut reference — PR §5.10, status `complete`

**Official reference (26.721 `[historical-record]` PM:113):** the exact
stable `Keyboard shortcuts` dialog listing **only active bindings**,
opened from Help + the effective Ctrl/Cmd+/ action; stable category
order; searchable. 26.908 ref+1 (`[runtime-observed: linux-preview
26.908.70816]` LX captures 05-07): a searchable "Keyboard shortcuts"
overlay ("Search shortcuts", 22 rows) — naming/confirmation only, never
the target bar. **Flauz claim (PR §5.10):** opens from Help + Ctrl/Cmd+/;
stable category order; overlay + searchable editable settings page both
visually confirmed `[PM; runtime-observed ev/11, ev/15]`.

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| KSR-C1 | Both entry paths open the shortcuts surface: Ctrl/Cmd+/ key and the Help/menu row | 26.721 `[historical-record]` PM:113 | GUI scene + source-read | entry-surface scene: press Ctrl+/; capture + VLM-read; source: the menu row and the `showKeyboardShortcuts` arm (src `ui.rs:10476` at HEAD per sweep §B; re-locate by token) | surface opens from both; frames + VLM read |
| KSR-C2 | The overlay lists **only bindings active in codexRS** — registry items with ≥1 effective binding; empty-default commands are not advertised | 26.721 `[historical-record]` PM:113; mechanism per SWEEP §F (overlay filter at base `3c9f113` ui.rs:38943) | source-read + GUI scene | source: the overlay's effective-binding filter; scene: one overlay frame; count advertised rows | filter present; advertised-row count matches the source-derived count at the review base (recount — do not reuse the 3c9f113 count of 47; the registry is now 76) |
| KSR-C3 | Stable category order Chat → Navigation → Panels → Project → Skills → Configure → App → General holds for the stable groups | 26.721 `[historical-record]` PM:112-113 | source-read + GUI scene | the group comparator + one overlay frame | the eight stable groups appear in the documented relative order; any Flauz-added group (e.g. `Thread`, which now carries openSideChat/toggleThreadUnread — src `ui.rs:2826-2852`, Appendix A) is **explicitly inventoried in B's report** as an addition, not hidden — a silently reordered stable group = defect-found |
| KSR-C4 | The searchable editable settings page renders the registry and filters by query (`/settings/keyboard-shortcuts` equivalent) | 26.721 `[historical-record]` PM:113; `[runtime-observed]` ev/11 | GUI scene | settings scene: open Settings → Keyboard shortcuts; type a bounded query; capture | page renders + filtering works; frames + VLM read |
| KSR-C5 | Overlay honesty vs known silent states: no advertised row may describe a confirmed runtime no-op without a documented residual (cross-check SWEEP §F: F-A1/A2/A3, F-A6, F-D1, F-D2) | 26.721 claim + SWEEP findings-of-record | source-read | re-run the SWEEP verification-battery greps (SWEEP §"VERIFICATION BATTERY") at the review base; diff the F-findings list | every still-silent advertised state is either fixed or documented as a residual in the parity row; an undocumented new silent state = defect-found |

**Attack surface (C):** counting the 26.908 22-row overlay as the target
bar (the version-skew trap — REF §R3.7 explicitly forbids it); verifying
the overlay but not the settings page (the claim covers both); "only
active bindings" verified by counting rows without checking the
empty-default filter; KSR-C5 answered from memory instead of re-running
the greps at the review base.

### 4.6 Feedback — PR §5.10, status `complete`

**Official reference (26.721 `[historical-record]` PM:103):** the exact
stable `Feedback` command + `/feedback`; five category IDs; required
details; default-on session logs. **Flauz claim (PR §5.10):** native
`Share feedback` dialog with the five recovered category IDs, validation,
default-on logs, no browser-tabs control; palette entry exists
`[PM; source-derived]`.

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| FBK-C1 | Exactly the five recovered category IDs exist and are the submission literals | 26.721 `[historical-record]` PM:103 | source-read | `FeedbackClassification` src `crates/codex-core/src/lib.rs:4358-4374` (re-verified, Appendix A): confirm `ALL: [Self; 5]` and every `as_str` literal (`bug`, `bad-result`, `good-result`, + the two remaining — B reads them) | five variants, exact literals; any sixth category or renamed literal = defect-found |
| FBK-C2 | Required-details validation: submit is blocked without a classification and without details, with the recorded error surface | 26.721 `[historical-record]` PM:103 | source-read + unit-test | submit path at src `ui.rs:9836+` (classification guard) and the details/validation guard; the `Action::SubmitFeedback` tests at `crates/codex-core/src/lib.rs:32947+` | guards on the live submit path + focused tests green |
| FBK-C3 | Session logs default ON | 26.721 `[historical-record]` PM:103 | source-read | `open_feedback_modal` src `ui.rs:9754-9770`: `feedback_include_logs = true` on open (re-verified, Appendix A) | default-on at every dialog-open path |
| FBK-C4 | No browser-tabs control when no browser surface exists (negative claim) | 26.721 `[historical-record]` PM:103 | source-read | the dialog render block (src `ui.rs:41722+` area) | no unconditional browser-tabs control; conditional (if any) requires an actual browser surface |
| FBK-C5 | Entry paths route to the same dialog: palette `Feedback` row, `/feedback`, Help/menu | 26.721 `[historical-record]` PM:103 | GUI scene + source-read | unauthenticated scene: open via Ctrl+K → "feedback" and via typed `/feedback`; SWEEP §C/D confirms arms at base (`open_feedback_modal` ui.rs:4096 / 7889) — re-locate by token | both open the `Share feedback` dialog; frames + VLM read |
| FBK-C6 | Submission contract: pinned typed `feedback/upload` through the supervised app-server; bounded details; thread ID when present; app version tag; redacted provider payloads; exact `Feedback uploaded` + retry copy | 26.721 `[historical-record]` PM:103 | source-read + unit-test | the `Effect::SubmitFeedback` handler + its tests; submission e2e through a real app-server needs a runtime (EQ-1) — if unavailable, honest not-run for the e2e leg | handler + tests verified; e2e either evidenced or recorded not-run with the EQ-1 reason |

**Attack surface (C):** verifying only the dialog render and skipping FBK-C6
(the deep contract is the row's content); claiming an authenticated
submission e2e in a lab with no runtime (fabrication); the five IDs present
in the enum but different literals serialized.

### 4.7 Stable-failure regression controls — PR §5.10, status `complete`

**Official reference (26.721-era public failure reports, `[historical-record]`
KF table):** the six public failure modes. **Flauz claim (KF + PR §5.10):**
all six controlled **as acceptance tests**: native paths; bounded
app-server pages + `useStateDbOnly: true`; 300 ms Git debounce; one
supervised tree + Windows Job Object; narrowly scoped owned logging.
Recovery cell: "controls are standing acceptance tests".

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| SFR-C1 | Multi-root white screen control = standing test (cross-ref MRW; separate evidence required) | 26.721 `[historical-record]` KF row 1 | unit-test | as MRW-C2 (independently cited — no evidence reuse across rows) | per MRW-C2 |
| SFR-C2 | 594 MB JSONL line: bounded reads only; direct live-JSONL reads forbidden; app-server pages bounded | 26.721 `[historical-record]` KF row 2 | source-read | the history/JSONL readers: bounded page sizes, no unbounded `read_to_string` (also an AGENTS.md invariant) | all live readers bounded; any unbounded read = defect-found |
| SFR-C3 | ~9 GB startup history scan: `thread/list` paginated and always sets `useStateDbOnly: true` | 26.721 `[historical-record]` KF row 3 | source-read | the `thread/list` request construction | pagination + the flag on every call site |
| SFR-C4 | git.exe storm control = standing test (cross-ref GPH) | 26.721 `[historical-record]` KF row 4 | unit-test | as GPH-C3 (independently cited) | per GPH-C3 |
| SFR-C5 | Process-cleanup storm: one supervised tree, graceful shutdown, one bounded fallback; Windows Job Object | 26.721 `[historical-record]` KF row 5 | source-read (+ unit-test where present) | `crates/codex-platform/src/process.rs`: supervision + the Windows Job Object arm; per AGENTS.md, no polling `taskkill` loops | supervision + Job Object present; the Job Object arm must be read in the `#[cfg(windows)]` code (compiled by the windows-latest matrix) |
| SFR-C6 | Unbounded logging: owned state narrowly scoped; no provider log duplication | 26.721 `[historical-record]` KF row 6 | source-read | owned logging/storage scope | no unbounded log writes on owned paths |
| SFR-C7 | The whole set is standing: CI green on **both matrices** at the review SHA | Flauz-side standing claim (Recovery cell) | unit-test (CI citation) | cite the latest full CI run on the review base (both `windows-latest` + `ubuntu-24.04`) | both matrices green at the cited SHA; a single-matrix citation = not-run with reason |

**Attack surface (C):** SFR-C5 "verified" by reading only the Unix arm;
controls verified as code but the standing-test claim (the Recovery cell)
never checked against CI at the review SHA; per-control evidence reused
across SFR/MRW/GPH rows without independent citation.

### 4.8 Side chats — PR §5.1 (added by C2 audit), status `complete` (WO-P2-006, PR #18 → `5287c29f3f`)

**Official reference (26.707 note + current docs; `[historical-record +
docs-derived]` A §1 — confidence medium-high):** `Open side chat`
Ctrl/Cmd+Alt+S opens a temporary side conversation without interrupting
the main chat; `/side` appears in the current command set. **Flauz claim
(PR §5.1):** Ctrl+Alt+S (Cmd+Alt+S on macOS) opens the side panel with the
main chat still selected; the side composer submits into the side thread
without selecting it; `/side` (availability-guarded, menu row) reopens
the panel; close dismisses it with the main view intact; the main chat's
selection and active turn are untouched. Evidence: WO-P2-006 D9/D9b
(VLM-read).

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| SCH-C1 | Registry metadata: `openSideChat`, title `Open side chat`, description `Start a temporary side conversation without leaving this chat`, Thread group, default `CmdOrCtrl+Alt+S`; single owner of the accelerator | 26.721-era `[historical-record + docs-derived]` A §1; Flauz registry | source-read + unit-test | src `ui.rs:2826-2831` (re-verified, Appendix A); the owners test at src `ui.rs:48157` (`owners == ["openSideChat"]`) | exact metadata + exact-one-owner test green |
| SCH-C2 | Ctrl+Alt+S opens the side panel with the main chat still selected | baseline claim; GUI `[runtime-observed D9]` | GUI scene (EQ-1 dependent) | D9-pattern scene (`d9-wo-p2-006-side-chats-verify.sh` steps 1-3): create/select a main chat, press ctrl+alt+s, capture; VLM-read: side panel visible + main still selected (D9 verbatim: side panel text "Ask a side question without interrupting this chat.") | frames + VLM read confirm both facts; if EQ-1 = no runtime, fall back to source + the WO-P2-006 state test + not-run(reason) |
| SCH-C3 | Side submit creates the side thread **without selecting it** (new sidebar item appears above; main stays highlighted) | baseline claim; D9 006/007 | GUI scene (EQ-1) | D9 steps 4-6 + aftermath frame; VLM verbatim from `vlmd9a.json` ("a new item 'quick aside question' appears above it … remains highlighted/selected") | VLM read confirms non-selection semantics |
| SCH-C4 | `/side` is availability-guarded: with no task → honest message fallback (never a silent swallow); menu row hidden without a task | Flauz claim + WO-P2-007 input-quality doctrine | source-read + unit-test | executor arm + guard (SWEEP §D at base: `ui.rs:7764-7775`, menu row `23499-23501` — re-locate by token); confirm the guard reports, not silently returns | guard message path present; a silent `return true` on the unavailable path = defect-found (the F-D1 pattern) |
| SCH-C5 | Close returns to the main view; main chat selection and active turn untouched | baseline claim; WO-P2-006 state test | unit-test + GUI scene (EQ-1) | the WO-P2-006 state test (selection/active-turn untouched) + D9 steps 7 (close) frames | state test green + frames show main intact |
| SCH-C6 | Registry count integrity at the review base: `openSideChat` in `KEYBOARD_SHORTCUT_COMMAND_IDS`; `MAX_KEYBOARD_SHORTCUT_COMMANDS` consistent with the registry length (76 at `d479c7b`) | Flauz-side integrity (supersedes the SWEEP 71-vs-72 quirk per PR §9 override 18) | source-read + unit-test | src `crates/codex-core/src/lib.rs:132` (re-verified = 76) + count the registry ids; run any registry-count test | constant == registry length; mismatch = defect-found (the truncation quirk class) |

**Attack surface (C):** "main chat untouched" claimed without the state
test or without frames; byte-identical frames presented as "panel opened"
(md5 must differ — the D11b/D10 pattern); `/side` verified only via the
menu row (whose hidden guard is honest) and never via the typed path;
SCH-C6 answered with the stale 71/72 numbers instead of recounting at the
review base.

### 4.9 Projects and chats — PR §5.1, status `partial`

**Official reference:** grouping, bounded full-text chat search,
archive/unarchive/delete, rename, pinning `[historical-record A §1]`;
multi-folder local projects in-baseline 26.715 — `Edit project` adds
related folders + primary choice; new chats, Git, AGENTS.md/skills/
config.toml discovery use the primary folder; secondary folders for file
search/read/edit `[docs-derived A §6]`; unified pinned threads + shared
thread snapshots (2026-08-20) are cloud-side `[docs-derived A §9]` —
**documented deferral**. **Flauz claim (PR §5.1):** bounded search with
stable snippets + pagination, command menu, archive/delete/rename,
codexRS-owned bounded pinning `[historical-record PM]`; sidebar Chats list
+ Ctrl+G palette entry render `[runtime-observed ev/06, ev/08]`;
multi-folder model on main (WO-P1-003, `d06ae3b`): `LocalProjectSummary.
folders` + Edit project surface + primary-swap re-key + related folders
in file search; single-path legacy projects load primary-only
`[source-derived]`.

The row is `partial` by design: B verifies the **claimed slice**; the
unclaimed remainder (cloud sync — deferred; richer metadata — P3
residual) is out of scope and must not be counted against parity.

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| PRJ-C1 | Bounded full-text chat search: stable snippets + pagination; bounded page sizes; `useStateDbOnly` on the live path | 26.721 `[historical-record]` PM | source-read + unit-test | the thread-search request path + its bounds tests | bounded + paginated; unbounded query = defect-found |
| PRJ-C2 | Archive/delete/rename/pinning vertical slice with honest guards (palette ArchiveChat requires a selected chat) | 26.721 `[historical-record]` PM | source-read + unit-test | the archive/rename/pin actions + palette guards (SWEEP §C at base: guards `3706/3707`) | guards honest (no silent states beyond documented F-A1/A2/A3); tests green |
| PRJ-C3 | Multi-folder model at the review base: `LocalProjectSummary.folders` (cap 16, primary never a member), Add/Remove/SetPrimary actions with honest guards, primary-swap re-keys the registry row (identity + manual order preserved; old primary parks at the front of related) | 26.715 in-baseline `[docs-derived A §6]`; Flauz delivery WO-P1-003 | source-read + unit-test | src `crates/codex-core/src/lib.rs:914-927` (`folders: Vec<PathBuf>`), `normalize_local_project_folders` at `lib.rs:7483` (re-verified, Appendix A); the WO-P1-003 workspace test suite | model + cap + re-key semantics verified in source AND the focused tests green (CI citation at the review SHA) |
| PRJ-C4 | Edit project surface GUI: multi-folder project listed + primary-cwd indicator; palette file search hits the RELATED folder; Make-primary swap with surface-follow + identity preserved; cwd follows the new primary; swapped state persists across close/reopen | 26.715 `[docs-derived A §6]`; WO-P1-003 D6 evidence | GUI scene | re-run the seeded scene (`seed-wo-p1-003.py` + `scene-d6-v5.sh` pattern) — seeding is the documented lab device for the native picker; VLM-read the six D6 assertions | six assertions each evidenced by frame + VLM read (or not-run with reason if the lab cannot run the binary) |
| PRJ-C5 | Discovery contract: cwd/Git/AGENTS.md/skills/config.toml stay **primary-only**; related folders join file search/read/edit after the primary | 26.715 `[docs-derived A §6]` | source-read + unit-test | the discovery call sites + WO-P1-003 tests | contract holds at every call site; a related-folder leak into cwd/Git/config discovery = defect-found |
| PRJ-C6 | Legacy single-path projects load primary-only; storage schema v4 with v3→v4 migration | Flauz delivery claim | source-read + unit-test | storage schema + migration tests | migration present + green |
| PRJ-C7 | Sidebar Chats list + Ctrl+G palette entry render | `[runtime-observed]` ev/06, ev/08 | GUI scene | entry-surface scene: Ctrl+G capture; sidebar render | frames + VLM read |

**Attack surface (C):** multi-folder verified from unit tests alone (the
WO contract demanded GUI evidence — D6 exists; the honest step at the
review base is a re-run or an explicit not-run); claiming the add-folder
**native picker** was GUI-verified (it cannot render under Xvfb — the
seed-script workaround is the documented device); conflating this row
with 26.727 multi-repo review (different capability, open); counting the
cloud-side deferral against the row.

### 4.10 Settings shell — PR §5.9, status `partial`

**Official reference:** 274 px shell: `Back to app`, bounded search
(Ctrl/Cmd+F), Personal/Integrations/Coding/Archived groups, filtering,
no-results state; full stable registry of 26 sections `[historical-record
A §9]`; 2026-08-11: Settings > Import added `[docs-derived]`; Linux
preview runtime (WO-LAB-001): the Ctrl+K palette's dynamic Settings group
lists 10 pages at the login surface `[runtime-observed: linux-preview
26.908.70816; codex-linux/03]` — **ref+1, version-skew labeled (Pets is
26.908-only)**. **Flauz claim (PR §5.9):** stable-shaped shell; nav
renders 15 rows in 3 groups; `SettingsSection` enum has 18 sections
(CodeReview/Worktrees/ArchivedChats contextual/hidden) `[source-derived;
runtime-observed ev/07]`; settings search filters nav correctly
("import" → Personal + Import) `[runtime-observed ev/23]`; remaining
sections only with working host contracts (ledger bounded).

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| SET-C1 | Shell shape: `Back to app` affordance, bounded settings search, the group structure, a no-results state | 26.721 `[historical-record]` A §9 | GUI scene + source-read | settings scene (open via Ctrl+,); capture the shell; src: "Back to app" at `ui.rs:32190` (re-verified, Appendix A) + the search field + no-results copy | all four shell elements evidenced |
| SET-C2 | Nav renders 15 rows in 3 groups; `SettingsSection` has 18 variants with CodeReview/Worktrees/ArchivedChats contextual/hidden | 26.721 `[historical-record]` A §9 (stable registry = 26 sections — **bound, not bar**, see C6) | source-read + GUI scene | src enum at `ui.rs:2494-2513` (re-verified: 18 variants) + `DEFAULT_NAV_SECTIONS: [Self; 15]` at `ui.rs:2522-2538`; one settings frame | counts match source; frame shows the 3-group nav |
| SET-C3 | Settings search filters the nav correctly ("import" → Personal + Import) | 26.721 `[historical-record]` A §9; `[runtime-observed]` ev/23 | GUI scene | type "import" in the settings search; capture | filtered nav shows exactly the Personal group + Import row |
| SET-C4 | Palette settings-page indexing (WO-P2-004 slice): every default-nav section reachable from the palette; the six added entries (Profile, Import, Browser, Configuration, Hooks, Git) resolve by query and navigate to their pages unauthenticated; Personalization palette title aligned with its nav label | official dynamic indexing is LX 03-04 (ref+1 naming); Flauz slice per WO-P2-004 | GUI scene + unit-test | the d7/d7b scene pattern (`scene-d7.sh`): Ctrl+K, query each of the six names, Return, capture the page; run the registry coverage + filtering tests | each query resolves + navigates (frames + VLM); tests green |
| SET-C5 | Host-contract honesty: sections not backed by a working host render honest empty/guard states (e.g. Hooks "No hooks found"), never fake content | Flauz-side honesty claim (ledger bounded) | source-read + GUI scene (Hooks page capture from C4) | the render arms for the bounded sections | no fabricated content; an empty state presented as data = defect-found |
| SET-C6 | Residual honesty: the 18-vs-26 gap stays a documented bound (host contracts deferred); the official 10-page palette list is cited ref+1 where used; Pets never counted as target | PR §5.9 Gap cell; LX version-skew rule | documentation check | re-read PR §5.9 + the ledger entry at the review base | the bound wording stands; any criterion or report text citing LX without the ref+1 label = C flags it |

**Attack surface (C):** treating Flauz's 18-variant enum as parity with the
official 26-section registry without the host-contract bound; using the
26.908 10-page list as the 26.825 bar (Pets is 26.908-only); SET-C3
verified only with a query that hits the first group (use "import" — it
crosses groups); palette-indexing claimed from WO-P2-004's archived
evidence without re-running at the review base.

### 4.11 Keyboard and accessibility — PR §5.10, status `partial`

**Official reference (26.721 `[historical-record]` A §10):** Ctrl/Cmd+K,
Shift+P, G, P search-files drill-in, arrows/Enter/Escape; dynamic Settings
group in palette; editable shortcut registry with stable grouping;
complete focus order, screen-reader labels. Current docs add Clear
terminal (Ctrl+L/K), font-size (Ctrl+±/0), Toggle file tree
(Ctrl+Shift+E) — post-baseline, intro dates `[unverified]`
`[docs-derived]`. Linux preview 26.908 (ref+1): Ctrl+/ overlay with 22
rows, runtime-confirming the file-tree binding and surfacing
Flauz-absent bindings (PR §9 override 16). **Flauz claim (PR §5.10):**
native command palette with verified stable registry subset (51-command
palette registry at WO-P2-004; every default-nav settings section
indexed); Ctrl+/ overlay + editable settings page `[runtime-observed
ev/11, ev/15]`; Ctrl+P routes through the `searchFiles` interceptor
(WO-P2-007) with the F-A4 no-workspace residual documented; pending:
remaining stable commands, complete focus order, screen-reader labels,
reduced motion.

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| KAX-C1 | Palette mechanics: Ctrl+K / Ctrl+Shift+P / Ctrl+G open the palette; Ctrl+P (workspace-seeded) opens the Search-files palette; Escape closes byte-identical (D10b pattern) | 26.721 `[historical-record]` A §10 + PM:112 | GUI scene | D10b scene pattern (`wo-p2-007/d10b/`): seed a workspace via the StorageOpened restore path; Ctrl+P capture; Escape capture + md5 | palette opens with the "Search files" placeholder; Escape close byte-identical (md5) |
| KAX-C2 | Registry integrity at the review base: registry↔`ACTIVE_KEYBOARD_SHORTCUTS` set-equality; zero dead command ids; `MAX_KEYBOARD_SHORTCUT_COMMANDS` == registry length | Flauz-side integrity (SWEEP §B verdict; override 18 supersession) | source-read + unit-test | re-run the SWEEP battery greps at the review base (SWEEP §"VERIFICATION BATTERY") + read `lib.rs:132` (re-verified = 76) | set-equal; constant == length; any dead id = defect-found |
| KAX-C3 | WO-P2-007 fix intact at the review base: no `OpenFileSearch` action/binding anywhere; `ctrl_p_routes_to_the_search_files_command` test present and green | Flauz delivery claim (PR #20 → `a3c0e01`) | source-read + unit-test | grep for `OpenFileSearch` (expect zero hits); run/cite the regression test | zero hits + test green; the action reappearing = defect-found (regression) |
| KAX-C4 | F-A4 residual documented and unchanged in behavior: with no workspace, Ctrl+P stays silent (the Files palette early-return) — a documented residual, **not a defect**, unless undocumented or changed without record | SWEEP finding-of-record + WO-P2-007 known limitations | source-read + documentation check | the `open_command_palette` Files-mode guard; PR §9 override 17 FIXED note stands | guard present + documented; if behavior changed (e.g. now honest guidance), the docs must have moved with it — a silent change = finding |
| KAX-C5 | Claimed subset works end-to-end: the palette rows for the stable registry subset dispatch to real surfaces (no dead palette entries; the one SWEEP PARTIAL — CommitOrPush pending-PR silent skip F-A6 — stays documented) | 26.721 `[historical-record]` PM:112 | source-read (sweep re-run) + spot GUI | re-run the SWEEP §C verification (grep battery) at the review base; spot-check 2-3 palette rows in the scene from KAX-C1 | no new dead entries; F-A6 either fixed or still documented |
| KAX-C6 | Pending items stay honestly pending: remaining stable commands, complete focus order, screen-reader labels, reduced motion are NOT claimed complete anywhere in the row/report at the review base | PR §5.10 Gap cell | documentation check | re-read PR §5.10 at the review base | pending list intact; any quiet completion claim without new evidence = C attacks it |
| KAX-C7 | Palette Settings group entries at the review base: the settings entries render with title/description/icon mirroring the nav rows; `Personalization` title aligned | Flauz slice (WO-P2-004) | GUI scene | one palette frame (from KAX-C1's scene) + the registry source | entries visible in the Settings group; alignment holds |

**Attack surface (C):** the 22-row 26.908 overlay cited as the 26.825
inventory (the single biggest version-skew trap — REF §R3.7: "must not
cite the overlay rows as the 26.825 bar without the version-skew label");
the row reported as "complete" because WO-P2-004/007 closed (it is partial
by design — pending a11y); the context-scoped chord model difference
(official Ctrl+L / Ctrl+Shift+C are context-scoped; Flauz binds globally —
REF §R3.9) claimed as parity; KAX-C5 answered from the archived sweep
without re-running the greps at the review base.

### 4.12 Notifications and tray — PR §5.10, status `partial`

**Official reference (26.721 `[historical-record]` A §10):** background
completion notification (banner + Windows quiet-hours toast / Linux
freedesktop best-effort); tray tooltip count. Official Linux preview
notifications `[unverified]` (WO-LAB-001 bound). **Flauz claim (PR
§5.10):** bounded in-app banner (Open/Dismiss) + matching
window-title/notification-area tooltip; gh-missing toast banners observed
on multiple pages `[runtime-observed ev/03/04]`; pending tray groups,
badges, sounds (ledger enhancement); Linux tray/global shortcuts
unavailable (packaging row).

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| NTR-C1 | Background completion shows the bounded in-app banner with Open/Dismiss | 26.721 `[historical-record]` A §10 | source-read + unit-test (+ GUI only if EQ-1 allows) | the banner path (completion → banner state → Open/Dismiss actions); GUI needs a completing background turn (EQ-1) | banner path + actions verified in source; GUI either evidenced or not-run(EQ-1) |
| NTR-C2 | Window-title/notification-area tooltip reports the bounded count of running/awaiting-approval non-selected chats and returns to `codexRS` when none remain | 26.721 `[historical-record]` PM:104 | source-read + unit-test | the title/tooltip count derivation + its bounds | count logic + restore-to-base verified; tests green |
| NTR-C3 | Linux freedesktop best-effort notification path exists without blocking the UI | 26.721 `[historical-record]` PM:104 | source-read | `crates/codex-platform/src/desktop_notifications.rs` | best-effort, non-blocking, freedesktop-shaped |
| NTR-C4 | Pending set stays honest: tray groups/badges/sounds not claimed; Linux tray unavailability documented (packaging row cross-ref) | PR §5.10 Gap cell | documentation check | re-read the row + packaging row | bounds stand |
| NTR-C5 | gh-missing toast banners render on the affected pages (re-verification of ev/03/04) | `[runtime-observed]` ev/03/04 | GUI scene | unauthenticated scene: open the PR/Plugins surface that triggers the gh-required banner | banner visible; frame + VLM read |

**Attack surface (C):** claiming the Windows quiet-hours toast verified
(any Windows GUI evidence — impossible in this lab — is fabricated;
windows cells are historical-record only); tray parity claimed while
Linux tray is documented unavailable; NTR-C1 "GUI-verified" with a
banner triggered by something other than background completion (the
claim is completion-specific — gh-missing banners are a different
trigger, that is NTR-C5).

### 4.13 Activity view & unread attention — PR §5.10 (added by C2 audit), status `partial` (WO-P2-008, PR #23 → `00a3392`)

**Official reference (26.727 → in 26.825 target; `[historical-record +
docs-derived]` A §10 + WO-R-REF R1):** bell icon / Ctrl/Cmd+Alt+U shows
recently engaged chats needing attention; Shift+Esc clears unread
indicators; Ctrl+Alt+A next chat needing attention; per-chat mark-unread
Ctrl+Shift+U (post-baseline docs row); view shape/empty states/clearing
semantics beyond Shift+Esc `[unverified]` (auth-walled). **Flauz claim
(PR §5.10):** bounded `needs_attention_task_ids` (capped
`MAX_VISIBLE_THREADS`, not persisted) flagged on background turn
completion (failed turns included) + approval requests in non-selected
chats; visit clears, archive drops; sidebar 6px dot + medium-weight title;
four registry bindings with honest statuses; `MAX_KEYBOARD_SHORTCUT_
COMMANDS` 71→76; 7 core state tests + 6 app binding tests; D11/D11b
evidence with the honest no-runtime limitation documented.

| ID | Criterion | Ref | Evidence class | Cheapest honest step | Pass condition |
| --- | --- | --- | --- | --- | --- |
| ACT-C1 | State machine: background turn completion (incl. failed turns) + approval requests flag non-selected chats; visit clears; archive drops; cap = `MAX_VISIBLE_THREADS` with FIFO drop; not persisted (session state) | 26.727→26.825 `[docs-derived]` (R1 §4-5, with the unverified-clearing caveat); Flauz claim per WO-P2-008 | unit-test + source-read | run the 7 codex-core state tests at the review base (per-test output — the WO-P2-008 house pattern); source: `needs_attention_task_ids` at `crates/codex-core/src/lib.rs:4757/8882-8891/10533/10654` (re-verified, Appendix A) | all 7 tests green with per-test output recorded; source semantics match the claim |
| ACT-C2 | Four registry bindings with exact metadata: `toggleThreadUnread` Ctrl/Cmd+Shift+U ("Mark chat unread", Thread); `nextUnreadChat` Ctrl/Cmd+Alt+A ("Next chat needing attention", Navigation); `clearAllUnread` Shift+Escape ("Clear all unread indicators", Navigation); `toggleActivityView` Ctrl/Cmd+Alt+U ("Toggle Activity view", Navigation) | 26.727→26.825 `[docs-derived]` (R1 shortcuts table; CL verbatim "Cmd/Ctrl+Opt+U") | source-read + unit-test | src `ui.rs:2847-2852` + `2924-2943` (re-verified, Appendix A); run the 6 app binding tests (CI authoritative — local test-mode gpui compile is excluded by the OOM discipline per the WO-P2-008 precedent) | exact ids/titles/defaults/groups; exact-one-owner accelerators; tests green on CI |
| ACT-C3 | Honest statuses on the real binary (D11b verbatim): `Select a chat before marking it unread.` / `No chats need attention.` / `Activity view is not available yet. Use "Next chat needing attention" to jump to unread chats.` / `No unread chats` — all four bindings resolve visibly, never silent | Flauz delivery claim (WO-P2-007 input-quality doctrine) | GUI scene | re-run `d11b-empty-surface.sh` at the review base: four keypresses on the entry surface; frames + md5 sequence + VLM reads | all four frames differ; VLM reads match the verbatim statuses |
| ACT-C4 | `nextUnreadChat` jump semantics: cyclic sidebar-order jump; the selected chat is never a candidate; honest empty status | Flauz delivery claim (R1 §4 gives no official ordering — chosen interpretation, honestly labeled) | unit-test (+ GUI only if EQ-1) | the jump tests (CI); GUI needs ≥2 flagged threads (EQ-1) | jump tests green; GUI either evidenced or not-run(EQ-1) — the D11 precedent documents this exact bound |
| ACT-C5 | Sidebar dot + medium-weight title render for flagged chats | official visual treatment `[unverified]` (R1 §4); Flauz's dot = chosen interpretation | source-read + unit-test (+ GUI only if EQ-1) | the sidebar render path (src `ui.rs:8628-8647` area, re-verified token `needs_attention`); GUI dot-on-row needs a runtime-enabled lab (the documented D11 residual — unit-covered) | render path verified in source + the state test coverage named; GUI not-run(EQ-1) is acceptable **only** with the documented-residual citation |
| ACT-C6 | Activity view surface: absent by design (separate future WO; binding gives honest guidance). B must NOT count its absence as a defect and must NOT claim the surface exists; the unread-persistence-across-restart reference behavior stays `[unverified]` (session state by design) | PR §5.10 residuals; R1 open questions 1-3 | documentation check + GUI (C3's fourth status) | re-read PR §5.10 residual wording + the D11b fourth binding | the residual wording stands; the guidance status renders (covered by ACT-C3) |

**Attack surface (C):** dot-on-row or jump-between-chats claimed as
**GUI-observed** without a runtime-enabled lab (the D11 documented limit
— fabricating it is automatic failure); the state machine verified only
via the D11b binding statuses (those prove the honest paths, not the
flagging paths — C1's tests are the flagging evidence); the missing
Activity view surface counted against parity (documented residual, future
WO); version mislabeling in either direction (26.727 is in-target — not
baseline-missing, not ref+1).

## 5. Criteria summary

| Row | Status | Criteria | GUI-scene dependent on EQ-1 | Unit-test anchored | Source-only |
| --- | --- | --- | --- | --- | --- |
| Runtime bootstrap | complete | 5 | C4, C5 | C3 | C1, C2 |
| Multi-root workspace handling | complete (control) | 3 | — | C2 | C1, C3 |
| Git process hygiene | complete (control) | 4 | — | C3 | C1, C2, C4 |
| Marketplace admin-disabled install | complete | 4 | C4 (or not-run) | C4 | C1, C2, C3 |
| Keyboard shortcut reference | complete | 5 | C1, C2, C4 | C2 | C2, C3, C5 |
| Feedback | complete | 6 | C5 | C2, C6 | C1, C2, C3, C4, C6 |
| Stable-failure regression controls | complete (control) | 7 | — | C1-C5, C7 | C2, C3, C5, C6 |
| Side chats | complete | 6 | C2, C3, C5 | C1, C5, C6 | C1, C4, C6 |
| Projects and chats | partial | 7 | C4, C7 | C1-C3, C5, C6 | C1-C3, C5, C6 |
| Settings shell | partial | 6 | C1-C5 | C2, C4 | C1, C2, C5, C6 |
| Keyboard and accessibility | partial | 7 | C1, C7 | C2, C3 | C2-C6 |
| Notifications and tray | partial | 5 | C5 (+C1 if EQ-1) | C1, C2 | C1-C4 |
| Activity view & unread attention | partial | 6 | C3 | C1, C2, C4, C5 | C1, C2, C5, C6 |
| **Total** | | **71** | | | |

## 6. Follow-up WO candidates surfaced during rubric derivation (advisory — Lead decides)

1. **Plugins palette indexing** — `SettingsSection::Plugins` has no
   palette entry; `OpenPlugins` routes to Marketplace while the
   `DEFAULT_NAV_SECTIONS` doc comment claims full indexing (SWEEP
   secondary finding; still open at `d479c7b` — the doc comment at
   `ui.rs:2516-2520` makes the same claim). A small catalog order in the
   WO-P2-004 pattern, or a doc-comment fix.
2. **Advertised-while-disabled overlay rows** — the SWEEP §F honesty
   findings (16 guard-gated rows advertised; F-A1/A2/A3 state-dependent
   silent rows) are recorded but carry no WO; a bounded honesty order
   (guidance vs silent fall-through) would close them.
3. **F-A4 no-workspace Ctrl+P parity treatment** — already flagged by
   WO-P2-007 as follow-up material; needs a Lead decision on the official
   no-workspace behavior (officially `[unverified]`).
4. **Authenticated official-app reference run** — the standing upgrade
   path for every `[unverified]` official shape this rubric hit (R1
   open questions 1-5, R2 open questions 1-5, R3 open questions 1-5):
   an authenticated run of the operator's 26.825.51511 machine (or the
   Linux preview with credentials) with palette/bell/address-bar dumps.
5. **EQ-1 lab runtime provisioning** — a lab-infrastructure mini-order
   (WO-LAB class) to make a supervised `codex` runtime available (or
   explicitly permanently unavailable) to LINUX_GUI_LAB, so
   thread-dependent GUI criteria stop depending on which day the scene
   runs (the D9 vs D11 discrepancy recorded in §3).

## Appendix A — anchors re-verified at HEAD `d479c7b` during derivation

All verified by direct read during RWO-020 (this is rubric grounding, not
Flauz verification):

- `crates/codex-core/src/lib.rs:132` — `MAX_KEYBOARD_SHORTCUT_COMMANDS: usize = 76`
- `crates/codex-core/src/lib.rs:137-214` — `KEYBOARD_SHORTCUT_COMMAND_IDS` (ends `toggleFullScreen`)
- `crates/codex-core/src/lib.rs:227-234` — `STABLE_REFERENCE` (`26.721.3996.0`, CLI `0.146.0-alpha.3.1`, SHA-256 `39e9e041…`)
- `crates/codex-core/src/lib.rs:4757, 4832, 8882-8891, 10533, 10654` — `needs_attention_task_ids` state + cap
- `crates/codex-core/src/lib.rs:4358-4374` — `FeedbackClassification` (5 variants, `ALL: [Self; 5]`; `as_str` literals `bug`/`bad-result`/`good-result`/… — B reads the remainder)
- `crates/codex-core/src/lib.rs:914-927` — `LocalProjectSummary` + `folders: Vec<PathBuf>`; `:7483` `normalize_local_project_folders`
- `crates/codex-platform/src/lib.rs:261-281+` — `resolve_codex_binary` override order (explicit → `CODEX_RS_CODEX_BIN` → win stable cache → win npm → …)
- `crates/codex-app/src/ui.rs:2494-2513` — `enum SettingsSection` (18 variants); `:2522-2538` `DEFAULT_NAV_SECTIONS: [Self; 15]`
- `crates/codex-app/src/ui.rs:2826-2831` — `openSideChat` registry item (Thread group, `CmdOrCtrl+Alt+S`)
- `crates/codex-app/src/ui.rs:2847-2852` — `toggleThreadUnread` (`CmdOrCtrl+Shift+U`)
- `crates/codex-app/src/ui.rs:2924-2943` — `toggleActivityView` / `nextUnreadChat` / `clearAllUnread`
- `crates/codex-app/src/ui.rs:9754-9770` — `open_feedback_modal` (`feedback_include_logs = true` at 9763)
- `crates/codex-app/src/ui.rs:10499-10523` — interceptor arms for openSideChat/toggleActivityView/nextUnreadChat/clearAllUnread/toggleThreadUnread
- `crates/codex-app/src/ui.rs:31885` — `"Access is turned off by your admin"` tooltip
- `crates/codex-app/src/ui.rs:32190` — `"Back to app"` settings-shell affordance
- `crates/codex-app/src/ui.rs:41722-41828` — feedback classification UI + validation gate
- `crates/codex-app/src/ui.rs:48157-48159` — side-chat exact-one-owner test anchors
- `crates/codex-app/src/ui.rs:1548-1553, 8628-8647` — sidebar `needs_attention` render path
- `docs/known-failures.md` — six stable-failure rows + active limitations + budgets
- `docs/parity-matrix.md:75, 93, 103, 104, 112, 113` — the PM rows cited above

Line anchors inherited from SWEEP (base `3c9f113`) or older evidence are
cited by **token** in §4; B re-locates them at the review base (the
WO-P2-007/008 commits shifted `ui.rs` by net additions; the sweep itself
documents −2 drift for its own base).

## Appendix B — derivation trace (what RWO-020 read to build this)

- `AGENTS.md`; `docs/WORK-ORDER-TEMPLATE.md` (closure gates — the rubric's
  five-gate shape mirrors them)
- `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` — §2-§10 in full
  (matrix rows quoted at §5.1/§5.6-§5.10; counts at §8.1; overrides at §9,
  esp. 4/9/14-18; changelog §10)
- `docs/research/FEATURE-PARITY-WORK-ORDERS.md` — rules, lifecycle, all
  WO entries incl. WO-P1-001/002/003, WO-P2-004..010, WO-PLAT-001,
  WO-LAB-001, WO-R-REF, WO-R-SWEEP; §5 no-WO gap decisions
- `docs/research/CODEX-REFERENCE-MATRIX.md` §10 (official product-shell
  rows) + §11 (runtime surface)
- `docs/parity-matrix.md` (PM rows for bootstrap/feedback/shortcuts/
  notifications/marketplace); `docs/known-failures.md` (KF table)
- Evidence: `codex-linux/README.md` (+ capture list), `codex-linux/
  vlm-reads.txt` (referenced), `p2-batch2-reference/README.md` (R1-R3 in
  full), `wo-p2-007/input-surface-sweep.md` (in full), `wo-p2-008/
  README.md`, `wo-p2-004/README.md`, `wo-p1-003/README.md`,
  `wo-p2-006/d9-wo-p2-006-side-chats-verify.sh`, `wo-p2-006/vlmd9a.json`,
  `codex-ref/version-delta-notes.md`
- Source spot-reads at `d479c7b` (Appendix A)

**Bounds of this rubric:** derived from the repo's own evidence layers +
read-only source reads. No cargo, no GUI scenes, no official-app run were
executed by RWO-020 (rubric production only — B executes). The 26.825
official side remains evidence-layered per VDN §E.1; every place the
official shape is `[unverified]` is marked as such and carries its
documented upgrade path.
