//! Bounded static-file hosting for the built web client (`web/dist`),
//! with SPA fallback and path-traversal refusal.

use std::fs;
use std::path::{Path, PathBuf};

use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

/// The maximum size of a served static file (bounded reads law).
pub const MAX_STATIC_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// Serves files from a fixed root directory.
#[derive(Debug, Clone)]
pub struct StaticFiles {
    root: PathBuf,
}

impl StaticFiles {
    /// Creates a static file server rooted at `root`.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// The served root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a request path to a response: known file → file bytes,
    /// unknown path → the SPA fallback (`index.html`) for extension-less
    /// routes, `404` for missing assets. Path traversal is refused.
    #[must_use]
    pub fn serve(&self, request_path: &str) -> Response {
        let relative = Self::safe_relative_path(request_path);
        let Some(relative) = relative else {
            return not_found();
        };
        let candidate = self.root.join(&relative);
        if candidate.is_file() {
            return self.serve_file(&candidate, &relative);
        }
        // SPA fallback: extension-less routes serve the shell.
        if relative.extension().is_none() {
            let index = self.root.join("index.html");
            if index.is_file() {
                return self.serve_file(&index, Path::new("index.html"));
            }
        }
        not_found()
    }

    /// Serves `index.html` (the app shell entry).
    #[must_use]
    pub fn serve_index(&self) -> Response {
        let index = self.root.join("index.html");
        if index.is_file() {
            self.serve_file(&index, Path::new("index.html"))
        } else {
            not_found()
        }
    }

    fn serve_file(&self, path: &Path, relative: &Path) -> Response {
        let Ok(metadata) = fs::metadata(path) else {
            return not_found();
        };
        if metadata.len() > MAX_STATIC_FILE_BYTES {
            return too_large();
        }
        let Ok(bytes) = fs::read(path) else {
            return not_found();
        };
        let content_type = content_type_for(relative);
        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(content_type) {
            headers.insert(header::CONTENT_TYPE, value);
        }
        headers.insert(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        );
        // The shell must never be served stale after an operator rebuild;
        // hashed assets are cached politely for the session.
        let cache = if relative.extension().is_some_and(|ext| ext == "html") {
            "no-store"
        } else {
            "no-cache"
        };
        if let Ok(value) = HeaderValue::from_str(cache) {
            headers.insert(header::CACHE_CONTROL, value);
        }
        (StatusCode::OK, headers, bytes).into_response()
    }

    /// Normalizes a request path to a safe relative path, refusing
    /// traversal (`..`, absolute components, backslash tricks, NUL, and
    /// non-normal components).
    fn safe_relative_path(request_path: &str) -> Option<PathBuf> {
        // An ABSOLUTE path (protocol-relative "//host/..." or a rooted
        // "/abs" form) must never be treated as an in-app route: a
        // missing absolute target is NOT SPA fallback material — it is
        // a refusal (404). Only relative in-app routes may fall back to
        // the shell (the 2026-09-25 Lead gate fix: "//etc/passwd" used
        // to SPA-serve the shell with 200).
        let path = request_path.trim_start_matches('/');
        if path.is_empty() {
            return Some(PathBuf::from("index.html"));
        }
        if request_path.starts_with("//") {
            return None;
        }
        if path.len() > 512 || path.contains('\0') || path.contains('\\') {
            return None;
        }
        let mut relative = PathBuf::new();
        for component in Path::new(path).components() {
            match component {
                std::path::Component::Normal(part) => relative.push(part),
                _ => return None,
            }
        }
        if relative.as_os_str().is_empty() {
            None
        } else {
            Some(relative)
        }
    }
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )],
        "not found",
    )
        .into_response()
}

fn too_large() -> Response {
    (
        StatusCode::FORBIDDEN,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )],
        "file too large to serve",
    )
        .into_response()
}

fn content_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
    {
        Some(ext) if ext == "html" || ext == "htm" => "text/html; charset=utf-8",
        Some(ext) if ext == "js" || ext == "mjs" => "text/javascript; charset=utf-8",
        Some(ext) if ext == "css" => "text/css; charset=utf-8",
        Some(ext) if ext == "json" || ext == "map" => "application/json; charset=utf-8",
        Some(ext) if ext == "svg" => "image/svg+xml",
        Some(ext) if ext == "png" => "image/png",
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ext) if ext == "ico" => "image/x-icon",
        Some(ext) if ext == "wasm" => "application/wasm",
        Some(ext) if ext == "txt" => "text/plain; charset=utf-8",
        Some(ext) if ext == "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

/// The health endpoint payload: gateway liveness and session count only
/// (no credentials, no tokens).
#[must_use]
pub fn health_payload(bind: &str, uptime_ms: u128, sessions: usize) -> Value {
    json!({
        "v": 1,
        "kind": "flauz.web-gateway.health",
        "status": "ok",
        "bind": bind,
        "uptimeMs": u64::try_from(uptime_ms).unwrap_or(u64::MAX),
        "sessions": sessions,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use axum::http::StatusCode;
    use tempfile::TempDir;

    use super::StaticFiles;

    fn fixture_web_root() -> TempDir {
        let dir = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
        fs::write(dir.path().join("index.html"), "<html>shell</html>")
            .unwrap_or_else(|error| panic!("index fixture: {error}"));
        fs::create_dir(dir.path().join("assets"))
            .unwrap_or_else(|error| panic!("assets dir: {error}"));
        fs::write(dir.path().join("assets/app.js"), "console.log(1);")
            .unwrap_or_else(|error| panic!("asset fixture: {error}"));
        fs::write(dir.path().join("secret.txt"), "top secret")
            .unwrap_or_else(|error| panic!("secret fixture: {error}"));
        dir
    }

    #[test]
    fn serves_index_and_assets_with_content_types() {
        let root = fixture_web_root();
        let statics = StaticFiles::new(root.path().to_path_buf());
        let index = statics.serve_index();
        assert_eq!(index.status(), StatusCode::OK);
        assert_eq!(
            index
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("text/html; charset=utf-8")
        );
        assert_eq!(
            index
                .headers()
                .get("cache-control")
                .and_then(|v| v.to_str().ok()),
            Some("no-store")
        );
        let asset = statics.serve("/assets/app.js");
        assert_eq!(asset.status(), StatusCode::OK);
        assert_eq!(
            asset
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("text/javascript; charset=utf-8")
        );
    }

    #[test]
    fn spa_routes_fall_back_to_the_shell() {
        let root = fixture_web_root();
        let statics = StaticFiles::new(root.path().to_path_buf());
        let response = statics.serve("/workspace");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("text/html; charset=utf-8")
        );
    }

    #[test]
    fn path_traversal_is_refused() {
        let root = fixture_web_root();
        let statics = StaticFiles::new(root.path().to_path_buf());
        for traversal in [
            "/../secret.txt",
            "/..%2Fsecret.txt",
            "/a/../../secret.txt",
            "//etc/passwd",
            "/assets/../../../secret.txt",
            "/C:\\windows\\win.ini",
        ] {
            let response = statics.serve(traversal);
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "traversal {traversal} must not be served"
            );
        }
    }

    #[test]
    fn missing_assets_are_404() {
        let root = fixture_web_root();
        let statics = StaticFiles::new(root.path().to_path_buf());
        assert_eq!(
            statics.serve("/assets/missing.js").status(),
            StatusCode::NOT_FOUND
        );
    }
}
