//! Loading screen renderer.

use crate::app::App;
use crate::app::screens::loading::ViewModel;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App, vm: ViewModel<'_>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let text = Text::from(vec![
        Line::from(Span::styled(
            "Fetching torrent metadata...",
            app.theme_state.theme.warning_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(vm.status, app.theme_state.theme.dimmed_style())),
    ]);

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Loading")
                .border_style(Style::default().fg(app.theme_state.theme.palette.warning)),
        )
        .centered();

    frame.render_widget(paragraph, chunks[1]);
}
