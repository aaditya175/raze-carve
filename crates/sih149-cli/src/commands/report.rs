use crate::display;
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args)]
pub struct ReportArgs {
    #[arg(long)]
    pub list: bool,
    #[arg(long)]
    pub case_id: Option<String>,
    #[arg(long)]
    pub export: Option<String>,
    #[arg(long, default_value = "pdf")]
    pub format: String,
    #[arg(short, long)]
    pub output: Option<String>,
    #[arg(long)]
    pub tsa_token: Option<String>,
}

fn get_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_default()
        .join("secureforge")
        .join("cases.db")
}

fn resolve_report_script() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SECUREFORGE_PIPELINE_DIR") {
        let p = PathBuf::from(dir).join("report_gen.py");
        if p.is_file() {
            return Some(p);
        }
    }
    let candidates = [
        PathBuf::from("pipeline/report_gen.py"),
        PathBuf::from("../pipeline/report_gen.py"),
        PathBuf::from("../../pipeline/report_gen.py"),
        PathBuf::from("/usr/local/share/secureforge/pipeline/report_gen.py"),
    ];
    for c in &candidates {
        if c.is_file() {
            return Some(c.clone());
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p1 = parent.join("pipeline/report_gen.py");
            if p1.is_file() {
                return Some(p1);
            }
            if let Some(gp) = parent.parent() {
                let p2 = gp.join("pipeline/report_gen.py");
                if p2.is_file() {
                    return Some(p2);
                }
            }
        }
    }
    None
}

/// Automatically generate forensic certificate PDF into cases/<case_id>/<case_id>_certificate.pdf
/// or output_override if specified.
pub fn generate_case_certificate_pdf(
    case_id: &str,
    template: &str,
    output_override: Option<&str>,
) -> Option<PathBuf> {
    let script = resolve_report_script()?;
    let db_path = get_db_path();
    let db = sih149_core::db::CaseDatabase::open(&db_path).ok()?;
    let case_record = db.get_case(case_id).ok()??;
    let carved_files = db.list_carved_files(case_id).unwrap_or_default();
    let raw_entries = db.get_audit_entries(case_id).unwrap_or_default();

    let rec_files: Vec<serde_json::Value> = carved_files
        .into_iter()
        .map(|f| {
            serde_json::json!({
                "filename": f.filename,
                "size": f.size_bytes,
                "size_bytes": f.size_bytes,
                "category": f.category,
                "hash": f.sha256,
                "sha256": f.sha256,
            })
        })
        .collect();

    let audit_entries: Vec<serde_json::Value> = raw_entries
        .into_iter()
        .filter_map(|e| serde_json::from_str(&e).ok())
        .collect();

    let target = if case_record.target.is_empty() {
        "Forensic Target"
    } else {
        &case_record.target
    };
    let op = if case_record.operation.is_empty() {
        template
    } else {
        &case_record.operation
    };
    let status = if case_record.status.is_empty() {
        "completed"
    } else {
        &case_record.status
    };
    let created_at = &case_record.created_at;
    let operator = if case_record.operator.is_empty() {
        "CLI Operator"
    } else {
        &case_record.operator
    };

    let tmp = std::env::temp_dir().join(format!("sf_cli_auto_rpt_{}.json", case_id));
    let payload = serde_json::json!({
        "case_id": case_id,
        "device": {
            "path": target,
            "model": "Forensic Block Device",
            "serial": format!("SF-{}", &case_id[..std::cmp::min(8, case_id.len())])
        },
        "target": target,
        "device_model": "Forensic Block Device",
        "device_serial": format!("SF-{}", &case_id[..std::cmp::min(8, case_id.len())]),
        "method": op,
        "operation": op,
        "status": status,
        "created_at": created_at,
        "date": created_at,
        "investigator": operator,
        "recovered_files": rec_files,
        "files": rec_files,
        "entries": audit_entries,
    });
    if std::fs::write(&tmp, payload.to_string()).is_err() {
        return None;
    }

    let output_file: PathBuf = if let Some(out) = output_override {
        PathBuf::from(out)
    } else {
        let dir = PathBuf::from("cases").join(case_id);
        let _ = std::fs::create_dir_all(&dir);
        dir.join(format!("{}_certificate.pdf", case_id))
    };

    if let Some(parent) = output_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut cmd = Command::new("python3");
    cmd.args([
        script.to_str().unwrap_or("report_gen.py"),
        "--input",
        tmp.to_str().unwrap_or(""),
        "--template",
        template,
        "--output",
        output_file.to_str().unwrap_or(""),
        "--case-id",
        case_id,
    ]);

    let res = cmd.status();
    let _ = std::fs::remove_file(&tmp);

    match res {
        Ok(s) if s.success() => {
            let canonical = output_file.canonicalize().unwrap_or_else(|_| {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(&output_file)
            });
            display::print_success(&format!(
                "Forensic certificate PDF generated: {}",
                canonical.display()
            ));
            Some(canonical)
        }
        _ => None,
    }
}

pub fn run(args: &ReportArgs) -> anyhow::Result<()> {
    let db_path = get_db_path();

    if args.list {
        display::print_section("Forensic Cases");
        if !db_path.exists() {
            display::print_info("No cases database found at ~/.local/share/secureforge/cases.db");
            return Ok(());
        }
        let db = sih149_core::db::CaseDatabase::open(&db_path)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let cases = db
            .list_cases()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        if cases.is_empty() {
            display::print_info("No cases recorded in database.");
        } else {
            display::print_case_table(&cases);
        }
        return Ok(());
    }

    if let Some(case_id) = &args.case_id {
        if args.export.is_none() && args.output.is_none() {
            display::print_section(&format!("Case Details: {}", case_id));
            if !db_path.exists() {
                display::print_error("No cases database found.");
                return Err(anyhow::anyhow!("No cases database found."));
            }
            let db = sih149_core::db::CaseDatabase::open(&db_path)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            match db
                .get_case(case_id)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?
            {
                Some(c) => {
                    println!("Case ID:       {}", c.id);
                    println!("Created:       {}", c.created_at);
                    println!("Operation:     {}", c.operation);
                    println!("Target:        {}", c.target);
                    println!(
                        "Operator:      {}",
                        if c.operator.is_empty() {
                            "N/A"
                        } else {
                            &c.operator
                        }
                    );
                    println!("Status:        {}", c.status);
                    if let Some(rp) = &c.report_path {
                        println!("Report:        {}", rp);
                    }
                    let audit_count = db.get_audit_entries(case_id).map(|a| a.len()).unwrap_or(0);
                    let carved_count = db.list_carved_files(case_id).map(|f| f.len()).unwrap_or(0);
                    println!("Audit Events:  {}", audit_count);
                    match db.get_verified_hashchain(case_id) {
                        Ok(Some(chain)) => {
                            let tip = chain.tip();
                            let tip_short = if tip.len() > 16 { &tip[..16] } else { tip };
                            println!(
                                "Audit Chain:   \x1b[1;38;2;52;211;153mVERIFIED\x1b[0m ({} links, tip: {}...)",
                                chain.len(),
                                tip_short
                            );
                        }
                        Ok(None) => {
                            if audit_count > 0 {
                                println!(
                                    "Audit Chain:   Unchained events ({} entries)",
                                    audit_count
                                );
                            } else {
                                println!("Audit Chain:   Empty");
                            }
                        }
                        Err(e) => {
                            println!("Audit Chain:   \x1b[1;38;2;248;113;113mINTEGRITY FAILED\x1b[0m ({})", e);
                        }
                    }
                    println!("Carved Files:  {}", carved_count);
                    return Ok(());
                }
                None => {
                    display::print_error(&format!("Case '{}' not found in database.", case_id));
                    return Err(anyhow::anyhow!("Case '{}' not found in database.", case_id));
                }
            }
        }
    }

    if args.export.is_some() || args.output.is_some() {
        display::print_section("Exporting Report");

        let output_file = args.output.clone().unwrap_or_else(|| {
            if args.format.eq_ignore_ascii_case("html") {
                "report_out.html".to_string()
            } else if args.format.eq_ignore_ascii_case("json") {
                "report_out.json".to_string()
            } else {
                "report_out.pdf".to_string()
            }
        });

        if args.format.eq_ignore_ascii_case("json") {
            let json_content = if let Some(case_id) = &args.case_id {
                let db = sih149_core::db::CaseDatabase::open(&db_path)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let c = db
                    .get_case(case_id)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?
                    .ok_or_else(|| anyhow::anyhow!("Case not found in database: {}", case_id))?;
                let files = db.list_carved_files(case_id).unwrap_or_default();
                let raw_entries = db.get_audit_entries(case_id).unwrap_or_default();
                let events: Vec<serde_json::Value> = raw_entries
                    .into_iter()
                    .filter_map(|e| serde_json::from_str(&e).ok())
                    .collect();
                let (chain_verified, chain_tip) = match db.get_verified_hashchain(case_id) {
                    Ok(Some(ch)) => (Some(true), Some(ch.tip().to_string())),
                    Ok(None) => (None, None),
                    Err(_) => (Some(false), None),
                };
                serde_json::to_string_pretty(&serde_json::json!({
                    "case": c,
                    "carved_files": files,
                    "audit_events": events,
                    "chain_verified": chain_verified,
                    "chain_tip": chain_tip,
                }))?
            } else if let Some(export_path) = &args.export {
                std::fs::read_to_string(export_path)?
            } else {
                anyhow::bail!("Must specify --case-id or --export to produce JSON audit report");
            };

            std::fs::write(&output_file, json_content)?;
            display::print_success(&format!("Audit report exported to {}", output_file));
            return Ok(());
        }

        let script = resolve_report_script().ok_or_else(|| {
            anyhow::anyhow!(
                "Report generator script not found (pipeline/report_gen.py). Ensure repository root exists."
            )
        })?;

        let template = match args.format.to_lowercase().as_str() {
            "recovery" => "recovery",
            "custody" | "chain_of_custody" => "custody",
            _ => "erasure",
        };

        let (input_json_path, _temp_file) = if let Some(export_file) = &args.export {
            if std::path::Path::new(export_file).is_file() {
                (std::path::PathBuf::from(export_file), None)
            } else {
                anyhow::bail!("Export input file not found: {}", export_file);
            }
        } else if let Some(case_id) = &args.case_id {
            let db = sih149_core::db::CaseDatabase::open(&db_path)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            let case_record = db
                .get_case(case_id)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?
                .ok_or_else(|| anyhow::anyhow!("Case not found in database: {}", case_id))?;
            let carved_files = db.list_carved_files(case_id).unwrap_or_default();
            let raw_entries = db.get_audit_entries(case_id).unwrap_or_default();

            let rec_files: Vec<serde_json::Value> = carved_files
                .into_iter()
                .map(|f| {
                    serde_json::json!({
                        "filename": f.filename,
                        "size": f.size_bytes,
                        "size_bytes": f.size_bytes,
                        "category": f.category,
                        "hash": f.sha256,
                        "sha256": f.sha256,
                    })
                })
                .collect();

            let audit_entries: Vec<serde_json::Value> = raw_entries
                .into_iter()
                .filter_map(|e| serde_json::from_str(&e).ok())
                .collect();

            let target = if case_record.target.is_empty() { "/dev/sdb" } else { &case_record.target };
            let op = if case_record.operation.is_empty() { template } else { &case_record.operation };
            let status = if case_record.status.is_empty() { "completed" } else { &case_record.status };
            let created_at = &case_record.created_at;
            let operator = if case_record.operator.is_empty() { "CLI Operator" } else { &case_record.operator };

            let tmp = std::env::temp_dir().join(format!("sf_cli_rpt_{}.json", case_id));
            let payload = serde_json::json!({
                "case_id": case_id,
                "device": {
                    "path": target,
                    "model": "Forensic Block Device",
                    "serial": format!("SF-{}", &case_id[..std::cmp::min(8, case_id.len())])
                },
                "target": target,
                "device_model": "Forensic Block Device",
                "device_serial": format!("SF-{}", &case_id[..std::cmp::min(8, case_id.len())]),
                "method": op,
                "operation": op,
                "status": status,
                "created_at": created_at,
                "date": created_at,
                "investigator": if operator.is_empty() { "CLI Operator" } else { operator },
                "recovered_files": rec_files,
                "files": rec_files,
                "entries": audit_entries,
            });
            std::fs::write(&tmp, payload.to_string())?;
            (tmp.clone(), Some(tmp))
        } else {
            anyhow::bail!("Must specify either --case-id or --export <FILE> to generate a report");
        };

        let mut cmd = Command::new("python3");
        cmd.args([
            script.to_string_lossy().as_ref(),
            "--input",
            input_json_path.to_string_lossy().as_ref(),
            "--template",
            template,
            "--output",
            &output_file,
        ]);

        if let Some(cid) = &args.case_id {
            cmd.args(["--case-id", cid]);
        }

        let token_to_use = args.tsa_token.clone().or_else(|| {
            if let Some(cid) = &args.case_id {
                let candidates = [
                    std::path::PathBuf::from(format!("{}.tsr", cid)),
                    std::env::temp_dir().join(format!("{}.tsr", cid)),
                ];
                candidates
                    .into_iter()
                    .find(|p| p.exists())
                    .map(|p| p.to_string_lossy().to_string())
            } else {
                None
            }
        });

        if let Some(token) = token_to_use {
            cmd.args(["--tsa-token", &token]);
        }

        let res = cmd.status();

        if let Some(ref tmp) = _temp_file {
            let _ = std::fs::remove_file(tmp);
        }

        match res {
            Ok(s) if s.success() => {
                display::print_success(&format!(
                    "Report exported successfully to {}.",
                    output_file
                ));
            }
            _ => {
                display::print_error("Failed to export report.");
                anyhow::bail!("Report generation failed");
            }
        }
        return Ok(());
    }

    display::print_info(
        "Use --list to view cases, --case-id <ID> to inspect, or --export/--output to generate a report.",
    );
    Ok(())
}
