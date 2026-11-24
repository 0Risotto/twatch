use super::*;

impl Gateway {
    /// Write the durable download list next to the history file.
    pub(super) fn persist_downloads(&self) {
        let config = self.config();
        let path = config.config_dir.join(DOWNLOADS_FILENAME);
        let data: Vec<PersistedDownload> = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            downloads.iter().map(DownloadEntry::persisted).collect()
        };
        if let Err(error) = write_private_json(&path, &data) {
            tracing::error!("Failed to persist downloads: {error:#}");
        }
    }

    /// Restore the durable list as paused/completed entries on startup.
    pub fn restore_downloads(&self) {
        let config = self.config();
        let path = config.config_dir.join(DOWNLOADS_FILENAME);
        let Ok(contents) = fs::read_to_string(&path) else {
            return;
        };
        let Ok(entries) = serde_json::from_str::<Vec<PersistedDownload>>(&contents) else {
            tracing::warn!("Downloads file is corrupted; starting with an empty list");
            return;
        };
        if entries.is_empty() {
            return;
        }

        let history = self.history.history();
        let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
        for entry in entries {
            if downloads.iter().any(|download| download.id == DownloadId::new(entry.id)) {
                continue;
            }
            let resume_position = history
                .iter()
                .find(|history| history.url == entry.url)
                .and_then(|history| history.progress.get(&entry.label))
                .map(|progress| progress.position);
            let completed = entry.state == DownloadState::Completed;
            let id = DownloadId::new(entry.id);
            downloads.push(DownloadEntry {
                id,
                torrent_id: None,
                torrent_name: entry.torrent_name,
                label: entry.label,
                file_names: entry.file_names,
                url: entry.url,
                progress: if completed { 1.0 } else { entry.progress.clamp(0.0, 1.0) },
                speed: 0,
                upload_speed: 0,
                peers: 0,
                total_size: entry.total_size,
                downloaded: if completed {
                    entry.total_size
                } else {
                    entry.downloaded.min(entry.total_size)
                },
                speed_ema: 0.0,
                kind: if entry.is_streaming {
                    TransferKind::Stream
                } else {
                    TransferKind::Download
                },
                state: if completed { DownloadState::Completed } else { DownloadState::Paused },
                stream_path: None,
                subtitles: entry.subtitles,
                error: None,
                file_index: entry.file_index,
                file_indices: entry.file_indices,
                output_subfolder: entry.output_subfolder,
                queue: entry.queue,
                resume_position,
            });
            self.next_download_id.fetch_max(id.raw().saturating_add(1), Ordering::SeqCst);
        }
        drop(downloads);
        self.emit(GatewayEvent::DownloadsChanged);
    }
}
