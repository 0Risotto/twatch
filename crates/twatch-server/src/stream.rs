//! Range-capable `/api/stream/{torrentId}/{fileIndex}` handler.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, SeekFrom};
use tokio::time::{Duration, sleep};
use tokio_util::io::ReaderStream;

use twatch_core::domain::torrent::TorrentId;
use twatch_engine::stream::StreamService;

use crate::state::AppState;

const STREAM_INIT_ATTEMPTS: u8 = 200;

/// Open a file stream, waiting while a freshly added torrent initializes.
pub(crate) async fn open_stream(
    stream_service: &StreamService,
    torrent: TorrentId,
    file_idx: usize,
) -> Result<impl AsyncRead + AsyncSeek + Unpin + Send + 'static, (StatusCode, &'static str)> {
    for attempt in 0..STREAM_INIT_ATTEMPTS {
        match stream_service.open(torrent, file_idx) {
            Ok(stream) => return Ok(stream),
            Err(error) => {
                let initializing = format!("{error:#}").contains("initializing");
                if !initializing {
                    tracing::warn!(
                        "Stream open failed for {}/{file_idx}: {error:#}",
                        torrent.raw()
                    );
                    return Err((StatusCode::NOT_FOUND, "Torrent file not found"));
                }
                if attempt + 1 == STREAM_INIT_ATTEMPTS {
                    tracing::warn!(
                        "Stream still initializing for {}/{file_idx}: {error:#}",
                        torrent.raw()
                    );
                    return Err((StatusCode::SERVICE_UNAVAILABLE, "Torrent is still initializing"));
                }
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
    Err((StatusCode::SERVICE_UNAVAILABLE, "Torrent is still initializing"))
}

/// Inclusive byte range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    #[must_use]
    pub const fn len(self) -> u64 {
        self.end - self.start + 1
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }
}

/// Serve a file from the local cache or downloads folder.
///
/// The id is resolved through the gateway's library index; the request never
/// supplies a filesystem path directly.
pub async fn library_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let Some(path) = state.gateway.library_file_path(&id) else {
        return (StatusCode::NOT_FOUND, "Library file not found").into_response();
    };
    let mut file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(error) => {
            tracing::warn!("Library stream open failed: {error:#}");
            return (StatusCode::NOT_FOUND, "Library file not found").into_response();
        }
    };
    let len = match file.metadata().await {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            tracing::warn!("Library stream metadata failed: {error:#}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Could not read file").into_response();
        }
    };

    let range_header = headers.get(header::RANGE).and_then(|value| value.to_str().ok());
    let range = match parse_range(range_header, len) {
        Ok(range) => range,
        Err(RangeError) => {
            let mut response_headers = HeaderMap::new();
            response_headers.insert(header::CONTENT_RANGE, header_value(format!("bytes */{len}")));
            return (StatusCode::RANGE_NOT_SATISFIABLE, response_headers).into_response();
        }
    };

    let (start, end, status) = match range {
        Some(range) => (range.start, range.end, StatusCode::PARTIAL_CONTENT),
        None => (0, len.saturating_sub(1), StatusCode::OK),
    };
    let content_len = if len == 0 { 0 } else { end - start + 1 };

    if let Err(error) = file.seek(SeekFrom::Start(start)).await {
        tracing::warn!("Library stream seek failed: {error:#}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Could not seek file").into_response();
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response_headers.insert(header::CONTENT_LENGTH, header_value(content_len));
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response_headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("inline"));
    response_headers.insert(
        header::CONTENT_TYPE,
        header_value(mime_guess::from_path(&path).first_or_octet_stream()),
    );
    if status == StatusCode::PARTIAL_CONTENT {
        response_headers
            .insert(header::CONTENT_RANGE, header_value(format!("bytes {start}-{end}/{len}")));
    }

    if method == Method::HEAD {
        return (status, response_headers).into_response();
    }

    let limited = file.take(content_len);
    let body = Body::from_stream(ReaderStream::with_capacity(limited, 64 * 1024));
    (status, response_headers, body).into_response()
}

/// The range header was malformed or unsatisfiable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeError;

impl std::fmt::Display for RangeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("invalid byte range")
    }
}

impl std::error::Error for RangeError {}

/// Parse a single `Range: bytes=...` header.
///
/// Returns `Ok(None)` when there is no range, `Err(())` when it is malformed or
/// unsatisfiable.  Multiple ranges are intentionally rejected: browsers and
/// players use a single range, and a multipart response buys us nothing.
pub fn parse_range(header: Option<&str>, len: u64) -> Result<Option<ByteRange>, RangeError> {
    let Some(value) = header else {
        return Ok(None);
    };
    let value = value.trim();
    let Some(spec) = value.strip_prefix("bytes=") else {
        return Err(RangeError);
    };
    if spec.contains(',') {
        return Err(RangeError);
    }
    let (start_s, end_s) = spec.split_once('-').ok_or(RangeError)?;
    if len == 0 {
        return Err(RangeError);
    }

    let (start, end) = if start_s.is_empty() {
        let suffix: u64 = end_s.parse().map_err(|_| RangeError)?;
        if suffix == 0 {
            return Err(RangeError);
        }
        (len.saturating_sub(suffix), len - 1)
    } else {
        let start: u64 = start_s.parse().map_err(|_| RangeError)?;
        if start >= len {
            return Err(RangeError);
        }
        let end = if end_s.is_empty() {
            len - 1
        } else {
            let end: u64 = end_s.parse().map_err(|_| RangeError)?;
            if end < start {
                return Err(RangeError);
            }
            end.min(len - 1)
        };
        (start, end)
    };

    Ok(Some(ByteRange { start, end }))
}

fn header_value(value: impl ToString) -> HeaderValue {
    HeaderValue::from_str(&value.to_string())
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"))
}

/// Serve a torrent file with `Accept-Ranges`, `HEAD`, and `206` support.
pub async fn stream_file(
    State(state): State<AppState>,
    Path((torrent_id, file_idx)): Path<(usize, usize)>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let Some(stream_service) = &state.stream else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Stream engine unavailable").into_response();
    };
    let torrent = TorrentId::new(torrent_id);
    let mut stream = match open_stream(stream_service, torrent, file_idx).await {
        Ok(stream) => stream,
        Err((status, message)) => return (status, message).into_response(),
    };

    let len = match stream.seek(SeekFrom::End(0)).await {
        Ok(position) => position,
        Err(error) => {
            tracing::warn!("Stream length failed for {torrent_id}/{file_idx}: {error:#}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Could not read torrent").into_response();
        }
    };

    let range_header = headers.get(header::RANGE).and_then(|value| value.to_str().ok());
    let range = match parse_range(range_header, len) {
        Ok(range) => range,
        Err(RangeError) => {
            let mut response_headers = HeaderMap::new();
            response_headers.insert(header::CONTENT_RANGE, header_value(format!("bytes */{len}")));
            return (StatusCode::RANGE_NOT_SATISFIABLE, response_headers).into_response();
        }
    };

    let (start, end, status) = match range {
        Some(range) => (range.start, range.end, StatusCode::PARTIAL_CONTENT),
        None => (0, len.saturating_sub(1), StatusCode::OK),
    };
    let content_len = if len == 0 { 0 } else { end - start + 1 };

    if let Err(error) = stream.seek(SeekFrom::Start(start)).await {
        tracing::warn!("Stream seek failed for {torrent_id}/{file_idx}: {error:#}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Could not seek torrent").into_response();
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response_headers.insert(header::CONTENT_LENGTH, header_value(content_len));
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response_headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("inline"));
    if let Some(mime) = stream_service.mime(torrent, file_idx) {
        response_headers.insert(header::CONTENT_TYPE, header_value(mime));
    } else {
        response_headers
            .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    }
    if status == StatusCode::PARTIAL_CONTENT {
        response_headers
            .insert(header::CONTENT_RANGE, header_value(format!("bytes {start}-{end}/{len}")));
    }

    if method == Method::HEAD {
        return (status, response_headers).into_response();
    }

    let limited = stream.take(content_len);
    let body = Body::from_stream(ReaderStream::with_capacity(limited, 64 * 1024));
    (status, response_headers, body).into_response()
}

#[cfg(test)]
mod tests {
    use super::parse_range;

    #[test]
    fn parses_standard_ranges() {
        assert_eq!(parse_range(Some("bytes=0-99"), 1000).unwrap().unwrap().len(), 100);
        assert_eq!(parse_range(Some("bytes=100-"), 1000).unwrap().unwrap().end, 999);
        assert_eq!(parse_range(Some("bytes=-100"), 1000).unwrap().unwrap().start, 900);
        assert_eq!(parse_range(None, 1000).unwrap(), None);
    }

    #[test]
    fn clamps_and_rejects() {
        assert_eq!(parse_range(Some("bytes=900-5000"), 1000).unwrap().unwrap().end, 999);
        assert!(parse_range(Some("bytes=1000-"), 1000).is_err());
        assert!(parse_range(Some("bytes=5-1"), 1000).is_err());
        assert!(parse_range(Some("bytes=0-1,3-4"), 1000).is_err());
        assert!(parse_range(Some("nibbles=0-1"), 1000).is_err());
    }
}
