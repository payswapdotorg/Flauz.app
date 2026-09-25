//! The per-session supervised app-server (WEB-001).
//!
//! The supervisor reuses `codex-platform`'s [`AppServerConnection`] — the
//! same spawn/restart/reap semantics as the desktop backend binding:
//!
//! - spawn `codex app-server` (piped stdio, `CODEX_HOME`/
//!   `CODEX_SQLITE_HOME` environment), reap via the managed child's
//!   graceful shutdown on drop or explicit shutdown;
//! - `initialize` with `experimentalApi: true` and the desktop's exact
//!   capability payload (including the stable-bridge opt-out list)
//!   immediately after spawn and after every restart;
//! - on death, restart with the desktop backend's backoff schedule
//!   (1 s initial delay, doubling, capped at 20 s, reset on a successful
//!   reconnect — mirrors `AppServerReconnectScheduler` in
//!   `codex-app/src/backend.rs`);
//! - truthful `connected` / `reconnecting` state events with named
//!   reasons at every transition.
//!
//! Requests from the browser are BROKERED through the connection (the
//! connection's router only delivers responses to requests it issued
//! itself, so the bridge re-issues each browser request with its own id
//! and re-encodes the response with the browser's original id).
//! Notifications pass through verbatim; server-initiated requests are
//! re-encoded and forwarded (their responses are routed back through
//! `respond_success`/`respond_error`).

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use serde_json::{Value, json};

use codex_platform::{
    AppServerConfig, AppServerConnection, AppServerError, AppServerEvent, CodexHome,
    ReceivedAppServerEvent,
};
use codex_protocol::{ClientInfo, InitializeCapabilities};

use crate::logging::Logger;
use crate::protocol::{
    self, gateway_error_codes, rpc_success, server_notification, server_request, supervisor_states,
};

/// The reconnect backoff schedule, mirroring the desktop backend's
/// `AppServerReconnectScheduler` (same constants, same doubling, same
/// reset-on-connect).
pub const RECONNECT_INITIAL_DELAY: Duration = Duration::from_secs(1);

/// The maximum reconnect backoff delay (desktop parity).
pub const RECONNECT_MAX_DELAY: Duration = Duration::from_secs(20);

/// The supervisor loop tick (desktop router parity).
const TICK: Duration = Duration::from_millis(25);

/// The event-channel backpressure budget before notifications are
/// dropped (and named) — desktop parity.
const EVENT_BACKPRESSURE_TIMEOUT: Duration = Duration::from_millis(100);

/// How long a server-initiated request may wait for browser delivery
/// before the supervisor reports a named delivery failure (the request
/// is then rejected back to the app-server honestly — never silently
/// wedged).
const SERVER_REQUEST_DELIVERY_TIMEOUT: Duration = Duration::from_secs(2);

/// The maximum bytes of a reconnect reason string.
const MAX_REASON_BYTES: usize = 300;

/// The maximum number of interned method names (the bounded bridge
/// method table).
const MAX_INTERNED_METHODS: usize = 512;

/// The stable-bridge notification opt-out list, copied verbatim from the
/// desktop backend's `STABLE_OPT_OUT_NOTIFICATION_METHODS`
/// (`codex-app/src/backend.rs`): the web client consumes the
/// experimental surface (`turn/*`, `item/*`, `thread/status/changed`,
/// `account/login/completed`, …), exactly like the desktop UI.
const STABLE_OPT_OUT_NOTIFICATION_METHODS: &[&str] = &[
    "thread/environment/connected",
    "thread/environment/disconnected",
    "rawResponseItem/completed",
    "command/exec/outputDelta",
    "externalAgentConfig/import/progress",
    "thread/compacted",
    "windows/worldWritableWarning",
    "turn/moderationMetadata",
    "authStatusChange",
    "loginChatGptComplete",
    "codex/event/task_started",
    "codex/event/agent_reasoning",
    "codex/event/agent_message",
    "codex/event/task_complete",
    "codex/event/mcp_tool_call_begin",
    "codex/event/mcp_tool_call_end",
    "codex/event/exec_command_begin",
    "codex/event/exec_command_end",
    "codex/event/exec_command_output_delta",
    "codex/event/exec_approval_request",
    "codex/event/apply_patch_approval_request",
    "codex/event/background_event",
    "codex/event/turn_diff",
    "codex/event/get_history_entry_response",
    "codex/event/agent_reasoning_delta",
    "codex/event/agent_reasoning_section_break",
    "codex/event/agent_message_delta",
    "codex/event/stream_error",
    "codex/event/error",
    "codex/event/turn_aborted",
    "codex/event/plan_delta",
    "codex/event/plan_update",
    "codex/event/patch_apply_begin",
    "codex/event/patch_apply_end",
    "codex/event/item_started",
    "codex/event/item_completed",
    "codex/event/user_message",
    "codex/event/agent_reasoning_raw_content",
    "codex/event/agent_reasoning_raw_content_delta",
    "codex/event/web_search_begin",
    "codex/event/web_search_end",
    "codex/event/mcp_list_tools_response",
    "codex/event/list_skills_response",
    "codex/event/list_remote_skills_response",
    "codex/event/remote_skill_downloaded",
    "codex/event/list_custom_prompts_response",
    "codex/event/raw_response_item",
    "codex/event/agent_message_content_delta",
    "codex/event/reasoning_content_delta",
    "codex/event/reasoning_raw_content_delta",
    "codex/event/warning",
    "codex/event/undo_started",
    "codex/event/undo_completed",
    "codex/event/shutdown_complete",
    "codex/event/entered_review_mode",
    "codex/event/exited_review_mode",
    "codex/event/view_image_tool_call",
    "codex/event/mcp_startup_update",
    "codex/event/mcp_startup_complete",
    "codex/event/remote_task_created",
    "codex/event/thread_rolled_back",
    "codex/event/thread_name_updated",
    "codex/event/elicitation_request",
    "codex/event/dynamic_tool_call_request",
    "codex/event/request_user_input",
    "codex/event/terminal_interaction",
    "codex/event/token_count",
    "codex/event/deprecation_notice",
    "thread/closed",
    "rawResponse/completed",
    "warning",
];

/// Commands the bridge sends into the supervisor.
#[derive(Debug)]
pub enum SupervisorCommand {
    /// Broker a browser JSON-RPC request through the supervised
    /// connection.
    BridgeRequest {
        method: String,
        id: Value,
        params: Value,
    },
    /// Forward a browser notification verbatim.
    BridgeNotification { method: String, params: Value },
    /// Route a browser response to a server-initiated request.
    BridgeResponse {
        id: Value,
        result: Option<Value>,
        error: Option<Value>,
    },
    /// Shut down: reap the app-server and end the loop.
    Shutdown,
}

/// Events the supervisor reports to the bridge.
#[derive(Debug)]
pub enum SupervisorEvent {
    /// A truthful supervised-app-server state transition (connected /
    /// reconnecting) with a named reason.
    State { state: &'static str, reason: String },
    /// A frame for the browser: a re-encoded app-server notification or
    /// server-initiated request.
    Frame(Value),
    /// A response to a brokered browser request (result or a named
    /// `(code, message)` failure).
    Response {
        id: Value,
        result: Result<Value, (i64, String)>,
    },
    /// Notifications were dropped under backpressure (named, honest).
    NotificationsDropped { count: usize },
    /// The supervisor loop ended.
    Closed,
}

/// The supervisor configuration (a view of the gateway config).
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    /// The official codex binary.
    pub codex_binary: std::path::PathBuf,
    /// The resolved CODEX_HOME.
    pub codex_home: CodexHome,
    /// The request timeout for brokered requests.
    pub request_timeout: Duration,
    /// The maximum in-flight brokered requests.
    pub max_inflight_requests: usize,
}

/// The outcome of handling one command.
#[derive(Debug)]
enum CommandOutcome {
    /// Keep the connection running.
    Continue,
    /// The app-server transport rejected the frame; restart it.
    Died(String),
    /// The supervisor was asked to shut down.
    Shutdown,
}

/// The web client identity used for the app-server `initialize` handshake
/// (the supervisor owns the handshake, exactly like the desktop backend;
/// the browser never sends `initialize`).
fn web_client_info() -> ClientInfo {
    ClientInfo {
        name: "codex-rs".to_owned(),
        title: Some("Flauz Web".to_owned()),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

/// The initialize capabilities, mirroring the desktop backend's payload
/// verbatim.
fn web_initialize_capabilities() -> InitializeCapabilities {
    InitializeCapabilities {
        experimental_api: true,
        request_attestation: false,
        mcp_server_openai_form_elicitation: Some(true),
        opt_out_notification_methods: Some(
            STABLE_OPT_OUT_NOTIFICATION_METHODS
                .iter()
                .map(|method| (*method).to_owned())
                .collect(),
        ),
    }
}

/// The desktop backend's reconnect schedule (mirrored).
#[derive(Debug)]
struct ReconnectScheduler {
    next_attempt: u32,
    next_delay: Duration,
}

impl Default for ReconnectScheduler {
    fn default() -> Self {
        Self {
            next_attempt: 1,
            next_delay: RECONNECT_INITIAL_DELAY,
        }
    }
}

impl ReconnectScheduler {
    fn schedule(&mut self) -> (u32, Duration) {
        let attempt = self.next_attempt;
        let delay = self.next_delay;
        self.next_attempt = self.next_attempt.saturating_add(1);
        self.next_delay = self.next_delay.saturating_mul(2).min(RECONNECT_MAX_DELAY);
        (attempt, delay)
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Spawns the supervisor thread. The thread owns the supervised
/// app-server lifecycle until [`SupervisorCommand::Shutdown`] is received
/// or the command channel disconnects; the connection is reaped on exit.
pub fn spawn_supervisor(
    config: SupervisorConfig,
    commands: Receiver<SupervisorCommand>,
    events: Sender<SupervisorEvent>,
    logger: Logger,
    session_id: String,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("flauz-web-gateway-supervisor".to_owned())
        .spawn(move || {
            let mut supervisor = Supervisor {
                config,
                commands,
                events,
                logger,
                session_id,
                method_registry: MethodRegistry::default(),
            };
            supervisor.run();
        })
        .unwrap_or_else(|error| panic!("could not spawn the gateway supervisor thread: {error}"))
}

struct Supervisor {
    config: SupervisorConfig,
    commands: Receiver<SupervisorCommand>,
    events: Sender<SupervisorEvent>,
    logger: Logger,
    session_id: String,
    method_registry: MethodRegistry,
}

impl Supervisor {
    fn run(&mut self) {
        let mut scheduler = ReconnectScheduler::default();
        let mut restarts: u32 = 0;
        'supervision: loop {
            // Spawn (with backoff) until a connection is live.
            let mut backoff: Option<Duration> = None;
            let connection = loop {
                if let Some(delay) = backoff.take()
                    && !self.sleep_while_listening(delay)
                {
                    break 'supervision;
                }
                match self.start_connection() {
                    Ok(connection) => break Arc::new(connection),
                    Err(error) => {
                        let (attempt, delay) = scheduler.schedule();
                        self.emit_state(
                            supervisor_states::RECONNECTING,
                            format!(
                                "the supervised app-server is unavailable ({error}); \
                                 restarting (attempt {attempt}, in {} ms)",
                                delay.as_millis()
                            ),
                        );
                        self.logger.warn(
                            "supervisor_reconnect_scheduled",
                            &[
                                ("session_id", self.session_id.as_str()),
                                ("attempt", &attempt.to_string()),
                                ("delay_ms", &delay.as_millis().to_string()),
                            ],
                        );
                        backoff = Some(delay);
                    }
                }
            };
            scheduler.reset();
            restarts = restarts.saturating_add(1);
            let reason = if restarts == 1 {
                "the supervised app-server is attached and initialized".to_owned()
            } else {
                format!("the supervised app-server restarted (restart {restarts})")
            };
            self.emit_state(supervisor_states::CONNECTED, reason);
            self.logger.info(
                "supervisor_connected",
                &[
                    ("session_id", self.session_id.as_str()),
                    ("restarts", &restarts.to_string()),
                ],
            );

            match self.run_connected(&connection) {
                ConnectedPhase::Died(reason) => {
                    let (attempt, delay) = scheduler.schedule();
                    self.emit_state(
                        supervisor_states::RECONNECTING,
                        format!(
                            "the supervised app-server disconnected ({reason}); restarting \
                             (attempt {attempt}, in {} ms)",
                            delay.as_millis()
                        ),
                    );
                    self.logger.warn(
                        "supervisor_reconnect_scheduled",
                        &[
                            ("session_id", self.session_id.as_str()),
                            ("attempt", &attempt.to_string()),
                            ("delay_ms", &delay.as_millis().to_string()),
                        ],
                    );
                }
                ConnectedPhase::Shutdown => break 'supervision,
            }
            // The dead connection is dropped here: the managed child is
            // reaped by its own graceful shutdown.
        }
        let _ = self.events.send(SupervisorEvent::Closed);
        self.logger.info(
            "supervisor_closed",
            &[("session_id", self.session_id.as_str())],
        );
    }

    /// Spawns and initializes a supervised connection (one attempt).
    fn start_connection(&self) -> Result<AppServerConnection, String> {
        let config = AppServerConfig {
            request_timeout: self.config.request_timeout,
            ..AppServerConfig::new(
                self.config.codex_binary.clone(),
                self.config.codex_home.clone(),
            )
        };
        let connection = AppServerConnection::spawn(config).map_err(|error| {
            bounded_reason(&format!("could not start codex app-server: {error}"))
        })?;
        connection
            .initialize_with_capabilities(web_client_info(), Some(web_initialize_capabilities()))
            .map_err(|error| {
                bounded_reason(&format!("app-server initialization failed: {error}"))
            })?;
        Ok(connection)
    }

    /// Runs the connected phase until the app-server dies or the
    /// supervisor is shut down.
    fn run_connected(&mut self, connection: &Arc<AppServerConnection>) -> ConnectedPhase {
        let inflight = Arc::new(AtomicUsize::new(0));
        loop {
            match self.commands.recv_timeout(TICK) {
                Ok(command) => match self.handle_command(connection, &inflight, command) {
                    CommandOutcome::Continue => {
                        while let Ok(next) = self.commands.try_recv() {
                            match self.handle_command(connection, &inflight, next) {
                                CommandOutcome::Continue => {}
                                CommandOutcome::Died(reason) => {
                                    return ConnectedPhase::Died(reason);
                                }
                                CommandOutcome::Shutdown => return ConnectedPhase::Shutdown,
                            }
                        }
                    }
                    CommandOutcome::Died(reason) => return ConnectedPhase::Died(reason),
                    CommandOutcome::Shutdown => return ConnectedPhase::Shutdown,
                },
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return ConnectedPhase::Shutdown,
            }
            let mut died: Option<String> = None;
            loop {
                match connection.try_recv_event() {
                    Ok(Some(received)) => {
                        if let Some(reason) = self.handle_event(received) {
                            died = Some(reason);
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(error) => {
                        died = Some(bounded_reason(&format!("transport closed: {error}")));
                        break;
                    }
                }
            }
            if let Some(reason) = died {
                return ConnectedPhase::Died(reason);
            }
        }
    }

    fn handle_command(
        &mut self,
        connection: &Arc<AppServerConnection>,
        inflight: &Arc<AtomicUsize>,
        command: SupervisorCommand,
    ) -> CommandOutcome {
        match command {
            SupervisorCommand::BridgeRequest { method, id, params } => {
                self.broker_request(connection, inflight, method, id, params);
                CommandOutcome::Continue
            }
            SupervisorCommand::BridgeNotification { method, params } => {
                let frame = json!({"method": method, "params": params});
                match connection.notify(&frame) {
                    Ok(()) => CommandOutcome::Continue,
                    Err(error) => CommandOutcome::Died(bounded_reason(&format!(
                        "notification transport failed: {error}"
                    ))),
                }
            }
            SupervisorCommand::BridgeResponse { id, result, error } => {
                let outcome = if let Some(error) = error {
                    let code = error.get("code").and_then(Value::as_i64).unwrap_or(-32000);
                    let message = error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("request declined")
                        .to_owned();
                    // The frozen app-server error surface takes a static
                    // message; the browser-side dynamic reason rides the
                    // structured log (never lost, never dropped silently).
                    self.logger.warn(
                        "bridge.server_request_error",
                        &[("reason", message.as_str())],
                    );
                    connection.respond_error(&id, code, "request declined by the browser client")
                } else {
                    connection.respond_success(&id, &result.unwrap_or(Value::Null))
                };
                match outcome {
                    Ok(()) => CommandOutcome::Continue,
                    Err(error) => CommandOutcome::Died(bounded_reason(&format!(
                        "response transport failed: {error}"
                    ))),
                }
            }
            SupervisorCommand::Shutdown => CommandOutcome::Shutdown,
        }
    }

    /// Brokers one browser request: the connection's router only
    /// delivers responses to requests it issued itself, so the request
    /// is re-issued with a connection-side id on a short-lived worker
    /// thread; the response is re-encoded with the browser's original id.
    fn broker_request(
        &mut self,
        connection: &Arc<AppServerConnection>,
        inflight: &Arc<AtomicUsize>,
        method: String,
        id: Value,
        params: Value,
    ) {
        let Some(static_method) = self.method_registry.intern(&method) else {
            let _ = self.events.send(SupervisorEvent::Response {
                id,
                result: Err((
                    gateway_error_codes::INVALID_METHOD,
                    format!(
                        "method {:?} cannot be accepted (empty, oversized, or the bounded \
                         method table is full)",
                        bounded_reason(&method)
                    ),
                )),
            });
            return;
        };
        if inflight.load(Ordering::Acquire) >= self.config.max_inflight_requests {
            let _ = self.events.send(SupervisorEvent::Response {
                id,
                result: Err((
                    gateway_error_codes::TOO_MANY_INFLIGHT,
                    "too many concurrent bridged requests are already in flight; retry after \
                     pending requests settle"
                        .to_owned(),
                )),
            });
            return;
        }
        inflight.fetch_add(1, Ordering::AcqRel);
        let connection = Arc::clone(connection);
        let events = self.events.clone();
        let inflight = Arc::clone(inflight);
        let inflight_outer = Arc::clone(&inflight);
        let id_for_spawn = id.clone();
        let spawned = thread::Builder::new()
            .name("flauz-web-gateway-request".to_owned())
            .spawn(move || {
                let result = connection.request::<Value, Value>(static_method, params);
                let response = match result {
                    Ok(value) => SupervisorEvent::Response {
                        id: id_for_spawn,
                        result: Ok(value),
                    },
                    Err(AppServerError::RequestTimedOut(method)) => SupervisorEvent::Response {
                        id: id_for_spawn,
                        result: Err((
                            gateway_error_codes::REQUEST_TIMEOUT,
                            format!("app-server request `{method}` timed out"),
                        )),
                    },
                    Err(error) => SupervisorEvent::Response {
                        id: id_for_spawn,
                        result: Err((
                            gateway_error_codes::TRANSPORT,
                            bounded_reason(&format!("app-server transport: {error}")),
                        )),
                    },
                };
                // Responses must not be dropped: the bounded event channel
                // applies honest backpressure to the bridge pump.
                let _ = events.send(response);
                inflight.fetch_sub(1, Ordering::AcqRel);
            });
        if spawned.is_err() {
            inflight_outer.fetch_sub(1, Ordering::AcqRel);
            let _ = self.events.send(SupervisorEvent::Response {
                id,
                result: Err((
                    gateway_error_codes::TRANSPORT,
                    "the gateway could not spawn a request worker".to_owned(),
                )),
            });
        }
    }

    /// Handles one received app-server event; `Some(reason)` means the
    /// supervised app-server died (or a server request could not be
    /// delivered) and the supervisor must restart it.
    fn handle_event(&mut self, received: ReceivedAppServerEvent) -> Option<String> {
        let (event, _guard) = received.into_parts();
        let mut dropped: usize = 0;
        match event {
            AppServerEvent::Notification { method, params } => {
                let frame = server_notification(&method, &params);
                if self
                    .events
                    .send_timeout(SupervisorEvent::Frame(frame), EVENT_BACKPRESSURE_TIMEOUT)
                    .is_err()
                {
                    dropped = dropped.saturating_add(1);
                }
            }
            AppServerEvent::Request { id, method, params } => {
                let frame = server_request(&id, &method, &params);
                if self
                    .events
                    .send_timeout(
                        SupervisorEvent::Frame(frame),
                        SERVER_REQUEST_DELIVERY_TIMEOUT,
                    )
                    .is_err()
                {
                    // Honest delivery failure: the supervisor restarts and
                    // the pending server request is failed by the
                    // connection's own pending-request teardown.
                    return Some(format!(
                        "a server request ({method}) could not be delivered to the browser"
                    ));
                }
            }
            AppServerEvent::NotificationsDropped { count } => {
                dropped = dropped.saturating_add(count);
            }
            AppServerEvent::Disconnected => {
                return Some("the app-server closed its transport".to_owned());
            }
        }
        if dropped > 0 {
            let _ = self
                .events
                .send(SupervisorEvent::NotificationsDropped { count: dropped });
        }
        None
    }

    fn emit_state(&self, state: &'static str, reason: String) {
        let _ = self.events.send(SupervisorEvent::State { state, reason });
    }

    /// Sleeps for `delay` in small ticks while remaining responsive to a
    /// shutdown command; returns `false` when the supervisor should exit.
    fn sleep_while_listening(&self, delay: Duration) -> bool {
        let deadline = Instant::now() + delay;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return true;
            }
            match self.commands.recv_timeout(remaining.min(TICK)) {
                Ok(SupervisorCommand::Shutdown) => return false,
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return false,
            }
        }
    }
}

/// The connected-phase outcome.
#[derive(Debug)]
enum ConnectedPhase {
    /// The app-server died with a named reason; restart it.
    Died(String),
    /// The supervisor was asked to shut down.
    Shutdown,
}

/// A bounded intern table for method names: `AppServerConnection::request`
/// takes `&'static str` methods, so bridged method names are interned
/// with a hard cap (no unbounded leak; a full table rejects with a named
/// error).
#[derive(Debug, Default)]
struct MethodRegistry {
    interned: std::sync::Mutex<Vec<&'static str>>,
}

impl MethodRegistry {
    fn intern(&self, method: &str) -> Option<&'static str> {
        if method.is_empty() || method.len() > protocol::MAX_METHOD_BYTES {
            return None;
        }
        let Ok(mut interned) = self.interned.lock() else {
            return None;
        };
        if let Some(found) = interned.iter().find(|existing| **existing == method) {
            return Some(found);
        }
        if interned.len() >= MAX_INTERNED_METHODS {
            return None;
        }
        let leaked: &'static str = Box::leak(method.to_owned().into_boxed_str());
        interned.push(leaked);
        Some(leaked)
    }
}

/// Clamps a reason string to [`MAX_REASON_BYTES`] on a character
/// boundary.
fn bounded_reason(reason: &str) -> String {
    let mut bounded = String::new();
    for ch in reason.chars() {
        if bounded.len() + ch.len_utf8() > MAX_REASON_BYTES {
            break;
        }
        bounded.push(ch);
    }
    bounded
}

/// Builds a browser response frame from a brokered result (used by the
/// bridge pump).
#[must_use]
pub fn brokered_response(id: &Value, result: &Result<Value, (i64, String)>) -> Value {
    match result {
        Ok(value) => rpc_success(id, value.clone()),
        Err((code, message)) => crate::protocol::rpc_failure(id, *code, message),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        MAX_INTERNED_METHODS, MethodRegistry, RECONNECT_INITIAL_DELAY, RECONNECT_MAX_DELAY,
        ReconnectScheduler,
    };
    use crate::protocol::MAX_METHOD_BYTES;

    #[test]
    fn reconnect_schedule_mirrors_the_desktop_backend() {
        let mut scheduler = ReconnectScheduler::default();
        assert_eq!(scheduler.schedule(), (1, Duration::from_secs(1)));
        assert_eq!(scheduler.schedule(), (2, Duration::from_secs(2)));
        assert_eq!(scheduler.schedule(), (3, Duration::from_secs(4)));
        assert_eq!(scheduler.schedule(), (4, Duration::from_secs(8)));
        assert_eq!(scheduler.schedule(), (5, Duration::from_secs(16)));
        assert_eq!(scheduler.schedule(), (6, RECONNECT_MAX_DELAY));
        assert_eq!(scheduler.schedule(), (7, RECONNECT_MAX_DELAY));
        scheduler.reset();
        assert_eq!(scheduler.schedule(), (1, RECONNECT_INITIAL_DELAY));
    }

    #[test]
    fn method_registry_is_bounded_and_reuses_interned_names() {
        let registry = MethodRegistry::default();
        let first = registry.intern("thread/list");
        let second = registry.intern("thread/list");
        assert_eq!(first, second, "interned names must be reused");
        assert!(registry.intern("").is_none());
        assert!(
            registry.intern(&"x".repeat(MAX_METHOD_BYTES + 1)).is_none(),
            "oversized method names must be rejected"
        );
        for index in 0..MAX_INTERNED_METHODS - 1 {
            let method = format!("m{index}/x");
            assert!(
                registry.intern(&method).is_some(),
                "intern slot {index} must be available"
            );
        }
        assert!(
            registry.intern("one/method/too/many").is_none(),
            "the intern table must be bounded"
        );
    }
}
