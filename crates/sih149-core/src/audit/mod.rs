pub mod hashchain;
pub mod schema;
pub mod signing;

use crate::audit::hashchain::HashChain;
use crate::audit::schema::*;
use crate::error::CoreError;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct AuditEngine {
    chain: Arc<Mutex<HashChain>>,
    case_id: String,
    operator: Option<String>,
    #[allow(dead_code)] // reserved for future on-disk report export
    output_dir: PathBuf,
}

impl AuditEngine {
    pub fn new(case_id: String, operator: Option<String>, output_dir: &Path) -> Self {
        Self {
            chain: Arc::new(Mutex::new(HashChain::new())),
            case_id,
            operator,
            output_dir: output_dir.to_path_buf(),
        }
    }

    pub fn record_wipe(
        &self,
        device: &DeviceInfo,
        method: &str,
        result: OperationResult,
    ) -> Result<String, CoreError> {
        let entry = AuditEntry {
            id: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).max(0) as u64,
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: OperationType::DiskWipe,
            target: device.path.clone(),
            params: {
                let mut map = BTreeMap::new();
                map.insert(
                    "method".to_string(),
                    serde_json::Value::String(method.to_string()),
                );
                map.insert(
                    "device".to_string(),
                    serde_json::to_value(device).unwrap_or(serde_json::Value::Null),
                );
                map
            },
            result,
            prev_hash: String::new(),
            entry_hash: String::new(),
            seq: 0,
        };

        let mut chain = self
            .chain
            .lock()
            .map_err(|e| CoreError::Disk(e.to_string()))?;
        chain.append(entry)
    }

    pub fn record_scan(&self, source: &str, result: OperationResult) -> Result<String, CoreError> {
        let entry = AuditEntry {
            id: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).max(0) as u64,
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: OperationType::RecoveryScan,
            target: source.to_string(),
            params: BTreeMap::new(),
            result,
            prev_hash: String::new(),
            entry_hash: String::new(),
            seq: 0,
        };

        let mut chain = self
            .chain
            .lock()
            .map_err(|e| CoreError::Disk(e.to_string()))?;
        chain.append(entry)
    }

    pub fn verify_chain(&self) -> bool {
        let Ok(chain) = self.chain.lock() else {
            return false;
        };
        chain.verify()
    }

    pub fn export_json(&self, path: &Path) -> Result<(), CoreError> {
        let chain = self
            .chain
            .lock()
            .map_err(|e| CoreError::Disk(e.to_string()))?;
        let log = chain.to_audit_log(self.case_id.clone(), self.operator.clone());
        let json =
            serde_json::to_string_pretty(&log).map_err(|e| CoreError::Parse(e.to_string()))?;
        std::fs::write(path, json).map_err(CoreError::Io)
    }

    pub fn chain_tip(&self) -> String {
        let Ok(chain) = self.chain.lock() else {
            return String::new();
        };
        chain.tip().to_string()
    }

    /// Reconstruct and verify an AuditEngine from an existing AuditLog.
    pub fn from_log(log: AuditLog, output_dir: &Path) -> Result<Self, CoreError> {
        let chain = HashChain::from_log(&log)?;
        Ok(Self {
            chain: Arc::new(Mutex::new(chain)),
            case_id: log.case_id,
            operator: log.operator,
            output_dir: output_dir.to_path_buf(),
        })
    }

    /// Reconstruct and verify an AuditEngine from an on-disk JSON audit log.
    pub fn from_json_file(path: &Path, output_dir: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path).map_err(CoreError::Io)?;
        let log: AuditLog =
            serde_json::from_str(&content).map_err(|e| CoreError::Parse(e.to_string()))?;
        Self::from_log(log, output_dir)
    }
}
