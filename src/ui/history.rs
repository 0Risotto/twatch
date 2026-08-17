//! History screen renderer.

use crate::app::App;
use crate::app::screens::history::ViewModel;
use crate::model::display_name;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use super::theme::{color_footer, styled_block};

pub fn render(frame: &mut Frame, area: Rect, app: &App, vm: ViewModel) {
    let header_area;
    let list_area;
    let footer_area;

    if vm.searching {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);
        header_area = chunks[0];
        list_area = chunks[2];
        footer_area = chunks[3];
        render_search_bar(frame, chunks[1], app, &vm);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(area);
        header_area = chunks[0];
        list_area = chunks[1];
        footer_area = chunks[2];
    }

    let header = Paragraph::new(Line::from(Span::styled(
        format!(" {} entries in history", vm.entries.len()),
        app.theme_state.theme.accent_style(),
    )))
    .block(styled_block(" History ", app.theme_state.theme.palette.accent));
    frame.render_widget(header, header_area);

    let items = build_items(app, &vm);
    let list = List::new(items)
        .block(styled_block(" Entries ", app.theme_state.theme.palette.border))
        .highlight_style(Style::default());
    frame.render_widget(list, list_area);

    let footer = Paragraph::new(color_footer(
        "[/] search    [Enter] re-add    [r] rename    [d] delete    [q/Esc] back",
        &app.theme_state.theme,
    ))
    .centered();
    frame.render_widget(footer, footer_area);
}

fn render_search_bar(frame: &mut Frame, area: Rect, app: &App, vm: &ViewModel) {
    let search_text = if app.history_query.is_empty() {
        "Search: ".to_string()
    } else {
        let left = format!("Search: {}", app.history_query);
        let right = format!("| {}", vm.entries.len());
        let pad = (area.width.saturating_sub(2) as usize)
            .saturating_sub(left.len())
            .saturating_sub(right.len());
        format!("{left}{}{right}", " ".repeat(pad))
    };
    let bar = Paragraph::new(Span::styled(search_text, app.theme_state.theme.warning_style()))
        .block(styled_block(" Search ", app.theme_state.theme.palette.warning));
    frame.render_widget(bar, area);
}

fn build_items<'a>(app: &App, vm: &'a ViewModel) -> Vec<ListItem<'a>> {
    vm.entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == vm.selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let name = display_name(entry);

            let name_style = if is_selected {
                app.theme_state.theme.accent_style()
            } else {
                app.theme_state.theme.text_style()
            };

            let url_display = if entry.url.len() > 60 {
                format!("{}...", &entry.url[..57])
            } else {
                entry.url.clone()
            };

            let spans = vec![
                Span::styled(prefix, app.theme_state.theme.accent_style()),
                Span::styled(name, name_style),
            ];

            ListItem::new(vec![
                Line::from(spans),
                Line::from(Span::styled(
                    format!("    {url_display}"),
                    app.theme_state.theme.dimmed_style(),
                )),
            ])
        })
        .collect()
}
