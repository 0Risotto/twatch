//! Persistent torrent history domain.

pub mod ports;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Resume position for one file inside a torrent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackProgress {
    pub position: f64,
    pub duration: f64,
    pub updated_at: u64,
}

impl PlaybackProgress {
    /// Fraction of the file that has been played, clamped to `0..=1`.
    #[must_use]
    pub fn fraction(&self) -> f64 {
        if self.duration <= 0.0 { 0.0 } else { (self.position / self.duration).clamp(0.0, 1.0) }
    }
}

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
    /// Resume positions keyed by the file name inside the torrent.
    #[serde(default)]
    pub progress: BTreeMap<String, PlaybackProgress>,
}

/// Return the user-friendly name for a history entry.
///
/// Prefers `custom_name`; falls back to the original torrent name.
#[must_use]
pub fn display_name(entry: &HistoryEntry) -> &str {
    entry.custom_name.as_deref().unwrap_or(&entry.torrent_name)
}
