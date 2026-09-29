//! Recovered file classification and duplicate detection.
//!
//! After carving, files are automatically categorized into:
//! Documents, Media, Archives, Databases, System, Unknown.
//!
//! Image files are deduplicated using perceptual hashing (dHash)
//! to group visually identical files regardless of compression.

pub mod dhash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileCategory {
    Media,
    Documents,
    Archives,
    Databases,
    Executables,
    System,
    Unknown,
}

impl FileCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileCategory::Media => "Media",
            FileCategory::Documents => "Documents",
            FileCategory::Archives => "Archives",
            FileCategory::Databases => "Databases",
            FileCategory::Executables => "Executables",
            FileCategory::System => "System",
            FileCategory::Unknown => "Unknown",
        }
    }
}

/// Categorizes a filename extension into its forensic classification category.
pub fn categorize_extension(ext: &str) -> FileCategory {
    match ext
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "mp4" | "mkv" | "avi" | "mp3"
        | "wav" | "flac" => FileCategory::Media,
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "csv" | "md" | "rtf" => {
            FileCategory::Documents
        }
        "zip" | "tar" | "gz" | "7z" | "bz2" | "xz" | "rar" | "iso" => FileCategory::Archives,
        "sqlite" | "db" | "sql" | "sqlite3" | "mdb" => FileCategory::Databases,
        "exe" | "dll" | "so" | "bin" | "elf" => FileCategory::Executables,
        "sys" | "ini" | "cfg" | "log" => FileCategory::System,
        _ => FileCategory::Unknown,
    }
}
