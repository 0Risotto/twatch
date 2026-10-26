//! Torrent engine and stream server ports.

use super::{TorrentId, TorrentInfo, TorrentStats};
use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;
use std::path::Path;

/// Downloads and inspects torrents through the engine adapter.
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait TorrentPort: Interface {
    /// Fetch a torrent's metadata without downloading any data.
    async fn preview(&self, url: &str) -> Result<TorrentInfo>;

    /// Download a single file by index for streaming.
    async fn download(&self, url: &str, file_idx: usize) -> Result<TorrentId>;

    /// Download one stream file plus companion files (subtitle sidecars).
    /// Adapters that cannot select extra files may keep the default.
    async fn download_with_extra(
        &self,
        url: &str,
        file_idx: usize,
        _extra_indices: &[usize],
    ) -> Result<TorrentId> {
        self.download(url, file_idx).await
    }

    /// Download selected files to a directory on disk.
    async fn download_to_folder(
        &self,
        url: &str,
        indices: &[usize],
        dir: &Path,
    ) -> Result<TorrentId>;

    /// Query download progress and speed for a running torrent.
    fn get_stats(&self, id: &TorrentId) -> Result<TorrentStats>;

    /// Pause a torrent.  Adapters that cannot pause may keep the default.
    async fn pause(&self, _id: &TorrentId) -> Result<()> {
        Ok(())
    }

    /// Resume a paused torrent.
    async fn resume(&self, _id: &TorrentId) -> Result<()> {
        Ok(())
    }

    /// Remove a torrent from the engine.  `delete_files` is intentionally
    /// separate from twatch's own safe file deletion path.
    async fn remove(&self, _id: &TorrentId, _delete_files: bool) -> Result<()> {
        Ok(())
    }
}

/// Resolves local HTTP stream URLs served by the embedded stream server.
pub trait StreamPort: Interface {
    /// The stream URL for a file within a running torrent.
    fn stream_url(&self, id: &TorrentId, file_idx: usize) -> String;
}
