//! Concrete service implementations.

pub mod player;
pub mod storage;
pub mod torrent;

pub use player::RealPlayer;
pub use storage::RealStorage;
pub use torrent::TorrentEngineImpl;
