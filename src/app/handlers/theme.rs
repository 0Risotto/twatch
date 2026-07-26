use crate::app::App;
use crate::ui::Theme;
use crossterm::event::KeyCode;
pub(crate) fn open_theme_picker(app: &mut App) {
    app.theme_picker_original = app.theme;
    app.theme_picker = true;
    app.theme_picker_filter.clear();
    let idx = Theme::ALL.iter().position(|t| *t == app.theme).unwrap_or(0);
    app.theme_picker_selected = idx;
    app.theme_picker_scroll = 0;
}

pub(crate) fn theme_picker_input(app: &mut App, code: KeyCode) {
    let all = Theme::ALL;
    let filter = app.theme_picker_filter.to_lowercase();
    let filtered: Vec<usize> = all
        .iter()
        .enumerate()
        .filter(|(_, t)| filter.is_empty() || t.name.to_lowercase().contains(&filter))
        .map(|(i, _)| i)
        .collect();

    match code {
        KeyCode::Esc => {
            app.theme = app.theme_picker_original;
            app.theme_picker = false;
        }
        KeyCode::Enter => {
            if let Some(&idx) = filtered.get(app.theme_picker_selected) {
                app.theme = all[idx];
                app.config.theme = Theme::to_config_name(app.theme.name);
                let _ = app.config.save();
            }
            app.theme_picker = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.theme_picker_selected > 0 {
                app.theme_picker_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.theme_picker_selected < filtered.len().saturating_sub(1) {
                app.theme_picker_selected += 1;
            }
        }
        KeyCode::Backspace => {
            app.theme_picker_filter.pop();
            app.theme_picker_selected = 0;
        }
        KeyCode::Char(c) if !c.is_control() && app.theme_picker_filter.len() < 64 => {
            app.theme_picker_filter.push(c);
            app.theme_picker_selected = 0;
        }
        _ => {}
    }

    // Preview the currently highlighted theme
    if app.theme_picker {
        if let Some(&idx) = filtered.get(app.theme_picker_selected) {
            app.theme = all[idx];
        }
    }
}
