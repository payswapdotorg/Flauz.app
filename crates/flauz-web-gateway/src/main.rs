//! The `flauz-web-gateway` binary: argument parsing, startup validation,
//! signal-driven graceful shutdown.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use flauz_web_gateway::config::{GatewayConfig, SessionTokens};
use flauz_web_gateway::logging::Logger;
use flauz_web_gateway::server::serve;

const USAGE: &str = "flauz-web-gateway — the Flauz web client gateway (WEB-001)

USAGE:
    flauz-web-gateway [OPTIONS]

OPTIONS:
    --bind <ADDR>                 Bind address (default 127.0.0.1:8610, localhost-only).
                                  A non-loopback bind requires --session-token-file
                                  and refuses to start without it.
    --web-root <DIR>              Static hosting root for the web build (default web/dist).
    --codex-binary <PATH>         Official codex binary (default: CODEX_RS_CODEX_BIN or PATH).
    --codex-home <DIR>            CODEX_HOME for supervised app-servers (default: ~/.codex).
    --session-token-file <PATH>   Operator-provisioned session tokens (one per line);
                                  required for non-local binds; optional for localhost.
    --max-sessions <N>            Maximum concurrent sessions (default 8, max 32).
    --max-inflight <N>            Maximum in-flight bridged requests per session
                                  (default 32, max 64).
    --request-timeout-secs <N>    Brokered request timeout in seconds (default 10).
    -h, --help                    Print this help.
    -V, --version                 Print the version.

LOGS:
    Structured JSON lines on stdout. Credentials, tokens, auth URLs, and
    raw frames are never logged.
";

struct Args {
    bind: Option<SocketAddr>,
    web_root: Option<PathBuf>,
    codex_binary: Option<PathBuf>,
    codex_home: Option<PathBuf>,
    session_token_file: Option<PathBuf>,
    max_sessions: Option<usize>,
    max_inflight: Option<usize>,
    request_timeout_secs: Option<u64>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        bind: None,
        web_root: None,
        codex_binary: None,
        codex_home: None,
        session_token_file: None,
        max_sessions: None,
        max_inflight: None,
        request_timeout_secs: None,
    };
    let mut values = std::env::args().skip(1);
    while let Some(flag) = values.next() {
        let mut value_for = |name: &str| -> Result<String, String> {
            values
                .next()
                .ok_or_else(|| format!("--{name} requires a value"))
        };
        match flag.as_str() {
            "--bind" => {
                let value = value_for("bind")?;
                args.bind = Some(
                    value
                        .parse()
                        .map_err(|_| format!("--bind value {value:?} is not a valid address"))?,
                );
            }
            "--web-root" => args.web_root = Some(PathBuf::from(value_for("web-root")?)),
            "--codex-binary" => args.codex_binary = Some(PathBuf::from(value_for("codex-binary")?)),
            "--codex-home" => args.codex_home = Some(PathBuf::from(value_for("codex-home")?)),
            "--session-token-file" => {
                args.session_token_file = Some(PathBuf::from(value_for("session-token-file")?))
            }
            "--max-sessions" => {
                let value = value_for("max-sessions")?;
                args.max_sessions = Some(
                    value
                        .parse()
                        .map_err(|_| format!("--max-sessions value {value:?} is not a number"))?,
                );
            }
            "--max-inflight" => {
                let value = value_for("max-inflight")?;
                args.max_inflight = Some(
                    value
                        .parse()
                        .map_err(|_| format!("--max-inflight value {value:?} is not a number"))?,
                );
            }
            "--request-timeout-secs" => {
                let value = value_for("request-timeout-secs")?;
                args.request_timeout_secs = Some(value.parse().map_err(|_| {
                    format!("--request-timeout-secs value {value:?} is not a number")
                })?);
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("flauz-web-gateway {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other:?} (see --help)")),
        }
    }
    Ok(args)
}

fn build_config(args: Args) -> Result<GatewayConfig, String> {
    let web_root = args.web_root.unwrap_or_else(|| PathBuf::from("web/dist"));
    let mut config = GatewayConfig::with_defaults(web_root)
        .map_err(|error| format!("could not resolve the gateway defaults: {error}"))?;
    if let Some(bind) = args.bind {
        config.bind = bind;
    }
    if let Some(binary) = args.codex_binary {
        config.codex_binary = binary;
    }
    if let Some(home) = args.codex_home {
        let resolved = codex_platform::CodexHome::resolve(Some(home))
            .map_err(|error| format!("--codex-home is not usable: {error}"))?;
        config.codex_home = resolved;
    }
    if let Some(token_file) = args.session_token_file {
        let tokens = SessionTokens::load(&token_file)
            .map_err(|error| format!("--session-token-file rejected: {error}"))?;
        config.session_tokens = Some(tokens);
    }
    if let Some(max_sessions) = args.max_sessions {
        config.max_sessions = max_sessions;
    }
    if let Some(max_inflight) = args.max_inflight {
        config.max_inflight_requests = max_inflight;
    }
    if let Some(secs) = args.request_timeout_secs {
        config.request_timeout = std::time::Duration::from_secs(secs.max(1));
    }
    Ok(config)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("flauz-web-gateway: {error}");
            return ExitCode::from(2);
        }
    };
    let config = match build_config(args) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("flauz-web-gateway: {error}");
            return ExitCode::from(2);
        }
    };
    let redactions = config
        .session_tokens
        .as_ref()
        .map(|tokens| tokens.token_material_for_redaction())
        .unwrap_or_default();
    let logger = Logger::with_redactions(redactions);
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("flauz-web-gateway: could not start the async runtime: {error}");
            return ExitCode::from(1);
        }
    };
    runtime.block_on(async move {
        let handle = match serve(config, logger).await {
            Ok(handle) => handle,
            Err(error) => {
                eprintln!("flauz-web-gateway: {error}");
                return ExitCode::from(1);
            }
        };
        // Wait for Ctrl+C (SIGINT) or SIGTERM, then drain gracefully.
        wait_for_shutdown_signal().await;
        match handle.shutdown().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("flauz-web-gateway: {error}");
                ExitCode::from(1)
            }
        }
    })
}

#[cfg(unix)]
async fn wait_for_shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let Ok(mut term) = signal(SignalKind::terminate()) else {
        let _ = tokio::signal::ctrl_c().await;
        return;
    };
    tokio::select! {
        _ = term.recv() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
