//! `flauz-web-gateway` — the WEB-001 web foundation gateway.
//!
//! A transparent WebSocket-to-app-server bridge plus static-file hosting for
//! the Flauz web client (`web/`):
//!
//! - one supervised official `codex app-server` per authenticated browser
//!   session, reusing `codex-platform`'s [`AppServerConnection`] (the same
//!   spawn/restart/reap supervision semantics as the desktop backend
//!   binding);
//! - a localhost-only default bind; non-local binding is an explicit
//!   operator opt-in that refuses to start without provisioned
//!   authenticated-session tokens;
//! - JSON-RPC (app-server dialect) frames bridged between the browser and
//!   the supervised app-server with ZERO product logic — the app-server
//!   JSON-RPC protocol is the only capability contract (Wave-6 kernel
//!   addendum §1);
//! - structured JSON logs that never contain credentials, tokens, or raw
//!   frames;
//! - graceful shutdown with named disconnect events.
//!
//! The gateway WebSocket protocol (the `session.claim` handshake, gateway
//! state events, and the transparent frame dialect) is documented by
//! [`protocol::gateway_protocol_descriptor`] and served at
//! `GET /gateway-protocol.json`.

pub mod config;
pub mod logging;
pub mod protocol;
pub mod server;
pub mod session;
pub mod static_files;
pub mod supervisor;

pub use config::{GatewayConfig, GatewayConfigError, SessionTokens};
pub use server::{GatewayHandle, GatewayServer, ServeError};
