# Security policy

## Supported versions

| Version | Security updates |
| --- | --- |
| `main` | Yes |
| Latest pre-release | Yes |
| Older pre-releases | No |

Until the first stable release, fixes land on `main` and the next
candidate.

## Reporting a vulnerability

Do not open a public issue. Use
[GitHub private vulnerability reporting](https://github.com/payswapdotorg/Flauz.app/security/advisories/new)
and include:

- the affected commit or release;
- platform and official Codex CLI version;
- the trust boundary involved;
- minimal reproduction steps using isolated fixtures;
- impact and any known mitigations.

Remove credentials, tokens, private keys, user history, screenshots, and
raw provider payloads before reporting. A maintainer will acknowledge the
report as capacity allows, confirm scope, and coordinate disclosure after
a fix is available.

## Security posture

The production security review of record is
[docs/research/SECURITY-REVIEW-2026-09.md](docs/research/SECURITY-REVIEW-2026-09.md)
(SEC-001, Wave 8): the grep-evidenced credential-law audit across every
crate and the web client (1,058 raw pattern hits triaged; **zero
credential-material violations**), the `flauz-web-gateway`
security-surface review (bind law, session-token law, origin/WSS
handling, request-timeout and in-flight caps, static-file traversal
refusal — each with code pointers), and the threat model.

**Reviewed and enforced by the code:**

- credentials are references only (`flausec_...`) — credential-shaped
  values are rejected at every contract boundary;
- the gateway binds `127.0.0.1` by default and refuses non-loopback
  binds without provisioned session tokens;
- the gateway WebSocket requires the `session.claim` handshake before
  any bridged frame; frames, methods, sessions, and in-flight requests
  are bounded;
- static hosting refuses path traversal and serves bounded files with
  `nosniff`;
- structured logs never receive credential material and defensively
  redact provisioned tokens;
- the web client persists no token material client-side.

**Operator-owned (not provided by this software):**

- TLS termination for any non-local gateway deployment (front the
  gateway with a TLS reverse proxy — see finding SEC-F-01 in the review);
- the provisioning and file permissions of `--session-token-file` and
  `CODEX_HOME`;
- the official Codex CLI installation and provider accounts.

**Deferred and tracked (named, not hidden):**

- web-client authenticated-journey verification rides the W-AUTH
  operator-credential recovery path (the FV gate record);
- the remaining review findings (SEC-F-02..F-06, none blocking) are
  recorded with recovery paths in the review's findings register.

## In scope

- app-server framing, request routing, and approval handling;
- process-tree supervision and shutdown;
- direct access to live Codex-owned files;
- Computer Use window selection, capture, and input authorization;
- codexRS-owned SQLite state;
- Git and terminal command boundaries;
- the `flauz-web-gateway` transport bridge and static hosting;
- release artifacts and dependency-policy bypasses.

## Out of scope

- vulnerabilities in the separately installed official Codex CLI or
  provider services;
- unsupported modified builds;
- issues that require exposing third-party accounts, credentials, or
  data;
- social engineering and denial-of-service testing against public
  services.

Upstream Codex issues should follow the
[OpenAI Codex security policy](https://github.com/openai/codex/security/policy).
