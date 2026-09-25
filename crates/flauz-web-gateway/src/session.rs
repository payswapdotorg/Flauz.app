//! The authenticated session bridge: one WebSocket ↔ one supervised
//! app-server (WEB-001).
//!
//! Session law (Wave-6 kernel addendum §4): the browser MUST complete
//! the `session.claim` handshake before any app-server frame is
//! bridged; JSON-RPC traffic before the handshake is refused with the
//! named `handshake_required` denial. When the operator provisioned
//! session tokens, a missing or invalid token is refused with
//! `invalid_token`. Authentication itself flows through the app-server's
//! own auth API over the transparent bridge — the gateway never sees or
//! handles credentials.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crossbeam_channel::{Receiver as CrossbeamReceiver, Sender as CrossbeamSender, bounded};
use futures::{SinkExt, StreamExt, stream::SplitSink, stream::SplitStream};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::watch;

use crate::config::GatewayConfig;
use crate::logging::Logger;
use crate::protocol::{
    self, ClientFrame, MAX_MALFORMED_FRAMES, classify_client_frame, denial_codes, gateway_error,
    gateway_shutdown, gateway_state, session_claimed, session_denied,
};
use crate::static_files::StaticFiles;
use crate::supervisor::{
    self, SupervisorCommand, SupervisorConfig, SupervisorEvent, spawn_supervisor,
};

/// The time the browser has to complete the session handshake.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);

/// The bounded capacity of the bridge's outgoing frame queue.
pub const OUTGOING_QUEUE_CAPACITY: usize = 256;

/// Shared gateway state for all handlers.
#[derive(Clone)]
pub struct Shared {
    /// The validated gateway configuration.
    pub config: Arc<GatewayConfig>,
    /// The structured logger (never receives credentials).
    pub logger: Logger,
    /// The static file host.
    pub statics: Arc<StaticFiles>,
    /// The number of authenticated, live sessions.
    pub active_sessions: Arc<AtomicUsize>,
    /// The session-id minting counter.
    pub session_counter: Arc<AtomicU64>,
    /// The graceful-shutdown signal.
    pub shutdown: watch::Receiver<bool>,
    /// The instant the gateway started (health payload).
    pub started_at: std::time::Instant,
}

/// Outgoing frames produced by the session machinery.
enum OutFrame {
    /// A text frame to send.
    Text(String),
    /// Close the socket after this frame.
    Close,
}

/// The WebSocket endpoint handler.
pub async fn websocket_handler(
    State(shared): State<Shared>,
    upgrade: WebSocketUpgrade,
    request: Request,
) -> Response {
    if let Some(refusal) = refuse_cross_origin_ws(&shared, &request) {
        return refusal;
    }
    let max_frame = shared.config.max_frame_bytes;
    upgrade
        .max_message_size(max_frame)
        .max_frame_size(max_frame)
        .on_upgrade(move |socket| async move {
            run_session(shared, socket).await;
        })
}

/// A drive-by WebSocket connection from a website (non-gateway origin)
/// is refused while the gateway binds loopback: the browser same-origin
/// guarantee is enforced, and non-browser clients (no `Origin` header)
/// are not affected.
fn refuse_cross_origin_ws(shared: &Shared, request: &Request) -> Option<Response> {
    if !shared.config.binds_loopback() {
        return None;
    }
    let origin = request.headers().get(axum::http::header::ORIGIN)?;
    let origin = origin.to_str().ok()?;
    let host = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))?
        .split(':')
        .next()
        .unwrap_or_default();
    let local = ["localhost", "127.0.0.1", "[::1]", "::1"];
    if local.contains(&host) {
        None
    } else {
        shared.logger.warn(
            "websocket_origin_refused",
            &[("origin_host", host), ("origin_kind", "cross_site")],
        );
        Some(
            (
                StatusCode::FORBIDDEN,
                "cross-origin WebSocket connections are refused by the localhost gateway",
            )
                .into_response(),
        )
    }
}

/// Runs one authenticated session: handshake → supervisor → bridge.
async fn run_session(shared: Shared, socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let claimed = perform_handshake(&shared, &mut receiver, &mut sender).await;
    let (session_id, guard) = match claimed {
        Ok(claimed) => claimed,
        Err(()) => return,
    };

    // Wire the supervisor.
    let (command_tx, command_rx) = bounded::<SupervisorCommand>(64);
    let (event_tx, event_rx) = bounded::<SupervisorEvent>(512);
    let supervisor_config = SupervisorConfig {
        codex_binary: shared.config.codex_binary.clone(),
        codex_home: shared.config.codex_home.clone(),
        request_timeout: shared.config.request_timeout,
        max_inflight_requests: shared.config.max_inflight_requests,
    };
    let supervisor_logger = shared.logger.clone();
    let supervisor_session_id = session_id.clone();
    let supervisor_handle = spawn_supervisor(
        supervisor_config,
        command_rx,
        event_tx,
        supervisor_logger,
        supervisor_session_id,
    );

    // The outgoing pump: the sync supervisor thread → a bounded async
    // queue → the WebSocket sink.
    let (out_tx, mut out_rx) = mpsc::channel::<OutFrame>(OUTGOING_QUEUE_CAPACITY);
    let pump = tokio::task::spawn_blocking({
        let out_tx = out_tx.clone();
        move || pump_supervisor_events(event_rx, out_tx)
    });

    shared.logger.info(
        "session_claimed",
        &[("session_id", session_id.as_str()), ("origin", "local")],
    );

    let mut malformed = 0_usize;
    let mut shutdown_rx = shared.shutdown.clone();
    let mut close_reason = "the browser closed the session".to_owned();
    loop {
        tokio::select! {
            outgoing = out_rx.recv() => match outgoing {
                Some(OutFrame::Text(frame)) => {
                    if sender
                        .send(Message::Text(Utf8Bytes::from(frame)))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Some(OutFrame::Close) => {
                    close_reason = "the supervised app-server session ended".to_owned();
                    break;
                }
                None => break,
            },
            incoming = receiver.next() => match incoming {
                Some(Ok(message)) => {
                    match handle_incoming_message(
                        &command_tx,
                        &out_tx,
                        &mut malformed,
                        message,
                    )
                    .await
                    {
                        IncomingOutcome::Continue => {}
                        IncomingOutcome::Denied => {
                            close_reason = "the session was refused by the gateway".to_owned();
                            break;
                        }
                    }
                }
                Some(Err(_)) => {
                    close_reason = "the WebSocket transport failed".to_owned();
                    break;
                }
                None => break,
            },
            _ = shutdown_rx.changed() => {
                close_reason = "the gateway is shutting down".to_owned();
                let _ = out_tx
                    .send(OutFrame::Text(
                        gateway_shutdown("the gateway is shutting down; reconnect shortly")
                            .to_string(),
                    ))
                    .await;
                break;
            }
        }
    }

    // Teardown: name the disconnect, reap the supervised app-server,
    // release the session slot.
    let _ = sender
        .send(Message::Close(Some(CloseFrame {
            code: axum::extract::ws::close_code::NORMAL,
            reason: Utf8Bytes::from(close_reason),
        })))
        .await;
    let _ = command_tx.send(SupervisorCommand::Shutdown);
    let _ = supervisor_handle.join().is_ok();
    let _ = pump.await;
    drop(guard);
    shared
        .logger
        .info("session_closed", &[("session_id", session_id.as_str())]);
}

/// The incoming-message disposition.
enum IncomingOutcome {
    /// Keep bridging.
    Continue,
    /// The gateway refused the frame and closed the session.
    Denied,
}

async fn handle_incoming_message(
    command_tx: &CrossbeamSender<SupervisorCommand>,
    out_tx: &mpsc::Sender<OutFrame>,
    malformed: &mut usize,
    message: Message,
) -> IncomingOutcome {
    let text = match message {
        Message::Text(text) => text,
        Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => {
            // Binary frames and control traffic never carry app-server
            // frames; pings are answered by the transport.
            return IncomingOutcome::Continue;
        }
        Message::Close(_) => return IncomingOutcome::Denied,
    };
    let Ok(value) = serde_json::from_str::<Value>(text.as_str()) else {
        *malformed += 1;
        if *malformed > MAX_MALFORMED_FRAMES {
            return IncomingOutcome::Denied;
        }
        let _ = out_tx
            .send(OutFrame::Text(
                gateway_error(
                    protocol::gateway_error_codes::MALFORMED_MESSAGE,
                    "the frame was not valid JSON",
                )
                .to_string(),
            ))
            .await;
        return IncomingOutcome::Continue;
    };
    let command = match classify_client_frame(&value) {
        ClientFrame::SessionClaim { .. } => {
            // A second claim on a claimed socket is refused.
            let _ = out_tx
                .send(OutFrame::Text(
                    session_denied(
                        denial_codes::HANDSHAKE_ALREADY_CLAIMED,
                        "this session is already claimed; open a new connection for another \
                         session",
                    )
                    .to_string(),
                ))
                .await;
            return IncomingOutcome::Denied;
        }
        ClientFrame::Request { method, id, params } => {
            SupervisorCommand::BridgeRequest { method, id, params }
        }
        ClientFrame::Notification { method, params } => {
            SupervisorCommand::BridgeNotification { method, params }
        }
        ClientFrame::Response { id, result, error } => {
            SupervisorCommand::BridgeResponse { id, result, error }
        }
        ClientFrame::Malformed => {
            *malformed += 1;
            let refusal = *malformed > MAX_MALFORMED_FRAMES;
            let _ = out_tx
                .send(OutFrame::Text(
                    gateway_error(
                        protocol::gateway_error_codes::MALFORMED_MESSAGE,
                        "the frame is neither a gateway control message nor an app-server \
                         JSON-RPC frame",
                    )
                    .to_string(),
                ))
                .await;
            return if refusal {
                IncomingOutcome::Denied
            } else {
                IncomingOutcome::Continue
            };
        }
    };
    // Bounded command queue with honest refusal on sustained backpressure.
    let refused = command_tx
        .send_timeout(command, Duration::from_millis(100))
        .is_err();
    if refused {
        let _ = out_tx
            .send(OutFrame::Text(
                gateway_error(
                    protocol::gateway_error_codes::TOO_MANY_INFLIGHT,
                    "the gateway is saturated; retry shortly",
                )
                .to_string(),
            ))
            .await;
    }
    IncomingOutcome::Continue
}

/// Performs the session handshake: exactly one `session.claim` first,
/// within [`HANDSHAKE_TIMEOUT`]. Any app-server frame before the
/// handshake is refused with the named `handshake_required` denial
/// (the auth handshake refusal law).
async fn perform_handshake(
    shared: &Shared,
    receiver: &mut SplitStream<WebSocket>,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<(String, SessionGuard), ()> {
    let message = tokio::time::timeout(HANDSHAKE_TIMEOUT, receiver.next()).await;
    let message = match message {
        Ok(Some(Ok(message))) => message,
        _ => return Err(()),
    };
    let text = match message {
        Message::Text(text) => text,
        _ => {
            deny_and_close(
                sender,
                denial_codes::MALFORMED_MESSAGE,
                "the session handshake must be a session.claim text frame",
            )
            .await;
            return Err(());
        }
    };
    let value: Value = match serde_json::from_str(text.as_str()) {
        Ok(value) => value,
        Err(_) => {
            deny_and_close(
                sender,
                denial_codes::MALFORMED_MESSAGE,
                "the session handshake must be a session.claim JSON frame",
            )
            .await;
            return Err(());
        }
    };
    let token = match classify_client_frame(&value) {
        ClientFrame::SessionClaim { token } => token,
        ClientFrame::Request { .. } | ClientFrame::Notification { .. } => {
            // THE auth handshake refusal: app-server traffic before the
            // handshake is refused by name.
            deny_and_close(
                sender,
                denial_codes::HANDSHAKE_REQUIRED,
                "claim a session before sending app-server frames",
            )
            .await;
            return Err(());
        }
        _ => {
            deny_and_close(
                sender,
                denial_codes::MALFORMED_MESSAGE,
                "the session handshake must be a session.claim frame",
            )
            .await;
            return Err(());
        }
    };
    if let Some(tokens) = shared.config.session_tokens.as_ref()
        && !tokens.validate(token.as_deref().unwrap_or_default())
    {
        deny_and_close(
            sender,
            denial_codes::INVALID_TOKEN,
            "a valid provisioned session token is required on this bind",
        )
        .await;
        return Err(());
    }
    // Session-limit admission (bounded concurrency).
    let previous = shared.active_sessions.fetch_add(1, Ordering::AcqRel);
    if previous >= shared.config.max_sessions {
        shared.active_sessions.fetch_sub(1, Ordering::AcqRel);
        deny_and_close(
            sender,
            denial_codes::SESSION_LIMIT_REACHED,
            "the gateway session limit is reached; close another session first",
        )
        .await;
        return Err(());
    }
    let guard = SessionGuard {
        active_sessions: Arc::clone(&shared.active_sessions),
    };
    let session_id = mint_session_id(&shared.session_counter);
    let claimed = session_claimed(&session_id);
    if sender
        .send(Message::Text(Utf8Bytes::from(claimed.to_string())))
        .await
        .is_err()
    {
        return Err(());
    }
    Ok((session_id, guard))
}

async fn deny_and_close(sender: &mut SplitSink<WebSocket, Message>, code: &str, message: &str) {
    let denied = session_denied(code, message);
    let _ = sender
        .send(Message::Text(Utf8Bytes::from(denied.to_string())))
        .await;
    let _ = sender
        .send(Message::Close(Some(CloseFrame {
            code: axum::extract::ws::close_code::POLICY,
            reason: Utf8Bytes::from(format!("gateway: {code}")),
        })))
        .await;
}

/// Pumps supervisor events (sync crossbeam) into the bounded async
/// outgoing queue with honest backpressure (`blocking_send`).
fn pump_supervisor_events(
    events: CrossbeamReceiver<SupervisorEvent>,
    out_tx: mpsc::Sender<OutFrame>,
) {
    while let Ok(event) = events.recv() {
        let frame = match event {
            SupervisorEvent::State { state, reason } => gateway_state(state, &reason),
            SupervisorEvent::Frame(frame) => frame,
            SupervisorEvent::Response { id, result } => supervisor::brokered_response(&id, &result),
            SupervisorEvent::NotificationsDropped { count } => gateway_error(
                protocol::gateway_error_codes::NOTIFICATIONS_DROPPED,
                &format!(
                    "{count} live update(s) were dropped under backpressure; refresh to \
                     re-sync state"
                ),
            ),
            SupervisorEvent::Closed => {
                let _ = out_tx.blocking_send(OutFrame::Close);
                return;
            }
        };
        if out_tx
            .blocking_send(OutFrame::Text(frame.to_string()))
            .is_err()
        {
            return;
        }
    }
}

/// Decrements the active-session count when released.
struct SessionGuard {
    active_sessions: Arc<AtomicUsize>,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        self.active_sessions.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Mints a session label (`gwsess_<hex>`). Session ids are LABELS, not
/// secrets: they never authorize anything and never derive from
/// credentials.
fn mint_session_id(counter: &AtomicU64) -> String {
    use std::hash::{BuildHasher, Hasher};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let count = counter.fetch_add(1, Ordering::Relaxed);
    let pid = u64::from(std::process::id());
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u128(nanos ^ (u128::from(count) << 96) ^ (u128::from(pid) << 64));
    hasher.write_u64(count.rotate_left(32) ^ pid);
    let part_a = hasher.finish();
    hasher.write_u64(part_a.rotate_left(17) ^ count);
    let part_b = hasher.finish();
    format!("gwsess_{part_a:016x}{part_b:016x}")
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::mint_session_id;

    #[test]
    fn session_ids_are_labeled_prefixed_and_unique() {
        let counter = AtomicU64::new(0);
        let first = mint_session_id(&counter);
        let second = mint_session_id(&counter);
        assert!(first.starts_with("gwsess_"), "prefix missing: {first}");
        assert_eq!(first.len(), "gwsess_".len() + 32);
        assert_ne!(first, second);
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }
}
