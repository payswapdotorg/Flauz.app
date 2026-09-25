//! Structured JSON-lines logging for the gateway.
//!
//! Every log line is a single JSON object with a bounded, pre-named field
//! set. The logger NEVER receives and NEVER emits credentials, session
//! tokens, auth URLs, or raw protocol frames: a defensive redaction scan
//! rejects (and names) any field value that carries configured secret
//! material, and frame/token payloads are never passed to the logger in
//! the first place.

use std::io::Write;
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;

use serde_json::Value;

/// The maximum length of a single field value that is emitted.
pub const MAX_FIELD_BYTES: usize = 512;

/// The maximum number of fields in one event.
pub const MAX_FIELDS: usize = 12;

/// A structured event logger writing JSON lines to standard output.
#[derive(Debug, Clone)]
pub struct Logger {
    sink: Sink,
    /// Secret material that must never appear in output (defensive
    /// redaction guard; normally the logger is simply never handed
    /// secrets).
    redactions: Arc<Vec<String>>,
}

/// Where structured lines are written: standard output for the served
/// gateway, or a caller-owned capture buffer for tests and the
/// no-credential log scan.
#[derive(Debug, Clone)]
enum Sink {
    Stdout,
    #[cfg(test)]
    Buffer(Arc<Mutex<Vec<u8>>>),
}

impl Sink {
    fn write_line(&self, line: &str) {
        match self {
            Self::Stdout => {
                let mut sink = std::io::stdout().lock();
                let _ = writeln!(sink, "{line}");
            }
            #[cfg(test)]
            Self::Buffer(buffer) => {
                let mut sink = buffer
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let _ = writeln!(sink, "{line}");
            }
        }
    }
}

impl Logger {
    /// Creates a logger writing to standard output.
    #[must_use]
    pub fn stdout() -> Self {
        Self {
            sink: Sink::Stdout,
            redactions: Arc::new(Vec::new()),
        }
    }

    /// Creates a logger that additionally redacts the provided secret
    /// substrings defensively (tokens provisioned by the operator).
    #[must_use]
    pub fn with_redactions(redactions: Vec<String>) -> Self {
        Self {
            sink: Sink::Stdout,
            redactions: Arc::new(redactions),
        }
    }

    /// Emits one structured event. Field values are bounded and scanned
    /// against the redaction set; a value carrying secret material is
    /// replaced with `"[REDACTED]"` (and never logged as-is).
    pub fn log(&self, level: &str, event: &str, fields: &[(&str, &str)]) {
        let mut record = serde_json::Map::new();
        record.insert("ts".to_owned(), Value::String(now_rfc3339()));
        record.insert(
            "level".to_owned(),
            Value::String(level.to_ascii_lowercase()),
        );
        record.insert("event".to_owned(), Value::String(event.to_owned()));
        for (name, value) in fields.iter().take(MAX_FIELDS) {
            record.insert(
                sanitize_field_name(name),
                Value::String(sanitize_value(value, &self.redactions)),
            );
        }
        let line = Value::Object(record);
        self.sink.write_line(&line.to_string());
    }

    /// Emits an `info` event.
    pub fn info(&self, event: &str, fields: &[(&str, &str)]) {
        self.log("info", event, fields);
    }

    /// Emits a `warn` event.
    pub fn warn(&self, event: &str, fields: &[(&str, &str)]) {
        self.log("warn", event, fields);
    }

    /// Emits an `error` event.
    pub fn error(&self, event: &str, fields: &[(&str, &str)]) {
        self.log("error", event, fields);
    }

    /// Builds a logger writing into a caller-owned buffer (tests and the
    /// no-credential log scan).
    #[cfg(test)]
    #[must_use]
    pub fn into_buffer(buffer: Arc<Mutex<Vec<u8>>>, redactions: Vec<String>) -> Self {
        Self {
            sink: Sink::Buffer(buffer),
            redactions: Arc::new(redactions),
        }
    }
}

fn sanitize_field_name(name: &str) -> String {
    if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        "field".to_owned()
    } else {
        name.to_owned()
    }
}

fn sanitize_value(value: &str, redactions: &[String]) -> String {
    let mut sanitized = value.to_owned();
    for secret in redactions {
        if !secret.is_empty() && sanitized.contains(secret.as_str()) {
            sanitized = "[REDACTED]".to_owned();
            break;
        }
    }
    if sanitized.len() > MAX_FIELD_BYTES {
        let mut truncated = String::new();
        for ch in sanitized.chars().take(MAX_FIELD_BYTES) {
            if truncated.len() + ch.len_utf8() > MAX_FIELD_BYTES {
                break;
            }
            truncated.push(ch);
        }
        truncated.push('…');
        return truncated;
    }
    sanitized
}

fn now_rfc3339() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    // RFC 3339 UTC, seconds precision (the F2 canonical form).
    let days = now / 86_400;
    let seconds_of_day = now % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Converts days since the Unix epoch to a civil (year, month, day)
/// date (Howard Hinnant's algorithm; no external time dependency).
fn civil_from_days(days: u64) -> (i64, u32, u32) {
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::Logger;

    fn capture_logger() -> (Logger, Arc<Mutex<Vec<u8>>>) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        (Logger::into_buffer(Arc::clone(&buffer), Vec::new()), buffer)
    }

    #[test]
    fn events_are_single_json_lines_with_ts_level_event() {
        let (logger, buffer) = capture_logger();
        logger.info(
            "session_opened",
            &[("session_id", "gwsess_abc"), ("origin", "local")],
        );
        let line = String::from_utf8(
            buffer
                .lock()
                .map(|mut b| std::mem::take(&mut *b))
                .unwrap_or_default(),
        )
        .unwrap_or_default();
        let record: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|error| panic!("log line was not JSON ({error}): {line}"));
        assert_eq!(record["level"], "info");
        assert_eq!(record["event"], "session_opened");
        assert_eq!(record["session_id"], "gwsess_abc");
        assert!(record["ts"].as_str().map(str::len).unwrap_or(0) >= 20);
    }

    #[test]
    fn no_credential_material_is_ever_emitted() {
        let secret = "0123456789abcdef0123456789abcdef";
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let logger = Logger::into_buffer(Arc::clone(&buffer), vec![secret.to_owned()]);
        // A hostile caller tries to log secret material in every field.
        logger.info(
            "session_claimed",
            &[
                ("session_id", secret),
                ("note", &format!("token {secret} inline")),
                ("url", "https://example.test/?token=abc"),
            ],
        );
        let line = String::from_utf8(
            buffer
                .lock()
                .map(|mut b| std::mem::take(&mut *b))
                .unwrap_or_default(),
        )
        .unwrap_or_default();
        assert!(
            !line.contains(secret),
            "secret material appeared in the log output: {line}"
        );
        let record: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|error| panic!("log line was not JSON ({error}): {line}"));
        assert_eq!(record["session_id"], "[REDACTED]");
        assert_eq!(record["note"], "[REDACTED]");
    }

    #[test]
    fn oversized_values_are_truncated() {
        let (logger, buffer) = capture_logger();
        let long = "x".repeat(5_000);
        logger.info("event", &[("detail", &long)]);
        let line = String::from_utf8(
            buffer
                .lock()
                .map(|mut b| std::mem::take(&mut *b))
                .unwrap_or_default(),
        )
        .unwrap_or_default();
        assert!(line.len() < 1_000, "oversized field was not truncated");
    }
}
