use ratatui::{
    layout::Rect,
    widgets::Paragraph,
    Frame,
};

use crate::ui::theme::Theme;

#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    pub content: String,
    pub cursor_position: usize,
    pub is_masked: bool,
    pub multiline: bool,
}

impl TextInputState {
    pub fn new(is_masked: bool, multiline: bool) -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            is_masked,
            multiline,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_position >= self.content.len() {
            self.content.push(c);
            self.cursor_position = self.content.len();
        } else {
            self.content.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 && !self.content.is_empty() {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_position < self.content.len() {
            self.content.remove(self.cursor_position);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_to_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_to_end(&mut self) {
        self.cursor_position = self.content.len();
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
    }

    pub fn set_text(&mut self, text: &str) {
        self.content = text.to_string();
        self.cursor_position = self.content.len();
    }
}

pub fn render_text_input(
    frame: &mut Frame,
    area: Rect,
    input: &TextInputState,
    theme: &Theme,
    is_focused: bool,
    title: &str,
    placeholder: &str,
) {
    let block = theme.block(title, is_focused);

    let display_content = if input.is_masked {
        "*".repeat(input.content.len())
    } else {
        input.content.clone()
    };

    let (text, text_style) = if display_content.is_empty() && !is_focused {
        (placeholder.to_string(), theme.muted_style())
    } else {
        (display_content, theme.text_style())
    };

    let p = Paragraph::new(text)
        .block(block)
        .style(text_style);

    frame.render_widget(p, area);

    // If focused, render the visual terminal cursor
    if is_focused {
        let inner = area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 1,
        });

        if !input.multiline {
            let cursor_x = inner.x + (input.cursor_position as u16).min(inner.width.saturating_sub(1));
            let cursor_y = inner.y;
            frame.set_cursor_position((cursor_x, cursor_y));
        } else {
            // Compute multiline cursor coordinates
            let mut line = 0;
            let mut col = 0;
            for (i, c) in input.content.chars().enumerate() {
                if i >= input.cursor_position {
                    break;
                }
                if c == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }
            let cursor_x = inner.x + (col as u16).min(inner.width.saturating_sub(1));
            let cursor_y = inner.y + (line as u16).min(inner.height.saturating_sub(1));
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
