//! Persistent torrent history domain.

pub mod ports;

use serde::{Deserialize, Serialize};

/// A single entry in the torrent history list, persisted as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub custom_name: Option<String>,
    pub torrent_name: String,
    pub added_at: u64,
    #[serde(default)]
    pub watched: bool,
    #[serde(default)]
    pub watched_files: Vec<String>,
    #[serde(default)]
    pub downloaded: bool,
    #[serde(default)]
    pub downloaded_files: Vec<String>,
}

/// Return the user-friendly name for a history entry.
///
/// Prefers `custom_name`; falls back to the original torrent name.
#[must_use]
pub fn display_name(entry: &HistoryEntry) -> &str {
    entry.custom_name.as_deref().unwrap_or(&entry.torrent_name)
}
