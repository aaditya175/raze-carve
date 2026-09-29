use serde::{Deserialize, Serialize};
use sih149_core::carver::engine::CarvingEngine;
use sih149_core::carver::entropy_clustering::{cluster_entropy_streams, EntropyType};
use sih149_core::carver::filesystem_records::{scan_fat_directory_entries, scan_ntfs_mft_records};
use sih149_core::carver::scanner::SectorScanner;
use sih149_core::carver::signatures::SignatureDatabase;
use sih149_core::carver::structure::{
    jpeg::parse_jpeg_metadata, pdf::parse_pdf_metadata, png::parse_png_metadata,
    sqlite::parse_sqlite_metadata, zip::parse_zip_metadata,
};
use sih149_core::disk::EvidenceSource;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub source_path: String,
    pub output_dir: String,
    pub file_types: Vec<String>,
    pub min_confidence: u8,
    #[serde(default)]
    pub recovery_modes: Option<Vec<String>>,
    #[serde(default)]
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRequest {
    pub source_path: String,
    pub output_dir: String,
    pub files: Vec<CarvedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractResult {
    pub extracted_count: u32,
    pub total_size_bytes: u64,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarvedFile {
    pub id: String,
    pub filename: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub confidence: u8,
    pub offset_bytes: u64,
    pub category: String,
    pub recovery_method: Option<String>,
    pub original_metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub sector_current: u64,
    pub sector_total: u64,
    pub percent: f32,
    pub files_found: u32,
    pub speed_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub total_files: u32,
    pub total_size_bytes: u64,
    pub duration_secs: u64,
    pub entropy_heatmap: Vec<f32>,
    pub files: Vec<CarvedFile>,
    #[serde(default)]
    pub case_id: Option<String>,
}

lazy_static::lazy_static! {
    static ref SCAN_CANCEL_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

/// Resolve a relative evidence/file path that may be hidden under the crate
/// while the app runs from a different working directory (`cargo tauri dev`
/// runs with CWD = `<root>/src-tauri`, so `tests/fixtures/*` would otherwise
/// not resolve). Absolute paths and block devices pass through unchanged.
pub fn resolve_evidence_path(path: &str) -> std::path::PathBuf {
    let p = std::path::PathBuf::from(path);
    if p.is_absolute() || p.exists() {
        return p;
    }
    // Try the current working directory, then its parent (project root), and
    // finally the crate source dir as candidate bases.
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.clone());
        candidates.push(cwd.join(".."));
        candidates.push(cwd.join("..").join(".."));
    }
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_dir = std::path::PathBuf::from(manifest);
        candidates.push(manifest_dir.clone());
        candidates.push(manifest_dir.join(".."));
    }
    for base in candidates {
        let candidate = base.join(path);
        if candidate.exists() {
            return candidate;
        }
    }
    p
}

/// Resolve a project-relative directory (e.g. `plugins/signatures`) the same
/// way as the evidence path above, defaulting to the raw path when not found.
fn resolve_project_child_path(path: &str) -> std::path::PathBuf {
    let installed_root = if let Some(relative) = path.strip_prefix("plugins/") {
        std::env::var_os("SECUREFORGE_PLUGINS_DIR")
            .map(std::path::PathBuf::from)
            .map(|root| root.join(relative))
    } else {
        None
    };
    if let Some(candidate) = installed_root.filter(|candidate| candidate.exists()) {
        return candidate;
    }
    let resolved = resolve_evidence_path(path);
    if resolved == *path && Path::new(path).is_relative() {
        // Fall back to the given path even if it could not be re-anchored.
        std::path::PathBuf::from(path)
    } else {
        resolved
    }
}

/// Loads signatures from TOML directory with built-in fallbacks
fn load_signature_db(signatures_dir: &Path) -> Result<SignatureDatabase, String> {
    let mut signatures = sih149_core::carver::signatures::SignatureDatabase::builtin().signatures;

    if signatures_dir.is_dir() {
        if let Ok(toml_sigs) =
            sih149_core::plugins::toml_loader::load_signatures_from_dir(signatures_dir)
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

    if signatures.is_empty() {
        signatures = sih149_core::carver::signatures::SignatureDatabase::builtin().signatures;
    }

    Ok(SignatureDatabase { signatures })
}

#[tauri::command]
pub async fn start_scan(config: ScanConfig, app_handle: AppHandle) -> Result<ScanResult, String> {
    // IPC input validation: refuse empty paths, cap file-types count, and
    // keep min_confidence in a sane range to prevent excessive resource use.
    if config.source_path.trim().is_empty() {
        return Err("source_path must not be empty".to_string());
    }
    if config.output_dir.trim().is_empty() {
        return Err("output_dir must not be empty".to_string());
    }
    if config.file_types.len() > 32 {
        return Err("file_types list too large (max 32)".to_string());
    }
    if config.min_confidence > 100 {
        return Err("min_confidence must be between 0 and 100".to_string());
    }
    if config.source_path.len() > 4096 || config.output_dir.len() > 4096 {
        return Err("path arguments too long".to_string());
    }

    SCAN_CANCEL_FLAG.store(false, Ordering::SeqCst);

    let app_handle_bg = app_handle.clone();
    let config_bg = config.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<ScanResult, String> {
        let progress = |p: ScanProgress| {
            let _ = app_handle_bg.emit("scan-progress", p);
        };
        run_scan(&config_bg, progress)
    })
    .await
    .map_err(|e| format!("Scan task panicked: {}", e))??;

    Ok(result)
}

/// Perform a carving scan against a block device or raw image. Shared by the
/// Tauri IPC command and the HTTP server route so both modes behave identically.
pub fn run_scan(
    config: &ScanConfig,
    mut on_progress: impl FnMut(ScanProgress),
) -> Result<ScanResult, String> {
    let signatures_dir = resolve_project_child_path("plugins/signatures");
    let sig_db = load_signature_db(&signatures_dir)?;
    let signatures = sig_db.signatures.clone();
    let scanner = SectorScanner::new(sig_db).map_err(|e| e.to_string())?;

    let mut lua_host = sih149_core::plugins::lua_host::LuaPluginHost::new();
    let scripts_dir = resolve_project_child_path("plugins/scripts");
    if scripts_dir.is_dir() {
        let _ = lua_host.load_directory(&scripts_dir);
    }

    let engine = if lua_host.plugin_count() > 0 {
        CarvingEngine::new(scanner).with_lua_plugins(lua_host)
    } else {
        CarvingEngine::new(scanner)
    };

    let started = Instant::now();
    let case_id = format!("scan-{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let _ = crate::commands::reports::create_desktop_case(
        &case_id,
        "Forensic Analyst",
        &config.source_path,
        "recovery",
    );
    let _ = crate::commands::reports::record_desktop_audit_event(
        &case_id,
        "recovery_scan_started",
        serde_json::json!({
            "source": config.source_path,
            "output_dir": config.output_dir,
            "file_types": config.file_types,
            "min_confidence": config.min_confidence,
        }),
    );

    let source_path = resolve_evidence_path(&config.source_path);

    on_progress(ScanProgress {
        sector_current: 0,
        sector_total: 100,
        percent: 0.0,
        files_found: 0,
        speed_mbps: 0.0,
    });

    // Enforce software write-blocking and transparent E01 / block device / raw image handling
    let mut evidence = EvidenceSource::open(&source_path).map_err(|e| {
        format!(
            "Failed to open {} in read-only write-blocked mode: {}",
            source_path.display(),
            e
        )
    })?;

    let carve_hits = engine
        .carve_with_progress_and_cancel(
            &mut evidence,
            |pct| {
                on_progress(ScanProgress {
                    sector_current: (pct * 100.0) as u64,
                    sector_total: 100,
                    percent: (pct * 30.0) as f32,
                    files_found: 0,
                    speed_mbps: 0.0,
                });
            },
            &SCAN_CANCEL_FLAG,
        )
        .map_err(|e| e.to_string())?;

    if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
        let _ = crate::commands::reports::update_desktop_case_status(&case_id, "cancelled");
        let _ = crate::commands::reports::record_desktop_audit_event(
            &case_id,
            "recovery_scan_cancelled",
            serde_json::json!({ "phase": "signature_scan" }),
        );
        return Err("Scan cancelled by user".to_string());
    }

    let is_dry_run = config.dry_run.unwrap_or(false);
    if !is_dry_run && !config.output_dir.trim().is_empty() {
        std::fs::create_dir_all(&config.output_dir).map_err(|e| e.to_string())?;
    }
    let out_dir = Path::new(&config.output_dir);

    use std::io::{Read, Seek, SeekFrom, Write};
    let mut files = Vec::new();
    let mut total_size_bytes: u64 = 0;
    let min_confidence = config.min_confidence;

    let active_modes: Vec<String> = config
        .recovery_modes
        .clone()
        .unwrap_or_else(|| vec!["hybrid".to_string()]);
    let use_fs = active_modes
        .iter()
        .any(|m| m == "filesystem" || m == "hybrid");
    let use_structure = active_modes
        .iter()
        .any(|m| m == "structure" || m == "hybrid");
    let use_entropy = active_modes.iter().any(|m| m == "entropy" || m == "hybrid");

    // 1. Filesystem Metadata & Inode Carving (NTFS $MFT + FAT32 DirRecords)
    if use_fs {
        let mut fs_buf = vec![0u8; 1024 * 1024 * 8];
        let fs_read = evidence
            .seek(SeekFrom::Start(0))
            .ok()
            .and_then(|_| evidence.read(&mut fs_buf).ok());
        if let Some(n) = fs_read {
            let mft_hits = scan_ntfs_mft_records(&fs_buf[..n], 0);
            for (mft_off, meta) in mft_hits {
                if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
                    break;
                }
                if let Some(payload) = meta.resident_bytes {
                    let safe_name = std::path::Path::new(&meta.filename)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("resident_file");
                    let out_name = format!("recovered_{}", safe_name);
                    let out_file_path = out_dir.join(&out_name);
                    if !is_dry_run {
                        if let Err(error) = std::fs::write(&out_file_path, &payload) {
                            tracing::warn!(
                                path = %out_file_path.display(),
                                %error,
                                "Skipping NTFS resident hit because recovered payload could not be written"
                            );
                            continue;
                        }
                    }
                    total_size_bytes += payload.len() as u64;
                    files.push(CarvedFile {
                        id: Uuid::new_v4().to_string(),
                        filename: out_name,
                        file_type: meta.file_extension.clone(),
                        size_bytes: payload.len() as u64,
                        confidence: 95,
                        offset_bytes: mft_off,
                        category: "Filesystem Records".to_string(),
                        recovery_method: Some("Filesystem Metadata (NTFS $MFT Inode)".to_string()),
                        original_metadata: Some(format!(
                            "Created: {} | Modified: {} | File: {}",
                            meta.created_at.unwrap_or_default(),
                            meta.modified_at.unwrap_or_default(),
                            meta.filename
                        )),
                    });
                }
            }
            let fat_hits = scan_fat_directory_entries(&fs_buf[..n], 0);
            for (fat_off, meta) in fat_hits {
                if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
                    break;
                }
                let safe_name = std::path::Path::new(&meta.filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("fat_entry");
                let method_desc = meta
                    .attributes
                    .get("FAT_Name_Type")
                    .cloned()
                    .unwrap_or_else(|| "FAT32 DirEntry".to_string());
                tracing::info!(
                    filename = %safe_name,
                    offset = fat_off,
                    method = %method_desc,
                    "FAT metadata located but not reported as recovered: this scanner has no validated cluster-chain extraction for the payload"
                );
            }
        }
    }

    // 2. Signature and Deep Structural Carving
    let total_hits = carve_hits.len();
    let mut last_progress_emit = Instant::now();

    for (hit_idx, (offset, confidence, extension)) in carve_hits.iter().enumerate() {
        if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
            let _ = crate::commands::reports::update_desktop_case_status(&case_id, "cancelled");
            let _ = crate::commands::reports::record_desktop_audit_event(
                &case_id,
                "recovery_scan_cancelled",
                serde_json::json!({ "extracted_files": files.len(), "hit_index": hit_idx }),
            );
            return Err("Scan cancelled by user".to_string());
        }

        let confidence_pct: u8 = match confidence {
            sih149_core::carver::confidence::Confidence::Low => 40,
            sih149_core::carver::confidence::Confidence::Medium => 65,
            sih149_core::carver::confidence::Confidence::High => 90,
            sih149_core::carver::confidence::Confidence::Absolute => 100,
        };
        if confidence_pct < min_confidence {
            continue;
        }
        if !config.file_types.is_empty() && !config.file_types.contains(extension) {
            continue;
        }

        let filename = format!("carved_{}_{:010}.{}", extension, offset, extension);
        let out_file_path = out_dir.join(&filename);

        // Read up to 8MB from evidence at offset
        let mut buf = vec![0u8; 1024 * 1024 * 8];
        let read_res = evidence
            .seek(SeekFrom::Start(*offset))
            .ok()
            .and_then(|_| evidence.read(&mut buf).ok());

        let mut method_str = "Signature Magic Header".to_string();
        let mut metadata_str: Option<String> = None;
        let mut file_len = match read_res {
            Some(0) => {
                tracing::warn!(
                    offset = *offset,
                    "Skipping signature hit because no evidence bytes were readable"
                );
                continue;
            }
            Some(n) => n,
            None => {
                tracing::warn!(
                    offset = *offset,
                    "Skipping signature hit because the evidence read failed"
                );
                continue;
            }
        };
        {
            let n = file_len;
            let mut matched_structure = false;

            // Deep structural syntax parsing & metadata extraction (prioritized over naive footers)
            if use_structure {
                match extension.as_str() {
                    "zip" | "docx" | "xlsx" | "pptx" | "apk" | "jar" => {
                        if let Some(zmeta) = parse_zip_metadata(&buf[..n]) {
                            if zmeta.exact_archive_size > 0
                                && (zmeta.exact_archive_size as usize) <= n
                            {
                                file_len = zmeta.exact_archive_size as usize;
                                matched_structure = true;
                            }
                            method_str = "Deep Structural (ZIP Central Directory)".to_string();
                            let doc = zmeta
                                .document_type
                                .unwrap_or_else(|| "ZIP Archive".to_string());
                            let previews = zmeta
                                .contained_files
                                .iter()
                                .take(3)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ");
                            metadata_str = Some(format!(
                                "{} | {} files: [{}] | Uncompressed: {} B",
                                doc, zmeta.file_count, previews, zmeta.total_uncompressed_bytes
                            ));
                        }
                    }
                    "sqlite" | "db" | "sqlite3" => {
                        if let Some(smeta) = parse_sqlite_metadata(&buf[..n]) {
                            if smeta.exact_db_size > 0 && (smeta.exact_db_size as usize) <= n {
                                file_len = smeta.exact_db_size as usize;
                                matched_structure = true;
                            }
                            method_str = "Deep Structural (SQLite B-Tree Header)".to_string();
                            metadata_str = Some(format!(
                                "Page Size: {} B | Pages: {} | Tables: [{}]",
                                smeta.page_size,
                                smeta.page_count,
                                smeta.table_names.join(", ")
                            ));
                        }
                    }
                    "png" => {
                        if let Some(pmeta) = parse_png_metadata(&buf[..n]) {
                            if pmeta.exact_file_size > 0 && (pmeta.exact_file_size as usize) <= n {
                                file_len = pmeta.exact_file_size as usize;
                                matched_structure = true;
                            }
                            method_str = "Deep Structural (PNG Chunks)".to_string();
                            metadata_str = Some(format!(
                                "{}x{} px | Bit Depth: {} | Text Tags: {}",
                                pmeta.width,
                                pmeta.height,
                                pmeta.bit_depth,
                                pmeta.text_tags.len()
                            ));
                        }
                    }
                    "jpg" | "jpeg" => {
                        if let Some(jmeta) = parse_jpeg_metadata(&buf[..n]) {
                            if jmeta.exact_file_size > 0 && (jmeta.exact_file_size as usize) <= n {
                                file_len = jmeta.exact_file_size as usize;
                                matched_structure = true;
                            }
                            method_str = "Deep Structural (JPEG EXIF/Markers)".to_string();
                            metadata_str = Some(format!(
                                "Camera: {} | Date: {} | Comments: {}",
                                jmeta.camera_make.unwrap_or_else(|| "Standard".to_string()),
                                jmeta.date_time.unwrap_or_else(|| "N/A".to_string()),
                                jmeta.comments.len()
                            ));
                        }
                    }
                    "pdf" => {
                        if let Some(pdfmeta) = parse_pdf_metadata(&buf[..n]) {
                            if pdfmeta.exact_file_size > 0
                                && (pdfmeta.exact_file_size as usize) <= n
                            {
                                file_len = pdfmeta.exact_file_size as usize;
                                matched_structure = true;
                            }
                            method_str = "Deep Structural (PDF Trailer & Info)".to_string();
                            metadata_str = Some(format!(
                                "Version: PDF-{} | Title: {} | Author: {}",
                                pdfmeta.version,
                                pdfmeta.title.unwrap_or_else(|| "N/A".to_string()),
                                pdfmeta.author.unwrap_or_else(|| "N/A".to_string())
                            ));
                        }
                    }
                    _ => {}
                }
            }

            // Fallback to magic footer delimiter only if structural parsing did not determine boundary
            if !matched_structure {
                let sig = signatures.iter().find(|s| s.extension == *extension);
                if let Some(Some(footer_hex)) = sig.map(|s| &s.magic_footer) {
                    if let Ok(footer_bytes) = SignatureDatabase::parse_hex(footer_hex) {
                        if !footer_bytes.is_empty() {
                            if let Some(pos) = buf[..n]
                                .windows(footer_bytes.len())
                                .position(|w| w == footer_bytes.as_slice())
                            {
                                file_len = pos + footer_bytes.len();
                                method_str = "Header & Footer Signature Delimiter".to_string();
                            }
                        }
                    }
                }
            }

            if !is_dry_run {
                let write_result = (|| -> std::io::Result<()> {
                    let mut file = std::fs::File::create(&out_file_path)?;
                    file.write_all(&buf[..file_len])?;
                    file.sync_all()
                })();
                if let Err(error) = write_result {
                    tracing::warn!(
                        path = %out_file_path.display(),
                        %error,
                        "Skipping signature hit because recovered payload could not be written"
                    );
                    continue;
                }
            }
        }

        let category = match extension.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "Images".to_string(),
            "pdf" | "doc" | "docx" | "txt" => "Documents".to_string(),
            "zip" | "tar" | "gz" | "7z" => "Archives".to_string(),
            "sqlite" | "db" => "Databases".to_string(),
            _ => extension.to_uppercase(),
        };

        total_size_bytes += file_len as u64;

        files.push(CarvedFile {
            id: Uuid::new_v4().to_string(),
            filename,
            file_type: extension.clone(),
            size_bytes: file_len as u64,
            confidence: confidence_pct,
            offset_bytes: *offset,
            category,
            recovery_method: Some(method_str),
            original_metadata: metadata_str,
        });

        if last_progress_emit.elapsed().as_millis() >= 200 || hit_idx + 1 == total_hits {
            let elapsed = started.elapsed().as_secs_f32().max(0.001);
            let pct = 30.0 + ((hit_idx + 1) as f32 / total_hits.max(1) as f32) * 60.0;
            on_progress(ScanProgress {
                sector_current: (hit_idx + 1) as u64,
                sector_total: total_hits as u64,
                percent: pct.min(95.0),
                files_found: files.len() as u32,
                speed_mbps: (total_size_bytes as f32 / elapsed) / (1024.0 * 1024.0),
            });
            last_progress_emit = Instant::now();
        }
    }

    // 3. Statistical Entropy Stream Clustering (Headerless/Fragmented Data)
    if use_entropy {
        let mut ent_buf = vec![0u8; 1024 * 1024 * 8];
        let ent_read = evidence
            .seek(SeekFrom::Start(0))
            .ok()
            .and_then(|_| evidence.read(&mut ent_buf).ok());
        if let Some(n) = ent_read {
            let clusters = cluster_entropy_streams(&ent_buf[..n], 0, 4096);
            for c in clusters.iter().take(5) {
                if c.cluster_type == EntropyType::CompressedOrEncrypted && c.size_bytes >= 8192 {
                    let covered = files
                        .iter()
                        .any(|f| (f.offset_bytes as i64 - c.start_offset as i64).abs() < 4096);
                    if !covered {
                        let stream_name = format!("entropy_stream_{:010}.bin", c.start_offset);
                        let stream_path = out_dir.join(&stream_name);
                        let slice_len = std::cmp::min(
                            c.size_bytes as usize,
                            n.saturating_sub(c.start_offset as usize),
                        );
                        if slice_len > 0 {
                            if !is_dry_run {
                                if let Err(error) = std::fs::write(
                                    &stream_path,
                                    &ent_buf[c.start_offset as usize
                                        ..c.start_offset as usize + slice_len],
                                ) {
                                    tracing::warn!(
                                        path = %stream_path.display(),
                                        %error,
                                        "Skipping entropy stream because recovered payload could not be written"
                                    );
                                    continue;
                                }
                            }
                            total_size_bytes += slice_len as u64;
                            files.push(CarvedFile {
                                id: Uuid::new_v4().to_string(),
                                filename: stream_name,
                                file_type: "bin".to_string(),
                                size_bytes: slice_len as u64,
                                confidence: 75,
                                offset_bytes: c.start_offset,
                                category: "Raw Streams".to_string(),
                                recovery_method: Some("Statistical Entropy Stream (Shannon > 7.2)".to_string()),
                                original_metadata: Some(format!("Avg Entropy: {:.2} b/B | Continuous Compressed/Encrypted Block", c.avg_entropy)),
                            });
                        }
                    }
                }
            }
        }
    }

    if let Ok(db_path) = crate::commands::reports::get_db_path() {
        if let Ok(db) = sih149_core::db::CaseDatabase::open(&db_path) {
            for f in &files {
                let file_path = out_dir.join(&f.filename);
                let sha256 = if !is_dry_run && file_path.exists() {
                    sih149_core::audit::signing::hash_file(&file_path).ok()
                } else {
                    None
                };
                let rec = sih149_core::db::CarvedFileRecord {
                    filename: f.filename.clone(),
                    file_type: f.file_type.clone(),
                    size_bytes: f.size_bytes,
                    confidence: f.confidence,
                    offset_bytes: f.offset_bytes,
                    output_path: if is_dry_run {
                        String::new()
                    } else {
                        file_path.to_string_lossy().into_owned()
                    },
                    category: f.category.clone(),
                    sha256,
                };
                let _ = db.insert_carved_file(&case_id, &rec);
            }
        }
    }

    let duration_secs = started.elapsed().as_secs();
    let _ = crate::commands::reports::update_desktop_case_status(&case_id, "completed");
    let _ = crate::commands::reports::record_desktop_audit_event(
        &case_id,
        "recovery_scan_completed",
        serde_json::json!({
            "total_files": files.len(),
            "total_size_bytes": total_size_bytes,
            "duration_secs": duration_secs,
        }),
    );

    // Automatically generate recovery certificate PDF in cases/<case_id>/<case_id>_certificate.pdf
    let rpt_dir = format!("cases/{}", case_id);
    let _ = std::fs::create_dir_all(&rpt_dir);
    let rpt_file = format!("{}/{}_certificate.pdf", rpt_dir, case_id);
    let _ = crate::commands::reports::export_report(case_id.clone(), "recovery".into(), rpt_file, None);

    let elapsed = started.elapsed().as_secs_f32().max(0.001);
    on_progress(ScanProgress {
        sector_current: 100,
        sector_total: 100,
        percent: 100.0,
        files_found: files.len() as u32,
        speed_mbps: 100.0 / elapsed,
    });

    Ok(ScanResult {
        total_files: files.len() as u32,
        total_size_bytes,
        duration_secs,
        entropy_heatmap: vec![0.1, 0.4, 0.8],
        files,
        case_id: Some(case_id),
    })
}

#[tauri::command]
pub fn cancel_scan() -> Result<(), String> {
    SCAN_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn get_file_hex_preview(
    file_path: String,
    offset: u64,
    length: usize,
    allowed_root: Option<String>,
) -> Result<String, String> {
    get_file_hex_preview_str(&file_path, offset, length, allowed_root.as_deref())
}

/// Shared hex-preview implementation used by the Tauri command and the HTTP
/// `/api/hex` route so both modes return identical results.
pub fn get_file_hex_preview_str(
    file_path: &str,
    offset: u64,
    length: usize,
    allowed_root: Option<&str>,
) -> Result<String, String> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::Path;

    let path = Path::new(file_path);
    let resolved = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve {}: {}", file_path, e))?;

    if let Some(root_str) = allowed_root {
        if !root_str.is_empty() {
            let root = Path::new(root_str)
                .canonicalize()
                .map_err(|e| format!("Invalid allowed_root: {}", e))?;
            if !resolved.starts_with(&root) {
                return Err(format!(
                    "Access denied: {} is outside the permitted case directory {}",
                    file_path, root_str
                ));
            }
        }
    }

    // Read-only inspection of a selected block device is allowed for forensic
    // acquisition. Other /dev paths remain protected from arbitrary preview.
    let is_block_device = sih149_core::disk::block_device::is_block_device(&resolved);
    if sih149_core::wiper::file_wiper::is_protected_path(&resolved) && !is_block_device {
        return Err(format!(
            "Access denied: protected system path: {}",
            file_path
        ));
    }

    let resolved_str = resolved.to_string_lossy();
    if resolved_str.contains("/.ssh")
        || resolved_str.contains("/.gnupg")
        || resolved_str.contains("/.aws")
        || resolved_str.ends_with("/id_rsa")
        || resolved_str.ends_with("/id_ed25519")
        || resolved_str.contains("/.bash_history")
        || resolved_str.contains("/.zsh_history")
    {
        return Err(format!(
            "Access denied: sensitive security credential path: {}",
            file_path
        ));
    }

    let mut file =
        File::open(&resolved).map_err(|e| format!("Failed to open {}: {}", file_path, e))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| e.to_string())?;
    let len = length.clamp(16, 4096);
    let mut buf = vec![0u8; len];
    let n = file.read(&mut buf).map_err(|e| e.to_string())?;
    buf.truncate(n);
    let hex = buf
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");
    Ok(hex)
}

#[tauri::command]
pub fn extract_selected_files(request: ExtractRequest) -> Result<ExtractResult, String> {
    use std::fs::{self, File};
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::path::Path;

    let out_dir = Path::new(&request.output_dir);
    if !out_dir.as_os_str().is_empty() {
        fs::create_dir_all(out_dir).map_err(|e| format!("Failed to create output dir: {}", e))?;
    }

    let source_path = Path::new(&request.source_path);
    let resolved_src = source_path.canonicalize().map_err(|e| {
        format!(
            "Failed to resolve source path {}: {}",
            request.source_path, e
        )
    })?;

    let mut evidence = File::open(&resolved_src).map_err(|e| {
        format!(
            "Failed to open evidence image {}: {}",
            request.source_path, e
        )
    })?;

    let mut extracted_names = Vec::new();
    let mut total_size_bytes = 0u64;

    for file_info in &request.files {
        let safe_filename = Path::new(&file_info.filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("recovered_file");
        let dest_path = out_dir.join(safe_filename);

        evidence
            .seek(SeekFrom::Start(file_info.offset_bytes))
            .map_err(|e| format!("Seek failed for {}: {}", file_info.filename, e))?;

        let mut dest_file = File::create(&dest_path)
            .map_err(|e| format!("Failed to create {}: {}", dest_path.display(), e))?;

        let mut remaining = file_info.size_bytes;
        let mut chunk = vec![0u8; 64 * 1024];
        while remaining > 0 {
            let to_read = (remaining as usize).min(chunk.len());
            let n = evidence
                .read(&mut chunk[..to_read])
                .map_err(|e| format!("Read error for {}: {}", file_info.filename, e))?;
            if n == 0 {
                break;
            }
            dest_file
                .write_all(&chunk[..n])
                .map_err(|e| format!("Write error for {}: {}", dest_path.display(), e))?;
            remaining = remaining.saturating_sub(n as u64);
        }
        dest_file.sync_all().map_err(|e| e.to_string())?;

        let actual_size = file_info.size_bytes - remaining;
        total_size_bytes += actual_size;
        extracted_names.push(safe_filename.to_string());
    }

    Ok(ExtractResult {
        extracted_count: extracted_names.len() as u32,
        total_size_bytes,
        files: extracted_names,
    })
}
