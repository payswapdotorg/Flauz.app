# P2 batch-2 official reference notes — Activity view, browser address-bar history, palette/a11y residuals

Task WO-R-REF — next-batch P2 reference research (official Codex side). Author:
Worker A (reference). Date: 2026-09-18 (UTC). Branch:
`research/p2-batch2-reference` (documents only). Base: `main` @
`3c9f113851686e1774ce355ad744b56d6c937b2a`.

Scope: the three next-batch P2 capabilities by user impact — **R1** Activity
view + unread attention (parity item 8), **R2** browser address-bar history /
Google fallback (parity item 12), and **R3** the palette/shortcut half of
parity item 17 (remaining stable palette commands, keyboard-shortcut
inventory, focus order, screen-reader labels, reduced motion). Research and
documentation only — no product code, no Flauz-side implementation, no
proposed fixes. The Tech Lead turns this into bounded work orders.

## Method (per the program's reference doctrine)

Current parity target = historical baseline (Codex Desktop
`26.721.3996.0` + bundled CLI `0.146.0-alpha.3.1`) + current installed delta
(ChatGPT desktop `26.825.51511`). Features from `26.908+` (the Linux preview
line the lab ran) are **reference+1** — recorded here as forward-looking notes
only, never as the current target. Provenance labels follow parity report §4.3
exactly: `[runtime-observed]` (with the `linux-preview 26.908.70816`
version-skew form where applicable), `[source-derived]`, `[docs-derived]`,
`[historical-record]`; where every evidence layer is silent, `[unverified]`.
Conflict order (PR §7.3): runtime-observed > source-derived > docs-derived >
historical-record. Platform honesty (PR §7.4): Linux runtime evidence never
fills Windows/macOS cells.

### Citation key

| Tag | Source layer |
| --- | --- |
| PR §n | `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` (canonical, reconciled) |
| A §n | `docs/research/CODEX-REFERENCE-MATRIX.md` (Worker A predecessor matrix, commit `c083c38`) |
| Δ §n | `docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md` (26.825 delta layer) |
| VDN §n | `docs/research/evidence/codex-ref/version-delta-notes.md` (raw Worker A delta notes) |
| CL | `docs/research/evidence/codex-ref/changelog-delta-extract.md` (verbatim changelog extract) |
| LX | `docs/research/evidence/codex-linux/` (official Linux preview runtime evidence: `README.md`, `vlm-reads.txt`) |
| PM | `docs/parity-matrix.md` (26.721-era historical record) |
| B2 §n | `docs/research/FLAUZ-REFERENCE-MATRIX.md` (Worker B2, commit `a2343d3`) |
| src | Read-only source reads at base `3c9f113` (`crates/codex-app/src/ui.rs`, `crates/codex-platform/src/browser.rs`) — used only to state the Flauz-side current behavior each gap is measured against |

---

## R1 — Activity view + unread attention (parity item 8; shipped 26.727)

Parity anchors: PR §5.10 row "Activity view & unread attention (added by C2
audit)"; PR §8.2 item 8 ("Activity view & unread attention (26.727) — depends
on unread state"); PR §9 override 4 (unread state: P3 polish → P2,
current target).

### Reference behavior

1. **What it is.** The official 26.727 release "Added a new 'Activity view' in
   the sidebar to view which chats you engaged with recently and require
   attention. Click the bell or use Cmd/Ctrl+Opt+U to change to the new
   view." — verbatim changelog text `[docs-derived]` (CL, 2026-07-30 entry,
   "Other improvements" section). A matrix wording: "bell icon /
   Ctrl/Cmd+Alt+U shows recently engaged chats needing attention"
   `[historical-record + docs-derived]` (A §10 Sidebar).
2. **Shell placement + entry points.** It is a sidebar view ("in the sidebar",
   CL verbatim) of the 275 px shell sidebar — the same sidebar that carries
   Toggle sidebar Ctrl/Cmd+B and adaptive close ≤720 px at baseline
   `[historical-record]` (A §10 Sidebar). Entry points evidenced: (a) the bell
   icon, (b) the Ctrl/Cmd+Alt+U keyboard path. A matrix: "Activity view is the
   post-baseline sidebar addition (in-window at 26.727 → included in current)"
   `[historical-record + docs-derived]` (A §10). No third entry point (palette
   command, menu row) is evidenced in any layer `[unverified]`.
3. **What it shows.** "which chats you engaged with recently and require
   attention" (CL verbatim) — i.e. the membership criterion is
   recently-engaged + needs-attention. Which event classes populate
   "engaged", how rows are ordered, whether they group by project, and what
   per-row actions exist (open? archive? mark read?) are **not stated by any
   evidence layer** `[unverified]`.
4. **Unread / attention state.** The unread-attention model has three
   evidenced bindings (current official docs table, A §"Keyboard shortcuts"):
   Clear all unread indicators — Shift+Esc; Next chat needing attention —
   Ctrl+Alt+A; Toggle Activity view — Ctrl+Alt+U. All three are marked
   post-baseline (no ●) in the current-docs table `[docs-derived]` (A). The
   parity report carries all three on the §5.10 row with the label
   `[historical-record + docs-derived] A §10` (PR §5.10). How unread state is
   *represented visually* (bolding? badge counts? dot?) is `[unverified]` —
   no layer describes the visual treatment.
5. **How attention clears.** Only the global "Clear all unread indicators"
   Shift+Esc action is evidenced `[docs-derived]` (A shortcuts table; PR
   §5.10). Whether viewing a chat clears its indicator, whether there is a
   per-item ack, and whether the Activity view itself marks items read on
   view are `[unverified]`.
6. **Related unread surfaces.** Current official docs list a per-chat "mark
   unread" action bound Ctrl+Shift+U (post-baseline; baseline ● only for the
   adjacent pin/rename bindings) `[docs-derived]` (A shortcuts table). The
   baseline notifications row already counted "running/awaiting-approval
   non-selected chats" in title + tray tooltip `[historical-record]` (A §10
   Notifications) — the pre-26.727 attention signal the Activity view then
   gave a dedicated surface. Whether Ctrl+Shift+U mark-unread and the Activity
   view share one unread store is `[unverified]`.
7. **Multi-project interaction.** No evidence layer describes how the Activity
   view behaves across projects (all projects? current project? project
   grouping?) `[unverified]`. Note the capability landed one release after
   multi-folder projects (26.715) and in the same release as multi-repository
   review (26.727) `[docs-derived]` (Δ §3.1), so the official product had a
   multi-project sidebar when it shipped — but its scoping behavior was not
   captured.
8. **Auth wall (honest bound).** The official Linux preview (26.908.70816,
   reference+1) gates the entire shell behind sign-in: "No sidebar, no
   composer, no terminal/browser affordances are visible pre-auth"
   `[runtime-observed: linux-preview 26.908.70816]` (LX README finding 1).
   The Activity view itself was therefore **not runtime-observable** in
   WO-LAB-001; it is listed among the questions the lab upgrade was expected
   to make "directly checkable" only after authentication (PR §8.4). All
   Activity-view shape claims above remain `[historical-record]`/
   `[docs-derived]` on Windows/macOS layers only.

### Keyboard shortcuts (official)

| Action | Windows binding | macOS form | Baseline (26.721)? | Provenance |
| --- | --- | --- | --- | --- |
| Toggle Activity view | Ctrl+Alt+U | Cmd+Opt+U | No — 26.727 | `[docs-derived]` A shortcuts table (row "— (26.727)"); CL verbatim "Cmd/Ctrl+Opt+U" |
| Next chat needing attention | Ctrl+Alt+A | Cmd+Opt+A | No — post-baseline | `[docs-derived]` A shortcuts table; PR §5.10 |
| Clear all unread indicators | Shift+Esc | Shift+Esc | No — post-baseline | `[docs-derived]` A shortcuts table; PR §5.10 |
| Mark chat unread (adjacent surface) | Ctrl+Shift+U | Cmd+Shift+U | No — post-baseline | `[docs-derived]` A shortcuts table (baseline ● only pin/rename) |

Flauz-side current state (for gap measurement, `[source-derived]` at
`3c9f113` per PR §5.10 row): no Activity view or unread-attention surface —
no bell, none of the three bindings (key registry `ui.rs:4459-4552`), no
unread state; Flauz-side absence was source-verified in the C2 sweep at
`f113515` before the row was declared (PR §5.10, §6).

### Empty / loading / error states

`[unverified]` — no evidence layer describes the Activity view's empty state
(no chats needing attention), loading state, or error behavior. The lab could
not observe it (auth-walled, §R1.8); the changelog and docs are silent. The
only adjacent evidenced empty-state pattern is the signed-in empty workspace
offering `Open folder` `[historical-record]` (A §10 Empty/loading/error
states), which does not transfer to this view.

### Current-version notes

- **Baseline 26.721:** not present. The capability is post-baseline
  (26.727, 2026-07-30) `[docs-derived]` (Δ §3.1; VDN §B).
- **Current target 26.825:** present since 26.727 (in-window → included in
  the current installed reference) `[historical-record + docs-derived]`
  (A §10 Sidebar). The 26.825-line changelog entry (2026-08-25) adds nothing
  to this capability `[docs-derived]` (CL). Priority consequence: the PM
  ledger's old "unread state" P3-polish note was upgraded to a P2 capability
  by the version-skew rule (PR §9 override 4).
- **Reference+1 (26.908+):** no Activity-view change is evidenced in the
  26.908 line `[docs-derived]` (VDN §B reference+1 table). The 26.908
  Pets "bell follow-progress" (floating Pets controls with a bell icon that
  follows task progress) is a *different* bell surface on a different feature
  line and is itself reference+1/out of target `[docs-derived]` (Δ §4;
  VDN §B). The Linux preview could not confirm any Activity-view runtime
  behavior (auth-walled, §R1.8) `[runtime-observed: linux-preview
  26.908.70816 — bound]` (LX README "Bounds").

### OPEN QUESTIONS (R1)

1. **Activity view contents:** event classes, row ordering, grouping
   (by project? by recency?), per-row actions, and visual unread treatment —
   no layer answers; `[unverified]`. **Upgrade path:** authenticated run of
   the official app (operator's 26.825.51511 machine, or the Linux preview
   in LINUX_GUI_LAB with credentials) with the bell opened and screens
   captured; the app.asar route-inspection pattern (`/inbox`-adjacent route
   inventory, A §10) could also answer placement questions if a 26.825
   bundle is ever inspectable — the baseline route inventory carries no
   activity route, consistent with the 26.727 introduction.
2. **Clearing semantics:** does viewing a chat clear its indicator; is there
   per-item ack; does the Activity view clear on open — `[unverified]`;
   same upgrade path.
3. **Multi-project scoping:** all-projects vs current-project membership —
   `[unverified]`; same upgrade path.
4. **Palette/menu entry points** beyond bell + shortcut — `[unverified]`;
   an authenticated palette dump would settle it.
5. **Unread store coupling:** whether Ctrl+Shift+U mark-unread, the
   running/awaiting-approval tray count (baseline behavior), and the Activity
   view share one unread model — `[unverified]`.

---

## R2 — Browser address-bar history / Google fallback (parity item 12; shipped 26.727)

Parity anchors: PR §5.4 In-app browser row (gap cell: "ux — P2 (address-bar
history/Google fallback, 26.727; no WO yet — needs scoping)"); PR §8.2 item
12; Δ §5.2 ("annotated on the §5.4 In-app browser row (P2)"). WO-P1-002
(merged `d15333e`) closed the browser *discoverability* gap only — the
address-bar delta "is explicitly NOT in this order's scope"
(`docs/research/FEATURE-PARITY-WORK-ORDERS.md`, WO-P1-002 scope note).

### Reference behavior

1. **Baseline navigation affordances (26.721, in-target).** The in-app
   browser panel has an address bar with back/forward (Alt+←/→ on Windows),
   reload (Ctrl+R / Ctrl+Shift+R no-cache), and copy-URL (Ctrl+Shift+C,
   browser focused) `[historical-record]` (A §4 "Address bar / navigation";
   A shortcuts table rows marked ● behavior). Panel bindings: Open browser
   tab Ctrl/Cmd+T, Toggle browser panel Ctrl/Cmd+Shift+B `[historical-record]`
   (A §4). The browser itself is a native panel with bounded tabs,
   URL/navigation state, JPEG frame streaming, and keyboard/pointer/scroll
   input against a supervised isolated Edge/Chrome/Chromium profile
   `[historical-record]` (A §4).
2. **The 26.727 delta (the item-12 core, in-target).** Verbatim: "Type in the
   built-in browser's address bar to revisit pages from your browsing history
   or search Google when there's no match. Manage your browsing history in
   Settings, and let ChatGPT search that history when a task needs to find a
   page you visited before." `[docs-derived]` (CL, 2026-07-30 entry). Three
   sub-behaviors: (a) address-bar input is matched against **browsing
   history** and can revisit a previously visited page; (b) on **no match**,
   the input becomes a **Google search**; (c) browsing history is
   **persistent** (manageable in Settings) and **agent-searchable** ("let
   ChatGPT search that history"). A matrix summary: "address bar revisits
   pages from browsing history or searches Google when there's no match;
   browsing-history management in Settings; agent can search that history"
   `[docs-derived]` (A §4 delta column; PR §5.4).
3. **History granularity + persistence.** The official evidence does not
   state whether the history store is per-tab, per-panel, or global, nor its
   retention bounds — `[unverified]`. That it *persists across sessions* is
   implied by "manage your browsing history in Settings" (a settings-managed
   store is not per-tab ephemeral) `[docs-derived — explicit inference from
   CL wording]`. The browser runtime's `BrowserUser.history` capability is
   recorded at baseline as **gated to internal/Public-Beta builds in stable
   Prod** `[historical-record]` (A §4 "Browser state / capabilities") — the
   capability flag existed pre-26.727 behind a gate; 26.727 is when the
   user-facing address-bar behavior shipped.
4. **Agent-side surface.** "let ChatGPT search that history when a task needs
   to find a page you visited before" (CL) — the agent queries the same
   history store during tasks `[docs-derived]`. The fork runtime's
   browser-agent RPC surface contains a `getUserHistory` method name (it
   appears in Flauz's own interceptor table, src `browser.rs:2354`), which is
   the natural carrier for this behavior `[source-derived]` — its official
   request/response shape is `[unverified]`.
5. **Settings surface.** "Manage your browsing history in Settings" (CL) —
   a browsing-history management section in Settings (26.727) `[docs-derived]`.
   The same release also "Updated browser settings to show only supported
   browsers" (CL) — adjacent browser-settings churn. What the management UI
   offers (list? delete-by-item? clear-all? per-site?) is `[unverified]`.
   Note the baseline stable Settings registry (26 sections) has a
   `browser-use` section `[historical-record]` (A §9 Section registry) — the
   26.825-registry's exact section layout is `[unverified]` (no 26.825 asar
   inspection exists, VDN §E.1).
6. **URL display/edit affordances.** Baseline address bar shows the URL and
   is editable (navigate on submit); the copy-URL action is Ctrl+Shift+C
   with the browser focused `[historical-record]` (A §4). A current-docs
   binding "Go to line / focus address bar — Ctrl+L" exists and is
   post-baseline (no ●) `[docs-derived]` (A shortcuts table). Whether the
   official address bar displays a trimmed URL or offers
   completion/dropdown UI when matching history is `[unverified]` (the CL
   text says "revisit pages from your browsing history" without describing
   the affordance — dropdown, inline completion, or direct jump).
7. **Reference+1 runtime (26.908, NOT target).** The Linux preview's Ctrl+/
   shortcuts overlay lists "Reload Browser Page — Ctrl+R" and "Force Reload
   Browser Page — Ctrl+Shift+R" as named rows
   `[runtime-observed: linux-preview 26.908.70816]` (LX `vlm-reads.txt`
   READ 7; PR §5.4) — runtime-naming the baseline-evidenced reload behavior
   on a reference+1 build. The browser panel itself is auth-walled in the
   preview (no browser surface pre-auth, LX README finding 1), so no
   address-bar runtime observation exists at any version.

### Keyboard shortcuts (official)

| Action | Windows binding | Baseline (26.721)? | Provenance |
| --- | --- | --- | --- |
| Focus address bar (official row title: "Go to line / focus address bar") | Ctrl+L | No — post-baseline | `[docs-derived]` A shortcuts table |
| Browser back / forward | Alt+← / Alt+→ (+Mouse Back/Forward) | Yes (behavior ●) | `[historical-record]` A §4 + shortcuts table |
| Reload page | Ctrl+R | Yes (behavior ●) | `[historical-record]` A §4; runtime-named on 26.908 overlay (reference+1, LX READ 7) |
| Force reload (no-cache) | Ctrl+Shift+R | Yes (behavior ●) | `[historical-record]` A §4; runtime-named on 26.908 overlay (reference+1, LX READ 7) |
| Copy browser URL | Ctrl+Shift+C (browser focused) | Yes (●) | `[historical-record]` A shortcuts table |
| Toggle browser browse/comment mode | Ctrl+. | No — post-baseline | `[docs-derived]` A shortcuts table |
| Open browser tab / toggle browser panel | Ctrl+T / Ctrl+Shift+B | Yes (●) | `[historical-record]` A §4 |

**Flauz cross-check (`[source-derived]`, read at `3c9f113`):**
`focusBrowserAddressBar` already exists — registry row "Focus browser address
bar / Focus the in-app browser address bar", Navigation group, CmdOrCtrl+L
(`ui.rs:2956-2961`); palette entry (`ui.rs:3391`, title `ui.rs:3435`);
dispatch (`ui.rs:10436`); implementation fills the input with the active
browser URL, focuses it, and selects all (`ui.rs:7082-7102`). Address submit
(`ui.rs:5893-5912`): `browser_navigation_url` normalizes input — URL-shaped
input gets a scheme (`github.com/…` → `https://github.com/…`,
`localhost:3000` → `http://localhost:3000`), **non-URL input already becomes
a Google search** (`native rust client` →
`https://www.google.com/search?q=native+rust+client`; `javascript:` schemes
are forced into the search path), and an unchanged URL dispatches reload
instead of navigate (test `browser_address_matches_stable_url_and_search_
normalization`, `ui.rs:51004-51026`). Back/forward: Alt+←/→ rows
(`ui.rs:2962-2982`) ride per-tab CDP `Page.getNavigationHistory` /
`Page.navigateToHistoryEntry` (`browser.rs:3820-3847`, back/forward wrappers
`browser.rs:2138-2142`) — live per-tab navigation lists, **not** a persisted
browsing-history store. No reload/force-reload keybinding exists (reload only
via the panel toolbar button `ui.rs:26543` and unchanged-URL resubmit
`ui.rs:5906`); no copy-URL action exists (Ctrl+Shift+C is bound to Copy
working directory, `ui.rs:2837-2843`); the `getUserHistory` browser-agent RPC
is stubbed `method_not_found` (`browser.rs:2354`); no browsing-history
storage of any kind exists in the tree (only `download_history`,
`browser.rs:1984`); the Browser settings page carries site rules + Full CDP
switch, no history management (PR §5.4 "Browser permissions & settings").
B2 corroborates the panel-level state: "Address bar — FocusBrowserAddressBar
Ctrl+L … shortcut + panel" and "Navigation history — back/forward buttons +
Alt+Left/Alt+Right … in-panel" (B2 §4).

**Net reference delta for item 12** (official target minus Flauz today):
the **history-revisit step** of address-bar matching (typed input matched
against a persisted browsing history before the Google fallback fires —
Flauz's fallback is purely syntactic, no history lookup), the **persistent
browsing-history store** (per-tab CDP history is not it), the **Settings
history-management surface**, the **agent history-search** (`getUserHistory`
currently method-not-found), the **reload/force-reload keybindings**, and the
**context-scoped copy-URL** action. The Google fallback itself is *already
present* Flauz-side — the parity row's shorthand "address-bar
history/Google fallback" should not be read as "Google fallback missing".

### Empty / loading / error states

Official side: `[unverified]` — no layer describes the address-bar history
dropdown's empty/no-match presentation (the CL text implies Google search is
the no-match path), the history Settings page's states, or failure behavior
of history search. Baseline-evidenced browser error states remain in force
for the panel itself: download `failed`/`canceled` states with clean staged
transfer discard `[historical-record]` (A "Error states" table). Flauz-side
states for the *existing* surface: entry-surface toggles surface the honest
guard "Open a chat before opening the Browser." (post-WO-P1-002,
PR §5.4) `[runtime-observed]`.

### Current-version notes

- **Baseline 26.721:** address bar + back/forward + reload/no-cache +
  copy-URL present `[historical-record]` (A §4). No history-revisit, no
  Google fallback, no history management (the `BrowserUser.history`
  capability flag is gated to internal/Public-Beta in stable Prod at
  baseline) `[historical-record]` (A §4).
- **Current target 26.825:** history revisit + Google fallback + Settings
  management + agent history search, all since 26.727 `[docs-derived]` (CL;
  Δ §3.1; A §4 delta column). The 26.825 line adds browser *extensions*
  (Edge/Brave/Opera/Vivaldi), WebMCP site tools, and cloud browser sign-in
  (CL 2026-08-25) — separate §5.4 rows, not part of item 12.
- **Reference+1 (26.908+):** "browser tab width/scroll stability"
  improvements (VDN §B reference+1; Δ §4) — out of target. The Ctrl+R/
  Ctrl+Shift+R overlay naming (LX READ 7) is 26.908 runtime evidence of a
  baseline behavior, cited for naming only.

### OPEN QUESTIONS (R2)

1. **History-revisit affordance shape:** dropdown list, inline completion,
   or direct navigation on history match — `[unverified]`. **Upgrade path:**
   authenticated official-app run typing partial URLs/queries into the
   address bar with prior history seeded.
2. **History store granularity and bounds:** per-tab/per-panel/global;
   retention caps; whether incognito-style sessions exist — `[unverified]`.
3. **`getUserHistory` official contract:** request/response shape, paging,
   and whether the GUI history search and agent history search hit one
   endpoint — `[unverified]`; a 26.825 asar/schema inspection or an
   authenticated supervised-app-server probe would answer it.
4. **Settings management surface contents:** itemized list, delete-by-item,
   clear-all, per-site controls — `[unverified]`.
5. **Whether Ctrl+L was bound at baseline** (the row is post-baseline in the
   docs table, and "Go to line" hints at a shared terminal/browser chord) —
   `[unverified]`; also unresolved generally in VDN §E.4 (intro dates for
   post-baseline bindings).
6. **26.908+ watch:** whether the 26.908 tab-stability work changed
   address-bar behavior — reference+1, out of target, noted for planning
   only.

---

## R3 — Remaining stable palette commands, keyboard shortcuts, focus order, screen-reader labels, reduced motion (part of parity item 17)

Parity anchors: PR §5.10 "Keyboard and accessibility" row (gap cell: "P2
residuals (remaining stable commands, focus order, screen-reader labels); P3
(Clear terminal / font-size / file-tree shortcut deltas)"); PR §8.2 item 17
("remaining stable palette commands + focus order + screen-reader labels +
reduced-motion" listed among the ledger-enhancement residuals); PR §9
overrides 16-17 (runtime-surfaced shortcut delta; Ctrl+P silent no-op).

### Reference behavior — palette command inventory

1. **Palette mechanics (baseline, in-target).** Ctrl/Cmd+K,
   Ctrl/Cmd+Shift+P, Ctrl/Cmd+G open the palette; arrow selection, Enter,
   pointer activation, Escape close; Suggested order `New chat`, `Open
   folder`, workspace-aware `Search files`, then the dynamic `Settings`
   group; compact one-line row density unfiltered, descriptions when
   filtered `[historical-record]` (A §10 Command palette; PM "Keyboard and
   accessibility"). Baseline also carries the Ctrl/Cmd+P Search-files
   drill-in `[historical-record]` (A §10).
2. **Official stable registry (recovered subset, baseline).** The historical
   record enumerates the recovered command-menu rows: New standalone chat,
   Open folder, Back, Forward, Find, Toggle pin, Previous chat, Next chat,
   Toggle sidebar, Toggle bottom panel, Toggle Review panel, Commit or push,
   Create PR, Create draft PR, Create branch, Merge PR, Open PR on GitHub,
   Open review tab, Open terminal, Go to skills, Force reload skills, MCP,
   the supported Settings sections, Feedback, Log out, Process manager —
   plus the distinct `toggleReviewTab` row ("Toggle review / Show or hide
   Review for the current Git-backed chat", Panels group, unassigned
   default), `toggleMaximizeSidePanel` ("Toggle maximize side panel / Expand
   or restore the side panel", General fallback, unassigned), the six
   `git.*` rows (exact titles/descriptions, Project grouping, unassigned
   defaults, editable, command-menu IDs), `Approve request` / `Decline
   request` (editable Enter/Escape defaults, dispatch only while an approval
   card owns the active request), `copyDeeplink` (Ctrl/Cmd+Alt+L),
   `copySessionId` (Ctrl/Cmd+Alt+C), `copyWorkingDirectory`
   (Ctrl/Cmd+Shift+C), `forkThread`, `forceReloadSkills`, `openSkills`,
   `keyboardShortcuts`, `mcpSettings`, and `thread1`–`thread9` ("Go to chat
   N / Open the visible chat in this shortcut slot", Ctrl/Cmd+1–9)
   `[historical-record]` (PM "Keyboard and accessibility" row).
3. **Flauz palette today vs that registry (`[source-derived]`, `3c9f113`).**
   `PaletteCommand::ALL` = 51 commands (`ui.rs:3348-3400`; 45 → 51 with
   WO-P2-004's six settings entries, PR §5.10). It covers every row of the
   recovered stable subset above **except** — present in the official
   command menu per PM but absent from Flauz's *palette* (several exist in
   Flauz's editable shortcut registry only): **Toggle review (tab)**,
   **Toggle maximize side panel**, **Fork** (`forkThread`),
   **Copy deeplink / Copy session ID / Copy working directory**,
   **Approve request / Decline request**, **Rename chat** (official overlay
   row, LX READ 7), **Go to chat 1–9**. Delta names, evidenced one by one:
   registry-vs-palette presence per src (`ui.rs:2780-3290` registry ids
   `renameThread`, `copyDeeplink`, `copySessionId`, `copyWorkingDirectory`,
   `forkThread`, `toggleReviewTab`, `toggleMaximizeSidePanel`,
   `approval.approve/decline`, `thread1-9` — none of these have a
   `PaletteCommand` entry), official presence per PM row + LX READ 7
   (Rename chat) `[historical-record + source-derived]`.
4. **The "remaining stable commands" ledger item is NOT fully enumerated
   anywhere.** The historical record's open tail is "Add the remaining
   stable commands and Settings sections…" (PM) without a name list, and no
   layer contains a complete official palette dump at 26.721 or 26.825 —
   the recovered subset (§R3.2) is what the codexRS program verified.
   A complete 26.825 inventory is `[unverified]` (no 26.825 asar
   inspection exists — VDN §E.1; the authenticated palette is auth-walled —
   LX README finding 1). The 26.908 login-surface palette shows only the
   unauthenticated slice: Quick actions (New chat Ctrl+N, Open folder
   Ctrl+O), the dynamic Settings group (10 pages), "Panels: Open terminal
   (Ctrl+`)", and a Chats search group `[runtime-observed: linux-preview
   26.908.70816]` (LX README captures 02-04; PR §9 override 14).

### Reference behavior — keyboard-shortcut inventory

5. **Baseline-verified bindings (●).** The current-docs table with baseline
   markers (A §"Keyboard shortcuts", `[docs-derived]` + `[historical-record]`
   for ● rows) is the canonical inventory; ● rows include: Open command menu
   Ctrl+Shift+P/Ctrl+K; Open settings Ctrl+,; Open keyboard shortcuts Ctrl+/;
   Open folder Ctrl+O; Navigate back/forward Ctrl+[ / Ctrl+] (+Mouse
   Back/Forward); Toggle sidebar Ctrl+B; Toggle bottom panel Ctrl+J; New
   chat Ctrl+N/Ctrl+Shift+O; New standalone chat Ctrl+Alt+O; pin/rename
   Ctrl+Alt+P / Ctrl+Alt+R; Open side chat Ctrl+Alt+S; Search chats
   (assignable, none by default); Find in chat Ctrl+F (next/prev Ctrl+G /
   Shift+F3); Previous/next chat (Ctrl+Shift+Tab, Ctrl+Shift+[, Ctrl+PageUp /
   Ctrl+Tab, Ctrl+Shift+], Ctrl+PageDown); go-to-chat 1-9 Ctrl+1-9; model
   picker Ctrl+Shift+M; project picker Ctrl+Alt+Shift+O; Approve/decline
   Enter/Escape; Search files Ctrl+P; review tab/panel Ctrl+Shift+G /
   Ctrl+Alt+B; browser tab/panel Ctrl+T / Ctrl+Shift+B; browser
   back/forward/reload Alt+←/→/Ctrl+R; copy browser URL Ctrl+Shift+C
   (focused); copy conversation path (macOS-only); copy deeplink / session
   ID / working directory Ctrl+Alt+L / Ctrl+Alt+C / Ctrl+Shift+C.
6. **Post-baseline current-docs rows (in the 26.825 target, intro dates
   `[unverified]` — VDN §E.4).** Font size Ctrl++/-/0; Toggle terminal
   Ctrl+`; Clear terminal Ctrl+L/Ctrl+K (focused); Clear all unread
   indicators Shift+Esc; undo/redo Ctrl+Z/Ctrl+Y; Close tab/window
   Ctrl+W/F11/Ctrl+Q; quick chat Ctrl+Alt+N/Ctrl+Shift+N; Archive chat /
   mark unread Ctrl+Shift+A/Ctrl+Shift+U; Next chat needing attention
   Ctrl+Alt+A; open recent chat 1-6 Ctrl+Alt+1-6; voice chat/dictation
   Ctrl+Shift+V/D (feature in-baseline 26.715, binding not in baseline
   registry); Switch Chat/Work/Codex Alt+1/2/3 (same caveat); Toggle
   Activity view Ctrl+Alt+U; run environment action Win+Shift+D; Toggle file
   tree Ctrl+Shift+E; focus address bar Ctrl+L; browser browse/comment mode
   Ctrl+.; restore previous prompt ↑ (empty composer); Take an Appshot (both
   ⌘/Alt keys — 26.908 on Windows, reference+1) `[docs-derived]` (A
   shortcuts table).
7. **Reference+1 runtime overlay (26.908 — NOT the target inventory).** The
   Linux preview's Ctrl+/ overlay is a searchable "Keyboard shortcuts"
   surface ("Search shortcuts") with **22 rows** across Chat / Navigation /
   General: Chat — New chat Ctrl+N/Ctrl+Shift+O, Archive chat Ctrl+Shift+A,
   New standalone chat Ctrl+Alt+O, Toggle pin Ctrl+Alt+P; Navigation — Find
   Ctrl+F, Back Ctrl+[/Mouse Back, Forward Ctrl+]/Mouse Forward, Next/Prev
   recently viewed chat Ctrl+Tab/Ctrl+Shift+Tab, Switch to Work Alt+2;
   General — Close Tab Ctrl+W/Ctrl+F4, Copy deeplink Ctrl+Alt+L, Copy
   working directory Ctrl+Shift+C, Force Reload/Reload Browser Page
   Ctrl+Shift+R/Ctrl+R, Open command menu Ctrl+K/Ctrl+Shift+P, Rename chat
   Ctrl+Alt+R, Search Files… Ctrl+P, Show keyboard shortcuts Ctrl+/, Toggle
   File Tree Ctrl+Shift+E `[runtime-observed: linux-preview 26.908.70816]`
   (LX `vlm-reads.txt` READ 7; captures 05-07; PR §5.10). Clear-terminal and
   font-size rows do not appear in the captured sections (LX). This overlay
   **runtime-confirms** the docs-derived Toggle-file-tree binding and
   surfaces bindings absent from Flauz's registry/palette (PR §9 override
   16): recent-chat cycling Ctrl+Tab/Ctrl+Shift+Tab, Switch to Work Alt+2,
   Close Tab Ctrl+W/Ctrl+F4 twin, plus the R3.3 palette-absent names.
8. **Editable-shortcut system (baseline).** Official `/settings/
   keyboard-shortcuts`: text/keystroke-prefix search, capture with conflict
   feedback, set/replace/Shift-append/remove/per-command reset, confirmed
   reset-all, versioned overrides, two-chord sequences, dynamic titlebar
   labels `[historical-record]` (A §9). Stable grouping comparator:
   Chat → Navigation → Panels → Project → Skills → Configure → App →
   General `[historical-record]` (A §10 Accessibility; PM). Titlebar menus
   expose the first effective binding and remove unassigned labels
   `[historical-record]` (PM).
9. **Flauz shortcut surface today (for gap measurement).** Editable
   registry with 72 ids (`ui.rs:2780-3290`, 71→72 with side chats — PR §5.1
   Side chats row) including the stable comparator groups; Ctrl+/ overlay +
   searchable/editable settings page both runtime-confirmed
   `[runtime-observed]` (B2 §10; ev/11, ev/15); B2's 35-row inventory is the
   pre-wave visual confirmation set (B2 §"Keyboard-shortcut inventory").
   Known defects/absences (src + PR): **Ctrl+P is bound but dispatches into
   nothing** (`ui.rs:4527` binds OpenFileSearch; no handler registered —
   PR §9 override 17; palette "Search files" is the working entry); absent
   from the registry entirely: recent-chat cycling Ctrl+Tab/Ctrl+Shift+Tab,
   Switch to Work Alt+2, font-size Ctrl+±/0, Clear terminal Ctrl+L/K, Toggle
   file tree Ctrl+Shift+E, undo/redo Ctrl+Z/Y, quick-chat, mark-unread,
   open-recent-1-6, browse/comment mode Ctrl+., environment-action
   `[source-derived]` (src registry sweep; PR §9 override 16; PR §8.2 P3
   tail). Note the official model is **context-scoped chords** (Ctrl+L =
   clear-terminal when terminal focused, focus-address-bar otherwise;
   Ctrl+Shift+C = copy-URL when browser focused, copy-working-directory
   otherwise) `[historical-record + docs-derived]` (A) — Flauz's registry
   binds both chords globally to one meaning each (src), which any
   shortcut-parity order must reconcile.

### Reference behavior — focus order, screen-reader labels, reduced motion

10. **Focus order (baseline).** Official carries "complete focus order"
    across the main surfaces — stated as an official capability the codexRS
    program had not reproduced (A §10 Keyboard and accessibility row; PR
    §5.10). Evidenced specifics: focus traps on destructive confirmations —
    the Reset Memories and Reset-all-keyboard-shortcuts confirmations each
    receive focus once per opening and confine Tab/Shift+Tab to Cancel and
    Reset `[historical-record]` (A §10 Accessibility; PM); approvals
    approve/decline yield to modal/popup/select/elicitation/structured-input
    focus contexts `[historical-record]` (A "Error states" table); focus
    traps around archive deletion Cancel/Delete with sequential
    acknowledgement `[historical-record]` (PM Settings row, archived-chats
    section). The full per-surface Tab/arrow traversal contract (palette →
    sidebar → composer → timeline → inspector) is not written down anywhere
    as a spec — `[unverified]` beyond the rows above.
11. **Screen-reader labels.** Official "screen-reader labels" are recorded
    as an existing official surface not yet reproduced (A §10; PM open tail
    "complete focus order, screen-reader labels"). No layer captures the
    official labeling pattern itself (names/roles/announcements) —
    `[unverified]` in detail; the claim of existence is `[historical-record]`.
12. **Reduced motion.** Official Appearance settings include **reduced
    motion On/Off** `[historical-record]` (A §9 Appearance). Flauz today:
    reduced motion is functional On/Off — `On` immediately removes the
    Switch/Checkbox transitions and the scrollbar idle fade; legacy stored
    `System` values resolve to `Off` without rewriting; the native
    Windows/Linux shell does not yet expose a shared OS motion signal
    `[historical-record]` (PM Settings row) — i.e. manual toggle parity
    exists, OS-signal following does not (PM open tail: "OS-level
    reduced-motion following"; PR §5.10 pending list).

### Empty / loading / error states

Official palette: no-results state exists for the settings-search field
(A §9 Settings shell) and by extension the filtered palette keeps
descriptions when filtered (A §10); a palette-specific empty state is
otherwise `[unverified]`. Shortcuts overlay: searchable with a "Search
shortcuts" field; no-match presentation `[unverified]` (the 26.908 overlay
was captured only with content, LX READ 7). Editable-shortcut conflict
feedback on capture is the evidenced "error" behavior `[historical-record]`
(A §9). Focus-trap behavior on destructive confirmations is the evidenced
error-adjacent contract (§R3.10). Flauz-side: palette no-results state
runtime-confirmed ("No matches" pre-WO-P2-004, ev/18; post-WO-P2-004 all
default-nav sections resolve, `docs/research/evidence/wo-p2-004/`)
`[runtime-observed]`.

### Current-version notes

- **Baseline 26.721:** everything in §R3.2/§R3.5/§R3.8/§R3.10-12 marked ● or
  historical-record is in-baseline; the ●-marked table is the baseline
  inventory (A).
- **Current target 26.825:** adds the post-baseline current-docs rows
  (§R3.6) — intro dates `[unverified]` (VDN §E.4) — plus (by 26.727/26.825
  product evolution) the Activity-view/unread rows already covered in R1.
  The 26.825 palette/overlay micro-details are `[unverified]` (no 26.825
  bundle inspection; VDN §E.1).
- **Reference+1 (26.908+):** the 22-row runtime overlay (§R3.7) is 26.908
  evidence — forward-looking only; Pets quick-chat bindings (Option+Space /
  Win+Alt+P) and the Appshot chord are 26.908 features, out of target
  `[docs-derived]` (Δ §4). Any future order must not cite the overlay rows
  as the 26.825 bar without the version-skew label.

### OPEN QUESTIONS (R3)

1. **The complete official palette inventory at 26.825** — the single
   biggest unknown for "remaining stable palette commands": the recovered
   subset is historical-record-complete for what it lists, but the PM tail
   implies more rows existed than were recovered. **Upgrade path:**
   authenticated palette dump (all groups, filtered queries) on the
   operator's 26.825.51511, or a 26.825 asar inspection if a bundle ever
   becomes available.
2. **Whether the R3.3 palette-absent names are palette rows officially or
   registry-only** — e.g. "Go to chat 1-9", "Approve/Decline request":
   PM records them as editable commands; their command-menu presence is
   implied (git.* rows have "command-menu IDs") but not row-by-row
   evidenced — `[unverified]`.
3. **Focus-order contract detail** (per-surface traversal spec) and
   **screen-reader labeling pattern** — `[unverified]` beyond §R3.10-11; an
   accessibility-tree inspection (Windows UIA / macOS AX) on the official
   app would upgrade both.
4. **Intro dates for the post-baseline shortcut rows** (VDN §E.4) — needed
   to place each precisely at 26.727 vs 26.825; currently all in-target but
   undated.
5. **Context-scoped chord conflicts** (Ctrl+L, Ctrl+Shift+C, Ctrl+K) — the
   official precedence rules between terminal/browser/chat contexts are
   evidenced only by the row wordings ("(focused)", "(browser focused)") —
   `[unverified]` as a formal precedence spec.
6. **Whether official reduced-motion follows OS signals** (a "System" mode)
   — the official record shows On/Off only; Flauz's missing OS-following is
   a codexRS-side open item, but the official behavior is also not fully
   specified — `[unverified]`.

---

## SCOPING HINTS (advisory only — the Lead writes the orders)

**R1 — Activity view + unread attention.** The evidence supports a two-layer
split. Layer 1 is the *unread-attention state*: a per-chat unread/needs-
attention flag with the three bindings (Ctrl+Alt+U toggle view, Ctrl+Alt+A
next-needing-attention, Shift+Esc clear-all) and a visual treatment — this is
the load-bearing dependency the parity row already names ("unread-state
dependent") and it is independently testable without the view. Layer 2 is
the *Activity view itself* (bell entry point + sidebar view listing
recently-engaged chats needing attention) — bounded if built on layer 1's
store, with membership/ordering chosen conservatively where the reference is
`[unverified]` (recommend: recency-ordered, all-projects, open-on-click) and
honestly labeled as a chosen interpretation. A third, optional sliver is the
adjacent Ctrl+Shift+U mark-unread row — small, but it belongs to the same
unread store and would land naturally with layer 1. Anything needing the
official shape beyond this (grouping, per-row actions) should wait for the
authenticated re-run in R1's open questions rather than be guessed.

**R2 — Browser address-bar history / Google fallback.** The Google fallback
is already implemented Flauz-side (src `browser_navigation_url`), so the
order should be re-scoped to the actual delta, which splits cleanly into:
(a) a *persistent browsing-history store* (bounded, codexRS-owned storage
per the program's pattern) fed by browser navigations; (b) the
*history-revisit step* in address-bar matching (history lookup before the
existing Google fallback fires — smallest user-visible win, rides (a));
(c) a *Settings history-management surface* (Browser settings page section:
list + delete + clear-all); and (d) the *agent history-search* — defer or
stub, because the official `getUserHistory` contract is `[unverified]` and
the interceptor already returns method-not-found (a contract-free guess
here would violate the program's reference doctrine). Independently, the
*missing keybindings* (reload Ctrl+R / force-reload Ctrl+Shift+R, the
context-scoped copy-URL half of Ctrl+Shift+C) are a tiny, self-contained
binding order that could ship with (a)/(b) or alone. The open questions
(granularity, affordance shape) bound (a)/(b)'s design; recommend the Lead
pin conservative choices (global history, dropdown-on-match) and label them
as interpretations.

**R3 — Palette/shortcut/a11y residuals.** Three natural orders. Order one:
*palette delta names* — add the evidenced registry-but-not-palette rows
(Toggle review, Toggle maximize side panel, Fork, Copy deeplink/session ID/
working directory, Approve/Decline request, Rename chat, Go to chat 1-9) to
`PaletteCommand::ALL` with the PM-recorded titles/descriptions — a pure
catalog order in the WO-P2-004 pattern, plus fixing the Ctrl+P silent-no-op
wiring defect (override 17) which is one handler registration. Order two:
*shortcut-delta rows* — the post-baseline in-target bindings absent from
Flauz's registry (font-size, Clear terminal, Toggle file tree, undo/redo,
archive/mark-unread, recent-chat cycling, Close Tab twin, browse/comment
mode, open-recent-1-6…), which needs a prior Lead decision on
context-scoped chords (Ctrl+L/Ctrl+Shift+C/Ctrl+K conflicts) since Flauz's
registry is global-binding — that decision is the real scoping work, the
rows themselves are mechanical. Order three: *a11y pass* — focus-order
completion and screen-reader labels across the main surfaces (largest,
needs its own evidence run per surface; the official detail is thin, so
expect the order to be written against GPUI accessibility primitives plus
the evidenced focus-trap contracts) with reduced-motion OS-following as a
small separate item. The 26.908 overlay must stay a reference+1 annotation
in all three, not a target bar.

---

*Bounds: this document records official-side reference behavior only, from
the repo's own evidence layers plus read-only source reads at base
`3c9f113`. No authenticated official-app access existed for this research;
all auth-walled shapes are marked. 26.908+ observations are reference+1 and
are not part of the 26.825 target.*
