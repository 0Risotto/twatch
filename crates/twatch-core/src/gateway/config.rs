use super::*;

impl Gateway {
    // ── Config ───────────────────────────────────────────────────────

    pub fn update_theme(&self, name: &str) -> GatewayResult<()> {
        if theme::find(name).is_none() {
            return Err(GatewayError::UnknownTheme(name.to_owned()));
        }
        let config = {
            let mut guard = self.config.write().unwrap_or_else(|e| e.into_inner());
            guard.web_theme = name.to_owned();
            guard.clone()
        };
        config.save()?;
        Ok(())
    }
}
