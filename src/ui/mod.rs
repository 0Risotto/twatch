//! Presentation layer.
//!
//! Initializes the terminal, renders screens from their view models,
//! and restores the terminal on exit.  No business logic lives here.

pub mod badges;
pub(crate) mod browser;
pub(crate) mod history;
pub(crate) mod input;
pub(crate) mod loading;
pub(crate) mod player;
pub(crate) mod rename;
pub(crate) mod search_popup;
pub(crate) mod sidebar;
mod theme;
pub(crate) mod theme_picker;
pub(crate) mod welcome;

use crate::app::App;
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
///
/// On failure after raw mode is enabled, restores the terminal before
/// returning so the caller never leaves the TTY mangled.
pub fn init() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    if let Err(e) = execute!(stdout(), EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(e.into());
    }
    match Terminal::new(CrosstermBackend::new(stdout())) {
        Ok(terminal) => Ok(terminal),
        Err(e) => {
            let _ = restore();
            Err(e.into())
        }
    }
}

/// Leave the alternate screen and disable raw mode.
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Install a panic hook that restores the terminal before unwinding,
/// so a crash never leaves raw mode / the alternate screen active.
pub fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore();
        original(info);
    }));
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
        base.render(frame, base_area, app);

        // The sidebar sits above the base but BELOW modal overlays, so
        // full-frame popups (search, theme picker) paint over it.
        if !base.renders_full_frame() {
            sidebar::render(frame, sidebar_area, app);
        }

        // Modal overlays stacked above the base render on top of it.
        for screen in &screens[base_idx + 1..] {
            let target_area = if screen.renders_full_frame() { area } else { main_area };
            screen.render(frame, target_area, app);
        }
    } else {
        sidebar::render(frame, sidebar_area, app);
    }
}

/// Index of the topmost non-modal screen: the visible base that
/// everything else hides behind.
pub fn base_screen_idx(screens: &[&dyn crate::app::screens::Screen]) -> Option<usize> {
    screens.iter().rposition(|s| !s.is_modal())
}
