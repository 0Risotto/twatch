//! Built-in theme catalogue shared with the web UI.
//!
//! The actual palettes live in `web/src/styles/themes.css`. Rust owns the
//! canonical list of names and modes so config validation and the API stay in sync.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSpec {
    pub name: &'static str,
    pub display_name: &'static str,
    pub mode: &'static str,
}

pub const THEMES: &[ThemeSpec] = &[
    ThemeSpec { name: "tokyonight-dark", display_name: "Tokyo Night", mode: "dark" },
    ThemeSpec { name: "tokyonight-light", display_name: "Tokyo Night", mode: "light" },
    ThemeSpec { name: "kanagawa-dark", display_name: "Kanagawa", mode: "dark" },
    ThemeSpec { name: "kanagawa-light", display_name: "Kanagawa", mode: "light" },
    ThemeSpec { name: "nord-dark", display_name: "Nord", mode: "dark" },
    ThemeSpec { name: "nord-light", display_name: "Nord", mode: "light" },
    ThemeSpec { name: "gruvbox-dark", display_name: "Gruvbox", mode: "dark" },
    ThemeSpec { name: "gruvbox-light", display_name: "Gruvbox", mode: "light" },
    ThemeSpec { name: "catppuccin-dark", display_name: "Catppuccin", mode: "dark" },
    ThemeSpec { name: "catppuccin-light", display_name: "Catppuccin", mode: "light" },
    ThemeSpec { name: "purple-dark", display_name: "Purple", mode: "dark" },
    ThemeSpec { name: "purple-light", display_name: "Purple", mode: "light" },
    ThemeSpec { name: "emerald-dark", display_name: "Emerald", mode: "dark" },
    ThemeSpec { name: "emerald-light", display_name: "Emerald", mode: "light" },
    ThemeSpec { name: "monokai-dark", display_name: "Monokai", mode: "dark" },
    ThemeSpec { name: "monokai-light", display_name: "Monokai", mode: "light" },
    ThemeSpec { name: "solarized-dark", display_name: "Solarized", mode: "dark" },
    ThemeSpec { name: "solarized-light", display_name: "Solarized", mode: "light" },
    ThemeSpec { name: "amber-dark", display_name: "Amber", mode: "dark" },
    ThemeSpec { name: "amber-light", display_name: "Amber", mode: "light" },
    ThemeSpec { name: "dracula-dark", display_name: "Dracula", mode: "dark" },
    ThemeSpec { name: "dracula-light", display_name: "Dracula", mode: "light" },
    ThemeSpec { name: "rosepine-dark", display_name: "Rosé Pine", mode: "dark" },
    ThemeSpec { name: "rosepine-light", display_name: "Rosé Pine", mode: "light" },
    ThemeSpec { name: "ice-dark", display_name: "Ice", mode: "dark" },
    ThemeSpec { name: "ice-light", display_name: "Ice", mode: "light" },
    ThemeSpec { name: "amoled-dark", display_name: "AMOLED", mode: "dark" },
    ThemeSpec { name: "amoled-light", display_name: "AMOLED", mode: "light" },
    ThemeSpec { name: "orchid-dark", display_name: "Orchid", mode: "dark" },
    ThemeSpec { name: "orchid-light", display_name: "Orchid", mode: "light" },
    ThemeSpec { name: "intellij-dark", display_name: "IntelliJ", mode: "dark" },
    ThemeSpec { name: "intellij-light", display_name: "IntelliJ", mode: "light" },
    ThemeSpec { name: "highcontrast-dark", display_name: "High Contrast", mode: "dark" },
    ThemeSpec { name: "highcontrast-light", display_name: "High Contrast", mode: "light" },
    ThemeSpec { name: "everforest-dark", display_name: "Everforest", mode: "dark" },
    ThemeSpec { name: "everforest-light", display_name: "Everforest", mode: "light" },
    ThemeSpec { name: "amoled-midnight-dark", display_name: "AMOLED Midnight", mode: "dark" },
    ThemeSpec { name: "amoled-midnight-light", display_name: "AMOLED Midnight", mode: "light" },
    ThemeSpec { name: "amoled-grey-calm-dark", display_name: "AMOLED Grey Calm", mode: "dark" },
    ThemeSpec { name: "amoled-grey-calm-light", display_name: "AMOLED Grey Calm", mode: "light" },
    ThemeSpec { name: "nu-disco-dark", display_name: "Nu Disco", mode: "dark" },
    ThemeSpec { name: "nu-disco-light", display_name: "Nu Disco", mode: "light" },
    ThemeSpec { name: "amethyst-oled-dark", display_name: "Amethyst OLED", mode: "dark" },
    ThemeSpec { name: "amethyst-oled-light", display_name: "Amethyst OLED", mode: "light" },
    ThemeSpec { name: "ocean-dark", display_name: "Ocean", mode: "dark" },
    ThemeSpec { name: "ocean-light", display_name: "Ocean", mode: "light" },
    ThemeSpec { name: "forest-dark", display_name: "Forest", mode: "dark" },
    ThemeSpec { name: "forest-light", display_name: "Forest", mode: "light" },
    ThemeSpec { name: "sunset-dark", display_name: "Sunset", mode: "dark" },
    ThemeSpec { name: "sunset-light", display_name: "Sunset", mode: "light" },
    ThemeSpec { name: "sakura-dark", display_name: "Sakura", mode: "dark" },
    ThemeSpec { name: "sakura-light", display_name: "Sakura", mode: "light" },
    ThemeSpec { name: "coffee-dark", display_name: "Coffee", mode: "dark" },
    ThemeSpec { name: "coffee-light", display_name: "Coffee", mode: "light" },
];

#[must_use]
pub const fn all() -> &'static [ThemeSpec] {
    THEMES
}

#[must_use]
pub fn find(name: &str) -> Option<&'static ThemeSpec> {
    THEMES.iter().find(|theme| theme.name == name)
}

#[must_use]
pub const fn default_theme() -> &'static str {
    "emerald-dark"
}
