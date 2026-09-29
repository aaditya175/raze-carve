use super::format_bytes;
use crate::{
    app::{entropy_label, App},
    theme::Theme,
};
/// Forensic Entropy & LBA Spectrum Visualizer
/// Renders an innovative high-density spectrogram waveform, multi-tier
/// LBA distribution, Shannon information density telemetry, and NIST SP 800-88 compliance.
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(area);

    render_drive_list(frame, main[0], app);
    render_entropy_panel(frame, main[1], app);
}

fn render_drive_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .drives
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let selected = app.entropy_drive_cursor == i;
            let is_scanning = app.entropy_scanning == Some(i);

            let (avg_e, avg_str) = if is_scanning {
                (0.0, format!("{} scan", Theme::spinner(app.tick)))
            } else if d.entropy_loaded && !d.entropy_samples.is_empty() {
                let avg = d.entropy_stats.avg;
                (avg, format!("{:.2}b", avg))
            } else {
                (0.0, "  ─   ".to_string())
            };

            let ind = if selected { "▶" } else { " " };
            let name_color = if selected { Theme::CYAN } else { Theme::TEXT };
            let bg = if selected { Theme::SURFACE2 } else { Theme::BG };
            let ent_color = if is_scanning {
                Theme::WARNING
            } else if d.entropy_loaded {
                Theme::entropy_color(avg_e)
            } else {
                Theme::MUTED
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", ind), Style::default().fg(Theme::CYAN)),
                    Span::styled(
                        format!("/dev/{:<8}", d.name),
                        Style::default().fg(name_color).add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(
                        format!("{:<6}", d.drive_type),
                        Style::default().fg(Theme::BLUE),
                    ),
                    Span::styled(format!("  {}", avg_str), Style::default().fg(ent_color)),
                ]),
                Line::raw(""),
            ])
            .style(Style::default().bg(bg))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Drives [↑↓] [E] Analyze ",
            Style::default()
                .fg(Theme::PURPLE)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::BG));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_entropy_panel(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // header / telemetry HUD
            Constraint::Length(3), // legend & standard threshold
            Constraint::Min(8),    // Spectrogram Waveform Matrix
            Constraint::Length(7), // Distribution statistics & compliance
        ])
        .split(area);

    let d = match app.drives.get(app.entropy_drive_cursor) {
        Some(d) => d,
        None => {
            let para = Paragraph::new("  No drive selected.")
                .style(Style::default().fg(Theme::MUTED));
            frame.render_widget(para, area);
            return;
        }
    };

    let (avg_e, avg_label) = if d.entropy_loaded && !d.entropy_samples.is_empty() {
        let avg = d.entropy_stats.avg;
        let (lbl, _) = entropy_label(avg);
        (avg, lbl)
    } else {
        (0.0, "Unscanned")
    };

    // ── Header HUD ────────────────────────────────────────────────────────────
    let is_current_scanning = app.entropy_scanning == Some(app.entropy_drive_cursor);
    let sanitization_badge = if is_current_scanning {
        Span::styled(
            format!("  {} [SCANNING LBA SECTORS IN PROGRESS...]", Theme::spinner(app.tick)),
            Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD),
        )
    } else if !d.entropy_loaded {
        Span::styled("[STATUS: PENDING SCAN - Press E/Enter]", Style::default().fg(Theme::MUTED))
    } else if avg_e < 0.05 {
        Span::styled(
            "[NIST 800-88: PURGED / ZERO-FILLED]",
            Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD),
        )
    } else if avg_e > 7.90 {
        Span::styled(
            "[HIGH ENTROPY: ENCRYPTED / RANDOM WIPED]",
            Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            "[DATA RESIDUAL: UNWIPED RESIDUAL EVIDENCE]",
            Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD),
        )
    };

    let header_lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Target: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("/dev/{} ", d.name),
                Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled("• ", Style::default().fg(Theme::BORDER)),
            Span::styled(format!("{} ", d.model), Style::default().fg(Theme::TEXT)),
            Span::styled(format!("[{}] ", d.drive_type), Style::default().fg(Theme::BLUE)),
            Span::styled(format_bytes(d.size_bytes), Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("   ", Style::default()),
            sanitization_badge,
        ]),
        Line::from(vec![
            Span::styled("  Shannon Entropy: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.4} b/B ", avg_e),
                Style::default()
                    .fg(Theme::entropy_color(avg_e))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({})", avg_label),
                Style::default().fg(Theme::entropy_color(avg_e)),
            ),
            Span::styled("   •   Information Density: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.1}% ", (avg_e / 8.0) * 100.0),
                Style::default().fg(Theme::TEAL),
            ),
            Span::styled(
                format!("  [H(X) = -Σ P(x)·log₂P(x) | {} sectors sampled]", d.entropy_samples.len()),
                Style::default().fg(Theme::MUTED),
            ),
        ]),
    ];

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " ◈ Forensic LBA Entropy Telemetry HUD ",
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));
    frame.render_widget(Paragraph::new(header_lines).block(header_block), rows[0]);

    // ── Legend ────────────────────────────────────────────────────────────────
    render_legend(frame, rows[1]);

    // ── Spectrogram Waveform Matrix ───────────────────────────────────────────
    render_waveform_spectrogram(frame, rows[2], d);

    // ── Distribution Statistics ───────────────────────────────────────────────
    render_stats(frame, rows[3], d, avg_e);
}

fn render_legend(frame: &mut Frame, area: Rect) {
    let bands = [
        (" 0.0-1.0b", Theme::ENT_0, "Purged/Zero"),
        (" 1.0-2.0b", Theme::ENT_1, "Low-Var Sparse"),
        (" 2.0-4.0b", Theme::ENT_2, "Plaintext/Code"),
        (" 4.0-6.0b", Theme::ENT_3, "Binary Executable"),
        (" 6.0-7.5b", Theme::ENT_4, "Packed Archive"),
        (" 7.5-8.0b", Theme::ENT_5, "Encrypted/CSPRNG"),
    ];

    let mut spans = vec![Span::raw("  ")];
    for (val, col, desc) in bands {
        spans.push(Span::styled("■", Style::default().fg(col)));
        spans.push(Span::styled(
            format!("{} ", val),
            Style::default().fg(col).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!("{}   ", desc), Style::default().fg(Theme::TEXT_DIM)));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Entropy Intensity Scale ",
            Style::default().fg(Theme::MUTED),
        ))
        .style(Style::default().bg(Theme::BG));
    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

fn render_waveform_spectrogram(frame: &mut Frame, area: Rect, d: &crate::app::DriveEntry) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " LBA Entropy Waveform Spectrogram [Y: 0.0 → 8.0 b/B  |  X: Physical LBA Offset] ",
            Style::default()
                .fg(Theme::PURPLE)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if !d.entropy_loaded || d.entropy_samples.is_empty() {
        let prompt = Paragraph::new(vec![
            Line::raw(""),
            Line::from(Span::styled(
                "  No entropy data sampled for this drive.",
                Style::default().fg(Theme::MUTED),
            )),
            Line::from(Span::styled(
                "  Press [E] to execute a high-resolution cryptographic entropy sweep.",
                Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD),
            )),
        ]);
        frame.render_widget(prompt, inner);
        return;
    }

    let samples = &d.entropy_samples;
    let w = inner.width as usize;
    let h = inner.height as usize;
    if w < 16 || h < 4 {
        return;
    }

    // Reserve 7 columns on the left for vertical Y-axis labels ("8.0 ┤ ", etc.)
    let y_axis_w = 7;
    let plot_w = w.saturating_sub(y_axis_w + 1);
    let plot_h = h.saturating_sub(2).max(2);

    let step = (samples.len() as f64 / plot_w as f64).max(1.0);
    let mut lines: Vec<Line> = Vec::with_capacity(h);

    // Render each row of the spectrogram from 8.0 b/B (top) down to 0.0 b/B (bottom)
    for row in 0..plot_h {
        let frac_top = (plot_h - row) as f64 / plot_h as f64;
        let frac_bottom = (plot_h - 1 - row) as f64 / plot_h as f64;
        let e_top = frac_top * 8.0;
        let e_bottom = frac_bottom * 8.0;

        // Y-axis label on specific intervals
        let y_label = if row == 0 {
            " 8.0 ┤ "
        } else if row == plot_h / 4 {
            " 6.0 ┤ "
        } else if row == plot_h / 2 {
            " 4.0 ┤ "
        } else if row == (plot_h * 3) / 4 {
            " 2.0 ┤ "
        } else if row == plot_h - 1 {
            " 0.0 ┤ "
        } else {
            "     │ "
        };

        let mut spans = Vec::with_capacity(plot_w + 2);
        spans.push(Span::styled(y_label, Style::default().fg(Theme::MUTED)));

        for col in 0..plot_w {
            let idx = (col as f64 * step) as usize;
            let e = samples.get(idx).copied().unwrap_or(0.0);

            if e >= e_bottom {
                let color = Theme::entropy_color(e);
                if e < e_top {
                    // Top peak boundary glyph: choose sub-block height
                    let sub_frac = (e - e_bottom) / (e_top - e_bottom).max(0.001);
                    let glyph = if sub_frac < 0.25 {
                        " "
                    } else if sub_frac < 0.50 {
                        "▄"
                    } else if sub_frac < 0.75 {
                        "▅"
                    } else {
                        "█"
                    };
                    spans.push(Span::styled(
                        glyph,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ));
                } else {
                    // Filled column body
                    let ch = if e > 7.0 {
                        "█"
                    } else if e > 4.0 {
                        "▓"
                    } else if e > 2.0 {
                        "▒"
                    } else {
                        "░"
                    };
                    spans.push(Span::styled(ch, Style::default().fg(color)));
                }
            } else {
                // Background grid reference dot
                if row == plot_h / 2 {
                    spans.push(Span::styled("┄", Style::default().fg(Theme::BORDER)));
                } else {
                    spans.push(Span::raw(" "));
                }
            }
        }
        lines.push(Line::from(spans));
    }

    // ── Clean, Non-Overlapping LBA Bottom Axis ────────────────────────────────
    let mut axis_chars = vec!['─'; plot_w];
    // Place tick marks at 0%, 25%, 50%, 75%, 100%
    let tick_pcts = [0.0, 0.25, 0.50, 0.75, 1.0];
    for &pct in &tick_pcts {
        let pos = ((plot_w.saturating_sub(1) as f64) * pct).round() as usize;
        if pos < axis_chars.len() {
            axis_chars[pos] = '┬';
        }
    }
    let axis_str: String = axis_chars.into_iter().collect();
    lines.push(Line::from(vec![
        Span::styled("     └─", Style::default().fg(Theme::MUTED)),
        Span::styled(axis_str, Style::default().fg(Theme::MUTED)),
    ]));

    // Label row below ticks with exact calculated widths
    let total_gb = d.size_bytes as f64 / 1e9;
    let mut label_line = format!("{:>7} ", "");
    let quarter_w = plot_w / 4;
    let l0 = "0.0 GB";
    let l1 = format!("{:.1} GB", total_gb * 0.25);
    let l2 = format!("{:.1} GB", total_gb * 0.50);
    let l3 = format!("{:.1} GB", total_gb * 0.75);
    let l4 = format!("{:.1} GB", total_gb);

    label_line.push_str(&format!("{:<width$}", l0, width = quarter_w));
    label_line.push_str(&format!("{:<width$}", l1, width = quarter_w));
    label_line.push_str(&format!("{:<width$}", l2, width = quarter_w));
    label_line.push_str(&format!("{:<width$}", l3, width = quarter_w.saturating_sub(l4.len())));
    label_line.push_str(&l4);

    lines.push(Line::from(Span::styled(
        label_line,
        Style::default().fg(Theme::TEXT_DIM),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_stats(frame: &mut Frame, area: Rect, d: &crate::app::DriveEntry, avg_e: f64) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Statistical Distribution & Variance Analysis ",
            Style::default().fg(Theme::TEXT_DIM),
        ))
        .style(Style::default().bg(Theme::SURFACE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if !d.entropy_loaded || d.entropy_samples.is_empty() {
        return;
    }

    // All values are pre-computed — zero iteration here
    let stats = &d.entropy_stats;
    let n = d.entropy_samples.len();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    // KPI Summary Line
    let kpi_line = Line::from(vec![
        Span::styled("  Min: ", Style::default().fg(Theme::MUTED)),
        Span::styled(format!("{:.3}b  ", stats.min), Style::default().fg(Theme::entropy_color(stats.min)).add_modifier(Modifier::BOLD)),
        Span::styled("Avg: ", Style::default().fg(Theme::MUTED)),
        Span::styled(format!("{:.3}b  ", avg_e), Style::default().fg(Theme::entropy_color(avg_e)).add_modifier(Modifier::BOLD)),
        Span::styled("Max: ", Style::default().fg(Theme::MUTED)),
        Span::styled(format!("{:.3}b  ", stats.max), Style::default().fg(Theme::entropy_color(stats.max)).add_modifier(Modifier::BOLD)),
        Span::styled("Std Dev (σ): ", Style::default().fg(Theme::MUTED)),
        Span::styled(format!("{:.4}  ", stats.std_dev), Style::default().fg(Theme::CYAN)),
        Span::styled("Uniformity: ", Style::default().fg(Theme::MUTED)),
        Span::styled(
            if stats.std_dev < 0.15 {
                "■ UNIFORM PATTERN (Potential Wiped/CSPRNG)"
            } else if stats.std_dev > 1.5 {
                "▲ HIGH VARIANCE (Active Filesystem / Fragmented)"
            } else {
                "● NORMAL VARIANCE"
            },
            Style::default().fg(if stats.std_dev < 0.15 { Theme::SUCCESS } else { Theme::WARNING }),
        ),
    ]);
    frame.render_widget(Paragraph::new(kpi_line), chunks[0]);

    // Render 6-tier visual distribution bars using pre-computed bucket counts
    let bucket_meta = [
        ("0.0-1.0b Zeroed",  Theme::ENT_0),
        ("1.0-2.0b Sparse",  Theme::ENT_1),
        ("2.0-4.0b Text",    Theme::ENT_2),
        ("4.0-6.0b Binary",  Theme::ENT_3),
        ("6.0-7.5b Archive", Theme::ENT_4),
        ("7.5-8.0b Cipher",  Theme::ENT_5),
    ];

    let bar_area_w = (chunks[1].width as usize).saturating_sub(42).max(10);
    let mut bar_lines = Vec::new();

    for (bi, (label, color)) in bucket_meta.iter().enumerate() {
        let count = stats.buckets[bi];
        let pct = (count as f64 / n as f64) * 100.0;
        let filled_chars = ((pct / 100.0) * bar_area_w as f64).round() as usize;
        let bar_filled: String = "█".repeat(filled_chars);
        let bar_empty: String = "░".repeat(bar_area_w.saturating_sub(filled_chars));

        bar_lines.push(Line::from(vec![
            Span::styled(format!("  {:<16} ", label), Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(format!("{:>5.1}% ", pct), Style::default().fg(*color).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::default().fg(Theme::BORDER)),
            Span::styled(bar_filled, Style::default().fg(*color)),
            Span::styled(bar_empty, Style::default().fg(Theme::SURFACE2)),
            Span::styled("] ", Style::default().fg(Theme::BORDER)),
            Span::styled(format!("{:>4} sectors", count), Style::default().fg(Theme::MUTED)),
        ]));
    }

    frame.render_widget(Paragraph::new(bar_lines), chunks[1]);
}
