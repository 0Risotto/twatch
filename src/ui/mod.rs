//! Terminal UI module.
//!
//! Initializes the terminal, draws screens, and restores on exit.

mod browser;
mod history;
mod input;
mod player;
mod sidebar;
mod theme;
mod theme_picker;
mod welcome;

use crate::app::App;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use std::io::{Stdout, stdout};

pub use theme::{
    Palette, Theme, centered_rect, color_footer, format_size, format_speed, styled_block,
};

pub fn init() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}

pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    frame.render_widget(ratatui::widgets::Block::default().style(app.theme.bg_style()), area);

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    let main_area = panes[0];
    let sidebar_area = panes[1];

    match app.screen {
        crate::model::Screen::Welcome => welcome::draw(frame, main_area, app),
        crate::model::Screen::Input => input::draw(frame, main_area, app),
        crate::model::Screen::Loading => draw_loading(frame, main_area, app),
        crate::model::Screen::Browser => {
            browser::draw(frame, main_area, app);
            if app.renaming {
                history::draw_browser_rename_overlay(frame, main_area, app);
            }
        }
        crate::model::Screen::Player => player::draw(frame, main_area, app),
        crate::model::Screen::History => history::draw(frame, main_area, app),
    }

    sidebar::draw(frame, sidebar_area, app);

    if app.theme_picker {
        theme_picker::draw(frame, area, app);
    }
}

fn draw_loading(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let text = Text::from(vec![
        Line::from(Span::styled("Fetching torrent metadata...", app.theme.warning_style())),
        Line::from(""),
        Line::from(Span::styled(&app.status_message, app.theme.dimmed_style())),
    ]);

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Loading")
                .border_style(Style::default().fg(app.theme.palette.warning)),
        )
        .centered();

    frame.render_widget(paragraph, chunks[1]);
}
