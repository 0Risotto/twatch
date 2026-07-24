//! Default [`TorrentService`](crate::traits::TorrentService) implementation.
//!
//! Wraps a librqbit `Session` with an embedded HTTP streaming server.

use crate::model::{TorrentId, TorrentInfo, TorrentStats};
use crate::traits::TorrentService;
use anyhow::{Context, Result};
use async_trait::async_trait;
use librqbit::{AddTorrent, AddTorrentOptions, AddTorrentResponse, Session};
use shaku::{Component, Module, ModuleBuildContext};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The production torrent engine, backed by librqbit.
pub struct TorrentEngineImpl {
    pub rt: tokio::runtime::Runtime,
    pub session: Arc<Session>,
    pub http_port: u16,
}

/// Shaku parameters for [`TorrentEngineImpl`].
///
/// All three fields must be set via [`AppModule::builder`].
#[derive(Default)]
pub struct TorrentEngineImplParameters {
    pub rt: Option<tokio::runtime::Runtime>,
    pub session: Option<Arc<Session>>,
    pub http_port: Option<u16>,
}

impl<M: Module> Component<M> for TorrentEngineImpl {
    type Interface = dyn TorrentService;
    type Parameters = TorrentEngineImplParameters;

    #[allow(clippy::expect_used)]
    fn build(
        _context: &mut ModuleBuildContext<M>,
        params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self {
            rt: params.rt.expect("TorrentEngineImpl: rt parameter required"),
            session: params.session.expect("TorrentEngineImpl: session parameter required"),
            http_port: params.http_port.expect("TorrentEngineImpl: http_port parameter required"),
        })
    }
}

#[async_trait]
impl TorrentService for TorrentEngineImpl {
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
                let files: Vec<crate::model::TorrentFile> = resp
                    .info
                    .files
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .enumerate()
                    .map(|(i, f)| crate::model::TorrentFile {
                        index: i,
                        name: f
                            .path
                            .iter()
                            .map(|p| String::from_utf8_lossy(p.as_ref()))
                            .collect::<Vec<_>>()
                            .join("/"),
                        size: f.length,
                    })
                    .collect();
                Ok(TorrentInfo { name, files })
            }
            _ => anyhow::bail!("Expected list-only response"),
        }
    }

    async fn download(&self, url: &str, file_idx: usize) -> Result<TorrentId> {
        self.add_torrent_with_opts(url, vec![file_idx], None).await
    }

    async fn download_to_folder(
        &self,
        url: &str,
        indices: &[usize],
        dir: &Path,
    ) -> Result<TorrentId> {
        self.add_torrent_with_opts(url, indices.to_vec(), Some(dir)).await
    }

    fn get_stream_url(&self, id: &TorrentId, file_idx: usize) -> Result<String> {
        Ok(format!("http://127.0.0.1:{}/stream/{}/{}", self.http_port, id.0, file_idx))
    }

    fn get_stats(&self, id: &TorrentId) -> Result<TorrentStats> {
        let stats = self
            .session
            .with_torrents(|torrents| {
                for (tid, h) in torrents {
                    if tid == id.0 {
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

        #[allow(clippy::cast_sign_loss)]
        Ok(TorrentStats {
            progress,
            download_speed: stats
                .live
                .as_ref()
                .map(|l| format!("{}", l.download_speed).parse().unwrap_or(0))
                .unwrap_or(0),
            upload_speed: stats
                .live
                .as_ref()
                .map(|l| format!("{}", l.upload_speed).parse().unwrap_or(0))
                .unwrap_or(0),
            peers: 0,
            total_size: stats.total_bytes,
            downloaded: stats.progress_bytes,
        })
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.rt.spawn(future);
    }
}

impl TorrentEngineImpl {
    async fn add_torrent_with_opts(
        &self,
        url: &str,
        file_indices: Vec<usize>,
        output_dir: Option<&Path>,
    ) -> Result<TorrentId> {
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

        let id = match response {
            AddTorrentResponse::Added(id, _) => id,
            AddTorrentResponse::AlreadyManaged(id, _) => id,
            AddTorrentResponse::ListOnly(_) => anyhow::bail!("Unexpected list-only response"),
        };

        Ok(TorrentId(id))
    }
}

// ---------------------------------------------------------------------------
// Embedded HTTP streaming server
// ---------------------------------------------------------------------------

/// Maximum concurrent HTTP stream connections.
const MAX_CONNECTIONS: usize = 16;

/// Maximum request path length (prevents split-collect memory exhaustion).
const MAX_PATH_LEN: usize = 256;

/// HTTP read timeout.
const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Boot the chunked-HTTP stream server on an ephemeral port.
///
/// Returns the bound port number.  The server runs in a background
/// tokio task for the lifetime of the `Session`.
pub async fn start_stream_server(session: Arc<Session>) -> Result<u16> {
    use tokio::net::TcpListener;

    let listener =
        TcpListener::bind("127.0.0.1:0").await.context("Failed to bind stream server")?;
    let port = listener.local_addr()?.port();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONNECTIONS));

    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let session = session.clone();
                let permit = semaphore.clone().acquire_owned().await;
                tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(e) = handle_stream(stream, session).await {
                        tracing::error!("Stream request error: {e:#}");
                    }
                });
            }
        }
    });

    Ok(port)
}

/// Parse an HTTP request, look up `torrent_id` / `file_idx` from the
/// URL path, and stream the file data back as chunked transfer encoding.
async fn handle_stream(
    mut client: impl AsyncReadExt + AsyncWriteExt + Unpin,
    session: Arc<Session>,
) -> Result<()> {
    let mut buf = vec![0u8; 4096];
    let n = tokio::time::timeout(READ_TIMEOUT, client.read(&mut buf))
        .await
        .context("read timed out")?
        .context("read failed")?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let path =
        request.lines().next().and_then(|line| line.split_whitespace().nth(1)).unwrap_or("/");

    if path.len() > MAX_PATH_LEN {
        client.write_all(b"HTTP/1.1 414 URI Too Long\r\nContent-Length: 0\r\n\r\n").await?;
        return Ok(());
    }

    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() < 3 || parts[0] != "stream" {
        client.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n").await?;
        return Ok(());
    }

    let torrent_id: usize = parts[1].parse().context("Invalid torrent ID")?;
    let file_idx: usize = parts[2].parse().context("Invalid file index")?;

    let handle = session
        .with_torrents(|torrents| {
            for (tid, h) in torrents {
                if tid == torrent_id {
                    return Some(h.clone());
                }
            }
            None
        })
        .context("Torrent not found")?;

    let mut file_stream = handle.stream(file_idx).context("Failed to open file stream")?;

    client
        .write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\n\r\n",
        )
        .await?;

    let mut file_buf = vec![0u8; 65536];

    loop {
        let n = file_stream.read(&mut file_buf).await?;
        if n == 0 {
            break;
        }

        let chunk_header = format!("{n:X}\r\n");
        client.write_all(chunk_header.as_bytes()).await?;
        client.write_all(&file_buf[..n]).await?;
        client.write_all(b"\r\n").await?;
    }

    client.write_all(b"0\r\n\r\n").await?;
    Ok(())
}
