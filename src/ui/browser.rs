//! Browser screen renderer.

use crate::app::App;
use crate::app::screens::browser::{ConfirmViewModel, EntryViewModel, ViewModel};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Clear, List, ListItem, Paragraph},
};

use super::badges::badge_for;
use super::theme::{centered_rect, color_footer, format_size, styled_block};

pub fn render(frame: &mut Frame, area: Rect, app: &App, vm: ViewModel<'_>) {
    match vm.searching {
        Some(query) => render_searching(frame, area, app, &vm, query),
        None => render_normal(frame, area, app, &vm),
    }

    if let Some(confirm) = vm.confirm {
        render_delete_confirm(frame, area, app, confirm);
    }
}

fn render_normal(frame: &mut Frame, area: Rect, app: &App, vm: &ViewModel<'_>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let title_text = if vm.selected_count > 0 {
        format!(" {} — {} selected for download", vm.title, vm.selected_count)
    } else {
        format!(" {}", vm.title)
    };
    let title = Paragraph::new(Span::styled(title_text, app.theme_state.theme.accent_style()))
        .block(styled_block(" Torrent Contents ", app.theme_state.theme.palette.accent));
    frame.render_widget(title, chunks[0]);

    let items = build_items(app, vm);
    let list = List::new(items)
        .block(styled_block(" Files ", app.theme_state.theme.palette.border))
        .highlight_style(Style::default());
    frame.render_widget(list, chunks[1]);

    let footer = Paragraph::new(color_footer(
        "[/] search  [Enter] expand  [w] watch  [d] dl  [x] delete  [Space] toggle  [r] rename  [q] back",
        &app.theme_state.theme,
    ))
    .centered();
    frame.render_widget(footer, chunks[2]);
}

fn render_searching(frame: &mut Frame, area: Rect, app: &App, vm: &ViewModel<'_>, query: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(Span::styled(
        format!(" {}", vm.title),
        app.theme_state.theme.accent_style(),
    ))
    .block(styled_block(" Torrent Contents ", app.theme_state.theme.palette.accent));
    frame.render_widget(title, chunks[0]);

    let matches = vm.search_matches.unwrap_or(0);
    let search_text = if query.is_empty() {
        "Search: ".to_string()
    } else {
        let left = format!("Search: {query}");
        let right = format!("| {matches}");
        let pad = (chunks[1].width.saturating_sub(2) as usize)
            .saturating_sub(left.len())
            .saturating_sub(right.len());
        format!("{left}{}{right}", " ".repeat(pad))
    };
    let bar = Paragraph::new(Span::styled(search_text, app.theme_state.theme.warning_style()))
        .block(styled_block(" Search ", app.theme_state.theme.palette.warning));
    frame.render_widget(bar, chunks[1]);

    let items = build_items(app, vm);
    let list = List::new(items)
        .block(styled_block(" Files ", app.theme_state.theme.palette.border))
        .highlight_style(Style::default());
    frame.render_widget(list, chunks[2]);

    let footer = Paragraph::new(color_footer(
        "[Esc] cancel  [q] exit  [Enter] select",
        &app.theme_state.theme,
    ))
    .centered();
    frame.render_widget(footer, chunks[3]);
}

fn build_items<'a>(app: &App, vm: &'a ViewModel<'a>) -> Vec<ListItem<'a>> {
    vm.entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_hovered = i == vm.cursor;
            match entry {
                EntryViewModel::Folder { name, depth, expanded } => {
                    let indent = " ".repeat(depth * 2);
                    let icon = if *expanded { "▹" } else { "▸" };
                    let (prefix, style) = if is_hovered {
                        ("▶ ", app.theme_state.theme.accent_style())
                    } else {
                        ("  ", app.theme_state.theme.folder_style())
                    };
                    ListItem::new(Line::from(vec![
                        Span::raw(indent),
                        Span::styled(prefix, app.theme_state.theme.accent_style()),
                        Span::styled(format!("{icon} {name}/"), style),
                    ]))
                }
                EntryViewModel::File { file, depth, status, selected } => {
                    let indent = " ".repeat(depth * 2);
                    let prefix = match (is_hovered, selected) {
                        (true, _) => "▶ ",
                        (false, true) => " ✓ ",
                        (false, false) => "   ",
                    };
                    let name_style = if is_hovered {
                        app.theme_state.theme.accent_style()
                    } else if *selected {
                        app.theme_state.theme.success_style()
                    } else {
                        app.theme_state.theme.text_style()
                    };
                    let mut spans = vec![
                        Span::raw(indent),
                        Span::styled(prefix, app.theme_state.theme.accent_style()),
                        Span::styled(&file.name, name_style),
                    ];
                    if let Some(badge) = badge_for(file, status, &app.theme_state.theme) {
                        spans.push(badge);
                    }
                    spans.push(Span::styled(
                        format!("  ({})", format_size(file.size)),
                        app.theme_state.theme.dimmed_style(),
                    ));
                    ListItem::new(Line::from(spans))
                }
            }
        })
        .collect()
}

fn render_delete_confirm(frame: &mut Frame, area: Rect, app: &App, confirm: ConfirmViewModel<'_>) {
    let file_count = confirm.total;
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

    let block = styled_block(" Delete files? ", app.theme_state.theme.palette.warning);
    frame.render_widget(block, popup);

    let header = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            format!(" You are deleting {} file(s):", file_count),
            app.theme_state.theme.warning_style(),
        )),
        Line::from(""),
    ]));
    frame.render_widget(header, chunks[1]);

    let mut line_idx = 3;
    for name in confirm.files.iter().take(8) {
        let display =
            if name.len() > 50 { format!("   {}...", &name[..47]) } else { format!("   {name}") };
        frame.render_widget(
            Paragraph::new(Span::styled(display, app.theme_state.theme.dimmed_style())),
            chunks[line_idx],
        );
        line_idx += 1;
    }
    if file_count > 8 {
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("   ... and {} more", file_count - 8),
                app.theme_state.theme.dimmed_style(),
            )),
            chunks[line_idx],
        );
        line_idx += 1;
    }

    let yes_style = if confirm.yes {
        app.theme_state.theme.accent_style()
    } else {
        app.theme_state.theme.text_style()
    };
    let no_style = if !confirm.yes {
        app.theme_state.theme.accent_style()
    } else {
        app.theme_state.theme.text_style()
    };

    let buttons = Line::from(vec![
        Span::styled("[Yes]", yes_style),
        Span::raw("  "),
        Span::styled("[No]", no_style),
    ]);
    let help = Line::from(Span::styled(
        "Enter to confirm, Esc/q to cancel, h/l to toggle",
        app.theme_state.theme.dimmed_style(),
    ));

    frame.render_widget(Paragraph::new(Text::from(vec![buttons, help])), chunks[line_idx + 1]);
}
