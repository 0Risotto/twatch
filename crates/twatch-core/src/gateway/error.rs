//! User-facing gateway errors.
//!
//! Adapter failures stay inside [`GatewayError::Internal`]; callers can map the
//! named variants to protocol-specific responses without inspecting strings.

use thiserror::Error;

/// Errors returned by public gateway operations.
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("No active torrent session")]
    NoSession,
    #[error("No files selected")]
    NoFilesSelected,
    #[error("No matching files")]
    NoMatchingFiles,
    #[error("File not found")]
    FileNotFound,
    #[error("Download not found")]
    DownloadNotFound,
    #[error("History entry not found")]
    HistoryNotFound,
    #[error("Stream is not ready yet")]
    StreamNotReady,
    #[error("Unknown theme: {0}")]
    UnknownTheme(String),
    #[error("Unknown player preference: {0}")]
    UnknownPlayer(String),
    #[error("File name must be between 1 and 256 characters")]
    InvalidFileNameLength,
    #[error("File name cannot contain path separators")]
    InvalidFileNameCharacters,
    #[error("Invalid file name")]
    InvalidFileName,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Result alias for public gateway operations.
pub type GatewayResult<T> = Result<T, GatewayError>;
