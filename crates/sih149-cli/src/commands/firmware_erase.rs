use crate::display;
use clap::Args;
use sih149_core::wiper::firmware::{
    ata_detect_frozen, ata_enhanced_erase, ata_secure_erase, detect_hpa_dco,
    detect_nvme_capabilities, nvme_block_erase, nvme_crypto_erase, FirmwareMethod,
};
use std::path::Path;

/// Firmware-level secure erase: NVMe sanitize or ATA secure erase.
///
/// This goes deeper than software-layer overwrites. The drive's internal
/// controller handles the erase, including reallocated/remapped sectors
/// that the OS has no visibility into.
///
/// REQUIRES:
///   Linux: root (CAP_SYS_ADMIN), `nvme-cli` or `hdparm` packages installed
///   The target drive must NOT be the boot/system drive
#[derive(Args)]
pub struct FirmwareEraseArgs {
    /// Target block device (e.g. /dev/sdb, /dev/nvme1n1)
    #[arg(short, long)]
    pub device: String,

    /// Erase method:
    ///   nvme-crypto  — NVMe sanitize crypto erase (destroys encryption key)
    ///   nvme-block   — NVMe sanitize block erase (rewrites all blocks)
    ///   ata          — ATA Security Erase (single pass)
    ///   ata-enhanced — ATA Enhanced Security Erase (includes remapped sectors)
    ///   auto         — detect capabilities and choose the strongest available
    #[arg(short, long, default_value = "auto")]
    pub method: String,

    /// ATA security password (required for ATA methods, ignored for NVMe)
    #[arg(long, value_name = "PASSWORD")]
    pub ata_password: Option<String>,

    /// Skip confirmation — DANGER: this is irreversible firmware-level destruction
    #[arg(long)]
    pub yes: bool,
}

pub fn run(args: &FirmwareEraseArgs) -> anyhow::Result<()> {
    let device = Path::new(&args.device);

    if !sih149_core::wiper::firmware::is_valid_block_device_path(device) {
        display::print_error(&format!(
            "Invalid block device path: {} (must be a valid /dev block device)",
            args.device
        ));
        anyhow::bail!("Invalid block device path");
    }

    if sih149_core::wiper::file_wiper::is_protected_drive(device) {
        display::print_error(&format!(
            "Refusing firmware erase on protected/system drive: {}",
            args.device
        ));
        anyhow::bail!("Refusing to erase protected system drive");
    }

    if sih149_core::wiper::firmware::is_device_mounted(device) {
        display::print_error(&format!(
            "Device or partition on {} is currently mounted. Unmount all partitions before firmware erase.",
            args.device
        ));
        anyhow::bail!("Target drive has mounted partitions");
    }

    if !args.yes {
        println!("═══════════════════════════════════════════════════");
        println!("  FIRMWARE-LEVEL ERASE — IRREVERSIBLE");
        println!("  Device : {}", args.device);
        println!("  Method : {}", args.method);
        println!("═══════════════════════════════════════════════════");
        println!();
        println!("This command issues a FIRMWARE secure erase command directly");
        println!("to the drive. The drive's controller erases ALL sectors,");
        println!("including reallocated blocks invisible to the OS.");
        println!();
        println!("There is NO undo. ALL data will be PERMANENTLY destroyed.");
        println!();
        print!("Type ERASE-FIRMWARE to continue: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut input = String::new();
        std::io::BufRead::read_line(&mut std::io::BufReader::new(std::io::stdin()), &mut input)?;
        if input.trim() != "ERASE-FIRMWARE" {
            display::print_warn("Aborted by user.");
            return Ok(());
        }
    }

    display::print_section("Firmware Erase");

    let is_nvme = args.device.contains("nvme");
    let method = if args.method == "auto" {
        detect_best_method(device, is_nvme)?
    } else {
        parse_method(&args.method)?
    };

    let ata_password = if matches!(
        method,
        FirmwareMethod::AtaSecureErase | FirmwareMethod::AtaEnhancedErase
    ) {
        match args.ata_password.as_deref().filter(|password| !password.is_empty()) {
            Some(password) => password,
            None => anyhow::bail!(
                "ATA erase requires --ata-password. Use a unique, temporary password and retain it until the drive reports completion."
            ),
        }
    } else {
        ""
    };

    println!("Device : {}", args.device);
    println!("Method : {:?}", method);

    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("secureforge")
        .join("cases.db");
    let case_id = format!(
        "fw-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );

    if let Ok(db) = sih149_core::db::CaseDatabase::open(&db_path) {
        let _ = db.create_case(
            &case_id,
            "CLI Operator",
            &args.device,
            &format!("firmware_purge ({})", method.as_str()),
        );
        let audit_payload = serde_json::json!({
            "event": "firmware_erase_started",
            "target": &args.device,
            "method": method.as_str(),
        });
        let json_str = audit_payload.to_string();
        let hash = sih149_core::audit::signing::hash_bytes(json_str.as_bytes());
        let _ = db.insert_audit_entry(&case_id, &json_str, &hash);
    }

    // Pre-flight checks
    if matches!(
        method,
        FirmwareMethod::AtaSecureErase | FirmwareMethod::AtaEnhancedErase
    ) {
        match ata_detect_frozen(device) {
            Ok(true) => {
                if let Ok(db) = sih149_core::db::CaseDatabase::open(&db_path) {
                    let _ = db.update_case_status(&case_id, "failed");
                    let err_json = serde_json::json!({
                        "event": "firmware_erase_failed",
                        "error": "Drive is in FROZEN ATA security state",
                    })
                    .to_string();
                    let hash = sih149_core::audit::signing::hash_bytes(err_json.as_bytes());
                    let _ = db.insert_audit_entry(&case_id, &err_json, &hash);
                }
                anyhow::bail!(
                    "Drive is in FROZEN security state — the BIOS locked it at boot.\n\
                     Power-cycle (not reboot) the machine without triggering another BIOS freeze,\n\
                     then retry. On some systems, hot-unplugging and reinserting breaks the freeze."
                );
            }
            Ok(false) => {}
            Err(e) => {
                display::print_warn(&format!("Could not check frozen state: {} — continuing", e))
            }
        }

        match detect_hpa_dco(device) {
            Ok(info) => {
                if info.hpa_enabled {
                    display::print_warn(
                        "Host Protected Area (HPA) detected — hidden sectors exist that this erase may not reach. \
                         Use hdparm --yes-i-know-what-i-am-doing --dco-restore to expand before erasing for full coverage."
                    );
                }
                if info.dco_enabled {
                    display::print_warn(
                        "Device Configuration Overlay (DCO) detected — drive may report fewer sectors than it contains."
                    );
                }
            }
            Err(e) => display::print_warn(&format!("Could not check HPA/DCO: {} — continuing", e)),
        }
    }

    if matches!(
        method,
        FirmwareMethod::NvmeCryptoErase | FirmwareMethod::NvmeBlockErase
    ) {
        match detect_nvme_capabilities(device) {
            Ok(caps) => {
                if matches!(method, FirmwareMethod::NvmeCryptoErase) && !caps.sanitize_supported {
                    display::print_warn(
                        "Crypto erase not confirmed via id-ctrl SANICAP — the drive may still support it \
                         (parse is heuristic). Proceeding, but verify the sanitize log page after completion."
                    );
                }
            }
            Err(e) => display::print_warn(&format!(
                "Could not query NVMe capabilities: {} — continuing",
                e
            )),
        }
    }

    println!("Issuing firmware erase command…");

    let result =
        match method {
            FirmwareMethod::NvmeCryptoErase => {
                nvme_crypto_erase(device).map_err(|e| anyhow::anyhow!(e.to_string()))?
            }
            FirmwareMethod::NvmeBlockErase => {
                nvme_block_erase(device).map_err(|e| anyhow::anyhow!(e.to_string()))?
            }
            FirmwareMethod::AtaSecureErase => ata_secure_erase(device, ata_password)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?,
            FirmwareMethod::AtaEnhancedErase => ata_enhanced_erase(device, ata_password)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?,
        };

    if let Ok(db) = sih149_core::db::CaseDatabase::open(&db_path) {
        let status = if result.success {
            "completed"
        } else {
            "failed"
        };
        let _ = db.update_case_status(&case_id, status);
        let audit_payload = serde_json::json!({
            "event": if result.success { "firmware_erase_completed" } else { "firmware_erase_failed" },
            "target": &args.device,
            "method": method.as_str(),
            "duration_secs": result.duration_secs,
            "success": result.success,
        });
        let json_str = audit_payload.to_string();
        let hash = sih149_core::audit::signing::hash_bytes(json_str.as_bytes());
        let _ = db.insert_audit_entry(&case_id, &json_str, &hash);
    }

    println!();
    println!("Case ID : {}", case_id);
    println!("Command output:\n{}", result.command_output.trim());
    println!();
    println!("Duration: {}s", result.duration_secs);

    if result.success {
        display::print_success("Firmware erase completed successfully.");
    } else {
        if let Some(ref diag) = result.diagnostic {
            display::print_error(&format!("Diagnostic: {}", diag));
        }
        display::print_error("Firmware erase reported a failure. Check command output above.");
        anyhow::bail!("Firmware erase failed");
    }

    Ok(())
}

fn detect_best_method(device: &Path, is_nvme: bool) -> anyhow::Result<FirmwareMethod> {
    if is_nvme {
        match detect_nvme_capabilities(device) {
            Ok(caps) if caps.sanitize_supported => {
                println!("Auto-detected: NVMe crypto erase (SANICAP supported).");
                Ok(FirmwareMethod::NvmeCryptoErase)
            }
            _ => {
                println!("Auto-detected: NVMe block erase (falling back from crypto erase).");
                Ok(FirmwareMethod::NvmeBlockErase)
            }
        }
    } else {
        println!("Auto-detected: ATA Enhanced Security Erase (covers remapped sectors).");
        Ok(FirmwareMethod::AtaEnhancedErase)
    }
}

fn parse_method(s: &str) -> anyhow::Result<FirmwareMethod> {
    s.parse::<FirmwareMethod>().map_err(|e| anyhow::anyhow!(e))
}
