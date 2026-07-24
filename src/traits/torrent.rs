//! `BitTorrent` operations interface.
//!
//! This trait abstracts the underlying librqbit engine so the rest of
//! the application never touches the torrent library directly.

use crate::model::{TorrentId, TorrentInfo, TorrentStats};
use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

#[async_trait]
pub trait TorrentService: Interface {
    /// Fetch a torrent's metadata (name + file list) without downloading any data.
    async fn preview(&self, url: &str) -> Result<TorrentInfo>;

    /// Download a single file by index for streaming.
    async fn download(&self, url: &str, file_idx: usize) -> Result<TorrentId>;

    /// Download selected files to a specific directory on disk.
    async fn download_to_folder(
        &self,
        url: &str,
        indices: &[usize],
        dir: &Path,
    ) -> Result<TorrentId>;

    /// Build the local HTTP stream URL that mpv/vlc will read from.
    fn get_stream_url(&self, id: &TorrentId, file_idx: usize) -> Result<String>;

    /// Query download progress / speed for a running torrent.
    fn get_stats(&self, id: &TorrentId) -> Result<TorrentStats>;

    /// Spawn a boxed future on the internal tokio runtime.
    ///
    /// Exists as `spawn_boxed` (instead of a generic `spawn`) because
    /// generic methods are not object-safe on trait objects.
    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);
}
