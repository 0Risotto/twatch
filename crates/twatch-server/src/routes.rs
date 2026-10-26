//! REST API routes.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

use twatch_core::domain::history::HistoryEntry;
use twatch_core::domain::search::{FieldMap, SearchConfig};
use twatch_core::gateway::{
    BootstrapSnapshot, ConfigView, DeleteReport, DownloadSnapshot, PlayerSnapshot, SessionSnapshot,
};
use twatch_core::theme::{self, ThemeSpec};
use twatch_engine::covers::{Cover, CoverQuery};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct SelectFilesRequest {
    pub indices: Vec<usize>,
    pub selected: bool,
}

#[derive(Debug, Deserialize)]
pub struct WatchRequest {
    pub index: usize,
}

#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub indices: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RenameFileRequest {
    pub index: usize,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteFilesRequest {
    pub indices: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryRenameRequest {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryUrlRequest {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(default)]
    pub page: u32,
}

#[derive(Debug, Deserialize)]
pub struct PlayerOpenRequest {
    pub download_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct ConfigPatch {
    pub theme: Option<String>,
    pub player: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryDto {
    pub url: String,
    pub custom_name: Option<String>,
    pub torrent_name: String,
    pub added_at: u64,
    pub watched: bool,
    pub watched_files: Vec<String>,
    pub downloaded: bool,
    pub downloaded_files: Vec<String>,
}

impl From<HistoryEntry> for HistoryEntryDto {
    fn from(entry: HistoryEntry) -> Self {
        Self {
            url: entry.url,
            custom_name: entry.custom_name,
            torrent_name: entry.torrent_name,
            added_at: entry.added_at,
            watched: entry.watched,
            watched_files: entry.watched_files,
            downloaded: entry.downloaded,
            downloaded_files: entry.downloaded_files,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldMapDto {
    pub name: String,
    pub info_hash: String,
    pub seeders: String,
    pub leechers: String,
    pub size: String,
    pub username: String,
}

impl From<FieldMap> for FieldMapDto {
    fn from(field_map: FieldMap) -> Self {
        Self {
            name: field_map.name,
            info_hash: field_map.info_hash,
            seeders: field_map.seeders,
            leechers: field_map.leechers,
            size: field_map.size,
            username: field_map.username,
        }
    }
}

impl From<FieldMapDto> for FieldMap {
    fn from(field_map: FieldMapDto) -> Self {
        Self {
            name: field_map.name,
            info_hash: field_map.info_hash,
            seeders: field_map.seeders,
            leechers: field_map.leechers,
            size: field_map.size,
            username: field_map.username,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchConfigDto {
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub field_map: FieldMapDto,
}

impl From<SearchConfig> for SearchConfigDto {
    fn from(config: SearchConfig) -> Self {
        Self { url: config.url, headers: config.headers, field_map: config.field_map.into() }
    }
}

impl From<SearchConfigDto> for SearchConfig {
    fn from(config: SearchConfigDto) -> Self {
        Self { url: config.url, headers: config.headers, field_map: config.field_map.into() }
    }
}

#[derive(Debug, Deserialize)]
pub struct CoverBatchRequest {
    pub items: Vec<CoverItem>,
}

#[derive(Debug, Deserialize)]
pub struct CoverItem {
    pub title: String,
    #[serde(default)]
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct Health {
    pub ok: bool,
    pub version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct OkResponse {
    pub ok: bool,
}

pub async fn covers(
    State(state): State<AppState>,
    Json(request): Json<CoverBatchRequest>,
) -> Json<Vec<Cover>> {
    let queries: Vec<CoverQuery> = request
        .items
        .into_iter()
        .take(60)
        .map(|item| CoverQuery { title: item.title, kind: item.kind })
        .collect();
    Json(state.covers.resolve_many(&queries).await)
}

pub async fn health() -> Json<Health> {
    Json(Health { ok: true, version: env!("CARGO_PKG_VERSION") })
}

pub async fn bootstrap(State(state): State<AppState>) -> Json<BootstrapSnapshot> {
    Json(state.gateway.bootstrap())
}

pub async fn session(State(state): State<AppState>) -> Json<Option<SessionSnapshot>> {
    Json(state.gateway.session())
}

pub async fn preview(
    State(state): State<AppState>,
    Json(request): Json<PreviewRequest>,
) -> Result<Json<SessionSnapshot>, ApiError> {
    let url = request.url.trim();
    if url.is_empty() {
        return Err(ApiError::bad_request("A magnet link or torrent URL is required"));
    }
    Ok(Json(state.gateway.preview(url).await?))
}

pub async fn close_session(State(state): State<AppState>) -> Json<OkResponse> {
    state.gateway.close_session();
    Json(OkResponse { ok: true })
}

pub async fn select_files(
    State(state): State<AppState>,
    Json(request): Json<SelectFilesRequest>,
) -> Result<Json<SessionSnapshot>, ApiError> {
    state.gateway.select_files(&request.indices, request.selected)?;
    Ok(Json(state.gateway.session().ok_or_else(|| ApiError::bad_request("No active session"))?))
}

pub async fn watch_file(
    State(state): State<AppState>,
    Json(request): Json<WatchRequest>,
) -> Result<Json<DownloadSnapshot>, ApiError> {
    Ok(Json(state.gateway.watch_file(request.index)?))
}

pub async fn download_files(
    State(state): State<AppState>,
    Json(request): Json<DownloadRequest>,
) -> Result<Json<DownloadSnapshot>, ApiError> {
    Ok(Json(state.gateway.download_files(&request.indices)?))
}

pub async fn rename_file(
    State(state): State<AppState>,
    Json(request): Json<RenameFileRequest>,
) -> Result<Json<SessionSnapshot>, ApiError> {
    state.gateway.rename_file(request.index, &request.name)?;
    Ok(Json(state.gateway.session().ok_or_else(|| ApiError::bad_request("No active session"))?))
}

pub async fn delete_files(
    State(state): State<AppState>,
    Json(request): Json<DeleteFilesRequest>,
) -> Json<DeleteReport> {
    Json(state.gateway.delete_files(&request.indices))
}

pub async fn downloads(State(state): State<AppState>) -> Json<Vec<DownloadSnapshot>> {
    Json(state.gateway.downloads())
}

pub async fn pause_download(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.pause_download(id).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn resume_download(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.resume_download(id).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn remove_download(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.remove_download(id).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<twatch_core::domain::search::SearchResult>>, ApiError> {
    if params.q.trim().is_empty() {
        return Ok(Json(Vec::new()));
    }
    Ok(Json(state.gateway.search(params.q.trim(), params.page).await?))
}

pub async fn get_search_config(State(state): State<AppState>) -> Json<SearchConfigDto> {
    Json(state.gateway.search_config().into())
}

pub async fn put_search_config(
    State(state): State<AppState>,
    Json(config): Json<SearchConfigDto>,
) -> Result<Json<SearchConfigDto>, ApiError> {
    let config: SearchConfig = config.into();
    state.gateway.update_search_config(config.clone())?;
    Ok(Json(config.into()))
}

pub async fn history(State(state): State<AppState>) -> Json<Vec<HistoryEntryDto>> {
    Json(state.gateway.history().into_iter().map(HistoryEntryDto::from).collect())
}

pub async fn rename_history(
    State(state): State<AppState>,
    Json(request): Json<HistoryRenameRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.rename_history(&request.url, &request.name)?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn delete_history(
    State(state): State<AppState>,
    Json(request): Json<HistoryUrlRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state.gateway.delete_history(&request.url)?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn open_history(
    State(state): State<AppState>,
    Json(request): Json<HistoryUrlRequest>,
) -> Result<Json<SessionSnapshot>, ApiError> {
    Ok(Json(state.gateway.open_history(&request.url).await?))
}

pub async fn player(State(state): State<AppState>) -> Json<PlayerSnapshot> {
    Json(state.gateway.player())
}

pub async fn open_player(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlayerOpenRequest>,
) -> Result<Json<PlayerSnapshot>, ApiError> {
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1:7878");
    let scheme =
        headers.get("x-forwarded-proto").and_then(|value| value.to_str().ok()).unwrap_or("http");
    let base_url = format!("{scheme}://{host}");
    Ok(Json(state.gateway.open_player(request.download_id, &base_url)?))
}

pub async fn stop_player(State(state): State<AppState>) -> Json<OkResponse> {
    state.gateway.stop_player();
    Json(OkResponse { ok: true })
}

pub async fn themes() -> Json<Vec<ThemeSpec>> {
    Json(theme::THEMES.to_vec())
}

pub async fn config(State(state): State<AppState>) -> Json<ConfigView> {
    Json(state.gateway.config_view())
}

pub async fn patch_config(
    State(state): State<AppState>,
    Json(request): Json<ConfigPatch>,
) -> Result<Json<ConfigView>, ApiError> {
    if let Some(theme) = request.theme {
        state.gateway.update_theme(&theme)?;
    }
    if let Some(player) = request.player {
        state.gateway.update_player(&player)?;
    }
    Ok(Json(state.gateway.config_view()))
}
