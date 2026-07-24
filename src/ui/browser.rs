use crate::app::App;
use crate::model::DisplayEntry;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use super::{format_size, styled_block};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    if app.is_searching {
        draw_with_search(frame, area, app);
    } else {
        draw_normal(frame, area, app);
    }
}

fn draw_normal(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    // Header
    let torrent = app.torrent_name.as_deref().unwrap_or("Torrent");
    let selected = app.selected_count();
    let title_text = if selected > 0 {
        format!(" {} — {} selected for download", torrent, selected)
    } else {
        format!(" {}", torrent)
    };
    let title = Paragraph::new(Span::styled(title_text, Style::default().fg(Color::Cyan)))
        .block(styled_block(" Torrent Contents ", Color::Cyan));
    frame.render_widget(title, chunks[0]);

    // Entries
    let items = build_items(app, false);
    let list = List::new(items)
        .block(styled_block(" Files ", Color::White))
        .highlight_style(Style::default());
    frame.render_widget(list, chunks[1]);

    // Footer
    frame.render_widget(
        footer(
            "[/] search  [Enter] expand  [w] watch  [d] dl  [Space] toggle  [r] rename  [q] back",
        ),
        chunks[2],
    );
}

fn draw_with_search(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    let torrent = app.torrent_name.as_deref().unwrap_or("Torrent");
    let title =
        Paragraph::new(Span::styled(format!(" {}", torrent), Style::default().fg(Color::Cyan)))
            .block(styled_block(" Torrent Contents ", Color::Cyan));
    frame.render_widget(title, chunks[0]);

    // Search bar
    let matches = app
        .display_entries
        .iter()
        .filter(|e| crate::app::entry_matches(&app.search_query, e))
        .count();
    let search_text = if app.search_query.is_empty() {
        "Search: ".to_string()
    } else {
        let left = format!("Search: {}", app.search_query);
        let right = format!("| {}", matches);
        let pad = (chunks[1].width.saturating_sub(2) as usize)
            .saturating_sub(left.len())
            .saturating_sub(right.len());
        format!("{}{}{}", left, " ".repeat(pad), right)
    };
    let bar = Paragraph::new(Span::styled(search_text, Style::default().fg(Color::Yellow)))
        .block(styled_block(" Search ", Color::Yellow));
    frame.render_widget(bar, chunks[1]);

    // Entries (filtered)
    let items = build_items(app, true);
    let list = List::new(items)
        .block(styled_block(" Files ", Color::White))
        .highlight_style(Style::default());
    frame.render_widget(list, chunks[2]);

    // Footer
    frame.render_widget(footer("[Esc] cancel  [q] exit  [Enter] select"), chunks[3]);
}

fn build_items(app: &App, filtered: bool) -> Vec<ListItem<'_>> {
    let visible = if filtered {
        app.visible_entries()
    } else {
        app.display_entries.iter().enumerate().collect()
    };

    visible
        .iter()
        .map(|(display_idx, entry)| {
            let is_hovered = *display_idx == app.selected_file;

            match entry {
                DisplayEntry::Folder { name, depth, expanded } => {
                    let indent = " ".repeat(depth * 2);
                    let icon = if *expanded { "▹" } else { "▸" };
                    let (prefix, style) = if is_hovered {
                        ("▶ ", Style::default().fg(Color::Cyan))
                    } else {
                        ("  ", Style::default().fg(Color::Yellow))
                    };
                    ListItem::new(Line::from(vec![
                        Span::raw(indent),
                        Span::styled(prefix, Style::default().fg(Color::Cyan)),
                        Span::styled(format!("{icon} {name}/"), style),
                    ]))
                }
                DisplayEntry::File { file, depth } => {
                    let indent = " ".repeat(depth * 2);
                    let checked = app.selected_files.get(file.index).copied().unwrap_or(false);
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
                        Span::raw(indent),
                        Span::styled(prefix, Style::default().fg(Color::Cyan)),
                        Span::styled(&file.name, name_style),
                        Span::styled(
                            format!("  ({})", format_size(file.size)),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]))
                }
            }
        })
        .collect()
}

fn footer(text: &str) -> Paragraph<'_> {
    Paragraph::new(Line::from(Span::styled(text, Style::default().fg(Color::DarkGray)))).centered()
}
