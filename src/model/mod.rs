//! Domain data types for the twatch application.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TorrentId(pub usize);

#[derive(Debug, Clone)]
pub struct TorrentFile {
    pub index: usize,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct TorrentInfo {
    pub name: String,
    pub files: Vec<TorrentFile>,
}

#[derive(Debug, Clone)]
pub struct TorrentStats {
    pub progress: f64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub peers: usize,
    pub total_size: u64,
    pub downloaded: u64,
}

/// An actively downloading or streaming torrent shown in the sidebar.
#[derive(Debug, Clone)]
pub struct ActiveDownload {
    pub torrent_id: TorrentId,
    pub torrent_name: String,
    pub file_name: String,
    pub progress: f64,
    pub download_speed: u64,
    pub total_size: u64,
    pub downloaded: u64,
    pub is_streaming: bool,
}

/// A single entry in the collapsible folder-tree browser list.
#[derive(Debug, Clone)]
pub enum DisplayEntry {
    /// A collapsible subfolder.
    Folder { name: String, depth: usize, expanded: bool },
    /// A regular file from the torrent.
    File { file: TorrentFile, depth: usize },
}

/// A single entry in the torrent history list, persisted as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub custom_name: Option<String>,
    pub torrent_name: String,
    pub added_at: u64,
}

/// Return the user-friendly name for a history entry.
///
/// Prefers `custom_name`; falls back to the original torrent name.
#[must_use]
pub fn display_name(entry: &HistoryEntry) -> &str {
    entry.custom_name.as_deref().unwrap_or(&entry.torrent_name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Input,
    Loading,
    Browser,
    Player,
    History,
}

#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub value: String,
    pub cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry {
    Add,
    History,
    Quit,
}

impl MenuEntry {
    pub const ALL: [MenuEntry; 3] = [MenuEntry::Add, MenuEntry::History, MenuEntry::Quit];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            MenuEntry::Add => "[a] Add a torrent",
            MenuEntry::History => "[h] History",
            MenuEntry::Quit => "[q] Quit",
        }
    }
}

/// Validate a user-supplied torrent URL / magnet link.
///
/// Returns `Err(message)` if the input is empty or does not look like
/// a recognised torrent identifier.
pub fn validate_torrent_input(input: &str) -> Result<(), &'static str> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("URL cannot be empty");
    }
    if trimmed.starts_with("magnet:") {
        return Ok(());
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(());
    }
    Err("Must be a magnet link or an http(s) URL")
}

/// Subsequence fuzzy match.  Returns `true` when every character in
/// `query` appears in `target` in order (case-insensitive).
#[must_use]
pub fn fuzzy_match(query: &str, target: &str) -> bool {
    let query_lower = query.to_lowercase();
    let mut qchars = query_lower.chars().peekable();
    let target_lower = target.to_lowercase();
    for c in target_lower.chars() {
        if qchars.peek() == Some(&c) {
            qchars.next();
        }
    }
    qchars.peek().is_none()
}
