#![allow(clippy::unwrap_used)]

use twatch::config::Config;
use twatch::ui::Theme;

#[test]
fn theme_all_is_non_empty() {
    assert!(!Theme::ALL.is_empty());
}

#[test]
fn theme_default_is_tokyonight_dark() {
    let default = Theme::default();
    assert_eq!(default.name, "TokyoNight Dark");
    assert_eq!(default, Theme::ALL[0]);
}

#[test]
fn theme_cycle_wraps_around() {
    let last = Theme::ALL[Theme::ALL.len() - 1];
    assert_eq!(last.next(), Theme::ALL[0]);
}

#[test]
fn theme_cycle_sequential() {
    for i in 0..Theme::ALL.len() - 1 {
        assert_eq!(Theme::ALL[i].next(), Theme::ALL[i + 1]);
    }
}

#[test]
fn theme_from_name_valid_kebab() {
    assert!(Theme::from_name("tokyonight-dark").is_some());
    assert!(Theme::from_name("tokyonight-light").is_some());
    assert!(Theme::from_name("kanagawa-dark").is_some());
    assert!(Theme::from_name("nord-light").is_some());
    assert!(Theme::from_name("gruvbox-dark").is_some());
    assert!(Theme::from_name("catppuccin-mocha").is_some());
    assert!(Theme::from_name("catppuccin-latte").is_some());
}

#[test]
fn theme_from_name_valid_display() {
    assert!(Theme::from_name("TokyoNight Dark").is_some());
    assert!(Theme::from_name("Kanagawa Light").is_some());
    assert!(Theme::from_name("Nord Light").is_some());
}

#[test]
fn theme_from_name_invalid() {
    assert!(Theme::from_name("nonsense").is_none());
    assert!(Theme::from_name("").is_none());
    assert!(Theme::from_name("cyan").is_none());
}

#[test]
fn theme_to_config_name() {
    assert_eq!(Theme::to_config_name("TokyoNight Dark"), "tokyonight-dark");
    assert_eq!(Theme::to_config_name("Catppuccin Latte"), "catppuccin-latte");
    assert_eq!(Theme::to_config_name("Nord Light"), "nord-light");
}

#[test]
fn palette_accent_is_not_default_black() {
    for theme in &Theme::ALL {
        assert_ne!(
            theme.palette.accent,
            ratatui::style::Color::Black,
            "{} has black accent",
            theme.name
        );
    }
}

#[test]
fn palette_fields_are_distinct() {
    for theme in &Theme::ALL {
        assert_ne!(theme.palette.accent, theme.palette.text_dimmed);
        assert_ne!(theme.palette.accent, theme.palette.text_primary);
        assert_ne!(theme.palette.success, theme.palette.warning);
    }
}

#[test]
fn all_themes_have_unique_names() {
    let names: Vec<&str> = Theme::ALL.iter().map(|t| t.name).collect();
    let mut deduped = names.clone();
    deduped.sort_unstable();
    deduped.dedup();
    assert_eq!(names.len(), deduped.len(), "theme names must be unique");
}

#[test]
fn config_load_theme_field() {
    let tmp = std::env::temp_dir().join("twatch_test_config_load_theme");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let config_path = tmp.join("config.json");
    let json = r#"{"theme": "nord-dark"}"#;
    std::fs::write(&config_path, json).unwrap();

    let read_back = std::fs::read_to_string(&config_path).unwrap();
    let parsed: Config = serde_json::from_str(&read_back).unwrap();
    assert_eq!(parsed.theme, "nord-dark");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn config_save_roundtrip() {
    let tmp = std::env::temp_dir().join("twatch_test_config_roundtrip");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let config =
        Config { config_dir: tmp.clone(), theme: "kanagawa-dark".to_string(), ..Config::default() };
    config.save().unwrap();

    let config_path = tmp.join("config.json");
    let contents = std::fs::read_to_string(&config_path).unwrap();
    let parsed: Config = serde_json::from_str(&contents).unwrap();
    assert_eq!(parsed.theme, "kanagawa-dark");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn config_save_creates_dirs() {
    let tmp = std::env::temp_dir().join("twatch_test_config_mkdir");
    let _ = std::fs::remove_dir_all(&tmp);

    let config = Config { config_dir: tmp.clone(), ..Config::default() };
    config.save().unwrap();
    assert!(tmp.join("config.json").exists());

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn config_default_theme_is_tokyonight() {
    let config = Config::default();
    assert_eq!(config.theme, "tokyonight-dark");
}
