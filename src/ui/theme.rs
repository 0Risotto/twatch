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
    pub surface: Color,
    pub accent: Color,
    pub text_primary: Color,
    pub text_dimmed: Color,
    pub input_cursor: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub folder: Color,
    pub gauge: Color,
    pub border: Color,
    pub hover: Color,
    pub badge_dl: Color,
    pub badge_stream: Color,
}

// ── Theme ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub name: &'static str,
    pub palette: Palette,
}

impl Theme {
    pub const ALL: [Theme; 11] = [
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
        Theme::PURPLE,
    ];

    // ── Individual themes ────────────────────────────────────────

    const TOKYONIGHT_DARK: Theme = Theme {
        name: "TokyoNight Dark",
        palette: Palette {
            bg: Color::Rgb(12, 14, 20),
            fg: Color::Rgb(192, 202, 245),
            surface: Color::Rgb(22, 22, 30),
            accent: Color::Rgb(122, 162, 247),
            text_primary: Color::Rgb(192, 202, 245),
            text_dimmed: Color::Rgb(86, 95, 137),
            input_cursor: Color::Rgb(137, 180, 250),
            success: Color::Rgb(158, 206, 106),
            warning: Color::Rgb(224, 175, 104),
            error: Color::Rgb(247, 118, 142),
            folder: Color::Rgb(255, 158, 100),
            gauge: Color::Rgb(122, 162, 247),
            border: Color::Rgb(26, 27, 38),
            hover: Color::Rgb(31, 34, 56),
            badge_dl: Color::Rgb(224, 175, 104),
            badge_stream: Color::Rgb(187, 154, 247),
        },
    };

    const TOKYONIGHT_LIGHT: Theme = Theme {
        name: "TokyoNight Light",
        palette: Palette {
            bg: Color::Rgb(225, 226, 231),
            fg: Color::Rgb(55, 96, 191),
            surface: Color::Rgb(208, 213, 227),
            accent: Color::Rgb(46, 125, 233),
            text_primary: Color::Rgb(55, 96, 191),
            text_dimmed: Color::Rgb(132, 140, 181),
            input_cursor: Color::Rgb(53, 138, 255),
            success: Color::Rgb(88, 117, 57),
            warning: Color::Rgb(140, 108, 62),
            error: Color::Rgb(245, 42, 101),
            folder: Color::Rgb(177, 92, 0),
            gauge: Color::Rgb(46, 125, 233),
            border: Color::Rgb(168, 174, 203),
            hover: Color::Rgb(205, 209, 221),
            badge_dl: Color::Rgb(140, 108, 62),
            badge_stream: Color::Rgb(152, 84, 241),
        },
    };

    const KANAGAWA_DARK: Theme = Theme {
        name: "Kanagawa Dark",
        palette: Palette {
            bg: Color::Rgb(22, 22, 29),
            fg: Color::Rgb(220, 215, 186),
            surface: Color::Rgb(26, 26, 34),
            accent: Color::Rgb(126, 156, 216),
            text_primary: Color::Rgb(220, 215, 186),
            text_dimmed: Color::Rgb(114, 113, 105),
            input_cursor: Color::Rgb(156, 171, 202),
            success: Color::Rgb(152, 187, 108),
            warning: Color::Rgb(230, 195, 132),
            error: Color::Rgb(232, 36, 36),
            folder: Color::Rgb(255, 160, 102),
            gauge: Color::Rgb(126, 156, 216),
            border: Color::Rgb(31, 31, 40),
            hover: Color::Rgb(29, 29, 38),
            badge_dl: Color::Rgb(230, 195, 132),
            badge_stream: Color::Rgb(149, 127, 184),
        },
    };

    const KANAGAWA_LIGHT: Theme = Theme {
        name: "Kanagawa Light",
        palette: Palette {
            bg: Color::Rgb(242, 236, 188),
            fg: Color::Rgb(84, 84, 100),
            surface: Color::Rgb(229, 221, 176),
            accent: Color::Rgb(77, 105, 155),
            text_primary: Color::Rgb(84, 84, 100),
            text_dimmed: Color::Rgb(138, 137, 128),
            input_cursor: Color::Rgb(93, 87, 163),
            success: Color::Rgb(111, 137, 78),
            warning: Color::Rgb(222, 152, 0),
            error: Color::Rgb(200, 64, 83),
            folder: Color::Rgb(233, 138, 0),
            gauge: Color::Rgb(77, 105, 155),
            border: Color::Rgb(160, 156, 172),
            hover: Color::Rgb(231, 219, 160),
            badge_dl: Color::Rgb(222, 152, 0),
            badge_stream: Color::Rgb(118, 107, 144),
        },
    };

    const NORD_DARK: Theme = Theme {
        name: "Nord Dark",
        palette: Palette {
            bg: Color::Rgb(36, 42, 54),
            fg: Color::Rgb(216, 222, 233),
            surface: Color::Rgb(46, 52, 64),
            accent: Color::Rgb(136, 192, 208),
            text_primary: Color::Rgb(216, 222, 233),
            text_dimmed: Color::Rgb(76, 86, 106),
            input_cursor: Color::Rgb(143, 188, 187),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            folder: Color::Rgb(208, 135, 112),
            gauge: Color::Rgb(136, 192, 208),
            border: Color::Rgb(59, 66, 82),
            hover: Color::Rgb(46, 52, 64),
            badge_dl: Color::Rgb(235, 203, 139),
            badge_stream: Color::Rgb(180, 142, 173),
        },
    };

    const NORD_LIGHT: Theme = Theme {
        name: "Nord Light",
        palette: Palette {
            bg: Color::Rgb(236, 239, 244),
            fg: Color::Rgb(46, 52, 64),
            surface: Color::Rgb(216, 222, 233),
            accent: Color::Rgb(94, 129, 172),
            text_primary: Color::Rgb(46, 52, 64),
            text_dimmed: Color::Rgb(129, 161, 193),
            input_cursor: Color::Rgb(129, 161, 193),
            success: Color::Rgb(122, 156, 102),
            warning: Color::Rgb(199, 151, 63),
            error: Color::Rgb(191, 97, 106),
            folder: Color::Rgb(208, 135, 112),
            gauge: Color::Rgb(136, 192, 208),
            border: Color::Rgb(129, 161, 193),
            hover: Color::Rgb(229, 233, 240),
            badge_dl: Color::Rgb(199, 151, 63),
            badge_stream: Color::Rgb(180, 142, 173),
        },
    };

    const GRUVBOX_DARK: Theme = Theme {
        name: "Gruvbox Dark",
        palette: Palette {
            bg: Color::Rgb(29, 32, 33),
            fg: Color::Rgb(235, 219, 178),
            surface: Color::Rgb(40, 40, 40),
            accent: Color::Rgb(131, 165, 152),
            text_primary: Color::Rgb(235, 219, 178),
            text_dimmed: Color::Rgb(146, 131, 116),
            input_cursor: Color::Rgb(142, 192, 124),
            success: Color::Rgb(184, 187, 38),
            warning: Color::Rgb(250, 189, 47),
            error: Color::Rgb(251, 73, 52),
            folder: Color::Rgb(254, 128, 25),
            gauge: Color::Rgb(131, 165, 152),
            border: Color::Rgb(50, 48, 47),
            hover: Color::Rgb(40, 40, 40),
            badge_dl: Color::Rgb(250, 189, 47),
            badge_stream: Color::Rgb(211, 134, 155),
        },
    };

    const GRUVBOX_LIGHT: Theme = Theme {
        name: "Gruvbox Light",
        palette: Palette {
            bg: Color::Rgb(251, 241, 199),
            fg: Color::Rgb(60, 56, 54),
            surface: Color::Rgb(242, 229, 188),
            accent: Color::Rgb(7, 102, 120),
            text_primary: Color::Rgb(60, 56, 54),
            text_dimmed: Color::Rgb(124, 111, 100),
            input_cursor: Color::Rgb(66, 123, 88),
            success: Color::Rgb(121, 116, 14),
            warning: Color::Rgb(181, 118, 20),
            error: Color::Rgb(157, 0, 6),
            folder: Color::Rgb(175, 58, 3),
            gauge: Color::Rgb(7, 102, 120),
            border: Color::Rgb(213, 196, 161),
            hover: Color::Rgb(235, 219, 178),
            badge_dl: Color::Rgb(181, 118, 20),
            badge_stream: Color::Rgb(143, 63, 113),
        },
    };

    const CATPPUCCIN_MOCHA: Theme = Theme {
        name: "Catppuccin Mocha",
        palette: Palette {
            bg: Color::Rgb(17, 17, 27),
            fg: Color::Rgb(205, 214, 244),
            surface: Color::Rgb(24, 24, 37),
            accent: Color::Rgb(137, 180, 250),
            text_primary: Color::Rgb(205, 214, 244),
            text_dimmed: Color::Rgb(108, 112, 134),
            input_cursor: Color::Rgb(116, 199, 236),
            success: Color::Rgb(166, 227, 161),
            warning: Color::Rgb(249, 226, 175),
            error: Color::Rgb(243, 139, 168),
            folder: Color::Rgb(250, 179, 135),
            gauge: Color::Rgb(137, 180, 250),
            border: Color::Rgb(30, 30, 46),
            hover: Color::Rgb(24, 24, 37),
            badge_dl: Color::Rgb(249, 226, 175),
            badge_stream: Color::Rgb(203, 166, 247),
        },
    };

    const CATPPUCCIN_LATTE: Theme = Theme {
        name: "Catppuccin Latte",
        palette: Palette {
            bg: Color::Rgb(239, 241, 245),
            fg: Color::Rgb(76, 79, 105),
            surface: Color::Rgb(204, 208, 218),
            accent: Color::Rgb(30, 102, 245),
            text_primary: Color::Rgb(76, 79, 105),
            text_dimmed: Color::Rgb(156, 160, 176),
            input_cursor: Color::Rgb(32, 159, 181),
            success: Color::Rgb(64, 160, 43),
            warning: Color::Rgb(223, 142, 29),
            error: Color::Rgb(210, 15, 57),
            folder: Color::Rgb(254, 100, 11),
            gauge: Color::Rgb(30, 102, 245),
            border: Color::Rgb(188, 192, 204),
            hover: Color::Rgb(230, 233, 239),
            badge_dl: Color::Rgb(223, 142, 29),
            badge_stream: Color::Rgb(136, 57, 239),
        },
    };

    const PURPLE: Theme = Theme {
        name: "Purple",
        palette: Palette {
            bg: Color::Rgb(20, 19, 31),
            fg: Color::Rgb(224, 222, 244),
            surface: Color::Rgb(26, 24, 38),
            accent: Color::Rgb(196, 167, 231),
            text_primary: Color::Rgb(224, 222, 244),
            text_dimmed: Color::Rgb(110, 106, 134),
            input_cursor: Color::Rgb(212, 184, 240),
            success: Color::Rgb(49, 182, 176),
            warning: Color::Rgb(246, 193, 119),
            error: Color::Rgb(235, 111, 150),
            folder: Color::Rgb(240, 192, 128),
            gauge: Color::Rgb(196, 167, 231),
            border: Color::Rgb(31, 29, 46),
            hover: Color::Rgb(26, 24, 38),
            badge_dl: Color::Rgb(246, 193, 119),
            badge_stream: Color::Rgb(235, 111, 146),
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
    pub fn surface_style(&self) -> Style {
        Style::default().fg(self.palette.fg).bg(self.palette.surface)
    }

    #[must_use]
    pub fn input_cursor_style(&self) -> Style {
        Style::default().fg(self.palette.input_cursor).bg(self.palette.bg)
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
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.palette.error).bg(self.palette.bg)
    }

    #[must_use]
    pub fn folder_style(&self) -> Style {
        Style::default().fg(self.palette.folder).bg(self.palette.bg)
    }

    #[must_use]
    pub fn gauge_style(&self) -> Style {
        Style::default().fg(self.palette.gauge)
    }

    #[must_use]
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.palette.border)
    }

    #[must_use]
    pub fn hover_style(&self) -> Style {
        Style::default().bg(self.palette.hover)
    }

    #[must_use]
    pub fn badge_dl_style(&self) -> Style {
        Style::default().fg(self.palette.badge_dl).bg(self.palette.bg)
    }

    #[must_use]
    pub fn badge_stream_style(&self) -> Style {
        Style::default().fg(self.palette.badge_stream).bg(self.palette.bg)
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
