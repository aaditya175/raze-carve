use crate::error::CoreError;
use rusqlite::{params, Connection};
use std::collections::BTreeMap;
use std::path::Path;

pub struct CaseDatabase {
    conn: Connection,
}

impl CaseDatabase {
    pub fn open(db_path: &Path) -> Result<Self, CoreError> {
        let conn = Connection::open(db_path).map_err(|e| CoreError::Disk(e.to_string()))?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -20000;",
        )
        .map_err(|e| CoreError::Disk(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cases (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                operator TEXT DEFAULT '',
                target TEXT NOT NULL DEFAULT '',
                operation TEXT DEFAULT '',
                operation_type TEXT DEFAULT '',
                status TEXT DEFAULT 'running',
                report_path TEXT
            )",
            [],
        )
        .map_err(|e| CoreError::Disk(e.to_string()))?;

        // Automatic column migrations so databases created by Tauri or older schemas work seamlessly
        let _ = conn.execute("ALTER TABLE cases ADD COLUMN operator TEXT DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE cases ADD COLUMN operation TEXT DEFAULT ''", []);
        let _ = conn.execute(
            "ALTER TABLE cases ADD COLUMN operation_type TEXT DEFAULT ''",
            [],
        );
        let _ = conn.execute("ALTER TABLE cases ADD COLUMN report_path TEXT", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS carved_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT, case_id TEXT NOT NULL,
                filename TEXT, file_type TEXT, size_bytes INTEGER,
                confidence INTEGER, offset_bytes INTEGER, output_path TEXT,
                category TEXT, sha256 TEXT,
                FOREIGN KEY (case_id) REFERENCES cases(id)
            )",
            [],
        )
        .map_err(|e| CoreError::Disk(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT, case_id TEXT NOT NULL,
                entry_json TEXT, entry_hash TEXT, recorded_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| CoreError::Disk(e.to_string()))?;

        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_carved_files_case_id ON carved_files(case_id);
             CREATE INDEX IF NOT EXISTS idx_carved_files_case_category ON carved_files(case_id, category);
             CREATE INDEX IF NOT EXISTS idx_audit_entries_case_id ON audit_entries(case_id);",
        )
        .map_err(|e| CoreError::Disk(e.to_string()))?;

        Ok(Self { conn })
    }

    pub fn create_case(
        &self,
        case_id: &str,
        operator: &str,
        target: &str,
        operation: &str,
    ) -> Result<(), CoreError> {
        let created_at = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO cases (id, created_at, operator, target, operation, operation_type, status) VALUES (?1, ?2, ?3, ?4, ?5, ?5, 'running')",
            params![case_id, created_at, operator, target, operation],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;
        Ok(())
    }

    pub fn update_case_status(&self, case_id: &str, status: &str) -> Result<(), CoreError> {
        self.conn
            .execute(
                "UPDATE cases SET status = ?1 WHERE id = ?2",
                params![status, case_id],
            )
            .map_err(|e| CoreError::Disk(e.to_string()))?;
        Ok(())
    }

    pub fn list_cases(&self) -> Result<Vec<CaseRecord>, CoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, created_at, COALESCE(operator, ''), target, 
             COALESCE(NULLIF(operation, ''), operation_type, 'unknown'), 
             status, report_path FROM cases ORDER BY created_at DESC",
            )
            .map_err(|e| CoreError::Disk(e.to_string()))?;

        let case_iter = stmt
            .query_map([], |row| {
                Ok(CaseRecord {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    operator: row.get(2)?,
                    target: row.get(3)?,
                    operation: row.get(4)?,
                    status: row.get(5)?,
                    report_path: row.get(6)?,
                })
            })
            .map_err(|e| CoreError::Disk(e.to_string()))?;

        let mut cases = Vec::new();
        for case in case_iter {
            cases.push(case.map_err(|e| CoreError::Disk(e.to_string()))?);
        }
        Ok(cases)
    }

    pub fn get_case(&self, case_id: &str) -> Result<Option<CaseRecord>, CoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, created_at, COALESCE(operator, ''), target, 
             COALESCE(NULLIF(operation, ''), operation_type, 'unknown'), 
             status, report_path FROM cases WHERE id = ?1",
            )
            .map_err(|e| CoreError::Disk(e.to_string()))?;

        let mut case_iter = stmt
            .query_map(params![case_id], |row| {
                Ok(CaseRecord {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    operator: row.get(2)?,
                    target: row.get(3)?,
                    operation: row.get(4)?,
                    status: row.get(5)?,
                    report_path: row.get(6)?,
                })
            })
            .map_err(|e| CoreError::Disk(e.to_string()))?;

        if let Some(case) = case_iter.next() {
            return Ok(Some(case.map_err(|e| CoreError::Disk(e.to_string()))?));
        }
        Ok(None)
    }

    pub fn insert_carved_file(
        &self,
        case_id: &str,
        file: &CarvedFileRecord,
    ) -> Result<i64, CoreError> {
        self.conn.execute(
            "INSERT INTO carved_files (case_id, filename, file_type, size_bytes, confidence, offset_bytes, output_path, category, sha256) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![case_id, file.filename, file.file_type, file.size_bytes, file.confidence, file.offset_bytes, file.output_path, file.category, file.sha256],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_carved_files(&self, case_id: &str) -> Result<Vec<CarvedFileRecord>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT filename, file_type, size_bytes, confidence, offset_bytes, output_path, category, sha256 
             FROM carved_files WHERE case_id = ?1"
        ).map_err(|e| CoreError::Disk(e.to_string()))?;

        let file_iter = stmt
            .query_map(params![case_id], |row| {
                Ok(CarvedFileRecord {
                    filename: row.get(0)?,
                    file_type: row.get(1)?,
                    size_bytes: row.get(2)?,
                    confidence: row.get(3)?,
                    offset_bytes: row.get(4)?,
                    output_path: row.get(5)?,
                    category: row.get(6)?,
                    sha256: row.get(7)?,
                })
            })
            .map_err(|e| CoreError::Disk(e.to_string()))?;

        let mut files = Vec::new();
        for file in file_iter {
            files.push(file.map_err(|e| CoreError::Disk(e.to_string()))?);
        }
        Ok(files)
    }

    pub fn list_carved_files_by_category(
        &self,
        case_id: &str,
        category: &str,
    ) -> Result<Vec<CarvedFileRecord>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT filename, file_type, size_bytes, confidence, offset_bytes, output_path, category, sha256 
             FROM carved_files WHERE case_id = ?1 AND category = ?2"
        ).map_err(|e| CoreError::Disk(e.to_string()))?;

        let file_iter = stmt
            .query_map(params![case_id, category], |row| {
                Ok(CarvedFileRecord {
                    filename: row.get(0)?,
                    file_type: row.get(1)?,
                    size_bytes: row.get(2)?,
                    confidence: row.get(3)?,
                    offset_bytes: row.get(4)?,
                    output_path: row.get(5)?,
                    category: row.get(6)?,
                    sha256: row.get(7)?,
                })
            })
            .map_err(|e| CoreError::Disk(e.to_string()))?;

        let mut files = Vec::new();
        for file in file_iter {
            files.push(file.map_err(|e| CoreError::Disk(e.to_string()))?);
        }
        Ok(files)
    }

    pub fn insert_audit_entry(
        &self,
        case_id: &str,
        entry_json: &str,
        entry_hash: &str,
    ) -> Result<(), CoreError> {
        // All newly written production events are formal chained AuditEntry
        // values. Historic raw JSON remains readable but cannot be described
        // as a verified hash chain and is intentionally not mixed into one.
        let existing = self.get_audit_entries(case_id)?;
        let mut parsed_entries = Vec::with_capacity(existing.len());
        for stored in &existing {
            let entry = serde_json::from_str::<crate::audit::schema::AuditEntry>(stored)
                .map_err(|_| CoreError::InvalidStructure(format!(
                    "Case {} contains legacy unchained audit data; start a new case before appending verifiable events",
                    case_id
                )))?;
            parsed_entries.push(entry);
        }
        let mut chain = if parsed_entries.is_empty() {
            crate::audit::hashchain::HashChain::new()
        } else {
            crate::audit::hashchain::HashChain::from_entries(parsed_entries)?
        };

        let formal_entry =
            match serde_json::from_str::<crate::audit::schema::AuditEntry>(entry_json) {
                Ok(entry) => {
                    // A caller may provide a prebuilt formal entry (the public API
                    // supports that for import/testing), but it must already carry
                    // the expected next sequence and link. Never trust the hash
                    // column alone as proof of integrity.
                    if entry.seq != chain.len() as u64 || entry.prev_hash != chain.tip() {
                        return Err(CoreError::InvalidStructure(
                            "Provided audit entry does not link to the current chain tip"
                                .to_string(),
                        ));
                    }
                    let candidate = crate::audit::hashchain::HashChain::from_entries({
                        let mut entries = chain.entries().to_vec();
                        entries.push(entry.clone());
                        entries
                    })?;
                    if !entry_hash.is_empty() && entry.entry_hash != entry_hash {
                        return Err(CoreError::InvalidStructure(
                            "Provided audit entry hash does not match the stored hash argument"
                                .to_string(),
                        ));
                    }
                    // `candidate` being constructed is the verification step.
                    let _ = candidate;
                    entry
                }
                Err(_) => {
                    let payload: serde_json::Value = serde_json::from_str(entry_json)
                        .unwrap_or_else(|_| serde_json::Value::String(entry_json.to_string()));
                    let event_name = payload
                        .get("event")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("audit_event")
                        .to_string();
                    let (target, operation_name) = self.case_audit_context(case_id)?;
                    let mut params = BTreeMap::new();
                    params.insert(
                        "event".to_string(),
                        serde_json::Value::String(event_name.clone()),
                    );
                    params.insert("payload".to_string(), payload);
                    let event_lower = event_name.to_ascii_lowercase();
                    let success = !event_lower.contains("failed")
                        && !event_lower.contains("cancelled")
                        && !event_lower.contains("aborted");
                    let entry = crate::audit::schema::AuditEntry {
                        id: chain.len() as u64 + 1,
                        seq: 0,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        operation: audit_operation_type(&operation_name),
                        target,
                        params,
                        result: crate::audit::schema::OperationResult {
                            success,
                            message: event_name,
                            pre_hash: None,
                            post_hash: None,
                            sectors_processed: None,
                            bad_sectors: None,
                            files_recovered: None,
                            entropy_post: None,
                        },
                        prev_hash: String::new(),
                        entry_hash: String::new(),
                    };
                    chain.append(entry)?;
                    chain.entries().last().cloned().ok_or(CoreError::Unknown)?
                }
            };
        let stored_json = serde_json::to_string(&formal_entry)
            .map_err(|error| CoreError::Parse(error.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO audit_entries (case_id, entry_json, entry_hash) VALUES (?1, ?2, ?3)",
                params![case_id, stored_json, formal_entry.entry_hash],
            )
            .map_err(|e| CoreError::Disk(e.to_string()))?;
        Ok(())
    }

    fn case_audit_context(&self, case_id: &str) -> Result<(String, String), CoreError> {
        self.conn
            .query_row(
                "SELECT target, COALESCE(NULLIF(operation, ''), operation_type, 'unknown') FROM cases WHERE id = ?1",
                params![case_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| CoreError::Disk(error.to_string()))
    }

    pub fn get_audit_entries(&self, case_id: &str) -> Result<Vec<String>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT entry_json FROM audit_entries WHERE case_id = ?1 ORDER BY id ASC")
            .map_err(|e| CoreError::Disk(e.to_string()))?;

        let entry_iter = stmt
            .query_map(params![case_id], |row| row.get::<_, String>(0))
            .map_err(|e| CoreError::Disk(e.to_string()))?;

        let mut entries = Vec::new();
        for entry in entry_iter {
            entries.push(entry.map_err(|e| CoreError::Disk(e.to_string()))?);
        }
        Ok(entries)
    }

    /// Reconstructs and cryptographically verifies the hashchain for a case from its stored audit entries.
    /// Returns:
    /// - `Ok(Some(chain))` if all entries parse as formal `AuditEntry` and pass cryptographic chain verification.
    /// - `Ok(None)` if no entries exist or entries are unchained event payloads.
    /// - `Err(CoreError::InvalidStructure(...))` if entries are formal `AuditEntry`s but have been tampered with.
    pub fn get_verified_hashchain(
        &self,
        case_id: &str,
    ) -> Result<Option<crate::audit::hashchain::HashChain>, CoreError> {
        let raw_entries = self.get_audit_entries(case_id)?;
        if raw_entries.is_empty() {
            return Ok(None);
        }

        let mut parsed_entries = Vec::with_capacity(raw_entries.len());
        for entry_str in &raw_entries {
            match serde_json::from_str::<crate::audit::schema::AuditEntry>(entry_str) {
                Ok(entry) => parsed_entries.push(entry),
                Err(_) => return Ok(None),
            }
        }

        let chain = crate::audit::hashchain::HashChain::from_entries(parsed_entries)?;
        Ok(Some(chain))
    }
}

fn audit_operation_type(operation: &str) -> crate::audit::schema::OperationType {
    let operation = operation.to_ascii_lowercase();
    if operation.contains("recover") || operation.contains("carv") {
        crate::audit::schema::OperationType::RecoveryScan
    } else if operation.contains("shred") || operation.contains("file") {
        crate::audit::schema::OperationType::FileWipe
    } else if operation.contains("wipe")
        || operation.contains("eras")
        || operation.contains("purge")
    {
        crate::audit::schema::OperationType::DiskWipe
    } else {
        crate::audit::schema::OperationType::HashVerification
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaseRecord {
    pub id: String,
    pub created_at: String,
    pub operator: String,
    pub target: String,
    pub operation: String,
    pub status: String,
    pub report_path: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CarvedFileRecord {
    pub filename: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub confidence: u8,
    pub offset_bytes: u64,
    pub output_path: String,
    pub category: String,
    pub sha256: Option<String>,
}
