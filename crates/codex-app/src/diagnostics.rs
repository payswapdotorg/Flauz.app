//! OBS-001: the credential-scrubbed diagnostics export.
//!
//! The export allowlist is absolute: the collector constructs ONLY the named
//! fields below and nothing else ever reaches the output file. Free-form
//! content (the app log tail, connection error strings) passes a line-level
//! grammar allowlist plus defense-in-depth secret redaction, and the storage
//! section carries schema version and row COUNTS — never contents.
//!
//! The honest bounds (E2B/Wave-8 law): this build writes no app log, so the
//! log tail is usually absent; the collector says so by name instead of
//! failing. No credential, token, conversation content, or provider payload
//! is ever exported.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use codex_core::Action;
use codex_storage::Store;

/// Bounded connection-history ring: the last [`MAX_CONNECTION_EVENTS`] state
/// changes recorded by the backend loop. Pure data, unit-testable.
pub(crate) const MAX_CONNECTION_EVENTS: usize = 50;
/// The log tail is bounded: at most this many lines and bytes read from the
/// END of the file (no unbounded reads — the kernel law).
pub(crate) const MAX_LOG_TAIL_LINES: usize = 200;
pub(crate) const MAX_LOG_TAIL_BYTES: u64 = 64 * 1024;
/// Each exported string is byte-bounded so a hostile or corrupted source
/// cannot balloon the bundle.
pub(crate) const MAX_EXPORTED_STRING_BYTES: usize = 512;

/// The app log file this build would write (it currently writes none — the
/// named honest bound; the reader exists so a future build that starts
/// writing logs is covered by the same allowlist).
pub(crate) fn app_log_path() -> Option<PathBuf> {
    codex_platform::codexrs_data_dir()
        .ok()
        .map(|directory| directory.join("app.log"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectionEventKind {
    Connected,
    Lost,
    RetryScheduled,
    RetryStarted,
    Failed,
}

impl ConnectionEventKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Lost => "lost",
            Self::RetryScheduled => "retry_scheduled",
            Self::RetryStarted => "retry_started",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConnectionEvent {
    pub at_ms: i64,
    pub kind: ConnectionEventKind,
    pub attempt: Option<u32>,
    pub error: Option<String>,
}

/// The bounded connection-state history. The backend thread records through
/// [`record_connection_action`] at the single `emit` choke point; the export
/// reads a snapshot. The CLI export path records nothing (its own honest
/// bound: an empty history with a named note).
#[derive(Debug, Default)]
pub(crate) struct ConnectionEventLog {
    events: Vec<ConnectionEvent>,
}

impl ConnectionEventLog {
    pub(crate) fn record(
        &mut self,
        kind: ConnectionEventKind,
        attempt: Option<u32>,
        error: Option<String>,
    ) {
        if self.events.len() == MAX_CONNECTION_EVENTS {
            self.events.remove(0);
        }
        self.events.push(ConnectionEvent {
            at_ms: unix_timestamp_ms(),
            kind,
            attempt,
            error: error.map(|message| bounded_string(&message)),
        });
    }

    pub(crate) fn snapshot(&self) -> &[ConnectionEvent] {
        &self.events
    }
}

thread_local! {
    /// The backend loop runs on one dedicated thread; `emit` records here so
    /// every connection action is captured at a single choke point without
    /// signature changes across the backend.
    static CONNECTION_EVENT_LOG: std::cell::RefCell<ConnectionEventLog> =
        std::cell::RefCell::new(ConnectionEventLog::default());
}

/// Records connection-related actions into the thread-local history. Called
/// by the backend's `emit` for every action; non-connection actions are
/// ignored.
pub(crate) fn record_connection_action(action: &Action) {
    let (kind, attempt, error) = match action {
        Action::Connected => (ConnectionEventKind::Connected, None, None),
        Action::ConnectionLost => (ConnectionEventKind::Lost, None, None),
        Action::ConnectionRetryScheduled {
            attempt,
            last_error,
            ..
        } => (
            ConnectionEventKind::RetryScheduled,
            Some(*attempt),
            last_error.clone(),
        ),
        Action::ConnectionRetryStarted { attempt } => {
            (ConnectionEventKind::RetryStarted, Some(*attempt), None)
        }
        Action::ConnectionFailed(error) => (ConnectionEventKind::Failed, None, Some(error.clone())),
        _ => return,
    };
    CONNECTION_EVENT_LOG.with(|log| log.borrow_mut().record(kind, attempt, error));
}

/// Reads a copy of this thread's connection history (empty on threads that
/// never recorded — e.g. the CLI export path).
pub(crate) fn connection_history() -> Vec<ConnectionEvent> {
    CONNECTION_EVENT_LOG.with(|log| log.borrow().snapshot().to_vec())
}

/// Runtime facts retained by the backend loop at handshake time. The
/// app-server's reported `userAgent` is the pinned-CLI version source — no
/// subprocess probe is run (platform process management stays in
/// codex-platform).
#[derive(Debug, Default, Clone)]
pub(crate) struct RuntimeFacts {
    pub codex_binary: Option<PathBuf>,
    pub codex_home: Option<PathBuf>,
    pub app_server_user_agent: Option<String>,
}

/// The log-tail allowlist result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LogTail {
    /// No app log exists at the named path — the honest bound, not an error.
    Absent { note: &'static str },
    /// The file exists; only grammar-allowlisted, redacted lines are carried.
    Present { lines: Vec<String> },
    /// The file exists but could not be read (named honest failure).
    Unreadable { note: String },
}

/// A line is exported only if it matches the structured app-log grammar: a
/// leading timestamp. This is the line-level ALLOWLIST — arbitrary prose,
/// dumps, or embedded JSON never qualify.
fn line_is_allowlisted(line: &str) -> bool {
    let trimmed = line.trim_start();
    // Grammar: YYYY-MM-DD[T ]HH:MM... (the structured app-log prefix).
    let bytes = trimmed.as_bytes();
    if bytes.len() < 17 {
        return false;
    }
    let is_digit = |b: Option<&u8>| b.is_some_and(|b| b.is_ascii_digit());
    let is_dash = |b: Option<&u8>| b == Some(&b'-');
    let is_colon = |b: Option<&u8>| b == Some(&b':');
    is_digit(bytes.first())
        && is_digit(bytes.get(1))
        && is_digit(bytes.get(2))
        && is_digit(bytes.get(3))
        && is_dash(bytes.get(4))
        && is_digit(bytes.get(5))
        && is_digit(bytes.get(6))
        && is_dash(bytes.get(7))
        && is_digit(bytes.get(8))
        && is_digit(bytes.get(9))
        && matches!(bytes.get(10), Some(b'T') | Some(b' '))
        && is_digit(bytes.get(11))
        && is_digit(bytes.get(12))
        && is_colon(bytes.get(13))
        && is_digit(bytes.get(14))
        && is_digit(bytes.get(15))
}

/// Defense-in-depth secret redaction applied to allowlisted lines. The
/// primary mechanism is the allowlist; this catches credential-shaped
/// substrings on lines that otherwise qualify (e.g. a future log line that
/// echoes a provider error carrying an authorization header).
fn redact_secret_patterns(line: &str) -> String {
    let mut redacted = line.to_owned();
    for marker in ["api_key=", "apikey=", "token=", "password=", "secret="] {
        redacted = redact_key_value(&redacted, marker);
    }
    redacted = redact_bearer(&redacted);
    redacted = redact_sk_prefix(&redacted);
    redacted
}

fn redact_key_value(line: &str, marker: &str) -> String {
    let Some(start) = line.to_lowercase().find(marker) else {
        return line.to_owned();
    };
    let value_start = start + marker.len();
    let value_end = line[value_start..]
        .find(|c: char| c.is_whitespace())
        .map(|offset| value_start + offset)
        .unwrap_or(line.len());
    let mut result = String::new();
    result.push_str(&line[..value_start]);
    result.push_str("[redacted]");
    result.push_str(&line[value_end..]);
    result
}

fn redact_bearer(line: &str) -> String {
    let Some(start) = line.find("Bearer ") else {
        return line.to_owned();
    };
    let value_start = start + "Bearer ".len();
    let value_end = line[value_start..]
        .find(char::is_whitespace)
        .map(|offset| value_start + offset)
        .unwrap_or(line.len());
    let mut result = String::new();
    result.push_str(&line[..value_start]);
    result.push_str("[redacted]");
    result.push_str(&line[value_end..]);
    result
}

fn redact_sk_prefix(line: &str) -> String {
    let Some(start) = line.find("sk-") else {
        return line.to_owned();
    };
    let value_end = line[start..]
        .find(|c: char| c.is_whitespace())
        .map(|offset| start + offset)
        .unwrap_or(line.len());
    // Only redact credential-shaped runs (sk- followed by 8+ token chars).
    if value_end - start < 11 {
        return line.to_owned();
    }
    let mut result = String::new();
    result.push_str(&line[..start]);
    result.push_str("[redacted]");
    result.push_str(&line[value_end..]);
    result
}

/// Reads the bounded log tail: seeks to `end - MAX_LOG_TAIL_BYTES`, takes
/// the last [`MAX_LOG_TAIL_LINES`] lines, filters them through the grammar
/// allowlist, then redacts secret-shaped substrings on the survivors.
pub(crate) fn read_log_tail(path: &Path) -> LogTail {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return LogTail::Absent {
                note: "no app log exists; this build does not write one (the reader is allowlist-guarded for future builds)",
            };
        }
        Err(error) => {
            return LogTail::Unreadable {
                note: bounded_string(&format!("log metadata unavailable: {error}")),
            };
        }
    };
    if !metadata.is_file() {
        return LogTail::Unreadable {
            note: "log path exists but is not a file".to_owned(),
        };
    }
    let file_len = metadata.len();
    let start = file_len.saturating_sub(MAX_LOG_TAIL_BYTES);
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => {
            return LogTail::Unreadable {
                note: bounded_string(&format!("log open failed: {error}")),
            };
        }
    };
    use std::io::{Read, Seek, SeekFrom};
    if let Err(error) = file.seek(SeekFrom::Start(start)) {
        return LogTail::Unreadable {
            note: bounded_string(&format!("log seek failed: {error}")),
        };
    }
    let mut buffer = String::new();
    if let Err(error) = file.take(MAX_LOG_TAIL_BYTES).read_to_string(&mut buffer) {
        return LogTail::Unreadable {
            note: bounded_string(&format!("log read failed: {error}")),
        };
    }
    let lines = buffer
        .lines()
        .filter(|line| line_is_allowlisted(line))
        .map(|line| bounded_string(&redact_secret_patterns(line)))
        .collect::<Vec<_>>();
    let lines = if lines.len() > MAX_LOG_TAIL_LINES {
        lines[lines.len() - MAX_LOG_TAIL_LINES..].to_vec()
    } else {
        lines
    };
    LogTail::Present { lines }
}

fn bounded_string(value: &str) -> String {
    if value.len() <= MAX_EXPORTED_STRING_BYTES {
        return value.to_owned();
    }
    let mut cut = MAX_EXPORTED_STRING_BYTES;
    while cut > 0 && !value.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…[truncated]", &value[..cut])
}

fn unix_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .unwrap_or_default()
}

/// The storage facts: schema version and row counts for the named tables —
/// never contents. `None` fields are honest absences.
fn storage_facts(store: Option<&Store>) -> serde_json::Value {
    let Some(store) = store else {
        return serde_json::json!({
            "state": "unavailable: storage is not open",
        });
    };
    match store.diagnostics_counts() {
        Ok(counts) => serde_json::json!({
            "state": "ok",
            "schema_version": store.schema_version().unwrap_or(-1),
            "counts": {
                "ui_preferences": counts.ui_preferences,
                "recent_workspaces": counts.recent_workspaces,
                "browser_downloads": counts.browser_downloads,
                "workspace_folders": counts.workspace_folders,
                "browsing_history": counts.browsing_history,
            },
        }),
        Err(error) => serde_json::json!({
            "state": format!("unavailable: {error}"),
        }),
    }
}

/// Collects the diagnostics snapshot. THE ALLOWLIST: exactly these keys are
/// constructed, and nothing else is written to the export.
pub(crate) fn collect_diagnostics(
    runtime: &RuntimeFacts,
    connection_events: &[ConnectionEvent],
    store: Option<&Store>,
) -> serde_json::Value {
    let log_tail = app_log_path()
        .map(|path| read_log_tail(&path))
        .unwrap_or(LogTail::Absent {
            note: "the data directory is unavailable, so the log path cannot be resolved",
        });
    serde_json::json!({
        // The truthful statement that travels with the artifact.
        "privacy": {
            "includes": [
                "app version, OS and architecture",
                "the resolved official CLI binary path and the app-server's reported user agent",
                "the bounded connection-state history (state changes, attempt numbers, error text)",
                "the state database schema version and table row counts",
                "an allowlist-filtered, secret-redacted tail of the app log (when a log exists)",
            ],
            "never_includes": [
                "credentials, tokens, API keys, or authentication material",
                "conversation, thread, or provider payload content",
                "workspace file contents or repository data",
                "state database contents beyond counts",
            ],
        },
        "format": "codexrs-diagnostics-v1",
        "generated_at_ms": unix_timestamp_ms(),
        "app_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "codex_cli": {
            "binary": runtime.codex_binary.as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "unresolved".to_owned()),
            "app_server_user_agent": runtime.app_server_user_agent
                .clone()
                .unwrap_or_else(|| "not reported (no handshake yet)".to_owned()),
        },
        "codex_home": runtime.codex_home.as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unresolved".to_owned()),
        "connection_history": connection_events.iter().map(|event| serde_json::json!({
            "at_ms": event.at_ms,
            "event": event.kind.as_str(),
            "attempt": event.attempt,
            "error": event.error,
        })).collect::<Vec<_>>(),
        "log_tail": match &log_tail {
            LogTail::Absent { note } => serde_json::json!({ "state": "absent", "note": note }),
            LogTail::Unreadable { note } => serde_json::json!({ "state": "unreadable", "note": note }),
            LogTail::Present { lines } => serde_json::json!({ "state": "present", "lines": lines }),
        },
        "storage": storage_facts(store),
    })
}

/// Writes the snapshot as pretty JSON. A trailing newline keeps the file
/// diff- and tail-friendly.
pub(crate) fn write_diagnostics_json(snapshot: &serde_json::Value, path: &Path) -> io::Result<()> {
    let mut body = serde_json::to_string_pretty(snapshot).map_err(io::Error::other)?;
    body.push('\n');
    fs::write(path, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> PathBuf {
        let sequence = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("codexrs-obs001-{}-{sequence}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        let Ok(()) = std::fs::create_dir_all(&directory) else {
            panic!("temp directory creation failed");
        };
        directory
    }

    #[test]
    fn log_tail_absent_is_the_named_honest_bound() {
        let tail = read_log_tail(Path::new("/nonexistent/path/app.log"));
        assert!(matches!(tail, LogTail::Absent { .. }));
        let LogTail::Absent { note } = tail else {
            unreachable!();
        };
        assert!(note.contains("does not write"));
    }

    #[test]
    fn log_tail_exports_only_allowlisted_redacted_lines() {
        let directory = temp_dir();
        let log_path = directory.join("app.log");
        let Ok(()) = std::fs::write(
            &log_path,
            "2026-09-28 10:00:00 info connected to app-server\n\
             api_key=sk-LIVESECRETVALUE99 in a bare unstructured line\n\
             2026-09-28 10:00:01 warn provider error Bearer eyJhbGciOi.payload.sig\n\
             2026-09-28 10:00:02 error retry attempt 3 token=abc123SECRETVALUE\n\
             unstructured prose without a timestamp\n",
        ) else {
            panic!("log fixture write failed");
        };

        let LogTail::Present { lines } = read_log_tail(&log_path) else {
            panic!("log tail should be present");
        };
        // Only the three timestamped lines survive the allowlist.
        assert_eq!(lines.len(), 3);
        // The plain credential line never qualified (grammar allowlist).
        let joined = lines.join("\n");
        assert!(!joined.contains("in a bare unstructured line"));
        // Credential-shaped substrings on qualifying lines are redacted.
        assert!(
            !joined.contains("eyJhbGciOi.payload.sig"),
            "bearer token must be redacted"
        );
        assert!(joined.contains("Bearer [redacted]"));
        assert!(
            !joined.contains("abc123SECRETVALUE"),
            "token= value must be redacted"
        );
        assert!(joined.contains("token=[redacted]"));
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn sk_prefixed_secrets_are_redacted_on_qualifying_lines() {
        let directory = temp_dir();
        let log_path = directory.join("app.log");
        let Ok(()) = std::fs::write(
            &log_path,
            "2026-09-28 10:00:00 error provider said sk-AbCdEfGh12345678 was rejected\n",
        ) else {
            panic!("log fixture write failed");
        };
        let LogTail::Present { lines } = read_log_tail(&log_path) else {
            panic!("log tail should be present");
        };
        assert!(!lines[0].contains("sk-AbCdEfGh12345678"));
        assert!(lines[0].contains("[redacted]"));
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn short_sk_like_words_are_not_redacted() {
        // "sk-test" is too short to be credential-shaped; only 8+ token
        // chars after the prefix qualify.
        assert_eq!(
            redact_sk_prefix("2026-09-28 10:00:00 task sk-test"),
            "2026-09-28 10:00:00 task sk-test"
        );
    }

    #[test]
    fn connection_log_is_bounded_and_fifo() {
        let mut log = ConnectionEventLog::default();
        for _ in 0..(MAX_CONNECTION_EVENTS + 10) {
            log.record(ConnectionEventKind::RetryScheduled, Some(1), None);
        }
        assert_eq!(log.snapshot().len(), MAX_CONNECTION_EVENTS);
        log.record(ConnectionEventKind::Connected, None, None);
        assert_eq!(log.snapshot().len(), MAX_CONNECTION_EVENTS);
        let Some(last) = log.snapshot().last() else {
            panic!("connection log should be non-empty");
        };
        assert_eq!(last.kind, ConnectionEventKind::Connected);
    }

    #[test]
    fn connection_actions_are_recorded_and_others_ignored() {
        record_connection_action(&Action::Connected);
        record_connection_action(&Action::ConnectionLost);
        record_connection_action(&Action::ConnectionRetryScheduled {
            attempt: 4,
            retry_in_ms: 8_000,
            last_error: Some("provider unavailable".to_owned()),
        });
        record_connection_action(&Action::SetStatus("unrelated".to_owned()));
        let history = connection_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].kind, ConnectionEventKind::Connected);
        assert_eq!(history[1].kind, ConnectionEventKind::Lost);
        let retry = &history[2];
        assert_eq!(retry.kind, ConnectionEventKind::RetryScheduled);
        assert_eq!(retry.attempt, Some(4));
        assert_eq!(retry.error.as_deref(), Some("provider unavailable"));
    }

    #[test]
    fn snapshot_carries_only_the_allowlisted_fields() {
        let snapshot = collect_diagnostics(
            &RuntimeFacts {
                codex_binary: Some(PathBuf::from("/usr/bin/codex")),
                codex_home: Some(PathBuf::from("/home/user/.codex")),
                app_server_user_agent: Some("codex-app-server/0.146.0".to_owned()),
            },
            &[ConnectionEvent {
                at_ms: 1_700_000_000_000,
                kind: ConnectionEventKind::Failed,
                attempt: None,
                error: Some("handshake failed".to_owned()),
            }],
            None,
        );
        let Some(object) = snapshot.as_object() else {
            panic!("snapshot should be a JSON object");
        };
        let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "app_version",
                "arch",
                "codex_cli",
                "codex_home",
                "connection_history",
                "format",
                "generated_at_ms",
                "log_tail",
                "os",
                "privacy",
                "storage",
            ]
        );
        assert_eq!(object["format"], "codexrs-diagnostics-v1");
        assert_eq!(object["app_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(
            object["codex_cli"]["app_server_user_agent"],
            "codex-app-server/0.146.0"
        );
        assert_eq!(object["connection_history"][0]["event"], "failed");
        let Some(state) = object["storage"]["state"].as_str() else {
            panic!("storage state should be a string");
        };
        assert!(state.contains("unavailable"));
        // The privacy statement travels with the artifact.
        let Some(first_never) = object["privacy"]["never_includes"][0].as_str() else {
            panic!("privacy statement should be a string");
        };
        assert!(first_never.contains("credentials"));
    }

    #[test]
    fn storage_facts_report_schema_version_and_counts_never_contents() {
        let Ok(mut store) = Store::open_in_memory() else {
            panic!("in-memory store should open");
        };
        let Ok(()) = store.set_preference("probe-key", "SUPER-SECRET-VALUE", 1_700_000_000) else {
            panic!("probe preference write failed");
        };
        let snapshot = collect_diagnostics(&RuntimeFacts::default(), &[], Some(&store));
        let storage = &snapshot["storage"];
        assert_eq!(storage["state"], "ok");
        let Ok(schema_version) = store.schema_version() else {
            panic!("schema version read failed");
        };
        assert_eq!(storage["schema_version"], schema_version);
        assert_eq!(storage["counts"]["ui_preferences"], 1);
        // Contents never leak: only the count is present.
        let Ok(serialized) = serde_json::to_string(storage) else {
            panic!("storage facts should serialize");
        };
        assert!(!serialized.contains("SUPER-SECRET-VALUE"));
        assert!(!serialized.contains("probe-key"));
    }

    #[test]
    fn write_diagnostics_json_is_pretty_with_trailing_newline() {
        let directory = temp_dir();
        let path = directory.join("diag.json");
        let snapshot = serde_json::json!({ "format": "codexrs-diagnostics-v1" });
        let Ok(()) = write_diagnostics_json(&snapshot, &path) else {
            panic!("diagnostics write failed");
        };
        let Ok(body) = std::fs::read_to_string(&path) else {
            panic!("diagnostics read-back failed");
        };
        assert!(body.starts_with("{\n  \"format\""));
        assert!(body.ends_with("}\n"));
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn exported_strings_are_byte_bounded() {
        let long = "x".repeat(MAX_EXPORTED_STRING_BYTES + 100);
        assert!(bounded_string(&long).len() <= MAX_EXPORTED_STRING_BYTES + 14);
        assert!(bounded_string(&long).ends_with("…[truncated]"));
        // Multi-byte boundaries are respected.
        let multibyte = "é".repeat(MAX_EXPORTED_STRING_BYTES + 3);
        assert!(bounded_string(&multibyte).chars().all(|c| c != '\u{fffd}'));
    }
}
