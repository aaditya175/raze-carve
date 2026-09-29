use crate::error::Result;

#[derive(Debug, Clone)]
pub struct SqliteCarveMetadata {
    pub page_size: u32,
    pub page_count: u32,
    pub exact_db_size: u64,
    pub table_names: Vec<String>,
}

/// Validates SQLite database structure from a buffer.
pub fn validate_sqlite(data: &[u8]) -> Result<bool> {
    if data.len() < 100 {
        return Ok(false);
    }

    let magic = b"SQLite format 3\0";
    if &data[0..16] != magic {
        return Ok(false);
    }

    let page_size_raw = u16::from_be_bytes([data[16], data[17]]) as u32;
    let page_size = if page_size_raw == 1 {
        65536
    } else {
        page_size_raw
    };
    if page_size < 512 || !page_size.is_power_of_two() {
        return Ok(false);
    }

    // Reserved space at end of each page (byte 20) must not exceed 64
    if data[20] > 64 {
        return Ok(false);
    }

    // Check write/read version numbers (bytes 18, 19): 1=legacy, 2=WAL
    if data[18] > 2 || data[19] > 2 {
        return Ok(false);
    }

    if !(data.len() as u32).is_multiple_of(page_size) {
        if data.len() as u32 >= page_size {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Deep structural parser for SQLite databases.
/// Extracts page geometry, in-header database size, and table names from page 1.
pub fn parse_sqlite_metadata(data: &[u8]) -> Option<SqliteCarveMetadata> {
    if data.len() < 100 {
        return None;
    }
    let magic = b"SQLite format 3\0";
    if &data[0..16] != magic {
        return None;
    }

    let page_size_raw = u16::from_be_bytes([data[16], data[17]]) as u32;
    let page_size = if page_size_raw == 1 {
        65536
    } else {
        page_size_raw
    };
    if page_size < 512 || (page_size & (page_size - 1)) != 0 {
        return None;
    }

    let in_header_pages = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);
    let page_count = if in_header_pages > 0 {
        in_header_pages
    } else {
        // When in-header page count is 0 (WAL mode or in-flight uncommitted transaction),
        // determine actual page count by scanning contiguous valid SQLite pages,
        // stopping at zero-padded slack. Default to at least 1 page.
        let mut count = 1u32;
        let max_pages = (data.len() as u32) / page_size;
        while count < max_pages {
            let page_offset = (count as usize) * (page_size as usize);
            if page_offset + (page_size as usize) > data.len() {
                break;
            }
            let page_bytes = &data[page_offset..page_offset + (page_size as usize)];
            // SQLite B-tree pages start with a valid page type flag:
            // 0x02 (interior index), 0x05 (interior table), 0x0a (leaf index), 0x0d (leaf table)
            let flag = page_bytes[0];
            let is_btree = matches!(flag, 0x02 | 0x05 | 0x0a | 0x0d);

            if is_btree {
                let cell_content = u16::from_be_bytes([page_bytes[5], page_bytes[6]]) as u32;
                let valid_cell_content = cell_content == 0 || cell_content <= page_size;
                if valid_cell_content {
                    count += 1;
                    continue;
                }
            }

            // If the page is all zeros (common for wiped drive slack following DB), stop
            if page_bytes.iter().all(|&b| b == 0) {
                break;
            }

            // Non-zero page that could be freelist or overflow
            count += 1;
        }
        count.max(1)
    };

    let exact_db_size = (page_count as u64) * (page_size as u64);

    // Scan page 1 for table definitions: "CREATE TABLE <name>"
    let mut table_names = Vec::new();
    let page1_len = std::cmp::min(data.len(), page_size as usize);
    let page1_str = String::from_utf8_lossy(&data[100..page1_len]);

    for token in page1_str.split("CREATE TABLE ") {
        if let Some(name_end) = token.find(['(', ' ', '\n']) {
            let candidate = token[..name_end].trim().trim_matches('"').trim_matches('`');
            if !candidate.is_empty()
                && candidate.chars().all(|c| c.is_alphanumeric() || c == '_')
                && !table_names.contains(&candidate.to_string())
                && table_names.len() < 20
            {
                table_names.push(candidate.to_string());
            }
        }
    }

    Some(SqliteCarveMetadata {
        page_size,
        page_count,
        exact_db_size,
        table_names,
    })
}
