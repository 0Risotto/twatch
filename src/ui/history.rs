use crate::app::App;
use crate::model::display_name;
use crate::traits::StorageService;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph},
};
use shaku::HasComponent;
use std::sync::Arc;

use crate::ui::theme::{color_footer, styled_block};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let storage: Arc<dyn StorageService> = app.module.resolve();
    let all = storage.history();

    let entries: Vec<_> = if app.search_query.is_empty() {
        all.iter().collect()
    } else {
        all.iter()
            .filter(|e| {
                let name = display_name(e);
                crate::model::fuzzy_match(&app.search_query, name)
                    || crate::model::fuzzy_match(&app.search_query, &e.url)
            })
            .collect()
    };

    if app.is_searching {
        draw_with_search(frame, area, app, &entries);
    } else {
        draw_normal(frame, area, app, &entries);
    }

    if app.renaming && app.screen == crate::model::Screen::History {
        draw_rename_overlay(frame, area, app);
    }
}

fn draw_normal(frame: &mut Frame, area: Rect, app: &App, entries: &[&crate::model::HistoryEntry]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let header = Paragraph::new(Text::from(Span::styled(
        format!(" {} entries in history", entries.len()),
        app.theme.accent_style(),
    )))
    .block(styled_block(" History ", app.theme.palette.accent));
    frame.render_widget(header, chunks[0]);

    let items = build_items(app, entries);
    let list = List::new(items)
        .block(styled_block(" Entries ", app.theme.palette.border))
        .highlight_style(Style::default());
    frame.render_widget(list, chunks[1]);

    let footer = Paragraph::new(color_footer(
        "[/] search    [Enter] re-add    [r] rename    [d] delete    [q/Esc] back",
        &app.theme,
    ))
    .centered();
    frame.render_widget(footer, chunks[2]);
}

fn draw_with_search(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    entries: &[&crate::model::HistoryEntry],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let header = Paragraph::new(Text::from(Span::styled(
        format!(" {} entries in history", entries.len()),
        app.theme.accent_style(),
    )))
    .block(styled_block(" History ", app.theme.palette.accent));
    frame.render_widget(header, chunks[0]);

    let search_text = if app.search_query.is_empty() {
        "Search: ".to_string()
    } else {
        let left = format!("Search: {}", app.search_query);
        let right = format!("| {}", entries.len());
        let pad = (chunks[1].width.saturating_sub(2) as usize)
            .saturating_sub(left.len())
            .saturating_sub(right.len());
        format!("{}{}{}", left, " ".repeat(pad), right)
    };
    let bar = Paragraph::new(Span::styled(search_text, app.theme.warning_style()))
        .block(styled_block(" Search ", app.theme.palette.warning));
    frame.render_widget(bar, chunks[1]);

    let items = build_items(app, entries);
    let list = List::new(items)
        .block(styled_block(" Entries ", app.theme.palette.border))
        .highlight_style(Style::default());
    frame.render_widget(list, chunks[2]);

    let footer =
        Paragraph::new(color_footer("[Esc] cancel    [q] exit    [j/k] navigate", &app.theme))
            .centered();
    frame.render_widget(footer, chunks[3]);
}

fn build_items<'a>(app: &App, entries: &[&'a crate::model::HistoryEntry]) -> Vec<ListItem<'a>> {
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.history_selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let name = display_name(entry);

            let name_style =
                if is_selected { app.theme.accent_style() } else { app.theme.text_style() };

            let url_display = if entry.url.len() > 60 {
                format!("{}...", &entry.url[..57])
            } else {
                entry.url.clone()
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, app.theme.accent_style()),
                    Span::styled(name, name_style),
                ]),
                Line::from(Span::styled(format!("    {url_display}"), app.theme.dimmed_style())),
            ])
        })
        .collect()
}

fn draw_rename_overlay(frame: &mut Frame, area: Rect, app: &App) {
    use crate::ui::theme::centered_rect;

    let popup = centered_rect(60, 20, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(popup);

    let text = Text::from(vec![
        Line::from(Span::styled("Rename entry", app.theme.accent_style())),
        Line::from(Span::styled(format!("▸ {}", app.rename_input.value), app.theme.text_style())),
    ]);

    let para = Paragraph::new(text)
        .block(styled_block(" Rename ", app.theme.palette.accent))
        .style(Style::default());

    frame.render_widget(para, chunks[1]);
}

pub fn draw_browser_rename_overlay(frame: &mut Frame, area: Rect, app: &App) {
    use crate::ui::theme::centered_rect;

    let popup = centered_rect(60, 20, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(popup);

    let text = Text::from(vec![
        Line::from(Span::styled("Rename file", app.theme.accent_style())),
        Line::from(Span::styled(format!("▸ {}", app.rename_input.value), app.theme.text_style())),
    ]);

    let para = Paragraph::new(text)
        .block(styled_block(" Rename ", app.theme.palette.accent))
        .style(Style::default());

    frame.render_widget(para, chunks[1]);
}
