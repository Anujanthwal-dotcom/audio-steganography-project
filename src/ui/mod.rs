pub mod components;
pub mod theme;
pub mod views;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppMode};
use components::dialogs::render_dialog;

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Responsive minimum size check (Section 6.4: minimum 80x24)
    if size.width < 80 || size.height < 24 {
        let warning_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(app.theme.warning_style())
            .title(" Terminal Dimension Warning ")
            .title_alignment(Alignment::Center)
            .style(app.theme.text_style());

        let warning_text = format!(
            "\nTerminal window too small (minimum: 80x24)\nCurrent dimensions: {}x{}\n\nPlease resize your terminal window to continue.",
            size.width, size.height
        );

        let p = Paragraph::new(warning_text)
            .alignment(Alignment::Center)
            .block(warning_block);

        frame.render_widget(p, size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & Tabs
            Constraint::Min(0),    // Main Active View
            Constraint::Length(1), // Footer Shortcuts
        ])
        .split(size);

    components::header::render_header(frame, chunks[0], app);

    match app.mode {
        AppMode::Embed => views::embed_view::render_embed_view(frame, chunks[1], app),
        AppMode::Extract => views::extract_view::render_extract_view(frame, chunks[1], app),
        AppMode::Inspector => views::inspector_view::render_inspector_view(frame, chunks[1], app),
    }

    components::footer::render_footer(frame, chunks[2], app);

    // Modal Dialog overlay if active
    if app.dialog.is_open() {
        render_dialog(frame, &app.dialog, &app.theme);
    }
}
