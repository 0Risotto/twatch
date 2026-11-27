//! Serializable snapshots exposed by the gateway.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::theme::ThemeSpec;

/// Lifecycle state shown in the downloads UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DownloadState {
    Starting,
    Downloading,
    Streaming,
    Paused,
    Completed,
    Failed,
}

/// A single file in the current torrent session.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FileSnapshot {
    pub index: usize,
    pub name: String,
    #[ts(type = "number")]
    pub size: u64,
    pub selected: bool,
    pub watched: bool,
    pub downloaded: bool,
    pub downloading: bool,
}

/// The currently browsed torrent.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionSnapshot {
    pub url: String,
    pub name: String,
    pub files: Vec<FileSnapshot>,
    pub selected_count: usize,
    #[ts(type = "number")]
    pub selected_size: u64,
    #[ts(type = "number")]
    pub total_size: u64,
    pub download_root: Option<String>,
}

/// One subtitle file inside the torrent, addressable by its file index.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubtitleTrack {
    pub index: usize,
    pub name: String,
}

/// One playable file in the current torrent, used to build a next/previous queue.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct QueueFile {
    pub index: usize,
    pub name: String,
    #[ts(type = "number")]
    pub size: u64,
}

/// Which local root owns a library file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LibraryRoot {
    Cache,
    Downloads,
}

impl LibraryRoot {
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Cache => "cache",
            Self::Downloads => "downloads",
        }
    }
}

/// The kind of media the library found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LibraryMediaKind {
    Video,
    Audio,
}

/// A playable file discovered under the local cache or downloads folder.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LibraryEntry {
    pub id: String,
    pub name: String,
    pub relative_path: String,
    pub root: LibraryRoot,
    pub media_kind: LibraryMediaKind,
    #[ts(type = "number")]
    pub size: u64,
    #[ts(type = "number | null")]
    pub modified_ms: Option<u64>,
    pub stream_path: String,
}

/// A live or recently completed download.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DownloadSnapshot {
    #[ts(type = "number")]
    pub id: u64,
    pub torrent_id: Option<usize>,
    pub torrent_name: String,
    pub label: String,
    pub file_names: Vec<String>,
    pub url: String,
    pub progress: f64,
    #[ts(type = "number")]
    pub speed: u64,
    #[ts(type = "number")]
    pub upload_speed: u64,
    pub peers: usize,
    #[ts(type = "number")]
    pub total_size: u64,
    #[ts(type = "number")]
    pub downloaded: u64,
    #[ts(type = "number | null")]
    pub eta_seconds: Option<u64>,
    pub is_streaming: bool,
    pub state: DownloadState,
    pub stream_path: Option<String>,
    pub subtitles: Vec<SubtitleTrack>,
    pub error: Option<String>,
    /// Index of this file inside the torrent (streams only).
    pub file_index: usize,
    /// Playable files in the torrent, in torrent order (streams only).
    pub queue: Vec<QueueFile>,
    /// Saved resume position for this file, if any.
    pub resume_position: Option<f64>,
}

/// Config fields that are safe to send to the browser.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConfigView {
    pub theme: String,
    pub download_dir: String,
}

/// Everything the SPA needs for its first paint.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BootstrapSnapshot {
    pub version: String,
    pub config: ConfigView,
    pub session: Option<SessionSnapshot>,
    pub downloads: Vec<DownloadSnapshot>,
    pub themes: Vec<ThemeSpec>,
    pub search_configured: bool,
}

/// A result of the safe delete operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeleteReport {
    pub deleted: usize,
    pub failed: Vec<String>,
}

/// Severity for a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}
