use super::*;

impl Gateway {
    // ── Search ───────────────────────────────────────────────────────

    pub async fn search(&self, query: &str, page: u32) -> GatewayResult<Vec<SearchResult>> {
        let config = self.search_config();
        Ok(self.search.search(query, &config, page).await?)
    }

    pub fn update_search_config(&self, config: SearchConfig) -> GatewayResult<()> {
        config.save(&self.config().config_dir)?;
        *self.search_config.write().unwrap_or_else(|e| e.into_inner()) = config;
        Ok(())
    }
}
