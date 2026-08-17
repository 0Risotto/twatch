use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Mutex;
use twatch::domain::torrent::TorrentId;
use twatch::domain::torrent::ports::{StreamPort, TorrentPort};
use twatch::domain::torrent::{TorrentInfo, TorrentStats};

pub struct MockTorrentService {
    stats: Mutex<TorrentStats>,
}

impl MockTorrentService {
    pub const fn new() -> Self {
        Self {
            stats: Mutex::new(TorrentStats {
                progress: 0.0,
                download_speed: 0,
                upload_speed: 0,
                peers: 0,
                total_size: 0,
                downloaded: 0,
            }),
        }
    }

    /// Override the stats returned by [`TorrentPort::get_stats`].
    pub fn set_stats(&self, stats: TorrentStats) {
        *self.stats.lock().unwrap() = stats;
    }
}

#[async_trait]
impl TorrentPort for MockTorrentService {
    async fn preview(&self, _url: &str) -> Result<TorrentInfo> {
        Ok(TorrentInfo { name: "mock".into(), files: vec![] })
    }

    async fn download(&self, _url: &str, _file_idx: usize) -> Result<TorrentId> {
        Ok(TorrentId::new(0))
    }

    async fn download_to_folder(
        &self,
        _url: &str,
        _indices: &[usize],
        _dir: &Path,
    ) -> Result<TorrentId> {
        Ok(TorrentId::new(0))
    }

    fn get_stats(&self, _id: &TorrentId) -> Result<TorrentStats> {
        Ok(self.stats.lock().unwrap().clone())
    }
}

pub struct MockStreamPort;

impl MockStreamPort {
    pub const fn new() -> Self {
        Self
    }
}

impl StreamPort for MockStreamPort {
    fn stream_url(&self, _id: &TorrentId, file_idx: usize) -> String {
        format!("http://127.0.0.1:1/stream/0/{file_idx}")
    }
}
