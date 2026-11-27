use super::*;

impl Gateway {
    // ── Player ───────────────────────────────────────────────────────

    pub fn open_player(
        &self,
        download_id: DownloadId,
        base_url: &str,
    ) -> GatewayResult<PlayerSnapshot> {
        self.open_player_with_token(download_id, base_url, None)
    }

    /// Open a download in a native player with an optional media query token.
    pub fn open_player_with_token(
        &self,
        download_id: DownloadId,
        base_url: &str,
        token: Option<&str>,
    ) -> GatewayResult<PlayerSnapshot> {
        let download = {
            let downloads = self.downloads.read().unwrap_or_else(|e| e.into_inner());
            downloads
                .iter()
                .find(|entry| entry.id == download_id)
                .map(DownloadEntry::snapshot)
                .ok_or(GatewayError::DownloadNotFound)?
        };
        let stream_path = download.stream_path.clone().ok_or(GatewayError::StreamNotReady)?;
        let url = stream_url(base_url, &stream_path, token);

        self.player.play(&url, &download.label);
        {
            let mut state = self.player_state.lock().unwrap_or_else(|e| e.into_inner());
            state.title = Some(download.label);
            state.stream_path = Some(stream_path);
            state.download_id = Some(download_id.raw());
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
