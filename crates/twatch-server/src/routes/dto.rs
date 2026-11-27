use serde::{Deserialize, Serialize};
use ts_rs::TS;

use twatch_core::domain::history::{HistoryEntry, PlaybackProgress};
use twatch_core::domain::search::{FieldMap, SearchConfig};

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
pub struct ProgressRequest {
    pub url: String,
    pub file: String,
    pub position: f64,
    pub duration: f64,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(default)]
    pub page: u32,
}

#[derive(Debug, Deserialize)]
pub struct ConfigPatch {
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryEntryDto {
    pub url: String,
    pub custom_name: Option<String>,
    pub torrent_name: String,
    #[ts(type = "number")]
    pub added_at: u64,
    pub watched: bool,
    pub watched_files: Vec<String>,
    pub downloaded: bool,
    pub downloaded_files: Vec<String>,
    pub progress: std::collections::BTreeMap<String, PlaybackProgress>,
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
            progress: entry.progress,
        }
    }
}

fn default_category_field() -> String {
    "category".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FieldMapDto {
    pub name: String,
    pub info_hash: String,
    pub seeders: String,
    pub leechers: String,
    pub size: String,
    pub username: String,
    #[serde(default = "default_category_field")]
    pub category: String,
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
            category: field_map.category,
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
            category: field_map.category,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchConfigDto {
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub field_map: FieldMapDto,
    #[serde(default)]
    pub blocked_categories: Vec<String>,
}

impl From<SearchConfig> for SearchConfigDto {
    fn from(config: SearchConfig) -> Self {
        Self {
            url: config.url,
            headers: config.headers,
            field_map: config.field_map.into(),
            blocked_categories: config.blocked_categories,
        }
    }
}

impl From<SearchConfigDto> for SearchConfig {
    fn from(config: SearchConfigDto) -> Self {
        Self {
            url: config.url,
            headers: config.headers,
            field_map: config.field_map.into(),
            blocked_categories: config.blocked_categories,
        }
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
