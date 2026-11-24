use super::*;

impl Gateway {
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
}
