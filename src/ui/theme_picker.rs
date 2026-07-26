use crate::app::App;
use crate::ui::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::centered_rect;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
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

    let all = Theme::ALL;
    let filter = app.theme_state.picker_filter.to_lowercase();
    let filtered: Vec<(usize, &Theme)> = all
        .iter()
        .enumerate()
        .filter(|(_, t)| filter.is_empty() || t.name.to_lowercase().contains(&filter))
        .collect();

    let visible_count = (chunks[0].height as usize).min(filtered.len());
    let scroll = if filtered.is_empty() {
        0
    } else {
        let sel = app.theme_state.picker_selected.min(filtered.len().saturating_sub(1));
        if sel < app.theme_state.picker_scroll {
            sel
        } else if sel >= app.theme_state.picker_scroll + visible_count {
            sel - visible_count + 1
        } else {
            app.theme_state.picker_scroll
        }
        .min(filtered.len().saturating_sub(visible_count))
    };

    let items: Vec<ListItem> = filtered
        .iter()
        .skip(scroll)
        .take(visible_count)
        .enumerate()
        .map(|(i, (_, theme))| {
            let is_current = **theme == app.theme_state.theme;
            let is_selected = i + scroll == app.theme_state.picker_selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let suffix = if theme.name.contains("Light") || theme.name.contains("Latte") {
                "  [Light]"
            } else {
                "  [Dark]"
            };
            let style = if is_selected {
                app.theme_state.theme.accent_style()
            } else if is_current {
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

    // Search bar
    let search_text = if app.theme_state.picker_filter.is_empty() {
        "Filter: ".to_string()
    } else {
        format!("Filter: {}", app.theme_state.picker_filter)
    };
    let bar = Paragraph::new(Span::styled(search_text, app.theme_state.theme.warning_style()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme_state.theme.palette.warning)),
        );
    frame.render_widget(bar, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(Span::styled(
        "[↑↓] Navigate  [Enter] Select  [Esc] Close  Type to filter",
        app.theme_state.theme.dimmed_style(),
    )));
    frame.render_widget(footer, chunks[2]);
}
