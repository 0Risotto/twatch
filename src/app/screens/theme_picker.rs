//! The theme picker popup screen.

use super::{Screen, Transition};
use crate::app::App;
use crate::ui::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use std::any::Any;

/// One row of the theme list.
pub struct ThemeItem {
    pub name: &'static str,
    pub is_current: bool,
    pub is_selected: bool,
}

/// View model for [`ThemePickerScreen`].
pub struct ViewModel<'a> {
    pub filter: &'a str,
    pub items: Vec<ThemeItem>,
}

/// Picks a theme with live preview; Esc restores the original.
pub struct ThemePickerScreen {
    original: Theme,
    selected: usize,
    filter: String,
}

impl ThemePickerScreen {
    #[must_use]
    pub fn new(current: Theme) -> Self {
        let idx = Theme::ALL.iter().position(|t| *t == current).unwrap_or(0);
        Self { original: current, selected: idx, filter: String::new() }
    }

    /// Indices of themes matching the current filter.
    fn filtered_indices(&self) -> Vec<usize> {
        let filter = self.filter.to_lowercase();
        Theme::ALL
            .iter()
            .enumerate()
            .filter(|(_, t)| filter.is_empty() || t.name.to_lowercase().contains(&filter))
            .map(|(i, _)| i)
            .collect()
    }

    /// Preview the currently highlighted theme.
    fn preview_selected(&mut self, app: &mut App) {
        let filtered = self.filtered_indices();
        if let Some(&idx) = filtered.get(self.selected) {
            app.theme_state.theme = Theme::ALL[idx];
        }
    }
}

impl Screen for ThemePickerScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        let filtered = self.filtered_indices();
        match key.code {
            KeyCode::Esc => {
                app.theme_state.theme = self.original;
                return Some(Transition::Pop);
            }
            KeyCode::Enter => {
                if let Some(&idx) = filtered.get(self.selected) {
                    crate::application::theme::apply_theme(app, Theme::ALL[idx]);
                }
                return Some(Transition::Pop);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < filtered.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.selected = 0;
            }
            KeyCode::Char(c) if !c.is_control() && self.filter.len() < 64 => {
                self.filter.push(c);
                self.selected = 0;
            }
            _ => {}
        }

        self.preview_selected(app);
        None
    }

    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
        crate::ui::theme_picker::render(frame, area, app, self.view(app));
    }

    fn is_modal(&self) -> bool {
        true
    }

    fn renders_full_frame(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ThemePickerScreen {
    /// Build the view model rendered by the presentation layer.
    #[must_use]
    pub fn view<'a>(&'a self, app: &'a App) -> ViewModel<'a> {
        let filtered = self.filtered_indices();
        let items = filtered
            .iter()
            .enumerate()
            .map(|(pos, &idx)| ThemeItem {
                name: Theme::ALL[idx].name,
                is_current: Theme::ALL[idx] == app.theme_state.theme,
                is_selected: pos == self.selected,
            })
            .collect();
        ViewModel { filter: &self.filter, items }
    }
}
