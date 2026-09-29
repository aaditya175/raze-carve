use serde::{Deserialize, Serialize};
use sih149_core::disk::block_device::BlockDevice;
use sih149_core::disk::DiskSource;
use sih149_core::wiper::file_wiper::validate_wipe_target;
use sih149_core::wiper::patterns::is_known_method;
use sih149_core::wiper::verify::verify_pattern_wipe;
use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeConfig {
    pub device_path: String,
    pub method: String,
    pub verify: bool,
    pub expert_passphrase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeProgress {
    pub sector_current: u64,
    pub sector_total: u64,
    pub percent: f32,
    pub speed_mbps: f32,
    pub eta_seconds: u64,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeResult {
    pub success: bool,
    pub sectors_wiped: u64,
    pub bad_sectors: u64,
    pub duration_secs: u64,
    pub method_used: String,
    pub verified: bool,
    #[serde(default)]
    pub case_id: Option<String>,
}

lazy_static::lazy_static! {
    static ref WIPE_CANCEL_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

#[tauri::command]
pub async fn start_wipe(config: WipeConfig, app_handle: AppHandle) -> Result<WipeResult, String> {
    // IPC input validation: device path must be present and the method
    // must be one of the known values before we spawn any work.
    if config.device_path.trim().is_empty() {
        return Err("device_path must not be empty".to_string());
    }
    if config.device_path.len() > 4096 {
        return Err("device_path too long".to_string());
    }
    if !is_known_method(&config.method) {
        return Err(format!("Unknown wipe method: {}", config.method));
    }
    validate_wipe_target(std::path::Path::new(&config.device_path))
        .map_err(|error| error.to_string())?;

    WIPE_CANCEL_FLAG.store(false, Ordering::SeqCst);

    let app_handle_bg = app_handle.clone();
    let config_bg = config.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<WipeResult, String> {
        let progress = |p: WipeProgress| {
            let _ = app_handle_bg.emit("wipe-progress", p);
        };
        run_wipe(&config_bg, progress)
    })
    .await
    .map_err(|e| format!("Wipe task panicked: {}", e))??;

    Ok(result)
}

/// Perform a drive sanitization wipe against a block device. Shared by the
/// Tauri IPC command and the HTTP server route so both modes execute real wipes.
pub fn run_wipe(
    config: &WipeConfig,
    mut on_progress: impl FnMut(WipeProgress),
) -> Result<WipeResult, String> {
    if !is_known_method(&config.method) {
        return Err(format!("Unknown wipe method: {}", config.method));
    }
    validate_wipe_target(std::path::Path::new(&config.device_path))
        .map_err(|error| error.to_string())?;

    let target_path = std::path::Path::new(&config.device_path);
    let is_protected = sih149_core::wiper::file_wiper::is_protected_drive(target_path);
    let is_mounted = sih149_core::wiper::file_wiper::is_mounted_block_device(target_path);

    if is_protected || is_mounted {
        let is_authorized = match config.expert_passphrase.as_deref() {
            Some(pass) => crate::commands::auth::verify_expert_passphrase_sync(pass),
            None => false,
        };
        if !is_authorized {
            let reason = if is_protected {
                format!(
                    "Refusing to wipe protected system drive '{}' without valid expert passphrase authorization",
                    config.device_path
                )
            } else {
                format!(
                    "Device or partition on '{}' is currently mounted. Unmount all partitions or provide valid expert passphrase authorization",
                    config.device_path
                )
            };
            return Err(reason);
        }
    }

    let case_id = format!("wipe-{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let _ = crate::commands::reports::create_desktop_case(
        &case_id,
        "Forensic Operator",
        &config.device_path,
        &format!("erasure ({})", config.method),
    );
    let _ = crate::commands::reports::record_desktop_audit_event(
        &case_id,
        "wipe_started",
        serde_json::json!({
            "target": config.device_path,
            "method": config.method,
            "verify": config.verify,
        }),
    );

    let mut disk = match BlockDevice::open_for_wipe(&config.device_path) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("Failed to open {}: {}", config.device_path, e);
            let _ = crate::commands::reports::update_desktop_case_status(&case_id, "failed");
            let _ = crate::commands::reports::record_desktop_audit_event(
                &case_id,
                "wipe_failed",
                serde_json::json!({ "error": msg }),
            );
            return Err(msg);
        }
    };

    let size = match disk.size() {
        Ok(s) => s,
        Err(e) => {
            let msg = e.to_string();
            let _ = crate::commands::reports::update_desktop_case_status(&case_id, "failed");
            let _ = crate::commands::reports::record_desktop_audit_event(
                &case_id,
                "wipe_failed",
                serde_json::json!({ "error": msg }),
            );
            return Err(msg);
        }
    };
    let chunk_size: u64 = 1024 * 1024;

    let passes = sih149_core::wiper::patterns::passes_for_method(&config.method);

    let started = Instant::now();
    let total_bytes = size * passes.len() as u64;
    let mut bytes_done_all_passes: u64 = 0;

    let mut last_emit = Instant::now();
    let mut last_pattern = None;
    for (pass_idx, pass) in passes.iter().enumerate() {
        let mut pattern =
            sih149_core::wiper::patterns::WipePattern::for_method_pass(&config.method, *pass);
        let mut buf = vec![0u8; chunk_size as usize];
        if let Err(e) = disk.seek(SeekFrom::Start(0)) {
            let msg = e.to_string();
            let _ = crate::commands::reports::update_desktop_case_status(&case_id, "failed");
            let _ = crate::commands::reports::record_desktop_audit_event(
                &case_id,
                "wipe_failed",
                serde_json::json!({ "error": msg }),
            );
            return Err(msg);
        }
        let mut written: u64 = 0;

        while written < size {
            if WIPE_CANCEL_FLAG.load(Ordering::SeqCst) {
                let _ = crate::commands::reports::update_desktop_case_status(&case_id, "cancelled");
                let _ = crate::commands::reports::record_desktop_audit_event(
                    &case_id,
                    "wipe_cancelled",
                    serde_json::json!({
                        "bytes_written": bytes_done_all_passes,
                        "sectors_wiped": bytes_done_all_passes / 512,
                    }),
                );
                return Err("Wipe cancelled".to_string());
            }

            let this_chunk = std::cmp::min(chunk_size, size - written) as usize;
            pattern.fill(&mut buf[..this_chunk]);
            if let Err(e) = disk.write_all(&buf[..this_chunk]) {
                let msg = e.to_string();
                let _ = crate::commands::reports::update_desktop_case_status(&case_id, "failed");
                let _ = crate::commands::reports::record_desktop_audit_event(
                    &case_id,
                    "wipe_failed",
                    serde_json::json!({ "error": msg }),
                );
                return Err(msg);
            }
            written += this_chunk as u64;
            bytes_done_all_passes += this_chunk as u64;

            let elapsed = started.elapsed().as_secs_f32().max(0.001);
            let speed_mbps = (bytes_done_all_passes as f32 / elapsed) / (1024.0 * 1024.0);
            let percent = (bytes_done_all_passes as f32 / total_bytes as f32) * 100.0;
            let remaining_bytes = total_bytes.saturating_sub(bytes_done_all_passes);
            let eta_seconds = if speed_mbps > 0.0 {
                (remaining_bytes as f32 / (speed_mbps * 1024.0 * 1024.0)) as u64
            } else {
                0
            };

            // Throttle progress callback to 200ms intervals or final chunk
            if last_emit.elapsed().as_millis() >= 200 || written >= size {
                let progress = WipeProgress {
                    sector_current: bytes_done_all_passes / 512,
                    sector_total: total_bytes / 512,
                    percent,
                    speed_mbps,
                    eta_seconds,
                    phase: format!("Wiping (pass {}/{})", pass_idx + 1, passes.len()),
                };
                on_progress(progress);
                last_emit = Instant::now();
            }
        }
        if let Err(e) = disk.flush() {
            let msg = e.to_string();
            let _ = crate::commands::reports::update_desktop_case_status(&case_id, "failed");
            let _ = crate::commands::reports::record_desktop_audit_event(
                &case_id,
                "wipe_failed",
                serde_json::json!({ "error": msg }),
            );
            return Err(msg);
        }
        if let Err(e) = disk.sync_all() {
            let msg = e.to_string();
            let _ = crate::commands::reports::update_desktop_case_status(&case_id, "failed");
            let _ = crate::commands::reports::record_desktop_audit_event(
                &case_id,
                "wipe_failed",
                serde_json::json!({ "error": msg }),
            );
            return Err(msg);
        }
        last_pattern = Some(pattern);
    }

    let mut verified = false;
    if config.verify {
        let pattern = last_pattern
            .as_ref()
            .ok_or_else(|| "No wipe pass completed; cannot verify".to_string())?;
        match verify_pattern_wipe(&mut disk, pattern, 10) {
            Ok(v) => {
                verified = v;
                if !verified {
                    let _ =
                        crate::commands::reports::update_desktop_case_status(&case_id, "failed");
                    let _ = crate::commands::reports::record_desktop_audit_event(
                        &case_id,
                        "wipe_verification_failed",
                        serde_json::json!({ "target": config.device_path }),
                    );
                    return Err("Verification failed: residual data pattern detected".to_string());
                }
            }
            Err(e) => {
                let msg = e.to_string();
                let _ = crate::commands::reports::update_desktop_case_status(&case_id, "failed");
                let _ = crate::commands::reports::record_desktop_audit_event(
                    &case_id,
                    "wipe_failed",
                    serde_json::json!({ "error": msg }),
                );
                return Err(msg);
            }
        }
    }

    let duration_secs = started.elapsed().as_secs();
    let _ = crate::commands::reports::update_desktop_case_status(&case_id, "completed");
    let _ = crate::commands::reports::record_desktop_audit_event(
        &case_id,
        "wipe_completed",
        serde_json::json!({
            "target": config.device_path,
            "sectors_wiped": size / 512,
            "duration_secs": duration_secs,
            "method": config.method,
            "verified": verified,
        }),
    );

    // Automatically generate forensic certificate PDF in cases/<case_id>/<case_id>_certificate.pdf
    let rpt_dir = format!("cases/{}", case_id);
    let _ = std::fs::create_dir_all(&rpt_dir);
    let rpt_file = format!("{}/{}_certificate.pdf", rpt_dir, case_id);
    let _ = crate::commands::reports::export_report(case_id.clone(), "erasure".into(), rpt_file, None);

    Ok(WipeResult {
        success: true,
        sectors_wiped: size / 512,
        bad_sectors: 0,
        duration_secs,
        method_used: config.method.clone(),
        verified,
        case_id: Some(case_id),
    })
}

#[tauri::command]
pub fn cancel_wipe() -> Result<(), String> {
    WIPE_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn estimate_wipe_time(device_path: String, method: String) -> Result<u64, String> {
    let passes = sih149_core::wiper::patterns::passes_for_method(&method).len() as u64;

    let size_bytes: u64 = if let Ok(disk) = BlockDevice::open_readonly(&device_path) {
        disk.size().unwrap_or(500 * 1024 * 1024 * 1024)
    } else if let Ok(meta) = std::fs::metadata(&device_path) {
        if meta.len() > 0 {
            meta.len()
        } else {
            500 * 1024 * 1024 * 1024
        }
    } else {
        500 * 1024 * 1024 * 1024
    };

    let throughput_bps: u64 = 100 * 1024 * 1024; // ~100 MB/s typical sustained write
    let total_bytes = size_bytes.saturating_mul(passes);
    let seconds = (total_bytes / throughput_bps).max(10);
    Ok(seconds)
}
