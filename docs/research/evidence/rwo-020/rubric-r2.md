# RWO-020 — Reference-behavior review rubric, second dispatch (WO-REVIEW-001, deep phase)

- **Task:** RWO-020 — build the checkable verification rubric for the
  parity-matrix rows claiming non-missing status, from the repository's own
  reference evidence.
- **Author:** Review worker (RWO-020 re-dispatch), 2026-09-20 (UTC).
- **Lineage note (duplicate dispatch):** this branch already carried the
  first dispatch's deliverable (`rubric.md` + `notes.md`, commit `734b1e7`,
  2026-09-19). Per the WO-P2-008 duplicate-delivery precedent (PR §10,
  2026-09-19: both pushes preserved, the verified lineage adjudicated by
  the Lead), the first dispatch's files are preserved verbatim and this
  second dispatch ships as `rubric-r2.md`. The reconciliation record is
  appended to `notes.md` (§N8). The Lead adjudicates which lineage B
  executes (or both — they are complementary: `rubric.md` covers 13 rows /
  71 criteria incl. the three `complete` rows this file flags in Appendix
  E.5; this file adds the verified-anchor table, verbatim-status criteria,
  the NT-2 evidence-class trap, and B's result-capture template).
- **Branch:** `research/rwo-020`, based on `main` @
  `d479c7b9e725e1af1353140d49226b3f9a16be9d` (base verified with
  `git rev-parse d479c7b9e725e1af1353140d49226b3f9a16be9d^{commit}`;
  identical base to the first dispatch — no drift).
- **Review target:** `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md`
  (canonical parity matrix; §5 rows, §8 counts, §9 overrides).
- **Consumers:** Worker **B** executes this rubric against the Flauz build;
  Worker **C** attacks the results. RWO-020 itself does **not** verify Flauz
  (per the work-order split).

## 0. Scope

The 10 rows in scope (the 8 `complete` rows plus the highest-traffic
`partial` rows, per the WO-REVIEW-001 deep-phase brief):

| # | Rubric ID | Parity-report row (§) | Claimed status | Version layer of the claim |
|---|-----------|------------------------|----------------|---------------------------|
| 1 | RB | Runtime bootstrap (§5.1) | complete | 26.721 baseline |
| 2 | MW | Multi-root workspace handling (§5.6, regression control) | complete | 26.721 baseline |
| 3 | KS | Keyboard shortcut reference (§5.10) | complete | 26.721 baseline (+26.908 runtime cross-check) |
| 4 | FB | Feedback (§5.10) | complete | 26.721 baseline |
| 5 | SC | Side chats (§5.1, added by C2 audit) | complete (WO-P2-006 CLOSED) | 26.721 baseline + current docs |
| 6 | PC | Projects and chats (§5.1) | partial (multi-folder CLOSED via WO-P1-003) | 26.715/26.721 baseline + 26.825 cloud deferrals |
| 7 | SS | Settings shell (§5.9) | partial | 26.721 baseline (+26.825 Import; 26.908 cross-check) |
| 8 | KA | Keyboard and accessibility (§5.10) | partial (WO-P2-004/007 sub-closures) | 26.721 baseline + current docs + 26.908 cross-check |
| 9 | NT | Notifications and tray (§5.10) | partial | 26.721 baseline |
| 10 | AV | Activity view & unread attention (§5.10, added by C2) | partial (WO-P2-008 state+bindings; view surface pending) | 26.727 → 26.825 current target |

The two remaining `complete` rows (Git process hygiene; Marketplace
admin-disabled install; Stable-failure regression controls — see PR §8.1)
that fall outside the brief's minimum list are covered by MW (the
KF-regression-control class) and the source anchors of Appendix A where
they intersect; they are noted in Appendix E.5 for the Lead's scoping.

Not in scope (documented deferrals — never count against parity, never
fake): Scheduled tasks (incl. 26.825 event triggers), Voice input, Sites
(cloud parts), Visualizations, Cloud environments, Appshots (26.908 Windows
= reference+1), Pets and Codex Micro (26.908 = reference+1),
connector-approval methods, plugin/App OAuth, SSH profiles/remote chats,
Settings host contracts, Computer History (macOS + cloud), unified pins /
shared thread snapshots (cloud). See Appendix C.

## 1. Method

### 1.1 Version tags (parity report §2, §7.7)

- **[26.721]** — historical baseline `Codex Desktop 26.721.3996.0` + bundled
  CLI `0.146.0-alpha.3.1` (pinned in `reference/stable-26.721.3996.0/manifest.json`).
- **[26.825]** — current target `ChatGPT desktop 26.825.51511`
  (operator-installed reference). Current parity target = baseline +
  current delta (`docs/research/evidence/codex-ref/version-delta-notes.md`).
- **[26.908]** — `reference+1` (Linux preview `26.908.70816`, WO-LAB-001).
  Runtime observations from this layer **never upgrade a 26.825 claim**;
  they are forward-looking cross-checks only (Pets line = out of target).

### 1.2 Evidence classes (parity report §4.3)

`[runtime-observed]` (incl. the `linux-preview 26.908.70816` skew form),
`[source-derived]`, `[docs-derived]`, `[historical-record]`, plus the honest
`[unverified]` when every layer is silent. Conflict order (§7.3):
runtime-observed > source-derived > docs-derived > historical-record.

### 1.3 Verification step classes (this rubric)

- **S** — source-read: read the cited file/symbol at the reviewed SHA.
- **U** — unit-test: run the named focused test(s); per-test output.
- **G** — GUI scene: LINUX_GUI_LAB reproduction of the cited scene
  (Xvfb + picom recipe per the sealed `session.sh` pattern; VLM-read frames
  + md5 sequences as the archive format).
- **R** — runtime overlay: observe the official Linux preview app
  (`26.908.70816`, userspace extract per `evidence/codex-linux/`) side by
  side with Flauz — only where the official surface is unauthenticated.

### 1.4 Verdict vocabulary for Worker B (one per criterion)

- `verified` — the criterion held; record the evidence pointer (frame md5 /
  test name / file:line at the reviewed SHA).
- `defect-found` — the criterion failed; record the reproduction and the
  affected claim.
- `not-run` — never silently skipped; record the honest bound (e.g. "needs
  authenticated turn", "WINDOWS_GUI_LAB unavailable", "no codex CLI runtime
  in lab").

Anti-fabrication rules (binding): no mock runtimes (WO-P2-008 D11 doctrine);
no md5/VLM read invented without the frames; a `not-run` with its true
reason is always acceptable and is NOT a review failure.

### 1.5 Attack rules for Worker C (what falsifies a rubric pass)

1. Any criterion recorded `verified` without its evidence pointer.
2. Any result that upgrades a claim beyond its version layer (26.908
   observation used to "prove" a 26.825 behavior, or vice versa).
3. Any deferred capability counted against a row (Appendix C).
4. Any drift between the Flauz claim's own scope and what was tested (e.g.
   testing the gh-missing banner and recording it as the *completion*
   banner — see NT-2).
5. Source anchors that no longer resolve at the reviewed SHA (line drift is
   fine if the symbol resolves; symbol gone = investigate before verdict).

### 1.6 Citation key

- **PR §n** — `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` §n.
- **A §n** — `docs/research/CODEX-REFERENCE-MATRIX.md` §n (Worker A,
  commit `c083c38`).
- **B2 §n** — `docs/research/FLAUZ-REFERENCE-MATRIX.md` §n (Worker B2,
  commit `a2343d3`).
- **PM** — `docs/parity-matrix.md` (historical layer).
- **KF** — `docs/known-failures.md` (failure-table layer).
- **VDN** — `docs/research/evidence/codex-ref/version-delta-notes.md`.
- **LX** — `docs/research/evidence/codex-linux/` (WO-LAB-001 official
  runtime evidence, `26.908.70816`).
- **REF2** — `docs/research/evidence/p2-batch2-reference/README.md`
  (WO-R-REF batch-2 reference research).
- **ev/NN** — `docs/research/evidence/flauz/NN-*.png` (B2 runtime captures).
- **WO evidence** — `docs/research/evidence/<wo-id>/` per closure.

---

## R-01 — Runtime bootstrap (§5.1; claimed complete)

### Official reference behavior

**[26.721]** The official packaged app pins its bundled CLI by hash and
resolves it through an explicit override/fallback order
`[historical-record] PM "Runtime bootstrap"`. Pinned identity:
`OpenAI.Codex 26.721.3996.0` + CLI `0.146.0-alpha.3.1`, SHA-256
`39e9e041ea33ac34aad9578adfe660c5c7a6dc8f82620b77623960f9352a6ef3`
(`reference/stable-26.721.3996.0/manifest.json`; CLI surface reproduced
runtime-observed in VDN §A: `codex --version` → `codex-cli 0.146.0-alpha.3.1`).
Adjacent official bound (not a Flauz gap, PR §9 override 15): the official
Linux app **hard-fails** without its bundled runtime (no window); Flauz's
graceful "Resolving…" degradation is a deliberate Flauz-ahead difference.

### Flauz claim under review (PR §5.1, verbatim scope)

"Exact packaged-CLI hash check, override/fallback order preserved
`[historical-record] PM`; boots to entry surface with app-server online
footer `[runtime-observed] B2 ev/06`."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| RB-1 | `STABLE_REFERENCE` constants equal the pinned manifest: package `OpenAI.Codex`, version `26.721.3996.0`, CLI `0.146.0-alpha.3.1`, `cli_sha256` = `39e9e0…a6ef3` (compare against `reference/stable-26.721.3996.0/manifest.json`). | S | `crates/codex-core/src/lib.rs` `STABLE_REFERENCE` (anchor verified at base: lib.rs:227-234) vs manifest. | 26.721 |
| RB-2 | The CLI resolver order is exactly: explicit argument → `CODEX_RS_CODEX_BIN` env → (Windows) hash-verified stable cache → (Windows) APPDATA npm candidate → bare `codex`/`codex.exe`; the stable-cache arm is *hash-gated* on both the cached copy and any fresh copy from the packaged source. | S | `crates/codex-platform/src/lib.rs` `resolve_codex_binary` (anchor at base: lib.rs:261-283) + `windows_stable_codex_cache_candidate` (`sha256_matches` gate). | 26.721 |
| RB-3 | Booting the real binary (Linux lab, runtime present) reaches the entry surface with the sidebar footer reading **"App-server online"**. | G | Re-run the ev/06 boot scene; VLM-read the footer (source anchor for the string: `crates/codex-app/src/ui.rs:14384` at base). | 26.721 |
| RB-4 | With no runtime resolvable, the app stays up in "Resolving…" auto-retry (degradation claim, distinct from official fatal) — confirms the row's recovery cell. | G | ev/24 reproduction (no-runtime scene; no mock runtime — use the lab's genuine absence). | 26.721 (Flauz-ahead difference) |

### Attack vectors (C)

- Hash constant drifted from the manifest (silent re-pin) → RB-1 fails.
- Reordered resolver (env before explicit) or an un-gated cache path → RB-2 fails.
- ev/06 is B2-era evidence; if B does not re-run RB-3 on the reviewed SHA,
  the "boots to entry surface" half of the claim rests on stale frames —
  record which SHA the GUI evidence belongs to.

---

## R-02 — Multi-root workspace handling (§5.6 regression control; claimed complete)

### Official reference behavior

**[26.721]** The *official* app shipped the multi-root white-screen failure
(public failure report; POSIX path handling reaching Windows drive paths +
missing process cwd) `[historical-record] KF failure table row 1`. The
official "behavior" to mirror is the *absence* of this failure; the row is a
regression control, not a feature (PR §9 override 9 keeps this row separate
from the multi-folder projects capability — do not merge).

### Flauz claim under review (PR §5.6)

"Native `Path`/`PathBuf`, no browser path shim — Windows multi-root white
screen controlled (acceptance test) `[historical-record] KF`."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| MW-1 | Workspace/project/root paths are native `Path`/`PathBuf` end to end; no string-URL conversion shim sits between path storage and filesystem/process use (spot-check the load-bearing route: `LocalProjectSummary { path, folders }` → cwd preparation → `thread/start`). | S | `crates/codex-core/src/lib.rs` `LocalProjectSummary` (anchor at base: lib.rs:914-927, `folders: Vec<PathBuf>`); follow `cwd`/`runtime_workspace_roots` construction. | 26.721 |
| MW-2 | The acceptance test covering the multi-root control exists in-tree and passes (KF status "Implemented" for row 1). | U | Locate the multi-root/path-shim regression test (search the workspace tests for the KF control; run it with per-test output). If no dedicated test exists, record `defect-found` **for the evidence claim** (KF says "acceptance test") with the honest wording — do not invent one. | 26.721 |
| MW-3 | Windows runtime white-screen validation is explicitly bounded: WINDOWS_GUI_LAB unavailable → the row's Linux/CI evidence classes are the only closure evidence (honest labeling, not a defect). | — | Verify the row/ledger wording keeps this bound (PR §3 lab bounds). Record `not-run` for Windows runtime with that reason. | 26.721 |

### Attack vectors (C)

- Any workspace path flowing through `to_string_lossy`/URL normalization
  before filesystem/process use on the load-bearing route.
- MW-2 claimed `verified` with a test name that does not exist or does not
  exercise multi-root path handling.
- The row being "verified" by GUI Linux frames — the control is
  source/test-class evidence; Linux frames neither prove nor disprove the
  Windows failure mode (platform honesty, ledger rule 5).

---

## R-03 — Keyboard shortcut reference (§5.10; claimed complete)

### Official reference behavior

**[26.721]** "Exact stable `Keyboard shortcuts` dialog listing only active
bindings" `[historical-record]` — opens from Help and the effective
Ctrl/Cmd+/ action; stable category order Chat → Navigation → Panels →
Project → Skills → Configure → App → General; platform-aware keycaps;
bounded search over title/description/ID with stable no-results copy;
Escape semantics (first Escape clears a non-empty search, second closes);
`/settings/keyboard-shortcuts` is the editable twin (PM "Active keyboard
shortcut reference"). **[26.908, reference+1 cross-check only]** the
official Linux preview runtime shows the Ctrl+/ overlay as a searchable
"Keyboard shortcuts" dialog ("Search shortcuts" field; Chat/Navigation/
General sections; 22 rows) `[runtime-observed: linux-preview 26.908.70816;
LX 05-07 + vlm-reads.txt READ 7]`. The 22-row inventory is a 26.908
observation — it is a cross-check on dialog shape, NOT the 26.721 parity
bar, and must not be used to fail Flauz rows for missing bindings (that
delta is the §5.10 KA P3 gap).

### Flauz claim under review (PR §5.10)

"Implemented: opens from Help + Ctrl/Cmd+/, stable category order; overlay +
searchable editable settings page both visually confirmed
`[historical-record] PM; runtime-observed ev/11, ev/15`."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| KS-1 | Ctrl+/ is bound to the shortcuts-registry dialog action and is the single owner of that key. | S/U | `crates/codex-app/src/ui.rs` `ShowKeyboardShortcutsShortcut` binding (anchor at base: ui.rs:4733 `KeyBinding::new(&shortcut("/"), …)`); ownership-test pattern as in the openSideChat/searchFiles tests. | 26.721 |
| KS-2 | The dialog lists **only** bindings active in codexRS, in the stable category order (Chat → Navigation → Panels → Project → Skills → Configure → App → General). | G | ev/15-class scene: open overlay, VLM-read section headers in order; flag any inactive/phantom row. | 26.721 |
| KS-3 | Search is bounded and filters; no-results copy renders; Escape: first clears a non-empty query and stays open, second closes. | G | ev/15-extension scene: type a no-match query → no-results copy; Escape ×2. | 26.721 |
| KS-4 | Help menu path opens the same dialog; the editable `/settings/keyboard-shortcuts` page renders and its rows agree with the overlay (same registry, editable). | G | ev/11 reproduction (settings page) + Help-menu click-through. | 26.721 |
| KS-5 | Cross-check only: the Flauz overlay's *shape* (searchable, categorized) is consistent with the official 26.908 runtime overlay (LX 05-07). Differences in row inventory are the known KA P3 gap, not a KS failure. | R (optional) | Side-by-side with LX frames. | 26.908 (cross-check) |

### Attack vectors (C)

- A registry row rendered in the overlay whose action is dead/never-handled
  (WO-R-SWEEP found 24/25 bound-GPUI-actions-declared-but-never-handled at
  base `3c9f113` — check whether any of those surface as overlay rows;
  "listing only active bindings" is the load-bearing word).
- Category order drift or a category added/removed vs the stable comparator.
- KS-5 misused as a pass/fail on row inventory (version-layer violation).

---

## R-04 — Feedback (§5.10; claimed complete)

### Official reference behavior

**[26.721]** "Exact stable `Feedback` command + `/feedback`; five category
IDs; required details; default-on session logs" `[historical-record]`
(PR §5.10). PM elaborates: native `Share feedback` dialog with the five
recovered category IDs, required-details validation, default-on
current-session logs, no browser-tabs control when no browser surface
exists; submission via the pinned typed `feedback/upload` request through
the supervised app-server; bounded details; selected thread ID when
present; app version tag; exact "Feedback uploaded" + retry copy; Help
menu and editable App-group unassigned command route to the same dialog.

### Flauz claim under review (PR §5.10)

"Implemented as native `Share feedback` dialog with the five recovered
category IDs, validation, default-on logs, no browser-tabs control; palette
entry exists `[historical-record] PM; source-derived palette`."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| FB-1 | Exactly five category IDs, exact stable strings: `bug`, `bad-result`, `good-result`, `safety_check`, `other`. | S | `FeedbackClassification` (anchor at base: `crates/codex-core/src/lib.rs:4359-4383`; dialog renders `FeedbackClassification::ALL`, `crates/codex-app/src/ui.rs:41719`). | 26.721 |
| FB-2 | The typed `feedback/upload` request exists on the supervised path with bounded details + thread-ID-when-present + app version tag. | S | `feedback/upload` in `crates/codex-protocol/src/lib.rs` + `crates/codex-platform/src/app_server.rs` (anchors present at base). | 26.721 |
| FB-3 | Three entry points route to the same dialog: palette `Feedback` command, `/feedback` slash command, Help menu `Share feedback`. | G | Scene: open via each of the three; VLM-read identical dialog. (Help anchor `ui.rs:41776` at base; slash row in the executor catalog.) | 26.721 |
| FB-4 | Validation blocks empty required details; session logs default ON; no browser-tabs control renders. | G | Scene: attempt submit with empty details → validation state; read the logs toggle default. | 26.721 |
| FB-5 | Submit uses the typed request and surfaces the exact "Feedback uploaded" / retry copy. | G | Bound: an authenticated app-server is needed for a real upload — if unavailable, record `not-run` (auth bound) and keep FB-5 source-read only (copy strings in-tree). | 26.721 |

### Attack vectors (C)

- A sixth category, a renamed ID, or `SafetyCheck` serialized differently
  than the stable `safety_check` string.
- Logs default OFF, or validation not blocking.
- `/feedback` present in the claim but absent from the slash executor
  (claim says "palette + slash + Help paths").

---

## R-05 — Side chats (§5.1, added by C2 audit; claimed complete, WO-P2-006 CLOSED @ `5287c29f3f`)

### Official reference behavior

**[26.721 + current docs]** "`Open side chat` Ctrl/Cmd+Alt+S; temporary side
conversation without interrupting the main chat; `/side` in current command
set" `[historical-record + docs-derived] A §1` — PR §5.1 flags this
official-side citation as **confidence medium-high** (26.707 "side
conversations" note + current docs). REF2/LX add no runtime observation of
the official side-chat surface (auth-walled; LX README bound 1). Version
treatment: the capability is treated as in-baseline-with-medium-high
confidence + current-docs-confirmed.

### Flauz claim under review (PR §5.1)

"Ctrl+Alt+S (Cmd+Alt+S on macOS) opens the side panel with the main chat
still selected; the side composer submits into the side thread without
selecting it; `/side` (availability-guarded, menu row) reopens the panel;
close dismisses it with the main view intact; the main chat's selection
and active turn are untouched `[source-derived ui.rs + codex-core lib.rs
@764e4db: OpenSideChatShortcut binding + openSideChat interceptor command
+ KEYBOARD_SHORTCUT_COMMAND_IDS 71→72; runtime-observed LINUX_GUI_LAB
D9/D9b]`."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| SC-1 | Alt+S (Cmd+Alt+S on macOS) is bound and owned by exactly one registry command (`openSideChat`); it is a member of `KEYBOARD_SHORTCUT_COMMAND_IDS`. | S/U | Anchors at base: `crates/codex-app/src/ui.rs:4698` (binding), `ui.rs:2675/2826` (`openSideChat`), ownership test at `ui.rs:48157-48159`. | 26.721+current |
| SC-2 | Opening the side panel leaves the main chat selected; the side composer placeholder reads the side-question copy. | G | Re-run `evidence/wo-p2-006/d9-wo-p2-006-side-chats-verify.sh` (VLM D9a read: side panel "Ask a side question without interrupting this chat."; selected sidebar item unchanged). | 26.721+current |
| SC-3 | Submitting in the side composer creates the side thread WITHOUT selecting it (sidebar shows the new item while the main chat stays highlighted). | G | D9 frames 005→007 (VLM read: "quick aside question" appears above; "side chat fixture main" remains highlighted/selected). | 26.721+current |
| SC-4 | `/side` is availability-guarded and reopens the panel; close dismisses the panel with the main view intact (byte-comparable main frame or VLM-read). | G | D9b close-probe (`d9b-close-probe.sh`, click probes) + slash guard at `ui.rs:7822/23628` (anchors at base). | 26.721+current |
| SC-5 | The main chat's active-turn state is untouched by open/submit/close of the side chat. | U | The WO-P2-006 in-tree state test (delivery `764e4db`; ownership/state tests at `ui.rs:49020+` at base); re-run with per-test output. | 26.721+current |

### Attack vectors (C)

- Side submit *selecting* the side thread (SC-3 is the heart of "without
  interrupting").
- `/side` row rendered while its guard should hide it (or vice versa).
- SC-5 recorded `verified` from a GUI frame alone — it is a state-machine
  criterion; require the unit test or an equivalent source pointer.
- The official-side citation itself is medium-high confidence: if B finds a
  conflicting official behavior in a *newer* official doc, record it — do
  not force-fit the rubric.

---

## R-06 — Projects and chats (§5.1; claimed partial, multi-folder CLOSED via WO-P1-003 @ `d06ae3b`)

### Official reference behavior

**[26.715, in-baseline]** Multi-folder local projects: `Edit project` adds
related folders + a primary choice; new chats, Git, AGENTS.md/skills/
config.toml discovery use the primary folder; secondary folders serve file
search/read/edit `[docs-derived] A §6; VDN pre-baseline context 26.715]`.
**[26.721]** Grouping, bounded full-text chat search, archive/unarchive/
delete, rename, pinning `[historical-record] A §1`. **[26.825, cloud-side —
DEFERRED]** unified pinned threads + shared thread snapshots (2026-08-20)
`[docs-derived] A §9` — proprietary cloud, never counted (Appendix C).
P3 residual (in-claim, not a defect): richer metadata.

### Flauz claim under review (PR §5.1)

"Bounded search with stable snippets + pagination, command menu,
archive/delete/rename, codexRS-owned bounded pinning; sidebar Chats list +
Ctrl+G palette entry render `[runtime-observed] ev/06, ev/08`;
multi-folder model on main (WO-P1-003): `LocalProjectSummary.folders`
(cap 16) + Edit project surface + primary-swap re-key + related folders in
file search; single-path legacy projects load primary-only
`[source-derived core lib.rs:908, 4719; B2 §6]`."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| PC-1 | `LocalProjectSummary` carries `folders: Vec<PathBuf>` capped at `MAX_LOCAL_PROJECT_FOLDERS = 16`; `path` never a member of `folders`; folders absolute + unique. | S | Anchors at base: `crates/codex-core/src/lib.rs:117` (cap), `lib.rs:914-927` (struct + invariants), `lib.rs:10171` (enforcement site). | 26.715/26.721 |
| PC-2 | Legacy single-path projects load primary-only (no data loss, no schema force-upgrade beyond v4). | S/U | Schema v4 migration path + legacy-load test (WO-P1-003 delivery); storage `user_version = 4` (`workspace_folders` table; `seed-wo-p1-003.py` shows the shape). | 26.715/26.721 |
| PC-3 | Edit project surface: primary row with Primary badge; related rows with Make-primary + Remove; Add folder; Done. | G | Re-run `evidence/wo-p1-003/scene-d6-v5.sh` with `seed-wo-p1-003.py` (captures 03/04). | 26.715 |
| PC-4 | Primary swap re-keys: new-chat path indicator follows the new primary; state persists across close/reopen (v4 round-trip). | G | D6 captures 04-06 (path indicator `…/alpha` → `…/beta`; swap persists). | 26.715 |
| PC-5 | Related folders join file search: palette file search finds `BETA-NOTES.md` from the related folder while primary is `alpha`. | G | D6 capture 02 (Ctrl+K → "search files" → `BETA`). | 26.715 |
| PC-6 | Sidebar Chats list + Ctrl+G palette entry render (baseline grouping surface). | G | ev/06/ev/08-class boot scene on the reviewed SHA. | 26.721 |
| PC-7 | Bounded search/pagination, archive/delete/rename, pinning remain intact (the historical-complete slice the partial row still claims). | S/U | Focused tests named in PM "Projects and chats" history; if any were dropped since, record `defect-found`. | 26.721 |

### Attack vectors (C)

- Cap enforced on add but not on import/migration (check all `folders` write
  sites, not just the UI add path).
- Primary swap persisting in the UI but not re-keying the prepared cwd for
  the next `thread/start` (PC-4's two halves).
- File search returning related-folder hits only after a restart (state
  propagation), or leaking non-workspace paths.
- PC-7 verified by prose instead of named tests (C demands test names).

---

## R-07 — Settings shell (§5.9; claimed partial)

### Official reference behavior

**[26.721]** 274 px shell: `Back to app`, bounded search (Ctrl/Cmd+F),
Personal/Integrations/Coding/Archived groups, filtering, no-results state;
full stable registry of 26 sections `[historical-record] A §9`. **[26.825]**
Settings > Import added (2026-08-11) `[docs-derived; VDN delta table]`.
**[26.908, cross-check only]** the official login-surface palette lists a
10-page Settings group (General, Import, Appearance, Voice, Pets(26.908-only
line), Git, Connections, Environments, Worktrees, Configuration)
`[runtime-observed: linux-preview 26.908.70816; LX 03]`.

### Flauz claim under review (PR §5.9)

"Stable-shaped shell implemented; nav renders 15 rows in 3 groups;
`SettingsSection` enum has 18 sections (CodeReview/Worktrees/ArchivedChats
contextual/hidden) `[source-derived ui.rs:2398-2417; runtime-observed
ev/07]`; settings search filters nav correctly ("import" → Personal +
Import) `[runtime-observed ev/23]; remaining sections only with working
host contracts (ledger bounded)`."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| SS-1 | `SettingsSection` enum = 18 sections; `DEFAULT_NAV_SECTIONS` = exactly the 15 default-nav rows; CodeReview/Worktrees/ArchivedChats contextual-hidden as claimed. | S | Anchors at base: `crates/codex-app/src/ui.rs:2494` (enum), `ui.rs:2522` (`const DEFAULT_NAV_SECTIONS: [Self; 15]`). | 26.721 |
| SS-2 | Nav renders the 15 rows grouped Personal/Integrations/Coding (+ Archived handling per claim). | G | ev/07-class scene on the reviewed SHA; VLM-read group headers + row count. | 26.721 |
| SS-3 | Settings search filters the nav: "import" → Personal + Import rows only; garbage query → no-results state; search is bounded. | G | ev/23 reproduction + a no-match query. | 26.721 |
| SS-4 | Import route exists (2026-08-11 official addition) with the three providers Claude Code / Claude Cowork / Cursor. | S | `ImportProvider::{ClaudeCode,ClaudeCowork,Cursor}` (PR §5.9 cites `core lib.rs:3478-3485`, `ui.rs:34696+` — re-resolve the symbols at the reviewed SHA). | 26.825 |
| SS-5 | Absent sections (Voice, Pets, Cloud preferences/environments, Appshots, Computer history, Environments…) are host-contract-bound deferrals: verify they are *labeled* bounded in the ledger, not silently dropped. | S | Ledger §5 gap decisions + PR §5.9 Gap cell wording. (These must NOT count against the partial row — Appendix C.) | 26.721/26.825 |
| SS-6 | Cross-check only: palette Settings group indexes every default-nav section (the SS↔KA seam; WO-P2-004 closure) — execute under KA-1 and cross-reference here. | G | See KA-1. | 26.908 cross-check |

### Attack vectors (C)

- Row count drift (16 rows / 4 groups) or enum growth without the matrix
  row being updated — the *claim's own numbers* are the bar for this
  criterion, not the official 26.
- SS-4 verified only by nav-row presence; the claim's provider set must be
  source-verified.
- Using the official 26-section registry to fail Flauz (the row's partial
  status already encodes that delta; host-contract bounds are deferrals).

---

## R-08 — Keyboard and accessibility (§5.10; claimed partial, with WO-P2-004 + WO-P2-007 sub-closures)

### Official reference behavior

**[26.721]** Ctrl/Cmd+K, Ctrl/Cmd+Shift+P, Ctrl/Cmd+G; Ctrl/Cmd+P
search-files drill-in; arrows/Enter/Escape; dynamic Settings group in the
palette; editable shortcut registry with stable grouping; complete focus
order, screen-reader labels `[historical-record] A §10`. **[current docs]**
Clear terminal Ctrl+L/Ctrl+K, font-size Ctrl+±/0, Toggle file tree
Ctrl+Shift+E (post-baseline, intro `[unverified]` — VDN §E.4)
`[docs-derived]`. **[26.908, reference+1 cross-check]** runtime overlay
shows 22 rows incl. bindings absent from Flauz's registry (Search Files
Ctrl+P, Copy deeplink Ctrl+Alt+L, Copy working directory Ctrl+Shift+C,
Rename chat Ctrl+Alt+R, Close Tab Ctrl+W, Archive chat Ctrl+Shift+A, New
standalone chat Ctrl+Alt+O, Toggle pin Ctrl+Alt+P, Back/Forward Ctrl+[/],
recent-chat cycling Ctrl+Tab/Ctrl+Shift+Tab, Switch to Work Alt+2)
`[runtime-observed: linux-preview 26.908.70816; LX 05-07 + PR §9 override
16]` — that inventory delta is the recorded P3 gap, not a new failure.

### Flauz claim under review (PR §5.10, condensed to its checkable core)

"Native command palette with verified stable registry subset
(**51-command** palette registry; every default-nav settings section
indexed — WO-P2-004 merged `7aa7163`: six new entries Profile, Import,
Browser, Configuration, Hooks, Git with title/description/icon mirroring
the settings-nav rows, no guards; `SettingsSection::DEFAULT_NAV_SECTIONS`
registry with coverage + filtering tests); Ctrl+/ overlay + editable
settings page `[runtime-observed ev/11, ev/15]`; Ctrl+P routes through the
`searchFiles` interceptor (WO-P2-007 merged `a3c0e01`: dead `OpenFileSearch`
removed; regression test; D10b lab evidence). Pending: remaining stable
commands, complete focus order, screen-reader labels, reduced-motion."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| KA-1 | Every `DEFAULT_NAV_SECTIONS` row has a palette entry (coverage test exists and passes; the Settings palette group lists all default-nav sections, unauthenticated + repository-independent). | U + G | Coverage/filtering tests at the `ui.rs:48947-48981` region (anchors at base); then the D7/D7b scene (`evidence/wo-p2-004/scene-d7.sh`): empty-query group + scrolled fold (wheel-scroll — arrow-cycling does NOT scroll the fold, per the WO-P2-004 calibration note). | 26.721 (+26.908 runtime confirmation LX 03/04) |
| KA-2 | Palette queries "import", "profile", "browser", "configuration", "hooks", "git" each resolve and navigate to their settings pages unauthenticated. | G | D7 captures 03-14 (the ev/18 "import"→No-matches gap remediation). | 26.721/26.825 |
| KA-3 | `PaletteCommand::ALL` = 51 commands at the reviewed SHA; any drift is recorded against the claim's number, with the growth attributed (WO delivery vs silent drift). | S | Anchor at base: `crates/codex-app/src/ui.rs:3407` `const ALL: [Self; 51]`. | 26.721 |
| KA-4 | Ctrl+P is owned by exactly one registry command (`searchFiles`) and opens the Files palette with a workspace seeded; with NO workspace the Files palette early-returns by design (documented F-A4 residual — silent on the bare entry surface is the known, documented behavior, not a new defect). | U + G | Ownership test at `ui.rs:51185-51201` (anchor at base: "Ctrl+P is owned by exactly one registry command: searchFiles"); D10b scene (`evidence/wo-p2-007/` workspace-seeded). | 26.721 |
| KA-5 | Palette entry bindings: Ctrl+K / Ctrl+Shift+P / Ctrl+G live and dispatch (registry subset verified per PM history). | G | Boot scene: three palette opens. | 26.721 |
| KA-6 | Open residuals stay honestly open: remaining stable commands, complete focus order, screen-reader labels, reduced-motion are pending — the claim is `partial`; these do NOT fail the row, but any of them claimed done without evidence is a finding. | S | PR §5.10 Gap cell + ledger §5 item 17 wording. | current |

### Attack vectors (C)

- A default-nav section present in SS-1 but missing from KA-1's coverage
  (the SS↔KA seam is where WO-P2-004 actually closed).
- Coverage test passing while the *palette group listing* hides entries
  below the fold without scroll (B must capture the scrolled frame — D7b
  calibration note says arrow-cycling cannot reach it).
- KA-4's F-A4 no-workspace residual re-reported as a defect (it is
  documented follow-up material — WO-R-SWEEP finding class).
- 26.908 overlay inventory used as the pass bar (version-layer violation;
  the correct home for that delta is the P3 shortcut-delta gap).

---

## R-09 — Notifications and tray (§5.10; claimed partial)

### Official reference behavior

**[26.721]** Background completion notification: in-app banner (Open/
Dismiss) + Windows quiet-hours-respecting notification-area toast; Linux
freedesktop best-effort; title + tray tooltip count of running/
awaiting-approval non-selected chats, reverting when none remain
`[historical-record] A §10 Notifications; PM row 104]`. **[26.825,
DEFERRED]** event-triggered scheduled-task notifications go through the
cloud Scheduled inbox — out of scope (Appendix C).

### Flauz claim under review (PR §5.10)

"Bounded in-app banner (Open/Dismiss) + matching window-title/
notification-area tooltip; gh-missing toast banners observed on multiple
pages `[runtime-observed ev/03/04]; pending tray groups, badges, sounds
(ledger enhancement); Linux tray/global shortcuts unavailable (see
packaging row).`"

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| NT-1 | A bounded in-app banner with Open/Dismiss actions exists on the cited paths (gh-missing toast on Repository/PR/Plugins pages — the runtime-observed half of the claim). | G | ev/03/ev/04-class scene on the reviewed SHA (gh absent in lab → the honest banner renders; Open/Dismiss actionable). | 26.721 |
| NT-2 | The *completion*-banner + window-title/tooltip count (the `[historical-record]` half) is evidenced by source + tests, and B records explicitly that its runtime GUI evidence in this wave is adjacent-class (error toasts), NOT a completion event — the completion path needs an authenticated background turn. | S + not-run note | Title-tooltip count logic + banner component (source-read); any focused tests named in PM history. GUI half: `not-run` with the auth bound. | 26.721 |
| NT-3 | Tooltip-count semantics: non-selected chats running or awaiting approval; reverts to the plain app name when none remain. | S | Title/tooltip construction site (source-read; confirm the count source matches the non-selected running/awaiting-approval subset, not a stale copy of PM prose). | 26.721 |
| NT-4 | Tray groups/badges/sounds remain pending (ledger enhancement) and Linux tray/global shortcuts stay documented platform bounds — deferral labeling intact. | S | Ledger §5 + KF "Active release-candidate limitations" + `docs/platform-support.md`. | 26.721 |

### Attack vectors (C)

- **The NT-2 trap is the point of this row:** ev/03/04 are error-path
  banners; recording them as evidence of the *completion* banner would be
  an evidence-class substitution. C checks every `verified` on NT-2 for
  exactly this.
- Title tooltip counting all chats (not the non-selected running/
  awaiting-approval subset).

---

## R-10 — Activity view & unread attention (§5.10, added by C2 audit; claimed partial — flipped missing→partial by WO-P2-008 @ `00a3392`)

### Official reference behavior

**[26.727 → in the 26.825 current target]** "Activity view" in the sidebar:
bell icon / Ctrl/Cmd+Alt+U shows recently engaged chats needing attention
(verbatim changelog, 2026-07-30) `[docs-derived]`; bindings: Clear all
unread indicators Shift+Esc; Next chat needing attention Ctrl+Alt+A;
Toggle Activity view Ctrl+Alt+U; adjacent per-chat mark-unread Ctrl+Shift+U
`[docs-derived] A shortcuts table; REF2 R1 §"Keyboard shortcuts"]`.
**Honest reference bounds (REF2 R1):** view contents (event classes,
ordering, grouping, per-row actions), visual unread treatment, empty
states, multi-project scoping, and visit-clears semantics are all
`[unverified]` — no evidence layer describes them; the official Linux
preview could not observe the view (auth-walled; LX bound 1). Baseline
26.721: not present (post-baseline capability).

### Flauz claim under review (PR §5.10, condensed to its checkable core)

"Unread-attention session state + four registry bindings on main
(WO-P2-008): bounded `needs_attention_task_ids` (capped
`MAX_VISIBLE_THREADS`, not persisted) flagged on background turn completion
(failed turns included) + approval requests in non-selected chats; visit
clears, archive drops; sidebar 6px dot + medium-weight title;
`toggleThreadUnread` Ctrl+Shift+U (round-trip + honest no-selection status),
`nextUnreadChat` Ctrl+Alt+A (cyclic sidebar-order jump, selected chat never
a candidate, honest empty statuses), `clearAllUnread` Shift+Escape (honest
count incl. 'No unread chats'), `toggleActivityView` Ctrl+Alt+U (honest
guidance — view surface is a separate future WO);
`MAX_KEYBOARD_SHORTCUT_COMMANDS` 71→76; 7 core state tests + 6 app binding
tests `[source-derived ui.rs/lib.rs; runtime-observed: evidence
wo-p2-008/ — D11b four bindings VLM-read verbatim statuses, D11 honest
no-selection path]`."

### Criteria

| ID | Checkable criterion | Class | Cheapest honest verification | Version |
|----|---------------------|-------|-------------------------------|----------|
| AV-1 | `MAX_KEYBOARD_SHORTCUT_COMMANDS = 76` and the four command IDs (`toggleThreadUnread`, `nextUnreadChat`, `clearAllUnread`, `toggleActivityView`) are registry members. | S | Anchors at base: `crates/codex-core/src/lib.rs:132` (MAX=76), `lib.rs:143/154-156` (IDs). | 26.825 (26.727 bindings) |
| AV-2 | `needs_attention_task_ids` is bounded (cap `MAX_VISIBLE_THREADS`, FIFO eviction), session-only (not persisted), flagged on background turn completion including failed turns, and on approval requests in non-selected chats; visit clears; archive drops. | U | The 7 core state-machine tests (anchors at base: `crates/codex-core/src/lib.rs:37359+` — assertions on `["background", "failing"]` etc.); run with per-test output (WO-P2-008 gate-2 form). | 26.825 (bounded-by-design where reference is unverified) |
| AV-3 | All four bindings resolve visibly at the empty surface with the exact verbatim statuses: "Select a chat before marking it unread." / "No chats need attention." / "Activity view is not available yet. Use "Next chat needing attention" to jump to unread chats." / "No unread chats". | G | Re-run `evidence/wo-p2-008/d11b-empty-surface.sh`; VLM-read the four frames; md5 sequence differs frame-to-frame (D11b format). | 26.825 (honest-guidance pattern) |
| AV-4 | The four app binding tests (registry ownership/metadata/dispatch) pass. | U | The 6 codex-app tests (CI matrix is authoritative; local test-mode gpui compile is OOM-excluded per the WO-P2-008 README — cite the CI run or per-test output where feasible). | 26.825 |
| AV-5 | The full-flow scene (chats created → mark unread → jump → remark → clear) is honestly bounded in a no-runtime lab: D11's form (capture + document the no-selection honest path) is the accepted evidence class; dot-on-row and jump-between-chats are unit-covered only there. B must either run it in a runtime-enabled lab or record `not-run` with the D11 bound — never mock a runtime. | G (bounded) | `d11-unread-attention.sh`; compare against the D11 README framing. | 26.825 |
| AV-6 | Scope honesty: NO Activity-view *surface* is claimed (bell/view = separate future WO; the Ctrl+Alt+U binding gives honest guidance meanwhile); unread persistence across restarts is session-only by design (reference behavior unverified). Any claim beyond this scope is a finding; the residuals themselves are NOT defects. | S | PR §5.10 Gap cell + wo-p2-008 README "Documented residuals". | 26.825 |

### Attack vectors (C)

- `nextUnreadChat` offering the currently selected chat as a candidate
  (violates "selected chat never a candidate").
- `clearAllUnread` not reporting the honest count, or clearing without the
  status line.
- Any mock runtime used to "pass" AV-5 (automatic evidence rejection).
- AV-2's FIFO cap verified only on the push site — check the eviction at
  the `lib.rs:8888-8891` region and any other write sites.
- A bell/view surface silently appearing in a screenshot while the claim
  says none exists (scope violation in the *other* direction — claim
  understates reality, matrix row stale).

---

## Appendix A — Source anchors verified at the review base

All anchors below were resolved at `research/rwo-020` = `main` @
`d479c7b9e725e1af1353140d49226b3f9a16be9d` (RWO-020's own sweep, 2026-09-20;
line numbers drift, symbols are the stable reference):

| Anchor | What it proves | Rubric use |
|---|---|---|
| `crates/codex-core/src/lib.rs:227-234` `STABLE_REFERENCE` | pinned 26.721.3996.0 / 0.146.0-alpha.3.1 / sha256 `39e9e0…a6ef3` | RB-1 |
| `crates/codex-platform/src/lib.rs:261-283` `resolve_codex_binary` (+`windows_stable_codex_cache_candidate`, `sha256_matches`) | explicit → env → hash-gated cache → npm → bare-name order | RB-2 |
| `crates/codex-app/src/ui.rs:14384` `"App-server online"` | boot footer copy | RB-3 |
| `crates/codex-core/src/lib.rs:914-927` `LocalProjectSummary` + `folders: Vec<PathBuf>` | native-path model (no shim) | MW-1, PC-1 |
| `crates/codex-core/src/lib.rs:117` `MAX_LOCAL_PROJECT_FOLDERS = 16` | folder cap | PC-1 |
| `crates/codex-app/src/ui.rs:4733` Ctrl+/ binding; `ui.rs:3127/3505` palette/settings rows | shortcuts dialog entries | KS-1, KS-4 |
| `crates/codex-core/src/lib.rs:4359-4383` `FeedbackClassification` (`bug`, `bad-result`, `good-result`, `safety_check`, `other`) | five stable category IDs | FB-1 |
| `crates/codex-app/src/ui.rs:41719/41776` `Share feedback` dialog + category buttons | dialog surface | FB-1, FB-3 |
| `crates/codex-protocol/src/lib.rs` + `crates/codex-platform/src/app_server.rs` `feedback/upload` | typed upload request | FB-2 |
| `crates/codex-app/src/ui.rs:4698` alt-s binding; `ui.rs:2675/2826/8323/10499` `openSideChat`; `ui.rs:7822/23628` `/side` guard; `ui.rs:48157-48159` ownership test | side-chat wiring | SC-1, SC-4 |
| `crates/codex-app/src/ui.rs:2494` `enum SettingsSection`; `ui.rs:2522` `DEFAULT_NAV_SECTIONS: [Self; 15]`; `ui.rs:48947+` coverage iteration | settings shell counts | SS-1, KA-1 |
| `crates/codex-app/src/ui.rs:3407` `const ALL: [Self; 51]` (`PaletteCommand`) | palette registry size | KA-3 |
| `crates/codex-app/src/ui.rs:3189/10582` `searchFiles` command + Files-palette dispatch; `ui.rs:51185-51201` Ctrl+P ownership test | Ctrl+P interceptor routing | KA-4 |
| `crates/codex-core/src/lib.rs:132` `MAX_KEYBOARD_SHORTCUT_COMMANDS = 76`; `lib.rs:143/154-156` attention command IDs; `lib.rs:4757` `needs_attention_task_ids`; `lib.rs:8882-8891` capped push/eviction; `lib.rs:37359+` state tests | unread-attention machinery | AV-1, AV-2 |

## Appendix B — Evidence-tree index (per-row entry points)

| Row | Official-side evidence | Flauz-side evidence |
|---|---|---|
| RB | PM "Runtime bootstrap"; VDN §A (CLI runtime surface); `reference/stable-26.721.3996.0/manifest.json` | ev/06 (boot footer); ev/24 (no-runtime degradation); LX `official-app-log-extracts.txt` (official fatal counter-example) |
| MW | KF failure table row 1 (official failure record) | KF control claim; source anchors Appendix A |
| KS | PM "Active keyboard shortcut reference"; LX 05-07 + vlm-reads READ 7 (26.908 overlay cross-check) | ev/11 (editable page), ev/15 (overlay) |
| FB | PM "Feedback" (five IDs, upload contract) | palette source anchors; ui.rs dialog |
| SC | A §1 (medium-high confidence) | `evidence/wo-p2-006/` (D9 scene, D9b close probe, vlmd9a/b.json) |
| PC | A §1/§6; VDN 26.715 pre-baseline context | ev/06/08; `evidence/wo-p1-003/` (D6 6-frame scene + seed + vlm-reads) |
| SS | A §9; VDN delta (Import 2026-08-11); LX 03 (26.908 palette cross-check) | ev/07, ev/23; `evidence/wo-p2-004/` (15 captures, scenes d7/d7b) |
| KA | A §10; current-docs shortcuts; LX 05-07 (26.908 overlay, PR §9 override 16) | ev/08-11/15/18; wo-p2-004/; wo-p2-007/ (D10 defect proof, D10b fix proof, input-surface-sweep.md) |
| NT | A §10 Notifications; PM row 104 | ev/03/04 (error-banner class); source-read for completion path |
| AV | REF2 R1 (full reference analysis + unverified map) | `evidence/wo-p2-008/` (D11 limited scene, D11b empty-surface scene, md5s, VLM reads) |

## Appendix C — Deferral guard (never count, never fake)

Cloud environments; Scheduled tasks (incl. 26.825 event triggers + Scheduled
inbox notifications); Voice input; Sites (cloud parts); Visualizations;
Appshots (26.908 Windows = reference+1); Pets and Codex Micro (26.908 =
reference+1, incl. the 26.908 Pets "bell" — a different bell surface from
the Activity view, per REF2 R1 current-version notes); connector-specific
approval methods; plugin OAuth/no-auth callback; App OAuth; import
unsupported-project reporting; SSH profiles/remote chats; Settings host
contracts (the absent sections in SS-5); Computer History (macOS + cloud);
unified pinned threads / shared thread snapshots (cloud); Windows/macOS
GUI-lab validation as a class (labs unavailable — honest `not-run`, not
`defect-found`). Also protected: the intentional differences (unsigned
portable packaging GUI-006; native GPUI instead of Electron) and the
26.908→26.825 version-skew rule (26.908 observations never upgrade or
downgrade a 26.825 claim).

## Appendix D — Result-capture template (Worker B fills one block per criterion)

```markdown
#### <criterion-id> — <verdict: verified | defect-found | not-run>
- Reviewed SHA: <40-char>
- Step class run: S | U | G | R (which, concretely)
- Evidence pointer: <test name(s) + per-test result | frame file + md5 |
  file:line quote>
- Version layer honored: 26.721 | 26.825 | 26.908-cross-check
- Notes / bounds: <honest reason if not-run; reproduction if defect>
```

## Appendix E — Honest limits of this rubric (RWO-020's own)

1. The official-side Activity-view reference is itself `[unverified]` in
   shape (REF2 R1); AV-2/AV-5 therefore verify Flauz against the *documented
   binding semantics*, not a visual reference — the visual dot/title
   treatment has no official counterpart to compare against.
2. Side chats' official citation is medium-high confidence (A §1); if a
   future official doc contradicts Ctrl+Alt+S semantics, SC-3's
   "without interrupting" remains the load-bearing invariant.
3. Line numbers in Appendix A are base-`d479c7b` snapshots; symbols are
   the stable anchors (per §1.5 rule 5).
4. GUI scenes reference the sealed lab recipe (Xvfb + picom, isolated
   HOME/XDG/CODEX_HOME) as practiced in the cited WO evidence dirs; RWO-020
   did not execute them (not this order's job) and records no results on
   Flauz's behalf.
5. The brief's minimum list omits three of the eight `complete` rows (Git
   process hygiene; Marketplace admin-disabled install; Stable-failure
   regression controls — PR §8.1). The sibling first-dispatch `rubric.md`
   §4.3/§4.4/§4.7 covers exactly those three rows (13 rows / 71 criteria
   total); this file's Appendix E.5 flag is therefore resolved by
   coexistence — the Lead chooses the lineage(s) B executes. MW here covers
   the KF-regression-control *class* (its sibling controls share the KF
   evidence pattern).
