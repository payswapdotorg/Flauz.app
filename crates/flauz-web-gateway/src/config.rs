//! Gateway configuration and the bind/auth law (Wave-6 kernel addendum §4).

use std::fmt;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;

use codex_platform::{CodexHome, resolve_codex_binary};
use codex_protocol::DEFAULT_MAX_FRAME_BYTES;

/// The default bind address: localhost only. Non-local binding is an
/// explicit operator opt-in and requires provisioned session tokens.
pub const DEFAULT_BIND_ADDR: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 1);

/// The default bind port.
pub const DEFAULT_BIND_PORT: u16 = 8610;

/// The default port label used in logs and help output.
pub const DEFAULT_BIND_LABEL: &str = "127.0.0.1:8610";

/// Default maximum concurrently authenticated gateway sessions.
pub const DEFAULT_MAX_SESSIONS: usize = 8;

/// Default maximum in-flight bridged browser requests per session.
pub const DEFAULT_MAX_INFLIGHT_REQUESTS: usize = 32;

/// Default request timeout, mirroring the desktop backend's app-server
/// request timeout.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum number of concurrently in-flight bridged browser requests per
/// session (hard bound).
pub const MAX_INFLIGHT_REQUESTS: usize = 64;

/// Maximum number of authenticated gateway sessions (hard bound).
pub const MAX_SESSIONS: usize = 32;

/// The maximum number of bytes of a session token file that is read
/// (bounded reads law).
pub const MAX_TOKEN_FILE_BYTES: u64 = 64 * 1024;

/// The minimum length of a session token.
pub const MIN_TOKEN_BYTES: usize = 16;

/// The maximum length of a session token.
pub const MAX_TOKEN_BYTES: usize = 256;

/// The maximum number of distinct tokens a token file may carry.
pub const MAX_TOKENS: usize = 128;

/// Errors returned when a [`GatewayConfig`] cannot be validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayConfigError {
    /// A non-local bind address was requested without provisioned
    /// authenticated-session tokens.
    NonLocalBindRequiresSessionTokens { bind: SocketAddr },
    /// The configured web root does not exist or is not a directory.
    WebRootUnavailable { path: PathBuf },
    /// The session token file could not be read.
    TokenFileUnreadable { path: PathBuf },
    /// The session token file is invalid (named reason).
    TokenFileInvalid { path: PathBuf, reason: String },
    /// CODEX_HOME could not be resolved for supervised app-servers.
    CodexHomeUnavailable,
    /// The maximum session count is outside the allowed range.
    InvalidMaxSessions { requested: usize },
    /// The maximum in-flight request count is outside the allowed range.
    InvalidMaxInflightRequests { requested: usize },
}

impl fmt::Display for GatewayConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonLocalBindRequiresSessionTokens { bind } => write!(
                formatter,
                "refusing to bind non-local address {bind}: non-local binding \
                 requires explicit operator opt-in via --session-token-file \
                 (provisioned authenticated-session tokens); the default bind \
                 is localhost-only ({DEFAULT_BIND_LABEL})"
            ),
            Self::WebRootUnavailable { path } => write!(
                formatter,
                "web root {} does not exist or is not a directory",
                path.display()
            ),
            Self::TokenFileUnreadable { path } => write!(
                formatter,
                "session token file {} could not be read",
                path.display()
            ),
            Self::TokenFileInvalid { path, reason } => write!(
                formatter,
                "session token file {} is invalid: {reason}",
                path.display()
            ),
            Self::CodexHomeUnavailable => {
                formatter.write_str("CODEX_HOME could not be resolved for supervised app-servers")
            }
            Self::InvalidMaxSessions { requested } => write!(
                formatter,
                "max sessions {requested} is outside the allowed range 1..={MAX_SESSIONS}"
            ),
            Self::InvalidMaxInflightRequests { requested } => write!(
                formatter,
                "max in-flight requests {requested} is outside the allowed range \
                 1..={MAX_INFLIGHT_REQUESTS}"
            ),
        }
    }
}

impl std::error::Error for GatewayConfigError {}

/// Operator-provisioned authenticated-session tokens (one per line). Only
/// SHA-free in-memory token strings are kept; they are never logged and
/// never appear in any artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTokens {
    tokens: Vec<String>,
}

impl SessionTokens {
    /// Loads tokens from a file: one token per line, `#` comments and
    /// blank lines ignored. Reads are bounded to
    /// [`MAX_TOKEN_FILE_BYTES`].
    ///
    /// # Errors
    ///
    /// Returns a [`GatewayConfigError`] when the file cannot be read, a
    /// token is malformed, or the token count exceeds [`MAX_TOKENS`].
    pub fn load(path: &Path) -> Result<Self, GatewayConfigError> {
        let metadata = fs::metadata(path).map_err(|_| GatewayConfigError::TokenFileUnreadable {
            path: path.to_path_buf(),
        })?;
        if metadata.len() > MAX_TOKEN_FILE_BYTES {
            return Err(GatewayConfigError::TokenFileInvalid {
                path: path.to_path_buf(),
                reason: format!("file exceeds {MAX_TOKEN_FILE_BYTES} bytes"),
            });
        }
        let contents =
            fs::read_to_string(path).map_err(|_| GatewayConfigError::TokenFileUnreadable {
                path: path.to_path_buf(),
            })?;
        Self::from_text(path, &contents)
    }

    /// Parses tokens from text (exposed for tests).
    ///
    /// # Errors
    ///
    /// Returns a [`GatewayConfigError`] when a token is malformed or the
    /// token count exceeds [`MAX_TOKENS`].
    pub fn from_text(path: &Path, text: &str) -> Result<Self, GatewayConfigError> {
        let mut tokens = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let valid = line.len() >= MIN_TOKEN_BYTES
                && line.len() <= MAX_TOKEN_BYTES
                && line
                    .bytes()
                    .all(|byte| byte.is_ascii_graphic() || byte == b' ')
                && !line.contains(char::is_whitespace);
            if !valid {
                return Err(GatewayConfigError::TokenFileInvalid {
                    path: path.to_path_buf(),
                    reason: format!(
                        "a token line is malformed (expected {MIN_TOKEN_BYTES}..={MAX_TOKEN_BYTES} \
                         printable non-whitespace bytes)"
                    ),
                });
            }
            tokens.push(line.to_owned());
        }
        if tokens.is_empty() {
            return Err(GatewayConfigError::TokenFileInvalid {
                path: path.to_path_buf(),
                reason: "no tokens present".to_owned(),
            });
        }
        if tokens.len() > MAX_TOKENS {
            return Err(GatewayConfigError::TokenFileInvalid {
                path: path.to_path_buf(),
                reason: format!("more than {MAX_TOKENS} tokens"),
            });
        }
        Ok(Self { tokens })
    }

    /// The number of provisioned tokens.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Returns a copy of the token material for the logger's DEFENSIVE
    /// redaction guard only (so an accidental log line can never carry
    /// it). This is the only sanctioned consumer of the raw material.
    #[must_use]
    pub fn token_material_for_redaction(&self) -> Vec<String> {
        self.tokens.clone()
    }

    /// Whether any tokens are provisioned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Validates a presented token with a fixed-time comparison against
    /// every provisioned token. Token material is never exposed beyond
    /// this call.
    #[must_use]
    pub fn validate(&self, presented: &str) -> bool {
        let mut matched = false;
        for token in &self.tokens {
            // Fixed-time comparison over the padded length so an attacker
            // cannot shortcut on length mismatch timing.
            matched |= fixed_time_eq(presented.as_bytes(), token.as_bytes());
        }
        matched
    }
}

/// Byte-wise fixed-time equality over the maximum of both lengths.
fn fixed_time_eq(left: &[u8], right: &[u8]) -> bool {
    let len = left.len().max(right.len());
    let mut diff: u8 =
        left.len().min(usize::from(u8::MAX)) as u8 ^ right.len().min(usize::from(u8::MAX)) as u8;
    for index in 0..len {
        let l = left.get(index).copied().unwrap_or(0);
        let r = right.get(index).copied().unwrap_or(0);
        diff |= l ^ r;
    }
    diff == 0
}

/// The resolved gateway configuration.
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// The address to bind. Loopback by default.
    pub bind: SocketAddr,
    /// The directory holding the built web client (static hosting root).
    pub web_root: PathBuf,
    /// The official `codex` binary used to spawn app-servers.
    pub codex_binary: PathBuf,
    /// The resolved CODEX_HOME for supervised app-servers.
    pub codex_home: CodexHome,
    /// Operator-provisioned authenticated-session tokens, when enabled.
    pub session_tokens: Option<SessionTokens>,
    /// Maximum concurrently authenticated sessions.
    pub max_sessions: usize,
    /// Maximum in-flight bridged browser requests per session.
    pub max_inflight_requests: usize,
    /// The app-server request timeout used by the supervisor.
    pub request_timeout: Duration,
    /// The maximum accepted WebSocket text frame bytes.
    pub max_frame_bytes: usize,
}

impl GatewayConfig {
    /// Builds a configuration with the documented defaults: localhost-only
    /// bind, `web_root` as the static root, the resolved official codex
    /// binary and CODEX_HOME, no session tokens, and desktop-parity
    /// request bounds.
    ///
    /// # Errors
    ///
    /// Returns a [`GatewayConfigError`] only from the underlying
    /// CODEX_HOME resolution (no home directory available).
    pub fn with_defaults(web_root: PathBuf) -> Result<Self, GatewayConfigError> {
        let bind = SocketAddr::new(IpAddr::V4(DEFAULT_BIND_ADDR), DEFAULT_BIND_PORT);
        let codex_home =
            CodexHome::resolve(None).map_err(|_| GatewayConfigError::CodexHomeUnavailable)?;
        Ok(Self {
            bind,
            web_root,
            codex_binary: resolve_codex_binary(None),
            codex_home,
            session_tokens: None,
            max_sessions: DEFAULT_MAX_SESSIONS,
            max_inflight_requests: DEFAULT_MAX_INFLIGHT_REQUESTS,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
        })
    }

    /// Whether the configured bind address is loopback-only.
    #[must_use]
    pub fn binds_loopback(&self) -> bool {
        self.bind.ip().is_loopback()
    }

    /// Validates the configuration against the bind/auth law and bounds.
    ///
    /// # Errors
    ///
    /// - [`GatewayConfigError::NonLocalBindRequiresSessionTokens`] when a
    ///   non-loopback bind is requested without provisioned session
    ///   tokens (the gateway refuses to start);
    /// - [`GatewayConfigError::WebRootUnavailable`] when the web root is
    ///   missing;
    /// - bound errors for the session and in-flight limits.
    pub fn validate(&self) -> Result<(), GatewayConfigError> {
        if !self.binds_loopback() && self.session_tokens.is_none() {
            return Err(GatewayConfigError::NonLocalBindRequiresSessionTokens { bind: self.bind });
        }
        if !self.web_root.is_dir() {
            return Err(GatewayConfigError::WebRootUnavailable {
                path: self.web_root.clone(),
            });
        }
        if self.max_sessions == 0 || self.max_sessions > MAX_SESSIONS {
            return Err(GatewayConfigError::InvalidMaxSessions {
                requested: self.max_sessions,
            });
        }
        if self.max_inflight_requests == 0 || self.max_inflight_requests > MAX_INFLIGHT_REQUESTS {
            return Err(GatewayConfigError::InvalidMaxInflightRequests {
                requested: self.max_inflight_requests,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use codex_platform::CodexHome;

    use super::{DEFAULT_BIND_LABEL, GatewayConfig, GatewayConfigError, MAX_TOKENS, SessionTokens};

    fn config_with_bind(bind: SocketAddr) -> GatewayConfig {
        let mut config = valid_config();
        config.bind = bind;
        config
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

    #[test]
    fn default_bind_is_localhost_only() {
        let config = valid_config();
        assert!(config.binds_loopback());
        assert_eq!(
            config.bind.to_string(),
            DEFAULT_BIND_LABEL,
            "the documented default bind must be {DEFAULT_BIND_LABEL}"
        );
    }

    #[test]
    fn non_local_bind_refused_without_session_tokens() {
        let bind = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8610);
        let error = config_with_bind(bind).validate().unwrap_err();
        assert_eq!(
            error,
            GatewayConfigError::NonLocalBindRequiresSessionTokens { bind }
        );
        assert!(error.to_string().contains("refusing to bind non-local"));
    }

    #[test]
    fn non_local_bind_allowed_with_session_tokens() {
        let bind = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8610);
        let mut config = config_with_bind(bind);
        let Ok(tokens) =
            SessionTokens::from_text(Path::new("tokens"), "0123456789abcdef0123456789abcdef")
        else {
            panic!("token parse failed");
        };
        config.session_tokens = Some(tokens);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn missing_web_root_is_rejected() {
        let mut config = valid_config();
        config.web_root = PathBuf::from("does-not-exist-web-root");
        assert!(matches!(
            config.validate(),
            Err(GatewayConfigError::WebRootUnavailable { .. })
        ));
    }

    #[test]
    fn session_bounds_are_enforced() {
        let mut config = valid_config();
        config.max_sessions = 0;
        assert!(matches!(
            config.validate(),
            Err(GatewayConfigError::InvalidMaxSessions { .. })
        ));
        let mut config = valid_config();
        config.max_inflight_requests = 0;
        assert!(matches!(
            config.validate(),
            Err(GatewayConfigError::InvalidMaxInflightRequests { .. })
        ));
    }

    #[test]
    fn token_file_parsing_rejects_malformed_lines() {
        let path = PathBuf::from("tokens");
        for bad in [
            "",                            // no tokens at all
            "short",                       // below MIN_TOKEN_BYTES
            "with space 0123456789abcdef", // whitespace inside a token
            "bad\ttoken 0123456789abcdef", // control character
        ] {
            assert!(
                SessionTokens::from_text(&path, bad).is_err(),
                "expected rejection for {bad:?}"
            );
        }
        let tokens = SessionTokens::from_text(
            &path,
            "# comment\n0123456789abcdef0123456789abcdef\n\n0123456789abcdef0123456789abcdee\n",
        )
        .unwrap_or_else(|error| panic!("valid token file rejected: {error}"));
        assert_eq!(tokens.len(), 2);
        assert!(tokens.validate("0123456789abcdef0123456789abcdef"));
        assert!(tokens.validate("0123456789abcdef0123456789abcdee"));
        assert!(!tokens.validate("0123456789abcdef0123456789abcddf"));
        assert!(!tokens.validate(""));
    }

    #[test]
    fn token_count_is_bounded() {
        let path = PathBuf::from("tokens");
        let text = (0..=MAX_TOKENS)
            .map(|index| format!("{index:016x}{index:016x}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(SessionTokens::from_text(&path, &text).is_err());
    }
}
