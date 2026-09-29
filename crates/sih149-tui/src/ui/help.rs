use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, _app: &crate::app::App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_keybinds(frame, cols[0]);
    render_about(frame, cols[1]);
}

fn render_keybinds(frame: &mut Frame, area: Rect) {
    let sections: &[(&str, &[(&str, &str)])] = &[
        (
            "Global Navigation",
            &[
                ("F1", "Dashboard — system stats & live audit feed"),
                ("F2", "Drive Manager — block device selector & SMART"),
                ("F3", "Entropy Spectrogram — LBA density & Shannon visualizer"),
                ("F4", "Sanitize Wizard — NIST 800-88 / DoD erasure"),
                ("F5", "Forensic Carver — signature-based file recovery"),
                ("F6", "Surface Map — 2D sector grid, cluster map, spectrogram"),
                ("F7", "Diagnostics — live subsystem self-tests"),
                ("F8", "Help & Reference (this documentation)"),
                ("Q / Ctrl+C", "Quit SecureForge"),
            ],
        ),
        (
            "Drive Browser (F2)",
            &[
                ("↑ / ↓", "Navigate connected block devices"),
                ("Enter", "Select target drive for sanitization"),
                ("E", "Open full entropy heatmap for drive"),
                ("R", "Force hardware hotplug rescan"),
            ],
        ),
        (
            "Entropy View (F3)",
            &[
                ("↑ / ↓", "Switch between connected block devices"),
                ("E / Enter", "Run async entropy scan over sectors"),
            ],
        ),
        (
            "Sanitize Wizard (F4)",
            &[
                ("↑ / ↓", "Choose erasure standard (Zero/DoD/Gutmann)"),
                ("V", "Toggle cryptographic verification pass"),
                ("E", "Toggle expert passphrase unlock"),
                ("Enter", "Prompt confirmation dialog & initiate wipe"),
                ("Y / N", "Confirm permanent erasure / cancel"),
            ],
        ),
        (
            "Forensic Carver (F5)",
            &[
                ("Tab", "Cycle cursor between Source, Output, Conf"),
                ("↑ / ↓", "Adjust minimum confidence threshold"),
                ("Type", "Enter custom device or raw image path"),
                ("Backspace", "Erase typed characters"),
                ("Enter", "Execute carving engine on source"),
                ("p / P", "Live hex viewer for selected carved file"),
                ("Esc", "Close hex viewer modal"),
            ],
        ),
        (
            "Surface Map (F6)",
            &[
                ("↑ / ↓", "Switch target block device"),
                ("Tab / 1-3", "Toggle mode: Bad Sectors, Clusters, Entropy"),
                ("S / Space", "Execute physical surface sector sweep"),
                ("← / →", "Navigate individual sectors to inspect LBA/hex"),
                ("R", "Force rescan of surface cells"),
            ],
        ),
        (
            "Diagnostics & Tests (F7)",
            &[
                ("Enter / Space", "Run live diagnostic self-test suite"),
                ("R", "Re-execute all assertion checks"),
                (
                    "↑ / ↓ (k / j)",
                    "Scroll assertion results & inspect details",
                ),
            ],
        ),
    ];

    let mut lines = vec![Line::raw("")];
    for (section, keys) in sections {
        lines.push(Line::from(Span::styled(
            format!("  {}", section),
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  ──────────────────────────────────────",
            Style::default().fg(Theme::BORDER),
        )));
        for (key, desc) in *keys {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:15}", key),
                    Style::default()
                        .fg(Theme::PURPLE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(*desc, Style::default().fg(Theme::TEXT)),
            ]));
        }
        lines.push(Line::raw(""));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " ⌨  Keyboard Reference ",
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_about(frame: &mut Frame, area: Rect) {
    let entropy_legend: &[(&str, ratatui::style::Color, &str)] = &[
        ("░  0–1 bits/byte", Theme::ENT_0, "Dead / Zeroed sectors"),
        ("░  1–2 bits/byte", Theme::ENT_1, "Near-zero (mostly empty)"),
        (
            "▒  2–4 bits/byte",
            Theme::ENT_2,
            "Low entropy (text, FS structures)",
        ),
        (
            "▒  4–6 bits/byte",
            Theme::ENT_3,
            "Moderate (typical data files)",
        ),
        ("▓  6–7 bits/byte", Theme::ENT_4, "High (compressed data)"),
        ("█  7–8 bits/byte", Theme::ENT_5, "Encrypted / compressed"),
    ];

    let mut lines = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "  ▓ SecureForge v0.1.0",
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "  Integrated Secure Data Erasure &",
            Style::default().fg(Theme::TEXT),
        )),
        Line::from(Span::styled(
            "  Forensic File Recovery Platform",
            Style::default().fg(Theme::TEXT),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "  Standards Compliance:",
            Style::default().fg(Theme::MUTED),
        )),
        Line::from(Span::styled(
            "    ✓ NIST SP 800-88 Rev 1",
            Style::default().fg(Theme::SUCCESS),
        )),
        Line::from(Span::styled(
            "    ✓ DoD 5220.22-M  (3 & 7 pass)",
            Style::default().fg(Theme::SUCCESS),
        )),
        Line::from(Span::styled(
            "    ✓ Gutmann 35-pass",
            Style::default().fg(Theme::SUCCESS),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "  ──────────────── Entropy Legend ─────────────────",
            Style::default().fg(Theme::BORDER),
        )),
        Line::raw(""),
    ];

    for (sym, color, desc) in entropy_legend {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<18}", sym),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(*desc, Style::default().fg(Theme::TEXT_DIM)),
        ]));
    }

    lines.extend([
        Line::raw(""),
        Line::from(Span::styled(
            "  ──────────────────────────────────────────────────",
            Style::default().fg(Theme::BORDER),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "  ⚠ Always run with sudo for block device access.",
            Style::default().fg(Theme::WARNING),
        )),
        Line::from(Span::styled(
            "    Wipe operations are IRREVERSIBLE.",
            Style::default()
                .fg(Theme::DANGER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "  Built for Smart India Hackathon 2024 — SIH-149",
            Style::default().fg(Theme::MUTED),
        )),
        Line::from(Span::styled(
            "  License: MIT",
            Style::default().fg(Theme::MUTED),
        )),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " ⬡ About & Entropy Guide ",
            Style::default()
                .fg(Theme::PURPLE)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}
