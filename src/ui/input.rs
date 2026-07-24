use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use super::{centered_rect, styled_block};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(60, 30, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(1)])
        .split(popup_area);

    let input_text: Vec<Line> = vec![Line::from(vec![
        Span::styled("▸ ", Style::default().fg(Color::Cyan)),
        Span::styled(&app.input.value, Style::default().fg(Color::White)),
    ])];

    let input_paragraph = Paragraph::new(Text::from(input_text))
        .block(styled_block(" Magnet / Torrent URL ", Color::Cyan))
        .style(Style::default().fg(Color::White));

    frame.render_widget(input_paragraph, chunks[1]);

    // Show cursor position hint
    if !app.input.value.is_empty() {
        let cursor_hint = format!("cursor at position {}", app.input.cursor);
        let hint = Paragraph::new(Text::from(Span::styled(
            cursor_hint,
            Style::default().fg(Color::DarkGray),
        )))
        .centered();
        frame.render_widget(hint, chunks[2]);
    }

    // Instructions
    let instructions = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            "Paste a magnet link or .torrent URL and press Enter",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "[Enter] Confirm    [Esc] Back",
            Style::default().fg(Color::DarkGray),
        )),
    ]))
    .centered();

    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    frame.render_widget(instructions, bottom[1]);

    // Show status message
    if !app.status_message.is_empty() {
        let status = Paragraph::new(Text::from(Span::styled(
            &app.status_message,
            Style::default().fg(Color::Yellow),
        )))
        .centered();

        let status_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        frame.render_widget(status, status_area[0]);
    }
}
