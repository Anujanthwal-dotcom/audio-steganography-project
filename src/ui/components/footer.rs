use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, AppMode};

pub fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let spans = match app.mode {
        AppMode::Embed => vec![
            Span::styled(" [Tab] ", theme.accent_style()),
            Span::styled("Switch Focus  ", theme.text_style()),
            Span::styled("│ [Enter] ", theme.accent_style()),
            Span::styled("Navigate / Select  ", theme.text_style()),
            Span::styled("│ [Ctrl+S] / [E] ", theme.accent_style()),
            Span::styled("Embed & Save  ", Style::default().fg(theme.success_mint).add_modifier(Modifier::BOLD)),
            Span::styled("│ [2] ", theme.accent_style()),
            Span::styled("Extract  ", theme.text_style()),
            Span::styled("│ [3] ", theme.accent_style()),
            Span::styled("Inspector  ", theme.text_style()),
            Span::styled("│ [Q/Esc] ", theme.danger_style()),
            Span::styled("Quit", theme.text_style()),
        ],
        AppMode::Extract => vec![
            Span::styled(" [Tab] ", theme.accent_style()),
            Span::styled("Switch Focus  ", theme.text_style()),
            Span::styled("│ [Enter] ", theme.accent_style()),
            Span::styled("Select / Unlock  ", Style::default().fg(theme.success_mint).add_modifier(Modifier::BOLD)),
            Span::styled("│ [C] ", theme.accent_style()),
            Span::styled("Copy  ", theme.text_style()),
            Span::styled("│ [S] ", theme.accent_style()),
            Span::styled("Save to File  ", theme.text_style()),
            Span::styled("│ [1] ", theme.accent_style()),
            Span::styled("Embed  ", theme.text_style()),
            Span::styled("│ [3] ", theme.accent_style()),
            Span::styled("Inspector  ", theme.text_style()),
            Span::styled("│ [Q/Esc] ", theme.danger_style()),
            Span::styled("Quit", theme.text_style()),
        ],
        AppMode::Inspector => vec![
            Span::styled(" [Tab] ", theme.accent_style()),
            Span::styled("Switch Focus  ", theme.text_style()),
            Span::styled("│ [Enter] ", theme.accent_style()),
            Span::styled("Select & Inspect  ", Style::default().fg(theme.success_mint).add_modifier(Modifier::BOLD)),
            Span::styled("│ [1] ", theme.accent_style()),
            Span::styled("Embed Mode  ", theme.text_style()),
            Span::styled("│ [2] ", theme.accent_style()),
            Span::styled("Extract Mode  ", theme.text_style()),
            Span::styled("│ [Q/Esc] ", theme.danger_style()),
            Span::styled("Quit", theme.text_style()),
        ],
    };

    let p = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.surface));
    frame.render_widget(p, area);
}
