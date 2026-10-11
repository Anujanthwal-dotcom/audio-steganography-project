use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub enum DialogType {
    Success { title: String, message: String },
    Error { title: String, message: String },
    Warning { title: String, message: String },
    Info { title: String, message: String },
}

#[derive(Debug, Clone, Default)]
pub struct DialogState {
    pub current: Option<DialogType>,
}

impl DialogState {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn show_success(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.current = Some(DialogType::Success {
            title: title.into(),
            message: message.into(),
        });
    }

    pub fn show_error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.current = Some(DialogType::Error {
            title: title.into(),
            message: message.into(),
        });
    }

    pub fn show_warning(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.current = Some(DialogType::Warning {
            title: title.into(),
            message: message.into(),
        });
    }

    pub fn show_info(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.current = Some(DialogType::Info {
            title: title.into(),
            message: message.into(),
        });
    }

    pub fn dismiss(&mut self) -> bool {
        if self.current.is_some() {
            self.current = None;
            true
        } else {
            false
        }
    }

    pub fn is_open(&self) -> bool {
        self.current.is_some()
    }
}

pub fn render_dialog(frame: &mut Frame, dialog_state: &DialogState, theme: &Theme) {
    let dialog = match &dialog_state.current {
        Some(d) => d,
        None => return,
    };

    let (title, message, border_color) = match dialog {
        DialogType::Success { title, message } => (title, message, theme.success_mint),
        DialogType::Error { title, message } => (title, message, theme.danger_crimson),
        DialogType::Warning { title, message } => (title, message, theme.warning_amber),
        DialogType::Info { title, message } => (title, message, theme.cyan_accent),
    };

    let area = frame.area();
    // Center a 60x10 popup
    let popup_width = 64.min(area.width.saturating_sub(4));
    let popup_height = 12.min(area.height.saturating_sub(4));

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(popup_height)) / 2),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(popup_width)) / 2),
            Constraint::Length(popup_width),
            Constraint::Min(0),
        ])
        .split(vertical_layout[1]);

    let popup_area = horizontal_layout[1];

    // Clear background behind modal
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.surface));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),
            Constraint::Length(1),
        ])
        .split(inner);

    let message_p = Paragraph::new(message.as_str())
        .style(Style::default().fg(theme.text_normal))
        .wrap(Wrap { trim: true });
    frame.render_widget(message_p, chunks[0]);

    let hint_p = Paragraph::new(" [ Press Esc or Enter to dismiss ] ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC));
    frame.render_widget(hint_p, chunks[1]);
}
