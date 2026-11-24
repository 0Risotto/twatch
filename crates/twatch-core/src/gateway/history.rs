use super::*;

impl Gateway {
    #[must_use]
    pub fn history(&self) -> Vec<HistoryEntry> {
        self.history.history()
    }

    pub fn rename_history(&self, url: &str, name: &str) -> Result<()> {
        let index = self
            .history
            .history()
            .iter()
            .position(|entry| entry.url == url)
            .context("History entry not found")?;
        self.history.rename_entry(index, name)
    }

    pub fn delete_history(&self, url: &str) -> Result<()> {
        let index = self
            .history
            .history()
            .iter()
            .position(|entry| entry.url == url)
            .context("History entry not found")?;
        self.history.remove_entry(index)
    }

    /// Persist the resume position for one file in the history entry.
    pub fn save_progress(&self, url: &str, file_name: &str, position: f64, duration: f64) {
        if !position.is_finite() || !duration.is_finite() || position < 0.0 || duration <= 0.0 {
            return;
        }
        let position = position.min(duration);
        self.history.set_progress(url, file_name, position, duration);

        // Keep the live download snapshot in sync so a page reload can restore
        // playback without waiting for a new stream registration.
        let mut changed = false;
        {
            let mut downloads = self.downloads.write().unwrap_or_else(|e| e.into_inner());
            for entry in downloads.iter_mut() {
                if entry.url == url
                    && entry.label == file_name
                    && entry.resume_position != Some(position)
                {
                    entry.resume_position = Some(position);
                    changed = true;
                }
            }
        }
        if changed {
            self.emit(GatewayEvent::DownloadsChanged);
        }
    }
}
