use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph},
};

use super::{format_size, styled_block};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let selected = app.selected_count();
    let title_text = if selected > 0 {
        format!(" {} file(s) — {} selected for download", app.files.len(), selected)
    } else {
        format!(" {} file(s) — Space: select, d: download", app.files.len())
    };

    let title =
        Paragraph::new(Text::from(Span::styled(title_text, Style::default().fg(Color::Cyan))))
            .block(styled_block(" Torrent Contents ", Color::Cyan));
    frame.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_hovered = i == app.selected_file;
            let checked = app.selected_files.get(i).copied().unwrap_or(false);

            let prefix = match (is_hovered, checked) {
                (true, _) => "▶ ",
                (false, true) => " ✓ ",
                (false, false) => "   ",
            };

            let name_style = if is_hovered {
                Style::default().fg(Color::Cyan)
            } else if checked {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(&file.name, name_style),
                Span::styled(
                    format!("  ({})", format_size(file.size)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(styled_block(" Files ", Color::White))
        .highlight_style(Style::default());

    frame.render_widget(list, chunks[1]);

    let footer = Paragraph::new(Text::from(vec![Line::from(Span::styled(
        "[w] Watch    [d] Download selected    [Space] Toggle    [r] Rename    [q] Back",
        Style::default().fg(Color::DarkGray),
    ))]))
    .centered();
    frame.render_widget(footer, chunks[2]);
}
