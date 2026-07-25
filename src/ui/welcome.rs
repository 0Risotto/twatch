use crate::app::App;
use crate::model::MenuEntry;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use super::styled_block;

const ASCII_ART: &str = r#"▄▄▄█████▓ █     █░ ▄▄▄     ▄▄▄█████▓ ▄████▄   ██░ ██
▓  ██▒ ▓▒▓█░ █ ░█░▒████▄   ▓  ██▒ ▓▒▒██▀ ▀█  ▓██░ ██▒
▒ ▓██░ ▒░▒█░ █ ░█ ▒██  ▀█▄ ▒ ▓██░ ▒░▒▓█    ▄ ▒██▀▀██░
░ ▓██▓ ░ ░█░ █ ░█ ░██▄▄▄▄██░ ▓██▓ ░ ▒▓▓▄ ▄██▒░▓█ ░██
  ▒██▒ ░ ░░██▒██▓  ▓█   ▓██▒ ▒██▒ ░ ▒ ▓███▀ ░░▓█▒░██▓
  ▒ ░░   ░ ▓░▒ ▒   ▒▒   ▓▒█░ ▒ ░░   ░ ░▒ ▒  ░ ▒ ░░▒░▒
    ░      ▒ ░ ░    ▒   ▒▒ ░   ░      ░  ▒    ▒ ░▒░ ░
  ░        ░   ░    ░   ▒    ░      ░         ░  ░░ ░
             ░          ░  ░        ░ ░       ░  ░  ░
                                    ░              "#;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let lines: Vec<Line> = ASCII_ART
        .lines()
        .map(|line| Line::from(Span::styled(line.to_string(), app.theme.accent_style())))
        .collect();

    let mut text_lines = vec![Line::from(""), Line::from("")];
    text_lines.extend(lines);
    text_lines.push(Line::from(""));
    text_lines.push(Line::from(""));
    text_lines.push(Line::from(Span::styled(
        "Terminal torrent streaming client",
        app.theme.text_style(),
    )));
    text_lines.push(Line::from(""));

    let menu_styles = [
        app.theme.success_style(),
        app.theme.badge_stream_style(),
        app.theme.warning_style(),
        app.theme.error_style(),
    ];

    for (i, entry) in MenuEntry::ALL.iter().enumerate() {
        let is_sel = i == app.menu_selected;
        let prefix = if is_sel { "  > " } else { "    " };
        let style = if is_sel { app.theme.accent_style() } else { menu_styles[i] };
        text_lines.push(Line::from(Span::styled(format!("{prefix}{}", entry.label()), style)));
    }

    let paragraph = Paragraph::new(Text::from(text_lines))
        .block(styled_block(" twatch ", app.theme.palette.accent))
        .centered();

    frame.render_widget(paragraph, area);
}
