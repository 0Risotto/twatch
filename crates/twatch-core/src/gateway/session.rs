use super::*;

impl Gateway {
    /// Fetch metadata, remember it, and make it the current session.
    pub async fn preview(&self, url: &str) -> GatewayResult<SessionSnapshot> {
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
                progress: std::collections::BTreeMap::new(),
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
    pub async fn open_history(&self, url: &str) -> GatewayResult<SessionSnapshot> {
        self.preview(url).await
    }

    /// Close the current browse session.  Live downloads are unaffected.
    pub fn close_session(&self) {
        *self.session.write().unwrap_or_else(|e| e.into_inner()) = None;
        self.emit(GatewayEvent::SessionChanged);
    }

    /// Select or deselect files by index.
    pub fn select_files(&self, indices: &[usize], selected: bool) -> GatewayResult<()> {
        {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_mut().ok_or(GatewayError::NoSession)?;
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
    pub fn select_all(&self, selected: bool) -> GatewayResult<()> {
        {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_mut().ok_or(GatewayError::NoSession)?;
            session.selected.fill(selected);
        }
        self.emit(GatewayEvent::SessionChanged);
        Ok(())
    }
}
