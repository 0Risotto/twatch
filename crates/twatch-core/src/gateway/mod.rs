//! Headless application gateway shared by `twatchd`, the desktop shells, and
//! tests.
//!
//! The gateway owns the current browse session, the live download registry, the
//! player state, and the event stream.  It has no HTTP or UI types in it;
//! `twatch-server` turns these snapshots into JSON and WebSocket frames.

use std::fs;
mod config;
mod downloads;
mod files;
mod history;
mod lifecycle;
mod persistence;
mod player;
mod search;
mod session;
mod tick;
use std::io::Write;
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

/// One playable file in the current torrent, used to build a next/previous queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueFile {
    pub index: usize,
    pub name: String,
    pub size: u64,
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
    /// Index of this file inside the torrent (streams only).
    pub file_index: usize,
    /// Playable files in the torrent, in torrent order (streams only).
    pub queue: Vec<QueueFile>,
    /// Saved resume position for this file, if any.
    pub resume_position: Option<f64>,
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
    file_index: usize,
    file_indices: Vec<usize>,
    output_subfolder: Option<String>,
    queue: Vec<QueueFile>,
    resume_position: Option<f64>,
}

/// The durable part of a download, restored in a paused state on startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedDownload {
    id: u64,
    url: String,
    torrent_name: String,
    label: String,
    file_names: Vec<String>,
    total_size: u64,
    downloaded: u64,
    progress: f64,
    is_streaming: bool,
    state: DownloadState,
    file_index: usize,
    file_indices: Vec<usize>,
    output_subfolder: Option<String>,
    queue: Vec<QueueFile>,
    subtitles: Vec<SubtitleTrack>,
}

const DOWNLOADS_FILENAME: &str = "downloads.json";

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
            file_index: self.file_index,
            queue: self.queue.clone(),
            resume_position: self.resume_position,
        }
    }

    fn persisted(&self) -> PersistedDownload {
        PersistedDownload {
            id: self.id,
            url: self.url.clone(),
            torrent_name: self.torrent_name.clone(),
            label: self.label.clone(),
            file_names: self.file_names.clone(),
            total_size: self.total_size,
            downloaded: self.downloaded,
            progress: self.progress,
            is_streaming: self.is_streaming,
            state: self.state,
            file_index: self.file_index,
            file_indices: self.file_indices.clone(),
            output_subfolder: self.output_subfolder.clone(),
            queue: self.queue.clone(),
            subtitles: self.subtitles.clone(),
        }
    }
}

/// Extensions the browser player or a native player can reasonably open.
const PLAYABLE_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "mkv", "webm", "avi", "mov", "ts", "m2ts", "flv", "wmv", "ogv", "mpg", "mpeg",
    "mp3", "m4a", "aac", "flac", "opus", "ogg", "oga", "wav", "wma",
];

fn is_playable_file(name: &str) -> bool {
    name.rsplit('.').next().is_some_and(|extension| {
        let extension = extension.to_ascii_lowercase();
        PLAYABLE_EXTENSIONS.iter().any(|candidate| *candidate == extension)
    })
}

fn playable_queue(files: &[TorrentFile]) -> Vec<QueueFile> {
    files
        .iter()
        .filter(|file| is_playable_file(&file.name))
        .map(|file| QueueFile { index: file.index, name: file.name.clone(), size: file.size })
        .collect()
}

fn write_private_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value).context("Failed to serialize downloads")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create download state directory")?;
    }
    let tmp = path.with_extension("tmp");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)
        .context("Failed to open download state file")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600)).ok();
    }
    file.write_all(json.as_bytes()).context("Failed to write download state")?;
    file.flush().context("Failed to flush download state")?;
    fs::rename(&tmp, path).context("Failed to replace download state")?;
    Ok(())
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
mod playable_tests {
    use super::{is_playable_file, playable_queue};
    use crate::domain::torrent::TorrentFile;

    #[test]
    fn filters_to_playable_media_and_keeps_torrent_order() {
        let files = vec![
            TorrentFile { index: 0, name: "Show.S01E01.mkv".into(), size: 10 },
            TorrentFile { index: 1, name: "Show.S01E01.en.srt".into(), size: 2 },
            TorrentFile { index: 2, name: "Show.S01E02.MKV".into(), size: 11 },
            TorrentFile { index: 3, name: "extras/notes.txt".into(), size: 1 },
            TorrentFile { index: 4, name: "Show.S01E03.flac".into(), size: 12 },
        ];
        assert!(is_playable_file("Show.S01E01.mkv"));
        assert!(is_playable_file("Show.S01E02.MKV"));
        assert!(is_playable_file("track.flac"));
        assert!(!is_playable_file("Show.S01E01.en.srt"));
        assert!(!is_playable_file("notes.txt"));

        let queue = playable_queue(&files);
        assert_eq!(queue.iter().map(|file| file.index).collect::<Vec<_>>(), vec![0, 2, 4]);
        assert_eq!(queue[1].name, "Show.S01E02.MKV");
    }
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
