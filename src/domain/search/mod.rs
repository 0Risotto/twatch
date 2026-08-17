//! Torrent search domain.

pub mod ports;

/// Number of results shown per page in the search popup.
pub const SEARCH_PAGE_SIZE: usize = 21;

/// A search result from the user-configured JSON API.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub info_hash: String,
    pub seeders: u64,
    pub leechers: u64,
    pub size: u64,
    pub username: String,
}
