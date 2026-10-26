//! Range-capable file streaming over a librqbit session.
//!
//! `librqbit` exposes its `FileStream` through `Api::api_stream`, but does not
//! re-export the concrete type.  We return `impl AsyncRead + AsyncSeek` instead
//! and let the HTTP layer turn it into a body.  This keeps the adapter boundary
//! narrow and gives the browser proper `206 Partial Content` support.

use anyhow::Result;
use librqbit::Api;
use librqbit::api::TorrentIdOrHash;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncSeek};

use twatch_core::domain::torrent::TorrentId;

/// Shared streaming facade over one librqbit session.
#[derive(Clone)]
pub struct StreamService {
    session: Arc<librqbit::Session>,
}

impl StreamService {
    #[must_use]
    pub const fn new(session: Arc<librqbit::Session>) -> Self {
        Self { session }
    }

    /// Open a seekable stream for one file in a managed torrent.
    pub fn open(
        &self,
        torrent_id: TorrentId,
        file_idx: usize,
    ) -> Result<impl AsyncRead + AsyncSeek + Unpin + Send + 'static> {
        let api = Api::new(self.session.clone(), None);
        Ok(api.api_stream(TorrentIdOrHash::Id(torrent_id.raw()), file_idx)?)
    }

    /// Best-effort MIME type from the torrent file name.
    #[must_use]
    pub fn mime(&self, torrent_id: TorrentId, file_idx: usize) -> Option<&'static str> {
        let api = Api::new(self.session.clone(), None);
        api.torrent_file_mime_type(TorrentIdOrHash::Id(torrent_id.raw()), file_idx).ok()
    }

    /// Best-effort file name from the torrent metadata.
    #[must_use]
    pub fn file_name(&self, torrent_id: TorrentId, file_idx: usize) -> Option<String> {
        self.session.with_torrents(|torrents| {
            for (tid, handle) in torrents {
                if tid == torrent_id.raw() {
                    return handle
                        .with_metadata(|metadata| {
                            if let Some(files) = metadata.info.files.as_ref() {
                                files.get(file_idx).map(|file| {
                                    file.path
                                        .iter()
                                        .map(|part| {
                                            String::from_utf8_lossy(part.as_ref()).into_owned()
                                        })
                                        .collect::<Vec<_>>()
                                        .join("/")
                                })
                            } else {
                                metadata
                                    .info
                                    .name
                                    .as_ref()
                                    .map(|name| String::from_utf8_lossy(name.as_ref()).into_owned())
                            }
                        })
                        .ok()
                        .flatten();
                }
            }
            None
        })
    }

    /// Total length of a file, without keeping a stream alive.
    pub async fn len(&self, torrent_id: TorrentId, file_idx: usize) -> Result<u64> {
        use tokio::io::{AsyncSeekExt, SeekFrom};
        let mut stream = self.open(torrent_id, file_idx)?;
        Ok(stream.seek(SeekFrom::End(0)).await?)
    }

    /// The underlying session, for torrent control (pause/resume/delete).
    #[must_use]
    pub const fn session(&self) -> &Arc<librqbit::Session> {
        &self.session
    }
}
