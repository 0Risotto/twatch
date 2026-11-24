use super::*;

impl Gateway {
    // ── Downloads ────────────────────────────────────────────────────

    /// Start downloading one file for streaming.
    pub fn watch_file(&self, index: usize) -> Result<DownloadSnapshot> {
        let (url, torrent_name, file, subtitles, queue) = {
            let guard = self.session.read().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_ref().context("No active torrent session")?;
            let file = session.files.get(index).context("File not found")?.clone();
            let subtitles = collect_subtitles(&session.files);
            let queue = playable_queue(&session.files);
            (session.url.clone(), session.name.clone(), file, subtitles, queue)
        };
        let resume_position = self
            .history
            .history()
            .iter()
            .find(|entry| entry.url == url)
            .and_then(|entry| entry.progress.get(&file.name))
            .map(|progress| progress.position);

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
            file_index: file.index,
            file_indices: Vec::new(),
            output_subfolder: None,
            queue,
            resume_position,
        };
        let snapshot = entry.snapshot();
        self.downloads.write().unwrap_or_else(|e| e.into_inner()).push(entry);
        self.emit(GatewayEvent::DownloadsChanged);
        self.persist_downloads();

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
        let (url, torrent_name, output_dir, output_subfolder, file_names, total_size, subtitles) = {
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
                session.download_root().map(ToOwned::to_owned),
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
            file_index: 0,
            file_indices: indices.to_vec(),
            output_subfolder,
            queue: Vec::new(),
            resume_position: None,
        };
        let snapshot = entry.snapshot();
        self.downloads.write().unwrap_or_else(|e| e.into_inner()).push(entry);
        self.emit(GatewayEvent::DownloadsChanged);
        self.persist_downloads();

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
        let recheck = file_names.iter().any(|name| !output_dir.join(name).exists());
        self.spawner.spawn(Box::pin(async move {
            match torrent.download_to_folder_rechecked(&url, &indices, &output_dir, recheck).await {
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
}
