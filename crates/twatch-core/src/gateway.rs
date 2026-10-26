//! Headless application gateway shared by `twatchd` and, eventually, the TUI.
//!
//! The gateway owns the current browse session, the live download registry, the
//! player state, and the event stream.  It has no HTTP or terminal types in it;
//! `twatch-server` turns these snapshots into JSON and WebSocket frames.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};

use crate::config::Config;
use crate::domain::common::spawner::TaskSpawner;
use crate::domain::history::HistoryEntry;
use crate::domain::history::ports::HistoryPort;
use crate::domain::player::ports::PlayerPort;
use crate::domain::search::ports::SearchPort;
use crate::domain::search::{SearchConfig, SearchResult};
use crate::domain::torrent::ports::TorrentPort;
use crate::domain::torrent::{TorrentFile, TorrentId, TorrentStats};
use crate::theme::{self, ThemeSpec};

/// Lifecycle state shown in the downloads UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DownloadState {
    Starting,
    Downloading,
    Streaming,
    Paused,
    Completed,
    Failed,
}

/// A single file in the current torrent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSnapshot {
    pub index: usize,
    pub name: String,
    pub size: u64,
    pub selected: bool,
    pub watched: bool,
    pub downloaded: bool,
    pub downloading: bool,
}

/// The currently browsed torrent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub url: String,
    pub name: String,
    pub files: Vec<FileSnapshot>,
    pub selected_count: usize,
    pub selected_size: u64,
    pub total_size: u64,
    pub download_root: Option<String>,
}

/// One subtitle file inside the torrent, addressable by its file index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTrack {
    pub index: usize,
    pub name: String,
}

/// A live or recently completed download.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSnapshot {
    pub id: u64,
    pub torrent_id: Option<usize>,
    pub torrent_name: String,
    pub label: String,
    pub file_names: Vec<String>,
    pub url: String,
    pub progress: f64,
    pub speed: u64,
    pub upload_speed: u64,
    pub peers: usize,
    pub total_size: u64,
    pub downloaded: u64,
    pub eta_seconds: Option<u64>,
    pub is_streaming: bool,
    pub state: DownloadState,
    pub stream_path: Option<String>,
    pub subtitles: Vec<SubtitleTrack>,
    pub error: Option<String>,
}

/// What the external player is doing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSnapshot {
    pub running: bool,
    pub title: Option<String>,
    pub stream_path: Option<String>,
    pub download_id: Option<u64>,
}

/// Config fields that are safe to send to the browser.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigView {
    pub theme: String,
    pub download_dir: String,
    pub player: String,
}

/// Everything the SPA needs for its first paint.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapSnapshot {
    pub version: String,
    pub config: ConfigView,
    pub session: Option<SessionSnapshot>,
    pub downloads: Vec<DownloadSnapshot>,
    pub player: PlayerSnapshot,
    pub themes: Vec<ThemeSpec>,
    pub search_configured: bool,
}

/// A result of the safe delete operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReport {
    pub deleted: usize,
    pub failed: Vec<String>,
}

/// Severity for a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Events the WebSocket layer forwards to connected clients.
#[derive(Debug, Clone)]
pub enum GatewayEvent {
    SessionChanged,
    DownloadsChanged,
    PlayerChanged,
    Toast { level: ToastLevel, message: String },
}

/// Internal completion messages from spawned download tasks.
enum InternalEvent {
    Ready { id: u64, torrent_id: TorrentId, stream_path: Option<String> },
    Failed { id: u64, message: String },
}

struct SessionState {
    url: String,
    name: String,
    files: Vec<TorrentFile>,
    selected: Vec<bool>,
    watched_files: Vec<String>,
    downloaded_files: Vec<String>,
    downloading_files: Vec<String>,
}

impl SessionState {
    fn new(
        url: String,
        name: String,
        files: Vec<TorrentFile>,
        watched_files: Vec<String>,
        downloaded_files: Vec<String>,
    ) -> Self {
        let selected = vec![false; files.len()];
        Self {
            url,
            name,
            files,
            selected,
            watched_files,
            downloaded_files,
            downloading_files: Vec::new(),
        }
    }

    fn download_root(&self) -> Option<&str> {
        if self.files.len() > 1 { Some(&self.name) } else { None }
    }

    fn download_root_dir(&self, download_dir: &Path) -> PathBuf {
        match self.download_root() {
            Some(root) => download_dir.join(root),
            None => download_dir.to_path_buf(),
        }
    }

    fn snapshot(&self) -> SessionSnapshot {
        let files: Vec<FileSnapshot> = self
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| FileSnapshot {
                index,
                name: file.name.clone(),
                size: file.size,
                selected: self.selected.get(index).copied().unwrap_or(false),
                watched: self.watched_files.iter().any(|f| f == &file.name),
                downloaded: self.downloaded_files.iter().any(|f| f == &file.name),
                downloading: self.downloading_files.iter().any(|f| f == &file.name),
            })
            .collect();

        let selected_count = files.iter().filter(|f| f.selected).count();
        let selected_size = files.iter().filter(|f| f.selected).map(|f| f.size).sum();
        let total_size = files.iter().map(|f| f.size).sum();

        SessionSnapshot {
            url: self.url.clone(),
            name: self.name.clone(),
            files,
            selected_count,
            selected_size,
            total_size,
            download_root: self.download_root().map(ToOwned::to_owned),
        }
    }

    fn prune_missing(&mut self, download_dir: &Path) {
        let root = self.download_root_dir(download_dir);
        self.watched_files.retain(|f| root.join(f).exists());
        self.downloaded_files.retain(|f| root.join(f).exists());
    }

    fn mark_downloading(&mut self, names: &[String]) {
        for name in names {
            if !self.downloading_files.iter().any(|f| f == name) {
                self.downloading_files.push(name.clone());
            }
        }
    }

    fn mark_downloaded(&mut self, names: &[String]) {
        for name in names {
            self.downloading_files.retain(|f| f != name);
            if !self.downloaded_files.iter().any(|f| f == name) {
                self.downloaded_files.push(name.clone());
            }
        }
    }

    fn mark_watched(&mut self, name: &str) {
        if !self.watched_files.iter().any(|f| f == name) {
            self.watched_files.push(name.to_owned());
        }
    }
}

struct DownloadEntry {
    id: u64,
    torrent_id: Option<TorrentId>,
    torrent_name: String,
    label: String,
    file_names: Vec<String>,
    url: String,
    progress: f64,
    speed: u64,
    upload_speed: u64,
    peers: usize,
    total_size: u64,
    downloaded: u64,
    speed_ema: f64,
    is_streaming: bool,
    state: DownloadState,
    stream_path: Option<String>,
    subtitles: Vec<SubtitleTrack>,
    error: Option<String>,
}

impl DownloadEntry {
    fn snapshot(&self) -> DownloadSnapshot {
        let remaining = self.total_size.saturating_sub(self.downloaded);
        let eta_seconds = if self.speed_ema > 1.0 && remaining > 0 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Some((remaining as f64 / self.speed_ema).ceil() as u64)
        } else {
            None
        };

        DownloadSnapshot {
            id: self.id,
            torrent_id: self.torrent_id.map(TorrentId::raw),
            torrent_name: self.torrent_name.clone(),
            label: self.label.clone(),
            file_names: self.file_names.clone(),
            url: self.url.clone(),
            progress: self.progress.clamp(0.0, 1.0),
            speed: self.speed,
            upload_speed: self.upload_speed,
            peers: self.peers,
            total_size: self.total_size,
            downloaded: self.downloaded,
            eta_seconds,
            is_streaming: self.is_streaming,
            state: self.state,
            stream_path: self.stream_path.clone(),
            subtitles: self.subtitles.clone(),
            error: self.error.clone(),
        }
    }
}

#[derive(Default)]
struct PlayerState {
    title: Option<String>,
    stream_path: Option<String>,
    download_id: Option<u64>,
}

/// The headless twatch application.
pub struct Gateway {
    config: RwLock<Config>,
    search_config: RwLock<SearchConfig>,
    session: RwLock<Option<SessionState>>,
    downloads: RwLock<Vec<DownloadEntry>>,
    player_state: Mutex<PlayerState>,
    next_download_id: AtomicU64,
    revision: AtomicU64,
    events: broadcast::Sender<GatewayEvent>,
    internal_tx: mpsc::UnboundedSender<InternalEvent>,
    internal_rx: Mutex<mpsc::UnboundedReceiver<InternalEvent>>,
    torrent: Arc<dyn TorrentPort>,
    history: Arc<dyn HistoryPort>,
    search: Arc<dyn SearchPort>,
    player: Arc<dyn PlayerPort>,
    spawner: Arc<dyn TaskSpawner>,
}

impl Gateway {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Config,
        search_config: SearchConfig,
        torrent: Arc<dyn TorrentPort>,
        history: Arc<dyn HistoryPort>,
        search: Arc<dyn SearchPort>,
        player: Arc<dyn PlayerPort>,
        spawner: Arc<dyn TaskSpawner>,
    ) -> Arc<Self> {
        let (events, _) = broadcast::channel(256);
        let (internal_tx, internal_rx) = mpsc::unbounded_channel();
        Arc::new(Self {
            config: RwLock::new(config),
            search_config: RwLock::new(search_config),
            session: RwLock::new(None),
            downloads: RwLock::new(Vec::new()),
            player_state: Mutex::new(PlayerState::default()),
            next_download_id: AtomicU64::new(1),
            revision: AtomicU64::new(1),
            events,
            internal_tx,
            internal_rx: Mutex::new(internal_rx),
            torrent,
            history,
            search,
            player,
            spawner,
        })
    }

    // ── Read snapshots ───────────────────────────────────────────────

    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::SeqCst)
    }

    #[must_use]
    pub fn session(&self) -> Option<SessionSnapshot> {
        self.session.read().unwrap_or_else(|e| e.into_inner()).as_ref().map(SessionState::snapshot)
    }

    #[must_use]
    pub fn downloads(&self) -> Vec<DownloadSnapshot> {
        self.downloads
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .map(DownloadEntry::snapshot)
            .collect()
    }

    #[must_use]
    pub fn player(&self) -> PlayerSnapshot {
        let state = self.player_state.lock().unwrap_or_else(|e| e.into_inner());
        PlayerSnapshot {
            running: self.player.is_running(),
            title: state.title.clone(),
            stream_path: state.stream_path.clone(),
            download_id: state.download_id,
        }
    }

    #[must_use]
    pub fn config(&self) -> Config {
        self.config.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    #[must_use]
    pub fn config_view(&self) -> ConfigView {
        let config = self.config();
        ConfigView {
            theme: config.web_theme,
            download_dir: config.download_dir.display().to_string(),
            player: config.player,
        }
    }

    #[must_use]
    pub fn search_config(&self) -> SearchConfig {
        self.search_config.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    #[must_use]
    pub fn bootstrap(&self) -> BootstrapSnapshot {
        BootstrapSnapshot {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            config: self.config_view(),
            session: self.session(),
            downloads: self.downloads(),
            player: self.player(),
            themes: theme::THEMES.to_vec(),
            search_configured: !self.search_config().url.is_empty(),
        }
    }

    /// Subscribe to live changes.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<GatewayEvent> {
        self.events.subscribe()
    }

    // ── Session ──────────────────────────────────────────────────────

    /// Fetch metadata, remember it, and make it the current session.
    pub async fn preview(&self, url: &str) -> Result<SessionSnapshot> {
        let info = self.torrent.preview(url).await?;
        self.history.add_entry(url, &info.name);

        let entry = self.history.history().into_iter().find(|entry| entry.url == url).unwrap_or(
            HistoryEntry {
                url: url.to_owned(),
                custom_name: None,
                torrent_name: info.name.clone(),
                added_at: 0,
                watched: false,
                watched_files: Vec::new(),
                downloaded: false,
                downloaded_files: Vec::new(),
            },
        );

        let mut session = SessionState::new(
            url.to_owned(),
            info.name,
            info.files,
            entry.watched_files,
            entry.downloaded_files,
        );
        let download_dir = self.config().download_dir;
        session.prune_missing(&download_dir);
        let snapshot = session.snapshot();

        *self.session.write().unwrap_or_else(|e| e.into_inner()) = Some(session);
        self.emit(GatewayEvent::SessionChanged);
        Ok(snapshot)
    }

    /// Open a history entry by fetching its metadata again.
    pub async fn open_history(&self, url: &str) -> Result<SessionSnapshot> {
        self.preview(url).await
    }

    /// Close the current browse session.  Live downloads are unaffected.
    pub fn close_session(&self) {
        *self.session.write().unwrap_or_else(|e| e.into_inner()) = None;
        self.emit(GatewayEvent::SessionChanged);
    }

    /// Select or deselect files by index.
    pub fn select_files(&self, indices: &[usize], selected: bool) -> Result<()> {
        {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_mut().context("No active torrent session")?;
            for &index in indices {
                if let Some(slot) = session.selected.get_mut(index) {
                    *slot = selected;
                }
            }
        }
        self.emit(GatewayEvent::SessionChanged);
        Ok(())
    }

    /// Select every file (or clear the selection).
    pub fn select_all(&self, selected: bool) -> Result<()> {
        {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_mut().context("No active torrent session")?;
            session.selected.fill(selected);
        }
        self.emit(GatewayEvent::SessionChanged);
        Ok(())
    }

    // ── History ──────────────────────────────────────────────────────

    #[must_use]
    pub fn history(&self) -> Vec<HistoryEntry> {
        self.history.history()
    }

    pub fn rename_history(&self, url: &str, name: &str) -> Result<()> {
        let index = self
            .history
            .history()
            .iter()
            .position(|entry| entry.url == url)
            .context("History entry not found")?;
        self.history.rename_entry(index, name)
    }

    pub fn delete_history(&self, url: &str) -> Result<()> {
        let index = self
            .history
            .history()
            .iter()
            .position(|entry| entry.url == url)
            .context("History entry not found")?;
        self.history.remove_entry(index)
    }

    // ── Search ───────────────────────────────────────────────────────

    pub async fn search(&self, query: &str, page: u32) -> Result<Vec<SearchResult>> {
        let config = self.search_config();
        self.search.search(query, &config, page).await
    }

    pub fn update_search_config(&self, config: SearchConfig) -> Result<()> {
        config.save(&self.config().config_dir)?;
        *self.search_config.write().unwrap_or_else(|e| e.into_inner()) = config;
        Ok(())
    }

    // ── Downloads ────────────────────────────────────────────────────

    /// Start downloading one file for streaming.
    pub fn watch_file(&self, index: usize) -> Result<DownloadSnapshot> {
        let (url, torrent_name, file, subtitles) = {
            let guard = self.session.read().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_ref().context("No active torrent session")?;
            let file = session.files.get(index).context("File not found")?.clone();
            let subtitles = collect_subtitles(&session.files);
            (session.url.clone(), session.name.clone(), file, subtitles)
        };

        let subtitle_indices: Vec<usize> = subtitles.iter().map(|track| track.index).collect();
        let id = self.next_download_id.fetch_add(1, Ordering::SeqCst);
        let entry = DownloadEntry {
            id,
            torrent_id: None,
            torrent_name,
            label: file.name.clone(),
            file_names: vec![file.name.clone()],
            url: url.clone(),
            progress: 0.0,
            speed: 0,
            upload_speed: 0,
            peers: 0,
            total_size: file.size,
            downloaded: 0,
            speed_ema: 0.0,
            is_streaming: true,
            state: DownloadState::Starting,
            stream_path: None,
            subtitles,
            error: None,
        };
        let snapshot = entry.snapshot();
        self.downloads.write().unwrap_or_else(|e| e.into_inner()).push(entry);
        self.emit(GatewayEvent::DownloadsChanged);

        let torrent = self.torrent.clone();
        let tx = self.internal_tx.clone();
        let file_index = file.index;
        self.spawner.spawn(Box::pin(async move {
            match torrent.download_with_extra(&url, file_index, &subtitle_indices).await {
                Ok(torrent_id) => {
                    let path = format!("/api/stream/{}/{}", torrent_id.raw(), file_index);
                    let _ =
                        tx.send(InternalEvent::Ready { id, torrent_id, stream_path: Some(path) });
                }
                Err(error) => {
                    let _ = tx.send(InternalEvent::Failed {
                        id,
                        message: format!("Stream failed: {error}"),
                    });
                }
            }
        }));

        Ok(snapshot)
    }

    /// Start a batch download of the selected files.
    pub fn download_files(&self, indices: &[usize]) -> Result<DownloadSnapshot> {
        if indices.is_empty() {
            bail!("No files selected");
        }

        let download_dir = self.config().download_dir;
        let (url, torrent_name, output_dir, file_names, total_size, subtitles) = {
            let guard = self.session.read().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_ref().context("No active torrent session")?;
            let files: Vec<TorrentFile> =
                indices.iter().filter_map(|index| session.files.get(*index).cloned()).collect();
            if files.is_empty() {
                bail!("No matching files");
            }
            let names: Vec<String> = files.iter().map(|f| f.name.clone()).collect();
            let total: u64 = files.iter().map(|f| f.size).sum();
            (
                session.url.clone(),
                session.name.clone(),
                session.download_root_dir(&download_dir),
                names,
                total,
                collect_subtitles(&session.files),
            )
        };

        let label = if file_names.len() == 1 {
            file_names[0].clone()
        } else {
            format!("{} files", file_names.len())
        };
        let id = self.next_download_id.fetch_add(1, Ordering::SeqCst);
        let entry = DownloadEntry {
            id,
            torrent_id: None,
            torrent_name,
            label,
            file_names: file_names.clone(),
            url: url.clone(),
            progress: 0.0,
            speed: 0,
            upload_speed: 0,
            peers: 0,
            total_size,
            downloaded: 0,
            speed_ema: 0.0,
            is_streaming: false,
            state: DownloadState::Starting,
            stream_path: None,
            subtitles,
            error: None,
        };
        let snapshot = entry.snapshot();
        self.downloads.write().unwrap_or_else(|e| e.into_inner()).push(entry);
        self.emit(GatewayEvent::DownloadsChanged);

        {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            if let Some(session) = guard.as_mut() {
                session.mark_downloading(&file_names);
            }
        }
        self.emit(GatewayEvent::SessionChanged);

        let torrent = self.torrent.clone();
        let tx = self.internal_tx.clone();
        let indices = indices.to_vec();
        self.spawner.spawn(Box::pin(async move {
            match torrent.download_to_folder(&url, &indices, &output_dir).await {
                Ok(torrent_id) => {
                    let _ = tx.send(InternalEvent::Ready { id, torrent_id, stream_path: None });
                }
                Err(error) => {
                    let _ = tx.send(InternalEvent::Failed {
                        id,
                        message: format!("Download failed: {error}"),
                    });
                }
            }
        }));

        Ok(snapshot)
    }

    /// Poll the engine.  Call this from a 250 ms ticker (daemon) or a test.
    pub fn tick(&self) {
        self.drain_internal();

        let mut changed = false;
        let mut completed: Vec<(String, Vec<String>, String)> = Vec::new();

        {
            let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
            for entry in downloads.iter_mut() {
                if !matches!(entry.state, DownloadState::Downloading | DownloadState::Streaming) {
                    continue;
                }
                let Some(torrent_id) = entry.torrent_id else {
                    continue;
                };
                let Ok(stats) = self.torrent.get_stats(&torrent_id) else {
                    continue;
                };
                let before = (entry.progress, entry.speed, entry.peers, entry.downloaded);
                apply_stats(entry, &stats);
                if before != (entry.progress, entry.speed, entry.peers, entry.downloaded) {
                    changed = true;
                }

                if !entry.is_streaming
                    && entry.progress >= 1.0
                    && entry.state != DownloadState::Completed
                {
                    entry.state = DownloadState::Completed;
                    entry.speed = 0;
                    entry.speed_ema = 0.0;
                    completed.push((
                        entry.url.clone(),
                        entry.file_names.clone(),
                        entry.label.clone(),
                    ));
                }
            }
        }

        if changed {
            self.emit(GatewayEvent::DownloadsChanged);
        }

        for (url, file_names, label) in completed {
            for name in &file_names {
                self.history.mark_downloaded(&url, name);
            }
            {
                let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
                if let Some(session) = guard.as_mut() {
                    if session.url == url {
                        session.mark_downloaded(&file_names);
                    }
                }
            }
            self.emit(GatewayEvent::SessionChanged);
            self.emit(GatewayEvent::Toast {
                level: ToastLevel::Success,
                message: format!("Download complete: {label}"),
            });
            self.emit(GatewayEvent::DownloadsChanged);
        }
    }

    fn drain_internal(&self) {
        let mut ready: Vec<(u64, TorrentId, Option<String>)> = Vec::new();
        let mut failed: Vec<(u64, String)> = Vec::new();
        {
            let mut rx = self.internal_rx.lock().unwrap_or_else(|e| e.into_inner());
            while let Ok(event) = rx.try_recv() {
                match event {
                    InternalEvent::Ready { id, torrent_id, stream_path } => {
                        ready.push((id, torrent_id, stream_path));
                    }
                    InternalEvent::Failed { id, message } => failed.push((id, message)),
                }
            }
        }

        let mut dirty = false;
        let mut session_dirty = false;
        let mut toasts: Vec<(ToastLevel, String)> = Vec::new();

        for (id, torrent_id, stream_path) in ready {
            let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
            let Some(entry) = downloads.iter_mut().find(|entry| entry.id == id) else {
                continue;
            };
            entry.torrent_id = Some(torrent_id);
            entry.stream_path = stream_path;
            entry.state = if entry.is_streaming {
                DownloadState::Streaming
            } else {
                DownloadState::Downloading
            };
            let is_streaming = entry.is_streaming;
            let file_names = entry.file_names.clone();
            let url = entry.url.clone();
            let label = entry.label.clone();
            drop(downloads);

            if is_streaming {
                if let Some(name) = file_names.first() {
                    self.history.mark_watched(&url, name);
                    let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
                    if let Some(session) = guard.as_mut() {
                        if session.url == url {
                            session.mark_watched(name);
                        }
                    }
                    session_dirty = true;
                }
                toasts.push((ToastLevel::Info, format!("Streaming: {label}")));
            }
            dirty = true;
        }

        for (id, message) in failed {
            let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = downloads.iter_mut().find(|entry| entry.id == id) {
                entry.state = DownloadState::Failed;
                entry.error = Some(message.clone());
                entry.speed = 0;
                entry.speed_ema = 0.0;
                dirty = true;
            }
            toasts.push((ToastLevel::Error, message));
        }

        if dirty {
            self.emit(GatewayEvent::DownloadsChanged);
        }
        if session_dirty {
            self.emit(GatewayEvent::SessionChanged);
        }
        for (level, message) in toasts {
            self.emit(GatewayEvent::Toast { level, message });
        }
    }

    pub async fn pause_download(&self, id: u64) -> Result<()> {
        let torrent_id = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            let entry =
                downloads.iter().find(|entry| entry.id == id).context("Download not found")?;
            entry.torrent_id.context("Download is still starting")?
        };
        self.torrent.pause(&torrent_id).await?;
        {
            let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = downloads.iter_mut().find(|entry| entry.id == id) {
                entry.state = DownloadState::Paused;
                entry.speed = 0;
                entry.speed_ema = 0.0;
            }
        }
        self.emit(GatewayEvent::DownloadsChanged);
        Ok(())
    }

    pub async fn resume_download(&self, id: u64) -> Result<()> {
        let torrent_id = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            let entry =
                downloads.iter().find(|entry| entry.id == id).context("Download not found")?;
            entry.torrent_id.context("Download is still starting")?
        };
        self.torrent.resume(&torrent_id).await?;
        {
            let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = downloads.iter_mut().find(|entry| entry.id == id) {
                entry.state = if entry.is_streaming {
                    DownloadState::Streaming
                } else {
                    DownloadState::Downloading
                };
            }
        }
        self.emit(GatewayEvent::DownloadsChanged);
        Ok(())
    }

    pub async fn remove_download(&self, id: u64) -> Result<()> {
        let torrent_id = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            downloads.iter().find(|entry| entry.id == id).and_then(|entry| entry.torrent_id)
        };
        if let Some(torrent_id) = torrent_id {
            self.torrent.remove(&torrent_id, false).await?;
        }
        self.downloads.write().unwrap_or_else(|e| e.into_inner()).retain(|entry| entry.id != id);

        let should_stop = {
            let mut state = self.player_state.lock().unwrap_or_else(|e| e.into_inner());
            if state.download_id == Some(id) {
                state.title = None;
                state.stream_path = None;
                state.download_id = None;
                true
            } else {
                false
            }
        };
        if should_stop {
            self.player.kill();
            self.emit(GatewayEvent::PlayerChanged);
        }
        self.emit(GatewayEvent::DownloadsChanged);
        Ok(())
    }

    // ── Files ────────────────────────────────────────────────────────

    /// Delete selected files from disk and sync status/history.
    pub fn delete_files(&self, indices: &[usize]) -> DeleteReport {
        let download_dir = self.config().download_dir;
        let (url, files, root) = {
            let guard = self.session.read().unwrap_or_else(|e| e.into_inner());
            let Some(session) = guard.as_ref() else {
                return DeleteReport::default();
            };
            let files: Vec<(usize, TorrentFile)> = indices
                .iter()
                .filter_map(|index| session.files.get(*index).cloned().map(|f| (*index, f)))
                .collect();
            (session.url.clone(), files, session.download_root_dir(&download_dir))
        };

        let mut report = DeleteReport::default();
        let mut removed_names = Vec::new();
        for (_, file) in files {
            let path = root.join(&file.name);
            if !path.starts_with(&root) {
                report.failed.push(file.name);
                continue;
            }
            match std::fs::remove_file(&path) {
                Ok(()) => {
                    report.deleted += 1;
                    removed_names.push(file.name.clone());
                    self.history.mark_deleted(&url, &file.name);
                }
                Err(error) => {
                    tracing::warn!("Failed to delete {}: {error}", file.name);
                    report.failed.push(file.name);
                }
            }
        }

        if !removed_names.is_empty() {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            if let Some(session) = guard.as_mut() {
                session.watched_files.retain(|f| !removed_names.contains(f));
                session.downloaded_files.retain(|f| !removed_names.contains(f));
                session.downloading_files.retain(|f| !removed_names.contains(f));
                for index in indices {
                    if let Some(slot) = session.selected.get_mut(*index) {
                        *slot = false;
                    }
                }
            }
            drop(guard);
            self.emit(GatewayEvent::SessionChanged);
        }
        report
    }

    /// Rename one file on disk and in the session/history.
    pub fn rename_file(&self, index: usize, new_name: &str) -> Result<String> {
        if new_name.is_empty() || new_name.len() > 256 {
            bail!("File name must be between 1 and 256 characters");
        }
        if new_name.contains('/') || new_name.contains('\\') || new_name.contains('\0') {
            bail!("File name cannot contain path separators");
        }

        let download_dir = self.config().download_dir;
        let (url, old_name, old_path, new_path, root) = {
            let guard = self.session.read().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_ref().context("No active torrent session")?;
            let old = session.files.get(index).context("File not found")?.clone();
            let root = session.download_root_dir(&download_dir);
            let old_path = root.join(&old.name);
            let new_path = root.join(new_name);
            (session.url.clone(), old.name, old_path, new_path, root)
        };

        if !new_path.starts_with(&root) {
            bail!("Invalid file name");
        }

        if old_path.exists() {
            if let Some(parent) = new_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(&old_path, &new_path)?;
        }

        {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_mut().context("No active torrent session")?;
            if let Some(file) = session.files.iter_mut().find(|file| file.index == index) {
                file.name = new_name.to_owned();
            }
            for list in [
                &mut session.watched_files,
                &mut session.downloaded_files,
                &mut session.downloading_files,
            ] {
                for name in list.iter_mut() {
                    if *name == old_name {
                        *name = new_name.to_owned();
                    }
                }
            }
        }

        self.history.rename_file(&url, &old_name, new_name);
        self.emit(GatewayEvent::SessionChanged);
        Ok(format!("Renamed to: {new_name}"))
    }

    // ── Player ───────────────────────────────────────────────────────

    pub fn open_player(&self, download_id: u64, base_url: &str) -> Result<PlayerSnapshot> {
        let download = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            downloads
                .iter()
                .find(|entry| entry.id == download_id)
                .map(DownloadEntry::snapshot)
                .context("Download not found")?
        };
        let stream_path = download.stream_path.clone().context("Stream is not ready yet")?;
        let url = format!("{}{}", base_url.trim_end_matches('/'), stream_path);

        self.player.play(&url, &download.label);
        {
            let mut state = self.player_state.lock().unwrap_or_else(|e| e.into_inner());
            state.title = Some(download.label);
            state.stream_path = Some(stream_path);
            state.download_id = Some(download_id);
        }
        self.emit(GatewayEvent::PlayerChanged);
        Ok(self.player())
    }

    pub fn stop_player(&self) {
        self.player.kill();
        {
            let mut state = self.player_state.lock().unwrap_or_else(|e| e.into_inner());
            *state = PlayerState::default();
        }
        self.emit(GatewayEvent::PlayerChanged);
    }

    // ── Config ───────────────────────────────────────────────────────

    pub fn update_theme(&self, name: &str) -> Result<()> {
        if theme::find(name).is_none() {
            bail!("Unknown theme: {name}");
        }
        let config = {
            let mut guard = self.config.write().unwrap_or_else(|e| e.into_inner());
            guard.web_theme = name.to_owned();
            guard.clone()
        };
        config.save()?;
        Ok(())
    }

    pub fn update_player(&self, player: &str) -> Result<()> {
        if !matches!(player, "auto" | "mpv" | "vlc" | "none") {
            bail!("Unknown player preference: {player}");
        }
        let config = {
            let mut guard = self.config.write().unwrap_or_else(|e| e.into_inner());
            guard.player = player.to_owned();
            guard.clone()
        };
        config.save()?;
        Ok(())
    }

    // ── Events ───────────────────────────────────────────────────────

    fn emit(&self, event: GatewayEvent) {
        self.revision.fetch_add(1, Ordering::SeqCst);
        let _ = self.events.send(event);
    }
}

fn apply_stats(entry: &mut DownloadEntry, stats: &TorrentStats) {
    entry.progress = stats.progress.clamp(0.0, 1.0);
    entry.speed = stats.download_speed;
    entry.upload_speed = stats.upload_speed;
    entry.peers = stats.peers;
    entry.total_size = stats.total_size;
    entry.downloaded = stats.downloaded;
    #[allow(clippy::cast_precision_loss)]
    let speed = stats.download_speed as f64;
    entry.speed_ema =
        if entry.speed_ema == 0.0 { speed } else { 0.7 * entry.speed_ema + 0.3 * speed };
}

fn is_subtitle_file(name: &str) -> bool {
    let extension = name.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();
    matches!(extension.as_str(), "srt" | "ass" | "ssa" | "vtt")
}

fn collect_subtitles(files: &[TorrentFile]) -> Vec<SubtitleTrack> {
    files
        .iter()
        .filter(|file| is_subtitle_file(&file.name))
        .map(|file| SubtitleTrack { index: file.index, name: file.name.clone() })
        .collect()
}

#[cfg(test)]
mod subtitle_tests {
    use super::{collect_subtitles, is_subtitle_file};
    use crate::domain::torrent::TorrentFile;

    #[test]
    fn collects_only_browser_convertible_subtitles() {
        let files = vec![
            TorrentFile { index: 0, name: "movie.mkv".into(), size: 10 },
            TorrentFile { index: 1, name: "subs/movie.en.srt".into(), size: 2 },
            TorrentFile { index: 2, name: "subs/movie.es.ass".into(), size: 2 },
            TorrentFile { index: 3, name: "subs/movie.sup".into(), size: 2 },
            TorrentFile { index: 4, name: "subs/movie.en.vtt".into(), size: 2 },
        ];
        assert!(is_subtitle_file("subs/movie.en.srt"));
        assert!(!is_subtitle_file("movie.mkv"));
        let subtitles = collect_subtitles(&files);
        assert_eq!(subtitles.iter().map(|track| track.index).collect::<Vec<_>>(), vec![1, 2, 4]);
    }
}
