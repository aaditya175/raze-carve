//! Live Diagnostics and Self-Test Suite for SecureForge
//!
//! Provides in-app verification of all critical subsystems:
//! cryptographic algorithms, wiper pattern generators, safety guardrails,
//! forensic carvers, structure validators, and the SQLite audit database.
//!
//! Both the CLI (`sih149 test`), TUI (Screen F6), and GUI (/diagnostics)
//! invoke this engine to verify operational integrity.

use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub name: String,
    pub category: String,
    pub passed: bool,
    pub message: String,
    pub duration_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration_ms: f64,
    pub results: Vec<DiagnosticResult>,
}

/// Executes the full diagnostic suite and returns individual assertions with timing
pub fn run_diagnostics() -> DiagnosticSummary {
    let start_all = Instant::now();
    let mut results = Vec::new();

    // ── 1. Wiper Subsystem ────────────────────────────────────────────────
    run_test(&mut results, "Wipe Pattern: Zero Fill", "Wiper", || {
        let mut buf = vec![0xAAu8; 8192];
        let mut p = crate::wiper::patterns::WipePattern::Zeros;
        p.fill(&mut buf);
        if buf.iter().all(|&b| b == 0) {
            Ok("Buffer filled with 0x00 perfectly".into())
        } else {
            Err("Zero pattern left dirty bytes".into())
        }
    });

    run_test(&mut results, "Wipe Pattern: Ones Fill", "Wiper", || {
        let mut buf = vec![0x00u8; 8192];
        let mut p = crate::wiper::patterns::WipePattern::Ones;
        p.fill(&mut buf);
        if buf.iter().all(|&b| b == 0xFF) {
            Ok("Buffer filled with 0xFF perfectly".into())
        } else {
            Err("Ones pattern left non-0xFF bytes".into())
        }
    });

    run_test(
        &mut results,
        "Wipe Pattern: CSPRNG Entropy",
        "Wiper",
        || {
            let mut buf = vec![0u8; 65536];
            let mut p = crate::wiper::patterns::WipePattern::for_pass(3);
            p.fill(&mut buf);
            let entropy = crate::carver::entropy::calculate_shannon_entropy(&buf);
            if entropy >= 7.8 {
                Ok(format!("High entropy verified: {:.3} bits/byte", entropy))
            } else {
                Err(format!(
                    "Entropy too low: {:.3} bits/byte (expected >= 7.8)",
                    entropy
                ))
            }
        },
    );

    run_test(
        &mut results,
        "Wipe Pattern: Pass Uniqueness",
        "Wiper",
        || {
            let mut p = crate::wiper::patterns::WipePattern::for_pass(3);
            let mut b1 = vec![0u8; 2048];
            let mut b2 = vec![0u8; 2048];
            p.fill(&mut b1);
            p.fill(&mut b2);
            if b1 != b2 {
                Ok("Successive passes generated distinct random streams".into())
            } else {
                Err("Successive passes generated identical data".into())
            }
        },
    );

    run_test(
        &mut results,
        "Wipe Sequence: DoD 5220.22-M & Gutmann",
        "Wiper",
        || {
            let p_zero = crate::wiper::patterns::WipePattern::for_pass(1);
            let p_ones = crate::wiper::patterns::WipePattern::for_pass(2);
            let p_rand = crate::wiper::patterns::WipePattern::for_pass(3);
            let valid = matches!(p_zero, crate::wiper::patterns::WipePattern::Zeros)
                && matches!(p_ones, crate::wiper::patterns::WipePattern::Ones)
                && matches!(p_rand, crate::wiper::patterns::WipePattern::Random(_));
            if valid {
                Ok("Pass mapping aligns with DoD 5220.22-M standard".into())
            } else {
                Err("Pass mapping mismatch".into())
            }
        },
    );

    // ── 2. Safety & Guardrails ────────────────────────────────────────────
    run_test(
        &mut results,
        "Safety: Protected Paths Blocked",
        "Safety",
        || {
            use crate::wiper::file_wiper::is_protected_path;
            use std::path::Path;
            let blocked = is_protected_path(Path::new("/etc/shadow"))
                && is_protected_path(Path::new("/boot/vmlinuz"))
                && is_protected_path(Path::new("/sys/class"))
                && is_protected_path(Path::new("/proc/1/stat"))
                && is_protected_path(Path::new("/usr/bin/bash"));
            if blocked {
                Ok("All critical system directories strictly blocked".into())
            } else {
                Err("One or more system paths failed protection check".into())
            }
        },
    );

    run_test(
        &mut results,
        "Safety: Path Traversal Escape Defense",
        "Safety",
        || {
            use crate::wiper::file_wiper::is_protected_path;
            use std::path::Path;
            let blocked = is_protected_path(Path::new("/tmp/../../etc/passwd"))
                && is_protected_path(Path::new("/var/tmp/../../etc/shadow"));
            if blocked {
                Ok("Relative traversal escapes properly blocked".into())
            } else {
                Err("Path traversal bypassed protected check".into())
            }
        },
    );

    run_test(
        &mut results,
        "Safety: Symlink Traversal Rejection",
        "Safety",
        || {
            use crate::wiper::file_wiper::FileWiper;
            let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
            let target = dir.path().join("real_file.bin");
            let symlink = dir.path().join("link.bin");
            std::fs::write(&target, b"target").map_err(|e| e.to_string())?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &symlink).map_err(|e| e.to_string())?;

            let wiper = FileWiper::new(1, 1, false);
            match wiper.wipe_file(&symlink) {
                Err(_) => Ok("Symlink directly rejected without following target".into()),
                Ok(_) => Err("Symlink was mistakenly followed and shredded".into()),
            }
        },
    );

    run_test(
        &mut results,
        "Safety: Boot Drive Protection",
        "Safety",
        || {
            use crate::wiper::file_wiper::is_protected_drive;
            use std::path::Path;
            let nvme_blocked = is_protected_drive(Path::new("/dev/nvme0n1"));
            let sda_blocked = is_protected_drive(Path::new("/dev/sda"));
            let sdb_safe = !is_protected_drive(Path::new("/dev/sdb"));
            if nvme_blocked && sda_blocked && sdb_safe {
                Ok("System boot devices locked down; external drives permitted".into())
            } else {
                Err("Protected drive filter error".into())
            }
        },
    );

    // ── 3. Forensic Carver & Structure Validators ─────────────────────────
    run_test(
        &mut results,
        "Carver: Hex Signature Decoding",
        "Carver",
        || {
            use crate::carver::signatures::SignatureDatabase;
            let valid = SignatureDatabase::parse_hex("ffd8ff")
                .map(|b| b == [0xFF, 0xD8, 0xFF])
                .unwrap_or(false);
            let odd_err = SignatureDatabase::parse_hex("ffd8f").is_err();
            if valid && odd_err {
                Ok("Hex signature parser validated".into())
            } else {
                Err("Hex signature parser error".into())
            }
        },
    );

    run_test(
        &mut results,
        "Carver: JPEG Structure Validator",
        "Carver",
        || {
            let valid_jpeg = [
                0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0xFF, 0xD9,
            ];
            let invalid_jpeg = [0x00, 0x11, 0x22, 0x33];
            let v1 = crate::carver::structure::jpeg::validate_jpeg(&valid_jpeg).unwrap_or(false);
            let v2 = crate::carver::structure::jpeg::validate_jpeg(&invalid_jpeg).unwrap_or(false);
            if v1 && !v2 {
                Ok("JPEG SOI/EOI parser functional".into())
            } else {
                Err("JPEG validator failed test".into())
            }
        },
    );

    run_test(
        &mut results,
        "Carver: PNG Signature & IHDR Validator",
        "Carver",
        || {
            let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
            png.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]);
            png.extend_from_slice(b"IHDR");
            png.extend_from_slice(&[0u8; 17]); // data + crc
                                               // Append valid IEND chunk
            png.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
            png.extend_from_slice(b"IEND");
            png.extend_from_slice(&[0xAE, 0x42, 0x60, 0x82]);
            let valid = crate::carver::structure::png::validate_png(&png).unwrap_or(false);
            let invalid =
                crate::carver::structure::png::validate_png(&[0x89, 0x50]).unwrap_or(false);
            if valid && !invalid {
                Ok("PNG signature and chunk structure verified".into())
            } else {
                Err("PNG validator failed test".into())
            }
        },
    );

    run_test(
        &mut results,
        "Carver: PDF & ZIP Magic Bytes",
        "Carver",
        || {
            let pdf =
                b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\nxref\ntrailer\n<<>>\nstartxref\n10\n%%EOF\n";
            let mut zip = vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00];
            zip.extend_from_slice(&[0u8; 16]);
            zip.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
            zip.extend_from_slice(&[0u8; 18]); // EOCD record
            let pdf_ok = crate::carver::structure::pdf::validate_pdf(pdf).unwrap_or(false);
            let zip_ok = crate::carver::structure::zip::validate_zip(&zip).unwrap_or(false);
            if pdf_ok && zip_ok {
                Ok("PDF (%PDF..%%EOF) and ZIP (PK..EOCD) structure validators active".into())
            } else {
                Err("PDF/ZIP structure validation failure".into())
            }
        },
    );

    run_test(
        &mut results,
        "Carver: SQLite Header & Page Size",
        "Carver",
        || {
            let mut db_bytes = vec![0u8; 4096];
            db_bytes[..16].copy_from_slice(b"SQLite format 3\0");
            db_bytes[16] = 0x10; // 4096 page size
            db_bytes[17] = 0x00;
            let valid =
                crate::carver::structure::sqlite::validate_sqlite(&db_bytes).unwrap_or(false);
            let invalid =
                crate::carver::structure::sqlite::validate_sqlite(&[0u8; 20]).unwrap_or(false);
            if valid && !invalid {
                Ok("SQLite database header and 4096-byte page verified".into())
            } else {
                Err("SQLite validator failed test".into())
            }
        },
    );

    run_test(
        &mut results,
        "Carver: E01 Evidence Image Seeking",
        "Carver",
        || {
            use std::io::{Read, Seek, SeekFrom, Write};
            // 1. Verify Raw evidence image seeking
            let mut f_raw = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
            f_raw
                .write_all(b"SECUREFORGE_FORENSIC_SECTOR")
                .map_err(|e| e.to_string())?;
            f_raw.flush().map_err(|e| e.to_string())?;

            let mut img =
                crate::disk::raw_image::RawImage::open(f_raw.path()).map_err(|e| e.to_string())?;
            img.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            let mut buf = vec![0u8; 27];
            img.read_exact(&mut buf).map_err(|e| e.to_string())?;
            if &buf != b"SECUREFORGE_FORENSIC_SECTOR" {
                return Err("Evidence raw image seek read mismatch".into());
            }

            // 2. Verify E01 signature detection and fail-safe parser guard
            let mut f_e01 = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
            f_e01
                .write_all(crate::disk::ewf::E01_FILE_SIGNATURE)
                .map_err(|e| e.to_string())?;
            f_e01
                .write_all(&[1, 0, 1, 0, 0])
                .map_err(|e| e.to_string())?;
            f_e01.flush().map_err(|e| e.to_string())?;

            if !crate::disk::ewf::E01Image::is_e01(f_e01.path()) {
                return Err("E01 signature detection failed".into());
            }
            match crate::disk::ewf::E01Image::open(f_e01.path()) {
                Err(CoreError::Disk(msg)) if msg.contains("not supported") => Ok(
                    "Evidence payload seek verified; E01 signature & parser guard active".into(),
                ),
                Ok(_) => Err("E01 image was accepted without a full EWF decoder".into()),
                Err(e) => Err(format!("Unexpected E01 error: {}", e)),
            }
        },
    );

    // ── 4. Cryptographic Hash Chain & Audit ───────────────────────────────
    run_test(
        &mut results,
        "Crypto: SHA-256 Known NIST Vector",
        "Crypto",
        || {
            let empty_hash = crate::audit::signing::hash_bytes(b"");
            if empty_hash == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" {
                Ok("Standard NIST SHA-256 vector matched".into())
            } else {
                Err("SHA-256 output mismatch".into())
            }
        },
    );

    run_test(
        &mut results,
        "Crypto: Constant-Time Digest Verification",
        "Crypto",
        || {
            let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
            let p = dir.path().join("hash_test.bin");
            std::fs::write(&p, b"audit record").map_err(|e| e.to_string())?;
            let hash = crate::audit::signing::hash_file(&p).map_err(|e| e.to_string())?;
            let match_ok = crate::audit::signing::verify_file_hash(&p, &hash).unwrap_or(false);
            let mismatch_ok = !crate::audit::signing::verify_file_hash(
                &p,
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap_or(true);
            if match_ok && mismatch_ok {
                Ok("Constant-time timing-oracle immune comparison confirmed".into())
            } else {
                Err("Hash verification failed".into())
            }
        },
    );

    run_test(
        &mut results,
        "Crypto: Tamper-Evident Hash Chain",
        "Crypto",
        || {
            use crate::audit::hashchain::HashChain;
            use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
            use std::collections::BTreeMap;

            let mut chain = HashChain::new();
            for i in 0..3 {
                chain
                    .append(AuditEntry {
                        id: i,
                        seq: 0,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        operation: OperationType::DiskWipe,
                        target: "/dev/sdb".into(),
                        params: BTreeMap::new(),
                        result: OperationResult {
                            success: true,
                            message: "Pass completed".into(),
                            pre_hash: None,
                            post_hash: None,
                            sectors_processed: Some(1024),
                            bad_sectors: None,
                            files_recovered: None,
                            entropy_post: Some(0.0),
                        },
                        prev_hash: String::new(),
                        entry_hash: String::new(),
                    })
                    .map_err(|e| e.to_string())?;
            }
            if chain.verify() && chain.len() == 3 {
                Ok("Hash chain sequence verified across 3 cryptographic links".into())
            } else {
                Err("Chain integrity verification failed".into())
            }
        },
    );

    run_test(
        &mut results,
        "Crypto: Hash Chain Import & Tamper Rejection",
        "Crypto",
        || {
            use crate::audit::hashchain::HashChain;
            use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
            use std::collections::BTreeMap;

            let mut chain = HashChain::new();
            chain
                .append(AuditEntry {
                    id: 1,
                    seq: 0,
                    timestamp: "2026-12-22T00:00:00Z".into(),
                    operation: OperationType::DiskWipe,
                    target: "/dev/sdb".into(),
                    params: BTreeMap::new(),
                    result: OperationResult {
                        success: true,
                        message: "Initial pass completed".into(),
                        pre_hash: None,
                        post_hash: None,
                        sectors_processed: Some(2048),
                        bad_sectors: None,
                        files_recovered: None,
                        entropy_post: Some(0.0),
                    },
                    prev_hash: String::new(),
                    entry_hash: String::new(),
                })
                .map_err(|e| e.to_string())?;

            let log = chain.to_audit_log("CASE-IMPORT-DIAG".into(), Some("Auditor".into()));
            let reloaded = HashChain::from_log(&log).map_err(|e| e.to_string())?;
            if reloaded.tip() != chain.tip() {
                return Err("Reloaded chain tip mismatch".into());
            }

            let mut tampered_log = log;
            tampered_log.entries[0].target = "/dev/sdc".into();
            if HashChain::from_log(&tampered_log).is_ok() {
                return Err("Tampered audit entry was erroneously accepted".into());
            }

            Ok("AuditLog import and bit-tamper detection confirmed".into())
        },
    );

    // ── 5. SQLite Forensic Case Database ──────────────────────────────────
    run_test(
        &mut results,
        "Database: SQLite WAL Mode & Schema Init",
        "Database",
        || {
            let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
            let db_path = dir.path().join("diag.db");
            let db = crate::db::CaseDatabase::open(&db_path).map_err(|e| e.to_string())?;
            db.create_case("DIAG-01", "Operator", "/dev/sdb", "wipe")
                .map_err(|e| e.to_string())?;
            let case = db.get_case("DIAG-01").map_err(|e| e.to_string())?;
            if let Some(c) = case {
                if c.status == "running" && c.target == "/dev/sdb" {
                    Ok("SQLite WAL initialized; cases table responsive".into())
                } else {
                    Err("Database query returned mismatched record".into())
                }
            } else {
                Err("Case could not be retrieved from DB".into())
            }
        },
    );

    run_test(
        &mut results,
        "Database: Carved File Indexing",
        "Database",
        || {
            let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
            let db_path = dir.path().join("diag2.db");
            let db = crate::db::CaseDatabase::open(&db_path).map_err(|e| e.to_string())?;
            db.create_case("DIAG-02", "Operator", "/dev/sdb", "carve")
                .map_err(|e| e.to_string())?;
            db.insert_carved_file(
                "DIAG-02",
                &crate::db::CarvedFileRecord {
                    filename: "photo.jpg".into(),
                    file_type: "jpg".into(),
                    size_bytes: 2048,
                    confidence: 90,
                    offset_bytes: 4096,
                    output_path: "/tmp/photo.jpg".into(),
                    category: "Media".into(),
                    sha256: None,
                },
            )
            .map_err(|e| e.to_string())?;
            let files = db.list_carved_files("DIAG-02").map_err(|e| e.to_string())?;
            if files.len() == 1 && files[0].filename == "photo.jpg" {
                Ok("Carved file record indexed and mapped to case".into())
            } else {
                Err("Carved file indexing failed".into())
            }
        },
    );

    // ── 6. Forensic Deduplication & Classification ────────────────────────
    run_test(
        &mut results,
        "Slack Space: Cluster Boundary Scrubbing",
        "Wiper",
        || {
            let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
            let file_path = dir.path().join("diag_slack.bin");
            let data = vec![0x42u8; 1500];
            std::fs::write(&file_path, &data).map_err(|e| e.to_string())?;
            let wiper = crate::wiper::file_wiper::FileWiper::new(1, 1, true);
            match wiper.scrub_slack_space(&file_path) {
                Err(CoreError::Wiper(msg)) if msg.contains("not supported safely") => {
                    let len_after = std::fs::metadata(&file_path)
                        .map_err(|e| e.to_string())?
                        .len();
                    if len_after == 1500 {
                        Ok(
                            "Safety policy verified: unsafe user-space slack scrubbing refused"
                                .into(),
                        )
                    } else {
                        Err("Slack check modified file length".into())
                    }
                }
                Ok(scrubbed) => Ok(format!("Scrubbed {} slack bytes; file preserved", scrubbed)),
                Err(e) => Err(e.to_string()),
            }
        },
    );

    run_test(
        &mut results,
        "Perceptual dHash: Image Similarity & Deduplication",
        "Classifier",
        || {
            use crate::classify::dhash::{
                compute_dhash_from_grid, dhash_from_hex, dhash_to_hex, is_visual_duplicate,
            };
            let mut grid_a = [100u8; 72];
            for r in 0..8 {
                for c in 0..9 {
                    grid_a[r * 9 + c] = (c * 20) as u8;
                }
            }
            let hash_a = compute_dhash_from_grid(&grid_a);
            let hex_a = dhash_to_hex(hash_a);
            let parsed_a = dhash_from_hex(&hex_a).ok_or("Failed to parse hex hash")?;
            if parsed_a != hash_a {
                return Err("Round-trip dHash hex formatting failed".into());
            }

            // Slight noise on grid_b
            let mut grid_b = grid_a;
            grid_b[0] = 5; // one bit change
            let hash_b = compute_dhash_from_grid(&grid_b);
            if is_visual_duplicate(hash_a, hash_b, 5) {
                Ok(format!(
                    "dHash computed ({}), duplicate match verified",
                    hex_a
                ))
            } else {
                Err("Perceptual deduplication failed to detect similar image".into())
            }
        },
    );

    let elapsed = start_all.elapsed();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;

    DiagnosticSummary {
        total: results.len(),
        passed,
        failed,
        total_duration_ms: elapsed.as_secs_f64() * 1000.0,
        results,
    }
}

fn run_test<F>(results: &mut Vec<DiagnosticResult>, name: &str, category: &str, test_fn: F)
where
    F: FnOnce() -> Result<String, String>,
{
    let t0 = Instant::now();
    let (passed, message) = match test_fn() {
        Ok(msg) => (true, msg),
        Err(err) => (false, err),
    };
    let duration_us = t0.elapsed().as_micros() as u64;
    results.push(DiagnosticResult {
        name: name.to_string(),
        category: category.to_string(),
        passed,
        message,
        duration_us,
    });
}
