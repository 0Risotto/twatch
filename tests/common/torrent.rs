use anyhow::Result;
use async_trait::async_trait;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use twatch::model::{TorrentId, TorrentInfo, TorrentStats};
use twatch::traits::TorrentService;

pub struct MockTorrentService {
    rt: tokio::runtime::Runtime,
}

impl MockTorrentService {
    pub fn new() -> Self {
        Self { rt: tokio::runtime::Runtime::new().unwrap() }
    }
}

#[async_trait]
impl TorrentService for MockTorrentService {
    async fn preview(&self, _url: &str) -> Result<TorrentInfo> {
        Ok(TorrentInfo { name: "mock".into(), files: vec![] })
    }

    async fn download(&self, _url: &str, _file_idx: usize) -> Result<TorrentId> {
        Ok(TorrentId(0))
    }

    async fn download_to_folder(
        &self,
        _url: &str,
        _indices: &[usize],
        _dir: &Path,
    ) -> Result<TorrentId> {
        Ok(TorrentId(0))
    }

    fn get_stream_url(&self, _id: &TorrentId, _file_idx: usize) -> Result<String> {
        Ok("http://127.0.0.1:1/stream/0/0".into())
    }

    fn get_stats(&self, _id: &TorrentId) -> Result<TorrentStats> {
        Ok(TorrentStats {
            progress: 0.0,
            download_speed: 0,
            upload_speed: 0,
            peers: 0,
            total_size: 0,
            downloaded: 0,
        })
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.rt.block_on(future);
    }
}
