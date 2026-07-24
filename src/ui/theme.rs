//! UI theme helpers — palettes, themes, layout, styling, formatting.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders},
};

// ── Palette ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub text_primary: Color,
    pub text_dimmed: Color,
    pub success: Color,
    pub warning: Color,
    pub stream_badge: Color,
}

// ── Theme ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub name: &'static str,
    pub palette: Palette,
}

impl Theme {
    pub const ALL: [Theme; 10] = [
        Theme::TOKYONIGHT_DARK,
        Theme::TOKYONIGHT_LIGHT,
        Theme::KANAGAWA_DARK,
        Theme::KANAGAWA_LIGHT,
        Theme::NORD_DARK,
        Theme::NORD_LIGHT,
        Theme::GRUVBOX_DARK,
        Theme::GRUVBOX_LIGHT,
        Theme::CATPPUCCIN_MOCHA,
        Theme::CATPPUCCIN_LATTE,
    ];

    // ── Individual themes ────────────────────────────────────────

    const TOKYONIGHT_DARK: Theme = Theme {
        name: "TokyoNight Dark",
        palette: Palette {
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(192, 202, 245),
            accent: Color::Rgb(122, 162, 247),
            text_primary: Color::Rgb(192, 202, 245),
            text_dimmed: Color::Rgb(86, 95, 137),
            success: Color::Rgb(158, 206, 106),
            warning: Color::Rgb(224, 175, 104),
            stream_badge: Color::Rgb(187, 154, 247),
        },
    };

    const TOKYONIGHT_LIGHT: Theme = Theme {
        name: "TokyoNight Light",
        palette: Palette {
            bg: Color::Rgb(225, 226, 231),
            fg: Color::Rgb(55, 96, 191),
            accent: Color::Rgb(46, 125, 233),
            text_primary: Color::Rgb(55, 96, 191),
            text_dimmed: Color::Rgb(132, 140, 181),
            success: Color::Rgb(88, 117, 57),
            warning: Color::Rgb(140, 108, 62),
            stream_badge: Color::Rgb(152, 84, 241),
        },
    };

    const KANAGAWA_DARK: Theme = Theme {
        name: "Kanagawa Dark",
        palette: Palette {
            bg: Color::Rgb(31, 31, 40),
            fg: Color::Rgb(220, 215, 186),
            accent: Color::Rgb(126, 156, 216),
            text_primary: Color::Rgb(220, 215, 186),
            text_dimmed: Color::Rgb(114, 113, 105),
            success: Color::Rgb(152, 187, 108),
            warning: Color::Rgb(230, 195, 132),
            stream_badge: Color::Rgb(149, 127, 184),
        },
    };

    const KANAGAWA_LIGHT: Theme = Theme {
        name: "Kanagawa Light",
        palette: Palette {
            bg: Color::Rgb(242, 236, 188),
            fg: Color::Rgb(84, 84, 100),
            accent: Color::Rgb(77, 105, 155),
            text_primary: Color::Rgb(84, 84, 100),
            text_dimmed: Color::Rgb(138, 137, 128),
            success: Color::Rgb(111, 137, 78),
            warning: Color::Rgb(222, 152, 0),
            stream_badge: Color::Rgb(118, 107, 144),
        },
    };

    const NORD_DARK: Theme = Theme {
        name: "Nord Dark",
        palette: Palette {
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(216, 222, 233),
            accent: Color::Rgb(136, 192, 208),
            text_primary: Color::Rgb(216, 222, 233),
            text_dimmed: Color::Rgb(76, 86, 106),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            stream_badge: Color::Rgb(180, 142, 173),
        },
    };

    const NORD_LIGHT: Theme = Theme {
        name: "Nord Light",
        palette: Palette {
            bg: Color::Rgb(236, 239, 244),
            fg: Color::Rgb(46, 52, 64),
            accent: Color::Rgb(94, 129, 172),
            text_primary: Color::Rgb(46, 52, 64),
            text_dimmed: Color::Rgb(129, 161, 193),
            success: Color::Rgb(122, 156, 102),
            warning: Color::Rgb(199, 151, 63),
            stream_badge: Color::Rgb(158, 122, 168),
        },
    };

    const GRUVBOX_DARK: Theme = Theme {
        name: "Gruvbox Dark",
        palette: Palette {
            bg: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(235, 219, 178),
            accent: Color::Rgb(131, 165, 152),
            text_primary: Color::Rgb(235, 219, 178),
            text_dimmed: Color::Rgb(146, 131, 116),
            success: Color::Rgb(184, 187, 38),
            warning: Color::Rgb(250, 189, 47),
            stream_badge: Color::Rgb(211, 134, 155),
        },
    };

    const GRUVBOX_LIGHT: Theme = Theme {
        name: "Gruvbox Light",
        palette: Palette {
            bg: Color::Rgb(251, 241, 199),
            fg: Color::Rgb(60, 56, 54),
            accent: Color::Rgb(7, 102, 120),
            text_primary: Color::Rgb(60, 56, 54),
            text_dimmed: Color::Rgb(124, 111, 100),
            success: Color::Rgb(121, 116, 14),
            warning: Color::Rgb(181, 118, 20),
            stream_badge: Color::Rgb(143, 63, 113),
        },
    };

    const CATPPUCCIN_MOCHA: Theme = Theme {
        name: "Catppuccin Mocha",
        palette: Palette {
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            accent: Color::Rgb(137, 180, 250),
            text_primary: Color::Rgb(205, 214, 244),
            text_dimmed: Color::Rgb(108, 112, 134),
            success: Color::Rgb(166, 227, 161),
            warning: Color::Rgb(249, 226, 175),
            stream_badge: Color::Rgb(203, 166, 247),
        },
    };

    const CATPPUCCIN_LATTE: Theme = Theme {
        name: "Catppuccin Latte",
        palette: Palette {
            bg: Color::Rgb(239, 241, 245),
            fg: Color::Rgb(76, 79, 105),
            accent: Color::Rgb(30, 102, 245),
            text_primary: Color::Rgb(76, 79, 105),
            text_dimmed: Color::Rgb(156, 160, 176),
            success: Color::Rgb(64, 160, 43),
            warning: Color::Rgb(223, 142, 29),
            stream_badge: Color::Rgb(136, 57, 239),
        },
    };

    // ── Convenience methods ───────────────────────────────────────

    #[must_use]
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.palette.accent).bg(self.palette.bg)
    }

    #[must_use]
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.palette.text_primary).bg(self.palette.bg)
    }

    #[must_use]
    pub fn dimmed_style(&self) -> Style {
        Style::default().fg(self.palette.text_dimmed).bg(self.palette.bg)
    }

    #[must_use]
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.palette.success).bg(self.palette.bg)
    }

    #[must_use]
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.palette.warning).bg(self.palette.bg)
    }

    #[must_use]
    pub fn stream_style(&self) -> Style {
        Style::default().fg(self.palette.stream_badge).bg(self.palette.bg)
    }

    #[must_use]
    pub fn bg_style(&self) -> Style {
        Style::default().bg(self.palette.bg)
    }

    /// Cycle to the next theme in ALL, wrapping around.
    #[must_use]
    pub fn next(self) -> Self {
        let pos = Self::ALL.iter().position(|t| t == &self).unwrap_or(0);
        Self::ALL[(pos + 1) % Self::ALL.len()]
    }

    /// Look up a theme by its kebab-case config name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| {
            let config_name = Self::to_config_name(t.name);
            config_name == name || t.name == name
        })
    }

    /// Convert a display name to a kebab-case config key.
    #[must_use]
    pub fn to_config_name(name: &str) -> String {
        name.to_lowercase().replace(' ', "-")
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::ALL[0]
    }
}

// ── Layout / formatting helpers ──────────────────────────────────────

#[must_use]
pub fn styled_block(title: &str, color: Color) -> Block<'_> {
    Block::default().borders(Borders::ALL).title(title).border_style(Style::default().fg(color))
}

#[must_use]
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[must_use]
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 { format!("{bytes} B") } else { format!("{size:.1} {}", UNITS[unit_idx]) }
}

#[must_use]
pub fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_size(bytes_per_sec))
}

/// Build a footer line where `[key]` parts are styled with the accent color
/// and the text between them uses the dimmed color.
#[must_use]
pub fn color_footer<'a>(text: &str, theme: &Theme) -> ratatui::text::Line<'a> {
    let accent = theme.accent_style();
    let dimmed = theme.dimmed_style();
    let mut spans: Vec<ratatui::text::Span<'a>> = Vec::new();
    let mut remainder = text;

    while let Some(start) = remainder.find('[') {
        if start > 0 {
            spans.push(Span::styled(remainder[..start].to_string(), dimmed));
        }
        remainder = &remainder[start..];
        if let Some(end) = remainder.find(']') {
            spans.push(Span::styled(remainder[..=end].to_string(), accent));
            remainder = &remainder[end + 1..];
        } else {
            spans.push(Span::styled(remainder.to_string(), dimmed));
            break;
        }
    }
    if !remainder.is_empty() {
        spans.push(Span::styled(remainder.to_string(), dimmed));
    }
    ratatui::text::Line::from(spans)
}
