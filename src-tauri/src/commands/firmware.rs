use serde::{Deserialize, Serialize};
use sih149_core::wiper::firmware::{
    ata_detect_frozen, ata_enhanced_erase, ata_secure_erase, detect_hpa_dco,
    detect_nvme_capabilities, nvme_block_erase, nvme_crypto_erase, FirmwareMethod,
};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareEraseConfig {
    /// Block device path, e.g. /dev/sdb or /dev/nvme1n1
    pub device_path: String,
    /// "auto", "nvme-crypto", "nvme-block", "ata", or "ata-enhanced"
    pub method: String,
    /// ATA security password (only used for ATA methods)
    pub ata_password: Option<String>,
    pub expert_passphrase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareCapabilities {
    pub is_nvme: bool,
    pub nvme_sanitize_supported: bool,
    pub ata_frozen: bool,
    pub hpa_enabled: bool,
    pub dco_enabled: bool,
    pub recommended_method: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareEraseResult {
    pub method_used: String,
    pub success: bool,
    pub command_output: String,
    pub duration_secs: u64,
    pub warnings: Vec<String>,
    pub password_used: Option<String>,
    #[serde(default)]
    pub case_id: Option<String>,
}

/// Called before erase to show the user what we found and what will happen.
#[tauri::command]
pub fn detect_firmware_capabilities(device_path: String) -> Result<FirmwareCapabilities, String> {
    let is_virtual = device_path.contains("loop") || device_path.contains("zram") || device_path.contains("ram");
    if is_virtual {
        return Ok(FirmwareCapabilities {
            is_nvme: false,
            nvme_sanitize_supported: false,
            ata_frozen: false,
            hpa_enabled: false,
            dco_enabled: false,
            recommended_method: "zero".to_string(),
            warnings: vec![
                "Virtual / in-memory device detected. Physical controller firmware commands (NVMe/ATA) are not applicable. Use standard software overwriting (NIST Clear / DoD 5220.22-M).".to_string(),
            ],
        });
    }

    let device = Path::new(&device_path);
    let is_nvme = device_path.contains("nvme");
    let mut warnings = Vec::new();

    let nvme_sanitize_supported = if is_nvme {
        match detect_nvme_capabilities(device) {
            Ok(caps) => caps.sanitize_supported,
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("No such file or directory") {
                    warnings.push("nvme-cli utility is not installed on this host (required for physical NVMe sanitize commands).".to_string());
                } else {
                    warnings.push(format!("Could not query NVMe capabilities: {}", e));
                }
                false
            }
        }
    } else {
        false
    };

    let ata_frozen = if !is_nvme {
        match ata_detect_frozen(device) {
            Ok(frozen) => {
                if frozen {
                    warnings.push(
                        "Drive is in FROZEN security state. Power-cycle (not reboot) \
                         the machine to unfreeze before issuing a secure erase."
                            .to_string(),
                    );
                }
                frozen
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("No such file or directory") {
                    warnings.push("hdparm utility is not installed on this host (required for physical ATA security commands).".to_string());
                } else {
                    warnings.push(format!("Could not check ATA frozen state: {}", e));
                }
                false
            }
        }
    } else {
        false
    };

    let (hpa_enabled, dco_enabled) = if !is_nvme {
        match detect_hpa_dco(device) {
            Ok(info) => {
                if info.hpa_enabled {
                    warnings.push(
                        "Host Protected Area (HPA) detected — hidden sectors may survive the erase. \
                         Use hdparm --yes-i-know-what-i-am-doing --dco-restore first for full coverage."
                            .to_string()
                    );
                }
                if info.dco_enabled {
                    warnings.push(
                        "Device Configuration Overlay (DCO) detected — drive may report \
                         fewer sectors than it physically contains."
                            .to_string(),
                    );
                }
                (info.hpa_enabled, info.dco_enabled)
            }
            Err(e) => {
                let err_msg = e.to_string();
                if !err_msg.contains("No such file or directory") {
                    warnings.push(format!("Could not check HPA/DCO: {}", e));
                }
                (false, false)
            }
        }
    } else {
        (false, false)
    };

    let recommended_method = if is_nvme {
        if nvme_sanitize_supported {
            "nvme-crypto"
        } else {
            "nvme-block"
        }
    } else if !ata_frozen {
        "ata-enhanced"
    } else {
        "ata"
    }
    .to_string();

    Ok(FirmwareCapabilities {
        is_nvme,
        nvme_sanitize_supported,
        ata_frozen,
        hpa_enabled,
        dco_enabled,
        recommended_method,
        warnings,
    })
}

#[tauri::command]
pub async fn start_firmware_erase(
    config: FirmwareEraseConfig,
) -> Result<FirmwareEraseResult, String> {
    // Firmware-level erases are irreversibly destructive at the hardware
    // controller level. Require expert passphrase authorization for ALL
    // firmware erases (not just system drives) as a defense-in-depth guard.
    let authorized = match &config.expert_passphrase {
        Some(pass) => crate::commands::auth::verify_expert_passphrase(pass.clone())
            .await
            .unwrap_or(false),
        None => false,
    };
    if !authorized {
        return Err(format!(
            "Safety guard: Refusing firmware-level erase on {} — expert passphrase required and did not verify.",
            config.device_path
        ));
    }

    let case_id = format!("fw-{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let _ = crate::commands::reports::create_desktop_case(
        &case_id,
        "Forensic Operator",
        &config.device_path,
        &format!("firmware_purge ({})", config.method),
    );
    let _ = crate::commands::reports::record_desktop_audit_event(
        &case_id,
        "firmware_erase_started",
        serde_json::json!({
            "target": &config.device_path,
            "method": &config.method,
        }),
    );

    let case_id_bg = case_id.clone();
    tokio::task::spawn_blocking(move || -> Result<FirmwareEraseResult, String> {
        let device = Path::new(&config.device_path);
        let is_nvme = config.device_path.contains("nvme");
        let mut warnings = Vec::new();

        let method = match config.method.as_str() {
            "auto" => {
                if is_nvme {
                    match detect_nvme_capabilities(device) {
                        Ok(caps) if caps.sanitize_supported => FirmwareMethod::NvmeCryptoErase,
                        _ => {
                            warnings.push(
                                "NVMe sanitize capability not confirmed — using block erase."
                                    .to_string(),
                            );
                            FirmwareMethod::NvmeBlockErase
                        }
                    }
                } else {
                    FirmwareMethod::AtaEnhancedErase
                }
            }
            other => match other.parse::<FirmwareMethod>() {
                Ok(m) => m,
                Err(_) => {
                    let err = format!(
                        "Unknown method '{}'. Use: auto, nvme-crypto, nvme-block, ata, ata-enhanced",
                        other
                    );
                    let _ = crate::commands::reports::update_desktop_case_status(&case_id_bg, "failed");
                    let _ = crate::commands::reports::record_desktop_audit_event(
                        &case_id_bg,
                        "firmware_erase_failed",
                        serde_json::json!({ "error": &err }),
                    );
                    return Err(err);
                }
            },
        };

        // Safety check: refuse if ATA drive is frozen (command will fail anyway, but give a clear message)
        if matches!(
            method,
            FirmwareMethod::AtaSecureErase | FirmwareMethod::AtaEnhancedErase
        ) {
            if let Ok(true) = ata_detect_frozen(device) {
                let err = "Drive is in FROZEN ATA security state — power-cycle the machine (not reboot) \
                     before issuing a secure erase command."
                    .to_string();
                let _ = crate::commands::reports::update_desktop_case_status(&case_id_bg, "failed");
                let _ = crate::commands::reports::record_desktop_audit_event(
                    &case_id_bg,
                    "firmware_erase_failed",
                    serde_json::json!({ "error": &err }),
                );
                return Err(err);
            }
        }

        let generated_pass: String = {
            use rand::RngCore;
            let mut bytes = [0u8; 8];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            bytes.iter().map(|b| format!("{:02x}", b)).collect()
        };
        let ata_password = config.ata_password.as_deref().unwrap_or(&generated_pass);

        let result = match &method {
            FirmwareMethod::NvmeCryptoErase => {
                nvme_crypto_erase(device).map_err(|e| e.to_string())?
            }
            FirmwareMethod::NvmeBlockErase => {
                nvme_block_erase(device).map_err(|e| e.to_string())?
            }
            FirmwareMethod::AtaSecureErase => {
                ata_secure_erase(device, ata_password).map_err(|e| e.to_string())?
            }
            FirmwareMethod::AtaEnhancedErase => {
                ata_enhanced_erase(device, ata_password).map_err(|e| e.to_string())?
            }
        };

        let method_label = method.as_str();

        let mut final_warnings = warnings;
        if !result.success {
            if let Some(ref diag) = result.diagnostic {
                final_warnings.push(format!("DIAGNOSTIC: {}", diag));
            }
            if let Some(ref pass) = result.password_used {
                final_warnings.push(format!(
                    "CRITICAL: ATA erase failed. If the drive is locked, unlock with password: '{}'",
                    pass
                ));
            }
        }

        let status = if result.success {
            "completed"
        } else {
            "failed"
        };
        let _ = crate::commands::reports::update_desktop_case_status(&case_id_bg, status);
        let event_name = if result.success {
            "firmware_erase_completed"
        } else {
            "firmware_erase_failed"
        };
        let _ = crate::commands::reports::record_desktop_audit_event(
            &case_id_bg,
            event_name,
            serde_json::json!({
                "method": method_label,
                "success": result.success,
                "duration_secs": result.duration_secs,
                "warnings": &final_warnings,
            }),
        );

        Ok(FirmwareEraseResult {
            method_used: method_label.to_string(),
            success: result.success,
            command_output: result.command_output,
            duration_secs: result.duration_secs,
            warnings: final_warnings,
            password_used: result.password_used,
            case_id: Some(case_id_bg),
        })
    })
    .await
    .map_err(|e| format!("Firmware erase task panicked: {}", e))?
}
