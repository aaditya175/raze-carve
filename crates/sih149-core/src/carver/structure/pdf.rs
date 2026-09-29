use crate::error::Result;

#[derive(Debug, Clone)]
pub struct PdfCarveMetadata {
    pub version: String,
    pub exact_file_size: u64,
    pub title: Option<String>,
    pub author: Option<String>,
    pub creation_date: Option<String>,
}

/// Validates PDF structure from a buffer.
pub fn validate_pdf(data: &[u8]) -> Result<bool> {
    if data.len() < 9 {
        return Ok(false);
    }

    if &data[0..5] != b"%PDF-" {
        return Ok(false);
    }

    let eof_sig = b"%%EOF";
    for i in (0..=data.len().saturating_sub(eof_sig.len())).rev() {
        if &data[i..i + eof_sig.len()] == eof_sig {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Deep structural parser for PDF documents.
/// Locates exact EOF position and extracts document metadata (/Title, /Author).
pub fn parse_pdf_metadata(data: &[u8]) -> Option<PdfCarveMetadata> {
    if data.len() < 9 || &data[0..5] != b"%PDF-" {
        return None;
    }

    let version = String::from_utf8_lossy(&data[5..8]).to_string();

    let eof_sig = b"%%EOF";
    let mut exact_file_size = 0u64;

    for i in (0..=data.len().saturating_sub(eof_sig.len())).rev() {
        if &data[i..i + eof_sig.len()] == eof_sig {
            // Find end of line after %%EOF
            let mut end_pos = i + eof_sig.len();
            while end_pos < data.len() && (data[end_pos] == b'\r' || data[end_pos] == b'\n') {
                end_pos += 1;
            }
            exact_file_size = end_pos as u64;
            break;
        }
    }

    if exact_file_size == 0 {
        return None;
    }

    let end_limit = std::cmp::min(exact_file_size as usize, data.len());
    let head_chunk = &data[..std::cmp::min(end_limit, 65536)];
    let tail_chunk = if end_limit > 65536 {
        let start = end_limit.saturating_sub(65536);
        &data[start..end_limit]
    } else {
        &[][..]
    };

    let head_str = String::from_utf8_lossy(head_chunk);
    let tail_str = if !tail_chunk.is_empty() {
        Some(String::from_utf8_lossy(tail_chunk))
    } else {
        None
    };

    let extract_field = |tag: &str| -> Option<String> {
        let find_in_str = |s: &str| -> Option<String> {
            if let Some(pos) = s.find(tag) {
                let rest = &s[pos + tag.len()..];
                if let Some(start) = rest.find('(') {
                    if let Some(end) = rest[start + 1..].find(')') {
                        let val = rest[start + 1..start + 1 + end].trim();
                        if !val.is_empty() {
                            return Some(val.to_string());
                        }
                    }
                }
            }
            None
        };

        // Check head first, then tail (where standard trailer /Info dict resides)
        find_in_str(&head_str).or_else(|| tail_str.as_ref().and_then(|ts| find_in_str(ts)))
    };

    let title = extract_field("/Title");
    let author = extract_field("/Author");
    let creation_date = extract_field("/CreationDate");

    Some(PdfCarveMetadata {
        version,
        exact_file_size,
        title,
        author,
        creation_date,
    })
}
