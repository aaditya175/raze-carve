use crate::app::{shannon_entropy, App, CarvedFile, WipeMethod, WipePhase};
use sih149_core::carver::engine::CarvingEngine;
use sih149_core::carver::entropy_clustering::{cluster_entropy_streams, EntropyType};
use sih149_core::carver::filesystem_records::{scan_fat_directory_entries, scan_ntfs_mft_records};
use sih149_core::carver::scanner::SectorScanner;
use sih149_core::carver::signatures::SignatureDatabase;
use sih149_core::carver::structure::{
    jpeg::parse_jpeg_metadata, pdf::parse_pdf_metadata, png::parse_png_metadata,
    sqlite::parse_sqlite_metadata, zip::parse_zip_metadata,
};
use sih149_core::disk::block_device::BlockDevice;
use sih149_core::disk::{DiskSource, EvidenceSource};
use sih149_core::wiper::file_wiper::validate_wipe_target;
use sih149_core::wiper::verify::verify_pattern_wipe;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn start_wipe(app: Arc<Mutex<App>>, device_path: String, method: WipeMethod, do_verify: bool) {
    std::thread::spawn(move || {
        let wipe_cancel = {
            let a = app.lock().unwrap();
            a.wipe_cancel.store(false, Ordering::SeqCst);
            a.wipe_cancel.clone()
        };

        if let Err(error) = validate_wipe_target(std::path::Path::new(&device_path)) {
            let mut a = app.lock().unwrap();
            a.wipe_phase = WipePhase::Error(error.to_string());
            a.push_log(format!("Safety check failed: {}", error));
            return;
        }
        let db_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("secureforge")
            .join("cases.db");
        let case_id = format!(
            "wipe-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        let db = sih149_core::db::CaseDatabase::open(&db_path).ok();

        let size = {
            match BlockDevice::open_for_wipe(&device_path) {
                Ok(disk) => match disk.size() {
                    Ok(s) => s,
                    Err(e) => {
                        let mut a = app.lock().unwrap();
                        a.wipe_phase = WipePhase::Error(e.to_string());
                        a.push_log(format!("Error: {}", e));
                        if let Some(d) = db.as_ref() {
                            let _ =
                                d.create_case(&case_id, "TUI Operator", &device_path, "erasure");
                            let _ = d.update_case_status(&case_id, "failed");
                        }
                        return;
                    }
                },
                Err(e) => {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error(format!("Cannot open {}: {}", device_path, e));
                    a.push_log(format!("Error: Cannot open {}: {}", device_path, e));
                    if let Some(d) = db.as_ref() {
                        let _ = d.create_case(&case_id, "TUI Operator", &device_path, "erasure");
                        let _ = d.update_case_status(&case_id, "failed");
                    }
                    return;
                }
            }
        };

        let method_str = match method {
            WipeMethod::Zero => "zero",
            WipeMethod::Random => "random",
            WipeMethod::Dod3 => "dod3",
            WipeMethod::Dod7 => "dod7",
            WipeMethod::Nist => "nist",
            WipeMethod::Gutmann => "gutmann",
        };

        if let Some(d) = db.as_ref() {
            let _ = d.create_case(
                &case_id,
                "TUI Operator",
                &device_path,
                &format!("erasure ({})", method_str),
            );
            let audit_init = serde_json::json!({
                "event": "wipe_started",
                "target": &device_path,
                "method": method_str,
                "verify": do_verify,
            })
            .to_string();
            let hash = sih149_core::audit::signing::hash_bytes(audit_init.as_bytes());
            let _ = d.insert_audit_entry(&case_id, &audit_init, &hash);
        }

        let passes = sih149_core::wiper::patterns::passes_for_method(method_str);
        let total_passes = passes.len();
        let chunk_size: usize = 4 * 1024 * 1024; // 4MB chunks
        let started = Instant::now();

        let mut buf = vec![0u8; chunk_size];
        let mut last_pattern = None;
        for (pass_idx, &pass_type) in passes.iter().enumerate() {
            if wipe_cancel.load(Ordering::Relaxed) {
                let mut a = app.lock().unwrap();
                a.wipe_phase = WipePhase::Error("Wipe cancelled by operator".to_string());
                a.push_log_level(crate::app::LogLevel::Warning, "Wipe aborted by user");
                if let Some(d) = db.as_ref() {
                    let _ = d.update_case_status(&case_id, "aborted");
                    let audit_abort = serde_json::json!({
                        "event": "wipe_aborted",
                        "target": &device_path,
                        "pass": pass_idx + 1,
                    })
                    .to_string();
                    let hash = sih149_core::audit::signing::hash_bytes(audit_abort.as_bytes());
                    let _ = d.insert_audit_entry(&case_id, &audit_abort, &hash);
                }
                return;
            }

            let mut pattern =
                sih149_core::wiper::patterns::WipePattern::for_method_pass(method_str, pass_type);

            let mut disk = match BlockDevice::open_for_wipe(&device_path) {
                Ok(d) => d,
                Err(e) => {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error(format!("Open error: {}", e));
                    if let Some(d) = db.as_ref() {
                        let _ = d.update_case_status(&case_id, "failed");
                    }
                    return;
                }
            };

            if disk.seek(SeekFrom::Start(0)).is_err() {
                let mut a = app.lock().unwrap();
                a.wipe_phase = WipePhase::Error("Seek error".to_string());
                if let Some(d) = db.as_ref() {
                    let _ = d.update_case_status(&case_id, "failed");
                }
                return;
            }

            let mut written: u64 = 0;
            let mut last_speed_update = Instant::now();
            let mut last_ui_update = Instant::now();
            let mut bytes_since_update: u64 = 0;
            let mut speed_mbps: f64 = 0.0;

            while written < size {
                if wipe_cancel.load(Ordering::Relaxed) {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error("Wipe cancelled by operator".to_string());
                    a.push_log_level(crate::app::LogLevel::Warning, "Wipe aborted by user");
                    if let Some(d) = db.as_ref() {
                        let _ = d.update_case_status(&case_id, "aborted");
                        let audit_abort = serde_json::json!({
                            "event": "wipe_aborted",
                            "target": &device_path,
                            "bytes_written": written,
                            "pass": pass_idx + 1,
                        })
                        .to_string();
                        let hash = sih149_core::audit::signing::hash_bytes(audit_abort.as_bytes());
                        let _ = d.insert_audit_entry(&case_id, &audit_abort, &hash);
                    }
                    return;
                }

                let this_chunk = std::cmp::min(chunk_size as u64, size - written) as usize;
                pattern.fill(&mut buf[..this_chunk]);
                if disk.write_all(&buf[..this_chunk]).is_err() {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error("Write error".to_string());
                    if let Some(d) = db.as_ref() {
                        let _ = d.update_case_status(&case_id, "failed");
                    }
                    return;
                }
                written += this_chunk as u64;
                bytes_since_update += this_chunk as u64;

                if last_speed_update.elapsed().as_millis() >= 250 {
                    let elapsed_s = last_speed_update.elapsed().as_secs_f64();
                    if elapsed_s >= 0.05 {
                        let instant = (bytes_since_update as f64) / (elapsed_s * 1_048_576.0);
                        if instant.is_finite() && instant >= 0.0 {
                            speed_mbps = if speed_mbps == 0.0 {
                                instant
                            } else {
                                speed_mbps * 0.7 + instant * 0.3
                            };
                        }
                    }
                    bytes_since_update = 0;
                    last_speed_update = Instant::now();
                }

                // Throttle UI mutex acquisition to once every 150ms instead of every 4MB chunk.
                if last_ui_update.elapsed().as_millis() >= 150 || written >= size {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Running {
                        pass: pass_idx + 1,
                        total_passes,
                        bytes_done: written,
                        bytes_total: size,
                        speed_mbps,
                        started,
                    };
                    last_ui_update = Instant::now();
                }
            }
            if let Err(error) = disk.sync_all() {
                let mut a = app.lock().unwrap();
                a.wipe_phase = WipePhase::Error(format!("Flush error: {}", error));
                if let Some(d) = db.as_ref() {
                    let _ = d.update_case_status(&case_id, "failed");
                }
                return;
            }
            last_pattern = Some(pattern);
        }

        if do_verify {
            {
                let mut a = app.lock().unwrap();
                a.wipe_phase = WipePhase::Verifying;
                a.push_log("Verifying wipe...".to_string());
            }

            let mut disk = match BlockDevice::open_for_wipe(&device_path) {
                Ok(d) => d,
                Err(e) => {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error(format!("Verify open error: {}", e));
                    if let Some(d) = db.as_ref() {
                        let _ = d.update_case_status(&case_id, "failed");
                    }
                    return;
                }
            };

            let Some(pattern) = last_pattern.as_ref() else {
                let mut a = app.lock().unwrap();
                a.wipe_phase =
                    WipePhase::Error("No wipe pass completed; cannot verify".to_string());
                return;
            };
            match verify_pattern_wipe(&mut disk, pattern, 10) {
                Ok(true) => {
                    let mut a = app.lock().unwrap();
                    let elapsed = started.elapsed();
                    a.wipe_phase = WipePhase::Done {
                        success: true,
                        elapsed,
                    };
                    a.push_log(format!(
                        "Wipe verified successfully in {:.1}s",
                        elapsed.as_secs_f64()
                    ));
                    if let Some(d) = db.as_ref() {
                        let _ = d.update_case_status(&case_id, "completed");
                        let audit_done = serde_json::json!({
                            "event": "wipe_completed",
                            "target": &device_path,
                            "method": method_str,
                            "verified": true,
                            "passes": total_passes,
                            "elapsed_seconds": elapsed.as_secs_f64(),
                        })
                        .to_string();
                        let hash = sih149_core::audit::signing::hash_bytes(audit_done.as_bytes());
                        let _ = d.insert_audit_entry(&case_id, &audit_done, &hash);
                    }
                }
                Ok(false) => {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error(
                        "Verification failed: non-zero/pattern bytes detected".to_string(),
                    );
                    a.push_log("ERROR: Verification failed!".to_string());
                    if let Some(d) = db.as_ref() {
                        let _ = d.update_case_status(&case_id, "failed");
                        let audit_fail = serde_json::json!({
                            "event": "wipe_verification_failed",
                            "target": &device_path,
                            "error": "Verification failed: non-zero/pattern bytes detected",
                        })
                        .to_string();
                        let hash = sih149_core::audit::signing::hash_bytes(audit_fail.as_bytes());
                        let _ = d.insert_audit_entry(&case_id, &audit_fail, &hash);
                    }
                }
                Err(e) => {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error(format!("Verification error: {}", e));
                    if let Some(d) = db.as_ref() {
                        let _ = d.update_case_status(&case_id, "failed");
                    }
                }
            }
        } else {
            let mut a = app.lock().unwrap();
            let elapsed = started.elapsed();
            a.wipe_phase = WipePhase::Done {
                success: true,
                elapsed,
            };
            a.push_log(format!(
                "Wipe completed in {:.1}s (unverified)",
                elapsed.as_secs_f64()
            ));
            if let Some(d) = db.as_ref() {
                let _ = d.update_case_status(&case_id, "completed");
                let audit_done = serde_json::json!({
                    "event": "wipe_completed",
                    "target": &device_path,
                    "method": method_str,
                    "verified": false,
                    "passes": total_passes,
                    "elapsed_seconds": elapsed.as_secs_f64(),
                })
                .to_string();
                let hash = sih149_core::audit::signing::hash_bytes(audit_done.as_bytes());
                let _ = d.insert_audit_entry(&case_id, &audit_done, &hash);
            }
        }
    });
}

pub fn start_entropy_scan(app: Arc<Mutex<App>>, drive_idx: usize, num_samples: usize) {
    // NOTE: Caller must set app.entropy_scanning = Some(drive_idx) BEFORE calling
    // this function. We cannot lock the mutex here because the caller already holds it,
    // and std::sync::Mutex is NOT reentrant — locking again = deadlock.

    std::thread::spawn(move || {
        let path = {
            let a = app.lock().unwrap();
            if drive_idx >= a.drives.len() {
                return;
            }
            a.drives[drive_idx].path.clone()
        };

        let chunk_size: usize = 4 * 1024; // 4KB — fast random reads, less disk stall per sample
        let mut samples = Vec::with_capacity(num_samples);

        let mut disk = match EvidenceSource::open(&path) {
            Ok(d) => d,
            Err(e) => {
                let mut a = app.lock().unwrap();
                a.entropy_scanning = None;
                a.push_log_level(
                    crate::app::LogLevel::Error,
                    format!("Cannot open {}: {}", path, e),
                );
                return;
            }
        };

        let total_size = disk.size().unwrap_or(0);
        if total_size > 0 {
            let step = (total_size.saturating_sub(chunk_size as u64)) / (num_samples as u64);
            let mut buf = vec![0u8; chunk_size];

            for i in 0..num_samples {
                let offset = (i as u64) * step;
                if disk.seek(SeekFrom::Start(offset)).is_ok() {
                    let n = disk.read(&mut buf).unwrap_or(0);
                    if n > 0 {
                        samples.push(shannon_entropy(&buf[..n]));
                    }
                }
                // Yield 2ms between every sample — lets the OS schedule the UI
                // thread so it can render/respond regardless of disk latency.
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        }

        let mut a = app.lock().unwrap();
        a.entropy_scanning = None;
        if samples.is_empty() {
            a.push_log_level(
                crate::app::LogLevel::Error,
                format!("Failed to read {}: permission denied or unreadable", path),
            );
            return;
        }

        if drive_idx < a.drives.len() {
            let name = a.drives[drive_idx].name.clone();
            // Compute stats once here — UI render path reads cached values, zero iteration
            let stats = crate::app::EntropyStats::compute(&samples);
            let avg = stats.avg;
            a.drives[drive_idx].entropy_samples = samples;
            a.drives[drive_idx].entropy_stats = stats;
            a.drives[drive_idx].entropy_loaded = true;
            a.push_log_level(
                crate::app::LogLevel::Success,
                format!("Entropy /dev/{}: {:.3} b/B avg", name, avg),
            );
            a.set_status(format!("Entropy loaded: /dev/{} ({:.2} b/B)", name, avg));
        }
    });
}

fn builtin_signatures() -> SignatureDatabase {
    // Always start with the comprehensive builtin catalog
    let mut signatures = SignatureDatabase::builtin().signatures;

    // Merge TOML plugin signatures on top (additive, not replacement)
    let sig_candidates = [
        std::path::PathBuf::from("plugins/signatures"),
        std::path::PathBuf::from("../plugins/signatures"),
        std::path::PathBuf::from("../../plugins/signatures"),
    ];
    if let Some(sig_dir) = sig_candidates.iter().find(|p| p.is_dir()) {
        if let Ok(toml_sigs) = sih149_core::plugins::toml_loader::load_signatures_from_dir(sig_dir)
        {
            for sig in toml_sigs {
                let magic_header = sig
                    .header_bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                let magic_footer = sig
                    .footer_bytes
                    .map(|fb| fb.iter().map(|b| format!("{:02x}", b)).collect::<String>());
                // Fix: extract first word from name and lowercase it as extension,
                // e.g. "MP3 (ID3v2)" → "mp3", "TIFF (Little Endian)" → "tiff"
                let extension = sig
                    .extensions
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| {
                        sig.name
                            .split(|c: char| c.is_whitespace() || c == '(' || c == '/')
                            .next()
                            .unwrap_or(&sig.name)
                            .to_lowercase()
                    });
                // Convert mask: if all true, pass None (backward compat)
                let has_wildcards = sig.header_mask.iter().any(|m| !m);
                let header_mask = if has_wildcards {
                    Some(sig.header_mask)
                } else {
                    None
                };
                signatures.push(sih149_core::carver::signatures::FileSignature {
                    extension,
                    description: sig.name,
                    magic_header,
                    magic_footer,
                    max_size: sig.max_size_bytes,
                    header_mask,
                });
            }
        }
    }

    SignatureDatabase { signatures }
}

pub fn start_carver_scan(
    app: Arc<Mutex<App>>,
    source: String,
    output_dir: String,
    min_confidence: u8,
    mode_idx: usize,
) {
    // NOTE: Do NOT lock app here — caller already holds the mutex.
    // std::sync::Mutex is not reentrant; locking again = deadlock.

    std::thread::spawn(move || {
        let (carver_cancel, inspect_only) = {
            let a = app.lock().unwrap();
            a.carver_cancel.store(false, Ordering::SeqCst);
            (a.carver_cancel.clone(), a.carver.inspect_only)
        };
        let db_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("secureforge")
            .join("cases.db");
        let case_id = format!(
            "rec-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        let db = sih149_core::db::CaseDatabase::open(&db_path).ok();
        if let Some(d) = db.as_ref() {
            let _ = d.create_case(&case_id, "TUI Operator", &source, "recovery");
            let audit_init = serde_json::json!({
                "event": "carve_started",
                "source": &source,
                "min_confidence": min_confidence,
                "mode_idx": mode_idx,
            })
            .to_string();
            let hash = sih149_core::audit::signing::hash_bytes(audit_init.as_bytes());
            let _ = d.insert_audit_entry(&case_id, &audit_init, &hash);
        }

        let db_sig = builtin_signatures();
        let scanner = match SectorScanner::new(db_sig) {
            Ok(s) => s,
            Err(e) => {
                let mut a = app.lock().unwrap();
                a.carver.scanning = false;
                a.push_log_level(
                    crate::app::LogLevel::Error,
                    format!("Scanner init error: {}", e),
                );
                if let Some(d) = db.as_ref() {
                    let _ = d.update_case_status(&case_id, "failed");
                }
                return;
            }
        };

        let mut lua_host = sih149_core::plugins::lua_host::LuaPluginHost::new();
        let script_candidates = [
            std::path::PathBuf::from("plugins/scripts"),
            std::path::PathBuf::from("../plugins/scripts"),
            std::path::PathBuf::from("../../plugins/scripts"),
        ];
        if let Some(scripts_dir) = script_candidates.iter().find(|p| p.is_dir()) {
            let _ = lua_host.load_directory(scripts_dir);
        }

        let engine = if lua_host.plugin_count() > 0 {
            CarvingEngine::new(scanner).with_lua_plugins(lua_host)
        } else {
            CarvingEngine::new(scanner)
        };

        let mut evidence = match EvidenceSource::open(&source) {
            Ok(d) => d,
            Err(e) => {
                let mut a = app.lock().unwrap();
                a.carver.scanning = false;
                a.push_log_level(
                    crate::app::LogLevel::Error,
                    format!("Failed to open {}: {}", source, e),
                );
                if let Some(d) = db.as_ref() {
                    let _ = d.update_case_status(&case_id, "failed");
                }
                return;
            }
        };

        let app_progress = Arc::clone(&app);
        let mut last_progress_update = Instant::now();
        let carve_hits = engine
            .carve_with_progress_and_cancel(
                &mut evidence,
                |pct| {
                    if last_progress_update.elapsed().as_millis() >= 100 || pct >= 1.0 {
                        if let Ok(mut a) = app_progress.lock() {
                            a.carver.progress = pct * 0.7; // First 70% is disk sector scanning
                            let pct_lbl = (pct * 100.0).clamp(0.0, 100.0);
                            a.set_status(format!("Sector scan: {:.1}% complete", pct_lbl));
                        }
                        last_progress_update = Instant::now();
                    }
                },
                &carver_cancel,
            )
            .unwrap_or_default();

        if let Ok(mut a) = app.lock() {
            a.carver.progress = 0.7;
            a.carver.log.push(format!("Sector scan finished: {} candidate hit(s)", carve_hits.len()));
            a.set_status(format!("Found {} candidate hits — parsing file structures...", carve_hits.len()));
        }

        if !inspect_only {
            let _ = std::fs::create_dir_all(&output_dir);
        }
        let mut recovered_items = Vec::new();

        // Mode: 0 = Hybrid, 1 = Deep Structural, 2 = Filesystem Metadata, 3 = Entropy Stream, 4 = Magic
        let use_fs = mode_idx == 0 || mode_idx == 2;
        let use_struct = mode_idx == 0 || mode_idx == 1;
        let use_entropy = mode_idx == 0 || mode_idx == 3;
        let use_magic = mode_idx == 0 || mode_idx == 4;

        // 1. Filesystem Metadata Inode Carving
        if use_fs && !carver_cancel.load(Ordering::Relaxed) {
            let mut fs_buf = vec![0u8; 1024 * 1024 * 8];
            let read_ok = evidence
                .seek(SeekFrom::Start(0))
                .ok()
                .and_then(|_| evidence.read(&mut fs_buf).ok());
            if let Some(n) = read_ok {
                for (_off, meta) in scan_ntfs_mft_records(&fs_buf[..n], 0) {
                    if carver_cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Some(payload) = meta.resident_bytes {
                        let safe_name = std::path::Path::new(&meta.filename)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("resident_file");
                        let fname = format!("recovered_{}", safe_name);
                        let fpath = format!("{}/{}", output_dir, fname);
                        if !inspect_only {
                            let _ = std::fs::write(&fpath, &payload);
                        }
                        let ent = shannon_entropy(&payload);
                        let file_sha = Some(sih149_core::audit::signing::hash_bytes(&payload));
                        let item = CarvedFile {
                            path: if inspect_only {
                                fname.clone()
                            } else {
                                fpath.clone()
                            },
                            file_type: meta.file_extension.clone(),
                            size_bytes: payload.len() as u64,
                            confidence: 95,
                            entropy: ent,
                            method: "Filesystem MFT Inode".to_string(),
                            metadata: format!(
                                "Created: {} | Name: {}",
                                meta.created_at.unwrap_or_default(),
                                meta.filename
                            ),
                            offset_bytes: _off,
                            selected: false,
                        };
                        recovered_items.push(item.clone());
                        if let Ok(mut a) = app.lock() {
                            a.carver.log.push(format!("Found MFT Resident: {}", fname));
                            if a.carver.log.len() > 50 {
                                a.carver.log.remove(0);
                            }
                            a.carver.found.push(item);
                            let count = a.carver.found.len();
                            a.set_status(format!("Found {} carved files...", count));
                        }
                        if let Some(d) = db.as_ref() {
                            let rec = sih149_core::db::CarvedFileRecord {
                                filename: fname,
                                file_type: meta.file_extension.clone(),
                                size_bytes: payload.len() as u64,
                                confidence: 95,
                                offset_bytes: _off,
                                output_path: if inspect_only { String::new() } else { fpath },
                                category: meta.file_extension.to_uppercase(),
                                sha256: file_sha,
                            };
                            let _ = d.insert_carved_file(&case_id, &rec);
                        }
                    }
                }
                for (_off, meta) in scan_fat_directory_entries(&fs_buf[..n], 0) {
                    if carver_cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    let safe_name = std::path::Path::new(&meta.filename)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("fat_entry");
                    let fname = format!("recovered_{}", safe_name);
                    let out_path = format!("{}/{}", output_dir, fname);
                    let method_label = if meta.attributes.contains_key("FAT_Name_Type") {
                        "Filesystem FAT LFN".to_string()
                    } else {
                        "Filesystem FAT DirEntry".to_string()
                    };
                    let item = CarvedFile {
                        path: if inspect_only {
                            fname.clone()
                        } else {
                            out_path.clone()
                        },
                        file_type: meta.file_extension.clone(),
                        size_bytes: meta.file_size_bytes,
                        confidence: 90,
                        entropy: 6.0,
                        method: method_label,
                        metadata: format!(
                            "Modified: {} | Cluster: {}",
                            meta.modified_at.unwrap_or_default(),
                            meta.attributes
                                .get("Start_Cluster")
                                .cloned()
                                .unwrap_or_default()
                        ),
                        offset_bytes: _off,
                        selected: false,
                    };
                    recovered_items.push(item.clone());
                    if let Ok(mut a) = app.lock() {
                        a.carver.log.push(format!("Found FAT DirEntry: {}", fname));
                        if a.carver.log.len() > 50 {
                            a.carver.log.remove(0);
                        }
                        a.carver.found.push(item);
                        let count = a.carver.found.len();
                        a.set_status(format!("Found {} carved files...", count));
                    }
                    if let Some(d) = db.as_ref() {
                        let rec = sih149_core::db::CarvedFileRecord {
                            filename: fname,
                            file_type: meta.file_extension.clone(),
                            size_bytes: meta.file_size_bytes,
                            confidence: 90,
                            offset_bytes: _off,
                            output_path: if inspect_only {
                                String::new()
                            } else {
                                out_path
                            },
                            category: meta.file_extension.to_uppercase(),
                            sha256: None,
                        };
                        let _ = d.insert_carved_file(&case_id, &rec);
                    }
                }
            }
        }

        // 2. Signature and Structural Carving
        if (use_magic || use_struct) && !carver_cancel.load(Ordering::Relaxed) {
            let total_hits = carve_hits.len();
            let mut last_sync = Instant::now();
            for (idx, (offset, confidence, ext)) in carve_hits.iter().enumerate() {
                if carver_cancel.load(Ordering::Relaxed) {
                    break;
                }

                let conf_pct = match confidence {
                    sih149_core::carver::confidence::Confidence::Low => 40,
                    sih149_core::carver::confidence::Confidence::Medium => 65,
                    sih149_core::carver::confidence::Confidence::High => 90,
                    sih149_core::carver::confidence::Confidence::Absolute => 100,
                };
                if conf_pct < min_confidence {
                    continue;
                }

                let filename = format!("carved_{}_{:010}.{}", ext, offset, ext);
                let filepath = format!("{}/{}", output_dir, filename);

                let mut buf = vec![0u8; 1024 * 1024 * 4];
                let bytes_read = evidence
                    .seek(SeekFrom::Start(*offset))
                    .ok()
                    .and_then(|_| evidence.read(&mut buf).ok())
                    .unwrap_or(0);

                let mut method = "Signature Magic".to_string();
                let mut meta_str = String::new();
                let mut final_len = bytes_read;

                if bytes_read > 0 && use_struct {
                    match ext.as_str() {
                        "zip" | "docx" | "xlsx" | "pptx" | "apk" | "jar" => {
                            if let Some(zm) = parse_zip_metadata(&buf[..bytes_read]) {
                                let doc_type_desc = zm
                                    .document_type
                                    .as_deref()
                                    .map(|dt| format!(" ({})", dt))
                                    .unwrap_or_default();
                                method = format!("Structural (ZIP EOCD{})", doc_type_desc);
                                meta_str = format!(
                                    "{} internal files | {} B",
                                    zm.file_count, zm.total_uncompressed_bytes
                                );
                                if zm.exact_archive_size > 0
                                    && (zm.exact_archive_size as usize) <= bytes_read
                                {
                                    final_len = zm.exact_archive_size as usize;
                                }
                            }
                        }
                        "sqlite" | "db" | "sqlite3" => {
                            if let Some(sm) = parse_sqlite_metadata(&buf[..bytes_read]) {
                                method = "Structural (SQLite B-Tree)".to_string();
                                meta_str = format!(
                                    "Page size: {} B | Tables: [{}]",
                                    sm.page_size,
                                    sm.table_names.join(", ")
                                );
                                if sm.exact_db_size > 0 && (sm.exact_db_size as usize) <= bytes_read
                                {
                                    final_len = sm.exact_db_size as usize;
                                }
                            }
                        }
                        "png" => {
                            if let Some(pm) = parse_png_metadata(&buf[..bytes_read]) {
                                method = "Structural (PNG Chunks)".to_string();
                                meta_str = format!(
                                    "{}x{} px | Tags: {}",
                                    pm.width,
                                    pm.height,
                                    pm.text_tags.len()
                                );
                                if pm.exact_file_size > 0
                                    && (pm.exact_file_size as usize) <= bytes_read
                                {
                                    final_len = pm.exact_file_size as usize;
                                }
                            }
                        }
                        "jpg" | "jpeg" => {
                            if let Some(jm) = parse_jpeg_metadata(&buf[..bytes_read]) {
                                method = "Structural (JPEG EXIF)".to_string();
                                meta_str = format!(
                                    "Camera: {} | Date: {}",
                                    jm.camera_make.unwrap_or_default(),
                                    jm.date_time.unwrap_or_default()
                                );
                                if jm.exact_file_size > 0
                                    && (jm.exact_file_size as usize) <= bytes_read
                                {
                                    final_len = jm.exact_file_size as usize;
                                }
                            }
                        }
                        "pdf" => {
                            if let Some(pm) = parse_pdf_metadata(&buf[..bytes_read]) {
                                method = "Structural (PDF Trailer)".to_string();
                                meta_str = format!(
                                    "PDF-{} | Author: {}",
                                    pm.version,
                                    pm.author.unwrap_or_default()
                                );
                                if pm.exact_file_size > 0
                                    && (pm.exact_file_size as usize) <= bytes_read
                                {
                                    final_len = pm.exact_file_size as usize;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if bytes_read > 0 && !inspect_only {
                    let _ = std::fs::write(&filepath, &buf[..final_len]);
                }

                let ent = if final_len > 0 {
                    shannon_entropy(&buf[..final_len])
                } else {
                    0.0
                };

                let item = CarvedFile {
                    path: if inspect_only {
                        filename.clone()
                    } else {
                        filepath.clone()
                    },
                    file_type: ext.clone(),
                    size_bytes: final_len as u64,
                    confidence: conf_pct,
                    entropy: ent,
                    method,
                    metadata: meta_str,
                    offset_bytes: *offset,
                    selected: false,
                };
                recovered_items.push(item.clone());

                if last_sync.elapsed().as_millis() >= 100 || idx + 1 == total_hits {
                    if let Ok(mut a) = app.lock() {
                        let hit_ratio = if total_hits > 0 { (idx as f64 / total_hits as f64).clamp(0.0, 1.0) } else { 1.0 };
                        a.carver.progress = 0.7 + 0.25 * hit_ratio;
                        a.carver.found = recovered_items.clone();
                        a.carver.log.push(format!("Carved {}: {} (0x{:X})", ext.to_uppercase(), filename, offset));
                        if a.carver.log.len() > 50 {
                            a.carver.log.remove(0);
                        }
                        let pct_val = a.carver.progress * 100.0;
                        a.set_status(format!("Validating hits: {}/{} ({:.1}%)", idx + 1, total_hits, pct_val));
                    }
                    last_sync = Instant::now();
                }

                if let Some(d) = db.as_ref() {
                    let file_sha = if final_len > 0 && !inspect_only {
                        Some(sih149_core::audit::signing::hash_bytes(&buf[..final_len]))
                    } else {
                        None
                    };
                    let rec = sih149_core::db::CarvedFileRecord {
                        filename: filename.clone(),
                        file_type: ext.clone(),
                        size_bytes: final_len as u64,
                        confidence: conf_pct,
                        offset_bytes: *offset,
                        output_path: if inspect_only {
                            String::new()
                        } else {
                            filepath
                        },
                        category: ext.to_uppercase(),
                        sha256: file_sha,
                    };
                    let _ = d.insert_carved_file(&case_id, &rec);
                }
            }
        }

        // 3. Statistical Entropy Stream Clustering
        if use_entropy && !carver_cancel.load(Ordering::Relaxed) {
            let mut ent_buf = vec![0u8; 1024 * 1024 * 8];
            let read_n = evidence
                .seek(SeekFrom::Start(0))
                .ok()
                .and_then(|_| evidence.read(&mut ent_buf).ok())
                .unwrap_or(0);
            if read_n > 0 {
                let clusters = cluster_entropy_streams(&ent_buf[..read_n], 0, 4096);
                for c in clusters.iter().take(3) {
                    if carver_cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    if c.cluster_type == EntropyType::CompressedOrEncrypted && c.size_bytes >= 8192
                    {
                        let stream_name = format!("entropy_stream_{:010}.bin", c.start_offset);
                        let stream_path = format!("{}/{}", output_dir, stream_name);
                        let slice_len = std::cmp::min(
                            c.size_bytes as usize,
                            read_n.saturating_sub(c.start_offset as usize),
                        );
                        if slice_len > 0 {
                            if !inspect_only {
                                let _ = std::fs::write(
                                    &stream_path,
                                    &ent_buf[c.start_offset as usize
                                        ..c.start_offset as usize + slice_len],
                                );
                            }
                            let item = CarvedFile {
                                path: if inspect_only {
                                    stream_name.clone()
                                } else {
                                    stream_path.clone()
                                },
                                file_type: "bin".to_string(),
                                size_bytes: slice_len as u64,
                                confidence: 75,
                                entropy: c.avg_entropy,
                                method: "Entropy Stream (>7.2b)".to_string(),
                                metadata: format!(
                                    "Avg Entropy: {:.2} b/B | Continuous Block",
                                    c.avg_entropy
                                ),
                                offset_bytes: c.start_offset,
                                selected: false,
                            };
                            recovered_items.push(item.clone());
                            if let Ok(mut a) = app.lock() {
                                a.carver.log.push(format!("Carved Entropy Stream: {}", stream_name));
                                if a.carver.log.len() > 50 {
                                    a.carver.log.remove(0);
                                }
                                a.carver.found.push(item);
                                let count = a.carver.found.len();
                                a.set_status(format!("Found {} carved files...", count));
                            }
                            if let Some(d) = db.as_ref() {
                                let rec = sih149_core::db::CarvedFileRecord {
                                    filename: stream_name,
                                    file_type: "bin".to_string(),
                                    size_bytes: slice_len as u64,
                                    confidence: 75,
                                    offset_bytes: c.start_offset,
                                    output_path: if inspect_only {
                                        String::new()
                                    } else {
                                        stream_path
                                    },
                                    category: "BINARY".to_string(),
                                    sha256: None,
                                };
                                let _ = d.insert_carved_file(&case_id, &rec);
                            }
                        }
                    }
                }
            }
        }

        let is_cancelled = carver_cancel.load(Ordering::Relaxed);
        let mut a = app.lock().unwrap();
        a.carver.scanning = false;
        a.carver.progress = 1.0;
        a.carver.found = recovered_items;
        let count = a.carver.found.len();
        if is_cancelled {
            a.push_log_level(
                crate::app::LogLevel::Warning,
                format!("Carving cancelled: {} files recovered before stop", count),
            );
            a.set_status(format!("Carving cancelled ({} files)", count));
            if let Some(d) = db.as_ref() {
                let _ = d.update_case_status(&case_id, "aborted");
                let audit_done = serde_json::json!({
                    "event": "carve_aborted",
                    "files_recovered": count,
                })
                .to_string();
                let hash = sih149_core::audit::signing::hash_bytes(audit_done.as_bytes());
                let _ = d.insert_audit_entry(&case_id, &audit_done, &hash);
            }
        } else {
            a.push_log_level(
                crate::app::LogLevel::Success,
                format!("Carving complete: {} files recovered", count),
            );
            a.set_status(format!("Carving complete: {} files", count));
            if let Some(d) = db.as_ref() {
                let _ = d.update_case_status(&case_id, "completed");
                let audit_done = serde_json::json!({
                    "event": "carve_completed",
                    "files_recovered": count,
                })
                .to_string();
                let hash = sih149_core::audit::signing::hash_bytes(audit_done.as_bytes());
                let _ = d.insert_audit_entry(&case_id, &audit_done, &hash);
            }
        }
    });
}

pub fn start_surface_scan(app: Arc<Mutex<App>>, drive_idx: usize, cells: usize) {
    std::thread::spawn(move || {
        let (device_path, drive_name) = {
            let mut a = app.lock().unwrap();
            a.surface_map.scanning = true;
            match a.drives.get(drive_idx) {
                Some(d) => (d.path.clone(), d.name.clone()),
                None => {
                    a.surface_map.scanning = false;
                    return;
                }
            }
        };

        {
            let mut a = app.lock().unwrap();
            a.push_log_level(
                crate::app::LogLevel::Info,
                format!("Starting surface & cluster scan: /dev/{}", drive_name),
            );
            a.set_status(format!("Scanning surface: /dev/{}", drive_name));
        }

        let result = sih149_core::disk::surface_scanner::scan_surface(&device_path, cells);
        let mut a = app.lock().unwrap();
        a.surface_map.scanning = false;
        match result {
            Ok(res) => {
                a.push_log_level(
                    crate::app::LogLevel::Success,
                    format!(
                        "Surface scan complete: Health {:.1}%, {} good, {} bad sectors",
                        res.health_score, res.good_count, res.bad_count
                    ),
                );
                a.set_status(format!("Surface Scan: {:.1}% Health", res.health_score));
                a.surface_map.scan_result = Some(res);
            }
            Err(error) => {
                a.push_log_level(
                    crate::app::LogLevel::Error,
                    format!("Surface scan failed: {}", error),
                );
                a.set_status("Surface scan failed".to_string());
            }
        }
    });
}

pub fn extract_selected_carver_files(
    app: Arc<Mutex<App>>,
    source: String,
    output_dir: String,
    file_indices: Vec<usize>,
) {
    std::thread::spawn(move || {
        let files_to_extract: Vec<CarvedFile> = {
            let a = app.lock().unwrap();
            file_indices
                .iter()
                .filter_map(|&idx| a.carver.found.get(idx).cloned())
                .collect()
        };

        if files_to_extract.is_empty() {
            let mut a = app.lock().unwrap();
            a.push_log_level(
                crate::app::LogLevel::Warning,
                "No files selected to extract",
            );
            a.set_status("No files selected to extract");
            return;
        }

        let _ = std::fs::create_dir_all(&output_dir);

        let mut evidence = match EvidenceSource::open(&source) {
            Ok(d) => d,
            Err(e) => {
                let mut a = app.lock().unwrap();
                a.push_log_level(
                    crate::app::LogLevel::Error,
                    format!("Open failed for {}: {}", source, e),
                );
                a.set_status("Evidence open failed");
                return;
            }
        };

        let mut success_count = 0usize;
        for file in &files_to_extract {
            let filename = std::path::Path::new(&file.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("carved_file");
            let out_file_path = std::path::Path::new(&output_dir).join(filename);

            let mut buf = vec![0u8; file.size_bytes as usize];
            if evidence.seek(SeekFrom::Start(file.offset_bytes)).is_ok()
                && evidence.read_exact(&mut buf).is_ok()
                && std::fs::write(&out_file_path, &buf).is_ok()
            {
                success_count += 1;
            }
        }

        let mut a = app.lock().unwrap();
        a.push_log_level(
            crate::app::LogLevel::Success,
            format!(
                "Extracted {}/{} files to {}",
                success_count,
                files_to_extract.len(),
                output_dir
            ),
        );
        a.set_status(format!(
            "Extracted {} file(s) to {}",
            success_count, output_dir
        ));
    });
}
