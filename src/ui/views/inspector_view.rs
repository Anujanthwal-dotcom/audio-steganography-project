use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

use crate::app::{App, InspectorFocus};
use crate::stego::capacity::format_bytes;
use crate::ui::components::file_picker::render_file_picker;

pub fn render_inspector_view(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(42), // Left: File browser
            Constraint::Percentage(58), // Right: Diagnostics & Stego Analysis
        ])
        .split(area);

    let is_browser_focused = app.inspector_focus == InspectorFocus::FileBrowser;
    render_file_picker(
        frame,
        columns[0],
        &app.file_picker,
        theme,
        is_browser_focused,
        " Select Audio to Inspect ",
    );

    // Diagnostics Right Column
    let is_diag_focused = app.inspector_focus == InspectorFocus::Details;
    let diag_block = theme.block(" Audio & Steganography Diagnostics ", is_diag_focused);

    let mut lines = Vec::new();

    if let Some(wav) = &app.inspected_wav {
        let meta = &wav.metadata;

        lines.push(Line::from(vec![
            Span::styled("--- WAV Carrier Specification ---", theme.accent_style()),
        ]));

        if let Some(path) = &meta.file_path {
            lines.push(Line::from(vec![
                Span::styled("File:         ", theme.muted_style()),
                Span::styled(path.display().to_string(), theme.text_style()),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("File Size:    ", theme.muted_style()),
            Span::styled(meta.formatted_size(), theme.text_style()),
            Span::styled("  |  Sample Rate: ", theme.muted_style()),
            Span::styled(format!("{} Hz", meta.sample_rate), theme.text_style()),
        ]));

        let channels_str = match meta.channels {
            1 => "Mono (1 channel)".to_string(),
            2 => "Stereo (2 channels)".to_string(),
            other => format!("{other} channels"),
        };

        lines.push(Line::from(vec![
            Span::styled("Channels:     ", theme.muted_style()),
            Span::styled(channels_str, theme.text_style()),
            Span::styled("  |  Bit Depth:   ", theme.muted_style()),
            Span::styled(format!("{}-bit PCM", meta.bits_per_sample), theme.text_style()),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Total Samples:", theme.muted_style()),
            Span::styled(format!(" {}", crate::audio::format_number(meta.total_samples)), theme.text_style()),
            Span::styled("  |  Duration:    ", theme.muted_style()),
            Span::styled(meta.duration_formatted(), theme.text_style()),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Max Capacity: ", theme.muted_style()),
            Span::styled(format_bytes(meta.max_capacity_bytes()), theme.accent_style()),
            Span::styled(" (1 bit per sample LSB)", theme.muted_style()),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("--- Steganographic Analysis ---", theme.accent_style()),
        ]));

        match &app.inspected_header {
            Some(Ok(header)) => {
                lines.push(Line::from(vec![
                    Span::styled("Signature:    ", theme.muted_style()),
                    Span::styled(
                        "VALID \"STEG\" SIGNATURE DETECTED",
                        Style::default().fg(theme.success_mint).add_modifier(Modifier::BOLD),
                    ),
                ]));

                lines.push(Line::from(vec![
                    Span::styled("Version:      ", theme.muted_style()),
                    Span::styled(format!("0x{:02X} (Sonosteg v{})", header.version, header.version), theme.text_style()),
                ]));

                let enc_str = if header.is_encrypted() {
                    "YES - ChaCha20-Poly1305 (Argon2id KDF)"
                } else {
                    "NO - Plaintext"
                };

                let comp_str = if header.is_compressed() {
                    "YES - Deflate / Zstandard"
                } else {
                    "NO - Raw"
                };

                lines.push(Line::from(vec![
                    Span::styled("Encrypted:    ", theme.muted_style()),
                    Span::styled(
                        enc_str,
                        if header.is_encrypted() {
                            theme.accent_style()
                        } else {
                            theme.text_style()
                        },
                    ),
                ]));

                lines.push(Line::from(vec![
                    Span::styled("Compressed:   ", theme.muted_style()),
                    Span::styled(comp_str, theme.text_style()),
                ]));

                if header.is_encrypted() {
                    let salt_hex: String = header.salt.iter().map(|b| format!("{b:02x}")).collect();
                    let nonce_hex: String = header.nonce.iter().map(|b| format!("{b:02x}")).collect();

                    lines.push(Line::from(vec![
                        Span::styled("Argon2 Salt:  ", theme.muted_style()),
                        Span::styled(salt_hex, theme.muted_style()),
                    ]));

                    lines.push(Line::from(vec![
                        Span::styled("ChaCha Nonce: ", theme.muted_style()),
                        Span::styled(nonce_hex, theme.muted_style()),
                    ]));
                }

                lines.push(Line::from(vec![
                    Span::styled("Payload Size: ", theme.muted_style()),
                    Span::styled(format!("{} bytes ({:.2} KB)", header.payload_len, header.payload_len as f64 / 1024.0), theme.accent_style()),
                ]));
            }
            Some(Err(err)) => {
                lines.push(Line::from(vec![
                    Span::styled("Signature:    ", theme.muted_style()),
                    Span::styled("NO STEGANOGRAPHIC PAYLOAD DETECTED", theme.muted_style()),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Scan Status:  ", theme.muted_style()),
                    Span::styled(err.as_str(), theme.muted_style()),
                ]));
            }
            None => {
                lines.push(Line::from(vec![
                    Span::styled("Signature:    ", theme.muted_style()),
                    Span::styled("Scanning carrier samples...", theme.muted_style()),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("--- Quick Actions ---", theme.accent_style()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Press [1] to open as Carrier in Embed Mode", theme.text_style()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Press [2] to open in Extract Mode", theme.text_style()),
        ]));
    } else {
        lines.push(Line::from(vec![Span::styled(
            "Select an audio file from the browser on the left to view comprehensive diagnostics and scan for steganographic payloads.",
            theme.muted_style(),
        )]));
    }

    let p = Paragraph::new(lines)
        .block(diag_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(p, columns[1]);
}
