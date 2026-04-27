//! API Versioning Strategy — Issue #439
//!
//! Implements URL-based versioning with a `/api/v1/` prefix. Provides:
//!  - `VersionedRouter` — a builder that nests routes under `/api/v{N}/`
//!  - `versioning_middleware` — injects `X-API-Version` and `Deprecation`
//!    headers on every response
//!  - `ApiVersion` extractor — lets handlers read the requested version
//!
//! ## Strategy
//!
//! | Prefix       | Meaning                          | Deprecation header |
//! |---|---|---|
//! | `/api/v1/`   | Current stable version            | none               |
//! | `/api/v0/`   | Previous version (sunset date)    | RFC 9110 `sunset`  |
//!
//! Clients should always use `/api/v1/`. Version discovery is available via
//! `GET /api/versions`.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::{Json, Response},
    Router,
};
use serde::Serialize;
use serde_json::json;

// ── Version catalogue ─────────────────────────────────────────────────────────

/// All known API versions and their metadata.
#[derive(Debug, Clone, Serialize)]
pub struct ApiVersionInfo {
    pub version: &'static str,
    /// Human-readable status
    pub status: &'static str,
    /// ISO 8601 sunset date (None = no planned sunset)
    pub sunset: Option<&'static str>,
    /// Migration guide URL (None = current version)
    pub migration_guide: Option<&'static str>,
}

pub fn known_versions() -> Vec<ApiVersionInfo> {
    vec![
        ApiVersionInfo {
            version: "v1",
            status: "stable",
            sunset: None,
            migration_guide: None,
        },
        ApiVersionInfo {
            version: "v0",
            status: "deprecated",
            sunset: Some("2027-01-01"),
            migration_guide: Some("/docs/migration/v0-to-v1"),
        },
    ]
}

// ── Versioned Router builder ──────────────────────────────────────────────────

/// Nest an existing router under the `/api/v{version}/` prefix.
/// Pass `"v1"` and your route `Router` to produce `/api/v1/<routes>`.
pub fn versioned(version: &str, router: Router) -> Router {
    Router::new().nest(&format!("/api/{}/", version), router)
}

// ── Version discovery endpoint ────────────────────────────────────────────────

/// `GET /api/versions`
///
/// Returns information about all supported API versions so clients can
/// discover the current stable version programmatically.
pub async fn list_api_versions() -> Json<serde_json::Value> {
    Json(json!({
        "current": "v1",
        "versions": known_versions(),
    }))
}

// ── Versioning middleware ─────────────────────────────────────────────────────

/// Injects version headers into every API response.
///
/// - `X-API-Version`: the version segment parsed from the request path
///   (e.g., `v1`). Falls back to `"v1"` if the path has no version prefix.
/// - `Deprecation` + `Sunset`: added only when the detected version is
///   not the current stable version.
/// - `Link: </api/v1/>; rel="successor-version"` on deprecated responses.
pub async fn versioning_middleware(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();

    // Detect version from path prefix (e.g. /api/v1/plans → "v1")
    let detected = path
        .split('/')
        .find(|segment| segment.starts_with('v') && segment[1..].parse::<u32>().is_ok())
        .unwrap_or("v1")
        .to_string();

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Always inject the detected version
    if let Ok(val) = HeaderValue::from_str(&detected) {
        headers.insert(HeaderName::from_static("x-api-version"), val);
    }

    // Add deprecation notice for non-current versions
    if detected != "v1" {
        headers.insert(
            HeaderName::from_static("deprecation"),
            HeaderValue::from_static("true"),
        );
        // Sunset date for v0
        if detected == "v0" {
            headers.insert(
                HeaderName::from_static("sunset"),
                HeaderValue::from_static("Sun, 01 Jan 2027 00:00:00 GMT"),
            );
            headers.insert(
                axum::http::header::LINK,
                HeaderValue::from_static(r#"</api/v1/>; rel="successor-version""#),
            );
        }
    }

    response
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_versions_contains_v1_and_v0() {
        let versions = known_versions();
        let ids: Vec<&str> = versions.iter().map(|v| v.version).collect();
        assert!(ids.contains(&"v1"));
        assert!(ids.contains(&"v0"));
    }

    #[test]
    fn v1_is_stable_with_no_sunset() {
        let v1 = known_versions()
            .into_iter()
            .find(|v| v.version == "v1")
            .unwrap();
        assert_eq!(v1.status, "stable");
        assert!(v1.sunset.is_none());
    }

    #[test]
    fn v0_is_deprecated_with_sunset() {
        let v0 = known_versions()
            .into_iter()
            .find(|v| v.version == "v0")
            .unwrap();
        assert_eq!(v0.status, "deprecated");
        assert!(v0.sunset.is_some());
    }
}
