use crate::audit::schema::{AuditEntry, AuditLog};
use crate::error::CoreError;
use sha2::{Digest, Sha256};

pub struct HashChain {
    entries: Vec<AuditEntry>,
    tip: String, // hash of last entry
    seq: u64,    // monotonically increasing entry counter
}

impl Default for HashChain {
    fn default() -> Self {
        Self::new()
    }
}

impl HashChain {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            // SHA-256 hex digest is exactly 64 chars
            tip: "0".repeat(64),
            seq: 0,
        }
    }

    pub fn append(&mut self, mut entry: AuditEntry) -> Result<String, CoreError> {
        entry.seq = self.seq;
        let mut entry_clone = entry.clone();
        entry_clone.prev_hash = String::new();
        entry_clone.entry_hash = String::new();
        entry_clone.seq = self.seq;

        let entry_json =
            serde_json::to_string(&entry_clone).map_err(|e| CoreError::Parse(e.to_string()))?;

        let prev_hash = self.tip.clone();

        let mut hasher = Sha256::new();
        hasher.update(entry_json.as_bytes());
        hasher.update(prev_hash.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        entry.prev_hash = prev_hash;
        entry.entry_hash = hash.clone();

        self.entries.push(entry);
        self.tip = hash.clone();
        self.seq += 1;

        Ok(hash)
    }

    pub fn verify(&self) -> bool {
        let mut expected_prev = "0".repeat(64);
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.prev_hash != expected_prev {
                return false;
            }
            // Sequence numbers must be strictly increasing from 0.
            if entry.seq != index as u64 {
                return false;
            }

            let mut entry_clone = entry.clone();
            entry_clone.prev_hash = String::new();
            entry_clone.entry_hash = String::new();
            entry_clone.seq = entry.seq;

            let entry_json = match serde_json::to_string(&entry_clone) {
                Ok(s) => s,
                Err(_) => return false,
            };

            let mut hasher = Sha256::new();
            hasher.update(entry_json.as_bytes());
            hasher.update(entry.prev_hash.as_bytes());
            let computed = format!("{:x}", hasher.finalize());

            if computed != entry.entry_hash {
                return false;
            }
            expected_prev = computed;
        }
        true
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn tip(&self) -> &str {
        &self.tip
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn to_audit_log(&self, case_id: String, operator: Option<String>) -> AuditLog {
        AuditLog {
            case_id,
            created_at: chrono::Utc::now().to_rfc3339(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            operator,
            entries: self.entries.clone(),
            chain_tip: self.tip.clone(),
        }
    }

    /// Reconstruct and cryptographically verify a HashChain from existing AuditEntries.
    pub fn from_entries(entries: Vec<AuditEntry>) -> Result<Self, CoreError> {
        let mut expected_prev = "0".repeat(64);
        let mut tip = "0".repeat(64);
        for (index, entry) in entries.iter().enumerate() {
            if entry.prev_hash != expected_prev {
                return Err(CoreError::InvalidStructure(format!(
                    "Audit entry at index {} has invalid prev_hash: expected {}, got {}",
                    index, expected_prev, entry.prev_hash
                )));
            }
            if entry.seq != index as u64 {
                return Err(CoreError::InvalidStructure(format!(
                    "Audit entry at index {} has invalid sequence number: expected {}, got {}",
                    index, index, entry.seq
                )));
            }

            let mut entry_clone = entry.clone();
            entry_clone.prev_hash = String::new();
            entry_clone.entry_hash = String::new();
            entry_clone.seq = entry.seq;

            let entry_json =
                serde_json::to_string(&entry_clone).map_err(|e| CoreError::Parse(e.to_string()))?;

            let mut hasher = Sha256::new();
            hasher.update(entry_json.as_bytes());
            hasher.update(entry.prev_hash.as_bytes());
            let computed = format!("{:x}", hasher.finalize());

            if computed != entry.entry_hash {
                return Err(CoreError::InvalidStructure(format!(
                    "Audit entry at index {} has tampered entry_hash: computed {}, stored {}",
                    index, computed, entry.entry_hash
                )));
            }
            expected_prev = computed.clone();
            tip = computed;
        }

        let seq = entries.len() as u64;
        Ok(Self { entries, tip, seq })
    }

    /// Reconstruct and cryptographically verify a HashChain from an exported AuditLog,
    /// ensuring the chain tip matches the manifest.
    pub fn from_log(log: &AuditLog) -> Result<Self, CoreError> {
        let chain = Self::from_entries(log.entries.clone())?;
        if chain.tip() != log.chain_tip {
            return Err(CoreError::InvalidStructure(format!(
                "Audit log chain_tip mismatch: computed {}, manifest {}",
                chain.tip(),
                log.chain_tip
            )));
        }
        Ok(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::schema::{OperationResult, OperationType};
    use std::collections::BTreeMap;

    fn make_test_entry(id: u64, target: &str) -> AuditEntry {
        AuditEntry {
            id,
            seq: 0,
            timestamp: "2026-12-22T12:00:00Z".to_string(),
            operation: OperationType::DiskWipe,
            target: target.to_string(),
            params: BTreeMap::new(),
            result: OperationResult {
                success: true,
                message: "Test pass completed".to_string(),
                pre_hash: None,
                post_hash: None,
                sectors_processed: Some(1024),
                bad_sectors: None,
                files_recovered: None,
                entropy_post: Some(0.0),
            },
            prev_hash: String::new(),
            entry_hash: String::new(),
        }
    }

    #[test]
    fn test_hash_chain_from_entries_roundtrip() {
        let mut chain = HashChain::new();
        chain.append(make_test_entry(1, "/dev/sdb")).unwrap();
        chain.append(make_test_entry(2, "/dev/sdc")).unwrap();

        let entries = chain.entries().to_vec();
        let reconstructed = HashChain::from_entries(entries).unwrap();
        assert_eq!(reconstructed.len(), 2);
        assert_eq!(reconstructed.tip(), chain.tip());
        assert!(reconstructed.verify());
    }

    #[test]
    fn test_hash_chain_from_log_tamper_detection() {
        let mut chain = HashChain::new();
        chain.append(make_test_entry(1, "/dev/sdb")).unwrap();
        chain.append(make_test_entry(2, "/dev/sdc")).unwrap();

        let log = chain.to_audit_log("CASE-TEST".to_string(), Some("Operator".to_string()));
        let valid_chain = HashChain::from_log(&log).unwrap();
        assert_eq!(valid_chain.tip(), log.chain_tip);

        // Tamper with target in first entry
        let mut tampered_log = log.clone();
        tampered_log.entries[0].target = "/dev/tampered".to_string();
        let err = HashChain::from_log(&tampered_log);
        assert!(err.is_err());

        // Tamper with chain tip manifest
        let mut tampered_tip_log = log;
        tampered_tip_log.chain_tip = "0".repeat(64);
        let err_tip = HashChain::from_log(&tampered_tip_log);
        assert!(err_tip.is_err());
    }
}
