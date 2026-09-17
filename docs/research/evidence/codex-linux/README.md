# WO-LAB-001 evidence — Official Linux preview app, runtime-observed (2026-09-17)

The official **ChatGPT/Codex desktop app for Linux (preview)** was downloaded
(selective userspace extraction from the signed `.deb`), launched in
LINUX_GUI_LAB (Xvfb :102 + picom, 1600x1000, isolated HOME/XDG, no
credentials), driven through three scene scripts, and VLM-verified. This
upgrades the official Linux-A evidence layer from evidence-layers-only
(`[historical-record]`/`[docs-derived]`) to **`[runtime-observed]`** for every
surface below — bounded to the **unauthenticated slice**.

**Version skew (must be cited with every row):** package
`chatgpt 26.908.70816` (`latest`, built 2026-09-14). The parity target
remains the operator-installed **26.825.51511**; 26.908 is the "reference+1"
line (Pets quick chat, Windows Appshots). Observations are labeled
`[runtime-observed: linux-preview 26.908.70816]` and any 26.908-only feature
is called out. The installed-26.825 GUI remains unverified on its exact
build; rows keep their version-skew notes.

## Captures (all 1600x1000 root frames; app window 1090x760 at +255+120)

| File | Surface | Provenance |
| --- | --- | --- |
| `01-login-surface.png` | Login surface: OpenAI logo, "Sign in to ChatGPT", "Continue to sign in", "Sign in with an API key", "Sign up"; min/close window controls | `[runtime-observed: linux-preview 26.908.70816]` |
| `02-palette-terminal-query-auth-pending.png` | Command palette query "terminal" → "Panels: Open terminal (Ctrl+`)" + empty "Chats" group; background auth-pending surface: "Continue signing in with your browser" / "Cancel sign-in" / "Browser didn't open?" / "Copy sign-in link" | same |
| `03-command-palette-top.png` | Ctrl+K palette at login: placeholder "Search chats or run a command"; "Quick actions": New chat (Ctrl+N), Open folder (Ctrl+O); "Settings": General, Import, Appearance, Voice, Pets, Git, Connections, Environments, Worktrees, Configuration | same (Pets = 26.908 feature) |
| `04-command-palette-import-query.png` | Palette query "import" → Settings: "Import" (dynamic settings indexing runtime-confirmed) | same |
| `05-shortcuts-overlay-top.png` | Ctrl+/ "Keyboard shortcuts" overlay: search field "Search shortcuts"; Chat + Navigation sections | same |
| `06-shortcuts-overlay-scrolled.png` | Overlay scrolled: "General" section (Close Tab, Copy deeplink, Copy working directory, Reload/Force Reload Browser Page, Open command menu, Rename chat, Search Files..., Show keyboard shortcuts, Toggle File Tree) | same |
| `07-shortcuts-overlay-scrolled2.png` | Overlay scrolled further (same inventory, bottom rows) | same |
| `08-db-error-dialog.png` | DB-recovery dialog elicited by the lab's missing-native state: "ChatGPT cannot access its local database." + DB path `$HOME/.codex/sqlite/codex-dev.db` + "Error: better-sqlite3 is only bundled with the Electron app" + Retry / Back Up and Rebuild / Quit | same (lab-elicited) |
| `09-noprobe-ctrl-shift-o.png` | Ctrl+Shift+O at login: byte-identical frame to siblings (silent no-op at the unauthenticated surface; also Ctrl+F, Ctrl+Alt+O) | same |

## Reproducibility

- `official-session.sh` — session lifecycle (Xvfb :102 + picom; isolated
  HOME/XDG_DATA_HOME/XDG_CONFIG_HOME/XDG_CACHE_HOME; `ChatGPT --no-sandbox
  --disable-gpu --ozone-platform=x11 --lang=en-US`; lavapipe Vulkan ICD).
- `scene-a1-baseline.sh`, `scene-a2-deeper.sh`, `scene-a3-overlay.sh` — the
  three scenes (capture naming a1-/a2-/a3-).
- `deb-metadata.txt` — package control, payload layout, runtime requirements.
- `official-app-log-extracts.txt` — the fatal-without-runtime log (run 2),
  app-server stdio spawn + window-ready log (run 5), on-device-model/Vulkan
  fallbacks (run 1).
- `vlm-reads.txt` — full VLM-read transcripts.
- App userspace: `/home/z/parity-lab/official-app/usr/lib/chatgpt/`
  (selective extract: ChatGPT, codex-launcher, app.asar, app.asar.unpacked,
  codex, codex-code-mode-host, rg, paks, icu, locales/en-US, swiftshader).

## Load-bearing findings (feeding the parity report)

1. **Auth gates the entire shell.** Unauthenticated Linux shows ONLY the
   login surface (+ palette + shortcuts overlay). No sidebar, no composer,
   no terminal/browser affordances are visible pre-auth. Official
   "persistent affordance shapes" therefore remain **auth-walled** on the
   official side; Flauz's unauthenticatable-but-explorable shell is a real,
   now runtime-evidenced difference (Flauz renders the full shell + honest
   sign-in card on send).
2. **Official startup hard-depends on the bundled codex runtime**
   (`resources/codex`, app-server via stdio). Without it: fatal error, NO
   window ("Unable to locate the Codex CLI binary or required runtime
   components"). Flauz degrades gracefully instead ("Resolving…", auto-retry,
   ev/24) — Flauz is ahead on this resilience axis.
3. **Command palette indexes settings pages at login** (Settings group:
   General, Import, Appearance, Voice, Pets, Git, Connections, Environments,
   Worktrees, Configuration) — runtime-confirms the WO-P2-004 reference on
   Linux. Palette also carries "Panels: Open terminal (Ctrl+`)" and a
   "Chats" search group.
4. **Searchable "Keyboard shortcuts" overlay exists (Ctrl+/)** with 22
   runtime-verified rows (Chat/Navigation/General) — including bindings
   absent from Flauz's registry: Search Files Ctrl+P, Toggle File Tree
   Ctrl+Shift+E, Copy deeplink Ctrl+Alt+L, Copy working directory
   Ctrl+Shift+C, Rename chat Ctrl+Alt+R, Reload/Force Reload Browser Page
   Ctrl+R/Ctrl+Shift+R, Close Tab Ctrl+W. Flauz has no shortcuts overlay
   surface — new gap surfaced by this run.
5. **DB contract:** `$HOME/.codex/sqlite/codex-dev.db` (better-sqlite3);
   recovery surface "Back Up and Rebuild" with exact copy captured.
6. **Terminal backend:** node-pty (payload); device kit
   (@worklouder/device-kit-oai with serialport + node-hid) shipped in
   payload. cua_node payload (~166 MB) present although the official docs
   state Computer Use is NOT available in the Linux preview (macOS/Windows
   only) — payload presence ≠ feature availability; the docs statement
   stands as the official claim.
7. **Window/userData identity:** class "ChatGPT", userData
   `$XDG_CONFIG_HOME/ChatGPT` with a late (denied) request for `…/Codex`;
   `codex://` protocol registration attempted at startup.
8. Login-surface binding no-ops (byte-identical frames): Ctrl+Shift+O,
   Ctrl+F, Ctrl+Alt+O.

## Bounds (unchanged platform honesty)

- Unauthenticated slice only — account surfaces (sidebar, chats, projects,
  terminal/browser panels, settings pages content, Work/Pets surfaces)
  remain `[auth-walled]` for the official side.
- Linux slice only — Windows/macOS cells stay `[historical-record]`/
  `[docs-derived]`.
- Preview-quality build; 26.908.70816 vs 26.825.51511 skew noted above.
