use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, AppMode};

pub fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let mode_str = match app.mode {
        AppMode::Embed => "EMBED",
        AppMode::Extract => "EXTRACT",
        AppMode::Inspector => "INSPECTOR",
    };

    let title_line = Line::from(vec![
        Span::styled(
            " SONOSTEG ",
            Style::default()
                .fg(theme.cyan_accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v0.1.0  ", theme.muted_style()),
        Span::styled("─── ", theme.muted_style()),
        Span::styled("Secure Audio Steganography Rust TUI ", theme.text_style()),
        Span::styled("─── [Mode: ", theme.muted_style()),
        Span::styled(
            mode_str,
            Style::default()
                .fg(theme.violet_accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("] ", theme.muted_style()),
    ]);

    let tab_titles = vec![
        " [1] Embed (Hide) ",
        " [2] Extract (Reveal) ",
        " [3] Audio Inspector ",
    ];

    let selected_tab = match app.mode {
        AppMode::Embed => 0,
        AppMode::Extract => 1,
        AppMode::Inspector => 2,
    };

    let tabs = Tabs::new(tab_titles)
        .select(selected_tab)
        .style(theme.muted_style())
        .highlight_style(
            Style::default()
                .fg(theme.cyan_accent)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Length(1), // Tabs line
            Constraint::Length(1), // Divider border
        ])
        .split(area);

    frame.render_widget(Paragraph::new(title_line), chunks[0]);
    frame.render_widget(tabs, chunks[1]);

    let divider = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.text_muted));
    frame.render_widget(divider, chunks[2]);
}
