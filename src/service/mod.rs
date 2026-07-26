//! Concrete service implementations.

pub mod player;
pub mod search;
pub mod storage;
pub mod torrent;

pub use player::RealPlayer;
pub use search::SearchServiceImpl;
pub use storage::RealStorage;
pub use torrent::TorrentEngineImpl;

use std::sync::Mutex;

pub(crate) fn lock_state<T>(state: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    state.lock().unwrap_or_else(|e| e.into_inner())
}
