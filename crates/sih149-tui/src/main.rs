use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod app;
mod theme;
mod ui;
mod worker;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("SecureForge Interactive Forensic TUI\n\nUsage: sforge [OPTIONS]\n\nOptions:\n  -h, --help     Print help\n  -V, --version  Print version");
        return Ok(());
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("sforge {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    enable_raw_mode()?;
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = Arc::new(Mutex::new(app::App::new()));
    {
        let mut a = app.lock().unwrap();
        a.load_drives();
        a.push_log_level(app::LogLevel::Info, "SecureForge TUI started");
        let n = a.drives.len();
        a.push_log_level(
            if n > 0 {
                app::LogLevel::Success
            } else {
                app::LogLevel::Warning
            },
            format!("{} block device(s) detected", n),
        );
        if n == 0 {
            a.push_log_level(app::LogLevel::Warning, "Try running with: sudo sforge");
        }
    }

    // Dedicated background thread for hotplug drive detection (every 3s, non-blocking, skips during wipes/scans)
    let bg_app = Arc::clone(&app);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(3));
            let (should_skip, should_quit) = {
                if let Ok(a) = bg_app.try_lock() {
                    let busy = matches!(a.wipe_phase, app::WipePhase::Running { .. } | app::WipePhase::Verifying)
                        || a.carver.scanning
                        || a.surface_map.scanning;
                    (busy, a.should_quit)
                } else {
                    (true, false)
                }
            };
            if should_quit {
                break;
            }
            if !should_skip {
                let current = app::App::query_current_drives();
                if let Ok(mut a) = bg_app.lock() {
                    a.apply_drive_changes(current);
                }
            }
        }
    });

    let tick_rate = Duration::from_millis(50); // 20fps

    loop {
        // Try to acquire the lock non-blockingly. If busy (worker writing scan
        // results), sleep a short time and skip this frame — never block the
        // UI thread waiting for the mutex.
        let lock_result = app.try_lock();
        let mut guard = match lock_result {
            Ok(g) => g,
            Err(_) => {
                // Lock held by worker — sleep briefly and retry next iteration
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
        };

        // Advance animation tick and render in a single lock acquisition
        guard.tick = guard.tick.wrapping_add(1);
        terminal.draw(|f| ui::render(f, &mut guard))?;
        drop(guard);

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // Ignore key-release events (crossterm sends both on some terminals)
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }
                let needs_wipe = {
                    // Spin-try for up to 20ms before giving up on key this frame
                    let mut retries = 4;
                    let mut result = None;
                    loop {
                        match app.try_lock() {
                            Ok(mut a) => {
                                result = Some(handle_key(&mut a, key.code, key.modifiers, Arc::clone(&app)));
                                break;
                            }
                            Err(_) if retries > 0 => {
                                retries -= 1;
                                std::thread::sleep(Duration::from_millis(5));
                            }
                            Err(_) => break, // Drop the key event rather than freeze
                        }
                    }
                    result.flatten()
                };
                if let Some((path, method, verify)) = needs_wipe {
                    {
                        let mut a = app.lock().unwrap();
                        a.wipe_phase = app::WipePhase::Running {
                            pass: 1,
                            total_passes: method.passes(),
                            bytes_done: 0,
                            bytes_total: 1,
                            speed_mbps: 0.0,
                            started: std::time::Instant::now(),
                        };
                        a.push_log_level(
                            app::LogLevel::Warning,
                            format!("Wipe started: {} → {}", path, method.label()),
                        );
                    }
                    worker::start_wipe(Arc::clone(&app), path, method, verify);
                }
            }
        }

        if app.lock().unwrap().should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Returns Some((path, method, verify)) when a wipe should be started in a background thread.
fn handle_key(
    app: &mut app::App,
    code: KeyCode,
    modifiers: KeyModifiers,
    app_arc: Arc<Mutex<app::App>>,
) -> Option<(String, app::WipeMethod, bool)> {
    // Global: quit (suppressed if operator is typing in carver text input fields)
    let in_text_input = app.screen == app::Screen::Carver
        && (app.carver.cursor_field == 0 || app.carver.cursor_field == 1);
    if (!in_text_input && matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')))
        || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
    {
        app.wipe_cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
        app.carver_cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
        app.should_quit = true;
        return None;
    }

    // Popup handling takes priority
    if app.popup != app::Popup::None {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                if let app::Popup::Confirm { .. } = &app.popup.clone() {
                    let drive_path = app
                        .selected_drive
                        .and_then(|i| app.drives.get(i))
                        .map(|d| d.path.clone());
                    let method = app.selected_method();
                    let do_verify = app.wipe_verify;
                    app.popup = app::Popup::None;
                    if let Some(path) = drive_path {
                        return Some((path, method, do_verify));
                    }
                } else {
                    app.popup = app::Popup::None;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.popup = app::Popup::None;
                if matches!(app.wipe_phase, app::WipePhase::Confirming) {
                    app.wipe_phase = app::WipePhase::Idle;
                }
            }
            _ => {}
        }
        return None;
    }

    // Modal / screen-specific Esc handling takes precedence over screen navigation
    if code == KeyCode::Esc {
        if app.carver.preview_hex.is_some() {
            app.carver.preview_hex = None;
            return None;
        }
        // Go back to previous screen
        let prev = app.prev_screen.clone();
        app.navigate(prev);
        return None;
    }

    // Global screen navigation via number keys and function keys
    if !in_text_input {
        // When on SurfaceMap, 1/2/3 switch visualizer submodes, so let handle_surfacemap handle them
        let is_surfacemap_submode = app.screen == app::Screen::SurfaceMap
            && matches!(code, KeyCode::Char('1') | KeyCode::Char('2') | KeyCode::Char('3'));

        if !is_surfacemap_submode {
            match code {
                KeyCode::Char('1') | KeyCode::F(1) => {
                    app.navigate(app::Screen::Dashboard);
                    return None;
                }
                KeyCode::Char('2') | KeyCode::F(2) => {
                    app.navigate(app::Screen::DriveManager);
                    return None;
                }
                KeyCode::Char('3') | KeyCode::F(3) => {
                    app.entropy_drive_cursor = app.drive_cursor;
                    app.navigate(app::Screen::Entropy);
                    return None;
                }
                KeyCode::Char('4') | KeyCode::F(4) => {
                    app.navigate(app::Screen::WipeWizard);
                    return None;
                }
                KeyCode::Char('5') | KeyCode::F(5) => {
                    app.navigate(app::Screen::Carver);
                    return None;
                }
                KeyCode::Char('6') | KeyCode::F(6) => {
                    app.surface_map.drive_cursor = app.drive_cursor;
                    app.navigate(app::Screen::SurfaceMap);
                    return None;
                }
                KeyCode::Char('7') | KeyCode::F(7) => {
                    app.navigate(app::Screen::SelfTest);
                    return None;
                }
                KeyCode::Char('8') | KeyCode::F(8) => {
                    app.navigate(app::Screen::Help);
                    return None;
                }
                KeyCode::Char('[') => {
                    app.prev_screen();
                    return None;
                }
                KeyCode::Char(']') => {
                    app.next_screen();
                    return None;
                }
                _ => {}
            }
        }
    }

    // Screen-specific handlers
    match app.screen.clone() {
        app::Screen::Dashboard => handle_dashboard(app, code),
        app::Screen::DriveManager => handle_drives(app, code, app_arc),
        app::Screen::Entropy => handle_entropy(app, code, app_arc),
        app::Screen::WipeWizard => handle_wipe(app, code),
        app::Screen::Carver => handle_carver(app, code, app_arc),
        app::Screen::SurfaceMap => handle_surfacemap(app, code, app_arc),
        app::Screen::SelfTest => handle_selftest(app, code),
        app::Screen::Help => {}
    }

    None
}

fn handle_dashboard(app: &mut app::App, code: KeyCode) {
    match code {
        KeyCode::Enter | KeyCode::Char(' ') => {
            app.navigate(app::Screen::DriveManager);
        }
        KeyCode::Right | KeyCode::Tab => {
            app.next_screen();
        }
        KeyCode::Left | KeyCode::BackTab => {
            app.prev_screen();
        }
        _ => {}
    }
}

fn handle_selftest(app: &mut app::App, code: KeyCode) {
    match code {
        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('r') | KeyCode::Char('R') => {
            app.run_self_tests();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.test_cursor = app.test_cursor.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(ref s) = app.test_summary {
                if app.test_cursor + 1 < s.results.len() {
                    app.test_cursor += 1;
                }
            }
        }
        _ => {}
    }
}

fn handle_drives(app: &mut app::App, code: KeyCode, app_arc: Arc<Mutex<app::App>>) {
    match code {
        KeyCode::Up => {
            if app.drive_cursor > 0 {
                app.drive_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.drive_cursor + 1 < app.drives.len() {
                app.drive_cursor += 1;
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.load_drives();
            app.set_status("Drive list refreshed");
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.entropy_drive_cursor = app.drive_cursor;
            app.navigate(app::Screen::Entropy);
            let idx = app.entropy_drive_cursor;
            app.entropy_scanning = Some(idx);
            app.set_status("Analyzing entropy in background…");
            worker::start_entropy_scan(app_arc, idx, 256);
        }
        KeyCode::Enter if !app.drives.is_empty() => {
            app.selected_drive = Some(app.drive_cursor);
            let name = app.drives[app.drive_cursor].name.clone();
            app.set_status(format!("Selected /dev/{} for sanitization", name));
            app.push_log_level(
                app::LogLevel::Info,
                format!("Drive selected: /dev/{}", name),
            );
            app.navigate(app::Screen::WipeWizard);
        }
        _ => {}
    }
}

fn handle_entropy(app: &mut app::App, code: KeyCode, app_arc: Arc<Mutex<app::App>>) {
    match code {
        KeyCode::Up => {
            if app.entropy_drive_cursor > 0 {
                app.entropy_drive_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.entropy_drive_cursor + 1 < app.drives.len() {
                app.entropy_drive_cursor += 1;
            }
        }
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter => {
            let idx = app.entropy_drive_cursor;
            if idx < app.drives.len() {
                app.entropy_scanning = Some(idx);
                app.set_status("Analyzing entropy in background…");
                worker::start_entropy_scan(app_arc, idx, 256);
            }
        }
        _ => {}
    }
}

fn handle_surfacemap(app: &mut app::App, code: KeyCode, app_arc: Arc<Mutex<app::App>>) {
    match code {
        KeyCode::Tab | KeyCode::Char('m') | KeyCode::Char('M') => {
            app.surface_map.vis_mode = match app.surface_map.vis_mode {
                app::SurfaceVisMode::BadSectors => app::SurfaceVisMode::ClusterAllocation,
                app::SurfaceVisMode::ClusterAllocation => app::SurfaceVisMode::EntropySpectrogram,
                app::SurfaceVisMode::EntropySpectrogram => app::SurfaceVisMode::BadSectors,
            };
            app.set_status(format!(
                "Visualizer Mode: {}",
                app.surface_map.vis_mode.label()
            ));
        }
        KeyCode::Char('1') => {
            app.surface_map.vis_mode = app::SurfaceVisMode::BadSectors;
            app.set_status("Visualizer Mode: Bad Sector Surface Health");
        }
        KeyCode::Char('2') => {
            app.surface_map.vis_mode = app::SurfaceVisMode::ClusterAllocation;
            app.set_status("Visualizer Mode: Filesystem Cluster Allocation");
        }
        KeyCode::Char('3') => {
            app.surface_map.vis_mode = app::SurfaceVisMode::EntropySpectrogram;
            app.set_status("Visualizer Mode: Shannon Entropy Spectrogram");
        }
        KeyCode::Char('s')
        | KeyCode::Char('S')
        | KeyCode::Char(' ')
        | KeyCode::Char('r')
        | KeyCode::Char('R')
        | KeyCode::Enter => {
            let idx = app.surface_map.drive_cursor;
            let density = app.surface_map.density;
            app.set_status("Running surface sector & cluster scan in background…");
            worker::start_surface_scan(app_arc, idx, density);
        }
        KeyCode::Up => {
            if app.surface_map.drive_cursor > 0 {
                app.surface_map.drive_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.surface_map.drive_cursor + 1 < app.drives.len() {
                app.surface_map.drive_cursor += 1;
            }
        }
        KeyCode::Left => {
            app.surface_map.selected_cell = app.surface_map.selected_cell.saturating_sub(1);
        }
        KeyCode::Right => {
            let total = app
                .surface_map
                .scan_result
                .as_ref()
                .map(|r| r.cells.len())
                .unwrap_or(256);
            if app.surface_map.selected_cell + 1 < total {
                app.surface_map.selected_cell += 1;
            }
        }
        _ => {}
    }
}

fn handle_wipe(app: &mut app::App, code: KeyCode) {
    if matches!(
        app.wipe_phase,
        app::WipePhase::Running { .. } | app::WipePhase::Verifying
    ) {
        if matches!(code, KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc) {
            app.wipe_cancel
                .store(true, std::sync::atomic::Ordering::SeqCst);
            app.set_status("Aborting sanitization wipe...");
            app.push_log_level(app::LogLevel::Warning, "Cancellation signal sent to wiper");
        }
        return;
    }

    match code {
        KeyCode::Up => {
            if app.wipe_method_cursor > 0 {
                app.wipe_method_cursor -= 1;
            }
        }
        KeyCode::Down => {
            let max = app::WipeMethod::all().len() - 1;
            if app.wipe_method_cursor < max {
                app.wipe_method_cursor += 1;
            }
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.wipe_verify = !app.wipe_verify;
            app.set_status(format!(
                "Post-wipe verify: {}",
                if app.wipe_verify { "ON" } else { "OFF" }
            ));
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.wipe_expert = !app.wipe_expert;
            app.set_status(if app.wipe_expert {
                "Expert mode enabled"
            } else {
                "Expert mode disabled"
            });
        }
        KeyCode::Enter => {
            let drive_idx = match app.selected_drive {
                Some(i) if i < app.drives.len() => i,
                _ => {
                    app.selected_drive = None;
                    app.set_error("No valid drive selected — press F2, select a drive, then Enter");
                    return;
                }
            };
            let d = &app.drives[drive_idx];
            app.popup = app::Popup::Confirm {
                title: "⚠ Confirm Permanent Erasure".to_string(),
                message: format!(
                    "PERMANENTLY erase /dev/{} ({}, {}) using {}?\n\n  This will DESTROY ALL DATA and cannot be undone.",
                    d.name,
                    d.model.chars().take(24).collect::<String>(),
                    crate::ui::format_bytes(d.size_bytes),
                    app.selected_method().label(),
                ),
            };
            app.wipe_phase = app::WipePhase::Confirming;
        }
        _ => {}
    }
}

fn handle_carver(app: &mut app::App, code: KeyCode, app_arc: Arc<Mutex<app::App>>) {
    if app.carver.preview_hex.is_some()
        && matches!(
            code,
            KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Enter
        )
    {
        app.carver.preview_hex = None;
        return;
    }

    if app.carver.scanning {
        if matches!(code, KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc) {
            app.carver_cancel
                .store(true, std::sync::atomic::Ordering::SeqCst);
            app.set_status("Aborting carver scan...");
            app.push_log_level(app::LogLevel::Warning, "Cancellation signal sent to carver");
        }
        return;
    }
    match code {
        KeyCode::Tab => {
            app.carver.cursor_field = (app.carver.cursor_field + 1) % 4;
        }
        KeyCode::Up => {
            if app.carver.cursor_field == 2 {
                let n = app::RECOVERY_MODES.len();
                app.carver.recovery_mode_index = (app.carver.recovery_mode_index + 1) % n;
            } else if app.carver.cursor_field == 3 {
                if app.carver.min_confidence < 100 {
                    app.carver.min_confidence =
                        app.carver.min_confidence.saturating_add(5).min(100);
                }
            } else if !app.carver.found.is_empty() && app.carver.result_cursor > 0 {
                app.carver.result_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.carver.cursor_field == 2 {
                let n = app::RECOVERY_MODES.len();
                app.carver.recovery_mode_index = if app.carver.recovery_mode_index > 0 {
                    app.carver.recovery_mode_index - 1
                } else {
                    n - 1
                };
            } else if app.carver.cursor_field == 3 {
                if app.carver.min_confidence > 5 {
                    app.carver.min_confidence = app.carver.min_confidence.saturating_sub(5).max(5);
                }
            } else if !app.carver.found.is_empty()
                && app.carver.result_cursor + 1 < app.carver.found.len()
            {
                app.carver.result_cursor += 1;
            }
        }
        KeyCode::Left => {
            if app.carver.cursor_field == 2 {
                let n = app::RECOVERY_MODES.len();
                app.carver.recovery_mode_index = if app.carver.recovery_mode_index > 0 {
                    app.carver.recovery_mode_index - 1
                } else {
                    n - 1
                };
            }
        }
        KeyCode::Right => {
            if app.carver.cursor_field == 2 {
                let n = app::RECOVERY_MODES.len();
                app.carver.recovery_mode_index = (app.carver.recovery_mode_index + 1) % n;
            }
        }
        KeyCode::Char('i') | KeyCode::Char('I') if app.carver.cursor_field >= 2 => {
            app.carver.inspect_only = !app.carver.inspect_only;
            app.set_status(format!(
                "Carver inspect mode: {}",
                if app.carver.inspect_only {
                    "ON (Dry-Run / In-Memory Preview)"
                } else {
                    "OFF (Direct Extraction to Disk)"
                }
            ));
        }
        KeyCode::Char(' ') => {
            if app.carver.cursor_field == 2 {
                let n = app::RECOVERY_MODES.len();
                app.carver.recovery_mode_index = (app.carver.recovery_mode_index + 1) % n;
            } else if app.carver.cursor_field == 0 {
                app.carver.source.push(' ');
            } else if app.carver.cursor_field == 1 {
                app.carver.output_dir.push(' ');
            } else if !app.carver.found.is_empty() {
                if let Some(f) = app.carver.found.get_mut(app.carver.result_cursor) {
                    f.selected = !f.selected;
                }
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') if app.carver.cursor_field >= 2 => {
            let all_sel = app.carver.found.iter().all(|f| f.selected);
            for f in &mut app.carver.found {
                f.selected = !all_sel;
            }
            app.set_status(format!(
                "Artifacts selected: {}",
                if all_sel { 0 } else { app.carver.found.len() }
            ));
        }
        KeyCode::Char('p') | KeyCode::Char('P') if app.carver.cursor_field >= 2 => {
            if let Some(f) = app.carver.found.get(app.carver.result_cursor) {
                let mut buf = vec![0u8; 256.min(f.size_bytes.max(16) as usize)];
                if let Ok(mut src_file) = std::fs::File::open(&app.carver.source) {
                    use std::io::{Read, Seek, SeekFrom};
                    let _ = src_file.seek(SeekFrom::Start(f.offset_bytes));
                    let read_n = src_file.read(&mut buf).unwrap_or(0);
                    buf.truncate(read_n);
                }
                let hex_lines: Vec<String> = buf
                    .chunks(16)
                    .enumerate()
                    .map(|(row, chunk)| {
                        let hex_part = chunk
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        let ascii_part = chunk
                            .iter()
                            .map(|&b| {
                                if b.is_ascii_graphic() || b == b' ' {
                                    b as char
                                } else {
                                    '.'
                                }
                            })
                            .collect::<String>();
                        format!("{:04X}  {:<48}  |{}|", row * 16, hex_part, ascii_part)
                    })
                    .collect();
                app.carver.preview_hex = Some(format!(
                    "Artifact: {} | Offset: 0x{:X} | Size: {} B\n\n{}",
                    f.path,
                    f.offset_bytes,
                    f.size_bytes,
                    hex_lines.join("\n")
                ));
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') | KeyCode::Char('e') | KeyCode::Char('E')
            if app.carver.cursor_field >= 2 =>
        {
            let mut indices: Vec<usize> = app
                .carver
                .found
                .iter()
                .enumerate()
                .filter_map(|(i, f)| if f.selected { Some(i) } else { None })
                .collect();
            if indices.is_empty() && !app.carver.found.is_empty() {
                indices.push(app.carver.result_cursor);
            }
            if !indices.is_empty() {
                let src = app.carver.source.clone();
                let out = app.carver.output_dir.clone();
                app.set_status(format!("Extracting {} selected file(s)...", indices.len()));
                worker::extract_selected_carver_files(app_arc, src, out, indices);
            }
        }
        KeyCode::Char(c) => match app.carver.cursor_field {
            0 => app.carver.source.push(c),
            1 => app.carver.output_dir.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app.carver.cursor_field {
            0 => {
                app.carver.source.pop();
            }
            1 => {
                app.carver.output_dir.pop();
            }
            _ => {}
        },
        KeyCode::Enter => {
            if app.carver.source.is_empty() {
                app.set_error("Source path is empty — type a path or /dev/sdX");
                return;
            }
            app.carver.scanning = true;
            app.carver.found.clear();
            app.carver.log.clear();
            app.carver.progress = 0.0;
            app.carver.result_cursor = 0;
            app.carver.preview_hex = None;
            let src = app.carver.source.clone();
            let out = app.carver.output_dir.clone();
            let conf = app.carver.min_confidence;
            let mode_idx = app.carver.recovery_mode_index;
            let (mode_name, _) = app::RECOVERY_MODES[mode_idx % app::RECOVERY_MODES.len()];
            let inspect_label = if app.carver.inspect_only {
                "inspect"
            } else {
                "recover"
            };
            app.carver.log.push(format!(
                "Starting {} of {} [{}]",
                inspect_label, src, mode_name
            ));
            app.push_log_level(
                app::LogLevel::Info,
                format!("Carver {} started: {} [{}]", inspect_label, src, mode_name),
            );
            app.set_status(format!("Scanning {} ({})", src, mode_name));
            worker::start_carver_scan(app_arc, src, out, conf, mode_idx);
        }
        _ => {}
    }
}
