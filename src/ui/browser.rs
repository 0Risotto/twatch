use crate::app::App;
use crate::model::DisplayEntry;
use crate::ui::theme::centered_rect;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Clear, List, ListItem, Paragraph},
};

use super::{color_footer, format_size, styled_block};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    if app.is_searching {
        draw_with_search(frame, area, app);
    } else {
        draw_normal(frame, area, app);
    }

    if app.confirm_delete {
        draw_delete_confirm(frame, area, app);
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
    let title = Paragraph::new(Span::styled(title_text, app.theme.accent_style()))
        .block(styled_block(" Torrent Contents ", app.theme.palette.accent));
    frame.render_widget(title, chunks[0]);

    // Entries
    let items = build_items(app, false);
    let list = List::new(items)
        .block(styled_block(" Files ", app.theme.palette.border))
        .highlight_style(Style::default());
    frame.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new(color_footer(
        "[/] search  [Enter] expand  [w] watch  [d] dl  [x] delete  [Space] toggle  [r] rename  [q] back",
        &app.theme,
    ))
    .centered();
    frame.render_widget(footer, chunks[2]);
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
    let title = Paragraph::new(Span::styled(format!(" {}", torrent), app.theme.accent_style()))
        .block(styled_block(" Torrent Contents ", app.theme.palette.accent));
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
    let bar = Paragraph::new(Span::styled(search_text, app.theme.warning_style()))
        .block(styled_block(" Search ", app.theme.palette.warning));
    frame.render_widget(bar, chunks[1]);

    // Entries (filtered)
    let items = build_items(app, true);
    let list = List::new(items)
        .block(styled_block(" Files ", app.theme.palette.border))
        .highlight_style(Style::default());
    frame.render_widget(list, chunks[2]);

    // Footer
    let footer = Paragraph::new(color_footer("[Esc] cancel  [q] exit  [Enter] select", &app.theme))
        .centered();
    frame.render_widget(footer, chunks[3]);
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
                        ("▶ ", app.theme.accent_style())
                    } else {
                        ("  ", app.theme.folder_style())
                    };
                    ListItem::new(Line::from(vec![
                        Span::raw(indent),
                        Span::styled(prefix, app.theme.accent_style()),
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
                        app.theme.accent_style()
                    } else if checked {
                        app.theme.success_style()
                    } else {
                        app.theme.text_style()
                    };
                    let mut spans = vec![
                        Span::raw(indent),
                        Span::styled(prefix, app.theme.accent_style()),
                        Span::styled(&file.name, name_style),
                    ];
                    if app.watched_files.iter().any(|f| f == &file.name) {
                        spans.push(Span::styled(" [watched]", app.theme.badge_stream_style()));
                    } else if app.downloading_files.iter().any(|f| f == &file.name) {
                        spans.push(Span::styled(" [downloading]", app.theme.warning_style()));
                    } else if app.downloaded_files.iter().any(|f| f == &file.name) {
                        spans.push(Span::styled(" [downloaded]", app.theme.badge_dl_style()));
                    }
                    spans.push(Span::styled(
                        format!("  ({})", format_size(file.size)),
                        app.theme.dimmed_style(),
                    ));
                    ListItem::new(Line::from(spans))
                }
            }
        })
        .collect()
}

fn draw_delete_confirm(frame: &mut Frame, area: Rect, app: &App) {
    let selected: Vec<&str> = app
        .selected_files
        .iter()
        .enumerate()
        .filter(|(_, s)| **s)
        .filter_map(|(i, _)| app.files.get(i).map(|f| f.name.as_str()))
        .collect();

    let file_count = selected.len();
    #[allow(clippy::cast_possible_truncation)]
    let popup_height = (file_count.min(8) + 8) as u16;
    let popup = centered_rect(60, popup_height, area);

    frame.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [Constraint::Length(1)]
                .iter()
                .copied()
                .cycle()
                .take(file_count.min(8) + 8)
                .chain(std::iter::once(Constraint::Min(0)))
                .collect::<Vec<_>>(),
        )
        .split(popup);

    let block = styled_block(" Delete files? ", app.theme.palette.warning);
    frame.render_widget(block, popup);

    let header = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            format!(" You are deleting {} file(s):", file_count),
            app.theme.warning_style(),
        )),
        Line::from(""),
    ]));
    frame.render_widget(header, chunks[1]);

    let mut line_idx = 3;
    for name in selected.iter().take(8) {
        let display =
            if name.len() > 50 { format!("   {}...", &name[..47]) } else { format!("   {name}") };
        frame.render_widget(
            Paragraph::new(Span::styled(display, app.theme.dimmed_style())),
            chunks[line_idx],
        );
        line_idx += 1;
    }
    if file_count > 8 {
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("   ... and {} more", file_count - 8),
                app.theme.dimmed_style(),
            )),
            chunks[line_idx],
        );
        line_idx += 1;
    }

    let yes_style =
        if app.confirm_delete_yes { app.theme.accent_style() } else { app.theme.text_style() };
    let no_style =
        if !app.confirm_delete_yes { app.theme.accent_style() } else { app.theme.text_style() };

    let buttons = Line::from(vec![
        Span::styled("[Yes]", yes_style),
        Span::raw("  "),
        Span::styled("[No]", no_style),
    ]);
    let help = Line::from(Span::styled(
        "Enter to confirm, Esc/q to cancel, h/l to toggle",
        app.theme.dimmed_style(),
    ));

    frame.render_widget(Paragraph::new(Text::from(vec![buttons, help])), chunks[line_idx + 1]);
}
