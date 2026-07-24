//! Concrete service implementations.
//!
//! Each module contains the default ("real") implementation of the
//! corresponding trait from [`crate::traits`].  Mock implementations
//! for tests live here as well (gated behind `#[cfg(test)]`).

pub mod player;
pub mod storage;
pub mod torrent;

pub use player::RealPlayer;
pub use storage::RealStorage;
pub use torrent::TorrentEngineImpl;
