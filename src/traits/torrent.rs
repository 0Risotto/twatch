//! `BitTorrent` operations interface.

use crate::model::{TorrentId, TorrentInfo, TorrentStats};
use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

#[async_trait]
pub trait TorrentService: Interface {
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

    /// Return the local HTTP stream URL for a file within a torrent.
    fn get_stream_url(&self, id: &TorrentId, file_idx: usize) -> Result<String>;

    /// Query download progress and speed for a running torrent.
    fn get_stats(&self, id: &TorrentId) -> Result<TorrentStats>;

    /// Spawn a boxed future on the internal runtime.
    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);
}
