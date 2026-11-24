use super::*;

impl Gateway {
    // ── Config ───────────────────────────────────────────────────────

    pub fn update_theme(&self, name: &str) -> Result<()> {
        if theme::find(name).is_none() {
            bail!("Unknown theme: {name}");
        }
        let config = {
            let mut guard = self.config.write().unwrap_or_else(|e| e.into_inner());
            guard.web_theme = name.to_owned();
            guard.clone()
        };
        config.save()?;
        Ok(())
    }

    pub fn update_player(&self, player: &str) -> Result<()> {
        if !matches!(player, "auto" | "mpv" | "vlc" | "none") {
            bail!("Unknown player preference: {player}");
        }
        let config = {
            let mut guard = self.config.write().unwrap_or_else(|e| e.into_inner());
            guard.player = player.to_owned();
            guard.clone()
        };
        config.save()?;
        Ok(())
    }
}
