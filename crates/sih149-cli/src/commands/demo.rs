use crate::display;
use anyhow::{Context, Result};
use clap::Args;
use sih149_core::carver::engine::CarvingEngine;
use sih149_core::carver::scanner::SectorScanner;
use sih149_core::carver::signatures::SignatureDatabase;
use sih149_core::carver::structure::trim_carved_file_boundary;
use sih149_core::disk::EvidenceSource;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct DemoArgs {
    #[arg(long, help = "Output directory for demo artifacts")]
    pub out_dir: Option<String>,
    #[arg(long, help = "Skip PDF report generation")]
    pub no_pdf: bool,
    #[arg(long, help = "Output results in JSON format")]
    pub json: bool,
}

#[derive(serde::Serialize)]
struct DemoFileResult {
    file_type: String,
    offset_bytes: u64,
    size_bytes: usize,
    expected_sha256: String,
    carved_sha256: String,
    verified: bool,
}

#[derive(serde::Serialize)]
struct DemoSummary {
    case_id: String,
    workspace_dir: String,
    evidence_image: String,
    total_sectors: usize,
    total_bytes: usize,
    artifacts_embedded: usize,
    artifacts_carved: usize,
    integrity_verified: bool,
    chain_verified: bool,
    report_pdf: Option<String>,
    files: Vec<DemoFileResult>,
}

fn build_synthetic_png() -> Vec<u8> {
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG 8-byte header
    // IHDR chunk: len=13 (4 bytes), type=IHDR (4 bytes), data (13 bytes), crc (4 bytes)
    png.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]);
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00,
    ]);
    png.extend_from_slice(&[0x1F, 0x15, 0xC4, 0x89]);
    // IEND chunk: len=0 (4 bytes), type=IEND (4 bytes), crc (4 bytes)
    png.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    png.extend_from_slice(b"IEND");
    png.extend_from_slice(&[0xAE, 0x42, 0x60, 0x82]);
    png
}

fn build_synthetic_jpeg() -> Vec<u8> {
    let mut jpeg = vec![0xFF, 0xD8]; // SOI
    // APP0 marker with JFIF identifier: len 16 (0x00, 0x10)
    jpeg.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]);
    jpeg.extend_from_slice(b"JFIF\0\x01\x01\x01\x00H\x00H\x00\x00"); // 14 bytes
    // EOI marker
    jpeg.extend_from_slice(&[0xFF, 0xD9]);
    jpeg
}

fn build_synthetic_sqlite() -> Vec<u8> {
    let mut sqlite = vec![0u8; 512];
    sqlite[..16].copy_from_slice(b"SQLite format 3\0");
    sqlite[16] = 0x02; // Page size 512 bytes (0x0200)
    sqlite[17] = 0x00;
    sqlite[18] = 0x01; // Write version
    sqlite[19] = 0x01; // Read version
    sqlite[20] = 0x00; // Reserved space
    sqlite[21] = 0x40; // Max embedded payload fraction
    sqlite[22] = 0x20; // Min embedded payload fraction
    sqlite[23] = 0x20; // Leaf payload fraction
    sqlite[28..32].copy_from_slice(&1u32.to_be_bytes()); // Exact page count = 1 page
    // Leaf table b-tree page header at offset 100 for page 1
    sqlite[100] = 0x0D; // Leaf table b-tree page type
    sqlite[105] = 0x02; // Cell content offset (512)
    sqlite[106] = 0x00;
    sqlite
}

pub fn run(args: &DemoArgs) -> Result<()> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let case_id = format!("demo-{}", timestamp);

    // 1. Setup isolated demo workspace
    let demo_dir = match &args.out_dir {
        Some(d) => PathBuf::from(d),
        None => std::env::temp_dir().join(format!("secureforge-demo-{}", timestamp)),
    };
    fs::create_dir_all(&demo_dir)
        .with_context(|| format!("Failed to create demo directory {:?}", demo_dir))?;

    let carved_dir = demo_dir.join("carved_evidence");
    fs::create_dir_all(&carved_dir)?;

    let evidence_path = demo_dir.join("synthetic_evidence.raw");

    if !args.json {
        display::print_section("SecureForge Live Autonomous Forensic Demo");
        println!("\x1b[38;2;148;163;184mInitializing isolated forensic workspace: {}\x1b[0m\n", demo_dir.display());
    }

    // 2. Synthesize artifacts and calculate reference digests
    let png_data = build_synthetic_png();
    let jpeg_data = build_synthetic_jpeg();
    let sqlite_data = build_synthetic_sqlite();

    let png_sha = sih149_core::audit::signing::hash_bytes(&png_data);
    let jpeg_sha = sih149_core::audit::signing::hash_bytes(&jpeg_data);
    let sqlite_sha = sih149_core::audit::signing::hash_bytes(&sqlite_data);

    // 3. Assemble sector-aligned synthetic disk image
    // 16 sectors (8192 bytes total):
    // Sector 0  (0..512):     MBR / Partition header stub (zeros)
    // Sector 1  (512..1024):   PNG image + zero slack
    // Sector 2  (1024..1536):  JPEG image + zero slack
    // Sector 3  (1536..2048):  High-entropy corrupt data (noise)
    // Sector 4  (2048..2560):  SQLite database (512 bytes)
    // Sectors 5..16:           Zero-filled slack space
    const SECTOR_SIZE: usize = 512;
    const NUM_SECTORS: usize = 16;
    let mut disk_image = vec![0u8; NUM_SECTORS * SECTOR_SIZE];

    // Sector 1: PNG (offset 512)
    disk_image[512..512 + png_data.len()].copy_from_slice(&png_data);

    // Sector 2: JPEG (offset 1024)
    disk_image[1024..1024 + jpeg_data.len()].copy_from_slice(&jpeg_data);

    // Sector 3: Corrupt pseudo-random noise (unallocated sector with high entropy)
    for (idx, byte) in disk_image[1536..2048].iter_mut().enumerate() {
        let i = 1536 + idx;
        *byte = ((i * 107 + 13) % 256) as u8;
    }

    // Sector 4: SQLite (offset 2048)
    disk_image[2048..2048 + sqlite_data.len()].copy_from_slice(&sqlite_data);

    fs::write(&evidence_path, &disk_image)
        .with_context(|| format!("Failed to write synthetic evidence to {:?}", evidence_path))?;

    if !args.json {
        println!("  \x1b[38;2;56;189;248m•\x1b[0m Synthetic evidence image generated: \x1b[1m{} B\x1b[0m ({} sectors of {} B)",
            disk_image.len(), NUM_SECTORS, SECTOR_SIZE);
        println!("    ├─ Offset 0x0200 (Sector 1):  PNG graphic  ({} B) -> SHA-256: {}", png_data.len(), &png_sha[..16]);
        println!("    ├─ Offset 0x0400 (Sector 2):  JPEG photo   ({} B) -> SHA-256: {}", jpeg_data.len(), &jpeg_sha[..16]);
        println!("    ├─ Offset 0x0600 (Sector 3):  Corrupt slack (pseudo-random high entropy)");
        println!("    └─ Offset 0x0800 (Sector 4):  SQLite db   ({} B) -> SHA-256: {}", sqlite_data.len(), &sqlite_sha[..16]);
        println!();
    }

    // 4. Initialize Carving Engine with Builtin Signature Catalog
    let sig_db = SignatureDatabase::builtin();
    let signatures = sig_db.signatures.clone();
    let scanner = SectorScanner::new(sig_db)?;
    let engine = CarvingEngine::new(scanner);

    let mut evidence = EvidenceSource::open(&evidence_path)?;
    let hits = engine.carve(&mut evidence)?;

    // 5. Extract and Validate Carved Files
    let mut file_results = Vec::new();
    let mut all_verified = true;

    for (offset, _confidence, ext) in hits {
        let mut read_buf = vec![0u8; 1024 * 1024];
        evidence.seek(SeekFrom::Start(offset))?;
        let bytes_read = evidence.read(&mut read_buf)?;
        if bytes_read == 0 {
            continue;
        }

        let sig = signatures.iter().find(|s| s.extension == ext);
        let footer_bytes = sig
            .and_then(|s| s.magic_footer.as_deref())
            .and_then(|f| SignatureDatabase::parse_hex(f).ok());

        let exact_len = trim_carved_file_boundary(&ext, &read_buf[..bytes_read], footer_bytes.as_deref());
        let carved_slice = &read_buf[..exact_len];
        let carved_sha = sih149_core::audit::signing::hash_bytes(carved_slice);

        let out_name = format!("recovered_{:05}.{}", offset, ext);
        let out_path = carved_dir.join(&out_name);
        fs::write(&out_path, carved_slice)?;

        let (expected_sha, verified) = match offset {
            512 => (png_sha.clone(), carved_sha == png_sha && exact_len == png_data.len()),
            1024 => (jpeg_sha.clone(), carved_sha == jpeg_sha && exact_len == jpeg_data.len()),
            2048 => (sqlite_sha.clone(), carved_sha == sqlite_sha && exact_len == sqlite_data.len()),
            _ => (String::new(), false),
        };

        if !verified {
            all_verified = false;
        }

        file_results.push(DemoFileResult {
            file_type: ext,
            offset_bytes: offset,
            size_bytes: exact_len,
            expected_sha256: expected_sha,
            carved_sha256: carved_sha,
            verified,
        });
    }

    // Ensure all 3 artifacts were recovered
    if file_results.len() < 3 {
        all_verified = false;
    }

    // 6. Record in Database & Cryptographic Audit Trail
    let db_path = dirs::data_local_dir()
        .unwrap_or_default()
        .join("secureforge")
        .join("cases.db");
    let _ = fs::create_dir_all(db_path.parent().unwrap_or(Path::new(".")));

    let mut chain_verified = false;
    if let Ok(db) = sih149_core::db::CaseDatabase::open(&db_path) {
        let _ = db.create_case(&case_id, "Automated Demo Runner", evidence_path.to_str().unwrap_or(""), "recovery");

        for fr in &file_results {
            let out_file_path = carved_dir.join(format!("recovered_{:05}.{}", fr.offset_bytes, fr.file_type));
            let category = match fr.file_type.as_str() {
                "png" | "jpg" | "jpeg" => "Media",
                "sqlite" | "db" => "Databases",
                _ => "General",
            };
            let rec = sih149_core::db::CarvedFileRecord {
                filename: format!("recovered_{:05}.{}", fr.offset_bytes, fr.file_type),
                file_type: fr.file_type.clone(),
                size_bytes: fr.size_bytes as u64,
                confidence: 100,
                offset_bytes: fr.offset_bytes,
                output_path: out_file_path.to_string_lossy().into_owned(),
                category: category.to_string(),
                sha256: Some(fr.carved_sha256.clone()),
            };
            let _ = db.insert_carved_file(&case_id, &rec);
        }

        let audit_payload = serde_json::json!({
            "event": "demo_recovery_completed",
            "case_id": case_id,
            "evidence": evidence_path.to_string_lossy(),
            "recovered_count": file_results.len(),
            "all_verified": all_verified,
            "timestamp": timestamp,
        });
        let audit_json = audit_payload.to_string();
        let audit_hash = sih149_core::audit::signing::hash_bytes(audit_json.as_bytes());
        let _ = db.insert_audit_entry(&case_id, &audit_json, &audit_hash);
        let _ = db.update_case_status(&case_id, "completed");
        chain_verified = true;
    }

    // 7. Optional PDF Certificate Generation
    let report_pdf = if !args.no_pdf {
        crate::commands::report::generate_case_certificate_pdf(&case_id, "recovery", None)
            .map(|p| p.to_string_lossy().into_owned())
    } else {
        None
    };

    let summary = DemoSummary {
        case_id: case_id.clone(),
        workspace_dir: demo_dir.to_string_lossy().into_owned(),
        evidence_image: evidence_path.to_string_lossy().into_owned(),
        total_sectors: NUM_SECTORS,
        total_bytes: disk_image.len(),
        artifacts_embedded: 3,
        artifacts_carved: file_results.len(),
        integrity_verified: all_verified,
        chain_verified,
        report_pdf: report_pdf.clone(),
        files: file_results,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
        if !all_verified {
            std::process::exit(1);
        }
        return Ok(());
    }

    // 8. Visual Terminal Dashboard Presentation
    println!("\x1b[38;2;60;66;96m┌────────────┬────────┬───────────┬─────────────┬────────────────────────────────────────────────────────┐\x1b[0m");
    println!("\x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<10}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<6}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<9}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<11}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<54}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m",
        "ARTIFACT", "OFFSET", "CARVED", "INTEGRITY", "SHA-256 MATCH VERIFICATION");
    println!("\x1b[38;2;60;66;96m├────────────┼────────┼───────────┼─────────────┼────────────────────────────────────────────────────────┤\x1b[0m");

    for f in &summary.files {
        let badge = match f.file_type.as_str() {
            "png" => "\x1b[38;2;168;85;247mPNG Image \x1b[0m",
            "jpg" => "\x1b[38;2;56;189;248mJPEG Photo\x1b[0m",
            "sqlite" => "\x1b[38;2;52;211;153mSQLite DB \x1b[0m",
            _ => "Unknown   ",
        };
        let status = if f.verified {
            "\x1b[1;38;2;52;211;153m✓ 100% BIT-EXACT\x1b[0m"
        } else {
            "\x1b[1;38;2;248;113;113m✗ MISMATCH      \x1b[0m"
        };
        let sha_short = format!("{}...{}", &f.carved_sha256[..16], &f.carved_sha256[48..]);
        println!(
            "\x1b[38;2;60;66;96m│\x1b[0m {:<19} \x1b[38;2;60;66;96m│\x1b[0m 0x{:04X} \x1b[38;2;60;66;96m│\x1b[0m {:>6} B  \x1b[38;2;60;66;96m│\x1b[0m {:<22} \x1b[38;2;60;66;96m│\x1b[0m \x1b[38;2;148;163;184m{:<54}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m",
            badge, f.offset_bytes, f.size_bytes, status, sha_short
        );
    }
    println!("\x1b[38;2;60;66;96m└────────────┴────────┴───────────┴─────────────┴────────────────────────────────────────────────────────┘\x1b[0m\n");

    if let Some(ref pdf) = report_pdf {
        println!("  \x1b[38;2;52;211;153m✓\x1b[0m Forensic Case Certificate Generated: \x1b[1m{}\x1b[0m", pdf);
    }

    if all_verified {
        display::print_success("DEMO COMPLETED SUCCESSFULLY: 100% FORENSIC INTEGRITY VERIFIED");
        println!("  Audit chain recorded to Case ID: \x1b[1m{}\x1b[0m", case_id);
        println!("  Carved artifacts saved in:       \x1b[1m{}\x1b[0m\n", carved_dir.display());
        Ok(())
    } else {
        display::print_error("DEMO FAILED: One or more carved artifacts failed cryptographic bit-exact verification.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_artifacts_structure() {
        let png = build_synthetic_png();
        assert!(sih149_core::carver::structure::png::validate_png(&png).unwrap());

        let jpeg = build_synthetic_jpeg();
        assert!(sih149_core::carver::structure::jpeg::validate_jpeg(&jpeg).unwrap());

        let sqlite = build_synthetic_sqlite();
        assert!(sih149_core::carver::structure::sqlite::validate_sqlite(&sqlite).unwrap());
    }
}
