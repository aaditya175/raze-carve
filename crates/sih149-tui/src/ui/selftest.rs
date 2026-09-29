use crate::{app::App, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Hero stats bar
            Constraint::Min(0),    // Results table
            Constraint::Length(5), // Selected test explanation
        ])
        .split(area);

    render_hero_bar(frame, chunks[0], app);
    render_table(frame, chunks[1], app);
    render_detail_card(frame, chunks[2], app);
}

fn render_hero_bar(frame: &mut Frame, area: Rect, app: &App) {
    let summary = app.test_summary.as_ref();
    let total = summary.map(|s| s.total).unwrap_or(0);
    let passed = summary.map(|s| s.passed).unwrap_or(0);
    let failed = summary.map(|s| s.failed).unwrap_or(0);
    let duration = summary.map(|s| s.total_duration_ms).unwrap_or(0.0);

    let status_text = if app.test_running {
        "  ⟳ RUNNING DIAGNOSTIC ASSERTIONS... "
    } else if summary.is_none() {
        "  READY — Press [Enter] or [Space] to run diagnostic self-tests "
    } else if failed == 0 {
        "  ✓ ALL SUBSYSTEMS HEALTHY — Zero Errors Detected "
    } else {
        "  ✗ FAILURES DETECTED — Check highlighted test details below "
    };

    let status_color = if app.test_running {
        Theme::WARNING
    } else if summary.is_none() {
        Theme::CYAN
    } else if failed == 0 {
        Theme::SUCCESS
    } else {
        Theme::DANGER
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                status_text,
                Style::default()
                    .fg(Theme::BG)
                    .bg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   ", Style::default()),
            Span::styled(
                format!("Total: {}  │  ", total),
                Style::default().fg(Theme::TEXT),
            ),
            Span::styled(
                format!("Passed: {}  │  ", passed),
                Style::default()
                    .fg(Theme::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("Failed: {}  │  ", failed),
                Style::default()
                    .fg(if failed > 0 {
                        Theme::DANGER
                    } else {
                        Theme::MUTED
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("Latency: {:.2} ms", duration),
                Style::default().fg(Theme::TEXT_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  [Enter / Space] ",
                Style::default()
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Run All Diagnostics    ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                "[↑↓] ",
                Style::default()
                    .fg(Theme::PURPLE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Select Test    ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                "[F1–F7] ",
                Style::default()
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Switch Screens", Style::default().fg(Theme::MUTED)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(status_color))
        .title(Span::styled(
            " ⬡ Subsystem Self-Test Engine ",
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn render_table(frame: &mut Frame, area: Rect, app: &App) {
    let header = Row::new(vec![
        Cell::from(" Status").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Category").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Assertion Name").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Latency").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Verification Output").style(
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = match &app.test_summary {
        Some(summary) => summary
            .results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let selected = app.test_cursor == i;
                let (status_icon, status_color) = if r.passed {
                    ("✓ PASS", Theme::SUCCESS)
                } else {
                    ("✗ FAIL", Theme::DANGER)
                };

                let category_color = match r.category.as_str() {
                    "Wiper" => Theme::CYAN,
                    "Safety" => Theme::DANGER,
                    "Carver" => Theme::PURPLE,
                    "Crypto" => Theme::WARNING,
                    "Database" => Theme::BLUE,
                    "Classifier" => Theme::TEAL,
                    _ => Theme::TEXT,
                };

                let row_bg = if selected { Theme::SURFACE2 } else { Theme::BG };
                let indicator = if selected { "▶ " } else { "  " };

                Row::new(vec![
                    Cell::from(format!("{}{}", indicator, status_icon)).style(
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(r.category.clone()).style(
                        Style::default()
                            .fg(category_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(r.name.clone()).style(Style::default().fg(if selected {
                        Theme::CYAN
                    } else {
                        Theme::TEXT
                    })),
                    Cell::from(format!("{} µs", r.duration_us))
                        .style(Style::default().fg(Theme::TEXT_DIM)),
                    Cell::from(r.message.clone()).style(Style::default().fg(if r.passed {
                        Theme::TEXT
                    } else {
                        Theme::DANGER
                    })),
                ])
                .style(Style::default().bg(row_bg))
                .height(1)
            })
            .collect(),
        None => vec![Row::new(vec![
            Cell::from("  ─"),
            Cell::from("─"),
            Cell::from("Press [Enter] or [Space] to execute self-tests"),
            Cell::from("─"),
            Cell::from("Verifies Wiper, Safety, Carver, Crypto, and Database live"),
        ])
        .style(Style::default().fg(Theme::MUTED))],
    };

    let widths = [
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(34),
        Constraint::Length(10),
        Constraint::Min(0),
    ];

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(
                " Diagnostic Test Results  [↑↓] Scroll ",
                Style::default()
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Theme::BG)),
    );
    frame.render_widget(table, area);
}

fn render_detail_card(frame: &mut Frame, area: Rect, app: &App) {
    let summary = app.test_summary.as_ref();
    let current_result = summary.and_then(|s| s.results.get(app.test_cursor));

    let content = if let Some(r) = current_result {
        let (status_str, color) = if r.passed {
            ("VERIFIED PASSED", Theme::SUCCESS)
        } else {
            ("FAILED", Theme::DANGER)
        };
        vec![
            Line::from(vec![
                Span::styled(
                    format!("  {} : ", r.name),
                    Style::default()
                        .fg(Theme::CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", status_str),
                    Style::default()
                        .fg(Theme::BG)
                        .bg(color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "  Category: {}  │  Execution Time: {} microseconds",
                        r.category, r.duration_us
                    ),
                    Style::default().fg(Theme::MUTED),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Assertion: ", Style::default().fg(Theme::MUTED)),
                Span::styled(&r.message, Style::default().fg(Theme::TEXT)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "  No test selected. Press [Enter] to run diagnostic tests.",
            Style::default().fg(Theme::MUTED),
        ))]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Assertion Inspector ",
            Style::default()
                .fg(Theme::PURPLE)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}
