# flauz-web-gateway (WEB-001)

The Flauz web client gateway: a transparent WebSocket-to-app-server
bridge plus static-file hosting for the web frontend (`web/`).

**Contract base:** `docs/F2-CONTRACT-KERNEL.md` + the Wave-2..5 addenda +
the Wave-6 kernel addendum (`docs/research/WAVE6-WORK-ORDERS.md` §1–§9).
The gateway adds ZERO product logic: the app-server JSON-RPC protocol is
the only capability contract; a capability missing from the protocol is a
protocol work order, never a gateway-side or frontend-side fabrication.

## Running

```
cargo run -p flauz-web-gateway -- --web-root ../web/dist
```

Options (see `--help`):

| Flag | Default | Meaning |
| --- | --- | --- |
| `--bind <ADDR>` | `127.0.0.1:8610` | Bind address. **Loopback-only by default.** A non-loopback bind requires `--session-token-file` and the gateway refuses to start without it. |
| `--web-root <DIR>` | `web/dist` | Static hosting root for the built web client (SPA fallback to `index.html`). |
| `--codex-binary <PATH>` | `CODEX_RS_CODEX_BIN` or `PATH` | The official `codex` binary used to spawn supervised app-servers. |
| `--codex-home <DIR>` | `~/.codex` | `CODEX_HOME` for supervised app-servers (isolated homes for labs/tests). |
| `--session-token-file <PATH>` | none | Operator-provisioned authenticated-session tokens (one per line, 16–256 printable non-whitespace bytes). Required for non-local binds; optional extra gate for localhost. |
| `--max-sessions <N>` | 8 | Maximum concurrent authenticated sessions (hard cap 32). |
| `--max-inflight <N>` | 32 | Maximum in-flight bridged requests per session (hard cap 64). |
| `--request-timeout-secs <N>` | 10 | Brokered request timeout (desktop parity). |

## Endpoints

- `GET /` — the web shell (static hosting with SPA fallback).
- `GET /ws` — the WebSocket bridge (see the protocol below).
- `GET /healthz` — liveness + session count (no credentials).
- `GET /gateway-protocol.json` — the machine-readable gateway protocol
  descriptor (the same contract `web/src/gateway/protocol.ts` mirrors).

## The WebSocket protocol (v1)

The browser speaks the app-server's own JSON dialect — frames
`{"method", "id", "params"?}` (request), `{"method", "params"?}`
(notification), `{"id", "result"}` / `{"id", "error"}` (response) — with
NO `jsonrpc` field, exactly like the app-server wire format — plus a
minimal gateway control envelope (`{"type": ...}` frames):

1. **Handshake:** the first message MUST be
   `{"type":"session.claim","token":?}`. The gateway mints a session,
   spawns ONE supervised app-server for the session, performs the
   `initialize` handshake (the supervisor owns it, exactly like the
   desktop backend; the browser never sends `initialize`), and replies
   `{"type":"session.claimed","sessionId":"gwsess_...","protocolVersion":1}`.
   Any app-server frame before the handshake is refused with
   `{"type":"session.denied","code":"handshake_required",...}` and the
   socket closes. When tokens are provisioned, a missing/invalid token
   is refused with `code:"invalid_token"`.
2. **Transparent bridging:** requests are BROKERED through the supervised
   connection (the connection's router only delivers responses to
   requests it issued itself, so the bridge re-issues each request and
   re-encodes the response with the browser's original id);
   notifications pass through verbatim; server-initiated requests
   (approvals, …) are forwarded with their ids, and their responses are
   routed back.
3. **Truthful state events:**
   `{"type":"gateway.state","state":"connected"|"reconnecting","reason":...}`
   on every supervised-app-server transition (named reasons, always).
   `{"type":"gateway.shutdown","reason":...}` precedes a graceful
   gateway shutdown. `{"type":"gateway.error","error":{...}}` carries
   transport-level refusals (malformed frame, invalid method, too many
   in-flight, timeout, dropped notifications).

### Supervision semantics

Identical to the desktop backend binding (any divergence is a
deviation): spawn via `codex-platform`'s `AppServerConnection`; `initialize`
with `experimentalApi: true` and the desktop's exact capabilities payload;
on death, restart with the desktop's backoff schedule (1 s initial,
doubling, 20 s cap, reset on success); reap via the managed child's
graceful shutdown. Requests that cannot be brokered (empty/oversized
method, saturated inflight budget, worker spawn failure) are answered
with named gateway errors — never dropped silently.

## Logs

Structured JSON lines on stdout (`{"ts","level","event",...}`). The
logger never receives and never emits credentials, tokens, auth URLs, or
raw frames; a defensive redaction guard replaces any value carrying
provisioned token material with `[REDACTED]`.

## Tests

```
cargo test -p flauz-web-gateway --all-targets
cargo clippy -p flauz-web-gateway --all-targets -- -D warnings
cargo fmt -p flauz-web-gateway -- --check
```

The integration suite spawns a real gateway, a real WebSocket client,
and a real supervised (fake) app-server process
(`flauz-fake-app-server`, a bounded test double) covering: the
handshake-refusal law, the claim → brokered round-trip → streamed turn
notifications path, the kill → named `reconnecting` → restart →
`connected` → serving-again supervision path, static hosting + traversal
refusal, token-gated handshakes, and the non-local-bind refusal.
