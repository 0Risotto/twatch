//! Application state, screens, and main event loop.
//!
//! [`App`] holds all UI-relevant state plus the DI [`AppModule`].
//! Service dependencies are resolved on demand via
//! [`HasComponent::resolve`].  Async torrent work is dispatched
//! through an internal mpsc event bus.

use crate::model::{
    ActiveDownload, InputState, Screen, TorrentFile, TorrentId, TorrentInfo, TorrentStats,
};
use crate::traits::{PlayerService, StorageService, TorrentService};
use crate::{config::Config, module::AppModule, ui};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::CrosstermBackend};
use shaku::HasComponent;
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
    pub status_message: String,
    pub running: bool,
    pub history_selected: usize,
    pub menu_selected: usize,
    pub renaming: bool,
    pub rename_input: InputState,
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
            status_message: String::new(),
            running: true,
            history_selected: 0,
            menu_selected: 0,
            renaming: false,
            rename_input: InputState::default(),
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
}

/// Run the main application loop.
///
/// Drives the TUI with ratatui-crossterm, drains the MPSC event
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
                    app.set_pending_url(url);
                    app.torrent_name = Some(info.name.clone());
                    app.files = info.files.clone();
                    app.selected_files = vec![false; info.files.len()];
                    app.selected_file = 0;
                    app.screen = Screen::Browser;
                    app.status_message =
                        format!("{} file(s) found — w: watch, d: download", app.files.len());
                }
                AppEvent::PreviewFailed(msg) => {
                    app.set_error(msg);
                    app.screen = Screen::Input;
                }
                AppEvent::DownloadReady { id, stream_url, file_name, is_watch } => {
                    app.torrent_id = Some(id.clone());
                    app.active_downloads.push(ActiveDownload {
                        torrent_id: id.clone(),
                        torrent_name: app.torrent_name.clone().unwrap_or_default(),
                        file_name: file_name.clone(),
                        progress: 0.0,
                        download_speed: 0,
                        total_size: 0,
                        downloaded: 0,
                        is_streaming: is_watch,
                    });
                    if is_watch && !stream_url.is_empty() {
                        let player: Arc<dyn PlayerService> = app.module.resolve();
                        player.play(&stream_url, &file_name);
                        app.status_message = format!("Watching: {}", file_name);
                    } else {
                        app.status_message =
                            format!("Downloading to: {}", app.config.download_dir.display());
                    }
                    app.screen = Screen::Player;
                }
                AppEvent::DownloadFailed(msg) => {
                    app.set_error(msg);
                    app.screen = Screen::Browser;
                }
            }
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handlers::handle_key(&mut app, key.code);
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
                if let Ok(stats) = torrent.get_stats(&dl.torrent_id) {
                    dl.progress = stats.progress;
                    dl.download_speed = stats.download_speed;
                    dl.total_size = stats.total_size;
                    dl.downloaded = stats.downloaded;
                }
            }
            // Also keep the single-torrent stats for the Player screen.
            if let Some(id) = &app.torrent_id {
                if let Ok(stats) = torrent.get_stats(id) {
                    app.stats = Some(stats);
                }
            }
        }

        if let Some(msg) = app.error_message.take() {
            app.status_message = msg;
        }
    }

    Ok(())
}
