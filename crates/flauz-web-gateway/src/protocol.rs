//! The gateway WebSocket protocol: the session handshake, gateway state
//! events, and the transparent app-server frame dialect.
//!
//! The browser speaks the app-server's own JSON-RPC dialect (frames shaped
//! `{"method", "id", "params"}` / `{"id", "result" | "error"}` — the
//! app-server wire envelope carries no `jsonrpc` field) plus a minimal
//! gateway control envelope (frames shaped `{"type": "..."}`). The
//! descriptor served at `GET /gateway-protocol.json` documents this
//! contract; `web/src/gateway/protocol.ts` is its TypeScript mirror.
//!
//! The gateway adds ZERO product logic (Wave-6 kernel addendum §1): it
//! never interprets, filters, or fabricates app-server methods.

use serde_json::{Value, json};

/// The gateway protocol version (bumped on any envelope change).
pub const GATEWAY_PROTOCOL_VERSION: u32 = 1;

/// Client → gateway: claim an authenticated session. Must be the first
/// message on the socket.
pub const MSG_SESSION_CLAIM: &str = "session.claim";

/// Gateway → client: the session handshake succeeded; a supervised
/// app-server is attached and bridging is active.
pub const MSG_SESSION_CLAIMED: &str = "session.claimed";

/// Gateway → client: the session handshake (or a frame) was refused; the
/// socket is closed right after.
pub const MSG_SESSION_DENIED: &str = "session.denied";

/// Gateway → client: a truthful supervised-app-server state transition.
pub const MSG_GATEWAY_STATE: &str = "gateway.state";

/// Gateway → client: graceful shutdown is in progress.
pub const MSG_GATEWAY_SHUTDOWN: &str = "gateway.shutdown";

/// Gateway → client: a transport-level protocol error (not a refusal).
pub const MSG_GATEWAY_ERROR: &str = "gateway.error";

/// The maximum length of a JSON-RPC method name accepted by the bridge.
pub const MAX_METHOD_BYTES: usize = 128;

/// The maximum number of malformed frames tolerated before the socket is
/// closed.
pub const MAX_MALFORMED_FRAMES: usize = 3;

/// Named refusal codes for [`MSG_SESSION_DENIED`].
pub mod denial_codes {
    /// A JSON-RPC frame arrived before the session handshake.
    pub const HANDSHAKE_REQUIRED: &str = "handshake_required";
    /// A second session claim arrived on a claimed socket.
    pub const HANDSHAKE_ALREADY_CLAIMED: &str = "handshake_already_claimed";
    /// The presented session token is missing or invalid.
    pub const INVALID_TOKEN: &str = "invalid_token";
    /// The gateway session limit is reached.
    pub const SESSION_LIMIT_REACHED: &str = "session_limit_reached";
    /// The frame exceeded the maximum accepted size.
    pub const MESSAGE_TOO_LARGE: &str = "message_too_large";
    /// The frame was not valid JSON.
    pub const MALFORMED_MESSAGE: &str = "malformed_message";
}

/// Supervised app-server states reported through [`MSG_GATEWAY_STATE`].
pub mod supervisor_states {
    /// A supervised app-server is spawned and initialized; bridging is
    /// live.
    pub const CONNECTED: &str = "connected";
    /// The supervised app-server died; the supervisor is restarting it
    /// with backoff (the desktop backend's reconnect semantics).
    pub const RECONNECTING: &str = "reconnecting";
}

/// Gateway error codes for [`MSG_GATEWAY_ERROR`] (transport-level).
pub mod gateway_error_codes {
    /// The bridged request is not valid JSON.
    pub const MALFORMED_MESSAGE: i64 = -32000;
    /// The method name is not acceptable (empty, oversized, or the
    /// bounded intern table is full).
    pub const INVALID_METHOD: i64 = -32001;
    /// Too many concurrent bridged requests are in flight.
    pub const TOO_MANY_INFLIGHT: i64 = -32002;
    /// The bridged request timed out.
    pub const REQUEST_TIMEOUT: i64 = -32003;
    /// The supervised app-server transport rejected the frame.
    pub const TRANSPORT: i64 = -32004;
    /// Live updates were dropped under backpressure (refresh to
    /// re-sync).
    pub const NOTIFICATIONS_DROPPED: i64 = -32005;
}

/// How an incoming browser frame is classified.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientFrame {
    /// A session claim (`session.claim`), optionally carrying a token.
    SessionClaim { token: Option<String> },
    /// A JSON-RPC request: `{"method", "id", "params"?}`.
    Request {
        method: String,
        id: Value,
        params: Value,
    },
    /// A JSON-RPC notification: `{"method", "params"?}` (no id).
    Notification { method: String, params: Value },
    /// A response to a server-initiated request:
    /// `{"id", "result"}` or `{"id", "error"}`.
    Response {
        id: Value,
        result: Option<Value>,
        error: Option<Value>,
    },
    /// Not a recognized frame in either dialect.
    Malformed,
}

/// Classifies a parsed browser frame.
#[must_use]
pub fn classify_client_frame(value: &Value) -> ClientFrame {
    let Some(object) = value.as_object() else {
        return ClientFrame::Malformed;
    };
    if object.get("type").and_then(Value::as_str) == Some(MSG_SESSION_CLAIM) {
        let token = object
            .get("token")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        return ClientFrame::SessionClaim { token };
    }
    let method = object.get("method").and_then(Value::as_str);
    match method {
        Some(method) => {
            if method.is_empty() || method.len() > MAX_METHOD_BYTES {
                return ClientFrame::Malformed;
            }
            let params = object.get("params").cloned().unwrap_or(Value::Null);
            match object.get("id") {
                Some(id) => ClientFrame::Request {
                    method: method.to_owned(),
                    id: id.clone(),
                    params,
                },
                None => ClientFrame::Notification {
                    method: method.to_owned(),
                    params,
                },
            }
        }
        None => {
            let id = object.get("id").cloned();
            let result = object.get("result").cloned();
            let error = object.get("error").cloned();
            match (id, result.is_some() || error.is_some()) {
                (Some(id), true) => ClientFrame::Response { id, result, error },
                _ => ClientFrame::Malformed,
            }
        }
    }
}

/// Builds a `session.claimed` message.
#[must_use]
pub fn session_claimed(session_id: &str) -> Value {
    json!({
        "type": MSG_SESSION_CLAIMED,
        "sessionId": session_id,
        "protocolVersion": GATEWAY_PROTOCOL_VERSION,
    })
}

/// Builds a `session.denied` message with a named code and message.
#[must_use]
pub fn session_denied(code: &str, message: &str) -> Value {
    json!({
        "type": MSG_SESSION_DENIED,
        "code": code,
        "message": message,
    })
}

/// Builds a `gateway.state` message with a truthful reason.
#[must_use]
pub fn gateway_state(state: &str, reason: &str) -> Value {
    json!({
        "type": MSG_GATEWAY_STATE,
        "state": state,
        "reason": reason,
    })
}

/// Builds a `gateway.shutdown` message with a named reason.
#[must_use]
pub fn gateway_shutdown(reason: &str) -> Value {
    json!({
        "type": MSG_GATEWAY_SHUTDOWN,
        "reason": reason,
    })
}

/// Builds a `gateway.error` message carrying a JSON-RPC error body.
#[must_use]
pub fn gateway_error(code: i64, message: &str) -> Value {
    json!({
        "type": MSG_GATEWAY_ERROR,
        "error": { "code": code, "message": message },
    })
}

/// Builds a JSON-RPC success response in the app-server dialect.
#[must_use]
pub fn rpc_success(id: &Value, result: Value) -> Value {
    json!({ "id": id, "result": result })
}

/// Builds a JSON-RPC error response in the app-server dialect.
#[must_use]
pub fn rpc_failure(id: &Value, code: i64, message: &str) -> Value {
    json!({ "id": id, "error": { "code": code, "message": message } })
}

/// Re-encodes a supervised app-server notification as a browser frame.
#[must_use]
pub fn server_notification(method: &str, params: &Value) -> Value {
    json!({ "method": method, "params": params })
}

/// Re-encodes a supervised app-server request as a browser frame.
#[must_use]
pub fn server_request(id: &Value, method: &str, params: &Value) -> Value {
    json!({ "id": id, "method": method, "params": params })
}

/// The gateway protocol descriptor served at `GET /gateway-protocol.json`:
/// the single-source documentation of the WebSocket contract (the web
/// client's `gateway/protocol.ts` mirrors this descriptor).
#[must_use]
pub fn gateway_protocol_descriptor() -> Value {
    json!({
        "v": GATEWAY_PROTOCOL_VERSION,
        "kind": "flauz.web-gateway.protocol",
        "transport": "websocket",
        "endpoint": "/ws",
        "clientToGateway": [
            {
                "type": MSG_SESSION_CLAIM,
                "description": "The authenticated-session handshake. Must be the first message; \
                                optional `token` string is required when the operator provisioned \
                                session tokens.",
                "fields": { "type": "session.claim", "token?": "string" },
            },
            {
                "dialect": "app-server-jsonrpc",
                "description": "Transparent app-server JSON-RPC dialect frames: requests \
                                ({method, id, params?}), notifications ({method, params?}), and \
                                responses to server-initiated requests ({id, result} or \
                                {id, error}). The gateway brokers requests (re-issuing them \
                                through the supervised connection) and never interprets them.",
            },
        ],
        "gatewayToClient": [
            {
                "type": MSG_SESSION_CLAIMED,
                "description": "The supervised app-server is attached and bridging is active.",
                "fields": { "type": "session.claimed", "sessionId": "string", "protocolVersion": 1 },
            },
            {
                "type": MSG_SESSION_DENIED,
                "description": "The handshake or frame was refused; the socket closes. `code` is \
                                one of handshake_required | handshake_already_claimed | \
                                invalid_token | session_limit_reached | message_too_large | \
                                malformed_message.",
                "fields": { "type": "session.denied", "code": "string", "message": "string" },
            },
            {
                "type": MSG_GATEWAY_STATE,
                "description": "Truthful supervised-app-server state: connected | reconnecting \
                                (with the named reason). The supervisor restarts a dead app-server \
                                with the desktop backend's backoff schedule.",
                "fields": { "type": "gateway.state", "state": "string", "reason": "string" },
            },
            {
                "type": MSG_GATEWAY_SHUTDOWN,
                "description": "Graceful gateway shutdown; the browser should show a named \
                                disconnection and retry.",
                "fields": { "type": "gateway.shutdown", "reason": "string" },
            },
            {
                "type": MSG_GATEWAY_ERROR,
                "description": "Transport-level protocol error (malformed frame, invalid method, \
                                too many in-flight requests, request timeout).",
                "fields": { "type": "gateway.error", "error": { "code": "number", "message": "string" } },
            },
        ],
        "bounds": {
            "maxFrameBytes": 16777216,
            "maxMalformedFramesBeforeClose": MAX_MALFORMED_FRAMES,
            "maxMethodBytes": MAX_METHOD_BYTES,
        },
        "credentialLaw": "Credentials never appear in gateway logs, URLs, or control messages; \
                          authentication flows through the app-server's own auth API over the \
                          transparent bridge.",
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ClientFrame, classify_client_frame, denial_codes, gateway_protocol_descriptor,
        session_claimed, session_denied,
    };

    #[test]
    fn frame_classification_covers_the_two_dialects() {
        assert_eq!(
            classify_client_frame(&json!({"type": "session.claim"})),
            ClientFrame::SessionClaim { token: None }
        );
        assert_eq!(
            classify_client_frame(&json!({"type": "session.claim", "token": "t0123456789abcdef"})),
            ClientFrame::SessionClaim {
                token: Some("t0123456789abcdef".to_owned())
            }
        );
        assert_eq!(
            classify_client_frame(&json!({"method": "thread/list", "id": 1, "params": {}})),
            ClientFrame::Request {
                method: "thread/list".to_owned(),
                id: json!(1),
                params: json!({}),
            }
        );
        assert_eq!(
            classify_client_frame(
                &json!({"method": "account/login/cancel", "params": {"loginId": "x"}})
            ),
            ClientFrame::Notification {
                method: "account/login/cancel".to_owned(),
                params: json!({"loginId": "x"}),
            }
        );
        assert_eq!(
            classify_client_frame(&json!({"id": "srv-1", "result": {"decision": "approved"}})),
            ClientFrame::Response {
                id: json!("srv-1"),
                result: Some(json!({"decision": "approved"})),
                error: None,
            }
        );
        assert_eq!(
            classify_client_frame(&json!({"id": 9, "error": {"code": -32000, "message": "no"}})),
            ClientFrame::Response {
                id: json!(9),
                result: None,
                error: Some(json!({"code": -32000, "message": "no"})),
            }
        );
        for malformed in [
            json!("plain string"),
            json!(42),
            json!({}),
            json!({"id": 3}),
            json!({"method": ""}),
            json!({"type": "unknown.control"}),
        ] {
            assert_eq!(classify_client_frame(&malformed), ClientFrame::Malformed);
        }
    }

    #[test]
    fn claimed_and_denied_shapes_are_stable() {
        let claimed = session_claimed("gwsess_0123456789abcdef");
        assert_eq!(claimed["type"], "session.claimed");
        assert_eq!(claimed["sessionId"], "gwsess_0123456789abcdef");
        assert_eq!(claimed["protocolVersion"], 1);
        let denied = session_denied(
            denial_codes::HANDSHAKE_REQUIRED,
            "claim a session before sending app-server frames",
        );
        assert_eq!(denied["type"], "session.denied");
        assert_eq!(denied["code"], "handshake_required");
    }

    #[test]
    fn protocol_descriptor_documents_the_contract() {
        let descriptor = gateway_protocol_descriptor();
        assert_eq!(descriptor["kind"], "flauz.web-gateway.protocol");
        let text = descriptor.to_string();
        for required in [
            "session.claim",
            "session.claimed",
            "session.denied",
            "gateway.state",
            "gateway.shutdown",
            "gateway.error",
        ] {
            assert!(
                text.contains(required),
                "descriptor must mention {required}"
            );
        }
        assert!(
            !text.to_lowercase().contains("token=\""),
            "the descriptor must not embed token material"
        );
    }
}
