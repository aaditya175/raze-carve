//! Firmware-level secure erase commands.
//! Wraps `hdparm` (ATA) and `nvme-cli` (NVMe) via subprocess.
//!
//! WARNING: every function here that performs an erase is genuinely
//! destructive and irreversible against real hardware. Requires root
//! (CAP_SYS_ADMIN) and the `hdparm`/`nvme-cli` packages installed.
use crate::error::CoreError;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareMethod {
    NvmeCryptoErase,
    NvmeBlockErase,
    AtaSecureErase,
    AtaEnhancedErase,
}

impl FirmwareMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirmwareMethod::NvmeCryptoErase => "nvme-crypto",
            FirmwareMethod::NvmeBlockErase => "nvme-block",
            FirmwareMethod::AtaSecureErase => "ata",
            FirmwareMethod::AtaEnhancedErase => "ata-enhanced",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "nvme-crypto" => Some(FirmwareMethod::NvmeCryptoErase),
            "nvme-block" => Some(FirmwareMethod::NvmeBlockErase),
            "ata" => Some(FirmwareMethod::AtaSecureErase),
            "ata-enhanced" => Some(FirmwareMethod::AtaEnhancedErase),
            _ => None,
        }
    }
}

impl std::fmt::Display for FirmwareMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for FirmwareMethod {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| {
            CoreError::Wiper(format!(
                "Unknown firmware erase method '{}'. Valid: nvme-crypto, nvme-block, ata, ata-enhanced",
                s
            ))
        })
    }
}

#[derive(Debug)]
pub struct FirmwareEraseResult {
    pub method: FirmwareMethod,
    pub success: bool,
    pub command_output: String,
    pub duration_secs: u64,
    pub password_used: Option<String>,
    pub diagnostic: Option<String>,
}

#[derive(Debug)]
pub struct NvmeCapabilities {
    pub sanitize_supported: bool,
}

#[derive(Debug)]
pub struct HpaInfo {
    pub hpa_enabled: bool,
    pub dco_enabled: bool,
}

/// Validates that a device path is a safe block-device target for
/// firmware-level commands. Only accepts canonical absolute paths under
/// /dev with no single-dot/double-dot traversal, no shell metacharacters,
/// and requires the target to actually exist as a block device.
pub fn is_valid_block_device_path(device: &Path) -> bool {
    let s = device.to_string_lossy();
    // Must be an absolute /dev path, not a relative/arbitrary path.
    if !s.starts_with("/dev/") {
        return false;
    }
    // Refuse traversal and shell-unsafe characters outright.
    if s.contains("..") || s.contains('\0') {
        return false;
    }
    if s.chars().any(|c| {
        matches!(
            c,
            ' ' | ';' | '&' | '|' | '`' | '$' | '(' | ')' | '<' | '>' | '\'' | '"' | '\\'
        )
    }) {
        return false;
    }
    // Follow symlinks (like /dev/disk/by-id/..., /dev/disk/by-path/...) to verify canonical target is in /dev/
    let canonical = match std::fs::canonicalize(device) {
        Ok(c) => c,
        Err(_) => return false,
    };
    if !canonical.to_string_lossy().starts_with("/dev/") {
        return false;
    }

    // Must actually be a block device node (not a regular file, not a directory).
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        match std::fs::metadata(&canonical) {
            Ok(md) => md.file_type().is_block_device(),
            Err(_) => false,
        }
    }
    #[cfg(not(unix))]
    {
        std::fs::metadata(&canonical).is_ok()
    }
}

/// Returns true if the block device or any of its partitions are currently mounted.
pub fn is_device_mounted(device: &Path) -> bool {
    let canonical = match std::fs::canonicalize(device) {
        Ok(c) => c,
        Err(_) => device.to_path_buf(),
    };
    let dev_str = canonical.to_string_lossy();

    #[cfg(target_os = "linux")]
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let mut parts = line.split_whitespace();
            if let Some(mounted_dev) = parts.next() {
                if mounted_dev == dev_str || mounted_dev.starts_with(&*dev_str) {
                    return true;
                }
            }
        }
    }
    false
}

/// Decodes controller/driver status messages from `nvme` and `hdparm` outputs into
/// structured actionable diagnostics for forensic examiners.
pub fn decode_firmware_error(method: FirmwareMethod, output: &str) -> Option<String> {
    let lower = output.to_lowercase();
    match method {
        FirmwareMethod::NvmeCryptoErase | FirmwareMethod::NvmeBlockErase => {
            if lower.contains("invalid command opcode") || lower.contains("opcode(0x1)") {
                Some("Controller reports Invalid Command Opcode (0x1): NVMe Sanitize is not supported by drive firmware".to_string())
            } else if lower.contains("sanitize in progress")
                || lower.contains("status: 0x1b")
                || lower.contains("0x1b")
            {
                Some("Sanitize In Progress (0x1B): Another sanitize operation is currently active on the drive".to_string())
            } else if lower.contains("persistent memory region") || lower.contains("0x23") {
                Some("Sanitize Prohibited (0x23): Persistent Memory Region (PMR) is currently enabled on the device".to_string())
            } else if lower.contains("device or resource busy") || lower.contains("ebusy") {
                Some("Device or resource busy: Drive partitions are mounted or opened with exclusive locks".to_string())
            } else if lower.contains("permission denied") || lower.contains("eacces") {
                Some("Permission denied: Firmware sanitization requires root privileges (CAP_SYS_ADMIN)".to_string())
            } else {
                None
            }
        }
        FirmwareMethod::AtaSecureErase | FirmwareMethod::AtaEnhancedErase => {
            if lower.contains("frozen") {
                Some("ATA Security is FROZEN: System firmware locked security feature set at boot; power-cycle required".to_string())
            } else if lower.contains("locked") {
                Some("ATA Security is LOCKED: Drive already has a password set; security-disable or unlock required".to_string())
            } else if lower.contains("not supported") {
                Some("ATA Security feature set is not supported by this storage controller or bridge".to_string())
            } else if lower.contains("busy") {
                Some(
                    "Drive is busy or in low-power standby; wake the drive before issuing erase"
                        .to_string(),
                )
            } else {
                None
            }
        }
    }
}

fn valid_block_device(device: &Path) -> bool {
    is_valid_block_device_path(device)
}

fn run_command(cmd: &str, device: &Path, args: &[&str]) -> Result<(bool, String), CoreError> {
    if !valid_block_device(device) {
        return Err(CoreError::Wiper(format!(
            "Refusing to run '{}' on unsafe target: {} (must be a valid /dev block device path)",
            cmd,
            device.display()
        )));
    }
    if crate::wiper::file_wiper::is_protected_drive(device) {
        return Err(CoreError::Wiper(format!(
            "Refusing firmware erase on protected/system drive: {}",
            device.display()
        )));
    }
    if is_device_mounted(device) {
        return Err(CoreError::Wiper(format!(
            "Device or partition on {} is currently mounted. Unmount all partitions before firmware erase.",
            device.display()
        )));
    }
    let canonical = std::fs::canonicalize(device).unwrap_or_else(|_| device.to_path_buf());
    let device_str = canonical.to_string_lossy();
    let mut full_args: Vec<&str> = Vec::with_capacity(args.len() + 1);
    full_args.extend_from_slice(args);
    full_args.push(&device_str);

    let output = Command::new(cmd).args(&full_args).output().map_err(|e| {
        CoreError::Wiper(format!(
            "Failed to run {} {:?}: {} (is {} installed and are you running as root?)",
            cmd, full_args, e, cmd
        ))
    })?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok((output.status.success(), combined))
}

/// Checks the NVMe Sanitize log page for crypto-erase support.
/// Parses `nvme id-ctrl` SANICAP bits via `nvme-cli`'s human-readable
/// output — brittle against nvme-cli version differences; treat a `false`
/// result conservatively (may be a parse miss, not definitive absence).
pub fn detect_nvme_capabilities(device: &Path) -> Result<NvmeCapabilities, CoreError> {
    let (ok, output) = run_command("nvme", device, &["id-ctrl", "-H"])?;
    if !ok {
        return Err(CoreError::Wiper(format!("nvme id-ctrl failed: {}", output)));
    }
    // Human-readable id-ctrl output includes a line like:
    // "[0:0] : 0x1  Crypto Erase Supported"
    let sanitize_supported = output.to_lowercase().contains("crypto erase supported");
    Ok(NvmeCapabilities { sanitize_supported })
}

/// NVMe Sanitize with Crypto Erase (sanact=2). Cryptographically destroys
/// the media encryption key — data becomes unrecoverable even without
/// overwriting every block, but only if the drive actually implements
/// full-disk encryption at rest (verify via detect_nvme_capabilities first).
pub fn nvme_crypto_erase(device: &Path) -> Result<FirmwareEraseResult, CoreError> {
    let started = Instant::now();
    let (success, output) = run_command("nvme", device, &["sanitize", "--sanact=2"])?;
    let diagnostic = if !success {
        decode_firmware_error(FirmwareMethod::NvmeCryptoErase, &output)
    } else {
        None
    };
    Ok(FirmwareEraseResult {
        method: FirmwareMethod::NvmeCryptoErase,
        success,
        command_output: output,
        duration_secs: started.elapsed().as_secs(),
        password_used: None,
        diagnostic,
    })
}

/// NVMe Sanitize with Block Erase (sanact=1). Physically erases every
/// block at the flash-translation-layer level. Slower than crypto erase
/// but doesn't depend on the drive's encryption implementation.
pub fn nvme_block_erase(device: &Path) -> Result<FirmwareEraseResult, CoreError> {
    let started = Instant::now();
    let (success, output) = run_command("nvme", device, &["sanitize", "--sanact=1"])?;
    let diagnostic = if !success {
        decode_firmware_error(FirmwareMethod::NvmeBlockErase, &output)
    } else {
        None
    };
    Ok(FirmwareEraseResult {
        method: FirmwareMethod::NvmeBlockErase,
        success,
        command_output: output,
        duration_secs: started.elapsed().as_secs(),
        password_used: None,
        diagnostic,
    })
}

/// Checks whether the ATA drive is in a "frozen" security state — BIOS/
/// firmware sometimes freezes the security feature set at boot, which
/// blocks secure-erase commands until the drive is power-cycled (not just
/// rebooted) with no intervening freeze command from the OS.
pub fn ata_detect_frozen(device: &Path) -> Result<bool, CoreError> {
    let (ok, output) = run_command("hdparm", device, &["-I"])?;
    if !ok {
        return Err(CoreError::Wiper(format!("hdparm -I failed: {}", output)));
    }
    Ok(output.contains("frozen"))
}

/// ATA Security Erase (single pass, drive-internal). Requires setting a
/// temporary security password first, per the ATA spec — hdparm handles
/// both steps here as two calls.
pub fn ata_secure_erase(device: &Path, password: &str) -> Result<FirmwareEraseResult, CoreError> {
    let started = Instant::now();

    let (set_ok, set_out) = run_command(
        "hdparm",
        device,
        &["--user-master", "u", "--security-set-pass", password],
    )?;
    if !set_ok {
        let diagnostic = decode_firmware_error(FirmwareMethod::AtaSecureErase, &set_out);
        return Ok(FirmwareEraseResult {
            method: FirmwareMethod::AtaSecureErase,
            success: false,
            command_output: format!("security-set-pass failed: {}", set_out),
            duration_secs: started.elapsed().as_secs(),
            password_used: None,
            diagnostic,
        });
    }

    let (erase_ok, erase_out) = run_command(
        "hdparm",
        device,
        &["--user-master", "u", "--security-erase", password],
    )?;

    let mut combined_output = format!("{}\n{}", set_out, erase_out);
    if !erase_ok {
        // Attempt emergency unlock if erase failed so drive is not left permanently locked
        let (_unlock_ok, unlock_out) = run_command(
            "hdparm",
            device,
            &["--user-master", "u", "--security-disable", password],
        )
        .unwrap_or((false, String::new()));
        combined_output.push_str(&format!("\n[EMERGENCY UNLOCK ATTEMPT]: {}", unlock_out));
    }

    let diagnostic = if !erase_ok {
        decode_firmware_error(FirmwareMethod::AtaSecureErase, &combined_output)
    } else {
        None
    };

    Ok(FirmwareEraseResult {
        method: FirmwareMethod::AtaSecureErase,
        success: erase_ok,
        command_output: combined_output,
        duration_secs: started.elapsed().as_secs(),
        password_used: Some(password.to_string()),
        diagnostic,
    })
}

/// ATA Enhanced Security Erase — writes a vendor-defined pattern to every
/// user-addressable sector, including reallocated/reassigned sectors that
/// plain Secure Erase can miss. Not all drives support the enhanced mode;
/// check via `hdparm -I` for "supported: enhanced erase" before calling.
pub fn ata_enhanced_erase(device: &Path, password: &str) -> Result<FirmwareEraseResult, CoreError> {
    let started = Instant::now();

    let (set_ok, set_out) = run_command(
        "hdparm",
        device,
        &["--user-master", "u", "--security-set-pass", password],
    )?;
    if !set_ok {
        let diagnostic = decode_firmware_error(FirmwareMethod::AtaEnhancedErase, &set_out);
        return Ok(FirmwareEraseResult {
            method: FirmwareMethod::AtaEnhancedErase,
            success: false,
            command_output: format!("security-set-pass failed: {}", set_out),
            duration_secs: started.elapsed().as_secs(),
            password_used: None,
            diagnostic,
        });
    }

    let (erase_ok, erase_out) = run_command(
        "hdparm",
        device,
        &["--user-master", "u", "--security-erase-enhanced", password],
    )?;

    let mut combined_output = format!("{}\n{}", set_out, erase_out);
    if !erase_ok {
        let (_unlock_ok, unlock_out) = run_command(
            "hdparm",
            device,
            &["--user-master", "u", "--security-disable", password],
        )
        .unwrap_or((false, String::new()));
        combined_output.push_str(&format!("\n[EMERGENCY UNLOCK ATTEMPT]: {}", unlock_out));
    }

    let diagnostic = if !erase_ok {
        decode_firmware_error(FirmwareMethod::AtaEnhancedErase, &combined_output)
    } else {
        None
    };

    Ok(FirmwareEraseResult {
        method: FirmwareMethod::AtaEnhancedErase,
        success: erase_ok,
        command_output: combined_output,
        duration_secs: started.elapsed().as_secs(),
        password_used: Some(password.to_string()),
        diagnostic,
    })
}

/// Detects Host Protected Area / Device Configuration Overlay — hidden
/// regions some drives reserve that standard erase commands don't always
/// reach. Parsing is heuristic (hdparm's text output isn't a stable API);
/// treat this as advisory, not a compliance guarantee.
pub fn detect_hpa_dco(device: &Path) -> Result<HpaInfo, CoreError> {
    let (n_ok, n_out) = run_command("hdparm", device, &["-N"])?;
    let hpa_enabled = n_ok && n_out.contains("HPA is enabled");

    let (dco_ok, dco_out) = run_command("hdparm", device, &["--dco-identify"])?;
    let dco_enabled = dco_ok && !dco_out.to_lowercase().contains("no dco");

    Ok(HpaInfo {
        hpa_enabled,
        dco_enabled,
    })
}
