use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    #[serde(rename = "magic")]
    Magic,
    #[serde(rename = "structure")]
    Structure,
    #[serde(rename = "filesystem")]
    Filesystem,
    #[serde(rename = "entropy")]
    Entropy,
    #[serde(rename = "hybrid")]
    Hybrid,
}

impl RecoveryStrategy {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Magic => "Signature Magic (Header/Footer)",
            Self::Structure => "Deep Structural Syntax Carving",
            Self::Filesystem => "Filesystem Metadata (MFT/FAT)",
            Self::Entropy => "Statistical Entropy Clustering",
            Self::Hybrid => "Hybrid Multi-Strategy Ensemble",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Magic => "Scans sector boundaries for known magic byte headers and matching footer delimiters.",
            Self::Structure => "Parses internal container chunks (ZIP Central Directory, SQLite B-Tree, PNG/JPEG markers).",
            Self::Filesystem => "Recovers deleted NTFS $MFT records and FAT directory entries to restore original filenames and timestamps.",
            Self::Entropy => "Classifies continuous statistical Shannon entropy blocks for headerless and fragmented data streams.",
            Self::Hybrid => "Ensemble combining all techniques for maximum file discovery, metadata reconstruction, and confidence scoring.",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "magic" | "signature" | "headers" => Self::Magic,
            "structure" | "syntax" | "chunks" => Self::Structure,
            "filesystem" | "fs" | "mft" | "fat" | "inode" => Self::Filesystem,
            "entropy" | "statistical" | "cluster" => Self::Entropy,
            _ => Self::Hybrid,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCarvedArtifact {
    pub offset_bytes: u64,
    pub size_bytes: u64,
    pub file_type: String,
    pub confidence: u8,
    pub original_filename: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub recovery_method: String,
    pub metadata_summary: String,
}

impl UnifiedCarvedArtifact {
    pub fn from_fs_metadata(
        offset: u64,
        meta: &crate::carver::filesystem_records::CarvedFsMetadata,
    ) -> Self {
        Self {
            offset_bytes: offset,
            size_bytes: meta.file_size_bytes,
            file_type: meta.file_extension.clone(),
            confidence: 90,
            original_filename: Some(meta.filename.clone()),
            created_at: meta.created_at.clone(),
            modified_at: meta.modified_at.clone(),
            recovery_method: format!("Filesystem ({})", meta.filesystem_type),
            metadata_summary: format!(
                "{} file, resident: {}",
                meta.filesystem_type, meta.is_resident
            ),
        }
    }

    pub fn from_entropy_cluster(
        cluster: &crate::carver::entropy_clustering::EntropyCluster,
    ) -> Self {
        let (file_type, type_label) = match cluster.cluster_type {
            crate::carver::entropy_clustering::EntropyType::CompressedOrEncrypted => {
                ("bin", "compressed/encrypted")
            }
            crate::carver::entropy_clustering::EntropyType::BinaryExecutable => {
                ("bin", "binary/code")
            }
            crate::carver::entropy_clustering::EntropyType::StructuredText => {
                ("txt", "structured text")
            }
            _ => ("bin", "unknown stream"),
        };
        Self {
            offset_bytes: cluster.start_offset,
            size_bytes: cluster.size_bytes,
            file_type: file_type.to_string(),
            confidence: 60,
            original_filename: None,
            created_at: None,
            modified_at: None,
            recovery_method: "Entropy Clustering".to_string(),
            metadata_summary: format!(
                "{}, avg entropy: {:.2} bits/byte",
                type_label, cluster.avg_entropy
            ),
        }
    }
}
