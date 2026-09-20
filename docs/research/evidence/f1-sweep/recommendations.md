# F1 closure packet — closure recommendations (WO-F1-SWEEP-001)

- **Work order:** WO-F1-SWEEP-001 (Wave 1, Worker C — evidence-only)
- **Base:** `a664644718210e254952db518606d80906eee448`
- **Date:** 2026-09-19
- **Audience:** Tech Lead, for the F1 close-gate decision
  (docs/IMPLEMENTATION-ROADMAP.md F1 Gate: "every remaining full-reference
  difference is explicitly classified; release-critical journeys remain green;
  and the final parity/accessibility/release journeys pass").

> **Lead integration note (2026-09-19, Task 85):** the proposed follow-up IDs
> WO-P2-013..016 in this packet were renumbered to **WO-P2-017..020** — the
> WO-P2-013 ID is already assigned to the in-flight Activity-view work order
> (Wave-1 Worker B, dispatched 20:00 UTC before this packet was authored).
> Scope text unchanged; only the IDs moved.
>
> **Closure status (2026-09-20):** WO-P2-018 **CLOSED** (PR #37 → `b562397ce6325f12d21f67f0549f5860b7801306`; D18 evidence wo-p2-018/) and WO-P2-020 **CLOSED** (PR #38 → `30572a43afcf3af6fa29d2b9da097467d74e7401`; D20 evidence wo-p2-020/ — D-3 resolved: separate WO, honest-status treatment). WO-P2-017 r1 REJECTED by Lead verification (deterministic palette-close panic, D17 evidence) — fix round in flight; WO-P2-019 dispatched (modal focus traps).

## 1. Gate clause 1 — "every remaining full-reference difference is explicitly classified"

**Delivered by this packet** (parity-inventory.md): all 41 Product-parity rows
classified (5 closed / 1 in-flight / 17 bounded / 10 P2 / 8 P3; 41 = 5+1+17+10+8)
plus the PR finding-16 shortcut delta (9 closed, 2 P3).

**Lead decisions still required to freeze the classification** (not new work —
triage):

| Decision | Options | Recommendation |
| --- | --- | --- |
| D-1 Layer rule for post-baseline items (26.727/26.825) | keep the pinned-baseline rule of this inventory (P3) vs adopt the parity report's current-target rule (P2) | freeze the **pinned-baseline rule for F1** (the parity-matrix is the F1 spine) and carry the re-sliced items (multi-repository projects, artifacts renderers/canvas, first-run welcome set) as an explicit "current-target backlog" annex for F2 |
| D-2 "unread state" staleness in PM Projects-and-chats row | edit the row text (Lead-owned) or leave | edit after WO-P2-012/Activity-view merge — the row currently lists a remainder that WO-P2-008 closed (see parity-inventory §6.3) |
| D-3 F-A4 (Ctrl+P silent with no workspace) | extend WO-P2-012's scope or open WO-P2-020 | extend WO-P2-012 if the rebase is cheap (one more guarded status); otherwise WO-P2-020 (below) |

## 2. Gate clause 2 — "release-critical journeys remain green"

**F1-blocking items** (the two in-flight streams + their residuals):

1. **Merge WO-P2-012** (guard honesty, six evidenced silent no-op states +
   executor fall-through tests). Branches exist
   (`origin/feat/wo-p2-012-guard-honesty` `0547055`, `-r2` `4fc763a`) on an
   older base — rebase/reconcile per the roadmap (Workers A/B lanes). Until it
   lands, J-04's "a failed capability request must never end in a silent
   no-op" (PJ L197) is contradicted by F-A1/A2/A3/A6/F-D1/D2
   (accessibility-inventory §e) — these are the only user-visible defects in
   the packet besides F-A4.
2. **Land or explicitly defer the Activity-view surface** (J-17's primary
   surface; deferred `toggleActivityView` target, honest-guidance binding
   meanwhile). Recommendation: land it this wave (Workers A/B own it per the
   wave plan); if schedule pressure forces a defer, record the defer as a
   bounded F1 residual with the honest-guidance binding as the accepted
   interim — do not leave it undecided.
3. **Re-run the release-critical journey battery on the merged result**
   (house evidence convention): side chats, Ctrl+P palette, palette rows +
   guards, browser chords, browsing history, unread attention — the D-series
   scenes already exist per WO; the Lead re-runs them at the post-merge SHA.

**Not blocking (defer to F2+), with anchors in parity-inventory §2:**
in-app Markdown/code editing (PR §8.2 item 7 — needs Lead scoping first);
WebMCP site tools (blocked on fork-runtime; gated in stable); Record & Replay
(Computer-Use-dependent); browser extension beyond Chrome (adjacent
deliverable); multi-repository review; permission-profile granular editor;
artifacts renderers/canvas/generated-image editing; dynamic Computer Use
tools; billing entry points; first-run welcome/diagnostics/update prompt;
tray groups/badges/sounds; Skills recommended/install flows; the P3 polish
set (rows 2/4/8/11/12/13/14/23); the bounded set (proprietary/awaiting public
contracts — rows 9/15/18/20/22/24–27/31–36/40/41).

## 3. Gate clause 3 — "the final parity/accessibility/release journeys pass"

### 3.1 Accessibility closure set (from accessibility-inventory.md)

| Finding | Class | F1 treatment |
| --- | --- | --- |
| Silent no-op family F-A1/A2/A3/A6/F-D1/D2 | defect | **blocker** — WO-P2-012 (in-flight) |
| F-A4 Ctrl+P silent without workspace | defect | **blocker-decision** — WO-P2-012 scope extension or WO-P2-020 |
| Palette close focus restoration | gap | **blocker** for the "final accessibility journeys" clause — WO-P2-017 |
| Visible palette entry (no toolbar button) | gap | **blocker** for the discoverability contract (PJ §1 layer 1) — WO-P2-018 |
| Icon-only archived-chats deletion; attention-dot label | gap | WO-P2-018 |
| Tab-trap completion for the four non-evidenced confirmation modals | gap | WO-P2-019 |
| Broader focus order; full contrast parity | gap | bounded F1 close-out slice inside WO-P2-017/015 + a contrast item; the remainder is honestly open (PM baseline) |
| Screen-reader labels / AT tree; OS-level reduced-motion; status AT announcement; titlebar AT naming | platform-bound | **document, do not block F1** — record as upstream-GPUI dependencies in the close packet; revisit at F11 (web client has its own a11y pass) |

### 3.2 Release-gate execution set

Run the scenes defined in release-gate-inventory.md at the release-candidate
SHA. Minimum F1-blocking set: RG-RECONNECT-01..05, RG-SOAK-01/02/05,
RG-FRESH-01..04, PKG-1..5, CI-1..3. Full battery in the file.

## 4. Recommended follow-up work orders (bounded scopes; IDs continue the house series)

| ID | Title | Bounded scope | Files/subsystems | Tests / evidence |
| --- | --- | --- | --- | --- |
| **WO-P2-017** | Palette/overlay close — focus restoration | on `close_command_palette` (ui.rs:8961–8971) and the shortcuts-overlay close path, explicitly restore focus to the previously focused surface (or a deterministic default); no other behavior | `crates/codex-app/src/ui.rs` only | 2 focused tests (restore target after palette close; after overlay close) + one D-scene (open → close → focus probe); a11y inventory §(a)/(d) anchors |
| **WO-P2-018** | Visible-entry + label parity for icon-only controls | add a visible command-palette entry button in the chrome; give the archived-chats single deletion a visible label (or tooltip); give the unread-attention dot a tooltip/text alternative | `crates/codex-app/src/ui.rs` only | focused render tests + D-scene (cold start: open palette via button); journey-inventory §2 anchors |
| **WO-P2-019** | Confirmation-modal focus-trap completion | extend the evidenced tab/shift-tab trap pattern (ui.rs:5112–5156) to the four confirmation modals that have focus handles but no trap bindings (remote pairing, remote confirmation, account logout, plugin install) — or record an explicit waiver | `crates/codex-app/src/ui.rs` only | focused tests per modal + D-scene |
| **WO-P2-020** | F-A4 — honest feedback for Ctrl+P without workspace | the Files palette early-return (ui.rs:8943–8945) emits a bounded honest status instead of silence (parity treatment decision: honest status vs empty palette) | `crates/codex-app/src/ui.rs` (+ status string in core if needed) | 1 focused test + D-scene; only if D-3 chooses a separate WO over the WO-P2-012 scope extension |
| **WO-UX-001** | Keyboard-only journey battery (a11y scenes) | define + run RG-A11Y-01 (keyboard-only full journey: palette → settings → terminal → browser → history → attention, no pointer) and RG-A11Y-02 (focus-order probes for the source-unverifiable findings) | docs/evidence only (scene scripts) | scene evidence + VLM reads; settles accessibility-inventory §source-unverifiable 1–4 |
| **WO-REL-001** | Reconnect/recovery release scenes | execute RG-RECONNECT-01..07 at the release SHA in LINUX_GUI_LAB (isolated `CODEX_HOME`, pinned CLI, `CODEX_RS_CODEX_BIN` per the RWO-022 FW-7 restore pattern) | lab assets + evidence | per-scene pass criteria in release-gate-inventory §(a) |
| **WO-REL-002** | Fresh-machine/packaging release scenes | execute FR-1..FR-4 + PKG-1..PKG-5 from the published archives on a clean profile | lab assets + evidence | per-scene pass criteria in release-gate-inventory §(c) |
| **WO-LAB-002** | Soak scenes | execute RG-SOAK-01..05 (≥ the blocking subset 01/02/05 full-length) with the sampling log + gates G-1..G-7 | lab assets + evidence | per-scene pass criteria in release-gate-inventory §(b) |

WO-P2-012 itself (in-flight) and the Activity-view surface (in-flight) are
already owned by Workers A/B this wave — no new order needed, only the
merge/defer decisions.

## 5. Recommended lab scenes (LINUX_GUI_LAB; probes summarized — full
procedures in release-gate-inventory.md)

| Scene | Probe (one line) | Gate |
| --- | --- | --- |
| RG-RECONNECT-01 | kill app-server child mid-session; observe footer retry cadence 1→2→4→8→16→20 s (cap) | bounded timer + labels |
| RG-RECONNECT-02 | repeated kills while a reconnect is pending | exactly one queued reconnect (race safe) |
| RG-RECONNECT-03 | let a retry succeed, then kill again | auto-return online + timer reset |
| RG-RECONNECT-04 | loaded/background chats during loss | rehydrate + resume after reconnect |
| RG-RECONNECT-05 | unreachable runtime at first launch | retryable startup failure |
| RG-RECONNECT-06 | approval request during loss | approval survives recovery |
| RG-RECONNECT-07 | Bedrock-login restart path | exactly one runtime instance |
| RG-SOAK-01 | ≥ 4 h session, RSS sampled 15 min | G-1 memory bound |
| RG-SOAK-02 | ≥ 100 chats, multi-page histories | G-3 bounded growth + pages |
| RG-SOAK-03 | ≥ 20 streamed turns, variable-height content | G-2 latency |
| RG-SOAK-04 | terminal tabs + browser streaming long-run | G-3/G-7 |
| RG-SOAK-05 | child kill at T+2 h under soak | G-5 reconnect-under-soak |
| RG-FRESH-01..04 | About window / failed-backend visibility / empty-workspace Open folder / runtime resolution order | PM L111 contract |
| RG-PKG-01..05 | ZIP/tar.gz verify steps, overlay sibling, desktop entry never-overwrite, XDG bounds, Xvfb smoke | platform-support.md |
| RG-A11Y-01 | keyboard-only full journey across the six F1 surfaces | no pointer required |
| RG-A11Y-02 | focus-order probes (palette/overlay close, modal traps, PTY focus) | settles source-unverifiable items |

## 6. Honesty notes

1. This packet contains **zero product-code changes**; every recommendation
   above is a proposed scope for a Lead-dispatched work order (Workers A/B own
   the product lanes this wave).
2. Every classification carries an anchor; where source inspection was
   inconclusive the item is marked `source-unverifiable` with the runtime
   evidence that would settle it (parity-inventory §6; accessibility-inventory
   §source-unverifiable; journey-inventory §4).
3. Expected Lead verification: spot-check a sample of anchors — the load-bearing
   ones are `AppServerReconnectScheduler` (backend.rs:1796),
   `KEYBOARD_SHORTCUT_COMMAND_IDS` (lib.rs:139–215), the attention-dot render
   (ui.rs:14774–14782), the footer connection states (ui.rs:14819–14860),
   the modal trap bindings (ui.rs:5112–5156), and the palette open/close pair
   (ui.rs:8939–8971).
