pub mod jpeg;
pub mod pdf;
pub mod png;
pub mod sqlite;
pub mod zip;

/// Trims the buffer of a carved file to its exact logical boundary based on
/// footer signatures or structural parsers (PNG chunks, JPEG markers, PDF %%EOF,
/// ZIP central directory/EOCD, SQLite B-tree page counts).
///
/// Falls back to the full `data.len()` if the file type does not declare a footer
/// or structural sizing cannot be determined.
pub fn trim_carved_file_boundary(
    extension: &str,
    data: &[u8],
    footer_bytes: Option<&[u8]>,
) -> usize {
    // 1. Prioritize format-aware structural parsers first (they understand headers, chunks, and internal markers)
    match extension.to_ascii_lowercase().as_str() {
        "zip" | "docx" | "xlsx" | "pptx" | "apk" | "jar" => {
            if let Some(m) = zip::parse_zip_metadata(data) {
                if m.exact_archive_size > 0 && (m.exact_archive_size as usize) <= data.len() {
                    return m.exact_archive_size as usize;
                }
            }
        }
        "sqlite" | "db" | "sqlite3" => {
            if let Some(m) = sqlite::parse_sqlite_metadata(data) {
                if m.exact_db_size > 0 && (m.exact_db_size as usize) <= data.len() {
                    return m.exact_db_size as usize;
                }
            }
        }
        "png" => {
            if let Some(m) = png::parse_png_metadata(data) {
                if m.exact_file_size > 0 && (m.exact_file_size as usize) <= data.len() {
                    return m.exact_file_size as usize;
                }
            }
        }
        "jpg" | "jpeg" => {
            if let Some(m) = jpeg::parse_jpeg_metadata(data) {
                if m.exact_file_size > 0 && (m.exact_file_size as usize) <= data.len() {
                    return m.exact_file_size as usize;
                }
            }
        }
        "pdf" => {
            if let Some(m) = pdf::parse_pdf_metadata(data) {
                if m.exact_file_size > 0 && (m.exact_file_size as usize) <= data.len() {
                    return m.exact_file_size as usize;
                }
            }
        }
        _ => {}
    }

    // 2. Fallback to magic footer delimiter
    if let Some(footer) = footer_bytes {
        if !footer.is_empty() {
            if let Some(pos) = data.windows(footer.len()).position(|w| w == footer) {
                return pos + footer.len();
            }
        }
    }

    data.len()
}
