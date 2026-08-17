//! Watch/download background flow.
//!
//! Each download is a siloed task: it owns its own oneshot channel
//! and carries its full metadata (url, names, label) with its event,
//! so concurrent downloads — including several files of the same
//! torrent — never interfere with each other or read stale session
//! state.

use crate::app::App;
use crate::app::screens::Transition;
use crate::app::screens::loading::LoadingScreen;
use crate::app::screens::registry;
use crate::domain::common::spawner::TaskSpawner;
use crate::domain::history::ports::HistoryPort;
use crate::domain::player::ports::PlayerPort;
use crate::domain::torrent::TorrentFile;
use crate::domain::torrent::TorrentId;
use crate::domain::torrent::ports::{StreamPort, TorrentPort};
use crate::model::ActiveDownload;
use shaku::HasComponent;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;

/// Everything a download task needs to report about itself.
///
/// Carried with every event so the handler never relies on shared
/// session state that may have moved on since the download started.
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// Unique silo id (matches the loading screen pushed for it).
    pub id: u64,
    /// The torrent url being downloaded.
    pub url: String,
    /// Torrent display name at spawn time.
    pub torrent_name: String,
    /// Human label: the file name (watch) or "N files" (batch).
    pub label: String,
    /// The files this download covers.
    pub file_names: Vec<String>,
    /// True for streaming (watch) downloads.
    pub is_watch: bool,
}

/// Events produced by download background tasks.
pub enum DownloadEvent {
    Ready { task: DownloadTask, torrent: TorrentId, stream_url: String },
    Failed { task: DownloadTask, message: String },
}

/// Owns one channel per in-flight download.
#[derive(Default)]
pub struct DownloadFlow {
    next_id: u64,
    pending: Vec<(u64, oneshot::Receiver<DownloadEvent>)>,
}

impl DownloadFlow {
    /// Create an idle flow with no in-flight downloads.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a background task that downloads a single file for
    /// streaming, then resolves the stream URL.  Returns the silo id.
    pub fn spawn_watch(
        &mut self,
        app: &App,
        url: String,
        file_index: usize,
        file_name: String,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let task = DownloadTask {
            id,
            url: url.clone(),
            torrent_name: app.torrent.torrent_name.clone().unwrap_or_default(),
            label: file_name.clone(),
            file_names: vec![file_name],
            is_watch: true,
        };

        let torrent: Arc<dyn TorrentPort> = app.module.resolve();
        let stream: Arc<dyn StreamPort> = app.module.resolve();
        let spawner: Arc<dyn TaskSpawner> = app.module.resolve();
        let (tx, rx) = oneshot::channel();
        self.pending.push((id, rx));

        spawner.spawn(Box::pin(async move {
            let event = match torrent.download(&url, file_index).await {
                Ok(torrent_id) => {
                    let stream_url = stream.stream_url(&torrent_id, file_index);
                    DownloadEvent::Ready { task, torrent: torrent_id, stream_url }
                }
                Err(e) => DownloadEvent::Failed { task, message: format!("Failed: {e}") },
            };
            let _ = tx.send(event);
        }));

        id
    }

    /// Spawn a background task that downloads the selected files to
    /// `output_dir`.  Returns the silo id.
    pub fn spawn_batch(
        &mut self,
        app: &App,
        url: String,
        files: Vec<TorrentFile>,
        output_dir: PathBuf,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let file_names: Vec<String> = files.iter().map(|f| f.name.clone()).collect();
        let indices: Vec<usize> = files.iter().map(|f| f.index).collect();
        let label = if file_names.len() == 1 {
            file_names[0].clone()
        } else {
            format!("{} files", file_names.len())
        };

        let task = DownloadTask {
            id,
            url: url.clone(),
            torrent_name: app.torrent.torrent_name.clone().unwrap_or_default(),
            label,
            file_names,
            is_watch: false,
        };

        let torrent: Arc<dyn TorrentPort> = app.module.resolve();
        let spawner: Arc<dyn TaskSpawner> = app.module.resolve();
        let (tx, rx) = oneshot::channel();
        self.pending.push((id, rx));

        // Multi-file torrents download into `<output_dir>/<torrent name>/`,
        // mirroring librqbit's default sub-folder behaviour.
        let output_dir = app.torrent.download_root_dir(&output_dir);

        spawner.spawn(Box::pin(async move {
            let event = match torrent.download_to_folder(&url, &indices, &output_dir).await {
                Ok(torrent_id) => {
                    DownloadEvent::Ready { task, torrent: torrent_id, stream_url: String::new() }
                }
                Err(e) => DownloadEvent::Failed { task, message: format!("Failed: {e}") },
            };
            let _ = tx.send(event);
        }));

        id
    }

    /// Collect events from every download that has finished since the
    /// last drain.  In-flight downloads stay queued.
    pub fn drain(&mut self) -> Vec<DownloadEvent> {
        let mut events = Vec::new();
        self.pending.retain_mut(|(_, rx)| match rx.try_recv() {
            Ok(event) => {
                events.push(event);
                false
            }
            Err(oneshot::error::TryRecvError::Empty) => true,
            Err(oneshot::error::TryRecvError::Closed) => false,
        });
        events
    }
}

/// Apply a download event to the application state.
pub fn apply(app: &mut App, event: DownloadEvent) {
    match event {
        DownloadEvent::Ready { task, torrent, stream_url } => {
            app.torrent_id = Some(torrent);
            app.active_downloads.push(ActiveDownload {
                torrent_id: torrent,
                torrent_name: task.torrent_name.clone(),
                file_name: task.label.clone(),
                file_names: task.file_names.clone(),
                url: task.url.clone(),
                progress: 0.0,
                download_speed: 0,
                total_size: 0,
                downloaded: 0,
                is_streaming: task.is_watch,
            });
            if task.is_watch && !stream_url.is_empty() {
                let file_name = &task.file_names[0];
                let storage: Arc<dyn HistoryPort> = app.module.resolve();
                storage.mark_watched(&task.url, file_name);
                if !app.torrent.watched_files.iter().any(|f| f == file_name) {
                    app.torrent.watched_files.push(file_name.clone());
                }
                let player: Arc<dyn PlayerPort> = app.module.resolve();
                player.play(&stream_url, file_name);
                app.status_message = format!("Watching: {}", task.label);
                app.apply_transition(Transition::Replace(registry::player()));
            } else {
                for file_name in &task.file_names {
                    if !app.torrent.downloading_files.iter().any(|f| f == file_name) {
                        app.torrent.downloading_files.push(file_name.clone());
                    }
                }
                app.status_message =
                    format!("Downloading to: {}", app.config.download_dir.display());
                pop_loading_if_matching(app, task.id);
            }
        }
        DownloadEvent::Failed { task, message } => {
            app.set_error(message);
            pop_loading_if_matching(app, task.id);
        }
    }
}

/// Pop the top loading screen only when it belongs to `task_id`, so a
/// finished download never dismisses another download's loading screen.
fn pop_loading_if_matching(app: &mut App, task_id: u64) {
    if let Some(loading) = app.top_as::<LoadingScreen>() {
        if loading.task() == Some(task_id) {
            app.apply_transition(Transition::Pop);
        }
    }
}
