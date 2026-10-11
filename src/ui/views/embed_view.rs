use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, EmbedFocus};
use crate::stego::CapacityStats;
use crate::ui::components::{
    audio_info::render_audio_info,
    file_picker::render_file_picker,
    text_input::render_text_input,
};

pub fn render_embed_view(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(12),   // Top: File browser & message editor
            Constraint::Length(7), // Bottom: Audio info & capacity gauge
        ])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(42), // Left: File browser
            Constraint::Percentage(58), // Right: Editor & Controls
        ])
        .split(main_chunks[0]);

    // 1. Audio File Browser
    let is_browser_focused = app.embed_focus == EmbedFocus::FileBrowser;
    let selected_name = app
        .loaded_cover_wav
        .as_ref()
        .and_then(|w| w.metadata.file_path.as_ref())
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "None".to_string());

    let browser_title = format!(" Audio File Browser (Selected: {selected_name}) ");
    render_file_picker(
        frame,
        top_chunks[0],
        &app.file_picker,
        theme,
        is_browser_focused,
        &browser_title,
    );

    // 2. Right Pane: Editor & Options & Button
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Secret message editor
            Constraint::Length(3), // Passphrase input
            Constraint::Length(3), // Compression toggle & Embed button
        ])
        .split(top_chunks[1]);

    // Message Editor
    let is_msg_focused = app.embed_focus == EmbedFocus::MessageInput;
    render_text_input(
        frame,
        right_chunks[0],
        &app.message_input,
        theme,
        is_msg_focused,
        " Secret Message Editor ",
        "Enter secret message to conceal within carrier audio...",
    );

    // Passphrase Input
    let is_pass_focused = app.embed_focus == EmbedFocus::PassphraseInput;
    render_text_input(
        frame,
        right_chunks[1],
        &app.passphrase_input,
        theme,
        is_pass_focused,
        " Passphrase (Optional Authenticated Encryption) ",
        "Leave blank for unencrypted steganography...",
    );

    // Controls line: Compression Toggle & Embed Action Button
    let controls_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Compression toggle
            Constraint::Percentage(50), // Embed button
        ])
        .split(right_chunks[2]);

    // Compression Toggle Box
    let is_compress_focused = app.embed_focus == EmbedFocus::CompressToggle;
    let comp_check = if app.compress_enabled { "[X]" } else { "[ ]" };
    let comp_style = if is_compress_focused {
        theme.accent_style()
    } else {
        theme.text_style()
    };
    let comp_block = theme.block(" Compression ", is_compress_focused);
    let comp_p = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {comp_check} Deflate Compression "), comp_style),
        Span::styled("(Space)", theme.muted_style()),
    ]))
    .block(comp_block);
    frame.render_widget(comp_p, controls_chunks[0]);

    // Embed Action Button Box
    let is_btn_focused = app.embed_focus == EmbedFocus::EmbedButton;
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
    let btn_block = theme.block(" Action ", is_btn_focused);
    let btn_p = Paragraph::new(" ▶ EMBED & SAVE STEGO AUDIO (Enter) ")
        .alignment(ratatui::layout::Alignment::Center)
        .style(btn_style)
        .block(btn_block);
    frame.render_widget(btn_p, controls_chunks[1]);

    // 3. Bottom: Audio info & capacity
    let meta = app.loaded_cover_wav.as_ref().map(|w| &w.metadata);
    let total_samples = meta.map(|m| m.total_samples).unwrap_or(0);
    let payload_bytes = app.message_input.content.len();
    let cap_stats = CapacityStats::calculate(
        total_samples,
        payload_bytes,
        crate::stego::TOTAL_OVERHEAD,
    );

    render_audio_info(
        frame,
        main_chunks[1],
        meta,
        Some(&cap_stats),
        theme,
        " Audio Capacity & Embedding Stats ",
    );
}
