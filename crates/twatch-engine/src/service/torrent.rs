//! Default [`TorrentPort`](twatch_core::domain::torrent::ports::TorrentPort) and
//! [`StreamPort`](twatch_core::domain::torrent::ports::StreamPort) implementations.
//!
//! Wraps a librqbit `Session` with an embedded HTTP streaming server.

use anyhow::{Context, Result};
use async_trait::async_trait;
use librqbit::{AddTorrent, AddTorrentOptions, AddTorrentResponse, Session};
use shaku::{Component, Module, ModuleBuildContext};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use twatch_core::domain::torrent::TorrentId;
use twatch_core::domain::torrent::ports::{StreamPort, TorrentPort};
use twatch_core::domain::torrent::{TorrentInfo, TorrentStats};

/// The production torrent engine, backed by librqbit.
pub struct TorrentEngineImpl {
    pub session: Arc<Session>,
}

/// Shaku parameters for [`TorrentEngineImpl`].
#[derive(Default)]
pub struct TorrentEngineImplParameters {
    pub session: Option<Arc<Session>>,
}

impl<M: Module> Component<M> for TorrentEngineImpl {
    type Interface = dyn TorrentPort;
    type Parameters = TorrentEngineImplParameters;

    #[allow(clippy::expect_used)]
    fn build(
        _context: &mut ModuleBuildContext<M>,
        params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self {
            session: params.session.expect("TorrentEngineImpl: session parameter required"),
        })
    }
}

#[async_trait]
impl TorrentPort for TorrentEngineImpl {
    async fn preview(&self, url: &str) -> Result<TorrentInfo> {
        let response = self
            .session
            .add_torrent(
                AddTorrent::from_url(url),
                Some(AddTorrentOptions { list_only: true, ..Default::default() }),
            )
            .await
            .context("Failed to fetch torrent metadata")?;

        match response {
            AddTorrentResponse::ListOnly(resp) => {
                let name = String::from_utf8_lossy(resp.info.name.as_deref().unwrap_or(b"Unknown"))
                    .into_owned();
                let files: Vec<twatch_core::model::TorrentFile> =
                    if let Some(files_list) = &resp.info.files {
                        files_list
                            .iter()
                            .enumerate()
                            .map(|(i, f)| twatch_core::model::TorrentFile {
                                index: i,
                                name: f
                                    .path
                                    .iter()
                                    .map(|p| String::from_utf8_lossy(p.as_ref()))
                                    .collect::<Vec<_>>()
                                    .join("/"),
                                size: f.length,
                            })
                            .collect()
                    } else if let Some(length) = resp.info.length {
                        vec![twatch_core::model::TorrentFile {
                            index: 0,
                            name: name.clone(),
                            size: length,
                        }]
                    } else {
                        vec![]
                    };
                Ok(TorrentInfo { name, files })
            }
            _ => anyhow::bail!("Expected list-only response"),
        }
    }

    async fn download(&self, url: &str, file_idx: usize) -> Result<TorrentId> {
        self.add_torrent_with_opts(url, vec![file_idx], None).await
    }

    async fn download_with_extra(
        &self,
        url: &str,
        file_idx: usize,
        extra_indices: &[usize],
    ) -> Result<TorrentId> {
        let mut indices = Vec::with_capacity(extra_indices.len() + 1);
        indices.push(file_idx);
        indices.extend_from_slice(extra_indices);
        indices.sort_unstable();
        indices.dedup();
        self.add_torrent_with_opts(url, indices, None).await
    }

    async fn download_to_folder(
        &self,
        url: &str,
        indices: &[usize],
        dir: &Path,
    ) -> Result<TorrentId> {
        self.add_torrent_with_opts(url, indices.to_vec(), Some(dir)).await
    }

    fn get_stats(&self, id: &TorrentId) -> Result<TorrentStats> {
        let stats = self
            .session
            .with_torrents(|torrents| {
                for (tid, h) in torrents {
                    if tid == id.raw() {
                        return Some(h.stats());
                    }
                }
                None
            })
            .context("Torrent not found")?;
        let progress = if stats.total_bytes > 0 {
            stats.progress_bytes as f64 / stats.total_bytes as f64
        } else {
            0.0
        };

        // Speeds are MiB/s as f64; peers are the live peer count.
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let mib_to_bytes = |mbps: f64| (mbps * 1024.0 * 1024.0).max(0.0) as u64;
        #[allow(clippy::cast_sign_loss)]
        Ok(TorrentStats {
            progress,
            download_speed: stats
                .live
                .as_ref()
                .map(|l| mib_to_bytes(l.download_speed.mbps))
                .unwrap_or(0),
            upload_speed: stats
                .live
                .as_ref()
                .map(|l| mib_to_bytes(l.upload_speed.mbps))
                .unwrap_or(0),
            peers: stats.live.as_ref().map(|l| l.snapshot.peer_stats.live).unwrap_or(0),
            total_size: stats.total_bytes,
            downloaded: stats.progress_bytes,
        })
    }

    async fn pause(&self, id: &TorrentId) -> Result<()> {
        let handle = self
            .session
            .with_torrents(|torrents| {
                for (tid, h) in torrents {
                    if tid == id.raw() {
                        return Some(h.clone());
                    }
                }
                None
            })
            .context("Torrent not found")?;
        self.session.pause(&handle).await.context("Failed to pause torrent")
    }

    async fn resume(&self, id: &TorrentId) -> Result<()> {
        let handle = self
            .session
            .with_torrents(|torrents| {
                for (tid, h) in torrents {
                    if tid == id.raw() {
                        return Some(h.clone());
                    }
                }
                None
            })
            .context("Torrent not found")?;
        self.session.unpause(&handle).await.context("Failed to resume torrent")
    }

    async fn remove(&self, id: &TorrentId, delete_files: bool) -> Result<()> {
        self.session
            .delete(librqbit::api::TorrentIdOrHash::Id(id.raw()), delete_files)
            .await
            .context("Failed to remove torrent")
    }
}

/// Production [`StreamPort`] that resolves URLs on the embedded
/// HTTP streaming server.
pub struct StreamUrlProvider {
    pub http_port: u16,
}

/// Shaku parameters for [`StreamUrlProvider`].
#[derive(Default)]
pub struct StreamUrlProviderParameters {
    pub http_port: Option<u16>,
}

impl<M: Module> Component<M> for StreamUrlProvider {
    type Interface = dyn StreamPort;
    type Parameters = StreamUrlProviderParameters;

    #[allow(clippy::expect_used)]
    fn build(
        _context: &mut ModuleBuildContext<M>,
        params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self {
            http_port: params.http_port.expect("StreamUrlProvider: http_port parameter required"),
        })
    }
}

impl StreamPort for StreamUrlProvider {
    fn stream_url(&self, id: &TorrentId, file_idx: usize) -> String {
        format!("http://127.0.0.1:{}/stream/{}/{}", self.http_port, id.raw(), file_idx)
    }
}

impl TorrentEngineImpl {
    async fn add_torrent_with_opts(
        &self,
        url: &str,
        file_indices: Vec<usize>,
        output_dir: Option<&Path>,
    ) -> Result<TorrentId> {
        let only_files: HashSet<usize> = file_indices.iter().copied().collect();
        let mut opts = AddTorrentOptions {
            overwrite: true,
            only_files: Some(file_indices),
            ..Default::default()
        };

        if let Some(dir) = output_dir {
            opts.output_folder = Some(dir.display().to_string());
        }

        let response = self
            .session
            .add_torrent(AddTorrent::from_url(url), Some(opts))
            .await
            .context("Failed to add torrent")?;

        let (id, handle, already_managed) = match response {
            AddTorrentResponse::Added(id, handle) => (id, handle, false),
            AddTorrentResponse::AlreadyManaged(id, handle) => (id, handle, true),
            AddTorrentResponse::ListOnly(_) => anyhow::bail!("Unexpected list-only response"),
        };

        // If this torrent was already managed, `only_files` in the add call may
        // be ignored. Newly added torrents already received it and cannot be
        // updated until initialization completes.
        if already_managed
            && let Err(error) = self.session.update_only_files(&handle, &only_files).await
        {
            tracing::warn!("Could not update only_files for torrent {id}: {error:#}");
        }

        Ok(TorrentId::new(id))
    }
}
