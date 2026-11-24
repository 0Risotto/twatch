use super::*;

impl Gateway {
    // ── Files ────────────────────────────────────────────────────────

    /// Delete selected files from disk and sync status/history.
    pub fn delete_files(&self, indices: &[usize]) -> DeleteReport {
        let download_dir = self.config().download_dir;
        let (url, files, root) = {
            let guard = self.session.read().unwrap_or_else(|e| e.into_inner());
            let Some(session) = guard.as_ref() else {
                return DeleteReport::default();
            };
            let files: Vec<(usize, TorrentFile)> = indices
                .iter()
                .filter_map(|index| session.files.get(*index).cloned().map(|f| (*index, f)))
                .collect();
            (session.url.clone(), files, session.download_root_dir(&download_dir))
        };

        let mut report = DeleteReport::default();
        let mut removed_names = Vec::new();
        for (_, file) in files {
            let path = root.join(&file.name);
            if !path.starts_with(&root) {
                report.failed.push(file.name);
                continue;
            }
            match std::fs::remove_file(&path) {
                Ok(()) => {
                    report.deleted += 1;
                    removed_names.push(file.name.clone());
                    self.history.mark_deleted(&url, &file.name);
                }
                Err(error) => {
                    tracing::warn!("Failed to delete {}: {error}", file.name);
                    report.failed.push(file.name);
                }
            }
        }

        if !removed_names.is_empty() {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            if let Some(session) = guard.as_mut() {
                session.watched_files.retain(|f| !removed_names.contains(f));
                session.downloaded_files.retain(|f| !removed_names.contains(f));
                session.downloading_files.retain(|f| !removed_names.contains(f));
                for index in indices {
                    if let Some(slot) = session.selected.get_mut(*index) {
                        *slot = false;
                    }
                }
            }
            drop(guard);
            self.emit(GatewayEvent::SessionChanged);
        }
        report
    }

    /// Rename one file on disk and in the session/history.
    pub fn rename_file(&self, index: usize, new_name: &str) -> GatewayResult<String> {
        if new_name.is_empty() || new_name.len() > 256 {
            return Err(GatewayError::InvalidFileNameLength);
        }
        if new_name.contains('/') || new_name.contains('\\') || new_name.contains('\0') {
            return Err(GatewayError::InvalidFileNameCharacters);
        }

        let download_dir = self.config().download_dir;
        let (url, old_name, old_path, new_path, root) = {
            let guard = self.session.read().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_ref().ok_or(GatewayError::NoSession)?;
            let old = session.files.get(index).ok_or(GatewayError::FileNotFound)?.clone();
            let root = session.download_root_dir(&download_dir);
            let old_path = root.join(&old.name);
            let new_path = root.join(new_name);
            (session.url.clone(), old.name, old_path, new_path, root)
        };

        if !new_path.starts_with(&root) {
            return Err(GatewayError::InvalidFileName);
        }

        if old_path.exists() {
            if let Some(parent) = new_path.parent() {
                std::fs::create_dir_all(parent).map_err(anyhow::Error::from)?;
            }
            std::fs::rename(&old_path, &new_path).map_err(anyhow::Error::from)?;
        }

        {
            let mut guard = self.session.write().unwrap_or_else(|e| e.into_inner());
            let session = guard.as_mut().ok_or(GatewayError::NoSession)?;
            if let Some(file) = session.files.iter_mut().find(|file| file.index == index) {
                file.name = new_name.to_owned();
            }
            for list in [
                &mut session.watched_files,
                &mut session.downloaded_files,
                &mut session.downloading_files,
            ] {
                for name in list.iter_mut() {
                    if *name == old_name {
                        *name = new_name.to_owned();
                    }
                }
            }
        }

        self.history.rename_file(&url, &old_name, new_name);
        self.emit(GatewayEvent::SessionChanged);
        Ok(format!("Renamed to: {new_name}"))
    }
}
