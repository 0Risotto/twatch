//! Torrent search interface.
//!
//! Queries a user-configured JSON API and returns a list of results.

use crate::config::search::SearchConfig;
use crate::model::SearchResult;
use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;

#[async_trait]
pub trait SearchService: Interface {
    /// Fetch torrent search results from the configured API.
    async fn search(&self, query: &str, config: &SearchConfig) -> Result<Vec<SearchResult>>;
}
