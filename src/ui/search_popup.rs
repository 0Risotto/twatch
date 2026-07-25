use crate::app::App;
use crate::model::SearchResult;
use crate::ui::{centered_rect, format_size};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup = centered_rect(78, 58, area);

    let results: &[SearchResult] = current_page(app);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Search ")
        .border_style(Style::default().fg(app.theme.palette.accent))
        .style(app.theme.surface_style());

    frame.render_widget(block.clone(), popup);
    let inner = block.inner(popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let search_title = if app.torrent_search_query.is_empty() {
        "Query: ".to_string()
    } else if app.torrent_search_query != app.search_fetched_query && !results.is_empty() {
        let count =
            visible_filtered_count(results, &app.torrent_search_query, &app.search_fetched_query);
        format!("Filter: {}  ({}/{})", app.torrent_search_query, count, results.len())
    } else {
        format!("Query: {}  (Page {})", app.torrent_search_query, app.search_page + 1)
    };
    let bar = Paragraph::new(Span::styled(search_title, app.theme.warning_style())).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.palette.warning)),
    );
    frame.render_widget(bar, chunks[0]);

    if app.search_config.url.is_empty() {
        let mut lines =
            vec![Line::from(Span::styled("No API configured.", app.theme.dimmed_style()))];
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press [c] to configure your endpoint.",
            app.theme.accent_style(),
        )));
        let msg = Paragraph::new(Text::from(lines)).centered();
        frame.render_widget(msg, chunks[1]);
    } else if app.search_busy {
        let msg =
            Paragraph::new(Line::from(Span::styled("Searching...", app.theme.warning_style())));
        frame.render_widget(msg, chunks[1]);
    } else if results.is_empty() && app.search_fetched_query.is_empty() {
        let msg = Paragraph::new(Text::from(vec![
            Line::from(Span::styled("Type a query and press Enter.", app.theme.dimmed_style())),
            Line::from(""),
            Line::from(Span::styled("Press [c] to change endpoint.", app.theme.accent_style())),
        ]))
        .centered();
        frame.render_widget(msg, chunks[1]);
    } else if results.is_empty() {
        let msg = Paragraph::new(Text::from(vec![
            Line::from(Span::styled("No results on this page.", app.theme.dimmed_style())),
            Line::from(""),
            Line::from(Span::styled("Try ← or → for other pages.", app.theme.accent_style())),
        ]))
        .centered();
        frame.render_widget(msg, chunks[1]);
    } else {
        draw_results_table(frame, chunks[1], app, results);
    }

    let footer_text = if results.is_empty() && app.search_fetched_query.is_empty() {
        "[Enter] fetch  [c] config  [Esc] close"
    } else {
        "[Enter] add  [← →] page  [c] config  [/] clear  [Esc] close"
    };
    let footer = Paragraph::new(Line::from(Span::styled(footer_text, app.theme.dimmed_style())));
    frame.render_widget(footer, chunks[2]);

    if app.search_config_open {
        draw_config_overlay(frame, area, app);
    }
}

fn draw_results_table(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App,
    page_results: &[SearchResult],
) {
    let available_height = area.height as usize;
    let header_height = 2;

    let filtered: Vec<&SearchResult> = page_results
        .iter()
        .filter(|r| {
            app.torrent_search_query.is_empty()
                || app.torrent_search_query == app.search_fetched_query
                || crate::model::fuzzy_match(&app.torrent_search_query, &r.name)
        })
        .collect();

    let row_count = available_height.saturating_sub(header_height).min(filtered.len());

    let widths = [
        Constraint::Length(5),
        Constraint::Min(25),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(12),
    ];

    let accent = app.theme.palette.accent;
    let seeders = app.theme.palette.badge_dl;
    let leechers = app.theme.palette.folder;
    let size_col = app.theme.palette.badge_stream;

    let header = Row::new(vec![
        Cell::from(Span::styled("#", Style::default().fg(app.theme.palette.text_dimmed))),
        Cell::from(Span::styled("  ▶ Name", Style::default().fg(accent))),
        Cell::from(Span::styled("⬆Seeders", Style::default().fg(seeders))),
        Cell::from(Span::styled("⬇Leechers", Style::default().fg(leechers))),
        Cell::from(Span::styled("Size", Style::default().fg(size_col))),
    ])
    .style(Style::default().bg(app.theme.palette.hover))
    .bottom_margin(1);

    let rows: Vec<Row> = filtered
        .iter()
        .take(row_count)
        .enumerate()
        .map(|(i, r)| {
            let is_sel = i == app.search_selected;
            let name_style = if is_sel { app.theme.accent_style() } else { Style::default() };
            let prefix = if is_sel { "▶ " } else { "  " };

            Row::new(vec![
                Cell::from(Span::styled(format!(" {}", i + 1), Style::default())),
                Cell::from(Span::styled(
                    format!("{}{}", prefix, truncate(&r.name, 60)),
                    name_style,
                )),
                Cell::from(Span::styled(format_seeders(r.seeders), Style::default().fg(seeders))),
                Cell::from(Span::styled(format!("{}", r.leechers), Style::default().fg(leechers))),
                Cell::from(Span::styled(format_size(r.size), Style::default().fg(size_col))),
            ])
            .style(Style::default().bg(if is_sel {
                app.theme.palette.hover
            } else {
                Color::Reset
            }))
        })
        .collect();

    let mut state = TableState::default().with_selected(Some(app.search_selected));

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(2)
        .style(Style::default().fg(app.theme.palette.fg).bg(Color::Reset));

    frame.render_stateful_widget(table, area, &mut state);
}

fn visible_filtered_count(results: &[SearchResult], query: &str, fetched: &str) -> usize {
    results
        .iter()
        .filter(|r| {
            query.is_empty() || query == fetched || crate::model::fuzzy_match(query, &r.name)
        })
        .count()
}

fn format_seeders(n: u64) -> String {
    if n >= 1000 { format!("{:.1}k", n as f64 / 1000.0) } else { format!("{n}") }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}...", &s[..max - 3]) } else { s.to_string() }
}

fn draw_config_overlay(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let popup = centered_rect(60, 25, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" API Endpoint ")
        .border_style(Style::default().fg(app.theme.palette.accent))
        .style(app.theme.surface_style());

    frame.render_widget(block.clone(), popup);
    let inner = block.inner(popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Min(1)])
        .split(inner);

    let url_line = format!("▸ {}", app.search_config_input);
    let input = Paragraph::new(Line::from(Span::styled(url_line, app.theme.text_style()))).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.palette.accent)),
    );
    frame.render_widget(input, chunks[0]);

    let help = Paragraph::new(Text::from(vec![
        Line::from(Span::styled("Paste your API endpoint URL.", app.theme.dimmed_style())),
        Line::from(Span::styled(
            "Use {query} as a placeholder for the search term.",
            app.theme.dimmed_style(),
        )),
        Line::from(""),
        Line::from(Span::styled("[Enter] Save  [Esc] Cancel", app.theme.accent_style())),
    ]))
    .centered();
    frame.render_widget(help, chunks[1]);
}

fn current_page(app: &App) -> &[SearchResult] {
    let start = app.search_page as usize * 20;
    let all = &app.search_all_results;
    if start >= all.len() {
        return &[];
    }
    let end = (start + 20).min(all.len());
    &all[start..end]
}
