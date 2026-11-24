//! API error envelope.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use twatch_core::gateway::GatewayError;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    #[must_use]
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }

    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    #[must_use]
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "A valid twatch token is required")
    }

    #[must_use]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }
}

impl From<GatewayError> for ApiError {
    fn from(error: GatewayError) -> Self {
        let status = match &error {
            GatewayError::NoSession
            | GatewayError::NoFilesSelected
            | GatewayError::NoMatchingFiles
            | GatewayError::UnknownTheme(_)
            | GatewayError::UnknownPlayer(_)
            | GatewayError::InvalidFileNameLength
            | GatewayError::InvalidFileNameCharacters
            | GatewayError::InvalidFileName => StatusCode::BAD_REQUEST,
            GatewayError::FileNotFound
            | GatewayError::DownloadNotFound
            | GatewayError::HistoryNotFound => StatusCode::NOT_FOUND,
            GatewayError::StreamNotReady => StatusCode::CONFLICT,
            GatewayError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self { status, message: error.to_string() }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        tracing::warn!("API error: {error:#}");
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: error.to_string() }
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: ErrorInner<'a>,
}

#[derive(Serialize)]
struct ErrorInner<'a> {
    code: u16,
    message: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body =
            ErrorBody { error: ErrorInner { code: self.status.as_u16(), message: &self.message } };
        (self.status, Json(body)).into_response()
    }
}
