# Protocol Compatibility

**Owner:** Tech Lead (COMP-001, Wave 8b) · **Pinned:** 2026-09-27 ·
**Applies to:** the JSON-RPC app-server protocol spoken by the codexRS desktop
client (`crates/codex-protocol`, `crates/codex-platform/src/app_server.rs`,
`crates/codex-app`).

This document is the compatibility contract for every future protocol change.
It records what is frozen, how evolution is allowed, what the version fields
promise, and how mismatches behave — each claim anchored to code or an
enforcing test on current `main`.

## 1. Scope and authority

The protocol is the newline-delimited JSON-RPC conversation between the native
client and a supervised official `codex app-server` process:

- the envelope (`ClientRequest`, `ClientNotification`, server requests,
  responses, and notifications) defined in `crates/codex-protocol/src/lib.rs`;
- every typed `*Params` / `*Response` / `*Notification` shape in that crate;
- the handshake sequence (`initialize` → response → `initialized`);
- the client capabilities object sent with `initialize`.

The behavioral reference is stable `26.721.3996.0`
(`reference/stable-26.721.3996.0/manifest.json`); the executable counterpart is
the pinned official CLI recorded in the FV gate
(`docs/research/evidence/fv-gate/FV-GATE-RECORD.md`, pinned CLI
`0.146.0-alpha.3.1`). The F2 contract kernel's freeze laws apply: field names
are stable, serialization is camelCase JSON, and a shape change is a contract
change requiring a recorded amendment.

## 2. Frozen surfaces and their enforcing tests

The law is the **wire shape**: the exact bytes a shape serializes to. Each
frozen surface is enforced by a byte-for-byte test.

| Frozen surface | Enforcing test |
| --- | --- |
| `initialize` request (with `capabilities: null`) and the `initialized` notification | `initialize_wire_shape_matches_generated_schema` (`crates/codex-protocol/src/lib.rs`) |
| `initialize` request **as spoken** (with the desktop capabilities object) | `initialize_wire_shape_with_capabilities_matches_generated_schema` (`crates/codex-protocol/src/lib.rs`, added by COMP-001) |
| Desktop capability values (experimental API, form elicitation, the stable opt-out method list) | `initialize_capabilities_match_the_stable_desktop_contract` (`crates/codex-app/src/backend.rs`) |
| `app/installed` request and response | `apps_installed_wire_shape_matches_generated_schema` (`crates/codex-protocol/src/lib.rs`) |
| `externalAgentConfig/detect`, `/import`, progress/completed notifications, histories read | the external-agent fixture test asserting encode/decode against schema-derived JSON (`crates/codex-protocol/src/lib.rs`) |
| Frame envelope limits and bounded reads | `read_bounded_frame` tests (`crates/codex-protocol/src/lib.rs`) |

The app-side typed callers (`crates/codex-platform/src/app_server.rs`,
`crates/codex-app/src/backend.rs`) use these types directly; there is no second
serialization path, so the tests above are the single gate.

## 3. The additive-only evolution rule

- **New optional fields are allowed** on existing shapes, encoded as
  `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]` so the
  wire bytes of existing conversations do not change for clients that never
  set them.
- **Renames and removals are never allowed.** A renamed field is a removed
  field plus an added one; either breaks every frozen byte assertion and the
  stable reference simultaneously.
- **New methods are allowed** (new request/response types plus a typed client
  method); they change nothing for a peer that never calls them.
- **Changing a field's type or semantics is a removal** and follows the
  removal rule: it is a breaking change, forbidden without a recorded
  architecture amendment and a version-policy decision.
- Any change to a frozen shape must update the enforcing test's expected bytes
  **in the same commit**, add a row to the compatibility matrix (§6), and
  reference the work order that authorized it. A shape change that lands
  without its test change is a contract violation by definition — the gate
  fails.

## 4. Version-field semantics

`ClientInfo.version` (`crates/codex-protocol/src/lib.rs`) is
**informational, not a gate**:

- the client sends its own build version
  (`env!("CARGO_PKG_VERSION")`, e.g. `crates/codex-app/src/main.rs` in the
  probe path and `crates/codex-app/src/backend.rs` in the app handshake);
- the server's response (`InitializeResponse.user_agent`,
  `codex_home`, `platform_family`, `platform_os`) is likewise informational;
- the handshake validates exactly one response field — `codex_home` — via
  `verify_reported_home` (`crates/codex-platform/src/app_server.rs`), which
  exists to guarantee supervision isolation, not protocol versioning;
- no code path in this repository compares `clientInfo.version` against any
  minimum, and the pinned official app-server accepts the handshake for every
  client version this repository has ever shipped (the FV gate executed the
  full scene battery with the same handshake across rc.12 → rc.14).

Consequently: **bumping the client version never changes protocol behavior.**
Protocol capability is expressed by the capabilities object and the methods the
client actually calls — never by the version string.

## 5. Mismatch behavior (observed truth)

What actually happens when the two sides disagree, anchored to code:

1. **Server sends a request method the client does not implement.** The client
   answers JSON-RPC `-32601 "unsupported client request"`
   (`crates/codex-app/src/backend.rs`, the fallthrough arm of the app-server
   request dispatcher). The conversation continues; no crash, no retry storm.
2. **Server sends a notification the client does not know.** The notification
   is ignored (`handle_notification`'s `_ => {}` fallthrough in
   `crates/codex-app/src/backend.rs`). This is what makes additive-only
   notification evolution safe: a newer server teaching the client new events
   degrades to silence on old clients.
3. **Client calls a method the server does not implement.** The official
   app-server returns a JSON-RPC error; the typed client surfaces it as
   `AppServerError::RequestFailed { code }`
   (`crates/codex-platform/src/app_server.rs`) and the call site reports the
   failure honestly. The client never guesses or falls back to an untyped
   variant.
4. **Handshake ordering violations.** `initialize` twice →
   `AppServerError::AlreadyInitialized`; calling before `initialize` →
   `NotInitialized`. Both are local guards; they never reach the wire as
   protocol traffic.
5. **Malformed or oversized frames.** Rejected by the bounded frame reader
   (`ProtocolError::FrameTooLarge` and friends) before any deserialization.

These five behaviors are the compatibility posture: **fail closed per call,
survive per conversation.**

## 6. Compatibility matrix (git-history-derived)

Every protocol-shape change that has landed on `main`, with the client version
that first spoke it. Versions are the workspace version at the committing
revision; "shape" lists only the wire-visible additions.

| First shipped in | Protocol shape added |
| --- | --- |
| 0.1.0-rc.1 | Bootstrap typed envelope; `initialize`/`initialized`; chat memory controls; external-agent import flow |
| 0.1.0-rc.2 | Remote-control methods; `review/start` RPC; critical event consistency fix |
| 0.1.0-rc.5 | Installed-apps runtime contract (`app/installed`, `app/list/updated` handling) |
| 0.1.0-rc.12 | Foundation + boundary lock: bounded envelope, frame limits, the frozen wire-shape tests themselves |
| 0.1.0-rc.14 | Current shape: all §2 surfaces frozen and enforced; capabilities contract pinned to the stable desktop set |

No released version has ever renamed or removed a protocol field; the matrix
therefore records additions only. Keeping that property is the policy this
document enforces.

## 7. The freeze gate

```
cargo test -p codex-protocol
```

must be green — in particular the `*_wire_shape_matches_generated_schema`
tests. The expected byte strings in those tests are the committed schema
artifact: they were derived from the generated schema of the pinned official
app-server, and any intentional change to them is a contract amendment that
must update this document and the matrix in the same commit. CI runs this
suite on every push (`cargo test --workspace` in `.github/workflows/ci.yml`).

## 8. Change procedure

1. Open a focused work order naming the surface, the addition, and the peer
   (official app-server version) that must tolerate it.
2. Implement additively (§3); update the enforcing test's expected bytes and
   this document's matrix in the same change.
3. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`,
   `cargo test --workspace` (the repository gate commands).
4. The Tech Lead merges only with the gate green and the matrix row present.

A change that cannot be expressed as an addition — a rename, a removal, a
semantic redefinition — requires an architecture amendment recorded before
implementation, per the F2 kernel's synchronization law.
