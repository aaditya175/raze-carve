use crate::error::CoreError;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Compare two ASCII hex digest strings in constant time (no early exit on
/// the first mismatching byte). This prevents a remote/IPC timing oracle
/// from recovering a digest byte-by-byte via `verify_file_hash`.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (ai, bi) in a.iter().zip(b.iter()) {
        diff |= ai ^ bi;
    }
    diff == 0
}

/// Compute SHA-256 of a file using a streaming 1MB buffer.
/// Does not load the whole file into memory, preventing OOM on multi-GB disk images.
pub fn hash_file(path: &Path) -> Result<String, CoreError> {
    let mut file = fs::File::open(path).map_err(CoreError::Io)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024]; // 1MB streaming buffer, reused
    loop {
        let n = file.read(&mut buf).map_err(CoreError::Io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute SHA-256 of arbitrary bytes  
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Verify a file's hash matches expected (constant-time comparison)
pub fn verify_file_hash(path: &Path, expected_hash: &str) -> Result<bool, CoreError> {
    let hash = hash_file(path)?;
    Ok(constant_time_eq(&hash, expected_hash))
}

fn resolve_timestamp_script() -> Option<PathBuf> {
    let script_candidates = [
        PathBuf::from("pipeline/timestamp.py"),
        PathBuf::from("../pipeline/timestamp.py"),
        PathBuf::from("../../pipeline/timestamp.py"),
    ];
    for p in &script_candidates {
        if p.is_file() {
            return Some(p.clone());
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p1 = parent.join("pipeline/timestamp.py");
            if p1.is_file() {
                return Some(p1);
            }
            if let Some(gp) = parent.parent() {
                let p2 = gp.join("pipeline/timestamp.py");
                if p2.is_file() {
                    return Some(p2);
                }
            }
        }
    }
    None
}

/// Inspects an RFC 3161 timestamp response or token buffer directly in Rust:
/// Validates that the DER buffer contains the expected 32-byte SHA-256 message imprint
/// and does not indicate a rejected PKIStatus.
pub fn verify_rfc3161_token_imprint(
    token_bytes: &[u8],
    expected_hash_hex: &str,
) -> Result<bool, CoreError> {
    let clean_hex = expected_hash_hex.trim();
    if clean_hex.len() != 64 || !clean_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CoreError::Parse(
            "Invalid SHA-256 hex format: expected 64 hexadecimal characters".to_string(),
        ));
    }
    if token_bytes.len() < 32 {
        return Ok(false);
    }
    let mut expected_bytes = [0u8; 32];
    for i in 0..32 {
        expected_bytes[i] = u8::from_str_radix(&clean_hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| CoreError::Parse("Invalid hex byte".to_string()))?;
    }

    // Search for the 32-byte message imprint in the token buffer
    let has_imprint = token_bytes
        .windows(32)
        .any(|window| window == expected_bytes);
    if !has_imprint {
        return Ok(false);
    }

    // Check if DER PKIStatus indicates rejection:
    // RFC 3161 PKIStatus: 0=granted, 1=grantedWithMods, 2=rejection
    // In DER encoding, PKIStatus is an INTEGER inside PKIStatusInfo SEQUENCE at the beginning of TimeStampResp
    // e.g. [0x30, ..., 0x02, 0x01, 0x02] (INTEGER value >= 2)
    if token_bytes[0] == 0x30 {
        let scan_limit = std::cmp::min(32, token_bytes.len() - 2);
        for i in 0..scan_limit {
            if token_bytes[i] == 0x02 && token_bytes[i + 1] == 0x01 && token_bytes[i + 2] >= 2 {
                return Ok(false); // PKIStatus: rejection / error
            }
        }
    }

    Ok(true)
}

/// Request an RFC 3161 timestamp token for a given SHA-256 digest via pipeline/timestamp.py
pub fn request_rfc3161_timestamp(
    hash_hex: &str,
    output_path: &Path,
    tsa_url: Option<&str>,
) -> Result<bool, CoreError> {
    let clean_hex = hash_hex.trim();
    if clean_hex.len() != 64 || !clean_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CoreError::Parse(
            "Invalid SHA-256 hex format: expected 64 hexadecimal characters".to_string(),
        ));
    }

    let script = resolve_timestamp_script()
        .ok_or_else(|| CoreError::Disk("pipeline/timestamp.py not found".to_string()))?;

    let mut cmd = std::process::Command::new("python3");
    cmd.arg(script)
        .arg("--hash")
        .arg(clean_hex)
        .arg("--output")
        .arg(output_path);
    if let Some(url) = tsa_url {
        cmd.arg("--tsa-url").arg(url);
    }

    let status = cmd.status().map_err(CoreError::Io)?;
    Ok(status.success())
}

/// Verify an RFC 3161 timestamp token against a SHA-256 digest via pipeline/timestamp.py
/// with native Rust DER imprint fallback.
pub fn verify_rfc3161_timestamp(token_path: &Path, hash_hex: &str) -> Result<bool, CoreError> {
    let clean_hex = hash_hex.trim();
    if clean_hex.len() != 64 || !clean_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CoreError::Parse(
            "Invalid SHA-256 hex format: expected 64 hexadecimal characters".to_string(),
        ));
    }

    if let Some(script) = resolve_timestamp_script() {
        let mut cmd = std::process::Command::new("python3");
        cmd.arg(script)
            .arg("--verify")
            .arg("--token")
            .arg(token_path)
            .arg("--hash")
            .arg(clean_hex);

        if let Ok(status) = cmd.status() {
            return Ok(status.success());
        }
    }

    // Native fallback when python3 is not available
    let token_bytes = fs::read(token_path).map_err(CoreError::Io)?;
    verify_rfc3161_token_imprint(&token_bytes, clean_hex)
}

/// Simple report integrity record saved alongside PDF
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportManifest {
    pub report_path: String,
    pub report_hash: String,
    pub generated_at: String,
    pub case_id: String,
    pub chain_tip: String,
    pub hash_verified: bool,
    #[serde(default)]
    pub tsa_token_path: Option<String>,
}

impl ReportManifest {
    pub fn new(report_path: &Path, hash: String, case_id: String, chain_tip: String) -> Self {
        Self {
            report_path: report_path.to_string_lossy().into_owned(),
            report_hash: hash,
            generated_at: chrono::Utc::now().to_rfc3339(),
            case_id,
            chain_tip,
            hash_verified: true,
            tsa_token_path: None,
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), CoreError> {
        let json =
            serde_json::to_string_pretty(self).map_err(|e| CoreError::Parse(e.to_string()))?;
        fs::write(path, json).map_err(CoreError::Io)
    }

    pub fn load(path: &Path) -> Result<Self, CoreError> {
        let json = fs::read_to_string(path).map_err(CoreError::Io)?;
        serde_json::from_str(&json).map_err(|e| CoreError::Parse(e.to_string()))
    }
}
