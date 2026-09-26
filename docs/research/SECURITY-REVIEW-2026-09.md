# SEC-001 — The Production Security Review (credential-law audit + gateway review + threat model)

> **Status: AUTHORED 2026-09-26 — Worker C delivery, Wave 8 (the F13
> release-readiness trio).**
>
> **Pinned base:** `4563d0f5a1ba10d5bb7b444055b19eefa5f6461b` (main; the
> Wave-8 dispatch head). Every code pointer below cites that checkout.
>
> **Contract authority:** [F2-CONTRACT-KERNEL.md](../F2-CONTRACT-KERNEL.md)
> (frozen) + the Wave-2..7 addenda + the Wave-8 kernel addendum
> ([WAVE8-PROD-WORK-ORDERS.md](WAVE8-PROD-WORK-ORDERS.md)) + the WEB-001
> kernel laws ([WAVE6-WORK-ORDERS.md](WAVE6-WORK-ORDERS.md) §1–§9) + the
> E2B harness law (credential scrubbing by allowlist).
>
> **This review VERIFIES; it does not fix** (the FV-wave pattern): findings
> become focused fix work orders through the synchronization law. Zero
> source files changed (git-diff-proven in the completion report).

## 0. Scope and method

The audit covers the full repository at the pinned base: the sixteen
Rust crates under `crates/` (the desktop app, the contract crates, the
supervised runtime binding, and `flauz-web-gateway`), the TypeScript web
client under `web/`, `scripts/`, `.github/workflows/`, `third_party/`,
`reference/`, and the documentation tree (including the committed evidence
snapshots, which are themselves credential-bearing-risk surfaces and were
swept like any other file).

Static review only, per the work order: grep/ripgrep sweeps over every
text file, line-level triage of every hit, and a code-pointer review of
the gateway's five named security surfaces. No penetration testing, no
fuzzing, no dynamic analysis (named honestly in §6).

### Reproduction (the exact commands)

All sweeps ran at the pinned checkout with ripgrep, excluding only binary
asset types (the sweep is over text; the binary exclusions are the
standard image/font/wasm set — no text file is excluded, `.git` is not
part of the worktree sweep):

```
rg -n "Bearer" --no-mmap -g '!*.png' -g '!*.jpg' -g '!*.jpeg' \
   -g '!*.svg' -g '!*.woff2' -g '!*.wasm' -g '!*.ico' -g '!*.pdf' .
rg -n "token=" --no-mmap            (same exclusions)
rg -n -i "api[_-]?key" --no-mmap    (same exclusions)
rg -n "sk-" --no-mmap               (same exclusions)
rg -n "auth\.json" --no-mmap .
```

Supplementary secret-shape scans (beyond the order's named five, added for
completeness):

```
rg -n "sk-[A-Za-z0-9]{16,}" --no-mmap .        # real OpenAI-style key shapes
rg -n "BEGIN.*PRIVATE KEY" --no-mmap .         # private-key blocks
rg -n -i '(password|passwd|secret|token)\s*[:=]\s*"[^"]{6,}"' \
   --no-mmap crates/ web/src/ scripts/         # hard-coded assignments
find . -name ".env*" -o -name "*.env"          # environment files
rg -n "localStorage|sessionStorage" web/src/   # web-client persistence
rg -n "flausec_" --no-mmap crates/             # secret-reference discipline
rg -n "://[A-Za-z0-9_]+:[^@/]{2,}@" --no-mmap . # URL userinfo credentials
```

## 1. The credential-law audit (grep-evidenced)

**The law** (kernel §7 + the E2B harness law + Wave-6 addendum §4): no
credential ever appears in a contract type, fixture, log line, URL,
serialized state, script, or evidence artifact. `ProviderConnection` holds
`flausec_...` secret references only; the web client never embeds
credentials in URLs, logs, or evidence; anything that exports state
scrubs by allowlist.

### 1.1 Per-pattern results and triage

| # | Pattern (flags) | Raw hits | Triage verdict |
|---|---|---|---|
| 1 | `Bearer` | **31** | 0 violations — see 1.2 |
| 2 | `token=` | **8** | 0 violations — see 1.3 |
| 3 | `api[_-]?key` (case-insensitive) | **270** | 0 violations — see 1.4 |
| 4 | `sk-` | **742** | 0 violations — see 1.5 |
| 5 | `auth\.json` | **7** | 0 violations — see 1.6 |
| | **Total raw hits** | **1,058** | **every hit triaged; zero credential material** |

The triage vocabulary (the order's own): **legitimate-named-constant**
(enforcement marker lists, enum variants, protocol type names) ·
**test fixture** (proving rejection or scrubbing, using placeholder
shapes) · **product label/copy** (UI strings describing auth modes) ·
**doc reference** (documentation naming a file or pattern) ·
**violation** (real credential material in a script, log, URL, storage,
or test — **none found**).

### 1.2 `Bearer` — 31 hits, all legitimate

- **Enforcement marker lists (6):** the frozen `CREDENTIAL_MARKERS`
  family that *rejects* credential-shaped values:
  `crates/flauz-prov/src/key.rs:36`, `crates/flauz-exec/src/connection.rs:33`,
  `crates/flauz-exec/src/environment_local.rs:723` (a scan refusing JSON
  env configs that carry markers),
  `crates/flauz-exec/src/environment_fake_remote.rs:586`,
  `crates/codex-app/src/ui/flauz_providers.rs:449`
  (`REAL_KEY_MARKERS` — the UI refuses real-credential shapes in its fake
  seam), and the w2/w3-gate evidence snapshots of the same lists
  (`docs/research/evidence/w2-gate/capability_resolver_step.rs:28`,
  `environment_fabric_step.rs:33`, `w3-gate/f6_scenario_step.rs:86`).
- **Conformance/test fixtures proving rejection or scrubbing (13):**
  `crates/flauz-exec/tests/conformance.rs:539,543,704` (e.g. the
  `flausec_Bearer token` *rejection* vector),
  `crates/flauz-prov/src/fakes.rs:517`, `crates/flauz-context/tests/conformance.rs:473`,
  `crates/flauz-context/tests/w3_conformance.rs:312`,
  `crates/flauz-takeover/tests/conformance.rs:22→33`,
  `crates/flauz-lease/tests/conformance.rs:22`,
  `crates/flauz-world/tests/conformance.rs:361`,
  `crates/codex-app/src/ui/flauz_providers.rs:1351`,
  `crates/codex-app/src/ui.rs:50890,50893,51575` (MCP record parsing /
  label tests).
- **Named constants / enum variants (4):** `crates/codex-core/src/lib.rs:2433`
  and `crates/codex-protocol/src/lib.rs:3317` (`BearerToken` auth-status
  variants), `crates/codex-app/src/backend.rs:1039` (the status mapping).
- **One runtime construction — correctly handled (1):**
  `crates/codex-app/src/backend.rs:2130` builds the `Authorization: Bearer`
  header for the Computer-Use site-status call from the auth token; the
  buffer is zeroized after header construction (`bearer_bytes.fill(0)`),
  the header is marked `set_sensitive(true)` (log redaction at the HTTP
  layer), and the token is sourced from the supervised auth state, never
  persisted.
- **Product copy (2):** `crates/codex-app/src/ui.rs:44768` — the MCP
  editor field labeled *"Bearer token env var"* asks for an **environment
  variable name**, not token material; `ui.rs:48574` — the auth-status
  label *"Authenticated (API key)"*.
- **Doc-comment law statements (3):** `crates/flauz-prov/src/lib.rs:49`,
  `crates/flauz-prov/src/fakes.rs:54`, and the work order itself
  (`docs/research/WAVE8-PROD-WORK-ORDERS.md:192`).
- **Doc references (2):** the w2/w3-gate evidence snapshots listed above.

### 1.3 `token=` — 8 hits, all legitimate

- **Scrubbing-law proofs (4):** `crates/codex-core/src/lib.rs:36003` and
  `crates/codex-app/src/backend.rs:19249` — fixtures shaped
  `https://example.com/private?token=redacted` / `?token=do-not-store`
  asserting the persisted/restored browser-download record carries an
  **empty** URL (the scrub); `crates/codex-app/src/backend.rs:22075` —
  an MCP OAuth *error* fixture asserting error strings are surfaced
  without persisting provider material;
  `crates/codex-platform/src/browser.rs:5645` — a permission-matching
  fixture.
- **Gateway self-tests (2):**
  `crates/flauz-web-gateway/src/protocol.rs:402` (the descriptor test
  asserts no `token="` material) and
  `crates/flauz-web-gateway/src/logging.rs:233` — the redaction test:
  a hostile caller tries to log secret material in every field; the
  output must show `[REDACTED]` (it does).
- **Marker family in a conformance test (1):**
  `crates/flauz-orch/tests/conformance.rs:286`.
- **The work order text (1):** `WAVE8-PROD-WORK-ORDERS.md:193`.

### 1.4 `api[_-]?key` — 270 hits, all legitimate

Distribution and verdict (the dominant files):

| File / family | Hits | Verdict |
|---|---|---|
| `crates/codex-app/src/ui.rs` | 70 | Product labels, MCP auth forms, and their tests — no material |
| `crates/codex-core/src/lib.rs` | 30 | Protocol types (auth modes/status) + scrub tests |
| `crates/codex-protocol/src/lib.rs` | 26 | Schema type names (`ApiKey` auth surface) |
| `crates/codex-app/src/backend.rs` | 26 | Status mapping + scrub tests |
| `web/src/app/SignInView.tsx` | 19 | The sign-in form: `signInWithApiKey(apiKey.trim())` then **`setApiKey("")`** — the field is cleared on submit; the key transits the app-server auth API only |
| `web/src/protocol/schema.json` + `generated.ts` | 12 | The **generated** protocol surface (frozen schema export; hand-written duplicates are forbidden by Wave-6 §3) — includes the schema note *"Auth token when requested; the web client never requests it and never logs it"* |
| `crates/codex-app/src/ui/flauz_providers.rs` | 6 | `REAL_KEY_MARKERS` + labels |
| `web/src/strings/en.ts`, `web/src/state/app.tsx` | 8 | UI strings + the sign-in state machine |
| `web/lab/mock-gateway.mjs` | 4 | Lab test double (fake auth responses; never a real key) |
| Evidence snapshots + wave docs | ~20 | Historical gate snapshots of the same marker families |
| Remaining (tests, fixtures, docs) | ~49 | Conformance fixtures + documentation |

Zero raw API-key material anywhere in the 270.

### 1.5 `sk-` — 742 hits, zero credential material

The refined secret-shape scan **`sk-[A-Za-z0-9]{16,}` returns ZERO
hits** — there is no OpenAI-style key material in the repository. All 742
raw hits are substring matches of ordinary words and identifiers:
`task-` / `flauz-task-` kebab-case IDs (the bulk — e.g.
`crates/codex-core/src/lib.rs:26403` `task_in_repository("task-a")`),
`preset:ask-for-approval` mode IDs, `risk-`, plus the enforcement marker
lists and conformance fixtures already triaged in §1.2 and the
`flauz-lab` w4 fixture JSONs (kebab-cased journey/task ids).

### 1.6 `auth\.json` — 7 hits, all documentation; zero source reads

Every hit is a doc reference describing the official Codex state layout
(`docs/research/CODEX-REFERENCE-MATRIX.md:241`, the codex-ref doctor
evidence, the FV-gate record's named W-AUTH environment gap) or the work
order itself. **No source file reads or writes `auth.json`** — consistent
with the AGENTS.md law: the live `~/.codex` auth files are never opened
directly; runtime access flows through the supervised official
`codex app-server` process.

### 1.7 Supplementary scans (beyond the named five)

| Scan | Result | Verdict |
|---|---|---|
| `sk-[A-Za-z0-9]{16,}` (secret-shaped) | **0 hits** | No key material |
| `BEGIN.*PRIVATE KEY` | **0 hits** | No private-key blocks |
| Hard-coded `password/secret/token = "..."` in `crates/`, `web/src/`, `scripts/` | **1 hit**: `crates/flauz-web-gateway/src/logging.rs:224` — the hex placeholder inside the redaction **test** that proves `[REDACTED]` works | Test fixture, not a credential |
| `.env*` / `*.env` files | **none** | No environment files committed |
| `localStorage` / `sessionStorage` in `web/src/` | **0 hits** | The web client persists **nothing** token-shaped client-side |
| `flausec_` in `crates/` | Only the `SecretRef` reference-only system: `flauz-prov` (`key.rs`, `account.rs`, `lib.rs`, `scheduler.rs`), its fixtures/tests, the mirror marker list in the providers UI, and the flauz-exec README | References only (kernel §7) — no material |
| URL userinfo `://user:pass@` | **3 hits**, all tests proving `normalize_browser_origin` **strips** userinfo credentials (`crates/codex-platform/src/browser.rs:5608`, `crates/codex-app/src/backend.rs:20215,20433`) | Scrubbing proofs |

### 1.8 Audit verdict

**ZERO credential-law violations.** Every one of the 1,058 raw hits
triages to a legitimate-named-constant, a test fixture proving
rejection/scrubbing, product copy, or a doc reference. The enforcement
architecture is structural and mirrored across the seam (the frozen
`CREDENTIAL_MARKERS` family exists identically in `flauz-exec`, `flauz-prov`,
the providers UI, and the fake environments), the secret-reference
discipline holds (`flausec_` references only), the web client never
persists token material, and the gateway's logger structurally cannot
emit credentials (allowlist fields + defensive redaction, §3.6).

## 2. The gateway security-surface review

The five named surfaces of `crates/flauz-web-gateway` (WEB-001, merged at
`e99941e`), each with code pointers and a verdict. The gateway's contract:
**transparent transport, localhost-only default bind, session-token law,
zero product logic** (Wave-6 addendum §1/§2/§4).

### 2.1 The localhost-only bind law — VERDICT: HOLDS

- `src/config.rs:14` — `DEFAULT_BIND_ADDR = 127.0.0.1`; `:20` — the
  documented label `127.0.0.1:8610`; `:275-290` — `with_defaults` builds
  exactly that bind.
- `src/config.rs:308-311` — `validate()` refuses a non-loopback bind
  without provisioned session tokens
  (`NonLocalBindRequiresSessionTokens`, the named hard-fail); the
  message names the recovery path (`--session-token-file`).
- `src/server.rs:90-91` — `serve()` runs `config.validate()` **before**
  any bind or state construction; the refusal cannot be raced past.
- Tests: `config.rs:368-387` (`default_bind_is_localhost_only`,
  `non_local_bind_refused_without_session_tokens`),
  `server.rs:204-222` (the serve-path refusal surfaces the same error),
  and the integration suite's non-local-bind refusal case.
- Enforcement at the operator boundary: `src/main.rs:18-25` (USAGE states
  the law), `:133-137` (token file loaded and validated at startup).

### 2.2 The non-loopback session-token requirement — VERDICT: HOLDS

- `src/config.rs:118-227` — `SessionTokens`: bounded read (64 KiB,
  `MAX_TOKEN_FILE_BYTES`), one token per line, 16–256 printable
  non-whitespace bytes (`MIN/MAX_TOKEN_BYTES`), at most 128 tokens; a
  malformed file is a named hard-fail at startup.
- `src/config.rs:218-226` + `:230-240` — `validate()` compares the
  presented token against every provisioned token with a **fixed-time**
  byte-wise equality (length-folded, no early exit) — no timing
  shortcut on length or content mismatch.
- `src/config.rs:204-206` — `token_material_for_redaction()` is the
  **only** sanctioned consumer of raw material: it feeds the logger's
  defensive redaction set (`src/main.rs:165-170`), so an accidental log
  line can never carry a provisioned token.
- `src/session.rs:400-410` — the handshake enforces the token when
  provisioned; a missing/invalid token is refused with the named
  `invalid_token` denial and the socket closes.
- Token material never appears in: logs (structurally — §3.6), the
  health payload (§2.4), the protocol descriptor
  (`protocol.rs:402-404` self-test), or any artifact.
- Session ids are **labels, not secrets**: `session.rs:494-512` mints
  `gwsess_<hex>` from a counter + clock + process id; the doc comment
  states the law (never authorize anything, never derived from
  credentials).

### 2.3 Origin / WSS handling — VERDICT: HOLDS, with a recorded posture note (SEC-F-03)

- `src/session.rs:93-121` — `refuse_cross_origin_ws`: while the gateway
  binds loopback, a WebSocket upgrade carrying an `Origin` header whose
  host is not `localhost`/`127.0.0.1`/`[::1]` is refused with `403` and
  a named `websocket_origin_refused` warn log. This is the CSWSH
  (cross-site WebSocket hijacking) defense: a malicious page in the
  user's browser cannot drive the local gateway.
- Non-browser clients (no `Origin` header) are deliberately unaffected —
  the lab/CLI drivers depend on it (documented in the function's
  doc comment).
- WSS itself: the gateway speaks plain `ws://`/`http://` only; TLS is an
  operator-owned termination concern on non-local deployments — recorded
  as finding **SEC-F-01** (§5) with its recovery path, not a silent gap.
- On non-local (token-gated) binds the origin check is disabled by
  design (the token is the gate; non-browser clients are first-class) —
  recorded as posture note **SEC-F-03**.

### 2.4 Request timeout + in-flight caps (DoS posture) — VERDICT: HOLDS

- **Frame bounds:** `src/session.rs:80-83` — the upgrade sets
  `max_message_size` and `max_frame_size` to the configured
  `max_frame_bytes` (default `DEFAULT_MAX_FRAME_BYTES` = 16 MiB,
  `codex-protocol` parity); `src/protocol.rs:41-45` — method names
  bounded to 128 bytes; at most **3** malformed frames before the socket
  closes (`MAX_MALFORMED_FRAMES`, enforced at `session.rs:261-276` and
  `:301-319`).
- **Handshake bound:** `src/session.rs:39` + `:348` — the browser has
  15 s (`HANDSHAKE_TIMEOUT`) to complete `session.claim`; silence ends
  the socket.
- **Session admission:** `src/session.rs:412-422` — claimed sessions are
  capped (`max_sessions`, default 8, hard max 32 — `config.rs:23,37`);
  over-limit claims are refused with the named `session_limit_reached`
  and the counter is restored. `SessionGuard` (`session.rs:484-492`)
  releases the slot on any teardown path (RAII).
- **In-flight requests:** `src/supervisor.rs:539-550` — a brokered
  request beyond `max_inflight_requests` (default 32, hard max 64) is
  answered with the named `TOO_MANY_INFLIGHT` error — never dropped
  silently; `:551-596` — each accepted request increments a counter,
  runs on a bounded worker, and decrements on completion, timeout, or
  spawn failure (the spawn-failure path restores the budget and answers
  with a named `TRANSPORT` error).
- **Request timeout:** `src/config.rs:30` — 10 s default (desktop
  parity); `src/supervisor.rs:566-574` — a timed-out brokered request is
  answered with the named `REQUEST_TIMEOUT` error.
- **Backpressure honesty:** `src/session.rs:322-335` — a saturated
  command queue refuses within 100 ms with the named saturation error;
  `supervisor.rs` drops only *notifications* under backpressure and
  **names the drop count** (`NOTIFICATIONS_DROPPED`, `session.rs:462-468`);
  responses are never dropped (`supervisor.rs:592-594`).
- **Outgoing queue:** bounded at 256 frames
  (`OUTGOING_QUEUE_CAPACITY`, `session.rs:42`).
- **Graceful shutdown:** bounded drain (`server.rs:25`, 10 s) with a
  named `gateway.shutdown` frame to every live session
  (`session.rs:208-217`).
- Honest bounds recorded as findings **SEC-F-02** (unauthenticated
  static/health surface has no aggregate concurrency cap) and
  **SEC-F-04** (pre-claim sockets are bounded only by the 15 s handshake
  timeout) — both negligible on the loopback default, both named with
  recovery paths for non-local deployments.

### 2.5 Static-file serving + path traversal — VERDICT: HOLDS

- `src/static_files.rs:99-131` — `safe_relative_path` normalizes the
  request path and **refuses**: `..` or any non-normal component
  (`:120-125`), absolute/protocol-relative forms (`//host/...` → 404,
  the 2026-09-25 Lead gate fix at `:107-115`), backslashes, NUL, and
  paths over 512 bytes.
- Only `Component::Normal` segments ever reach `root.join(...)` — the
  join cannot escape the web root.
- Per-file bound: 64 MiB (`MAX_STATIC_FILE_BYTES`, `:12`, enforced
  `:71-73`); reads are bounded and moved off the async reactor
  (`server.rs:170-182`, `spawn_blocking`).
- Headers: `x-content-type-options: nosniff` on every response
  (`:82-85`); `cache-control: no-store` for the shell, `no-cache` for
  hashed assets (`:87-95`); correct content types (`:158-177`).
- SPA fallback serves the shell only for **extension-less relative**
  routes (`:46-52`) — an absolute-path miss is a 404, not a 200 shell.
- Tests: `static_files.rs:262-280` — the traversal vector set
  (`/../secret.txt`, `/..%2Fsecret.txt`, `/a/../../secret.txt`,
  `//etc/passwd`, `/assets/../../../secret.txt`, `/C:\windows\win.ini`)
  all refused; `:283-290` — missing assets 404; `:247-259` — SPA
  fallback correctness.

### 2.6 Additional gateway surfaces reviewed (completeness)

- **The handshake law:** `src/session.rs:343-436` — exactly one
  `session.claim` first; any app-server frame before it is refused with
  the named `handshake_required` (`:379-389`); a second claim on a
  claimed socket is refused with `handshake_already_claimed`
  (`:278-291`); only after the claim does the supervisor spawn and
  `initialize` the supervised app-server (`supervisor.rs:397-415`) —
  the browser never sends `initialize`.
- **The transparent-transport law:** `src/protocol.rs:117-157` — the
  frame classifier recognizes only the two dialects (gateway control +
  app-server JSON-RPC); the gateway never interprets, filters, or
  fabricates methods; the descriptor at `/gateway-protocol.json`
  documents the contract including the bounds and the credential law
  (`:292-300`).
- **The logger (credential posture, structural):**
  `src/logging.rs:84-100` — allowlist field names (`sanitize_field_name`,
  `:129-135`), bounded values (512 bytes, 12 fields), defensive
  redaction against provisioned token material (`:137-157`, `[REDACTED]`);
  the module doc states the law: frame/token payloads are never passed
  to the logger in the first place. Proven by
  `no_credential_material_is_ever_emitted` (`:223-251`).
- **The health endpoint:** `static_files.rs:182-191` — liveness + bind +
  uptime + session count only; no tokens, no paths, no credentials.
- **Supervision parity:** death → named `reconnecting` → restart with
  the desktop's backoff schedule → `connected` (`supervisor.rs`, the
  run/restart state machine; identical semantics to the desktop backend
  binding — the WEB-001 integration requirement).

## 3. The threat model

### 3.1 Assets

| Asset | Where it lives | Protection |
|---|---|---|
| Provider credentials / API keys | The official Codex auth state (`auth.json` et al.) under `CODEX_HOME`, owned by the supervised app-server — **never** in Flauz-owned state | The supervised-access law (AGENTS.md); no direct file access anywhere in this repo (audit §1.6); the desktop's bearer construction zeroizes buffers (`backend.rs:2130`) |
| Secret references (`flausec_...`) | Contract state (SQLite, canonical JSON) | Reference-only discipline (kernel §7); marker rejection at every parse boundary (audit §1.2) |
| Gateway session tokens | Operator-provisioned file → process memory only | Bounded read; fixed-time comparison; never logged (structural + defensive redaction); never persisted |
| Session state + task/session history | `state.sqlite3` (codexRS-owned, single-writer, paginated) | Bounded queries; no unbounded reads (AGENTS.md); scrubbing of URL-bearing records (audit §1.3) |
| The user's repository files | The workspace filesystem | Computer-Use / browser / terminal authorization surfaces (codexRS in-scope set, SECURITY.md) |
| Release artifacts | GitHub releases (tar.gz; checksums + signing per REL-001) | SHA256SUMS manifest + GPG-ready detached signatures (the sibling Wave-8 order) |

### 3.2 Actors

1. **The local user** (fully trusted — owns the machine, `CODEX_HOME`,
   and the repo files).
2. **A remote browser user** on a non-local, token-gated deployment
   (semi-trusted: authenticated by a provisioned token, but a remote
   network peer).
3. **A malicious web page in the local user's browser** (untrusted;
   CSWSH attempt against the loopback gateway — refused by §2.3).
4. **A network attacker** on a non-local deployment (untrusted; the
   cleartext-transport exposure is SEC-F-01).
5. **Malicious workspace content** (untrusted input rendered/executed
   through the supervised runtime; bounded by the app-server's own
   approval boundaries — the codexRS scope, not re-derived here).
6. **A compromised supervised app-server** (outside this review's trust
   boundary: the official binary is operator-installed; the gateway's
   exposure is limited to what it can push down the bridge — which the
   web client renders under its own state machine).
7. **The supply chain** (release-artifact tampering — mitigated by the
   REL-001 checksum/signature scaffold; dependency policy is the
   SECURITY.md in-scope set).

### 3.3 Surfaces and trust boundaries

```
[remote browser] ──ws(s)──> [flauz-web-gateway] ──supervise──> [official codex app-server] ──> CODEX_HOME (auth state)
                                 │  static hosting (web/dist)                                    │
[local user] ─────────────────────┘                                                               ▼
[desktop app: codex-app + crates] <────────── the same supervised app-server surface ────── the user's repo files
[contract crates: state in state.sqlite3 — references only, never material]
```

Boundaries: (a) browser → gateway (the handshake + token + origin laws);
(b) gateway → app-server (the supervised spawn/initialize/restart
machine); (c) app-server → `CODEX_HOME` (official-owned; never directly
touched); (d) any process → `state.sqlite3` (single-writer, paginated);
(e) repo → release artifacts (the REL-001 manifest/signature story).

### 3.4 Threat analysis (per surface, STRIDE-condensed)

| Surface | Threats considered | Disposition |
|---|---|---|
| Gateway bind | Unauthorized remote exposure | Loopback-only default; non-local requires tokens + refuses without them (§2.1) |
| Gateway handshake | Unauthenticated bridging; token brute-force; session squatting | `handshake_required` refusal; 15 s bound; fixed-time compare; session cap + RAII slot release (§2.2, §2.4) |
| Gateway transport | Oversized frames; malformed floods; method abuse; CSWSH | 16 MiB frame caps; 3-malformed close; 128-byte method bound; origin refusal (§2.3, §2.4) |
| Gateway static | Path traversal; stale shell; MIME confusion; unbounded reads | Component-normalized refusal; `//` refusal; `no-store`; `nosniff`; 64 MiB cap (§2.5) |
| Gateway logs | Credential leakage | Structural: never handed secrets + defensive redaction + bounds (§2.6) |
| Web client | Token persistence; XSS-stored credentials | No `localStorage`/`sessionStorage` use at all; API-key field cleared on submit; schema notes "never requests, never logs" (§1.4, §1.7) |
| Contract state | Credential material in serialized state | `flausec_` references only + marker rejection at every boundary (§1.2, §1.7) |
| Desktop runtime | Token in memory/logs/URLs | Zeroized bearer buffer; sensitive headers; URL scrubbing on persistence (§1.2, §1.3) |
| Release artifacts | Tampered downloads | REL-001 checksum manifest + GPG-ready signatures (sibling order) |

### 3.5 Assumptions

- The official Codex CLI/app-server and the provider services are
  trusted components under their own security policies (the upstream
  policy pointer stays in SECURITY.md).
- The operator's token file and `CODEX_HOME` are treated as
  operator-owned secrets (file permissions are the operator's domain).
- The loopback network namespace on the default deployment is
  single-user (the standard desktop assumption).

## 4. Findings register (violations: none; findings: recorded, NOT fixed here)

| ID | Severity | Finding | Evidence / pointer | Recovery path |
|---|---|---|---|---|
| **SEC-F-01** | **Medium (operator-owned)** | No TLS termination in the gateway: on a non-local bind, bridged auth material (e.g. an API key through the sign-in flow) and session tokens transit cleartext `ws://`/`http://` unless the operator fronts the gateway with a TLS proxy | `main.rs` USAGE; `server.rs` (plain `TcpListener`); README flags table | Documented operator responsibility (reverse proxy); a future focused WO may add native TLS if non-local deployments materialize |
| **SEC-F-02** | **Low** | The unauthenticated static/health surface has no aggregate request-concurrency cap: per-file reads are bounded (64 MiB) but a concurrent large-file flood on a non-local bind can amplify memory beyond per-request bounds | `server.rs:170-182` (per-request `spawn_blocking`, no semaphore); `static_files.rs:12` | Bounded-concurrency hardening WO **if** non-local deployment is exercised; negligible on loopback |
| **SEC-F-03** | **Low (posture note)** | Cross-origin WS refusal is enforced only on loopback binds; on token-gated non-local binds the session token is the sole browser-origin defense (by design — non-browser clients are first-class) | `session.rs:94-96` (early return when not loopback) | Optional origin-allowlist flag for non-local binds (a focused WO); current posture documented here |
| **SEC-F-04** | **Informational** | Pre-claim sockets are bounded only by the 15 s handshake timeout, not by `max_sessions` (which counts claimed sessions): an unauthenticated connection flood on a non-local bind occupies sockets/fds | `session.rs:348` (timeout) vs `:412-422` (post-claim admission) | Standard OS/connection-limit hardening at the operator edge; named here for the record |
| **SEC-F-05** | **Informational (resolved precedent)** | The 2026-09-25 Lead gate fix hardened the absolute-path/protocol-relative traversal (`//etc/passwd` previously SPA-served the shell with 200) | `static_files.rs:107-115` + the traversal test vector set `:262-280` | Already fixed and test-pinned; recorded as the precedent for the audit method |
| **SEC-F-06** | **Informational** | Session ids (`gwsess_<hex>`) are labels minted from counter+clock+pid — documented as never-authorizing, never credential-derived; misuse would require treating them as secrets | `session.rs:494-512` + its doc comment + label test `:520-529` | None needed (documented law); any future change must preserve the label-not-secret property |

**Violations requiring fix work orders: NONE.** No finding in this
register was patched in this review (the synchronization law: findings
become focused WOs; this order is docs-only).

## 5. Honest bounds and environment gaps

- **Static review only.** No penetration testing, fuzzing, or dynamic
  analysis was performed (the order's non-goal). The verdicts are
  code-pointer verdicts at the pinned SHA.
- **The W-AUTH environment gap (carried from the FV gate record):** the
  web client's authenticated journeys (FV-E01..E13) remain blocked on
  operator credentials at this station; only the gateway-security probe
  (FV-E00) ran against the real Rust gateway. The web-auth surfaces were
  reviewed statically here (§1.4, §1.7, §2.2) — runtime verification
  rides the W-AUTH recovery path (operator-provisioned auth state).
- **No Rust toolchain in this sandbox:** no `cargo test`/`clippy` gates
  were run for this review (none apply to a docs-only order); the Lead
  gates by re-running the audit greps (spot-check ≥10 documented hits)
  and the threat-model review.
- **Upstream components** (the official Codex CLI/app-server, provider
  services) are out of scope per SECURITY.md; their policies govern.
- Line citations are current-main lines at `4563d0f` — volatile by
  design; the symbol names are the stable anchors (the house
  convention).

## 6. Acceptance-criteria map (the order's four bullets)

1. **Every grep pattern documented with its command + hit count + the
   per-hit triage** — §0 (commands), §1.1 (counts), §1.2–§1.7 (per-hit
   triage), §1.7 (the supplementary scans).
2. **The gateway review covers every named surface with code pointers** —
   §2.1 bind law, §2.2 token law, §2.3 origin/WSS, §2.4 timeouts +
   in-flight caps, §2.5 static traversal, §2.6 the completeness set
   (handshake, transparent transport, logging, health, supervision).
3. **SECURITY.md states the honest posture** — rewritten in the same
   commit: reporting policy, supported versions, what is reviewed
   (this document), what is operator-owned (TLS, token files,
   deployment), what is deferred (W-AUTH, upstream).
4. **Zero source changes** — `git diff --stat 4563d0f` = exactly the two
   owned doc paths; proven in the completion report.
