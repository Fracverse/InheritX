use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

/// Request ID header name.
pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Injects a unique `x-request-id` into each request and propagates it to the response.
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    req.headers_mut().insert(
        X_REQUEST_ID.clone(),
        HeaderValue::from_str(&request_id).unwrap(),
    );

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        X_REQUEST_ID.clone(),
        HeaderValue::from_str(&request_id).unwrap(),
    );
    response
}

/// Logs each incoming request with its method, URI, and assigned request ID.
pub async fn request_logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let request_id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();

    tracing::info!(request_id = %request_id, method = %method, uri = %uri, "incoming request");

    let response = next.run(req).await;

    tracing::info!(
        request_id = %request_id,
        status = %response.status(),
        "request completed"
    );

    response
}

/// Adds security headers to every response:
/// - Strict-Transport-Security (HSTS)
/// - Content-Security-Policy (CSP)
/// - X-Frame-Options
/// - X-Content-Type-Options
/// - Referrer-Policy
pub async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}

/// Enforces a per-request timeout. Returns 408 if the handler exceeds the limit.
pub async fn request_timeout_middleware(
    req: Request<Body>,
    next: Next,
    duration: Duration,
) -> Response {
    match timeout(duration, next.run(req)).await {
        Ok(response) => response,
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            axum::Json(serde_json::json!({ "error": "Request timed out" })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware, routing::get, Router};
    use tower::ServiceExt;

    async fn dummy_handler() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn test_request_id_is_injected() {
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(middleware::from_fn(request_id_middleware));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert!(response.headers().contains_key(&X_REQUEST_ID));
    }

    #[tokio::test]
    async fn test_security_headers_present() {
        let app = Router::new()
            .route("/", get(dummy_handler))
            .layer(middleware::from_fn(security_headers_middleware));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert!(response.headers().contains_key("strict-transport-security"));
        assert!(response.headers().contains_key("x-frame-options"));
        assert!(response.headers().contains_key("x-content-type-options"));
    }

    #[tokio::test]
    async fn test_request_timeout_fires() {
        let app = Router::new()
            .route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    "done"
                }),
            )
            .layer(middleware::from_fn(|req, next| {
                request_timeout_middleware(req, next, Duration::from_millis(50))
            }));

        let response = app
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }
}
