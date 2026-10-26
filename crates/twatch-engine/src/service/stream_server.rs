//! Embedded chunked-HTTP streaming server.
//!
//! Serves file data from a librqbit [`Session`] on a loopback-only
//! ephemeral port so external players (mpv/vlc) can stream torrents
//! without waiting for the download to finish.

use anyhow::{Context, Result};
use librqbit::Session;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
