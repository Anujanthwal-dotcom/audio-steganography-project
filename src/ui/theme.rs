use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub cyan_accent: Color,
    pub violet_accent: Color,
    pub success_mint: Color,
    pub warning_amber: Color,
    pub danger_crimson: Color,
    pub text_normal: Color,
    pub text_muted: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(18, 20, 26),
            surface: Color::Rgb(30, 34, 45),
            cyan_accent: Color::Rgb(0, 229, 255),
            violet_accent: Color::Rgb(189, 147, 249),
            success_mint: Color::Rgb(80, 250, 123),
            warning_amber: Color::Rgb(255, 184, 108),
            danger_crimson: Color::Rgb(255, 85, 85),
            text_normal: Color::Rgb(248, 248, 242),
            text_muted: Color::Rgb(98, 114, 164),
        }
    }
}

impl Theme {
    pub fn block<'a>(&self, title: &'a str, focused: bool) -> Block<'a> {
        let border_color = if focused {
            self.cyan_accent
        } else {
            self.text_muted
        };

        let title_style = if focused {
            Style::default()
                .fg(self.cyan_accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.text_normal)
        };

        Block::default()
            .borders(Borders::ALL)
            .border_type(if focused {
                BorderType::Thick
            } else {
                BorderType::Rounded
            })
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(self.bg).fg(self.text_normal))
            .title(title)
            .title_style(title_style)
    }

    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text_normal)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.cyan_accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success_mint)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning_amber)
    }

    pub fn danger_style(&self) -> Style {
        Style::default().fg(self.danger_crimson)
    }
}
