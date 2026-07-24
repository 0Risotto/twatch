use crate::app::App;
use crate::model::MenuEntry;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
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
        .map(|line| Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Cyan))))
        .collect();

    let mut text_lines = vec![Line::from(""), Line::from("")];
    text_lines.extend(lines);
    text_lines.push(Line::from(""));
    text_lines.push(Line::from(""));
    text_lines.push(Line::from(Span::styled(
        "Terminal torrent streaming client",
        Style::default().fg(Color::White),
    )));
    text_lines.push(Line::from(""));

    // Menu entries
    for (i, entry) in MenuEntry::ALL.iter().enumerate() {
        let is_sel = i == app.menu_selected;
        let prefix = if is_sel { "  > " } else { "    " };
        let style = if is_sel {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        text_lines.push(Line::from(Span::styled(format!("{prefix}{}", entry.label()), style)));
    }

    let paragraph = Paragraph::new(Text::from(text_lines))
        .block(styled_block(" twatch ", Color::Cyan))
        .centered();

    frame.render_widget(paragraph, area);
}
