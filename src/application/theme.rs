//! Theme selection use case.

use crate::app::App;
use crate::ui::Theme;

/// Apply `theme` and persist the choice to the config file.
pub fn apply_theme(app: &mut App, theme: Theme) {
    app.theme_state.theme = theme;
    app.config.theme = Theme::to_config_name(theme.name);
    if let Err(e) = app.config.save() {
        app.set_error(format!("Theme applied, but failed to persist: {e}"));
    }
}
