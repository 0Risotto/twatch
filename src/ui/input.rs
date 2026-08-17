//! Input screen renderer.

use crate::app::App;
use crate::app::screens::input::ViewModel;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use super::{centered_rect, color_footer, styled_block};

pub fn render(frame: &mut Frame, area: Rect, app: &App, vm: ViewModel<'_>) {
    let popup_area = centered_rect(60, 30, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(1)])
        .split(popup_area);

    let input_text: Vec<Line> = vec![Line::from(vec![
        Span::styled("▸ ", app.theme_state.theme.input_cursor_style()),
        Span::styled(vm.value, app.theme_state.theme.text_style()),
    ])];

    let input_paragraph = Paragraph::new(Text::from(input_text))
        .block(styled_block(" Magnet / Torrent URL ", app.theme_state.theme.palette.accent))
        .style(app.theme_state.theme.text_style());

    frame.render_widget(input_paragraph, chunks[1]);

    if !vm.value.is_empty() {
        let cursor_hint = format!("cursor at position {}", vm.cursor);
        let hint = Paragraph::new(Text::from(Span::styled(
            cursor_hint,
            app.theme_state.theme.dimmed_style(),
        )))
        .centered();
        frame.render_widget(hint, chunks[2]);
    }

    let instructions = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            "Paste a magnet link or .torrent URL and press Enter",
            app.theme_state.theme.dimmed_style(),
        )),
        color_footer("[Enter] Confirm    [Esc] Back", &app.theme_state.theme),
    ]))
    .centered();

    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    frame.render_widget(instructions, bottom[1]);

    if !app.status_message.is_empty() {
        let status = Paragraph::new(Text::from(Span::styled(
            &app.status_message,
            app.theme_state.theme.warning_style(),
        )))
        .centered();

        let status_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        frame.render_widget(status, status_area[0]);
    }
}
