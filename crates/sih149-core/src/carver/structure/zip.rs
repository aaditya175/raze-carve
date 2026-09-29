use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ZipCarveMetadata {
    pub file_count: usize,
    pub exact_archive_size: u64,
    pub contained_files: Vec<String>,
    pub total_uncompressed_bytes: u64,
    pub document_type: Option<String>,
}

/// Validates ZIP structure from a buffer.
pub fn validate_zip(data: &[u8]) -> Result<bool> {
    if data.len() < 22 {
        return Ok(false);
    }

    // ZIP Magic Header (Local File Header PK\x03\x04)
    if data[0] != 0x50 || data[1] != 0x4B || data[2] != 0x03 || data[3] != 0x04 {
        return Ok(false);
    }

    let eocd_sig = [0x50, 0x4B, 0x05, 0x06];
    for i in (0..=data.len().saturating_sub(22)).rev() {
        if data[i..i + 4] == eocd_sig {
            let cd_offset =
                u32::from_le_bytes([data[i + 16], data[i + 17], data[i + 18], data[i + 19]])
                    as usize;
            let total_entries = u16::from_le_bytes([data[i + 10], data[i + 11]]) as usize;
            let comment_len = u16::from_le_bytes([data[i + 20], data[i + 21]]) as usize;
            if i + 22 + comment_len <= data.len()
                && cd_offset <= i
                && (total_entries == 0
                    || (cd_offset + 4 <= i
                        && data[cd_offset..cd_offset + 4] == [0x50, 0x4B, 0x01, 0x02]))
            {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Deep structural parser for ZIP/DOCX/XLSX/PPTX/APK archives.
/// Reconstructs the internal directory tree, filenames, uncompressed sizes,
/// and exact file boundary by parsing the Central Directory structures.
pub fn parse_zip_metadata(data: &[u8]) -> Option<ZipCarveMetadata> {
    if data.len() < 22 {
        return None;
    }
    if data[0] != 0x50 || data[1] != 0x4B || data[2] != 0x03 || data[3] != 0x04 {
        return None;
    }

    let eocd_sig = [0x50, 0x4B, 0x05, 0x06];
    let mut eocd_offset = None;
    for i in (0..=data.len().saturating_sub(22)).rev() {
        if data[i..i + 4] == eocd_sig {
            let cd_offset =
                u32::from_le_bytes([data[i + 16], data[i + 17], data[i + 18], data[i + 19]])
                    as usize;
            let total_entries = u16::from_le_bytes([data[i + 10], data[i + 11]]) as usize;
            let comment_len = u16::from_le_bytes([data[i + 20], data[i + 21]]) as usize;
            if i + 22 + comment_len <= data.len()
                && cd_offset <= i
                && (total_entries == 0
                    || (cd_offset + 4 <= i
                        && data[cd_offset..cd_offset + 4] == [0x50, 0x4B, 0x01, 0x02]))
            {
                eocd_offset = Some(i);
                break;
            }
        }
    }

    let eocd = eocd_offset?;
    let comment_len = u16::from_le_bytes([data[eocd + 20], data[eocd + 21]]) as usize;
    let exact_archive_size = (eocd + 22 + comment_len) as u64;

    let total_entries = u16::from_le_bytes([data[eocd + 10], data[eocd + 11]]) as usize;
    let cd_size = u32::from_le_bytes([
        data[eocd + 12],
        data[eocd + 13],
        data[eocd + 14],
        data[eocd + 15],
    ]) as usize;
    let cd_offset = u32::from_le_bytes([
        data[eocd + 16],
        data[eocd + 17],
        data[eocd + 18],
        data[eocd + 19],
    ]) as usize;

    let mut contained_files = Vec::new();
    let mut total_uncompressed = 0u64;

    if cd_offset + cd_size <= eocd && cd_offset + cd_size <= data.len() {
        let mut ptr = cd_offset;
        let cd_end = cd_offset + cd_size;

        while ptr + 46 <= cd_end && contained_files.len() < 50 {
            // Central directory file header signature: PK\x01\x02
            if data[ptr..ptr + 4] != [0x50, 0x4B, 0x01, 0x02] {
                break;
            }

            let uncompressed_size = u32::from_le_bytes([
                data[ptr + 24],
                data[ptr + 25],
                data[ptr + 26],
                data[ptr + 27],
            ]) as u64;
            total_uncompressed += uncompressed_size;

            let fname_len = u16::from_le_bytes([data[ptr + 28], data[ptr + 29]]) as usize;
            let extra_len = u16::from_le_bytes([data[ptr + 30], data[ptr + 31]]) as usize;
            let comment_len = u16::from_le_bytes([data[ptr + 32], data[ptr + 33]]) as usize;

            let fname_start = ptr + 46;
            if fname_start + fname_len <= cd_end {
                if let Ok(fname) = std::str::from_utf8(&data[fname_start..fname_start + fname_len])
                {
                    contained_files.push(fname.to_string());
                }
            }

            let entry_len = match 46usize
                .checked_add(fname_len)
                .and_then(|l| l.checked_add(extra_len))
                .and_then(|l| l.checked_add(comment_len))
            {
                Some(len) if len > 0 => len,
                _ => break,
            };

            ptr += entry_len;
        }
    }

    let mut doc_type = None;
    for f in &contained_files {
        if f.starts_with("word/") {
            doc_type = Some("Microsoft Word Document (.docx)".to_string());
            break;
        } else if f.starts_with("xl/") {
            doc_type = Some("Microsoft Excel Spreadsheet (.xlsx)".to_string());
            break;
        } else if f.starts_with("ppt/") {
            doc_type = Some("Microsoft PowerPoint (.pptx)".to_string());
            break;
        } else if f == "AndroidManifest.xml" {
            doc_type = Some("Android Application Package (.apk)".to_string());
            break;
        } else if f.ends_with(".class") {
            doc_type = Some("Java Archive (.jar)".to_string());
            break;
        }
    }

    Some(ZipCarveMetadata {
        file_count: total_entries,
        exact_archive_size,
        contained_files,
        total_uncompressed_bytes: total_uncompressed,
        document_type: doc_type,
    })
}
