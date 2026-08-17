//! Torrent engine and stream server ports.

use super::{TorrentId, TorrentInfo, TorrentStats};
use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;
use std::path::Path;

/// Downloads and inspects torrents through the engine adapter.
#[async_trait]
pub trait TorrentPort: Interface {
    /// Fetch a torrent's metadata without downloading any data.
    async fn preview(&self, url: &str) -> Result<TorrentInfo>;

    /// Download a single file by index for streaming.
    async fn download(&self, url: &str, file_idx: usize) -> Result<TorrentId>;

    /// Download selected files to a directory on disk.
    async fn download_to_folder(
        &self,
        url: &str,
        indices: &[usize],
        dir: &Path,
    ) -> Result<TorrentId>;

    /// Query download progress and speed for a running torrent.
    fn get_stats(&self, id: &TorrentId) -> Result<TorrentStats>;
}

/// Resolves local HTTP stream URLs served by the embedded stream server.
pub trait StreamPort: Interface {
    /// The stream URL for a file within a running torrent.
    fn stream_url(&self, id: &TorrentId, file_idx: usize) -> String;
}
