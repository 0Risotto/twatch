//! Presentation layer.
//!
//! Initializes the terminal, renders screens from their view models,
//! and restores the terminal on exit.  No business logic lives here.

pub mod badges;
mod browser;
mod history;
mod input;
mod loading;
mod player;
mod rename;
mod search_popup;
mod sidebar;
mod theme;
mod theme_picker;
mod welcome;

use crate::app::App;
use crate::app::screens::ScreenViewModel;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use std::io::{Stdout, stdout};

pub use theme::{
    Palette, Theme, centered_rect, color_footer, format_size, format_speed, styled_block,
};

/// Enter raw mode + alternate screen and create the terminal.
pub fn init() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}

/// Leave the alternate screen and disable raw mode.
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Render the current screen: the topmost non-modal screen plus any
/// modal overlays pushed above it.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    frame.render_widget(
        ratatui::widgets::Block::default().style(app.theme_state.theme.bg_style()),
        area,
    );

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    let main_area = panes[0];
    let sidebar_area = panes[1];

    let screens: Vec<&dyn crate::app::screens::Screen> = app.screens().collect();

    // The topmost non-modal screen is the visible base; screens below
    // it are hidden.
    if let Some(base_idx) = base_screen_idx(&screens) {
        let base = screens[base_idx];
        let base_area = if base.renders_full_frame() { area } else { main_area };
        render_screen(frame, base_area, app, base.view(app));

        // Modal overlays stacked above the base render on top of it.
        for screen in &screens[base_idx + 1..] {
            let target_area = if screen.renders_full_frame() { area } else { main_area };
            render_screen(frame, target_area, app, screen.view(app));
        }
    }

    sidebar::render(frame, sidebar_area, app);
}

/// Index of the topmost non-modal screen: the visible base that
/// everything else hides behind.
pub fn base_screen_idx(screens: &[&dyn crate::app::screens::Screen]) -> Option<usize> {
    screens.iter().rposition(|s| !s.is_modal())
}

fn render_screen(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App,
    vm: ScreenViewModel<'_>,
) {
    match vm {
        ScreenViewModel::Welcome(vm) => welcome::render(frame, area, app, vm),
        ScreenViewModel::Input(vm) => input::render(frame, area, app, vm),
        ScreenViewModel::Loading(vm) => loading::render(frame, area, app, vm),
        ScreenViewModel::Browser(vm) => browser::render(frame, area, app, vm),
        ScreenViewModel::Player(vm) => player::render(frame, area, app, vm),
        ScreenViewModel::History(vm) => history::render(frame, area, app, vm),
        ScreenViewModel::Search(vm) => search_popup::render(frame, area, app, vm),
        ScreenViewModel::ThemePicker(vm) => theme_picker::render(frame, area, app, vm),
        ScreenViewModel::Rename(vm) => rename::render(frame, area, app, vm),
    }
}
