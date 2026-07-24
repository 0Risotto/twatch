//! Service interfaces.
//!
//! Traits in this module define the contracts that every service
//! implementation must fulfil.  Consumers depend on these traits
//! rather than on concrete types, which makes the application
//! testable and decoupled.

pub mod player;
pub mod storage;
pub mod torrent;

pub use player::PlayerService;
pub use storage::StorageService;
pub use torrent::TorrentService;
