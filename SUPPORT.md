# Support

Choose the channel that matches the request:

- use [GitHub Discussions](https://github.com/Kiwunaka/codexRS/discussions) for
  setup help, ideas, and general questions;
- use [GitHub Issues](https://github.com/Kiwunaka/codexRS/issues) for
  reproducible codexRS bugs and scoped feature proposals;
- use [private vulnerability reporting](https://github.com/Kiwunaka/codexRS/security/advisories/new)
  for security reports.

Before opening a bug, check the
[platform matrix](docs/platform-support.md), update to the latest candidate, and
verify the official `codex` executable separately.

Include the codexRS version or commit, operating system, desktop session on
Linux, output of `codex --version`, expected behavior, actual behavior, and
minimal reproduction steps. Redact paths and never attach credentials, live
`.codex` data, raw provider payloads, or private repository content.

## Exporting a diagnostics bundle (OBS-001)

When you report a bug, you can attach a support diagnostics bundle:

- in the app: press `Ctrl+K` and run **Export diagnostics**, or
- from a terminal: `codexrs --diagnostics-out <PATH>`.

The bundle is credential-scrubbed by allowlist. It includes the app version,
OS and architecture facts, the resolved official CLI binary and the
app-server's reported user agent, the bounded connection-state history
(state changes, attempt numbers, error text), the state database schema
version and table row counts, and — when an app log exists — an
allowlist-filtered, secret-redacted tail of it. It never includes
credentials, tokens, conversation or provider payload content, workspace
file contents, or any state database contents beyond counts. Each bundle
carries its own `privacy` section restating exactly this.

## Provider and runtime outage behavior (OBS-001)

What the app does when the runtime or provider is down, with the repository
evidence for each behavior:

| Behavior | What happens | Evidence |
| --- | --- | --- |
| Connection state machine | The connection surface tracks Offline / Connecting / Online / Recovering (with attempt number, retry countdown, and last error) / Failed. | `ConnectionStatus` in `crates/codex-core/src/lib.rs`; connection actions in `crates/codex-app/src/backend.rs` |
| Reconnect cadence | Automatic reconnect with bounded exponential backoff: 1 s, 2 s, 4 s, 8 s, 16 s, then capped at 20 s, one in-flight attempt at a time (duplicates coalesced). The cadence resets to 1 s after a successful reconnect. | `AppServerReconnectScheduler` and the test `app_server_reconnect_uses_bounded_exponential_backoff_and_deduplicates` (`crates/codex-app/src/backend.rs`) |
| UI event backpressure | If the UI cannot keep up, events are byte-budgeted and lower-priority notifications are coalesced with a visible status note; connection-loss is never dropped (priority law). | `UiBackpressureSignals`, `semantic_ui_queue_overload_prioritizes_connection_loss` (`crates/codex-app/src/backend.rs`) |
| Interrupted turns | A turn interrupted by a dropped connection is safety-buffered with bounded retry metadata and surfaces an honest retry affordance; committed steers are preserved with message boundaries; a stale interrupted turn is never retried over newer turns. | `safety_retry_preserves_committed_steers_with_message_boundaries`, `safety_retry_requires_the_interrupted_turn_to_remain_latest`, `safety_buffering_notification_keeps_only_bounded_retry_metadata` (`crates/codex-app/src/backend.rs`) |
| Composer drafts | Draft text lives in UI state only and is not cleared by connection loss; a failed submit becomes a retryable turn instead of discarding the draft. | The retryable-turn paths above; draft retention across reconnects is exercised by the FV journey battery (`docs/research/evidence/fv-gate/`) |
| Honest failure states | Provider/account failures surface bounded, truthful status messages (never a fake success); offline start is tolerated. | `ConnectionFailed` handling and the bounded status paths (`crates/codex-app/src/backend.rs`); the Wave-7 FV gate record (`docs/research/evidence/fv-gate/FV-GATE-RECORD.md`) |

This community project has no guaranteed response time or commercial support
SLA.
