//! Integration tests for the Python pipeline workers
use std::path::PathBuf;
use std::process::Command;

pub fn python3_available() -> bool {
    Command::new("python3").arg("--version").output().is_ok()
}

pub fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_report_gen_help() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/report_gen.py");
    if !script.exists() {
        return;
    }
    let status = Command::new("python3")
        .arg(&script)
        .arg("--help")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_classify_help() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/classify.py");
    if !script.exists() {
        return;
    }
    let status = Command::new("python3")
        .arg(&script)
        .arg("--help")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_timestamp_help() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/timestamp.py");
    if !script.exists() {
        return;
    }
    let status = Command::new("python3")
        .arg(&script)
        .arg("--help")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_timestamp_verify_rejection_and_validation() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/timestamp.py");
    if !script.exists() {
        return;
    }

    // 1. Verifying without token or hash must exit with non-zero
    let status_missing_args = Command::new("python3")
        .arg(&script)
        .arg("--verify")
        .status()
        .unwrap();
    assert!(!status_missing_args.success());

    // 2. Verifying a non-existent token must exit with non-zero
    let status_nonexistent = Command::new("python3")
        .arg(&script)
        .arg("--verify")
        .arg("--token")
        .arg("/tmp/nonexistent_tsr_token.tsr")
        .arg("--hash")
        .arg("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        .status()
        .unwrap();
    assert!(!status_nonexistent.success());

    // 3. Verifying with invalid hex hash must exit with non-zero
    let temp_empty_token = std::env::temp_dir().join(format!("empty_{}.tsr", std::process::id()));
    std::fs::write(&temp_empty_token, b"").unwrap();
    let status_invalid_hex = Command::new("python3")
        .arg(&script)
        .arg("--verify")
        .arg("--token")
        .arg(&temp_empty_token)
        .arg("--hash")
        .arg("invalid_hex_string")
        .status()
        .unwrap();
    assert!(!status_invalid_hex.success());

    // 4. Verifying an empty token must exit with non-zero
    let status_empty = Command::new("python3")
        .arg(&script)
        .arg("--verify")
        .arg("--token")
        .arg(&temp_empty_token)
        .arg("--hash")
        .arg("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        .status()
        .unwrap();
    assert!(!status_empty.success());
    let _ = std::fs::remove_file(&temp_empty_token);
}

#[test]
fn test_classify_empty_dir() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/classify.py");
    if !script.exists() {
        return;
    }

    let temp_dir = std::env::temp_dir().join(format!("secforge_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let output = Command::new("python3")
        .arg(&script)
        .arg("--scan-dir")
        .arg(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_classify_with_file() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/classify.py");
    if !script.exists() {
        return;
    }

    let temp_dir = std::env::temp_dir().join(format!("secforge_test_file_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let file_path = temp_dir.join("test.txt");
    std::fs::write(&file_path, "Hello world").unwrap();

    let output = Command::new("python3")
        .arg(&script)
        .arg("--scan-dir")
        .arg(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        assert!(stdout.contains("path") || stdout.contains("test.txt"));
    }
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_report_gen_missing_input() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/report_gen.py");
    if !script.exists() {
        return;
    }

    let output = Command::new("python3").arg(&script).output().unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_report_gen_chain_of_custody_and_erasure() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/report_gen.py");
    if !script.exists() {
        return;
    }

    let temp_dir = std::env::temp_dir().join(format!("secforge_rpt_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let audit_json = temp_dir.join("audit.json");
    let test_data = serde_json::json!({
        "case_id": "CASE-CUSTODY-001",
        "investigator": "Forensic Specialist",
        "target": "/dev/sdc",
        "method": "DoD 5220.22-M",
        "passes": 3,
        "entries": [
            {
                "timestamp": "2026-12-20T10:00:00Z",
                "released_by": "Officer A",
                "received_by": "Forensic Lab",
                "purpose": "Evidence intake",
                "hash": "a1b2c3d4e5f6"
            }
        ]
    });
    std::fs::write(&audit_json, test_data.to_string()).unwrap();

    let custody_out = temp_dir.join("custody.pdf");
    let out_custody = Command::new("python3")
        .arg(&script)
        .arg("--input")
        .arg(&audit_json)
        .arg("--template")
        .arg("custody")
        .arg("--output")
        .arg(&custody_out)
        .output()
        .unwrap();

    assert!(
        out_custody.status.success(),
        "report_gen custody template must succeed: {}",
        String::from_utf8_lossy(&out_custody.stderr)
    );

    let erasure_out = temp_dir.join("erasure.pdf");
    let out_erasure = Command::new("python3")
        .arg(&script)
        .arg("--input")
        .arg(&audit_json)
        .arg("--template")
        .arg("erasure")
        .arg("--output")
        .arg(&erasure_out)
        .output()
        .unwrap();

    assert!(
        out_erasure.status.success(),
        "report_gen erasure template must succeed: {}",
        String::from_utf8_lossy(&out_erasure.stderr)
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_report_gen_recovery_template_with_recovered_files() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/report_gen.py");
    if !script.exists() {
        return;
    }

    let temp_dir = std::env::temp_dir().join(format!("secforge_rec_rpt_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let audit_json = temp_dir.join("recovery_audit.json");
    let test_data = serde_json::json!({
        "case_id": "CASE-REC-001",
        "investigator": "Carver Operator",
        "device": "/dev/sdb",
        "target": "/dev/sdb",
        "recovered_files": [
            {
                "filename": "carved_001.jpg",
                "size": 1048576,
                "category": "image",
                "hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            },
            {
                "filename": "carved_002.pdf",
                "size": 524288,
                "category": "document",
                "hash": "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"
            }
        ]
    });
    std::fs::write(&audit_json, test_data.to_string()).unwrap();

    let recovery_out = temp_dir.join("recovery.pdf");
    let out_recovery = Command::new("python3")
        .arg(&script)
        .arg("--input")
        .arg(&audit_json)
        .arg("--template")
        .arg("recovery")
        .arg("--output")
        .arg(&recovery_out)
        .output()
        .unwrap();

    assert!(
        out_recovery.status.success(),
        "report_gen recovery template must succeed: {}",
        String::from_utf8_lossy(&out_recovery.stderr)
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_classify_magic_byte_sniffing() {
    if !python3_available() {
        return;
    }
    let root = get_workspace_root();
    let script = root.join("pipeline/classify.py");
    if !script.exists() {
        return;
    }

    let temp_dir = std::env::temp_dir().join(format!("secforge_sniff_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    // Write a PDF with misleading .bin extension
    let pdf_file = temp_dir.join("document.bin");
    std::fs::write(&pdf_file, b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\n%%EOF\n").unwrap();

    // Write a PNG with misleading .dat extension
    let png_file = temp_dir.join("image.dat");
    std::fs::write(
        &png_file,
        b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00",
    )
    .unwrap();

    let output = Command::new("python3")
        .arg(&script)
        .arg("--scan-dir")
        .arg(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("application/pdf"));
    assert!(stdout.contains("image/png"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
