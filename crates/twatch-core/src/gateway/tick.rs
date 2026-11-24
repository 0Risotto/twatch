use super::*;

impl Gateway {
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

        let had_completed = !completed.is_empty();
        for (url, file_names, label) in completed {
            for name in &file_names {
                self.history.mark_downloaded(&url, name);
            }
            {
                let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
                if let Some(session) = guard.as_mut()
                    && session.url == url
                {
                    session.mark_downloaded(&file_names);
                }
            }
            self.emit(GatewayEvent::SessionChanged);
            self.emit(GatewayEvent::Toast {
                level: ToastLevel::Success,
                message: format!("Download complete: {label}"),
            });
            self.emit(GatewayEvent::DownloadsChanged);
        }
        if had_completed {
            self.persist_downloads();
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
                    if let Some(session) = guard.as_mut()
                        && session.url == url
                    {
                        session.mark_watched(name);
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
            self.persist_downloads();
        }
        if session_dirty {
            self.emit(GatewayEvent::SessionChanged);
        }
        for (level, message) in toasts {
            self.emit(GatewayEvent::Toast { level, message });
        }
    }
}
