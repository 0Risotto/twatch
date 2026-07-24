//! Service interfaces.

pub mod player;
pub mod storage;
pub mod torrent;

pub use player::PlayerService;
pub use storage::StorageService;
pub use torrent::TorrentService;
