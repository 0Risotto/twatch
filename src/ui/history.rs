use crate::app::App;
use crate::model::display_name;
use crate::traits::StorageService;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph},
};
use shaku::HasComponent;
use std::sync::Arc;

use crate::ui::theme::styled_block;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let storage: Arc<dyn StorageService> = app.module.resolve();
    let entries = storage.history();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    // Header
    let header = Paragraph::new(Text::from(Span::styled(
        format!(" {} entries in history", entries.len()),
        Style::default().fg(Color::Cyan),
    )))
    .block(styled_block(" History ", Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Entries list
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.history_selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let name = display_name(entry);

            let name_style = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            let url_display = if entry.url.len() > 60 {
                format!("{}...", &entry.url[..57])
            } else {
                entry.url.clone()
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(name, name_style),
                ]),
                Line::from(Span::styled(
                    format!("    {url_display}"),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let list = List::new(items).block(styled_block(" Entries ", Color::White));
    frame.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new(Text::from(vec![Line::from(Span::styled(
        "[Enter] Re-add    [r] Rename    [d] Delete    [q/Esc] Back",
        Style::default().fg(Color::DarkGray),
    ))]))
    .centered();
    frame.render_widget(footer, chunks[2]);

    // Rename overlay
    if app.renaming && app.screen == crate::model::Screen::History {
        draw_rename_overlay(frame, area, app);
    }
}

fn draw_rename_overlay(frame: &mut Frame, area: Rect, app: &App) {
    use crate::ui::theme::centered_rect;

    let popup = centered_rect(60, 20, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(popup);

    let text = Text::from(vec![
        Line::from(Span::styled("Rename entry", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(
            format!("▸ {}", app.rename_input.value),
            Style::default().fg(Color::White),
        )),
    ]);

    let para =
        Paragraph::new(text).block(styled_block(" Rename ", Color::Cyan)).style(Style::default());

    frame.render_widget(para, chunks[1]);
}

// Re-export the draw function for browser rename overlay too
pub fn draw_browser_rename_overlay(frame: &mut Frame, area: Rect, app: &App) {
    use crate::ui::theme::centered_rect;

    let popup = centered_rect(60, 20, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(popup);

    let text = Text::from(vec![
        Line::from(Span::styled("Rename file", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(
            format!("▸ {}", app.rename_input.value),
            Style::default().fg(Color::White),
        )),
    ]);

    let para =
        Paragraph::new(text).block(styled_block(" Rename ", Color::Cyan)).style(Style::default());

    frame.render_widget(para, chunks[1]);
}
