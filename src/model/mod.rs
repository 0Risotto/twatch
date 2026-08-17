//! Application session types and re-exports of domain types.
//!
//! Domain types are re-exported from [`crate::domain`]; screen and
//! session state live in [`crate::app`].

pub use crate::domain::common::{fuzzy_match, validate_torrent_input};
pub use crate::domain::history::{HistoryEntry, display_name};
pub use crate::domain::search::{SEARCH_PAGE_SIZE, SearchResult};
pub use crate::domain::torrent::file_tree::entry_matches;
pub use crate::domain::torrent::{
    DisplayEntry, FileStatus, FileTree, TorrentFile, TorrentId, TorrentInfo, TorrentStats,
};

/// An actively downloading or streaming torrent shown in the sidebar.
#[derive(Debug, Clone)]
pub struct ActiveDownload {
    pub torrent_id: TorrentId,
    pub torrent_name: String,
    /// Human label for the sidebar (file name, or "N files").
    pub file_name: String,
    /// Every file covered by this download.
    pub file_names: Vec<String>,
    pub url: String,
    pub progress: f64,
    pub download_speed: u64,
    pub total_size: u64,
    pub downloaded: u64,
    pub is_streaming: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry {
    Search,
    Add,
    History,
    Quit,
}

impl MenuEntry {
    pub const ALL: [MenuEntry; 4] =
        [MenuEntry::Add, MenuEntry::Search, MenuEntry::History, MenuEntry::Quit];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            MenuEntry::Search => "[s] Search torrents",
            MenuEntry::Add => "[a] Add a torrent",
            MenuEntry::History => "[h] History",
            MenuEntry::Quit => "[q] Quit",
        }
    }
}
