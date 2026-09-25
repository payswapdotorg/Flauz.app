//! The gateway server assembly: routes, health, protocol descriptor,
//! static hosting, and graceful shutdown.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize};

use axum::Router;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::config::GatewayConfig;
use crate::logging::Logger;
use crate::protocol::gateway_protocol_descriptor;
use crate::session::{self, Shared};
use std::time::Duration;

use crate::static_files::{StaticFiles, health_payload};

/// The bound on graceful-shutdown drain time.
const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// Errors returned while starting the gateway.
#[derive(Debug)]
pub enum ServeError {
    /// The configuration failed validation.
    Config(crate::config::GatewayConfigError),
    /// The bind address could not be bound.
    Bind(std::io::Error),
}

impl std::fmt::Display for ServeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "gateway configuration rejected: {error}"),
            Self::Bind(error) => write!(formatter, "could not bind the gateway address: {error}"),
        }
    }
}

impl std::error::Error for ServeError {}

/// A running gateway handle.
#[derive(Debug)]
pub struct GatewayHandle {
    /// The actual bound address (useful when port 0 was requested).
    pub local_addr: SocketAddr,
    /// The graceful-shutdown trigger.
    shutdown: watch::Sender<bool>,
    /// The server task.
    join: tokio::task::JoinHandle<Result<(), std::io::Error>>,
}

impl GatewayHandle {
    /// Triggers a graceful shutdown and waits for the drain to finish
    /// (bounded by [`GRACEFUL_SHUTDOWN_TIMEOUT`]).
    ///
    /// # Errors
    ///
    /// Returns an error when the server task failed or the drain timed
    /// out.
    pub async fn shutdown(mut self) -> Result<(), ServeError> {
        let _ = self.shutdown.send(true);
        let drained = tokio::time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, &mut self.join).await;
        match drained {
            Ok(Ok(Ok(()))) => Ok(()),
            Ok(Ok(Err(error))) => Err(ServeError::Bind(error)),
            Ok(Err(_)) => Ok(()),
            Err(_) => Err(ServeError::Bind(std::io::Error::other(
                "graceful shutdown drain timed out",
            ))),
        }
    }
}

/// Serves the gateway: WebSocket bridge at `/ws`, health at `/healthz`,
/// the protocol descriptor at `/gateway-protocol.json`, and static
/// hosting for the web build with SPA fallback.
///
/// # Errors
///
/// Returns [`ServeError::Config`] when the configuration violates the
/// bind/auth law (a non-local bind without provisioned session tokens
/// refuses to start) and [`ServeError::Bind`] when the address cannot be
/// bound.
pub async fn serve(config: GatewayConfig, logger: Logger) -> Result<GatewayHandle, ServeError> {
    config.validate().map_err(ServeError::Config)?;
    let bind = config.bind;
    let statics = StaticFiles::new(config.web_root.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    // Bind first so a bind failure is reported before any state is built.
    let listener = TcpListener::bind(bind).await.map_err(ServeError::Bind)?;
    let local_addr = listener.local_addr().map_err(ServeError::Bind)?;
    let shared = Shared {
        config: Arc::new(config),
        logger,
        statics: Arc::new(statics),
        active_sessions: Arc::new(AtomicUsize::new(0)),
        session_counter: Arc::new(AtomicU64::new(0)),
        shutdown: shutdown_rx,
        started_at: std::time::Instant::now(),
    };
    let origin_refusal_note = if shared.config.binds_loopback() {
        "localhost-only"
    } else {
        "non-local (token-gated)"
    };
    shared.logger.info(
        "gateway_starting",
        &[
            ("bind", &shared.config.bind.to_string()),
            ("web_root", &shared.config.web_root.display().to_string()),
            (
                "codex_binary",
                &shared.config.codex_binary.display().to_string(),
            ),
            ("bind_law", origin_refusal_note),
            (
                "session_tokens",
                if shared.config.session_tokens.is_some() {
                    "provisioned"
                } else {
                    "none"
                },
            ),
        ],
    );
    shared
        .logger
        .info("gateway_listening", &[("bind", &local_addr.to_string())]);
    let app = Router::new()
        .route("/ws", get(session::websocket_handler))
        .route("/healthz", get(health_handler))
        .route("/gateway-protocol.json", get(protocol_handler))
        .fallback(static_handler)
        .with_state(shared);
    let mut graceful_rx = shutdown_tx.subscribe();
    let join = tokio::spawn(async move {
        let serve = axum::serve(listener, app).with_graceful_shutdown(async move {
            // Wait until the shutdown flag flips (the handle triggers it,
            // or the process signal path in `main`).
            while !*graceful_rx.borrow_and_update() && graceful_rx.changed().await.is_ok() {}
        });
        serve.await
    });
    Ok(GatewayHandle {
        local_addr,
        shutdown: shutdown_tx,
        join,
    })
}

async fn health_handler(State(shared): State<Shared>) -> Response {
    let sessions = shared
        .active_sessions
        .load(std::sync::atomic::Ordering::Acquire);
    let uptime = shared.started_at.elapsed().as_millis();
    let payload = health_payload(&shared.config.bind.to_string(), uptime, sessions);
    axum::Json(payload).into_response()
}

async fn protocol_handler() -> Response {
    axum::Json(gateway_protocol_descriptor()).into_response()
}

async fn static_handler(State(shared): State<Shared>, request: Request) -> Response {
    let path = request.uri().path().to_owned();
    let statics = Arc::clone(&shared.statics);
    // Static serving is bounded, blocking file IO: keep it off the
    // async reactor.
    let response = tokio::task::spawn_blocking(move || {
        if path == "/" {
            statics.serve_index()
        } else {
            statics.serve(&path)
        }
    })
    .await;
    match response {
        Ok(response) => response,
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "static serving failed").into_response(),
    }
}

/// A tiny alias for the public gateway server type.
pub type GatewayServer = GatewayHandle;

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, SocketAddr};
    use std::path::PathBuf;
    use std::time::Duration;

    use codex_platform::CodexHome;

    use crate::config::{GatewayConfig, GatewayConfigError};
    use crate::server::ServeError;

    #[test]
    fn non_local_bind_without_tokens_refuses_to_serve() {
        let mut config = valid_config();
        let Ok(bind) = "0.0.0.0:8610".parse() else {
            panic!("test bind parse failed");
        };
        config.bind = bind;
        // Deliberately no session tokens.
        let error = match config.validate() {
            Err(error) => error,
            Ok(()) => panic!("a non-local bind without tokens must be refused"),
        };
        assert!(matches!(
            error,
            GatewayConfigError::NonLocalBindRequiresSessionTokens { .. }
        ));
        // The serve() path surfaces the same refusal before any bind.
        let display = ServeError::Config(error).to_string();
        assert!(display.contains("refusing to bind non-local"));
    }

    fn valid_config() -> GatewayConfig {
        // Hermetic: resolve CODEX_HOME from an explicit temp dir — the
        // test never depends on the runner's $HOME having ~/.codex (the
        // Lead-gate environment does not) and needs no unsafe env writes.
        let home_dir = std::env::temp_dir().join("flauz-web-gateway-test-codex-home");
        std::fs::create_dir_all(&home_dir).expect("test codex home dir");
        let codex_home = CodexHome::resolve(Some(home_dir)).expect("codex home resolves");
        GatewayConfig {
            bind: SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 8610),
            web_root: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            codex_binary: PathBuf::from("codex"),
            codex_home,
            session_tokens: None,
            max_sessions: 8,
            max_inflight_requests: 64,
            request_timeout: Duration::from_secs(30),
            max_frame_bytes: codex_protocol::DEFAULT_MAX_FRAME_BYTES,
        }
    }
}
