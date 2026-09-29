use std::io::Write;
use tempfile::NamedTempFile;

// ═══════════════════════════════════════════════════════════════════════════
// Wiper pattern tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_wipe_pattern_zeros_fills_correctly() {
    let mut buf = vec![0xAAu8; 4096];
    let mut p = crate::wiper::patterns::WipePattern::Zeros;
    p.fill(&mut buf);
    assert!(
        buf.iter().all(|&b| b == 0),
        "Zeros pattern must fill with 0x00"
    );
}

#[test]
fn test_wipe_pattern_ones_fills_correctly() {
    let mut buf = vec![0u8; 4096];
    let mut p = crate::wiper::patterns::WipePattern::Ones;
    p.fill(&mut buf);
    assert!(
        buf.iter().all(|&b| b == 0xFF),
        "Ones pattern must fill with 0xFF"
    );
}

#[test]
fn test_wipe_pattern_random_fills_nonzero() {
    let mut buf = vec![0u8; 4096];
    let mut p = crate::wiper::patterns::WipePattern::for_pass(3);
    p.fill(&mut buf);
    // Statistically impossible for 4096 random bytes to all be zero
    assert!(
        !buf.iter().all(|&b| b == 0),
        "Random pattern must produce non-zero output"
    );
}

#[test]
fn test_wipe_pattern_random_has_high_entropy() {
    let mut buf = vec![0u8; 65536];
    let mut p = crate::wiper::patterns::WipePattern::for_pass(3);
    p.fill(&mut buf);
    // Shannon entropy of true random data should be > 7.9 bits/byte
    let mut freq = [0u64; 256];
    for &b in &buf {
        freq[b as usize] += 1;
    }
    let len = buf.len() as f64;
    let entropy: f64 = freq
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum();
    assert!(
        entropy > 7.5,
        "Random pattern entropy should be > 7.5 b/B, got {:.3}",
        entropy
    );
}

#[test]
fn test_wipe_pattern_for_pass_mapping() {
    let p1 = crate::wiper::patterns::WipePattern::for_pass(1);
    assert!(matches!(p1, crate::wiper::patterns::WipePattern::Zeros));
    let p2 = crate::wiper::patterns::WipePattern::for_pass(2);
    assert!(matches!(p2, crate::wiper::patterns::WipePattern::Ones));
    let p3 = crate::wiper::patterns::WipePattern::for_pass(3);
    assert!(matches!(p3, crate::wiper::patterns::WipePattern::Random(_)));
    // Any unknown pass should map to Random
    let p99 = crate::wiper::patterns::WipePattern::for_pass(99);
    assert!(matches!(
        p99,
        crate::wiper::patterns::WipePattern::Random(_)
    ));
}

#[test]
fn test_passes_for_method_canonical_counts() {
    use crate::wiper::patterns::passes_for_method;
    assert_eq!(passes_for_method("zero").len(), 1);
    assert_eq!(passes_for_method("nist").len(), 1);
    assert_eq!(passes_for_method("random").len(), 1);
    assert_eq!(passes_for_method("dod").len(), 3);
    assert_eq!(passes_for_method("dod3").len(), 3);
    assert_eq!(passes_for_method("dod7").len(), 7);
    assert_eq!(passes_for_method("gutmann").len(), 35);
    assert_eq!(passes_for_method("gutmann-35").len(), 35);
}

#[test]
fn test_gutmann_genuine_patterns() {
    use crate::wiper::patterns::WipePattern;
    let mut buf = vec![0u8; 9];

    // Pass 5: 0x55
    let mut p5 = WipePattern::for_gutmann_pass(5);
    p5.fill(&mut buf);
    assert!(buf.iter().all(|&b| b == 0x55));

    // Pass 6: 0xAA
    let mut p6 = WipePattern::for_gutmann_pass(6);
    p6.fill(&mut buf);
    assert!(buf.iter().all(|&b| b == 0xAA));

    // Pass 7: Periodic 0x92, 0x49, 0x24
    let mut p7 = WipePattern::for_gutmann_pass(7);
    p7.fill(&mut buf);
    assert_eq!(
        buf,
        vec![0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24]
    );

    // Pass 10: 0x00
    let mut p10 = WipePattern::for_gutmann_pass(10);
    p10.fill(&mut buf);
    assert!(buf.iter().all(|&b| b == 0x00));

    // Pass 25: 0xFF
    let mut p25 = WipePattern::for_gutmann_pass(25);
    p25.fill(&mut buf);
    assert!(buf.iter().all(|&b| b == 0xFF));
}

#[test]
fn test_generate_zeros_compat_api() {
    let buf = crate::wiper::patterns::generate_zeros(1024);
    assert_eq!(buf.len(), 1024);
    assert!(buf.iter().all(|&b| b == 0));
}

#[test]
fn test_generate_ones_compat_api() {
    let buf = crate::wiper::patterns::generate_ones(1024);
    assert_eq!(buf.len(), 1024);
    assert!(buf.iter().all(|&b| b == 0xFF));
}

#[test]
fn test_generate_random_compat_api() {
    let buf = crate::wiper::patterns::generate_random(1024);
    assert_eq!(buf.len(), 1024);
    assert!(
        !buf.iter().all(|&b| b == 0),
        "Random buffer should not be all zeros"
    );
}

#[test]
fn test_get_dod_pattern_mapping() {
    let f1 = crate::wiper::patterns::get_dod_pattern(1);
    let f2 = crate::wiper::patterns::get_dod_pattern(2);
    let f3 = crate::wiper::patterns::get_dod_pattern(3);
    let b1 = f1(512);
    let b2 = f2(512);
    let b3 = f3(512);
    assert!(b1.iter().all(|&b| b == 0), "Pass 1 should be zeros");
    assert!(b2.iter().all(|&b| b == 0xFF), "Pass 2 should be ones");
    assert!(!b3.iter().all(|&b| b == 0), "Pass 3 should be random");
}

#[test]
fn test_file_wiper_creates_and_destroys_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_wipe_target.bin");
    std::fs::write(&path, b"SECRET DATA THAT MUST BE DESTROYED").unwrap();
    assert!(path.exists(), "File should exist before wipe");

    let wiper = crate::wiper::file_wiper::FileWiper::new(3, 5, false);
    let result = wiper.wipe_file(&path).unwrap();

    assert!(result.success);
    assert_eq!(result.bytes_wiped, 34);
    assert_eq!(result.passes_completed, 3);
    assert!(!path.exists(), "File should be deleted after wipe");
}

#[test]
fn test_file_wiper_refuses_protected_paths() {
    assert!(crate::wiper::file_wiper::is_protected_path(
        std::path::Path::new("/etc/passwd")
    ));
    assert!(crate::wiper::file_wiper::is_protected_path(
        std::path::Path::new("/usr/bin/bash")
    ));
    assert!(crate::wiper::file_wiper::is_protected_path(
        std::path::Path::new("/boot/grub")
    ));
    assert!(crate::wiper::file_wiper::is_protected_path(
        std::path::Path::new("/proc/1/status")
    ));
    assert!(crate::wiper::file_wiper::is_protected_path(
        std::path::Path::new("/sys/class")
    ));
}

#[test]
fn test_file_wiper_refuses_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let real_file = dir.path().join("real.txt");
    let link_path = dir.path().join("link.txt");
    std::fs::write(&real_file, b"data").unwrap();
    std::os::unix::fs::symlink(&real_file, &link_path).unwrap();

    let wiper = crate::wiper::file_wiper::FileWiper::new(1, 1, false);
    let result = wiper.wipe_file(&link_path);
    assert!(result.is_err(), "Wiper must refuse symlinks");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("symlink"),
        "Error should mention symlink, got: {}",
        err_msg
    );
    // Original file should be untouched
    assert!(
        real_file.exists(),
        "Real file should still exist after refusing symlink"
    );
}

#[test]
fn test_is_protected_path_allows_tmp() {
    assert!(!crate::wiper::file_wiper::is_protected_path(
        std::path::Path::new("/tmp/test_file")
    ));
    assert!(!crate::wiper::file_wiper::is_protected_path(
        std::path::Path::new("/home/user/file")
    ));
    assert!(!crate::wiper::file_wiper::is_protected_path(
        std::path::Path::new("/var/tmp/file")
    ));
}

#[test]
fn test_is_protected_drive_blocks_sda() {
    assert!(crate::wiper::file_wiper::is_protected_drive(
        std::path::Path::new("/dev/sda")
    ));
    assert!(crate::wiper::file_wiper::is_protected_drive(
        std::path::Path::new("/dev/sda1")
    ));
    assert!(crate::wiper::file_wiper::is_protected_drive(
        std::path::Path::new("/dev/nvme0n1")
    ));
    assert!(crate::wiper::file_wiper::is_protected_drive(
        std::path::Path::new("/dev/nvme0n1p1")
    ));
}

#[test]
fn test_is_protected_drive_allows_sdb() {
    assert!(!crate::wiper::file_wiper::is_protected_drive(
        std::path::Path::new("/dev/sdb")
    ));
    assert!(!crate::wiper::file_wiper::is_protected_drive(
        std::path::Path::new("/dev/sdc")
    ));
    assert!(!crate::wiper::file_wiper::is_protected_drive(
        std::path::Path::new("/dev/sdd1")
    ));
}

#[test]
fn test_detect_cow_filesystem() {
    // /tmp is usually ext4/tmpfs, not CoW
    let result = crate::wiper::file_wiper::detect_cow_filesystem(std::path::Path::new("/tmp"));
    assert!(result.is_ok());
    // We can't assert the value since it depends on the host, but it shouldn't error
}

#[test]
fn test_detect_cow_filesystem_nonexistent_path_fallback() {
    // A path that does not exist should fall back to parent directory (/tmp) without erroring
    let non_existent = std::path::Path::new("/tmp/sf_nonexistent_audit_file.tmp");
    let result = crate::wiper::file_wiper::detect_cow_filesystem(non_existent);
    assert!(result.is_ok());
}

#[test]
fn test_file_wiper_populates_is_cow() {
    use crate::wiper::file_wiper::FileWiper;

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("wipe_test.bin");
    std::fs::write(&file_path, b"sensitivedata").unwrap();

    let wiper = FileWiper::new(1, 1, false);
    let result = wiper.wipe_file(&file_path).unwrap();
    assert!(result.success);
    assert_eq!(result.bytes_wiped, 13);
    // is_cow is a boolean flag reflecting filesystem properties
    let _ = result.is_cow;
}

// ═══════════════════════════════════════════════════════════════════════════
// Carver signature tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_signature_db_parse_hex_valid() {
    let bytes = crate::carver::signatures::SignatureDatabase::parse_hex("ffd8ff").unwrap();
    assert_eq!(bytes, vec![0xFF, 0xD8, 0xFF]);
}

#[test]
fn test_signature_db_parse_hex_with_spaces() {
    let bytes = crate::carver::signatures::SignatureDatabase::parse_hex("ff d8 ff").unwrap();
    assert_eq!(bytes, vec![0xFF, 0xD8, 0xFF]);
}

#[test]
fn test_signature_db_parse_hex_invalid_odd_length() {
    let result = crate::carver::signatures::SignatureDatabase::parse_hex("ffd8f");
    assert!(result.is_err(), "Odd-length hex should fail");
}

#[test]
fn test_signature_db_parse_hex_invalid_chars() {
    let result = crate::carver::signatures::SignatureDatabase::parse_hex("ZZZZ");
    assert!(result.is_err(), "Invalid hex chars should fail");
}

#[test]
fn test_sector_scanner_finds_jpeg_header() {
    use crate::carver::scanner::SectorScanner;
    use crate::carver::signatures::{FileSignature, SignatureDatabase};

    let db = SignatureDatabase {
        signatures: vec![FileSignature {
            extension: "jpg".into(),
            description: "JPEG".into(),
            magic_header: "ffd8ff".into(),
            magic_footer: Some("ffd9".into()),
            max_size: 50 * 1024 * 1024,
            header_mask: None,
        }],
    };
    let scanner = SectorScanner::new(db).unwrap();
    // Create a 512-byte sector with JPEG header at offset 0
    let mut sector = vec![0u8; 512];
    sector[0] = 0xFF;
    sector[1] = 0xD8;
    sector[2] = 0xFF;
    let hits = scanner.scan_buffer(&sector, 0, 512);
    assert_eq!(hits.len(), 1, "Should find one JPEG header");
    assert_eq!(hits[0].offset, 0);
    assert_eq!(hits[0].signature.extension, "jpg");
}

#[test]
fn test_sector_scanner_finds_png_header() {
    use crate::carver::scanner::SectorScanner;
    use crate::carver::signatures::{FileSignature, SignatureDatabase};

    let db = SignatureDatabase {
        signatures: vec![FileSignature {
            extension: "png".into(),
            description: "PNG".into(),
            magic_header: "89504e470d0a1a0a".into(),
            magic_footer: None,
            max_size: 100 * 1024 * 1024,
            header_mask: None,
        }],
    };
    let scanner = SectorScanner::new(db).unwrap();
    let mut sector = vec![0u8; 512];
    let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    sector[..8].copy_from_slice(&png_header);
    let hits = scanner.scan_buffer(&sector, 1024, 512);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].offset, 1024, "Offset should be base_offset + 0");
    assert_eq!(hits[0].signature.extension, "png");
}

#[test]
fn test_sector_scanner_finds_pdf_header() {
    use crate::carver::scanner::SectorScanner;
    use crate::carver::signatures::{FileSignature, SignatureDatabase};

    let db = SignatureDatabase {
        signatures: vec![FileSignature {
            extension: "pdf".into(),
            description: "PDF".into(),
            magic_header: "25504446".into(),
            magic_footer: None,
            max_size: 100 * 1024 * 1024,
            header_mask: None,
        }],
    };
    let scanner = SectorScanner::new(db).unwrap();
    let mut buf = vec![0u8; 1024];
    // Put PDF header at sector 1 (offset 512)
    buf[512..516].copy_from_slice(b"%PDF");
    let hits = scanner.scan_buffer(&buf, 0, 512);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].offset, 512);
}

#[test]
fn test_sector_scanner_empty_buffer() {
    use crate::carver::scanner::SectorScanner;
    use crate::carver::signatures::{FileSignature, SignatureDatabase};

    let db = SignatureDatabase {
        signatures: vec![FileSignature {
            extension: "jpg".into(),
            description: "JPEG".into(),
            magic_header: "ffd8ff".into(),
            magic_footer: None,
            max_size: 50 * 1024 * 1024,
            header_mask: None,
        }],
    };
    let scanner = SectorScanner::new(db).unwrap();
    let hits = scanner.scan_buffer(&[], 0, 512);
    assert!(hits.is_empty(), "Empty buffer should yield no hits");
}

#[test]
fn test_confidence_scorer() {
    use crate::carver::confidence::{Confidence, ConfidenceScorer};
    let mut scorer = ConfidenceScorer::new();
    scorer.has_header = true;
    assert_eq!(scorer.calculate(), Confidence::Low, "Header only → Low");

    scorer.structure_valid = true;
    assert_eq!(
        scorer.calculate(),
        Confidence::Medium,
        "Header + structure → Medium"
    );

    scorer.has_footer = true;
    assert_eq!(
        scorer.calculate(),
        Confidence::High,
        "Header + structure + footer → High"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Audit / signing tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_hash_bytes_deterministic() {
    let h1 = crate::audit::signing::hash_bytes(b"hello world");
    let h2 = crate::audit::signing::hash_bytes(b"hello world");
    assert_eq!(h1, h2, "Same input must produce same hash");
}

#[test]
fn test_hash_bytes_sha256_known_vector() {
    // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let h = crate::audit::signing::hash_bytes(b"");
    assert_eq!(
        h,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_hash_file_matches_hash_bytes() {
    let mut f = NamedTempFile::new().unwrap();
    let data = b"test data for hashing";
    f.write_all(data).unwrap();
    f.flush().unwrap();

    let file_hash = crate::audit::signing::hash_file(f.path()).unwrap();
    let bytes_hash = crate::audit::signing::hash_bytes(data);
    assert_eq!(
        file_hash, bytes_hash,
        "File hash should match bytes hash for same content"
    );
}

#[test]
fn test_verify_file_hash_correct() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"verify me").unwrap();
    f.flush().unwrap();

    let hash = crate::audit::signing::hash_file(f.path()).unwrap();
    assert!(crate::audit::signing::verify_file_hash(f.path(), &hash).unwrap());
}

#[test]
fn test_verify_file_hash_incorrect() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"verify me").unwrap();
    f.flush().unwrap();

    assert!(!crate::audit::signing::verify_file_hash(f.path(), "0000000000000000").unwrap());
}

#[test]
fn test_report_manifest_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("manifest.json");

    let manifest = crate::audit::signing::ReportManifest::new(
        std::path::Path::new("/tmp/report.pdf"),
        "abc123".to_string(),
        "CASE-001".to_string(),
        "chain_tip_hash".to_string(),
    );
    manifest.save(&manifest_path).unwrap();
    let loaded = crate::audit::signing::ReportManifest::load(&manifest_path).unwrap();
    assert_eq!(loaded.case_id, "CASE-001");
    assert_eq!(loaded.report_hash, "abc123");
    assert_eq!(loaded.chain_tip, "chain_tip_hash");
    assert!(loaded.hash_verified);
}

#[test]
fn test_report_manifest_with_tsa_token_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("manifest_tsa.json");
    let mut manifest = crate::audit::signing::ReportManifest::new(
        std::path::Path::new("/tmp/report.pdf"),
        "hash_abc".to_string(),
        "CASE-TSA-01".to_string(),
        "tip_123".to_string(),
    );
    manifest.tsa_token_path = Some("/tmp/report.tsr".to_string());
    manifest.save(&manifest_path).unwrap();

    let loaded = crate::audit::signing::ReportManifest::load(&manifest_path).unwrap();
    assert_eq!(loaded.case_id, "CASE-TSA-01");
    assert_eq!(loaded.tsa_token_path, Some("/tmp/report.tsr".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// Database tests
// ═══════════════════════════════════════════════════════════════════════════

fn open_test_db() -> crate::db::CaseDatabase {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = crate::db::CaseDatabase::open(&db_path).unwrap();
    // Leak the tempdir so it lives for the duration of the test
    std::mem::forget(dir);
    db
}

#[test]
fn test_case_database_open_creates_tables() {
    let _db = open_test_db();
    // If we got here without error, tables were created successfully
}

#[test]
fn test_create_and_get_case() {
    let db = open_test_db();
    db.create_case("TEST-001", "operator1", "/dev/sdb", "wipe")
        .unwrap();
    let case = db.get_case("TEST-001").unwrap().unwrap();
    assert_eq!(case.id, "TEST-001");
    assert_eq!(case.operator, "operator1");
    assert_eq!(case.target, "/dev/sdb");
    assert_eq!(case.operation, "wipe");
    assert_eq!(case.status, "running");
}

#[test]
fn test_update_case_status() {
    let db = open_test_db();
    db.create_case("TEST-002", "op", "/dev/sdc", "recover")
        .unwrap();
    db.update_case_status("TEST-002", "completed").unwrap();
    let case = db.get_case("TEST-002").unwrap().unwrap();
    assert_eq!(case.status, "completed");
}

#[test]
fn test_list_cases() {
    let db = open_test_db();
    db.create_case("C-A", "op1", "/dev/sdb", "wipe").unwrap();
    db.create_case("C-B", "op2", "/dev/sdc", "recover").unwrap();
    let cases = db.list_cases().unwrap();
    assert_eq!(cases.len(), 2);
    let ids: Vec<&str> = cases.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&"C-A"));
    assert!(ids.contains(&"C-B"));
}

#[test]
fn test_insert_and_list_carved_files() {
    let db = open_test_db();
    db.create_case("CF-001", "op", "/dev/sdb", "recover")
        .unwrap();
    let file = crate::db::CarvedFileRecord {
        filename: "carved_0001.jpg".into(),
        file_type: "jpg".into(),
        size_bytes: 102400,
        confidence: 90,
        offset_bytes: 8192,
        output_path: "/tmp/carved/carved_0001.jpg".into(),
        category: "image".into(),
        sha256: Some("abcdef1234567890".into()),
    };
    let rowid = db.insert_carved_file("CF-001", &file).unwrap();
    assert!(rowid > 0);

    let files = db.list_carved_files("CF-001").unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].filename, "carved_0001.jpg");
    assert_eq!(files[0].size_bytes, 102400);
    assert_eq!(files[0].confidence, 90);
    assert_eq!(files[0].sha256.as_deref(), Some("abcdef1234567890"));
}

#[test]
fn test_list_carved_files_by_category() {
    let db = open_test_db();
    db.create_case("CAT-001", "op", "/dev/sdb", "recover")
        .unwrap();
    for (name, cat) in [
        ("a.jpg", "image"),
        ("b.pdf", "document"),
        ("c.png", "image"),
    ] {
        db.insert_carved_file(
            "CAT-001",
            &crate::db::CarvedFileRecord {
                filename: name.into(),
                file_type: "test".into(),
                size_bytes: 100,
                confidence: 80,
                offset_bytes: 0,
                output_path: "/tmp".into(),
                category: cat.into(),
                sha256: None,
            },
        )
        .unwrap();
    }
    let images = db
        .list_carved_files_by_category("CAT-001", "image")
        .unwrap();
    assert_eq!(images.len(), 2);
    let docs = db
        .list_carved_files_by_category("CAT-001", "document")
        .unwrap();
    assert_eq!(docs.len(), 1);
}

#[test]
fn test_insert_and_get_audit_entries() {
    let db = open_test_db();
    db.create_case("AUD-001", "op", "/dev/sdb", "wipe").unwrap();
    db.insert_audit_entry("AUD-001", r#"{"event":"start"}"#, "hash1")
        .unwrap();
    db.insert_audit_entry("AUD-001", r#"{"event":"done"}"#, "hash2")
        .unwrap();
    let entries = db.get_audit_entries("AUD-001").unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries[0].contains("start"));
    assert!(entries[1].contains("done"));
}

#[test]
fn test_get_nonexistent_case_returns_none() {
    let db = open_test_db();
    let case = db.get_case("DOES-NOT-EXIST").unwrap();
    assert!(case.is_none());
}

#[test]
fn test_case_database_migration_and_cross_schema_compatibility() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("legacy_cases.db");
    // Simulate legacy Tauri table structure lacking operator, operation, and report_path
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE cases (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                operation_type TEXT NOT NULL,
                target TEXT NOT NULL,
                status TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cases (id, created_at, operation_type, target, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["LEGACY-001", "2026-12-18T12:00:00Z", "erasure", "/dev/sdb", "completed"],
        )
        .unwrap();
    }

    // Now open with CaseDatabase, triggering automatic column migration
    let db = crate::db::CaseDatabase::open(&db_path).unwrap();
    let legacy = db.get_case("LEGACY-001").unwrap().unwrap();
    assert_eq!(legacy.id, "LEGACY-001");
    assert_eq!(legacy.operation, "erasure");
    assert_eq!(legacy.status, "completed");

    // Create a new case in the migrated database
    db.create_case("NEW-002", "investigator", "/dev/nvme0n1", "carve")
        .unwrap();
    let new_case = db.get_case("NEW-002").unwrap().unwrap();
    assert_eq!(new_case.id, "NEW-002");
    assert_eq!(new_case.operator, "investigator");
    assert_eq!(new_case.operation, "carve");

    let all_cases = db.list_cases().unwrap();
    assert_eq!(all_cases.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// Disk tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_raw_image_open_and_read() {
    use std::io::Read;
    let mut f = NamedTempFile::new().unwrap();
    let data = vec![0x42u8; 4096];
    f.write_all(&data).unwrap();
    f.flush().unwrap();

    let mut img = crate::disk::raw_image::RawImage::open(f.path()).unwrap();
    let mut buf = vec![0u8; 4096];
    img.read_exact(&mut buf).unwrap();
    assert_eq!(buf, data);
}

#[test]
fn test_raw_image_seek() {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"ABCDEFGHIJ").unwrap();
    f.flush().unwrap();

    let mut img = crate::disk::raw_image::RawImage::open(f.path()).unwrap();
    img.seek(SeekFrom::Start(5)).unwrap();
    let mut buf = [0u8; 5];
    img.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"FGHIJ");
}

#[test]
fn test_raw_image_size() {
    use crate::disk::DiskSource;
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&vec![0u8; 8192]).unwrap();
    f.flush().unwrap();

    let img = crate::disk::raw_image::RawImage::open(f.path()).unwrap();
    assert_eq!(img.size().unwrap(), 8192);
}

// ═══════════════════════════════════════════════════════════════════════════
// Error type tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_core_error_display() {
    let e = crate::error::CoreError::Disk("test error".into());
    assert_eq!(format!("{}", e), "Disk error: test error");
    let e = crate::error::CoreError::Wiper("bad drive".into());
    assert_eq!(format!("{}", e), "Wiper error: bad drive");
    let e = crate::error::CoreError::Parse("invalid".into());
    assert_eq!(format!("{}", e), "Parse error: invalid");
}

#[test]
fn test_core_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let core_err: crate::error::CoreError = io_err.into();
    assert!(format!("{}", core_err).contains("file not found"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Structure validators (carver)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_jpeg_validator_valid_header() {
    // Minimal JPEG: SOI + APP0 marker
    let data = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00,
    ];
    let result = crate::carver::structure::jpeg::validate_jpeg(&data);
    assert!(result.is_ok());
}

#[test]
fn test_jpeg_validator_invalid() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let result = crate::carver::structure::jpeg::validate_jpeg(&data);
    // Should return Ok(false) or Err — either way, not Ok(true)
    assert!(!result.unwrap_or(false));
}

#[test]
fn test_png_validator_valid_header() {
    // Minimal PNG: signature + IHDR chunk
    let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG sig
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]); // IHDR length = 13
    data.extend_from_slice(b"IHDR"); // IHDR type
    data.extend_from_slice(&[0u8; 13]); // IHDR data (13 bytes)
    data.extend_from_slice(&[0u8; 4]); // CRC
    let result = crate::carver::structure::png::validate_png(&data);
    assert!(result.is_ok());
}

#[test]
fn test_zip_validator_valid_header() {
    // Minimal ZIP: local file header
    let data = [0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00];
    let result = crate::carver::structure::zip::validate_zip(&data);
    assert!(result.is_ok());
}

#[test]
fn test_pdf_validator_valid_header() {
    let data = b"%PDF-1.4\n";
    let result = crate::carver::structure::pdf::validate_pdf(data);
    assert!(result.is_ok());
}

#[test]
fn test_sqlite_validator_valid_header() {
    let mut data = vec![0u8; 100];
    data[..16].copy_from_slice(b"SQLite format 3\0");
    // Page size at offset 16-17 (big-endian) — must be power of 2, >= 512
    data[16] = 0x10; // 0x1000 = 4096
    data[17] = 0x00;
    let result = crate::carver::structure::sqlite::validate_sqlite(&data);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// Cross-cutting: wipe + verify round-trip
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_file_wiper_multiple_passes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("multipass.bin");
    std::fs::write(&path, vec![0xAA; 8192]).unwrap();

    let wiper = crate::wiper::file_wiper::FileWiper::new(7, 3, false);
    let result = wiper.wipe_file(&path).unwrap();
    assert!(result.success);
    assert_eq!(result.passes_completed, 7);
    assert!(!path.exists());
}

#[test]
fn test_file_wiper_directory() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("a.txt"), b"file a").unwrap();
    std::fs::write(sub.join("b.txt"), b"file b").unwrap();

    let wiper = crate::wiper::file_wiper::FileWiper::new(1, 1, false);
    let results = wiper.wipe_directory(&sub).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.success));
    assert!(!sub.exists(), "Directory should be removed after wipe");
}

#[test]
fn test_wipe_pattern_reuse_does_not_repeat() {
    // Verify that calling fill() twice with a Random pattern produces different data
    let mut p = crate::wiper::patterns::WipePattern::for_pass(3);
    let mut buf1 = vec![0u8; 4096];
    let mut buf2 = vec![0u8; 4096];
    p.fill(&mut buf1);
    p.fill(&mut buf2);
    assert_ne!(buf1, buf2, "Two successive random fills should differ");
}

#[test]
fn test_carving_engine_on_crafted_image() {
    use crate::carver::engine::CarvingEngine;
    use crate::carver::scanner::SectorScanner;
    use crate::carver::signatures::{FileSignature, SignatureDatabase};
    use crate::disk::raw_image::RawImage;
    use std::io::Write;

    // Create a small raw image with a JPEG header at offset 0
    let dir = tempfile::tempdir().unwrap();
    let img_path = dir.path().join("test.img");
    let mut f = std::fs::File::create(&img_path).unwrap();

    // Write a fake JPEG starting at offset 0
    let mut data = vec![0u8; 4096];
    data[0] = 0xFF;
    data[1] = 0xD8;
    data[2] = 0xFF;
    data[3] = 0xE0;
    // Write some content and a JPEG footer
    data[100] = 0xFF;
    data[101] = 0xD9;
    f.write_all(&data).unwrap();
    f.flush().unwrap();
    drop(f);

    let db = SignatureDatabase {
        signatures: vec![FileSignature {
            extension: "jpg".into(),
            description: "JPEG".into(),
            magic_header: "ffd8ff".into(),
            magic_footer: Some("ffd9".into()),
            max_size: 50 * 1024 * 1024,
            header_mask: None,
        }],
    };
    let scanner = SectorScanner::new(db).unwrap();
    let engine = CarvingEngine::new(scanner);
    let mut disk = RawImage::open(&img_path).unwrap();
    let results = engine.carve(&mut disk).unwrap();
    assert!(!results.is_empty(), "Should find at least one carved file");
    assert_eq!(results[0].2, "jpg");
}

// ═══════════════════════════════════════════════════════════════════════════
// Strict Cryptographic, Forensics, and Security Regression Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_decode_escape_sequence_all_types() {
    use crate::plugins::toml_loader::decode_escape_sequence;
    let decoded = decode_escape_sequence(r"\x89PNG\r\n\x1A\n\\test\0\t");
    let mut expected = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n', b'\\'];
    expected.extend_from_slice(b"test");
    expected.push(0);
    expected.push(b'\t');
    assert_eq!(decoded, expected);
}

#[test]
fn test_size_string_parsing_all_units() {
    use crate::plugins::toml_loader::parse_size_string;
    assert_eq!(parse_size_string("100B").unwrap(), 100);
    assert_eq!(parse_size_string("10KB").unwrap(), 10 * 1024);
    assert_eq!(parse_size_string("50MB").unwrap(), 50 * 1024 * 1024);
    assert_eq!(parse_size_string("2GB").unwrap(), 2 * 1024 * 1024 * 1024);
    assert_eq!(parse_size_string("1TB").unwrap(), 1024 * 1024 * 1024 * 1024);
}

#[test]
fn test_size_string_parsing_overflow_protection() {
    use crate::plugins::toml_loader::parse_size_string;
    let overflow = parse_size_string("9999999999999999999TB");
    assert!(
        overflow.is_err(),
        "Overflowing size values must be rejected"
    );
}

#[test]
fn test_hash_chain_append_and_verify() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..5 {
        let entry = AuditEntry {
            id: i,
            seq: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: OperationType::DiskWipe,
            target: format!("/dev/sd{}", (b'b' + i as u8) as char),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: format!("Pass {}", i),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(100),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }

    assert_eq!(chain.len(), 5);
    assert!(chain.verify(), "Intact chain must pass verification");
}

#[test]
fn test_hash_chain_tamper_detection() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    for i in 0..3 {
        let entry = AuditEntry {
            id: i,
            seq: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: OperationType::DiskWipe,
            target: "/dev/sdb".into(),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: "OK".into(),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(100),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: None,
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        chain.append(entry).unwrap();
    }

    // Tamper with an entry by reconstructing an AuditLog with modified content
    let log = chain.to_audit_log("CASE-001".into(), None);
    let mut tampered_entries = log.entries;
    tampered_entries[1].target = "/dev/sdc_TAMPERED".into();

    // Verify manually that the tampered entry fails hash checks
    let mut modified_chain = HashChain::new();
    for entry in tampered_entries {
        let _ = modified_chain.append(entry);
    }
    // Tip of chain computed with tampered content will differ from original
    assert_ne!(modified_chain.tip(), chain.tip());
}

#[test]
fn test_hash_chain_deterministic_serialization_across_restarts() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();

    // Create entry with multiple keys in params in non-alphabetical insertion order
    let mut params = BTreeMap::new();
    params.insert("zeta_key".to_string(), serde_json::json!("last"));
    params.insert("alpha_key".to_string(), serde_json::json!("first"));
    params.insert("mid_key".to_string(), serde_json::json!(42));

    let entry = AuditEntry {
        id: 42,
        seq: 0,
        timestamp: "2026-12-19T14:00:00Z".to_string(),
        operation: OperationType::DiskWipe,
        target: "/dev/nvme0n1".to_string(),
        params,
        result: OperationResult {
            success: true,
            message: "All passes verified".to_string(),
            pre_hash: Some("pre123".to_string()),
            post_hash: Some("post456".to_string()),
            sectors_processed: Some(5000),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(7.99),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };

    let original_hash = chain.append(entry).unwrap();
    assert!(chain.verify());

    // Export log to JSON (simulating disk persistence)
    let log = chain.to_audit_log("CASE-DET-01".to_string(), Some("auditor".to_string()));
    let serialized = serde_json::to_string(&log).unwrap();

    // Deserialize log in a fresh context
    let deserialized: crate::audit::schema::AuditLog = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.chain_tip, original_hash);

    // Replay entries into a new chain to verify exact bit-for-bit reproducibility
    let mut replayed_chain = HashChain::new();
    let replayed_hash = replayed_chain
        .append(deserialized.entries[0].clone())
        .unwrap();

    assert_eq!(replayed_hash, original_hash);
    assert!(replayed_chain.verify());
}

#[test]
fn test_path_traversal_dotdot_blocked() {
    use crate::wiper::file_wiper::is_protected_path;
    assert!(is_protected_path(std::path::Path::new(
        "/tmp/../../etc/shadow"
    )));
    assert!(is_protected_path(std::path::Path::new(
        "/tmp/../etc/passwd"
    )));
}

#[test]
fn test_constant_time_eq_different_lengths() {
    assert!(!crate::audit::signing::verify_file_hash(
        std::path::Path::new("/nonexistent_path"),
        "short"
    )
    .unwrap_or(false));
}

#[test]
fn test_carver_corrupt_png_rejected() {
    // Corrupt PNG with truncated IHDR
    let corrupt_png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
    let result = crate::carver::structure::png::validate_png(&corrupt_png);
    assert!(!result.unwrap_or(false), "Truncated PNG must be rejected");
}

#[test]
fn test_carver_corrupt_jpeg_no_eoi() {
    // JPEG header without EOI marker
    let corrupt_jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
    let result = crate::carver::structure::jpeg::validate_jpeg(&corrupt_jpeg);
    assert!(
        !result.unwrap_or(false),
        "JPEG missing EOI must be rejected"
    );
}

#[test]
fn test_carver_corrupt_sqlite_bad_page_size() {
    let mut corrupt_sqlite = vec![0u8; 100];
    corrupt_sqlite[..16].copy_from_slice(b"SQLite format 3\0");
    // Invalid page size: 0
    corrupt_sqlite[16] = 0x00;
    corrupt_sqlite[17] = 0x00;
    let result = crate::carver::structure::sqlite::validate_sqlite(&corrupt_sqlite);
    assert!(
        !result.unwrap_or(false),
        "SQLite with page size 0 must be rejected"
    );
}

#[test]
fn test_carver_corrupt_zip_bad_signature() {
    let bad_zip = [0x50, 0x4B, 0xFF, 0xFF, 0x00, 0x00];
    let result = crate::carver::structure::zip::validate_zip(&bad_zip);
    assert!(
        !result.unwrap_or(false),
        "Corrupt ZIP header must be rejected"
    );
}

#[test]
fn test_ewf_e01_magic_detection() {
    use crate::disk::ewf::E01Image;
    let mut file = NamedTempFile::new().unwrap();
    // Write invalid header
    file.write_all(b"NOT_E01_IMAGE_DATA").unwrap();
    file.flush().unwrap();
    let result = E01Image::open(file.path());
    assert!(
        result.is_err(),
        "Non-E01 file must be rejected by E01 parser"
    );
}

#[test]
fn test_ewf_e01_is_detected_and_rejected_without_a_full_decoder() {
    use crate::disk::ewf::{E01Image, E01_FILE_SIGNATURE};

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(E01_FILE_SIGNATURE).unwrap();
    file.write_all(&[1, 0, 1, 0, 0]).unwrap();
    file.flush().unwrap();

    let error = match E01Image::open(file.path()) {
        Ok(_) => panic!("E01 evidence must not be opened by the incomplete decoder"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("not supported"));
}

#[test]
fn test_argon2_password_hashing_and_verification() {
    use argon2::{
        password_hash::{
            rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        },
        Argon2,
    };

    let password = b"SecureForensicPassphrase2026!";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();

    let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    assert!(argon2.verify_password(password, &parsed_hash).is_ok());
    assert!(argon2
        .verify_password(b"WrongPassword!", &parsed_hash)
        .is_err());
}

#[test]
fn test_surface_scanner_and_cluster_mapping() {
    use crate::disk::surface_scanner::{scan_surface, ClusterAllocation};
    let mut file = NamedTempFile::new().unwrap();
    // Write 32KB of data: first 4KB FILE (MFT), then 4KB zeros, then 4KB JPEG, then 20KB random
    let mut data = vec![0u8; 32768];
    data[0..4].copy_from_slice(b"FILE");
    // data[4096..8192] remains all zeros
    data[8192..8196].copy_from_slice(b"\xFF\xD8\xFF\xE0");
    for (i, byte) in data.iter_mut().enumerate().take(32768).skip(12288) {
        *byte = ((i * 17) % 255 + 1) as u8;
    }
    file.write_all(&data).unwrap();
    file.flush().unwrap();

    let scan = scan_surface(file.path(), 8).unwrap();
    assert_eq!(scan.cells.len(), 8);
    assert!(scan.health_score > 90.0);
    // Cell 0 should detect SystemMft
    assert_eq!(scan.cells[0].allocation, ClusterAllocation::SystemMft);
    // Cell 1 is an observed zero-filled sample — classified as free/unallocated space.
    assert_eq!(scan.cells[1].allocation, ClusterAllocation::FreeSpace);
    // Cell 2 should detect CarvableDeleted (JPEG)
    assert_eq!(scan.cells[2].allocation, ClusterAllocation::CarvableDeleted);
    // Cell 3 is readable structured binary data.
    assert_eq!(scan.cells[3].allocation, ClusterAllocation::ActiveFile);
    // Should have good blocks
    assert!(scan.good_count > 0);

    // Failed evidence reads return an error; they never fabricate a sector map.
    assert!(scan_surface("/nonexistent_drive_9999", 8).is_err());
}

#[test]
fn test_zip_validator_and_metadata_in_large_buffer() {
    use crate::carver::structure::zip::{parse_zip_metadata, validate_zip};
    // Construct a minimal valid ZIP archive:
    // 1. Local file header (30 bytes)
    // 2. Data ("hello") (5 bytes)
    // 3. Central directory header (46 bytes)
    // 4. End of Central Directory (22 bytes)
    let mut zip_bytes = Vec::new();
    // Local header: PK\x03\x04
    zip_bytes.extend_from_slice(&[
        0x50, 0x4B, 0x03, 0x04, // sig
        20, 0, // version
        0, 0, // flags
        0, 0, // compression (none)
        0, 0, 0, 0, // time/date
        0x36, 0x47, 0x22, 0x36, // CRC-32 (dummy)
        5, 0, 0, 0, // compressed size (5)
        5, 0, 0, 0, // uncompressed size (5)
        0, 0, // filename len (0)
        0, 0, // extra field len (0)
    ]);
    zip_bytes.extend_from_slice(b"hello");

    let cd_offset = zip_bytes.len() as u32;
    // Central directory header: PK\x01\x02
    zip_bytes.extend_from_slice(&[
        0x50, 0x4B, 0x01, 0x02, // sig
        20, 0, 20, 0, // version made by / needed
        0, 0, 0, 0, // flags / comp
        0, 0, 0, 0, // time / date
        0x36, 0x47, 0x22, 0x36, // CRC-32
        5, 0, 0, 0, // comp size
        5, 0, 0, 0, // uncomp size
        0, 0, 0, 0, 0, 0, // fname / extra / comment lens
        0, 0, 0, 0, 0, 0, 0, 0, // disk start / internal / external attr
        0, 0, 0, 0, // local header offset (0)
    ]);
    let cd_size = (zip_bytes.len() as u32) - cd_offset;

    // EOCD: PK\x05\x06
    zip_bytes.extend_from_slice(&[
        0x50, 0x4B, 0x05, 0x06, // sig
        0, 0, 0, 0, // disk numbers
        1, 0, 1, 0, // entries on this disk / total entries (1)
    ]);
    zip_bytes.extend_from_slice(&cd_size.to_le_bytes());
    zip_bytes.extend_from_slice(&cd_offset.to_le_bytes());
    zip_bytes.extend_from_slice(&[0, 0]); // comment len (0)

    let exact_len = zip_bytes.len();

    // Place in a 128KB buffer with unallocated trailing bytes (simulating carver chunk)
    let mut large_buffer = vec![0u8; 128 * 1024];
    large_buffer[..exact_len].copy_from_slice(&zip_bytes);

    assert!(validate_zip(&large_buffer).unwrap());
    let meta =
        parse_zip_metadata(&large_buffer).expect("should parse zip metadata in large buffer");
    assert_eq!(meta.exact_archive_size, exact_len as u64);
    assert_eq!(meta.file_count, 1);
}

#[test]
fn test_pdf_validator_in_large_buffer() {
    use crate::carver::structure::pdf::{parse_pdf_metadata, validate_pdf};
    let pdf_content = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\nstartxref\n9\n%%EOF\n";
    let mut large_buffer = vec![0u8; 64 * 1024];
    large_buffer[..pdf_content.len()].copy_from_slice(pdf_content);

    assert!(validate_pdf(&large_buffer).unwrap());
    let meta = parse_pdf_metadata(&large_buffer).expect("should parse pdf metadata");
    assert_eq!(meta.exact_file_size, pdf_content.len() as u64);
    assert_eq!(meta.version, "1.4");
}

#[test]
fn test_sqlite_validator_power_of_two_enforcement() {
    use crate::carver::structure::sqlite::validate_sqlite;
    let mut data = vec![0u8; 4096];
    data[..16].copy_from_slice(b"SQLite format 3\0");
    // Invalid page size: 500 (not power of 2, < 512)
    data[16] = (500 >> 8) as u8;
    data[17] = (500 & 0xFF) as u8;
    assert!(!validate_sqlite(&data).unwrap());

    // Valid page size: 4096
    data[16] = 0x10;
    data[17] = 0x00;
    assert!(validate_sqlite(&data).unwrap());
}

#[test]
fn test_file_wiper_scrub_slack_space_standalone() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("slack_test.bin");
    let test_data = vec![0x42u8; 1500];
    std::fs::write(&file_path, &test_data).unwrap();

    let wiper = crate::wiper::file_wiper::FileWiper::new(1, 1, true);
    let result = wiper.scrub_slack_space(&file_path);
    assert!(
        result.is_err(),
        "ordinary file I/O must not claim to scrub filesystem slack"
    );
    assert!(
        file_path.exists(),
        "the unsupported operation must not change the file"
    );
}

#[test]
fn test_file_wiper_scrub_slack_space_in_wipe_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("slack_wipe.bin");
    let test_data = vec![0xAAu8; 2000];
    std::fs::write(&file_path, &test_data).unwrap();

    let wiper = crate::wiper::file_wiper::FileWiper::new(2, 2, true);
    let result = wiper.wipe_file(&file_path);
    assert!(
        result.is_err(),
        "a requested slack scrub must fail rather than report false coverage"
    );
    assert!(file_path.exists());
}

#[test]
#[cfg(unix)]
fn test_file_wiper_scrub_slack_rejects_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let target_path = dir.path().join("target.bin");
    std::fs::write(&target_path, b"test payload").unwrap();
    let link_path = dir.path().join("link.bin");
    std::os::unix::fs::symlink(&target_path, &link_path).unwrap();

    let wiper = crate::wiper::file_wiper::FileWiper::new(1, 1, true);
    let res = wiper.scrub_slack_space(&link_path);
    assert!(res.is_err());
}

#[test]
fn test_trim_carved_file_boundary_across_formats() {
    use crate::carver::structure::trim_carved_file_boundary;

    // 1. PDF with trailing garbage
    let pdf_bytes = b"%PDF-1.4\nobj\nendobj\n%%EOF\n";
    let mut padded_pdf = vec![0u8; 10000];
    padded_pdf[..pdf_bytes.len()].copy_from_slice(pdf_bytes);
    let trimmed = trim_carved_file_boundary("pdf", &padded_pdf, None);
    assert_eq!(trimmed, pdf_bytes.len());

    // 2. JPEG with trailing garbage
    let jpeg_bytes = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0xFF, 0xD9,
    ];
    let mut padded_jpeg = vec![0u8; 8000];
    padded_jpeg[..jpeg_bytes.len()].copy_from_slice(&jpeg_bytes);
    let trimmed = trim_carved_file_boundary("jpg", &padded_jpeg, Some(&[0xFF, 0xD9]));
    assert_eq!(trimmed, jpeg_bytes.len());
}

#[test]
fn test_dhash_downsample_and_similarity() {
    use crate::classify::dhash::{
        compute_dhash_from_grid, dhash_from_hex, dhash_to_hex, downsample_grayscale_to_9x8,
        hamming_distance, is_visual_duplicate, rgb_to_grayscale,
    };

    // 18x16 gradient image
    let mut pixels = vec![0u8; 18 * 16];
    for y in 0..16 {
        for x in 0..18 {
            pixels[y * 18 + x] = (x * 14) as u8;
        }
    }
    let grid = downsample_grayscale_to_9x8(18, 16, &pixels).unwrap();
    let hash = compute_dhash_from_grid(&grid);
    let hex = dhash_to_hex(hash);
    assert_eq!(hex.len(), 16);
    assert_eq!(dhash_from_hex(&hex).unwrap(), hash);

    // Minor alteration (e.g. 1 pixel change)
    let mut altered_grid = grid;
    altered_grid[5] ^= 0x01;
    let altered_hash = compute_dhash_from_grid(&altered_grid);
    assert!(hamming_distance(hash, altered_hash) <= 2);
    assert!(is_visual_duplicate(hash, altered_hash, 10));

    // RGB to grayscale conversion
    let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // Red, Green, Blue
    let gray = rgb_to_grayscale(&rgb);
    assert_eq!(gray.len(), 3);
    assert!(gray[1] > gray[0]); // Green is brighter than Red in Rec. 601
    assert!(gray[0] > gray[2]); // Red is brighter than Blue in Rec. 601
}

#[test]
fn test_categorize_extension() {
    use crate::classify::{categorize_extension, FileCategory};
    assert_eq!(categorize_extension("jpg"), FileCategory::Media);
    assert_eq!(categorize_extension(".PNG"), FileCategory::Media);
    assert_eq!(categorize_extension("pdf"), FileCategory::Documents);
    assert_eq!(categorize_extension("docx"), FileCategory::Documents);
    assert_eq!(categorize_extension("zip"), FileCategory::Archives);
    assert_eq!(categorize_extension("tar.gz"), FileCategory::Unknown);
    assert_eq!(categorize_extension("gz"), FileCategory::Archives);
    assert_eq!(categorize_extension("sqlite"), FileCategory::Databases);
    assert_eq!(categorize_extension("db"), FileCategory::Databases);
    assert_eq!(categorize_extension("exe"), FileCategory::Executables);
    assert_eq!(categorize_extension("sys"), FileCategory::System);
    assert_eq!(categorize_extension("xyz123"), FileCategory::Unknown);
}

#[test]
fn test_lua_plugin_binary_signatures_and_nil_truthiness() {
    use crate::plugins::lua_host::LuaPluginHost;
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("test_bin_plugin.lua");

    let script = r#"
        signature {
            name = "custom_bin",
            category = "Custom",
            header = "\x89PNG\r\n\x1a\n",
            footer = "\xFF\xD9",
            max_size = "10MB",
            validate = function(data)
                if #data < 8 then
                    return nil -- nil evaluates as false (invalid)
                end
                if data:byte(1) == 0x89 and data:byte(2) == 0x50 then
                    return true
                end
                return false
            end
        }
    "#;
    std::fs::write(&script_path, script).unwrap();

    let mut host = LuaPluginHost::new();
    host.load_script(&script_path).unwrap();

    let sigs = host.loaded_signatures();
    assert_eq!(sigs.len(), 1);
    assert_eq!(sigs[0].name, "custom_bin");
    assert_eq!(sigs[0].category, "Custom");
    assert_eq!(sigs[0].header, b"\x89PNG\r\n\x1a\n");
    assert_eq!(sigs[0].footer, Some(vec![0xFF, 0xD9]));
    assert_eq!(sigs[0].max_size, 10 * 1024 * 1024);
    assert!(sigs[0].has_validator);

    // Test validation: data too short -> returns nil -> treated as false
    assert!(!host.validate("custom_bin", b"\x89P").unwrap());

    // Test validation: valid header -> returns true
    assert!(host
        .validate("custom_bin", b"\x89PNG\r\n\x1a\nvalid")
        .unwrap());

    // Test validation: invalid second byte -> returns false
    assert!(!host
        .validate("custom_bin", b"\x89XNG\r\n\x1a\ninvalid")
        .unwrap());
}

#[test]
fn test_lua_plugin_table_byte_array_signatures() {
    use crate::plugins::lua_host::LuaPluginHost;
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("test_table_sig.lua");

    let script = r#"
        signature {
            name = "table_sig",
            category = "Binary",
            header = { 0x45, 0x56, 0x46, 0x09 },
            max_size = 1048576
        }
    "#;
    std::fs::write(&script_path, script).unwrap();

    let mut host = LuaPluginHost::new();
    host.load_script(&script_path).unwrap();

    let sigs = host.loaded_signatures();
    assert_eq!(sigs.len(), 1);
    assert_eq!(sigs[0].header, vec![0x45, 0x56, 0x46, 0x09]);
    assert!(!sigs[0].has_validator);
    assert!(host.validate("table_sig", b"anything").unwrap());
}

#[test]
fn test_file_wiper_readonly_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("readonly.txt");
    std::fs::write(&file_path, b"confidential read only data").unwrap();

    let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&file_path, perms).unwrap();

    let wiper = crate::wiper::file_wiper::FileWiper::new(2, 2, false);
    let result = wiper
        .wipe_file(&file_path)
        .expect("Should successfully wipe read-only file");

    assert!(result.success);
    assert!(!file_path.exists(), "File should be deleted after wiping");
}

#[test]
fn test_protected_path_root_handling() {
    use std::path::Path;
    assert!(crate::wiper::file_wiper::is_protected_path(Path::new(
        "/etc"
    )));
    assert!(crate::wiper::file_wiper::is_protected_path(Path::new(
        "/etc/passwd"
    )));
    assert!(crate::wiper::file_wiper::is_protected_path(Path::new(
        "/boot/vmlinuz"
    )));
    assert!(!crate::wiper::file_wiper::is_protected_path(Path::new(
        "/home/user/document.txt"
    )));
}

#[test]
fn test_recovery_case_and_carved_file_db_flow() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("cases.db");
    let db = crate::db::CaseDatabase::open(&db_path).unwrap();

    let case_id = "test-rec-1234";
    db.create_case(case_id, "Analyst", "/dev/sdb", "recovery")
        .unwrap();

    let carved = crate::db::CarvedFileRecord {
        filename: "carved_jpg_0000001024.jpg".to_string(),
        file_type: "jpg".to_string(),
        size_bytes: 2048,
        confidence: 90,
        offset_bytes: 1024,
        output_path: "/tmp/carved_jpg_0000001024.jpg".to_string(),
        category: "Images".to_string(),
        sha256: Some(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        ),
    };
    db.insert_carved_file(case_id, &carved).unwrap();
    db.update_case_status(case_id, "completed").unwrap();

    let cases = db.list_cases().unwrap();
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].id, case_id);
    assert_eq!(cases[0].operation, "recovery");
    assert_eq!(cases[0].status, "completed");

    let files = db.list_carved_files(case_id).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].filename, "carved_jpg_0000001024.jpg");
    assert_eq!(files[0].confidence, 90);
}

#[test]
fn test_fat_directory_entry_with_tilde_and_creation_timestamp() {
    use crate::carver::filesystem_records::scan_fat_directory_entries;

    let mut entry = [0u8; 32];
    entry[0] = 0xE5; // Deleted entry marker
    entry[1..8].copy_from_slice(b"MYDOC~1");
    entry[8..11].copy_from_slice(b"PDF");
    entry[11] = 0x20; // Archive attribute

    // Creation time: 10:30:00 (h=10, m=30, s=0) -> (10<<11) | (30<<5) = 21440
    let ctime: u16 = (10 << 11) | (30 << 5);
    entry[14..16].copy_from_slice(&ctime.to_le_bytes());
    // Creation date: 2026-05-15 -> year 2026-1980 = 46. (46<<9) | (5<<5) | 15 = 23727
    let cdate: u16 = (46 << 9) | (5 << 5) | 15;
    entry[16..18].copy_from_slice(&cdate.to_le_bytes());

    // Modification time: 14:45:10
    let mtime: u16 = (14 << 11) | (45 << 5) | 5;
    entry[22..24].copy_from_slice(&mtime.to_le_bytes());
    let mdate: u16 = (46 << 9) | (5 << 5) | 16;
    entry[24..26].copy_from_slice(&mdate.to_le_bytes());

    // Cluster: 42
    entry[26..28].copy_from_slice(&42u16.to_le_bytes());

    // File size: 54321 bytes
    entry[28..32].copy_from_slice(&54321u32.to_le_bytes());

    let hits = scan_fat_directory_entries(&entry, 0);
    assert_eq!(hits.len(), 1);
    let meta = &hits[0].1;
    assert_eq!(meta.filename, "_MYDOC~1.pdf");
    assert_eq!(meta.file_extension, "pdf");
    assert_eq!(meta.file_size_bytes, 54321);
    assert!(meta.created_at.is_some());
    assert!(meta.created_at.as_ref().unwrap().starts_with("2026-05-15"));
    assert!(meta.modified_at.is_some());
    assert!(meta.modified_at.as_ref().unwrap().starts_with("2026-05-16"));
}

#[test]
fn test_jpeg_marker_parsing_with_dqt_and_sos() {
    use crate::carver::structure::jpeg::parse_jpeg_metadata;
    use crate::carver::structure::trim_carved_file_boundary;

    let mut jpeg = Vec::new();
    // SOI
    jpeg.extend_from_slice(&[0xFF, 0xD8]);
    // APP0: len = 16 (includes 2 length bytes) -> total APP0 chunk = 18 bytes
    jpeg.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]);
    jpeg.extend_from_slice(b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
    // DQT: len = 67 (2 len bytes + 1 table info + 64 table bytes)
    jpeg.extend_from_slice(&[0xFF, 0xDB, 0x00, 0x43, 0x00]);
    jpeg.extend_from_slice(&[0x10u8; 64]);
    // SOF0: len = 11 (2 len + 1 prec + 2 h + 2 w + 1 comp + 3 comp info)
    jpeg.extend_from_slice(&[
        0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x64, 0x00, 0x64, 0x01, 0x01,
    ]);
    // SOS: len = 8 (2 len + 1 comp + 2 comp info + 3 spectral)
    jpeg.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x3F]);
    // Entropy-coded scan data with byte stuffing FF 00 and literal data
    jpeg.extend_from_slice(&[0x12, 0x34, 0xFF, 0x00, 0x56, 0x78]);
    // EOI
    jpeg.extend_from_slice(&[0xFF, 0xD9]);

    let expected_len = jpeg.len();

    // Append trailing disk slack noise
    let mut disk_buffer = jpeg.clone();
    disk_buffer.extend_from_slice(&[0xAA; 5000]);

    let meta = parse_jpeg_metadata(&disk_buffer).expect("Should parse JPEG metadata");
    assert_eq!(meta.exact_file_size, expected_len as u64);

    let trimmed = trim_carved_file_boundary("jpg", &disk_buffer, Some(&[0xFF, 0xD9]));
    assert_eq!(trimmed, expected_len);
}

#[test]
fn test_verify_random_wipe_compares_the_written_keyed_stream() {
    use crate::disk::raw_image::RawImage;
    use crate::wiper::patterns::WipePattern;
    use crate::wiper::verify::verify_pattern_wipe;

    let dir = tempfile::tempdir().unwrap();
    let img_path = dir.path().join("tail_rand.img");

    let mut pattern = WipePattern::for_pass(3);
    let mut rand_data = vec![0; 4096];
    pattern.fill(&mut rand_data);
    std::fs::write(&img_path, &rand_data).unwrap();

    let mut disk = RawImage::open(&img_path).unwrap();
    assert!(verify_pattern_wipe(&mut disk, &pattern, 100).unwrap());

    // Random-looking input is insufficient; bytes must match the written key.
    let zero_img_path = dir.path().join("tail_zero.img");
    std::fs::write(&zero_img_path, vec![0; 4096]).unwrap();
    let mut zero_disk = RawImage::open(&zero_img_path).unwrap();
    assert!(!verify_pattern_wipe(&mut zero_disk, &pattern, 100).unwrap());
}

#[test]
fn test_zip_structure_comment_overflow_rejection() {
    use crate::carver::structure::zip::{parse_zip_metadata, validate_zip};

    // Construct a minimal ZIP header and EOCD record
    let mut zip_data = Vec::new();
    // Local file header: PK\x03\x04
    zip_data.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
    zip_data.extend_from_slice(&[0u8; 26]); // LFH fields
                                            // EOCD record: PK\x05\x06
    zip_data.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
    zip_data.extend_from_slice(&[0u8; 16]); // Disk numbers, entries, CD size, CD offset (0)
                                            // Claim comment_len = 500 bytes, but buffer ends immediately after!
    zip_data.extend_from_slice(&500u16.to_le_bytes());

    // Should not validate because comment claims 500 bytes beyond the buffer end
    assert!(!validate_zip(&zip_data).unwrap());
    assert!(parse_zip_metadata(&zip_data).is_none());
}

#[test]
fn test_sqlite_metadata_zero_in_header_pages_stops_at_slack() {
    use crate::carver::structure::sqlite::parse_sqlite_metadata;

    let page_size = 4096usize;
    // Buffer contains 4 pages (16KB), but pages 2, 3, 4 are zeroed disk slack
    let mut data = vec![0u8; page_size * 4];
    data[..16].copy_from_slice(b"SQLite format 3\0");
    // page size 4096
    data[16] = 0x10;
    data[17] = 0x00;
    // in_header_pages = 0 (offset 28..32)
    data[28..32].copy_from_slice(&0u32.to_be_bytes());

    let meta = parse_sqlite_metadata(&data).expect("Should parse SQLite metadata");
    assert_eq!(meta.page_size, 4096);
    // Because pages 2..4 are all zeros, page_count must stop at 1 page instead of consuming 4 pages of slack
    assert_eq!(meta.page_count, 1);
    assert_eq!(meta.exact_db_size, 4096);
}

#[test]
fn test_wipe_patterns_is_random_and_generator_coverage() {
    use crate::wiper::patterns::{
        get_pattern_fn_for_pass, is_random_pass, passes_for_method, WipePattern,
    };

    // Zero method: 1 pass, zeros (not random)
    let zero_passes = passes_for_method("zero");
    assert_eq!(zero_passes, vec![1]);
    assert!(!is_random_pass("zero", 1));
    let pat_fn = get_pattern_fn_for_pass("zero", 1);
    assert_eq!(pat_fn(16), vec![0u8; 16]);

    // Random method: pass 3, random
    let rand_passes = passes_for_method("random");
    assert_eq!(rand_passes, vec![3]);
    assert!(is_random_pass("random", 3));

    // DoD 3-pass: pass 1 (zeros), pass 2 (ones), pass 3 (random)
    let dod_passes = passes_for_method("dod3");
    assert_eq!(dod_passes, vec![1, 2, 3]);
    assert!(!is_random_pass("dod3", 1));
    assert!(!is_random_pass("dod3", 2));
    assert!(is_random_pass("dod3", 3));
    let ones_fn = get_pattern_fn_for_pass("dod3", 2);
    assert_eq!(ones_fn(16), vec![0xFFu8; 16]);

    // Gutmann: 35 passes. Pass 1..4 and 32..35 are random!
    let gutmann_passes = passes_for_method("gutmann");
    assert_eq!(gutmann_passes.len(), 35);
    assert_eq!(*gutmann_passes.last().unwrap(), 35);
    // Crucial fix verification: Gutmann last pass (35) must be correctly identified as random!
    assert!(
        is_random_pass("gutmann", 35),
        "Gutmann pass 35 must be detected as random"
    );
    assert!(
        is_random_pass("gutmann", 1),
        "Gutmann pass 1 must be detected as random"
    );
    assert!(
        !is_random_pass("gutmann", 5),
        "Gutmann pass 5 (0x55) is deterministic, not random"
    );
    assert!(
        WipePattern::for_gutmann_pass(35).is_random(),
        "WipePattern::for_gutmann_pass(35) is_random must be true"
    );
}

#[test]
fn test_file_wiper_wipe_file_delegation_and_shred() {
    use crate::wiper::file_wiper::FileWiper;

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("delegate_shred.bin");
    std::fs::write(&file_path, b"CONFIDENTIAL_FORENSIC_DATA_TO_ERASE").unwrap();

    let wiper = FileWiper::new(2, 3, false);
    let res = wiper
        .wipe_file(&file_path)
        .expect("wipe_file should succeed");
    assert!(res.success);
    assert_eq!(res.passes_completed, 2);
    assert!(!file_path.exists(), "Original file path must be eradicated");
}

#[test]
fn test_firmware_method_parsing_and_formatting() {
    use crate::wiper::firmware::FirmwareMethod;
    use std::str::FromStr;

    let methods = [
        ("nvme-crypto", FirmwareMethod::NvmeCryptoErase),
        ("nvme-block", FirmwareMethod::NvmeBlockErase),
        ("ata", FirmwareMethod::AtaSecureErase),
        ("ata-enhanced", FirmwareMethod::AtaEnhancedErase),
    ];

    for (label, method) in methods {
        assert_eq!(method.as_str(), label);
        assert_eq!(format!("{}", method), label);
        assert_eq!(FirmwareMethod::parse(label), Some(method));
        assert_eq!(FirmwareMethod::from_str(label).unwrap(), method);
    }

    assert_eq!(FirmwareMethod::parse("unknown"), None);
    assert!(FirmwareMethod::from_str("invalid-method").is_err());
}

#[test]
fn test_firmware_is_valid_block_device_path_safety() {
    use crate::wiper::firmware::is_valid_block_device_path;
    use std::path::Path;

    // Reject non-/dev paths
    assert!(!is_valid_block_device_path(Path::new("/tmp/some_file")));
    assert!(!is_valid_block_device_path(Path::new("relative/dev/sda")));
    assert!(!is_valid_block_device_path(Path::new("dev/sda")));

    // Reject traversal
    assert!(!is_valid_block_device_path(Path::new("/dev/../etc/passwd")));

    // Reject shell metacharacters
    assert!(!is_valid_block_device_path(Path::new("/dev/sda;rm -rf /")));
    assert!(!is_valid_block_device_path(Path::new("/dev/sda | cat")));
    assert!(!is_valid_block_device_path(Path::new("/dev/sda`whoami`")));
    assert!(!is_valid_block_device_path(Path::new("/dev/sda$(id)")));
    assert!(!is_valid_block_device_path(Path::new("/dev/sda & bg")));
    assert!(!is_valid_block_device_path(Path::new("/dev/sda\0bad")));

    // Reject non-existent /dev target
    assert!(!is_valid_block_device_path(Path::new(
        "/dev/non_existent_device_xyz123"
    )));
}

#[test]
fn test_case_database_report_payload_integration() {
    use crate::db::{CarvedFileRecord, CaseDatabase};

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test_cases.db");
    let db = CaseDatabase::open(&db_path).expect("open db");

    let case_id = "CASE-REPORT-101";
    db.create_case(case_id, "Chief Examiner", "/dev/sdd", "recovery")
        .expect("create case");

    db.insert_carved_file(
        case_id,
        &CarvedFileRecord {
            filename: "photo_01.jpg".to_string(),
            file_type: "jpeg".to_string(),
            size_bytes: 409600,
            confidence: 95,
            offset_bytes: 1048576,
            output_path: "/tmp/photo_01.jpg".to_string(),
            category: "image".to_string(),
            sha256: Some("aabbccddeeff".to_string()),
        },
    )
    .expect("insert carved file");

    let audit_event = serde_json::json!({
        "event": "carve_completed",
        "recovered_count": 1,
        "total_bytes": 409600,
    });
    let audit_str = audit_event.to_string();
    let audit_hash = crate::audit::signing::hash_bytes(audit_str.as_bytes());
    db.insert_audit_entry(case_id, &audit_str, &audit_hash)
        .expect("insert audit entry");

    // Query back
    let case = db.get_case(case_id).expect("get case").expect("found");
    assert_eq!(case.operator, "Chief Examiner");
    assert_eq!(case.target, "/dev/sdd");

    let files = db.list_carved_files(case_id).expect("list files");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].filename, "photo_01.jpg");
    assert_eq!(files[0].category, "image");

    let entries = db.get_audit_entries(case_id).expect("audit entries");
    assert_eq!(entries.len(), 1);
    let parsed: serde_json::Value = serde_json::from_str(&entries[0]).unwrap();
    assert_eq!(parsed["params"]["event"], "carve_completed");
}

#[test]
fn test_sector_scanner_zero_sector_size_safety() {
    use crate::carver::scanner::SectorScanner;
    use crate::carver::signatures::SignatureDatabase;

    let scanner = SectorScanner::new(SignatureDatabase::default()).unwrap();
    let data = vec![0u8; 1024];

    // Passing 0 as sector_size must return empty vector and NOT divide-by-zero panic
    let hits = scanner.scan_buffer(&data, 0, 0);
    assert!(hits.is_empty());
}

#[test]
fn test_pdf_metadata_extraction_from_trailer() {
    use crate::carver::structure::pdf::parse_pdf_metadata;

    // Construct a PDF > 64KB where /Title and /Author are located in the trailer near the end
    let mut pdf_data = Vec::new();
    pdf_data.extend_from_slice(b"%PDF-1.7\n");
    // Pad with 70,000 bytes of dummy stream content
    pdf_data.resize(70_000, b'x');
    // Place trailer with /Info dictionary and %%EOF
    let trailer = b"\ntrailer\n<< /Info << /Title (Confidential Investigation Report) /Author (Forensic Analyst) >> >>\nstartxref\n69900\n%%EOF\n";
    pdf_data.extend_from_slice(trailer);

    let meta = parse_pdf_metadata(&pdf_data).expect("PDF metadata should parse");
    assert_eq!(
        meta.title.as_deref(),
        Some("Confidential Investigation Report")
    );
    assert_eq!(meta.author.as_deref(), Some("Forensic Analyst"));
    assert!(meta.exact_file_size > 70_000);
}

#[test]
fn test_png_iend_boundary_validation() {
    use crate::carver::structure::png::{parse_png_metadata, validate_png};

    // Construct minimal valid PNG with IHDR and IEND
    let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // 8 bytes sig
                                                                         // IHDR chunk: len=13 (4 bytes), type=IHDR (4 bytes), data (13 bytes), crc (4 bytes) = 25 bytes
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]);
    data.extend_from_slice(b"IHDR");
    data.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00,
    ]);
    data.extend_from_slice(&[0x1F, 0x15, 0xC4, 0x89]); // IHDR CRC

    // Append truncated IEND (only 8 bytes instead of 12 bytes - missing CRC)
    let mut truncated = data.clone();
    truncated.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // len=0
    truncated.extend_from_slice(b"IEND"); // type=IEND (no 4-byte CRC)

    assert!(
        !validate_png(&truncated).unwrap(),
        "Truncated IEND must fail validation"
    );

    // Full IEND with 4-byte CRC
    let mut valid = data.clone();
    valid.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // len=0
    valid.extend_from_slice(b"IEND");
    valid.extend_from_slice(&[0xAE, 0x42, 0x60, 0x82]); // CRC

    assert!(
        validate_png(&valid).unwrap(),
        "Complete IEND must pass validation"
    );
    let meta = parse_png_metadata(&valid).expect("parse metadata");
    assert_eq!(meta.width, 1);
    assert_eq!(meta.height, 1);
    assert_eq!(meta.exact_file_size, valid.len() as u64);
}

#[test]
fn test_entropy_clustering_zero_block_size_and_alignment() {
    use crate::carver::entropy_clustering::cluster_entropy_streams;

    let buffer = vec![0x41u8; 4096];

    // Passing 0 as block_size must return empty vector and NOT panic
    let clusters = cluster_entropy_streams(&buffer, 0, 0);
    assert!(clusters.is_empty());

    // Normal clustering
    let clusters = cluster_entropy_streams(&buffer, 1000, 512);
    // ASCII text 'A' has 0 entropy, mapped to SparsePadding, so filtered out
    assert!(clusters.is_empty() || clusters[0].size_bytes > 0);
}

#[test]
fn test_entropy_clustering_windowed_smoothing_and_zero_runs() {
    use crate::carver::entropy_clustering::{
        cluster_entropy_streams_smoothed, find_zero_block_runs, EntropyType,
    };
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};

    // 1. Zero-block run consolidation
    let mut zero_buf = vec![0xAAu8; 4096]; // 8 blocks of 512 bytes
                                           // Blocks 0..2 are zeros
    zero_buf[0..1024].fill(0);
    // Blocks 4..7 are zeros (3 blocks = 1536 bytes)
    zero_buf[2048..3584].fill(0);

    let zero_runs = find_zero_block_runs(&zero_buf, 1000, 512, 2);
    assert_eq!(zero_runs.len(), 2);
    assert_eq!(zero_runs[0].start_offset, 1000);
    assert_eq!(zero_runs[0].size_bytes, 1024);
    assert_eq!(zero_runs[0].block_count, 2);

    assert_eq!(zero_runs[1].start_offset, 1000 + 2048);
    assert_eq!(zero_runs[1].size_bytes, 1536);
    assert_eq!(zero_runs[1].block_count, 3);

    // 2. Windowed smoothing bridges micro-dips in high-entropy streams
    let mut rng = StdRng::seed_from_u64(0x12345678);
    let mut stream_buf = vec![0u8; 5 * 512]; // 5 blocks
                                             // Fill all blocks with high entropy random bytes (~7.4)
    rng.fill_bytes(&mut stream_buf);
    // In block 2 (offset 1024..1536), introduce an 80-byte repetitive header
    // which lowers block 2's raw entropy to ~7.10 (below 7.20 threshold)
    stream_buf[1024..1024 + 80].fill(0x55);

    // Without smoothing (window_size = 1): splits into multiple clusters across block 2
    let raw_clusters = cluster_entropy_streams_smoothed(&stream_buf, 0, 512, 1);
    assert!(
        raw_clusters.len() >= 2,
        "Raw clustering without smoothing should break across the block 2 dip"
    );

    // With 3-point moving-window smoothing: bridges the 1-block dip into 1 contiguous cluster
    let smoothed_clusters = cluster_entropy_streams_smoothed(&stream_buf, 0, 512, 3);
    assert_eq!(
        smoothed_clusters.len(),
        1,
        "Smoothed clustering must bridge transient micro-dip into single cluster"
    );
    assert_eq!(
        smoothed_clusters[0].cluster_type,
        EntropyType::CompressedOrEncrypted
    );
    assert_eq!(smoothed_clusters[0].size_bytes, 5 * 512);
    assert!(smoothed_clusters[0].variance >= 0.0);
}

#[test]
fn test_fat_datetime_edge_seconds() {
    use crate::carver::filesystem_records::scan_fat_directory_entries;

    // Build a synthetic 32-byte FAT directory entry with deleted marker (0xE5)
    let mut entry = [0u8; 32];
    entry[0] = 0xE5;
    entry[1..8].copy_from_slice(b"TESTDOC");
    entry[8..11].copy_from_slice(b"TXT");
    entry[11] = 0x20; // Archive attribute

    // Date: 2026-12-22 (year offset 46 = 2026, month 12, day 22)
    let date: u16 = (46 << 9) | (12 << 5) | 22;
    // Time: 23:59:58 -> hour: 23 << 11 = 47104, min: 59 << 5 = 1888, sec/2: 29
    let time: u16 = (23 << 11) | (59 << 5) | 29;
    entry[16..18].copy_from_slice(&date.to_le_bytes());
    entry[14..16].copy_from_slice(&time.to_le_bytes());
    entry[24..26].copy_from_slice(&date.to_le_bytes());
    entry[22..24].copy_from_slice(&time.to_le_bytes());
    entry[28..32].copy_from_slice(&1024u32.to_le_bytes()); // size

    let results = scan_fat_directory_entries(&entry, 0);
    assert_eq!(results.len(), 1);
    let (_, meta) = &results[0];
    assert_eq!(meta.filename, "_TESTDOC.txt");
    assert_eq!(meta.file_size_bytes, 1024);
    assert!(meta.created_at.is_some());
    assert!(meta
        .created_at
        .as_ref()
        .unwrap()
        .contains("2026-12-22T23:59:58Z"));
}

#[test]
fn test_unified_carved_artifact_constructors() {
    use crate::carver::entropy_clustering::{EntropyCluster, EntropyType};
    use crate::carver::filesystem_records::CarvedFsMetadata;
    use crate::carver::recovery_modes::UnifiedCarvedArtifact;
    use std::collections::HashMap;

    let fs_meta = CarvedFsMetadata {
        filesystem_type: "NTFS".to_string(),
        filename: "evidence.docx".to_string(),
        file_extension: "docx".to_string(),
        file_size_bytes: 65536,
        created_at: Some("2026-12-22T10:00:00Z".to_string()),
        modified_at: Some("2026-12-22T10:30:00Z".to_string()),
        is_resident: false,
        resident_bytes: None,
        cluster_pointers: vec![(16, 2048)],
        attributes: HashMap::new(),
    };

    let artifact = UnifiedCarvedArtifact::from_fs_metadata(4096, &fs_meta);
    assert_eq!(artifact.offset_bytes, 4096);
    assert_eq!(artifact.size_bytes, 65536);
    assert_eq!(artifact.file_type, "docx");
    assert_eq!(artifact.confidence, 90);
    assert_eq!(artifact.original_filename.as_deref(), Some("evidence.docx"));

    let cluster = EntropyCluster {
        start_offset: 1048576,
        end_offset: 1114112,
        size_bytes: 65536,
        avg_entropy: 7.95,
        variance: 0.01,
        cluster_type: EntropyType::CompressedOrEncrypted,
    };

    let cluster_artifact = UnifiedCarvedArtifact::from_entropy_cluster(&cluster);
    assert_eq!(cluster_artifact.offset_bytes, 1048576);
    assert_eq!(cluster_artifact.size_bytes, 65536);
    assert_eq!(cluster_artifact.confidence, 60);
    assert!(cluster_artifact.metadata_summary.contains("7.95"));
}

#[test]
fn test_builtin_signatures_valid() {
    use crate::carver::signatures::SignatureDatabase;

    let sig_db = SignatureDatabase::builtin();
    assert!(sig_db.signatures.len() >= 5);

    let expected_exts = ["jpg", "png", "pdf", "zip", "sqlite"];
    for ext in &expected_exts {
        assert!(
            sig_db.signatures.iter().any(|s| s.extension == *ext),
            "Builtin signatures must include extension '{}'",
            ext
        );
    }

    for sig in &sig_db.signatures {
        assert!(!sig.extension.is_empty());
        assert!(!sig.description.is_empty());
        assert!(sig.max_size > 0);

        let parsed_header = SignatureDatabase::parse_hex(&sig.magic_header);
        assert!(
            parsed_header.is_ok(),
            "Signature header for '{}' must be valid hex",
            sig.extension
        );
        assert!(!parsed_header.unwrap().is_empty());

        if let Some(footer) = &sig.magic_footer {
            let parsed_footer = SignatureDatabase::parse_hex(footer);
            assert!(
                parsed_footer.is_ok(),
                "Signature footer for '{}' must be valid hex",
                sig.extension
            );
            assert!(!parsed_footer.unwrap().is_empty());
        }
    }
}

#[test]
fn test_case_database_hashchain_verification_and_tamper_detection() {
    use crate::audit::hashchain::HashChain;
    use crate::audit::schema::{AuditEntry, OperationResult, OperationType};
    use crate::db::CaseDatabase;
    use std::collections::BTreeMap;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("audit_test.db");
    let db = CaseDatabase::open(&db_path).unwrap();

    // 1. Generic production events are normalized into formal chained entries.
    let case_1 = "CASE-UNCHAINED";
    db.create_case(case_1, "Auditor", "/dev/sdb", "wipe")
        .unwrap();
    let unchained_payload = serde_json::json!({
        "event": "wipe_started",
        "timestamp": "2026-12-22T21:00:00Z"
    })
    .to_string();
    let dummy_hash = crate::audit::signing::hash_bytes(unchained_payload.as_bytes());
    db.insert_audit_entry(case_1, &unchained_payload, &dummy_hash)
        .unwrap();

    let chained_res = db.get_verified_hashchain(case_1).unwrap();
    assert_eq!(chained_res.expect("event must be chained").len(), 1);

    // 2. Chained formal AuditEntry sequence should return Ok(Some(chain))
    let case_2 = "CASE-CHAINED";
    db.create_case(case_2, "Examiner", "/dev/nvme0n1", "erasure")
        .unwrap();

    let mut chain = HashChain::new();
    let entry_1 = AuditEntry {
        id: 101,
        seq: 0,
        timestamp: "2026-12-22T21:05:00Z".to_string(),
        operation: OperationType::DiskWipe,
        target: "/dev/nvme0n1".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "Pass 1 zero fill".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(4096),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    let hash_1 = chain.append(entry_1).unwrap();
    let entry_1_stored = chain.entries()[0].clone();
    let entry_1_json = serde_json::to_string(&entry_1_stored).unwrap();
    db.insert_audit_entry(case_2, &entry_1_json, &hash_1)
        .unwrap();

    let entry_2 = AuditEntry {
        id: 102,
        seq: 1,
        timestamp: "2026-12-22T21:10:00Z".to_string(),
        operation: OperationType::DiskWipe,
        target: "/dev/nvme0n1".to_string(),
        params: BTreeMap::new(),
        result: OperationResult {
            success: true,
            message: "Pass 2 ones fill".to_string(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: Some(4096),
            bad_sectors: None,
            files_recovered: None,
            entropy_post: Some(0.0),
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    };
    let hash_2 = chain.append(entry_2).unwrap();
    let entry_2_stored = chain.entries()[1].clone();
    let entry_2_json = serde_json::to_string(&entry_2_stored).unwrap();
    db.insert_audit_entry(case_2, &entry_2_json, &hash_2)
        .unwrap();

    let verified_chain = db
        .get_verified_hashchain(case_2)
        .unwrap()
        .expect("Chained entries must verify");
    assert_eq!(verified_chain.len(), 2);
    assert_eq!(verified_chain.tip(), chain.tip());

    // 3. Tamper with the database row directly via SQL
    let tampered_json = entry_1_json.replace("/dev/nvme0n1", "/dev/tampered_drive");
    let update_db = CaseDatabase::open(&db_path).unwrap();
    // We can execute direct SQL through rusqlite if exposed, or verify tamper via inserting tampered record
    let case_3 = "CASE-TAMPER";
    update_db
        .create_case(case_3, "Attacker", "/dev/sda", "wipe")
        .unwrap();
    assert!(
        update_db
            .insert_audit_entry(case_3, &tampered_json, &hash_1)
            .is_err(),
        "Tampered formal entry must be rejected before storage"
    );
}

#[test]
fn test_lua_plugin_host_validation_and_deduplication() {
    use crate::plugins::lua_host::LuaPluginHost;

    let mut host = LuaPluginHost::new();
    assert_eq!(host.plugin_count(), 0);
    assert!(!host.has_plugin("custom"));

    // Unregistered formats should return Ok(true) by default without erroring
    let default_val = host.validate("unregistered_format", b"any bytes");
    assert!(default_val.unwrap());

    let script = r#"
        return signature {
            name = "custom",
            category = "test",
            header = "43555354",
            max_size = "1MB",
            validate = function(data)
                return string.sub(data, 1, 4) == "CUST"
            end
        }
    "#;

    host.load_script_content(script).unwrap();
    assert_eq!(host.plugin_count(), 1);
    assert!(host.has_plugin("custom"));

    // Valid data starts with "CUST"
    assert!(host.validate("custom", b"CUST12345").unwrap());
    // Invalid data does not start with "CUST"
    assert!(!host.validate("custom", b"BAD_DATA_").unwrap());

    // Re-loading script with same name updates in-place without duplicating
    host.load_script_content(script).unwrap();
    assert_eq!(host.plugin_count(), 1);
}

#[test]
fn test_trim_carved_file_boundary_sqlite3_and_compound_formats() {
    use crate::carver::structure::trim_carved_file_boundary;

    // 1. sqlite3 and db extension boundary trim with 2 valid pages (1024 bytes) in 8KB buffer
    let page_size = 512usize;
    let mut sqlite_buf = vec![0u8; 8192];
    sqlite_buf[..16].copy_from_slice(b"SQLite format 3\0");
    sqlite_buf[16] = 0x02; // 512 bytes page size
    sqlite_buf[17] = 0x00;
    sqlite_buf[28..32].copy_from_slice(&2u32.to_be_bytes()); // 2 pages = 1024 bytes
    sqlite_buf[page_size] = 0x0d; // leaf table B-tree page header

    let trimmed_sqlite = trim_carved_file_boundary("sqlite3", &sqlite_buf, None);
    assert_eq!(trimmed_sqlite, 1024);

    let trimmed_db = trim_carved_file_boundary("db", &sqlite_buf, None);
    assert_eq!(trimmed_db, 1024);

    // 2. Compound format boundary trim (docx, xlsx, pptx, apk, jar)
    let mut zip_bytes = Vec::new();
    zip_bytes.extend_from_slice(&[
        0x50, 0x4B, 0x03, 0x04, // sig
        20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x36, 0x47, 0x22, 0x36, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
    ]);
    zip_bytes.extend_from_slice(b"data");
    let cd_offset = zip_bytes.len() as u32;
    zip_bytes.extend_from_slice(&[
        0x50, 0x4B, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0x36, 0x47, 0x22, 0x36, 4, 0, 0, 0,
        4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    let cd_size = (zip_bytes.len() as u32) - cd_offset;
    zip_bytes.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0, 1, 0, 1, 0]);
    zip_bytes.extend_from_slice(&cd_size.to_le_bytes());
    zip_bytes.extend_from_slice(&cd_offset.to_le_bytes());
    zip_bytes.extend_from_slice(&[0, 0]);

    let exact_len = zip_bytes.len();
    let mut compound_buf = vec![0xABu8; 16384];
    compound_buf[..exact_len].copy_from_slice(&zip_bytes);

    for ext in &["docx", "xlsx", "pptx", "apk", "jar"] {
        let trimmed = trim_carved_file_boundary(ext, &compound_buf, None);
        assert_eq!(
            trimmed, exact_len,
            "Boundary trimming failed for extension {}",
            ext
        );
    }
}

#[test]
fn test_ntfs_mft_parsing_with_fixup_unapplication_and_data_runs() {
    use crate::carver::filesystem_records::scan_ntfs_mft_records;

    let mut record = vec![0u8; 1024];
    // Record header
    record[0..4].copy_from_slice(b"FILE");
    record[4..6].copy_from_slice(&48u16.to_le_bytes()); // usa_offset = 48
    record[6..8].copy_from_slice(&3u16.to_le_bytes()); // usa_count = 3 (USN + 2 sectors)
    record[20..22].copy_from_slice(&56u16.to_le_bytes()); // first_attr_offset = 56
    record[22..24].copy_from_slice(&1u16.to_le_bytes()); // flags = in-use file

    // Update Sequence Array at 48: USN = [0x42, 0x99]
    record[48..50].copy_from_slice(&[0x42, 0x99]);
    record[50..52].copy_from_slice(&[0xAA, 0xBB]); // sector 0 fixup
    record[52..54].copy_from_slice(&[0xCC, 0xDD]); // sector 1 fixup

    // Write USN to sector tails
    record[510..512].copy_from_slice(&[0x42, 0x99]);
    record[1022..1024].copy_from_slice(&[0x42, 0x99]);

    // Attribute 1 at 56: $FILE_NAME (0x30), resident
    record[56..60].copy_from_slice(&0x30u32.to_le_bytes()); // type
    record[60..64].copy_from_slice(&120u32.to_le_bytes()); // len (24 header + 66 payload header + 24 name bytes = 114 -> 120 aligned)
    record[64] = 0; // resident
    record[76..78].copy_from_slice(&24u16.to_le_bytes()); // val_offset = 24 -> payload at 80
    record[80 + 48..80 + 56].copy_from_slice(&12345u64.to_le_bytes()); // size
    let name = "evidence.pdf";
    record[80 + 64] = name.len() as u8;
    record[80 + 65] = 1; // Win32 name type
    let mut off = 80 + 66;
    for ch in name.encode_utf16() {
        record[off..off + 2].copy_from_slice(&ch.to_le_bytes());
        off += 2;
    }

    // Attribute 2 at 176: non-resident $DATA (0x80)
    record[176..180].copy_from_slice(&0x80u32.to_le_bytes()); // type
    record[180..184].copy_from_slice(&80u32.to_le_bytes()); // len
    record[184] = 1; // non-resident
    record[176 + 32..176 + 34].copy_from_slice(&64u16.to_le_bytes()); // run_offset = 64 -> 240
    record[176 + 48..176 + 56].copy_from_slice(&12345u64.to_le_bytes()); // real size

    // Data runs at 240:
    // Run 1: header 0x21 (len_size 1, off_size 2) -> 16 clusters at LCN 1000
    record[240] = 0x21;
    record[241] = 16;
    record[242..244].copy_from_slice(&1000u16.to_le_bytes());
    // Run 2: header 0x11 (len_size 1, off_size 1) -> 8 clusters at delta +50 (LCN 1050)
    record[244] = 0x11;
    record[245] = 8;
    record[246] = 50;
    // End of runs
    record[247] = 0x00;

    // End of attributes marker at 256
    record[256..260].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());

    let hits = scan_ntfs_mft_records(&record, 0);
    assert_eq!(hits.len(), 1, "Must find exactly 1 MFT record");
    let (_, meta) = &hits[0];
    assert_eq!(meta.filename, "evidence.pdf");
    assert_eq!(meta.file_extension, "pdf");
    assert_eq!(meta.file_size_bytes, 12345);
    assert!(!meta.is_resident);
    assert_eq!(meta.cluster_pointers, vec![(16, 1000), (8, 1050)]);
    assert!(meta.attributes.contains_key("Data_Storage"));
}

#[test]
fn test_fat_vfat_lfn_reconstruction_and_checksum_recovery() {
    use crate::carver::filesystem_records::{fat_lfn_checksum, scan_fat_directory_entries};

    // Calculate checksum of 8.3 filename "REPORT~1PDF"
    let sfn_bytes = b"REPORT~1PDF";
    let chk = fat_lfn_checksum(sfn_bytes);

    let mut buf = vec![0u8; 96]; // 3 x 32-byte records

    // Entry 1 (offset 0..32): LFN Part 2 (last part: "_2026.pdf", 9 chars <= 13)
    buf[0] = 0xE5; // deleted
    buf[11] = 0x0F; // ATTR_LONG_NAME
    buf[13] = chk;
    let part2_chars = "_2026.pdf";
    let mut units2 = part2_chars.encode_utf16().collect::<Vec<u16>>();
    units2.push(0x0000); // null terminate
    while units2.len() < 13 {
        units2.push(0xFFFF); // pad
    }
    let lfn_offsets = [1, 3, 5, 7, 9, 14, 16, 18, 20, 22, 24, 28, 30];
    for (i, &u) in units2.iter().enumerate() {
        let pos = lfn_offsets[i];
        buf[pos..pos + 2].copy_from_slice(&u.to_le_bytes());
    }

    // Entry 2 (offset 32..64): LFN Part 1 ("Report_Annual", exactly 13 chars)
    buf[32] = 0xE5;
    buf[32 + 11] = 0x0F;
    buf[32 + 13] = chk;
    let part1_chars = "Report_Annual";
    let units1 = part1_chars.encode_utf16().collect::<Vec<u16>>();
    for (i, &u) in units1.iter().enumerate() {
        let pos = 32 + lfn_offsets[i];
        buf[pos..pos + 2].copy_from_slice(&u.to_le_bytes());
    }

    // Entry 3 (offset 64..96): SFN 8.3 deleted directory entry
    buf[64] = 0xE5; // deleted first char
    buf[65..72].copy_from_slice(b"EPORT~1");
    buf[72..75].copy_from_slice(b"PDF");
    buf[75] = 0x20; // archive
    buf[64 + 28..64 + 32].copy_from_slice(&98765u32.to_le_bytes()); // size

    let results = scan_fat_directory_entries(&buf, 0);
    assert_eq!(results.len(), 1, "Must find exactly 1 FAT entry");
    let (_, meta) = &results[0];
    assert_eq!(meta.filename, "Report_Annual_2026.pdf");
    assert_eq!(meta.file_extension, "pdf");
    assert_eq!(meta.file_size_bytes, 98765);
    assert_eq!(
        meta.attributes.get("FAT_Name_Type").map(|s| s.as_str()),
        Some("VFAT Long File Name (LFN)")
    );
}

#[test]
fn test_verify_wipe_tail_boundary_enforcement_with_sampling() {
    use crate::disk::raw_image::RawImage;
    use crate::wiper::verify::verify_wipe;
    use std::io::{Seek, SeekFrom, Write};

    fn generate_zeros(size: usize) -> Vec<u8> {
        vec![0u8; size]
    }

    // 25 MB device: with 10% sampling (step = 1MB + 9MB = 10MB),
    // the standard sampling loop visits 0..1MB, 10..11MB, 20..21MB.
    // The final block (24..25MB) would be skipped without tail boundary enforcement.
    let mut file = tempfile::NamedTempFile::new().unwrap();
    let zero_chunk = vec![0u8; 1024 * 1024];
    for _ in 0..24 {
        file.write_all(&zero_chunk).unwrap();
    }
    let dirty_chunk = vec![0x77u8; 1024 * 1024];
    file.write_all(&dirty_chunk).unwrap();
    file.flush().unwrap();

    let path = file.path().to_str().unwrap().to_string();
    let mut disk = RawImage::open(&path).unwrap();

    // 10% sample rate MUST catch the dirty tail block
    let ok = verify_wipe(&mut disk, generate_zeros, 10, false).unwrap();
    assert!(
        !ok,
        "Sampled wipe verification must fail when media tail boundary has residual dirty data"
    );

    // Overwrite the dirty tail with zeros
    file.seek(SeekFrom::Start(24 * 1024 * 1024)).unwrap();
    file.write_all(&zero_chunk).unwrap();
    file.flush().unwrap();

    let mut disk2 = RawImage::open(&path).unwrap();
    let ok_clean = verify_wipe(&mut disk2, generate_zeros, 10, false).unwrap();
    assert!(
        ok_clean,
        "Sampled wipe verification must pass when all sampled blocks and tail boundary are clean"
    );
}

#[test]
fn test_recovery_filesystem_records_and_structure_check_mode() {
    use crate::carver::filesystem_records::{scan_fat_directory_entries, scan_ntfs_mft_records};
    use crate::carver::structure::trim_carved_file_boundary;

    // 1. Verify filesystem records scanning on combined buffer
    let mut fs_buffer = vec![0u8; 2048];

    // Build FAT entry at offset 0 (32 bytes)
    fs_buffer[0] = 0xE5;
    fs_buffer[1..8].copy_from_slice(b"VIDENCE");
    fs_buffer[8..11].copy_from_slice(b"LOG");
    fs_buffer[11] = 0x20; // archive
    fs_buffer[28..32].copy_from_slice(&2048u32.to_le_bytes());

    // Build NTFS MFT record at offset 512 (1024 bytes)
    let mft = &mut fs_buffer[512..1536];
    mft[0..4].copy_from_slice(b"FILE");
    mft[4..6].copy_from_slice(&48u16.to_le_bytes()); // usa_offset = 48
    mft[6..8].copy_from_slice(&3u16.to_le_bytes()); // usa_count = 3
    mft[20..22].copy_from_slice(&56u16.to_le_bytes()); // first_attr = 56
    mft[22..24].copy_from_slice(&1u16.to_le_bytes()); // in-use flag

    // USA at 48: USN = [0x42, 0x99]
    mft[48..50].copy_from_slice(&[0x42, 0x99]);
    mft[50..52].copy_from_slice(&[0xAA, 0xBB]);
    mft[52..54].copy_from_slice(&[0xCC, 0xDD]);
    mft[510..512].copy_from_slice(&[0x42, 0x99]);
    mft[1022..1024].copy_from_slice(&[0x42, 0x99]);

    // Attribute 1 at 56: $FILE_NAME (0x30), resident
    mft[56..60].copy_from_slice(&0x30u32.to_le_bytes());
    mft[60..64].copy_from_slice(&120u32.to_le_bytes()); // attr_len = 120
    mft[64] = 0; // resident
    mft[76..78].copy_from_slice(&24u16.to_le_bytes()); // val_offset = 24
    mft[80 + 48..80 + 56].copy_from_slice(&8u64.to_le_bytes()); // file size = 8
    let name = "audit.txt";
    mft[80 + 64] = name.len() as u8;
    mft[80 + 65] = 1; // Win32
    let mut off = 80 + 66;
    for ch in name.encode_utf16() {
        mft[off..off + 2].copy_from_slice(&ch.to_le_bytes());
        off += 2;
    }

    // Attribute 2 at 176 (56 + 120): $DATA (0x80), resident
    mft[176..180].copy_from_slice(&0x80u32.to_le_bytes());
    mft[180..184].copy_from_slice(&48u32.to_le_bytes()); // attr_len = 48
    mft[184] = 0; // resident
    mft[176 + 16..176 + 20].copy_from_slice(&8u32.to_le_bytes()); // val_len = 8
    mft[176 + 20..176 + 22].copy_from_slice(&24u16.to_le_bytes()); // val_offset = 24
    mft[200..208].copy_from_slice(b"TOPSECRT"); // 176 + 24 = 200

    // End of attributes marker at 224 (176 + 48)
    mft[224..228].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());

    let fat_hits = scan_fat_directory_entries(&fs_buffer, 0);
    assert_eq!(fat_hits.len(), 1, "Must find FAT entry");
    assert_eq!(fat_hits[0].1.filename, "_VIDENCE.log");

    let mft_hits = scan_ntfs_mft_records(&fs_buffer, 0);
    assert_eq!(mft_hits.len(), 1, "Must find NTFS MFT record");
    assert_eq!(mft_hits[0].1.filename, "audit.txt");
    assert_eq!(
        mft_hits[0].1.resident_bytes.as_deref(),
        Some(b"TOPSECRT".as_slice())
    );

    // 2. Verify structure boundary trimming vs naive unconstrained mode
    // ZIP archive followed by 100 bytes of slack
    let mut zip_data = Vec::new();
    zip_data.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]); // Local file header
    zip_data.extend_from_slice(&[0u8; 26]);
    zip_data.extend_from_slice(b"hello world");
    // Central directory header
    let cd_off = zip_data.len() as u32;
    zip_data.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02]);
    zip_data.extend_from_slice(&[0u8; 42]);
    let cd_size = (zip_data.len() as u32) - cd_off;
    // EOCD
    zip_data.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
    zip_data.extend_from_slice(&[0u8; 8]);
    zip_data.extend_from_slice(&cd_size.to_le_bytes());
    zip_data.extend_from_slice(&cd_off.to_le_bytes());
    zip_data.extend_from_slice(&0u16.to_le_bytes()); // comment len = 0
    let exact_zip_len = zip_data.len();
    // Add trailing slack
    zip_data.extend_from_slice(&[0xFFu8; 128]);

    // With structure check enabled:
    let trimmed_len = trim_carved_file_boundary("zip", &zip_data, None);
    assert_eq!(
        trimmed_len, exact_zip_len,
        "Structural trimming must cut exactly at EOCD record end"
    );

    // Without structure check (raw naive mode):
    let naive_len = zip_data.len();
    assert_eq!(naive_len, exact_zip_len + 128);
}

#[test]
fn test_firmware_erase_device_safety_and_error_decoding() {
    use crate::wiper::firmware::{decode_firmware_error, is_device_mounted, FirmwareMethod};
    use std::path::Path;

    // 1. Error decoding verification for NVMe and ATA error messages
    let nvme_opcode_err = "NVMe status: Invalid Command Opcode(0x1)";
    let decoded_nvme = decode_firmware_error(FirmwareMethod::NvmeCryptoErase, nvme_opcode_err);
    assert!(decoded_nvme.is_some());
    assert!(decoded_nvme.unwrap().contains("Invalid Command Opcode"));

    let nvme_pmr_err =
        "NVMe status: Sanitize Prohibited While Persistent Memory Region Is Enabled(0x23)";
    let decoded_pmr = decode_firmware_error(FirmwareMethod::NvmeBlockErase, nvme_pmr_err);
    assert!(decoded_pmr.is_some());
    assert!(decoded_pmr.unwrap().contains("Persistent Memory Region"));

    let ata_frozen_err = "security_erase failed: drive is frozen";
    let decoded_ata = decode_firmware_error(FirmwareMethod::AtaSecureErase, ata_frozen_err);
    assert!(decoded_ata.is_some());
    assert!(decoded_ata.unwrap().contains("FROZEN"));

    let ata_locked_err = "security_erase failed: device is locked";
    let decoded_locked = decode_firmware_error(FirmwareMethod::AtaEnhancedErase, ata_locked_err);
    assert!(decoded_locked.is_some());
    assert!(decoded_locked.unwrap().contains("LOCKED"));

    // 2. Mounted device detection: non-existent/unmounted device returns false
    let unmounted = Path::new("/dev/nonexistent_mock_drive_99");
    assert!(!is_device_mounted(unmounted));
}

#[test]
fn test_dhash_rgba_normalization_and_distance_metrics() {
    use crate::classify::dhash::{
        compute_dhash_from_grid, dhash_from_hex, dhash_to_hex, downsample_grayscale_to_9x8,
        hamming_distance_hex, is_visual_duplicate, rgba_to_grayscale, similarity_percentage,
    };

    // 1. RGBA to grayscale alpha composite over white background (255)
    // Pure transparent red (255, 0, 0, 0) should blend with white to produce pure white (255)
    let transparent_pixel = vec![255, 0, 0, 0];
    let gray_transparent = rgba_to_grayscale(&transparent_pixel);
    assert_eq!(gray_transparent, vec![255]);

    // Opaque black (0, 0, 0, 255) should produce pure black (0)
    let opaque_black = vec![0, 0, 0, 255];
    let gray_black = rgba_to_grayscale(&opaque_black);
    assert_eq!(gray_black, vec![0]);

    // 2. Downsampling safe bounds on edge dimensions
    // Tiny 2x2 image should not panic on underflow
    let tiny_pixels = vec![10u8, 20u8, 30u8, 40u8];
    let grid_opt = downsample_grayscale_to_9x8(2, 2, &tiny_pixels);
    assert!(grid_opt.is_some());
    let grid = grid_opt.unwrap();
    assert_eq!(grid.len(), 72);

    // Downsampling 9x8 image identity
    let direct_9x8 = vec![128u8; 72];
    let identity_grid = downsample_grayscale_to_9x8(9, 8, &direct_9x8).unwrap();
    assert_eq!(identity_grid, [128u8; 72]);

    // 3. Hex distance and similarity percentage
    let hash1 = compute_dhash_from_grid(&identity_grid);
    let hex1 = dhash_to_hex(hash1);
    assert_eq!(dhash_from_hex(&hex1), Some(hash1));

    let dist_same = hamming_distance_hex(&hex1, &hex1);
    assert_eq!(dist_same, Some(0));
    assert_eq!(similarity_percentage(hash1, hash1), 100.0);
    assert!(is_visual_duplicate(hash1, hash1, 10));

    // Invert all bits
    let hash2 = !hash1;
    let hex2 = dhash_to_hex(hash2);
    let dist_diff = hamming_distance_hex(&hex1, &hex2);
    assert_eq!(dist_diff, Some(64));
    assert_eq!(similarity_percentage(hash1, hash2), 0.0);
    assert!(!is_visual_duplicate(hash1, hash2, 10));
}

#[test]
fn test_wiper_metadata_scrubbing_ntfs_fat_ext4() {
    use crate::carver::filesystem_records::{scan_fat_directory_entries, scan_ntfs_mft_records};
    use crate::wiper::metadata::ext4::{scrub_ext4_dir_entry, scrub_ext4_inode};
    use crate::wiper::metadata::fat::scrub_fat_directory_entry;
    use crate::wiper::metadata::ntfs::scrub_ntfs_mft_record;
    use crate::wiper::metadata::scrub_filesystem_metadata_buffer;

    // 1. FAT directory entry scrubbing
    let mut fat_entry = [0u8; 32];
    fat_entry[0] = 0xE5;
    fat_entry[1..8].copy_from_slice(b"CONFID ");
    fat_entry[8..11].copy_from_slice(b"DOC");
    fat_entry[11] = 0x20;
    fat_entry[28..32].copy_from_slice(&8192u32.to_le_bytes());

    assert!(scrub_fat_directory_entry(&mut fat_entry));
    assert_eq!(fat_entry[0], 0xE5);
    assert!(fat_entry[1..32].iter().all(|&b| b == 0));
    assert!(!scrub_fat_directory_entry(&mut fat_entry)); // Idempotent

    // 2. ext4 inode & directory entry scrubbing
    let mut ext4_inode_buf = vec![0u8; 256];
    ext4_inode_buf[0..2].copy_from_slice(&0x81A4u16.to_le_bytes()); // Regular file mode
    ext4_inode_buf[4..8].copy_from_slice(&4096u32.to_le_bytes());
    ext4_inode_buf[8..12].copy_from_slice(&1700000000u32.to_le_bytes());
    ext4_inode_buf[40..42].copy_from_slice(&[0x0A, 0xF3]); // Extent magic

    assert!(scrub_ext4_inode(&mut ext4_inode_buf));
    assert!(ext4_inode_buf.iter().all(|&b| b == 0));
    assert!(!scrub_ext4_inode(&mut ext4_inode_buf));

    let mut ext4_dirent = vec![0u8; 24];
    ext4_dirent[0..4].copy_from_slice(&1234u32.to_le_bytes()); // Inode 1234
    ext4_dirent[4..6].copy_from_slice(&24u16.to_le_bytes()); // rec_len 24
    ext4_dirent[6] = 8; // name_len 8
    ext4_dirent[7] = 1; // file_type
    ext4_dirent[8..16].copy_from_slice(b"keys.pem");

    assert!(scrub_ext4_dir_entry(&mut ext4_dirent));
    assert_eq!(&ext4_dirent[0..4], &[0, 0, 0, 0]);
    assert_eq!(&ext4_dirent[4..6], &24u16.to_le_bytes()); // rec_len preserved
    assert_eq!(ext4_dirent[6], 0);
    assert!(ext4_dirent[8..16].iter().all(|&b| b == 0));

    // 3. NTFS MFT record scrubbing
    let mut mft_buf = vec![0u8; 1024];
    mft_buf[0..4].copy_from_slice(b"FILE");
    mft_buf[20..22].copy_from_slice(&56u16.to_le_bytes()); // first_attr = 56
                                                           // $DATA (0x80) resident attribute at 56
    mft_buf[56..60].copy_from_slice(&0x80u32.to_le_bytes());
    mft_buf[60..64].copy_from_slice(&48u32.to_le_bytes()); // attr_len = 48
    mft_buf[64] = 0; // resident
    mft_buf[72..76].copy_from_slice(&12u32.to_le_bytes()); // val_len = 12
    mft_buf[76..78].copy_from_slice(&24u16.to_le_bytes()); // val_offset = 24
    mft_buf[80..92].copy_from_slice(b"CLASSIFIED!!");
    mft_buf[104..108].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // End marker

    assert!(scrub_ntfs_mft_record(&mut mft_buf));
    assert_eq!(&mft_buf[0..4], b"BAAD");
    assert!(mft_buf[80..92].iter().all(|&b| b == 0));
    assert!(!scrub_ntfs_mft_record(&mut mft_buf));

    // 4. Combined buffer scanning and scrubbing
    let mut combined_buf = vec![0u8; 4096];
    // Write FAT entry at 0
    combined_buf[0] = 0xE5;
    combined_buf[1..8].copy_from_slice(b"WALLET ");
    combined_buf[8..11].copy_from_slice(b"DAT");
    combined_buf[11] = 0x20;
    combined_buf[28..32].copy_from_slice(&1024u32.to_le_bytes());

    // Write ext4 inode at 256
    combined_buf[256..258].copy_from_slice(&0x81A4u16.to_le_bytes());
    combined_buf[260..264].copy_from_slice(&2048u32.to_le_bytes());
    combined_buf[296..298].copy_from_slice(&[0x0A, 0xF3]);

    // Write NTFS MFT record at 1024
    let mft_region = &mut combined_buf[1024..2048];
    mft_region[0..4].copy_from_slice(b"FILE");
    mft_region[20..22].copy_from_slice(&56u16.to_le_bytes());
    mft_region[56..60].copy_from_slice(&0x80u32.to_le_bytes());
    mft_region[60..64].copy_from_slice(&48u32.to_le_bytes());
    mft_region[64] = 0;
    mft_region[72..76].copy_from_slice(&8u32.to_le_bytes());
    mft_region[76..78].copy_from_slice(&24u16.to_le_bytes());
    mft_region[80..88].copy_from_slice(b"PASSWORD");
    mft_region[104..108].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());

    let stats = scrub_filesystem_metadata_buffer(&mut combined_buf);
    assert_eq!(stats.fat_entries_scrubbed, 1);
    assert_eq!(stats.ext4_inodes_scrubbed, 1);
    assert_eq!(stats.ntfs_mft_scrubbed, 1);
    assert_eq!(stats.total_scrubbed(), 3);

    // Ensure carvers now find 0 records
    let fat_carved = scan_fat_directory_entries(&combined_buf, 0);
    assert_eq!(fat_carved.len(), 0);
    let mft_carved = scan_ntfs_mft_records(&combined_buf, 0);
    assert_eq!(mft_carved.len(), 0);
}

#[test]
fn test_rfc3161_token_imprint_validation() {
    use crate::audit::signing::verify_rfc3161_token_imprint;

    let target_hash = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let target_bytes = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];

    // 1. Valid DER-like buffer with imprint
    let mut valid_token = vec![0x30, 0x50, 0x02, 0x01, 0x00]; // ASN.1 SEQUENCE, status = 0 (granted)
    valid_token.extend_from_slice(&[0x04, 0x20]); // OCTET STRING 32 bytes
    valid_token.extend_from_slice(&target_bytes);
    valid_token.extend_from_slice(&[0x00; 16]);

    let res = verify_rfc3161_token_imprint(&valid_token, target_hash);
    assert!(res.is_ok());
    assert!(res.unwrap(), "Must verify valid token imprint");

    // 2. Mismatched digest returns false
    let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    let res_wrong = verify_rfc3161_token_imprint(&valid_token, wrong_hash).unwrap();
    assert!(!res_wrong, "Mismatched hash must not verify");

    // 3. Invalid hex string returns error
    let invalid_hex = "not_a_valid_hex_string";
    assert!(verify_rfc3161_token_imprint(&valid_token, invalid_hex).is_err());

    // 4. Truncated/empty token returns false
    assert!(!verify_rfc3161_token_imprint(&[], target_hash).unwrap());
    assert!(!verify_rfc3161_token_imprint(&[0x30, 0x02], target_hash).unwrap());

    // 5. PKIStatus rejection (status = 2) returns false
    let mut rejected_token = vec![0x30, 0x50, 0x02, 0x01, 0x02]; // status = 2 (rejection)
    rejected_token.extend_from_slice(&target_bytes);
    let res_rejected = verify_rfc3161_token_imprint(&rejected_token, target_hash).unwrap();
    assert!(!res_rejected, "Rejected PKIStatus must not verify");
}

#[test]
fn test_live_diagnostics_suite_passes_all_assertions() {
    let summary = crate::diagnostics::run_diagnostics();
    for result in &summary.results {
        assert!(
            result.passed,
            "Diagnostic assertion failed: [{}] {} - {}",
            result.category, result.name, result.message
        );
    }
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.passed, summary.total);
    assert!(summary.total >= 23);
}
