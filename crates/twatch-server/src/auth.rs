//! Local-first authentication and browser hardening.

use axum::extract::{Request, State};
use axum::http::{HeaderValue, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::ApiError;
use crate::state::AppState;

/// Extract a bearer token or the `twatch_token` cookie.
#[must_use]
pub fn token_from_headers(headers: &header::HeaderMap) -> Option<String> {
    if let Some(value) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
        && let Some(token) = value.strip_prefix("Bearer ")
    {
        return Some(token.to_owned());
    }
    let cookies = headers.get(header::COOKIE).and_then(|v| v.to_str().ok())?;
    cookies.split(';').find_map(|cookie| {
        cookie.trim().strip_prefix("twatch_token=").map(std::string::ToString::to_string)
    })
}

#[must_use]
pub fn has_valid_token(headers: &header::HeaderMap, expected: &str) -> bool {
    if let Some(value) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
        && let Some(token) = value.strip_prefix("Bearer ")
        && constant_time_eq(token.as_bytes(), expected.as_bytes())
    {
        return true;
    }
    if let Some(cookies) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        for cookie in cookies.split(';') {
            if let Some(token) = cookie.trim().strip_prefix("twatch_token=")
                && constant_time_eq(token.as_bytes(), expected.as_bytes())
            {
                return true;
            }
        }
    }
    false
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (left, right) in a.iter().zip(b) {
        diff |= left ^ right;
    }
    diff == 0
}

fn hostname(host: &str) -> &str {
    if let Some(rest) = host.strip_prefix('[') {
        return rest.split(']').next().unwrap_or(rest);
    }
    host.split(':').next().unwrap_or(host)
}

fn is_loopback_host(host: &str) -> bool {
    matches!(hostname(host), "localhost" | "127.0.0.1" | "::1")
}

fn origin_hostname(origin: &str) -> Option<&str> {
    let authority = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .or_else(|| origin.strip_prefix("tauri://"))
        .or_else(|| origin.strip_prefix("http://tauri.localhost"))?;
    Some(hostname(authority))
}

/// Reject DNS-rebinding hosts and cross-origin mutations.
fn check_host_and_origin(state: &AppState, request: &Request) -> Result<(), ApiError> {
    let host =
        request.headers().get(header::HOST).and_then(|v| v.to_str().ok()).unwrap_or("127.0.0.1");

    if !state.allow_remote && !is_loopback_host(host) {
        return Err(ApiError::forbidden("Refusing a non-loopback Host header"));
    }

    if let Some(origin) = request.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        let Some(origin_host) = origin_hostname(origin) else {
            return Err(ApiError::forbidden("Invalid Origin header"));
        };
        if !state.allow_remote {
            if !is_loopback_host(origin_host) {
                return Err(ApiError::forbidden("Cross-origin requests are not allowed"));
            }
        } else if origin_host != hostname(host) {
            return Err(ApiError::forbidden("Origin does not match Host"));
        }
    }

    Ok(())
}

/// Middleware for every `/api` route except the health probe.
pub async fn require_token(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    check_host_and_origin(&state, &request)?;
    if !has_valid_token(request.headers(), &state.token) {
        return Err(ApiError::unauthorized());
    }
    Ok(next.run(request).await)
}

/// Add a long-lived HttpOnly cookie to the SPA entry response.
#[must_use]
pub fn token_cookie(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "twatch_token={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=31536000"
    ))
    .unwrap_or_else(|_| HeaderValue::from_static("twatch_token=invalid; Path=/"))
}

/// Standard browser hardening headers.
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; img-src 'self' data: blob: https:; media-src 'self' blob:; \
             style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' ws: wss:; \
             font-src 'self' data:; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
        ),
    );
    response
}

/// Helper for handlers that need the cookie after an index response.
pub fn with_cookie(mut response: Response, token: &str) -> Response {
    response.headers_mut().append(header::SET_COOKIE, token_cookie(token));
    response
}

/// Used by tests and the SPA bootstrap to tell whether a request is authenticated.
#[must_use]
pub fn is_authenticated(state: &AppState, headers: &header::HeaderMap) -> bool {
    has_valid_token(headers, &state.token)
}

/// Convert `ApiError` into a response without losing the `IntoResponse` import
/// in callers that only need a type.
pub fn error_response(error: ApiError) -> Response {
    error.into_response()
}
