use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    Dashboard,
    DriveManager,
    Entropy,
    WipeWizard,
    Carver,
    SurfaceMap,
    SelfTest,
    Help,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WipeMethod {
    Zero,
    Random,
    Dod3,
    Dod7,
    Nist,
    Gutmann,
}

impl WipeMethod {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Zero => "Zero Fill        (1-pass, fast)",
            Self::Random => "Pseudo-Random    (1-pass PRNG)",
            Self::Dod3 => "DoD 5220.22-M    (3-pass)",
            Self::Dod7 => "DoD 5220.22-M    (7-pass)",
            Self::Nist => "NIST SP 800-88   (Clear)",
            Self::Gutmann => "Gutmann          (35-pass, max)",
        }
    }
    pub fn passes(&self) -> usize {
        match self {
            Self::Zero | Self::Random | Self::Nist => 1,
            Self::Dod3 => 3,
            Self::Dod7 => 7,
            Self::Gutmann => 35,
        }
    }
    pub fn risk_label(&self) -> (&'static str, u8) {
        match self {
            Self::Zero => ("Low", 1),
            Self::Random => ("Low", 2),
            Self::Dod3 => ("Standard", 3),
            Self::Dod7 => ("High", 4),
            Self::Nist => ("Standard", 3),
            Self::Gutmann => ("Maximum", 5),
        }
    }
    pub fn all() -> Vec<WipeMethod> {
        vec![
            Self::Zero,
            Self::Random,
            Self::Dod3,
            Self::Dod7,
            Self::Nist,
            Self::Gutmann,
        ]
    }
}

/// Pre-computed entropy statistics — computed once when scan finishes, never on every frame.
#[derive(Clone, Debug, Default)]
pub struct EntropyStats {
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    /// Bucket counts for 6 entropy bands: [0-1, 1-2, 2-4, 4-6, 6-7.5, 7.5-8]
    pub buckets: [usize; 6],
}

impl EntropyStats {
    pub fn compute(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        let n = samples.len() as f64;
        let avg = samples.iter().sum::<f64>() / n;
        let (min, max) = samples
            .iter()
            .fold((f64::MAX, f64::MIN), |(mn, mx), &e| (mn.min(e), mx.max(e)));
        let variance = samples.iter().map(|&e| (e - avg).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        let mut buckets = [0usize; 6];
        for &e in samples {
            let i = if e < 1.0 {
                0
            } else if e < 2.0 {
                1
            } else if e < 4.0 {
                2
            } else if e < 6.0 {
                3
            } else if e < 7.5 {
                4
            } else {
                5
            };
            buckets[i] += 1;
        }
        Self { avg, min, max, std_dev, buckets }
    }
}

#[derive(Clone, Debug)]
pub struct DriveEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub model: String,
    pub is_system: bool,
    pub drive_type: String,
    /// Entropy samples per 64 KB sector (0.0–8.0 bits/byte)
    pub entropy_samples: Vec<f64>,
    pub entropy_loaded: bool,
    /// Pre-computed stats — zero-cost to read in render path
    pub entropy_stats: EntropyStats,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WipePhase {
    Idle,
    Confirming,
    Running {
        pass: usize,
        total_passes: usize,
        bytes_done: u64,
        bytes_total: u64,
        speed_mbps: f64,
        started: Instant,
    },
    Verifying,
    Done {
        success: bool,
        elapsed: Duration,
    },
    Error(String),
}

pub const RECOVERY_MODES: &[(&str, &str)] = &[
    (
        "Hybrid Ensemble (All-in-One)",
        "Combines MFT/FAT metadata, syntax validation & entropy clustering",
    ),
    (
        "Deep Structural Syntax",
        "Parses container trees (ZIP EOCD, SQLite B-Tree, PNG/JPEG markers)",
    ),
    (
        "Filesystem Inode/MFT",
        "Extracts deleted NTFS $MFT and FAT records (recovers original names & dates)",
    ),
    (
        "Statistical Entropy Stream",
        "Classifies Shannon entropy blocks for fragmented or headerless files",
    ),
    (
        "Signature Magic Headers",
        "Fast sector-aligned magic byte header and footer delimiter scanning",
    ),
];

#[derive(Clone, Debug)]
pub struct CarvedFile {
    pub path: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub confidence: u8,
    pub entropy: f64,
    pub method: String,
    pub metadata: String,
    pub offset_bytes: u64,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct CarverState {
    pub source: String,
    pub output_dir: String,
    pub min_confidence: u8,
    pub recovery_mode_index: usize,
    pub scanning: bool,
    pub progress: f64,
    pub found: Vec<CarvedFile>,
    pub log: Vec<String>,
    pub cursor_field: usize,
    pub result_cursor: usize,
    pub inspect_only: bool,
    pub preview_hex: Option<String>,
}

impl Default for CarverState {
    fn default() -> Self {
        Self {
            source: String::new(),
            output_dir: dirs::home_dir()
                .map(|h| h.join("recovered").to_string_lossy().to_string())
                .unwrap_or_default(),
            min_confidence: 70,
            recovery_mode_index: 0,
            scanning: false,
            progress: 0.0,
            found: Vec::new(),
            log: Vec::new(),
            cursor_field: 0,
            result_cursor: 0,
            inspect_only: false,
            preview_hex: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceVisMode {
    BadSectors,
    ClusterAllocation,
    EntropySpectrogram,
}

impl SurfaceVisMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BadSectors => "Bad Sector Surface Health",
            Self::ClusterAllocation => "Filesystem Cluster Allocation",
            Self::EntropySpectrogram => "Shannon Entropy Spectrogram",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceMapState {
    pub drive_cursor: usize,
    pub vis_mode: SurfaceVisMode,
    pub scan_result: Option<sih149_core::disk::surface_scanner::SurfaceScanResult>,
    pub scanning: bool,
    pub selected_cell: usize,
    pub density: usize,
}

impl Default for SurfaceMapState {
    fn default() -> Self {
        Self {
            drive_cursor: 0,
            vis_mode: SurfaceVisMode::BadSectors,
            scan_result: None,
            scanning: false,
            selected_cell: 0,
            density: 256,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Popup {
    None,
    Confirm { title: String, message: String },
    Error(String),
    Info(String),
}

pub struct App {
    pub screen: Screen,
    pub prev_screen: Screen,
    pub drives: Vec<DriveEntry>,
    pub drive_cursor: usize,
    pub selected_drive: Option<usize>,
    pub wipe_method_cursor: usize,
    pub wipe_verify: bool,
    pub wipe_expert: bool,
    pub wipe_phase: WipePhase,
    pub carver: CarverState,
    pub surface_map: SurfaceMapState,
    pub popup: Popup,
    pub log: VecDeque<(LogLevel, String)>,
    #[allow(dead_code)]
    pub last_refresh: Instant,
    pub tick: u64,
    pub should_quit: bool,
    pub status_msg: Option<(String, Instant, bool)>, // (msg, time, is_err)
    pub entropy_drive_cursor: usize,
    pub test_summary: Option<sih149_core::diagnostics::DiagnosticSummary>,
    pub test_running: bool,
    pub test_cursor: usize,
    pub wipe_cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub carver_cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub entropy_scanning: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            screen: Screen::Dashboard,
            prev_screen: Screen::Dashboard,
            drives: Vec::new(),
            drive_cursor: 0,
            selected_drive: None,
            wipe_method_cursor: 2,
            wipe_verify: true,
            wipe_expert: false,
            wipe_phase: WipePhase::Idle,
            carver: CarverState::default(),
            surface_map: SurfaceMapState::default(),
            popup: Popup::None,
            log: VecDeque::new(),
            last_refresh: Instant::now(),
            tick: 0,
            should_quit: false,
            status_msg: None,
            entropy_drive_cursor: 0,
            test_summary: None,
            test_running: false,
            test_cursor: 0,
            wipe_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            carver_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            entropy_scanning: None,
        };
        app.run_self_tests();
        app
    }

    pub fn run_self_tests(&mut self) {
        self.test_running = true;
        let summary = sih149_core::diagnostics::run_diagnostics();
        let passed = summary.passed;
        let total = summary.total;
        self.test_summary = Some(summary);
        self.test_running = false;
        self.push_log_level(
            LogLevel::Success,
            format!(
                "Diagnostic self-tests complete: {}/{} passed",
                passed, total
            ),
        );
        self.set_status(format!("Diagnostics: {}/{} passed", passed, total));
    }

    pub fn push_log_level(&mut self, level: LogLevel, msg: impl Into<String>) {
        self.log.push_back((level, msg.into()));
        if self.log.len() > 500 {
            self.log.pop_front();
        }
    }

    pub fn push_log(&mut self, msg: impl Into<String>) {
        self.push_log_level(LogLevel::Info, msg);
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some((msg.into(), Instant::now(), false));
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.status_msg = Some((msg.into(), Instant::now(), true));
    }

    pub fn navigate(&mut self, s: Screen) {
        self.prev_screen = self.screen.clone();
        self.screen = s;
    }

    pub fn next_screen(&mut self) {
        let next = match self.screen {
            Screen::Dashboard => Screen::DriveManager,
            Screen::DriveManager => Screen::Entropy,
            Screen::Entropy => Screen::WipeWizard,
            Screen::WipeWizard => Screen::Carver,
            Screen::Carver => Screen::SurfaceMap,
            Screen::SurfaceMap => Screen::SelfTest,
            Screen::SelfTest => Screen::Help,
            Screen::Help => Screen::Dashboard,
        };
        self.navigate(next);
    }

    pub fn prev_screen(&mut self) {
        let prev = match self.screen {
            Screen::Dashboard => Screen::Help,
            Screen::DriveManager => Screen::Dashboard,
            Screen::Entropy => Screen::DriveManager,
            Screen::WipeWizard => Screen::Entropy,
            Screen::Carver => Screen::WipeWizard,
            Screen::SurfaceMap => Screen::Carver,
            Screen::SelfTest => Screen::SurfaceMap,
            Screen::Help => Screen::SelfTest,
        };
        self.navigate(prev);
    }

    pub fn selected_method(&self) -> WipeMethod {
        WipeMethod::all()
            .into_iter()
            .nth(self.wipe_method_cursor)
            .unwrap_or(WipeMethod::Dod3)
    }

    pub fn query_current_drives() -> Vec<DriveEntry> {
        let mut list = Vec::new();
        if let Ok(output) = std::process::Command::new("lsblk")
            .args(["-J", "-b", "-o", "NAME,SIZE,TYPE,MOUNTPOINT,MODEL,ROTA"])
            .output()
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(bds) = json["blockdevices"].as_array() {
                        for bd in bds {
                            let is_disk = bd["type"].as_str() == Some("disk");
                            let is_loop = bd["type"].as_str() == Some("loop") || bd["name"].as_str().unwrap_or("").starts_with("loop");
                            if !is_disk && !is_loop {
                                continue;
                            }
                            let name = bd["name"].as_str().unwrap_or("").to_string();
                            let path = format!("/dev/{}", name);
                            let size = bd["size"]
                                .as_u64()
                                .or_else(|| bd["size"].as_str().and_then(|s| s.parse().ok()))
                                .unwrap_or(0);
                            if is_loop && size == 0 {
                                continue;
                            }
                            let mut model =
                                bd["model"].as_str().unwrap_or("Unknown").trim().to_string();
                            if model == "Unknown" && is_loop {
                                model = "Virtual Loop Disk".to_string();
                            }
                            let rota = bd["rota"].as_bool().unwrap_or(true);
                            let is_nvme = name.starts_with("nvme");
                            let dtype = if is_loop {
                                "LOOP"
                            } else if name.starts_with("zram") {
                                "ZRAM"
                            } else if is_nvme {
                                "NVMe"
                            } else if !rota {
                                "SSD"
                            } else {
                                "HDD"
                            }
                            .to_string();
                            let is_system = sih149_core::wiper::file_wiper::is_protected_drive(
                                std::path::Path::new(&path),
                            );
                            list.push(DriveEntry {
                                name,
                                path,
                                size_bytes: size,
                                model,
                                is_system,
                                drive_type: dtype,
                                entropy_samples: Vec::new(),
                                entropy_loaded: false,
                                entropy_stats: EntropyStats::default(),
                            });
                        }
                    }
                }
            }
        }
        if list.is_empty() {
            for prefix in &["sda", "sdb", "sdc", "nvme0n1", "vda"] {
                let path = format!("/dev/{}", prefix);
                if std::path::Path::new(&path).exists() {
                    let is_system = sih149_core::wiper::file_wiper::is_protected_drive(
                        std::path::Path::new(&path),
                    );
                    list.push(DriveEntry {
                        name: prefix.to_string(),
                        path,
                        size_bytes: 0,
                        model: "Unknown".to_string(),
                        is_system,
                        drive_type: "Disk".to_string(),
                        entropy_samples: Vec::new(),
                        entropy_loaded: false,
                        entropy_stats: EntropyStats::default(),
                    });
                }
            }
        }
        list
    }

    pub fn load_drives(&mut self) {
        self.drives = Self::query_current_drives();
        self.push_log_level(
            LogLevel::Success,
            format!("Detected {} drive(s)", self.drives.len()),
        );
    }

    pub fn apply_drive_changes(&mut self, current: Vec<DriveEntry>) {
        // 1. Check for removed drives
        let removed: Vec<String> = self
            .drives
            .iter()
            .filter(|d| !current.iter().any(|c| c.name == d.name))
            .map(|d| d.name.clone())
            .collect();

        // 2. Check for added drives
        let added: Vec<DriveEntry> = current
            .into_iter()
            .filter(|c| !self.drives.iter().any(|d| d.name == c.name))
            .collect();

        for r_name in removed {
            self.push_log_level(
                LogLevel::Warning,
                format!("⚡ Drive DISCONNECTED: /dev/{}", r_name),
            );
            self.set_status(format!("Drive unplugged: /dev/{}", r_name));
            self.drives.retain(|d| d.name != r_name);
            if self.drive_cursor >= self.drives.len() && !self.drives.is_empty() {
                self.drive_cursor = self.drives.len() - 1;
            }
            if self.entropy_drive_cursor >= self.drives.len() && !self.drives.is_empty() {
                self.entropy_drive_cursor = self.drives.len() - 1;
            }
        }

        for new_drive in added {
            self.push_log_level(
                LogLevel::Success,
                format!(
                    "⚡ Drive CONNECTED: /dev/{} ({}, {})",
                    new_drive.name, new_drive.model, new_drive.drive_type
                ),
            );
            self.set_status(format!("New drive detected: /dev/{}", new_drive.name));
            self.drives.push(new_drive);
        }
    }

    /// Real-time polling check for hotplug insertion or removal of drives
    #[allow(dead_code)]
    pub fn poll_drive_changes(&mut self) {
        let current = Self::query_current_drives();
        self.apply_drive_changes(current);
    }
}

/// Shannon entropy in bits/byte (0.0–8.0)
pub fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f64;
    freq.iter().filter(|&&c| c > 0).fold(0.0, |acc, &c| {
        let p = c as f64 / len;
        acc - p * p.log2()
    })
}

/// Map entropy value to a descriptive label
pub fn entropy_label(e: f64) -> (&'static str, u8) {
    match e as u32 {
        0 => ("Dead / Zeroed", 0),
        1 => ("Near-Zero", 1),
        2..=3 => ("Low", 2),
        4..=5 => ("Moderate", 3),
        6 => ("High", 4),
        _ => ("Encrypted/Compressed", 5),
    }
}
