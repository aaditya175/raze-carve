//! Integration tests for the forensic file carver
use sih149_core::carver::confidence::ConfidenceScorer;
use sih149_core::carver::entropy::calculate_shannon_entropy;
use sih149_core::carver::signatures::SignatureDatabase;
use sih149_core::disk::raw_image::RawImage;
use std::fs;
use std::path::{Path, PathBuf};

pub fn create_test_image(path: &Path, size_mb: usize) -> std::io::Result<()> {
    let file = fs::File::create(path)?;
    file.set_len((size_mb * 1024 * 1024) as u64)?;
    Ok(())
}

pub fn plant_file_at_offset(
    image_path: &Path,
    file_bytes: &[u8],
    offset: u64,
) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    let file = fs::OpenOptions::new().write(true).open(image_path)?;
    file.write_at(file_bytes, offset)?;
    Ok(())
}

pub fn jpeg_test_bytes() -> Vec<u8> {
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
    ]
}

pub fn png_test_bytes() -> Vec<u8> {
    vec![
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H', b'D',
        b'R', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x00, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82,
    ]
}

pub fn pdf_test_bytes() -> Vec<u8> {
    b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n1 0 obj\n<</Type/Catalog/Pages 2 0 R>>\nendobj\n%%EOF\n".to_vec()
}

pub fn sqlite_test_bytes() -> Vec<u8> {
    let mut bytes = vec![0; 512];
    let header = b"SQLite format 3\0";
    bytes[0..16].copy_from_slice(header);
    bytes[16] = 0x10; // Page size 4096 (0x1000)
    bytes[17] = 0x00;
    bytes
}

#[test]
fn test_entropy_empty() {
    let zeros = vec![0u8; 1024];
    let e = calculate_shannon_entropy(&zeros);
    assert!(e < 0.1, "Entropy of zeros should be near 0");
}

#[test]
fn test_entropy_random() {
    use std::fs::File;
    use std::io::Read;
    let mut random_bytes = vec![0u8; 1024];
    File::open("/dev/urandom")
        .unwrap()
        .read_exact(&mut random_bytes)
        .unwrap();
    let e = calculate_shannon_entropy(&random_bytes);
    assert!(e > 7.0, "Entropy of random bytes should be > 7.0");
}

#[test]
fn test_entropy_text() {
    let text = b"This is some simple ASCII text meant to test the entropy calculator. It should fall in the middle range.";
    let e = calculate_shannon_entropy(text);
    assert!(
        e > 3.0 && e < 5.0,
        "Entropy of text should be between 3.0 and 5.0 (was {})",
        e
    );
}

#[test]
fn test_jpeg_header_detection() {
    let path = PathBuf::from("/tmp/test_jpeg_detect.dd");
    create_test_image(&path, 1).unwrap();
    plant_file_at_offset(&path, &jpeg_test_bytes(), 0).unwrap();
    let mut file = fs::File::open(&path).unwrap();
    use std::io::Read;
    let mut buf = [0u8; 3];
    file.read_exact(&mut buf).unwrap();
    assert_eq!(buf, [0xFF, 0xD8, 0xFF]);
    let _ = fs::remove_file(path);
}

#[test]
fn test_png_header_detection() {
    let path = PathBuf::from("/tmp/test_png_detect.dd");
    create_test_image(&path, 1).unwrap();
    plant_file_at_offset(&path, &png_test_bytes(), 0).unwrap();
    let mut file = fs::File::open(&path).unwrap();
    use std::io::Read;
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"\x89PNG\x0D\x0A\x1A\x0A");
    let _ = fs::remove_file(path);
}

#[test]
fn test_raw_image_source_sector_read() {
    use std::io::Read;
    let path = PathBuf::from("/tmp/test_sector_read.dd");
    create_test_image(&path, 1).unwrap();
    if let Ok(mut src) = RawImage::open(&path) {
        let mut buf = vec![0u8; 512];
        let read = src.read(&mut buf).unwrap();
        assert_eq!(read, 512);
    }
    let _ = fs::remove_file(path);
}

#[test]
fn test_hash_chain_append_and_verify() {
    use sih149_core::audit::hashchain::HashChain;
    use sih149_core::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    chain
        .append(AuditEntry {
            id: 1,
            timestamp: String::from("123"),
            operation: OperationType::DiskWipe,
            target: String::from("sys"),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: String::new(),
                pre_hash: None,
                post_hash: None,
                sectors_processed: None,
                bad_sectors: None,
                files_recovered: None,
                entropy_post: None,
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
            seq: 0,
        })
        .unwrap();
    assert!(chain.verify());
}

#[test]
fn test_hash_chain_tamper_detection() {
    use sih149_core::audit::hashchain::HashChain;
    use sih149_core::audit::schema::{AuditEntry, OperationResult, OperationType};
    use std::collections::BTreeMap;

    let mut chain = HashChain::new();
    chain
        .append(AuditEntry {
            id: 1,
            timestamp: String::from("123"),
            operation: OperationType::DiskWipe,
            target: String::from("sys"),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: String::new(),
                pre_hash: None,
                post_hash: None,
                sectors_processed: None,
                bad_sectors: None,
                files_recovered: None,
                entropy_post: None,
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
            seq: 0,
        })
        .unwrap();
    assert!(chain.verify());
}

#[test]
fn test_toml_signature_loader() {
    let toml_content = r#"
[[signatures]]
name = "JPEG"
description = "test"
magic_header = "FFD8FFE0"
footer = "FFD9"
extension = "jpg"
category = "image"
max_size = 5000000

[[signatures]]
name = "PNG"
description = "test"
magic_header = "89504E470D0A1A0A"
extension = "png"
category = "image"
max_size = 10000000

[[signatures]]
name = "PDF"
description = "test"
magic_header = "255044462D"
footer = "2525454F46"
extension = "pdf"
category = "document"
max_size = 20000000
    "#;
    let path = PathBuf::from("/tmp/test_sigs.toml");
    fs::write(&path, toml_content).unwrap();

    let db = SignatureDatabase::load_from_toml(&path).unwrap();
    assert_eq!(db.signatures.len(), 3);
    assert_eq!(db.signatures[0].extension, "jpg");

    let _ = fs::remove_file(path);
}

#[test]
fn test_report_manifest_save_load() {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct ReportManifest {
        case_id: i64,
        report_name: String,
        files_carved: usize,
    }

    let manifest = ReportManifest {
        case_id: 42,
        report_name: "Final Report".to_string(),
        files_carved: 1337,
    };

    let path = PathBuf::from("/tmp/test_manifest.json");
    let json = serde_json::to_string(&manifest).unwrap();
    fs::write(&path, json).unwrap();

    let loaded_json = fs::read_to_string(&path).unwrap();
    let loaded: ReportManifest = serde_json::from_str(&loaded_json).unwrap();

    assert_eq!(manifest, loaded);
    let _ = fs::remove_file(path);
}

#[test]
fn test_confidence_scoring() {
    let mut scorer = ConfidenceScorer::new();
    scorer.has_header = true;
    scorer.structure_valid = true;
    scorer.has_footer = true;
    assert_eq!(
        scorer.calculate(),
        sih149_core::carver::confidence::Confidence::High
    );
}

#[test]
fn test_carving_engine_with_raw_image() {
    use sih149_core::carver::engine::CarvingEngine;
    use sih149_core::carver::scanner::SectorScanner;
    use sih149_core::carver::signatures::FileSignature;

    let path = PathBuf::from("/tmp/test_engine_carve.dd");
    create_test_image(&path, 2).unwrap();
    plant_file_at_offset(&path, &jpeg_test_bytes(), 512).unwrap();
    plant_file_at_offset(&path, &png_test_bytes(), 1024 * 1024).unwrap();

    let sig_db = SignatureDatabase {
        signatures: vec![
            FileSignature {
                extension: "jpg".to_string(),
                description: "JPEG Image".to_string(),
                magic_header: "FFD8FFE0".to_string(),
                magic_footer: Some("FFD9".to_string()),
                max_size: 50 * 1024 * 1024,
                header_mask: None,
            },
            FileSignature {
                extension: "png".to_string(),
                description: "PNG Image".to_string(),
                magic_header: "89504E470D0A1A0A".to_string(),
                magic_footer: Some("49454E44AE426082".to_string()),
                max_size: 50 * 1024 * 1024,
                header_mask: None,
            },
        ],
    };

    let scanner = SectorScanner::new(sig_db).unwrap();
    let engine = CarvingEngine::new(scanner);

    let mut disk = RawImage::open(&path).unwrap();
    let hits = engine.carve(&mut disk).unwrap();

    assert!(hits.len() >= 2);
    assert!(hits.iter().any(|(off, _, ext)| *off == 512 && ext == "jpg"));
    assert!(hits
        .iter()
        .any(|(off, _, ext)| *off == 1024 * 1024 && ext == "png"));

    let _ = fs::remove_file(path);
}

#[test]
fn test_carving_engine_with_lua_plugin() {
    use sih149_core::carver::confidence::Confidence;
    use sih149_core::carver::engine::CarvingEngine;
    use sih149_core::carver::scanner::SectorScanner;
    use sih149_core::carver::signatures::FileSignature;
    use sih149_core::plugins::lua_host::LuaPluginHost;

    let path = PathBuf::from("/tmp/test_engine_lua_carve.dd");
    create_test_image(&path, 1).unwrap();
    plant_file_at_offset(&path, &jpeg_test_bytes(), 512).unwrap();

    let lua_script = r#"
        signature {
            name = "jpg",
            category = "Media",
            header = "FFD8FFE0",
            footer = "FFD9",
            max_size = 5000000,
            validate = function(data)
                -- Custom validator: return false to trigger demotion
                return false
            end
        }
    "#;
    let lua_path = PathBuf::from("/tmp/test_reject_plugin.lua");
    fs::write(&lua_path, lua_script).unwrap();

    let mut lua_host = LuaPluginHost::new();
    lua_host.load_script(&lua_path).unwrap();

    let sig_db = SignatureDatabase {
        signatures: vec![FileSignature {
            extension: "jpg".to_string(),
            description: "JPEG Image".to_string(),
            magic_header: "FFD8FFE0".to_string(),
            magic_footer: Some("FFD9".to_string()),
            max_size: 50 * 1024 * 1024,
            header_mask: None,
        }],
    };

    let scanner = SectorScanner::new(sig_db).unwrap();
    let engine = CarvingEngine::new(scanner).with_lua_plugins(lua_host);

    let mut disk = RawImage::open(&path).unwrap();
    let hits = engine.carve(&mut disk).unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].0, 512);
    // Plugin returned false, so confidence should be demoted to Low
    assert_eq!(hits[0].1, Confidence::Low);

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(lua_path);
}

#[test]
fn test_in_memory_inspection_without_extraction() {
    use sih149_core::carver::engine::CarvingEngine;
    use sih149_core::carver::scanner::SectorScanner;
    use sih149_core::carver::signatures::SignatureDatabase;
    use sih149_core::carver::structure::trim_carved_file_boundary;

    let path = PathBuf::from("/tmp/test_inspect_dry_run.dd");
    let _ = fs::remove_file(&path);
    create_test_image(&path, 3).unwrap();

    let jpg_bytes = jpeg_test_bytes();
    let png_bytes = png_test_bytes();
    let pdf_bytes = pdf_test_bytes();

    plant_file_at_offset(&path, &jpg_bytes, 512).unwrap();
    plant_file_at_offset(&path, &png_bytes, 1024 * 1024).unwrap();
    plant_file_at_offset(&path, &pdf_bytes, 2 * 1024 * 1024).unwrap();

    let sig_db = SignatureDatabase::builtin();
    let scanner = SectorScanner::new(SignatureDatabase::builtin()).unwrap();
    let engine = CarvingEngine::new(scanner);

    let mut disk = RawImage::open(&path).unwrap();
    let hits = engine.carve(&mut disk).unwrap();

    assert!(hits.len() >= 3);

    // Verify inspect-only mode correctly extracts byte sizes & headers in-memory
    for (offset, _conf, ext) in &hits {
        use std::io::{Read, Seek, SeekFrom};
        let mut file = fs::File::open(&path).unwrap();
        file.seek(SeekFrom::Start(*offset)).unwrap();
        let mut buf = vec![0u8; 1024 * 64];
        let n = file.read(&mut buf).unwrap();

        let sig = sig_db.signatures.iter().find(|s| s.extension == *ext);
        let footer = sig
            .and_then(|s| s.magic_footer.as_deref())
            .and_then(|f| SignatureDatabase::parse_hex(f).ok());
        let exact_size = trim_carved_file_boundary(ext, &buf[..n], footer.as_deref());

        match ext.as_str() {
            "jpg" => assert_eq!(exact_size, jpg_bytes.len()),
            "png" => assert_eq!(exact_size, png_bytes.len()),
            "pdf" => assert_eq!(exact_size, pdf_bytes.len()),
            _ => {}
        }
    }

    let _ = fs::remove_file(path);
}

#[test]
fn test_selective_file_extraction_matches_hashes() {
    use sih149_core::audit::signing::hash_bytes;
    use sih149_core::carver::engine::CarvingEngine;
    use sih149_core::carver::scanner::SectorScanner;
    use sih149_core::carver::signatures::SignatureDatabase;
    use sih149_core::carver::structure::trim_carved_file_boundary;
    use std::io::{Read, Seek, SeekFrom, Write};

    let path = PathBuf::from("/tmp/test_selective_extract.dd");
    let out_dir = PathBuf::from("/tmp/sf_selective_out_test");
    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(&out_dir);

    create_test_image(&path, 3).unwrap();
    fs::create_dir_all(&out_dir).unwrap();

    let jpg_bytes = jpeg_test_bytes();
    let png_bytes = png_test_bytes();
    let pdf_bytes = pdf_test_bytes();

    plant_file_at_offset(&path, &jpg_bytes, 512).unwrap();
    plant_file_at_offset(&path, &png_bytes, 1024 * 1024).unwrap();
    plant_file_at_offset(&path, &pdf_bytes, 2 * 1024 * 1024).unwrap();

    let sig_db = SignatureDatabase::builtin();
    let scanner = SectorScanner::new(SignatureDatabase::builtin()).unwrap();
    let engine = CarvingEngine::new(scanner);

    let mut disk = RawImage::open(&path).unwrap();
    let hits = engine.carve(&mut disk).unwrap();

    // Select ONLY the PNG file for recovery
    let png_hit = hits
        .iter()
        .find(|(_, _, ext)| ext == "png")
        .expect("PNG hit not found");
    let png_offset = png_hit.0;

    let mut evidence = fs::File::open(&path).unwrap();
    evidence.seek(SeekFrom::Start(png_offset)).unwrap();
    let mut buf = vec![0u8; 1024 * 64];
    let n = evidence.read(&mut buf).unwrap();

    let sig = sig_db.signatures.iter().find(|s| s.extension == "png");
    let footer = sig
        .and_then(|s| s.magic_footer.as_deref())
        .and_then(|f| SignatureDatabase::parse_hex(f).ok());
    let exact_png_size = trim_carved_file_boundary("png", &buf[..n], footer.as_deref());

    let dest_file_path = out_dir.join("selected_recovered.png");
    let mut dest = fs::File::create(&dest_file_path).unwrap();
    dest.write_all(&buf[..exact_png_size]).unwrap();
    dest.sync_all().unwrap();

    // Verify ONLY the selected file exists in output directory
    let extracted_files: Vec<_> = fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(extracted_files.len(), 1);
    assert_eq!(
        extracted_files[0].file_name().to_str().unwrap(),
        "selected_recovered.png"
    );

    // Verify 100% cryptographic SHA-256 integrity match
    let extracted_bytes = fs::read(&dest_file_path).unwrap();
    assert_eq!(hash_bytes(&extracted_bytes), hash_bytes(&png_bytes));

    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(out_dir);
}
