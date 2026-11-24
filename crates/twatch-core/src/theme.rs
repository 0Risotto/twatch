//! Built-in theme catalogue shared with the web UI.
//!
//! The actual palettes live in `web/src/styles/themes.css`. Rust owns the
//! canonical list of names and modes so config validation and the API stay in sync.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ThemeSpec {
    pub name: &'static str,
    pub display_name: &'static str,
    pub mode: ThemeMode,
}

pub const THEMES: &[ThemeSpec] = &[
    ThemeSpec { name: "tokyonight-dark", display_name: "Tokyo Night", mode: ThemeMode::Dark },
    ThemeSpec { name: "tokyonight-light", display_name: "Tokyo Night", mode: ThemeMode::Light },
    ThemeSpec { name: "kanagawa-dark", display_name: "Kanagawa", mode: ThemeMode::Dark },
    ThemeSpec { name: "kanagawa-light", display_name: "Kanagawa", mode: ThemeMode::Light },
    ThemeSpec { name: "nord-dark", display_name: "Nord", mode: ThemeMode::Dark },
    ThemeSpec { name: "nord-light", display_name: "Nord", mode: ThemeMode::Light },
    ThemeSpec { name: "gruvbox-dark", display_name: "Gruvbox", mode: ThemeMode::Dark },
    ThemeSpec { name: "gruvbox-light", display_name: "Gruvbox", mode: ThemeMode::Light },
    ThemeSpec { name: "catppuccin-dark", display_name: "Catppuccin", mode: ThemeMode::Dark },
    ThemeSpec { name: "catppuccin-light", display_name: "Catppuccin", mode: ThemeMode::Light },
    ThemeSpec { name: "purple-dark", display_name: "Purple", mode: ThemeMode::Dark },
    ThemeSpec { name: "purple-light", display_name: "Purple", mode: ThemeMode::Light },
    ThemeSpec { name: "emerald-dark", display_name: "Emerald", mode: ThemeMode::Dark },
    ThemeSpec { name: "emerald-light", display_name: "Emerald", mode: ThemeMode::Light },
    ThemeSpec { name: "monokai-dark", display_name: "Monokai", mode: ThemeMode::Dark },
    ThemeSpec { name: "monokai-light", display_name: "Monokai", mode: ThemeMode::Light },
    ThemeSpec { name: "solarized-dark", display_name: "Solarized", mode: ThemeMode::Dark },
    ThemeSpec { name: "solarized-light", display_name: "Solarized", mode: ThemeMode::Light },
    ThemeSpec { name: "amber-dark", display_name: "Amber", mode: ThemeMode::Dark },
    ThemeSpec { name: "amber-light", display_name: "Amber", mode: ThemeMode::Light },
    ThemeSpec { name: "dracula-dark", display_name: "Dracula", mode: ThemeMode::Dark },
    ThemeSpec { name: "dracula-light", display_name: "Dracula", mode: ThemeMode::Light },
    ThemeSpec { name: "rosepine-dark", display_name: "Rosé Pine", mode: ThemeMode::Dark },
    ThemeSpec { name: "rosepine-light", display_name: "Rosé Pine", mode: ThemeMode::Light },
    ThemeSpec { name: "ice-dark", display_name: "Ice", mode: ThemeMode::Dark },
    ThemeSpec { name: "ice-light", display_name: "Ice", mode: ThemeMode::Light },
    ThemeSpec { name: "amoled-dark", display_name: "AMOLED", mode: ThemeMode::Dark },
    ThemeSpec { name: "amoled-light", display_name: "AMOLED", mode: ThemeMode::Light },
    ThemeSpec { name: "orchid-dark", display_name: "Orchid", mode: ThemeMode::Dark },
    ThemeSpec { name: "orchid-light", display_name: "Orchid", mode: ThemeMode::Light },
    ThemeSpec { name: "intellij-dark", display_name: "IntelliJ", mode: ThemeMode::Dark },
    ThemeSpec { name: "intellij-light", display_name: "IntelliJ", mode: ThemeMode::Light },
    ThemeSpec { name: "highcontrast-dark", display_name: "High Contrast", mode: ThemeMode::Dark },
    ThemeSpec { name: "highcontrast-light", display_name: "High Contrast", mode: ThemeMode::Light },
    ThemeSpec { name: "everforest-dark", display_name: "Everforest", mode: ThemeMode::Dark },
    ThemeSpec { name: "everforest-light", display_name: "Everforest", mode: ThemeMode::Light },
    ThemeSpec {
        name: "amoled-midnight-dark",
        display_name: "AMOLED Midnight",
        mode: ThemeMode::Dark,
    },
    ThemeSpec {
        name: "amoled-midnight-light",
        display_name: "AMOLED Midnight",
        mode: ThemeMode::Light,
    },
    ThemeSpec {
        name: "amoled-grey-calm-dark",
        display_name: "AMOLED Grey Calm",
        mode: ThemeMode::Dark,
    },
    ThemeSpec {
        name: "amoled-grey-calm-light",
        display_name: "AMOLED Grey Calm",
        mode: ThemeMode::Light,
    },
    ThemeSpec { name: "nu-disco-dark", display_name: "Nu Disco", mode: ThemeMode::Dark },
    ThemeSpec { name: "nu-disco-light", display_name: "Nu Disco", mode: ThemeMode::Light },
    ThemeSpec { name: "amethyst-oled-dark", display_name: "Amethyst OLED", mode: ThemeMode::Dark },
    ThemeSpec {
        name: "amethyst-oled-light",
        display_name: "Amethyst OLED",
        mode: ThemeMode::Light,
    },
    ThemeSpec { name: "ocean-dark", display_name: "Ocean", mode: ThemeMode::Dark },
    ThemeSpec { name: "ocean-light", display_name: "Ocean", mode: ThemeMode::Light },
    ThemeSpec { name: "forest-dark", display_name: "Forest", mode: ThemeMode::Dark },
    ThemeSpec { name: "forest-light", display_name: "Forest", mode: ThemeMode::Light },
    ThemeSpec { name: "sunset-dark", display_name: "Sunset", mode: ThemeMode::Dark },
    ThemeSpec { name: "sunset-light", display_name: "Sunset", mode: ThemeMode::Light },
    ThemeSpec { name: "sakura-dark", display_name: "Sakura", mode: ThemeMode::Dark },
    ThemeSpec { name: "sakura-light", display_name: "Sakura", mode: ThemeMode::Light },
    ThemeSpec { name: "coffee-dark", display_name: "Coffee", mode: ThemeMode::Dark },
    ThemeSpec { name: "coffee-light", display_name: "Coffee", mode: ThemeMode::Light },
    ThemeSpec { name: "mono-dark", display_name: "Black & White", mode: ThemeMode::Dark },
    ThemeSpec { name: "mono-light", display_name: "Black & White", mode: ThemeMode::Light },
];

#[must_use]
pub const fn all() -> &'static [ThemeSpec] {
    THEMES
}

#[must_use]
pub fn find(name: &str) -> Option<&'static ThemeSpec> {
    THEMES.iter().find(|theme| theme.name == name)
}
