use super::theme::{format_size, format_speed, styled_block};
use crate::app::App;
use crate::model::ActiveDownload;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Gauge, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let downloads = &app.active_downloads;

    let title = format!(" Downloads ({}) ", downloads.len());
    let block = styled_block(&title, app.theme.palette.accent);

    if downloads.is_empty() {
        let empty = Paragraph::new(Span::styled("No active downloads", app.theme.dimmed_style()))
            .block(block)
            .centered();
        frame.render_widget(empty, area);
        return;
    }

    let item_height: u16 = 5;
    let max_items = area.height.saturating_sub(2) / item_height;

    let constraints: Vec<_> = (0..max_items)
        .map(|_| Constraint::Length(item_height))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();
    let inner = block.inner(area);
    let chunks =
        Layout::default().direction(Direction::Vertical).constraints(constraints).split(inner);

    frame.render_widget(block, area);

    for (i, dl) in downloads.iter().take(max_items as usize).enumerate() {
        draw_download_entry(frame, chunks[i], dl, app);
    }
}

fn draw_download_entry(frame: &mut Frame, area: Rect, dl: &ActiveDownload, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(area);

    let kind = if dl.is_streaming {
        Span::styled("[stream]", app.theme.badge_stream_style())
    } else {
        Span::styled("[dl]", app.theme.warning_style())
    };
    let name_line = Line::from(vec![
        kind,
        Span::raw(" "),
        Span::styled(format!("{} / {}", dl.torrent_name, dl.file_name), app.theme.text_style()),
    ]);

    let name_para = Paragraph::new(Text::from(name_line)).wrap(Wrap { trim: true });
    frame.render_widget(name_para, chunks[0]);

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let pct = ((dl.progress * 100.0).clamp(0.0, 100.0)) as u16;
    let gauge_color = if dl.progress >= 1.0 {
        app.theme.palette.success
    } else if dl.is_streaming {
        app.theme.palette.badge_stream
    } else {
        app.theme.palette.badge_dl
    };
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color))
        .percent(pct)
        .label(format!("{:.0}%", dl.progress * 100.0));
    frame.render_widget(gauge, chunks[1]);

    let stat = if dl.progress >= 1.0 {
        Span::styled("Done", app.theme.success_style())
    } else {
        Span::styled(
            format!(
                "{}  {} / {}",
                format_speed(dl.download_speed),
                format_size(dl.downloaded),
                format_size(dl.total_size),
            ),
            app.theme.dimmed_style(),
        )
    };
    frame.render_widget(Paragraph::new(Line::from(stat)), chunks[2]);
}
