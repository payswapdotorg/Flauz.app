//! `flauz-fake-app-server` — a deterministic, bounded app-server test
//! double for the gateway's integration tests (never shipped, never used
//! against live state).
//!
//! Speaks the app-server wire dialect on stdio: newline-delimited JSON
//! frames `{"method", "id", "params"?}` / `{"id", "result" | "error"}` /
//! notifications `{"method", "params"}` — with NO `jsonrpc` field,
//! exactly like the official `codex app-server`.
//!
//! Supported surface (the WEB-001 integration slice): `initialize` +
//! `initialized`, `account/read`, `getAuthStatus`, `thread/list`,
//! `thread/read`, `thread/resume`, `turn/start` (streaming
//! `turn/started` → `item/completed` → `turn/completed`), and the
//! test-only `test/exit` (exits the process so the gateway's supervision
//! restart path can be exercised).

use std::io::{BufRead, Write};

const MAX_LINE_BYTES: usize = 1024 * 1024;

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut errors = 0;
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        if line.len() > MAX_LINE_BYTES {
            eprintln!("frame too large: {}", line.len());
            break;
        }
        let value: serde_json::Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => {
                errors += 1;
                eprintln!("malformed frame #{errors}");
                continue;
            }
        };
        let Some(object) = value.as_object() else {
            continue;
        };
        let id = object.get("id").cloned();
        let method = object
            .get("method")
            .and_then(|m| m.as_str())
            .map(str::to_owned);
        match (id, method) {
            (Some(id), Some(method)) => {
                if !handle_request(&mut stdout, &id, &method) {
                    break;
                }
            }
            (None, Some(method)) => {
                if method == "initialized" {
                    // The initialized notification needs no response.
                } else {
                    eprintln!("notification: {method}");
                }
            }
            _ => {}
        }
        let _ = stdout.flush();
    }
}

/// Handles one request; returns `false` when the process should exit
/// (the `test/exit` hook).
fn handle_request(stdout: &mut std::io::Stdout, id: &serde_json::Value, method: &str) -> bool {
    use serde_json::json;
    let home = std::env::var("CODEX_HOME").unwrap_or_default();
    match method {
        "initialize" => respond(
            stdout,
            id,
            json!({
                "userAgent": "flauz-fake-app-server/1",
                "codexHome": home,
                "platformFamily": "unix",
                "platformOs": std::env::consts::OS,
            }),
        ),
        "account/read" => respond(
            stdout,
            id,
            json!({
                "account": {
                    "type": "chatgpt",
                    "email": "user@example.test",
                    "planType": "pro",
                },
                "requiresOpenaiAuth": false,
            }),
        ),
        "getAuthStatus" => respond(
            stdout,
            id,
            json!({
                "authMethod": "chatgpt",
                "accountId": "acct_fake",
                "requiresOpenaiAuth": false,
            }),
        ),
        "account/logout" => respond(stdout, id, json!({})),
        "thread/list" => respond(
            stdout,
            id,
            json!({
                "data": [
                    {
                        "id": "thread-fake-1",
                        "sessionId": "sess-fake-1",
                        "preview": "Plan the community garden layout",
                        "name": "Community garden layout",
                        "cwd": "/tmp/garden",
                        "createdAt": 1_760_000_000_i64,
                        "updatedAt": 1_760_000_100_i64,
                        "recencyAt": 1_760_000_100_i64,
                        "status": {},
                        "gitInfo": null,
                        "turns": [],
                    },
                    {
                        "id": "thread-fake-2",
                        "sessionId": "sess-fake-2",
                        "preview": "Quarterly budget analysis for the studio",
                        "name": "Studio budget analysis",
                        "cwd": "/tmp/studio",
                        "createdAt": 1_760_000_200_i64,
                        "updatedAt": 1_760_000_300_i64,
                        "recencyAt": 1_760_000_300_i64,
                        "status": {},
                        "gitInfo": null,
                        "turns": [],
                    },
                ],
                "nextCursor": null,
                "backwardsCursor": null,
            }),
        ),
        "thread/read" => respond(
            stdout,
            id,
            json!({
                "thread": thread_fixture(),
            }),
        ),
        "thread/resume" => respond(
            stdout,
            id,
            json!({
                "thread": thread_fixture(),
                "initialTurnsPage": {
                    "data": [],
                    "nextCursor": null,
                    "backwardsCursor": null,
                },
                "model": "gpt-5.6-sol",
                "reasoningEffort": "high",
            }),
        ),
        "turn/start" => {
            notify(
                stdout,
                "turn/started",
                json!({"threadId": "thread-fake-1", "turn": {"id": "turn-1", "status": "running"}}),
            );
            notify(
                stdout,
                "item/completed",
                json!({"threadId": "thread-fake-1", "turnId": "turn-1", "item": {"type": "agentMessage", "text": "The community garden layout is planned."}}),
            );
            respond(
                stdout,
                id,
                json!({
                    "threadId": "thread-fake-1",
                    "turn": {"id": "turn-1", "status": "completed"},
                }),
            );
            notify(
                stdout,
                "turn/completed",
                json!({"threadId": "thread-fake-1", "turn": {"id": "turn-1", "status": "completed"}}),
            );
        }
        "test/exit" => {
            respond(stdout, id, json!({}));
            let _ = stdout.flush();
            return false;
        }
        other => {
            respond_error(stdout, id, -32601, &format!("method not found: {other}"));
        }
    }
    true
}

fn thread_fixture() -> serde_json::Value {
    use serde_json::json;
    json!({
        "id": "thread-fake-1",
        "sessionId": "sess-fake-1",
        "preview": "Plan the community garden layout",
        "name": "Community garden layout",
        "cwd": "/tmp/garden",
        "createdAt": 1_760_000_000_i64,
        "updatedAt": 1_760_000_100_i64,
        "status": {},
        "gitInfo": null,
        "turns": [],
    })
}

fn respond(stdout: &mut std::io::Stdout, id: &serde_json::Value, result: serde_json::Value) {
    use serde_json::json;
    let _ = writeln!(stdout, "{}", json!({"id": id, "result": result}));
}

fn respond_error(stdout: &mut std::io::Stdout, id: &serde_json::Value, code: i64, message: &str) {
    use serde_json::json;
    let _ = writeln!(
        stdout,
        "{}",
        json!({"id": id, "error": {"code": code, "message": message}})
    );
}

fn notify(stdout: &mut std::io::Stdout, method: &str, params: serde_json::Value) {
    use serde_json::json;
    let _ = writeln!(stdout, "{}", json!({"method": method, "params": params}));
}
