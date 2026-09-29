use crate::{
    app::{App, SurfaceVisMode},
    theme::Theme,
    ui::format_bytes,
};
/// Physical Surface & Sector Map Visualizer
/// Displays interactive 2D bad sector surface grid, filesystem cluster allocation,
/// and Shannon entropy spectrogram with live telemetry.
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sih149_core::disk::surface_scanner::{ClusterAllocation, SectorHealth};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Top header & mode selector
            Constraint::Min(10),   // 2D Sector Grid
            Constraint::Length(3), // Dynamic Legend
            Constraint::Length(8), // Bottom KPIs & Sector Inspector
        ])
        .split(area);

    render_header(frame, main_layout[0], app);
    render_grid(frame, main_layout[1], app);
    render_legend(frame, main_layout[2], app);
    render_telemetry_and_inspector(frame, main_layout[3], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let sm = &app.surface_map;
    let drive_info = app.drives.get(sm.drive_cursor);
    let drive_name = drive_info.map(|d| d.name.as_str()).unwrap_or("None");
    let drive_type = drive_info
        .map(|d| d.drive_type.as_str())
        .unwrap_or("Unknown");
    let drive_size = drive_info
        .map(|d| format_bytes(d.size_bytes))
        .unwrap_or_else(|| "0 B".to_string());

    let mode_spans = vec![
        Span::styled(
            if sm.vis_mode == SurfaceVisMode::BadSectors {
                " [1. Bad Sector Health] "
            } else {
                "  1. Bad Sector Health  "
            },
            if sm.vis_mode == SurfaceVisMode::BadSectors {
                Style::default()
                    .fg(Theme::BG)
                    .bg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::MUTED)
            },
        ),
        Span::raw("  "),
        Span::styled(
            if sm.vis_mode == SurfaceVisMode::ClusterAllocation {
                " [2. Cluster Allocation] "
            } else {
                "  2. Cluster Allocation  "
            },
            if sm.vis_mode == SurfaceVisMode::ClusterAllocation {
                Style::default()
                    .fg(Theme::BG)
                    .bg(Theme::PURPLE)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::MUTED)
            },
        ),
        Span::raw("  "),
        Span::styled(
            if sm.vis_mode == SurfaceVisMode::EntropySpectrogram {
                " [3. Entropy Spectrogram] "
            } else {
                "  3. Entropy Spectrogram  "
            },
            if sm.vis_mode == SurfaceVisMode::EntropySpectrogram {
                Style::default()
                    .fg(Theme::BG)
                    .bg(Theme::WARNING)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::MUTED)
            },
        ),
    ];

    let lines = vec![
        Line::from(vec![
            Span::styled(" Target Drive [↑↓]: ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(
                format!("/dev/{} ", drive_name),
                Style::default()
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({} • {})   ", drive_type, drive_size),
                Style::default().fg(Theme::MUTED),
            ),
            Span::styled("Mode [Tab / 1-3]: ", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::from(mode_spans),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " ▓ Surface & Sector Visualizer [S] Start Scan [R] Rescan [←→] Select Sector ",
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_grid(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            format!(
                " Surface Grid Matrix ({}) ",
                app.surface_map.vis_mode.label()
            ),
            Style::default()
                .fg(Theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sm = &app.surface_map;
    if sm.scanning {
        let pulse = Theme::spinner(app.tick);
        let dots = ".".repeat(((app.tick / 4) % 6 + 1) as usize);
        let prompt = Paragraph::new(vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled(
                    format!("  {} SCANNING PHYSICAL SURFACE & CLUSTERS IN PROGRESS{}", pulse, dots),
                    Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::raw(""),
            Line::from(Span::styled("  Analyzing sector read latency, bad blocks, and filesystem allocation map...", Style::default().fg(Theme::TEXT_DIM))),
            Line::from(Span::styled("  Background worker active — live surface cells will populate when pass completes.", Style::default().fg(Theme::CYAN))),
        ]);
        frame.render_widget(prompt, inner);
        return;
    }

    let res = match &sm.scan_result {
        Some(r) => r,
        None => {
            let prompt = Paragraph::new(vec![
                Line::raw(""),
                Line::from(Span::styled("  Surface has not been scanned yet.", Style::default().fg(Theme::MUTED))),
                Line::from(Span::styled("  Press [S] or [Space] to perform a high-resolution surface & cluster analysis.", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))),
            ]);
            frame.render_widget(prompt, inner);
            return;
        }
    };

    let cols = (inner.width as usize).saturating_sub(4).max(16);
    let rows = inner.height as usize;
    let cells = &res.cells;

    let mut lines = Vec::new();
    let selected_idx = sm.selected_cell;

    for r in 0..rows {
        let mut spans = Vec::new();
        spans.push(Span::raw("  "));

        for c in 0..cols {
            let idx = r * cols + c;
            if idx >= cells.len() {
                break;
            }
            let cell = &cells[idx];
            let is_sel = idx == selected_idx;

            let (glyph, color) = match sm.vis_mode {
                SurfaceVisMode::BadSectors => match cell.health {
                    SectorHealth::Healthy => ("■", Theme::SUCCESS),
                    SectorHealth::Normal => ("■", Theme::CYAN),
                    SectorHealth::Slow => ("▲", Theme::WARNING),
                    SectorHealth::Bad => ("✖", Theme::DANGER),
                    SectorHealth::Reallocated => ("◆", Theme::PURPLE),
                    SectorHealth::Zeroed => ("_", Theme::BLUE),
                },
                SurfaceVisMode::ClusterAllocation => match cell.allocation {
                    ClusterAllocation::Unknown => ("?", Theme::MUTED),
                    ClusterAllocation::SystemMft => ("§", Theme::PURPLE),
                    ClusterAllocation::ActiveFile => ("■", Theme::BLUE),
                    ClusterAllocation::Fragmented => ("≈", Theme::WARNING),
                    ClusterAllocation::CarvableDeleted => ("★", Theme::SUCCESS),
                    ClusterAllocation::FreeSpace => ("·", Theme::MUTED),
                    ClusterAllocation::Wiped => ("_", Theme::SURFACE2),
                    ClusterAllocation::BadSector => ("✖", Theme::DANGER),
                },
                SurfaceVisMode::EntropySpectrogram => {
                    let e = cell.entropy;
                    let ch = if e < 1.0 {
                        " "
                    } else if e < 3.0 {
                        "░"
                    } else if e < 6.0 {
                        "▒"
                    } else if e < 7.5 {
                        "▓"
                    } else {
                        "█"
                    };
                    (ch, Theme::entropy_color(e))
                }
            };

            if is_sel {
                spans.push(Span::styled(
                    glyph,
                    Style::default()
                        .fg(Theme::BG)
                        .bg(color)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(glyph, Style::default().fg(color)));
            }
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_legend(frame: &mut Frame, area: Rect, app: &App) {
    let sm = &app.surface_map;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Legend ",
            Style::default().fg(Theme::TEXT_DIM),
        ))
        .style(Style::default().bg(Theme::SURFACE));

    let spans = match sm.vis_mode {
        SurfaceVisMode::BadSectors => vec![
            Span::raw("  "),
            Span::styled("■ ", Style::default().fg(Theme::SUCCESS)),
            Span::styled("<15ms Healthy   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("■ ", Style::default().fg(Theme::CYAN)),
            Span::styled("15-80ms Normal   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("▲ ", Style::default().fg(Theme::WARNING)),
            Span::styled(">80ms Slow   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(
                "✖ ",
                Style::default()
                    .fg(Theme::DANGER)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("UNC Bad Sector   ", Style::default().fg(Theme::DANGER)),
            Span::styled("◆ ", Style::default().fg(Theme::PURPLE)),
            Span::styled("Reallocated   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("_ ", Style::default().fg(Theme::BLUE)),
            Span::styled("Zeroed/Wiped", Style::default().fg(Theme::TEXT_DIM)),
        ],
        SurfaceVisMode::ClusterAllocation => vec![
            Span::raw("  "),
            Span::styled("§ ", Style::default().fg(Theme::PURPLE)),
            Span::styled("System/$MFT   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("■ ", Style::default().fg(Theme::BLUE)),
            Span::styled("Active File   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("≈ ", Style::default().fg(Theme::WARNING)),
            Span::styled("Fragmented Run   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("★ ", Style::default().fg(Theme::SUCCESS)),
            Span::styled("Carvable Deleted   ", Style::default().fg(Theme::SUCCESS)),
            Span::styled("· ", Style::default().fg(Theme::MUTED)),
            Span::styled("Free Unallocated   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("_ ", Style::default().fg(Theme::SURFACE2)),
            Span::styled("Sanitized Wiped", Style::default().fg(Theme::TEXT_DIM)),
        ],
        SurfaceVisMode::EntropySpectrogram => vec![
            Span::raw("  "),
            Span::styled("░ 0-1b ", Style::default().fg(Theme::ENT_0)),
            Span::styled("Zero/Dead   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("▒ 1-4b ", Style::default().fg(Theme::ENT_2)),
            Span::styled("Low/Text   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("▒ 4-6b ", Style::default().fg(Theme::ENT_3)),
            Span::styled("Executables   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("▓ 6-7.5b ", Style::default().fg(Theme::ENT_4)),
            Span::styled("Packed Binary   ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("█ 7.5-8b ", Style::default().fg(Theme::ENT_5)),
            Span::styled("Encrypted/Cipher", Style::default().fg(Theme::DANGER)),
        ],
    };

    frame.render_widget(Paragraph::new(vec![Line::from(spans)]).block(block), area);
}

fn render_telemetry_and_inspector(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let sm = &app.surface_map;

    // Left: Telemetry & KPI Panel
    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Surface Diagnostics Telemetry ",
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));

    let left_lines = if let Some(ref r) = sm.scan_result {
        let health_color = if r.health_score > 98.0 {
            Theme::SUCCESS
        } else if r.health_score > 85.0 {
            Theme::WARNING
        } else {
            Theme::DANGER
        };
        vec![
            Line::from(vec![
                Span::styled("  Health Rating: ", Style::default().fg(Theme::TEXT_DIM)),
                Span::styled(
                    format!("{:.1}% ", r.health_score),
                    Style::default()
                        .fg(health_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if r.health_score > 98.0 {
                        "[OPTIMAL]"
                    } else {
                        "[DEGRADED]"
                    },
                    Style::default().fg(health_color),
                ),
                Span::styled(
                    format!(
                        "  •  Throughput: {:.0} MB/s",
                        if r.scan_duration_ms > 0 {
                            (r.total_size_bytes as f64 / (r.scan_duration_ms as f64 / 1000.0)) / 1e6
                        } else {
                            240.0
                        }
                    ),
                    Style::default().fg(Theme::TEXT_DIM),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Good Sectors: ", Style::default().fg(Theme::TEXT_DIM)),
                Span::styled(
                    format!("{} ", r.good_count),
                    Style::default()
                        .fg(Theme::SUCCESS)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  Slow: ", Style::default().fg(Theme::TEXT_DIM)),
                Span::styled(
                    format!("{} ", r.slow_count),
                    Style::default()
                        .fg(Theme::WARNING)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  Bad (UNC): ", Style::default().fg(Theme::TEXT_DIM)),
                Span::styled(
                    format!("{} ", r.bad_count),
                    Style::default()
                        .fg(if r.bad_count > 0 {
                            Theme::DANGER
                        } else {
                            Theme::MUTED
                        })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  Zeroed: ", Style::default().fg(Theme::TEXT_DIM)),
                Span::styled(
                    format!("{}", r.zeroed_count),
                    Style::default().fg(Theme::BLUE),
                ),
            ]),
            Line::from(vec![Span::styled(
                format!(
                    "  Defects Logged: {}  |  Sampled Blocks: {} (4KB cluster)",
                    r.defects.len(),
                    r.total_blocks
                ),
                Style::default().fg(Theme::MUTED),
            )]),
        ]
    } else {
        vec![
            Line::raw(""),
            Line::from(Span::styled(
                "  No telemetry data. Press [S] to scan.",
                Style::default().fg(Theme::MUTED),
            )),
        ]
    };

    frame.render_widget(Paragraph::new(left_lines).block(left_block), chunks[0]);

    // Right: Selected Sector Inspector
    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Sector Deep Inspector [←→ Navigate] ",
            Style::default()
                .fg(Theme::PURPLE)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));

    let right_lines = if let Some(ref r) = sm.scan_result {
        if let Some(cell) = r.cells.get(sm.selected_cell) {
            let alloc_str = cell
                .file_hint
                .clone()
                .unwrap_or_else(|| format!("{:?}", cell.allocation));
            vec![
                Line::from(vec![
                    Span::styled(
                        format!("  Block #{:<4} ", cell.index),
                        Style::default()
                            .fg(Theme::CYAN)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("LBA: ", Style::default().fg(Theme::TEXT_DIM)),
                    Span::styled(
                        format!("0x{:08X}  ", cell.lba),
                        Style::default()
                            .fg(Theme::TEXT)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Offset: ", Style::default().fg(Theme::TEXT_DIM)),
                    Span::styled(
                        format!("0x{:08X}", cell.offset_bytes),
                        Style::default().fg(Theme::TEXT_DIM),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Latency: ", Style::default().fg(Theme::TEXT_DIM)),
                    Span::styled(
                        format!("{:.2} ms  ", cell.latency_ms),
                        Style::default().fg(Theme::WARNING),
                    ),
                    Span::styled("Entropy: ", Style::default().fg(Theme::TEXT_DIM)),
                    Span::styled(
                        format!("{:.3} b/B  ", cell.entropy),
                        Style::default().fg(Theme::entropy_color(cell.entropy)),
                    ),
                    Span::styled("Class: ", Style::default().fg(Theme::TEXT_DIM)),
                    Span::styled(
                        alloc_str,
                        Style::default()
                            .fg(Theme::SUCCESS)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Preview: ", Style::default().fg(Theme::TEXT_DIM)),
                    Span::styled(
                        match cell.allocation {
                            ClusterAllocation::FreeSpace | ClusterAllocation::Wiped => {
                                "00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  [Clean 0x00]"
                            }
                            ClusterAllocation::SystemMft => {
                                "46 49 4C 45 30 00 03 00 28 00 00 00 01 00 00 00  [NTFS $MFT RECORD]"
                            }
                            ClusterAllocation::CarvableDeleted => {
                                if let Some(ref h) = cell.file_hint {
                                    if h.contains("JPEG") {
                                        "FF D8 FF E0 00 10 4A 46 49 46 00 01 01 01 00 48  [JPEG EXIF SOI]"
                                    } else if h.contains("PNG") {
                                        "89 50 4E 47 0D 0A 1A 0A 00 00 00 0D 49 48 44 52  [PNG SIGNATURE]"
                                    } else if h.contains("PDF") {
                                        "25 50 44 46 2D 31 2E 37 0A 25 E2 E3 CF D3 0A 34  [PDF HEADER]"
                                    } else {
                                        "50 4B 03 04 14 00 00 00 08 00 00 00 00 00 00 00  [ZIP ARCHIVE]"
                                    }
                                } else {
                                    "FF D8 FF E1 12 34 45 78 00 00 00 00 50 4B 03 04  [Carvable Signature]"
                                }
                            }
                            ClusterAllocation::BadSector => {
                                "XX XX XX XX XX XX XX XX XX XX XX XX XX XX XX XX  [I/O SENSE: UNC]"
                            }
                            _ => {
                                if cell.health == SectorHealth::Zeroed {
                                    "00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  [Zero-Filled Cluster]"
                                } else {
                                    "48 65 78 20 44 61 74 61 20 53 65 67 6D 65 6E 74  [Structured File Stream]"
                                }
                            }
                        },
                        Style::default().fg(Theme::MUTED),
                    ),
                ]),
            ]
        } else {
            vec![Line::from(Span::styled(
                "  Select a sector to inspect.",
                Style::default().fg(Theme::MUTED),
            ))]
        }
    } else {
        vec![
            Line::raw(""),
            Line::from(Span::styled(
                "  No sector selected.",
                Style::default().fg(Theme::MUTED),
            )),
        ]
    };

    frame.render_widget(Paragraph::new(right_lines).block(right_block), chunks[1]);
}
