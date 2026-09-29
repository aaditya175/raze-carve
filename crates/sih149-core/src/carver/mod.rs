//! Advanced forensic file carving and recovery engine.
//!
//! Recovers deleted files from formatted, corrupted, or raw media
//! using multiple carving strategies:
//! - **Signature-based**: Header/footer magic byte matching
//! - **Structure-based**: Internal file structure & chunk validation
//! - **Filesystem-based**: Inode, NTFS $MFT, and FAT directory entry carving
//! - **Entropy-based**: Statistical block classification and clustering
//! - **Hybrid Ensemble**: Combined multi-strategy extraction with metadata fusion
//!
//! Operates in strict read-only mode to preserve evidential integrity.

pub mod confidence;
pub mod engine;
pub mod entropy;
pub mod entropy_clustering;
pub mod filesystem_records;
pub mod recovery_modes;
pub mod scanner;
pub mod signatures;
pub mod structure;
