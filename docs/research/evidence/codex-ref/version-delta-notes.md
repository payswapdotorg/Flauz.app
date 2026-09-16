# Version delta notes — baseline 26.721.3996.0 → current 26.825.51511

Task 41-A raw research notes (Worker A, Official Codex Reference Lab).
Date: 2026-09-16 (UTC). These notes feed
`docs/research/CODEX-FLAUZ-CURRENT-VERSION-DELTA.md` (to be produced at wave
convergence). Provenance labels per the lab scheme; every external claim
carries its URL.

---

## A. Runtime-observed: official CLI 0.146.0-alpha.3.1 surface (baseline runtime)

Executed 2026-09-16 with `CODEX_HOME=/tmp/qa-codex-home` (isolated; a bare
run writes `~/.codex` — forbidden). Binary:
`/home/z/parity-lab/tools/codex-cli/codex-x86_64-unknown-linux-musl`
(SHA-256 pinned in `reference/stable-26.721.3996.0/manifest.json`).

- `codex --version` → `codex-cli 0.146.0-alpha.3.1`.
- 24 subcommands (root `--help`): `exec`, `review`, `login`, `logout`, `mcp`,
  `plugin`, `mcp-server`, `app-server`, `remote-control`, `completion`,
  `update`, `doctor`, `sandbox`, `debug`, `apply`, `resume`, `archive`,
  `delete`, `unarchive`, `fork`, `cloud`, `exec-server`, `features`, `help`.
- Notable root flags: `--enable/--disable <FEATURE>`
  (= `features.<name>=true|false`), `--remote ws://|wss://|unix://|unix://PATH`
  (connect the TUI to a remote app server) + `--remote-auth-token-env`,
  `--strict-config`, `--search` (native Responses `web_search` tool, no
  per-call approval), `--no-alt-screen`, `--oss --local-provider
  lmstudio|ollama`, `-p <profile>` layers `$CODEX_HOME/<name>.config.toml`
  (config profile v2), `--add-dir <DIR>` (extra writable dirs),
  `--dangerously-bypass-approvals-and-sandbox`, `--dangerously-bypass-hook-trust`.
  Approval policies: `untrusted`, `on-request`, `never` (note: baseline CLI
  dropped `on-failure`, which E2B's 0.11.0 build still had).
- `codex app-server --help`: subcommands `daemon`, `proxy`, `generate-ts`,
  `generate-json-schema`; options `--listen stdio://|unix://|unix://PATH|ws://IP:PORT|off`
  (default `stdio://`), `--stdio`, `--analytics-default-enabled`
  (first-party default; users opt out via `[analytics] enabled=false`), and a
  full WebSocket-auth family (`--ws-auth capability-token|signed-bearer-token`,
  `--ws-token-file`, `--ws-token-sha256`, `--ws-shared-secret-file`,
  `--ws-issuer`, `--ws-audience`, `--ws-max-clock-skew-seconds`).
- `codex exec --help`: subcommands `resume`, `review`; flags `--ephemeral`
  (no session persistence), `--ignore-user-config`, `--ignore-rules`
  (skip execpolicy `.rules`), `--output-schema <FILE>` (JSON Schema for the
  final response), `--json` (JSONL events), `-o/--output-last-message`,
  `--skip-git-repo-check`.
- `codex resume --help`: picker by default; `--last`, `--all` (disables cwd
  filtering, shows CWD column), `--include-non-interactive`; session id or
  name (UUID precedence).
- `codex login --help`: `status`, `--with-api-key` (stdin),
  `--with-access-token` (stdin), `--device-auth`.
- **App-server JSON schema** (`app-server generate-json-schema --out`):
  - stable bundle: 82 definitions; `ClientRequest` = **89** methods;
    `ServerRequest` = **10** methods; `ServerNotification` = **70** types.
  - v2 bundle: 534 definitions; `ClientRequest` = 89 methods (identical set
    to stable); `ServerNotification` = 70.
  - Full 89-method list preserved in `cli-runtime-surface.txt` (same
    directory). Highlights beyond the codexRS-used set: `command/exec*`
    (exec/resize/terminate/write), `fs/*` (copy, createDirectory,
    getMetadata, readDirectory, readFile, remove, unwatch, watch, writeFile),
    `experimentalFeature/{enablement/set,list}`, `externalAgentConfig/*`,
    `plugin/share/*` (checkout, delete, list, save, updateTargets),
    `plugin/installed`, `plugin/skill/read`, `modelProvider/capabilities/read`,
    `account/{rateLimitResetCredit/consume, sendAddCreditsNudgeEmail,
    usage/read, workspaceMessages/read}`, `skills/extraRoots/set`,
    `thread/{approveGuardianDeniedAction, inject_items, metadata/update}`,
    `windowsSandbox/{readiness,setupStart}`, `mcpServer/tool/call`,
    `config/value/write`, `review/start`, `attestation/generate` (server→client).
  - Measurement note: the parity matrix's `[historical-record]` "experimental
    schema: 126 client request methods" was **not** reproduced by the schema
    generator (89 in both bundles). The extra experimental methods are
    presumably negotiated at runtime behind `experimentalApi: true`. Recorded
    as a measurement difference, not a contradiction.
- **Feature flags** (`codex features list`; 37 enabled): stable+true include
  `apps`, `auth_elicitation`, `browser_use`, `browser_use_external`,
  `browser_use_full_cdp_access`, `code_mode_host`, `computer_use`,
  `enable_request_compression`, `fast_mode`, `goals`, `guardian_approval`,
  `hooks`, `image_generation`, `in_app_browser`, `mentions_v2`, `multi_agent`,
  `personality`, `plugin_sharing`, `plugins`, `remote_compaction_v2`,
  `remote_plugin`, `shell_snapshot`, `shell_tool`,
  `skill_mcp_dependency_install`, `skill_search`, `tool_call_mcp_elicitation`,
  `tool_suggest`, `unified_exec`, `workspace_dependencies`. `memories` is
  stable but default **false**. Under development: `artifact`, `chronicle`,
  `code_mode*`, `exec_permission_approvals`, `external_agent_memory_import`,
  `mcp_2026_07_28`, `multi_agent_v2`, `realtime_conversation`,
  `standalone_web_search`, `token_budget`, … Full list:
  `features-list-0.146.0-alpha.3.1.txt`.
- **Doctor** (`codex doctor`): reports auth file `auth.json`; state DBs
  `state_5.sqlite`, `logs_2.sqlite`, `goals_1.sqlite`, `memories_1.sqlite`,
  `thread_history_1.sqlite`; app-server daemon dir (settings.json, pid files,
  control socket; "ephemeral mode" when not running); default sandbox
  "restricted fs + restricted network · approval OnRequest"; wire API
  `responses`; WebSocket preferred with HTTPS fallback; in this sandbox the
  model endpoint handshake returned region-403 (lab bound, no auth). Full
  output: `doctor-0.146.0-alpha.3.1.txt`.

Prior-session E2B evidence (codex-cli **0.11.0**, openai-codex template,
`/home/z/my-project/e2b-evidence/workspace-a*.json`) `[historical-record]`:
subcommand set was much smaller (exec/login/mcp/proto/completion/debug/apply),
approval policies still included `on-failure`, sandbox values
read-only/workspace-write/danger-full-access, model `codex-mini-latest`,
no app-server/plugin/features/doctor/resume/fork surface. Confirms the
runtime surface grew massively 0.11 → 0.146.

## B. Docs-derived: desktop app delta timeline (official changelog)

Source: `https://learn.chatgpt.com/docs/changelog` (retrieved 2026-09-16).
Curated verbatim extract (2026-09-14 → 2026-07-23):
`changelog-delta-extract.md` in this directory.

### Version-line mapping (inference, stated explicitly)

Desktop releases with explicit version numbers in the changelog: 26.707
(2026-07-09), 26.715 (2026-07-23), 26.727 (2026-07-30), 26.908 (2026-09-11).
The current installed reference **26.825.51511** carries no dated changelog
entry of its own; by the weekly `26.<MMDD>`-style cadence it maps to the
**2026-08-25** release line ("Browser extensions, site tools, and cloud
sign-in"). This mapping is an inference from the version scheme; labeled
`[docs-derived]` with this caveat. The baseline **26.721.3996.0** (captured
2026-07-24) sits between 26.715 (in-baseline) and 26.727 (post-baseline).

### Delta entries (post-baseline → current 26.825)

| Date | Entry | Parity-relevant content |
| --- | --- | --- |
| 2026-07-30 | **26.727** — Browser upgrades, multi-repository review, and image editing | Address bar revisits history / Google fallback; browsing-history management in Settings; agent history search; Chrome extension tab mentions + page-text into side chat; YouTube Q&A; right-click Ask ChatGPT; multi-repo review for multi-folder projects (lines changed per repo, `Review` across repos); generated-image editing (expanded viewer, Focused/Canvas views, comments, targeted edits); **Activity view** (bell, Cmd/Ctrl+Opt+U); Windows long-path install reliability; "Record & Replay" skill-from-demo entry appears adjacent (demonstrate a workflow → reusable skill; Computer Use enabled) |
| 2026-07-29 | Sign in with ChatGPT (beta) | Plugin/partner-site OAuth via ChatGPT identity (Airtable, GitLab, HubSpot, Notion, Supabase, Vercel); partner receives name/email/picture; plugin access still approved separately |
| 2026-07-31 | GPT-5.4 / GPT-5.4 mini retirement | Unavailable in Codex (ChatGPT sign-in) from 2026-08-31; remain on API/API-key sessions |
| 2026-08-10 | Daybreak Blue / Daybreak Red | Two cyber access tiers (Trusted Access for Cyber); Blue = defensive work with GPT-5.6 Sol; Red = GPT-5.6 Cyber for authorized offense; least-privilege profiles + Auto-review |
| 2026-08-11 | **Linux desktop preview + agent imports** | Official ChatGPT desktop app for Linux in **preview**: Ubuntu/Debian/Fedora, x64 + ARM64, `.deb`/`.rpm` (current docs also cover Arch via script; Ubuntu 24.04/26.04, Debian 13, Fedora 43/44); sign in → projects, local files, Codex. **Settings > Import**: Claude Code, Claude Cowork, Cursor (instructions, settings, skills, plugins, projects, recent work) with automatic-update sync; CLI `/import` for Claude Code + Cursor |
| 2026-08-13 | **Computer History** | macOS, Pro/Business/Enterprise, opt-in: app/web activity → memories + timeline for ChatGPT and Codex; choose contributing apps/sites; pause; review/delete; EEA/UK/CH added 2026-08-20 |
| 2026-08-17 | Public plugin catalog CSV export | Enterprise owners/admins: CSV of public plugins visible to the workspace |
| 2026-08-19 | GitLab support in Codex cloud (beta) | Connect GitLab projects, environments, tasks from issues/MRs with `@codex`, one-off/automatic MR reviews (cloud side) |
| 2026-08-20 | Codex and ChatGPT updates | Apple Messages plugin (macOS; approval-gated send); Site co-editing (workspace editors); editable Site URLs; Computer History EEA/UK/CH; **shared thread snapshots** (read-only local-thread share, secret-pattern redaction, view/revoke under data controls > Shared links); **unified pinned threads** (desktop + iOS) |
| 2026-08-24 | `codex mcp-server` deprecated | Codex-as-MCP-server CLI deprecated; use the app server; removed 2026-09-05 |
| 2026-08-25 | **26.825 line — Browser extensions, site tools, and cloud sign-in** | ChatGPT browser extension for **Edge, Brave, Opera, Vivaldi** (+Chrome; Opera lacks side chat), configured in Settings > Computer Use; **WebMCP site tools** in the desktop app's built-in browser for ChatGPT Work and Codex (GPT-5.6 Sol/Terra; not Luna; not Enterprise/Edu); cloud browser sign-in (web/iOS/Android, separate from local browser); event-triggered scheduled tasks (Gmail/Slack/GitHub events; filters; `Run now`; Scheduled inbox) |

### Reference+1 (beyond the installed 26.825; for planning only)

| Date | Entry | Content |
| --- | --- | --- |
| 2026-09-05 | `codex mcp-server` removed | Deprecation completed |
| 2026-09-11 | **26.908** — Quick chats with Pets and Appshots on Windows | Pets quick chat from floating controls (Option+Space / Win+Alt+P; `@` context, `$` skill, bell follow-progress; Settings > Pets; Show pet shortcut; Hide pet command); **Appshots on Windows** (both Alt keys; screenshot + available text; customizable shortcut + target chat); Sources panel open/download; Codex Micro `Insert text` key; pet reset; unfinished comments preserved across chats; dictation Main language; browser tab width/scroll stability |
| 2026-09-14 | GPT-5.5 retirement | Retires 2026-10-14 from ChatGPT/Work/Codex (all plans); switch to GPT-5.6 Sol (`gpt-5.6-sol`) |

### Pre-baseline context (already inside 26.721)

- 26.707 (2026-07-09): Codex joins the ChatGPT desktop app (macOS+Windows);
  Codex can be the default view; in-app Markdown/code editing + inline
  annotations; PR Chat (GitHub PR review in context, inline feedback, patch
  accept/reject); Sites custom domains; plugin management moved into
  Settings; Full-access + Ultra warning dialog; Computer Use faster with
  GPT-5.6.
- 26.715 (2026-07-23): ChatGPT Voice (GPT-Live) in Chat/Work/Codex + Remote
  on iOS; macOS Screen context (appshot of frontmost window); multi-folder
  local projects (Edit project; primary folder drives new chats, Git, AGENTS.md
  /skills/config.toml discovery; secondary folders for file search/read/edit).

## C. Docs-derived: upstream openai/codex runtime releases (GitHub API)

Fetched 2026-09-16 via the GitHub API (releases endpoint; no clone). Stable
releases in scope, with release-note headlines relevant to what the desktop
app surfaces:

- **rust-v0.146.0** (2026-07-29) — `/new`/`/clear` session naming; pin
  threads; side conversations without closing; Agent Plugins manifests,
  workspace plugin publishing, Bedrock + Claude Code marketplaces; fork with
  paginated history incl. temporary forks; app-server → remote Code Mode
  hosts over WebSocket; standalone web search for custom providers;
  executor-provided skills + resource reads; enterprise update controls.
  https://github.com/openai/codex/releases/tag/rust-v0.146.0
- **rust-v0.146.1** (2026-08-05) — safer auto-review defaults for
  cyber-capable models; terminal explains permission changes.
- **rust-v0.147.0** (2026-08-07) — portable Agent Plugins; plugin search
  across local/personal/workspace/remote catalogs; persistent manually
  ordered conversation sections; incremental transcript browsing;
  `--approve-for-me`; Cursor skills import + Claude/Cursor conversation sync;
  MCP 2026-07-28 protocol (paginated discovery, multi-round, non-blocking
  startup); Bedrock cached web search + remote compaction; removed
  `codex exec --full-auto`.
- **rust-v0.148.0** (2026-08-18) — `/export` TUI→Markdown; `codex exec fork`;
  archive/restore from resume picker; draft prompts during init; credits/cost
  in `/status`; Bedrock Runtime built-in provider; hooks run async commands
  and invoke MCP tools; turns reconnect through provider outages; MCP servers
  recover after OAuth reauth without restart.
- **rust-v0.149.0** (2026-08-20) — `codex agents` interactive dashboard;
  `/cd` `/pwd` `/cwd`; `codex queue` (message existing sessions); Vim
  expansion; `codex doctor` diagnoses endpoint protection, network/proxy,
  **desktop app state**, update connectivity; SDK `max`/`ultra` reasoning.
- **rust-v0.150.0** (2026-08-26) — `@` mentions reference other Codex tasks;
  `/copy` picker (responses, code blocks, blockquotes); auto titles for
  unnamed terminal tasks + `/rename` suggestions; permission-mode cycling
  shortcuts; `Interrupt` hooks.
- **rust-v0.151.0** (2026-08-29) — optional-MCP grace period; extensions
  inspect/replace MCP tool results; per-repository plugin catalogs;
  model-aware Ultra reasoning fallback.
- **rust-v0.152.0** (2026-09-01) — Vim `/`+`?` search; rate-limit banners
  with actions (usage/credits/reset/plan); credential-refresh progress incl.
  Bedrock reauth; MCP server names with `:`/`@`/`/`/`.`; per-tool
  `output_token_limit`; configurable `thread/shellCommand` timeouts (>1 h);
  planning tool disabled by default (`tools.update_plan.enabled`).
- **rust-v0.153.0** (2026-09-03) — Vim undo/redo; plugin CLI remote
  marketplace install/remove; `tui.auto_recap=false` + `/recap`; TUI history
  shows full patches/background-terminal input/completed commands; Plus/Team
  half-allowance warnings; TUI reconnect after app-server drop preserving
  drafts; app-server thread metadata gains nullable `model`/`reasoningEffort`;
  async structured questions (`request_user_input_async`) when
  model-catalog-enabled; `features.context_management.experimental_mode`
  (token-budget context, history notes, `new_context` tool; Plus/Pro/Pro Lite
  with Codex backend; excluded for API-key/custom providers).
- **rust-v0.153.1–0.153.4** (2026-09-03…04) — **GPT-6-Astra** model catalog
  (API-configurable without default switch → picker visibility → Bedrock
  Mantle/Runtime routes → bundled default when unconfigured); Fast tier copy
  corrected to "2x speed, increased usage"; async-question guidance.
- **rust-v0.154.0** (2026-09-09, latest stable) — GPT-6-Astra in model
  picker; **experimental worktree support** (`--worktree`, `/worktree`,
  browse + resume isolated checkouts); inline question answering while Codex
  continues (suggested choices, custom text, keep draft); **Windows shared
  background Codex server** (daemon lifecycle + managed updates); Vim `R`
  replace mode; rich-text copy; `/copy` status/fields; existing sessions pick
  up newly installed plugin tools; MCP OAuth refresh coordination;
  macOS sandbox blocks terminal input injection; read-only transcript with
  retry when a conversation is open in another app; `codex mcp-server`
  entry point removed.
- **rust-v0.155.0-alpha.1…12 + alpha.2.x hotfix line** (2026-09-10…16) — in
  flight; also `rusty-v8-v152.2.0` (2026-09-16) and a `voice-cygwin-*` tag
  (voice runtime work). Not part of the 26.825 reference.

Alpha releases of 0.146 (incl. the baseline's own 0.146.0-alpha.3.1 lineage)
and 0.154 hotfix alphas exist in between; only stable lines are listed above.

## D. Docs-derived: current official documentation map (retrieved 2026-09-16)

Doc index: `https://learn.chatgpt.com/llms.txt` (Markdown twins at
`/docs/<path>.md`). Pages most relevant to the reference matrix:

- Desktop app: `docs/app.md`; Windows: `docs/windows/windows-app.md`;
  **Linux preview: `docs/linux/linux-app.md`**; settings reference:
  `docs/reference/settings.md`; commands/shortcuts:
  `docs/reference/commands.md` (macOS/Windows/Linux tables + `codex://` deep
  links); slash commands: `docs/reference/slash-commands.md` (24 commands,
  `$` skills, `/prompts:`).
- Surfaces: browser (`docs/browser.md`), Computer Use
  (`docs/computer-use.md`), integrated terminal (`docs/integrated-terminal.md`),
  code review (`docs/code-review.md`), worktrees
  (`docs/environments/git-worktrees.md`), local/cloud environments
  (`docs/environments/*.md`), MCP (`docs/extend/mcp.md`), Record & Replay
  (`docs/extend/record-and-replay.md`), plugins (`docs/plugins.md`), skills
  (`docs/build-skills.md`), scheduled tasks (`docs/automations.md`),
  notifications (`docs/notifications.md`), remote connections
  (`docs/remote-connections.md`), Remote (`docs/remote.md`), Pets
  (`docs/pets.md`), Codex Micro (`docs/features/codex-micro.md`), Voice
  (`docs/features/voice.md`), Appshots (`docs/appshots.md`), Sites
  (`docs/sites.md`), visualizations (`docs/visualizations.md`), artifacts
  viewer (`docs/artifacts-viewer.md`), projects (`docs/projects.md`),
  memories (`docs/customization/memories.md`), Computer History
  (`docs/customization/computer-history.md`), import (`docs/import.md`),
  permissions (`docs/permissions.md`, `docs/permission-modes.md`), sandboxing
  (`docs/sandboxing.md`, auto-review), hooks (`docs/hooks.md`), WebMCP
  (`docs/webmcp.md`), app-server (`docs/app-server.md`), models
  (`docs/models.md`), feature maturity (`docs/feature-maturity.md`),
  what's new (`docs/whats-new.md`).
- Enterprise: Windows deployment (`docs/enterprise/windows-deployment.md`),
  managed app updates (`docs/enterprise/manage-app-updates.md`), managed
  configuration, plugin/skill controls, workspace model availability.
- Notable current-doc statements: the app is "ChatGPT desktop app" with
  Chat/Work/Codex; `codex://` scheme kept for compatibility; the Linux app
  installs from `persistent.oaistatic.com/codex-app-prod/linux/...`.

## E. Unverified / open questions (checked, not resolvable from here)

1. Exact bundle contents/asar diff of 26.825.51511 — the installed reference
   lives on the operator's machine; no public artifact listing was found
   (checked: changelog, docs, GitHub openai/codex releases — desktop bundles
   are not published there). Any 26.825-only UI micro-details (exact copy,
   new dialogs beyond changelog entries) are `[unverified]`.
2. Whether the Linux preview app ships Computer Use, Appshots, Voice, or
   Pets — the Linux install doc describes projects/local files/Codex only;
   checked `docs/linux/linux-app.md` (silent on those surfaces).
3. GUI surfacing of runtime-only features at 26.825: `/worktree` command
   (runtime 0.154), `@` task mentions (0.150), plugin publishing/sharing UI,
   `codex agents` dashboard, `codex queue` — the runtime releases are
   documented, the desktop changelog does not name them; labeled
   `[unverified]` for GUI surface, `[docs-derived]` for runtime.
4. Introduction dates for `Toggle terminal` (Ctrl+`), `Clear terminal`,
   `Toggle file tree`, font-size shortcuts — present in current
   `docs/reference/commands.md`, absent from the baseline-recorded registry
   (`docs/parity-matrix.md`); not dated by the changelog.
5. The 126-vs-89 experimental method count discrepancy (see §A).
6. macOS-specific mechanics of Computer Use/Appshots (Windows mechanics are
   richly recorded in-repo; macOS equivalents beyond changelog statements
   are `[unverified]`).

## F. Method & reproduction

- CLI runs: `CODEX_HOME=/tmp/qa-codex-home
  /home/z/parity-lab/tools/codex-cli/codex-x86_64-unknown-linux-musl <args>`
  (2026-09-16). `~/.codex` untouched (verified rule from Task 41).
- GitHub API: `GET /repos/openai/codex/releases` (paginated, PAT auth; token
  never logged/committed). No clone (disk bound).
- Web: `web-search` + `page_reader` functions (z-ai) against
  `learn.chatgpt.com` (Markdown twins), `openai.com/release-notes` (JS-only,
  no extractable content), help center search.
- E2B prior-session evidence: `/home/z/my-project/e2b-evidence/workspace-a.json`
  + `workspace-a-journeys.json` (read-only).
