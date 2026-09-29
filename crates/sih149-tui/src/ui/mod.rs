pub mod carver;
pub mod dashboard;
pub mod drives;
pub mod entropy;
pub mod help;
pub mod selftest;
pub mod surfacemap;
pub mod wipe;

use crate::app::{App, Popup, Screen};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Main render entry point
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header + tab bar
            Constraint::Min(0),    // content
            Constraint::Length(1), // status bar
        ])
        .split(area);

    render_header(frame, chunks[0], app);

    match app.screen {
        Screen::Dashboard => dashboard::render(frame, chunks[1], app),
        Screen::DriveManager => drives::render(frame, chunks[1], app),
        Screen::Entropy => entropy::render(frame, chunks[1], app),
        Screen::WipeWizard => wipe::render(frame, chunks[1], app),
        Screen::Carver => carver::render(frame, chunks[1], app),
        Screen::SurfaceMap => surfacemap::render(frame, chunks[1], app),
        Screen::SelfTest => selftest::render(frame, chunks[1], app),
        Screen::Help => help::render(frame, chunks[1], app),
    }

    render_statusbar(frame, chunks[2], app);

    if app.popup != Popup::None {
        render_popup(frame, area, app);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(area);

    let tabs: &[(&str, Screen)] = &[
        (" 1 Dash ", Screen::Dashboard),
        (" 2 Drives ", Screen::DriveManager),
        (" 3 Entropy ", Screen::Entropy),
        (" 4 Sanitize ", Screen::WipeWizard),
        (" 5 Carver ", Screen::Carver),
        (" 6 Surface ", Screen::SurfaceMap),
        (" 7 Tests ", Screen::SelfTest),
        (" 8 Help ", Screen::Help),
    ];

    let tab_spans: Vec<Span> = tabs
        .iter()
        .map(|(label, screen)| {
            if &app.screen == screen {
                Span::styled(
                    *label,
                    Style::default()
                        .fg(Theme::BG)
                        .bg(Theme::CYAN)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    *label,
                    Style::default().fg(Theme::TEXT_DIM).bg(Theme::SURFACE),
                )
            }
        })
        .collect();

    // Left: animated logo + tabs
    let logo_text = format!(" ⬡ SECUREFORGE {} ", Theme::spinner(app.tick));
    let logo = Span::styled(
        logo_text,
        Style::default()
            .fg(Theme::pulse_color(app.tick))
            .bg(Theme::BG)
            .add_modifier(Modifier::BOLD),
    );
    let sep = Span::styled("│", Style::default().fg(Theme::BORDER).bg(Theme::SURFACE));

    let mut spans = vec![
        logo,
        Span::styled("  ", Style::default().bg(Theme::SURFACE)),
    ];
    for (i, s) in tab_spans.into_iter().enumerate() {
        if i > 0 {
            spans.push(sep.clone());
        }
        spans.push(s);
    }

    let block = Block::default()
        .style(Style::default().bg(Theme::SURFACE))
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Theme::BORDER));
    let para = Paragraph::new(Line::from(spans)).block(block.clone());
    frame.render_widget(para, header_chunks[0]);

    // Right: Live activity equalizer + defense grade status
    let activity = Theme::activity_bar(app.tick);
    let right_pill = Line::from(vec![
        Span::styled(format!(" {} ", activity), Style::default().fg(Theme::TEAL)),
        Span::styled("● ", Style::default().fg(Theme::pulse_color(app.tick))),
        Span::styled(
            "DEFENSE-GRADE ",
            Style::default()
                .fg(Theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let right_para = Paragraph::new(right_pill)
        .alignment(ratatui::layout::Alignment::Right)
        .block(block);
    frame.render_widget(right_para, header_chunks[1]);
}

fn render_statusbar(frame: &mut Frame, area: Rect, app: &App) {
    let spinner = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"][(app.tick / 2) as usize % 8];

    let (msg, is_err) = if let Some((ref m, ref t, err)) = app.status_msg {
        if t.elapsed().as_secs() < 6 {
            (m.as_str(), err)
        } else {
            ("", false)
        }
    } else {
        ("", false)
    };

    let msg_span = if !msg.is_empty() {
        if is_err {
            Span::styled(
                format!(" ✗ {} ", msg),
                Style::default()
                    .fg(Theme::DANGER)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                format!(" ✓ {} ", msg),
                Style::default()
                    .fg(Theme::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            )
        }
    } else {
        let hint_str = match app.screen {
            Screen::Dashboard => " [1–8] Mode  [Tab/[] Prev/Next  [Enter] Drives  [Q] Quit ",
            Screen::DriveManager => {
                " [↑↓] Navigate  [Enter] Select for Wipe  [E] Entropy  [1–8] Mode  [Q] Quit "
            }
            Screen::Entropy => {
                " [↑↓] Drive  [E/Enter] Scan Entropy  [1–8] Mode  [Q] Quit "
            }
            Screen::WipeWizard => {
                " [↑↓] Method  [V] Verify  [E] Expert  [Enter] Wipe  [1–8] Mode  [Q] Quit "
            }
            Screen::Carver => {
                " [Tab] Field  [↑↓] Mode/Conf  [Enter] Carve  [1–8] Mode  [Q] Quit "
            }
            Screen::SurfaceMap => {
                " [Tab/M] Mode  [S/Space] Scan  [↑↓] Drive  [←→] Sector  [1–8] Mode  [Q] Quit "
            }
            Screen::SelfTest => {
                " [Enter/Space] Run Tests  [↑↓] Scroll  [1–8] Mode  [Q] Quit "
            }
            Screen::Help => " [1–8] Switch Mode  [Q] Quit ",
        };
        Span::styled(
            format!(" {} {} ", spinner, hint_str),
            Style::default().fg(Theme::CYAN),
        )
    };

    let badge = Span::styled(
        " NIST SP 800-88 R1 │ SecureForge v0.1.0 ",
        Style::default().fg(Theme::INDIGO),
    );

    let line = Line::from(vec![msg_span, badge]);
    let para = Paragraph::new(line).style(Style::default().bg(Theme::BG));
    frame.render_widget(para, area);
}

pub fn render_popup(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(62, 35, area);
    frame.render_widget(Clear, popup_area);

    let (title, body, color) = match &app.popup {
        Popup::Confirm { title, message } => (title.as_str(), message.as_str(), Theme::WARNING),
        Popup::Error(e) => ("  Error ", e.as_str(), Theme::DANGER),
        Popup::Info(i) => ("  Info  ", i.as_str(), Theme::CYAN),
        Popup::None => return,
    };

    let is_confirm = matches!(app.popup, Popup::Confirm { .. });

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::SURFACE2));

    let buttons = if is_confirm {
        "\n\n  [Y] Confirm Erasure     [N] Cancel"
    } else {
        "\n\n  [Enter / Esc] Close"
    };

    let text = format!("\n  {}\n{}", body, buttons);
    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Theme::TEXT));
    frame.render_widget(para, popup_area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}

pub fn format_bytes(n: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = n as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < units.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    format!("{:.1} {}", size, units[idx])
}

pub fn format_dur(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
