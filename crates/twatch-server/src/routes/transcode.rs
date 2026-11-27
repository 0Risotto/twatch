use std::hash::{Hash, Hasher};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::error::ApiError;
use crate::state::AppState;
use crate::transcode::TranscodeService;

fn base_url(headers: &HeaderMap) -> String {
    let host =
        headers.get(header::HOST).and_then(|value| value.to_str().ok()).unwrap_or("127.0.0.1:7878");
    let scheme =
        headers.get("x-forwarded-proto").and_then(|value| value.to_str().ok()).unwrap_or("http");
    format!("{scheme}://{host}")
}

fn source_url(base_url: &str, path: &str, token: &str) -> String {
    let mut url = format!("{}{}", base_url.trim_end_matches('/'), path);
    url.push_str(if path.contains('?') { "&token=" } else { "?token=" });
    url.push_str(token);
    url
}

fn library_key(id: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    format!("library-{:016x}", hasher.finish())
}

async fn read_file(
    transcode: &TranscodeService,
    key: &str,
    file: &str,
) -> Result<Response, ApiError> {
    let Some(bytes) = transcode.read(key, file).await else {
        return Err(ApiError::not_found("Transcode file not found"));
    };
    let content_type =
        if file.ends_with(".m3u8") { "application/vnd.apple.mpegurl" } else { "video/mp2t" };
    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
        (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
    ];
    Ok((StatusCode::OK, headers, bytes).into_response())
}

pub async fn download_file(
    State(state): State<AppState>,
    Path((id, file)): Path<(u64, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let key = format!("download-{id}");
    if file == "index.m3u8" {
        let stream_path = state
            .gateway
            .downloads()
            .into_iter()
            .find(|download| download.id == id)
            .and_then(|download| download.stream_path)
            .ok_or_else(|| ApiError::new(StatusCode::CONFLICT, "Stream is not ready"))?;
        let input = source_url(&base_url(&headers), &stream_path, &state.token);
        state.transcode.ensure(&key, &input).await.map_err(|error| {
            tracing::warn!("Transcode start failed: {error}");
            ApiError::new(StatusCode::SERVICE_UNAVAILABLE, error)
        })?;
    }
    read_file(&state.transcode, &key, &file).await
}

pub async fn library_file(
    State(state): State<AppState>,
    Path((id, file)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let key = library_key(&id);
    if file == "index.m3u8" {
        let stream_path = state
            .gateway
            .library()
            .into_iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.stream_path)
            .ok_or_else(|| ApiError::not_found("Library file not found"))?;
        let input = source_url(&base_url(&headers), &stream_path, &state.token);
        state.transcode.ensure(&key, &input).await.map_err(|error| {
            tracing::warn!("Library transcode start failed: {error}");
            ApiError::new(StatusCode::SERVICE_UNAVAILABLE, error)
        })?;
    }
    read_file(&state.transcode, &key, &file).await
}

pub async fn stop(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Json<super::dto::OkResponse> {
    state.transcode.stop(&format!("download-{id}")).await;
    Json(super::dto::OkResponse { ok: true })
}
