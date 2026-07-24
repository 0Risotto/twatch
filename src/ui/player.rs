use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Gauge, Paragraph},
};

use super::{format_size, format_speed, styled_block};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Text::from(Span::styled(
        &app.status_message,
        Style::default().fg(Color::Cyan),
    )))
    .block(styled_block(" Now Playing ", Color::Cyan));
    frame.render_widget(title, chunks[0]);

    // Progress gauge
    if let Some(stats) = &app.stats {
        let progress = stats.progress.min(1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let pct = ((progress * 100.0).clamp(0.0, 100.0)) as u16;

        let gauge = Gauge::default()
            .block(styled_block(" Download Progress ", Color::White))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(pct)
            .label(format!("{:.1}%", progress * 100.0));

        frame.render_widget(gauge, chunks[2]);

        // Stats
        let stats_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(chunks[3]);

        let dl_text = vec![
            Line::from(Span::styled(
                format_speed(stats.download_speed),
                Style::default().fg(Color::Green),
            )),
            Line::from(Span::styled("download", Style::default().fg(Color::DarkGray))),
        ];

        let peers_text = vec![
            Line::from(Span::styled(
                format!("{}", stats.peers),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled("peers", Style::default().fg(Color::DarkGray))),
        ];

        let size_text = vec![
            Line::from(Span::styled(
                format!("{} / {}", format_size(stats.downloaded), format_size(stats.total_size)),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled("downloaded", Style::default().fg(Color::DarkGray))),
        ];

        let dl_para = Paragraph::new(Text::from(dl_text)).centered();
        let peers_para = Paragraph::new(Text::from(peers_text)).centered();
        let size_para = Paragraph::new(Text::from(size_text)).centered();

        frame.render_widget(dl_para, stats_chunks[0]);
        frame.render_widget(peers_para, stats_chunks[1]);
        frame.render_widget(size_para, stats_chunks[2]);
    }

    // Footer
    let footer = Paragraph::new(Text::from(vec![Line::from(Span::styled(
        "[q/Esc] Stop playback and return to browser",
        Style::default().fg(Color::DarkGray),
    ))]))
    .centered();
    frame.render_widget(footer, chunks[4]);
}
