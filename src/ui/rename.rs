//! Rename popup renderer.

use crate::app::App;
use crate::app::screens::rename::ViewModel;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use super::theme::{centered_rect, styled_block};

pub fn render(frame: &mut Frame, area: Rect, app: &App, vm: ViewModel<'_>) {
    let popup = centered_rect(60, 20, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(popup);

    let text = Text::from(vec![
        Line::from(Span::styled(vm.title, app.theme_state.theme.accent_style())),
        Line::from(Span::styled(format!("▸ {}", vm.value), app.theme_state.theme.text_style())),
    ]);

    let para = Paragraph::new(text)
        .block(styled_block(" Rename ", app.theme_state.theme.palette.accent))
        .style(Style::default());

    frame.render_widget(para, chunks[1]);
}
