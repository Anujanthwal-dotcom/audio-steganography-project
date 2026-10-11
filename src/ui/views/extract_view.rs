use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, ExtractFocus};
use crate::ui::components::{file_picker::render_file_picker, text_input::render_text_input};

pub fn render_extract_view(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(46), // Left: Selection & Unlock
            Constraint::Percentage(54), // Right: Extracted secret output
        ])
        .split(area);

    // Left Column Chunks
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(48), // File picker
            Constraint::Length(6),      // Stego detection details
            Constraint::Length(3),      // Passphrase input
            Constraint::Length(3),      // Unlock & Extract button
        ])
        .split(columns[0]);

    // 1. File Picker
    let is_browser_focused = app.extract_focus == ExtractFocus::FileBrowser;
    render_file_picker(
        frame,
        left_chunks[0],
        &app.file_picker,
        theme,
        is_browser_focused,
        " Select Stego Audio File ",
    );

    // 2. Stego Detection Details
    let details_block = theme.block(" Stego Signature Detection ", false);
    let mut details_lines = Vec::new();

    if let Some(wav) = &app.loaded_stego_wav {
        details_lines.push(Line::from(vec![
            Span::styled("File Size: ", theme.muted_style()),
            Span::styled(wav.metadata.formatted_size(), theme.text_style()),
            Span::styled("  |  Samples: ", theme.muted_style()),
            Span::styled(crate::audio::format_number(wav.metadata.total_samples), theme.text_style()),
        ]));

        if let Some(header) = &app.detected_header {
            details_lines.push(Line::from(vec![
                Span::styled("Detected Stego Magic: ", theme.muted_style()),
                Span::styled(
                    "\"STEG\" (v1) [VALID]",
                    Style::default().fg(theme.success_mint).add_modifier(Modifier::BOLD),
                ),
            ]));

            let enc_str = if header.is_encrypted() {
                "ChaCha20-Poly1305 (Argon2id KDF)"
            } else {
                "None (Plaintext)"
            };

            let comp_str = if header.is_compressed() {
                "Deflate / Zstandard"
            } else {
                "Raw (Uncompressed)"
            };

            details_lines.push(Line::from(vec![
                Span::styled("Encryption: ", theme.muted_style()),
                Span::styled(
                    enc_str,
                    if header.is_encrypted() {
                        theme.accent_style()
                    } else {
                        theme.text_style()
                    },
                ),
            ]));

            details_lines.push(Line::from(vec![
                Span::styled("Compression: ", theme.muted_style()),
                Span::styled(comp_str, theme.text_style()),
                Span::styled("  |  Payload: ", theme.muted_style()),
                Span::styled(format!("{} bytes", header.payload_len), theme.accent_style()),
            ]));
        } else {
            details_lines.push(Line::from(vec![
                Span::styled("Stego Signature: ", theme.muted_style()),
                Span::styled(
                    "No \"STEG\" signature found in this audio carrier",
                    theme.danger_style(),
                ),
            ]));
        }
    } else {
        details_lines.push(Line::from(vec![Span::styled(
            "Select a .wav file above to scan for steganographic payloads.",
            theme.muted_style(),
        )]));
    }

    let details_p = Paragraph::new(details_lines).block(details_block);
    frame.render_widget(details_p, left_chunks[1]);

    // 3. Passphrase Input
    let is_pass_focused = app.extract_focus == ExtractFocus::PassphraseInput;
    render_text_input(
        frame,
        left_chunks[2],
        &app.extract_passphrase_input,
        theme,
        is_pass_focused,
        " Passphrase ",
        "Enter passphrase to decrypt (if encrypted)...",
    );

    // 4. Unlock & Extract Action Button
    let is_btn_focused = app.extract_focus == ExtractFocus::UnlockButton;
    let btn_style = if is_btn_focused {
        Style::default()
            .bg(theme.cyan_accent)
            .fg(theme.bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.success_mint)
            .add_modifier(Modifier::BOLD)
    };
    let btn_block = theme.block(" Extract ", is_btn_focused);
    let btn_p = Paragraph::new(" 🔓 UNLOCK & EXTRACT PAYLOAD (Enter) ")
        .alignment(ratatui::layout::Alignment::Center)
        .style(btn_style)
        .block(btn_block);
    frame.render_widget(btn_p, left_chunks[3]);

    // Right Column: Extracted Secret Message & Actions
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Extracted message view
            Constraint::Length(3), // Actions box ([C] Copy, [S] Save)
        ])
        .split(columns[1]);

    // Extracted Message View
    let is_msg_focused = app.extract_focus == ExtractFocus::ExtractedView;
    let msg_block = theme.block(" Extracted Secret Message ", is_msg_focused);

    let (display_text, text_style) = match (&app.extracted_message, &app.extracted_status) {
        (Some(msg), Some(status)) => (
            format!("{status}\n\n{msg}"),
            Style::default().fg(theme.text_normal),
        ),
        (Some(msg), None) => (msg.clone(), Style::default().fg(theme.text_normal)),
        (None, Some(err)) => (err.clone(), theme.danger_style()),
        (None, None) => (
            "No message extracted yet. Select a stego audio carrier and press Unlock.".to_string(),
            theme.muted_style(),
        ),
    };

    let msg_p = Paragraph::new(display_text)
        .block(msg_block)
        .style(text_style)
        .wrap(Wrap { trim: false });
    frame.render_widget(msg_p, right_chunks[0]);

    // Actions Box
    let actions_block = Block::default()
        .title(" Actions ")
        .title_style(theme.accent_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.text_muted))
        .style(Style::default().bg(theme.bg));

    let action_spans = vec![
        Span::styled(" [C] ", theme.accent_style()),
        Span::styled("Copy to Clipboard    ", theme.text_style()),
        Span::styled("│ [S] ", theme.accent_style()),
        Span::styled("Save to .txt File", theme.text_style()),
    ];
    let actions_p = Paragraph::new(Line::from(action_spans))
        .alignment(ratatui::layout::Alignment::Center)
        .block(actions_block);
    frame.render_widget(actions_p, right_chunks[1]);
}
