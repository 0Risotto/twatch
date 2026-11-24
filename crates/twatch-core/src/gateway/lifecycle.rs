use super::*;

impl Gateway {
    pub async fn pause_download(&self, id: u64) -> Result<()> {
        let torrent_id = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            downloads.iter().find(|entry| entry.id == id).context("Download not found")?.torrent_id
        };
        if let Some(torrent_id) = torrent_id {
            self.torrent.pause(&torrent_id).await?;
        }
        {
            let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = downloads.iter_mut().find(|entry| entry.id == id) {
                entry.state = DownloadState::Paused;
                entry.speed = 0;
                entry.speed_ema = 0.0;
            }
        }
        self.emit(GatewayEvent::DownloadsChanged);
        self.persist_downloads();
        Ok(())
    }

    pub async fn resume_download(&self, id: u64) -> Result<()> {
        let (torrent_id, is_streaming, url, file_index, file_indices, subtitles, output_subfolder) = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            let entry =
                downloads.iter().find(|entry| entry.id == id).context("Download not found")?;
            (
                entry.torrent_id,
                entry.is_streaming,
                entry.url.clone(),
                entry.file_index,
                entry.file_indices.clone(),
                entry.subtitles.clone(),
                entry.output_subfolder.clone(),
            )
        };

        if let Some(torrent_id) = torrent_id {
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
            self.persist_downloads();
            return Ok(());
        }

        // A restored entry has no live torrent yet. Re-add it on demand.
        {
            let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = downloads.iter_mut().find(|entry| entry.id == id) {
                entry.state = DownloadState::Starting;
                entry.error = None;
            }
        }
        self.emit(GatewayEvent::DownloadsChanged);
        self.persist_downloads();

        let config = self.config();
        let output_dir = match output_subfolder {
            Some(folder) => config.download_dir.join(folder),
            None => config.download_dir,
        };
        let torrent = self.torrent.clone();
        let tx = self.internal_tx.clone();
        self.spawner.spawn(Box::pin(async move {
            if is_streaming {
                let subtitle_indices: Vec<usize> =
                    subtitles.iter().map(|track| track.index).collect();
                match torrent.download_with_extra(&url, file_index, &subtitle_indices).await {
                    Ok(torrent_id) => {
                        let path = format!("/api/stream/{}/{}", torrent_id.raw(), file_index);
                        let _ = tx.send(InternalEvent::Ready {
                            id,
                            torrent_id,
                            stream_path: Some(path),
                        });
                    }
                    Err(error) => {
                        let _ = tx.send(InternalEvent::Failed {
                            id,
                            message: format!("Stream failed: {error}"),
                        });
                    }
                }
            } else {
                match torrent.download_to_folder(&url, &file_indices, &output_dir).await {
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
            }
        }));

        Ok(())
    }

    pub async fn remove_download(&self, id: u64) -> Result<()> {
        let entry = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            downloads
                .iter()
                .find(|entry| entry.id == id)
                .map(|entry| (entry.torrent_id, entry.url.clone(), entry.file_names.clone()))
        };
        let Some((torrent_id, url, file_names)) = entry else {
            return Ok(());
        };
        if let Some(torrent_id) = torrent_id {
            self.torrent.remove(&torrent_id, false).await?;
        } else {
            // The add task may still be running. Remember the id so the
            // eventual Ready event can remove the engine torrent.
            self.cancelled.lock().unwrap_or_else(|e| e.into_inner()).insert(id);
        }
        self.downloads.write().unwrap_or_else(|e| e.into_inner()).retain(|entry| entry.id != id);

        let session_changed = {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            match guard.as_mut() {
                Some(session) if session.url == url => {
                    let before = session.downloading_files.len();
                    session.downloading_files.retain(|name| !file_names.contains(name));
                    session.downloading_files.len() != before
                }
                _ => false,
            }
        };
        if session_changed {
            self.emit(GatewayEvent::SessionChanged);
        }

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
        self.persist_downloads();
        Ok(())
    }
}
