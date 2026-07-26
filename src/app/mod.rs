//! Application state, screens, and main event loop.

use crate::model::{
    ActiveDownload, DisplayEntry, InputState, Screen, SearchResult, TorrentFile, TorrentId,
    TorrentInfo, TorrentStats,
};
use crate::traits::{PlayerService, SearchService, StorageService, TorrentService};
use crate::ui::Theme;
use crate::{config::Config, module::AppModule, ui};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::CrosstermBackend};
use shaku::HasComponent;
use std::collections::HashSet;
use std::io::Stdout;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

pub mod handlers;

/// Internal messages sent from background tasks to the main loop.
enum AppEvent {
    PreviewReady { url: String, info: TorrentInfo },
    PreviewFailed(String),
    DownloadReady { id: TorrentId, stream_url: String, file_name: String, is_watch: bool },
    DownloadFailed(String),
    SearchResults(Vec<SearchResult>),
}

/// The top-level application state.
///
/// Owns the DI module and all UI-navigation fields.  Services are
/// retrieved from `module` through [`HasComponent::resolve`].
pub struct App {
    pub module: AppModule,
    pub config: Config,
    pub screen: Screen,
    pub input: InputState,
    pub files: Vec<TorrentFile>,
    pub selected_files: Vec<bool>,
    pub selected_file: usize,
    pub torrent_id: Option<TorrentId>,
    pub stats: Option<TorrentStats>,
    pub active_downloads: Vec<ActiveDownload>,
    pub torrent_name: Option<String>,
    pub expanded_paths: HashSet<String>,
    pub display_entries: Vec<DisplayEntry>,
    pub search_query: String,
    pub is_searching: bool,
    pub status_message: String,
    pub running: bool,
    pub history_selected: usize,
    pub history_orig_index: usize,
    pub menu_selected: usize,
    pub renaming: bool,
    pub rename_input: InputState,
    pub theme: Theme,
    pub theme_picker: bool,
    pub theme_picker_filter: String,
    pub theme_picker_selected: usize,
    pub theme_picker_scroll: usize,
    pub theme_picker_original: Theme,
    pub search_open: bool,
    pub torrent_search_query: String,
    pub search_selected: usize,
    pub search_busy: bool,
    pub search_page: u32,
    pub search_all_results: Vec<SearchResult>,
    pub search_fetched_query: String,
    pub search_config: crate::config::search::SearchConfig,
    pub search_config_open: bool,
    pub search_config_input: String,
    pub watched_files: Vec<String>,
    pub downloaded_files: Vec<String>,
    pub downloading_files: Vec<String>,
    pub confirm_delete: bool,
    pub confirm_delete_yes: bool,
    pending_url: Option<String>,
    error_message: Option<String>,
    task_busy: bool,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl App {
    /// Create a new application with the given DI module and configuration.
    ///
    /// Ensures `config.download_dir` exists on disk.
    pub fn new(module: AppModule, config: Config) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        std::fs::create_dir_all(&config.download_dir).ok();

        let theme = Theme::from_name(&config.theme).unwrap_or_default();

        let search_config = crate::config::search::SearchConfig::load(&config.config_dir);

        Ok(Self {
            module,
            config,
            screen: Screen::Welcome,
            input: InputState::default(),
            files: Vec::new(),
            selected_files: Vec::new(),
            selected_file: 0,
            torrent_id: None,
            stats: None,
            active_downloads: Vec::new(),
            torrent_name: None,
            expanded_paths: HashSet::new(),
            display_entries: Vec::new(),
            search_query: String::new(),
            is_searching: false,
            status_message: String::new(),
            running: true,
            history_selected: 0,
            history_orig_index: 0,
            menu_selected: 0,
            renaming: false,
            rename_input: InputState::default(),
            theme,
            theme_picker: false,
            theme_picker_filter: String::new(),
            theme_picker_selected: 0,
            theme_picker_scroll: 0,
            theme_picker_original: Theme::default(),
            search_open: false,
            torrent_search_query: String::new(),
            search_selected: 0,
            search_busy: false,
            search_page: 0,
            search_all_results: Vec::new(),
            search_fetched_query: String::new(),
            search_config,
            search_config_open: false,
            search_config_input: String::new(),
            watched_files: Vec::new(),
            downloaded_files: Vec::new(),
            downloading_files: Vec::new(),
            confirm_delete: false,
            confirm_delete_yes: false,
            pending_url: None,
            error_message: None,
            task_busy: false,
            event_rx,
            event_tx,
        })
    }

    /// Queue an error that will be shown on the next frame.
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
    }

    /// Spawn a background task that performs a search query.
    pub fn enqueue_search(&self, query: String, page: u32) {
        let searcher: Arc<dyn SearchService> = self.module.resolve();
        let mut config = self.search_config.clone();
        config.page = page;
        let tx = self.event_tx.clone();
        let torrent: Arc<dyn TorrentService> = self.module.resolve();
        torrent.spawn_boxed(Box::pin(async move {
            match searcher.search(&query, &config).await {
                Ok(results) => {
                    let _ = tx.send(AppEvent::SearchResults(results));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::SearchResults(Vec::new()));
                    tracing::warn!("Search failed: {e}");
                }
            }
        }));
    }

    /// The currently previewed URL, if any.
    #[must_use]
    pub fn pending_url(&self) -> Option<&str> {
        self.pending_url.as_deref()
    }

    pub fn set_pending_url(&mut self, url: String) {
        self.pending_url = Some(url);
    }

    pub fn clear_pending_url(&mut self) {
        self.pending_url = None;
    }

    /// Spawn a background task that fetches torrent metadata for `url`.
    ///
    /// Results are delivered via the internal mpsc channel and
    /// processed in the main event loop.
    pub fn enqueue_preview(&self, url: String) {
        let torrent: Arc<dyn TorrentService> = self.module.resolve();
        let t = torrent.clone();
        let tx = self.event_tx.clone();

        torrent.spawn_boxed(Box::pin(async move {
            match t.preview(&url).await {
                Ok(info) => {
                    let _ = tx.send(AppEvent::PreviewReady { url, info });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::PreviewFailed(format!("Failed: {e}")));
                }
            }
        }));
    }

    /// Spawn a background task that downloads a single file for
    /// streaming, then resolves the stream URL.
    pub fn enqueue_watch(&self, url: String, file_index: usize, file_name: String) {
        let torrent: Arc<dyn TorrentService> = self.module.resolve();
        let t = torrent.clone();
        let tx = self.event_tx.clone();

        torrent.spawn_boxed(Box::pin(async move {
            match t.download(&url, file_index).await {
                Ok(id) => {
                    let stream_url = t.get_stream_url(&id, file_index).unwrap_or_default();
                    let _ = tx.send(AppEvent::DownloadReady {
                        id,
                        stream_url,
                        file_name,
                        is_watch: true,
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::DownloadFailed(format!("Failed: {e}")));
                }
            }
        }));
    }

    /// Spawn a background task that downloads the selected files to
    /// `output_dir`.
    pub fn enqueue_download_batch(
        &self,
        url: String,
        indices: Vec<usize>,
        file_name: String,
        output_dir: std::path::PathBuf,
    ) {
        let torrent: Arc<dyn TorrentService> = self.module.resolve();
        let t = torrent.clone();
        let tx = self.event_tx.clone();

        torrent.spawn_boxed(Box::pin(async move {
            match t.download_to_folder(&url, &indices, &output_dir).await {
                Ok(id) => {
                    let _ = tx.send(AppEvent::DownloadReady {
                        id,
                        stream_url: String::new(),
                        file_name,
                        is_watch: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::DownloadFailed(format!("Failed: {e}")));
                }
            }
        }));
    }

    /// Number of currently toggled files.
    #[must_use]
    pub fn selected_count(&self) -> usize {
        self.selected_files.iter().filter(|&&s| s).count()
    }

    /// Indices of currently toggled files.
    #[must_use]
    pub fn selected_indices(&self) -> Vec<usize> {
        self.selected_files.iter().enumerate().filter(|(_, s)| **s).map(|(i, _)| i).collect()
    }

    /// Rebuild `display_entries` as a recursive tree from `self.files`.
    /// Only children of expanded folders are included.  Folders sort
    /// before files at each depth.
    pub fn rebuild_entries(&mut self) {
        let mut entries = Vec::new();
        rebuild_level(&self.files, "", 0, &self.expanded_paths, &mut entries, &mut HashSet::new());
        self.display_entries = entries;

        // Clamp cursor.
        self.clamp_selection();
    }

    /// Return the list of entries that pass the current search filter.
    #[must_use]
    pub fn visible_entries(&self) -> Vec<(usize, &DisplayEntry)> {
        if self.search_query.is_empty() {
            return self.display_entries.iter().enumerate().collect();
        }
        self.display_entries
            .iter()
            .enumerate()
            .filter(|(_, e)| entry_matches(&self.search_query, e))
            .collect()
    }

    fn clamp_selection(&mut self) {
        let visible = self.visible_entries();
        if visible.is_empty() {
            self.selected_file = 0;
            return;
        }
        if self.selected_file >= visible.len() {
            self.selected_file = visible.len() - 1;
        }
    }

    /// Cycle through available themes and persist the choice.
    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.config.theme = Theme::to_config_name(self.theme.name);
        let _ = self.config.save();
    }
}

// ── Tree helpers ──

fn rebuild_level(
    files: &[TorrentFile],
    prefix: &str,
    depth: usize,
    expanded: &HashSet<String>,
    entries: &mut Vec<DisplayEntry>,
    _seen_folders: &mut HashSet<String>,
) {
    let mut folders = std::collections::BTreeSet::new();
    let mut file_indices = Vec::new();

    for (i, file) in files.iter().enumerate() {
        if !file.name.starts_with(prefix) {
            continue;
        }
        let rest = &file.name[prefix.len()..];
        if let Some(slash_idx) = rest.find('/') {
            let folder_name = &rest[..slash_idx];
            folders.insert(folder_name.to_string());
        } else {
            file_indices.push(i);
        }
    }

    for folder_name in &folders {
        let full = format!("{prefix}{folder_name}/");
        let is_expanded = expanded.contains(&full);

        entries.push(DisplayEntry::Folder {
            name: folder_name.clone(),
            depth,
            expanded: is_expanded,
        });

        if is_expanded {
            rebuild_level(files, &full, depth + 1, expanded, entries, _seen_folders);
        }
    }

    for i in &file_indices {
        entries.push(DisplayEntry::File { file: files[*i].clone(), depth });
    }
}

/// True when `query` fuzzy-matches a display entry's visible text.
#[must_use]
pub fn entry_matches(query: &str, entry: &DisplayEntry) -> bool {
    match entry {
        DisplayEntry::Folder { name, .. } => crate::model::fuzzy_match(query, name),
        DisplayEntry::File { file, .. } => crate::model::fuzzy_match(query, &file.name),
    }
}

/// Run the main application loop.
///
/// Drives the `TUI` with ratatui-crossterm, drains the `mpsc` event
/// channel, and dispatches keyboard input to [`handlers::handle_key`].
pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    module: AppModule,
    config: Config,
) -> Result<()> {
    let mut app = App::new(module, config)?;

    while app.running {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        while let Ok(event) = app.event_rx.try_recv() {
            app.task_busy = false;
            match event {
                AppEvent::PreviewReady { url, info } => {
                    let storage: Arc<dyn StorageService> = app.module.resolve();
                    storage.add_entry(&url, &info.name);
                    app.watched_files = storage
                        .history()
                        .iter()
                        .find(|e| e.url == url)
                        .map(|e| e.watched_files.clone())
                        .unwrap_or_default();
                    app.downloaded_files = storage
                        .history()
                        .iter()
                        .find(|e| e.url == url)
                        .map(|e| e.downloaded_files.clone())
                        .unwrap_or_default();
                    let dl = app.config.download_dir.clone();
                    app.watched_files.retain(|f| dl.join(f).exists());
                    app.downloaded_files.retain(|f| dl.join(f).exists());
                    app.set_pending_url(url);
                    app.torrent_name = Some(info.name.clone());
                    app.files = info.files.clone();
                    app.selected_files = vec![false; info.files.len()];
                    app.selected_file = 0;
                    app.expanded_paths.clear();
                    app.rebuild_entries();
                    app.screen = Screen::Browser;
                    app.status_message =
                        format!("{} file(s) found — w: watch, d: download", app.files.len());
                }
                AppEvent::PreviewFailed(msg) => {
                    app.set_error(msg);
                    app.screen = Screen::Input;
                }
                AppEvent::DownloadReady { id, stream_url, file_name, is_watch } => {
                    let download_url = app.pending_url().map(|s| s.to_string()).unwrap_or_default();
                    app.torrent_id = Some(id.clone());
                    app.active_downloads.push(ActiveDownload {
                        torrent_id: id.clone(),
                        torrent_name: app.torrent_name.clone().unwrap_or_default(),
                        file_name: file_name.clone(),
                        url: download_url,
                        progress: 0.0,
                        download_speed: 0,
                        total_size: 0,
                        downloaded: 0,
                        is_streaming: is_watch,
                    });
                    if is_watch && !stream_url.is_empty() {
                        let storage: Arc<dyn StorageService> = app.module.resolve();
                        if let Some(url) = app.pending_url() {
                            storage.mark_watched(url, &file_name);
                        }
                        if !app.watched_files.iter().any(|f| f == &file_name) {
                            app.watched_files.push(file_name.clone());
                        }
                        let player: Arc<dyn PlayerService> = app.module.resolve();
                        player.play(&stream_url, &file_name);
                        app.status_message = format!("Watching: {}", file_name);
                        app.screen = Screen::Player;
                    } else {
                        if !app.downloading_files.iter().any(|f| f == &file_name) {
                            app.downloading_files.push(file_name.clone());
                        }
                        app.status_message =
                            format!("Downloading to: {}", app.config.download_dir.display());
                        app.screen = Screen::Browser;
                    }
                }
                AppEvent::DownloadFailed(msg) => {
                    app.set_error(msg);
                    app.screen = Screen::Browser;
                }
                AppEvent::SearchResults(results) => {
                    app.search_busy = false;
                    app.search_all_results = results;
                    app.search_selected = 0;
                    app.search_page = 0;
                    app.status_message =
                        format!("{} result(s) found", app.search_all_results.len());
                }
            }
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handlers::handle_key(&mut app, key);
                }
                _ => {}
            }
        }

        // Poll stats for every active download.
        {
            let torrent: Arc<dyn TorrentService> = app.module.resolve();
            for dl in &mut app.active_downloads {
                if dl.progress >= 1.0 {
                    continue;
                }
                let before = dl.progress;
                if let Ok(stats) = torrent.get_stats(&dl.torrent_id) {
                    dl.progress = stats.progress;
                    dl.download_speed = stats.download_speed;
                    dl.total_size = stats.total_size;
                    dl.downloaded = stats.downloaded;
                }
                if before < 1.0 && dl.progress >= 1.0 && !dl.is_streaming {
                    let storage: Arc<dyn StorageService> = app.module.resolve();
                    storage.mark_downloaded(&dl.url, &dl.file_name);
                    app.downloading_files.retain(|f| f != &dl.file_name);
                    if !app.downloaded_files.iter().any(|f| f == &dl.file_name) {
                        app.downloaded_files.push(dl.file_name.clone());
                    }
                    app.status_message = format!("Download complete: {}", dl.file_name);
                }
            }
            // Also keep the single-torrent stats for the Player screen.
            if let Some(id) = &app.torrent_id {
                if let Ok(stats) = torrent.get_stats(id) {
                    app.stats = Some(stats);
                }
            }
        }

        app.active_downloads.retain(|dl| dl.is_streaming || dl.progress < 1.0);
        if app.active_downloads.is_empty() {
            app.torrent_id = None;
        }

        if let Some(msg) = app.error_message.take() {
            app.status_message = msg;
        }
    }

    Ok(())
}
