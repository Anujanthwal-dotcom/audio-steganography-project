use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};

use crate::audio::AudioMetadata;
use crate::stego::CapacityStats;
use crate::ui::theme::Theme;

pub fn render_audio_info(
    frame: &mut Frame,
    area: Rect,
    metadata: Option<&AudioMetadata>,
    capacity: Option<&CapacityStats>,
    theme: &Theme,
    title: &str,
) {
    let block = Block::default()
        .title(title)
        .title_style(theme.accent_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.text_muted))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let meta = match metadata {
        Some(m) => m,
        None => {
            let p = Paragraph::new(" No audio carrier selected. Please select a .wav file above.")
                .style(theme.muted_style());
            frame.render_widget(p, inner);
            return;
        }
    };

    let channels_str = match meta.channels {
        1 => "1 (Mono)",
        2 => "2 (Stereo)",
        other => &format!("{other} Channels"),
    };

    let line1 = Line::from(vec![
        Span::styled("Total Audio Samples: ", theme.muted_style()),
        Span::styled(crate::audio::format_number(meta.total_samples), theme.text_style()),
        Span::styled("  |  Duration: ", theme.muted_style()),
        Span::styled(meta.duration_formatted(), theme.text_style()),
        Span::styled("  |  Channels: ", theme.muted_style()),
        Span::styled(channels_str, theme.text_style()),
        Span::styled("  |  Rate: ", theme.muted_style()),
        Span::styled(format!("{} Hz", meta.sample_rate), theme.text_style()),
        Span::styled("  |  Depth: ", theme.muted_style()),
        Span::styled(format!("{}-bit", meta.bits_per_sample), theme.text_style()),
    ]);

    let cap_stats = capacity.copied().unwrap_or_else(|| {
        CapacityStats::calculate(meta.total_samples, 0, crate::stego::TOTAL_OVERHEAD)
    });

    let line2 = Line::from(vec![
        Span::styled("Max Embedding Capacity: ", theme.muted_style()),
        Span::styled(cap_stats.formatted_max_capacity(), theme.accent_style()),
    ]);

    let line3 = Line::from(vec![
        Span::styled("Message Payload Size:   ", theme.muted_style()),
        Span::styled(
            format!(
                "{} + {} overhead ({:.2}% of total capacity)",
                cap_stats.formatted_payload_size(),
                cap_stats.overhead_bytes,
                cap_stats.usage_percentage
            ),
            theme.text_style(),
        ),
    ]);

    // Capacity Gauge
    let gauge_ratio = (cap_stats.usage_percentage / 100.0).clamp(0.0, 1.0);
    let gauge_color = if !cap_stats.fits() {
        theme.danger_crimson
    } else if cap_stats.usage_percentage > 80.0 {
        theme.warning_amber
    } else {
        theme.success_mint
    };

    let gauge_label = format!(
        "{:.2}%  [{}]",
        cap_stats.usage_percentage,
        cap_stats.status_text()
    );

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(1), // line 1
            ratatui::layout::Constraint::Length(1), // line 2
            ratatui::layout::Constraint::Length(1), // line 3
            ratatui::layout::Constraint::Length(1), // gauge
        ])
        .split(inner);

    frame.render_widget(Paragraph::new(line1), chunks[0]);
    frame.render_widget(Paragraph::new(line2), chunks[1]);
    frame.render_widget(Paragraph::new(line3), chunks[2]);

    if chunks.len() > 3 {
        let gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(gauge_color)
                    .bg(theme.surface)
                    .add_modifier(Modifier::BOLD),
            )
            .ratio(gauge_ratio)
            .label(gauge_label);
        frame.render_widget(gauge, chunks[3]);
    }
}
