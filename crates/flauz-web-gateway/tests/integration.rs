//! WEB-001 gateway integration tests: a real gateway server, a real
//! WebSocket client, and a real supervised (fake) app-server process —
//! the full bridge round-trip, the auth handshake refusal law, static
//! hosting, the bind/auth law, and the supervision restart path.

use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, tungstenite};

use flauz_web_gateway::config::{GatewayConfig, SessionTokens};
use flauz_web_gateway::logging::Logger;
use flauz_web_gateway::server::serve;

struct Fixture {
    _home: tempfile::TempDir,
    _web_root: tempfile::TempDir,
    config: GatewayConfig,
}

fn fixture_config() -> Fixture {
    let home = tempfile::TempDir::new().unwrap_or_else(|error| panic!("temp home: {error}"));
    let web_root =
        tempfile::TempDir::new().unwrap_or_else(|error| panic!("temp web root: {error}"));
    std::fs::write(
        web_root.path().join("index.html"),
        "<html>flauz web shell fixture</html>",
    )
    .unwrap_or_else(|error| panic!("index fixture: {error}"));
    // Hermetic: construct directly (every field is overridden anyway) so
    // the fixture never depends on the runner's $HOME having ~/.codex.
    let codex_home = codex_platform::CodexHome::resolve(Some(home.path().to_path_buf()))
        .unwrap_or_else(|error| panic!("codex home resolve: {error}"));
    let config = GatewayConfig {
        bind: "127.0.0.1:0"
            .parse()
            .unwrap_or_else(|_| panic!("bind parse")),
        web_root: web_root.path().to_path_buf(),
        codex_binary: PathBuf::from(env!("CARGO_BIN_EXE_flauz-fake-app-server")),
        codex_home,
        session_tokens: None,
        max_sessions: 8,
        max_inflight_requests: 64,
        request_timeout: Duration::from_secs(5),
        max_frame_bytes: codex_protocol::DEFAULT_MAX_FRAME_BYTES,
    };
    Fixture {
        _home: home,
        _web_root: web_root,
        config,
    }
}

async fn ws_connect(
    port: u16,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let url = format!("ws://127.0.0.1:{port}/ws");
    for _ in 0..40 {
        if let Ok((stream, _)) = connect_async(url.as_str()).await {
            return stream;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("could not connect the test WebSocket client to {url}");
}

async fn next_json<S>(stream: &mut S) -> Value
where
    S: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            panic!("timed out waiting for a bridge frame");
        }
        let next = tokio::time::timeout(remaining, stream.next()).await;
        match next {
            Ok(Some(Ok(Message::Text(text)))) => {
                return serde_json::from_str(text.as_str())
                    .unwrap_or_else(|error| panic!("non-JSON frame ({error})"));
            }
            Ok(Some(Ok(_))) => continue,
            Ok(Some(Err(error))) => panic!("bridge transport error: {error}"),
            Ok(None) => panic!("the bridge closed before a frame arrived"),
            Err(_) => panic!("timed out waiting for a bridge frame"),
        }
    }
}

async fn expect_message_kind<S>(stream: &mut S, kind: &str) -> Value
where
    S: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            panic!("timed out waiting for a {kind} frame");
        }
        match tokio::time::timeout(remaining, stream.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let value: Value = serde_json::from_str(text.as_str())
                    .unwrap_or_else(|error| panic!("non-JSON frame ({error})"));
                if value["type"].as_str() == Some(kind) {
                    return value;
                }
            }
            Ok(Some(Ok(_))) => {}
            Ok(Some(Err(error))) => panic!("bridge transport error: {error}"),
            Ok(None) => panic!("the bridge closed before a {kind} frame arrived"),
            Err(_) => panic!("timed out waiting for a {kind} frame"),
        }
    }
}

#[tokio::test]
async fn jsonrpc_before_handshake_is_refused_by_name() {
    let fixture = fixture_config();
    let handle = serve(fixture.config, Logger::stdout())
        .await
        .unwrap_or_else(|error| panic!("serve: {error}"));
    let port = handle.local_addr.port();

    let mut client = ws_connect(port).await;
    client
        .send(Message::Text(
            json!({"method": "thread/list", "id": 1, "params": {"limit": 20}})
                .to_string()
                .into(),
        ))
        .await
        .unwrap_or_else(|error| panic!("send: {error}"));
    let denied = expect_message_kind(&mut client, "session.denied").await;
    assert_eq!(denied["code"], "handshake_required");
    assert!(
        denied["message"]
            .as_str()
            .unwrap_or_default()
            .contains("claim a session"),
        "the refusal must name the recovery action: {denied}"
    );
    handle
        .shutdown()
        .await
        .unwrap_or_else(|error| panic!("shutdown: {error}"));
}

#[tokio::test]
async fn claim_then_round_trip_then_supervision_restart() {
    let fixture = fixture_config();
    let handle = serve(fixture.config, Logger::stdout())
        .await
        .unwrap_or_else(|error| panic!("serve: {error}"));
    let port = handle.local_addr.port();

    let mut client = ws_connect(port).await;
    // 1. The authenticated-session handshake.
    client
        .send(Message::Text(
            json!({"type": "session.claim"}).to_string().into(),
        ))
        .await
        .unwrap_or_else(|error| panic!("claim send: {error}"));
    let claimed = expect_message_kind(&mut client, "session.claimed").await;
    let session_id = claimed["sessionId"].as_str().unwrap_or_default().to_owned();
    assert!(
        session_id.starts_with("gwsess_"),
        "bad session id: {session_id}"
    );

    // 2. The supervisor attaches and reports connected.
    let state = expect_message_kind(&mut client, "gateway.state").await;
    assert_eq!(state["state"], "connected");

    // 3. A JSON-RPC request round-trips through the REAL bridge into the
    //    REAL supervised app-server process.
    client
        .send(Message::Text(
            json!({
                "method": "thread/list",
                "id": 41,
                "params": {"limit": 20, "sortKey": "recency_at", "sortDirection": "desc", "useStateDbOnly": true}
            })
            .to_string().into(),
        ))
        .await
        .unwrap_or_else(|error| panic!("request send: {error}"));
    let response = wait_for_id(&mut client, 41).await;
    let data = response["result"]["data"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        data.len(),
        2,
        "the fake app-server must return its fixtures"
    );
    assert_eq!(data[0]["id"], "thread-fake-1");

    // 4. A turn streams notifications through the transparent bridge.
    client
        .send(Message::Text(
            json!({
                "method": "turn/start",
                "id": 42,
                "params": {"threadId": "thread-fake-1", "input": [{"type": "text", "text": "plan the layout"}]}
            })
            .to_string().into(),
        ))
        .await
        .unwrap_or_else(|error| panic!("turn send: {error}"));
    let mut saw_started = false;
    let mut saw_completed = false;
    let mut turn_response: Option<Value> = None;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while tokio::time::Instant::now() < deadline
        && !(saw_started && saw_completed && turn_response.is_some())
    {
        let frame = next_json(&mut client).await;
        match frame["method"].as_str() {
            Some("turn/started") => saw_started = true,
            Some("turn/completed") => saw_completed = true,
            _ => {}
        }
        if frame["id"].as_i64() == Some(42) && frame.get("result").is_some() {
            turn_response = Some(frame);
        }
    }
    assert!(
        saw_started && saw_completed,
        "turn notifications must stream"
    );
    let turn_response =
        turn_response.unwrap_or_else(|| panic!("the turn/start response never arrived"));
    assert_eq!(turn_response["result"]["turn"]["status"], "completed");

    // 5. Kill the supervised app-server (the test-only hook): the
    //    gateway must report a named reconnecting state, restart the
    //    app-server with the desktop backoff schedule, and return to
    //    connected — then serve requests again.
    client
        .send(Message::Text(
            json!({"method": "test/exit", "id": 43}).to_string().into(),
        ))
        .await
        .unwrap_or_else(|error| panic!("exit send: {error}"));
    let mut got_exit_ack = false;
    let mut saw_reconnecting = false;
    let mut recovered = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while tokio::time::Instant::now() < deadline && !(got_exit_ack && recovered) {
        let frame = next_json(&mut client).await;
        if frame["id"].as_i64() == Some(43)
            && (frame.get("result").is_some() || frame.get("error").is_some())
        {
            got_exit_ack = true;
        } else if frame["type"].as_str() == Some("gateway.state") {
            match frame["state"].as_str() {
                Some("reconnecting") => {
                    saw_reconnecting = true;
                    let reason = frame["reason"].as_str().unwrap_or_default();
                    assert!(
                        !reason.is_empty(),
                        "the reconnecting state must name its reason"
                    );
                }
                Some("connected") => recovered = true,
                _ => {}
            }
        }
    }
    assert!(got_exit_ack, "the exit hook must be acknowledged");
    assert!(saw_reconnecting, "the gateway must name the disconnection");
    assert!(recovered, "the supervisor must restart the app-server");
    client
        .send(Message::Text(
            json!({
                "method": "thread/list",
                "id": 44,
                "params": {"limit": 20, "sortKey": "recency_at", "sortDirection": "desc", "useStateDbOnly": true}
            })
            .to_string().into(),
        ))
        .await
        .unwrap_or_else(|error| panic!("post-restart request send: {error}"));
    let response = wait_for_id(&mut client, 44).await;
    assert_eq!(
        response["result"]["data"].as_array().map(Vec::len),
        Some(2),
        "the restarted app-server must serve requests again"
    );

    handle
        .shutdown()
        .await
        .unwrap_or_else(|error| panic!("shutdown: {error}"));
}

async fn wait_for_id<S>(stream: &mut S, id: i64) -> Value
where
    S: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            panic!("timed out waiting for the response to id {id}");
        }
        match tokio::time::timeout(remaining, stream.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let value: Value = serde_json::from_str(text.as_str())
                    .unwrap_or_else(|error| panic!("non-JSON frame ({error})"));
                if value["id"].as_i64() == Some(id)
                    && (value.get("result").is_some() || value.get("error").is_some())
                {
                    return value;
                }
            }
            Ok(Some(Ok(_))) => {}
            Ok(Some(Err(error))) => panic!("bridge transport error: {error}"),
            Ok(None) => panic!("the bridge closed before the response to id {id}"),
            Err(_) => panic!("timed out waiting for the response to id {id}"),
        }
    }
}

#[tokio::test]
async fn static_hosting_serves_the_shell_and_descriptor() {
    let fixture = fixture_config();
    let handle = serve(fixture.config, Logger::stdout())
        .await
        .unwrap_or_else(|error| panic!("serve: {error}"));
    let port = handle.local_addr.port();

    let index = http_get(port, "/").await;
    assert!(index.contains("200 OK"), "index status: {index}");
    assert!(index.contains("flauz web shell fixture"));

    let missing = http_get(port, "/assets/missing-asset.js").await;
    assert!(
        missing.contains("404 Not Found"),
        "missing asset: {missing}"
    );

    let descriptor = http_get(port, "/gateway-protocol.json").await;
    assert!(descriptor.contains("flauz.web-gateway.protocol"));
    assert!(descriptor.contains("session.claim"));

    let health = http_get(port, "/healthz").await;
    assert!(health.contains("\"status\":\"ok\""), "health: {health}");

    let traversal = http_get(port, "/../Cargo.toml").await;
    assert!(
        !traversal.contains("[package]"),
        "path traversal must not be served: {traversal}"
    );

    handle
        .shutdown()
        .await
        .unwrap_or_else(|error| panic!("shutdown: {error}"));
}

async fn http_get(port: u16, path: &str) -> String {
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap_or_else(|error| panic!("tcp connect: {error}"));
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let request = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .unwrap_or_else(|error| panic!("http write: {error}"));
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response).await;
    String::from_utf8_lossy(&response).into_owned()
}

#[tokio::test]
async fn provisioned_tokens_gate_the_handshake() {
    let fixture = fixture_config();
    let mut config = fixture.config;
    config.session_tokens = Some(
        SessionTokens::from_text(Path::new("tokens"), "test-token-0123456789abcdef")
            .unwrap_or_else(|error| panic!("token fixture: {error}")),
    );
    let handle = serve(config, Logger::stdout())
        .await
        .unwrap_or_else(|error| panic!("serve: {error}"));
    let port = handle.local_addr.port();

    // A claim WITHOUT a token is refused by name.
    let mut client = ws_connect(port).await;
    client
        .send(Message::Text(
            json!({"type": "session.claim"}).to_string().into(),
        ))
        .await
        .unwrap_or_else(|error| panic!("claim send: {error}"));
    let denied = expect_message_kind(&mut client, "session.denied").await;
    assert_eq!(denied["code"], "invalid_token");

    // A claim WITH the valid token is accepted.
    let mut client = ws_connect(port).await;
    client
        .send(Message::Text(
            json!({"type": "session.claim", "token": "test-token-0123456789abcdef"})
                .to_string()
                .into(),
        ))
        .await
        .unwrap_or_else(|error| panic!("claim send: {error}"));
    let _claimed = expect_message_kind(&mut client, "session.claimed").await;

    handle
        .shutdown()
        .await
        .unwrap_or_else(|error| panic!("shutdown: {error}"));
}

#[tokio::test]
async fn non_local_bind_refuses_to_start_without_tokens() {
    let fixture = fixture_config();
    let mut config = fixture.config;
    let Ok(bind) = "0.0.0.0:0".parse() else {
        panic!("bind parse");
    };
    config.bind = bind;
    config.session_tokens = None;
    let refused = serve(config, Logger::stdout()).await;
    let error = match refused {
        Err(error) => error,
        Ok(handle) => {
            let _ = handle.shutdown().await;
            panic!("a non-local bind without tokens must refuse to start");
        }
    };
    assert!(
        error.to_string().contains("refusing to bind non-local"),
        "the refusal must be named: {error}"
    );
}
