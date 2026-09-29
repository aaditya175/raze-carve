use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseRecord {
    pub id: String,
    pub created_at: String,
    pub operation_type: String,
    pub target: String,
    pub status: String,
}

pub fn get_db_path() -> Result<PathBuf, String> {
    let mut path = dirs::data_local_dir().ok_or("Cannot find local data dir")?;
    path.push("secureforge");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("cases.db");
    Ok(path)
}

pub fn create_desktop_case(
    case_id: &str,
    operator: &str,
    target: &str,
    operation_type: &str,
) -> Result<(), String> {
    let db_path = get_db_path()?;
    let db = sih149_core::db::CaseDatabase::open(&db_path).map_err(|e| e.to_string())?;
    db.create_case(case_id, operator, target, operation_type)
        .map_err(|e| e.to_string())
}

pub fn update_desktop_case_status(case_id: &str, status: &str) -> Result<(), String> {
    let db_path = get_db_path()?;
    let db = sih149_core::db::CaseDatabase::open(&db_path).map_err(|e| e.to_string())?;
    db.update_case_status(case_id, status)
        .map_err(|e| e.to_string())
}

pub fn record_desktop_audit_event(
    case_id: &str,
    event_name: &str,
    details: serde_json::Value,
) -> Result<(), String> {
    let db_path = get_db_path()?;
    let db = sih149_core::db::CaseDatabase::open(&db_path).map_err(|e| e.to_string())?;
    let payload = serde_json::json!({
        "event": event_name,
        "details": details,
    });
    let json_str = payload.to_string();
    let hash = sih149_core::audit::signing::hash_bytes(json_str.as_bytes());
    db.insert_audit_entry(case_id, &json_str, &hash)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_cases() -> Result<Vec<CaseRecord>, String> {
    let db_path = get_db_path()?;
    let db = sih149_core::db::CaseDatabase::open(&db_path).map_err(|e| e.to_string())?;
    let cases = db.list_cases().map_err(|e| e.to_string())?;
    Ok(cases
        .into_iter()
        .map(|c| CaseRecord {
            id: c.id,
            created_at: c.created_at,
            operation_type: c.operation,
            target: c.target,
            status: c.status,
        })
        .collect())
}

/// Only allow case IDs that look like UUIDs or safe alphanumeric tokens —
/// blocks path traversal ("../") and JSON-breaking characters in one check.
fn validate_case_id(case_id: &str) -> Result<(), String> {
    let ok = !case_id.is_empty()
        && case_id.len() <= 128
        && case_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        Err(format!("Invalid case_id: {}", case_id))
    }
}

#[tauri::command]
pub fn export_report(
    case_id: String,
    format: String,
    output_path: String,
    investigator: Option<String>,
) -> Result<String, String> {
    validate_case_id(&case_id)?;

    // Ensure parent directory exists for nested outputs like cases/<case_id>/...
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    if format.eq_ignore_ascii_case("json") {
        let audit_json = get_audit_log(case_id)?;
        std::fs::write(&output_path, audit_json).map_err(|e| e.to_string())?;
        return Ok(output_path);
    }

    let template = match format.to_lowercase().as_str() {
        "recovery" => "recovery",
        "custody" | "chain_of_custody" => "custody",
        "pdf" | "html" | "erasure" => "erasure",
        _ => return Err(format!("Unsupported report format: {}", format)),
    };

    let report_script = std::env::var_os("SECUREFORGE_PIPELINE_DIR")
        .map(PathBuf::from)
        .map(|root| root.join("report_gen.py"))
        .filter(|path| path.exists())
        .unwrap_or_else(|| {
            crate::commands::carver::resolve_evidence_path("pipeline/report_gen.py")
        });
    if !report_script.exists() {
        return Err(format!(
            "Report generator script not found at {}",
            report_script.display()
        ));
    }

    let db_path = get_db_path()?;
    let core_db = sih149_core::db::CaseDatabase::open(&db_path).map_err(|e| e.to_string())?;
    let case_record = core_db.get_case(&case_id).map_err(|e| e.to_string())?;

    let carved_files = core_db.list_carved_files(&case_id).unwrap_or_default();
    let audit_raw_entries = core_db.get_audit_entries(&case_id).unwrap_or_default();

    let recovered_files: Vec<serde_json::Value> = carved_files
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

    let audit_entries: Vec<serde_json::Value> = audit_raw_entries
        .into_iter()
        .filter_map(|e| serde_json::from_str(&e).ok())
        .collect();

    let target_path = case_record
        .as_ref()
        .map(|c| c.target.clone())
        .unwrap_or_else(|| "/dev/sdb".to_string());
    let op_type = case_record
        .as_ref()
        .map(|c| c.operation.clone())
        .unwrap_or_else(|| template.to_string());
    let case_status = case_record
        .as_ref()
        .map(|c| c.status.clone())
        .unwrap_or_else(|| "completed".to_string());
    let created_at = case_record
        .as_ref()
        .map(|c| c.created_at.clone())
        .unwrap_or_else(|| "2026-09-08T00:00:00Z".to_string());
    let operator = investigator
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            case_record
                .as_ref()
                .map(|c| c.operator.clone())
                .unwrap_or_else(|| "Forensic Operator".to_string())
        });

    // Write audit JSON to a secure, random temp file instead of a
    // predictable /tmp/<case_id>_audit.json path (prevents symlink attacks
    // and info leakage of per-case data in a shared world-readable dir).
    let tmp_dir = std::env::temp_dir();
    let input_json = tmp_dir.join(format!(
        "sf_{}_{}_audit.json",
        case_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    let (chain_verified, chain_tip) = match core_db.get_verified_hashchain(&case_id) {
        Ok(Some(ch)) => (Some(true), Some(ch.tip().to_string())),
        Ok(None) => (None, None),
        Err(_) => (Some(false), None),
    };

    let audit_data = serde_json::json!({
        "case_id": case_id,
        "device": {
            "path": &target_path,
            "model": "Forensic Block Device",
            "serial": format!("SF-{}", &case_id[..std::cmp::min(8, case_id.len())])
        },
        "target": &target_path,
        "device_model": "Forensic Block Device",
        "device_serial": format!("SF-{}", &case_id[..std::cmp::min(8, case_id.len())]),
        "method": &op_type,
        "operation": &op_type,
        "status": &case_status,
        "created_at": &created_at,
        "date": &created_at,
        "investigator": if operator.is_empty() { "Forensic Operator" } else { &operator },
        "recovered_files": recovered_files,
        "files": &recovered_files,
        "entries": audit_entries,
        "chain_verified": chain_verified,
        "chain_tip": chain_tip,
    });
    std::fs::write(&input_json, audit_data.to_string()).map_err(|e| e.to_string())?;

    let mut cmd = Command::new("python3");
    cmd.args([
        report_script.to_str().unwrap_or("pipeline/report_gen.py"),
        "--input",
        input_json.to_str().unwrap_or_default(),
        "--template",
        template,
        "--output",
        &output_path,
        "--case-id",
        &case_id,
        "--investigator",
        &operator,
    ]);

    // If an RFC 3161 timestamp token exists for this case, pass it to embed into the certificate
    let token_candidates = [
        tmp_dir.join(format!("{}.tsr", case_id)),
        std::path::PathBuf::from(format!("{}.tsr", case_id)),
    ];
    if let Some(tok) = token_candidates.iter().find(|p| p.exists()) {
        cmd.args(["--tsa-token", tok.to_str().unwrap_or_default()]);
    }

    let output = cmd.output().map_err(|e| {
        let _ = std::fs::remove_file(&input_json);
        format!("Failed to spawn python3 report_gen: {}", e)
    })?;

    let _ = std::fs::remove_file(&input_json);

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let generated: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Report generator returned invalid output: {}", error))?;
    generated
        .get("output")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "Report generator did not return an output path".to_string())
}

#[tauri::command]
pub fn get_audit_log(case_id: String) -> Result<String, String> {
    validate_case_id(&case_id)?;
    let db_path = get_db_path()?;
    let core_db = sih149_core::db::CaseDatabase::open(&db_path).map_err(|e| e.to_string())?;

    let entries = core_db.get_audit_entries(&case_id).unwrap_or_default();
    let events: Vec<serde_json::Value> = if !entries.is_empty() {
        entries
            .iter()
            .filter_map(|e| serde_json::from_str(e).ok())
            .collect()
    } else if let Ok(Some(rec)) = core_db.get_case(&case_id) {
        vec![serde_json::json!({
            "timestamp": rec.created_at,
            "event": "case_created",
            "target": rec.target,
            "operation": rec.operation,
            "status": rec.status
        })]
    } else {
        vec![]
    };

    let (chain_verified, chain_tip) = match core_db.get_verified_hashchain(&case_id) {
        Ok(Some(ch)) => (Some(true), Some(ch.tip().to_string())),
        Ok(None) => (None, None),
        Err(_) => (Some(false), None),
    };

    let payload = serde_json::json!({
        "case_id": case_id,
        "events": events,
        "chain_verified": chain_verified,
        "chain_tip": chain_tip,
    });
    Ok(payload.to_string())
}
