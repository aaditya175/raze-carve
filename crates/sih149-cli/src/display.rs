use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// Cyberpunk / Tokyo-Night truecolor ANSI codes
const CLR_RESET: &str = "\x1b[0m";
const CLR_BOLD: &str = "\x1b[1m";
const CLR_DIM: &str = "\x1b[2m";

const CLR_CYAN: &str = "\x1b[38;2;56;189;248m";
const CLR_VIOLET: &str = "\x1b[38;2;168;85;247m";
const CLR_MINT: &str = "\x1b[38;2;52;211;153m";
const CLR_AMBER: &str = "\x1b[38;2;251;191;36m";
const CLR_ROSE: &str = "\x1b[38;2;248;113;113m";
const CLR_SLATE: &str = "\x1b[38;2;148;163;184m";
const CLR_BORDER: &str = "\x1b[38;2;60;66;96m";

pub fn print_banner() {
    println!("{CLR_BORDER}┌────────────────────────────────────────────────────────────────────────┐{CLR_RESET}");
    println!("{CLR_BORDER}│{CLR_RESET}  {CLR_BOLD}{CLR_CYAN}🔒 SECUREFORGE{CLR_RESET} {CLR_VIOLET}:: FORENSIC SANITIZATION & RECOVERY ENGINE{CLR_RESET}        {CLR_BORDER}│{CLR_RESET}");
    println!("{CLR_BORDER}│{CLR_RESET}  {CLR_DIM}v0.1.0  ●  NIST SP 800-88 Rev 1  ●  DoD 5220.22-M  ●  RFC 3161 Certified{CLR_RESET}{CLR_BORDER}│{CLR_RESET}");
    println!("{CLR_BORDER}└────────────────────────────────────────────────────────────────────────┘{CLR_RESET}");
}

pub fn print_section(title: &str) {
    println!("\n{CLR_CYAN}▸ {CLR_BOLD}{title}{CLR_RESET}");
    println!("{CLR_BORDER}{}{CLR_RESET}", "─".repeat(title.len() + 4));
}

pub fn print_step(current: usize, total: usize, name: &str) {
    println!("{CLR_VIOLET}[{current}/{total}]{CLR_RESET} {CLR_BOLD}{name}{CLR_RESET}");
}

pub fn create_spinner(message: &'static str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .expect("valid template"),
    );
    pb.set_message(message);
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn print_drive_table(drives: &[serde_json::Value]) {
    println!("{CLR_BORDER}┌────────────────────┬─────────────────┬──────────┬────────────────────────┐{CLR_RESET}");
    println!("{CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<18}{CLR_RESET} {CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<15}{CLR_RESET} {CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<8}{CLR_RESET} {CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<22}{CLR_RESET} {CLR_BORDER}│{CLR_RESET}", "NAME", "CAPACITY", "TYPE", "MOUNTPOINT");
    println!("{CLR_BORDER}├────────────────────┼─────────────────┼──────────┼────────────────────────┤{CLR_RESET}");
    for d in drives {
        let name = d["name"].as_str().unwrap_or("Unknown");
        let size = d["size"].as_u64().unwrap_or(0);
        let typ = d["type"].as_str().unwrap_or("disk");
        let tran = d["tran"].as_str().unwrap_or("");
        let rota = d["rota"].as_bool().unwrap_or(true);
        let rm = d["rm"].as_bool().unwrap_or(false) || d["rm"].as_u64() == Some(1);
        let mount = d["mountpoint"].as_str().unwrap_or(
            d["children"]
                .as_array()
                .and_then(|c| c.first())
                .and_then(|p| p["mountpoint"].as_str())
                .unwrap_or("unmounted"),
        );

        let (badge_text, color) = if typ == "part" {
            ("[PART]", CLR_SLATE)
        } else if tran.eq_ignore_ascii_case("nvme") || name.starts_with("nvme") {
            ("[NVMe]", CLR_CYAN)
        } else if tran.eq_ignore_ascii_case("usb") || rm {
            ("[USB]", CLR_AMBER)
        } else if typ == "loop" || name.starts_with("loop") {
            ("[LOOP]", CLR_VIOLET)
        } else if name.starts_with("zram") {
            ("[ZRAM]", CLR_SLATE)
        } else if !rota {
            ("[SSD]", CLR_MINT)
        } else {
            ("[HDD]", CLR_SLATE)
        };
        let typ_badge = format!(
            "{}{}{}{}",
            color,
            badge_text,
            CLR_RESET,
            " ".repeat(8usize.saturating_sub(badge_text.len()))
        );

        println!(
            "{CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{:<18}{CLR_RESET} {CLR_BORDER}│{CLR_RESET} {:<15} {CLR_BORDER}│{CLR_RESET} {} {CLR_BORDER}│{CLR_RESET} {:<22} {CLR_BORDER}│{CLR_RESET}",
            name, format_bytes(size), typ_badge, mount
        );
    }
    println!("{CLR_BORDER}└────────────────────┴─────────────────┴──────────┴────────────────────────┘{CLR_RESET}");
}

pub fn print_case_table(cases: &[sih149_core::db::CaseRecord]) {
    println!("{CLR_BORDER}┌────────────────────────┬─────────────────────┬──────────────┬──────────────────────┬─────────────┐{CLR_RESET}");
    println!("{CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<22}{CLR_RESET} {CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<19}{CLR_RESET} {CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<12}{CLR_RESET} {CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<20}{CLR_RESET} {CLR_BORDER}│{CLR_RESET} {CLR_BOLD}{CLR_CYAN}{:<11}{CLR_RESET} {CLR_BORDER}│{CLR_RESET}", "CASE ID", "CREATED", "OPERATION", "TARGET", "STATUS");
    println!("{CLR_BORDER}├────────────────────────┼─────────────────────┼──────────────┼──────────────────────┼─────────────┤{CLR_RESET}");
    for c in cases {
        let created = if c.created_at.len() >= 19 {
            &c.created_at[..19]
        } else {
            &c.created_at
        };
        let (st_str, col) = match c.status.to_lowercase().as_str() {
            "completed" | "pass" => ("DONE", CLR_MINT),
            "running" | "active" => ("RUNNING", CLR_CYAN),
            _ => ("PENDING", CLR_AMBER),
        };
        let status_badge = format!(
            "{}{}{}{}",
            col,
            st_str,
            CLR_RESET,
            " ".repeat(11usize.saturating_sub(st_str.len()))
        );
        println!(
            "{CLR_BORDER}│{CLR_RESET} {:<22} {CLR_BORDER}│{CLR_RESET} {:<19} {CLR_BORDER}│{CLR_RESET} {:<12} {CLR_BORDER}│{CLR_RESET} {:<20} {CLR_BORDER}│{CLR_RESET} {} {CLR_BORDER}│{CLR_RESET}",
            c.id, created, c.operation, c.target, status_badge
        );
    }
    println!("{CLR_BORDER}└────────────────────────┴─────────────────────┴──────────────┴──────────────────────┴─────────────┘{CLR_RESET}");
}

pub fn print_success(msg: &str) {
    println!("{CLR_MINT}{CLR_BOLD}✔ [PASS]{CLR_RESET} {msg}");
}

pub fn print_error(msg: &str) {
    eprintln!("{CLR_ROSE}{CLR_BOLD}✖ [FAIL]{CLR_RESET} {msg}");
}

pub fn print_warn(msg: &str) {
    println!("{CLR_AMBER}{CLR_BOLD}▲ [WARN]{CLR_RESET} {msg}");
}

pub fn print_info(msg: &str) {
    println!("{CLR_CYAN}{CLR_BOLD}ℹ [INFO]{CLR_RESET} {msg}");
}

pub fn format_bytes(n: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = n as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, units[unit_idx])
}

pub fn format_duration(secs: u64) -> String {
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
