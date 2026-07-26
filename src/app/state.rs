//! Focused sub-state structs used by [`super::App`].

use crate::config::search::SearchConfig;
use crate::model::{DisplayEntry, SearchResult, TorrentFile};
use crate::ui::Theme;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct BrowserState {
    pub files: Vec<TorrentFile>,
    pub selected_files: Vec<bool>,
    pub selected_file: usize,
    pub torrent_name: Option<String>,
    pub expanded_paths: HashSet<String>,
    pub display_entries: Vec<DisplayEntry>,
    pub watched_files: Vec<String>,
    pub downloaded_files: Vec<String>,
    pub downloading_files: Vec<String>,
    pub confirm_delete: bool,
    pub confirm_delete_yes: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ThemeState {
    pub theme: Theme,
    pub picker: bool,
    pub picker_filter: String,
    pub picker_selected: usize,
    pub picker_scroll: usize,
    pub picker_original: Theme,
}

#[derive(Debug, Clone, Default)]
pub struct SearchPopupState {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub busy: bool,
    pub page: u32,
    pub all_results: Vec<SearchResult>,
    pub fetched_query: String,
    pub config: SearchConfig,
    pub config_open: bool,
    pub config_input: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HistoryState {
    pub selected: usize,
    pub orig_index: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WelcomeState {
    pub menu_selected: usize,
}
