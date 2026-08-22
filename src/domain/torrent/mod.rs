//! Torrent domain: entities, value objects, and engine ports.

pub mod file_tree;
pub mod ports;

pub use file_tree::FileTree;

/// Opaque handle to a torrent managed by the engine.
///
/// The raw identifier is owned by the adapter; domain code treats
/// the handle as an opaque token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TorrentId(usize);

impl TorrentId {
    /// Wrap an adapter-supplied raw identifier.
    #[must_use]
    pub const fn new(raw: usize) -> Self {
        Self(raw)
    }

    /// Raw adapter identifier.  Engine adapters only.
    #[must_use]
    pub(crate) const fn raw(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TorrentFile {
    pub index: usize,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct TorrentInfo {
    pub name: String,
    pub files: Vec<TorrentFile>,
}

#[derive(Debug, Clone)]
pub struct TorrentStats {
    pub progress: f64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub peers: usize,
    pub total_size: u64,
    pub downloaded: u64,
}

/// A single entry in the collapsible folder-tree browser list.
#[derive(Debug, Clone)]
pub enum DisplayEntry {
    /// A collapsible subfolder.
    Folder { name: String, depth: usize, expanded: bool },
    /// A regular file from the torrent.
    File { file: TorrentFile, depth: usize },
}

impl DisplayEntry {
    /// Nesting depth of this entry in the tree projection.
    #[must_use]
    pub const fn depth(&self) -> usize {
        match self {
            Self::Folder { depth, .. } | Self::File { depth, .. } => *depth,
        }
    }
}

/// Lifecycle status of a single file within the current session.
///
/// Aggregated from history (`watched` / `downloaded`) and the live
/// download list (`downloading`).  This is the single source of truth
/// used by the presentation layer to derive status badges.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FileStatus {
    pub watched: bool,
    pub downloaded: bool,
    pub downloading: bool,
}
