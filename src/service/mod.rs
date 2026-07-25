//! Concrete service implementations.

pub mod player;
pub mod search;
pub mod storage;
pub mod torrent;

pub use player::RealPlayer;
pub use search::SearchServiceImpl;
pub use storage::RealStorage;
pub use torrent::TorrentEngineImpl;
