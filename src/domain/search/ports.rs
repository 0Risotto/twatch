//! Torrent search port.

use super::{SearchConfig, SearchResult};
use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;

/// Queries a user-configured JSON API and returns a list of results.
#[async_trait]
pub trait SearchPort: Interface {
    /// Fetch torrent search results from the configured API.
    ///
    /// `page` is the 0-based result page to request.
    async fn search(
        &self,
        query: &str,
        config: &SearchConfig,
        page: u32,
    ) -> Result<Vec<SearchResult>>;
}
