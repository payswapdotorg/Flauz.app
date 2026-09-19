# F1 closure packet — accessibility inventory (WO-F1-SWEEP-001)

- **Work order:** WO-F1-SWEEP-001 (Wave 1, Worker C — evidence-only)
- **Base:** `a664644718210e254952db518606d80906eee448`
- **Date:** 2026-09-19
- **Scope:** source-level audit of the keyboard-only / labels / focus /
  state-presentation surface against `docs/PRODUCT-UX-JOURNEYS.md` §7
  (accessibility and discoverability) and the parity-matrix keyboard rows.
  GPUI platform limits are recorded honestly: a platform-bound classification
  is a valid finding, not a hidden gap.

Classification vocabulary: `defect` (user-visible a11y break, anchored) ·
`gap` (missing affordance) · `platform-bound` (needs upstream GPUI/shell) ·
`pass`. Anchors: `PM` = docs/parity-matrix.md, `PR` = parity report,
`FW` = work-orders doc, source = `symbol (file:~line)` (line numbers volatile;
symbol is the stable anchor).

## (a) Palette / route / overlay surfaces — keyboard-only open, navigate, close

| Surface | Open | Navigate | Close | Verdict | Anchors |
| --- | --- | --- | --- | --- | --- |
| Command palette (Unified) | Ctrl/Cmd+K, Ctrl/Cmd+Shift+P (KeyBindings) | MoveUp/MoveDown actions → `move_selection` (wrapping, index-safe); Enter via `InputEvent::PressEnter` → `activate_selected` | Escape action → `close_and_clear` | **pass** | `OpenCommandMenu` bindings (ui.rs:5056–5057); input auto-focus on open (ui.rs:8952–8958, `open_command_palette` deferred `input.focus`); MoveUp/MoveDown handlers (ui.rs:5027–5034); `move_selection` (ui.rs:4639–4647); PressEnter (ui.rs:4136); `activate_selected` (ui.rs:4650); Escape handler (ui.rs:5026–5028) |
| Command palette — focus after close | — | — | `close_command_palette` drops the palette entity and restores composer file-search state but performs **no explicit focus restore** to the previously focused surface | **gap** (focus destination after close is unmanaged in source; the runtime landing point is source-unverifiable — a lab probe with focus visualization would settle it) | `close_command_palette` (ui.rs:8961–8971) |
| Files palette (Ctrl+P) | `searchFiles` interceptor arm (WO-P2-007) | same palette machinery | Escape byte-identical (D10b lab evidence) | **pass with a documented defect state** — see F-A4 under (e) | PM L112; FW WO-P2-007; no-workspace early-return (ui.rs:8943–8945) |
| Chat-search palette (Ctrl+G) | `OpenChatSearch` binding | "Back to commands" keyboard-selectable row | Escape | **pass** | ui.rs:5058; back row (ui.rs:4906–4924) |
| Keyboard-shortcuts overlay (Ctrl+/) | `ShowKeyboardShortcutsShortcut` binding; opens with deferred search focus | searchable list | first Escape clears the query and stays open; second Escape closes — matches the stable contract | **pass** | ui.rs:5097; `toggle_keyboard_shortcuts` (ui.rs:10460–10466); `handle_keyboard_shortcuts_escape` (ui.rs:10468–10478); PM L113 |
| Settings routes | Ctrl/Cmd+comma → General; sidebar entry; palette Settings group (all default-nav sections indexed, WO-P2-004) | Settings search field focused by Ctrl/Cmd+F; Escape returns | Back to app / Escape | **pass** (matrix-anchored) | ui.rs:5096; PM L101 ("bounded search field focused by Ctrl/Cmd+F", "Escape return" exercised); `settings_search: InputState` (ui.rs:5631) |
| Terminal dock | Ctrl/Cmd+J toggles an existing Bottom tab (never creates); Ctrl/Cmd+` opens terminal; registry `toggleTerminal` | tab strip; session selection | hide/close paths | **pass with note** (keyboard-openable and keyboard-driven; explicit focus transfer into the PTY on open was not source-verified — runtime probe) | ui.rs:5092–5094; `render_terminal_dock` / `render_terminal_tab_strip` (ui.rs:26366/26596); PM L88 |
| Browser panel | `toggleBrowserPanel` Ctrl+Shift+B, `openBrowserTab` Ctrl+T, `focusBrowserAddressBar` Ctrl+L (registry defaults) | browser pane key handling (`handle_browser_key_down`) incl. WO-P2-011 context-scoped chords | panel toggle | **pass** | registry defaults (ui.rs:3810–3812); `handle_browser_key_down` (ui.rs:7549); FW WO-P2-011 |
| Destructive confirmation modals (6 evidenced traps) | opened from their flows | tab / shift-tab confined to Cancel/confirm inside the modal KeyContext | Escape where bound | **pass** | tab/shift-tab KeyBindings for RemoveLocalProjectModal, DeleteArchivedTasksModal, ResetMemoriesModal, ClearBrowsingHistoryModal, ResetKeyboardShortcutsModal, AllowAllBrowserSitesModal (ui.rs:5112–5156); focus handles + once-per-open request flags (ui.rs:5717–5735; request pattern at ui.rs:43539+); Escape bindings for AboutDialog / McpElicitation / StructuredUserInput (ui.rs:5099–5101); PM L101/L113 (focus-trap contract) |
| Remaining confirmation modals (remote pairing / remote confirmation / account logout / plugin install) | opened from their flows | focus handles exist; **no tab/shift-tab trap KeyBinding found for these contexts** in the bind_keys block | — | **gap** (extend or explicitly waive the trap pattern for these four; verify at runtime) | focus handles (ui.rs:5716–5735) vs trap bindings (ui.rs:5112–5156) |
| Approvals (Enter/Escape) | `approval.approve` Enter / `approval.decline` Escape, editable | yields to native modal, popup-menu, select, MCP-elicitation, and structured-input focus contexts instead of approving a background request | — | **pass** (the yielding behavior is the focus-order safety property) | PM L112 + L83 |

## (b) Visible labels vs icon-only controls

| Control | Presentation | Verdict | Anchors |
| --- | --- | --- | --- |
| Sidebar chat rows | status icon + title + relative time, all textual/shape | **pass** | row render (ui.rs:14732–14786) |
| Palette rows / group headings | text title + description + text headings | **pass** | `render_command` (ui.rs:4678+); `group_heading` (ui.rs:~4868) |
| Unread-attention dot | 6 px dot only — no text label, tooltip, or accessible name | **gap** (state is NOT color-only — see (c) — but it is unlabeled for assistive tech and undescribed for low-vision users) | dot render (ui.rs:14774–14782) |
| Archived-chats single deletion | icon-only destructive action | **gap** (no visible label on a destructive control) | PM L101 ("icon-only single deletion") |
| Titlebar window controls (minimize/maximize/close) | glyph-only conventional chrome; GPUI `window_control_area` exposes no accessible name at this layer | **platform-bound** for AT naming; visually conventional (minor) | `render_title_bar_control` (ui.rs:14015–14061) |
| Command palette entry affordance | keyboard shortcuts only — no visible toolbar button | **gap** (pointer-only users and AT users who do not know the chord cannot open the palette from visible chrome; also a discoverability contract issue, PRODUCT-UX-JOURNEYS §1 layer 1) | PR §5.10 Keyboard row UX-parity cell ("no toolbar button for palette") |

## (c) Non-color-only state presentation

| State | Channels | Verdict | Anchors |
| --- | --- | --- | --- |
| Unread attention | dot (shape) **plus** medium font weight on the title | **pass** (two non-color channels; label gap recorded in (b)) | weight (ui.rs:14765–14767); dot (ui.rs:14774–14782) |
| Chat/task status | status icon (shape) + status color | **pass** | `task_status_icon` (ui.rs:14728) |
| App-server connection | text labels ("Offline" / "Connecting…" / "App-server online" / "Reconnecting…" / "Retry N in Ns" / failed) + color + bounded tooltip | **pass** (text-first) | `render_sidebar_footer` (ui.rs:14819–14860) |
| Find active occurrence | stronger active-occurrence styling over the match | **pass with note** (color + emphasis; the runtime non-color distinction is source-unverifiable — lab probe) | PM L112 |
| Diff add/remove | color plus optional symbol diff markers preference | **pass** | PM L101 (Appearance preferences: "color/symbol diff markers") |

## (d) Focus order stability on surface open/close

| Behavior | Verdict | Anchors |
| --- | --- | --- |
| Palette open focuses input (deferred, one frame) | **pass** | ui.rs:8955–8958 |
| Palette close does not explicitly restore prior focus | **gap** (the same finding as (a)) | ui.rs:8961–8971 |
| Shortcuts overlay open focuses search (deferred) | **pass** | ui.rs:10460–10466 |
| Confirmation modals receive focus once per opening; Tab/Shift+Tab confined (6 evidenced traps) | **pass** | ui.rs:43539+ (request-once pattern); ui.rs:5112–5156 |
| Local project rows as native tab stops with Enter/Space activation and high-contrast focus outline | **pass** (bounded scope — this is the shipped baseline) | PM §Keyboard accessibility baseline (L128–130) |
| Broader focus order across all surfaces | **gap** (explicitly open) | PM §Keyboard accessibility baseline (L130–131: "Broader focus-order, screen-reader, reduced-motion, and contrast work remains open"); PM L112 remainder |

## (e) Bounded status messages

| Behavior | Verdict | Anchors |
| --- | --- | --- |
| Status messages are bounded (16 KiB) and rendered as a bottom banner | **pass** | `Action::SetStatus` → `bounded_string(message, 16 * 1024)` (lib.rs:20311–20313); banner reservation (ui.rs:43767–43770) + render (ui.rs:43864); 33 dispatch sites in ui.rs |
| Honest (non-silent) outcomes for guarded actions | **pass** where delivered (attention bindings' honest no-selection statuses; browser chord guards "Open a page before reloading the Browser." / "Open a page before copying the Browser address.") | FW WO-P2-008; FW WO-P2-011 (ui.rs:10360, 10777 region) |
| Silent no-op family F-A1/F-A2/F-A3/F-A6/F-D1/F-D2 (archive / pin / rename / git.commit-with-pending-PR / typed /review unavailable / /compact not-ready — advertised chords that dispatch with zero feedback in their guarded states) | **defect** (user-visible; remediation in-flight on the WO-P2-012 branches — "Select a chat before archiving it." etc. observed in the branch diff) | FW WO-R-SWEEP findings of record; PR §5.10 Gap cell; WO-P2-012 branches (`origin/feat/wo-p2-012-guard-honesty` `0547055`, `-r2` `4fc763a`) |
| F-A4: Ctrl+P stays silent with NO workspace open (Files palette early-return by design) | **defect** (documented residual; keyboard user presses an advertised chord with zero feedback on the bare entry surface; parity treatment of the no-workspace state is open scope — WO-P2-012 territory) | ui.rs:8943–8945; FW WO-P2-007 known limitations (F-A4) |
| F-A5: thread1–9 empty slots do nothing | **pass** (matches the official "safely do nothing" contract — by design) | PM L112; FW WO-R-SWEEP |
| Status announcement to assistive tech (live-region equivalent) | **platform-bound** (no live-region/announcement concept at the current GPUI layer) | — |

## (f) Reduced motion / contrast — GPUI controllability, honestly recorded

| Area | State | Verdict | Anchors |
| --- | --- | --- | --- |
| In-app reduced-motion toggle | `ReducedMotionPreference` On/Off; `On` immediately removes the used Switch and Checkbox transitions and the scrollbar idle fade via `gpui_component::motion::set_reduced_motion` | **pass** | enum (lib.rs:456); `reduced_motion_enabled` (ui.rs:506–507); apply (ui.rs:565–566); Settings control with On/Off labels (ui.rs:33672–33692); PM L101 |
| OS-level reduced-motion following | legacy stored `System` values resolve to `Off` and are not rewritten, because the native Windows/Linux shell exposes no shared OS motion signal | **platform-bound** (not implementable at the current GPUI/shell layer — recorded as a platform-bound classification, not a hidden gap) | PM L101; ui.rs:506–507 |
| Contrast control | Appearance contrast preference feeds theme mixing (`normalized_appearance_contrast`) | **pass** (control exists) | ui.rs:670–690; PM L101 |
| Full contrast parity across surfaces | explicitly open | **gap** | PM L112 remainder ("contrast"); PM §Keyboard accessibility baseline (L130–131) |
| Screen-reader labels / AT tree exposure | GPUI exposes no accessibility tree on Windows/Linux at the current layer | **platform-bound** (needs upstream GPUI; the parity intent is P2 — see parity-inventory §2 row 38) | PM L112 remainder; PM §Keyboard accessibility baseline |
| UI/code font sizes | bounded adjustable sizes | **pass** (partial low-vision support) | PM L101 |
| Shortcut editing accessibility | capture with conflict feedback; reset confirmations focus-trapped | **pass** | PM L113; ui.rs:5112–5156 |

## Findings summary

| Class | Count | Items |
| --- | --- | --- |
| `defect` | 2 | F-A4 silent Ctrl+P without workspace; F-A1/A2/A3/A6/F-D1/D2 silent-no-op family (remediation in-flight, WO-P2-012) |
| `gap` | 7 | palette focus restore on close; palette visible entry affordance; attention-dot label; archived icon-only delete label; tab-trap coverage for the four non-evidenced confirmation modals; broader focus order; full contrast parity |
| `platform-bound` | 4 | screen-reader labels / AT tree; OS-level reduced-motion following; status-message AT announcement; titlebar-control AT naming |
| `pass` | 18 | see sections (a)–(f) |

## Source-unverifiable notes

1. Runtime focus landing point after palette/overlay close (needs a lab probe
   with focus visualization; RWO-021's a02/a03 byte-identical open/close
   captures evidence layout stability, not focus order).
2. Terminal open → PTY focus transfer.
3. Find active-occurrence non-color distinction at runtime.
4. The four non-evidenced confirmation modals' actual Tab behavior at runtime
   (source shows focus handles but no trap bindings).
