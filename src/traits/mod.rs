//! Service interfaces.

pub mod player;
pub mod search;
pub mod storage;
pub mod torrent;

pub use player::PlayerService;
pub use search::SearchService;
pub use storage::StorageService;
pub use torrent::TorrentService;
