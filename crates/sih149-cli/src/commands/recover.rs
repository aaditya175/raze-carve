use crate::display;
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use sih149_core::carver::engine::CarvingEngine;
use sih149_core::carver::filesystem_records::{scan_fat_directory_entries, scan_ntfs_mft_records};
use sih149_core::carver::scanner::SectorScanner;
use sih149_core::carver::signatures::SignatureDatabase;
use sih149_core::disk::EvidenceSource;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct RecoverArgs {
    #[arg(short, long)]
    pub source: String,
    #[arg(short, long)]
    pub output: Option<String>,
    #[arg(short, long)]
    pub types: Option<String>,
    #[arg(long, default_value = "50")]
    pub min_confidence: u8,
    #[arg(long)]
    pub no_structure_check: bool,
    #[arg(long)]
    pub no_fs_records: bool,
    #[arg(long)]
    pub inspect: bool,
    #[arg(short = 'S', long)]
    pub select: Option<String>,
    #[arg(short = 'i', long)]
    pub interactive: bool,
}

#[derive(Clone, Debug)]
struct CarveCandidate {
    index: usize,
    filename: String,
    file_type: String,
    offset_bytes: u64,
    size_bytes: u64,
    confidence_pct: u8,
    category: String,
    metadata_snippet: Option<String>,
    resident_payload: Option<Vec<u8>>,
}

pub fn run(args: &RecoverArgs) -> anyhow::Result<()> {
    display::print_section("File Recovery");

    if !Path::new(&args.source).exists() {
        anyhow::bail!("Source path does not exist");
    }

    // Load signatures — start with comprehensive builtin catalog, then merge plugins
    let mut signatures = sih149_core::carver::signatures::SignatureDatabase::builtin().signatures;
    let sig_candidates = [
        PathBuf::from("plugins/signatures"),
        PathBuf::from("../plugins/signatures"),
        PathBuf::from("../../plugins/signatures"),
    ];
    if let Some(plugins_dir) = sig_candidates.iter().find(|p| p.is_dir()) {
        if let Ok(toml_sigs) =
            sih149_core::plugins::toml_loader::load_signatures_from_dir(plugins_dir)
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

    let sig_db = SignatureDatabase {
        signatures: signatures.clone(),
    };
    let scanner = SectorScanner::new(sig_db)?;

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

    display::print_section("Scanning...");
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message(format!("Carving files from {}...", args.source));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    let mut evidence = EvidenceSource::open(&args.source)
        .map_err(|e| anyhow::anyhow!("Failed to open evidence {}: {}", args.source, e))?;

    let mut fs_hits = Vec::new();
    if !args.no_fs_records {
        let mut fs_buf = vec![0u8; 1024 * 1024 * 8];
        let read_ok = evidence
            .seek(SeekFrom::Start(0))
            .ok()
            .and_then(|_| evidence.read(&mut fs_buf).ok());
        if let Some(n) = read_ok {
            fs_hits.extend(scan_ntfs_mft_records(&fs_buf[..n], 0));
            fs_hits.extend(scan_fat_directory_entries(&fs_buf[..n], 0));
        }
    }

    let carve_hits = engine
        .carve(&mut evidence)
        .map_err(|e| anyhow::anyhow!(e))?;

    pb.finish_with_message("Carving complete");

    // Collect candidates
    let mut candidates: Vec<CarveCandidate> = Vec::new();
    let mut cand_index: usize = 0;

    // 1. Filesystem hits
    for (offset, meta) in &fs_hits {
        let confidence_pct: u8 = if meta.filesystem_type == "NTFS" {
            95
        } else {
            90
        };
        if confidence_pct < args.min_confidence {
            continue;
        }
        if let Some(types) = &args.types {
            if !types
                .split(',')
                .any(|t| t.trim().eq_ignore_ascii_case(&meta.file_extension))
            {
                continue;
            }
        }

        let safe_name = Path::new(&meta.filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("recovered_file");
        let filename = format!("recovered_{}", safe_name);

        let (size_bytes, payload, meta_snippet) = if let Some(ref p) = meta.resident_bytes {
            (
                p.len() as u64,
                Some(p.clone()),
                Some(format!("NTFS Resident Inode: {}", meta.filename)),
            )
        } else {
            (
                meta.file_size_bytes,
                None,
                Some(format!(
                    "{} Record: {}",
                    meta.filesystem_type, meta.filename
                )),
            )
        };

        candidates.push(CarveCandidate {
            index: cand_index,
            filename,
            file_type: meta.file_extension.clone(),
            offset_bytes: *offset,
            size_bytes,
            confidence_pct,
            category: "FILESYSTEM".to_string(),
            metadata_snippet: meta_snippet,
            resident_payload: payload,
        });
        cand_index += 1;
    }

    // 2. Structural and signature hits
    for (offset, confidence, extension) in &carve_hits {
        let confidence_pct: u8 = match confidence {
            sih149_core::carver::confidence::Confidence::Low => 40,
            sih149_core::carver::confidence::Confidence::Medium => 65,
            sih149_core::carver::confidence::Confidence::High => 90,
            sih149_core::carver::confidence::Confidence::Absolute => 100,
        };

        if confidence_pct < args.min_confidence {
            continue;
        }
        if let Some(types) = &args.types {
            if !types
                .split(',')
                .any(|t| t.trim().eq_ignore_ascii_case(extension))
            {
                continue;
            }
        }

        let filename = format!("carved_{}_{:010}.{}", extension, offset, extension);

        let mut buf = vec![0u8; 1024 * 1024 * 8];
        let read_res = evidence
            .seek(SeekFrom::Start(*offset))
            .ok()
            .and_then(|_| evidence.read(&mut buf).ok());

        if let Some(n) = read_res {
            let sig = signatures.iter().find(|s| s.extension == *extension);
            let footer_bytes = sig
                .and_then(|s| s.magic_footer.as_deref())
                .and_then(|f| SignatureDatabase::parse_hex(f).ok());
            let file_len = if args.no_structure_check {
                if let Some(ref footer) = footer_bytes {
                    if !footer.is_empty() {
                        buf[..n]
                            .windows(footer.len())
                            .position(|w| w == footer.as_slice())
                            .map(|pos| pos + footer.len())
                            .unwrap_or(n)
                    } else {
                        n
                    }
                } else {
                    n
                }
            } else {
                sih149_core::carver::structure::trim_carved_file_boundary(
                    extension,
                    &buf[..n],
                    footer_bytes.as_deref(),
                )
            };

            let preview_hex = buf[..file_len.min(16)]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            candidates.push(CarveCandidate {
                index: cand_index,
                filename,
                file_type: extension.clone(),
                offset_bytes: *offset,
                size_bytes: file_len as u64,
                confidence_pct,
                category: extension.to_uppercase(),
                metadata_snippet: Some(format!("Header: [{}]", preview_hex)),
                resident_payload: None,
            });
            cand_index += 1;
        }
    }

    display::print_section("Discovered Forensic Artifacts");

    println!("\x1b[38;2;60;66;96m┌─────┬──────────────────────────────┬──────────┬─────────────┬─────────────────┬────────────┐\x1b[0m");
    println!(
        "\x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<3}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<28}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<8}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<11}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<15}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1;38;2;56;189;248m{:<10}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m",
        "#", "FILENAME", "TYPE", "SIZE", "BYTE OFFSET", "CONFIDENCE"
    );
    println!("\x1b[38;2;60;66;96m├─────┼──────────────────────────────┼──────────┼─────────────┼─────────────────┼────────────┤\x1b[0m");

    for c in &candidates {
        let conf_badge = if c.confidence_pct >= 85 {
            format!("\x1b[1;38;2;52;211;153m{:>3}%\x1b[0m", c.confidence_pct)
        } else if c.confidence_pct >= 60 {
            format!("\x1b[1;38;2;56;189;248m{:>3}%\x1b[0m", c.confidence_pct)
        } else {
            format!("\x1b[1;38;2;251;191;36m{:>3}%\x1b[0m", c.confidence_pct)
        };

        let size_str = if c.size_bytes < 1024 {
            format!("{} B", c.size_bytes)
        } else if c.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", c.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.2} MB", c.size_bytes as f64 / (1024.0 * 1024.0))
        };

        println!(
            "\x1b[38;2;60;66;96m│\x1b[0m \x1b[38;2;148;163;184m{:<3}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[1m{:<28}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m \x1b[38;2;168;85;247m{:<8}\x1b[0m \x1b[38;2;60;66;96m│\x1b[0m {:<11} \x1b[38;2;60;66;96m│\x1b[0m {:<15} \x1b[38;2;60;66;96m│\x1b[0m {:<20} \x1b[38;2;60;66;96m│\x1b[0m",
            c.index, c.filename, c.file_type, size_str, c.offset_bytes, conf_badge
        );
    }
    println!("\x1b[38;2;60;66;96m└─────┴──────────────────────────────┴──────────┴─────────────┴─────────────────┴────────────┘\x1b[0m");

    let is_inspect_only =
        args.inspect || (args.output.is_none() && args.select.is_none() && !args.interactive);

    if is_inspect_only {
        println!("\n\x1b[1;36m[INSPECT PREVIEWS]\x1b[0m");
        for c in &candidates {
            if let Some(snippet) = &c.metadata_snippet {
                println!(
                    "  \x1b[38;2;56;189;248m[{}]\x1b[0m \x1b[1m{}\x1b[0m (offset 0x{:X}): {}",
                    c.index, c.filename, c.offset_bytes, snippet
                );
            }
        }
        println!("\n\x1b[1;32m✓ Inspection Mode Complete:\x1b[0m {} artifact(s) discovered in memory. No files written to disk.", candidates.len());
        println!(
            "  To extract all files:          sih149 recover -s {} -o ./recovered",
            args.source
        );
        println!(
            "  To extract specific items:     sih149 recover -s {} -o ./recovered --select 0,2",
            args.source
        );
        println!(
            "  Interactive selection prompt:  sih149 recover -s {} -o ./recovered -i\n",
            args.source
        );
        return Ok(());
    }

    // Determine which candidates to extract
    let selected_indices: Vec<usize> = if args.interactive {
        print!("\nEnter comma-separated indices (e.g. '0,2') or 'all' to recover [default: all]: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
            (0..candidates.len()).collect()
        } else {
            trimmed
                .split(',')
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .filter(|&idx| idx < candidates.len())
                .collect()
        }
    } else if let Some(ref select_str) = args.select {
        let mut idxs = Vec::new();
        for token in select_str.split(',') {
            let t = token.trim();
            if let Ok(num) = t.parse::<usize>() {
                if num < candidates.len() {
                    idxs.push(num);
                    continue;
                }
            }
            // Match by filename or extension
            for c in &candidates {
                if c.filename.eq_ignore_ascii_case(t) || c.filename.contains(t) {
                    idxs.push(c.index);
                }
            }
        }
        idxs.sort_unstable();
        idxs.dedup();
        idxs
    } else {
        (0..candidates.len()).collect()
    };

    if selected_indices.is_empty() {
        println!("\x1b[1;33m[NOTICE]\x1b[0m No files were selected for extraction.");
        return Ok(());
    }

    let out_dir_str = args
        .output
        .clone()
        .unwrap_or_else(|| "./recovered".to_string());
    let out_dir = Path::new(&out_dir_str);
    fs::create_dir_all(out_dir)?;

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
        let _ = d.create_case(&case_id, "CLI Operator", &args.source, "recovery");
    }

    display::print_section(&format!(
        "Extracting Selected Files to {}",
        out_dir.display()
    ));

    let mut extracted_count = 0usize;
    for &idx in &selected_indices {
        let c = &candidates[idx];
        let out_file_path = out_dir.join(&c.filename);

        let mut file_sha = None;
        let mut write_ok = false;

        if let Some(ref payload) = c.resident_payload {
            if fs::write(&out_file_path, payload).is_ok() {
                write_ok = true;
                file_sha = Some(sih149_core::audit::signing::hash_bytes(payload));
            }
        } else {
            // Read from disk source
            let mut buf = vec![0u8; c.size_bytes as usize];
            if evidence.seek(SeekFrom::Start(c.offset_bytes)).is_ok()
                && evidence.read_exact(&mut buf).is_ok()
            {
                if let Ok(mut f) = fs::File::create(&out_file_path) {
                    if f.write_all(&buf).is_ok() && f.sync_all().is_ok() {
                        write_ok = true;
                        file_sha = Some(sih149_core::audit::signing::hash_bytes(&buf));
                    }
                }
            }
        }

        if write_ok {
            extracted_count += 1;
            println!(
                "  \x1b[32m✓\x1b[0m [{}] Extracted \x1b[1m{}\x1b[0m ({} B) -> SHA-256: \x1b[38;2;56;189;248m{}\x1b[0m",
                c.index,
                c.filename,
                c.size_bytes,
                file_sha.as_deref().unwrap_or("N/A")
            );

            if let Some(d) = db.as_ref() {
                let rec = sih149_core::db::CarvedFileRecord {
                    filename: c.filename.clone(),
                    file_type: c.file_type.clone(),
                    size_bytes: c.size_bytes,
                    confidence: c.confidence_pct,
                    offset_bytes: c.offset_bytes,
                    output_path: out_file_path.to_string_lossy().into_owned(),
                    category: c.category.clone(),
                    sha256: file_sha,
                };
                let _ = d.insert_carved_file(&case_id, &rec);
            }
        } else {
            eprintln!(
                "  \x1b[31m✗\x1b[0m [{}] Failed to write {}",
                c.index, c.filename
            );
        }
    }

    if let Some(d) = db.as_ref() {
        let _ = d.update_case_status(&case_id, "completed");
        let audit_payload = serde_json::json!({
            "event": "recovery_completed",
            "source": args.source,
            "output_dir": out_dir_str,
            "extracted_count": extracted_count,
            "selected_count": selected_indices.len(),
            "filesystem_hits": fs_hits.len(),
            "structure_check_enabled": !args.no_structure_check,
        });
        let json_str = audit_payload.to_string();
        let hash = sih149_core::audit::signing::hash_bytes(json_str.as_bytes());
        let _ = d.insert_audit_entry(&case_id, &json_str, &hash);
    }

    println!(
        "\n{} carved file(s) extracted and saved to: {}",
        extracted_count,
        out_dir.display()
    );
    display::print_success("Recovery operation complete.");

    crate::commands::report::generate_case_certificate_pdf(&case_id, "recovery", None);

    Ok(())
}
