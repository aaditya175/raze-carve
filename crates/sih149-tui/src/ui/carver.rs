use super::format_bytes;
use crate::{app::App, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main[0]);

    render_config(frame, top[0], app);
    render_scan_progress(frame, top[1], app);
    render_results_header(frame, main[1], app);
    render_results(frame, main[2], app);
}

fn render_config(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.carver;
    let cursor = c.cursor_field;

    fn field<'a>(label: &'a str, val: &'a str, active: bool) -> Line<'a> {
        let val_color = if active { Theme::CYAN } else { Theme::TEXT };
        let border = if active { "▌ " } else { "  " };
        Line::from(vec![
            Span::styled(border, Style::default().fg(Theme::CYAN)),
            Span::styled(format!("{:<11}", label), Style::default().fg(Theme::MUTED)),
            Span::styled(
                if val.is_empty() {
                    "[empty — type here]"
                } else {
                    val
                },
                Style::default()
                    .fg(if val.is_empty() {
                        Theme::MUTED
                    } else {
                        val_color
                    })
                    .add_modifier(if active {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ])
    }

    let conf_color = match c.min_confidence {
        0..=49 => Theme::DANGER,
        50..=74 => Theme::WARNING,
        _ => Theme::SUCCESS,
    };

    let (mode_name, mode_desc) =
        crate::app::RECOVERY_MODES[c.recovery_mode_index % crate::app::RECOVERY_MODES.len()];

    let lines = vec![
        field("Source:", &c.source, cursor == 0),
        field("Output:", &c.output_dir, cursor == 1),
        Line::from(vec![
            Span::styled(
                if cursor == 2 { "▌ " } else { "  " },
                Style::default().fg(Theme::CYAN),
            ),
            Span::styled("Strategy:", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!(" [{}]", mode_name),
                Style::default()
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [Space/↑↓ cycle]", Style::default().fg(Theme::MUTED)),
        ]),
        Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                format!("▸ {}", mode_desc),
                Style::default().fg(Theme::TEXT_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                if cursor == 3 { "▌ " } else { "  " },
                Style::default().fg(Theme::CYAN),
            ),
            Span::styled("Threshold:", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!(" ≥{}%", c.min_confidence),
                Style::default().fg(conf_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [↑↓ adjust]", Style::default().fg(Theme::MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Mode: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                if c.inspect_only {
                    " [Inspect & Preview (Dry Run)] "
                } else {
                    " [Direct Recovery (Write)] "
                },
                Style::default()
                    .fg(if c.inspect_only {
                        Theme::WARNING
                    } else {
                        Theme::SUCCESS
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[i] Toggle", Style::default().fg(Theme::CYAN)),
        ]),
        Line::from(vec![
            Span::styled(
                "  [Tab] ",
                Style::default()
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Fields  ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                "[Enter] ",
                Style::default()
                    .fg(Theme::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if c.scanning {
                    "Scanning…"
                } else if c.inspect_only {
                    "Start Inspection Scan"
                } else {
                    "Start Deep Recovery"
                },
                Style::default().fg(Theme::TEXT),
            ),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if c.scanning {
            Theme::WARNING
        } else {
            Theme::BORDER
        }))
        .title(Span::styled(
            " ⬡ Forensic Recovery Engine  [Tab] Cycle Fields ",
            Style::default()
                .fg(Theme::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_scan_progress(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.carver;
    let safe_progress = if c.progress.is_nan() {
        0.0
    } else {
        c.progress.clamp(0.0, 1.0)
    };
    let pct = (safe_progress * 100.0) as u16;
    let status = if c.scanning {
        format!(
            "{} Scanning… {:.1}% ({} file{} carved)",
            Theme::spinner(app.tick),
            safe_progress * 100.0,
            c.found.len(),
            if c.found.len() == 1 { "" } else { "s" }
        )
    } else if pct == 100 {
        format!("Complete — {} file(s) recovered", c.found.len())
    } else {
        "Ready — configure and press Enter".to_string()
    };

    let gauge_color = if c.scanning {
        Theme::WARNING
    } else if pct == 100 {
        Theme::SUCCESS
    } else {
        Theme::MUTED
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(gauge_color))
                .title(Span::styled(
                    " Scan Progress ",
                    Style::default()
                        .fg(Theme::SUCCESS)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(Theme::SURFACE)),
        )
        .gauge_style(Style::default().fg(gauge_color))
        .percent(pct)
        .label(format!("  {}", status));
    frame.render_widget(gauge, rows[0]);

    let log_items: Vec<ListItem> = c
        .log
        .iter()
        .rev()
        .take(6)
        .map(|l| {
            ListItem::new(Span::styled(
                format!("▸ {}", l),
                Style::default().fg(Theme::TEXT_DIM),
            ))
        })
        .collect();
    let list = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(
                " Operation Log ",
                Style::default().fg(Theme::CYAN),
            ))
            .style(Style::default().bg(Theme::SURFACE)),
    );
    frame.render_widget(list, rows[1]);
}

fn render_results_header(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.carver;
    let mut spans = vec![Span::styled(
        format!("  Recovered: {} artifacts  │  ", c.found.len()),
        Style::default()
            .fg(Theme::CYAN)
            .add_modifier(Modifier::BOLD),
    )];

    if c.scanning {
        let spin = Theme::spinner(app.tick);
        spans.push(Span::styled(
            format!("{} Scanning sectors...  │  ", spin),
            Style::default()
                .fg(Theme::WARNING)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let mut types: Vec<&str> = c.found.iter().map(|f| f.file_type.as_str()).collect();
    types.sort_unstable();
    types.dedup();
    for t in &types {
        let count = c.found.iter().filter(|f| f.file_type == *t).count();
        spans.push(Span::styled(
            format!(" {} ({}) ", t, count),
            Style::default().fg(Theme::TEXT).bg(Theme::SURFACE2),
        ));
        spans.push(Span::raw(" "));
    }
    if types.is_empty() && !c.scanning {
        spans.push(Span::styled(
            "No files carved yet — run scan above",
            Style::default().fg(Theme::MUTED),
        ));
    }

    let line = Line::from(spans);
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG));
    let para = Paragraph::new(line).block(block).wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}

fn render_results(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.carver;
    if c.found.is_empty() {
        let lines = if c.scanning {
            vec![
                Line::raw(""),
                Line::from(vec![
                    Span::styled(
                        format!("  {} FORENSIC SECTOR & METADATA SCAN IN PROGRESS...", Theme::spinner(app.tick)),
                        Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::raw(""),
                Line::from(Span::styled("  Scanning raw sectors, NTFS MFT records, FAT directory entries, and entropy streams...", Style::default().fg(Theme::TEXT_DIM))),
                Line::from(Span::styled("  Recovered files will populate this table dynamically in real-time as they are discovered.", Style::default().fg(Theme::CYAN))),
            ]
        } else {
            vec![
                Line::raw(""),
                Line::from(Span::styled(
                    "  No carved files yet. Select a source (e.g. /dev/sdb or disk.dd) and press Enter.",
                    Style::default().fg(Theme::MUTED),
                )),
            ]
        };
        let para = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(Theme::BG));
        frame.render_widget(para, area);
        return;
    }

    let (table_area, preview_area) = if let Some(ref hex) = c.preview_hex {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);
        (chunks[0], Some((chunks[1], hex)))
    } else {
        (area, None)
    };

    let selected_count = c.found.iter().filter(|f| f.selected).count();

    let header = Row::new(vec![
        Cell::from("Sel").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(" File / Original Name").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Type").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Size").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Recovery Technique").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Conf%").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Extracted Metadata / Entropy").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = c
        .found
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let active_cursor = c.result_cursor == i;
            let conf_color = match f.confidence {
                0..=49 => Theme::DANGER,
                50..=74 => Theme::WARNING,
                _ => Theme::SUCCESS,
            };
            let ent_color = Theme::entropy_color(f.entropy);
            let (_, ent_level) = crate::app::entropy_label(f.entropy);
            let ent_bar =
                "█".repeat(ent_level as usize).to_string() + &"░".repeat(5 - ent_level as usize);

            let meta_display = if !f.metadata.is_empty() {
                format!("{} │ {:.1}b {}", f.metadata, f.entropy, ent_bar)
            } else {
                format!("{:.2} b/B {}", f.entropy, ent_bar)
            };

            let sel_icon = if f.selected { "[x]" } else { "[ ]" };
            let sel_color = if f.selected {
                Theme::SUCCESS
            } else {
                Theme::MUTED
            };

            Row::new(vec![
                Cell::from(sel_icon)
                    .style(Style::default().fg(sel_color).add_modifier(Modifier::BOLD)),
                Cell::from(format!(" {}", f.path.rsplit('/').next().unwrap_or(&f.path))).style(
                    Style::default().fg(if active_cursor {
                        Theme::CYAN
                    } else {
                        Theme::TEXT
                    }),
                ),
                Cell::from(f.file_type.clone()).style(Style::default().fg(Theme::BLUE)),
                Cell::from(format_bytes(f.size_bytes)).style(Style::default().fg(Theme::TEXT_DIM)),
                Cell::from(if f.method.is_empty() {
                    "Signature Magic".to_string()
                } else {
                    f.method.clone()
                })
                .style(Style::default().fg(Theme::TEAL)),
                Cell::from(format!("{}%", f.confidence))
                    .style(Style::default().fg(conf_color).add_modifier(Modifier::BOLD)),
                Cell::from(meta_display).style(Style::default().fg(ent_color)),
            ])
            .style(Style::default().bg(if active_cursor {
                Theme::SURFACE2
            } else {
                Theme::BG
            }))
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Length(28),
        Constraint::Length(8),
        Constraint::Length(9),
        Constraint::Length(24),
        Constraint::Length(7),
        Constraint::Min(0),
    ];

    let title_str = format!(
        " Carved Artifacts [Sel: {}/{}]  [Space] Select  [a] All  [p] Hex Preview  [x/e] Extract ",
        selected_count,
        c.found.len()
    );

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(
                title_str,
                Style::default()
                    .fg(Theme::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Theme::BG)),
    );
    frame.render_widget(table, table_area);

    if let Some((p_area, hex_str)) = preview_area {
        let p_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::CYAN))
            .title(Span::styled(
                " ⬡ Live Hex / Raw Evidence Inspection Preview  [p] Dismiss ",
                Style::default()
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Theme::SURFACE));
        let p_para = Paragraph::new(hex_str.as_str())
            .block(p_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(p_para, p_area);
    }
}
