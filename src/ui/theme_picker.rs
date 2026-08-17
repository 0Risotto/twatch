//! Theme picker renderer.

use crate::app::App;
use crate::app::screens::theme_picker::ViewModel;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::centered_rect;

pub fn render(frame: &mut Frame, area: Rect, app: &App, vm: ViewModel<'_>) {
    let popup = centered_rect(55, 70, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Themes ")
        .border_style(Style::default().fg(app.theme_state.theme.palette.accent))
        .style(app.theme_state.theme.surface_style());

    frame.render_widget(block.clone(), popup);

    let inner = block.inner(popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3), Constraint::Length(1)])
        .split(inner);

    let selected_idx = vm.items.iter().position(|t| t.is_selected).unwrap_or(0);
    let visible_count = (chunks[0].height as usize).min(vm.items.len());
    let scroll = if selected_idx < visible_count {
        0
    } else {
        (selected_idx - visible_count + 1).min(vm.items.len().saturating_sub(visible_count))
    };

    let items: Vec<ListItem> = vm
        .items
        .iter()
        .skip(scroll)
        .take(visible_count)
        .map(|theme| {
            let prefix = if theme.is_selected { "▶ " } else { "  " };
            let suffix = if theme.name.contains("Light") || theme.name.contains("Latte") {
                "  [Light]"
            } else {
                "  [Dark]"
            };
            let style = if theme.is_selected {
                app.theme_state.theme.accent_style()
            } else if theme.is_current {
                app.theme_state.theme.success_style()
            } else {
                app.theme_state.theme.text_style()
            };
            let full = format!("{prefix}{}{suffix}", theme.name);
            ListItem::new(Line::from(Span::styled(full, style)))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[0]);

    let search_text = if vm.filter.is_empty() {
        "Filter: ".to_string()
    } else {
        format!("Filter: {}", vm.filter)
    };
    let bar = Paragraph::new(Span::styled(search_text, app.theme_state.theme.warning_style()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme_state.theme.palette.warning)),
        );
    frame.render_widget(bar, chunks[1]);

    let footer = Paragraph::new(Line::from(Span::styled(
        "[↑↓] Navigate  [Enter] Select  [Esc] Close  Type to filter",
        app.theme_state.theme.dimmed_style(),
    )));
    frame.render_widget(footer, chunks[2]);
}
